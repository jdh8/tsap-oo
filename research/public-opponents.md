# 公開十胡／四色牌實作：作為 tsap-oo 對手的可行性

查核日期：2026-11-15。問題：網路上是否有現成的十胡或四色牌電腦玩家，
能與本 engine 的 `HeuristicBot` 對打？本文查的是可取得的實作、策略入口與
接入成本，沒有執行第三方程式，也沒有測得任何候選的勝率。
本地比對基準為 `62b04efa2b96ae2e22df1c12645562de79de3f20`。

## 結論

**有公開的電腦玩家程式碼，但這次沒有確認到可直接接上 tsap-oo 的完整對手。**
主要障礙是規則差異、沒有共通 bot 協定，以及部分專案沒有明示原始碼授權。
「公開 GitHub repo」、「有 AI」與「可直接移植、可公平評測」要分開判斷。

- **技術上優先評估 `chunhua1972/four-color-game`。** 它把策略拆成純
  TypeScript 的 `chooseAiCommand(observation, difficulty, randomIndex)`，
  觀測內已有自己手牌、公開資訊與合法動作，最接近本專案的 `Strategy`。
  但目前未找到明示授權，且它的兵卒對、將槓與強制動作規則不同。
  [策略][ch-ai]、[規則][ch-preset]、[目錄][ch-tree]
- **若要求有明確寬鬆授權，`binhtruong-sj/tu-sac` 值得研究，但應視為
  策略抽取／修復候選。** 它有 MIT 授權、Julia 棄牌評分及較舊的吃碰策略；
  預設載入的 `tsGUI.jl` 卻有空的 bot 函數，不能把 repo 中的 AI 名稱當成
  「已能完整自動對局」的證據。[授權][binh-license]、[主程式][binh-main]、
  [舊程式][binh-old]
- **`canhcutcon/tu_sac_terminal` 適合補充一種不同的啟發式。** Python
  Medium bot 用最少散牌與已公開牌的數量決策，有吃牌和棄牌入口；不過原遊戲
  以「沒有散牌」為勝利條件，沒有本 engine 的十胡門檻，且未找到明示授權。
  [bot][canh-bot]、[規則實作][canh-rules]、[專案設定][canh-project]
- **`Zizuixixiang/cedarduet` 提供另一個清楚的弱基準。** Python 的
  `choose_local_npc_action` 確有策略與相關測試原始碼，但採 PolyForm
  Noncommercial 1.0.0，屬有非商業限制的公開源碼候選，不能當作 Apache-2.0
  專案可直接納入的寬鬆授權依賴。[策略][cedar-ai]、[測試][cedar-tests]、
  [授權][cedar-license]
- **不要把多人遊戲誤列為現成對手。** `Huno705/tu-sac` 是兩人連線遊戲，
  `hongde88/four-color-cards-game` 要四位玩家；已檢視的說明及主要遊戲程式
  未提供可抽出的電腦策略。它們可供規則或 UI 比對，這項需求的順位較低。
  [Huno 說明][huno-readme]、[Huno 引擎][huno-engine]、
  [hongde 說明][hongde-readme]、[hongde 房間實作][hongde-room]

這些是**接入可行性的排序，不是棋力排名**；沒有資料證明任何候選比目前的
`HeuristicBot` 強。[本地 heuristic](../src/heuristic.rs) 已會估計胡數缺口、
散牌、搭子及未見牌，不能因其他專案把 bot 稱作「AI／Medium／Hard」就認定
它更適合當強對手。

## 評估基準：本 engine 需要什麼

本專案採台灣十胡：四人、112 張牌、莊家 21 張其餘 20 張、十胡門檻、六張
廢牌；公開摸牌不直接收入暗手。胡與槓由 `Round` 強制處理，策略只回答吃／碰／
過以及棄牌；將帥不可丟、剛吃入的牌鎖住、暗三張不能拆來吃牌、吃完必須還有
合法棄牌。普通對子含兵卒對，將帥各自作一張牌組，不作對子或碰。
[README](../README.md)、[規則值](../src/rules.rs)、
[回合邏輯](../src/round.rs)

對手最小介面是：

```rust
fn claim(&mut self, view: &View<'_>, options: &ClaimOptions) -> Claim;
fn discard(&mut self, view: &View<'_>) -> Card;
```

