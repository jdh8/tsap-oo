//! Browser front end for 十胡 and 八胡: one human against one to three greedy
//! bots.
//!
//! The engine holds the whole game; JS is a thin renderer that calls one
//! method per user action or bot step and redraws from the JSON snapshot
//! each returns.  `Core` is plain Rust so the flow is testable natively.

use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};
use serde::{Deserialize, Serialize};
use tsap_oo::susik::{Card, Color, Hand, Meld, Rank, Solver};
use tsap_oo::{
    Claim, Eval, Event, ExpectBot, Game, HeuristicBot, Phase, Round, Rules, Seat, Strategy, Table,
    View, discardable,
};
use wasm_bindgen::prelude::*;

const HUMAN: Seat = Seat::ALL[0];

/// The hint's bot, and with `search` the opponents' too.  It plays by the DP
/// with 10 concealed cards or fewer as wasm, where a decision must fit a
/// frame or so, and 14 as the desktop's native code, some 190 ms a decision
/// there (`doc/expectation.md`, batch 3 and D).  Past the gate it plays by a
/// graph one claim deep, two to 17 cards on the desktop (`doc/eval.md`,
/// batch 4), its leaves read from the tables of that table.
fn bot(rules: &Rules, search: bool) -> ExpectBot {
    let (gate, deep) = if search { (14, 17) } else { (10, 0) };
    ExpectBot::new(rules)
        .gated(gate)
        .shallow(Eval::of(rules), deep)
}

/// What the human calls each seat at a table of `players`, counting on from
/// them in turn order
fn name(players: usize, seat: Seat) -> &'static str {
    let names: &[&str] = match players {
        2 => &["你", "對家"],
        3 => &["你", "下家", "上家"],
        _ => &["你", "下家", "對家", "上家"],
    };
    names[(seat.index() + players - HUMAN.index()) % players]
}

// ---------------------------------------------------------------------------
// Settings: what the page's settings form sends
// ---------------------------------------------------------------------------

/// One field per control of the page's settings form, named alike.  Absent
/// fields take their defaults, so a stored settings object from an older
/// page still loads.
#[derive(Deserialize)]
#[serde(default)]
struct Settings {
    /// 胡 to win: 10 for 十胡, 8 for 八胡
    threshold: u32,
    /// Players at the table, the human among them
    players: usize,
}

impl Default for Settings {
    fn default() -> Self {
        let rules = Rules::new();
        Self {
            threshold: rules.threshold,
            players: rules.players,
        }
    }
}

impl Settings {
    /// From the page's JSON; anything unreadable is the default table
    fn parse(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
    }

    fn rules(&self) -> Rules {
        let mut rules = Rules::new();
        rules.threshold = self.threshold;
        // Stored settings are the page's to edit; a table the engine cannot
        // deal is the nearest one it can.
        rules.players = self.players.clamp(2, Seat::COUNT);
        rules
    }
}

// ---------------------------------------------------------------------------
// Snapshot: what the page draws
// ---------------------------------------------------------------------------

/// One move the hint rates: a card code to discard, or `pass`, `pong`,
/// `eat<index>`, and what the bot makes of it, its pick being the highest.
/// Where [`ExpectBot`] searches (`points`) that is the expected points,
/// plus or minus; elsewhere, `HeuristicBot`'s grade, a priority without a
/// unit, `null` when the move leaves the hand dead.
#[derive(Serialize)]
struct HintJson {
    code: String,
    value: Option<f64>,
    points: bool,
}

#[derive(Serialize)]
struct CardJson {
    code: String,
    glyph: char,
    color: &'static str,
}

impl From<Card> for CardJson {
    fn from(card: Card) -> Self {
        Self {
            code: [card.color.letter(), card.rank.letter()]
                .into_iter()
                .collect(),
            glyph: card.glyph(),
            color: match card.color {
                Color::Red => "red",
                Color::Yellow => "yellow",
                Color::Green => "green",
                Color::White => "white",
            },
        }
    }
}

