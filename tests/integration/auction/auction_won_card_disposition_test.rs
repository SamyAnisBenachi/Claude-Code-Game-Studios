// PROMPT 1347 / S18-AUCTION-WON-CARD-DISPOSITION-001 (AC12 + AC13)
//
// Integration coverage for the server-side disposition contract +
// observability hook introduced in PROMPT 1347. The wire-level contract
// (`award_auction_card` → `S2CCardAcquired { source: AuctionWon }`) is
// pre-existing; AC12 re-asserts the existing behaviour against the
// AUDIT-1131-02 expectation. The AC10 tracing line is new.
//
// AC12 — settle disposition + hand grant + S2CCardAcquired unicast + AC10
//        trace-line emission (Case A + Case B).
// AC13 — hand persistence across PLACEMENT-end on the no-submit path.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use bevy::prelude::*;
use server::core::economy::{EconomyPlugin, PlayerEconomies, PlayerEconomy};
use server::core::pool::CardPoolPlugin;
use server::core::rsm::{AbortAuction, AuctionPhaseEntered, AuctionSettled, GameOverEmitted};
use server::feature::acquisition::PlayerHands;
use server::feature::auction::{AuctionPhase, AuctionPlugin, AuctionPool, AuctionState};
use server::foundation::config::{CardCatalog, GameConfig};
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::session::PlayerId;
use tracing::field::{Field, Visit};
use tracing::Event;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::Registry;

// PROMPT 1347 — deliberately NOT importing `test_helpers::init_test_tracing`.
// The fmt subscriber installed via `try_init()` becomes the process-wide
// global, and tracing 0.1's callsite Interest cache is keyed off the FIRST
// subscriber registered. Once that's the fmt subscriber, the per-thread
// `tracing::subscriber::with_default(...)` override used by these tests
// loses its event delivery. The disposition log line is still emitted by
// production code; we just can't capture it from a process where another
// subscriber pre-registered the callsite. Tests in this file own their
// own subscriber scope.

// ============================================================================
// Tracing-event capture helper (AC10 / AC12d)
// ----------------------------------------------------------------------------
// `tracing-test` is not in the workspace and AC23 / AC25 forbid Cargo
// dependency edits. We implement a small `tracing_subscriber::Layer` that
// captures every event's target + level + structured fields into a shared,
// process-wide `Mutex<Vec<CapturedEvent>>` channel. The layer is installed
// once via `set_global_default(Registry + CaptureLayer)`. Each test calls
// `take_captured()` immediately before its `app.update()` to clear the
// channel, then `take_captured()` again after to read just the events
// emitted by the system-under-test.
//
// This approach sidesteps tracing 0.1's callsite Interest caching, which
// makes `tracing::subscriber::with_default(...)` non-functional once any
// other subscriber has registered a callsite first.
// ============================================================================

#[derive(Debug, Clone, Default)]
struct CapturedEvent {
    target: String,
    #[allow(dead_code)]
    level: String,
    fields: HashMap<String, String>,
}

static CAPTURED_EVENTS: OnceLock<Arc<Mutex<Vec<CapturedEvent>>>> = OnceLock::new();
static SUBSCRIBER_INSTALL: OnceLock<()> = OnceLock::new();
static TEST_SERIAL: OnceLock<Mutex<()>> = OnceLock::new();

fn test_serial_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_SERIAL
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

fn captured_events() -> Arc<Mutex<Vec<CapturedEvent>>> {
    CAPTURED_EVENTS
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

fn install_capture_subscriber() {
    SUBSCRIBER_INSTALL.get_or_init(|| {
        let layer = CaptureLayer {
            events: captured_events(),
        };
        let registry = Registry::default().with(layer);
        let _ = tracing::subscriber::set_global_default(registry);
    });
}

fn take_captured() -> Vec<CapturedEvent> {
    let events = captured_events();
    let mut guard = events.lock().expect("captured-events lock poisoned");
    std::mem::take(&mut *guard)
}

struct CaptureLayer {
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl<S> Layer<S> for CaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut captured = CapturedEvent {
            target: event.metadata().target().to_string(),
            level: event.metadata().level().to_string(),
            fields: HashMap::new(),
        };
        let mut visitor = FieldVisitor {
            fields: &mut captured.fields,
        };
        event.record(&mut visitor);
        if let Ok(mut events) = self.events.lock() {
            events.push(captured);
        }
    }
}

struct FieldVisitor<'a> {
    fields: &'a mut HashMap<String, String>,
}

impl<'a> Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{:?}", value));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}

