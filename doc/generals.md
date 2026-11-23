# How often cards appear: the design

Written 2026-11-23, base `803d18e`.  It started from the book's 「對子的上限」
(The limit on pairs): `ExpectBot` sometimes eats one pair more on purpose and
counts on later generals (將帥) to make it up.  That chapter has since been
removed (`../tsap-oo-book` `86caae2`): the limit is only "three sevens are
twenty-one" written another way, and a person who lays the hand out as seven
groups of three will not eat too many pairs.  What is left here is the
computer's problem.  Measuring when it does this (`examples/overspend.rs`)
showed that it values hands that can only win on a general too high.  This
document splits the work that follows into batches, each with a gate to pass
before the next.  No code is written yet.

**The main line is the appearance rates, not a formula for pairs.**  The
extra pair is a symptom: `Graph` assumes that from the middle of the round on
every public card is drawn evenly from the unseen cards.  That assumption is
wrong, most of all for generals (batch 1: only four tenths of what it
assumes), and off for the other ranks too (車馬炮 three tenths more, soldiers
three tenths fewer early on).  It is this assumption that needs fixing.  A
formula for how many pairs a hand can have may not exist, and may be too
complicated if it does; it is not sought here.  Whether an overspend should
be blocked is a check for after the fix.

## Questions to answer

1. How far is the frequency with which a public card of the middle and late
   round is of each rank from the "unseen copies ÷ unseen cards" `Graph`
   assumes?  How does it change with the draws left?  (Batch 1 measured it.)
2. With the measured rates in `Outlook`, does `ExpectBot` get stronger?
   Generals first (batch 2), then every rank (batch 2b).
3. Once that is fixed, how many pair eats that overspend before the hand is
   waiting remain?  This checks whether the appearance rates are fixed well
   enough; it is not a search for a rule that blocks pairs.
4. Which sentences of the book on pairs (`material.md`, 「吃對子是在湊張數」
   (Eating a pair is making up the card count)) need changing?

## Progress

Each batch is about one session.  Update this table when a batch is done.

| Batch | Output | Changes play | Gate | Status |
| --- | --- | --- | --- | --- |
| 0 | `examples/overspend.rs`, `research/overspend.py`, this document | No | None | Done (`803d18e`) |
| 1 | The generals' discount ρ, by draws left | No; must measure zero deals that part | ρ within the gate below 0.85 | Done, ρ = 0.406, passed |
| 2 | `Outlook`'s discount into `Graph`, calibration, A/B | Yes | The A/B runs only once the calibration passes; commit only on a win | Calibration failed: the hands that extend 0 are fixed, the 搭子 got worse; no A/B run |
| 2b | Every rank's appearance rate into `Graph`, calibration, A/B | Yes | Batch 2's calibration passed; the A/B runs only once its own calibration passes; commit only on a win | Done, win on both the web and the desktop |
| 3 | Relearn the `Eval` table under the new `Outlook`, A/B | Yes | Batch 2 or 2b has won and 2b has its verdict; commit only on a win | Dropped: the CIs of the web and the desktop both straddle 0; the table is not replaced |
| 4 | Overspending before the hand is waiting: rerun the statistics, check whether the appearance rates are fixed well enough | Candidate | See batch 4 | Statistics done: 18.8% of B1 eaten, the 10% gate not passed, the waiting table within 3.3 percentage points; the probe is skipped and the line ends.  The A/B of `waits` is insufficient data and reverted |
| 5 | Revision of the book | No | Batches 2, 2b and 4 have their verdicts | Not started |

## Terms

- **Fully melded**: a hand with no loose cards (雜牌) and no more than 1 胡
  short: its 20 cards are all in melds, pairs and lone generals.  A general
  is a meld by itself, so such a hand waits on every general not yet seen.
- **Extension**: the other cards a fully melded hand waits on, each adding one
  card to a meld in hand: a pair ponged into a triplet, a concealed triplet
  konged, soldiers of three colours eaten into four.  A meld exposed on the
  table takes no card.  Whenever a general wins, an extension card wins as
  well, since an extension too brings at least 1 胡 more.  A hand that extends
  0 is one kind of fully melded hand, not a class apart.
- **Overspend**: after a pair is eaten, `ceiling(generals = 0)` falls short
  of the threshold while `ceiling(generals = 1)` reaches it.  Extensions
  count within `generals = 0`, so only a general makes an overspent hand
  good.  Split further by the ceiling before the eat:
  - **B1**: before the eat `generals = 0` reaches the threshold; only the
    eat leaves the hand one general short.
  - **B2**: one short already before the eat.
  - **A** is where `generals = 1` falls short as well, and **C** where
    `generals = 0` still reaches, the same groups as in `doc/hu-ceiling.md`.

## What batch 0 measured

Seat 1 is the desktop's `ExpectBot` (gate 14, depth 17) against three
`HeuristicBot`s.  20,000 rounds, seed base 1797200000, base `b95dde4`:

```sh
cargo run --release --example overspend -- 20000 1797200000 > pairs.csv 2> waiting.csv
python3 research/overspend.py pairs.csv waiting.csv
```

### How good a wait a fully melded hand is

Hands waiting after their discard, 14 cards or fewer (the range `Graph`
expands whole).  Each cell is the actual win rate and its standard error,
with `Graph`'s estimate of the win rate and the number of hands in
parentheses:

