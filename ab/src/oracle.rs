//! Oracles: `ExpectBot` as it ships, with one of its inputs read off the
//! other seats' hands.
//!
//! Each measures what one channel of reading the opponents could be worth
//! to the DP as it is built, before any inference is written
//! (`doc/opponents.md`).  They cheat through `View::peek`, so they live
//! here and not in the library, as the Peeker does.  They read the hands
//! and never the pile's order, so `RESHUFFLE` stays unbiased.  `Cards` is
//! the one that does not cheat: it reads only the other seats' card counts.
use new::susik::{Card, Hand, Solver};
use new::{Claim, ClaimOptions, Event, ExpectBot, Game, HeuristicBot, Phase, Round};
use new::{Rules, Seat, Strategy, View};
use rand::Rng;

/// The input an [`Oracle`] replaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `live` less the cards the other seats keep: their hands but the
    /// cards loose in either layout `HeuristicBot` grades, which it
    /// discards from
    Live,
    /// h bent by the other seats' waits, read off their hands
    Waits,
    /// h bent by the fewest concealed cards another seat holds: public
    Cards,
    /// A discard's value less what the card gives the other seat
    Danger,
}

impl Kind {
    /// `ORACLE`'s value
    pub fn parse(name: &str) -> Self {
        match name {
            "live" => Self::Live,
            "waits" => Self::Waits,
            "cards" => Self::Cards,
            "danger" => Self::Danger,
            _ => panic!("ORACLE is live, waits, cards or danger"),
        }
    }
}

/// How often another seat wins on a card, by its waits at the decision (0 to
/// 4 or more copies unseen) and the card's lead in draws, over how often
/// one does at that lead whatever it holds: `PLAYERS=2 hazard 20000 400`
const WAITS: [[f64; 26]; 5] = [
    [
        0.214, 0.538, 0.660, 0.757, 0.805, 0.859, 0.878, 0.908, 0.921, 0.938, 0.935, 0.946, 0.956,
        0.953, 0.971, 0.974, 0.975, 0.978, 0.981, 0.982, 0.990, 0.989, 0.991, 1.000, 1.001, 1.001,
    ],
    [
        0.385, 0.652, 0.502, 0.775, 0.730, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736,
        0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736, 0.736,
    ],
    [
        1.228, 1.042, 1.009, 1.421, 1.285, 1.442, 1.475, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326,
        1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326, 1.326,
    ],
    [
        1.904, 1.620, 1.625, 1.693, 1.749, 1.777, 1.623, 1.799, 1.649, 1.556, 1.714, 1.842, 1.682,
        1.649, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600, 1.600,
    ],
    [
        3.032, 2.827, 2.682, 2.407, 2.323, 2.105, 2.106, 1.851, 1.873, 1.774, 1.888, 1.762, 1.731,
        1.867, 1.537, 1.447, 1.463, 1.463, 1.463, 1.463, 1.463, 1.463, 1.463, 1.463, 1.463, 1.463,
    ],
];

