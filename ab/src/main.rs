//! A/B: the working tree's `HeuristicBot` in seat 1 against the control's in
//! the other three, paired with four of the control's on the same deal.
//!
//! `new` is the working tree and `old` the revision `run.sh` exported.  A bot
//! reads only its own crate's `View`, so the two crates' rounds are stepped
//! in lockstep, every claim and discard applied to both.  Set `SAMPLES` to
//! seat a `MonteCarloBot` of that many samples as the candidate instead, and
//! `PEEK` as well to seat the Peeker.  `GATE_CARDS` and `GATE_SHANTEN` gate
//! the `MonteCarloBot` (`MonteCarloBot::gated`): it searches only with that
//! many concealed cards or fewer, or within that shanten.  Set `EXPECT` to
//! seat an `ExpectBot` instead, its `Outlook::new` h scaled by that factor
//! (1.5 is `ExpectBot::new`'s at four), or with `FOUR` its `Outlook::default`
//! whatever the table, or with `EXPECT=new` the outlook `ExpectBot::new`
//! ships and, past the gate, `Eval::of`'s tables, and gated at
//! `GATE_CARDS` (default 10), and with `SHALLOW` as well playing past the
//! gate by `ExpectBot::shallow` with that many cards two claims deep, its
//! leaves read from the tables in the file `EVAL` names (an absolute path:
//! `run.sh` runs in `ab/`) in place of `Eval::default`; and with `ORACLE`
//! (`live`, `danger`, or its halves `win` and `claim`) that `ExpectBot`
//! with one input read off the other hands (`oracle.rs`); and with `WARY`
//! `ExpectBot::wary`, with `COUNTING` `ExpectBot::counting`.  Set `BASELINE_SAMPLES` to seat the control's
//! `MonteCarloBot` of that many samples in seat 1 on the control side, gated
//! at `GATE_CARDS` (default 10), in place of its `HeuristicBot`: a head to
//! head of two bots that each beat it.  Set `BASELINE_EXPECT` to seat the
//! control's `ExpectBot::new` there instead, gated as it ships whatever
//! `GATE_CARDS` says, or at `BASELINE_GATE`: a wider gate against the one in
//! place; with `BASELINE_SHALLOW` it plays past its gate as `SHALLOW` has
//! the candidate, its leaves read from the control's `Eval::default`, or
//! with `BASELINE_EXPECT=new` from its `Eval::of`: the control as it ships.
//! `PLAYERS` (default 4) is the table's size on both sides, and
//! `THRESHOLD` (default 10) the 胡 to win: 8 is `Rules::eight`.
//!
//! Set `RESHUFFLE` to K to average out the luck after the two first part
//! ways: the deal is played once with the candidate, and at its first choice
//! the control would not make, the pile left is shuffled K times and each
//! side plays every shuffle out from there.  Before that choice every
//! decision saw only cards already turned up, so the pile's order is still
//! uniform there: the estimate stays unbiased.  A deal that never parts
//! counts 0, as it does in the plain estimate.
//!
//! Set `TRACE` to a file's path (absolute: `run.sh` runs in `ab/`) for a
//! line there on each deal that parts, tab-separated under a header: the
//! deal's seed and dealer, the mean and the standard deviation of the
//! difference among its shuffles, the two win rates, the decisions before
//! the split, the cards left in the pile, and the two acts.  Where they are
//! two discards of an `EXPECT` bot at a table of two at 10 胡, each is
//! instead the card, the bot's value of it, its `Evidence` and
//! `Giveaway::cost`, what the `danger` oracle keeps of the value, and what
//! the other seat does with the card.  Sort by the mean for the deals that
//! moved the most.
#[cfg(feature = "peek")]
mod oracle;
#[cfg(feature = "peek")]
mod peeker;

use new::susik::Card;
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom as _;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::{Mutex, OnceLock};

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

/// `EXPECT`'s bot, before `WARY`, `COUNTING` and `ORACLE`
fn expect_bot(rules: &new::Rules) -> Option<new::ExpectBot> {
    let gate = |name| std::env::var(name).ok().map(|s| s.parse().unwrap());
    // `EXPECT` puts an ExpectBot in the candidate seat, h scaled by it.
    let scale = std::env::var("EXPECT").ok()?;
    // `EXPECT=new` is the bot as it ships: `ExpectBot::new`'s outlook
    // and `Eval::of`'s tables.
    let ships = scale == "new";
    let mut bot = new::ExpectBot::new(rules);
    if !ships {
        let scale: f64 = scale.parse().unwrap();
        let mut outlook = if std::env::var_os("FOUR").is_some() {
            new::Outlook::default()
        } else {
            new::Outlook::new(rules)
        };
        outlook.end.iter_mut().for_each(|h| *h *= scale);
        bot = bot.with_outlook(outlook);
    }
    bot = bot.gated(gate("GATE_CARDS").unwrap_or(10));
    if let Some(deep) = gate("SHALLOW") {
        // `EVAL` reads the tables from a file: ones not built in yet.
        let eval = std::env::var("EVAL").map_or_else(
            |_| {
                if ships {
                    new::Eval::of(rules)
                } else {
                    new::Eval::default()
                }
            },
            |path| new::Eval::from_bytes(&std::fs::read(path).unwrap()).unwrap(),
        );
        bot = bot.shallow(eval, deep);
    }
    Some(bot)
}

