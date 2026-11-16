//! Peeker: `MonteCarloBot` with the other hands in plain sight.
//!
//! It samples with `View::peek`, which keeps every hand and shuffles only the
//! pile, so no reading of the opponents can beat it: its gain over
//! `MonteCarloBot` is the ceiling on what such reading is worth.  It cheats,
//! so it lives here and not in the library (`doc/stronger-bot.md`).
// ponytail: a copy of `MonteCarloBot`'s search with one line changed.  The
// library keeps no hook for a cheat.
use new::{Claim, ClaimOptions, Game, HeuristicBot, Phase, Round, RoundError, Seat, View};
use new::{Rules, Strategy, susik::Card};
use rand::Rng;

/// As `MonteCarloBot`'s `CONFIDENCE` and `CANDIDATES`
const CONFIDENCE: f64 = 1.0;
const CANDIDATES: usize = 3;

pub struct Peeker<R> {
    rng: R,
    samples: usize,
    policy: HeuristicBot,
}

impl<R: Rng> Peeker<R> {
    pub fn new(rules: &Rules, samples: usize, rng: R) -> Self {
        Self {
            rng,
            samples,
            policy: HeuristicBot::new(rules),
        }
    }

    fn choose<A: Copy>(
        &mut self,
        view: &View<'_>,
        actions: &[A],
        act: impl Fn(&mut Round, A) -> Result<(), RoundError>,
    ) -> A {
        if actions.len() < 2 || self.samples < 2 {
            return actions[0];
        }
        let mut gains = vec![(0.0_f64, 0.0_f64); actions.len()];
        for _ in 0..self.samples {
            let world = view.peek(&mut self.rng);
            let mut scores = actions.iter().map(|&action| {
                let mut round = world.clone();
                act(&mut round, action).expect("the round offered this action");
                f64::from(playout(&mut self.policy, round, view.seat()))
            });
            let base = scores.next().expect("two actions or more");
            for (gain, score) in gains[1..].iter_mut().zip(scores) {
                gain.0 += score - base;
                gain.1 += (score - base).powi(2);
            }
        }
        let n = self.samples as f64;
        let mut best = (0.0, actions[0]);
        for (&(sum, squares), &action) in gains.iter().zip(actions) {
            let error = ((squares - sum * sum / n) / (n - 1.0) * n).sqrt();
            if sum > best.0 && sum > CONFIDENCE * error {
                best = (sum, action);
            }
        }
        best.1
    }
}

fn playout(policy: &mut HeuristicBot, mut round: Round, seat: Seat) -> i32 {
    loop {
        match round.phase() {
            Phase::Claim(s) => {
                let options = round.claim_options(s);
                let claim = policy.claim(&View::new(&round, s), &options);
                round.claim(s, claim)
            }
            Phase::Discard(s) => {
                let card = policy.discard(&View::new(&round, s));
                round.discard(s, card)
            }
            Phase::Finished => break,
        }
        .expect("the greedy bot acts legally");
    }
    let mut game = Game::new(*round.rules(), round.dealer());
    game.settle(round.result().expect("the round is finished"));
    game.scores()[seat.index()]
}

impl<R: Rng> Strategy for Peeker<R> {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let seat = view.seat();
        let mut actions: Vec<_> = self
            .policy
            .claims(view, options)
            .into_iter()
            .map(|(action, _)| action)
            .collect();
        actions.truncate(CANDIDATES);
        self.choose(view, &actions, |round, claim| round.claim(seat, claim))
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let seat = view.seat();
        let mut actions: Vec<_> = self
            .policy
            .discards(view)
            .into_iter()
            .map(|(action, _)| action)
            .collect();
        actions.truncate(CANDIDATES);
        self.choose(view, &actions, |round, card| round.discard(seat, card))
    }

    fn name(&self) -> &str {
        "peeker"
    }
}
