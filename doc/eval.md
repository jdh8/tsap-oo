# The eval beyond the gate: a shallow DP with values at the leaves: design

Written 2026-11-19, base `b22f1f0`.  This document takes over application I
of `doc/expectation.md` (labels for long hands) and route 4 of
`doc/stronger-bot.md` (distilling into a fast model), with the teacher changed
from Monte Carlo to the single-player dynamic programming.  No code is written
yet; the costs below are estimates except where marked "measured".

## The questions to answer

`ExpectBot`'s gate is 14 cards on the desktop and 10 on the web.  The 14-card
gate lets through only 39% of the decisions (measured, `expectation.md`
application D); 15 to 17 cards are 31% and 18 or more 29%, and that 60% is
played as `HeuristicBot`.  The exact graph is too dear there: 1.6 seconds at
17 cards, 22 seconds at 20 (measured, `research/dp-nodes.md`).

The proposal has the shape of a chess engine: the graph is expanded only one
or two levels, and a leaf's value comes from an eval that takes about a
microsecond.  The eval's unit is points, and what it learns is the value of
the exact DP.

1. How many points do `HeuristicBot`'s choices at 15 to 20 cards give up by
   the exact values?  This is the upper bound on the prize.
2. How accurately can the eval learn the exact values?
3. With one or two levels added, how far is the choice from the exact one,
   and how many milliseconds does a decision take?
4. Does it beat the `ExpectBot` that plays as `HeuristicBot` beyond the gate?

## Progress

Each batch is about one session.  Update this table when a batch is done.

| Batch | Output | Changes play | Gate | Status |
| --- | --- | --- | --- | --- |
| 0 | This document | No | None | Done |
| 1 | The prize: how many points `HeuristicBot` gives up by the exact values at each card count; the calibration of long hands | No | A predicted gain at 15 to 20 cards of 0.05 points a deal or more | Done: +0.35 ± 0.07; the win rate of long hands is underestimated by 24% |
| 2 | The framework: a depth limit on the graph, the leaves' curves, a census by depth, the sensitivity to leaf error | No, to be measured at zero deals that part | Depth 1 within 100 milliseconds on average at 20 cards | Done: 29 milliseconds; web k = 1, desktop k = 2 up to 17 cards; k = 1 tolerates σ 2, bias ±1, λ 0.5 |
| 3 | Labels, pattern tables, the offline loss table | No | A predicted gain on new rounds of 0.05 points a deal or more, with the bias inside the range batch 2 sets | Done: D = 16, web +0.34 ± 0.06, desktop +0.27 ± 0.06; k = 1 bias −0.03, slope 0.97, residual 0.65 |
| 4 | The shallow graph beyond the gate, one A/B each for the web and the desktop | Candidate | Commit only on a win | Done: web +0.19 ± 0.05, desktop +0.20 ± 0.05, both wins |
| 5 | Applications, see the table below | Each its own | One win in batch 4 | May start; A is done with batch 4 |
| | Tables of two and three: fall back to `HeuristicBot` when the cards swept exceed the length of the table | Yes | Commit only on a win | Done: zero deals part at a table of four; +10.4 to +11.3 at tables of two and three, all wins |

0.05 points a deal is the lower edge of the improvements this repo can
measure: batch 3 of `hu-ceiling.md` is +0.053, and the standard error of a
reshuffled A/B of 100,000 deals is about 0.014.

## Shape

### The shallow graph

The recursion is that of `expectation.md`, unchanged.  The roots are every
hand the decision at hand can leave (discarding a card, or claiming and then
discarding one).  A node k claims from a root is not expanded and is a
**leaf**: when the sweep reaches step t, its value is read from the eval's
curve V̂(hand, exposed 胡, t).

- **k = 0** is a pure eval bot: the hand each choice leaves is valued
  directly, at a cost of the same order as `HeuristicBot`'s.
- **k = 1, 2** look one or two claims further.
- **The ceiling pruning stays.**  A hand that cannot reach the threshold is
  still merged into the "will not win" node, whose value is exact, so the
  eval need not learn dead hands.
- **Nothing changes inside the gate.**  With the hand inside the gate the
  full graph is built as before.

The costs follow from the edge counts of `dp-nodes.md` (at 20 cards about 83
edges a node and 31 microseconds a node, at 17 cards about 60 edges):

| k | Leaves (an estimate) | A decision (an estimate) | Where it is affordable |
| ---: | ---: | --- | --- |
| 0 | 10 to 60, the roots themselves | within 0.1 milliseconds | the web, playouts |
| 1 | 10² to 10³ | a few milliseconds | the web |
| 2 | 10⁴ to 10⁵ | 0.1 to 0.5 seconds | the desktop |

The same kind of estimate for route 3 was two to three orders of magnitude
low, so batch 2 takes a census first.

### Three differences from a chess engine

- **A level is a claim, not a card.**  A pass stays in the same node with t
  one less.  So a leaf needs a whole curve over t, t at most 45 (a table of
  four starts with 25 draws; the cards of each segment of
  `Outlook::default()` added up).
- **It is an expectation, not minimax.**  The other seats are still
  `Outlook`'s four constants.
- **The eval's bias does not cancel.**  In chess every line runs to a leaf,
  so an eval that is high across the board changes no choice.  Here a path
  that has not used up its k claims runs to the exhausted round and takes an
  exact 0; one that has takes the eval.  If the eval is b points high, the
  bot claims b points' worth more.  The fitted weights of
  `research/fitted-weights.md` lost exactly between eating and not eating.
  So the eval's absolute value must be calibrated, and at every t.

### The eval: pattern tables of the blocks

Cards form melds only within nine blocks (`blocks` in `src/ceiling.rs`): a
將士象 block and a 車馬炮 block for each of the four colours, and one block
of soldiers.  A block's **pattern** is, for each kind of card, (how many in
hand, how many in `live`); the two sum to at most 4, so 15 values per kind
of card.  Symmetric patterns are merged into one:

| Block | How many | Symmetry | Patterns |
| --- | ---: | --- | ---: |
| 車馬炮 | 4, sharing one table | the three kinds of card interchange | 680 |
| 將士象 | 4, sharing one table | 士 and 象 interchange | 1,800 |
| Soldiers | 1 | the four colours interchange | 3,060 |

The model has the shape of NNUE, and is much smaller:

- **One D-dimensional vector per pattern**, 5,540 in all.  The exposed 胡
  (one per 胡) and the total count of `live` (one per 8 cards) have a vector
  each as well.
- **The accumulator** a is the sum of the nine blocks' vectors and these two
  global vectors.
- **The output** is V̂(t) = w(t) · relu(a) + b(t), one set for each t from 1
  to 45; V̂(0) is fixed at 0, as in the recursion.

"Counting melds, 搭子 and lone cards" is a special case of it: D = 1, with
each pattern's number filled in by hand.  Here the data fill them in, and D
is swept from 1 to 16.  Nine table lookups and 45 inner products come to
about a microsecond a leaf (an estimate); the D = 16 table is 355 KB, about
90 KB quantised to 8 bits.

The symmetry is a property of the game, not an assumption of the model: the
four colours score alike, and 士 and 象, or 車, 馬 and 炮, have the same
standing in their melds.  Batch 3 verifies it with the exact DP: interchange
the colours or the kinds of card of the hand and `live` as in the table
above, and the roots' values do not change.

