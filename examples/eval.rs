//! The pattern tables of `doc/eval.md`, batch 3: their labels, their
//! training and what they lose offline.
//!
//! Seat 1 plays the desktop [`ExpectBot`] (gated at 14 cards) among three
//! [`HeuristicBot`]s, the A/B table, and every decision of it at 11 to 20
//! cards is valued by the exact graph, [`Discards::Loose`] from 18 cards.
//! Cards are counted as the gate counts them: in hand when answering an
//! offer, after the discard when discarding.
//!
//! - `label` writes, for each such graph, the curve of every root and of up
//!   to 64 nodes each one and two claims away.
//! - `train` fits tables of each width to the labels by Adam on the squared
//!   error of every count of public cards to come; one round in ten is held
//!   out.
//! - `loss` stops the graph 0, 1 and 2 claims from the roots, reads each
//!   leaf from the tables, and logs what the answer it picks falls short of
//!   the best by the exact values, beside `HeuristicBot`'s first choice, and
//!   the leaves' error against their exact curves.
//!
//! ```sh
//! cargo run --release --example eval -- label [rounds] [seed base] [labels]
//! cargo run --release --example eval -- train [labels] [prefix] [epochs] [dims...]
//! cargo run --release --example eval -- loss [rounds] [seed base] [tables...]
//! ```
//!
//! `PLAYERS` (default 4) is the table's size, `THRESHOLD` (default 10) the
//! 胡 to win, `BASE` the 底 the values are paid with (none unless set), and
//! `EXPECT` (default 1.5) what the h of that table's [`Outlook::new`] is
//! scaled by: tables for a table of two are labelled with `PLAYERS=2
//! EXPECT=1.3`.
use rand::rngs::SmallRng;
use rand::{RngExt as _, SeedableRng as _};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write as _};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use tsap_oo::eval::{self, CARDS, Features, ROWS};
use tsap_oo::expect::END_SCALE;
use tsap_oo::susik::{Card, Hand, Meld};
use tsap_oo::{
    Claim, ClaimOptions, Discards, Eval, ExpectBot, Graph, HeuristicBot, Outlook, Payoff, Prune,
    Round, Rules, Seat, Step, Strategy, Table, View, discardable,
};

/// The gate the desktop plays at
const GATE: usize = 14;

/// Nodes sampled a graph at each depth past the roots
const PER_DEPTH: usize = 64;

/// The roots of each answer, `HeuristicBot`'s order first
type Groups = Vec<Vec<(Hand, u32)>>;

/// What one decision needs: the live cards, the draws left, its card count
/// and its answers
struct Spot {
    live: Hand,
    draws: usize,
    cards: usize,
    groups: Groups,
}

impl Spot {
    fn new(view: &View<'_>, cards: usize, groups: Groups) -> Self {
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card))
            .chain(view.offer().map(|offer| offer.card));
        Self {
            live: shown.fold(Hand::FULL - view.hand(), Hand::without),
            draws: view.pile_len().saturating_sub(view.rules().dead()),
            cards,
            groups,
        }
    }

    fn roots(&self) -> Vec<(Hand, u32)> {
        self.groups.iter().flatten().copied().collect()
    }

    /// The exact graph, as `examples/regret` builds it
    fn exact(&self, rules: &Rules) -> Graph {
        let prune = Prune {
            discards: if self.cards >= 18 {
                Discards::Loose
            } else {
                Discards::All
            },
            ..Prune::default()
        };
        Graph::new(rules, self.live, &self.roots(), prune)
    }

    /// The value of each answer: the best of its roots
    fn by_group(&self, roots: &[usize], values: &[f64]) -> Vec<f64> {
        let mut nodes = roots.iter();
        self.groups
            .iter()
            .map(|group| {
                nodes
                    .by_ref()
                    .take(group.len())
                    .map(|&node| values[node])
                    .fold(f64::NEG_INFINITY, f64::max)
            })
            .collect()
    }
}

/// Seat 1: plays as `ExpectBot`, shows `each` every decision
struct Hook<F> {
    bot: ExpectBot,
    greedy: HeuristicBot,
    each: F,
}

