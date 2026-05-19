use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    ConfirmationModal, FanSlotIndex, FanSlotState, GhostPlacementChanged, HandContents,
    HandSubmitButton, HandSubmitButtonClicked, HandSubmitInteractionState, HandUiOutboundMessages,
    HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements, PlacementTimer,
    ReserveStripForFanSlot,
};
use shared::card::CardId;
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn hu_11_placement_entry_shows_active_submit_and_resets_pending() {
    let mut app = app_with_hand_ui_in_session();
    let stale = placement(CardId(900), PlayTarget::BoardCell { lane: 5, cell: 8 });
    app.world_mut()
        .resource_mut::<PendingPlacements>()
        .placements
        .push(stale);
    set_hand(&mut app, [CardId(101)]);
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let submit = submit_button(&mut app);
    assert_eq!(
        app.world().get::<Visibility>(submit),
        Some(&Visibility::Visible)
    );
    assert_eq!(text(&app, submit), "Submit (0 cards)");
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Active)
    );
    assert!(
        app.world()
            .resource::<PendingPlacements>()
            .placements
            .is_empty(),
        "PLACEMENT entry should reset the local staging queue"
    );
}

#[test]
fn hu_13_valid_drop_stages_ghost_updates_submit_and_reserve_strip() {
    let mut app = app_with_hand_ui_in_session();
    set_hand(&mut app, [CardId(201), CardId(202)]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let slot = fan_slot(&mut app, 0);
    let reserve_strip = app
        .world_mut()
        .spawn((ReserveStripForFanSlot(0), Visibility::Hidden))
        .id();
    let target = PlayTarget::BoardCell { lane: 1, cell: 2 };

    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(7),
        target: Some(target.clone()),
    });
    app.update();

    assert_eq!(
        ghost_messages(&app),
        vec![GhostPlacementChanged {
            target: Some(target.clone()),
            card_id: Some(CardId(201)),
        }]
    );
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Ghost)
    );
    let submit = submit_button(&mut app);
    assert_eq!(text(&app, submit), "Submit (1 cards)");
    assert_eq!(
        app.world().get::<Visibility>(reserve_strip),
        Some(&Visibility::Visible)
    );

    let pending = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].card_id, CardId(201));
    assert_eq!(pending[0].target, target);
    assert_eq!(pending[0].current_mana_spend, 1);
    assert_eq!(pending[0].reserve_mana_spend, 0);
}

#[test]
fn hu_14_invalid_drop_hides_drag_returns_active_and_writes_no_ghost() {
    let mut app = app_with_hand_ui_in_session();
    set_hand(&mut app, [CardId(301)]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let slot = fan_slot(&mut app, 0);
    app.world_mut().entity_mut(slot).insert(FanSlotState::Ghost);
    set_drag_visibility(&mut app, Visibility::Visible);

    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(7),
        target: None,
    });
    app.update();

    assert_drag_visibility(&mut app, Visibility::Hidden);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Active)
    );
    assert!(
        ghost_messages(&app).is_empty(),
        "invalid drops must not publish GhostPlacementChanged"
    );
}

#[test]
fn hu_16_zero_card_submit_sends_empty_submission_and_locks_button() {
    let mut app = app_with_hand_ui_in_session();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    click_submit(&mut app);

    let outbound = &app
        .world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements;
    assert_eq!(outbound.len(), 1);
    assert!(outbound[0].placements.is_empty());

    let submit = submit_button(&mut app);
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Inactive)
    );
    assert_eq!(text(&app, submit), "Submitted");
    assert_eq!(count_with::<ConfirmationModal>(&mut app), 0);
}

#[test]
fn hu_17_rapid_duplicate_submit_clicks_emit_only_one_submission() {
    let mut app = app_with_hand_ui_in_session();
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let submit = submit_button(&mut app);
    for _ in 0..10 {
        app.world_mut()
            .write_message(HandSubmitButtonClicked { button: submit });
    }
    app.update();

    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .submit_placements
            .len(),
        1,
        "same-frame duplicate clicks should be absorbed after the first send"
    );

    click_submit(&mut app);
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .submit_placements
            .len(),
        1,
        "inactive submit button must not enqueue another placement submission"
    );
}

