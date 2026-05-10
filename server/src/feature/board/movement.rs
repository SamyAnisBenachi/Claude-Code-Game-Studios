use bevy::ecs::system::ParamSet;
use bevy::prelude::{
    Commands, Component, Entity, Message, MessageReader, MessageWriter, Query, Res, ResMut, With,
};
use shared::session::PlayerId;

use crate::core::board::{BoardPosition, UnitOwner, UnitStats};
use crate::core::session::SessionConfig;
use crate::feature::board::{BoardConfig, BoardOccupancy, LaneId};
use crate::feature::prism::{PrismCollected, PrismLaneKey, PrismPresence};

const PLAYER_A_TEAM_ID: u8 = 0;
const PLAYER_B_TEAM_ID: u8 = 1;

/// Bonus movement cells applied during RESOLUTION sub-step 2.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChargeBonus(pub u8);

/// Marker for units that silently ignore displacement keyword effects.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Irremovable;

/// Board-owned REPEL displacement request.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepelDisplacement {
    pub target: Entity,
    pub amount: u8,
}

/// Board-owned ATTRACT displacement request.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttractDisplacement {
    pub caster: Entity,
    pub target: Entity,
    pub amount: u8,
}

/// Board-owned CHANGE LANE displacement request.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeLaneDisplacement {
    pub target: Entity,
    /// Positive values move toward higher lane numbers; negative values move lower.
    pub delta: i8,
}

/// Internal board message emitted when a Trap fires on enemy entry.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrapTrigger {
    pub trap_entity: Entity,
    pub unit_entity: Entity,
    pub trap_owner: PlayerId,
    pub unit_owner: PlayerId,
    pub lane: LaneId,
    pub cell: u8,
}

/// Applies Formula F1 to compute the new cell after one movement sub-step.
pub fn apply_f1(current_cell: u8, direction: i16, mp: u8, cell_min: u8, cell_max: u8) -> u8 {
    let new_cell = current_cell as i16 + direction * mp as i16;
    new_cell.clamp(cell_min as i16, cell_max as i16) as u8
}

