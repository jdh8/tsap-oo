# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Two settlement rules, both off in `Rules::new`.  `Rules::base` is a 底:
  each seat pays the 底 and the 胡 above the threshold in place of the
  whole 胡 (`Rules::pays`; at a 底 of 1, 13 胡 is paid 1 + 3), and a 相公
  pays the threshold and the 底 (`Rules::foul`).  `Rules::flower` is 點花:
  at a win the top card of the pile is turned where it lies
  (`RoundResult::flower`), and each card like it in the winner's layout is
  one 花胡, one 胡 more from each seat (`RoundResult::flower_hu`).  Both
  are as Board Game Arena's Chinese help has them; the sources are in
  `../tsap-oo-book/research/settlement.md`.  `web/` and the desktop app
  offer them under 「設定」 as 「結算」 and 「點花」, and the result sheet
  shows the 點花, the 花胡 and what each seat pays.  No bot knows either
  rule and nothing is measured under them (`doc/variants.md`, "底 and
  點花"); the hint's points are still those of the whole 胡.

- `doc/opponents.md` plans reading the opponent at a table of two: an
  oracle for each channel before any inference is written.  `ORACLE=live`,
  `waits` or `danger` in `ab/` seats `ExpectBot` as it ships with one input
  read off the other hand (`ab/src/oracle.rs`): the cards the opponent
  keeps taken out of `live`, h bent by its waits, each discard's value less
  what the card gives away.  `ORACLE=cards` bends h by the opponent's count
  of concealed cards, which is public.  `cargo run --release --example
  hazard -- [rounds] [seed]` measures the two bends: how often another seat
  wins on a card by how many draws on it comes, over the same at decisions
  with as many draws left.  At two players, 20,000 rounds, seed 400: 0.21
  times on the next cards of a seat that is not waiting, 3.0 times of one
  that waits on four copies or more; 0.29 to 1.44 times by the count of
  cards.  The runs are declared in the document.  On the web setting at two
  players and 10 胡, 10,000 deals each against `1430e6b`, `RESHUFFLE=8`:
  `danger` +0.984 ± 0.046 points a deal, 95% CI [+0.893, +1.074], 7,679
  deals part, seed base 1803600000, win; `live` +0.107 ± 0.036, [+0.037,
  +0.178], 7,610 part, seed base 1803500000, win; `waits` +0.011 ± 0.018,
  [−0.024, +0.047], 2,320 part, seed base 1803300000, and `cards` −0.009 ±
  0.015, [−0.039, +0.020], 1,660 part, seed base 1803400000, both
  insufficient data and closed: neither CI reaches +0.1.  The zero runs of
  the hooks part on no deal (2,000, 2,000 and 500 deals, seed bases
  1803000000 to 1803200000).  On the desktop setting, 8,000 deals:
  `danger` +1.039 ± 0.051, [+0.939, +1.139], 6,024 part, seed base
  1804400000, win, and `live` +0.172 ± 0.039, [+0.095, +0.249], 5,921 part,
  seed base 1804300000, win, so both are open.  The Peeker
  (`RESHUFFLE=0`, 3,200 deals, seed base 1803700000) against the desktop
  bot: +1.874 ± 0.200, [+1.482, +2.267], 1,806 differ, win.  The bot that
  ships against `HeuristicBot`: two players web +0.405 ± 0.042, [+0.324,
  +0.487], 10,000 deals, seed base 1803800000; three players web +0.348 ±
  0.063, [+0.225, +0.472], 10,000 deals, 1803900000; two players desktop
  +0.615 ± 0.067, [+0.483, +0.746], 4,000 deals, 1804000000; all wins.  `ORACLE=waits` and
  `ORACLE=cards` were then taken out of `ab/`, leaving `live` and `danger`;
  `01f98fd` has them.  No bot changes.

- `Giveaway` reads what a discard gives the next seat off the public
  cards, and `ExpectBot::wary` discards by each card's value less its
  `Giveaway::cost`; it is off unless asked for, and no bot that ships asks
  yet.  A card's `Evidence` is whether the next seat threw a copy from its
  hand (`View::thrown`), let one go (a copy on the pile, or `View::safe`),
  or neither, and then how many kinds of the card's run it threw and how
  many copies of them are unseen.  `cargo run --release --example giveaway
  -- [rounds] [seed]` measures the costs: the web page's bot against
  `HeuristicBot` at a table of two, every card it could discard played as
  the `danger` oracle plays it.  Over 4,500 rounds each way, seed 600,
  `HeuristicBot` claims 41.2% of the cards it has shown nothing about, 7.7%
  of those whose run it threw a kind of, 2.3% of those it let go and none
  it threw; the costs run from 0.002 to 1.236 points.  `ORACLE=win` and
  `ORACLE=claim` in `ab/` are the two halves of `danger`, and `WARY=1`
  seats `ExpectBot::wary`.  The design and the runs declared for it are
  batch 3 of `doc/opponents.md`.  At two players and 10 胡 on the web
  setting, control `6a61a3e`, `RESHUFFLE=8`: the bots as they ship part on
  0 of 2,000 deals (seed base 1804500000); `ORACLE=win` is +0.471 ± 0.025
  points a deal, 95% CI [+0.422, +0.521], over 10,000 deals (1804600000),
  a win; `ORACLE=claim` is +0.246 ± 0.043, [+0.162, +0.330], over 10,000
  (1804700000), a win; `WARY=1` is −0.061 ± 0.016, [−0.092, −0.030], over
  40,000 (1804800000), a loss, so `wary` stays off and its desktop run was
  not made.  The trace of that run closes the tables as fitted: the deal
  pays 0.315 ± 0.042 for a card eaten where the oracle, one card ahead,
  charges 1.747, and a card ponged is the better one to throw.

- `cargo run --release --example kept -- [rounds] [seed]` measures what
  the public cards say of the cards still to come at a table of two: where
  the copies of each kind the web page's bot has not seen are, and how many
  come up before the round ends, by the kind's `Evidence`.  Over 4,500
  rounds each way, seed 700, `HeuristicBot` keeps 7.2% of the copies of a
  kind it threw and 14.3% of one it has shown nothing about, and of the
  cards it leaves free the first come up 0.988 times as often as their rank
  at large and the second 1.004: next to nothing.  An `ExpectBot` counting
  each unseen card by that table (`COUNTING=1` in `ab/`) was measured and
  taken out; `dbe68dc` has it.  At two players and 10 胡 on the web
  setting, control `8a8ecf9`, `RESHUFFLE=8`: the bots as they ship parted
  on 0 of 2,000 deals with the hook in (seed base 1805000000); `COUNTING=1`
  was −0.006 ± 0.009 points a deal, 95% CI [−0.025, +0.012], over 40,000
  deals (1805100000), 9,929 part: insufficient data, and closed as
  declared, since the CI does not reach +0.05.  Its desktop run was not
  made.  The design is batch 3 of `doc/opponents.md`.  No bot changes.

- `cargo run --release --example posterior -- [rounds] [seed]` measures
  what the other seat's whole history says of its hand at a table of two:
  hands dealt from the unseen cards and replayed through `HeuristicBot`,
  the ones that reproduce every public event kept, against the hand it
  holds.  `CHAIN` samples by swaps from that hand where dealing thins out.
  The design is batch 4 of `doc/opponents.md`.  No bot changes.

- `TRACE=FILE` has `ab/` write a line for each deal that parts: its seed,
  the mean and spread of the difference among its shuffles, where the two
  parted and on which acts; and for two discards of an `EXPECT` bot at a
  table of two at 10 胡, each card's value, `Evidence`, `Giveaway::cost`,
  what the `danger` oracle keeps of it and what the other seat does with
  it.  Sorted by the mean it is the deals that moved the most, which a
  loss is owed before it is called dead.  No bot changes.

- `ExpectBot::outlook_mut` gives the bot's table model, to read or to change
  between decisions.

- `Outlook::new` has a table for two players at 8 胡, measured from 10,000
  rounds of `HeuristicBot` self-play (`PLAYERS=2 THRESHOLD=8 outlook 10000
  280`): within the gate h is 0.0694 a card against 0.0544 at 10 胡, L
  −11.32, e 0.727; its h calibrates at × 1.3 (seed 283).
  `examples/eval.rs` takes `THRESHOLD`, and `BASELINE_EXPECT=new` in `ab/`
  has the control read its own `Eval::of` past the gate, where any other
  `BASELINE_EXPECT` reads its `Eval::default`.

- `cargo run --release --example away -- CLONE [deals] [seed]` is the
  return match of `guest`: `ExpectBot` at chunhua1972/four-color-game's
  table (`ee33cf0`), where `node` runs their own engine from a clone
  through `examples/away.mjs`: their rules, their scores, their Normal AI
  as host.  The bot keeps its beliefs from home (a pair of soldiers
  melds, the claimed card is locked, a win pays its 胡), and wins and
  kongs are always taken.  10,000 deals, seed base 1795470698, seat 1,
  dealer rotating: among three hosts, guest − host is +1.551 ± 0.088
  points a deal, 95% CI [+1.377, +1.724]; among three guests, host −
  guest is −1.270 ± 0.078, 95% CI [−1.422, −1.117].  No illegal action
  and no foul by the guest.

- `cargo run --release --example guest -- CLONE [deals] [seed] [expect]`
  plays a match against another program's bot: the Normal AI of
  chunhua1972/four-color-game (`ee33cf0`), which `node` imports from a
  clone through `examples/guest.mjs`.  Their repository states no licence
  and none of it is copied.  `Round` stays the rules authority, so this
  is the guest adapted to this table.  10,000 deals, seed base
  1795469454, seat 1, dealer rotating, host `HeuristicBot`: among three
  hosts, guest − host is −4.003 ± 0.217 points a deal, 95% CI
  [−4.429, −3.577]; among three guests, host − guest is +5.456 ± 0.244,
  95% CI [+4.978, +5.934].

- `ab/queue.sh SLUG [deals] [control]` queues an A/B through pueue, one at
  a time: it freezes the checkout, uncommitted files included, into a
  worktree under `~/ab/` and runs `ab/run.sh` there at `nice -n19`
  (`SCHED_IDLE` where `chrt` exists), so the main checkout stays free to
  edit and build.  Not background QoS: `taskpolicy -b` keeps the run on an
  M4's six E-cores, 68.6 s against 12.1 s for 1,200 deals; `nice -n19`
  took 12.6 s.  No bot changes: `ab/queue.sh pueue-harness 20000`
  against `04d3609`, seed base 1795459850, reads 0 deals that part.

- `ab/cores.sh 0|1|2` is the queue's power switch on a Mac: 0 pauses the
  queue and the run in flight, 1 keeps pueue and its tasks on the E-cores
  (background QoS, set on the running process tree and inherited by tasks
  that start later), 2 gives every core back.  1 runs cooler and quieter
  but about five times slower, and is not cheaper: the rest of the machine
  stays awake all that time.  A fresh pueued starts at 2.

- `AFTER=ID ab/queue.sh ...` queues a run behind pueue task ID and runs
  it only if that task's log says `Verdict: win`; otherwise the task
  shows as failed.

- `doc/generals.md`, batch 4.  `HeuristicBot` counting a general it waits
  on as 3 where any other card counts 8 (8 × ρ) is not adopted: against
  `8dc900b` with `RESHUFFLE=8`, 200,000 deals, seed base 1795461522, 435
  deals part, −0.000 ± 0.001 points a deal, 95% CI [−0.003, +0.002],
  insufficient data; the tables of three, of two and of 8 胡 were gated on
  a win and did not run.  `examples/overspend` rerun on `8dc900b`, 20,000
  rounds, seed base 1796461521: `ExpectBot` eats 781 of the 4,144 pairs
  that leave the hand a general short (18.8%, from 24.9%), and 27 of the
  60 that leave it fully melded with nothing else to wait on (from 35 of
  39); its estimate of a waiting hand's chance is off by 3.3 percentage
  points on average.  No bot changes.

- `research/settlement-variants.md`: what the public write-ups add on top
  of "the winner collects their 胡 from each seat": 底, 花, 押點, the 高胡
  variant, 五龍 doubling, 金門's 加三/加八 and 開六/拾八 offsets.  None
  has the discarder pay alone.  Nothing modelled.

- `examples/outlook.rs` measures the generals' rate ρ (`doc/generals.md`,
  batch 1): the generals among the public cards over the sum of K ÷ N, K
  the generals and N the cards the watching seat has not seen, rebuilt
  from its dealt hand and the events, by draws left; and the same ratio
  for every rank.  `HeuristicBot` self-play, 20,000 rounds a table, seed
  base 1795364857, base `74e99e2`, seats with 10 concealed cards or fewer:
  ρ is 0.406 ± 0.002 at four seats and 10 胡, 0.515 at three, 0.647 at
  two and 0.383 at 8 胡, under the 0.85 gate.  `Graph` takes a public card
  for a general more than twice as often as it is one.  Discarded ranks
  (士象車馬包) come up 16 to 33% more often than an even spread, 兵卒
  32% less early in the round.  Measurement only: `ab/run.sh 20000`
  against `74e99e2`, seed base 1795365381, reads 0 deals that part.

