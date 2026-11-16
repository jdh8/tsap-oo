//! What each term of [`HeuristicBot`]'s grade is worth in points, fitted by
//! least squares to rounds four bots play.  The numbers behind
//! `research/fitted-weights.md`.
//!
//! Two fits, since what a term is worth across hands need not be what it is
//! worth between the choices of one hand:
//!
//! - **Hands.**  At every discard each seat's hand is laid out as the bot
//!   grades it and set against the points the seat ends the round with, by
//!   stage of the round.
//! - **Choices.**  At every [`EVERY`]th discard each card the seat could
//!   shed is tried and the round played out from there, the same pile for
//!   all.  The fit is of what a card scores beyond the bot's own pick on how
//!   their terms differ, so whatever the choices share drops out.
//!
//! ```sh
//! cargo run --release --example fit -- [rounds] [seed base]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use tsap_oo::susik::Hand;
use tsap_oo::{Game, HeuristicBot, Phase, Round, Rules, Seat, Strategy as _, Terms, View};

/// Stages of a round, by the cards left in the pile, and all of them
const STAGES: [&str; 4] = [
    "early (23+)",
    "mid (14-22)",
    "late (-13)",
    "the whole round",
];

/// The regressors of a hand: a constant, the 胡 already exposed, the bot's
/// terms, and what the bot does not grade by but a seat's prospects hang
/// on: the cards left in the pile, the cards the seat has not seen, the
/// fewest concealed cards another seat holds and the 胡 the other seats
/// have exposed
const HAND: [&str; 13] = [
    "(constant)",
    "exposed 胡",
    "hu",
    "loose",
    "short",
    "own",
    "kept",
    "lonely",
    "waits",
    "pile",
    "unseen",
    "their fewest",
    "their 胡",
];

/// What the bot weighs each of [`HAND`] at, in 32nds of a 胡: `SCALE`,
/// `WEIGHT`, `OWN` and `WAIT` of `src/heuristic.rs`
const HAND_NOW: [f64; 13] = [
    0.0, 32.0, 32.0, -128.0, -64.0, 2.0, 1.0, -1.0, 8.0, 0.0, 0.0, 0.0, 0.0,
];

/// The regressors of a choice: the bot's terms
const CHOICE: [&str; 7] = ["hu", "loose", "short", "own", "kept", "lonely", "waits"];
const CHOICE_NOW: [f64; 7] = [32.0, -128.0, -64.0, 2.0, 1.0, -1.0, 8.0];

/// One discard in this many has its choices played out
const EVERY: u64 = 8;

/// The most 胡 of grade a choice may fall behind the bot's pick and still
/// count as near it
const NEAR: f64 = 2.0;

/// Choices by the 胡 of grade they fall behind the bot's pick: up to each
/// of these, then beyond
const GAPS: [f64; 4] = [0.0, 0.5, 2.0, 8.0];

/// The sums a least squares fit needs
#[derive(Clone, Copy)]
struct Sums<const K: usize> {
    n: f64,
    xx: [[f64; K]; K],
    xy: [f64; K],
}

impl<const K: usize> Sums<K> {
    const NEW: Self = Self {
        n: 0.0,
        xx: [[0.0; K]; K],
        xy: [0.0; K],
    };

    fn add(&mut self, x: &[f64; K], y: f64) {
        self.n += 1.0;
        for i in 0..K {
            self.xy[i] += x[i] * y;
            for j in 0..K {
                self.xx[i][j] += x[i] * x[j];
            }
        }
    }

    /// The coefficients: Gaussian elimination on the normal equations
    fn fit(&self) -> [f64; K] {
        let (mut a, mut b) = (self.xx, self.xy);
        for col in 0..K {
            let pivot = (col..K)
                .max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))
                .expect("a column");
            a.swap(col, pivot);
            b.swap(col, pivot);
            let pivot = a[col];
            for row in 0..K {
                if row != col {
                    let factor = a[row][col] / pivot[col];
                    for (cell, above) in a[row].iter_mut().zip(pivot) {
                        *cell -= factor * above;
                    }
                    b[row] -= factor * b[col];
                }
            }
        }
        std::array::from_fn(|i| b[i] / a[i][i])
    }
}

/// What one shard of rounds adds up to
#[derive(Clone, Copy)]
struct Tally {
    /// By stage, as [`STAGES`]
    hands: [Sums<13>; 4],
    /// Every choice, and those within [`NEAR`] of the bot's pick
    choices: [Sums<7>; 2],
    /// By gap, as [`GAPS`]: choices, and their points beyond the pick's
    gaps: [[f64; 2]; 5],
}

