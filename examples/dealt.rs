//! What a dealt hand holds: the 胡 already in the twenty cards of a seat
//! that is not the dealer, the melds they come from, and the cards left
//! outside every meld.  No bot plays; these are facts of the deal.
//!
//! ```sh
//! cargo run --release --example dealt -- [deals] [seed]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::susik::{Meld, Solver};
use tsap_oo::{Round, Rules, Seat};

const KINDS: [&str; 7] = [
    "將帥",
    "將士象",
    "車馬包",
    "三色兵卒",
    "四色兵卒",
    "暗刻",
    "暗槓",
];

/// The row of [`KINDS`] a meld counts under; pairs score nothing
const fn kind(meld: &Meld) -> Option<usize> {
    Some(match meld {
        Meld::Single(_) => 0,
        Meld::Court(_) => 1,
        Meld::Field(_) => 2,
        Meld::Soldiers(colors) if colors.len() == 3 => 3,
        Meld::Soldiers(_) => 4,
        Meld::Triplet(_) => 5,
        Meld::Quad(_) => 6,
        Meld::Pair(_) => return None,
    })
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let deals = args.next().flatten().unwrap_or(10_000);
    let seed = args.next().flatten().unwrap_or(0);
    let rules = Rules::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    // The layout worth the most 胡, and the one with the fewest loose cards
    let mut rich = Solver::new(rules.concealed, Some(0)).locked();
    let mut tidy = Solver::new(rules.concealed, Some(100)).locked();
    // Hands by 胡 and by loose cards; a hand is twenty cards
    let (mut by_hu, mut by_loose) = ([0u64; 41], [0u64; 21]);
    let mut from = [0u64; 7];
    let mut hands = 0u64;
    for _ in 0..deals {
        let round = Round::deal(rules, Seat::ALL[0], &mut rng);
        for seat in &Seat::ALL[1..] {
            let hand = round.hand(*seat);
            let layout = rich.layout(hand).expect("loose cards are allowed");
            by_hu[usize::try_from(layout.score).unwrap_or(0).min(40)] += 1;
            for meld in &layout.melds {
                if let Some(kind) = kind(meld) {
                    from[kind] += u64::from(rules.concealed.score(meld));
                }
            }
            let layout = tidy.layout(hand).expect("loose cards are allowed");
            by_loose[layout.loose.len()] += 1;
            hands += 1;
        }
        rich.clear();
        tidy.clear();
    }

    let share = |n: u64| 100.0 * n as f64 / hands as f64;
    let mean = |counts: &[u64]| {
        let total: u64 = counts.iter().zip(0..).map(|(&n, i)| n * i).sum();
        total as f64 / hands as f64
    };
    println!("{hands} hands of twenty cards, seed {seed}");
    println!("\n胡 in the hand: mean {:.2}", mean(&by_hu));
    let mut below = 0;
    for (hu, &n) in by_hu.iter().enumerate().filter(|&(_, &n)| n > 0) {
        below += n;
        println!(
            "  {hu:2}  {:6.2}%  up to here {:6.2}%",
            share(n),
            share(below)
        );
    }
    println!("\n胡 a hand, by meld:");
    for (name, &n) in KINDS.iter().zip(&from) {
        println!("  {name}  {:.2}", n as f64 / hands as f64);
    }
    println!("\nloose cards: mean {:.2}", mean(&by_loose));
    for (loose, &n) in by_loose.iter().enumerate().filter(|&(_, &n)| n > 0) {
        println!("  {loose:2}  {:6.2}%", share(n));
    }
}