| | 15 draws or more | 8–14 draws | 0–7 draws |
| --- | ---: | ---: | ---: |
| Fully melded, extends 0 | 50% ± 4 (65%, 168) | 39% ± 3 (58%, 236) | 23% ± 4 (43%, 129) |
| Fully melded, extends 1 | 67% ± 4 (67%, 159) | 47% ± 4 (61%, 189) | 45% ± 8 (49%, 40) |
| Fully melded, extends 2 | 62% ± 3 (70%, 363) | 61% ± 3 (63%, 329) | 32% ± 5 (48%, 101) |
| Fully melded, extends 3 | 74% ± 3 (72%, 293) | 61% ± 4 (64%, 146) | 48% ± 7 (53%, 50) |
| Fully melded, extends 4 or more | 80% ± 1 (77%, 1,079) | 70% ± 2 (68%, 409) | 44% ± 7 (58%, 50) |
| 搭子, waits on 1 | 38% ± 3 (30%, 285) | 26% ± 3 (22%, 174) | 17% ± 5 (13%, 65) |
| 搭子, waits on 2 | 44% ± 3 (39%, 339) | 40% ± 3 (30%, 212) | 24% ± 5 (16%, 62) |
| 搭子, waits on 3 | 55% ± 1 (46%, 1,346) | 36% ± 2 (35%, 558) | 17% ± 3 (24%, 123) |
| 搭子, waits on 4 | 54% ± 1 (46%, 1,334) | 41% ± 3 (37%, 367) | 27% ± 5 (27%, 70) |
| 搭子, waits on 5 or more | 60% ± 1 (55%, 1,300) | 47% ± 3 (44%, 382) | 33% ± 6 (32%, 64) |
| 士象 waiting for its own general | 33% ± 2 (40%, 523) | 24% ± 3 (30%, 217) | 11% ± 4 (20%, 56) |

- **A fully melded hand that extends 0 is overestimated by 15 to 20 percentage
  points.**  In fact it does about as well as a 搭子 that waits on 2 to 4
  cards; `Graph` takes it for 20 percentage points better than a 搭子 that
  waits on 4.
- **A 搭子 is underestimated by 5 to 9 percentage points with 15 draws or
  more left.**
- **士象 waiting for its own general is overestimated too**, by 6 to 9
  percentage points.
- **The more extensions, the closer the estimate.**  With 3 or more and 8
  draws or more left it is within 4 percentage points; with 1 or 2 it still
  runs up to 14 percentage points high.  Over all fully melded hands the
  estimate is 67% against an actual 64%.

### When the engine overspends

The 51,317 decisions where a pair could be eaten:

| | Decisions | Eaten | Own estimate, eat − pass (points) |
| --- | ---: | ---: | ---: |
| C: within the limit | 39,686 | 19,830 | +0.11 |
| B1: one short only after eating | 4,176 | 1,039 | −2.23 |
| B2: one short already before eating | 1,629 | 616 | −0.66 |
| A: short even with one more drawn | 5,826 | 243 | −2.31 |

Of the 1,135 decisions that are fully melded after eating, 984 have extensions
to wait on, all of them in C: the extension is the pair's way out, with no
need of a general.  Those that extend 0 and overspend are 39 in B1 (35 eaten)
and 42 in B2 (32 eaten), both by its own estimate 10.7 points better than
passing.  By what passing waits on (the 70 of C included):

| If it passes | Decisions | Eaten | Own estimate, eat − pass (points) |
| --- | ---: | ---: | ---: |
| Not waiting | 46 | 43 | +18.4 |
| 士象 waiting for its own general | 23 | 23 | +11.4 |
| 搭子 waits on 1 to 2 | 10 | 10 | +15.2 |
| 搭子 waits on 3 or more | 47 | 45 | +10.4 |
| Already fully melded | 25 | 0 | −1.8 |

- **Moving to fully melded from not waiting, from 士象, or from a 搭子 that waits
  on 1 to 2 cards holds up**: in the table before, the actual win rate of
  fully melded hands is higher or level as well.
- **Moving from a 搭子 that waits on 3 cards or more, the 10 points of its
  own estimate come from the overestimate**: the actual win rates are level
  to slightly worse.
- **B1 is almost all not yet waiting** (only 39 of the 4,176 are fully melded
  after eating).  The 1,039 eaten are mostly marginal decisions, only 0 to 2
  points better by its own estimate.  If generals are overestimated these
  most likely lose, but these statistics cannot tell.

