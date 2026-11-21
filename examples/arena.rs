//! Four greedy bots play a number of rounds and report the outcomes.
//!
//! ```console
//! cargo run --release --example arena -- 1000
//! ```
//!
//! A second argument seats a Monte Carlo bot of that many samples first:
//!
//! ```console
//! cargo run --release --example arena -- 100 16
//! ```
//!
//! A third argument seeds the deals (0 by default), so that runs can be
//! spread over the cores and added up.

use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::susik::{Meld, Solver};
use tsap_oo::{Game, HeuristicBot, MonteCarloBot, Rules, Seat, Step, Strategy, Table};

/// Rows of the winning-layout table; 碰 and 明槓 are melds shown on the
/// table (claimed, or completed by the winning card), 暗刻 and 暗槓 are not
const KINDS: [&str; 10] = [
    "將帥",
    "將士象",
    "車馬包",
    "三色兵卒",
    "四色兵卒",
    "暗刻",
    "碰",
    "暗槓",
    "明槓",
    "對子",
];

const fn kind(meld: &Meld, shown: bool) -> usize {
    match meld {
        Meld::Single(_) => 0,
        Meld::Court(_) => 1,
        Meld::Field(_) => 2,
        Meld::Soldiers(colors) if colors.len() == 3 => 3,
        Meld::Soldiers(_) => 4,
        Meld::Triplet(_) => 5 + shown as usize,
        Meld::Quad(_) => 7 + shown as usize,
        Meld::Pair(_) => 9,
    }
}

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let samples: Option<usize> = std::env::args().nth(2).and_then(|s| s.parse().ok());
    let rules = Rules::new();
    let mut game = Game::new(rules, Seat::ALL[0]);
    let seed: u64 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut wins = [0; Seat::COUNT];
    let (mut exhausted, mut fouls) = (0, 0);
    let mut hu_total = 0;
    // Melds of the winning layouts and the 胡 they bring, by row of KINDS
    let (mut melds, mut meld_hu) = ([0u32; KINDS.len()], [0u32; KINDS.len()]);
    // The same for the winner's dealt hand, laid out for the most 胡
    let (mut dealt_melds, mut dealt_hu) = ([0u32; KINDS.len()], [0u32; KINDS.len()]);
    let mut rich = Solver::new(rules.concealed, Some(0)).locked();
    for _ in 0..rounds {
        let mut bots =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        if let Some(samples) = samples {
            let seed = SmallRng::from_rng(&mut rng);
            bots[0] = Some(Box::new(MonteCarloBot::new(&rules, samples, seed)));
        }
        let round = game.deal(&mut rng);
        let dealt = Seat::ALL.map(|seat| round.hand(seat));
        let mut table = Table::new(round, bots);
        while table.step().expect("bots act legally") != Step::Finished {}
        let result = table.round().result().expect("finished");
        match result.winner {
            Some(seat) => {
                wins[seat.index()] += 1;
                hu_total += result.hu;
                // Exposed melds come first, then the winning card's meld
                let shown =
                    table.round().exposed(seat).len() + usize::from(result.winning_card.is_some());
                for (i, meld) in result.layout.iter().enumerate() {
                    let values = if i < shown {
                        rules.exposed
                    } else {
                        rules.concealed
                    };
                    let k = kind(meld, i < shown);
                    melds[k] += 1;
                    meld_hu[k] += values.score(meld);
                }
                let layout = rich
                    .layout(dealt[seat.index()])
                    .expect("loose cards are allowed");
                for meld in &layout.melds {
                    let k = kind(meld, false);
                    dealt_melds[k] += 1;
                    dealt_hu[k] += rules.concealed.score(meld);
                }
                rich.clear();
            }
            None if result.foul.is_some() => fouls += 1,
            None => exhausted += 1,
        }
        game.settle(result);
    }
    let won = rounds - exhausted - fouls;
    println!("{rounds} rounds: {won} won, {exhausted} exhausted, {fouls} fouled");
    for seat in Seat::ALL {
        println!(
            "  {seat}: {} wins, score {}",
            wins[seat.index()],
            game.scores()[seat.index()]
        );
    }
    if won > 0 {
        println!("  mean winning hu: {:.2}", f64::from(hu_total) / won as f64);
        println!("\nmelds a win and the 胡 they bring, then the same dealt to the winner:");
        let per_win = |x: u32| f64::from(x) / won as f64;
        for k in 0..KINDS.len() {
            println!(
                "  {}  {:.3}  {:.2}  {:.3}  {:.2}",
                KINDS[k],
                per_win(melds[k]),
                per_win(meld_hu[k]),
                per_win(dealt_melds[k]),
                per_win(dealt_hu[k])
            );
        }
        let total = |hu: &[u32]| per_win(hu.iter().sum());
        println!("  合計  {:.2}  {:.2}", total(&meld_hu), total(&dealt_hu));
    }
}
