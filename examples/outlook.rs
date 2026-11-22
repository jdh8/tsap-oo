//! The table model of `doc/expectation.md`, batch 2: the four constants of
//! an [`Outlook`] and the public cards per draw, measured from the event
//! logs of `HeuristicBot` self-play, then the win chances the DP predicts
//! with them checked against the wins that follow.
//!
//! Every public card — a draw, or a discard from a hand — is counted once
//! for each seat watching it, and as gated when that seat holds 10
//! concealed cards or fewer.  A segment is the discards before a draw and
//! the draw, counted only when the round reaches the draw (or, for the
//! last, the exhausted pile).
//!
//! The calibration plays fresh deals, seed + 1, and at each discard that
//! leaves 10 cards or fewer predicts the hand kept, by [`Graph::values`]:
//! its chance to win and its points, under three outlooks that share the
//! graph — one number for h and the cards a draw, a table for h, a table
//! for both.
//!
//! The generals' rate (`doc/generals.md`, batch 1) is ρ, the generals
//! among the public cards over their sum of K ÷ N, K the generals and N
//! the cards the watching seat has not seen, rebuilt from its dealt hand
//! and the events: `Graph` takes a public card for a general at K ÷ N.
//!
//! With 0 rounds it measures nothing and calibrates [`Outlook::new`]
//! and three changes to it, one constant each.
//!
//! ```sh
//! cargo run --release --example outlook -- [rounds] [seed] [calibration rounds]
//! ```
//!
//! `THRESHOLD` (default 10) is the 胡 to win: 8 is 八胡.  `PLAYERS` (default
//! 4) is the table's size.
use tsap_oo::susik::{Hand, Meld, Rank};
use tsap_oo::{
    Event, Graph, HeuristicBot, Outlook, Payoff, Phase, Prune, Rules, Seat, Step, Strategy, Table,
    View,
};

mod common;

/// The concealed cards from which the DP plays
const GATE: usize = 10;

/// Draws in a round: the pile after the deal less the dead tail
const fn draws(rules: &Rules) -> usize {
    112 - (20 * rules.players + 1) - rules.dead()
}

/// Events of `n` trials
#[derive(Default, Clone, Copy)]
struct Count {
    n: u64,
    k: u64,
}

impl Count {
    fn add(&mut self, hit: bool) {
        self.n += 1;
        self.k += u64::from(hit);
    }

    fn rate(self) -> f64 {
        self.k as f64 / self.n.max(1) as f64
    }
}

/// Generals that came up against the generals expected
#[derive(Default, Clone, Copy)]
struct Rho {
    n: u64,
    expected: f64,
    actual: u64,
}

impl Rho {
    fn add(&mut self, expected: f64, general: bool) {
        self.n += 1;
        self.expected += expected;
        self.actual += u64::from(general);
    }

    fn merge(self, other: Self) -> Self {
        Self {
            n: self.n + other.n,
            expected: self.expected + other.expected,
            actual: self.actual + other.actual,
        }
    }

    fn rate(self) -> f64 {
        self.actual as f64 / self.expected
    }

    /// The standard error of the rate, each card a Bernoulli trial at its
    /// own chance
    fn se(self) -> f64 {
        let p = self.actual as f64 / self.n as f64;
        (self.n as f64 * p * (1.0 - p)).sqrt() / self.expected
    }
}

/// Watching seats, split by the gate
#[derive(Default, Clone, Copy)]
struct Split {
    all: Count,
    gated: Count,
}

impl Split {
    fn add(&mut self, gated: bool, hit: bool) {
        self.all.add(hit);
        if gated {
            self.gated.add(hit);
        }
    }
}

#[derive(Default)]
struct Tally {
    /// Seats at the table
    players: usize,
    /// Draws in a round
    span: usize,
    /// Another seat wins on the card, by draws left before it
    end: Vec<Split>,
    /// The same, gated, by the fewest concealed cards of another seat
    end_by_shortest: Vec<Count>,
    /// Completed segments and their public cards, by draws left
    draws: Vec<u64>,
    cards: Vec<u64>,
    /// A card other than a general comes to the seat to eat
    eat: Split,
    /// A general is the seat's own draw
    general: Split,
    /// 胡 another seat wins at: (sum, count), all and gated
    loss: [(u64, u64); 2],
    /// The generals' rate by draws left: all, gated, and gated less the
    /// seat's own discards
    rho: Vec<[Rho; 3]>,
    /// Every rank's rate, gated, by [`phase`] and rank: with the seat's
    /// own discards and without
    ranks: [[[Rho; 2]; 7]; 3],
}

