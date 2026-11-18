# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `HeuristicBot`'s `short` term, and so the web hint, is at least the
  half-胡 `ceiling(generals = 1)` falls short of the threshold
  (`doc/hu-ceiling.md`, batch 4): the 搭子's hope can no longer promise
  more than any layout of the hand reaches.  `ab/run.sh 200000` against
  `7ad669b` (seed base 1794954270, `RESHUFFLE=8`), 555 deals part:
  +0.002 ± 0.000 points a deal, 95% CI [+0.001, +0.003], a win, though
  the smallest measured so far.  Two forms were measured on the way and
  not kept: replacing `short` outright is −0.380 ± 0.064 on a 5,000-deal
  smoke test (the term's pull toward the threshold goes), and the same
  floor with `generals = 0` is −0.036 ± 0.003 (200,000 deals, seed base
  1794953471), the whole loss being pongs the control takes and the
  candidate passes.
- `HeuristicBot`, and so the web hint, no longer eats a pair that drops
  `ceiling(generals = 0)` below the threshold (`doc/hu-ceiling.md`,
  batch 3): groups 甲 and 乙 of batch 2.  A pair eat whose ceiling still
  reaches the threshold stands.  `ab/run.sh 800000` against `5b62dc6`
  (seed base 1794947206, `RESHUFFLE=8`), 59,885 deals part, all of them
  the control eating a pair: +0.053 ± 0.002 points a deal, 95% CI
  [+0.049, +0.056], a win.  乙 gives +0.0353 [+0.0323, +0.0383], 甲
  +0.0173 [+0.0162, +0.0184].

### Fixed

- The web test `waits_foretell_the_win` checked stale waits when the round
  ended on the draw that the human's discard brought on.

### Added

- The pair rule in `draw`, measured and not shipped (`doc/hu-ceiling.md`,
  batch 4): `HeuristicBot::draw` leaving out the pair eats that
  `claims` leaves out since batch 3.  `ab/run.sh 400000` against
  `39d37b2` (seed base 1794956595, `RESHUFFLE=8`), 3,072 deals part:
  −0.002 ± 0.000 a deal, 95% CI [−0.003, −0.001], a loss.  Every split is
  the control passing and the candidate claiming: the next seat's pass is
  graded by the draw it foresees, which is worth less without those eats,
  so claims that lost to it before now win.  A trace of the first 100,000
  deals has the loss spread over pair eats (−0.88 a split), 車馬包 and
  將士象 eats, and pongs.  Kept as `research/hu-ceiling-draw-pairs.patch`.

- `examples/nodes.rs`, a census of the single-player DP that route 3 of
  `doc/stronger-bot.md` sketched: from every settled hand of `HeuristicBot`
  self-play, the (hand, exposed 胡) states a chain of claims and discards
  reaches while `ceiling` still meets the threshold, and the edges between
  them.  A fresh hand of twenty has about 7 × 10⁵ states and 6 × 10⁷
  edges and takes 22 s to walk, two to three orders above the sketch's
  guess; only hands of twelve cards or fewer are in the tens of
  milliseconds (`research/dp-nodes.md`).  Batch 4 of `doc/hu-ceiling.md`
  stays four separate candidates.  No bot changes.

- `ceiling`, the most 胡 a hand can still win with if every card it wants
  came and at most a given number of generals from outside; `None` when no
  layout of 21 cards exists (`doc/hu-ceiling.md`, batch 1).  It lays the
  hand out block by block — each colour's 將士象 and 車馬包, and the
  soldiers — every meld either dealt or finished by one outside card, and
  ignores who could supply the cards and in what order, so it is an upper
  bound.  Guarded by hand-worked cases and by `tests/round.rs`: at every
  decision of the eventual winner, the ceiling is at least the 胡 won.
  No bot changes: `ab/run.sh 20000` against `1aab13e` (seed base
  1795100000, `RESHUFFLE=8`) read 0 deals that part.

- The 胡 ceiling's probe, measured and not shipped (`doc/hu-ceiling.md`,
  batch 2): `HeuristicBot::claims` dropping a pair eat that lowers
  `ceiling(generals = 0)`.  `ab/run.sh 800000` against `168e1e4` (seed base
  1794940763, `RESHUFFLE=8`), 357,189 deals part: −2.173 ± 0.008 a deal,
  95% CI [−2.189, −2.156], a loss.  By the groups declared beforehand, 丙
  (ceiling still reaches the threshold) −2.213 [−2.229, −2.196]; 乙 +0.0254
  [+0.0228, +0.0279]; 甲 +0.0143 [+0.0133, +0.0154].  Since 甲 and 乙 lie
  above 0, batch 3 blocks only those.  The probe and its trace are kept as
  `research/hu-ceiling-probe.patch` and `research/hu-ceiling-trace.patch`.

