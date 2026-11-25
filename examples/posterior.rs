//! What the other seat's whole history says about its hand at a table of
//! two: the measurement before any inference by sampling
//! (`doc/opponents.md`, batch 4).
//!
//! `ExpectBot` as it ships plays `HeuristicBot`, each deal twice, once in
//! either seat.  At every discard it values in points, hands for the other
//! seat are dealt from the cards the seat has not seen, and each is played
//! through the round from the deal, `HeuristicBot` holding it: a hand fits
//! when every public event comes out as it did.  Nothing drawn enters a
//! hand, so the cards that left it are public and a hand now is a deal
//! then.  The hands that fit are the posterior: `HeuristicBot` has no dice,
//! so each is as likely as another.
//!
//! ```sh
//! cargo run --release --example posterior -- [rounds] [seed]
//! ```
//!
//! `PROPOSALS` (default 20000) is the most hands dealt at a decision and
//! `SAMPLES` (default 50) the hands that fit it stops at; a decision with
//! fewer than [`LEAST`] is thin and counts as the bot's own.  Late in a
//! round one hand in thousands fits, so `CHAIN` (default 0, for none) walks
//! instead: from the hand the other seat holds, which no seat could start
//! from, it swaps a card of the hand for one of the pile and keeps the swap
//! if the hand still fits, and takes a hand every `CHAIN` swaps after five
//! times as many.  Where both reach, the two should agree.  `GATE_CARDS`
//! and `SHALLOW` are as in `giveaway`.  By the other seat's concealed cards
//! it prints:
//!
//! - the hands dealt for one that fits, or the swaps tried for one kept, in
//!   bits;
//! - how much of the spread of the copies it holds of each kind the
//!   posterior's mean takes off the prior's, the kinds dealt evenly;
//! - the chance the posterior gives a win on a discard it does win on, and
//!   on one it does not, beside the same of the first [`PRIOR`] hands dealt;
//! - what discarding by value less that chance of [`LOSS`] points keeps a
//!   round, one card ahead, as `giveaway` counts the oracle's win half.
use rand::rngs::SmallRng;
use rand::seq::SliceRandom as _;
use rand::{RngExt as _, SeedableRng as _};
use tsap_oo::susik::{Card, Hand};
use tsap_oo::{
    Claim, Eval, Event, ExpectBot, Game, HeuristicBot, Phase, Round, Rules, Seat, Strategy, View,
};

mod common;

/// A decision with fewer hands that fit is thin
const LEAST: usize = 20;

/// The hands dealt that stand for the prior
const PRIOR: usize = 200;

/// What the seat loses when the other wins on its discard, in points: the
/// mean of `giveaway`'s rows
// ponytail: one price for every win; settle each sampled hand's if the
// chance turns out worth pricing.
const LOSS: f64 = 9.0;

/// The prices a chance of [`LOSS`] is charged at: what T's deals paid for a
/// win, and the oracle's
const PRICES: [f64; 2] = [0.29, 1.0];

#[derive(Default, Clone, Copy)]
struct Count {
    decisions: u64,
    thin: u64,
    /// log2 of hands dealt over hands that fit, over the decisions not thin
    bits: f64,
    /// Squared error of the copies held of each kind: prior and posterior
    spread: [f64; 2],
    /// Discards the other seat wins on, and the chance given them:
    /// posterior and prior
    wins: u64,
    hit: [f64; 2],
    /// The same of the discards it does not win on
    others: u64,
    miss: [f64; 2],
    /// Points kept one card ahead: the oracle, the posterior at each of
    /// [`PRICES`], the prior at the last
    kept: [f64; 4],
    /// Cards won on: the bot's, and each of the same but the oracle
    won: [u64; 4],
    /// Discards moved by each but the oracle
    moved: [u64; 3],
}

impl Count {
    fn add(&mut self, o: &Self) {
        self.decisions += o.decisions;
        self.thin += o.thin;
        self.bits += o.bits;
        self.wins += o.wins;
        self.others += o.others;
        for i in 0..2 {
            self.spread[i] += o.spread[i];
            self.hit[i] += o.hit[i];
            self.miss[i] += o.miss[i];
        }
        for i in 0..4 {
            self.kept[i] += o.kept[i];
            self.won[i] += o.won[i];
        }
        for i in 0..3 {
            self.moved[i] += o.moved[i];
        }
    }
}

/// `PROPOSALS`, `SAMPLES` and `CHAIN`
#[derive(Clone, Copy)]
struct Limits {
    proposals: usize,
    samples: usize,
    chain: usize,
}

/// By the other seat's concealed cards: 18 or more, 15 to 17, 12 to 14, 9 to
/// 11, 8 or fewer
type Counts = [Count; 5];