fn cards(meld: &Meld) -> Vec<CardJson> {
    meld.cards().iter().map(CardJson::from).collect()
}

/// A meld on the table with the card it took from the table first, as the
/// log reads and as a Japanese mahjong table lays out a chi taken from the
/// left: 車 eaten with 馬包 reads 車馬包, and 包 eaten with 車馬 reads 包車馬.
fn claimed(meld: &Meld, card: Card) -> Vec<CardJson> {
    let mut row: Vec<Card> = meld.cards().without(card).iter().collect();
    row.insert(0, card);
    row.into_iter().map(CardJson::from).collect()
}

/// The cards `seat` ate, ponged and konged, one for each of
/// [`Round::exposed`] in the same order
fn claims(round: &Round, seat: Seat) -> impl Iterator<Item = Card> {
    round.events().iter().filter_map(move |event| match *event {
        Event::Eat { seat: s, card, .. }
        | Event::Pong { seat: s, card }
        | Event::Kong { seat: s, card }
            if s == seat =>
        {
            Some(card)
        }
        _ => None,
    })
}

/// [`Round::exposed`] for the page: each meld through [`claimed`]
fn exposed(round: &Round, seat: Seat) -> Vec<Vec<CardJson>> {
    let melds = round.exposed(seat).iter();
    melds
        .zip(claims(round, seat))
        .map(|(meld, card)| claimed(meld, card))
        .collect()
}

/// A hand in card order, colour by colour from 將 down to 包, with the
/// soldiers last.
///
/// No melds are set apart: but for the soldiers, a card only ever melds
/// with its own colour, so the colours already read like the suits of a
/// bridge hand.  Soldiers go together because across colours they make
/// 三色卒.
fn arrange(hand: Hand) -> Vec<CardJson> {
    let (soldiers, others): (Vec<Card>, Vec<Card>) =
        hand.iter().partition(|card| card.rank == Rank::Soldier);
    let cards = others.into_iter().chain(soldiers);
    cards.map(CardJson::from).collect()
}

/// The most 胡 the melds within `hand` make, the rest left loose
fn hand_hu(rules: &Rules, hand: Hand) -> u32 {
    let best = Solver::new(rules.concealed, Some(0)).locked().best(hand);
    best.and_then(|hu| hu.try_into().ok()).unwrap_or(0)
}

/// The 胡 of the cards the human sees at `seat`: the melds on the table,
/// plus the best melds of the hand where it `shown`.  A winner's is the
/// winning count, which has the winning card too.
fn visible_hu(rules: &Rules, round: &Round, seat: Seat, shown: bool) -> u32 {
    if let Some(result) = round.result()
        && result.winner == Some(seat)
    {
        return result.hu;
    }
    let exposed = round.exposed(seat).iter();
    exposed.map(|meld| rules.exposed.score(meld)).sum::<u32>()
        + if shown {
            hand_hu(rules, round.hand(seat))
        } else {
            0
        }
}

#[derive(Serialize)]
struct SeatJson {
    name: &'static str,
    /// 胡 so far in the cards the human sees here: see [`visible_hu`]
    hu: u32,
    concealed: usize,
    exposed: Vec<Vec<CardJson>>,
    /// The human's hand always; a bot's only once the round is over.
    /// In [`arrange`]'s order: the soldiers last.
    hand: Option<Vec<CardJson>>,
}

#[derive(Serialize)]
struct OfferJson {
    card: CardJson,
    source: usize,
    drawn: bool,
}

#[derive(Serialize)]
struct OptionsJson {
    pong: bool,
    /// A drawn general: one of `eats` must be taken, there is no pass
    forced: bool,
    eats: Vec<Vec<CardJson>>,
    /// Claims shown disabled: taking them would leave nothing to discard
    blocked_pong: bool,
    blocked_eats: Vec<Vec<CardJson>>,
}

