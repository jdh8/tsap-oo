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
(「結算」, 「點花」).  **The rules engine has them.  Of the bots only
`ExpectBot`'s gate knows the 底 ("The DP reads the 底" below): read there
alone it is a win of +0.017 points a deal as the web's bot, and read past
the gate too, against tables learnt on the whole 胡, a loss of 0.74.
With tables learnt under a 底 of 1 the web's bot reads it throughout, a
win of +0.089 over the gate alone ("An `Eval` labelled under a 底").  The
desktop's bot wins both ways too: +0.038 for the 底 in its gate, and
+0.063 over that with the tables.  No bot knows the 點花.**

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
| `base-web-gate` | 1 | `EXPECT=new GATE_CARDS=10 SHALLOW=0`: the web's bot, the 底 read in the gate only | `BASELINE_EXPECT=new BASELINE_SHALLOW=0`, control `e0ac321` | 100,000 | A win, near +0.01 | 1795695031 | 5,945 | +0.017 ± 0.002, 95% CI [+0.012, +0.021]: win |
| `base-desktop-gate` | 1 | `EXPECT=new GATE_CARDS=14 SHALLOW=17`: the desktop's bot as `c5ce6de` has it, the 底 read in the gate only | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17`, control `e0ac321` | 40,000 | A win | 1795723537 | 8,084 | +0.038 ± 0.006, 95% CI [+0.026, +0.050]: win |

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
the web's setting).  The second is declared below.

Declared 2026-11-26, before its data: `base-web-gate`.  `ExpectBot::values`
builds the shallow graph with `Rules::base` cleared, so past the gate wins
and leaves are both in the whole 胡 and play there is the control's; in
the gate the whole graph reads the 底 as in `base-leaves-2`.  The control
is `main` HEAD, `e0ac321`, whose bots are `b44ad28`'s.  100,000 deals: the
deals that part should be the gate's alone, near `base-leaves-2`'s 4%,
where 30,000 deals did not resolve an effect of this size.

Read 2026-11-26.  **Read in the gate only, the 底 is a win as the web's
bot.**  Over 100,000 deals it parts on 5,945 (among them a mean of
+0.280, a standard deviation of 5.09 within a deal and 2.45 between),
+0.017 ± 0.002 points a deal, 95% CI [+0.012, +0.021]; its seat wins
25.19% of the deals against the control's 25.29%, so again it wins a
little less often and collects more when it does.  The deals that part,
6%, are near `base-leaves-2`'s 4%: past the gate play is the control's,
as declared.  `ExpectBot` reads the 底 so in `main`; the desktop's gate
under a 底, `Eval` retrained under one, and `HeuristicBot`'s weights are
not run.

Declared 2026-11-26, before its data: `base-desktop-gate`.  Since `c5ce6de`
`ExpectBot` reads the 底 in its gate at every setting, and only the web's
gate of 10 cards was measured.  This is the same change at the desktop's
setting, a gate of 14 cards and a graph two claims deep to 17: the
candidate is `c5ce6de` itself, in a worktree of its own, and the control
`e0ac321`, the last revision whose `ExpectBot` pays the whole 胡
throughout.  40,000 deals.  The desktop's trial under the whole 胡
(`eval.md`, batch 4) took 6.8 minutes for 400 deals of which 76% part,
0.07 seconds a play; a wider gate should part on more deals than the web's
6%, and at 15% the run takes about 4 hours and, at `base-web-gate`'s
spread, has a standard error near 0.006.  A win: nothing changes, and the
desktop's row is measured.  A loss: trace the deals that moved the most
before anything else.  Straddling 0: insufficient data, and the desktop
keeps what `main` has.

Read 2026-11-27.  **Read in the gate only, the 底 is a win as the
desktop's bot too.**  Over 40,000 deals it parts on 8,084 (among them a
mean of +0.190, a standard deviation of 5.48 within a deal and 1.93
between), +0.038 ± 0.006 points a deal, 95% CI [+0.026, +0.050]; its seat
wins 25.16% of the deals against the control's 25.55%, fewer and larger
wins as on the web.  One deal in five parts, against the web's 6% and the
15% guessed, and the run took 3.5 hours with the standard error where it
was put.  As declared: nothing changes, and the desktop's gate under a 底
is measured.

