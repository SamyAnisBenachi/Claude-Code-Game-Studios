# Story 010: Submit Pre-Validation — Mana & Reserve Checks

> **Epic**: Hand UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md)
**ADR Decision Summary**: Client-side pre-validation is defence-in-depth — it mirrors server validation (per `network-protocol.md` line 86) to give the player immediate feedback before the round-trip. Server performs the same validation authoritatively and may silently discard an invalid batch even if client pre-validation erroneously passes. Pre-validation never deducts gold or mana — it only gates the `C2SSubmitPlacement` send.

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**: Pre-validation reads `Res<PlayerEconomies>` (own player's mana/reserve state, populated from `S2CGoldUpdate`) and the local `PendingPlacements` vec. No ECS queries required — pure resource arithmetic. `SubmitValidationError` is a component marker placed on the Submit button entity.

**Control Manifest Rules (Presentation Layer)**:
- Required: Pre-validation never deducts `PlayerEconomies` values — read-only access only.
- Required: Submit button does NOT enter `Inactive` on validation failure — only on successful send.
- Required: All validation in `PresentationSet::StateSync` (same frame as submit click handling).

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 10, scoped to this story:*

- [ ] **HU-17b**: GIVEN the player has staged cards where `sum(placements[i].reserve_amount) > player.reserve_mana`, WHEN Submit is pressed, THEN:
  - (a) No `C2SSubmitPlacement` message is written to the Lightyear send queue
  - (b) The Submit button does NOT enter the `Inactive` interaction state (player can still adjust and re-submit)
  - (c) A `SubmitValidationError::ReserveOverdrawn` marker is attached to the Submit button entity
  - The Crimson inline label rendering (`"Reserve overdrawn"`, `#9C2000`) is ADVISORY (lead sign-off).

- [ ] **HU-17c**: GIVEN the player un-stages a card that was causing the reserve overdraw (bringing `sum(reserve_amount) ≤ player.reserve_mana`), WHEN Submit is pressed again, THEN:
  - Pre-validation passes
  - `C2SSubmitPlacement` is sent to the Lightyear queue
  - The previous `SubmitValidationError` component is removed from the Submit button entity

---

## Implementation Notes

*Derived from ADR-021 and GDD Rule 10:*

Pre-validation checks (from `network-protocol.md` line 86 / GDD Rule 10) — all run synchronously on Submit click before any Lightyear send:

1. `sum(placements[i].reserve_amount) ≤ player.reserve_mana`
2. `sum(card[i].cost − placements[i].reserve_amount) ≤ player.current_mana`
3. For each placement, `card_id` is in `player.current_hand` (from `Res<PlayerHands>`)
4. For each `BoardCell`, `TargetUnit`, `TargetObj`, `LaneWide`: lane in [1..=5], cell in [1..=8] if applicable

This story covers checks 1 and the correction path (HU-17c). Checks 2–4 are not covered by a dedicated AC but should be implemented per the full Rule 10 spec.

**Note — mana overdraw path (coverage gap flagged by QA)**: Rule 10 also defines `ManaOverdrawn` (`sum(cost − reserve_amount) > current_mana`) with an inline error `"Mana overdrawn"`. No story AC explicitly covers this path. It should be implemented alongside the reserve check for complete Rule 10 compliance, but its test case will be written by the implementer (no QA-specified test case exists for it).

**Error display**: On validation failure, add `SubmitValidationError::<variant>` component to the Submit button entity. Remove it when the condition clears (player un-stages or adjusts reserve) — do this proactively each frame the Submit button is in Active state, not just on click. This prevents stale error markers from persisting after the player resolves the issue.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Submit button lock on successful send (HU-17)
- [Story 011]: Reserve amount `[ + ]`/`[ - ]` controls that affect `reserve_amount` values read here

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-17b**: Reserve overdraw blocks submit
  - Given: PLACEMENT; card C staged with `reserve_amount = 3`; `PlayerEconomies.reserve_mana = 2` (overdraw: 3 > 2); Submit button Active
  - When: Click Submit; `App::update()` runs
  - Then: Lightyear outbound queue contains 0 `C2SSubmitPlacement` messages; Submit button interaction == `Active` (NOT Inactive); Submit button entity has `SubmitValidationError::ReserveOverdrawn` marker component
  - Edge cases: `reserve_amount = 2, reserve_mana = 2` (exactly equal) → validation passes; `reserve_amount = 0` → always passes reserve check

- **HU-17c**: Correction clears error + submit succeeds
  - Given: PLACEMENT; submit was attempted with overdraw; Submit button has `SubmitValidationError::ReserveOverdrawn`; player un-stages card C
  - When: `sum(reserve_amount)` is now 0 ≤ `reserve_mana = 2`; click Submit; `App::update()` runs
  - Then: `C2SSubmitPlacement` written to Lightyear queue; Submit button entity has NO `SubmitValidationError` component; Submit button enters `Inactive`
  - Edge cases: Un-stage partial amount (adjust reserve_amount from 3 to 2) → reserve check now passes; submit succeeds; error cleared

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/submit_prevalidation_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 (Submit button and staging core — this story adds a validation gate before the send)
- Unlocks: None directly
