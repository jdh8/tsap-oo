# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Rules`: the Taiwanese 十胡 table.
- `Round`: the runtime-checked state machine — dealing, face-up draws,
  claims resolved as win > forced kong > pong > eat, locked triplets and
  locked claimed cards, dead cards, 天胡.
- `Game`: the scoreboard and dealer rotation.
- `View`, `Strategy`, `Table`, `HeuristicBot`: the bot framework and a greedy
  bot.
- `web/`: the browser front end, one human against three bots.  Your hand
  is arranged into its best melds automatically, loose cards last.  A `?`
  button (or the `?` key) opens a how-to-play sheet with the 胡 of each meld.
  A 提示 button (or the `h` key) rings the claim or discard a greedy bot
  would make in your place.  The felt is blue so the green suit stands out
  against it.