/// Applies REPEL X by pushing the target toward its own spawn side.
pub fn apply_repel(
    target_cell: u8,
    target_owner: PlayerId,
    repel_amount: u8,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> Option<u8> {
    let direction = -advance_direction(target_owner, session_config, board_config)?;
    Some(apply_f1(
        target_cell,
        direction,
        repel_amount,
        board_config.cell_min,
        board_config.cell_max,
    ))
}

/// Applies ATTRACT X by pulling the target toward the caster's current cell.
pub fn apply_attract(
    caster_cell: u8,
    target_cell: u8,
    attract_amount: u8,
    target_is_enemy: bool,
    board_config: &BoardConfig,
) -> u8 {
    let direction = (caster_cell as i16 - target_cell as i16).signum();
    let distance = (caster_cell as i16 - target_cell as i16).abs();
    let max_pull = if target_is_enemy {
        distance.saturating_sub(1)
    } else {
        distance
    };
    let effective_amount = attract_amount.min(max_pull as u8);

    apply_f1(
        target_cell,
        direction,
        effective_amount,
        board_config.cell_min,
        board_config.cell_max,
    )
}

/// Applies CHANGE LANE bounds and same-owner minion occupancy checks.
pub fn apply_change_lane(
    unit_lane: LaneId,
    delta: i8,
    occupancy: &BoardOccupancy,
    unit_owner: PlayerId,
    board_config: &BoardConfig,
) -> LaneId {
    let new_lane = unit_lane as i16 + delta as i16;
    if new_lane < 1 || new_lane > i16::from(board_config.lane_count) {
        return unit_lane;
    }

    let new_lane = new_lane as LaneId;
    if occupancy.minion_slots.contains_key(&(unit_owner, new_lane)) {
        return unit_lane;
    }

    new_lane
}

/// Resolves the board-advance direction for the owning player's team side.
pub fn advance_direction(
    player: PlayerId,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> Option<i16> {
    match session_config.team_map.get(&player).copied() {
        Some(PLAYER_A_TEAM_ID) => Some(board_config.player_a_direction),
        Some(PLAYER_B_TEAM_ID) => Some(board_config.player_b_direction),
        _ => None,
    }
}

/// Returns the enemy Trap at a final destination cell, if one can trigger.
///
/// Trap ownership is team-aware through `SessionConfig`; if either player has
/// no team assignment, the Trap is treated as non-triggering.
pub fn check_trap_trigger(
    occupancy: &BoardOccupancy,
    session_config: &SessionConfig,
    unit_owner: PlayerId,
    lane: LaneId,
    destination_cell: u8,
) -> Option<(PlayerId, Entity)> {
    let unit_team = session_config.team_map.get(&unit_owner).copied()?;

    occupancy
        .traps
        .iter()
        .filter_map(|((trap_owner, trap_lane, trap_cell), trap_entity)| {
            if *trap_lane != lane || *trap_cell != destination_cell {
                return None;
            }

            let trap_team = session_config.team_map.get(trap_owner).copied()?;
            (trap_team != unit_team).then_some((*trap_owner, *trap_entity))
        })
        .min_by_key(|(trap_owner, _)| trap_owner.0)
}

/// Resolves the spawn-side prism cell for the owning player's team side.
pub fn own_prism_cell(
    player: PlayerId,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> Option<u8> {
    match session_config.team_map.get(&player).copied() {
        Some(PLAYER_A_TEAM_ID) => Some(board_config.player_a_spawn_cell),
        Some(PLAYER_B_TEAM_ID) => Some(board_config.player_b_spawn_cell),
        _ => None,
    }
}

/// Checks a sub-step 5 endpoint and emits a collection message when a prism is present.
///
/// Prism reward delivery and collected-state mutation are owned by the Prism System.
pub fn check_prism_collection(
    owner: PlayerId,
    final_cell: u8,
    config: &BoardConfig,
    session_config: &SessionConfig,
    lane: LaneId,
    prism_presence: &Query<(&PrismLaneKey, &PrismPresence)>,
    writer: &mut MessageWriter<PrismCollected>,
) {
    if own_prism_cell(owner, session_config, config) != Some(final_cell) {
        return;
    }

    let prism_is_present = prism_presence
        .iter()
        .any(|(key, presence)| key.player == owner && key.lane == lane && !presence.collected);

    if prism_is_present {
        writer.write(PrismCollected {
            player_id: owner,
            lane,
        });
    }
}

/// Advances all surviving standard units by their movement points.
///
/// Combat Resolution owns when sub-step 5 fires. This system only applies the
/// board movement formula to currently queryable unit entities.
pub fn apply_standard_movement(
    mut commands: Commands,
    board_config: Res<BoardConfig>,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    mut prism_collected: MessageWriter<PrismCollected>,
    prism_presence: Query<(&PrismLaneKey, &PrismPresence)>,
    mut units: Query<(Entity, &mut BoardPosition, &UnitStats, &UnitOwner)>,
) {
    for (unit_entity, mut position, stats, owner) in &mut units {
        let Some(direction) = advance_direction(owner.0, &session_config, &board_config) else {
            continue;
        };

        let destination_cell = apply_f1(
            position.cell,
            direction,
            stats.mp,
            board_config.cell_min,
            board_config.cell_max,
        );

        if destination_cell != position.cell {
            position.cell = destination_cell;
            trigger_trap_after_entry(
                &mut commands,
                &mut occupancy,
                &session_config,
                &mut trap_triggers,
                unit_entity,
                owner.0,
                position.lane,
                position.cell,
            );
        }

        check_prism_collection(
            owner.0,
            position.cell,
            &board_config,
            &session_config,
            position.lane,
            &prism_presence,
            &mut prism_collected,
        );
    }
}

/// Advances units with CHARGE X by their bonus movement amount.
///
/// Combat Resolution owns when sub-step 2 fires. This system intentionally
/// updates only the final destination cell for the sub-step; intermediate cells
/// are not represented as occupied by this movement pass.
pub fn apply_charge_movement(
    mut commands: Commands,
    board_config: Res<BoardConfig>,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    mut units: Query<(Entity, &mut BoardPosition, &ChargeBonus, &UnitOwner)>,
) {
    for (unit_entity, mut position, charge, owner) in &mut units {
        let Some(direction) = advance_direction(owner.0, &session_config, &board_config) else {
            continue;
        };

        let destination_cell = apply_f1(
            position.cell,
            direction,
            charge.0,
            board_config.cell_min,
            board_config.cell_max,
        );
        if destination_cell == position.cell {
            continue;
        }

        position.cell = destination_cell;
        trigger_trap_after_entry(
            &mut commands,
            &mut occupancy,
            &session_config,
            &mut trap_triggers,
            unit_entity,
            owner.0,
            position.lane,
            position.cell,
        );
    }
}

/// Executes queued REPEL displacements.
#[allow(clippy::too_many_arguments)]
pub fn apply_repel_displacements(
    mut commands: Commands,
    mut displacements: MessageReader<RepelDisplacement>,
    board_config: Res<BoardConfig>,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    irremovable: Query<(), With<Irremovable>>,
    mut units: Query<(&mut BoardPosition, &UnitOwner)>,
) {
    for displacement in displacements.read() {
        if irremovable.get(displacement.target).is_ok() {
            continue;
        }

        let Ok((mut position, owner)) = units.get_mut(displacement.target) else {
            continue;
        };
        let Some(destination_cell) = apply_repel(
            position.cell,
            owner.0,
            displacement.amount,
            &session_config,
            &board_config,
        ) else {
            continue;
        };
        let destination_lane = position.lane;

        let origin_cell = position.cell;
        commit_unit_destination(
            &mut commands,
            &mut occupancy,
            &session_config,
            &mut trap_triggers,
            displacement.target,
            owner.0,
            &mut position,
            destination_lane,
            destination_cell,
        );
        tracing::info!(
            owner = owner.0.0,
            lane = ?destination_lane,
            origin_cell,
            destination_cell,
            amount = displacement.amount,
            "apply_repel_displacements: REPEL committed"
        );
    }
}

/// Executes queued ATTRACT displacements.
#[allow(clippy::too_many_arguments)]
pub fn apply_attract_displacements(
    mut commands: Commands,
    mut displacements: MessageReader<AttractDisplacement>,
    board_config: Res<BoardConfig>,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    irremovable: Query<(), With<Irremovable>>,
    mut units: ParamSet<(
        Query<(&BoardPosition, &UnitOwner)>,
        Query<(&mut BoardPosition, &UnitOwner)>,
    )>,
) {
    for displacement in displacements.read() {
        if irremovable.get(displacement.target).is_ok() {
            continue;
        }

        let (caster_cell, caster_owner) = {
            let caster_positions = units.p0();
            let Ok((caster_position, caster_owner)) = caster_positions.get(displacement.caster)
            else {
                continue;
            };
            (caster_position.cell, caster_owner.0)
        };
        let mut target_positions = units.p1();
        let Ok((mut target_position, owner)) = target_positions.get_mut(displacement.target) else {
            continue;
        };
        let Some(target_is_enemy) = players_are_enemies(caster_owner, owner.0, &session_config)
        else {
            continue;
        };
        let destination_cell = apply_attract(
            caster_cell,
            target_position.cell,
            displacement.amount,
            target_is_enemy,
            &board_config,
        );
        let destination_lane = target_position.lane;
        let origin_cell = target_position.cell;

        commit_unit_destination(
            &mut commands,
            &mut occupancy,
            &session_config,
            &mut trap_triggers,
            displacement.target,
            owner.0,
            &mut target_position,
            destination_lane,
            destination_cell,
        );
        tracing::info!(
            owner = owner.0.0,
            caster_owner = caster_owner.0,
            lane = ?destination_lane,
            origin_cell,
            destination_cell,
            amount = displacement.amount,
            target_is_enemy,
            "apply_attract_displacements: ATTRACT committed"
        );
    }
}

fn players_are_enemies(
    first: PlayerId,
    second: PlayerId,
    session_config: &SessionConfig,
) -> Option<bool> {
    let first_team = session_config.team_map.get(&first).copied()?;
    let second_team = session_config.team_map.get(&second).copied()?;

    Some(first_team != second_team)
}

/// Executes queued CHANGE LANE displacements.
#[allow(clippy::too_many_arguments)]
pub fn apply_change_lane_displacements(
    mut commands: Commands,
    mut displacements: MessageReader<ChangeLaneDisplacement>,
    session_config: Res<SessionConfig>,
    board_config: Res<BoardConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    irremovable: Query<(), With<Irremovable>>,
    mut units: Query<(&mut BoardPosition, &UnitOwner)>,
) {
    for displacement in displacements.read() {
        if irremovable.get(displacement.target).is_ok() {
            continue;
        }

        let Ok((mut position, owner)) = units.get_mut(displacement.target) else {
            continue;
        };
        let original_lane = position.lane;
        let destination_lane = apply_change_lane(
            original_lane,
            displacement.delta,
            &occupancy,
            owner.0,
            &board_config,
        );
        if destination_lane == original_lane {
            continue;
        }

        let destination_cell = position.cell;
        commit_unit_destination(
            &mut commands,
            &mut occupancy,
            &session_config,
            &mut trap_triggers,
            displacement.target,
            owner.0,
            &mut position,
            destination_lane,
            destination_cell,
        );
        move_minion_slot(
            &mut occupancy,
            owner.0,
            original_lane,
            destination_lane,
            displacement.target,
        );
        tracing::info!(
            owner = owner.0.0,
            origin_lane = ?original_lane,
            destination_lane = ?destination_lane,
            cell = destination_cell,
            delta = displacement.delta,
            "apply_change_lane_displacements: CHANGE_LANE committed"
        );
    }
}

/// Commits same-cell lane movement in deterministic original-lane order.
///
/// Keyword/card logic owns deciding which units should CHANGE LANE. Board owns
/// the position commit and Trap trigger side effect once destinations are known.
pub fn commit_lane_change_destinations(
    mut commands: Commands,
    session_config: Res<SessionConfig>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut trap_triggers: MessageWriter<TrapTrigger>,
    mut units: Query<(
        Entity,
        &mut BoardPosition,
        &UnitOwner,
        &LaneChangeDestination,
    )>,
) {
    let mut changes = units
        .iter_mut()
        .map(|(unit_entity, position, owner, lane_change)| {
            (
                unit_entity,
                owner.0,
                lane_change.original_lane,
                lane_change.destination_lane,
                position.cell,
            )
        })
        .collect::<Vec<_>>();
    changes
        .sort_by_key(|(unit_entity, _, original_lane, _, _)| (*original_lane, unit_entity.index()));

    for (unit_entity, unit_owner, _, destination_lane, destination_cell) in changes {
        let Ok((_, mut position, _, _)) = units.get_mut(unit_entity) else {
            continue;
        };

        commit_unit_destination(
            &mut commands,
            &mut occupancy,
            &session_config,
            &mut trap_triggers,
            unit_entity,
            unit_owner,
            &mut *position,
            destination_lane,
            destination_cell,
        );
    }
}

/// Commits a final board destination and fires any enemy Trap at that cell.
pub fn commit_unit_destination(
    commands: &mut Commands,
    occupancy: &mut BoardOccupancy,
    session_config: &SessionConfig,
    trap_triggers: &mut MessageWriter<TrapTrigger>,
    unit_entity: Entity,
    unit_owner: PlayerId,
    position: &mut BoardPosition,
    destination_lane: LaneId,
    destination_cell: u8,
) {
    if position.lane == destination_lane && position.cell == destination_cell {
        return;
    }

    position.lane = destination_lane;
    position.cell = destination_cell;
    trigger_trap_after_entry(
        commands,
        occupancy,
        session_config,
        trap_triggers,
        unit_entity,
        unit_owner,
        position.lane,
        position.cell,
    );
}

/// Pre-resolved lane movement destination owned by board movement commit code.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaneChangeDestination {
    pub original_lane: LaneId,
    pub destination_lane: LaneId,
}

#[allow(clippy::too_many_arguments)]
fn trigger_trap_after_entry(
    commands: &mut Commands,
    occupancy: &mut BoardOccupancy,
    session_config: &SessionConfig,
    trap_triggers: &mut MessageWriter<TrapTrigger>,
    unit_entity: Entity,
    unit_owner: PlayerId,
    lane: LaneId,
    cell: u8,
) {
    let Some((trap_owner, trap_entity)) =
        check_trap_trigger(occupancy, session_config, unit_owner, lane, cell)
    else {
        return;
    };

    occupancy.traps.remove(&(trap_owner, lane, cell));
    commands.entity(trap_entity).despawn();
    trap_triggers.write(TrapTrigger {
        trap_entity,
        unit_entity,
        trap_owner,
        unit_owner,
        lane,
        cell,
    });
}

fn move_minion_slot(
    occupancy: &mut BoardOccupancy,
    unit_owner: PlayerId,
    original_lane: LaneId,
    destination_lane: LaneId,
    unit_entity: Entity,
) {
    if occupancy
        .minion_slots
        .get(&(unit_owner, original_lane))
        .is_some_and(|entity| *entity == unit_entity)
    {
        occupancy.minion_slots.remove(&(unit_owner, original_lane));
    }
    occupancy
        .minion_slots
        .insert((unit_owner, destination_lane), unit_entity);
}
