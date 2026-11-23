# Prior art: mahjong and rummy AI

Research checked 2026-11-16.  The question: do the four routes listed in
`doc/stronger-bot.md` (as of `3aa5b0c`) have precedents in the literature on
mahjong, gin rummy and other draw-and-discard games?  Which of them carry
over to a 十胡 bot with a hand-written grade and Monte Carlo and no
deep-learning pipeline?  Does 十胡 have written rules for two or three
players?  Is there precedent or evidence for "two players first, then
extend to four"?  This document only reads the literature and public source
code; it played no rounds and changed no bot.

## Answer

- **The first thing worth doing is measurement, not play.**  Today's A/B is
  already the duplicate of the literature (the same deal played once with
  the candidate and once without), and all the noise left is "after the two
  sides' play parts".  Backgammon's rollouts and poker's DIVAT / MIVAT /
  AIVAT deal with exactly this stretch: from where they part, reshuffle the
  pile left many times and average, then take off the "luck" of every draw.
  [GNU Backgammon manual][gnubg-ro], [DIVAT][divat], [MIVAT][mivat],
  [AIVAT][aivat]
- **Before any opponent inference, measure how much a "Monte Carlo that sees
  the other seats' hands" can win.**  The research on Scrabble and on gin
  rummy both measured this ceiling first: if the ceiling is low, the
  inference need not be built.  The conclusion for Dou Dizhu is that a
  perfect opponent model is only a little better than a uniform belief.
  [Richards & Amir 2007][scrabble], [Kelidari et al. 2026][gold],
  [Cowling et al. 2012][ismcts]
- **"Don't feed the next seat" is the use of opponent information in rummy
  games that has competition results behind it.**  MARJJ, the winner of the
  EAAI 2021 gin rummy competition, grades a discard as "the value of its own
  hand" plus bonuses and penalties for "how useful this card is to the
  opponent"; with one group of them set to zero, the win rate against the
  original setting drops to 47% to 48%.
  [Ahmed et al. 2021][gr-marjj], [competition results][gr-results]
- **Mahjong AIs model opponents almost entirely to compute the expected loss
  of dealing in, and that part does not carry over.**  In Japanese mahjong
  whoever deals in pays in full; in 十胡 dealing in costs nothing extra, and
  `doc/stronger-bot.md` has already measured that defence does not pay.  Two
  other things do carry over: dynamic programming for the single-player
  expected value, and replacing the opponents with an environment of "the
  chance each turn that the round ends".  [Mizukami & Tsuruoka 2015][mizukami],
  [Kurita & Hoki 2019][kurita], [tomohxx 和了確率][tomohxx-prob]
- **For the two-player game the sources say "it can be played" and none says
  "how the rules change".**  Chinese Wikipedia writes 2 to 6 players, Board
  Game Arena offers 2 to 4 with ranked play defaulting to two, and the deal
  is always 20 cards each and 21 to the dealer; no source writes down any
  adjustment of the rules at fewer players.  The precedents show that
  algorithms carry from two players to many; play is not guaranteed to.
  Evaluating two-player mahjong still took 200,000 games.
  [Chinese Wikipedia, 十胡][zhwiki-shihu], [BGA help][bga-zh],
  [Liu et al. 2023][olss]
- **No AI paper or thesis on Four Color Cards or 十胡 was found.**  The
  public implementations and bots are in
  [`public-opponents.md`](public-opponents.md).

## A. Mahjong AI

| System | Method | What it uses beyond its own hand | How strength is measured |
| --- | --- | --- | --- |
| [Suphx][suphx] | Five CNNs (discard, riichi, eat, pong, kong), supervised learning then self-play reinforcement learning | Each seat's discards, running scores, riichi; in training also the other seats' hands and the wall | 1,000,000 games offline against three fixed opponents; over 5,000 games on Tenhou |
| [Mizukami & Tsuruoka][mizukami] | Single-player mahjong play + three opponent models + Monte Carlo | Each seat's discards, exposed melds and the turn, fitted to Houou-table (鳳凰卓) game records | 1,000 duplicate games on the same walls; 2,634 games on Tenhou |
| [Kurita & Hoki][kurita] | Abstracts four-player mahjong into four single-player MDPs, the opponents replaced by a stochastic environment | The chances of an opponent winning and of dealing in, taken from game records | 3,557 games, the difference in placing from Bakuuchi within the same game |
| [akochan][akochan] | C++ that computes the expected points of each choice explicitly | Each seat's chance of waiting, each tile's chance of dealing in and the value of the hand | Self-play on the same seed with the first dealer changed |
| [Mortal][mortal] | Deep reinforcement learning | Most pages of its documentation are blank; could not be verified | 1-vs-3 duplicate, the same seed played once from each of the four seats |
| [kanachan][kanachan] | Imitation learning on Mahjong Soul game records plus fine-tuning in stages, no hand-made features | The record sequence of the whole game | Could not be verified |
| [ACH / JueJong][ach] | An actor-critic form of weighted CFR, self-play, aiming at a Nash equilibrium | The opponent's discards and exposed melds | 10,000 games against each rule bot; 1,000 games against a human champion |
| [Botzone, Chinese Official mahjong][oim] | A competition: heuristics, supervised learning, reinforcement learning | Depends on the team | Swiss system plus duplicate, 24 seatings on the same wall |

### Suphx

- **Oracle guiding.**  First train an oracle that sees the other seats'
  hands and the wall, then switch these "perfect features" off step by step
  with a dropout of decaying probability until it is a normal agent.  The
  authors say plain distillation does not work: an agent with less
  information cannot imitate one with more.  [§3.3][suphx]
- **Global reward prediction.**  A GRU predicts the final reward of the whole
  game from the information of each round, and the difference between the
  predictions of two consecutive rounds serves as that round's reward.  This
  is credit assignment from the "game" to the "round"; 十胡 settles every
  round, so it has no use here.  [§3.2][suphx]
- **pMCPA.**  After each round's deal, fix its own hand, deal the other
  seats and the wall at random, play out a number of trajectories with the
  offline policy, then fine-tune the policy on those trajectories.  The
  offline experiment used 100,000 trajectories; the fine-tuned version won
  66% against the one without, measured over only a few hundred games;
  being too slow, the Suphx on Tenhou did not use it.
  [§3.4, §4.3, note 8][suphx]
- **Look-ahead features.**  Over a hundred features come from a depth-first
  search: "discard a certain tile, then replace n more: can the hand win
  with S points?"  The search ignores the opponents and considers only its
  own draws and discards.  This amounts to using route 3 (the single-player
  expected value) as an input to the grade.  [§2][suphx]
- **Evaluation.**  1,000,000 games are generated at random offline, and
  every agent plays this same batch of deals against three fixed weak
  agents; the metric is Tenhou's stable rank, and 800,000 of the 1,000,000
  games are drawn 1,000 times over to see the spread.  The authors state
  that this is to suppress the variance of the deal.  Over 5,000 games of
  live play on Tenhou, stable rank 8.74.  [§4, §5, appendix C][suphx]

Carries over: how the look-ahead features are made.  Does not: the other
three items all need a deep-learning pipeline.

### Mizukami & Tsuruoka and Bakuuchi

- **Three opponent models**, all fitted to Houou-table game records: whether
  an opponent is waiting (logistic regression, 6,888 features, AUC 0.777,
  expert humans 0.778), whether each kind of tile is one it wins on (34
  logistic regressions), and the value of the winning hand (a linear
  regression on the log of the value).  The features are public
  information such as discards, the number of exposed melds and the turn.
  [§IV–VI][mizukami]