- `doc/generals.md`: the plan that follows from `examples/overspend.rs`,
  in batches with gates: measure how often a public card is a general
  against the unseen generals over the unseen cards `Graph` takes it for,
  put the ratio in `Outlook`, relearn `Eval`, then look again at the pairs
  eaten a general short.  No bot changes.

- `doc/generals.md`, batch 3: `Eval` relearnt under the four-player
  `Outlook` that won batch 2b (rank rates, h × 1; 2,000 labelled rounds,
  seed base 1796250000, D = 16, held-out RMSE 0.90) is not adopted.
  Against `3423521` with `RESHUFFLE=8`, neither setting is a win: web
  +0.019 ± 0.025 points a deal, 95% CI [−0.030, +0.069] (60,000 deals,
  37,228 part, seed base 1795429248); desktop +0.020 ± 0.036, 95% CI
  [−0.050, +0.091] (20,000 deals, 9,262 part, seed base 1796429248).
  Insufficient data, not a loss; the batch is dropped because an effect
  of that size is not worth a second table.  No bot changes.

- `examples/overspend.rs` and `research/overspend.py`: every pair the desktop
  `ExpectBot` may eat, by the hand's `ceiling` before and after with no
  general from outside and with one, and every hand a discard leaves waiting,
  with the DP's chance to win and the outcome.  Seat 1 among three
  `HeuristicBot`s, 20,000 rounds, seed base 1797200000, base `b95dde4`.  A
  hand that wins on every general not yet seen (fully melded) and on nothing
  else won 50% ± 4 with 15 draws or more left, 39% ± 3 with 8 to 14 and 23%
  ± 4 with 7 or fewer, where the DP gave it 65%, 58% and 43%; two cards one
  short of a meld waiting on four cards won 54%, 41% and 27% against 46%, 37%
  and 27%.  One that also waits on three cards or more that extend a concealed
  meld (a pong, a kong, a fourth colour) is valued within 4 points from 8
  draws up; with one or two, up to 14 high.  The bot ate 1,039 of the 4,176
  pairs that leave the hand a general short of the threshold, and 35 of the 39
  that leave it fully melded with nothing else to wait on, by its own values
  10.7 points better than not.  Measurement only; no bot plays differently.

