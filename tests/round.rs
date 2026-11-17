//! Scripted deals reaching every claim, the forced kong, a win, and 天胡

use susik::{Card, Color, Hand, Meld};
use tsap_oo::{
    Claim, ClaimOptions, Event, HeuristicBot, Phase, Round, RoundError, Rules, Seat, Strategy,
    View, discardable,
};

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
fn a_drawn_general_is_eaten() {
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
    // Nobody wants the red soldier, so Second draws the general and must
    // eat it, alone or with the 士象: no pass, no pong for First.
    round.discard(FIRST, card("紅兵")).unwrap();
    assert_eq!(round.phase(), Phase::Claim(SECOND));
    let options = round.claim_options(SECOND);
    let court = Meld::Court(general.color);
    assert_eq!(options.eats, [Meld::Single(general), court]);
    assert!(options.forced());
    assert_eq!(
        round.claim(SECOND, Claim::Pass),
        Err(RoundError::NotAllowed)
    );
    round.claim(SECOND, Claim::Eat(court)).unwrap();
    assert_eq!(round.phase(), Phase::Discard(SECOND));
    assert_eq!(
        round.events().last(),
        Some(&Event::Eat {
            seat: SECOND,
            meld: court,
            card: general
        })
    );
    // The general went to the table, never into the hand.
    assert_eq!(round.hand(SECOND), second - hand("..士象."));
    assert_eq!(round.exposed(SECOND), [court]);
    assert_eq!(round.locked(), Some(general));
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

/// 大肚: 俥俥俥傌炮 is a triplet and two loose cards, never 俥俥 and 俥傌炮.
#[test]
fn a_concealed_triplet_is_not_split_to_win() {
    let belly = hand("帥帥帥帥仕仕仕仕相相相相.仕仕相相俥俥俥傌炮..");
    let filler = hand("俥俥俥俥傌傌傌傌炮炮炮炮兵兵兵兵.帥帥帥帥..");
    let more = hand(".傌傌傌炮炮炮兵兵兵兵.將將將將士士士士象象.");
    let rest = hand("..象象車車車車馬馬馬馬包包包包卒卒卒卒.將將");
    assert!(susik::partition(belly, Rules::new().concealed).is_some());
    let round = Round::from_pile(Rules::new(), THIRD, pile([belly, filler, more, rest])).unwrap();
    assert_eq!(round.phase(), Phase::Discard(THIRD));
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
    use susik::Meld;
    use tsap_oo::{Event, Game, HeuristicBot, Phase, Rules, Seat, Step, Table, View};

    /// Bots play out whole rounds; cards are conserved after every step,
    /// every win reaches the threshold, and none comes on a card the winner
    /// was proven not to win on.
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
        let (mut wins, mut exhausted, mut fouls, mut proven) = (0, 0, 0, 0);
        for _ in 0..200 {
            let bots = Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as _));
            let mut table = Table::new(game.deal(&mut rng), bots);
            let mut safe = Seat::ALL.map(|_| susik::Hand::EMPTY);
            loop {
                let round = table.round();
                if round.phase() != Phase::Finished {
                    safe = Seat::ALL.map(|seat| View::new(round, seat).safe(seat));
                }
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
                    if let Some(card) = result.winning_card {
                        assert!(!safe[winner.index()].contains(card));
                        proven += safe[winner.index()].len();
                    }
                    assert_eq!(
                        result.layout.iter().map(|m| m.cards().len()).sum::<usize>(),
                        21
                    );
                    game.settle(result);
                    assert_eq!(game.dealer(), winner);
                }
                None if result.foul.is_some() => {
                    // Only a drawn general eaten alone or a forced kong
                    // strands a seat.
                    let events = table.round().events();
                    assert!(matches!(
                        events[events.len() - 2],
                        Event::Eat {
                            meld: Meld::Single(_),
                            ..
                        } | Event::Kong { .. }
                    ));
                    game.settle(result);
                    fouls += 1;
                }
                None => exhausted += 1,
            }
        }
        assert_eq!(wins + exhausted + fouls, 200);
        assert!(wins > 0, "the bots never won");
        assert!(proven > 0, "no card was ever proven safe");
        assert_eq!(game.scores().iter().sum::<i32>(), 0);
    }
}

/// A determinized round moves only the cards the seat cannot see.
#[cfg(feature = "rand")]
#[test]
fn a_determinized_round_looks_the_same() {
    use rand::SeedableRng as _;
    use tsap_oo::View;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    let round = Round::deal(Rules::new(), FIRST, &mut rng);
    let world = View::new(&round, SECOND).determinize(&mut rng);
    conserved(&world);
    assert_eq!(world.hand(SECOND), round.hand(SECOND));
    assert_eq!(world.phase(), round.phase());
    assert_eq!(world.pile_len(), round.pile_len());
    for seat in Seat::ALL {
        assert_eq!(world.hand(seat).len(), round.hand(seat).len());
    }
    assert_ne!(world.hand(THIRD), round.hand(THIRD));
}

