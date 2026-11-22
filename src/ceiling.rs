//! The most 胡 a hand can still win with, if every card it wants comes.
//!
//! A win is 21 cards: the exposed melds, the concealed cards that are
//! kept, and the cards that come from outside, one per meld they complete
//! (`doc/hu-ceiling.md`).  The ceiling asks for the best layout of that
//! kind and ignores who could supply the cards and in what order, so it
//! is an upper bound on the 胡 the hand ends at.

use crate::rules::Rules;
use susik::{Card, Color, Hand, Meld, Rank, Values, completions};

/// The most 胡 `hand` can win with beside `exposed`, if every card in
/// `live` came when wanted and at most `generals` generals came from
/// outside; `None` if no layout of 21 cards exists
///
/// `live` is the cards not yet seen, none of them in `hand`.  A general
/// never leaves the hand, and from outside it is only eaten alone or into
/// 將士象, never paired or ponged.  Three or four identical cards kept are a
/// triplet, a quad, or a kong with the fourth from outside, which does not
/// count against `generals`.
#[must_use]
pub fn ceiling(
    rules: &Rules,
    exposed: &[Meld],
    hand: Hand,
    live: Hand,
    generals: usize,
) -> Option<u32> {
    let shown: usize = exposed.iter().map(Meld::size).sum();
    let base: u32 = exposed.iter().map(|meld| rules.exposed.score(meld)).sum();
    reach(rules, shown, hand, live, generals).map(|hu| hu + base)
}

/// The most 胡 the kept cards of `hand` and the cards from outside add
/// beside `shown` exposed cards, as [`ceiling`] but without the exposed
/// melds' own 胡
pub(crate) fn reach(
    rules: &Rules,
    shown: usize,
    hand: Hand,
    live: Hand,
    generals: usize,
) -> Option<u32> {
    let mut acc = Table::new(generals);
    acc.raise(0, 0, 0);
    for block in blocks() {
        let mut table = Table::new(generals);
        let mut lay = Lay {
            concealed: rules.concealed,
            exposed: rules.exposed,
            out: &mut table,
        };
        lay.lay(
            within(hand, block),
            Hand::EMPTY,
            within(live, block),
            0,
            0,
            0,
        );
        acc = acc.merge(&table);
    }
    // Discards less outside cards: what brings the count to 21.
    let target = i32::try_from(hand.len() + shown).ok()? - 21;
    (0..=generals).filter_map(|g| acc.get(target, g)).max()
}

/// The nine sets of cards that only meld among themselves: each colour's
/// 將士象 and 車馬炮, and the soldiers
fn blocks() -> impl Iterator<Item = Hand> {
    const COURT: [Rank; 3] = [Rank::General, Rank::Advisor, Rank::Elephant];
    const FIELD: [Rank; 3] = [Rank::Chariot, Rank::Horse, Rank::Cannon];
    let full = |cards: Vec<Card>| -> Hand { cards.into_iter().flat_map(|c| [c; 4]).collect() };
    Color::ALL
        .into_iter()
        .flat_map(move |color| {
            [COURT, FIELD].map(|ranks| full(ranks.map(|rank| Card::new(color, rank)).to_vec()))
        })
        .chain([full(
            Color::ALL
                .map(|color| Card::new(color, Rank::Soldier))
                .to_vec(),
        )])
}

/// The cards of `hand` in `block`
fn within(hand: Hand, block: Hand) -> Hand {
    hand.iter().filter(|&card| block.contains(card)).collect()
}

/// How far discards less outside cards may run either way
const SPAN: i32 = 32;

/// The most 胡 by (discards − outside cards, outside generals used)
struct Table {
    generals: usize,
    cells: Vec<Option<u32>>,
}

impl Table {
    fn new(generals: usize) -> Self {
        Self {
            generals,
            cells: vec![None; (2 * SPAN as usize + 1) * (generals + 1)],
        }
    }

    fn index(&self, dm: i32, g: usize) -> Option<usize> {
        let row = usize::try_from(dm + SPAN)
            .ok()
            .filter(|&row| row <= 2 * SPAN as usize)?;
        (g <= self.generals).then(|| row * (self.generals + 1) + g)
    }

    fn get(&self, dm: i32, g: usize) -> Option<u32> {
        self.cells[self.index(dm, g)?]
    }

    /// Record `hu` at the cell if it beats what is there
    fn raise(&mut self, dm: i32, g: usize, hu: u32) {
        if let Some(i) = self.index(dm, g)
            && self.cells[i].is_none_or(|best| hu > best)
        {
            self.cells[i] = Some(hu);
        }
    }

