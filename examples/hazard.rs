//! How h bends with what the other seats hold, for `doc/opponents.md`: how
//! often another seat wins on a public card, by how many draws on from a
//! decision the card comes and by what the other seats were at the
//! decision, over how often one does whatever they were, at decisions with
//! as many draws left.
//!
//! Two keys, each read when a seat is to claim or to discard: the copies
//! of the cards another seat wins on that it has not seen, the most among
//! the other seats (a hand is read for it, so only an oracle has it), and
//! the fewest concealed cards another seat holds (public).  A card's lead
//! is the draws left at the decision less the card's segment, as
//! [`Outlook::end`](tsap_oo::Outlook) counts segments.
//!
//! ```sh
//! cargo run --release --example hazard -- [rounds] [seed]
//! ```
//!
//! `THRESHOLD` (default 10) is the 胡 to win and `PLAYERS` (default 4) the
//! table's size.  The last two blocks are the tables `ab/src/oracle.rs`
//! holds.
use tsap_oo::susik::Hand;
use tsap_oo::{Event, HeuristicBot, Phase, Round, Rules, Seat, Step, Strategy, Table};

mod common;

/// The waits are counted to this many, and more count as this many
const WAITS: usize = 4;

/// A ratio needs this many cards; a thinner cell holds the one before it
const LEAST: u64 = 1000;

/// Cards another seat won on, against the wins expected of cards at their
/// decisions' draws left and their leads
#[derive(Default, Clone, Copy)]
struct Count {
    n: u64,
    k: u64,
    expected: f64,
}

/// The copies of the cards `seat` wins on that it has not seen
fn waits(round: &Round, seat: Seat) -> usize {
    let hand = round.hand(seat);
    let shown = Seat::ALL
        .into_iter()
        .flat_map(|seat| round.exposed(seat))
        .flat_map(|meld| meld.cards())
        .chain(round.discards().iter().map(|&(_, card)| card))
        .chain(round.offer().map(|offer| offer.card));
    shown
        .fold(Hand::FULL - hand, Hand::without)
        .distinct()
        .filter(|&(card, _)| round.win(seat, hand, card).is_some())
        .map(|(_, n)| usize::from(n))
        .sum()
}

/// Each key's wins over the wins expected, by lead, as rows of a Rust table
fn print(title: &str, keys: &[Vec<Count>]) {
    println!("\n{title}: wins over expected by lead, then the cards");
    for (key, row) in keys.iter().enumerate() {
        if row.iter().all(|c| c.n < LEAST) {
            continue;
        }
        let mut last = 1.0;
        let ratios: Vec<String> = row
            .iter()
            .map(|c| {
                if c.n >= LEAST && c.expected > 0.0 {
                    last = c.k as f64 / c.expected;
                }
                format!("{last:.3}")
            })
            .collect();
        println!("    /* {key:2} */ [{}],", ratios.join(", "));
        let cards: Vec<String> = row.iter().map(|c| c.n.to_string()).collect();
        println!("    //        {}", cards.join(", "));
    }
}

fn main() {
    let mut args = std::env::args().skip(1).map(|s| s.parse::<u64>().ok());
    let rounds = args.next().flatten().unwrap_or(1000);
    let seed = args.next().flatten().unwrap_or(0);
    let mut rules = Rules::new();
    if let Ok(threshold) = std::env::var("THRESHOLD") {
        rules.threshold = threshold.parse().expect("THRESHOLD is a number");
    }
    if let Ok(players) = std::env::var("PLAYERS") {
        rules.players = players.parse().expect("PLAYERS is a number");
    }
    let span = 112 - (20 * rules.players + 1) - rules.dead();
    let played = common::deals(rules, rounds, seed, |round| {
        let bots =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        let mut table = Table::new(round, bots);
        // (who decides, events so far, draws left, waits, fewest cards)
        let mut decisions = Vec::new();
        loop {
            let round = table.round();
            if let Phase::Claim(seat) | Phase::Discard(seat) = round.phase() {
                let others = || {
                    Seat::ALL[..rules.players]
                        .iter()
                        .filter(move |&&s| s != seat)
                };
                decisions.push((
                    seat,
                    round.events().len(),
                    round.pile_len() - rules.dead(),
                    others().map(|&s| waits(round, s)).max().expect("a seat"),
                    others()
                        .map(|&s| round.hand(s).len())
                        .min()
                        .expect("a seat"),
                ));
            }
            let Step::Acted(_) = table.step().expect("bots act legally") else {
                break;
            };
        }
        (decisions, table.round().events().to_vec())
    });

    // Every decision's cards to come: (draws left, lead, another seat
    // wins, waits, fewest cards)
    let mut seen = Vec::new();
    for (decisions, events) in &played {
        // The public cards: (event, segment, who wins on it)
        let mut cards = Vec::new();
        let mut drawn = 0;
        for (i, &event) in events.iter().enumerate() {
            let draw = match event {
                Event::Draw { .. } => true,
                // A rejected draw is one card, logged twice
                Event::Discard { seat, card } => {
                    if i > 0 && events[i - 1] == (Event::Draw { seat, card }) {
                        continue;
                    }
                    false
                }
                _ => continue,
            };
            let winner = match events.get(i + 1) {
                Some(&Event::Win { seat, .. }) => Some(seat),
                _ => None,
            };
            cards.push((i, span - drawn, winner));
            drawn += usize::from(draw);
        }
        for &(seat, from, draws, waits, fewest) in decisions {
            for &(_, segment, winner) in cards.iter().filter(|&&(i, ..)| i >= from) {
                let hit = winner.is_some_and(|s| s != seat);
                seen.push((draws, draws - segment, hit, waits.min(WAITS), fewest));
            }
        }
    }
    // h by the decision's draws left and the lead, whatever the seats hold
    let mut all = vec![vec![(0_u64, 0_u64); span + 1]; span + 1];
    for &(draws, lead, hit, ..) in &seen {
        all[draws][lead].0 += 1;
        all[draws][lead].1 += u64::from(hit);
    }
    let mut by_waits = vec![vec![Count::default(); span + 1]; WAITS + 1];
    let mut by_cards = vec![vec![Count::default(); span + 1]; 22];
    for &(draws, lead, hit, waits, fewest) in &seen {
        let (n, k) = all[draws][lead];
        for count in [&mut by_waits[waits][lead], &mut by_cards[fewest][lead]] {
            count.n += 1;
            count.k += u64::from(hit);
            count.expected += k as f64 / n as f64;
        }
    }

    println!(
        "{rounds} rounds of HeuristicBot self-play, seed {seed}, {} players, {} to win",
        rules.players, rules.threshold
    );
    print(
        &format!("by the waits of another seat, 0 to {WAITS} or more"),
        &by_waits,
    );
    print("by the fewest concealed cards of another seat", &by_cards);
}
