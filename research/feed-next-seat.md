# Not feeding the next seat: fining a discard by the chance the next seat can eat it

Measured 2026-11-17.  The question: 16% to 56% of discards can be eaten by
the next seat into a run or a pair ([`discard-danger.md`](discard-danger.md)),
which advances the next seat's hand directly.  MARJJ, the champion of a gin
rummy competition, adds "how useful this card is to the opponent" to the
grade of a discard ([`prior-art.md`](prior-art.md)).  Would `HeuristicBot`
get stronger by doing the same?  This is the third item of "Suggested order
of next steps" in `doc/stronger-bot.md`, measured once at a table of four
and once at a table of two.

## Answer

**No win at a table of four, and the bot is unchanged.  A small win at a
table of two.**

- **As a weight it loses.**  Taking 32 × the chance the next seat can eat
  it off each discard, a table of four reads −0.172 ± 0.017 points a deal,
  loss.  It overturns the order in which lone cards go: nearly all of the
  loss is in substitutions across kinds, of the sort "discard a 車馬炮
  instead and keep a 士象".
- **As a choice among like cards, a table of four shows no difference.**
  Taking 4 times (less than the gaps in grade between the kinds of lone
  card), the first run of 100,000 deals reads +0.015 ± 0.008 and the second
  of 400,000 deals −0.007 ± 0.004, both insufficient data, and the two
  differ by less than three standard errors.
- **The same change is a win at a table of two:** +0.007 ± 0.003 points a
  deal.  At a table of two the next seat is the only opponent, and every
  card fed to it counts directly against oneself; at a table of four the
  next seat is only one of three.  This agrees with what the literature
  expects (in gin rummy the opponent is the only beneficiary), but the size
  is only a few thousandths of the standard deviation of one deal's score
  difference at a table of two.

## Approach

`research/feed-next-seat.patch` (against `src/heuristic.rs` at `e40cca4`).
At a discard, the grade of each card that could be discarded loses
`FEED × edible`:

- `edible` is the chance the next seat can eat the card into a meld: it
  holds the same card (making a pair), or the other two cards of the same
  colour (將士象, 車馬炮), or two or more soldiers of other colours
  (soldiers of three or four colours).
- The next seat holds h concealed cards, drawn from the U cards the
  discarding seat has not seen; with n copies of a card not yet seen, the
  chance the next seat holds at least one is 1 − C(U − n, h) / C(U, h).
  Different partner cards are treated as independent.  A partner card that
  is all out has n = 0 and drops out by itself (the "wall" of layer 3).
- Whether the next seat wants to eat is not asked.  Only the discard at the
  top level changes; the discard after an eat or a pong does not.

The A/B is `ab/run.sh`, the candidate in the second seat, the control
`e40cca4`, the pile reshuffled 8 times from where the two part
([`reshuffle.md`](reshuffle.md)).  Every deal count was fixed before the
run, and the seed base taken from the clock at the time.  The table of two
follows [`two-seats.md`](two-seats.md): in a scratch copy, `Seat::COUNT` of
both versions is changed to 2.

## Data

| | FEED 32, four players | FEED 4, four players | FEED 4, four players | FEED 4, two players |
| --- | ---: | ---: | ---: | ---: |
| seed base | 1794847173 | 1794848292 | 1794849203 | 1794848696 |
| Deals | 100,000 | 100,000 | 400,000 | 200,000 |
| Deals that part | 45,847 | 10,373 | 41,738 | 18,257 |
| Score difference (points a deal) | −0.172 ± 0.017 | +0.015 ± 0.008 | −0.007 ± 0.004 | +0.007 ± 0.003 |
| 95% CI | [−0.205, −0.140] | [−0.002, +0.031] | [−0.015, +0.001] | [+0.002, +0.012] |
| Verdict | loss | insufficient data | insufficient data | win |
| Win rate, control → candidate | 24.35% → 23.97% | 24.24% → 24.27% | 24.26% → 24.24% | 49.91% → 49.95% |
| Standard deviation within a split (points) | 19.10 | 19.28 | 19.31 | 9.50 |
| Standard deviation between splits (points) | 3.87 | 4.40 | 4.24 | 1.69 |

### Where FEED 32 loses

By the iron rules, a loss is traced before it is declared.  Another 6,000
deals (seed base 1794848201; these deals serve only the trace and gave no
verdict) are grouped by the kinds of card the two sides discarded at the
step where they part; each group is the mean score difference over 8
reshuffles:

| Control discards → candidate discards | Deals | Mean (points) | Total (points) |
| --- | ---: | ---: | ---: |
| 士象 → 車馬炮 | 1,034 | −0.35 | −363 |
| 車馬炮 → 士象 | 181 | −0.64 | −116 |
| soldiers → 車馬炮 | 307 | −0.16 | −49 |
| soldiers → 士象 | 449 | −0.05 | −20 |
| 士象 → 士象 | 247 | +0.08 | +20 |
| 車馬炮 → 車馬炮 | 518 | +0.32 | +165 |

Substitutions across kinds are all negative; picking another card of the
same kind is positive.  `FEED = 4` was chosen on this: below the gaps in
grade between the kinds of lone card, so that it only picks among like
cards.  Its verdict uses seed bases taken afresh afterwards.

## Caveats

- **The two FEED 4 runs at a table of four do not agree well.**  +0.015 ±
  0.008 and −0.007 ± 0.004 differ by 0.022 ± 0.009, about 2.4 standard
  errors.  Both are insufficient data, and they were not pooled.
- **The chance of an eat is crude.**  Partner cards are treated as
  independent, and whether the next seat wants to eat is not asked.  At a
  table of four it guards only against the next seat, and does not count the
  chance that another seat pongs the card.
- **The table of two is a game in a scratch copy.**  The win at a table of
  two does not change the library: the library's bot is written for a table
  of four, and does not branch for a table of two until that is an official
  way to play.
