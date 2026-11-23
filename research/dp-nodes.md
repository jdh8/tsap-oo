# The node count of single-player dynamic programming

Measured 2026-11-18.  The question: in the dynamic programming of route 3 of
`doc/stronger-bot.md`, a state is (concealed cards, exposed 胡), an edge is
"claim a card not yet seen, then discard one", and hands whose ceiling
cannot reach the threshold are pruned.  It estimated that a hand of 20 cards
has 10³ to 10⁴ reachable sub-hands and 10⁶ to 10⁷ table lookups, on the
order of milliseconds.  What does the count come to?  No change to play; the
base is `67d849a`.

## Answer

- **The estimate was two to three orders of magnitude too low.**  A freshly
  dealt hand of 20 cards averages 7.0 × 10⁵ nodes and 5.8 × 10⁷ edges, and
  takes 22 seconds to count once; the largest hand has 1.2 × 10⁶ nodes and
  1.2 × 10⁸ edges.  Neither the web page's hint nor an A/B of 800,000 deals
  can afford it.
- **Pruning only the nodes that "do not reach the threshold at
  `generals = 0` either" leaves 80% of them.**  Pruning does not save it.
- **Only hands of 12 cards or fewer are on the order of milliseconds**: 12
  cards average 2.0 × 10³ nodes, 5.8 × 10⁴ edges and 41 ms; 11 cards
  1.1 × 10³, 2.8 × 10⁴ and 20 ms; 10 cards 6 × 10², 1.5 × 10⁴ and 11 ms.
  That is the same stretch as the gate of route 6 (search only at 10
  concealed cards or fewer).
- So batch 4 of `doc/hu-ceiling.md` is not taken over by dynamic
  programming; its four candidates each get their own A/B.  A gated DP is
  another route, at a cost close to that of the gated Monte Carlo.

## Method

`examples/nodes.rs`.  Four `HeuristicBot`s in self-play; after every
decision of every seat, once the hand has settled (it has discarded or
passed), breadth first from that hand:

- A node is (concealed cards, exposed 胡).  The same concealed cards can be
  reached through different melds, for example ponging 士士 and then
  discarding 相, against eating 士相 into 將士相 and then discarding 士; the
  exposed 胡 differ, so the concealed cards alone are not the state.
- An edge: every card not yet seen (`live`, by the same computation as
  `HeuristicBot::start`, fixed at the root), every meld it can make in
  `locked_completions` (generals (將帥) are not eaten into pairs), and every
  card that can be discarded after.
- The nodes kept are those whose `ceiling(generals = 1)` reaches the
  threshold; those that reach it at `generals = 0` as well are counted apart
  ("certain").  Pruned nodes are remembered too and not computed again.
- Each decision point records the nodes, the certain nodes, the edges and
  the time; the edges counted are those that lead to a kept node, including
  one already seen.

The time is that of this census: two calls of `ceiling` for each new node
and one `HashMap` lookup for each edge, with none of the arithmetic of an
expectation.  The DP has to walk the same edges at least.

## Numbers

5 rounds, seed 1, hands of every size (`nodes 5 1`):

| Cards | Decision points | Nodes a point | Certain a point | Edges a point | Most nodes | Most edges | ms a point |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 20 | 17 | 699,775 | 556,517 | 57,846,262 | 1,225,317 | 118,096,407 | 21,814 |
| 19 | 1 | 132,504 | 109,910 | 9,401,181 | 132,504 | 9,401,181 | 3,796 |
| 18 | 8 | 140,355 | 100,918 | 8,459,740 | 322,182 | 21,238,151 | 3,846 |
| 17 | 20 | 65,139 | 51,392 | 3,891,157 | 251,516 | 18,409,176 | 1,605 |
| 16 | 4 | 19,498 | 14,678 | 821,270 | 29,308 | 1,388,991 | 430 |
| 15 | 5 | 16,503 | 11,746 | 643,687 | 33,076 | 1,309,333 | 353 |
| 14 | 14 | 8,040 | 5,730 | 300,434 | 20,502 | 825,158 | 168 |

