//! Where the cards a seat has not seen lie at a table of two, beside what
//! is public about each kind: what an inference for `live` has to go on
//! (`doc/opponents.md`, batch 3).
//!
//! `ExpectBot` as it ships plays `HeuristicBot`, each deal twice, once in
//! either seat.  At every discard it values in points, each kind it has not
//! seen every copy of is counted by its [`Evidence`] and by the copies not
//! seen: how many of them the other seat keeps, as the `live` oracle of
//! `ab/src/oracle.rs` reads it (its hand less every kind loose in either
//! layout `HeuristicBot` grades), how many it holds loose, and how often it
//! keeps them all, which leaves the kind dead.  The rest are in the pile.
//!
//! The last columns are what `live` stands for: the copies that came up
//! before the round ended, drawn or thrown from the other seat's hand, over
//! the copies expected if every card the seat has not seen were as likely
//! to be each public card that followed, and the same if every card of the
//! kind's rank were as likely to be each that followed of that rank; and
//! that again over the cards the other seat did not take, by a win or a
//! claim.  The last is the one the DP could take: `Outlook` weighs the
//! ranks already, and a card the other seat takes is none of this seat's.
//!
//! ```sh
//! cargo run --release --example kept -- [rounds] [seed]
//! ```
//!
//! `GATE_CARDS` (default 10) is the bot's gate and `SHALLOW` (default 0) the
//! cards it plays two claims deep with past it, as in `giveaway`.
use tsap_oo::susik::{Card, Hand, Solver};
use tsap_oo::{
    Eval, Event, Evidence, ExpectBot, Giveaway, HeuristicBot, Phase, Round, Rules, Seat, Strategy,
    View,
};

mod common;

/// A cell needs this many kinds; a thinner one holds its row's ratio
const LEAST: u64 = 1000;

#[derive(Default, Clone, Copy)]
struct Count {
    kinds: u64,
    copies: u64,
    kept: u64,
    loose: u64,
    /// Kinds with every unseen copy kept
    dead: u64,
    /// Copies that came up later in the round
    came: u64,
    /// The same if every unseen card were as likely to
    expected: f64,
    /// The same if every unseen card of the rank were
    ranked: f64,
    /// Copies that came up and the other seat did not take
    free: u64,
    /// The same if every unseen card of the rank were as likely to
    left: f64,
}

impl Count {
    fn add(&mut self, other: Self) {
        self.kinds += other.kinds;
        self.copies += other.copies;
        self.kept += other.kept;
        self.loose += other.loose;
        self.dead += other.dead;
        self.came += other.came;
        self.expected += other.expected;
        self.ranked += other.ranked;
        self.free += other.free;
        self.left += other.left;
    }
}

/// Thrown, passed, and fresh by the run's kinds thrown and by its copies
/// not seen, as `FRESH` of `src/giveaway.rs`
type Counts = [Count; 20];

/// A decision: the events before it, the cards not seen, and each kind's
/// cell
type Decision = (usize, Hand, Vec<(usize, Card)>);

/// Count every kind `me` has not seen every copy of, at its discard
fn record(round: &Round, me: Seat, layouts: &mut [Solver; 2], counts: &mut Counts) -> Decision {
    let (view, opp) = (View::new(round, me), me.next(round.rules().players));
    let giveaway = Giveaway::new(&view);
    let shown = Seat::ALL
        .into_iter()
        .flat_map(|seat| view.exposed(seat))
        .flat_map(|meld| meld.cards())
        .chain(view.discards().iter().map(|&(_, card)| card));
    let unseen = shown.fold(Hand::FULL - view.hand(), Hand::without);
    let hand = round.hand(opp);
    let mut loose = Hand::EMPTY;
    for solver in layouts {
        loose += solver.layout(hand).expect("loose layouts exist").loose;
    }
    let mut kinds = Vec::new();
    for (card, n) in unseen.distinct() {
        let cell = match giveaway.evidence(card) {
            Evidence::Thrown => 0,
            Evidence::Passed => 1,
            Evidence::Fresh { thrown, unseen } => 2 + 6 * thrown + unseen,
        };
        let held = u64::from(hand.count(card));
        let kept = if loose.contains(card) { 0 } else { held };
        counts[cell].add(Count {
            kinds: 1,
            copies: u64::from(n),
            kept,
            loose: held - kept,
            dead: u64::from(kept == u64::from(n)),
            ..Count::default()
        });
        kinds.push((cell, card));
    }
    (round.events().len(), unseen, kinds)
}

