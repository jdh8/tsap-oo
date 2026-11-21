//! A/B: the working tree's `HeuristicBot` in seat 1 against the control's in
//! the other three, paired with four of the control's on the same deal.
//!
//! `new` is the working tree and `old` the revision `run.sh` exported.  A bot
//! reads only its own crate's `View`, so the two crates' rounds are stepped
//! in lockstep, every claim and discard applied to both.  Set `SAMPLES` to
//! seat a `MonteCarloBot` of that many samples as the candidate instead, and
//! `PEEK` as well to seat the Peeker.  `GATE_CARDS` and `GATE_SHANTEN` gate
//! the `MonteCarloBot` (`MonteCarloBot::gated`): it searches only with that
//! many concealed cards or fewer, or within that 向聽數.  Set `EXPECT` to
//! seat an `ExpectBot` instead, its `Outlook::new` h scaled by that factor
//! (1.5 is `ExpectBot::new`'s at four), or with `FOUR` its `Outlook::default`
//! whatever the table, and gated at
//! `GATE_CARDS` (default 10), and with `SHALLOW` as well playing past the
//! gate by `ExpectBot::shallow` with that many cards two claims deep.  Set
//! `BASELINE_SAMPLES` to seat the control's
//! `MonteCarloBot` of that many samples in seat 1 on the control side, gated
//! at `GATE_CARDS` (default 10), in place of its `HeuristicBot`: a head to
//! head of two bots that each beat it.  Set `BASELINE_EXPECT` to seat the
//! control's `ExpectBot::new` there instead, gated as it ships whatever
//! `GATE_CARDS` says, or at `BASELINE_GATE`: a wider gate against the one in
//! place; with `BASELINE_SHALLOW` it plays past its gate as `SHALLOW` has
//! the candidate.  `PLAYERS` (default 4) is the table's size on both sides, and
//! `THRESHOLD` (default 10) the 胡 to win: 8 is 八胡.
//!
//! Set `RESHUFFLE` to K to average out the luck after the two first part
//! ways: the deal is played once with the candidate, and at its first choice
//! the control would not make, the pile left is shuffled K times and each
//! side plays every shuffle out from there.  Before that choice every
//! decision saw only cards already turned up, so the pile's order is still
//! uniform there: the estimate stays unbiased.  A deal that never parts
//! counts 0, as it does in the plain estimate.
#[cfg(feature = "peek")]
mod peeker;

use new::susik::Card;
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom as _;

const CANDIDATE: usize = 1;

/// The table's size: `PLAYERS`, or four
fn players() -> usize {
    std::env::var("PLAYERS").map_or(new::Seat::COUNT, |s| s.parse().unwrap())
}

/// The 胡 to win: `THRESHOLD`, or ten
fn threshold() -> u32 {
    std::env::var("THRESHOLD").map_or(10, |s| s.parse().unwrap())
}

/// How many times the pile is reshuffled where the two part, unless
/// `RESHUFFLE` says; 0 is the plain paired estimate
// ponytail: set on `1b644f2` against `6fc8f21` (`research/reshuffle.md`):
// 8 and 16 cost the same per unit of precision, and 8 is cheaper the more
// often a candidate parts.
const RESHUFFLE: usize = 8;

/// A decision, as both crates take it
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Act {
    Claim(new::Claim),
    Discard(Card),
}

/// Where the candidate first parts from the control
struct Split {
    /// The decisions before it
    script: Vec<Act>,
    control: Act,
    candidate: Act,
    /// Cards left in the pile: the deck's first `left`
    left: usize,
}

/// Seat 1's score and whether it won, or where the candidate first parted
enum Outcome {
    Done(i32, bool),
    Split(Split),
}

