//! What one seat may see.

use crate::round::{ClaimOptions, Event, Offer, Phase, Round};
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

    /// What this seat may do with the offer; empty unless the round waits
    /// on its claim
    #[must_use]
    pub fn claim_options(&self) -> ClaimOptions {
        self.round.claim_options(self.seat)
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

    /// The cards `seat` has shown it does not win on, one of each
    ///
    /// Winning is compulsory, so a card that came up and ended nothing wins
    /// for no seat it was offered to: every seat when it was drawn, every
    /// seat but its owner when it was discarded.  A seat's cards change only
    /// when it claims — nothing drawn enters a hand — so that holds until
    /// its next eat, pong or kong.  It is a proof, not a guess: such a card
    /// cannot deal in to `seat` now.
    #[must_use]
    pub fn safe(&self, seat: Seat) -> Hand {
        let mut safe = Hand::EMPTY;
        for &event in self.round.events() {
            let card = match event {
                Event::Eat { seat: s, .. }
                | Event::Pong { seat: s, .. }
                | Event::Kong { seat: s, .. }
                    if s == seat =>
                {
                    safe = Hand::EMPTY;
                    continue;
                }
                Event::Draw { card, .. } => card,
                Event::Discard { seat: s, card } if s != seat => card,
                _ => continue,
            };
            if !safe.contains(card) {
                safe.insert(card);
            }
        }
        safe
    }

    /// A round that looks the same from this seat, with the cards it cannot
    /// see — the other hands and the pile — dealt afresh
    ///
    /// The deal is uniform over the unseen cards.  It ignores what the play
    /// so far says about them: a seat that passed may be dealt the pair to
    /// pong with, and one that did not win a hand that would have.
    // ponytail: uniform; reject or weigh deals by the events if a sampling
    // bot turns out to be fooled by impossible ones.
    #[cfg(feature = "rand")]
    #[must_use]
    pub fn determinize(&self, rng: &mut (impl rand::Rng + ?Sized)) -> Round {
        self.round.determinize(self.seat, rng)
    }

    /// The round with every hand as it is and only the pile shuffled: what
    /// a seat that saw every hand would sample
    ///
    /// A cheat, for the Peeker in `ab/`, which measures the most that reading
    /// the other hands could be worth.  It is hidden and behind the `peek`
    /// feature so that no bot that sits at a table can reach it.
    #[cfg(feature = "peek")]
    #[doc(hidden)]
    #[must_use]
    pub fn peek(&self, rng: &mut (impl rand::Rng + ?Sized)) -> Round {
        use rand::seq::SliceRandom as _;
        let mut round = self.round.clone();
        round.pile_mut().shuffle(rng);
        round
    }
}