/// The same by the fewest concealed cards another seat holds, 0 to 21; a
/// count too rare to measure leaves h alone: `PLAYERS=2 hazard 20000 400`
const CARDS: [[f64; 26]; 22] = [
    [1.0; 26],
    [1.0; 26],
    [1.0; 26],
    [1.0; 26],
    [
        1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097,
        1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097, 1.097,
    ],
    [
        1.245, 1.190, 1.405, 1.189, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868,
        0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868, 0.868,
    ],
    [
        1.441, 1.338, 1.404, 1.089, 1.250, 1.006, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297,
        1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297, 1.297,
    ],
    [
        1.409, 1.173, 1.364, 1.166, 1.317, 1.103, 1.113, 1.181, 0.998, 0.998, 0.998, 0.998, 0.998,
        0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998, 0.998,
    ],
    [
        1.377, 1.248, 1.296, 1.130, 1.229, 1.125, 1.230, 0.975, 1.307, 0.928, 1.404, 1.013, 0.855,
        0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855, 0.855,
    ],
    [
        1.318, 1.227, 1.326, 1.285, 1.241, 1.185, 1.184, 1.162, 1.125, 1.083, 1.016, 1.051, 0.926,
        0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997, 0.997,
    ],
    [
        1.372, 1.353, 1.299, 1.286, 1.194, 1.262, 1.085, 1.317, 1.176, 1.203, 1.087, 1.238, 1.341,
        1.052, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118, 1.118,
    ],
    [
        1.264, 1.237, 1.159, 1.156, 1.089, 1.123, 1.126, 1.031, 1.074, 1.010, 1.016, 0.968, 1.075,
        1.039, 1.011, 1.059, 1.095, 1.095, 1.095, 1.095, 1.095, 1.095, 1.095, 1.095, 1.095, 1.095,
    ],
    [
        1.209, 1.193, 1.128, 1.154, 1.125, 1.087, 1.139, 1.126, 1.061, 1.090, 1.094, 1.057, 1.031,
        1.185, 1.039, 1.064, 0.994, 0.894, 0.894, 0.894, 0.894, 0.894, 0.894, 0.894, 0.894, 0.894,
    ],
    [
        1.111, 1.183, 1.240, 1.185, 1.092, 1.229, 1.153, 1.082, 1.070, 1.133, 1.289, 1.026, 1.037,
        1.117, 1.035, 0.938, 1.152, 1.116, 1.116, 1.116, 1.116, 1.116, 1.116, 1.116, 1.116, 1.116,
    ],
    [
        0.971, 1.031, 1.043, 1.050, 1.059, 1.053, 1.036, 1.024, 1.000, 1.032, 0.981, 1.004, 1.046,
        0.923, 1.035, 0.962, 1.076, 1.111, 0.992, 0.971, 0.946, 0.946, 0.946, 0.946, 0.946, 0.946,
    ],
    [
        0.775, 0.955, 0.963, 1.029, 1.030, 1.020, 0.999, 1.085, 1.044, 1.031, 1.025, 0.996, 1.077,
        1.024, 1.065, 1.047, 1.072, 1.063, 1.051, 0.912, 0.902, 0.902, 0.902, 0.902, 0.902, 0.902,
    ],
    [
        0.764, 1.038, 1.028, 1.098, 1.171, 1.069, 1.126, 1.045, 1.101, 1.157, 1.126, 1.161, 1.150,
        1.113, 1.085, 1.058, 1.017, 1.144, 1.214, 1.176, 1.176, 1.176, 1.176, 1.176, 1.176, 1.176,
    ],
    [
        0.495, 0.731, 0.747, 0.865, 0.893, 0.907, 0.882, 0.990, 0.962, 0.962, 0.905, 0.968, 0.995,
        0.953, 0.942, 0.992, 0.961, 0.907, 1.081, 0.918, 0.972, 1.027, 0.910, 1.068, 1.068, 1.068,
    ],
    [
        0.489, 0.633, 0.737, 0.854, 0.907, 0.922, 0.957, 0.961, 0.951, 0.920, 0.958, 0.945, 0.961,
        0.997, 1.002, 1.028, 1.091, 0.964, 0.819, 0.981, 0.912, 1.061, 1.080, 1.080, 1.080, 1.080,
    ],
    [
        0.341, 0.657, 0.809, 0.742, 0.962, 1.053, 1.044, 0.975, 1.140, 0.930, 1.125, 1.123, 1.091,
        1.114, 1.186, 1.079, 1.126, 1.218, 1.218, 1.218, 1.218, 1.218, 1.218, 1.218, 1.218, 1.218,
    ],
    [
        0.289, 0.494, 0.659, 0.719, 0.769, 0.846, 0.875, 0.866, 0.904, 0.935, 0.933, 0.952, 0.913,
        0.958, 0.962, 0.964, 0.919, 0.978, 0.956, 0.996, 0.983, 0.921, 0.983, 0.980, 0.965, 0.965,
    ],
    [1.0; 26],
];

pub struct Oracle<R> {
    bot: ExpectBot,
    kind: Kind,
    /// The bot's own h, which [`Kind::Waits`] and [`Kind::Cards`] bend
    end: Vec<f64>,
    /// The other seat's answers, for [`Kind::Danger`]
    policy: HeuristicBot,
    /// `HeuristicBot`'s two layouts, for [`Kind::Live`]
    layouts: [Solver; 2],
    /// `View::peek` shuffles the pile, which no oracle reads
    rng: R,
}

impl<R: Rng> Oracle<R> {
    /// # Panics
    ///
    /// Unless [`Kind::Live`], away from a table of two at 10 胡: the tables
    /// are measured there, and [`Kind::Danger`] asks one other seat.
    pub fn new(rules: &Rules, mut bot: ExpectBot, kind: Kind, rng: R) -> Self {
        assert!(
            kind == Kind::Live || (rules.players, rules.threshold) == (2, 10),
            "{kind:?} is for two players at 10 胡"
        );
        Self {
            end: bot.outlook_mut().end.clone(),
            bot,
            kind,
            policy: HeuristicBot::new(rules),
            layouts: [1, 16].map(|cost| Solver::new(rules.concealed, Some(cost)).locked()),
            rng,
        }
    }

