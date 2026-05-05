# Standard-Tier Accessibility Disposition - Sprint 6

| Field | Value |
|---|---|
| Evidence ID | S6-04 Standard-tier accessibility disposition |
| Date | 2026-05-05 |
| QA condition | `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` |
| QA-COND-0005 status | Open |
| Source draft | `design/accessibility-requirements.md` (read-only draft for this pass) |
| Related UX draft | `design/ux/settings-accessibility.md` |
| GSS-008 implementation commit | [`4b505afa6bf465ea5b09360d4ef1d29859146f49`](https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/commit/4b505afa6bf465ea5b09360d4ef1d29859146f49) |
| GSS-008 evidence | `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md` |
| Disposition verdict | QA-COND-0005 remains Open. Only the PLACEMENT timer-extension sub-gap is implemented and verified by GSS-008. |

## Scope

This document records the Sprint 6 Standard-tier accessibility evidence
disposition after GSS-008. It does not change the accessibility requirements
draft, does not close QA-COND-0005, and does not claim verification for rows
that have not been verified.

GSS-008 supports exactly one closure movement:

- PLACEMENT timer extension is Implemented / Verified via GSS-008.
- QA-COND-0005 remains Open because the remaining Standard-tier rows still need
  implementation, evidence, reclassification, or accepted-risk disposition.

## GSS-008 Evidence Linkage

GSS-008 implemented the server-authoritative multiplayer PLACEMENT timer
extension path from ADR-023:

- implementation commit:
  [`4b505afa6bf465ea5b09360d4ef1d29859146f49`](https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/commit/4b505afa6bf465ea5b09360d4ef1d29859146f49)
- evidence:
  `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`
- story:
  `production/epics/game-session-system/story-008-placement-timer-multiplier-authority.md`

Relevant GSS-008 verification commands recorded in the evidence:

- `cargo test -p server --test placement_timer_multiplier_test`
- `cargo test -p server --test rsm_placement_timer_multiplier_test`
- `cargo test -p server --test reconnect_snapshot_test`
- `cargo test -p server --test game_config_defaults_test`
- `cargo test -p server --test session_ready_test`
- `cargo test -p server --test rsm_timers_test`
- `cargo test -p client --test hand_ui_placement_timer_test`
- `cargo test -p client --test presentation_plugin_scaffold_test`

Relevant test files from the GSS-008 verification set:

- `server/tests/placement_timer_multiplier_test.rs`
- `server/tests/rsm_placement_timer_multiplier_test.rs`
- `tests/integration/session/reconnect_snapshot_test.rs`
- `tests/integration/hand-ui/placement_timer_test.rs`
- `server/tests/game_config_defaults_test.rs`
- `server/tests/session_ready_test.rs`
- `server/tests/rsm_timers_test.rs`
- `tests/integration/presentation/presentation_plugin_scaffold_test.rs`

## Standard-Tier Checklist Disposition

Rows below are copied from the Standard-tier rows in
`design/accessibility-requirements.md`. The source draft status is not edited in
this evidence pass.

