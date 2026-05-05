use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;
use shared::card::CardId;
use shared::protocol::{PlayTarget, S2CPlacementReveal};

use super::PresentationSet;
use crate::state::ClientState;
use crate::ui::hand::{
    GhostClickedEvent, GhostDragStartEvent, GhostPlacementChanged, ObjectiveCell,
    PlacementTargetUnit,
};
use crate::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

pub mod rendering_constants;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct CardAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardRenderingEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCamera;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellNode;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostUnit {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetUnitGhost {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveTargetGhost {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneGhostWash {
    pub card_id: CardId,
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardGhostInteraction {
    pub card_id: CardId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardGhostPickable;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpawnHighlightState {
    #[default]
    Inactive,
    ValidSpawn,
}

impl SpawnHighlightState {
    pub fn tint(self) -> Color {
        match self {
            Self::Inactive => Color::srgba(0.12, 0.24, 0.30, 0.55),
            Self::ValidSpawn => Color::srgba(1.0, 0.82, 0.24, 0.88),
        }
    }
}

pub struct BoardRenderingPlugin;

impl Plugin for BoardRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .add_message::<GhostPlacementChanged>()
            .add_message::<GhostClickedEvent>()
            .add_message::<GhostDragStartEvent>()
            .add_message::<Pointer<Click>>()
            .add_message::<Pointer<Press>>()
            .add_systems(
                OnEnter(ClientState::InSession),
                insert_board_rendering_session_resources,
            )
            .add_systems(
                OnExit(ClientState::InSession),
                remove_board_rendering_session_resources,
            )
            .add_systems(
                Update,
                (
                    apply_ghost_placement_changed_system,
                    emit_ghost_drag_start_events_system,
                    emit_ghost_clicked_events_system,
                    clear_ghosts_on_placement_reveal_system,
                )
                    .chain()
                    .in_set(PresentationSet::MessageDrain)
                    .run_if(in_state(ClientState::InSession)),
            );
    }
}

pub fn apply_ghost_placement_changed_system(
    mut commands: Commands,
    board_layout: Res<BoardLayout>,
    mut changes: MessageReader<GhostPlacementChanged>,
    ghost_units: Query<(Entity, &GhostUnit)>,
    lane_washes: Query<(Entity, &LaneGhostWash)>,
    target_markers: Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
    target_units: Query<(Entity, &PlacementTargetUnit, Option<&Pickable>)>,
    objectives: Query<(Entity, &ObjectiveCell, Option<&Pickable>)>,
) {
    let mut latest_changes: Vec<(CardId, Option<PlayTarget>)> = Vec::new();

    for change in changes.read() {
        let Some(card_id) = change.card_id else {
            continue;
        };

        if let Some((_existing_card_id, target)) = latest_changes
            .iter_mut()
            .find(|(existing_card_id, _target)| *existing_card_id == card_id)
        {
            *target = change.target.clone();
        } else {
            latest_changes.push((card_id, change.target.clone()));
        }
    }

    for (card_id, target) in latest_changes {
        clear_card_ghosts(
            &mut commands,
            card_id,
            &ghost_units,
            &lane_washes,
            &target_markers,
            &objective_markers,
        );

        match target {
            Some(PlayTarget::BoardCell { lane, cell }) => {
                spawn_ghost_unit(&mut commands, &board_layout, card_id, lane, cell);
            }
            Some(PlayTarget::TargetUnit { unit_id, .. }) => {
                apply_target_unit_ghost(&mut commands, card_id, unit_id, &target_units);
            }
            Some(PlayTarget::TargetObj { player_id, lane }) => {
                apply_objective_target_ghost(&mut commands, card_id, player_id, lane, &objectives);
            }
            Some(PlayTarget::LaneWide { lane }) => {
                spawn_lane_ghost_wash(&mut commands, &board_layout, card_id, lane);
            }
            Some(PlayTarget::Instant) | None => {}
        }
    }
}

pub fn clear_ghosts_on_placement_reveal_system(
    mut commands: Commands,
    mut receivers: Query<&mut MessageReceiver<S2CPlacementReveal>>,
    ghost_units: Query<(Entity, &GhostUnit)>,
    lane_washes: Query<(Entity, &LaneGhostWash)>,
    target_markers: Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
) {
    let mut saw_reveal = false;
    for mut receiver in &mut receivers {
        for _message in receiver.receive() {
            saw_reveal = true;
        }
    }

    if saw_reveal {
        clear_all_board_ghosts(
            &mut commands,
            &ghost_units,
            &lane_washes,
            &target_markers,
            &objective_markers,
        );
    }
}

pub fn emit_ghost_clicked_events_system(
    mut clicks: MessageReader<Pointer<Click>>,
    ghost_interactions: Query<&BoardGhostInteraction>,
    mut writer: MessageWriter<GhostClickedEvent>,
) {
    for click in clicks.read() {
        if click.event.button != PointerButton::Primary {
            continue;
        }

        let Ok(ghost) = ghost_interactions.get(click.entity) else {
            continue;
        };

        writer.write(GhostClickedEvent {
            card_id: ghost.card_id,
        });
    }
}

pub fn emit_ghost_drag_start_events_system(
    mut presses: MessageReader<Pointer<Press>>,
    ghost_interactions: Query<&BoardGhostInteraction>,
    mut writer: MessageWriter<GhostDragStartEvent>,
) {
    for press in presses.read() {
        if press.event.button != PointerButton::Primary {
            continue;
        }

        let Ok(ghost) = ghost_interactions.get(press.entity) else {
            continue;
        };

        writer.write(GhostDragStartEvent {
            card_id: ghost.card_id,
        });
    }
}

fn insert_board_rendering_session_resources(mut commands: Commands) {
    let board_layout = BoardLayout::default();

    commands.insert_resource(board_layout);
    commands.insert_resource(CardAtlas::default());
    spawn_board_camera(&mut commands, &board_layout);
    spawn_board_grid(&mut commands, &board_layout);
}

fn remove_board_rendering_session_resources(
    mut commands: Commands,
    board_entities: Query<Entity, With<BoardRenderingEntity>>,
) {
    for entity in &board_entities {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<BoardLayout>();
    commands.remove_resource::<CardAtlas>();
}

fn spawn_board_camera(commands: &mut Commands, board_layout: &BoardLayout) {
    let camera_xy = board_center(board_layout);

    commands.spawn((
        BoardRenderingEntity,
        BoardCamera,
        Camera2d,
        Transform::from_xyz(
            camera_xy.x,
            camera_xy.y,
            rendering_constants::Z_BOARD_CAMERA,
        ),
    ));
}

fn spawn_board_grid(commands: &mut Commands, board_layout: &BoardLayout) {
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            spawn_cell_node(commands, board_layout, lane, cell);
        }
    }
}

fn spawn_cell_node(commands: &mut Commands, board_layout: &BoardLayout, lane: u8, cell: u8) {
    let world_xy = board_layout.cell_to_world(lane, cell);
    let highlight_state = SpawnHighlightState::Inactive;

    commands.spawn((
        BoardRenderingEntity,
        BoardCellNode,
        LaneCell { lane, cell },
        highlight_state,
        Sprite::from_color(
            highlight_state.tint(),
            Vec2::splat(rendering_constants::CELL_NODE_SIZE),
        ),
        Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_CELL_NODES),
    ));
}

fn spawn_ghost_unit(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_id: CardId,
    lane: u8,
    cell: u8,
) {
    let world_xy = board_layout.cell_to_world(lane, cell);

    commands.spawn((
        BoardRenderingEntity,
        GhostUnit { card_id },
        BoardGhostInteraction { card_id },
        Pickable::default(),
        Sprite::from_color(
            Color::srgba(1.0, 1.0, 1.0, 0.5),
            Vec2::splat(rendering_constants::CELL_NODE_SIZE),
        ),
        Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_GHOST_UNIT),
    ));
}

