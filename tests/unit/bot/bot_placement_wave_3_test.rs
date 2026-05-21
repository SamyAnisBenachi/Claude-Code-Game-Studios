//! Wave 3 placement heuristic regression tests (PROMPT 1602).
//!
//! Scope: prove that the bot action loop, when the placement-heuristic
//! resources are all present, walks its hand and submits a legal placement
//! batch instead of the Wave-1 empty-vector fail-safe. Three focused scenarios:
//!
//! 1. **Legal placement selected** — a minion card in hand + a free
//!    spawn-range cell yields one `PlacedCardSubmit` whose target lies inside
//!    the bot's spawn range and whose mana-split is well-formed; the decision
//!    log records `BotDecisionKind::PlacementSubmitted { placements_len: 1 }`.
//! 2. **No-overlap across the batch** — two minion cards in the same hand
//!    occupy two different lanes (or only the first lane, if no other slot is
//!    available) because the staged occupancy view advances between picks.
//! 3. **No legal placement → no-op** — a hand made of only effect cards
//!    (Spell / Order / DoubleFace) produces an empty placement vector and
//!    the decision log records `BotDecisionKind::EmptyPlacementFailsafe`.
//!
//! These tests intentionally bypass the full `BoardPlugin` schedule and run
//! the bot loop in isolation against hand-built resources, mirroring the
//! Wave 2.5 funnel test (`bot_auction_bid_funnel_wave_2_5_test`).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::time::TimePlugin;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{DraftReadySignal, RoundPhase, RoundState};
use server::core::session::config::SessionConfig;
use server::feature::acquisition::PlayerHands;
use server::feature::auction::PendingBotBids;
use server::feature::board::{
    BoardConfig, BoardOccupancy, PlacementSubmissionReceived, SpawnRangeState,
};
use server::feature::bot::{bot_action_loop, BotDecisionKind, BotDecisionLog, BotPlayers, BotState};
use server::foundation::config::CardCatalog;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{GameMode, PlacementTimerMultiplier, PlacedCardSubmit, PlayTarget};
use shared::session::PlayerId;

const BOT: PlayerId = PlayerId(1 << 63);
const HUMAN: PlayerId = PlayerId(11);
const SEED: u64 = 0xC0DE_F00D_BEEF_FACE;

fn card_id(raw: u32) -> CardId {
    CardId(raw)
}

fn test_session() -> SessionConfig {
    let mut team_map = HashMap::new();
    team_map.insert(BOT, 1);
    team_map.insert(HUMAN, 0);
    let mut class_map = HashMap::new();
    class_map.insert(BOT, ClassId::Iop);
    class_map.insert(HUMAN, ClassId::Cra);
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map,
        class_map,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
    }
}

fn make_card(id: u32, card_type: CardType, cost: u32) -> CardData {
    CardData {
        id: card_id(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: None,
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost,
        atk: 2,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: Some(1),
    }
}

fn catalog_from(cards: Vec<CardData>) -> CardCatalog {
    CardCatalog {
        cards: cards.into_iter().map(|card| (card.id, card)).collect(),
    }
}

fn economy_with(current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold: 0,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

/// Build an app wired with the Wave-3 placement-heuristic resources. The bot
/// is placed on team 1 (`PLAYER_B_TEAM_ID`); the catalog, hand, occupancy,
/// spawn range, and economies are caller-supplied so each test can exercise
/// one scenario.
fn make_placement_app(
    hand: Vec<CardId>,
    catalog: CardCatalog,
    economies: PlayerEconomies,
    occupancy: BoardOccupancy,
) -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.add_message::<DraftReadySignal>();
    app.add_message::<PlacementSubmissionReceived>();
    app.add_systems(Update, bot_action_loop);

    let mut round_state = RoundState::new();
    round_state.phase = RoundPhase::Placement;
    round_state.round_number = 1;
    app.insert_resource(round_state);
    app.insert_resource(test_session());

    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(BOT, SEED));
    app.insert_resource(bots);
    app.init_resource::<BotDecisionLog>();

    let mut hands = PlayerHands::default();
    hands.hands.insert(BOT, hand);
    app.insert_resource(hands);

    app.insert_resource(catalog);
    app.insert_resource(economies);
    app.insert_resource(BoardConfig::default());
    app.insert_resource(SpawnRangeState::default());
    app.insert_resource(occupancy);
    app.init_resource::<PendingBotBids>();

    app
}

