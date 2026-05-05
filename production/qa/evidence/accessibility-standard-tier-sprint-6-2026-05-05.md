# Standard-Tier Accessibility Disposition Register - Sprint 6

| Field | Value |
|---|---|
| Evidence ID | S6-04 Standard-tier accessibility disposition |
| Date | 2026-05-05 |
| QA condition | `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` |
| QA-COND-0005 status | Open |
| Source draft | `design/accessibility-requirements.md` |
| Related UX draft | `design/ux/settings-accessibility.md` |
| GSS-008 evidence | `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md` |
| Disposition verdict | QA-COND-0005 remains Open. Every source row now has exactly one allowed final disposition. Only A11Y-ST-09 is implemented and evidenced. |

## Scope

This register records the Sprint 6 S6-04 disposition of every Standard-tier
accessibility source row and every Sprint 6 dependency row. It does not edit the
accessibility requirements draft, does not implement code, does not close
QA-COND-0005, and does not claim evidence that has not been captured.

Allowed final dispositions used by this register:

- implemented + evidence attached
- evidence-only required
- must implement in Sprint 6
- later sprint / blocked dependency

No row currently uses `accepted risk with producer signoff` or `reclassified out
of Production -> Polish gate`, because no producer signoff block is present in
the source evidence for this pass.

## GSS-008 Evidence Linkage

GSS-008 implemented the server-authoritative multiplayer PLACEMENT timer
extension path from ADR-023:

- implementation commit:
  `4b505afa6bf465ea5b09360d4ef1d29859146f49`
- evidence:
  `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`
- story:
  `production/epics/game-session-system/story-008-placement-timer-multiplier-authority.md`

GSS-008 verification covers multiplayer-safe values 1x, 1.5x, 2x, and 3x;
excludes 0.5x from multiplayer Standard-tier negotiation; freezes the neutral
effective multiplier at `SessionReady`; applies the frozen value through the
RSM PLACEMENT duration; carries the duration in server phase/snapshot data; and
keeps client presentation from recomputing the active timer locally.

## Source Row Disposition Register