fn candidate_bot(rules: &new::Rules, seed: u64) -> Box<dyn new::Strategy> {
    let gate = |name| std::env::var(name).ok().map(|s| s.parse().unwrap());
    // `EXPECT` puts an ExpectBot in the candidate seat, h scaled by it.
    if let Ok(scale) = std::env::var("EXPECT") {
        let scale: f64 = scale.parse().unwrap();
        let mut outlook = if std::env::var_os("FOUR").is_some() {
            new::Outlook::default()
        } else {
            new::Outlook::new(rules)
        };
        outlook.end.iter_mut().for_each(|h| *h *= scale);
        let mut bot = new::ExpectBot::new(rules)
            .with_outlook(outlook)
            .gated(gate("GATE_CARDS").unwrap_or(10));
        if let Some(deep) = gate("SHALLOW") {
            bot = bot.shallow(new::Eval::default(), deep);
        }
        return Box::new(bot);
    }
    // `SAMPLES` puts a MonteCarloBot in the candidate seat, its rng
    // derived from the deal's seed so a deal replays.
    let Ok(n) = std::env::var("SAMPLES") else {
        return Box::new(new::HeuristicBot::new(rules));
    };
    let (n, rng) = (n.parse().unwrap(), SmallRng::seed_from_u64(!seed));
    #[cfg(feature = "peek")]
    if std::env::var_os("PEEK").is_some() {
        return Box::new(peeker::Peeker::new(rules, n, rng));
    }
    // `GATE_CARDS` and `GATE_SHANTEN` are `MonteCarloBot::gated`'s two.
    let cards = gate("GATE_CARDS").unwrap_or(usize::MAX);
    let shanten = gate("GATE_SHANTEN").map(|n| n as u32);
    Box::new(new::MonteCarloBot::new(rules, n, rng).gated(cards, shanten))
}

fn baseline_bot(rules: &old::Rules, seed: u64) -> Option<Box<dyn old::Strategy>> {
    if std::env::var_os("BASELINE_EXPECT").is_some() {
        let mut bot = old::ExpectBot::new(rules);
        if let Ok(cards) = std::env::var("BASELINE_GATE") {
            bot = bot.gated(cards.parse().unwrap());
        }
        if let Ok(deep) = std::env::var("BASELINE_SHALLOW") {
            bot = bot.shallow(old::Eval::default(), deep.parse().unwrap());
        }
        return Some(Box::new(bot));
    }
    let n = std::env::var("BASELINE_SAMPLES").ok()?.parse().unwrap();
    let cards = std::env::var("GATE_CARDS").map_or(10, |s| s.parse().unwrap());
    let rng = SmallRng::seed_from_u64(!seed);
    Some(Box::new(
        old::MonteCarloBot::new(rules, n, rng).gated(cards, None),
    ))
}

fn to_new(claim: old::Claim) -> new::Claim {
    match claim {
        old::Claim::Pass => new::Claim::Pass,
        old::Claim::Pong => new::Claim::Pong,
        old::Claim::Eat(meld) => new::Claim::Eat(meld),
    }
}

fn to_old(claim: new::Claim) -> old::Claim {
    match claim {
        new::Claim::Pass => old::Claim::Pass,
        new::Claim::Pong => old::Claim::Pong,
        new::Claim::Eat(meld) => old::Claim::Eat(meld),
    }
}

