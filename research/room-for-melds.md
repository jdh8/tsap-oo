# How many melds a hand has room for: the ブロック理論 of 十胡

Measured 2026-11-18.  The question: the concealed cards of 十胡 only go
down, never up, so what shape a hand can end in, and how many 胡 it can
reach, is bounded from the deal.  All that floats is a 搭子 eaten into a
meld, a lone card eaten into a pair (0 胡) and a pair ponged (1 胡).
Japanese mahjong's 「5ブロック理論」 (five-block theory) says a hand has room
for only five melds, so surplus 搭子 should be broken early.  If
`HeuristicBot` counts in the same way how many melds a 十胡 hand still has
room for, eats fewer pairs early and keeps the room for melds that score 胡,
does it get stronger?  The bot's base is `b894777`.

## Answer

**The arithmetic is right, yet none of the four trials won; the bot is
unchanged.**

- **Counting "room for how many melds" alone shows no difference.**  Capping
  the 胡 the 搭子 may yet supply at the number of three-card melds the hand
  still has room for: +0.005 ± 0.004 points a deal, insufficient data.  It
  changes play in only 6% of deals.
- **Counting the 1 胡 of a ponged pair loses.**  −0.22 to −0.29 points a
  deal.  Once the 胡 hoped for exceed the room, the bot breaks melds that
  score 胡 to eat pairs, for the sake of two loose cards fewer, and ends up
  eating more of them.
- **Never eating a pair below the threshold loses heavily.**  −1.50 points a
  deal; the win rate falls from 24.20% to 20.33%.  Eating pairs is the bot's
  main way of getting rid of lone cards, and pays well on average.
- **The pair eats at the margin do lose, but they are few.**  The pairs the
  cap keeps the bot from eating are worth +0.26 ± 0.12 points each, and come
  up once in fifty deals.  That is a number from groups made after the fact,
  not a verdict.

## The arithmetic

A drawn card is turned face up and never enters the hand; the concealed
cards only go down, at an eat, a pong, a kong or a discard.  So the final 21
cards are the dealt concealed melds, plus exposed melds that each hold
exactly one card from outside.

Lay the concealed cards out as melds, and O cards lie outside them: the
loose cards, plus the cards of the pairs.  From then on each claim takes one
card in and discards one; only the claim that wins discards none.  Say these
O cards end up as u three-card melds (a 搭子 eaten into a meld, a pair
ponged), b pairs made by eating and r pairs kept in hand:

- Concealed cards used: 2u + b + 2r, plus the d cards discarded, O in all.
- There are u + b claims, one of which is the win, so d = u + b − 1.

Eliminating d between the two:

**O + 1 = 3u + 2(b + r)**

The cards outside the melds, plus the winning card, split exactly into
three-card melds and pairs.  Hence:

- **At most (O + 1) ÷ 3 three-card melds, rounded down.**  21 cards make at
  most 7 melds: that is the ブロック limit of 十胡.
- **The cards left over must be even in number.**  When O is a multiple of
  3, O + 1 leaves a remainder of 1 on division by 3; that one card cannot
  make a pair, so the limit drops by 1 more.  "A 搭子 and a lone card"
  (O = 3) makes no meld at all: eating into the 搭子 means discarding the
  lone card, and the hand is all melds without having won.
- **Eating a pair takes 2 off O.**  O with a remainder of 2 on division by 3
  is the tightest shape (two 搭子 and a lone card, exactly two melds);
  eating a pair then drops the limit by 2 at once.  With a remainder of 0 or
  1 the hand has to make a pair anyway, and eating one does not lower the
  limit.

| O | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Three-card melds at most | 0 | 0 | 1 | 0 | 1 | 2 | 1 | 2 | 3 | 2 | 3 | 4 |

This arithmetic leaves luck out: a drawn general (將帥; taken by force, 1 胡
more and one more discard), a concealed triplet konged, and soldiers of
three colours eating the fourth all shift the remainder of O.  Nor does it
count breaking a ready meld to make the numbers fit.

## What the bot counted before

