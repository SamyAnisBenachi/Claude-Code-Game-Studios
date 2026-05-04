# Story 003: Snapshot Spawn, Units, Objectives, and HP Bars

> **Epic**: Board Rendering
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-003`
**ADR Governing Implementation**: [ADR-020: Board/Lane System State Architecture](../../../docs/architecture/adr-020-board-lane-state-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story turns authoritative server state into visible board entities. `S2CGameSnapshot` and replicated board components rebuild the current visual board with unit sprites, standing objectives, and HP bar children. Final art is not required; placeholder atlas frames are acceptable while preserving the atlas and batching contract.

## Acceptance Criteria

- [ ] `S2CGameSnapshot` rebuild clears stale board-rendering entities before spawning the snapshot view.
- [ ] Every visible unit has `LaneCell`, `Transform`, `Sprite`, owner/card identity markers, and an HP bar child.
- [ ] HP bar fill width and color derive from current/max HP using GDD thresholds and epsilon handling.
- [ ] Health bar child local Z is `Z_HEALTH_BARS - Z_UNITS`, not absolute `Z_HEALTH_BARS`.
- [ ] Standing objectives render identically regardless of real/fake identity.
- [ ] Missing card art uses a placeholder atlas frame and logs a warning without panic.
- [ ] Snapshot rebuild leaves `AnimQueue`, `PendingPhaseChange`, and `PendingResolutionScript` in a cleared or explicitly reconciled state.

## Implementation Notes

- Board Rendering consumes replicated display state only; it does not import server feature modules directly.
- Objective identity is not used for standing-objective rendering. It is only cached for reveal behavior after the allowed S2C identity message.
- Use atlas-backed sprites for placeholders so batching behavior is representative.
- `UpdateHpBars` writes HP bar fill scale directly; do not attach `Animator<Transform>` to HP bar fill entities.

## Out of Scope

- Placement reveal collect-and-tween behavior (Story 005).
- Resolution playback and movement/death event dispatch (Story 006).
- Objective destruction reveal/HUD fanout (Story 008).
- Final performance evidence (Story 010).

## QA Test Cases

- **Snapshot rebuild**
  - Given: stale board entities exist and a snapshot contains two units plus objectives
  - When: snapshot rebuild runs
  - Then: stale entities are gone and visible entities match the snapshot exactly.

- **HP local Z**
  - Given: a unit at `Z_UNITS`
  - When: its HP bar child is spawned
  - Then: child local Z equals `Z_HEALTH_BARS - Z_UNITS` and computed global Z equals `Z_HEALTH_BARS`.

- **Missing art fallback**
  - Given: a snapshot references an unknown `card_id`
  - When: unit render spawns
  - Then: placeholder frame is used, HP bar still exists, and no panic occurs.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/board_rendering/snapshot_spawn_test.rs`
- Optional screenshot evidence once placeholders are visible.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-board-layout-card-atlas.md), [Story 002](story-002-board-grid-camera-and-z-layers.md), server Board/Lane replicated components.
- Unlocks: Stories 005, 006, 007, 008, 009.