Whatever the eval cannot see is fixed at training: `Rules` (十胡) and
`Outlook` (h × 1.5).  Change either and the tables must be retrained.  八胡
has no tables of its own yet and uses the ten-胡 tables beyond the gate;
measured, and it cannot be told better from worse ("八胡: the ten-胡 tables
beyond the gate").

### Labels

One sweep of an exact graph makes the value of every node at every t a
label, with no sampling noise.  Route 2 fitted the score of a round, whose
standard deviation is 30 (`fitted-weights.md`).

- **Sample from the distribution the eval is actually asked about.**  A leaf
  is a hand one or two levels below a real decision point, so each graph
  gives only the nodes within three levels of a root, not the deep short
  hands that make up most of the graph.
- **The graphs of long hands are built offline.**  At 17 cards or fewer
  nothing is pruned; at 18 cards or more the graph uses `Discards::Loose`
  inside (about 5 seconds at 20 cards, measured) and the roots are not
  pruned.  At 13 cards or fewer `Loose` makes 3% of the decisions choose
  wrongly and costs 0.010 points a decision (measured); its cost at 18 cards
  or more has not been measured.
- **The labels do not go into the repo.**  What goes in is the example that
  makes them, the seed and the trained tables.

## Batch 1: the prize

No new core is written; the existing `ExpectBot` is used.  Seat 1 plays as
the desktop's present bot (a 14-card gate) and the other three seats are
`HeuristicBot`, the same table as the A/B's.  For each of seat 1's decisions,
the value of every choice is also computed from the graph of "Labels" above,
and these are recorded:

- **R(n):** how many points `HeuristicBot`'s first choice gives up against
  the best choice, averaged by the number n of concealed cards after the
  decision.  Claims and discards both count.
- **N(n):** how many decisions of n cards a round has.
- **Calibration:** the same graph is swept once more with `Payoff::Win`, and
  the predicted win rate is set, by card count, against whether this seat
  actually won.  The calibration of `research/outlook.md` only goes to 10
  cards.

The exchange rate comes from application D: opening the gate from 10 cards
to 14 is +0.132 points a deal, and the matching quantity in the model is
Σ N(n) · R(n), n from 11 to 14.  The ratio of the two is ρ.

**Predicted gain = ρ · Σ N(n) · R(n), n from 15 to 20.**

Gate: a predicted gain of 0.05 points a deal or more.  Short of that it
stops here, is written up in `research/`, and the bot is not changed.  The
calibration is only reported: if the predicted win rate is off by far more
at 15 cards or more than at 10 or fewer, go back to `expectation.md` and fix
the model first, and do not go on.

The round count and the seed are fixed before the run.  A few rounds are
tried for time and memory before the count is fixed: the 20-card graph
unpruned has 1.2 × 10⁸ edges (measured), which is why `Loose` is used at 18
cards or more, and the thread count depends on memory too.

Batch 1's declaration (2026-11-19, written before the run, base `9794936`):
`examples/regret.rs`, `regret 1000 1795400000`, 1,000 rounds, seed base
1795400000, clear of the seed ranges `ab/` and `expectation.md` have used.
Cards are counted the way the gate counts them: those in hand when answering
an offer, and those left after the discard when discarding.

The round count is set by a trial run: 100 rounds (seed base 1, not counted)
took 8 minutes on 10 threads, 35 CPU seconds a round and 2.6 GB at most; the
predicted gain at 15 to 20 cards was 0.23 ± 0.10.  The floor of the standard
error is application D's 0.014 / 0.132 = 11%; at 1,000 rounds the relative
errors of the two Σ N · R fall to 11% and 6%, and more rounds would save
little; about 80 minutes.

### Batch 1 results

`regret 1000 1795400000`: 7,944 decisions, 4,469 seconds (10 threads).  R is
the points given up per decision, N the decisions per round, and "Off" the
share where `HeuristicBot`'s first choice is not the best:

| Cards | Decisions | N | R | Off | Max | N · R | Predicted win | Actual win | 95% CI |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 20 | 1,623 | 1.623 | 0.203 | 21.7% | 5.06 | 0.330 | 0.156 | 0.226 | [0.206, 0.246] |
| 19 | 147 | 0.147 | 0.231 | 21.1% | 6.37 | 0.034 | 0.143 | 0.177 | [0.124, 0.247] |
| 18 | 588 | 0.588 | 0.191 | 22.1% | 8.61 | 0.112 | 0.187 | 0.248 | [0.215, 0.285] |
| 17 | 1,464 | 1.464 | 0.137 | 20.1% | 10.29 | 0.201 | 0.161 | 0.223 | [0.203, 0.245] |
| 16 | 423 | 0.423 | 0.056 | 10.4% | 1.93 | 0.024 | 0.211 | 0.243 | [0.205, 0.287] |
| 15 | 655 | 0.655 | 0.112 | 15.7% | 9.35 | 0.073 | 0.205 | 0.221 | [0.191, 0.255] |
| 14 | 861 | 0.861 | 0.088 | 14.2% | 5.99 | 0.076 | 0.202 | 0.229 | [0.202, 0.258] |
| 13 | 407 | 0.407 | 0.110 | 12.3% | 5.81 | 0.045 | 0.263 | 0.258 | [0.218, 0.303] |
| 12 | 485 | 0.485 | 0.244 | 12.2% | 15.33 | 0.119 | 0.249 | 0.243 | [0.207, 0.283] |
| 11 | 436 | 0.436 | 0.125 | 11.7% | 7.00 | 0.055 | 0.248 | 0.243 | [0.205, 0.286] |
| 10 or fewer | 855 | 0.855 | 0.124 | 7.9% | 17.74 | 0.106 | 0.312 | 0.282 | |

By band; the standard errors are per round, and the error of the predicted
gain includes application D's 0.014 and the covariance of the two sums:

| Cards | Σ N · R | Standard error | Predicted gain | Standard error |
| --- | ---: | ---: | ---: | ---: |
| 11 to 14 | 0.293 | 0.043 | 0.132 (application D) | 0.014 |
| 15 to 20 | 0.773 | 0.047 | **0.348** | 0.067 |
| 15 to 17 | 0.298 | 0.026 | 0.134 | 0.027 |
| 18 to 20 | 0.476 | 0.037 | 0.214 | 0.043 |
| 10 or fewer | 0.106 | 0.031 | 0.048 | 0.016 |

- **The gate is passed:** 15 to 20 cards predict +0.35 ± 0.07 points a
  round, against a gate of 0.05.  15 to 17 cards alone are as large as
  application D; 18 to 20 cards are more, because 20-card decisions are the
  most numerous and `HeuristicBot` is not the best there one time in five.
- **The exchange rate checks out once.**  The same ρ gives +0.048 ± 0.016
  for 10 cards or fewer, and what was actually measured is `ExpectBot`
  against `HeuristicBot` at +0.076 ± 0.008 (`expectation.md` batch 3), 1.6
  standard errors apart.  The direction is an underestimate, so 0.35 is on
  the conservative side.
- **R has a heavy tail.**  Each row's maximum is 30 to 100 times its mean,
  and most of Σ N · R comes from a few large mistakes; the standard error of
  N · R is computed per round and already includes this.

Calibration, merged by band:

| Cards | Decisions | Predicted win | Actual win | Predicted / actual |
| --- | ---: | ---: | ---: | ---: |
| 10 or fewer | 855 | 0.312 | 0.282 | 1.11 |
| 11 to 14 | 2,189 | 0.233 | 0.240 | 0.97 |
| 15 to 17 | 2,542 | 0.181 | 0.226 | 0.80 |
| 18 to 20 | 2,358 | 0.163 | 0.228 | 0.71 |

- **The bias of long hands runs the other way: an underestimate.**  10 cards
  or fewer are overestimated by 11% and 15 or more underestimated by 24%,
  about twice the size and the opposite direction, which is not "off by far
  more", so the work goes on as declared; but this is a systematic bias, not
  noise.
- **18 cards or more are underestimated further** (0.71 against 0.80), and
  the graphs at 18 cards or more have their discards pruned: `Loose` makes
  the DP assume it will play worse later.  h × 1.5 was tuned on positions of
  10 cards or fewer, and applied to the 46 cards of a whole round it may
  fine too heavily; that is the other candidate.  Neither has been measured.
- Decisions of the same round are correlated, so the intervals are too
  narrow.
- What it means for the eval: the eval learns the DP's value, and learns
  this bias with it.  The leaves and the exact 0 of an exhausted round are
  in the same model, so what moves the claims is the eval's bias against the
  DP (batch 2's sensitivity), not the DP's against reality.  Application C
  (the hint showing the chance to win) has not passed calibration at 15
  cards or more, and the model must be fixed first.

## Batch 2: the framework

- **`Prune` gains a depth limit.**  The breadth-first build records how many
  claims each node is from a root, and those at the limit are not expanded.
- **`Graph::values` takes the leaves' curves.**  With no leaves it is
  bit-identical to what it is now.
- **Census.**  `examples/nodes.rs`, on decisions of 15 to 20 cards, builds
  once each for k = 0, 1, 2: nodes, leaves, and the milliseconds to build
  and to sweep.
- **Sensitivity.**  On decisions of 11 to 14 cards, the leaves at k = 0, 1,
  2 read the exact value plus an error, and the measure is how many points
  the chosen root gives up by the exact values.  Two kinds of error: noise
  independent at each leaf (standard deviation 0.25, 0.5, 1, 2 points), and
  a bias common to all leaves (±0.5, ±1 points).  A real eval's errors are
  correlated between similar hands, so the independent-noise column is the
  pessimistic one.

The output is two numbers that batch 3's gate needs: the k each platform can
afford, and how much error and bias the eval may have at that k while the
loss per decision stays within half of `HeuristicBot`'s R(n).

Gate: `ab/run.sh` reads 0 deals that part (no bot uses the depth limit).  If
depth 1 averages over 100 milliseconds at 20 cards, the estimate above is
wrong again and only k = 0 is usable; find out why before going on.

Batch 2's declaration (2026-11-19, written before the run, base `93b2d7b`):

- **Census:** `nodes 200 1795500000 20 0 k`, once each for k = 0, 1, 2, run
  in turn, single-threaded, with nothing else running.  As in `dp-nodes.md`
  it is every discard of four `HeuristicBot`s in self-play; all discards are
  tried (`Discards::All`).  The sweep reads 0 at the leaves, and the eval's
  time is not included.
- **Sensitivity:** `examples/leaves.rs`, `leaves 2000 1795600000`, 2,000
  rounds, the same table as batch 1 (seat 1 is the `ExpectBot` with a
  14-card gate).  The error is the same shift at every t ≥ 1 (a hand is
  misjudged, and misjudged consistently), and V̂(0) is still 0.  Two things
  were seen in the trial run and added to the design before the run:
  - **Independent noise is not pessimistic at k ≥ 1.**  A root has hundreds
    of leaves below it, and independent noise cancels in the weighted
    average; an error that misjudges similar hands together does not.  So
    the k ≥ 1 column is optimistic, and only the k = 0 column is the
    pessimistic one meant above.
  - **One more kind of error: scale.**  V̂ = λ V, λ = 0.8, 0.5, is what an
    underfitted eval shrinking towards 0 looks like; a common bias at k = 0
    only acts against pruned roots, and a scale is different.
- Trial run of 100 rounds (seed base 1, not counted): 228 decisions, 13
  seconds (10 threads); 9, 265 and 1,408 leaves a decision on average at
  k = 0, 1, 2.
- **Zero deals part:** `SEED_BASE=1795700000 ab/run.sh 20000`, control
  `93b2d7b`, after the two above have finished.

### Batch 2 results

**Census** (`nodes 200 1795500000 20 0 k`, 2,875 discards, native,
single-threaded).  Averages per decision; the time is the milliseconds to
build and sweep:

| Cards | Decisions | k = 0 leaves | ms | k = 1 leaves | ms | Most nodes | k = 2 leaves | ms | Most nodes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 200 | 13 | 0.42 | 1,350 | 29.1 | 3,633 | 27,031 | 509 | 113,859 |
| 19 | 51 | 13 | 0.22 | 1,209 | 17.2 | 2,237 | 20,968 | 313 | 54,409 |
| 18 | 205 | 12 | 0.23 | 951 | 14.9 | 2,141 | 13,345 | 218 | 45,694 |
| 17 | 529 | 11 | 0.18 | 758 | 10.7 | 1,940 | 9,420 | 144 | 36,169 |
| 16 | 155 | 11 | 0.16 | 665 | 8.3 | 1,701 | 6,980 | 96 | 28,493 |
| 15 | 255 | 10 | 0.15 | 509 | 6.5 | 1,371 | 4,399 | 64 | 20,550 |
| 14 | 385 | 9 | 0.11 | 374 | 4.3 | 1,066 | 2,621 | 36 | 11,058 |
| 11 | 244 | 7 | 0.07 | 161 | 1.6 | 536 | 572 | 7.7 | 3,787 |

- **The gate is passed:** depth 1 averages 29 milliseconds at 20 cards,
  against a gate of 100.  The estimate above, "1 is 10² to 10³ leaves, a few
  milliseconds", was right this time; so was 0.1 to 0.5 seconds for k = 2,
  with 20 cards at the upper edge.
- **Building the graph is 98%.**  The sweep is 0.2 milliseconds at k = 1 and
  12 milliseconds at k = 2; the time goes to the `reach` of each new node.
  With the eval estimated at a microsecond a leaf, k = 1 adds another 1.4
  milliseconds and k = 2 another 27 (20 cards).
- **Desktop:** k = 2 averages 144 milliseconds up to 17 cards, within the
  186 milliseconds of 14 cards; from 18 cards it is 218, 313 and 509
  milliseconds, over.  So the desktop has **k = 2 at 15 to 17 cards and
  k = 1 from 18**.
- **Web:** wasm is 1.7 times slower (`expectation.md` application C), so
  k = 1 averages about 50 milliseconds at 20 cards; the move with the most
  nodes comes to about 130 milliseconds by its node count, and p95 is below
  the slowest, so p95 is very likely within 0.1 seconds, to be measured in
  batch 4 as in application C.  The web has **k = 1 at 11 to 20 cards**.
- The census has discards only; a claim has a few more roots and a somewhat
  larger graph.

**Sensitivity** (`leaves 2000 1795600000`, 4,653 decisions, 245 seconds, 10
threads).  Points given up per decision by the exact values; "Off" is the
share chosen wrongly, and the standard error is per decision.  With every
error at 0 all three depths choose the best, which the program asserts:

| k | Independent noise | Common bias | Scale | 11 | 12 | 13 | 14 | All | Off | Standard error |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 0.25 | | | 0.018 | 0.018 | 0.017 | 0.024 | 0.020 | 12.8% | 0.001 |
| 0 | 0.5 | | | 0.039 | 0.041 | 0.049 | 0.055 | 0.048 | 18.1% | 0.002 |
| 0 | 1 | | | 0.084 | 0.098 | 0.110 | 0.119 | 0.105 | 23.1% | 0.005 |
| 0 | 2 | | | 0.206 | 0.256 | 0.258 | 0.262 | 0.249 | 28.7% | 0.010 |
| 0 | | −1 | | 0.006 | 0.014 | 0.007 | 0.013 | 0.011 | 2.0% | 0.001 |
| 0 | | −0.5 | | 0.002 | 0.003 | 0.002 | 0.003 | 0.003 | 0.9% | 0.000 |
| 0 | | +0.5, +1 | 0.8, 0.5 | 0 | 0 | 0 | 0 | 0 | 0% | 0 |
| 1 | 0.25 | | | 0.000 | 0.001 | 0.001 | 0.001 | 0.001 | 2.7% | 0.000 |
| 1 | 0.5 | | | 0.001 | 0.003 | 0.003 | 0.003 | 0.003 | 4.9% | 0.000 |
| 1 | 1 | | | 0.006 | 0.007 | 0.008 | 0.011 | 0.008 | 7.9% | 0.001 |
| 1 | 2 | | | 0.017 | 0.023 | 0.024 | 0.029 | 0.024 | 11.7% | 0.002 |
| 1 | | ±0.5, ±1 | | ≤ 0.001 | ≤ 0.001 | ≤ 0.001 | ≤ 0.001 | ≤ 0.001 | ≤ 1.0% | |
| 1 | | | 0.8 | 0.002 | 0.001 | 0.001 | 0.002 | 0.002 | 1.3% | 0.001 |
| 1 | | | 0.5 | 0.019 | 0.017 | 0.011 | 0.013 | 0.015 | 3.5% | 0.002 |
| 2 | 2 | | | 0.001 | 0.001 | 0.003 | 0.005 | 0.003 | 4.0% | 0.000 |
| 2 | | ±0.5, ±1 | | ≤ 0.001 | 0 | ≤ 0.001 | ≤ 0.001 | ≤ 0.001 | ≤ 0.5% | |
| 2 | | | 0.5 | 0.004 | 0.005 | 0.004 | 0.012 | 0.007 | 3.3% | 0.001 |

The smaller noises and the 0.8 scale at k = 2 are all around 0.001 and are
omitted.  The yardstick is half of `HeuristicBot`'s R(n) from batch 1:
0.063, 0.122, 0.055 and 0.044 at 11 to 14 cards.

- **The common bias has almost no effect, contrary to the worry of "Shape"
  above.**  From any root, while cards are plentiful almost every path
  claims once more and reaches a leaf, so a bias shifts all the roots alike.
  A choice goes wrong only at k = 0, with the bias low, when one root is a
  dead hand the ceiling pruned (whose value is exact); a high bias never
  changed a choice at k = 0.  "If the eval is b points high, the bot claims
  b points' worth more" does not hold, because both sides of eating or not
  eating run to leaves.
- **What k = 0 fears is independent noise.**  σ = 0.5 gives up 0.055 at 14
  cards, over half of R(14), 0.044; σ = 0.25 passes at every card count.  So
  k = 0 needs a leaf error σ **within 0.4 points** (interpolated).
- **k = 1 passes for every error tried:** at most 0.029 for σ = 2 (14 cards)
  and at most 0.019 for λ = 0.5.  Independent noise is optimistic here (see
  the declaration), but a scale is an error correlated throughout, and it
  passes too.
- **k = 2 is another three to eight times smaller than k = 1.**

The two numbers for batch 3:

| Platform | k | Leaf error σ | Common bias | Scale λ |
| --- | --- | --- | --- | --- |
| Web | 1 | within 2 points | within ±1 point | 0.5 or more |
| Desktop | 2 at 15 to 17 cards, 1 from 18 | as above | as above | as above |

So batch 3's bias gate is: the mean of V̂ − V by (cards, t) within ±1 point,
the slope of the regression of V̂ on V 0.5 or more, and the standard
deviation of the residual within 2 points.  The tolerance of k = 1 for σ
comes from independent noise cancelling, so the condition on the residual is
looser than the other two; batch 3's loss table measures R̂(n) directly, and
that governs.

- **Zero deals part:** `ab/run.sh 20000` against `93b2d7b` (seed base
  1795700000, `RESHUFFLE=8`) reads 0 deals that part.

## Batch 3: labels, pattern tables, the offline loss

- **Labels.**  Batch 1's example gains an output: the nine pattern indices,
  the exposed 胡, the count of `live`, and the value at each t.
- **Training.**  One example, hand-written gradient descent, no added
  dependency, a fixed seed.  The loss is the squared error at each t.  One
  training each for D = 1, 4, 16.
- **Inference.**  `src/eval.rs`: the pattern indices, the lookups, the
  curve.  The tables are compiled into the binary.
- **Loss table.**  On rounds the training has not seen, for each card count
  and each k: how many points the chosen root gives up by the exact values,
  R̂(n), and the mean of V̂ − V by (cards, t), which is the bias.

Gates, both to be passed, with the k batch 2 fixed for each platform:

- ρ · Σ N(n) · (R(n) − R̂(n)) is 0.05 points a deal or more.  n runs from
  11 to 20 on the web and from 15 to 20 on the desktop.
- The bias is inside the range batch 2's sensitivity sets.

Failing that, try these in order, each rerunning the same loss table: add a
hidden layer; give the accumulator `ceiling` minus the threshold as well
(already computed when the graph is built); give it the knapsack's "fewest
outside cards still short of the threshold" as well.  If all three fail,
stop, and the eval does not go into a bot.

Tests:

Batch 3's declaration (2026-11-19, written before the run, base `7f0d5f1`):
the three modes of `examples/eval.rs`, the same table as batch 1, with the
exact graph built for each of seat 1's decisions at 11 to 20 cards (`Loose`
from 18).

- **Labels:** `eval label 2000 1795800000`, 2,000 rounds.  Each graph gives
  all its roots, and at most 64 nodes each (sampled uniformly) of those one
  and two claims from a root; each node records the values for t = 1 to
  min(T, 46).  T is at most 45 at a table of four, so the row t = 46 never
  has a label and stays at its initial value; see "Two and three players".
- **Training:** `eval train labels eval 20 1 4 16`.  Adam, the learning rate
  3 × 10⁻³ falling linearly to a tenth, 20 epochs, 256 nodes a batch, the
  seed of the initial values 1795800000 + D; rounds whose number leaves 9
  when divided by 10 are held out for validation.  Of the three D, the table
  with the smallest validation RMSE is **the table judged**, fixed before
  the loss table.
- **Loss table:** `eval loss 500 1795900000 eval-1.bin eval-4.bin eval-16.bin`,
  500 rounds, all three tables reported, the gate reading only the table
  judged.  The shallow graph uses `Prune::default()` (all discards tried)
  plus the depth limit; `HeuristicBot`'s R is measured on the same
  decisions, and ρ uses the Σ N · R of 11 to 14 cards from the same run.
  The bias counts only leaves that are in the exact graph too; a cell needs
  at least 1,000 values to count towards "the worst cell".
- **Remedies:** on a failure, in the order above, each retrained on the same
  labels, with a new seed base for the loss table: 1796000000, 1796100000,
  1796200000.
- **Zero deals part:** `SEED_BASE=1796300000 ab/run.sh 20000`, control
  `7f0d5f1`, after all the above have finished.

Trial runs (seed bases 1 and 100, not counted): labelling takes 28 CPU
seconds a round and 3.4 GB at most; the loss table 47 CPU seconds a round; a
lookup 0.2 microseconds.  A table trained on 30 rounds of labels chose worse
than `HeuristicBot` on 20 rounds, and D = 16 overfitted (training RMSE 1.16,
validation 2.8), so the labels take 2,000 rounds.

- **Symmetry.**  After interchanging colours, 士 and 象, or 車, 馬 and 炮,
  the pattern indices do not change; under the same interchanges the root
  values of the exact DP do not change either.
- **Indices.**  The indices of each of the three tables fill 0 to the
  pattern count minus one, and no two patterns that are not symmetric
  collide.
- **Unchanged with no leaves.**  With the depth limit greater than the
  graph's depth, the values are bit-identical to those with no limit.
- **Legal.**  In the four-player game of `tests/round.rs`, one seat is given
  a bot that uses the shallow graph, and it plays through without an illegal
  move.  This one moves with the bot to batch 4: no bot uses the tables in
  batch 3.

### Batch 3 results

**Labels:** `eval label 2000 1795800000`: 14,397 graphs, 1,951,898 nodes,
66,895,056 values, 8,466 seconds (10 threads), a file of 326 MB.

**Training:** `eval train labels eval 20 1 4 16`, within 30 seconds for each
D.  On the 206,004 validation nodes, the RMSE of the mean value at each t is
3.37:

| D | Training RMSE | Validation RMSE | Roots | One claim | Two claims |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2.21 | 2.21 | 1.92 | 2.12 | 2.36 |
| 4 | 0.84 | 0.99 | 1.03 | 0.99 | 0.99 |
| 16 | 0.52 | **0.78** | 0.82 | 0.77 | 0.79 |

The table judged is D = 16, compiled into the binary (`src/eval-16.bin`,
f32, 359 KB, `Eval::default`).

**Loss table** (`eval loss 500 1795900000`, 3,664 decisions, 2,266 seconds,
10 threads; a lookup averages 0.26 microseconds over the three tables).  R
is `HeuristicBot`'s first choice and R̂ the shallow graph's choice, both in
points given up per decision by the exact values:

| Cards | Decisions | N | R | D = 16 k = 0 | k = 1 | k = 2 | D = 4 k = 1 | D = 1 k = 1 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 795 | 1.590 | 0.185 | 0.051 | 0.033 | 0.021 | 0.039 | 0.080 |
| 19 | 58 | 0.116 | 0.108 | 0.059 | 0.050 | 0.002 | 0.075 | 0.181 |
| 18 | 300 | 0.600 | 0.127 | 0.054 | 0.018 | 0.017 | 0.033 | 0.061 |
| 17 | 722 | 1.444 | 0.098 | 0.092 | 0.032 | 0.008 | 0.049 | 0.123 |
| 16 | 210 | 0.420 | 0.073 | 0.045 | 0.024 | 0.003 | 0.013 | 0.106 |
| 15 | 387 | 0.774 | 0.103 | 0.115 | 0.022 | 0.006 | 0.040 | 0.126 |
| 14 | 451 | 0.902 | 0.080 | 0.047 | 0.022 | 0.012 | 0.024 | 0.143 |
| 13 | 260 | 0.520 | 0.115 | 0.184 | 0.025 | 0.002 | 0.019 | 0.146 |
| 12 | 230 | 0.460 | 0.156 | 0.151 | 0.062 | 0.015 | 0.106 | 0.104 |
| 11 | 251 | 0.502 | 0.135 | 0.073 | 0.009 | 0.001 | 0.021 | 0.123 |

The Σ N · R of 11 to 14 cards is 0.271 ± 0.044 (0.293 in batch 1), and ρ
uses this run's.  Predicted gain, the error including application D's 0.014
and the covariance:

| D | Web: k = 1 at 11 to 20 cards | Desktop: k = 2 at 15 to 17 cards, k = 1 from 18 |
| ---: | ---: | ---: |
| 1 | 0.042 ± 0.047 | 0.147 ± 0.040 |
| 4 | 0.301 ± 0.053 | 0.245 ± 0.055 |
| 16 | **0.340 ± 0.058** | **0.267 ± 0.058** |

The leaves against the exact curves (leaves that are in the exact graph too,
one value per t):

| D | k | Values | V̂ − V | Slope | Residual | RMSE | Worst cell |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 0 | 1.5 × 10⁶ | −0.063 | 0.940 | 0.789 | 0.817 | −0.27 |
| 16 | 1 | 8.7 × 10⁷ | −0.025 | 0.965 | 0.652 | 0.661 | +0.22 |
| 16 | 2 | 1.0 × 10⁹ | −0.019 | 0.975 | 0.599 | 0.604 | −0.43 |
| 4 | 1 | 8.7 × 10⁷ | −0.055 | 0.954 | 0.836 | 0.850 | −0.65 |
| 1 | 1 | 8.7 × 10⁷ | −0.060 | 0.854 | 1.676 | 1.738 | −1.62 |

- **Both gates are passed, on both platforms.**  Web +0.34 and desktop
  +0.27, against a gate of 0.05; the bias, slope and residual at k = 1 are
  −0.03, 0.97 and 0.65, against a range of ±1, 0.5 and 2.  None of the three
  remedies need be tried.
- **The shallow graph takes back most of the prize.**  The web's R̂ is under
  half of R at every card count; added up over 11 to 20 cards, Σ N · R̂ is
  0.21 and Σ N · R is 0.91; the desktop's k = 2 leaves only 0.003 to 0.008
  at 15 to 17 cards.  The web's +0.34 is larger than the desktop's because
  11 to 14 cards count in it too.
- **k = 0 is not enough.**  Its residual is 0.79, and batch 2 says k = 0
  needs within 0.4; its R̂ is larger than R at 13 and 15 cards.  The web uses
  k = 1, as batch 2 says.
- **Nine added vectors can learn the threshold, with D at 4 or more.**
  D = 1 takes back only a little at k = 1 (+0.04 ± 0.05, insufficient data).
- **The cells of the bias are almost all within ±0.1.**  The exceptions are
  0 to 1 cards at k = 2 (−0.4 to −12), with fewer than 1,000 values, which
  do not count; they are hands that have eaten down to a few cards and wait
  on one card, and are scarce in the training data too.
- The predicted gain comes from the same ρ, and inherits batch 1's
  calibration and the approximation of the exchange rate; the real answer is
  batch 4's A/B.
- **Zero deals part:** `ab/run.sh 20000` against `7f0d5f1` (seed base
  1796300000, `RESHUFFLE=8`) reads 0 deals that part.

## Batch 4: A/B

By the iron rules of `CLAUDE.md`, two independent experiments, each with its
deal count and seed base written into this document before the run:

| | Candidate | Control |
| --- | --- | --- |
| Web | 10-card gate, a graph k deep beyond it | `ExpectBot`, 10-card gate |
| Desktop | 14-card gate, a graph k deep beyond it | `ExpectBot`, 14-card gate |

The other three seats are all `HeuristicBot`.  With a decision in the order
of milliseconds, `RESHUFFLE=8` is used.

The timing conditions are those of `expectation.md` applications C and D:
the web's wasm p95 within 0.1 seconds; the desktop's mean no more than the
present 186 milliseconds of 14 cards.

How to read it:

- **win:** commit.  The hint marks points beyond the gate too, no longer a
  grade without a unit.
- **loss:** first trace the deals that moved the most, then compare with
  batch 3's bias.  If the bias is accurate and it still loses, the problem
  is the single-player model's approximation on long hands; go back to batch
  1's calibration.
- **insufficient data:** add deals.

Code: `ExpectBot::shallow(eval, deep)`: beyond the gate a graph two deep up
to `deep` cards and one deep past that, the leaves reading
`Eval::default()`; cards are counted the way the gate counts them.  The web
has `deep` = 0 (one deep at all of 11 to 20 cards), the desktop 17.  `ab/`
gains two switches: `SHALLOW` is the candidate's `deep`, and `BASELINE_GATE`
the gate of the `ExpectBot` on the control's side.

Timing (2026-11-19, `examples/expect_time.rs` with one more argument,
`deep`; seed base 1000, in milliseconds):

| Cards | Desktop: `expect_time 300 1000 14 17`, native | | | wasm: `expect_time 2000 1000 10 0` | | |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| | Decisions | Mean | p95 | Decisions | Mean | p95 |
| All | 2,442 | 62.1 | 230 | 16,189 | 18.0 | 54.6 |
| 14 | 242 | 180 | 581 | 1,700 | 8.1 | 16.9 |
| 15 | 245 | 50.9 | 137 | 1,528 | 10.2 | 21.1 |
| 16 | 120 | 68.8 | 209 | 805 | 13.5 | 27.9 |
| 17 | 405 | 127 | 302 | 2,639 | 19.1 | 39.4 |
| 18 | 216 | 16.8 | 42.7 | 1,404 | 24.4 | 54.8 |
| 19 | 31 | 17.8 | 38.8 | 230 | 29.3 | 56.0 |
| 20 | 461 | 27.9 | 57.2 | 3,031 | 44.8 | 101 |

- **The desktop passes:** the mean at each card count from 15 to 20 is
  within the 180 milliseconds of 14 cards, the most being 127 at 17 cards;
  the slowest move is 853 milliseconds, against 1.4 seconds inside the gate.
- **The web passes, but narrowly:** p95 over all decisions is 55
  milliseconds, within 0.1 seconds; p95 at 20 cards alone is 101
  milliseconds, and the slowest move is 1.1 seconds (the slowest
  `HeuristicBot` hint is 255 milliseconds).

Batch 4's declaration (2026-11-19, written before the run, base `5c3cc24`,
which is the control too):