impl<F: FnMut(&View<'_>, Spot)> Strategy for Hook<F> {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let mut claims: Vec<Claim> = self
            .greedy
            .claims(view, options)
            .into_iter()
            .map(|(claim, _)| claim)
            .collect();
        let every = (!options.forced())
            .then_some(Claim::Pass)
            .into_iter()
            .chain(options.pong.then_some(Claim::Pong))
            .chain(options.eats.iter().map(|&meld| Claim::Eat(meld)));
        for claim in every {
            if !claims.contains(&claim) {
                claims.push(claim);
            }
        }
        let (hand, card) = (view.hand(), view.offer().expect("an offer").card);
        let rules = view.rules();
        let base = base(view);
        let groups = claims
            .iter()
            .map(|&claim| {
                let meld = match claim {
                    Claim::Pass => return vec![(hand, base)],
                    Claim::Pong => Meld::Triplet(card),
                    Claim::Eat(meld) => meld,
                };
                let rest = hand - meld.cards().without(card);
                let base = base + rules.exposed.score(&meld);
                discardable(rest, Some(card))
                    .distinct()
                    .map(|(discard, _)| (rest.without(discard), base))
                    .collect()
            })
            .collect();
        (self.each)(view, Spot::new(view, hand.len(), groups));
        self.bot.claim(view, options)
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let hand = view.hand();
        let base = base(view);
        let groups = self
            .greedy
            .discards(view)
            .into_iter()
            .map(|(card, _)| vec![(hand.without(card), base)])
            .collect();
        (self.each)(view, Spot::new(view, hand.len() - 1, groups));
        self.bot.discard(view)
    }
}

/// The 胡 of the seat's exposed melds
fn base(view: &View<'_>) -> u32 {
    let rules = view.rules();
    view.exposed(view.seat())
        .iter()
        .map(|meld| rules.exposed.score(meld))
        .sum()
}

/// Play round `seed` and collect what `each` makes of seat 1's decisions at
/// 11 to 20 cards
fn play<T: 'static>(seed: u64, mut each: impl FnMut(&Rules, Spot) -> T + 'static) -> Vec<T> {
    let rules = rules();
    let mut rng = SmallRng::seed_from_u64(seed);
    let dealer = Seat::ALL[(seed % rules.players as u64) as usize];
    let round = Round::deal(rules, dealer, &mut rng);
    let log = Rc::<RefCell<Vec<T>>>::default();
    let sink = Rc::clone(&log);
    let mut seats =
        Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
    seats[1] = Some(Box::new(Hook {
        bot: ExpectBot::new(&rules)
            .with_outlook(outlook(&rules))
            .gated(GATE),
        greedy: HeuristicBot::new(&rules),
        each: move |view: &View<'_>, spot: Spot| {
            if (11..=20).contains(&spot.cards) {
                sink.borrow_mut().push(each(view.rules(), spot));
            }
        },
    }));
    let mut table = Table::new(round, seats);
    assert_eq!(table.run(), Ok(Step::Finished));
    drop(table);
    log.take()
}

/// `f` on rounds `base..base + rounds` over every core, in round order
fn spread<T: Send>(rounds: u64, base: u64, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()) as u64;
    let f = &f;
    let mut out: Vec<(u64, T)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    (t..rounds)
                        .step_by(threads as usize)
                        .map(|i| (i, f(base + i)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a round panicked"))
            .collect()
    });
    out.sort_by_key(|x| x.0);
    out.into_iter().map(|x| x.1).collect()
}

/// The table: [`Rules::new`] with `PLAYERS` seats, `THRESHOLD` to win and
/// the 底 `BASE`
fn rules() -> Rules {
    let mut rules = Rules::new();
    if let Ok(threshold) = std::env::var("THRESHOLD") {
        rules.threshold = threshold.parse().expect("THRESHOLD is a number");
    }
    if let Ok(players) = std::env::var("PLAYERS") {
        rules.players = players.parse().expect("PLAYERS is a number");
    }
    if let Ok(base) = std::env::var("BASE") {
        rules.base = Some(base.parse().expect("BASE is a number"));
    }
    rules
}