#[test]
fn the_greedy_bot_draws_rather_than_eat_into_a_meld_it_holds() {
    // Second holds 俥傌炮 in yellow and nothing else to go with a 炮.  Eating
    // First's 炮 would only swap a loose card for it and cost the draw.
    let second = hand("帥仕兵兵.帥仕相相俥傌炮兵.將馬馬包包.將包卒");
    let offered = card("黃炮");
    // Deal the rest in turn, so no other seat holds more than two alike.
    let mut others = [Hand::EMPTY; 3];
    let rest = Hand::FULL - second - Hand::EMPTY.with(offered);
    for (i, card) in rest.iter().take(60).enumerate() {
        others[i % 3].insert(card);
    }
    let [first, third, fourth] = others;
    let hands = [first.with(offered), second, third, fourth];
    let mut round = Round::from_pile(Rules::new(), FIRST, pile(hands)).unwrap();
    round.discard(FIRST, offered).unwrap();
    assert_eq!(round.phase(), Phase::Claim(SECOND));
    let options = round.claim_options(SECOND);
    assert!(options.eats.contains(&Meld::Field(Color::Yellow)));
    let mut bot = HeuristicBot::new(&Rules::new());
    assert_eq!(bot.claim(&View::new(&round, SECOND), &options), Claim::Pass);
}

#[test]
fn a_pair_held_may_eat_into_a_pair_but_the_bot_does_not() {
    // Second holds 俥傌炮炮 in yellow: First's 炮 may be ponged, eaten into
    // 俥傌炮, or eaten into a pair keeping 俥傌炮 in hand.
    let second = hand("帥仕兵兵.帥仕相相俥傌炮炮兵.將馬馬包.將包卒");
    let offered = card("黃炮");
    let mut others = [Hand::EMPTY; 3];
    let rest = Hand::FULL - second - Hand::EMPTY.with(offered);
    for (i, card) in rest.iter().take(60).enumerate() {
        others[i % 3].insert(card);
    }
    let [first, third, fourth] = others;
    let hands = [first.with(offered), second, third, fourth];
    let mut round = Round::from_pile(Rules::new(), FIRST, pile(hands)).unwrap();
    round.discard(FIRST, offered).unwrap();
    assert_eq!(round.phase(), Phase::Claim(SECOND));
    let options = round.claim_options(SECOND);
    assert!(options.pong);
    assert!(options.eats.contains(&Meld::Field(Color::Yellow)));
    assert!(options.eats.contains(&Meld::Pair(offered)));
    let mut bot = HeuristicBot::new(&Rules::new());
    let claim = bot.claim(&View::new(&round, SECOND), &options);
    assert_ne!(claim, Claim::Eat(Meld::Pair(offered)));
}

/// At every decision of the eventual winner, the ceiling with the generals
/// that came from outside afterwards is at least the 胡 won.
#[cfg(feature = "rand")]
#[test]
fn the_ceiling_bounds_every_win() {
    use rand::SeedableRng as _;
    use rand::rngs::SmallRng;
    use std::cell::RefCell;
    use std::rc::Rc;
    use susik::Rank;
    use tsap_oo::{Step, Table, ceiling};

    struct Point {
        seat: Seat,
        hand: Hand,
        exposed: Vec<Meld>,
        live: Hand,
    }
    struct Recorder {
        bot: HeuristicBot,
        log: Rc<RefCell<Vec<Point>>>,
    }
    impl Recorder {
        fn record(&self, view: &View<'_>) {
            let shown = Seat::ALL
                .into_iter()
                .flat_map(|seat| view.exposed(seat))
                .flat_map(Meld::cards)
                .chain(view.discards().iter().map(|&(_, card)| card));
            // The offer stays live: the seat may be about to claim it
            self.log.borrow_mut().push(Point {
                seat: view.seat(),
                hand: view.hand(),
                exposed: view.exposed(view.seat()).to_vec(),
                live: shown.fold(Hand::FULL - view.hand(), Hand::without),
            });
        }
    }
    impl Strategy for Recorder {
        fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
            self.record(view);
            self.bot.claim(view, options)
        }
        fn discard(&mut self, view: &View<'_>) -> Card {
            self.record(view);
            self.bot.discard(view)
        }
    }
    let generals = |hand: Hand| hand.iter().filter(|c| c.rank == Rank::General).count();

    let rules = Rules::new();
    let mut rng = SmallRng::seed_from_u64(2026);
    let mut checked = 0;
    for i in 0..100 {
        let round = Round::deal(rules, Seat::ALL[i % Seat::COUNT], &mut rng);
        let log = Rc::new(RefCell::new(Vec::new()));
        let bots = Seat::ALL.map(|_| {
            let bot = HeuristicBot::new(&rules);
            Some(Box::new(Recorder {
                bot,
                log: log.clone(),
            }) as Box<dyn Strategy>)
        });
        let mut table = Table::new(round, bots);
        while table.step().unwrap() != Step::Finished {}
        let round = table.round();
        let result = round.result().unwrap();
        let Some(winner) = result.winner else {
            continue;
        };
        let total = result
            .layout
            .iter()
            .map(|m| generals(m.cards()))
            .sum::<usize>();
        for point in log.borrow().iter().filter(|p| p.seat == winner) {
            // A kong's fourth general does not count as one from outside
            let kongs = round.exposed(winner)[point.exposed.len()..]
                .iter()
                .filter(|m| matches!(m, Meld::Quad(c) if c.rank == Rank::General))
                .count();
            let held = generals(point.hand)
                + point
                    .exposed
                    .iter()
                    .map(|m| generals(m.cards()))
                    .sum::<usize>();
            let outside = total - held - kongs;
            let bound = ceiling(&rules, &point.exposed, point.hand, point.live, outside);
            assert!(
                bound.is_some_and(|hu| hu >= result.hu),
                "round {i}: {} beside {:?} with {outside} outside generals: ceiling {bound:?} < {} {:?}",
                point.hand,
                point.exposed,
                result.hu,
                result.layout
            );
            checked += 1;
        }
    }
    assert!(checked > 500, "{checked}");
}
