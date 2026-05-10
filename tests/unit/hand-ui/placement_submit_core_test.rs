use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    ConfirmationModal, FanSlotIndex, FanSlotState, GhostPlacementChanged, HandContents,
    HandSubmitButton, HandSubmitButtonClicked, HandSubmitInteractionState, HandUiOutboundMessages,
    HandUiPlacementDropResolved, HandUiPlugin, PendingPlacements, ReserveStripForFanSlot,
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
