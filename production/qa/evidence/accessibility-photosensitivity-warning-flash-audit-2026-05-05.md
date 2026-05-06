# Photosensitivity Warning and Flash-Frequency Audit

| Field | Value |
|---|---|
| Evidence ID | A11Y-BS-03 photosensitivity warning and flash-frequency audit |
| Date | 2026-05-05 |
| Story | `production/epics/accessibility-settings/story-003-photosensitivity-warning-and-flash-audit.md` |
| QA condition | `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` |
| QA-COND-0005 status | Open |
| Local flash rule | Maximum 3 flashes per second |
| Warning implementation | Implemented now in `client/src/ui/photosensitivity_warning.rs` |
| Warning test target | `cargo test -p client --test accessibility_settings_photosensitivity_warning_test` |

## Warning Copy Source

The implemented warning copy is stored once in
`client::ui::photosensitivity_warning::PHOTOSENSITIVITY_WARNING_COPY`:

> Lanes and Lies uses brief impact flashes, timer urgency effects, objective-destruction bursts, and phase transitions. Stop playing and consult a medical professional if you feel discomfort, dizziness, eye twitching, or nausea.

The warning is spawned by `PhotosensitivityWarningPlugin` on boot as a local
presentation UI overlay. It does not alter server-authoritative phase, timer,
auction, combat, objective, or GAME_OVER state. The overlay can be acknowledged
through `PhotosensitivityWarningAcknowledged` or the `PhotosensitivityWarningAcknowledge`
interaction; acknowledgement hides the overlay for the current app run.

## Audit Table

