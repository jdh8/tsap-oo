//! The scoring table and the thresholds of 十胡.

use susik::Values;

/// The rules of a game
///
/// [`Rules::new`] is the common Taiwanese table.  Start from it and change
/// fields; the struct is non-exhaustive so a literal will not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Rules {
    /// Minimum 胡 to win: the eponymous ten, or eight in 八胡
    pub threshold: u32,
    /// Cards at the bottom of the pile that are never drawn (廢牌); a round
    /// that reaches them is exhausted
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
    /// Ten to win, six dead cards, and per meld: lone general 1, pair 0,
    /// 將士象 2, 車馬包 1, soldiers of three colours 3 and of four 5,
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