/// The outlook seat 1 plays by and every graph is valued by: the table's,
/// its h scaled by `EXPECT` (default [`END_SCALE`])
fn outlook(rules: &Rules) -> Outlook {
    let scale =
        std::env::var("EXPECT").map_or(END_SCALE, |s| s.parse().expect("EXPECT is a number"));
    let mut outlook = Outlook::new(rules);
    outlook.end.iter_mut().for_each(|h| *h *= scale);
    outlook
}

/// One labelled node: its round, the decision's cards, its depth and
/// cards, its rows and its values by public cards to come from 1
struct Label {
    round: u32,
    decision: u8,
    depth: u8,
    cards: u8,
    features: Features,
    values: Vec<f32>,
}

impl Label {
    fn write(&self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        out.write_all(&self.round.to_le_bytes())?;
        out.write_all(&[
            self.decision,
            self.depth,
            self.cards,
            self.values.len() as u8,
        ])?;
        for f in self.features {
            out.write_all(&f.to_le_bytes())?;
        }
        for v in &self.values {
            out.write_all(&v.to_le_bytes())?;
        }
        Ok(())
    }

    fn read(input: &mut impl std::io::Read) -> Option<Self> {
        let mut head = [0; 8 + 22];
        input.read_exact(&mut head).ok()?;
        let features =
            std::array::from_fn(|i| u16::from_le_bytes([head[8 + 2 * i], head[9 + 2 * i]]));
        let mut values = vec![0; 4 * usize::from(head[7])];
        input.read_exact(&mut values).ok()?;
        Some(Self {
            round: u32::from_le_bytes([head[0], head[1], head[2], head[3]]),
            decision: head[4],
            depth: head[5],
            cards: head[6],
            features,
            values: values
                .as_chunks::<4>()
                .0
                .iter()
                .map(|&b| f32::from_le_bytes(b))
                .collect(),
        })
    }
}

fn label(rounds: u64, base: u64, path: &str) {
    let start = Instant::now();
    let labels: Vec<Vec<Vec<Label>>> = spread(rounds, base, |seed| {
        let outlook = outlook(&rules());
        let mut rng = SmallRng::seed_from_u64(!seed);
        let round = (seed - base) as u32;
        play(seed, move |rules, spot| {
            let graph = spot.exact(rules);
            let mut nodes: Vec<usize> = graph.roots().iter().copied().filter(|&n| n > 0).collect();
            nodes.sort_unstable();
            nodes.dedup();
            for depth in 1..=2 {
                let mut at: Vec<usize> = (1..graph.len())
                    .filter(|&n| graph.depth(n) == depth)
                    .collect();
                let k = at.len().min(PER_DEPTH);
                for i in 0..k {
                    let j = rng.random_range(i..at.len());
                    at.swap(i, j);
                }
                nodes.extend_from_slice(&at[..k]);
            }
            let curves = graph.curves(&outlook, spot.draws, Payoff::Points, &nodes);
            nodes
                .iter()
                .zip(curves)
                .map(|(&node, curve)| {
                    let (hand, base) = graph.state(node);
                    Label {
                        round,
                        decision: spot.cards as u8,
                        depth: graph.depth(node) as u8,
                        cards: hand.len() as u8,
                        features: eval::features(hand, base, spot.live),
                        values: curve[1..curve.len().min(CARDS + 1)]
                            .iter()
                            .map(|&v| v as f32)
                            .collect(),
                    }
                })
                .collect()
        })
    });
    let mut out = BufWriter::new(std::fs::File::create(path).expect("the labels file"));
    let (mut count, mut values, mut graphs) = (0, 0, 0);
    for label in labels.iter().flatten().flatten() {
        label.write(&mut out).expect("written");
        count += 1;
        values += label.values.len();
    }
    for round in &labels {
        graphs += round.len();
    }
    out.flush().expect("flushed");
    println!(
        "seed base {base}, {rounds} rounds, {graphs} graphs, {count} nodes, {values} values, {:.0} s",
        start.elapsed().as_secs_f64()
    );
}

