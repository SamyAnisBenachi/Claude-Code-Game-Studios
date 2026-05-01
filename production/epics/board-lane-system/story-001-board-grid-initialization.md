# Story 001: Board Grid Initialization

> **Epic**: Board / Lane System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: Placements are buffered as plain Rust data in `PendingPlacements` (not ECS entities) until `S2CPlacementReveal` is enqueued; this ADR also defines the core board data structures (`BoardGrid`, `BoardOccupancy`, `SpawnRangeState`) that all downstream stories depend on.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `Commands::spawn()` without Bundle uses Bevy 0.15+ Required Components API. `#[derive(Resource, Default)]` required for all board resources. No Bundles. Verify `app.init_resource::<T>()` is the correct init pattern in 0.18.

**Control Manifest Rules (this layer)**:
- Required: `server/src/feature/board/` is the home for all board data structures — `feature/ → core/ → foundation/` dependency direction only
- Required: No game state on client — `BoardGrid` and all occupancy resources are server-only (`server/` crate)
- Forbidden: Never derive `Resource` in the `shared/` crate
- Guardrail: Board data structures are persistent resources, not per-frame query hot paths — no performance concern at this layer

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [x] **BL-21**: GIVEN a game initialises, WHEN the board is set up, THEN each player has exactly 5 Minion slots (one per lane), all initially empty.
- [x] **NEW-001a**: GIVEN the server starts a game session, WHEN the board is initialized, THEN a `BoardGrid` resource exists with exactly 5 lanes and 8 cells per lane (cells numbered 1–8 inclusive, not 0-indexed).
- [x] **NEW-001b**: GIVEN the board is initialized, WHEN `BoardConfig` is queried, THEN `player_a_direction = +1i16` and `player_b_direction = -1i16` are accessible for use by the F1 movement formula.

---

## Implementation Notes

*Derived from ADR-007 Implementation Guidelines and GDD Rule 1, Rule 2:*

Define the following resources in `server/src/feature/board/`:

```rust
// BoardGrid: the sole authoritative spatial state — GDD Rule 1/2, TR-BLS-001
// Cell index 0 = absolute cell 1; index 7 = absolute cell 8
#[derive(Resource, Default)]
pub struct BoardGrid {
    pub lanes: [[Option<BoardCell>; 8]; 5],
}

// BoardOccupancy: tracks what each player has placed per lane/cell
// Queried by spawn range validation (Story 003) and occupancy checks (Story 004)
#[derive(Resource, Default)]
pub struct BoardOccupancy {
    // per player: which lane has their Minion (None = empty slot)
    pub minion_slots: [[Option<Entity>; 5]; 2],  // [player_index][lane_index]
    // per player: which (lane, cell) cells have Traps/Structures
    pub traps: HashMap<(PlayerId, LaneId, u8), Entity>,
    pub structures: HashMap<(PlayerId, LaneId, u8), Entity>,
    // per player: which lanes have Fields
    pub fields: HashMap<(PlayerId, LaneId), Entity>,
}

// SpawnRangeState: tracks fakes destroyed per player — drives F2 validation (Story 003)
#[derive(Resource, Default)]
pub struct SpawnRangeState {
    pub fakes_destroyed: [u8; 2],  // [player_index]: 0–2
}

// PrismState: tracks whether each player's prism is present per lane (Story 009)
#[derive(Resource)]
pub struct PrismState {
    pub present: [[bool; 5]; 2],  // [player_index][lane_index]: true = prism uncollected
}

impl Default for PrismState {
    fn default() -> Self {
        Self { present: [[true; 5]; 2] }  // all prisms present at game start
    }
}

// BoardConfig: structural constants — not from GameConfig (see ADR-007 NOTE)
// spawn_cell_A = 1 and spawn_cell_B = 8 are physical board constants, not tuning knobs
#[derive(Resource)]
pub struct BoardConfig {
    pub player_a_direction: i16,   // +1
    pub player_b_direction: i16,   // -1
    pub player_a_spawn_cell: u8,   // 1
    pub player_b_spawn_cell: u8,   // 8
    pub player_a_objective_cell: u8, // 8
    pub player_b_objective_cell: u8, // 1
    pub lane_count: u8,            // 5
    pub cells_per_lane: u8,        // 8
    pub cell_min: u8,              // 1
    pub cell_max: u8,              // 8
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            player_a_direction: 1,
            player_b_direction: -1,
            player_a_spawn_cell: 1,
            player_b_spawn_cell: 8,
            player_a_objective_cell: 8,
            player_b_objective_cell: 1,
            lane_count: 5,
            cells_per_lane: 8,
            cell_min: 1,
            cell_max: 8,
        }
    }
}
```

Register all resources via `app.init_resource::<T>()` in `BoardPlugin::build()`. `BoardConfig` is inserted via `app.insert_resource(BoardConfig::default())`.

The `liv-bevy-018` skill is mandatory on all `.rs` files in `server/src/feature/board/`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: Movement formula using `BoardConfig` direction constants
- Story 003: Spawn range validation using `SpawnRangeState`
- Story 004: Occupancy enforcement using `BoardOccupancy`
- Story 005: `PendingPlacements` resource (placement buffer)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **NEW-001a**: Board has exactly 5 lanes and 8 cells per lane
  - Given: `World::new()` with `BoardGrid::default()` inserted as resource
  - When: Query `Res<BoardGrid>` and inspect `lanes` field
  - Then: `lanes.len() == 5`; each inner array has length `8`
  - Edge cases: All cells initially `None`

- **NEW-001b**: BoardConfig direction constants
  - Given: `World::new()` with `BoardConfig::default()` inserted
  - When: Query `Res<BoardConfig>`
  - Then: `player_a_direction == 1i16`; `player_b_direction == -1i16`
  - Edge cases: `cell_min == 1`; `cell_max == 8`; `lane_count == 5`

- **BL-21**: Minion slots all initially empty
  - Given: `World::new()` with `BoardOccupancy::default()` inserted
  - When: Query `Res<BoardOccupancy>` and inspect `minion_slots`
  - Then: `minion_slots[0]` (PlayerA) has all 5 entries `None`; same for `minion_slots[1]` (PlayerB)
  - Edge cases: `traps` HashMap empty; `structures` HashMap empty; `fields` HashMap empty

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/board_grid_initialization_test.rs` — must exist and pass

**Status**: [x] Created and passed locally with `cargo test -p server --test board_grid_initialization_test`

---

## Dependencies

- Depends on: None
- Unlocks: Stories 002, 003, 004 (all require BoardConfig and BoardOccupancy resources)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 3/3 passing. BL-21, NEW-001a, and NEW-001b are covered by `tests/unit/board-lane-system/board_grid_initialization_test.rs`.
**Deviations**: Advisory only: story manifest v2026-04-29 is older than current control manifest v2026-05-01. Advisory only: current `TR-BLS-001` registry text maps primarily to the grid and direction requirements; BL-21 minion-slot initialization remains covered by the GDD acceptance criteria and story-scoped unit test.
**Test Evidence**: Logic: `tests/unit/board-lane-system/board_grid_initialization_test.rs`; `cargo test -p server --test board_grid_initialization_test` passed 4/4.
**Code Review**: Skipped - Lean mode.
