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
known.

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
| 0 | `examples/hazard.rs`, `ab/src/oracle.rs`, `ExpectBot::outlook_mut`, this document | No; owes zero deals that part | None | Code written; the zero runs are queued |
| 1 | The oracles and the Peeker at two players, and the bot that ships against `HeuristicBot` | No: oracles never ship | None | Queued 2026-11-25 |
| 2 | h by the fewest concealed cards into `Outlook` | Yes | `cards` wins on the web and on the desktop | Waits on batch 1 |
| 3 | An inference for each open channel, largest first | Yes | Its oracle wins on both settings; a gatekeeper is in place | Waits on batch 1 |

## The oracles

An oracle is `ExpectBot` as it ships with one input replaced by what the
opponent's hand says.  It is a probe, not a ceiling: it says what the truth
in that one input is worth to the DP as it is built.  Only the Peeker
bounds every inference from above, and it has its own flaw (it also removes
the search's fault of deciding separately in each sampled deal).

They live in `ab/src/oracle.rs` and are seated with `ORACLE=`.  Each reads
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
- `examples/hazard.rs` and the two tables it printed, in `ab/src/oracle.rs`.
- `ab/src/oracle.rs` and `ORACLE=` in `ab/src/main.rs`.

The zero runs (declared 2026-11-25 before any data; control `1430e6b`,
`RESHUFFLE=8`, candidate `EXPECT=new`, no `ORACLE`):

| Setting | Table | Control | Deals | seed base | Part |
| --- | --- | --- | ---: | ---: | ---: |
| Web | two players, 10 胡 | `BASELINE_EXPECT=new BASELINE_SHALLOW=0` | 2,000 | 1803000000 | |
| Web | four players, 10 胡 | `BASELINE_EXPECT=1 BASELINE_SHALLOW=0` | 2,000 | 1803100000 | |
| Desktop | two players, 10 胡 | `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17` | 500 | 1803200000 | |

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
| A | `waits` | 1803300000 | | | | |
| B | `cards` | 1803400000 | | | | |
| C | `live` | 1803500000 | | | | |
| D | `danger` | 1803600000 | | | | |

**The same on the desktop setting**, 8,000 deals each, `RESHUFFLE=8`,
`GATE_CARDS=14 SHALLOW=17` against `BASELINE_GATE=14 BASELINE_SHALLOW=17`.
Each runs only if its web run is a win (`pueue add --after`, as
`AFTER=` of `ab/queue.sh` does it).

| Run | `ORACLE` | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | ---: | ---: | ---: | --- | --- |
| A′ | `waits` | 1804100000 | | | | |
| B′ | `cards` | 1804200000 | | | | |
| C′ | `live` | 1804300000 | | | | |
| D′ | `danger` | 1804400000 | | | | |

**The Peeker against the bot that ships**, 3,200 deals as on 2026-11-17,
`RESHUFFLE=0` (every replay would be a search), candidate `SAMPLES=128
PEEK=1`, control `BASELINE_EXPECT=new BASELINE_GATE=14 BASELINE_SHALLOW=17`,
the desktop bot.

| Run | seed base | Differ | Candidate − control | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| P | 1803700000 | | | | |

**The bot that ships against `HeuristicBot`**, which
[`variants.md`](variants.md) lists as never measured under the rule in
force.  Candidate `EXPECT=new`, control the default seat, `RESHUFFLE=8`.

| Run | Table | Setting | Deals | seed base | Part | Candidate − control | 95% CI | Verdict |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| H2 | two players | Web | 10,000 | 1803800000 | | | | |
| H3 | three players | Web | 10,000 | 1803900000 | | | | |
| H2′ | two players | Desktop | 4,000 | 1804000000 | | | | |

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
  waiting; `cards` is its public floor.
- **`danger`** is layers 1 to 3 of `stronger-bot.md` with a price in
  points; none of the three won at four players, where the cost is shared
  among three seats.
- **A gatekeeper first.**  An inference learns `HeuristicBot`'s habits.
  `stronger-bot.md` asks for a second opponent that must not regress
  before any such layer; the `guest` example's host
  (chunhua1972/four-color-game's Normal AI) is the one at hand.

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
- **No four-player or 8-胡 oracles yet.**  The tables are measured at two
  players and 10 胡 and `Oracle::new` refuses another table, but for
  `live`.
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
