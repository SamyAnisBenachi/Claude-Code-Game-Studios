pub const Z_BOARD_BACKGROUND: f32 = -0.5;
pub const Z_FIELD_WASH: f32 = 0.0;
// PROMPT 1489 — Krosmaga-style lane surface + rails sit above the field wash
// but below the cell nodes so the cell-affordance dots remain visible on top
// of the lane painting and the wash overlay can still darken the lanes.
pub const Z_LANE_SURFACE: f32 = 0.45;
pub const Z_LANE_RAILS: f32 = 0.55;
pub const Z_CELL_NODES: f32 = 1.0;
pub const Z_BOARD_CHROME: f32 = 1.5;
// PROMPT 1390 (S19-BR-PLAYAREA-HIERARCHY-TARGETING-FEEDBACK-001):
// Targeting overlay surfaces sit between the board chrome and the
// traps/structures layer so the dim wash darkens cell-node + chrome art
// while leaving objectives, units, HP bars, and ghosts visually on top.
// ADR-021 z-order is preserved: every targeting overlay stays below
// Z_TRAPS_STRUCTURES and therefore below objectives / units / hover cards.
pub const Z_TARGETING_DIM_WASH: f32 = 1.55;
pub const Z_TARGETING_VALID_RING: f32 = 1.6;
pub const Z_TARGETING_ENDPOINT_RING: f32 = 1.7;
pub const Z_TARGETING_INVALID_MARKER: f32 = 1.7;
pub const Z_TRAPS_STRUCTURES: f32 = 2.0;
pub const Z_OBJECTIVES: f32 = 2.5;
pub const Z_UNITS: f32 = 3.0;
pub const Z_HEALTH_BARS: f32 = 3.1;
// PROMPT 1390 — the source-card link sits above units but below ghosts
// so a placement ghost preview never gets occluded by the link.
pub const Z_SOURCE_CARD_LINK: f32 = 3.3;
pub const Z_GHOST_UNIT: f32 = 3.5;
pub const Z_GRID_OVERLAY: f32 = 3.6;

pub const Z_BOARD_CAMERA: f32 = 999.0;
pub const CELL_NODE_SIZE: f32 = 28.0;
pub const LANE_RAIL_THICKNESS: f32 = 3.0;
pub const UNIT_SPRITE_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(48.0, 64.0);
// PROMPT 1489 — Krosmaga-style footing shadow anchors each unit to its cell.
// The footing is a non-pickable child sprite tucked just below the unit body
// and below it on local-Z so the unit art stays on top.
pub const UNIT_FOOTING_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(42.0, 10.0);
pub const UNIT_FOOTING_Y_OFFSET: f32 = -27.0;
pub const UNIT_FOOTING_LOCAL_Z: f32 = -0.08;
pub const OBJECTIVE_SPRITE_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(64.0, 96.0);
pub const HP_BAR_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(42.0, 5.0);
pub const HP_BAR_Y_OFFSET: f32 = 38.0;
pub const HEALTH_BAR_LOCAL_Z: f32 = Z_HEALTH_BARS - Z_UNITS;
pub const STATUS_ICON_SIZE: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(16.0, 16.0);
pub const STATUS_ICON_TOP_RIGHT_X_OFFSET: f32 = 16.0;
pub const STATUS_ICON_TOP_RIGHT_Y_OFFSET: f32 = 24.0;
pub const STATUS_ICON_SLOT_STEP_X: f32 = 16.0;
pub const STATUS_ICON_LOCAL_Z: f32 = 0.05;
