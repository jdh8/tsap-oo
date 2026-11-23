# Public 十胡 / Four Color Cards implementations: feasibility as opponents for tsap-oo

Research checked 2026-11-15.  The question: is there a ready-made computer
player of 十胡 or Four Color Cards on the net that can play against this
engine's `HeuristicBot`?  This document checks which implementations can be
obtained, their strategy entry points and the cost of plugging them in; it
ran no third-party program and measured no candidate's win rate.
The local base for comparison is `62b04efa2b96ae2e22df1c12645562de79de3f20`.

## Answer

**There is public code for computer players, but this time no complete
opponent that plugs straight into tsap-oo was confirmed.**
The main obstacles are differences in rules, the lack of a common bot
protocol, and some projects stating no source licence.  "A public GitHub
repo", "has an AI" and "can be ported directly and evaluated fairly" are
to be judged separately.

- **Technically, evaluate `chunhua1972/four-color-game` first.**  It splits
  the strategy out as a pure TypeScript
  `chooseAiCommand(observation, difficulty, randomIndex)`, and the
  observation already holds its own hand, the public information and the
  legal actions, the closest to this project's `Strategy`.  But no stated
  licence has been found so far, and its rules on pairs of soldiers, the
  kong of generals (將帥) and forced actions differ.
  [strategy][ch-ai], [rules][ch-preset], [tree][ch-tree]
- **If a clear permissive licence is required, `binhtruong-sj/tu-sac` is
  worth studying, but should be seen as a candidate for extracting or
  repairing a strategy.**  It has an MIT licence, a discard grade in Julia
  and an older eat-and-pong strategy; the `tsGUI.jl` loaded by default,
  however, has empty bot functions, and the AI names in the repo cannot be
  taken as evidence that it "can already play a whole game automatically".
  [licence][binh-license], [main program][binh-main],
  [old program][binh-old]
- **`canhcutcon/tu_sac_terminal` suits as a supplement with a different
  heuristic.**  The Python Medium bot decides by the fewest loose cards and
  the count of cards already public, and has entry points for eating and
  for discarding; but the original game's winning condition is "no loose
  cards", without this engine's 10-胡 threshold, and no stated licence was
  found.
  [bot][canh-bot], [rules implementation][canh-rules],
  [project settings][canh-project]
- **`Zizuixixiang/cedarduet` offers another clear weak baseline.**  The
  Python `choose_local_npc_action` does have source for a strategy and its
  tests, but it is under PolyForm Noncommercial 1.0.0, a source-available
  candidate with a non-commercial restriction, and cannot serve as a
  permissively licensed dependency that an Apache-2.0 project takes in
  directly.  [strategy][cedar-ai], [tests][cedar-tests],
  [licence][cedar-license]
- **Do not list multiplayer games as ready-made opponents by mistake.**
  `Huno705/tu-sac` is a two-player networked game, and
  `hongde88/four-color-cards-game` needs four players; the READMEs and
  main game code inspected offer no computer strategy that could be
  extracted.  They can serve for comparing rules or UI, and rank lower for
  this need.
  [Huno README][huno-readme], [Huno engine][huno-engine],
  [hongde README][hongde-readme], [hongde room implementation][hongde-room]

This is **a ranking by feasibility of integration, not by playing
strength**; no data shows any candidate to be stronger than the present
`HeuristicBot`.  The [local heuristic](../src/heuristic.rs) already
estimates the 胡 short of the threshold, loose cards, 搭子 and unseen
cards, and another project calling its bot "AI / Medium / Hard" is no
ground for thinking it better suited as a strong opponent.

## The yardstick: what this engine needs

This project plays Taiwanese 十胡: four players, 112 cards, 21 to the
dealer and 20 to the others, a threshold of 10 胡, six dead cards; a
public draw does not go into the concealed hand.  Wins and kongs are forced
by `Round`, and a strategy only answers eat / pong / pass and the discard;
generals may not be discarded, the card just eaten is locked, a concealed
triplet may not be split to eat, and an eat must leave a legal discard.
Ordinary pairs include pairs of soldiers; each general is a meld by itself
and is never a pair or ponged.
[README](../README.md), [rule values](../src/rules.rs),
[round logic](../src/round.rs)

The minimal interface of an opponent is:

```rust
fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim;
fn discard(&mut self, view: &View<'_>) -> Card;
```

