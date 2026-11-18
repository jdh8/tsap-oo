//! What an [`ExpectBot`] decision costs, the numbers behind batch 4C of
//! `doc/expectation.md`: one sits in seat 1 among three [`HeuristicBot`]s
//! and every claim and discard it ranks is timed, the call the hint would
//! make.  Single-threaded, so it also runs as WASI under node, whose V8 is
//! the browser's wasm engine:
//!
//! ```sh
//! cargo run --release --example expect_time -- [rounds] [seed base]
//! cargo build --release --example expect_time --target wasm32-wasip1
//! node --no-warnings research/wasi.mjs target/wasm32-wasip1/release/examples/expect_time.wasm [rounds] [seed base]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use tsap_oo::susik::Card;
use tsap_oo::{
    Claim, ClaimOptions, ExpectBot, HeuristicBot, Phase, Round, Rules, Seat, Step, Strategy, Table,
    View,
};

/// Concealed cards once discarded, and the microseconds the ranking took
type Decision = (usize, u128);

struct Timed {
    bot: ExpectBot,
    log: Rc<RefCell<Vec<Decision>>>,
}

impl Timed {
    fn record(&self, view: &View<'_>, start: Instant) {
        let micros = start.elapsed().as_micros();
        let discarding = matches!(view.phase(), Phase::Discard(_));
        let cards = view.hand().len() - usize::from(discarding);
        self.log.borrow_mut().push((cards, micros));
    }
}

impl Strategy for Timed {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let start = Instant::now();
        let claim = self.bot.claims(view, options)[0].0;
        self.record(view, start);
        claim
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let start = Instant::now();
        let card = self.bot.discards(view)[0].0;
        self.record(view, start);
        card
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut arg = |default: u64| args.next().and_then(|s| s.parse().ok()).unwrap_or(default);
    let (rounds, base) = (arg(1000), arg(0));
    let rules = Rules::new();
    let log = Rc::<RefCell<Vec<Decision>>>::default();

    for seed in base..base + rounds {
        let mut rng = SmallRng::seed_from_u64(seed);
        let round = Round::deal(rules, Seat::ALL[(seed % 4) as usize], &mut rng);
        let mut seats =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        seats[1] = Some(Box::new(Timed {
            bot: ExpectBot::new(&rules),
            log: Rc::clone(&log),
        }));
        assert_eq!(Table::new(round, seats).run(), Ok(Step::Finished));
    }

    println!("seed base {base}, {rounds} rounds");
    println!("| cards | decisions | mean ms | p50 | p95 | p99 | max |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    // The concealed cards a row keeps
    type Keep = fn(usize) -> bool;
    let rows: [(&str, Keep); 3] = [
        ("all", |_| true),
        ("over 10", |c| c > 10),
        ("10 or fewer", |c| c <= 10),
    ];
    for (label, keep) in rows {
        let mut micros: Vec<u128> = log
            .borrow()
            .iter()
            .filter(|d| keep(d.0))
            .map(|d| d.1)
            .collect();
        micros.sort_unstable();
        let n = micros.len();
        let ms = |q: f64| micros[((n as f64 * q) as usize).min(n - 1)] as f64 / 1e3;
        let mean = micros.iter().sum::<u128>() as f64 / n as f64 / 1e3;
        println!(
            "| {label} | {n} | {mean:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            ms(0.5),
            ms(0.95),
            ms(0.99),
            ms(1.0),
        );
    }
}
