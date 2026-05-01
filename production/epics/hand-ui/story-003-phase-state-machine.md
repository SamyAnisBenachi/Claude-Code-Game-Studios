# Story 003: Phase State Machine — Visibility & Input Gating

> **Epic**: Hand UI
> **Status**: In Progress
> **Owner**: codex-hand-ui-003-phase-state-machine
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-009: RSM Phase State](../../docs/architecture/adr-009-rsm-phase-state.md)
**ADR Decision Summary**: Hand UI reads phase from `Res<CurrentClientPhase>` exclusively — never drains `MessageReceiver<S2CPhaseChanged>` directly (that buffer is drained once by `phase_sink_system` in `PresentationSet::PhaseTransition`). Phase-transition systems in HandUiPlugin run in `PresentationSet::PhaseTransition` and read the updated resource. All tween cancellation on phase exit must use `Animator<T>::set_tweenable()` — never despawn+respawn.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `MessageReceiver<S2CPhaseChanged>` (Lightyear) can only be drained once per frame — `phase_sink_system` is the only drainer (ADR-021). HandUiPlugin phase-transition system must be in `PresentationSet::PhaseTransition` and read `Res<CurrentClientPhase>`. Confirm `Animator<T>::set_tweenable()` exists by name in bevy_tweening 0.18 before using — run `cargo check` against a stub (ADR-021 Verification Required item 3).

**Control Manifest Rules (Presentation Layer)**:
- Required: Single `phase_sink_system` drains `MessageReceiver<S2CPhaseChanged>` (Lightyear). All sub-plugins read `Res<CurrentClientPhase>` — never `MessageReceiver<S2CPhaseChanged>` directly.
- Required: `PresentationSet` execution order: `PhaseTransition` → `MessageDrain` → `StateSync` → `AnimationTick`.
- Required: Tween cancel-and-replace via `Animator<T>::set_tweenable(new_tween)`. Never despawn+respawn hand entities mid-animation.
- Required: All Hand UI phase-transition systems run `in_state(ClientState::InSession)`.
- Forbidden: Register `MessageReceiver<S2CPhaseChanged>` in HandUiPlugin — first drain consumes all.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rules 3 and 12, scoped to this story:*

- [ ] **HU-04**: GIVEN RSM transitions to RESOLUTION (i.e. `Res<CurrentClientPhase>` is updated to `RESOLUTION`), WHEN the HandUiPlugin phase-transition system runs, THEN after exactly one `App::update()` tick:
  - (a) The fan root entity, Submit button entity, and timer entity all have `Visibility::Hidden`
  - (b) No `Animator<Transform>`, `Animator<BackgroundColor>`, or `Animator<Style>` component exists on any Hand UI entity — **Implementer note**: enumerate the complete list of `Animator<T>` specializations used in this epic at implementation time and add them all to this assertion.

- [ ] **HU-05**: GIVEN Hand UI is in RESOLUTION (`HIDDEN` state, all elements invisible), WHEN `Res<CurrentClientPhase>` updates to `DRAFT_SHOP`, THEN:
  - The fan root entity has `Visibility::Visible`
  - Each rendered fan card slot's `card_id` component matches the current hand contents from the most recently delivered `S2CCardAcquired` messages or snapshot

- [ ] **HU-06**: GIVEN `Res<CurrentClientPhase>` is `DRAFT_AUCTION` (Hand UI is in `PASSIVE_LOCKED` state), WHEN the player clicks a card in the hand fan, THEN no `C2SActivateCard` message is written to the outbound Lightyear queue (input fully suppressed in `PASSIVE_LOCKED`).

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 3 (Phase Behavior State Matrix):*

Implement a dedicated `hand_ui_phase_transition_system` in `PresentationSet::PhaseTransition` that reads `Res<CurrentClientPhase>` and drives a local `HandUiMode` resource (or component on the fan root entity):