- `research/seven-slots.md` and the `census` example: where a winning
  hand's 胡 come from.  A winning hand is 21 cards, seven slots of three at
  1 胡 each, so 胡 = 7 + 2k + 將士象 + 2 × (concealed triplets, soldiers
  and kongs) + the bonus of four-card melds, k = (singles − pairs) / 3.
  The identity held on all 19404 wins of 20000 rounds of four
  `HeuristicBot`s (seed 1795000000); k is 0 on 56.4% of them and +1 on
  35.2%.  It is the arithmetic `doc/hu-ceiling.md` builds on.  No bot
  changes.

- `tauri/`, a desktop client: `web/`'s page in a Tauri window, the engine as
  native code on its own thread (`cd tauri && cargo run --release`).  Its
  three opponents search: `MonteCarloBot`s of 128 samples gated to 10
  concealed cards or fewer, the gate that won its A/B below, about a second
  a searched decision.  Neither number is a setting, as no other gate has a
  verdict.  The hint is still `HeuristicBot`'s.  `web/app.js` reaches the
  engine through one object, the wasm in a browser and Tauri commands in the
  window; `WebGame::searching` seats the searching opponents.  The browser
  page deals and plays as before.

- `HeuristicBot::shanten`, the 向聽數: how many claims a hand is from
  waiting once the seat has answered, searched up to a limit, since a count
  of melds and 搭子 cannot see the ten 胡 a win needs.  A hand that eating
  or ponging the offer would leave waiting counts as waiting;
  `View::claim_options` is public for that.  It is not a term of the grade,
  and `MonteCarloBot::gated` uses it: a `MonteCarloBot` that searches only
  with so many concealed cards or fewer, or within so many claims of
  waiting, and plays the greedy choice elsewhere.  `ab/` reads the gate
  from `GATE_CARDS` and `GATE_SHANTEN`.  Ungated, a 128-sample decision
  takes 3.1 s and a round 25 CPU-s; decisions at 10 cards or fewer are 9.7%
  of them and 2.7% of the compute (`gate` example, 256 rounds, seed base
  1794901984).  `HeuristicBot` plays as before
  (`ab/run.sh 20000`, control 4596efe, seed base 1794866617): 0 of 20000
  deals part.

  Three gates against `HeuristicBot` (`research/gated-monte-carlo.md`):
  four seats, 128 samples, `RESHUFFLE=0`, control 4596efe, 40000 deals
  each, declared before the data.  Searching only with 10 concealed cards
  or fewer (`GATE_CARDS=10`, seed base 1794910000): +0.070 ± 0.026 points
  a deal, 95% CI [+0.019, +0.121], 1518 deals part, win.  Only while
  waiting (`GATE_CARDS=0 GATE_SHANTEN=0`, seed base 1794950000): +0.048 ±
  0.030, 95% CI [−0.011, +0.108], 1161 deals part, inconclusive.  Either
  (`GATE_CARDS=10 GATE_SHANTEN=0`, seed base 1794990000): +0.061 ± 0.035,
  95% CI [−0.009, +0.130], 2302 deals part, inconclusive.  The three lie
  within one another's error, so the data do not rank the gates.  No bot's
  default changes: `MonteCarloBot::new` is still ungated and the web page
  still plays `HeuristicBot`.  The `gate` example times where an ungated
  search spends its time.

