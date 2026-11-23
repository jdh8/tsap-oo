# A table of two: do the same changes show more clearly there?

Measured 2026-11-16; the Monte Carlo rerun and the Peeker 2026-11-17.  The
question: at a table of four, doing the right thing is not always rewarded.
If the same rules are played by two, does an A/B tell good from bad in fewer
deals?  Two changes to `HeuristicBot` known to work at four are each
measured again here at a table of two, then `MonteCarloBot` against
`HeuristicBot` as well, rerun the next day on a fresh seed base, along with
the Peeker, which peeks at the hands.  Neither the library nor the bots were
changed; the table of two exists only in scratch copies made for the
measurement.

## Answer

**The engine plays a table of two with one constant changed.  The two
hand-efficiency changes are no easier to see at two: one is harder, one
slightly easier.  The positive confidence interval Monte Carlo first
measured did not repeat on the rerun: insufficient data.  The Peeker, which
peeks at the opponent's hand, wins 2.8 points a deal: information pays, and
more computing on the same information does not.**

- **A table of two is ready-made.**  Change `Seat::COUNT` from 4 to 2 and
  the library deals, plays out and settles with no further change; the rules
  are as before (20 cards each, 21 to the dealer, 6 dead cards, ten 胡).
  This is the same as Board Game Arena's table of two: its rules do not
  change with the number of players ([`prior-art.md`](prior-art.md), section
  C).
- **A round is four tenths cheaper.**  For the same bot, a round at two
  takes 0.57 to 0.61 times the compute of a round at four.
- **A table of two is almost never exhausted.**  1 round in 20,000 was
  exhausted, against 599 at four.  The winner's mean 胡 is almost the same
  (12.13 against 12.20).
- **In the deals that differ, the ratio of signal to noise is no better.**
  Over the deals where play differs, the mean of the score difference
  divided by its standard deviation: "keep a waiting hand" falls from 0.18
  at four to 0.10 at two; "do not eat into a whole meld" is 0.12 against
  0.13.
- **z at the same 200,000 deals**: 10.7 → 6.3 for the former, 16.1 → 19.3
  for the latter.  Converted into the compute needed for the same
  confidence, the former needs 1.8 times as much at two and the latter only
  0.4 times.
- **`MonteCarloBot` at 128 samples was +0.347 ± 0.175 points a deal at two
  the first time**, 95% CI [+0.005, +0.689], 3,200 deals.  **The rerun on a
  fresh seed base is +0.050 ± 0.172**, 95% CI [−0.287, +0.388], insufficient
  data.  The first run's z of about 2.0 did not hold.  The same deal count
  at four gave +0.148 ± 0.276, insufficient data as well.
- **The Peeker at two is +2.824 ± 0.192 points a deal**, 95% CI [+2.449,
  +3.200], 3,200 deals, win; the win rate goes from 49.31% to 60.88%.  It is
  a `MonteCarloBot` that sees the opponent's hand and samples only the pile.
  Less the Monte Carlo rerun that is +2.77 ± 0.26: the upper bound of
  opponent inference at a table of two.

Both of these `HeuristicBot` changes are about "how to play one's own hand
faster".  Decisions that interact with the opponent (feeding, defence) were
not measured: the only version tried at four was not kept in the repository
and cannot be replayed.

## Method

The rules of a table of two are those of four with the seat count set to 2:

- 20 cards each, 21 to the dealer, a pile of 71 cards, 6 of them dead.
- The previous seat and the next are the same person, so every card the
  opponent discards or draws can be eaten.
- The winner collects their 胡 from the only opponent.

The A/B is `ab/`'s program as it was: the candidate in the second seat and
the control in the first, compared with the same deal with the control in
both seats, the dealer rotating.  The candidate and the control of each pair
are two versions from the repository, each with the same constant changed.
The deal count was fixed before the run at 200,000, the same as at four at
the time; the seed base is the clock at the time.