/// Count the copies of each decision's kinds that came up after it: the
/// cards drawn, and the ones the other seat threw from its hand
fn follow(round: &Round, me: Seat, decisions: &[Decision], counts: &mut Counts) {
    let events = round.events();
    // Whether the other seat took the card that came up at `i`
    let taken = |i: usize| match events.get(i + 1) {
        Some(
            &(Event::Eat { seat, .. }
            | Event::Pong { seat, .. }
            | Event::Kong { seat, .. }
            | Event::Win { seat, .. }),
        ) => seat != me,
        _ => false,
    };
    for (start, unseen, kinds) in decisions {
        let (mut came, mut free) = (Hand::EMPTY, Hand::EMPTY);
        for i in *start..events.len() {
            let card = match events[i] {
                Event::Draw { card, .. } => card,
                // A rejected draw is logged as its drawer's discard too.
                Event::Discard { seat, card }
                    if seat != me && events[i - 1] != (Event::Draw { seat, card }) =>
                {
                    card
                }
                _ => continue,
            };
            came.insert(card);
            if !taken(i) {
                free.insert(card);
            }
        }
        let of = |hand: &Hand, card: Card| hand.iter().filter(|c| c.rank == card.rank).count();
        for &(cell, card) in kinds {
            let (cell, n) = (&mut counts[cell], f64::from(unseen.count(card)));
            cell.came += u64::from(came.count(card));
            cell.free += u64::from(free.count(card));
            cell.expected += n * came.len() as f64 / unseen.len() as f64;
            cell.ranked += n * of(&came, card) as f64 / of(unseen, card) as f64;
            cell.left += n * of(&free, card) as f64 / of(unseen, card) as f64;
        }
    }
}

/// `round` played out with `bot` in `me`'s seat against `HeuristicBot`
fn play(mut round: Round, me: Seat, mut bot: ExpectBot, counts: &mut Counts) {
    let rules = *round.rules();
    let mut policy = HeuristicBot::new(&rules);
    let mut layouts = [1, 16].map(|cost| Solver::new(rules.concealed, Some(cost)).locked());
    let mut decisions = Vec::new();
    loop {
        match round.phase() {
            Phase::Finished => break follow(&round, me, &decisions, counts),
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
                        decisions.push(record(&round, me, &mut layouts, counts));
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

fn line(name: &str, c: Count) {
    let share = |k: u64, n: u64| 100.0 * k as f64 / n.max(1) as f64;
    println!(
        "{name:<14}{:>10}{:>10}{:>8.1}%{:>8.1}%{:>8.1}%{:>8.1}%{:>9.3}{:>9.3}{:>9.3}",
        c.kinds,
        c.copies,
        share(c.kept, c.copies),
        share(c.loose, c.copies),
        share(c.dead, c.kinds),
        share(c.came, c.copies),
        c.came as f64 / c.expected.max(f64::MIN_POSITIVE),
        c.came as f64 / c.ranked.max(f64::MIN_POSITIVE),
        c.free as f64 / c.left.max(f64::MIN_POSITIVE),
    );
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
        let mut counts = Counts::default();
        for me in [Seat::ALL[0], Seat::ALL[1]] {
            let bot = ExpectBot::new(&rules)
                .gated(gate)
                .shallow(Eval::of(&rules), deep);
            play(round.clone(), me, bot, &mut counts);
        }
        counts
    });
    let mut counts = Counts::default();
    for deal in &played {
        for (cell, &other) in counts.iter_mut().zip(deal) {
            cell.add(other);
        }
    }
    let sum = |cells: &[Count]| {
        let mut sum = Count::default();
        cells.iter().for_each(|&cell| sum.add(cell));
        sum
    };
    println!("{rounds} rounds each way, seed {seed}, gate {gate}, shallow {deep}");
    println!(
        "{:<14}{:>10}{:>10}{:>9}{:>9}{:>9}{:>9}{:>9}{:>9}{:>9}",
        "evidence", "kinds", "copies", "kept", "loose", "dead", "came", "ratio", "in rank", "free"
    );
    line("thrown", counts[0]);
    line("passed", counts[1]);
    for thrown in [2, 1, 0] {
        let row = &counts[2 + 6 * thrown..8 + 6 * thrown];
        line(&format!("fresh, {thrown}"), sum(row));
    }
    line("all", sum(&counts));
    for thrown in 0..3 {
        println!("\nfresh, {thrown} of the run's kinds thrown, by its copies not seen");
        for (unseen, &cell) in counts[2 + 6 * thrown..8 + 6 * thrown].iter().enumerate() {
            line(&format!("{} or {}", 2 * unseen, 2 * unseen + 1), cell);
        }
    }
    // The ratio in a rank of the cards left free, as the table `dbe68dc`'s
    // `Giveaway::chance` held
    let ratio = |c: Count| c.free as f64 / c.left.max(f64::MIN_POSITIVE);
    println!("\nconst COMES_THROWN: f64 = {:.3};", ratio(counts[0]));
    println!("const COMES_PASSED: f64 = {:.3};", ratio(counts[1]));
    println!("const COMES_FRESH: [[f64; 6]; 3] = [");
    for row in counts[2..].chunks(6) {
        let all = sum(row);
        let cells: Vec<String> = row
            .iter()
            .map(|&c| format!("{:.3}", ratio(if c.kinds < LEAST { all } else { c })))
            .collect();
        println!("    [{}],", cells.join(", "));
    }
    println!("];");
}
