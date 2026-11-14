//! Scripted deals reaching every claim, the forced kong, a win, and 天胡

use susik::{Card, Color, Hand, Meld};
use tsap_oo::{Claim, ClaimOptions, Event, Phase, Round, RoundError, Rules, Seat, discardable};

const FIRST: Seat = Seat::ALL[0];
const SECOND: Seat = Seat::ALL[1];
const THIRD: Seat = Seat::ALL[2];
const FOURTH: Seat = Seat::ALL[3];

fn hand(s: &str) -> Hand {
    s.parse().unwrap()
}

fn card(s: &str) -> Card {
    s.parse().unwrap()
}

/// A pile dealing exactly `hands`: `hands[0]` (21 cards) to `dealer`, then
/// 20 each to the seats after in turn order.  The undealt cards sit below.
fn pile(hands: [Hand; Seat::COUNT]) -> Vec<Card> {
    assert_eq!(hands[0].len(), 21);
    let dealt = hands.iter().fold(Hand::EMPTY, |a, &b| a + b);
    let mut pile: Vec<Card> = (Hand::FULL - dealt).iter().collect();
    // `from_pile` pops twenty for the dealer, twenty for each seat after,
    // then one more for the dealer.
    let mut dealer = hands[0];
    let extra = dealer.first().unwrap();
    dealer.remove(extra);
    pile.push(extra);
    for seat in (1..Seat::COUNT).rev() {
        pile.extend(hands[seat]);
    }
    pile.extend(dealer);
    pile
}

/// Every card is in exactly one place.
fn conserved(round: &Round) {
    let mut all = Hand::EMPTY;
    for seat in Seat::ALL {
        all += round.hand(seat);
        for meld in round.exposed(seat) {
            all += meld.cards();
        }
    }
    all.extend(round.discards().iter().map(|&(_, card)| card));
    all.extend(round.offer().map(|offer| offer.card));
    all.extend(round.result().and_then(|result| result.winning_card));
    assert_eq!(all.len() + round.pile_len(), 112);
    assert!(all.is_subset(Hand::FULL));
}