/// A card that would win the human's hand, the 胡 it wins at, and how many
/// copies the human has not seen
#[derive(Serialize)]
struct WaitJson {
    card: CardJson,
    hu: u32,
    left: u8,
}

/// A discard that leaves the human's hand waiting, and on what
#[derive(Serialize)]
struct DiscardWaitsJson {
    code: String,
    waits: Vec<WaitJson>,
}

/// The cards the human has not seen: in other hands or the pile, counted
/// as `HeuristicBot` counts them
fn unseen(round: &Round) -> Hand {
    let shown = Seat::ALL
        .into_iter()
        .flat_map(|seat| round.exposed(seat))
        .flat_map(|meld| meld.cards())
        .chain(round.discards().iter().map(|&(_, card)| card))
        .chain(round.offer().map(|offer| offer.card));
    shown.fold(Hand::FULL - round.hand(HUMAN), Hand::without)
}

/// The cards that would win `hand` beside the human's exposed melds
fn waits(round: &Round, hand: Hand, unseen: Hand) -> Vec<WaitJson> {
    Card::ALL
        .into_iter()
        .filter_map(|card| {
            let (hu, _) = round.win(HUMAN, hand, card)?;
            Some(WaitJson {
                card: card.into(),
                hu,
                left: unseen.count(card),
            })
        })
        .collect()
}

#[derive(Serialize)]
struct DiscardJson {
    seat: usize,
    card: CardJson,
}

#[derive(Serialize)]
struct ResultJson {
    winner: Option<usize>,
    foul: Option<usize>,
    hu: u32,
    /// The winner's melds on the table
    exposed: Vec<Vec<CardJson>>,
    /// The winner's hand through [`arrange`], without the winning card
    hand: Vec<CardJson>,
    winning_card: Option<CardJson>,
    from: Option<usize>,
}

#[derive(Serialize)]
struct Snapshot {
    phase: &'static str,
    actor: Option<usize>,
    human: usize,
    dealer: usize,
    /// 胡 to win, for the page's wording
    threshold: u32,
    pile: usize,
    /// The cards at the bottom of the pile that are never drawn
    dead: usize,
    /// One per seat at the table, as are `seats`
    scores: Vec<i32>,
    seats: Vec<SeatJson>,
    offer: Option<OfferJson>,
    /// The human's claim options while the round waits on their claim
    options: Option<OptionsJson>,
    /// The cards the human may not discard: the one just claimed and the
    /// generals
    locked: Vec<String>,
    discards: Vec<DiscardJson>,
    /// The human's waits (聽牌), empty while they discard or the round is over
    waits: Vec<WaitJson>,
    /// While the human discards, each discard that leaves them waiting
    discard_waits: Vec<DiscardWaitsJson>,
    log: Vec<String>,
    result: Option<ResultJson>,
}

fn describe(players: usize, event: &Event) -> String {
    let name = |seat| name(players, seat);
    match *event {
        Event::Draw { seat, card } => format!("{}摸 {card}", name(seat)),
        Event::Discard { seat, card } => format!("{}打 {card}", name(seat)),
        Event::Eat {
            seat,
            meld: Meld::Single(_),
            card,
        } => format!("{}吃 {card}", name(seat)),
        Event::Eat { seat, meld, card } => format!("{}吃 {card}：{meld}", name(seat)),
        Event::Pong { seat, card } => format!("{}碰 {card}", name(seat)),
        Event::Kong { seat, card } => format!("{}槓 {card}", name(seat)),
        Event::Win { seat, hu } => format!("{}胡牌，{hu} 胡", name(seat)),
        Event::Foul { seat } => format!("{}相公", name(seat)),
        Event::Exhausted => "流局".to_owned(),
    }
}

