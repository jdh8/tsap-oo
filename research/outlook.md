# The table model's constants and calibration

Measured 2026-11-18, base `c0aea83`.  Batch 2 of `doc/expectation.md`: the
single-player dynamic programming folds the other seats into four constants
(h, L, e, generals (將帥)) and the public cards a draw, measured from
`HeuristicBot` self-play; the measured constants then predict the chance to
win, which is compared with whether the hand did win.  No change to play.

## Answer

- **h and the public cards a draw both change a lot with the pile, so both
  became tables.**  h rises from 0.018 with 24 draws left to 0.12 at the
  end; the public cards a draw fall from 3.2 to 1.3.  A single average
  (h 0.071, 2.14 cards) overcounts T by more than 40% when the hand is
  short.
- **e is 0.277, not the 0.45 guessed**: fewer cards come to the seat to eat
  than thought.  L is −12.3, and the chance that a general is the seat's own
  draw is 0.216.
- **The predictions are high throughout, all in one direction.**  Over
  22,853 decisions the predicted win rate is 0.382 and the actual 0.291;
  every one of the ten bins but the lowest is high.  The expected score is
  predicted at +7.6 and is actually +3.0.  Two tables do better than two
  averages, but only a little.
- **h times 1.5 takes up most of the bias**: 0.306 against 0.290 overall,
  the four lowest bins inside their 95% intervals, the score +3.4 against
  +2.9.  e times 0.7 and the cards a segment times 0.8 each move it only a
  little.  So by the gate, the bias has a direction and one constant
  explains it: **batch 2 passes.**
- The bias left is in the bins that predict high (0.04 to 0.09), perhaps
  because the DP reckons by its own play while it is `HeuristicBot` that
  plays.

## Method

`examples/outlook.rs`.  Four `HeuristicBot`s in self-play; after each round,
every public card (a draw, or a card discarded from a hand; a draw nobody
wants is also recorded as a `Discard`, which does not count a second time)
and each seat's count of concealed cards at that moment are rebuilt from
`Round::events`.  Each card counts once for each of the four seats; "inside
the gate" means that seat holds 10 concealed cards or fewer, the positions
the dynamic programming would be used in.

- **h**: the card let another seat win.  Split by draws left.
- **L**: the 胡 another seat wins with, negated.
- **e**: a card that is not a general and that nobody wins on comes to this
  seat to eat: its own draw; or the previous seat's draw that the previous
  seat did not eat; or the previous seat's discard; and no other seat pongs
  or kongs it.
- **Generals**: a general nobody wins on is this seat's own draw.
- **Public cards a segment**: a segment is the discards before a draw plus
  that draw, counted only over the rounds that reach that draw (the last
  segment over those that reach exhaustion), so a round that ends early is
  not counted twice, here and in h.  Segment 0 is the discards after the
  last draw and before the round is exhausted.

`Outlook` therefore became two tables by draws left (`end`, `cards`) plus
three numbers; `Graph::values` now takes the draws left and lays out from
the tables the h before each public card, their total being the cards of
segments 0 to that one added up, less the card being decided on.  A segment
with fewer than 500 cards inside the gate takes the h of the nearest
segment: segment 25, straight off the deal, takes segment 24's, and the
last, segment 0, takes segment 1's.

The calibration plays new rounds on another seed.  At every discard that
leaves 10 cards or fewer it builds the graph from the hand as the discard
actually left it and sweeps twice: the payoff is the win (1 or 0) and the
score.  The decisions go into ten bins by predicted chance to win, to see
whether the Wilson 95% interval of the actual win rate covers the mean
prediction.  Decisions of the same round are correlated, so the intervals
are too narrow.

Constants: `outlook 10000 20` (10,000 rounds, seed 20).  Calibration: the
10,000 rounds of the same run, seed 21.  Calibration with one constant
changed: `outlook 0 22 10000` (seed 23, starting from `Outlook::default()`).
All three seeds were fixed before the runs, and so were the round counts.

## Constants

10,000 rounds, seed 20.

| | All | Inside the gate |
| --- | ---: | ---: |
| h (per card) | 0.0276 | 0.0711 |
| L | −12.20 | −12.26 |
| e | 0.2757 | 0.2765 |
| A general is the seat's own draw | 0.2500 | 0.2156 |
| Public cards a draw | 2.136 | |

Inside the gate the generals come to less than 1/4: a hand inside the gate
has eaten and ponged a lot, and after an eat or a pong the next card is the
next seat's draw, so the seat's own turn to draw comes round less often.

By draws left (the cards a segment are not split by the gate):

| Draws left | Segments | Cards a segment | h, all | h, inside the gate | Cards inside the gate |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 25 | 9,982 | 3.235 | 0.0011 | 0.0000 | 92 |
| 24 | 9,909 | 2.731 | 0.0031 | 0.0175 | 513 |
| 22 | 9,513 | 2.392 | 0.0104 | 0.0269 | 2,414 |
| 20 | 8,734 | 2.158 | 0.0205 | 0.0441 | 5,764 |
| 18 | 7,574 | 1.945 | 0.0345 | 0.0513 | 9,090 |
| 16 | 6,201 | 1.821 | 0.0436 | 0.0582 | 11,043 |
| 14 | 4,854 | 1.716 | 0.0543 | 0.0666 | 10,906 |
| 12 | 3,678 | 1.626 | 0.0676 | 0.0776 | 9,697 |
| 10 | 2,621 | 1.572 | 0.0775 | 0.0851 | 7,899 |
| 8 | 1,853 | 1.495 | 0.0930 | 0.0966 | 6,193 |
| 6 | 1,233 | 1.430 | 0.0917 | 0.0983 | 4,325 |
| 4 | 805 | 1.369 | 0.1068 | 0.1099 | 2,902 |
| 2 | 511 | 1.311 | 0.1209 | 0.1217 | 1,865 |
| 1 | 401 | 1.259 | 0.1132 | 0.1126 | 1,457 |
| 0 | 326 | 0.279 | 0.0826 | 0.0803 | 299 |

All 26 segments are `END` and `CARDS` in `src/expect.rs`.

h inside the gate, by the fewest concealed cards another seat holds:

| Fewest concealed cards | 2 | 4 | 6 | 8 | 10 | 12 | 14 | 16 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| h | 0.157 | 0.119 | 0.103 | 0.080 | 0.059 | 0.041 | 0.018 | 0.010 |
| Cards | 795 | 4,559 | 12,664 | 22,649 | 16,644 | 10,479 | 4,746 | 388 |

A fifteenfold spread, wider than the sevenfold by the pile (the chance of
being won on in `research/safe-cards.md` behaves the same way).  This is the
dimension h should gain next.

## Calibration

10,000 rounds, seed 21, 22,853 decisions.  The three table models share one
graph:

| | Predicted win rate | Actual | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: |
| One h, one count of cards a draw | 0.403 | 0.291 | 8.20 | 2.98 ± 0.29 |
| A table of h, one count of cards | 0.402 | 0.291 | 7.97 | 2.98 |
| Two tables | 0.382 | 0.291 | 7.57 | 2.98 |

Two tables, in ten bins:

| Bin | Decisions | Predicted | Actual | 95% interval | Covered | Predicted score | Actual score |
| ---: | ---: | ---: | ---: | --- | --- | ---: | ---: |
| 0 | 3,809 | 0.022 | 0.021 | [0.017, 0.026] | yes | −8.33 | −9.15 |
| 1 | 2,587 | 0.153 | 0.095 | [0.084, 0.107] | no | −3.62 | −6.62 |
| 2 | 3,036 | 0.250 | 0.169 | [0.156, 0.183] | no | 0.86 | −3.02 |
| 3 | 2,786 | 0.349 | 0.255 | [0.239, 0.272] | no | 5.48 | 0.98 |
| 4 | 2,988 | 0.450 | 0.345 | [0.328, 0.362] | no | 10.24 | 4.97 |
| 5 | 2,397 | 0.547 | 0.423 | [0.404, 0.443] | no | 15.35 | 9.15 |
| 6 | 1,743 | 0.652 | 0.493 | [0.470, 0.517] | no | 20.84 | 13.08 |
| 7 | 2,462 | 0.751 | 0.592 | [0.573, 0.612] | no | 25.13 | 17.45 |
| 8 | 1,020 | 0.831 | 0.704 | [0.675, 0.731] | no | 29.11 | 22.91 |
| 9 | 25 | 0.915 | 0.760 | [0.566, 0.885] | no | 35.00 | 27.28 |

The shape is right: the actual win rate rises monotonically with the
prediction.  The bias is all upward; the ratio of actual to predicted rises
from 0.62 in bin 1 to 0.85 in bin 8, so the less hope a hand has, the more
it is overestimated.

### Which constant

Starting from the measured constants and changing one at a time (10,000
rounds, seed 23, 22,939 decisions):

| | Predicted | Actual | Bins covered | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: | ---: |
| Measured | 0.379 | 0.290 | 1/10 | 7.43 | 2.85 ± 0.29 |
| h × 1.5 | 0.306 | 0.290 | 5/10 | 3.43 | 2.85 |
| Cards a segment × 0.8 | 0.356 | 0.290 | 1/10 | 6.80 | 2.85 |
| e × 0.7 | 0.369 | 0.290 | 1/10 | 6.98 | 2.85 |

h × 1.5, in ten bins:

| Bin | Decisions | Predicted | Actual | 95% interval | Covered |
| ---: | ---: | ---: | ---: | --- | --- |
| 0 | 5,134 | 0.031 | 0.034 | [0.030, 0.040] | yes |
| 1 | 4,005 | 0.149 | 0.146 | [0.135, 0.157] | yes |
| 2 | 3,150 | 0.248 | 0.249 | [0.234, 0.264] | yes |
| 3 | 3,101 | 0.350 | 0.366 | [0.350, 0.383] | yes |
| 4 | 2,320 | 0.445 | 0.420 | [0.400, 0.441] | no |
| 5 | 1,697 | 0.549 | 0.484 | [0.461, 0.508] | no |
| 6 | 2,100 | 0.651 | 0.585 | [0.564, 0.606] | no |
| 7 | 1,252 | 0.739 | 0.648 | [0.621, 0.674] | no |
| 8 | 177 | 0.828 | 0.763 | [0.695, 0.819] | no |
| 9 | 3 | 0.905 | 0.667 | [0.208, 0.939] | yes |

- **h is that constant.**  Counting less T or fewer eats discounts hopeless
  and hopeful hands about alike and leaves the shape as it is; only a larger
  h pulls the four low bins into their intervals.
- **Why h measures low is not yet known.**  What is measured is the average
  over every card; the DP's other approximations (`live` fixed at the root,
  the other seats' discards taken as uniform) all raise my chances, so the
  1.5 may also be making up for them.  h by the fewest concealed cards
  another seat holds is the first explanation to try: the hands calibrated
  are mostly in rounds where the other seats are close to waiting too.
- **The 1.5 was read off seed 23**; no other multiplier was swept, and
  `Outlook::default()` is still the measured values.  The A/B of batch 3
  does not use the rounds of seed 23.
- The overestimate left is concentrated in the bins predicting 0.45 or more,
  0.03 to 0.09 a bin.  The DP reckons on playing optimally from then on
  while the hands measured are `HeuristicBot`'s, so this direction is
  expected (`doc/expectation.md`, "Calibration").

## What is not accounted for

- Decisions of the same round are correlated, and the Wilson intervals are
  too narrow; the count of bins "covered" is a loose reading.
- "Comes to the seat to eat" looks only at position and at the other seats'
  pongs and kongs, not at whether the seat can eat the card: e is the
  probability the dynamic programming needs, not how often a seat eats.
- The constants come from `HeuristicBot`'s play; once `ExpectBot` sits at
  the table, the other seats' h stays the same, while its own e and generals
  may change.

## 8 胡

Measured 2026-11-20, base `c922a91`.  Step 2 of "The order to fill them in"
in `variants.md`: the same measurement, threshold 8.  `examples/outlook.rs`
takes `THRESHOLD`.

