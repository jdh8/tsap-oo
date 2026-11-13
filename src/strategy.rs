//! The decision interface of a player.

use crate::round::{Claim, ClaimOptions};
use crate::view::View;
use susik::Card;

/// A player: answers offers and discards
///
/// Winning and konging are compulsory and applied by the round, so these are
/// the only decisions.  Implementations see the round only through a
/// [`View`].
pub trait Strategy {
    /// Answer the offered card; the reply must be [`Claim::Pass`] or one of
    /// `options`
    fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim;

    /// Choose one of the hand's [`discardable`](crate::discardable) cards,
    /// given [`View::locked`]
    fn discard(&mut self, view: &View<'_>) -> Card;

    /// A short name for logs
    fn name(&self) -> &str {
        "bot"
    }
}
