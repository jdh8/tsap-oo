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
//! Then the grade as a whole is put in points, the way a chess engine puts
//! its evaluation in pawns: the even shards' hands give the mean points of a
//! hand by its grade, made monotone, and the odd shards' choices check that
//! the curve tells what a choice costs.  `research/grade-in-points.md`.
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

/// The curve's bins: one a 胡 of grade, the first from [`LOW`] down and the
/// last from `LOW + BINS - 1` up
const BINS: usize = 72;
const LOW: f64 = -40.0;

/// [`GAPS`] cut finer, for what a choice behind the pick costs
const FINE: [f64; 8] = [0.0, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];

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
struct Tally {
    /// By stage, as [`STAGES`]
    hands: [Sums<13>; 4],
    /// Every choice, and those within [`NEAR`] of the bot's pick
    choices: [Sums<7>; 2],
    /// By gap, as [`GAPS`]: choices, and their points beyond the pick's
    gaps: [[f64; 2]; 5],
    /// By stage and bin of grade: hands, and the points they end with
    curve: [[[f64; 2]; BINS]; 4],
    /// Choices other than the pick: the stage, the pick's grade, the
    /// choice's and its points beyond the pick's
    trials: Vec<(usize, f64, f64, f64)>,
}

/// The stage of `round`, as [`STAGES`]
fn stage(round: &Round) -> usize {
    match round.pile_len() {
        23.. => 0,
        14..=22 => 1,
        _ => 2,
    }
}

/// The 胡 `seat` has exposed
fn shown(round: &Round, seat: Seat) -> u32 {
    let melds = round.exposed(seat).iter();
    melds.map(|meld| round.rules().exposed.score(meld)).sum()
}

/// The bin of [`BINS`] a grade falls in
fn bin(grade: f64) -> usize {
    ((grade - LOW).floor().max(0.0) as usize).min(BINS - 1)
}

/// A curve's `bins` of (hands, points) as (grade, mean points) knots, the
/// means made to rise with the grade by pooling adjacent violators
fn monotone(bins: &[[f64; 2]; BINS]) -> Vec<(f64, f64)> {
    // (hands, points, bins pooled)
    let mut pools: Vec<(f64, f64, usize)> = Vec::new();
    let mut knots = Vec::new();
    for (i, &[n, sum]) in bins.iter().enumerate() {
        if n == 0.0 {
            continue;
        }
        knots.push(LOW + i as f64 + 0.5);
        pools.push((n, sum, 1));
        while let [.., a, b] = pools[..]
            && a.1 / a.0 > b.1 / b.0
        {
            pools.truncate(pools.len() - 2);
            pools.push((a.0 + b.0, a.1 + b.1, a.2 + b.2));
        }
    }
    let means = pools
        .iter()
        .flat_map(|&(n, sum, bins)| std::iter::repeat_n(sum / n, bins));
    knots.into_iter().zip(means).collect()
}

/// The points of `grade` on a curve of `knots`: between two knots a line,
/// beyond the ends flat
fn read(knots: &[(f64, f64)], grade: f64) -> f64 {
    let i = knots.partition_point(|&(at, _)| at < grade);
    match (i.checked_sub(1).map(|i| knots[i]), knots.get(i)) {
        (Some((x0, y0)), Some(&(x1, y1))) => y0 + (y1 - y0) * (grade - x0) / (x1 - x0),
        (Some((_, y)), None) | (None, Some(&(_, y))) => y,
        (None, None) => 0.0,
    }
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
    let (stage, shown) = (stage(round), f64::from(shown(round, seat)));
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
        tally
            .trials
            .push((stage, shown + grade0, shown + grade, dy));
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
    let stage = stage(round);
    let shown = Seat::ALL.map(|seat| shown(round, seat));
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
        // The grade the bot compares claims by: exposed 胡 and all.
        let grade = x[1] + (0..7).map(|i| CHOICE_NOW[i] * x[2 + i]).sum::<f64>() / 32.0;
        for stage in [stage, 3] {
            let cell = &mut tally.curve[stage][bin(grade)];
            cell[0] += 1.0;
            cell[1] += y;
        }
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
                        curve: [[[0.0; 2]; BINS]; 4],
                        trials: Vec::new(),
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
    curve(&shards);
}

