# Fitting the grade's constants to data: what each term is worth in points

Measured 2026-11-16.  The question: `HeuristicBot`'s grade is a weighted sum
with a few hand-tuned constants (`SCALE`, `WEIGHT`, `OWN`, `WAIT`).  Fitted
to the results of self-play, what should these weights be?  Does playing by
the fitted weights make it stronger?  The bot's base is `10a046f`;
`HeuristicBot::terms` and the `fit` example are new here and do not change
play (`ab/run.sh 20000`, seed base 1794823476, 0 of 20,000 deals differ).

## Answer

**The fitted weights are far from the hand-tuned ones, yet playing by them
is no stronger.  Tuning constants has no visible room left; the bot is
unchanged.**

- **1 胡 in hand is worth about 1.4 points.**  The two fits give 1.38 ± 0.03
  and 1.42 ± 0.05.  This is the exchange rate `doc/stronger-bot.md` asks
  for.
- **The two fits agree with each other, and both are far from the hand-tuned
  weights.**  In the grade's units, where 1 胡 in hand counts 32: a loose
  card −45 to −51 (hand-tuned −128), the gap to the threshold −11 to −18
  (hand-tuned −64), a 搭子 2.0 to 2.8 (hand-tuned 1), the cards waited on 24
  to 26 (hand-tuned 8).
- **Playing by the fitted weights never won.**  Three A/Bs of 200,000 deals
  each: two insufficient data, one loss.  The one that replaced every weight
  played differently in 30% of deals, and its score difference was still
  −0.005 ± 0.034; the weights fitted only near the choice gave −0.070 ±
  0.025.
- **The discard the bot picks is better than the other choices on average.**
  The other choices graded within 16 behind it score 0.46 points less on
  average, and the further behind, the more they lose.  Choices of the same
  grade make no difference (+0.09 ± 0.12).
- **The grade is steeper near the choice.**  Within 64 behind, each 32
  behind costs about 2 points; at 256 or more behind, each 32 costs less
  than 1 point.  A linear fit reads this bend as "loose cards weigh too
  much", but those choices would never be picked anyway, and changing it
  does not make play better or worse.

## The terms of the grade

`HeuristicBot` lays the hand out twice (once for the most 胡, once for the
fewest loose cards) and takes the layout of the higher grade.  The grade is
the sum of the terms below, each times its weight; `HeuristicBot::terms`
returns the value of each term:

| Term | Meaning | Hand-tuned weight |
| --- | --- | ---: |
| `hu` | The 胡 of the melds | 32 |
| `loose` | The number of loose cards | −128 |
| `short` | How many half-胡 short of the threshold, less what the 搭子 may yet supply | −64 |
| `own` | For each loose card, the copies of it not yet seen | 2 |
| `kept` | 搭子: the 胡 once eaten × the completing cards not yet seen (four at most) | 1 |
| `lonely` | Lone cards: the 胡 once eaten × 4 | −1 |
| `waits` | The cards not yet seen that this hand wins on | 8 |

At a claim, the 胡 of the exposed melds are counted in as well, at 32.

## How it was measured

Four `HeuristicBot`s play 60,000 rounds under `Rules::new()`, seed base
20261119, the dealer rotating.  Two least-squares fits are made:

