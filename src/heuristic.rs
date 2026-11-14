//! A greedy bot.

use crate::Seat;
use crate::round::{Claim, ClaimOptions, discardable};
use crate::rules::Rules;
use crate::strategy::Strategy;
use crate::view::View;
use susik::{Card, Hand, Layout, Meld, Rank, Solver, Values};

/// What a loose card, or a 胡 short of the threshold, costs a hand.
const WEIGHT: i32 = 4;

/// Loose-card costs of the two solvers: the first lays a hand out for 胡,
/// the second for the fewest loose cards.
// ponytail: two points of the 胡/loose trade-off, found by arena play; a
// solver returning the whole frontier would be exact.
const COSTS: [u32; 2] = [1, 16];

/// What a 胡 is worth in the units of [`prospects`]: the loose cards only
/// settle what the melds leave open.
// ponytail: one knob found by arena play, like `COSTS` and `OWN`.
const SCALE: i32 = 32;

/// What each unseen copy of a loose card adds to its grade.
const OWN: i32 = 2;

/// Copies of a card in the deck, and the most finishing cards a 搭子 is
/// graded on.
const COPIES: i32 = 4;

/// What a dead hand costs: more than any hand is worth, so one is kept only
/// when every choice is dead.
const DEAD: i32 = 1000;

/// A greedy bot: grades a hand by its 胡 less a cost for every loose card
/// and every 胡 short of the threshold, and takes whichever claim or discard
/// grades highest
///
/// A hand that already holds its ten races to meld its loose cards; one
/// that does not keeps the melds that can still reach ten, counting the 胡
/// of each 搭子, two cards one short of a meld, that a card not yet seen
/// could finish.  Among loose cards it keeps the 搭子, the dearer the meld
/// and the more cards left to finish it the longer, and sheds the cards that
/// go with nothing, starting with the ones another seat would most like to
/// pick up late.  It counts the cards it has seen — its own, the melds on
/// the table, the discards — but never guesses at another seat's hand.  It
/// is deterministic and looks one decision ahead.
#[derive(Debug, Clone)]
pub struct HeuristicBot {
    solvers: [Solver; 2],
    exposed: Values,
    threshold: u32,
    /// Lays a hand out for the 胡 its melds are worth beyond a general a
    /// card
    surplus: Solver,
    /// 胡 of a general
    general: i32,
    /// 胡 of the meld a loose card of each [`family`] would be eaten into
    eaten: [i32; 3],
    /// The cards the seat has not seen: in other hands or the pile
    live: Hand,
}

impl HeuristicBot {
    /// A bot for `rules`
    #[must_use]
    pub fn new(rules: &Rules) -> Self {
        Self {
            solvers: COSTS.map(|cost| Solver::new(rules.concealed, Some(cost)).locked()),
            exposed: rules.exposed,
            threshold: rules.threshold,
            surplus: Solver::new(surplus(rules.concealed), Some(0)).locked(),
            general: to_i32(rules.concealed.single),
            eaten: [
                rules.exposed.court,
                rules.exposed.field,
                rules.exposed.three_soldiers,
            ]
            .map(to_i32),
            live: Hand::EMPTY,
        }
    }