    /// Lay another block beside this one
    fn merge(&self, other: &Self) -> Self {
        let mut out = Self::new(self.generals);
        let cells = |t: &Self| {
            (-SPAN..=SPAN)
                .flat_map(|dm| (0..=t.generals).map(move |g| (dm, g)))
                .filter_map(|(dm, g)| t.get(dm, g).map(|hu| (dm, g, hu)))
                .collect::<Vec<_>>()
        };
        let others = cells(other);
        for (dm1, g1, hu1) in cells(self) {
            for &(dm2, g2, hu2) in &others {
                out.raise(dm1 + dm2, g1 + g2, hu1 + hu2);
            }
        }
        out
    }
}

/// Enumerates the layouts of one block
struct Lay<'a> {
    concealed: Values,
    exposed: Values,
    out: &'a mut Table,
}

impl Lay<'_> {
    /// Place `hand`, whose cards in `fixed` have had their discards decided,
    /// with `live` still to come from outside, having so far discarded `dm`
    /// more than taken from outside, used `g` outside generals and scored
    /// `hu`
    fn lay(&mut self, hand: Hand, fixed: Hand, live: Hand, dm: i32, g: usize, hu: u32) {
        let Some(card) = hand.first() else {
            // Outside generals eaten alone
            let kings: u8 = live
                .distinct()
                .filter(|(c, _)| c.rank == Rank::General)
                .map(|(_, n)| n)
                .sum();
            for s in 0..=kings {
                let s = usize::from(s);
                if g + s > self.out.generals {
                    break;
                }
                let dm = dm - i32::try_from(s).expect("under four");
                self.out.raise(
                    dm,
                    g + s,
                    hu + u32::try_from(s).expect("under four") * self.exposed.single,
                );
            }
            return;
        };
        let count = hand.count(card);
        if !fixed.contains(card) {
            // Decide how many copies to discard: no general, and three or
            // four kept are a set and nothing else.
            let most = if card.rank == Rank::General { 0 } else { count };
            for discarded in 0..=most {
                let kept = count - discarded;
                let rest = (0..discarded).fold(hand, |h, _| h.without(card));
                let dm = dm + i32::from(discarded);
                match kept {
                    0 => self.lay(rest, fixed, live, dm, g, hu),
                    1 | 2 => self.lay(rest, fixed.with(card), live, dm, g, hu),
                    3 => {
                        let gone = rest - Meld::Triplet(card).cards();
                        self.lay(gone, fixed, live, dm, g, hu + self.concealed.triplet);
                        if live.contains(card) {
                            let hu = hu + self.exposed.quad;
                            self.lay(gone, fixed, live.without(card), dm - 1, g, hu);
                        }
                    }
                    _ => {
                        let gone = rest - Meld::Quad(card).cards();
                        self.lay(gone, fixed, live, dm, g, hu + self.concealed.quad);
                    }
                }
            }
            return;
        }
        let rest = hand.without(card);
        for meld in completions(rest, card) {
            let hu = hu + self.concealed.score(&meld);
            self.lay(hand - meld.cards(), fixed, live, dm, g, hu);
        }
        for (outside, _) in live.distinct() {
            if rest.count(outside) >= 4 {
                continue;
            }
            let king = outside.rank == Rank::General;
            let g = g + usize::from(king);
            if g > self.out.generals {
                continue;
            }
            for meld in completions(rest.with(outside), card) {
                let cards = meld.cards();
                // The outside card is in the meld and the rest is held;
                // the hand may hold another copy of it for a later meld
                if !cards.contains(outside) || !cards.without(outside).is_subset(hand) {
                    continue;
                }
                if king && !matches!(meld, Meld::Court(_)) {
                    continue;
                }
                let hu = hu + self.exposed.score(&meld);
                self.lay(
                    hand - cards.without(outside),
                    fixed,
                    live.without(outside),
                    dm - 1,
                    g,
                    hu,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hand(s: &str) -> Hand {
        s.parse().unwrap()
    }

    /// The ceiling beside `exposed`, every unseen card live
    fn ceil(exposed: &[Meld], cards: &str, generals: usize) -> Option<u32> {
        let hand = hand(cards);
        let shown = exposed.iter().fold(hand, |h, m| h + m.cards());
        ceiling(&Rules::new(), exposed, hand, Hand::FULL - shown, generals)
    }

    use Color::{Green, Red, White, Yellow};
    use Meld::{Court, Field, Triplet};

    /// Four exposed courts and a ponged white soldier: fifteen cards, 9 胡
    const FIFTEEN: [Meld; 5] = [
        Court(Red),
        Court(Yellow),
        Court(Green),
        Court(White),
        Triplet(Card::new(White, Rank::Soldier)),
    ];

    #[test]
    fn the_old_table_of_loose_cards() {
        // O = 5, two 搭子 and a lone card: two melds
        assert_eq!(ceil(&FIFTEEN, "俥傌.俥傌.車.", 0), Some(11));
        // O = 8, four 搭子 beside four courts: three melds
        assert_eq!(ceil(&FIFTEEN[..4], "俥傌.俥傌.車馬.車馬", 0), Some(11));
    }

    #[test]
    fn a_pair_and_a_lone_card_make_nothing() {
        // 俥傌兵 beside two generals: the eaten 車馬炮 forces 兵 out and the
        // hand is melded short; only pairs fit, or an outside general
        let exposed = [
            Court(Green),
            Court(White),
            Field(Green),
            Field(White),
            Triplet(Card::new(White, Rank::Soldier)),
        ];
        assert_eq!(ceil(&exposed, "帥帥俥傌兵...", 0), Some(7 + 2));
        assert_eq!(ceil(&exposed, "帥帥俥傌兵...", 1), Some(7 + 4));
    }

    #[test]
    fn a_general_beside_a_pair() {
        let exposed = [
            Field(Red),
            Field(Yellow),
            Field(Green),
            Field(White),
            Court(Green),
            Court(White),
        ];
        // 帥仕仕: single and pair for 9, or 帥仕 eats 相 for 10
        assert_eq!(ceil(&exposed, "帥仕仕...", 0), Some(10));
        assert_eq!(ceil(&exposed, "帥仕仕...", 1), Some(10));
    }

    #[test]
    fn three_generals_kong_or_stay_a_triplet() {
        let exposed = [
            Field(Yellow),
            Field(Green),
            Field(White),
            Court(Green),
            Court(White),
        ];
        // Triplet and 俥傌炮 for 11, or the kong, 俥 paired, 傌炮 out for 13
        assert_eq!(ceil(&exposed, "帥帥帥俥傌炮...", 0), Some(13));
        let mut live = Hand::FULL - hand("帥帥帥俥傌炮...");
        live = exposed.iter().fold(live, |h, m| h - m.cards());
        live.remove(Card::new(Red, Rank::General));
        let rules = Rules::new();
        assert_eq!(
            ceiling(&rules, &exposed, hand("帥帥帥俥傌炮..."), live, 0),
            Some(11)
        );
    }

    #[test]
    fn a_fat_belly_reaches_ten_only_on_its_fourth() {
        let exposed = [
            Field(Red),
            Field(Yellow),
            Field(White),
            Court(Red),
            Court(Yellow),
        ];
        let hand = hand("..車車車馬包.");
        let rules = Rules::new();
        let live = exposed.iter().fold(Hand::FULL - hand, |h, m| h - m.cards());
        // The kong, 馬 paired and 炮 out
        assert_eq!(ceiling(&rules, &exposed, hand, live, 0), Some(13));
        // Without the fourth 車 the set must be broken by a discard first:
        // 車炮 eat a 馬 and 車馬 a 炮, two 車馬炮 for 9, short of ten
        let gone = live.without(Card::new(Green, Rank::Chariot));
        assert_eq!(ceiling(&rules, &exposed, hand, gone, 0), Some(9));
    }

    #[test]
    fn a_soldier_between_two_others_splits_when_short() {
        // 紅兵 黃兵 黃兵 綠卒 beside 車馬 and 5 胡 shown, a discard owed
        let exposed = [
            Field(Red),
            Field(Yellow),
            Field(Green),
            Field(White),
            Triplet(Card::new(White, Rank::Advisor)),
        ];
        let rules = Rules::new();
        let held = hand("兵.兵兵.卒.車馬");
        let live = exposed.iter().fold(Hand::FULL - held, |h, m| h - m.cards());
        let top = |cards: &str, live| ceiling(&rules, &exposed, hand(cards), live, 0);
        // 紅黃 and 黃綠 each eat a third colour and 車馬 go: 5 + 3 + 3
        assert_eq!(top("兵.兵兵.卒.車馬", live), Some(11));
        assert_eq!(top("兵.兵兵.卒.馬", live), Some(11));
        // Without the second 黃兵 the three colours and 車馬炮 make 9; ten
        // takes a white 卒 for four colours and the 車 paired
        assert_eq!(top("兵.兵.卒.車馬", live), Some(10));
        let white = Card::new(White, Rank::Soldier);
        let none = (0..4).fold(live, |h, _| h.without(white));
        assert_eq!(top("兵.兵.卒.車馬", none), Some(9));
    }

    #[test]
    fn a_finished_hand_is_its_own_ceiling() {
        // 21 cards, nothing from outside: 4 + 1 + 2 + 1 + 1 + 3
        let exposed = [Field(Red), Field(Yellow), Field(Green), Field(White)];
        assert_eq!(ceil(&exposed, "帥.帥仕相.將.將卒卒卒", 0), Some(12));
    }
}
