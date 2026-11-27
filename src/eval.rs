//! What a hand is worth in points, read from tables instead of searched.
//!
//! Cards meld only within nine blocks: each colour's 將士象 and 車馬炮, and
//! the soldiers.  A block's **pattern** is, for each card in it, how many
//! the hand holds and how many are live; patterns that swap the colours, 士
//! with 象, or 車, 馬 and 炮 among themselves are one.  Each pattern, the
//! exposed 胡 and the count of live cards look up a vector; their sum, past
//! a ReLU, gives the value by each count of public cards to come.  The
//! tables are learnt from [`Graph`](crate::Graph)'s values with
//! [`Outlook::default`](crate::Outlook) and h × 1.5, under the rules of 十胡
//! (`doc/eval.md`).  They are a table's own: [`Eval::of`] has another set
//! for two players.

use susik::{Card, Color, Hand, Rank};

/// The most public cards to come a curve runs to: what a round of four
/// starts with, the most the labels hold.  A longer round reads the last.
pub const CARDS: usize = 45;

/// The draws a round starts with at every table
/// ([`Rules::dead`](crate::Rules::dead)): [`CARDS`] public cards by
/// [`Outlook::default`](crate::Outlook), and the most the labels hold
/// whatever a bot's own outlook counts a draw
pub const DRAWS: usize = 25;

/// Patterns of a 車馬炮 block: three cards that swap
const FIELDS: usize = 680;
/// Patterns of a 將士象 block: the general, and 士象 that swap
const COURTS: usize = 15 * 120;
/// Patterns of the soldiers: four colours that swap
const SOLDIERS: usize = 3060;
/// Exposed 胡 0 to 15, more counted as 15
const BASES: usize = 16;
/// Live cards by eights, 88 or more as the last
const LIVES: usize = 12;

/// Rows of the one embedding table: fields, courts, soldiers, bases, lives
pub const ROWS: usize = FIELDS + COURTS + SOLDIERS + BASES + LIVES;

/// The rows a position sums: four fields, four courts, the soldiers, the
/// exposed 胡 and the live count
pub type Features = [u16; 11];

/// (held, live) of one card as one of 15 values, held and live summing to
/// at most 4
fn cell(hand: Hand, live: Hand, card: Card) -> usize {
    let h = usize::from(hand.count(card));
    let l = usize::from(live.count(card)).min(4 - h);
    5 * h - h * (h.saturating_sub(1)) / 2 + l
}

const fn choose(n: usize, k: usize) -> usize {
    let mut r = 1;
    let mut i = 0;
    while i < k {
        r = r * (n - i) / (i + 1);
        i += 1;
    }
    r
}

/// The rank of a multiset of cells among the multisets of its size
fn multiset<const K: usize>(mut cells: [usize; K]) -> usize {
    cells.sort_unstable();
    cells
        .iter()
        .enumerate()
        .map(|(i, &c)| choose(c + i, i + 1))
        .sum()
}

/// The rows of (`hand`, `base`) while `live` cards are unseen
#[must_use]
pub fn features(hand: Hand, base: u32, live: Hand) -> Features {
    let at = |color, rank| cell(hand, live, Card::new(color, rank));
    let mut rows = [0; 11];
    for (i, color) in Color::ALL.into_iter().enumerate() {
        let field = multiset([
            at(color, Rank::Chariot),
            at(color, Rank::Horse),
            at(color, Rank::Cannon),
        ]);
        let court = at(color, Rank::General) * 120
            + multiset([at(color, Rank::Advisor), at(color, Rank::Elephant)]);
        rows[i] = field;
        rows[4 + i] = FIELDS + court;
    }
    rows[8] = FIELDS + COURTS + multiset(Color::ALL.map(|color| at(color, Rank::Soldier)));
    rows[9] = FIELDS + COURTS + SOLDIERS + (base as usize).min(BASES - 1);
    rows[10] = FIELDS + COURTS + SOLDIERS + BASES + (live.len() / 8).min(LIVES - 1);
    rows.map(|row| u16::try_from(row).expect("rows fit in u16"))
}

/// The pattern tables
///
/// The parameters run: the embedding, [`ROWS`] rows of `dim`; then the
/// output weights, [`CARDS`] rows of `dim`; then the output biases, one
/// for each count of public cards to come from 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Eval {
    dim: usize,
    params: Vec<f32>,
    /// The 底 the labels were paid with: see [`Eval::under`]
    base: Option<u32>,
}

