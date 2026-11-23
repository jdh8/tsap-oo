# Dynamic programming for the single-player expectation: design

Written 2026-11-18, base `7034b41`.  This document takes over route 3 of
`doc/stronger-bot.md`: the core is a dynamic programming that reckons how
many points a hand is still worth, switched on only when the hand is short;
applications hang on it one by one, each with its own gate.  The core of
batch 1 is in `src/expect.rs`; the costs below are estimates except where
marked "measured".

## The questions to answer

With 10 concealed cards or fewer, `MonteCarloBot` searches with 128 samples
at 0.9 seconds a decision and beats `HeuristicBot` by +0.070 ± 0.026 points
a deal (`research/gated-monte-carlo.md`).  In the same range a hand can
reach only a few hundred to a few thousand sub-hands
(`research/dp-nodes.md`).  For a dynamic programming that works through all
of them:

1. How many milliseconds does a decision take?
2. Does it beat `HeuristicBot`?  If so, how does it compare with +0.070?
3. Where else can what the core computes be used?

## Progress

Each batch is about one session.  Update this table when a batch is done.

| Batch | Output | Changes play | Gate | Status |
| --- | --- | --- | --- | --- |
| 0 | This document | No | None | Done |
| 1 | The core: graph, sweep, tests, timing | No, and zero deals that part must be measured | All tests pass; 10 cards average within 50 ms | Done: 14 ms |
| 2 | The table model's four constants, measured from self-play | No | The predicted win rate matches the actual (see "Calibration") | Done: high, explained by h |
| 3 | `ExpectBot`, a 10-card gate, A/B | Candidate | Commit only on a win | Done: win, +0.076 |
| 4 | Applications, see the table below | Each its own | Batch 3 is a win | In progress: B done, win, +0.047; C's hint changed to `ExpectBot`, marking points inside the gate; D, the desktop opened to 14 cards, win, +0.132; D2 stopped |

Batch 1's 50 ms was declared beforehand: the census takes 11 ms at 10 cards,
and a core more than five times slower would mean the cost estimates below
are wrong and must be understood before going on.

Batch 1 measured (`research/dp-nodes.md`, "The graph of one decision", 200
rounds, seed 2): for a discard decision that leaves 10 cards, one shared
graph built from every card that may be discarded takes 13.0 ms to build and
0.9 ms to sweep; 12 cards take 52 ms and 13 cards 132 ms.  The gate is
passed, so measuring the four constants can begin.  `ab/run.sh 200000` (seed
base 1794971744, control `7d3157a`) reads 0 deals that part: no bot calls
this module.

Batch 2 measured (`research/outlook.md`, `HeuristicBot` self-play, constants
over 10,000 rounds at seed 20, calibration over 10,000 rounds at seed 21): h
and the public cards a draw both change a lot with the pile (h 0.018 to
0.12, the cards 3.2 to 1.3), so both became tables by draws left; e 0.277,
L −12.3, generals (將帥) 0.216.  The predicted win rate is high in nine bins
of ten, 0.382 against 0.291 overall, and its shape is monotonic.  Changing
one constant at a time (seed 23), h × 1.5 brings the whole to 0.306 against
0.290 and the four low bins inside their intervals, while e and the cards
each move it only a little: the bias has a direction and h explains it, so
the gate is passed.  `Outlook::default()` is still the measured values; the
multiplier of h is a question for batch 3.  `ab/run.sh 200000` (seed base
1794984588, control `c0aea83`) reads 0 deals that part.

Batch 3 measured (control `b920a26`, `RESHUFFLE=8`, 100,000 deals each,
declared under "A"): the measured h is +0.071 ± 0.008 points a deal, 95% CI
[+0.055, +0.087], 7,526 deals part; h × 1.5 is +0.076 ± 0.008,
[+0.060, +0.093], 7,631 deals part.  Both are wins; they differ by 0.005,
and the standard error of the difference is near 0.011, so it is within the
error.  As declared, the one with the higher point estimate is used: the h
of `ExpectBot::new` is the measured one × 1.5 (`END_SCALE`).  That is as
large as the gated Monte Carlo's +0.070 ± 0.026, yet a decision takes a
dozen or so milliseconds against 0.9 seconds.

