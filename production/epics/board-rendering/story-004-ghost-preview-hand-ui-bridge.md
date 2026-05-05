# Story 004: Ghost Preview and Hand UI Bridge

> **Epic**: Board Rendering
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement Trace**:
- Primary GDD trace: Board Rendering Rule 8, "Ghost unit lifecycle", and ACs `BR-8`, `BR-8b`, `BR-8c`, `BR-8d`, `BR-8e`, `BR-10`, and `BR-11`.
- Supporting active TRs: `TR-BR-003` (Z-layer constants, including `Z_GHOST_UNIT`), `TR-HU-002` (PLACEMENT drag-to-stage and `GhostPlacementChanged` bridge), `TR-HU-003` (Instant staging sends `GhostPlacementChanged { target: Some(Instant) }`), and `TR-HU-006` (TargetUnit placement highlight/hover behavior upstream of board ghost rendering).
- Supporting only: `TR-BR-002` covers `BoardLayout` / `BR-2` coordinate authority. This story consumes `BoardLayout` but does not claim `TR-BR-002` as the ghost lifecycle requirement.

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

Hand UI owns card drag, placement target discovery, staging state, and submit ownership. Board Rendering owns board-space ghost previews and the reverse board-ghost gestures that Hand UI consumes for un-staging. This story connects the existing Hand UI ghost messages to Board Rendering visuals without sending any Lightyear or C2S gameplay messages.

Spawn range highlights are not part of this story. Story 009 owns persistent spawn range highlight updates from the authoritative replicated/snapshot source. Hand UI Story 006 already owns drag-time `BoardCellHighlighted` placement highlights.

## Acceptance Criteria

- [ ] Board Rendering reads `GhostPlacementChanged` as a Bevy-internal `MessageReader<GhostPlacementChanged>` message, not as a Lightyear `MessageReceiver`.
- [ ] `Some(PlayTarget::BoardCell { lane, cell })` spawns or moves exactly one `GhostUnit` for that `card_id` at `BoardLayout.cell_to_world(lane, cell)` with `Transform.translation.z == Z_GHOST_UNIT`.
- [ ] Re-staging the same `card_id` replaces or moves the existing ghost; after deferred commands flush, exactly one `GhostUnit` exists for that `card_id`.
- [ ] `GhostUnit` is a world-space `Sprite` entity with `Sprite.color` alpha `0.5`, no HP bar child, no status indicator child, and no Lightyear replication component.
- [ ] `Some(PlayTarget::TargetUnit { lane, unit_id })` applies a `TargetUnitGhost` marker to the matching unit entity and spawns no new board entity.
- [ ] `Some(PlayTarget::TargetObj { player_id, lane })` applies an `ObjectiveTargetGhost` marker to the matching objective entity and spawns no new unit ghost.
- [ ] `Some(PlayTarget::LaneWide { lane })` spawns or moves exactly one `LaneGhostWash` for that `card_id` covering the target lane column.
- [ ] `Some(PlayTarget::Instant)` is a Board Rendering no-op: no board ghost entity is spawned and no board ghost marker is added.
- [ ] `target: None` clears all ghost entities and marker components for the corresponding `card_id`; if none exist, the clear is a no-op with no panic.
- [ ] On `S2CPlacementReveal`, Board Rendering clears all ghost entities and marker components immediately before or during the reveal handoff so real replicated entities are the only placement visuals after reveal.
- [ ] Clicking any board ghost variant writes exactly one `GhostClickedEvent { card_id }` Bevy-internal message and does not directly remove the ghost.
- [ ] Mouse-down on any board ghost variant writes exactly one `GhostDragStartEvent { card_id }` Bevy-internal message and leaves staging ownership with Hand UI.
- [ ] Board Rendering does not add, remove, or recompute `BoardCellHighlighted` spawn range / placement highlight markers in this story.
- [ ] Board Rendering never sends `C2SSubmitPlacement` or any other placement submit message; Hand UI remains the submit-message owner.

## Control Manifest Rules

