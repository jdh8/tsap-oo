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
//! A third argument seeds the deals (0 by default).
//!
//! The deals are shuffled first, on one thread, so a seed gives the same
//! deals whatever the machine.  Four `HeuristicBot`s then play them on every
//! core: the table is the same from each seat, so a deal is played from the
//! first seat and turned to its dealer, the winner before, afterwards.  A
//! Monte Carlo bot keeps its seat, and those rounds are played one by one.

use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::susik::{Card, Hand, Meld, Solver, shuffle};
use tsap_oo::{
    Game, HeuristicBot, MonteCarloBot, Round, RoundResult, Rules, Seat, Step, Strategy, Table,
};

/// Rows of the winning-layout table; 碰 and 明槓 are melds shown on the
/// table (claimed, or completed by the winning card), 暗刻 and 暗槓 are not
const KINDS: [&str; 10] = [
    "將帥",
    "將士象",
    "車馬炮",
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

/// A finished round: its result, how many melds the winner had shown, and
/// the hand the winner was dealt
struct Played {
    result: RoundResult,
    shown: usize,
    dealt: Hand,
}

/// Play `pile` out with `dealer` dealing, a Monte Carlo bot in the first
/// seat if given
fn play(rules: Rules, dealer: Seat, pile: Vec<Card>, first: Option<(usize, SmallRng)>) -> Played {
    let mut bots =
        Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
    if let Some((samples, seed)) = first {
        bots[0] = Some(Box::new(MonteCarloBot::new(&rules, samples, seed)));
    }
    let round = Round::from_pile(rules, dealer, pile).expect("a shuffled deck is the full deck");
    let dealt = Seat::ALL.map(|seat| round.hand(seat));
    let mut table = Table::new(round, bots);
    while table.step().expect("bots act legally") != Step::Finished {}
    let result = table.round().result().expect("finished").clone();
    let (shown, dealt) = result.winner.map_or((0, Hand::EMPTY), |seat| {
        (table.round().exposed(seat).len(), dealt[seat.index()])
    });
    Played {
        result,
        shown,
        dealt,
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
    // The draws of a run that deals and plays round by round, in its order
    let deals: Vec<_> = (0..rounds)
        .map(|_| {
            let seed = samples.map(|_| SmallRng::from_rng(&mut rng));
            (seed, shuffle(&mut rng))
        })
        .collect();
    // Under two samples a Monte Carlo bot plays as `HeuristicBot`
    let alike = samples.is_none_or(|samples| samples < 2);
    let mut ahead = Vec::new();
    if alike && rounds > 0 {
        let threads = std::thread::available_parallelism().map_or(1, usize::from);
        std::thread::scope(|scope| {
            let workers: Vec<_> = deals
                .chunks(rounds.div_ceil(threads))
                .map(|chunk| {
                    scope.spawn(move || {
                        let play = |(_, pile): &(_, Vec<Card>)| {
                            play(rules, Seat::ALL[0], pile.clone(), None)
                        };
                        chunk.iter().map(play).collect::<Vec<_>>()
                    })
                })
                .collect();
            for worker in workers {
                ahead.extend(worker.join().expect("a round panicked"));
            }
        });
    }
    let mut ahead = ahead.into_iter();
    for (seed, pile) in deals {
        let dealer = game.dealer();
        let Played {
            result,
            shown,
            dealt,
        } = match ahead.next() {
            // Played from the first seat: turn the seats to the dealer
            Some(mut played) => {
                let turn = |seat: Seat| dealer.advance(seat.index(), rules.players);
                played.result.winner = played.result.winner.map(turn);
                played.result.foul = played.result.foul.map(turn);
                played
            }
            None => play(rules, dealer, pile, samples.zip(seed)),
        };
        let result = &result;
        match result.winner {
            Some(seat) => {
                wins[seat.index()] += 1;
                hu_total += result.hu;
                // Exposed melds come first, then the winning card's meld
                let shown = shown + usize::from(result.winning_card.is_some());
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
                let layout = rich.layout(dealt).expect("loose cards are allowed");
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