fn spawn_lane_ghost_wash(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_id: CardId,
    lane: u8,
) {
    let start = board_layout.cell_to_world(lane, 1);
    let end = board_layout.cell_to_world(lane, BOARD_CELL_COUNT);
    let center = (start + end) * 0.5;
    let size = Vec2::new(
        board_layout.cell_width * f32::from(BOARD_CELL_COUNT),
        board_layout.lane_height * 0.72,
    );

    commands.spawn((
        BoardRenderingEntity,
        LaneGhostWash { card_id, lane },
        BoardGhostInteraction { card_id },
        Pickable::default(),
        Sprite::from_color(Color::srgba(0.36, 0.74, 1.0, 0.28), size),
        Transform::from_xyz(center.x, center.y, rendering_constants::Z_FIELD_WASH),
    ));
}

fn apply_target_unit_ghost(
    commands: &mut Commands,
    card_id: CardId,
    unit_id: shared::protocol::EntityId,
    target_units: &Query<(Entity, &PlacementTargetUnit, Option<&Pickable>)>,
) {
    let Some((entity, _target_unit, pickable)) = target_units
        .iter()
        .find(|(_entity, target_unit, _pickable)| target_unit.unit_id == unit_id)
    else {
        return;
    };

    insert_target_marker(
        commands,
        entity,
        pickable.is_some(),
        TargetUnitGhost { card_id },
    );
}

