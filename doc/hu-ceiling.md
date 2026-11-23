# The 胡 ceiling: design

Written 2026-11-18, base `6d5a2a8`; batches 0 to 3 are complete and two items
of batch 4 are done, with the progress in the table below.  This document
replaces the "cap used only to refuse pair eats" experiment in route 2 of
`doc/stronger-bot.md`: the arithmetic becomes exact, and the engineering is
split into five batches, each with a gate to pass before the next is done.

## The question to answer

The most 胡 a hand can reach without luck can be computed.  Does
`HeuristicBot` get stronger for knowing that number?  The first use is pair
eats: for the ones that would drop the ceiling below the threshold, is it
better not to eat?

## Progress

Each batch is about one session.  Update this table when a batch is done.

| Batch | Output | Changes play | Gate | Status |
| --- | --- | --- | --- | --- |
| 0 | The arithmetic and the statistics into the repo | no | none | done |
| 1 | The `ceiling` solver and its tests | no, to be measured at zero deals that part | all tests pass | done |
| 2 | The probe's A/B, with groups declared beforehand | candidate | see batch 2 | done: loss overall (−2.173 ± 0.008, CI [−2.189, −2.156]); A +0.0143 and B +0.0254 points a deal, both CIs above 0 → batch 3 (800,000 deals, seed 1794940763, control 168e1e4) |
| 3 | The narrowed rule, an A/B on a new seed | yes | commit only on a win | done: win (+0.053 ± 0.002, CI [+0.049, +0.056]; 800,000 deals, seed 1794947206, control 5b62dc6) |
| 4 | Other uses of the ceiling | candidate | a win in batch 2 or 3 | in progress: the node counts are measured and dynamic programming does not take over (`research/dp-nodes.md`); the floor on `short`, win (+0.002 ± 0.000, CI [+0.001, +0.003]; 200,000 deals, seed 1794954270, control 7ad669b); the pair-eat rule in `draw`, loss (−0.002 ± 0.000, CI [−0.003, −0.001]; 400,000 deals, seed 1794956595, control 39d37b2) |

## Why the original experiment was replaced

The arithmetic of `research/room-for-melds.md` counts "how many three-card
melds there is still room for", and has three gaps:

- **No price.**  The cap only makes `short` take off a few more half 胡, 64
  each; eating a pair saves 256 for two loose cards fewer.  The arithmetic
  says "cannot be made"; the grade takes it only as "a bit further".
- **Only one layout is looked at.**  Whether a general (將帥) counts as a
  single or goes into 將士象, and whether soldiers of three colours wait for
  the fourth colour, each change the answer to "how many more pairs are
  needed".
- **Four-card melds are not counted.**  Kongs and soldiers of four colours
  are not in it.

The prize is small too: the bound estimated by grouping after the fact for
"cap only" is about +0.006 points a deal.  So the engineering goes behind
the measurement, and stops if nothing can be measured.

## The arithmetic: seven slots

A winning hand is always 21 cards; see it as 7 slots of three cards, each
slot 1 胡 at base:

**胡 = 7 + 2k + the number of 將士象 + 2 × (concealed triplets + soldiers of
three colours + soldiers of four colours + kongs) + bonus**

- **k = (singles − pairs) ÷ 3.**  The total is 21 cards, so it is always an
  integer.
- **Singles:** the lone generals, plus the extra card of each four-card
  meld.
- **Bonus:** soldiers of four colours +1, exposed kong +2, concealed quad
  +4.
- **Winning:** the terms after the 7 total at least 3 (1 at 八胡).

From this:

- **A general matched with a pair is a base slot.**  A general by itself
  gives no extra 胡; when the singles outnumber the pairs by three cards,
  those three cards make a slot of 3 胡, equal to a concealed triplet.
- **A kong is a concealed triplet's slot plus a single worth 3 胡** (5 胡 for
  a concealed quad), and soldiers of four colours are the slot of three
  colours plus a single worth 2 胡.  Matched with a pair, a kong has +4 and
  four colours +3, enough alone.
- **The number of free pairs is the remainder of (singles − pairs already
  eaten) divided by 3.**  Eating one more pair at remainder 0 lowers k by 1:
  the ceiling drops 2 胡, and two more pairs have to be found.  This is the
  same condition as the old arithmetic's "O divided by 3 leaves 2".

