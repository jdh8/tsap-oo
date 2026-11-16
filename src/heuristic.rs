//! A greedy bot.

use crate::Seat;
use crate::round::{Claim, ClaimOptions, discardable, options};
use crate::rules::Rules;
use crate::strategy::Strategy;
use crate::view::View;
use susik::{Card, Hand, Layout, Meld, Rank, Solver, Values, locked_completions};

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

/// What each unseen card a hand would win on adds to its grade: more than
/// any 搭子 is graded per finishing card, since a win is taken from any seat.
// ponytail: one knob, set before its A/B and not tuned.
const WAIT: i32 = 8;

/// What a dead hand costs: more than any hand is worth, so one is kept only
/// when every choice is dead.
const DEAD: i32 = 1000;

/// The terms of a hand's grade, each before its weight
///
/// [`HeuristicBot`] lays a hand out twice, once for 胡 and once for the
/// fewest loose cards, and grades it by whichever layout comes out higher.
/// These are that layout's counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Terms {
    /// 胡 of the melds
    pub hu: i32,
    /// Cards outside every meld
    pub loose: i32,
    /// Half-胡 the melds fall short of the threshold, after the half-胡
    /// the 搭子 among the loose cards may bring
    pub short: i32,
    /// Unseen copies of the loose cards, each of which would pair one
    pub own: i32,
    /// Over the loose cards in a 搭子: the 胡 of the meld each would be
    /// eaten into, times the unseen cards that would finish it, four at
    /// most
    pub kept: i32,
    /// Over the loose cards that go with nothing: four times the 胡 of the
    /// meld each would be eaten into
    pub lonely: i32,
    /// Unseen cards the hand would win on
    pub waits: i32,
}

