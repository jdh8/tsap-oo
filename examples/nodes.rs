//! What the single-player DP of `doc/expectation.md` costs a decision: at
//! every discard of `HeuristicBot` self-play, the [`Graph`] from each
//! discard the hand may make, its size, the time to build it and to sweep
//! it over the public cards left.
//!
//! With a pruning of the discards, it also builds the graph that tries
//! them all and reports how often the best root differs and what that
//! costs by the unpruned values.
//!
//! With a depth, the graph stops that many claims from the roots
//! ([`Prune::depth`]); its leaves are swept as worth 0, the time the curves
//! of an eval would take aside.
//!
//! ```sh
//! cargo run --release --example nodes -- [rounds] [seed] [most cards after the discard] [discards 0-3] [depth]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::time::Instant;
use tsap_oo::susik::Hand;
use tsap_oo::{
    Discards, Game, Graph, HeuristicBot, Outlook, Payoff, Phase, Prune, Rules, Seat, Step,
    Strategy, Table, View, discardable,
};

/// The node of the best root and its value
fn best(graph: &Graph, values: &[f64]) -> (usize, f64) {
    graph
        .roots()
        .iter()
        .enumerate()
        .map(|(i, &node)| (i, values[node]))
        .fold((0, f64::NEG_INFINITY), |a, b| if b.1 > a.1 { b } else { a })
}

#[derive(Default)]
struct Row {
    decisions: u64,
    roots: u64,
    nodes: u64,
    edges: u64,
    leaves: u64,
    most_nodes: u64,
    build: f64,
    sweep: f64,
    /// Decisions whose best root the pruning changes
    changed: u64,
    /// What the pruned choice gives up by the unpruned values
    regret: f64,
    most_regret: f64,
    /// Unpruned less pruned value of the best root
    bias: f64,
}

impl Row {
    fn add(&mut self, other: &Self) {
        self.decisions += other.decisions;
        self.roots += other.roots;
        self.nodes += other.nodes;
        self.edges += other.edges;
        self.leaves += other.leaves;
        self.most_nodes = self.most_nodes.max(other.most_nodes);
        self.build += other.build;
        self.sweep += other.sweep;
        self.changed += other.changed;
        self.regret += other.regret;
        self.most_regret = self.most_regret.max(other.most_regret);
        self.bias += other.bias;
    }

    fn print(&self, label: &str, compare: bool) {
        let per = |n: f64| n / self.decisions as f64;
        print!(
            "{label:>5}  {:6}  {:5.1}  {:8.1}  {:9.1}  {:9.1}  {:9}  {:8.2}  {:8.2}",
            self.decisions,
            per(self.roots as f64),
            per(self.nodes as f64),
            per(self.edges as f64),
            per(self.leaves as f64),
            self.most_nodes,
            per(self.build) * 1e3,
            per(self.sweep) * 1e3,
        );
        if compare {
            print!(
                "  {:6.2}%  {:7.4}  {:7.3}  {:7.4}",
                per(self.changed as f64) * 100.0,
                per(self.regret),
                self.most_regret,
                per(self.bias),
            );
        }
        println!();
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let rounds = args.next().flatten().unwrap_or(100);
    let seed = args.next().flatten().unwrap_or(0);
    let most = args.next().flatten().unwrap_or(20) as usize;
    let discards = match args.next().flatten().unwrap_or(0) {
        0 => Discards::All,
        1 => Discards::Loose,
        2 => Discards::Alone,
        _ => Discards::Live,
    };
    let depth = args.next().flatten().map(|d| d as u32);
    let prune = Prune {
        discards,
        depth,
        ..Prune::default()
    };
    let compare = discards != Discards::All && depth.is_none();
    let rules = Rules::new();
    let outlook = Outlook::default();
    let mut game = Game::new(rules, Seat::ALL[0]);
    let mut rng = SmallRng::seed_from_u64(seed);
    // By cards in hand after the discard
    let mut by_size: Vec<Row> = (0..21).map(|_| Row::default()).collect();

    for _ in 0..rounds {
        let round = game.deal(&mut rng);
        let bots =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        let mut table = Table::new(round, bots);
        loop {
            let round = table.round();
            if let Phase::Discard(seat) = round.phase() {
                let view = View::new(round, seat);
                let hand = view.hand();
                if hand.len() <= most + 1 {
                    let base = view
                        .exposed(seat)
                        .iter()
                        .map(|meld| rules.exposed.score(meld))
                        .sum();
                    let roots: Vec<(Hand, u32)> = discardable(hand, view.locked())
                        .distinct()
                        .map(|(card, _)| (hand.without(card), base))
                        .collect();
                    let shown = Seat::ALL
                        .into_iter()
                        .flat_map(|seat| view.exposed(seat))
                        .flat_map(|meld| meld.cards())
                        .chain(view.discards().iter().map(|&(_, card)| card));
                    let live = shown.fold(Hand::FULL - hand, Hand::without);
                    let draws = view.pile_len() - rules.dead();

                    let start = Instant::now();
                    let graph = Graph::new(&rules, live, &roots, prune);
                    let built = start.elapsed().as_secs_f64();
                    let zeros = vec![vec![0.0]; graph.leaves().len()];
                    let values = graph.values_with(&outlook, draws, Payoff::Points, &zeros);
                    let row = &mut by_size[hand.len() - 1];
                    row.decisions += 1;
                    row.roots += roots.len() as u64;
                    row.nodes += graph.len() as u64 - 1;
                    row.edges += graph.edges() as u64;
                    row.leaves += graph.leaves().len() as u64;
                    row.most_nodes = row.most_nodes.max(graph.len() as u64 - 1);
                    row.build += built;
                    row.sweep += start.elapsed().as_secs_f64() - built;

                    if compare {
                        let whole = Graph::new(&rules, live, &roots, Prune::default());
                        let full = whole.values(&outlook, draws, Payoff::Points);
                        let (choice, value) = best(&graph, &values);
                        let (right, top) = best(&whole, &full);
                        let regret = top - full[whole.roots()[choice]];
                        row.changed += u64::from(choice != right && regret > 0.0);
                        row.regret += regret;
                        row.most_regret = row.most_regret.max(regret);
                        row.bias += top - value;
                    }
                }
            }
            let step = table.step().expect("bots act legally");
            let Step::Acted(_) = step else { break };
        }
        let result = table.round().result().expect("finished").clone();
        game.settle(&result);
    }

    println!(
        "{rounds} rounds, seed {seed}, {discards:?} discards, depth {depth:?}: a graph from every discard of a decision"
    );
    print!("\ncards  points  roots  nodes/pt   edges/pt  leaves/pt  max nodes  build ms  sweep ms");
    if compare {
        print!("  changed  regret  max reg  bias");
    }
    println!();
    let mut total = Row::default();
    for (size, row) in by_size.iter().enumerate().rev() {
        if row.decisions == 0 {
            continue;
        }
        total.add(row);
        row.print(&size.to_string(), compare);
    }
    total.print("all", compare);
}