| Category | Source document and rule | Owner system | Triggering phase or event | Flash count per second / not-flashing declaration | Exposure | Red-flash exposure | Reduced-motion interaction already specified | Final disposition |
|---|---|---|---|---|---|---|---|---|
| RESOLUTION playback queue | `design/gdd/board-rendering.md` Rule 9 and F4; `design/gdd/card-animations.md` Rule C-8 | Board Rendering / Card Animations | `S2CResolutionEvent` during RESOLUTION | Not a flash sequence. Groups are timed movement, fades, numbers, and state changes over 600 ms sub-step windows with 150 ms pauses. | Partial-screen board entities; no full-screen exposure except objective destruction rows below. | None from the queue itself. | Reduced-motion mode is future Settings scope; no consumer implemented in this story. | No action for queue timing. Warning implemented now covers residual sensory risk. |
| GAME_OVER transition | `design/gdd/board-rendering.md` EC-PHASE-GAMEOVER; `design/gdd/hud.md` Rule 10 | Board Rendering / HUD | `S2CPhaseChanged(GAME_OVER)` after RESOLUTION | HUD phase label changes instantly and freezes. Board may skip remaining groups but must complete objective reveal first. Not flashing by itself. | HUD-only text state plus board state; no full-screen exposure from HUD. | None. | `design/ux/settings-accessibility.md` allows GAME_OVER Settings; reduced-motion consumers are future scope. | No remediation. Warning implemented now. |
| GAME_OVER result-entry / settlement-style effects | `design/gdd/hud.md` Rule 10; `design/ux/interaction-patterns.md` PTN-OVR-003 gap for future result panel | Future result screen / HUD | GAME_OVER result entry | Not fully specified. Existing HUD stays quiet and frozen; future result panel must be re-audited before implementation. | Unknown future result panel; current HUD exposure is partial-screen and static. | None in current HUD. | Not specified for future result panel. | Warning implemented now; future GAME_OVER result panel requires a follow-up flash audit before ship. |
| Combat FIRST STRIKE hit / impact flash | `design/gdd/combat-resolution.md` Visual/Audio Requirements; `design/gdd/card-animations.md` V.1 and V.3 | Combat Resolution / Card Animations | FIRST STRIKE impact in RESOLUTION sub-step 3 | One 1-frame Prism White impact fill per hit. A single frame at 60 FPS is about 16.7 ms; repeated hits are separated by sub-step timing, not a strobe loop. | Partial-screen at impacted unit/cell. | None; Prism White. | Reduced-motion not specified for necessary combat readability; unit movement remains necessary. | Compliant with local rule as specified. Warning implemented now for sensitive players. |
| Standard combat hit / impact flash | `design/gdd/combat-resolution.md` Visual/Audio Requirements; `design/gdd/card-animations.md` V.1 and V.3 | Combat Resolution / Card Animations | Standard combat impact in RESOLUTION sub-step 6 | One 1-frame Warm Orange impact fill per hit; not a repeating strobe. | Partial-screen at impacted unit/cell. | No red flash; warm orange. | Reduced-motion not specified for necessary combat readability. | Compliant with local rule as specified. Warning implemented now. |
| Damage number fade / float | `design/gdd/combat-resolution.md` R4; `design/gdd/card-animations.md` damage-number lifecycle | Card Animations | Combat damage event | Not flashing. Text floats and fades over about 500 ms. Multiple sources are offset to avoid overlap. | Partial-screen world-space text above target. | Damage color is red/crimson family but fades, not flashes. | No reduced-motion consumer specified. | No action. |
| Objective damage HP feedback | `design/gdd/combat-resolution.md` R6; `design/gdd/board-rendering.md` Rule 6; `design/gdd/objective-system.md` D1 | Objective System / Board Rendering / HUD | Objective takes damage in RESOLUTION sub-step 6 | Not flashing. HP bar or number changes are state updates; HUD dot does not animate. | Partial-screen objective/HUD elements. | Objective damage text may use Crimson Slate but is not a repeated flash. | No reduced-motion consumer specified. | No action. |
| Objective destruction burst | `design/gdd/combat-resolution.md` Visual/Audio Requirements; `design/gdd/board-rendering.md` Rule 12 and F4; `design/gdd/objective-system.md` Reveal Moment | Board Rendering / Card Animations | `ObjectiveDestroyed` reveal after 500 ms hold | Full-screen Prism White overlay is 3 frames over about 240 ms: 3 flashes / 0.24 s equals 12.5 flashes/sec if counted as separate flashes, and therefore exceeds the local rule or cannot prove compliance. | Full-screen high-contrast exposure. | None; Prism White, but high contrast. | Reduced-motion mode is future scope; no consumer exists now. | Warning implemented now. Follow-up remediation required before release if producer requires strict local-rule compliance rather than warning-only disposition. Candidate remediation: replace 3-frame full-screen overlay with one non-repeating fade/hold or lane-local reveal. |
| Real objective reveal lane flood | `design/gdd/combat-resolution.md` Visual/Audio Requirements; `design/gdd/board-rendering.md` Rule 12 | Board Rendering | Real objective reveal | Not flashing. Warm gold fill floods lane for about 400 ms. | Lane column partial-screen. | None. | No reduced-motion consumer specified. | No action. |
| Fake objective reveal glyph / crack overlay | `design/gdd/combat-resolution.md` Visual/Audio Requirements; `design/gdd/board-rendering.md` Rule 12; `design/gdd/objective-system.md` Reveal Moment | Board Rendering | Fake objective reveal | Not flashing. Static crack overlay and `?` glyph scale/dissolve over about 200 ms after hold. | Objective/lane-local partial-screen. | None. | No reduced-motion consumer specified. | No action. |
| HUD objective dot destruction | `design/gdd/hud.md` Rules 6, 7, and 14 | HUD | `HudObjectiveUpdate` after Board Rendering drains objective destruction | Declared not flashing. ALIVE to DESTROYED is instantaneous and has no tween, sparkle, pulse, or identity reveal. | Small HUD dot partial-screen. | None. | Reduced motion not needed because no animation. | No action. |
| Routine HUD phase label changes | `design/gdd/hud.md` Rules 5, 9, 10, and 14 | HUD | `S2CPhaseChanged` | Declared not flashing. Text changes in place; no fade, flash, pulse, urgency color, or scale tween. | Small HUD text. | None. | Reduced motion not needed because no animation. | No action. |
| HUD numeric gold/mana tweens | `design/gdd/hud.md` Rule 14 | HUD | `S2CGoldUpdate`, `S2CGoldBroadcast`, snapshot rebuild | Not flashing. Numeric tween max 300 ms; forbidden from flashing, pulsing, urgency colors, and large scale tweens. | HUD readout partial-screen. | None. | Reduced motion not specified; current spec already restrained. | No action. |
| DRAFT_INITIAL and DRAFT_SHOP timer color urgency | `design/gdd/shop-auction-ui.md` Rules 5 and D.5c; `design/ux/interaction-patterns.md` PTN-DSP-005 | Shop/Auction UI | Timer crosses yellow/red thresholds | Not a flash. Color changes cross-fade over 300 ms. Red-zone pulse is specified as 2 Hz fill opacity and one vertical swell per second, under 3 flashes/sec if implemented exactly. | Timer bar partial-screen. | Red/crimson urgency exposure present. | `design/ux/settings-accessibility.md` says reduced motion removes timer pulse, but consumer is future scope. | Warning implemented now; reduced-motion consumer remains future story, not implemented here. |
| DRAFT_AUCTION timer red zone and pulse | `design/gdd/shop-auction-ui.md` Rules 3 and VA.3 | Shop/Auction UI / Auction UI | Auction timer under 5 seconds | Red-zone pulse is 2 Hz and therefore under local 3 flashes/sec if implemented as specified. Color transitions cross-fade over 300 ms. | Timer bar partial-screen. | Crimson-Amber red-zone exposure present. | Reduced-motion consumer future scope. | Warning implemented now; no broad reduced-motion implementation in this story. |
| DRAFT_AUCTION bid-accepted reset flash | `design/gdd/shop-auction-ui.md` Rule 6 and VA.3 | Shop/Auction UI | `S2CAuctionBidAccepted` | One 60 ms Prism White flash at 80% opacity; not repeating. | Timer bar partial-screen. | None; Prism White. | Reduced-motion consumer future scope. | Compliant as single flash. Warning implemented now. |
| DRAFT_AUCTION settlement overlays | `design/gdd/shop-auction-ui.md` Rule 9 and VA.5; `design/ux/interaction-patterns.md` PTN-OVR-002 | Shop/Auction UI | `S2CAuctionSettled` | Not flashing. Overlays hold 400 ms or 1.0 s; NO BIDS desaturates/fades. | Panel-level partial-screen only; board and HUD remain visible. | None. | Interaction patterns specify reduced motion uses static flash/cut for settlement, but consumer is future scope. | No action. |
| Shop purchase / insufficient-gold feedback | `design/gdd/shop-auction-ui.md` Rule 3; `design/ux/interaction-patterns.md` PTN-FDB-002 | Shop/Auction UI | Invalid purchase attempt | Single brief tint/flash, 150-200 ms depending source; not repeated. | Local input/counter partial-screen. | Red/amber exposure present but local and non-repeating. | Reduced-motion not specified. | Compliant as single flash. Warning implemented now. |
| Placement reveal flip | `design/gdd/card-animations.md` Rule C-2 and Visual/Audio Requirements; `design/gdd/board-rendering.md` Rule 7 | Card Animations / Board Rendering | `S2CPlacementReveal` at RESOLUTION entry | 3-frame flip over 80-100 ms; one Prism White edge-on squash flash and one 1-frame player-side color ring. This is at most two single-frame flashes per unit reveal, not a repeating full-screen strobe. | Partial-screen per unit across lanes; multiple lanes simultaneous but not full-screen. | No red; player-side Terracotta may be orange-red for Player B. | Reduced-motion not specified for this necessary reveal. | Compliant as specified. Warning implemented now. |
| Board unit reveal tween | `design/gdd/board-rendering.md` Rule 7 | Board Rendering | Opponent newly replicated placement reveal | Not flashing. Scale and alpha tween over 250 ms. | Partial-screen board entities. | None. | No reduced-motion consumer specified. | No action. |
| Phase-transition fades / panel transitions | `design/gdd/card-animations.md` Rule C-2; `design/gdd/shop-auction-ui.md` panel transitions; `design/ux/settings-accessibility.md` Transitions & Animations | Presentation / Shop-Auction / Settings future | Phase entry/exit, Settings open/close, category switches | Declared not flashing. Fades, slides, or cross-fades with durations 80-350 ms; no strobe loop. | Panel or UI-region partial-screen; Settings dim is full-screen but static/fade only. | None. | Settings UX specifies reduced motion cuts/fades and prohibits repeated glow/pulse. | No action for this story. |
| Settings / Accessibility animation specs | `design/ux/settings-accessibility.md` Transitions & Animations | Future Settings / Accessibility | Settings open/close, category switch, capture modal | Declared not flashing. Prohibited motion includes no repeating glow/pulse except optional short focus/confirmation highlight and no animation hiding text. | Settings panel/full-screen dim; static/fade only. | None. | Reduced-motion alternatives specified in UX. | No action; full Settings foundation out of scope. |
| Interaction pattern red flash | `design/ux/interaction-patterns.md` PTN-FDB-002 | Shared UI patterns | Validation error feedback | Single 150 ms red flash with persistent text error. Not repeated and not full-screen. | Local input/control partial-screen. | Red flash exposure present. | Not specified. | Compliant as single local flash. Warning implemented now. |
| Interaction pattern resource/selection pulses | `design/ux/interaction-patterns.md` PTN-DSP-002, PTN-DSP-003, PTN-DSP-008, PTN-OVR-001 | Shared UI patterns | Resource gain, selection, waiting slot | Not a flash if implemented as specified: pulses are slow scale/opacity cues and not full-screen. Some are continuous or repeated but below 3 Hz where a frequency is given. | Local UI partial-screen. | None unless owning system uses red error state. | Reduced-motion patterns replace pulses with static/fade where specified. | No release-gate blocker found. |

