# Story 003: Photosensitivity Warning and Flash Audit

> **Epic**: Accessibility Settings
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility disposition row `A11Y-BS-03` requires photosensitivity warning and flash-frequency audit evidence before release-gate confidence. The row may be producer-reclassified out of Sprint 6 must-implement only after the audit evidence is attached.

**Primary Sources**:

- `design/accessibility-requirements.md`: A11Y-BS-03 Screen flash warning. Requires a pre-launch photosensitivity notice and audit of RESOLUTION combat flash and GAME_OVER objective-destruction burst against the local max 3 flashes per second rule.
- `design/ux/settings-accessibility.md`: Settings / Accessibility owns sensory comfort and safe warning surfaces, while reduced motion is a distinct preference with later consumers.
- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`: A11Y-BS-03 is marked must implement in Sprint 6 and still blocks QA-COND-0005 closure.

**GDD Trace**:

- `design/gdd/card-animations.md` Visual/Audio Requirements state that impact flashes are flat 1-frame fills, objective destruction uses a 3-frame Prism White overlay over 240 ms, placement reveal uses a 3-frame flip over 80 to 100 ms, and GAME_OVER can override animation playback while preserving mandatory objective reveal.
- `design/gdd/board-rendering.md` Rule 9 and Rule 10 define RESOLUTION animation queue playback, GAME_OVER buffering, and mandatory objective reveal sequencing before GameOver transition.
- `design/gdd/board-rendering.md` Rule 12 defines objective destruction reveal behavior: 500 ms hold, real-reveal golden flash or fake crack overlay, sequential lane ordering, and slot clearing.
- `design/gdd/combat-resolution.md` Visual and UI requirements require RESOLUTION hit feedback, damage numbers, objective HP updates, objective damage, objective destruction, and no interactive elements during RESOLUTION.
- `design/gdd/hud.md` Rule 9 and Rule 10 require HUD persistence during RESOLUTION and frozen HUD behavior at GAME_OVER, while Rule 14 forbids HUD flashing, pulsing, urgency colors, and large scale tweens.
- `design/gdd/shop-auction-ui.md` timer, bid, settlement, and panel transition effects include timer color cross-fades, bid feedback, settlement overlays, and phase dismissal behavior that must be included in the flash inventory.

**TR IDs**: N/A. No registered `TR-AS-*` or `TR-A11Y-*` requirement exists yet. This story traces directly to the accessibility requirements row A11Y-BS-03 and the GDD/UX rules above.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: The warning and audit are client-side presentation/readiness work. Any warning UI is local presentation state only. Any gameplay state or phase behavior remains server-authoritative and driven by existing S2C messages. Animation behavior must remain inside ADR-021 presentation boundaries and must not create duplicate Lightyear drains.

**Engine**: Bevy 0.18 + WASM browser evidence | **Risk**: MEDIUM

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file. Use `liv-bevy-lightyear` only if implementation touches Lightyear/networking `.rs` files.

**Control Manifest Rules (2026-05-05)**:

- Required: UI always uses bevy_ui for panels, HUD, hand fan, shop panels, and warning surfaces.
- Required: `PresentationSet` order is `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: all client UI reads phase through shared presentation resources. Do not add a second `MessageReceiver<S2CPhaseChanged>` drain.
- Forbidden: never let client-local Settings or warnings alter server-authoritative phase, timer, auction, combat, objective, or GAME_OVER state.
- Forbidden: never implement reduced-motion consumers as broad scope creep in this story unless the audit evidence identifies a release-gate blocker and the producer explicitly requires that remediation.
- Guardrail: presentation steady-state stays below 1 ms per frame; phase-boundary UI toggles stay below a 3 ms spike.

---

## Scope

### In Scope

- Create a photosensitivity and flash-frequency audit for all shipped or planned release-gate visual effects that can flash, flicker, pulse, strobe, burst, or create full-screen high-contrast exposure.
- Audit these exact categories:
  - RESOLUTION playback.
  - GAME_OVER transition and result-entry effects.
  - Combat hit and impact flash effects.
  - Objective damage and objective destruction effects.
  - Phase transition effects.
  - Animation specs in Card Animations, Board Rendering, HUD, Shop/Auction UI, and Settings / Accessibility.
