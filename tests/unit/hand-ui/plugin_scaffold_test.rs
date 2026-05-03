use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::state::ClientState;
use client::ui::hand::{
    FanSlotIndex, GridSlotIndex, HandUiEntities, HandUiEntity, HandUiPlugin,
    ReserveStripForFanSlot, DRAFT_INITIAL_GRID_SLOT_COUNT, HAND_FAN_SLOT_COUNT,
    HAND_UI_ENTITY_COUNT,
};

#[test]
fn hand_ui_initializes_prepooled_hidden_entities_on_session_entry() {
    let mut app = app_with_hand_ui_in_session();

    assert_eq!(count_with::<HandUiEntity>(&mut app), HAND_UI_ENTITY_COUNT);
    assert_eq!(count_with::<FanSlotIndex>(&mut app), HAND_FAN_SLOT_COUNT);
    assert_eq!(
        count_with::<GridSlotIndex>(&mut app),
        DRAFT_INITIAL_GRID_SLOT_COUNT
    );
    assert_eq!(count_with::<HandDragSprite>(&mut app), 1);
    assert_all_hidden::<FanSlotIndex>(&mut app);
    assert_all_hidden::<GridSlotIndex>(&mut app);
    assert_all_hidden::<ReserveStripForFanSlot>(&mut app);
    assert_all_hidden::<HandDragSprite>(&mut app);
    assert_indices(
        fan_indices(&mut app),
        0..HAND_FAN_SLOT_COUNT as u8,
        "fan slot",
    );
    assert_indices(
        grid_indices(&mut app),
        0..DRAFT_INITIAL_GRID_SLOT_COUNT as u8,
        "grid slot",
    );
}

#[test]
fn hand_ui_does_not_respawn_pooled_entities_during_session_updates() {
    let mut app = app_with_hand_ui_in_session();
    let initial = pooled_entities(&app);

    set_all_pooled_visibility(&mut app, Visibility::Visible);
    for _ in 0..3 {
        app.update();
    }
    set_all_pooled_visibility(&mut app, Visibility::Hidden);
    for _ in 0..3 {
        app.update();
    }

    assert_eq!(pooled_entities(&app), initial);
    assert_eq!(count_with::<FanSlotIndex>(&mut app), HAND_FAN_SLOT_COUNT);
    assert_eq!(
        count_with::<GridSlotIndex>(&mut app),
        DRAFT_INITIAL_GRID_SLOT_COUNT
    );
    assert_eq!(count_with::<HandDragSprite>(&mut app), 1);
}

#[test]
fn hand_ui_despawns_on_session_exit_and_rebuilds_on_reentry() {
    let mut app = app_with_hand_ui_in_session();
    let first_session_entities = pooled_entities(&app);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::Lobby);
    app.update();

    assert!(app.world().get_resource::<HandUiEntities>().is_none());
    assert_eq!(count_with::<HandUiEntity>(&mut app), 0);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    let second_session_entities = pooled_entities(&app);
    assert_eq!(count_with::<HandUiEntity>(&mut app), HAND_UI_ENTITY_COUNT);
    assert!(first_session_entities.is_disjoint(&second_session_entities));
}

fn app_with_hand_ui_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn assert_all_hidden<T: Component>(app: &mut App) {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<T>>();
    assert!(query
        .iter(app.world())
        .all(|visibility| matches!(visibility, Visibility::Hidden)));
}

fn assert_indices(
    actual: impl IntoIterator<Item = u8>,
    expected: impl IntoIterator<Item = u8>,
    label: &str,
) {
    let actual = actual.into_iter().collect::<BTreeSet<_>>();
    let expected = expected.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{label} indices should be contiguous");
}

fn fan_indices(app: &mut App) -> Vec<u8> {
    let mut query = app.world_mut().query::<&FanSlotIndex>();
    query.iter(app.world()).map(|index| index.0).collect()
}

fn grid_indices(app: &mut App) -> Vec<u8> {
    let mut query = app.world_mut().query::<&GridSlotIndex>();
    query.iter(app.world()).map(|index| index.0).collect()
}

fn pooled_entities(app: &App) -> BTreeSet<Entity> {
    let entities = app.world().resource::<HandUiEntities>();
    entities
        .fan_slots
        .iter()
        .chain(entities.grid_slots.iter())
        .chain(std::iter::once(&entities.drag_sprite))
        .copied()
        .collect()
}

fn set_all_pooled_visibility(app: &mut App, visibility: Visibility) {
    let entities = pooled_entities(app);
    for entity in entities {
        *app.world_mut()
            .get_mut::<Visibility>(entity)
            .expect("pooled hand UI entity should have visibility") = visibility;
    }
}
