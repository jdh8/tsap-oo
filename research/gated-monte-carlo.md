# Monte Carlo that searches only late: where does the gate open?

Measured 2026-11-17.  The question: `MonteCarloBot` at 128 samples takes
seconds a move, and an A/B of 3,200 deals cannot tell whether it is stronger
than `HeuristicBot`.  If it searched only when the hand is short, or already
waiting, and played greedily otherwise, the cost would fall by a factor of
tens, and the same time would play more than ten times the deals; can it be
measured then?

## Answer

**A `MonteCarloBot` that searches only with 10 concealed cards or fewer
beats `HeuristicBot`: +0.070 ± 0.026 points a deal, 95% CI [+0.019, +0.121],
40,000 deals, win.  Searching only while waiting, and the union of the two
conditions, are each insufficient data at 40,000 deals.  The three point
estimates lie within one another's error, and these data cannot tell which
gate is better.**

- **The 10-card gate is the first search to beat `HeuristicBot` at a table
  of four.**  The same bot ungated is +0.148 ± 0.276 (3,200 deals,
  insufficient data); the two intervals overlap, and what the nine tenths of
  the decisions the gate shuts out are worth cannot be measured.
- **It searches only 9.7% of the decisions, for 2.7% of the search time.**
  Ungated, a round takes 25 CPU seconds and a decision 3.1 seconds; a
  decision at 10 cards or fewer takes 0.9 seconds.
- **The waiting gate is wider, yet no better.**  Decisions while waiting are
  18% of all and 6.9% of the time: +0.048 ± 0.030, the interval straddling
  0.
- **The union is no larger.**  Searching when either condition holds, the
  deals that part number 2,302, near the sum of the two, yet the score
  difference is +0.061 ± 0.035.  The three runs used different deals and the
  gaps are all around one standard error, so the union cannot be called
  worse.
- **The effect is small.**  +0.07 points a deal is about the low end of
  `HeuristicBot`'s past improvements, with the win rate going 24.00% →
  24.16%.

## Method

`MonteCarloBot::gated(cards, shanten)` searches in two cases and returns the
greedy first choice otherwise:

- the concealed cards after the discard number no more than `cards`;
- or `shanten` is given and `HeuristicBot::shanten` is within that limit: 0
  is waiting, 1 is waiting after one more eat or pong.  If the eat or pong
  on offer would leave the hand waiting, that counts as waiting too.

`ab/` reads the two numbers from `GATE_CARDS` and `GATE_SHANTEN`.  All three
runs are at a table of four, the candidate in one seat, 128 samples,
`RESHUFFLE=0` (every replay would be one more search), the control
`4596efe`, the deal count fixed before the run at 40,000 each, each with a
seed base of its own:

```sh
SAMPLES=128 RESHUFFLE=0 GATE_CARDS=10 \
  ab/target/release/tsap-oo-ab 1794910000 40000
SAMPLES=128 RESHUFFLE=0 GATE_CARDS=0 GATE_SHANTEN=0 \
  ab/target/release/tsap-oo-ab 1794950000 40000
SAMPLES=128 RESHUFFLE=0 GATE_CARDS=10 GATE_SHANTEN=0 \
  ab/target/release/tsap-oo-ab 1794990000 40000
```

The three ran one after another, on a binary copied out after the build, so
nothing was rebuilt in the meantime.  The binary of the first run is older
than that of the other two: `HeuristicBot::shanten` did not yet look at the
eat or pong on offer.  With only `GATE_CARDS` set the 向聽數 is never asked
for, so the two versions play the same under the first run's settings; the
first 2,000 deals of the first run, replayed once with each binary, give
output identical to the digit (55 deals differ, −0.062 ± 0.090).  The binary
of the last two runs is the committed version.

What was scheduled at first was `GATE_CARDS=10 GATE_SHANTEN=1` (20,000
deals), cancelled before it began: the timing at that point showed the
decisions within one eat or pong of waiting to be 37% of all, too wide (49%
once the eat or pong on offer is counted, see the table below).  The last
two runs above replaced it.

## Data

### A/B (40,000 deals each, control `4596efe`)

| Gate | 10 cards or fewer | Waiting | 10 cards or fewer, or waiting |
| --- | ---: | ---: | ---: |
| `GATE_CARDS` | 10 | 0 | 10 |
| `GATE_SHANTEN` | unset | 0 | 0 |
| seed base | 1794910000 | 1794950000 | 1794990000 |
| Deals that differ | 1,518 | 1,161 | 2,302 |
| Score difference (points a deal) | +0.070 ± 0.026 | +0.048 ± 0.030 | +0.061 ± 0.035 |
| 95% CI | [+0.019, +0.121] | [−0.011, +0.108] | [−0.009, +0.130] |
| Verdict | win | insufficient data | insufficient data |
| Win rate, control → candidate | 24.00% → 24.16% | 24.30% → 24.34% | 24.38% → 24.51% |
| z (score difference ÷ standard error) | 2.7 | 1.6 | 1.7 |
| Time on 16 threads | 44 minutes | 90 minutes | 104 minutes |

z is worked back from the three decimals printed and is approximate.  The
times are from the timestamps of the output files.

### Where the search time goes

The `gate` example seats an ungated 128-sample `MonteCarloBot` in seat 1
with `HeuristicBot` in the other three seats, times every decision, and
sorts the decisions by whether each gate would let them through:

```sh
cargo run --release --example gate -- 256 1794901984 8 128
```

Seed base 1794901984, 256 rounds, 2,065 decisions, 25.2 CPU seconds a round:

| Gate | Decisions | Share | Time | Each (milliseconds) | Times the greedy choice was overturned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ungated | 2,065 | 100.0% | 100.0% | 3,129 | 284 |
| 10 cards or fewer | 200 | 9.7% | 2.7% | 876 | 16 |
| Waiting | 372 | 18.0% | 6.9% | 1,194 | 22 |
| 10 cards or fewer, or waiting | 460 | 22.3% | 8.3% | 1,162 | 36 |
| Within one eat or pong of waiting | 1,016 | 49.2% | 31.9% | 2,030 | 88 |

"Share" is of the number of decisions, "Time" the share of the search time.
8 threads ran at once, so the milliseconds a decision are somewhat longer
than when run alone; the shares are unaffected.

## How to read it

- **The win is called by the iron rules.**  The lower bound of the interval
  is +0.019, and z is about 2.7.  The same batch of experiments read three
  gates and one of them crossed the line; at a table of two, the run with z
  about 2.0 did not hold up when rerun ([`two-seats.md`](two-seats.md)).
  For more confidence, run 40,000 deals once more on a new seed base, 44
  minutes.
- **The gates cannot be told apart.**  The standard error of a difference
  between two of them is around 0.04, and the gaps are 0.02 at most.
  Telling 0.02 points apart takes over a million deals a gate.
- **Why short hands are cheap.**  The length of a playout and the cost of
  each step both shrink with the hand.  A waiting hand is not necessarily
  short, so the waiting gate costs more a decision.

- **The search overturns the greedy choice less often late.**  Ungated, 14%
  of the decisions are overturned; at 10 cards or fewer 8%, and while
  waiting 6%.

## Not yet done

- Rerun the 10-card gate once on a new seed base.
- Other card counts (8, 12, 14).  Each is an A/B of 40,000 deals or more.
- The web page is out of scope: a decision at 10 cards or fewer takes 0.9
  seconds even natively, and the web page stays with `HeuristicBot`.  This
  bot is for a native front end.
