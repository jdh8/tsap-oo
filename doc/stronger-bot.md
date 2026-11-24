# A stronger bot: routes other than Monte Carlo

Dated 2026-11-16.  Control base: `10a046f`.

This document says where `HeuristicBot` gets stronger from here, which route
comes first, and what counts at each step.  The experiments' numbers are in
`CHANGELOG.md`, the measurements in `research/`, and what others have done in
`research/prior-art.md`; only the design and the current state are kept here.

## Goal and constraints

- **"Stronger" is defined** by the result of `ab/run.sh`: the candidate bot
  in one seat with `main` HEAD's `HeuristicBot` in the other three, against
  the same deal with four control bots, and the 95% confidence interval of
  the score difference a round lies above 0 (see the Iron rules in
  `CLAUDE.md`).
- **The web page must be able to use it.**  The web page's opponents and
  hint are both `HeuristicBot`.  `MonteCarloBot` at 128 samples compiled to
  wasm averages 0.83 seconds a move, too slow.  So every route here must
  keep its cost offline, or stay at the order of `HeuristicBot`.
- **The scoring decides what defence is worth.**  The winner collects its 胡
  from each of the other three seats, and the seat that deals in pays no
  more.  So the cost of dealing in is "the round ends now and I pay one
  share", about 12 points on average, plus whatever chance there still was
  of winning oneself.

## Current state

- `HeuristicBot` is one-step greedy: the hand's 胡, less the loose cards and
  the distance from the threshold, then how many cards not yet seen can fill
  its 搭子.  It counts the cards it has seen but does not guess what the
  other seats hold.
- `MonteCarloBot` at 128 samples against the current `HeuristicBot`: +0.148
  ± 0.276 points a deal, 95% CI [−0.392, +0.689], insufficient data.  What
  it does is "assume the unseen cards are spread evenly, then look a few
  steps further".  This suggests that computing more on the same information
  may leave little room.  At a table of two the same bot was +0.347 ± 0.175
  the first time and +0.050 ± 0.172 on the rerun, insufficient data
  (route 5).
- A `MonteCarloBot` that searches only with 10 concealed cards or fewer is
  +0.070 ± 0.026 points a deal, 95% CI [+0.019, +0.121], 40,000 deals, win
  (route 6).  It is the first search to beat `HeuristicBot` at a table of
  four, at a cost tens of times smaller than the ungated one's.
- The Peeker, which peeks at the opponents' hands, is +2.824 ± 0.192 points
  a deal at a table of two, win (route 1).  It is the same search; the only
  difference is whether the opponents' hands are real or guessed.
- Each past improvement to `HeuristicBot` was about +0.08 to +0.28 points a
  round.  An A/B of 200,000 deals has a standard error of about 0.01 to
  0.03, larger the more deals are played differently, so an improvement
  under 0.05 points needs more deals to be seen.
- 1 胡 in hand is worth about 1.4 points; the grade counts it as 32
  (`research/fitted-weights.md`).  The grade itself is an integer for
  ranking, with no unit.

## What prior research says

No AI paper was found on 十胡 or Four Color Cards; what can be borrowed comes
from mahjong and gin rummy (`research/prior-art.md`, which has the source of
each item).  What bears on this plan:

- **Mahjong AIs use their opponent models almost entirely to compute the
  loss from dealing in.**  In Japanese mahjong the seat that deals in pays
  in full, so it is worth it; in 十胡 it pays no more.  After Mizukami and
  Tsuruoka added an opponent model the deal-in rate fell and the win rate
  fell too, the same phenomenon as the defence experiments here.  This part
  does not carry over.
- **The champion of the gin rummy competition relied on "how useful is this
  card to the opponent".**  MARJJ grades a discard by the value of its own
  hand plus this term; with one group of them zeroed, the win rate drops to
  47% to 48%.  This supports "not feeding the next seat".  The same paper
  also found the parameters insensitive over a wide range, the same as
  route 2.
- **What opponent inference is worth differs widely between games, so
  measure the ceiling first.**  Letting the search peek at the opponents'
  cards directly: in Scrabble it adds 37 points a game while actual
  inference got only 5; in gin rummy the win rate goes from 26% to 85%; in
  Dou Dizhu there is almost no difference; in two-player mahjong a random
  opponent model is as good as a learnt one.
- **The lesson of `../gin-rummy-engine` is that the sample count matters
  more than the opponent model.**  In two-player gin rummy a Monte Carlo of
  the same architecture clearly beats the greedy bot, and catching up with
  the strongest opponent came from making the playout fast and adding
  samples; all three "understand the opponent" diagnoses were refuted by
  A/B, and one of them, dealing the opponent's hand more accurately, made it
  weaker.
- **Evaluation noise has ready-made solutions.**  Everyone pairs on the same
  deal (`ab/` already does).  The noise left after pairing is pressed down
  with rollouts in backgammon and with estimators that subtract the luck in
  poker, so two-player poker needs 5 to 10 times fewer hands.

## Routes

| | Route | Uses new information | Usable on the web page | Engineering |
| --- | --- | --- | --- | --- |
| 0 | Measure accurately | no bot change | not affected | small |
| 1 | Read the opponents | yes | yes | small, can be staged |
| 2 | Fit the grade's constants to data | no | yes | medium |
| 3 | Exact single-player expectation | no | depends on the depth | medium |
| 4 | Distil Monte Carlo into a fast model | no | yes | large |
| 5 | A table of two as a test bed | no bot change | not affected | small |
| 6 | Monte Carlo that searches only late | no | no, left to the native interface | small |

