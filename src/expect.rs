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
use crate::{Rules, discardable};
use std::collections::HashMap;
use susik::{Card, Hand, Meld, Rank, Solver, locked_completions};

/// The table as one seat models it
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outlook {
    /// The chance another seat ends the round before each public card
    pub end: f64,
    /// What the seat scores then: minus what it pays the winner
    pub loss: f64,
    /// The chance a public card other than a general comes to the seat to
    /// eat: its own draw, or its 上家's that the 上家 lets go, or the
    /// 上家's discard, with no pong from another seat
    pub eat: f64,
    /// The chance a general is the seat's own draw, which it must eat
    pub general: f64,
}

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
}

impl Default for Prune {
    fn default() -> Self {
        Self {
            generals: Some(1),
            discards: Discards::All,
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
    /// The chance the next public card is this one
    weight: f64,
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
    roots: Vec<usize>,
    /// The hand of each node
    hands: Vec<Hand>,
    /// Each node's entries end here in `entries`
    ends: Vec<u32>,
    entries: Vec<Entry>,
    /// Each group's children end here in `children`
    groups: Vec<u32>,
    children: Vec<u32>,
    /// Each node's chance that the next public card is none of its entries
    rest: Vec<f64>,
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
            bases: vec![0],
            graph: Self {
                threshold: rules.threshold,
                roots: Vec::new(),
                hands: vec![Hand::EMPTY],
                ends: Vec::new(),
                entries: Vec::new(),
                groups: Vec::new(),
                children: Vec::new(),
                rest: Vec::new(),
            },
        };
        build.graph.roots = roots
            .iter()
            .map(|&(hand, base)| build.node(hand, base) as usize)
            .collect();
        build.graph.ends.push(0);
        build.graph.rest.push(1.0);
        let mut node = 1;
        while node < build.graph.hands.len() {
            build.expand(node);
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

    /// The value of every node, under `payoff`, with `cards` public cards
    /// to come
    #[must_use]
    pub fn values(&self, outlook: &Outlook, cards: usize, payoff: Payoff) -> Vec<f64> {
        let (loss, foul) = match payoff {
            Payoff::Points => (outlook.loss, -3.0 * f64::from(self.threshold)),
            Payoff::Win => (0.0, 0.0),
        };
        let mut before = vec![0.0; self.len()];
        let mut after = before.clone();
        for _ in 0..cards {
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
                let mut sum = stay * self.rest[node];
                for entry in &self.entries[first as usize..self.ends[node] as usize] {
                    let answer = match entry.kind {
                        Kind::Win(hu) => match payoff {
                            Payoff::Points => 3.0 * f64::from(hu),
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
                            let eat = (eats..entry.last).map(best).fold(pass, f64::max);
                            outlook.eat * eat + (1.0 - outlook.eat) * pass
                        }
                    };
                    sum += entry.weight * answer;
                }
                *value = outlook.end * loss + (1.0 - outlook.end) * sum;
            }
            std::mem::swap(&mut before, &mut after);
        }
        before
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
    /// The exposed 胡 of each node
    bases: Vec<u32>,
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
            self.bases.push(base);
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
        let (hand, base) = (self.graph.hands[node], self.bases[node]);
        let total = self.live.len() as f64;
        let mut rest = 1.0;
        for (card, count) in self.live.distinct() {
            let weight = f64::from(count) / total;
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
                rest -= weight;
                self.graph.entries.push(Entry {
                    weight,
                    kind,
                    first,
                    last: self.graph.groups.len() as u32,
                });
            }
        }
        self.graph.ends.push(self.graph.entries.len() as u32);
        self.graph.rest.push(rest);
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
fn mates(a: Card, b: Card) -> bool {
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

    fn root(graph: &Graph, outlook: &Outlook, cards: usize, payoff: Payoff) -> f64 {
        graph.values(outlook, cards, payoff)[graph.roots()[0]]
    }

    const FREE: Outlook = Outlook {
        end: 0.0,
        loss: 0.0,
        eat: 0.5,
        general: 0.0,
    };

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
                let got = root(&graph, &FREE, t as usize, Payoff::Points);
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
            let total = self.live.len() as f64;
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
                        self.outlook.eat
                    };
                    e * best(self, true) + (1.0 - e) * best(self, false)
                };
                sum += f64::from(count) / total * answer;
            }
            let v = self.outlook.end * self.outlook.loss + (1.0 - self.outlook.end) * sum;
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
            discards: Discards::All,
        };
        let mut claimed = 0;
        for _ in 0..200 {
            let size = 1 + rng.below(6);
            let hand = rng.hand(Hand::FULL, size);
            let n = 8 + rng.below(16);
            let live = rng.hand(Hand::FULL - hand, n);
            let base = rng.below(12) as u32;
            let outlook = Outlook {
                end: 0.2 * rng.unit(),
                loss: -20.0 * rng.unit(),
                eat: rng.unit(),
                general: rng.unit(),
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
                    discards: Discards::All,
                };
                Graph::new(&rules, live, &[(hand, base)], prune)
            };
            let (cut, whole) = (graph(Some(generals)), graph(None));
            let (p, w) = (
                root(&cut, &FREE, 6, Payoff::Win),
                root(&whole, &FREE, 6, Payoff::Win),
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
                .map(|t| root(&graph, &FREE, t, Payoff::Win))
                .collect();
            assert!(
                chances.windows(2).all(|w| w[0] <= w[1] + 1e-15),
                "{chances:?}"
            );
            assert!(chances.iter().all(|&p| (0.0..=1.0 + 1e-12).contains(&p)));
        }
    }
}
