//! The scoreboard across deals.

use crate::round::{DealError, Round, RoundResult};
use crate::{Rules, Seat};
use susik::Card;

/// Scores and the dealer across rounds
///
/// The winner collects their 胡 from each of the other three seats and deals
/// next; after an exhausted pile the same dealer deals again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Game {
    rules: Rules,
    dealer: Seat,
    scores: [i32; 4],
}

impl Game {
    /// A fresh scoreboard with `dealer` to deal first
    #[must_use]
    pub const fn new(rules: Rules, dealer: Seat) -> Self {
        Self {
            rules,
            dealer,
            scores: [0; 4],
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
    pub const fn scores(&self) -> [i32; 4] {
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

    /// Record a finished round: pay the winner and pass them the deal
    pub fn settle(&mut self, result: &RoundResult) {
        let Some(winner) = result.winner else {
            return;
        };
        let hu = i32::try_from(result.hu).unwrap_or(i32::MAX / 4);
        for seat in Seat::ALL {
            self.scores[seat as usize] += if seat == winner { 3 * hu } else { -hu };
        }
        self.dealer = winner;
    }
}
