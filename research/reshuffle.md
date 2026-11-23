# Reshuffling the pile from where the two part: how many deals can an A/B save?

Measured 2026-11-17.  The question: `ab/` has the candidate and the control
play the same deal, but once the two play differently, the cards drawn
afterwards decide most of the score difference.  If the pile left where the
two first part is reshuffled K times, each played out by both, and the score
differences averaged, can the same confidence be had for fewer deals and
less compute?  This is the first step of route 0 in `doc/stronger-bot.md`.

## Answer

**Yes.  At K = 8 the same standard error takes a little over a fifth of the
deals and a quarter of the compute.  `ab/` defaults to K = 8 from now on.**

- **The luck after the split is the bulk of the noise.**  With the pile
  reshuffled from one and the same split, the score difference has a
  standard deviation of 18 to 19 points; the true expectations of different
  splits have a standard deviation of only 4.4 to 4.7 points.
- **The estimator is unbiased, and passes acceptance.**  With the candidate
  the same as the control, not one of 20,000 deals parts and the score
  difference is exactly 0.  The change used for calibration (known to be
  +0.283 ± 0.018) was measured once each at 8, 16 and 32 reshuffles, and
  every confidence interval covers +0.283.
- **K anywhere from 8 to 16 is about the same.**  In compute per unit of
  confidence, K = 8 saves 4.3 times over the original, K = 16 4.0 times and
  K = 32 3.6 times.  In deals, the larger K the more is saved: 5.5, 7.9 and
  11.5 times.  The more often a candidate parts, the less the fixed cost of
  the stretch before the split matters and the smaller the best K, so the
  default is 8.
- **Candidates like `MonteCarloBot` keep the original estimator**
  (`RESHUFFLE=0`).  Every replay has to run the Monte Carlo once more, and
  the deals saved do not buy back that much compute; this was not measured.

## Why it is unbiased

Before the split, every decision of every seat sees only the cards already
turned up.  So at the moment of the split, the order of the cards left in
the pile is still uniformly random given everything known.  The rest of the
original deal is only one sample from that uniform distribution; averaging K
new samples in its place leaves the expectation unchanged and only averages
out the luck after the split.

The four seats' concealed cards are part of the position and cannot be dealt
again.  That is exactly why `Round::determinize` cannot be used: it deals
the other seats' hands again as well.

Where the two sides play exactly the same round, the score difference is 0
as before, so nothing needs replaying.

## Approach

`ab/src/main.rs`.  For each deal:

1. Shuffle the whole deck with the deal's seed and play it once with the
   candidate in the second seat.  At each of the candidate's decisions, the
   control's bot is also asked what it would do in the same place.
2. Stop where the two first differ, and record every decision before it,
   each side's choice, and how many cards the pile has left.  Cards are
   drawn from the end of the pile, so what is left is the first `left` cards
   of the original deal.
3. K times over: shuffle those `left` cards once, deal the same deal again
   with `Round::from_pile`, replay the recorded decisions to the split, then
   on one side take the control's choice with the control in all four
   seats, and on the other take the candidate's choice with the candidate
   staying in the second seat.  Both sides use the pile from the same
   shuffle.
4. The deal's score difference is the mean of the K.  A deal that never
   parts counts 0.

The replay only walks the recorded decisions and calls no bot, so the
stretch before the split takes almost no time.  The library needs no change:
both crates use only the public `Round::from_pile` and `Round::pile_len`.

The same harness also does the analysis of variance (one-way): among the
deals that part, the variation among the K replays is "the luck of the pile
within one split", and the variation of the means over K, less 1/K of the
former, is "the true differences between splits".

To reproduce (the calibration set):

