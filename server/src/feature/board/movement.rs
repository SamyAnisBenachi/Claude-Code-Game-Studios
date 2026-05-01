use bevy::prelude::{Query, Res};
use shared::session::PlayerId;

use crate::core::board::{BoardPosition, UnitOwner, UnitStats};
use crate::core::session::SessionConfig;
use crate::feature::board::BoardConfig;

const PLAYER_A_TEAM_ID: u8 = 0;
const PLAYER_B_TEAM_ID: u8 = 1;

/// Applies Formula F1 to compute the new cell after one movement sub-step.
pub fn apply_f1(current_cell: u8, direction: i16, mp: u8, cell_min: u8, cell_max: u8) -> u8 {
    let new_cell = current_cell as i16 + direction * mp as i16;
    new_cell.clamp(cell_min as i16, cell_max as i16) as u8
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

/// Advances all surviving standard units by their movement points.
///
/// Combat Resolution owns when sub-step 5 fires. This system only applies the
/// board movement formula to currently queryable unit entities.
pub fn apply_standard_movement(
    board_config: Res<BoardConfig>,
    session_config: Res<SessionConfig>,
    mut units: Query<(&mut BoardPosition, &UnitStats, &UnitOwner)>,
) {
    for (mut position, stats, owner) in &mut units {
        let Some(direction) = advance_direction(owner.0, &session_config, &board_config) else {
            continue;
        };

        position.cell = apply_f1(
            position.cell,
            direction,
            stats.mp,
            board_config.cell_min,
            board_config.cell_max,
        );
    }
}