- **How they go into the Monte Carlo.**  Each candidate discard scores
  `Sim(tile) × Π(1 − chance of dealing in) − Σ expected loss from dealing in`.
  In the simulation it plays itself by the "single-player mahjong" play; an
  opponent looks at no hand and carries only two flags (waiting, folded),
  wins or declares riichi by probability, and discards whatever it draws.
  Every candidate tile uses the same random numbers to drive the opponents
  and the wall.  [§VII][mizukami]
- **Results.**  Against Mattari Mahjong, the strongest public program, mean
  placing 2.48 ± 0.07 against 2.51, not significant (p = 0.29); against its
  own predecessor, significant (p = 0.01).  On Tenhou 2,634 games at R1718,
  the predecessor 1,441 games at R1689, the difference in placing not
  significant (p = 0.22).  Win rate 0.245 against the predecessor's 0.256,
  deal-in rate 0.131 against 0.148: it dealt in less and won less.
  [§VIII][mizukami]
- **The predecessor was "single-player mahjong".**  First a single player
  that never eats or pongs and only seeks the fastest win, then folding and
  exposed melds added on.  The authors' own review: single-player play
  alone is not strong enough, because it never folds and often deals in.
  [§III, §IX][mizukami]

The core of this line is the trade of "speed" against "the loss from dealing
in".  A win on a discard in Japanese mahjong is paid in full by whoever
dealt in, so the trade has weight; in 十胡 dealing in costs nothing extra,
and the two experiments in `doc/stronger-bot.md` measured the same thing
(fewer deal-ins, fewer wins of its own) as a net loss.  The opponent model
itself (logistic regression on public features) can be copied as it is, but
its use here has to change.

### Kurita & Hoki

- Defines four single-player MDPs.  The main one (Inclusive Policy Solitary
  Mahjong) replaces the other three seats with a stochastic environment:
  each tile it discards has a chance of dealing in and ending the round;
  each turn that passes has a chance of another seat winning and ending it;
  the tiles the other seats discard appear by probability and can be eaten,
  ponged or won on.  The other three MDPs each look only at winning,
  waiting and folding, and the values they compute serve as features.
  [§IV–VI][kurita]
- At a shanten number of 0 to 1 it uses the main MDP's value directly; at 2
  to 3 the features from the other three MDPs; further away, simple rules.
  Each decision takes a few seconds.  [§VII][kurita]
- 3,557 east-only games, one seat against Bakuuchi and two seats of manue:
  mean placing 2.23 ± 0.04 against Bakuuchi's 2.29 ± 0.04.  The authors'
  statistic is the difference in placing between the two within the same
  game, mean 0.0574, standard deviation 1.822, one-sided significance level
  0.03.  [§VII][kurita]
- The authors point out that carrying CFR-like methods over to multiplayer
  mahjong is not easy.  [§III][kurita]

Carries over: this is exactly the shape route 3 needs.  Single-player
dynamic programming plus the chance that "someone wins this turn", and
`research/safe-cards.md` has already measured how that chance moves with an
opponent's count of concealed cards.

### Computing the single-player expected value

- **tomohxx, *麻雀アルゴリズム*.**  The state is (hand, turn).  The draw
  step: each useful kind of tile is weighted by "copies left ÷ tiles left in
  the wall"; the discard step: the maximum over every tile that can be
  discarded.  Computed backwards from the last turn, this is dynamic
  programming on a directed acyclic graph of (hand, turn).  Replace the
  probability with points, and at a win take the larger of "the points of
  the win" and "the expected value of playing on", and the result is
  expected points.  The model has one player, no opponents and no exposed
  melds; the graph may include edges that change the hand's shape and edges
  that go back in shanten.  [和了確率][tomohxx-prob]
- **nekobean/mahjong-cpp** (the question wrote tomohxx/mahjong-cpp; the
  actual repo is under nekobean, and its algorithms cite tomohxx) provides
  the shanten number, useful tiles, the chance of waiting, the chance of
  winning and expected points, and is made into a web 何切るシミュレーター.
  [README][mahjong-cpp]
- **Shanten number.**  The table-lookup method of tomohxx/shanten-number;
  Mortal's shanten number is a Rust port of tomohxx's algorithm.
  [shanten-number][shanten], [Mortal meta][mortal-meta]
- **Li & Yan, *Let's Play Mahjong!*.**  Defines the deficiency (that is, the
  shanten number) and gives the discard strategy that "maximises the chance
  of completing the hand within k tile changes".  The fast algorithms that
  followed take "which tiles can still be had" into account.
  [arXiv 1903.03294][lets-play], [arXiv 2108.06832][deficiency]

### Mortal, akochan, kanachan

- **Mortal's evaluation.**  1-vs-3 duplicate: a set of four games shares one
  seed, the challenger sits East, South, West and North in turn, and the
  other three seats are the control.  The simulator guarantees the same
  wall, deal and dora under the same (seed, round, honba).  Example:
  akochan challenging Mortal "3.1", 13,152 games for the challenger, mean
  placing 2.568 against 2.477.  [strength.md][mortal-strength]
- **akochan's explicit expected value.**  Only the headers were read, not
  the implementation.  `tenpai_est.hpp` estimates for each opponent the
  chance of waiting and, for each kind of tile and each han and fu, the
  chance of it being won on a discard or on its own draw, and enumerates
  the possible waiting hands; `cal_exp` in `exp_values.hpp` combines its own
  chance of winning (the comment states that it comes from the solution of
  single-player mahjong), each seat's chance of waiting, each tile's chance
  and cost of dealing in, and the chance of waiting at an exhausted round
  into one figure of expected points; `betaori.hpp` computes the expected
  value of folding.  Self-play repeats over combinations of seed and first
  dealer, which the README says suits comparing against the old version
  after a change of strategy.
  [tenpai_est.hpp][akochan-tenpai], [exp_values.hpp][akochan-exp],
  [betaori.hpp][akochan-betaori], [readme][akochan]
- **kanachan.**  Deliberately uses no hand-made features and relies on a
  large body of Mahjong Soul game records and a large model.  The author
  says outright that this needs big data and big compute.  Nothing carries
  over to this project.  [README][kanachan]

Carries over: how Mortal and akochan evaluate, which `ab/` already does.
akochan's headers split the expected value into "winning by the
single-player solution", "another seat ending the round", "dealing in" and
"an exhausted round", each with a chance and a value, the same shape as
Kurita and Hoki's MDP.

### Tencent: ACH / JueJong, OLSS, LuckyJ

- **Why two players.**  The ACH paper is direct about it: CFR is guaranteed
  to converge to a Nash equilibrium only in two-player zero-sum games, and
  policy-gradient self-play has no such guarantee; Li et al. (Suphx) had
  already done four-player mahjong with supervised learning and traditional
  reinforcement learning, and the authors say that they, by contrast, study
  1-on-1 mahjong from the game-theoretic side, that is, with a Nash
  equilibrium as the goal.  The benchmark's description likewise says it is
  there to advance mahjong research from the game-theoretic side.  The
  other reason is that this two-player game itself has a large number of
  players in Tencent's online games.  [§1, §7.1, appendix A][ach]
- **The two-player mahjong's spec.**  Rules close to Chinese Official
  mahjong, with only 72 tiles (characters, winds, dragons, flowers,
  seasons) and 13 each.  The size of an information set is about 10^11,
  against about 10^3 in poker.
  [ACH appendix A][ach], [OLSS appendix D][olss]
- **Evaluation.**  10,000 games against rule bots every 500 training steps;
  a best response trained separately measures a lower bound on
  exploitability.  Against humans: over 7,700 games with 157 players it won
  4.56 points a game on average; against the champion Haihua Cheng (程海華)
  over 1,000 games, 0.82 ± 0.96 points, one-sided p = 0.19.  The authors'
  wording is that the champion failed to exploit JueJong effectively, not
  that JueJong won significantly.  [§7][ach]
