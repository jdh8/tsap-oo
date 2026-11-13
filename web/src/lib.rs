//! Browser front end for 十胡: one human (east) against three greedy bots.
//!
//! The engine holds the whole game; JS is a thin renderer that calls one
//! method per user action or bot step and redraws from the JSON snapshot
//! each returns.  `Core` is plain Rust so the flow is testable natively.

use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};
use serde::Serialize;
use tsap_oo::susik::{Card, Color, Hand, Meld, Solver};
use tsap_oo::{Claim, Event, Game, HeuristicBot, Phase, Round, Rules, Seat, Strategy, Table};
use wasm_bindgen::prelude::*;

const HUMAN: Seat = Seat::East;

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

/// A hand sorted into its best melds, with the loose cards as a last group
fn arrange(rules: &Rules, hand: Hand) -> Vec<Vec<CardJson>> {
    // The bots' grading (loose cards at 1 each), so the grouping shows what
    // they would see in the same hand.
    let layout = Solver::new(rules.concealed, Some(1))
        .layout(hand)
        .expect("layouts with loose cards always exist");
    let mut groups: Vec<_> = layout.melds.iter().map(cards).collect();
    if !layout.loose.is_empty() {
        groups.push(layout.loose.iter().map(CardJson::from).collect());
    }
    groups
}

#[derive(Serialize)]
struct SeatJson {
    seat: char,
    concealed: usize,
    exposed: Vec<Vec<CardJson>>,
    /// The human's hand always; a bot's only once the round is over.
    /// Grouped by [`arrange`]: the melds, then the loose cards.
    hand: Option<Vec<Vec<CardJson>>>,
}

#[derive(Serialize)]
struct OfferJson {
    card: CardJson,
    source: char,
    drawn: bool,
}

#[derive(Serialize)]
struct OptionsJson {
    pong: bool,
    eats: Vec<Vec<CardJson>>,
}

#[derive(Serialize)]
struct DiscardJson {
    seat: char,
    card: CardJson,
}

#[derive(Serialize)]
struct ResultJson {
    winner: Option<char>,
    hu: u32,
    layout: Vec<Vec<CardJson>>,
    winning_card: Option<CardJson>,
    from: Option<char>,
}

#[derive(Serialize)]
struct Snapshot {
    phase: &'static str,
    actor: Option<char>,
    human: char,
    dealer: char,
    pile: usize,
    scores: [i32; 4],
    seats: Vec<SeatJson>,
    offer: Option<OfferJson>,
    /// The human's claim options while the round waits on their claim
    options: Option<OptionsJson>,
    locked: Option<String>,
    discards: Vec<DiscardJson>,
    log: Vec<String>,
    result: Option<ResultJson>,
}

fn describe(event: &Event) -> String {
    match *event {
        Event::Draw { seat, card } => format!("{seat}摸 {card}"),
        Event::Discard { seat, card } => format!("{seat}打 {card}"),
        Event::Eat { seat, meld, card } => format!("{seat}吃 {card}：{meld}"),
        Event::Pong { seat, card } => format!("{seat}碰 {card}"),
        Event::Kong { seat, card } => format!("{seat}槓 {card}"),
        Event::Win { seat, hu } => format!("{seat}胡牌，{hu} 胡"),
        Event::Exhausted => "流局".to_owned(),
    }
}

fn snapshot(game: &Game, round: &Round) -> Snapshot {
    let finished = round.phase() == Phase::Finished;
    let (phase, actor) = match round.phase() {
        Phase::Claim(seat) => ("claim", Some(seat.letter())),
        Phase::Discard(seat) => ("discard", Some(seat.letter())),
        Phase::Finished => ("finished", None),
    };
    let options = (round.phase() == Phase::Claim(HUMAN)).then(|| {
        let options = round.claim_options(HUMAN);
        OptionsJson {
            pong: options.pong,
            eats: options.eats.iter().map(cards).collect(),
        }
    });
    Snapshot {
        phase,
        actor,
        human: HUMAN.letter(),
        dealer: round.dealer().letter(),
        pile: round.pile_len(),
        scores: game.scores(),
        seats: Seat::ALL
            .into_iter()
            .map(|seat| SeatJson {
                seat: seat.letter(),
                concealed: round.hand(seat).len(),
                exposed: round.exposed(seat).iter().map(cards).collect(),
                hand: (seat == HUMAN || finished).then(|| arrange(game.rules(), round.hand(seat))),
            })
            .collect(),
        offer: round.offer().map(|offer| OfferJson {
            card: offer.card.into(),
            source: offer.source.letter(),
            drawn: offer.drawn,
        }),
        options,
        locked: round.locked().map(|card| CardJson::from(card).code),
        discards: round
            .discards()
            .iter()
            .map(|&(seat, card)| DiscardJson {
                seat: seat.letter(),
                card: card.into(),
            })
            .collect(),
        log: round.events().iter().map(describe).collect(),
        result: round.result().map(|result| ResultJson {
            winner: result.winner.map(Seat::letter),
            hu: result.hu,
            layout: result.layout.iter().map(cards).collect(),
            winning_card: result.winning_card.map(CardJson::from),
            from: result.from.map(Seat::letter),
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
    fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let game = Game::new(Rules::new(), Seat::ALL[rng.random_range(..Seat::ALL.len())]);
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
    /// `seed` is a decimal string from JS (wasm has no entropy)
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(seed: &str) -> Self {
        Self {
            core: Core::new(seed.parse().unwrap_or(0)),
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

    /// Drive whole rounds: the bots step, the human passes every claim and
    /// discards their first legal card, and every snapshot serializes.
    #[test]
    fn human_can_muddle_through() {
        let mut core = Core::new(7);
        let mut rounds = 0;
        while rounds < 5 {
            let json = core.snapshot();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["human"], "E");
            let grouped: usize = value["seats"][0]["hand"]
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
            match core.table.round().phase() {
                Phase::Finished => {
                    assert!(value["result"].is_object());
                    assert!(
                        value["seats"][1]["hand"].is_array(),
                        "bots reveal at the end"
                    );
                    core.next_round();
                    rounds += 1;
                }
                Phase::Claim(HUMAN) => {
                    assert!(value["options"].is_object());
                    core.claim(Claim::Pass);
                }
                Phase::Discard(HUMAN) => {
                    let locked = core.table.round().locked();
                    let card = core
                        .table
                        .round()
                        .hand(HUMAN)
                        .iter()
                        .find(|&c| Some(c) != locked)
                        .unwrap();
                    core.discard(&CardJson::from(card).code);
                }
                _ => {
                    assert!(value["seats"][1]["hand"].is_null(), "bots stay concealed");
                    assert!(matches!(core.table.step(), Ok(Step::Acted(_))));
                }
            }
        }
    }
}