## The model

### Why single-player is tractable

A drawn card does not enter the hand: it is turned face up for everyone, and
goes to the discards if nobody wants it.  The concealed cards change only on
an eat, a pong or a kong, by three or four fewer each time (counting the
card then discarded).  So from my seat a round is a string of public cards,
and to each I have only three responses: win (compulsory), take it and
discard a card, or pass.

The concealed cards plus the exposed cards always number 20 (21 when it is
my turn to discard), so the state needs only two things:

**State s = (the concealed cards, the total 胡 of the exposed melds)**

The same concealed cards can be reached through different melds with
different 胡, so the concealed cards alone are not enough; the count of
exposed cards follows from the count of concealed ones and need not be kept.
These are the nodes that `examples/nodes.rs` takes a census of.

### What I can do with each public card

| Where the card comes from | Win, kong, pong | Eat |
| --- | --- | --- |
| My draw | Yes | Yes; a general must be eaten |
| The previous seat's draw | Yes | Only if the previous seat does not eat it; not a general |
| The previous seat's discard | Yes | Yes; nobody discards a general |
| The other two seats' draws and discards | Yes | No |

Winning and konging are compulsory, and `Round::resolve` deals with them
before anyone is asked.  Nobody contests a pong: there are only four of a
card.  An eat is overridden by another seat's pong, and the previous seat
eats its own draw first.

### The recursion

t is how many public cards are still to come this round, and V(s, t) how
many points this state is worth:

**V(s, t) = h · L + (1 − h) · Σ p(c) · [e(c) · the best response when I may eat + (1 − e(c)) · the best response when I may not]**

**V(s, 0) = 0**

- **p(c)** is the probability that the next public card is c: the copies of
  c in `live` over the total of `live`.  `live` is the cards I have not
  seen, reckoned as in `HeuristicBot::start` and fixed at the root.
- **e(c)** is the probability that the card comes to me to eat.  A general
  can be eaten only when I draw it myself, which is another value.
- **h** is the probability that another seat ends the round before this card
  appears, and **L** the points I then pay (negative).  Both may vary with t
  and with the other seats' counts of concealed cards.
- **The response**, in order:
  1. I can win: 3 × the 胡, and it ends there.  The test is that of
     `Round::win`.
  2. I hold three of it: kong, compulsory.
  3. Otherwise the maximum over passing, ponging and (when I may eat) every
     way of eating.  A general I draw myself cannot be passed.
  4. Which card to discard after taking it is a maximum too:
     max V(s′ − d, t − 1), where d may be neither the card just taken nor a
     general (`discardable`).  Having no card to discard is a 相公,
     −3 × the threshold.
  5. Passing is V(s, t − 1).
- **Pruning.**  States whose `ceiling(generals = 1)` falls short of the
  threshold are not expanded but all merged into one node D that "will not
  win any more": D(t) = h · L + (1 − h) · D(t − 1).

The payoff follows `Game::settle`: when I win, each of the three seats pays
me my 胡; when another seat wins, I pay its 胡, whoever dealt in; an
exhausted round is 0.  So the unit is points, the same thing the A/B
measures.

### The graph is built only once

Every edge uses up exactly one public card, so V(·, t) looks only at
V(·, t − 1):

1. **Build the graph**, once a decision.  Breadth first from the roots,
   numbering the nodes and storing the edges in flat arrays:
   node → (card c, the points of a win or a kong) → the children by pong,
   the children by eat.  A child is the state after taking the card and
   discarding one.
2. **Sweep**, t from 1 to T.  Each pass walks the whole edge array once with
   nothing but additions, multiplications and maxima, two arrays as long as
   the node count taking turns as t and t − 1.

