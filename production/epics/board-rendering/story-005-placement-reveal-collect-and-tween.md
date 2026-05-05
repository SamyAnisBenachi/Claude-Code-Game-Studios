# Story 005: Placement Reveal Collect and Tween

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`

**Requirement Trace**:
- Primary GDD trace: Board Rendering Rule 7 and AC `BR-7` require a one-frame collect-then-reveal buffer after `S2CPlacementReveal`, opponent-only reveal tweens, same-frame simultaneous tween start, and deterministic lane/cell audit ordering.
- Animation handoff trace: the `PlacementRevealAnimReady` payload section requires Board Rendering to emit exactly one batch message with `PlacementRevealEntry { unit, lane, cell }` entries; Card Animations consumes that batch in one pass for the flip/squash animation.
- Recovery trace: `EC-REVEAL-WAIT`, `BR-EC-STUCK`, and `BR-EC-PLACEMENT-STUCK` require `Time<Virtual>` timeout handling and `C2SRequestSnapshot` recovery through Network Protocol NP-43.
- Supporting active TRs: `TR-BR-003` covers replicated board position/Z-layer context needed to target visible entities, and `TR-BR-005` covers `PendingResolutionScript` / pending recovery state. No active TR currently maps the Rule 7 placement-reveal collect behavior directly, so this story traces that behavior to the GDD rule and AC instead of the stale `TR-BR-001` AnimQueue requirement owned by Story 006.

**ADR Governing Implementation**: [ADR-017: Combat Resolution Execution Architecture](../../../docs/architecture/adr-017-combat-resolution-execution-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

At RESOLUTION entry, both players' newly committed placements must appear as one simultaneous reveal beat. Board Rendering collects the newly visible replicated units, emits `PlacementRevealAnimReady`, and Card Animations applies the parallel scale/alpha tween.

## Acceptance Criteria

- [ ] `S2CPlacementReveal` starts a one-frame collect window for newly replicated opponent placement entities; local player's already-visible placements are excluded from reveal tween collection.
- [ ] Board Rendering emits one `PlacementRevealAnimReady` message per reveal batch.
- [ ] Reveal entries are sorted by lane then cell for deterministic tests.
- [ ] All reveal entries start their tween in the same animation pass.
- [ ] Reveal duration uses `unit_reveal_tween_duration_ms` (default 250ms, allowed 150-400ms) and completes before `pre_animation_pause_ms` begins.
- [ ] If `S2CPlacementReveal` enters `ResolutionReveal` but `S2CResolutionEvent` does not arrive before `resolution_reveal_timeout_ms`, Board Rendering enqueues one `C2SRequestSnapshot` recovery request per NP-43 and logs a "ResolutionReveal stuck" warning.
- [ ] If `PendingResolutionScript` exists but `S2CPlacementReveal` never arrives before `resolution_reveal_timeout_ms`, Board Rendering enqueues `C2SRequestSnapshot`, logs a "PendingResolutionScript stuck" warning, and leaves the pending script intact until snapshot receipt.

## Control Manifest Rules

- Manifest reviewed against `docs/architecture/control-manifest.md` version `2026-05-05`.
- Board Rendering is client-side presentation only: no authoritative state, no validation, and no C2S game-logic messages. The only allowed C2S message in this scope is `C2SRequestSnapshot` for desync/stuck-state recovery defined by Network Protocol NP-43.
- `S2CPlacementReveal` is distinct from `C2SSubmitPlacement` and must not expose mana-spend fields.
- Use Bevy 0.18 message APIs for intra-client handoff: `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`, and `app.add_message::<T>()`; do not use removed `EventReader`, `EventWriter`, or `Events<T>`.
- Use Bevy Required Components API and ADR-021 sprite/atlas patterns; do not use `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, or `Handle<TextureAtlas>`.
- Board units and reveal visuals stay world-space `Sprite` + `Transform` entities below bevy_ui. Do not render board units or reveal effects as bevy_ui nodes.
- `BoardLayout` and `CardAtlas` are session-scoped resources; systems reading them must run only while the client is in-session.
- Tween cancel/replace must use the ADR-021 `Animator<T>::set_tweenable(new_tween)` pattern where this story touches active reveal tweens; do not despawn and respawn state-bearing board entities to restart a tween.

## Performance Notes