impl Default for Eval {
    /// The tables of batch 3 of `doc/eval.md`: width 16, learnt from
    /// `eval label 2000 1795800000`; held-out RMSE 0.78 points
    // ponytail: f32, 359 KB; quantize if the wasm build minds
    fn default() -> Self {
        Self::from_bytes(include_bytes!("eval-16.bin")).expect("the tables built in")
    }
}

impl Eval {
    /// The tables a table of `rules` plays by: two players have their own,
    /// learnt from `PLAYERS=2 EXPECT=1.3 eval label 2000 1799800000` at 10 胡
    /// (held-out RMSE 0.53 points) and from `PLAYERS=2 THRESHOLD=8 EXPECT=1.3
    /// eval label 2000 1801600000` at 8 胡 (0.40 points); four at 10 胡 with
    /// a 底 of 1 have theirs, learnt [`under`](Self::under) it from `BASE=1
    /// EXPECT=1 eval label 2000 1806000000` (0.25 of its points); any other
    /// table has [`Eval::default`]
    ///
    /// The two players' tables go with the outlook
    /// [`ExpectBot::new`](crate::ExpectBot::new) gives them, which counts 36
    /// public cards a round at 10 胡 and 35 at 8: their rows beyond are not
    /// learnt.
    // ponytail: three players' own tables measured insufficient data against
    // the four-player ones (`doc/eval.md`, "Tables for two and three
    // players"), so they stay out of the binary.  Under a 底 only the one
    // table is labelled (`doc/variants.md`, "An `Eval` labelled under a 底").
    #[must_use]
    pub fn of(rules: &crate::Rules) -> Self {
        let (bytes, learnt): (&[u8], _) = match (rules.players, rules.threshold, rules.base) {
            (4, 10, Some(1)) => (include_bytes!("eval-16-base.bin"), rules.base),
            (2, 10, _) => (include_bytes!("eval-16-2.bin"), None),
            (2, 8, _) => (include_bytes!("eval-16-2-8.bin"), None),
            _ => return Self::default(),
        };
        Self::from_bytes(bytes)
            .expect("the tables built in")
            .under(learnt)
    }

    /// How many parameters tables of width `dim` have
    #[must_use]
    pub const fn len(dim: usize) -> usize {
        (ROWS + CARDS) * dim + CARDS
    }

    /// Tables of width `dim` from their parameters; `None` if the count is
    /// not [`len(dim)`](Self::len)
    #[must_use]
    pub fn new(dim: usize, params: Vec<f32>) -> Option<Self> {
        (dim > 0 && params.len() == Self::len(dim)).then_some(Self {
            dim,
            params,
            base: None,
        })
    }

    /// The same tables, as ones learnt from values paid with the 底 `base`
    /// ([`Rules::base`](crate::Rules::base)); tables are taken for learnt
    /// on the whole 胡 unless this says otherwise
    ///
    /// Past its gate an [`ExpectBot`](crate::ExpectBot) reads a table's 底
    /// only with tables learnt under it, and pays the whole 胡 with any
    /// other, as their leaves are.
    #[must_use]
    pub fn under(mut self, base: Option<u32>) -> Self {
        self.base = base;
        self
    }

    /// The 底 the tables were learnt under: [`under`](Self::under)'s
    #[must_use]
    pub const fn base(&self) -> Option<u32> {
        self.base
    }

