//! Browser front end for 十胡 and 八胡: one human against three greedy bots.
//!
//! The engine holds the whole game; JS is a thin renderer that calls one
//! method per user action or bot step and redraws from the JSON snapshot
//! each returns.  `Core` is plain Rust so the flow is testable natively.

use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};
use serde::{Deserialize, Serialize};
use tsap_oo::susik::{Card, Color, Hand, Meld, Solver};
use tsap_oo::{
    Claim, Event, Game, HeuristicBot, Phase, Round, Rules, Seat, Strategy, Table, View, discardable,
};
use wasm_bindgen::prelude::*;

const HUMAN: Seat = Seat::ALL[0];

/// What the human calls each seat, counting on from them in turn order
fn name(seat: Seat) -> &'static str {
    const NAMES: [&str; Seat::COUNT] = ["你", "下家", "對家", "上家"];
    NAMES[(seat.index() + Seat::COUNT - HUMAN.index()) % Seat::COUNT]
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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            threshold: Rules::new().threshold,
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
        rules
    }
}

// ---------------------------------------------------------------------------
// Snapshot: what the page draws
// ---------------------------------------------------------------------------

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

/// A hand sorted into its best melds, with the loose cards as a last group.
///
/// Lone generals go with the loose cards although each is a meld: there
/// they head their colour, right beside any loose 士 or 象, so the 將士 and
/// 將象 waits read at a glance.  The page marks them with a coin instead.
fn arrange(rules: &Rules, hand: Hand) -> Vec<Vec<CardJson>> {
    // The bots' grading (loose cards at 1 each), so the grouping shows what
    // they would see in the same hand.
    let layout = Solver::new(rules.concealed, Some(1))
        .layout(hand)
        .expect("layouts with loose cards always exist");
    let mut loose = layout.loose;
    let mut groups = Vec::new();
    for meld in &layout.melds {
        match meld {
            Meld::Single(_) => loose += meld.cards(),
            _ => groups.push(cards(meld)),
        }
    }
    if !loose.is_empty() {
        groups.push(loose.iter().map(CardJson::from).collect());
    }
    groups
}

/// The most 胡 the melds within `hand` make, the rest left loose
fn hand_hu(rules: &Rules, hand: Hand) -> u32 {
    let best = Solver::new(rules.concealed, Some(0)).best(hand);
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
    /// Grouped by [`arrange`]: the melds, then the loose cards.
    hand: Option<Vec<Vec<CardJson>>>,
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
    /// The winner's melds on the table, ending with the meld of the winning
    /// card: that card is public whoever turned it up
    exposed: Vec<Vec<CardJson>>,
    /// The winner's melds from the hand
    concealed: Vec<Vec<CardJson>>,
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
    scores: [i32; Seat::COUNT],
    seats: Vec<SeatJson>,
    offer: Option<OfferJson>,
    /// The human's claim options while the round waits on their claim
    options: Option<OptionsJson>,
    /// The cards the human may not discard: the one just claimed and the
    /// generals
    locked: Vec<String>,
    discards: Vec<DiscardJson>,
    log: Vec<String>,
    result: Option<ResultJson>,
}

