# Story 014: PLACEMENT Staged Disclosure Accessibility

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility disposition row `A11Y-ST-14` states that PLACEMENT staged disclosure needs browser/WASM evidence showing card selection, lane selection, cell selection, and mana split/submit disclosure without later controls appearing prematurely.

**Current drag/drop assessment**: The implemented Hand UI flow already covers card selection, valid target highlighting, valid drop staging, fan ghost state, reserve strip visibility after staging, submit pre-validation, and correction. It does not fully satisfy A11Y-ST-14 as an evidence-only row because the current drag/drop flow moves from card selection directly to the full valid target set and does not expose a testable lane-then-cell disclosure step or UI guidance state. Treat this story as UI guidance hardening plus evidence.

**GDD**: `design/gdd/hand-ui.md`
**UX Spec**: `design/ux/hand-ui.md`, `design/ux/hud.md`
**Accessibility Source**: `design/accessibility-requirements.md`
**Requirement**: `TR-HU-002`, `TR-HU-003`, `TR-HU-004`, `TR-HU-008`, `TR-PRES-001`, A11Y-ST-14

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-002: Client-Server Authority Model](../../../docs/architecture/adr-002-client-server-authority.md), [ADR-023: Placement Timer Accessibility Authority](../../../docs/architecture/adr-023-placement-timer-accessibility-authority.md)

**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.

**GDD trace**:

- `design/gdd/hand-ui.md` HU-12 through HU-13 require PLACEMENT drag-to-stage target highlighting, valid target drop confirmation, `GhostPlacementChanged`, fan ghost state, Submit count update, and reserve strip visibility after staging.
- `design/gdd/hand-ui.md` HU-18 through HU-19 require Instant cards to highlight the fan plate, clear board highlights, stage to `PlayTarget::Instant`, and update Submit count.
- `design/gdd/hand-ui.md` HU-17b and HU-17c require submit pre-validation to block current/reserve overdraw, keep Submit active, attach `SubmitValidationError`, clear the error after correction, and send only after the split is valid.
- `design/gdd/hand-ui.md` HU-25 through HU-27 require reserve split controls to clamp reserve spend, disable the increment control at ceiling, preserve other staged cards, and hide the strip for zero-cost cards.

**UX and accessibility trace**:

- `design/accessibility-requirements.md` A11Y-ST-14 requires PLACEMENT staged disclosure: select card -> select lane -> select cell -> confirm mana split. The mana split input must not appear until card and lane are selected.
- `design/ux/hand-ui.md` Interaction Map requires PLACEMENT card selection to show valid targets, board target confirmation to stage the card, reserve controls to follow staged cards, and Submit invalid state to provide an inline error without sending.
- `design/ux/hand-ui.md` Accessibility requires the reserve strip to appear only after a card is staged, Submit count text to communicate staged count, and drag interactions to have final accessibility evidence for select card, valid target, reserve adjust, and submit.
- `design/ux/hud.md` requires PLACEMENT focus to remain inside the game canvas while the timer is active.

**Engine**: Bevy 0.18 + Lightyear 0.26 + WASM browser evidence | **Risk**: HIGH

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:

- `HandUiPlugin` remains the third `PresentationPlugin` sub-plugin after Card Animations and Board Rendering.
- Hand UI reads phase through `Res<CurrentClientPhase>` only. It must not drain `MessageReceiver<S2CPhaseChanged>`.
- Hand UI reads `Res<PlayerEconomyView>` for own current/reserve mana. It must not drain economy S2C messages directly.
- The drag sprite and disclosure guidance UI are bevy_ui `Node` entities, not world-space sprites.
- UI work runs in ADR-021 `PresentationSet` order and remains session-scoped under `ClientState::InSession`.
- `GhostPlacementChanged` remains a Bevy-internal message written by Hand UI and consumed by Board Rendering.
- Client-side staged disclosure is presentation guidance only. It must not mutate authoritative hand, board, objective, phase, current mana, or reserve mana state.
- Active PLACEMENT timer display continues to use server-provided phase/snapshot duration. No local Settings multiplier is applied in Hand UI.

---

## Scope

### In Scope