#[test]
fn eat_pong_kong_and_win() {
    // First: concealed triplets of the red court (9) and a lone yellow
    // general (1), pairs, and a loose 俥傌 waiting for a red cannon.
    let first = hand("帥帥帥仕仕仕相相相俥傌兵兵.帥仕仕炮.將將.將將");
    // Second eats First's yellow cannon into a field; Third could pong it.
    let second = hand(".俥傌炮.士士象象車車馬馬包包.將將士士象象卒");
    let third = hand("俥俥傌傌.相相相炮炮兵兵兵.卒卒.車車馬馬包包");
    // Fourth holds three white soldiers and the red cannon First wants.
    let fourth = hand("相炮.帥帥帥.士士車車馬馬包包.士士象象卒卒卒");
    let mut round =
        Round::from_pile(Rules::new(), FIRST, pile([first, second, third, fourth])).unwrap();
    assert_eq!(round.phase(), Phase::Discard(FIRST));
    assert_eq!(round.hand(FIRST), first);
    assert_eq!(round.hand(FOURTH), fourth);
    assert_eq!(round.pile_len(), 31);
    conserved(&round);

    assert_eq!(
        round.discard(SECOND, card("黃炮")),
        Err(RoundError::WrongPhase)
    );
    round.discard(FIRST, card("黃炮")).unwrap();
    assert_eq!(round.phase(), Phase::Claim(SECOND));
    assert_eq!(
        round.claim_options(SECOND),
        ClaimOptions {
            pong: false,
            eats: vec![Meld::Pair(card("黃炮")), Meld::Field(Color::Yellow)],
            blocked: vec![],
        }
    );
    assert_eq!(round.claim(THIRD, Claim::Pong), Err(RoundError::WrongPhase));
    assert_eq!(
        round.claim(SECOND, Claim::Pong),
        Err(RoundError::NotAllowed)
    );
    conserved(&round);

    // Second stakes an eat; Third, holding a pair, is still asked and passes.
    round
        .claim(SECOND, Claim::Eat(Meld::Field(Color::Yellow)))
        .unwrap();
    assert_eq!(round.phase(), Phase::Claim(THIRD));
    assert_eq!(
        round.claim_options(THIRD),
        ClaimOptions {
            pong: true,
            eats: vec![],
            blocked: vec![],
        }
    );
    round.claim(THIRD, Claim::Pass).unwrap();
    assert_eq!(round.phase(), Phase::Discard(SECOND));
    assert_eq!(round.exposed(SECOND), [Meld::Field(Color::Yellow)]);
    assert_eq!(round.hand(SECOND).len(), 18);
    assert_eq!(round.locked(), Some(card("黃炮")));
    assert_eq!(
        round.discard(SECOND, card("黃炮")),
        Err(RoundError::Locked(card("黃炮")))
    );
    assert_eq!(
        round.discard(SECOND, card("紅帥")),
        Err(RoundError::NotInHand(card("紅帥")))
    );
    conserved(&round);

    // Second's white soldier completes Fourth's triplet: a forced kong.
    round.discard(SECOND, card("白卒")).unwrap();
    assert_eq!(round.phase(), Phase::Discard(FOURTH));
    assert_eq!(round.exposed(FOURTH), [Meld::Quad(card("白卒"))]);
    assert_eq!(
        round.events().last(),
        Some(&Event::Kong {
            seat: FOURTH,
            card: card("白卒")
        })
    );
    assert_eq!(
        round.discard(FOURTH, card("白卒")),
        Err(RoundError::Locked(card("白卒")))
    );
    conserved(&round);

    // Fourth's red cannon completes First's field (1): with the concealed
    // triplets (9), the lone yellow general (1), and the four generals of
    // the green and white pairs scored as singles (4), that is 15.
    round.discard(FOURTH, card("紅炮")).unwrap();
    assert_eq!(round.phase(), Phase::Finished);
    let result = round.result().unwrap();
    assert_eq!(result.winner, Some(FIRST));
    assert_eq!(result.hu, 15);
    assert_eq!(result.from, Some(FOURTH));
    assert_eq!(result.winning_card, Some(card("紅炮")));
    assert_eq!(result.layout[0], Meld::Field(Color::Red));
    assert_eq!(
        result.layout.iter().map(|m| m.cards().len()).sum::<usize>(),
        21
    );
    assert_eq!(
        round.events().last(),
        Some(&Event::Win {
            seat: FIRST,
            hu: 15
        })
    );
    assert_eq!(
        round.discard(FIRST, card("紅帥")),
        Err(RoundError::WrongPhase)
    );
    conserved(&round);
}

#[test]
fn a_drawn_general_is_kept() {
    // The hands of `eat_pong_kong_and_win`, with a green general on top of
    // the pile: First holds its pair and Second the 士象 to eat it with.
    let first = hand("帥帥帥仕仕仕相相相俥傌兵兵.帥仕仕炮.將將.將將");
    let second = hand(".俥傌炮.士士象象車車馬馬包包.將將士士象象卒");
    let third = hand("俥俥傌傌.相相相炮炮兵兵兵.卒卒.車車馬馬包包");
    let fourth = hand("相炮.帥帥帥.士士車車馬馬包包.士士象象卒卒卒");
    let mut pile = pile([first, second, third, fourth]);
    let general = card("綠將");
    let at = pile[..31].iter().position(|&c| c == general).unwrap();
    pile.swap(at, 30);
    let mut round = Round::from_pile(Rules::new(), FIRST, pile).unwrap();

    assert_eq!(
        round.discard(FIRST, card("紅帥")),
        Err(RoundError::General(card("紅帥")))
    );
    // Nobody wants the red soldier, so Second draws the general and keeps
    // it: no eat for Second, no pong for First.
    round.discard(FIRST, card("紅兵")).unwrap();
    assert_eq!(round.phase(), Phase::Discard(SECOND));
    assert_eq!(
        round.events().last(),
        Some(&Event::Keep {
            seat: SECOND,
            card: general
        })
    );
    assert_eq!(round.hand(SECOND), second.with(general));
    assert!(round.exposed(SECOND).is_empty());
    assert_eq!(round.locked(), None);
    assert_eq!(
        round.discard(SECOND, general),
        Err(RoundError::General(general))
    );
    conserved(&round);

    // Neither generals nor the locked card are ever discardable.
    let generals = hand("帥..將將.");
    assert!(discardable(generals, None).is_empty());
    assert_eq!(
        discardable(generals.with(card("白卒")), None),
        hand("...卒")
    );
    assert!(discardable(generals.with(card("白卒")), Some(card("白卒"))).is_empty());
}