- Record for each audited effect:
  - Source document and rule.
  - Owner system.
  - Triggering phase or event.
  - Flash count per second or declared not-flashing.
  - Full-screen or partial-screen exposure.
  - Red-flash exposure if present.
  - Reduced-motion interaction if already specified.
  - Required remediation, warning, producer decision, or no-action disposition.
- Add or require a pre-launch photosensitivity warning if the audit concludes the warning is needed now for release-gate confidence.
- If the producer reclassifies A11Y-BS-03 out of Sprint 6 must-implement, record the decision only after the audit table is attached in the exact evidence file path.
- Preserve QA-COND-0005 as Open.

### Out of Scope

- Do not implement full reduced-motion mode or reduced-motion consumers across HUD, Hand UI, Shop/Auction UI, Board Rendering, Card Animations, or Settings unless this story's audit evidence identifies a specific release-gate blocker and producer explicitly requires a scoped remediation.
- Do not redesign combat, objective, GAME_OVER, or phase-transition animations beyond specific audit-driven safety remediations.
- Do not implement brightness/gamma controls, independent volume controls, colorblind modes, input remapping, pause-anywhere behavior, or Help/tutorial persistence.
- Do not close QA-COND-0005 from this story alone.
- Do not modify sprint status, session-state files, the project asset directory, or `AGENTS.md`.

---

## Acceptance Criteria

- [ ] The evidence file exists at exactly `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`.
- [ ] The evidence file contains an audit table covering RESOLUTION playback, GAME_OVER transition and result-entry effects, combat hit and impact flash effects, objective damage and objective destruction effects, phase transition effects, and animation specs.
- [ ] Every audit row includes source document, owner system, triggering phase or event, flash count per second or declared not-flashing, full-screen or partial-screen exposure, red-flash exposure, reduced-motion interaction if already specified, and final disposition.
- [ ] The audit explicitly covers `design/gdd/card-animations.md`, `design/gdd/board-rendering.md`, `design/gdd/combat-resolution.md`, `design/gdd/objective-system.md`, `design/gdd/hud.md`, `design/gdd/shop-auction-ui.md`, `design/ux/interaction-patterns.md`, and `design/ux/settings-accessibility.md`.
- [ ] Effects that exceed or cannot prove compliance with the local max 3 flashes per second rule are assigned one of these dispositions: scoped remediation required, warning implemented now, producer reclassification after audit, or accepted risk with producer signoff.
- [ ] Effects declared not-flashing identify the reason, such as color cross-fade, static state change, non-repeating single-frame impact, or no full-screen exposure.
- [ ] The evidence file records whether the photosensitivity warning is implemented now or whether the producer reclassifies A11Y-BS-03 out of Sprint 6 must-implement after reviewing the attached audit.
- [ ] If the warning is implemented now, the implemented warning copy is stored in one test-observable source and appears before gameplay exposure in a title, boot, Settings, or equivalent pre-match surface.
- [ ] If the warning is implemented now, browser/WASM evidence shows the warning before entering a match and records its dismissal or acknowledgment behavior.
- [ ] If the warning is not implemented now, the evidence file contains a dated producer reclassification block that includes row ID A11Y-BS-03, decision text, release-gate rationale, follow-up owner, and follow-up timing.
- [ ] A11Y-BS-03 is not marked complete, accepted-risk, or reclassified unless the audit evidence file above exists and contains the completed audit table.
- [ ] QA-COND-0005 remains Open and the evidence file states that this story does not close QA-COND-0005 by itself.
- [ ] No full reduced-motion implementation is included unless the evidence file identifies a specific release-gate blocker and records explicit producer instruction for that scoped remediation.
- [ ] `git diff --check` passes.

---

## Implementation Notes

