//! PROMPT 1244 — S18-PLACEMENT-SUBMISSION-REJECTION-FEEDBACK-001.
//!
//! Hand-UI consumer side of the placement rejection feedback gap. When the
//! server unicasts `S2CPlacementRejected`, the client-side
//! `handle_placement_rejected_system` MUST revert the optimistic Submitted
//! view installed by `submit_pending_placements` so the player can correct
//! their batch and retry:
//!
//! - `PlacementTimer::submitted` returns to `false`
//! - Submit button text returns to `Submit (N cards)`
//! - Submit button interaction returns to `Active`
//! - Submitted checkmark hides
//! - Disclosure step becomes `Correction { error: ServerRejected { reason } }`
//!
//! Authority remains server-side: the handler never accepts placements or
//! mutates `PendingPlacements`; the rejection only re-enables the local
//! affordance.

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
        HandSubmitInteractionState, HandUiEntities, HandUiOutboundMessages,
        HandUiPlacementDropResolved, HandUiPlacementRejectedReceived, HandUiPlugin,
        PendingPlacements, PlacementDisclosureGuidance, PlacementDisclosureState,
        PlacementDisclosureStep, PlacementTimer, SubmitValidationError,
    },
    shared::BoardLayout,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlacementRejectedReason, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn rejection_after_submit_reverts_submitted_state_and_shows_correction() {
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

    // Sanity — optimistic Submitted view is installed before the rejection.
    assert!(
        app.world().resource::<PlacementTimer>().submitted,
        "placement timer must report submitted=true after manual submit click",
    );
    assert_eq!(
        submit_button_interaction(&mut app),
        HandSubmitInteractionState::Inactive,
        "submit button interaction must be Inactive after manual submit",
    );
    assert_eq!(
        submit_button_text(&mut app),
        "Submitted",
        "submit button text must read 'Submitted' after manual submit",
    );
    assert_eq!(
        submitted_checkmark_visibility(&mut app),
        Visibility::Visible,
        "submitted checkmark must be Visible after manual submit",
    );
    assert!(
        matches!(
            app.world().resource::<PlacementDisclosureState>().step,
            PlacementDisclosureStep::Submitted
        ),
        "disclosure step must be Submitted after manual submit; got {:?}",
        app.world().resource::<PlacementDisclosureState>().step,
    );

    // Server unicasts S2CPlacementRejected; the drain system normally turns
    // this into HandUiPlacementRejectedReceived. The test writes the internal
    // message directly to exercise the handler without a live Lightyear server.
    app.world_mut()
        .write_message(HandUiPlacementRejectedReceived {
            reason: PlacementRejectedReason::SpawnRangeRejected,
        });
    run_update(&mut app);

    assert!(
        !app.world().resource::<PlacementTimer>().submitted,
        "placement timer must report submitted=false after rejection",
    );
    assert_eq!(
        submit_button_interaction(&mut app),
        HandSubmitInteractionState::Inactive,
        "submit button interaction must stay Inactive until the rejected batch is edited",
    );
    assert_eq!(
        submit_button_text(&mut app),
        "Submit (1 cards)",
        "submit button text must reflect the staged_count after rejection",
    );
    assert_eq!(
        submitted_checkmark_visibility(&mut app),
        Visibility::Hidden,
        "submitted checkmark must be Hidden after rejection",
    );
    assert_eq!(
        app.world().resource::<PlacementDisclosureState>().step,
        PlacementDisclosureStep::Correction {
            error: SubmitValidationError::ServerRejected {
                reason: PlacementRejectedReason::SpawnRangeRejected,
            },
        },
        "disclosure step must be Correction with ServerRejected(SpawnRangeRejected) after rejection",
    );
    // Authority preservation: PendingPlacements is the player's staged batch
    // and must NOT be cleared by the rejection — they need to be able to
    // edit and resubmit.
    assert_eq!(
        app.world().resource::<PendingPlacements>().staged_count(),
        1,
        "PendingPlacements must NOT be cleared on rejection; the player keeps \
         their staged batch and resubmits",
    );
}