/// The draws left of `doc/generals.md`: 15 or more, 8 to 14, 7 or fewer
const fn phase(left: usize) -> usize {
    match left {
        15.. => 0,
        8.. => 1,
        _ => 2,
    }
}

impl Tally {
    fn new(rules: &Rules) -> Self {
        let span = draws(rules);
        Self {
            players: rules.players,
            span,
            end: vec![Split::default(); span + 1],
            end_by_shortest: vec![Count::default(); 22],
            draws: vec![0; span + 1],
            cards: vec![0; span + 1],
            rho: vec![[Rho::default(); 3]; span + 1],
            ..Self::default()
        }
    }

    /// One finished round's events, from the hands dealt
    fn round(&mut self, events: &[Event], dealer: Seat, dealt: [Hand; 4]) {
        let mut concealed = Seat::ALL.map(|seat| if seat == dealer { 21 } else { 20 });
        let seats = &Seat::ALL[..self.players];
        // What each seat has not seen
        let mut unseen = dealt.map(|hand| Hand::FULL - hand);
        // The cards of a meld that come from the hand
        let reveal = |unseen: &mut [Hand; 4], by: Seat, meld: Meld, card| {
            for &watcher in seats.iter().filter(|&&w| w != by) {
                unseen[watcher.index()] = unseen[watcher.index()] - meld.cards().without(card);
            }
        };
        let (mut drawn, mut pending) = (0, 0);
        for (i, &event) in events.iter().enumerate() {
            let (source, card, draw) = match event {
                Event::Draw { seat, card } => (seat, card, true),
                Event::Discard { seat, card } => {
                    if i > 0 && events[i - 1] == (Event::Draw { seat, card }) {
                        continue;
                    }
                    concealed[seat.index()] -= 1;
                    (seat, card, false)
                }
                Event::Eat { seat, meld, card } => {
                    concealed[seat.index()] -= meld.size() - 1;
                    reveal(&mut unseen, seat, meld, card);
                    continue;
                }
                Event::Pong { seat, card } => {
                    concealed[seat.index()] -= 2;
                    reveal(&mut unseen, seat, Meld::Triplet(card), card);
                    continue;
                }
                Event::Kong { seat, card } => {
                    concealed[seat.index()] -= 3;
                    reveal(&mut unseen, seat, Meld::Quad(card), card);
                    continue;
                }
                Event::Exhausted => {
                    self.draws[0] += 1;
                    self.cards[0] += pending;
                    break;
                }
                Event::Win { .. } | Event::Foul { .. } => break,
            };
            let left = self.span - drawn;
            if draw {
                self.draws[left] += 1;
                self.cards[left] += pending + 1;
                drawn += 1;
                pending = 0;
            } else {
                pending += 1;
            }
            let next = events.get(i + 1).copied();
            let winner = match next {
                Some(Event::Win { seat, hu }) => Some((seat, hu)),
                _ => None,
            };
            // Whoever took the card away from an eater
            let blocks = |watcher: Seat| match next {
                Some(Event::Pong { seat, .. } | Event::Kong { seat, .. }) => seat != watcher,
                Some(Event::Eat { seat, .. }) => draw && seat == source && seat != watcher,
                _ => false,
            };
            let general = card.rank == Rank::General;
            for &watcher in seats {
                let gated = concealed[watcher.index()] <= GATE;
                let live = &mut unseen[watcher.index()];
                let mut by_rank = [0.0; 7];
                for (c, n) in live.distinct() {
                    by_rank[c.rank as usize] += f64::from(n) / live.len() as f64;
                }
                // A seat's own discard is a card it has seen
                let own = !draw && watcher == source;
                for (j, rho) in self.rho[left].iter_mut().enumerate() {
                    if j == 0 || (gated && (j == 1 || !own)) {
                        rho.add(by_rank[0], general);
                    }
                }
                if gated {
                    let phase = &mut self.ranks[phase(left)];
                    for (rank, (rho, expected)) in phase.iter_mut().zip(by_rank).enumerate() {
                        let hit = card.rank as usize == rank;
                        rho[0].add(expected, hit);
                        if !own {
                            rho[1].add(expected, hit);
                        }
                    }
                }
                if !own {
                    *live = live.without(card);
                }
                let other = winner.filter(|&(seat, _)| seat != watcher);
                self.end[left].add(gated, other.is_some());
                if gated {
                    let shortest = seats
                        .iter()
                        .filter(|&&seat| seat != watcher)
                        .map(|seat| concealed[seat.index()])
                        .min()
                        .expect("another seat");
                    self.end_by_shortest[shortest].add(other.is_some());
                }
                if let Some((_, hu)) = other {
                    for (j, (sum, n)) in self.loss.iter_mut().enumerate() {
                        if j == 0 || gated {
                            *sum += u64::from(hu);
                            *n += 1;
                        }
                    }
                }
                if winner.is_some() {
                    continue;
                }
                if card.rank == Rank::General {
                    self.general.add(gated, draw && source == watcher);
                } else {
                    let place = (draw && source == watcher) || source.next(self.players) == watcher;
                    self.eat.add(gated, place && !blocks(watcher));
                }
            }
        }
    }