impl Terms {
    /// The grade, in [`SCALE`]ths of a 胡
    const fn grade(&self) -> i32 {
        SCALE * (self.hu - WEIGHT * self.loose) - SCALE * WEIGHT / 2 * self.short
            + OWN * self.own
            + self.kept
            - self.lonely
            + WAIT * self.waits
    }
}

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
/// is deterministic and looks one decision ahead, but for the draw it would
/// forgo: the seat next to draw claims only what beats the mean of its best
/// answers to every card it might draw instead.  Its own draw has no such
/// second chance, so it takes whatever beats leaving the hand as it is.
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
        self.terms_of(hand, need)
            .map_or(-SCALE * DEAD, |terms| terms.grade())
    }

    /// The terms of `hand`'s grade when its melds must be worth `need` to
    /// win, unless it is dead: of its two layouts, the one that grades
    /// higher
    fn terms_of(&mut self, hand: Hand, need: i32) -> Option<Terms> {
        if self.dead(hand, need) {
            return None;
        }
        let waits = self.waits(hand, need);
        self.solvers
            .iter_mut()
            .zip(COSTS)
            .map(|(solver, cost)| {
                let layout = solver
                    .layout(hand)
                    .expect("layouts with loose cards always exist");
                let loose = to_i32(layout.loose.len());
                let hu = layout.score + to_i32(cost) * loose;
                let [own, kept, lonely, hope] =
                    prospects(hand, &layout, self.live, self.eaten, self.general);
                Terms {
                    hu,
                    loose,
                    short: (2 * (need - hu) - hope).max(0),
                    own,
                    kept,
                    lonely,
                    waits,
                }
            })
            .max_by_key(Terms::grade)
    }

    /// How many unseen cards `hand` would win on
    fn waits(&mut self, hand: Hand, need: i32) -> i32 {
        // The winning card melds with at most two others, so a hand that
        // cannot be laid out with two loose cards or fewer waits on nothing.
        let loose = self.solvers[1]
            .layout(hand)
            .map_or(usize::MAX, |layout| layout.loose.len());
        if loose > 2 {
            return 0;
        }
        let live = self.live;
        live.distinct()
            .filter(|&(card, _)| self.win(hand, card, need).is_some())
            .map(|(_, n)| i32::from(n))
            .sum()
    }

    /// The hand's value after its best discard other than `locked`
    fn after_discard(&mut self, hand: Hand, locked: Card, need: i32) -> i32 {
        discardable(hand, Some(locked))
            .distinct()
            .map(|(card, _)| self.value(hand.without(card), need))
            .max()
            // A drawn general or a kong may leave nothing: a 相公.
            .unwrap_or(-SCALE * DEAD)
    }

    /// The grade of claiming `card` into `meld` and discarding
    fn take(&mut self, hand: Hand, meld: Meld, card: Card, need: i32) -> i32 {
        let gain = to_i32(self.exposed.score(&meld));
        let rest = hand - meld.cards().without(card);
        SCALE * gain + self.after_discard(rest, card, need - gain)
    }

    /// The grade of winning on `card`, if the hand does: its 胡, as a hand
    /// with no loose card and nothing short grades
    fn win(&mut self, hand: Hand, card: Card, need: i32) -> Option<i32> {
        locked_completions(hand, card)
            .into_iter()
            .filter_map(|meld| {
                // At 16 a loose card, a layout without any comes first.
                let layout = self.solvers[1].layout(hand - meld.cards().without(card))?;
                let hu = to_i32(self.exposed.score(&meld)) + layout.score;
                (layout.loose.is_empty() && hu >= need).then_some(SCALE * hu)
            })
            .max()
    }

    /// What `hand` is worth with a draw to come, when leaving the drawn
    /// card grades `pass`: the mean, over the cards not yet seen, of the
    /// best answer to each
    ///
    /// A win and a kong are taken unasked and a general cannot be left;
    /// any other card is claimed only when that beats `pass`.
    // ponytail: the draw is taken for certain, though a pong of the card
    // passed takes it away and a pile down to its dead cards has none.
    fn draw(&mut self, hand: Hand, need: i32, pass: i32) -> i32 {
        let live = self.live;
        let total: i32 = live
            .distinct()
            .map(|(card, n)| {
                let grade = if let Some(win) = self.win(hand, card, need) {
                    win
                } else if hand.count(card) == 3 {
                    self.take(hand, Meld::Quad(card), card, need)
                } else {
                    let options = options(hand, card, true);
                    let best = options
                        .pong
                        .then_some(Meld::Triplet(card))
                        .into_iter()
                        .chain(options.eats.iter().copied())
                        .map(|meld| self.take(hand, meld, card, need))
                        .max();
                    match best {
                        Some(best) if options.forced() => best,
                        Some(best) => best.max(pass),
                        None => pass,
                    }
                };
                i32::from(n) * grade
            })
            .sum();
        total / to_i32(live.len()).max(1)
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

/// What the loose cards of `layout` promise: [`Terms::own`],
/// [`Terms::kept`] and [`Terms::lonely`], which tell hands of the same melds
/// apart, and the half-胡 they may yet bring
///
/// A loose card is one of a 搭子 when the hand holds a second card of its
/// meld and a third is `live`, not yet seen.  It is then kept at the 胡
/// of the meld it would be eaten into (`eaten`, by [`family`]) for each live
/// card that would finish it, up to [`COPIES`], and promises half that 胡:
/// the other half is its partner's.  So a 搭子 is kept, soldiers before
/// 將士象 before 車馬包, and the one with more cards left to come before the
/// one with fewer.
///
/// A card that goes with nothing is lonely by as much as a 搭子 with every
/// finishing card left is kept, so the dearest goes first, before it can
/// finish another seat's hand.  Either way each live copy of the card,
/// which would pair it, counts as its own.
///
/// The second card may be a lone general or one of a pair — 車車 馬 still
/// eats a 包 — but not one of three or four alike, which are locked and
/// never eat; nor does a card finish the meld when the hand holds three of
/// it, since the fourth can only come as a kong.  士象 without their general
/// are the poorest 搭子: only the seat's own draw brings a general and no
/// seat ever discards one, so they are graded as if a single card were left
/// to finish them, and a general is worth `general` alone, so the 士象 add
/// only the rest of the meld.
fn prospects(hand: Hand, layout: &Layout, live: Hand, eaten: [i32; 3], general: i32) -> [i32; 4] {
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
            let own = i32::from(live.count(card));
            if finishers == 0 {
                [own, 0, COPIES * eaten[kin], 0]
            } else if kin == 0 && !hand.contains(Card::new(card.color, Rank::General)) {
                let worth = eaten[kin] - general;
                [own, worth, 0, worth]
            } else {
                [own, finishers.min(COPIES) * eaten[kin], 0, eaten[kin]]
            }
        })
        .fold([0; 4], |sum, card| {
            std::array::from_fn(|i| sum[i] + card[i])
        })
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