- **Web:** `EXPECT=1.5 GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1
  SEED_BASE=1795055385 ab/run.sh 30000`, 30,000 deals, `RESHUFFLE=8`.
- **Desktop:** `EXPECT=1.5 GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1
  BASELINE_GATE=14 SEED_BASE=1795155385 ab/run.sh 20000`, 20,000 deals,
  `RESHUFFLE=8`, after the web's run has finished.
- Trial run of 400 deals (seed base 1, not counted): web 1 minute, 83% part,
  standard error 0.363; desktop 6.8 minutes, 76% part, standard error 0.331.
  So the web's 30,000 deals take about 80 minutes with a standard error of
  about 0.042, and the desktop's 20,000 deals about 5.7 hours and about
  0.047; the +0.34 and +0.27 batch 3 predicts are seven and six standard
  errors.

### Batch 4 results

| | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | --- | ---: | --- | --- |
| Web | 30,000 | 25,146 | 24.44% → 25.09% | +0.191 ± 0.046 | [+0.100, +0.281] | win |
| Desktop | 20,000 | 15,243 | 24.57% → 25.23% | +0.204 ± 0.052 | [+0.102, +0.306] | win |

- **Both are wins.**  The web's hint now uses a graph one deep beyond the
  gate; the desktop's opponents and hint use two deep at 15 to 17 cards and
  one deep from 18.  The hint marks points at every move.