`HeuristicBot`'s grade has a term `short`: how many half-胡 the melds' 胡
are short of the threshold, 64 off for each.  It first takes off `hope`, the
胡 the 搭子 among the loose cards may yet supply.  `hope` as it stood had two
gaps:

- **No cap.**  For a hand of two 搭子 and no lone card, `hope` counts both
  as eaten into melds; by the arithmetic only one can be.
- **No pairs.**  A pair is a meld of 0 胡 in the layout; the 1 胡 more that
  ponging it brings is not in `hope`.

On the other hand, eating a pair is progress in the grade: two loose cards
fewer (the one eaten into a pair and the one discarded after the eat), 128
saved for each, the same as eating into a meld that scores 胡.

## The four trials

The first three change the `hope` that `prospects` computes, and nothing
else:

| Name | Cap on 搭子 | Remainder condition | Hope for pongs of pairs |
| --- | --- | --- | --- |
| Lenient | yes | no | yes |
| Strict | yes | yes | yes |
| Cap only | yes | yes | no |

- Each card of a 搭子 keeps its half-胡 as before; each card of a pair that
  can be ponged counts 1 half-胡 (only while a copy of it is still unseen).
  Sorted from the largest down, only the first 2 × limit cards count.
- O is that layout's loose cards plus the cards of its pairs; when a 士 or
  象 leans on the hand's own general, that general counts as one card too,
  since it will leave its own meld to make 將士象.
- `research/room-for-melds.patch` is "strict", against `src/heuristic.rs` at
  `b894777`.  "Lenient" replaces the line that computes `room` with
  `let room = (outside + 1) / 3;`; "cap only" deletes the line
  `hopes.extend([pong; 2]);`.  For the latter two the unit tests' expected
  values have to change to match.

The fourth leaves the grade alone:

- **No pair eats.**  `research/room-for-melds-nopair.patch`.  While the 胡
  of the melds in hand (layout one, the one that charges 1 for each loose
  card) are still short of the threshold, `weighed` does not consider eating
  a pair; once they are enough it plays as before.

The A/B is `ab/run.sh 100000 b894777`, the candidate in the second seat, the
pile reshuffled 8 times from where the two part
([`reshuffle.md`](reshuffle.md)).  Every deal count was set before the run,
and each seed base is the clock at that moment.  Two ablations of "lenient"
(pairs only, cap only) were scheduled at first as well; they were cancelled
before the start and replaced by "strict" and "no pair eats"; "cap only" was
added only after the first three had been read.

To reproduce (taking "strict" as the example):

```sh
git apply research/room-for-melds.patch
SEED_BASE=1794934225 ab/run.sh 100000 b894777
```

## Data

| | Lenient | Strict | Cap only | No pair eats |
| --- | ---: | ---: | ---: | ---: |
| seed base | 1794933710 | 1794934225 | 1794936088 | 1794934919 |
| Deals | 100,000 | 100,000 | 100,000 | 100,000 |
| Deals that part | 28,275 | 33,313 | 6,093 | 52,775 |
| Score difference (points a deal) | −0.290 ± 0.014 | −0.216 ± 0.014 | +0.005 ± 0.004 | −1.497 ± 0.020 |
| 95% CI | [−0.317, −0.263] | [−0.244, −0.189] | [−0.003, +0.014] | [−1.536, −1.458] |
| Verdict | loss | loss | insufficient data | loss |
| Win rate, control → candidate | 24.40% → 23.81% | 24.27% → 23.82% | 24.21% → 24.22% | 24.20% → 20.33% |
| Mean over the deals that part (points) | −1.027 | −0.650 | +0.083 | −2.837 |
| Standard deviation within one position (points) | 16.44 | 16.08 | 12.65 | 17.91 |
| Standard deviation between positions (points) | 5.70 | 5.20 | 3.31 | 5.44 |

## Where it loses

By the iron rule, a loss is traced before it is declared.
`research/room-for-melds-trace.patch` makes `ab/` print every split under
`TRACE=1` (the seed, each side's decision, the hand, the mean score
difference over the 8 reshuffles), and adds `ab/tally.py`, which groups them
by "the control's decision → the candidate's decision":

