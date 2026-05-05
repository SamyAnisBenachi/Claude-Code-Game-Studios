# Story 002: Board Grid, Camera, and Z Layers

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-003`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story creates the visible world-space board shell: fixed orthographic camera, 5 x 8 cell nodes, named Z constants, and spawn range tint targets. It proves that the board is a world-space sprite surface below bevy_ui before units or messages are introduced.

## Acceptance Criteria

- [x] Exactly one `Camera2d` entity exists for the board view.
- [x] The camera has an orthographic projection and no perspective projection.
- [x] Cell node entities are spawned for all 40 lane/cell coordinates.
- [x] Every cell node carries a `LaneCell { lane, cell }` marker.
- [x] Z values are named constants in `rendering_constants.rs`; no inline Z literals are used in board spawn functions.
- [x] Board cell nodes use world-space `Sprite` plus `Transform`, not bevy_ui `Node`.
- [x] Spawn highlight state is represented by sprite tint/state, not a separate Z layer.

## Implementation Notes

- Required Z constants from the GDD: `Z_FIELD_WASH`, `Z_CELL_NODES`, `Z_TRAPS_STRUCTURES`, `Z_OBJECTIVES`, `Z_UNITS`, `Z_HEALTH_BARS`, `Z_GHOST_UNIT`.
- Health bar Z is world-space 3.1 but child local Z must be 0.1 when parent unit is at 3.0. This story defines the constants; Story 003 uses the local child offset.
- Use `Sprite::from_color` for solid-color placeholder sprites in Bevy 0.18. `Sprite { color, ..default() }` without an image is invisible.
- Keep the camera fixed for M2. Pan/zoom is out of scope unless a later UX spec explicitly changes it.

## Out of Scope

- Unit/objective/HP spawning (Story 003).
- Ghost preview and spawn range interaction with Hand UI (Story 004).
- Performance evidence for final art (Story 010).

## QA Test Cases

- **Camera projection**
  - Given: `BoardRenderingPlugin` is registered and `app.update()` has run
  - When: the camera query is inspected
  - Then: one entity has `Camera2d`, `OrthographicProjection` is present, no perspective projection exists, and camera Z is positive.

- **Grid completeness**
  - Given: session entry completed
  - When: querying `LaneCell` markers
  - Then: 40 unique lane/cell pairs exist, covering lanes 1..=5 and cells 1..=8.

- **Z literal guard**
  - Given: board rendering source files
  - When: CI scans board spawn code
  - Then: Transform Z assignments use named constants or derived local offsets only.

## Test Evidence

**Required evidence**:
- Logic: `tests/unit/board_rendering/board_grid_camera_test.rs`
- CI guard or lint note for Z constants.

**Status**: [x] Complete - `cargo test -p client --test board_rendering_grid_camera_test` passed 6/6 on 2026-05-05.

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-board-layout-card-atlas.md).
- Unlocks: Story 003 and Story 004.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 7/7 passing.
**Implementation**: Worker commit `6ae82d7`; main integration commit `ce3658b`.
**Deviations**:
- Advisory: Story manifest version is 2026-05-01 while the current control manifest is 2026-05-05.
- Advisory: Active TR-BR-003 also mentions BoardPosition replication; this remains outside the declared Story 002 grid/camera/Z-layer scope.
**Test Evidence**: Logic evidence at `tests/unit/board_rendering/board_grid_camera_test.rs`; `cargo test -p client --test board_rendering_grid_camera_test` passed 6/6. Adjacent scaffold regression `cargo test -p client --test board_rendering_plugin_scaffold_test` passed 9/9. `cargo fmt -p client -- --check` and `cargo check -p client` passed.
**Code Review**: Skipped - lean mode.