- The web page shows your waits (聽牌): while you are one card from
  winning, a row under the hand lists every card that wins, each with the
  胡 it wins at and the copies you have not seen.  While you discard, each
  card whose discard leaves you waiting is marked 聽, and pointing at it
  shows those waits.  `Round::win` is public for it and takes the hand to
  try, so a discard can be tried before it is made.

- `ab/` averages out the luck after the two bots part: at the candidate's
  first choice the control would not make, it reshuffles the pile left 8
  times and plays both choices out on each (`RESHUFFLE`, 0 for the plain
  paired estimate).  The pile's order is still uniform there, so the
  estimate stays unbiased; no library change is needed, as the deal is
  rebuilt with `Round::from_pile` and replayed to the split.  Calibrated on
  1b644f2 against 6fc8f21 (`research/reshuffle.md`): within one split the
  shuffles spread by 18 to 19 points, the splits' true means by only 4.4 to
  4.7.  At K = 8 (40000 deals, seed base 1794846881) +0.289 ± 0.017, 95% CI
  [+0.256, +0.322], against +0.283 ± 0.018 from 200000 plain deals: a fifth
  of the deals, a quarter of the compute.  The same tree against itself
  (`ab/run.sh 20000`, control e40cca4, seed base 1794846981): 0 of 20000
  deals part.  The iron rules in `CLAUDE.md` now name this estimate.
- `research/feed-next-seat.md`: not feeding the next seat, by taking off
  each discard's grade `FEED` times the chance the next seat can eat it;
  the bot is unchanged.  All against e40cca4 with 8 reshuffles.  `FEED` 32
  overrides the order of loose cards and loses at four seats (100000 deals,
  seed base 1794847173): −0.172 ± 0.017, 95% CI [−0.205, −0.140].  `FEED`
  4, which only picks among like cards, is inconclusive at four seats twice
  (100000 deals, seed base 1794848292: +0.015 ± 0.008, [−0.002, +0.031];
  400000 deals, seed base 1794849203: −0.007 ± 0.004, [−0.015, +0.001]) and
  wins at two (200000 deals, seed base 1794848696: +0.007 ± 0.003, [+0.002,
  +0.012]).  The patch is kept beside it to replay.
- The Peeker, in `ab/src/peeker.rs`: `MonteCarloBot` with every hand in
  sight, sampling only the pile, through the hidden `View::peek` behind the
  new non-default `peek` feature.  `SAMPLES=n PEEK=1` seats it.  It cheats,
  so it stays out of the library; it measures the most that reading the
  other seats could be worth.
- `View::safe`: the cards a seat is proven not to win on.  Winning is
  compulsory and a hand changes only when its seat claims, so a card that
  came up and ended nothing stays safe against every seat it was offered to
  until that seat's next eat, pong or kong.  A playout test holds every win
  to it.
- The `safe` example and `research/safe-cards.md`: over 20000 rounds (seed
  base 20261117) the proof never fails but covers only about 5% of the cards
  a seat could discard, taken against one other seat; what sets a card's
  danger is how many concealed cards that seat has left, from 0.02% at
  twenty to between 3% and 5% at eight or fewer.
- `doc/stronger-bot.md`: the ways to a stronger bot other than Monte Carlo,
  and where the first two of them, reading the other seats and fitting the
  grade's weights to data, stand.  It now opens with what the survey below
  found and adds two ways that leave the bot alone: measuring better, by
  replaying each deal from where the two bots first differ over reshuffled
  piles, and a table of two seats to try ideas at.  The bot that is to
  measure the ceiling by peeking at the other hands is named Peeker and kept
  out of the library, behind a `peek` feature the web does not enable.
- `research/prior-art.md`: what mahjong and rummy bots do that could carry
  over.  No paper on four-colour cards was found.  Mahjong bots model the
  other seats to price dealing in, which does not pay here; the gin rummy
  champion grades a discard by how useful it is to the opponent; and paired
  deals leave the noise that follows the first differing move, which
  backgammon rollouts and poker's luck-adjusted scores remove.
