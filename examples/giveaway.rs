//! What each discard gives the other seat at a table of two, beside what is
//! public about the card: the rows an inference for `danger` is fitted on
//! and checked against (`doc/opponents.md`, batch 3).
//!
//! `ExpectBot` as it ships plays `HeuristicBot`, each deal twice, once in
//! either seat.  At every discard it values in points, each card it could
//! put down is played on the round as it stands, as the `danger` oracle of
//! `ab/src/oracle.rs` plays it: whether the other seat wins on it and what
//! that settles at, and whether `HeuristicBot` there claims it and what the
//! DP in that seat makes of the claim over a pass.
//!
//! ```sh
//! cargo run --release --example giveaway -- [rounds] [seed] [rows]
//! ```
//!
//! It prints the tables of `src/giveaway.rs`: the mean, by a card's
//! [`Evidence`], of what the seat loses when the other wins on it and of
//! what the other gains by claiming it.  A cell of fewer than 1,000 cards
//! holds its row's mean.  With a third argument of `rows` it prints the
//! cards instead, as CSV, to look for evidence the tables do not hold.
//!
//! `GATE_CARDS` (default 10) is the bot's gate and `SHALLOW` (default 0) the
//! cards it plays two claims deep with past it: the defaults are the web
//! page's bot, and 14 and 17 the desktop's.  One row a card:
//!
//! | Column | |
//! | --- | --- |
//! | `deal`, `seat`, `decision` | Which discard |
//! | `card`, `rank` | `Card::index` and the rank, 1 (士) to 6 (soldiers) |
//! | `value`, `top` | The DP's value of the discard, and of its best |
//! | `win`, `score` | The other seat wins on it; this seat's settled score |
//! | `claim` | 1 claimed and valued, 2 claimed past the gate, 3 konged |
//! | `gain` | The DP's value of the claim over a pass, in the other seat |
//! | `opp_n`, `opp_hu`, `opp_melds` | The other seat's concealed cards, exposed 胡 and melds |
//! | `my_n`, `draws` | This seat's concealed cards and the draws left |
//! | `safe` | `View::safe` proves it does not win |
//! | `thrown` | The other seat's own discards of it from its hand |
//! | `pile`, `shown`, `mine` | Copies on the discard pile, in exposed melds, in this hand |
//! | `mates_thrown`, `mates_pile` | The kinds it makes a run with that the other seat threw, and that lie on the pile |
//! | `mates_unseen` | The copies of those kinds this seat has not seen |
use tsap_oo::susik::{Card, Hand, Rank};
use tsap_oo::{
    Claim, Eval, Event, Evidence, ExpectBot, Game, Giveaway, HeuristicBot, Phase, Round, Rules,
    Seat, Strategy, View,
};

/// A cell needs this many cards; a thinner one holds its row's mean
const LEAST: u64 = 1000;

/// A card's evidence, what discarding it gave away, and its row of the CSV
type Row = (Evidence, f64, String);

mod common;

/// Whether `a` and `b` go into one run: 將士象 or 車馬炮 of a colour, or
/// soldiers of two colours
fn mates(a: Card, b: Card) -> bool {
    let group = |rank| match rank {
        Rank::General | Rank::Advisor | Rank::Elephant => 0,
        Rank::Chariot | Rank::Horse | Rank::Cannon => 1,
        Rank::Soldier => 2,
    };
    a != b && group(a.rank) == group(b.rank) && ((a.rank == Rank::Soldier) != (a.color == b.color))
}

/// The rows of one discard of `me`'s: every card it may put down
fn record(
    bot: &mut ExpectBot,
    policy: &mut HeuristicBot,
    round: &Round,
    me: Seat,
    decision: usize,
    rows: &mut Vec<Row>,
) {
    let rules = round.rules();
    let (view, opp) = (View::new(round, me), me.next(rules.players));
    let ranked = bot.discards(&view);
    let top = ranked[0].1;
    let events = round.events();
    let (safe, thrown, giveaway) = (view.safe(opp), view.thrown(opp), Giveaway::new(&view));
    let pile: Hand = view.discards().iter().map(|&(_, card)| card).collect();
    let shown: Hand = Seat::ALL
        .into_iter()
        .flat_map(|seat| view.exposed(seat))
        .flat_map(|meld| meld.cards())
        .collect();
    let opp_hu: u32 = view
        .exposed(opp)
        .iter()
        .map(|meld| rules.exposed.score(meld))
        .sum();
    for (card, value) in ranked {
        let mut after = round.clone();
        after.discard(me, card).expect("a legal discard");
        let (mut win, mut score, mut claim, mut gain) = (0, 0, 0, 0.0);
        match after.events().get(events.len() + 1) {
            Some(&Event::Win { seat, .. }) if seat == opp => {
                let mut game = Game::new(*rules, after.dealer());
                game.settle(after.result().expect("the round is finished"));
                (win, score) = (1, game.scores()[me.index()]);
            }
            Some(&Event::Kong { seat, .. }) if seat == opp => claim = 3,
            _ => {}
        }
        if let (Phase::Claim(other), Some(offer)) = (after.phase(), after.offer())
            && other == opp
            && !offer.drawn
            && offer.source == me
        {
            let (theirs, options) = (View::new(&after, opp), after.claim_options(opp));
            let answer = policy.claim(&theirs, &options);
            if answer != Claim::Pass {
                claim = 2;
                if bot.searches(&theirs, theirs.hand().len()) {
                    let answers = bot.claims(&theirs, &options);
                    let of = |claim| answers.iter().find(|&&(c, _)| c == claim).map(|&(_, v)| v);
                    if let (Some(taken), Some(passed)) = (of(answer), of(Claim::Pass)) {
                        (claim, gain) = (1, taken - passed);
                    }
                }
            }
        }
        let run = || Card::ALL.into_iter().filter(move |&mate| mates(card, mate));
        let unseen = Hand::FULL - view.hand() - pile - shown;
        let cost = if win == 1 {
            (value - f64::from(score)).max(0.0)
        } else if claim == 1 {
            f64::max(gain, 0.0)
        } else {
            0.0
        };
        let row = format!(
            "{},{decision},{},{},{value:.3},{top:.3},{win},{score},{claim},{gain:.3},\
             {},{opp_hu},{},{},{},{},{},{},{},{},{},{},{}",
            me.index(),
            card.index(),
            card.rank as usize,
            view.concealed_len(opp),
            view.exposed(opp).len(),
            view.hand().len(),
            view.pile_len() - rules.dead(),
            u8::from(safe.contains(card)),
            thrown.count(card),
            pile.count(card),
            shown.count(card),
            view.hand().count(card),
            run().filter(|&mate| thrown.contains(mate)).count(),
            run().filter(|&mate| pile.contains(mate)).count(),
            run()
                .map(|mate| usize::from(unseen.count(mate)))
                .sum::<usize>(),
        );
        rows.push((giveaway.evidence(card), cost, row));
    }
}