Constants: `THRESHOLD=8 outlook 10000 800` (10,000 rounds, seed 800; the
calibration is the 10,000 rounds of the same run, seed 801).  Calibration
with one constant changed: `THRESHOLD=8 outlook 0 802 10000` (seed 803,
starting from the 8 胡 tables).  The three seeds and the round counts were
all fixed before the runs.

| | 10 胡 | 8 胡 |
| --- | ---: | ---: |
| h inside the gate (per card) | 0.0711 | 0.0854 |
| L inside the gate | −12.26 | −11.30 |
| e inside the gate | 0.2765 | 0.2599 |
| A general is the seat's own draw, inside the gate | 0.2156 | 0.2088 |
| Public cards a draw | 2.136 | 2.278 |

h is higher than at 10 胡 in every segment, reaching 0.14 to 0.17 in the last
few; straight off the deal it is 0.035, where 10 胡 has 0.018.  All 26
segments are `END_8` and `CARDS_8` in `src/expect.rs`, and `Outlook::new`
picks by `Rules::threshold`.

The calibration (seed 801, 19,758 decisions) is biased in the same direction
as at 10 胡: the two tables predict a win rate of 0.406 against an actual
0.304, and a score of +7.54 against +2.75 ± 0.30.  With one constant changed
(seed 803, 19,288 decisions):

| | Predicted win rate | Actual | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: |
| Measured | 0.407 | 0.307 | 7.56 | 2.95 ± 0.30 |
| h × 1.5 | 0.320 | 0.307 | 3.43 | 2.95 |
| Cards a segment × 0.8 | 0.391 | 0.307 | 7.06 | 2.95 |
| e × 0.7 | 0.397 | 0.307 | 7.15 | 2.95 |

h × 1.5 takes up most of the bias at 8 胡 too, so `END_SCALE` stays.

### A/B

Declared (2026-11-20, written down before the run; control `c922a91`,
`RESHUFFLE=8`, `THRESHOLD=8`, a table of four, every other seat a
`HeuristicBot`).  The candidate is the 8 胡 `Outlook` and the control the
10 胡 one, both times 1.5, with the same gate and shallow graph:

| | Settings | Deals | Seed base |
| --- | --- | ---: | ---: |
| Web | `EXPECT=1.5 GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 40,000 | 1795400000 |
| Desktop | `EXPECT=1.5 GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1 BASELINE_GATE=14 BASELINE_SHALLOW=17` | 20,000 | 1795500000 |

- Each gets its own verdict.  Win: that setting uses the 8 胡 tables.  Loss:
  that setting goes back to the 10 胡 tables (`Outlook::new` also looks at
  the gate, or does not take 8).  Straddling 0: insufficient data, play is
  left as it is, and it is written down.
- Trial runs (seed base 1, not counted): web, 400 deals in 35 seconds, 16%
  part, −0.091 ± 0.162; desktop, 200 deals in 95 seconds, 24% part,
  −0.354 ± 0.215.  So the web's 40,000 deals take about 1 hour with a
  standard error near 0.016, and the desktop's 20,000 about 2.6 hours with
  about 0.022.

### Results

| | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | --- | ---: | --- | --- |
| Web | 40,000 | 4,988 | 24.80% → 24.87% | +0.022 ± 0.013 | [−0.004, +0.048] | insufficient data |
| Desktop | 20,000 | 3,619 | 25.39% → 25.43% | +0.005 ± 0.024 | [−0.041, +0.052] | insufficient data |

- **Both settings straddle 0; play is left as it is.**  `ExpectBot::new`
  goes on using the 10 胡 `Outlook` at 8 胡; the 8 胡 tables stay in
  `Outlook::new`, and the `EXPECT` candidate of `ab/` uses them.
- **The difference is small to begin with.**  Only 12% (web) and 18%
  (desktop) of the deals part: the 10 胡 tables and the 8 胡 tables choose
  the same move in most deals.  The web's upper bound is +0.048 and its
  lower −0.004: even if it is a gain, it is under 0.05 points a deal.
- **The trial runs' minus signs did not recur** (web −0.091, desktop −0.354,
  both over 400 deals or fewer).
- If the web's really is +0.022, telling it apart with 80% power takes about
  110,000 deals, close to 3 hours; that is not scheduled.  An `Eval` for
  8 胡 (step 3 in `variants.md`) is worth doing before this, and its labels
  can use `Outlook::new` directly.

## Tables of two and three

**This section was measured under the old rules.**  Since 2026-11-21 the
dead cards of a small table are 6 plus the 20 of each empty seat, and every
table draws 25 cards; the tables here were measured over rounds of 65 and 45
draws and have been removed from the code.  The measurements under the new
rules are in the last section, "Empty seats' hands are dead".

Measured 2026-11-20, base `4e461d0`.  The other half of step 2 of "The order
to fill them in" in `variants.md`: the same measurement with two or three
seats at the table, 10 胡.  `examples/outlook.rs` takes `PLAYERS`; the draws
in a round go by the number of players (65 at a table of two, 45 at a table
of three), and the tables grow to match.

Constants: `PLAYERS=2 outlook 10000 200` and `PLAYERS=3 outlook 10000 300`
(the calibration is the 10,000 rounds of the same run, seeds 201 and 301).
Calibration with one constant changed: `PLAYERS=2 outlook 0 202 10000` and
`PLAYERS=3 outlook 0 302 10000` (seeds 203 and 303, starting from that
table's tables).  The seeds and the round counts were all fixed before the
runs.

| Inside the gate | Four players | Three | Two |
| --- | ---: | ---: | ---: |
| h (per card) | 0.0711 | 0.0700 | 0.0561 |
| L | −12.26 | −12.20 | −12.16 |
| e | 0.2765 | 0.4271 | 0.7522 |
| A general is the seat's own draw | 0.2156 | 0.3013 | 0.4610 |
| Public cards a draw (not split by the gate) | 2.136 | 1.922 | 1.706 |
| Decisions inside the gate (per 10,000 rounds) | 22,853 | 19,931 | 15,811 |

- **e and the generals differ most.**  At a table of two the only opponent
  is also the previous seat, and every card it draws or discards comes to
  the seat to eat: e is 0.75, 2.7 times that of four players, and 0.46 of
  the generals are the seat's own draws.  The four-player tables undercount
  the cards the seat can eat by 60% at a table of two and by a third at a
  table of three.
- **Rounds at a small table never reach the later part of the pile.**  In
  10,000 rounds at a table of two, the segments with 500 cards or more
  inside the gate go only as far as 40 draws left; later segments take the h
  of the nearest segment (0.094), and the cards a segment likewise take the
  nearest segment's in the segments reached fewer than 100 times (34 draws
  left or fewer) (`Tally::outlook`).  At a table of three these are 18 or
  fewer and 15 or fewer.  At four players every segment has enough, and the
  tables have not changed.
- All the segments are `END_2` and `CARDS_2` (66 segments) and `END_3` and
  `CARDS_3` (46 segments) in `src/expect.rs`, and `Outlook::new` picks by
  `Rules::players`.  8 胡 at a small table was not measured; it uses that
  table's 10 胡 tables.

Calibration, two tables (seeds 201 and 301):

| | Decisions | Predicted win rate | Actual | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: | ---: |
| Two players | 15,811 | 0.544 | 0.513 | 1.05 | 0.32 ± 0.19 |
| Three players | 19,931 | 0.439 | 0.368 | 3.95 | 1.35 ± 0.25 |

The bias runs the same way as at four players but is much smaller: four
players have 0.382 against 0.291.  With one constant changed (seeds 203 and
303):

| | Two players: predicted | Actual | Predicted score | Actual score | Three players: predicted | Actual | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Measured | 0.546 | 0.513 | 1.11 | 0.34 ± 0.19 | 0.443 | 0.375 | 4.16 | 1.60 ± 0.25 |
| h × 1.5 | 0.452 | 0.513 | −1.18 | 0.34 | 0.354 | 0.375 | 0.87 | 1.60 |
| Cards a segment × 0.8 | 0.537 | 0.513 | 0.89 | 0.34 | 0.429 | 0.375 | 3.69 | 1.60 |
| e × 0.7 | 0.538 | 0.513 | 0.92 | 0.34 | 0.433 | 0.375 | 3.81 | 1.60 |

- **h × 1.5 overshoots at a small table.**  A table of two goes from 0.033
  too high to 0.061 too low, with the actual win rate above the prediction
  in eight bins of nine; a table of three is 0.021 too low, its five low
  bins predicted too low and its four high ones covered.
- Interpolating linearly between "measured" and "× 1.5", the multiplier at
  which the bias of the win rate vanishes is 1.18 at a table of two and 1.38
  at a table of three.  **1.2 and 1.4** are taken as each table's calibrated
  multiplier: read off seeds 203 and 303 only, with no other multiplier
  swept.

### A/B

Declared (2026-11-20, written down before the run; control `4e461d0`,
`RESHUFFLE=8`, 10 胡, every other seat a `HeuristicBot`).  The candidate is
that table's `Outlook` times the multiplier in `EXPECT`, the control is
`ExpectBot::new` (the four-player tables times 1.5), and the gate and the
shallow graph are the same on both sides:

| | Players | Settings | Multiplier | Deals | Seed base |
| --- | ---: | --- | ---: | ---: | ---: |
| A | 2 | Web | 1.2 | 100,000 | 1796400000 |
| B | 2 | Web | 1.5 | 100,000 | 1796500000 |
| C | 3 | Web | 1.4 | 50,000 | 1796600000 |
| D | 3 | Web | 1.5 | 50,000 | 1796700000 |
| E | 2 | Desktop | 1.2 | 10,000 | 1796800000 |
| F | 3 | Desktop | 1.4 | 10,000 | 1796900000 |

Web is `GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1 BASELINE_SHALLOW=0`, and
desktop is `GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1 BASELINE_GATE=14
BASELINE_SHALLOW=17`.

- Each of the six runs gets its own verdict.  Win: that table and that
  setting use that table's tables and that multiplier; when both multipliers
  win on the web at one table, the calibrated one (1.2, 1.4) is used,
  whichever scores higher.  Loss: first trace the deals that moved the most.
  Straddling 0: insufficient data, that setting's play is left as it is, and
  it is written down.
- The desktop runs only the calibrated multipliers: a deal costs 15 times
  the web's.
- Trial runs (seed base 1, not counted): web, two players, 400 deals in 6
  seconds, 31 deals part, −0.006 ± 0.054, and three players, 400 deals, 61
  deals part, −0.298 ± 0.153, both at 1.5; desktop, two players, 200 deals
  in 46 seconds, 47 deals part, +0.171 ± 0.120, and three players, 200 deals
  in 51 seconds, 40 deals part, +0.296 ± 0.246, also both at 1.5.  So each
  web run takes about 13 to 25 minutes, with a standard error near 0.003
  (two players) and 0.014 (three); each desktop run about 40 minutes, with
  about 0.017 and 0.035.
- The shallow graph is not used while more than 45 public cards are to come
  (`ExpectBot::depth`), and that count goes by the `Outlook`: a small
  table's tables have fewer public cards a draw, so the candidate starts on
  the shallow graph earlier than the control.  This too counts as part of
  the candidate.

### Results

| | Players | Settings | Multiplier | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | --- | ---: | ---: | ---: | --- | ---: | --- | --- |
| A | 2 | Web | 1.2 | 100,000 | 6,657 | 50.35% → 50.28% | −0.019 ± 0.004 | [−0.027, −0.011] | loss |
| B | 2 | Web | 1.5 | 100,000 | 6,780 | 50.50% → 50.48% | −0.011 ± 0.004 | [−0.019, −0.003] | loss |
| C | 3 | Web | 1.4 | 50,000 | 8,542 | 33.99% → 34.01% | +0.001 ± 0.012 | [−0.024, +0.025] | insufficient data |
| D | 3 | Web | 1.5 | 50,000 | 8,583 | 33.50% → 33.45% | −0.023 ± 0.013 | [−0.048, +0.002] | insufficient data |
| E | 2 | Desktop | 1.2 | 10,000 | 2,188 | 50.79% → 51.11% | +0.066 ± 0.021 | [+0.025, +0.107] | win |
| F | 3 | Desktop | 1.4 | 10,000 | 2,232 | 34.06% → 34.30% | +0.051 ± 0.031 | [−0.011, +0.113] | insufficient data |

- **The desktop at a table of two is a win; the web at a table of two is a
  loss at both multipliers; all three runs at a table of three straddle 0.**
- A's loss has been traced (`research/outlook-trace.patch`: `TRACE=1` prints
  the pile and the candidate's count of concealed cards at every point where
  the two part, and `ab/tally.py` sorts them; replaying A's 100,000 deals
  and E's 10,000 gives the same totals as the table above):

  | Where the two part | A: part | Per deal that parts | Per deal | E: part | Per deal that parts | Per deal |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | Inside the gate | 6,199 | +0.118 ± 0.058 | +0.007 ± 0.004 | 2,186 | +0.304 ± 0.095 | +0.066 ± 0.021 |
  | Past the gate, only the candidate on the shallow graph | 456 | −5.83 ± 0.33 | −0.027 ± 0.002 | 2 | | |
  | Past the gate, both on it | 2 | | | 0 | | |

- **The loss is not in the tables but in the shallow graph starting early.**
  It is what the last point of the declaration spoke of: the tables of two
  have only 1.1 to 1.4 public cards a draw, so with 41 draws left the public
  cards to come are already within 45, and the candidate uses the shallow
  graph from there on; the control, on the four-player tables, has to wait
  until 25, and nearly every round at a table of two ends before that.  Over
  those 16 draws the leaves read the four-player `Eval`, and each deal that
  parts loses 5.8 points (the same thing the two cheap fixes lose to in
  `eval.md`, "Two and three players").
- Inside the gate, where only the `Outlook` differs, the deals that part
  come out positive, but that is a split made after the fact, not a verdict.

### Second pass: the shallow graph goes by draws

`ExpectBot::depth` checks one more condition: with more draws left than
`eval::DRAWS` (25, the draws in a round of four) it leaves the shallow graph
alone, however many public cards its own `Outlook` counts them as.  With
`Outlook::default` the two conditions are the same thing (25 draws left are
exactly 45 public cards, and 26 are 48), so `ExpectBot::new` plays the same
at every table; only a candidate with other tables changes.  The four-player
reading of zero deals that part is in the results below.

Declared (2026-11-20, written down before the run; control `4e461d0`,
`RESHUFFLE=8`, 10 胡, settings as in the first pass, the candidate the
working tree: that table's tables times the calibrated multiplier, plus the
condition above).  New seeds; the deals of the first pass do not count:

| | Players | Settings | Multiplier | Deals | Seed base |
| --- | ---: | --- | ---: | ---: | ---: |
| G | 2 | Web | 1.2 | 200,000 | 1797000000 |
| H | 3 | Web | 1.4 | 100,000 | 1797200000 |
| I | 2 | Desktop | 1.2 | 10,000 | 1797300000 |
| J | 3 | Desktop | 1.4 | 20,000 | 1797400000 |

- Each of the four runs gets its own verdict, by the rules of the first
  pass.  Win: that table and that setting use that table's tables and
  multiplier.  Loss: trace first.  Straddling 0: that setting's play is left
  as it is.
- 1.5 is not run again: the first pass declared that when both win, the
  calibrated one is used.
- Deal counts: A is +0.007 a deal inside the gate, which takes a standard
  error under 0.003 to tell apart, so G is 200,000 deals; at a table of
  three more deals part and the variance is larger, so H and J are twice the
  first pass's.  About 3.3 hours.

### Results of the second pass

2026-11-21.

| | Players | Settings | Multiplier | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | --- | ---: | ---: | ---: | --- | ---: | --- | --- |
| G | 2 | Web | 1.2 | 200,000 | 12,310 | 50.34% → 50.38% | +0.006 ± 0.002 | [+0.002, +0.011] | win |
| H | 3 | Web | 1.4 | 100,000 | 5,085 | 33.37% → 33.44% | +0.019 ± 0.005 | [+0.009, +0.029] | win |
| I | 2 | Desktop | 1.2 | 10,000 | 2,188 | 50.58% → 50.77% | +0.039 ± 0.020 | [−0.000, +0.078] | insufficient data |
| J | 3 | Desktop | 1.4 | 20,000 | 3,785 | 33.79% → 33.92% | +0.016 ± 0.020 | [−0.024, +0.055] | insufficient data |

- **The web's settings win at both tables.**  `ExpectBot::new` (a 10-card
  gate) uses each table's own tables at 10 胡 tables of two and three, with h
  times 1.2 and 1.4; the web page's hint is this bot.  The scores are tiny:
  +0.006 a deal at a table of two and +0.019 at a table of three, with only
  6% and 5% of the deals parting.
- **The desktop's settings straddle 0 at both tables; play is left as it
  is.**  Under the desktop's settings the `bot` of `web/` pins the tables
  back to the four-player ones (`Outlook::default` times `END_SCALE`).  I's
  lower bound is −0.0004.  E of the first pass was a win, but that candidate
  started on the shallow graph earlier and is not the same as the present
  code, so it cannot be pooled in; the two runs point the same way, and only
  another run that is long enough will give a verdict.
- **The deals that part at a table of three fell from 17% to 5%** (C's
  8,542/50,000 to H's 5,085/100,000).  The candidates of C and H differ only
  in when the shallow graph starts, so it accounts for two thirds of the
  first pass's parting at a table of three; a table of two only went from
  6.7% to 6.2%.
- 8 胡 at a small table was not measured; `ExpectBot::new` still uses the
  four-player 10 胡 tables there.

### Zero deals that part

The code after `ExpectBot::new` changed tables, with the candidate on the
four-player tables (`FOUR` of `ab/`, `EXPECT=1.5`) and the control the
`ExpectBot::new` of `4e461d0`: neither the extra condition in `depth` nor
the desktop's pinning back to the four-player tables should change a single
move.

| Players | Web's settings, 20,000 deals | Seed base | Desktop's settings, 2,000 deals | Seed base |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 0 deals part | 1797700000 | 0 deals part | 1797800000 |
| 3 | 0 deals part | 1797900000 | 0 deals part | 1798000000 |
| 2 | 0 deals part | 1798100000 | 0 deals part | 1798200000 |

Before the second pass started there were two more four-player runs (the
code with `depth` changed and `ExpectBot::new` not yet on other tables):
web, 20,000 deals, seed base 1797500000; desktop, 2,000 deals, seed base
1797600000; both 0 deals that part.

### Against `HeuristicBot`

`variants.md` says there is no number that counts for `ExpectBot` against
`HeuristicBot` at a small table.  Declared (2026-11-21, written down before
the run; control the `HeuristicBot` of `4e461d0`, `RESHUFFLE=8`, 10 胡, the
candidate the `ExpectBot` of the web's settings, that is, the candidate of G
and H).  Measurement only, no change to play:

| | Players | Settings | Deals | Seed base |
| --- | ---: | --- | ---: | ---: |
| K | 2 | `EXPECT=1.2 GATE_CARDS=10 SHALLOW=0` | 50,000 | 1798300000 |
| L | 3 | `EXPECT=1.4 GATE_CARDS=10 SHALLOW=0` | 50,000 | 1798400000 |

| | Players | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | ---: | --- | --- |
| K | 2 | 50,000 | 5,029 | 50.13% → 50.63% | +0.106 ± 0.007 | [+0.091, +0.120] | win |
| L | 3 | 50,000 | 4,456 | 33.30% → 33.68% | +0.118 ± 0.010 | [+0.098, +0.137] | win |

Under the web's settings `ExpectBot` scores about 0.11 points a deal more
than `HeuristicBot` at a small table, and its win rate is 0.4 to 0.5
percentage points higher.  Only a tenth of the deals part: rounds at a small
table are short, and few decisions get inside the gate.

## Empty seats' hands are dead

2026-11-21, base `107032c`.  The dead cards of tables of two and three went
from 6 to 6 plus the 20 of each empty seat (`Rules::dead`), and every table
draws 25 cards; the reasons and the measured rate of exhausted rounds are in
`doc/variants.md`, "The dead cards of a small table".

The two tables of `Outlook` are looked up by draws left.  Under the old
rules a table of two counted from 65 and ended after 12 draws on average, so
the tables' segments of 25 and below were borrowed from the few rounds that
got that far; under the new rules a round starts at 25.  The tables,
multipliers and A/Bs of the previous section therefore no longer apply; the
tables are removed from `src/expect.rs`, and `Outlook::new` and
`ExpectBot::new` use the four-player tables (h × 1.5) at a small table for
now, until they are measured again below.

### Zero deals that part at four players

`Rules::dead()` is 6 at four players, where the rules have not changed, so
no move should change.  The candidate is the working tree, the control
`107032c`, `RESHUFFLE=8`, the deal counts and seeds fixed before the runs:

| Candidate and control | Deals | Seed base | Part |
| --- | ---: | ---: | ---: |
| `HeuristicBot` | 20,000 | 1798500000 | 0 deals |
| `ExpectBot`, web's settings | 20,000 | 1798600000 | 0 deals |
| `ExpectBot`, desktop's settings | 2,000 | 1798700000 | 0 deals |

A small table has no such run to make: the old and the new rules are
different rounds, and the two sides' `Round`s cannot step together.

### Constants

Measured as in the previous section, under the new rules: `PLAYERS=2 outlook
10000 210` and `PLAYERS=3 outlook 10000 310` (the calibration is the 10,000
rounds of the same run, seeds 211 and 311).  Calibration with one constant
changed: `PLAYERS=2 outlook 0 212 10000` and `PLAYERS=3 outlook 0 312 10000`
(seeds 213 and 313, starting from the new tables).  The seeds and the round
counts were all fixed before the runs and repeat none of the old rules'.

| Inside the gate | Four players | Three | Two | Old rules: three | Two |
| --- | ---: | ---: | ---: | ---: | ---: |
| h (per card) | 0.0711 | 0.0699 | 0.0544 | 0.0700 | 0.0561 |
| L | −12.26 | −12.18 | −12.11 | −12.20 | −12.16 |
| e | 0.2765 | 0.4164 | 0.7442 | 0.4271 | 0.7522 |
| A general is the seat's own draw | 0.2156 | 0.3094 | 0.4688 | 0.3013 | 0.4610 |
| Public cards a draw (not split by the gate) | 2.136 | 1.936 | 1.711 | 1.922 | 1.706 |
| Decisions inside the gate (per 10,000 rounds) | 22,853 | 19,497 | 15,567 | 19,931 | 15,811 |
| Exhausted rounds (per 10,000 rounds) | | 327 | 412 | | |

- **The four constants have hardly changed.**  How many cards a round draws
  has nothing to do with the length of the pile, so the table seen inside
  the gate is the same one; what changes is how the tables line up with
  draws left.
- **Every segment can now be measured.**  The old tables borrowed the
  segments of 40 draws left or fewer at a table of two and of 18 or fewer at
  a table of three; the new ones borrow only at the two ends: the segments
  with fewer than 500 cards inside the gate are 25 and 24 draws left (at
  both tables) and 0 at a table of three, as at four players.
- The tables are `END_2`, `CARDS_2`, `END_3` and `CARDS_3` in
  `src/expect.rs`, 26 segments each, and `Outlook::new` picks by
  `Rules::players`.  8 胡 at a small table was not measured and uses that
  table's 10 胡 tables.

Calibration, two tables (seeds 211 and 311):

| | Decisions | Predicted win rate | Actual | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: | ---: |
| Two players | 15,567 | 0.510 | 0.465 | 1.39 | 0.21 ± 0.18 |
| Three players | 19,497 | 0.418 | 0.351 | 4.15 | 1.53 ± 0.25 |

With one constant changed (seeds 213 and 313):

| | Two players: predicted | Actual | Predicted score | Actual score | Three players: predicted | Actual | Predicted score | Actual score |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Measured | 0.510 | 0.470 | 1.42 | 0.33 ± 0.19 | 0.418 | 0.356 | 4.08 | 1.71 ± 0.24 |
| h × 1.5 | 0.432 | 0.470 | −0.87 | 0.33 | 0.339 | 0.356 | 0.79 | 1.71 |
| Cards a segment × 0.8 | 0.481 | 0.470 | 1.28 | 0.33 | 0.390 | 0.356 | 3.55 | 1.71 |
| e × 0.7 | 0.501 | 0.470 | 1.24 | 0.33 | 0.408 | 0.356 | 3.75 | 1.71 |

- The bias runs the same way as under the old rules and as at four players,
  and h × 1.5 overshoots just the same.
- Interpolating linearly between "measured" and "× 1.5", the multiplier at
  which the bias of the win rate vanishes is 1.26 at a table of two and 1.39
  at a table of three (1.18 and 1.38 under the old rules).  **1.3 and 1.4**
  are taken: read off seeds 213 and 313 only, with no other multiplier
  swept.

### A/B

Declared (2026-11-21, written down before the run; control `07d5602`, the
commit that changed the rules, in which a small table uses the four-player
tables times 1.5; `RESHUFFLE=8`, 10 胡, every other seat a `HeuristicBot`).
The candidate is the working tree: `07d5602` plus the new tables of
`Outlook::new`, its play changing only with the switches below.  Web is a
10-card gate and the shallow graph one deep (`GATE_CARDS=10 SHALLOW=0`);
desktop is a 14-card gate and two deep at 17 cards or fewer (`GATE_CARDS=14
SHALLOW=17`).

Three questions:

- **The shallow graph (M, N, S, T).**  Since the rules changed, every round
  at a small table starts with 25 draws left, and past the gate the shallow
  graph is used from the first move, its leaves the four-player `Eval`;
  under the old rules it was hardly ever reached, so this play has never
  been measured.  The candidate is the present play (`FOUR=1 EXPECT=1.5`);
  the control is an `ExpectBot` with the same gate that plays as
  `HeuristicBot` past it (`BASELINE_EXPECT=1`, plus `BASELINE_GATE=14` on
  the desktop).
- **The table's own tables (O, P, U, V).**  A rerun of G to J of the
  previous section: the candidate is that table's tables times the
  calibrated multiplier (`EXPECT=1.3` and `1.4`), the control the
  four-player tables times 1.5, with the same gate and shallow graph on both
  sides (`BASELINE_SHALLOW=0` or `17`).
- **Against `HeuristicBot` (Q, R).**  A rerun of K and L of the previous
  section, measurement only: the candidate is the web's settings at
  `07d5602` (`FOUR=1 EXPECT=1.5 GATE_CARDS=10 SHALLOW=0`), the control
  `HeuristicBot`.

| | Question | Players | Settings | Deals | Seed base |
| --- | --- | ---: | --- | ---: | ---: |
| M | Shallow graph | 2 | Web | 10,000 | 1798800000 |
| N | Shallow graph | 3 | Web | 15,000 | 1798900000 |
| O | Own tables × 1.3 | 2 | Web | 10,000 | 1799000000 |
| P | Own tables × 1.4 | 3 | Web | 40,000 | 1799100000 |
| Q | Against `HeuristicBot` | 2 | Web | 10,000 | 1799200000 |
| R | Against `HeuristicBot` | 3 | Web | 15,000 | 1799300000 |
| S | Shallow graph | 2 | Desktop | 4,000 | 1799400000 |
| T | Shallow graph | 3 | Desktop | 4,000 | 1799500000 |
| U | Own tables × 1.3 | 2 | Desktop | 4,000 | 1799600000 |
| V | Own tables × 1.4 | 3 | Desktop | 6,000 | 1799700000 |

- Each of the ten runs gets its own verdict.  **The shallow graph:** win,
  the play stays.  Loss, first trace the deals that moved the most, then
  have that setting fall back to `HeuristicBot` past the gate at a small
  table.  Straddling 0, insufficient data, play is left as it is (it stays,
  the same handling as 8 胡), and it is written down.  **The table's own
  tables:** win, that table and that setting use that table's tables and
  multiplier.  Loss, trace first.  Straddling 0, insufficient data, the
  four-player tables stay in use.  **Q and R** change no play.
- Trial runs (seed base 1, not counted; 400 deals each on the web, 200 each
  on the desktop):

  | | Seconds | Part | Candidate minus control |
  | --- | ---: | ---: | ---: |
  | M | 73 | 358 | +0.223 ± 0.214 |
  | N | 62 | 333 | +0.138 ± 0.301 |
  | O | 67 | 230 | −0.641 ± 0.170 |
  | P | 35 | 108 | +0.009 ± 0.166 |
  | Q | | 359 | +0.356 ± 0.215 |
  | R | | 335 | +0.217 ± 0.304 |
  | S | 248 | 168 | +0.058 ± 0.308 |
  | T | 191 | 146 | +0.079 ± 0.420 |
  | U | 186 | 112 | −0.076 ± 0.190 |
  | V | 111 | 56 | +0.389 ± 0.258 |

- **The deals that part went from 5% to 6% under the old rules to 27% to
  90%,** because both sides use the shallow graph past the gate, and the
  graph reads the `Outlook`.  The standard error is much larger for it:
  sizes like G's +0.006 and H's +0.019 under the old rules would now take
  hundreds of thousands of deals to tell apart.
- The deal counts are set by the standard error.  The shallow graph is
  +0.19 ± 0.05 and +0.20 ± 0.05 at four players (`eval.md`, batch 4), so M,
  N, Q and R take a standard error of 0.05 or less: 0.043, 0.049, 0.043,
  0.050.  O and P take 0.034 and 0.017; P can only tell a difference of
  0.035 or more.  A deal on the desktop takes 0.6 to 1.2 seconds, so time
  sets the deal counts: standard errors near 0.069, 0.094, 0.043 and 0.047,
  and S and T can only tell differences of 0.14 and 0.19 or more.
- Time: about 3.8 hours for the six web runs and about 4.4 hours for the
  four desktop runs.  The web's run first.

### Results: M and N, the rest withdrawn

2026-11-21.

| | Players | Settings | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | --- | ---: | ---: | --- | ---: | --- | --- |
| M | 2 | Web | 10,000 | 8,962 | 48.20% → 47.81% | −0.184 ± 0.045 | [−0.271, −0.096] | loss |
| N | 3 | Web | 15,000 | 12,655 | 32.96% → 33.84% | +0.216 ± 0.051 | [+0.116, +0.315] | win |

- **The four-player `Eval` still works at a table of three and is negative
  at a table of two.**  The +0.216 at a table of three is as large as the
  +0.19 at four players (`eval.md`, batch 4); using it past the gate at a
  table of two scores 0.18 points a deal less than playing as
  `HeuristicBot`.
- **The cause is how many cards each player gets, not the pile.**  Cutting
  the pile lines up the draws in a round (about 12 at all three tables), but
  the empty seats' draws are not made up for: the same 25 cards are shared
  by two players, each drawing twice what a seat of four does, and the cards
  that come to the seat to eat go from 0.28 to 0.74.  `Eval` cannot see the
  `Outlook`, and the leaves' values were labelled for a table of four
  (`eval.md`, "The eval: pattern tables of the blocks": with other `Rules`
  or another `Outlook` the tables must be trained again).
- **O to V are withdrawn, before any of them had data that counts.**  The
  candidates of O, P, U and V pair that table's `Outlook` with the
  four-player `Eval`, so the graph and the leaves each speak of a different
  table, and O's trial run (−0.641 ± 0.170) is probably just that; measuring
  it could not tell bad tables from a mismatch.  Q and R measure the play of
  `07d5602`, and M says that play should not stay at a table of two.
  Whether to run S and T is decided once that table has an `Eval` of its
  own.  These seed bases (1799000000 to 1799700000) are not used again.
- **Next: train an `Eval` each for tables of two and three,** labelled with
  that table's `Outlook` times the multiplier calibrated above, then run the
  A/B.  The declaration and the results are in `doc/eval.md`, "Tables for
  two and three players".  The deals that M's loss owes a trace are left for
  there too: if the new tables beat `HeuristicBot` past the gate at a table
  of two, the tables were the cause; if they do not, then trace.


### Results: each table's own `Eval`

2026-11-22, the control `07d5602` throughout.  The details, the loss tables
and the declaration are in `doc/eval.md`, "Tables for two and three
players".

| | Table | Settings | Deals | Candidate minus control | 95% CI | Verdict |
| --- | --- | --- | ---: | ---: | --- | --- |
| W | Two players, 10 胡 | Web | 10,000 | +0.375 ± 0.042 | [+0.293, +0.457] | win |
| Y | Two players, 10 胡 | Desktop | 4,000 | +0.320 ± 0.056 | [+0.210, +0.431] | win |
| X | Three players, 10 胡 | Web | 40,000 | −0.021 ± 0.025 | [−0.070, +0.028] | insufficient data |
| Z | Three players, 10 胡 | Desktop | 6,000 | −0.095 ± 0.061 | [−0.215, +0.025] | insufficient data |
| W8 | Two players, 8 胡 | Web | 10,000 | +0.302 ± 0.040 | [+0.224, +0.380] | win |
| Y8 | Two players, 8 胡 | Desktop | 4,000 | +0.048 ± 0.056 | [−0.061, +0.158] | insufficient data |

- **Two players at 10 胡 use their own `Outlook` (times 1.3) and `Eval`**
  (`ExpectBot::new`, `Eval::of`).  Every other table stays on the tables of
  four players at 10 胡.
- **M's loss matches the tables' bias.**  The four-player tables
  underestimate each leaf by 3.06 points at a table of two and by 0.96 at a
  table of three; the two-player tables' own bias is −0.02.  Against the
  same control, the four-player `Outlook` and `Eval` are −0.184 and the
  two-player `Outlook` and `Eval` +0.375 (the two changed together and
  cannot be told apart).  No deals of M were traced separately.
- **The multiplier 1.4 at a table of three and two players at 8 胡 are not
  adopted,** one for insufficient data, the other because only the web's
  settings won and the declaration asked for both runs.

## Against the bot that ships

Declared in `doc/eval.md`, "The table of two against the bot that ships":
the `Outlook` of two players at 10 胡 measured with the bot a player meets
in every seat (`BOT=web`, `BOT=desktop`), on the seeds and round counts of
"Empty seats' hands are dead", base `39e3870`.  h or e a tenth or more off
the table in the code opens batch 2 for that client.

| Inside the gate | Code (`HeuristicBot`) | Web's bot | Desktop's bot |
| --- | ---: | ---: | ---: |
| h (per card) | 0.0544 | 0.0568 | owed |
| L | −12.11 | −12.01 | owed |
| e | 0.7442 | 0.7372 | owed |
| A general is the seat's own draw | 0.4688 | 0.4618 | owed |
| Public cards a draw (not split by the gate) | 1.711 | 1.789 | owed |
| Decisions inside the gate (per 10,000 rounds) | 15,567 | 18,516 | owed |

- **The web: closed.**  h is 4% and e 1% off the code's, under the tenth
  the declaration asks for, so batch 2 is not designed for the web's bot.
- **The shape by draws left moved more than the total.**  h by draws left
  is 0.096 to 0.087 at 0 to 3 draws left against the code's 0.069 to
  0.077, and 0.021 at 23 to 25 against 0.033; the cards a segment run to
  3.13 at 25 draws left against 2.78.  The declaration reads only the
  totals.
- **The tables in the code overpredict more against this opponent.**
  Against the web's bot on seed 213, starting from the tables in the code:

  | | Predicted win rate | Actual | Predicted score | Actual score |
  | --- | ---: | ---: | ---: | ---: |
  | Measured | 0.545 | 0.476 | 1.80 | 0.06 ± 0.17 |
  | h × 1.5 | 0.460 | 0.476 | −0.54 | 0.06 |
  | Cards a segment × 0.8 | 0.516 | 0.476 | 1.58 | 0.06 |
  | e × 0.7 | 0.533 | 0.476 | 1.55 | 0.06 |

  Interpolated as in "Constants", the bias of the win rate vanishes at h ×
  1.40, against 1.26 under `HeuristicBot` self-play; the code takes 1.3.
  The new tables on seed 211 predict 0.457 against 0.478 (−0.27 points
  against 0.03 ± 0.17).  No multiplier is changed: that is batch 2's, and
  the web's is closed.
