# 「好牌先打」 (good cards first): the order of lone cards

Declared 2026-11-20.  The hypothesis: "good cards first" may be wrong.
[The danger of a discard](discard-danger.md) measured only how often
soldiers, 士象 and 車馬炮 are won on when they come down; `HeuristicBot`'s
order of lone cards itself has never been through an A/B.

`prospects` fines a lone card `COPIES * eaten[kin]`, that is, the 胡 of the
meld it would be eaten into (soldiers 3, 將士象 2, 車馬炮 1), so the dearer
a lone card, the sooner it goes.  Both candidates keep the average level as
it is and change only the order:

| Candidate | Fine on a lone card | Soldiers | 士象 | 車馬炮 |
| --- | --- | ---: | ---: | ---: |
| current (control) | `COPIES * eaten[kin]` | 3 | 2 | 1 |
| flat | `COPIES * eaten[0]` | 2 | 2 | 2 |
| reverse | `COPIES * (eaten[1] + eaten[2] - eaten[kin])` | 1 | 2 | 3 |

Control `b25fb0c`, four seats, `RESHUFFLE=8`, 100,000 deals each, run on
gamete:

- flat: seed base 1795400000
- reverse: seed base 1795500000

## Results

| | flat | reverse |
| --- | ---: | ---: |
| Deals | 100,000 | 100,000 |
| Deals that part | 48,315 | 70,456 |
| Score difference (points a deal) | −0.057 ± 0.017 | +0.027 ± 0.022 |
| 95% CI | [−0.090, −0.025] | [−0.015, +0.070] |
| Verdict | loss | insufficient data |
| Win rate, control → candidate | 24.22% → 24.08% | 24.35% → 24.45% |
| Mean over the deals that part (points) | −0.119 | +0.039 |
| Standard deviation within a split (points) | 19.10 | 20.39 |
| Standard deviation between splits (points) | 3.48 | 3.76 |

flat loses; reverse's CI straddles 0, which is insufficient data, not "no
loss".  Neither goes into `HeuristicBot`, and play is unchanged.  The
hypothesis that good cards first may be wrong is neither confirmed nor
refuted: with the order turned right round, 100,000 deals measure no
difference.

## Where flat loses

By the iron rules, a loss is traced before it is declared.  Apply
`research/room-for-melds-trace.patch` and take the first 20,000 deals of the
same seed base:

```sh
git apply research/room-for-melds-trace.patch
TRACE=1 SEED_BASE=1795400000 ab/run.sh 20000 b25fb0c > out.txt
python3 ab/tally.py out.txt 20000
```

Of these 20,000 deals 9,634 part, at −0.008 ± 0.037 points a deal, which by
itself shows neither a win nor a loss; the groups below were formed after
seeing the results, and serve only to look for the cause and to pick the
next experiment.

| Control → candidate | Deals | Mean (points) | ± | Contribution a deal (points) |
| --- | ---: | ---: | ---: | ---: |
| eat 車馬炮 → pass | 287 | −0.71 | 0.38 | −0.0102 |
| eat a pair → pass | 33 | −1.59 | 1.46 | −0.0026 |
| pass → eat 將士象 | 32 | −1.62 | 0.84 | −0.0026 |
| pass → eat soldiers | 44 | −1.14 | 0.53 | −0.0025 |
| pass → eat a pair | 19 | −1.97 | 2.53 | −0.0019 |
| pass → eat 車馬炮 | 10 | −2.28 | 2.11 | −0.0011 |
| eat soldiers → eat soldiers | 7 | +0.63 | 3.51 | +0.0002 |
| pass → pong | 6 | +1.10 | 2.40 | +0.0003 |
| eat soldiers → pass | 1 | +16.63 | — | +0.0008 |
| pong → pass | 11 | +1.67 | 1.79 | +0.0009 |
| eat 將士象 → pass | 46 | +0.66 | 0.65 | +0.0015 |
| discard → discard | 9,138 | +0.02 | 0.08 | +0.0087 |

95% of the deals that part do so at a discard, and the discards together
come to +0.02 ± 0.08: which lone card goes first measures no difference.
The 496 deals of eats and pongs together come to −0.69 ± 0.29 points, −0.017
a deal; the fine on a lone card is also the valuation of whether to eat, and
that is what flat disturbs; the largest group is where the control ate
車馬炮 and the candidate now does not.

The discards again, by which kind each side discarded:

| Control → candidate | Deals | Mean (points) | ± | Contribution a deal (points) |
| --- | ---: | ---: | ---: | ---: |
| 士象 → 車馬炮 | 6,128 | −0.06 | 0.10 | −0.0175 |
| soldiers → 士象 | 900 | −0.10 | 0.22 | −0.0046 |
| 車馬炮 → 車馬炮 | 36 | +0.27 | 1.50 | +0.0005 |
| 車馬炮 → soldiers | 19 | +1.43 | 2.62 | +0.0014 |
| soldiers → 車馬炮 | 1,657 | +0.05 | 0.18 | +0.0045 |
| 車馬炮 → 士象 | 398 | +1.23 | 0.42 | +0.0244 |

("士象" includes 將.)  94% of the discards that part come while the pile
still has 24 cards or more, at a mean of +0.00 points.

Conclusion: flat's loss is real (the verdict comes from the 100,000 deals
declared), but the trace points at the valuation of eats, not at the order
of discards.  To ask about good cards first again, the next experiment
should change only the order of lone cards at a discard and leave the
valuation of eats as it is, with a fresh seed and a fresh declaration.