| RSM Phase | Hand UI Mode | Visibility action |
|-----------|--------------|-------------------|
| LOBBY | HIDDEN | All elements Hidden; cancel all Animators |
| DRAFT_INITIAL | GRID | Grid overlay Visible; fan below (read-only) |
| DRAFT_SHOP | PASSIVE | Fan Visible; Instant cards input-active |
| DRAFT_AUCTION | PASSIVE_LOCKED | Fan Visible; ALL input suppressed |
| PLACEMENT | STAGING | Fan in drag-and-stage mode; Submit button Visible |
| RESOLUTION | HIDDEN | All elements Hidden immediately; cancel all Animators |

**Tween cancellation on RESOLUTION entry**: For each active `Animator<T>` on any Hand UI entity, call `animator.set_tweenable(Tween::new(Duration::ZERO, ...))` or equivalent zero-duration tween to immediately stop the animation. Do NOT despawn the entity — game-state components (slot state markers) must survive.

**Input suppression in PASSIVE_LOCKED**: The `C2SActivateCard` send path must gate on `HandUiMode != PASSIVE_LOCKED` before enqueuing the message. A click on a fan card in PASSIVE_LOCKED is silently absorbed — no message, no sound (per GDD VA-10).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 004]: DRAFT_INITIAL grid display and purchase flow
- [Story 005]: PLACEMENT staging (Submit button state, drag mechanics)
- [Story 013]: Reconnect rebuild (STAGING entry from snapshot)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-04**: RESOLUTION hides all elements and cancels tweens
  - Given: `App` in `ClientState::InSession`; Hand UI in STAGING state (PLACEMENT phase); some `Animator<Transform>` components active on fan slot entities
  - When: `Res<CurrentClientPhase>` updated to `RESOLUTION`; `App::update()` runs
  - Then: Query fan root entity → `Visibility::Hidden`; query submit button entity → `Visibility::Hidden`; query timer entity → `Visibility::Hidden`
  - Then: Query all Hand UI entities for `Animator<Transform>` (and all other `Animator<T>` specializations used) → zero results
  - Edge cases: Transition from PLACEMENT (mid-drag) to RESOLUTION — drag sprite must also become `Visibility::Hidden` with no orphaned `Animator` components

- **HU-05**: DRAFT_SHOP restore after RESOLUTION
  - Given: `App` in `ClientState::InSession`; Hand UI in HIDDEN (RESOLUTION); `PlayerHands` resource contains [card_id_A, card_id_B]; fan slots pre-configured with those card_ids
  - When: `Res<CurrentClientPhase>` updated to `DRAFT_SHOP`; `App::update()` runs
  - Then: Fan root entity → `Visibility::Visible`; fan slots for indices 0 and 1 → `Visibility::Visible` with `card_id` components == [card_id_A, card_id_B]
  - Edge cases: Hand with 0 cards in DRAFT_SHOP → fan root Visible but all slots Hidden; fan root entity still provides anchor for future card arrivals

- **HU-06**: PASSIVE_LOCKED input suppression
  - Given: `App` in `ClientState::InSession`; `Res<CurrentClientPhase>` set to `DRAFT_AUCTION`; hand fan visible with Instant card at slot 0
  - When: Click event fired on fan slot 0 entity
  - Then: Verify outbound Lightyear message queue contains no `C2SActivateCard` entry (use Lightyear test utilities to inspect outbound queue)
  - Edge cases: Rapid-click 5 times on slot 0 → still 0 `C2SActivateCard` messages enqueued; transition to DRAFT_SHOP and click → 1 `C2SActivateCard` message enqueued (confirms gate is phase-specific)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/phase_state_machine_test.rs` — must exist and pass

**Status**: [x] Created and passing locally on worker branch

---

## Dependencies

- Depends on: Story 001 (pre-pooled entities must exist to toggle visibility on)
- Unlocks: Story 004 (DRAFT_INITIAL grid entry), Story 005 (PLACEMENT staging), Story 013 (reconnect rebuild)