- `research/two-seats.md`: the same rules at two seats, by changing
  `Seat::COUNT` in a scratch copy; the library and the bots are unchanged.
  Two changes that won at four seats win at two, each on 200000 paired
  deals, but no more clearly.  Keeping a waiting hand (3526146 against
  a74b951, seed base 1794830208, 3901 differ): +0.030 ± 0.005 points a
  deal, 95% CI [+0.021, +0.040], for 1.8 times the compute of the four-seat
  verdict.  Not eating into a whole meld (1b644f2 against 6fc8f21, seed
  base 1794830528, 21854 differ): +0.222 ± 0.011, 95% CI [+0.199, +0.244],
  for 0.4 times.  `MonteCarloBot` at 128 samples against `HeuristicBot`,
  both at 3aa5b0c (3200 deals, seed base 1794830779, 1251 differ): +0.347 ±
  0.175, 95% CI [+0.005, +0.689], winning 50.41% of rounds for 49.44%; a
  win by the rule, on one run with its bound at zero, where four seats gave
  +0.148 ± 0.276.  A second run on a fresh seed base did not repeat it
  (both at e40cca4, 3200 deals, seed base 1794846583, 1202 differ): +0.050 ±
  0.172, 95% CI [−0.287, +0.388], winning 51.47% for 51.47%; inconclusive.
  The Peeker at 128 samples wins at two seats (e40cca4 with the Peeker
  against e40cca4, `RESHUFFLE=0`, 3200 deals, seed base 1794847090, 1616
  differ): +2.824 ± 0.192, 95% CI [+2.449, +3.200], winning 60.88% of rounds
  for 49.31%.  Less the rerun that is +2.77 ± 0.26 points a deal, the most
  that reading the other hand could be worth at two seats.
- `HeuristicBot::terms` and `Terms`: the seven counts a hand's grade is the
  weighted sum of.  Play is unchanged: on paired deals against 10a046f
  (`ab/run.sh 20000`, seed base 1794823476), 0 of 20000 deals differ.
- The `fit` example and `research/fitted-weights.md`: what each term is
  worth in points, by least squares over 60000 rounds (seed base 20261119),
  once across the hands held at every discard and once between the choices
  of one discard, each played out.  A 胡 in the hand is worth about 1.4 points.
  Both fits weigh a loose card at about −48 where the bot has −128, a
  half-胡 short of the threshold at −11 to −18 against −64, and a card the
  hand wins on at about 25 against 8; yet the bot's own pick beats the
  discards graded within 16 of it by 0.46 points on average.
- The `fit` example also puts the grade in points, and
  `research/grade-in-points.md` reads it (60000 rounds, seed base 20261119;
  the earlier tables print unchanged).  Across hands the grade maps to
  points by a rising curve, about 0.5 points for each 32 below a grade of 0 and
  1.4 above, and by the stage of the round: a grade of 0 is worth +8.2 points
  with 23 cards or more in the pile, −2.0 with 14 to 22 and −4.9 with fewer.
  Between the choices of one discard it does not: the curve, fitted on half
  the rounds, tells a third of what a choice within 64 of the pick costs
  on the other half, and the cost itself does not rise with the gap, 4.1
  points at 32 to 64 behind and 1.6 at 64 to 128.  Choices within 8 of the
  pick cost nothing measurable (+0.04 ± 0.10).  The example prints the grade
  whole, where a 胡 held counts 32; its tables are those of d3bbb27 row for
  row (1600 rounds, seed base 20261119), their grades 32 times as large.
- The `dealt` example: the 胡 already in a dealt hand of twenty cards, the
  melds they come from, and the cards left outside every meld.  No bot
  plays.  Over 100000 deals (seed 20261118) a hand holds 6.63 胡 and 7.28
  loose cards on average.
- The web hint grades every move: each card in your hand and each claim
  button shows how far the bot grades it behind its pick, and 死 marks a
  move that leaves the hand short of the 胡 to win.  Equal moves show as
  equal.  The number is the bot's own grade, a whole number without a unit:
  it orders the moves and is no price (`research/grade-in-points.md`).