    /// Public cards a segment, over all
    fn per_draw(&self) -> f64 {
        self.cards.iter().sum::<u64>() as f64 / self.draws.iter().sum::<u64>() as f64
    }

    /// The tables, h gated; a segment with fewer than `least` gated cards
    /// takes h from the nearest that has them, and one played fewer than
    /// 100 times its cards: a round of two seldom lasts to the last draws
    fn outlook(&self, least: u64) -> Outlook {
        let nearest =
            |full: Vec<usize>| move |k: usize| full.iter().copied().min_by_key(|&j| j.abs_diff(k));
        let all = || 0..=self.span;
        let near = nearest(all().filter(|&k| self.end[k].gated.n >= least).collect());
        let played = nearest(all().filter(|&k| self.draws[k] >= 100).collect());
        let (sum, n) = self.loss[1];
        Outlook {
            end: all()
                .map(|k| near(k).map_or(self.end(), |j| self.end[j].gated.rate()))
                .collect(),
            cards: all()
                .map(|k| {
                    played(k).map_or(self.per_draw(), |j| {
                        self.cards[j] as f64 / self.draws[j] as f64
                    })
                })
                .collect(),
            loss: -(sum as f64) / n as f64,
            eat: self.eat.gated.rate(),
            general: self.general.gated.rate(),
        }
    }

    /// The gated h over all segments
    fn end(&self) -> f64 {
        let (n, k) = self
            .end
            .iter()
            .fold((0, 0), |(n, k), s| (n + s.gated.n, k + s.gated.k));
        k as f64 / n as f64
    }

    fn print(&self) {
        println!("\ndraws left  segments  per draw  h all    h gated  gated cards");
        for left in (0..=self.span).rev() {
            let s = self.end[left];
            println!(
                "{left:>10}  {:8}  {:8.3}  {:.5}  {:.5}  {:11}",
                self.draws[left],
                self.cards[left] as f64 / self.draws[left].max(1) as f64,
                s.all.rate(),
                s.gated.rate(),
                s.gated.n,
            );
        }
        let total = |f: fn(&Split) -> Count| {
            self.end.iter().map(f).fold(Count::default(), |a, c| Count {
                n: a.n + c.n,
                k: a.k + c.k,
            })
        };
        let (all, gated) = (total(|s| s.all), total(|s| s.gated));
        println!("\nh gated, by the fewest concealed cards of another seat");
        for (len, c) in self.end_by_shortest.iter().enumerate() {
            if c.n > 0 {
                println!("{len:>3}  {:.5}  {:8}", c.rate(), c.n);
            }
        }
        let mean = |(sum, n): (u64, u64)| sum as f64 / n as f64;
        println!("\n         all      gated");
        println!("h        {:.5}  {:.5}", all.rate(), gated.rate());
        println!(
            "L        {:<7.3}  {:.3}",
            -mean(self.loss[0]),
            -mean(self.loss[1])
        );
        println!(
            "e        {:.5}  {:.5}",
            self.eat.all.rate(),
            self.eat.gated.rate()
        );
        println!(
            "general  {:.5}  {:.5}",
            self.general.all.rate(),
            self.general.gated.rate()
        );
        println!("per draw {:.4}", self.per_draw());

        println!("\nρ, generals that came up over K ÷ N, ± one standard error");
        println!("draws left  ρ all            ρ gated          gated, not own   gated cards");
        let cell = |r: Rho| format!("{:.3} ± {:.3}", r.rate(), r.se());
        for left in (0..=self.span).rev() {
            let [all, gated, others] = self.rho[left];
            if all.n > 0 {
                println!(
                    "{left:>10}  {:15}  {:15}  {:15}  {:11}",
                    cell(all),
                    cell(gated),
                    cell(others),
                    gated.n
                );
            }
        }
        let names = ["將帥", "士仕", "象相", "車俥", "馬傌", "包炮", "卒兵"];
        for (j, title) in ["with", "less"].iter().enumerate() {
            println!("\nevery rank, gated, {title} the seat's own discards: came up over expected");
            println!("rank  draws 15+        8–14             0–7");
            for (rank, name) in names.iter().enumerate() {
                let [a, b, c] = self.ranks.map(|phase| cell(phase[rank][j]));
                println!("{name}  {a:15}  {b:15}  {c:15}");
            }
        }
        let sum = self.rho.iter().fold([Rho::default(); 3], |a, r| {
            [0, 1, 2].map(|j| a[j].merge(r[j]))
        });
        println!(
            "{:>10}  {:15}  {:15}  {:15}  {:11}",
            "all",
            cell(sum[0]),
            cell(sum[1]),
            cell(sum[2]),
            sum[1].n
        );
    }

