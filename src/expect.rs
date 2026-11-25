//! What a hand is worth in points, by a single-player dynamic programme.
//!
//! Nothing drawn enters a hand, so from one seat a round is a run of public
//! cards, each of which it wins on, claims and discards after, or lets go.
//! A state is (concealed hand, 胡 of the exposed melds); the exposed count
//! follows from the hand's.  The other seats are four constants, an
//! [`Outlook`].  The design and its approximations are in
//! `doc/expectation.md`.
//!
//! [`Graph::new`] lays out every state a chain of claims reaches from the
//! roots, once a decision; [`Graph::values`] then sweeps it once per public
//! card to come.

use crate::ceiling::reach;
use crate::{
    Claim, ClaimOptions, Eval, Giveaway, HeuristicBot, Rules, Seat, Strategy, View, discardable,
};
use std::collections::HashMap;
use susik::{Card, Hand, Meld, Rank, Solver, locked_completions};

/// The table as one seat models it
///
/// The two tables run by draws left, from 0: the public cards after the
/// last draw.  Each card with `k` draws left belongs to segment `k`: the
/// discards before the draw that leaves `k - 1`, then that draw.  A table's
/// last entry holds for more draws.
#[derive(Debug, Clone, PartialEq)]
pub struct Outlook {
    /// The chance another seat ends the round before a public card, by
    /// draws left
    pub end: Vec<f64>,
    /// How many public cards a segment holds, by draws left
    pub cards: Vec<f64>,
    /// What the seat scores then: minus what it pays the winner
    pub loss: f64,
    /// The chance a public card other than a general comes to the seat to
    /// eat: its own draw, or its 上家's that the 上家 lets go, or the
    /// 上家's discard, with no pong from another seat
    pub eat: f64,
    /// The chance a general is the seat's own draw, which it must eat
    pub general: f64,
    /// How often a card drawn is of each rank, the general first, over the
    /// rank's share of the unseen cards, by draws left: a segment's one
    /// draw is a card at its unseen count times its rank's rate, over the
    /// sum of those
    pub drawn: Vec<[f64; 7]>,
    /// The same of a card another seat discards: the rest of a segment
    pub discarded: Vec<[f64; 7]>,
    /// The share of the discards that are the seat's own, which bring it
    /// nothing, by draws left.  `eat` counts them among its cards, so a
    /// graph takes `eat` over the share of a segment that is not the
    /// seat's own.
    pub own: Vec<f64>,
}

impl Outlook {
    /// The table measured at `rules`' table size, or at four under its
    /// threshold, or [`Outlook::default`]
    ///
    /// A table of three is measured at 10 胡 only, a table of two at 10
    /// and at 8 胡, all with the 25 draws a round [`Rules::dead`] leaves
    /// it.
    #[must_use]
    pub fn new(rules: &Rules) -> Self {
        match (rules.players, rules.threshold) {
            (2, 8) => Self {
                end: END_2_8.to_vec(),
                cards: CARDS_2_8.to_vec(),
                loss: -11.32,
                eat: 0.7269,
                general: 0.4796,
                ..Self::even()
            },
            (2, _) => Self {
                end: END_2.to_vec(),
                cards: CARDS_2.to_vec(),
                loss: -12.11,
                eat: 0.7442,
                general: 0.4688,
                ..Self::even()
            },
            (3, _) => Self {
                end: END_3.to_vec(),
                cards: CARDS_3.to_vec(),
                loss: -12.18,
                eat: 0.4164,
                general: 0.3094,
                ..Self::even()
            },
            (_, 8) => Self {
                end: END_8.to_vec(),
                cards: CARDS_8.to_vec(),
                loss: -11.30,
                eat: 0.2599,
                general: 0.2088,
                ..Self::even()
            },
            _ => Self::default(),
        }
    }

    /// No table, every public card drawn evenly from the unseen ones, the
    /// seat's own discards among them
    fn even() -> Self {
        Self {
            end: Vec::new(),
            cards: Vec::new(),
            loss: 0.0,
            eat: 0.0,
            general: 0.0,
            drawn: vec![[1.0; 7]],
            discarded: vec![[1.0; 7]],
            own: vec![0.0],
        }
    }

    /// The segment of each public card to come, the last card first:
    /// segments `0..=draws` less the card being decided on
    fn segments(&self, draws: usize) -> Vec<usize> {
        let total: f64 = (0..=draws).map(|k| at(&self.cards, k)).sum();
        let (mut k, mut upto) = (0, at(&self.cards, 0));
        (1..=(total - 1.0).round().max(0.0) as usize)
            .map(|t| {
                while t as f64 > upto && k < draws {
                    k += 1;
                    upto += at(&self.cards, k);
                }
                k
            })
            .collect()
    }
}

/// `table` at segment `k`, its last entry holding for more
fn at<T: Copy>(table: &[T], k: usize) -> T {
    table[k.min(table.len() - 1)]
}

impl Default for Outlook {
    /// Measured from `HeuristicBot` self-play while the seat holds 10
    /// concealed cards or fewer: `research/outlook.md`
    fn default() -> Self {
        Self {
            end: END.to_vec(),
            cards: CARDS.to_vec(),
            loss: -12.26,
            eat: 0.2765,
            general: 0.2156,
            drawn: by_phase(DRAWN),
            discarded: by_phase(DISCARDED),
            own: vec![0.0],
        }
    }
}

/// A table by draws left of one by the three phases the rates are measured
/// in: 15 draws left or more, 8 to 14, 7 or fewer
fn by_phase<T: Copy>(rates: [T; 3]) -> Vec<T> {
    (0..=15)
        .map(|k| rates[usize::from(k < 15) + usize::from(k < 8)])
        .collect()
}

/// h by draws left, 0 to 25: `outlook 10000 20`
const END: [f64; 26] = [
    0.1126, 0.1126, 0.1217, 0.1164, 0.1099, 0.1121, 0.0983, 0.1021, 0.0966, 0.0821, 0.0851, 0.0824,
    0.0775, 0.0723, 0.0666, 0.0671, 0.0582, 0.0524, 0.0513, 0.0409, 0.0441, 0.0375, 0.0269, 0.0202,
    0.0175, 0.0175,
];

/// Public cards a segment by draws left, 0 to 25: `outlook 10000 20`
const CARDS: [f64; 26] = [
    0.2791, 1.2594, 1.3112, 1.4403, 1.3689, 1.4043, 1.4298, 1.4611, 1.4954, 1.5066, 1.5715, 1.5844,
    1.6264, 1.6332, 1.7161, 1.7365, 1.8207, 1.9039, 1.9455, 2.0456, 2.1579, 2.2592, 2.3920, 2.5734,
    2.7310, 3.2345,
];

