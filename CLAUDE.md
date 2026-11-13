# tsap-oo

十胡, the Taiwanese Four Color Cards game, on top of the `susik` deck
crate.  Everything about the
cards themselves — `Card`, `Hand`, `Meld`, the layout `Solver` — lives in
susik; this crate owns dealing, the claim priority, scoring, and the bots.
The browser front end is the standalone crate in `web/`.

After updating the codebase, please

- Format the code with `cargo fmt` (also in `web/`).
- Lint with `cargo clippy --all-targets --all-features -- -D warnings`.
- Run the tests with `cargo test --all-features`, then `cargo check
  --no-default-features`, then `cd web && cargo test`.
- Build the docs with `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`.
- Note user-visible changes in `CHANGELOG.md` under `[Unreleased]`.
- Propose a clear and descriptive commit message.

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