Caveat: this is observation.  The hands that reach each kind of wait differ
to begin with (their 胡, the other seats' progress), and "level" is only
good to 5 percentage points or so.

## Why the overestimate (a conjecture, for batch 1 to test)

`Graph` takes every public card as drawn evenly from the unseen cards, so the
chance that a public card is a general is K ÷ N (K the unseen generals, N
the unseen cards).  In fact:

- **Generals only come out of the pile.**  The public cards include the
  other seats' discards, and there is no general among those.
- **A general in another seat's hand never comes out.**  What the other
  seats discard is never a general, so the later it is, the larger the share
  of the unseen generals that sits in their hands.

Worked back from the win rates, generals appear about half to six tenths as
often as assumed.  This number has not been measured directly.

## Batch 1: measure the discount

`examples/outlook.rs` gains one item.  For every public card, once for each
seat:

- Expected: that seat's K ÷ N at the time.  The cards that seat has not seen
  must be rebuilt from `Round::events`, not just the count of concealed
  cards.
- Actual: whether the card is a general.

**ρ = the sum of the actual ÷ the sum of the expected**, by draws left, once
over everything and once within the gate, in the same format as the `end`
and `cards` tables.  A segment with fewer than 500 cards takes the value of
the nearest segment, as h does.

- The constants and the calibration's seed and round count are fixed before
  the run; four players at 10 胡 is measured first, and the tables of two
  and three players and of 8 胡 by the same command with `PLAYERS` and
  `THRESHOLD`.
- `src/` is not touched; `ab/run.sh 20000` must read 0 deals that part.

The gate:

- **Overall ρ within the gate is below 0.85**: do batch 2.
- **Otherwise**: the overestimate has another cause.  First see where the
  fully melded hands that extend 0 lose (the other seats winning first more
  often than h says?  the generals gathered in the other seats' hands?); this
  line stops here and is written up in `research/`.

### What batch 1 measured

`HeuristicBot` self-play, 20,000 rounds a table, seed base 1795364857, base
`74e99e2`, fixed before the run:

```sh
cargo run --release --example outlook -- 20000 1795364857 0
PLAYERS=3 cargo run --release --example outlook -- 20000 1795364857 0
PLAYERS=2 cargo run --release --example outlook -- 20000 1795364857 0
THRESHOLD=8 cargo run --release --example outlook -- 20000 1795364857 0
```

ρ, ± one standard error:

| Table | All | Within the gate | Within the gate, without the seat's own discards |
| --- | ---: | ---: | ---: |
| Four players, 10 胡 | 0.383 ± 0.001 | 0.406 ± 0.002 | 0.480 ± 0.003 |
| Three players, 10 胡 | 0.458 ± 0.001 | 0.515 ± 0.003 | 0.619 ± 0.004 |
| Two players, 10 胡 | 0.549 ± 0.002 | 0.647 ± 0.005 | 0.792 ± 0.006 |
| Four players, 8 胡 | 0.358 ± 0.001 | 0.383 ± 0.003 | 0.463 ± 0.003 |

- **The gate is passed: all four tables are below 0.85.**  Lower still than
  the half to six tenths worked back from the win rates.
- **ρ is roughly the share of a segment's cards that are drawn.**  In the
  table for four players at 10 胡 within the gate it is 0.40 to 0.46 with 1
  to 12 draws left and falls to 0.28 to 0.36 with 20 or more: early on a
  segment has 2 to 3 public cards, most of them other seats' discards.  The
  last segment (0 draws left) has only discards, and ρ is 0.
- **The fewer the players, the higher ρ**: with fewer other seats, discards
  are a smaller share of the public cards, and fewer generals stay in other
  seats' hands.
- **The seat's own discards account for 0.07 to 0.15.**  `Graph` counts the
  card the seat is about to discard as a public card too, so batch 2 uses
  the column that includes its own discards.

The table for four players at 10 胡 within the gate, by draws left (from 0;
a segment with fewer than 500 cards takes the nearest):

```text
[0.0000, 0.4362, 0.4448, 0.4424, 0.4470, 0.4505, 0.4537, 0.4091, 0.4535,
 0.4597, 0.4278, 0.4187, 0.4207, 0.3960, 0.4032, 0.4182, 0.3869, 0.3656,
 0.3767, 0.3671, 0.3609, 0.3437, 0.3386, 0.2828, 0.2926, 0.2926]
```

`src/` is untouched.  `ab/run.sh 20000` (control `74e99e2`, seed base
1795365381): 0 deals part.

## Batch 2: the discount into `Graph`

`Outlook` gains a table `generals` (ρ by draws left).  In `Graph` the weight
of a general is multiplied by ρ, which is to take the unseen generals for ρK
cards:

- **Winning on a general** (`Kind::Win`, the card a general): weight × ρ.
- **Eating a general** (`Kind::General`): weight × ρ, times the `general` it
  had before.
- **Other cards**: N becomes N − (1 − ρ)K, and the weights grow with it.
  The generals no longer counted are not empty public cards but other cards;
  part of the few percentage points by which a 搭子 is underestimated comes
  from here.

`rest` is 1 less the sum of the weights, as before.  ρ changes with time, so
the weights are multiplied in `sweep` by that step's segment and cannot be
fixed in `expand`; building the graph does not change.

Calibrate first, then A/B:

1. **Calibration.**  Rerun `examples/overspend` on a fresh seed and read the
   table of waiting hands above.  The gate: in the three cells of fully melded
   hands that extend 0, the estimated win rate falls within ± 2 standard errors
   of the actual, and no 搭子 row gets worse.  If it fails, go back to the
   shape of ρ (split within and past the gate?  split by who drew?) and run
   no A/B.
2. **A/B.**  The candidate is `ExpectBot`, the control `main` HEAD's own
   `ExpectBot` (`BASELINE_EXPECT=1`), once each on the web's and the
   desktop's settings, `RESHUFFLE=8`, a fresh seed base.  The deal count is
   fixed before any score is seen: first time 5,000 deals on another seed
   and throw the result away; batch 4 of `doc/eval.md` had standard errors
   of 0.046 and 0.052 points at 30,000 and 20,000 deals, and the effect here
   is put below 0.1 points, so expect to need more deals.  If the estimate
   is over an hour, detach and write the state to memory.

The `Eval` tables were learnt under the old `Outlook` and are not relearnt in
this batch: the leaves past the gate still carry the old overestimate, and
only the first layer or two of the graph and the play within the gate move
to the new one.  The inconsistency is deliberate; batch 3 deals with it.

The verdict follows `CLAUDE.md`: a win only when the CI lies above 0; a CI
that straddles 0 is insufficient data; a loss is not final until the deals
that moved the most have been traced.

### What batch 2 measured

`Outlook::generals` is batch 1's table for four players at 10 胡 within the
gate, used only at four players and 10 胡; other tables have `[1.0]` and no
change to play.  `examples/overspend`, 20,000 rounds, seed base 1795366613,
base `a9e194e` plus this change, fixed before the run (34 minutes).

Estimate − actual (percentage points), batch 0 → batch 2:

| | 15 draws or more | 8–14 draws | 0–7 draws |
| --- | ---: | ---: | ---: |
| Fully melded, extends 0 | +15 → −3 | +19 → −5 | +20 → −2 |
| Fully melded, extends 4 or more | −3 → −14 | −2 → −9 | +14 → −2 |
| 搭子, waits on 1 | −8 → −16 | −4 → −6 | −4 → −2 |
| 搭子, waits on 2 | −5 → −13 | −10 → −13 | −8 → −11 |
| 搭子, waits on 3 | −9 → −13 | −1 → −6 | +7 → +5 |
| 搭子, waits on 4 | −8 → −13 | −4 → −3 | 0 → −3 |
| 搭子, waits on 5 or more | −5 → −3 | −3 → −2 | −1 → +8 |

- **The three cells that extend 0 pass**: 47% ± 5 estimated at 44%, 45% ± 4
  at 40%, 27% ± 5 at 25%.
- **The 搭子 rows fail**: with 15 draws or more left, those that wait on 1
  to 4 cards are underestimated by 13 to 16 percentage points, where batch 0
  had 5 to 9.  Those that extend 4 or more also went from accurate to 14
  percentage points under.  The gate is not passed; no A/B is run.

The two batches differ in both seed and play, so the first 5,000 rounds were
rerun on the same seed base, each hand estimated both with ρ and with ρ = 1
(a temporary trace, not kept).  The same hands, 15 draws or more left:

| | Actual | ρ | ρ = 1 | Hands |
| --- | ---: | ---: | ---: | ---: |
| Fully melded, extends 0 | 53% ± 8 | 43% | 64% | 38 |
| Fully melded, extends 2 | 72% ± 5 | 54% | 70% | 71 |
| Fully melded, extends 4 or more | 79% ± 2 | 67% | 77% | 296 |
| 搭子, waits on 1 | 41% ± 5 | 27% | 32% | 103 |
| 搭子, waits on 3 | 55% ± 3 | 42% | 47% | 321 |
| 搭子, waits on 4 | 53% ± 3 | 45% | 46% | 333 |
| 士象 waiting for its own general | 39% ± 5 | 27% | 41% | 98 |

- **The 搭子 got worse because of ρ itself, not because the hands changed.**
  N − (1 − ρ)K gives the other cards about 8% more weight, but a 搭子 also
  has fewer routes that eat a general and then win; the net is 1 to 5
  percentage points lower.
- **ρ is right for the generals; what is wrong is who gets the share they
  lose.**  Extensions rest on pongs and concealed triplets, and a 搭子
  mostly waits on 車馬炮 and 士象: in batch 2b's table these all run 15 to
  33% above the average.  N − (1 − ρ)K spreads the share evenly over all
  cards, which gives them too little.
- **The shape of ρ needs no change**: a hand that extends 0 waits on
  generals alone, and all three cells came out right.

## Batch 2b: the appearance rate of every rank

Batch 2 fixes only the generals' row; this batch replaces the whole
assumption and is the second half of the main line.  `Graph` assumes the
unseen cards are spread evenly: the chance that a public card is a given
card is its unseen copies ÷ N.  Batch 1 rebuilds the cards each seat has not
seen, and while at it extends ρ to every rank: the times a public card is of
the rank ÷ the expectation under an even spread.  The same cards as batch 1:
four players at 10 胡, `HeuristicBot` self-play of 20,000 rounds, seed base
1795364857, base `74e99e2`, within the gate, the seat's own discards
included, ± one standard error:

| Rank | 15 draws or more | 8–14 draws | 0–7 draws |
| --- | ---: | ---: | ---: |
| Generals | 0.376 ± 0.004 | 0.422 ± 0.004 | 0.431 ± 0.006 |
| 士仕 | 1.161 ± 0.008 | 1.184 ± 0.008 | 1.257 ± 0.014 |
| 象相 | 1.186 ± 0.008 | 1.213 ± 0.008 | 1.251 ± 0.014 |
| 車俥 | 1.271 ± 0.008 | 1.252 ± 0.008 | 1.246 ± 0.014 |
| 馬傌 | 1.290 ± 0.008 | 1.263 ± 0.008 | 1.249 ± 0.014 |
| 包炮 | 1.333 ± 0.008 | 1.303 ± 0.008 | 1.298 ± 0.014 |
| Soldiers | 0.684 ± 0.006 | 0.787 ± 0.006 | 0.882 ± 0.011 |

Without the seat's own discards, generals are 0.47 to 0.49, soldiers 0.76,
0.84 and 0.92, and the other ranks 1.16 to 1.26.

- **What the generals lose is not spread evenly over the other cards.**  For
  each public card the ranks' chances sum to 1; 車馬炮 and 士象 run 15 to
  33% over, being the cards the other seats discard, and soldiers some 30%
  under early on, nearing the average only later, being kept for soldiers of
  three and four colours.  Batch 2's N − (1 − ρ)K spreads what the generals
  lose evenly over the other cards; this table says otherwise.
- **A 搭子 being underestimated by 5 to 9 percentage points with 15 draws or
  more left** (batch 0's table of waiting hands) may come partly from here:
  a 搭子 that waits on 車馬炮 waits on just the cards that appear more, and
  one that waits on soldiers the reverse.

### Draws and discards apart

Public cards have two sources: draws from the pile and the other seats'
discards.  `examples/outlook` tells the two apart; four players at 10 胡,
`HeuristicBot` self-play of 20,000 rounds, seed base 1795370029, base
`a9e194e`, within the gate, without the seat's own discards:

| Rank | Drawn, 15 or more | 8–14 | 0–7 | Discarded by other seats, 15 or more | 8–14 | 0–7 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Generals | 0.759 | 0.684 | 0.636 | 0 | 0 | 0 |
| 士仕 | 1.096 | 1.122 | 1.134 | 1.247 | 1.206 | 1.293 |
| 象相 | 1.135 | 1.115 | 1.130 | 1.252 | 1.288 | 1.456 |
| 車俥 | 1.023 | 1.036 | 1.109 | 1.490 | 1.519 | 1.483 |
| 馬傌 | 1.046 | 1.101 | 1.111 | 1.534 | 1.600 | 1.641 |
| 包炮 | 1.050 | 1.122 | 1.142 | 1.557 | 1.630 | 1.674 |
| Soldiers | 1.009 | 1.046 | 1.125 | 0.381 | 0.456 | 0.500 |

The standard errors are 0.006 to 0.017 for draws and 0.008 to 0.033 for
discards.

- **Discards are skewed the most**: no generals, soldiers at only four to
  five tenths of the average, 車馬炮 five to seven tenths over.
- **Draws are not spread evenly over the unseen cards either.**  The pile
  really is shuffled, but the unseen cards are the pile plus the other
  seats' concealed cards, and those are not random: a dealt general always
  stays.  So the rate of drawing a general is 0.76, and lower the later it
  is (0.64): the pile keeps shrinking, and the other seats' concealed cards
  make up more and more of the unseen cards.  The other ranks' draw rates
  are 1.0 to 1.14, close to an even spread of what the generals lose.
- **Batch 1's "ρ is roughly the share of a segment's cards that are drawn"
  is the mix of these two columns**: one drawn card to a segment, the rest
  discards, generals at 0.7 × the share drawn.
- **What it means for batch 2's calibration**: N − (1 − ρ)K is about right
  for draws and far off for discards: discarded 車馬炮 and 士象 come 20 to
  60% more often than it reckons, and these are just what a 搭子 waits on
  and what an extension needs to pong.  Batch 2b's tables can be learnt by
  these two columns apart and mixed by the draw share of `Outlook::cards`,
  which is smoother than learning the mixed rate directly.

How: `Outlook`'s `generals` gives way to two tables by rank and by three
bands of draws left, `drawn` and `discarded`.  One card of a segment is
drawn (a share of 1 ÷ `cards`; the last segment has none) and the rest are
discards.  At each step `Graph::sweep` normalises each of the two and then
mixes them: the drawn share goes to each card by "unseen copies × draw
rate", the discarded share by "unseen copies × discard rate".  Building the
graph does not change; `Entry` only records the rank as well.

- **士象 share one rate, and 車馬炮 share one.**  `Eval`'s features take
  these ranks for interchangeable
  (`symmetric_hands_share_patterns_and_values`), and a rate for each would
  break that symmetry.  Within a class the gap is within 4%, the one
  exception being discarded 象相 with 0–7 draws left (1.46 against 1.29 for
  士仕).
- **The seat's own discards are not taken off separately.**  Of the discards
  a seat within the gate sees, 30 to 40% are its own (0.402 with 15 draws or
  more left, 0.335 with 8–14, 0.302 with 0–7), cards it can neither win on
  nor pong.  `Outlook::own` can take this share off the discarded part
  (`eat` already counts the seat's own discards in its denominator, so `eat`
  is divided back at the same time), but the calibration says that is worse
  (the table below), so it stays at 0 and h absorbs it as before.

The gates and the method are batch 2's:

1. **Do it once batch 2's calibration passes**, without waiting for its A/B
   to win: if the generals' row straddles 0, the whole table is still the
   more complete fix.  The control is the `main` HEAD of the time; if batch
   2 did not go in, the generals' row is measured along with the rest.
2. **Calibration.**  Rerun `examples/overspend` on a fresh seed; the gate is
   that the 搭子 rows of the waiting table improve and no fully melded row gets
   worse.  If it fails, no A/B is run.
3. **A/B.**  Candidate and control are both `ExpectBot`,
   `BASELINE_EXPECT=1`, the deal count fixed before any score is seen.

### What batch 2b measured

The rates are those of the cards under "Draws and discards apart" above
(seed base 1795370029), with 士象 and 車馬炮 pooled:

| | Generals | 士象 | 車馬炮 | Soldiers |
| --- | ---: | ---: | ---: | ---: |
| Drawn, 15 or more | 0.759 | 1.116 | 1.040 | 1.009 |
| Drawn, 8–14 | 0.684 | 1.118 | 1.086 | 1.046 |
| Drawn, 0–7 | 0.636 | 1.132 | 1.121 | 1.125 |
| Discarded, 15 or more | 0 | 1.250 | 1.527 | 0.381 |
| Discarded, 8–14 | 0 | 1.247 | 1.583 | 0.456 |
| Discarded, 0–7 | 0 | 1.373 | 1.597 | 0.500 |

Calibration: `examples/overspend`, 20,000 rounds, seed base 1795450000, base
`a9e194e` plus this change, fixed before the run.  Seat 1 plays the
candidate (the new rates, h × 1.5), and each hand records the estimates of
several models at once (a temporary trace, not kept).  Estimate − actual
(percentage points):

| | Actual | Even draw, h × 1.5 (current) | Rates, h × 1.5 (candidate) | Rates, h × 1 | Rates, own discards taken off, h × 1 | Hands |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 15 draws or more: fully melded, extends 0 | 49% ± 4 | +17 | −7 | +3 | +3 | 126 |
| Fully melded, extends 2 | 65% ± 3 | +5 | −12 | −3 | −4 | 354 |
| Fully melded, extends 4 or more | 78% ± 1 | −1 | −13 | −5 | −7 | 1,108 |
| 搭子, waits on 1 | 36% ± 2 | −6 | −10 | −1 | −5 | 371 |
| 搭子, waits on 3 | 55% ± 1 | −9 | −13 | −3 | −8 | 1,400 |
| 搭子, waits on 4 | 55% ± 1 | −9 | −9 | 0 | −5 | 1,490 |
| 搭子, waits on 5 or more | 58% ± 1 | −4 | −9 | 0 | −4 | 1,269 |
| 士象 waiting for its own general | 35% ± 2 | +5 | −11 | −3 | −4 | 367 |
| 8–14 draws: fully melded, extends 0 | 42% ± 4 | +15 | −6 | +3 | +3 | 159 |
| Fully melded, extends 4 or more | 67% ± 2 | +1 | −11 | −1 | −2 | 409 |
| 搭子, waits on 3 | 38% ± 2 | −3 | −5 | +4 | +1 | 568 |
| 搭子, waits on 4 | 40% ± 2 | −3 | −3 | +6 | +3 | 404 |
| 0–7 draws: fully melded, extends 0 | 22% ± 4 | +20 | +3 | +7 | +7 | 87 |
| 搭子, waits on 3 | 25% ± 4 | −2 | −2 | +3 | +1 | 117 |
| All 33 cells, mean absolute miss weighted by hands | | 5.9 | 9.4 | 2.4 | 4.5 | 12,419 |
| All, win rate | 52.3% | 50.1% | 43.0% | 51.9% | 48.9% | |

- **The candidate as first declared (rates, h × 1.5) fails the gate**: with
  15 draws or more left the 搭子 rows go from 4 to 9 under to 9 to 13 under,
  as in batch 2.  Batch 2's diagnosis, "the lost share goes to the wrong
  cards", is wrong: given to 車馬炮 and 士象 as measured, a 搭子 still comes
  out lower, because its routes through generals and soldiers are fewer too.
- **h × 1.5 was offsetting the generals' overestimate all along.**  The 1.5
  of `research/outlook.md` was calibrated on the even-draw model; that model
  takes generals for two and a half times what they are, and scaling h up
  just pulls the overall win rate back, at the cost of underestimating the
  hands that do not rely on generals.  With the rates right there is no more
  need to scale: with the rates and the measured h the miss over the 33
  cells is 2.4 percentage points against 5.9 for the current model, and
  51.9% overall against an actual 52.3%.
- **The misses that remain**: fully melded hands and those waiting for
  generals with 0–7 draws left are still 5 to 9 percentage points high, and a
  搭子 that waits on 4 with 8–14 draws left is 6 high.
- **Taking off the seat's own discards makes it worse instead** (4.5), so
  `own` is 0.  It would need a smaller h to come level, and h is measured:
  there is no reason to tune it again.
- **So the A/B's candidate becomes the rates with h × 1** (`ExpectBot::new`
  at four players and 10 胡), other tables unchanged.  This candidate was
  settled after seeing the calibration and has not played a calibration of
  its own; the verdict is left to the A/B.  The `Eval` tables were learnt
  under the even draw and h × 1.5, so the leaves are still the old ones;
  batch 3 deals with that.

The A/B, fixed before the run (launched 2026-11-23).  The candidate is the
working tree's `ExpectBot::new` (rates, h × 1), the control `a9e194e`'s
`ExpectBot::new`, `RESHUFFLE=8`, the commands in
`target/generals-2b/run.sh`:

| Setting | Environment variables | Deals | seed base |
| --- | --- | ---: | --- |
| Web | `EXPECT=new GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 60,000 | 1794000000 |
| Desktop | `EXPECT=new GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1 BASELINE_GATE=14 BASELINE_SHALLOW=17` | 20,000 | 1794200000 |

The web setting was timed over 5,000 deals on another seed (1795310000): 9
minutes, standard error 0.075, the result thrown away; the standard error at
60,000 deals is put at 0.022.  It is adopted only if both settings win.

Result (2026-11-23): both win; adopted.

| Setting | Deals that part | Candidate − control (points a deal) | 95% CI | Seat 1 win rate | Verdict |
| --- | ---: | ---: | --- | --- | --- |
| Web | 25,242 / 60,000 | +0.056 ± 0.023 | [+0.012, +0.101] | 25.03% → 25.20% | win |
| Desktop | 10,655 / 20,000 | +0.103 ± 0.043 | [+0.019, +0.187] | 25.19% → 25.48% | win |

The waiting table of `examples/overspend` now uses the bot's own `Outlook`
(rates, h × 1), so what batch 4's rerun measures is the current model.

A cheaper version besides: `HeuristicBot`'s `waits` weighted by the ranks'
rates, on the default seat of `ab/run.sh`, measured together with batch 4's
generals × ρ or apart.

Caveats:

- **These are statistics of `HeuristicBot` self-play.**  Which cards appear
  depends on what the other seats discard; they must be measured again when
  all four seats are `ExpectBot`, and again when play changes.
- **What is measured is the appearance rate, not where the cards are.**
  `Graph` needs only the former.  `View::determinize` deals the unseen cards
  evenly to the other seats and the pile; knowing what the other seats
  mostly hold takes another measurement, which is not in this batch.
- **Ranks may not be fine enough.**  The four colours of one rank, and how
  many copies of the card have already appeared, may each have a different
  rate.  Ranks first; split if the calibration fails.

## Batch 3: relearn `Eval`

Done only after batch 2 or 2b has won and 2b has its verdict; the table is
relearnt only once.  Rerun the labelling and training of `examples/eval`
under the new `Outlook` (the commands and round count of batch 3 of
`doc/eval.md`, a fresh seed), replace `src/eval-16.bin` (the table of two's
`eval-16-2.bin` waits until its own ρ is measured), and A/B against the
`main` HEAD of the time.  Commit only on a win.

Declared (2026-11-23, written before the run; base and control are both
`3423521`):

- **Labelling:** `EXPECT=1 eval label 2000 1796250000`, 2,000 rounds.
  `EXPECT=1` is the `Outlook` of `ExpectBot::new` at four players and 10 胡
  (rates, h × 1).  From batch 3's 8,466 seconds the estimate is about 2.5
  hours; detach.
- **Training:** `eval train labels eval 20 1 4 16`, as before; of the three
  D, the table with the smallest validation RMSE goes in.
- **Only four players at 10 胡 changes.**  The table of three and 8 胡 read
  `Eval::default` too, but their `Outlook` is still the even draw with
  h × 1.5, consistent with the old table; the new table is stored apart, and
  `Eval::of` reads it only at four players and 10 胡.
- **The offline loss table is not run.**  Batch 3's gate has already been
  passed; this is only a change of table, and the verdict is left to the
  A/B.
- **A/B:** the settings, deal counts and seeds are written here once the
  labelling is done and timed, before any score is seen.

Labelling: 14,468 graphs, 1,959,151 nodes, 66,918,770 values, 8,767 seconds.
Training, validation RMSE (that of the mean by t is 3.55): D = 1 2.27, D = 4
1.23, **D = 16 0.90** (the old table 0.78, when that of the mean was 3.37).
D = 16 goes in.

The A/B, fixed before the run (2026-11-23).  The candidate is the working
tree's `ExpectBot::new`, reading the new table past the gate
(`EVAL=target/generals-3/eval-16.bin`, not yet compiled into the binary),
the control `3423521`, `RESHUFFLE=8`, the commands in
`target/generals-3/run.sh`.  The settings and deal counts are batch 2b's,
and so are the timings (the web, 60,000 deals in 1.6 hours; the desktop,
20,000 deals in 3.5 hours), with no trial run of its own:

| Setting | Environment variables | Deals | seed base |
| --- | --- | ---: | --- |
| Web | `EXPECT=new GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 60,000 | 1795429248 |
| Desktop | `EXPECT=new GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1 BASELINE_GATE=14 BASELINE_SHALLOW=17` | 20,000 | 1796429248 |

The new table is compiled in only if both win (at four players and 10 胡
only).

Result (2026-11-24): neither is a win; the table is not replaced and this
batch is dropped.

| Setting | Deals that part | Candidate − control (points a deal) | 95% CI | Seat 1 win rate | Verdict |
| --- | ---: | ---: | --- | --- | --- |
| Web | 37,228 / 60,000 | +0.019 ± 0.025 | [−0.030, +0.069] | 25.50% → 25.55% | insufficient data |
| Desktop | 9,262 / 20,000 | +0.020 ± 0.036 | [−0.050, +0.091] | 25.94% → 25.94% | insufficient data |

- **A CI that straddles 0 is insufficient data: not a loss, and not "no
  difference" either.**  The new table was not measured to be worse, so the
  deals that moved the most were not traced.
- **It is dropped because it is not worth measuring further.**  Both
  settings' estimates are around +0.02; if that is the true size, the web
  needs some 360,000 deals (about 14 hours) before the CI leaves 0, and what
  that buys is a table only four players at 10 胡 read.
- **The inconsistency at the leaves stays.**  The `Eval` of four players at
  10 胡 is still the table learnt under the even draw and h × 1.5, while
  the graph uses the rates and h × 1.  Batch 2b's win was measured on just
  this inconsistency.
- The new table stays in `target/generals-3/eval-16.bin` and did not go into
  `src/`.  When `Outlook` changes again and a relearn is due, label afresh
  rather than carry on from this one.

## Batch 4: overspending before the hand is waiting

Once batches 2 and 2b have their verdicts, rerun `examples/overspend` on a
fresh seed:

- **The share of B1 that is eaten falls below 10%**: with the overestimate
  fixed the engine stops eating by itself, and this line ends.
- **Otherwise go back to the appearance rates first**: whichever cell of the
  waiting table is still off shows which rate is not yet fixed.  Only when the
  table is accurate and the engine still eats is the probe made.  Read the
  probe as a diagnosis first: the group it wins in is where `Graph` is still
  wrong.  As in batch 2 of `doc/hu-ceiling.md`: the candidate `ExpectBot` does
  not eat a B1 pair unless the hand is fully melded after eating.  The groups
  declared beforehand:

  | Group | Condition | Expected |
  | --- | --- | --- |
  | I | Not fully melded after eating, and not waiting if it passes | Passing is better |
  | II | Fully melded after eating; passing leaves a 搭子 that waits on 3 or more | Passing is slightly better or level |
  | III | Fully melded after eating, otherwise | Eating is better (the probe does not block it; a control only) |

  The probe and the trace are saved as a patch and do not go into `main`.
  Commit only on an overall win; if only a group's CI lies above 0, narrow
  to it and measure once more, and the verdict looks at that run alone.

Another candidate belongs in this batch: `HeuristicBot`'s `waits` counts a
general as `WAIT` like any other card it waits on.  Multiplying generals by
ρ is an A/B by itself, on the default seat of `ab/run.sh`, and far cheaper.

Batch 4's declaration (2026-11-24, written down before the runs), two runs
that do not depend on each other, both queued through pueue:

- The statistics: `examples/overspend`, 20,000 rounds as in batch 0, seed
  base 1796461521, on `8dc900b` unchanged.
- The A/B of `waits`: a general the hand would win on counts 3 where any
  other card counts `WAIT` = 8 (8 × 0.406, rounded), at every table.
  `ab/queue.sh generals-wait`: 200,000 deals, `RESHUFFLE=8`, four players
  at 10 胡, control `8dc900b`, seed base the clock when the task starts.  The
  other tables' ρ differ, so a win here ships the 3 for four players at 10
  胡 only, until each of them has an A/B of its own.
- The other tables (declared the same day, before task 1's data): the
  weight is now 8 × the table's ρ, rounded, so 3 at four players (10 and 8
  胡), 4 at three and 5 at two, and 8 where no ρ was measured.  Three A/Bs
  queued with `AFTER=1`, each run only if the A/B above is a win, each
  200,000 deals, `RESHUFFLE=8`, control `8dc900b`, seed base the clock at
  its start: `PLAYERS=3`, `PLAYERS=2`, `THRESHOLD=8`.  Each table ships its
  weight on its own win; the others go back to 8.

The probe waits on the statistics, and batch 5 on this batch's verdicts.

### What batch 4 measured

The A/B of `waits` (2026-11-24), `HeuristicBot` with a general counting 3,
four players at 10 胡, control `8dc900b`, `RESHUFFLE=8`, seed base
1795461522:

| Deals that part | Candidate − control (points a deal) | 95% CI | Seat 1 win rate | Verdict |
| ---: | ---: | --- | --- | --- |
| 435 / 200,000 | −0.000 ± 0.001 | [−0.003, +0.002] | 24.10% → 24.10% | insufficient data |

- **The weight almost never changes a play**: one deal in 460 parts, and
  among those the mean is −0.097 points.  The CI straddles 0, so it is
  insufficient data, and at ± 0.003 points a deal nothing here is worth
  more deals.  `WAIT` stays 8 for every card; `src/heuristic.rs` is
  reverted.
- **The three other tables did not run**: they were gated on a win here.
  The patch is in `target/generals-4/wait-general.patch`.

The statistics: `examples/overspend`, 20,000 rounds, seed base 1796461521,
base `8dc900b`; the CSVs are in `target/generals-4/`.  The 51,586 decisions
where a pair could be eaten, batch 0 → batch 4:

| | Decisions | Eaten | Share eaten | Own estimate, eat − pass (points) |
| --- | ---: | ---: | ---: | ---: |
| C: within the limit | 39,686 → 40,099 | 19,830 → 19,948 | 50% → 50% | +0.11 → −0.12 |
| B1: one short only after eating | 4,176 → 4,144 | 1,039 → 781 | 24.9% → 18.8% | −2.23 → −2.72 |
| B2: one short already before eating | 1,629 → 1,384 | 616 → 502 | 38% → 36% | −0.66 → −0.55 |
| A: short even with one more drawn | 5,826 → 5,959 | 243 → 271 | 4% → 5% | −2.31 → −1.86 |

- **The gate is not passed**: 18.8% of B1 is eaten, down a quarter from
  24.9%, above the 10% that would end the line.
- **The eats the overestimate drove are gone.**  Fully melded after eating
  and extending 0, B1 eats 27 of 60 where batch 0 ate 35 of 39.  Over
  B1, B2 and C, when passing leaves a 搭子 that waits on 3 or more, it eats
  15 of 52 (7 of 9 on 3, 8 of 37 on 4, 0 of 6 on 5 or more) where batch 0
  ate 45 of 47.
- **The B1 pairs still eaten are marginal.**  The 781 are better than the
  best other play by 0.84 points on average by its own estimate (median
  0.39; 93% under 2 points), 0.033 points a round in all.  B1 won 7% ± 1
  after eating and 15% ± 1 after passing, but that is observation: it
  passes with the better hands.

The waiting table, actual win rate ± its standard error (the engine's
estimate, hands):

| | 15 draws or more | 8–14 draws | 0–7 draws |
| --- | ---: | ---: | ---: |
| Fully melded, extends 0 | 60% ± 4 (52%, 125) | 44% ± 4 (45%, 174) | 25% ± 4 (30%, 116) |
| Fully melded, extends 1 | 65% ± 4 (58%, 131) | 45% ± 4 (52%, 172) | 48% ± 6 (37%, 61) |
| Fully melded, extends 2 | 67% ± 3 (62%, 343) | 51% ± 3 (57%, 305) | 40% ± 5 (43%, 86) |
| Fully melded, extends 3 | 70% ± 3 (66%, 296) | 62% ± 4 (60%, 185) | 62% ± 9 (48%, 29) |
| Fully melded, extends 4 or more | 79% ± 1 (73%, 1,165) | 62% ± 3 (67%, 374) | 54% ± 6 (52%, 71) |
| 搭子, waits on 1 | 43% ± 3 (35%, 327) | 26% ± 3 (25%, 207) | 13% ± 4 (14%, 62) |
| 搭子, waits on 2 | 42% ± 3 (44%, 357) | 37% ± 3 (34%, 246) | 16% ± 4 (19%, 68) |
| 搭子, waits on 3 | 52% ± 1 (52%, 1,342) | 39% ± 2 (42%, 532) | 32% ± 4 (28%, 121) |
| 搭子, waits on 4 | 57% ± 1 (55%, 1,402) | 41% ± 2 (46%, 444) | 22% ± 5 (34%, 77) |
| 搭子, waits on 5 or more | 57% ± 1 (58%, 1,274) | 47% ± 2 (51%, 410) | 35% ± 6 (42%, 74) |
| 士象 waiting for its own general | 32% ± 3 (33%, 338) | 17% ± 3 (22%, 163) | 13% ± 5 (13%, 38) |

- **The mean absolute miss over the 33 cells, weighted by hands, is 3.3
  percentage points**, and no row is off the same way in all three bands.
  The hands that extend 0, 15 to 20 high in batch 0, are now 8 low, 1 high
  and 5 high.
- **What is left has a pattern by draws, not by rank**: with 15 draws or
  more left every fully melded row is 4 to 8 low (extends 4 or more, 79% ± 1
  against 73%, is the surest), and with 8–14 left most rows are 3 to 7
  high.  That points at h or `cards` by draws left, not at a rank's rate.
- **The 搭子 rows with 15 draws or more left are within 2**, save the one
  that waits on 1 (8 low).

The probe is skipped (2026-11-24) and this line ends here.  The declaration
says the probe is made only when the table is accurate and the engine still
eats.  The table is close, and the engine still eats 18.8%, but what the
probe could win is about the 0.033 points a round above, and batch 3 could
not resolve 0.02 at the desktop's settings in 20,000 deals (standard error
0.036).  Whether the B1 pairs still eaten win or lose was not measured.

## Batch 5: the book

「對子能有幾個」 (How many pairs there can be) has been removed
(`../tsap-oo-book` `86caae2`), and what remains moved to `src/material.md`,
「吃對子是在湊張數」 and 「把搭子吃成對子」 (Eating a 搭子 into a pair),
`src/seven.md` (lone cards and pairs differ by a multiple of 3) and
`src/generals.md` (the safety line).  The book no longer teaches people to
count a limit, so the terms extension, fully melded and overspend stay out.
By the `AGENTS.md` over there, what play measures is written as a direction
only, without numbers.  What is left to change is in `material.md`:

- The three rules of 「對子該不該吃」 (whether to eat a pair) are
  `HeuristicBot`'s results; `ExpectBot` is the hint now.  The second (do not
  eat a pair that leaves the hand unable to reach 10 胡, assuming no general
  is drawn afterwards) is rewritten once batches 2 and 4 have their
  verdicts.
