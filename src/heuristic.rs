//! A greedy bot.

use crate::round::{Claim, ClaimOptions};
use crate::rules::Rules;
use crate::strategy::Strategy;
use crate::view::View;
use susik::{Card, Hand, Meld, Solver, Values};

/// A greedy bot: grades a hand by its best layout with loose cards at a
/// cost, and takes whichever claim or discard grades highest
///
/// It is deterministic and looks one decision ahead.  It never reasons
/// about other seats or the pile.
#[derive(Debug, Clone)]
pub struct HeuristicBot {
    solver: Solver,
    exposed: Values,
}

impl HeuristicBot {
    /// A bot for `rules`
    #[must_use]
    pub fn new(rules: &Rules) -> Self {
        Self {
            solver: Solver::new(rules.concealed, Some(1)),
            exposed: rules.exposed,
        }
    }

    fn value(&mut self, hand: Hand) -> i32 {
        self.solver
            .best(hand)
            .expect("layouts with loose cards always exist")
    }

    /// The hand's value after its best discard other than `locked`
    fn after_discard(&mut self, hand: Hand, locked: Card) -> i32 {
        hand.distinct()
            .filter(|&(card, _)| card != locked)
            .map(|(card, _)| self.value(hand.without(card)))
            .max()
            .unwrap_or(0)
    }
}

impl Strategy for HeuristicBot {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        self.solver.clear();
        let hand = view.hand();
        let card = view.offer().expect("claims answer an offer").card;
        let mut best = (self.value(hand), Claim::Pass);
        let mut consider = |bot: &mut Self, meld: Meld, claim: Claim| {
            let gain = i32::try_from(bot.exposed.score(&meld)).unwrap_or(i32::MAX / 4);
            let score = gain + bot.after_discard(hand - meld.cards().without(card), card);
            if score > best.0 {
                best = (score, claim);
            }
        };
        if options.pong {
            consider(self, Meld::Triplet(card), Claim::Pong);
        }
        for &meld in &options.eats {
            consider(self, meld, Claim::Eat(meld));
        }
        best.1
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        self.solver.clear();
        let hand = view.hand();
        let locked = view.locked();
        hand.distinct()
            .map(|(card, _)| card)
            .filter(|&card| Some(card) != locked)
            .max_by_key(|&card| self.value(hand.without(card)))
            .expect("a hand to discard from")
    }

    fn name(&self) -> &str {
        "greedy"
    }
}