| Source row | Draft status | Disposition | Evidence state | Blocks QA-COND-0005 closure? |
|---|---|---|---|---|
| Minimum text size - HUD (gold, mana, round number) | Not Started | Needs new story | No browser/WASM measurement evidence for 20px resource counters or 40px auction price counter. | Yes |
| Minimum text size - card text (cost, ATK, HP, keyword) | Not Started | Needs new story | No browser/WASM measurement evidence for stat badges or keyword text floors. | Yes |
| Text contrast - UI on backgrounds | Not Started | Needs new story | No contrast audit evidence for 4.5:1 body text or 7:1 auction price counter. | Yes |
| Colorblind mode - Protanopia / Deuteranopia | Partially addressed | Implementation in progress | Art bible shape/icon backup exists in design, but settings toggle, palette implementation, and browser verification are not evidenced here. | Yes |
| Colorblind mode - Tritanopia | Not Started | Needs new story | No palette implementation or auction escalation readability evidence. | Yes |
| UI scaling | Not Started | Needs new story | No implementation or evidence for 75%-150% menu/HUD scaling, independent HUD scaling, or layout checks. | Yes |
| Motion / animation reduction mode | Not Started | Needs new story | No reduced-motion preference implementation or verification for auction panel entrance, bid pulse, phase-transition sweep, or related UI motion. | Yes |
| Full input remapping (keyboard + mouse) | Not Started | Needs new story | No implementation or evidence for rebindable player-facing actions, conflict blocking, persistence, or browser shortcut rejection. | Yes |
| PLACEMENT timer extension | Not Started in draft | Implemented / Verified via GSS-008 | GSS-008 evidence verifies multiplayer-safe values 1x, 1.5x, 2x, 3x; neutral highest-request-wins authority; freeze at `SessionReady`; RSM effective duration; reconnect snapshot; and Hand UI server-duration use. | No for this sub-gap |
| Hold-to-press alternatives | Not Started | Accepted risk candidate | No current evidence shows hold-to-confirm inputs exist or do not exist. Needs an explicit hold-input audit before producer can accept as risk. | Yes until accepted or remediated |
| DRAFT_SHOP ready signal - retractable | Addressed in design | Implemented + evidence needed | RSM logic evidence includes ready retraction in `server/tests/rsm_timers_test.rs`; no browser/UI evidence verifies visible retractable control behavior. | Yes until UI evidence is captured |
| Auction bid buttons - immediate preset commitments | Addressed in design | Implemented + evidence needed | `production/epics/shop-auction-ui/story-005-auction-bid-buttons-affordability-and-inflight.md` records passing bid-button tests; manual visual/accessibility evidence is deferred to SAU-009. | Yes until SAU-009/equivalent evidence is captured |
| Mana pools: distinct container shapes | Not Started | Needs new story | HUD reserve text exists in adjacent HUD evidence, but the required distinct current/reserve container shapes are not verified. | Yes |
| PLACEMENT staged disclosure | Not Started | Implementation in progress | Hand UI staging and submit behavior exists in prior stories, but the required guided staged-disclosure UX is not fully specified as verified evidence. | Yes |
| Tutorial persistence | Not Started | Needs new story | No Help/tutorial prompt registry, replay, reset, or persistence evidence. | Yes |
| Phase label always visible | Addressed in design | Implemented + evidence needed | HUD-003 logic evidence verifies phase label text updates; browser/WASM visibility, occlusion, and "not animation alone" evidence are not recorded here. | Yes until visual evidence is captured |
| Gold counter always visible | Addressed in design | Implemented + evidence needed | HUD-002/HUD-009/HUD-010 logic evidence covers gold display behavior, but browser/WASM occlusion and full-opacity visibility evidence are not recorded here. | Yes until visual evidence is captured |
| DRAFT_INITIAL: clear objective | Not Started | Needs new story | No implementation or evidence for the dismissible and retrievable start objective overlay. | Yes |
| Visual indicators for audio cues | Not Started | Needs new story | No complete gameplay-critical audio-cue audit. PLACEMENT timer presentation has adjacent Hand UI tests, but auction final-5s, RESOLUTION outcome, and full visual-backup verification are not evidenced here. | Yes |

## Basic Baseline Rows In The Standard Target

The source draft also contains Basic-tier rows. Because the project target is
Standard, these baseline rows need evidence or explicit reclassification before
the overall accessibility condition can close. They are not part of the GSS-008
verified timer sub-gap.

| Source row | Draft status | Disposition | Evidence state | Blocks QA-COND-0005 closure? |
|---|---|---|---|---|
| Color-as-only-indicator audit | Partially addressed | Implementation in progress | Art bible coverage exists for player side, class identity, auction escalation, and ATK/HP shapes; objective dots and damage/heal backup remain unverified. | Yes |
| Brightness / gamma controls | Not Started | Needs new story | No implementation or evidence for graphics settings, calibration preview, or contrast-preserving adjustment. | Yes |
| Screen flash warning | Not Started | Needs new story | No photosensitivity warning or RESOLUTION/GAME_OVER flash audit evidence. | Yes |
| Pause anywhere | Not Started | Needs new story | No implementation or evidence for safe-phase pause behavior, queued unsafe-phase pause request, or solo-play pause behavior. | Yes |
| Independent volume controls | Not Started | Needs new story | No implementation or evidence for independent Music, SFX, and UI audio buses with persistence. | Yes |
| No dialogue / voiced content | N/A | Reclassify out of Sprint 6 gate | Current design has no voiced dialogue; subtitle customisation should not block QA-COND-0005 unless voice is added. | No, if producer accepts the reclassification |

## Rows Preventing QA-COND-0005 Closure

QA-COND-0005 cannot close until every blocking row above is either verified,
implemented and evidenced, reclassified out of the Sprint 6 gate, or explicitly
accepted as risk by the user/producer.

