# ADR-020: Board/Lane System State Architecture

## Status

Accepted

## Date

2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / Gameplay / Networking |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 and Lightyear 0.26 all post-cutoff |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/engine-reference/bevy/breaking-changes.md`, `docs/engine-reference/bevy/deprecated-apis.md`, ADR-007 (placement buffer), ADR-017 (combat resolution exclusive system), ADR-018 (UnitKeywordState component) |
| **Post-Cutoff APIs Used** | Required Components API (Bevy 0.15+ — replaces deprecated bundles); `commands.spawn((ComponentA, ComponentB))` without Bundle; `world.query::<Q>()` in exclusive system context; Lightyear 0.26 `Replicate::to_clients(NetworkTarget)` component for entity replication scope |
| **Post-Cutoff APIs NOT Used** | `SpriteBundle`, any `*Bundle` type — all deprecated in Bevy 0.15+. Do not spawn unit entities using Bundle structs. |
| **Verification Required** | (1) **VERIFIED 2026-04-30**: `ReplicateTo` does NOT exist in Lightyear 0.26.0. The correct API is `Replicate::to_clients(NetworkTarget::All)` — a `Replicate` component (in `lightyear_replication::send::components`) using `ReplicationMode::SingleServer(target)`. Adding this component also auto-inserts `Replicating` and `ReplicationGroup` via Bevy Required Components. All ADR references to `ReplicateTo` have been updated to `Replicate::to_clients(NetworkTarget::All)`. Verified against `lightyear_replication/src/send/components.rs` at tag 0.26.0. (2) CONFIRMED: `world.query::<Q>()` in exclusive system (`fn resolve_combat(world: &mut World)`) is safe in Bevy 0.18 — no `unsafe` required. (3) Confirm `world.resource_mut::<BoardState>()` conflicts with any regular system holding `ResMut<BoardState>` are resolved by the exclusive system running outside the regular Update schedule. (4) ADVISORY: The `advance_phase` symbol referenced in `BoardPlugin` (`.after(advance_phase)`) must resolve to a concrete system identifier from the RSM module — confirm it is exported from `server::core::rsm`. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (server authority — all board state is server-only; clients receive replicated view); ADR-003 (workspace layout — board module lives in `server/src/feature/board/`); ADR-007 (placement buffer — defines `PlacementBuffer` resource; this ADR defines what happens after the buffer commits); ADR-009 (RSM phase events — `PlacementPhaseEntered` opens the placement window; `ResolutionPhaseEntered` triggers sub-step 1 commit); ADR-017 (combat resolution — `resolve_combat` exclusive system is the primary caller of board API functions); ADR-018 (keyword system — `UnitKeywordState` component lives on unit entities) |
| **Enables** | Combat Resolution implementation (resolve_combat can now call board API functions with defined signatures); Objective System implementation (reads board for unit-at-objective state); Prism System (board emits PrismCollected events during sub-step 5); Board Rendering (clients receive replicated `BoardPosition` components from unit entities and protocol-delivered spawn range); Spawn-range validation during PLACEMENT (reads `SpawnRangeState`) |
| **Blocks** | Any story implementing PLACEMENT validation, board movement, unit death, or RESOLUTION sub-steps 1–6. No combat or placement story can be implemented until this ADR is Accepted. |
| **Ordering Note** | ADR-007 (placement buffer) is Accepted — the `PlacementBuffer` resource and its commit-at-RESOLUTION-start semantics are fixed. This ADR picks up where ADR-007 ends: at the moment sub-step 1 commits the buffer into the live board. ADR-017 (combat resolution) must also be Accepted before the board API calling conventions can be verified. |

## Context

### Problem Statement

The Board/Lane System owns the authoritative 5-lane × 8-cell grid, including all unit positions, occupancy slots (Minion slot per player per lane), spawn range per player, and prism state. Combat Resolution, Keyword System, and Objective System must all query and mutate this data during RESOLUTION sub-steps. Board Rendering on clients must receive unit positions through Lightyear's component replication.

Two competing requirements must be satisfied simultaneously:
- **Fast spatial queries** during RESOLUTION: `resolve_combat` (exclusive system) needs O(1) lookup of "what entities are at (lane, cell)?". Bevy's `world.query()` with `&BoardPosition` is O(n) over all units — too slow if units are queried repeatedly across 6 sub-steps with up to 10 alive units across 5 lanes.
- **Client replication**: Lightyear replicates ECS entities with their components to clients. Unit data (position, HP, ATK, MP) must live on ECS entity components to participate in this replication.

Neither a pure-ECS approach (no index, slow spatial queries) nor a pure-resource approach (fast index, no Lightyear replication integration) satisfies both requirements. A hybrid approach — ECS entities for units plus a `BoardState` index resource — is required.

A secondary decision is spawn range state: the GDD (Formula F2) requires a per-player live projection derived from fake objectives destroyed (0–2) to validate Minion placement, build reconnect snapshots, and emit live client updates. This state must persist between rounds and be readable during PLACEMENT validation. Objective System owns the destruction facts/counters; Board/Lane owns the live projection.

### Constraints

- **Required Components API** (Bevy 0.15+): Bundles are deprecated. Unit entities must be spawned using `commands.spawn((ComponentA, ComponentB, ...))` or equivalent, not with Bundle structs.
- **Exclusive system world access** (ADR-017): `resolve_combat` receives `world: &mut World`. Board API functions called from within it use `world.query()` for entity access and `world.resource_mut::<BoardState>()` for index access.
- **Lightyear replication at entity granularity** (ADR-007, ADR-001): There is no per-component replication scope in Lightyear 0.26. Units added to a replication group are visible to all clients for all replicated components. Units must NOT be added to any replication group until after `S2CPlacementReveal` is enqueued (sub-step 1 commit — per ADR-007).
- **BoardIndex update discipline**: The `BoardState` spatial index must remain consistent with entity component state at all times. All board mutations (spawn, move, remove) go through board API functions — direct component mutation outside the board API is forbidden.
- **Feature layer module**: The Board/Lane System is a Feature-layer system (it depends on Card Data, Economy, and RSM events). Its module lives in `server/src/feature/board/`, not `server/src/core/`.

### Requirements

- After sub-step 1 commit, the board holds live unit entities with position components and a synchronized spatial index.
- `get_units_at_cell(lane, cell)` returns all entities at that position in O(1).
- Minion slot occupancy is tracked per player per lane; validation runs during PLACEMENT buffer acceptance.
- `SpawnRangeState` per player persists across rounds, is readable during PLACEMENT, is the snapshot source for `PlayerSnapshot.spawn_range_cells`, and is the source for `ResolutionEvent::SpawnRangeChanged`.
- Board API functions are pure Rust module functions callable from `resolve_combat` without a Bevy `App` — testable with `World::new()`.
- Unit entities added to the Lightyear replication group broadcast `BoardPosition`, `UnitHp`, and other display components to all clients.

## Decision

The Board/Lane System stores authoritative live board state in two layers:

1. **Bevy ECS entities** — one entity per alive unit. Carries display-ready components for Lightyear replication. Added to the replication group at sub-step 1 commit (per ADR-007).

2. **`BoardState` resource** — a plain Rust resource holding:
   - Spatial index: `HashMap<(u8, u8), Vec<Entity>>` — maps (lane, cell) to all entities at that position.
   - Minion slots: `HashMap<(PlayerId, u8), Option<Entity>>` — maps (player, lane) to the Minion entity currently occupying that lane's Minion slot, if any.

3. **`SpawnRangeState` resource** — a Board/Lane-owned resource holding the live authoritative per-player spawn range projection. It is intentionally separate from Objective System `ObjectiveCounters`: Objective owns facts; Board/Lane owns projection. It is also intentionally not a Lightyear replicated component: live transport is `ResolutionEvent::SpawnRangeChanged` inside `S2CResolutionEvent`; recovery transport is `PlayerSnapshot.spawn_range_cells`.

All mutations to board state — spawning, moving, removing units, and spawn range projection — go through functions in `server/src/feature/board/api.rs`. These functions update both the entity components and `BoardState` index atomically for spatial mutations, and update `SpawnRangeState` for spawn range mutations. No code outside `board/api.rs` may mutate `BoardPosition` components, `BoardState` index entries, or `SpawnRangeState` values directly.

### Architecture Diagram

```
SERVER WORLD
┌──────────────────────────────────────────────────────────────────┐
│  BoardState (Resource — server only)                             │
│  ├── position_index: HashMap<(lane: u8, cell: u8), Vec<Entity>>  │
│  │     Keys: (1..=5, 1..=8)                                      │
│  │     Values: All unit entities currently at that (lane, cell)  │
│  │     Updated atomically by board/api.rs on every spawn/move    │
│  │                                                               │
│  ├── minion_slots: HashMap<(PlayerId, lane: u8), Option<Entity>> │
│  │     Keys: (Player A|B, 1..=5)                                 │
│  │     Values: Entity of occupying Minion, or None               │
│  │     Cleared in board/api.rs when Minion is removed            │
│  │                                                               │
│  └── no spawn range fields here; see SpawnRangeState below       │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  SpawnRangeState (Resource - server only, Board/Lane owned)      │
│  └── fakes_destroyed: [u8; 2]    // [player_a, player_b]          │
│        Range: 0-2 per player                                     │
│        Updated by Board/Lane from Objective destruction facts    │
│        Read during PLACEMENT validation (Formula F2)             │
│        Source for PlayerSnapshot.spawn_range_cells               │
│        Source for ResolutionEvent::SpawnRangeChanged             │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  Unit ECS Entities  (server-authoritative; Lightyear-replicated) │
│  Spawned at sub-step 1 commit (PlacementBuffer → BoardState)     │
│                                                                  │
│  Required Components on each unit entity:                        │
│  ├── BoardPosition { lane: u8, cell: u8 }   [replicated]         │
│  ├── UnitOwner(PlayerId)                     [replicated]         │
│  ├── UnitCardRef(CardId)                     [replicated]         │
│  ├── UnitType(CardType)  // Minion/Trap/Structure/Field [rep'd]  │
│  ├── UnitStats { atk: u8, mp: u8, ar: u8 }  [replicated]        │
│  ├── CurrentHp(i32)                          [replicated]         │
│  ├── UnitKeywordState     (ADR-018)          [replicated]         │
│  └── Replicate::to_clients(NetworkTarget::All)  // Lightyear  │
│       Added at sub-step 1 commit — NOT at spawn of buffer        │
└──────────────────────────────────────────────────────────────────┘