- `examples/arena.rs` breaks the winning layouts down by meld: melds a win
  and the 胡 they bring, with 碰 and 明槓 apart from 暗刻 and 暗槓, then
  the same for the winner's dealt hand laid out for the most 胡.  At
  `4e461d0`, 20,000 rounds, seed 20261117: 12.21 胡 a win, 將帥 2.76,
  將士象 1.92, 三色兵卒 1.79, 明槓 1.30, 碰 1.28, 暗刻 1.17; the winners
  were dealt 7.90 胡, against 6.63 for a hand off the deal (`dealt`).

- Tables of two and three have their own `Outlook`, measured under
  `Rules::dead` from 10,000 rounds each of `HeuristicBot` self-play (seeds
  210 and 310, `research/outlook.md`, "Empty seats' hands are dead"): at two a card
  comes to the seat to eat 0.74 of the time and at three 0.42, against 0.28
  at four.  `Outlook::new` picks it by `Rules::players`,
  `examples/outlook.rs` takes `PLAYERS`, and `ab/` takes `FOUR` to keep the
  table of four in the candidate.  `eval::DRAWS` is the 25 draws a round
  starts with.  Which table plays by its own is under Changed.  The tables
  measured before `Rules::dead`, over rounds of 65 and 45 draws, and their
  A/Bs are in `research/outlook.md`, "Tables of two and three", and no
  longer hold.