/// The rates of the cards drawn, by phase, while the seat holds 10
/// concealed cards or fewer; 士象 share one and 車馬炮 another, as
/// [`Eval`] takes them for alike: `doc/generals.md`, batch 2b
const DRAWN: [[f64; 7]; 3] = [
    [0.7590, 1.1157, 1.1157, 1.0397, 1.0397, 1.0397, 1.0092],
    [0.6838, 1.1184, 1.1184, 1.0861, 1.0861, 1.0861, 1.0464],
    [0.6363, 1.1323, 1.1323, 1.1205, 1.1205, 1.1205, 1.1254],
];

/// The rates of the cards the other seats discard, as [`DRAWN`]
const DISCARDED: [[f64; 7]; 3] = [
    [0.0000, 1.2499, 1.2499, 1.5267, 1.5267, 1.5267, 0.3812],
    [0.0000, 1.2468, 1.2468, 1.5825, 1.5825, 1.5825, 0.4557],
    [0.0000, 1.3731, 1.3731, 1.5968, 1.5968, 1.5968, 0.5000],
];

/// [`END`] at 8 胡: `THRESHOLD=8 outlook 10000 800`
const END_8: [f64; 26] = [
    0.1734, 0.1734, 0.1429, 0.1479, 0.1221, 0.1462, 0.1350, 0.1432, 0.1146, 0.1238, 0.1087, 0.1058,
    0.1069, 0.0888, 0.0899, 0.0912, 0.0726, 0.0721, 0.0702, 0.0568, 0.0519, 0.0493, 0.0432, 0.0324,
    0.0350, 0.0350,
];

/// [`CARDS`] at 8 胡: `THRESHOLD=8 outlook 10000 800`
const CARDS_8: [f64; 26] = [
    0.3168, 1.2466, 1.2917, 1.3016, 1.4740, 1.4203, 1.3701, 1.4603, 1.4460, 1.4533, 1.5207, 1.5631,
    1.5912, 1.6754, 1.6991, 1.7635, 1.8283, 1.9323, 1.9720, 2.0716, 2.1837, 2.3270, 2.4861, 2.6536,
    2.8891, 3.4176,
];

/// [`END`] at a table of two: `PLAYERS=2 outlook 10000 210`
const END_2: [f64; 26] = [
    0.0686, 0.0686, 0.0768, 0.0750, 0.0760, 0.0708, 0.0727, 0.0730, 0.0694, 0.0619, 0.0692, 0.0595,
    0.0610, 0.0593, 0.0599, 0.0522, 0.0510, 0.0438, 0.0445, 0.0395, 0.0399, 0.0366, 0.0302, 0.0325,
    0.0325, 0.0325,
];

/// [`CARDS`] at a table of two: `PLAYERS=2 outlook 10000 210`
const CARDS_2: [f64; 26] = [
    0.1044, 1.1260, 1.1395, 1.1434, 1.1617, 1.1500, 1.1740, 1.1620, 1.1928, 1.2160, 1.2223, 1.2512,
    1.2974, 1.3227, 1.3466, 1.3733, 1.4301, 1.4667, 1.5107, 1.6046, 1.6747, 1.7673, 1.9017, 2.0284,
    2.2150, 2.7830,
];

/// [`END`] at a table of two at 8 胡: `PLAYERS=2 THRESHOLD=8 outlook 10000 280`
const END_2_8: [f64; 26] = [
    0.0804, 0.0804, 0.0804, 0.0804, 0.0804, 0.1043, 0.1009, 0.0967, 0.0789, 0.0847, 0.0882, 0.0893,
    0.0822, 0.0858, 0.0721, 0.0705, 0.0656, 0.0636, 0.0638, 0.0572, 0.0483, 0.0474, 0.0449, 0.0453,
    0.0453, 0.0453,
];

/// [`CARDS`] at a table of two at 8 胡: `PLAYERS=2 THRESHOLD=8 outlook 10000 280`
const CARDS_2_8: [f64; 26] = [
    0.0556, 1.0671, 1.0885, 1.1045, 1.1475, 1.1223, 1.1085, 1.1314, 1.1505, 1.1949, 1.1907, 1.2229,
    1.2569, 1.2801, 1.3031, 1.3327, 1.3943, 1.4493, 1.5240, 1.5986, 1.6644, 1.8038, 1.9186, 2.0679,
    2.2780, 2.8782,
];

/// [`END`] at a table of three: `PLAYERS=3 outlook 10000 310`
const END_3: [f64; 26] = [
    0.1049, 0.1049, 0.0987, 0.0916, 0.1269, 0.1146, 0.1066, 0.0837, 0.0964, 0.0926, 0.0832, 0.0784,
    0.0782, 0.0759, 0.0648, 0.0639, 0.0603, 0.0631, 0.0473, 0.0490, 0.0397, 0.0299, 0.0315, 0.0293,
    0.0293, 0.0293,
];

/// [`CARDS`] at a table of three: `PLAYERS=3 outlook 10000 310`
const CARDS_3: [f64; 26] = [
    0.2966, 1.2457, 1.2570, 1.2313, 1.2431, 1.2598, 1.2937, 1.3465, 1.3432, 1.3812, 1.3916, 1.4084,
    1.4598, 1.5051, 1.5379, 1.5845, 1.6402, 1.6984, 1.7598, 1.8495, 1.9354, 2.0223, 2.1552, 2.2948,
    2.4743, 2.9898,
];

/// What a round is worth to the seat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Payoff {
    /// Points as [`Game::settle`](crate::Game::settle) pays them
    Points,
    /// 1 for a win and 0 otherwise: the chance to win
    Win,
}

/// The discards a node tries after a claim
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Discards {
    /// Every discardable card
    #[default]
    All,
    /// Only the cards loose in one of the two layouts `HeuristicBot`
    /// grades, when there are any
    Loose,
    /// Of those, only the ones no other loose card goes with, when there
    /// are any
    Alone,
    /// As [`Alone`](Self::Alone), but two loose cards go together only
    /// while a card that finishes them is live
    Live,
}

/// What [`Graph::new`] leaves out
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prune {
    /// A node whose [`ceiling`](crate::ceiling()) with this many generals
    /// from outside falls short of the threshold is not expanded; `None`
    /// expands every node
    pub generals: Option<usize>,
    /// The discards each node tries
    pub discards: Discards,
    /// A node this many claims from the nearest root is a leaf, not
    /// expanded: [`Graph::values_with`] reads its value from a curve.
    /// `None` expands every node.
    pub depth: Option<u32>,
}

impl Default for Prune {
    fn default() -> Self {
        Self {
            generals: Some(1),
            discards: Discards::All,
            depth: None,
        }
    }
}