200 rounds, seed 2, hands of 13 cards or fewer (`nodes 200 2 13`):

| Cards | Decision points | Nodes a point | Certain a point | Edges a point | Most nodes | Most edges | ms a point |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 13 | 225 | 5,405 | 3,921 | 205,698 | 29,606 | 1,654,648 | 114 |
| 12 | 297 | 1,957 | 1,352 | 57,890 | 9,231 | 414,569 | 41 |
| 11 | 340 | 1,061 | 710 | 27,859 | 4,722 | 176,690 | 20 |
| 10 | 143 | 607 | 409 | 14,690 | 4,295 | 158,800 | 11 |
| 9 | 153 | 193 | 128 | 3,244 | 953 | 24,843 | 3.2 |
| 8 | 114 | 130 | 87 | 2,126 | 892 | 23,395 | 2.0 |
| 7 | 55 | 63 | 41 | 812 | 221 | 3,966 | 0.9 |
| 6 | 40 | 16 | 10 | 126 | 61 | 726 | 0.2 |
| 5 | 25 | 18 | 12 | 152 | 72 | 887 | 0.2 |
| 4 or fewer | 14 | 3 | 1 | 9 | 9 | 31 | 0.0 |

## Discarding only loose cards

Measured 2026-11-18, base `2dcc17e`.  The question: a card in a meld is
hardly ever one to discard, and a card in a 搭子 seldom is; if a node tries
only the discards that leaves, how much smaller is the graph?  The fourth
argument of `nodes`:

- **1:** discard only the cards that at least one of `HeuristicBot`'s two
  layouts (a loose card costing 1 and 16) takes as loose; when there is
  none at all, try every card as before.
- **2:** of those, discard only the lone cards that make no 搭子 with
  another loose card; when there is no lone card, fall back to 1.
- **3:** see "A 搭子 counts only while the card it waits on is live" below.

Means per point, with the factor against no pruning in brackets
(`nodes 5 1 20 p` and `nodes 200 2 13 p`; the two runs at 20 cards ran at
the same time, so their times are a little high):

| Cards | Pruning | Nodes | Edges | ms |
| ---: | ---: | ---: | ---: | ---: |
| 20 | 0 | 699,775 | 57,846,262 | 21,814 |
| 20 | 1 | 104,580 (÷ 7) | 4,578,275 (÷ 13) | 5,176 |
| 20 | 2 | 33,697 (÷ 21) | 658,851 (÷ 88) | 1,241 |
| 17 | 0 | 65,139 | 3,891,157 | 1,605 |
| 17 | 2 | 6,205 | 111,895 | 176 |
| 16 | 2 | 2,857 | 38,048 | 73 |
| 14 | 2 | 1,396 | 19,112 | 32 |
| 13 | 0 | 5,405 | 205,698 | 114 |
| 13 | 1 | 1,913 (÷ 2.8) | 43,865 (÷ 4.7) | 50 |
| 13 | 2 | 1,227 (÷ 4.4) | 17,847 (÷ 12) | 28 |
| 10 | 0 | 608 | 14,690 | 10.8 |
| 10 | 1 | 316 (÷ 1.9) | 5,299 (÷ 2.8) | 6.4 |
| 10 | 2 | 229 (÷ 2.7) | 2,563 (÷ 5.7) | 4.4 |

- 20 cards still take 1.2 seconds, the same order as the gated Monte Carlo;
  pruning does not save the full hand.
- It pushes the gate up: 16 cards under pruning 2 (73 ms) are cheaper than
  13 cards unpruned (114 ms).
- This counts the size of the graph only.  How much too low the pruned
  discards make the values, and whether the choice at the root changes,
  waits for the core (`doc/expectation.md`, batch 1) to compare point by
  point on hands it can compute unpruned as well; the time includes two
  layouts for each edge, which the core has to pay too.

