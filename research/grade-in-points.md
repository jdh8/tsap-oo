# Converting the grade to points: how far normalisation gets

Measured 2026-11-16.  The question: chess engines convert their internal
evaluation to "pawns", so that the number has a fixed meaning.  Can
`HeuristicBot`'s grade be converted to score in the same way?  Once
converted, can "this choice is graded x below the pick" be read as "y points
less"?  The bot's base is `fc6ca57`; this time the `fit` example only prints
a few more tables, and play and the earlier tables are unchanged.

The grade is the whole number the bot sorts by, without a unit.  1 胡 in
hand counts 32 in it and the other terms are weights, so everything below
gives the grade itself, not divided by 32.

## Answer

**A hand's grade converts to points, but by the stage of the round; the
difference in grade between two choices does not convert.**

- **A hand's grade against its score is a monotone curve.**  A hand graded
  −1280 scores −10.9 points on average, 0 is +4.6 points, 320 is +17.8
  points and 640 is +32.9 points.
- **The curve bends near 0.**  From −640 to 0 each 32 of grade is about 0.5
  points, from 0 to 640 about 1.4 points.  A grade below 0 is mostly what
  loose cards take off, and a loose card takes off 128 while really worth
  only 2.2 points.
- **The same grade is worth less the later it is.**  A hand graded 0 is
  worth +8.2 points with 23 cards or more in the pile, −2.0 points with 14
  to 22 and −4.9 points with 13 or fewer.  The conversion has to go by
  stretches.
- **The curve does not tell what a choice costs.**  Used to predict "how
  many points discarding another card instead loses", choices within 64
  behind really lose 3.2 to 3.3 times the prediction; those 64 or more
  behind, 1.2 to 1.3 times.
- **The score difference is not even monotone in the grade difference.**
  Within 8 behind there is no difference (+0.04 ± 0.10 points), 32 to 64
  behind loses 4.1 points, and 64 to 128 behind loses only 1.6 points
  instead.
- **So the number in the web page's hint is a rank, not a price.**  It does
  not convert directly to points, and the bot is unchanged.

## How it was measured

The same run of play as `research/fitted-weights.md`: four `HeuristicBot`s
play 60,000 rounds under `Rules::new()`, seed base 20261119, the dealer
rotating, split into 16 shards.

1. **The curve.**  At every discard of the even shards, record each of the
   four seats' grade and its final score for the round.  The grade is the
   number the bot compares claims by: the exposed 胡 times 32 plus the grade
   of the hand.  In bins of 32, take the mean score of each bin, then pool
   them monotone with pool adjacent violators.  One curve for the whole
   round, and three more by the cards left in the pile.
2. **The check.**  At one discard in eight of the odd shards, every card the
   discarder may discard is played to the end (the data the choice fit of
   `fitted-weights.md` uses).  The curve's prediction is "the points this
   choice's grade maps to − the points the pick's grade maps to", linear
   between bins.  The actual score difference is fitted to the prediction by
   least squares through the origin; a slope of 1 means the curve tells it
   right.
3. **The cost of a choice.**  The same choices over all 16 shards, grouped
   by how far the grade falls behind the pick, taking the mean score
   difference directly.

The curve and the check use different rounds.  The standard errors come
from the spread between shards.

To reproduce (about 17 minutes on 16 cores):

```sh
cargo run --release --example fit -- 60000 20261119
```

## Data

### What a hand's grade is worth in points

Each bin is the hands "graded from this number up to 32 above it", and the
number is the mean score after pooling monotone.  The full table, a row for
every 32, is printed by the command above.

| Grade | Whole round | 23 cards or more | 14–22 cards | 13 cards or fewer |
| ---: | ---: | ---: | ---: | ---: |
| −1280 or below | −10.87 | −10.87 | −11.23 | −10.88 |
| −960 | −7.93 | −7.82 | −10.81 | −8.06 |
| −640 | −4.68 | −4.33 | −10.08 | −8.06 |
| −320 | −0.50 | +0.86 | −7.16 | −7.14 |
| −160 | +1.91 | +4.32 | −4.76 | −6.15 |
| 0 | +4.57 | +8.18 | −1.96 | −4.94 |
| 160 | +11.42 | +14.99 | +6.84 | +2.40 |
| 320 | +17.82 | +23.64 | +14.27 | +7.59 |
| 480 | +23.10 | +27.16 | +21.23 | +11.25 |
| 640 | +32.91 | +37.70 | +28.62 | +17.25 |