- **Write more only if the verdict disagrees with "laying the hand out as
  seven groups is enough".**  For instance, if batch 4 measures that pairs
  made good by generals mostly lose, add a sentence to that passage not to
  count on later generals; if they are measured to hold up, leave it alone.

## Closing each batch

As `CLAUDE.md` says: `cargo fmt`, clippy, the three test suites, the docs,
`CHANGELOG.md`, a proposed commit message, and a commit only when the user
says so.  Win or lose, a measurement's numbers, deal count, seed base,
control SHA and verdict are written down, and the progress table of this
document is updated.

## Not doing

- **No search for a formula for the number of pairs.**  Such a formula may
  not exist, and may be too complicated if it does: how many pairs can be
  eaten depends on the 胡, the extensions, the draws left and whether each
  kind of card will still appear.  A drawn general must be eaten, and each
  one that comes asks for one more pair to go with it, so how many pairs
  the hand needs in the end cannot be fixed beforehand; yet for that very
  reason a pair in hand is better than a 搭子, since a general fills its
  group the moment it comes.  Eating too many pairs is the result of
  misjudged appearance rates; with the rates right, `Graph` works it out by
  itself.
- **No change to `ceiling` or `Prune::generals`.**  The ceiling only answers
  whether the threshold can be reached; how often it is reached is
  `Outlook`'s business.
- **No guessing how many generals the other seats hold.**  ρ is a table and
  does not look at what the other seats ate or ponged this round.
- **The tables of two and three players and of 8 胡 do not follow along.**
  Each one's ρ can be measured in passing, but each changes only on an A/B
  of its own, as is the rule of `Outlook::new` now.

## Risks and open questions

- **ρ may not be the cause.**  The gap in win rates may also come from the
  other seats winning more often than h says in such positions.  Batch 1's
  gate is there for this.
- **One table may not be enough.**  ρ may differ by whether the seat or
  another drew the general, and within and past the gate.  One table first;
  split if the calibration fails.
- **The payoff may be too small to measure.**  A fully melded hand that
  extends 0 comes up only about 0.03 times a round; the effect comes mostly
  from the decisions a few steps before it, and its size is unknown.
- **The statistics come from one seat of `ExpectBot` against three
  `HeuristicBot`s.**  With `ExpectBot` in all four seats, generals may stay
  in hand differently.
- **Batch 0's table was read on one set of cards.**  Batch 2's calibration
  and A/B must both use fresh seeds.
