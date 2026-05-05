# Story 011: Spawn Range Highlights

> **Epic**: Board Rendering
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-008`, `TR-NP-014`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*

**ADR Governing Implementation**: ADR-021: Presentation Layer Architecture; ADR-020: Board/Lane System State Architecture; ADR-011: Reconnect and Snapshot; ADR-008: Lightyear Channel Config
**ADR Decision Summary**: Board Rendering is a read-only presentation layer. Spawn range highlights seed from snapshots and update from ordered reliable resolution-log entries. Board/Lane owns live projection; Objective destruction events alone are not a spawn range source.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use `liv-bevy-018` for Bevy rendering code and `liv-bevy-lightyear` if touching Lightyear receiver code.

**Control Manifest Rules (Presentation layer)**:
- Required: Board content is world-space `Sprite` + `Transform`, not bevy_ui.
- Required: `PlayerSnapshot.spawn_range_cells` seeds reconnect/initial highlight state.
- Required: `ResolutionEvent::SpawnRangeChanged` is the live update path.
- Forbidden: Do not derive spawn range from `ObjectiveDestroyed.was_fake`.
- Forbidden: Do not consume a replicated `SpawnRange` component.

---

## Blockers

- `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md` must define the shared `SpawnRangeChanged` variant.
- `production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md` must make `SpawnRangeState` the snapshot/live source.

---

## Acceptance Criteria

- [ ] **Snapshot seed**: Given `S2CGameSnapshot` includes `PlayerSnapshot.spawn_range_cells` for both players, when Board Rendering rebuilds the board, then every `BoardCellNode` has `SpawnHighlightState` matching the valid cells for the corresponding player side.

- [ ] **Live update consumption**: Given an ordered `S2CResolutionEvent` contains `SpawnRangeChanged { player_id, new_spawn_range_cells }`, when Board Rendering drains the resolution event, then only that player's cell-node highlight states update.

- [ ] **Persistence**: Given spawn range highlights update during RESOLUTION, when the board transitions to DRAFT/PLACEMENT, then the updated highlights persist and the newly unlocked cells render with the spawn-active visual state.

- [ ] **Objective event separation**: Given `ObjectiveDestroyed { was_fake: true }` arrives without a matching `SpawnRangeChanged`, then Board Rendering performs the objective reveal/clear behavior but does not change spawn highlights.

- [ ] **OBJMISS behavior**: Given no objective entity exists when `ObjectiveDestroyed` is processed, then no objective entity is spawned, no panic occurs, and spawn highlights change only if a separate `SpawnRangeChanged` entry exists.

- [ ] **No replicated component**: Board Rendering does not query or register a `SpawnRange` replicated component.

---

## Implementation Notes

Add or finish the presentation state described in the GDD:

```rust
struct SpawnHighlightState {
    player_id: PlayerId,
    in_spawn_range: bool,
}
```

Attach it to `BoardCellNode` entities. Snapshot rebuild computes the initial state from each public `PlayerSnapshot.spawn_range_cells`. Resolution-message drain applies `SpawnRangeChanged` by recomputing affected node state from `new_spawn_range_cells`.

The update path should live in the existing Board Rendering message-drain/resolution-event path. Do not add a duplicate `MessageReceiver<S2CPhaseChanged>` or any second drain for a Lightyear message that already has a single owner.

## Out of Scope

- Protocol schema or server event production.
- Final browser frame-time, atlas, or art evidence.
- Drag-time Hand UI placement pre-validation.
- Objective reveal/HUD fanout beyond preserving the corrected spawn range separation.

---

## QA Test Cases

- **Snapshot seed**
  - Given: Player A snapshot range = 2 and Player B snapshot range = 1
  - When: board rebuild runs
  - Then: Player A side cells 1-2 are active, Player B side cell 8 is active, and the rest are inactive.

- **Live update**
  - Given: Player A starts at range 1
  - When: `SpawnRangeChanged { player_id: PlayerA, new_spawn_range_cells: 2 }` is processed
  - Then: Player A's second-cell nodes become active and Player B nodes are unchanged.

- **No ObjectiveDestroyed derivation**
  - Given: `ObjectiveDestroyed { was_fake: true }` without `SpawnRangeChanged`
  - When: the event is processed
  - Then: spawn highlight state remains unchanged.

---

## Test Evidence

**Story Type**: Visual/Feel
**Required evidence**:
- Unit/integration support: `tests/unit/board_rendering/spawn_range_highlights_test.rs`
- Visual evidence deferred to the later final board evidence story.

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md`.
- Depends on: `production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md`.
- Unlocks: final Board Rendering evidence story covering status-icon atlas, spawn-highlight visual evidence, and browser frame-time evidence.
