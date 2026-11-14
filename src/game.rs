//! The scoreboard across deals.

use crate::round::{DealError, Round, RoundResult};
use crate::{Rules, Seat};
use susik::Card;

/// Scores and the dealer across rounds
///
/// The winner collects their 胡 from each other seat and deals next; after an
/// exhausted pile the same dealer deals again, as after a foul (相公), whose
/// seat pays the threshold to each other seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Game {
    rules: Rules,
    dealer: Seat,
    scores: [i32; Seat::COUNT],
}

impl Game {
    /// A fresh scoreboard with `dealer` to deal first
    #[must_use]
    pub const fn new(rules: Rules, dealer: Seat) -> Self {
        Self {
            rules,
            dealer,
            scores: [0; Seat::COUNT],
        }
    }

    /// The rules in force
    #[must_use]
    #[inline]
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// Who deals the next round
    #[must_use]
    #[inline]
    pub const fn dealer(&self) -> Seat {
        self.dealer
    }

    /// Scores in [`Seat::ALL`] order
    #[must_use]
    #[inline]
    pub const fn scores(&self) -> [i32; Seat::COUNT] {
        self.scores
    }

    /// Deal the next round from a pile in draw order
    ///
    /// # Errors
    ///
    /// See [`Round::from_pile`].
    pub fn round_from_pile(&self, pile: Vec<Card>) -> Result<Round, DealError> {
        Round::from_pile(self.rules, self.dealer, pile)
    }

    /// Deal the next round from a shuffled deck
    #[cfg(feature = "rand")]
    #[must_use]
    pub fn deal(&self, rng: &mut (impl rand::Rng + ?Sized)) -> Round {
        Round::deal(self.rules, self.dealer, rng)
    }

    /// Record a finished round: pay the winner and pass them the deal, or
    /// collect a foul's penalty
    pub fn settle(&mut self, result: &RoundResult) {
        // The winner collects their 胡; a foul "collects" minus the threshold.
        let (collector, hu) = match (result.winner, result.foul) {
            (Some(winner), _) => (winner, to_i32(result.hu)),
            (None, Some(foul)) => (foul, -to_i32(self.rules.threshold)),
            (None, None) => return,
        };
        let others = Seat::COUNT as i32 - 1;
        for seat in Seat::ALL {
            self.scores[seat.index()] += if seat == collector { others * hu } else { -hu };
        }
        if let Some(winner) = result.winner {
            self.dealer = winner;
        }
    }
}

/// 胡 are far below `i32::MAX`.
fn to_i32(n: u32) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX / 4)
}
