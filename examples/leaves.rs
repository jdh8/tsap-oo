//! How much error the leaves of a shallow graph can bear, batch 2 of
//! `doc/eval.md`.  Seat 1 plays the desktop [`ExpectBot`] (gated at 14
//! cards) among three [`HeuristicBot`]s, the A/B table.  At each of its
//! decisions with 11 to 14 cards, the graph stops 0, 1 and 2 claims from the
//! roots ([`Prune::depth`]) and each leaf reads its exact curve, from the
//! whole graph, with an error: an offset drawn for each leaf, or one shared
//! by all, the same at every count of public cards to come; or every value
//! scaled, the shrinkage of an eval that fits too little.  The regret is
//! what the answer the shallow graph picks falls short of the best by the
//! exact values.
//!
//! Cards are counted as the gate counts them: in hand when answering an
//! offer, after the discard when discarding.
//!
//! ```sh
//! cargo run --release --example leaves -- [rounds] [seed base]
//! ```
use rand::rngs::SmallRng;
use rand::{RngExt as _, SeedableRng as _};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use tsap_oo::expect::END_SCALE;
use tsap_oo::susik::{Card, Hand, Meld};
use tsap_oo::{
    Claim, ClaimOptions, ExpectBot, Graph, HeuristicBot, Outlook, Payoff, Prune, Round, Rules,
    Seat, Step, Strategy, Table, View, discardable,
};

/// The gate the desktop plays at
const GATE: usize = 14;

const DEPTHS: [u32; 3] = [0, 1, 2];

/// Each leaf's own offset (its standard deviation), the offset all leaves
/// share, and the scale of every value; the first is exact
const ERRORS: [(f64, f64, f64); 11] = [
    (0.0, 0.0, 1.0),
    (0.25, 0.0, 1.0),
    (0.5, 0.0, 1.0),
    (1.0, 0.0, 1.0),
    (2.0, 0.0, 1.0),
    (0.0, -1.0, 1.0),
    (0.0, -0.5, 1.0),
    (0.0, 0.5, 1.0),
    (0.0, 1.0, 1.0),
    (0.0, 0.0, 0.8),
    (0.0, 0.0, 0.5),
];

struct Decision {
    cards: usize,
    leaves: [usize; DEPTHS.len()],
    /// By depth and error
    regret: [[f64; ERRORS.len()]; DEPTHS.len()],
}

struct Logged {
    bot: ExpectBot,
    outlook: Outlook,
    rng: SmallRng,
    log: Rc<RefCell<Vec<Decision>>>,
}

/// The best of each group's roots: `groups[i]` many consecutive `roots`
fn by_group(sizes: &[usize], roots: &[usize], values: &[f64]) -> Vec<f64> {
    let mut nodes = roots.iter();
    sizes
        .iter()
        .map(|&n| {
            nodes
                .by_ref()
                .take(n)
                .map(|&node| values[node])
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect()
}

/// The first of the largest
fn argmax(xs: &[f64]) -> usize {
    (0..xs.len()).fold(0, |a, b| if xs[b] > xs[a] { b } else { a })
}

impl Logged {
    fn record(&mut self, view: &View<'_>, cards: usize, groups: &[Vec<(Hand, u32)>]) {
        if !(11..=GATE).contains(&cards) {
            return;
        }
        let rules = view.rules();
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card))
            .chain(view.offer().map(|offer| offer.card));
        let live = shown.fold(Hand::FULL - view.hand(), Hand::without);
        let draws = view.pile_len().saturating_sub(rules.dead());
        let roots: Vec<_> = groups.iter().flatten().copied().collect();
        let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();

        let whole = Graph::new(rules, live, &roots, Prune::default());
        let exact = whole.values(&self.outlook, draws, Payoff::Points);
        let truth = by_group(&sizes, whole.roots(), &exact);
        let top = truth.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let index: HashMap<_, _> = (1..whole.len()).map(|n| (whole.state(n), n)).collect();

        let mut decision = Decision {
            cards,
            leaves: [0; DEPTHS.len()],
            regret: [[0.0; ERRORS.len()]; DEPTHS.len()],
        };
        for (d, &depth) in DEPTHS.iter().enumerate() {
            let prune = Prune {
                depth: Some(depth),
                ..Prune::default()
            };
            let graph = Graph::new(rules, live, &roots, prune);
            let nodes: Vec<usize> = graph
                .leaves()
                .iter()
                .map(|&leaf| index[&graph.state(leaf)])
                .collect();
            let curves = whole.curves(&self.outlook, draws, Payoff::Points, &nodes);
            decision.leaves[d] = nodes.len();
            for (e, &(spread, bias, scale)) in ERRORS.iter().enumerate() {
                let off: Vec<Vec<f64>> = curves
                    .iter()
                    .map(|curve| {
                        let offset = bias + spread * gauss(&mut self.rng);
                        // Nothing to come is worth 0, as in the recurrence
                        std::iter::once(0.0)
                            .chain(curve[1..].iter().map(|v| scale * v + offset))
                            .collect()
                    })
                    .collect();
                let values = graph.values_with(&self.outlook, draws, Payoff::Points, &off);
                let chosen = argmax(&by_group(&sizes, graph.roots(), &values));
                decision.regret[d][e] = top - truth[chosen];
            }
            assert!(
                decision.regret[d][0] == 0.0,
                "exact leaves pick the best at depth {depth}"
            );
        }
        self.log.borrow_mut().push(decision);
    }
}

