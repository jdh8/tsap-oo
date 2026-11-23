//! A match against another program's bot: the Normal AI of
//! [chunhua1972/four-color-game](https://github.com/chunhua1972/four-color-game)
//! (`research/public-opponents.md`), checked at `ee33cf0`.
//!
//! ```console
//! git clone https://github.com/chunhua1972/four-color-game ~/four-color-game
//! cargo run --release --example guest -- ~/four-color-game [deals] [seed] [expect]
//! ```
//!
//! Their repository states no licence, so none of it is copied here: `node`
//! (24 or later) imports their `chooseAiCommand` from the clone through
//! `guest.mjs`, and this file hands it decisions.  [`Round`] stays the only
//! authority on the rules.  The guest chooses among the claims and discards
//! this engine allows, and never sees a win or a kong, which are compulsory
//! here.  Its hand grade is its own, tuned to its own rules (no pair of
//! soldiers, a general kong of 6 concealed and 8 exposed), so this measures
//! the guest as adapted to this table, not its strength at home.
//!
//! Each deal is played four ways and scored from seat 1 through a fresh
//! [`Game::settle`], the dealer rotating: four hosts, the guest among three
//! hosts, four guests, and the host among three guests.  The host is
//! `HeuristicBot`, or `ExpectBot::new` with a fourth argument of `expect`.
//! `deals` defaults to 10000 and `seed` to the clock.

use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::io::{BufRead as _, BufReader, Write as _};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::rc::Rc;
use tsap_oo::susik::{Card, Hand, Meld, shuffle};
use tsap_oo::{
    Claim, ClaimOptions, ExpectBot, Game, HeuristicBot, Round, Rules, Seat, Step, Strategy, Table,
    View, discardable,
};

/// The pipes to `guest.mjs`; every guest seat shares the one process, which
/// keeps nothing between requests
type Pipes = Rc<RefCell<(ChildStdin, BufReader<ChildStdout>)>>;

struct Guest(Pipes);

/// A hand as comma-separated [`Card::index`]es, ascending
fn csv(hand: Hand) -> String {
    let cards: Vec<_> = hand.iter().map(|card| card.index().to_string()).collect();
    cards.join(",")
}

impl Guest {
    fn ask(&self, request: &str) -> i32 {
        let (stdin, stdout) = &mut *self.0.borrow_mut();
        writeln!(stdin, "{request}").expect("guest.mjs is running");
        stdin.flush().expect("guest.mjs is running");
        let mut reply = String::new();
        stdout.read_line(&mut reply).expect("guest.mjs answers");
        reply
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("guest.mjs answered {reply:?} to {request:?}"))
    }
}

impl Strategy for Guest {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let card = view.offer().expect("a claim answers an offer").card;
        let mut claims: Vec<_> = options.eats.iter().copied().map(Claim::Eat).collect();
        if options.pong {
            claims.push(Claim::Pong);
        }
        // Only blocked claims: nothing to choose.
        if claims.is_empty() {
            return Claim::Pass;
        }
        let melds: Vec<_> = claims
            .iter()
            .map(|claim| match claim {
                Claim::Eat(meld) => format!("e{}", csv(meld.cards())),
                _ => format!("p{}", csv(Meld::Triplet(card).cards())),
            })
            .collect();
        let reply = self.ask(&format!(
            "c {} {} {} {}",
            csv(view.hand()),
            card.index(),
            u8::from(options.forced()),
            melds.join(";")
        ));
        usize::try_from(reply).map_or(Claim::Pass, |i| claims[i])
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let allowed: Hand = discardable(view.hand(), view.locked())
            .distinct()
            .map(|(card, _)| card)
            .collect();
        let reply = self.ask(&format!("d {} {}", csv(view.hand()), csv(allowed)));
        Card::from_index(usize::try_from(reply).expect("a discard is a card"))
    }

    fn name(&self) -> &str {
        "guest"
    }
}

/// Mean, standard error and 95% confidence interval of paired differences
fn report(label: &str, diffs: &[f64]) {
    let n = diffs.len() as f64;
    let mean = diffs.iter().sum::<f64>() / n;
    let var = diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let se = (var / n).sqrt();
    println!(
        "{label}: {mean:+.3} ± {se:.3} points a deal, 95% CI [{:+.3}, {:+.3}]",
        mean - 1.96 * se,
        mean + 1.96 * se
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let clone = args.next().expect("the path to a four-color-game clone");
    let deals: u64 = args.next().map_or(10_000, |s| s.parse().unwrap());
    let base: u64 = args.next().map_or_else(
        || {
            let now = std::time::SystemTime::now();
            now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        },
        |s| s.parse().unwrap(),
    );
    let expect = args.next().is_some_and(|s| s == "expect");

    let mut node = Command::new("node")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/guest.mjs"))
        .arg(clone)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("node is installed");
    let pipes: Pipes = Rc::new(RefCell::new((
        node.stdin.take().unwrap(),
        BufReader::new(node.stdout.take().unwrap()),
    )));

    let rules = Rules::new();
    let host = || -> Box<dyn Strategy> {
        if expect {
            Box::new(ExpectBot::new(&rules))
        } else {
            Box::new(HeuristicBot::new(&rules))
        }
    };
    let guest = || -> Box<dyn Strategy> { Box::new(Guest(Rc::clone(&pipes))) };
    // Seat 1's score and whether it won, with `first` there and `rest`
    // in the other three
    let play = |pile: &[Card],
                dealer: Seat,
                first: &dyn Fn() -> Box<dyn Strategy>,
                rest: &dyn Fn() -> Box<dyn Strategy>| {
        let round = Round::from_pile(rules, dealer, pile.to_vec()).expect("a full deck");
        let bots = Seat::ALL.map(|seat| Some(if seat.index() == 0 { first() } else { rest() }));
        let mut table = Table::new(round, bots);
        while table.step().expect("bots act legally") != Step::Finished {}
        let result = table.round().result().expect("finished");
        let mut game = Game::new(rules, dealer);
        game.settle(result);
        (
            f64::from(game.scores()[0]),
            u64::from(result.winner == Some(Seat::ALL[0])),
        )
    };

    // Fields: four hosts, guest among hosts, four guests, host among guests
    let mut wins = [0u64; 4];
    let (mut away, mut home) = (Vec::new(), Vec::new());
    for i in 0..deals {
        let pile = shuffle(&mut SmallRng::seed_from_u64(base + i));
        let dealer = Seat::ALL[(i % 4) as usize];
        let fields: [(&dyn Fn() -> _, &dyn Fn() -> _); 4] = [
            (&host, &host),
            (&guest, &host),
            (&guest, &guest),
            (&host, &guest),
        ];
        let scores = fields.map(|(first, rest)| play(&pile, dealer, first, rest));
        for (total, (_, won)) in wins.iter_mut().zip(scores) {
            *total += won;
        }
        away.push(scores[1].0 - scores[0].0);
        home.push(scores[3].0 - scores[2].0);
    }

    let name = if expect { "ExpectBot" } else { "HeuristicBot" };
    println!("seed base {base}, {deals} deals, host {name}");
    let rate = |w: u64| 100.0 * w as f64 / deals as f64;
    println!(
        "seat 1 wins: host among hosts {:.2}%, guest among hosts {:.2}%, \
         guest among guests {:.2}%, host among guests {:.2}%",
        rate(wins[0]),
        rate(wins[1]),
        rate(wins[2]),
        rate(wins[3])
    );
    report("among three hosts, guest − host", &away);
    report("among three guests, host − guest", &home);
    // Closing its input ends guest.mjs.
    drop(pipes);
    node.wait().expect("guest.mjs was running");
}
