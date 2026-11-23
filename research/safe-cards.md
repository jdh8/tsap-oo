# Safe cards: how much 「不能過水」 (no passing up a win) proves

Measured 2026-11-16.  The question: winning is compulsory, so a card that
comes down and nobody wins on proves that no seat asked could win on it at
the time.  How many of the discards to choose from does this proof cover?
And what sets the danger of a card not proven safe?  This is measured from
`HeuristicBot` self-play; the bot's base is `0b5d48f`, and `View::safe` is
new with this measurement.

## Answer

**The proof is sound but sparse; what really sets the danger is how many
concealed cards the opponent has left.**

- **A card proven safe was never won on.**  Over 20,000 rounds and about
  8.48 million (candidate discard, opponent) combinations, the assert in
  the example never fired.
- **Only about 5% of the combinations are proven safe.**  Every row lies
  between 2.7% and 6.6%.  Each time an opponent eats or pongs, every proof
  against it is void, and `HeuristicBot` eats and pongs a great deal.
- **With the safe cards taken out, the rest are only a little more
  dangerous** (2.44% becomes 2.58%, for example), because only a twentieth
  was taken out.
- **The opponent's concealed card count spreads the danger two
  hundredfold.**  While the opponent still holds 19 to 20 cards, a card not
  proven safe is won on by it 0.02% of the time; with 8 or fewer left, 3%
  to 5%.
- **Soldiers are about 1.5 times 士象 and 車馬炮 in every row**, in
  agreement with `discard-danger.md`.

## Why the proof holds

`Round::resolve` checks every public card for a win first: a draw is offered
to all four seats, a discard to the other three, and the round ends at once
if anyone can win.  So a round still in play means that none of those asked
could win at the time.

Whether a seat can win on a card depends only on its concealed cards, its
exposed melds and the rules.  A drawn card never enters a hand; concealed
cards only decrease, on an eat, a pong or a kong, and exposed melds only
increase then.  So "cannot win on it" holds until that seat's next eat, pong
or kong.  After that its concealed cards have changed and its threshold is
lower too, a card it could not win on may become one it can, and the proof
must be voided whole.

## Method

Four `HeuristicBot`s play 20,000 rounds under `Rules::new()`, seed base
20261117, the dealer rotating.  At every discard, each card the discarding
seat could put down (without repeats) is paired with each of the other three
seats, and two things are recorded:

1. Whether `View::safe` proves that this seat cannot win on it.
2. Whether this seat can in fact win on it, decided as `Round` decides it.

Rows group by the concealed card count of the seat tested.  "Actual
discards" takes only the card the discarding seat really put down.

To reproduce (the bot must be `0b5d48f`; later bots play differently, and
the numbers will differ slightly):

```sh
cargo run --release --example safe -- 20000 20261117
```

## Data

### Any card that could be discarded, against a single opponent

| Opponent's concealed cards | Kind | Combinations | Proven safe | Won on (all) | Won on (not proven) |
| --- | --- | ---: | ---: | ---: | ---: |
| 20–19 | 士象 | 915,517 | 4.64% | 0.016% | 0.017% |
| 20–19 | 車馬炮 | 1,428,488 | 2.68% | 0.017% | 0.017% |
| 20–19 | soldiers | 441,415 | 6.49% | 0.025% | 0.027% |
| 18–17 | 士象 | 689,919 | 5.55% | 0.135% | 0.143% |
| 18–17 | 車馬炮 | 1,128,166 | 4.35% | 0.125% | 0.130% |
| 18–17 | soldiers | 338,110 | 5.82% | 0.187% | 0.198% |
| 16–15 | 士象 | 325,947 | 5.48% | 0.468% | 0.496% |
| 16–15 | 車馬炮 | 537,808 | 4.92% | 0.513% | 0.540% |
| 16–15 | soldiers | 170,232 | 4.69% | 0.815% | 0.855% |
| 14–13 | 士象 | 363,346 | 5.57% | 0.817% | 0.865% |
| 14–13 | 車馬炮 | 598,204 | 5.18% | 0.871% | 0.919% |
| 14–13 | soldiers | 192,950 | 4.87% | 1.370% | 1.440% |
| 12–11 | 士象 | 237,165 | 5.68% | 1.634% | 1.732% |
| 12–11 | 車馬炮 | 382,858 | 5.52% | 1.602% | 1.696% |
| 12–11 | soldiers | 131,547 | 4.84% | 2.359% | 2.479% |
| 10–9 | 士象 | 109,081 | 5.54% | 2.440% | 2.584% |
| 10–9 | 車馬炮 | 171,789 | 5.46% | 2.508% | 2.652% |
| 10–9 | soldiers | 62,027 | 4.59% | 3.798% | 3.981% |
| 8–7 | 士象 | 57,816 | 5.52% | 2.977% | 3.151% |
| 8–7 | 車馬炮 | 90,157 | 5.61% | 3.200% | 3.390% |
| 8–7 | soldiers | 33,268 | 4.98% | 4.638% | 4.881% |
| 6–5 | 士象 | 19,291 | 5.94% | 3.012% | 3.202% |
| 6–5 | 車馬炮 | 29,546 | 5.65% | 3.445% | 3.652% |
| 6–5 | soldiers | 11,457 | 5.27% | 4.678% | 4.939% |
| 4–3 | 士象 | 3,633 | 5.34% | 3.716% | 3.926% |
| 4–3 | 車馬炮 | 5,501 | 5.36% | 3.581% | 3.784% |
| 4–3 | soldiers | 2,170 | 5.99% | 4.286% | 4.559% |
| 2–0 | 士象 | 620 | 5.97% | 2.903% | 3.087% |
| 2–0 | 車馬炮 | 829 | 4.58% | 3.136% | 3.287% |
| 2–0 | soldiers | 256 | 6.64% | 2.734% | 2.929% |

