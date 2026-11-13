# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
