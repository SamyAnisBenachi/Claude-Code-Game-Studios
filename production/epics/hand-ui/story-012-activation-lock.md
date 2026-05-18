# Story 012: Activation Lock — DRAFT_SHOP Instant Card Lock & Timeout

> **Epic**: Hand UI
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md)
**ADR Decision Summary**: Single-shot activation lock prevents double-click `C2SActivateCard` storms during latency spikes. The slot locks on send and unlocks on: (a) `S2CGoldUpdate` received, (b) `S2CActivationRejected` received (OQ8 — NOT YET IN NP GDD), or (c) `activate_timeout_ms` elapses. `S2CCardAcquired` is NOT a valid unlock signal (instant cards never add to hand).

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `HandSlotState::ActivationLocked` is a component on the fan slot entity. `Time<Virtual>` for timeout countdown. `S2CActivationRejected` — Lightyear inbound — does NOT exist yet in the NP GDD (see BLOCKED status below).

**Control Manifest Rules (Presentation Layer)**:
- Required: All activation lock systems `in_state(ClientState::InSession)`.
- Required: `S2CGoldUpdate` is drained in `PresentationSet::MessageDrain` — activation lock unlock from S2CGoldUpdate must check slot state in `PresentationSet::StateSync`.

---

> **BLOCKED (refreshed 2026-05-18 by PROMPT 1303 after the PROMPT 1297 `C2SActivateCard` disposition audit).** The legacy OQ8 wording — "`S2CActivationRejected` is not registered in `design/gdd/network-protocol.md` as of 2026-04-30" — is stale. As of `origin/main`, the GDD `design/gdd/network-protocol.md` DOES register `S2CActivationRejected` (S2C table row, `ActivationRejectedReason` enum, NP-50 wrong-phase AC, NP-55 dispatcher no-op AC, and `activate_timeout_ms` tuning knob — all present). The remaining blockers are now **Rust-side**, not GDD-side:
>
> 1. **Shared protocol registration missing.** `S2CActivationRejected` and `ActivationRejectedReason` are still absent from `shared/src/protocol.rs`. Wire-up is tracked by `production/epics/lightyear-protocol-verification/story-009-s2c-activation-rejected-protocol-register.md` (Sprint 18 candidate `S18-PROTOCOL-S2CACTIVATIONREJECTED-REGISTER-001`, NOT activated). Until that story lands, no client `MessageReceiver<S2CActivationRejected>` can be added and HU-28b cannot be wired.
> 2. **Server card-activation dispatcher missing.** `server/src/network/mod.rs::receive_c2s_messages` currently drains `C2SActivateCard` with `tracing::info!` only — no `S2CGoldUpdate` (NP-55 no-op confirmation), no `S2CActivationRejected` (NP-50 wrong-phase or any other rejection variant), no game-state effect. A future `card-activation` epic dispatcher story (placeholder slug `S19-CARD-ACTIVATION-DISPATCHER-001`) must land before HU-28 / HU-28b are functionally observable — otherwise the activation lock would time out 100% of the time and HU-28a (the `S2CGoldUpdate` unlock path) would never fire either. See PROMPT-1297 audit report §3 and §5 for the full sequencing.
>
> **Action required before opening this story**: Land (1) the protocol registration story, then (2) the server card-activation dispatcher story. HU-29 (timeout path) is the only AC implementable independently of the dispatcher; if needed, HU-29 can be split into a separate story so the activation-lock timer behaviour ships ahead of HU-28 / HU-28b.
>
> **When the blockers above clear**: Run `/story-readiness` on this story file and a re-review by QL-STORY-READY is required before the story enters the sprint.
>
> **Note on the original OQ8 framing.** The hand-ui GDD (`design/gdd/hand-ui.md`) Open Questions table still references the old OQ8 wording ("`S2CActivationRejected` not in NP GDD"). The hand-ui GDD revision is owned by game-designer + network-programmer and was explicitly out of scope for PROMPT 1303. Treat the OQ8 wording in hand-ui.md as also stale; the network-protocol.md GDD is the current authoritative source.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 5c, scoped to this story:*