fn snapshot(game: &Game, round: &Round) -> Snapshot {
    let players = game.rules().players;
    let finished = round.phase() == Phase::Finished;
    let (phase, actor) = match round.phase() {
        Phase::Claim(seat) => ("claim", Some(seat.index())),
        Phase::Discard(seat) => ("discard", Some(seat.index())),
        Phase::Finished => ("finished", None),
    };
    let options = (round.phase() == Phase::Claim(HUMAN)).then(|| {
        let options = round.claim_options(HUMAN);
        OptionsJson {
            pong: options.pong,
            forced: options.forced(),
            eats: options.eats.iter().map(cards).collect(),
            blocked_pong: options.blocked.contains(&Claim::Pong),
            blocked_eats: options
                .blocked
                .iter()
                .filter_map(|claim| match claim {
                    Claim::Eat(meld) => Some(cards(meld)),
                    _ => None,
                })
                .collect(),
        }
    });
    Snapshot {
        phase,
        actor,
        human: HUMAN.index(),
        dealer: round.dealer().index(),
        threshold: game.rules().threshold,
        pile: round.pile_len(),
        dead: game.rules().dead(),
        scores: game.scores()[..players].to_vec(),
        seats: Seat::ALL[..players]
            .iter()
            .map(|&seat| {
                let shown = seat == HUMAN || finished;
                SeatJson {
                    name: name(players, seat),
                    hu: visible_hu(game.rules(), round, seat, shown),
                    concealed: round.hand(seat).len(),
                    exposed: exposed(round, seat),
                    hand: shown.then(|| arrange(round.hand(seat))),
                }
            })
            .collect(),
        offer: round.offer().map(|offer| OfferJson {
            card: offer.card.into(),
            source: offer.source.index(),
            drawn: offer.drawn,
        }),
        options,
        locked: if round.phase() == Phase::Discard(HUMAN) {
            let hand = round.hand(HUMAN);
            (hand - discardable(hand, round.locked()))
                .distinct()
                .map(|(card, _)| CardJson::from(card).code)
                .collect()
        } else {
            Vec::new()
        },
        discards: round
            .discards()
            .iter()
            .map(|&(seat, card)| DiscardJson {
                seat: seat.index(),
                card: card.into(),
            })
            .collect(),
        waits: match round.phase() {
            Phase::Discard(HUMAN) | Phase::Finished => Vec::new(),
            _ => waits(round, round.hand(HUMAN), unseen(round)),
        },
        discard_waits: if round.phase() == Phase::Discard(HUMAN) {
            let hand = round.hand(HUMAN);
            let unseen = unseen(round);
            discardable(hand, round.locked())
                .distinct()
                .map(|(card, _)| DiscardWaitsJson {
                    code: CardJson::from(card).code,
                    waits: waits(round, hand.without(card), unseen),
                })
                .filter(|discard| !discard.waits.is_empty())
                .collect()
        } else {
            Vec::new()
        },
        log: round
            .events()
            .iter()
            .map(|event| describe(players, event))
            .collect(),
        result: round.result().map(|result| ResultJson {
            winner: result.winner.map(Seat::index),
            foul: result.foul.map(Seat::index),
            hu: result.hu,
            exposed: result
                .winner
                .map_or_else(Vec::new, |seat| exposed(round, seat)),
            hand: result
                .winner
                .map_or_else(Vec::new, |seat| arrange(round.hand(seat))),
            winning_card: result.winning_card.map(CardJson::from),
            from: result.from.map(Seat::index),
        }),
    }
}

// ---------------------------------------------------------------------------
// Core: the game, testable without a browser
// ---------------------------------------------------------------------------

struct Core {
    game: Game,
    table: Table,
    rng: StdRng,
    /// Whether the opponents search: the desktop client's, too slow as wasm
    search: bool,
}