/// `round` played out with `bot` in `me`'s seat against `HeuristicBot`
fn play(mut round: Round, me: Seat, mut bot: ExpectBot, rows: &mut Vec<Row>) {
    let mut policy = HeuristicBot::new(round.rules());
    let mut decision = 0;
    loop {
        match round.phase() {
            Phase::Finished => break,
            Phase::Claim(seat) => {
                let (view, options) = (View::new(&round, seat), round.claim_options(seat));
                let claim = if seat == me {
                    bot.claim(&view, &options)
                } else {
                    policy.claim(&view, &options)
                };
                round.claim(seat, claim).expect("bots act legally");
            }
            Phase::Discard(seat) => {
                let view = View::new(&round, seat);
                let card: Card = if seat == me {
                    if bot.searches(&view, view.hand().len() - 1) {
                        record(&mut bot, &mut policy, &round, me, decision, rows);
                        decision += 1;
                    }
                    bot.discard(&view)
                } else {
                    policy.discard(&view)
                };
                round.discard(seat, card).expect("bots act legally");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let number = |i: usize, default| {
        args.get(i)
            .map_or(default, |s| s.parse().expect("a number"))
    };
    let (rounds, seed) = (number(0, 1000), number(1, 0));
    let knob =
        |name, default| std::env::var(name).map_or(default, |s| s.parse().expect("a number"));
    let (gate, deep) = (knob("GATE_CARDS", 10), knob("SHALLOW", 0));
    let mut rules = Rules::new();
    rules.players = 2;
    let played = common::deals(rules, rounds, seed, |round| {
        let mut rows = Vec::new();
        for me in [Seat::ALL[0], Seat::ALL[1]] {
            let bot = ExpectBot::new(&rules)
                .gated(gate)
                .shallow(Eval::of(&rules), deep);
            play(round.clone(), me, bot, &mut rows);
        }
        rows
    });
    if args.get(2).is_some_and(|s| s == "rows") {
        println!(
            "deal,seat,decision,card,rank,value,top,win,score,claim,gain,\
             opp_n,opp_hu,opp_melds,my_n,draws,safe,thrown,pile,shown,mine,\
             mates_thrown,mates_pile,mates_unseen"
        );
        for (deal, rows) in played.iter().enumerate() {
            for (.., row) in rows {
                println!("{deal},{row}");
            }
        }
        return;
    }
    // (cards, cost): thrown, passed, and fresh by mates thrown and unseen
    let (mut thrown, mut passed, mut fresh) = ((0_u64, 0.0), (0_u64, 0.0), [[(0_u64, 0.0); 6]; 3]);
    for &(evidence, cost, _) in played.iter().flatten() {
        let cell = match evidence {
            Evidence::Thrown => &mut thrown,
            Evidence::Passed => &mut passed,
            Evidence::Fresh { thrown, unseen } => &mut fresh[thrown][unseen],
        };
        cell.0 += 1;
        cell.1 += cost;
    }
    println!(
        "{rounds} rounds each way, seed {seed}, gate {gate}, shallow {deep}: \
         the cost of a discard in points, then the cards"
    );
    let mean = |(n, cost): (u64, f64)| cost / n.max(1) as f64;
    println!("const THROWN: f64 = {:.3}; // {}", mean(thrown), thrown.0);
    println!("const PASSED: f64 = {:.3}; // {}", mean(passed), passed.0);
    println!("const FRESH: [[f64; 6]; 3] = [");
    for row in fresh {
        let all = row.iter().fold((0, 0.0), |a, c| (a.0 + c.0, a.1 + c.1));
        let costs: Vec<String> = row
            .iter()
            .map(|&cell| format!("{:.3}", mean(if cell.0 < LEAST { all } else { cell })))
            .collect();
        println!("    [{}],", costs.join(", "));
        let cards: Vec<String> = row.iter().map(|c| c.0.to_string()).collect();
        println!("    // {}", cards.join(", "));
    }
    println!("];");
}