- Add or verify test-observable PLACEMENT disclosure states for the sequence: no card selected, card selected, lane/cell target guidance active, staged card confirmed, reserve/current split visible, submit/correction active.
- Verify or repair the card selection step so selecting a PLACEMENT fan card does not reveal reserve split controls before a valid target is confirmed.
- Verify or repair lane/cell target guidance for board-target cards. The UI must make the target-selection step explicit enough for browser/WASM evidence to distinguish card selection from target choice.
- Preserve existing valid highlight behavior for Minion, TargetObj, LaneWide, TargetUnit, and Instant cards.
- Preserve existing staged card behavior: `FanSlotState::Ghost`, `GhostPlacementChanged`, pending placement, Submit count, board ghost or Instant fan ghost.
- Verify or repair reserve/current mana split disclosure after staging. The user must see the per-card reserve allocation and the implied current mana contribution without relying on color alone.
- Preserve Story 010 submit pre-validation behavior exactly: overdraw blocks send, Submit remains active, `SubmitValidationError` is attached, correction clears the error, and successful submit follows the existing successful-send path.
- Capture browser/WASM evidence that A11Y-ST-14 is implemented with explicit target-step guidance and staged split disclosure.

### Out of Scope

- Do not implement broader Settings / Accessibility preferences, colorblind palettes, UI scaling, reduced motion, tutorial persistence, or full input remapping.
- Do not change card costs, card target legality, spawn range rules, server placement validation, or `C2SSubmitPlacement` payload shape.
- Do not change HAND-UI-010 validation semantics or error precedence.
- Do not add a submit confirmation modal, undo modal, or delayed confirmation step.
- Do not change PLACEMENT timer authority or locally multiply the timer duration.
- Do not close QA-COND-0005 from this story alone.
- Do not modify `production/sprint-status.yaml`, `production/session-state/**`, `AGENTS.md`, or unrelated epics.

---

## Acceptance Criteria

- [x] **A11Y-ST-14 entry state**: GIVEN PLACEMENT begins, WHEN Hand UI enters STAGING, THEN Submit and timer are visible, fan cards are selectable, no card is selected, no board target guidance is active, no board cells or fan plate are highlighted, and all reserve/current split controls are hidden.

- [x] **A11Y-ST-14 card selection disclosure**: GIVEN a board-target card is selected from the PLACEMENT fan, WHEN the selection/drag begins, THEN the original fan slot enters the active selection visual state, the drag sprite or selected-card affordance is visible, the disclosure guidance state communicates target selection, and reserve/current split controls remain hidden.

- [x] **A11Y-ST-14 lane/cell target guidance**: GIVEN a selected Minion card, WHEN valid target guidance is active, THEN the UI exposes a test-observable target-disclosure state for lane/cell selection and the highlighted board cells equal the existing HU-12 valid set. Browser/WASM evidence must show the player is being guided to choose a lane/cell target before mana split controls appear.

- [x] **Existing highlight semantics preserved**: GIVEN TargetObj, LaneWide, TargetUnit, and Instant cards are selected, WHEN their target guidance is active, THEN Story 006 and Story 007 highlight semantics are unchanged: objectives highlight only surviving opponent objectives, LaneWide highlights all lane cells, TargetUnit uses `TargetUnitHover` with no `BoardCellHighlighted` cells, and Instant uses `FanPlateHighlighted` with no board cell highlights.

- [x] **Invalid target preserves disclosure recovery**: GIVEN a card is selected and the player releases on an invalid target or outside the valid zone, WHEN the drop resolves, THEN the drag sprite hides, the fan slot returns to `FanSlotState::Active`, no `GhostPlacementChanged` message is written, pending placements remain unchanged, and reserve/current split controls remain hidden.

- [x] **Staged card state follows target confirmation**: GIVEN a selected card is released on a valid board target, WHEN the drop resolves, THEN one pending placement exists for that card, `GhostPlacementChanged { target: Some(<resolved target>), card_id: Some(card_id) }` is written, the fan slot enters `FanSlotState::Ghost`, Submit text increments to `Submit (N cards)`, and the staged card guidance state replaces target-selection guidance.

- [x] **Instant staged state follows fan plate confirmation**: GIVEN an Instant card is selected, WHEN it is released on the highlighted fan plate, THEN the card stages to `PlayTarget::Instant`, no board highlight remains, the fan slot enters `FanSlotState::Ghost`, Submit count increments, and the same staged-card disclosure rules apply.

- [x] **Reserve/current mana strip disclosure timing**: GIVEN a cost greater than 0 card has not yet been staged, THEN its reserve/current split controls are hidden. GIVEN that card is staged, THEN its split controls become visible only after target confirmation and expose both reserve spend and current spend in a non-color-only way. GIVEN a zero-cost card is staged, THEN the split controls remain hidden.

- [x] **Reserve/current split behavior preserved**: GIVEN a staged card with reserve controls visible, WHEN `+` or `-` is activated, THEN `reserve_mana_spend` and `current_mana_spend` update using the existing Story 011 clamp and ceiling rules, disabled increment behavior remains unchanged, and no other staged card is auto-decremented.