impl HeuristicBot {
    /// The answers to the offer, best first, each with its grade: the first
    /// is the bot's own choice
    ///
    /// A grade is a priority and has no unit.  A 胡 the hand holds counts 32
    /// in it, but the other terms are weights, so a difference of 32 is not
    /// a 胡 won or lost: grades only order the moves.  An answer that leaves
    /// the hand dead grades near −32000.
    pub fn claims(&mut self, view: &View<'_>, options: &ClaimOptions) -> Vec<(Claim, i32)> {
        let need = self.start(view);
        let hand = view.hand();
        let offer = view.offer().expect("claims answer an offer");
        let card = offer.card;
        // A drawn general cannot be passed.
        let pass = (!options.forced()).then(|| {
            let pass = self.value(hand, need);
            // Passing brings the next seat a draw; a claim must beat that
            // too.
            if view.seat() == offer.source.next() {
                (self.draw(hand, need, pass), Claim::Pass)
            } else {
                (pass, Claim::Pass)
            }
        });
        let melds = options
            .pong
            .then_some((Meld::Triplet(card), Claim::Pong))
            .into_iter()
            .chain(options.eats.iter().map(|&meld| (meld, Claim::Eat(meld))));
        let mut graded: Vec<(i32, Claim)> = pass
            .into_iter()
            .chain(melds.map(|(meld, claim)| (self.take(hand, meld, card, need), claim)))
            .collect();
        // Stable: the earlier of equals stays first, passing before claiming.
        graded.sort_by_key(|&(grade, _)| std::cmp::Reverse(grade));
        graded
            .into_iter()
            .map(|(grade, claim)| (claim, grade))
            .collect()
    }

    /// The legal discards, best first, each with its grade as for
    /// [`claims`](Self::claims): the first is the bot's own choice
    pub fn discards(&mut self, view: &View<'_>) -> Vec<(Card, i32)> {
        let need = self.start(view);
        let hand = view.hand();
        let mut graded: Vec<(i32, Card)> = discardable(hand, view.locked())
            .distinct()
            .map(|(card, _)| (self.value(hand.without(card), need), card))
            .collect();
        // Stable, and the later of equals goes first.
        graded.reverse();
        graded.sort_by_key(|&(grade, _)| std::cmp::Reverse(grade));
        graded
            .into_iter()
            .map(|(grade, card)| (card, grade))
            .collect()
    }
}

impl HeuristicBot {
    /// The [`Terms`] of the grade `hand` would get in `view`'s seat, or
    /// `None` when it is dead: too poor to reach the threshold however the
    /// cards fall
    ///
    /// `hand` is the seat's own or what a discard would leave of it.
    pub fn terms(&mut self, view: &View<'_>, hand: Hand) -> Option<Terms> {
        let need = self.start(view);
        self.terms_of(hand, need)
    }
}

impl Strategy for HeuristicBot {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        self.claims(view, options)[0].0
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        self.discards(view)[0].0
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
    fn waits() {
        let mut bot = HeuristicBot::new(&Rules::eight());
        let hand: Hand = "帥兵.帥帥仕俥炮兵.象象包包卒.將".parse().unwrap();
        let seen = "相相相仕仕.兵俥傌.馬馬將士士士.包包車車車將士包";
        bot.live = Hand::FULL - hand - seen.parse().unwrap();
        let mut value = |card: &str| bot.value(hand.without(card.parse().unwrap()), 7);
        // Shedding the 仕 waits on three 傌; the 俥 or the 炮, on nothing.
        assert!(value("黃仕") > value("黃俥"));
        assert!(value("黃仕") > value("黃炮"));
    }

    #[test]
    fn draws() {
        let mut bot = HeuristicBot::new(&Rules::new());
        let hand: Hand = "兵傌.兵..".parse().unwrap();
        let mut draw = |seen: &str, need| {
            bot.live = Hand::FULL - hand - seen.parse().unwrap();
            let pass = bot.value(hand, need);
            bot.draw(hand, need, pass) - pass
        };
        // A third soldier, a 兵 or a 傌 to pair: the draw is worth having.
        let open = draw("...", 0);
        assert!(open > 0);
        // Less so once the soldiers that would finish the meld are out,
        let pairs = draw("..卒卒卒卒.卒卒卒卒", 0);
        assert!(pairs < open);
        // and less again with nothing left to pair.
        assert!(draw("兵兵兵傌傌傌.兵兵兵.卒卒卒卒.卒卒卒卒", 0) < pairs);
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
            let [own, kept, lonely, hope] =
                super::prospects(hand, &layout, live, bot.eaten, bot.general);
            (OWN * own + kept - lonely, hope)
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
            ("仕相...", 14, 2), // 士象 beg for theirs, below 車馬
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