The dear part is building the graph (`ceiling`, `locked_completions`, the
hash table), and that is what the census measured.  The sweep is
edges × T floating-point operations, with T a few dozen at most:

| Concealed cards | Nodes (measured) | Edges (measured) | Build (measured, census) | Sweep, T = 25 (an estimate) |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 6 × 10² | 1.5 × 10⁴ | 11 ms | 4 × 10⁵ operations |
| 12 | 2.0 × 10³ | 5.8 × 10⁴ | 41 ms | 1.5 × 10⁶ operations |
| 13 | 5.4 × 10³ | 2.1 × 10⁵ | 114 ms | 5 × 10⁶ operations |
| 20 | 7.0 × 10⁵ | 5.8 × 10⁷ | 22 s | Unaffordable |

So what `research/dp-nodes.md` calls "the chances left are a multiplier of
about 60" multiplies only the sweep, not the build.  Batch 1 measured it:
the sweep is 6% to 9% of the build.

### The root

The root uses the actual position, not the model: `ClaimOptions` already
says whether I may eat or pong this card, and `View::locked` which card may
not be discarded.

- **Discarding:** each card d that may be discarded is worth V(hand − d, T).
- **Eating and ponging:** passing is V(hand, T); each eat or pong is worth
  the value of the best discard after it.
- **T** is reckoned from the pile: the draws left are
  `pile_len − Rules::dead()`, and the public cards of each segment (a draw
  and the discards before it) are added up from the table, less the card
  being decided on.

### The table model: four constants

Everything about the other seats is folded in here.  All of it is measured
from `HeuristicBot` self-play on fixed seeds, never tuned on the deals of an
A/B; the tool that measures is an example, and the numbers go into
`research/`.

| Constant | Meaning | First version | Measured (batch 2) |
| --- | --- | --- | --- |
| h | The probability, before each public card, that another seat ends the round | A table in segments by draws left | A table, 0.018 to 0.12 |
| L | The points I pay when another seat wins | One number: the mean 胡 of the other seats' wins, negated | −12.3 |
| e | The probability that a public card comes to me to eat | Two numbers: generals, the rest | 0.216, 0.277 |
| Public cards a draw | The conversion to T | One number | Also became a table, 3.2 to 1.3 |

`research/safe-cards.md` measured that the other seats' counts of concealed
cards spread the chance of being won on two hundredfold, so the dimension h
should gain next is the fewest concealed cards another seat holds
(`View::concealed_len`).  The first version leaves it out: before it is
added, calibration numbers must say that this is where the model is off.

### Known approximations

- **`live` is fixed at the root.**  A card can be wanted more times than it
  has copies left, and the cards the other seats take are not deducted from
  the probabilities.  Putting `live` in the state would multiply the nodes
  by several orders of magnitude.
- **The other seats' discards are taken as uniform.**  Drawn cards really
  are uniform to me; discarded cards are what the other seats do not want,
  skewed towards cards they have no use for.
- **Turn order is flattened.**  e is an average; in fact after I eat or pong
  the next card is the next seat's draw, and two cards go by before one I
  may eat.  The exact way is to put "whose card is next" into the time
  dimension, which makes the sweep four times as dear and leaves the build
  as it is.
- **My discards have no consequences.**  A discard may deal in, or may be
  eaten or ponged and so add public cards.  The danger of dealing in did not
  make the bot stronger in two experiments (`stronger-bot.md`, route 1), and
  the first version leaves it out.
- **Pruning at `generals = 1`.**  A hand that needs two generals from
  outside to reach the threshold is taken as never winning.
- **This round only.**  That the winner deals the next round is not counted.

Each of these is a candidate for an A/B of its own later, not work for the
first version.

## Calibration

The model has four constants and six approximations, and an A/B will only
say "won" or "did not win", not where it is wrong.  So batch 2 also measures
something that needs no A/B:

- At every decision point of self-play with 10 concealed cards or fewer,
  record the chance to win the DP predicts (the same sweep, with the payoff
  changed to 1 for a win and 0 otherwise) and whether the hand won in the
  end.