- [x] **Submit invalid flow preserves HAND-UI-010**: GIVEN staged placements overdraw reserve mana or current mana, WHEN Submit is pressed, THEN no `C2SSubmitPlacement` is sent, Submit remains `Active`, `SubmitValidationError::ReserveOverdrawn` or `SubmitValidationError::ManaOverdrawn` is attached with the existing reserve-first precedence, and the disclosure guidance keeps the player in correction flow.

- [x] **Correction and submit success preserve HAND-UI-010**: GIVEN a prior submit attempt failed pre-validation, WHEN the player adjusts reserve/current split or un-stages cards so both aggregate spends fit `PlayerEconomyView`, THEN the next Submit clears `SubmitValidationError`, sends exactly one `C2SSubmitPlacement`, changes Submit to `Submitted`, and shows the submitted checkmark through the existing successful-send path.

- [x] **Browser/WASM evidence completeness**: The evidence document captures PLACEMENT entry, card selected, lane/cell target guidance, valid target highlight, valid stage, reserve/current split adjustment, invalid submit, correction, and successful submit. It also records that later controls are not shown before their disclosure step.

- [x] **QA-COND-0005 impact statement**: The evidence document states that this story implements and evidences only A11Y-ST-14 and that QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented/evidenced, reclassified, dependency-blocked, or accepted as risk.

- [x] `git diff --check` passes.

---

## Implementation Notes

- Treat this as a UI guidance/evidence story, not a gameplay rewrite. The existing drag/drop, highlight, staging, reserve strip, and submit paths should remain the source behavior.
- If the current mouse drag flow is retained, add a stable disclosure state, prompt, stepper, focus label, or equivalent UI affordance that makes the lane/cell selection step observable in automated tests and visible in browser/WASM evidence.
- Do not expose reserve/current split controls while a card is only selected or being dragged. They appear after a valid target has staged the card.
- Do not hide Submit on PLACEMENT entry. Story 005 requires Submit to remain visible as `Submit (0 cards)`. A11Y-ST-14 is about not showing later target and mana-split decisions prematurely, not about removing Submit.
- The reserve/current strip may reuse the existing Story 011 reserve strip implementation, but evidence must show the player can understand reserve spend and current spend without relying on color alone.
- Existing Story 010 pre-validation is a guardrail. Guidance changes may add explanatory copy or focus state, but must not change validation sums, error precedence, button active state on failure, or successful-send behavior.
- Keep any evidence-only measurement overlay, debug label, or capture harness out of normal shipping UI unless it is also part of the accepted player-facing guidance.

## Performance Budget

No gameplay-loop performance impact expected beyond a small fixed Hand UI guidance surface and test-observable state. Disclosure state updates must remain O(1) per interaction, target highlight calculation remains the existing O(board cells) path from Story 006, and Presentation steady-state must stay under 1 ms per frame with interaction spikes under the ADR-021 3 ms guardrail.

---

## QA Test Cases

- **Entry disclosure baseline**
  - Given: PLACEMENT begins with at least one playable hand card
  - When: Hand UI enters STAGING
  - Then: Submit reads `Submit (0 cards)`, timer is visible, no target guidance is active, no target highlights exist, and every reserve/current split control is hidden

- **Card selected before target**
  - Given: PLACEMENT and a board-target card in the fan
  - When: the card is selected or drag-started
  - Then: target guidance is active, the valid highlight set appears, the drag sprite or selected-card affordance appears, and reserve/current split controls remain hidden

- **Lane/cell target guidance**
  - Given: a selected Minion card and a board with valid spawn cells, occupied cells, and an already staged Minion target
  - When: target guidance updates
  - Then: the test-observable disclosure state is lane/cell target selection and highlighted cells equal spawn range minus occupied cells minus already staged Minion cells

- **Target kind regressions**
  - Given: TargetObj, LaneWide, TargetUnit, and Instant cards are selected in separate cases
  - When: target guidance updates
  - Then: existing Story 006 and Story 007 highlight semantics remain unchanged for each target type

- **Valid stage reveals split**
  - Given: a cost greater than 0 board-target card is selected
  - When: it is released on a valid highlighted target
  - Then: the card stages, fan slot becomes `Ghost`, Submit count increments, `GhostPlacementChanged` is written, and the reserve/current split control becomes visible for that staged card

- **Invalid drop keeps split hidden**
  - Given: a card is selected
  - When: it is released outside any valid target
  - Then: the card returns active, no ghost message is written, pending placements remain unchanged, and reserve/current split controls stay hidden