Board/Lane API — Called from resolve_combat (exclusive system) and board tick systems
┌──────────────────────────────────────────────────────────────────┐
│  board/api.rs  (pure Rust fns, no Bevy system params)            │
│                                                                  │
│  Spatial queries (read-only, take &BoardState):                  │
│    get_units_at_cell(state, lane, cell) -> &[Entity]             │
│    get_units_at_objective_cell(state, player) -> &[Entity]       │
│    is_minion_slot_occupied(state, player, lane) -> bool          │
│    validate_placement(state, spawn_ranges, player, card_type,    │
│                       lane, cell)                                │
│        -> Result<(), PlacementError>                             │
│                                                                  │
│  Mutations (take &mut World or (&mut BoardState, EntityMut)):     │
│    spawn_unit(world, lane, cell, owner, card_id, stats,          │
│               keywords) -> Entity                                │
│    move_unit(state, commands, entity, new_lane, new_cell)        │
│    remove_unit(state, commands, entity)                          │
│    change_lane_unit(state, commands, entity, new_lane)           │
│    expand_spawn_range(spawn_ranges, player) // fake destroyed    │
│    clear_board(state, commands)          // OnResolutionEnd       │
└──────────────────────────────────────────────────────────────────┘

WRITE ACCESS RULES
┌──────────────────────────────────────────────────────────────────┐
│  PlacementBuffer commit system (PlacementPhaseEntered subscriber)│
│    → calls spawn_unit per committed entry                        │
│    → adds Replicate after S2CPlacementReveal is enqueued         │
│                                                                  │
│  resolve_combat (exclusive system — ADR-017)                     │
│    → calls move_unit, remove_unit, change_lane_unit              │
│    → calls get_units_at_cell, get_units_at_objective_cell        │
│    → emits PrismCollected when unit ends sub-step 5 at prism cell│
│                                                                  │
│  Board/Lane spawn range projection path                          │
│    → consumes Objective System fake-destruction fact             │
│    → calls expand_spawn_range(&mut SpawnRangeState, player)      │
│    → appends SpawnRangeChanged after ObjectiveDestroyed          │
│                                                                  │
│  Board cleanup system (OnResolutionEnd subscriber)               │
│    → sweeps 0-HP units, clears orphaned Minion slots             │
└──────────────────────────────────────────────────────────────────┘