`View` exposes only the seat's own hand, each seat's exposed melds and hand
size, the discards, the current public card, the cards left and the rules;
it does not expose the other seats' hands or the order of the pile.
`Table::step` sends the answer back to `Round` to validate, and an illegal
action returns an error.  The existing arena is four `HeuristicBot`s in one
Rust process, and no cross-language, standard input/output or network bot
protocol has been implemented.
[Strategy](../src/strategy.rs), [View](../src/view.rs),
[Table](../src/driver.rs), [arena](../examples/arena.rs)

`ClaimOptions::blocked` describes the eats that are blocked, and cannot be
output as a legal action; when `forced()` is true a pass is not allowed.
The list of discards should come from
`discardable(view.hand(), view.locked())`, which excludes generals and every
copy of the locked card.  `View` has at present no stream of the table's
events, history of passes or running score; an external strategy that needs
these cannot claim to keep its original behaviour with a mere conversion of
snapshot format.
[ClaimOptions / discardable](../src/round.rs), [View](../src/view.rs)

So what can be reused is the strategy that "**chooses among the legal
options this engine gives**".  Two game engines that each deal, judge wins
and settle cannot directly produce a meaningful head-to-head win rate.
Every "fit / cost / priority" below is an engineering judgement made from
the source code, not a promise by any author.

## The core candidates at a glance

| Project | Computer strategy confirmed | Integration verdict | Licence found | Version checked / date of last commit |
| --- | --- | --- | --- | --- |
| [chunhua1972/four-color-game][ch-tree] | Pure TS `chooseAiCommand`; Easy is random, Normal evaluates the hand's potential and the exposed 胡 | **First technical candidate**; already draws the line between observation and legal actions | No LICENSE found; API license is null | [`ee33cf0`][ch-commit], 2026-10-06 |
| [canhcutcon/tu_sac_terminal][canh-tree] | Python `Bot.decide`, `advise_discard`, `advise_claim` | **Supplementary baseline candidate**; the strategy is easy to pick out, but the original rules authority and whatever does not fit 十胡 have to be removed | No LICENSE found; `pyproject.toml` lists no licence | [`83df231`][canh-commit], 2026-11-13 |
| [binhtruong-sj/tu-sac][binh-tree] | Julia `play1Card`, the older `gpHandlePlay1Card` / `gpHandleMatch*` | **Licensable candidate for strategy research**; stubs on the main path, state coupled to the GUI / network | [MIT][binh-license] | [`695ee45`][binh-commit], 2025-12-02 |
| [Zizuixixiang/cedarduet][cedar-tree] | Python `choose_local_npc_action`; prefers a win, the larger eaten meld, and discarding cards with few duplicates | **Weak baseline candidate for non-commercial evaluation**; the input can be separated, but the BGA rules need adapting | [PolyForm Noncommercial 1.0.0][cedar-license] | [`31e0bfe`][cedar-commit], 2026-11-14 |
| [leadurway/four-color-cards][lead-tree] | `runComputerTurn`, `executeComputerDiscard` inside React | **Low priority**; has a bot, but the traditional mode's strategy is simple, the rules differ greatly, and it is mixed with the UI | No LICENSE found; API license is null | [`940dd96`][lead-commit], 2026-08-28 |
| [tw0961177543-cpu/sisepai][sise-tree] | `aiAnswer`, `pickDiscard`, `aiWantsPeng`, `aiChooseChi` in a single HTML file | **Low priority**; hand heuristics and random strategy, and the original game differs greatly | No licence file found in the root / the tree checked | [`eac82fc`][sise-commit], 2026-11-08 |
| [Huno705/tu-sac][huno-tree] | No bot seen in the README / main engine read | Two humans over a network; not a ready-made opponent | No LICENSE found | [`e26b087`][huno-commit], 2026-07-19 |
| [hongde88/four-color-cards-game][hongde-tree] | No bot seen in the README / room and player models read | Needs four players; not a ready-made opponent | No LICENSE found | [`eaa7d5a`][hongde-commit]; last push by the API 2021-06-06 |