/// How a node answers one public card
#[derive(Debug, Clone, Copy)]
enum Kind {
    /// Wins at this 胡
    Win(u32),
    /// Kongs, then discards
    Kong,
    /// A general: eaten if the seat drew it, else let go
    General,
    /// May pass; the first group is a pong if `pong`, the rest are eats
    Claim { pong: bool },
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    /// How many of this card are unseen
    count: f64,
    /// Its rank, which [`Outlook::drawn`] and
    /// [`discarded`](Outlook::discarded) weigh it by
    rank: Rank,
    kind: Kind,
    /// Groups `first..last`: the claims, each a list of the nodes its
    /// discards lead to
    first: u32,
    last: u32,
}

/// The hands a chain of claims reaches from the roots of one decision, and
/// the edges between them
///
/// Node 0 stands for every hand that can no longer win: the ones the
/// ceiling prunes.  It has no edges.
#[derive(Debug, Clone)]
pub struct Graph {
    threshold: u32,
    /// The seats that pay a win and are paid a foul
    others: f64,
    /// The unseen cards of each rank
    unseen: [f64; 7],
    roots: Vec<usize>,
    /// The hand of each node
    hands: Vec<Hand>,
    /// The exposed 胡 of each node
    bases: Vec<u32>,
    /// The nodes [`Prune::depth`] leaves unexpanded, in order
    leaves: Vec<usize>,
    /// The fewest claims from a root to each node
    depths: Vec<u32>,
    /// Each node's entries end here in `entries`
    ends: Vec<u32>,
    entries: Vec<Entry>,
    /// Each group's children end here in `children`
    groups: Vec<u32>,
    children: Vec<u32>,
}

impl Graph {
    /// Every state reachable from `roots`, each a settled concealed hand
    /// and its exposed 胡, while `live` cards are yet to be seen
    ///
    /// `live` is fixed for the whole graph, so a card may be claimed more
    /// often than it remains.
    #[must_use]
    pub fn new(rules: &Rules, live: Hand, roots: &[(Hand, u32)], prune: Prune) -> Self {
        let mut build = Build {
            rules,
            live,
            prune,
            win: Solver::new(rules.concealed, None).locked(),
            layouts: [1, 16].map(|cost| Solver::new(rules.concealed, Some(cost)).locked()),
            index: HashMap::new(),
            depth: 0,
            graph: Self {
                threshold: rules.threshold,
                others: (rules.players - 1) as f64,
                unseen: live.distinct().fold([0.0; 7], |mut unseen, (card, n)| {
                    unseen[card.rank as usize] += f64::from(n);
                    unseen
                }),
                roots: Vec::new(),
                hands: vec![Hand::EMPTY],
                bases: vec![0],
                leaves: Vec::new(),
                depths: vec![0],
                ends: Vec::new(),
                entries: Vec::new(),
                groups: Vec::new(),
                children: Vec::new(),
            },
        };
        build.graph.roots = roots
            .iter()
            .map(|&(hand, base)| build.node(hand, base) as usize)
            .collect();
        build.graph.ends.push(0);
        let mut node = 1;
        // Breadth first, so a node's depth is its fewest claims from a root
        while node < build.graph.hands.len() {
            if prune
                .depth
                .is_some_and(|depth| build.graph.depths[node] >= depth)
            {
                build.graph.leaves.push(node);
                build.graph.ends.push(build.graph.entries.len() as u32);
            } else {
                build.depth = build.graph.depths[node] + 1;
                build.expand(node);
            }
            node += 1;
        }
        build.graph
    }

    /// The node of each root, in order; 0 for a root that cannot win
    #[must_use]
    pub fn roots(&self) -> &[usize] {
        &self.roots
    }

    /// How many nodes, the one that cannot win included
    #[must_use]
    pub fn len(&self) -> usize {
        self.hands.len()
    }

    /// Whether the graph is only the node that cannot win
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hands.len() == 1
    }

    /// How many edges: a claim followed by a discard
    #[must_use]
    pub fn edges(&self) -> usize {
        self.children.len()
    }

    /// The nodes [`Prune::depth`] left unexpanded, in order
    #[must_use]
    pub fn leaves(&self) -> &[usize] {
        &self.leaves
    }

    /// The fewest claims from a root to `node`, 0 for the node that cannot
    /// win
    #[must_use]
    pub fn depth(&self, node: usize) -> u32 {
        self.depths[node]
    }

    /// The concealed hand and exposed 胡 of `node`
    #[must_use]
    pub fn state(&self, node: usize) -> (Hand, u32) {
        (self.hands[node], self.bases[node])
    }

    /// The value of every node, under `payoff`, with `draws` draws left
    /// after the public card being decided on, or the discard
    ///
    /// # Panics
    ///
    /// If the graph has [`leaves`](Self::leaves): see
    /// [`values_with`](Self::values_with).
    #[must_use]
    pub fn values(&self, outlook: &Outlook, draws: usize, payoff: Payoff) -> Vec<f64> {
        self.sweep(outlook, draws, payoff, &[], |_| ())
    }

    /// As [`values`](Self::values), each leaf worth its curve: `leaves[i]`
    /// is the value of leaf `i` by public cards to come, from 0, its last
    /// entry holding for more
    ///
    /// # Panics
    ///
    /// If `leaves` is not one curve a leaf.
    #[must_use]
    pub fn values_with(
        &self,
        outlook: &Outlook,
        draws: usize,
        payoff: Payoff,
        leaves: &[Vec<f64>],
    ) -> Vec<f64> {
        self.sweep(outlook, draws, payoff, leaves, |_| ())
    }

    /// The value of each of `nodes` by public cards to come, from 0 to
    /// every card [`values`](Self::values) sweeps
    ///
    /// # Panics
    ///
    /// As [`values`](Self::values).
    #[must_use]
    pub fn curves(
        &self,
        outlook: &Outlook,
        draws: usize,
        payoff: Payoff,
        nodes: &[usize],
    ) -> Vec<Vec<f64>> {
        let mut curves = vec![vec![0.0]; nodes.len()];
        self.sweep(outlook, draws, payoff, &[], |values| {
            for (curve, &node) in curves.iter_mut().zip(nodes) {
                curve.push(values[node]);
            }
        });
        curves
    }

    /// The sweep, with `each` shown the values after every public card
    fn sweep(
        &self,
        outlook: &Outlook,
        draws: usize,
        payoff: Payoff,
        leaves: &[Vec<f64>],
        mut each: impl FnMut(&[f64]),
    ) -> Vec<f64> {
        assert_eq!(leaves.len(), self.leaves.len(), "one curve a leaf");
        let (loss, foul) = match payoff {
            Payoff::Points => (outlook.loss, -self.others * f64::from(self.threshold)),
            Payoff::Win => (0.0, 0.0),
        };
        let mut before = vec![0.0; self.len()];
        let mut after = before.clone();
        for (step, k) in outlook.segments(draws).into_iter().enumerate() {
            let end = at(&outlook.end, k);
            // One card of a segment is its draw, but of the last, which has
            // none; the rest are discards, the seat's own among them.
            let draw = if k == 0 {
                0.0
            } else {
                at(&outlook.cards, k).recip().min(1.0)
            };
            let discard = (1.0 - draw) * (1.0 - at(&outlook.own, k));
            let (drawn, discarded) = (at(&outlook.drawn, k), at(&outlook.discarded, k));
            // `share` of the cards, split among the unseen by `rates`
            let split = |share: f64, rates: [f64; 7]| {
                let sum: f64 = self.unseen.iter().zip(rates).map(|(n, r)| n * r).sum();
                if sum > 0.0 { share / sum } else { 0.0 }
            };
            let (d, s) = (split(draw, drawn), split(discard, discarded));
            let weights: [f64; 7] = std::array::from_fn(|r| d * drawn[r] + s * discarded[r]);
            let eat = (outlook.eat / (draw + discard)).min(1.0);
            let best = |group: u32| {
                let first = if group == 0 {
                    0
                } else {
                    self.groups[group as usize - 1]
                };
                self.children[first as usize..self.groups[group as usize] as usize]
                    .iter()
                    .map(|&child| before[child as usize])
                    .fold(f64::NEG_INFINITY, f64::max)
            };
            let forced = |entry: &Entry| {
                (entry.first..entry.last)
                    .map(best)
                    .fold(f64::NEG_INFINITY, f64::max)
                    .max(if entry.first == entry.last {
                        foul
                    } else {
                        f64::NEG_INFINITY
                    })
            };
            for (node, value) in after.iter_mut().enumerate() {
                let stay = before[node];
                let first = if node == 0 { 0 } else { self.ends[node - 1] };
                let (mut sum, mut rest) = (0.0, 1.0);
                for entry in &self.entries[first as usize..self.ends[node] as usize] {
                    let answer = match entry.kind {
                        Kind::Win(hu) => match payoff {
                            Payoff::Points => self.others * f64::from(hu),
                            Payoff::Win => 1.0,
                        },
                        Kind::Kong => forced(entry),
                        Kind::General => {
                            outlook.general * forced(entry) + (1.0 - outlook.general) * stay
                        }
                        Kind::Claim { pong } => {
                            let mut eats = entry.first;
                            let mut pass = stay;
                            if pong {
                                pass = pass.max(best(eats));
                                eats += 1;
                            }
                            let eaten = (eats..entry.last).map(best).fold(pass, f64::max);
                            eat * eaten + (1.0 - eat) * pass
                        }
                    };
                    let weight = entry.count * weights[entry.rank as usize];
                    rest -= weight;
                    sum += weight * answer;
                }
                sum += stay * rest;
                *value = end * loss + (1.0 - end) * sum;
            }
            for (&leaf, curve) in self.leaves.iter().zip(leaves) {
                after[leaf] = curve[(step + 1).min(curve.len() - 1)];
            }
            std::mem::swap(&mut before, &mut after);
            each(&before);
        }
        before
    }
}