FRAME SEQUENCE — PLACEMENT → RESOLUTION sub-step 1
┌──────────────────────────────────────────────────────────────────┐
│  PlacementPhaseEntered fires:                                    │
│    board tick system opens PlacementBuffer (per ADR-007)         │
│                                                                  │
│  C2SSubmitPlacement received (during PLACEMENT phase):           │
│    validate_placement against BoardState + PlacementBuffer       │
│    → deduct mana (Economy API)                                   │
│    → write to PlacementBuffer (no entity spawn yet)             │
│                                                                  │
│  BeginResolution fires → resolve_combat exclusive runs:          │
│    Sub-step 1: commit PlacementBuffer                            │
│      for each pending placement:                                 │
│        entity = spawn_unit(world, lane, cell, owner, ...)        │
│        (entity has all components EXCEPT Replicate)              │
│      enqueue S2CPlacementReveal on ReliableChannel               │
│      add Replicate::to_clients(NetworkTarget::All) to each spawned entity                      │
│      clear PlacementBuffer                                       │
│                                                                  │
│    Sub-steps 2–6: combat resolution (resolve_combat body)        │
│      uses get_units_at_cell, move_unit, remove_unit, etc.        │
│                                                                  │
│  OnResolutionEnd fires:                                          │
│    board cleanup system: sweep 0-HP, clear orphaned slots        │
└──────────────────────────────────────────────────────────────────┘
```

### Key Interfaces

```rust
// server/src/feature/board/state.rs

