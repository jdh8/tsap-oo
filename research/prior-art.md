# 先前研究：麻將與 rummy 類 AI

查核日期：2026-11-16。問題：`doc/stronger-bot.md`（`3aa5b0c` 時的版本）列出
的四條路線，在麻將、gin rummy 與其他摸打類遊戲的文獻裡有沒有先例？哪些能
搬到一個手寫評分加 Monte Carlo、沒有深度學習管線的十胡 bot？十胡有沒有兩人、
三人的成文規則？「先做兩人版再擴到四人」有沒有先例與證據？本文只讀文獻與
公開原始碼，沒有跑任何對局，也沒有改動 bot。

## 結論

- **最值得先做的是量測，不是打法。** 現在的 A/B 已經是文獻裡的 duplicate
  （同一副牌、有無候選各打一次），剩下的雜訊全在「兩邊打法分歧之後」。
  backgammon 的 rollout 與撲克的 DIVAT／MIVAT／AIVAT 處理的正是這一段：
  從分歧點把剩下的牌堆重洗多次取平均，再扣掉每一張摸牌的「運氣」。
  [GNU Backgammon 手冊][gnubg-ro]、[DIVAT][divat]、[MIVAT][mivat]、
  [AIVAT][aivat]
- **做對手推論之前，先量「看得到別家手牌的 Monte Carlo」能贏多少。** Scrabble
  與 gin rummy 的研究都先量這個上限：上限小，推論就不必做。鬥地主的結論是
  完美的對手模型只比均勻信念好一點。[Richards & Amir 2007][scrabble]、
  [Kelidari et al. 2026][gold]、[Cowling et al. 2012][ismcts]
- **「不餵下家」是 rummy 類裡有比賽結果支持的對手資訊用法。** EAAI 2021
  gin rummy 比賽的冠軍 MARJJ，棄牌評分就是「自己手牌的價值」加上「這張牌對
  對手有多大用處」的加減分；把其中一組歸零，對原設定的勝率就掉到 47% 到
  48%。
  [Ahmed et al. 2021][gr-marjj]、[比賽結果][gr-results]
- **麻將 AI 的對手模型幾乎都為了算放銃的期望損失，這一塊搬不過來。** 日麻
  放銃者付全額；十胡放銃不多賠，`doc/stronger-bot.md` 也已量到防守不值錢。
  能搬的是另外兩樣：單人期望值的動態規劃，以及把對手換成「每一巡有多少機率
  結束這一局」的環境。[Mizukami & Tsuruoka 2015][mizukami]、
  [Kurita & Hoki 2019][kurita]、[tomohxx 和了確率][tomohxx-prob]
- **兩人版查得到「可以玩」，查不到「怎麼改規則」。** 中文維基寫 2 到 6 人、
  Board Game Arena 提供 2 到 4 人且排位預設兩人，發牌都是每人 20 張、莊家
  21 張，沒有任何來源寫出人數少時的規則調整。先例顯示演算法能從兩人搬到
  多人，打法不保證能搬；兩人麻將的評估仍然用了 20 萬局。
  [中文維基〈十胡〉][zhwiki-shihu]、[BGA 說明][bga-zh]、[Liu et al. 2023][olss]
- **沒有找到任何以四色牌或十胡為題的 AI 論文或學位論文。** 公開的實作與 bot
  見 [`public-opponents.md`](public-opponents.md)。

## A. 麻將 AI

| 系統 | 方法 | 用到手牌以外的什麼 | 怎麼量強度 |
| --- | --- | --- | --- |
| [Suphx][suphx] | 五個 CNN（打牌、立直、吃、碰、槓），監督學習後自對局強化學習 | 各家棄牌、累計分數、立直；訓練時還看得到別家手牌與牌山 | 離線 100 萬局對三個固定對手；天鳳 5,000 多局 |
| [Mizukami & Tsuruoka][mizukami] | 單人麻將的打法 + 三個對手模型 + Monte Carlo | 各家棄牌、副露、巡目，擬合自鳳凰卓牌譜 | 同牌山 duplicate 1,000 局；天鳳 2,634 局 |
| [Kurita & Hoki][kurita] | 把四人麻將抽象成四個單人 MDP，對手換成機率環境 | 對手和牌、放銃的機率，取自牌譜 | 3,557 局，同一局內與 Bakuuchi 的名次差 |
| [akochan][akochan] | C++ 明確算每個選擇的期望點數 | 每家的聽牌機率、每張牌的放銃機率與打點 | 同 seed 換起家的自對局 |
| [Mortal][mortal] | 深度強化學習 | 文件多數頁面空白，未能查證 | 1 對 3 duplicate，同 seed 四個座位各打一次 |
| [kanachan][kanachan] | 雀魂牌譜的模仿學習加分階段微調，不用人工特徵 | 整局的牌譜序列 | 未能查證 |
| [ACH／JueJong][ach] | actor-critic 版的加權 CFR，自對局，以 Nash equilibrium 為目標 | 對手的棄牌與副露 | 對規則 bot 各 10,000 局；對人類冠軍 1,000 局 |
| [Botzone 國標麻將][oim] | 比賽：啟發式、監督學習、強化學習 | 視隊伍而定 | 瑞士制加 duplicate，同牌山 24 種座位排列 |

### Suphx

- **Oracle guiding。** 先訓練一個看得到別家手牌與牌山的 oracle，再把這些
  「完美特徵」用機率遞減的 dropout 逐步關掉，讓它變成正常 agent。作者說直接
  蒸餾行不通：資訊少的 agent 模仿不了資訊多的。[§3.3][suphx]
- **Global reward prediction。** 一個 GRU 從各局的資訊預測整場的最終獎勵，
  以相鄰兩局預測值的差當作該局的獎勵。這是「場」對「局」的功勞分配，十胡
  一局就結算，用不上。[§3.2][suphx]
- **pMCPA。** 每局起手後，固定自己的手牌，隨機發別家與牌山，用離線策略
  打完若干條軌跡，再用這些軌跡微調策略。離線實驗用 10 萬條軌跡，微調後對
  未微調的版本勝率 66%，只測了數百局；因為太慢，天鳳上的 Suphx 沒有用它。
  [§3.4、§4.3、註 8][suphx]
- **前瞻特徵。** 一百多個特徵來自深度優先搜尋：「打掉某張牌，再換 n 張，
  能不能和出 S 點的牌」。搜尋時不看對手，只考慮自己的摸與打。這等於把
  路線三（單人期望值）當成評分的輸入。[§2][suphx]
- **評估。** 離線隨機產生 100 萬局，每個 agent 都在這同一批牌上對三個固定
  的弱 agent；指標是天鳳的安定段位，再從 100 萬局抽 80 萬局重複 1,000 次看
  離散程度。作者寫明這是為了壓掉配牌的變異。天鳳實戰 5,000 多局，安定段位
  8.74。[§4、§5、附錄 C][suphx]

能搬的：前瞻特徵的作法。不能搬的：其餘三項都要深度學習管線。

### Mizukami & Tsuruoka 與 Bakuuchi