/// Adam's state
struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    step: i32,
}

impl Adam {
    fn update(&mut self, params: &mut [f32], grads: &[f32], lr: f32) {
        const B1: f32 = 0.9;
        const B2: f32 = 0.999;
        self.step += 1;
        let c1 = 1.0 - B1.powi(self.step);
        let c2 = 1.0 - B2.powi(self.step);
        for (((p, &g), m), v) in params
            .iter_mut()
            .zip(grads)
            .zip(&mut self.m)
            .zip(&mut self.v)
        {
            *m = B1 * *m + (1.0 - B1) * g;
            *v = B2 * *v + (1.0 - B2) * g * g;
            *p -= lr * (*m / c1) / ((*v / c2).sqrt() + 1e-8);
        }
    }
}

/// Squared error and count over `labels`, by depth
fn errors(eval: &Eval, labels: &[&Label]) -> ([f64; 3], [f64; 3], [usize; 3]) {
    let (mut sq, mut bias, mut n) = ([0.0; 3], [0.0; 3], [0; 3]);
    for label in labels {
        let curve = eval.forward(&label.features);
        let d = usize::from(label.depth).min(2);
        for (t, &y) in label.values.iter().enumerate() {
            let e = curve[t + 1] - f64::from(y);
            sq[d] += e * e;
            bias[d] += e;
            n[d] += 1;
        }
    }
    (sq, bias, n)
}

