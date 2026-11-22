# Hand layout and the mark on generals (將帥) in other implementations

Research checked 15 November 2026. The questions: how do other Four Color
Cards programs (台灣十胡, Vietnamese Tứ sắc, relatives) lay out the player's
hand; do they set a lone general apart as the meld it is, or keep it in the
hand next to 士／象; what mark do physical decks print on generals, and do
programs copy it; and what is the lone-general meld called?

Each claim is tagged **[code]** (read in source or shipped assets),
**[image]** (seen in a scan, photo or store screenshot, nothing more) or
**[inferred]**.

## Answer

- **Hand layout.** Every program found shows the hand as one flat row (two
  rows in one case) of upright narrow strips, sorted, with no gaps between
  groups.  The two Taiwanese 十胡 programs sort **by colour, then by rank**,
  so 將士象 and 車馬炮 of one colour fall next to each other.  Vietnamese
  apps either do the same or order the row set by set (complete sets first,
  leftovers at the right end), still without gaps.  Free drag-to-arrange
  exists in two hobby projects only.  Nobody draws the hand as spaced meld
  boxes the way `web/` does today.
- **Lone general.** No program found takes a dealt general out of the hand.
  It sits in the row at the head of its colour, beside any 士／象 of that
  colour, and is merely unclickable when discarding.  The single-general
  meld appears as a separate group in only two places: on the table, when a
  general is *drawn* (Board Game Arena and one hobby project lay it down at
  once), and in the scored layout of a finished hand.
- **Mark on the card.** In the Taiwanese deck the general — and only the
  general — carries a **cash coin (a ring with a square hole set on its
  corner) in the middle of the card**, between the upright and the inverted
  character.  This is on all four colours, in two independent pictures.  The
  Vietnamese short deck marks it differently: 福 in an octagonal cartouche
  with a small star (a round dot on the red suit) on each side.  Long
  Teochew, Hong Kong and Fuzhou decks do not distinguish the general at all.
  Board Game Arena reproduces the Taiwanese coin faithfully; one hobby
  project uses a star; the commercial apps show nothing visible.
- **Terminology.** Written sources say 「將、帥單支」 or 「單張將／帥」;
  one code base labels the meld 「單將」.  No source gives a Taiwanese
  (Hokkien) reading or a dedicated name for it.

## Hand layout and the lone general, by implementation

Taiwanese 十胡 first, then Tứ sắc.

