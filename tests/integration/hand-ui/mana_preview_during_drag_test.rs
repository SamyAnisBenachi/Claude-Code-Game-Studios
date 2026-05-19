//! PROMPT 1336 — HUD mana label preview during a `Placement` drag
//! (`S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001`).
//!
//! Closes the AC5 / AC6 / AC8 gaps left by PROMPT 1228 and conforms test
//! coverage to the project-canonical `tests/integration/hand-ui/` layout
//! demanded by AC9 (this file supersedes the PROMPT 1228 bin at
//! `tests/integration/hud/hud_mana_preview_during_drag_test.rs`, which is
//! deleted in the same commit).
//!
//! The system under test is [`client::ui::hud::sync_mana_text_system`],
//! delegating to the pure helper [`client::ui::hud::project_mana_preview`].
//! All assertions drive `ActivePlacementDrag` and `PendingPlacements` via
//! direct resource insertion (no `Pointer<*>` event synthesis), keeping the
//! suite independent of the R1 drag-pipeline-dead repair (PROMPT 1127 §1).
//! ADR-002 binding preserved — the preview never mutates
//! [`client::presentation::PlayerEconomyView`].

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{ActivePlacementDrag, HandCardCatalog, PendingPlacements, PlacementTargetKind};
use client::ui::hud::{HudEntities, HudPlugin, ManaDisplayState};
use shared::card::{CardCatalog, CardData, CardId, CardType, ClassId, Keyword, Rarity, UnitType};
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const MINION_CARD_ID: u32 = 4_242;
const SPELL_CARD_ID: u32 = 4_243;
const STAGED_CARD_ID: u32 = 4_244;

// ── AC1 + AC2 — Preview activates and paints projected current mana ────────

#[test]
fn ac1_preview_activates_on_minion_drag() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, /* current */ 5, /* reserve */ 2, /* cap */ 8);
    settle_hud_tween(&mut app);

    start_drag_with_card(&mut app, CardId(MINION_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    let state = mana_display_state(&app, entities.mana_label);
    assert_eq!(
        state.current_mana, 5,
        "ManaDisplayState.current_mana mirrors the authoritative pool — preview must NOT mutate it",
    );
    assert_eq!(
        state.reserve_mana, 2,
        "ManaDisplayState.reserve_mana mirrors the authoritative pool — preview must NOT mutate it",
    );
    assert!(
        !state.preview_overdrawn,
        "preview_overdrawn must remain false for an affordable Minion drag",
    );

    assert_eq!(
        text(&app, entities.mana_label),
        "1 / 8",
        "AC2 — HUD mana label paints the projected current (5 - 4 = 1)",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+2 reserve",
        "reserve label stays at authoritative reserve when cost is paid entirely from current",
    );
    assert_eq!(
        app.world().resource::<PlayerEconomyView>().current_mana,
        5,
        "ADR-002 — preview MUST NOT mutate PlayerEconomyView.current_mana",
    );
}

// ── AC3 — Reserve spillover when `cost > current_mana` ─────────────────────

#[test]
fn ac3_preview_spills_into_reserve_when_current_insufficient() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    // current=2 + reserve=3 = 5; cost=4 (MINION_CARD_ID) drains current
    // (2 - 2 = 0) and spills 2 into reserve (3 - 2 = 1).
    apply_economy(&mut app, /* current */ 2, /* reserve */ 3, /* cap */ 8);
    settle_hud_tween(&mut app);

    start_drag_with_card(&mut app, CardId(MINION_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.mana_label),
        "0 / 8",
        "AC3 — projected current clamps to 0 once cost drains the current pool (2 - 2 = 0)",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+1 reserve",
        "AC3 — remaining cost (4 - 2 = 2) spills into reserve, leaving 3 - 2 = 1",
    );
}

// ── AC4 — Preview resets on drag end / cancel / drop ───────────────────────

#[test]
fn ac4_preview_resets_when_drag_clears() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, /* current */ 5, /* reserve */ 2, /* cap */ 8);
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    start_drag_with_card(&mut app, CardId(MINION_CARD_ID));
    settle_hud_tween(&mut app);
    assert_eq!(
        text(&app, entities.mana_label),
        "1 / 8",
        "sanity: AC1 preview must paint 1 / 8 before we clear the drag",
    );

    clear_drag(&mut app);
    settle_hud_tween(&mut app);

    assert_eq!(
        text(&app, entities.mana_label),
        "5 / 8",
        "AC4 — HUD reverts to the authoritative pool on the tick after drag clear",
    );
    let state = mana_display_state(&app, entities.mana_label);
    assert!(
        !state.preview_overdrawn,
        "AC4 — preview_overdrawn marker must NOT survive past drag end",
    );
}

