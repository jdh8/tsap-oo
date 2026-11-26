# Variants beyond the default: what is missing

Compiled 2026-11-21, base `4e461d0`; the same day the dead cards of a small
table changed ("The dead cards of a small table", base `107032c`), and every
number measured at a small table before that is marked "old rules".  Added
2026-11-22: under the new rules the small tables had their `Outlook`
measured anew and their `Eval` retrained (`eval.md`, "Tables for two and
three players").

The default is four players at 10 胡 (`Rules::new`).  The other choices are
8 胡 (`Rules::eight`), tables of two and three (`Rules::players`), and since
2026-11-26 a 底 and a 點花 at the settlement ("底 and 點花" below), all of
them offered under 「設定」 (Settings) on the web page and in the desktop app.
**The rules engine is complete for these variants; the bots are not:**
nearly all of the bots' constants, tables and measurements come from four
players at 10 胡 and are simply used as they are elsewhere.  A table of
five is not implemented.

| | Four players, 10 胡 | 8 胡 | Tables of two and three | A table of five |
| --- | --- | --- | --- | --- |
| Dealing, eats and pongs, settlement | Yes | Yes | Yes | No |
| The rules have a source | Yes | Yes | Only "can be played" | Yes; not implemented |
| `HeuristicBot`'s constants | Measured | Used as they are, not measured | Used as they are, not measured | |
| `Outlook` (the table model) | Measured | Measured; at four players 10 胡's is used, insufficient data | Measured; two players use their own at 10 and at 8 胡 (wins), three players four players' | |
| `Eval` (the table past the gate) | Trained, win | At four players 10 胡's is used, insufficient data | Trained; two players use their own at 10 and at 8 胡 (wins), three players are insufficient data and use four players' | |
| The points the hint shows | Throughout | Throughout; past the gate they are 10 胡's values, but at two players 8 胡's | Throughout; at two players that table's values, at three four players' | |
| A/Bs that count | Every change | Two | Ten under the new rules (M, N, W to Z, W8, Y8, W8′, Y8′) | |
| Measuring tools in `examples/` | Yes | `outlook` and `eval` | `outlook` and `eval` | |

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

## 底 and 點花

Added 2026-11-26, base `8173c01`.  Two settlement rules, both off in
`Rules::new` and both under 「設定」 on the web page and in the desktop app
(「結算」, 「點花」).  **The rules engine has them and no bot knows of
either.  One change has been measured under a 底 ("The DP reads the 底"
below): alone it is a win of +0.009 points a deal, and with the web's
tables, learnt on the whole 胡, a loss of 0.74 a deal.  It is not in
`main`.**

| Rule | Field | What it does | Source |
| --- | --- | --- | --- |
| 底 | `Rules::base`, `None` or the 底 | Each seat pays the 底 and the 胡 above the threshold in place of the whole 胡 (`Rules::pays`): at a 底 of 1, 13 胡 is paid 1 + 3.  A 相公 pays the threshold and the 底 (`Rules::foul`), 11 | Board Game Arena's Chinese help: 「跟所有其他玩家收取胡數減去門檻的點數」, with 「1 點押點」 added to every win and loss, and a 相公 at 「扣 11 點」 |
| 點花 | `Rules::flower` | At a win the top card of the pile is turned where it lies (`RoundResult::flower`); each card like it in the winner's layout, in colour and face, is one 花胡 (`RoundResult::flower_hu`), one 胡 more from each seat, three at most | The same page: 「從牌庫抽最上方的牌作為點花，若與玩家持有的牌相同，一張可多得一胡，稱為花胡」 |

The sources, quoted and dated, and the other schemes the write-ups have
(five cards turned, a card a player, a flat 底) are in
`../tsap-oo-book/research/settlement.md`.  The page offers a 底 of 1 only,
Board Game Arena's figure; the field takes any.

- **What our reading adds.**  The 花胡 are paid on top of the 胡 and do not
  count toward the threshold: the card is turned after the win.  "The same
  card" is the same colour and face, which is what makes three the most.
  A 相公 pays 11 to each seat; the English help can be read as 10 once and
  1 to each.
- **What is left out.**  With the pile empty Board Game Arena turns the
  round's first discard; here a win always leaves the dead cards, so the
  pile is never empty (`dead_tail` 0 turns no card).  The 點花 is not an
  `Event`: no seat acts after it.
- **The bots.**  `ExpectBot` prices a win at the whole 胡 from each seat
  and a 相公 at the threshold (`src/expect.rs`, `Payoff::Points`), and its
  `Outlook` and `Eval` were fitted to rounds paid that way; `HeuristicBot`'s
  weights were tuned on them too.  With a 底 of 1 a win of 12 胡 collects 3
  a seat and one of 13 collects 4, so a 胡 more is worth a third more, not
  a twelfth: the trade between a bigger hand and a faster one moves, and by
  how much is not measured.  The points the hint shows are still those of
  the whole 胡.  The 點花 is drawn after the round and is the same in
  expectation for any hand of singles, so it should move play little; that
  is not measured either.
- **`ab/`** has `BASE`, set on both sides as `THRESHOLD` and `PLAYERS`
  are; the control must have the field, so `b44ad28` or later.  There is no
  `FLOWER`: 點花 is left alone.  Every change to how a bot plays under a
  底 is its own run.

### The DP reads the 底: a win alone, a loss with the tables

Declared 2026-11-26, before any data.  The control is `b44ad28` throughout,
four players at 10 胡, `RESHUFFLE=8`, each run on the clock's seed base.

`Graph` pays a win `Rules::pays` and a 相公 `Rules::foul`, and takes off
`Outlook::loss`, which is measured in 胡, what a 底 leaves unpaid.  Every
win is at the threshold or above, so that is the same 9 for each at a 底
of 1: −12.26 is −3.26, exactly so while the other seats play as they did.
Without a 底 none of the three moves.  `Eval`'s tables are not retrained,
so past the gate a shallow graph has its wins in the new points and its
leaves in the old; `HeuristicBot` is not touched.

| Run | 底 | Candidate | Control | Deals | Declared | Seed base | Part | Result |
| --- | --- | --- | --- | ---: | --- | ---: | ---: | --- |
| `base-zero-web` | none | `EXPECT=new GATE_CARDS=10 SHALLOW=0` | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 2,000 | No deal parts | 1795672484 | 0 | As declared |
| `base-zero-desktop` | none | `EXPECT=new GATE_CARDS=14 SHALLOW=17` | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17` | 500 | No deal parts | 1795672583 | 0 | As declared |
| `base-leaves` | 1 | `EXPECT=new GATE_CARDS=10`: the DP in the gate, `HeuristicBot` past it | `BASELINE_EXPECT=1`: the same with the whole 胡 | 30,000 | A win | 1795672717 | 1,300 | +0.005 ± 0.004, 95% CI [−0.003, +0.013]: insufficient data |
| `base-leaves-2` | 1 | The same as `base-leaves` | The same | 200,000 | A win | 1795676472 | 8,782 | +0.009 ± 0.002, 95% CI [+0.006, +0.012]: win |
| `base-web-2` | 1 | `EXPECT=new GATE_CARDS=10 SHALLOW=0`: the web's bot | `BASELINE_EXPECT=new BASELINE_SHALLOW=0`: the web's bot today | 30,000 | Not known; only if `base-leaves` or `base-leaves-2` wins | 1795679206 | 21,979 | −0.740 ± 0.014, 95% CI [−0.767, −0.713]: loss |
| `base-web-bare-2` | 1 | `EXPECT=new GATE_CARDS=10`: the web's bot without the tables | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 30,000 | Not known; only if `base-leaves` or `base-leaves-2` wins | 1795685070 | 24,205 | +0.008 ± 0.013, 95% CI [−0.018, +0.034]: insufficient data |

Read 2026-11-26.  Without a 底 no deal parts, with the web's bot or the
desktop's.  With a 底 of 1 the change alone parts on 1,300 deals of 30,000
(among them a mean of +0.108, a standard deviation of 5.36 within a deal
and 2.75 between) and its seat wins 24.82% of the deals against the
control's 24.93%.  The CI straddles 0, which is neither a win nor a loss:
the two runs behind it did not start, and the change stays out of `main`.
At this spread an effect of +0.005 needs about 170,000 deals for a CI clear
of 0.

Declared 2026-11-26, before its data: `base-leaves-2` is the same run at
200,000 deals on a fresh seed base.  It gives its verdict alone; the 30,000
deals above are not pooled into it.  The two web runs wait on it as they
waited on the first.

Read 2026-11-26.  **Alone the change is a win.**  Over 200,000 deals it
parts on 8,782 (among them a mean of +0.200, a standard deviation of 5.49
within a deal and 2.57 between), +0.009 ± 0.002 points a deal, 95% CI
[+0.006, +0.012]; its seat wins 24.35% of the deals against the control's
24.45%, so it wins a little less often and collects more when it does.
**With the web's tables it is a heavy loss.**  As the web's bot, the DP
with `Eval`'s leaves past the gate, it parts on 21,979 deals of 30,000 and
loses 0.740 ± 0.014 a deal, 95% CI [−0.767, −0.713]; its seat wins 16.88%
of the deals against 25.01%.  The mechanism is the one declared above:
past the gate the graph's wins are in the 底's points (a win of 12 胡
collects 3 a seat) while its leaves, `Eval`'s values, are in the whole 胡
(a hand near winning is worth about 10 a seat), so a hand that wins is
worth less than one that stays a leaf, and the bot steers away from the
shapes that win.  A trace of the loss (`base-web-trace`, 2,000 deals, seed
base 1795694203, `TRACE` on; −0.846 ± 0.057, 1,451 part) finds no few
deals to blame: 912 splits are discards at a mean of −1.10 and 539 are
claims at −1.28, the largest single kind the control's pong passed over by
the candidate (56 deals, −3.35), and the mean loss a split grows from −0.8
in the first five decisions to −1.9 past twenty, where the leaves are
nearest.  **Without the tables the web's bot
is not told from the one it has.**  The DP in the gate reading the 底 and
`HeuristicBot` past it, against the web's bot today with its tables,
parts on 24,205 deals of 30,000 and is +0.008 ± 0.013, 95% CI [−0.018,
+0.034], insufficient data: under a 底 the tables as they are give the
web's bot nothing that can be measured.

What follows.  The change is a win only where `HeuristicBot` plays past
the gate, which neither client ships, and a loss as the web's bot, which
both do with a 底 on; so it stays out of `main` until `Eval` is retrained
on rounds paid with a 底 (then `base-web-2` again), or the 底 is read in
the gate only and the shallow graph keeps the whole 胡 (its own run on
the web's setting).  Neither is started.

`base-leaves` is the change alone.  The other two are what the web would
ship under a 底 before the tables are retrained: the tables as they are,
or none.  The deal counts are from the web's trial under the whole 胡
(`doc/eval.md`: 400 deals a minute, a standard error of 0.363 at 400
deals), since no trial runs beside the queue; a 底 pays less a round, so
the standard error in points should be smaller, by how much not known.
Not run: the desktop's gate under a 底, the tables retrained, and
`HeuristicBot`'s weights.

## What 8 胡 lacks

1. **The `Outlook` `ExpectBot` uses was measured at 10 胡.**  8 胡's table
   has been measured (`Outlook::new`; `research/outlook.md`, "8 胡"): within
   the gate h is 0.085 against 10 胡's 0.071 and L −11.30 against −12.26,
   and h × 1.5 takes up the calibration's bias just the same.  The A/B of
   switching to it (control `c922a91`) straddles 0 on both settings: web
   +0.022 ± 0.013, 95% CI [−0.004, +0.048], 40,000 deals, seed base
   1795400000; desktop +0.005 ± 0.024, [−0.041, +0.052], 20,000 deals, seed
   base 1795500000.  Insufficient data; `ExpectBot::new` keeps 10 胡's
   table at four players.  Two players at 8 胡 have their own, measured at
   8 胡 (item 2 of "What tables of two and three lack").
2. **`Eval` is 10 胡's table.**  The shallow graph past the gate uses it
   for the values of its leaves, so a hand 2 胡 from winning is taken for
   one 4 胡 away.  The A/B against "playing as `HeuristicBot` past the gate"
   (control `09eb4c4`) straddles 0 on both settings: web +0.006 ± 0.074, 95%
   CI [−0.138, +0.151], 10,000 deals, seed base 1795200000; desktop +0.080 ±
   0.049, [−0.015, +0.175], 20,000 deals, seed base 1795300000.
   Insufficient data; play is unchanged (`eval.md`, "8 胡").  That is four
   players; two players at 8 胡 have a table labelled at 8 胡, a win on
   both settings.
3. **At four and three players the points the hint shows past the gate are
   10 胡's values.**  The ordering was not measured to be worse than
   `HeuristicBot`'s, but the numbers themselves are not expected scores at
   8 胡.
4. **`HeuristicBot`'s constants have not been measured at 8 胡.**  The
   weights of lone cards, 搭子 and waits were all set on A/Bs at four
   players and 10 胡.
5. **Most measuring tools hard-code 10 胡.**  Of the 16 programs in
   `examples/` only `outlook` and `eval` take `THRESHOLD`; the rest use
   `Rules::new()` and must be made to take a threshold before 8 胡 can be
   measured.

## What tables of two and three lack

1. **The rules have no source.**  Every source found says two or three can
   play, and none writes out how the rules change with fewer players
   (`research/prior-art.md`, section C).  Dealing is as at a table of four,
   the same as Board Game Arena; the dead cards are our own decision ("The
   dead cards of a small table").  At a table of two the only opponent is
   both the previous seat and the next.
2. **Two players have their own `Outlook` and `Eval`, at 10 and at 8 胡;
   three players do not.**  Under the new rules both tables had `Outlook`
   measured anew (`research/outlook.md`, "Empty seats' hands are dead": e is
   0.74 at two and 0.42 at three, 0.28 at four), and each labelled and
   trained an `Eval` with it (`eval.md`, "Tables for two and three
   players").  The control is `07d5602` throughout:
   - Two players at 10 胡, own `Outlook` (times 1.3) and `Eval`: web +0.375
     ± 0.042 (against playing as `HeuristicBot` past the gate), desktop
     +0.320 ± 0.056 (against four players' table), both wins.
     `ExpectBot::new` and `Eval::of` use them at two players and 10 胡.
   - Three players at 10 胡, own `Outlook` (times 1.4) and `Eval`, against
     four players' tables: web −0.021 ± 0.025, desktop −0.095 ± 0.061, both
     insufficient data.  Four players' tables are used as they are; at a
     table of three four players' `Eval` is a win against `HeuristicBot`
     (+0.216 ± 0.051, web).
   - Two players at 8 胡, the tables of two players at 10 胡 against those
     of four players at 10 胡: web +0.302 ± 0.040, win; desktop +0.048 ±
     0.056, insufficient data.  The declaration was to switch only if both
     win, so they were not adopted.
   - Two players at 8 胡, own `Outlook` measured at 8 胡 (times 1.3) and an
     `Eval` labelled under it, against the tables of four players at 10 胡
     (control `1433f27`): web +0.445 ± 0.040, 95% CI [+0.367, +0.522],
     10,000 deals, seed base 1801700000; desktop +0.154 ± 0.037, [+0.081,
     +0.226], 8,000 deals, seed base 1801800000; both wins.
     `ExpectBot::new` and `Eval::of` use them at two players and 8 胡
     (`eval.md`, "An `Eval` for two players at 8 胡").
3. **Only at two players are the points the hint shows that table's
   values.**  Three players are shown the values of four players at 10 胡,
   1 point low on average (measured at 10 胡).
4. **At three it cannot be told whether it was `Outlook` or `Eval` that did
   not help.**  The candidate changed both at once; three players' table
   with four players' `Outlook` was not measured (the table was labelled
   under three players' `Outlook`, so the two have to change together).
5. **The numbers under the old rules stay in `research/outlook.md`, "Tables
   of two and three", all void.**  They include `ExpectBot`'s +0.106 and
   +0.118 against `HeuristicBot`.  Under the new rules the bot that ships
   beats `HeuristicBot` at 10 胡 (`opponents.md`, H runs, `RESHUFFLE=8`,
   control `1430e6b`): two players web +0.405 ± 0.042, 10,000 deals, seed
   base 1803800000; three players web +0.348 ± 0.063, 10,000 deals,
   1803900000; two players desktop +0.615 ± 0.067, 4,000 deals,
   1804000000; all wins.
6. **`HeuristicBot`'s constants are those of four players.**  One place is
   known to differ at a small table: not feeding the next seat (`FEED` 4)
   was a win at a table of two under the old rules (+0.007 ± 0.003) and
   insufficient data at four, so it was not adopted (`stronger-bot.md`).
7. **Most measuring tools hard-code four players.**  Only `outlook` and
   `eval` take `PLAYERS`; the rest as in item 5 of 8 胡.

8 胡 at a table of three can be chosen; it lacks what both lack, and has no
measurement at all.  8 胡 at a table of two has its own tables.

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
   for four players at 10 胡 and at 8 胡, and for two and three players at
   10 胡** (the small tables measured anew under the new rules).  8 胡 at
   the small tables remains.
3. Train an `Eval` for every variant (`eval.md`, application D).  Every
   table has 25 cards to draw, so the table need not be made longer.
   Changes play; an A/B for each.  Only when this is done are the hint's
   points those of the variant.  **Done for two and three players at ten
   胡**: a win at two, insufficient data at three.  The three tables of 8 胡
   remain; at two players and 8 胡 the web setting is already a win with the
   tables of two players at 10 胡.
4. Measure `HeuristicBot`'s constants anew for each variant.  At the small
   tables try `FEED` first.
5. A table of five, if wanted.
