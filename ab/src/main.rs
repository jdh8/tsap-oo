//! A/B: the working tree's `HeuristicBot` in seat 1 against the control's in
//! the other three, paired with four of the control's on the same deal.
//!
//! `new` is the working tree and `old` the revision `run.sh` exported.  A bot
//! reads only its own crate's `View`, so the two crates' rounds are stepped
//! in lockstep, every claim and discard applied to both.
use new::Strategy as _;
use old::Strategy as _;
use rand::SeedableRng as _;
use rand::rngs::SmallRng;

const CANDIDATE: usize = 1;

/// Seat 1's score and whether it won, with the candidate there or not
fn play(seed: u64, dealer: usize, candidate: bool) -> (i32, bool) {
    let (nr, or) = (new::Rules::new(), old::Rules::new());
    let mut a = new::Round::deal(
        nr,
        new::Seat::ALL[dealer],
        &mut SmallRng::seed_from_u64(seed),
    );
    let mut b = old::Round::deal(
        or,
        old::Seat::ALL[dealer],
        &mut SmallRng::seed_from_u64(seed),
    );
    loop {
        match (a.phase(), b.phase()) {
            (new::Phase::Finished, old::Phase::Finished) => break,
            (new::Phase::Claim(s), old::Phase::Claim(t)) => {
                assert_eq!(s.index(), t.index());
                let claim = if candidate && s.index() == CANDIDATE {
                    new::HeuristicBot::new(&nr).claim(&new::View::new(&a, s), &a.claim_options(s))
                } else {
                    match old::HeuristicBot::new(&or)
                        .claim(&old::View::new(&b, t), &b.claim_options(t))
                    {
                        old::Claim::Pass => new::Claim::Pass,
                        old::Claim::Pong => new::Claim::Pong,
                        old::Claim::Eat(meld) => new::Claim::Eat(meld),
                    }
                };
                a.claim(s, claim).unwrap();
                b.claim(
                    t,
                    match claim {
                        new::Claim::Pass => old::Claim::Pass,
                        new::Claim::Pong => old::Claim::Pong,
                        new::Claim::Eat(meld) => old::Claim::Eat(meld),
                    },
                )
                .unwrap();
            }
            (new::Phase::Discard(s), old::Phase::Discard(t)) => {
                assert_eq!(s.index(), t.index());
                let card = if candidate && s.index() == CANDIDATE {
                    new::HeuristicBot::new(&nr).discard(&new::View::new(&a, s))
                } else {
                    old::HeuristicBot::new(&or).discard(&old::View::new(&b, t))
                };
                a.discard(s, card).unwrap();
                b.discard(t, card).unwrap();
            }
            phases => panic!("the rounds fell out of step: {phases:?}"),
        }
    }
    let result = a.result().unwrap();
    let mut game = new::Game::new(nr, new::Seat::ALL[dealer]);
    game.settle(result);
    (
        game.scores()[CANDIDATE],
        result.winner.map(new::Seat::index) == Some(CANDIDATE),
    )
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().unwrap());
    let (base, n) = (args.next().unwrap(), args.next().unwrap());
    let threads = std::thread::available_parallelism().unwrap().get() as u64;
    // (sum d, sum d², differing, control wins, candidate wins)
    let total = std::thread::scope(|scope| {
        let shards: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut acc = (0.0f64, 0.0f64, 0u64, 0u64, 0u64);
                    for i in (t..n).step_by(threads as usize) {
                        let dealer = (i % 4) as usize;
                        let (control, cw) = play(base + i, dealer, false);
                        let (treated, tw) = play(base + i, dealer, true);
                        let d = f64::from(treated - control);
                        acc.0 += d;
                        acc.1 += d * d;
                        acc.2 += u64::from(d != 0.0);
                        acc.3 += u64::from(cw);
                        acc.4 += u64::from(tw);
                    }
                    acc
                })
            })
            .collect();
        shards
            .into_iter()
            .map(|h| h.join().unwrap())
            .fold((0.0, 0.0, 0, 0, 0), |a, b| {
                (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3, a.4 + b.4)
            })
    });
    let nf = n as f64;
    let mean = total.0 / nf;
    let se = ((total.1 - nf * mean * mean) / (nf - 1.0) / nf).sqrt();
    let (lo, hi) = (mean - 1.96 * se, mean + 1.96 * se);
    println!("seed base {base}, {n} deals, {} differ", total.2);
    println!(
        "seat 1 wins: control {:.2}%, candidate {:.2}%",
        100.0 * total.3 as f64 / nf,
        100.0 * total.4 as f64 / nf
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