`View` 只公開本家手牌、各家明組與手牌張數、棄牌、當前公開牌、剩餘牌數及
規則；不公開他家手牌和牌堆順序。`Table::step` 將回覆送回 `Round` 驗證，
非法動作會返回錯誤。現有 arena 是同一個 Rust 行程內的四個 `HeuristicBot`，
並沒有已實作的跨語言、標準輸入輸出或網路 bot 協定。
[Strategy](../src/strategy.rs)、[View](../src/view.rs)、
[Table](../src/driver.rs)、[arena](../examples/arena.rs)

`ClaimOptions::blocked` 是被阻擋吃法的說明，不能當合法動作輸出；
`forced()` 為真時不准過。棄牌清單應來自
`discardable(view.hand(), view.locked())`，它排除將帥及所有同種鎖牌。
`View` 目前沒有全場事件流、過牌歷史或累積比分；需要這些資料的外部策略，
不能宣稱只做快照格式轉換就能保持原行為。
[ClaimOptions／discardable](../src/round.rs)、[View](../src/view.rs)

因此可重用的重點是「**在本 engine 給的合法選項中做選擇**」的策略。兩套
遊戲引擎各自發牌、判胡與結算，不能直接產生有意義的對戰勝率。以下所有
「適合／成本／優先度」都是根據原始碼作的工程判斷，並非作者承諾。

## 核心候選一覽

| 專案 | 已確認的電腦策略 | 接入判斷 | 查得的授權 | 已查版本／最後提交日期 |
| --- | --- | --- | --- | --- |
| [chunhua1972/four-color-game][ch-tree] | 純 TS `chooseAiCommand`；Easy 隨機、Normal 評估手牌潛力與明胡 | **優先技術候選**；已有 observation／合法動作分界 | 未找到 LICENSE；API license 為 null | [`ee33cf0`][ch-commit]，2026-10-06 |
| [canhcutcon/tu_sac_terminal][canh-tree] | Python `Bot.decide`、`advise_discard`、`advise_claim` | **補充基準候選**；策略容易辨識，需移除原規則權威與十胡不合處 | 未找到 LICENSE；`pyproject.toml` 未列授權 | [`83df231`][canh-commit]，2026-11-13 |
| [binhtruong-sj/tu-sac][binh-tree] | Julia `play1Card`、舊版 `gpHandlePlay1Card`／`gpHandleMatch*` | **可授權的策略研究候選**；主路徑有 stub、狀態與 GUI／網路耦合 | [MIT][binh-license] | [`695ee45`][binh-commit]，2025-12-02 |
| [Zizuixixiang/cedarduet][cedar-tree] | Python `choose_local_npc_action`；優先勝利、較大吃牌組與少重複牌棄牌 | **非商業評測的弱基準候選**；輸入可分離，但 BGA 規則需適配 | [PolyForm Noncommercial 1.0.0][cedar-license] | [`31e0bfe`][cedar-commit]，2026-11-14 |
| [leadurway/four-color-cards][lead-tree] | React 內 `runComputerTurn`、`executeComputerDiscard` | **低優先**；有 bot，但傳統模式策略簡單、規則差異大、與 UI 混合 | 未找到 LICENSE；API license 為 null | [`940dd96`][lead-commit]，2026-08-28 |
| [tw0961177543-cpu/sisepai][sise-tree] | 單檔 HTML 的 `aiAnswer`、`pickDiscard`、`aiWantsPeng`、`aiChooseChi` | **低優先**；手牌啟發式與隨機策略，原玩法差異大 | 根目錄／已查 tree 未找到授權檔 | [`eac82fc`][sise-commit]，2026-11-08 |
| [Huno705/tu-sac][huno-tree] | 已讀 README／主要引擎未見 bot | 兩位真人連線；不是現成對手 | 未找到 LICENSE | [`e26b087`][huno-commit]，2026-07-19 |
| [hongde88/four-color-cards-game][hongde-tree] | 已讀 README／房間與玩家模型未見 bot | 要四位玩家；不是現成對手 | 未找到 LICENSE | [`eaa7d5a`][hongde-commit]；API 最後推送 2021-06-06 |

