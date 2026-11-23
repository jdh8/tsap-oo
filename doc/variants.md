# Variants beyond the default: what is missing

Compiled 2026-11-21, base `4e461d0`; the same day the dead cards of a small
table changed ("The dead cards of a small table", base `107032c`), and every
number measured at a small table before that is marked "old rules".  Added
2026-11-22: under the new rules the small tables had their `Outlook`
measured anew and their `Eval` retrained (`eval.md`, "Tables for two and
three players").

The default is four players at ten 胡 (`Rules::new`).  The other choices are
八胡 (`Rules::eight`) and tables of two and three (`Rules::players`), all of
them offered under 「設定」 (Settings) on the web page and in the desktop app.
**The rules engine is complete for these variants; the bots are not:**
nearly all of the bots' constants, tables and measurements come from four
players at ten 胡 and are simply used as they are elsewhere.  A table of
five is not implemented.

| | Four players, ten 胡 | 八胡 | Tables of two and three | A table of five |
| --- | --- | --- | --- | --- |
| Dealing, eats and pongs, settlement | Yes | Yes | Yes | No |
| The rules have a source | Yes | Yes | Only "can be played" | Yes; not implemented |
| `HeuristicBot`'s constants | Measured | Used as they are, not measured | Used as they are, not measured | |
| `Outlook` (the table model) | Measured | Measured; ten 胡's is used, insufficient data | Measured; two players at ten 胡 use their own (win), the rest four players' | |
| `Eval` (the table past the gate) | Trained, win | Ten 胡's is used, insufficient data | Trained; two players at ten 胡 use their own (win), three players are insufficient data and use four players' | |
| The points the hint shows | Throughout | Throughout; past the gate they are ten 胡's values | Throughout; at two players and ten 胡 that table's values, elsewhere four players' | |
| A/Bs that count | Every change | Two | Eight under the new rules (M, N, W to Z, W8, Y8) | |
| Measuring tools in `examples/` | Yes | `outlook` only | `outlook` and `eval` | |

## What is there

- **The rules.**  The threshold and the 相公 penalty follow
  `Rules::threshold`; the size of the table follows `Rules::players`, with
  dealing and settlement the same as at four (20 cards each, 21 to the
  dealer, a win or a 相公 settled with each other seat).  The dead cards are
  the last 6 plus the 20 of each empty seat, so every table draws 25 cards
  (the next section).  The invariants of `tests/round.rs` run at tables of
  two and three as well.
- **Both clients.**  「設定」 → 「玩法」 (variant), 「人數」 (players); the
  threshold in the help follows, and the seats are named by the number of
  players (a table of two has only 「對家」, the seat opposite).
- **The bots read the rules wherever they can.**  `HeuristicBot` counting
  "how many 胡 short", `ExpectBot`'s DP judging a win, the pruning, the 相公
  penalty: all use the threshold of `Rules`; the DP's payoff is multiplied
  by the seats at the table.
- **The switches of `ab/`.**  `THRESHOLD` and `PLAYERS`, changed on both
  sides at once.

## The dead cards of a small table

Changed 2026-11-21, base `107032c`.  **Empty seats' hands are dead:** the
dead cards are the last 6, plus the 20 each empty seat would have been dealt
(`Rules::dead`).

| Players | Dealt | Dead | To draw | Dead before | To draw before |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 81 | 6 | 25 | 6 | 25 |
| 3 | 61 | 26 | 25 | 6 | 45 |
| 2 | 41 | 46 | 25 | 6 | 65 |

### Why

Before, `dead_tail` ignored the number of players: always 6 cards, dealing
as at a table of four like Board Game Arena.  None of the sources found
says what changes with fewer players (`research/prior-art.md`, section C),
so the dead cards of a small table were our own choice all along; the old
choice made a small table's pile 20 to 40 cards longer, and the question is
what those cards did.

Measurement (`examples/length.rs` of `research/dead-tail.patch` on
`107032c`, `length 2000 7`): `HeuristicBot` self-play, 2,000 rounds a row,
setting `dead_tail` directly.  The standard error of the exhaustion rate is
about 0.4 percentage points.

| Players | Dead | To draw | Exhausted | Mean draws | Median | 90th percentile | Draws a player |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 6 | 25 | 3.2% | 12.1 | 12 | 20 | 3.0 |
| 3 | 6 | 45 | 0.0% | 12.3 | 11 | 21 | 4.1 |
| 3 | 16 | 35 | 0.1% | 12.3 | 11 | 21 | 4.1 |
| 3 | 20 | 31 | 1.1% | 12.2 | 11 | 21 | 4.1 |
| 3 | **26** | 25 | 3.5% | 12.1 | 11 | 21 | 4.0 |
| 3 | 30 | 21 | 8.2% | 11.9 | 11 | 21 | 4.0 |
| 2 | 6 | 65 | 0.0% | 12.1 | 11 | 21 | 6.1 |
| 2 | 26 | 45 | 0.1% | 12.1 | 11 | 21 | 6.0 |
| 2 | 36 | 35 | 0.6% | 12.1 | 11 | 21 | 6.0 |
| 2 | **46** | 25 | 3.8% | 11.9 | 11 | 21 | 5.9 |
| 2 | 50 | 21 | 8.5% | 11.6 | 11 | 21 | 5.8 |

- **How many cards a round draws does not depend on the number of players.**
  All three tables average 12 draws, and nine rounds in ten end within 21.
  The extra cards of a small table are almost never drawn, and taking them
  away does not shorten a round.
- **The extra cards do one thing only: keep a small table from exhausting.**
  With only 6 held back, not one of 2,000 rounds at two or at three was
  exhausted, against 3.2% at four.  The exhausted round is part of the rules
  (no one wins and the dealer stays), and a small table in effect did not
  have it.
- **The same cards to draw give the same exhaustion rate.**  At 25 the three
  tables have 3.2%, 3.5% and 3.8%, within the standard error.  So what is
  aligned is the cards to draw, not the dead cards.
- **Each player still gets more turns than at four.**  The same 12 draws are
  shared among fewer players: 3.0 draws a player at four, 4.0 at three, 5.9
  at two.  The pile cannot change that.

Japanese three-player mahjong removes the two to eight of characters, so
that the wall and the number of turns come close to those of four players.
The same cannot be done here: with one colour fewer (28 cards) soldiers of
four colours cannot be formed, a table of two left with two colours would
not even have soldiers of three, and the deck is in `susik`.  Leaving the
empty seats' hands undealt at the bottom of the pile does the same thing
without touching the deck or the scoring; "the empty seats' hands" also
gives the number 20 an origin, where it would otherwise be one more constant
picked.

### What changed

- **`Rules::dead()`** is the number of cards that are never drawn:
  `dead_tail + 20 × (4 − players)`.  `Rules::dead_tail` is still 6 and now
  means the dead cards of a full table.  `Round` is exhausted once `dead()`
  cards are left; `tests/round.rs` checks that every table draws only 25
  cards and that the round is exhausted right after the 25th.
- **Everything that reads "draws left" now uses it:** `ExpectBot::draws` and
  the five programs in `examples/`.
- **The web page and the desktop app:** the number of dead cards shown
  beside the pile and the exhaustion condition in the help follow the number
  of players.
- **A table of four does not change at all.**  `dead()` is 6 at four.  For
  the zero deals that part see `research/outlook.md`, "Empty seats' hands
  are dead".
- **Dealing, settlement and the threshold are untouched.**

### What it does to the bots

- **`HeuristicBot` does not look at the pile,** so every move is the same as
  before; only the round is exhausted after 25 draws.
- **`ExpectBot` looks its tables up by draws left,** so the `Outlook`s
  measured for the small tables before (66 and 46 segments, measured over
  rounds with 65 and 45 cards to draw) are void altogether: under the new
  rules a table of two opens at "25 draws left", and that segment of the old
  table held values borrowed from rounds played into their late stage.  The
  old tables were removed from `src/expect.rs`; see item 4 of "What tables
  of two and three lack" below.
- **The shallow graph is usable throughout at a small table.**  It is used
  only within 25 draws left (`eval::DRAWS`); before, most rounds at a small
  table ended before that, and past the gate play was in effect
  `HeuristicBot`'s; now every table starts from 25.
- **`Eval` has to be retrained for each table.**  Cutting the pile only
  aligned how many cards a round draws; it did not give back the cards the
  empty seats would draw: each player's share of the draws at two is still
  twice that at four, and the cards that can be eaten go from 0.28 to 0.74.
  At a table of two four players' table undervalues every leaf by 3 points,
  and playing by it past the gate loses to playing as `HeuristicBot`
  (−0.184 ± 0.045); the table of two's own table is a win.  See "What
  tables of two and three lack" below.

## What 八胡 lacks

1. **The `Outlook` `ExpectBot` uses was measured at ten 胡.**  八胡's table
   has been measured (`Outlook::new`; `research/outlook.md`, "八胡"): within
   the gate h is 0.085 against ten 胡's 0.071 and L −11.30 against −12.26,
   and h × 1.5 takes up the calibration's bias just the same.  The A/B of
   switching to it (control `c922a91`) straddles 0 on both settings: web
   +0.022 ± 0.013, 95% CI [−0.004, +0.048], 40,000 deals, seed base
   1795400000; desktop +0.005 ± 0.024, [−0.041, +0.052], 20,000 deals, seed
   base 1795500000.  Insufficient data; `ExpectBot::new` keeps ten 胡's
   table.
2. **`Eval` is ten 胡's table.**  The shallow graph past the gate uses it
   for the values of its leaves, so a hand 2 胡 from winning is taken for
   one 4 胡 away.  The A/B against "playing as `HeuristicBot` past the gate"
   (control `09eb4c4`) straddles 0 on both settings: web +0.006 ± 0.074, 95%
   CI [−0.138, +0.151], 10,000 deals, seed base 1795200000; desktop +0.080 ±
   0.049, [−0.015, +0.175], 20,000 deals, seed base 1795300000.
   Insufficient data; play is unchanged (`eval.md`, "八胡").
3. **The points the hint shows past the gate are ten 胡's values.**  The
   ordering was not measured to be worse than `HeuristicBot`'s, but the
   numbers themselves are not expected scores at 八胡.
4. **`HeuristicBot`'s constants have not been measured at 八胡.**  The
   weights of lone cards, 搭子 and waits were all set on A/Bs at four
   players and ten 胡.
5. **Most measuring tools hard-code ten 胡.**  Of the 13 programs in
   `examples/` only `outlook` takes `THRESHOLD`; the rest use `Rules::new()`
   and must be made to take a threshold before 八胡 can be measured.

## What tables of two and three lack

1. **The rules have no source.**  Every source found says two or three can
   play, and none writes out how the rules change with fewer players
   (`research/prior-art.md`, section C).  Dealing is as at a table of four,
   the same as Board Game Arena; the dead cards are our own decision ("The
   dead cards of a small table").  At a table of two the only opponent is
   both the previous seat and the next.
2. **Two players at ten 胡 have their own `Outlook` and `Eval`; the other
   small tables do not.**  Under the new rules both tables had `Outlook`
   measured anew (`research/outlook.md`, "Empty seats' hands are dead": e is
   0.74 at two and 0.42 at three, 0.28 at four), and each labelled and
   trained an `Eval` with it (`eval.md`, "Tables for two and three
   players").  The control is `07d5602` throughout:
   - Two players at ten 胡, own `Outlook` (times 1.3) and `Eval`: web +0.375
     ± 0.042 (against playing as `HeuristicBot` past the gate), desktop
     +0.320 ± 0.056 (against four players' table), both wins.
     `ExpectBot::new` and `Eval::of` use them at two players and ten 胡.
   - Three players at ten 胡, own `Outlook` (times 1.4) and `Eval`, against
     four players' tables: web −0.021 ± 0.025, desktop −0.095 ± 0.061, both
     insufficient data.  Four players' tables are used as they are; at a
     table of three four players' `Eval` is a win against `HeuristicBot`
     (+0.216 ± 0.051, web).
   - Two players at 八胡, the tables of two players at ten 胡 against those
     of four players at ten 胡: web +0.302 ± 0.040, win; desktop +0.048 ±
     0.056, insufficient data.  The declaration was to switch only if both
     win, so four players' tables are used as they are.  **The web setting
     alone is a win and is not yet in use.**
3. **Only at two players and ten 胡 are the points the hint shows that
   table's values.**  Three players and two players at 八胡 are shown the
   values of four players at ten 胡: 1 point low on average at three, 3
   points low at two (measured at ten 胡).
4. **At three it cannot be told whether it was `Outlook` or `Eval` that did
   not help.**  The candidate changed both at once; three players' table
   with four players' `Outlook` was not measured (the table was labelled
   under three players' `Outlook`, so the two have to change together).
5. **The numbers under the old rules stay in `research/outlook.md`, "Tables
   of two and three", all void.**  They include `ExpectBot`'s +0.106 and
   +0.118 against `HeuristicBot`.  Under the new rules the overall
   difference of `ExpectBot` from `HeuristicBot` at a small table has not
   been measured directly.
6. **`HeuristicBot`'s constants are those of four players.**  One place is
   known to differ at a small table: not feeding the next seat (`FEED` 4)
   was a win at a table of two under the old rules (+0.007 ± 0.003) and
   insufficient data at four, so it was not adopted (`stronger-bot.md`).
7. **Most measuring tools hard-code four players.**  Only `outlook` takes
   `PLAYERS`; the rest as in item 5 of 八胡.

八胡 at a table of two or three can be chosen; it lacks what both lack, and
has no measurement at all.

## A table of five

Not implemented.  The five-player game the sources describe wins on 8 胡 or
more and lets a poor hand fold (國瑜企業社), and adds a set of 28 cards for
each player beyond four (English Wikipedia); see
`../tsap-oo-book/research/small-tables.md`.  To build it: `Seat::COUNT` is
4; the deck is in `susik` and has 112 cards; `Round` has no action to fold;
the bots and the measurements as in the two sections above.

## The order to fill them in

From cheap to dear, each item the tool of the next:

1. `examples/` take a threshold and a number of players (at least `outlook`,
   `eval`, `arena`).  No change to play.
2. Measure an `Outlook` for every variant, with `ExpectBot::new` choosing by
   `Rules`.  Changes play; an A/B for each.  This step also gives the
   numbers of `ExpectBot` against `HeuristicBot` at a small table.  **Done
   for four players at ten 胡 and at 八胡, and for two and three players at
   ten 胡** (the small tables measured anew under the new rules).  八胡 at
   the small tables remains.
3. Train an `Eval` for every variant (`eval.md`, application D).  Every
   table has 25 cards to draw, so the table need not be made longer.
   Changes play; an A/B for each.  Only when this is done are the hint's
   points those of the variant.  **Done for two and three players at ten
   胡**: a win at two, insufficient data at three.  The three tables of 八胡
   remain; at two players and 八胡 the web setting is already a win with the
   tables of two players at ten 胡.
4. Measure `HeuristicBot`'s constants anew for each variant.  At the small
   tables try `FEED` first.
5. A table of five, if wanted.