/// The round as `me` saw it up to a decision of its own
struct History<'a> {
    rules: Rules,
    me: Seat,
    /// The hands as dealt: `me`'s, and the other seat's less what it holds
    /// now, which every seat has seen leave it
    dealt: Hand,
    left: Hand,
    /// The public events before the decision
    events: &'a [Event],
}

impl History<'_> {
    /// Whether the other seat, holding `hand` now with `rest` in the pile,
    /// would have played every public event as it went
    // ponytail: a seat's eat that a pong took back is replayed as its pass.
    fn fits(&self, hand: Hand, rest: &[Card], policy: &mut HeuristicBot) -> bool {
        let (me, opp) = (self.me, self.me.next(2));
        let dealer = Seat::ALL[0];
        let mut pile = rest.to_vec();
        pile.extend(self.events.iter().rev().filter_map(|event| match event {
            Event::Draw { card, .. } => Some(*card),
            _ => None,
        }));
        // The dealer's twenty come off first, the other seat's twenty, then
        // the dealer's last card.
        let (first, second) = if me == dealer {
            (self.dealt, self.left + hand)
        } else {
            (self.left + hand, self.dealt)
        };
        let mut first = first.iter();
        pile.extend(first.next());
        pile.extend(second.iter());
        pile.extend(first);
        let Ok(mut round) = Round::from_pile(self.rules, dealer, pile) else {
            panic!("the cards of a round are the deck");
        };
        loop {
            let seen = round.events().len();
            if seen > self.events.len() || round.events() != &self.events[..seen] {
                return false;
            }
            if seen == self.events.len() && round.phase() == Phase::Discard(me) {
                return true;
            }
            let next = self.events.get(seen);
            let played = match round.phase() {
                Phase::Finished => return false,
                Phase::Claim(seat) if seat == me => {
                    let claim = match next {
                        Some(&Event::Eat { seat, meld, .. }) if seat == me => Claim::Eat(meld),
                        Some(&Event::Pong { seat, .. }) if seat == me => Claim::Pong,
                        _ => Claim::Pass,
                    };
                    round.claim(me, claim)
                }
                Phase::Discard(seat) if seat == me => match next {
                    Some(&Event::Discard { seat, card }) if seat == me => round.discard(me, card),
                    _ => return false,
                },
                Phase::Claim(_) => {
                    let (view, options) = (View::new(&round, opp), round.claim_options(opp));
                    let claim = policy.claim(&view, &options);
                    round.claim(opp, claim)
                }
                Phase::Discard(_) => {
                    let card = policy.discard(&View::new(&round, opp));
                    round.discard(opp, card)
                }
            };
            if played.is_err() {
                return false;
            }
        }
    }
}

/// The first of the best by `key`, as `rank` leaves equals
fn pick(n: usize, key: impl Fn(usize) -> f64) -> usize {
    (0..n).fold(0, |best, i| if key(i) > key(best) { i } else { best })
}