#[test]
fn dealer_wins_on_the_deal() {
    let dream = hand("帥帥帥帥仕仕仕仕相相相相.帥帥帥仕仕仕相相相..");
    let filler = hand("俥俥俥俥傌傌傌傌炮炮炮炮兵兵兵兵.帥俥俥俥..");
    let more = hand(".傌傌傌傌炮炮炮炮兵兵兵兵.將將將將士士士士.");
    let rest = hand("..象象象象車車車車馬馬馬馬包包包包卒卒卒卒.");
    let round = Round::from_pile(Rules::new(), THIRD, pile([dream, filler, more, rest])).unwrap();
    assert_eq!(round.phase(), Phase::Finished);
    let result = round.result().unwrap();
    assert_eq!(result.winner, Some(THIRD));
    assert_eq!(result.hu, 33);
    assert_eq!(result.winning_card, None);
    assert_eq!(result.from, None);
    conserved(&round);
}

#[test]
fn a_short_pile_is_rejected() {
    assert!(Round::from_pile(Rules::new(), FIRST, Vec::new()).is_err());
    // A fifth white soldier in place of a red general.
    let mut twice = Hand::FULL.iter().collect::<Vec<_>>();
    twice[0] = twice[111];
    assert!(Round::from_pile(Rules::new(), FIRST, twice).is_err());
}

#[cfg(feature = "rand")]
mod playouts {
    use super::{FIRST, conserved};
    use rand::SeedableRng as _;
    use rand::rngs::SmallRng;
    use tsap_oo::{Event, Game, HeuristicBot, Phase, Rules, Seat, Step, Table};

    /// Bots play out whole rounds; cards are conserved after every step and
    /// every win reaches the threshold.
    #[test]
    fn bots_finish_rounds() {
        playout(Rules::new());
    }

    /// The same under 八胡: nothing in the engine assumes ten.
    #[test]
    fn bots_finish_rounds_of_eight() {
        playout(Rules::eight());
    }

    fn playout(rules: Rules) {
        let mut game = Game::new(rules, FIRST);
        let mut rng = SmallRng::seed_from_u64(2026);
        let (mut wins, mut exhausted, mut fouls) = (0, 0, 0);
        for _ in 0..200 {
            let bots = Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as _));
            let mut table = Table::new(game.deal(&mut rng), bots);
            loop {
                match table.step().unwrap() {
                    Step::Acted(seat) => {
                        let round = table.round();
                        conserved(round);
                        // A claim never strands a seat without a discard.
                        if round.phase() == Phase::Discard(seat) {
                            let cards = tsap_oo::discardable(round.hand(seat), round.locked());
                            assert!(!cards.is_empty());
                        }
                    }
                    Step::Human(_) => unreachable!("every seat has a bot"),
                    Step::Finished => break,
                }
            }
            let result = table.round().result().unwrap();
            conserved(table.round());
            match result.winner {
                Some(winner) => {
                    wins += 1;
                    assert!(result.hu >= rules.threshold);
                    assert_eq!(
                        result.layout.iter().map(|m| m.cards().len()).sum::<usize>(),
                        21
                    );
                    game.settle(result);
                    assert_eq!(game.dealer(), winner);
                }
                None if result.foul.is_some() => {
                    // Only a draw or a forced kong strands a seat.
                    let events = table.round().events();
                    assert!(matches!(
                        events[events.len() - 2],
                        Event::Keep { .. } | Event::Kong { .. }
                    ));
                    game.settle(result);
                    fouls += 1;
                }
                None => exhausted += 1,
            }
        }
        assert_eq!(wins + exhausted + fouls, 200);
        assert!(wins > 0, "the bots never won");
        assert_eq!(game.scores().iter().sum::<i32>(), 0);
    }
}