| Row ID | Source row | Tier | Current evidence | Final disposition | Required audit or test | Producer signoff required | Signoff evidence | QA-COND-0005 impact | Follow-up path |
|---|---|---|---|---|---|---|---|---|---|
| A11Y-ST-01 | Minimum text size - HUD | Standard | No browser/WASM measurement evidence for 20px resource counters or 40px auction price counter. | evidence-only required | Browser/WASM capture at 1366x768 and 1920x1080 measuring HUD gold, mana, round, and auction price text floors. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 006 or equivalent browser evidence pass. |
| A11Y-ST-02 | Minimum text size - card text | Standard | No browser/WASM measurement evidence for card cost, ATK, HP, or keyword text floors. | evidence-only required | Browser/WASM card readability capture measuring stat badges at 18px minimum and keyword text at 14px minimum. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 006 or equivalent browser evidence pass. |
| A11Y-ST-03 | Text contrast - UI on backgrounds | Standard | No contrast audit evidence for 4.5:1 body text or 7:1 auction price counter. | evidence-only required | Contrast audit against browser/WASM screenshots for HUD, cards, Settings, Shop/Auction, and auction price counter. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 006 or equivalent contrast evidence pass. |
| A11Y-ST-04 | Colorblind mode - Protanopia / Deuteranopia | Standard | Art bible and design docs define shape/icon backups, but no settings toggle, palette implementation, or browser evidence is attached. | later sprint / blocked dependency | Implement and verify Protanopia and Deuteranopia selector, palette application, and shape/icon backup browser evidence. | No | N/A | Blocks closure. | A11Y-DEP-01, A11Y-DEP-02, `production/epics/accessibility-settings/EPIC.md` story 002. |
| A11Y-ST-05 | Colorblind mode - Tritanopia | Standard | No Tritanopia palette implementation or auction escalation readability evidence is attached. | later sprint / blocked dependency | Implement and verify Tritanopia selector, palette application, auction escalation readability, and shape/icon backups. | No | N/A | Blocks closure. | A11Y-DEP-01, A11Y-DEP-02, `production/epics/accessibility-settings/EPIC.md` story 002. |
| A11Y-ST-06 | UI scaling | Standard | Settings UX specifies 75%-150% menu and HUD scaling, but no implementation or layout evidence is attached. | later sprint / blocked dependency | Implement menu and HUD scale preferences, persistence, and browser layout checks at 75%, 100%, and 150%. | No | N/A | Blocks closure. | A11Y-DEP-01, A11Y-DEP-02, `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md`. |
| A11Y-ST-07 | Motion / animation reduction mode | Standard | UX specs list motion exposures and reduced-motion alternatives, but no preference implementation or consumer evidence is attached. | later sprint / blocked dependency | Implement reduced-motion preference and verify consumers for auction entrance, bid pulse, timer pulse, phase transitions, class reveal, panel motion, frame flicker, and nonessential scale motion. | No | N/A | Blocks closure. | A11Y-DEP-01, A11Y-DEP-02, `production/epics/accessibility-settings/EPIC.md` story 003. |
| A11Y-ST-08 | Full input remapping | Standard | Settings UX defines input remapping requirements, but no canonical input action registry, conflict blocking, reserved shortcut rejection, or persistence evidence is attached. | later sprint / blocked dependency | Define canonical action registry, implement keyboard/mouse remapping, reject conflicts and browser shortcuts, and verify persistence. | No | N/A | Blocks closure. | A11Y-DEP-03, `production/epics/accessibility-settings/EPIC.md` story 004. |
| A11Y-ST-09 | PLACEMENT timer extension | Standard | GSS-008 evidence verifies server-authoritative values 1x, 1.5x, 2x, 3x; neutral highest-request-wins authority; freeze at `SessionReady`; RSM effective duration; reconnect snapshot; and Hand UI server-duration use. | implemented + evidence attached | Retain GSS-008 regression commands and `git diff --check` evidence from `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`. | No | N/A | Closes sub-gap. | No follow-up for this row unless timer authority regresses. QA-COND-0005 as a whole remains Open. |
| A11Y-ST-10 | Hold-to-press alternatives | Standard | Spot audit found `design/ux/hand-ui.md` states no hold-to-confirm interaction exists in Hand UI. Full UX and implementation audit is not attached. | evidence-only required | Audit UX specs and implementation for hold-to-confirm, long-press, press-and-hold, timer-gated button hold, and pointer-held flows. If any shipped hold flow exists, implement an alternative or obtain producer accepted-risk signoff per flow. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 004. |
| A11Y-ST-11 | DRAFT_SHOP ready signal - retractable | Standard | RSM logic evidence covers ready retraction in `server/tests/rsm_timers_test.rs`; no browser/UI evidence verifies visible retractable control behavior. | evidence-only required | Browser/WASM manual or automated evidence showing ready can be retracted before all-ready fires and that the control is visibly reversible. | No | N/A | Blocks closure. | Existing Shop/Auction evidence pass, SAU-009 equivalent, or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-12 | Auction bid buttons - immediate preset commitments | Standard | SAU-005 records bid-button tests and immediate preset commitment behavior; manual visual/accessibility evidence remains deferred. | evidence-only required | Browser/WASM capture showing preset total commitment labels, 44x44 targets, focus rings, affordability gating, in-flight disable, one-send semantics, and BIDDING feedback. | No | N/A | Blocks closure. | SAU-009 equivalent or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-13 | Mana pools: distinct container shapes | Standard | HUD/UX requirements call for current mana bar and reserve mana diamond shapes, but no implementation confirmation or browser evidence is attached. | must implement in Sprint 6 | Implement or verify distinct current/reserve mana container shapes and capture browser/WASM evidence proving shape distinction without color. | No | N/A | Blocks closure. | Sprint 6 HUD/accessibility implementation story or `production/epics/accessibility-settings/EPIC.md` story 006 after implementation exists. |
| A11Y-ST-14 | PLACEMENT staged disclosure | Standard | Hand UI staging behavior exists in prior stories, but the guided staged-disclosure UX is not fully verified as browser evidence. | evidence-only required | Browser/WASM capture showing card selection, lane selection, cell selection, and mana split/submit disclosure sequence without showing later controls prematurely. | No | N/A | Blocks closure. | Hand UI evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-15 | Tutorial persistence | Standard | Settings UX defines Help replay/reset behavior, but no tutorial/help prompt registry, replay, reset, or persistence implementation exists. | later sprint / blocked dependency | Implement prompt registry, dismissed-prompt persistence, Help replay, reset-all, and per-prompt reset evidence. | No | N/A | Blocks closure. | A11Y-DEP-06, `production/epics/accessibility-settings/EPIC.md` story 005. |
| A11Y-ST-16 | Phase label always visible | Standard | HUD logic evidence verifies phase label text updates; no browser/WASM evidence verifies visibility, non-occlusion, and non-animation-only phase communication. | evidence-only required | Browser/WASM capture across phases proving phase label remains visible, readable, non-occluded, and not communicated by animation alone. | No | N/A | Blocks closure. | HUD evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-17 | Gold counter always visible | Standard | HUD and economy logic evidence covers gold display behavior; no browser/WASM occlusion or full-opacity visibility evidence is attached. | evidence-only required | Browser/WASM capture across DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT, RESOLUTION, and GAME_OVER proving gold counter visibility and opacity. | No | N/A | Blocks closure. | HUD/Shop-Auction evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-18 | DRAFT_INITIAL: clear objective | Standard | No implementation or evidence for the dismissible and retrievable start objective overlay. | must implement in Sprint 6 | Implement and capture evidence for the start objective overlay text, dismissal, retrieval path, and browser readability. | No | N/A | Blocks closure. | Sprint 6 cognitive support story or `production/epics/accessibility-settings/EPIC.md` story 005 plus DRAFT_INITIAL UI work. |
| A11Y-ST-19 | Visual indicators for audio cues | Standard | Source docs identify timer, auction, RESOLUTION, and objective audio cues. Hand UI has visible timer number and `TimerUrgencyAudio`, but no complete gameplay-critical audio-cue audit is attached. | evidence-only required | Audit all gameplay-critical audio cues and map each cue to visible backup, non-shipping audio status, or implementation dependency. Any shipped audio-only critical cue must be remediated or producer-signed as risk. | No | N/A | Blocks closure. | A11Y-DEP-04, `production/epics/accessibility-settings/EPIC.md` story 003. |
| A11Y-BS-01 | Color-as-only-indicator audit | Basic baseline under Standard target | Art bible and design docs cover some shape/text backups; objective dots and damage/heal backup remain unverified. | evidence-only required | Color-as-only audit for player side, class identity, objective status, auction escalation, ATK/HP, damage/heal, and other gameplay-critical state. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 002. |
| A11Y-BS-02 | Brightness / gamma controls | Basic baseline under Standard target | Settings UX specifies brightness/gamma sliders, but render calibration approach and contrast-preserving implementation are not defined or evidenced. | later sprint / blocked dependency | Define render calibration approach, implement controls, and re-verify UI contrast under adjustment range. | No | N/A | Blocks closure. | A11Y-DEP-05 and a future accessibility-settings video calibration story. |
| A11Y-BS-03 | Screen flash warning | Basic baseline under Standard target | UX/GDD specs contain flashes, flicker, bursts, and result-screen photosensitivity notes; no formal Harding FPA-style audit or photosensitivity warning evidence is attached. | must implement in Sprint 6 | Audit RESOLUTION, GAME_OVER, combat hit, objective destruction, phase transition, and animation specs for flash frequency and full-screen flash exposure; add pre-launch photosensitivity warning evidence. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 003 or dedicated Sprint 6 flash-warning story. |
| A11Y-BS-04 | Pause anywhere | Basic baseline under Standard target | Settings UX defines safe/unsafe phase request behavior, but no implementation or server-safe pause behavior evidence is attached. | later sprint / blocked dependency | Implement or formally disposition safe-phase pause, unsafe-phase pause-request indicator, and solo-play pause behavior. | No | N/A | Blocks closure. | A11Y-DEP-01 and Settings / pause implementation story. |
| A11Y-BS-05 | Independent volume controls | Basic baseline under Standard target | Settings UX specifies Music, SFX, and UI bus sliders. Hand UI references a `ui_hand` channel, but no full audio bus pipeline or persistence evidence is attached. | later sprint / blocked dependency | Define audio controls pipeline, implement Music/SFX/UI buses with persistence, and verify muting does not remove visual backups. | No | N/A | Blocks closure. | A11Y-DEP-04 and future accessibility-settings audio-controls story. |
| A11Y-NA-01 | No dialogue / voiced content | N/A in source draft | `design/accessibility-requirements.md` states no voiced dialogue exists and subtitle requirements are minimal. Spot audit found no shipped voiced dialogue or spoken instruction evidence, but no producer not-applicable signoff is attached. | evidence-only required | Attach a formal narrative, UX, audio, and gameplay audit for voiced dialogue or spoken instructions. If none ships, producer must sign the not-applicable decision before the row stops blocking closure. | No | N/A | Blocks closure. | Producer not-applicable decision or future subtitle/accessibility story if voice is added. |