    /// Forget the memos and count the 胡 the seat still needs from its hand
    fn start(&mut self, view: &View<'_>) -> i32 {
        for solver in self.solvers.iter_mut().chain([&mut self.surplus]) {
            solver.clear();
        }
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card))
            .chain(view.offer().map(|offer| offer.card));
        self.live = shown.fold(Hand::FULL - view.hand(), Hand::without);
        let have: u32 = view
            .exposed(view.seat())
            .iter()
            .map(|meld| self.exposed.score(meld))
            .sum();
        to_i32(self.threshold) - to_i32(have)
    }

    /// Whether `hand` cannot be worth `need` even if a general came for
    /// every card it gave up, and one more to win on
    ///
    /// A general is the only card that comes unasked, and it pushes another
    /// card out.  A hand that falls short at that rate can be walked, one
    /// drawn general at a time, down to nothing but generals and then to a
    /// 相公; a hand that does not can always answer a general by discarding
    /// and stay so, or win.  Only the melds worth more than a general a card
    /// — a concealed quad, soldiers of four colours — are worth keeping
    /// through that, which is what `surplus` counts.
    fn dead(&mut self, hand: Hand, need: i32) -> bool {
        let surplus = self
            .surplus
            .layout(hand)
            .expect("layouts with loose cards always exist");
        (to_i32(hand.len()) + 1) * self.general + surplus.score < need
    }

    /// Grade `hand` when its melds must be worth `need` to win
    fn value(&mut self, hand: Hand, need: i32) -> i32 {
        if self.dead(hand, need) {
            return -SCALE * DEAD;
        }
        self.solvers
            .iter_mut()
            .zip(COSTS)
            .map(|(solver, cost)| {
                let layout = solver
                    .layout(hand)
                    .expect("layouts with loose cards always exist");
                let loose = to_i32(layout.loose.len());
                let hu = layout.score + to_i32(cost) * loose;
                let (grade, hope) = prospects(hand, &layout, self.live, self.eaten, self.general);
                // In half-胡, like `hope`.
                let short = (2 * (need - hu) - hope).max(0);
                SCALE * (2 * (hu - WEIGHT * loose) - WEIGHT * short) / 2 + grade
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

/// 將士象, 車馬包 or soldiers: the cards that meld together
const fn family(rank: Rank) -> usize {
    match rank {
        Rank::General | Rank::Advisor | Rank::Elephant => 0,
        Rank::Chariot | Rank::Horse | Rank::Cannon => 1,
        Rank::Soldier => 2,
    }
}

/// What the loose cards of `layout` promise: a grade to tell hands of the
/// same melds apart, and the half-胡 they may yet bring
///
/// A loose card is one of a 搭子 when the hand holds a second card of its
/// meld and a third is `live`, not yet seen.  It is then graded at the 胡
/// of the meld it would be eaten into (`eaten`, by [`family`]) for each live
/// card that would finish it, up to [`COPIES`], and promises half that 胡:
/// the other half is its partner's.  So a 搭子 is kept, soldiers before
/// 將士象 before 車馬包, and the one with more cards left to come before the
/// one with fewer.
///
/// A card that goes with nothing is graded that far below nothing, so the
/// dearest goes first, before it can finish another seat's hand.  Either
/// way a card is worth [`OWN`] more for each live copy of itself, which
/// would pair it.
///
/// The second card may be a lone general or one of a pair — 車車 馬 still
/// eats a 包 — but not one of three or four alike, which are locked and
/// never eat; nor does a card finish the meld when the hand holds three of
/// it, since the fourth can only come as a kong.  士象 without their general
/// are the poorest 搭子: only the seat's own draw brings a general, so half
/// the live ones count, and a general is worth `general` alone, so the 士象
/// add only the rest of the meld.
fn prospects(hand: Hand, layout: &Layout, live: Hand, eaten: [i32; 3], general: i32) -> (i32, i32) {
    let free = layout
        .melds
        .iter()
        .filter(|meld| meld.size() >= 3)
        .fold(hand, |rest, meld| rest - meld.cards());
    layout
        .loose
        .iter()
        .map(|card| {
            let kin = family(card.rank);
            let mate = |other: Card| {
                other != card
                    && family(other.rank) == kin
                    && (card.rank == Rank::Soldier || other.color == card.color)
                    && hand.count(other) < 3
            };
            let finishers: i32 = Hand::FULL
                .distinct()
                .filter(|&(third, _)| {
                    mate(third)
                        && free
                            .distinct()
                            .any(|(other, _)| other != third && mate(other))
                })
                .map(|(third, _)| i32::from(live.count(third)))
                .sum();
            let own = OWN * i32::from(live.count(card));
            if finishers == 0 {
                (own - COPIES * eaten[kin], 0)
            } else if kin == 0 && !hand.contains(Card::new(card.color, Rank::General)) {
                let worth = eaten[kin] - general;
                (own + ((finishers + 1) / 2).min(COPIES) * worth, worth)
            } else {
                (own + finishers.min(COPIES) * eaten[kin], eaten[kin])
            }
        })
        .fold((0, 0), |sum, (grade, hope)| (sum.0 + grade, sum.1 + hope))
}

/// What each meld is worth beyond a general for every card in it
fn surplus(values: Values) -> Values {
    let over = |value: u32, cards: u32| value.saturating_sub(cards * values.single);
    Values {
        single: 0,
        pair: over(values.pair, 2),
        triplet: over(values.triplet, 3),
        quad: over(values.quad, 4),
        court: over(values.court, 3),
        field: over(values.field, 3),
        three_soldiers: over(values.three_soldiers, 3),
        four_soldiers: over(values.four_soldiers, 4),
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
        // A drawn general cannot be passed: any eat beats the placeholder.
        let mut best = if options.forced() {
            (i32::MIN, Claim::Pass)
        } else {
            (self.value(hand, need), Claim::Pass)
        };
        let mut consider = |bot: &mut Self, meld: Meld, claim: Claim| {
            let gain = to_i32(bot.exposed.score(&meld));
            let score = SCALE * gain
                + bot.after_discard(hand - meld.cards().without(card), card, need - gain);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_hands() {
        let mut bot = HeuristicBot::new(&Rules::new());
        // (hand, the most it is sure of)
        for (hand, most) in [
            ("帥.帥..", 3),        // two generals and a third to win on
            ("帥仕...", 3),        // the advisor goes when a general comes
            ("帥仕相...", 4),      // three more generals beat 將士象 and one
            ("帥兵兵兵...", 5),    // a triplet is worth no more than generals
            ("帥兵.兵.卒.卒", 7),  // soldiers of four colours are
            ("帥兵兵兵兵...", 10), // and a concealed quad
        ] {
            let hand: Hand = hand.parse().unwrap();
            assert!(!bot.dead(hand, most), "{hand} can make {most}");
            assert!(bot.dead(hand, most + 1), "{hand} cannot make {}", most + 1);
        }
    }

    #[test]
    fn loose_cards() {
        let rules = Rules::new();
        let bot = HeuristicBot::new(&rules);
        let mut solver = Solver::new(rules.concealed, Some(1)).locked();
        let mut prospects = |hand: &str, seen: &str| {
            let hand: Hand = hand.parse().unwrap();
            let live = Hand::FULL - hand - seen.parse().unwrap();
            let layout = solver.layout(hand).unwrap();
            super::prospects(hand, &layout, live, bot.eaten, bot.general)
        };
        // (hand, its grade, the half-胡 it hopes for) with nothing else seen
        for (hand, grade, hope) in [
            ("俥傌...", 20, 2),        // a 搭子: four 炮 and three copies each
            ("俥俥傌...", 10, 1),      // one of a pair still eats
            ("俥俥俥傌...", 2, 0),     // three alike are locked
            ("俥俥傌炮炮炮...", 2, 0), // and so is the 炮 this would eat
            ("兵兵..卒.", 18, 3),      // soldiers work the same way
            ("兵兵兵..卒.", -6, 0),
            ("帥仕...", 14, 2), // a general is company
            ("仕相...", 16, 2), // 士象 beg for theirs, below 車馬
            ("仕...", -2, 0),   // alone, the dearer card goes first
            ("傌...", 2, 0),
            ("兵...", -6, 0),
        ] {
            assert_eq!(prospects(hand, "..."), (grade, hope), "{hand}");
        }
        // A 搭子 fades with its finishing cards and dies with the last.
        assert_eq!(prospects("俥傌...", "炮炮..."), (16, 2));
        assert_eq!(prospects("俥傌...", "炮炮炮炮..."), (4, 0));
        // A copy seen is one fewer to pair with.
        assert_eq!(prospects("傌...", "傌傌..."), (-2, 0));
    }
}
