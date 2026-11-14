# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Rules::eight`: 八胡, the same table with eight to win (and eight for a
  相公), at the same four seats and deal.
- `web/`: a 設定 dialog (or the `s` key) to choose 十胡 or 八胡.  Applying it
  starts a new game; the choice is kept in the browser.  The title, the
  how-to-play sheet and the 相公 message follow the table in play.

### Changed

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

### Fixed

- `web/`: the 莊 mark beside your own score no longer drops below it.

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