- `Outlook::new` picks the table model by `Rules::threshold`: at 8 胡 one
  measured from 10,000 rounds of `HeuristicBot` self-play at eight to win,
  seed 800 (`research/outlook.md`, 8 胡).  `examples/outlook.rs` and `ab/`'s
  `EXPECT` candidate take it; `examples/outlook.rs` takes `THRESHOLD`.
  `ExpectBot::new` still plays the 10-胡 table: A/B of the 8 胡 table at
  `THRESHOLD=8` against `c922a91`, `RESHUFFLE=8`, h × 1.5 on both sides: on
  the web's settings +0.022 ± 0.013 points a deal, 95% CI [−0.004, +0.048],
  40,000 deals, seed base 1795400000; on the desktop's +0.005 ± 0.024, 95%
  CI [−0.041, +0.052], 20,000 deals, seed base 1795500000.  Both
  insufficient data; play is left as it is.

- `research/discard-order.md`: 好牌先打 put to the A/B.  `HeuristicBot`
  fines a lone card `COPIES * eaten[kin]`, so the dearest goes first
  (兵卒 3, 將士象 2, 車馬包 1).  Against `b25fb0c`, `RESHUFFLE=8`, 100,000
  deals each: all at 2, −0.057 ± 0.017 points a deal, 95% CI [−0.090,
  −0.025], seed base 1795400000, a loss; the order reversed, +0.027 ±
  0.022, 95% CI [−0.015, +0.070], seed base 1795500000, insufficient data.
  A trace of the loss's first 20,000 deals puts it in the eats the fine
  also values, not in the discards (+0.02 ± 0.08 a split).  Play is left
  as it is.

- `doc/variants.md`: what 8 胡 and tables of two and three still lack.  The
  rules are complete; the bots' constants, `Outlook` and `Eval` come from
  four players at 10 胡 and are used as they are.

- `ab/` takes `THRESHOLD`, the 胡 to win on both sides (default 10), to
  measure a bot at 8 胡.  At 8 胡 `ExpectBot` plays past its gate by the
  shallow graph on the 10-胡 `Eval` tables, which had not been measured.
  A/B at `THRESHOLD=8` against `09eb4c4`, `RESHUFFLE=8`, the control
  playing past the same gate as `HeuristicBot`: on the web's settings
  +0.006 ± 0.074 points a deal, 95% CI [−0.138, +0.151], 10,000 deals, seed
  base 1795200000; on the desktop's +0.080 ± 0.049, 95% CI [−0.015, +0.175],
  20,000 deals, seed base 1795300000.  Both insufficient data; play is left
  as it is.

- `ExpectBot::shallow`, batch 4 of `doc/eval.md`: past its gate the bot
  plays by a graph two claims deep up to a card count and one deep beyond,
  each leaf valued by `Eval`, in place of `HeuristicBot`.
  `ExpectBot::searches` says where it values its answers in points.  The
  web's hint plays it one claim deep past its 10-card gate, and the
  desktop's opponents and hint two deep to 17 cards and one beyond past their
  14-card gate, so the hint marks the expected points given up on every
  move.  A/B against `5c3cc24`, `RESHUFFLE=8`: on the web +0.191 ± 0.046
  points a deal, 95% CI [+0.100, +0.281], 30,000 deals, seed base
  1795055385, win; on the desktop +0.204 ± 0.052, 95% CI [+0.102, +0.306],
  20,000 deals, seed base 1795155385, win.  A wasm decision takes 18 ms on
  average, p95 55 ms, 1.1 s at worst; a native desktop one 62 ms, p95
  230 ms.  `ab/` takes `SHALLOW` and `BASELINE_GATE`, and
  `examples/expect_time` a `deep`.

- Tables of two and three: `Rules::players`, 2 to `Seat::COUNT`, 4 by
  default, dealt and scored as at four (twenty each, a win or a 相公 settled
  with each other seat).  The web and desktop clients choose it under 設定 →
  人數.  `Seat::advance` and `Seat::next` take the table size, and
  `Round::from_pile` rejects a table outside 2 to 4 or a dealer beyond it.
  `ExpectBot` prices a win and a 相公 by the seats at the table; the rest
  of every bot is as tuned at four.  Play at four is unchanged:
  `ab/run.sh 20000` against `6ebab9b` (seed base 1795053807) parts on no
  deal.

- `Eval`, batch 3 of `doc/eval.md`: what a hand is worth in points by each
  count of public cards to come, from nine pattern tables (each colour's
  將士象 and 車馬包, and the soldiers, symmetric patterns merged), the
  exposed 胡 and the live count, summed past a ReLU.  `Eval::default` is
  the width-16 table built in (`src/eval-16.bin`, 359 KB), learnt from
  `Graph`'s values; `Graph::depth` goes with it.  No bot uses it yet.
  `examples/eval` labels, trains and measures: labels from 2,000 deals
  (seed base 1795800000), held-out RMSE 0.78 points against 3.37 for the
  mean.  On 500 fresh deals (seed base 1795900000, base `7f0d5f1`) a graph
  one claim deep with these leaves would gain +0.34 ± 0.06 points a deal on
  the web at 11 to 20 cards, and two deep to 17 cards and one beyond
  +0.27 ± 0.06 on the desktop, past the 0.05 gate; the leaves' mean error is
  −0.03 points, slope 0.97, residual SD 0.65.  Predicted, not played: the
  A/B is batch 4.  No bot changes: `ab/run.sh 20000` against `7f0d5f1`
  (seed base 1796300000, `RESHUFFLE=8`) read 0 deals that part.
