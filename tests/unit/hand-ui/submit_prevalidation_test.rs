use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, HandContents, HandSubmitButton, HandSubmitButtonClicked,
    HandSubmitInteractionState, HandUiOutboundMessages, HandUiPlacementDropResolved, HandUiPlugin,
    PendingPlacements, PlacementTimer, SubmitValidationError,
};
use shared::card::CardId;
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn test_reserve_overdraw_blocks_manual_submit_and_marks_button() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 10, 2);
    set_pending_placements(&mut app, vec![placement(CardId(10), 0, 3)]);

    click_submit(&mut app);

    assert_eq!(submission_count(&app), 0);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Active)
    );
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ReserveOverdrawn)
    );
}

#[test]
fn test_current_mana_overdraw_blocks_manual_submit_and_marks_button() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 3, 10);
    set_pending_placements(&mut app, vec![placement(CardId(20), 4, 0)]);

    click_submit(&mut app);

    assert_eq!(submission_count(&app), 0);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Active)
    );
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ManaOverdrawn)
    );
}

#[test]
fn test_submit_prevalidation_reports_reserve_first_when_both_pools_overdraw() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 0, 0);
    set_pending_placements(&mut app, vec![placement(CardId(30), 1, 1)]);

    click_submit(&mut app);

    assert_eq!(submission_count(&app), 0);
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ReserveOverdrawn)
    );
}

#[test]
fn test_exact_current_and_reserve_spends_pass_validation() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 3, 2);
    set_pending_placements(&mut app, vec![placement(CardId(40), 3, 2)]);

    click_submit(&mut app);

    let submissions = submissions(&app);
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].placements, vec![placement(CardId(40), 3, 2)]);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Inactive)
    );
    assert_eq!(submit_error(&mut app), None);
}

#[test]
fn test_correction_clears_submit_error_and_sends_submission() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 3, 2);
    set_pending_placements(&mut app, vec![placement(CardId(50), 1, 3)]);
    click_submit(&mut app);
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ReserveOverdrawn)
    );

    set_pending_placements(&mut app, vec![placement(CardId(50), 2, 2)]);
    click_submit(&mut app);

    let submissions = submissions(&app);
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].placements, vec![placement(CardId(50), 2, 2)]);
    assert_eq!(submit_error(&mut app), None);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Inactive)
    );
}

#[test]
fn test_timer_expiry_uses_submit_prevalidation_gate() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 0, 1);
    set_pending_placements(&mut app, vec![placement(CardId(60), 1, 0)]);
    app.world_mut()
        .resource_mut::<PlacementTimer>()
        .remaining_ms = 0;

    run_update(&mut app);

    assert_eq!(submission_count(&app), 0);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Active)
    );
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ManaOverdrawn)
    );
}

#[test]
fn test_grace_expiry_uses_submit_prevalidation_gate() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 5, 0);
    set_pending_placements(&mut app, vec![placement(CardId(70), 0, 1)]);
    {
        let mut timer = app.world_mut().resource_mut::<PlacementTimer>();
        timer.remaining_ms = 0;
        timer.in_grace_window = true;
        timer.grace_remaining_ms = 1;
    }

    run_for(&mut app, Duration::from_millis(1));

    assert_eq!(submission_count(&app), 0);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Active)
    );
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ReserveOverdrawn)
    );
}

#[test]
fn test_grace_window_drop_uses_submit_prevalidation_gate() {
    let mut app = app_with_hand_ui_in_placement();
    set_economy(&mut app, 0, 0);
    set_hand(&mut app, [CardId(80)]);
    {
        let mut timer = app.world_mut().resource_mut::<PlacementTimer>();
        timer.remaining_ms = 0;
        timer.in_grace_window = true;
        timer.grace_remaining_ms = 200;
    }

    let slot = fan_slot(&mut app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(7),
        target: Some(PlayTarget::BoardCell { lane: 1, cell: 1 }),
    });
    run_update(&mut app);

    assert_eq!(submission_count(&app), 0);
    assert_eq!(
        submit_state(&mut app),
        Some(HandSubmitInteractionState::Active)
    );
    assert_eq!(
        submit_error(&mut app),
        Some(SubmitValidationError::ManaOverdrawn)
    );
}

fn app_with_hand_ui_in_placement() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
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

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
    run_update(app);
}

fn set_economy(app: &mut App, current_mana: u32, reserve_mana: u32) {
    let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
    economy.current_mana = current_mana;
    economy.reserve_mana = reserve_mana;
}

fn set_pending_placements(app: &mut App, placements: Vec<PlacedCardSubmit>) {
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements = placements;
}

fn click_submit(app: &mut App) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
    run_update(app);
}

fn run_update(app: &mut App) {
    run_for(app, Duration::ZERO);
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
}

fn placement(
    card_id: CardId,
    current_mana_spend: u32,
    reserve_mana_spend: u32,
) -> PlacedCardSubmit {
    PlacedCardSubmit {
        card_id,
        target: PlayTarget::BoardCell { lane: 1, cell: 1 },
        current_mana_spend,
        reserve_mana_spend,
    }
}

fn submissions(app: &App) -> Vec<shared::protocol::C2SSubmitPlacement> {
    app.world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements
        .clone()
}

fn submission_count(app: &App) -> usize {
    submissions(app).len()
}

fn submit_button(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit button should exist")
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn submit_state(app: &mut App) -> Option<HandSubmitInteractionState> {
    let button = submit_button(app);
    app.world()
        .get::<HandSubmitInteractionState>(button)
        .copied()
}

fn submit_error(app: &mut App) -> Option<SubmitValidationError> {
    let button = submit_button(app);
    app.world().get::<SubmitValidationError>(button).copied()
}