- `HeuristicBot::claims` and `HeuristicBot::discards` are public and return
  each move with its grade, best first.  The grade is the `i32` the bot
  sorts by, a priority without a unit: a 胡 held counts 32 in it, but it is
  not 胡.  Play is unchanged: on paired
  deals against fd5b9b2 (`ab/run.sh 20000`, seed base 1794786340), 0 of
  20000 deals differ.  Nor does returning the grade whole change it:
  against d3bbb27 (`ab/run.sh 20000`, seed base 1794845426), 0 of 20000
  deals differ.

### Changed

- The web help sheet is cut to the essentials: six rules and the 胡 table,
  with links to the book for the rest.  It opens with the rule that sets
  the game apart from mahjong: a drawn card is turned face up on the table
  and never enters the hand.

- On a touch screen a discard in the web page takes two taps: the first
  picks the card and lists what it leaves you waiting on, which a finger
  could not see for want of hovering, and the second plays it.  A mouse
  still plays on the first click.  The page goes by the pointer last
  pressed, and a new setting, 打牌, pins either way.  Applying the settings
  deals a new game only when the table changes.

- The web page fits a phone.  The hand stays on one row at any width: its
  cards narrow, and the pieces shrink with them, instead of wrapping.  On a
  screen under 40rem wide the page is exactly one screen tall and never
  wider: the table in the middle takes the room left over and scrolls by
  itself, so the hand and the buttons stay put; the cards you only read
  (melds, discards, waits) turn from strips to stubs, the opponents' backs
  close up, and the header wraps.

- A seat holding a pair may eat the third copy into a pair as well as pong
  it: 車馬包包 can eat a 包 into 包包 and keep 車馬包 in hand.  Nothing
  forbade it; the claim options left it out.  The bots still do not weigh
  it, so their play is unchanged: `ab/run.sh` on 200,000 deals, seed base
  1794854887, control 1945c1f, 0 deals part.

- An exposed meld on the web table leads with the card it took from the
  table, as the log reads and as a Japanese mahjong table lays out a chi,
  instead of putting it in the middle as a Taiwanese mahjong table does.
  Display only.
- The web hand puts its loose soldiers last, all colours together: beside a
  炮 they make nothing, but with soldiers of other colours they make 三色卒.
  Display only; the hint and the bots are untouched.
- `HeuristicBot` does not discard by danger: two trials did not win and
  play is unchanged (`ab/run.sh 20000`, control 0b5d48f, seed base
  1794817500, 0 of 20000 deals differ).  Both priced a discard at the rate
  cards win for each seat not proven safe against it, by that seat's
  concealed cards, on 200000 paired deals against 0b5d48f.  Taking 12 × 32
  times that rate off the grade (seed base 1794815549, 5328 differ):
  −0.037 ± 0.010 points a deal, 95% CI [−0.057, −0.017], a loss; the seat
  dealt in less but lost as often, since a waiting hand is offered every
  draw at the table, and won less.  Breaking only ties in grade by it (seed
  base 1794816883, 2513 differ): −0.002 ± 0.007, 95% CI [−0.015, +0.011],
  inconclusive.
- `HeuristicBot` keeps its hand-tuned weights: three trials of fitted ones
  did not win and play is unchanged.  Each ran 200000 paired deals against
  10a046f.  Every weight from the fit across hands (seed base 1794819512,
  59981 differ): −0.005 ± 0.034 points a deal, 95% CI [−0.071, +0.060],
  inconclusive.  `WAIT` alone, 8 to 24 (seed base 1794820392, 2221 differ):
  +0.002 ± 0.007, 95% CI [−0.012, +0.017], inconclusive.  The 搭子 term
  eightfold and `WAIT` at 72, from the fit between near choices (seed base
  1794822233, 35378 differ): −0.070 ± 0.025, 95% CI [−0.120, −0.021], a
  loss; over its first 60000 deals the seat lost where it broke a meld to
  keep a 搭子 and where it claimed or passed differently, not where it only
  kept a different loose card, and won 4% less often.
- `HeuristicBot` still claims out of a wait: one trial did not win and play
  is unchanged.  A hand that waits ponged or ate whenever that graded
  higher, though it left fewer unseen cards to win on: 帥帥將將將 炮炮 兵卒
  at its eight 胡 ponged the 炮 and went from eight waiting cards to three.
  Grading such a claim below passing, on 200000 paired deals against
  fc6ca57 (seed base 1794828275, 3607 differ): +0.007 ± 0.011 points a
  deal, 95% CI [−0.014, +0.029], winning 24.23% of rounds for 24.19%.
  Verdict: inconclusive.