/// What [`ExpectBot::new`] scales the measured h by
// ponytail: the calibration's factor (`research/outlook.md`).  It and 1 both
// beat `HeuristicBot` by the same margin within error (doc/expectation.md,
// batch 3); this one scored higher.
pub const END_SCALE: f64 = 1.5;

/// A bot that plays by [`Graph`] while its hand is short and as
/// [`HeuristicBot`] before, or by a shallow graph read with [`Eval`]
///
/// Inside the gate every answer is worth the best value, in points, of the
/// hands it can leave after a discard; equals keep `HeuristicBot`'s order.
#[derive(Debug, Clone)]
pub struct ExpectBot {
    policy: HeuristicBot,
    outlook: Outlook,
    /// The most concealed cards the bot plays by the DP with, once it has
    /// discarded
    cards: usize,
    /// Past the gate: the tables that value a leaf, and the most cards a
    /// graph two claims deep is built for, one deep beyond
    shallow: Option<(Eval, usize)>,
    /// Unseen cards taken for gone: see [`gone_mut`](Self::gone_mut)
    gone: Hand,
    /// Whether a discard is worth less by its [`Giveaway::cost`]
    wary: bool,
}

impl ExpectBot {
    /// A bot for `rules` gated at 10 cards, with [`Outlook::default`] at
    /// four players and 10 胡, and elsewhere with its h scaled by
    /// [`END_SCALE`] and its cards drawn evenly; two players play by
    /// [`Outlook::new`], its h scaled by 1.3, which [`Eval::of`]'s tables
    /// for them are learnt with
    #[must_use]
    pub fn new(rules: &Rules) -> Self {
        // ponytail: the four-player 10-胡 tables at four players' 8 胡 and
        // at a table of three too.  `Outlook::new`'s measured insufficient
        // data against them at four and, with tables of their own, at three
        // (`doc/eval.md`, "Tables for two and three players").
        let (mut outlook, scale) = match (rules.players, rules.threshold) {
            (2, 8 | 10) => (Outlook::new(rules), 1.3),
            // The ranks' rates are measured at four players and 10 胡
            // only, and with them the measured h calibrates unscaled
            // (`doc/generals.md`, batch 2b).
            (Seat::COUNT, 10) => (Outlook::default(), 1.0),
            _ => {
                let even = Outlook {
                    drawn: vec![[1.0; 7]],
                    discarded: vec![[1.0; 7]],
                    ..Outlook::default()
                };
                (even, END_SCALE)
            }
        };
        outlook.end.iter_mut().for_each(|h| *h *= scale);
        Self {
            policy: HeuristicBot::new(rules),
            outlook,
            cards: 10,
            shallow: None,
            gone: Hand::EMPTY,
            wary: false,
        }
    }

    /// The table model, to read or to change between decisions
    #[must_use]
    pub const fn outlook_mut(&mut self) -> &mut Outlook {
        &mut self.outlook
    }

    /// Unseen cards the bot takes for gone, never to come up: none unless
    /// set here, before each decision
    ///
    /// For the oracles in `ab/`, which read them off the other hands
    /// (`doc/opponents.md`); hidden and behind the `peek` feature like
    /// [`View::peek`].
    #[cfg(feature = "peek")]
    #[doc(hidden)]
    #[must_use]
    pub const fn gone_mut(&mut self) -> &mut Hand {
        &mut self.gone
    }

    /// The same bot with another table model
    #[must_use]
    pub fn with_outlook(mut self, outlook: Outlook) -> Self {
        self.outlook = outlook;
        self
    }

    /// The same bot playing by the DP with `cards` concealed cards or fewer
    /// once it has discarded, as [`MonteCarloBot::gated`](crate::MonteCarloBot)
    #[must_use]
    pub const fn gated(mut self, cards: usize) -> Self {
        self.cards = cards;
        self
    }

