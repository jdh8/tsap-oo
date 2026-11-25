//! What a discard gives the next seat, read off the public cards.
//!
//! [`ExpectBot`](crate::ExpectBot) values a discard by the hand it leaves.
//! The next seat may win on the card, or eat or pong it; what that costs is
//! measured here against what every seat can see of the card, and
//! [`ExpectBot::wary`](crate::ExpectBot::wary) takes it off the discard's
//! value.  The design and the measurement are in `doc/opponents.md`, batch 3.
//!
//! A concealed hand only shrinks: nothing drawn enters it.  So a card the
//! next seat let go it has still no use for, and one it threw from its hand
//! says that the hand held nothing to go with it, nor with the kinds that
//! make a run with it.

use crate::View;
use crate::expect::mates;
use susik::{Card, Hand};

/// What is public about a card a seat may discard, as the next seat has
/// treated it and its kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Evidence {
    /// The next seat discarded a copy from its hand
    Thrown,
    /// The next seat let a copy go: one lies on the discard pile, or
    /// [`View::safe`] holds the card
    Passed,
    /// Neither
    Fresh {
        /// The kinds the card makes a run with that the next seat discarded
        /// from its hand: 0, 1, or 2 for more
        thrown: usize,
        /// The copies of those kinds the seat has not seen, by twos: 0 for
        /// none or one, to 5 for ten or more
        unseen: usize,
    },
}

/// The cost in points of a [`Evidence::Thrown`] card
const THROWN: f64 = 0.002;

/// The cost in points of a [`Evidence::Passed`] card
const PASSED: f64 = 0.051;

/// The cost in points of a [`Evidence::Fresh`] card, by `thrown` and by
/// `unseen`
// ponytail: measured at a table of two at 10 胡 against `HeuristicBot`, by
// the web page's bot (`examples/giveaway.rs`, 4,500 rounds, seed 600).
// Another table, or an opponent with other habits, owes its own.
const FRESH: [[f64; 6]; 3] = [
    [0.896, 0.581, 0.753, 0.887, 0.937, 1.236],
    [0.357, 0.274, 0.297, 0.330, 0.569, 0.639],
    [0.109, 0.109, 0.109, 0.109, 0.109, 0.109],
];

/// What each discard of a seat gives the next one, by the public cards
#[derive(Debug, Clone, Copy)]
pub struct Giveaway {
    thrown: Hand,
    passed: Hand,
    unseen: Hand,
}

impl Giveaway {
    /// The public cards as `view`'s seat sees them, against the next seat
    #[must_use]
    pub fn new(view: &View<'_>) -> Self {
        let next = view.seat().next(view.rules().players);
        let pile: Hand = view.discards().iter().map(|&(_, card)| card).collect();
        let shown = crate::Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards());
        Self {
            thrown: view.thrown(next),
            // One of each: a card may lie four deep on the pile.
            passed: pile.distinct().fold(view.safe(next), |passed, (card, _)| {
                if passed.contains(card) {
                    passed
                } else {
                    passed.with(card)
                }
            }),
            unseen: shown.fold(Hand::FULL - view.hand() - pile, Hand::without),
        }
    }

    /// What is public about `card`
    #[must_use]
    pub fn evidence(&self, card: Card) -> Evidence {
        if self.thrown.contains(card) {
            return Evidence::Thrown;
        }
        if self.passed.contains(card) {
            return Evidence::Passed;
        }
        let run = || Card::ALL.into_iter().filter(move |&mate| mates(card, mate));
        let unseen: usize = run().map(|mate| usize::from(self.unseen.count(mate))).sum();
        Evidence::Fresh {
            thrown: run()
                .filter(|&mate| self.thrown.contains(mate))
                .count()
                .min(2),
            unseen: (unseen / 2).min(5),
        }
    }

    /// What discarding `card` gives the next seat, in points: the mean, over
    /// cards with the same [`Evidence`], of what the seat loses when the
    /// next one wins on it and of what the next one gains by claiming it
    #[must_use]
    pub fn cost(&self, card: Card) -> f64 {
        match self.evidence(card) {
            Evidence::Thrown => THROWN,
            Evidence::Passed => PASSED,
            Evidence::Fresh { thrown, unseen } => FRESH[thrown][unseen],
        }
    }
}
