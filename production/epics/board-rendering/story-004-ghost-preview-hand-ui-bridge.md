# Story 004: Ghost Preview and Hand UI Bridge

> **Epic**: Board Rendering
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-002`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

Hand UI owns card drag and staging input. Board Rendering owns the world-space ghost unit and spawn highlights that make the staged placement legible. This story connects the existing Hand UI ghost messages to board-space visuals without sending any network messages.

## Acceptance Criteria

- [ ] Board Rendering reads the Bevy-internal ghost/staging messages from Hand UI.
- [ ] A single ghost unit preview is shown at the mapped lane/cell while a valid placement is staged or hovered.
- [ ] Ghost preview uses `Z_GHOST_UNIT` and does not appear as bevy_ui.
- [ ] Invalid or cleared ghost messages hide the ghost without despawning steady-state board resources unnecessarily.
- [ ] Spawn range highlights update from replicated/configured spawn range state and `BoardLayout`.
- [ ] Board Rendering never sends `C2SSubmitPlacement`; Hand UI remains the submit-message owner.

## Implementation Notes

- `GhostPlacementChanged`, `GhostClickedEvent`, and related existing Hand UI messages are Bevy messages, not Lightyear protocol messages.
- Keep the bridge one-way for visuals: Hand UI emits placement intent; Board Rendering visualizes it.
- Validity display must not become authoritative. Server placement validation remains the source of truth.
- If `SpawnRange` replication is not yet available, scope this story to ghost lifecycle and mark spawn highlight updates as pending follow-up under Story 009.

## Out of Scope

- Card drag implementation in Hand UI.
- Placement batch submission to the server.
- Placement reveal after server acceptance (Story 005).

## QA Test Cases

- **Ghost lifecycle**
  - Given: a staged valid lane/cell message
  - When: Board Rendering processes the message
  - Then: one ghost entity is visible at `BoardLayout.cell_to_world(lane, cell)` with `Z_GHOST_UNIT`.

- **Ghost clear**
  - Given: a ghost is visible
  - When: Hand UI emits a clear/deselect message
  - Then: the ghost becomes hidden and no `C2SSubmitPlacement` is sent by Board Rendering.

- **Spawn highlight source**
  - Given: spawn range state changes
  - When: highlight update runs
  - Then: only cells inside the local player's legal spawn band are tinted active.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
- Screenshot or manual evidence once UI interaction exists.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-board-layout-card-atlas.md), [Story 002](story-002-board-grid-camera-and-z-layers.md), Hand UI ghost message definitions.
- Unlocks: Story 005 and Hand UI placement polish.