    /// The same bot playing past its gate by a graph two claims deep with
    /// `deep` concealed cards or fewer once it has discarded, and one claim
    /// deep beyond, each leaf valued by `eval` (`doc/eval.md`); as
    /// `HeuristicBot` still while more public cards are to come than
    /// [`eval::CARDS`](crate::eval::CARDS) or more draws than
    /// [`eval::DRAWS`](crate::eval::DRAWS): never under [`Rules::new`],
    /// whose every table starts within both
    #[must_use]
    pub fn shallow(mut self, eval: Eval, deep: usize) -> Self {
        self.shallow = Some((eval, deep));
        self
    }

    /// The same bot discarding by each card's value less its
    /// [`Giveaway::cost`], what the public cards say it gives the next
    /// seat, wherever it values discards in points; its claims are as
    /// before
    ///
    /// The costs are measured at a table of two at 10 胡
    /// (`doc/opponents.md`, batch 3).
    // ponytail: a claim is not charged for the discard it forces.  The
    // measured h already holds the average of what discards give away, so
    // only a choice among discards is free of counting it twice.
    #[must_use]
    pub const fn wary(mut self) -> Self {
        self.wary = true;
        self
    }

    /// The draws left at `view`'s table
    fn draws(view: &View<'_>) -> usize {
        view.pile_len().saturating_sub(view.rules().dead())
    }

    /// How deep a graph is built with `cards` concealed cards once
    /// discarded: `Some(None)` whole, `None` none, as `HeuristicBot` plays
    fn depth(&self, view: &View<'_>, cards: usize) -> Option<Option<u32>> {
        if cards <= self.cards {
            return Some(None);
        }
        let (_, deep) = self.shallow.as_ref()?;
        // A leaf's curve stands still past its end while its parents' clock
        // runs on, so every claim looks like a loss; and the table was
        // learnt from rounds of 25 draws, which start within it.  An
        // outlook may count fewer cards a draw, so the draws are held too:
        // with more left, as a table of two had before `Rules::dead`, the
        // leaves lost 5.8 points a split (`research/outlook.md`, "Tables of
        // two and three").
        let draws = Self::draws(view);
        if draws > crate::eval::DRAWS || self.outlook.segments(draws).len() > crate::eval::CARDS {
            return None;
        }
        Some(Some(if cards <= *deep { 2 } else { 1 }))
    }

    /// Whether the bot values its answers in points at `view`'s table with
    /// `cards` concealed cards once discarded, rather than by
    /// `HeuristicBot`'s grades
    #[must_use]
    pub fn searches(&self, view: &View<'_>, cards: usize) -> bool {
        self.depth(view, cards).is_some()
    }

    /// The cards `view`'s seat has not seen, less the ones taken for gone,
    /// and the 胡 of its exposed melds
    fn table(&self, view: &View<'_>) -> (Hand, u32) {
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card))
            .chain(view.offer().map(|offer| offer.card));
        let live = shown.fold(Hand::FULL - view.hand(), Hand::without) - self.gone;
        let rules = view.rules();
        let base = view
            .exposed(view.seat())
            .iter()
            .map(|meld| rules.exposed.score(meld))
            .sum();
        (live, base)
    }

    /// The value of each group of roots: the best of its nodes
    // ponytail: a drawn offer counts one public card fewer than is left;
    // `Graph::values` takes draws, not cards.
    fn values(
        &self,
        view: &View<'_>,
        live: Hand,
        groups: &[Vec<(Hand, u32)>],
        depth: Option<u32>,
    ) -> Vec<f64> {
        let roots: Vec<_> = groups.iter().flatten().copied().collect();
        let prune = Prune {
            depth,
            ..Prune::default()
        };
        let graph = Graph::new(view.rules(), live, &roots, prune);
        let draws = Self::draws(view);
        let curves: Vec<_> = graph
            .leaves()
            .iter()
            .map(|&leaf| {
                let (hand, base) = graph.state(leaf);
                let (eval, _) = self.shallow.as_ref().expect("leaves only past the gate");
                eval.curve(hand, base, live)
            })
            .collect();
        let values = graph.values_with(&self.outlook, draws, Payoff::Points, &curves);
        let mut nodes = graph.roots().iter();
        groups
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

    /// The answers to the offer, best first, each with its value in points;
    /// outside the gate, unless [`shallow`](Self::shallow), `HeuristicBot`'s,
    /// with its grades
    pub fn claims(&mut self, view: &View<'_>, options: &ClaimOptions) -> Vec<(Claim, f64)> {
        let graded = self.policy.claims(view, options);
        let hand = view.hand();
        let Some(depth) = self.depth(view, hand.len()) else {
            return graded
                .into_iter()
                .map(|(claim, grade)| (claim, f64::from(grade)))
                .collect();
        };
        let card = view.offer().expect("claims answer an offer").card;
        // `HeuristicBot` leaves some eats out; the DP weighs them all.
        let mut claims: Vec<Claim> = graded.into_iter().map(|(claim, _)| claim).collect();
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
        let (live, base) = self.table(view);
        let exposed = view.rules().exposed;
        let groups: Vec<Vec<(Hand, u32)>> = claims
            .iter()
            .map(|&claim| {
                let meld = match claim {
                    Claim::Pass => return vec![(hand, base)],
                    Claim::Pong => Meld::Triplet(card),
                    Claim::Eat(meld) => meld,
                };
                let rest = hand - meld.cards().without(card);
                let base = base + exposed.score(&meld);
                discardable(rest, Some(card))
                    .distinct()
                    .map(|(discard, _)| (rest.without(discard), base))
                    .collect()
            })
            .collect();
        rank(claims, self.values(view, live, &groups, depth))
    }

    /// The legal discards, best first, each with its value in points;
    /// outside the gate, unless [`shallow`](Self::shallow), `HeuristicBot`'s,
    /// with its grades
    pub fn discards(&mut self, view: &View<'_>) -> Vec<(Card, f64)> {
        let graded = self.policy.discards(view);
        let hand = view.hand();
        let Some(depth) = self.depth(view, hand.len() - 1) else {
            return graded
                .into_iter()
                .map(|(card, grade)| (card, f64::from(grade)))
                .collect();
        };
        let (live, base) = self.table(view);
        let cards: Vec<Card> = graded.into_iter().map(|(card, _)| card).collect();
        let groups: Vec<_> = cards
            .iter()
            .map(|&card| vec![(hand.without(card), base)])
            .collect();
        rank(cards, self.values(view, live, &groups, depth))
    }
}

/// `actions` with their values, best first; equals keep their order
fn rank<A>(actions: Vec<A>, values: Vec<f64>) -> Vec<(A, f64)> {
    let mut ranked: Vec<_> = actions.into_iter().zip(values).collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked
}

impl Strategy for ExpectBot {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        self.claims(view, options)[0].0
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let ranked = self.discards(view);
        if !self.wary || !self.searches(view, view.hand().len() - 1) {
            return ranked[0].0;
        }
        let giveaway = Giveaway::new(view);
        // The first of equals, as `rank` leaves them.
        let mut best = (ranked[0].0, f64::NEG_INFINITY);
        for (card, value) in ranked {
            let kept = value - giveaway.cost(card);
            if kept > best.1 {
                best = (card, kept);
            }
        }
        best.0
    }

    fn name(&self) -> &str {
        "expect"
    }
}