- Prefer making the audit evidence the central deliverable. The warning implementation is conditional on audit and producer disposition, but the audit itself is mandatory.
- Treat the local max 3 flashes per second rule from `design/accessibility-requirements.md` as the release-gate threshold for this story.
- The audit should distinguish:
  - Color cross-fades from flashes.
  - One-frame impact fills from repeated flashes.
  - Partial-screen effects from full-screen or large-area exposure.
  - Red flash exposure from non-red high-contrast flash exposure.
- If warning UI is implemented, keep it local to presentation. It must not block network handshake, session authority, reconnect, or phase state. It can gate the player's voluntary entry into gameplay surfaces, but it must not alter authoritative game state after a match starts.
- Use a single text source for warning copy so tests and evidence can assert exact copy without duplicating strings.
- Do not use this story to build the reduced-motion preference consumers. If audit finds unsafe repeated flashing, scope a direct flash-safety remediation or record producer disposition rather than expanding into full reduced-motion mode.

## Performance Budget

No gameplay-loop performance impact expected from the audit itself. If warning UI is implemented, it is a title, boot, Settings, or pre-match presentation surface with no steady-state in-match work after dismissal or acknowledgment. Any warning UI toggle must remain within the ADR-021 presentation guardrail: below 1 ms steady-state UI work and below a 3 ms opening or dismissal spike.

---

## QA Test Cases

- **Audit file completeness**
  - Given: the story is being closed
  - When: `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md` is inspected
  - Then: the audit table contains rows for every required category and every required source document

- **Flash-frequency disposition**
  - Given: each audited effect has a recorded flash count or not-flashing declaration
  - When: an effect exceeds or cannot prove compliance with the local max 3 flashes per second rule
  - Then: the row has a final disposition of scoped remediation required, warning implemented now, producer reclassification after audit, or accepted risk with producer signoff

- **Warning implemented path**
  - Given: the audit requires implementing the photosensitivity warning now
  - When: the player reaches the selected pre-game warning surface
  - Then: the warning copy is visible before gameplay exposure, can be dismissed or acknowledged as specified, and browser/WASM evidence captures the behavior

- **Producer reclassification path**
  - Given: the producer reclassifies A11Y-BS-03 out of Sprint 6 must-implement
  - When: the evidence file is inspected
  - Then: the dated producer block includes row ID A11Y-BS-03, decision text, release-gate rationale, follow-up owner, follow-up timing, and a link back to the completed audit table

- **Reduced-motion scope guard**
  - Given: the story changes are reviewed
  - When: changed implementation files are inspected
  - Then: no broad reduced-motion consumers are implemented unless the evidence file records a specific release-gate blocker and explicit producer instruction for that scoped remediation

---

## Test Evidence

**Story Type**: Config/Data

**Required evidence document**:

- `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`

**Required automated test targets if warning UI is implemented now**:

- `tests/unit/accessibility_settings/photosensitivity_warning_test.rs`
  - Registered as `accessibility_settings_photosensitivity_warning_test`
  - Command: `cargo test -p client --test accessibility_settings_photosensitivity_warning_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required evidence contents**:

- Audit inventory table for all required categories and source documents.
- Flash-frequency assessment against the local max 3 flashes per second rule.
- Full-screen exposure assessment.
- Red-flash exposure assessment.
- Final disposition for each audited effect.
- One final A11Y-BS-03 decision block with either warning implemented now or producer reclassification after audit.
- QA-COND-0005 impact statement.

**QA-COND-0005 impact statement required in evidence**:

Story 003 supplies the A11Y-BS-03 photosensitivity warning and flash-frequency audit evidence required for release-gate confidence or producer reclassification. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, or accepted as risk.

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `design/accessibility-requirements.md` A11Y-BS-03 source row and `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md` Sprint 6 disposition register.
- Depends on: [Story 001](story-001-settings-accessibility-foundation-and-preferences.md) - Ready, only if warning UI is implemented through the Settings / Accessibility surface. The audit evidence path itself can be produced without waiting for Story 001 implementation.
- Depends on: ADR-002 and ADR-021 Accepted.
- Unlocks: A11Y-BS-03 release-gate disposition evidence for QA-COND-0005. Does not unlock QA-COND-0005 closure by itself.

## Blockers

None.
