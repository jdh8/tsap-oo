//! Where a [`MonteCarloBot`]'s time goes: one sits in seat 1 among three
//! [`HeuristicBot`]s, every decision it makes is timed, and the decisions
//! are tallied by the gates [`MonteCarloBot::gated`] could set.  The numbers
//! behind `research/gated-monte-carlo.md`.
//!
//! ```sh
//! cargo run --release --example gate -- [rounds] [seed base] [threads] [samples]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tsap_oo::susik::Card;
use tsap_oo::{
    Claim, ClaimOptions, HeuristicBot, MonteCarloBot, Phase, Round, Rules, Seat, Step, Strategy,
    Table, View,
};

/// One decision: concealed cards once discarded, shanten up to 1, the
/// microseconds the search took, and whether it overrode the greedy choice
type Decision = (usize, Option<u32>, u128, bool);

/// Whether a gate would let a decision be searched
type Gate = fn(&Decision) -> bool;

struct Timed {
    mc: MonteCarloBot<SmallRng>,
    greedy: HeuristicBot,
    log: Rc<RefCell<Vec<Decision>>>,
}

impl Timed {
    fn record(&mut self, view: &View<'_>, micros: u128, overrode: bool) {
        let discarding = matches!(view.phase(), Phase::Discard(_));
        let cards = view.hand().len() - usize::from(discarding);
        let shanten = self.greedy.shanten(view, 1);
        self.log
            .borrow_mut()
            .push((cards, shanten, micros, overrode));
    }
}

impl Strategy for Timed {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let greedy = self.greedy.claims(view, options)[0].0;
        let start = Instant::now();
        let claim = self.mc.claim(view, options);
        self.record(view, start.elapsed().as_micros(), claim != greedy);
        claim
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let greedy = self.greedy.discards(view)[0].0;
        let start = Instant::now();
        let card = self.mc.discard(view);
        self.record(view, start.elapsed().as_micros(), card != greedy);
        card
    }
}

/// Play round `seed` and return the decisions of seat 1
fn play(seed: u64, samples: usize) -> Vec<Decision> {
    let rules = Rules::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    let dealer = Seat::ALL[(seed % 4) as usize];
    let round = Round::deal(rules, dealer, &mut rng);
    let log = Rc::<RefCell<Vec<Decision>>>::default();
    let mut seats =
        Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
    seats[1] = Some(Box::new(Timed {
        mc: MonteCarloBot::new(&rules, samples, rng),
        greedy: HeuristicBot::new(&rules),
        log: Rc::clone(&log),
    }));
    let mut table = Table::new(round, seats);
    assert_eq!(table.run(), Ok(Step::Finished));
    drop(table);
    log.take()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut arg = |default: u64| args.next().and_then(|s| s.parse().ok()).unwrap_or(default);
    let (rounds, base, threads) = (arg(128), arg(0), arg(8));
    let samples = usize::try_from(arg(128)).expect("a sample count");

    let log: Vec<Decision> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    (t..rounds)
                        .step_by(usize::try_from(threads).expect("a thread count"))
                        .flat_map(|i| play(base + i, samples))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a round panicked"))
            .collect()
    });

    let total: u128 = log.iter().map(|d| d.2).sum();
    println!(
        "seed base {base}, {rounds} rounds, {samples} samples, {threads} threads: \
         {} decisions, {:.1} s a round",
        log.len(),
        total as f64 / 1e6 / rounds as f64,
    );
    println!("| gate | decisions | share | time | ms each | overrides |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: |");
    let gates: [(&str, Gate); 5] = [
        ("none", |_| true),
        ("10 cards or fewer", |d| d.0 <= 10),
        ("waiting", |d| d.1 == Some(0)),
        ("10 cards or fewer, or waiting", |d| {
            d.0 <= 10 || d.1 == Some(0)
        }),
        ("within one claim of waiting", |d| d.1.is_some()),
    ];
    for (name, open) in gates {
        let inside: Vec<_> = log.iter().filter(|d| open(d)).collect();
        let micros: u128 = inside.iter().map(|d| d.2).sum();
        println!(
            "| {name} | {} | {:.1}% | {:.1}% | {:.0} | {} |",
            inside.len(),
            100.0 * inside.len() as f64 / log.len() as f64,
            100.0 * micros as f64 / total as f64,
            micros as f64 / 1e3 / inside.len().max(1) as f64,
            inside.iter().filter(|d| d.3).count(),
        );
    }
}