impl Core {
    fn new(seed: u64, settings: &Settings, search: bool) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let rules = settings.rules();
        let game = Game::new(rules, Seat::ALL[rng.random_range(..rules.players)]);
        let table = Self::seat(&game, &mut rng, search);
        Self {
            game,
            table,
            rng,
            search,
        }
    }

    fn seat(game: &Game, rng: &mut StdRng, search: bool) -> Table {
        let rules = game.rules();
        let bots = Seat::ALL.map(|seat| -> Option<Box<dyn Strategy>> {
            if seat == HUMAN {
                None
            } else if search {
                Some(Box::new(bot(rules, true)))
            } else {
                Some(Box::new(HeuristicBot::new(rules)))
            }
        });
        Table::new(game.deal(rng), bots)
    }

    fn snapshot(&self) -> String {
        serde_json::to_string(&snapshot(&self.game, self.table.round()))
            .expect("snapshot serialization")
    }

    /// One bot action; a no-op while the human must act or the round is over
    fn step(&mut self) {
        // The bots only ever answer with offered claims and held cards.
        let _ = self.table.step();
    }

    /// The human's claim; an illegal one is ignored
    fn claim(&mut self, claim: Claim) {
        let _ = self.table.round_mut().claim(HUMAN, claim);
    }

    fn eat(&mut self, index: usize) {
        let options = self.table.round().claim_options(HUMAN);
        if let Some(&meld) = options.eats.get(index) {
            self.claim(Claim::Eat(meld));
        }
    }

    /// The human's discard by card code; an illegal one is ignored
    fn discard(&mut self, code: &str) {
        if let Ok(card) = code.parse::<Card>() {
            let _ = self.table.round_mut().discard(HUMAN, card);
        }
    }

    /// How [`ExpectBot`] rates every move in the human's place, its own pick
    /// first; empty when it is not the human's decision
    fn hint(&self) -> Vec<HintJson> {
        let round = self.table.round();
        let mut bot = bot(self.game.rules(), self.search);
        let view = View::new(round, HUMAN);
        let discarding = matches!(round.phase(), Phase::Discard(_));
        let points = bot.searches(&view, round.hand(HUMAN).len() - usize::from(discarding));
        let rated: Vec<(String, f64)> = match round.phase() {
            Phase::Claim(HUMAN) => {
                let options = round.claim_options(HUMAN);
                bot.claims(&view, &options)
                    .into_iter()
                    .map(|(claim, grade)| {
                        let code = match claim {
                            Claim::Pass => "pass".to_owned(),
                            Claim::Pong => "pong".to_owned(),
                            Claim::Eat(meld) => {
                                let i = options.eats.iter().position(|&m| m == meld);
                                format!("eat{}", i.expect("the bot eats an offered meld"))
                            }
                        };
                        (code, grade)
                    })
                    .collect()
            }
            Phase::Discard(HUMAN) => bot
                .discards(&view)
                .into_iter()
                .map(|(card, grade)| (CardJson::from(card).code, grade))
                .collect(),
            _ => Vec::new(),
        };
        rated
            .into_iter()
            .map(|(code, value)| HintJson {
                code,
                // A dead hand grades about -32000, far below any live one;
                // the DP prices a dead hand in points like any other.
                value: (points || value > -16_000.0).then_some(value),
                points,
            })
            .collect()
    }

    /// Settle the finished round and deal the next
    fn next_round(&mut self) {
        let Some(result) = self.table.round().result() else {
            return;
        };
        self.game.settle(result);
        self.table = Self::seat(&self.game, &mut self.rng, self.search);
    }
}

// ---------------------------------------------------------------------------
// The wasm surface
// ---------------------------------------------------------------------------

/// A game of 十胡 against one to three bots
#[wasm_bindgen]
pub struct WebGame {
    core: Core,
}

#[wasm_bindgen]
impl WebGame {
    /// `seed` is a decimal string from JS (wasm has no entropy); `settings`
    /// is the settings form as JSON, e.g. `{"threshold":8,"players":3}`
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(seed: &str, settings: &str) -> Self {
        Self {
            core: Core::new(seed.parse().unwrap_or(0), &Settings::parse(settings), false),
        }
    }

