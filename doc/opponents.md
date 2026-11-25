# Reading the opponent at a table of two: the design

Written 2026-11-25, base `1430e6b`.  It takes up route 1 of
[`stronger-bot.md`](stronger-bot.md) where that document left it: the
Peeker, a Monte Carlo that sees the other hand, was +2.824 ± 0.192 points a
deal at a table of two, and nothing that computes more on the same
information has come to a fifth of that.  This document splits "read the
opponent" into channels, measures each with an oracle before any inference
is written, and gates the inference on the oracle.

**The Peeker's number is void.**  It was measured on 2026-11-17, with 65
draws a round; since `07d5602` (2026-11-21) the hand of the empty seats is
dead and a table of two draws 25 ([`variants.md`](variants.md)).  And its
control was `HeuristicBot`; what ships at two players now is `ExpectBot`
with tables of its own.  The room left above the bot that ships is not
known.  (P has measured it since: +1.874 ± 0.200 against the desktop bot.)

**The table is two players at 10 胡**, where the Peeker's signal was more
than ten standard errors in 3,200 deals, where a point the opponent gains is
a point this seat loses, and where every discard passes the one opponent.
A channel that wins here is measured at 8 胡 and at four players before it
ships there.

## Questions to answer

1. How much room is left above the bot that ships, under the rule in
   force?  (The Peeker again, against `ExpectBot`.)
2. Which input of the DP carries it: the cards that will still come
   (`live`), how soon the opponent wins (h), or what a discard gives away?
3. Does h bent by what is public already, the opponent's count of concealed
   cards, win by itself?  [`expectation.md`](expectation.md) names it "the
   dimension h should gain next".
4. For a channel that is open: how much of the oracle's gain does an
   inference from the public cards recover?

## Progress

Update this table when a batch is done.

| Batch | Output | Changes play | Gate | Status |
| --- | --- | --- | --- | --- |
| 0 | `examples/hazard.rs`, `ab/src/oracle.rs`, `ExpectBot::outlook_mut`, this document | No; owes zero deals that part | None | Done: zero deals part in all three runs |
| 1 | The oracles and the Peeker at two players, and the bot that ships against `HeuristicBot` | No: oracles never ship | None | Done: `danger` and `live` win on the web and on the desktop; `waits` and `cards` are closed |
| 2 | h by the fewest concealed cards into `Outlook` | Yes | `cards` wins on the web and on the desktop | Not started: B did not win |
| 3 | An inference for each open channel, largest first | Yes | Its oracle wins on both settings; a gatekeeper is in place | `danger` first: its halves `win` and `claim` both win (W, X), and `ExpectBot::wary` on `Giveaway`'s tables loses on the web (T), so it stays off; T's trace closes `Giveaway` as fitted: the oracle's price one card ahead is not what the deal pays.  `live`: the public cards say next to nothing about what still comes (`kept`), and `ExpectBot::counting` on those chances reads −0.006 ± 0.009 on the web (L): closed, and taken out.  Batch 4 is next |
| 4 | `examples/posterior.rs`: what the hands that fit the other seat's whole history say of its hand | No | None; it gates a sampler | Queued behind L: both samplers, 200 rounds, seed 800 |

## The oracles

