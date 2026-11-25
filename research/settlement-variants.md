# Settlement in the public write-ups: more than "each seat pays its 胡"

Checked 2026-11-23.  The question: do the public write-ups of 十胡 all have
the winner simply collect their 胡 from each of the other three seats?  This
reads public web pages only; no rounds were played and no code changed.
This repository's settlement (`Game::settle`: the winner collects their 胡
from every seat, and a 相公 pays every seat 10 胡) is what the README calls
the mainstream Taiwanese rule; this note records the layers it does not
model.

Added 2026-11-26: Board Game Arena's help, which this note had not read,
pays "the 胡 less the threshold" with a stake of 1 and turns a 點花 for
花胡; a 2009 blog post has a 底 of NT$5 and a 台 of NT$1, a card drawn for
each player.  Both rules of Board Game Arena are now in `Rules` (`base`,
`flower`; `doc/variants.md`, "底 and 點花"), and the quotes are in
`../tsap-oo-book/research/settlement.md`.

## Answer

**All the write-ups agree only that all three seats pay.  How much they pay
has a different stack of layers in each: 底, 花, 押點, a threshold,
doubling, offsets.  None says the discarder (放槍) pays alone.**

| Source | Who pays | How much | 胡 above the threshold | Doubling, other |
| --- | --- | --- | --- | --- |
| Chinese Wikipedia, 《十胡》 | 「放銃或自摸都是閒家全賠莊家」 (on a discard or a self-draw, every other seat pays the winner in full) | 台 and 底, no figures | Not stated | At five players or more, 五龍 doubles both 台 and 底; a missed kong makes the seat liable (包牌), how much not stated |
| 屏少的家 and 河洛人 (two blog posts, nearly identical) | Not stated | 底 (e.g. NT$50) plus 花 (NT$10 a 花); formula 「胡數 × 押點」 (胡 × stake) | The standard game only asks whether the hand reaches 10; in the 高胡 variant each 胡 beyond adds one 花 | 「清胡」 under 10 胡 is not paid; a 相公 pays every seat 「10 胡 × 押點」 |
| Kinmen Daily News, 《學玩四色（群胡）牌》 | Every seat pays | 群胡: self-draw 3 子, 著花 2 子, a plain win 1 子 (1 子 is NT$10); 四色: the 胡 plus three or plus eight | No threshold in 四色; the 胡 go straight into the formula | 開六 and 拾八 double; an opponent's 開六 or 拾八 offsets what the winner collects |

## The layers

- **底 (base).**  Both blog posts have it: a win first collects a fixed
  底, counted apart from the 胡.  Wikipedia only says a seat that sits out
  「損失該輪壓底」 (loses that round's base stake), not how 底 is
  collected.
- **花.**  The blogs: after the win, five more cards are turned from the
  pile, and each that matches a card in the hand in colour and face is one
  花; five hits or five misses are five 花 (「反」).  The Taichung version
  counts the four-colour soldiers as 4 胡 and adds 1 花 at payment.  The
  著花 of Kinmen's 群胡 means something else: 2 子 with a 花, 1 without.
- **押點 (stake).**  The blogs' formula is 「胡數 × 您的押點」 (胡 times
  your stake).  Each seat may stake a different amount, so each pays a
  different sum; a 相公 pays every seat 「10 胡 × 押點」, a fixed 10 胡
  whatever the hand.  This repository's 相公 penalty comes from here.
- **胡 above the threshold.**  The standard game pays once the hand
  reaches 10, and the 胡 beyond earn nothing more; in the 高胡 variant each
  胡 beyond adds one 花.  Kinmen's 四色 has no threshold, and the 胡 get
  three added (3 胡 is 6, 5 胡 is 8, …) or eight (3 胡 is 11, 5 胡 is
  13, …).
- **The dealer.**  Wikipedia writes every payment as 「閒家全賠莊家」,
  which on its face has all three seats pay and no discarder pay alone; it
  does not say whether the dealer's wins or losses double.  The blogs and
  the Kinmen article only give the dealer one extra card, with no
  difference in payment.
- **Doubling and offsets.**  Wikipedia: 五龍 doubles.  Kinmen's 四色:
  a 開六 (three alike) or a 拾八 (four alike) doubles, e.g. 「九胡十七」
  becomes 34; and an opponent's 開六 or 拾八 offsets what the winner
  collects, while a 砍 (three cards) offsets only against the other seats,
  not the winner.
- **放槍 (dealing in).**  The blogs define 放槍 as 「摸到別家正在聽的牌」
  (drawing the card another seat is waiting on); none of the three sources
  has the discarder pay differently from the others.

## Not found

- The only source Wikipedia cites, 遊戲基地's 《四色牌十胡篇》, is now a
  404.
- No source gives a Taiwanese rule for a doubled dealer or a doubled
  self-draw; the 「自摸翻倍」 (self-draw doubles) in search snippets comes
  from Fuzhou 四色牌 and Hengyang 十胡卡, not Taiwanese 十胡.

## Sources

- [十胡 - 維基百科](https://zh.wikipedia.org/zh-tw/%E5%8D%81%E8%83%A1),
  last revised 2023-05-04
- [金門日報 學玩四色（群胡）牌](https://www.kmdn.gov.tw/1117/1271/1274/228160),
  陳麗玉, 2013-08-24; the author writes as a beginner and says the
  rules differ from village to village
- [四色牌 @ 屏少的家](https://q04080070.pixnet.net/blog/post/26778394), 2010-08-06
- [四色牌 - 河洛人](https://lovepuppy999.pixnet.net/blog/posts/10014616370), 2008-04-26
- [四色牌十胡篇 - 遊戲基地](https://www.gamebase.com.tw/wekey/大蕃薯遊戲網戰/四色牌十胡篇/), dead