## Effects Requiring Follow-Up

| Effect | Reason | Follow-up owner |
|---|---|---|
| Objective destruction full-screen Prism White overlay | The specified 3-frame / 240 ms full-screen overlay cannot prove compliance with the local max 3 flashes per second rule. | Board Rendering / Card Animations, with producer decision before release if warning-only disposition is not sufficient. |
| Future GAME_OVER result panel | The result screen is not fully designed. A future implementation must not introduce un-audited full-screen flashes. | Future GAME_OVER screen owner. |
| Reduced-motion consumers | UX specifies reduced-motion alternatives for repeated pulses and frame flicker, but this story explicitly avoids broad reduced-motion implementation. | Accessibility Settings follow-up story. |

## A11Y-BS-03 Decision Block

| Field | Value |
|---|---|
| Row ID | A11Y-BS-03 |
| Decision | Warning implemented now plus audit evidence attached. |
| Release-gate rationale | A formal audit is now attached. Most specified effects are local, one-shot, cross-fade, or non-flashing. Objective destruction remains the only effect that exceeds or cannot prove compliance with the local max 3 flashes per second rule, so the pre-launch warning is implemented now while scoped remediation or producer disposition remains available before release. |
| Follow-up owner | Producer with Board Rendering / Card Animations owner for objective destruction remediation decision. |
| Follow-up timing | Before Production -> Polish release gate if warning-only disposition is insufficient. |