    /// Tables from their parameters as little-endian `f32`, the width
    /// following from the count
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let (words, rest) = bytes.as_chunks::<4>();
        let params: Vec<f32> = words.iter().map(|&b| f32::from_le_bytes(b)).collect();
        if !rest.is_empty() || params.len() < CARDS {
            return None;
        }
        let dim = (params.len() - CARDS) / (ROWS + CARDS);
        Self::new(dim, params)
    }

    /// The parameters as little-endian `f32`
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.params.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    /// The width of the vectors
    #[must_use]
    pub const fn dim(&self) -> usize {
        self.dim
    }

    /// The parameters, laid out as the type says
    #[must_use]
    pub fn params(&self) -> &[f32] {
        &self.params
    }

    /// The value of the rows `features` by public cards to come, from 0 to
    /// [`CARDS`]; nothing to come is worth 0
    #[must_use]
    pub fn forward(&self, features: &Features) -> Vec<f64> {
        let d = self.dim;
        let mut acc = vec![0.0_f32; d];
        for &row in features {
            let row = &self.params[usize::from(row) * d..][..d];
            acc.iter_mut().zip(row).for_each(|(a, x)| *a += x);
        }
        acc.iter_mut().for_each(|a| *a = a.max(0.0));
        let weights = &self.params[ROWS * d..][..CARDS * d];
        let biases = &self.params[(ROWS + CARDS) * d..];
        std::iter::once(0.0)
            .chain(
                weights.chunks_exact(d).zip(biases).map(|(w, b)| {
                    f64::from(w.iter().zip(&acc).map(|(w, a)| w * a).sum::<f32>() + b)
                }),
            )
            .collect()
    }

    /// The value of (`hand`, `base`) by public cards to come while `live`
    /// cards are unseen, as [`Graph::values_with`](crate::Graph) reads a
    /// leaf
    #[must_use]
    pub fn curve(&self, hand: Hand, base: u32, live: Hand) -> Vec<f64> {
        self.forward(&features(hand, base, live))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, Outlook, Payoff, Prune, Rules};
    use std::collections::HashSet;

    /// Every cell of `k` cards, as base-15 digits
    fn every<const K: usize>() -> impl Iterator<Item = [usize; K]> {
        (0..15_usize.pow(K as u32)).map(|mut n| {
            std::array::from_fn(|_| {
                let d = n % 15;
                n /= 15;
                d
            })
        })
    }

    #[test]
    fn patterns_number_their_tables() {
        let sorted = |mut c: Vec<usize>| {
            c.sort_unstable();
            c
        };
        let check = |ids: Vec<(usize, Vec<usize>)>, count: usize| {
            let mut seen = vec![None; count];
            for (id, key) in ids {
                match &seen[id] {
                    None => seen[id] = Some(key),
                    Some(other) => assert_eq!(*other, key, "two patterns share {id}"),
                }
            }
            assert!(seen.iter().all(Option::is_some), "a gap below {count}");
        };
        check(
            every::<3>()
                .map(|c| (multiset(c), sorted(c.to_vec())))
                .collect(),
            FIELDS,
        );
        check(
            every::<3>()
                .map(|[g, a, e]| {
                    (
                        g * 120 + multiset([a, e]),
                        [vec![g], sorted(vec![a, e])].concat(),
                    )
                })
                .collect(),
            COURTS,
        );
        check(
            every::<4>()
                .map(|c| (multiset(c), sorted(c.to_vec())))
                .collect(),
            SOLDIERS,
        );
        let cells: HashSet<usize> = (0..=4)
            .flat_map(|h| (0..=4 - h).map(move |l| (h, l)))
            .map(|(h, l)| {
                let card = Card::new(Color::Red, Rank::Horse);
                let hand = (0..h).fold(Hand::EMPTY, |x, _| x.with(card));
                let live = (0..l).fold(Hand::EMPTY, |x, _| x.with(card));
                cell(hand, live, card)
            })
            .collect();
        assert_eq!(cells, (0..15).collect());
    }

    /// xorshift64, so that the tests need no `rand`
    struct Rng(u64);

    impl Rng {
        fn below(&mut self, n: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % n as u64) as usize
        }

        fn shuffle<T>(&mut self, xs: &mut [T]) {
            for i in (1..xs.len()).rev() {
                xs.swap(i, self.below(i + 1));
            }
        }
    }

    /// A map of the cards that keeps every meld and its score: the colours
    /// of the court and field blocks, the soldiers' colours apart from
    /// them, 士 with 象, and 車馬炮 per colour
    fn symmetry(rng: &mut Rng) -> impl Fn(Card) -> Card {
        let mut colors = Color::ALL;
        let mut soldiers = Color::ALL;
        rng.shuffle(&mut colors);
        rng.shuffle(&mut soldiers);
        let swap = rng.below(2) == 1;
        let fields: [[Rank; 3]; 4] = std::array::from_fn(|_| {
            let mut ranks = [Rank::Chariot, Rank::Horse, Rank::Cannon];
            rng.shuffle(&mut ranks);
            ranks
        });
        move |card| {
            let c = card.color as usize;
            match card.rank {
                Rank::Soldier => Card::new(soldiers[c], Rank::Soldier),
                Rank::General => Card::new(colors[c], Rank::General),
                Rank::Advisor | Rank::Elephant => {
                    let flip = (card.rank == Rank::Advisor) == swap;
                    let rank = if flip { Rank::Elephant } else { Rank::Advisor };
                    Card::new(colors[c], rank)
                }
                rank => Card::new(colors[c], fields[c][rank as usize - Rank::Chariot as usize]),
            }
        }
    }

    #[test]
    fn symmetric_hands_share_patterns_and_values() {
        let rules = Rules::new();
        let outlook = Outlook::default();
        let mut rng = Rng(0x5eed_0fca_4d17);
        let mut claimed = 0;
        for _ in 0..100 {
            let mut deck: Vec<Card> = Hand::FULL.iter().collect();
            rng.shuffle(&mut deck);
            let n = 5 + rng.below(5);
            let hand: Hand = deck[..n].iter().copied().collect();
            let live: Hand = deck[n..n + 40].iter().copied().collect();
            let base = 4 + rng.below(8) as u32;
            let map = symmetry(&mut rng);
            let (h2, l2) = (
                hand.iter().map(&map).collect::<Hand>(),
                live.iter().map(&map).collect::<Hand>(),
            );
            let key = |f: Features| {
                let mut f = f;
                f[..4].sort_unstable();
                f[4..8].sort_unstable();
                f
            };
            assert_eq!(key(features(hand, base, live)), key(features(h2, base, l2)));
            let value = |hand: Hand, live| {
                let roots: Vec<_> = hand
                    .distinct()
                    .map(|(c, _)| (hand.without(c), base))
                    .collect();
                let graph = Graph::new(&rules, live, &roots, Prune::default());
                let values = graph.values(&outlook, 12, Payoff::Points);
                let mut roots: Vec<f64> = graph.roots().iter().map(|&r| values[r]).collect();
                roots.sort_by(f64::total_cmp);
                (roots, graph.edges())
            };
            let ((a, edges), (b, _)) = (value(hand, live), value(h2, l2));
            claimed += usize::from(edges > 0);
            for (x, y) in a.iter().zip(&b) {
                assert!((x - y).abs() < 1e-9, "{hand} / {h2}: {a:?} ≠ {b:?}");
            }
        }
        assert!(claimed > 20, "only {claimed} graphs claim");
    }

    #[test]
    fn the_tables_are_built_in() {
        assert_eq!(Eval::default().dim(), 16);
    }

    /// Only two players have tables of their own, one for each threshold
    #[test]
    fn two_have_their_own_tables() {
        for rules in [Rules::new(), Rules::eight()] {
            for players in 2..=4 {
                let mut rules = rules;
                rules.players = players;
                assert_eq!(Eval::of(&rules) != Eval::default(), players == 2);
                assert_eq!(Eval::of(&rules).dim(), 16);
            }
        }
        let two = |rules: Rules| {
            Eval::of(&Rules {
                players: 2,
                ..rules
            })
        };
        assert!(two(Rules::new()) != two(Rules::eight()));
    }

    /// Under a 底 only four players at 10 胡 and a 底 of 1 have tables
    /// learnt under it; every other table keeps the ones it has without
    #[test]
    fn one_table_is_learnt_under_a_base() {
        for rules in [Rules::new(), Rules::eight()] {
            for players in 2..=4 {
                for base in [1, 2] {
                    let plain = Rules { players, ..rules };
                    let under = Rules {
                        base: Some(base),
                        ..plain
                    };
                    let own = (players, rules.threshold, base) == (4, 10, 1);
                    assert_eq!(Eval::of(&under).base(), own.then_some(1));
                    assert_eq!(Eval::of(&under) != Eval::of(&plain), own);
                }
            }
        }
    }

    /// A row no label reached keeps its initial bias, 0, and a round longer
    /// than the labels reads it for every leaf
    #[test]
    fn every_row_of_the_tables_is_learnt() {
        let eval = Eval::default();
        let biases = &eval.params()[(ROWS + CARDS) * eval.dim()..];
        assert!(biases.iter().all(|&b| b != 0.0));
    }

    #[test]
    fn the_layout_round_trips() {
        let params: Vec<f32> = (0..Eval::len(2)).map(|i| i as f32 * 0.5).collect();
        let eval = Eval::new(2, params).unwrap();
        assert_eq!(Eval::from_bytes(&eval.to_bytes()), Some(eval.clone()));
        assert_eq!(Eval::new(3, vec![0.0; Eval::len(2)]), None);
        // One live row of (1, 1) and the rest 0: relu(2) per t
        let mut params = vec![0.0; Eval::len(1)];
        params[ROWS - 1] = 2.0;
        params[ROWS..ROWS + CARDS].fill(1.5);
        params[ROWS + CARDS..].fill(-1.0);
        let curve = Eval::new(1, params)
            .unwrap()
            .curve(Hand::EMPTY, 0, Hand::FULL);
        assert_eq!(curve.len(), CARDS + 1);
        assert_eq!(curve[0], 0.0);
        assert!(curve[1..].iter().all(|&v| v == 2.0));
    }
}
