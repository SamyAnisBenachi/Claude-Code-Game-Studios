# Story 006: Spawn Range Live Update Contract

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: `TR-NP-014`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*

**ADR Governing Implementation**: ADR-003: Cargo Workspace Structure; ADR-008: Lightyear Channel Config; ADR-011: Reconnect and Snapshot; ADR-020: Board/Lane System State Architecture
**ADR Decision Summary**: Shared protocol types live in `shared/src/protocol.rs`. Spawn range live updates use the existing reliable ordered `S2CResolutionEvent` batch, while reconnect recovery uses `PlayerSnapshot.spawn_range_cells`. Board/Lane owns the live projection; Objective owns destruction facts only.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: No new Lightyear channel. Register any new or revised protocol enum variant through the existing shared protocol registration path.

**Control Manifest Rules (Foundation layer)**:
- Required: `ResolutionEvent::SpawnRangeChanged { player_id, new_spawn_range_cells }` is a concrete shared protocol variant.
- Required: The event travels inside `S2CResolutionEvent` on `ReliableChannel`.
- Required: `SpawnRangeChanged` is ordered after the corresponding `ObjectiveDestroyed` entry in the same event batch.
- Forbidden: Do not add a replicated `SpawnRange` component.
- Forbidden: Do not make connected clients depend on snapshot-only updates.

---

## Acceptance Criteria

- [x] **NP-33 / TR-NP-014 - schema**: `ResolutionEvent` includes `SpawnRangeChanged { player_id: PlayerId, new_spawn_range_cells: u8 }` in `shared/src/protocol.rs`.

- [x] **NP-33 / TR-NP-014 - reliable live transport**: Given a fake objective destruction expands the attacker's spawn range, when `S2CResolutionEvent` is emitted, then the event batch includes `SpawnRangeChanged` on the same `ReliableChannel` message as the rest of the resolution log.

- [x] **NP-33 / TR-NP-014 - ordering**: Given the same fake destruction, when the event array is inspected, then `ObjectiveDestroyed { was_fake: true }` appears before `SpawnRangeChanged { player_id: attacker, new_spawn_range_cells }`.

- [x] **Snapshot recovery role**: `PlayerSnapshot.spawn_range_cells` remains present and public for initial connect/reconnect recovery, but no implementation treats snapshots as the live update path for connected clients.

- [x] **No replicated component**: No `SpawnRange` ECS component is registered for Lightyear replication and no protocol registration attempts to replicate spawn range as a component.

---

## Implementation Notes

Keep the payload intentionally small and public:

```rust
ResolutionEvent::SpawnRangeChanged {
    player_id: PlayerId,
    new_spawn_range_cells: u8,
}
```

This story only establishes the wire/schema contract. Board/Lane Story 012 owns when the event is produced and which authoritative server resource supplies `new_spawn_range_cells`. Board Rendering Story 011 owns client consumption and highlight updates.

## Out of Scope

- Updating `SpawnRangeState` in Board/Lane.
- Repairing snapshot source to read `SpawnRangeState`.
- Board Rendering `SpawnHighlightState` or cell-node visuals.
- Objective reward/counter behavior.

---

## QA Test Cases

- **Schema compile**
  - Given: the shared protocol after adding the variant
  - When: `cargo check -p shared` runs
  - Then: `ResolutionEvent::SpawnRangeChanged` compiles and registers through the existing protocol path.

- **Ordering fixture**
  - Given: a test event batch containing fake `ObjectiveDestroyed`
  - When: the resolution event builder appends spawn range changes
  - Then: `SpawnRangeChanged` appears later in the array than the matching `ObjectiveDestroyed`.

- **No component grep**
  - Given: protocol and replicated component registration code
  - When: searching for a replicated `SpawnRange` component registration
  - Then: no production registration exists.

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**:
- `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
- `cargo check -p shared` if `shared/src/protocol.rs` or protocol/schema registration is touched.

**Status**: [x] Created and verified 2026-05-05

---

## Dependencies

- Depends on: Lightyear Protocol Story 002 complete enough to own `ResolutionEvent` in `shared/src/protocol.rs`.
- Unlocks: `production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md`.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 5/5 passing.
**Deviations**: None.
**Test Evidence**: `tests/unit/protocol/spawn_range_live_update_contract_test.rs`; `cargo test -p shared --test spawn_range_live_update_contract` passed 5/5; `cargo test -p shared`, `cargo check -p shared`, `cargo check -p server`, `cargo check -p client`, `cargo fmt -p shared -- --check`, and `git diff --check` passed.
**Code Review**: Skipped - lean review mode.
**Scope Notes**: NP-006 only. BLS-012, BR-011, spawn highlight visuals, `design/assets/**`, unrelated `AGENTS.md`, and `production/session-state/codex-orchestrator-state.md` were not touched.