use bevy::prelude::*;
use shared::session::PlayerId;
use shared::protocol::CardId;
use std::collections::HashMap;

/// Authoritative spatial index and occupancy tracking for the live board.
/// All spatial state reads/writes go through board/api.rs.
/// Direct HashMap entry manipulation outside api.rs is forbidden.
#[derive(Resource, Default)]
pub struct BoardState {
    /// (lane 1-5, cell 1-8) → all unit entities at that position.
    pub position_index: HashMap<(u8, u8), Vec<Entity>>,
    /// (owner PlayerId, lane 1-5) → the occupying Minion entity, or None.
    pub minion_slots: HashMap<(PlayerId, u8), Option<Entity>>,
}

/// Authoritative Board/Lane-owned live spawn range projection.
/// ObjectiveCounters owns destruction facts; this resource owns placement range.
#[derive(Resource, Default)]
pub struct SpawnRangeState {
    /// Fakes destroyed per player: [player_a_count, player_b_count]. Range 0–2.
    pub fakes_destroyed: [u8; 2],
}

/// Placement error variants for server-side placement validation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlacementError {
    /// Lane's Minion slot already occupied by this player.
    MinionSlotOccupied,
    /// Cell is outside the player's spawn range (Formula F2).
    OutOfSpawnRange,
    /// Cell already has a Trap/Structure belonging to this player.
    CellOccupiedBySameType,
    /// Player not registered in board state.
    PlayerNotFound,
}
```

```rust
// server/src/feature/board/components.rs — unit entity Required Components

use bevy::prelude::*;
use shared::session::PlayerId;
use shared::protocol::{CardId, CardType};

/// Absolute position on the board. Replicated to clients.
/// Player A advance direction: +1 (cell 1 → 8).
/// Player B advance direction: -1 (cell 8 → 1).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardPosition {
    pub lane: u8,   // 1–5
    pub cell: u8,   // 1–8 (absolute)
}

/// The player who owns this unit. Replicated to clients.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitOwner(pub PlayerId);

/// The card definition this unit was spawned from. Replicated to clients (for art/name lookup).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitCardRef(pub CardId);

/// Minion / Trap / Structure / Field. Replicated (for rendering layer and slot validation).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitType(pub CardType);

/// Base stats from card definition. Replicated (for combat preview UI).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitStats {
    pub atk: u8,
    pub mp: u8,
    pub ar: u8,  // armor reduction
    pub hp_base: u8,
}

/// Current HP; starts at hp_base and decreases as damage is applied.
/// Replicated (for HP bar display). May become negative during sub-step processing.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurrentHp(pub i32);
```

```rust
// server/src/feature/board/api.rs (key function signatures)

use bevy::prelude::*;
use shared::session::PlayerId;
use shared::protocol::{CardId, CardType};
use crate::feature::board::state::{BoardState, PlacementError, SpawnRangeState};
use crate::feature::board::components::*;

// ─── Spatial Queries (read-only; take &BoardState) ───────────────────────

/// All unit entities at (lane, cell). Returns empty slice if none.
pub fn get_units_at_cell<'s>(state: &'s BoardState, lane: u8, cell: u8)
    -> &'s [Entity];

/// All unit entities positioned at the opponent's objective cell.
/// Player A units at cell 8; Player B units at cell 1.
pub fn get_units_at_objective_cell<'s>(state: &'s BoardState, player: PlayerId)
    -> &'s [Entity];

/// True if (player, lane)'s Minion slot has an occupying entity.
pub fn is_minion_slot_occupied(state: &BoardState, player: PlayerId, lane: u8) -> bool;

/// Returns Ok if (player, card_type, lane, cell) is a legal placement.
/// Checks spawn range (Formula F2), cell occupancy, Minion slot.
pub fn validate_placement(
    state: &BoardState,
    spawn_ranges: &SpawnRangeState,
    player: PlayerId,
    card_type: CardType,
    lane: u8,
    cell: u8,
) -> Result<(), PlacementError>;

// ─── Mutations (update both entity components AND BoardState index) ───────

/// Spawn a unit entity from card data. Adds all Required Components.
/// Does NOT add Replicate — caller adds Replicate::to_clients(NetworkTarget::All) after S2CPlacementReveal.
/// Updates position_index and minion_slots in BoardState.
pub fn spawn_unit(
    world: &mut World,
    lane: u8,
    cell: u8,
    owner: PlayerId,
    card_id: CardId,
    card_type: CardType,
    stats: UnitStats,
    keywords: UnitKeywordState,
) -> Entity;