- Split them into ten bins by predicted value, and compare each bin's
  predicted and actual win rates.

The gate: all ten bins lie inside their own 95% intervals, or the bias runs
one way and one constant can explain it.  If the bias is large and has no
direction, fix the model first and do not run batch 3.

The predicted chance is reckoned by the DP's own play, while the hands
measured are `HeuristicBot`'s, so the actual values will be a little lower;
what to look at is the shape, not the absolute values.

## The shape of the code

One module, `src/expect.rs`.  `Outlook`, `Graph` and `Payoff` were written
in batch 1; `ExpectBot` is still a proposal:

```rust
/// The table as one seat models it: h and the public cards a draw, both by
/// draws left, L, and e for the rest and for generals
pub struct Outlook {
    pub end: Vec<f64>, pub cards: Vec<f64>, pub loss: f64, pub eat: f64, pub general: f64,
}
impl Default for Outlook { /* measured, research/outlook.md */ }

/// The hands a chain of claims can reach from the roots of one decision,
/// each a settled hand and its exposed 胡, and the edges between them
pub struct Graph { /* nodes, flat edges */ }

impl Graph {
    pub fn new(rules: &Rules, live: Hand, roots: &[(Hand, u32)], prune: Prune) -> Self;
    pub fn roots(&self) -> &[usize];

    /// The value of every node with `draws` draws left, under `payoff`:
    /// points for the bot, 1 for a win to get its probability
    pub fn values(&self, outlook: &Outlook, draws: usize, payoff: Payoff) -> Vec<f64>;
}

/// Plays by `Graph` with `cards` concealed cards or fewer, greedily before
pub struct ExpectBot { /* HeuristicBot, Outlook, cards */ }

impl ExpectBot {
    pub fn claims(&mut self, view: &View<'_>, options: &ClaimOptions) -> Vec<(Claim, f64)>;
    pub fn discards(&mut self, view: &View<'_>) -> Vec<(Card, f64)>;
}
impl Strategy for ExpectBot { /* the first of each */ }
```

- Batch 2 measured the public cards a draw falling with the pile from 3.2 to
  1.3, so like h they became a table in `Outlook`; `values` takes the draws
  left and lays out each card's h itself.
- `Prune` has two fields: how many generals from outside the ceiling uses
  (`None` does not prune), and the four ways of pruning discards from
  `research/dp-nodes.md` (`Discards`).  The default is 1 general and no
  pruning of discards: the 10-card gate has no need to prune, and pruning
  costs about 0.02 points a decision.
- `claims` and `discards` take the shape of `HeuristicBot`'s functions of
  the same names, the best first, so the web page's hint works by swapping
  one type, and the unit of its numbers goes from grade to points.
- The gate follows the first condition of `MonteCarloBot::gated`: the count
  of concealed cards after the discard.  Past the gate it returns
  `HeuristicBot`'s choice.
- The census in `examples/nodes.rs` now uses `Graph::new`, so the same code
  counts the nodes and builds the graph.
- A `Graph` is not reused across decisions.  The graph of the last decision
  holds most of this decision's nodes, but `live` has changed, and both the
  pruning and the probabilities must be redone; a build takes only a dozen
  or so milliseconds, so it is not worth it.

### Tests

The tests in `src/expect.rs`, batch 1:

- **A hand that can be worked out.**  Waiting on one card, with k copies in
  `live` out of n in all and h = 0: V = 3 × 胡 × (1 − (1 − k∕n)ᵗ).
- **Against brute force.**  Hands of 6 cards or fewer, expanded straight
  from the recursion with no graph, the eats and pongs decided by `options`
  of `round.rs`; the two agree within 10⁻⁹ (the additions come in another
  order, so not bit for bit).
- **Monotonic.**  With h = 0, no triplet and no general eaten, the chance to
  win does not fall as t grows, and lies between 0 and 1.  "V does not fall
  when `live` has one more wanted card" was removed: one more wanted card
  also lowers the probability of the other cards, so it is no theorem.
