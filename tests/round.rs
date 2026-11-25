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

/// The 點花 is the top of the pile, turned where it lies: three cards like
/// it in the winner's 33 胡 are three 花胡, and with a 底 of 1 each seat
/// pays 1 + 23 + 3.
#[test]
fn a_base_and_a_flower_pay() {
    let dream = hand("帥帥帥帥仕仕仕仕相相相相.帥帥帥仕仕仕相相相..");
    let filler = hand("俥俥俥俥傌傌傌傌炮炮炮炮兵兵兵兵.帥俥俥俥..");
    let more = hand(".傌傌傌傌炮炮炮炮兵兵兵兵.將將將將士士士士.");
    let rest = hand("..象象象象車車車車馬馬馬馬包包包包卒卒卒卒.");
    let mut pile = pile([dream, filler, more, rest]);
    // The 31 cards left after the deal lie first, the last of them on top.
    let top = 30;
    let held = pile[..=top].iter().position(|&c| dream.count(c) == 3);
    pile.swap(held.unwrap(), top);
    let flower = pile[top];

    let mut rules = Rules::new();
    (rules.base, rules.flower) = (Some(1), true);
    let round = Round::from_pile(rules, THIRD, pile.clone()).unwrap();
    let result = round.result().unwrap();
    assert_eq!(result.hu, 33);
    assert_eq!((result.flower, result.flower_hu()), (Some(flower), 3));
    conserved(&round);
    let mut game = tsap_oo::Game::new(rules, THIRD);
    game.settle(result);
    assert_eq!(game.scores(), [-27, -27, 81, -27]);

    // Off, as in `Rules::new`: no card is turned and the whole 胡 is paid.
    let round = Round::from_pile(Rules::new(), THIRD, pile).unwrap();
    let result = round.result().unwrap();
    assert_eq!((result.flower, result.flower_hu()), (None, 0));
    let mut game = tsap_oo::Game::new(Rules::new(), THIRD);
    game.settle(result);
    assert_eq!(game.scores(), [-33, -33, 99, -33]);
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
    use tsap_oo::{
        Eval, Event, Evidence, ExpectBot, Game, Giveaway, HeuristicBot, Phase, Round, Rules, Seat,
        Step, Strategy, Table, View,
    };

    /// Bots play out whole rounds; cards are conserved after every step,
    /// every win reaches the threshold, and none comes on a card the winner
    /// was proven not to win on.
    #[test]
    fn bots_finish_rounds() {
        playout(Rules::new());
    }

    /// The same under 8 胡: nothing in the engine assumes ten.
    #[test]
    fn bots_finish_rounds_of_eight() {
        playout(Rules::eight());
    }

    /// The same at tables of three and two: twenty each still, the seats
    /// beyond the table never act and never pay.
    #[test]
    fn bots_finish_rounds_at_smaller_tables() {
        for players in [3, 2] {
            let mut rules = Rules::new();
            rules.players = players;
            playout(rules);
        }
    }

    /// The same with a 底 and a 點花: the settlement alone changes.
    #[test]
    fn bots_finish_rounds_with_a_base_and_a_flower() {
        let mut rules = Rules::new();
        (rules.base, rules.flower) = (Some(1), true);
        playout(rules);
    }

    /// The same with an `ExpectBot` in the second seat, which plays by the
    /// DP once its hand is short and by a graph one claim deep before
    #[test]
    fn expect_bot_finishes_rounds() {
        let rules = Rules::new();
        std::thread::scope(|scope| {
            for seed in 0..4 {
                scope.spawn(move || game(rules, 4052 + seed, 4, true));
            }
        });
    }

    /// A wary `ExpectBot` plays a table of two out.  What a seat threw is
    /// what left its hand but into its melds, a rejected draw never among
    /// it, and the next seat reads every card on the pile as let go.
    #[test]
    fn thrown_cards_left_the_hand() {
        let mut rules = Rules::new();
        rules.players = 2;
        let mut rng = SmallRng::seed_from_u64(1125);
        for _ in 0..8 {
            let round = Round::deal(rules, FIRST, &mut rng);
            let dealt = Seat::ALL.map(|seat| round.hand(seat));
            let mut bots =
                Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
            bots[1] = Some(Box::new(
                ExpectBot::new(&rules).shallow(Eval::of(&rules), 0).wary(),
            ));
            let mut table = Table::new(round, bots);
            while table.step().unwrap() != Step::Finished {
                let round = table.round();
                for &seat in &Seat::ALL[..2] {
                    let view = View::new(round, seat.next(2));
                    let thrown = view.thrown(seat);
                    let mut left = round.hand(seat) + thrown;
                    for &event in round.events() {
                        let (meld, card) = match event {
                            Event::Eat {
                                seat: s,
                                meld,
                                card,
                            } if s == seat => (meld, card),
                            Event::Pong { seat: s, card } if s == seat => {
                                (Meld::Triplet(card), card)
                            }
                            Event::Kong { seat: s, card } if s == seat => (Meld::Quad(card), card),
                            _ => continue,
                        };
                        left += meld.cards().without(card);
                    }
                    assert_eq!(left, dealt[seat.index()]);
                    let giveaway = Giveaway::new(&view);
                    for &(_, card) in round.discards() {
                        let evidence = giveaway.evidence(card);
                        assert_eq!(evidence == Evidence::Thrown, thrown.contains(card));
                        assert!(matches!(evidence, Evidence::Thrown | Evidence::Passed));
                    }
                }
            }
        }
    }

    /// Eight games of 25 rounds, a thread each
    fn playout(rules: Rules) {
        let [wins, proven] = std::thread::scope(|scope| {
            let games: Vec<_> = (0..8)
                .map(|seed| scope.spawn(move || game(rules, 2026 + seed, 25, false)))
                .collect();
            games.into_iter().fold([0, 0], |[wins, proven], game| {
                let [w, p] = game.join().expect("a game panicked");
                [wins + w, proven + p]
            })
        });
        assert!(wins > 0, "the bots never won");
        assert!(proven > 0, "no card was ever proven safe");
    }

    /// Wins and the cards proven safe over `rounds`, with an `ExpectBot`
    /// in the second seat if `expect`
    fn game(rules: Rules, seed: u64, rounds: usize, expect: bool) -> [usize; 2] {
        let mut game = Game::new(rules, FIRST);
        let mut rng = SmallRng::seed_from_u64(seed);
        let (mut wins, mut exhausted, mut fouls, mut proven) = (0, 0, 0, 0);
        for _ in 0..rounds {
            let mut bots =
                Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
            if expect {
                bots[1] = Some(Box::new(
                    ExpectBot::new(&rules).shallow(Eval::of(&rules), 0),
                ));
            }
            let mut table = Table::new(game.deal(&mut rng), bots);
            for &seat in &Seat::ALL[rules.players..] {
                assert!(table.round().hand(seat).is_empty());
            }
            let mut safe = Seat::ALL.map(|_| susik::Hand::EMPTY);
            loop {
                let round = table.round();
                if round.phase() != Phase::Finished {
                    safe = Seat::ALL.map(|seat| View::new(round, seat).safe(seat));
                }
                match table.step().unwrap() {
                    Step::Acted(seat) => {
                        assert!(
                            seat.index() < rules.players,
                            "a seat beyond the table acted"
                        );
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
            // Every table draws 25 cards at most, and only those exhaust it.
            let events = table.round().events();
            let draws = events
                .iter()
                .filter(|event| matches!(event, Event::Draw { .. }))
                .count();
            let drawn_out = table.round().pile_len() == rules.dead();
            assert!(draws <= 25 && table.round().pile_len() >= rules.dead());
            assert_eq!(result.winner.is_none() && result.foul.is_none(), {
                drawn_out && draws == 25 && events.last() == Some(&Event::Exhausted)
            });
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
                    // The 點花 is turned only under its rule; each other
                    // seat pays the 胡 as the rules have it, and the 花胡.
                    assert_eq!(result.flower.is_some(), rules.flower);
                    let each = rules.pays(result.hu) + result.flower_hu();
                    assert!(each <= rules.pays(result.hu) + 3);
                    let before = game.scores()[winner.index()];
                    game.settle(result);
                    let won = (rules.players - 1) * each as usize;
                    assert_eq!(game.scores()[winner.index()] - before, won as i32);
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
        assert_eq!(wins + exhausted + fouls, rounds);
        assert_eq!(game.scores().iter().sum::<i32>(), 0);
        assert!(
            game.scores()[rules.players..]
                .iter()
                .all(|&score| score == 0)
        );
        [wins, proven]
    }
}

/// The hand of each empty seat is dead with the six, so that every table
/// has 25 cards to draw.
#[test]
fn an_empty_seat_leaves_its_hand_dead() {
    for (players, dead) in [(4, 6), (3, 26), (2, 46)] {
        let mut rules = Rules::new();
        rules.players = players;
        assert_eq!(rules.dead(), dead, "{players} players");
        let round = Round::from_pile(rules, FIRST, Hand::FULL.iter().collect()).unwrap();
        assert_eq!(round.pile_len() - rules.dead(), 25, "{players} players");
    }
}

/// A table is 2 to 4 players, and the dealer sits at it.
#[test]
fn a_deal_needs_a_table() {
    let deck = || Hand::FULL.iter().collect::<Vec<_>>();
    for (players, dealer, ok) in [
        (1, FIRST, false),
        (2, SECOND, true),
        (2, THIRD, false),
        (5, FIRST, false),
    ] {
        let mut rules = Rules::new();
        rules.players = players;
        assert_eq!(
            Round::from_pile(rules, dealer, deck()).is_ok(),
            ok,
            "{players} players"
        );
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
    let check = |i: usize, round: Round| {
        let mut checked = 0;
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
            return 0;
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
        checked
    };
    let rounds: Vec<_> = (0..100)
        .map(|i| (i, Round::deal(rules, Seat::ALL[i % Seat::COUNT], &mut rng)))
        .collect();
    let checked: usize = std::thread::scope(|scope| {
        let shards: Vec<_> = rounds
            .chunks(13)
            .map(|shard| {
                let check = &check;
                scope.spawn(move || {
                    shard
                        .iter()
                        .map(|(i, round)| check(*i, round.clone()))
                        .sum::<usize>()
                })
            })
            .collect();
        shards
            .into_iter()
            .map(|shard| shard.join().expect("a round panicked"))
            .sum()
    });
    assert!(checked > 500, "{checked}");
}