/// Move a unit to a new (lane, cell). Updates position_index in BoardState.
/// Enforces board bounds ([1,8] for cell, [1,5] for lane) via F1.
/// Called by resolve_combat for standard movement, CHARGE X, REPEL, ATTRACT.
pub fn move_unit(
    state: &mut BoardState,
    world: &mut World,
    entity: Entity,
    new_lane: u8,
    new_cell: u8,
);

/// Remove a unit entity from the board. Updates position_index and minion_slots.
/// Does NOT despawn the entity — caller decides timing (despawn at end of sub-step or frame).
pub fn remove_unit_from_board(
    state: &mut BoardState,
    entity: Entity,
    owner: PlayerId,
    lane: u8,
    card_type: CardType,
);

/// Execute CHANGE LANE for a Minion. Validates destination availability.
/// Silent no-op if new_lane is out-of-bounds or destination Minion slot is occupied.
pub fn change_lane_unit(
    state: &mut BoardState,
    world: &mut World,
    entity: Entity,
    new_lane: u8,
) -> bool; // true if CHANGE LANE executed; false = no-op

/// Expand spawn range for (player) by 1 (fake objective destroyed).
/// Clamped at 2 (cannot exceed 2 regardless of how many times called).
pub fn expand_spawn_range(state: &mut SpawnRangeState, player: PlayerId);

/// Sweep remaining 0-HP units and clear orphaned Minion slots.
/// Called from board cleanup system on OnResolutionEnd.
pub fn cleanup_board(state: &mut BoardState, world: &mut World);
```

```rust
// server/src/feature/board/plugin.rs

use bevy::prelude::*;
use crate::core::rsm::events::{PlacementPhaseEntered, ResolutionPhaseEntered, OnResolutionEnd};
use crate::feature::board::state::{BoardState, SpawnRangeState};
use crate::feature::board::system::{
    open_placement_window, commit_placement_buffer, cleanup_board_on_resolution_end,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardState>()
           .init_resource::<SpawnRangeState>()
            .add_systems(Update, (
                open_placement_window,     // reads PlacementPhaseEntered
                cleanup_board_on_resolution_end,  // reads OnResolutionEnd
            ).after(advance_phase));
        // commit_placement_buffer is called inside resolve_combat (exclusive system)
        // at sub-step 1 — not registered as a regular Bevy system.
    }
}
```

### Movement Formula Implementation

The GDD Formula F1 implemented in Rust (from GDD note: cast to i16 to prevent overflow):

```rust
/// Apply GDD Formula F1: new_cell = clamp(current_cell + direction × mp, 1, 8).
/// direction: +1 for Player A, -1 for Player B.
pub fn apply_movement_formula(current_cell: u8, direction: i8, mp: u8) -> u8 {
    let result = current_cell as i16 + direction as i16 * mp as i16;
    result.clamp(1, 8) as u8
}

/// Advance direction for a player: +1 (Player A) or -1 (Player B).
pub fn advance_direction(player: PlayerId) -> i8 {
    if player == PlayerId(0) { 1 } else { -1 }
}