```sh
git apply research/room-for-melds-trace.patch
TRACE=1 SEED_BASE=1794934225 ab/run.sh 20000 b894777 > out.txt
python3 ab/tally.py out.txt 20000
```

The first three tables each take the first 20,000 deals of their A/B (the
same seed base); "cap only" is all 100,000 deals.  These are the very deals
the verdict was given on, and the groups were made after seeing the results,
so the rows serve only to look for causes and to pick the next experiment.

### Lenient

First 20,000 deals: 5,568 deals part, −0.321 ± 0.031 points a deal.

| Control → candidate | Deals | Mean (points) | ± | Contribution a deal (points) |
| --- | ---: | ---: | ---: | ---: |
| pass → eat pair | 2,332 | −1.66 | 0.16 | −0.1934 |
| pass → pong | 244 | −4.71 | 0.67 | −0.0575 |
| pong → pass | 682 | −1.14 | 0.28 | −0.0390 |
| eat soldiers → pass | 108 | −2.07 | 1.19 | −0.0112 |
| eat soldiers → eat soldiers | 68 | −2.60 | 1.35 | −0.0088 |
| eat soldiers → eat pair | 28 | −5.73 | 2.05 | −0.0080 |
| discard → discard | 1,037 | −0.12 | 0.30 | −0.0062 |
| pass → eat soldiers | 29 | −3.91 | 1.86 | −0.0057 |
| eat 車馬炮 → pass | 241 | −0.38 | 0.38 | −0.0046 |
| eat 車馬炮 → eat pair | 125 | −0.18 | 0.62 | −0.0012 |
| eat 將士象 → eat 將 | 16 | −1.22 | 1.42 | −0.0010 |
| eat 將士象 → pass | 226 | −0.08 | 0.51 | −0.0010 |
| pass → eat 車馬炮 | 6 | −1.15 | 0.80 | −0.0003 |
| pass → eat 將士象 | 44 | −0.11 | 0.97 | −0.0003 |
| eat 將士象 → pong | 3 | −1.08 | 1.08 | −0.0002 |
| eat pair → eat 車馬炮 | 2 | +0.00 | 0.00 | +0.0000 |
| eat 將士象 → eat pair | 29 | +0.47 | 1.38 | +0.0007 |
| pong → eat 將士象 | 114 | +1.25 | 0.69 | +0.0071 |
| eat pair → pass | 234 | +0.78 | 0.32 | +0.0091 |

### Strict

First 20,000 deals: 6,653 deals part, −0.187 ± 0.032 points a deal.

| Control → candidate | Deals | Mean (points) | ± | Contribution a deal (points) |
| --- | ---: | ---: | ---: | ---: |
| pass → eat pair | 1,970 | −1.28 | 0.17 | −0.1260 |
| pass → pong | 202 | −3.58 | 0.74 | −0.0361 |
| pong → pass | 608 | −1.18 | 0.32 | −0.0360 |
| eat soldiers → pass | 118 | −3.75 | 0.87 | −0.0221 |
| eat soldiers → eat pair | 21 | −7.35 | 2.36 | −0.0077 |
| eat 車馬炮 → eat pair | 122 | −0.71 | 0.58 | −0.0043 |
| pass → eat soldiers | 22 | −2.66 | 2.03 | −0.0029 |
| pass → eat 將士象 | 41 | −1.11 | 1.18 | −0.0023 |
| eat soldiers → eat soldiers | 63 | −0.69 | 1.29 | −0.0022 |
| eat 將士象 → eat 將 | 8 | −0.44 | 2.82 | −0.0002 |
| eat pair → eat soldiers | 1 | +0.25 | | +0.0000 |
| eat 車馬炮 → pong | 3 | +0.42 | 0.87 | +0.0001 |
| eat 將士象 → pong | 2 | +3.44 | 2.94 | +0.0003 |
| eat pair → eat 車馬炮 | 6 | +2.12 | 1.84 | +0.0006 |
| pass → eat 車馬炮 | 15 | +1.08 | 1.30 | +0.0008 |
| eat 將士象 → eat pair | 49 | +0.64 | 0.89 | +0.0016 |
| eat 車馬炮 → pass | 256 | +0.45 | 0.34 | +0.0057 |
| eat pair → pass | 900 | +0.19 | 0.19 | +0.0086 |
| pong → eat 將士象 | 136 | +1.50 | 0.65 | +0.0102 |
| eat 將士象 → pass | 231 | +0.88 | 0.61 | +0.0102 |
| discard → discard | 1,879 | +0.16 | 0.19 | +0.0148 |

