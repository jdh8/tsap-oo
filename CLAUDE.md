# tsap-oo

十胡, the Taiwanese Four Color Cards game, on top of the `susik` deck
crate.  Everything about the
cards themselves — `Card`, `Hand`, `Meld`, the layout `Solver` — lives in
susik; this crate owns dealing, the claim priority, scoring, and the bots.
The browser front end is the standalone crate in `web/`, and the A/B harness
the one in `ab/`.

After updating the codebase, please

- Format the code with `cargo fmt` (also in `web/`).
- Lint with `cargo clippy --all-targets --all-features -- -D warnings`.
- Run the tests with `cargo test --all-features`, then `cargo check
  --no-default-features`, then `cd web && cargo test`.
- Build the docs with `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`.
- Note user-visible changes in `CHANGELOG.md` under `[Unreleased]`.
- Propose a clear and descriptive commit message.

## Iron rules

Taken from `../pons` (its `AGENTS.md` and `docs/measurement.md`) and cut down
to what a card-game bot needs.

### Measurement

- Never ship a change to how a bot plays on analysis alone.  Run the A/B,
  `ab/run.sh [deals] [control]`, and commit only on a win.  The web hint is
  `HeuristicBot`, so a hint fix is a bot change.
- The control is `main` HEAD's own code, not a knob set back.  `ab/run.sh`
  exports it under another package name and `ab/` steps the two crates'
  rounds in lockstep, so each bot reads its own crate's `View`.  It measures
  `HeuristicBot`; another bot needs its seat edited in `ab/src/main.rs`.
- Pair the deals: the candidate in one seat among three control bots,
  against four control bots on the same deal.  Each deal has its own seed,
  the dealer rotates, and the seat is scored through a fresh `Game::settle`.
  Report the mean difference, its standard error and the 95% CI.  The plain
  `arena` cannot see a gain of this size: a seat's score has a standard
  deviation near 30 a round.
- Take a fresh seed base for each experiment (the script's default) and
  declare the deal count before the data.  Deals a constant was tuned on
  never give its verdict.
- Verdict: `win` when the CI lies above 0, `loss` when it lies below.
  A CI that straddles 0 is insufficient data, never "no loss".
- Before declaring a loss dead, trace the deals that moved the most.
- Never rebuild while a run is in flight.
- Write it down in `CHANGELOG.md` and the commit message: the numbers, the
  deal count, the seed base, the control's SHA and the verdict.
- A change meant to leave play alone (a refactor, a speed-up) owes the same
  harness reading zero deals that differ.

### Workflow

- Commit directly on `main`, and only when asked.

Invariants, each guarded by `tests/round.rs`:

- Every card is in exactly one place: a hand, an exposed meld, the pile,
  the discards, the offer, or the winning card.
- Winning and konging are compulsory and resolved before anyone is asked;
  the claim queue runs in priority order, and an eat stands only if no later
  seat pongs.
- The card just claimed is locked for that turn's discard.
- Melds formed with a public card score as exposed; only dealt melds score
  as concealed.

Rules questions: the Chinese Wikipedia article on 十胡 and the common
Taiwanese write-ups agree on the table in `Rules::new`; where they disagree
(four-colour soldiers 4 vs 5, 將 kong values) the README says which was
chosen.