- `Prune::depth`, batch 2 of `doc/eval.md`: a node that many claims from
  the nearest root is a leaf, not expanded, and `Graph::values_with` reads
  its value from a curve by public cards to come; `Graph::curves`,
  `Graph::leaves` and `Graph::state` go with it.  No bot uses it yet.
  `nodes` takes the depth as a fifth argument (200 deals, seed 1795500000):
  depth 1 builds a 20-card decision in 29 ms, depth 2 in 0.5 s.
  `examples/leaves` measures what error the leaves bear (2,000 deals, seed
  base 1795600000, base `93b2d7b`): at depth 1, an offset of σ = 2 points
  for each leaf, one of ±1 shared by all, or every value halved costs at
  most 0.03 points a decision at 11 to 14 cards, under half of what
  `HeuristicBot` gives up; at depth 0 the per-leaf σ must stay within 0.4.
  No bot changes: `ab/run.sh 20000` against `93b2d7b` (seed base
  1795700000, `RESHUFFLE=8`) read 0 deals that part.
- `examples/regret`, batch 1 of `doc/eval.md`: at each decision of the
  desktop `ExpectBot` (gate 14) among three `HeuristicBot`s, what
  `HeuristicBot`'s first choice gives up by the DP's values, and the DP's
  win chance against the outcome.  1,000 deals, seed base 1795400000, base
  `9794936`: scaled by application D's +0.132, the decisions at 15 to 20
  cards are worth +0.35 ± 0.07 points a deal, past the 0.05 gate.  The
  win chance runs 11% high at 10 cards or fewer and 24% low at 15 or more.
  Measurement only; no bot plays differently.
- `cargo desktop`, an alias in `.cargo/config.toml` that runs the desktop
  client from the repository root (`cd tauri && cargo run --release`).
- `ExpectBot`: with 10 concealed cards or fewer it plays every claim and
  discard by the single-player DP of `doc/expectation.md` (`Graph`), in
  points; before that it plays as `HeuristicBot`.  Its table model is
  `Outlook::default` with h × 1.5 (`END_SCALE`).  `ab/` seats it with
  `EXPECT=<h scale>`.  Batch 3, against `b920a26` (`RESHUFFLE=8`, 100,000
  deals each, declared before the data): measured h (seed base
  1794988309) +0.071 ± 0.008 points a deal, 95% CI [+0.055, +0.087],
  7,526 deals part, a win; h × 1.5 (seed base 1795088309) +0.076 ± 0.008,
  [+0.060, +0.093], 7,631 part, a win.  The two are within error of each
  other; the higher was declared the default.  That matches the gated
  Monte Carlo's +0.070 ± 0.026 at a fraction of the cost.
- `ab/` takes `BASELINE_SAMPLES=<n>`: the control's seat 1 is then a
  `MonteCarloBot` of n samples gated at `GATE_CARDS`, for a head to head
  of two bots that each beat `HeuristicBot`.
- `ab/` takes `BASELINE_EXPECT=1`: the control's seat 1 is then its own
  `ExpectBot::new`, for a wider gate against the one in place.
  `examples/expect_time` takes a `[gate]` and times each card count above
  10.  `ab/run.sh` renames the control with `perl` instead of GNU `sed`,
  so it runs on macOS.

### Changed

- Two players at 8 胡 play by tables of their own, as two players at 10 胡
  do.  `ExpectBot::new` takes the `Outlook` measured at 8 胡 (h × 1.3), and
  `Eval::of(&Rules)` picks tables labelled under it
  (`src/eval-16-2-8.bin`, 359 KB, from `PLAYERS=2 THRESHOLD=8 EXPECT=1.3
  eval label 2000 1801600000`; held-out RMSE 0.40 points); the web and
  desktop clients' hint and opponents use both, and the hint's points past
  the gate are now 8 胡's.  Until now this table played by the tables of
  four players at 10 胡, where a hand 2 胡 short of winning is taken for
  one 4 胡 short.  A/Bs against `1433f27` playing so, `RESHUFFLE=8`, deal
  counts and the table judged (D = 16) declared before the labels
  (`doc/eval.md`, "An `Eval` for two players at 8 胡"):

  - Web settings: +0.445 ± 0.040 points a deal, 95% CI [+0.367, +0.522],
    10,000 deals, 7,858 part, seed base 1801700000, win.
  - Desktop settings: +0.154 ± 0.037, [+0.081, +0.226], 8,000 deals, 5,504
    part, seed base 1801800000, win.
  - Every other table reads 0 deals that part against `1433f27`: on the
    web's settings four, three and two players at 10 胡 and four and three
    at 8 胡, 2,000 deals each, seed bases 1802000000 to 1802400000; on the
    desktop's four, three and two players at 10 胡, 500 deals each, seed
    bases 1802500000 to 1802700000.  Before the table was wired in, two
    players at 8 胡 read 0 of 2,000 too (seed base 1801900000).

  The wasm grows from 982 KB to 1,344 KB (782 KB to 1,116 KB gzipped).

- This repository is English throughout.  The design docs in `doc/` and the
  measurements in `research/` were Chinese and are translated in place:
  every number, table, command, seed base and SHA is as it was, and the
  section titles other files and comments cite are now the English ones.
  `research/overspend.py` prints English labels, and the groups 甲, 乙 and
  丙 of `doc/hu-ceiling.md` and `research/hu-ceiling-trace.patch` are A, B
  and C.  Card faces and the game's own terms (胡, 將士象, 搭子) stay in
  Chinese, and so does the page in `web/`.  Research into what the rules
  are from source to source moved to `../tsap-oo-book/research/`, in
  Chinese: `generals-triplets-and-kongs.md`,
  `soldier-pairs-and-triplets.md` and, as `small-tables.md`, the
  per-source table of two- and three-player rules from
  `research/prior-art.md`.  No change to play: only comments changed in
  `src/`.