    /// The current state as JSON
    #[must_use]
    pub fn snapshot(&self) -> String {
        self.core.snapshot()
    }

    /// Let one bot act, then the state
    pub fn step(&mut self) -> String {
        self.core.step();
        self.core.snapshot()
    }

    /// Decline the offered card
    pub fn pass(&mut self) -> String {
        self.core.claim(Claim::Pass);
        self.core.snapshot()
    }

    /// Pong the offered card
    pub fn pong(&mut self) -> String {
        self.core.claim(Claim::Pong);
        self.core.snapshot()
    }

    /// Eat the offered card into the `index`-th option
    pub fn eat(&mut self, index: usize) -> String {
        self.core.eat(index);
        self.core.snapshot()
    }

    /// Discard the card with this code, e.g. `RK`
    pub fn discard(&mut self, code: &str) -> String {
        self.core.discard(code);
        self.core.snapshot()
    }

    /// The greedy bot's rating of your moves as a JSON array, its pick
    /// first: `[{"code": "RK", "behind": 0, "points": true}, ...]`, where
    /// `code` is a card code, `pass`, `pong` or `eat<index>` and `behind` how
    /// far it falls behind the pick: expected points when `points`, where
    /// [`ExpectBot`] searches, else `HeuristicBot`'s grade, `null` for a move
    /// that leaves the hand dead (short of the 胡 to win however the cards
    /// come); empty when it is not your decision
    #[must_use]
    pub fn hint(&self) -> String {
        serde_json::to_string(&self.core.hint()).expect("hints serialize")
    }

    /// Settle the round and deal the next
    pub fn next_round(&mut self) -> String {
        self.core.next_round();
        self.core.snapshot()
    }
}