Verification: over 20,000 rounds of `HeuristicBot` self-play at four seats
(seed 1795000000), the identity holds hand by hand on all 19,404 winning
hands.  The statistics of the same rounds:

| | |
| --- | ---: |
| Lone generals a hand | 2.76, of which 0.70 were drawn |
| Pairs a hand | 2.12 |
| k = −1 / 0 / +1 / +2 or more | 5.5% / 56.4% / 35.2% / 2.9% |
| Wins that would fall short of ten 胡 without the 2k | 9.8% |

The statistics program is `examples/census.rs`; the arithmetic and the full
tables are in `research/seven-slots.md`.

## Definition of the ceiling

The **ceiling** is the most 胡 a layout of a hand's final 21 cards can have,
under the assumptions below:

- Any card not yet seen comes when it is wanted, in as many copies as are
  still unseen.
- At most `generals` generals come from outside (the fourth card of a kong
  is not counted).

The final layout is made of three kinds of thing: melds already exposed;
melds made wholly of concealed cards, scored as concealed; and melds of
concealed cards kept plus one outside card, scored as exposed.

| Plus one outside card | Concealed cards kept | 胡 |
| --- | ---: | ---: |
| Pair | 1 | 0 |
| Pong | 2 | 1 |
| Exposed kong | 3 | 6 |
| 將士象, 車馬炮 | 2 | 2, 1 |
| Soldiers of three colours | 2 | 3 |
| Soldiers of four colours | 3 | 5 |
| Lone general (counts as one outside general) | 0 | 1 |
| 士象 plus an outside general (counts as one) | 2 | 2 |

Constraints:

- **Card count.**  With n concealed cards, e cards in the exposed melds, m
  outside cards and d discarded, d − m = n + e − 21.  A hand awaiting a card
  has d = m − 1, and a hand that has just eaten and not yet discarded has
  d = m.
- **Generals cannot be discarded**, nor ponged or eaten into a pair.
- **Three or four identical cards kept are a concealed triplet, a concealed
  quad or an exposed kong**, and cannot be split.
- **Every outside card must have a copy not yet seen**, as many copies as
  the times the same card is wanted.

What it deliberately ignores, which makes it an upper bound: who can eat
whose card, that the card just eaten cannot be discarded, the order of
events, 相公, and the other seats taking the cards first.

`generals` at 0 is "without luck"; at 1 it is "one more general drawn".
Both are needed, because a winning hand has 0.70 drawn generals on average,
and looking at 0 alone would judge live hands dead.

## Batch 0: the arithmetic and the statistics into the repo

- `examples/census.rs`: the statistics above, written the way
  `examples/dealt.rs` is, with a `[[example]]` entry added to `Cargo.toml`.
- `research/seven-slots.md`: the arithmetic, the verification of the
  identity, the full tables of statistics, the tools and the seed.
- `src/` is not touched, and no A/B is needed.

## Batch 1: the `ceiling` solver

A new module `src/ceiling.rs`, one pure function:

```rust
/// The most 胡 `hand` can win with beside `exposed`, if every card in
/// `live` came when wanted and at most `generals` generals came from
/// outside; `None` if no layout of 21 cards exists
pub fn ceiling(rules: &Rules, exposed: &[Meld], hand: Hand, live: Hand, generals: usize) -> Option<u32>
```

`live` is computed as in `HeuristicBot::start`.  `ab/` and batch 2's
candidate both need it, so it is public.

Method:

- **Blocks.**  Cards meld with one another only within nine blocks: one of
  將士象 and one of 車馬炮 for each of the four colours, and one of soldiers.
  Outside cards too come only from the same block.
- **Enumerate each block.**  Split the block's concealed cards into the
  kinds of meld above and the cards discarded, and for each (d − m, outside
  generals used) record the most 胡.  A block is at most 16 cards and is
  enumerated directly, the result stored under the key (cards in hand,
  cards unseen).
- **Merge.**  Stack the nine blocks one by one, keeping the most 胡 for each
  (d − m, generals); at the end read the cell d − m = n + e − 21 and add the
  胡 of the exposed melds.

Tests:

- **Hand-worked cases.**  O = 2 to 8 from the old arithmetic's table, a 搭子
  plus a lone card (cannot be made), 大肚 (`車車車馬炮` wins only on the fourth
  車), a general matched with a pair, three generals, a concealed triplet
  plus a kong.
