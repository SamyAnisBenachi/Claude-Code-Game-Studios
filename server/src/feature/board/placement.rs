use shared::card::CardType;
use shared::session::PlayerId;

use crate::core::session::SessionConfig;
use crate::feature::board::BoardConfig;

const PLAYER_A_TEAM_ID: u8 = 0;
const PLAYER_B_TEAM_ID: u8 = 1;
const MAX_FAKE_OBJECTIVES_DESTROYED: u8 = 2;

/// Returns whether a card type must pass spawn-range validation.
///
/// GDD Formula F2 applies to Minion placements only. Structures and Traps
/// bypass spawn range and continue to later placement checks.
pub fn requires_spawn_range_validation(card_type: CardType) -> bool {
    matches!(card_type, CardType::Minion)
}

/// Implements GDD Formula F2 for Minion placement cells.
///
/// The caller supplies `fakes_destroyed` from `SpawnRangeState` at validation
/// time. Unknown players or invalid team assignments reject silently.
pub fn validate_spawn_range(
    target_cell: u8,
    player: PlayerId,
    fakes_destroyed: u8,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> bool {
    let expansion = fakes_destroyed.min(MAX_FAKE_OBJECTIVES_DESTROYED);

    match session_config.team_map.get(&player).copied() {
        Some(PLAYER_A_TEAM_ID) => {
            let min_cell = board_config.player_a_spawn_cell;
            let max_cell = min_cell
                .saturating_add(expansion)
                .min(board_config.cell_max);

            target_cell >= min_cell && target_cell <= max_cell
        }
        Some(PLAYER_B_TEAM_ID) => {
            let max_cell = board_config.player_b_spawn_cell;
            let min_cell = max_cell
                .saturating_sub(expansion)
                .max(board_config.cell_min);

            target_cell >= min_cell && target_cell <= max_cell
        }
        _ => false,
    }
}
