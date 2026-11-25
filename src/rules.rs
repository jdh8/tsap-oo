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
    /// Minimum 胡 to win: the eponymous 10, or 8
    pub threshold: u32,
    /// Cards at the bottom of the pile that are never drawn (廢牌) at a full
    /// table; [`Rules::dead`] adds the hands of the seats left empty
    pub dead_tail: usize,
    /// Values of melds dealt into the hand and never shown
    pub concealed: Values,
    /// Values of melds formed with a public card: a claim, the winning
    /// card, or a dealt triplet konged with its fourth
    pub exposed: Values,
    /// 底: what each seat pays a winner besides the 胡 above the threshold,
    /// or `None` for the whole 胡 and no 底; see [`Rules::pays`]
    pub base: Option<u32>,
    /// 點花: at a win the top card of the pile is turned, and each card like
    /// it in the winner's layout is one 花胡, paid as one 胡 more; see
    /// [`RoundResult::flower`](crate::RoundResult::flower)
    pub flower: bool,
}

impl Rules {
    /// The common Taiwanese 十胡 table
    ///
    /// Four players, ten to win, six dead cards ([`Rules::dead`] at a smaller
    /// table), and per meld: lone general 1, pair 0,
    /// 將士象 2, 車馬炮 1, soldiers of three colours 3 and of four 5,
    /// triplet 3 concealed or 1 ponged, quad 8 concealed or 6 konged.
    /// Each seat pays the winner's whole 胡: no 底 and no 點花.
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
            base: None,
            flower: false,
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

    /// 8 胡: the same table with eight to win
    ///
    /// A 相公 pays the threshold, so eight here too.
    #[must_use]
    pub const fn eight() -> Self {
        let mut rules = Self::new();
        rules.threshold = 8;
        rules
    }
}

impl Rules {
    /// What each other seat pays a winner of `hu` 胡, before any 花胡
    ///
    /// The whole 胡, or with a 底 ([`Rules::base`]) the 底 and the 胡 above
    /// the threshold: at ten to win and a 底 of 1, 13 胡 is paid 1 + 3.
    /// Board Game Arena's Chinese help has this with its 1 點押點; the
    /// sources are in `../tsap-oo-book/research/settlement.md`.
    #[must_use]
    pub const fn pays(&self, hu: u32) -> u32 {
        match self.base {
            None => hu,
            Some(base) => base + hu.saturating_sub(self.threshold),
        }
    }

    /// What a 相公 pays each other seat: the threshold, and the 底 if any
    #[must_use]
    pub const fn foul(&self) -> u32 {
        match self.base {
            None => self.threshold,
            Some(base) => self.threshold + base,
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 13 胡 is paid whole, or 1 + 3 with a 底 of 1; a 相公 pays the 底 too.
    #[test]
    fn a_base_pays_the_hu_above_the_threshold() {
        let mut rules = Rules::new();
        assert_eq!((rules.pays(13), rules.foul()), (13, 10));
        rules.base = Some(1);
        assert_eq!((rules.pays(13), rules.pays(10), rules.foul()), (4, 1, 11));
        rules.threshold = 8;
        assert_eq!((rules.pays(13), rules.foul()), (6, 9));
    }
}
