//! The return match of `guest`: [`ExpectBot`] as the guest at
//! [chunhua1972/four-color-game](https://github.com/chunhua1972/four-color-game)'s
//! table (`ee33cf0`), under its rules and its scores.
//!
//! ```console
//! git clone https://github.com/chunhua1972/four-color-game ~/four-color-game
//! cargo run --release --example away -- ~/four-color-game [deals] [seed]
//! ```
//!
//! `node` (24 or later) runs their engine from the clone through `away.mjs`,
//! which plays each deal four ways as `guest` does and prints the report;
//! this file only answers for the bot.  It looks at their table through
//! [`Round::seen`] and ranks its answers as at home; the first one their
//! engine allows is played.  So the bot keeps its own beliefs, wrong there:
//! that a pair of soldiers is a meld, that the card just claimed is locked,
//! that a win pays its 胡 (theirs pays 1 and each 胡 over 10, and a flower
//! card's copies), and that a general kong is worth 8 concealed and 6
//! exposed.  The pile is drawn to its end there, so [`Rules::dead_tail`] is
//! 0.  Wins and kongs, compulsory at home, `away.mjs` always takes.
//!
//! `deals` defaults to 10000 and `seed` to the clock.

use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{Command, Stdio};
use tsap_oo::susik::{Card, ColorSet, Hand, Meld, Rank};
use tsap_oo::{Claim, ClaimOptions, ExpectBot, Offer, Round, Rules, Seat, View};

fn cards(csv: &str) -> Vec<Card> {
    let indices = csv.split(',').filter(|s| !s.is_empty());
    indices
        .map(|s| Card::from_index(s.parse().unwrap()))
        .collect()
}

/// The meld of these cards, which their engine vouches for
fn meld(csv: &str) -> Meld {
    let cards = cards(csv);
    let first = cards[0];
    match (cards.len(), first.rank) {
        (1, _) => Meld::Single(first),
        (2, _) => Meld::Pair(first),
        (4, _) if cards[1] == first => Meld::Quad(first),
        (_, Rank::Soldier) if cards[1] != first => {
            let mut colors = ColorSet::EMPTY;
            for card in cards {
                colors.insert(card.color);
            }
            Meld::Soldiers(colors)
        }
        _ if cards[1] == first => Meld::Triplet(first),
        (_, Rank::General) => Meld::Court(first.color),
        _ => Meld::Field(first.color),
    }
}

/// The bot's answer to a request of `away.mjs`
fn answer(bot: &mut ExpectBot, rules: Rules, request: &str) -> i32 {
    let f: Vec<_> = request.split(' ').collect();
    let seat = |s: &str| Seat::ALL[s.parse::<usize>().unwrap()];
    let exposed: Vec<Vec<Meld>> = f[7]
        .split('|')
        .map(|melds| {
            let melds = melds.split(';').filter(|s| !s.is_empty());
            melds.map(meld).collect()
        })
        .collect();
    let discards = f[6].split(',').filter(|s| !s.is_empty()).map(seat);
    let discards = discards.zip(cards(f[5])).collect();
    let offer = (f[0] == "c").then(|| Offer {
        card: Card::from_index(f[8].parse().unwrap()),
        source: seat(f[10]),
        drawn: f[9] == "1",
    });
    let round = Round::seen(
        rules,
        seat(f[2]),
        seat(f[1]),
        cards(f[4]).into_iter().collect::<Hand>(),
        exposed.try_into().expect("four seats"),
        discards,
        f[3].parse().unwrap(),
        offer,
    );
    let view = View::new(&round, seat(f[1]));
    if offer.is_none() {
        // Nothing is locked in a position seen, so every card their engine
        // lets go is ranked.
        return bot.discards(&view)[0].0.index() as i32;
    }
    let claims: Vec<_> = f[11]
        .split(';')
        .map(|claim| match claim.split_at(1) {
            ("p", _) => Claim::Pong,
            (_, cards) => Claim::Eat(meld(cards)),
        })
        .collect();
    let options = ClaimOptions {
        pong: claims.contains(&Claim::Pong),
        eats: claims
            .iter()
            .filter_map(|claim| match claim {
                Claim::Eat(meld) => Some(*meld),
                _ => None,
            })
            .collect(),
        blocked: Vec::new(),
    };
    // Their engine lets a seat pass a general another seat drew; the bot,
    // which never may at home, eats it.
    let (best, _) = bot.claims(&view, &options)[0];
    claims
        .iter()
        .position(|&claim| claim == best)
        .map_or(-1, |i| i as i32)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let clone = args.next().expect("the path to a four-color-game clone");
    let deals = args.next().unwrap_or_else(|| "10000".into());
    let base = args.next().unwrap_or_else(|| {
        let now = std::time::SystemTime::now();
        let since = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        since.as_secs().to_string()
    });
    let mut node = Command::new("node")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/away.mjs"))
        .args([clone, deals, base])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node is installed");
    let mut stdin = node.stdin.take().unwrap();
    let stdout = BufReader::new(node.stdout.take().unwrap());

    let mut rules = Rules::new();
    rules.dead_tail = 0;
    let mut bot = ExpectBot::new(&rules);
    for line in stdout.lines() {
        let line = line.expect("away.mjs writes text");
        match line.strip_prefix("? ") {
            Some(request) => {
                writeln!(stdin, "{}", answer(&mut bot, rules, request)).expect("away.mjs listens");
            }
            None => println!("{line}"),
        }
    }
    assert!(node.wait().expect("away.mjs was running").success());
}