- `Graph` weighs each public card to come by its rank's rate
  (`Outlook::drawn` for a segment's draw, `Outlook::discarded` for the
  other seats' discards, by phase, 士象 and 車馬炮 pooled) where it took
  every unseen card as alike, and at four players and 10 胡
  `ExpectBot::new` plays by the measured h unscaled in place of
  `END_SCALE`; other tables keep the even draw and `END_SCALE`
  (`doc/generals.md`, batch 2b).  Calibrated on `examples/overspend`,
  20,000 rounds, seed base 1795450000: the mean miss over 33 cells of the
  waiting table falls from 5.9 to 2.4 points.  `ab/run.sh` against
  `a9e194e`, `RESHUFFLE=8`: the web setting, 60,000 deals, seed base
  1794000000, 25,242 part, +0.056 ± 0.023 a deal, CI [+0.012, +0.101],
  **win**; the desktop setting, 20,000 deals, seed base 1794200000, 10,655
  part, +0.103 ± 0.043, CI [+0.019, +0.187], **win**.
  `examples/overspend.rs` now uses the bot's own outlook.

- `examples/arena.rs` shuffles every deal first, on one thread, and plays
  four `HeuristicBot`s on every core: the table is the same from each seat,
  so a deal is played from the first seat and turned to its dealer
  afterwards.  The output is the same to the byte (`1500 0 7`, `1500`,
  `40 4 3`), and `20000 0 20261117` takes 2.4 minutes on 16 cores in place
  of some 33.  A Monte Carlo bot of two samples or more keeps its seat and
  is played round by round as before.

- `web/` (and so the desktop client) no longer sets the melds of a hand
  apart, in play or on the result sheet: a hand lies in card order, colour
  by colour, with the soldiers last.  Outside the
  soldiers a card melds only within its colour, so the colours read like
  the suits of a bridge hand.  The result sheet shows the winner's melds on
  the table, then the hand with the winning card by itself to its right, as
  at a mahjong table.  No bot is touched.
- A table of three or two leaves the hand of each empty seat dead.
  `Rules::dead()` is the cards never drawn: `dead_tail`, now the six of a
  full table, and twenty for each seat left empty, so 6 at four, 26 at
  three and 46 at two, and every table draws 25 cards a round.  With the
  six alone, as Board Game Arena deals, a table of three had 45 cards to
  draw and a table of two 65; no source says what a smaller table changes.
  In `HeuristicBot` self-play a round takes 12 draws whatever the table and
  the pile, so the longer pile only kept a smaller table from ever
  exhausting: 0.0% of rounds at three and at two against 3.2% at four, and
  3.5% and 3.8% with 25 to draw (2,000 rounds a row, `length 2000 7` of
  `research/dead-tail.patch` on `107032c`; `doc/variants.md`, "The dead cards of a small table").
  The web and desktop clients show the table's dead cards.  Play at four is
  unchanged: against `107032c`, `RESHUFFLE=8`, no deal parts in 20,000 of
  `HeuristicBot` (seed base 1798500000), 20,000 of `ExpectBot` on the web's
  settings (1798600000) or 2,000 on the desktop's (1798700000).

  The smaller tables' own `Outlook`s were measured by draws left over the
  longer rounds and no longer hold; `Outlook::new` has them measured anew
  (the next entry).  Past its gate `ExpectBot` now plays the shallow graph from
  the deal on at a smaller table too, every round starting within
  `eval::DRAWS`.

- Two players at 10 胡 play by tables of their own.  `ExpectBot::new`
  takes that table's `Outlook`, measured anew under `Rules::dead` (10,000
  rounds, seed 210; h × 1.3 by its calibration), and `Eval::of(&Rules)`
  picks tables labelled with it (`src/eval-16-2.bin`, 359 KB, from
  `PLAYERS=2 EXPECT=1.3 eval label 2000 1799800000`; held-out RMSE 0.53
  points); the web and desktop clients' hint and opponents use both.
  Cutting the pile gives a smaller table the draws of a full one, not the
  share of them: two players still draw twice as often each, and a card
  comes to the seat to eat 0.74 of the time against 0.28 at four.  The
  four-player tables read 3.06 points low a leaf at two players and 0.96 at
  three (300 rounds each, seed bases 1800000000 and 1800100000).  A/Bs
  against `07d5602` (the dead hands alone, not on the branch),
  `RESHUFFLE=8`, deal counts declared first (`doc/eval.md`, "Tables for two and three players"):

  - The four-player tables past the gate at a smaller table, against
    playing as `HeuristicBot` there, web settings: at two −0.184 ± 0.045
    points a deal, 95% CI [−0.271, −0.096], 10,000 deals, seed base
    1798800000, loss; at three +0.216 ± 0.051, [+0.116, +0.315], 15,000
    deals, seed base 1798900000, win.
  - Two players' own `Outlook` and `Eval`: on the web's settings, against
    playing as `HeuristicBot` past the gate, +0.375 ± 0.042, [+0.293,
    +0.457], 10,000 deals, seed base 1800200000, win; on the desktop's,
    against the four-player tables, +0.320 ± 0.056, [+0.210, +0.431], 4,000
    deals, seed base 1800400000, win.
  - Three players' own (h × 1.4, labels of seed base 1799900000) against
    the four-player tables: web −0.021 ± 0.025, [−0.070, +0.028], 40,000
    deals, seed base 1800300000; desktop −0.095 ± 0.061, [−0.215, +0.025],
    6,000 deals, seed base 1800500000.  Both insufficient data: three
    players keep the tables of four, and theirs are not built in.
  - Two players at 8 胡 by the 10-胡 tables of two against those of four:
    web +0.302 ± 0.040, [+0.224, +0.380], 10,000 deals, seed base
    1800600000, win; desktop +0.048 ± 0.056, [−0.061, +0.158], 4,000 deals,
    seed base 1800700000, insufficient data.  Declared as both or neither,
    so 8 胡 at two keeps the tables of four.

  Every other table plays as before: against `07d5602` no deal parts in
  2,000 on the web's settings at four, at three, at 8 胡 of two, three and
  four (seed bases 1800800000 to 1801200000) or in 500 on the desktop's at
  four, three and 8 胡 of two (1801300000 to 1801500000).  The web's wasm
  grows from 620 KB to 982 KB.  `examples/eval` takes `PLAYERS` and
  `EXPECT`, and `ab/` takes `EVAL`, a file of tables for the candidate, and
  `EXPECT=new`, the bot as it ships.