/// One discard of `me`'s: deal, fit, and count
#[allow(clippy::too_many_arguments)]
fn record(
    bot: &mut ExpectBot,
    policy: &mut HeuristicBot,
    round: &Round,
    me: Seat,
    dealt: [Hand; 2],
    limits: Limits,
    rng: &mut SmallRng,
    counts: &mut Counts,
) {
    let (view, opp) = (View::new(round, me), me.next(2));
    let truth = round.hand(opp);
    let n = truth.len();
    let shown = Seat::ALL
        .into_iter()
        .flat_map(|seat| view.exposed(seat))
        .flat_map(|meld| meld.cards())
        .chain(view.discards().iter().map(|&(_, card)| card));
    let unseen = shown.fold(Hand::FULL - view.hand(), Hand::without);
    let history = History {
        rules: *round.rules(),
        me,
        dealt: dealt[me.index()],
        left: dealt[opp.index()] - truth,
        events: round.events(),
    };
    let ranked = bot.discards(&view);
    let cards: Vec<Card> = ranked.iter().map(|&(card, _)| card).collect();
    let wins = |hand: Hand| -> Vec<bool> {
        cards
            .iter()
            .map(|&card| round.win(opp, hand, card).is_some())
            .collect()
    };

    let mut deck: Vec<Card> = unseen.iter().collect();
    let mut prior = vec![0.0; cards.len()];
    for _ in 0..PRIOR {
        let (hand, _) = deck.partial_shuffle(rng, n);
        for (chance, won) in prior.iter_mut().zip(wins(hand.iter().copied().collect())) {
            *chance += f64::from(u8::from(won)) / PRIOR as f64;
        }
    }
    let (mut tried, mut fit) = (0_usize, 0_usize);
    // Of each kind, the copies held and their squares, summed over the fits
    let (mut held, mut squares) = ([0.0; 28], [0.0; 28]);
    let mut post = vec![0.0; cards.len()];
    let mut take = |hand: Hand| {
        for (card, copies) in hand.distinct() {
            held[card.index()] += f64::from(copies);
            squares[card.index()] += f64::from(copies).powi(2);
        }
        for (chance, won) in post.iter_mut().zip(wins(hand)) {
            *chance += f64::from(u8::from(won));
        }
    };
    if limits.chain == 0 {
        while tried < limits.proposals && fit < limits.samples {
            let (hand, rest) = deck.partial_shuffle(rng, n);
            let hand: Hand = hand.iter().copied().collect();
            tried += 1;
            if history.fits(hand, rest, policy) {
                fit += 1;
                take(hand);
            }
        }
    } else {
        // The hand it holds, then the pile: a swap across the two is a move,
        // kept if the hand still fits, so every hand that fits is as likely
        deck = truth.iter().chain((unseen - truth).iter()).collect();
        let mut moved = 0_usize;
        for step in 1..=limits.chain * (5 + limits.samples) {
            if n > 0 && n < deck.len() {
                let (i, j) = (rng.random_range(..n), rng.random_range(n..deck.len()));
                deck.swap(i, j);
                tried += 1;
                if history.fits(deck[..n].iter().copied().collect(), &deck[n..], policy) {
                    moved += 1;
                } else {
                    deck.swap(i, j);
                }
            }
            if step > 5 * limits.chain && step % limits.chain == 0 {
                fit += 1;
                take(deck[..n].iter().copied().collect());
            }
        }
        // The swaps tried for one kept, where `bits` has the hands dealt
        (tried, moved) = (tried.max(1), moved.max(1));
        tried = tried * fit / moved;
    }
    debug_assert!(
        history.fits(truth, &(unseen - truth).iter().collect::<Vec<_>>(), policy),
        "the hand it holds fits"
    );

    // What each discard gives away for true, as the oracle's win half
    let events = round.events().len();
    let cost: Vec<Option<f64>> = ranked
        .iter()
        .map(|&(card, value)| {
            let mut after = round.clone();
            after.discard(me, card).expect("a legal discard");
            match after.events().get(events + 1) {
                Some(&Event::Win { seat, .. }) if seat == opp => {
                    let mut game = Game::new(*round.rules(), after.dealer());
                    game.settle(after.result().expect("the round is finished"));
                    Some((value - f64::from(game.scores()[me.index()])).max(0.0))
                }
                _ => None,
            }
        })
        .collect();

    let count = &mut counts[((20 - n.min(20)) / 3).min(4)];
    count.decisions += 1;
    let thin = fit < LEAST;
    count.thin += u64::from(thin);
    let kept = |i: usize| ranked[i].1 - cost[i].unwrap_or(0.0);
    let oracle = pick(cards.len(), kept);
    count.kept[0] += kept(oracle) - kept(0);
    count.won[0] += u64::from(cost[0].is_some());
    if thin {
        // As the bot plays: nothing kept, the same card won on
        for won in &mut count.won[1..] {
            *won += u64::from(cost[0].is_some());
        }
        return;
    }
    post.iter_mut().for_each(|chance| *chance /= fit as f64);
    count.bits += (tried as f64 / fit as f64).log2();
    for (card, copies) in unseen.distinct() {
        let mean = held[card.index()] / fit as f64;
        let even = n as f64 * f64::from(copies) / unseen.len() as f64;
        let truth = f64::from(truth.count(card));
        count.spread[0] += (truth - even).powi(2);
        // Less the noise of a mean of `fit` hands
        let noise = (squares[card.index()] / fit as f64 - mean.powi(2)) / fit as f64;
        count.spread[1] += (truth - mean).powi(2) - noise;
    }
    for (i, cost) in cost.iter().enumerate() {
        let (n, sum) = if cost.is_some() {
            (&mut count.wins, &mut count.hit)
        } else {
            (&mut count.others, &mut count.miss)
        };
        *n += 1;
        sum[0] += post[i];
        sum[1] += prior[i];
    }
    let chances = [(&post, PRICES[0]), (&post, PRICES[1]), (&prior, PRICES[1])];
    for (k, (chance, price)) in chances.into_iter().enumerate() {
        let choice = pick(cards.len(), |i| ranked[i].1 - price * LOSS * chance[i]);
        count.kept[k + 1] += kept(choice) - kept(0);
        count.won[k + 1] += u64::from(cost[choice].is_some());
        count.moved[k] += u64::from(choice != 0);
    }
}