fn terms(t: Terms) -> [f64; 7] {
    [t.hu, t.loose, t.short, t.own, t.kept, t.lonely, t.waits].map(f64::from)
}

/// Let the bot make the pending decision; whether there was one
fn step(bot: &mut HeuristicBot, round: &mut Round) -> bool {
    match round.phase() {
        Phase::Finished => return false,
        Phase::Claim(seat) => {
            let claim = bot.claim(&View::new(round, seat), &round.claim_options(seat));
            round.claim(seat, claim).expect("the bot claims legally");
        }
        Phase::Discard(seat) => {
            let card = bot.discard(&View::new(round, seat));
            round.discard(seat, card).expect("the bot discards legally");
        }
    }
    true
}

/// The points each seat ends `round` with
fn points(round: &Round) -> [i32; Seat::COUNT] {
    let mut game = Game::new(*round.rules(), round.dealer());
    game.settle(round.result().expect("the round is over"));
    game.scores()
}

/// Try every card `seat` could discard and play the round out after each
fn choices(bot: &mut HeuristicBot, round: &Round, seat: Seat, tally: &mut Tally) {
    let view = View::new(round, seat);
    let mut pick = None;
    for (card, grade) in bot.discards(&view) {
        // A dead hand has no terms.
        let Some(t) = bot.terms(&view, round.hand(seat).without(card)) else {
            continue;
        };
        let mut after = round.clone();
        after.discard(seat, card).expect("the card is discardable");
        while step(bot, &mut after) {}
        let (x, y) = (terms(t), f64::from(points(&after)[seat.index()]));
        // The first is the bot's own pick.
        let Some((grade0, x0, y0)) = pick else {
            pick = Some((grade, x, y));
            continue;
        };
        let (gap, dx, dy): (f64, [f64; 7], f64) = (
            grade0 - grade,
            std::array::from_fn(|i| x[i] - x0[i]),
            y - y0,
        );
        tally.choices[0].add(&dx, dy);
        if gap <= NEAR {
            tally.choices[1].add(&dx, dy);
        }
        let row = GAPS.iter().position(|&most| gap <= most);
        let row = &mut tally.gaps[row.unwrap_or(GAPS.len())];
        row[0] += 1.0;
        row[1] += dy;
    }
}

/// Add every seat's hand at this discard, `card` gone from `seat`'s, each
/// against the `points` the seat ends the round with
fn hands(
    bot: &mut HeuristicBot,
    round: &Round,
    (seat, card): (Seat, tsap_oo::susik::Card),
    seen: &mut Vec<(usize, Seat, [f64; 13])>,
) {
    let rules = round.rules();
    let stage = match round.pile_len() {
        23.. => 0,
        14..=22 => 1,
        _ => 2,
    };
    let shown = Seat::ALL.map(|seat| {
        let melds = round.exposed(seat).iter();
        melds.map(|meld| rules.exposed.score(meld)).sum::<u32>()
    });
    // The discarder's own card is not out yet.
    let out: usize = Seat::ALL
        .iter()
        .flat_map(|&seat| round.exposed(seat))
        .map(|meld| meld.cards().len())
        .sum::<usize>()
        + round.discards().len()
        + usize::from(round.offer().is_some());
    for other in Seat::ALL {
        let hand = round.hand(other);
        let hand = if other == seat {
            hand.without(card)
        } else {
            hand
        };
        // A dead hand has no terms; the bot never keeps one.
        let Some(t) = bot.terms(&View::new(round, other), hand) else {
            continue;
        };
        let rivals = [1, 2, 3].map(|n| other.advance(n));
        let fewest = rivals.iter().map(|&seat| round.hand(seat).len()).min();
        let theirs: u32 = rivals.iter().map(|&seat| shown[seat.index()]).sum();
        let unseen = Hand::FULL.len() - round.hand(other).len() - out;
        let mut x = [0.0; 13];
        x[0] = 1.0;
        x[1] = f64::from(shown[other.index()]);
        x[2..9].copy_from_slice(&terms(t));
        x[9] = round.pile_len() as f64;
        x[10] = unseen as f64;
        x[11] = fewest.expect("three rivals") as f64;
        x[12] = f64::from(theirs);
        seen.push((stage, other, x));
    }
}

