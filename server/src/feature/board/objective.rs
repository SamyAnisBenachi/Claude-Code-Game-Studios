use bevy::prelude::{Entity, Message, MessageWriter, Query, Res, With};
use shared::session::PlayerId;

use crate::core::board::{BoardPosition, UnitOwner, UnitStats};
use crate::core::session::SessionConfig;
use crate::feature::board::{BoardConfig, LaneId};

const PLAYER_A_TEAM_ID: u8 = 0;
const PLAYER_B_TEAM_ID: u8 = 1;

/// Internal board signal emitted when a unit is at its owner's objective cell.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitAtObjective {
    pub unit_id: Entity,
    pub lane: LaneId,
}

/// Returns true when `owner` is at that player's objective cell.
pub fn is_at_objective(
    owner: PlayerId,
    cell: u8,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> bool {
    objective_cell_for(owner, session_config, board_config)
        .is_some_and(|objective| cell == objective)
}

/// Emits `UnitAtObjective` once for each unit currently on its objective cell.
pub fn detect_objective_presence(
    board_config: Res<BoardConfig>,
    session_config: Res<SessionConfig>,
    mut objective_hits: MessageWriter<UnitAtObjective>,
    units: Query<(Entity, &BoardPosition, &UnitOwner), With<UnitStats>>,
) {
    for (unit_id, position, owner) in &units {
        if is_at_objective(owner.0, position.cell, &session_config, &board_config) {
            objective_hits.write(UnitAtObjective {
                unit_id,
                lane: position.lane,
            });
        }
    }
}

fn objective_cell_for(
    owner: PlayerId,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> Option<u8> {
    match session_config.team_map.get(&owner).copied() {
        Some(PLAYER_A_TEAM_ID) => Some(board_config.player_a_objective_cell),
        Some(PLAYER_B_TEAM_ID) => Some(board_config.player_b_objective_cell),
        _ => None,
    }
}