- **An invariant of self-play.**  At every decision point of the winner
  before the win, when no more than `generals` outside generals come after
  it, the ceiling is no less than the final 胡.  It lives in
  `tests/round.rs`.
- **Zero deals that part.**  `ab/run.sh 20000` reads 0 deals that differ.

Two things that came out of batch 1, which batch 2 must follow:

- **`live` and the card just discarded.**  At a decision to eat or pong, the
  invariant's test counts the card on the table in `live`, because it can
  still be had; the `live` of `HeuristicBot::start` does not include it.
  When the probe compares "pass" with "eat", "pass" uses the bot's `live`
  (the card goes to the discards), and "eat" puts the meld in `exposed`.
- **Three identical cards may be split by discarding one first.**  The
  ceiling ignores order, so `車車車馬炮` without the fourth 車 still counts two
  exposed 車馬炮 (discard one 車 first).  `locked_completions` does not allow
  that eat at the moment; the ceiling is an upper bound, and this is
  deliberate.

## Batch 2: the probe

The candidate (the probe) changes one thing only: **when it is its turn to
decide on an eat or pong, a pair eat that would lower the `generals = 0`
ceiling is left out of the choices.**

- It blocks only real decisions (`HeuristicBot::claims`), and leaves the
  imagined pair eats in `draw` alone.  So the grade of "pass" is exactly the
  control's, and the only parting can be "pair eat → something else"; any
  other kind of parting means the implementation is wrong.  The bot's
  forecast of its own future therefore disagrees with its actual play; this
  is deliberate, to be dealt with if batch 3 wins.
- Pair eats that leave the ceiling unchanged go on as before.  They are well
  worth it on average (never eating is −1.50 points a deal).

Trace: for every parting `ab/` prints four more numbers: the ceiling before
the eat and after it, with `generals` at 0 and at 1.  Neither the probe nor
the trace is on `main`; each is kept as a patch;
`hu-ceiling-trace.patch` carries `ab/tally.py` and `ab/groups.py`:

```sh
git apply research/hu-ceiling-probe.patch research/hu-ceiling-trace.patch
TRACE=1 ab/run.sh 800000 > out.txt
python3 ab/groups.py out.txt 800000
```

The groups declared beforehand, by the ceiling after the eat:

| Group | Condition | Expectation |
| --- | --- | --- |
| A | Below the threshold even with `generals = 1` | better not to eat |
| B | Below the threshold with `generals = 0`, reaches it with `generals = 1` | slightly better not to eat |
| C | Still reaches the threshold with `generals = 0` | unknown, eating may be better |

Measurement:

- A new seed base, `RESHUFFLE=8`, control `HEAD`.
- The deal count is set before any score is seen.  First time 5,000 deals on
  another seed and throw the result away; 800,000 deals by default, changed
  to 400,000 deals if the projection exceeds three hours.  Over an hour,
  detach by the global rule and write the state to `in-flight-runs.md` in
  memory.

Gate:

- **Win overall** (CI above 0): the probe itself may be committed.  The
  groups only decide whether batch 3 narrows it further.
- **Not a win overall, but the CI of A, of B or of both is above 0**: do
  batch 3.
- **Otherwise**: this line stops here, is written up in `research/` and
  `CHANGELOG.md`, and the bot does not change.  A group whose CI straddles 0
  is insufficient data, not "may be kept".

Results (800,000 deals, seed base 1794940763, `RESHUFFLE=8`, control
`168e1e4`, 357,189 deals part):

| | Points a deal | 95% CI | Verdict |
| --- | ---: | --- | --- |
| Overall | −2.173 ± 0.008 | [−2.189, −2.156] | loss |
| A | +0.0143 | [+0.0133, +0.0154] | above 0 |
| B | +0.0254 | [+0.0228, +0.0279] | above 0 |
| C | −2.213 | [−2.229, −2.196] | below 0 |

The probe is not committed; A and B pass the second gate, so batch 3 is
done.

## Batch 3: the narrowed rule

Block only the groups of batch 2 whose CI is above 0.  A new seed base, the
deal count set before the run from the parting rate and the standard
deviation measured in batch 2, commit only on a win.  Batch 2's groups were
read on those deals, so the verdict rests on this run alone.

