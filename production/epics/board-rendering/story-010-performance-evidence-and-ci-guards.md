# Story 010: Performance Evidence and CI Guards

> **Epic**: Board Rendering
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-003`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story adds the non-gameplay guardrails that keep Board Rendering maintainable: Z literal checks, single-drain checks, atlas-count evidence, and browser/WASM performance captures for the visible board path.

## Acceptance Criteria

- [ ] CI or local guard fails if inline Z literals are introduced in board rendering spawn code.
- [ ] CI or local guard verifies `MessageReceiver<S2CPhaseChanged>` is not drained by Board Rendering.
- [ ] A worst-case board state is captured in browser/WASM performance evidence.
- [ ] Board frame time evidence stays within the GDD target or records a blocker with concrete remediation.
- [ ] Atlas handle count evidence shows expected unit atlas plus board-elements atlas usage, excluding approved standalone exceptions.
- [ ] Screenshot evidence confirms the board is nonblank and framed at target desktop viewport.

## Implementation Notes

- This is a documentation/guardrail story, not a visual feature story.
- If final art bible changes atlas count, update the draw-call budget note with technical-director approval.
- Keep evidence files under `production/qa/evidence/` and reference the exact build/test command used.

## Out of Scope

- Implementing missing rendering features from earlier stories.
- Solving art pipeline atlas splits.
- Browser automation infrastructure beyond the lightweight evidence needed for this epic.

## QA Test Cases

- **Z guard**
  - Given: board rendering source
  - When: the guard scan runs
  - Then: no inline Z literals are accepted outside named constants or derived local offsets.

- **Single phase drain**
  - Given: client source
  - When: `MessageReceiver<S2CPhaseChanged>` is scanned
  - Then: only the shared phase sink owns it.

- **WASM visual performance**
  - Given: a worst-case board state in browser
  - When: performance capture is recorded
  - Then: frame timing and draw-call observations are documented with pass/fail status.

## Test Evidence

**Required evidence**:
- Config/Data: CI or local guard notes in `production/qa/evidence/board-rendering-performance-evidence.md`
- Browser screenshot/performance evidence for a populated board.

**Status**: [ ] Not yet created

## Dependencies

- Depends on: Stories 002 and 003 for a visible board; Story 009 for final status icon atlas count if included.
- Unlocks: Board Rendering epic closure.
