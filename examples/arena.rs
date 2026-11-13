//! Four greedy bots play a number of rounds and report the outcomes.
//!
//! ```console
//! cargo run --release --example arena -- 1000
//! ```

use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::{Game, HeuristicBot, Rules, Seat, Step, Table};

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let rules = Rules::new();
    let mut game = Game::new(rules, Seat::East);
    let mut rng = SmallRng::seed_from_u64(0);
    let mut wins = [0; 4];
    let mut exhausted = 0;
    let mut hu_total = 0;
    for _ in 0..rounds {
        let bots = Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as _));
        let mut table = Table::new(game.deal(&mut rng), bots);
        while table.step().expect("bots act legally") != Step::Finished {}
        let result = table.round().result().expect("finished");
        match result.winner {
            Some(seat) => {
                wins[seat as usize] += 1;
                hu_total += result.hu;
            }
            None => exhausted += 1,
        }
        game.settle(result);
    }
    let won = rounds - exhausted;
    println!("{rounds} rounds: {won} won, {exhausted} exhausted");
    for seat in Seat::ALL {
        println!(
            "  {seat}: {} wins, score {}",
            wins[seat as usize],
            game.scores()[seat as usize]
        );
    }
    if won > 0 {
        println!("  mean winning hu: {:.2}", f64::from(hu_total) / won as f64);
    }
}