## Browser / WASM Warning Evidence

Automated UI evidence is captured by
`tests/unit/accessibility_settings/photosensitivity_warning_test.rs`:

- The warning copy is stored in one test-observable source constant.
- `PhotosensitivityWarningPlugin` spawns the visible warning at boot before any
  gameplay state is entered.
- The warning remains visible if `ClientState::InSession` is entered before
  acknowledgement.
- Acknowledgement through either the message path or interaction path marks the
  warning acknowledged and hides the overlay.

No browser screenshot is attached in this evidence pass. The test-observable boot
surface is the current local proof because the client title/menu browser harness
does not yet exist on `origin/main`.

## QA-COND-0005 Impact Statement

Story 003 supplies the A11Y-BS-03 photosensitivity warning and flash-frequency
audit evidence required for release-gate confidence or producer reclassification.
It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all
remaining Standard-tier rows are implemented and evidenced, reclassified, or
accepted as risk.

## Verification Log

| Command | Result |
|---|---|
| `cargo test -p client --test accessibility_settings_photosensitivity_warning_test` | Passed post-rebase: 4 passed, 0 failed. |
| `cargo fmt -p client -- --check` | Passed post-rebase. |
| `cargo check -p client` | Passed post-rebase. |
| `git diff --check` | Passed post-rebase. |