- The collect window is one frame only and should be message-driven from `S2CPlacementReveal`; steady-state PLACEMENT/RESOLUTION frames with no reveal batch should do no reveal sorting or tween setup work.
- Sort only the current reveal batch by lane/cell. Do not scan or reorder unrelated board entities after the batch is collected.
- Keep reveal handoff within ADR-021's Presentation budget: less than 1 ms steady-state, with phase-boundary/reveal setup fitting inside the less than 3 ms Presentation spike guardrail.
- Browser/WASM frame-time evidence for `BR-FRAME-TIME` and reconnect rebuild timing remains Story 010 scope.

## Implementation Notes

- This story uses Card Animations; it does not implement the tween lens itself.
- Board Rendering owns collection and entity targeting. Card Animations owns animation mechanics.
- Use `Time<Virtual>` for timeout tests.
- `C2SRequestSnapshot` is now defined by `design/gdd/network-protocol.md` NP-43. This story only enqueues recovery requests for reveal/stuck states; broader reconnect snapshot rebuild behavior remains Story 007 scope.
- Verify the exact Lightyear 0.26 newly-replicated entity detection API before implementation. The GDD requires opponent ownership filtering in addition to newly-replicated detection so local player's already-visible placements do not reveal-tween.

## Out of Scope

- Full RESOLUTION sub-step queue playback (Story 006).
- Reconnect snapshot rebuild and general desync recovery outside reveal/stuck paths (Story 007).
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
  - Given: `S2CPlacementReveal` arrives but `S2CResolutionEvent` does not arrive before `resolution_reveal_timeout_ms`
  - When: timeout elapses
  - Then: recovery path logs a warning and enqueues `C2SRequestSnapshot`.

## Test Evidence

**Required evidence**:
- Visual/Feel: `production/qa/evidence/board-rendering-placement-reveal-evidence.md`
- Integration support: `tests/integration/board_rendering/placement_reveal_test.rs`

**Status**: [x] Integration support created; visual evidence deferred/advisory

## Dependencies

- Depends on: [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md) Complete for snapshot-spawned board entities and `LaneCell`/HP sprite context; implementation integrated at `c0dc500` and `/story-done` closure is present on current `origin/main` at `f4f529`.
- Depends on: [Story 004](story-004-ghost-preview-hand-ui-bridge.md) Complete at `730f155` for ghost cleanup on `S2CPlacementReveal`.
- Depends on: [Card Animations Story 005](../card-animations/story-005-placement-reveal-parallelism.md) Complete for the `PlacementRevealAnimReady` consumer and same-pass animation handling.
- Not a dependency: Board Rendering Story 007 reconnect rebuild, because this story only enqueues reveal/stuck `C2SRequestSnapshot` requests and does not implement snapshot rebuild.
- Unlocks: Story 006.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 7/7 passing. One-frame opponent-only placement reveal collection, single sorted `PlacementRevealAnimReady` batch, same-pass Card Animations reveal start, `unit_reveal_tween_duration_ms` default/range use, and both stuck-state recovery request paths verified.
**Deviations**: None blocking. Advisory only: the Visual/Feel sign-off file `production/qa/evidence/board-rendering-placement-reveal-evidence.md` is not present yet; integration evidence covers the functional reveal/recovery path.
**Test Evidence**: `cargo test -p client --test board_rendering_placement_reveal_test` passed 3/3; `cargo test -p client --test card_animations_placement_reveal_test` passed 9/9; adjacent board rendering regressions and `cargo test -p shared` passed. `cargo fmt -p client -- --check`, `cargo fmt -p shared -- --check`, `cargo check -p client`, and `git diff --check` passed.
**Code Review**: Complete locally. Lean mode applied because `production/review-mode.txt` is absent; QL-TEST-COVERAGE and LP-CODE-REVIEW external gates were skipped.
**Verification Notes**: Worker commit `a7d792e092d380ec15c7cabcac7effec0c52839a` was integrated. `C2SRequestSnapshot` is an empty C2S reliable protocol message, registered in `shared/src/protocol.rs`, and wired into the client Lightyear sender surface for reveal/stuck recovery only. No reconnect rebuild, full resolution playback, Shop/Auction UI, Hand UI, server placement authority, session-state unrelated files, sprint-status, design, or asset scope was included.