```sh
S=$(mktemp -d)
git archive 1b644f2 | tar -xm -C "$S"
rm -rf "$S/ab" && mkdir -p "$S/ab/target/base"
cp -r ab/src ab/Cargo.toml ab/Cargo.lock "$S/ab/"
git archive 6fc8f21 | tar -xm -C "$S/ab/target/base"
sed -i '0,/^name = "tsap-oo"$/s//name = "tsap-oo-base"/' \
  "$S/ab/target/base/Cargo.toml"
# 1b644f2 has no `peek` feature, so the Peeker goes with it.
sed -i '/^\[features\]/,$d' "$S/ab/Cargo.toml"
(cd "$S/ab" && cargo build --release)
RESHUFFLE=0 "$S/ab/target/release/tsap-oo-ab" 1794765506 200000
RESHUFFLE=32 "$S/ab/target/release/tsap-oo-ab" 1794846831 12000
RESHUFFLE=8 "$S/ab/target/release/tsap-oo-ab" 1794846881 40000
RESHUFFLE=16 "$S/ab/target/release/tsap-oo-ab" 1794846911 40000
```

The acceptance run, with the candidate the same as the control, uses the
repository as it stands: `ab/run.sh 20000` (control `e40cca4`, seed base
1794846981).

Every deal count was fixed before the run, and the seed base taken from the
clock at the time; the first line is a replay of the original run.

## Data

The calibration set is "not eating into a meld already whole": candidate
`1b644f2`, control `6fc8f21`, a table of four.

| | Original | K = 32 | K = 8 | K = 16 |
| --- | ---: | ---: | ---: | ---: |
| seed base | 1794765506 | 1794846831 | 1794846881 | 1794846911 |
| Deals | 200,000 | 12,000 | 40,000 | 40,000 |
| Deals that differ (original) / part (reshuffled) | 16,804 | 1,903 | 6,682 | 6,534 |
| Score difference (points a deal) | +0.283 ± 0.018 | +0.258 ± 0.021 | +0.289 ± 0.017 | +0.284 ± 0.014 |
| 95% CI | [+0.249, +0.318] | [+0.217, +0.300] | [+0.256, +0.322] | [+0.257, +0.312] |
| Win rate, control → candidate | 24.50% → 25.05% | 24.02% → 24.50% | 25.01% → 25.58% | 24.15% → 24.71% |
| Standard deviation within a split (points) | | 18.36 | 19.12 | 18.93 |
| Standard deviation between splits (points) | | 4.63 | 4.41 | 4.74 |

The replay of the original run is identical to the digit with what
`CHANGELOG.md` records, so the new harness at `RESHUFFLE=0` is the original
estimator.

"Part" outnumbers "differ": of the deals where the two sides first make
different decisions, about half end with the same score after all.

### Cost

The compute per deal is the CPU time the harness used over the deal count,
on 16 threads.  The variance per deal is the square of the standard
deviation times the deal count (the standard deviation worked back from the
two ends of the printed confidence interval).

| | Original | K = 32 | K = 8 | K = 16 |
| --- | ---: | ---: | ---: | ---: |
| CPU time per deal | 6.5 ms | 20.8 ms | 8.3 ms | 12.6 ms |
| Variance per deal (points²) | 61.9 | 5.4 | 11.3 | 7.9 |
| Deals for the same confidence | 1 | 1/11.5 | 1/5.5 | 1/7.9 |
| Compute for the same confidence | 1 | 1/3.6 | 1/4.3 | 1/4.0 |

The variance per deal is about p × (μ² + σ²split + σ²luck ÷ K): p is the
share of the deals that part (about 16%), and μ the mean score difference of
the deals that part (about 1.7 points).  All three K in the table above fit
this formula.

## Caveats

- **The calibration is one change only.**  About 16% of its deals part.  For
  a change whose share is very different (a fit of the weights that plays
  30% of the deals differently, say), the best K will differ and so will the
  factor saved: the larger the differences between splits, the less
  reshuffling helps.
- **The timing is not clean.**  During these runs another A/B, a Monte Carlo
  one at a table of two under `nice 10`, was on the same machine.  It was
  the same for all four, and the factors are only approximate.
- **Monte Carlo candidates were not measured.**  On a replay the candidate's
  bot is built from scratch, so `MonteCarloBot`'s random numbers differ;
  this does not affect unbiasedness, but whether it is worth it was not
  measured.