fn drain_submissions(app: &mut App) -> Vec<PlacementSubmissionReceived> {
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<PlacementSubmissionReceived>>()
        .drain()
        .collect()
}

fn last_decision(app: &App) -> BotDecisionKind {
    app.world()
        .resource::<BotDecisionLog>()
        .last()
        .expect("expected at least one bot decision entry")
        .decision
        .clone()
}

// =============================================================================
// Test 1 — legal placement: bot picks a Minion card and submits one placement
// inside its spawn range.
// =============================================================================

#[test]
fn test_bot_placement_wave3_submits_legal_minion_when_resources_present() {
    // Arrange
    let card = make_card(101, CardType::Minion, 2);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(BOT, economy_with(5, 0));

    let mut app = make_placement_app(
        vec![card.id],
        catalog_from(vec![card.clone()]),
        economies,
        BoardOccupancy::default(),
    );

    // Act
    app.update();

    // Assert: exactly one submission with one placement targeting the bot's
    // spawn range. Team 1 spawns at cell_max = 8 with no fakes destroyed, so
    // the forward-most legal cell is 8.
    let submissions = drain_submissions(&mut app);
    assert_eq!(submissions.len(), 1, "exactly one placement submission");
    let submission = &submissions[0];
    assert_eq!(submission.player, BOT);
    assert!(submission.peer_id.is_none(), "bots carry peer_id = None");
    assert_eq!(submission.placements.len(), 1, "one card placed");

    let placement: &PlacedCardSubmit = &submission.placements[0];
    assert_eq!(placement.card_id, card.id);
    match placement.target {
        PlayTarget::BoardCell { lane, cell } => {
            assert_eq!(lane, 1, "first available lane picked");
            assert_eq!(cell, 8, "team B forward-most spawn cell");
        }
        ref other => panic!("expected BoardCell target, got {:?}", other),
    }
    assert_eq!(
        placement.current_mana_spend + placement.reserve_mana_spend,
        card.cost,
        "split sums to card cost"
    );
    assert_eq!(
        placement.current_mana_spend, card.cost,
        "bot pays from current first when current has the funds"
    );

    // Decision log entry
    match last_decision(&app) {
        BotDecisionKind::PlacementSubmitted { placements_len } => {
            assert_eq!(placements_len, 1);
        }
        other => panic!("expected PlacementSubmitted, got {:?}", other),
    }
}

// =============================================================================
// Test 2 — no-overlap: two minion cards in hand pick two distinct lanes
// because the staged occupancy view advances between picks.
// =============================================================================

#[test]
fn test_bot_placement_wave3_two_minions_pick_distinct_lanes_no_overlap() {
    // Arrange
    let card_a = make_card(201, CardType::Minion, 1);
    let card_b = make_card(202, CardType::Minion, 1);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(BOT, economy_with(5, 0));

    let mut app = make_placement_app(
        vec![card_a.id, card_b.id],
        catalog_from(vec![card_a.clone(), card_b.clone()]),
        economies,
        BoardOccupancy::default(),
    );

    // Act
    app.update();

    // Assert: exactly one submission with two placements, each in a different
    // lane (no two minions share the same (player, lane) slot).
    let submissions = drain_submissions(&mut app);
    assert_eq!(submissions.len(), 1);
    let placements = &submissions[0].placements;
    assert_eq!(placements.len(), 2, "both minions placed in the same batch");

    let lanes: Vec<u8> = placements
        .iter()
        .map(|p| match p.target {
            PlayTarget::BoardCell { lane, .. } => lane,
            ref other => panic!("expected BoardCell, got {:?}", other),
        })
        .collect();
    assert_ne!(
        lanes[0], lanes[1],
        "two minions must land in distinct lanes; got lanes={:?}",
        lanes
    );
    assert_eq!(lanes[0], 1, "first minion lands in lane 1");
    assert_eq!(lanes[1], 2, "second minion advances to lane 2");

    match last_decision(&app) {
        BotDecisionKind::PlacementSubmitted { placements_len } => {
            assert_eq!(placements_len, 2);
        }
        other => panic!("expected PlacementSubmitted, got {:?}", other),
    }
}