impl WebGame {
    /// [`WebGame::new`] with opponents that search: an [`ExpectBot`] that
    /// plays by the single-player DP with 14 concealed cards or fewer and by
    /// a shallow graph before, and the hint by the same.  For the desktop
    /// client, as plain Rust: a decision at the gate takes some 190 ms of
    /// native code, 1.4 s at worst.
    #[must_use]
    pub fn searching(seed: &str, settings: &str) -> Self {
        Self {
            core: Core::new(seed.parse().unwrap_or(0), &Settings::parse(settings), true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsap_oo::Step;

    /// The settings JSON picks the table; junk and absent fields default.
    #[test]
    fn settings_pick_the_threshold() {
        assert_eq!(Settings::parse("junk").threshold, 10);
        assert_eq!(Settings::parse("{}").threshold, 10);
        let core = Core::new(7, &Settings::parse(r#"{"threshold":8}"#), false);
        let value: serde_json::Value = serde_json::from_str(&core.snapshot()).unwrap();
        assert_eq!(value["threshold"], 8);
        assert_eq!(core.game.rules(), &Rules::eight());
    }

    /// A table of three shows three seats and is dealt to three; a table
    /// the engine cannot deal is the nearest it can.
    #[test]
    fn settings_pick_the_table() {
        assert_eq!(Settings::parse("{}").players, 4);
        assert_eq!(Settings::parse(r#"{"players":9}"#).rules().players, 4);
        assert_eq!(Settings::parse(r#"{"players":0}"#).rules().players, 2);
        let core = Core::new(7, &Settings::parse(r#"{"players":3}"#), false);
        let value: serde_json::Value = serde_json::from_str(&core.snapshot()).unwrap();
        assert_eq!(value["seats"].as_array().unwrap().len(), 3);
        assert_eq!(value["scores"].as_array().unwrap().len(), 3);
        assert_eq!(value["seats"][2]["name"], "上家");
        assert!(core.table.round().hand(Seat::ALL[3]).is_empty());
    }

    /// A hand lies in card order, melds or not, with every soldier last; a
    /// lone general still counts its 胡.
    #[test]
    fn a_hand_lies_in_card_order_soldiers_apart() {
        let card = |color, rank| Card::new(color, rank);
        let code = |card: Card| CardJson::from(card).code;
        let stray = card(Color::Red, Rank::Elephant);
        let cannon = card(Color::Red, Rank::Cannon);
        let red = card(Color::Red, Rank::Soldier);
        let general = card(Color::Green, Rank::General);
        let advisor = card(Color::Green, Rank::Advisor);
        let green = card(Color::Green, Rank::Soldier);
        let hand = Hand::from_iter([green, advisor, red, general, cannon, stray]);
        let codes: Vec<String> = arrange(hand).into_iter().map(|card| card.code).collect();
        let order = [stray, cannon, general, advisor, red, green];
        assert_eq!(codes, order.map(code));
        // The general alone is 1 胡; its 象 makes 將士象 for 2.
        let hand = Hand::from_iter([general, advisor, stray]);
        assert_eq!(hand_hu(&Rules::new(), hand), 1);
        let elephant = card(Color::Green, Rank::Elephant);
        assert_eq!(hand_hu(&Rules::new(), hand.with(elephant)), 2);
    }

    /// With a bot in the human's seat, every win of theirs on a card comes
    /// on one of the waits shown just before, at the 胡 shown, and each
    /// discard leaves the waits shown for it.
    #[test]
    fn waits_foretell_the_win() {
        let mut rng = StdRng::seed_from_u64(7);
        let game = Game::new(Rules::new(), HUMAN);
        let mut wins = 0;
        while wins < 5 {
            let bots = Seat::ALL
                .map(|_| Some(Box::new(HeuristicBot::new(game.rules())) as Box<dyn Strategy>));
            let mut table = Table::new(game.deal(&mut rng), bots);
            let pairs = |waits: &[WaitJson]| -> Vec<(String, u32)> {
                waits.iter().map(|w| (w.card.code.clone(), w.hu)).collect()
            };
            let mut waits = Vec::new();
            let mut foretold = None;
            while table.round().phase() != Phase::Finished {
                let snapshot = snapshot(&game, table.round());
                if table.round().phase() == Phase::Discard(HUMAN) {
                    assert!(snapshot.waits.is_empty());
                    foretold = Some((snapshot.discard_waits, table.round().events().len()));
                } else {
                    assert!(snapshot.discard_waits.is_empty());
                    if let Some((discards, n)) = foretold.take() {
                        // An unclaimed draw is logged as its drawer's
                        // discard too, so take the event this discard pushed.
                        let Event::Discard { card, .. } = table.round().events()[n] else {
                            unreachable!("the human discarded");
                        };
                        let code = CardJson::from(card).code;
                        let shown = discards.iter().find(|d| d.code == code);
                        assert_eq!(
                            shown.map_or_else(Vec::new, |d| pairs(&d.waits)),
                            pairs(&snapshot.waits),
                        );
                    }
                    waits = snapshot.waits;
                }
                table.step().unwrap();
            }
            // The round can end on the draw the human's discard brings on.
            if let Some((discards, n)) = foretold {
                let Event::Discard { card, .. } = table.round().events()[n] else {
                    unreachable!("the human discarded");
                };
                let code = CardJson::from(card).code;
                waits = discards
                    .into_iter()
                    .find(|d| d.code == code)
                    .map_or_else(Vec::new, |d| d.waits);
            }
            let result = table.round().result().unwrap();
            if let (Some(HUMAN), Some(card)) = (result.winner, result.winning_card) {
                let code = CardJson::from(card).code;
                let wait = waits.iter().find(|wait| wait.card.code == code);
                assert_eq!(wait.map(|wait| wait.hu), Some(result.hu));
                wins += 1;
            }
        }
    }

    /// Drive whole rounds: the bots step, the human passes every claim and
    /// discards their first legal card, and every snapshot serializes.
    #[test]
    fn human_can_muddle_through() {
        let mut core = Core::new(7, &Settings::default(), false);
        let mut rounds = 0;
        while rounds < 5 {
            let json = core.snapshot();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["human"], HUMAN.index());
            let shown = value["seats"][HUMAN.index()]["hand"].as_array().unwrap();
            assert_eq!(
                shown.len(),
                core.table.round().hand(HUMAN).len(),
                "arranging keeps every card"
            );
            let rules = core.game.rules();
            for seat in Seat::ALL {
                let round = core.table.round();
                let exposed = round.exposed(seat).iter();
                let table: u32 = exposed.map(|meld| rules.exposed.score(meld)).sum();
                let hu = value["seats"][seat.index()]["hu"].as_u64().unwrap();
                assert_eq!(
                    value["seats"][seat.index()]["exposed"]
                        .as_array()
                        .unwrap()
                        .len(),
                    round.exposed(seat).len(),
                    "every exposed meld has its claim"
                );
                if value["result"]["winner"] == seat.index() {
                    assert_eq!(hu, value["result"]["hu"], "a winner shows the win");
                } else if seat == HUMAN || round.phase() == Phase::Finished {
                    assert!(hu >= u64::from(table), "a shown hand only adds");
                } else {
                    assert_eq!(hu, u64::from(table), "a hidden hand counts nothing");
                }
            }
            for wait in value["waits"].as_array().unwrap() {
                assert!(wait["hu"].as_u64().unwrap() >= u64::from(rules.threshold));
            }
            match core.table.round().phase() {
                Phase::Finished => {
                    let result = &value["result"];
                    if let Some(winner) = result["winner"].as_u64() {
                        // The winning card stands apart from the hand.
                        assert_eq!(
                            result["hand"], value["seats"][winner as usize]["hand"],
                            "the result shows the winner's hand"
                        );
                    }
                    assert!(
                        value["seats"][HUMAN.next(Seat::COUNT).index()]["hand"].is_array(),
                        "bots reveal at the end"
                    );
                    core.next_round();
                    rounds += 1;
                }
                Phase::Claim(HUMAN) => {
                    assert!(value["options"].is_object());
                    let hints = core.hint();
                    assert!(
                        hints.iter().all(|hint| hint.value <= hints[0].value),
                        "the pick is the best"
                    );
                    let hint = hints[0].code.clone();
                    let eats = value["options"]["eats"].as_array().unwrap().len();
                    assert!(
                        hint == "pass"
                            || (hint == "pong" && value["options"]["pong"] == true)
                            || (0..eats).any(|i| hint == format!("eat{i}")),
                        "the hint is an offered claim: {hint}"
                    );
                    // A drawn general cannot be passed.
                    if value["options"]["forced"] == true {
                        assert_ne!(hint, "pass");
                        core.eat(0);
                    } else {
                        core.claim(Claim::Pass);
                    }
                }
                Phase::Discard(HUMAN) => {
                    assert!(value["waits"].as_array().unwrap().is_empty());
                    let round = core.table.round();
                    let legal = discardable(round.hand(HUMAN), round.locked());
                    let hints = core.hint();
                    assert_eq!(hints.len(), legal.distinct().count(), "every move is rated");
                    for hint in &hints {
                        assert!(legal.contains(hint.code.parse::<Card>().unwrap()));
                    }
                    let card = legal.first().unwrap();
                    core.discard(&CardJson::from(card).code);
                }
                _ => {
                    assert!(core.hint().is_empty());
                    assert!(
                        value["seats"][HUMAN.next(Seat::COUNT).index()]["hand"].is_null(),
                        "bots stay concealed"
                    );
                    assert!(matches!(core.table.step(), Ok(Step::Acted(_))));
                }
            }
        }
    }
}