- **OLSS** is real-time search on the same two-player mahjong by a group
  with overlapping authors.  The evaluation section states: two-player
  mahjong has high variance, so 200,000 games were played, 100,000 deals
  generated at random and each played twice with the two sides swapping
  seats.  Even so the 95% confidence interval is still ±0.13 fan, and the
  lead a game with search applied is 0.14 to 0.22 fan.  The search has two
  kinds of opponent model: "discard one tile at random by the distribution
  of the tiles left" and the learnt ACH model; the two give the same result
  within error (against ACH, 0.14 against 0.18; against PPO, 0.22 against
  0.20).  [appendix D, §7.2, table 3][olss]
- **Was it carried to four players?**  No paper on LuckyJ was found.  By
  media reports of Tencent's announcement, LuckyJ uses ACH, the
  policy-optimisation algorithm (self-play with reinforcement learning plus
  regret minimisation), and an imperfect-information search based on
  "optimistic value estimation"; in July 2023 it reached 10 dan in Tenhou's
  特上 room, stable rank 10.68, over 1,321 games.  These are all
  second-hand sources, and the training details of the four-player version
  could not be verified.  [IT之家][luckyj-ithome], [量子位][luckyj-qbit],
  [TechNode][luckyj-technode]
- **Another mahjong paper with overlapping authors** is RVR (IEEE CoG
  2022).  The abstract says that mahjong has much hidden information and
  complex scoring, and that the reward signal is highly random and has high
  variance; the method reduces the variance of the reward with a relative
  value network that sees global information plus an expected reward
  network.  The full text was not obtained; the abstract does not say which
  mahjong it is, only that it beat 62.5% of the opponents on Botzone.
  [DOI 10.1109/CoG51982.2022.9893584][rvr]

That is, by the announcement what was carried over is the algorithm, and
the multiplayer mahjong paper uses a separate device for the variance of
the reward.  Nowhere in the papers read is the "play" learnt in two-player
mahjong said to be put to use at four players.

Carries over: for a hand-written bot, only two observations.  Evaluation
has to swap seats on the same deal, and even 200,000 games of two-player
mahjong could only just resolve 0.14 fan; treating the opponent in the
search as a random discarder cannot be told apart from using a learnt
opponent model.  That is, in this highly random game a more accurate
opponent model did not make the search stronger.

### Chinese Official mahjong and the Botzone competitions

- The two IJCAI Mahjong AI Competitions, of 2020 and 2022.  The top sixteen
  fall into three kinds: heuristics from human knowledge, supervised
  learning, reinforcement learning; the authors conclude that the latter
  two are clearly stronger than heuristics.  In the first competition most
  teams used heuristics: a search computes the shanten number, the hand
  then decides which pattern to go for, and it is all wired into a
  behaviour tree.  [§5][oim]
- **How the duplicate is done.**  In four-player mahjong eats and pongs
  change the order of the draws, so the wall is cut into four parts and
  each seat draws only from its own, the deal included; one wall is played
  24 times, the four agents taking every seating, and the scores are
  summed as one match.  The authors say this suppresses not only the luck
  of the wall but also one seat's influence on another.  Around that is a
  Swiss system, because a four-player game cannot be scheduled as a round
  robin; a stable ranking takes hundreds of rounds.  Flower tiles, which
  affect only the score and not play, are simply removed.  [§4.2][oim]
- The match of humans against the AI changes the duplicate into "the humans
  each sit in the same seat at different tables, facing the same wall and
  the same three baseline AIs", because people remember the wall.
  [§4.4][oim]
- The demonstration paper of the third competition restates the same
  format.  [IJCAI-24][ijcai24]

Carries over: the duplicate with the wall cut by seat (see section D).  To
remember: in this competition the hand-written heuristics lost to the
learnt methods.

### Others

- **Meowjong** does three-player mahjong.  The authors treat it as a game of
  its own (fewer tiles, fiercer play), not a step before four players.
  [arXiv 2202.12847][meowjong]
- **CFR-p** adds a hierarchical policy abstraction to CFR and applies it to
  two-player mahjong.  Only the abstract was read.
  [arXiv 2307.12087][cfrp]

## B. Gin rummy and rummy AI

### The EAAI 2021 Gin Rummy Undergraduate Research Challenge

14 teams submitted 14 AIs and 14 papers, of which 13 were accepted.  The
format: the 14 AIs are split into two groups of 7, the top four of each
group go on to the last eight, and the last eight play 10,000 games in all
and are ranked by Elo.  First MARJJ, joint second GettysburgPlayer and
GinRummyAndTonic.  [results][gr-results], [challenge home page][gr-home]

| Paper | Contribution | Numbers |
| --- | --- | --- |
| [A Heuristic Evaluation Function for Hand Strength Estimation][gr-marjj] (MARJJ, first) | Discard score = present deadwood + future deadwood decaying by turn (only the best k draws) + the card's use to the opponent; knocks by MCCFR | With the bonus or penalty for "the opponent can certainly make a run" or "can certainly make a set" set to zero, the win rate against the original setting over 50,000 games is 0.469 to 0.480 |
| [Knocking in the Game of Gin Rummy][gr-knock] (joint second) | Linear regression over a population of AIs that play differently, learning when to knock | 10,000 games for each pairing, Wilson intervals |
| [Extracting Learned Discard and Knocking Strategies][gr-extract] (joint second) | Learns the expected utility of a discard; knocks by MCCFR, and compares the equilibria under different rules | |
| [Estimating Card Fitness for Discard][gr-fitness] | Ranks the discards by four properties, then chooses with a simplified CFR | |
| [A Deterministic Neural Network Approach][gr-dnn] | Compares heuristic and neural-network models for drawing, discarding and knocking | |
| [A Data-Driven Approach for Gin Rummy Hand Evaluation][gr-data] | CNN, Monte Carlo and Bayesian inference compute two scores, attack and defence | 55.4% ± 0.8% against Simple |
| [Opponent Hand Estimation in the Game of Gin Rummy][gr-ohe] | Bayes and two kinds of neural network estimate the opponent's hand, comparing the accuracy at each stage | Used for the discard, 5% more wins |
| [Heisenbot][gr-heisen] | Three sets of rules for drawing, discarding and knocking, the combination chosen by simulated tournaments | |
| [A Highly-Parameterized Ensemble][gr-ensemble] | Discards by a weighted ensemble of several grades, the parameters tuned in batches | 0.701 → 0.739 against the baseline |
| [Random Forests for Opponent Hand Estimation][gr-rf] | A random forest trained on self-play data estimates each card's chance of being in the opponent's hand | 61% against the baseline that does not estimate |
| [Opponent Modeling and Myopic Meld Distance][gr-mmd] | The hand's distance from melds + the cards the opponent wants + a conservative knock, compared by removing one at a time | 74.6% against Simple (500 games) |
| [Opponent Hand Estimation Using Deep Neural Networks][gr-dnn-ohe] | Trains a DNN on data from a heuristic agent to estimate the opponent's hand, then turns back to improve the agent | 855/1,500 = 57% |
| [Modeling Expert Knowledge in a Heuristic-Based Gin Rummy Agent][gr-tonic] (Tonic) | Writes expert maxims as a model with parameters, grid search | At least 68% against the baseline |

Three things can be read off:

- **The top three all solve the knock as a small decision of its own.**  Two
  papers use MCCFR and one linear regression.  The discard is a
  hand-written or learnt grade; MARJJ's authors say the discard has too
  many information sets for CFR to be used directly.  [MARJJ][gr-marjj],
  [Maravich et al.][gr-knock], [Goldstein et al.][gr-extract]