- [ ] **HU-28** *(BLOCKED: OQ8)*: GIVEN the player clicks an Instant card in hand during DRAFT_SHOP, WHEN `C2SActivateCard` is sent, THEN:
  - The card slot enters `HandSlotState::ActivationLocked`
  - Subsequent clicks on that slot produce no further `C2SActivateCard` messages until one of: (a) `S2CGoldUpdate` received, (b) `S2CActivationRejected` received, or (c) `activate_timeout_ms` (3000 ms) elapses
  - `S2CCardAcquired` is NOT a valid unlock signal — do not implement this as an unlock trigger

- [ ] **HU-28b** *(BLOCKED: OQ8)*: GIVEN an Instant card slot is in `HandSlotState::ActivationLocked` AND `S2CActivationRejected` is received for that card, THEN:
  - The slot immediately reverts to `HandSlotState::Active`
  - Clicks are accepted again (no need to wait for timeout)

- [ ] **HU-29**: GIVEN an Instant card slot is in `HandSlotState::ActivationLocked` AND `activate_timeout_ms` (3000 ms) elapses with no `S2CGoldUpdate` or `S2CActivationRejected` received, THEN:
  - The slot reverts to `HandSlotState::Active`
  - Clicks are accepted again (player may retry)

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 5c:*

1. **Lock on send**: Immediately after `C2SActivateCard` is enqueued in the Lightyear sender, set the fan slot entity to `HandSlotState::ActivationLocked` and start a `ActivationLockTimer { remaining_ms: activate_timeout_ms }` countdown via `Time<Virtual>`.

2. **Unlock on `S2CGoldUpdate`** (HU-28a): In `PresentationSet::MessageDrain`, on `S2CGoldUpdate` receipt, check if any fan slot has `HandSlotState::ActivationLocked`. If so, remove the lock and timer. (Assumption: all instant cards cost ≥ 1 mana → every successful activation produces an `S2CGoldUpdate`. GDD Rule 5c notes: if a zero-cost instant is ever added, this unlock breaks — a dedicated `S2CActivationConfirmed` would then be needed.)

3. **Unlock on `S2CActivationRejected`** (HU-28b — BLOCKED: OQ8): When `S2CActivationRejected` is defined in NP GDD and registered in Lightyear protocol, drain it in `PresentationSet::MessageDrain` and immediately remove the lock from the affected slot.

4. **Unlock on timeout** (HU-29): In `PresentationSet::StateSync`, decrement `ActivationLockTimer`. When `remaining_ms` reaches 0, revert `HandSlotState::ActivationLocked` to `HandSlotState::Active`.

5. **Drag suppression in DRAFT_SHOP** (GDD Rule 5d): During DRAFT_SHOP, drag-start on any card is suppressed (no drag sprite, no lift). This is handled by the phase state machine (Story 003 PASSIVE state) — not part of activation lock.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 003]: PASSIVE state input suppression (DRAFT_SHOP general interaction model)
- [Story 007]: PLACEMENT Instant card staging (different phase, different mechanic)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-29** (ADEQUATE — testable now without OQ8):
  - Activation lock timeout revert
  - Given: DRAFT_SHOP; Instant card at slot 1; `C2SActivateCard` sent → slot enters `HandSlotState::ActivationLocked`; `ActivationLockTimer { remaining_ms: 3000 }`
  - When: Advance `Time<Virtual>` by 3001ms; `App::update()` runs
  - Then: Fan slot 1 has `HandSlotState::Active`; subsequent click on slot 1 → `C2SActivateCard` enqueued (slot accepts clicks again)
  - Edge cases: Click during lock → no `C2SActivateCard`; `S2CGoldUpdate` arrives before timeout → lock cleared before timer fires

- **HU-28** (DEFERRED — blocked on OQ8):
  - When OQ8 resolves: Given DRAFT_SHOP; click Instant slot → `C2SActivateCard` sent → slot `ActivationLocked`; rapid second click → assert no second `C2SActivateCard`; third click → still no `C2SActivateCard`

- **HU-28b** (DEFERRED — blocked on OQ8):
  - When OQ8 resolves: Given slot `ActivationLocked`; inject `S2CActivationRejected { card_id: X }`; App::update() → assert slot has `HandSlotState::Active` immediately; click again → `C2SActivateCard` enqueued

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/hand-ui/activation_lock_test.rs` — HU-29 test must exist and pass before story can be marked partial-done
- HU-28 and HU-28b tests cannot be written until OQ8 resolves

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (DRAFT_SHOP phase entry handled there)
- Unlocks: None
