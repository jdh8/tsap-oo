//! A bot that samples the hidden cards and plays every choice out.

use crate::round::{Claim, ClaimOptions, Phase, Round, RoundError, discardable};
use crate::{Game, HeuristicBot, Rules, Seat, Strategy, View};
use rand::Rng;
use susik::Card;

/// How many standard errors a choice must beat the greedy one by.
// ponytail: one knob, set by arena play; playouts are noisy and the greedy
// choice is a strong prior.
const CONFIDENCE: f64 = 2.0;

/// A Monte Carlo bot: deals the cards it cannot see at random, plays each
/// legal choice to the end of the round with [`HeuristicBot`]s in every
/// seat, and leaves the greedy choice only for one that clearly scores
/// better on average
///
/// Every choice is tried against the same sampled deals, so luck of the deal
/// cancels between them.  A decision costs `samples` playouts per choice.
// ponytail: flat sampling with the deals uniform over the unseen cards (see
// `View::determinize`) and the round's own score as the payoff.  Next steps,
// in order of expected gain: drop the choices the greedy grade ranks last,
// spend the samples where the means are close (UCB), weigh deals by what
// the other seats did.
#[derive(Debug, Clone)]
pub struct MonteCarloBot<R> {
    rng: R,
    samples: usize,
    policy: HeuristicBot,
}

impl<R: Rng> MonteCarloBot<R> {
    /// A bot for `rules` that plays `samples` deals out per decision
    #[must_use]
    pub fn new(rules: &Rules, samples: usize, rng: R) -> Self {
        Self {
            rng,
            samples,
            policy: HeuristicBot::new(rules),
        }
    }

    /// The first action, unless the sampled deals say another one scores
    /// better by [`CONFIDENCE`] standard errors; then the best such
    fn choose<A: Copy>(
        &mut self,
        view: &View<'_>,
        actions: &[A],
        act: impl Fn(&mut Round, A) -> Result<(), RoundError>,
    ) -> A {
        if actions.len() < 2 || self.samples < 2 {
            return actions[0];
        }
        // Sums of each action's gain over the first, and of its square.
        let mut gains = vec![(0.0_f64, 0.0_f64); actions.len()];
        for _ in 0..self.samples {
            let world = view.determinize(&mut self.rng);
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

/// Play `round` out with `policy` in every seat; what `seat` scores
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

impl<R: Rng> Strategy for MonteCarloBot<R> {
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim {
        let seat = view.seat();
        // The greedy answer first: it stands unless clearly beaten.
        let greedy = self.policy.claim(view, options);
        let actions: Vec<Claim> = [greedy]
            .into_iter()
            .chain((!options.forced()).then_some(Claim::Pass))
            .chain(options.pong.then_some(Claim::Pong))
            .chain(options.eats.iter().copied().map(Claim::Eat))
            .enumerate()
            .filter(|&(i, claim)| i == 0 || claim != greedy)
            .map(|(_, claim)| claim)
            .collect();
        self.choose(view, &actions, |round, claim| round.claim(seat, claim))
    }

    fn discard(&mut self, view: &View<'_>) -> Card {
        let seat = view.seat();
        let greedy = self.policy.discard(view);
        let actions: Vec<Card> = [greedy]
            .into_iter()
            .chain(
                discardable(view.hand(), view.locked())
                    .distinct()
                    .map(|(card, _)| card)
                    .filter(|&card| card != greedy),
            )
            .collect();
        self.choose(view, &actions, |round, card| round.discard(seat, card))
    }

    fn name(&self) -> &str {
        "monte carlo"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Step, Table};
    use rand::SeedableRng as _;
    use rand::rngs::SmallRng;

    /// A sampled round against greedy bots runs to its end without an
    /// illegal action, whatever the sampled deals made of the claim queue.
    #[test]
    fn plays_a_round() {
        let rules = Rules::new();
        let mut rng = SmallRng::seed_from_u64(1);
        let round = Round::deal(rules, Seat::ALL[0], &mut rng);
        let mut seats =
            Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as Box<dyn Strategy>));
        seats[1] = Some(Box::new(MonteCarloBot::new(&rules, 2, rng)));
        let mut table = Table::new(round, seats);
        assert_eq!(table.run(), Ok(Step::Finished));
    }
}