表內日期是來源的活動紀錄，不代表目前有人維護或本機可成功啟動。未找到授權
指本次檢視的根目錄、README／套件設定及 GitHub metadata 沒有明示授權；
不能據此當成 MIT 或 Apache 專案直接納入、散布原始碼。日期與 license
metadata 可由各 repo 的 [chunhua][ch-api]、[canhcutcon][canh-api]、
[binh][binh-api]、[leadurway][lead-api]、[Huno][huno-api]、
[hongde][hongde-api] API 重查；引用的程式碼固定在已查 commit。
公開 repo 與授權的差別也見 [GitHub 的官方說明][github-license-doc]。

## 候選細節

### chunhua1972/four-color-game：最小的策略接入面

`packages/game-core` 的 README 明確將核心與 React、Supabase、Node 分離。
實際的 `AiObservation` 包含 `ownHand`、`public`、`seat`、`legalOptions`；
`chooseAiCommand` 直接回傳一個候選動作，亂數由參數注入。Normal 棄牌時
比較移除各種牌後的 `scoreHandPotential`，吃碰時比較吃牌後最佳棄牌的
潛力與明胡；Easy 從選項抽樣。這不是搜尋型或訓練型模型。
[README][ch-readme]、[型別][ch-types]、[策略][ch-ai]

最小接法是把 `View` 與本引擎生成的合法動作轉成其 observation，讓它只做
選擇。避免連其 `enumerateLegalClaims`、`evaluateHu` 或整個 Supabase 遊戲
一起移植。原專案用 112 張實體牌 ID，本專案用 28 種牌與張數；adapter
必須穩定分配同種牌的副本 ID，並把回傳的實體 ID 對回本地牌種。
[型別][ch-types]、[策略][ch-ai]、[牌編碼][ch-catalog]、
[本地 View](../src/view.rs)

確定要處理的差異：

- `allowSoldierPair: false`，而本 engine 允許兵卒對；外部策略的手牌評分
  對兵對也不加一般對子的分數。
- `allowPassHu: true`、`mandatoryKong: false`；本 engine 在呼叫策略前就
  已完成強制胡／槓，不應把這兩種選擇再交給外部 bot。
- 將槓是暗 6／明 8，本 engine 是暗 8／明 6；它有點花，本 engine 沒有。
- 其吃牌列舉可用手中三張的一部分組牌，本 engine 禁止拆暗三張吃牌；不能
  把外部列舉結果當成本 engine 的合法選項。
- 即使傳入本地合法選項，策略在估計吃牌後的最佳棄牌時，也只排除將帥，
  沒有本地的同種鎖牌限制。因此可以保持輸出合法，卻仍高估某些吃法；
  若修正這段評估，須將結果標為適配版策略。

來源：[preset][ch-preset]、[牌型／分值][ch-patterns]、[吃牌列舉][ch-claims]、
[策略][ch-ai]、[本地規則](../src/rules.rs)、[本地回合](../src/round.rs)。

作者提交了測試與模擬報告，README 宣稱包含合法動作、守恆等驗證；本次只作
原始碼與說明的靜態查核，沒有重跑，也沒有用它們證明對打強度。取得可用授權後，這是
最適合先做小型 adapter 的候選。[README][ch-readme]

### canhcutcon/tu_sac_terminal：可解釋的散牌策略

Medium 的 `advise_discard` 對合法棄牌逐一比較剩餘散牌數，平手再用
`usefulness`：自己持有的搭配牌與尚未公開的搭配牌越多，越傾向保留。
`advise_claim` 優先勝利／強制動作，之後選散牌最少、取牌最多的組合，
只有減少散牌才自願吃牌。Easy 從散牌或合法動作抽樣。這提供與十胡估分
不同的基準風格。[bots.py][canh-bot]

它的 README 說 bot 只看自己與公開資訊，但 `Bot.decide` 實際接受整個
`Engine`，不是型別上受限的 observation。已讀的估價程式用本家手牌與
`public_seen`；若接入，仍應重新建立受限輸入，不能把其他手牌傳過去。
原規則的 `is_win` 只檢查散牌為零，`score_round` 對贏家加一勝，無十胡
計分；預設 `training_basic` 明說是練習規則，允許拆 Khạp、無強制吃對、
牌堆用完才流局，家規 profile 也不是已驗證的台灣十胡。
[README][canh-readme]、[bot][canh-bot]、[規則][canh-rules]、
[training profile][canh-training]、[家規 profile][canh-family]

最小接法應抽取散牌分析／動作排序，讓 `Round` 決定可吃可丟的集合。若為了
十胡門檻重新加入胡數目標，應把它標成「改寫後的策略」，不能再用原專案
名稱暗示是未改動的原版對手。明示授權仍待解決。