| Implementation | Hand layout | Lone general |
| --- | --- | --- |
| [Board Game Arena][bga-panel], "Taiwanese style rules" | **[code]** One inline row of 44 × 144 px sprites inside `#fcc_hand`, centred, no overlap, no gaps.  `sortHand` orders by `type/10` then `type%10`: colour (red, green, yellow, white), then rank (general 1 … soldier 9).  No grouping, no drag, no alternative sort.  ([client JS][bga-js], [CSS][bga-css]) | **[code]** Stays in the hand, first card of its colour; the discard handler skips `.c11 .c21 .c31 .c41`.  A *drawn* general must be taken by the drawer (「抽出將/帥時，抽牌者必須吃下」, [zh help][bga-zh]) and goes to the table as a one-card group; `group_types` 1–4 are the four single generals, 1 point each.  **[inferred]** that dealt generals stay in hand until the win screen — the JS never moves them. |
| [國粹天天樂][gc] 四色牌(十胡), Abacus Technology | **[image]** ([screenshot][gc-shot]) One edge-to-edge row along the bottom, cards clipped by the screen edge so only the upper character shows.  Order seen: 紅 帥帥俥傌炮炮 · 綠 士象 · 黃 帥 · 白 將象 · 紅兵兵 · 黃兵兵 — colour blocks by rank, soldiers collected at the right end.  Claimable cards pop up out of the row.  Exposed melds are small overlapped groups beside the avatar. | **[image]** In the hand: 紅帥帥 lead the red block, 黃帥 stands alone between green and white, 白將 sits directly left of 白象.  Nothing sets them apart. |
| [chunhua1972/four-color-game][chunhua] (hobby, React) | **[code]** `sortHand(ids, 'color' \| 'role')` behind two toggle buttons; the sorted list is cut in half and drawn as two `hand-row`s.  No meld grouping. | **[code]** In the hand, `disabled` for discard, caption 「將不能打出」.  A drawn general: 「翻者翻到將時不能過，沒有其他吃法就收單將」 — an exposed `general_single`. |
| [leadurway/four-color-cards][leadurway] (hobby, React; says 「正宗客家二十張玩法」) | **[code]** `sortHandForDisplay` puts identical cards side by side, most copies first, then colour, then rank. | **[code]** In the hand.  The win solver has a `single_general` group named 「單張…」. |
| [Lapira "Tu sac"][lapira] | **[image]** One clipped row.  Order seen: 紅 相仕帥 · 黃 相仕帥 · three green cards (hidden by a banner) · 卒卒兵 · then 綠象, 紅傌, 白將.  **[inferred]** complete sets first, leftovers last. | **[image]** The lone 白將 is the last card of the row, among the leftovers, not set apart. |
| [Little Pencil "Tứ Sắc – Bài 4 Màu"][pencil] | **[image]** One clipped row: 白 象士將 · 綠 馬包車 · 白 馬包車 · 紅 傌傌 · 綠 士士 · 綠 將將 · 紅 相帥 · 白 馬象 · 黃 帥.  **[inferred]** sets, then pairs, then partial sets, then singles. | **[image]** 紅 相帥 are kept adjacent as a partial set; the lone 黃帥 is last.  In the hand throughout. |
| [daigia "Tứ sắc – Four Color"][daigia] | **[image]** One clipped row in colour blocks: 綠 包包象將 · 紅 仕仕仕(傌)帥帥 · 白 車象 · 黃 帥. | **[image]** In the hand, at the end of its colour block. |
| [SPN-TECH "Tu Sac – Four Color"][spn] (same art as [cherrylee][cherry]) | **[image]** One row, cards overlapping by about a tenth, whole card visible: 白將將將 · 綠 將士士象車馬 · 黃 帥相俥炮 · 白 包包 · 紅 仕相相傌 · 白兵. | **[image]** In the hand. |
| [Huno705/tu-sac][huno] (hobby) | **[code]** Wrapping flex row; `sortHand` by colour then Tướng-Sĩ-Tượng-Xe-Pháo-Mã, soldiers pushed to the end.  The comment gives the reason: 「để các bộ Tướng–Sĩ–Tượng và Xe–Pháo–Mã tiềm năng nằm cạnh nhau」 (so potential sets lie side by side). | **[code]** In the hand; a drawn one gets a 「Nhận Tướng」 button. |
| [canhcutcon/tu_sac_terminal][canh] (hobby, terminal) | **[code]** Four modes: by colour, by rank, a "khu vực xếp bộ" where the player builds groups, and free move/swap. | **[code]** `group_kind` names a one-card general group 「Tướng lẻ」, and only inside the player-built grouping mode. |
| [hongde88/four-color-cards-game][hongde] (hobby, Vietnamese rules) | **[code]** One absolutely positioned row, 40 px pitch, **free drag to reorder**; the order is the player's. | **[code]** In the hand. |

[vinagames][vina] describes "2 buttons that help you with the organization
of your private cards" and automatic melding; read through a summarising
fetch, not seen.

## The mark on generals

### Physical decks