- **三個對手模型**，都用鳳凰卓牌譜擬合：對手是否聽牌（logistic regression，
  6,888 個特徵，AUC 0.777，人類高手 0.778）、每一種牌是不是它的和牌張
  （34 個 logistic regression）、和牌打點（對數打點的線性迴歸）。特徵是
  棄牌、副露數、巡目這類公開資訊。[§IV–VI][mizukami]
- **怎麼接進 Monte Carlo。** 每張候選棄牌的分數是
  `Sim(牌) × Π(1 − 放銃機率) − Σ 期望放銃損失`。模擬裡自己照「單人麻將」的
  打法；對手不看手牌，只帶兩個旗標（聽牌、棄和），依機率和牌或立直，摸到
  的牌直接打掉。每張候選牌用同一組亂數控制對手與牌山。[§VII][mizukami]
- **結果。** 對公開最強的 Mattari Mahjong，平均名次 2.48 ± 0.07 對 2.51，
  不顯著（p = 0.29）；對自己的前作顯著（p = 0.01）。天鳳 2,634 局 R1718，
  前作 1,441 局 R1689，名次差不顯著（p = 0.22）。和牌率 0.245 對前作的
  0.256，放銃率 0.131 對 0.148：放銃少了，和牌也少了。[§VIII][mizukami]
- **前作是「單人麻將」。** 先做一個不吃不碰、只求最快和牌的單人玩家，再補上
  棄和與副露。作者的檢討是：只會單人打法不夠強，因為不會棄和、常放銃。
  [§III、§IX][mizukami]

這條線的核心是「速度」對「放銃損失」的取捨。日麻榮和由放銃者付全額，所以
取捨有分量；十胡放銃不多賠，`doc/stronger-bot.md` 的兩次實驗已量到同樣的
現象（放銃少了、自己胡的也少了）而且是淨輸。對手模型本身的作法（用公開
特徵做 logistic regression）可以直接照做，但它在這裡的用途要換。

### Kurita & Hoki

- 定義四個單人 MDP。主要的一個（Inclusive Policy Solitary Mahjong）把另外
  三家換成一個機率環境：自己每打一張，有機率放銃而結束；每過一巡，有機率
  別家和牌而結束；別家打出的牌依機率出現，可以吃碰或榮和。另外三個 MDP
  各只看和牌、聽牌、棄和，算出來的值當成特徵。[§IV–VI][kurita]
- 向聽數 0 到 1 時直接用主要 MDP 的值；2 到 3 時用另外三個 MDP 的特徵；
  更遠時用簡單規則。每個決定幾秒。[§VII][kurita]
- 3,557 局東風戰，一席對 Bakuuchi 與兩席 manue：平均名次 2.23 ± 0.04 對
  Bakuuchi 的 2.29 ± 0.04。作者用的統計量是同一局裡兩者的名次差，平均
  0.0574、標準差 1.822，單尾顯著水準 0.03。[§VII][kurita]
- 作者指出 CFR 類方法要搬到多人麻將並不容易。[§III][kurita]

能搬的：這正是路線三需要的形狀。單人動態規劃加上「這一巡有人胡掉」的
機率，而 `research/safe-cards.md` 已經量過這個機率怎麼隨對手的暗牌張數變。

### 單人期望值的計算

- **tomohxx《麻雀アルゴリズム》。** 狀態是（手牌，巡目）。摸牌的一步：
  對每一種有用的牌，以「剩餘張數 ÷ 牌山剩餘張數」加權；打牌的一步：對所有
  可打的牌取最大值。從最後一巡往回算，就是一張（手牌，巡目）的有向無環圖
  上的動態規劃。把機率換成點數，並在和牌時取「和牌的點數」與「繼續打的
  期望值」的較大者，就得到期望點數。模型只有一個玩家，沒有對手與副露；
  圖裡可以包含手變與退向聽的邊。[和了確率][tomohxx-prob]
- **nekobean/mahjong-cpp**（題目寫成 tomohxx/mahjong-cpp，實際的 repo 在
  nekobean 名下，演算法引用 tomohxx）提供向聽數、有效牌、聽牌機率、和牌
  機率與期望點數，並做成網頁版的何切るシミュレーター。[README][mahjong-cpp]
- **向聽數。** tomohxx/shanten-number 的查表法；Mortal 的向聽數是 tomohxx
  演算法的 Rust 移植。[shanten-number][shanten]、[Mortal meta][mortal-meta]
- **Li & Yan，*Let's Play Mahjong!*。** 定義 deficiency（即向聽數），並給出
  「在 k 次換牌內完成手牌的機率最大」的棄牌策略。後續的快速演算法把「還
  拿得到哪些牌」算進去。[arXiv 1903.03294][lets-play]、
  [arXiv 2108.06832][deficiency]

### Mortal、akochan、kanachan

- **Mortal 的評估。** 1 對 3 duplicate：一組四局用同一個 seed，挑戰者依序坐
  東南西北，另外三席是對照。模擬器保證同一個（seed，局，本場）下牌山、配牌、
  寶牌都相同。例：akochan 挑戰 Mortal「3.1」，挑戰者 13,152 局，平均名次
  2.568 對 2.477。[strength.md][mortal-strength]
- **akochan 的明確期望值。** 只讀了標頭檔，沒讀實作。`tenpai_est.hpp` 對每個
  對手估聽牌機率，以及每一種牌、每一種番符被榮和或自摸的機率，並列舉可能的
  聽牌手牌；`exp_values.hpp` 的 `cal_exp` 把自己的和牌機率（註解寫明來自
  單人麻將的解）、各家聽牌機率、每張牌的放銃機率與代價、流局聽牌機率合成
  一個期望點數；`betaori.hpp` 算棄和的期望值。自對局以 seed 與起家的組合
  重複，README 說這適合「改了策略之後對舊版比較」。
  [tenpai_est.hpp][akochan-tenpai]、[exp_values.hpp][akochan-exp]、
  [betaori.hpp][akochan-betaori]、[readme][akochan]
- **kanachan。** 刻意不用人工特徵，靠雀魂的大量牌譜與大模型。作者自己說這
  需要大資料與大算力。對本專案沒有可搬的東西。[README][kanachan]

能搬的：Mortal 與 akochan 的評估方式，`ab/` 已經在做。akochan 的標頭檔把
期望值分成「自己照單人解和牌」「別家結束這一局」「放銃」「流局」幾塊，
各有機率與價值，形狀與 Kurita 和 Hoki 的 MDP 相同。

### Tencent：ACH／JueJong、OLSS、LuckyJ

- **為什麼選兩人。** ACH 的論文寫得很直接：CFR 只在兩人零和遊戲保證收斂到
  Nash equilibrium，而 policy gradient 的自對局沒有這個保證；Li et al.
  （Suphx）已經用監督學習與傳統強化學習做了四人麻將，「相對地，我們從賽局
  理論的角度研究 1-on-1 麻將，也就是以 Nash equilibrium 為目標」。benchmark
  的說明也寫「為了從賽局理論的角度促進麻將研究」。另一個理由是這個兩人
  遊戲本身在騰訊的線上遊戲裡有大量玩家。[§1、§7.1、附錄 A][ach]
- **兩人麻將的規格。** 規則近似國標麻將，只用 72 張牌（萬、風、三元、花、
  季），每人 13 張。資訊集大小約 10^11，撲克約 10^3。
  [ACH 附錄 A][ach]、[OLSS 附錄 D][olss]