    /// ρ gated by draws left; a segment with fewer than `least` gated cards
    /// takes the nearest that has them
    fn generals(&self, least: u64) -> Vec<f64> {
        let full: Vec<_> = (0..=self.span)
            .filter(|&k| self.rho[k][1].n >= least)
            .collect();
        (0..=self.span)
            .map(|k| {
                let j = full.iter().min_by_key(|&&j| j.abs_diff(k));
                self.rho[*j.expect("a segment with enough cards")][1].rate()
            })
            .collect()
    }
}

/// Predictions against outcomes, in one tenth of the predicted chances
#[derive(Default, Clone, Copy)]
struct Bin {
    n: u64,
    chance: f64,
    wins: u64,
    points: f64,
    scored: f64,
    scored2: f64,
}

/// Wilson's 95% interval for `k` of `n`
fn wilson(k: u64, n: u64) -> (f64, f64) {
    let (n, p, z) = (n as f64, k as f64 / n as f64, 1.96_f64);
    let mid = (p + z * z / (2.0 * n)) / (1.0 + z * z / n);
    let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / (1.0 + z * z / n);
    (mid - half, mid + half)
}

fn calibrate(rules: &Rules, outlooks: &[(&str, Outlook)], rounds: u64, seed: u64) {
    let mut bins = vec![[Bin::default(); 10]; outlooks.len()];
    let played = common::deals(*rules, rounds, seed, |round| {
        let bots = Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(rules)) as Box<dyn Strategy>));
        let mut table = Table::new(round, bots);
        // (seat, outlook, predicted chance, predicted points)
        let mut predictions = Vec::new();
        loop {
            let round = table.round();
            let before = round.events().len();
            let kept = if let Phase::Discard(seat) = round.phase()
                && round.hand(seat).len() <= GATE + 1
            {
                let view = View::new(round, seat);
                let base: u32 = view
                    .exposed(seat)
                    .iter()
                    .map(|meld| rules.exposed.score(meld))
                    .sum();
                let shown = Seat::ALL
                    .into_iter()
                    .flat_map(|seat| view.exposed(seat))
                    .flat_map(|meld| meld.cards())
                    .chain(view.discards().iter().map(|&(_, card)| card));
                let live = shown.fold(Hand::FULL - view.hand(), Hand::without);
                Some((
                    seat,
                    view.hand(),
                    base,
                    live,
                    view.pile_len() - rules.dead(),
                ))
            } else {
                None
            };
            let step = table.step().expect("bots act legally");
            if let Some((seat, hand, base, live, draws)) = kept {
                let Event::Discard { card, .. } = table.round().events()[before] else {
                    unreachable!("a discard is logged first");
                };
                let graph =
                    Graph::new(rules, live, &[(hand.without(card), base)], Prune::default());
                let root = graph.roots()[0];
                for (i, (_, outlook)) in outlooks.iter().enumerate() {
                    let chance = graph.values(outlook, draws, Payoff::Win)[root];
                    let points = graph.values(outlook, draws, Payoff::Points)[root];
                    predictions.push((seat, i, chance, points));
                }
            }
            let Step::Acted(_) = step else { break };
        }
        (
            predictions,
            table.round().result().expect("finished").clone(),
        )
    });
    for (predictions, result) in played {
        for (seat, i, chance, points) in predictions {
            let others = (rules.players - 1) as f64;
            let scored = match (result.winner, result.foul) {
                (Some(winner), _) if winner == seat => others * f64::from(result.hu),
                (Some(_), _) => -f64::from(result.hu),
                (None, Some(foul)) if foul == seat => -others * f64::from(rules.threshold),
                (None, Some(_)) => f64::from(rules.threshold),
                (None, None) => 0.0,
            };
            let bin = &mut bins[i][((chance * 10.0) as usize).min(9)];
            bin.n += 1;
            bin.chance += chance;
            bin.wins += u64::from(result.winner == Some(seat));
            bin.points += points;
            bin.scored += scored;
            bin.scored2 += scored * scored;
        }
    }

    println!("\ncalibration: {rounds} rounds, seed {seed}, discards leaving {GATE} cards or fewer");
    for ((name, _), bins) in outlooks.iter().zip(&bins) {
        println!("\n{name}");
        println!("bin  decisions  predicted  won     95% CI of won      in  points  scored  ± 95%");
        let mut total = Bin::default();
        for (i, bin) in bins.iter().enumerate() {
            total.n += bin.n;
            total.chance += bin.chance;
            total.wins += bin.wins;
            total.points += bin.points;
            total.scored += bin.scored;
            total.scored2 += bin.scored2;
            print_bin(&i.to_string(), bin);
        }
        print_bin("all", &total);
    }
}