struct Build<'a> {
    rules: &'a Rules,
    live: Hand,
    prune: Prune,
    /// Exact partitions, for the win
    win: Solver,
    /// `HeuristicBot`'s two layouts, for [`Discards`]
    layouts: [Solver; 2],
    index: HashMap<(Hand, u32), u32>,
    /// The depth of the nodes being added
    depth: u32,
    graph: Graph,
}

impl Build<'_> {
    /// The node of (`hand`, `base`), added if new; 0 if it cannot win
    fn node(&mut self, hand: Hand, base: u32) -> u32 {
        if let Some(&node) = self.index.get(&(hand, base)) {
            return node;
        }
        let reaches = self.prune.generals.is_none_or(|generals| {
            reach(self.rules, 20 - hand.len(), hand, self.live, generals)
                .is_some_and(|hu| hu + base >= self.rules.threshold)
        });
        let node = if reaches {
            self.graph.hands.push(hand);
            self.graph.bases.push(base);
            self.graph.depths.push(self.depth);
            u32::try_from(self.graph.hands.len() - 1).expect("nodes fit in u32")
        } else {
            0
        };
        self.index.insert((hand, base), node);
        node
    }

    /// The 胡 `hand` wins at on `card` beside `base`, if it reaches the
    /// threshold, as [`Round::win`](crate::Round::win)
    fn wins(&mut self, hand: Hand, base: u32, card: Card) -> Option<u32> {
        locked_completions(hand, card)
            .iter()
            .filter_map(|meld| {
                let rest = self.win.best(hand - meld.cards().without(card))?;
                Some(base + self.rules.exposed.score(meld) + u32::try_from(rest).ok()?)
            })
            .max()
            .filter(|&hu| hu >= self.rules.threshold)
    }

    /// Add the group of nodes that claiming `card` into `meld` and
    /// discarding leads to, unless it has none
    fn group(&mut self, hand: Hand, base: u32, card: Card, meld: Meld) -> bool {
        let rest = hand - meld.cards().without(card);
        let base = base + self.rules.exposed.score(&meld);
        let start = self.graph.children.len();
        for (discard, _) in self.discards(rest, card).distinct() {
            let child = self.node(rest.without(discard), base);
            self.graph.children.push(child);
        }
        let added = self.graph.children.len() > start;
        if added {
            self.graph.groups.push(self.graph.children.len() as u32);
        }
        added
    }

    fn expand(&mut self, node: usize) {
        let (hand, base) = self.graph.state(node);
        for (card, count) in self.live.distinct() {
            let first = self.graph.groups.len() as u32;
            let kind = if let Some(hu) = self.wins(hand, base, card) {
                Some(Kind::Win(hu))
            } else if hand.count(card) == 3 {
                self.group(hand, base, card, Meld::Quad(card));
                Some(Kind::Kong)
            } else if card.rank == Rank::General {
                for meld in locked_completions(hand, card) {
                    if matches!(meld, Meld::Single(_) | Meld::Court(_)) {
                        self.group(hand, base, card, meld);
                    }
                }
                Some(Kind::General)
            } else {
                let pong =
                    hand.count(card) == 2 && self.group(hand, base, card, Meld::Triplet(card));
                for meld in locked_completions(hand, card) {
                    if !matches!(meld, Meld::Triplet(_) | Meld::Quad(_)) {
                        self.group(hand, base, card, meld);
                    }
                }
                (self.graph.groups.len() as u32 > first).then_some(Kind::Claim { pong })
            };
            if let Some(kind) = kind {
                self.graph.entries.push(Entry {
                    count: f64::from(count),
                    rank: card.rank,
                    kind,
                    first,
                    last: self.graph.groups.len() as u32,
                });
            }
        }
        self.graph.ends.push(self.graph.entries.len() as u32);
    }

    /// The discards a node tries after claiming `locked`; see [`Discards`]
    fn discards(&mut self, rest: Hand, locked: Card) -> Hand {
        let all = discardable(rest, Some(locked));
        if self.prune.discards == Discards::All {
            return all;
        }
        let mut loose = Hand::EMPTY;
        for solver in &mut self.layouts {
            loose += solver.layout(rest).expect("loose layouts exist").loose;
        }
        let free: Hand = all.iter().filter(|&card| loose.contains(card)).collect();
        if free.is_empty() {
            return all;
        }
        let live = self.live;
        let check = self.prune.discards == Discards::Live;
        let alone: Hand = free
            .iter()
            .filter(|&card| {
                !loose.iter().any(|other| {
                    mates(card, other)
                        && (!check
                            || live
                                .distinct()
                                .any(|(third, _)| mates(card, third) && mates(other, third)))
                })
            })
            .collect();
        if self.prune.discards == Discards::Loose || alone.is_empty() {
            free
        } else {
            alone
        }
    }
}

