//! Every pair the desktop [`ExpectBot`] may eat, by what the eat does to the
//! hand's [`ceiling`] with no general from outside and with one, and what
//! the bot makes of it.  Seat 1 plays it among three [`HeuristicBot`]s, the
//! A/B table.  One CSV row a decision on stdout, and on stderr one for each
//! hand a discard leaves waiting: what it waits on, the DP's chance to win
//! and whether it won.
//!
//! ```sh
//! cargo run --release --example overspend -- [rounds] [seed base] > pairs.csv 2> waiting.csv
//! ```
//!
//! `research/overspend.py` makes the tables of the two files.
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::rc::Rc;
use tsap_oo::susik::{Card, Hand, Meld, Rank, Solver, locked_completions};
use tsap_oo::{
    Claim, ClaimOptions, Eval, ExpectBot, Graph, HeuristicBot, Outlook, Payoff, Prune, Round,
    Rules, Seat, Step, Strategy, Table, View, ceiling, discardable,
};

struct Logged {
    bot: ExpectBot,
    win: Solver,
    log: Rc<RefCell<Vec<String>>>,
    /// A row for each hand left waiting by a discard
    waiting: Rc<RefCell<Vec<String>>>,
}

impl Logged {
    /// Unseen cards `hand` wins on beside `base`: generals, which any seat's
    /// draw brings and no discard does, and the rest
    fn waits(&mut self, rules: &Rules, hand: Hand, base: u32, live: Hand) -> (u32, u32) {
        let (mut generals, mut others) = (0, 0);
        for (card, n) in live.distinct() {
            let wins = locked_completions(hand, card).iter().any(|meld| {
                self.win
                    .best(hand - meld.cards().without(card))
                    .and_then(|rest| u32::try_from(rest).ok())
                    .is_some_and(|rest| base + rules.exposed.score(meld) + rest >= rules.threshold)
            });
            if wins && card.rank == Rank::General {
                generals += u32::from(n);
            } else if wins {
                others += u32::from(n);
            }
        }
        (generals, others)
    }
}

impl Strategy for Logged {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let answers = self.bot.claims(view, options);
        let played = answers[0].0;
        let offer = view.offer().expect("an offer");
        let pair = Claim::Eat(Meld::Pair(offer.card));
        if options.forced() || !options.eats.contains(&Meld::Pair(offer.card)) {
            return played;
        }
        let rules = view.rules();
        let (hand, exposed) = (view.hand(), view.exposed(view.seat()));
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card))
            .chain([offer.card]);
        let live = shown.fold(Hand::FULL - hand, Hand::without);
        let base: u32 = exposed.iter().map(|meld| rules.exposed.score(meld)).sum();
        let eaten = [exposed, &[Meld::Pair(offer.card)]].concat();
        let rest = hand.without(offer.card);
        let top = |exposed: &[Meld], hand, g| {
            ceiling(rules, exposed, hand, live, g).map_or(-1, |hu| hu as i32)
        };
        let value = |want: &dyn Fn(Claim) -> bool| {
            answers
                .iter()
                .filter(|a| want(a.0))
                .map(|a| a.1)
                .fold(f64::NEG_INFINITY, f64::max)
        };
        let kings: u32 = live
            .distinct()
            .filter(|(card, _)| card.rank == Rank::General)
            .map(|(_, n)| u32::from(n))
            .sum();
        let pass = self.waits(rules, hand, base, live);
        // The discard after the eat that waits on the most
        let eat = discardable(rest, Some(offer.card))
            .distinct()
            .map(|(card, _)| self.waits(rules, rest.without(card), base, live))
            .max_by_key(|&(g, o)| g + o)
            .unwrap_or((0, 0));
        self.log.borrow_mut().push(format!(
            "{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{},{},{},{},{},{},{},{}{}",
            hand.len(),
            base,
            view.pile_len().saturating_sub(rules.dead()),
            live.len(),
            kings,
            top(exposed, hand, 0),
            top(exposed, hand, 1),
            top(&eaten, rest, 0),
            top(&eaten, rest, 1),
            match played {
                p if p == pair => "pair",
                Claim::Pass => "pass",
                _ => "other",
            },
            value(&|c| c == pair),
            value(&|c| c == Claim::Pass),
            value(&|c| c != pair && c != Claim::Pass),
            pass.0,
            pass.1,
            eat.0,
            eat.1,
            u8::from(offer.source == view.seat()),
            hand,
            exposed
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" "),
            offer.card.color.name(),
            offer.card,
        ));
        played
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let card = self.bot.discard(view);
        let rules = view.rules();
        let hand = view.hand().without(card);
        let exposed = view.exposed(view.seat());
        let shown = Seat::ALL
            .into_iter()
            .flat_map(|seat| view.exposed(seat))
            .flat_map(|meld| meld.cards())
            .chain(view.discards().iter().map(|&(_, card)| card));
        let live = shown.fold(Hand::FULL - view.hand(), Hand::without);
        let base: u32 = exposed.iter().map(|meld| rules.exposed.score(meld)).sum();
        let (generals, others) = self.waits(rules, hand, base, live);
        if generals + others == 0 {
            return card;
        }
        let draws = view.pile_len().saturating_sub(rules.dead());
        // The DP's chance to win from here, where the bot plays by it whole
        let chance = if hand.len() <= 14 {
            // `ExpectBot::new`'s at four players and ten 胡
            let outlook = Outlook::default();
            let graph = Graph::new(rules, live, &[(hand, base)], Prune::default());
            graph.values(&outlook, draws, Payoff::Win)[graph.roots()[0]]
        } else {
            f64::NAN
        };
        let kings: u32 = live
            .distinct()
            .filter(|(card, _)| card.rank == Rank::General)
            .map(|(_, n)| u32::from(n))
            .sum();
        self.waiting.borrow_mut().push(format!(
            "{},{},{},{},{},{},{},{:.4}",
            hand.len(),
            base,
            draws,
            live.len(),
            kings,
            generals,
            others,
            chance
        ));
        card
    }
}