## Dependency Register

| Dependency ID | Dependency row | Current evidence | Required evidence | Blocks source rows | Sprint 6 gate impact | Follow-up path |
|---|---|---|---|---|---|---|
| A11Y-DEP-01 | Settings/accessibility screen | `design/ux/settings-accessibility.md` is Complete pending `/ux-review`; `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md` is Ready. | Evidence that Settings opens in supported contexts, exposes the required accessibility controls, handles unsafe phases safely, and remains keyboard-operable. | A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-BS-04. | Blocks closure for dependent rows until implemented or producer reclassifies them. | `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md`. |
| A11Y-DEP-02 | Preference persistence | Settings UX defines browser storage and runtime fallback, but no preference resource or persistence evidence exists. | Evidence for localStorage/profile persistence, storage failure fallback, and independent menu/HUD scale plus accessibility preference fields. | A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-ST-15. | Blocks closure for dependent rows until implementation and evidence exist. | `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md`. |
| A11Y-DEP-03 | Input action registry | Settings UX lists action groups, but no canonical registry is implemented or evidenced. | Registry of rebindable player-facing actions, conflict model, reserved browser shortcut list, and persistence tests. | A11Y-ST-08, A11Y-ST-10. | Blocks input remapping closure and any hold-flow remediation. | `production/epics/accessibility-settings/EPIC.md` story 004. |
| A11Y-DEP-04 | Audio controls pipeline | Hand UI has `TimerUrgencyAudio` and an intended `ui_hand` channel; Settings UX requires Music/SFX/UI buses. No full audio bus implementation or gate audit exists. | Audit of shipped audio content and evidence for Music, SFX, and UI bus controls with persistence, or producer decision if no audio pipeline ships in this gate. | A11Y-ST-19, A11Y-BS-05. | Blocks closure for visual-backup and volume-control rows. | `production/epics/accessibility-settings/EPIC.md` story 003 plus future audio-controls story. |
| A11Y-DEP-05 | Render calibration approach | Settings UX requires brightness/gamma controls and contrast preservation, but no renderer/shader/browser calibration approach is selected. | Technical decision for brightness/gamma implementation path and contrast re-verification under the adjustment range. | A11Y-BS-02. | Blocks brightness/gamma closure. | Future accessibility-settings video calibration story. |
| A11Y-DEP-06 | Tutorial/help prompt registry | Settings UX specifies Help replay/reset and prompt categories. No registry, ownership decision, or persistence evidence exists. | Prompt registry, owner decision for tutorial/help copy, dismiss/replay/reset persistence evidence, and DRAFT_INITIAL objective overlay retrieval path. | A11Y-ST-15, A11Y-ST-18. | Blocks tutorial persistence and DRAFT_INITIAL clear-objective closure. | `production/epics/accessibility-settings/EPIC.md` story 005. |