fn print_bin(label: &str, bin: &Bin) {
    if bin.n == 0 {
        return;
    }
    let n = bin.n as f64;
    let (chance, won) = (bin.chance / n, bin.wins as f64 / n);
    let (low, high) = wilson(bin.wins, bin.n);
    let (points, scored) = (bin.points / n, bin.scored / n);
    let sd = (bin.scored2 / n - scored * scored).max(0.0).sqrt();
    println!(
        "{label:>3}  {:9}  {chance:9.4}  {won:.4}  [{low:.4}, {high:.4}]  {}  {points:6.2}  {scored:6.2}  {:5.2}",
        bin.n,
        if (low..=high).contains(&chance) {
            "yes"
        } else {
            " no"
        },
        1.96 * sd / n.sqrt(),
    );
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let rounds = args.next().flatten().unwrap_or(1000);
    let seed = args.next().flatten().unwrap_or(0);
    let check = args.next().flatten().unwrap_or(rounds);
    let mut rules = Rules::new();
    if let Ok(threshold) = std::env::var("THRESHOLD") {
        rules.threshold = threshold.parse().expect("THRESHOLD is a number");
    }
    if let Ok(players) = std::env::var("PLAYERS") {
        rules.players = players.parse().expect("PLAYERS is a number");
    }
    if rounds == 0 {
        perturb(&rules, check, seed + 1);
        return;
    }
    let mut tally = Tally::new(&rules);
    let played = common::deals(rules, rounds, seed, |round| {
        let bots =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        let dealt = Seat::ALL.map(|seat| round.hand(seat));
        let mut table = Table::new(round, bots);
        table.run().expect("bots act legally");
        (dealt, table.round().clone())
    });
    for (dealt, round) in &played {
        tally.round(round.events(), round.dealer(), *dealt);
    }
    println!(
        "{rounds} rounds of HeuristicBot self-play, seed {seed}, {} players, {} to win",
        rules.players, rules.threshold
    );
    tally.print();
    // h from segments with fewer gated cards than this is too noisy
    let tables = tally.outlook(500);
    println!("\n{tables:.4?}");
    println!("generals: {:.4?}", tally.generals(500));
    let flat = Outlook {
        end: vec![tally.end()],
        cards: vec![tally.per_draw()],
        ..tables.clone()
    };
    let h = Outlook {
        cards: flat.cards.clone(),
        ..tables.clone()
    };
    let outlooks = [
        ("one h, one rate a draw", flat),
        ("h by draws left, one rate a draw", h),
        ("both by draws left", tables),
    ];
    calibrate(&rules, &outlooks, check, seed + 1);
}

/// The measured outlook and one constant changed at a time, to see which
/// one the calibration points at
fn perturb(rules: &Rules, rounds: u64, seed: u64) {
    let base = Outlook::new(rules);
    let scale = |table: &[f64], by: f64| table.iter().map(|x| x * by).collect();
    let outlooks = [
        ("measured", base.clone()),
        (
            "h × 1.5",
            Outlook {
                end: scale(&base.end, 1.5),
                ..base.clone()
            },
        ),
        (
            "cards a segment × 0.8",
            Outlook {
                cards: scale(&base.cards, 0.8),
                ..base.clone()
            },
        ),
        (
            "e × 0.7",
            Outlook {
                eat: base.eat * 0.7,
                ..base.clone()
            },
        ),
    ];
    calibrate(rules, &outlooks, rounds, seed);
}