- **Opponent information is useful, and its use is plain.**  MARJJ's five
  groups of bonuses and penalties look only at "cards known to be in the
  opponent's hand" and "cards already dead": the penalty rises when the
  opponent already has cards that make a run or a set with this one, and
  falls when this card can no longer be of use to the opponent.  The three
  papers that estimate the opponent's hand win 55% to 61% against the
  versions that do not.  [MARJJ][gr-marjj], [Francis et al.][gr-ohe],
  [Hein et al.][gr-rf], [Mishra & Aggarwal][gr-dnn-ohe]
- **The parameters are not sensitive.**  MARJJ's authors swept every
  parameter and conclude that the win rate stays around 50% over a wide
  range, the best value reaching only 51%.  The knock paper found that a
  regression fitted on one kind of opponent gets worse against another.
  Both points agree with the conclusion of `research/fitted-weights.md`.
  [MARJJ][gr-marjj], [Maravich et al.][gr-knock]

### Earlier research and environments

- **Kotnik & Kalita 2003.**  The same neural-network value function trained
  once by TD self-play and once by co-evolution.  The evolved version is
  the stronger; the best one won 87.5% in a tournament.
  [ICML 2003][kotnik]
- **OpenSpiel**'s gin rummy plays one hand only; a comment says the
  information states exceed 10^85 and the opponent's hand at the start is
  one of 41 choose 10.  **RLCard**'s gin rummy represents the state as 5
  planes of 52 dimensions.  Both are environments and come with no strong
  bot.  [gin_rummy.h][openspiel], [games.md][rlcard]
- **Kelidari et al. 2026 (arXiv 2607.06854).**  First builds a fixed rule
  bot as a yardstick: it computes the minimum deadwood exactly, knocks
  whenever it can and never guesses the opponent.  It beats every
  reinforcement-learning agent the authors trained 70% to 99% of the time.
  Two numbers bear directly on this project: a determinized search with
  uniform deals wins at most 26% against it; the same search, if it peeks
  at the opponent's hand, wins 41% at 10 rollouts a move and 85% at 120.
  Also, feeding the opponent's hand straight to the learning agent did not
  help (24.7% against 25.5%).  The abstract's numbers are taken from v2:
  the whole set of training techniques lifts the self-play champion from
  about 30% to about 34%.  [v2][gold]

### Other draw-and-discard games

- **Indian Rummy.**  One paper on a rule bot: MinDist (the edit distance
  from the hand to the nearest legal layout) as the grade, plus a model of
  the opponent's hand, in a two-player zero-sum simulation.  One on
  reinforcement learning: PPO, after one pass of imitation learning on
  stronger demonstration data, 53.9% against the strongest search-based
  opponent.  Only the abstracts of the two were read.
  [arXiv 2601.00024][mindist], [arXiv 2606.21975][irumai]
- **Dhumbal** (a Nepalese multiplayer card game).  Over 1,024 games the rule
  bot wins 88.3%, ISMCTS 9.0% and PPO 1.5%.  Only the abstract was read.
  [arXiv 2510.11736][dhumbal]
- **跑胡子 / 字牌, Conquian, Khanhoo.**  No AI paper was found.  The 跑胡子
  projects a GitHub search turned up are, by their descriptions, win
  checkers and game servers; they were not opened one by one.

### The author's own gin-rummy-engine

What was read is the README at `d77f2d7`, `docs/strong-opponents.md`,
`docs/marjj-parity-plan.md` and `docs/marjj-parity-design.md`.  It has the
same architecture as tsap-oo (`Strategy`, `View`, a greedy `HeuristicBot`, a
determinized `MonteCarloBot`), so it is the closest comparison.

- **In two-player gin rummy, a Monte Carlo of uniform deals and greedy
  playouts clearly beats the greedy bot.**  `mc:64` wins 63.6% (62.8% to
  64.4%) of whole matches against `greedy`; against the baseline
  `EaaiSimpleBot` it gets stronger all the way as the samples go from 64 to
  512 (65.4% → 74.7%); against MARJJ it levels off only after 512 (55.1% at
  1024 samples).  The same approach in four-player 十胡 is +0.148 ± 0.276
  points, no visible difference.
  [README][gre-readme], [plan][gre-plan]
- **Catching up with the strongest opponent took samples, not an opponent
  model.**  The gap to MARJJ was closed with 512 samples after each
  decision was made 3.9 times faster (play the same bit for bit).  The
  three diagnoses put forward under "understand the opponent" were each
  refuted by the cheapest A/B: playouts by the opponent's knocking habits,
  dealing opponent hands that have developed, changing the value function.
  [plan §6, §8][gre-plan]
- **A more accurate belief is not a stronger bot.**  Correcting the hands
  dealt to the opponent to match the measured deadwood gained 3.2 and 2.0
  points against the two weak opponents and lost 1.2 points against MARJJ:
  the bot was no longer undercut but no longer dared to knock either, and
  lost in the longer race to gin.  The author's conclusion is that only
  measurement has the say.  [design D3, M2.5][gre-design]
- **Loosening the threshold for leaving the greedy choice usually makes it
  weaker.**  Leaving it on noise is worse than playing the greedy choice.
  tsap-oo's `CONFIDENCE = 1` is the same knob.  [plan §6][gre-plan]
- **Measurement.**  Mirrored pairs with the seats swapped, confidence
  intervals with the pair as the unit, no stopping midway, no change of
  seed; every change is measured against three opponents (the target plus
  two gatekeepers it "must not regress" against), so as not to learn to
  beat one opponent only.  tsap-oo has only the one opponent,
  `HeuristicBot`, at present.  [README][gre-readme], [plan §5][gre-plan]
- **The lesson in procedure.**  All three mechanisms first "looked very
  plausible" in the game statistics and were then refuted by the A/B.  The
  plan therefore lays down: name the mechanism, find the smallest fixed
  version that can switch it on and off, measure that version first, and
  only then build anything else.  [plan §8][gre-plan]

Carries over: sample count and playout speed come before the opponent
model; every idea in the "opponent" direction has its smallest fixed version
measured first.  To note: gin rummy's Monte Carlo has the knock, a
high-leverage decision, to find; in 十胡 the win is compulsory, and there is
no such decision.

## C. 十胡 and Four Color Cards themselves

### AI papers and theses

None was found.  Places searched: arXiv titles and full text; general web
search (Chinese, English, and Tứ Sắc in Vietnamese); and the index pages of
the National Digital Library of Theses and Dissertations in Taiwan
(臺灣博碩士論文知識加值系統), searched for 四色牌 together with 電腦對局,
人工智慧 and 演算法 (computer game playing, artificial intelligence,
algorithm).  The search engines returned only papers on other games such as
mahjong and bridge; the only one to mention Four Color Cards is a master's
thesis on a board-game teaching aid for the elderly (only the search
snippet was seen; it was not opened).  The library's own search needs a
session and was not queried directly.

The public implementations and bots are already collected in
[`public-opponents.md`](public-opponents.md) and are not repeated here.

### Rules for two and three players

The per-source table (source, player count, deal, what changes at fewer
players) is in `../tsap-oo-book/research/small-tables.md`, in Chinese.

Conclusions:

- **That two and three players "can play" has several independent
  sources**: Chinese Wikipedia, 國史館 (Taiwan Historica), BGA, Huang,
  English and Vietnamese Wikipedia, Vinagames.
- **No source writes down any adjustment of the rules for two or three
  players.**  The deal is always 20 cards each and 21 to the dealer.
  Chinese Wikipedia's "20 each at 2 to 5 players" leaves a pile of only
  112 − 101 = 11 cards at five players, which agrees with the statement
  「五龍因剩餘牌堆張數較少」 in the same passage, but it has no citation.