## Rows Still Blocking QA-COND-0005 Closure

QA-COND-0005 remains Open. The following source rows still block closure because
they require evidence, implementation, a dependency, or a future producer
decision:

- A11Y-ST-01, A11Y-ST-02, A11Y-ST-03.
- A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-ST-08.
- A11Y-ST-10, A11Y-ST-11, A11Y-ST-12, A11Y-ST-13, A11Y-ST-14, A11Y-ST-15.
- A11Y-ST-16, A11Y-ST-17, A11Y-ST-18, A11Y-ST-19.
- A11Y-BS-01, A11Y-BS-02, A11Y-BS-03, A11Y-BS-04, A11Y-BS-05.
- A11Y-NA-01.

Rows currently marked `must implement in Sprint 6`:

- A11Y-ST-13 - Mana pools: distinct container shapes.
- A11Y-ST-18 - DRAFT_INITIAL: clear objective.
- A11Y-BS-03 - Screen flash warning.

Rows currently marked `later sprint / blocked dependency`:

- A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-ST-08.
- A11Y-ST-15.
- A11Y-BS-02, A11Y-BS-04, A11Y-BS-05.

## Rows No Longer Blocking QA-COND-0005 Closure

Only one source row no longer blocks as an individual sub-gap:

- A11Y-ST-09 - PLACEMENT timer extension. It is implemented and evidenced by
  GSS-008.

