#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod driver;
pub mod game;
pub mod heuristic;
pub mod round;
pub mod rules;
pub mod strategy;
pub mod view;

pub use driver::{Step, Table};
pub use game::Game;
pub use heuristic::HeuristicBot;
pub use round::{
    Claim, ClaimOptions, DealError, Event, Offer, Phase, Round, RoundError, RoundResult,
    discardable,
};
pub use rules::Rules;
pub use strategy::Strategy;
pub use susik;
pub use view::View;

use core::fmt;

/// A seat at the table, numbered from 0 in turn order
///
/// Seats carry no compass points: the next player (下家) is the next number,
/// wrapping around after the last.  The only table size modelled is
/// [`Seat::COUNT`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seat(u8);

impl Seat {
    /// Players at the table
    pub const COUNT: usize = 4;

    /// Seats in turn order
    pub const ALL: [Self; Self::COUNT] = {
        let mut all = [Self(0); Self::COUNT];
        let mut i = 0;
        while i < Self::COUNT {
            all[i] = Self(i as u8);
            i += 1;
        }
        all
    };

    /// The position in turn order, an index into [`Seat::ALL`]
    #[must_use]
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The seat `n` places along in turn order
    #[must_use]
    #[inline]
    pub const fn advance(self, n: usize) -> Self {
        Self::ALL[(self.index() + n) % Self::COUNT]
    }

    /// The next player, 下家
    #[must_use]
    #[inline]
    pub const fn next(self) -> Self {
        self.advance(1)
    }
}

/// The seat number counted from 1
impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index() + 1)
    }
}
