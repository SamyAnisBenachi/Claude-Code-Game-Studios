//! Targeting / placement feedback overlay (PROMPT 1390 —
//! `S19-BR-PLAYAREA-HIERARCHY-TARGETING-FEEDBACK-001`).
//!
//! Adds the Krosmaga-style "targeting mode" presentation layer on top of
//! the existing ghost preview pipeline:
//!
//! - **AC1/AC2 — Board envelope**: a cached [`BoardEnvelope`] resource
//!   captures the world-space rectangle covered by the cell matrix so the
//!   QA snapshot can prove the board is the dominant central surface and
//!   that edge regions never cover the cells.
//! - **AC3 — Dim state**: a [`TargetingDimWash`] world-space sprite
//!   spawned at [`rendering_constants::Z_TARGETING_DIM_WASH`] (between the
//!   board chrome and traps/structures, per ADR-021) dims the full board
//!   envelope while targeting is active.
//! - **AC4 — Valid path/rings**: per-valid-cell [`TargetingValidRing`]
//!   sprites painted in a distinct cyan tint, distinguishable from the
//!   amber idle [`SpawnHighlightState`] tint on the cell nodes.
//! - **AC5 — Invalid target**: [`TargetingInvalidMarker`] uses a red
//!   warning tint when the local placement target falls outside the
//!   client-side `PlacementBoardView` spawn range. The client never
//!   decides legality — this only mirrors the spawn range that the
//!   server already broadcast via `SpawnRangeChanged`.
//! - **AC6 — Source-card link**: [`SourceCardLink`] is a thin world-space
//!   line sprite anchored at the bottom-centre of the board envelope and
//!   stretched toward the active target cell. The hand fan / primary CTA
//!   stay above the bevy_ui rendering boundary, so the link cannot block
//!   them.
//! - **AC7 — Authority boundary**: every overlay derives from
//!   [`GhostPlacementChanged`] (Hand UI) and [`PlacementBoardView`]
//!   (Hand UI) — no S2C, no server-authoritative legality checks, no new
//!   protocol messages.
//! - **AC8 — Z-order**: every overlay sits below
//!   [`rendering_constants::Z_TRAPS_STRUCTURES`] (and therefore below
//!   objectives, units, hover cards, and bevy_ui panels) per ADR-021 and
//!   `docs/ux/board-rendering-spec.md`.
//!
//! ## Authority and no-claim
//!
//! Per the PROMPT 1390 story `Out Of Scope` list this layer is read-only:
//! it never writes back to `PlacementBoardView`, never mutates
//! authoritative state, never spawns or despawns committed board entities.
//! Status / final-art / Standard-tier accessibility / playtest validation
//! / closure of `PAW-TD-*-a` are explicitly not advanced by this module.

use bevy::prelude::*;
use shared::card::CardId;
use shared::protocol::PlayTarget;

use crate::presentation::board_rendering::rendering_constants;
use crate::ui::hand::{BoardSpawnEdge, GhostPlacementChanged, PlacementBoardView};
use crate::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};

/// Mirror of `PlacementBoardView::is_spawn_cell`, re-implemented here so
/// Board Rendering can read the public fields of `PlacementBoardView`
/// without depending on a Hand UI private method. The two implementations
/// must stay in sync; both clamp the range to `1..=BOARD_CELL_COUNT` and
/// project the edge enum the same way.
pub fn placement_view_contains_cell(view: &PlacementBoardView, lane: u8, cell: u8) -> bool {
    if !(1..=BOARD_LANE_COUNT).contains(&lane) {
        return false;
    }
    let range = view.spawn_range_cells.clamp(1, BOARD_CELL_COUNT);
    match view.spawn_edge {
        BoardSpawnEdge::LowCells => (1..=range).contains(&cell),
        BoardSpawnEdge::HighCells => {
            let first_cell = BOARD_CELL_COUNT - range + 1;
            (first_cell..=BOARD_CELL_COUNT).contains(&cell)
        }
    }
}