/// A standard normal draw, by Box and Muller
fn gauss(rng: &mut SmallRng) -> f64 {
    let (u, v): (f64, f64) = (rng.random(), rng.random());
    (-2.0 * (1.0 - u).ln()).sqrt() * (std::f64::consts::TAU * v).cos()
}

impl Strategy for Logged {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
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
        self.record(view, hand.len(), &groups);
        self.bot.claim(view, options)
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let hand = view.hand();
        let rules = view.rules();
        let base: u32 = view
            .exposed(view.seat())
            .iter()
            .map(|meld| rules.exposed.score(meld))
            .sum();
        let groups: Vec<_> = discardable(hand, view.locked())
            .distinct()
            .map(|(card, _)| vec![(hand.without(card), base)])
            .collect();
        self.record(view, hand.len() - 1, &groups);
        self.bot.discard(view)
    }
}

/// Play round `seed`: seat 1's decisions
fn play(seed: u64, outlook: &Outlook) -> Vec<Decision> {
    let rules = Rules::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    let round = Round::deal(rules, Seat::ALL[(seed % 4) as usize], &mut rng);
    let log = Rc::<RefCell<Vec<Decision>>>::default();
    let mut seats =
        Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
    seats[1] = Some(Box::new(Logged {
        bot: ExpectBot::new(&rules).gated(GATE),
        outlook: outlook.clone(),
        rng: SmallRng::seed_from_u64(!seed),
        log: Rc::clone(&log),
    }));
    let mut table = Table::new(round, seats);
    assert_eq!(table.run(), Ok(Step::Finished));
    drop(table);
    log.take()
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let rounds = args.next().flatten().unwrap_or(100);
    let base = args.next().flatten().unwrap_or(0);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()) as u64;
    let mut outlook = Outlook::default();
    outlook.end.iter_mut().for_each(|h| *h *= END_SCALE);

    let start = Instant::now();
    let played: Vec<Vec<Decision>> = std::thread::scope(|scope| {
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
    let decisions: Vec<&Decision> = played.iter().flatten().collect();
    println!(
        "seed base {base}, {rounds} rounds, {} decisions at 11 to {GATE} cards, {:.0} s on {threads} threads",
        decisions.len(),
        start.elapsed().as_secs_f64()
    );

    let bands: Vec<(String, Vec<&Decision>)> = (11..=GATE)
        .map(|n| {
            let rows = decisions.iter().copied().filter(|d| d.cards == n).collect();
            (n.to_string(), rows)
        })
        .chain(std::iter::once(("all".to_owned(), decisions.clone())))
        .collect();
    print!("\n| depth | spread | bias | scale |");
    for (label, rows) in &bands {
        print!(" {label} ({}) |", rows.len());
    }
    println!(" worse | SE |");
    println!(
        "| ---: | ---: | ---: | ---: |{} ---: | ---: |",
        " ---: |".repeat(bands.len())
    );
    for (d, depth) in DEPTHS.iter().enumerate() {
        let leaves = decisions.iter().map(|x| x.leaves[d]).sum::<usize>() as f64;
        println!(
            "| {depth} | leaves | {:.0} | |{}",
            leaves / decisions.len() as f64,
            " |".repeat(bands.len() + 2)
        );
        for (e, (spread, bias, scale)) in ERRORS.iter().enumerate().skip(1) {
            print!("| {depth} | {spread} | {bias:+} | {scale} |");
            for (_, rows) in &bands {
                let mean = rows.iter().map(|x| x.regret[d][e]).sum::<f64>() / rows.len() as f64;
                print!(" {mean:.4} |");
            }
            // Per decision; decisions of one round are correlated
            let xs: Vec<f64> = decisions.iter().map(|x| x.regret[d][e]).collect();
            let n = xs.len() as f64;
            let mean = xs.iter().sum::<f64>() / n;
            let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
            let worse = xs.iter().filter(|&&x| x > 1e-9).count() as f64 / n;
            println!(" {:.1}% | {:.4} |", 100.0 * worse, (var / n).sqrt());
        }
    }
}
