//! What the compulsory win proves: four [`HeuristicBot`]s play, and at every
//! discard each card the seat could put down is tested against each other
//! seat, apart by whether [`View::safe`] rules that seat out.  The numbers
//! behind `research/safe-cards.md`.
//!
//! ```sh
//! cargo run --release --example safe -- [rounds] [seed base]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::susik::{Card, Rank, Solver, locked_completions};
use tsap_oo::{HeuristicBot, Phase, Round, Rules, Seat, Strategy as _, View, discardable};

/// Rows: the concealed cards of the seat tested, in pairs from 20–19 down
const ROWS: usize = 10;

/// Per row and family: cards safe, cards not, and the wins among each
type Tally = [[[u64; 4]; 3]; ROWS];

const fn family(rank: Rank) -> usize {
    match rank {
        Rank::General | Rank::Advisor | Rank::Elephant => 0,
        Rank::Chariot | Rank::Horse | Rank::Cannon => 1,
        Rank::Soldier => 2,
    }
}

/// Whether `seat` wins on `card`, as [`Round`] decides it
fn wins(round: &Round, seat: Seat, card: Card) -> bool {
    let rules = round.rules();
    let hand = round.hand(seat);
    let shown: u32 = round
        .exposed(seat)
        .iter()
        .map(|meld| rules.exposed.score(meld))
        .sum();
    let mut solver = Solver::new(rules.concealed, None).locked();
    locked_completions(hand, card).into_iter().any(|meld| {
        solver
            .layout(hand - meld.cards().without(card))
            .is_some_and(|layout| {
                shown + rules.exposed.score(&meld) + u32::try_from(layout.score).unwrap_or(0)
                    >= rules.threshold
            })
    })
}

/// Play round `seed`: every card each seat could discard, against each
/// other seat; then the same for the discards made
fn play(seed: u64, any: &mut Tally, made: &mut Tally) {
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
                let view = View::new(&round, seat);
                let choice = HeuristicBot::new(&rules).discard(&view);
                for other in [1, 2, 3].map(|n| seat.advance(n, Seat::COUNT)) {
                    let safe = view.safe(other);
                    let row = (20 - view.concealed_len(other).min(20)) / 2;
                    for (card, _) in discardable(view.hand(), view.locked()).distinct() {
                        let (proven, won) = (safe.contains(card), wins(&round, other, card));
                        assert!(!(proven && won), "{card} was proven safe");
                        let cell = usize::from(!proven);
                        for tally in [&mut *any]
                            .into_iter()
                            .chain((card == choice).then_some(&mut *made))
                        {
                            tally[row.min(ROWS - 1)][family(card.rank)][cell] += 1;
                            tally[row.min(ROWS - 1)][family(card.rank)][2 + cell] += u64::from(won);
                        }
                    }
                }
                round
                    .discard(seat, choice)
                    .expect("the bot discards legally");
            }
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(|arg| arg.parse::<u64>());
    let rounds = args.next().map_or(20_000, |n| n.expect("a round count"));
    let base = args.next().map_or(20_261_117, |n| n.expect("a seed base"));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let mut total = [Tally::default(); 2];
    std::thread::scope(|scope| {
        let shards: Vec<_> = (0..threads as u64)
            .map(|shard| {
                scope.spawn(move || {
                    let mut tallies = [Tally::default(); 2];
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
    for (name, tally) in ["any card the seat could discard", "the discards made"]
        .into_iter()
        .zip(total)
    {
        println!("\n{name}, against one other seat");
        println!("concealed family     pairs  safe%  win% of all  win% of unsafe");
        for (row, families) in tally.into_iter().enumerate() {
            for (family, [safe, not, _, won]) in
                ["士象", "車馬炮", "兵卒"].into_iter().zip(families)
            {
                let all = (safe + not).max(1) as f64;
                println!(
                    "{:2}-{:<2}    {family}\t{:9} {:6.2} {:8.3} {:12.3}",
                    20 - 2 * row,
                    if row == ROWS - 1 { 0 } else { 19 - 2 * row },
                    safe + not,
                    100.0 * safe as f64 / all,
                    100.0 * won as f64 / all,
                    100.0 * won as f64 / not.max(1) as f64,
                );
            }
        }
    }
}