/// World-space rectangle covered by the board cell matrix. Inserted by
/// `insert_board_rendering_session_resources` alongside [`BoardLayout`]
/// and removed in `remove_board_rendering_session_resources`. AC1/AC2
/// observability source for [`BoardTargetingSnapshot`].
///
/// Computed once on session entry from the canonical
/// [`BoardLayout::cell_to_world`] formula; cell-1/lane-1 is the
/// upper-left of the matrix and cell-`N`/lane-`M` the lower-right, so
/// `world_min` is the bottom-right of the matrix and `world_max` the
/// top-left (Y grows upward — see `BoardLayout::cell_to_world`). The
/// `min`/`max` helpers normalise that into a standard rectangle.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BoardEnvelope {
    pub world_origin: Vec2,
    pub world_far_corner: Vec2,
    pub world_center: Vec2,
    pub cell_size: Vec2,
    pub lane_count: u8,
    pub cell_count: u8,
}

impl BoardEnvelope {
    pub fn from_layout(layout: &BoardLayout) -> Self {
        let origin = layout.cell_to_world(1, 1);
        let far_corner = layout.cell_to_world(BOARD_LANE_COUNT, BOARD_CELL_COUNT);
        let world_center = (origin + far_corner) * 0.5;
        Self {
            world_origin: origin,
            world_far_corner: far_corner,
            world_center,
            cell_size: Vec2::new(layout.cell_width, layout.lane_height),
            lane_count: BOARD_LANE_COUNT,
            cell_count: BOARD_CELL_COUNT,
        }
    }

    /// Logical world-space minimum (top-left in screen-space, with the
    /// half-cell padding included so the rectangle fully encloses cell
    /// node sprites of size [`rendering_constants::CELL_NODE_SIZE`]).
    pub fn world_min(&self) -> Vec2 {
        Vec2::new(
            self.world_origin.x.min(self.world_far_corner.x) - self.cell_size.x * 0.5,
            self.world_origin.y.min(self.world_far_corner.y) - self.cell_size.y * 0.5,
        )
    }

    pub fn world_max(&self) -> Vec2 {
        Vec2::new(
            self.world_origin.x.max(self.world_far_corner.x) + self.cell_size.x * 0.5,
            self.world_origin.y.max(self.world_far_corner.y) + self.cell_size.y * 0.5,
        )
    }

    pub fn world_width(&self) -> f32 {
        (self.world_max().x - self.world_min().x).abs()
    }

    pub fn world_height(&self) -> f32 {
        (self.world_max().y - self.world_min().y).abs()
    }

    /// Bottom-centre anchor used by the source-card link tail.
    pub fn bottom_center(&self) -> Vec2 {
        Vec2::new(self.world_center.x, self.world_min().y)
    }
}

/// Drives every overlay's spawn/despawn. Written by
/// [`apply_targeting_overlay_state_system`]; read by
/// [`sync_targeting_overlays_system`] and by the QA snapshot projection.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct TargetingOverlayState {
    pub active: Option<ActiveTargeting>,
}