Route 0 comes first: five of seven A/Bs gave insufficient data, and what
blocks them is the standard error, not the ideas.  Then route 1, because
only it lets the bot use information it did not use before; routes 2, 3 and
4 compute the same information more accurately, and the Monte Carlo result
hints that the room there is limited.

**Current state**: the defensive part of route 1 (layers 1 and 2) was tried
in two experiments and route 2 in three; none won and the bot is unchanged.
Defence is worth little in 十胡; with the grade's constants replaced by
fitted values, no difference in strength shows either.  Converting the grade
to points was measured too: it works for hands, not for choices.  The rule
that a waiting hand does not eat or pong into fewer waiting cards was tried
once, insufficient data.  The table of two was probed once (route 5): the
engine plays it with one constant changed, and two changes known to work
still work at a table of two but are no easier to see; the positive
confidence interval `MonteCarloBot` first measured at a table of two did not
reproduce on the rerun.  The Peeker was measured: seeing the opponent's hand
at a table of two is worth 2.8 points a deal, so the ceiling of opponent
inference is high enough.
The first step of route 0 is done: reshuffling the pile 8 times from the
point of parting gives the same confidence for a quarter of the compute, and
`ab/` does so by default from now on.  "Not feeding the next seat" did not
win at a table of four and won narrowly at a table of two.
Route 6 was measured: `MonteCarloBot` searching only with 10 concealed cards
or fewer, at a table of four, +0.070 ± 0.026, win; searching only while
waiting and the union of the two were both insufficient data.
`HeuristicBot` and the web page are unchanged.

Suggested order of next steps:

1. **Do opponent inference, first at a table of two**: "are the cards I want
   still there" and the Monte Carlo deal.  The ceiling is +2.77 ± 0.26
   points a deal (Peeker minus Monte Carlo, `research/two-seats.md`), an
   order of magnitude above `HeuristicBot`'s past improvements.
2. **Measure the Peeker at a table of four.**  The sizes at a table of two
   do not carry over (route 5); the ceiling at a table of four decides
   whether inference is worth carrying back.
3. **Measure the other `HeuristicBot` ideas again with the reshuffled A/B**:
   the ones that gave insufficient data before (comparing danger only when
   the grades tie, blocking only the eats and pongs that drop the waiting
   cards by half or more) are now four times cheaper.

## Route 0: measure accurately (first step done)

### Where the noise is

`ab/` has the candidate and the control play the same deal.  On deals where
the two play alike the score difference is exactly 0; all the noise is in
the deals played differently.  Working back from each experiment's standard
error, the score difference on those deals has a standard deviation of 27 to
37 points, about the standard deviation of one seat's score in a round
(about 30).  That is, pairing cancels only the luck before the parting; once
the two play differently, what is drawn afterwards decides most of the score
difference.

This is what "doing the right thing is not always rewarded" looks like in
numbers: a decision worth 1 to 6 points on average, buried in luck with a
standard deviation of 30 points.

### Method: reshuffle the pile from the point of parting

When the two first make different decisions, record the position, reshuffle
the pile left K times, play both out from that position K times, and take
the mean score difference.

- **Unbiased.**  Every decision before the parting depends only on cards
  already turned up, so the order of the pile left is still uniformly random
  at that moment.  Reshuffling does not change the expectation; it only
  averages out the luck after the parting.  The four seats' concealed cards
  are part of the position and cannot be redealt.
- **The library needs no change.**  Cards are drawn from the tail of the
  pile, so the pile left is the first few cards of the original deck;
  shuffle those, redeal with `Round::from_pile`, and replay the recorded
  decisions up to the point of parting.

### Results (`research/reshuffle.md`)

Calibrated on the "do not eat into a meld already complete" run (`1b644f2`
against `6fc8f21`, known to be +0.283 ± 0.018, 200,000 deals):

- **Within one position the luck of the pile has a standard deviation of 18
  to 19 points; the true difference between positions is only 4.4 to 4.7
  points.**  The bulk of the noise is indeed after the parting.
- **Acceptance passed.**  With the candidate and the control identical, not
  one of 20,000 deals parts and the score difference is exactly 0; the
  confidence intervals at 8, 16 and 32 reshuffles all cover +0.283.
- **At K = 8 the same confidence takes about a fifth of the deals and a
  quarter of the compute.**  K = 16 and K = 32 take fewer deals still, for
  about the same compute or slightly more.

`ab/` defaults to K = 8 from now on (`RESHUFFLE`), and the measurement iron
rules in `CLAUDE.md` change with it.  Candidates like `MonteCarloBot` use
`RESHUFFLE=0` as before: every replay is another search.

### What can be added later

In order of the effect the literature reports, all after the step above:

- **Subtract the luck of the draw.**  For each card drawn, add "the expected
  grade before this card − the grade after" to the score; this term has zero
  expectation.  It works in two-player poker; in six-player poker it cut the
  variance by only a fifth.
- **Cut the pile by seat.**  Each seat draws only from its own stack, so when
  eats and pongs change the order, the cards the other seats draw do not
  change.  Chinese Official mahjong competitions do this, and give no
  numbers.
- **Test as the run goes.**  Stop when there is enough, so the average deal
  count falls; "the deal count is set before seeing the data" must become
  "the boundary is set before seeing the data".

