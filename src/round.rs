//! One deal, from the cards going out to a win or an exhausted pile.
//!
//! Every card that leaves a hand or the pile is public — a discard, and a
//! draw, which is turned face up as it is taken — and every public card is
//! an [`Offer`] that the table resolves in a fixed order:
//!
//! 1. A win (胡) outranks everything.  Whoever can lay out all 21 cards for
//!    the threshold takes the card at once; a drawer's own win comes first,
//!    then the seats in turn order.  Winning is compulsory (不能過水).
//! 2. The fourth card of a concealed triplet must be konged (槓); the kong
//!    is applied for the seat without asking.
//! 3. Then the seats are asked in turn order, the drawer first: anyone
//!    holding a pair may pong (碰); the drawer and the next player may
//!    eat (吃) the card into a 將士象, 車馬包, a colour set of soldiers, or
//!    a pair.  A pong beats an eat, and among equal claims the earlier seat
//!    wins.  A concealed triplet is never split for a claim, and the card
//!    just claimed cannot be discarded in the same turn.
//! 4. Unclaimed, the card joins the discard pile and the next player
//!    draws.
//!
//! The state machine is runtime-checked: actions return [`RoundError`]
//! instead of panicking, and [`Phase`] says whose decision is pending.  The
//! round shows every hand and the pile through its accessors; bots and
//! user interfaces keep their own information hygiene (see [`View`]).
//!
//! [`View`]: crate::View

use crate::{Rules, Seat};
use std::collections::VecDeque;
use susik::{Card, Hand, Meld, Solver, completions};
use thiserror::Error;

/// Whose decision the round waits for
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// This seat may claim the offered card or pass
    Claim(Seat),
    /// This seat must discard
    Discard(Seat),
    /// The round is over; see [`Round::result`]
    Finished,
}

/// A public card being resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Offer {
    /// The card
    pub card: Card,
    /// Who drew or discarded it
    pub source: Seat,
    /// Drawn from the pile (face up) rather than discarded from a hand
    pub drawn: bool,
}

/// A seat's answer to an offer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Claim {
    /// Decline
    Pass,
    /// Pong with a pair from the hand
    Pong,
    /// Eat into this meld, one of [`ClaimOptions::eats`]
    Eat(Meld),
}

/// What a seat may do with the offered card
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimOptions {
    /// Whether a pong is possible
    pub pong: bool,
    /// The melds the card may be eaten into
    pub eats: Vec<Meld>,
}

impl ClaimOptions {
    /// Whether nothing but passing is possible
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.pong && self.eats.is_empty()
    }
}

/// Something that happened, in order; see [`Round::events`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    /// A card was drawn face up from the pile
    Draw {
        /// Who drew
        seat: Seat,
        /// The card
        card: Card,
    },
    /// A card was discarded, from the hand or as a rejected draw
    Discard {
        /// Who discarded
        seat: Seat,
        /// The card
        card: Card,
    },
    /// A card was eaten
    Eat {
        /// Who ate
        seat: Seat,
        /// The meld formed
        meld: Meld,
        /// The card eaten
        card: Card,
    },
    /// A card was ponged
    Pong {
        /// Who ponged
        seat: Seat,
        /// The card
        card: Card,
    },
    /// A card was konged
    Kong {
        /// Who konged
        seat: Seat,
        /// The card
        card: Card,
    },
    /// A seat won
    Win {
        /// Who won
        seat: Seat,
        /// With how many 胡
        hu: u32,
    },
    /// The pile ran down to the dead cards
    Exhausted,
}

/// How a round ended
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RoundResult {
    /// The winner, or `None` for an exhausted pile
    pub winner: Option<Seat>,
    /// The winner's 胡
    pub hu: u32,
    /// The winner's full layout: exposed melds, then the meld of the winning
    /// card, then the concealed melds
    pub layout: Vec<Meld>,
    /// The card that completed the hand; `None` when the dealer won on the
    /// deal (天胡)
    pub winning_card: Option<Card>,
    /// Who put the winning card on the table, if not the winner: the
    /// discarder, or another seat's draw
    pub from: Option<Seat>,
}