`base-leaves` is the change alone.  The other two are what the web would
ship under a 底 before the tables are retrained: the tables as they are,
or none.  The deal counts are from the web's trial under the whole 胡
(`doc/eval.md`: 400 deals a minute, a standard error of 0.363 at 400
deals), since no trial runs beside the queue; a 底 pays less a round, so
the standard error in points should be smaller, by how much not known.
Not run: the desktop's gate under a 底, the tables retrained, and
`HeuristicBot`'s weights.

### An `Eval` labelled under a 底

Declared 2026-11-26, before any data, base `c5ce6de`; everything is queued
through pueue from one frozen copy of the working tree
(`target/base-eval/queue.sh`).

**Why.**  Past the gate the bots still play for the whole 胡: the shallow
graph clears the 底, because `Eval`'s leaves were learnt on it.  That is
most of a round, every decision above 10 cards on the web and above 14 on
the desktop.  And under a 底 the tables as they are were not told from
`HeuristicBot` past the gate: `base-web-bare-2` is +0.008 ± 0.013 for the
bot without them, of which the 底 in its gate is +0.009 (`base-leaves-2`),
where under the whole 胡 the tables won +0.191 ± 0.046 (`eval.md`,
batch 4).  With a 底 of 1 a 胡 above the threshold is a third of a small
win, not a twelfth, so the hands worth building are not the same.

The method is that of `eval.md`, "Tables for two and three players", with
`examples/eval.rs` taking `BASE`:

- **The `Outlook` is not measured anew.**  `HeuristicBot` does not read
  the 底, so the rounds it plays are the same: h, e and the cards a draw
  stand, and L is the same in 胡, off which `Graph` takes what a 底 leaves
  unpaid.
