//! How big the single-player DP of `doc/stronger-bot.md` (route 3) would
//! be: from every settled hand of `HeuristicBot` self-play, the sub-hands a
//! chain of claims and discards could reach while [`ceiling`] still reaches
//! the threshold, and the edges between them.
//!
//! A node is (concealed hand, 胡 still needed); the cards not yet seen are
//! fixed at the root, as the DP would fix them.  An edge claims one such
//! card into a meld and discards one card.  A node is kept when the ceiling
//! with one general from outside reaches the threshold; those that reach it
//! with none are counted too.
//!
//! ```sh
//! cargo run --release --example nodes -- [rounds] [seed] [most cards in hand] [prune] [exact]
//! ```
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use tsap_oo::susik::{Card, Hand, Meld, Rank, Solver, locked_completions};
use tsap_oo::{
    Game, HeuristicBot, Phase, Rules, Seat, Step, Strategy, Table, View, ceiling, discardable,
};

/// Nodes kept with one general from outside, nodes that need none, edges
/// Whether two loose cards wait on a third to make a meld
fn mates(a: Card, b: Card) -> bool {
    let court = |rank| matches!(rank, Rank::General | Rank::Advisor | Rank::Elephant);
    match (a.rank, b.rank) {
        (Rank::Soldier, Rank::Soldier) => a.color != b.color,
        (Rank::Soldier, _) | (_, Rank::Soldier) => false,
        (x, y) => a.color == b.color && x != y && court(x) == court(y),
    }
}

/// The discards a node tries.  `prune` 0: all of them.  1: only the cards
/// loose in one of the two layouts `HeuristicBot` grades, when there are
/// any.  2: of those, only the ones no other loose card goes with, when
/// there are any.  3: as 2, but two loose cards go together only while a
/// card that finishes them is live.  A hand left with `exact` cards or fewer tries them all.
fn discards(
    solvers: &mut [Solver; 2],
    rest: Hand,
    locked: Card,
    prune: u64,
    exact: usize,
    live: Hand,
) -> Hand {
    let all = discardable(rest, Some(locked));
    if prune == 0 || rest.len() <= exact + 1 {
        return all;
    }
    let mut loose = Hand::EMPTY;
    for solver in solvers {
        loose += solver.layout(rest).expect("loose layouts exist").loose;
    }
    let free: Hand = all.iter().filter(|&card| loose.contains(card)).collect();
    if free.is_empty() {
        return all;
    }
    let alone: Hand = free
        .iter()
        .filter(|&card| {
            !loose.iter().any(|other| {
                mates(card, other)
                    && (prune == 2
                        || live
                            .distinct()
                            .any(|(third, _)| mates(card, third) && mates(other, third)))
            })
        })
        .collect();
    if prune == 1 || alone.is_empty() {
        free
    } else {
        alone
    }
}

