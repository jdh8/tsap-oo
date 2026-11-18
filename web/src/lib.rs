//! Browser front end for 十胡 and 八胡: one human against three greedy bots.
//!
//! The engine holds the whole game; JS is a thin renderer that calls one
//! method per user action or bot step and redraws from the JSON snapshot
//! each returns.  `Core` is plain Rust so the flow is testable natively.

use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};
use serde::{Deserialize, Serialize};
use tsap_oo::susik::{Card, Color, Hand, Meld, Rank, Solver};
use tsap_oo::{
    Claim, Event, ExpectBot, Game, HeuristicBot, Phase, Round, Rules, Seat, Strategy, Table, View,
    discardable,
};
use wasm_bindgen::prelude::*;

const HUMAN: Seat = Seat::ALL[0];

/// The most concealed cards the searching opponents search with: the one
/// gate with an A/B verdict (`doc/expectation.md`, batch 3)
const GATE_CARDS: usize = 10;

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

/// One move the hint rates: a card code to discard, or `pass`, `pong`,
/// `eat<index>`, and how far it falls behind the bot's pick, `null` when it
/// leaves the hand dead.  Inside [`ExpectBot`]'s gate (`points`) that is
/// the expected points it gives up; outside, `HeuristicBot`'s grade, a
/// priority without a unit.
#[derive(Serialize)]
struct HintJson {
    code: String,
    behind: Option<f64>,
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

/// A hand sorted into its best melds, with the loose cards as a last group.
///
/// Lone generals go with the loose cards although each is a meld: there
/// they head their colour, right beside any loose 士 or 象, so the 將士 and
/// 將象 waits read at a glance.  The page marks them with a coin instead.
/// Loose soldiers go last, together: beside a 炮 they make nothing, but
/// with soldiers of other colours they make 三色卒.
fn arrange(rules: &Rules, hand: Hand) -> Vec<Vec<CardJson>> {
    // The bots' grading (loose cards at 1 each), so the grouping shows what
    // they would see in the same hand.
    let layout = Solver::new(rules.concealed, Some(1))
        .locked()
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
        let (soldiers, others): (Vec<Card>, Vec<Card>) =
            loose.iter().partition(|card| card.rank == Rank::Soldier);
        groups.push(
            others
                .into_iter()
                .chain(soldiers)
                .map(CardJson::from)
                .collect(),
        );
    }
    groups
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
    /// The human's waits (聽牌), empty while they discard or the round is over
    waits: Vec<WaitJson>,
    /// While the human discards, each discard that leaves them waiting
    discard_waits: Vec<DiscardWaitsJson>,
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
                    exposed: exposed(round, seat),
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
        log: round.events().iter().map(describe).collect(),
        result: round.result().map(|result| {
            let public = result.winner.map_or(0, |seat| {
                round.exposed(seat).len() + usize::from(result.winning_card.is_some())
            });
            let (exposed, concealed) = result.layout.split_at(public);
            // The winning card is claimed like the rest, so it too sits in
            // the middle of its meld.
            let taken = result
                .winner
                .into_iter()
                .flat_map(|seat| claims(round, seat));
            let taken = taken.chain(result.winning_card);
            ResultJson {
                winner: result.winner.map(Seat::index),
                foul: result.foul.map(Seat::index),
                hu: result.hu,
                exposed: exposed
                    .iter()
                    .zip(taken)
                    .map(|(meld, card)| claimed(meld, card))
                    .collect(),
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
    /// Whether the opponents search: the desktop client's, too slow as wasm
    search: bool,
}

impl Core {
    fn new(seed: u64, settings: &Settings, search: bool) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let game = Game::new(
            settings.rules(),
            Seat::ALL[rng.random_range(..Seat::ALL.len())],
        );
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
                Some(Box::new(ExpectBot::new(rules).gated(GATE_CARDS)))
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
        let mut bot = ExpectBot::new(self.game.rules()).gated(GATE_CARDS);
        let view = View::new(round, HUMAN);
        // ponytail: mirrors `ExpectBot`'s own gate test; give the bot a
        // `searches` method if the gate ever stops being a card count.
        let discarding = matches!(round.phase(), Phase::Discard(_));
        let points = round.hand(HUMAN).len() - usize::from(discarding) <= GATE_CARDS;
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
        let best = rated.first().map_or(0.0, |&(_, value)| value);
        rated
            .into_iter()
            .map(|(code, value)| HintJson {
                code,
                // A dead hand grades about 32000 down, far below any live
                // one; the DP prices a dead hand in points like any other.
                behind: (points || best - value <= 16_000.0).then_some(best - value),
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
    /// far it falls behind the pick: expected points when `points`, inside
    /// [`ExpectBot`]'s gate, else `HeuristicBot`'s grade, `null` for a move
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
    /// plays by the single-player DP with 10 concealed cards or fewer and
    /// greedily before.  For the desktop client, as plain Rust; a decision
    /// at the gate takes some 14 ms of native code, unmeasured as wasm.
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

    /// 大肚: three alike stay a triplet, with the rest of 俥傌炮 loose.
    #[test]
    fn three_alike_stay_together() {
        let hand: Hand = ".俥俥俥傌炮..".parse().unwrap();
        let sizes: Vec<usize> = arrange(&Rules::new(), hand).iter().map(Vec::len).collect();
        assert_eq!(sizes, [3, 2]);
    }

    /// A lone general heads its colour among the loose cards.
    #[test]
    fn lone_general_sits_with_the_loose_cards() {
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

    /// Loose soldiers of every colour go last, after the 炮.
    #[test]
    fn loose_soldiers_go_last() {
        let red = Card::new(Color::Red, Rank::Soldier);
        let cannon = Card::new(Color::Red, Rank::Cannon);
        let green = Card::new(Color::Green, Rank::Soldier);
        let hand = Hand::from_iter([red, cannon, green]);
        let codes: Vec<Vec<String>> = arrange(&Rules::new(), hand)
            .into_iter()
            .map(|group| group.into_iter().map(|card| card.code).collect())
            .collect();
        let code = |card: Card| CardJson::from(card).code;
        assert_eq!(codes, [[code(cannon), code(red), code(green)]]);
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
                    if let Some(card) = result["winning_card"].as_object() {
                        let last = result["exposed"].as_array().unwrap().last().unwrap();
                        let last = last.as_array().unwrap();
                        assert_eq!(
                            last[0]["code"], card["code"],
                            "the winning card leads the last exposed meld"
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
                    let hints = core.hint();
                    assert_eq!(hints[0].behind, Some(0.0), "the pick is the best");
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
                        value["seats"][HUMAN.next().index()]["hand"].is_null(),
                        "bots stay concealed"
                    );
                    assert!(matches!(core.table.step(), Ok(Step::Acted(_))));
                }
            }
        }
    }
}