- **評估。** 對規則 bot 每 500 訓練步打 10,000 局；另外訓練 best response 量
  exploitability 的下界。對人類：157 位玩家 7,700 多局平均每局贏 4.56 分；
  對冠軍程海華 1,000 局，0.82 ± 0.96 分，單尾 p = 0.19。作者的措辭是冠軍
  「沒能有效地 exploit JueJong」，不是 JueJong 顯著贏。[§7][ach]
- **OLSS** 是作者有重疊的一組人在同一個兩人麻將上做的即時搜尋。評估段落
  寫明：兩人麻將變異很大，所以打了 200,000 局，隨機產生 100,000 副牌，
  同一副牌雙方交換座位各打一次。即使如此，95% 信賴區間仍是 ±0.13 番，
  而套用搜尋後每局的領先是 0.14 到 0.22 番。搜尋裡的對手模型有兩種：
  「依剩餘牌的分布隨機打一張」與學來的 ACH 模型；兩者的結果在誤差內相同
  （對 ACH 是 0.14 對 0.18，對 PPO 是 0.22 對 0.20）。
  [附錄 D、§7.2、表 3][olss]
- **有沒有搬到四人。** LuckyJ 沒有找到論文。依媒體轉述的騰訊公告，LuckyJ
  用的是 ACH 這個策略最佳化演算法（強化學習加 regret minimization 的
  自對局），以及一個基於「樂觀價值估計」的不完美資訊搜尋；2023 年 7 月在
  天鳳特上房達到十段，安定段位 10.68，用了 1,321 局。這些都是二手來源，
  四人版的訓練細節未能查證。[IT之家][luckyj-ithome]、[量子位][luckyj-qbit]、
  [TechNode][luckyj-technode]
- **作者有重疊的另一篇麻將論文**是 RVR（IEEE CoG 2022）。摘要說麻將的隱藏
  資訊多、計分複雜，獎勵訊號的隨機性與變異很大；作法是用看得到全域資訊的
  relative value network 加上 expected reward network 來降低獎勵的變異。
  全文未取得，摘要沒有寫是哪一種麻將，只說在 Botzone 上勝過 62.5% 的對手。
  [DOI 10.1109/CoG51982.2022.9893584][rvr]

也就是說，依公告搬過去的是演算法，而多人麻將的那篇論文另外用了一套處理
獎勵變異的東西。讀到的論文沒有任何地方說兩人麻將學到的「打法」被拿去用在
四人。

能搬的：對手寫的 bot 來說只有兩個觀察。評估要用同一副牌換座位，而且兩人
麻將打了 20 萬局也只勉強分得出 0.14 番；搜尋裡把對手當成隨機出牌，與用
學來的對手模型分不出高下。也就是說，在這個隨機性很高的遊戲裡，把對手
模型做得更準並沒有讓搜尋變強。

### 國標麻將與 Botzone 的比賽

- 2020 與 2022 兩屆 IJCAI 麻將 AI 比賽。前十六名的作法分三類：依人類知識的
  啟發式、監督學習、強化學習；作者的結論是後兩者明顯強於啟發式。第一屆
  多數隊伍用啟發式：用搜尋算向聽數，再依手牌決定要做哪一種牌型，接成一棵
  行為樹。[§5][oim]
- **Duplicate 的作法。** 四人麻將裡吃碰會改變摸牌順序，所以把牌山分成四份，
  每一席只從自己那一份摸，配牌也在裡面；同一副牌山打 24 局，四個 agent
  坐遍所有排列，分數加總算一場。作者說這不只壓掉牌山的運氣，也減少一家
  對另一家的影響。外層再用瑞士制，因為四人遊戲排不了循環賽；要數百輪才能
  得到穩定的排名。花牌因為只影響分數不影響打法，直接拿掉。[§4.2][oim]
- 人機賽把 duplicate 改成「人類各自坐在不同桌的同一個位子、面對同樣的牌山
  與同樣的三個基準 AI」，因為人會記牌山。[§4.4][oim]
- 第三屆的示範論文重述同一套賽制。[IJCAI-24][ijcai24]

能搬的：按座位切開牌山的 duplicate（見 D 節）。要記住的：在這個比賽裡，
手寫的啟發式輸給了學習的方法。

### 其他

- **Meowjong** 做三人麻將。作者把它當成一個獨立的遊戲（牌少、打法更兇），
  不是四人的前一步。[arXiv 2202.12847][meowjong]
- **CFR-p** 把 CFR 加上階層式的策略抽象，用在兩人麻將。只讀了摘要。
  [arXiv 2307.12087][cfrp]

## B. Gin rummy 與 rummy 類 AI

### EAAI 2021 Gin Rummy 大學生研究挑戰

14 隊交了 14 個 AI 與 14 篇論文，13 篇被接受。賽制：14 個 AI 分成兩組各
7 個，每組前四進八強，八強共打 10,000 局，以 Elo 排名。第一名 MARJJ，
第二名並列 GettysburgPlayer 與 GinRummyAndTonic。[結果][gr-results]、
[挑戰首頁][gr-home]

| 論文 | 貢獻 | 數字 |
| --- | --- | --- |
| [A Heuristic Evaluation Function for Hand Strength Estimation][gr-marjj]（MARJJ，第一名） | 棄牌分數 = 現在的 deadwood + 隨回合衰減的未來 deadwood（只看最好的 k 張摸牌）+ 這張牌對對手的用處；knock 用 MCCFR | 把「對手確定能成順」或「確定能成刻」的加減分歸零，對原設定 50,000 局的勝率是 0.469 到 0.480 |
| [Knocking in the Game of Gin Rummy][gr-knock]（第二名並列） | 以一群打法不同的 AI 做線性迴歸，學何時 knock | 每組對戰 10,000 局，Wilson 區間 |
| [Extracting Learned Discard and Knocking Strategies][gr-extract]（第二名並列） | 學棄牌的期望效用；knock 用 MCCFR，並比較不同規則下的均衡 | |
| [Estimating Card Fitness for Discard][gr-fitness] | 以四個特性給棄牌排名，再用簡化的 CFR 選 | |
| [A Deterministic Neural Network Approach][gr-dnn] | 啟發式與神經網路模型比較摸、棄、knock | |
| [A Data-Driven Approach for Gin Rummy Hand Evaluation][gr-data] | CNN、Monte Carlo 與 Bayes 推論算攻守兩個分數 | 對 Simple 55.4% ± 0.8% |
| [Opponent Hand Estimation in the Game of Gin Rummy][gr-ohe] | Bayes 與兩種神經網路估對手手牌，比較各階段的準確度 | 用在棄牌上，勝率多 5% |
| [Heisenbot][gr-heisen] | 摸、棄、knock 三套規則，用模擬賽挑組合 | |
| [A Highly-Parameterized Ensemble][gr-ensemble] | 棄牌用多個評分的加權 ensemble，分批調參數 | 對基準 0.701 → 0.739 |
| [Random Forests for Opponent Hand Estimation][gr-rf] | 自對局資料訓練 random forest，估每張牌在對手手上的機率 | 對不估牌的基準 61% |
| [Opponent Modeling and Myopic Meld Distance][gr-mmd] | 手牌離成組的距離 + 對手想要的牌 + 保守的 knock，逐項拿掉比較 | 對 Simple 74.6%（500 局） |
| [Opponent Hand Estimation Using Deep Neural Networks][gr-dnn-ohe] | 用啟發式 agent 產生資料訓練 DNN 估對手手牌，再回頭改進 agent | 855／1,500 = 57% |
| [Modeling Expert Knowledge in a Heuristic-Based Gin Rummy Agent][gr-tonic]（Tonic） | 把專家口訣寫成帶參數的模型，grid search | 對基準至少 68% |

