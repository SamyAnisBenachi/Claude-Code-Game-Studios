use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, FanSlotState, FanZoneBounds, GhostClickedEvent, GhostDragStartEvent,
    GhostPlacementChanged, HandContents, HandFanCardClicked, HandSubmitButton,
    HandUiPlacementCursorMoved, HandUiPlacementDragEnded, HandUiPlacementDropResolved,
    HandUiPlugin, PendingPlacements, ReserveStripForFanSlot,
};
use shared::card::CardId;
use shared::protocol::{PlacedCardSubmit, PlayTarget, RoundPhase};
use shared::session::PlayerId;

#[test]
fn hu_21_board_ghost_click_unstages_atomically() {
    let mut app = app_with_hand_ui_in_placement([CardId(10)]);
    let reserve_strip = reserve_strip(&mut app, 0, Visibility::Hidden);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 2, cell: 4 },
    );
    assert_eq!(
        app.world().get::<Visibility>(reserve_strip),
        Some(&Visibility::Visible)
    );
    let mut cursor = drained_ghost_cursor(&app);

    app.world_mut().write_message(GhostClickedEvent {
        card_id: CardId(10),
    });
    app.update();

    assert_unstaged(&mut app, 0, CardId(10), reserve_strip);
    assert_eq!(
        ghost_messages_since(&app, &mut cursor),
        vec![GhostPlacementChanged {
            target: None,
            card_id: Some(CardId(10)),
        }]
    );
}

#[test]
fn hu_21_unknown_ghost_click_is_ignored() {
    let mut app = app_with_hand_ui_in_placement([CardId(20)]);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    let mut cursor = drained_ghost_cursor(&app);

    app.world_mut().write_message(GhostClickedEvent {
        card_id: CardId(999),
    });
    app.update();

    assert_eq!(
        app.world().resource::<PendingPlacements>().placements,
        vec![PlacedCardSubmit {
            card_id: CardId(20),
            target: PlayTarget::BoardCell { lane: 1, cell: 1 },
            current_mana_spend: 1,
            reserve_mana_spend: 0,
        }]
    );
    let slot = fan_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Ghost)
    );
    assert_eq!(submit_text(&mut app), "Submit (1 cards)");
    assert!(ghost_messages_since(&app, &mut cursor).is_empty());
}

#[test]
fn hu_21b_board_ghost_drag_unstages_only_on_fan_zone_release() {
    let mut app = app_with_hand_ui_in_placement([CardId(30)]);
    app.insert_resource(FanZoneBounds {
        x_min: 100.0,
        x_max: 900.0,
        y_min: 600.0,
        y_max: 680.0,
    });
    let reserve_strip = reserve_strip(&mut app, 0, Visibility::Hidden);
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 2, cell: 4 },
    );
    let mut cursor = drained_ghost_cursor(&app);

    drag_ghost_and_release(&mut app, CardId(30), Vec2::new(450.0, 640.0));

    assert_unstaged(&mut app, 0, CardId(30), reserve_strip);
    assert_eq!(
        ghost_messages_since(&app, &mut cursor),
        vec![GhostPlacementChanged {
            target: None,
            card_id: Some(CardId(30)),
        }]
    );

    let mut app = app_with_hand_ui_in_placement([CardId(31)]);
    app.insert_resource(FanZoneBounds {
        x_min: 100.0,
        x_max: 900.0,
        y_min: 600.0,
        y_max: 680.0,
    });
    stage_card(
        &mut app,
        0,
        PlayerId(7),
        PlayTarget::BoardCell { lane: 2, cell: 4 },
    );
    let mut cursor = drained_ghost_cursor(&app);

    drag_ghost_and_release(&mut app, CardId(31), Vec2::new(450.0, 200.0));

    assert_eq!(
        app.world().resource::<PendingPlacements>().placements,
        vec![PlacedCardSubmit {
            card_id: CardId(31),
            target: PlayTarget::BoardCell { lane: 2, cell: 4 },
            current_mana_spend: 1,
            reserve_mana_spend: 0,
        }]
    );
    let slot = fan_slot(&mut app, 0);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Ghost)
    );
    assert_eq!(submit_text(&mut app), "Submit (1 cards)");
    assert!(ghost_messages_since(&app, &mut cursor).is_empty());
}

#[test]
fn hu_21c_instant_fan_slot_click_unstages() {
    let mut app = app_with_hand_ui_in_placement([CardId(40)]);
    let reserve_strip = reserve_strip(&mut app, 0, Visibility::Hidden);
    stage_card(&mut app, 0, PlayerId(7), PlayTarget::Instant);
    let mut cursor = drained_ghost_cursor(&app);

    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .write_message(HandFanCardClicked { card: slot });
    app.update();

    assert_unstaged(&mut app, 0, CardId(40), reserve_strip);
    assert_eq!(
        ghost_messages_since(&app, &mut cursor),
        vec![GhostPlacementChanged {
            target: None,
            card_id: Some(CardId(40)),
        }]
    );
}

fn app_with_hand_ui_in_placement<const N: usize>(cards: [CardId; N]) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(HandUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_hand(&mut app, cards);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
}

fn stage_card(app: &mut App, slot_index: u8, owner_id: PlayerId, target: PlayTarget) {
    let slot = fan_slot(app, slot_index);
    app.world_mut().write_message(HandUiPlacementDropResolved {
        card: slot,
        owner_id,
        target: Some(target),
    });
    app.update();
}

fn drag_ghost_and_release(app: &mut App, card_id: CardId, release_position: Vec2) {
    app.world_mut()
        .write_message(GhostDragStartEvent { card_id });
    app.world_mut().write_message(HandUiPlacementCursorMoved {
        world_position: Some(release_position),
    });
    app.world_mut().write_message(HandUiPlacementDragEnded);
    app.update();
}

fn assert_unstaged(app: &mut App, slot_index: u8, card_id: CardId, reserve_strip: Entity) {
    assert!(
        app.world()
            .resource::<PendingPlacements>()
            .placements
            .is_empty(),
        "card {card_id:?} should be removed from the pending placement queue"
    );
    let slot = fan_slot(app, slot_index);
    assert_eq!(
        app.world().get::<FanSlotState>(slot),
        Some(&FanSlotState::Active)
    );
    assert_eq!(submit_text(app), "Submit (0 cards)");
    assert_eq!(
        app.world().get::<Visibility>(reserve_strip),
        Some(&Visibility::Hidden)
    );
}

fn reserve_strip(app: &mut App, slot_index: u8, visibility: Visibility) -> Entity {
    app.world_mut()
        .spawn((ReserveStripForFanSlot(slot_index), visibility))
        .id()
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn submit_text(app: &mut App) -> String {
    let mut query = app
        .world_mut()
        .query_filtered::<&Text, With<HandSubmitButton>>();
    query
        .single(app.world())
        .expect("submit text should exist")
        .0
        .clone()
}

fn drained_ghost_cursor(app: &App) -> MessageCursor<GhostPlacementChanged> {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn ghost_messages_since(
    app: &App,
    cursor: &mut MessageCursor<GhostPlacementChanged>,
) -> Vec<GhostPlacementChanged> {
    let messages = app.world().resource::<Messages<GhostPlacementChanged>>();
    cursor.read(messages).cloned().collect()
}