## Route 1: read the opponents

### Public information

`Round::events` records every draw, discard, eat, pong and kong, all public.
With each seat's exposed melds and concealed card count, a seat can know:

- For each card put down, who was asked and who did not win.
- When each seat ate or ponged (its concealed cards change only then).
- How many concealed cards each seat has left and how many 胡 its exposed
  melds are worth.
- How many of each kind of card have not appeared.

The inferences these allow fall into four layers by reliability.

### Layer 1: safe cards (measured, not adopted by the bot)

Winning is compulsory, so when a card is put down and nobody wins, none of
the seats asked could win on it then.  Concealed cards change only on an
eat, pong or kong, so the conclusion holds until that seat's next eat, pong
or kong.  This is a proof from the rules, independent of how the opponents
play.

`View::safe(seat)` returns the cards a seat is proved unable to win on.  The
play tests in `tests/round.rs` check that no win is ever on a card proved
safe.

Measurement (`research/safe-cards.md`): the proof was never wrong, but it
covers only about 5% of the (candidate discard, opponent) pairs, because
every eat or pong by the opponent voids all of it.

### Layer 2: the opponents' progress (measured, not adopted by the bot)

The same measurement shows that the chance a card not proved safe is won on
by a seat varies about two hundredfold with that seat's concealed card
count: 0.02% while it still holds 19 to 20 cards, 3% to 5% at 8 or fewer.
The concealed card count is public.  This tells the bot which opponent is
worth defending against.

### Two experiments: the danger of a discard

Both against `0b5d48f`, 200,000 deals each, the deal count set before the
run.  The danger is, for each opponent not proved safe, the chance of being
won on looked up by its concealed card count, summed over the three seats.

| Method | seed base | Deals that differ | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| 12 × 32 × danger taken off the grade | 1794815549 | 5,328 | −0.037 ± 0.010 | [−0.057, −0.017] | loss |
| Danger compared only when the grades tie | 1794816883 | 2,513 | −0.002 ± 0.007 | [−0.015, +0.011] | insufficient data |

The first method assumes 1 point of score equals 32 of grade.  Tracing the
16,927 deals of the same batch whose first parting is a discard, the
outcomes of the candidate bot's seat are:

| Outcome | Control | Candidate |
| --- | ---: | ---: |
| Wins itself | 2,447 | 2,272 |
| Deals in | 3,460 | 3,057 |
| Another seat wins, not on its discard | 9,879 | 10,356 |
| Exhausted round or 相公 | 1,141 | 1,242 |

Deal-ins fell by 403, yet the losses in total rose by 74 and its own wins
fell by 175.  Of the 1,691 deals where the control dealt in and the
candidate did not, another seat won anyway in 1,394 (82%).

### What was learnt

- **Defence is worth little in 十胡.**  Every drawn card is turned up and
  offered to all four seats, so a waiting seat wins without your dealing in;
  and the seat that deals in pays no more.  Dodging a deal-in mostly just
  changes who deals in.
- **The hand's progress is dearer than thought.**  A hand slowed down to
  play safe cards loses more than dodging deal-ins gains.  Route 2 measured
  32 of grade at about 1.4 points; "1 point = 32" prices the grade too
  cheaply.
- **The version that needs no exchange rate has no visible effect.**
  Comparing danger only when the grades tie, the standard error over 200,000
  deals is already down to 0.007 and still no difference shows.
- **The late game may differ, but the sample is too small.**  The first
  method is +0.38 ± 0.39 points on the 704 deals that part only with 13
  cards or fewer left in the pile; the early and the middle game are
  negative.

### Layer 3: walls (deferred)

For an opponent to win on a card, its hand must hold cards that meld with
it: the same card (a pair wait or a triplet wait), or the other
two kinds of the same colour (making a run), or soldiers of other colours.
How many of these partner cards have not appeared is something the discarder
can count, and it grades each card's danger more finely.

This is still defence, bound by the same conclusion above, so it is not done
for now.

### Layer 4: inference from behaviour (later)

What an opponent did not eat, did not pong and discarded from its hand all
reveal what it lacks.  For example, a seat that discards a red 俥 from its
hand is mostly not waiting with the red 傌 and red 炮.  Such inference covers
far more cases than safe cards do, but what it learns is `HeuristicBot`'s
habits, which need not hold for people.  Whether to accept that limit must
be decided before this layer is built.

### Directions not yet tried

Added 2026-11-25: the plan that takes these up at a table of two, an oracle
for each channel before any inference, is in [`opponents.md`](opponents.md).
The Peeker's number below was measured before the hands of the empty seats
were dead, and is void.

Reading the opponents is not only for dodging deal-ins.  The following use
the same public information, to win faster oneself or to make the opponents
win slower:

- **Measure the ceiling first: a Monte Carlo that peeks at the hands.**  Let
  `MonteCarloBot`'s deal use the other seats' real hands (the pile still
  random), against `HeuristicBot` on the same A/B.  No opponent inference
  can do better than it.  If even peeking adds only a few tenths of a point,
  "are the cards I want still there" and "the Monte Carlo deal" below and
  layer 4 can all be skipped.  It also removes the search's fault of
  deciding separately in each deal, so it overestimates what inference can
  get; as an upper bound that is just right.

  This bot is called **Peeker**; the name says it cheats, lest it be seated
  one day as a champion.  What keeps it out is where it lives, not its name:

  - A `Strategy` gets only a `View`, and `View` does not give the other
    seats' hands, so the Peeker cannot be written as a legal `Strategy`
    without a back door in `View`.
  - The back door is not opened as public API.  `#[doc(hidden)]` and a
    non-default feature `peek` keep it shut; `web/` does not enable the
    feature, so it cannot be compiled in.
  - The Peeker itself lives in `ab/` (or the scratch copy for the table of
    two), not in the library.  It is a tool for measuring the ceiling, not a
    bot that may sit at the table.

  **The ceiling measured (table of two, measured).**  The Peeker at 128
  samples against `e40cca4`'s `HeuristicBot`, 3,200 deals, seed base
  1794847090, `RESHUFFLE=0`: +2.824 ± 0.192 points a deal, 95% CI [+2.449,
  +3.200], win; 1,616 deals differ, win rate 49.31% → 60.88%.  Under the
  same conditions the `MonteCarloBot` that does not peek is +0.050 ± 0.172
  (seed base 1794846583), and the difference of the two is +2.77 ± 0.26.
  Peeking adds more than a few tenths of a point, so the two items below
  and layer 4 are worth doing.  The table of four is not yet measured.
- **Not feeding the next seat (measured, not adopted by the bot).**  At a
  discard, take off `FEED` × the chance the next seat can eat it
  (`research/feed-next-seat.md`).  `FEED` 32 overturned the order in which
  lone cards are discarded, −0.172 ± 0.017 at a table of four, loss; `FEED`
  4 only chooses among cards of the same kind, insufficient data both times
  at a table of four (+0.015 ± 0.008, −0.007 ± 0.004) and +0.007 ± 0.003 at
  a table of two, win.  The next seat is the only opponent at a table of two
  and just one of three at a table of four, as expected; but the size
  cannot be measured at a table of four.
- **Are the cards I want still there.**  The bot now treats every unseen
  card as still to come.  The cards an opponent holds on to will in fact not
  come; what it ate, ponged and discarded can correct the value of each
  搭子.
- **The Monte Carlo deal.**  `View::determinize` currently deals uniformly,
  and deals impossible rounds (someone should have won long ago).  The proof
  of layer 1 can be used directly to reject those.  `../gin-rummy-engine`
  got weaker against the strongest opponent after making the hands dealt to
  the opponent more accurate; this item waits for a ceiling number first.