- `HeuristicBot` does not count the melds a hand has room for: four trials
  did not win and play is unchanged.  The cards outside a hand's melds and
  the winning card make only so many melds of three, and what is left over
  must go in pairs, so a pair eaten early takes the place of a meld worth
  胡.  Each trial ran 100000 paired deals against b894777.  Hoping for the
  dearest 搭子 and pongs of pairs that fit, room rounded down (seed base
  1794933710, 28275 part): −0.290 ± 0.014 points a deal, 95% CI [−0.317,
  −0.263], a loss.  The same with the leftover held to pairs (seed base
  1794934225, 33313 part): −0.216 ± 0.014, 95% CI [−0.244, −0.189], a loss;
  both ate pairs the control passed on, since a hand with more hopes than
  room loses none by breaking a 搭子.  That cap on the 搭子 alone, no pongs
  hoped for (seed base 1794936088, 6093 part): +0.005 ± 0.004, 95% CI
  [−0.003, +0.014], inconclusive.  Eating no pair while the hand's melds
  are short of the threshold (seed base 1794934919, 52775 part): −1.497 ±
  0.020, 95% CI [−1.536, −1.458], winning 20.33% of rounds for 24.20%, a
  loss.  `research/room-for-melds.md` has the arithmetic, the patches and
  where each trial parted from the control.
- `MonteCarloBot` at 128 samples no longer measurably beats
  `HeuristicBot`, which has improved four times since 0.2.1 put it at 26.2%
  against 23.8%.  Seated among three `HeuristicBot`s and paired with a
  fourth on the same deals (`SAMPLES=128 ab/run.sh 3200`, control 6428dab,
  seed base 1794777431, 1082 of 3200 deals differ), it scores +0.148 ±
  0.276 points a deal, 95% CI [-0.392, +0.689], and wins 24.97% of rounds
  against 24.47%.  Verdict: inconclusive.

## [0.2.1] - 2026-11-16

### Added

- `MonteCarloBot` (feature `rand`): deals the unseen cards at random and
  plays the three choices `HeuristicBot` grades highest out with greedy
  bots, leaving the greedy choice only for one that scores better by a
  standard error.  At 128 samples it wins 26.2% of rounds against three
  `HeuristicBot`s, where a fourth `HeuristicBot` in its seat wins 23.8% of
  the same 3200 deals.
- `View::determinize` (feature `rand`): a round that looks the same from a
  seat, with the other hands and the pile dealt afresh.
- The `arena` example takes a second argument, the samples of a Monte Carlo
  bot to seat first, and a third, the seed of the deals.
- The `danger` example: how often each kind of card, put down at each stage
  of a round, would deal in, and into which meld.  Its reading is written up
  in `research/discard-danger.md`.

### Changed

- `HeuristicBot`, and so the web hint, keeps a waiting hand: a hand is
  graded 8 higher for each unseen card it would win on.  Two discards that
  left the same melds and loose cards used to tie whether or not one of
  them left the hand a card from winning — a 仕 or a 俥 off 帥帥仕俥炮, where
  only the 仕 waits on the 傌 — and the tie went to the later card.  A/B
  against `a74b951`, the new bot in one seat among three old ones and four
  old ones on the same 200000 deals (seed base 1794772616, 3551 differ):
  +0.111 ± 0.010 points a deal, 95% CI [+0.091, +0.132], winning 24.52% of
  rounds for 24.24%.  Verdict: win.
- `HeuristicBot`, and so the web hint, no longer eats a card into a meld
  the hand already holds whole — a 俥 into 俥傌炮 with no second 俥.  Passing
  was graded as if no draw followed, so trading a loose card for a milder
  one looked like a gain; the seat next to draw now claims only what beats
  the draw by 3 胡 of grade.  A/B against `6fc8f21`, the new bot in one
  seat among three old ones and four old ones on the same 200000 deals
  (seed base 1794765506): +0.283 ± 0.018 points a deal, 95% CI [+0.249,
  +0.318], and 25.05% of deals won where the old bot wins 24.50%.