| Deck | What the general carries | Other ranks |
| --- | --- | --- |
| Taiwan — [Commons scan][c-tw] (Outlookxp, 2013) and the maker [國瑜企業社][guoyu]'s own card images | **[image]** A cash coin between the two characters: a ring, inside it a square standing on one corner.  About as wide as one stroke group of the character.  Present on 帥 (red, yellow) and 將 (green, white) alike, absent from all 24 other cards. | **[image]** No centre mark.  Every rank has its own ornament in the black bar at each end: general a plain bar; 士 a bar notched at one side; 象 a diagonal cut; 車 a round tab in the middle; **馬 two round dots**; 包 two thin vertical lines; 兵卒 three slashes.  Chinese Wikipedia: 「上下兩端則會有像水滸牌的類似標記」 ([十胡][zh-10]). |
| Vietnam, short deck — card scans shipped in [binhtruong-sj/tu-sac][binh] | **[image]** 福 in an octagonal cartouche, with a small five-pointed star above and below it (a round dot instead of a star on the red 帥). | **[image]** A bare small character, no cartouche, no stars: 士 祿, 象 壽, 車 榮, 包 富, 馬 華, 兵 貴.  Matches [Pollett][pollett] on the short type: "a decorative motif, usually a Chinese character without any relation with the card's rank (for instance, the one shown by all the Generals reads 'happiness')". |
| Hong Kong, long — [Commons][c-hk] (天生公司) | **[image]** The maker's name 天生公司, the same as on 士 and 相. | 車馬炮卒 carry two fish instead. |
| Teochew, long — [Commons][c-tc] (恆發) | **[image]** The maker's name 恆發製造 / 香港恆發牌廠製造. | A different knot or flower on each rank. |
| Fuzhou, long — [Commons][c-fz] | **[image]** Nothing; centres are blank. | Nothing. |
| 廣水 (Hubei) — [Commons][c-gs] | **[image]** Nothing. | Nothing. |

So "generals are marked" is a Taiwanese and Vietnamese fact, not a general
one, and the two marks differ.  In the Hong Kong and Teochew decks the
general is, if anything, the *plain* card: it holds the maker's name where
the others hold a picture.

### Digital reproductions

- **Board Game Arena** **[code/assets]** — the sprite sheets
  ([Chinese][bga-cn], [chess symbols][bga-w], [K Q J 9 8 7 A][bga-pc]) redraw
  the Taiwanese deck: all seven end ornaments, and a coin (ring with a
  four-pointed star) in the centre of the four generals only.  The coin
  survives even in the two Western styles.
