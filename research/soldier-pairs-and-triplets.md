# Pairs and triplets of soldiers (兵卒)

Research checked 14 November 2026. The question: can identical soldiers —
same colour, so 兵兵 or 卒卒卒 — form a pair (對子), a triplet (坎 / 碰) or a
quad (開招 / 槓) like any other card, or are soldiers confined to the
three- and four-colour sets?

## Answer

- **Triplets and quads: yes.** No source excludes soldiers.
- **Pairs: disputed.** Board Game Arena forbids them outright; neither
  Wikipedia article has such a rule.

tsap-oo allows all three, following the Wikipedia reading. This is left as
is: a pair scores 0 胡, so the only effect of the BGA rule would be on which
hands are complete and which eats are legal, not on any score.

## Sources

| Source | Soldier pair | Soldier triplet / quad |
| --- | --- | --- |
| [Board Game Arena rules][bga] | **Forbidden**: "Note that Soldiers cannot be melded into pairs!" No reason is given. | Not excluded. The general rule is "A set of cards with the same type AND color - melded into a PAIR, 3 of a kind or 4 of a kind", and the pair note is its only exception. |
| [Chinese Wikipedia][zh] | Allowed by silence: 對子 is one of the five shapes a card may be eaten into, with no exception for soldiers. | Silent. 碰 and 槓 are mentioned ("同樣有吃、碰、槓") but never defined. |
| [English Wikipedia][en] | Allowed by silence: any two identical cards. | Allowed by silence: three or four identical cards. |
| [pagat][pagat] | Silent — an overview with no meld rules. | Silent. |

Chinese Wikipedia's list of shapes, verbatim: 「三張三色的兵卒（三色兵）、四張
四色的兵卒（四色兵）、三張同色的將士象（帥仕相）、三張同色的車馬包（俥傌炮）、
對子」.

## Caveats

- The Board Game Arena page was read in English: the `zh.` address redirects
  to the English document.  Its scoring table (pong 1 / 3, kong 6 / 8, three
  soldiers 3, four soldiers 5) matches `Rules::new`, so it describes the
  same game.
- The meld definitions of the English Wikipedia article were read through a
  summarising fetch, not verbatim; treat that row as weaker evidence.
- pagat links to Wei-Hwa Huang's archived Taiwanese rules, which were not
  read.  They would be the next place to look.
- No Taiwanese write-up was found that states the pair restriction, so
  whether it is a Taiwanese table rule or a Board Game Arena choice is
  unknown.

## In the code

`susik::meld::completions` offers `Meld::Pair` and `Meld::Triplet` for every
card before it looks at the rank, soldiers included.  Adopting the BGA rule
would mean rejecting soldier pairs in tsap-oo, for both eating and the
winning layout; susik deliberately leaves the question of which shapes count
to the game.

[bga]: https://en.doc.boardgamearena.com/Gamehelpfourcolorcards
[zh]: https://zh.wikipedia.org/wiki/四色牌
[en]: https://en.wikipedia.org/wiki/Four_color_cards
[pagat]: https://pagat.com/rummy/si_se_pai.html