The dates in the table are the sources' records of activity, and do not
mean that anyone maintains them now or that they start on a local machine.
"No licence found" means that the root, the README / package settings and
the GitHub metadata inspected this time state no licence; that is no ground
for taking one in, or distributing its source, as if it were an MIT or
Apache project.  The dates and the licence metadata can be rechecked
through each repo's API: [chunhua][ch-api], [canhcutcon][canh-api],
[binh][binh-api], [leadurway][lead-api], [Huno][huno-api],
[hongde][hongde-api]; the code cited is pinned to the commits checked.
The difference between a public repo and a licence is also in
[GitHub's official documentation][github-license-doc].

## The candidates in detail

### chunhua1972/four-color-game: the smallest surface for plugging in a strategy

The README of `packages/game-core` explicitly separates the core from
React, Supabase and Node.  The actual `AiObservation` holds `ownHand`,
`public`, `seat` and `legalOptions`; `chooseAiCommand` returns one candidate
action directly, with the random number injected as a parameter.  When
discarding, Normal compares `scoreHandPotential` after removing each kind
of card, and when eating or ponging it compares the potential and the
exposed 胡 of the best discard after the eat; Easy samples from the
options.  This is not a searching or a trained model.
[README][ch-readme], [types][ch-types], [strategy][ch-ai]

The minimal hookup turns `View` and the legal actions this engine generates
into its observation, and lets it only choose.  Avoid porting its
`enumerateLegalClaims`, `evaluateHu` or the whole Supabase game along with
it.  The original project uses 112 physical card IDs, this project 28 kinds
of card and their counts; the adapter must assign the copy IDs of cards of
one kind stably, and map the physical ID returned back to the local kind.
[types][ch-types], [strategy][ch-ai], [card encoding][ch-catalog],
[local View](../src/view.rs)

Differences that certainly have to be handled:

- `allowSoldierPair: false`, whereas this engine allows pairs of soldiers;
  the external strategy's hand grade also gives a pair of soldiers none of
  the score of an ordinary pair.
- `allowPassHu: true`, `mandatoryKong: false`; this engine has already
  carried out the forced win / kong before the strategy is called, and
  these two choices should not be handed to the external bot again.
- A general kong is 6 concealed / 8 exposed, in this engine 8 concealed / 6
  exposed; it has 點花, this engine does not.
- Its enumeration of eats can form a meld with part of three of a kind in
  hand, whereas this engine forbids splitting a concealed triplet to eat;
  the external enumeration cannot be taken as this engine's legal options.
- Even when given the local legal options, the strategy excludes only
  generals when it estimates the best discard after an eat, without the
  local lock on every copy of the card.  So it can keep its output legal
  and still overvalue some eats; if this evaluation is corrected, the
  result has to be labelled an adapted strategy.

Sources: [preset][ch-preset], [melds / values][ch-patterns],
[enumeration of eats][ch-claims], [strategy][ch-ai],
[local rules](../src/rules.rs), [local round](../src/round.rs).

The author committed tests and a simulation report, and the README claims
they cover verification of legal actions, conservation and so on; this time
the source and the descriptions were only checked statically, nothing was
rerun, and they were not used to prove playing strength.  Once a usable
licence is obtained, this is the candidate best suited to a small adapter
first.  [README][ch-readme]

### canhcutcon/tu_sac_terminal: an explainable loose-card strategy

Medium's `advise_discard` compares, for each legal discard, the number of
loose cards left, and breaks ties by `usefulness`: the more partner cards
it holds and the more partner cards not yet public, the more it leans to
keeping the card.  `advise_claim` puts a win / a forced action first, then
chooses the combination with the fewest loose cards and the most cards
taken, and eats voluntarily only when that reduces the loose cards.  Easy
samples from the loose cards or the legal actions.  This offers a style of
baseline different from estimating 十胡 scores.  [bots.py][canh-bot]

Its README says the bot looks only at its own and public information, but
`Bot.decide` in fact accepts the whole `Engine`, not an observation
restricted by type.  The evaluation code read uses the seat's own hand and
`public_seen`; if it is plugged in, a restricted input should still be
built anew, and the other hands must not be passed over.
The original rules' `is_win` checks only that the loose cards are zero, and
`score_round` adds one win to the winner, with no 10-胡 scoring; the
default `training_basic` says outright that it is a practice rule set,
which allows splitting a Khạp, has no forced eat into a pair, and is an
exhausted round only when the pile is used up, and the house-rule profile
is not verified Taiwanese 十胡 either.
[README][canh-readme], [bot][canh-bot], [rules][canh-rules],
[training profile][canh-training], [house-rule profile][canh-family]

The minimal hookup should extract the loose-card analysis / the ordering of
actions and let `Round` decide the set of what can be eaten and discarded.
If a 胡 target is put back in for the 10-胡 threshold, it should be
labelled "a rewritten strategy", and the original project's name can no
longer be used to suggest an unmodified original opponent.  A stated
licence is still to be resolved.

### binhtruong-sj/tu-sac: MIT, but not a ready-made headless bot

This is the project among this round's core candidates with a clear MIT
LICENSE.  `TuSacManager.play1Card(player)` in `tsGUI.jl` has branches for
random play, sampling by card count, grading by cards seen and weight
tables, and `processList!` considers 搭子, the counts of public cards and
dead-card marks.  The older `tsGUI1.jl` also keeps entry points for
discarding and for matching.  These are actual strategy code, and not only
a README's claim of AI.  [LICENSE][binh-license],
[main strategy][binh-discard], [old strategy][binh-old-ai]

But the files and the start-up path need to be told apart:

- `runTUSAC.jl` loads `tsGUI.jl` at the end.  Before loading it may ask a
  hard-coded server for the version and overwrite the local `tsGUI.jl`;
  this is not a stable, reproducible entry point for a benchmark.
  [launcher][binh-launcher]
- In the `tsGUI.jl` checked, `TuSacManager.Match2Card` and `Match1or2Card`
  are `return []`, and so are the global `gpHandlePlay1Card`,
  `gpHandleMatch2Card` and `gpHandleMatch1or2Card`.  `hgamePlay` calls the
  latter, so that `play1Card` has a body is no ground for inferring that
  the default path is wired up.  [matching stubs][binh-match-stub],
  [global entry points and calls][binh-entry]
- `server.jl` is a server that sends out updated programs;
  `runTUSAC_server.jl` only copies the main file and has GameZero load it,
  and is not another, clean bot service.
  [server][binh-server], [server launcher][binh-server-launcher]
- `gpHandlePlay1Card` / `gpHandleMatch*` in `tsGUI1.jl` have real branches,
  but it is another single-file GUI program, not a standalone library; its
  parsing, dependencies and a complete game were not verified this time.
  [old version][binh-old-ai]

Its Tứ Sắc rules judge a win by the loose cards reaching zero, and have
further rules such as not adding loose cards and not splitting a Khạp; it
cannot be taken as an engine of the same rules as Taiwanese 十胡.  The
interface can also reach global state such as `all_hands` and `gameDeck`,
so extracting a strategy means confirming, one dependency at a time, that
each can be rebuilt from the public information of `View`.
[rule notes][binh-rules], [win check][binh-win], [main program][binh-main]

The advice is to pick first, at a pinned commit, one discard strategy that
actually has a body as a baseline, then to add the eat-and-pong choice;
whatever has to be written in must be recorded in the name and the
evaluation report.  If the original's whole behaviour must be kept, confirm
first that it runs standalone; for now it cannot be claimed that a
comparable original bot is at hand.

### Zizuixixiang/cedarduet: a clear Python NPC, under a restricted licence

`BgaTenHu.choose_local_npc_action(state, actor, participants)` first looks
for winning options of at least 10 胡 and chooses the one with the most
胡; otherwise among eats, pongs and kongs it first chooses the combination
that uses the most cards from the hand, then compares the combinations' 胡;
for the discard it prefers the card with the fewest duplicates in its own
hand, breaking ties by card ID.
`_options` generates the options from the seat's own hand and the public
state, and the project also has projections of the public / private state.
This is an understandable weak baseline that may be easy to extract, with
no guarantee of strength.  [strategy][cedar-ai]

The original project comes with tests such as "changing the other seats'
hands and the pile should not affect the NPC", "legality and conservation
of a complete NPC game" and "rooms mixing humans and NPCs".  What was
confirmed this time is that the test code exists; the tests were not run,
and nothing proves this bot stronger than `HeuristicBot`.
[tests][cedar-tests]

Its documents and code follow BGA's Taiwanese rules: no pairs of soldiers,
a general kong 6 concealed / 8 exposed; in the order of responses kong and
pong rank equal, a win / a kong may be passed, the pile is used until it is
empty, and the settlement formula also includes 點花.  None of these is
quite the same as this engine, and neither its own enumeration of legal
actions nor its scores can take the place of `Round`.
[rules][cedar-rules], [game implementation][cedar-ai],
[rules document][cedar-doc]

Its LICENSE is explicitly PolyForm Noncommercial 1.0.0.  If the use fits
that licence, it can be listed as an independent candidate for evaluation;
for general reuse / distribution it cannot be put on a par with the MIT
candidate.  Only the content of the source's licence is recorded here, and
no legal conclusion is offered on any particular commercial integration.
[LICENSE][cedar-license]

### leadurway/four-color-cards: has an AI, but the traditional mode is of limited value

This repo does both pair-catching and the traditional eat-and-pong game.
The hard AI of the 10 / 15-card pair-catching mode cannot be taken as a
十胡 opponent: in `runComputerTurn` the traditional mode wins if it can,
kongs if it can and pongs with a 75% chance; `executeComputerDiscard`
prefers to discard the first single card that is not a general, and
otherwise the first card that is not a general.  The strategy is tied into
React state, timers and the log UI, with no standalone observation
function.  [README][lead-readme], [response strategy][lead-claim],
[discard strategy][lead-discard]

The original program has only two seats, the player and the computer; the
traditional mode takes a draw into the hand and draws a replacement after a
kong.  Its `solveHu` does not limit melds of different colours to soldiers
and scores 車馬炮 2 胡, which does not agree with this engine's public
draw, no replacement draw after a kong, or its definitions of 胡 and melds.
Extracting the strategy may be more engineering than its simple strategy is
worth as an evaluation, and no stated licence was found.  [App][lead-app],
[melds and win check][lead-utils], [guide][lead-guide]

### tw0961177543-cpu/sisepai: a hand heuristic, costlier to adapt to the rules

This is a single-file web page for family use; `index.html` has
`usefulness`, `pickDiscard`, `aiKeep`, `aiWantsPeng`, `aiChooseChi` and
`aiAnswer`, which decide by the links among the seat's own cards and by
`Math.random`.  [README][sise-readme], [strategy][sise-ai]

The default is four players and 20 cards, but with `openDraw: false`,
`pairs: false`, `kaiAdd: true`, `minPts: 0`; 將士象 scores 1, 卒 of four
colours 4 and four of a kind 6, and `pickDiscard` itself does not exclude
generals.  These differences from the 10-胡 threshold, the face-up draw,
no adding to a pong and `discardable` mean that it cannot become a legal
opponent just by being wrapped in a JSON shell.  No licence file was found
this time, and its technical priority is below chunhua's.
[default rules and AI][sise-ai], [repo tree][sise-tree]

### Huno705 and hongde88: a playable multiplayer game is not a computer opponent

Huno's README describes the flow as two people creating a room, joining and
playing, and the main engine is `applyAction`; no entry point for a bot to
choose an action was found in the main files read.  Its two-player rules,
an exhausted round at seven cards, a win on having no loose cards, 將士象 at
1 and soldier melds at 1 / 4 also mean that its results and this engine's
cannot be taken for each other directly.  [README][huno-readme],
[engine][huno-engine]

hongde's README states that four players are needed; `Room` creates rooms
and deals for Socket.IO players, and offers no strategy for a computer
seat.  The UI and the connection flow can be read, but without a bot
written separately it cannot be an automatic opponent in the arena.
[README][hongde-readme], [Room][hongde-room]

## Other public leads and exclusions

| Source | What was found | Why it is not listed as an opponent that can be plugged in |
| --- | --- | --- |
| [Board Game Arena Four Color Cards][bga] | An official online game for 2–4 players, developer xnedkx; has a public help page | No callable bot interface or complete server / bot source was obtained; a public client and a playable site are not a public AI |
| [Lapira Tứ Sắc][lapira] | The author's Google Play page explicitly mentions offline play and virtual opponents | A product with computer opponents is confirmed, but no source or programmatic interface was found |
| [Vinagames Tứ Sắc][vina] | The author's site offers an online multiplayer game and how to play | No bot interface that this engine could call was found |
| [添丁四色牌][itch] | The author published a Unity HTML5 mini game | The public page is not enough to confirm the same rules as 十胡, an AI entry point or reusable source |
| [chengr4/four-color-cards][cheng] | `03a1430`: a README TODO, `go.mod`, no game code | A skeleton, not an engine or a bot |
| [god00612/FourColorCardGame][god] | `bd6c9a2`: the README says outright that it is a tool for tallying money | A scoring tool, not a strategy for play |
| [minhhoa11111999/tu-sac-app][minh] | `e89e66b`: HTML scoring, login and history | No game strategy seen |
| [timmykhoi1813-png/Tu-sac-online][timmy] | `afc02f9`: a Socket.IO multiplayer demo; `hasWin` judges a win by a hand of no more than four cards, all of one colour | No bot seen, and the winning condition does not match 十胡 |
| [binhtruong-sj/bettaTuSac][betta] | `48c20b3`: a support page and a privacy page | No strategy code |
| [TuSac/MapMahJongg][map] | `87647f9`: README, LICENSE, `.gitignore` | Even with a GPL-3.0 licence, the repo checked still has no bot implementation |
| [sampot/pg-sisek][sampot] | MIT-licensed, a five-card pair-matching scoring game played with the Four Color Cards deck | A different goal; not the 十胡 / Tứ Sắc opponent sought this time |

BGA can serve as a source for comparing rules, but its Chinese help forbids
pairs of soldiers, scores a general kong 6 concealed / 8 exposed, and has
點花; the local `Rules` has only the threshold, the dead cards and the
general meld values, with no kong score for each rank and no switch for
pairs of soldiers, so it cannot be claimed that changing the numbers in
`Rules` alone lines it up fully with BGA.
[BGA rules][bga-help], [local Rules](../src/rules.rs)

## A minimal integration and verification plan

What follows is the suggested next step; **nothing was implemented this
time**.

1. **First choose one candidate and an exact version.**  Technical
   validation can start with chunhua's Normal, but the licence has to be
   cleared up before its source is reused directly; if limited to MIT,
   first spot-check one complete strategy path of binh, and do not assume
   from the default launcher onwards that it works.
2. **Hook up only the `Strategy` boundary.**  Either a port into the same
   language or a resident child process will do.  With a child process,
   simple line-by-line JSON requests / replies are enough; that is an
   adapter to be added, not a protocol this project or any candidate
   already supports.  A request carries only the data `View` shows and the
   IDs of the legal actions, and the other side replies with one ID.
3. **Keep `Round` the only rules authority.**  Forced wins, forced kongs,
   generals that must be eaten, locked cards, concealed triplets, the 相公
   restriction and the 10-胡 threshold all go by the options this engine
   gives.  When extracting an original strategy, its original grade may be
   kept at first to observe the gap in rules, with an adjusted version
   listed separately; do not pool the results of the two.
4. **Verify the adaptation first, then compare playing strength.**  Use
   fixed positions to confirm the mapping of kinds / copies, the sources of
   eats and draws, the forced eat of a general, locked cards, and the
   ending with no legal discard.  Confirm with the same visible information
   and different hidden cards that the bot is not affected by hidden cards.
   Illegal outputs, timeouts and fallbacks are to be counted; a card must
   not be quietly chosen for it and still counted in the original
   strategy's win rate.
5. **Use the existing arena for four-player experiments.**  First one seat
   of the candidate against three of `HeuristicBot`, rotating through every
   seat and dealer, with fixed, replayable card orders and several seeds;
   also a mixed set-up in which the two strategies take two seats each and
   still compete each for itself.  Record the win rate, the net 胡 a round,
   the rates of exhausted rounds / 相公, the rate of illegal actions and
   the decision time.
   Report sample sizes and uncertainty, and draw no conclusion on playing
   strength from one set of seeds or from the winners' mean 胡 alone.

The input and legality requirements above come from the local
[Strategy](../src/strategy.rs), [View](../src/view.rs),
[Round](../src/round.rs), [Table](../src/driver.rs); the arena's extension
point is [where the four seats' bots are built](../examples/arena.rs).

## Search scope and caveats

Starting from the candidates of an earlier survey of how other
implementations lay out a hand (since removed; `git show
09a35a3:research/hand-layout-and-general-marks.md`), the sources were
checked again; in addition `十胡`, `四色牌`,
`four-color-cards`, `Four Color Cards`, `Si Se Pai`, `Tứ Sắc`, `Tu Sac` and
`tusac` were searched for, using search engines and GitHub repository
search to find repos, then reading the authors' READMEs, code, licence
files and commit / repository metadata.  A search snippet or a GitHub
project name was not taken as sufficient evidence that an AI exists.

Search engines were also used to check GitLab, Gitee and SourceForge, and
no further automatic opponent that the sources could confirm was found; the
Four Color Cards material found on SourceForge cannot count as a strategy.
UNO / four-colour poker-deck colour schemes, the four colour theorem / map
colouring and mahjong programs with similar names were excluded.

This was a static check of source code: no candidate's dependencies were
installed, no candidate's tests run, no third-party game server connected
to, no commercial program reverse-engineered and no playing strength
measured.  That nothing "plug-and-play" was found does not mean that no
other implementation exists on the net at all; unpublished or unindexed
projects, and old games that exist only as executables, may be missing
from these results.  "No bot / no licence" in this document should be read
as "not found within what was checked", not as an assertion about an
author's unpublished code or the state of their rights.

[ch-tree]: https://github.com/chunhua1972/four-color-game/tree/ee33cf080e3a3420bddc7d3cf1236d13b418dd71
[ch-commit]: https://github.com/chunhua1972/four-color-game/commit/ee33cf080e3a3420bddc7d3cf1236d13b418dd71
[ch-api]: https://api.github.com/repos/chunhua1972/four-color-game
[ch-readme]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/README.md
[ch-ai]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/packages/game-core/src/ai/policy.ts
[ch-types]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/packages/game-core/src/contracts/types.ts
[ch-preset]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/packages/game-core/src/rules/presets.ts
[ch-patterns]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/packages/game-core/src/rules/patterns.ts
[ch-claims]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/packages/game-core/src/rules/claims.ts
[ch-catalog]: https://github.com/chunhua1972/four-color-game/blob/ee33cf080e3a3420bddc7d3cf1236d13b418dd71/packages/game-core/src/tiles/catalog.ts
[canh-tree]: https://github.com/canhcutcon/tu_sac_terminal/tree/83df231d02d25def7ebadbd7beb0e1637b5f09ba
[canh-commit]: https://github.com/canhcutcon/tu_sac_terminal/commit/83df231d02d25def7ebadbd7beb0e1637b5f09ba
[canh-api]: https://api.github.com/repos/canhcutcon/tu_sac_terminal
[canh-readme]: https://github.com/canhcutcon/tu_sac_terminal/blob/83df231d02d25def7ebadbd7beb0e1637b5f09ba/README.md
[canh-bot]: https://github.com/canhcutcon/tu_sac_terminal/blob/83df231d02d25def7ebadbd7beb0e1637b5f09ba/src/tusac/bots.py
[canh-rules]: https://github.com/canhcutcon/tu_sac_terminal/blob/83df231d02d25def7ebadbd7beb0e1637b5f09ba/src/tusac/rules/training.py
[canh-project]: https://github.com/canhcutcon/tu_sac_terminal/blob/83df231d02d25def7ebadbd7beb0e1637b5f09ba/pyproject.toml
[canh-training]: https://github.com/canhcutcon/tu_sac_terminal/blob/83df231d02d25def7ebadbd7beb0e1637b5f09ba/config/rules.training.yaml
[canh-family]: https://github.com/canhcutcon/tu_sac_terminal/blob/83df231d02d25def7ebadbd7beb0e1637b5f09ba/config/rules.family_nhatrang.yaml
[binh-tree]: https://github.com/binhtruong-sj/tu-sac/tree/695ee45f73cbacb5e13572381767e02bf80b15a6
[binh-commit]: https://github.com/binhtruong-sj/tu-sac/commit/695ee45f73cbacb5e13572381767e02bf80b15a6
[binh-api]: https://api.github.com/repos/binhtruong-sj/tu-sac
[binh-license]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/LICENSE
[binh-main]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI.jl
[binh-discard]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI.jl#L2375-L2600
[binh-match-stub]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI.jl#L2928-L2933
[binh-entry]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI.jl#L6761-L6843
[binh-win]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI.jl#L2935-L3074
[binh-old]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI1.jl
[binh-old-ai]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tsGUI1.jl#L4279-L4411
[binh-rules]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/tuSac_rule.txt
[binh-launcher]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/runTUSAC.jl
[binh-server]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/server.jl
[binh-server-launcher]: https://github.com/binhtruong-sj/tu-sac/blob/695ee45f73cbacb5e13572381767e02bf80b15a6/runTUSAC_server.jl
[lead-tree]: https://github.com/leadurway/four-color-cards/tree/940dd968344d74125d5e57e09a3ae75e2d18c405
[lead-commit]: https://github.com/leadurway/four-color-cards/commit/940dd968344d74125d5e57e09a3ae75e2d18c405
[lead-api]: https://api.github.com/repos/leadurway/four-color-cards
[lead-readme]: https://github.com/leadurway/four-color-cards/blob/940dd968344d74125d5e57e09a3ae75e2d18c405/README.md
[lead-guide]: https://github.com/leadurway/four-color-cards/blob/940dd968344d74125d5e57e09a3ae75e2d18c405/GUIDE.md
[lead-utils]: https://github.com/leadurway/four-color-cards/blob/940dd968344d74125d5e57e09a3ae75e2d18c405/src/cardUtils.ts
[lead-app]: https://github.com/leadurway/four-color-cards/blob/940dd968344d74125d5e57e09a3ae75e2d18c405/src/App.tsx
[lead-claim]: https://github.com/leadurway/four-color-cards/blob/940dd968344d74125d5e57e09a3ae75e2d18c405/src/App.tsx#L1508-L1577
[lead-discard]: https://github.com/leadurway/four-color-cards/blob/940dd968344d74125d5e57e09a3ae75e2d18c405/src/App.tsx#L1784-L1867
[huno-tree]: https://github.com/Huno705/tu-sac/tree/e26b08732206d76705164510016729e4f29bbfeb
[huno-commit]: https://github.com/Huno705/tu-sac/commit/e26b08732206d76705164510016729e4f29bbfeb
[huno-api]: https://api.github.com/repos/Huno705/tu-sac
[huno-readme]: https://github.com/Huno705/tu-sac/blob/e26b08732206d76705164510016729e4f29bbfeb/README.md
[huno-engine]: https://github.com/Huno705/tu-sac/blob/e26b08732206d76705164510016729e4f29bbfeb/server/src/engine.ts
[hongde-tree]: https://github.com/hongde88/four-color-cards-game/tree/eaa7d5aecbade61e07f1cd129aa6a3ba13c6a6cc
[hongde-commit]: https://github.com/hongde88/four-color-cards-game/commit/eaa7d5aecbade61e07f1cd129aa6a3ba13c6a6cc
[hongde-api]: https://api.github.com/repos/hongde88/four-color-cards-game
[hongde-readme]: https://github.com/hongde88/four-color-cards-game/blob/eaa7d5aecbade61e07f1cd129aa6a3ba13c6a6cc/README.md
[hongde-room]: https://github.com/hongde88/four-color-cards-game/blob/eaa7d5aecbade61e07f1cd129aa6a3ba13c6a6cc/server/models/room.js
[cedar-tree]: https://github.com/Zizuixixiang/cedarduet/tree/31e0bfe52584cd2fbef6b8d3f0445955586675ff
[cedar-commit]: https://github.com/Zizuixixiang/cedarduet/commit/31e0bfe52584cd2fbef6b8d3f0445955586675ff
[cedar-ai]: https://github.com/Zizuixixiang/cedarduet/blob/31e0bfe52584cd2fbef6b8d3f0445955586675ff/app/games/four_color_cards/tenhu.py
[cedar-rules]: https://github.com/Zizuixixiang/cedarduet/blob/31e0bfe52584cd2fbef6b8d3f0445955586675ff/app/games/four_color_cards/tenhu_rules.py
[cedar-tests]: https://github.com/Zizuixixiang/cedarduet/blob/31e0bfe52584cd2fbef6b8d3f0445955586675ff/tests/test_four_color_tenhu.py
[cedar-doc]: https://github.com/Zizuixixiang/cedarduet/blob/31e0bfe52584cd2fbef6b8d3f0445955586675ff/docs/FOUR_COLOR_CARDS.md
[cedar-license]: https://github.com/Zizuixixiang/cedarduet/blob/31e0bfe52584cd2fbef6b8d3f0445955586675ff/LICENSE
[sise-tree]: https://github.com/tw0961177543-cpu/sisepai/tree/eac82fcb4a3aa1848c55ec7feede703652e1b3c2
[sise-commit]: https://github.com/tw0961177543-cpu/sisepai/commit/eac82fcb4a3aa1848c55ec7feede703652e1b3c2
[sise-ai]: https://github.com/tw0961177543-cpu/sisepai/blob/eac82fcb4a3aa1848c55ec7feede703652e1b3c2/index.html
[sise-readme]: https://github.com/tw0961177543-cpu/sisepai/blob/eac82fcb4a3aa1848c55ec7feede703652e1b3c2/README.md
[github-license-doc]: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository
[bga]: https://en.boardgamearena.com/gamepanel?game=fourcolorcards
[bga-help]: https://zh.doc.boardgamearena.com/Gamehelpfourcolorcards
[lapira]: https://play.google.com/store/apps/details?id=com.lapira.tusac.android
[vina]: https://www.vinagames.com/tusac.php
[itch]: https://cgcgtw.itch.io/cards
[cheng]: https://github.com/chengr4/four-color-cards/tree/03a1430
[god]: https://github.com/god00612/FourColorCardGame/blob/bd6c9a2/README.md
[minh]: https://github.com/minhhoa11111999/tu-sac-app/blob/e89e66b/index.html
[timmy]: https://github.com/timmykhoi1813-png/Tu-sac-online/blob/afc02f9/server.js
[betta]: https://github.com/binhtruong-sj/bettaTuSac/tree/48c20b3
[map]: https://github.com/TuSac/MapMahJongg/tree/87647f9
[sampot]: https://github.com/sampot/pg-sisek