// PROMPT 1399 — S18-PLACEMENT-SUBMIT-SILENT-NOOP-REPAIR-001
//
// Regression: the prior submit pipeline gated the network send on
// `HandSubmitInteractionState == Active`. When the presentational flag and
// `PlacementTimer::submitted` momentarily disagreed (button spawned with
// the default `Inactive` flag before the phase-transition system ran, any
// future state-sync ordering bug, etc.), legitimate submit clicks were
// silently swallowed — the AUDIT-1392-P01 "Confirm does nothing"
// failure mode in the 2026-05-18 dev-run.
//
// The repair drops the redundant `interaction_state` gate; the
// authoritative duplicate-submit guard is now solely
// `placement_timer.submitted` (set by every successful send before any
// peer call observes it). This test pins the new contract: as long as
// the timer says we have not submitted yet, clicking Submit MUST queue
// exactly one `C2SSubmitPlacement` regardless of the presentational
// `HandSubmitInteractionState`.
#[test]
fn hu_25_submit_proceeds_when_interaction_state_inactive_but_not_yet_submitted() {
    let mut app = app_with_hand_ui_in_session();
    set_economy(&mut app, 10, 10);
    set_hand(&mut app, [CardId(401)]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let submit = submit_button(&mut app);
    // Sanity: entering Placement promotes the submit button to Active.
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Active),
        "entering Placement must promote the submit button to Active"
    );

    // Stage one card so the click has a non-empty payload to send.
    let slot = fan_slot(&mut app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(7),
        target: Some(PlayTarget::BoardCell { lane: 1, cell: 2 }),
    });
    app.update();
    assert_eq!(
        app.world()
            .resource::<PendingPlacements>()
            .placements
            .len(),
        1,
        "fixture must stage exactly one placement"
    );

    // Force the regression precondition: presentational state out of sync
    // with the timer. This simulates the AUDIT-1392-P01 silent-no-op
    // scenario; the timer says we have not submitted yet, but the button
    // somehow reads `Inactive`.
    *app.world_mut()
        .get_mut::<HandSubmitInteractionState>(submit)
        .expect("submit button must have HandSubmitInteractionState") =
        HandSubmitInteractionState::Inactive;
    assert!(
        !app.world().resource::<PlacementTimer>().submitted,
        "PlacementTimer::submitted must remain false before the click"
    );

    click_submit(&mut app);

    let outbound = &app
        .world()
        .resource::<HandUiOutboundMessages>()
        .submit_placements;
    assert_eq!(
        outbound.len(),
        1,
        "click with timer.submitted=false MUST queue one C2SSubmitPlacement \
         even when HandSubmitInteractionState is Inactive (PROMPT 1399 \
         silent-no-op repair); got {} submissions",
        outbound.len()
    );
    assert_eq!(
        outbound[0].placements.len(),
        1,
        "queued submission must carry the one staged placement"
    );
    assert!(
        app.world().resource::<PlacementTimer>().submitted,
        "successful send must flip placement_timer.submitted=true"
    );
    assert_eq!(
        app.world().get::<HandSubmitInteractionState>(submit),
        Some(&HandSubmitInteractionState::Inactive),
        "submit button stays Inactive after send (now its terminal state)"
    );
    assert_eq!(text(&app, submit), "Submitted");
}

// PROMPT 1399 — companion duplicate-prevention coverage. The repair drops
// the interaction-state gate but keeps `placement_timer.submitted` as the
// duplicate guard. A second click after a successful send must NOT
// produce a second wire message regardless of how the presentational
// flags look in the intervening time.
#[test]
fn hu_26_second_click_after_send_is_suppressed_by_placement_timer_submitted() {
    let mut app = app_with_hand_ui_in_session();
    set_economy(&mut app, 10, 10);
    set_hand(&mut app, [CardId(501)]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let slot = fan_slot(&mut app, 0);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id: PlayerId(7),
        target: Some(PlayTarget::BoardCell { lane: 1, cell: 4 }),
    });
    app.update();

    // First click — should send exactly one C2SSubmitPlacement.
    click_submit(&mut app);
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .submit_placements
            .len(),
        1,
        "first click must queue one submission"
    );
    assert!(
        app.world().resource::<PlacementTimer>().submitted,
        "first send must set placement_timer.submitted=true"
    );

    // Second click — placement_timer.submitted=true now, so the call must
    // short-circuit at the timer gate and NOT enqueue a duplicate.
    click_submit(&mut app);
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .submit_placements
            .len(),
        1,
        "second click must be suppressed by placement_timer.submitted=true"
    );

    // Even an artificially re-promoted Active state must not bypass the
    // timer gate: this is the scenario the previous interaction-state
    // gate was nominally protecting against, and the new timer-only gate
    // continues to protect against it.
    let submit = submit_button(&mut app);
    *app.world_mut()
        .get_mut::<HandSubmitInteractionState>(submit)
        .expect("submit button must have HandSubmitInteractionState") =
        HandSubmitInteractionState::Active;
    click_submit(&mut app);
    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .submit_placements
            .len(),
        1,
        "click with Active state but placement_timer.submitted=true must \
         still be suppressed (timer is the authoritative duplicate guard)"
    );
}

fn app_with_hand_ui_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
}

fn set_economy(app: &mut App, current_mana: u32, reserve_mana: u32) {
    let mut economy = app.world_mut().resource_mut::<PlayerEconomyView>();
    economy.current_mana = current_mana;
    economy.reserve_mana = reserve_mana;
}

fn click_submit(app: &mut App) {
    let button = submit_button(app);
    app.world_mut()
        .write_message(HandSubmitButtonClicked { button });
    app.update();
}

fn placement(card_id: CardId, target: PlayTarget) -> PlacedCardSubmit {
    PlacedCardSubmit {
        card_id,
        target,
        current_mana_spend: 0,
        reserve_mana_spend: 0,
    }
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

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("entity should have Text")
        .0
        .clone()
}

fn ghost_messages(app: &App) -> Vec<GhostPlacementChanged> {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn set_drag_visibility(app: &mut App, visibility: Visibility) {
    let drag = drag_sprite(app);
    *app.world_mut()
        .get_mut::<Visibility>(drag)
        .expect("drag sprite should have Visibility") = visibility;
}

fn assert_drag_visibility(app: &mut App, expected: Visibility) {
    let drag = drag_sprite(app);
    assert_eq!(app.world().get::<Visibility>(drag), Some(&expected));
}

fn drag_sprite(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<HandDragSprite>>();
    query.single(app.world()).expect("drag sprite should exist")
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}