- **Less was gained than predicted.**  Batch 3 predicted web +0.34 ± 0.06
  and desktop +0.27 ± 0.06; the measurements are 2.0 and 0.9 combined
  standard errors lower.  The prediction goes by the DP's values, and batch
  1's calibration says the DP underestimates the win rate of long hands by
  24%, so part of the prize counted by it is not real.
- The web's and the desktop's gains are each relative to its own control
  (gates of 10 and 14 cards), and the two cannot be added.

## Two and three players: where the tables do not reach

Batches 1 to 4 are all at a table of four, which starts with 25 draws, and
the graph sweeps 45 public cards.  A table of two starts with 65 draws (174
cards) and a table of three with 45 (113 cards); `5c3cc24` opened these two
tables, and the bots were not measured again.

**Symptom (2026-11-19):** at the start of a round at a table of two, the
shallow graph would rather break a concealed triplet than discard a lone
card, the opposite of both `HeuristicBot` and the exact DP.  The same hand
of 21 cards with only the draws changed, a graph one deep:

| draws | Cards swept | First choice | Its value | Rank of red 炮 (the concealed triplet) |
| ---: | ---: | --- | ---: | ---: |
| 25 | 45 | white 象 | −5.60 | 15 |
| 26 | 48 | white 象 | −3.65 | 11 |
| 27 | 51 | white 象 | −2.45 | 7 |
| 28 | 55 | red 炮 | −1.70 | 1 |
| 30 to 65 | 61 to 174 | red 炮 | −1.27 to −1.17 | 1 |