// ── AC5 — Overdraw marker + clamped paint when cost > current + reserve ────

#[test]
fn ac5_overdrawn_marker_set_when_cost_exceeds_combined_pool() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    // Cost 5 (overdraw_card) > current 1 + reserve 1 = 2 combined pool.
    apply_economy(&mut app, /* current */ 1, /* reserve */ 1, /* cap */ 8);
    settle_hud_tween(&mut app);

    start_drag_with_card(&mut app, CardId(OVERDRAW_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    let state = mana_display_state(&app, entities.mana_label);
    assert!(
        state.preview_overdrawn,
        "AC5 — preview_overdrawn marker MUST be true when cost > baseline current + reserve",
    );
    assert_eq!(
        text(&app, entities.mana_label),
        "0 / 8",
        "AC5 — current pool clamps to 0 under overdraw",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "",
        "AC5 — reserve label clears (reserve clamps to 0 under overdraw)",
    );
}

// ── AC6 — Multi-card preview: subtract already-staged spend ────────────────

#[test]
fn ac6_preview_subtracts_already_staged_current_spend() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, /* current */ 6, /* reserve */ 0, /* cap */ 6);
    stage_placement(
        &mut app,
        CardId(STAGED_CARD_ID),
        /* current_mana_spend */ 3,
        /* reserve_mana_spend */ 0,
    );
    settle_hud_tween(&mut app);

    // In-flight Minion: cost = 2 (the small_minion entry).
    start_drag_with_card(&mut app, CardId(SMALL_MINION_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.mana_label),
        "1 / 6",
        "AC6 (current-only staged) — baseline (6 - 3 = 3) minus in-flight cost (2) leaves projected current = 1",
    );
}

#[test]
fn ac6_preview_subtracts_already_staged_reserve_spend() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, /* current */ 0, /* reserve */ 5, /* cap */ 5);
    stage_placement(
        &mut app,
        CardId(STAGED_CARD_ID),
        /* current_mana_spend */ 0,
        /* reserve_mana_spend */ 2,
    );
    settle_hud_tween(&mut app);

    start_drag_with_card(&mut app, CardId(SMALL_MINION_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.mana_label),
        "0 / 5",
        "AC6 (reserve staged) — current pool baseline is 0, projected current stays 0",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+1 reserve",
        "AC6 (reserve staged) — reserve baseline (5 - 2 = 3) minus in-flight reserve spillover (2) leaves projected reserve = 1",
    );
}

// ── AC7 — Preview suppressed outside `Phase::Placement` ────────────────────

#[test]
fn ac7_preview_suppressed_outside_placement_phase() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::DraftShop);
    apply_economy(&mut app, /* current */ 5, /* reserve */ 2, /* cap */ 8);
    settle_hud_tween(&mut app);

    // Drag is forcefully made "active" via direct resource insertion to
    // simulate a brittle path where the upstream suppression failed; the
    // HUD MUST still surface the authoritative readout because the phase
    // gate at `sync_mana_text_system` is defensive.
    start_drag_with_card(&mut app, CardId(MINION_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.mana_label),
        "5 / 8",
        "AC7 — phase != Placement MUST suppress the projection (authoritative paint)",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+2 reserve",
        "AC7 — reserve label remains authoritative outside Placement",
    );
    let state = mana_display_state(&app, entities.mana_label);
    assert!(
        !state.preview_overdrawn,
        "AC7 — overdraw marker MUST NOT be set outside Placement, even with active drag",
    );
}

// ── AC8 — Non-Minion card types: preview explicitly suppressed ─────────────

#[test]
fn ac8_preview_suppressed_for_non_minion_card_types() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    seed_hand_catalog(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    apply_economy(&mut app, /* current */ 5, /* reserve */ 2, /* cap */ 8);
    settle_hud_tween(&mut app);

    start_drag_with_card(&mut app, CardId(SPELL_CARD_ID));
    settle_hud_tween(&mut app);

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.mana_label),
        "5 / 8",
        "AC8 — non-Minion drag MUST NOT alter the HUD mana readout",
    );
    assert_eq!(
        text(&app, entities.reserve_label),
        "+2 reserve",
        "AC8 — non-Minion drag MUST NOT alter the HUD reserve readout",
    );
    let state = mana_display_state(&app, entities.mana_label);
    assert!(
        !state.preview_overdrawn,
        "AC8 — overdraw marker MUST NOT be set when the drag is non-Minion",
    );
}

