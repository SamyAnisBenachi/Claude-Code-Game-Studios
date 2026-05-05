# Story 003: Snapshot Spawn, Units, Objectives, and HP Bars

> **Epic**: Board Rendering
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-003`
**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](../../../docs/architecture/adr-001-objective-identity-unicast.md), [ADR-020: Board/Lane System State Architecture](../../../docs/architecture/adr-020-board-lane-state-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story turns authoritative server state into visible board entities. `S2CGameSnapshot` and replicated board components rebuild the current visual board with unit sprites, standing objectives, and HP bar children. Final art is not required; placeholder atlas frames are acceptable while preserving the atlas and batching contract.

## Traceability

- `TR-BR-003`: board rendering consumes replicated `BoardPosition`/unit display state and uses board z constants rather than inline literals. This story covers unit sprite spawning, `LaneCell`, world-space `Transform`/`Sprite` state, and HP child local-z handling.
- Snapshot rebuild: GDD Rule 11 and AC `BR-17` require `S2CGameSnapshot` to despawn stale board entities, cancel stale `Animator<Transform>` / `Animator<Sprite>` components, rebuild the snapshot view, and clear `AnimQueue`, `PendingPhaseChange`, and `PendingResolutionScript`. `BR-EC-LOBBY-SNAPSHOT` applies the same no-leftover-state rule when the snapshot arrives from Lobby.
- HP bars: GDD Rule 6, `BR-4`, `BR-5`, `BR-Z-LOCAL`, `BR-HP-EPSILON`, and `BR-HP-INVARIANT` require each live unit to have background/fill HP sprites, fill scale and color derived from current/max HP with epsilon-aware thresholds, local child z of `Z_HEALTH_BARS - Z_UNITS`, and no tween writer on HP fill scale.
- Standing objective identity isolation: GDD Rule 12, ADR-001, and `BR-19` require all standing objectives to render identically before destruction reveal. Standing objective rendering must not query identity-bearing components or read `ObjectiveIdentityCache`; the cache is only for allowed reveal/audio branches outside standing-objective display.
- Missing-card fallback: `BR-EC-CARDMISS` requires unknown card IDs to spawn exactly one placeholder-atlas unit, keep the HP bar visible, log a warning containing `card_id` or `asset-miss`, and never panic.
- Pending state reconciliation: `TR-BR-005`, Rule 11, `BR-17`, and `BR-EC-PLACEMENT-STUCK` make pending phase/script state subordinate to the authoritative snapshot. If `PendingPhaseChange` or `PendingResolutionScript` exists when the snapshot arrives, the rebuild path must clear or explicitly reconcile it with the snapshot phase so no stale pending state survives.
- Atlas/batching requirements: Rule 5, `BR-3a`, `BR-3b`, `BR-3c`, and `BR-2-ATLAS` require unit sprites and HP bars to use atlas-backed Bevy sprites, avoid per-unit material handles, and preserve the shared unit/board-elements atlas assumptions.

## Acceptance Criteria

- [ ] `S2CGameSnapshot` rebuild clears stale board-rendering entities before spawning the snapshot view.
- [ ] Every visible unit has `LaneCell`, `Transform`, `Sprite`, owner/card identity markers, and HP bar background/fill child sprites.
- [ ] HP bar fill width and color derive from current/max HP using GDD thresholds and `HP_THRESHOLD_EPSILON` handling.
- [ ] HP bar child local Z values are `Z_HEALTH_BARS - Z_UNITS`, not absolute `Z_HEALTH_BARS`.
- [ ] Standing objectives render from the same unknown objective atlas frame/component set regardless of real/fake identity, and standing-objective rendering does not query identity-bearing components or `ObjectiveIdentityCache`.
- [ ] Missing card art uses a placeholder atlas frame and logs a warning without panic.
- [ ] Snapshot rebuild leaves `AnimQueue`, `PendingPhaseChange`, and `PendingResolutionScript` cleared or explicitly reconciled with the snapshot phase; no stale pending value survives.

## Implementation Notes

- Board Rendering consumes replicated display state only; it does not import server feature modules directly.
- Objective identity is not used for standing-objective rendering. It is only cached for reveal behavior after the allowed S2C identity message.
- Use the Bevy 0.18 atlas pattern: `Sprite { texture_atlas: Some(TextureAtlas { layout, index }), .. }`. Do not use `Handle<TextureAtlas>`.
- Use atlas-backed sprites for placeholders so batching behavior is representative.
- `UpdateHpBars` writes HP bar fill scale directly; do not attach `Animator<Transform>` to HP bar fill entities.
- Snapshot rebuild should preserve the Rule 5/ADR-021 atlas and batching assumptions: unit sprites and HP bars stay on the shared unit atlas, board elements stay on the board-elements atlas, and no per-unit material handles are introduced. Final browser/WASM performance evidence for `BR-3`, `BR-FRAME-TIME`, and `BR-RECONNECT-TIME` remains Story 010 scope.

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