可以讀出的三件事：

- **前三名都把 knock 當成獨立的小決策來解。** 兩篇用 MCCFR，一篇用線性
  迴歸。棄牌則是手寫或學來的評分；MARJJ 的作者說棄牌的資訊集多到沒辦法
  直接用 CFR。[MARJJ][gr-marjj]、[Maravich et al.][gr-knock]、
  [Goldstein et al.][gr-extract]
- **對手資訊有用，而且用法很樸素。** MARJJ 的五組加減分只看「已知在對手
  手上的牌」與「已經死掉的牌」：對手已有能跟這張牌成順或成刻的牌就加罰，
  這張牌已經不可能被對手用上就減罰。三篇估對手手牌的論文，對不估牌的版本
  勝率是 55% 到 61%。[MARJJ][gr-marjj]、[Francis et al.][gr-ohe]、
  [Hein et al.][gr-rf]、[Mishra & Aggarwal][gr-dnn-ohe]
- **參數不敏感。** MARJJ 的作者把每個參數掃過一遍，結論是在很寬的範圍裡
  勝率都在 50% 上下，最好的值也只到 51%。knock 那一篇則發現在一種對手上
  擬合的迴歸換一種對手就變差。這兩點與 `research/fitted-weights.md` 的
  結論相同。[MARJJ][gr-marjj]、[Maravich et al.][gr-knock]

### 較早的研究與環境

- **Kotnik & Kalita 2003。** 同一個神經網路價值函數，用 TD 自對局與用共同
  演化各訓練一次。演化的版本比較強，最好的一個在錦標賽贏 87.5%。
  [ICML 2003][kotnik]
- **OpenSpiel** 的 gin rummy 只打一手，註解寫資訊狀態超過 10^85，開局時
  對手的手牌有 41 取 10 種。**RLCard** 的 gin rummy 用 5 個 52 維的平面表示
  狀態。兩者都是環境，不附強的 bot。[gin_rummy.h][openspiel]、[games.md][rlcard]
- **Kelidari et al. 2026（arXiv 2607.06854）。** 先做一個固定的規則 bot 當
  量尺：精確算最小 deadwood、能 knock 就 knock、完全不猜對手。它贏過作者
  訓練的每一個強化學習 agent 70% 到 99%。兩個與本專案直接相關的數字：
  均勻發牌的 determinized 搜尋對它最多 26%；同一個搜尋若偷看對手的手牌，
  每步 10 次 rollout 是 41%，120 次是 85%。另外，把對手的手牌直接餵給
  學習的 agent 並沒有幫助（24.7% 對 25.5%）。摘要的數字以 v2 為準：整套
  訓練技巧把自對局冠軍從約 30% 拉到約 34%。[v2][gold]

### 其他摸打類遊戲

- **Indian Rummy。** 一篇規則 bot：以 MinDist（手牌到最近的合法牌型的編輯
  距離）當評分，加上對手手牌的模型，兩人零和的模擬。一篇強化學習：PPO，
  先用較強的示範資料做一次模仿學習，對最強的搜尋型對手 53.9%。兩篇都只讀了
  摘要。[arXiv 2601.00024][mindist]、[arXiv 2606.21975][irumai]
- **Dhumbal**（尼泊爾的多人牌戲）。1,024 局裡規則 bot 勝率 88.3%，ISMCTS
  9.0%，PPO 1.5%。只讀了摘要。[arXiv 2510.11736][dhumbal]
- **跑胡子／字牌、Conquian、Khanhoo。** 沒有找到 AI 論文。GitHub 搜尋到的
  跑胡子專案，從說明看是胡牌判定與遊戲伺服器，沒有逐一打開。

### 作者自己的 gin-rummy-engine

讀的是 `d77f2d7` 的 README、`docs/strong-opponents.md`、
`docs/marjj-parity-plan.md` 與 `docs/marjj-parity-design.md`。它與 tsap-oo 的
架構相同（`Strategy`、`View`、貪心的 `HeuristicBot`、determinized 的
`MonteCarloBot`），所以是最接近的對照。

- **兩人 gin rummy 裡，均勻發牌加貪心 playout 的 Monte Carlo 明顯贏過貪心
  bot。** `mc:64` 對 `greedy` 贏 63.6%（62.8% 到 64.4%）的整場；對基準
  `EaaiSimpleBot`，樣本從 64 加到 512 一路變強（65.4% → 74.7%）；對 MARJJ
  到 512 之後才持平（1024 樣本是 55.1%）。同樣的作法在四人十胡是
  +0.148 ± 0.276 分，看不出差別。
  [README][gre-readme]、[計畫][gre-plan]
- **追上最強對手靠的是樣本數，不是對手模型。** 對 MARJJ 的差距是把每個
  決定算快 3.9 倍（打法逐位元相同）之後，用 512 樣本補上的。照「搞懂對手」
  提出的三個診斷都被最便宜的 A/B 否定：照對手的 knock 習慣 playout、發出
  發展過的對手手牌、改價值函數。[計畫 §6、§8][gre-plan]
- **信念越準不等於越強。** 把發給對手的手牌修到與實測的 deadwood 相符，
  對兩個弱對手各多 3.2 與 2.0 分，對 MARJJ 少 1.2 分：bot 不再被 undercut，
  卻也不敢 knock，輸在更長的 gin 競賽。作者的結論是「只有量測說了算」。
  [設計 D3、M2.5][gre-design]
- **放寬偏離貪心選擇的門檻通常變弱。** 依雜訊偏離比照著貪心打還差。
  tsap-oo 的 `CONFIDENCE = 1` 是同一個旋鈕。[計畫 §6][gre-plan]
- **量測。** 換座位的配對（mirrored pairs）、以配對為單位的信賴區間、
  不中途停、不換 seed；每次改動都對三個對手量（目標加兩個「不能退步」的
  守門對手），避免只學會贏一個對手。tsap-oo 目前只有 `HeuristicBot` 一個
  對手。[README][gre-readme]、[計畫 §5][gre-plan]
- **程序上的教訓。** 三個機制都是先從對局統計「看起來很有道理」，再被 A/B
  否定。計畫因此規定：先說出機制，找出能開關它的最小固定版本，先量那個
  版本，再蓋別的。[計畫 §8][gre-plan]

能搬的：樣本數與 playout 速度比對手模型優先；每個「對手」方向的想法先做
最小的固定版本去量。要注意的：gin rummy 的 Monte Carlo 有 knock 這個高槓桿
的決定可以找，十胡的胡是強制的，沒有對應的決定。