```sh
S=$(mktemp -d)
for pair in "A 3526146 a74b951" "B 1b644f2 6fc8f21" "M 3aa5b0c 3aa5b0c"; do
  set -- $pair
  mkdir -p "$S/$1/ab/target/base"
  git archive "$2" | tar -xm -C "$S/$1"
  git archive "$3" | tar -xm -C "$S/$1/ab/target/base"
  sed -i '0,/^name = "tsap-oo"$/s//name = "tsap-oo-base"/' \
    "$S/$1/ab/target/base/Cargo.toml"
  sed -i 's/COUNT: usize = 4/COUNT: usize = 2/' \
    "$S/$1/src/lib.rs" "$S/$1/ab/target/base/src/lib.rs"
  sed -i 's/(i % 4) as usize/(i % 2) as usize/' "$S/$1/ab/src/main.rs"
  (cd "$S/$1/ab" && cargo build --release)
done
"$S/A/ab/target/release/tsap-oo-ab" 1794830208 200000
"$S/B/ab/target/release/tsap-oo-ab" 1794830528 200000
```

The four-player numbers to compare with: pair A is from `CHANGELOG.md` (seed
base 1794772616) and was not rerun; pair B was replayed once on its original
seed base 1794765506, the score difference matching the record of the time
digit for digit, which also supplied the "deals that differ" not recorded
then.  The replay is the commands above without the two `sed` lines that
change the seat count.

For the Monte Carlo run, candidate and control are both `3aa5b0c`, each
turned into a table of two as above, with the candidate a `MonteCarloBot` of
128 samples.  The deal count was fixed before the run at 3,200, the same as
the run at four (`CHANGELOG.md`, `SAMPLES=128 ab/run.sh 3200`, control
`6428dab`):

```sh
SAMPLES=128 "$S/M/ab/target/release/tsap-oo-ab" 1794830779 3200
```

The rerun and the Peeker use `e40cca4`, each on a fresh seed base, the deal
count again fixed before the run at 3,200.  The rerun's candidate and
control are both `e40cca4`.  The Peeker's candidate is the working tree of
`e40cca4` plus `View::peek` and `ab/src/peeker.rs` (later committed as
`a86b4f2`), and its control `e40cca4`; `RESHUFFLE=0`, with no reshuffle from
where the two part.  Both scratch copies were turned into tables of two as
above:

```sh
SAMPLES=128 ab/target/release/tsap-oo-ab 1794846583 3200
SAMPLES=128 PEEK=1 RESHUFFLE=0 ab/target/release/tsap-oo-ab 1794847090 3200
```

The basic statistics use the `arena` example at `3aa5b0c`, the table of four
as the repository has it and the table of two again with only that constant
changed:

```sh
cargo run --release --example arena -- 20000
```

## Data

### Basic statistics (`3aa5b0c`, 20,000 rounds, seed 0)

| | Four | Two |
| --- | ---: | ---: |
| Won | 19,401 | 19,999 |
| Exhausted | 599 | 1 |
| 相公 | 0 | 0 |
| Rounds won by each seat | 4,856 / 4,932 / 4,848 / 4,765 | 9,938 / 10,061 |
| Winner's mean 胡 | 12.20 | 12.13 |

### Two changes known to work (200,000 deals each)

A is "keep a waiting hand" (`3526146` against `a74b951`); B is "do not eat
into a whole meld, and the next seat eats only when that beats the draw by 3
胡" (`1b644f2` against `6fc8f21`).

| | A, four | A, two | B, four | B, two |
| --- | ---: | ---: | ---: | ---: |
| seed base | 1794772616 | 1794830208 | 1794765506 | 1794830528 |
| Deals that differ | 3,551 | 3,901 | 16,804 | 21,854 |
| Score difference (points a deal) | +0.111 ± 0.010 | +0.030 ± 0.005 | +0.283 ± 0.018 | +0.222 ± 0.011 |
| 95% CI | [+0.091, +0.132] | [+0.021, +0.040] | [+0.249, +0.318] | [+0.199, +0.244] |
| Verdict | win | win | win | win |
| Win rate, control → candidate | 24.24% → 24.52% | 49.97% → 50.15% | 24.50% → 25.05% | 50.04% → 50.98% |
| z (score difference ÷ standard error) | 10.7 | 6.3 | 16.1 | 19.3 |

z and the numbers of the next table are worked back from the two ends of the
printed confidence interval, which is printed to three decimals only, so
they are approximate.

### Only the deals where play differs