- **Reserve/current split correction**
  - Given: a staged card overdraws current or reserve mana
  - When: Submit is pressed
  - Then: existing HAND-UI-010 behavior blocks send and keeps Submit active
  - Edge case: after split adjustment or un-stage fixes the aggregate spends, the next Submit sends exactly once and clears the error

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/hand-ui/placement_staged_disclosure_accessibility_test.rs`
  - Register as `hand_ui_placement_staged_disclosure_accessibility_test`
  - Command: `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test hand_ui_placement_submit_core_test`
- `cargo test -p client --test hand_ui_placement_drag_highlights_test`
- `cargo test -p client --test hand_ui_placement_instant_staging_test`
- `cargo test -p client --test hand_ui_placement_unstaging_test`
- `cargo test -p client --test hand_ui_reserve_mana_strip_test`
- `cargo test -p client --test hand_ui_submit_prevalidation_test`
- `cargo test -p client --test hand_ui_placement_timer_test`
- `cargo check -p client`
- `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`

**Required browser/WASM capture contents**:

- Browser build/source identifier, viewport size, UI scale, input method, and whether the run used mouse drag, keyboard focus, or both.
- PLACEMENT entry capture showing Submit, timer, selectable fan cards, no active target guidance, no board/fan target highlight, and no visible reserve/current split controls.
- Card-selected capture showing a selected or dragged card, visible target guidance, valid target highlights, and no reserve/current split controls.
- Lane/cell target capture showing the selected board target step and the valid target set. For Minion cards, include enough board context to identify lane and cell.
- Valid-stage capture showing fan ghost state, board ghost or Instant fan ghost, Submit count increment, and reserve/current split controls visible only after staging.
- Reserve/current split adjustment capture showing `+` and `-` behavior, disabled increment at ceiling, reserve spend, and current spend in a non-color-only presentation.
- Invalid-submit capture showing no outbound submit, Submit still active, and inline correction guidance for reserve or current overdraw.
- Correction capture showing split adjustment or un-stage followed by exactly one successful submit, `Submitted` text, and submitted checkmark.
- A note stating what target-step guidance was added and how it prevents later controls from appearing before their disclosure step.

**QA-COND-0005 impact statement required in evidence**:

Story 014 implements and evidences A11Y-ST-14 for PLACEMENT staged disclosure. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, dependency-blocked, or accepted as risk.

**Status**: [x] Implemented and captured.

---

## Dependencies

- Depends on: [Story 005](story-005-placement-submit-core.md) - Complete; provides Submit entry, valid drop staging, invalid drop recovery, pending placement, and successful submit behavior.
- Depends on: [Story 006](story-006-placement-drag-highlights.md) - Complete; provides valid target highlight semantics that must be preserved.
- Depends on: [Story 007](story-007-placement-instant-staging.md) - Complete; provides Instant fan plate staging semantics that must be preserved.
- Depends on: [Story 008](story-008-placement-unstaging.md) - Complete; provides correction by un-staging.
- Depends on: [Story 010](story-010-submit-prevalidation.md) - Complete; provides submit pre-validation behavior that must be preserved.
- Depends on: [Story 011](story-011-reserve-mana-strip.md) - Complete; provides reserve/current split controls and ceiling behavior.
- Depends on: ADR-002, ADR-021, and ADR-023 Accepted.
- Unlocks: A11Y-ST-14 evidence contribution for QA-COND-0005. Does not unlock QA-COND-0005 closure by itself.

## Blockers

None.

## Completion Notes

**Completed**: 2026-05-06
**Criteria**: 14/14 passing.
**Deviations**: None blocking. HAND-UI-014 implements and evidences only A11Y-ST-14 PLACEMENT staged disclosure; QA-COND-0005 remains Open because other Standard-tier accessibility rows still require implementation/evidence, reclassification, dependency-blocking, or accepted-risk disposition.
**Test Evidence**: `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`; capture artifacts under `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/`; `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test --jobs 1` passed 6/6; requested Hand UI regression groups passed 11/11, 13/13, and 25/25; `cargo fmt -p client -- --check`, `cargo check -p client --jobs 1`, and `git diff --check` passed.
**Code Review**: Skipped - lean mode. `production/review-mode.txt` is absent, so QL-TEST-COVERAGE and LP-CODE-REVIEW gates were not spawned.
**QA-COND-0005 Impact**: QA-COND-0005 remains Open. A11Y-ST-14 can move to implemented/evidenced in the Sprint 6 disposition register, but the broader condition is not closed by this story.