fn apply_objective_target_ghost(
    commands: &mut Commands,
    card_id: CardId,
    player_id: shared::session::PlayerId,
    lane: u8,
    objectives: &Query<(Entity, &ObjectiveCell, Option<&Pickable>)>,
) {
    let Some((entity, _objective, pickable)) =
        objectives.iter().find(|(_entity, objective, _pickable)| {
            objective.player_id == player_id && objective.lane == lane
        })
    else {
        return;
    };

    insert_objective_marker(
        commands,
        entity,
        pickable.is_some(),
        ObjectiveTargetGhost { card_id },
    );
}

fn insert_target_marker(
    commands: &mut Commands,
    entity: Entity,
    has_pickable: bool,
    marker: TargetUnitGhost,
) {
    let mut entity_commands = commands.entity(entity);
    entity_commands.insert((
        marker,
        BoardGhostInteraction {
            card_id: marker.card_id,
        },
    ));
    if !has_pickable {
        entity_commands.insert((Pickable::default(), BoardGhostPickable));
    }
}

fn insert_objective_marker(
    commands: &mut Commands,
    entity: Entity,
    has_pickable: bool,
    marker: ObjectiveTargetGhost,
) {
    let mut entity_commands = commands.entity(entity);
    entity_commands.insert((
        marker,
        BoardGhostInteraction {
            card_id: marker.card_id,
        },
    ));
    if !has_pickable {
        entity_commands.insert((Pickable::default(), BoardGhostPickable));
    }
}

fn clear_card_ghosts(
    commands: &mut Commands,
    card_id: CardId,
    ghost_units: &Query<(Entity, &GhostUnit)>,
    lane_washes: &Query<(Entity, &LaneGhostWash)>,
    target_markers: &Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: &Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
) {
    for (entity, ghost) in ghost_units {
        if ghost.card_id == card_id {
            despawn_if_exists(commands, entity);
        }
    }

    for (entity, wash) in lane_washes {
        if wash.card_id == card_id {
            despawn_if_exists(commands, entity);
        }
    }

    for (entity, marker, owned_pickable) in target_markers {
        if marker.card_id == card_id {
            remove_target_ghost_marker(commands, entity, owned_pickable.is_some());
        }
    }

    for (entity, marker, owned_pickable) in objective_markers {
        if marker.card_id == card_id {
            remove_objective_ghost_marker(commands, entity, owned_pickable.is_some());
        }
    }
}

fn clear_all_board_ghosts(
    commands: &mut Commands,
    ghost_units: &Query<(Entity, &GhostUnit)>,
    lane_washes: &Query<(Entity, &LaneGhostWash)>,
    target_markers: &Query<(Entity, &TargetUnitGhost, Option<&BoardGhostPickable>)>,
    objective_markers: &Query<(Entity, &ObjectiveTargetGhost, Option<&BoardGhostPickable>)>,
) {
    for (entity, _ghost) in ghost_units {
        despawn_if_exists(commands, entity);
    }

    for (entity, _wash) in lane_washes {
        despawn_if_exists(commands, entity);
    }

    for (entity, _marker, owned_pickable) in target_markers {
        remove_target_ghost_marker(commands, entity, owned_pickable.is_some());
    }

    for (entity, _marker, owned_pickable) in objective_markers {
        remove_objective_ghost_marker(commands, entity, owned_pickable.is_some());
    }
}

fn despawn_if_exists(commands: &mut Commands, entity: Entity) {
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.despawn();
    }
}

fn remove_target_ghost_marker(commands: &mut Commands, entity: Entity, remove_pickable: bool) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    entity_commands.remove::<(TargetUnitGhost, BoardGhostInteraction)>();
    if remove_pickable {
        entity_commands.remove::<(Pickable, BoardGhostPickable)>();
    }
}

fn remove_objective_ghost_marker(commands: &mut Commands, entity: Entity, remove_pickable: bool) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    entity_commands.remove::<(ObjectiveTargetGhost, BoardGhostInteraction)>();
    if remove_pickable {
        entity_commands.remove::<(Pickable, BoardGhostPickable)>();
    }
}

fn board_center(board_layout: &BoardLayout) -> Vec2 {
    Vec2::new(
        board_layout.board_origin.x
            + f32::from(BOARD_CELL_COUNT - 1) * board_layout.cell_width * 0.5,
        board_layout.board_origin.y
            - f32::from(BOARD_LANE_COUNT - 1) * board_layout.lane_height * 0.5,
    )
}
