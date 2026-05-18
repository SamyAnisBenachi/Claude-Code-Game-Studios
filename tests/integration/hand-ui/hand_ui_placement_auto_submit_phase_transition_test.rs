//! PROMPT 1226 — S18-PLACEMENT-AUTO-SUBMIT-CLIENT-001.
//!
//! Phase-transition auto-submit: when the local client observes a
//! Placement → Resolution phase change, any staged placements that have
//! not yet been manually submitted MUST be sent as one final
//! `C2SSubmitPlacement` BEFORE the existing `PendingPlacements` clear / timer
//! reset path runs (`hand_ui_phase_transition_system`, line 1349 region).
//!
//! Why: PROMPT 1209 (f48583d) added a 250 ms server-side grace window to
//! accept late placement submissions, but the grace window is moot if the
//! client never sends. PROMPT 1210 (c61bab3) repaired the drag cursor
//! coord-space; the staged-but-unsent failure mode then dominated the
//! placement-phase regression set. This test pins the client side of the
//! fix and remains server-authoritative (no local resolution).
//!
//! Coverage:
//! - AC1: staged placement + Placement → Resolution → exactly one
//!   `C2SSubmitPlacement` is queued and `PendingPlacements` is then cleared
//!   by the phase reset path.
//! - AC2: no staged placements + Placement → Resolution → NO
//!   `C2SSubmitPlacement` is queued (short-circuit: `no_pending_placements`).
//! - AC3: already manually submitted + Placement → Resolution → NO
//!   duplicate `C2SSubmitPlacement` is queued (short-circuit:
//!   `already_submitted`).
//! - AC4: no local player identity + staged placement + Placement →
//!   Resolution → NO `C2SSubmitPlacement` is queued (short-circuit:
//!   `no_local_player`).
//! - AC5: Placement → DraftShop (non-Resolution transition) with staged
//!   placement → NO `C2SSubmitPlacement` is queued (short-circuit:
//!   `not_placement_to_resolution`).

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use client::ui::{
    hand::{
        FanSlotIndex, HandCardCatalog, HandContents, HandSubmitButton, HandSubmitButtonClicked,
        HandUiOutboundMessages, HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements,
        PlacementTimer,
    },
    shared::BoardLayout,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn ac1_staged_placement_then_resolution_phase_queues_one_submit_before_clear() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(10)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    // Sanity: a placement is staged and no submission has been queued yet.
    assert_eq!(
        app.world().resource::<PendingPlacements>().staged_count(),
        1,
        "AC1 fixture must have one staged placement before phase transition",
    );
    assert!(
        submissions(&app).is_empty(),
        "AC1 no submission should be queued before the phase transition",
    );
    assert!(
        !app.world().resource::<PlacementTimer>().submitted,
        "AC1 placement timer must report submitted=false before transition",
    );

    // Drive Placement → Resolution via `CurrentClientPhase` (the same edge
    // `phase_sink_system` produces when an `S2CPhaseChanged(Resolution)` is
    // drained off the wire).
    set_phase(&mut app, RoundPhase::Resolution);
    run_update(&mut app);

    let submissions = submissions(&app);
    assert_eq!(
        submissions.len(),
        1,
        "AC1 Placement → Resolution with one staged placement MUST queue \
         exactly one C2SSubmitPlacement before PendingPlacements is cleared; \
         got {} submissions",
        submissions.len()
    );
    assert_eq!(
        submissions[0].placements.len(),
        1,
        "AC1 the auto-submitted C2SSubmitPlacement must carry the staged \
         placement payload; got {:?}",
        submissions[0].placements
    );
    assert_eq!(
        submissions[0].placements[0].card_id,
        CardId(10),
        "AC1 auto-submit payload must contain the staged CardId(10)",
    );
    assert_eq!(
        submissions[0].placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 1 },
        "AC1 auto-submit payload must carry the staged target unchanged",
    );

    // After the auto-submit, the existing phase-reset path MUST still clear
    // `PendingPlacements` (the auto-submit runs before the clear; the clear
    // remains the single owner of post-phase reset).
    assert_eq!(
        app.world().resource::<PendingPlacements>().staged_count(),
        0,
        "AC1 the phase-reset clear path must still run after auto-submit; \
         PendingPlacements must end up empty",
    );
}