1. **Hands.**  At every discard, record the terms of each of the four seats'
   hands (the discarder's less the card discarded), against that seat's
   final score for the round.  Four control variables that the bot does not
   look at, but that affect the score, go in as well: the cards left in the
   pile, the cards this seat has not seen, the fewest concealed cards among
   the other three seats, and the total 胡 exposed by the other three seats.
   Without the control variables, `own` turns into a stand-in for "the round
   is still young" (its coefficient goes from −2 to +20).
2. **Choices.**  At one discard in eight, try every card the discarder may
   discard, each on the same pile, with the four seats playing as
   `HeuristicBot` to the end.  Fit "this choice's score − the score of the
   bot's actual choice" to "the difference between the two in each term",
   with no constant term.  Everything the choices of one decision share (the
   stage of the round, the opponents' hands) cancels out.

The standard errors come from the spread of the fits of 16 shards, each
fitted by itself: every round falls in one shard only, and rounds are
independent of each other.

To reproduce (the bot must play as at `10a046f`; about half an hour on 16
cores):

```sh
cargo run --release --example fit -- 60000 20261119
```

## Data

"Points" is the change in score when the term goes up by 1; "at 32" converts
the same row to units where `hu` = 32, to compare directly with the
hand-tuned weights.

### The hand fit, whole round (3,471,684 records)

| Term | Hand-tuned | Points | ± | At 32 | ± |
| --- | ---: | ---: | ---: | ---: | ---: |
| Constant | | −17.948 | 0.272 | | |
| Exposed 胡 | 32 | 0.987 | 0.026 | 22.95 | 0.59 |
| `hu` | 32 | 1.377 | 0.032 | 32.00 | 0.73 |
| `loose` | −128 | −2.192 | 0.048 | −50.95 | 1.12 |
| `short` | −64 | −0.775 | 0.025 | −18.01 | 0.58 |
| `own` | 2 | −0.074 | 0.015 | −1.72 | 0.35 |
| `kept` | 1 | 0.087 | 0.004 | 2.03 | 0.08 |
| `lonely` | −1 | −0.055 | 0.005 | −1.28 | 0.12 |
| `waits` | 8 | 1.111 | 0.008 | 25.83 | 0.18 |
| Cards in the pile | | 0.005 | 0.009 | | |
| Cards not seen | | 0.241 | 0.008 | | |
| Fewest concealed cards of an opponent | | 0.146 | 0.021 | | |
| Opponents' exposed 胡 | | −0.297 | 0.008 | | |

### The hand fit, by cards left in the pile (at 32)

| Term | Hand-tuned | 23 cards or more | 14–22 cards | 13 cards or fewer |
| --- | ---: | ---: | ---: | ---: |
| Records | | 2,809,328 | 597,808 | 64,548 |
| Points `hu` is worth | | 1.365 ± 0.034 | 1.384 ± 0.042 | 1.010 ± 0.102 |
| Exposed 胡 | 32 | 23.03 ± 0.71 | 24.48 ± 0.94 | 25.96 ± 3.03 |
| `loose` | −128 | −54.56 ± 1.29 | −37.63 ± 1.29 | −26.99 ± 3.84 |
| `short` | −64 | −18.66 ± 0.64 | −16.52 ± 0.77 | −11.12 ± 2.33 |
| `own` | 2 | −1.60 ± 0.42 | −0.19 ± 0.33 | 1.47 ± 1.94 |
| `kept` | 1 | 2.08 ± 0.10 | 1.57 ± 0.13 | 1.44 ± 0.50 |
| `lonely` | −1 | −1.30 ± 0.12 | −2.35 ± 0.23 | −2.45 ± 0.57 |
| `waits` | 8 | 25.96 ± 0.26 | 28.60 ± 0.15 | 34.64 ± 1.24 |

80% of the samples fall at 23 cards or more in the pile: a round averages
about 14.5 discards, and after a claim the seat discards without drawing, so
the pile does not go down with them.

### The choice fit

| Term | Hand-tuned | All choices: points | ± | At 32 | ± | Within 64 behind: points | ± |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Records | | 949,096 | | | | 273,302 | |
| `hu` | 32 | 1.423 | 0.046 | 32.00 | 1.04 | −0.140 | 0.300 |
| `loose` | −128 | −1.985 | 0.068 | −44.65 | 1.53 | −2.998 | 0.677 |
| `short` | −64 | −0.468 | 0.029 | −10.52 | 0.65 | −0.821 | 0.350 |
| `own` | 2 | −0.186 | 0.023 | −4.19 | 0.52 | 0.054 | 0.057 |
| `kept` | 1 | 0.124 | 0.005 | 2.79 | 0.11 | 0.196 | 0.015 |
| `lonely` | −1 | −0.016 | 0.007 | −0.36 | 0.16 | −0.019 | 0.014 |
| `waits` | 8 | 1.060 | 0.021 | 23.83 | 0.48 | 1.684 | 0.114 |

Choices within 64 behind hardly change `hu`, so the `hu` of that column
cannot be pinned down, and the column cannot be converted to 32 either.  In
units where `loose` = −128 instead, `kept` is 8.4, `waits` is 72, `short` is
−35, `own` is 2.3 and `lonely` is −0.8.

### How much worse the other choices are than the bot's

| Grade behind | Choices | Score difference (points) | ± |
| --- | ---: | ---: | ---: |
| 0 | 23,906 | +0.086 | 0.116 |
| 0 to 16 | 174,606 | −0.463 | 0.084 |
| 16 to 64 | 74,790 | −2.604 | 0.126 |
| 64 to 256 | 317,996 | −3.118 | 0.058 |
| 256 or more | 357,798 | −8.709 | 0.082 |

## Results in the bot

All three take `10a046f` as the control, 200,000 deals each, the deal count
set before the run and each seed base the clock at that moment.

| Trial | seed base | Deals that differ | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| Every weight of the hand fit | 1794819512 | 59,981 | −0.005 ± 0.034 | [−0.071, +0.060] | insufficient data |
| Only `WAIT` changed from 8 to 24 | 1794820392 | 2,221 | +0.002 ± 0.007 | [−0.012, +0.017] | insufficient data |
| The choice fit (within 64 behind): the weight of `kept` ×8, `WAIT` 72 | 1794822233 | 35,378 | −0.070 ± 0.025 | [−0.120, −0.021] | loss |

The weights of the first trial (at 32): `hu` 32, `loose` −49, `short` −17,
`own` −2, `kept` 2, `lonely` −1, `waits` 25, exposed 胡 23.  They come from
the same fit run over 200,000 rounds, and equal the table above within the
errors.

The third trial takes, from the right-hand column of the choice fit, the two
terms that differ clearly from the hand-tuned ones once converted at
`loose` = −128.

### Where the third trial loses

The first 60,000 of the same deals were run again (seed base 1794822233) and
sorted by where the candidate bot and the control first decide differently.
These 60,000 deals come to −8,815 points in all.  "Deals in" includes
another seat winning on a card the seat itself drew.

| First split | Deals | Total score difference | A deal | ± |
| --- | ---: | ---: | ---: | ---: |
| Discard: the cards waited on differ | 285 | +224 | +0.79 | 1.69 |
| Discard: the loose cards or the 胡 differ | 4,178 | −2,496 | −0.60 | 0.26 |
| Discard: only `kept`, `own`, `lonely` differ | 2,781 | −344 | −0.12 | 0.39 |
| Claim: the candidate claims, the control declines | 4,324 | −3,205 | −0.74 | 0.31 |
| Claim: the candidate declines, the control claims | 6,708 | −3,027 | −0.45 | 0.26 |
| Claim: a different meld eaten | 1,197 | +33 | +0.03 | 0.49 |

| Outcome for the candidate's seat | Control | Candidate |
| --- | ---: | ---: |
| Wins itself | 4,029 | 3,851 |
| Deals in | 4,220 | 4,174 |
| Another seat wins, not dealt in by this seat | 10,417 | 10,672 |
| Exhausted round or 相公 | 807 | 776 |

The data the fit used are discards "with the same melds, differing only in
which loose card is kept", and there the weights make no visible difference
(−0.12 ± 0.39).  What loses is the decisions it never saw: breaking a meld
to keep a 搭子, and the choice between claiming and not claiming.  With the
weight of a 搭子 made eight times as large, these decisions change with it,
and the seat's own wins fall by 4%.

## How to read it

- **The exchange rate is about 1.4 points for 1 胡.**  The winner collects
  its 胡 from each of the three seats, so at the moment of winning 1 胡 is
  worth 3 points; 1 胡 in hand is not yet cashed, and discounts to 1.4
  points.  With 13 cards or fewer left in the pile it falls to 1.0 points.
- **Route 1 priced defence too dear at the time, not too cheap.**  "Dealing
  in costs 12 points" is 8.5 × 32 in the grade, not the 12 × 32 used then.
  That experiment lost because the deal-ins it dodged were mostly dealt by
  someone else instead; a smaller penalty would only come closer to no
  change.
- **Weights two or three times apart show no difference in strength.**  The
  grade only serves to order the few choices of one hand.  In most
  decisions the best choice is the same under both sets of weights; in the
  30% of deals of the first experiment where play differed, each side won
  some and lost some.
- **The fit says "loose cards are too dear"; it is really the bend of the
  grade.**  In the "other choices" table, 16 to 64 behind loses 2.6 points,
  and only 256 or more behind loses 8.7.  A linear model has a single slope
  to fit with, so it pushes down the terms of large gaps (loose cards, the
  threshold) and lifts those of small gaps (搭子, waiting).
- **The bot's ordering is right in the fine detail too.**  Choices less than
  16 behind already score 0.46 points less on average.  To beat it one has
  to find "which" of the choices behind are in fact better, and a linear
  combination of these seven terms cannot.
- **Weights fitted on one kind of decision do not carry over to another.**
  The grade's one set of weights governs discards, breaking melds and claims
  alike.  The weight of a 搭子 fitted on close discards alone loses when
  used on claims.

## Caveats

- **The opponents are the same bot.**  Both the scores and how good a choice
  is were measured against `HeuristicBot`.
- **A linear model.**  Interactions between terms (for example what a loose
  card is worth while waiting) are not in it.
- **The choice fit plays each choice out once.**  So it can measure the
  average, but not "which choice is best in this one decision"; it also
  looks at discards only, not at claims.
- **The standard error of the first A/B is 0.034.**  30% of deals differ,
  and the noise grows with them; bringing it down to 0.01 would take about
  2.3 million deals.  A difference within ±0.07 points cannot be seen this
  time.
- **The sorting uses the first 60,000 deals.**  The score difference of this
  stretch (−0.147 a deal) is lower than the −0.070 of all 200,000 deals; the
  signs of the classes are within the errors, and only "breaking melds" and
  the two classes of claims are clearly negative.
- **Weights that change with the state of the round were not tried.**  The
  fit by stretches of the pile shows loose cards getting cheaper and waiting
  dearer the later it gets, but replacing the whole set of weights already
  shows no difference, and a version by stretches would need more deals to
  tell apart.
