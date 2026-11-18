//! Where a winning hand's 胡 come from, slot by slot.
//!
//! A winning hand is 21 cards, so seven slots of three; at 1 胡 a slot that
//! is 7, and the other 3 must come from melds worth more than their cards.
//! Four `HeuristicBot`s play and every winning layout is counted by meld,
//! each with its excess `胡 − cards / 3`.
//!
//! ```sh
//! cargo run --release --example census -- [rounds] [seed]
//! ```
use std::collections::BTreeMap;
use tsap_oo::susik::{Meld, Solver};
use tsap_oo::{HeuristicBot, Rules, Seat, Step, Strategy, Table};

mod common;

const KINDS: [&str; 10] = [
    "對子",
    "車馬包",
    "碰",
    "將帥",
    "將士象",
    "三色兵卒",
    "暗刻",
    "四色兵卒",
    "明槓",
    "暗槓",
];

const fn kind(meld: &Meld, concealed: bool) -> usize {
    match meld {
        Meld::Pair(_) => 0,
        Meld::Field(_) => 1,
        Meld::Triplet(_) if !concealed => 2,
        Meld::Single(_) => 3,
        Meld::Court(_) => 4,
        Meld::Soldiers(colors) if colors.len() == 3 => 5,
        Meld::Triplet(_) => 6,
        Meld::Soldiers(_) => 7,
        Meld::Quad(_) if !concealed => 8,
        Meld::Quad(_) => 9,
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let rounds = args.next().flatten().unwrap_or(10_000);
    let seed = args.next().flatten().unwrap_or(0);
    let rules = Rules::new();
    let mut rich = Solver::new(rules.concealed, Some(0)).locked();

    let mut wins = 0u64;
    // Per kind: wins holding one, melds, melds dealt (concealed), thirds of excess
    let mut held = [0u64; 10];
    let mut melds = [0u64; 10];
    let mut dealt = [0u64; 10];
    let mut thirds = [0i64; 10];
    let mut by_hu = BTreeMap::<u32, u64>::new();
    let mut recipes = BTreeMap::<String, u64>::new();
    let mut singles_pairs = BTreeMap::<(u64, u64), u64>::new();
    // Dealt hands of twenty by the 胡 beyond 1 a meld of three: hands, wins
    let mut by_buff = [[0u64; 2]; 7];
    // By k = (singles - pairs) / 3: wins, and wins short of ten without 2k
    let mut by_k = BTreeMap::<i64, [u64; 2]>::new();
    // Dealt hands of twenty by generals: hands, wins
    let mut by_generals = [[0u64; 2]; 9];

    let played = common::deals(rules, rounds, seed, |round| {
        let hands = Seat::ALL.map(|seat| round.hand(seat));
        let bots =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        let mut table = Table::new(round, bots);
        while table.step().expect("bots act legally") != Step::Finished {}
        (hands, table.round().clone())
    });
    for (hands, round) in &played {
        let dealer = round.dealer();
        let buffs = Seat::ALL.map(|seat| {
            let layout = rich
                .layout(hands[seat.index()])
                .expect("loose cards are allowed");
            let buff: u32 = layout
                .melds
                .iter()
                .filter(|meld| meld.size() >= 3)
                .map(|meld| rules.concealed.score(meld) - 1)
                .sum();
            (buff as usize).min(6)
        });
        let generals = Seat::ALL.map(|seat| {
            let n: usize = hands[seat.index()]
                .distinct()
                .filter(|(card, _)| card.rank == tsap_oo::susik::Rank::General)
                .map(|(_, n)| usize::from(n))
                .sum();
            n.min(8)
        });
        rich.clear();
        let result = round.result().expect("finished");
        for seat in Seat::ALL {
            if seat != dealer {
                by_buff[buffs[seat.index()]][0] += 1;
                by_buff[buffs[seat.index()]][1] += u64::from(result.winner == Some(seat));
                by_generals[generals[seat.index()]][0] += 1;
                by_generals[generals[seat.index()]][1] += u64::from(result.winner == Some(seat));
            }
        }
        if let Some(winner) = result.winner {
            wins += 1;
            *by_hu.entry(result.hu).or_default() += 1;
            // Exposed melds, then the winning card's, then the concealed
            let shown = round.exposed(winner).len() + usize::from(result.winning_card.is_some());
            let mut count = [0u64; 10];
            let (mut cards, mut hu) = (0, 0);
            for (i, meld) in result.layout.iter().enumerate() {
                let concealed = i >= shown;
                let values = if concealed {
                    rules.concealed
                } else {
                    rules.exposed
                };
                let k = kind(meld, concealed);
                count[k] += 1;
                dealt[k] += u64::from(concealed);
                thirds[k] += 3 * i64::from(values.score(meld)) - meld.size() as i64;
                cards += meld.size();
                hu += values.score(meld);
            }
            assert_eq!((cards, hu), (21, result.hu), "{:?}", result.layout);
            for k in 0..10 {
                melds[k] += count[k];
                held[k] += u64::from(count[k] > 0);
            }
            let recipe: String = (4..10)
                .filter(|&k| count[k] > 0)
                .map(|k| format!("{}×{} ", KINDS[k], count[k]))
                .collect();
            *recipes.entry(recipe).or_default() += 1;
            let singles = count[3] + count[7] + count[8] + count[9];
            *singles_pairs.entry((singles, count[0])).or_default() += 1;
            // 胡 = 7 + 2k + 將士象 + 2 × (the slots of three worth 3) + what
            // the fourth card of a meld adds beyond a general's 1
            let k = (singles as i64 - count[0] as i64) / 3;
            let slots = count[5] + count[6] + count[7] + count[8] + count[9];
            let bonus = count[7] + 2 * count[8] + 4 * count[9];
            let sum = 7 + 2 * k + (count[4] + 2 * slots + bonus) as i64;
            assert_eq!(sum, i64::from(result.hu), "{:?}", result.layout);
            assert_eq!((singles as i64 - count[0] as i64) % 3, 0);
            let row = by_k.entry(k).or_default();
            row[0] += 1;
            row[1] += u64::from(k > 0 && sum - 2 * k < 10);
        }
    }

    let share = |n: u64, of: u64| 100.0 * n as f64 / of as f64;
    println!("{rounds} rounds, seed {seed}: {wins} wins");
    println!("\nkind      excess  wins with  melds/win  dealt/win  excess 胡/win");
    for k in 0..10 {
        println!(
            "{:<8}  {:+6.2}  {:8.2}%  {:9.3}  {:9.3}  {:+13.3}",
            KINDS[k],
            thirds[k] as f64 / 3.0 / melds[k].max(1) as f64,
            share(held[k], wins),
            melds[k] as f64 / wins as f64,
            dealt[k] as f64 / wins as f64,
            thirds[k] as f64 / 3.0 / wins as f64,
        );
    }
    println!("\nwinning 胡:");
    for (hu, &n) in &by_hu {
        println!("  {hu:2}  {:6.2}%", share(n, wins));
    }
    println!("\nmelds worth more than 1 胡 a slot:");
    let mut top: Vec<_> = recipes.iter().collect();
    top.sort_by_key(|&(_, &n)| std::cmp::Reverse(n));
    for (recipe, n) in top.into_iter().take(25) {
        let n = *n;
        println!("  {:6.2}%  {recipe}", share(n, wins));
    }
    println!("\n(generals + four-card melds, pairs):");
    for (&(singles, pairs), &n) in &singles_pairs {
        println!("  ({singles}, {pairs})  {:6.2}%", share(n, wins));
    }
    println!("\nk = (singles - pairs) / 3: wins, of which short of ten without 2k");
    for (k, &[n, needed]) in &by_k {
        println!(
            "  {k:+}  {:6.2}%  {:6.2}%",
            share(n, wins),
            share(needed, wins)
        );
    }
    println!("\ndealt generals (twenty cards): hands, win rate");
    let hands: u64 = by_generals.iter().map(|b| b[0]).sum();
    for (g, &[n, won]) in by_generals.iter().enumerate().filter(|(_, b)| b[0] > 0) {
        println!("  {g}  {:6.2}%  {:6.2}%", share(n, hands), share(won, n));
    }
    println!("\ndealt 胡 beyond 1 a meld of three (twenty cards): hands, win rate");
    let hands: u64 = by_buff.iter().map(|b| b[0]).sum();
    for (buff, &[n, won]) in by_buff.iter().enumerate() {
        println!(
            "  {buff}{}  {:6.2}%  {:6.2}%",
            if buff == 6 { "+" } else { " " },
            share(n, hands),
            share(won, n.max(1))
        );
    }
}