/// An action the round rejected; the round is unchanged
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RoundError {
    /// The seat's decision is not what the round waits for
    #[error("not this seat's turn for that action")]
    WrongPhase,
    /// The claim is not among the seat's options
    #[error("that claim is not available")]
    NotAllowed,
    /// The card is not in the hand
    #[error("{0} is not in the hand")]
    NotInHand(Card),
    /// The card was just claimed and cannot be discarded this turn
    #[error("{0} was just claimed and cannot be discarded now")]
    Locked(Card),
}

/// The pile handed to [`Round::from_pile`] was not the full deck
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
#[error("a deal needs all 112 cards")]
pub struct DealError;

/// One deal of 十胡
#[derive(Debug, Clone)]
pub struct Round {
    rules: Rules,
    dealer: Seat,
    hands: [Hand; 4],
    exposed: [Vec<Meld>; 4],
    pile: Vec<Card>,
    discards: Vec<(Seat, Card)>,
    events: Vec<Event>,
    phase: Phase,
    offer: Option<Offer>,
    queue: VecDeque<Seat>,
    eat: Option<(Seat, Meld)>,
    locked: Option<Card>,
    result: Option<RoundResult>,
}

impl Round {
    /// Deal from a pile holding the whole deck in draw order
    ///
    /// Cards come off the end: twenty to the dealer, twenty to each seat
    /// after, one more to the dealer, who then discards first — unless the
    /// deal already lays out for the threshold (天胡), in which case the
    /// round is finished at once.
    ///
    /// # Errors
    ///
    /// When `pile` is not a permutation of the 112-card deck.
    pub fn from_pile(rules: Rules, dealer: Seat, mut pile: Vec<Card>) -> Result<Self, DealError> {
        if pile.len() != 112 || pile.iter().copied().collect::<Hand>() != Hand::FULL {
            return Err(DealError);
        }
        let mut hands = [Hand::EMPTY; 4];
        for i in 0..4 {
            let hand = &mut hands[dealer.advance(i) as usize];
            for _ in 0..20 {
                hand.insert(pile.pop().expect("112 cards suffice"));
            }
        }
        hands[dealer as usize].insert(pile.pop().expect("112 cards suffice"));

        let mut round = Self {
            rules,
            dealer,
            hands,
            exposed: Default::default(),
            pile,
            discards: Vec::new(),
            events: Vec::new(),
            phase: Phase::Discard(dealer),
            offer: None,
            queue: VecDeque::new(),
            eat: None,
            locked: None,
            result: None,
        };
        if let Some(layout) = Solver::new(rules.concealed, None).layout(hands[dealer as usize])
            && let Ok(hu) = u32::try_from(layout.score)
            && hu >= rules.threshold
        {
            round.finish(Some(dealer), hu, layout.melds, None, None);
        }
        Ok(round)
    }

    /// Deal from a shuffled deck
    #[cfg(feature = "rand")]
    #[must_use]
    pub fn deal(rules: Rules, dealer: Seat, rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        Self::from_pile(rules, dealer, susik::shuffle(rng))
            .expect("a shuffled deck is the full deck")
    }

    /// The rules in force
    #[must_use]
    #[inline]
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// Who dealt
    #[must_use]
    #[inline]
    pub const fn dealer(&self) -> Seat {
        self.dealer
    }

    /// Whose decision is pending
    #[must_use]
    #[inline]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// The public card being resolved, during a [`Phase::Claim`]
    #[must_use]
    #[inline]
    pub const fn offer(&self) -> Option<Offer> {
        self.offer
    }

    /// A seat's concealed cards
    #[must_use]
    #[inline]
    pub const fn hand(&self, seat: Seat) -> Hand {
        self.hands[seat as usize]
    }

    /// A seat's exposed melds, in the order they were formed
    #[must_use]
    #[inline]
    pub fn exposed(&self, seat: Seat) -> &[Meld] {
        &self.exposed[seat as usize]
    }

    /// Unclaimed public cards, oldest first, with who put each down
    #[must_use]
    #[inline]
    pub fn discards(&self) -> &[(Seat, Card)] {
        &self.discards
    }

    /// Cards left in the pile, dead cards included
    #[must_use]
    #[inline]
    pub const fn pile_len(&self) -> usize {
        self.pile.len()
    }

