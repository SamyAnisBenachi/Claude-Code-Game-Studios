# Story 012: Spawn Range Authoritative Projection

> **Epic**: Board / Lane System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-003`, `TR-BLS-010`, `TR-NP-014`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*

**ADR Governing Implementation**: ADR-020: Board/Lane System State Architecture; ADR-008: Lightyear Channel Config; ADR-011: Reconnect and Snapshot
**ADR Decision Summary**: Board/Lane owns `SpawnRangeState` as the live authoritative spawn range projection. Objective System owns objective destruction facts and counters only. Network Protocol delivers live changes as ordered `SpawnRangeChanged` entries and uses `PlayerSnapshot.spawn_range_cells` for recovery.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use `liv-bevy-018` for Bevy code and `liv-bevy-lightyear` for Lightyear/protocol wiring before implementation.
**Performance Budget**: Spawn-range projection reads/writes must remain O(1) per player. Event assembly may add only the changed player's `SpawnRangeChanged` entry and must not scan board cells or recompute from objective counters.

**Control Manifest Rules (Feature layer)**:
- Required: `SpawnRangeState` is mutated only through Board/Lane API functions.
- Required: Snapshot builders read `SpawnRangeState` for `PlayerSnapshot.spawn_range_cells`.
- Required: Fake objective destruction appends `SpawnRangeChanged` after `ObjectiveDestroyed`.
- Forbidden: Do not recompute snapshot spawn range directly from `ObjectiveCounters`.
- Forbidden: Do not add a replicated `SpawnRange` component.

---

## Prerequisite Status

- `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md` is Complete as of `af11e1f` and defines/registers the shared `ResolutionEvent::SpawnRangeChanged` variant.
- Objective fake-destruction facts/counters are available from the completed Objective consequence path; BLS-012 owns turning those facts into the Board/Lane live projection.

---

## Acceptance Criteria

- [ ] **Single live source**: `SpawnRangeState` is the only server resource used by Board/Lane placement validation to determine current spawn range.

- [ ] **Objective fact consumption**: Given Objective System records a fake objective destroyed by the opponent, when Board/Lane consumes that fact, then Board/Lane updates `SpawnRangeState` for the attacker and clamps at the configured maximum.

- [ ] **No ObjectiveCounters drift**: Given `ObjectiveCounters.fake_destroyed` and `SpawnRangeState` could diverge due to ordering or reconnect assembly, when `S2CGameSnapshot` is built, then `PlayerSnapshot.spawn_range_cells` is read from `SpawnRangeState`, not recomputed directly from `ObjectiveCounters`.

- [ ] **Live event production**: Given `SpawnRangeState` changes during RESOLUTION, when `S2CResolutionEvent` is assembled, then a `SpawnRangeChanged { player_id, new_spawn_range_cells }` entry is included for the changed player.

- [ ] **Ordering contract**: The `SpawnRangeChanged` entry appears after the corresponding `ObjectiveDestroyed` entry in the same ordered event array.

- [ ] **No replicated component**: Board/Lane does not define, insert, or register a replicated `SpawnRange` component.

---

## Implementation Notes

The intended source chain is:

```text
Objective System
  owns objective HP/identity/destruction counters
  emits or exposes fake-destruction fact

Board/Lane System
  consumes fake-destruction fact
  updates SpawnRangeState
  uses SpawnRangeState for placement validation
  supplies SpawnRangeState to snapshot assembly
  contributes SpawnRangeChanged to S2CResolutionEvent
```

The story should remove any current snapshot shortcut that derives `spawn_range_cells` from `ObjectiveCounters`. `ObjectiveCounters` may remain the RSM/objective fact contract, but it is not the live projection source for placement validation or client recovery.

Centralize spawn range mutation behind a Board/Lane-owned API helper (ADR-020 names this `expand_spawn_range`) so placement validation, snapshot assembly, and resolution-event production all share the same `SpawnRangeState` source.

## Out of Scope

- Protocol schema definition (`NP-006`).
- Board Rendering highlight consumption (`BR-011`).
- Objective reward RNG, gold awards, mana cap rewards, or free-card pick.
- Final visual evidence.

---

## QA Test Cases

- **Snapshot source**
  - Given: `ObjectiveCounters.fake_destroyed(player) = 2` but `SpawnRangeState` is intentionally set to 1 in a test fixture
  - When: `S2CGameSnapshot` is built
  - Then: `PlayerSnapshot.spawn_range_cells == 1`, proving snapshot uses the live projection source.

- **Event ordering**
  - Given: a fake objective destruction updates `SpawnRangeState`
  - When: the resolution event array is assembled
  - Then: `ObjectiveDestroyed` precedes `SpawnRangeChanged`.

- **Clamp**
  - Given: `expand_spawn_range` is called three times for one player
  - When: `SpawnRangeState` is inspected
  - Then: the stored projection clamps at the two-fake maximum / three-cell placement range.

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Unit: `tests/unit/board-lane-system/spawn_range_authoritative_projection_test.rs`
- Integration: `tests/integration/combat/resolution_event_log_test.rs` covers ordered `ObjectiveDestroyed` -> `SpawnRangeChanged` assembly in `S2CResolutionEvent`.

**Status**: [x] Complete

---

## Dependencies

- Depends on: `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md` (Complete at `af11e1f` - shared `ResolutionEvent::SpawnRangeChanged` exists).
- Depends on: `production/epics/objective-system/story-005-destruction-consequence-path.md` (Complete - fake destruction updates counters and emits the Board/Lane fact).
- Depends on: `production/epics/objective-system/story-007-resolution-phase-subscription.md` (Complete - `ObjectiveDestroyed` sync path exists).
- Depends on: `production/epics/combat-resolution/story-009-objective-damage-gameover.md` (Complete - objective destruction enters the resolution log path).
- Unlocks: `production/epics/board-rendering/story-011-spawn-range-highlights.md`.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 6/6 passing.
**Deviations**: None.
**Test Evidence**: `tests/unit/board-lane-system/spawn_range_authoritative_projection_test.rs`, `tests/integration/combat/resolution_event_log_test.rs`, and `tests/unit/protocol/spawn_range_live_update_contract_test.rs` cover the BLS-012 acceptance criteria. Requested server and shared test bundles, `cargo fmt`, `cargo check -p server`, and `git diff --check` passed.
**Code Review**: Skipped - lean mode (`production/review-mode.txt` absent).
**Implementation**: Worker commit `cfc44ac888297782c439be9963d877fa4497dae1` was integrated onto `main` as `4418aae`.
**Scope Notes**: BR-011 spawn range highlights, final board visual/browser evidence, status icon/co-occupancy evidence, and design/assets work were not implemented or closed.
