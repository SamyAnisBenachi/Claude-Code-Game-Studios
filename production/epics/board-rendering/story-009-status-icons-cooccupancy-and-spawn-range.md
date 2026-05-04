# Story 009: Status Icons, Co-Occupancy, and Spawn Range

> **Epic**: Board Rendering
> **Status**: Partial
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-006`, `TR-BR-007`
**ADR Governing Implementation**: [ADR-018: Keyword System](../../../docs/architecture/adr-018-keyword-system.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story covers persistent status icon rendering, co-occupancy offsets, overflow badges, OUTNUMBERED indicators, and spawn range highlight updates. It is partially blocked by keyword display definitions and replicated spawn range source availability.

## Partial Blockers

- Keyword display data must expose display tier and remaining-duration ordering without hardcoding keyword names.
- Spawn range must have a replicated or snapshot-visible source for the client; the GDD decision points to a `SpawnRange` component but current protocol/code must confirm it.
- Trap face-down rendering remains blocked on the Network Protocol per-client visibility question.

## Acceptance Criteria

- [ ] A unit with 1..=3 active status effects renders one icon per effect.
- [ ] A unit with more than 3 active effects renders the top 3 plus one overflow badge.
- [ ] Tier-1 effects always outrank Tier-2 effects regardless of insertion order.
- [ ] Tier-2 effects sort by descending remaining duration within tier.
- [ ] Status icons inherit co-occupancy X offset from the unit parent.
- [ ] Icons use the board-elements atlas, not a third atlas.
- [ ] OUTNUMBERED indicator appears per unit according to the final keyword state contract.
- [ ] Spawn range highlights update from the authoritative replicated/snapshot value.

## Implementation Notes

- Do not hardcode names like `SHIELDED` to infer priority. Read display tier from keyword definitions.
- Co-occupancy offset must affect children through hierarchy, not by re-centering on the cell.
- Status effects are visual state only; client must not run gameplay keyword logic.
- This story can be split during story-readiness if keyword display and spawn range land at different times.

## Out of Scope

- Keyword gameplay implementation.
- Trap identity protocol work.
- Final art production for icons.

## QA Test Cases

- **Tier priority**
  - Given: one Tier-1 effect and three Tier-2 effects inserted in different orders
  - When: status icon update runs
  - Then: Tier-1 occupies slot 0 in every ordering.

- **Overflow badge**
  - Given: four active effects
  - When: icon update runs
  - Then: three icons plus one `+1` overflow badge exist.

- **Co-occupancy**
  - Given: a unit with co-occupancy X offset
  - When: status icon position is computed
  - Then: icon world X includes the unit offset.

## Test Evidence

**Required evidence**:
- Visual/Feel: `production/qa/evidence/board-rendering-status-icons-evidence.md`
- Unit/integration support: `tests/unit/board_rendering/status_icons_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md), keyword display definitions, spawn range replication/source.
- Unlocks: Board legibility polish and performance evidence.