fn describe(event: &Event) -> String {
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
        scores: game.scores(),
        seats: Seat::ALL
            .into_iter()
            .map(|seat| {
                let shown = seat == HUMAN || finished;
                SeatJson {
                    name: name(seat),
                    hu: visible_hu(game.rules(), round, seat, shown),
                    concealed: round.hand(seat).len(),
                    exposed: round.exposed(seat).iter().map(cards).collect(),
                    hand: shown.then(|| arrange(game.rules(), round.hand(seat))),
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
        log: round.events().iter().map(describe).collect(),
        result: round.result().map(|result| {
            let public = result.winner.map_or(0, |seat| {
                round.exposed(seat).len() + usize::from(result.winning_card.is_some())
            });
            let (exposed, concealed) = result.layout.split_at(public);
            ResultJson {
                winner: result.winner.map(Seat::index),
                foul: result.foul.map(Seat::index),
                hu: result.hu,
                exposed: exposed.iter().map(cards).collect(),
                concealed: concealed.iter().map(cards).collect(),
                winning_card: result.winning_card.map(CardJson::from),
                from: result.from.map(Seat::index),
            }
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
}

impl Core {
    fn new(seed: u64, settings: &Settings) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let game = Game::new(
            settings.rules(),
            Seat::ALL[rng.random_range(..Seat::ALL.len())],
        );
        let table = Self::seat(&game, &mut rng);
        Self { game, table, rng }
    }

    fn seat(game: &Game, rng: &mut StdRng) -> Table {
        let bots = Seat::ALL.map(|seat| {
            (seat != HUMAN).then(|| Box::new(HeuristicBot::new(game.rules())) as Box<dyn Strategy>)
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

    /// What a greedy bot would do in the human's place: a card code to
    /// discard, or `pass`, `pong`, `eat<index>`; `None` when it is not the
    /// human's decision
    fn hint(&self) -> Option<String> {
        let round = self.table.round();
        let mut bot = HeuristicBot::new(self.game.rules());
        let view = View::new(round, HUMAN);
        match round.phase() {
            Phase::Claim(HUMAN) => {
                let options = round.claim_options(HUMAN);
                Some(match bot.claim(&view, &options) {
                    // A forced claim has no pass; the bot never passes one.
                    Claim::Pass => "pass".to_owned(),
                    Claim::Pong => "pong".to_owned(),
                    Claim::Eat(meld) => {
                        format!("eat{}", options.eats.iter().position(|&m| m == meld)?)
                    }
                })
            }
            Phase::Discard(HUMAN) => Some(CardJson::from(bot.discard(&view)).code),
            _ => None,
        }
    }

    /// Settle the finished round and deal the next
    fn next_round(&mut self) {
        let Some(result) = self.table.round().result() else {
            return;
        };
        self.game.settle(result);
        self.table = Self::seat(&self.game, &mut self.rng);
    }
}

// ---------------------------------------------------------------------------
// The wasm surface
// ---------------------------------------------------------------------------

/// A game of 十胡 against three bots
#[wasm_bindgen]
pub struct WebGame {
    core: Core,
}

#[wasm_bindgen]
impl WebGame {
    /// `seed` is a decimal string from JS (wasm has no entropy); `settings`
    /// is the settings form as JSON, e.g. `{"threshold":8}`
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(seed: &str, settings: &str) -> Self {
        Self {
            core: Core::new(seed.parse().unwrap_or(0), &Settings::parse(settings)),
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

    /// The greedy bot's pick for your decision: a card code, `pass`, `pong`
    /// or `eat<index>`; empty when it is not your decision
    #[must_use]
    pub fn hint(&self) -> String {
        self.core.hint().unwrap_or_default()
    }

    /// Settle the round and deal the next
    pub fn next_round(&mut self) -> String {
        self.core.next_round();
        self.core.snapshot()
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
        let core = Core::new(7, &Settings::parse(r#"{"threshold":8}"#));
        let value: serde_json::Value = serde_json::from_str(&core.snapshot()).unwrap();
        assert_eq!(value["threshold"], 8);
        assert_eq!(core.game.rules(), &Rules::eight());
    }

    /// A lone general heads its colour among the loose cards.
    #[test]
    fn lone_general_sits_with_the_loose_cards() {
        use tsap_oo::susik::Rank;
        let stray = Card::new(Color::Red, Rank::Elephant);
        let general = Card::new(Color::Green, Rank::General);
        let advisor = Card::new(Color::Green, Rank::Advisor);
        let hand = Hand::from_iter([general, advisor, stray]);
        let codes: Vec<Vec<String>> = arrange(&Rules::new(), hand)
            .into_iter()
            .map(|group| group.into_iter().map(|card| card.code).collect())
            .collect();
        let code = |card: Card| CardJson::from(card).code;
        assert_eq!(codes, [[code(stray), code(general), code(advisor)]]);
        // The general alone is 1 胡; its 象 makes 將士象 for 2.
        assert_eq!(hand_hu(&Rules::new(), hand), 1);
        let elephant = Card::new(Color::Green, Rank::Elephant);
        assert_eq!(hand_hu(&Rules::new(), hand.with(elephant)), 2);
    }

    /// Drive whole rounds: the bots step, the human passes every claim and
    /// discards their first legal card, and every snapshot serializes.
    #[test]
    fn human_can_muddle_through() {
        let mut core = Core::new(7, &Settings::default());
        let mut rounds = 0;
        while rounds < 5 {
            let json = core.snapshot();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["human"], HUMAN.index());
            let grouped: usize = value["seats"][HUMAN.index()]["hand"]
                .as_array()
                .unwrap()
                .iter()
                .map(|group| group.as_array().unwrap().len())
                .sum();
            assert_eq!(
                grouped,
                core.table.round().hand(HUMAN).len(),
                "arranging keeps every card"
            );
            let rules = core.game.rules();
            for seat in Seat::ALL {
                let round = core.table.round();
                let exposed = round.exposed(seat).iter();
                let table: u32 = exposed.map(|meld| rules.exposed.score(meld)).sum();
                let hu = value["seats"][seat.index()]["hu"].as_u64().unwrap();
                if value["result"]["winner"] == seat.index() {
                    assert_eq!(hu, value["result"]["hu"], "a winner shows the win");
                } else if seat == HUMAN || round.phase() == Phase::Finished {
                    assert!(hu >= u64::from(table), "a shown hand only adds");
                } else {
                    assert_eq!(hu, u64::from(table), "a hidden hand counts nothing");
                }
            }
            match core.table.round().phase() {
                Phase::Finished => {
                    let result = &value["result"];
                    if let Some(card) = result["winning_card"].as_object() {
                        let last = result["exposed"].as_array().unwrap().last().unwrap();
                        assert!(
                            last.as_array()
                                .unwrap()
                                .iter()
                                .any(|c| c["code"] == card["code"]),
                            "the winning card's meld closes the exposed row"
                        );
                    }
                    assert!(
                        value["seats"][HUMAN.next().index()]["hand"].is_array(),
                        "bots reveal at the end"
                    );
                    core.next_round();
                    rounds += 1;
                }
                Phase::Claim(HUMAN) => {
                    assert!(value["options"].is_object());
                    let hint = core.hint().unwrap();
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
                    let round = core.table.round();
                    let legal = discardable(round.hand(HUMAN), round.locked());
                    let hint: Card = core.hint().unwrap().parse().unwrap();
                    assert!(legal.contains(hint));
                    let card = legal.first().unwrap();
                    core.discard(&CardJson::from(card).code);
                }
                _ => {
                    assert!(core.hint().is_none());
                    assert!(
                        value["seats"][HUMAN.next().index()]["hand"].is_null(),
                        "bots stay concealed"
                    );
                    assert!(matches!(core.table.step(), Ok(Step::Acted(_))));
                }
            }
        }
    }
}
