# Story 006: Resolution AnimQueue and Phase Buffering

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement Trace**:
- Primary trace: Board Rendering Rule 9, AC `BR-12`, and `TR-BR-001` require `S2CResolutionEvent` intake to partition the flat replay log into `AnimGroup`s by `sub_step`, sort groups ascending, and advance playback with `Time<Virtual>` timers.
- Phase-buffer trace: Rule 10, AC `BR-14`, AC `BR-15`, and `TR-BR-005` require `PendingPhaseChange` to buffer `S2CPhaseChanged(DRAFT_SHOP)` or `S2CPhaseChanged(GAME_OVER)` while resolution playback is active, then apply it only after the queue and objective reveal phase finish.
- Early-script trace: edge cases `EC-REVEAL-WAIT`, `EC-PLACEMENT-STUCK`, `BR-EC-EARLY`, `BR-EC-EARLY-CONSUME`, and `BR-EC-PLACEMENT-STUCK` require `PendingResolutionScript` to buffer a resolution script that arrives before placement reveal completes and to request snapshot recovery if reveal never arrives.
- Desync trace: edge case `EC-SUBSTEP-OOR` and AC `BR-24` require any out-of-range `sub_step` outside `[1, 6]` to discard the entire queue, log protocol desync, and enqueue `C2SRequestSnapshot`.
- Supporting visual trace: `TR-BR-004` covers WALL/collision visual dispatch during SS5 movement playback. This story creates the resolution queue and phase buffering path that later per-variant visual handlers consume.

**ADR Governing Implementation**: [ADR-017: Combat Resolution Execution Architecture](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story converts `S2CResolutionEvent` into grouped animation steps and buffers phase changes that arrive while RESOLUTION playback is still running.

## Dependency Revalidation

**Rechecked**: 2026-05-05

The prior blocker on final `ResolutionEvent` contract coverage is resolved. Combat Resolution Story 011 is Complete and documents CR-32 / `TR-CR-015` coverage for a single complete `S2CResolutionEvent` batch, chronological `(sub_step, trigger_index)` ordering, typed replay categories, and `S2CResolutionEvent` before `S2CPhaseChanged(DRAFT_SHOP)` delivery. Board Rendering Story 005 is also Complete and unlocks this story's placement-reveal-to-resolution handoff.

## Acceptance Criteria

- [x] `S2CResolutionEvent` is grouped into `AnimGroup`s by sub-step.
- [x] Groups are sorted ascending by sub-step.
- [x] Out-of-range sub-step values discard the entire queue, log protocol desync, and enqueue one `C2SRequestSnapshot` recovery request instead of partial playback.
- [x] `PendingResolutionScript` buffers a resolution script that arrives before placement reveal is complete.
- [x] `PendingPhaseChange` buffers `S2CPhaseChanged` while `BoardRenderState == ResolutionExecuting`.
- [x] Phase buffer is applied only after the animation queue drains.
- [x] Same-frame `S2CResolutionEvent` and `S2CPhaseChanged(DRAFT_SHOP)` preserves the resolution playback before DRAFT UI resumes.

## Control Manifest Rules

- Manifest reviewed against `docs/architecture/control-manifest.md` version `2026-05-05`.
- `S2CResolutionEvent` and `S2CPhaseChanged(DRAFT_SHOP)` must both remain on `ReliableChannel`; never split them across channels.
- `S2CResolutionEvent` is a single reliable broadcast sent after all 6 combat sub-steps complete. Board Rendering replays the batch locally at animation tempo and must not expect streamed per-sub-step S2C messages.
- Board Rendering is client-side presentation only. It may enqueue `C2SRequestSnapshot` for desync/stuck-state recovery, but it must not mutate authoritative game state or send C2S game-logic messages.
- `MessageReceiver<S2CPhaseChanged>` is drained only by the shared presentation phase sink. Board Rendering phase logic reads `CurrentClientPhase` and uses `PendingPhaseChange` for resolution playback gating.
- `BoardLayout`, `CardAtlas`, and animation queue systems are session-scoped presentation resources and must run only while the client is in-session.
- Use Bevy 0.18 message APIs for intra-client handoff: `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`, and `app.add_message::<T>()`; do not use removed `EventReader`, `EventWriter`, or `Events<T>`.
- Use Bevy Required Components API and ADR-021 sprite/atlas patterns; do not use `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, or `Handle<TextureAtlas>`.

## Implementation Notes

- Use `Time<Virtual>` timers so queue advancement can be tested headlessly.
- `AnimQueue` should be a resource, not a sentinel component pattern.
- Board Rendering schedules animation requests; Card Animations owns tween execution.
- `S2CResolutionEvent` must remain on ReliableChannel and must not be streamed per sub-step.
- `S2CResolutionEvent.events` is now a vector of `TaggedEvent { sub_step, trigger_index, event }`; queue grouping should use the outer `TaggedEvent.sub_step` as authoritative and reject values outside `[1, 6]`.
- `PendingResolutionScript` is cleared on consumption after `S2CPlacementReveal` starts the reveal handoff. It remains intact when `BR-EC-PLACEMENT-STUCK` requests a snapshot because snapshot receipt owns state reset.
- `PendingPhaseChange` is last-write-wins if multiple phase messages arrive during playback. `GAME_OVER` may skip remaining animation groups after the current group, but still runs objective reveal before entering final board state per `BR-15`.

## Performance Notes

- Queue construction is linear in the event count plus sorting of at most 6 distinct sub-step groups; no steady-state work should run when no resolution batch is pending.
- Group timers use integer millisecond config values and `Time<Virtual>` for deterministic headless tests.
- Keep the resolution intake and phase-buffer systems within ADR-021's Presentation budget: less than 1 ms steady-state and less than 3 ms on phase-boundary or queue-build frames.

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

**Status**: [x] Created and passing

## Dependencies

- Depends on: [Story 005](story-005-placement-reveal-collect-and-tween.md) Complete for the placement reveal collect/recovery path and `PendingResolutionScript` handoff.
- Depends on: [Combat Resolution Story 011](../combat-resolution/story-011-resolution-event-log.md) Complete for final `S2CResolutionEvent` schema, typed `ResolutionEvent` coverage, and phase-ordering contract.
- Unlocks: Story 008 and RESOLUTION visual QA.

## Completion Notes

**Completed**: 2026-05-06
**Criteria**: 7/7 passing
**Deviations**: None blocking. Story manifest version `2026-05-05` matches the current control manifest. Lean review mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW because `production/review-mode.txt` is absent.
**Test Evidence**: Integration test at `tests/integration/board_rendering/resolution_anim_queue_test.rs`; registered as `board_rendering_resolution_anim_queue_test` in `client/Cargo.toml`.
**Verification**: `cargo test -p client --test board_rendering_resolution_anim_queue_test` passed 5/5. Relevant board rendering/card animation regressions passed: `board_rendering_placement_reveal_test` 3/3, `board_rendering_snapshot_spawn_test` 5/5, `board_rendering_spawn_range_highlights_test` 4/4, `card_animations_anim_queue_test` 4/4, and `card_animations_objective_stagger_test` 3/3.
**QA-COND-0007 Impact**: BR-006 provides the replay queue and phase-buffering prerequisite for future resolution replay readability evidence. QA-COND-0007 remains Open; no manual playable-client QA or resolution readability capture is claimed.
**Code Review**: Skipped per lean review mode.
