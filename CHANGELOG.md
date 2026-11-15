# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The `dealt` example: the 胡 already in a dealt hand of twenty cards, the
  melds they come from, and the cards left outside every meld.  No bot
  plays.  Over 100000 deals (seed 20261118) a hand holds 6.63 胡 and 7.28
  loose cards on average.
- The web hint grades every move: each card in your hand and each claim
  button shows how many 胡 it falls behind the bot's pick, and 死 marks a
  move that leaves the hand short of the 胡 to win.  Equal moves show as
  equal.
- `HeuristicBot::claims` and `HeuristicBot::discards` are public and return
  each move with its grade in 胡, best first.  Play is unchanged: on paired
  deals against fd5b9b2 (`ab/run.sh 20000`, seed base 1794786340), 0 of
  20000 deals differ.

### Changed

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