Two examples of "pass → eat pair" (the concealed cards by red, yellow,
green, white, then the exposed melds):

- seed 1794934260: `帥仕相兵兵.仕仕俥傌炮兵兵.將象車車包.車包`, the red
  帥 exposed.  A green 包 comes; the control passes, and the candidate takes
  the 包 out of the green `車車包` to eat it into `包包`, −6.50 points.
- seed 1794934230: `帥仕相俥俥傌傌炮兵兵.兵.士象車馬馬包包.將將`.  A red
  相 comes; the control passes, and the candidate takes the 相 out of the
  red `帥仕相` to eat it into `相相`, +1.13 points.

### Cap only

All 100,000 deals: 6,093 deals part, +0.005 ± 0.004 points a deal.

| Control → candidate | Deals | Mean (points) | ± | Contribution a deal (points) |
| --- | ---: | ---: | ---: | ---: |
| pong → pass | 517 | −1.06 | 0.27 | −0.0055 |
| pass → eat pair | 908 | −0.46 | 0.14 | −0.0042 |
| eat 車馬炮 → eat pair | 46 | −0.66 | 0.63 | −0.0003 |
| pass → eat soldiers | 16 | −1.85 | 1.09 | −0.0003 |
| pong → eat 將士象 | 34 | −0.06 | 0.88 | −0.0000 |
| pass → eat 車馬炮 | 31 | +0.07 | 1.19 | +0.0000 |
| eat pair → eat soldiers | 4 | +0.62 | 0.38 | +0.0000 |
| eat soldiers → eat soldiers | 3 | +1.17 | 0.23 | +0.0000 |
| pong → eat 車馬炮 | 1 | +3.75 | | +0.0000 |
| eat 將士象 → eat pair | 13 | +0.53 | 0.39 | +0.0001 |
| eat soldiers → pass | 3 | +3.67 | 1.79 | +0.0001 |
| eat 將士象 → pong | 4 | +3.22 | 3.13 | +0.0001 |
| pass → eat 將士象 | 58 | +0.25 | 0.58 | +0.0001 |
| eat 將士象 → eat 將 | 16 | +1.06 | 2.03 | +0.0002 |
| eat 車馬炮 → pass | 223 | +0.08 | 0.28 | +0.0002 |
| pass → pong | 116 | +0.15 | 0.48 | +0.0002 |
| eat pair → eat 車馬炮 | 29 | +4.43 | 2.39 | +0.0013 |
| eat 將士象 → pass | 107 | +1.47 | 0.54 | +0.0016 |
| eat pair → pass | 1,949 | +0.26 | 0.12 | +0.0050 |
| discard → discard | 2,015 | +0.32 | 0.14 | +0.0064 |

### No pair eats

First 20,000 deals: 10,645 deals part, −1.416 ± 0.043 points a deal.

| Control → candidate | Deals | Mean (points) | ± | Contribution a deal (points) |
| --- | ---: | ---: | ---: | ---: |
| eat pair → pass | 9,833 | −2.77 | 0.08 | −1.3630 |
| pass → eat 將士象 | 158 | −1.88 | 0.51 | −0.0149 |
| pass → eat 車馬炮 | 202 | −1.15 | 0.35 | −0.0116 |
| eat pair → eat 車馬炮 | 80 | −2.47 | 0.85 | −0.0099 |
| pass → pong | 305 | −0.59 | 0.38 | −0.0090 |
| eat pair → eat 將士象 | 31 | −4.71 | 1.38 | −0.0073 |
| pass → eat soldiers | 33 | −0.46 | 1.56 | −0.0008 |
| eat pair → eat soldiers | 3 | +2.75 | 5.10 | +0.0004 |

