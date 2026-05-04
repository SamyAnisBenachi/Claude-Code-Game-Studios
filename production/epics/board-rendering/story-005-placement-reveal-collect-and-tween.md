# Story 005: Placement Reveal Collect and Tween

> **Epic**: Board Rendering
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-001`
**ADR Governing Implementation**: [ADR-017: Combat Resolution Execution Architecture](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

At RESOLUTION entry, both players' newly committed placements must appear as one simultaneous reveal beat. Board Rendering collects the newly visible replicated units, emits `PlacementRevealAnimReady`, and Card Animations applies the parallel scale/alpha tween.

## Acceptance Criteria

- [ ] `S2CPlacementReveal` starts a one-frame collect window for newly replicated placement entities.
- [ ] Board Rendering emits one `PlacementRevealAnimReady` message per reveal batch.
- [ ] Reveal entries are sorted by lane then cell for deterministic tests.
- [ ] All reveal entries start their tween in the same animation pass.
- [ ] Reveal duration stays within the GDD PLACEMENT/RESOLUTION reveal budget.
- [ ] If a reveal script arrives but no corresponding entities appear before timeout, Board Rendering requests snapshot recovery once `C2SRequestSnapshot` exists.

## Implementation Notes

- This story uses Card Animations; it does not implement the tween lens itself.
- Board Rendering owns collection and entity targeting. Card Animations owns animation mechanics.
- Use `Time<Virtual>` for timeout tests.
- `C2SRequestSnapshot` recovery is documented here but remains blocked until Story 007's protocol gate is clear.

## Out of Scope

- Full RESOLUTION sub-step queue playback (Story 006).
- Protocol implementation for `C2SRequestSnapshot` (outside this presentation epic).
- Final VFX polish from the art bible.

## QA Test Cases

- **Simultaneous reveal**
  - Given: five newly revealed entities across five lanes
  - When: `S2CPlacementReveal` is processed
  - Then: one `PlacementRevealAnimReady` contains all five entries and Card Animations can start them together.

- **Deterministic order**
  - Given: reveal entities are discovered in arbitrary query order
  - When: entries are emitted
  - Then: entries are sorted ascending by lane then cell.

- **Stuck reveal**
  - Given: `S2CPlacementReveal` arrives but matching entities do not appear before `resolution_reveal_timeout_ms`
  - When: timeout elapses
  - Then: recovery path logs a warning and, after protocol support exists, enqueues `C2SRequestSnapshot`.

## Test Evidence

**Required evidence**:
- Visual/Feel: `production/qa/evidence/board-rendering-placement-reveal-evidence.md`
- Integration support: `tests/integration/board_rendering/placement_reveal_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md), Card Animations placement reveal messages.
- Unlocks: Story 006.