- `HeuristicBot`, and so the web hint, tells eating the seat before's card
  from eating its own draw.  Leaving the first still brings a draw, which
  was graded at a flat 3 胡; it is now the mean, over the cards the seat has
  not seen, of its best answer to each — a win, a kong, a general it must
  eat, a claim where one beats leaving the card.  A hand with many cards to
  come holds out for them, and one with few takes what is offered.  Leaving
  its own draw hands the turn on, so nothing is added there, as before.  A/B
  against `1b644f2`, the new bot in one seat among three old ones and four
  old ones on the same 200000 deals (seed base 1794768139): +0.078 ± 0.015
  points a deal, 95% CI [+0.049, +0.108], a win; 24.41% of deals won where
  the old bot wins 24.28%.  A round of four such bots takes about six times
  as long, 15 ms where it took 2.3 ms.
- `HeuristicBot`, and so the web hint, holds 士象 without their general
  less dearly.  Only the seat's own draw brings a general and no seat
  discards one, yet the pair was graded on half the generals not yet seen;
  it is now graded as if one card were left to finish it.  A/B against
  `d74b170`, the new bot in one seat among three old ones and four old ones
  on the same 200000 deals (seed base 1794771322): +0.012 ± 0.006 points a
  deal, 95% CI [+0.001, +0.023], a win; 1538 deals differ, and 24.19% are
  won where the old bot wins 24.16%.  Tried beside it and dropped: grading
  the 士 or 象 next to its general for two cards, a loss alone (seed base
  1794770832: −0.050 ± 0.015, 95% CI [−0.079, −0.020]) and inconclusive
  with this change (seed base 1794770336: −0.027 ± 0.016, 95% CI [−0.058,
  +0.004]).
- `web/`: a meld on the table shows the card it took from the table in the
  middle, as a mahjong table lays out an eat — 車 eaten with 馬包 reads
  馬車包 — and so does the meld of the winning card.  Of four soldiers it is
  the second from the left.
- `web/`: 提示 rings the loose copy of the card to discard when your hand
  also holds one in a meld.

## [0.2.0] - 2026-11-15

### Added

- `Rules::eight`: 八胡, the same table with eight to win (and eight for a
  相公), at the same four seats and deal.
- `web/`: a 設定 dialog (or the `s` key) to choose 十胡 or 八胡.  Applying it
  starts a new game; the choice is kept in the browser.  The game is still
  titled 十胡; the how-to-play sheet and the 相公 message follow the table
  in play.
- `web/`: each seat shows the 胡 in the cards you can see of it: the melds on
  the table for the bots, and those plus the best melds within your hand for
  you, out of the 胡 to win.  Once a round is over the bots' revealed hands
  count too, and the winner shows the winning count.

### Changed

- `HeuristicBot` tells its loose cards apart.  It keeps a 搭子 (two cards
  one short of a meld), the dearer the meld the longer: soldiers of
  different colours, then 將士象, then 車馬包.  It sheds the cards that go
  with nothing first, 兵卒 and 士象 before 車馬包, so they are not left to
  finish another seat's hand late.  A card beside three or four alike is
  alone, since those never eat: 車車 馬 is a 搭子, 車車車 馬 is not, and
  neither is 車車 馬 beside 包包包, whose fourth 包 can only kong.  Against
  the previous bot, two seats each, it wins about 57% of the decided rounds.
- `HeuristicBot` counts the cards it has seen: its own, the melds on the
  table and the discards.  A 搭子 is worth more the more cards are left to
  finish it and nothing once they are all out; a loose card is worth a
  little for each copy of itself still to come.  A hand short of the
  threshold counts the 胡 its live 搭子 would bring towards it.  士象
  without their general rank below a 車馬包 搭子: only the seat's own draw
  brings a general, and it is worth a 胡 alone anyway.  Against the
  previous bot, two seats each, it wins about 52% of the decided rounds.