## How to read it

- **The two versions that count pongs of pairs lose by eating more pairs.**
  "Pass → eat pair" is 60% of the loss of "lenient" and two thirds of that
  of "strict".  Both examples above take a card out of a ready meld or a
  搭子 to eat a pair.  The presumed mechanism: once the cards of the pairs
  enter `hope`, the 胡 hoped for exceed the room, breaking a 搭子 no longer
  makes `short` grow, and the 256 for two loose cards fewer wins out.  This
  mechanism is read off the examples only and was not measured apart; but
  with the hope for pongs of pairs taken out, "pass → eat pair" falls from
  −0.126 a deal to −0.004.
- **The choice of pongs was the more accurate to begin with.**  In the two
  versions that count pongs of pairs, ponging more (−3.6 to −4.7 points) and
  ponging less (−1.1 to −1.2 points) are both negative.
- **Eating a pair pays well on average.**  "No pair eats" blocks the first
  pair in half the deals and loses 2.77 points each time.
- **The pairs the cap blocks are a different group.**  "Eat pair → pass" is
  +0.26 ± 0.12 points each time under "cap only", +0.19 ± 0.19 points under
  "strict" and +0.78 ± 0.32 points under "lenient", positive all three
  times.  Eating fewer of the pairs the arithmetic says lower the limit
  indeed does no harm, and may do a little good; only there are just 1,949
  of them in 100,000 deals.
- **"Cap only" loses in places too.**  Ponging less (−1.06 ± 0.27 points)
  and eating more pairs (−0.46 ± 0.14 points) come to −0.010 a deal
  together, cancelling 60% of the +0.015 the other rows add up to.

## Caveats

- **The groups are after the fact.**  The sign of each row was read on the
  same deals the verdict was given on and is no verdict; a rule designed
  from them has to be measured again on a fresh seed base.
- **The deals of the four experiments overlap heavily.**  The seed bases are
  a few hundred to a little over two thousand apart and the deal count is
  100,000, so the four numbers are not four independent checks.
- **Only the first split of each deal is recorded.**  Every later difference
  is charged to the first split, so it cannot be seen whether eating a pair
  costs more early or late.
- **The cap enters the term `short` alone.**  Each half-胡 costs 64, and
  eating a pair saves 256 for two loose cards fewer.  The arithmetic says
  "cannot be made"; the grade only takes it as "a little further off"; the
  weights were not touched, and no other weights were tried.
- **The arithmetic leaves luck out.**  A drawn general, a kong, or soldiers
  of four colours can each make a shape that cannot be made, so the limit is
  on the tight side; by how much was not measured.
- **The limit is computed for one layout only.**  The bot lays the hand out
  twice (weighing the 胡 and weighing the cards), caps each by itself, and
  does not look for the layout with the highest limit.  The 胡 of a
  concealed triplet konged and of soldiers of three colours eating the
  fourth are not in `hope` either.
- **The opponents are the same bot.**  The control and the other two seats
  are all `HeuristicBot`.
- **The "strict" patch makes one test fail.**  `heuristic::tests::shanten`
  ends by checking that every 向聽數 came up in its three rounds; with play
  changed, "two away" did not come up.  That is a check on the sample, not a
  wrong 向聽數.

## Not yet tried

- **The cap used only to refuse pair eats, leaving passes, pongs and
  discards alone.**  The "eat pair → something else" rows of "cap only" add
  up to about +0.006 points a deal; that is an upper bound picked out after
  the fact, and will most likely shrink on other deals.  Telling a
  difference this small apart needs 400,000 deals or more declared first.
- **Treating "cannot be made" as a dead hand rather than a deduction.**
  When the limit, even with the generals' help, still cannot reach the
  threshold, handle the hand as `dead` outright.