- Manifest reviewed against `docs/architecture/control-manifest.md` version `2026-05-05`.
- Board ghosts are Presentation-layer world-space sprites: `Sprite` + `Transform` with `Camera2d`, never bevy_ui `Node`.
- Hand drag previews remain bevy_ui `Node` entities owned by Hand UI; this story only renders the already-staged board preview.
- Use Bevy 0.18 message APIs: `#[derive(Message)]`, `MessageReader<T>`, `MessageWriter<T>`, and `app.add_message::<T>()`; do not use `EventReader`, `EventWriter`, or `Events<T>`.
- Use Bevy Required Components API; do not use `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, or `Handle<TextureAtlas>`.
- Use `Color::srgba` / `with_alpha` paths for ghost alpha. Do not use removed `Color::rgba`.
- When clearing ghost entities, use the Bevy 0.18 `commands.get_entity(entity)` `Result` form before `despawn()`.
- `BoardLayout` and `CardAtlas` are session-scoped resources and systems reading them must run only while the client is in-session.

## Implementation Notes

- `GhostPlacementChanged`, `GhostClickedEvent`, and `GhostDragStartEvent` are Bevy messages, not Lightyear protocol messages.
- Keep the bridge one-way for visuals: Hand UI emits placement intent; Board Rendering visualizes it.
- Reverse board-ghost gestures are still Hand UI-owned state transitions: Board Rendering only writes `GhostClickedEvent` / `GhostDragStartEvent`; Hand UI clears or restores the staged state by writing `GhostPlacementChanged`.
- Validity display must not become authoritative. Server placement validation remains the source of truth.
- Use the current `shared::protocol::PlayTarget` variants as the variant matrix for ghost behavior.
- Do not depend on `SpawnRange` replication/source availability in this story.

## Performance Notes

- The ghost bridge should be message-driven and idle when no `GhostPlacementChanged`, click, or drag-start input arrives.
- BoardCell ghost placement maps one lane/cell through `BoardLayout`; it must not run a 40-cell placement-highlight loop.
- The story may add at most the permitted ghost translucent batch described by Board Rendering Rule 5 and must not introduce a third atlas or per-ghost material.
- Keep the PLACEMENT update path within ADR-021's Presentation steady-state budget: less than 1 ms per frame in steady state, with no spawn/despawn churn outside ghost message changes.

## Out of Scope

- Card drag implementation in Hand UI.
- Drag-time placement highlight computation (`BoardCellHighlighted`, `TargetUnitHover`, and no-target overlays).
- Persistent spawn range highlight updates from replicated/snapshot state; owned by Story 009.
- Placement batch submission to the server.
- Placement reveal after server acceptance (Story 005).

## QA Test Cases

- **Ghost lifecycle**
  - Given: a staged valid `PlayTarget::BoardCell` message
  - When: Board Rendering processes the message
  - Then: one `GhostUnit` entity for that card is visible at `BoardLayout.cell_to_world(lane, cell)` with `Z_GHOST_UNIT`, alpha `0.5`, no HP bar, and no replication component.

- **Ghost clear**
  - Given: a ghost is visible
  - When: Hand UI emits a clear/deselect message
  - Then: all ghost entities and marker components for that `card_id` are cleared, and no `C2SSubmitPlacement` is sent by Board Rendering.

- **Variant matrix**
  - Given: `GhostPlacementChanged` is emitted with each `PlayTarget` variant
  - When: Board Rendering processes the message
  - Then: BoardCell creates `GhostUnit`, TargetUnit marks the unit, TargetObj marks the objective, LaneWide creates `LaneGhostWash`, and Instant creates no board ghost.

- **Reverse ghost events**
  - Given: a board ghost exists for a staged card
  - When: the player clicks or mouse-downs on the ghost
  - Then: Board Rendering writes `GhostClickedEvent` or `GhostDragStartEvent` respectively and leaves actual un-staging to Hand UI.

- **Reveal cleanup**
  - Given: one or more ghosts exist
  - When: `S2CPlacementReveal` is received
  - Then: all ghost entities and marker components are removed before the real placement visuals become the only board-space representation.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
- Screenshot or manual evidence once UI interaction exists.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-board-layout-card-atlas.md), [Story 002](story-002-board-grid-camera-and-z-layers.md), [Hand UI Story 005](../hand-ui/story-005-placement-submit-core.md), [Hand UI Story 006](../hand-ui/story-006-placement-drag-highlights.md), [Hand UI Story 007](../hand-ui/story-007-placement-instant-staging.md), and [Hand UI Story 008](../hand-ui/story-008-placement-unstaging.md).
- Dependency source: `GhostPlacementChanged`, `GhostClickedEvent`, and `GhostDragStartEvent` are registered in Hand UI; `PlayTarget` variants are defined in `shared::protocol`.
- Not a dependency: Story 009 spawn range replication/source work, because spawn range highlight updates are out of scope here.
- Unlocks: Story 005 and Hand UI placement polish.