## C. 十胡／四色牌本身

### AI 論文與學位論文

沒有找到。查過的地方：arXiv 標題與全文、一般網頁搜尋（中文、英文、越南文
的 Tứ Sắc）、以「四色牌」搭配電腦對局、人工智慧、演算法搜尋臺灣博碩士
論文知識加值系統的索引頁。搜尋引擎回的都是麻將、橋牌等其他遊戲的論文；
唯一提到四色牌的是一本樂齡桌遊教具的碩士論文（只看到搜尋摘要，未開啟）。
臺灣博碩士論文知識加值系統本身的檢索需要 session，沒有直接查。

公開的實作與 bot 已經在 [`public-opponents.md`](public-opponents.md) 整理，
這裡不重複。

### 兩人與三人的規則

| 來源 | 人數 | 發牌 | 人數少時改了什麼 |
| --- | --- | --- | --- |
| [中文維基〈十胡〉][zhwiki-shihu] | 「可供2人到6人遊戲」 | 「2~5人遊戲時，莊家摸二十一張，閒家摸二十張。6人遊戲時，莊家摸十六張牌，閒家摸十五張。」 | 沒寫。只寫「五人以上稱為五龍，所有賭注（台與底）變為兩倍」，五龍沒有 6 張廢牌 |
| [中文維基〈四色牌〉][zhwiki-sisepai] | 沒寫 | 沒寫 | |
| [國史館臺灣文獻館][thgov] | 「牌搭是二至四人」 | 「依照玩牌人數發牌數量以莊家比其他人多一支為原則」，沒給張數 | 沒寫 |
| [國瑜企業社][guoyu]（牌具廠商） | 四人玩法、五人玩法 | 「一人發20支牌，頭家發20支」（原文如此） | 五人：8 胡以上可胡，牌不好可以放棄。沒有二、三人 |
| [遊戲基地〈四色牌十胡篇〉][gamebase]（存檔） | 「四人版」 | 每人 20 張，莊家 21 張 | 只有四人 |
| Board Game Arena：[中文說明][bga-zh]、[英文說明][bga-en]、[遊戲頁][bga-panel] | 2 至 4 人；遊戲頁的資料是 `player_numbers` [2,3,4]、預設 2 人、排位（arena）2 人 | 每人 7 + 7 + 6 = 20 張，先手 21 張 | 說明頁沒有任何依人數變動的規則 |
| [pagat.com][pagat] | 沒寫 | 沒寫，轉介下面兩頁 | |
| [Wei-Hwa Huang][huang]（存檔） | 「anywhere from 2 to 4 people」；人多時（甚至四人）牌不夠，常減少手牌或混入更多副牌 | 每人 20 張 | 沒寫 |
| [tnt（CMU）][tnt] | 通常四人，有時三人，「只有兩個人玩相當無趣」 | 莊家 21，其餘 20 | 沒寫 |
| [英文維基][enwiki] | 資訊框：4 人（標準牌組有 2 到 3 人的變體）；內文：標準四人，兩人或三人也能玩；超過四人每多一人加一組 28 張 | 20 張，莊家 21 | 沒寫 |
| [越南文維基〈Tứ sắc〉][viwiki] | 適合四人，但兩人或三人都能玩 | 每人 20 張，莊家第一輪多拿一張 | 沒寫。另記順化有一種用同一副牌的三人玩法 bài xệp，是另一種遊戲 |
| [Vinagames Tứ Sắc][vina] | 「for 2-4 people and it is best to play the game with 4 people」 | 每人 20，先手 21 | 沒寫。輸家各自付給贏家 |

結論：

- **兩人與三人「可以玩」有多個獨立來源**：中文維基、國史館、BGA、Huang、
  英文與越南文維基、Vinagames。
- **沒有任何來源寫出兩人或三人的規則調整。** 發牌一律是每人 20 張、莊家
  21 張。中文維基的「2~5 人各 20 張」在五人時只剩 112 − 101 = 11 張牌堆，
  與同一段「五龍因剩餘牌堆張數較少」的說法一致，但沒有引註。
- **BGA 是唯一可操作的兩人規格，而且它的排位賽就是兩人。** 它的規則與人數
  無關：手裡打出的牌只有下家能吃，摸出的牌是摸牌者與下家都能吃，胡、槓、
  碰不限座位；達到 10 胡的贏家「跟所有其他玩家收取」。兩個語言版本的收分
  寫法不同：中文是每家收「胡數減去門檻」，英文是「超過門檻的胡數，加上
  每家 1 分」。[中文][bga-zh]、[英文][bga-en]
- **把這套規則套到兩人，遊戲會變。** 以下是從規則推出來的，不是來源說的。
  兩人時唯一的對手同時是上家與下家，所以每一張棄牌、每一張摸牌雙方都能
  吃，吃的座位限制實際上消失。牌堆是 112 − 41 = 71 張，不是四人時的 31 張。
  贏家只向一家收，輸家付全部，放銃與被自摸的代價相同。
- `chunhua1972/four-color-game` 的 2 到 4 席同樣用 20／21 張，5、6 席改成
  16／17 與 14／15 張；它的文件明說後者是自訂的擴充，不宣稱是傳統規則。
  [DEVplan.md §3.2][fcg-plan]
- **放銃要不要多賠，各地不同。** tnt 記的賭法裡，放銃的人有時要替另外兩家
  付；Vinagames 的規則是沒有打出「最差的牌」而放銃的人，替其他輸家付。
  本專案的規則是放銃不多賠，「防守不值錢」的結論只對這套規則成立。
  [tnt][tnt]、[Vinagames][vina]

## D. 先做兩人、再擴到多人

### 先例

| 先例 | 搬過去的 | 沒搬過去的，或要另外補的 |
| --- | --- | --- |
| Libratus（兩人）→ Pluribus（六人）[Brown & Sandholm 2019][pluribus] | 自對局算 blueprint、抽象化、即時搜尋這三件事的架構 | Libratus 只在「剩下的遊戲短到能解到底」時搜尋，六人時做不到，改成深度受限、葉節點讓每家在 k 種後續策略裡選；Nash equilibrium 的保證與「不會輸」的性質都沒有了，目標改成「實際上贏過人」 |
| ACH／JueJong（兩人麻將）→ LuckyJ（四人）[ACH][ach]、[IT之家][luckyj-ithome] | 依公告是 ACH 演算法與搜尋 | 沒有論文，細節未能查證；作者有重疊的多人麻將論文另外加了降低獎勵變異的網路 [RVR][rvr] |
| Poki（多人）→ PsOpti（兩人）[Billings et al. 2003][psopti] | 方向相反 | 為多人桌寫的 Poki「不是為兩人設計的，相當容易被打敗」；兩人版改用抽象化之後的線性規劃解 |
| 三人撲克嫁接兩人專家 [Abou Risk & Szafron 2010][risk] | 有人蓋牌後剩兩人的子賽局，換上兩人的 CFR 解 | 結果不一致：一個基礎策略變強，另一個變弱。能嫁接是因為撲克有人會退出；十胡沒有 |
| 單人麻將 → 四人麻將 [Mizukami & Tsuruoka 2015][mizukami] | 最快和牌的打法，成為模擬裡自己的策略 | 棄和、副露、對手模型都是後來補的；作者說只有單人打法不夠強 |

