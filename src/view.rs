//! What one seat may see.

use crate::round::{Offer, Phase, Round};
use crate::{Rules, Seat};
use susik::{Card, Hand, Meld};

/// A seat's information-hygienic window onto a [`Round`]
///
/// Other hands and the pile order are unreachable through it; everything
/// public — exposed melds, discards, the offer, and card counts — is.
#[derive(Debug, Clone, Copy)]
pub struct View<'a> {
    round: &'a Round,
    seat: Seat,
}

impl<'a> View<'a> {
    /// `seat`'s view of `round`
    #[must_use]
    pub const fn new(round: &'a Round, seat: Seat) -> Self {
        Self { round, seat }
    }

    /// Whose view this is
    #[must_use]
    #[inline]
    pub const fn seat(&self) -> Seat {
        self.seat
    }

    /// The rules in force
    #[must_use]
    #[inline]
    pub const fn rules(&self) -> &'a Rules {
        self.round.rules()
    }

    /// Who dealt
    #[must_use]
    #[inline]
    pub const fn dealer(&self) -> Seat {
        self.round.dealer()
    }

    /// Whose decision is pending
    #[must_use]
    #[inline]
    pub const fn phase(&self) -> Phase {
        self.round.phase()
    }

    /// The public card being resolved
    #[must_use]
    #[inline]
    pub const fn offer(&self) -> Option<Offer> {
        self.round.offer()
    }

    /// This seat's concealed cards
    #[must_use]
    #[inline]
    pub const fn hand(&self) -> Hand {
        self.round.hand(self.seat)
    }

    /// The card this seat just claimed and may not discard
    #[must_use]
    #[inline]
    pub const fn locked(&self) -> Option<Card> {
        self.round.locked()
    }

    /// Any seat's exposed melds
    #[must_use]
    #[inline]
    pub fn exposed(&self, seat: Seat) -> &'a [Meld] {
        self.round.exposed(seat)
    }

    /// How many concealed cards any seat holds
    #[must_use]
    #[inline]
    pub const fn concealed_len(&self, seat: Seat) -> usize {
        self.round.hand(seat).len()
    }

    /// Unclaimed public cards, oldest first
    #[must_use]
    #[inline]
    pub fn discards(&self) -> &'a [(Seat, Card)] {
        self.round.discards()
    }

    /// Cards left in the pile, dead cards included
    #[must_use]
    #[inline]
    pub const fn pile_len(&self) -> usize {
        self.round.pile_len()
    }
}
