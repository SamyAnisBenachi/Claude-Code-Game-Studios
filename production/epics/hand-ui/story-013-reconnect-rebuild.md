# Story 013: Reconnect Rebuild — PLACEMENT State Recovery

> **Epic**: Hand UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md), [ADR-009: RSM Phase State](../../docs/architecture/adr-009-rsm-phase-state.md)
**ADR Decision Summary**: On reconnect, `S2CGameSnapshot` is received as the first S2C message (before any live messages). Hand UI must rebuild STAGING state from the snapshot. The local `PendingPlacements` queue is empty after reconnect — no in-progress staging persists across disconnect. Any active drag at disconnect time is also cancelled. Timer is rebuilt from `snapshot.timer_remaining_ms`.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Reconnect detection via Lightyear `OnConnected` event + `snapshot_sent` flag mechanism (Foundation layer, ADR-011). `S2CGameSnapshot` is drained in `PresentationSet::MessageDrain`. Hand UI rebuilds state in `PresentationSet::StateSync` after snapshot is applied to `CurrentClientPhase`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Per-connection `snapshot_sent` flag. No live S2C enqueued to reconnecting client before snapshot processed.
- Required: Mandatory reconnect send order: `S2CHandshake`, `S2CGameSnapshot`, `S2CObjectiveIdentities`, `S2CPhaseChanged`. Hand UI reacts to `S2CGameSnapshot` then `S2CPhaseChanged`.
- Required: `S2CGameSnapshot` must set `CurrentClientPhase` correctly before Hand UI phase-transition runs.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Edge Cases (reconnect) and AC HU-24, scoped to this story:*

- [ ] **HU-24**: GIVEN the player reconnects during PLACEMENT (`S2CGameSnapshot` received with `phase = PLACEMENT` and `timer_remaining_ms = X`), WHEN Hand UI rebuilds, THEN:
  - (a) Hand UI's state machine is `STAGING` (per Story 003 phase transitions — PLACEMENT → STAGING)
  - (b) The local `PendingPlacements` vec is empty (staging state is not persisted across disconnect)
  - (c) The Submit button text reads `"Submit (0 cards)"`
  - (d) The `PlacementTimer.remaining_ms` resource value equals `snapshot.timer_remaining_ms`
  - (e) The pre-pooled drag sprite entity has `Visibility::Hidden` (any in-flight drag at disconnect is cancelled and does not persist after rebuild)

---

## Implementation Notes

*Derived from ADR-002, ADR-021, and GDD reconnect edge case:*

1. **Snapshot handling**: `S2CGameSnapshot` (drained in `PresentationSet::MessageDrain`) contains `phase`, `timer_remaining_ms`, and `hand: Vec<CardId>`. On receipt:
   - Update `CurrentClientPhase` to the snapshot's phase (this triggers Story 003's phase-transition system)
   - Set `PlacementTimer.remaining_ms = snapshot.timer_remaining_ms`
   - Rebuild fan slot bindings from `snapshot.hand`

2. **PendingPlacements reset**: On ANY reconnect (regardless of phase), clear the local `PendingPlacements` vec. There is no server-side state to restore for in-progress staging — the player must re-stage within the remaining timer window.

3. **Drag sprite**: On reconnect, set drag sprite entity to `Visibility::Hidden` unconditionally. Any in-flight drag at disconnect time is discarded.

4. **STAGING entry**: The phase-transition system (Story 003) handles PLACEMENT → STAGING visibility, which includes showing the Submit button. The timer is initialized from the snapshot's `timer_remaining_ms` instead of the full `placement_timer_seconds`. The urgency state (`urgency_fired`) must be recalculated based on the snapshot timer value: if `timer_remaining_ms ≤ placement_urgency_threshold_seconds * 1000`, set `TimerState::Urgent` and `urgency_fired = true` immediately (do not re-fire the audio event for an urgency that would have already fired).

5. **Designer flag (open edge case)**: Reconnect with `timer_remaining_ms = 0` — the server will have already auto-submitted for this player (server-side behavior). The expected client behavior is to wait for `S2CPhaseChanged(RESOLUTION)` which arrives in the reconnect send order. Hand UI should render a "Submitted" state with empty placements. Confirm this interpretation with the designer before implementation.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 003]: Phase-transition visibility (STAGING entry from snapshot uses the same path as normal entry)
- [Story 013]: This story IS the reconnect story — no other story handles HU-24

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-24**: Reconnect during PLACEMENT rebuilds STAGING with empty queue
  - Given: `App` in `ClientState::InSession`; simulate disconnect (session state cleared); reconnect event fires
  - When: Inject `S2CGameSnapshot { phase: PLACEMENT, timer_remaining_ms: 4500, hand: [card_A, card_B] }`; `App::update()` runs
  - Then: (a) `HandUiMode` == STAGING; (b) `PendingPlacements.len()` == 0; (c) Submit button text == `"Submit (0 cards)"`; (d) `PlacementTimer.remaining_ms` == 4500; (e) drag sprite entity has `Visibility::Hidden`
  - Edge cases:
    - Reconnect with `timer_remaining_ms = 2500` (urgency threshold = 5s → already past): assert `TimerState::Urgent` on timer entity; assert `urgency_fired == true`; assert no `TimerUrgencyAudio` message (do NOT re-fire for an already-elapsed urgency)
    - Reconnect with `phase = DRAFT_SHOP` (not PLACEMENT): assert Hand UI in PASSIVE state; timer not shown
    - **Open edge case (timer_remaining_ms = 0)**: confirm designer intent — expected: render "Submitted" state, wait for `S2CPhaseChanged(RESOLUTION)`

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/hand-ui/reconnect_rebuild_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (pre-pooled entities rebuilt from), Story 003 (phase state machine handles STAGING entry), Story 005 (PLACEMENT submit state reset)
- Unlocks: None (independent recovery path)
