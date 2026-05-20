# Story 015: S19-BR-RESOLUTION-EVENT-VISUAL-REPLAY-MUTATION-001 -- Resolution Event Visual Replay Mutation

> **Epic**: Board Rendering
> **Story ID**: `S19-BR-RESOLUTION-EVENT-VISUAL-REPLAY-MUTATION-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / Board Rendering + Card Animations + HUD fanout
> **Type**: Integration + Visual/Feel
> **Sprint**: Future Sprint 19 implementation wave; depends on Sprint 18 full-flow pass and resolution replay gap audit
> **Authored**: 2026-05-19 by PROMPT 1485
> **Authoring source-of-truth**: isolated worker branch `work/1485-resolution-event-visual-replay-mutation-story`
> **Source reports**: PROMPT 1472 full-flow PASS, PROMPT 1477 resolution/combat visual-state audit, Krosmaga reference reports PROMPT 1265 / 1266 / 1395
> **Estimated effort**: ~1.0d

---

## Status / No-Claim Banner

This story is a future candidate only. It does not activate Sprint 19, close
Sprint 18, claim release readiness, claim final-art completion, claim full
combat redesign, or claim that the playable client has final resolution feel.

The story preserves the existing server-authoritative contract: the client
visually replays `S2CResolutionEvent` and mutates presentation state only. It
must not add client-side gameplay validation, recompute combat outcomes, or send
new game-logic C2S messages.

Krosmaga reports are visual and composition references only. This story does not
copy Krosmaga assets, progression semantics, AP/end-turn rules, or combat rules.

---

## Source Findings

PROMPT 1472 established that the current full playable loop can pass end to end.
That pass is a dependency, not a release-readiness claim.

PROMPT 1477 found that the resolution/combat loop still has visual replay gaps:
the client can receive and queue the authoritative resolution log, but the replay
does not yet mutate all visible board/HUD state in sub-step order. Existing
Board Rendering Story 006 covers `S2CResolutionEvent` intake, grouping,
`AnimQueue` timing, and phase buffering; it does not claim complete per-event
state mutation.

Current local code already has partial feedback:

- `CombatDamage` can emit `DamageNumberSpawnRequested` for non-zero damage.
- `UnitDied` / `UnitRemoved` can spawn a transient kill marker.
- `SpawnRangeChanged` can fan out local spawn-range updates.

The missing implementation-ready slice is a deterministic replay applier that
walks each `TaggedEvent` at the moment its animation group is presented and
keeps visible board, objective, HUD/resource, and next-phase state coherent with
the event order.

Krosmaga reference reports PROMPT 1265 / 1266 / 1395 are dependencies for visual
hierarchy and feedback expectations: resolution outcomes should read at the
point of action, with board motion, damage, objective feedback, rewards, and
phase return behaving as one coherent sequence rather than as disconnected
markers.

---

## Scope

### In Scope

- Add a Board Rendering replay application path that applies
  `S2CResolutionEvent.events` in sorted `(sub_step, trigger_index)` order as
  each `AnimGroup` becomes active or drains.
- Mutate visible unit position for `UnitMoved` using `BoardLayout::cell_to_world`
  and the existing board z-layer/tween conventions.
- Mutate visible unit lane for `UnitChangedLane`, including `LaneCell`,
  transform target, co-occupancy/status placement, and any drag/targeting
  overlays that depend on lane/cell.
- Display `UnitPlaced` units from the replay when the unit was not already
  visible from placement reveal/snapshot, while avoiding duplicate visible
  entities for the same `unit_id`.
- Apply `CombatDamage` and `ObjectiveDamage` to visible HP bars/counters at the
  replay moment, including zero-damage SHIELD-blocked hits as a distinct visual
  cue without inventing damage.
- Apply `UnitDied` / `UnitRemoved` by marking or fading the visible unit during
  the replay group, then removing or hiding it before the next phase becomes
  visible.
- Apply `ObjectiveDestroyed` with objective HP/destruction feedback and any
  existing staged objective reveal/HUD-safe fanout, preserving hidden identity
  boundaries.
- Surface `GoldAwarded` as player-resource feedback through existing HUD or
  presentation messages, with deterministic fixtures covering kill and objective
  rewards.
- Preserve `SpawnRangeChanged` forwarding and verify it composes with the
  replay applier rather than bypassing sub-step order.
- Keep `PendingPhaseChange` / `BoardRenderState::ResolutionExecuting` active
  until all replay mutations and objective reveal handoff work needed for the
  visible result have completed.
- Add QA snapshot/debug fields or test-observable resources that expose current
  replay sub-step, applied trigger index, moved/removed unit counts, objective
  damage/destruction count, reward feedback count, and pending next phase.

### Out Of Scope

- Server combat rule changes or combat balance.
- New `ResolutionEvent` protocol categories unless a pre-existing category is
  insufficient and a separate protocol story approves it.
- Broad combat redesign.
- Final art, final VFX, final sound, or release-quality animation polish.
- Full live-session manual QA requirement; acceptance must be testable with
  deterministic fixtures or World/App-based presentation tests.
- Sprint closeout, `/story-done`, sprint-status edits, QA plan edits, or
  session-state edits.

---

## Dependencies And Parallelism

| Dependency | Required posture |
|---|---|
| PROMPT 1472 full-flow PASS | Required baseline: the loop receives resolution data and returns to the next phase. This story must not regress that flow. |
| PROMPT 1477 resolution/combat visual-state audit | Required gap source: replay must mutate visible state, not only prove queue timing. |
| Board Rendering Story 006 | Must remain Complete; provides `S2CResolutionEvent` grouping, `AnimQueue`, invalid-sub-step recovery, and phase buffering. |
| Combat Resolution Story 011 | Must remain Complete; provides canonical `TaggedEvent` / `ResolutionEvent` completeness and ordering. |
| Board Rendering Story 003 | Required for visible unit/objective/HP entities that replay mutations update. |
| Card Animations Stories 006 and 007 | Reuse staged objective reveal and damage-number lifecycle rather than creating duplicate animation systems. |
| HUD gold/objective surfaces | Required for reward/objective feedback fanout; if a message contract is missing, split that as a small prerequisite before implementation. |
| Krosmaga reports 1265 / 1266 / 1395 | Visual hierarchy and state-feedback references only; must not introduce Krosmaga rules or assets. |

This story owns Board Rendering replay mutation and should not run concurrently
with workers editing the same resolution queue/replay systems. It may run in
parallel with unrelated future-sprint work that does not touch
`client/src/presentation/board_rendering.rs`, Card Animations replay resources,
HUD resource feedback, or board-rendering tests.

---

## Acceptance Criteria

- [ ] **AC1 -- Replay applies in event order**: Given a deterministic
  `S2CResolutionEvent` fixture with mixed sub-steps and trigger indices, when
  the replay runs under `Time<Virtual>`, then visible mutation order follows
  ascending `sub_step` and stable ascending `trigger_index` within each group.
- [ ] **AC2 -- Unit movement mutates the board**: Given a visible unit and a
  `UnitMoved` event, when that event's replay group applies, then the unit's
  `LaneCell` and transform/tween target move from `from_cell` to `to_cell` on
  the same lane using `BoardLayout` coordinates.
- [ ] **AC3 -- Lane changes mutate the board**: Given a visible unit and a
  `UnitChangedLane` event, when the event applies, then the unit's lane, visual
  position, status icon anchors, and co-occupancy offsets reflect the destination
  lane before later sub-step events target it.
- [ ] **AC4 -- Placed/spawned units appear once**: Given a `UnitPlaced` event
  for a unit not currently visible, when the event applies, then exactly one
  visible board unit appears at the event lane/cell with placeholder-safe art and
  HP display; if the unit is already visible, no duplicate entity is spawned.
- [ ] **AC5 -- Damage updates HP visibly**: Given `CombatDamage` events,
  including non-lethal damage and SHIELD-blocked zero damage, when each event
  applies, then the defender HP bar/counter updates to `defender_hp_after`, a
  deterministic damage or blocked cue is emitted, and later events read the
  updated visible state.
- [ ] **AC6 -- Removal/death is visible and final**: Given `UnitDied` or
  `UnitRemoved`, when the event applies, then the unit receives a fade/marker or
  equivalent death state and is no longer visible or targetable before the next
  phase is applied.
- [ ] **AC7 -- Objective feedback mutates visible state**: Given
  `ObjectiveDamage` and `ObjectiveDestroyed`, when those events apply, then
  standing objective HP/destruction visuals and HUD-safe objective feedback
  reflect `objective_hp_after` and destruction without leaking hidden identity
  beyond existing allowed reveal contracts.
- [ ] **AC8 -- Rewards are visible**: Given kill and objective
  `GoldAwarded` events, when each event applies, then the appropriate player
  resource feedback is emitted exactly once and can be asserted without a live
  network session.
- [ ] **AC9 -- Spawn-range updates stay ordered**: Given
  `SpawnRangeChanged` mixed with damage/removal/objective events, when replay
  applies, then local spawn-range feedback is emitted at that event's ordered
  position and does not bypass earlier replay mutations.
- [ ] **AC10 -- Next phase is clear and delayed correctly**: Given a buffered
  `S2CPhaseChanged(DRAFT_SHOP)` or `S2CPhaseChanged(GAME_OVER)`, when replay
  still has unapplied visual mutations or staged objective reveal work, then
  `PendingPhaseChange` remains buffered; when replay fully drains, the next phase
  applies once and the board presents a coherent final state.
- [ ] **AC11 -- Deterministic test coverage exists**: World/App-based
  presentation tests cover unit movement, lane change, placed-unit display,
  damage HP update, removal/death, objective damage/destruction, reward feedback,
  spawn-range ordering, and phase return without requiring a full live session.
- [ ] **AC12 -- Authority and no-claim boundaries are preserved**: Completion
  notes state that the client only mutates presentation state from authoritative
  `S2CResolutionEvent` data and does not claim release readiness, final art,
  broad combat redesign, or sprint closeout.

---

## Implementation Notes

- Treat `TaggedEvent.sub_step` and `TaggedEvent.trigger_index` as the replay
  clock. Do not infer outcomes by querying current client state and recomputing
  combat.
- Prefer a small replay-applier system or resource that consumes the currently
  active `AnimQueueEvent::ResolutionReplay` entries rather than pre-emitting all
  feedback for the entire script during queue construction. This is the key
  difference from the existing partial feedback path.
- Keep mutation idempotent by tracking applied event identity, for example
  `(sub_step, trigger_index, variant, primary target id)`, so reconnect,
  duplicate local queue ticks, or phase buffering cannot double-spawn rewards,
  damage numbers, or units.
- Use Bevy 0.18 message APIs (`MessageWriter` / `MessageReader` and
  `app.add_message::<T>()`) for intra-client fanout.
- Keep board units and objectives as world-space sprites under ADR-021. Do not
  implement replayed board units/objectives as `bevy_ui` nodes.
- Preserve existing recovery behavior: invalid sub-steps still reject the whole
  script and request a snapshot instead of applying a partial visual state.
- Use `Time<Virtual>` in tests so group boundaries, fade markers, and phase
  return are deterministic.

---

## QA Test Cases

- **Ordered mixed replay fixture**
  - Given: events for `UnitPlaced`, `UnitMoved`, `CombatDamage`,
    `GoldAwarded`, `UnitRemoved`, `ObjectiveDamage`, `ObjectiveDestroyed`, and
    `SpawnRangeChanged` across sub-steps 1, 3, 4, 5, and 6.
  - When: the app advances virtual time one replay group at a time.
  - Then: a test-observable replay trace shows each event applied in
    `(sub_step, trigger_index)` order.

- **Movement and lane mutation**
  - Given: a visible unit at lane 2 cell 3.
  - When: `UnitChangedLane { from_lane: 2, to_lane: 3 }` then
    `UnitMoved { lane: 3, from_cell: 3, to_cell: 5 }` apply.
  - Then: the visible unit ends at lane 3 cell 5, with no stale lane 2 status
    icon or targetable marker.

- **HP, death, and phase handoff**
  - Given: two damage events reduce a unit to zero HP, followed by `UnitRemoved`
    and a buffered `DraftShop` phase.
  - When: virtual time advances through the final replay group.
  - Then: HP updates are visible before the death marker/fade, the unit is gone
    before `DraftShop`, and the buffered phase applies exactly once.

- **Objective and reward feedback**
  - Given: objective damage to HP 0, `ObjectiveDestroyed`, and objective
    `GoldAwarded`.
  - When: replay applies that group.
  - Then: objective HP/destruction visuals, HUD-safe objective update, and gold
    feedback are each emitted once.

---

## Test Evidence

**Story Type**: Integration + Visual/Feel

**Required evidence**:

- Integration:
  `tests/integration/board_rendering/resolution_event_visual_replay_mutation_test.rs`
  or equivalent registered client integration test.
- Focused unit/presentation tests are acceptable for small helpers, but the main
  coverage must use a real Bevy `App` / `World` with `BoardRenderingPlugin`,
  `CardAnimationsPlugin`, deterministic fixtures, and `Time<Virtual>`.
- Optional browser/WASM captures may be added after deterministic tests pass, but
  they are not required for initial story completion.

**Status**: [ ] Future implementation required

---

## Worker Contract

1. Worktree slug: `work/s19-br-resolution-event-visual-replay-mutation`.
2. Activate `liv-bevy-018` before touching Bevy/Rust files.
3. Read PROMPT 1472, PROMPT 1477, PROMPT 1265, PROMPT 1266, PROMPT 1395,
   ADR-017, ADR-021, Board Rendering Story 006, Combat Resolution Story 011,
   and current board-rendering/card-animation replay tests before editing.
4. Keep the client presentation-only and server-authoritative.
5. Do not run broad live-session QA as the only acceptance path; add
   deterministic World/App-based tests.
6. Do not edit sprint/session status or close any sprint from this story.