- **Pruning is lossless.**  When the ceiling takes the count of all the
  generals in `live`, the chance to win is the same pruned and unpruned.
  Generals must not be eaten for this: `live` is fixed at the root, so one
  general can be eaten several times, and the ceiling counts it once.
- **Legal**, batch 3.  One `ExpectBot` seated in the four-player rounds of
  `tests/round.rs` plays them out without breaking a rule.

## Applications

The core gives two things: how many points each choice is worth, and the
values of the same graph under other payoffs (the chance to win, the
expected 胡).  Each item below is an application with its own gate; any that
changes how a bot plays must run the A/B by the iron rules of `CLAUDE.md`,
with the deal count and the seed base fixed before the run.

| | Application | What it uses of the core | Changes play | Condition to start | Status |
| --- | --- | --- | --- | --- | --- |
| A | `ExpectBot`, a 10-card gate | The points of a choice | Candidate | Batch 2 passes | Done: win (batch 3) |
| B | The desktop app's opponents changed to `ExpectBot` | A | Yes | A is a win | Done: win (batch 4) |
| C | The web page's hint and opponents | A, compiled to wasm | Yes | A is a win; timing in wasm | Not started |
| D | The desktop's gate opened to 14 cards | A | Candidate | A is a win | Done: win, +0.132 |
| D2 | 15 to 17 cards, with `Discards::Live` in the graph | A, `Prune` | Candidate | D is a win | Stopped: too slow, no need to measure whether it wins |
| E | The hint shows the chance to win and the expected 胡 | Other payoffs | No | Batch 2's calibration passes | Not started |
| F | The two items left of batch 4 of `hu-ceiling.md` | A | Candidate | A is a win | Not started |
| G | The socket for opponent inference | p(c) and h replaced by inferred ones | Candidate | Route 1 has an inference to plug in | Not started |
| H | Monte Carlo's playouts and leaves | The points of a choice | Candidate | A is a win | Not started |
| I | Labels for long hands, to train an eval past the gate | The values of nodes | No | D is a win | Design: [`eval.md`](eval.md) |

### A: `ExpectBot` against `HeuristicBot`

Batch 3.  The candidate in `ab/src/main.rs` becomes `ExpectBot` with a
10-card gate.  Its decisions take milliseconds, so it uses the default
`RESHUFFLE=8`; the gated Monte Carlo could only use `RESHUFFLE=0`, with a
standard error of 0.026 over 40,000 deals.  The control is the
`HeuristicBot` of `main` HEAD.  The deal count is fixed once batch 1's
timing is in, and the run starts only after it is written here.

Batch 3's declaration (2026-11-18, written down before the run): control
`b920a26`, `RESHUFFLE=8`, a 10-card gate, two arms of 100,000 deals each.
Two versions of h are measured, each an experiment of its own:

- **Arm 1: the measured h** (`EXPECT=1`), seed base 1794988309.
- **Arm 2: h × 1.5** (`EXPECT=1.5`), seed base 1795088309.

The deal count comes from a trial run: 400 deals (seed base 1, not counted)
took 5 seconds with 8% parting and a standard error of 0.18, so 100,000
deals take about 20 minutes with a standard error near 0.011.  Each arm's
verdict is read by the iron rules on its own, and the better of the two arms
is not taken as the verdict; when both are wins, the one with the higher
point estimate is used, with a note of whether the difference between them
is within the error.

How to read it:

- **Win:** commit, and B to I may start.  The point estimate is compared
  with +0.070, but the two intervals will most likely overlap, so neither
  can be called the stronger.
- **Loss:** first trace the deals that moved the most, then check against
  the calibration's bias.  A loss with an accurate calibration means the
  problem is the single-player assumption itself, and B to H all stop.
- **Insufficient data:** add deals; they are cheap.

### B: the desktop app's opponents