- **BGA is the only spec that can be acted on for two players, and its
  ranked play is two-player.**  Its rules do not depend on the player count:
  a card discarded from a hand can be eaten only by the next seat, a drawn
  card by the drawer and the next seat, and a win, kong or pong is not
  limited by seat; the winner who reaches 10 胡 collects from every other
  player (「跟所有其他玩家收取」).  The two language versions write the
  payment differently: the Chinese has each seat pay "the 胡 less the
  threshold", the English "the 胡 over the threshold, plus 1 point from each
  seat".  [Chinese][bga-zh], [English][bga-en]
- **Applying these rules to two players changes the game.**  What follows
  is derived from the rules, not said by a source.  At two players the only
  opponent is at once the previous seat and the next seat, so both sides
  can eat every discard and every draw, and the seat restriction on eating
  in effect disappears.  The pile is 112 − 41 = 71 cards, not the 31 of four
  players.  The winner collects from one seat only and the loser pays it
  all, so dealing in costs the same as the opponent winning on its own
  draw.
- `chunhua1972/four-color-game` likewise deals 20 / 21 cards at 2 to 4
  seats, and changes to 16 / 17 and 14 / 15 cards at 5 and 6 seats; its
  document says outright that the latter are its own extension and does not
  claim them as traditional rules.  [DEVplan.md §3.2][fcg-plan]
- **Whether dealing in costs extra differs from place to place.**  In the
  stakes tnt records, whoever deals in sometimes has to pay for the other
  two seats; Vinagames' rule is that a seat that deals in without having
  discarded "the worst card" pays for the other losers.  This project's
  rule is that dealing in costs nothing extra, and the conclusion that
  "defence does not pay" holds only under this rule.
  [tnt][tnt], [Vinagames][vina]

## D. Two players first, then more

### Precedents

| Precedent | What carried over | What did not, or had to be added |
| --- | --- | --- |
| Libratus (two players) → Pluribus (six) [Brown & Sandholm 2019][pluribus] | The architecture of three things: a blueprint computed by self-play, abstraction, and real-time search | Libratus searched only when "the rest of the game is short enough to solve to the end", which cannot be done at six players; it became depth-limited, each player choosing among k continuation strategies at the leaves; the Nash-equilibrium guarantee and the "will not lose" property are both gone, and the goal became "beat humans in practice" |
| ACH / JueJong (two-player mahjong) → LuckyJ (four) [ACH][ach], [IT之家][luckyj-ithome] | By the announcement, the ACH algorithm and the search | No paper, and the details could not be verified; the multiplayer mahjong paper with overlapping authors adds a network that reduces the variance of the reward [RVR][rvr] |
| Poki (multiplayer) → PsOpti (two players) [Billings et al. 2003][psopti] | The opposite direction | Poki, written for multiplayer tables, was by the authors' account not designed for two players and is rather easily beaten; the two-player version uses instead the solution of a linear program after abstraction |
| Grafting two-player experts into three-player poker [Abou Risk & Szafron 2010][risk] | The subgame of the two players left after someone folds gets the two-player CFR solution | The results are inconsistent: one base strategy got stronger, the other weaker.  The graft is possible because in poker someone drops out; in 十胡 nobody does |
| Single-player mahjong → four-player mahjong [Mizukami & Tsuruoka 2015][mizukami] | The play of the fastest win, which becomes its own policy in the simulation | Folding, exposed melds and the opponent models were all added later; the authors say single-player play alone is not strong enough |

What they have in common: **what carries is the method, not the play
learnt.**  No case in the table above is one of "find good play in the
version with fewer players, then use it unchanged in the version with
more".  Tencent's stated reason for choosing two players is theory:
two-player zero-sum has the Nash-equilibrium guarantee, and exploitability
can be defined and measured.  Alberta's two-player version likewise starts
from "the optimal solution is guaranteed at least the value of the game".
[ACH §1][ach], [Billings et al. §1][psopti]

### How two-player zero-sum and multiplayer differ in theory

- **The equilibrium guarantee.**  In a two-player zero-sum game a
  Nash-equilibrium strategy does not lose in expectation to any opponent,
  and CFR's average strategy is guaranteed to converge to it.  At three
  players or more neither holds: the algorithm is not guaranteed to
  converge, and even if every player computes an equilibrium of its own and
  plays it, the combination need not be an equilibrium.  Pluribus's authors
  therefore say that in multiplayer games even "what the goal should be" is
  a problem.  [Brown & Sandholm][pluribus], [Abou Risk & Szafron][risk]
- **Multiplayer search.**  Sturtevant's analysis is that in multiplayer
  games UCT computes a mixed-strategy equilibrium and max^n a
  pure-strategy one.  Only the abstract was read.
  [Sturtevant 2008][sturtevant]
- **The trouble with determinization** is strategy fusion: the search makes
  its decision separately in each deal, while in fact it cannot tell which
  deal it is in.  Long et al. predict when PIMC is good enough from three
  properties (leaf correlation, bias, and how fast information is
  revealed), and point out that it does not do badly in practice at card
  games such as Skat.  [Cowling et al. §III][ismcts],
  [Long et al. 2010][pimc]
- **What inference is worth depends on the game.**
  - Skat (three players): fit "the chance of each hand feature on seeing
    this action" to game records and bias PIMC's deals by it, and it is
    clearly stronger; a later neural network that predicts where each card
    is makes it stronger again.  [Buro et al. 2009][skat],
    [Solinas et al. 2019][solinas]
  - Scrabble (two players): knowing the opponent's tiles outright is worth
    37 points a game; a Bayesian inference of the tiles it kept from the
    word it just played, 5.2 points a game (630 games).
    [Richards & Amir 2007][scrabble]
  - Dou Dizhu (three players): in the authors' own earlier study on a
    simplified Dou Dizhu, a perfect opponent model was only a little
    better than a uniform belief, so this paper does without a belief
    distribution altogether.  Their explanation is that positions in which
    useful information can be inferred from the opponents' actions are few.
    [Cowling et al. §I][ismcts]
  - Gin rummy: see the numbers of Kelidari et al. above.  Peeking at the
    opponent's hand takes the search from 26% to 85%, but feeding the hand
    straight to the learning agent does nothing.  [Kelidari et al.][gold]

### The author's reason: in two-player zero-sum, right and wrong are rewarded and punished more often

The author's claim: at four players a mistake is not always punished; in
two-player zero-sum a right decision is rewarded more often and a wrong one
punished more often, so it is easier to learn what a good decision is.

Sources that support the claim:

- **In two-player zero-sum the opponent's mistake does not become my loss in
  expectation.**  The Pluribus paper's account: the side that plays the
  equilibrium strategy does not lose in expectation; in games more complex
  than rock-paper-scissors, whenever the opponent picks a suboptimal action
  the equilibrium side wins in expectation.  Multiplayer games lack this
  property.  [Brown & Sandholm][pluribus]
- **Two-player zero-sum has a yardstick that does not depend on the
  opponent.**  ACH trains a best response separately to measure how far
  each agent can be exploited; OLSS reports exploitability directly on
  poker.  A four-player game has no such quantity.
  [ACH §7][ach], [OLSS §7.1][olss]
- **Estimators that take the luck off work far better at two players.**
  DIVAT cuts the hands needed to 1/5.7 to 1/7.2 in two-player limit poker;
  MIVAT matches it at two players and cuts the variance by only 20% in
  six-player limit poker, which the authors put down to too few features
  and to not capturing where the players yet to fold are seated.
  [DIVAT][divat], [MIVAT][mivat]