    /// Everything that has happened, oldest first
    #[must_use]
    #[inline]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// The card just claimed by the seat about to discard, which they may
    /// not discard
    #[must_use]
    #[inline]
    pub const fn locked(&self) -> Option<Card> {
        self.locked
    }

    /// How the round ended, once [`Phase::Finished`]
    #[must_use]
    #[inline]
    pub const fn result(&self) -> Option<&RoundResult> {
        self.result.as_ref()
    }

    /// What `seat` may do now; empty unless the round waits on their claim
    #[must_use]
    pub fn claim_options(&self, seat: Seat) -> ClaimOptions {
        match (self.phase, self.offer) {
            (Phase::Claim(s), Some(offer)) if s == seat => self.options_for(offer, seat),
            _ => ClaimOptions::default(),
        }
    }

    /// Answer the offer
    ///
    /// # Errors
    ///
    /// [`RoundError::WrongPhase`] unless the round waits on this seat's
    /// claim; [`RoundError::NotAllowed`] if the claim is not among
    /// [`Round::claim_options`].
    pub fn claim(&mut self, seat: Seat, claim: Claim) -> Result<(), RoundError> {
        if self.phase != Phase::Claim(seat) {
            return Err(RoundError::WrongPhase);
        }
        let offer = self.offer.expect("a claim phase resolves an offer");
        let options = self.options_for(offer, seat);
        match claim {
            Claim::Pass => {
                self.queue.pop_front();
            }
            Claim::Pong => {
                if !options.pong {
                    return Err(RoundError::NotAllowed);
                }
                self.take(seat, Meld::Triplet(offer.card), offer.card);
                self.events.push(Event::Pong {
                    seat,
                    card: offer.card,
                });
                return Ok(());
            }
            Claim::Eat(meld) => {
                if !options.eats.contains(&meld) {
                    return Err(RoundError::NotAllowed);
                }
                self.queue.pop_front();
                // The queue runs in priority order, so the first eat staked
                // is the one that stands if no pong beats it.
                if self.eat.is_none() {
                    self.eat = Some((seat, meld));
                }
            }
        }
        self.advance();
        Ok(())
    }

    /// Discard a card from the hand
    ///
    /// # Errors
    ///
    /// [`RoundError::WrongPhase`] unless the round waits on this seat's
    /// discard; [`RoundError::Locked`] for the card just claimed;
    /// [`RoundError::NotInHand`] for a card the seat does not hold.
    pub fn discard(&mut self, seat: Seat, card: Card) -> Result<(), RoundError> {
        if self.phase != Phase::Discard(seat) {
            return Err(RoundError::WrongPhase);
        }
        if self.locked == Some(card) {
            return Err(RoundError::Locked(card));
        }
        if !self.hands[seat as usize].remove(card) {
            return Err(RoundError::NotInHand(card));
        }
        self.locked = None;
        self.events.push(Event::Discard { seat, card });
        self.resolve(card, seat, false);
        Ok(())
    }

    /// The seats a public card is offered to, in priority order
    fn order(offer: Offer) -> impl Iterator<Item = Seat> {
        let (start, n) = if offer.drawn {
            (offer.source, 4)
        } else {
            (offer.source.next(), 3)
        };
        (0..n).map(move |i| start.advance(i))
    }

    fn options_for(&self, offer: Offer, seat: Seat) -> ClaimOptions {
        let hand = self.hands[seat as usize];
        let card = offer.card;
        let n = hand.count(card);
        let eater = seat == offer.source.next() || (offer.drawn && seat == offer.source);
        let eats = if eater {
            completions(hand, card)
                .into_iter()
                .filter(|meld| match meld {
                    // Holding a pair, one pongs instead.
                    Meld::Pair(_) => n == 1,
                    Meld::Court(_) | Meld::Field(_) | Meld::Soldiers(_) => true,
                    Meld::Single(_) | Meld::Triplet(_) | Meld::Quad(_) => false,
                })
                // A concealed triplet is locked: none of its cards may be
                // split off into a claim.
                .filter(|meld| {
                    meld.cards()
                        .without(card)
                        .distinct()
                        .all(|(c, _)| hand.count(c) < 3)
                })
                .collect()
        } else {
            Vec::new()
        };
        ClaimOptions { pong: n == 2, eats }
    }