The rule: a pair eat is left out of the choices only when it lowers the
`generals = 0` ceiling **and** drops it below the threshold (or leaves no
layout); that is A and B.  The deal count is set at 800,000: from batch 2's
parting rate and the standard errors of A and B, the effect is projected
at about +0.04 points a deal and the standard error at about 0.0014.
Timing 5,000 deals (seed 1, the result thrown away) took 14 seconds.

Results (800,000 deals, seed base 1794947206, `RESHUFFLE=8`, control
`5b62dc6`, 59,885 deals part, all of them "pair eat → something else", 59,650
of them to "pass"):

| | Points a deal | 95% CI | Verdict |
| --- | ---: | --- | --- |
| Overall | +0.053 ± 0.002 | [+0.049, +0.056] | win |
| A | +0.0173 | [+0.0162, +0.0184] | above 0 |
| B | +0.0353 | [+0.0323, +0.0383] | above 0 |

The narrowed rule goes into `HeuristicBot::claims`, and the web page's hint
changes with it.  The imagined pair eats in `draw` have not followed yet and
are left to batch 4.  The trace used for the measurement is still
`research/hu-ceiling-trace.patch`; its `tally.py` does not recognise the
lines that carry ceilings, so the groups need `groups.py`.

## Batch 4: other uses

Designed only on a win, each with its own A/B.  Originally the four items
below could also be replaced at once by one dynamic programme of
single-player expectation (`doc/stronger-bot.md`, route 3), in which the
ceiling is an exact pruning.  The nodes were counted (`research/dp-nodes.md`,
`examples/nodes.rs`): a freshly dealt hand of 20 averages 7 × 10⁵ nodes and
6 × 10⁷ edges, 22 seconds a decision, two to three orders of magnitude more
than route 3 estimated; only hands of 12 cards or fewer take tens of
milliseconds.  So dynamic programming does not take over batch 4 and the
four items are done one by one; the gated dynamic programme is left to
route 3.  The candidates:

- **`dead`.**  It now assumes every card is exchanged for a general, so
  loose that it blocks almost nothing.
- **`short` and `hope`.**  Replace the 搭子's count of half 胡 with "how far
  the ceiling is from the threshold".  Done, see below.
- **Discards.**  Do not discard the card that drops the ceiling below the
  threshold.
- **`draw`.**  Make the imagined pair eats agree with the actual rule.  The
  floor on `short` already lets `take` see the ceiling, but that is not
  batch 3's rule.  Done, loss, see below.

### The floor on `short` (measured, win)

