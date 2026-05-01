# Story 009: PLACEMENT Timer — Urgency, Grace Window & Submit Checkmark

> **Epic**: Hand UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: All timer display, urgency state, and grace window logic live in `HandUiPlugin`. `TimerUrgencyAudio` is a Bevy-internal message (not Lightyear) emitted exactly once when the 5s threshold is crossed. The 200ms grace window is a client-side window: the drag remains valid for 200ms after timer reaches 0. `C2SSubmitPlacement` fires at the end of grace window resolution regardless of outcome.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `Time<Virtual>` used for timer countdown — enables deterministic testing without real-time sleep. Urgency tween (Amber→Crimson pulse) uses `bevy_tweening`. `TimerUrgencyAudio` is a Bevy-internal `#[derive(Message)]` — register via `app.add_message::<TimerUrgencyAudio>()`. The "exactly once" firing constraint is enforced by a `TimerUrgencyFired` boolean on the timer resource (reset on PLACEMENT entry).

**Control Manifest Rules (Presentation Layer)**:
- Required: `MessageWriter<TimerUrgencyAudio>` — Bevy-internal, NOT Lightyear.
- Required: `PresentationSet` ordering — timer update in `StateSync`; tween in `AnimationTick`.
- Required: `placement_animation_cap_ms = 250ms` — no animation during PLACEMENT may exceed this cap.
- Required: All timer systems `in_state(ClientState::InSession)`.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rules 9, 11 and ACs HU-15/15b/22/23, scoped to this story:*

- [ ] **HU-15**: GIVEN the player has 2 cards staged and the PLACEMENT timer reaches 0 while a third card is mid-drag (lifted from fan, not yet dropped), WHEN the 200ms grace window elapses without a mouse-up on a valid target, THEN:
  - `C2SSubmitPlacement` is sent with exactly the 2 staged placements (the in-flight card is NOT included)
  - The in-flight drag cancels: drag sprite hidden, fan slot returns to `FanSlotState::Active`
  - The third card is not in the placements vec

- [ ] **HU-15b**: GIVEN the player has 2 cards staged and the PLACEMENT timer reaches 0 while a third Minion card is mid-drag over a valid highlighted board cell, WHEN the player releases mouse-up on that valid cell during the 200ms grace window, THEN:
  - The third card stages to that cell (same as a normal valid drop, per Story 005 HU-13)
  - `C2SSubmitPlacement` is sent with all 3 placements (including the grace-window drop)

- [ ] **HU-22**: GIVEN the placement timer remaining crosses the 5-second threshold (from > 5s to ≤ 5s), WHEN one tick fires, THEN:
  - The timer entity enters `TimerState::Urgent`
  - A single `TimerUrgencyAudio` Bevy-internal message is written exactly once
  - No second `TimerUrgencyAudio` message is written in subsequent ticks while the timer continues to count down
  - Amber color rendering (`#E87C1E`) and pulse animation are ADVISORY (lead sign-off). The state component and single-shot event are BLOCKING.

- [ ] **HU-23**: GIVEN the player submits at 7 seconds remaining (pre-validation passes), WHEN `C2SSubmitPlacement` fires, THEN:
  - (a) The timer entity continues decrementing each frame (timer does not stop on submit)
  - (b) A `TimerSubmittedCheckmark` marker entity has `Visibility::Visible` adjacent to the timer numeral

---

## Implementation Notes

*Derived from ADR-021 and GDD Rules 9 and 11:*

1. **Timer resource**: `PlacementTimer { remaining_ms: u32, urgency_fired: bool, in_grace_window: bool }`. `remaining_ms` decrements each frame via `Time<Virtual>.delta_secs() * 1000.0` cast to `u32`. `urgency_fired` prevents repeat `TimerUrgencyAudio` emissions.

2. **Urgency threshold** (HU-22): When `remaining_ms` crosses from above `placement_urgency_threshold_seconds * 1000` to at or below it (single-frame detection), write `MessageWriter<TimerUrgencyAudio>` and set `urgency_fired = true`. Set `TimerState::Urgent` on timer entity. Never fire again until PLACEMENT re-entry resets `urgency_fired = false`.