共同點：**搬得動的是方法，不是學到的打法。** 上表沒有任何一例是「在人少的
版本找出好的打法，然後原封不動用到人多的版本」。Tencent 選兩人的理由寫明
是理論：兩人零和有 Nash equilibrium 的保證，可以定義並量測 exploitability。
Alberta 的兩人版同樣從「最佳解保證至少拿到賽局值」出發。
[ACH §1][ach]、[Billings et al. §1][psopti]

### 兩人零和與多人的理論差異

- **均衡的保證。** 兩人零和遊戲裡，Nash equilibrium 策略期望上不輸給任何
  對手；CFR 的平均策略保證收斂到它。三人以上兩件事都不成立：演算法不保證
  收斂，而且即使每個人各自算出一個均衡來打，合起來也未必是均衡。Pluribus
  的作者因此說，在多人遊戲裡連「目標該是什麼」都成問題。
  [Brown & Sandholm][pluribus]、[Abou Risk & Szafron][risk]
- **多人的搜尋。** Sturtevant 的分析是 UCT 在多人遊戲算的是混合策略的均衡，
  max^n 算的是純策略的均衡。只讀了摘要。[Sturtevant 2008][sturtevant]
- **Determinization 的毛病**是 strategy fusion：搜尋在每一個發牌裡各自做
  決定，實際上卻分不出自己在哪一個發牌裡。Long 等人用三個性質（葉節點的
  相關性、偏向、資訊多快揭露）預測 PIMC 什麼時候夠好，並指出它在 Skat 這類
  牌戲上實際表現不差。[Cowling et al. §III][ismcts]、[Long et al. 2010][pimc]
- **推論值多少，隨遊戲而定。**
  - Skat（三人）：用牌譜擬合「看到這個動作時，各種手牌特徵的機率」，據此
    偏置 PIMC 的發牌，明顯變強；後續改用神經網路預測每張牌的位置，又再
    變強。[Buro et al. 2009][skat]、[Solinas et al. 2019][solinas]
  - Scrabble（兩人）：完全知道對手的牌每局多 37 分；用 Bayes 從對手剛打的
    字推它留下的牌，每局多 5.2 分（630 局）。[Richards & Amir 2007][scrabble]
  - 鬥地主（三人）：作者自己的前一篇研究用簡化版的鬥地主，完美的對手模型
    只比均勻信念好一點，所以這篇論文乾脆不做信念分布。他們的解釋是「能從
    對手的動作推出有用資訊的局面很少」。[Cowling et al. §I][ismcts]
  - Gin rummy：見上面 Kelidari et al. 的數字。偷看對手的手牌讓搜尋從 26%
    到 85%，但直接把手牌餵給學習的 agent 沒有用。[Kelidari et al.][gold]

### 作者的理由：兩人零和裡，對錯比較常被獎懲

作者的說法是：四人時犯錯不一定受罰；兩人零和時，對的決定比較常得到獎勵、
錯的比較常受罰，所以比較容易學到什麼是好的決定。

支持這個說法的來源：

- **兩人零和裡，對手的錯在期望上不會變成我的損失。** Pluribus 論文的說明
  是：打均衡策略的一方期望上不會輸；在比猜拳複雜的遊戲裡，對手只要選了
  次佳的動作，它期望上就會贏。多人時沒有這個性質。
  [Brown & Sandholm][pluribus]
- **兩人零和有一把與對手無關的尺。** ACH 另外訓練 best response 來量各個
  agent 被 exploit 的程度；OLSS 在撲克上直接報 exploitability。四人遊戲
  沒有對應的量。[ACH §7][ach]、[OLSS §7.1][olss]
- **扣掉運氣的估計量在兩人時效果好得多。** DIVAT 在兩人限注撲克把所需手數
  降到 1／5.7 到 1／7.2；MIVAT 在兩人時與它相當，在六人限注撲克只把變異數
  降了 20%，作者歸因於特徵太少，也沒抓到還沒蓋牌的人坐在哪裡。
  [DIVAT][divat]、[MIVAT][mivat]
- **Duplicate 的成本隨人數的階乘成長。** 兩人是同一副牌換座位打 2 局，三人
  撲克是 6 種排列，四人麻將是 24 種而且牌山要按座位切開。
  [OLSS 附錄 D][olss]、[Abou Risk & Szafron §4.4][risk]、[Lu et al. §4.2][oim]
- **本專案自己的數據。** 對照組放銃而候選沒放的 1,691 局裡，82% 還是別家
  胡了（`doc/stronger-bot.md`）。這就是「對的決定沒有被獎勵」。

不支持，或要打折的來源：

- **兩人麻將的評估並沒有變得便宜。** OLSS 用了 200,000 局加換座位，95%
  信賴區間還有 ±0.13 番，套用搜尋後的領先是 0.14 到 0.22 番；JueJong 對
  人類冠軍 1,000 局只得到 p = 0.19。
  [OLSS 附錄 D、表 3][olss]、[ACH §7][ach]
- **兩人撲克同樣被發牌的運氣淹沒。** 兩人限注撲克每手的標準差約 6 美元，
  要的精度約 0.05 美元，需要五萬手以上；2015 年的人機賽打了 80,000 手，
  差距很大卻只在顯著的邊緣。雜訊的主要來源是發牌，不是人數。
  [MIVAT][mivat]、[AIVAT 緒論][aivat]
- **均衡的保證幫不到對固定對手的 A/B。** 這是本文的推論，不是來源說的。
  上面前兩個論點講的都是「對任何對手都不輸」的策略。`ab/run.sh` 量的是
  對 `HeuristicBot` 的得分，在兩人與四人裡，「對」都只是「對這個對手賺得
  多」，Nash equilibrium 沒有多給什麼。
- **人少的版本學到的東西可能不在人多的版本裡。** Poki 的例子是反方向的
  證據；嫁接兩人專家的結果好壞參半。[Billings et al.][psopti]、
  [Abou Risk & Szafron §5][risk]

沒有查到的：沒有找到任何研究直接量「同一個錯誤在兩人與多人版本裡被懲罰的
頻率」，也沒有找到 ACH 或 Pluribus 的作者以「兩人比較容易學到好決定」當作
選兩人的理由。他們給的理由是理論保證與可計算性。

一個粗估（本文的算術，不是來源的數字；假設各家胡牌率相同、胡數都是 h、
流局約 6%）：四人時贏家收 3h、輸家各付 h，把自己的胡牌率提高 δ 約值 4hδ，
每局標準差約 1.7h；兩人時約值 2hδ，標準差約 1h。兩者的訊號雜訊比相近
（2.4δ 對 2.1δ）。兩人版若有優勢，得來自 δ 本身比較大：候選 bot 做了一半
而不是四分之一的決定，而且它的錯不會被第三家接走。這有可能，但沒有人量過。

### 降低評估變異的技術

現況：`ab/` 已經是配對的 duplicate。由 `doc/stronger-bot.md` 各次實驗的
標準誤反推，打法不同的那些局，得分差的標準差約 27 到 30 分（聽牌規則那一次
約 37），與單一座位一局的標準差（約 30）幾乎一樣大。標準誤是這個數乘上
√(不同的局數) 再除以總局數，所以要壓的是「分歧之後」的雜訊。