/// Play round `seed`
fn play(seed: u64, tally: &mut Tally) {
    let rules = Rules::new();
    let dealer = Seat::ALL[(seed % 4) as usize];
    let mut round = Round::deal(rules, dealer, &mut SmallRng::seed_from_u64(seed));
    let mut bot = HeuristicBot::new(&rules);
    let mut seen = Vec::new();
    let mut discards = seed;
    loop {
        if let Phase::Discard(seat) = round.phase() {
            let card = bot.discard(&View::new(&round, seat));
            hands(&mut bot, &round, (seat, card), &mut seen);
            if discards.is_multiple_of(EVERY) {
                choices(&mut bot, &round, seat, tally);
            }
            discards += 1;
            round.discard(seat, card).expect("the bot discards legally");
        } else if !step(&mut bot, &mut round) {
            break;
        }
    }
    let points = points(&round);
    for (stage, seat, x) in seen {
        let y = f64::from(points[seat.index()]);
        tally.hands[stage].add(&x, y);
        tally.hands[3].add(&x, y);
    }
}

/// The mean of the shards' `values` and its standard error
///
/// Rounds are independent and each lies in one shard, so the spread of the
/// shards gives the standard error of their mean.
fn spread(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let values: Vec<f64> = values.collect();
    let m = values.len() as f64;
    let mean = values.iter().sum::<f64>() / m;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (m - 1.0);
    (mean, (var / m).sqrt())
}

/// Print the fit of every shard's `sums` together, its `hu`th regressor
/// also scaled to the 32 the bot weighs a 胡 at
fn table<const K: usize>(names: [&str; K], now: [f64; K], hu: usize, sums: &[Sums<K>]) {
    let mut all = Sums::<K>::NEW;
    for shard in sums {
        all.n += shard.n;
        for i in 0..K {
            all.xy[i] += shard.xy[i];
            for j in 0..K {
                all.xx[i][j] += shard.xx[i][j];
            }
        }
    }
    let fit = all.fit();
    let unit = 32.0 / fit[hu];
    println!("{} rows", all.n);
    println!("term            now    points      ±   per hu×32      ±");
    for i in 0..K {
        let (_, se) = spread(sums.iter().map(|shard| shard.fit()[i]));
        println!(
            "{:12} {:6} {:9.4} {:6.4} {:11.2} {:6.2}",
            names[i],
            now[i],
            fit[i],
            se,
            unit * fit[i],
            unit.abs() * se,
        );
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(|arg| arg.parse::<u64>());
    let rounds = args.next().map_or(20_000, |n| n.expect("a round count"));
    let base = args.next().map_or(20_261_119, |n| n.expect("a seed base"));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let shards: Vec<Tally> = std::thread::scope(|scope| {
        let shards: Vec<_> = (0..threads as u64)
            .map(|shard| {
                scope.spawn(move || {
                    let mut tally = Tally {
                        hands: [Sums::NEW; 4],
                        choices: [Sums::NEW; 2],
                        gaps: [[0.0; 2]; 5],
                    };
                    for i in (shard..rounds).step_by(threads) {
                        play(base + i, &mut tally);
                    }
                    tally
                })
            })
            .collect();
        shards
            .into_iter()
            .map(|shard| shard.join().expect("a shard panicked"))
            .collect()
    });
    println!("{rounds} rounds, seed base {base}, {threads} shards");
    for (stage, name) in STAGES.into_iter().enumerate() {
        print!("\nhands, {name}: ");
        let sums: Vec<_> = shards.iter().map(|shard| shard.hands[stage]).collect();
        table(HAND, HAND_NOW, 2, &sums);
    }
    for (near, name) in ["all", "near the pick"].into_iter().enumerate() {
        print!("\nchoices, {name}: ");
        let sums: Vec<_> = shards.iter().map(|shard| shard.choices[near]).collect();
        table(CHOICE, CHOICE_NOW, 0, &sums);
    }
    println!("\nchoices by the 胡 of grade behind the pick");
    println!("up to    choices  points beyond the pick      ±");
    for row in 0..5 {
        let n: f64 = shards.iter().map(|shard| shard.gaps[row][0]).sum();
        let (mean, se) = spread(
            shards
                .iter()
                .map(|shard| shard.gaps[row][1] / shard.gaps[row][0]),
        );
        let most = GAPS.get(row).map_or("more".into(), |most| most.to_string());
        println!("{most:6} {n:9} {mean:23.3} {se:6.3}");
    }
}