**Cause:** `eval::CARDS` is 46, and the labels only go to t = 45.  The
weights of the row t = 46 are the initial values (|w| = 0.23, about 11 in
the other rows; bias 0), so every leaf's value at t = 46 is about 0, where
at t = 45 it is about −5 to −7.  Sweeping more than 46 cards, `Graph::sweep`
goes on using the last cell of the curve, and the more ways a hand has to
claim, the better it looks.  It is read from 26 draws (a pile of 32 cards),
and this hand flips at 28 draws.

**Two fixes tried that were not enough** (a table of two, trial runs of 400
deals each, seed base 1, not counted; the control is the `ExpectBot` that
uses `HeuristicBot` beyond the gate):

- Cutting t = 46 and holding at t = 45: −2.83 ± 0.31.  The leaves stop at
  the 45-card value while the roots' clock runs on, so every claim looks
  like a loss and the bot passes them all.
- Sweeping only 46 cards, as if that were all that is left: −2.11 ± 0.29
  (−0.24 ± 0.31 at a table of three).  The clocks agree, but `Outlook` and
  the labels are both a table of four's.

**The fix:** cut the row never learnt (`CARDS` = 45), and play as
`HeuristicBot` beyond the gate when the cards swept exceed `CARDS`.  A table
of four never exceeds it; at a table of two with 25 draws left the hand is
almost always inside the gate already (a trial run of 1,000 deals against
the control above has 0 deals that part, and 2 at a table of three).  The DP
inside the gate does not lose to `HeuristicBot` at tables of two and three
(trial runs of 1,000 deals: +0.07 ± 0.04, +0.16 ± 0.07) and is left alone.

`ab/` gains two switches: `PLAYERS` is the size of the table, and
`BASELINE_SHALLOW` the `deep` of the control's side.

Declaration (2026-11-19, written before the run; control `3ddb8f3`,
`RESHUFFLE=8`, the other seats all `HeuristicBot`).  The web is `EXPECT=1.5
GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1 BASELINE_SHALLOW=0`, the desktop
`EXPECT=1.5 GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1 BASELINE_GATE=14
BASELINE_SHALLOW=17`:

| | `PLAYERS` | Deals | seed base | Expected |
| --- | ---: | ---: | ---: | --- |
| Web | 4 | 20,000 | 1795094000 | 0 deals part |
| Desktop | 4 | 2,000 | 1795114000 | 0 deals part |
| Web | 2 | 2,000 | 1795124000 | win |
| Web | 3 | 2,000 | 1795134000 | win |
| Desktop | 2 | 500 | 1795144000 | win |
| Desktop | 3 | 500 | 1795154000 | win |

Trial runs (seed base 1, not counted): the version with t = 46 cut, against
`3ddb8f3`, is +8.38 ± 0.32 over 400 deals at a table of two and +9.85 ± 0.48
at a table of three; so the standard error of 2,000 deals is about 0.15 to
0.2, and of the desktop's 500 deals about 0.3.  The six runs take about 90
minutes in all.

### Results

| | `PLAYERS` | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | ---: | --- | --- |
| Web | 4 | 20,000 | 0 | 24.99% → 24.99% | 0 | | zero deals part |
| Desktop | 4 | 2,000 | 0 | 25.95% → 25.95% | 0 | | zero deals part |
| Web | 2 | 2,000 | 1,960 | 3.95% → 51.10% | +11.342 ± 0.157 | [+11.035, +11.649] | win |
| Web | 3 | 2,000 | 1,947 | 2.98% → 33.52% | +11.099 ± 0.230 | [+10.648, +11.550] | win |
| Desktop | 2 | 500 | 488 | 9.12% → 51.95% | +10.367 ± 0.296 | [+9.786, +10.948] | win |
| Desktop | 3 | 500 | 485 | 4.95% → 34.00% | +10.554 ± 0.431 | [+9.709, +11.398] | win |

- **All six are as expected.**  Play at a table of four has not changed; the
  control at tables of two and three hardly ever wins (3% to 9%), and the
  candidate returns to the win rate that table should have (a half at a
  table of two, a third at a table of three).
- **The hint at tables of two and three is `HeuristicBot`'s grade, not
  points, while the pile is still long.**  Points need that table's
  `Outlook` (`examples/outlook.rs`) and labels first, and the tables
  lengthened to 174 cards; the two fixes above say that forcing it through
  the four-player tables loses.  Not done yet.
- **Added 2026-11-21: there is an `Outlook`, and one more condition.**  The
  tables for two and three players have been measured
  (`research/outlook.md`, "Tables of two and three").  By them a round is 81
  public cards (two players) and 64 (three players), not the 174 worked out
  from the four-player table, so these are the two numbers a small table's
  tables must be lengthened to.  A small table's table has fewer public
  cards for each card drawn, and looking only at "the cards swept do not
  exceed `CARDS`" would have the shallow graph use the four-player `Eval`
  with 41 draws still left, measured to lose 5.8 points a split; so
  `ExpectBot::depth` also requires that the draws left do not exceed
  `eval::DRAWS` (25).  With `Outlook::default` the two conditions are the
  same thing.
