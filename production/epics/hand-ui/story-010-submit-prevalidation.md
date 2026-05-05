# Story 010: Submit Pre-Validation -- Mana & Reserve Checks

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time)*
**Requirement Summary**: Submit pre-validation reads the shared `PlayerEconomyView` and blocks `C2SSubmitPlacement` before send when staged explicit current/reserve spends overdraw the local player's current or reserve mana. Server-side validation remains authoritative and correction clears `SubmitValidationError` before a successful send.

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
**ADR Decision Summary**: Client-side pre-validation is defence-in-depth. It gives immediate local feedback before the network round-trip, but the server remains authoritative and may silently discard an invalid batch even if client pre-validation erroneously passes. Pre-validation never deducts gold or mana; it only gates the `C2SSubmitPlacement` send.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: The validation itself is pure resource arithmetic over `Res<PlayerEconomyView>` and the local staged placements vec. The story still touches Bevy UI components and a Lightyear `MessageSender<C2SSubmitPlacement>` send path, so implementation must use `liv-bevy-018` and `liv-bevy-lightyear`. `SubmitValidationError` is a component marker placed on the Submit button entity.

## Prerequisites Confirmed

- `NP-005` Complete at `705defa`: split `C2SSubmitPlacement` / `S2CPlacementReveal` payloads are available.
- `ECO-007` Complete at `a564d99`: `validate_explicit_mana_split` / `apply_explicit_mana_split` exist for server authority.
- `PRES-002` Complete at `8b10c6e`: `PlayerEconomyView` mirrors own `gold`, `current_mana`, `reserve_mana`, and `mana_cap` from authoritative S2C/snapshot data.
- `BLS-011` Complete at `aa543e5`: server-side placement submit validation remains authoritative for full Rule 10 validation and silent discard.
- `production/epics/hand-ui/story-005-placement-submit-core.md` is Complete and provides Submit button state/send behavior.
- `production/epics/hand-ui/story-011-reserve-mana-strip.md` is Complete and provides staged explicit current/reserve split values.

**Current Dependency State**: No prerequisite remains blocked or draft.

**Control Manifest Rules (Presentation Layer)**:
- Required: Pre-validation never mutates `PlayerEconomyView` values; read-only access only.
- Required: Submit button does NOT enter `Inactive` on validation failure; only on successful send.
- Required: Validation and Submit button state update run in `PresentationSet::StateSync` on the same frame as submit click handling.
- Required: Hand UI reads `Res<PlayerEconomyView>` and does not drain `S2CGoldUpdate` directly.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rule 10 and `TR-HU-008`, scoped to current/reserve mana pre-validation:*

- [ ] **HU-17b - reserve overdraw blocks submit**: GIVEN the player has staged cards where `sum(placements[i].reserve_mana_spend) > PlayerEconomyView.reserve_mana`, WHEN Submit is pressed, THEN:
  - (a) No `C2SSubmitPlacement` message is written to the Lightyear send queue
  - (b) The Submit button does NOT enter the `Inactive` interaction state; the player can still adjust and re-submit
  - (c) A `SubmitValidationError::ReserveOverdrawn` marker is attached to the Submit button entity
  - The Crimson inline label rendering (`"Reserve overdrawn"`, `#9C2000`) is ADVISORY (lead sign-off).

- [ ] **TR-HU-008 / Rule 10 - current mana overdraw blocks submit**: GIVEN the player has staged cards where `sum(placements[i].current_mana_spend) > PlayerEconomyView.current_mana`, WHEN Submit is pressed, THEN:
  - (a) No `C2SSubmitPlacement` message is written to the Lightyear send queue
  - (b) The Submit button does NOT enter the `Inactive` interaction state
  - (c) A `SubmitValidationError::ManaOverdrawn` marker is attached to the Submit button entity
  - The Crimson inline label rendering (`"Mana overdrawn"`, `#9C2000`) is ADVISORY (lead sign-off).

- [ ] **HU-17c - correction clears error and submit succeeds**: GIVEN the Submit button previously has `SubmitValidationError::ReserveOverdrawn` or `SubmitValidationError::ManaOverdrawn`, WHEN the player un-stages or adjusts a staged split so both aggregate spends are valid (`sum(reserve_mana_spend) <= reserve_mana` and `sum(current_mana_spend) <= current_mana`) and presses Submit again, THEN:
  - Pre-validation passes
  - `C2SSubmitPlacement` is sent to the Lightyear queue
  - The previous `SubmitValidationError` component is removed from the Submit button entity
  - The Submit button enters `Inactive` through the existing Story 005 successful-send path

---

## Implementation Notes

*Derived from ADR-021, ADR-002, PRES-002, and GDD Rule 10:*

Pre-validation checks in this story run synchronously on Submit click before any Lightyear send:

1. `sum(placements[i].reserve_mana_spend) <= PlayerEconomyView.reserve_mana`
2. `sum(placements[i].current_mana_spend) <= PlayerEconomyView.current_mana`

If both checks fail on the same click, attach/report `SubmitValidationError::ReserveOverdrawn` first. This keeps error priority deterministic and points the player to the reserve split controls first.

Rule 10 also names split-sum/card-cost, hand-membership, and target-range validation. `BLS-011` now implements those checks authoritatively on the server. HAND-UI-010 does not reimplement the non-economy server authority checks; this story's client-side scope is the current/reserve affordability gate required by `TR-HU-008`.