An oracle is `ExpectBot` as it ships with one input replaced by what the
opponent's hand says.  It is a probe, not a ceiling: it says what the truth
in that one input is worth to the DP as it is built.  Only the Peeker
bounds every inference from above, and it has its own flaw (it also removes
the search's fault of deciding separately in each sampled deal).

They live in `ab/src/oracle.rs` and are seated with `ORACLE=`: `live` and
`danger` still, with `win` and `claim`, the two halves of `danger` (batch
3); `waits` and `cards` only in `01f98fd`, since both were taken out once
their web runs read zero (batch 1).  Each reads
the other hands through `View::peek` and never the pile's order, so
`RESHUFFLE` stays unbiased: up to the first choice the control would not
make, every decision saw hands and cards already turned up, and the pile
left is still in uniform order.  The 46 dead cards of a table of two are
part of that pile; no oracle knows which they are, and no inference could.

| `ORACLE` | Input replaced | What it reads |
| --- | --- | --- |
| `live` | `live`, the cards taken as still to come | The opponent's hand |
| `waits` | h, by draws on from the decision | The opponent's hand |
| `danger` | The value of each discard | The opponent's hand |
| `cards` | h, by draws on from the decision | The opponent's count of concealed cards: public |

### `live`: are the cards I want still there

`ExpectBot` takes every card it has not seen as equally likely to come.
The oracle takes off the cards the opponent keeps: its concealed hand less
every kind that is loose in either of the two layouts `HeuristicBot`
grades, which are the cards it discards from.  The hook is
`ExpectBot::gone_mut`, hidden and behind the `peek` feature; an inference
would feed the same input.

Approximations: the opponent's loose cards stay in `live` at an even
chance, though they come only as its discards and sooner than a card of the
pile; and "keeps" is `HeuristicBot`'s layout now, not what it will hold
later.

### `waits`: how soon the opponent wins

`Outlook::end` is one table by draws left, whoever the opponent is.
`examples/hazard.rs` measures how it bends: at every decision of a seat it
reads a key off the other seat, then follows every public card to the end
of the round and counts the ones the other seat wins on, by the card's lead
(the draws left at the decision less the card's segment), over the wins
expected of cards at decisions with as many draws left.  Dividing by the
expectation at the same draws left takes the stage of the round out: early
in a round the opponent is far from waiting whatever the key says.

The oracle's key is the opponent's waits: the copies it has not seen of the
cards it wins on, 0 to 4 or more.  At a decision with `draws` left the
oracle multiplies `end[k]` by the ratio at lead `draws − k`.

`PLAYERS=2 hazard 20000 400`, `HeuristicBot` self-play (wins over expected;
the cards at lead 0):

| Waits | Lead 0 | 1 | 2 | 5 | 10 | Cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 0.214 | 0.538 | 0.660 | 0.859 | 0.935 | 702,596 |
| 1 | 0.385 | 0.652 | 0.502 | 0.736 | 0.736 | 3,112 |
| 2 | 1.228 | 1.042 | 1.009 | 1.442 | 1.326 | 5,579 |
| 3 | 1.904 | 1.620 | 1.625 | 1.777 | 1.714 | 32,766 |
| 4 or more | 3.032 | 2.827 | 2.682 | 2.105 | 1.888 | 97,458 |

A cell with fewer than 1,000 cards holds the ratio of the lead before it.

Approximations: the ratios are pooled over the deciding seat's hand; the
key is read once, at the root; and past the gate only the levels of the
shallow graph bend, since a leaf's curve is `Eval`'s, learnt under the h
that ships.

### `cards`: the same from what is public

The same measurement keyed by the opponent's concealed cards, which every
seat can count.  It does not cheat; it sits with the oracles because it
shares their table and their wrapper.

| Concealed cards | Lead 0 | 1 | 2 | 5 | 10 | Cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 0.289 | 0.494 | 0.659 | 0.846 | 0.933 | 191,813 |
| 17 | 0.495 | 0.731 | 0.747 | 0.907 | 0.905 | 126,671 |
| 14 | 0.971 | 1.031 | 1.043 | 1.053 | 0.981 | 80,809 |
| 11 | 1.264 | 1.237 | 1.159 | 1.123 | 1.016 | 44,820 |
| 8 | 1.377 | 1.248 | 1.296 | 1.125 | 1.404 | 17,078 |

With the stage of the round taken out the spread is fivefold at lead 0
(0.29 at 20 cards, 1.44 at 6) and mostly gone by lead 10, against
fourteenfold for the waits.  `research/outlook.md` measured fifteenfold by
this key at four players, with the stage of the round left in.

### `danger`: what a discard gives away

`ExpectBot` values a discard by the hand it leaves.  The oracle plays each
discard on the round it peeks at:

- if the opponent wins on the card, the discard is worth the settled score,
  when that is less than the DP's value;
- if `HeuristicBot` in the opponent's seat claims it, the discard is worth
  less by what the DP, run in that seat, makes of the claim over a pass,
  when that is positive.

A claim is worth less by what its best discard gives away.  Nothing here is
a constant to tune: at a table of two a point of the opponent's is a point
of this seat's.

Approximations: it looks one card ahead, so a card kept for safety is as
dangerous next turn, and the DP does not know it is stuck; the opponent's
gain is added on top of an h that already holds the average of it; a
discard the opponent kongs counts as safe.

## Batch 0: the tools

- `ExpectBot::outlook_mut`, and `ExpectBot::gone_mut` behind `peek`.  Both
  leave play alone and owe zero deals that part.
- `examples/hazard.rs` and the two tables it printed, in `ab/src/oracle.rs`
  until `waits` and `cards` were taken out.
- `ab/src/oracle.rs` and `ORACLE=` in `ab/src/main.rs`.

The zero runs (declared 2026-11-25 before any data; control `1430e6b`,
`RESHUFFLE=8`, candidate `EXPECT=new`, no `ORACLE`):

| Setting | Table | Control | Deals | seed base | Part |
| --- | --- | --- | ---: | ---: | ---: |
| Web | two players, 10 胡 | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 2,000 | 1803000000 | 0 |
| Web | four players, 10 胡 | `BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 2,000 | 1803100000 | 0 |
| Desktop | two players, 10 胡 | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17` | 500 | 1803200000 | 0 |

Web is `GATE_CARDS=10 SHALLOW=0` and desktop `GATE_CARDS=14 SHALLOW=17`, as
in [`eval.md`](eval.md).

## Batch 1: the oracles

Declared 2026-11-25, before any data.  Two players at 10 胡, control
`1430e6b`, queued through pueue from one frozen copy of the working tree.
The oracles were smoke-tested on 300 deals of seed base 7 to see that they
run and how long they take; those deals give no verdict.

**The oracles on the web setting**, 10,000 deals each, `RESHUFFLE=8`,
candidate `EXPECT=new GATE_CARDS=10 SHALLOW=0 ORACLE=…`, control
`BASELINE_EXPECT=new BASELINE_SHALLOW=0`: the bot that ships, in the same
seat.

| Run | `ORACLE` | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | --- | --- |
| A | `waits` | 1803300000 | 2,320 | +0.011 ± 0.018 | [−0.024, +0.047] | insufficient data; closed |
| B | `cards` | 1803400000 | 1,660 | −0.009 ± 0.015 | [−0.039, +0.020] | insufficient data; closed |
| C | `live` | 1803500000 | 7,610 | +0.107 ± 0.036 | [+0.037, +0.178] | win |
| D | `danger` | 1803600000 | 7,679 | +0.984 ± 0.046 | [+0.893, +1.074] | win |

`danger` is nine times `live`.  `waits` and `cards` are closed by the rule
below: bending h by the opponent as it stands, read off its hand or off its
public count, is worth under +0.05 points a deal to the web bot.  Both were
then taken out of `ab/src/oracle.rs`; `01f98fd` has them, and
`examples/hazard.rs` still prints their tables.

**The same on the desktop setting**, 8,000 deals each, `RESHUFFLE=8`,
`GATE_CARDS=14 SHALLOW=17` against `BASELINE_GATE=14 BASELINE_SHALLOW=17`.
Each runs only if its web run is a win (`pueue add --after`, as
`AFTER=` of `ab/queue.sh` does it).

| Run | `ORACLE` | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | --- | --- |
| A′ | `waits` | 1804100000 | | | | not run: A is not a win |
| B′ | `cards` | 1804200000 | | | | not run: B is not a win |
| C′ | `live` | 1804300000 | 5,921 | +0.172 ± 0.039 | [+0.095, +0.249] | win |
| D′ | `danger` | 1804400000 | 6,024 | +1.039 ± 0.051 | [+0.939, +1.139] | win |

`live` and `danger` are both open, and both worth more to the desktop bot
than to the web one (+0.172 against +0.107, +1.039 against +0.984), not
less as the risk below feared.

**The Peeker against the bot that ships**, 3,200 deals as on 2026-11-17,
`RESHUFFLE=0` (every replay would be a search), candidate `SAMPLES=128
PEEK=1`, control `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17`,
the desktop bot.

| Run | seed base | Differ | Candidate − control | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| P | 1803700000 | 1,806 | +1.874 ± 0.200 | [+1.482, +2.267] | win |

The room above the bot that ships is two thirds of the void +2.824, which
was against `HeuristicBot` with 65 draws a round.  `danger` on the web
(+0.984) is half of it, `live` on the desktop a tenth; they are measured
one at a time and on different settings, so they need not add up.

**The bot that ships against `HeuristicBot`**, which
[`variants.md`](variants.md) lists as never measured under the rule in
force.  Candidate `EXPECT=new`, control the default seat, `RESHUFFLE=8`.

| Run | Table | Setting | Deals | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| H2 | two players | Web | 10,000 | 1803800000 | 8,583 | +0.405 ± 0.042 | [+0.324, +0.487] | win |
| H3 | three players | Web | 10,000 | 1803900000 | 8,568 | +0.348 ± 0.063 | [+0.225, +0.472] | win |
| H2′ | two players | Desktop | 4,000 | 1804000000 | 3,382 | +0.615 ± 0.067 | [+0.483, +0.746] | win |

The bot that ships beats `HeuristicBot` at two and at three players at
10 胡, and on the desktop at two by half again as much as on the web.

### How the verdicts are read

- **A channel is open** when its oracle wins on the web and on the desktop.
  Batch 3 takes the open channels, the largest first.
- **A channel is closed** when the upper end of its web CI is under +0.1
  points a deal: an inference gets a part of what the oracle gets, and a
  part of that is under what 10,000 deals can see.
- **Otherwise it is insufficient data**: one rerun of 40,000 deals on a
  fresh seed base, read the same way.
- **A loss** is traced before it is called dead: the deals that moved the
  most, as `CLAUDE.md` asks.
- **`cards` is a candidate, not an oracle.**  A win on both settings starts
  batch 2.
- **P and the H runs gate nothing.**  P says how much room there is; if the
  oracles that win add up to a small part of it, the rest is in what they
  approximate away or in the search itself.

## Batch 2: h by the fewest concealed cards

**Not started: B did not win**, and neither did A, which reads the hand
that the count stands in for.  What follows is the plan as it was declared.

Only if B and B′ both win.  The table moves into `Outlook` (a bend of `end`
by lead and by the count), measured at each table it is to ship at, and
`ExpectBot::new` reads it.  The library's version owes a replay of B's
first 1,000 deals that matches `ORACLE=cards` word for word, then ships at
two players at 10 胡 on B and B′.  Two players at 8 胡 and the larger tables
each need their own measurement and A/B.

## Batch 3: an inference for each open channel

Designed once batch 1 has its verdicts, a section for each channel here.
What is known before:

- **`live`** plugs in as application G of
  [`expectation.md`](expectation.md): counts in place of a hand, since an
  inference gives chances.  The evidence is what the opponent let go
  (`View::safe` holds the proof for wins; the same events say what it did
  not eat or pong), what it discarded and what it exposed.
- **`waits`** would need the same evidence turned into a chance of
  waiting; `cards` is its public floor.  Both are closed (A and B).
- **`danger`** is layers 1 to 3 of `stronger-bot.md` with a price in
  points; none of the three won at four players, where the cost is shared
  among three seats.
- **A gatekeeper first.**  An inference learns `HeuristicBot`'s habits.
  `stronger-bot.md` asks for a second opponent that must not regress
  before any such layer; the `guest` example's host
  (chunhua1972/four-color-game's Normal AI) is the one at hand.

### `danger`: what the public cards say a discard gives away

Designed 2026-11-25, base `6a61a3e`.  The oracle plays each discard on the
round it peeks at; an inference has to say the same of a card from what
every seat can see of it.

**Where the oracle's gain lies.**  `examples/giveaway.rs` plays the web
page's bot against `HeuristicBot` at a table of two, each deal once in
either seat, and at every discard valued in points it plays every card the
seat could put down as the oracle plays it.  4,500 rounds, seed 600: 40,384
discards and 376,363 cards.  One card ahead, in the oracle's own values, the
oracle keeps 2.86 points a round over the bot's choice, 0.71 by the cards
the other seat wins on and 2.17 by the cards it claims; the first move one
discard in sixty, the second one in four.  It is a measure of where the
choices lie, not of points won: the A/B has the whole oracle at +0.984.

**What is public about a card.**  A concealed hand only shrinks, since
nothing drawn enters it.  So what the other seat let go it has still no use
for, and a card it threw from its hand says the hand held nothing to go
with it: no second copy, and none of the kinds that make a run with it.
Of the same rows:

| The card | Cards | The other seat wins on it | Claims it |
| --- | ---: | ---: | ---: |
| It threw a copy from its hand | 11,325 | 0.05% | 0.0% |
| A copy lies on the pile, or `View::safe` holds the card | 9,160 | 0.13% | 2.3% |
| Neither, and it threw two kinds of the card's run | 221 | 0.45% | 0.9% |
| Neither, and it threw one kind of the card's run | 35,132 | 1.96% | 7.7% |
| Neither, and none | 320,525 | 1.06% | 41.2% |

**The inference** is `Giveaway` (`src/giveaway.rs`): a card's `Evidence` is
the row above, and for the last three the copies of its run's kinds the seat
has not seen, by twos (the walls of layer 3).  `Giveaway::cost` is the
mean, over the cards of that evidence, of what the oracle takes off: the
fall to the settled score when the other seat wins on the card, and the
DP's value of `HeuristicBot`'s claim over a pass in that seat.  0.002 for a
thrown card, 0.051 for one let go, 0.109 to 1.236 for a fresh one.
`ExpectBot::wary` discards by each card's value less its cost, wherever the
bot values discards in points.  It is off unless asked for; `WARY=1` asks
in `ab/`.

Fitted on the first two thirds of those rounds and read on the rest, the
tables keep 0.46 of the oracle's 2.86 points a round, one card ahead, and
move 26% of the discards.  Thrown or let go alone keeps 0.11, and the run's
thrown kinds with it 0.34.

Approximations and what is left out:

- **Claims are as before.**  The oracle charges a claim for the discard it
  forces.  The measured h already holds the average of what discards give
  away, so only a choice among discards is free of counting it twice.
- **One table for every stage and both settings.**  The other seat's count
  of concealed cards and its exposed 胡 added nothing once the evidence was
  in; the costs are the web bot's, read by the desktop bot too.
- **It learns `HeuristicBot`.**  That a hand only shrinks is a rule; that
  a card let go or thrown was of no use is a habit.  A person may throw
  from a pair.
- **A hand dealt from the unseen cards is no model.**  The oracle run on 8
  such hands in place of the true one took 479 seconds for 300 smoke deals
  and changed 276 of them for nothing to be seen; a hand kept through a
  round is not a random one.  That code is not kept.

**The runs**, declared 2026-11-25 before any data; two players at 10 胡,
control `6a61a3e`, `RESHUFFLE=8`, queued through pueue from one frozen copy
of the working tree.  The smoke runs were on seed base 7 and give no
verdict; the tables are measured on other deals (`giveaway`, seed 600).

On the web setting, `EXPECT=new GATE_CARDS=10 SHALLOW=0` against
`BASELINE_EXPECT=new BASELINE_SHALLOW=0`:

| Run | Candidate | Deals | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| Z | as it ships: owes zero deals that part | 2,000 | 1804500000 | 0 | +0.000 ± 0.000 | [+0.000, +0.000] | zero deals part |
| W | `ORACLE=win` | 10,000 | 1804600000 | 755 | +0.471 ± 0.025 | [+0.422, +0.521] | win |
| X | `ORACLE=claim` | 10,000 | 1804700000 | 7,347 | +0.246 ± 0.043 | [+0.162, +0.330] | win |
| T | `WARY=1` | 40,000 | 1804800000 | 26,409 | −0.061 ± 0.016 | [−0.092, −0.030] | loss |

On the desktop setting, `GATE_CARDS=14 SHALLOW=17` against
`BASELINE_GATE=14 BASELINE_SHALLOW=17`, only if T is a win:

| Run | Candidate | Deals | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| T′ | `WARY=1` | 16,000 | 1804900000 | | | | not run: T is not a win |

Read 2026-11-26.  The two halves of `danger` both win, and the smaller
count is the larger gain: the cards the other seat wins on move one deal in
thirteen for +0.471, +6.2 points on a deal that parts, and the cards it
claims move three deals in four for +0.246.  `wary` parts on two deals in
three and loses 0.061 points a deal; its seat wins 49.77% of the deals
against the control's 50.27%.  It stays off, nothing ships, and the
gatekeeper is not built.

**The trace of T**, read 2026-11-26.  The same 40,000 deals, run again with
`TRACE=` from a frozen copy of the working tree, give T's line again: 26,409
deals part for −0.061 ± 0.016, so the dump leaves play alone.  A row is a
deal's first split: the two discards, the bot's value of each, its
`Evidence` and cost, what the oracle keeps of it one card ahead, and how
`HeuristicBot` would answer the card in the other seat.  That is the answer
the oracle prices; the seat itself holds the control.

The tables do what they were fitted to do.  At the split `wary` gives up
0.075 points of the bot's value for a card the tables cost 0.364 less, and
the oracle, with the hand in sight, costs it 0.349 less (0.833 for the
control's card, 0.484 for the candidate's).  One card ahead that is +0.274
on a deal that parts.  The deal pays −0.092:

| The oracle on the candidate's card | Deals | One card ahead | Candidate − control |
| --- | ---: | ---: | ---: |
| better than the control's | 9,482 | +1.820 | +0.293 ± 0.044 |
| the same | 1,365 | 0 | −0.163 ± 0.070 |
| worse | 15,562 | −0.644 | −0.321 ± 0.029 |

So the price is wrong, not the reading of the cards.  A least-squares fit of
a deal's difference on the value given up and on the two answers, the
candidate's card's less the control's, prices an answer by what the rest of
the deal makes of it; a card the other seat is not asked about is 0:

| Answer to the card | Cards | The oracle takes off | The tables | The deal |
| --- | ---: | ---: | ---: | ---: |
| Not asked | 21,561 | 0 | 0.674 | 0 |
| Pass | 11,062 | 0 | 0.831 | −0.099 ± 0.046 |
| Eat | 15,922 | 1.747 | 0.933 | 0.315 ± 0.042 |
| Pong | 3,568 | 1.387 | 0.831 | −0.339 ± 0.070 |
| Kong | 489 | 0 | 0.729 | −0.233 ± 0.177 |
| Win | 216 | 9.355 | 0.783 | 2.704 ± 0.267 |

A point of value given up costs 0.729 ± 0.172, and a deal that parts loses
0.085 ± 0.028 more that its first split does not account for: the later
discards, made the same way, and whatever the fit misses.

- **An eat costs a sixth of the oracle's price, a win under a third.**  It
  is the approximation named under the oracle: a card kept is as wanted
  next turn, and the seat denied it draws instead.  X showed it already,
  2.17 points a round one card ahead for +0.246 a deal.
- **A pong has the wrong sign.**  The candidate loses 0.459 ± 0.097 over
  the 1,685 deals where only the control's card would be ponged and gains
  0.381 ± 0.101 over the 1,641 where only its own would; a card passed
  leans the same way.  The trace holds no hands, so this is a reading and
  not a measurement: a card the other seat holds copies or mates of is
  deader to this seat than the DP's `live` knows, and `wary` keeps those
  and throws the ones nobody wants.  That is `live` pulling against
  `danger` on the same evidence.
- **The tables hardly tell the cards apart**: 0.67 for a card not asked
  about against 0.93 for one eaten, and 0.78 for one won on.  `wary` cuts
  the cards eaten from 37.8% to 22.5% of the first splits, which at 0.315
  is +0.048, against 0.055 for the value given up.  Cards won on fall from
  0.50% to 0.32%, +0.005.
- **Where it loses.**  On the 9,987 deals where neither card would be
  taken the candidate is −0.184 ± 0.031, which is −0.046 of the −0.061.
  By the stage, −0.052 of the −0.061 is from the deals that part with 70
  cards or more in the pile.  The 3,030 ties,
  where `wary` gives up nothing, are +0.016 ± 0.062.  First splits with 13
  concealed cards or fewer lean the other way, +0.169 ± 0.092 over 1,766
  deals, which is +0.0075 a deal and one cut picked among many.
- **The deals that moved the most** say nothing a table does not: the 15
  each way are 16 to 24 points on a gap of 0.00 to 1.04, 21 of the 30
  between two cards the other seat has shown nothing about and 19 with 70
  cards or more in the pile.  The hands stay as dealt through the
  reshuffles, so that is their luck.

`Giveaway` is dead as fitted, and not for want of deals.  Priced at what the
deal pays, its costs fall to a fifth and part only discards within 0.2 of
each other, and T's ties and its splits within 0.1 show nothing (+0.016 ±
0.062 and −0.037 ± 0.029 a deal that parts); and the half that pays, the
win, is the one the evidence cannot see, at 2% of the cards at most.  `danger` waits for
a reading of when the other seat is waiting, which `waits` and `cards` did
not give.  `live` is next: it is open on both settings, and the pong says
the same public cards are worth more there.

**The win half by the stage**, read 2026-11-26 off the same rows
(`giveaway 4500 600 rows`), before anything was written for it.  The
evidence cannot see the card the other seat wins on; its exposed 胡 and its
count of concealed cards, which every seat can read, see when it is close.
Of the cards it has not let go it wins on 0.08% while it holds 18 concealed
cards or more, 2.7% at 12 to 14 and 5.6% at 6 to 8; 0.34% with under 2 胡
exposed and 6.6% with 8 or 9; and on 0.09% of the cards it has let go.
While it holds 14 concealed cards or fewer, the copies of the card this
seat sees tell cards apart by a factor of three (4.4% for one, 1.5% for
three).

Fitted on the first 3,000 deals and read on the other 1,500 (3,000 rounds,
13,354 discards), a cost by 胡 and by cards, times a factor by the copies
seen:

| Discarding by value less | Discards moved | Kept a round, one card ahead | Cards won on |
| --- | ---: | ---: | ---: |
| nothing: the bot as it ships | | 0 | 221 |
| the oracle's win half | | 0.715 | |
| the cost by the stage at 0.29, what T's deals paid for a win | 1.6% | 0.011 | 214 |
| the same times the card's factor, at 0.29 | 3.3% | 0.010 | 212 |
| the same at the oracle's price | 5.1% | 0.012 | 206 |

That is a seventieth of the oracle's win half, where `Giveaway`'s tables
kept a sixth of the whole oracle and lost.  The stage says when the other
seat is waiting and not on what: the cards a seat chooses among at one
discard share the stage, and late in a round few of them have been let go.
Nothing is built for it and no run is made.

How they are read:

- **W and X gate nothing.**  They say which half of `danger` an inference
  should go after next; the two need not add up to D.
- **T is 40,000 deals** because a part of the oracle's +0.984 is what is
  looked for: the smoke run parted on two deals in three, which puts the
  standard error near 0.016.
- **`wary` ships at two players and 10 胡** if T and T′ both win and the
  gatekeeper below does not lose.  A straddling CI is insufficient data:
  one rerun at four times the deals on a fresh seed base.  A loss is traced
  first, by the deals that moved the most.
- **The gatekeeper is not built yet.**  It is the `guest` example's host at
  a table of two: `ExpectBot` with and without `wary` against the guest on
  paired deals.  It is built only if T wins, and before anything ships.
- **8 胡 and the larger tables** each owe their own tables and A/B.

### `live`: what the public cards say will still come

Designed 2026-11-26, base `8a8ecf9`.  The oracle takes the cards the other
seat keeps out of `live`; an inference has to say of each kind this seat has
not seen how likely its copies are to come, from what every seat can see.

**What is public about a kind.**  `examples/kept.rs` plays the web page's
bot against `HeuristicBot` at a table of two, each deal once in either
seat, and at every discard valued in points it takes each kind the seat has
not seen every copy of, by the same `Evidence` as `danger`: where the
copies are (kept by the other seat as the oracle reads it, loose in its
hand, or in the pile), and how many came up before the round ended, drawn
or thrown from the other seat's hand.  4,500 rounds, seed 700: 1,127,942
kinds and 3,362,744 copies.

| The kind | Copies | The other seat keeps | Holds loose | Kinds it keeps every copy of | Came up | Over its rank | Left free, over its rank |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| It threw a copy from its hand | 195,182 | 7.2% | 0.5% | 1.4% | 10.1% | 0.916 | 0.988 |
| It let a copy go | 254,574 | 8.7% | 3.0% | 1.2% | 10.3% | 0.966 | 1.019 |
| Neither, and it threw two kinds of the run | 4,804 | 22.1% | 0.0% | 6.1% | 6.8% | 0.880 | 0.822 |
| Neither, and it threw one kind of the run | 458,034 | 12.9% | 0.7% | 1.4% | 10.4% | 0.969 | 0.971 |
| Neither, and none | 2,450,150 | 14.3% | 7.2% | 1.3% | 13.9% | 1.013 | 1.004 |
| All | 3,362,744 | 13.3% | 5.6% | 1.3% | 12.9% | 1 | 1 |

"Over its rank" is the copies that came up over the copies expected if
every unseen card of the rank were as likely to be each card of that rank
that followed; `Outlook` weighs the ranks already.  "Left free" counts only
the cards the other seat did not take, by a win or a claim, which are the
ones this seat may answer.

- **The evidence does not find the dead kinds.**  The oracle's sharpest
  reading is a kind whose every unseen copy the other seat keeps.  That is
  1.3% of the kinds, and 1.2% to 1.4% in every row but the smallest.
- **What it holds less of does not come more.**  The other seat keeps half
  as much of a kind it threw (7.2% against 14.3%), and the kind comes up
  0.92 times as often, not more: the cards it holds loose are the ones it
  throws next, and a kind it threw has none loose left (0.5% against
  7.2%).
- **Left free, the rows are level**: 0.988 for a kind it threw, 1.019 for
  one it let go, 1.004 for one it has shown nothing about.  The fresh kinds
  come up more and it takes more of them.  By the copies of the run not
  seen the cells run from 0.939 to 1.046; only the 4,804 copies with two
  kinds of the run thrown, one in 700, are far from 1, at 0.822.
- **The pong of T's trace is not `live`.**  The reading there was that a
  card the other seat has use for is deader to this seat than the DP
  knows.  A fresh kind is as live as one thrown or let go.

**The inference** is `Giveaway::chance`, the last column by `Evidence`
with the copies of the run not seen by twos, as `Giveaway::cost` has them.
`Graph::weighed` counts each copy of a live card as its chance of one, in
the entries and in the count of its rank, so every card weighed alike is no
card weighed; `ExpectBot::counting` builds its graphs so.  It is off unless
asked for; `COUNTING=1` asks in `ab/`.

Approximations: the chances are over the rest of the round, one table for
every stage of it; they weigh the cards that come and not the pruning nor a
leaf's curve past the gate, which read `live` whole; and they learn
`HeuristicBot`, as `Giveaway::cost` does.

**The runs**, declared 2026-11-26 before any data; two players at 10 胡,
control `8a8ecf9`, `RESHUFFLE=8`, queued through pueue from one frozen copy
of the working tree.  The smoke run was 300 deals of seed base 7 with the
chances of an earlier cut of 60 rounds (64 part) and gives no verdict; the
table is measured on other deals (`kept`, seed 700).

On the web setting, `EXPECT=new GATE_CARDS=10 SHALLOW=0` against
`BASELINE_EXPECT=new BASELINE_SHALLOW=0`:

| Run | Candidate | Deals | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| Z₂ | as it ships: owes zero deals that part | 2,000 | 1805000000 | 0 | +0.000 ± 0.000 | [+0.000, +0.000] | zero deals part |
| L | `COUNTING=1` | 40,000 | 1805100000 | 9,929 | −0.006 ± 0.009 | [−0.025, +0.012] | insufficient data; closed |

On the desktop setting, `GATE_CARDS=14 SHALLOW=17` against
`BASELINE_GATE=14 BASELINE_SHALLOW=17`, only if L is a win:

| Run | Candidate | Deals | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| L′ | `COUNTING=1` | 16,000 | 1805200000 | | | | not run: L is not a win |

How they are read:

- **L is 40,000 deals** for a standard error near 0.008: the smoke run's
  was 0.093 over 300.  The table says there is little to find, so the run
  is there to close the evidence, not to look for a gain.
- **`counting` ships at two players and 10 胡** if L and L′ both win and
  the gatekeeper of `danger` does not lose.
- **This evidence is closed for `live`** if the upper end of L's CI is
  under +0.05, half of what the oracle gets on the web: it is not run
  again, and `counting`, `Giveaway::chance` and `Graph::weighed` are taken
  out.  Otherwise a straddling CI is insufficient data, and a loss is
  traced first.

Read 2026-11-26.  The bots as they ship part on no deal with the hook in
(Z₂).  `counting` parts on one deal in four and reads −0.006 ± 0.009 points
a deal; its seat wins 50.24% of the deals against the control's 50.26%.
The upper end of the CI, +0.012, is under +0.05, so this evidence is closed
for `live` as declared: the table said there was nothing to weigh, and
weighing by it moves nothing that 40,000 deals can see.  L′ was not run.
`ExpectBot::counting`, `Giveaway::chance` and `Graph::weighed` are taken
out in the commit after the one that records this; `examples/kept.rs` and
the table stay.

## Batch 4: the other seat's hand from its whole history

Designed 2026-11-26, base `8a8ecf9`.  Batch 3 read each card by what is
public about it, and both readings came to next to nothing: the chances of
`live` are level, and the stage says when the other seat is waiting and not
on what.  What is left is the whole of it at once: a hand that would have
done everything the other seat did.

**The posterior.**  Nothing drawn enters a hand, so every card that left
the other seat's hand is public, and a guess at the hand it holds now is a
guess at the hand it was dealt.  Deal that hand, replay the round with
`HeuristicBot` holding it, this seat doing what it did, and the guess fits
when every public event comes out as it went: each discard, each claim,
and each card it let go that it would have won on, konged or claimed.
`HeuristicBot` has no dice, so among hands dealt evenly from the unseen
cards the ones that fit are the posterior, each as likely as another.

**The measurement** is `examples/posterior.rs`: the web page's bot against
`HeuristicBot` at a table of two, each deal once in either seat, and at
every discard valued in points 50 hands that fit, by the other seat's count
of concealed cards:

- the hands dealt for one that fits, in bits: what the history says;
- how much of the spread of the copies it holds of each kind the
  posterior's mean takes off an even deal's: `live`;
- the chance the posterior gives a win on a discard it does win on and on
  one it does not, beside the same of 200 hands dealt evenly: `win`;
- what discarding by value less that chance of 9 points keeps a round, one
  card ahead, at 0.29 (what T's deals paid for a win) and at 1: beside the
  oracle's win half, 0.715 over `giveaway`'s rows, and the stage's 0.010.

Two samplers.  Dealing evenly and keeping what fits is exact, and thins
out: one hand in thousands fits late in a round (the debug smoke, 3 rounds
of seed 7: 1.3 bits with 18 cards or more, 4.0 with 15 to 17).  A chain
reaches the rest: from the hand the other seat holds it swaps a card of the
hand for one of the pile, keeps the swap if the hand still fits, and takes
a hand every 60 swaps after 300.  No seat could start there; the chain
says what the posterior holds, not that a bot can sample it.

**The runs**, declared 2026-11-26 before any data, queued through pueue
behind L from one frozen copy of the working tree.  Not an A/B: nothing
plays by it, and there is no verdict.

| Run | Sampler | Rounds | seed | Status |
| --- | --- | ---: | ---: | --- |
| P₁ | `PROPOSALS=20000 SAMPLES=50`: dealt evenly | 200 | 800 | queued |
| P₂ | `CHAIN=60 SAMPLES=50`: the chain | 200 | 800 | queued |

How they are read:

- **The chain is read only where it agrees with P₁**: its rows with 12
  cards or more against P₁'s that are not thin.  A chain that sticks to
  the hand it started from flatters the posterior.
- **A sampler is designed** if the posterior keeps a third of the oracle's
  win half or more, one card ahead: `Giveaway`'s tables kept a sixth of
  their oracle and lost.  Under a tenth closes it.  Between the two, the
  `claim` half is measured the same way before deciding.
- **It learns `HeuristicBot` to the letter.**  A person plays no hand the
  way `HeuristicBot` would, so against one nothing fits; a sampler that
  ships needs a looser test of fit, and the gatekeeper before anything
  else.

Approximations: every win costs 9 points, the mean of `giveaway`'s rows;
this seat's eat that a pong took back is replayed as its pass; the chain's
hands are not independent, and the noise taken off the posterior's spread
counts them as if they were.

## Closing each batch

As `CLAUDE.md` says: `cargo fmt`, clippy, the three test suites, the docs,
`CHANGELOG.md`, a proposed commit message, and a commit only when the user
says so.  Win or lose, a run's numbers, deal count, seed base, control SHA
and verdict are written into the tables above, and the progress table is
updated.

## Not doing

- **No oracle in the library, the web page or the desktop app.**  They
  cheat; `gone_mut` is the one hook, and it is shut without `peek`.
- **No tuning on the verdict deals.**  The oracles have no constant to
  tune; the two tables are measured on other deals (`hazard`, seed 400).
- **No four-player or 8-胡 oracles yet.**  `danger` asks one other seat and
  is measured at two players and 10 胡; `Oracle::new` refuses another table,
  but for `live`.
- **No oracle that sees the pile.**  It would bias `RESHUFFLE`, and nothing
  public says anything about the pile's order.

## Risks and open questions

- **An oracle that reads zero closes less than it seems.**  It closes
  "this input, corrected, in the DP as built".  `live` and `waits` both
  enter a model that fixes them at the root.
- **The opponent is always `HeuristicBot`.**  `danger` asks it for its
  claim, and `live` reads its layouts.  A person keeps and claims
  otherwise.
- **The oracles may overlap.**  They are measured one at a time; two that
  win need not add up.
- **The desktop bot searches deeper**, so an input may be worth less to
  it, as the tables of two at 8 胡 were (+0.445 on the web, +0.154 on the
  desktop).