- **The cost of a duplicate grows with the factorial of the player count.**
  Two players is 2 games on the same deal with the seats swapped,
  three-player poker 6 permutations, and four-player mahjong 24 with the
  wall cut by seat as well.
  [OLSS appendix D][olss], [Abou Risk & Szafron §4.4][risk],
  [Lu et al. §4.2][oim]
- **This project's own data.**  In the 1,691 deals where the control dealt
  in and the candidate did not, another seat still won 82% of the time
  (`doc/stronger-bot.md`).  That is "a right decision not rewarded".

Sources that do not support it, or that discount it:

- **Evaluating two-player mahjong did not get cheap.**  OLSS used 200,000
  games with the seats swapped and the 95% confidence interval is still
  ±0.13 fan, the lead with search applied 0.14 to 0.22 fan; JueJong against
  the human champion got only p = 0.19 from 1,000 games.
  [OLSS appendix D, table 3][olss], [ACH §7][ach]
- **Two-player poker is drowned in the luck of the deal just the same.**
  Two-player limit poker has a standard deviation of about 6 dollars a
  hand, the precision wanted is about 0.05 dollars, and that takes over
  fifty thousand hands; the 2015 match of humans against the AI played
  80,000 hands, and a large margin was only on the edge of significance.
  The main source of the noise is the deal, not the player count.
  [MIVAT][mivat], [AIVAT introduction][aivat]
- **The equilibrium guarantee does not help an A/B against a fixed
  opponent.**  This is this document's inference, not a source's.  The
  first two arguments above are both about a strategy that "does not lose
  to any opponent".  `ab/run.sh` measures the score against `HeuristicBot`;
  at two players as at four, "right" only means "earns more against this
  opponent", and the Nash equilibrium adds nothing.
- **What is learnt in the version with fewer players may not be there in
  the version with more.**  Poki's case is evidence in the opposite
  direction; the results of grafting two-player experts are half good and
  half bad.  [Billings et al.][psopti], [Abou Risk & Szafron §5][risk]

Not found: no study was found that directly measures "how often the same
mistake is punished in the two-player and multiplayer versions", nor were
the authors of ACH or Pluribus found to give "good decisions are easier to
learn at two players" as their reason for choosing two.  The reasons they
give are the theoretical guarantee and computability.

A rough estimate (this document's arithmetic, not a source's numbers;
assuming every seat has the same win rate, every win is h 胡, and exhausted
rounds are about 6%): at four players the winner collects 3h and the losers
pay h each, raising its own win rate by δ is worth about 4hδ, and the
standard deviation is about 1.7h a round; at two players it is worth about
2hδ, with a standard deviation of about 1h.  The two signal-to-noise ratios
are close (2.4δ against 2.1δ).  If the two-player version has an edge, it
has to come from δ itself being larger: the candidate bot makes half of the
decisions rather than a quarter, and its mistakes are not picked up by a
third seat.  That is possible, but nobody has measured it.

### Techniques that reduce evaluation variance

As things stand: `ab/` is already a paired duplicate.  Working back from
the standard errors of the experiments in `doc/stronger-bot.md`, on the
deals where play differs the score difference has a standard deviation of
about 27 to 30 points (about 37 for the one on the waiting rule), almost as
large as the standard deviation of one seat over a round (about 30).  The
standard error is this number times √(the deals that differ) over the total
deals, so the noise to suppress is the noise "after they part".

| Technique | What it does | Effect the sources report | What it means in `ab/` |
| --- | --- | --- | --- |
| Duplicate, common random numbers | The same deal and the same opponents, only the subject changes | Mortal, akochan, Botzone, OLSS and Mizukami all use it | Already there |
| Rollout | From the position being compared, redraw the randomness that follows many times and average; every candidate uses the same random numbers | GNU Backgammon's rollout feature, whose manual takes 1,296 games as its example; Mizukami's simulation uses the same random numbers for every candidate tile | From the first point where they part, reshuffle the pile left K times, play each side K times, and take the mean difference |
| Stratified first step | Make each random outcome of the first step or two come up exactly once | GNU Backgammon's quasi-random dice | In the reshuffles, rotate "the next card drawn" through every kind by the copies left |
| Taking off the luck (advantage sum, control variate) | For every chance event, add "the expected grade before the event − the grade after the event" to the result; this term has zero expectation, so it is unbiased | DIVAT 5.7 to 7.2 times; AIVAT cuts the standard deviation from 25.96 to 8.10 in two-player no-limit, over ten times fewer hands; MIVAT cut the variance by only 20% in six-player limit | At every draw, grade each kind of card possible in the pile and average, and add (the average − the grade of the card actually drawn) to the score |
| Truncation | Do not play to the end; after a few moves put the grade in place of the result | Tesauro and Galperin: far less variance in each trial, at least an order of magnitude faster; the GNU Backgammon manual warns that this brings in systematic error | The conversion of the grade into points holds only for "hands" (`research/grade-in-points.md`); used in an A/B it brings in bias, and is not advised |
| Cutting the pile by seat | Each seat draws only from its own stack, so the cards the other seats draw do not change when an eat or a pong changes the order | Botzone's authors say it reduces one seat's influence on another, and give no number | After the two sides part, each seat still draws the same cards, and the correlation between the two sides rises |
| Sequential test | Compute the likelihood ratio as the run goes, and stop when it is enough | fishtest's GSPRT; the paired pentanomial model saves substantially over the trinomial | Fewer deals on average, but the iron rule "the deal count is fixed before the data" has to become "the bounds are fixed before the data" |

Sources: [GNU Backgammon rollouts][gnubg-ro], [quasi-random dice][gnubg-qr],
[Tesauro & Galperin 1996][tesauro], [DIVAT][divat], [MIVAT][mivat],
[AIVAT][aivat], [Pluribus][pluribus], [Lu et al. §4.2][oim],
[Mortal][mortal-strength], [akochan][akochan], [fishtest][fishtest]

Further notes:

- **Why a rollout is unbiased.**  This is this document's inference.  Before
  they part, every decision of both sides depends only on cards already
  turned up, so at the moment they part the order of the pile left is still
  uniformly random to both sides.  Reshuffling that stretch K times and
  averaging leaves the expectation unchanged, and only averages out "the
  luck after they part".  The other seats' concealed cards cannot be
  redealt; they are part of the position.
- **How much it saves has to be measured first.**  The variance of the score
  difference splits into two parts: σ_b², the variance of the real
  difference between positions, and σ_w², the variance of the pile's luck
  within one position.  K reshuffles divide only the latter by K.  If σ_b
  is 5 points, K = 16 takes the standard deviation from 28 to about 8.6,
  the same as ten times the deals; if σ_b is 10 points, about five times.
  σ_b is unknown at present: 1,000 positions that part, reshuffled 32 times
  each, would measure it.  The cost is added only on the deals that part,
  so the smaller a candidate's change, the better the bargain.
- **Only the "chance event" half of AIVAT concerns this project.**  Its
  other half uses a known stochastic policy; `HeuristicBot` is
  deterministic, so that half is zero.  That is, what can be used here is
  MIVAT's form.  MIVAT also provides a way to choose the grading function:
  fit a linear function with "the minimum variance of the estimator" as the
  direct objective, and the features may use all four seats' hands, because
  the evaluator sees them.  The hand fit of the `fit` example already has
  the features ready.  [MIVAT][mivat]
- **Pluribus's match against humans** is an instance of AIVAT in a
  multiplayer game: 10,000 hands, winning 48 mbb a hand on average (an mbb
  is a thousandth of a big blind), standard error 25 mbb.
  [Brown & Sandholm][pluribus]