| 技術 | 具體做法 | 來源報告的效果 | 放到 `ab/` 的意思 |
| --- | --- | --- | --- |
| Duplicate、共同亂數 | 同一副牌、同樣的對手，只換受測者 | Mortal、akochan、Botzone、OLSS、Mizukami 都用 | 已經有了 |
| Rollout | 從要比較的局面出發，把之後的隨機性重抽很多次取平均；各個候選用同一組亂數 | GNU Backgammon 的 rollout 功能，手冊以 1,296 局為例；Mizukami 的模擬對每張候選牌用同一組亂數 | 從第一個分歧點起，把剩下的牌堆重洗 K 次，兩邊各打 K 次，取平均的差 |
| 分層的第一步 | 讓前一兩步的隨機結果剛好各出現一次 | GNU Backgammon 的 quasi-random dice | 重洗時讓「下一張摸到的牌」依剩餘張數輪過每一種 |
| 扣掉運氣（advantage sum、control variate） | 每一個機率事件，把「事件前的期望評分 − 事件後的評分」加到結果上；這一項期望為零，所以不偏 | DIVAT 5.7 到 7.2 倍；AIVAT 在兩人無限注把標準差從 25.96 降到 8.10，手數少十倍以上；MIVAT 在六人限注只降了 20% 的變異數 | 每摸一張牌，對牌堆裡每一種可能的牌算一次評分取平均，把（平均 − 實際那張的評分）加到得分上 |
| 截斷 | 不打到結束，打幾步後用評分代替結果 | Tesauro 與 Galperin：每次試驗的變異小得多，至少快一個數量級；GNU Backgammon 手冊提醒這會帶進系統誤差 | 評分換算成分只在「手牌」上成立（`research/grade-in-points.md`），在 A/B 裡用會帶進偏差，不建議 |
| 依座位切開牌堆 | 每一席只從自己那一疊摸，吃碰改變順序時別家摸到的牌不變 | Botzone 的作者說可以減少一家對另一家的影響，沒給數字 | 兩邊分歧之後，各席摸到的牌仍然相同，兩邊的相關性提高 |
| 序貫檢定 | 邊跑邊算概似比，夠了就停 | fishtest 的 GSPRT；配對的五項式模型比三項式「大幅節省」 | 平均局數變少，但「局數在看資料前定下」這條鐵則要改成「界線在看資料前定下」 |

來源：[GNU Backgammon rollouts][gnubg-ro]、[quasi-random dice][gnubg-qr]、
[Tesauro & Galperin 1996][tesauro]、[DIVAT][divat]、[MIVAT][mivat]、
[AIVAT][aivat]、[Pluribus][pluribus]、[Lu et al. §4.2][oim]、
[Mortal][mortal-strength]、[akochan][akochan]、[fishtest][fishtest]

補充說明：

- **Rollout 為什麼不偏。** 這是本文的推論。分歧之前兩邊的每一個決定都只
  依已經翻開的牌，所以在分歧的那一刻，剩下的牌堆的順序對兩邊來說仍是均勻
  隨機的。把那一段重洗 K 次取平均，期望值不變，只是把「分歧之後的運氣」
  平均掉。別家的暗牌不能重發，那是局面的一部分。
- **能省多少要先量。** 得分差的變異數可以拆成兩塊：不同局面之間真實差異的
  變異 σ_b²，與同一個局面內牌堆運氣的變異 σ_w²。重洗 K 次只把後者除以 K。
  若 σ_b 是 5 分，K = 16 時標準差從 28 降到約 8.6，等於局數的十倍；若 σ_b
  是 10 分，約五倍。σ_b 目前不知道：拿 1,000 個分歧的局面各重洗 32 次就
  量得出來。成本只加在分歧的局上，所以改動越小的候選越划算。
- **AIVAT 裡與本專案有關的只有「機率事件」那一半。** 它另一半是利用已知的
  隨機策略；`HeuristicBot` 是確定性的，那一半是零。也就是說這裡用得上的是
  MIVAT 的形式。MIVAT 還提供了挑評分函數的方法：直接以「估計量的變異數
  最小」為目標，擬合一個線性函數，特徵可以用到四家的手牌，因為評估者看得
  到。`fit` example 的手牌擬合已經有現成的特徵。[MIVAT][mivat]
- **Pluribus 的人機賽**是多人遊戲裡用 AIVAT 的實例：10,000 手，平均每手贏
  48 mbb（千分之一個大盲），標準誤 25 mbb。[Brown & Sandholm][pluribus]
- **麻將專案用的都是 duplicate，沒有人用 advantage sum。** Suphx 是固定
  100 萬局加重抽樣；Kurita 與 Hoki 用同一局內的名次差；Mizukami 用同牌山的
  對戰加 bootstrap 與 Welch t 檢定。[Suphx §4][suphx]、[Kurita & Hoki §VII][kurita]、
  [Mizukami §VIII][mizukami]

## 可以搬到十胡的想法

依「預期的資訊量 ÷ 成本」排序。前兩項是量測，不改 bot。

1. **從分歧點 rollout，並扣掉摸牌的運氣。**
   來源：[GNU Backgammon][gnubg-ro]、[MIVAT][mivat]、[AIVAT][aivat]。
   意思：`ab/src/main.rs` 在兩邊第一次做出不同決定時記下局面，把剩下的
   牌堆重洗 K 次各打一遍；再對每一張摸牌扣掉一個期望為零的修正。最簡單的
   修正是「這張牌讓誰立刻胡了」：評估者知道四家各聽什麼，算得出每一家被
   這張摸牌胡掉的機率與金額。
   成本：只在離線。新的估計量要先驗證：候選與對照相同時必須剛好是零，
   並且重現一個已知的結論（例如 −0.037 的那一次）。
   支持：計畫裡的七次 A/B（含 `MonteCarloBot` 的重測）有五次結論不明，
   瓶頸是標準誤。反對：效果取決於 σ_b，還沒量；六人撲克的例子顯示多人時
   運氣修正的效果有限。
2. **量「偷看手牌的 Monte Carlo」的上限。**
   來源：[Richards & Amir][scrabble]、[Kelidari et al.][gold]、
   [Cowling et al.][ismcts]。
   意思：讓 `MonteCarloBot` 的發牌直接用真實的別家手牌（牌堆順序仍然隨機，
   或連牌堆也用真的，各量一次），在同一個 A/B 上對 `HeuristicBot`。這是
   任何對手推論的天花板。
   成本：只在離線，改動很小。
   支持：均勻信念的 Monte Carlo 是 +0.148 ± 0.276；如果偷看也只多零點幾分，
   第四層的行為推論與「Monte Carlo 的發牌」都可以不做。反對：偷看同時消除
   了 strategy fusion，所以它高估推論能拿到的部分。