This does not close QA-COND-0005 as a whole.

## Producer Decisions Needed Before Closure

No producer signoff is recorded in this register. If the project wants to avoid
implementing or evidencing a row before the Production -> Polish gate, the
producer must provide a dated decision with row ID, decision text, reason the
risk is acceptable for the gate, and follow-up owner or explicit
no-follow-up-needed statement.

Exact decisions currently available but not signed:

| Row ID | Decision needed if not implemented/evidenced before closure |
|---|---|
| A11Y-ST-04 and A11Y-ST-05 | Either implement full colorblind palette/toggles and evidence them, or producer reclassifies full colorblind modes out of the Production -> Polish gate with a follow-up path. |
| A11Y-ST-06 | Either implement and evidence 75%-150% UI scaling, or producer reclassifies UI scaling out of the Production -> Polish gate with a follow-up path. |
| A11Y-ST-07 | Either implement and evidence reduced-motion mode, or producer reclassifies reduced motion out of the Production -> Polish gate with a follow-up path. |
| A11Y-ST-08 | Either implement and evidence full input remapping, or producer reclassifies input remapping out of the Production -> Polish gate with input-action-registry dependency acknowledged. |
| A11Y-ST-10 | If the hold-input audit finds no shipped hold flow, producer may accept risk or mark no implementation needed for this gate. If any hold flow ships, producer must either require alternatives or accept risk per flow. |
| A11Y-ST-15 | Either implement tutorial persistence and Help replay/reset, or producer reclassifies tutorial persistence out of the gate with a prompt-registry follow-up path. |
| A11Y-ST-19 | If the audio-cue audit finds any shipped audio-only critical cue, producer must require visual backup implementation or accept the exposure as risk. |
| A11Y-BS-02 | Either implement brightness/gamma controls with contrast re-verification, or producer reclassifies render calibration out of the gate with a follow-up path. |
| A11Y-BS-03 | Either add photosensitivity warning and flash audit evidence, or producer accepts the residual flash-warning risk after audit. |
| A11Y-BS-04 | Either implement safe-phase pause and unsafe-phase pause request behavior, or producer reclassifies pause-anywhere behavior out of the gate with multiplayer constraints recorded. |
| A11Y-BS-05 | Either implement independent Music/SFX/UI volume controls or producer accepts risk/reclassifies the audio-controls dependency if no full audio pipeline ships in this gate. |
| A11Y-NA-01 | Producer must sign the not-applicable decision if the formal audit confirms no voiced dialogue or spoken instructions ship. |

## Closure Rule

QA-COND-0005 can close only after all source rows are implemented and evidenced,
accepted as risk with producer signoff, reclassified out of the Production ->
Polish gate with producer signoff and follow-up path, or otherwise formally
dispositioned without an unverified Standard-tier blocker.

Current closure result:

- QA-COND-0005 remains Open.
- A11Y-ST-09 is the only closed sub-gap.
- No accepted-risk or reclassification signoff is present.
- Rows marked `must implement in Sprint 6` or `later sprint / blocked
  dependency` prevent closure.