// ── Harness ────────────────────────────────────────────────────────────────

const OVERDRAW_CARD_ID: u32 = 4_245;
const SMALL_MINION_CARD_ID: u32 = 4_246;

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    // `ActivePlacementDrag`, `HandCardCatalog`, and `PendingPlacements` are
    // normally provided by `HandUiPlugin`. The preview path reads them through
    // `Option<Res<…>>` so the HUD plugin can build without HandUi; the tests
    // construct them by hand to keep the suite independent of the heavier
    // HandUiPlugin wiring (Story 017 / Story 005 dependencies).
    app.init_resource::<ActivePlacementDrag>();
    app.init_resource::<PendingPlacements>();
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have Text"))
        .0
        .clone()
}

fn mana_display_state(app: &App, entity: Entity) -> ManaDisplayState {
    *app.world()
        .get::<ManaDisplayState>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should carry a ManaDisplayState"))
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    app.update();
}

fn apply_economy(app: &mut App, current_mana: u32, reserve_mana: u32, mana_cap: u8) {
    *app.world_mut().resource_mut::<PlayerEconomyView>() = PlayerEconomyView {
        gold: 10,
        current_mana,
        reserve_mana,
        mana_cap,
        initialized: true,
        last_update_source: None,
    };
}

fn start_drag_with_card(app: &mut App, card_id: CardId) {
    let drag_card = app.world_mut().spawn_empty().id();
    let mut drag = app.world_mut().resource_mut::<ActivePlacementDrag>();
    drag.card = Some(drag_card);
    drag.card_id = Some(card_id);
    drag.owner_id = Some(PlayerId(1));
    drag.target_kind = Some(PlacementTargetKind::Minion);
}

fn clear_drag(app: &mut App) {
    *app.world_mut().resource_mut::<ActivePlacementDrag>() = ActivePlacementDrag::default();
}

fn stage_placement(app: &mut App, card_id: CardId, current_spend: u32, reserve_spend: u32) {
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements
        .push(PlacedCardSubmit {
            card_id,
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: current_spend,
            reserve_mana_spend: reserve_spend,
        });
}

fn seed_hand_catalog(app: &mut App) {
    let mut catalog: CardCatalog = std::collections::HashMap::new();
    catalog.insert(
        CardId(MINION_CARD_ID),
        minion_card(MINION_CARD_ID, /* cost */ 4),
    );
    catalog.insert(
        CardId(SMALL_MINION_CARD_ID),
        minion_card(SMALL_MINION_CARD_ID, /* cost */ 2),
    );
    catalog.insert(
        CardId(OVERDRAW_CARD_ID),
        minion_card(OVERDRAW_CARD_ID, /* cost */ 5),
    );
    catalog.insert(
        CardId(SPELL_CARD_ID),
        CardData {
            id: CardId(SPELL_CARD_ID),
            name_fr: "Sort Aperçu".to_string(),
            name_en: "Preview Spell".to_string(),
            class: ClassId::Cra,
            family: None,
            rarity: Rarity::Common,
            card_type: CardType::Spell,
            unit_type: UnitType::Arcane,
            cost: 3,
            atk: 0,
            hp: 0,
            mp: 0,
            ar: 0,
            keywords: Vec::<Keyword>::new(),
            effect_text: String::new(),
            art_id: format!("test_{SPELL_CARD_ID}"),
            pool_copies_override: None,
        },
    );
    app.insert_resource(HandCardCatalog { cards: catalog });
}

fn minion_card(id: u32, cost: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: "Aperçu Mana".to_string(),
        name_en: "Mana Preview".to_string(),
        class: ClassId::Iop,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost,
        atk: 2,
        hp: 3,
        mp: 1,
        ar: 0,
        keywords: Vec::<Keyword>::new(),
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}

/// Run the app long enough that any in-flight HUD numeric tween
/// (`hud_tween_duration_ms` defaults to 300 ms) settles before we read the
/// rendered label text. Mirrors the helper introduced by PROMPT 1228.
fn settle_hud_tween(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_millis(1_000));
    app.update();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}
