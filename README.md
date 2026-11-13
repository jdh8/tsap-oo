# tsap-oo

[![Crates.io](https://img.shields.io/crates/v/tsap-oo)](https://crates.io/crates/tsap-oo)
[![Docs.rs](https://docs.rs/tsap-oo/badge.svg)](https://docs.rs/tsap-oo)
[![Build Status](https://github.com/jdh8/tsap-oo/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/tsap-oo)

十胡 (Taiwanese *tsa̍p-ôo*, "ten hu") is the Four Color Cards game of Taiwan:
four players, twenty cards each, and a hand wins when all twenty-one cards
lay out as melds worth ten 胡 or more.  This crate implements the Taiwanese
rules on top of the [susik] deck: a runtime-checked `Round` state machine
that resolves every public card in priority order (win, forced kong, pong,
eat), a `Game` scoreboard, an information-hygienic `View`, the `Strategy`
trait, a greedy `HeuristicBot`, and a `Table` that drives bots.  The name is
the Taiwanese reading, which no other game shares.

Play it in the browser at [jdh8.github.io/tsap-oo][web]; the front end in
[`web/`](web/) is the engine compiled to WebAssembly.

[susik]: https://github.com/jdh8/susik
[web]: https://jdh8.github.io/tsap-oo/

## The rules implemented

- 112 cards; the dealer gets 21, the others 20, and the dealer discards
  first.  Play runs counterclockwise.  The last six cards of the pile are
  dead: reaching them ends the round with no winner and the same dealer.
- A draw is turned face up.  Any public card — a draw or a discard — can
  be won on by anyone, must be konged by whoever holds its concealed
  triplet, may be ponged by anyone holding its pair, and may be eaten by
  the drawer or by the next player into 將士象, 車馬包, soldiers of three
  or four colours, or a pair.  Win beats kong beats pong beats eat; equal
  claims go to the earlier seat in turn order.  Winning is compulsory.
- A concealed triplet is never split for a claim, and the card just claimed
  cannot be discarded in the same turn.
- 胡: lone general 1, 將士象 2, 車馬包 1, soldiers of three colours 3 and of
  four 5, a triplet 3 concealed or 1 ponged, a quad 8 concealed or 6 konged,
  a pair 0.  "Concealed" means dealt and never shown; every meld formed with
  a public card is exposed.  Ten to win; the winner collects their 胡 from
  each other seat and deals next.

These follow the mainstream Taiwanese table (the Chinese Wikipedia article
and the common Taiwanese write-ups agree on it).  House variants — a
different value for four-colour soldiers, the 將 parity rule, five-player
八胡 — are not modelled; all values are fields of `Rules`.

## Modules

- the crate root: [`Seat`]
- [`rules`][mod-rules]: [`Rules`], the scoring table
- [`round`][mod-round]: one deal — [`Round`], [`Phase`], [`Offer`],
  [`Claim`], [`ClaimOptions`], [`Event`], [`RoundResult`]
- [`game`][mod-game]: the scoreboard — [`Game`]
- [`view`][mod-view], [`strategy`][mod-strategy], [`driver`][mod-driver]:
  [`View`], [`Strategy`], [`Table`]
- [`heuristic`][mod-heuristic]: [`HeuristicBot`]

## Feature flags

- `rand` (default): shuffled dealing (`Round::deal`, `Game::deal`)

## Quick start

```rust
use tsap_oo::{Game, HeuristicBot, Rules, Seat, Step, Table};

# #[cfg(feature = "rand")] {
let rules = Rules::new();
let mut game = Game::new(rules, Seat::East);
let mut rng = rand::rng();
let bots = Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as _));
let mut table = Table::new(game.deal(&mut rng), bots);
while table.step()? != Step::Finished {}
let result = table.round().result().expect("finished");
game.settle(result);
match result.winner {
    Some(seat) => println!("{seat} wins with {} 胡", result.hu),
    None => println!("exhausted"),
}
# }
# Ok::<(), tsap_oo::RoundError>(())
```

## Examples

- `arena`: bots play many rounds and report the outcomes
  (`cargo run --release --example arena -- 1000`)

[mod-rules]: https://docs.rs/tsap-oo/latest/tsap_oo/rules/index.html
[mod-round]: https://docs.rs/tsap-oo/latest/tsap_oo/round/index.html
[mod-game]: https://docs.rs/tsap-oo/latest/tsap_oo/game/index.html
[mod-view]: https://docs.rs/tsap-oo/latest/tsap_oo/view/index.html
[mod-strategy]: https://docs.rs/tsap-oo/latest/tsap_oo/strategy/index.html
[mod-driver]: https://docs.rs/tsap-oo/latest/tsap_oo/driver/index.html
[mod-heuristic]: https://docs.rs/tsap-oo/latest/tsap_oo/heuristic/index.html
[`Seat`]: https://docs.rs/tsap-oo/latest/tsap_oo/enum.Seat.html
[`Rules`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.Rules.html
[`Round`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.Round.html
[`Phase`]: https://docs.rs/tsap-oo/latest/tsap_oo/enum.Phase.html
[`Offer`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.Offer.html
[`Claim`]: https://docs.rs/tsap-oo/latest/tsap_oo/enum.Claim.html
[`ClaimOptions`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.ClaimOptions.html
[`Event`]: https://docs.rs/tsap-oo/latest/tsap_oo/enum.Event.html
[`RoundResult`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.RoundResult.html
[`Game`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.Game.html
[`View`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.View.html
[`Strategy`]: https://docs.rs/tsap-oo/latest/tsap_oo/trait.Strategy.html
[`Table`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.Table.html
[`HeuristicBot`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.HeuristicBot.html
