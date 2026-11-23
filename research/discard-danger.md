# The danger of a discard: which of soldiers, 士象 and 車馬炮 should go first

Measured 2026-11-16.  The question: 「好牌先打」 (good cards first) is there
so that a card the other seats want is not kept until later, only to deal
in.  車馬炮 are plainly the worst cards, but of soldiers and 士象, which is
worth more and more readily waited on, and so should go first?  This is
measured from `HeuristicBot` self-play, base `3526146`.

## Answer

**The order of card value is soldiers ≫ 士象 ≳ 車馬炮, and lone cards go in
that order.**

- **Soldiers are far more dangerous than the other two kinds.**  From the
  middle stage on, a soldier that comes down is won on about twice as often
  as a 士象 or a 車馬炮, and nearly all of it comes from runs (soldiers of
  three and of four colours).
- **士象 and 車馬炮 deal in at about the same total rate, 車馬炮 even a
  little higher, but that is not card value.**  With the wins split by the
  meld the winning card completes, all of the excess of 車馬炮 is in the
  pair waits (單吊) and triplet waits (對倒); in runs 士象 are slightly
  higher.
- **The edge of 士象 in runs is small.**  About 5% higher in the middle
  stage and about 11% in the late stage, far from double, and not
  statistically certain either.
- **The cost of dealing in does not depend on the kind.**  The winner's 胡
  when the card is won on are between 12.1 and 12.3 for all three kinds, so
  comparing the rates is enough.

`HeuristicBot`'s order of lone cards ([`prospects`](../src/heuristic.rs), by
the 胡 of the meld the card would be eaten into: soldiers 3, 將士象 2,
車馬炮 1) agrees with this, and this measurement changed nothing in the bot.

## Why it is so

A card's danger has two sources, to be looked at apart.

**Card value** is how many runs the card can complete, and the rules decide
it:

- Soldiers go together whatever their colours.  A hand with soldiers of two
  colours waits on the other two, 8 cards in all; with three colours it can
  still wait on the fourth.
- 士象 and 車馬炮 go only with cards of their own colour.  將士 waits on 象,
  將象 on 士 and 車馬 on 炮, each on the 4 copies of one card only.
- Nobody would discard a general (將帥), and nobody may.  So a 士象 搭子
  (lacking the general) can eat no discard and can only wait to draw the
  general itself; conversely, whoever holds a general has a use for both
  the 士 and the 象.  This is why 士象 are worth slightly more than 車馬炮.

**Pair waits and triplet waits** are waits any card has; their rate depends
on which singles and pairs the other seats keep in hand, that is, on how the
opponents play and not on the card itself.  `HeuristicBot` throws lone 士象
first and lone 車馬炮 later, so what stays in hand as a pair wait is mostly
車馬炮, and that column runs high.  With another set of opponents the number
would change with them.

So to compare card value, read the "Run" column; the total deal-in rate has
the opponents' habits mixed in.

## Method

Four `HeuristicBot`s play 20,000 rounds under `Rules::new()`, seed base
20261116, the dealer rotating.  At every discard:

1. For each card that is not a general (and of which the other three seats
   do not hold all four copies), check whether another seat could win if it
   came down now, and record whether the winning card completes a pair, a
   triplet (or a kong) or a run.  A card that can be won on in several ways
   counts in each column.  This is "any card that could come down",
   independent of what the discarding seat actually chose.
2. The same check for the card the discarding seat actually put down: this
   is "actual discards".

A win is decided as `Round` decides it: the meld the winning card makes
scores as exposed and the rest of the concealed hand as concealed, and with
the melds already exposed they must reach 10 胡.  "Next seat can eat" asks
only whether the next seat could eat the card into a run, not whether it
would want to.

Stages are set by the cards left in the pile: early with 23 or more, middle
with 14 to 22, late with 13 or fewer (the six dead cards included).

To reproduce:

```sh
cargo run --release --example danger -- 20000 20261116
```

## Data

### Any card that could come down

