use std::time::Duration;

use bevy::camera::NormalizedRenderTarget;
use bevy::color::Alpha;
use bevy::ecs::message::MessageCursor;
use bevy::picking::{
    backend::HitData,
    pointer::{Location, PointerId},
};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    presentation::{
        board_rendering::{
            rendering_constants, BoardCamera, BoardGhostInteraction, BoardRenderingPlugin,
            GhostUnit, LaneGhostWash, ObjectiveTargetGhost, TargetUnitGhost,
        },
        BoardLayout,
    },
    state::ClientState,
    ui::hand::{
        GhostClickedEvent, GhostDragStartEvent, GhostPlacementChanged, ObjectiveCell,
        PlacementTargetUnit,
    },
};
use shared::{card::CardId, protocol::PlayTarget, session::PlayerId};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn br_8_board_cell_ghost_replaces_existing_card_preview() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    let layout = *app.world().resource::<BoardLayout>();

    stage_ghost(
        &mut app,
        CardId(10),
        PlayTarget::BoardCell { lane: 2, cell: 4 },
    );

    let first = single_ghost_unit(&mut app, CardId(10));
    assert_eq!(
        app.world().get::<Transform>(first).unwrap().translation,
        layout
            .cell_to_world(2, 4)
            .extend(rendering_constants::Z_GHOST_UNIT)
    );
    assert_eq!(app.world().get::<Sprite>(first).unwrap().color.alpha(), 0.5);
    assert!(app.world().get::<Children>(first).is_none());

    stage_ghost(
        &mut app,
        CardId(10),
        PlayTarget::BoardCell { lane: 4, cell: 7 },
    );

    let second = single_ghost_unit(&mut app, CardId(10));
    assert_eq!(
        app.world().get::<Transform>(second).unwrap().translation,
        layout
            .cell_to_world(4, 7)
            .extend(rendering_constants::Z_GHOST_UNIT)
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(10)), 1);
}

#[test]
fn br_8_variant_matrix_marks_or_spawns_expected_board_ghosts() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    let unit = app
        .world_mut()
        .spawn(PlacementTargetUnit {
            owner_id: PlayerId(2),
            unit_id: 77,
        })
        .id();
    let objective = app
        .world_mut()
        .spawn(ObjectiveCell {
            player_id: PlayerId(2),
            lane: 3,
        })
        .id();

    stage_ghost(
        &mut app,
        CardId(20),
        PlayTarget::TargetUnit {
            lane: 1,
            unit_id: 77,
        },
    );
    assert_eq!(
        app.world().get::<TargetUnitGhost>(unit),
        Some(&TargetUnitGhost {
            card_id: CardId(20)
        })
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(20)), 0);

    stage_ghost(
        &mut app,
        CardId(21),
        PlayTarget::TargetObj {
            player_id: PlayerId(2),
            lane: 3,
        },
    );
    assert_eq!(
        app.world().get::<ObjectiveTargetGhost>(objective),
        Some(&ObjectiveTargetGhost {
            card_id: CardId(21)
        })
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(21)), 0);

    stage_ghost(&mut app, CardId(22), PlayTarget::LaneWide { lane: 5 });
    assert_eq!(lane_wash_lanes(&mut app, CardId(22)), vec![5]);
    assert_eq!(ghost_unit_count(&mut app, CardId(22)), 0);

    stage_ghost(&mut app, CardId(23), PlayTarget::Instant);
    assert_eq!(ghost_interaction_count(&mut app, CardId(23)), 0);
}

#[test]
fn br_10_clear_none_removes_matching_card_ghosts_without_spawn_range_edits() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    stage_ghost(
        &mut app,
        CardId(30),
        PlayTarget::BoardCell { lane: 1, cell: 1 },
    );
    stage_ghost(&mut app, CardId(31), PlayTarget::LaneWide { lane: 2 });

    clear_ghost(&mut app, CardId(30));
    assert_eq!(ghost_unit_count(&mut app, CardId(30)), 0);
    assert_eq!(lane_wash_lanes(&mut app, CardId(31)), vec![2]);

    clear_ghost(&mut app, CardId(999));
    assert_eq!(lane_wash_lanes(&mut app, CardId(31)), vec![2]);
}