fn train(path: &str, prefix: &str, epochs: usize, dims: &[usize]) {
    let mut input = BufReader::new(std::fs::File::open(path).expect("the labels file"));
    let labels: Vec<Label> = std::iter::from_fn(|| Label::read(&mut input)).collect();
    let (held, fit): (Vec<&Label>, Vec<&Label>) = labels.iter().partition(|l| l.round % 10 == 9);
    let values: usize = fit.iter().map(|l| l.values.len()).sum();
    println!(
        "{} nodes, {} held out; {values} values to fit",
        fit.len(),
        held.len()
    );
    // The mean by public cards to come starts the biases
    let mut mean = [(0.0, 0); CARDS];
    for label in &fit {
        for (t, &y) in label.values.iter().enumerate() {
            mean[t].0 += f64::from(y);
            mean[t].1 += 1;
        }
    }
    let (mut sq, mut n) = (0.0, 0);
    for label in &held {
        for (t, &y) in label.values.iter().enumerate() {
            sq += (f64::from(y) - mean[t].0 / f64::from(mean[t].1.max(1) as u32)).powi(2);
            n += 1;
        }
    }
    println!(
        "held-out RMSE of the mean by public cards to come: {:.3}",
        (sq / f64::from(n.max(1) as u32)).sqrt()
    );

    const LR: f32 = 3e-3;
    const BATCH: usize = 256;
    for &d in dims {
        let mut rng = SmallRng::seed_from_u64(1_795_800_000 + d as u64);
        let mut params = vec![0.0_f32; Eval::len(d)];
        let spread = (3.0 / 11.0_f32).sqrt();
        for p in &mut params[..ROWS * d] {
            *p = spread * (2.0 * rng.random::<f32>() - 1.0);
        }
        for p in &mut params[ROWS * d..(ROWS + CARDS) * d] {
            *p = 0.1 * (2.0 * rng.random::<f32>() - 1.0);
        }
        for (b, &(sum, n)) in params[(ROWS + CARDS) * d..].iter_mut().zip(&mean) {
            *b = if n > 0 {
                (sum / f64::from(n as u32)) as f32
            } else {
                0.0
            };
        }
        let mut adam = Adam {
            m: vec![0.0; params.len()],
            v: vec![0.0; params.len()],
            step: 0,
        };
        let mut order: Vec<usize> = (0..fit.len()).collect();
        let mut grads = vec![0.0_f32; params.len()];
        let (mut acc, mut hidden, mut dh) = (vec![0.0_f32; d], vec![0.0_f32; d], vec![0.0_f32; d]);
        let start = Instant::now();
        println!(
            "\nD = {d}\n\n| epoch | lr | train RMSE | held-out RMSE | s |\n| ---: | ---: | ---: | ---: | ---: |"
        );
        for epoch in 0..epochs {
            // Linear decay to a tenth
            let lr = LR * (1.0 - 0.9 * epoch as f32 / epochs.max(2) as f32);
            for i in (1..order.len()).rev() {
                order.swap(i, rng.random_range(0..=i));
            }
            let (mut sq, mut count) = (0.0_f64, 0_usize);
            for batch in order.chunks(BATCH) {
                grads.fill(0.0);
                let n: usize = batch.iter().map(|&i| fit[i].values.len()).sum();
                let scale = 2.0 / n as f32;
                for &i in batch {
                    let label = fit[i];
                    acc.fill(0.0);
                    for &row in &label.features {
                        let row = &params[usize::from(row) * d..][..d];
                        acc.iter_mut().zip(row).for_each(|(a, x)| *a += x);
                    }
                    for (h, &a) in hidden.iter_mut().zip(&acc) {
                        *h = a.max(0.0);
                    }
                    dh.fill(0.0);
                    for (t, &y) in label.values.iter().enumerate() {
                        let w = (ROWS + t) * d;
                        let b = (ROWS + CARDS) * d + t;
                        let pred: f32 = params[w..w + d]
                            .iter()
                            .zip(&hidden)
                            .map(|(w, h)| w * h)
                            .sum::<f32>()
                            + params[b];
                        let e = pred - y;
                        sq += f64::from(e * e);
                        let g = scale * e;
                        grads[b] += g;
                        for j in 0..d {
                            grads[w + j] += g * hidden[j];
                            dh[j] += g * params[w + j];
                        }
                    }
                    count += label.values.len();
                    for j in 0..d {
                        if acc[j] <= 0.0 {
                            dh[j] = 0.0;
                        }
                    }
                    for &row in &label.features {
                        let row = &mut grads[usize::from(row) * d..][..d];
                        row.iter_mut().zip(&dh).for_each(|(g, x)| *g += x);
                    }
                }
                adam.update(&mut params, &grads, lr);
            }
            let eval = Eval::new(d, params.clone()).expect("the layout");
            let (hsq, _, hn) = errors(&eval, &held);
            println!(
                "| {} | {lr:.5} | {:.3} | {:.3} | {:.0} |",
                epoch + 1,
                (sq / count as f64).sqrt(),
                (hsq.iter().sum::<f64>() / hn.iter().sum::<usize>() as f64).sqrt(),
                start.elapsed().as_secs_f64()
            );
        }
        let eval = Eval::new(d, params).expect("the layout");
        let (sq, bias, n) = errors(&eval, &held);
        println!("\n| depth | values | RMSE | bias |\n| ---: | ---: | ---: | ---: |");
        for k in 0..3 {
            println!(
                "| {k} | {} | {:.3} | {:+.3} |",
                n[k],
                (sq[k] / n[k] as f64).sqrt(),
                bias[k] / n[k] as f64
            );
        }
        let out = format!("{prefix}-{d}.bin");
        std::fs::write(&out, eval.to_bytes()).expect("the tables written");
        println!("\nwrote {out}");
    }
}

/// Bins of public cards to come
const BINS: [(usize, usize); 5] = [(1, 5), (6, 10), (11, 20), (21, 30), (31, CARDS)];

fn bin(t: usize) -> usize {
    BINS.iter()
        .position(|&(lo, hi)| (lo..=hi).contains(&t))
        .unwrap_or(BINS.len() - 1)
}

/// The leaves' error against their exact curves: the sums of a regression
/// of V̂ on V, and the mean error by (leaf cards, bin)
#[derive(Clone)]
struct Fit {
    n: f64,
    x: f64,
    y: f64,
    xx: f64,
    xy: f64,
    yy: f64,
    /// (count, sum) by leaf cards and bin
    cells: Vec<[(f64, f64); BINS.len()]>,
}