#[test]
fn rejection_preserves_pending_placements_for_edit_recovery() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(11), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(11)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    click_submit(&mut app);

    app.world_mut()
        .write_message(HandUiPlacementRejectedReceived {
            reason: PlacementRejectedReason::OccupancyRejected,
        });
    run_update(&mut app);

    // Server authority preserved — client may NOT accept locally. Pending
    // is intact so the player can adjust and click submit again.
    assert_eq!(
        app.world().resource::<PendingPlacements>().staged_count(),
        1,
    );
    assert_eq!(
        app.world().resource::<PlacementDisclosureState>().step,
        PlacementDisclosureStep::Correction {
            error: SubmitValidationError::ServerRejected {
                reason: PlacementRejectedReason::OccupancyRejected,
            },
        },
    );
}

#[test]
fn rejected_unchanged_pending_batch_cannot_resubmit_in_silent_loop() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(12), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(12)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    click_submit(&mut app);
    assert_eq!(submissions(&app).len(), 1);

    app.world_mut()
        .write_message(HandUiPlacementRejectedReceived {
            reason: PlacementRejectedReason::OccupancyRejected,
        });
    run_update(&mut app);
    click_submit(&mut app);

    assert_eq!(
        submissions(&app).len(),
        1,
        "unchanged rejected batch must not send another C2SSubmitPlacement",
    );
    assert_eq!(
        submit_button_interaction(&mut app),
        HandSubmitInteractionState::Inactive,
    );
    assert_eq!(
        guidance_text(&mut app),
        "Server rejected placement: slot taken, retarget or unstage",
    );
}

#[test]
fn retargeting_after_rejection_reenables_submit_and_sends_edited_batch() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(13), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(13)]);
    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    click_submit(&mut app);
    app.world_mut()
        .write_message(HandUiPlacementRejectedReceived {
            reason: PlacementRejectedReason::InvalidTarget,
        });
    run_update(&mut app);

    stage_card_in_slot(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 2 },
    );
    assert_eq!(
        submit_button_interaction(&mut app),
        HandSubmitInteractionState::Active,
    );
    click_submit(&mut app);

    let submissions = submissions(&app);
    assert_eq!(submissions.len(), 2);
    assert_eq!(
        submissions[1].placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 2 },
    );
}

#[test]
fn unstaging_after_rejection_clears_stale_batch_and_returns_to_card_selection() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_in_placement(test_catalog([(CardId(14), CardType::Minion)]));
    set_local_player(&mut app, PlayerId(7));
    set_hand(&mut app, [CardId(14)]);
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

    app.world_mut()
        .write_message(client::ui::hand::GhostClickedEvent {
            card_id: CardId(14),
        });
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<PendingPlacements>().staged_count(),
        0,
    );
    assert_eq!(submit_button_text(&mut app), "Submit (0 cards)");
    assert_eq!(
        app.world().resource::<PlacementDisclosureState>().step,
        PlacementDisclosureStep::CardSelection,
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

fn submit_button_text(app: &mut App) -> String {
    let entity = submit_button(app);
    app.world()
        .get::<Text>(entity)
        .map(|text| text.0.clone())
        .expect("submit button should have a Text component")
}

fn submit_button_interaction(app: &mut App) -> HandSubmitInteractionState {
    let entity = submit_button(app);
    app.world()
        .get::<HandSubmitInteractionState>(entity)
        .copied()
        .expect("submit button should have a HandSubmitInteractionState component")
}

fn guidance_text(app: &mut App) -> String {
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<PlacementDisclosureGuidance>>();
    query
        .single(app.world())
        .expect("placement disclosure guidance should exist")
        .0
        .clone()
}

fn submitted_checkmark_visibility(app: &mut App) -> Visibility {
    let entity = app.world().resource::<HandUiEntities>().submitted_checkmark;
    app.world()
        .get::<Visibility>(entity)
        .copied()
        .expect("submitted checkmark should have a Visibility component")
}

#[allow(dead_code)]
fn submissions(app: &App) -> Vec<shared::protocol::C2SSubmitPlacement> {
    app.world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .clone()
}