#[test]
fn br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui() {
    test_helpers::init_test_tracing();
    let mut app = app_with_board_rendering();
    stage_ghost(
        &mut app,
        CardId(40),
        PlayTarget::BoardCell { lane: 2, cell: 2 },
    );
    let ghost = single_ghost_unit(&mut app, CardId(40));
    let camera = board_camera(&mut app);
    let mut click_cursor = drained_cursor::<GhostClickedEvent>(&app);
    let mut drag_cursor = drained_cursor::<GhostDragStartEvent>(&app);

    app.world_mut().write_message(pointer_press(ghost, camera));
    app.world_mut().write_message(pointer_click(ghost, camera));
    app.update();

    assert_eq!(
        messages_since(&app, &mut drag_cursor),
        vec![GhostDragStartEvent {
            card_id: CardId(40)
        }]
    );
    assert_eq!(
        messages_since(&app, &mut click_cursor),
        vec![GhostClickedEvent {
            card_id: CardId(40)
        }]
    );
    assert_eq!(ghost_unit_count(&mut app, CardId(40)), 1);
}

fn app_with_board_rendering() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn stage_ghost(app: &mut App, card_id: CardId, target: PlayTarget) {
    app.world_mut().write_message(GhostPlacementChanged {
        target: Some(target),
        card_id: Some(card_id),
    });
    app.update();
}

fn clear_ghost(app: &mut App, card_id: CardId) {
    app.world_mut().write_message(GhostPlacementChanged {
        target: None,
        card_id: Some(card_id),
    });
    app.update();
}

fn single_ghost_unit(app: &mut App, card_id: CardId) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &GhostUnit)>();
    let matches = query
        .iter(app.world())
        .filter_map(|(entity, ghost)| (ghost.card_id == card_id).then_some(entity))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    matches[0]
}

fn ghost_unit_count(app: &mut App, card_id: CardId) -> usize {
    let mut query = app.world_mut().query::<&GhostUnit>();
    query
        .iter(app.world())
        .filter(|ghost| ghost.card_id == card_id)
        .count()
}

fn lane_wash_lanes(app: &mut App, card_id: CardId) -> Vec<u8> {
    let mut query = app.world_mut().query::<&LaneGhostWash>();
    query
        .iter(app.world())
        .filter_map(|wash| (wash.card_id == card_id).then_some(wash.lane))
        .collect()
}

fn ghost_interaction_count(app: &mut App, card_id: CardId) -> usize {
    let mut query = app.world_mut().query::<&BoardGhostInteraction>();
    query
        .iter(app.world())
        .filter(|ghost| ghost.card_id == card_id)
        .count()
}

fn board_camera(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<BoardCamera>>();
    query
        .single(app.world())
        .expect("board camera should exist")
}

fn pointer_location() -> Location {
    Location {
        target: NormalizedRenderTarget::None {
            width: 1,
            height: 1,
        },
        position: Vec2::ZERO,
    }
}

fn hit_data(camera: Entity) -> HitData {
    HitData::new(camera, 0.0, None, None)
}

fn pointer_press(target: Entity, camera: Entity) -> Pointer<Press> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Press {
            button: PointerButton::Primary,
            hit: hit_data(camera),
        },
        target,
    )
}

fn pointer_click(target: Entity, camera: Entity) -> Pointer<Click> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(),
        Click {
            button: PointerButton::Primary,
            hit: hit_data(camera),
            duration: Duration::ZERO,
        },
        target,
    )
}

fn drained_cursor<M: Message + Clone>(app: &App) -> MessageCursor<M> {
    let messages = app.world().resource::<Messages<M>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn messages_since<M: Message + Clone>(app: &App, cursor: &mut MessageCursor<M>) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    cursor.read(messages).cloned().collect()
}