/// Print the grade in points: the curve of each stage from the even shards'
/// hands, then how it tells the odd shards' choices
fn curve(shards: &[Tally]) {
    let knots: Vec<Vec<(f64, f64)>> = (0..4)
        .map(|stage| {
            let mut bins = [[0.0; 2]; BINS];
            for shard in shards.iter().step_by(2) {
                for (cell, add) in bins.iter_mut().zip(shard.curve[stage]) {
                    cell[0] += add[0];
                    cell[1] += add[1];
                }
            }
            println!("\ngrade in points, {}", STAGES[stage]);
            println!("grade      hands     mean  monotone");
            let knots = monotone(&bins);
            for &(at, points) in &knots {
                let [n, sum] = bins[bin(at)];
                println!("{:5} {n:10} {:8.3} {points:9.3}", at - 0.5, sum / n);
            }
            knots
        })
        .collect();
    // Per odd shard and gap, the last row every gap: choices, the 胡 behind,
    // the points beyond the pick, then for the curve of the stage and of the
    // whole round what it tells, that times the points, and its square.
    let rows: Vec<[[f64; 9]; 6]> = shards
        .iter()
        .skip(1)
        .step_by(2)
        .map(|shard| {
            let mut rows = [[0.0; 9]; 6];
            for &(stage, pick, grade, dy) in &shard.trials {
                let gap = pick - grade;
                let row = GAPS.iter().position(|&most| gap <= most);
                let told = [stage, 3].map(|s| read(&knots[s], grade) - read(&knots[s], pick));
                for row in [row.unwrap_or(GAPS.len()), 5] {
                    let row = &mut rows[row];
                    row[0] += 1.0;
                    row[1] += gap;
                    row[2] += dy;
                    for (i, told) in told.into_iter().enumerate() {
                        row[3 + 3 * i] += told;
                        row[4 + 3 * i] += told * dy;
                        row[5 + 3 * i] += told * told;
                    }
                }
            }
            rows
        })
        .collect();
    println!("\nwhat the curves tell of the odd shards' choices");
    println!(
        "up to    choices  胡 behind   points      ±  by stage  slope      ±  whole  slope      ±"
    );
    for row in 0..6 {
        let sum = |i: usize| rows.iter().map(|shard| shard[row][i]).sum::<f64>();
        let (points, se) = spread(rows.iter().map(|shard| shard[row][2] / shard[row][0]));
        let most = match GAPS.get(row) {
            Some(most) => most.to_string(),
            None if row == GAPS.len() => "more".into(),
            None => "all".into(),
        };
        print!(
            "{most:6} {:9} {:10.3} {points:8.3} {se:6.3}",
            sum(0),
            sum(1) / sum(0)
        );
        for i in [3, 6] {
            // Least squares through the origin: points on what was told.
            let slope = spread(
                rows.iter()
                    .map(|shard| shard[row][i + 1] / shard[row][i + 2]),
            );
            print!(" {:9.3} {:6.3} {:6.3}", sum(i) / sum(0), slope.0, slope.1);
        }
        println!();
    }
    // Every shard's choices by stage and gap: choices, 胡 behind, points.
    for (stage, name) in STAGES.into_iter().enumerate() {
        println!("\nwhat a choice behind the pick costs, {name}");
        println!("up to    choices  胡 behind   points      ±");
        let rows: Vec<[[f64; 3]; FINE.len() + 1]> = shards
            .iter()
            .map(|shard| {
                let mut rows = [[0.0; 3]; FINE.len() + 1];
                for &(at, pick, grade, dy) in &shard.trials {
                    if stage == 3 || stage == at {
                        let gap = pick - grade;
                        let row = FINE.iter().position(|&most| gap <= most);
                        let row = &mut rows[row.unwrap_or(FINE.len())];
                        row[0] += 1.0;
                        row[1] += gap;
                        row[2] += dy;
                    }
                }
                rows
            })
            .collect();
        for row in 0..=FINE.len() {
            let n: f64 = rows.iter().map(|shard| shard[row][0]).sum();
            let gap: f64 = rows.iter().map(|shard| shard[row][1]).sum();
            let filled = rows.iter().filter(|shard| shard[row][0] > 0.0);
            let (points, se) = spread(filled.map(|shard| shard[row][2] / shard[row][0]));
            let most = FINE.get(row).map_or("more".into(), |most| most.to_string());
            println!("{most:6} {n:9} {:10.3} {points:8.3} {se:6.3}", gap / n);
        }
    }
}