- **leadurway** **[code]** — `getDecoration` redraws the seven end ornaments
  as SVG.  The centre mark was planned (comment: "four dots for horse, coin
  for general") but `getCenterDecoration` now returns `null`: "removed at
  user request to increase simplicity and allow shorter heights".
- **hongde88** **[code]** — the general's inner panel gets
  `url('/images/star.svg')` (a black five-pointed star) in the centre and
  loses the centre rule the other cards have.
- **國粹天天樂, Lapira** **[image]** — both redraw the Taiwanese end
  ornaments (the slashes on 兵 are clear).  The hand is clipped above the
  centre, and no general appears full-height in the screenshots, so whether
  the coin is drawn is **not verified**.
- **SPN-TECH / cherrylee** **[image]** — full cards visible; every card has
  the same thin centre rule and the general has no extra mark.
- **chunhua1972, Huno705** **[code]** — no mark.  chunhua1972 prints one to
  four `·` under *every* tile as a copy index, which is worth knowing before
  using dots to mean "general".

## Terminology

| Term | Meaning | Source |
| --- | --- | --- |
| 將、帥單支 | a general on its own, 1 台 | [Chinese Wikipedia 十胡][zh-10]: 「將、帥單支為一台」 |
| 單張將／帥 | the same, as a named group | [BGA zh help][bga-zh]; leadurway 「單張…」 |
| 單將 / 將槓 | the single-general meld / general quad | chunhua1972 `MELD_NAMES` — a label in one hobby code base, not attested elsewhere |
| 一隻 | counter used for it | [國瑜][guoyu]: 「(將，帥)無論多少隻都以一隻一胡算」 |
| 軍落 | the 將士象 set | [zh-10] |
| 對子 / 開招, 開蕉 | pair / quad | [zh-10] |
| 孤兵卒 | a lone soldier in the dealt hand | [zh-10] |
| Tướng lẻ | lone general | canhcutcon `group_kind` |
| "A single king" | the same, as a valid set | [vinagames][vina] |
| Khạp / Quằn / Khui | concealed triple / dealt quad / quad made with the active card | [vinagames][vina]; 「KHẠP」 banner in the Lapira and Little Pencil screenshots |
| rác, "trash" | cards in no valid set | canhcutcon 「Rác / chờ」; [vinagames][vina] |

Not found: any Taiwanese (Hokkien) romanised term for the lone general, for
"meld" or for "partial set"; any generic Mandarin word for a partial set in
this game (sources list the shapes and say 牌組 or 組合).  單支 is presumably
read *tuann-ki* in Taiwanese, but no source says so.

Aside: the Chinese Wikipedia infobox romanises 十胡 as POJ *Cha̍p-hô͘*, TL
*Tsa̍p-hôo*, with an h; this repository writes *Tsa̍p-ôo*.  Not investigated
further.

## Caveats

- Store screenshots are single frames.  The sort rule behind them is read
  off one hand each; only the Board Game Arena, chunhua1972, leadurway,
  Huno705, canhcutcon and hongde88 rows rest on code.
- The hobby repositories are small and recent, several visibly
  machine-written; they show what a developer chose, not a tradition.
  Commits read: chunhua1972 `ee33cf0`, leadurway `940dd96`, Huno705
  `e26b087`, canhcutcon `83df231`, hongde88 `eaa7d5a`, binhtruong-sj
  `695ee45`.  Board Game Arena client version `231219-1040`.
- The Taiwanese coin rests on one Commons scan and the 國瑜 site images.
  Whether the scan is itself a 國瑜 deck, and whether other Taiwanese makers
  print the same coin, is unknown.
- The Vietnamese cartouche rests on one set of scans inside a repository
  (its red suit has an orange block painted over the centre) plus Pollett's
  sentence.  Pollett's own pictures could not be loaded.
- The other regional decks are one specimen each.
- No Hakka- or Teochew-specific program was found.  `sampot/pg-sisek` uses
  the deck for an unrelated five-card matching game and was left out.
- The World of Playing Cards pages on these decks were not reached; its
  "Hokkien" listing showed no relevant article.

## In the code

- `web/src/lib.rs` `arrange` runs `Solver::new(rules.concealed, Some(1))`
  over the hand and sends `Vec<Vec<CardJson>>`: one group per meld of the
  best layout, then the loose cards as a last group.
- `web/app.js` draws each group as a `.group` box; `web/style.css` puts
  0.25 rem inside a group and 0.9 rem between groups.
- A general is `Meld::Single`, so today it always forms **its own one-card
  group**, and a 士 or 象 of its colour lands in the loose group at the far
  end.  The pair 將士 is never shown together.
- `CardJson` is `code`, `glyph`, `color`.  `code` is colour letter plus rank
  letter, and the general's rank letter is `K`, so `[data-code$="K"]`
  already selects generals in CSS.
- A card face is one centred glyph in a strip with a thin inner border; the
  inverted second glyph and the end ornaments of the real card are not drawn.

## Implications for web/

**Put the lone general in the hand.**  Every implementation does, and the
two Taiwanese ones place it at the head of its colour, which is exactly
where the 將士 / 將象 waits become visible.  Concretely, in `arrange`:

- keep a general in its own group only when the layout has nothing of its
  colour to go with it;
- otherwise emit general + loose 士／象 of the same colour as one group
  (a partial 將士象), ahead of the remaining loose cards.

A smaller step with the same effect: stop emitting `Meld::Single` groups and
sort the generals to the front of the loose group by colour.  Going all the
way to a flat colour-then-rank row would match the other programs but throw
away the meld grouping, which none of them offers and which is this front
end's own strength; the evidence does not ask for that.

**Mark the general, since it will no longer be recognisable by standing
alone.**  The mark has a clear precedent, but it is not dots beside the
glyph:

- Taiwanese cards and Board Game Arena: one coin, centred, below the
  upright character.
- Vietnamese cards: a framed 福 with one star above and one below.
- Two round dots at each end are the Taiwanese mark of the **horse**, so a
  pair of dots would point a player who knows the deck at the wrong rank.

`text-emphasis` can still carry it if it draws one mark under the glyph
rather than dots beside it, for example
`.card[data-code$="K"] { text-emphasis: "◎"; text-emphasis-position: under right; }`
or the keyword `double-circle open`.  With a single glyph per card, "under"
is where the coin sits relative to the upper character on the real card.
Two things to check before relying on it: the emphasis mark is drawn at half
the font size and adds to the line box, which may shift the glyph off centre
in the flex-centred `.card`; and it scales with `font-size`, so it shrinks
to about 0.45 rem on the small meld and discard cards.  A `::after`
pseudo-element (the `::before` is already the inner border) containing ◎ or
a small ring with a rotated square gives the same picture with full control
over size and position.

Whichever way, mark generals **everywhere** they can appear — exposed
melds, the offer, the result sheet, the rules table — not only in the hand: on the
physical card the coin is part of the face, not a hint about the layout.
The fact that a general already counts 1 胡 could then be left to the
grouping (alone, or inside 將士象) rather than to its position.

[bga-panel]: https://en.boardgamearena.com/gamepanel?game=fourcolorcards
[bga-js]: https://x.boardgamearena.net/data/themereleases/current/games/fourcolorcards/231219-1040/fourcolorcards.js
[bga-css]: https://x.boardgamearena.net/data/themereleases/current/games/fourcolorcards/231219-1040/fourcolorcards.css
[bga-cn]: https://x.boardgamearena.net/data/themereleases/current/games/fourcolorcards/231219-1040/img/chinese_cards.png
[bga-w]: https://x.boardgamearena.net/data/themereleases/current/games/fourcolorcards/231219-1040/img/western_cards.png
[bga-pc]: https://x.boardgamearena.net/data/themereleases/current/games/fourcolorcards/231219-1040/img/playing_card_style.png
[bga-zh]: https://zh.doc.boardgamearena.com/Gamehelpfourcolorcards
[gc]: https://apps.apple.com/app/id1566814309
[gc-shot]: https://is1-ssl.mzstatic.com/image/thumb/PurpleSource221/v4/c8/c4/00/c8c400ce-2827-1efb-e676-db7d0199bda3/10-.jpg/2048x2048bb.jpg
[chunhua]: https://github.com/chunhua1972/four-color-game
[leadurway]: https://github.com/leadurway/four-color-cards
[huno]: https://github.com/Huno705/tu-sac
[canh]: https://github.com/canhcutcon/tu_sac_terminal
[hongde]: https://github.com/hongde88/four-color-cards-game
[binh]: https://github.com/binhtruong-sj/tu-sac
[lapira]: https://play.google.com/store/apps/details?id=com.lapira.tusac.android
[pencil]: https://play.google.com/store/apps/details?id=com.littlepencilstudio.tusac.sisepai.android
[daigia]: https://play.google.com/store/apps/details?id=com.daigia.card.tusac
[cherry]: https://play.google.com/store/apps/details?id=com.cherrylee.tusac_multiplatform
[spn]: https://apps.apple.com/app/id581243087
[vina]: https://www.vinagames.com/tusac.html
[guoyu]: https://www.guoyu.com.tw/zh/qa/1.html
[pollett]: https://www.oocities.org/a_pollett/cards18.htm
[zh-10]: https://zh.wikipedia.org/wiki/十胡
[c-tw]: https://commons.wikimedia.org/wiki/File:台灣的四色牌.png
[c-hk]: https://commons.wikimedia.org/wiki/File:香港的四色牌.png
[c-tc]: https://commons.wikimedia.org/wiki/File:潮州四色牌.png
[c-fz]: https://commons.wikimedia.org/wiki/File:福州的四色牌.tif
[c-gs]: https://commons.wikimedia.org/wiki/File:廣水四色牌.png
