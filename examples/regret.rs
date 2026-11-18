//! What `HeuristicBot` gives up by the DP's values, batch 1 of
//! `doc/eval.md`.  Seat 1 plays the desktop [`ExpectBot`] (gated at 14
//! cards) among three [`HeuristicBot`]s, the A/B table.  At each of its
//! decisions every answer is valued by a [`Graph`] as `ExpectBot` builds it,
//! [`Discards::Loose`] from 18 cards: the regret is what `HeuristicBot`'s
//! first choice falls short of the best, and the win chance of the answer
//! played is checked against whether the seat won.
//!
//! Cards are counted as the gate counts them: in hand when answering an
//! offer, after the discard when discarding.
//!
//! ```sh
//! cargo run --release --example regret -- [rounds] [seed base] [gain of 11 to 14] [its SE]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tsap_oo::expect::END_SCALE;
use tsap_oo::susik::{Card, Hand, Meld};
use tsap_oo::{
    Claim, ClaimOptions, Discards, ExpectBot, Graph, HeuristicBot, Outlook, Payoff, Prune, Round,
    Rules, Seat, Step, Strategy, Table, View, discardable,
};

/// The gate the desktop plays at
const GATE: usize = 14;

/// Cards, regret in points, and the predicted win chance of the answer
/// played
type Decision = (usize, f64, f64);

struct Logged {
    bot: ExpectBot,
    greedy: HeuristicBot,
    outlook: Outlook,
    log: Rc<RefCell<Vec<Decision>>>,
}

impl Logged {
    /// Value `groups` of roots, one per answer; log the regret of answer
    /// `greedy` and the win chance of answer `played`
    fn record(
        &self,
        view: &View<'_>,
        cards: usize,
        groups: &[Vec<(Hand, u32)>],
        greedy: usize,
        played: usize,
    ) {
        let rules = view.rules();
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card))
            .chain(view.offer().map(|offer| offer.card));
        let live = shown.fold(Hand::FULL - view.hand(), Hand::without);
        let prune = Prune {
            discards: if cards >= 18 {
                Discards::Loose
            } else {
                Discards::All
            },
            ..Prune::default()
        };
        let roots: Vec<_> = groups.iter().flatten().copied().collect();
        let graph = Graph::new(rules, live, &roots, prune);
        let draws = view.pile_len().saturating_sub(rules.dead_tail);
        let points = graph.values(&self.outlook, draws, Payoff::Points);
        let chances = graph.values(&self.outlook, draws, Payoff::Win);
        // The node each answer leads to: its best root by points
        let mut nodes = graph.roots().iter();
        let best: Vec<usize> = groups
            .iter()
            .map(|group| {
                nodes
                    .by_ref()
                    .take(group.len())
                    .copied()
                    .fold(None, |a: Option<usize>, b| match a {
                        Some(a) if points[a] >= points[b] => Some(a),
                        _ => Some(b),
                    })
                    .expect("every answer has a root")
            })
            .collect();
        let top = best
            .iter()
            .map(|&node| points[node])
            .fold(f64::NEG_INFINITY, f64::max);
        if cards <= GATE {
            assert_eq!(
                points[best[played]], top,
                "inside the gate the bot plays the best"
            );
        }
        let regret = top - points[best[greedy]];
        self.log
            .borrow_mut()
            .push((cards, regret, chances[best[played]]));
    }
}

impl Strategy for Logged {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let greedy = self.greedy.claims(view, options)[0].0;
        let played = self.bot.claim(view, options);
        let every: Vec<Claim> = (!options.forced())
            .then_some(Claim::Pass)
            .into_iter()
            .chain(options.pong.then_some(Claim::Pong))
            .chain(options.eats.iter().map(|&meld| Claim::Eat(meld)))
            .collect();
        let (hand, card) = (view.hand(), view.offer().expect("an offer").card);
        let rules = view.rules();
        let base: u32 = view
            .exposed(view.seat())
            .iter()
            .map(|meld| rules.exposed.score(meld))
            .sum();
        let groups: Vec<Vec<(Hand, u32)>> = every
            .iter()
            .map(|&claim| {
                let meld = match claim {
                    Claim::Pass => return vec![(hand, base)],
                    Claim::Pong => Meld::Triplet(card),
                    Claim::Eat(meld) => meld,
                };
                let rest = hand - meld.cards().without(card);
                let base = base + rules.exposed.score(&meld);
                discardable(rest, Some(card))
                    .distinct()
                    .map(|(discard, _)| (rest.without(discard), base))
                    .collect()
            })
            .collect();
        let at = |claim| {
            every
                .iter()
                .position(|&c| c == claim)
                .expect("a legal claim")
        };
        self.record(view, hand.len(), &groups, at(greedy), at(played));
        played
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let greedy = self.greedy.discards(view)[0].0;
        let played = self.bot.discard(view);
        let hand = view.hand();
        let rules = view.rules();
        let base: u32 = view
            .exposed(view.seat())
            .iter()
            .map(|meld| rules.exposed.score(meld))
            .sum();
        let every: Vec<Card> = discardable(hand, view.locked())
            .distinct()
            .map(|(card, _)| card)
            .collect();
        let groups: Vec<_> = every
            .iter()
            .map(|&card| vec![(hand.without(card), base)])
            .collect();
        let at = |card| {
            every
                .iter()
                .position(|&c| c == card)
                .expect("a legal discard")
        };
        self.record(view, hand.len() - 1, &groups, at(greedy), at(played));
        played
    }
}

