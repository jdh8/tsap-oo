//! Running a round with strategies in some seats.

use crate::Seat;
use crate::round::{Phase, Round, RoundError};
use crate::strategy::Strategy;
use crate::view::View;

/// What one [`Table::step`] did
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    /// A bot in this seat acted
    Acted(Seat),
    /// This seat has no bot; act through [`Table::round_mut`]
    Human(Seat),
    /// The round is over
    Finished,
}

/// A round with a [`Strategy`] in each bot seat
pub struct Table {
    round: Round,
    seats: [Option<Box<dyn Strategy>>; 4],
}

impl Table {
    /// Seat the strategies; `None` seats are played from outside
    #[must_use]
    pub fn new(round: Round, seats: [Option<Box<dyn Strategy>>; 4]) -> Self {
        Self { round, seats }
    }

    /// The round
    #[must_use]
    #[inline]
    pub const fn round(&self) -> &Round {
        &self.round
    }

    /// The round, for the seats played from outside
    #[must_use]
    #[inline]
    pub const fn round_mut(&mut self) -> &mut Round {
        &mut self.round
    }

    /// Let the pending seat's bot act once
    ///
    /// # Errors
    ///
    /// When the bot's answer is not legal; see [`RoundError`].
    pub fn step(&mut self) -> Result<Step, RoundError> {
        let (seat, claiming) = match self.round.phase() {
            Phase::Claim(seat) => (seat, true),
            Phase::Discard(seat) => (seat, false),
            Phase::Finished => return Ok(Step::Finished),
        };
        let Some(bot) = self.seats[seat as usize].as_mut() else {
            return Ok(Step::Human(seat));
        };
        let view = View::new(&self.round, seat);
        if claiming {
            let options = self.round.claim_options(seat);
            let claim = bot.claim(&view, &options);
            self.round.claim(seat, claim)?;
        } else {
            let card = bot.discard(&view);
            self.round.discard(seat, card)?;
        }
        Ok(Step::Acted(seat))
    }

    /// Step until a seat without a bot must act or the round ends
    ///
    /// # Errors
    ///
    /// See [`Table::step`].
    pub fn run(&mut self) -> Result<Step, RoundError> {
        loop {
            match self.step()? {
                Step::Acted(_) => {}
                step => return Ok(step),
            }
        }
    }
}