On the defensive side, there is more than one exchange rate (see "Converting
the grade to points" under route 2), the conclusion above is unchanged, and
these come later:

- **Folding.**  Give up progress for safety when the hand has no hope.
- **The discard after an eat or pong.**  After an eat or pong a card must be
  discarded, and that card's danger is not counted in the grade of the eat
  or pong.

## Route 2: fit the grade's constants to data (measured, not adopted by the bot)

`WEIGHT`, `COSTS`, `SCALE`, `OWN` and `WAIT` are hand-tuned.
`HeuristicBot::terms` splits the grade into seven terms (胡, loose cards,
distance from the threshold, copies of loose cards, 搭子, lone cards, waiting
cards), and the `fit` example fits them to self-play results by least
squares in two ways (`research/fitted-weights.md`):

- **Hands.**  Each term of the four seats' hands at every discard, against
  the round's final score, with control variables for the progress of the
  round.
- **Choices.**  Play every card the discarder could discard out to the end,
  and fit the differences in score to the differences in the terms.
  Everything shared within one decision cancels.

### Results

The two fits agree with each other and differ much from the hand-tuned
weights (in units of grade, 1 胡 in hand counting 32):

| Term | Hand-tuned | Fitted on hands | Fitted on choices |
| --- | ---: | ---: | ---: |
| Points a 胡 is worth | | 1.38 | 1.42 |
| Loose cards | −128 | −51 | −45 |
| Distance from the threshold (per half 胡) | −64 | −18 | −11 |
| 搭子 | 1 | 2.0 | 2.8 |
| Waiting cards | 8 | 26 | 24 |

Playing by them did not win, though.  All three against `10a046f`, 200,000
deals each, the deal count set before the run.

| Method | seed base | Deals that differ | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| All the weights fitted on hands | 1794819512 | 59,981 | −0.005 ± 0.034 | [−0.071, +0.060] | insufficient data |
| Only `WAIT` changed from 8 to 24 | 1794820392 | 2,221 | +0.002 ± 0.007 | [−0.012, +0.017] | insufficient data |
| The fit on close choices only: 搭子 ×8, `WAIT` 72 | 1794822233 | 35,378 | −0.070 ± 0.025 | [−0.120, −0.021] | loss |

### What was learnt

- **The exchange rate is about 1.4 points a 胡**, about 1.0 with 13 cards or
  fewer left in the pile.
- **The grade is insensitive to the weights.**  With the loose cards' weight
  cut to 40% and the waiting weight tripled, 30% of the deals are played
  differently and yet no difference in score shows.  The constants are not
  the bottleneck.
- **The bot's ranking is right on average.**  The other discards graded less
  than 16 behind score 0.46 points less on average when played out to the
  end; the further behind, the more they lose.
- **What the fit reads is the grade's curvature, not wrong weights.**
  Within 64 behind, every 32 is about 2 points; beyond 256 behind, every 32
  is under 1 point.  A linear model presses down the terms with large gaps
  and lifts those with small ones.
- **Weights fitted on one kind of decision do not carry to another.**  The
  third experiment's weights come from discards "with the same melds,
  differing only in which loose card is kept"; on that kind of decision
  there was no difference, and it lost on breaking melds and on eats and
  pongs.

### Converting the grade to points (measured, not adopted by the bot)

Chess engines convert their evaluation to pawns.  Converting the grade to
score in the same way gives (`research/grade-in-points.md`):

- **It works for hands.**  Score against grade is a monotonic curve, about
  0.5 points per 32 between −640 and 0 and about 1.4 points between 0 and
  640.
- **It must be in segments.**  A hand graded 0 is worth +8.2 points with 23
  cards or more in the pile, −2.0 points with 14 to 22, and −4.9 points with
  13 or fewer.
- **It does not work for choices.**  Predicting the cost of discarding
  another card from the curve, the choices within 64 behind actually lose
  more than 3 times the prediction.  Measured directly, 32 to 64 behind
  loses 4.1 points and 64 to 128 behind only 1.6 points, which is not even
  monotonic.
- **Within 8 behind is a tie** (+0.04 ± 0.10 points).

So no single exchange rate puts "the points defence saves" and "how far
behind in grade" on one scale: small gaps are about 3 points per 32, large
ones about 0.6 to 0.8 points.  When an experiment of route 1 needs points as
its unit, it must measure by playing the choices out to the end, not convert
from the grade.

### One experiment: a waiting hand does not eat or pong into fewer waiting cards

A hand at 8 胡: 3 胡 on the table, five generals (將帥) in hand, a pair of
red 炮, two loose cards, a yellow 兵 and a green 卒, already at 8 胡, waiting
on the red 兵 and the white 卒, 8 cards at most.  The next seat draws a red
炮 and the bot chooses to pong: 1 more 胡 and one loose card fewer (128 saved
in grade), but the wait is down to a pair wait on 3 cards at most, and
the win goes from 11 胡 to 9 胡.  The grade fines the two waiting soldiers as
ordinary loose cards, and adds only 8 for each live card.

The candidate's rule: when passing already leaves the hand waiting, rank the
choices whose eat or pong leaves fewer waiting cards behind passing.
Against `fc6ca57`, 200,000 deals, the deal count set before the run.

| Method | seed base | Deals that differ | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| Eats and pongs that leave fewer waiting cards rank behind passing | 1794828275 | 3,607 | +0.007 ± 0.011 | [−0.014, +0.029] | insufficient data |

The win rates are 24.23% against 24.19%.  The data are not enough and the
bot is unchanged.  The rule blocks such eats and pongs whatever their size:
one that goes from 4 waiting cards to 3 for 1 胡 is blocked too.  The version
that blocks only a drop of half or more in the waiting cards is untried, and
resolving 0.007 points needs about two million deals.

### Four experiments: how many melds a hand has room for

Concealed cards only decrease, so the cards outside the melds plus the
winning card split exactly into three-card melds and pairs
(`research/room-for-melds.md`).  Eating a pair uses up two cards' room for 0
胡; the original grade does not see this cost, and even counts it as
progress, two loose cards fewer.  All against `b894777`, 100,000 deals each,
the deal count set before the run.

| Method | seed base | Deals that part | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| Hope from 搭子 and from pongs of pairs, capped at the melds there is room for | 1794933710 | 28,275 | −0.290 ± 0.014 | [−0.317, −0.263] | loss |
| The same, with the cards left over held to pairs | 1794934225 | 33,313 | −0.216 ± 0.014 | [−0.244, −0.189] | loss |
| Cap on the 搭子 only, no pongs of pairs | 1794936088 | 6,093 | +0.005 ± 0.004 | [−0.003, +0.014] | insufficient data |
| No pair eaten below the threshold | 1794934919 | 52,775 | −1.497 ± 0.020 | [−1.536, −1.458] | loss |

Eating a pair is well worth it on average; never eating one loses 1.5 points
a deal.  With the hope of ponging pairs counted in, the bot instead breaks
melds that score 胡 to eat pairs.  The cap-only version changes play in only
6% of the deals; the pairs it no longer eats are +0.26 ± 0.12 points each, a
figure from grouping after the fact.

### Not yet tried

- **The 胡 ceiling.**  In the cap-only version above, fewer pongs and more
  pair eats lose and fewer pair eats gain, and the bound estimated after the
  fact is about +0.006 points a deal.  The "cap used only to refuse pair
  eats" that was to be tried is replaced by an exact arithmetic: work out
  the most 胡 a hand can reach without luck, and block only the pair eats
  that would drop it below the threshold.  The design and the progress by
  batch are in [`hu-ceiling.md`](hu-ceiling.md).
- **Average in units of points.**  `HeuristicBot::draw` averages the grade
  over the cards that may be drawn; the curve bends, so converting to points
  before averaging would change the eat and pong decisions.  The curve is
  inaccurate on choices, and there is no reason to expect it to win.
- **Weights that vary with the state of the round.**  The fit segmented by
  pile shows loose cards getting cheaper and waiting dearer the later it is.
  Swapping the whole set of weights shows no difference, so a segmented
  version needs two million deals or more to be resolved.
- **Non-linear or new features.**  That is already the territory of routes 3
  and 4.

## Route 3: exact single-player expectation

Replace "胡 minus the cost of loose cards" with a dynamic programme of "the
chance of winning within n draws left, and the expected 胡".  Eats already
have one layer of this expectation (`HeuristicBot::draw`); discards are
still graded statically.  The result is deterministic, with no sampling
noise, but it is still lookahead under a uniform belief, bound by the
ceiling Monte Carlo measured.

The condition to start was "route 2 shows a large residual that static
features cannot explain", but route 2 cannot measure that: it only shows
that the linear combination of these seven terms is used up.

Addendum of 2026-11-18.  The ceiling's probe ([`hu-ceiling.md`](hu-ceiling.md)
batch 2) is the first supporting data: the CIs of groups A and B lie above
0, which means the static grade misprices "whether it can get there", though
by only +0.014 and +0.025 points a deal.  What follows is inference not yet
measured.