fn census(
    rules: &Rules,
    exposed: &[Meld],
    hand: Hand,
    live: Hand,
    prune: u64,
    exact: usize,
) -> [u64; 3] {
    let mut solvers = [1, 16].map(|cost| Solver::new(rules.concealed, Some(cost)).locked());
    let reaches = |exposed: &[Meld], hand, generals| {
        ceiling(rules, exposed, hand, live, generals).is_some_and(|hu| hu >= rules.threshold)
    };
    let base =
        |exposed: &[Meld]| -> u32 { exposed.iter().map(|meld| rules.exposed.score(meld)).sum() };
    // By (hand, exposed 胡): kept with whether it needs no general, or dropped
    let mut seen = HashMap::new();
    let mut queue = VecDeque::new();
    let mut edges = 0;
    if reaches(exposed, hand, 1) {
        seen.insert((hand, base(exposed)), Some(reaches(exposed, hand, 0)));
        queue.push_back((hand, exposed.to_vec()));
    }
    while let Some((hand, exposed)) = queue.pop_front() {
        for (card, _) in live.distinct() {
            for meld in locked_completions(hand, card) {
                if matches!(meld, Meld::Pair(_)) && card.rank == Rank::General {
                    continue;
                }
                let rest = hand - meld.cards().without(card);
                let mut shown = exposed.clone();
                shown.push(meld);
                for (discard, _) in
                    discards(&mut solvers, rest, card, prune, exact, live).distinct()
                {
                    let next = rest.without(discard);
                    let kept = *seen.entry((next, base(&shown))).or_insert_with(|| {
                        reaches(&shown, next, 1).then(|| {
                            queue.push_back((next, shown.clone()));
                            reaches(&shown, next, 0)
                        })
                    });
                    edges += u64::from(kept.is_some());
                }
            }
        }
    }
    let kept = seen.values().flatten();
    let sure = kept.clone().filter(|&&sure| sure).count() as u64;
    [kept.count() as u64, sure, edges]
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let rounds = args.next().flatten().unwrap_or(100);
    let seed = args.next().flatten().unwrap_or(0);
    let most = args.next().flatten().unwrap_or(20) as usize;
    let prune = args.next().flatten().unwrap_or(0);
    let exact = args.next().flatten().unwrap_or(0) as usize;
    let rules = Rules::new();
    let mut game = Game::new(rules, Seat::ALL[0]);
    let mut rng = SmallRng::seed_from_u64(seed);
    // By cards in hand: points, nodes, sure nodes, edges, most nodes, most
    // edges, nanoseconds
    let mut by_size = [[0u64; 7]; 21];

    for _ in 0..rounds {
        let round = game.deal(&mut rng);
        let bots =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        let mut table = Table::new(round, bots);
        loop {
            let step = table.step().expect("bots act legally");
            let Step::Acted(seat) = step else { break };
            let round = table.round();
            let view = View::new(round, seat);
            let hand = view.hand();
            if round.phase() == Phase::Discard(seat) || hand.len() > most {
                continue;
            }
            let shown = Seat::ALL
                .into_iter()
                .flat_map(|seat| view.exposed(seat))
                .flat_map(|meld| meld.cards())
                .chain(view.discards().iter().map(|&(_, card)| card))
                .chain(view.offer().map(|offer| offer.card));
            let live = shown.fold(Hand::FULL - hand, Hand::without);
            let start = Instant::now();
            let [nodes, sure, edges] = census(&rules, view.exposed(seat), hand, live, prune, exact);
            let row = &mut by_size[hand.len()];
            row[0] += 1;
            row[1] += nodes;
            row[2] += sure;
            row[3] += edges;
            row[4] = row[4].max(nodes);
            row[5] = row[5].max(edges);
            row[6] += start.elapsed().as_nanos() as u64;
        }
        let result = table.round().result().expect("finished").clone();
        game.settle(&result);
    }

    println!(
        "{rounds} rounds, seed {seed}, prune {prune} above {exact} cards: nodes reachable with the ceiling at the threshold"
    );
    println!("\ncards  points  nodes/pt  sure/pt  edges/pt  max nodes  max edges  ms/pt");
    let mut total = [0u64; 7];
    for (size, row) in by_size.iter().enumerate().rev() {
        if row[0] == 0 {
            continue;
        }
        for (t, r) in total.iter_mut().zip(row) {
            *t += r;
        }
        total[4] = total[4].max(row[4]);
        total[5] = total[5].max(row[5]);
        print(&size.to_string(), row);
    }
    print("all", &total);
}

fn print(label: &str, row: &[u64; 7]) {
    let per = |n: u64| n as f64 / row[0] as f64;
    println!(
        "{label:>5}  {:6}  {:8.1}  {:7.1}  {:8.1}  {:9}  {:9}  {:6.2}",
        row[0],
        per(row[1]),
        per(row[2]),
        per(row[3]),
        row[4],
        row[5],
        per(row[6]) / 1e6,
    );
}
