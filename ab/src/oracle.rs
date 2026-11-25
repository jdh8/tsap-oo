//! Oracles: `ExpectBot` as it ships, with one of its inputs read off the
//! other seats' hands.
//!
//! Each measures what one channel of reading the opponents could be worth
//! to the DP as it is built, before any inference is written
//! (`doc/opponents.md`).  They cheat through `View::peek`, so they live
//! here and not in the library, as the Peeker does.  They read the hands
//! and never the pile's order, so `RESHUFFLE` stays unbiased.
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
    /// A discard's value less what the card gives the other seat
    Danger,
    /// [`Kind::Danger`]'s half for the card the other seat wins on
    Win,
    /// [`Kind::Danger`]'s half for the card the other seat claims
    Claim,
}

impl Kind {
    /// `ORACLE`'s value
    pub fn parse(name: &str) -> Self {
        match name {
            "live" => Self::Live,
            "danger" => Self::Danger,
            "win" => Self::Win,
            "claim" => Self::Claim,
            _ => panic!("ORACLE is live, danger, win or claim"),
        }
    }
}

pub struct Oracle<R> {
    bot: ExpectBot,
    kind: Kind,
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
    /// For [`Kind::Danger`] and its halves away from a table of two at
    /// 10 胡: they ask one other seat, and were measured there.
    pub fn new(rules: &Rules, bot: ExpectBot, kind: Kind, rng: R) -> Self {
        assert!(
            kind == Kind::Live || (rules.players, rules.threshold) == (2, 10),
            "{kind:?} is for two players at 10 胡"
        );
        Self {
            bot,
            kind,
            policy: HeuristicBot::new(rules),
            layouts: [1, 16].map(|cost| Solver::new(rules.concealed, Some(cost)).locked()),
            rng,
        }
    }

    /// Set the bot's input for this decision
    fn read(&mut self, view: &View<'_>) {
        if self.kind != Kind::Live {
            return;
        }
        let (seat, rules) = (view.seat(), view.rules());
        let world = view.peek(&mut self.rng);
        let mut gone = Hand::EMPTY;
        for &other in Seat::ALL[..rules.players].iter().filter(|&&s| s != seat) {
            let hand = world.hand(other);
            let mut loose = Hand::EMPTY;
            for solver in &mut self.layouts {
                loose += solver.layout(hand).expect("loose layouts exist").loose;
            }
            gone.extend(hand.iter().filter(|&card| !loose.contains(card)));
        }
        *self.bot.gone_mut() = gone;
    }

    /// `value`, the DP's of discarding `card` in `world`, less what the
    /// card gives the next seat, whose hand is in plain sight: down to the
    /// settled score if that seat wins on it, and less what the DP makes
    /// of the claim `HeuristicBot` answers with over a pass, in that
    /// seat's place.  At a table of two a point of the other seat's is a
    /// point of this seat's.
    // ponytail: a discard the other seat kongs counts as safe.  It is rare
    // and has no pass to value the kong against.
    pub fn kept(&mut self, world: &Round, seat: Seat, card: Card, value: f64) -> f64 {
        let mut round = world.clone();
        round.discard(seat, card).expect("a legal discard");
        let next = round.events().get(world.events().len() + 1);
        if let Some(&Event::Win { seat: winner, .. }) = next
            && winner != seat
        {
            if self.kind == Kind::Claim {
                return value;
            }
            let mut game = Game::new(*round.rules(), round.dealer());
            game.settle(round.result().expect("the round is finished"));
            return value.min(f64::from(game.scores()[seat.index()]));
        }
        if self.kind == Kind::Win {
            return value;
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

impl<R: Rng> Strategy for Oracle<R> {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        self.read(view);
        let ranked = self.bot.claims(view, options);
        if self.kind == Kind::Live || !self.bot.searches(view, view.hand().len()) {
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
        if self.kind == Kind::Live {
            return self.bot.discard(view);
        }
        let world = view.peek(&mut self.rng);
        self.safest(&world, view.seat()).0
    }

    fn name(&self) -> &str {
        "oracle"
    }
}