/// Play `deck` out, taking the first decisions from `script`, with the
/// candidate in seat 1 or not; with `split`, stop at the candidate's first
/// choice the control would not make
fn play(
    deck: &[Card],
    seed: u64,
    dealer: usize,
    script: &[Act],
    candidate: bool,
    split: bool,
) -> Outcome {
    let (mut nr, mut or) = (new::Rules::new(), old::Rules::new());
    (nr.players, or.players) = (players(), players());
    (nr.threshold, or.threshold) = (threshold(), threshold());
    let mut a = new::Round::from_pile(nr, new::Seat::ALL[dealer], deck.to_vec()).unwrap();
    let mut b = old::Round::from_pile(or, old::Seat::ALL[dealer], deck.to_vec()).unwrap();
    let mut bot = candidate_bot(&nr, seed);
    let mut baseline = baseline_bot(&or, seed);
    let mut done: Vec<Act> = Vec::new();
    loop {
        let (s, t, claim) = match (a.phase(), b.phase()) {
            (new::Phase::Finished, old::Phase::Finished) => break,
            (new::Phase::Claim(s), old::Phase::Claim(t)) => (s, t, true),
            (new::Phase::Discard(s), old::Phase::Discard(t)) => (s, t, false),
            phases => panic!("the rounds fell out of step: {phases:?}"),
        };
        assert_eq!(s.index(), t.index());
        let mut control = || {
            let view = old::View::new(&b, t);
            let mut heuristic;
            let bot: &mut dyn old::Strategy = match baseline.as_deref_mut() {
                Some(bot) if s.index() == CANDIDATE => bot,
                _ => {
                    heuristic = old::HeuristicBot::new(&or);
                    &mut heuristic
                }
            };
            if claim {
                Act::Claim(to_new(bot.claim(&view, &b.claim_options(t))))
            } else {
                Act::Discard(bot.discard(&view))
            }
        };
        let act = if let Some(&act) = script.get(done.len()) {
            act
        } else if candidate && s.index() == CANDIDATE {
            let view = new::View::new(&a, s);
            let ours = if claim {
                Act::Claim(bot.claim(&view, &a.claim_options(s)))
            } else {
                Act::Discard(bot.discard(&view))
            };
            if split {
                let theirs = control();
                if ours != theirs {
                    return Outcome::Split(Split {
                        script: done,
                        control: theirs,
                        candidate: ours,
                        left: a.pile_len(),
                    });
                }
            }
            ours
        } else {
            control()
        };
        match act {
            Act::Claim(c) => {
                a.claim(s, c).unwrap();
                b.claim(t, to_old(c)).unwrap();
            }
            Act::Discard(card) => {
                a.discard(s, card).unwrap();
                b.discard(t, card).unwrap();
            }
        }
        done.push(act);
    }
    let result = a.result().unwrap();
    let mut game = new::Game::new(nr, new::Seat::ALL[dealer]);
    game.settle(result);
    Outcome::Done(
        game.scores()[CANDIDATE],
        result.winner.map(new::Seat::index) == Some(CANDIDATE),
    )
}

fn done(outcome: Outcome) -> (i32, bool) {
    match outcome {
        Outcome::Done(score, won) => (score, won),
        Outcome::Split(_) => unreachable!("only a split play stops early"),
    }
}

/// Sums over the deals of the score difference `d`, its square and the two
/// win rates; how many differ (or part, with reshuffles); and over the deals
/// that part, the sums of `d`, `d²` and the variance among the shuffles
#[derive(Default)]
struct Totals {
    sum: f64,
    squares: f64,
    differ: u64,
    control: f64,
    candidate: f64,
    means: f64,
    mean_squares: f64,
    within: f64,
}

