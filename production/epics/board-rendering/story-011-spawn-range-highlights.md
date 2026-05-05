# Story 011: Spawn Range Highlights

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-008`, `TR-NP-014`

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

## Requirement Trace

- `TR-BR-008`: Persistent spawn range highlights seed from `PlayerSnapshot.spawn_range_cells` on snapshot rebuild and update from ordered `ResolutionEvent::SpawnRangeChanged` entries in `S2CResolutionEvent`; highlights persist across DRAFT/PLACEMENT frames and are not derived from `ObjectiveDestroyed` alone.
- `TR-NP-014`: Live spawn range updates use `ResolutionEvent::SpawnRangeChanged { player_id, new_spawn_range_cells }` inside the ordered reliable `S2CResolutionEvent` batch, after the corresponding `ObjectiveDestroyed`; `PlayerSnapshot.spawn_range_cells` is recovery/reconnect source only; there is no replicated `SpawnRange` component and no snapshot-only live update path.
- Direct GDD ACs: `BR-SPAWN-HIGHLIGHTS` and `BR-EC-OBJMISS` in `design/gdd/board-rendering.md`.
- Network Protocol AC: `NP-33` in `design/gdd/network-protocol.md`.

## Prerequisite Status

- NP-006 is Complete: `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md` defines/registers the shared `ResolutionEvent::SpawnRangeChanged` variant and closed on main at `af11e1f`.
- BLS-012 is Complete: `production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md` made `SpawnRangeState` the authoritative snapshot/live source, integrated as `4418aae`, and closed by story-done commit `922cedd`.
- BOARD-009 narrowed status-icon and co-occupancy scope is Complete. Its final visual/browser evidence remains separate, and BOARD-009 did not implement or close spawn range highlights.

## Scope

In scope:

- Seed persistent board-cell spawn highlight state from each public `PlayerSnapshot.spawn_range_cells` value during `S2CGameSnapshot` rebuild.
- Consume live `ResolutionEvent::SpawnRangeChanged { player_id, new_spawn_range_cells }` entries from the existing ordered `S2CResolutionEvent` drain.
- Update persistent `SpawnHighlightState` on `BoardCellNode` entities and recolor board-cell sprites so the unlocked spawn cells remain visually active in later DRAFT/PLACEMENT frames.
- Preserve the separation between objective destruction visuals and spawn range state: `ObjectiveDestroyed` alone never changes highlights.

---

## Acceptance Criteria

- [x] **Snapshot seed**: Given `S2CGameSnapshot` includes `PlayerSnapshot.spawn_range_cells` for both players, when Board Rendering rebuilds the board, then every `BoardCellNode` has `SpawnHighlightState` matching the valid cells for the corresponding player side.

- [x] **Live update consumption**: Given an ordered `S2CResolutionEvent` contains `SpawnRangeChanged { player_id, new_spawn_range_cells }`, when Board Rendering drains the resolution event, then only that player's cell-node highlight states update.

- [x] **Persistence**: Given spawn range highlights update during RESOLUTION, when the board transitions to DRAFT/PLACEMENT, then the updated highlights persist and the newly unlocked cells render with the spawn-active visual state.

- [x] **Objective event separation**: Given `ObjectiveDestroyed { was_fake: true }` arrives without a matching `SpawnRangeChanged`, then Board Rendering performs the objective reveal/clear behavior but does not change spawn highlights.

- [x] **OBJMISS behavior**: Given no objective entity exists when `ObjectiveDestroyed` is processed, then no objective entity is spawned, no panic occurs, and spawn highlights change only if a separate `SpawnRangeChanged` entry exists.

- [x] **No replicated component**: Board Rendering does not query or register a `SpawnRange` replicated component.

---

## Implementation Notes

Add or finish the presentation state described in the GDD:

```rust
struct SpawnHighlightState {
    player_id: PlayerId,
    in_spawn_range: bool,
}
```

Current client code already has a persistent `SpawnHighlightState` component on `BoardCellNode` entities as `Inactive` / `ValidSpawn`. Reuse or extend that component shape as needed so each board cell can represent the correct player-side spawn range without adding authoritative gameplay state to the client.

Snapshot rebuild computes the initial state from each public `PlayerSnapshot.spawn_range_cells`. Resolution-message drain applies `SpawnRangeChanged` by recomputing affected node state from `new_spawn_range_cells`.

The update path should live in the existing Board Rendering message-drain/resolution-event path. Do not add a duplicate `MessageReceiver<S2CPhaseChanged>` or any second drain for a Lightyear message that already has a single owner.

## Performance Budget

- Presentation steady-state work from spawn range highlights must remain under 1 ms/frame.
- Snapshot rebuild, phase-boundary, or live-update spike attributable to highlight refresh must remain under 3 ms.
- This story does not close total browser/WASM frame-time evidence; that remains in the separate Board Rendering evidence path.

## Out of Scope

- Protocol schema, server event production, or Board/Lane spawn range authority.
- Final browser frame-time, atlas, status-icon, spawn-highlight screenshot, or art evidence.
- Hand UI drag-time placement highlights and pre-validation.
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
- Deferred visual evidence path, created by the later final board evidence story when visual capture is required: `production/qa/evidence/board-rendering-spawn-range-highlights-evidence.md`

**Status**: [x] Unit/integration support created and verified on 2026-05-05. Final visual/browser evidence remains deferred to the later final Board Rendering evidence path and BOARD-012 browser/WASM perf evidence.

---

## Dependencies

- Depends on: `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md` (Complete - NP-006 shared `ResolutionEvent::SpawnRangeChanged` exists).
- Depends on: `production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md` (Complete - BLS-012 supplies `SpawnRangeState`, snapshot `spawn_range_cells`, and live `SpawnRangeChanged` production).
- Supporting status: `production/epics/board-rendering/story-009-status-icons-cooccupancy-and-spawn-range.md` is Complete for the narrowed status-icon/co-occupancy scope only; final status-icon and spawn-highlight visual evidence remains separate.
- Unlocks: final Board Rendering evidence story covering status-icon atlas, spawn-highlight visual evidence, and browser frame-time evidence.

---

## Completion Notes

Completed: 2026-05-05

Verdict: COMPLETE WITH NOTES

Criteria: 6/6 passing. Snapshot seeding, live `ResolutionEvent::SpawnRangeChanged` consumption, persistence through DRAFT/PLACEMENT, objective-event separation, OBJMISS behavior, and absence of any replicated `SpawnRange` component were verified.

Implementation: Worker branch `work/board-rendering-011-spawn-range-highlights` commit `ec7d0abc46e0d1a840b7347400cb9d6487bc2cf0` was fast-forwarded into `main`. Board Rendering now keeps persistent `SpawnHighlightState` on board cell nodes, seeds it from `PlayerSnapshot.spawn_range_cells`, updates it from ordered resolution-log `SpawnRangeChanged` entries, and leaves Hand UI drag-time ghost/highlight behavior separate.

Test Evidence: `cargo test -p client --test board_rendering_spawn_range_highlights_test --test board_rendering_grid_camera_test --test board_rendering_plugin_scaffold_test --test board_rendering_snapshot_spawn_test --test board_rendering_status_icons_test --test board_rendering_placement_reveal_test --test board_rendering_ghost_preview_bridge_test` passed 36/36 after integration. `cargo fmt -p client -- --check` and `cargo check -p client` passed.

Notes: No blocking GDD, ADR-011, ADR-020, ADR-021, Bevy 0.18, or Lightyear transport deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent. BOARD-012 browser/WASM perf evidence, final Board Rendering visual evidence, traps, final VFX, server spawn range authority, `design/assets/**`, and `AGENTS.md` were not implemented, closed, or touched.

Sprint status: `production/sprint-status.yaml` contains the matching BR-011 story row; that row was set to `done` with `completed: "2026-05-05"` because BLS-012 is complete and BR-011 is now verified.