### Short nodes left unpruned

The fifth argument N: nodes of N concealed cards or fewer in the graph try
every discard as before, and only longer ones are pruned.  The reason is
that short hands are cheap, and the likeliest to hold a discard that should
not be pruned.  Pruning 2, `nodes 5 1 20 2 N`, nodes and ms per point (the
runs at N = 7 and 10 ran at the same time):

| Cards | N = 0 | N = 7 | N = 10 |
| ---: | ---: | ---: | ---: |
| 20 | 33,697 / 1,241 | 61,904 / 1,785 | 157,556 / 4,388 |
| 17 | 6,205 / 176 | 12,675 / 293 | 30,073 / 697 |
| 16 | 2,857 / 73 | 4,457 / 96 | 9,191 / 196 |
| 14 | 1,396 / 32 | 2,926 / 57 | 5,974 / 121 |
| 13 | 1,542 / 31 | 3,411 / 58 | 5,876 / 107 |

Most of the graph's nodes are short hands, so leaving the bottom unpruned
gives back most of what was saved: N = 7 costs twice as much and N = 10 four
times, and at 13 cards it is almost back to the unpruned 114 ms.

### A 搭子 counts only while the card it waits on is live

Pruning 3: as pruning 2, but two loose cards count as a 搭子 only while the
card that completes them into a meld is still in `live`; once the cards
waited on are all out, both are lone cards and may be discarded.  It
replaces "unpruned at so many cards or fewer": whether to prune depends on
whether the card is still there, not on how long the hand is.  Nodes and ms
per point (`nodes 5 1 20 3`, `nodes 200 2 13 3`):

| Cards | Pruning 2 | Pruning 3 |
| ---: | ---: | ---: |
| 20 | 33,697 / 1,241 | 34,717 / 1,321 |
| 17 | 6,205 / 176 | 6,318 / 181 |
| 13 | 1,227 / 28 | 1,260 / 30 |
| 10 | 229 / 4.4 | 229 / 4.5 |

Within 3% more: dead 搭子 are few, and letting them back in costs almost
nothing.

## The graph of one decision: batch 1

Measured 2026-11-18, base `7d3157a`.  Once the core of batch 1 of
`doc/expectation.md` (`Graph` in `src/expect.rs`) was written,
`examples/nodes.rs` switched to it: the numbers above are for "a hand after
its discard", these are for "one decision to discard", whose roots are the
hands left by discarding each card that can be discarded, sharing one graph.
The card counts are after the discard.  Once built, the graph is swept by
the actual pile, T = (pile − dead cards) × 1.3.  The four constants of the
table model are provisional (h 0.015, L −12, e 0.45, generals 0.25) and are
measured only in batch 2; they affect only the values under "how much
pruning changes" below, not the sizes or the times.  Since batch 2 `nodes`
uses the measured `Outlook::default()` (`research/outlook.md`); to reproduce
the values below use `c0aea83`.

The parameters of the two sections above (the "certain" column, the fifth
argument N) went away with the old census; to reproduce them use
`examples/nodes.rs` at `7d3157a`.  The edges now include those that lead to
the "can no longer win" node, so there are slightly more than above.

200 rounds, seed 2, 13 cards or fewer after the discard, no discards pruned
(`nodes 200 2 13 0`):

| Cards | Decisions | Roots | Nodes | Edges | Most nodes | Build ms | Sweep ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 13 | 174 | 9.1 | 8,333 | 392,882 | 54,828 | 121.5 | 10.7 |
| 12 | 227 | 8.1 | 3,198 | 131,171 | 16,972 | 48.2 | 3.7 |
| 11 | 253 | 7.5 | 1,669 | 56,964 | 8,211 | 22.3 | 1.6 |
| 10 | 115 | 6.9 | 1,052 | 33,218 | 7,863 | 13.0 | 0.9 |
| 9 | 128 | 5.9 | 315 | 7,053 | 1,721 | 3.7 | 0.2 |
| 8 | 100 | 5.6 | 238 | 5,083 | 1,593 | 2.5 | 0.1 |
| 7 or fewer | 121 | 4.1 | 59 | 926 | 399 | 0.6 | 0.0 |