/// Play round `seed`: seat 1's decisions, and whether it won
fn play(seed: u64, outlook: &Outlook) -> (Vec<Decision>, bool) {
    let rules = Rules::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    let round = Round::deal(rules, Seat::ALL[(seed % 4) as usize], &mut rng);
    let log = Rc::<RefCell<Vec<Decision>>>::default();
    let mut seats =
        Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
    seats[1] = Some(Box::new(Logged {
        bot: ExpectBot::new(&rules).gated(GATE),
        greedy: HeuristicBot::new(&rules),
        outlook: outlook.clone(),
        log: Rc::clone(&log),
    }));
    let mut table = Table::new(round, seats);
    assert_eq!(table.run(), Ok(Step::Finished));
    let won = table.round().result().expect("finished").winner == Some(Seat::ALL[1]);
    drop(table);
    (log.take(), won)
}

/// Wilson's 95% interval for `k` of `n`
fn wilson(k: u64, n: u64) -> (f64, f64) {
    let (n, p, z) = (n as f64, k as f64 / n as f64, 1.96_f64);
    let mid = (p + z * z / (2.0 * n)) / (1.0 + z * z / n);
    let half = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / (1.0 + z * z / n);
    (mid - half, mid + half)
}

/// Mean and standard error of `xs`
fn mean_se(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, (var / n).sqrt())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut arg = |default: f64| args.next().and_then(|s| s.parse().ok()).unwrap_or(default);
    let (rounds, base) = (arg(100.0) as u64, arg(0.0) as u64);
    // Application D of doc/expectation.md: the gate from 10 to 14 cards
    let (gain, gain_se) = (arg(0.132), arg(0.014));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()) as u64;
    let mut outlook = Outlook::default();
    outlook.end.iter_mut().for_each(|h| *h *= END_SCALE);

    let start = Instant::now();
    let played: Vec<(Vec<Decision>, bool)> = std::thread::scope(|scope| {
        let outlook = &outlook;
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    (t..rounds)
                        .step_by(threads as usize)
                        .map(|i| play(base + i, outlook))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a round panicked"))
            .collect()
    });
    let decisions: usize = played.iter().map(|p| p.0.len()).sum();
    println!(
        "seed base {base}, {rounds} rounds, {decisions} decisions, {:.0} s on {threads} threads",
        start.elapsed().as_secs_f64()
    );

    println!(
        "\n| cards | decisions | per round | regret | worse | max | per round × regret | predicted win | won | 95% CI |"
    );
    println!("| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |");
    for cards in (0..=20).rev() {
        let rows: Vec<(f64, f64, bool)> = played
            .iter()
            .flat_map(|(log, won)| {
                log.iter()
                    .filter(|d| d.0 == cards)
                    .map(|d| (d.1, d.2, *won))
            })
            .collect();
        if rows.is_empty() {
            continue;
        }
        let n = rows.len() as f64;
        let regret: f64 = rows.iter().map(|r| r.0).sum();
        let wins = rows.iter().filter(|r| r.2).count() as u64;
        let (low, high) = wilson(wins, rows.len() as u64);
        println!(
            "| {cards} | {} | {:.3} | {:.4} | {:.1}% | {:.2} | {:.4} | {:.3} | {:.3} | [{low:.3}, {high:.3}] |",
            rows.len(),
            n / rounds as f64,
            regret / n,
            100.0 * rows.iter().filter(|r| r.0 > 1e-9).count() as f64 / n,
            rows.iter().map(|r| r.0).fold(0.0, f64::max),
            regret / rounds as f64,
            rows.iter().map(|r| r.1).sum::<f64>() / n,
            wins as f64 / n,
        );
    }

    // Σ N(n) · R(n) over a band is the band's regret a round
    let band = |lo: usize, hi: usize| -> Vec<f64> {
        played
            .iter()
            .map(|(log, _)| {
                log.iter()
                    .filter(|d| (lo..=hi).contains(&d.0))
                    .map(|d| d.1)
                    .sum()
            })
            .collect()
    };
    let low = band(11, 14);
    let (lo, lo_se) = mean_se(&low);
    println!("\n| band | Σ N·R a round | SE | predicted gain | SE |");
    println!("| --- | ---: | ---: | ---: | ---: |");
    println!("| 11 to 14 | {lo:.4} | {lo_se:.4} | {gain:.3} | {gain_se:.3} |");
    for (a, b) in [(15, 20), (15, 17), (18, 20), (0, 10)] {
        let high = band(a, b);
        let (hi, hi_se) = mean_se(&high);
        // The ratio's error by the delta method; the gain is independent
        let n = rounds as f64;
        let cov = low
            .iter()
            .zip(&high)
            .map(|(x, y)| (x - lo) * (y - hi))
            .sum::<f64>()
            / (n - 1.0)
            / n;
        let predicted = gain * hi / lo;
        let rel = (gain_se / gain).powi(2) + (hi_se / hi).powi(2) + (lo_se / lo).powi(2)
            - 2.0 * cov / (hi * lo);
        println!(
            "| {a} to {b} | {hi:.4} | {hi_se:.4} | {predicted:.3} | {:.3} |",
            predicted * rel.max(0.0).sqrt()
        );
    }
}