### binhtruong-sj/tu-sac：MIT，但不是現成的無介面 bot

這是本次核心候選中有明確 MIT LICENSE 的專案。`tsGUI.jl` 的
`TuSacManager.play1Card(player)` 有隨機、按牌計數抽樣、已見牌評分及
權重表等分支，`processList!` 會考慮搭子、公開牌計數與 dead-card 標記。
較舊的 `tsGUI1.jl` 也保留棄牌與配牌入口。這些是實際策略程式碼，而不只
README 的 AI 宣稱。[LICENSE][binh-license]、[主策略][binh-discard]、
[舊策略][binh-old-ai]

但需要特別分清檔案與啟動路徑：

- `runTUSAC.jl` 最後載入 `tsGUI.jl`。它在載入前可向硬編碼伺服器查版本、
  覆寫本地 `tsGUI.jl`；這不是穩定、可重現的 benchmark 入口。
  [啟動器][binh-launcher]
- 已查 `tsGUI.jl` 的 `TuSacManager.Match2Card`、`Match1or2Card` 是
  `return []`，全域 `gpHandlePlay1Card`、`gpHandleMatch2Card`、
  `gpHandleMatch1or2Card` 也一樣。`hgamePlay` 呼叫的是後者，不能拿
  `play1Card` 有內容來推論預設路徑已接好。[配牌 stub][binh-match-stub]、
  [全域入口與呼叫][binh-entry]
- `server.jl` 是傳送更新程式的伺服器；`runTUSAC_server.jl` 只是複製主檔
  再由 GameZero 載入，並非另有一個乾淨 bot service。
  [server][binh-server]、[server 啟動器][binh-server-launcher]
- `tsGUI1.jl` 的 `gpHandlePlay1Card`／`gpHandleMatch*` 有實質分支，
  但它是另一版單檔 GUI 程式，不是獨立函式庫；本次沒有驗證其解析、依賴
  或完整對局。[舊版][binh-old-ai]

其 Tứ Sắc 規則以散牌歸零判勝，另外有不增加散牌、不可拆 Khạp 等規則；
不能當成台灣十胡的同規則引擎。介面亦可碰到 `all_hands`、`gameDeck`
等全域狀態，抽取策略時必須逐一確認每項依賴可由 `View` 的公開資訊重建。
[規則筆記][binh-rules]、[判勝][binh-win]、[主程式][binh-main]

建議先在固定 commit 上選一個實際有內容的棄牌策略作基準，再補接吃碰選擇；
需要補寫的部分須記錄在名稱與評測報告內。若必須保持原版整套行為，應先
完成獨立運行的確認，現在不能聲稱已具備可比較的原版 bot。

### Zizuixixiang/cedarduet：有清楚的 Python NPC，授權有限制

`BgaTenHu.choose_local_npc_action(state, actor, participants)` 會先找
至少十胡的勝利選項，選胡數最高者；否則在吃碰槓中先選使用最多手牌的組合，
再比較組合胡數；棄牌則優先丟本家重複張數最少的牌，以牌 ID 打破平手。
`_options` 用本家手牌與公開狀態產生選項，專案另有公開／私有狀態投影。
這是一個可理解、可能容易抽取的弱基準，沒有棋力保證。[策略][cedar-ai]

原專案附有「更換他家手牌及牌堆不應影響 NPC」、「完整 NPC 對局的合法性與
守恆」、「真人／NPC 混合房間」等測試。本次確認的是測試程式碼存在，
沒有執行這些測試，也沒有證明此 bot 強於 `HeuristicBot`。[測試][cedar-tests]

其文件與程式採 BGA 台灣版規則：禁兵對、將槓暗 6／明 8；回應順位中槓與
碰同級，可以過胡／過槓，牌堆用到空，結算公式另含點花。這些都與本 engine
不完全相同，不能讓它原本的合法動作列舉或分數取代 `Round`。
[規則][cedar-rules]、[遊戲實作][cedar-ai]、[規則文件][cedar-doc]

其 LICENSE 明確是 PolyForm Noncommercial 1.0.0。若用途符合該授權，可
把它列為獨立評測候選；對一般重用／散布需求，不能把它與 MIT 候選等同。
這裡只記錄來源授權內容，沒有對特定商業整合提供法律結論。
[LICENSE][cedar-license]