- **Labels:** `BASE=1 EXPECT=1 eval label 2000 1806000000`, 2,000 rounds.
  `EXPECT=1` is the `Outlook` of `ExpectBot::new` at four players and 10 胡
  (the ranks' rates, h × 1), as in batch 3 of `generals.md`, whose
  labelling took 8,767 seconds.  Seat 1 plays `c5ce6de`'s `ExpectBot` at a
  gate of 14, the 底 read in its gate; every value is in the 底's points.
- **Training:** `eval train labels eval 20 1 4 16`.  The A/Bs are queued
  before the training is seen, so the table judged is fixed here as D = 16,
  which had the smallest validation RMSE every time before; the other two
  are reported.
- **Loss table:** not run.  The verdict is the A/B's.
- **Code.**  `Eval::under` marks tables as learnt under a 底, and
  `ExpectBot`'s shallow graph keeps `Rules::base` with tables learnt under
  the table's own, clearing it as before with any other.  `Eval::of` has
  no such table, so nothing that ships moves.  `ab/` takes `EVAL`'s file
  for learnt under `BASE`.
- **Zero deals part:** `base-eval-zero`, the web's bot at a 底 of 1 with
  the switch in and off, 2,000 deals.  The labelling starts only if it
  reads 0.
- **A/B:** control `c5ce6de`, four players at 10 胡, a 底 of 1,
  `RESHUFFLE=8`, each run on the clock's seed base, the other seats
  `HeuristicBot`.  The candidate reads the 底 throughout, its leaves from
  the new table; the control is the bot as it ships, the 底 in its gate
  and the whole 胡 past it.

  | Run | Setting | Candidate | Control | Deals | Declared | Seed base | Part | Result |
  | --- | --- | --- | --- | ---: | --- | ---: | ---: | --- |
  | `base-eval-zero` | Web | `BASE=1 EXPECT=new GATE_CARDS=10 SHALLOW=0` | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 2,000 | No deal parts | 1795701358 | 0 | As declared |
  | `base-web-eval` | Web | `BASE=1 EXPECT=new GATE_CARDS=10 SHALLOW=0 EVAL=…` | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 60,000 | A win | 1795711414 | 40,958 | +0.089 ± 0.008, 95% CI [+0.073, +0.104]: win |
  | `base-desktop-eval` | Desktop | `BASE=1 EXPECT=new GATE_CARDS=14 SHALLOW=17 EVAL=…` | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17` | 20,000 | Not known; starts only if `base-web-eval` wins | 1795735979 | 10,604 | +0.063 ± 0.012, 95% CI [+0.040, +0.086]: win |

  **Zero deals part** (2026-11-26, 96 seconds): seed base 1795701358, 0 of
  2,000 deals part, seat 1 winning 24.95% on both sides.  As declared; the
  labelling started.

- **How the verdicts are read.**  `base-web-eval` a win: the table is
  compiled in for four players at 10 胡 and a 底 of 1 (`Eval::of`, 359 KB
  more of wasm), with the runs of zero deals at the other tables and the
  replay of the wired bot that `eval.md` has for the tables of two; the
  desktop reads it too if `base-desktop-eval` wins, and if not `bot()`
  chooses by its setting.  A loss: trace the deals that moved the most
  first.  Straddling 0: insufficient data, the table stays out of the
  binary and `Eval::under` goes with it.
- **What a win would not show.**  The new table differs from the control's
  in the 底 and in the `Outlook` its labels were made under (the ranks'
  rates and h × 1, against the even draw and h × 1.5).  The second alone,
  on the whole 胡, read +0.019 ± 0.025 on the web and +0.020 ± 0.036 on
  the desktop, insufficient data both (`generals.md`, batch 3).  A win
  here is of the two together.
- **The deal counts.**  `base-web-2` and `base-web-bare-2` changed play
  past the gate under a 底 too: 30,000 deals, three in four parting, a
  standard error of 0.014 and 0.013.  So 60,000 deals are put at 0.010 and
  about 2.3 hours (`generals.md`'s 60,000 took 1.6 with 62% parting).  The
  desktop's spread under a 底 is not known; 20,000 deals took 3.5 hours
  with 46% parting and 5.7 with 76%.
- **The order in the queue:** `base-eval-zero`, the labels and the
  training, `base-web-eval`, `base-desktop-gate` (above), then
  `base-desktop-eval`; some 14 hours with every core.
- **Not covered.**  The tables of two, a table of three and 8 胡 under a
  底 keep the 底 in the gate only, and so does any 底 but 1.
  `HeuristicBot`'s weights under a 底 are not run: it has no candidate.

Read 2026-11-27.  **The labels and the tables.**  `eval label` took 9,476
seconds for its 2,000 rounds: 14,486 graphs, 1,950,147 nodes and
66,911,188 values, of which 194,778 nodes were held out and 60,212,183
values fitted.  The mean by public cards to come has a held-out RMSE of
1.103 of the 底's points.

| D | Train RMSE | Held-out RMSE |
| ---: | ---: | ---: |
| 1 | 0.690 | 0.726 |
| 4 | 0.333 | 0.397 |
| 16 | 0.136 | 0.245 |

D = 16 is again the smallest, and the table judged as declared
(`target/base-eval/eval-16.bin`); by depth its RMSE is 0.202 at the root's
values, 0.236 one claim deep and 0.262 two.  The whole 胡's table has
0.78 of its points (`eval.md`, batch 3): a point under a 底 is some three
of those at a small win, so the two are not compared.

**`base-web-eval` is a win.**  Over 60,000 deals it parts on 40,958
(among them a mean of +0.130, a standard deviation of 5.65 within a deal
and 1.27 between), +0.089 ± 0.008 points a deal, 95% CI [+0.073, +0.104];
its seat wins 24.89% of the deals against the control's 25.28%, so it wins
less often and collects more when it does, as the 底 in the gate did.  The
standard error was put at 0.010; the run took 3.4 hours, not 2.3.  The win
is over the bot that reads the 底 in its gate alone, itself +0.017 over
the whole 胡 throughout (`base-web-gate`), and it is of the 底 and the
labels' `Outlook` together, as said above.  So `base-desktop-eval` runs.

Declared 2026-11-27, before their data: **the table wired in.**
`Eval::of` gives four players at 10 胡 and a 底 of 1 the new table
(`src/eval-16-base.bin`, 359 KB, marked `under` its 底) and every other
table what it had; `bot()` reads `Eval::of`, so in the working tree the
desktop has it too, which stands only if `base-desktop-eval` wins.  The
wasm is not rebuilt beside the queue, so its size is not measured yet.
One pueue task after another from one frozen copy of the tree
(`base-wired`, `target/base-eval/wired.sh`), control `3f82e04`, whose
bots are `c5ce6de`'s, `RESHUFFLE=8`, clock seed bases, candidate
`EXPECT=new` and control `BASELINE_EXPECT=new` at the same gate and depth:

- **Zero deals part** at the tables the new one is not for, 2,000 deals
  each as the web's bot and 500 as the desktop's.

  | Setting | Table | 底 | Deals | Seed base | Part |
  | --- | --- | --- | ---: | ---: | ---: |
  | Web | four players, 10 胡 | none | 2,000 | 1795760142 | 0 |
  | Web | three players, 10 胡 | none | 2,000 | 1795760241 | 0 |
  | Web | two players, 10 胡 | none | 2,000 | 1795760342 | 0 |
  | Web | four players, 8 胡 | none | 2,000 | 1795760463 | 0 |
  | Web | two players, 8 胡 | none | 2,000 | 1795760580 | 0 |
  | Web | three players, 10 胡 | 1 | 2,000 | 1795760709 | 0 |
  | Web | two players, 10 胡 | 1 | 2,000 | 1795760826 | 0 |
  | Web | four players, 8 胡 | 1 | 2,000 | 1795760953 | 0 |
  | Web | four players, 10 胡 | 2 | 2,000 | 1795761068 | 0 |
  | Desktop | four players, 10 胡 | none | 500 | 1795761180 | 0 |
  | Desktop | two players, 10 胡 | 1 | 500 | 1795761320 | 0 |
  | Desktop | four players, 10 胡 | 2 | 500 | 1795761479 | 0 |

- **The replay.**  `base-web-eval`'s first 1,000 deals (seed base
  1795711414, control `c5ce6de`), once with `EXPECT=new` and once with
  `EXPECT=new EVAL=target/base-eval/eval-16.bin`: the output should be the
  same word for word, the wired bot being the one measured.  The task
  fails if the two differ.
- **The desktop** is settled by `base-desktop-eval`.  A win: `bot()`
  stands, and the desktop's replay is not run, as for the tables of two
  (`eval.md`, "Wired into the code").  Anything else: `bot()` gives the
  desktop `Eval::default` at this table, and that owes its own run of
  zero deals at the desktop's setting under a 底 of 1.

Read 2026-11-27.  **`base-desktop-eval` is a win.**  Over 20,000 deals it
parts on 10,604 (among them a mean of +0.118, a standard deviation of 5.61
within a deal and 1.12 between), +0.063 ± 0.012 points a deal, 95% CI
[+0.040, +0.086]; its seat wins 25.09% of the deals against the control's
25.36%.  The run took 6.7 hours.  The win is over the desktop's bot that
reads the 底 in its gate alone, itself +0.038 over the whole 胡 throughout
(`base-desktop-gate`), and as on the web it is of the 底 and the labels'
`Outlook` together.  So `bot()` stands: both clients play four players at
10 胡 and a 底 of 1 by the new table.

**The table wired in is as declared.**  No deal parts in any of the twelve
runs above, and the two replays are the same word for word: 699 of the
1,000 deals part, +0.093 ± 0.066.  The wasm is 1,709 KB, 1,452 KB
gzipped (`tsap-oo-wasm-size`, 2026-11-28), from 1,344 KB and 1,116 KB.

### A 底 at the tables of two and three

Declared 2026-11-27, before any data, base `7f0c316`; everything is queued
through pueue (`target/base-2/queue.sh`), the A/Bs from frozen copies of
the working tree.  Throughout: a 底 of 1, `RESHUFFLE=8`, clock seed bases,
the other seats `HeuristicBot`, and the control `7f0c316` unless said.

**Why.**  "Not covered" above: away from four players at 10 胡 the 底 is
read in the gate only, and that was measured at four players alone.  Two
players have tables of their own at 10 and at 8 胡, learnt on the whole
胡, and a table of three plays by four's.

- **Labels for two players,** as above with the `Outlook` those tables
  ship with: `PLAYERS=2 BASE=1 EXPECT=1.3 eval label 2000 1806100000` at
  10 胡 and the same with `THRESHOLD=8` and seed base 1806200000 at 8,
  each then `eval train labels eval 20 1 4 16` with D = 16 the table
  judged (`target/base-2/eval-10-16.bin`, `eval-8-16.bin`).  The two
  `Outlook`s are not measured anew, as at four.  The 8-胡 labels start
  only if `base-2-web-eval` wins.
- **Three players have no labels.**  The candidate is four players' table
  learnt under the 底, as three play by four's on the whole 胡.  Its
  labels were made under four players' `Outlook` (the ranks' rates, h × 1)
  and a table of three plays by the even draw and h × 1.5, so this pairs a
  table with an `Outlook` it was not learnt under.
- **No run of zero deals:** no code changes; `EVAL` and `BASE` are the
  switches `base-web-eval` used.

| Run | Table | Candidate | Control | Deals | Declared |
| --- | --- | --- | --- | ---: | --- |
| `base-2-gate` | Two, 10 胡, web | `EXPECT=new GATE_CARDS=10 SHALLOW=0`: the bot as it ships, the 底 in its gate | `BASELINE_EXPECT=new BASELINE_SHALLOW=0`, control `e0ac321`: the whole 胡 throughout | 100,000 | A win |
| `base-3-eval` | Three, 10 胡, web | `EXPECT=new GATE_CARDS=10 SHALLOW=0 EVAL=src/eval-16-base.bin` | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 30,000 | Not known |
| `base-2-web-eval` | Two, 10 胡, web | `EXPECT=new GATE_CARDS=10 SHALLOW=0 EVAL=…/eval-10-16.bin` | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 20,000 | A win |
| `base-2-desktop-eval` | Two, 10 胡, desktop | `EXPECT=new GATE_CARDS=14 SHALLOW=17 EVAL=…/eval-10-16.bin` | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17` | 8,000 | Not known; starts only if `base-2-web-eval` wins |
| `base-2-8-web-eval` | Two, 8 胡, web | As `base-2-web-eval` with `THRESHOLD=8` and `eval-8-16.bin` | The same | 20,000 | Not known; starts only if its labels were made |
| `base-2-8-desktop-eval` | Two, 8 胡, desktop | As `base-2-desktop-eval` with `THRESHOLD=8` and `eval-8-16.bin` | The same | 8,000 | Not known; starts only if `base-2-8-web-eval` wins |

- **How the verdicts are read.**  Each is its own.
  - `base-2-gate` a win: nothing changes, and the row of two is measured.
    A loss: trace the deals that moved the most, then the gate clears the
    底 at two, with its own run of zero deals.  Straddling 0: insufficient
    data, play stays.  The desktop's gate at two is not queued: it is owed
    only where `base-2-desktop-eval` is not a win, since a win there
    replaces the play it would measure.
  - An `eval` run a win: that table goes into `Eval::of` for its client,
    with the runs of zero deals at the other tables and the replay, as
    for four players; `bot()` chooses by its setting where only the web
    wins.  A loss: trace first.  Straddling 0: the table stays out.
  - `base-3-eval` a win: the desktop's run is declared next, and the arm
    of `Eval::of` reads bytes already compiled in.  Anything else: three
    players keep the 底 in the gate only, and labels of their own are
    not made, as on the whole 胡.
- **The deal counts.**  Under a 底 a standard error is about a quarter
  of the whole 胡's: at four players' desktop 0.012 at 20,000 deals
  against 0.049 at 8 胡.  The tables of two had 0.040 at 10,000 deals on the web
  and 0.037 at 8,000 on the desktop (`eval.md`, W8′ and Y8′), so 20,000
  and 8,000 deals are put near 0.008 and 0.010, where four players' wins
  were +0.089 and +0.063; about 1.5 and 3.5 hours.  Each labelling took
  2.4 to 4.8 hours before.  `base-2-gate` parts on few deals, as
  `base-web-gate` did at the same count.  Some 24 hours with every core
  if every run starts.
- **The wasm.**  Each table wired in is 359 KB more; with both of two
  players' that is six tables.  Whether the web takes them is decided
  when the verdicts and `tsap-oo-wasm-size` are read.
- **Not covered.**  Four players at 8 胡 under a 底, which have no tables
  of their own on the whole 胡 either; three players at 8 胡; any 底 but
  1; `HeuristicBot`'s weights, which still have no candidate.

Read 2026-11-28.  **`base-2-gate` is a win: read in the gate only, the 底
wins at a table of two as well.**  Seed base 1795798131, control
`e0ac321`.  Over 100,000 deals it parts on 7,989 (among them a mean of
+0.186, a standard deviation of 2.72 within a deal and 1.12 between),
+0.015 ± 0.001 points a deal, 95% CI [+0.012, +0.017]; its seat wins
49.77% of the deals against the control's 50.04%, fewer and larger wins
as at four.  8% of the deals part, near `base-web-gate`'s 6%, and the run
took 1.8 hours.  As declared: nothing changes, and the row of two is
measured.

Read 2026-11-28.  **`base-3-eval` is a win: as the web's bot, three
players play better by four's table learnt under the 底.**  Seed base
1795814816, control `7f0c316`.  Over 30,000 deals it parts on 22,434
(among them a mean of +0.150, a standard deviation of 4.62 within a deal
and 0.86 between), +0.112 ± 0.009 points a deal, 95% CI [+0.094,
+0.130]; its seat wins 33.06% of the deals against the control's 33.23%.
The run took 2.2 hours.  The win is over the bot that reads the 底 in
its gate alone, and it is of a table paired with an `Outlook` it was not
learnt under, as said above; it is larger than four players' own +0.089
all the same.  So the desktop's run is declared next, and the table goes
into `Eval::of` for the web with its runs of zero deals and its replay;
neither is run yet.

Read 2026-11-28.  **The labels and the tables of two at 10 胡**
(`base-2-label`).  `eval label` took 15,924 seconds for its 2,000 rounds,
seed base 1806100000: 16,961 graphs, 2,321,826 nodes and 65,535,950
values, of which 248,132 nodes were held out and 58,464,900 values
fitted.  The mean by public cards to come has a held-out RMSE of 0.658 of
the 底's points.

| D | Train RMSE | Held-out RMSE |
| ---: | ---: | ---: |
| 1 | 0.277 | 0.326 |
| 4 | 0.132 | 0.212 |
| 16 | 0.055 | 0.175 |

D = 16 is the smallest again, and the table judged as declared
(`target/base-2/eval-10-16.bin`, 359 KB); by depth its RMSE is 0.189 at
the root's values, 0.181 one claim deep and 0.165 two.  The labelling
took 4.4 hours, within the 2.4 to 4.8 it was put at.

Read 2026-11-28.  **`base-2-web-eval` is a win, as declared.**  Seed base
1795838870, control `7f0c316`.  Over 20,000 deals it parts on 14,789
(among them a mean of +0.087, a standard deviation of 2.90 within a deal
and 0.56 between), +0.064 ± 0.007 points a deal, 95% CI [+0.050,
+0.078]; its seat wins 49.20% of the deals against the control's 50.28%,
fewer and larger wins again.  The standard error was put near 0.008 and
the run took 1.6 hours.  The win is over the bot that reads the 底 in
its gate alone, itself +0.015 over the whole 胡 throughout
(`base-2-gate`); here the labels' `Outlook` is the control's, so it is of
the 底 alone.  So `base-2-desktop-eval` and the 8-胡 labels run, and the
table is owed to `Eval::of` for the web, with the runs of zero deals and
the replay, once the wasm's 359 KB are decided.

Declared 2026-11-28, before its data: **`base-3-desktop-eval`,** which
`base-3-eval`'s win calls for.  Three players at 10 胡, a 底 of 1, the
desktop's bot with four players' table learnt under the 底, queued
through `ab/queue.sh` behind what is in flight:

| Run | Table | Candidate | Control | Deals | Declared |
| --- | --- | --- | --- | ---: | --- |
| `base-3-desktop-eval` | Three, 10 胡, desktop | `EXPECT=new GATE_CARDS=14 SHALLOW=17 EVAL=src/eval-16-base.bin` | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17`, control `7f0c316` | 10,000 | Not known |

- **How it is read.**  A win: `Eval::of` gives three players at 10 胡
  under a 底 of 1 four's table for both clients, one arm.  Anything else:
  only the web takes it, `bot()` choosing by its setting, and the
  desktop keeps the 底 in the gate at three.  A loss is traced first.
- **The deal count.**  A deal's standard error on the desktop was 0.87 of
  the web's at four players (0.012 at 20,000 deals against 0.008 at
  60,000), and three's on the web was 0.009 at 30,000, so 10,000 deals
  are put near 0.014, where four players' win on the desktop was +0.063.
  About 4.5 hours: four's 20,000 took 6.7, and on the web a deal of three
  took 1.3 times a deal of four.
- **The wiring waits for the desktop's runs.**  Which clients an arm of
  `Eval::of` serves is this run's to say for three and
  `base-2-desktop-eval`'s for two, so no table is wired in before they
  are read, and one set of runs of zero deals then covers them all.

Read 2026-11-29.  **`base-2-desktop-eval` is a win: the desktop's bot at
a table of two plays better by the tables of two learnt under the 底.**
Seed base 1795844568, control `7f0c316`.  Over 8,000 deals it parts on
4,915 (among them a mean of +0.054, a standard deviation of 2.86 within a
deal and 0.47 between), +0.033 ± 0.010 points a deal, 95% CI [+0.014,
+0.052]; its seat wins 49.96% of the deals against the control's 50.65%.
The standard error was put near 0.010 and the run took 3.4 hours.  The
win is half the web's +0.064, as four players' desktop win (+0.063) was
below its web's (+0.089).  So the table of two at 10 胡 goes into
`Eval::of` for both clients, one arm, and the desktop's gate at two under
a 底 is not owed.

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
