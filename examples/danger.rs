//! How often each kind of card would deal in: four [`HeuristicBot`]s play,
//! and at every discard each card that could come down is tested against the
//! other three hands.  The numbers behind `research/discard-danger.md`.
//!
//! ```sh
//! cargo run --release --example danger -- [rounds] [seed base]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::susik::{Card, Hand, Meld, Rank, Solver, locked_completions};
use tsap_oo::{HeuristicBot, Phase, Round, Rules, Seat, Strategy as _, View};

/// Stages of a round, by the cards left in the pile
const STAGES: [&str; 3] = ["early (23+)", "mid (14-22)", "late (-13)"];
const FAMILIES: [&str; 3] = ["士象", "車馬包", "兵卒"];

/// Per family: cards tested, then how many of them win for another seat at
/// all, into a pair, into a triplet, into a run; the 胡 of those wins; and
/// how many the next seat could eat into a run
type Tally = [[u64; 7]; 3];

const fn family(rank: Rank) -> usize {
    match rank {
        Rank::General | Rank::Advisor | Rank::Elephant => 0,
        Rank::Chariot | Rank::Horse | Rank::Cannon => 1,
        Rank::Soldier => 2,
    }
}

const fn run(meld: &Meld) -> bool {
    matches!(meld, Meld::Court(_) | Meld::Field(_) | Meld::Soldiers(_))
}

/// Add `card`, put down by `seat`, to `tally`
fn test(tally: &mut Tally, round: &Round, rules: &Rules, seat: Seat, card: Card) {
    let row = &mut tally[family(card.rank)];
    let mut solver = Solver::new(rules.concealed, None).locked();
    // (at all, pair, triplet, run) over the other seats, and the most 胡
    let (mut wins, mut most) = ([false; 4], 0);
    for other in [1, 2, 3].map(|n| seat.advance(n)) {
        let hand = round.hand(other);
        let shown: u32 = round
            .exposed(other)
            .iter()
            .map(|meld| rules.exposed.score(meld))
            .sum();
        for meld in locked_completions(hand, card) {
            let Some(layout) = solver.layout(hand - meld.cards().without(card)) else {
                continue;
            };
            let hu = shown + rules.exposed.score(&meld) + u32::try_from(layout.score).unwrap_or(0);
            if hu < rules.threshold {
                continue;
            }
            wins[0] = true;
            wins[1] |= matches!(meld, Meld::Pair(_));
            wins[2] |= matches!(meld, Meld::Triplet(_) | Meld::Quad(_));
            wins[3] |= run(&meld);
            most = most.max(hu);
        }
    }
    row[0] += 1;
    for (count, win) in row[1..].iter_mut().zip(wins) {
        *count += u64::from(win);
    }
    row[5] += u64::from(most);
    row[6] += u64::from(
        locked_completions(round.hand(seat.next()), card)
            .iter()
            .any(run),
    );
}

/// Play round `seed`: every card that could come down, and the discards made
fn play(seed: u64, any: &mut [Tally; 3], made: &mut [Tally; 3]) {
    let rules = Rules::new();
    let dealer = Seat::ALL[(seed % 4) as usize];
    let mut round = Round::deal(rules, dealer, &mut SmallRng::seed_from_u64(seed));
    loop {
        match round.phase() {
            Phase::Finished => break,
            Phase::Claim(seat) => {
                let claim = HeuristicBot::new(&rules)
                    .claim(&View::new(&round, seat), &round.claim_options(seat));
                round.claim(seat, claim).expect("the bot claims legally");
            }
            Phase::Discard(seat) => {
                let stage = match round.pile_len() {
                    23.. => 0,
                    14..=22 => 1,
                    _ => 2,
                };
                let others = [1, 2, 3]
                    .into_iter()
                    .flat_map(|n| round.hand(seat.advance(n)).iter())
                    .fold(Hand::EMPTY, Hand::with);
                for (card, _) in Hand::FULL.distinct() {
                    // Generals stay put, and a card the other seats hold
                    // every copy of cannot come down on them.
                    if card.rank != Rank::General && others.count(card) < 4 {
                        test(&mut any[stage], &round, &rules, seat, card);
                    }
                }
                let card = HeuristicBot::new(&rules).discard(&View::new(&round, seat));
                test(&mut made[stage], &round, &rules, seat, card);
                round.discard(seat, card).expect("the bot discards legally");
            }
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(|arg| arg.parse::<u64>());
    let rounds = args.next().map_or(20_000, |n| n.expect("a round count"));
    let base = args.next().map_or(20_261_116, |n| n.expect("a seed base"));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let mut total = [[Tally::default(); 3]; 2];
    std::thread::scope(|scope| {
        let shards: Vec<_> = (0..threads as u64)
            .map(|shard| {
                scope.spawn(move || {
                    let mut tallies = [[Tally::default(); 3]; 2];
                    for i in (shard..rounds).step_by(threads) {
                        let [any, made] = &mut tallies;
                        play(base + i, any, made);
                    }
                    tallies
                })
            })
            .collect();
        for shard in shards {
            let tallies = shard.join().expect("a shard panicked");
            let cells = total
                .as_flattened_mut()
                .as_flattened_mut()
                .as_flattened_mut();
            let more = tallies.as_flattened().as_flattened().as_flattened();
            for (cell, n) in cells.iter_mut().zip(more) {
                *cell += n;
            }
        }
    });
    println!("{rounds} rounds, seed base {base}");
    for (name, tallies) in ["any card that could come down", "the discards made"]
        .into_iter()
        .zip(total)
    {
        println!("\n{name}");
        println!("stage        family  cards     win%   pair% triplet%  run%    胡   eat%");
        for (stage, tally) in STAGES.into_iter().zip(tallies) {
            for (family, row) in FAMILIES.into_iter().zip(tally) {
                let pct = |n: u64| 100.0 * n as f64 / row[0].max(1) as f64;
                println!(
                    "{stage:12} {family}\t{:8} {:6.3} {:6.3} {:6.3} {:6.3} {:5.2} {:6.3}",
                    row[0],
                    pct(row[1]),
                    pct(row[2]),
                    pct(row[3]),
                    pct(row[4]),
                    row[5] as f64 / row[1].max(1) as f64,
                    pct(row[6]),
                );
            }
        }
    }
}
