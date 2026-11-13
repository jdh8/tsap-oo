//! A greedy bot.

use crate::round::{Claim, ClaimOptions, discardable};
use crate::rules::Rules;
use crate::strategy::Strategy;
use crate::view::View;
use susik::{Card, Hand, Meld, Solver, Values};

/// What a loose card, or a 胡 short of the threshold, costs a hand.
const WEIGHT: i32 = 4;

/// Loose-card costs of the two solvers: the first lays a hand out for 胡,
/// the second for the fewest loose cards.
// ponytail: two points of the 胡/loose trade-off, found by arena play; a
// solver returning the whole frontier would be exact.
const COSTS: [u32; 2] = [1, 16];

/// A greedy bot: grades a hand by its 胡 less a cost for every loose card
/// and every 胡 short of the threshold, and takes whichever claim or discard
/// grades highest
///
/// A hand that already holds its ten races to meld its loose cards; one
/// that does not keeps the melds that can still reach ten.  It is
/// deterministic and looks one decision ahead.  It never reasons about
/// other seats or the pile.
#[derive(Debug, Clone)]
pub struct HeuristicBot {
    solvers: [Solver; 2],
    exposed: Values,
    threshold: u32,
}

impl HeuristicBot {
    /// A bot for `rules`
    #[must_use]
    pub fn new(rules: &Rules) -> Self {
        Self {
            solvers: COSTS.map(|cost| Solver::new(rules.concealed, Some(cost))),
            exposed: rules.exposed,
            threshold: rules.threshold,
        }
    }

    /// Forget the memos and count the 胡 the seat still needs from its hand
    fn start(&mut self, view: &View<'_>) -> i32 {
        for solver in &mut self.solvers {
            solver.clear();
        }
        let have: u32 = view
            .exposed(view.seat())
            .iter()
            .map(|meld| self.exposed.score(meld))
            .sum();
        to_i32(self.threshold) - to_i32(have)
    }

    /// Grade `hand` when its melds must be worth `need` to win
    fn value(&mut self, hand: Hand, need: i32) -> i32 {
        self.solvers
            .iter_mut()
            .zip(COSTS)
            .map(|(solver, cost)| {
                let layout = solver
                    .layout(hand)
                    .expect("layouts with loose cards always exist");
                let loose = to_i32(layout.loose.len());
                let hu = layout.score + to_i32(cost) * loose;
                hu - WEIGHT * (loose + (need - hu).max(0))
            })
            .max()
            .expect("two solvers")
    }

    /// The hand's value after its best discard other than `locked`
    fn after_discard(&mut self, hand: Hand, locked: Card, need: i32) -> i32 {
        discardable(hand, Some(locked))
            .distinct()
            .map(|(card, _)| self.value(hand.without(card), need))
            .max()
            .expect("claims leave a card to discard")
    }
}

/// Card counts and 胡 are far below `i32::MAX`.
fn to_i32(n: impl TryInto<i32>) -> i32 {
    n.try_into().unwrap_or(i32::MAX / 4)
}

impl Strategy for HeuristicBot {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let need = self.start(view);
        let hand = view.hand();
        let card = view.offer().expect("claims answer an offer").card;
        let mut best = (self.value(hand, need), Claim::Pass);
        let mut consider = |bot: &mut Self, meld: Meld, claim: Claim| {
            let gain = to_i32(bot.exposed.score(&meld));
            let score =
                gain + bot.after_discard(hand - meld.cards().without(card), card, need - gain);
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
        let need = self.start(view);
        let hand = view.hand();
        discardable(hand, view.locked())
            .distinct()
            .map(|(card, _)| card)
            .max_by_key(|&card| self.value(hand.without(card), need))
            .expect("a hand to discard from")
    }

    fn name(&self) -> &str {
        "greedy"
    }
}