/// True if a unit is at its player's objective cell (GDD Formula F3).
pub fn is_at_objective_cell(player: PlayerId, cell: u8) -> bool {
    (player == PlayerId(0) && cell == 8) || (player == PlayerId(1) && cell == 1)
}
```

## Alternatives Considered

### Alternative 1: Pure ECS — No BoardState Index Resource

- **Description**: Store all board data in Bevy entity components only. `resolve_combat` uses `world.query::<(&BoardPosition, &UnitOwner)>()` to find units at a given (lane, cell).
- **Pros**: Simpler data model — one source of truth (entity components). No index synchronization required.
- **Cons**: `world.query()` over all alive units to find units at a specific cell is O(n) per query. With 6 sub-steps and up to 10 alive units, resolve_combat could issue O(60) entity scans in a single exclusive system call. The BoardIndex eliminates all spatial scans to O(1) lookups.
- **Rejection Reason**: Performance. With the index, spatial lookups are one HashMap get. Without it, every CHARGE X movement, REPEL, ATTRACT, and objective-cell check scans all alive entities. The index is a small resource (at most 10 entities across 40 occupied cells) with negligible overhead.

### Alternative 2: Pure Resource — BoardState Owns All Unit Data (No ECS Entities for Units)

- **Description**: All unit data (position, HP, stats, keywords) stored in a `HashMap<UnitId, UnitRecord>` inside `BoardState`. No Bevy entities for units. Client projection written manually to S2C messages.
- **Pros**: Simple exclusive system access — `resolve_combat` only needs `world.resource_mut::<BoardState>()`. No entity component updates to synchronize.
- **Cons**: Lightyear's component replication cannot replicate non-entity data. Board position updates would require manual S2C broadcast of every position change — per sub-step, per lane, per unit. This is the approach for the placement buffer (where Lightyear replication is intentionally excluded), but for post-reveal live board state, Lightyear replication is the right delivery mechanism.
- **Rejection Reason**: ADR-007 establishes that unit entities ARE spawned at sub-step 1 commit and added to the Lightyear replication group. Post-commit unit positions and HP are meant to replicate automatically via component updates — not via manual S2C broadcast. A pure-resource approach would require the Board system to duplicate Lightyear's delivery mechanism.

### Alternative 3: Keep Spawn Range Inside `BoardState`

- **Description**: Store `spawn_range: [u8; 2]` directly on `BoardState` with the spatial index and Minion slots.
- **Pros**: One fewer resource and less parameter wiring for placement validation.
- **Cons**: Blurs two different contracts. `BoardState` is the spatial index for live entities; spawn range is a player projection derived from Objective destruction facts and used for protocol snapshot/live-update transport. Keeping it inside `BoardState` encouraged stale docs to treat it like ordinary replicated board state.
- **Rejection Reason**: `SpawnRangeState` needs a distinct source/transport contract: Board/Lane owns the live projection, Objective owns only destruction facts/counters, live clients receive `SpawnRangeChanged`, and reconnecting clients receive `PlayerSnapshot.spawn_range_cells`. A separate Board/Lane resource makes that ownership explicit.

## Consequences

### Positive

- `get_units_at_cell(state, lane, cell)` is a HashMap lookup — O(1) regardless of total unit count. `resolve_combat` can issue any number of spatial queries without performance concern.
- Unit ECS entities with `BoardPosition` components and `Replicate::to_clients(NetworkTarget::All)` give Lightyear complete information to replicate live board state to both clients after sub-step 1. No manual per-event S2C broadcast needed for position updates.
- The board API module (`board/api.rs`) is the single mutation path for spatial state. Any spatial inconsistency (position_index out of sync with components, orphaned Minion slots) is isolated to this module. Testing board correctness means testing this module.
- `WorldNew()` tests can insert `BoardState` as a resource, call `spawn_unit`/`move_unit`/etc., and assert index state without a live Lightyear session. All BL-* acceptance criteria in the GDD are testable this way.
- Spawn range (`SpawnRangeState`) is a Board/Lane-owned projection — readable at O(1) during PLACEMENT validation, used as the source for `PlayerSnapshot.spawn_range_cells`, and used to emit `ResolutionEvent::SpawnRangeChanged`.

### Negative

- The `BoardState` index must be kept in sync with entity component state at all times. Any code that directly mutates a `BoardPosition` component without going through `board/api.rs` creates a silent inconsistency — the entity is at one position, the index says another. This is the same constraint as Economy's `api.rs` boundary and requires the same enforcement (CI grep + code review gate).
- `remove_unit_from_board` does not despawn the entity immediately — the entity is removed from the index but may persist as a zombie in the ECS world for the remainder of the sub-step. Callers must explicitly despawn (via `commands.entity(e).despawn()`) after the sub-step that triggers the removal. This two-phase remove (index removal + later despawn) is necessary because despawning inside an exclusive system while queries are still open can panic. The cleanup system at `OnResolutionEnd` is the safety net.
- Lightyear 0.26 replication uses `Replicate::to_clients(NetworkTarget::All)` (not a `ReplicateTo` component — that name does not exist). The sub-step 1 commit path uses `commands.entity(e).insert(Replicate::to_clients(NetworkTarget::All))` after `S2CPlacementReveal` is enqueued.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| `BoardPosition` component mutated directly (outside api.rs), causing index drift | Medium | Silent spatial query corruption — units appear at wrong cell to resolve_combat | CI grep for `.lane =`, `.cell =` on `BoardPosition` outside board/api.rs. Code review gate on every board PR. |
| `Replicate::to_clients` added before `S2CPlacementReveal` is enqueued, leaking opponent placement | Low | Critical — breaks simultaneous-reveal invariant (ADR-007) | Sub-step 1 commit sequence is strictly ordered: reveal enqueue first, then `Replicate::to_clients(NetworkTarget::All)`. Integration test that asserts no board entity is replicated before `S2CPlacementReveal` fires in the same frame. |
| `world.query()` in exclusive system causes panic if ECS world is in inconsistent state during resolve_combat | Low | Runtime panic | Exclusive systems in Bevy have exclusive world access — no other system runs concurrently. `world.query()` is safe in exclusive context. Verify with Bevy 0.18 engine-reference. |
| BoardIndex out of sync after unexpected resolve_combat abort (timeout) | Low | Subsequent PLACEMENT round uses stale board state | `cleanup_board` on `OnResolutionEnd` sweeps inconsistencies as a safety net. Integration test: abort resolve_combat mid-sub-step and assert board state is consistent after cleanup. |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|-----------|-------------|--------------------------|
| `board-lane-system.md` | Rule 1 — Coordinate system: absolute cells 1–8; Player A +1 direction, Player B −1 direction | `BoardPosition { lane, cell }` absolute; `advance_direction(player)` returns ±1 |
| `board-lane-system.md` | Rule 3 — 1 Minion slot per player per lane | `minion_slots: HashMap<(PlayerId, lane), Option<Entity>>` in BoardState; `validate_placement` checks before accepting Minion |
| `board-lane-system.md` | Rule 4 — Spawn range: fakes_destroyed 0–2 | `SpawnRangeState` in Board/Lane; `validate_placement` applies Formula F2; `expand_spawn_range` increments on fake destruction and provides the source for snapshot/live transport |
| `board-lane-system.md` | Rule 5 — Cell occupancy limits by card type | `validate_placement` checks position_index for same-type cell occupancy (Trap/Structure); Minion slot for Minion type |
| `board-lane-system.md` | Rule 6 — Pending placement buffer commits atomically at sub-step 1 | `spawn_unit` called per buffer entry at sub-step 1; `Replicate::to_clients(NetworkTarget::All)` added after `S2CPlacementReveal` enqueued (ADR-007 pattern) |
| `board-lane-system.md` | Rule 7 — Board provides `get_units_at_cell`, `move_unit`, `remove_unit` etc. to Combat Resolution | Board API functions with those signatures; called from `resolve_combat` exclusive system |
| `board-lane-system.md` | Rule 8 — Standard movement: `new_cell = clamp(current + direction × mp, 1, 8)` | `apply_movement_formula(current_cell, direction, mp) -> u8` — F1 exact implementation |
| `board-lane-system.md` | Rule 9 — REPEL/ATTRACT/CHANGE LANE displacement | `move_unit` applies F1 with custom direction argument per GDD F1 table; `change_lane_unit` enforces lane bounds and Minion slot availability |
| `board-lane-system.md` | Rule 11 — Prism collected when player's unit ends sub-step 5 at prism cell | `move_unit` checks if new position == prism cell for that player; if so, emits `PrismCollected(player, lane)` (Bevy Message) |
| `board-lane-system.md` | Rule 12 — Board cleanup after OnResolutionEnd | `cleanup_board_on_resolution_end` system reads `OnResolutionEnd`; calls `cleanup_board(state, world)` |
| `board-lane-system.md` | F3 — Objective cell detection | `is_at_objective_cell(player, cell) -> bool` — checked at sub-step 6 end; emits `UnitAtObjective` if true |
| `board-lane-system.md` | BL-1 — Player A at cell 1 MP=3: new cell = 4 | `apply_movement_formula(1, +1, 3) = 4` — unit test in tests/unit/board/ |
| `board-lane-system.md` | BL-2 — Player A at cell 6 MP=3: new cell = 8 (clamped) | `apply_movement_formula(6, +1, 3) = clamp(9, 1, 8) = 8` |
| `board-lane-system.md` | BL-3 — Player B at cell 5 MP=2: new cell = 3 | `apply_movement_formula(5, -1, 2) = clamp(3, 1, 8) = 3` |
| `board-lane-system.md` | BL-4 — WALL unit (MP=0): no movement | `apply_movement_formula(cell, dir, 0) = cell` |
| `board-lane-system.md` | BL-5 — Player A 0 fakes, Minion at cell 2: rejected | `validate_placement(state, spawn_ranges, PlayerA, Minion, lane, 2)` returns `Err(OutOfSpawnRange)` when `SpawnRangeState.fakes_destroyed[0] == 0` |

## Performance Implications

- **CPU**: `get_units_at_cell` is one `HashMap::get()` call — O(1). Board cleanup is O(n entities) once per round on `OnResolutionEnd`. resolve_combat sub-steps issue at most ~50 spatial queries (5 lanes × ~10 units × sub-steps requiring cell checks) — all O(1) with the index. Total board CPU budget: < 0.5 ms per RESOLUTION phase.
- **Memory**: `BoardState` holds at most 10 entries in `position_index` (2 units × 5 lanes in a full game) and 10 Minion slot entries (2 players × 5 lanes). `SpawnRangeState` holds 2 u8 values. Total: < 2 KB.
- **Network**: Unit entity component replication via Lightyear is delta-compressed — only changed fields are transmitted. A single unit movement transmits one `BoardPosition` delta. Per ADR-002, total per-round network budget is < 1 KB.
- **Load Time**: `BoardState` is `Default`-initialized in plugin build; no asset loading.

## Migration Plan

This is a greenfield system — no existing board implementation. Implementation sequence:

1. **Define components** in `server/src/feature/board/components.rs`: `BoardPosition`, `UnitOwner`, `UnitCardRef`, `UnitType`, `UnitStats`, `CurrentHp`. No bundles — Required Components API only.
2. **Define `BoardState`** in `server/src/feature/board/state.rs` with `#[derive(Resource, Default)]`.
3. **Implement `board/api.rs`** functions: start with pure query functions (`get_units_at_cell`, `validate_placement`, `apply_movement_formula`) — these are unit-testable without spawning entities.
4. **Implement mutation functions** (`spawn_unit`, `move_unit`, `remove_unit_from_board`) — these require a `World`. Use `World::new()` tests with manually spawned entities.
5. **Register plugin**: `BoardPlugin` in the server app, initializing `BoardState` resource. Add systems for `open_placement_window` (reads `PlacementPhaseEntered`) and `cleanup_board_on_resolution_end` (reads `OnResolutionEnd`).
6. **Wire into resolve_combat** (ADR-017): After `resolve_combat` is implemented, call `spawn_unit` at sub-step 1, `move_unit` at sub-steps 2 and 5, `remove_unit_from_board` at sub-step 4, and `is_at_objective_cell` at sub-step 6.
7. **Add integration test**: Full one-round sequence — spawn units, run sub-step 5 movement, assert positions via both entity queries and BoardState index (both must agree).

