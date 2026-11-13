//! Scripted deals reaching every claim, the forced kong, a win, and 天胡

use susik::{Card, Color, Hand, Meld};
use tsap_oo::{Claim, ClaimOptions, Event, Phase, Round, RoundError, Rules, Seat};

fn hand(s: &str) -> Hand {
    s.parse().unwrap()
}

fn card(s: &str) -> Card {
    s.parse().unwrap()
}

/// A pile dealing exactly `hands`: `hands[0]` (21 cards) to `dealer`, then
/// 20 each to the seats after in turn order.  The undealt cards sit below.
fn pile(hands: [Hand; 4]) -> Vec<Card> {
    assert_eq!(hands[0].len(), 21);
    let dealt = hands.iter().fold(Hand::EMPTY, |a, &b| a + b);
    let mut pile: Vec<Card> = (Hand::FULL - dealt).iter().collect();
    // `from_pile` pops twenty for the dealer, twenty for each seat after,
    // then one more for the dealer.
    let mut dealer = hands[0];
    let extra = dealer.first().unwrap();
    dealer.remove(extra);
    pile.push(extra);
    for seat in (1..4).rev() {
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
    // East: concealed triplets of the red court (9) and a lone yellow
    // general (1), pairs, and a loose 俥傌 waiting for a red cannon.
    let east = hand("帥帥帥仕仕仕相相相俥傌兵兵.帥仕仕炮.將將.將將");
    // South eats East's yellow cannon into a field; West could pong it.
    let south = hand(".俥傌炮.士士象象車車馬馬包包.將將士士象象卒");
    let west = hand("俥俥傌傌.相相相炮炮兵兵兵.卒卒.車車馬馬包包");
    // North holds three white soldiers and the red cannon East wants.
    let north = hand("相炮.帥帥帥.士士車車馬馬包包.士士象象卒卒卒");
    let mut round =
        Round::from_pile(Rules::new(), Seat::East, pile([east, south, west, north])).unwrap();
    assert_eq!(round.phase(), Phase::Discard(Seat::East));
    assert_eq!(round.hand(Seat::East), east);
    assert_eq!(round.hand(Seat::North), north);
    assert_eq!(round.pile_len(), 31);
    conserved(&round);

    assert_eq!(
        round.discard(Seat::South, card("黃炮")),
        Err(RoundError::WrongPhase)
    );
    round.discard(Seat::East, card("黃炮")).unwrap();
    assert_eq!(round.phase(), Phase::Claim(Seat::South));
    assert_eq!(
        round.claim_options(Seat::South),
        ClaimOptions {
            pong: false,
            eats: vec![Meld::Pair(card("黃炮")), Meld::Field(Color::Yellow)],
        }
    );
    assert_eq!(
        round.claim(Seat::West, Claim::Pong),
        Err(RoundError::WrongPhase)
    );
    assert_eq!(
        round.claim(Seat::South, Claim::Pong),
        Err(RoundError::NotAllowed)
    );
    conserved(&round);

    // South stakes an eat; West, holding a pair, is still asked and passes.
    round
        .claim(Seat::South, Claim::Eat(Meld::Field(Color::Yellow)))
        .unwrap();
    assert_eq!(round.phase(), Phase::Claim(Seat::West));
    assert_eq!(
        round.claim_options(Seat::West),
        ClaimOptions {
            pong: true,
            eats: vec![]
        }
    );
    round.claim(Seat::West, Claim::Pass).unwrap();
    assert_eq!(round.phase(), Phase::Discard(Seat::South));
    assert_eq!(round.exposed(Seat::South), [Meld::Field(Color::Yellow)]);
    assert_eq!(round.hand(Seat::South).len(), 18);
    assert_eq!(round.locked(), Some(card("黃炮")));
    assert_eq!(
        round.discard(Seat::South, card("黃炮")),
        Err(RoundError::Locked(card("黃炮")))
    );
    assert_eq!(
        round.discard(Seat::South, card("紅帥")),
        Err(RoundError::NotInHand(card("紅帥")))
    );
    conserved(&round);

    // South's white soldier is the fourth of North's triplet: a forced kong.
    round.discard(Seat::South, card("白卒")).unwrap();
    assert_eq!(round.phase(), Phase::Discard(Seat::North));
    assert_eq!(round.exposed(Seat::North), [Meld::Quad(card("白卒"))]);
    assert_eq!(
        round.events().last(),
        Some(&Event::Kong {
            seat: Seat::North,
            card: card("白卒")
        })
    );
    assert_eq!(
        round.discard(Seat::North, card("白卒")),
        Err(RoundError::Locked(card("白卒")))
    );
    conserved(&round);

    // North's red cannon completes East's field (1): with the concealed
    // triplets (9), the lone yellow general (1), and the four generals of
    // the green and white pairs scored as singles (4), that is 15.
    round.discard(Seat::North, card("紅炮")).unwrap();
    assert_eq!(round.phase(), Phase::Finished);
    let result = round.result().unwrap();
    assert_eq!(result.winner, Some(Seat::East));
    assert_eq!(result.hu, 15);
    assert_eq!(result.from, Some(Seat::North));
    assert_eq!(result.winning_card, Some(card("紅炮")));
    assert_eq!(result.layout[0], Meld::Field(Color::Red));
    assert_eq!(
        result.layout.iter().map(|m| m.cards().len()).sum::<usize>(),
        21
    );
    assert_eq!(
        round.events().last(),
        Some(&Event::Win {
            seat: Seat::East,
            hu: 15
        })
    );
    assert_eq!(
        round.discard(Seat::East, card("紅帥")),
        Err(RoundError::WrongPhase)
    );
    conserved(&round);
}

#[test]
fn dealer_wins_on_the_deal() {
    let dream = hand("帥帥帥帥仕仕仕仕相相相相.帥帥帥仕仕仕相相相..");
    let filler = hand("俥俥俥俥傌傌傌傌炮炮炮炮兵兵兵兵.帥俥俥俥..");
    let more = hand(".傌傌傌傌炮炮炮炮兵兵兵兵.將將將將士士士士.");
    let rest = hand("..象象象象車車車車馬馬馬馬包包包包卒卒卒卒.");
    let round =
        Round::from_pile(Rules::new(), Seat::West, pile([dream, filler, more, rest])).unwrap();
    assert_eq!(round.phase(), Phase::Finished);
    let result = round.result().unwrap();
    assert_eq!(result.winner, Some(Seat::West));
    assert_eq!(result.hu, 33);
    assert_eq!(result.winning_card, None);
    assert_eq!(result.from, None);
    conserved(&round);
}

#[test]
fn a_short_pile_is_rejected() {
    assert!(Round::from_pile(Rules::new(), Seat::East, Vec::new()).is_err());
    // A fifth white soldier in place of a red general.
    let mut twice = Hand::FULL.iter().collect::<Vec<_>>();
    twice[0] = twice[111];
    assert!(Round::from_pile(Rules::new(), Seat::East, twice).is_err());
}

#[cfg(feature = "rand")]
mod playouts {
    use super::conserved;
    use rand::SeedableRng as _;
    use rand::rngs::SmallRng;
    use tsap_oo::{Game, HeuristicBot, Phase, Rules, Seat, Step, Table};

    /// Bots play out whole rounds; cards are conserved after every step and
    /// every win reaches the threshold.
    #[test]
    fn bots_finish_rounds() {
        let rules = Rules::new();
        let mut game = Game::new(rules, Seat::East);
        let mut rng = SmallRng::seed_from_u64(2026);
        let (mut wins, mut exhausted) = (0, 0);
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
                            let mut cards = round.hand(seat).distinct();
                            assert!(cards.any(|(card, _)| Some(card) != round.locked()));
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
                None => exhausted += 1,
            }
        }
        assert_eq!(wins + exhausted, 200);
        assert!(wins > 0, "the bots never won");
        assert_eq!(game.scores().iter().sum::<i32>(), 0);
    }
}