    /// The best winning layout of `seat`'s cards plus `card`, if it reaches
    /// the threshold
    fn win(&self, seat: Seat, card: Card) -> Option<(u32, Vec<Meld>)> {
        let hand = self.hands[seat as usize];
        let exposed = &self.exposed[seat as usize];
        let base: u32 = exposed.iter().map(|m| self.rules.exposed.score(m)).sum();
        let mut solver = Solver::new(self.rules.concealed, None);
        let mut best: Option<(u32, Vec<Meld>)> = None;
        for meld in completions(hand, card) {
            let Some(layout) = solver.layout(hand - meld.cards().without(card)) else {
                continue;
            };
            let hu =
                base + self.rules.exposed.score(&meld) + u32::try_from(layout.score).unwrap_or(0);
            if hu >= self.rules.threshold && best.as_ref().is_none_or(|(b, _)| hu > *b) {
                let mut melds = exposed.clone();
                melds.push(meld);
                melds.extend(layout.melds);
                best = Some((hu, melds));
            }
        }
        best
    }

    /// Put a public card on the table and resolve who gets it.
    fn resolve(&mut self, card: Card, source: Seat, drawn: bool) {
        let offer = Offer {
            card,
            source,
            drawn,
        };
        self.offer = Some(offer);
        for seat in Self::order(offer) {
            if let Some((hu, layout)) = self.win(seat, card) {
                let from = (!(drawn && seat == source)).then_some(source);
                self.finish(Some(seat), hu, layout, Some(card), from);
                return;
            }
        }
        for seat in Self::order(offer) {
            if self.hands[seat as usize].count(card) == 3 {
                self.take(seat, Meld::Quad(card), card);
                self.events.push(Event::Kong { seat, card });
                return;
            }
        }
        self.queue = Self::order(offer)
            .filter(|&seat| !self.options_for(offer, seat).is_empty())
            .collect();
        self.eat = None;
        self.advance();
    }

    /// Move the rest of `meld` out of `seat`'s hand, expose it with the
    /// claimed `card`, and hand `seat` the discard.
    fn take(&mut self, seat: Seat, meld: Meld, card: Card) {
        self.hands[seat as usize] -= meld.cards().without(card);
        self.exposed[seat as usize].push(meld);
        self.offer = None;
        self.queue.clear();
        self.eat = None;
        self.locked = Some(card);
        self.phase = Phase::Discard(seat);
    }

    /// Ask the next claimant, or settle the offer when none remain.
    fn advance(&mut self) {
        let offer = self.offer.expect("an offer is being resolved");
        while let Some(&seat) = self.queue.front() {
            // Once an eat is staked only a pong can beat it.
            if self.eat.is_some() && !self.options_for(offer, seat).pong {
                self.queue.pop_front();
                continue;
            }
            self.phase = Phase::Claim(seat);
            return;
        }
        if let Some((seat, meld)) = self.eat.take() {
            self.take(seat, meld, offer.card);
            self.events.push(Event::Eat {
                seat,
                meld,
                card: offer.card,
            });
            return;
        }
        // Unclaimed: the card settles on the discard pile as its source's
        // discard, and the next player draws.
        self.offer = None;
        if offer.drawn {
            self.events.push(Event::Discard {
                seat: offer.source,
                card: offer.card,
            });
        }
        self.discards.push((offer.source, offer.card));
        self.draw(offer.source.next());
    }

    fn draw(&mut self, seat: Seat) {
        if self.pile.len() <= self.rules.dead_tail {
            self.finish(None, 0, Vec::new(), None, None);
            return;
        }
        let card = self.pile.pop().expect("more than the dead tail remains");
        self.events.push(Event::Draw { seat, card });
        self.resolve(card, seat, true);
    }

    fn finish(
        &mut self,
        winner: Option<Seat>,
        hu: u32,
        layout: Vec<Meld>,
        winning_card: Option<Card>,
        from: Option<Seat>,
    ) {
        self.events.push(match winner {
            Some(seat) => Event::Win { seat, hu },
            None => Event::Exhausted,
        });
        self.result = Some(RoundResult {
            winner,
            hu,
            layout,
            winning_card,
            from,
        });
        self.offer = None;
        self.queue.clear();
        self.eat = None;
        self.locked = None;
        self.phase = Phase::Finished;
    }
}
