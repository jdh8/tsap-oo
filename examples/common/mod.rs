//! Self-play spread over the cores, with the output of a serial loop

use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::{Round, Rules, Seat};

/// `play` on each of `rounds` deals from `seed`, on every core, in deal order
///
/// The piles are the ones a `Game` would deal from `seed`, but seat 1 deals
/// them all.  Play only rotates with the dealer, so an example that never
/// reads absolute seats (only the dealer, the winner and the seats relative
/// to them) prints what the serial `game.deal` and `game.settle` loop did.
pub fn deals<T: Send>(
    rules: Rules,
    rounds: u64,
    seed: u64,
    play: impl Fn(Round) -> T + Sync,
) -> Vec<T> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let piles: Vec<_> = (0..rounds)
        .map(|_| tsap_oo::susik::shuffle(&mut rng))
        .collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let chunk = piles.len().div_ceil(threads).max(1);
    let play = &play;
    std::thread::scope(|scope| {
        let shards: Vec<_> = piles
            .chunks(chunk)
            .map(|piles| {
                scope.spawn(move || {
                    piles
                        .iter()
                        .map(|pile| {
                            let round = Round::from_pile(rules, Seat::ALL[0], pile.clone());
                            play(round.expect("a shuffled deck is the full deck"))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        shards
            .into_iter()
            .flat_map(|shard| shard.join().expect("a round panicked"))
            .collect()
    })
}