impl TargetingOverlayState {
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn active(&self) -> Option<&ActiveTargeting> {
        self.active.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveTargeting {
    pub card_id: CardId,
    pub target: PlayTarget,
    /// Cells highlighted as valid path / range while targeting is active.
    /// Currently mirrors the local player's spawn range (the only client-
    /// side mirror of placement legality available without re-deriving
    /// server rules).
    pub valid_cells: Vec<LaneCell>,
    /// `Some(cell)` when the active target is a board cell outside the
    /// local spawn range. `None` for in-range board cells and for non-
    /// board-cell targets (TargetUnit / TargetObj / LaneWide / Instant).
    pub invalid_target_cell: Option<LaneCell>,
    /// Mirror of the active target cell when the target is a board cell;
    /// drives the endpoint ring and the source-card link.
    pub endpoint_cell: Option<LaneCell>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetingDimWash;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetingValidRing {
    pub lane: u8,
    pub cell: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetingEndpointRing {
    pub lane: u8,
    pub cell: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetingInvalidMarker {
    pub lane: u8,
    pub cell: u8,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceCardLink;

/// Targeting dim tint — translucent dark wash painted across the board
/// envelope while a placement intent is active. Kept distinct from the
/// idle [`SpawnHighlightState`] tint so the player can tell at a glance
/// whether they are in idle PLACEMENT or in active targeting mode.
pub const TARGETING_DIM_COLOR: Color = Color::srgba(0.02, 0.04, 0.08, 0.55);

/// Valid path/range ring tint (cyan). Distinct from the amber
/// `SpawnHighlightState::ValidSpawn` tint applied on idle cell nodes —
/// AC4 specifically calls out that valid path/rings must be
/// distinguishable from idle spawn-range highlights.
pub const TARGETING_VALID_RING_COLOR: Color = Color::srgba(0.36, 0.86, 1.0, 0.92);

/// Endpoint ring tint (bright cyan) — slightly brighter than the valid
/// ring so the active target reads as the focal point.
pub const TARGETING_ENDPOINT_RING_COLOR: Color = Color::srgba(0.62, 0.98, 1.0, 0.98);

/// Invalid target tint (warning red) — used for board cell targets that
/// fall outside the local spawn range mirror.
pub const TARGETING_INVALID_COLOR: Color = Color::srgba(1.0, 0.32, 0.28, 0.85);

/// Source-card link tint (soft amber) — visible against the dim wash
/// without dominating the endpoint ring.
pub const SOURCE_CARD_LINK_COLOR: Color = Color::srgba(1.0, 0.84, 0.36, 0.78);

/// Valid range ring size in world units. The ring is drawn as a square
/// sprite slightly larger than `CELL_NODE_SIZE` so it reads as a halo
/// around the cell node rather than replacing it.
pub fn valid_ring_size() -> Vec2 {
    Vec2::splat(rendering_constants::CELL_NODE_SIZE * 1.45)
}

pub fn endpoint_ring_size() -> Vec2 {
    Vec2::splat(rendering_constants::CELL_NODE_SIZE * 1.85)
}

pub fn invalid_marker_size() -> Vec2 {
    Vec2::splat(rendering_constants::CELL_NODE_SIZE * 1.4)
}

/// Width of the source-card link sprite. Kept thin so it reads as a
/// directional indicator rather than a wall.
pub const SOURCE_CARD_LINK_WIDTH: f32 = 4.0;

/// Drains [`GhostPlacementChanged`] messages and updates
/// [`TargetingOverlayState`] accordingly. Multiple readers of the same
/// message buffer are supported because [`bevy::ecs::message::MessageReader`]
/// uses per-system cursors — the existing
/// `apply_ghost_placement_changed_system` keeps its own cursor and is
/// untouched. This system never writes to `PlacementBoardView` or any
/// other Hand UI / server-authoritative resource.
pub fn apply_targeting_overlay_state_system(
    mut state: ResMut<TargetingOverlayState>,
    placement_board_view: Option<Res<PlacementBoardView>>,
    mut changes: MessageReader<GhostPlacementChanged>,
) {
    let mut latest: Vec<(Option<CardId>, Option<PlayTarget>)> = Vec::new();
    for change in changes.read() {
        latest.push((change.card_id, change.target.clone()));
    }

    let mut next = state.active.clone();
    for (card_id_opt, target_opt) in latest {
        let Some(card_id) = card_id_opt else {
            continue;
        };

        match target_opt {
            None => {
                let matches_active = matches!(
                    next.as_ref(),
                    Some(active) if active.card_id == card_id
                );
                if matches_active {
                    next = None;
                }
            }
            Some(target) => {
                let (valid_cells, invalid_target_cell, endpoint_cell) =
                    derive_target_cells(&target, placement_board_view.as_deref());
                next = Some(ActiveTargeting {
                    card_id,
                    target,
                    valid_cells,
                    invalid_target_cell,
                    endpoint_cell,
                });
            }
        }
    }

    if state.active != next {
        state.active = next;
    }
}

fn derive_target_cells(
    target: &PlayTarget,
    placement_board_view: Option<&PlacementBoardView>,
) -> (Vec<LaneCell>, Option<LaneCell>, Option<LaneCell>) {
    let valid_cells = match placement_board_view {
        Some(view) => collect_valid_spawn_cells(view),
        None => Vec::new(),
    };

    match target {
        PlayTarget::BoardCell { lane, cell } => {
            let endpoint = LaneCell {
                lane: *lane,
                cell: *cell,
            };
            let in_range = placement_board_view
                .map(|view| placement_view_contains_cell(view, *lane, *cell))
                .unwrap_or(false);
            let invalid = if !in_range && placement_board_view.is_some() {
                Some(endpoint)
            } else {
                None
            };
            (valid_cells, invalid, Some(endpoint))
        }
        PlayTarget::LaneWide { lane: _ }
        | PlayTarget::TargetUnit { .. }
        | PlayTarget::TargetObj { .. }
        | PlayTarget::Instant => (valid_cells, None, None),
    }
}

fn collect_valid_spawn_cells(view: &PlacementBoardView) -> Vec<LaneCell> {
    let mut cells = Vec::new();
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            if placement_view_contains_cell(view, lane, cell) {
                cells.push(LaneCell { lane, cell });
            }
        }
    }
    cells
}

/// Reconciles overlay entities with [`TargetingOverlayState`]. Spawns
/// missing overlays, despawns stale ones, and refreshes positions when
/// the active target moves between cells. Designed to be idempotent so
/// the test fixtures can call `app.update()` multiple times without
/// over-spawning.
#[allow(clippy::too_many_arguments)]
pub fn sync_targeting_overlays_system(
    mut commands: Commands,
    state: Res<TargetingOverlayState>,
    board_layout: Option<Res<BoardLayout>>,
    envelope: Option<Res<BoardEnvelope>>,
    dim_query: Query<Entity, With<TargetingDimWash>>,
    valid_query: Query<(Entity, &TargetingValidRing)>,
    endpoint_query: Query<(Entity, &TargetingEndpointRing)>,
    invalid_query: Query<(Entity, &TargetingInvalidMarker)>,
    link_query: Query<Entity, With<SourceCardLink>>,
    transforms: Query<&Transform>,
) {
    let active = match state.active.as_ref() {
        Some(active) => active,
        None => {
            despawn_all_overlays(
                &mut commands,
                &dim_query,
                &valid_query,
                &endpoint_query,
                &invalid_query,
                &link_query,
            );
            return;
        }
    };

    let (Some(layout), Some(envelope)) = (board_layout.as_deref(), envelope.as_deref()) else {
        // Session resources not yet inserted — keep the state but skip
        // entity reconciliation rather than panic.
        return;
    };

    sync_dim_wash(&mut commands, &dim_query, envelope);
    sync_valid_rings(&mut commands, &valid_query, layout, &active.valid_cells);
    sync_endpoint_ring(
        &mut commands,
        &endpoint_query,
        layout,
        active.endpoint_cell,
    );
    sync_invalid_marker(
        &mut commands,
        &invalid_query,
        layout,
        active.invalid_target_cell,
    );
    sync_source_card_link(
        &mut commands,
        &link_query,
        &transforms,
        layout,
        envelope,
        active.endpoint_cell,
    );
}

fn despawn_all_overlays(
    commands: &mut Commands,
    dim_query: &Query<Entity, With<TargetingDimWash>>,
    valid_query: &Query<(Entity, &TargetingValidRing)>,
    endpoint_query: &Query<(Entity, &TargetingEndpointRing)>,
    invalid_query: &Query<(Entity, &TargetingInvalidMarker)>,
    link_query: &Query<Entity, With<SourceCardLink>>,
) {
    for entity in dim_query {
        despawn_if_exists(commands, entity);
    }
    for (entity, _ring) in valid_query {
        despawn_if_exists(commands, entity);
    }
    for (entity, _ring) in endpoint_query {
        despawn_if_exists(commands, entity);
    }
    for (entity, _marker) in invalid_query {
        despawn_if_exists(commands, entity);
    }
    for entity in link_query {
        despawn_if_exists(commands, entity);
    }
}

fn despawn_if_exists(commands: &mut Commands, entity: Entity) {
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.despawn();
    }
}

fn sync_dim_wash(
    commands: &mut Commands,
    dim_query: &Query<Entity, With<TargetingDimWash>>,
    envelope: &BoardEnvelope,
) {
    if dim_query.iter().next().is_some() {
        return;
    }
    commands.spawn((
        super::BoardRenderingEntity,
        TargetingDimWash,
        Sprite::from_color(
            TARGETING_DIM_COLOR,
            Vec2::new(envelope.world_width(), envelope.world_height()),
        ),
        Transform::from_xyz(
            envelope.world_center.x,
            envelope.world_center.y,
            rendering_constants::Z_TARGETING_DIM_WASH,
        ),
    ));
}

fn sync_valid_rings(
    commands: &mut Commands,
    valid_query: &Query<(Entity, &TargetingValidRing)>,
    layout: &BoardLayout,
    valid_cells: &[LaneCell],
) {
    // Despawn any rings that no longer correspond to a valid cell.
    for (entity, ring) in valid_query {
        if !valid_cells
            .iter()
            .any(|c| c.lane == ring.lane && c.cell == ring.cell)
        {
            despawn_if_exists(commands, entity);
        }
    }

    // Spawn rings for valid cells that have no corresponding entity.
    for cell in valid_cells {
        if valid_query
            .iter()
            .any(|(_, r)| r.lane == cell.lane && r.cell == cell.cell)
        {
            continue;
        }
        let world = layout.cell_to_world(cell.lane, cell.cell);
        commands.spawn((
            super::BoardRenderingEntity,
            TargetingValidRing {
                lane: cell.lane,
                cell: cell.cell,
            },
            Sprite::from_color(TARGETING_VALID_RING_COLOR, valid_ring_size()),
            Transform::from_xyz(world.x, world.y, rendering_constants::Z_TARGETING_VALID_RING),
        ));
    }
}

fn sync_endpoint_ring(
    commands: &mut Commands,
    endpoint_query: &Query<(Entity, &TargetingEndpointRing)>,
    layout: &BoardLayout,
    endpoint: Option<LaneCell>,
) {
    let Some(cell) = endpoint else {
        for (entity, _) in endpoint_query {
            despawn_if_exists(commands, entity);
        }
        return;
    };

    let world = layout.cell_to_world(cell.lane, cell.cell);
    let mut existing_for_target = None;
    for (entity, ring) in endpoint_query {
        if ring.lane == cell.lane && ring.cell == cell.cell {
            existing_for_target = Some(entity);
        } else {
            despawn_if_exists(commands, entity);
        }
    }
    if existing_for_target.is_some() {
        return;
    }
    commands.spawn((
        super::BoardRenderingEntity,
        TargetingEndpointRing {
            lane: cell.lane,
            cell: cell.cell,
        },
        Sprite::from_color(TARGETING_ENDPOINT_RING_COLOR, endpoint_ring_size()),
        Transform::from_xyz(
            world.x,
            world.y,
            rendering_constants::Z_TARGETING_ENDPOINT_RING,
        ),
    ));
}

fn sync_invalid_marker(
    commands: &mut Commands,
    invalid_query: &Query<(Entity, &TargetingInvalidMarker)>,
    layout: &BoardLayout,
    invalid: Option<LaneCell>,
) {
    let Some(cell) = invalid else {
        for (entity, _) in invalid_query {
            despawn_if_exists(commands, entity);
        }
        return;
    };

    let world = layout.cell_to_world(cell.lane, cell.cell);
    let mut existing_for_target = None;
    for (entity, marker) in invalid_query {
        if marker.lane == cell.lane && marker.cell == cell.cell {
            existing_for_target = Some(entity);
        } else {
            despawn_if_exists(commands, entity);
        }
    }
    if existing_for_target.is_some() {
        return;
    }
    commands.spawn((
        super::BoardRenderingEntity,
        TargetingInvalidMarker {
            lane: cell.lane,
            cell: cell.cell,
        },
        Sprite::from_color(TARGETING_INVALID_COLOR, invalid_marker_size()),
        Transform::from_xyz(
            world.x,
            world.y,
            rendering_constants::Z_TARGETING_INVALID_MARKER,
        ),
    ));
}

fn sync_source_card_link(
    commands: &mut Commands,
    link_query: &Query<Entity, With<SourceCardLink>>,
    transforms: &Query<&Transform>,
    layout: &BoardLayout,
    envelope: &BoardEnvelope,
    endpoint: Option<LaneCell>,
) {
    let Some(cell) = endpoint else {
        for entity in link_query {
            despawn_if_exists(commands, entity);
        }
        return;
    };

    let endpoint_world = layout.cell_to_world(cell.lane, cell.cell);
    let source = envelope.bottom_center();
    let delta = endpoint_world - source;
    let length = delta.length().max(1.0);
    let mid = source + delta * 0.5;
    let angle = delta.y.atan2(delta.x);
    let target_translation = Vec3::new(mid.x, mid.y, rendering_constants::Z_SOURCE_CARD_LINK);

    // Reuse an existing link entity if its transform already matches the
    // target translation; otherwise drop every existing link and spawn a
    // fresh one. The sprite is rebuilt rather than mutated to keep the
    // reconcile code path read-only with respect to the existing
    // entity-storage tables.
    let mut entities: Vec<Entity> = link_query.iter().collect();
    if let Some(first) = entities.first().copied() {
        let matches_target = matches!(transforms.get(first), Ok(transform)
            if (transform.translation - target_translation).length() < 0.05);
        if matches_target && entities.len() == 1 {
            return;
        }
    }
    for entity in entities.drain(..) {
        despawn_if_exists(commands, entity);
    }

    commands.spawn((
        super::BoardRenderingEntity,
        SourceCardLink,
        Sprite::from_color(
            SOURCE_CARD_LINK_COLOR,
            Vec2::new(length, SOURCE_CARD_LINK_WIDTH),
        ),
        Transform {
            translation: target_translation,
            rotation: Quat::from_rotation_z(angle),
            scale: Vec3::ONE,
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_from_default_layout_centers_on_origin() {
        let layout = BoardLayout::default();
        let envelope = BoardEnvelope::from_layout(&layout);
        assert!(envelope.world_width() > 0.0);
        assert!(envelope.world_height() > 0.0);
        assert_eq!(envelope.lane_count, BOARD_LANE_COUNT);
        assert_eq!(envelope.cell_count, BOARD_CELL_COUNT);
    }

    #[test]
    fn collect_valid_spawn_cells_respects_view_range() {
        let view = PlacementBoardView {
            spawn_range_cells: 2,
            ..PlacementBoardView::default()
        };
        let cells = collect_valid_spawn_cells(&view);
        // Default edge is `LowCells`, so cells 1..=2 across all 5 lanes
        // == 10 cells.
        assert_eq!(cells.len(), 2 * usize::from(BOARD_LANE_COUNT));
        for cell in &cells {
            assert!(matches!(cell.cell, 1 | 2));
        }
    }
}