`short` was 2 × (threshold − the melds' 胡) − the 搭子's hope, the hope being
the 胡 of the meld each 搭子 can be eaten into, whether or not there is room
for them.  The candidate: `short` is at least 2 × (threshold − ceiling), the
ceiling computed by `ceiling::reach` without the 胡 of the exposed melds;
the eats and pongs `take` imagines go the same way, so `draw` follows along.
Three forms, all against `7ad669b`:

| Form | Deals | seed base | Part | Points a deal | 95% CI | Verdict |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| Replaced outright by 2 × (threshold − ceiling), g = 0 | 5,000 (smoke test, seed 1) | 1 | 1,530 | −0.380 ± 0.064 | [−0.506, −0.255] | discarded |
| The larger of the two, g = 0 | 200,000 | 1794953471 | 8,482 | −0.036 ± 0.003 | [−0.043, −0.029] | loss |
| The larger of the two, g = 1 | 200,000 | 1794954270 | 555 | +0.002 ± 0.000 | [+0.001, +0.003] | win |

What was learnt:

- **With `short` replaced there is no gradient towards the threshold.**
  Most hands' ceilings pass the threshold, `short` becomes 0, and the bot no
  longer chases 胡.
- **g = 0 judges live hands dead.**  In the trace of the first 20,000 deals
  the loss is all in "the control pongs, the candidate passes" (207 deals,
  −3.35 each): after the pong the g = 0 ceiling drops below the threshold,
  but one drawn general is enough; the partings at a discard are +0.009 ±
  0.007.  g = 0 is right for the pair-eat rule (batch 3), and not for pongs.
- **g = 1 changes only 0.28% of the deals,** and the effect is only 0.002
  points a deal; this item's prize was estimated under +0.006 to begin
  with.

An example at a discard (the test
`a_soldier_between_two_others_splits_when_short`): 5 胡 exposed, and in hand
a red 兵, a yellow 兵, a yellow 兵, a green 卒, a white 車 and a white 馬, with
one card to discard.  Discarding the 車 or the 馬, the ceiling is 11: the
red and yellow, and the yellow and green, each eat a third colour.
Discarding a yellow 兵 leaves 10, and only one way there, "soldiers of four
colours plus a pair of 車"; waiting for the 包 gives only 9.  `HeuristicBot`
now discards the yellow 兵 whether or not the 胡 are enough: both layouts
take the concealed soldiers of three colours first, the extra yellow 兵 is
the loneliest loose card, and `hope` does not see the 6 胡 of the split.  So
the rule for discards must compare the ceilings after each discard; looking
only at "drops below the threshold" would miss this hand.

### The pair-eat rule in `draw` (measured, loss)

The candidate: when `draw` imagines the card drawn, the pair eats batch 3
blocks in `claims` are left out too (as before when a general is drawn and
must be eaten), so that the bot's forecast of the next card agrees with its
actual play.  The judgement of `claims` itself does not change.

| Deals | seed base | Part | Points a deal | 95% CI | Verdict |
| ---: | ---: | ---: | ---: | --- | --- |
| 400,000 | 1794956595 | 3,072 | −0.002 ± 0.000 | [−0.003, −0.001] | loss |

The deal count was set before the run from a smoke test of 5,000 deals (seed
1, discarded: 0.94% part).  Control `39d37b2`.

Trace (the first 100,000 deals of the same seed base,
`research/hu-ceiling-trace.patch`): the partings are all "the control
passes, the candidate eats or pongs".  The next seat's "pass" is valued by
`draw`; with the pair eats blocked the expectation of the draw falls, and
the eats and pongs that used to lose to "pass" win.  Every kind of extra eat
or pong loses:

| Control → candidate | Part | Points each | Standard error | Points a deal |
| --- | ---: | ---: | ---: | ---: |
| pass → pair eat | 150 | −0.884 | 0.342 | −0.0013 |
| pass → 車馬炮 eat | 284 | −0.308 | 0.166 | −0.0009 |
| pass → 將士象 eat | 214 | −0.349 | 0.195 | −0.0007 |
| pass → pong | 113 | −0.097 | 0.436 | −0.0001 |

What was learnt: `draw` overvalues the draw, which amounts to adding a
little to "pass", and that bias is useful.  Agreement is not the goal; to
change `draw`, the threshold of eats and pongs against "pass" in `claims`
has to be looked at with it.  The candidate is kept as
`research/hu-ceiling-draw-pairs.patch`.

## Wrapping up each batch

By `CLAUDE.md`: `cargo fmt`, clippy, the three sets of tests, the docs,
`CHANGELOG.md`, propose a commit message, and commit only when the user says
so.  A measurement, win or lose, is recorded with its numbers, deal count,
seed base, the control's SHA and the verdict, and the progress table of this
document is updated.

## What is not done

- **Not put in susik.**  The ceiling needs to know how many 胡 exposed and
  concealed cards are each worth and which cards are not yet seen; that is
  this crate's business.
- **No probabilities.**  The ceiling only answers whether a hand can get
  there, not how easily.
- **Not wired into `MonteCarloBot`, the web page or the desktop app.**  The
  hint is `HeuristicBot`, and changes only when the bot wins.

## Risks and open questions

- **The prize may not be measurable.**  The estimated bound is about +0.006
  points a deal.  "Cap only" had a standard error of 0.004 points over
  100,000 deals; at a similar parting rate, 800,000 deals give about 0.0015
  points, and the bound is only four times that.  One of the likeliest
  endings is to stop at batch 2 with insufficient data.
- **Group A may be almost empty.**  `short` already fines hands that cannot
  be made.  That is a result in itself: the bot was avoiding them already,
  and this line ends.
- **The solver is bigger than estimated.**  The soldiers' block has the
  constraints of four colours and of the counts of outside cards, and is the
  easiest place to get wrong; the invariant's test is there for it.
- **The statistics come from `HeuristicBot`'s own wins.**  The 2.76
  generals and the 0.70 drawn both carry this bot's habits.
- **The probe records only the first parting of each deal.**  The effect of
  the later pair eats is charged to the first.