- `HeuristicBot` no longer melds itself dead: it refuses any claim or
  discard after which the hand could not reach the threshold even with a
  general for every card it gave up.  Every 相公 came from such a hand; four
  bots fouled about once in 1,500 rounds before.
- A drawn general no longer goes into its drawer's hand: nothing drawn ever
  does.  It is eaten onto the table, alone (`Meld::Single`) or, when the
  drawer holds the 士象 and chooses so, as 將士象.  With no choice it
  happens unasked; otherwise the drawer is asked and may not pass
  (`ClaimOptions::forced`).  `Event::Keep` is gone; the event is an
  `Event::Eat`.  Generals on the table do not count towards a kong.
- `web/`: a drawn general with a choice shows a bare 吃 (the general alone)
  and 吃 with the 士象, and no 過.
- `Seat` is no longer the compass enum 東南西北 but a seat number in turn
  order: `Seat::ALL[i]`, `Seat::index`, and `Seat::COUNT` for the number of
  players.  It displays as its number counted from 1.  `Seat::letter` and
  `Seat::name` are gone.
- `web/`: seats are named from where you sit — 你, 下家, 對家, 上家 —
  instead of by wind.
- `web/`: lone generals in your hand are shown among the loose cards, at
  the head of their colour, so 將士 and 將象 sit side by side.  Generals
  carry a coin mark under the piece wherever they appear, as on the printed
  cards.
- `web/`: the 吃 and 碰 buttons show only the cards that leave your hand,
  never the card on the table: 吃 帥相 takes a 仕, 碰 shows your pair.
- `web/`: cards no longer draw a thin inner frame.  On the small cards
  (opponents, discards, claim buttons) it ran through the piece.
- `web/`: the 吃 and 碰 buttons are outlined and 過 and 提示 are gold, so
  the yellow and white cards on a claim button no longer blend into it.

### Fixed

- Three or four identical cards in a hand (大肚: 俥俥俥傌炮) are no longer
  split when a hand is laid out.  A win could count 俥俥 and 俥傌炮, or take
  傌 with a 俥 and 炮 out of the triplet; the bots graded hands that way, and
  `web/` grouped and counted yours that way.  Needs susik 0.1.1 for
  `Solver::locked`.  Against the bot that split them, two seats each, the
  bot wins about 50.6% of the decided rounds; against the 0.1.0 bot, with
  every change above, about 60%.
- `web/`: the 莊 mark beside your own score no longer drops below it.
- `web/`: the result sheet shows the winner's melds in two rows: the exposed
  melds ending with the winning card's meld, then the concealed ones.  A
  single wrapping row could put a concealed meld among the exposed.

## [0.1.0] - 2026-11-14

### Added

- `Rules`: the Taiwanese 十胡 table.
- `Round`: the runtime-checked state machine — dealing, face-up draws,
  claims resolved as win > forced kong > pong > eat, locked triplets and
  locked claimed cards, dead cards, 天胡.
- `Game`: the scoreboard and dealer rotation.
- `View`, `Strategy`, `Table`, `HeuristicBot`: the bot framework and a greedy
  bot.  The bot plays for the ten: a hand short of the threshold keeps the
  melds that can still reach it, and a hand that has its ten races to meld
  its loose cards.
- Generals are melds by themselves: a drawn general is kept by its drawer
  (`Event::Keep`), generals are never ponged or eaten into a pair, and they
  cannot be discarded while the hand holds anything else
  (`RoundError::General`, `discardable`).
- 相公: a seat left with nothing to discard ends the round and pays ten 胡
  to each other seat (`Event::Foul`, `RoundResult::foul`).  Claims that
  would lead there are not allowed and are listed in
  `ClaimOptions::blocked`; the web front end shows them as disabled
  buttons.
- A claim that would leave no card to discard is not offered: a hand fully
  melded below ten cannot win, and used to strand the seat.
- `web/`: the browser front end, one human, sitting south, against three bots.  Your hand
  is arranged into its best melds automatically, loose cards last.  A `?`
  button (or the `?` key) opens a how-to-play sheet with the 胡 of each meld.
  A 提示 button (or the `h` key) rings the claim or discard a greedy bot
  would make in your place.  The felt is blue so the green suit stands out
  against it.