- **The mahjong projects all use the duplicate, and none an advantage sum.**
  Suphx is a fixed 1,000,000 games plus resampling; Kurita and Hoki use the
  difference in placing within the same game; Mizukami uses matches on the
  same walls plus a bootstrap and Welch's t-test.  [Suphx §4][suphx],
  [Kurita & Hoki §VII][kurita], [Mizukami §VIII][mizukami]

## Ideas that carry over to 十胡

Ordered by "expected information ÷ cost".  The first two are measurement
and do not change the bot.

1. **Roll out from where they part, and take off the luck of the draws.**
   Sources: [GNU Backgammon][gnubg-ro], [MIVAT][mivat], [AIVAT][aivat].
   Meaning: `ab/src/main.rs` records the position when the two sides first
   make different decisions and reshuffles the pile left K times, playing
   each out; then it takes off, for every draw, a correction with zero
   expectation.  The simplest correction is "whom this card lets win at
   once": the evaluator knows what each of the four seats waits on, and can
   compute the chance that each seat wins on this draw and the amount.
   Cost: offline only.  The new estimator has to be validated first: it
   must be exactly zero when the candidate and the control are the same,
   and reproduce a known conclusion (the −0.037 one, for instance).
   For: of the plan's seven A/Bs (counting the re-measurement of
   `MonteCarloBot`), five are insufficient data, and the bottleneck is the
   standard error.  Against: the effect depends on σ_b, not yet measured;
   the six-player poker example shows that a luck correction has limited
   effect with many players.
2. **Measure the ceiling of a "Monte Carlo that peeks at the hands".**
   Sources: [Richards & Amir][scrabble], [Kelidari et al.][gold],
   [Cowling et al.][ismcts].
   Meaning: have `MonteCarloBot`'s deals use the other seats' real hands
   (the pile's order still random, or the real pile too, each measured
   once), against `HeuristicBot` on the same A/B.  This is the ceiling of
   any opponent inference.
   Cost: offline only, and a very small change.
   For: the Monte Carlo of a uniform belief is +0.148 ± 0.276; if peeking
   too adds only a few tenths of a point, the behavioural inference of
   layer 4 and "the Monte Carlo's deals" can both be left undone.  Against:
   peeking also removes strategy fusion, so it overestimates the part that
   inference can get.
3. **"Don't feed the next seat" written as MARJJ-style bonuses and
   penalties.**
   Source: [Ahmed et al.][gr-marjj].
   Meaning: add to the discard's grade a term for "this card's use to the
   next seat".  The penalty rises if the next seat has exposed, or can be
   inferred to hold, cards that meld with it; it falls if the cards that
   meld with it are all dead (the "walls" of layer 3 in
   `doc/stronger-bot.md`).
   Cost: of the order of `HeuristicBot`, usable on the web page.
   For: the plan already lists it as the next step; 16% to 56% of discards
   can be eaten into a run by the next seat.  Against: gin rummy is
   two-player and the opponent is the only one to benefit; the next seat in
   十胡 is only one of three, and neither of the two defence experiments
   won.  The expectation is a small positive value, which only item 1 can
   make measurable.
4. **The single-player expected value, plus "the chance each turn that
   another seat wins".**
   Sources: [tomohxx][tomohxx-prob], [Kurita & Hoki][kurita],
   [Li & Yan][lets-play], [Suphx §2][suphx].
   Meaning: route 3, but as Kurita and Hoki do it, replacing the other
   three seats with an environment that has a chance each turn of ending
   the round, the chance taken from the table by count of concealed cards
   in `research/safe-cards.md`.  The opponents' progress then enters the
   grade, instead of serving only to avoid dealing in.
   Cost: one computation depends on the number of states; in 十胡 all four
   seats may want every draw, so "the cards that can be had" are harder to
   define than in mahjong.  Whether the web page can use it is unknown.
   Against: the plan's own judgement stands.  Under a uniform belief a
   Monte Carlo that looks a few steps further did not win, and neither did
   the fitted weights; this item computes the same information more
   accurately.  No data supports doing it first.
5. **Fit "how far this seat is from winning" on public features.**
   Sources: [Mizukami & Tsuruoka §IV][mizukami], [akochan][akochan-tenpai].
   Meaning: logistic regression on the count of concealed cards, the 胡
   exposed, and the number and timing of eats and pongs.
   Cost: fitted offline, looked up in a table online.
   Against: the probability table of layer 2 is already a simplified form
   of this, and used for defence it is a net loss.  Not worth doing by
   itself unless item 4 needs it.
6. **Oracle guiding, pMCPA, distillation.**
   Source: [Suphx][suphx].
   All need a deep-learning training pipeline; even Suphx itself did not
   use pMCPA when it went live.  The plan puts distillation after "a Monte
   Carlo with an opponent model clearly beats `HeuristicBot`", an order
   that agrees with the literature.
7. **The two-player version as a research vehicle.**
   Source: section D.
   For: the spec is ready-made (BGA's table of two, and ranked play
   defaults to two players); the duplicate takes only 2 games with the
   seats swapped; the candidate makes half of the decisions.
   Against: there is no precedent for carrying the play learnt in the
   version with fewer players over to the version with more; evaluating
   two-player mahjong still took 200,000 games; at two players the seat
   restriction on eating disappears, the loser pays it all and the pile is
   71 cards, so "defence does not pay", a conclusion of four players, very
   likely does not hold, and conversely what is measured at two players is
   not guaranteed to hold at four.
   If it is done, the advice is to spend one small experiment first on
   confirming two things: the two-player version's standard deviation a
   round and its share of deals that part; and whether a known four-player
   conclusion (for instance the −0.037 of "take 12 胡 × danger off the
   grade") has the same sign in the two-player version.  Only with the same
   sign is there any talk of carrying over.
   Follow-up: the former and two known wins have since been measured, see
   [`two-seats.md`](two-seats.md); the candidate of the −0.037 run was not
   kept in the repository and cannot be replayed.

## Caveats

- **Full text not obtained:** ACH's OpenReview page blocks automated
  access, and what was read is the PDF the Internet Archive saved on
  2024-09-22; *Official International Mahjong* was read from the MDPI PDF
  saved by the Internet Archive; Pluribus from the archived copy of the
  author's site.  RVR (IEEE CoG 2022) only as the abstract obtained through
  Semantic Scholar and OpenAlex.  Of Sturtevant 2008, CFR-p, the two Indian
  Rummy papers and Dhumbal only the abstracts were read.
- **LuckyJ has no paper.**  Every detail comes from IT之家, 量子位 and
  TechNode relaying Tencent's announcement; Tencent's original announcement
  was not found.  "ACH is used in the four-player version" is the
  announcement's claim, with no technical document to check it against.
- **Mortal's method has no documentation.**  The official documentation
  itself says most of its pages are empty; only its way of evaluating is
  cited here.
- **Only akochan's headers and readme were read**, not the implementation,
  and nothing was run.
- **The National Digital Library of Theses and Dissertations in Taiwan was
  not searched directly.**  "No paper found" is a conclusion within what
  the search engines index.  CNKI and Wanfang were not searched either.
- **The versions of Bakuuchi after that have no public paper.**  Kurita and
  Hoki mention that the Bakuuchi they played against is stronger than the
  version of the 2015 paper, and unpublished.
- **fishtest's official page is only an overview**; the derivations of the
  GSPRT and the pentanomial model are in the documents it links to, which
  were not read.
- **Wolfe's 2002 baseline-strategy estimator for blackjack** (subtract from
  the player's result the baseline strategy's result on the same hand, a
  50-fold drop in the standard deviation) is seen only in MIVAT's account;
  the original was not read.  Its form is the same as the paired design of
  `ab/`.  [MIVAT][mivat]