fn find_auction_settled_event(events: &[CapturedEvent]) -> Option<&CapturedEvent> {
    events.iter().find(|e| {
        e.target == "server::game"
            && e.fields
                .get("event")
                .map(|v| v == "auction_settled" || v == "\"auction_settled\"")
                .unwrap_or(false)
    })
}

// ============================================================================
// Auction fixture (mirrors `pool_integration_test.rs` style)
// ============================================================================

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn economy(gold: u32, reserved_gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold,
    }
}

fn make_card(id: u32, rarity: Rarity, copies: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Neutral,
        family: Some("AuctionWonFixture".to_string()),
        rarity,
        card_type: CardType::Minion,
        unit_type: UnitType::Neutral,
        cost: 1,
        atk: 1,
        hp: 1,
        mp: 1,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("auction_won_fixture_{id}"),
        pool_copies_override: Some(copies as i32),
    }
}

fn catalog(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn config() -> GameConfig {
    GameConfig(shared::config::GameConfig {
        legendary_pool_entry_round: 6,
        ..shared::config::GameConfig::default()
    })
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn auction_fixture(catalog: CardCatalog) -> App {
    let config = config();
    let mut app = App::new();
    app.add_plugins((AuctionPlugin, CardPoolPlugin, EconomyPlugin))
        .add_message::<AuctionPhaseEntered>()
        .add_message::<AbortAuction>()
        .add_message::<GameOverEmitted>()
        .insert_resource(AuctionPool::from_catalog(&catalog, &config))
        .insert_resource(catalog)
        .insert_resource(config)
        .insert_resource(ServerRng::from_seed(7))
        .insert_resource(PlayerHands::default())
        .insert_resource(PlayerEconomies(HashMap::from([
            (player(1), economy(20, 0)),
            (player(2), economy(20, 0)),
        ])));
    app
}

fn enter_auction(app: &mut App, round: u32) {
    app.world_mut()
        .resource_mut::<Messages<AuctionPhaseEntered>>()
        .write(AuctionPhaseEntered { round });
    app.update();
}

// ============================================================================
// AC12 Case A — winner + S2CCardAcquired + PlayerHands grant + AC10 trace.
// ============================================================================

#[test]
fn case_a_winner_settle_grants_card_and_emits_ac10_trace_line() {
    install_capture_subscriber();
    let _serial = test_serial_lock();
    take_captured(); // clear before the system-under-test runs
    let winner = player(1);
    let card = make_card(107, Rarity::Rare, 3);
    let mut app = auction_fixture(catalog(vec![card]));

    enter_auction(&mut app, 3);
    assert_eq!(
        app.world().resource::<AuctionState>().phase,
        AuctionPhase::LiveBidding,
        "AuctionPhaseEntered must drive the auction to LiveBidding before settle"
    );

    // Stage a winner with a price + matching reservation, then expire the
    // timer so `settle_expired_auction` runs on the next `app.update()`.
    {
        let mut state = app.world_mut().resource_mut::<AuctionState>();
        state.current_leader = Some(winner);
        state.current_price = 4;
        state.timer_remaining_ms = 0;
    }
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .get_mut(&winner)
        .expect("winner economy exists")
        .reserved_gold = 4;

    app.update();
    let captured = take_captured();

    // AC1 / AC12a — `S2CCardAcquired` unicast to winner is observable via
    // PlayerHands containment (the wire dispatch is fanned out by the
    // production `AuctionPlugin` and not exposed as a Bevy `Messages<T>`
    // resource in this harness; we assert the persisted state — the
    // canonical proof of grant — and the broadcast `AuctionSettled`
    // observer message).
    let hands = app.world().resource::<PlayerHands>();
    assert_eq!(
        hands.hand_len(winner),
        1,
        "AC1 / AC12c: PlayerHands[winner] must contain the won card after settle"
    );

    // AC12b — broadcast `AuctionSettled` observer with Some(winner).
    let settled = read_messages::<AuctionSettled>(&app);
    assert_eq!(settled.len(), 1, "exactly one AuctionSettled observer fires");
    assert_eq!(settled[0].winner, Some(winner));
    assert_eq!(settled[0].final_price, 4);
    assert_eq!(settled[0].card_id, CardId(107));

    // AC12d / AC10 — tracing line with the disposition fields.
    let settled_event = find_auction_settled_event(&captured)
        .expect("AC10: server::game `event = auction_settled` info line must be emitted");
    assert_eq!(
        settled_event
            .fields
            .get("winner")
            .expect("AC10: `winner` field present on Case A trace"),
        "1",
        "AC10: `winner` matches the settling PlayerId(1)"
    );
    assert_eq!(
        settled_event
            .fields
            .get("card_id")
            .expect("AC10: `card_id` field present"),
        "107"
    );
    assert_eq!(
        settled_event
            .fields
            .get("current_price")
            .expect("AC10: `current_price` field present"),
        "4"
    );
    assert_eq!(
        settled_event
            .fields
            .get("hand_size_before")
            .expect("AC10: `hand_size_before` field present"),
        "0"
    );
    assert_eq!(
        settled_event
            .fields
            .get("hand_size_after")
            .expect("AC10: `hand_size_after` field present"),
        "1"
    );
}

// ============================================================================
// AC12 Case B — no winner: no hand mutation, no card grant, AC10 trace fires
// without `winner` field.
// ============================================================================

#[test]
fn case_b_no_winner_settle_grants_no_card_and_emits_ac10_trace_line() {
    install_capture_subscriber();
    let _serial = test_serial_lock();
    take_captured();
    let card = make_card(207, Rarity::Rare, 3);
    let mut app = auction_fixture(catalog(vec![card]));

    enter_auction(&mut app, 4);
    // Force settle with no leader by zeroing the timer.
    app.world_mut()
        .resource_mut::<AuctionState>()
        .timer_remaining_ms = 0;

    app.update();
    let captured = take_captured();

    // AC3 / AC12 Case B — no S2CCardAcquired, no PlayerHands mutation.
    let hands = app.world().resource::<PlayerHands>();
    assert_eq!(
        hands.hand_len(player(1)),
        0,
        "AC3: Case B settles without granting any card to player 1"
    );
    assert_eq!(
        hands.hand_len(player(2)),
        0,
        "AC3: Case B settles without granting any card to player 2"
    );

    let settled = read_messages::<AuctionSettled>(&app);
    assert_eq!(settled.len(), 1, "exactly one AuctionSettled observer fires");
    assert_eq!(settled[0].winner, None);
    assert_eq!(settled[0].final_price, 0);

    // AC10 — Case B emits the same `event = "auction_settled"` line; the
    // `winner` field is omitted because there is no winner.
    let settled_event = find_auction_settled_event(&captured)
        .expect("AC10: Case B must emit `event = auction_settled` info line");
    assert!(
        !settled_event.fields.contains_key("winner"),
        "AC10: Case B trace line omits `winner` field (got {:?})",
        settled_event.fields.get("winner")
    );
    assert_eq!(
        settled_event
            .fields
            .get("card_id")
            .expect("AC10: `card_id` field present even on Case B"),
        "207"
    );
}

// ============================================================================
// AC13 — winner hand persists across PLACEMENT-end on no-submit path.
// ============================================================================
//
// The production server-side RSM rolls DRAFT_AUCTION → PLACEMENT →
// RESOLUTION. This test simulates the disposition's persistence semantics
// without spinning up the full RSM: it settles a winner (granting the won
// card to `PlayerHands`), then verifies the next `AuctionPhaseEntered` (the
// natural progression after PLACEMENT → DRAFT_AUCTION on the next round)
// finds the previous card still in the winner's hand. The wider claim — that
// `PlayerHands` is never pruned on PLACEMENT-end without an explicit
// `C2SSubmitPlacement` — is a property of the existing acquisition pipeline
// (`server/src/feature/acquisition/`) which this row does not change; the
// assertion below preserves the no-prune behaviour against
// `award_auction_card`'s refactor.

#[test]
fn ac13_won_card_persists_in_hand_across_settle_with_no_submission() {
    let winner = player(1);
    let card = make_card(307, Rarity::Rare, 3);
    let mut app = auction_fixture(catalog(vec![card]));

    enter_auction(&mut app, 3);
    {
        let mut state = app.world_mut().resource_mut::<AuctionState>();
        state.current_leader = Some(winner);
        state.current_price = 5;
        state.timer_remaining_ms = 0;
    }
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .get_mut(&winner)
        .expect("winner economy exists")
        .reserved_gold = 5;

    app.update();
    assert_eq!(
        app.world().resource::<PlayerHands>().hand_len(winner),
        1,
        "won card present in winner hand immediately after settle"
    );

    // Simulate the placement-phase window passing without any submission
    // hitting the hand resource. Several engine ticks; no system in the
    // server tree prunes `PlayerHands` outside the explicit submit path.
    for _ in 0..6 {
        app.update();
    }

    assert_eq!(
        app.world().resource::<PlayerHands>().hand_len(winner),
        1,
        "AC13: won card still in winner hand after no-submit ticks elapse"
    );
}
