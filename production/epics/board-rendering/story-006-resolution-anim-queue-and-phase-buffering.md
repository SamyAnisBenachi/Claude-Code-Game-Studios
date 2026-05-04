# Story 006: Resolution AnimQueue and Phase Buffering

> **Epic**: Board Rendering
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-001`, `TR-BR-004`
**ADR Governing Implementation**: [ADR-017: Combat Resolution Execution Architecture](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story converts `S2CResolutionEvent` into grouped animation steps and buffers phase changes that arrive while RESOLUTION playback is still running. It is blocked until Combat Resolution provides stable `ResolutionEvent` variants and ordering for movement, attack, death, trap, prism, and objective events.

## Blocker

`ResolutionEvent` variant coverage is not stable enough for full dispatch. Queue grouping and phase buffering can be unit-tested with opaque sub-step fixtures, but production playback should not start until the Combat Resolution story for the final event log contract is complete.

## Acceptance Criteria

- [ ] `S2CResolutionEvent` is grouped into `AnimGroup`s by sub-step.
- [ ] Groups are sorted ascending by sub-step.
- [ ] Out-of-range sub-step values trigger desync recovery instead of partial playback.
- [ ] `PendingResolutionScript` buffers a resolution script that arrives before placement reveal is complete.
- [ ] `PendingPhaseChange` buffers `S2CPhaseChanged` while `BoardRenderState == ResolutionExecuting`.
- [ ] Phase buffer is applied only after the animation queue drains.
- [ ] Same-frame `S2CResolutionEvent` and `S2CPhaseChanged(DRAFT_SHOP)` preserves the resolution playback before DRAFT UI resumes.

## Implementation Notes

- Use `Time<Virtual>` timers so queue advancement can be tested headlessly.
- `AnimQueue` should be a resource, not a sentinel component pattern.
- Board Rendering schedules animation requests; Card Animations owns tween execution.
- `S2CResolutionEvent` must remain on ReliableChannel and must not be streamed per sub-step.

## Out of Scope

- Combat resolution server event generation.
- Individual VFX details for every keyword.
- Objective destruction HUD fanout (Story 008).

## QA Test Cases

- **Queue grouping**
  - Given: events with sub-steps 3, 1, 1, and 5
  - When: queue construction runs
  - Then: three groups exist in order 1, 3, 5 and group 1 contains both sub-step 1 events.

- **Phase buffering**
  - Given: `ResolutionExecuting` with a non-empty queue
  - When: `S2CPhaseChanged(DRAFT_SHOP)` arrives
  - Then: the phase is stored in `PendingPhaseChange` and not applied until queue drain.

- **Out-of-range event**
  - Given: an event with sub-step 9
  - When: intake validates the resolution script
  - Then: playback is rejected and recovery is requested/logged.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/board_rendering/resolution_anim_queue_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 005](story-005-placement-reveal-collect-and-tween.md), final Combat Resolution `ResolutionEvent` contract.
- Unlocks: Story 008 and RESOLUTION visual QA.
