//! PROMPT 1546 — PLACEMENT-ACCEPTED-ACK-PROTOCOL-IMPLEMENTATION.
//!
//! Hand-UI consumer side of the placement acceptance ACK contract. The
//! existing optimistic-Submitted flow (PROMPT 1399) flips
//! `PlacementTimer::submitted = true` the moment `C2SSubmitPlacement` leaves
//! the local outbox, with no positive server signal until reveal at phase
//! close. The new `S2CPlacementAccepted` lane (PROMPT 1546) records the
//! server-authoritative ACK into `PlacementSubmitAck` so QA snapshot
//! observability (PROMPT 1533/1543) and future UI polish can distinguish
//! "sent, waiting" from "server confirmed accepted".
//!
//! Authority: the ACK only transitions the resource; the optimistic UI
//! state is unchanged (this test asserts both: ack transitions, UI does not
//! regress).

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
        HandSubmitInteractionState, HandUiEntities, HandUiPlacementAcceptedReceived,
        HandUiPlacementDropResolved, HandUiPlacementRejectedReceived, HandUiPlugin,
        PlacementSubmitAck, PlacementTimer,
    },
    shared::BoardLayout,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlacementRejectedReason, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn default_placement_submit_ack_is_not_submitted() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));

    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::NotSubmitted,
        "PlacementSubmitAck must start NotSubmitted before any submit",
    );
}

#[test]
fn optimistic_submit_flag_transitions_ack_to_pending() {
    // The pending-ack watcher reads `placement_timer.submitted` (the
    // optimistic-Submitted flag set by every `submit_pending_placements`
    // caller). We assert the watcher's contract directly so the test is
    // not coupled to the full click-submit pipeline.
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::NotSubmitted,
    );

    app.world_mut()
        .resource_mut::<PlacementTimer>()
        .submitted = true;
    run_update(&mut app);

    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::PendingAck,
        "watcher must move NotSubmitted → PendingAck when placement_timer.submitted flips true",
    );
}

#[test]
fn accepted_ack_after_submit_transitions_to_accepted() {
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
    click_submit(&mut app);

    // Server unicasts S2CPlacementAccepted; the drain system turns it into
    // HandUiPlacementAcceptedReceived. The test writes the internal message
    // directly to exercise the handler without a live Lightyear server.
    app.world_mut()
        .write_message(HandUiPlacementAcceptedReceived {
            placements_len: 1,
            is_final: true,
        });
    run_update(&mut app);

    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::Accepted {
            placements_len: 1,
            is_final: true,
        },
        "PlacementSubmitAck must become Accepted carrying the server's placements_len + is_final",
    );
    // The optimistic UI MUST remain in effect — the ACK is observability only,
    // not a re-paint trigger (UX polish is out of scope for the ACK contract).
    assert!(
        app.world().resource::<PlacementTimer>().submitted,
        "Accepted ACK must NOT revert placement_timer.submitted; UI stays in Submitted state",
    );
    assert_eq!(
        submit_button_interaction(&mut app),
        HandSubmitInteractionState::Inactive,
        "Accepted ACK must NOT re-enable the submit button",
    );
}

#[test]
fn rejected_after_submit_transitions_ack_to_rejected() {
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
    click_submit(&mut app);

    app.world_mut()
        .write_message(HandUiPlacementRejectedReceived {
            reason: PlacementRejectedReason::SpawnRangeRejected,
        });
    run_update(&mut app);

    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::Rejected {
            reason: PlacementRejectedReason::SpawnRangeRejected,
        },
        "rejection MUST also transition PlacementSubmitAck to Rejected with the same reason",
    );
}

#[test]
fn phase_transition_out_of_placement_resets_ack() {
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
    click_submit(&mut app);
    app.world_mut()
        .write_message(HandUiPlacementAcceptedReceived {
            placements_len: 1,
            is_final: true,
        });
    run_update(&mut app);
    assert!(matches!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::Accepted { .. }
    ));

    // Phase transitions to Resolution then back to Placement; ack must reset.
    set_phase(&mut app, RoundPhase::Resolution);
    run_update(&mut app);
    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::NotSubmitted,
        "phase transition out of Placement MUST reset PlacementSubmitAck",
    );
}

#[test]
fn accepted_ack_after_phase_transition_still_records_state() {
    // Readiness §4 item 2 — phase-transition autosubmit may produce an ACK
    // that arrives after the client has already entered Resolution. The
    // resource must still record the Accepted state; the next phase-into-
    // Placement transition will clear it.
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(10), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));

    set_phase(&mut app, RoundPhase::Resolution);
    run_update(&mut app);
    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::NotSubmitted,
    );

    app.world_mut()
        .write_message(HandUiPlacementAcceptedReceived {
            placements_len: 2,
            is_final: true,
        });
    run_update(&mut app);
    assert_eq!(
        *app.world().resource::<PlacementSubmitAck>(),
        PlacementSubmitAck::Accepted {
            placements_len: 2,
            is_final: true,
        },
        "ACK that arrives outside Placement still updates the resource — the next \
         Placement entry will reset it",
    );
}

// ── helpers ──────────────────────────────────────────────────────────────

fn app_with_hand_ui_in_placement(catalog: HashMap<CardId, CardData>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    // Pre-existing harness gap (PROMPT 1520 inspect system requires
    // `ButtonInput<KeyCode>`, which `MinimalPlugins` does not provide).
    app.init_resource::<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>();
    app.init_resource::<bevy::input::ButtonInput<bevy::input::mouse::MouseButton>>();
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

fn submit_button_interaction(app: &mut App) -> HandSubmitInteractionState {
    let entity = submit_button(app);
    app.world()
        .get::<HandSubmitInteractionState>(entity)
        .copied()
        .expect("submit button should have a HandSubmitInteractionState component")
}

#[allow(dead_code)]
fn submitted_checkmark_visibility(app: &mut App) -> Visibility {
    let entity = app.world().resource::<HandUiEntities>().submitted_checkmark;
    app.world()
        .get::<Visibility>(entity)
        .copied()
        .expect("submitted checkmark should have a Visibility component")
}