// =============================================================================
// Test 3 — no-legal-placement → empty fail-safe + decision log records
// `EmptyPlacementFailsafe`. Uses a Spell-only hand because the Wave-3
// heuristic owns spatial placements only.
// =============================================================================

#[test]
fn test_bot_placement_wave3_no_legal_target_emits_empty_failsafe() {
    // Arrange — hand contains only effect cards the Wave-3 heuristic skips.
    let spell = make_card(301, CardType::Spell, 1);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(BOT, economy_with(5, 0));

    let mut app = make_placement_app(
        vec![spell.id],
        catalog_from(vec![spell.clone()]),
        economies,
        BoardOccupancy::default(),
    );

    // Act
    app.update();

    // Assert: empty placement vector + EmptyPlacementFailsafe decision.
    let submissions = drain_submissions(&mut app);
    assert_eq!(submissions.len(), 1, "one submission still queued");
    assert!(
        submissions[0].placements.is_empty(),
        "no legal placement → empty vector"
    );

    match last_decision(&app) {
        BotDecisionKind::EmptyPlacementFailsafe => {}
        other => panic!("expected EmptyPlacementFailsafe, got {:?}", other),
    }
}

// =============================================================================
// Test 4 — affordability gate: a minion the bot cannot pay for is skipped
// while a cheaper minion in the same hand still goes through.
// =============================================================================

#[test]
fn test_bot_placement_wave3_skips_unaffordable_card_but_places_affordable() {
    // Arrange — first card is too expensive, second is affordable.
    let expensive = make_card(401, CardType::Minion, 99);
    let affordable = make_card(402, CardType::Minion, 1);
    let mut economies = PlayerEconomies::default();
    economies.0.insert(BOT, economy_with(2, 0));

    let mut app = make_placement_app(
        vec![expensive.id, affordable.id],
        catalog_from(vec![expensive.clone(), affordable.clone()]),
        economies,
        BoardOccupancy::default(),
    );

    // Act
    app.update();

    // Assert: only the affordable card lands.
    let submissions = drain_submissions(&mut app);
    assert_eq!(submissions.len(), 1);
    let placements = &submissions[0].placements;
    assert_eq!(placements.len(), 1, "only the affordable card was placed");
    assert_eq!(placements[0].card_id, affordable.id);
}

// =============================================================================
// Test 5 — degrade-gracefully: when the heuristic-input resources are missing
// (legacy test scaffold), the bot falls back to the empty vector + records
// `EmptyPlacementFailsafe`, matching pre-Wave-3 behaviour.
// =============================================================================

#[test]
fn test_bot_placement_wave3_falls_back_when_heuristic_resources_missing() {
    // Arrange — no BoardConfig / SpawnRangeState / Occupancy / Catalog.
    let mut app = App::new();
    app.add_plugins(TimePlugin);
    app.add_message::<DraftReadySignal>();
    app.add_message::<PlacementSubmissionReceived>();
    app.add_systems(Update, bot_action_loop);

    let mut round_state = RoundState::new();
    round_state.phase = RoundPhase::Placement;
    round_state.round_number = 1;
    app.insert_resource(round_state);
    app.insert_resource(test_session());

    let mut bots = BotPlayers::default();
    bots.insert(BotState::new(BOT, SEED));
    app.insert_resource(bots);
    app.init_resource::<BotDecisionLog>();

    // Act
    app.update();

    // Assert: empty placement + EmptyPlacementFailsafe (Wave-1 fallback path).
    let submissions = drain_submissions(&mut app);
    assert_eq!(submissions.len(), 1);
    assert!(submissions[0].placements.is_empty());
    match last_decision(&app) {
        BotDecisionKind::EmptyPlacementFailsafe => {}
        other => panic!("expected EmptyPlacementFailsafe fallback, got {:?}", other),
    }
}
