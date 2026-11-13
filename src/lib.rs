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

use core::fmt::{self, Write as _};

/// A seat at the table
///
/// Play runs counterclockwise, east, south, west, north, as in mahjong: the
/// next player (下家) sits to the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Seat {
    /// 東
    East,
    /// 南
    South,
    /// 西
    West,
    /// 北
    North,
}

impl Seat {
    /// Seats in turn order
    pub const ALL: [Self; 4] = [Self::East, Self::South, Self::West, Self::North];

    /// The seat `n` places along in turn order
    #[must_use]
    #[inline]
    pub const fn advance(self, n: usize) -> Self {
        Self::ALL[(self as usize + n) % 4]
    }

    /// The next player, 下家
    #[must_use]
    #[inline]
    pub const fn next(self) -> Self {
        self.advance(1)
    }

    /// Uppercase ASCII letter
    #[must_use]
    #[inline]
    pub const fn letter(self) -> char {
        match self {
            Self::East => 'E',
            Self::South => 'S',
            Self::West => 'W',
            Self::North => 'N',
        }
    }

    /// Chinese name
    #[must_use]
    #[inline]
    pub const fn name(self) -> char {
        match self {
            Self::East => '東',
            Self::South => '南',
            Self::West => '西',
            Self::North => '北',
        }
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.name())
    }
}