| Stage | Kind | Sample | Won on | Pair wait | Triplet wait | Run | 胡 | Next seat can eat |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| early | 士象 | 1,763,715 | 0.908% | 0.310% | 0.256% | 0.364% | 12.11 | 24.80% |
| early | 車馬炮 | 2,624,786 | 0.957% | 0.291% | 0.310% | 0.370% | 12.10 | 24.13% |
| early | soldiers | 891,898 | 1.574% | 0.246% | 0.174% | 1.241% | 12.33 | 47.37% |
| middle | 士象 | 389,190 | 3.155% | 1.125% | 0.899% | 1.236% | 12.17 | 19.07% |
| middle | 車馬炮 | 580,358 | 3.387% | 1.198% | 1.094% | 1.177% | 12.22 | 16.87% |
| middle | soldiers | 195,455 | 6.708% | 1.079% | 0.777% | 5.194% | 12.31 | 35.86% |
| late | 士象 | 41,537 | 3.751% | 1.488% | 1.009% | 1.449% | 12.34 | 16.10% |
| late | 車馬炮 | 61,976 | 4.428% | 1.898% | 1.399% | 1.304% | 12.32 | 13.47% |
| late | soldiers | 20,785 | 9.156% | 1.674% | 1.313% | 6.630% | 12.28 | 27.39% |

### Actual discards

| Stage | Kind | Sample | Won on | Pair wait | Triplet wait | Run | 胡 | Next seat can eat |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| early | 士象 | 92,608 | 0.933% | 0.350% | 0.252% | 0.355% | 11.98 | 26.64% |
| early | 車馬炮 | 110,811 | 1.414% | 0.508% | 0.421% | 0.511% | 12.04 | 23.55% |
| early | soldiers | 30,567 | 0.730% | 0.134% | 0.079% | 0.556% | 12.10 | 56.03% |
| middle | 士象 | 16,113 | 4.723% | 1.682% | 1.452% | 1.756% | 12.07 | 20.88% |
| middle | 車馬炮 | 31,342 | 5.242% | 2.007% | 1.697% | 1.678% | 12.16 | 17.86% |
| middle | soldiers | 2,905 | 9.811% | 1.411% | 1.170% | 7.573% | 12.07 | 43.06% |
| late | 士象 | 1,652 | 8.777% | 3.390% | 2.785% | 3.208% | 12.48 | 20.04% |
| late | 車馬炮 | 3,279 | 8.814% | 3.843% | 3.263% | 2.318% | 12.35 | 15.58% |
| late | soldiers | 396 | 14.394% | 3.283% | 1.515% | 10.606% | 12.26 | 31.31% |

"胡" is the winner's mean 胡 when the card is won on (the highest when
several seats can win).

## How to read it

- **Good cards first is right.**  A soldier that comes down in the early
  stage is won on 1.6% of the time, and in the late stage 9.2%; every kind
  is more dangerous the later it goes, and soldiers rise the most.  Among
  actual discards, the soldiers discarded in the early stage dealt in only
  0.73% of the time, the lowest of the three kinds.
- **Actual discards deal in more often than "any card that could come
  down"**, most clearly in the middle and late stages.  A seat discards the
  cards it has no use for, and of the cards it has no use for, fewer are
  usually tied up in the other seats' hands.
- **In the early stage the run rates of the three kinds are nearly the
  same** (士象 0.36%, 車馬炮 0.37%); the gap between 士象 and 車馬炮 only
  appears in the middle stage.
- **Being eaten by the next seat** is highest for soldiers too, about twice
  the other two kinds.  An eat is not a deal-in, but it helps the next seat
  advance its hand.

## Caveats

- **The opponents are the same bot.**  The pair-wait and triplet-wait
  columns reflect what `HeuristicBot` keeps in hand; the run column is
  affected too (throwing lone 士象 early leaves fewer 將士 and 將象 搭子 at
  the table), only less.  People or another bot would give different
  numbers.
- **No confidence intervals.**  The discards of one round test nearly the
  same hands, so the samples are correlated.  Treated as independent
  samples, the late-stage gap in runs between 士象 and 車馬炮 (1.449%
  against 1.304%) is about 1.9 standard errors, and in truth weaker.  The
  gap between soldiers and the other two kinds is too large to be affected.
- **Why the edge of 士象 in runs is so small has not been verified.**  The
  explanation above (a 搭子 lacking its general can eat no discard, and lone
  士象 are thrown early) is only a conjecture.
- **This is danger, not proof of how to play.**  A card kept in hand has
  uses of its own; changing the bot's order of discards still takes
  `ab/run.sh`, as `CLAUDE.md` requires.