**Error display**: On validation failure, add `SubmitValidationError::<variant>` to the Submit button entity. Remove it when the condition clears (player un-stages or adjusts reserve/current split) before sending. This prevents stale error markers from persisting after the player resolves the issue.

## Performance Budget

No measurable performance impact expected. Validation is `O(n)` over staged placements, with `n <= 10` by hand size. It performs integer sums and one resource read, runs only on submit/error-clear state sync, and must stay inside the ADR-021 Presentation steady-state budget of `< 1 ms` per frame.

---

## Out of Scope

*Handled by neighbouring or completed stories; do not implement here:*

- [Story 005]: Submit button lock on successful send (HU-17)
- [Story 011]: Reserve `[ + ]` / `[ - ]` controls that mutate staged explicit current/reserve split values
- [BLS-011]: Server-authoritative full placement validation, pending-buffer writes, mana deduction at placement close, and silent discard
- Visual polish beyond attaching/removing the `SubmitValidationError` marker; inline label rendering remains advisory

---

## QA Test Cases

*Written for this readiness recheck. The developer implements against these cases.*

- **HU-17b**: Reserve overdraw blocks submit
  - Given: PLACEMENT; card C staged with `reserve_mana_spend = 3`; `PlayerEconomyView.reserve_mana = 2` (overdraw: 3 > 2); Submit button Active
  - When: Click Submit; `App::update()` runs
  - Then: Lightyear outbound queue contains 0 `C2SSubmitPlacement` messages; Submit button interaction == `Active` (NOT Inactive); Submit button entity has `SubmitValidationError::ReserveOverdrawn` marker component
  - Edge cases: `reserve_mana_spend = 2, reserve_mana = 2` (exactly equal) validation passes; `reserve_mana_spend = 0` always passes reserve check

- **TR-HU-008 / Rule 10**: Current mana overdraw blocks submit
  - Given: PLACEMENT; card C staged with `current_mana_spend = 4`; `PlayerEconomyView.current_mana = 3` (overdraw: 4 > 3); Submit button Active
  - When: Click Submit; `App::update()` runs
  - Then: Lightyear outbound queue contains 0 `C2SSubmitPlacement` messages; Submit button interaction == `Active`; Submit button entity has `SubmitValidationError::ManaOverdrawn` marker component
  - Edge cases: `current_mana_spend = 3, current_mana = 3` validation passes; if both current and reserve are overdrawn, `ReserveOverdrawn` wins precedence

- **HU-17c**: Correction clears error + submit succeeds
  - Given: PLACEMENT; submit was attempted with reserve or current overdraw; Submit button has the matching `SubmitValidationError` marker; player un-stages or adjusts split so both aggregate spends are within `PlayerEconomyView`
  - When: Click Submit; `App::update()` runs
  - Then: `C2SSubmitPlacement` is written to the Lightyear queue; Submit button entity has NO `SubmitValidationError` component; Submit button enters `Inactive`
  - Edge cases: Adjust reserve from `3` to `2` when `reserve_mana = 2` succeeds; move one mana from current to reserve succeeds only if both aggregate pools remain within their limits

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/submit_prevalidation_test.rs` - must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: `production/epics/hand-ui/story-005-placement-submit-core.md` (Complete)
- Depends on: `production/epics/hand-ui/story-011-reserve-mana-strip.md` (Complete)
- Depends on: `NP-005` (Complete at `705defa`)
- Depends on: `ECO-007` (Complete at `a564d99`)
- Depends on: `PRES-002` (Complete at `8b10c6e`)
- Depends on: `BLS-011` (Complete at `aa543e5`)
- Unlocks: HAND-UI-010 implementation launch; no downstream story directly

## Readiness Recheck Notes

Rechecked 2026-05-05 after the prerequisite chain completed. Remaining gaps were docs-only: stale blocked status/dependency wording, missing explicit current-mana overdraw criterion, and missing performance-budget note. No code/design blocker remains for HAND-UI-010.

## Completion Notes

**Completed**: 2026-05-05
**Verdict**: COMPLETE
**Criteria**: 3/3 passing. HU-17b reserve overdraw blocks submit, TR-HU-008 / Rule 10 current mana overdraw blocks submit, and HU-17c correction clears the error and sends successfully are covered by `tests/unit/hand-ui/submit_prevalidation_test.rs`.
**Test Evidence**: `cargo test -p client --test hand_ui_submit_prevalidation_test` passed 8/8. Requested adjacent client regressions passed 15/15 across placement timer, placement submit core, instant staging, and reserve mana strip tests. Server authority regression `cargo test -p server --test placement_submit_authority_validation_test` passed 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
**Verification**: Current `main` includes worker branch `work/hand-ui-010-submit-prevalidation` commit `ccecb7c17157e0ace1c34ba93438c46dd8371ff6` through main integration commit `8ee2860`. Submit pre-validation reads `PlayerEconomyView`, rejects reserve overdraw before current overdraw, attaches `SubmitValidationError`, does not lock the Submit button on failure, clears the error on correction, and sends `C2SSubmitPlacement` only after validation passes.
**Deviations**: None. Broader non-economy Rule 10 checks remain server-authoritative under BLS-011 and out of HAND-UI-010 scope.
**Code Review**: Lean mode default; QL-TEST-COVERAGE and LP-CODE-REVIEW skipped.
**Tech Debt**: None logged.