- Past its gate `ExpectBot` leaves the shallow graph alone while more than
  `eval::DRAWS` draws are left, whatever its own outlook counts a draw.  A
  first round of the A/B above lost without it at two on the web's settings
  (−0.019 ± 0.004 at h × 1.2, 95% CI [−0.027, −0.011], seed base
  1796400000; −0.011 ± 0.004 at × 1.5, 95% CI [−0.019, −0.003], seed base
  1796500000; 100,000 deals each): a trace puts the loss in 456 splits where
  the smaller table's outlook let the four-player `Eval` in up to 41 draws
  from the end, −5.8 ± 0.3 points each, and none of it inside the gate
  (+0.12 ± 0.06 a split).  With `Outlook::default` the two conditions are
  one, and play is unchanged: with the table of four in the candidate
  (`FOUR`) no deal parts at two, three or four players, 20,000 deals each
  on the web's settings and 2,000 on the desktop's, seed bases 1797700000
  to 1798200000.

- The web hint shows what the bot makes of each move in place of how far it
  falls behind the pick: where `ExpectBot` searches, the expected points of
  the round after the move, plus or minus; elsewhere, `HeuristicBot`'s whole
  grade.  The ring still marks the highest, and 死 a move that leaves the
  hand dead.

- The hint is `ExpectBot`'s instead of `HeuristicBot`'s, in the browser
  and the desktop client: with 10 concealed cards or fewer each mark is the
  expected points the move gives up against the pick, two decimals; before
  that the marks stay `HeuristicBot`'s unitless grades.  Batch 4C of
  `doc/expectation.md`, on batch 3's win (the hint is the bot); as wasm
  under V8 (`examples/expect_time`, 2,000 rounds, seed base 1000) a
  decision at the gate takes 6.6 ms on average, 84 ms at most.
- The desktop client's opponents and its hint play by the DP with 14
  concealed cards or fewer instead of 10; the browser stays at 10.  A
  decision at 14 cards takes some 190 ms of native code, 1.4 s at worst
  (`expect_time 300 1000 14`).  Application D of `doc/expectation.md`,
  gate 14 against the control's own `ExpectBot` gated at 10, three
  `HeuristicBot`s beside (control `e1d0c7d`, `RESHUFFLE=8`, 100,000 deals,
  seed base 1795214506, declared before the data): 27,472 deals part,
  seat 1 wins 24.63% against 24.98%, +0.132 ± 0.014 points a deal, 95% CI
  [+0.105, +0.159], a win.
- The desktop client's opponents are `ExpectBot`s gated at 10 cards
  instead of 128-sample `MonteCarloBot`s: a decision at the gate takes
  some 14 ms instead of 0.9 s.  Batch 4 of `doc/expectation.md`, head to
  head in seat 1 against the gated Monte Carlo, the others `HeuristicBot`
  (control `e123ba4`, `RESHUFFLE=0`, 100,000 deals declared before the
  data, seed base 1794990944): +0.047 ± 0.019 points a deal, 95% CI
  [+0.011, +0.084], 4,336 deals part, a win.
- The `census` and `outlook` examples play their rounds on every core
  (`examples/common/mod.rs`), 11× and 8× faster on 16 threads, and print
  the same output as before at the same seed: each deals the same piles
  with seat 1 dealing, which only rotates the play.  `cmp` against the
  serial binaries: `census 300 1795000000`, `outlook 200 7 100`,
  `outlook 0 7 100`.  `nodes` stays serial, since it times its graphs.
- The tests build at `opt-level = 2` (also in `web/`), and the long
  playouts run on threads: `cargo test --all-features` takes 11 s
  instead of 5.5 minutes, `web/`'s tests 3 s instead of 37 s.  The bot
  playouts are now eight games of 25 rounds (seeds 2026–2033) instead of
  one game of 200.
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
  batch 3): groups A and B of batch 2.  A pair eat whose ceiling still
  reaches the threshold stands.  `ab/run.sh 800000` against `5b62dc6`
  (seed base 1794947206, `RESHUFFLE=8`), 59,885 deals part, all of them
  the control eating a pair: +0.053 ± 0.002 points a deal, 95% CI
  [+0.049, +0.056], a win.  B gives +0.0353 [+0.0323, +0.0383], A
  +0.0173 [+0.0162, +0.0184].

### Removed

- `research/hand-layout-and-general-marks.md`: its question is closed and
  its account of the code is out of date.  The web hand has been in card
  order, without melds set apart, since `293463d`, and the generals carry
  the coin it found on the real cards (`web/style.css`).
- `doc/hand-planning.md`, added and removed within this cycle: rules of
  thumb for a person, which is the book's ground, and the book dropped
  that chapter.  Its arithmetic and statistics are those of
  `research/seven-slots.md`.

### Fixed