The grade at which the score is 0: about −350 in the early stretch, about
+30 in the middle one and about +100 in the late one.

Points for each 32 of grade (two rows of the table above subtracted):

| Interval | Whole round | 23 cards or more | 14–22 cards | 13 cards or fewer |
| --- | ---: | ---: | ---: | ---: |
| −640 to 0 | 0.46 | 0.63 | 0.41 | 0.16 |
| 0 to 640 | 1.42 | 1.48 | 1.53 | 1.11 |

Samples at 13 cards or fewer are few (about thirty thousand records in the
even shards, and only a few dozen to two hundred in each bin at −480 or
below), and that column pools to a single value at −480 or below.

### The curve's prediction of what a choice costs

The 501,555 choices of the odd shards.  "Predicted" is the score difference
the curves by stretch give, and "slope" is the slope of actual against
predicted; the two rightmost columns use the whole-round curve instead.

| Grade behind | Choices | Mean behind | Actual (points) | ± | Predicted | Slope | ± | Predicted by the whole-round curve | Slope | ± |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 12,207 | 0 | +0.365 | 0.149 | 0 | | | 0 | | |
| 0 to 16 | 92,586 | 8 | −0.487 | 0.140 | −0.160 | 3.18 | 0.29 | −0.132 | 3.27 | 0.35 |
| 16 to 64 | 41,248 | 29 | −2.586 | 0.214 | −0.564 | 3.29 | 0.22 | −0.476 | 3.42 | 0.27 |
| 64 to 256 | 168,970 | 165 | −3.104 | 0.072 | −2.527 | 1.31 | 0.04 | −2.179 | 1.52 | 0.04 |
| 256 or more | 186,544 | 371 | −8.704 | 0.138 | −7.313 | 1.18 | 0.02 | −6.368 | 1.35 | 0.02 |
| All | 501,555 | 197 | −4.612 | 0.129 | −3.647 | 1.20 | 0.02 | −3.166 | 1.37 | 0.03 |

### How far a choice is behind the pick, and what it really loses

The 949,096 choices of all 16 shards.

| Grade behind | Choices | Mean behind | Score difference (points) | ± | Points lost for each 32 behind |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 23,906 | 0 | +0.086 | 0.116 | |
| 0 to 8 | 106,420 | 5 | +0.036 | 0.099 | |
| 8 to 16 | 68,186 | 13 | −1.239 | 0.128 | 3.0 |
| 16 to 32 | 51,238 | 22 | −1.855 | 0.200 | 2.7 |
| 32 to 64 | 23,552 | 45 | −4.060 | 0.199 | 2.9 |
| 64 to 128 | 72,299 | 87 | −1.643 | 0.077 | 0.6 |
| 128 to 256 | 245,697 | 188 | −3.552 | 0.063 | 0.6 |
| 256 to 512 | 335,709 | 357 | −8.306 | 0.075 | 0.7 |
| 512 or more | 22,089 | 581 | −14.759 | 0.262 | 0.8 |

By stretch of the pile (score difference, points):

| Grade behind | 23 cards or more | ± | 14–22 cards | ± | 13 cards or fewer | ± |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | +0.023 | 0.141 | +0.277 | 0.368 | +2.144 | 0.807 |
| 0 to 8 | +0.044 | 0.099 | +0.027 | 0.175 | −1.425 | 0.582 |
| 8 to 16 | −1.193 | 0.130 | −1.635 | 0.255 | −2.526 | 0.900 |
| 16 to 32 | −1.691 | 0.213 | −3.174 | 0.460 | −2.815 | 1.245 |
| 32 to 64 | −3.814 | 0.227 | −5.341 | 0.751 | −4.915 | 1.901 |
| 64 to 128 | −1.623 | 0.081 | −1.963 | 0.313 | −1.004 | 0.495 |
| 128 to 256 | −3.601 | 0.070 | −3.198 | 0.134 | −2.413 | 0.467 |
| 256 to 512 | −8.203 | 0.087 | −8.933 | 0.158 | −6.990 | 0.441 |
| 512 or more | −14.005 | 0.253 | −16.805 | 0.586 | −12.487 | 1.246 |

