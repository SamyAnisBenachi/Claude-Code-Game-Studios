use bevy::prelude::*;
use shared::session::PlayerId;

pub const BOARD_LANE_COUNT: u8 = 5;
pub const BOARD_CELL_COUNT: u8 = 8;

/// Canonical board coordinate model shared by presentation systems.
///
/// Board Rendering owns insertion/removal of this session-scoped resource. HUD
/// reads it only to project objective lanes into scoreboard dot positions.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BoardLayout {
    pub board_origin: Vec2,
    pub cell_width: f32,
    pub lane_height: f32,
}

impl Default for BoardLayout {
    fn default() -> Self {
        Self {
            board_origin: Vec2::ZERO,
            cell_width: 64.0,
            lane_height: 80.0,
        }
    }
}

impl BoardLayout {
    pub fn cell_to_world(&self, lane: u8, cell: u8) -> Option<Vec2> {
        if !(1..=BOARD_LANE_COUNT).contains(&lane) || !(1..=BOARD_CELL_COUNT).contains(&cell) {
            return None;
        }

        Some(Vec2 {
            x: self.board_origin.x + f32::from(cell - 1) * self.cell_width,
            y: self.board_origin.y - f32::from(lane - 1) * self.lane_height,
        })
    }

    pub fn scoreboard_lane_center_x(&self, lane: u8) -> Option<f32> {
        if !(1..=BOARD_LANE_COUNT).contains(&lane) {
            return None;
        }

        Some(self.board_origin.x + f32::from(lane - 1) * self.lane_height)
    }
}

/// Presentation-side committed board cell marker.
///
/// Board Rendering owns updates to this marker when entities are spawned,
/// rebuilt, or moved. Card Animations reads it only to snap visuals back to the
/// committed cell during PLACEMENT animation cancellation.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneCell {
    pub lane: u8,
    pub cell: u8,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudObjectiveUpdate {
    pub target_player_id: PlayerId,
    pub lane: u8,
}