/// Whether two loose cards wait on a third to make a meld
pub(crate) fn mates(a: Card, b: Card) -> bool {
    let court = |rank| matches!(rank, Rank::General | Rank::Advisor | Rank::Elephant);
    match (a.rank, b.rank) {
        (Rank::Soldier, Rank::Soldier) => a.color != b.color,
        (Rank::Soldier, _) | (_, Rank::Soldier) => false,
        (x, y) => a.color == b.color && x != y && court(x) == court(y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round::options;
    use susik::Color;

    /// xorshift64, so that the tests need no `rand`
    struct Rng(u64);

    impl Rng {
        fn below(&mut self, n: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % n as u64) as usize
        }

        fn unit(&mut self) -> f64 {
            self.below(1 << 20) as f64 / f64::from(1 << 20)
        }

        /// `n` cards drawn from `from`
        fn hand(&mut self, from: Hand, n: usize) -> Hand {
            let mut cards: Vec<Card> = from.iter().collect();
            (0..n)
                .map(|_| cards.swap_remove(self.below(cards.len())))
                .collect()
        }
    }

    /// One public card a draw, so that `draws` is the cards to come
    fn root(graph: &Graph, outlook: &Outlook, draws: usize, payoff: Payoff) -> f64 {
        graph.values(outlook, draws, payoff)[graph.roots()[0]]
    }

    fn free() -> Outlook {
        Outlook {
            end: vec![0.0],
            cards: vec![1.0],
            loss: 0.0,
            eat: 0.5,
            general: 0.0,
            ..Outlook::even()
        }
    }

    /// A round starts with 25 draws left at every table, within the
    /// tables; with a shorter dead tail it starts beyond, where a leaf's
    /// curve would stand still
    #[test]
    fn the_shallow_graph_waits_for_its_tables() {
        for (players, dead_tail, searches) in
            [(2, 6, true), (3, 6, true), (4, 6, true), (4, 0, false)]
        {
            let mut rules = Rules::new();
            rules.players = players;
            rules.dead_tail = dead_tail;
            let mut pile: Vec<Card> = Hand::FULL.iter().collect();
            let mut rng = Rng(7);
            for i in (1..pile.len()).rev() {
                pile.swap(i, rng.below(i + 1));
            }
            let round = crate::Round::from_pile(rules, Seat::ALL[0], pile).unwrap();
            assert_eq!(round.phase(), crate::Phase::Discard(Seat::ALL[0]));
            let view = View::new(&round, Seat::ALL[0]);
            assert_eq!(ExpectBot::draws(&view) <= crate::eval::DRAWS, searches);
            let mut bot = ExpectBot::new(&rules).shallow(Eval::of(&rules), 0);
            assert_eq!(bot.searches(&view, 20), searches);
            // Nor does an outlook of few cards a draw bring it in early.
            let slow = Outlook {
                cards: vec![0.5],
                ..free()
            };
            assert_eq!(bot.clone().with_outlook(slow).searches(&view, 20), searches);
            let graded: Vec<_> = HeuristicBot::new(&rules)
                .discards(&view)
                .into_iter()
                .map(|(card, grade)| (card, f64::from(grade)))
                .collect();
            assert_eq!(bot.discards(&view) == graded, !searches);
        }
    }

    /// Two players play by their own table of that threshold, h × 1.3,
    /// and every other table by the table of four, its cards drawn evenly:
    /// the only ones whose own won their A/Bs
    #[test]
    fn only_two_play_by_their_own_outlook() {
        let four = ExpectBot::new(&Rules::new()).outlook;
        assert_eq!(four, Outlook::default());
        for rules in [Rules::new(), Rules::eight()] {
            for players in 2..=4 {
                let mut rules = rules;
                rules.players = players;
                let outlook = ExpectBot::new(&rules).outlook;
                if players == 2 {
                    let mut own = Outlook::new(&rules);
                    own.end.iter_mut().for_each(|h| *h *= 1.3);
                    assert_eq!(outlook, own);
                } else if (players, rules.threshold) == (4, 10) {
                    assert_eq!(outlook, four);
                } else {
                    let mut plain = Outlook {
                        drawn: vec![[1.0; 7]],
                        discarded: vec![[1.0; 7]],
                        ..Outlook::default()
                    };
                    plain.end.iter_mut().for_each(|h| *h *= END_SCALE);
                    assert_eq!(outlook, plain);
                }
            }
        }
    }

    /// A row no label reached keeps its initial bias, 0: the tables of two
    /// are learnt only as far as their own outlook counts
    #[test]
    fn every_row_a_table_reads_is_learnt() {
        for rules in [Rules::new(), Rules::eight()] {
            for players in 2..=4 {
                let mut rules = rules;
                rules.players = players;
                let rows = ExpectBot::new(&rules)
                    .outlook
                    .segments(crate::eval::DRAWS)
                    .len();
                let eval = Eval::of(&rules);
                let biases = &eval.params()[eval.params().len() - crate::eval::CARDS..];
                assert!(rows <= crate::eval::CARDS);
                assert!(biases[..rows].iter().all(|&b| b != 0.0), "{players}");
            }
        }
    }

    #[test]
    fn the_schedule_follows_the_segments() {
        let outlook = Outlook {
            end: vec![0.1, 0.2],
            cards: vec![2.0, 1.5],
            ..free()
        };
        // 2 + 1.5 + 1.5 less the card decided on, the last first
        assert_eq!(outlook.segments(2), [0, 0, 1, 2]);
        assert_eq!(outlook.segments(0), [0]);
    }

    #[test]
    fn one_wait_is_geometric() {
        // 俥傌 beside 9 胡 wins at 10 on 炮, and nothing else claims
        let rules = Rules::new();
        let hand: Hand = "俥傌...".parse().unwrap();
        let cannon = Card::new(Color::Red, Rank::Cannon);
        let elephant = Card::new(Color::White, Rank::Elephant);
        for (k, m) in [(1, 3), (2, 2), (3, 4)] {
            let live: Hand = std::iter::repeat_n(cannon, k)
                .chain(std::iter::repeat_n(elephant, m))
                .collect();
            let graph = Graph::new(&rules, live, &[(hand, 9)], Prune::default());
            let p = k as f64 / (k + m) as f64;
            for t in 0..8 {
                let want = 30.0 * (1.0 - (1.0 - p).powi(t));
                let got = root(&graph, &free(), t as usize, Payoff::Points);
                assert!(
                    (got - want).abs() < 1e-12,
                    "{k} of {}: {got} ≠ {want}",
                    k + m
                );
            }
        }
    }

    /// The recurrence expanded directly, claims judged by the round's own
    /// [`options`]
    struct Brute<'a> {
        rules: &'a Rules,
        live: Hand,
        outlook: Outlook,
        win: Solver,
        memo: HashMap<(Hand, u32, usize), f64>,
    }

    impl Brute<'_> {
        fn value(&mut self, hand: Hand, base: u32, t: usize) -> f64 {
            if t == 0 {
                return 0.0;
            }
            if let Some(&v) = self.memo.get(&(hand, base, t)) {
                return v;
            }
            let stay = self.value(hand, base, t - 1);
            let foul = -3.0 * f64::from(self.rules.threshold);
            // One card a draw, so every card but the last is drawn.
            let rates = if t == 1 {
                self.outlook.discarded[0]
            } else {
                self.outlook.drawn[0]
            };
            let share = if t == 1 {
                1.0 - self.outlook.own[0]
            } else {
                1.0
            };
            let total: f64 = self.live.iter().map(|c| rates[c.rank as usize]).sum();
            let mut sum = 0.0;
            for (card, count) in self.live.distinct() {
                let claim = |this: &mut Self, meld: Meld| {
                    let rest = hand - meld.cards().without(card);
                    let base = base + this.rules.exposed.score(&meld);
                    discardable(rest, Some(card))
                        .distinct()
                        .map(|(d, _)| this.value(rest.without(d), base, t - 1))
                        .fold(None, |best: Option<f64>, v| {
                            Some(best.map_or(v, |b| b.max(v)))
                        })
                        .unwrap_or(foul)
                };
                let win = locked_completions(hand, card)
                    .iter()
                    .filter_map(|meld| {
                        let rest = self.win.best(hand - meld.cards().without(card))?;
                        Some(base + self.rules.exposed.score(meld) + rest as u32)
                    })
                    .max()
                    .filter(|&hu| hu >= self.rules.threshold);
                let answer = if let Some(hu) = win {
                    3.0 * f64::from(hu)
                } else if hand.count(card) == 3 {
                    claim(self, Meld::Quad(card))
                } else {
                    let best = |this: &mut Self, eater: bool| {
                        let options = options(hand, card, eater);
                        let mut best = if options.forced() {
                            f64::NEG_INFINITY
                        } else {
                            stay
                        };
                        if options.pong {
                            best = best.max(claim(this, Meld::Triplet(card)));
                        }
                        for meld in options.eats {
                            best = best.max(claim(this, meld));
                        }
                        best
                    };
                    let e = if card.rank == Rank::General {
                        self.outlook.general
                    } else {
                        (self.outlook.eat / share).min(1.0)
                    };
                    e * best(self, true) + (1.0 - e) * best(self, false)
                };
                sum += share * f64::from(count) * rates[card.rank as usize] / total * answer;
            }
            let end = self.outlook.end[0];
            let v = end * self.outlook.loss + (1.0 - end) * sum;
            self.memo.insert((hand, base, t), v);
            v
        }
    }

    #[test]
    fn the_graph_is_the_recurrence() {
        let rules = Rules::new();
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let unpruned = Prune {
            generals: None,
            ..Prune::default()
        };
        let mut claimed = 0;
        for _ in 0..200 {
            let size = 1 + rng.below(6);
            let hand = rng.hand(Hand::FULL, size);
            let n = 8 + rng.below(16);
            let live = rng.hand(Hand::FULL - hand, n);
            let base = rng.below(12) as u32;
            let outlook = Outlook {
                end: vec![0.2 * rng.unit()],
                cards: vec![1.0],
                loss: -20.0 * rng.unit(),
                eat: rng.unit(),
                general: rng.unit(),
                drawn: vec![std::array::from_fn(|_| 0.1 + rng.unit())],
                discarded: vec![std::array::from_fn(|_| 0.1 + rng.unit())],
                own: vec![0.5 * rng.unit()],
            };
            let t = rng.below(5);
            let graph = Graph::new(&rules, live, &[(hand, base)], unpruned);
            let got = root(&graph, &outlook, t, Payoff::Points);
            let mut brute = Brute {
                rules: &rules,
                live,
                outlook,
                win: Solver::new(rules.concealed, None).locked(),
                memo: HashMap::new(),
            };
            let want = brute.value(hand, base, t);
            claimed += usize::from(graph.edges() > 0 && want != 0.0);
            assert!(
                (got - want).abs() <= 1e-9 * want.abs().max(1.0),
                "{hand} + {base} 胡, live {live}, t = {t}: {got} ≠ {want}"
            );
        }
        assert!(claimed > 50, "only {claimed} hands claim");
    }

    #[test]
    fn the_ceiling_prunes_no_win() {
        // Without eating generals, which the fixed `live` lets a hand eat
        // more often than they remain, a hand the ceiling rules out with
        // every live general cannot win
        let rules = Rules::new();
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        let [mut pruned, mut winning] = [0, 0];
        for _ in 0..100 {
            let n = 1 + rng.below(6);
            let hand = rng.hand(Hand::FULL, n);
            let live = Hand::FULL - hand;
            let base = rng.below(12) as u32;
            let generals = live
                .distinct()
                .filter(|(card, _)| card.rank == Rank::General)
                .map(|(_, n)| usize::from(n))
                .sum();
            let graph = |generals| {
                let prune = Prune {
                    generals,
                    ..Prune::default()
                };
                Graph::new(&rules, live, &[(hand, base)], prune)
            };
            let (cut, whole) = (graph(Some(generals)), graph(None));
            let (p, w) = (
                root(&cut, &free(), 6, Payoff::Win),
                root(&whole, &free(), 6, Payoff::Win),
            );
            assert!((p - w).abs() < 1e-12, "{hand} + {base}: {p} ≠ {w}");
            pruned += usize::from(cut.len() < whole.len());
            winning += usize::from(w > 0.0);
        }
        assert!(
            pruned > 20 && winning > 20,
            "{pruned} pruned, {winning} winning"
        );
    }

    #[test]
    fn exact_leaves_are_the_whole_graph() {
        let rules = Rules::new();
        let mut rng = Rng(0x0123_4567_89ab_cdef);
        let mut cut = 0;
        for _ in 0..40 {
            let n = 6 + rng.below(4);
            let hand = rng.hand(Hand::FULL, n);
            let base = rng.below(10) as u32;
            let live = rng.hand(Hand::FULL - hand, 40);
            let roots: Vec<_> = hand
                .distinct()
                .map(|(c, _)| (hand.without(c), base))
                .collect();
            let outlook = Outlook::default();
            let whole = Graph::new(&rules, live, &roots, Prune::default());
            let index: HashMap<_, _> = (1..whole.len()).map(|n| (whole.state(n), n)).collect();
            let want = whole.values(&outlook, 12, Payoff::Points);
            for depth in 0..4 {
                let prune = Prune {
                    depth: Some(depth),
                    ..Prune::default()
                };
                let graph = Graph::new(&rules, live, &roots, prune);
                let nodes: Vec<_> = graph
                    .leaves()
                    .iter()
                    .map(|&l| index[&graph.state(l)])
                    .collect();
                let curves = whole.curves(&outlook, 12, Payoff::Points, &nodes);
                let got = graph.values_with(&outlook, 12, Payoff::Points, &curves);
                cut += usize::from(graph.edges() < whole.edges());
                for (&g, &w) in graph.roots().iter().zip(whole.roots()) {
                    assert_eq!(got[g].to_bits(), want[w].to_bits(), "depth {depth}");
                }
            }
            // A depth past the graph's leaves nothing and changes nothing
            let deep = Prune {
                depth: Some(20),
                ..Prune::default()
            };
            let graph = Graph::new(&rules, live, &roots, deep);
            assert!(graph.leaves().is_empty());
            assert_eq!(graph.values(&outlook, 12, Payoff::Points), want);
        }
        assert!(cut > 30, "only {cut} graphs cut");
    }

    #[test]
    fn more_cards_never_hurt_a_free_hand() {
        // No set to kong and no general to eat, so every claim may be
        // passed
        let rules = Rules::new();
        let mut rng = Rng(0xdead_beef_cafe_f00d);
        for _ in 0..50 {
            let n = 4 + rng.below(6);
            let hand = rng.hand(Hand::FULL, n);
            if hand.distinct().any(|(_, n)| n >= 3) {
                continue;
            }
            let live = rng.hand(Hand::FULL - hand, 40);
            let base = rng.below(10) as u32;
            let graph = Graph::new(&rules, live, &[(hand, base)], Prune::default());
            let chances: Vec<f64> = (0..10)
                .map(|t| root(&graph, &free(), t, Payoff::Win))
                .collect();
            assert!(
                chances.windows(2).all(|w| w[0] <= w[1] + 1e-15),
                "{chances:?}"
            );
            assert!(chances.iter().all(|&p| (0.0..=1.0 + 1e-12).contains(&p)));
        }
    }
}