    /// Set the bot's input for this decision
    fn read(&mut self, view: &View<'_>) {
        let (seat, rules) = (view.seat(), view.rules());
        let others = Seat::ALL[..rules.players].iter().filter(|&&s| s != seat);
        let row = match self.kind {
            Kind::Live => {
                let world = view.peek(&mut self.rng);
                let mut gone = Hand::EMPTY;
                for &other in others {
                    let hand = world.hand(other);
                    let mut loose = Hand::EMPTY;
                    for solver in &mut self.layouts {
                        loose += solver.layout(hand).expect("loose layouts exist").loose;
                    }
                    gone.extend(hand.iter().filter(|&card| !loose.contains(card)));
                }
                *self.bot.gone_mut() = gone;
                return;
            }
            Kind::Waits => {
                let world = view.peek(&mut self.rng);
                let waits = others.map(|&other| waits(&world, other)).max();
                &WAITS[waits.expect("another seat").min(WAITS.len() - 1)]
            }
            Kind::Cards => {
                let fewest = others.map(|&other| view.concealed_len(other)).min();
                &CARDS[fewest.expect("another seat")]
            }
            Kind::Danger => return,
        };
        // A card of segment `k` comes `draws - k` draws on.
        let draws = view.pile_len().saturating_sub(rules.dead());
        let end = &mut self.bot.outlook_mut().end;
        for (k, (h, own)) in end.iter_mut().zip(&self.end).enumerate() {
            let lead = draws.saturating_sub(k).min(row.len() - 1);
            *h = own * if k <= draws { row[lead] } else { 1.0 };
        }
    }

    /// `value`, the DP's of discarding `card` in `world`, less what the
    /// card gives the next seat, whose hand is in plain sight: down to the
    /// settled score if that seat wins on it, and less what the DP makes
    /// of the claim `HeuristicBot` answers with over a pass, in that
    /// seat's place.  At a table of two a point of the other seat's is a
    /// point of this seat's.
    // ponytail: a discard the other seat kongs counts as safe.  It is rare
    // and has no pass to value the kong against.
    fn kept(&mut self, world: &Round, seat: Seat, card: Card, value: f64) -> f64 {
        let mut round = world.clone();
        round.discard(seat, card).expect("a legal discard");
        let next = round.events().get(world.events().len() + 1);
        if let Some(&Event::Win { seat: winner, .. }) = next
            && winner != seat
        {
            let mut game = Game::new(*round.rules(), round.dealer());
            game.settle(round.result().expect("the round is finished"));
            return value.min(f64::from(game.scores()[seat.index()]));
        }
        // The round may have drawn on, into a pile shuffled for the peek.
        let (Phase::Claim(other), Some(offer)) = (round.phase(), round.offer()) else {
            return value;
        };
        if other == seat || offer.drawn || offer.source != seat {
            return value;
        }
        let (view, options) = (View::new(&round, other), round.claim_options(other));
        let claim = self.policy.claim(&view, &options);
        if claim == Claim::Pass || !self.bot.searches(&view, view.hand().len()) {
            return value;
        }
        let answers = self.bot.claims(&view, &options);
        let of = |claim| answers.iter().find(|&&(c, _)| c == claim).map(|&(_, v)| v);
        match (of(claim), of(Claim::Pass)) {
            (Some(taken), Some(passed)) => value - (taken - passed).max(0.0),
            _ => value,
        }
    }

    /// The discard that keeps the most in `world`, where `seat` is to
    /// discard, with what it keeps and the DP's best value before
    fn safest(&mut self, world: &Round, seat: Seat) -> (Card, f64, f64) {
        let view = View::new(world, seat);
        let ranked = self.bot.discards(&view);
        let (first, top) = ranked[0];
        if !self.bot.searches(&view, view.hand().len() - 1) {
            return (first, top, top);
        }
        let mut best = (first, f64::NEG_INFINITY);
        // Best first, and a card only loses value: stop once none can win.
        for (card, value) in ranked {
            if value <= best.1 {
                break;
            }
            let kept = self.kept(world, seat, card, value);
            if kept > best.1 {
                best = (card, kept);
            }
        }
        (best.0, best.1, top)
    }
}

/// The copies of the cards `seat` wins on that it has not seen, as
/// `examples/hazard.rs` counts them
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

impl<R: Rng> Strategy for Oracle<R> {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        self.read(view);
        let ranked = self.bot.claims(view, options);
        if self.kind != Kind::Danger || !self.bot.searches(view, view.hand().len()) {
            return ranked[0].0;
        }
        // A claim is worth less by what its best discard gives away.
        let (world, seat) = (view.peek(&mut self.rng), view.seat());
        let mut best = (ranked[0].0, f64::NEG_INFINITY);
        for (claim, mut value) in ranked {
            if value <= best.1 {
                break;
            }
            if claim != Claim::Pass {
                let mut round = world.clone();
                round.claim(seat, claim).expect("the round offered this");
                if round.phase() == Phase::Discard(seat) {
                    let (_, kept, top) = self.safest(&round, seat);
                    value += kept - top;
                }
            }
            if value > best.1 {
                best = (claim, value);
            }
        }
        best.0
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        self.read(view);
        if self.kind != Kind::Danger {
            return self.bot.discard(view);
        }
        let world = view.peek(&mut self.rng);
        self.safest(&world, view.seat()).0
    }

    fn name(&self) -> &str {
        "oracle"
    }
}