fn candidate_bot(rules: &new::Rules, seed: u64) -> Box<dyn new::Strategy> {
    let gate = |name| std::env::var(name).ok().map(|s| s.parse().unwrap());
    if let Some(mut bot) = expect_bot(rules) {
        if std::env::var_os("WARY").is_some() {
            bot = bot.wary();
        }
        if std::env::var_os("COUNTING").is_some() {
            bot = bot.counting();
        }
        #[cfg(feature = "peek")]
        if let Ok(kind) = std::env::var("ORACLE") {
            let (kind, rng) = (oracle::Kind::parse(&kind), SmallRng::seed_from_u64(!seed));
            return Box::new(oracle::Oracle::new(rules, bot, kind, rng));
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
            // `BASELINE_EXPECT=new` reads the control's `Eval::of` too.
            let eval = if std::env::var("BASELINE_EXPECT").is_ok_and(|s| s == "new") {
                old::Eval::of(rules)
            } else {
                old::Eval::default()
            };
            bot = bot.shallow(eval, deep.parse().unwrap());
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

/// `TRACE`'s file, its header written
fn trace() -> Option<&'static Mutex<std::fs::File>> {
    static TRACE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    TRACE
        .get_or_init(|| {
            let mut file = std::fs::File::create(std::env::var_os("TRACE")?).unwrap();
            let card = |side: &str| {
                ["card", "value", "evidence", "cost", "kept", "answer"]
                    .map(|name| format!("\t{side}_{name}"))
                    .concat()
            };
            writeln!(
                file,
                "seed\tdealer\tmean\tsd\tcontrol_wins\tcandidate_wins\tdecisions\tpile\thand{}{}",
                card("control"),
                card("candidate")
            )
            .unwrap();
            Some(Mutex::new(file))
        })
        .as_ref()
}

/// What the other seat does with `card`, discarded by `seat` in `round`:
/// `win`, `kong`, `HeuristicBot`'s `eat`, `pong` or `pass`, or `none` when
/// it is not asked
#[cfg(feature = "peek")]
fn answer(round: &new::Round, seat: new::Seat, card: Card) -> &'static str {
    use new::Strategy as _;
    let mut next = round.clone();
    next.discard(seat, card).unwrap();
    match next.events().get(round.events().len() + 1) {
        Some(&new::Event::Win { seat: other, .. }) if other != seat => return "win",
        Some(&new::Event::Kong { seat: other, .. }) if other != seat => return "kong",
        _ => {}
    }
    let (new::Phase::Claim(other), Some(offer)) = (next.phase(), next.offer()) else {
        return "none";
    };
    if other == seat || offer.drawn || offer.source != seat {
        return "none";
    }
    let view = new::View::new(&next, other);
    match new::HeuristicBot::new(round.rules()).claim(&view, &next.claim_options(other)) {
        new::Claim::Pass => "pass",
        new::Claim::Pong => "pong",
        new::Claim::Eat(_) => "eat",
    }
}

/// `TRACE`'s columns for a split between two discards of an `EXPECT` bot
/// at a table of two at 10 胡: the candidate's concealed cards, and for
/// each discard what the module's documentation lists
#[cfg(feature = "peek")]
fn discards(deck: &[Card], dealer: usize, split: &Split) -> Option<String> {
    let (Act::Discard(control), Act::Discard(candidate)) = (split.control, split.candidate) else {
        return None;
    };
    let mut rules = new::Rules::new();
    (rules.players, rules.threshold) = (players(), threshold());
    if (rules.players, rules.threshold) != (2, 10) {
        return None;
    }
    let mut bot = expect_bot(&rules)?;
    let mut round = new::Round::from_pile(rules, new::Seat::ALL[dealer], deck.to_vec()).unwrap();
    for &act in &split.script {
        match (round.phase(), act) {
            (new::Phase::Claim(seat), Act::Claim(claim)) => {
                round.claim(seat, claim).unwrap();
            }
            (new::Phase::Discard(seat), Act::Discard(card)) => {
                round.discard(seat, card).unwrap();
            }
            step => panic!("the script fell out of step: {step:?}"),
        }
    }
    let seat = new::Seat::ALL[CANDIDATE];
    let view = new::View::new(&round, seat);
    let ranked = bot.discards(&view);
    let giveaway = new::Giveaway::new(&view);
    // The oracle reads the hands as they are; its rng is for a pile it
    // never shuffles here.
    let rng = SmallRng::seed_from_u64(0);
    let mut oracle = oracle::Oracle::new(&rules, bot, oracle::Kind::Danger, rng);
    let mut out = view.hand().len().to_string();
    for card in [control, candidate] {
        let value = ranked
            .iter()
            .find(|&&(ranked, _)| ranked == card)
            .map_or(f64::NAN, |&(_, value)| value);
        write!(
            out,
            "\t{card}\t{value:.3}\t{:?}\t{:.3}\t{:.3}\t{}",
            giveaway.evidence(card),
            giveaway.cost(card),
            oracle.kept(&round, seat, card, value),
            answer(&round, seat, card)
        )
        .unwrap();
    }
    Some(out)
}

/// `TRACE`'s columns for a split, after the pile's
fn detail(deck: &[Card], dealer: usize, split: &Split) -> String {
    #[cfg(feature = "peek")]
    if let Some(columns) = discards(deck, dealer, split) {
        return columns;
    }
    let _ = (deck, dealer);
    format!("\t{:?}\t{:?}", split.control, split.candidate)
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
    let detail = trace().map(|_| detail(&deck, dealer, &split));
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
    if let (Some(file), Some(detail)) = (trace(), detail) {
        writeln!(
            file.lock().unwrap(),
            "{seed}\t{dealer}\t{mean:.4}\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\t{detail}",
            within.sqrt(),
            cw / k,
            tw / k,
            split.script.len(),
            split.left
        )
        .unwrap();
    }
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
