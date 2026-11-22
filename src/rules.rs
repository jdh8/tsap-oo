//! The scoring table and the thresholds of 十胡.

use crate::Seat;
use susik::Values;

/// The rules of a game
///
/// [`Rules::new`] is the common Taiwanese table.  Start from it and change
/// fields; the struct is non-exhaustive so a literal will not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Rules {
    /// Players at the table, 2 to [`Seat::COUNT`](crate::Seat::COUNT): the
    /// first seats of [`Seat::ALL`](crate::Seat::ALL)
    pub players: usize,
    /// Minimum 胡 to win: the eponymous ten, or eight in 八胡
    pub threshold: u32,
    /// Cards at the bottom of the pile that are never drawn (廢牌) at a full
    /// table; [`Rules::dead`] adds the hands of the seats left empty
    pub dead_tail: usize,
    /// Values of melds dealt into the hand and never shown
    pub concealed: Values,
    /// Values of melds formed with a public card: a claim, the winning
    /// card, or a dealt triplet konged with its fourth
    pub exposed: Values,
}

impl Rules {
    /// The common Taiwanese 十胡 table
    ///
    /// Four players, ten to win, six dead cards ([`Rules::dead`] at a smaller
    /// table), and per meld: lone general 1, pair 0,
    /// 將士象 2, 車馬炮 1, soldiers of three colours 3 and of four 5,
    /// triplet 3 concealed or 1 ponged, quad 8 concealed or 6 konged.
    #[must_use]
    pub const fn new() -> Self {
        let concealed = Values {
            single: 1,
            pair: 0,
            triplet: 3,
            quad: 8,
            court: 2,
            field: 1,
            three_soldiers: 3,
            four_soldiers: 5,
        };
        Self {
            players: 4,
            threshold: 10,
            dead_tail: 6,
            concealed,
            exposed: Values {
                triplet: 1,
                quad: 6,
                ..concealed
            },
        }
    }
}

impl Rules {
    /// Cards at the bottom of the pile that are never drawn; a round that
    /// reaches them is exhausted
    ///
    /// [`Rules::dead_tail`], and the twenty cards of a hand for each seat
    /// of [`Seat::COUNT`] left empty: 6 at four, 26 at three, 46 at two.
    /// Every table then draws the same 25 cards a round, and exhausts about
    /// as often.  No source says what a smaller table changes; with only
    /// the six, a table of two had 65 cards to draw and never ran out
    /// (`doc/variants.md`).
    #[must_use]
    pub const fn dead(&self) -> usize {
        self.dead_tail + 20 * Seat::COUNT.saturating_sub(self.players)
    }

    /// 八胡: the same table with eight to win
    ///
    /// A 相公 pays the threshold, so eight here too.
    #[must_use]
    pub const fn eight() -> Self {
        let mut rules = Self::new();
        rules.threshold = 8;
        rules
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}
