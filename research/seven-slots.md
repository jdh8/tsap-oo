# Seven slots: where a winning hand's 胡 come from

Measured 2026-11-18.  The question: a winning hand is always 21 cards; seen
as 7 slots of three cards at a base of 1 胡 each, which melds do the other 3
胡 of the threshold come from?  The ceiling of `doc/hu-ceiling.md` is built
on this arithmetic; here are the arithmetic, its verification and the
statistics.  No change to play; the base is `6d5a2a8`.

## Answer

- **胡 = 7 + 2k + the number of 將士象 + 2 × (concealed triplets + soldiers
  of three colours + soldiers of four colours + kongs) + bonus**,
  k = (singles − pairs) ÷ 3, the bonus being +1 for soldiers of four
  colours, +2 for an exposed kong and +4 for a concealed quad.  The singles
  are the lone generals (將帥), plus the extra card of each four-card meld.
  It holds hand by hand on the 19,404 wins of 20,000 rounds of self-play by
  four `HeuristicBot`s, and k is a whole number on every one of them.
- **k is almost always 0 or +1.**  56.4% of wins have as many singles as
  pairs, 35.2% three more.  A quarter of the wins with k = +1 (9.1% of all)
  would be short of ten 胡 without those 2 胡.
- **A winning hand holds 2.76 lone generals and 2.12 pairs on average.**
  0.70 of the generals were drawn, and 1.13 of the pairs were eaten.
- **Eating a pair is the commonest negative term.**  Pairs pull a hand down
  by 1.41 胡 on average, −⅔ each; generals add 1.84 a hand, the largest
  positive term, followed by soldiers of three colours 1.23, exposed kongs
  1.02 and 將士象 0.94.

## The arithmetic

A meld of s cards worth v 胡 has v − s ÷ 3 more than the s ÷ 3 胡 its cards
are due.  The 胡 of 21 cards are therefore 7 plus the excess of each meld:

| Meld | Cards | 胡 | Excess | Splits into |
| --- | ---: | ---: | ---: | --- |
| Pair | 2 | 0 | −⅔ | a negative single |
| 車馬炮, pong | 3 | 1 | 0 | a base slot |
| General | 1 | 1 | +⅔ | a single |
| 將士象 | 3 | 2 | +1 | a slot +1 |
| Soldiers of three colours, concealed triplet | 3 | 3 | +2 | a slot +2 |
| Soldiers of four colours | 4 | 5 | +3⅔ | a slot +2, a single, bonus +1 |
| Exposed kong | 4 | 6 | +4⅔ | a slot +2, a single, bonus +2 |
| Concealed quad | 4 | 8 | +6⅔ | a slot +2, a single, bonus +4 |

Singles and pairs are ±⅔ each, ⅔ × (singles − pairs) together.  A four-card
meld is one slot plus one single, so 21 = 3 × slots + singles + 2 × pairs;
singles + 2 × pairs is a multiple of 3, that is, singles ≡ pairs (mod 3):
the difference is 3k, and ⅔ times it is 2k.

## Verification

For every win, `examples/census.rs` scores the exposed melds and the winning
card as exposed and the rest as concealed, and asserts that there are 21
cards, that the 胡 equal those of `Round::result`, that the identity holds
and that singles − pairs is a multiple of 3.  All 20,000 rounds pass.  The
excess 胡 of every kind come to 5.23 a hand, so a win averages 12.23 胡, the
same as the mean of the distribution of 胡 below.

## Statistics

Self-play by four `HeuristicBot`s, 20,000 rounds, seed 1795000000, the
dealer rotating as `Game::settle` has it; 19,404 rounds had a winner.
"Concealed" is the number scored as concealed in the final layout, so for
generals "total − concealed" is the ones drawn, and for pairs the ones
eaten.

| Meld | Excess each | Wins with one | Number a hand | Of them concealed | Excess 胡 a hand |
| --- | ---: | ---: | ---: | ---: | ---: |
| Pair | −0.67 | 92.87% | 2.118 | 0.991 | −1.412 |
| 車馬炮 | 0 | 62.55% | 0.866 | 0.448 | 0 |
| Pong | 0 | 76.78% | 1.300 | 0 | 0 |
| General | +0.67 | 95.42% | 2.758 | 2.063 | +1.839 |
| 將士象 | +1.00 | 67.53% | 0.939 | 0.531 | +0.939 |
| Soldiers of three colours | +2.00 | 54.91% | 0.614 | 0.282 | +1.228 |
| Concealed triplet | +2.00 | 33.93% | 0.389 | 0.389 | +0.778 |
| Soldiers of four colours | +3.67 | 16.93% | 0.171 | 0.054 | +0.626 |
| Exposed kong | +4.67 | 20.80% | 0.218 | 0 | +1.019 |
| Concealed quad | +6.67 | 3.12% | 0.031 | 0.031 | +0.208 |

By k:

| k | Wins | Short of ten 胡 without the 2k |
| ---: | ---: | ---: |
| −2 | 0.01% | 0 |
| −1 | 5.54% | 0 |
| 0 | 56.43% | 0 |
| +1 | 35.17% | 9.11% |
| +2 | 2.82% | 0.72% |
| +3 | 0.04% | 0.01% |

The number of 胡:

| 胡 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21+ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wins | 22.70% | 24.53% | 16.42% | 13.69% | 7.87% | 6.26% | 3.63% | 2.26% | 1.17% | 0.75% | 0.40% | 0.33% |

The commonest recipes (listing only the melds worth more 胡 than the base):

| Wins | Recipe |
| ---: | --- |
| 13.08% | 將士象 + soldiers of three colours |
| 6.38% | 將士象 × 2 + soldiers of three colours |
| 5.17% | 將士象 + soldiers of three colours + concealed triplet |
| 5.17% | 將士象 + concealed triplet |
| 4.80% | soldiers of three colours + concealed triplet |
| 4.38% | 將士象 + soldiers of four colours |
| 3.95% | soldiers of three colours |
| 3.53% | soldiers of four colours |
| 3.35% | 將士象 + exposed kong |
| 3.00% | exposed kong |

The 20 cards dealt to a seat other than the dealer, by the number of
generals and by "the 胡 its three-card melds have over 1 胡 each" (the shape
of most 胡 that `Solver` lays out), with the win rate of each:

| Generals | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Hands | 3.15% | 13.76% | 25.10% | 26.67% | 18.65% | 8.84% | 2.92% | 0.77% | 0.15% |
| Win rate | 11.02% | 15.37% | 19.49% | 22.89% | 26.59% | 29.70% | 30.80% | 32.90% | 36.67% |

| Excess 胡 | 0 | 1 | 2 | 3 | 4 | 5 | 6+ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Hands | 13.51% | 15.42% | 27.09% | 16.90% | 15.14% | 5.63% | 6.31% |
| Win rate | 9.20% | 13.77% | 20.99% | 27.02% | 29.31% | 34.26% | 35.89% |

The program prints the full distribution of (singles, pairs); the most
frequent are (2, 2) 21.51%, (3, 3) 18.29%, (4, 1) 14.49%, (5, 2) 11.53% and
(1, 1) 8.25%.

## Tool

```sh
cargo run --release --example census -- 20000 1795000000
```

About four minutes.  The output matches the numbers `doc/hu-ceiling.md`
quotes digit for digit.

## Caveats

- **The statistics come from `HeuristicBot`'s own wins.**  The number of
  generals and the number of pairs eaten both carry this bot's habits; the
  identity does not depend on the bot.
- **The layout is the one `Round::result` gives.**  When the same hand has
  another layout, k and the terms change; the 胡 do not.