- At a table of two or three the web's hint and the desktop's opponents
  broke concealed triplets from the deal on: `ExpectBot::shallow` read a row
  of `Eval` no label had reached.  The labels come from rounds of four,
  which start with 45 public cards to come, and the table ran to 46, its
  last row still as initialised; a sweep longer than that held it for every
  leaf, from 26 draws left (the order turned at 28 in the hand traced, see
  `doc/eval.md`).  `eval::CARDS` is now 45 and the unlearnt row is cut from
  `src/eval-16.bin`; while more public cards are to come than that, the bot
  plays past its gate as `HeuristicBot`, and `ExpectBot::searches` takes the
  `View` to say so.  Holding the last learnt row instead, or sweeping only
  the table's length, each lost to `HeuristicBot` past the gate in trials at
  two (−2.8 ± 0.3 and −2.1 ± 0.3 points a deal, 400 deals, not counted).
  A/B against `3ddb8f3`, `RESHUFFLE=8`, the control playing the shallow
  graph as shipped.  At four, play is unchanged: 0 of 20,000 deals part on
  the web's settings (seed base 1795094000) and 0 of 2,000 on the desktop's
  (1795114000).  On the web's, at two +11.342 ± 0.157 points a deal, 95% CI
  [+11.035, +11.649], 2,000 deals, seed base 1795124000, win; at three
  +11.099 ± 0.230, [+10.648, +11.550], 2,000 deals, 1795134000, win.  On the
  desktop's, at two +10.367 ± 0.296, [+9.786, +10.948], 500 deals,
  1795144000, win; at three +10.554 ± 0.431, [+9.709, +11.398], 500 deals,
  1795154000, win.  `ab/` takes `PLAYERS` and `BASELINE_SHALLOW`.

- The web test `waits_foretell_the_win` checked stale waits when the round
  ended on the draw that the human's discard brought on.

### Added

- `expect`: the table model of `doc/expectation.md`, batch 2, measured
  from `HeuristicBot` self-play by `examples/outlook.rs` (10,000 rounds,
  seed 20; `research/outlook.md`).  The chance another seat ends the
  round and the public cards a draw both move a lot with the pile (h
  0.018 to 0.12 a card, cards 3.2 to 1.3 a draw), so `Outlook` holds both
  as tables by draws left, and `Graph::values` takes the draws left
  rather than the cards to come.  `Outlook::default()` is the measured
  model: e 0.277, L −12.3, a general the seat's own draw 0.216.
  Calibrated on 10,000 fresh rounds (seed 21, 22,853 decisions leaving 10
  cards or fewer), the predicted chance to win runs high in nine of ten
  bins, 0.382 against 0.291 won, but rises with the wins; changing one
  constant at a time (seed 23), h × 1.5 takes most of it, 0.306 against
  0.290 with the four lowest bins inside their 95% intervals, while e and
  the cards barely move it.  The bias has one direction and one constant
  explains it, so the gate for batch 3 passes.  No bot uses it:
  `ab/run.sh 200000` against `c0aea83` (seed base 1794984588), 0 deals
  part.
- `expect`: the single-player DP of `doc/expectation.md`, batch 1.
  `Graph::new` lays out every (hand, exposed 胡) a chain of claims reaches
  from the roots of one decision, pruned by the ceiling and optionally by
  discard (`Prune`, `Discards`); `Graph::values` sweeps it once per public
  card to come under an `Outlook` of the other seats, in points or as the
  chance to win (`Payoff`).  Tested against a brute-force expansion of the
  recurrence that judges claims by the round's own rules.  No bot uses it
  yet.  `examples/nodes.rs` now builds it at every discard of
  `HeuristicBot` self-play (200 rounds, seed 2): a decision left with 10
  cards builds in 13.0 ms and sweeps in 0.9 ms, within the 50 ms gate
  declared before; 12 cards take 52 ms and 13 take 132 ms.  Pruning the
  discards changes the best root in 1% to 8% of decisions at about 0.02
  points each, so the default prunes none (`research/dp-nodes.md`).
  `ab/run.sh 200000` against `7d3157a` (seed base 1794971744): 0 deals
  part.
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

- `examples/nodes.rs` takes a pruning level and counts the same graph when
  a node discards only loose cards: 1 the cards loose in one of
  `HeuristicBot`'s two layouts, 2 only those no other loose card goes
  with, 3 as 2 while a card that finishes the two is still live.  A fresh
  hand of twenty falls from 7.0 × 10⁵ states to 1.0 × 10⁵, 3.4 × 10⁴ and
  3.5 × 10⁴ (22 s to 5.2 s, 1.2 s and 1.3 s), still too slow; thirteen
  cards fall from 114 ms to 30 ms and sixteen cost 74 ms, so the pruning
  widens the gate rather than removing it.  Leaving hands of N cards or
  fewer unpruned doubles the cost at N = 7 and quadruples it at N = 10.
  Sizes only: what the pruning costs in value waits for the core of
  `doc/expectation.md` (`research/dp-nodes.md`).  No bot changes.

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
  95% CI [−2.189, −2.156], a loss.  By the groups declared beforehand, C
  (ceiling still reaches the threshold) −2.213 [−2.229, −2.196]; B +0.0254
  [+0.0228, +0.0279]; A +0.0143 [+0.0133, +0.0154].  Since A and B lie
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

- `HeuristicBot::shanten`: how many claims a hand is from
  waiting once the seat has answered, searched up to a limit, since a count
  of melds and 搭子 cannot see the 10 胡 a win needs.  A hand that eating
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
  炮 they make nothing, but with soldiers of other colours they make 三色兵卒.
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
  at its 8 胡 ponged the 炮 and went from eight waiting cards to three.
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

- `Rules::eight`: 8 胡, the same table with eight to win (and eight for a
  相公), at the same four seats and deal.
- `web/`: a 設定 dialog (or the `s` key) to choose 10 胡 or 8 胡.  Applying it
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
- 相公: a seat left with nothing to discard ends the round and pays 10 胡
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
