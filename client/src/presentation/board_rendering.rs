use std::collections::HashMap;

use bevy::prelude::*;
use bevy_tweening::TweenAnim;
use lightyear::prelude::MessageReceiver;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    EntityId, ObjectiveSnapshot, PlayTarget, RoundPhase, S2CGameSnapshot, S2CPlacementReveal,
    UnitBoardLocation, UnitBoardState, UnitStatsSnapshot,
};
use shared::session::PlayerId;

use super::PresentationSet;
use crate::card_animations::{
    cancel_tween_anim_in_place, AnimQueue, BoardRebuildRequested, PendingObjectiveDestroyedEvents,
    PendingPhaseChange, StagedObjectiveRevealQueue,
};
use crate::state::{ClientGameSnapshotMessage, ClientState, CurrentClientPhase};
use crate::ui::hand::{
    GhostClickedEvent, GhostDragStartEvent, GhostPlacementChanged, ObjectiveCell,
    PlacementTargetUnit,
};
use crate::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

pub mod rendering_constants;

pub const UNIT_PLACEHOLDER_FRAME_INDEX: usize = 0;
pub const HP_BAR_WHITE_PIXEL_FRAME_INDEX: usize = 1;
pub const OBJECTIVE_UNKNOWN_FRAME_INDEX: usize = 0;
pub const HP_THRESHOLD_EPSILON: f32 = 1e-4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitAtlasFrame {
    pub frame_index: usize,
    pub max_hp: u8,
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct CardAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub board_elements_image: Handle<Image>,
    pub board_elements_layout: Handle<TextureAtlasLayout>,
    pub unit_frames: HashMap<CardId, UnitAtlasFrame>,
}

impl CardAtlas {
    pub fn with_unit_frame(mut self, card_id: CardId, frame_index: usize, max_hp: u8) -> Self {
        self.unit_frames.insert(
            card_id,
            UnitAtlasFrame {
                frame_index,
                max_hp,
            },
        );
        self
    }