Rows still preventing closure:

- Minimum text size - HUD.
- Minimum text size - card text.
- Text contrast - UI on backgrounds.
- Colorblind mode - Protanopia / Deuteranopia.
- Colorblind mode - Tritanopia.
- UI scaling.
- Motion / animation reduction mode.
- Full input remapping.
- Hold-to-press alternatives, unless the audit supports accepted-risk
  disposition and the producer accepts it.
- DRAFT_SHOP ready signal - retractable, until UI/browser evidence exists.
- Auction bid buttons - immediate preset commitments, until SAU-009 or
  equivalent evidence exists.
- Mana pools: distinct container shapes.
- PLACEMENT staged disclosure.
- Tutorial persistence.
- Phase label always visible, until UI/browser evidence exists.
- Gold counter always visible, until UI/browser evidence exists.
- DRAFT_INITIAL: clear objective.
- Visual indicators for audio cues.
- Color-as-only-indicator audit baseline gaps.
- Brightness / gamma controls.
- Screen flash warning.
- Pause anywhere.
- Independent volume controls.

The PLACEMENT timer-extension row no longer prevents QA-COND-0005 closure as an
individual sub-gap, because it is implemented and verified through GSS-008.

## Next Stories / Prompts Required

Use these exact follow-up story/prompt scopes to reduce or close the remaining
QA-COND-0005 gaps. These are documentation targets for the next implementation
passes; they are not implemented by this evidence file.

1. `S6-04A Settings/Accessibility screen implementation and persistence`
   - Build the Settings/Accessibility screen from
     `design/ux/settings-accessibility.md`.
   - Cover colorblind selector, reduced motion, PLACEMENT timer selector UI,
     menu UI scale, HUD UI scale, brightness/gamma controls, audio controls,
     input remapping, Help/tutorial access, keyboard navigation, focus rings,
     and persistence.
   - Reuse GSS-008 timer authority for the multiplayer timer selector; do not
     reimplement timer authority locally.

2. `S6-04B Browser/WASM text size, contrast, and UI scale evidence`
   - Capture browser/WASM evidence at 1366x768 and 1920x1080.
   - Verify HUD text floors, card text floors, auction price counter size,
     contrast ratios, and 75%/100%/150% menu and HUD scaling without overlap.

3. `S6-04C Colorblind modes and color-as-only gameplay audit`
   - Implement or verify Protanopia, Deuteranopia, and Tritanopia handling.
   - Verify player side, class identity, objective status, auction escalation,
     ATK/HP, and damage/heal signals have non-color backups.

4. `S6-04D Motion reduction and gameplay-critical visual backups`
   - Implement reduced-motion behavior for the motion-heavy UI listed in the
     draft.
   - Audit all gameplay-critical audio cues and verify visible backups for
     auction final-5s, PLACEMENT countdown, and RESOLUTION outcomes.

5. `S6-04E Input remapping and hold-to-press audit`
   - Implement conflict-free keyboard/mouse remapping and persistence.
   - Reject reserved browser/system shortcuts with inline feedback.
   - Audit all hold-to-confirm inputs; either implement alternatives or record a
     producer accepted-risk decision if no such inputs ship.

6. `S6-04F Cognitive support evidence`
   - Implement or verify distinct current/reserve mana container shapes.
   - Verify PLACEMENT staged disclosure.
   - Implement tutorial persistence, Help replay/reset, and DRAFT_INITIAL clear
     objective overlay.

7. `S6-04G Existing implemented-row browser evidence`
   - Capture browser/WASM evidence for DRAFT_SHOP ready/retract, auction bid
     buttons, phase label visibility, and gold counter visibility.
   - This may be merged with SAU-009 or split by owner if scheduling is easier.

8. `S6-04H Producer reclassification / accepted-risk decision`
   - Decide whether any rows should be reclassified out of the Sprint 6 gate or
     accepted as risk.
   - Until this decision is recorded, candidate rows still block
     QA-COND-0005 closure.

## Closure Rule

QA-COND-0005 remains Open. It can be closed only after:

- the PLACEMENT timer-extension sub-gap stays linked to GSS-008 evidence;
- all other Standard-tier rows above have implementation and browser/WASM
  evidence, or a recorded producer reclassification/accepted-risk disposition;
- the QA condition bug is updated with the complete evidence set; and
- the closure update explicitly states that no unverified Standard-tier blocker
  remains.