/// One deal: the score difference and the two win rates, and with
/// `reshuffles` > 0 and a split, the variance among the shuffles
fn deal(seed: u64, dealer: usize, reshuffles: usize) -> (f64, f64, f64, Option<f64>) {
    let deck = new::susik::shuffle(&mut SmallRng::seed_from_u64(seed));
    if reshuffles == 0 {
        let (control, cw) = done(play(&deck, seed, dealer, &[], false, false));
        let (treated, tw) = done(play(&deck, seed, dealer, &[], true, false));
        return (
            f64::from(treated - control),
            f64::from(u8::from(cw)),
            f64::from(u8::from(tw)),
            None,
        );
    }
    let split = match play(&deck, seed, dealer, &[], true, true) {
        Outcome::Done(_, won) => {
            let won = f64::from(u8::from(won));
            return (0.0, won, won, None);
        }
        Outcome::Split(split) => split,
    };
    let mut rng = SmallRng::seed_from_u64(seed ^ 0x9e37_79b9_7f4a_7c15);
    let (mut c, mut t) = (split.script.clone(), split.script.clone());
    c.push(split.control);
    t.push(split.candidate);
    let mut deck = deck;
    let (mut sum, mut squares, mut cw, mut tw) = (0.0, 0.0, 0.0, 0.0);
    for _ in 0..reshuffles {
        deck[..split.left].shuffle(&mut rng);
        let (control, won) = done(play(&deck, seed, dealer, &c, false, false));
        cw += f64::from(u8::from(won));
        let (treated, won) = done(play(&deck, seed, dealer, &t, true, false));
        tw += f64::from(u8::from(won));
        let d = f64::from(treated - control);
        sum += d;
        squares += d * d;
    }
    let k = reshuffles as f64;
    let mean = sum / k;
    let within = if reshuffles > 1 {
        (squares - k * mean * mean) / (k - 1.0)
    } else {
        0.0
    };
    (mean, cw / k, tw / k, Some(within))
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().unwrap());
    let (base, n) = (args.next().unwrap(), args.next().unwrap());
    let reshuffles: usize = std::env::var("RESHUFFLE").map_or(RESHUFFLE, |k| k.parse().unwrap());
    let threads = std::thread::available_parallelism().unwrap().get() as u64;
    let total = std::thread::scope(|scope| {
        let shards: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut acc = Totals::default();
                    for i in (t..n).step_by(threads as usize) {
                        let dealer = (i % players() as u64) as usize;
                        let (d, cw, tw, within) = deal(base + i, dealer, reshuffles);
                        acc.sum += d;
                        acc.squares += d * d;
                        acc.control += cw;
                        acc.candidate += tw;
                        if let Some(within) = within {
                            acc.differ += 1;
                            acc.means += d;
                            acc.mean_squares += d * d;
                            acc.within += within;
                        } else {
                            acc.differ += u64::from(d != 0.0);
                        }
                    }
                    acc
                })
            })
            .collect();
        shards
            .into_iter()
            .map(|h| h.join().unwrap())
            .fold(Totals::default(), |a, b| Totals {
                sum: a.sum + b.sum,
                squares: a.squares + b.squares,
                differ: a.differ + b.differ,
                control: a.control + b.control,
                candidate: a.candidate + b.candidate,
                means: a.means + b.means,
                mean_squares: a.mean_squares + b.mean_squares,
                within: a.within + b.within,
            })
    });
    let nf = n as f64;
    let mean = total.sum / nf;
    let se = ((total.squares - nf * mean * mean) / (nf - 1.0) / nf).sqrt();
    let (lo, hi) = (mean - 1.96 * se, mean + 1.96 * se);
    if reshuffles == 0 {
        println!("seed base {base}, {n} deals, {} differ", total.differ);
    } else {
        println!(
            "seed base {base}, {n} deals, {} part, each pile reshuffled {reshuffles} times",
            total.differ
        );
        // One-way analysis of variance over the deals that part: the spread
        // of one shuffle's difference about its deal's mean, and the spread
        // of those means once the shuffles' share is taken out.
        let m = total.differ as f64;
        let within = total.within / m;
        let spread = (total.mean_squares - total.means * total.means / m) / (m - 1.0);
        let between = spread - within / reshuffles as f64;
        if m > 1.0 {
            println!(
                "among deals that part: mean {:+.3}, sd within a deal {:.2}, between deals {:.2}",
                total.means / m,
                within.sqrt(),
                between.max(0.0).sqrt()
            );
        }
    }
    println!(
        "seat 1 wins: control {:.2}%, candidate {:.2}%",
        100.0 * total.control / nf,
        100.0 * total.candidate / nf
    );
    println!(
        "candidate − control: {mean:+.3} ± {se:.3} points a deal, 95% CI [{lo:+.3}, {hi:+.3}]"
    );
    println!(
        "Verdict: {}",
        if lo > 0.0 {
            "win"
        } else if hi < 0.0 {
            "loss"
        } else {
            "inconclusive"
        }
    );
}
