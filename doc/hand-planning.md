# Planning a hand from the deal: the allowance of pairs

Written 2026-11-19.  Mental arithmetic for a person at the table, not a
design for the bot.  The arithmetic comes from
[`research/seven-slots.md`](../research/seven-slots.md) and
[`research/room-for-melds.md`](../research/room-for-melds.md), and the rules
are the 十胡 of `Rules::new`.

## Rules of thumb

1. **車馬炮 and a pong are the baseline.**  All 21 cards at the baseline
   make 7 胡, 3 胡 short.
2. **One lone general (將帥) pays for one pair.**  A general plus a pair is
   3 cards for 1 胡, the same as one 車馬炮.
3. **Pairs may match the lone generals only once the extra 胡 reach 3.**
   將士象 adds 1; soldiers of three colours and a concealed triplet add 2.
   With only 1 or 2, the lone generals must outnumber the pairs by 3; with
   no extra 胡, by 6.
4. **The pairs a hand must hold are "singles % 3".**  Any more come only
   three at a time, and every three cost 2 胡.
5. **The 胡 of 車馬炮 and pongs buy no allowance.**  Many 胡 in hand do not
   mean more pairs may be eaten.

## The arithmetic

A winning hand is always 21 cards.  Write each meld down as three kinds of
number:

| Meld | Cards | 胡 | Counts as |
| --- | ---: | ---: | --- |
| 車馬炮, pong | 3 | 1 | baseline |
| general | 1 | 1 | single +1 |
| pair | 2 | 0 | pair +1 |
| 將士象 | 3 | 2 | extra 胡 +1 |
| soldiers of three colours, concealed triplet | 3 | 3 | extra 胡 +2 |
| soldiers of four colours | 4 | 5 | extra 胡 +3, single +1 |
| exposed kong | 4 | 6 | extra 胡 +4, single +1 |
| concealed quad | 4 | 8 | extra 胡 +6, single +1 |

**胡 = 7 + ⅔ × (singles − pairs) + extra 胡**

- The general in a 將士象 is not a single; it is already counted in the
  extra 胡.
- A four-card meld is a three-card meld plus one single, so each counts as
  one single.
- 21 is a multiple of 3 and three-card melds do not change the remainder, so
  "singles − pairs" is always a multiple of 3.

## The allowance of pairs

十胡 needs "⅔ × (singles − pairs) + extra 胡" to be at least 3:

| Singles − pairs | Extra 胡 at least | Recipe |
| ---: | ---: | --- |
| +6 | 0 | all 車馬炮 and pongs is enough too |
| +3 | 1 | one 將士象 |
| 0 | 3 | 將士象 + soldiers of three colours (or a concealed triplet), three 將士象, or one soldiers of four colours |
| −3 | 5 | 將士象 + two soldiers of three colours / concealed triplets, an exposed kong + 將士象, or one concealed quad |

- **An even number of extra 胡 wastes 1 胡.**  Extra 胡 of 2 meet the same
  requirement as 1, and 4 the same as 3.
- **A hand that can only make 將士象 has almost no allowance.**  With one or
  two 將士象 the lone generals must outnumber the pairs by 3; only with
  three may they be as many.
- **八胡 is only 1 胡 short.**  The same table, with each threshold of extra
  胡 2 lower, and 0 at the least.

## The pairs a hand must hold

Take the singles from the 21 cards, and the rest divide into three-card
melds and pairs:

| Singles | 0 | 1 | 2 | 3 | 4 | 5 | 6 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Pairs it must hold | 0 | 1 | 2 | 0 | 1 | 2 | 0 |

These pairs are no waste; they are the remainder that does not divide.
Pairs beyond them come three at a time: three pairs (6 cards) replace two
three-card melds, for 2 胡 less.

## Examples

Every row is a hand of 21 cards past the threshold:

| Lone generals | Melds with extra 胡 | Pairs | 車馬炮, pongs | 胡 |
| ---: | --- | ---: | ---: | ---: |
| 2 | 將士象 + soldiers of three colours | 2 | 3 | 10 |
| 3 | 將士象 | 0 | 5 | 10 |
| 4 | 將士象 | 1 | 4 | 10 |
| 2 | soldiers of four colours | 3 | 3 | 10 |
| 1 | exposed kong | 2 | 4 | 11 |
| 6 | none | 0 | 5 | 11 |

將士象 + soldiers of three colours is the commonest recipe for extra 胡 in
self-play, 13% of the wins.  The second row cannot hold a single pair.

## Using it at the table

1. **Count the lone generals.**  One you mean to make into 將士象 does not
   count.
2. **Estimate whether the extra 胡 reach 3.**  Look for a concealed triplet,
   a 搭子 of soldiers, 士象 beside a general.
3. **Work out the allowance.**  If the extra 胡 reach 3, the allowance is
   the number of lone generals; if they reach only 1 or 2, it is 3 fewer.
4. **Stop eating once the pairs catch up with the allowance.**  Do not eat
   yet either while the extra 胡 are still not in sight.

## Reminders

- **Eating a pair is the only way to digest lone cards.**  Each meld eaten
  or ponged discards only one card, and the one that wins discards none.
  Lone cards beyond that can only leave two by two through eaten pairs: one
  makes the pair, one is discarded.
- **A pair eaten cannot be taken back.**  A concealed pair in hand can still
  be ponged into a baseline meld; an exposed pair is 0 胡 to the end.
- **Drawing a general changes the remainder.**  A general must be taken.
  With 2 generals and two pairs already eaten, after one more is drawn the
  pairs the hand must hold are 0 or 3, so it can only add a third pair; that
  general's 1 胡 is exactly cancelled by the meld lost.
- **A concealed triplet can only be dealt.**  A drawn card is turned up and
  does not enter the hand; concealed cards only decrease, never increase.

## Evidence and caveats

- **The identity does not depend on play.**  It holds hand by hand for the
  19,404 winning hands of 20,000 rounds of `HeuristicBot` self-play at four
  seats (`examples/census.rs`).
- **The other numbers carry the bot's habits.**  A winning hand averages
  2.76 lone generals and 2.12 pairs; 56% have exactly as many of each, and
  35% three more singles.
- **"No pairs eaten while the 胡 fall short" is a big loss.**  −1.50 points
  a deal, with the win rate falling from 24.20% to 20.33%
  (`research/room-for-melds.md`).  The rule of thumb is "work out the
  allowance, then eat", not "eat less".
- **Not eating pairs beyond the allowance is a win in the bot.**
  `HeuristicBot` does not eat a pair "after which the threshold is out of
  reach unless another general is drawn": +0.053 ± 0.002 points a deal, 95%
  CI [+0.049, +0.056], 800,000 deals (`doc/hu-ceiling.md`, batch 3).  Batch
  2 also refused the pairs "after which the ceiling is lower but the
  threshold still within reach", and that group is −2.2 points a deal.
- **The generals and the extra 胡 of the dealt hand both show in the win
  rate.**  The 20 cards dealt to a seat other than the dealer:

  | Generals | 0 | 1 | 2 | 3 | 4 | 5 |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | Win rate | 11.0% | 15.4% | 19.5% | 22.9% | 26.6% | 29.7% |

  | Extra 胡 of the three-card melds dealt | 0 | 1 | 2 | 3 | 4 | 5 |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | Win rate | 9.2% | 13.8% | 21.0% | 27.0% | 29.3% | 34.3% |

- **These rules of thumb have not been measured on people.**  The arithmetic
  is exact, and the bot's ceiling is more accurate than the allowance worked
  out in the head: it tries every layout.  Whether people win more by
  playing to the rules of thumb has not been measured.