3. **「不餵下家」寫成 MARJJ 式的加減分。**
   來源：[Ahmed et al.][gr-marjj]。
   意思：棄牌評分加一項「這張牌對下家的用處」。下家已經亮出或可推知握有
   能跟它成組的牌就加罰；能跟它成組的牌已經死光（`doc/stronger-bot.md`
   第三層的「壁」）就減罰。
   成本：`HeuristicBot` 的量級，網頁可用。
   支持：計畫已經把它列為下一步；棄牌有 16% 到 56% 能被下家吃成順。反對：
   gin rummy 是兩人，對手是唯一的受益者；十胡的下家只是三家之一，防守的
   兩次實驗都沒贏。預期是小的正值，要靠第 1 項才量得到。
4. **單人期望值，加上「別家每一巡胡掉的機率」。**
   來源：[tomohxx][tomohxx-prob]、[Kurita & Hoki][kurita]、
   [Li & Yan][lets-play]、[Suphx §2][suphx]。
   意思：路線三，但照 Kurita 與 Hoki 的做法把另外三家換成一個每巡有機率
   結束這一局的環境，機率取自 `research/safe-cards.md` 那張依暗牌張數的表。
   這樣對手的進度就進了評分，而不是只用來躲銃。
   成本：算一次要看狀態數；十胡每張摸牌四家都能要，「能拿到的牌」比麻將
   難定義。網頁能不能用未知。
   反對：計畫自己的判斷成立。均勻信念下多看幾步的 Monte Carlo 沒有贏，
   擬合的權重也沒有贏，這一項是把同一份資訊算得更準。沒有數據支持先做。
5. **用公開特徵擬合「這一家離胡多遠」。**
   來源：[Mizukami & Tsuruoka §IV][mizukami]、[akochan][akochan-tenpai]。
   意思：以暗牌張數、亮出的胡數、吃碰的次數與時機做 logistic regression。
   成本：離線擬合，線上查表。
   反對：第二層的機率表已經是這件事的簡化版，用來防守是淨輸。除非第 4 項
   要用，否則不值得單獨做。
6. **Oracle guiding、pMCPA、蒸餾。**
   來源：[Suphx][suphx]。
   都需要深度學習的訓練管線；pMCPA 連 Suphx 自己上線時都沒用。計畫把蒸餾
   排在「帶對手模型的 Monte Carlo 明確贏過 `HeuristicBot`」之後，這個順序
   與文獻一致。
7. **兩人版當研究載具。**
   來源：D 節。
   支持：規格現成（BGA 的 2 人桌，排位預設就是兩人）；duplicate 只要換座位
   打 2 局；候選做一半的決定。
   反對：沒有先例是把人少的版本學到的打法搬到人多的版本；兩人麻將的評估
   仍是 20 萬局；兩人時吃的座位限制消失、輸家付全部、牌堆是 71 張，「防守
   不值錢」這個四人的結論很可能不成立，反過來在兩人量到的東西也不保證
   成立於四人。
   如果要做，建議先花一次小實驗確認兩件事：兩人版每局的標準差與分歧局的
   比例；以及一個四人已知結論（例如「從評分扣掉 12 胡 × 危險度」的
   −0.037）在兩人版是否同號。同號才談得上搬。
   後續：前一件與兩個已知的贏已經量了，見 [`two-seats.md`](two-seats.md)；
   −0.037 那一次的候選沒有留在版本庫裡，沒辦法重放。

## 限制

- **未能取得全文的：** ACH 的 OpenReview 頁擋自動存取，讀的是 Internet
  Archive 在 2024-09-22 存的 PDF；《Official International Mahjong》讀的
  是 Internet Archive 存的 MDPI PDF；Pluribus 讀的是作者網站的存檔。RVR
  （IEEE CoG 2022）只有經 Semantic Scholar 與 OpenAlex 取得的摘要。
  Sturtevant 2008、CFR-p、兩篇 Indian Rummy、Dhumbal 只讀了摘要。
- **LuckyJ 沒有論文。** 所有細節來自 IT之家、量子位與 TechNode 對騰訊公告
  的轉述，騰訊的原始公告沒有找到。「ACH 用在四人版」是公告的說法，沒有
  技術文件可以對照。
- **Mortal 的方法沒有文件。** 官方文件自己寫大部分頁面是空的；這裡只引用
  它的評估方式。
- **akochan 只讀了標頭檔與 readme**，沒有讀實作，也沒有執行。
- **臺灣博碩士論文知識加值系統沒有直接檢索。**「沒有找到論文」是搜尋引擎
  索引範圍內的結論。CNKI、萬方也沒有查。
- **Bakuuchi 之後的版本沒有公開論文。** Kurita 與 Hoki 提到他們對戰的
  Bakuuchi 比 2015 年論文的版本強，而且未發表。
- **fishtest 的官方頁只有概述**，GSPRT 與五項式模型的推導在它連出去的文件
  裡，沒有讀。
- **Wolfe 2002 的二十一點基準策略估計量**（把玩家的結果減去基準策略在同一手
  牌上的結果，標準差降 50 倍）只見於 MIVAT 的轉述，原文沒有讀。它的形式
  與 `ab/` 的配對設計相同。[MIVAT][mivat]
- **D 節的三段推論**（rollout 不偏、訊號雜訊比的粗估、均衡保證幫不到固定
  對手的 A/B）與 C 節「套到兩人會怎樣」是本文從規則與既有數據推出來的，
  文中都已標明，不是來源的結論。σ_b 沒有量過。兩人版在本文寫完之後量了
  一次，見 [`two-seats.md`](two-seats.md)：訊號雜訊比相近的粗估與實測相符。
- **2026 年年中以後的文獻**只查到 arXiv 上的 Kelidari et al.（7 月，8 月
  改版）與 IRumAI（6 月）。Botzone 第六屆比賽的頁面在搜尋結果裡出現，
  沒有開啟，結果未查。
- **gin rummy 各篇的勝率基準不同**（Simple player、各隊自己的基準、彼此），
  表裡的數字不能互相比較。

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
[zhwiki-sisepai]: https://zh.wikipedia.org/wiki/%E5%9B%9B%E8%89%B2%E7%89%8C
[thgov]: https://dict.th.gov.tw/Print.aspx?ID=2186
[guoyu]: https://www.guoyu.com.tw/zh/qa/1.html
[gamebase]: https://web.archive.org/web/20131005012051/http://www.gamebase.com.tw/wekey/%E5%A4%A7%E8%95%83%E8%96%AF%E9%81%8A%E6%88%B2%E7%B6%B2%E6%88%B0/%E5%9B%9B%E8%89%B2%E7%89%8C%E5%8D%81%E8%83%A1%E7%AF%87/
[bga-zh]: https://zh.doc.boardgamearena.com/Gamehelpfourcolorcards
[bga-en]: https://en.boardgamearena.com/doc/Gamehelpfourcolorcards
[bga-panel]: https://en.boardgamearena.com/gamepanel?game=fourcolorcards
[pagat]: https://www.pagat.com/rummy/si_se_pai.html
[huang]: https://web.archive.org/web/20081210050742/www.ofb.net/~whuang/ugcs/gp/ssp/
[tnt]: https://www.cs.cmu.edu/~tnt/rules.html
[enwiki]: https://en.wikipedia.org/wiki/Four_color_cards
[viwiki]: https://vi.wikipedia.org/wiki/T%E1%BB%A9_s%E1%BA%AFc
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