- **Why the graph is small.**  A drawn card nobody wants goes to the
  discards, so the concealed cards change only on an eat or pong, one to
  four fewer each time; the state is (concealed cards, exposed 胡, chances
  left), a DAG some ten deep.  V(hand, t) = Σ over the live cards of the
  chance it appears × max(pass, the best eat or pong and discard →
  V(hand′, t+1)) + useless cards × V(hand, t+1), each turn round the table
  multiplied by the chance that no other seat ends the round.
- **The estimate of the cost.**  If a hand of 20 has 10³ to 10⁴ reachable
  sub-hands, times about 60 chances and at most 28 kinds of card, that is
  10⁶ to 10⁷ table lookups, the order of milliseconds; Monte Carlo takes 0.9
  to 3.1 seconds a decision.  The number of sub-hands is a guess and must be
  measured first.
- **Not only cheaper.**  128 samples resolve only gaps of a few points (a
  seat's score has a standard deviation near 30), and after that the playout
  plays greedily; dynamic programming has no sampling noise and plays
  optimally afterwards.  So the gain Monte Carlo measured is not its upper
  bound.  It is also fast enough for the A/B to use `RESHUFFLE=8`.
- **Where it is harder than it looks.**  `live` is not in the state, and can
  only be fixed at the root and recomputed at each decision; not every card
  comes my way (an eat takes only my own draw and the previous seat's card,
  and the other seats' pongs go first); the other seats ending the round
  needs a chance per turn (the table in `research/safe-cards.md`); the nine
  blocks cannot be worked wholly apart because of the discard and the 10-胡
  threshold; and the payoff must be in points, not 胡.
- **Still expected to be small.**  This is still the same information under
  a uniform belief; the Monte Carlo that searches only late is +0.070 points
  a deal.

The first step is an example like `census` that counts the reachable
sub-hands and edges from the decision points of self-play.  It does not
change play and needs no A/B.

Counted on 2026-11-18 (`examples/nodes.rs`, `research/dp-nodes.md`): the
state is (concealed cards, exposed 胡), pruned where the ceiling cannot reach
the threshold.  A freshly dealt hand of 20 averages 7.0 × 10⁵ nodes and 5.8 ×
10⁷ edges, 22 seconds a census; pruning only what cannot get there at
`generals = 0` either, 80% are left.  The estimate above was two to three
orders of magnitude low; neither the web page nor the A/B can afford it, and
it does not take over batch 4 of `hu-ceiling.md`.  A hand of 12 cards is
about 2 × 10³ nodes and 41 milliseconds, one of 10 about 6 × 10² and 11
milliseconds: only the stretch past a gate like route 6's is on the order of
milliseconds.  If it is built, it is a gated dynamic programme, at the same
gate as the gated Monte Carlo, whose +0.070 is the control to take first.
The design, the progress by batch and the applications hung on it are in
[`expectation.md`](expectation.md).

## Route 4: distil Monte Carlo into a fast model

Label decisions offline with a Monte Carlo of many samples, train a small
model to imitate it, then iterate with the new model as the playout policy.
This directly solves Monte Carlo being too slow on the web page, but it
needs a data pipeline and a pure-Rust model, and its upper bound is what
Monte Carlo can find.

The condition to start: a Monte Carlo with an opponent model clearly beats
`HeuristicBot` on the A/B.

Addendum of 2026-11-19.  The teacher can be swapped for route 3's dynamic
programme: its values have no sampling noise, and the gated version has
already beaten `HeuristicBot` and the gated Monte Carlo.  The model is
lookup tables of the nine blocks' patterns, used as the leaf nodes of a
shallow graph.  The design and the progress by batch are in
[`eval.md`](eval.md).

## Route 5: a table of two as a test bed (probed)

### Rationale

At a table of four a mistake is not always punished; the other three seats
take the consequences away: in the defence experiments, 82% of the 1,691
deals where the control dealt in and the candidate did not were won by
another seat anyway.  In two-player zero-sum every point comes from the
opponent, so right decisions may be rewarded more often, and it is easier
to see what a good decision looks like.  The idea is to do Monte Carlo and
opponent inference at a table of two first, and extend to three and four
players once there are conclusions.

### What a table of two is

- **The rules do not change.**  20 cards each, 21 for the dealer, 6 dead
  cards, 10 胡.  Every source found says two or three can play, and none
  writes out what to change with fewer players; Board Game Arena's ranked
  default is two players, with rules that do not vary with the number
  (`research/prior-art.md`, section C).  (Added 2026-11-21: the dead cards
  later became 6 plus the 20 of each empty seat, so every table draws 25
  cards; `variants.md`, "The dead cards of a small table".  The numbers in
  this section were measured before the change.)
- **The engine changes one constant.**  With `Seat::COUNT` set to 2 (or 3)
  it deals, plays out and settles; only the tests and a few examples
  hard-code four seats.
- **The structure of the game changes.**  The only opponent is both the
  previous seat and the next seat, and can eat every card I discard or
  draw; every round it wins is paid for by me; the pile is 71 cards, and the
  round almost never exhausts (1 round in 20,000, against 599 at a table of
  four).

### Results of the probe

Two changes known to work at a table of four were each measured again at a
table of two, 200,000 deals each, the deal count set before the run
(`research/two-seats.md`).  A is "keep a waiting hand", B is "do not eat
into a meld already complete".

| | A, four players | A, two players | B, four players | B, two players |
| --- | ---: | ---: | ---: | ---: |
| seed base | 1794772616 | 1794830208 | 1794765506 | 1794830528 |
| Deals that differ | 3,551 | 3,901 | 16,804 | 21,854 |
| Score difference (points a deal) | +0.111 ± 0.010 | +0.030 ± 0.005 | +0.283 ± 0.018 | +0.222 ± 0.011 |
| z | 10.7 | 6.3 | 16.1 | 19.3 |
| Mean ÷ standard deviation, on the deals that differ | 0.18 | 0.10 | 0.12 | 0.13 |
| Compute for the same confidence | 1 | 1.8 | 1 | 0.4 |

`MonteCarloBot` at 128 samples against `HeuristicBot` was also measured
again at a table of two (`3aa5b0c`, 3,200 deals, the same as the run at a
table of four, the deal count set before the run):

| | Four players | Two players |
| --- | ---: | ---: |
| seed base | 1794777431 | 1794830779 |
| Deals that differ | 1,082 | 1,251 |
| Score difference (points a deal) | +0.148 ± 0.276 | +0.347 ± 0.175 |
| 95% CI | [−0.392, +0.689] | [+0.005, +0.689] |
| Win rate, control → candidate | 24.47% → 24.97% | 49.44% → 50.41% |
| Verdict | insufficient data | win |

By the iron rules this is a win, but the lower bound hugs 0 (z about 2.0).
These 3,200 deals at a table of two took 16 CPU hours (71 minutes on 16
threads).

A rerun on a fresh seed base did not reproduce it (`e40cca4`, 3,200 deals,
seed base 1794846583, 1,202 deals differ): +0.050 ± 0.172, 95% CI [−0.287,
+0.388], win rate 51.47% → 51.47%, insufficient data.  The two weighted by
inverse variance are +0.196 ± 0.123, and the interval still straddles 0.

### What the table of two taught

- **A round at a table of two is 40% cheaper**, and both changes still win
  at a table of two, in the same direction.
- **It is no easier to see.**  One change needs 1.8 times the compute at a
  table of two, the other only 0.4 times.  On the deals played differently,
  the ratio of signal to noise is 0.10 to 0.18 at both tables.
- **"Doing the right thing is not rewarded" has two components.**  One is
  luck: what is drawn after the decision.  It is as large at both tables,
  and it is what route 0 deals with.  The other is structure: at a table of
  four the same decision really is worth less, because the consequences are
  shared out among the other three seats.  This part is not noise but the
  game itself; a table of two magnifies it, yet what is measured is a fact
  about the table of two.
- **The sizes do not carry over.**  "Keep a waiting hand" is worth 13% of
  one win or loss at a table of four and only 6% at a table of two.
- **Looking a few steps further cannot be measured at a table of two
  either.**  On the same 3,200 deals, z is 0.5 at a table of four, 2.0 the
  first time at a table of two and 0.3 on the rerun.  In
  `../gin-rummy-engine` Monte Carlo clearly beats the greedy bot; here it
  does not; 3,200 deals have room for a true value around +0.3 points, and
  for 0 as well.
- **Seeing the opponent's hand is worth 2.8 points a deal.**  The same
  search with the real hands (Peeker, route 1) is +2.824 ± 0.192.  What is
  valuable at a table of two is information, not more computing.
- **The Monte Carlo experiments are stuck on compute, not on the deal
  count.**  Each round is 18 CPU seconds.  Route 0 cannot help: every replay
  is another Monte Carlo.  What is needed here is a faster playout, which is
  exactly how `../gin-rummy-engine` caught up with the strongest opponent.

### What the literature says

- **What carries over is the method, not the play.**  From Libratus to
  Pluribus, and from two-player mahjong's ACH to the four-player LuckyJ (the
  latter has no paper, only press accounts), what carried over was the
  algorithm; in no case was play learnt at the version with fewer players
  used at the version with more.  An example the other way: Poki, a poker
  program written for tables of many players, is easily beaten at a table of
  two.
- **Others choose two players for the guarantee of an equilibrium**, not
  because it is easier to learn.  That guarantee does not help an A/B
  against a fixed `HeuristicBot`.
- **Evaluating two-player mahjong is not cheap.**  Playing 200,000 games,
  each with the seats swapped, the 95% confidence interval is still ±0.13
  fan, while the effects to measure are only 0.14 to 0.22 fan.

### Recommendation

Keep the table of two as a second test bed, not as a shortcut to the table
of four.

- **How.**  `ab/run.sh` gets a switch for the seat count, changing that
  constant in each of the two versions as it exports them; the library is
  untouched.  Until the table of two becomes a proper way to play (one
  against one on the web page, say), the seat count need not be made a
  parameter, and no tests need be added for it.
- **When to use it.**  When an idea is about "interacting with the
  opponents" (feeding, defence, reading the opponents) or "looking a few
  steps further", measure once at each table; only with both in view is it
  known whether what the table of two taught carries over.  An idea that
  changes only the efficiency of one's own hand need only be measured at a
  table of four.
- **Research on opponent inference is done at a table of two first.**  The
  Peeker has a signal of over ten standard errors there in 3,200 deals.
  Improvements to the deal are measured at a table of two first, then
  verified at a table of four if they win.  The Monte Carlo that deals
  uniformly need not be run again: fixing its size would take around 40,000
  deals.
- **The condition for extending to three players.**  A change that clearly
  wins at a table of two and has the same sign at a table of four.

## Route 6: Monte Carlo that searches only late (measured)

### Rationale

The Monte Carlo experiments are stuck on compute (route 5): ungated, a round
takes 25 CPU seconds, and 3,200 deals cannot resolve +0.15 points.  Nearly
all the time goes on long hands, where the playout is long and each step is
dear too.  Searching only late, the same time plays more than ten times the
deals.

### Method

`MonteCarloBot::gated(cards, shanten)`: it searches only when the concealed
cards are no more than `cards`, or the shanten
(`HeuristicBot::shanten`, how many eats or pongs short of waiting) is no
more than `shanten`, and plays greedily otherwise.  The shanten is
found by search: a formula that counts melds and 搭子 cannot see the 10 胡 a
win needs.  It is not a term of the grade, and `HeuristicBot`'s play is
unchanged.

### Results (`research/gated-monte-carlo.md`)

A table of four, 128 samples, `RESHUFFLE=0`, control `4596efe`, 40,000 deals
each, the deal count set before the run.

| Gate | seed base | Deals that differ | Score difference (points a deal) | 95% CI | Verdict |
| --- | ---: | ---: | ---: | --- | --- |
| 10 concealed cards or fewer | 1794910000 | 1,518 | +0.070 ± 0.026 | [+0.019, +0.121] | win |
| Waiting | 1794950000 | 1,161 | +0.048 ± 0.030 | [−0.011, +0.108] | insufficient data |
| 10 cards or fewer, or waiting | 1794990000 | 2,302 | +0.061 ± 0.035 | [−0.009, +0.130] | insufficient data |

### What was learnt

- **Late search can be measured.**  The 10-card gate searches only 9.7% of
  the decisions and uses 2.7% of the search time; 40,000 deals take only 44
  minutes.
- **The gates cannot be told apart.**  The three point estimates lie within
  one another's error; the waiting gate is no better than the one that
  counts cards, and the union is no larger.
- **The effect is still small.**  +0.07 points a deal is the lower edge of
  `HeuristicBot`'s past improvements.  This agrees with the Peeker's
  conclusion: computing more on the same information leaves little room.
- **It is the platform for opponent inference.**  What a more accurate deal
  is worth can be measured inside this gate, at under an hour an A/B.

### Next

- The 10-card gate has z about 2.7, and three gates were read in the same
  batch; rerun it once on a fresh seed base.
- The native interface (which `MonteCarloBot` was kept for all along) uses
  the 10-card gate for its opponents and hint.
- The web page stays with `HeuristicBot`: a decision at 10 cards or fewer
  takes 0.9 seconds even natively.

## What is not done

- **Methods that find an equilibrium, such as CFR.**  Four players, not
  two-sided zero-sum, and the information sets are too large.  A table of
  two has an equilibrium to find in theory, but the teams that used it in
  the gin rummy competition used it only for small independent decisions
  such as the knock, the discard's information sets being too many for it;
  in 十胡 winning is compulsory, and there is no matching small decision.
- **Putting Monte Carlo in the web page.**  It was tried and is too slow;
  even gated, a decision takes 0.9 seconds natively (route 6).

## Known limits

The A/B's opponent is always `HeuristicBot`.  Layers 1 and 3 use only the
rules and the cards seen, and are not affected; layer 2's probability table,
layer 4, route 2 and route 4 all learn how to beat `HeuristicBot`, and
strength against people cannot be measured.

`../gin-rummy-engine` measures every change against three opponents: one
target and two "must not regress" gatekeepers.  The cheapest gatekeeper here
is an old version of `HeuristicBot`: `ab/run.sh` can already export any
version as the control.  This must be in place before layer 4 or route 4 is
started.

## Open questions

- Does the 10-card gate stand up on a rerun?  What about other card counts?

- Eats and pongs while waiting: would the version that blocks only a large
  drop in the waiting cards win?
- Would blocking the pair eats that drop the ceiling below the threshold
  win?  (`hu-ceiling.md`)
- Should the weights vary with the state of the round?  It needs an A/B of
  two million deals or more.
- Is late defence positive?  704 deals are not enough; it needs an
  experiment that changes only the late game.
- Is "not feeding the next seat" positive at a table of four?  Both runs of
  FEED 4 gave insufficient data; it needs a million deals or more.
- Is layer 4 worth doing if it works only against `HeuristicBot`?
- By how much does the Monte Carlo that peeks at the hands win at a table of
  four?  At a table of two it is +2.824 ± 0.192.
- How much of the Peeker's 2.8 points can inferring the opponents' hands
  from the public cards get?
- Are decisions that interact with the opponent worth more at a table of
  two?  If so, does the same change have the same sign at a table of four?
- Should the table of two become a proper way to play?  That needs tests,
  and a bot tuned for the table of two.