A deal where play is the same has a score difference of exactly 0; all the
signal and all the noise are in the deals that differ.

| | A, four | A, two | B, four | B, two |
| --- | ---: | ---: | ---: | ---: |
| Mean of the score difference (points) | 6.3 | 1.6 | 3.4 | 2.0 |
| Standard deviation of the score difference (points) | 35 | 15.5 | 27 | 15.6 |
| Mean ÷ standard deviation | 0.18 | 0.10 | 0.12 | 0.13 |

Scores at a table of two are smaller to begin with: the winner collects from
one seat only.  So the row to compare is the last.

### Cost

The CPU time the A/B program used, divided by the rounds played (two rounds
a deal), on 16 threads:

| | Four | Two | Two ÷ four |
| --- | ---: | ---: | ---: |
| A (the bot of `3526146`), a round | 18.9 ms | 11.5 ms | 0.61 |
| B (the bot of `1b644f2`), a round | 3.3 ms | 1.9 ms | 0.57 |

The time of A at four is from a separate timing of 20,000 deals (seed base
7); the rest are from the 200,000 deals of the tables above.

For the same z, the deals needed are in inverse proportion to the square of
z:

| | Deals needed at two | Compute needed at two |
| --- | ---: | ---: |
| A | 2.9 times four's | 1.8 times four's |
| B | 0.70 times four's | 0.40 times four's |

### `MonteCarloBot` at 128 samples against `HeuristicBot` (3,200 deals each)

| | Four | Two |
| --- | ---: | ---: |
| Versions of candidate and control | control `6428dab` | `3aa5b0c` |
| seed base | 1794777431 | 1794830779 |
| Deals that differ | 1,082 | 1,251 |
| Score difference (points a deal) | +0.148 ± 0.276 | +0.347 ± 0.175 |
| 95% CI | [−0.392, +0.689] | [+0.005, +0.689] |
| Verdict | insufficient data | win |
| Win rate, control → candidate | 24.47% → 24.97% | 49.44% → 50.41% |
| z | 0.5 | 2.0 |
| In the deals that differ, mean of the score difference (points) | 0.44 | 0.89 |
| In the deals that differ, standard deviation of the score difference (points) | 27 | 15.8 |
| In the deals that differ, mean ÷ standard deviation | 0.02 | 0.06 |

The four-player numbers are from `CHANGELOG.md` and were not rerun.  The
three commits that touched `src/` after `6428dab` were each confirmed over
20,000 deals to leave `HeuristicBot`'s play unchanged; `MonteCarloBot` was
not confirmed separately.

This run at a table of two took 973 CPU minutes, about 18 CPU seconds a
deal, 71 minutes on 16 threads.

### The rerun and the Peeker (table of two, 3,200 deals each, control `e40cca4`)

| | Monte Carlo, first run | Monte Carlo, rerun | Peeker |
| --- | ---: | ---: | ---: |
| Candidate's version | `3aa5b0c` | `e40cca4` | `e40cca4` plus the Peeker |
| Control's version | `3aa5b0c` | `e40cca4` | `e40cca4` |
| seed base | 1794830779 | 1794846583 | 1794847090 |
| Deals that differ | 1,251 | 1,202 | 1,616 |
| Score difference (points a deal) | +0.347 ± 0.175 | +0.050 ± 0.172 | +2.824 ± 0.192 |
| 95% CI | [+0.005, +0.689] | [−0.287, +0.388] | [+2.449, +3.200] |
| Verdict | win | insufficient data | win |
| Win rate, control → candidate | 49.44% → 50.41% | 51.47% → 51.47% | 49.31% → 60.88% |
| z | 2.0 | 0.3 | 14.7 |
| In the deals that differ, mean of the score difference (points) | 0.89 | 0.13 | 5.6 |
| In the deals that differ, standard deviation of the score difference (points) | 15.8 | 15.9 | 14.8 |
| In the deals that differ, mean ÷ standard deviation | 0.06 | 0.01 | 0.38 |

- **The Peeker less the Monte Carlo rerun**: +2.774 ± 0.258, 95% CI [+2.27,
  +3.28].  The two runs have different seed bases, so the difference is
  unpaired, and its standard error is the root of the sum of the two
  squares.