`Core::new` in `web/src/lib.rs` gives the opponents
`MonteCarloBot::gated(10, None)` when `searching`.  Once A is a win they
become `ExpectBot`: each searched decision falls from 0.9 seconds to tens of
milliseconds.  What it replaces is another bot that beats `HeuristicBot`, so
an A/B of `ExpectBot` against the gated Monte Carlo is recorded as well
(`RESHUFFLE=0`, since every move on the control's side is a search); on
insufficient data `ExpectBot` is chosen for its speed, and `CHANGELOG.md`
states that it has not been shown to be stronger.

Batch 4's declaration (2026-11-18, written down before the run): control
`e123ba4`, `RESHUFFLE=0`, a 10-card gate on both sides.  Seat 1 is a
128-sample `MonteCarloBot` on the control's side (`BASELINE_SAMPLES=128`)
and `ExpectBot::new` on the candidate's (`EXPECT=1.5`); the other three
seats are all `HeuristicBot`.  100,000 deals, seed base 1794990944.

The deal count comes from a trial run: 800 deals (seed base 1, not counted)
took 3 minutes and 30 CPU minutes, with 36 deals parting and a standard
error of 0.197, so the standard error of 100,000 deals is near 0.018; with
an A/B of `../pons` also running on the machine, about 4 hours.

How to read it: on a win or insufficient data, the desktop app's opponents
become `ExpectBot` (`web/src/lib.rs` is already changed and tested, and is
committed only once the verdict is in); on insufficient data, state as above
that it has not been shown to be stronger.  On a loss, first trace the deals
that moved the most, and the opponents stay on Monte Carlo.

Batch 4's result: 4,336 of the 100,000 deals part, and seat 1's win rate is
24.51% for the control and 24.61% for the candidate; candidate minus control
is +0.047 ± 0.019 points a deal, 95% confidence interval [+0.011, +0.084], a
win.  The desktop app's opponents are now `ExpectBot`, at 14 ms a searched
decision instead of 0.9 seconds.

### C: the web page

The web page stayed on `HeuristicBot` because the gated Monte Carlo takes
0.9 seconds even natively.  If `ExpectBot` takes a dozen or so milliseconds
natively, wasm being several times slower is still within one frame.  The
condition is A's win plus an actual timing in wasm; the hint is the bot, so
A's A/B is the hint's A/B as well.

C's timing (2026-11-18, `examples/expect_time.rs`): seat 1 is
`ExpectBot::new` and the other three `HeuristicBot`, and each of its
`claims` and `discards` is timed, the very call the hint would make.  It is
single-threaded, so the same code compiles to `wasm32-wasip1` and runs under
node's WASI with `research/wasi.mjs`; node's wasm engine is Chrome's V8.
Native is `-Ctarget-cpu=native`; wasm has no SIMD turned on, like the build
of `web/`.  2,000 rounds, seed base 1000; `SmallRng` is another generator on
32 bits, so the two sides deal different cards and have different decision
counts.  The unit is milliseconds:

| | Decisions | Mean | p50 | p95 | p99 | Max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native, 10 cards or fewer | 1,679 | 3.75 | 1.60 | 15.6 | 29.5 | 64.0 |
| wasm, 10 cards or fewer | 1,628 | 6.62 | 3.03 | 25.4 | 47.8 | 84.4 |
| Native, more than 10 cards | 14,375 | 1.38 | 0.27 | 5.63 | 9.17 | 76.4 |
| wasm, more than 10 cards | 14,604 | 2.35 | 0.46 | 9.82 | 15.7 | 255 |

wasm is about 1.7 times slower than native.  Inside the gate the mean is
6.6 ms, within one frame (16.7 ms); the p95 is 25 ms and the slowest 84 ms,
more than a frame but all within 0.1 seconds.  With more than 10 cards it
goes by `HeuristicBot`'s grades, that is, the present hint, whose slowest
255 ms is a cost already there, not one `ExpectBot` brings in.  The timing
condition is passed.

### D: a wider gate

The 10-card gate lets through 9.7% of the decisions.  The build takes 41 ms
at 12 cards and 114 ms at 13, which an A/B can still run.  Each card count
is a separate A/B, its control the `main` of the time.

D's timing (2026-11-18, `expect_time 300 1000 14`, native, in milliseconds):
a 14-card gate lets through 39% of the decisions (976/2,477), at a mean of
76 ms inside the gate.

| Cards | Decisions | Mean | p50 | p95 | p99 | Max |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 11 | 136 | 20.2 | 11.8 | 69.9 | 121 | 195 |
| 12 | 144 | 35.4 | 22.1 | 123 | 174 | 186 |
| 13 | 148 | 96.6 | 53.9 | 325 | 555 | 647 |
| 14 | 277 | 186 | 108 | 650 | 980 | 1,410 |

14 cards take 1.4 seconds at the slowest and on average a fifth of the gated
Monte Carlo (0.9 seconds), which the desktop can afford; wasm is another 1.7
times slower, so the web page stays at 10 cards.

D's declaration (2026-11-18, written down before the run): the desktop's
gate opened from 10 cards to 14, one experiment.  Control `e1d0c7d`,
`RESHUFFLE=8`.  Seat 1 is the control's own `ExpectBot::new` on the
control's side (a 10-card gate, `BASELINE_EXPECT=1`) and the same bot with a
14-card gate on the candidate's (`EXPECT=1.5 GATE_CARDS=14`); the other
three seats are all `HeuristicBot`.  100,000 deals, seed base 1795214506:
the clock plus 200,000, to stay clear of the deals batches 3 and 4 used.

The deal count comes from a trial run: 400 deals (seed base 1, not counted)
took 1 minute with 31% parting and a standard error of 0.260, so 100,000
deals take about 3.5 hours with a standard error near 0.016.

How to read it: on a win, the desktop's opponents and hint open to 14 cards
(`gate` in `web/src/lib.rs` is already changed and tested, and is committed
only once the verdict is in); on insufficient data or a loss, the desktop
stays at 10 cards, and on a loss the deals that moved the most are traced
first.

D's result: 27,472 of the 100,000 deals part, and seat 1's win rate is
24.63% for the control and 24.98% for the candidate; candidate minus control
is +0.132 ± 0.014 points a deal, 95% confidence interval [+0.105, +0.159], a
win.  The desktop's opponents and hint open to 14 cards; the web page stays
at 10.  10 cards against `HeuristicBot` is +0.076, so the stretch from 11 to
14 cards gives more than the one of 10 cards or fewer.

### D2: 15 to 17 cards, with melds and live 搭子 left whole in the graph

Stopped, for speed, not for whether it wins.

Past 14 cards it is too dear unpruned (1.6 seconds for a hand of 17,
`research/dp-nodes.md`), so a second gate was tried: for decisions with more
than 14 cards and up to 17, the nodes in the graph try only the discards of
`Discards::Live`, that is, they break up neither the cards that both layouts
take as melds nor a 搭子 whose awaited cards are still live.  The root is
not pruned: every discard and every eat or pong of the move at hand is still
reckoned, so the bot may still break them up now, and only assumes that it
will not later.

Timing (2026-11-19, native, 300 rounds, seed base 1000, in milliseconds):
the two gates together let through 71% of the decisions (1,746/2,452), at a
mean of 196 ms inside the gates.

| Cards | Pruning | Decisions | Mean | p50 | p95 | p99 | Max |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 14 | None | 254 | 172 | 106 | 569 | 954 | 1,360 |
| 15 | `Live` | 229 | 110 | 59.9 | 346 | 859 | 1,030 |
| 16 | `Live` | 92 | 211 | 123 | 725 | 1,497 | 1,497 |
| 17 | `Live` | 451 | 513 | 282 | 1,733 | 3,445 | 6,539 |

At 17 cards 1% of the decisions take more than 3.4 seconds, and the slowest
6.5 seconds.  That is too slow on the desktop, so there is no need to know
whether it wins: the A/B against the 14-card gate was declared at 20,000
deals (a trial run of 400 deals had 43% parting and 6.8 CPU seconds a deal,
6 times D's), was stopped soon after it started, printed no number, and has
no verdict.

The code was not kept.  To reproduce it: when `ExpectBot::values` builds the
graph, a hand past the gate uses
`Prune { discards: Discards::Live, ..Prune::default() }` instead.

### E: the chance to win and the expected 胡

Holding the play A chose fixed, sweep once more with the payoff changed to 1
for a win, and the chance to win comes out; change it to the 胡, and the
expected 胡.  This changes no play and only gives the hint two more numbers,
but it needs batch 2's calibration first: showing a probability that is off
is worse than showing none.  It is also the first thing in the book that can
be drawn as a chart: how a hand's value changes with the cards left.

### F: the uses left for the ceiling

The two items of batch 4 of `hu-ceiling.md` not yet done, "do not discard
the card that drops the ceiling below the threshold" and `dead`, are both
special cases of the DP inside the gate: such a child is just D.  Once A is
a win, these two need doing only past the gate, each on its own.

### G: opponent inference

In the model the other seats appear only in p(c), h and e.  Once route 1
infers whether the cards I want are still there, it plugs in by replacing
the uniform p(c) with the inferred one; the other seats' progress plugs into
h.  The core need not change, and `Outlook` gains a few fields.

### H: Monte Carlo

Two ways to plug in, both still only ideas: the playout policy changes from
`HeuristicBot` to `ExpectBot`; or a playout stops once it reaches 10
concealed cards or fewer and takes the DP's value as its result.  The former
makes each playout tens of times dearer; the latter is cheap.

### I: labels

Route 4 needs a great many labels of how many points a position is worth.
Inside the gate the DP's values have no sampling noise and can label short
hands; a hand of 20 cards still needs Monte Carlo.

Added 2026-11-19.  The graph of a long hand is unaffordable online but can
be built offline (1.6 seconds at 17 cards, 5 seconds at 20 with the discards
pruned), so the labels of long hands can come from the DP too.  The design
that trains an eval of microseconds on them and expands only a layer or two
past the gate is in [`eval.md`](eval.md).

### Adding an application

Add a row to the table above and then a section that says four things: which
output of the core it uses, whether it changes play, the condition to start,
and how to measure it.  When the core has to give another kind of output,
first see whether it can be written as another payoff (`Payoff`); if so the
core need not change.

## Non-goals

- **Hands of 20 cards.**  7 × 10⁵ nodes and 22 seconds, measured.  Past the
  gate it is `HeuristicBot`.
- **Putting `live` or the turn order in the state.**  Not in the first
  version; only once calibration or an A/B shows it is the bottleneck.
- **A cache across decisions.**  See above.
- **A multiplayer game tree.**  This is the single-player expectation; the
  other seats are four constants.

## Open questions

- ~~Is the sweep really cheaper than the build?~~  Yes, more than ten times
  (batch 1).
- ~~Are four constants enough?  Which one does the calibration's bias fall
  on?~~  The predictions are high throughout, and h × 1.5 takes up most of
  it (batch 2).  Why h measures low, or which approximation it is making up
  for, is not yet known; h split by the fewest concealed cards another seat
  holds is the first thing to try.
- ~~Does `ExpectBot` use the measured h, or h × 1.5?~~  Both win, and they
  differ within the error; h × 1.5 is used (batch 3).
- ~~Are no sampling noise and optimal play from then on worth more than
  Monte Carlo's +0.070?~~  As large: +0.076 ± 0.008 against +0.070 ± 0.026,
  the intervals overlap, and neither can be called the stronger; the
  single-player assumption did not eat it up (batch 3).  The two head to
  head is the A/B of application B.
- At how many cards should the gate open?  (Application D)
- How far off is taking the other seats' discards as uniform?  This is the
  first problem for opponent inference.  (Application G)