#[test]
fn ac2_no_staged_placements_resolution_phase_does_not_queue_submit() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(20), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(20)]);

    // No staging — phase transition should NOT emit a submit.
    set_phase(&mut app, RoundPhase::Resolution);
    run_update(&mut app);

    assert!(
        submissions(&app).is_empty(),
        "AC2 Placement → Resolution with NO staged placements must not \
         queue a C2SSubmitPlacement (short-circuit: no_pending_placements)",
    );
}

#[test]
fn ac3_already_submitted_resolution_phase_does_not_queue_duplicate() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(30), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(30)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    // Click submit manually first.
    click_submit(&mut app);
    assert_eq!(
        submissions(&app).len(),
        1,
        "AC3 manual submit must queue one C2SSubmitPlacement before transition",
    );
    assert!(
        app.world().resource::<PlacementTimer>().submitted,
        "AC3 placement timer must report submitted=true after manual click",
    );

    // Now transition to Resolution — must NOT add a second submit.
    set_phase(&mut app, RoundPhase::Resolution);
    run_update(&mut app);

    assert_eq!(
        submissions(&app).len(),
        1,
        "AC3 Placement → Resolution after a manual submit must NOT queue a \
         duplicate C2SSubmitPlacement (short-circuit: already_submitted); \
         got {} submissions",
        submissions(&app).len()
    );
}

#[test]
fn ac4_no_local_player_resolution_phase_short_circuits_without_submit() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(40), CardType::Minion)]));
    // Intentionally do NOT call set_local_player — `ClientSessionIdentity`
    // stays at its `Default` (player_id = None) so the auto-submit must
    // short-circuit with `no_local_player`.
    set_hand(&mut app, [CardId(40)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    set_phase(&mut app, RoundPhase::Resolution);
    run_update(&mut app);

    assert!(
        submissions(&app).is_empty(),
        "AC4 Placement → Resolution with no local player identity must NOT \
         queue a C2SSubmitPlacement (short-circuit: no_local_player)",
    );
}

#[test]
fn ac5_non_resolution_transition_does_not_queue_auto_submit() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(50), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(50)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );

    // Transition away from Placement to a non-Resolution phase. The
    // auto-submit edge is specifically Placement → Resolution; every other
    // edge must NOT submit (so a Placement → DraftShop rewind doesn't
    // double-send during a reconnect).
    set_phase(&mut app, RoundPhase::DraftShop);
    run_update(&mut app);

    assert!(
        submissions(&app).is_empty(),
        "AC5 Placement → DraftShop (non-Resolution edge) must NOT queue an \
         auto-submit (short-circuit: not_placement_to_resolution)",
    );
}

// ── helpers ──────────────────────────────────────────────────────────────

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog { cards: catalog });
    {
        let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
        economy.current_mana = 10;
        economy.reserve_mana = 10;
    }
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::Placement);
    run_update(&mut app);
    app
}

fn test_catalog<const N: usize>(entries: [(CardId, CardType); N]) -> HashMap<CardId, CardData> {
    entries
        .into_iter()
        .map(|(card_id, card_type)| (card_id, test_card(card_id, card_type)))
        .collect()
}

fn test_card(card_id: CardId, card_type: CardType) -> CardData {
    CardData {
        id: card_id,
        name_fr: format!("Carte {}", card_id.0),
        name_en: format!("Card {}", card_id.0),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{}", card_id.0),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
    run_update(app);
}

fn set_local_player(app: &mut App, player_id: PlayerId) {
    app.world_mut()
        .resource_mut::<ClientSessionIdentity>()
        .player_id = Some(player_id);
}

fn stage_card_in_slot(app: &mut App, slot_index: u8, owner_id: PlayerId, target: PlayTarget) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id,
        target: Some(target),
    });
    run_update(app);
}

fn click_submit(app: &mut App) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn submissions(app: &App) -> Vec<shared::protocol::C2SSubmitPlacement> {
    app.world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .clone()
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn submit_button(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit button should exist")
}