impl Default for Fit {
    fn default() -> Self {
        Self {
            n: 0.0,
            x: 0.0,
            y: 0.0,
            xx: 0.0,
            xy: 0.0,
            yy: 0.0,
            cells: vec![[(0.0, 0.0); BINS.len()]; 21],
        }
    }
}

impl Fit {
    fn add(&mut self, cards: usize, t: usize, v: f64, vhat: f64) {
        self.n += 1.0;
        self.x += v;
        self.y += vhat;
        self.xx += v * v;
        self.xy += v * vhat;
        self.yy += vhat * vhat;
        let cell = &mut self.cells[cards][bin(t)];
        cell.0 += 1.0;
        cell.1 += vhat - v;
    }

    fn merge(&mut self, other: &Self) {
        self.n += other.n;
        self.x += other.x;
        self.y += other.y;
        self.xx += other.xx;
        self.xy += other.xy;
        self.yy += other.yy;
        for (mine, theirs) in self.cells.iter_mut().zip(&other.cells) {
            for (a, b) in mine.iter_mut().zip(theirs) {
                a.0 += b.0;
                a.1 += b.1;
            }
        }
    }

    /// Mean error, slope, residual SD, RMSE
    fn summary(&self) -> (f64, f64, f64, f64) {
        let n = self.n;
        let (mx, my) = (self.x / n, self.y / n);
        let sxx = self.xx / n - mx * mx;
        let sxy = self.xy / n - mx * my;
        let syy = self.yy / n - my * my;
        let slope = sxy / sxx;
        let resid = (syy - slope * sxy).max(0.0).sqrt();
        let mse = (self.yy - 2.0 * self.xy + self.xx) / n;
        (my - mx, slope, resid, mse.sqrt())
    }

    /// The largest mean error of a cell with at least `least` values
    fn worst(&self, least: f64) -> f64 {
        self.cells
            .iter()
            .flatten()
            .filter(|c| c.0 >= least)
            .map(|c| c.1 / c.0)
            .fold(0.0, |a: f64, b| if b.abs() > a.abs() { b } else { a })
    }
}

const DEPTHS: usize = 3;

struct Loss {
    cards: usize,
    /// What `HeuristicBot`'s first choice gives up
    regret: f64,
    /// By table and depth, what the shallow graph's choice gives up
    shallow: Vec<[f64; DEPTHS]>,
    fits: Vec<[Fit; DEPTHS]>,
    leaves: usize,
    nanos: u128,
}

