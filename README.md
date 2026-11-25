# tsap-oo

[![Crates.io](https://img.shields.io/crates/v/tsap-oo)](https://crates.io/crates/tsap-oo)
[![Docs.rs](https://docs.rs/tsap-oo/badge.svg)](https://docs.rs/tsap-oo)
[![Build Status](https://github.com/jdh8/tsap-oo/actions/workflows/rust.yml/badge.svg)](https://github.com/jdh8/tsap-oo)

十胡 (Taiwanese *tsa̍p-ôo*, "ten hu") is the Four Color Cards game of Taiwan:
four players, twenty cards each, and a hand wins when all twenty-one cards
lay out as melds worth 10 胡 or more.  This crate implements the Taiwanese
rules on top of the [susik] deck: a runtime-checked `Round` state machine
that resolves every public card in priority order (win, forced kong, pong,
eat), a `Game` scoreboard, an information-hygienic `View`, the `Strategy`
trait, a greedy `HeuristicBot`, and a `Table` that drives bots.  The name is
the Taiwanese reading, which no other game shares.

Play it in the browser at [jdh8.github.io/tsap-oo][web]; the front end in
[`web/`](web/) is the engine compiled to WebAssembly.  [`tauri/`](tauri/)
puts the same page in a desktop window, against opponents that search:
`cargo desktop` from the repository root.

[susik]: https://github.com/jdh8/susik
[web]: https://jdh8.github.io/tsap-oo/

## The rules implemented

- 112 cards; the dealer gets 21, the others 20, and the dealer discards
  first.  Play runs counterclockwise.  The last six cards of the pile are
  dead: reaching them ends the round with no winner and the same dealer.
  At a table of three or two the twenty cards of each empty seat are dead
  as well, so every table draws 25 cards a round; no source says what a
  smaller table changes, and [`doc/variants.md`](doc/variants.md) has why
  this was chosen.
- A draw is turned face up and never enters a hand: only dealt cards are
  concealed, and a meld laid on the table is never added to or rearranged.
  Any public card — a draw or a discard — can
  be won on by anyone, must be konged by whoever holds its concealed
  triplet, may be ponged by anyone holding its pair, and may be eaten by
  the drawer or by the next player into 將士象, 車馬炮, soldiers of three
  or four colours, or a pair — even by a seat holding the pair, which may
  pong instead.  Win beats kong beats pong beats eat; equal
  claims go to the earlier seat in turn order.  Winning is compulsory.
- Generals (將帥) are each a meld by themselves, 1 胡 apiece however many
  are held.  A drawn general that nobody wins on or kongs must be eaten by
  its drawer, who lays it on the table alone or, holding the 士象, as 將士象
  if they prefer (`ClaimOptions::forced`: no pass).  Generals are never
  ponged or eaten into a pair, and may not be discarded.  Only three
  generals in hand kong the fourth; ones on the table do not count.
- A seat with nothing to discard is a 相公: the round ends, they pay the
  threshold, 10 胡, to each other seat, and the same dealer deals again.
  No claim that would lead there is allowed (`ClaimOptions::blocked`), so it
  only follows a drawn general or a forced kong.
- Three or four identical cards in hand are never split, for a claim or in
  the winning layout: 車車車馬炮 (大肚) is a triplet with 馬炮 loose, and wins
  only on the fourth 車, as 車馬炮 beside the triplet.  One of them may still
  be discarded, leaving an ordinary pair: no write-up forbids it, and unlike
  a split for a claim, which the meld on the table gives away, a discard in
  the river cannot be traced to its hand.  The card just claimed
  cannot be discarded in the same turn.  A claim must leave another card to
  discard that is not a general, so a hand cannot meld itself out below the
  threshold.
- 胡: lone general 1, 將士象 2, 車馬炮 1, soldiers of three colours 3 and of
  four 5, a triplet 3 concealed or 1 ponged, a quad 8 concealed or 6 konged,
  a pair 0.  "Concealed" means dealt and never shown; every meld formed with
  a public card is exposed.  Ten to win; the winner collects their 胡 from
  each other seat and deals next.

These follow the mainstream Taiwanese table (the Chinese Wikipedia article
and the common Taiwanese write-ups agree on it).  House variants — a
different value for four-colour soldiers, the 將 parity rule — are not
modelled; all values are fields of `Rules`.

8 胡 is `Rules::eight()`: the same table with eight to win, and so eight for
a 相公.

Two settlement rules are fields of `Rules`, both off in `Rules::new`.  With
a 底 (`Rules::base`) each seat pays the 底 and the 胡 above the threshold
instead of the whole 胡: at a 底 of 1, 13 胡 is paid 1 + 3, and a 相公 pays
the threshold and the 底, 11.  With 點花 (`Rules::flower`) the top card of
the pile is turned at a win, and each card like it in the winner's layout,
in colour and face, is one 花胡: one 胡 more from each seat, three at most.
Both are as Board Game Arena's Chinese help has them.  The bots do not
know either rule yet ([`doc/variants.md`](doc/variants.md), "底 and 點花").

The table is `Rules::players`, two to four, four by default; the write-ups
allow two to five.  A smaller table deals and scores the same: twenty cards
each, one more to the dealer, the rest to the pile, and a win or a 相公
settled with each other seat.  The five-player game 8 胡 is often named for
is not modelled (`Seat::COUNT` is four).  The bots were tuned at four and
at 10 胡; `ExpectBot` has tables of its own for two players, at 10 and at
8 胡, and plays three players and four players' 8 胡 by those of four at 10:
[`doc/variants.md`](doc/variants.md) lists what is missing.

Open questions, played provisionally:

- The 相公 penalty of 10 胡 to each seat is from the Taiwanese write-ups;
  that the round then ends and the dealer stays is a guess, as they do not
  say.
- A fourth general drawn by another seat is konged by the holder of the
  triplet (the forced kong outranks the drawer eating it) and scores as an
  exposed kong.
- That the drawer of a general may choose between the lone general and
  將士象 is from Board Game Arena, which lists both as eats; the Taiwanese
  write-ups only say it must be eaten.
- No adding to a pong: a ponged triplet never takes its fourth card.
  Board Game Arena ("Once a meld has been played to the table, it cannot be
  added to or changed") and the family rules of T. N. Truong and Wei-Hwa Huang say so
  outright; the Taiwanese write-ups only ever kong a triplet held in hand.
  The few pages that add to a pong mix in mahjong rules.

## Modules

- the crate root: [`Seat`], numbered in turn order
- [`rules`][mod-rules]: [`Rules`], the scoring table
- [`round`][mod-round]: one deal — [`Round`], [`Phase`], [`Offer`],
  [`Claim`], [`ClaimOptions`], [`Event`], [`RoundResult`]
- [`game`][mod-game]: the scoreboard — [`Game`]
- [`view`][mod-view], [`strategy`][mod-strategy], [`driver`][mod-driver]:
  [`View`], [`Strategy`], [`Table`]
- [`heuristic`][mod-heuristic]: [`HeuristicBot`]
- `montecarlo`: `MonteCarloBot`, which samples the unseen cards and plays
  each choice out (feature `rand`)

## Feature flags

- `rand` (default): shuffled dealing (`Round::deal`, `Game::deal`) and the
  sampling `MonteCarloBot`

## Quick start

```rust
use tsap_oo::{Game, HeuristicBot, Rules, Seat, Step, Table};

# #[cfg(feature = "rand")] {
let rules = Rules::new();
let mut game = Game::new(rules, Seat::ALL[0]);
let mut rng = rand::rng();
let bots = Seat::ALL.map(|_| Some(Box::new(HeuristicBot::new(&rules)) as _));
let mut table = Table::new(game.deal(&mut rng), bots);
while table.step()? != Step::Finished {}
let result = table.round().result().expect("finished");
game.settle(result);
match result.winner {
    Some(seat) => println!("seat {seat} wins with {} 胡", result.hu),
    None => println!("exhausted"),
}
# }
# Ok::<(), tsap_oo::RoundError>(())
```

## Examples

- `arena`: bots play many rounds and report the outcomes
  (`cargo run --release --example arena -- 1000`); a second argument seats
  a Monte Carlo bot of that many samples first, a third seeds the deals

[mod-rules]: https://docs.rs/tsap-oo/latest/tsap_oo/rules/index.html
[mod-round]: https://docs.rs/tsap-oo/latest/tsap_oo/round/index.html
[mod-game]: https://docs.rs/tsap-oo/latest/tsap_oo/game/index.html
[mod-view]: https://docs.rs/tsap-oo/latest/tsap_oo/view/index.html
[mod-strategy]: https://docs.rs/tsap-oo/latest/tsap_oo/strategy/index.html
[mod-driver]: https://docs.rs/tsap-oo/latest/tsap_oo/driver/index.html
[mod-heuristic]: https://docs.rs/tsap-oo/latest/tsap_oo/heuristic/index.html
[`Seat`]: https://docs.rs/tsap-oo/latest/tsap_oo/struct.Seat.html
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