## Validation Criteria

- [ ] `apply_movement_formula` passes BL-1 through BL-4 as unit tests with no Bevy app.
- [ ] `validate_placement` returns `Err(OutOfSpawnRange)` when `SpawnRangeState.fakes_destroyed[player] == 0` and cell > player's spawn cell. (BL-5)
- [ ] After `spawn_unit(world, lane=1, cell=1, PlayerA, ...)`: `get_units_at_cell(state, 1, 1)` returns the new entity; `is_minion_slot_occupied(state, PlayerA, 1)` returns true.
- [ ] After `move_unit(state, world, entity, lane=1, new_cell=4)`: `get_units_at_cell(state, 1, 1)` is empty; `get_units_at_cell(state, 1, 4)` contains the entity; `entity.get::<BoardPosition>().cell == 4`.
- [ ] After `remove_unit_from_board(state, entity, PlayerA, lane=1, Minion)`: `get_units_at_cell(state, 1, 4)` is empty; `is_minion_slot_occupied(state, PlayerA, 1)` returns false.
- [ ] `Replicate` is NOT present on unit entities immediately after `spawn_unit`. It is only added (via `Replicate::to_clients(NetworkTarget::All)`) after `S2CPlacementReveal` is enqueued. Verified by asserting entity lacks `Replicate` component before the reveal send, and has it after.
- [ ] `BoardPosition` component and `BoardState.position_index` agree for all alive units at every sub-step boundary in the integration test. No drift detected.
- [ ] `expand_spawn_range(&mut SpawnRangeState, PlayerA)` called 3× clamps at 2 (max fakes destroyed). Unit test.
- [ ] `change_lane_unit` to a lane that is out of bounds [1–5] is a silent no-op — entity remains in original lane and index unchanged.

## Related Decisions

- [ADR-002 — Client-Server Authority](./adr-002-client-server-authority.md) — Board unit entities are server-spawned; clients receive `BoardPosition` and stats via Lightyear replication only.
- [ADR-007 — Placement Buffer](./adr-007-placement-buffer.md) — Defines `PlacementBuffer` resource and the simultaneous-reveal invariant. ADR-020 picks up at sub-step 1 commit.
- [ADR-017 — Combat Resolution](./adr-017-combat-resolution-execution-architecture.md) — `resolve_combat` exclusive system calls board API functions for all spatial operations during sub-steps 1–6.
- [ADR-018 — Keyword System](./adr-018-keyword-system.md) — `UnitKeywordState` component lives on unit entities; keyword effects (REPEL, ATTRACT, CHANGE LANE, IRREMOVABLE) use board API functions for execution.
- [ADR-016 — Prism System](./adr-016-prism-system-architecture.md) — Board emits `PrismCollected(player, lane)` during sub-step 5; Prism System owns all reward delivery.
- `design/gdd/board-lane-system.md` — Primary GDD source; all 5 formulas (F1–F3) and BL-* acceptance criteria.
- `server/src/feature/board/` — Target implementation directory (greenfield).