### leadurway/four-color-cards：有 AI，但傳統模式的價值有限

這個 repo 同時做抓對子與傳統吃碰。不能把 10／15 張抓對模式的困難 AI
當成十胡對手：傳統模式在 `runComputerTurn` 中有胡就胡、可槓就槓、
以 75% 機率碰；`executeComputerDiscard` 優先丟非將帥的第一張單牌，
否則丟第一張非將帥。策略綁在 React state、計時器與記錄 UI 裡，沒有
獨立的 observation 函式。[README][lead-readme]、[回應策略][lead-claim]、
[棄牌策略][lead-discard]

原程式只有玩家與電腦兩席；傳統模式會把摸牌收入手牌、槓後補摸。其
`solveHu` 的異色組不限兵卒，車馬炮算 2 胡，與本 engine 的公開摸牌、
無補槓摸牌及胡數／牌型定義不一致。抽取策略的工程量可能比其簡單策略的
評測價值還大，且未找到明示授權。[App][lead-app]、
[牌型與判胡][lead-utils]、[指南][lead-guide]

### tw0961177543-cpu/sisepai：有手牌啟發式，規則適配成本較高

這是給家庭使用的網頁單檔，`index.html` 裡有 `usefulness`、`pickDiscard`、
`aiKeep`、`aiWantsPeng`、`aiChooseChi` 與 `aiAnswer`，會依本家手牌連結
與 `Math.random` 決策。[README][sise-readme]、[策略][sise-ai]

預設雖是四人、20 張，但 `openDraw: false`、`pairs: false`、
`kaiAdd: true`、`minPts: 0`；將士象 1 分、四色卒 4 分、四張 6 分，
且 `pickDiscard` 本身未排除將帥。這些與十胡門檻、公開翻牌、禁加槓及
`discardable` 的差異，使它不能直接套一個 JSON 外殼就成為合法對手。
本次未找到授權檔，技術優先度低於 chunhua。
[預設規則與 AI][sise-ai]、[repo tree][sise-tree]

### Huno705 與 hongde88：可玩多人遊戲不等於電腦對手

Huno 的說明把流程寫成兩人建房、加入並操作，主要引擎是 `applyAction`；
本次未在讀取的主要檔案找到 bot 選動作入口。其兩人規則、七張牌流局、
無散牌即可勝、將士象 1 分與兵組 1／4 分，也不能直接與本 engine 互認
結果。[README][huno-readme]、[引擎][huno-engine]

hongde 的 README 明寫需要四位玩家，`Room` 用 Socket.IO 玩家建房與發牌，
未提供電腦座位策略。可以讀 UI 與連線流程，但不能在沒有另寫 bot 的情況下
當 arena 的自動對手。[README][hongde-readme]、[Room][hongde-room]

## 其他公開線索與排除項

| 來源 | 查得內容 | 為何沒有列為可接入對手 |
| --- | --- | --- |
| [Board Game Arena 四色牌][bga] | 官方提供 2–4 人線上遊戲，開發者 xnedkx；有公開說明頁 | 沒有取得可呼叫的 bot 介面或完整 server／bot 原始碼；公開 client 與可玩網站不等於公開 AI |
| [Lapira Tứ Sắc][lapira] | 作者 Google Play 頁明確提到離線及虛擬對手 | 確認有電腦對手產品，但未找到原始碼或可程式呼叫介面 |
| [Vinagames Tứ Sắc][vina] | 作者網站提供線上多人遊戲與玩法 | 未找到可供本 engine 呼叫的 bot 介面 |
| [添丁四色牌][itch] | 作者發布 Unity HTML5 mini game | 公開頁不足以確認十胡同規則、AI 入口及可重用原始碼 |
| [chengr4/four-color-cards][cheng] | `03a1430`：README TODO、`go.mod`，沒有遊戲程式 | 骨架，不是 engine 或 bot |
| [god00612/FourColorCardGame][god] | `bd6c9a2`：README 明說金額統計工具 | 記分工具，不是打牌策略 |
| [minhhoa11111999/tu-sac-app][minh] | `e89e66b`：HTML 記分、登入與歷史功能 | 未見對局策略 |
| [timmykhoi1813-png/Tu-sac-online][timmy] | `afc02f9`：Socket.IO 多人 demo；`hasWin` 以手牌不多於四張且同色判勝 | 未見 bot，而且勝利條件與十胡不符 |
| [binhtruong-sj/bettaTuSac][betta] | `48c20b3`：支援頁與隱私頁 | 沒有策略程式碼 |
| [TuSac/MapMahJongg][map] | `87647f9`：README、LICENSE、`.gitignore` | 即使有 GPL-3.0 授權，已查 repo 仍沒有 bot 實作 |
| [sampot/pg-sisek][sampot] | 有 MIT 授權，使用四色牌牌具的五張配對得分遊戲 | 規則目標不同，不是本次要找的十胡／Tứ Sắc 對手 |