- **The gate is passed.**  A decision at 10 cards takes 14 ms; the limit
  declared beforehand was 50.
- **The sweep is over ten times cheaper than the build**: the sweep is 6% to
  9% of the build.  "The multiplier for the chances left multiplies the
  sweep only" holds.
- A decision has 7 roots, yet the graph is only about twice the size of a
  single hand's (10 cards: 1,052 nodes against the census's 607): the roots
  share most of their sub-hands.

### How much pruning the discards changes the choice

On the same decisions, one graph is built for each of prunings 1 to 3 and
one more unpruned; each of the two graphs picks its root of highest value.
"Changed" is the decisions where the root picked under pruning is the worse
one by the unpruned values; "loss" is how many points it gives up by the
unpruned values, averaged over every decision; "bias" is the best root's
unpruned value less its pruned value (`nodes 200 2 13 p`):

| Cards | Pruning | Build + sweep ms | Changed | Loss a decision | Largest loss | Bias |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 13 | 0 | 132.3 | | | | |
| 13 | 1 | 104.0 | 2.3% | 0.008 | 1.03 | 0.042 |
| 13 | 2 | 54.4 | 5.2% | 0.022 | 1.03 | 0.096 |
| 13 | 3 | 57.3 | 4.0% | 0.019 | 1.03 | 0.093 |
| 12 | 0 | 51.8 | | | | |
| 12 | 2 | 21.1 | 8.4% | 0.022 | 0.90 | 0.107 |
| 10 | 0 | 13.9 | | | | |
| 10 | 1 | 12.1 | 0.9% | 0.003 | 0.35 | 0.036 |
| 10 | 2 | 7.1 | 4.3% | 0.023 | 1.44 | 0.054 |
| 10 | 3 | 7.4 | 4.3% | 0.023 | 1.44 | 0.054 |
| All | 1 | 30.2 | 3.2% | 0.010 | 1.03 | 0.048 |
| All | 2 | 16.5 | 5.5% | 0.020 | 1.44 | 0.074 |
| All | 3 | 17.3 | 5.3% | 0.019 | 1.44 | 0.073 |

- **The gate at 10 cards needs no pruning.**  Unpruned takes 14 ms and
  pruning 2 saves 7 ms, in exchange for 4% of decisions picked wrong and
  0.02 points a decision; the gated Monte Carlo wins only +0.070 a deal.
- Pruning is for opening a wider gate (application D): 13 cards under
  pruning 2 (54 ms) cost the same as 12 cards unpruned (52 ms).  The price
  then is about 0.02 points a decision, and only an A/B can say whether it
  is worth it.
- Pruning 3 picks wrong a little less often than pruning 2 only at 13 cards;
  otherwise they are the same.
- At 6 cards or fewer not one decision changes: there the prunings find
  almost nothing to prune.

## Why the estimate was wrong

Route 3 pictured the "reachable sub-hands" as a few thousand, but each step
has about 28 kinds of card, one to three melds for each kind and a dozen or
more cards to discard after, to a depth of 7; the ceiling prunes only the
hands that cannot be made, and a freshly dealt hand can still make almost
anything.  The node count divides by about 5 to 8 for every three cards
fewer in hand, so 20 cards and 12 cards are three orders of magnitude apart.

## What is not counted

- `live` is fixed at the root, so the same card can be asked for more times
  than it has copies left; nor do a node's edges care whether the seat is in
  turn to eat.  Both make the numbers too large, but that is how the DP's
  state is defined, and counting them in would only make more states.
- The dimension of the chances left is not counted; it is a multiplier
  (about 60), not part of the nodes.
- The decision points include the repeats of one hand passing several
  times; for the DP `live` differs each time, so it has to compute again
  too.