### Actual discards, against a single opponent

| Opponent's concealed cards | Kind | Combinations | Proven safe | Won on (all) | Won on (not proven) |
| --- | --- | ---: | ---: | ---: | ---: |
| 20–19 | 士象 | 103,851 | 5.27% | 0.013% | 0.014% |
| 20–19 | 車馬炮 | 75,212 | 6.94% | 0.023% | 0.024% |
| 20–19 | soldiers | 53,184 | 2.28% | 0.017% | 0.017% |
| 18–17 | 士象 | 82,791 | 6.88% | 0.129% | 0.139% |
| 18–17 | 車馬炮 | 102,226 | 8.23% | 0.130% | 0.142% |
| 18–17 | soldiers | 22,706 | 2.72% | 0.159% | 0.163% |
| 16–15 | 士象 | 40,234 | 7.12% | 0.403% | 0.434% |
| 16–15 | 車馬炮 | 65,827 | 8.31% | 0.536% | 0.585% |
| 16–15 | soldiers | 7,233 | 3.59% | 0.705% | 0.731% |
| 14–13 | 士象 | 45,454 | 7.21% | 0.713% | 0.768% |
| 14–13 | 車馬炮 | 79,323 | 8.29% | 0.898% | 0.979% |
| 14–13 | soldiers | 8,058 | 4.17% | 1.216% | 1.269% |
| 12–11 | 士象 | 31,094 | 7.54% | 1.476% | 1.596% |
| 12–11 | 車馬炮 | 59,987 | 8.12% | 1.509% | 1.642% |
| 12–11 | soldiers | 5,280 | 5.08% | 2.860% | 3.013% |
| 10–9 | 士象 | 15,434 | 7.02% | 2.371% | 2.550% |
| 10–9 | 車馬炮 | 29,891 | 7.74% | 2.395% | 2.596% |
| 10–9 | soldiers | 2,694 | 5.16% | 4.343% | 4.579% |
| 8–7 | 士象 | 8,429 | 7.15% | 2.978% | 3.207% |
| 8–7 | 車馬炮 | 16,665 | 7.58% | 3.150% | 3.409% |
| 8–7 | soldiers | 1,626 | 6.21% | 5.043% | 5.377% |
| 6–5 | 士象 | 3,098 | 7.88% | 3.196% | 3.469% |
| 6–5 | 車馬炮 | 5,852 | 7.35% | 3.195% | 3.449% |
| 6–5 | soldiers | 688 | 5.52% | 5.233% | 5.538% |
| 4–3 | 士象 | 649 | 6.47% | 2.928% | 3.130% |
| 4–3 | 車馬炮 | 1,138 | 7.03% | 4.394% | 4.726% |
| 4–3 | soldiers | 121 | 5.79% | 4.959% | 5.263% |
| 2–0 | 士象 | 97 | 8.25% | 3.093% | 3.371% |
| 2–0 | 車馬炮 | 184 | 5.98% | 3.261% | 3.468% |
| 2–0 | soldiers | 20 | 15.00% | 0.000% | 0.000% |

## How to read it

- **A safe card is information of high quality and small quantity.**  It
  takes a card's risk against one seat from a few per cent down to zero, but
  comes up only once in twenty.  On it alone the bot will not improve much.
- **The opponent's progress is the big signal.**  The same card carries
  almost no risk against someone still holding 20 cards, and 3% to 5%
  against someone with only 8 left.  Concealed card counts are public.
- **The bot already leans towards safe discards without meaning to.**  Of
  the 士象 and 車馬炮 actually discarded, mostly 7% to 8% are proven safe,
  above the 5% of the candidates: when the seat still holds a copy of a card
  that has just come down, it is mostly a lone card, which goes first
  anyway.

## Results in the bot

Adding these two signals to `HeuristicBot`'s discard did not win in either
of two A/Bs, and the bot is unchanged.  The danger is the "Won on (not
proven)" column of the tables above, the three kinds merged, in units of one
ten-thousandth: 2, 14, 55, 95, 180, 280, 350, 370, 390, 390 (the last row
has a sample of only 1,705 combinations, so it reuses the row before).  The
control is `0b5d48f`, 200,000 deals each.

| Approach | seed base | Deals that differ | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| Take 12 × 32 × danger off the grade | 1794815549 | 5,328 | −0.037 ± 0.010 | [−0.057, −0.017] | loss |
| Compare danger only when grades tie | 1794816883 | 2,513 | −0.002 ± 0.007 | [−0.015, +0.011] | insufficient data |

The first approach made that seat deal in less, but not lose less: in the
16,927 deals that first part at a discard, its deal-ins fell from 3,460 to
3,057, wins by another seat that it did not deal in rose from 9,879 to
10,356, and its own wins fell from 2,447 to 2,272.  A waiting seat wins
without any one seat dealing in, because every drawn card is turned up and
offered to all four seats.  The design follow-up is in
`doc/stronger-bot.md`.

## Caveats

- **The opponents are the same bot.**  The 5% depends on how often the
  opponents eat and pong; the danger values depend on how fast they come to
  wait.  People would give different numbers; only the proof itself would
  not change.
- **No confidence intervals.**  The discards of one round test nearly the
  same hands, so the samples are correlated.
- **Split only by concealed card count and kind.**  How many 胡 the
  opponent still lacks, and how many of the unseen partner cards are left,
  were not broken out.