"32 to 64 behind costs more than 64 to 128 behind" shows in all three
stretches.

## How to read it

- **There are two exchange rates, not one.**  Between different hands, 32 of
  grade is worth 0.2 to 1.5 points; between two plays of the same hand,
  each 32 is worth about 3 points at 8 to 64 behind and only 0.6 to 0.8
  points at 64 or more behind.  The "1 胡 is about 1.4 points" of
  `fitted-weights.md` is the price of the term `hu`, close to the slope of
  the curve above 0.
- **The bend comes from the pricing of loose cards.**  A loose card takes
  128 off the grade and is fitted at only 2.2 points, that is 0.55 points
  for each 32.  Choices 64 to 256 behind have mostly kept one or two loose
  cards more, so each 32 loses only 0.6 points.
- **Small gaps are dear because they come from cheap terms.**  Choices 8 to
  64 behind differ in 搭子 and in the cards waited on.  A live card the
  hand wins on is 8 in the grade and is fitted at 1.1 points, 4.4 points for
  each 32.
- **A monotone conversion cannot fix the ordering.**  Converting the grade
  to points does not change which choice comes first.  When the terms of
  the grade are priced wrongly against each other (for example a waiting
  hand that pongs for one loose card fewer, its wait falling from 8 cards to
  3), the same one is still picked after the conversion.
- **Within 8 can be taken as a tie.**  This bin has a hundred thousand
  choices, with a score difference of +0.04 ± 0.10.
- **The mean in `HeuristicBot::draw` is taken over grades.**  The curve
  bends, so converting to points first and averaging after would give
  different claim decisions.  This is the only place where normalisation
  would change play, and it has not been tried; the curve is inaccurate on
  choices, so there is no reason to expect it to win.

## An example of a claim

8 胡, 3 胡 on the table, and in hand five generals (將帥), red 炮 red 炮,
yellow 兵 and green 卒; the 8 胡 are already there, and it waits on red 兵
and white 卒, 8 cards at most.  The next seat draws a red 炮.  The web
page's hint: pong is the pick, and pass is about 77 behind.

| | Pass | Pong |
| --- | --- | --- |
| Loose cards | yellow 兵, green 卒 | one of the two |
| Waits on | 8 cards at most | 3 cards at most |
| On winning | 11 胡 | 9 胡 |

The grade picks the pong because one loose card fewer saves 128 and the 1 胡
more of the pong adds 32, while waiting on 5 more cards is worth only 40.
This is exactly the case of the table above: the 32 to 64 that lies in the
cards waited on is dearer than the 64 to 128 that lies in loose cards.
Converting to points does not change this choice.

Making "a waiting hand does not claim into a wait on fewer cards" a rule was
tried once: `fc6ca57` as the control, `ab/run.sh 200000`, seed base
1794828275, 3,607 deals differ, +0.007 ± 0.011 points a deal, 95% CI
[−0.014, +0.029], win rate 24.23% against 24.19%.  Insufficient data; the
bot is unchanged.

## Caveats

- **The opponents are the same bot.**  Both the curve and the costs were
  measured against `HeuristicBot`.
- **The choices are discards only.**  The choice between claiming and
  passing was never compared by playing to the end, and the tables above do
  not hold for that kind of decision.
- **Each choice is played out once.**  That measures the mean of a group of
  choices, not which is better in a single decision.
- **The curve uses two variables only, the grade and the pile.**  The
  opponents' progress (their concealed cards, their exposed 胡) is not in
  it; the choices of one decision share these, so the check is not affected.
- **The late stretch has few samples.**  A pile of 13 cards or fewer
  accounts for only about 2% of the hands.