BGA 可作規則比對來源，但其中文說明禁兵對、將槓暗 6／明 8，並有點花；
本地 `Rules` 只有門檻、廢牌與一般牌型分值，沒有每個 rank 的槓分或兵對
開關，因此不能宣稱只換 `Rules` 數字就能完整對齊 BGA。
[BGA 規則][bga-help]、[本地 Rules](../src/rules.rs)

## 最小接入與驗證方案

以下是建議的下一步，**本次沒有實作**。

1. **先選一個候選與確切版本。** 技術驗證可先選 chunhua 的 Normal，
   但直接重用原始碼前須釐清授權；若限定 MIT，先抽查 binh 的一個完整
   策略路徑，不能從預設啟動器開始就假設可用。
2. **只接 `Strategy` 邊界。** 同語言移植或常駐子行程擇一即可。若使用
   子行程，簡單的逐行 JSON 請求／回覆已足夠；它是要新增的 adapter，
   不是本專案或任何候選已支援的協定。請求只帶 `View` 可見資料與合法
   動作 ID，對方回覆一個 ID。
3. **讓 `Round` 保持唯一規則權威。** 強制胡、強制槓、將帥必吃、鎖牌、
   暗三張、相公限制及十胡門檻，都以本 engine 給的選項為準。抽取原策略
   時，可先保留原評分來觀察規則落差，再另列調整版；不要混算兩者結果。
4. **先驗證適配，再比較棋力。** 用固定局面確認牌種／副本映射、吃牌與
   摸牌來源、將帥強制吃、鎖牌及沒有合法棄牌的終局。以相同可見資訊、
   不同隱藏牌內容確認 bot 不受隱藏牌影響。遇到非法輸出、逾時或 fallback
   要計數，不能悄悄替它選牌後仍算作原策略勝率。
5. **利用現有 arena 做四人實驗。** 先採一席候選對三席 `HeuristicBot`，
   輪換所有座位與莊家，用固定、可重放的牌序及多個 seed；另做兩種策略
   各佔兩席、仍各自競爭的混合配置。記錄勝率、每局淨胡、流局／相公率、
   非法動作率與決策時間。
   報告樣本數與不確定性，不用一組 seed 或只看贏家平均胡數下棋力結論。

上述輸入與合法性要求來自本地 [Strategy](../src/strategy.rs)、
[View](../src/view.rs)、[Round](../src/round.rs)、[Table](../src/driver.rs)；
arena 的擴充點是 [建立四席 bot 的位置](../examples/arena.rs)。

## 搜尋範圍與限制

從既有[其他實作的手牌排法研究](hand-layout-and-general-marks.md)的候選
出發，重新查核來源；另外搜尋 `十胡`、`四色牌`、`four-color-cards`、
`Four Color Cards`、`Si Se Pai`、`Tứ Sắc`、`Tu Sac`、`tusac`，以搜尋引擎
與 GitHub repository search 找 repo，再讀作者 README、程式碼、授權檔
及 commit／repository metadata。沒有把搜尋摘要或 GitHub 專案名稱當成
AI 存在的充分證據。

也用搜尋引擎補查 GitLab、Gitee、SourceForge，未找到更多能以來源確認的
自動對手；找到的 SourceForge 四色牌相關素材不能當成策略。排除 UNO／
四色撲克牌配色、四色定理／地圖著色及名稱相似的麻將程式。

本次是原始碼靜態查核：沒有安裝候選依賴、執行候選測試、連上第三方遊戲
伺服器、對商用程式做逆向工程或測量棋力。未找到「即接即用」不代表網路上
完全不存在其他實作；未公開、未被索引的專案與只有執行檔的舊遊戲，可能
不在這次結果內。文件中的「沒有 bot／授權」應讀成已查範圍內未找到，
不是對作者未公開程式或權利狀態作斷言。

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