- **Added again 2026-11-21: the pile of a small table is cut to 25 cards,
  and what this section describes no longer happens.**  The dead cards are
  now 6 plus the 20 of each empty seat (`Rules::dead`, `variants.md`, "The
  dead cards of a small table"), so tables of two and three start with 25
  draws as well, the cards swept do not exceed `CARDS`, and the shallow
  graph is usable from the first move.  The two conditions of `depth` stay,
  and what they stop is rules that set `dead_tail` short; the numbers above
  were all measured before the change.  The leaf values are still the
  four-player labels, and the A/B of this play at a small table is in
  `research/outlook.md`, "Empty seats' hands are dead".

## Tables for two and three players

2026-11-21, base `07d5602` (the revision with the rule change, not on a
branch).  The same thing as D of "Applications", with the size of the table
as its object.

**Why.**  Since the dead cards changed, a small table uses the shallow graph
from the first move, and the leaves are the four-player `Eval`.  The A/Bs
against "playing as `HeuristicBot` beyond the gate" (M and N of
`research/outlook.md`, "Empty seats' hands are dead", web settings): a table
of three +0.216 ± 0.051, a win; a table of two −0.184 ± 0.045, a loss.
Cutting the pile to 25 cards only aligned how many cards a round draws; each
player still gets more of the draws than at a table of four (twice as many
at a table of two), and the cards that can be eaten go from 0.28 to 0.42 and
0.74; the leaf values are labelled under the four-player `Outlook`, which at
a small table is the wrong table.

Declaration (2026-11-21, written before the run).  The method is batch 3's,
with only the table and the `Outlook` changed: `examples/eval.rs` takes
`PLAYERS` and `EXPECT`, seat 1 plays by that table's `Outlook::new` times
`EXPECT`, and the graph computes its values by it too; `ab/` takes `EVAL`,
and the candidate's leaves read the table in the file.

- **Labels:** `PLAYERS=2 EXPECT=1.3 eval label 2000 1799800000` and
  `PLAYERS=3 EXPECT=1.4 eval label 2000 1799900000`, 2,000 rounds each.  The
  multipliers are those `research/outlook.md` calibrated under the new
  rules.  By that table's own table, the public cards of a round are 36.8 at
  a table of two and 41.6 at a table of three (45.9 at a table of four), all
  within `CARDS` (45); the rows beyond have no labels, and the graph does
  not sweep that far either.
- **Training:** `eval train labels eval 20 1 4 16`, the settings of batch 3.
  Of the three D, the table with the smallest validation RMSE is the table
  judged.
- **Loss table:** `eval loss 300 1800000000` (two players) and `1800100000`
  (three players), with the same `PLAYERS` and `EXPECT`, 300 rounds each
  (500 rounds in batch 3; the time is kept for the A/B).  A report only,
  with no gate: the verdict is the A/B's.
- **A/B:** control `07d5602`, `RESHUFFLE=8`, 十胡, the other seats all
  `HeuristicBot`.  The candidate is that table's `Outlook` times its
  multiplier, that table's `Eval`, and the shallow graph on as the setting
  has it.  The control is the play that table should now be compared with:
  at a table of two the `ExpectBot` that plays as `HeuristicBot` beyond the
  gate (M says it is better than the four-player `Eval`), at a table of
  three the four-player `Eval` (N says it is the better).  The desktop
  settings have no numbers like M and N, and their control is the state of
  things after the rule change, that is the four-player `Eval`.

  | | Players | Setting | Candidate | Control | Deals | seed base |
  | --- | ---: | --- | --- | --- | ---: | ---: |
  | W | 2 | Web | `EXPECT=1.3 GATE_CARDS=10 SHALLOW=0 EVAL=…` | `BASELINE_EXPECT=1` | 10,000 | 1800200000 |
  | X | 3 | Web | `EXPECT=1.4 GATE_CARDS=10 SHALLOW=0 EVAL=…` | `BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 40,000 | 1800300000 |
  | Y | 2 | Desktop | `EXPECT=1.3 GATE_CARDS=14 SHALLOW=17 EVAL=…` | `BASELINE_EXPECT=1 BASELINE_GATE=14 BASELINE_SHALLOW=17` | 4,000 | 1800400000 |
  | Z | 3 | Desktop | `EXPECT=1.4 GATE_CARDS=14 SHALLOW=17 EVAL=…` | as Y | 6,000 | 1800500000 |

- **Each verdict is its own.**  win: that table at that setting uses that
  table's `Outlook` and `Eval`, and the table is compiled into the binary.
  loss: first trace the deals that moved the most.  Straddling 0:
  insufficient data, and that setting stays with the control's play; the web
  setting at a table of two is then a fall back to `HeuristicBot` beyond the
  gate, which needs one more condition and a run with zero deals that part.
- **The deal counts** follow the trial runs of `research/outlook.md`: the
  standard error is about 0.043 for W, about 0.017 for X, about 0.043 for Y
  and about 0.047 for Z.  X compares two `Eval`s, whose difference may be
  small, hence its larger count.
- **Time:** a trial run of the labelling over 40 rounds (seed base 1, not
  counted) took 38 CPU seconds a round and 2.8 GB at most at a table of two,
  46 CPU seconds and 3.5 GB at a table of three, so the two labelling runs
  take about 4.7 hours; the A/Bs about 3.7 hours.
- **The size of the tables.**  One D = 16 table is 359 KB, and the web's
  wasm is now 620 KB.  Compiling both in adds 720 KB; whether to quantise is
  decided after a win.

### Results at two players

**Labels:** 16,752 graphs, 2,293,823 nodes, 64,923,580 values, 11,109
seconds (the trial run estimated 2.1 hours; it took 3.1 hours), a file of
329 MB.

**Training:** within a minute for each D.  On the 235,344 validation nodes,
the RMSE of the mean value at each t is 2.17 (3.37 at a table of four):

| D | Training RMSE | Validation RMSE | Roots | One claim | Two claims |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.96 | 1.05 | 1.03 | 1.05 | 1.06 |
| 4 | 0.49 | 0.65 | 0.75 | 0.66 | 0.61 |
| 16 | 0.28 | **0.53** | 0.68 | 0.55 | 0.47 |

The table judged is D = 16 (`target/eval-2-16.bin`).

**Loss table** (`PLAYERS=2 EXPECT=1.3 eval loss 300 1800000000`, 2,614
decisions, 3,329 seconds).  The columns are those of batch 3; "Four" is the
four-player table (`src/eval-16.bin`), and the rest are the two-player
tables:

| Cards | Decisions | N | R | Four k = 1 | Four k = 2 | D = 16 k = 0 | k = 1 | k = 2 | D = 4 k = 1 | D = 1 k = 1 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 566 | 1.887 | 0.096 | 0.035 | 0.035 | 0.019 | 0.014 | 0.016 | 0.018 | 0.042 |
| 19 | 44 | 0.147 | 0.079 | 0.032 | 0.042 | 0.021 | 0.008 | 0.006 | 0.007 | 0.014 |
| 18 | 242 | 0.807 | 0.073 | 0.051 | 0.055 | 0.016 | 0.011 | 0.017 | 0.023 | 0.045 |
| 17 | 428 | 1.427 | 0.070 | 0.044 | 0.048 | 0.023 | 0.011 | 0.006 | 0.031 | 0.042 |
| 16 | 165 | 0.550 | 0.101 | 0.161 | 0.057 | 0.085 | 0.063 | 0.005 | 0.078 | 0.100 |
| 15 | 236 | 0.787 | 0.048 | 0.064 | 0.088 | 0.073 | 0.025 | 0.009 | 0.046 | 0.077 |
| 14 | 337 | 1.123 | 0.086 | 0.176 | 0.099 | 0.030 | 0.027 | 0.007 | 0.038 | 0.090 |
| 13 | 174 | 0.580 | 0.065 | 0.259 | 0.109 | 0.084 | 0.022 | 0.006 | 0.053 | 0.090 |
| 12 | 195 | 0.650 | 0.115 | 0.172 | 0.090 | 0.085 | 0.019 | 0.003 | 0.008 | 0.089 |
| 11 | 227 | 0.757 | 0.104 | 0.092 | 0.033 | 0.061 | 0.013 | 0.001 | 0.017 | 0.047 |

Predicted gain (the Σ N · R of 11 to 14 cards is 0.287 ± 0.047, converted to
0.132 ± 0.014 points):

| Table | Web: k = 1 at 11 to 20 cards | Desktop: k = 2 at 15 to 17 cards, k = 1 from 18 |
| --- | ---: | ---: |
| Four players, D = 16 | −0.051 ± 0.062 | 0.075 ± 0.028 |
| Two players, D = 1 | 0.087 ± 0.032 | 0.105 ± 0.027 |
| Two players, D = 4 | 0.214 ± 0.035 | 0.161 ± 0.035 |
| Two players, D = 16 | **0.257 ± 0.040** | **0.179 ± 0.038** |

The leaves against the exact curves:

| Table | k | Values | V̂ − V | Slope | Residual | RMSE | Worst cell |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Four players, D = 16 | 1 | 5.3 × 10⁷ | −3.060 | 1.164 | 1.141 | 3.287 | −4.51 |
| Two players, D = 16 | 0 | 9.5 × 10⁵ | −0.036 | 0.987 | 0.397 | 0.400 | −0.22 |
| Two players, D = 16 | 1 | 5.3 × 10⁷ | −0.017 | 0.991 | 0.326 | 0.327 | −0.11 |
| Two players, D = 16 | 2 | 6.1 × 10⁸ | −0.004 | 0.990 | 0.297 | 0.298 | −0.09 |
| Two players, D = 4 | 1 | 5.3 × 10⁷ | −0.006 | 0.980 | 0.486 | 0.488 | +0.22 |
| Two players, D = 1 | 1 | 5.3 × 10⁷ | −0.020 | 0.955 | 0.803 | 0.809 | −0.89 |

- **The four-player table underestimates every leaf by 3 points at a table
  of two.**  The bias is −3.06, and the more public cards the lower (−2.0 at
  1 to 5 cards, −4.5 at 31 cards or more).  The loss table predicts
  −0.05 ± 0.06 for it at the web settings, and M measured −0.184 ± 0.045.
  An explanation that fits but has not been verified separately: the wins
  inside the graph are exact values and not underestimated, while the leaves
  are all 3 points low, so the shallow graph leans towards the branch that
  can win right now.
- **The two-player table has no such bias:** at k = 1 it is −0.02, slope
  0.99, residual 0.33, more accurate than the four-player table at a table
  of four (−0.03, 0.97, 0.65).

**A/B W** (web settings, against `07d5602` playing as `HeuristicBot` beyond
the gate):

| | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | --- | ---: | --- | --- |
| W | 10,000 | 8,541 | 48.26% → 50.28% | +0.375 ± 0.042 | [+0.293, +0.457] | win |

seed base 1800200000, 2,805 seconds.  The loss table predicted
+0.257 ± 0.040.

**A/B Y** (desktop settings, against the four-player `Outlook` and `Eval` of
`07d5602`):

| | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | --- | ---: | --- | --- |
| Y | 4,000 | 2,910 | 50.29% → 51.55% | +0.320 ± 0.056 | [+0.210, +0.431] | win |

seed base 1800400000, 7,766 seconds (run alongside the three-player
labelling).

**Both settings are wins at a table of two**: a table of two uses its own
`Outlook` (times 1.3) and its own `Eval`.

### Results at three players

**Labels:** 15,456 graphs, 2,083,337 nodes, 65,506,421 values, 14,502
seconds (the last three hours alongside the two-player loss table and A/Bs),
a file of 325 MB.

**Training:** on the 199,336 validation nodes, the RMSE of the mean value at
each t is 2.90:

| D | Training RMSE | Validation RMSE | Roots | One claim | Two claims |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1.58 | 1.70 | 1.52 | 1.65 | 1.77 |
| 4 | 0.77 | 0.95 | 0.93 | 0.96 | 0.96 |
| 16 | 0.41 | **0.80** | 0.72 | 0.81 | 0.80 |

The table judged is D = 16 (`target/eval-3-16.bin`).

**Loss table** (`PLAYERS=3 EXPECT=1.4 eval loss 300 1800100000`, 2,293
decisions, 1,555 seconds):

| Cards | Decisions | N | R | Four k = 1 | Four k = 2 | D = 16 k = 0 | k = 1 | k = 2 | D = 4 k = 1 | D = 1 k = 1 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 498 | 1.660 | 0.170 | 0.025 | 0.021 | 0.031 | 0.012 | 0.012 | 0.023 | 0.067 |
| 19 | 54 | 0.180 | 0.050 | 0.028 | 0.009 | 0.097 | 0.003 | 0.005 | 0.016 | 0.114 |
| 18 | 228 | 0.760 | 0.125 | 0.032 | 0.020 | 0.046 | 0.024 | 0.016 | 0.036 | 0.045 |
| 17 | 385 | 1.283 | 0.118 | 0.023 | 0.016 | 0.031 | 0.011 | 0.008 | 0.015 | 0.052 |
| 16 | 168 | 0.560 | 0.088 | 0.008 | 0.003 | 0.068 | 0.004 | 0.010 | 0.038 | 0.078 |
| 15 | 230 | 0.767 | 0.131 | 0.063 | 0.048 | 0.113 | 0.034 | 0.027 | 0.057 | 0.130 |
| 14 | 265 | 0.883 | 0.133 | 0.060 | 0.003 | 0.026 | 0.030 | 0.005 | 0.054 | 0.109 |
| 13 | 160 | 0.533 | 0.083 | 0.015 | 0.001 | 0.099 | 0.034 | 0.001 | 0.039 | 0.074 |
| 12 | 154 | 0.513 | 0.078 | 0.074 | 0.008 | 0.051 | 0.012 | 0.003 | 0.039 | 0.079 |
| 11 | 151 | 0.503 | 0.133 | 0.023 | 0.001 | 0.069 | 0.005 | 0.003 | 0.008 | 0.090 |

Predicted gain (the Σ N · R of 11 to 14 cards is 0.268 ± 0.055):

| Table | Web: k = 1 at 11 to 20 cards | Desktop: k = 2 at 15 to 17 cards, k = 1 from 18 |
| --- | ---: | ---: |
| Four players, D = 16 | 0.341 ± 0.068 | 0.275 ± 0.071 |
| Three players, D = 1 | 0.176 ± 0.047 | 0.181 ± 0.054 |
| Three players, D = 4 | 0.350 ± 0.069 | 0.279 ± 0.072 |
| Three players, D = 16 | **0.404 ± 0.078** | **0.301 ± 0.076** |

The leaves against the exact curves:

| Table | k | Values | V̂ − V | Slope | Residual | RMSE | Worst cell |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Four players, D = 16 | 1 | 5.3 × 10⁷ | −0.960 | 1.019 | 0.771 | 1.233 | −1.62 |
| Three players, D = 16 | 0 | 9.3 × 10⁵ | −0.018 | 0.969 | 0.614 | 0.622 | −0.21 |
| Three players, D = 16 | 1 | 5.3 × 10⁷ | −0.003 | 0.983 | 0.492 | 0.495 | +0.17 |
| Three players, D = 16 | 2 | 6.0 × 10⁸ | −0.000 | 0.986 | 0.436 | 0.438 | −0.20 |
| Three players, D = 4 | 1 | 5.3 × 10⁷ | −0.023 | 0.966 | 0.725 | 0.732 | +0.19 |
| Three players, D = 1 | 1 | 5.3 × 10⁷ | −0.065 | 0.905 | 1.313 | 1.343 | −0.93 |

The four-player table underestimates by 1 point at a table of three (3
points at a table of two), with a slope of 1.02; its loss in choosing cards
is about the same as the three-player table's own (predicted gains of 0.34
against 0.40, error 0.07).

**A/B X, Z** (against the four-player `Outlook` and `Eval` of `07d5602`):

| | Setting | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | --- | ---: | ---: | --- | ---: | --- | --- |
| X | Web | 40,000 | 26,168 | 33.22% → 33.13% | −0.021 ± 0.025 | [−0.070, +0.028] | insufficient data |
| Z | Desktop | 6,000 | 3,525 | 34.46% → 34.22% | −0.095 ± 0.061 | [−0.215, +0.025] | insufficient data |

seed bases 1800300000 (6,138 seconds) and 1800500000 (5,177 seconds).

**Both settings straddle 0 at a table of three, and play is left alone**: a
table of three goes on using the four-player `Outlook` (times 1.5) and the
four-player `Eval`, and the three-player table is not compiled into the
binary.  This is not "as good": X's interval admits −0.07 to +0.03, and Z's
−0.22 to +0.03.  The candidate changed two things at once (`Outlook` and
`Eval`), and these two runs cannot tell which of them failed to help.

### 八胡 at two players

What won at a table of two is 十胡.  `Outlook::new` does not look at the
threshold at a table of two, and the two-player `Eval` is labelled at 十胡
as well; 八胡 at a table of two now (`07d5602`) uses the four-player ten-胡
`Outlook` and `Eval`.  Whether to change 八胡 along with it cannot rest on
inference alone.

Declaration (2026-11-22, written before the run; control `07d5602`,
`RESHUFFLE=8`, `PLAYERS=2 THRESHOLD=8`, the other seat `HeuristicBot`).  The
candidate is the two-player ten-胡 `Outlook` times 1.3 and the two-player
`Eval`; the control is the present play:

| | Setting | Candidate | Control | Deals | seed base |
| --- | --- | --- | --- | ---: | ---: |
| W8 | Web | `EXPECT=1.3 GATE_CARDS=10 SHALLOW=0 EVAL=…` | `BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 10,000 | 1800600000 |
| Y8 | Desktop | `EXPECT=1.3 GATE_CARDS=14 SHALLOW=17 EVAL=…` | `BASELINE_EXPECT=1 BASELINE_GATE=14 BASELINE_SHALLOW=17` | 4,000 | 1800700000 |

Verdict: if both runs are wins, a table of two uses its own tables whatever
the threshold.  Otherwise 八胡 at a table of two stays with the present play
(`ExpectBot::new` and the place that picks the table look at the threshold
as well), and a loss first has the deals that moved the most traced.

| | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | --- | ---: | --- | --- |
| W8 | 10,000 | 7,776 | 48.15% → 49.19% | +0.302 ± 0.040 | [+0.224, +0.380] | win |
| Y8 | 4,000 | 2,849 | 50.47% → 50.34% | +0.048 ± 0.056 | [−0.061, +0.158] | insufficient data |

1,951 seconds and 4,893 seconds.  **Not both runs are wins, so 八胡 at a
table of two keeps the four-player ten-胡 tables as before.**  The web
setting alone is a win and goes unused: the declaration tied the two
settings together, and the rule for the verdict is not changed once the data
are in.  To use it, `bot()` in `web/src/lib.rs` would have to pick the table
by its own setting, with a separate declaration and a separate measurement.

### Wired into the code

- **`Eval::of(&Rules)`**: two players at ten 胡 is `src/eval-16-2.bin`
  (which is `target/eval-2-16.bin`, 359 KB), and the rest is
  `Eval::default`.  The three-player table is not compiled into the binary.
- **`ExpectBot::new`**: two players at ten 胡 use `Outlook::new` times 1.3,
  and the rest is `Outlook::default` times `END_SCALE` as before.
- **`bot()` in `web/src/lib.rs`** uses `Eval::of(rules)`, shared by the web
  page and the desktop app.  The wasm goes from 620 KB to 982 KB (782 KB
  gzipped).
- **Rows 37 to 45 of the two-player table have no labels** (their biases are
  still the initial 0): the two-player `Outlook` counts only up to 36 public
  cards a round, and `ExpectBot::new` never reads those rows.  Pairing this
  table with another `Outlook` would read them.
- **`EXPECT=new` in `ab/`** is the shipped bot: the `Outlook` of
  `ExpectBot::new` and the table of `Eval::of`.

**Zero deals part** (control `07d5602`, `RESHUFFLE=8`, candidate
`EXPECT=new`, control `BASELINE_EXPECT=1` with the same gate and
`BASELINE_SHALLOW`).  Play at the other tables has not moved:

| Setting | Table | Deals | seed base | Part |
| --- | --- | ---: | ---: | ---: |
| Web | four players, ten 胡 | 2,000 | 1800800000 | 0 |
| Web | three players, ten 胡 | 2,000 | 1800900000 | 0 |
| Web | two players, 八胡 | 2,000 | 1801000000 | 0 |
| Web | four players, 八胡 | 2,000 | 1801100000 | 0 |
| Web | three players, 八胡 | 2,000 | 1801200000 | 0 |
| Desktop | four players, ten 胡 | 500 | 1801300000 | 0 |
| Desktop | three players, ten 胡 | 500 | 1801400000 | 0 |
| Desktop | two players, 八胡 | 500 | 1801500000 | 0 |

What is wired in for two players at ten 胡 is the very bot W measured: W's
first 1,000 deals (seed base 1800200000) were run once with `EXPECT=new` and
once with `EXPECT=1.3 EVAL=target/eval-2-16.bin`, and the output is the same
word for word (836 deals part, +0.232 ± 0.133).  Only tests and comments
have been touched since.

## 八胡: the ten-胡 tables beyond the gate

"Shape" says that until 八胡 has tables of its own, it is `HeuristicBot`
beyond the gate as before.  The code is not so: `bot()` in `web/src/lib.rs`
does not look at the threshold, and `ExpectBot::depth` only stops on the
count of public cards, so since batch 4 a table of four at 八胡 plays the
shallow graph on the ten-胡 tables beyond the gate.  This play has not been
measured.  The wins inside the graph and the ceiling pruning go by the
threshold of `Rules` and are right; the leaf values are 十胡's, and a hand 2
胡 short of winning is taken for one 4 胡 short.

`ab/` gains one switch: `THRESHOLD` is the threshold on both sides, 10 by
default.

Declaration (2026-11-20, written before the run; control `09eb4c4`,
`RESHUFFLE=8`, `THRESHOLD=8`, a table of four, the other seats all
`HeuristicBot`).  The candidate is the present play, and the control the
`ExpectBot` with the same gate that plays as `HeuristicBot` beyond it:

| | Setting | Deals | seed base |
| --- | --- | ---: | ---: |
| Web | `EXPECT=1.5 GATE_CARDS=10 SHALLOW=0 BASELINE_EXPECT=1` | 10,000 | 1795200000 |
| Desktop | `EXPECT=1.5 GATE_CARDS=14 SHALLOW=17 BASELINE_EXPECT=1 BASELINE_GATE=14` | 20,000 | 1795300000 |

- Each verdict is its own.  loss: that setting falls back to `HeuristicBot`
  beyond the gate when the threshold is not 10 (one more condition in
  `depth`, as for tables of two and three), and the zero-part run is made
  again.  win: the play stays, and that sentence of "Shape" is changed.
  Straddling 0: insufficient data, play is left alone, and it is written
  down.
- Trial run of 400 deals (seed base 1, not counted): web 56 seconds, 84%
  part, −0.798 ± 0.341; desktop 6 minutes, 78% part, −0.113 ± 0.344.  So the
  web's 10,000 deals take about 24 minutes with a standard error of about
  0.07, and the desktop's 20,000 deals about 5 hours and about 0.05.

### Results

| | Deals | Part | Seat 1 win rate (control → candidate) | Candidate minus control | 95% CI | Verdict |
| --- | ---: | ---: | --- | ---: | --- | --- |
| Web | 10,000 | 8,278 | 24.45% → 24.43% | +0.006 ± 0.074 | [−0.138, +0.151] | insufficient data |
| Desktop | 20,000 | 15,330 | 24.96% → 25.14% | +0.080 ± 0.049 | [−0.015, +0.175] | insufficient data |

- **Both settings straddle 0, and play is left alone.**  A table of four at
  八胡 goes on playing the shallow graph on the ten-胡 tables beyond the
  gate.  This is not "no loss": the web's interval admits −0.14, and the
  desktop's −0.02.
- **The minus signs of the trial run did not recur.**  The −0.798 ± 0.341 of
  400 deals (web) is +0.006 at 10,000 deals; the desktop's −0.113 became
  +0.080.
- **Compared with 十胡, no benefit of the tables was measured at 八胡.**
  The same settings at 十胡 are +0.191 ± 0.046 (web) and +0.204 ± 0.052
  (desktop), both wins.  八胡's own tables (application D) are still a
  candidate.
- If the desktop really is +0.08, telling it apart with 80% confidence would
  take about 60,000 deals in all, 15 hours (an estimate); not scheduled.

## Shape of the code

All still proposals.

```rust
pub struct Prune {
    pub generals: Option<usize>,
    pub discards: Discards,
    /// A node this many claims from a root is a leaf; `None` expands all
    pub depth: Option<u32>,
}

/// The pattern tables: what a hand is worth by its nine blocks
pub struct Eval { /* tables */ }

impl Eval {
    /// The value in points of (`hand`, `base`) with each count of public
    /// cards to come, from 0
    pub fn curve(&self, hand: Hand, base: u32, live: Hand) -> Vec<f64>;
}

impl Graph {
    pub fn leaves(&self) -> &[usize];
    pub fn values(&self, outlook: &Outlook, draws: usize, payoff: Payoff, leaves: &[Vec<f64>]) -> Vec<f64>;
}

impl ExpectBot {
    /// The same bot playing beyond its gate by a graph `depth` claims deep
    pub fn shallow(self, depth: u32, eval: Eval) -> Self;
}
```

- **No separate bot type.**  `ExpectBot` is now "exact inside the gate,
  `HeuristicBot` beyond", and one more field makes it "shallow beyond".
  k = 0 is the pure eval bot.  Choices of equal value keep `HeuristicBot`'s
  order as before.
- **A child differs from its parent in at most two blocks** (the block of
  the claim and the block of the discard), so the accumulator can be updated
  incrementally.  Not in the first version; do it once the eval is measured
  to be the bottleneck.

## Applications

After batch 4 has one win.  Those that change play each run their own A/B.

| | Application | Changes play | Condition to start |
| --- | --- | --- | --- |
| A | The hint marks points throughout | No, with batch 4 (done) | Batch 4 is a win |
| B | k = 0 for Monte Carlo's playouts and leaves (`expectation.md` application H) | Candidate | Batch 4 is a win |
| C | The hint shows the chance to win: one more output of the same table, labelled with `Payoff::Win` | No | Batch 1's calibration passes |
| D | Tables for 八胡 | Candidate | The web is to give 八胡 the same opponents |
| E | Opponent inference: `live` replaced by inferred counts, the pattern key unchanged | Candidate | Route 1 has an inference to plug in |

## Known approximations

- **All six items of `expectation.md` are inherited.**  The eval learns the
  model's value, and where the model is wrong it is wrong too.  A long hand
  looks further ahead, and "`live` fixed at the root" and "the other seats'
  discards taken as uniform" are more wrong than for a short hand; that is
  what batch 1's calibration measures.
- **The nine blocks' vectors add.**  The blocks constrain one another only
  through the threshold, the card count and the clock, and one layer of relu
  has to learn these three; should it learn them badly, batch 3 lists three
  remedies.
- **The labels at 18 cards or more come from graphs with pruned discards.**
- **The teacher is not the opponent.**  The table learns the DP's value, not
  how to beat `HeuristicBot`, so it has one doubt fewer than route 4's
  original distillation, "valid only against this opponent"; `Outlook`'s
  constants are still measured from `HeuristicBot` self-play.

## Not doing

- **A knapsack for every live card at run time.**  "How many cards short of
  the threshold" and "which cards make it one fewer" are mahjong's shanten
  and effective tiles; by the cost of the present `reach` it is about 100
  microseconds a leaf (an estimate), which k = 2 cannot afford.  It serves
  only as a remedy for batch 3, and then computed once, not for every card.
- **Hand-tuned weights for melds, 搭子 and lone cards.**  That is
  `HeuristicBot`'s grade; route 2 measured that its linear combinations are
  already used up.
- **Monte Carlo labels.**  The DP's values have no noise, and have already
  beaten the gated Monte Carlo (`expectation.md` batch 4).
- **Expanding in full once a leaf is short enough to be inside the gate.**
  How large the subgraphs under many leaves are together has not been
  measured; look again once batch 4 has won.
- **Reusing the accumulator or the graph across decisions.**  `live` changes
  at every decision.

## Open questions

- The win rate of long hands is underestimated by 24%: is it `Loose` or
  h × 1.5?  (Batch 1's calibration)
- ~~How accurate must the eval be, and how much error does one more level
  buy?~~  Batch 2: k = 0 needs σ within 0.4; k = 1 tolerates σ 2, bias ±1,
  λ 0.5.  Of the errors that misjudge similar hands together, only the scale
  was measured.
- ~~Can nine added vectors learn the threshold?  How large must D be?~~
  Batch 3: they can, validation RMSE 0.78 at D = 16; D = 1 is not enough.
- How far off are the labels at 18 cards or more, taken from `Loose` graphs?