/// `round` played out with `bot` in `me`'s seat against `HeuristicBot`
fn play(
    mut round: Round,
    me: Seat,
    mut bot: ExpectBot,
    limits: Limits,
    rng: &mut SmallRng,
    counts: &mut Counts,
) {
    let rules = *round.rules();
    let mut policy = HeuristicBot::new(&rules);
    // A second one for the replays: the first is in the middle of a round
    let mut replay = HeuristicBot::new(&rules);
    let dealt = [round.hand(Seat::ALL[0]), round.hand(Seat::ALL[1])];
    loop {
        match round.phase() {
            Phase::Finished => break,
            Phase::Claim(seat) => {
                let (view, options) = (View::new(&round, seat), round.claim_options(seat));
                let claim = if seat == me {
                    bot.claim(&view, &options)
                } else {
                    policy.claim(&view, &options)
                };
                round.claim(seat, claim).expect("bots act legally");
            }
            Phase::Discard(seat) => {
                let view = View::new(&round, seat);
                let card: Card = if seat == me {
                    if bot.searches(&view, view.hand().len() - 1) {
                        record(
                            &mut bot,
                            &mut replay,
                            &round,
                            me,
                            dealt,
                            limits,
                            rng,
                            counts,
                        );
                    }
                    bot.discard(&view)
                } else {
                    policy.discard(&view)
                };
                round.discard(seat, card).expect("bots act legally");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let number = |i: usize, default| {
        args.get(i)
            .map_or(default, |s| s.parse().expect("a number"))
    };
    let (rounds, seed) = (number(0, 100), number(1, 0));
    let knob =
        |name, default| std::env::var(name).map_or(default, |s| s.parse().expect("a number"));
    let (gate, deep) = (knob("GATE_CARDS", 10), knob("SHALLOW", 0));
    let limits = Limits {
        proposals: knob("PROPOSALS", 20000),
        samples: knob("SAMPLES", 50),
        chain: knob("CHAIN", 0),
    };
    let mut rules = Rules::new();
    rules.players = 2;
    let played = common::deals(rules, rounds, seed, |round| {
        // The deal's own dice, so a deal replays whatever thread has it
        let dice = round.hand(Seat::ALL[0]).to_bits() as u64 ^ seed;
        let mut rng = SmallRng::seed_from_u64(dice);
        let mut counts = Counts::default();
        for me in [Seat::ALL[0], Seat::ALL[1]] {
            let bot = ExpectBot::new(&rules)
                .gated(gate)
                .shallow(Eval::of(&rules), deep);
            play(round.clone(), me, bot, limits, &mut rng, &mut counts);
        }
        counts
    });
    let mut counts = Counts::default();
    for deal in &played {
        for (count, other) in counts.iter_mut().zip(deal) {
            count.add(other);
        }
    }
    let mut all = Count::default();
    counts.iter().for_each(|count| all.add(count));
    println!(
        "{rounds} rounds each way, seed {seed}, gate {gate}, shallow {deep}, {} hands that fit: \
         {} dealt at most, or every {} swaps of a chain",
        limits.samples, limits.proposals, limits.chain
    );
    println!(
        "{:<8}{:>10}{:>7}{:>7}{:>8}{:>8}{:>9}{:>9}{:>9}{:>9}{:>9}",
        "cards",
        "decisions",
        "thin",
        "bits",
        "held",
        "wins",
        "post",
        "prior",
        "others",
        "post",
        "prior"
    );
    let names = ["18+", "15-17", "12-14", "9-11", "8-"];
    let rows = names.iter().zip(&counts).chain([(&"all", &all)]);
    for (name, c) in rows {
        let full = (c.decisions - c.thin).max(1) as f64;
        println!(
            "{name:<8}{:>10}{:>6.1}%{:>7.1}{:>7.1}%{:>8}{:>8.2}%{:>8.2}%{:>9}{:>8.2}%{:>8.2}%",
            c.decisions,
            100.0 * c.thin as f64 / c.decisions.max(1) as f64,
            c.bits / full,
            100.0 * (1.0 - c.spread[1] / c.spread[0].max(f64::MIN_POSITIVE)),
            c.wins,
            100.0 * c.hit[0] / c.wins.max(1) as f64,
            100.0 * c.hit[1] / c.wins.max(1) as f64,
            c.others,
            100.0 * c.miss[0] / c.others.max(1) as f64,
            100.0 * c.miss[1] / c.others.max(1) as f64,
        );
    }
    println!(
        "\npoints kept a round, one card ahead, then the cards won on of the bot's {}",
        all.won[0]
    );
    let each = 2.0 * rounds as f64;
    println!("{:<28}{:>8.3}", "the oracle's win half", all.kept[0] / each);
    let names = [
        format!("the posterior at {}", PRICES[0]),
        format!("the posterior at {}", PRICES[1]),
        format!("the prior at {}", PRICES[1]),
    ];
    for (k, name) in names.iter().enumerate() {
        println!(
            "{name:<28}{:>8.3}{:>8}{:>8.2}% of discards moved",
            all.kept[k + 1] / each,
            all.won[k + 1],
            100.0 * all.moved[k] as f64 / all.decisions.max(1) as f64,
        );
    }
}