/// Play round `seed`: a row for each pair seat 1 might eat, and one for
/// each hand it left waiting
fn play(seed: u64) -> (Vec<String>, Vec<String>) {
    let rules = Rules::new();
    let mut rng = SmallRng::seed_from_u64(seed);
    let round = Round::deal(rules, Seat::ALL[(seed % 4) as usize], &mut rng);
    let log = Rc::<RefCell<Vec<String>>>::default();
    let waiting = Rc::<RefCell<Vec<String>>>::default();
    let mut seats =
        Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
    seats[1] = Some(Box::new(Logged {
        bot: ExpectBot::new(&rules)
            .gated(14)
            .shallow(Eval::of(&rules), 17),
        win: Solver::new(rules.concealed, None).locked(),
        log: Rc::clone(&log),
        waiting: Rc::clone(&waiting),
    }));
    let mut table = Table::new(round, seats);
    assert_eq!(table.run(), Ok(Step::Finished));
    let result = table.round().result().expect("finished");
    let won = result.winner == Some(Seat::ALL[1]);
    let end = format!(",{},{}", u8::from(won), u8::from(result.winner.is_some()));
    drop(table);
    let rows = |log: Rc<RefCell<Vec<String>>>| {
        log.take()
            .into_iter()
            .map(|row| format!("{seed},{row}{end}"))
            .collect()
    };
    (rows(log), rows(waiting))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut arg = |default: u64| args.next().and_then(|s| s.parse().ok()).unwrap_or(default);
    let (rounds, base) = (arg(100), arg(0));
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()) as u64;
    let rows: Vec<(Vec<String>, Vec<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    (t..rounds)
                        .step_by(threads as usize)
                        .map(|i| play(base + i))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a round panicked"))
            .collect()
    });
    println!(
        "seed,cards,base,draws,live,kings,c0b,c1b,c0a,c1a,played,v_pair,v_pass,v_other,\
         pass_g,pass_o,eat_g,eat_o,own,hand,exposed,offer,won,ended"
    );
    eprintln!("seed,cards,base,draws,live,kings,generals,others,chance,won,ended");
    for (pairs, waiting) in rows {
        for row in pairs {
            println!("{row}");
        }
        for row in waiting {
            eprintln!("{row}");
        }
    }
}