    pub fn unit_frame(&self, card_id: CardId) -> Option<UnitAtlasFrame> {
        self.unit_frames.get(&card_id).copied()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardRenderingEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCamera;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCellNode;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardSnapshotEntity;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnit {
    pub unit_id: EntityId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitOwner(pub PlayerId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitCard {
    pub card_id: Option<CardId>,
    pub frame_index: usize,
    pub used_missing_art_fallback: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitStats {
    pub hp_current: u8,
    pub hp_max: u8,
    pub atk: u8,
    pub mp: u8,
    pub ar: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardUnitSourceClass(pub ClassId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingObjective {
    pub owner_id: PlayerId,
    pub lane: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingObjectiveHp {
    pub hp_current: u8,
    pub hp_max: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpBarBackground;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct HpBarFill;

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveIdentityCache {
    identities: HashMap<(PlayerId, u8), bool>,
}

impl ObjectiveIdentityCache {
    pub fn insert(&mut self, player_id: PlayerId, lane: u8, is_fake: bool) {
        self.identities.insert((player_id, lane), is_fake);
    }

    pub fn clear(&mut self) {
        self.identities.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.identities.is_empty()
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BoardRenderingConfig {
    pub health_bar_green_threshold: f32,
    pub health_bar_red_threshold: f32,
}

impl Default for BoardRenderingConfig {
    fn default() -> Self {
        Self {
            health_bar_green_threshold: 0.6,
            health_bar_red_threshold: 0.3,
        }
    }
}

impl BoardRenderingConfig {
    pub fn assert_valid(self) {
        assert!(
            self.health_bar_red_threshold < self.health_bar_green_threshold,
            "HP threshold config invalid: red_threshold={} >= green_threshold={}",
            self.health_bar_red_threshold,
            self.health_bar_green_threshold
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpBarColor {
    Green,
    Yellow,
    Red,
}

impl HpBarColor {
    pub fn tint(self) -> Color {
        match self {
            Self::Green => Color::srgba(0.2, 0.92, 0.38, 1.0),
            Self::Yellow => Color::srgba(1.0, 0.78, 0.18, 1.0),
            Self::Red => Color::srgba(0.95, 0.18, 0.16, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HpBarVisual {
    pub fill: f32,
    pub color: HpBarColor,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardRenderState {
    #[default]
    Idle,
    Lobby,
    DraftInitial,
    DraftShop,
    DraftAuction,
    Placement,
    Resolution,
    GameOver,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardRenderSet {
    ReadMessages,
    ResolveStateMachine,
    SpawnEntities,
    ScheduleTweens,
    UpdateHpBars,
}

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
        BoardRenderingConfig::default().assert_valid();

        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<BoardRenderingConfig>()
            .init_resource::<BoardRenderState>()
            .init_resource::<ObjectiveIdentityCache>()
            .add_message::<ClientGameSnapshotMessage>()
            .add_message::<BoardRebuildRequested>()
            .add_message::<GhostPlacementChanged>()
            .add_message::<GhostClickedEvent>()
            .add_message::<GhostDragStartEvent>()
            .add_message::<Pointer<Click>>()
            .add_message::<Pointer<Press>>()
            .configure_sets(
                Update,
                (
                    BoardRenderSet::ReadMessages,
                    BoardRenderSet::ResolveStateMachine,
                    BoardRenderSet::SpawnEntities,
                    BoardRenderSet::ScheduleTweens,
                    BoardRenderSet::UpdateHpBars,
                )
                    .chain()
                    .run_if(in_state(ClientState::InSession)),
            )
            .configure_sets(
                Update,
                (
                    BoardRenderSet::ReadMessages.in_set(PresentationSet::MessageDrain),
                    BoardRenderSet::ResolveStateMachine.in_set(PresentationSet::StateSync),
                    BoardRenderSet::SpawnEntities.in_set(PresentationSet::StateSync),
                    BoardRenderSet::ScheduleTweens.in_set(PresentationSet::StateSync),
                    BoardRenderSet::UpdateHpBars.in_set(PresentationSet::StateSync),
                ),
            )
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
                    rebuild_board_from_snapshot_system.in_set(BoardRenderSet::ReadMessages),
                    update_hp_bars_system.in_set(BoardRenderSet::UpdateHpBars),
                ),
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

#[allow(clippy::too_many_arguments)]
fn rebuild_board_from_snapshot_system(
    mut commands: Commands,
    mut snapshots: MessageReader<ClientGameSnapshotMessage>,
    board_layout: Option<Res<BoardLayout>>,
    card_atlas: Option<Res<CardAtlas>>,
    config: Res<BoardRenderingConfig>,
    stale_entities: Query<Entity, With<BoardSnapshotEntity>>,
    mut render_state: ResMut<BoardRenderState>,
    mut current_phase: Option<ResMut<CurrentClientPhase>>,
    mut objective_identity_cache: ResMut<ObjectiveIdentityCache>,
    mut rebuild_writer: MessageWriter<BoardRebuildRequested>,
    mut tweens: Query<&mut TweenAnim>,
    mut anim_queue: Option<ResMut<AnimQueue>>,
    mut pending_phase: Option<ResMut<PendingPhaseChange>>,
    mut pending_objectives: Option<ResMut<PendingObjectiveDestroyedEvents>>,
    mut staged_objectives: Option<ResMut<StagedObjectiveRevealQueue>>,
) {
    let mut latest_snapshot = None;
    for snapshot in snapshots.read() {
        latest_snapshot = Some(snapshot.0.clone());
    }

    let Some(snapshot) = latest_snapshot else {
        return;
    };
    let Some(board_layout) = board_layout else {
        warn!("Board Rendering: snapshot ignored because BoardLayout is missing");
        return;
    };
    let Some(card_atlas) = card_atlas else {
        warn!("Board Rendering: snapshot ignored because CardAtlas is missing");
        return;
    };

    clear_pending_visual_state(
        &mut rebuild_writer,
        &mut tweens,
        anim_queue.as_deref_mut(),
        pending_phase.as_deref_mut(),
        pending_objectives.as_deref_mut(),
        staged_objectives.as_deref_mut(),
    );
    objective_identity_cache.clear();

    for entity in &stale_entities {
        commands.entity(entity).despawn();
    }

    *render_state = BoardRenderState::from_snapshot_phase(snapshot.phase);
    if let Some(current_phase) = current_phase.as_deref_mut() {
        current_phase.phase = snapshot.phase;
        current_phase.round = snapshot.round_number;
    }

    spawn_snapshot_objectives(&mut commands, &board_layout, &card_atlas, &snapshot);
    spawn_snapshot_units(
        &mut commands,
        &board_layout,
        &card_atlas,
        &config,
        &snapshot,
    );
}

fn clear_pending_visual_state(
    rebuild_writer: &mut MessageWriter<BoardRebuildRequested>,
    tweens: &mut Query<&mut TweenAnim>,
    anim_queue: Option<&mut AnimQueue>,
    pending_phase: Option<&mut PendingPhaseChange>,
    pending_objectives: Option<&mut PendingObjectiveDestroyedEvents>,
    staged_objectives: Option<&mut StagedObjectiveRevealQueue>,
) {
    rebuild_writer.write(BoardRebuildRequested);

    if let Some(anim_queue) = anim_queue {
        anim_queue.reset();
    }
    if let Some(pending_phase) = pending_phase {
        pending_phase.clear();
    }
    if let Some(pending_objectives) = pending_objectives {
        pending_objectives.clear();
    }
    if let Some(staged_objectives) = staged_objectives {
        staged_objectives.clear();
    }

    for mut tween in tweens.iter_mut() {
        if let Err(error) = cancel_tween_anim_in_place(&mut tween) {
            warn!("Board Rendering: failed to cancel tween during snapshot rebuild: {error}");
        }
    }
}

fn spawn_snapshot_units(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    config: &BoardRenderingConfig,
    snapshot: &S2CGameSnapshot,
) {
    for unit in &snapshot.board.units {
        spawn_snapshot_unit(commands, board_layout, card_atlas, config, snapshot, unit);
    }
}

fn spawn_snapshot_unit(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    config: &BoardRenderingConfig,
    snapshot: &S2CGameSnapshot,
    unit: &UnitBoardState,
) {
    let Some((lane, cell)) = visible_unit_cell(unit, snapshot.recipient_player_id) else {
        warn!(
            "Board Rendering: unit {:?} has out-of-range snapshot location; skipped",
            unit.unit_id
        );
        return;
    };

    let stats = board_unit_stats(unit, card_atlas);
    let (frame_index, used_missing_art_fallback) = unit_frame_index(unit, card_atlas);
    let world_xy = board_layout.cell_to_world(lane, cell);

    let unit_entity = commands
        .spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            BoardUnit {
                unit_id: unit.unit_id,
            },
            BoardUnitOwner(unit.owner_id),
            BoardUnitCard {
                card_id: unit.card_id,
                frame_index,
                used_missing_art_fallback,
            },
            stats,
            LaneCell { lane, cell },
            unit_sprite(card_atlas, frame_index),
            Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_UNITS),
        ))
        .id();

    if let Some(source_class) = unit.source_class {
        commands
            .entity(unit_entity)
            .insert(BoardUnitSourceClass(source_class));
    }

    spawn_hp_bar_children(commands, unit_entity, card_atlas, stats, config);
}

fn visible_unit_cell(unit: &UnitBoardState, recipient_player_id: PlayerId) -> Option<(u8, u8)> {
    match unit.location {
        UnitBoardLocation::BoardCell { lane, cell } => {
            in_board_bounds(lane, cell).then_some((lane, cell))
        }
        UnitBoardLocation::ObjectiveAttachment { lane } => {
            let cell = if unit.owner_id == recipient_player_id {
                BOARD_CELL_COUNT
            } else {
                1
            };
            in_board_bounds(lane, cell).then_some((lane, cell))
        }
    }
}

fn in_board_bounds(lane: u8, cell: u8) -> bool {
    (1..=BOARD_LANE_COUNT).contains(&lane) && (1..=BOARD_CELL_COUNT).contains(&cell)
}

fn unit_frame_index(unit: &UnitBoardState, card_atlas: &CardAtlas) -> (usize, bool) {
    let Some(card_id) = unit.card_id else {
        warn!(
            "Board Rendering asset-miss: unit {:?} has no card_id",
            unit.unit_id
        );
        return (UNIT_PLACEHOLDER_FRAME_INDEX, true);
    };

    if let Some(frame) = card_atlas.unit_frame(card_id) {
        (frame.frame_index, false)
    } else {
        warn!(
            "Board Rendering asset-miss: missing art for card_id {:?}; using placeholder",
            card_id
        );
        (UNIT_PLACEHOLDER_FRAME_INDEX, true)
    }
}

fn board_unit_stats(unit: &UnitBoardState, card_atlas: &CardAtlas) -> BoardUnitStats {
    let stats = unit.stats.unwrap_or_else(|| {
        warn!(
            "Board Rendering: unit {:?} missing stats in snapshot; defaulting to 1 HP",
            unit.unit_id
        );
        UnitStatsSnapshot {
            hp: 1,
            atk: 0,
            mp: 0,
            ar: 0,
        }
    });
    let hp_max = unit
        .card_id
        .and_then(|card_id| card_atlas.unit_frame(card_id))
        .map(|frame| frame.max_hp)
        .unwrap_or(stats.hp)
        .max(1);

    BoardUnitStats {
        hp_current: stats.hp,
        hp_max,
        atk: stats.atk,
        mp: stats.mp,
        ar: stats.ar,
    }
}

fn spawn_snapshot_objectives(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    snapshot: &S2CGameSnapshot,
) {
    for player in &snapshot.players {
        for objective in &player.objectives {
            spawn_standing_objective(
                commands,
                board_layout,
                card_atlas,
                snapshot.recipient_player_id,
                player.player_id,
                objective,
            );
        }
    }
}

fn spawn_standing_objective(
    commands: &mut Commands,
    board_layout: &BoardLayout,
    card_atlas: &CardAtlas,
    recipient_player_id: PlayerId,
    owner_id: PlayerId,
    objective: &ObjectiveSnapshot,
) {
    if objective.is_destroyed {
        return;
    }

    let lane = objective.lane;
    let cell = objective_cell(owner_id, recipient_player_id);
    if !in_board_bounds(lane, cell) {
        warn!(
            "Board Rendering: objective for {:?} lane {} is out of range; skipped",
            owner_id, lane
        );
        return;
    }

    let world_xy = board_layout.cell_to_world(lane, cell);
    let hp = StandingObjectiveHp {
        hp_current: objective.hp,
        hp_max: objective.hp.max(1),
    };
    let objective_entity = commands
        .spawn((
            BoardRenderingEntity,
            BoardSnapshotEntity,
            StandingObjective { owner_id, lane },
            hp,
            LaneCell { lane, cell },
            objective_unknown_sprite(card_atlas),
            Transform::from_xyz(world_xy.x, world_xy.y, rendering_constants::Z_OBJECTIVES),
        ))
        .id();

    spawn_objective_hp_bar_children(commands, objective_entity, card_atlas, hp);
}

fn objective_cell(owner_id: PlayerId, recipient_player_id: PlayerId) -> u8 {
    if owner_id == recipient_player_id {
        1
    } else {
        BOARD_CELL_COUNT
    }
}

fn spawn_hp_bar_children(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    stats: BoardUnitStats,
    config: &BoardRenderingConfig,
) {
    let visual = hp_bar_visual(stats.hp_current, stats.hp_max, *config);
    spawn_hp_bar_background(commands, parent, card_atlas);
    spawn_hp_bar_fill(commands, parent, card_atlas, visual);
}

fn spawn_objective_hp_bar_children(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    hp: StandingObjectiveHp,
) {
    let visual = hp_bar_visual(hp.hp_current, hp.hp_max, BoardRenderingConfig::default());
    spawn_hp_bar_background(commands, parent, card_atlas);
    spawn_hp_bar_fill(commands, parent, card_atlas, visual);
}

fn spawn_hp_bar_background(commands: &mut Commands, parent: Entity, card_atlas: &CardAtlas) {
    commands.spawn((
        BoardRenderingEntity,
        BoardSnapshotEntity,
        HpBarBackground,
        hp_bar_sprite(
            card_atlas,
            Color::srgba(0.08, 0.08, 0.08, 0.76),
            rendering_constants::HP_BAR_SIZE,
        ),
        Transform::from_xyz(
            0.0,
            rendering_constants::HP_BAR_Y_OFFSET,
            rendering_constants::HEALTH_BAR_LOCAL_Z,
        ),
        Visibility::Inherited,
        ChildOf(parent),
    ));
}

fn spawn_hp_bar_fill(
    commands: &mut Commands,
    parent: Entity,
    card_atlas: &CardAtlas,
    visual: HpBarVisual,
) {
    commands.spawn((
        BoardRenderingEntity,
        BoardSnapshotEntity,
        HpBarFill,
        hp_bar_sprite(
            card_atlas,
            visual.color.tint(),
            rendering_constants::HP_BAR_SIZE,
        ),
        Transform {
            translation: Vec3::new(
                hp_fill_offset_x(visual.fill),
                rendering_constants::HP_BAR_Y_OFFSET,
                rendering_constants::HEALTH_BAR_LOCAL_Z,
            ),
            scale: Vec3::new(visual.fill, 1.0, 1.0),
            ..default()
        },
        Visibility::Inherited,
        ChildOf(parent),
    ));
}

fn unit_sprite(card_atlas: &CardAtlas, frame_index: usize) -> Sprite {
    atlas_sprite(
        card_atlas.image.clone(),
        card_atlas.layout.clone(),
        frame_index,
        rendering_constants::UNIT_SPRITE_SIZE,
        Color::srgba(1.0, 1.0, 1.0, 1.0),
    )
}

fn objective_unknown_sprite(card_atlas: &CardAtlas) -> Sprite {
    atlas_sprite(
        card_atlas.board_elements_image.clone(),
        card_atlas.board_elements_layout.clone(),
        OBJECTIVE_UNKNOWN_FRAME_INDEX,
        rendering_constants::OBJECTIVE_SPRITE_SIZE,
        Color::srgba(1.0, 1.0, 1.0, 1.0),
    )
}

fn hp_bar_sprite(card_atlas: &CardAtlas, color: Color, size: Vec2) -> Sprite {
    atlas_sprite(
        card_atlas.image.clone(),
        card_atlas.layout.clone(),
        HP_BAR_WHITE_PIXEL_FRAME_INDEX,
        size,
        color,
    )
}

fn atlas_sprite(
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    index: usize,
    custom_size: Vec2,
    color: Color,
) -> Sprite {
    Sprite {
        image,
        texture_atlas: Some(TextureAtlas { layout, index }),
        custom_size: Some(custom_size),
        color,
        ..default()
    }
}

fn update_hp_bars_system(
    config: Res<BoardRenderingConfig>,
    units: Query<(&BoardUnitStats, &Children), With<BoardUnit>>,
    mut fills: Query<(&mut Transform, &mut Sprite), With<HpBarFill>>,
) {
    for (stats, children) in &units {
        let visual = hp_bar_visual(stats.hp_current, stats.hp_max, *config);
        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = fills.get_mut(child) {
                apply_hp_fill_visual(&mut transform, &mut sprite, visual);
            }
        }
    }
}

fn apply_hp_fill_visual(transform: &mut Transform, sprite: &mut Sprite, visual: HpBarVisual) {
    transform.scale.x = visual.fill;
    transform.translation.x = hp_fill_offset_x(visual.fill);
    sprite.color = visual.color.tint();
}

pub fn hp_bar_visual(hp_current: u8, hp_max: u8, config: BoardRenderingConfig) -> HpBarVisual {
    config.assert_valid();
    let hp_max_safe = hp_max.max(1);
    if hp_max == 0 {
        warn!("Board Rendering: UnitStats.hp_max=0 from server; clamped to 1");
    }

    let fill = (f32::from(hp_current) / f32::from(hp_max_safe)).clamp(0.0, 1.0);
    let color = if fill >= config.health_bar_green_threshold - HP_THRESHOLD_EPSILON {
        HpBarColor::Green
    } else if fill >= config.health_bar_red_threshold - HP_THRESHOLD_EPSILON {
        HpBarColor::Yellow
    } else {
        HpBarColor::Red
    };

    HpBarVisual { fill, color }
}

fn hp_fill_offset_x(fill: f32) -> f32 {
    -rendering_constants::HP_BAR_SIZE.x * (1.0 - fill) * 0.5
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

impl BoardRenderState {
    fn from_snapshot_phase(phase: RoundPhase) -> Self {
        match phase {
            RoundPhase::Handshaking => Self::Idle,
            RoundPhase::Lobby => Self::Lobby,
            RoundPhase::DraftInitial => Self::DraftInitial,
            RoundPhase::DraftShop => Self::DraftShop,
            RoundPhase::DraftAuction => Self::DraftAuction,
            RoundPhase::Placement => Self::Placement,
            RoundPhase::Resolution => Self::Resolution,
            RoundPhase::GameOver => Self::GameOver,
        }
    }
}