- **The two Monte Carlo runs together** (weighted by the inverse of the
  variance): +0.196 ± 0.123, 95% CI [−0.045, +0.436].  This only describes;
  it is not a verdict: the second run was made because of the first's
  result, and the two also differ in version (`e40cca4` lies between).
- **Cost**: the rerun took 991 CPU minutes and the Peeker 971, each about 18
  CPU seconds a deal.  The Peeker had the machine to itself, 73 minutes on
  16 threads; the rerun shared the CPU with another A/B and took 122
  minutes, a time that cannot serve as a baseline.

## Reading

- **Monte Carlo shows no more clearly at a table of two.**  The first run's
  z is 2.0 and the rerun's 0.3, no different from the 0.5 at four.  Both
  runs leave room for a true value around +0.3 points, and for 0; 3,200
  deals cannot tell them apart.  In `../gin-rummy-engine` a Monte Carlo of
  the same build clearly beats the greedy bot; nothing of the kind was
  measured here.
- **What pays is information, not more computing.**  The Peeker and
  `MonteCarloBot` are the same search with the same 128 samples, differing
  only in whether the opponent's hand is the real one or a uniform guess.
  One wins 2.8 points; the other cannot be measured.  In the deals that
  differ the Peeker's mean ÷ standard deviation is 0.38, where every other
  change in this document lies between 0.01 and 0.18.
- **2.8 points is an upper bound, not a forecast.**  Opponent inference can
  only guess the hand from the public cards and cannot get it all right; the
  Peeker also happens to remove the search's fault of deciding separately in
  each sampled deal.  How much of it inference can get was not measured, nor
  was the upper bound at four.
- **The noise comes from the deal, not the number of players.**  Once the
  two sides play differently, the cards drawn afterwards decide most of the
  score difference.  This is the same at two and at four: mean ÷ standard
  deviation lies between 0.10 and 0.18 at both.  In the literature, the
  evaluation of two-player mahjong and two-player poker is likewise drowned
  by the luck of the deal ([`prior-art.md`](prior-art.md), section D).
- **What a change is worth varies with the table.**  At four, "keep a
  waiting hand" is worth 6.3 points on each deal that differs, about 13% of
  one win or loss (win 3 × 12 points, lose 12 points, about 49 points
  apart); at two it is worth 1.6 points, about 6% of one win or loss (about
  24 points apart).  One possible cause is that at four three seats are
  discarding cards for the waiting hand to win on, and at two only one; this
  is a guess and was not measured.
- **So a size measured at one kind of table cannot be carried straight to
  the other.**  Both changes win at both tables, in the same direction; in
  points a deal, four is 3.7 times two (A) and 1.3 times (B).

## Caveats

- **The two Monte Carlo results disagree: one a win with its lower bound at
  +0.005, one insufficient data.**  The two intervals overlap and do not
  contradict each other, but the first cannot be relied on.  To pin its size
  down the standard error has to come down to about 0.05: some 40,000 deals,
  200 CPU hours.  During the first run a separate 32-deal timing ran on the
  same machine for 1 minute 41 seconds, which affects only the time, not the
  result.
- **The Peeker was measured only at a table of two, once.**  Its interval is
  more than ten standard errors from 0 and does not depend on a rerun; but
  what the hands of three seats are worth at four is another number.
- **Only two changes, and both about the efficiency of one's own hand.**
  The author's reasoning is that in a two-player zero-sum game right and
  wrong are rewarded and punished more often, which is most likely to show
  in decisions that interact with the opponent: at a table of two the
  opponent can eat every card I discard, and I pay for every round it wins.
  Decisions of this kind were not measured here.
- **No tests were run at a table of two.**  `tests/round.rs` hard-codes four
  seats, and so does one unit test of the library.  The A/B program aborts
  on an illegal move or when the two sides fall out of step, which did not
  happen in 800,000 rounds; beyond that, the invariants `CLAUDE.md` lists
  were not verified at a table of two.
- **`HeuristicBot` is tuned for four.**  Its constants and its "compare
  first before the next seat draws" logic were all measured at four.
  Whether it is a reasonable control at two was not measured.
- **The timings of A at four and A at two are not on the same deals**, while
  B ran the same deal count once each.  The cost ratio is only approximate.