3. **Timer expiry → grace window** (HU-15/15b):
   - When `remaining_ms` reaches 0, set `in_grace_window = true`; start a 200ms countdown (`grace_remaining_ms: u32 = 200`).
   - During grace window: continue advancing `grace_remaining_ms` via `Time<Virtual>`. Active drag remains valid.
   - At grace window end OR on valid drop during grace window: resolve and send `C2SSubmitPlacement` with all staged placements at that moment.
   - If drag is active at grace window end with no valid drop: cancel drag (HU-15 path) then send `C2SSubmitPlacement` with currently staged cards only.
   - If valid drop occurs during grace window (HU-15b): stage the card (Story 005 HU-13 path) then send `C2SSubmitPlacement` with the extended staged set.

4. **Submit checkmark** (HU-23): On `C2SSubmitPlacement` send, set `TimerSubmittedCheckmark` entity to `Visibility::Visible`. Timer continues to decrement. The checkmark persists until RESOLUTION entry (Story 003 hides all Hand UI).

5. **PLACEMENT timer display** (GDD Rule 11): Shows whole seconds (not milliseconds); large enough to read peripherally. Timer panel must never render directly over animated board content (semi-opaque Ink Blue panel required — ADVISORY rendering).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Submit button lock (handled there — this story triggers `C2SSubmitPlacement` via the timer expiry path, which then follows the same submit-lock path)
- [Story 003]: Timer visibility on RESOLUTION entry (hidden there)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-15**: Grace window elapses with no valid drop — 2 staged cards submitted
  - Given: PLACEMENT; cards A and B staged; card C mid-drag (drag sprite visible); timer reaches 0
  - When: `Time<Virtual>` advanced by 201ms (grace window elapsed); no mouse-up fired; `App::update()` runs
  - Then: `C2SSubmitPlacement { placements: [A, B] }` written (exactly 2); drag sprite has `Visibility::Hidden`; fan slot for C has `FanSlotState::Active`; C not in placements vec
  - Edge cases: Timer reaches 0 while NO drag active → `C2SSubmitPlacement` fires immediately with all currently staged cards (no grace window needed)

- **HU-15b**: Valid drop during grace window — 3 staged cards submitted
  - Given: PLACEMENT; cards A and B staged; card C mid-drag over valid highlighted cell (lane=1, cell=3); timer reaches 0; grace window starts
  - When: Simulate mouse-up on lane=1 cell=3 while `grace_remaining_ms > 0`; `App::update()` runs
  - Then: Card C stages to (lane=1, cell=3); `C2SSubmitPlacement { placements: [A, B, C] }` written (exactly 3)
  - Edge cases: Mouse-up on invalid cell during grace window → grace window continues until 200ms; at expiry, submit with [A, B] only

- **HU-22**: Urgency single-shot at 5s threshold
  - Given: PLACEMENT; `PlacementTimer { remaining_ms: 5001, urgency_fired: false }`
  - When: `Time<Virtual>` advanced by 2ms; `App::update()` → `remaining_ms` now 4999 (≤ 5000)
  - Then: Timer entity has `TimerState::Urgent`; count of `TimerUrgencyAudio` messages in bus == 1; `urgency_fired == true`
  - When: `Time<Virtual>` advanced by 500ms more; `App::update()` runs
  - Then: `TimerUrgencyAudio` message count still == 1 (no second emission)
  - Edge cases: Urgency already fired (`urgency_fired = true`) → advancing past threshold again emits nothing

- **HU-23**: Submit checkmark while timer running
  - Given: PLACEMENT; `PlacementTimer { remaining_ms: 7000 }`; pre-validation passes (0 or more staged cards)
  - When: Click Submit → `C2SSubmitPlacement` fires; `App::update()` runs
  - Then: `TimerSubmittedCheckmark` entity has `Visibility::Visible`; `PlacementTimer.remaining_ms` < 7000 (continues decrementing in subsequent ticks)
  - Edge cases: Timer hits 0 after submit → no grace window needed (already submitted); `C2SSubmitPlacement` NOT sent again

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/hand-ui/placement_timer_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 (PLACEMENT entry initializes timer; submit path triggered by grace window resolution)
- Unlocks: None (timer is a self-contained PLACEMENT feature)
