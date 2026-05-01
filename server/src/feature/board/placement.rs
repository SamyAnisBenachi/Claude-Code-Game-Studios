use shared::card::CardType;
use shared::session::PlayerId;

use crate::core::session::SessionConfig;
use crate::feature::board::{BoardConfig, BoardOccupancy, LaneId};

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

/// Returns whether `player` has an open Minion slot in `lane`.
///
/// Occupancy is per-player. Team capacity is derived from the current session's
/// team map so 1v1 has one slot per team lane and 2v2 has two.
pub fn is_minion_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    session_config: &SessionConfig,
) -> bool {
    let Some(team) = session_config.team_map.get(&player).copied() else {
        return false;
    };

    if occupancy.minion_slots.contains_key(&(player, lane)) {
        return false;
    }

    let team_capacity = session_config
        .team_map
        .values()
        .filter(|candidate| **candidate == team)
        .count();

    let team_count = occupancy
        .minion_slots
        .keys()
        .filter(|(slot_player, slot_lane)| {
            *slot_lane == lane && session_config.team_map.get(slot_player).copied() == Some(team)
        })
        .count();

    team_count < team_capacity
}

/// Returns whether `player` may place a Trap at `(lane, cell)`.
pub fn is_trap_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
) -> bool {
    !occupancy.traps.contains_key(&(player, lane, cell))
}

/// Returns whether `player` may place a Structure at `(lane, cell)`.
pub fn is_structure_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
) -> bool {
    !occupancy.structures.contains_key(&(player, lane, cell))
}

/// Returns whether `player` may place a Field in `lane`.
pub fn is_field_slot_available(occupancy: &BoardOccupancy, player: PlayerId, lane: LaneId) -> bool {
    !occupancy.fields.contains_key(&(player, lane))
}