- **The three inferences of section D** (the rollout is unbiased, the rough
  estimate of the signal-to-noise ratio, the equilibrium guarantee does not
  help an A/B against a fixed opponent) and section C's "what happens when
  it is applied to two players" are derived by this document from the rules
  and the existing data; each is marked as such in the text and is not a
  source's conclusion.  σ_b has not been measured.  The two-player version
  was measured once after this document was written, see
  [`two-seats.md`](two-seats.md): the rough estimate that the
  signal-to-noise ratios are close agrees with the measurement.
- **Of the literature after mid-2026** only Kelidari et al. on arXiv (July,
  revised in August) and IRumAI (June) were found.  The page of Botzone's
  sixth competition came up in the search results; it was not opened and
  the results were not checked.
- **The win rates of the gin rummy papers have different baselines** (the
  Simple player, each team's own baseline, one another), and the numbers in
  the table cannot be compared with each other.

[gre-readme]: https://github.com/jdh8/gin-rummy-engine/blob/d77f2d78bb0e657d030faa629ae93cc5bc6c1e47/README.md
[gre-plan]: https://github.com/jdh8/gin-rummy-engine/blob/d77f2d78bb0e657d030faa629ae93cc5bc6c1e47/docs/marjj-parity-plan.md
[gre-design]: https://github.com/jdh8/gin-rummy-engine/blob/d77f2d78bb0e657d030faa629ae93cc5bc6c1e47/docs/marjj-parity-design.md
[suphx]: https://arxiv.org/abs/2003.13590
[mizukami]: https://www.logos.ic.i.u-tokyo.ac.jp/~tsuruoka/papers/cig2015mizukami.pdf
[kurita]: https://arxiv.org/abs/1904.07491
[tomohxx-prob]: https://tomohxx.github.io/mahjong-algorithm-book/probability/
[mahjong-cpp]: https://github.com/nekobean/mahjong-cpp
[shanten]: https://github.com/tomohxx/shanten-number
[lets-play]: https://arxiv.org/abs/1903.03294
[deficiency]: https://arxiv.org/abs/2108.06832
[mortal]: https://github.com/Equim-chan/Mortal
[mortal-strength]: https://github.com/Equim-chan/Mortal/blob/main/docs/src/perf/strength.md
[mortal-meta]: https://github.com/Equim-chan/Mortal/blob/main/docs/src/ref/meta.md
[akochan]: https://github.com/critter-mj/akochan
[akochan-tenpai]: https://github.com/critter-mj/akochan/blob/master/ai_src/tenpai_est.hpp
[akochan-exp]: https://github.com/critter-mj/akochan/blob/master/ai_src/exp_values.hpp
[akochan-betaori]: https://github.com/critter-mj/akochan/blob/master/ai_src/betaori.hpp
[kanachan]: https://github.com/Cryolite/kanachan
[ach]: https://openreview.net/forum?id=DTXZqTNV5nW
[olss]: https://proceedings.mlr.press/v202/liu23k.html
[rvr]: https://doi.org/10.1109/CoG51982.2022.9893584
[luckyj-ithome]: https://www.ithome.com/0/705/059.htm
[luckyj-qbit]: https://www.qbitai.com/2023/07/67614.html
[luckyj-technode]: https://technode.com/2023/07/12/tencents-mahjong-ai-sets-new-gaming-record-on-international-mahjong-platform/
[oim]: https://www.mdpi.com/1999-4893/16/5/235
[ijcai24]: https://www.ijcai.org/proceedings/2024/1020
[meowjong]: https://arxiv.org/abs/2202.12847
[cfrp]: https://arxiv.org/abs/2307.12087
[gr-home]: https://cs.gettysburg.edu/~tneller/games/ginrummy/eaai/
[gr-results]: https://cs.gettysburg.edu/~tneller/games/ginrummy/eaai/gin-rummy-results.pdf
[gr-marjj]: https://ojs.aaai.org/index.php/AAAI/article/view/17820
[gr-knock]: https://ojs.aaai.org/index.php/AAAI/article/view/17837
[gr-extract]: https://ojs.aaai.org/index.php/AAAI/article/view/17827
[gr-fitness]: https://ojs.aaai.org/index.php/AAAI/article/view/17825
[gr-dnn]: https://ojs.aaai.org/index.php/AAAI/article/view/17840
[gr-data]: https://ojs.aaai.org/index.php/AAAI/article/view/17843
[gr-ohe]: https://ojs.aaai.org/index.php/AAAI/article/view/17824
[gr-heisen]: https://ojs.aaai.org/index.php/AAAI/article/view/17823
[gr-ensemble]: https://ojs.aaai.org/index.php/AAAI/article/view/17839
[gr-rf]: https://ojs.aaai.org/index.php/AAAI/article/view/17830
[gr-mmd]: https://ojs.aaai.org/index.php/AAAI/article/view/17826
[gr-dnn-ohe]: https://ojs.aaai.org/index.php/AAAI/article/view/17838
[gr-tonic]: https://ojs.aaai.org/index.php/AAAI/article/view/17834
[kotnik]: https://cdn.aaai.org/ICML/2003/ICML03-050.pdf
[openspiel]: https://github.com/google-deepmind/open_spiel/blob/master/open_spiel/games/gin_rummy/gin_rummy.h
[rlcard]: https://github.com/datamllab/rlcard/blob/master/docs/games.md
[gold]: https://arxiv.org/abs/2607.06854
[mindist]: https://arxiv.org/abs/2601.00024
[irumai]: https://arxiv.org/abs/2606.21975
[dhumbal]: https://arxiv.org/abs/2510.11736
[zhwiki-shihu]: https://zh.wikipedia.org/wiki/%E5%8D%81%E8%83%A1
[bga-zh]: https://zh.doc.boardgamearena.com/Gamehelpfourcolorcards
[bga-en]: https://en.boardgamearena.com/doc/Gamehelpfourcolorcards
[tnt]: https://www.cs.cmu.edu/~tnt/rules.html
[vina]: https://www.vinagames.com/tusac.html
[fcg-plan]: https://github.com/chunhua1972/four-color-game/blob/main/DEVplan.md
[pluribus]: https://web.archive.org/web/2022/https://www.cs.cmu.edu/~noamb/papers/19-Science-Superhuman.pdf
[psopti]: https://www.ijcai.org/Proceedings/03/Papers/097.pdf
[risk]: https://poker.cs.ualberta.ca/publications/AAMAS10.pdf
[sturtevant]: https://www.cs.du.edu/~sturtevant/papers/sturtevant2008uct.html
[ismcts]: https://eprints.whiterose.ac.uk/id/eprint/75048/
[pimc]: https://cdn.aaai.org/ojs/7562/7562-13-11092-1-2-20201228.pdf
[skat]: https://cs.du.edu/~sturtevant/papers/skat.pdf
[solinas]: https://arxiv.org/abs/1903.09604
[scrabble]: https://www.ijcai.org/Proceedings/07/Papers/239.pdf
[divat]: https://cdn.aaai.org/AAAI/2006/AAAI06-092.pdf
[mivat]: https://www.ijcai.org/Proceedings/09/Papers/326.pdf
[aivat]: https://arxiv.org/abs/1612.06915
[gnubg-ro]: https://www.gnu.org/software/gnubg/manual/html_node/Rollouts-in-GNU-Backgammon.html
[gnubg-qr]: https://www.gnu.org/software/gnubg/manual/html_node/Quasi_002dRandom-Dice.html
[tesauro]: https://proceedings.neurips.cc/paper_files/paper/1996/file/996009f2374006606f4c0b0fda878af1-Paper.pdf
[fishtest]: https://official-stockfish.github.io/docs/fishtest-wiki/Fishtest-Mathematics.html