fn loss(rounds: u64, base: u64, paths: &[String]) {
    let tables: Arc<Vec<Eval>> = Arc::new(
        paths
            .iter()
            .map(|p| Eval::from_bytes(&std::fs::read(p).expect("a table")).expect("tables"))
            .collect(),
    );
    let start = Instant::now();
    let played: Vec<Vec<Loss>> = spread(rounds, base, |seed| {
        let outlook = outlook(&rules());
        let tables = Arc::clone(&tables);
        play(seed, move |rules, spot| {
            let whole = spot.exact(rules);
            let index: HashMap<_, _> = (1..whole.len()).map(|n| (whole.state(n), n)).collect();
            let roots = spot.roots();
            let graphs: Vec<Graph> = (0..DEPTHS)
                .map(|k| {
                    let prune = Prune {
                        depth: Some(k as u32),
                        ..Prune::default()
                    };
                    Graph::new(rules, spot.live, &roots, prune)
                })
                .collect();
            // The leaves the exact graph also holds, by depth: (leaf, node)
            let known: Vec<Vec<(usize, usize)>> = graphs
                .iter()
                .map(|graph| {
                    graph
                        .leaves()
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &leaf)| index.get(&graph.state(leaf)).map(|&n| (i, n)))
                        .collect()
                })
                .collect();
            // One sweep of the exact graph: its roots, then every known leaf
            let mut nodes: Vec<usize> = whole.roots().to_vec();
            nodes.extend(known.iter().flatten().map(|x| x.1));
            let truths = whole.curves(&outlook, spot.draws, Payoff::Points, &nodes);
            let exact: HashMap<usize, f64> = nodes
                .iter()
                .zip(&truths)
                .map(|(&n, curve)| (n, curve[curve.len() - 1]))
                .collect();
            let root_values: Vec<f64> = (0..whole.len())
                .map(|n| exact.get(&n).copied().unwrap_or(0.0))
                .collect();
            let truth = spot.by_group(whole.roots(), &root_values);
            let top = truth.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let mut out = Loss {
                cards: spot.cards,
                regret: top - truth[0],
                shallow: vec![[0.0; DEPTHS]; tables.len()],
                fits: vec![Default::default(); tables.len()],
                leaves: 0,
                nanos: 0,
            };
            let mut offset = whole.roots().len();
            for (k, (graph, known)) in graphs.iter().zip(&known).enumerate() {
                let leaves = graph.leaves();
                let truths = &truths[offset..offset + known.len()];
                offset += known.len();
                for (e, table) in tables.iter().enumerate() {
                    let clock = Instant::now();
                    let curves: Vec<Vec<f64>> = leaves
                        .iter()
                        .map(|&leaf| {
                            let (hand, base) = graph.state(leaf);
                            table.curve(hand, base, spot.live)
                        })
                        .collect();
                    out.nanos += clock.elapsed().as_nanos();
                    out.leaves += leaves.len();
                    let values = graph.values_with(&outlook, spot.draws, Payoff::Points, &curves);
                    let picked = spot.by_group(graph.roots(), &values);
                    let chosen =
                        (0..picked.len()).fold(0, |a, b| if picked[b] > picked[a] { b } else { a });
                    out.shallow[e][k] = top - truth[chosen];
                    let fit = &mut out.fits[e][k];
                    for (&(i, _), truth) in known.iter().zip(truths) {
                        let cards = graph.state(leaves[i]).0.len();
                        for (t, &v) in truth.iter().enumerate().skip(1) {
                            fit.add(cards, t, v, curves[i][t.min(CARDS)]);
                        }
                    }
                }
            }
            out
        })
    });
    let all: Vec<&Loss> = played.iter().flatten().collect();
    let (leaves, nanos): (usize, u128) = (
        all.iter().map(|x| x.leaves).sum(),
        all.iter().map(|x| x.nanos).sum(),
    );
    println!(
        "seed base {base}, {rounds} rounds, {} decisions, {:.0} s; {:.2} µs a leaf",
        all.len(),
        start.elapsed().as_secs_f64(),
        nanos as f64 / 1000.0 / leaves as f64
    );

    let r = rounds as f64;
    print!("\n| cards | decisions | N | R |");
    for path in paths {
        for k in 0..DEPTHS {
            print!(" {path} k = {k} |");
        }
    }
    println!(
        "\n| ---: | ---: | ---: | ---: |{}",
        " ---: |".repeat(paths.len() * DEPTHS)
    );
    for cards in (11..=20).rev() {
        let rows: Vec<&&Loss> = all.iter().filter(|x| x.cards == cards).collect();
        let n = rows.len() as f64;
        print!(
            "| {cards} | {} | {:.3} | {:.4} |",
            rows.len(),
            n / r,
            rows.iter().map(|x| x.regret).sum::<f64>() / n
        );
        for e in 0..paths.len() {
            for k in 0..DEPTHS {
                print!(
                    " {:.4} |",
                    rows.iter().map(|x| x.shallow[e][k]).sum::<f64>() / n
                );
            }
        }
        println!();
    }

    // Per round, as examples/regret: the band's Σ N · R
    let band = |f: &dyn Fn(&Loss) -> f64| -> Vec<f64> {
        played
            .iter()
            .map(|round| round.iter().map(f).sum())
            .collect()
    };
    let low = band(&|x| if x.cards <= 14 { x.regret } else { 0.0 });
    let (lo, lo_se) = mean_se(&low);
    let (gain, gain_se) = (0.132, 0.014);
    println!(
        "\nΣ N · R at 11 to 14 cards: {lo:.4} ± {lo_se:.4}, worth {gain} ± {gain_se} (application D)"
    );
    println!(
        "\n| table | platform | Σ N · (R − R̂) | SE | predicted gain | SE |\n| --- | --- | ---: | ---: | ---: | ---: |"
    );
    for (e, path) in paths.iter().enumerate() {
        let web = band(&|x| x.regret - x.shallow[e][1]);
        let desktop = band(&|x| match x.cards {
            15..=17 => x.regret - x.shallow[e][2],
            18.. => x.regret - x.shallow[e][1],
            _ => 0.0,
        });
        for (platform, high) in [
            ("web, k = 1 at 11 to 20", web),
            ("desktop, k = 2 at 15 to 17, 1 at 18 to 20", desktop),
        ] {
            let (hi, hi_se) = mean_se(&high);
            let cov = low
                .iter()
                .zip(&high)
                .map(|(x, y)| (x - lo) * (y - hi))
                .sum::<f64>()
                / (r - 1.0)
                / r;
            let predicted = gain * hi / lo;
            let rel = (gain_se / gain).powi(2) + (hi_se / hi).powi(2) + (lo_se / lo).powi(2)
                - 2.0 * cov / (hi * lo);
            println!(
                "| {path} | {platform} | {hi:.4} | {hi_se:.4} | {predicted:.3} | {:.3} |",
                predicted.abs() * rel.max(0.0).sqrt()
            );
        }
    }

    println!(
        "\n| table | k | values | V̂ − V | slope | residual SD | RMSE | worst cell |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    let mut merged = vec![<[Fit; DEPTHS]>::default(); paths.len()];
    for x in &all {
        for (m, f) in merged.iter_mut().zip(&x.fits) {
            for k in 0..DEPTHS {
                m[k].merge(&f[k]);
            }
        }
    }
    for (path, fits) in paths.iter().zip(&merged) {
        for (k, fit) in fits.iter().enumerate() {
            let (bias, slope, resid, rmse) = fit.summary();
            println!(
                "| {path} | {k} | {:.0} | {bias:+.3} | {slope:.3} | {resid:.3} | {rmse:.3} | {:+.3} |",
                fit.n,
                fit.worst(1000.0)
            );
        }
    }
    for (path, fits) in paths.iter().zip(&merged) {
        for (k, fit) in fits.iter().enumerate() {
            print!("\n{path}, k = {k}: V̂ − V by leaf cards and public cards to come\n\n| cards |");
            for (lo, hi) in BINS {
                print!(" {lo} to {hi} |");
            }
            println!("\n| ---: |{}", " ---: |".repeat(BINS.len()));
            for cards in (0..=20).rev() {
                if fit.cells[cards].iter().all(|c| c.0 == 0.0) {
                    continue;
                }
                print!("| {cards} |");
                for &(n, s) in &fit.cells[cards] {
                    match n {
                        0.0 => print!(" |"),
                        n if n >= 1000.0 => print!(" {:+.2} |", s / n),
                        n => print!(" ({:+.2}) |", s / n),
                    }
                }
                println!();
            }
        }
    }
}

/// Mean and standard error of `xs`
fn mean_se(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let mean = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, (var / n).sqrt())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let num = |i: usize, default: u64| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(default);
    match args.first().map(String::as_str) {
        Some("label") => label(
            num(1, 100),
            num(2, 0),
            args.get(3).map_or("labels.bin", String::as_str),
        ),
        Some("train") => {
            let dims: Vec<usize> = args[4..].iter().filter_map(|s| s.parse().ok()).collect();
            train(
                args.get(1).map_or("labels.bin", String::as_str),
                args.get(2).map_or("eval", String::as_str),
                num(3, 20) as usize,
                if dims.is_empty() { &[1, 4, 16] } else { &dims },
            );
        }
        Some("loss") => loss(num(1, 100), num(2, 0), &args[3..]),
        _ => eprintln!("usage: eval label|train|loss ..."),
    }
}
