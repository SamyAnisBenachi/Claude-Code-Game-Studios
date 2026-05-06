# Standard-Tier Accessibility Disposition Register - Sprint 6

| Field | Value |
|---|---|
| Evidence ID | S6-04 Standard-tier accessibility disposition |
| Date | 2026-05-06 |
| QA condition | `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` |
| QA-COND-0005 status | Accepted Risk |
| Source draft | `design/accessibility-requirements.md` |
| Related UX draft | `design/ux/settings-accessibility.md` |
| GSS-008 evidence | `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md` |
| HUD-011 evidence | `production/qa/evidence/hud-011-mana-shapes-evidence.md` |
| HUD-012 evidence | `production/qa/evidence/hud-012-text-size-contrast-accessibility.md` |
| A11Y Settings 001 evidence | `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md` |
| HAND-UI-014 evidence | `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md` |
| SAU-012 evidence | `production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md` |
| A11Y-BS-03 evidence | `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md` |
| Disposition verdict | QA-COND-0005 is accepted risk for Sprint 6 friend-game scope. A11Y-ST-09, A11Y-ST-12, A11Y-ST-13, A11Y-ST-14, A11Y-ST-18, and A11Y-BS-03 are implemented and evidenced. HUD-owned portions of A11Y-ST-01 and A11Y-ST-03 are implemented and evidenced by HUD-012. Remaining Standard-tier rows are not verified; producer waived them for friend-game scope only with reason: "Standard-tier accessibility waived by producer. No public release, no obligation." |

## Scope

This register records the Sprint 6 S6-04 disposition of every Standard-tier
accessibility source row and every Sprint 6 dependency row. It does not edit the
accessibility requirements draft, does not implement code, and does not claim
evidence that has not been captured. It records the producer accepted-risk
waiver that removes QA-COND-0005 as an active Sprint 6 blocker for friend-game
scope only.

Allowed final dispositions used by this register:

- implemented + evidence attached
- evidence-only required
- must implement in Sprint 6
- later sprint / blocked dependency
- accepted risk with producer signoff

## Producer Accepted-Risk Decision

On 2026-05-06, the producer waived the remaining Standard-tier accessibility
scope for the Sprint 6 Production-to-Polish gate under friend-game scope:

> Friend-game scope - Standard-tier accessibility waived by producer. No public
> release, no obligation.

This is not verified Standard-tier accessibility completion. It is a scoped
accepted-risk decision for a non-public friend-game build. Any public, external,
commercial, or broader release candidate must revisit the remaining unresolved
rows and either implement, evidence, reclassify, or accept risk under that later
release scope.

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

## HUD-011 Evidence Linkage

HUD-011 implemented and verified the A11Y-ST-13 current/reserve mana shape
distinction:

- evidence:
  `production/qa/evidence/hud-011-mana-shapes-evidence.md`
- story:
  `production/epics/hud/story-011-current-reserve-mana-shapes.md`
- automated verification:
  `cargo test -p client --test hud_mana_shape_distinction_test` and
  `cargo test -p client --test hud_gold_mana_display_test`

HUD-011 evidence records current mana as a horizontal bar, reserve mana as a
diamond container, non-color component/layout assertions, and browser/WASM
color plus grayscale captures at `1366x768` and `1920x1080`.

## HUD-012 Evidence Linkage

HUD-012 implemented and verified HUD-owned text-size and contrast evidence for
A11Y-ST-01 and A11Y-ST-03:

- evidence:
  `production/qa/evidence/hud-012-text-size-contrast-accessibility.md`
- story:
  `production/epics/hud/story-012-text-size-and-contrast-accessibility.md`
- automated verification:
  `cargo test -p client --test hud_text_size_contrast_accessibility_test`,
  `cargo test -p client --test hud_gold_mana_display_test`,
  `cargo test -p client --test hud_phase_label_round_counter_test`, and
  `cargo test -p client --test hud_economy_auction_inline_gold_test`

HUD-012 evidence records browser/WASM captures at `1366x768` and `1920x1080`,
40 px HUD gold primary text, 20 px-or-better HUD resource/phase/round text and
cold-start placeholders, and HUD-owned text/background contrast ratios above
`4.5:1`. It does not cover card text, the actual auction price counter,
Settings, Shop/Auction, Hand UI, board, or result-screen contrast.

## A11Y Settings 001 Evidence Linkage

A11Y Settings 001 implemented and verified the Settings / Accessibility
foundation and preference storage dependency:

- evidence:
  `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`
- story:
  `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md`
- automated verification:
  `cargo test -p client --test accessibility_settings_preferences_test`,
  `cargo test -p client --test accessibility_settings_shell_test`,
  `cargo test -p client --test accessibility_settings_timer_selector_test`, and
  `cargo test -p client --test presentation_plugin_scaffold_test`

Story 001 evidence covers the Settings shell, stable query markers, keyboard
focus baseline, safe/unsafe phase entry behavior, versioned preference storage,
storage-write failure fallback, independent menu/HUD scale preference fields,
colorblind/reduced-motion preference fields, and the neutral multiplayer-safe
PLACEMENT timer selector. It does not implement colorblind palette application,
reduced-motion consumers, full input remapping, tutorial persistence,
brightness/gamma controls, audio bus controls, or final browser/WASM closure
evidence for QA-COND-0005.

## HAND-UI-014 Evidence Linkage

HAND-UI-014 implemented and verified the A11Y-ST-14 PLACEMENT staged disclosure
sub-gap:

- evidence:
  `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`
- story:
  `production/epics/hand-ui/story-014-placement-staged-disclosure-accessibility.md`
- automated verification:
  `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
  and the requested Hand UI regression targets

HAND-UI-014 evidence records the Browser/WASM sequence for PLACEMENT entry, card
selection, lane/cell target guidance, valid target highlight, valid stage,
reserve/current split adjustment, invalid submit, correction, and successful
submit. It also records that later controls are not shown before their
disclosure step and that QA-COND-0005 remains Open.

## SAU-012 Evidence Linkage

SAU-012 implemented and verified the A11Y-ST-18 DRAFT_INITIAL clear objective
overlay sub-gap.

- evidence:
  `production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md`
- Browser/WASM captures:
  `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/`
- story:
  `production/epics/shop-auction-ui/story-012-draft-initial-clear-objective-overlay.md`
- automated verification:
  `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test --jobs 1`
  and the requested Shop/Auction UI regression targets

SAU-012 evidence records overlay presence on DRAFT_INITIAL activation, exact
copy, visible focused dismiss control, Esc dismissal, retrieval affordance,
retrieved overlay, non-occlusion of the grid, timer, Ready control, HUD gold,
and hand surfaces, plus no overlay-originated C2S sends. It also records that
QA-COND-0005 remains Open.

## A11Y-BS-03 Evidence Linkage

A11Y-BS-03 implemented and verified the photosensitivity warning plus
flash-frequency audit evidence:

- evidence:
  `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`
- story:
  `production/epics/accessibility-settings/story-003-photosensitivity-warning-and-flash-audit.md`
- automated verification:
  `cargo test -p client --test accessibility_settings_photosensitivity_warning_test --jobs 1`
  and `cargo test -p client --test presentation_plugin_scaffold_test --jobs 1`

The evidence records the warning copy in
`client/src/ui/photosensitivity_warning.rs`, shows the boot warning appears
before gameplay exposure, and covers message plus interaction acknowledgement.
The audit documents objective destruction full-screen Prism White as exceeding
or unable to prove the local max 3 flashes/sec rule. That residual risk remains
visible as a follow-up for Board Rendering / Card Animations and producer
disposition if warning-only treatment is insufficient before release.
QA-COND-0005 remains Open.

## Source Row Disposition Register

| Row ID | Source row | Tier | Current evidence | Final disposition | Required audit or test | Producer signoff required | Signoff evidence | QA-COND-0005 impact | Follow-up path |
|---|---|---|---|---|---|---|---|---|---|
| A11Y-ST-01 | Minimum text size - HUD | Standard | HUD-012 evidence verifies HUD-owned gold, reserved-gold suffix, mana, reserve, phase, round, and cold-start placeholder text floors in browser/WASM captures at `1366x768` and `1920x1080`. The actual auction price counter remains outside HUD-012 and still lacks required browser evidence. | evidence-only required | Retain HUD-012 evidence for HUD-owned text floors; capture browser/WASM evidence for the actual auction price counter's required 40 px floor or formally disposition that separate owner. | No | N/A | Partially closes the HUD-owned text-size sub-gap; still blocks closure until the actual auction price counter and any remaining row exposure are evidenced or dispositioned. | Shop/Auction/accessibility closure evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-02 | Minimum text size - card text | Standard | No browser/WASM measurement evidence for card cost, ATK, HP, or keyword text floors. | evidence-only required | Browser/WASM card readability capture measuring stat badges at 18px minimum and keyword text at 14px minimum. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 006 or equivalent browser evidence pass. |
| A11Y-ST-03 | Text contrast - UI on backgrounds | Standard | HUD-012 evidence verifies HUD-owned text/background pairs at `4.5:1` or better, including reserved-gold suffix alpha compositing and DRAFT_AUCTION / RESOLUTION HUD states. Cards, Settings, Shop/Auction, Hand UI, board, result screens, and the actual auction price counter remain outside HUD-012. | evidence-only required | Retain HUD-012 evidence for HUD-owned contrast; complete browser/WASM contrast audit for cards, Settings, Shop/Auction, Hand UI, board, result screens, and the actual auction price counter's 7:1 exception. | No | N/A | Partially closes the HUD-owned contrast sub-gap; still blocks closure until remaining UI surfaces and the auction price exception are evidenced or dispositioned. | Accessibility closure evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-04 | Colorblind mode - Protanopia / Deuteranopia | Standard | A11Y Settings 001 implements and persists the colorblind selector field, and art/design docs define shape/icon backups. Palette application and browser evidence are not attached. | later sprint / blocked dependency | Implement and verify Protanopia and Deuteranopia palette application plus shape/icon backup browser evidence. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 002. |
| A11Y-ST-05 | Colorblind mode - Tritanopia | Standard | A11Y Settings 001 implements and persists the Tritanopia selector field. Tritanopia palette application and auction escalation readability evidence are not attached. | later sprint / blocked dependency | Implement and verify Tritanopia palette application, auction escalation readability, and shape/icon backups. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 002. |
| A11Y-ST-06 | UI scaling | Standard | A11Y Settings 001 implements independent menu/HUD scale preference fields, persistence, validation, and a tested Settings panel menu-scale hook. Full menu/HUD consumer application and browser layout evidence are not attached. | later sprint / blocked dependency | Verify menu and HUD scale consumers and capture browser layout checks at 75%, 100%, and 150%. | No | N/A | Blocks closure. | Future UI-scale consumer/evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-07 | Motion / animation reduction mode | Standard | A11Y Settings 001 implements and persists the reduced-motion preference field. No motion consumer implementation or evidence is attached. | later sprint / blocked dependency | Verify consumers for auction entrance, bid pulse, timer pulse, phase transitions, class reveal, panel motion, frame flicker, and nonessential scale motion. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 003. |
| A11Y-ST-08 | Full input remapping | Standard | Settings UX defines input remapping requirements, but no canonical input action registry, conflict blocking, reserved shortcut rejection, or persistence evidence is attached. | later sprint / blocked dependency | Define canonical action registry, implement keyboard/mouse remapping, reject conflicts and browser shortcuts, and verify persistence. | No | N/A | Blocks closure. | A11Y-DEP-03, `production/epics/accessibility-settings/EPIC.md` story 004. |
| A11Y-ST-09 | PLACEMENT timer extension | Standard | GSS-008 evidence verifies server-authoritative values 1x, 1.5x, 2x, 3x; neutral highest-request-wins authority; freeze at `SessionReady`; RSM effective duration; reconnect snapshot; and Hand UI server-duration use. | implemented + evidence attached | Retain GSS-008 regression commands and `git diff --check` evidence from `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`. | No | N/A | Closes sub-gap. | No follow-up for this row unless timer authority regresses. QA-COND-0005 as a whole remains Open. |
| A11Y-ST-10 | Hold-to-press alternatives | Standard | Spot audit found `design/ux/hand-ui.md` states no hold-to-confirm interaction exists in Hand UI. Full UX and implementation audit is not attached. | evidence-only required | Audit UX specs and implementation for hold-to-confirm, long-press, press-and-hold, timer-gated button hold, and pointer-held flows. If any shipped hold flow exists, implement an alternative or obtain producer accepted-risk signoff per flow. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 004. |
| A11Y-ST-11 | DRAFT_SHOP ready signal - retractable | Standard | RSM logic evidence covers ready retraction in `server/tests/rsm_timers_test.rs`; no browser/UI evidence verifies visible retractable control behavior. | evidence-only required | Browser/WASM manual or automated evidence showing ready can be retracted before all-ready fires and that the control is visibly reversible. | No | N/A | Blocks closure. | Existing Shop/Auction evidence pass, SAU-009 equivalent, or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-12 | Auction bid buttons - immediate preset commitments | Standard | SAU-011 evidence verifies preset total-commitment labels, 108x44 CSS-px targets, 2px focus-ring state, affordability gating, disabled/unaffordable focus skip, in-flight disable, one-send semantics, `BIDDING...` feedback, and local `YOU ARE LEADING` replacement in automated ECS tests plus Browser/WASM captures. | implemented + evidence attached | Retain SAU-011 evidence and regression commands from `production/qa/evidence/shop-auction-ui-auction-bid-target-focus-2026-05-05.md`. | No | N/A | Closes sub-gap. QA-COND-0005 as a whole remains Open. | No follow-up for this row unless auction bid target/focus accessibility regresses. |
| A11Y-ST-13 | Mana pools: distinct container shapes | Standard | HUD-011 evidence verifies current mana as a horizontal bar, reserve mana as a diamond, non-color component/layout assertions, and browser/WASM color plus grayscale captures at `1366x768` and `1920x1080`. | implemented + evidence attached | Retain HUD-011 evidence and regression commands from `production/qa/evidence/hud-011-mana-shapes-evidence.md`. | No | N/A | Closes sub-gap. QA-COND-0005 as a whole remains Open. | No follow-up for this row unless HUD mana shape accessibility regresses. |
| A11Y-ST-14 | PLACEMENT staged disclosure | Standard | HAND-UI-014 evidence verifies PLACEMENT entry, card selection, lane/cell target guidance, valid target highlight, valid stage, reserve/current split adjustment, invalid submit, correction, and successful submit in Browser/WASM capture evidence. | implemented + evidence attached | Retain HAND-UI-014 evidence and regression commands from `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`. | No | N/A | Closes sub-gap. QA-COND-0005 as a whole remains Open. | No follow-up for this row unless PLACEMENT staged disclosure regresses. |
| A11Y-ST-15 | Tutorial persistence | Standard | Settings UX defines Help replay/reset behavior, but no tutorial/help prompt registry, replay, reset, or persistence implementation exists. | later sprint / blocked dependency | Implement prompt registry, dismissed-prompt persistence, Help replay, reset-all, and per-prompt reset evidence. | No | N/A | Blocks closure. | A11Y-DEP-06, `production/epics/accessibility-settings/EPIC.md` story 005. |
| A11Y-ST-16 | Phase label always visible | Standard | HUD logic evidence verifies phase label text updates; no browser/WASM evidence verifies visibility, non-occlusion, and non-animation-only phase communication. | evidence-only required | Browser/WASM capture across phases proving phase label remains visible, readable, non-occluded, and not communicated by animation alone. | No | N/A | Blocks closure. | HUD evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-17 | Gold counter always visible | Standard | HUD and economy logic evidence covers gold display behavior; no browser/WASM occlusion or full-opacity visibility evidence is attached. | evidence-only required | Browser/WASM capture across DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT, RESOLUTION, and GAME_OVER proving gold counter visibility and opacity. | No | N/A | Blocks closure. | HUD/Shop-Auction evidence pass or `production/epics/accessibility-settings/EPIC.md` story 006. |
| A11Y-ST-18 | DRAFT_INITIAL: clear objective | Standard | SAU-012 evidence verifies the exact DRAFT_INITIAL objective copy, activation only after phase plus offering data, dismiss button focus, Esc dismissal, non-actionable outside dismissal without C2S sends, in-phase retrieval, hidden retrieval on phase exit, and Browser/WASM non-occlusion/readability captures. | implemented + evidence attached | Retain SAU-012 evidence, capture summary, and regression commands from `production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md`. | No | N/A | Closes sub-gap. QA-COND-0005 as a whole remains Open. | No follow-up for this row unless DRAFT_INITIAL objective overlay accessibility regresses. |
| A11Y-ST-19 | Visual indicators for audio cues | Standard | Source docs identify timer, auction, RESOLUTION, and objective audio cues. Hand UI has visible timer number and `TimerUrgencyAudio`, but no complete gameplay-critical audio-cue audit is attached. | evidence-only required | Audit all gameplay-critical audio cues and map each cue to visible backup, non-shipping audio status, or implementation dependency. Any shipped audio-only critical cue must be remediated or producer-signed as risk. | No | N/A | Blocks closure. | A11Y-DEP-04, `production/epics/accessibility-settings/EPIC.md` story 003. |
| A11Y-BS-01 | Color-as-only-indicator audit | Basic baseline under Standard target | Art bible and design docs cover some shape/text backups; objective dots and damage/heal backup remain unverified. | evidence-only required | Color-as-only audit for player side, class identity, objective status, auction escalation, ATK/HP, damage/heal, and other gameplay-critical state. | No | N/A | Blocks closure. | `production/epics/accessibility-settings/EPIC.md` story 002. |
| A11Y-BS-02 | Brightness / gamma controls | Basic baseline under Standard target | Settings UX specifies brightness/gamma sliders, but render calibration approach and contrast-preserving implementation are not defined or evidenced. | later sprint / blocked dependency | Define render calibration approach, implement controls, and re-verify UI contrast under adjustment range. | No | N/A | Blocks closure. | A11Y-DEP-05 and a future accessibility-settings video calibration story. |
| A11Y-BS-03 | Screen flash warning | Basic baseline under Standard target | Photosensitivity warning and flash-frequency audit evidence is attached at `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`. The warning is implemented in `client/src/ui/photosensitivity_warning.rs` and verified by focused client tests. The audit records objective destruction full-screen Prism White as exceeding or unable to prove the local max 3 flashes/sec rule. | implemented + evidence attached | Retain A11Y-BS-03 evidence and regression commands; do not claim strict objective-destruction flash compliance unless the residual Prism White effect is remediated or formally dispositioned later. | No | N/A | Closes the A11Y-BS-03 warning/audit sub-gap. QA-COND-0005 as a whole remains Open. | Board Rendering / Card Animations follow-up for objective-destruction Prism White if warning-only disposition is insufficient before release. |
| A11Y-BS-04 | Pause anywhere | Basic baseline under Standard target | A11Y Settings 001 implements safe Settings entry and queues unsafe-phase full-panel requests for the next safe boundary. No broader server-safe pause, unsafe-phase request indicator, or solo-play pause behavior evidence is attached. | later sprint / blocked dependency | Implement or formally disposition safe-phase pause, unsafe-phase pause-request indicator, and solo-play pause behavior. | No | N/A | Blocks closure. | Settings / pause implementation story. |
| A11Y-BS-05 | Independent volume controls | Basic baseline under Standard target | Settings UX specifies Music, SFX, and UI bus sliders. Hand UI references a `ui_hand` channel, but no full audio bus pipeline or persistence evidence is attached. | later sprint / blocked dependency | Define audio controls pipeline, implement Music/SFX/UI buses with persistence, and verify muting does not remove visual backups. | No | N/A | Blocks closure. | A11Y-DEP-04 and future accessibility-settings audio-controls story. |
| A11Y-NA-01 | No dialogue / voiced content | N/A in source draft | `design/accessibility-requirements.md` states no voiced dialogue exists and subtitle requirements are minimal. Spot audit found no shipped voiced dialogue or spoken instruction evidence, but no producer not-applicable signoff is attached. | evidence-only required | Attach a formal narrative, UX, audio, and gameplay audit for voiced dialogue or spoken instructions. If none ships, producer must sign the not-applicable decision before the row stops blocking closure. | No | N/A | Blocks closure. | Producer not-applicable decision or future subtitle/accessibility story if voice is added. |

## Dependency Register

| Dependency ID | Dependency row | Current evidence | Required evidence | Blocks source rows | Sprint 6 gate impact | Follow-up path |
|---|---|---|---|---|---|---|
| A11Y-DEP-01 | Settings/accessibility screen | A11Y Settings 001 evidence verifies the Settings shell opens in safe contexts, queues unsafe-phase full-panel requests for the next safe boundary, exposes the Story 001 accessibility controls, provides stable markers, and remains keyboard-operable by focus order plus Enter/Space/Esc tests. | No additional foundation evidence required for this dependency. Dependent source rows still need their feature-specific implementation and browser evidence. | A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-BS-04. | No longer blocks as an empty Settings foundation; dependent source rows still block QA-COND-0005 until their own requirements are evidenced or formally dispositioned. | `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`. |
| A11Y-DEP-02 | Preference persistence | A11Y Settings 001 evidence verifies versioned single-payload preference storage, native in-memory fallback, browser/localStorage path compilation, storage-write failure warning behavior, and independent menu/HUD scale plus colorblind, reduced-motion, and timer preference fields. | No additional foundation evidence required for this dependency. Dependent source rows still need persistence consumers, browser evidence, or feature-specific follow-up as applicable. | A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-ST-15. | No longer blocks as missing preference infrastructure; dependent source rows still block QA-COND-0005 until their own requirements are evidenced or formally dispositioned. | `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`. |
| A11Y-DEP-03 | Input action registry | Settings UX lists action groups, but no canonical registry is implemented or evidenced. | Registry of rebindable player-facing actions, conflict model, reserved browser shortcut list, and persistence tests. | A11Y-ST-08, A11Y-ST-10. | Blocks input remapping closure and any hold-flow remediation. | `production/epics/accessibility-settings/EPIC.md` story 004. |
| A11Y-DEP-04 | Audio controls pipeline | Hand UI has `TimerUrgencyAudio` and an intended `ui_hand` channel; Settings UX requires Music/SFX/UI buses. No full audio bus implementation or gate audit exists. | Audit of shipped audio content and evidence for Music, SFX, and UI bus controls with persistence, or producer decision if no audio pipeline ships in this gate. | A11Y-ST-19, A11Y-BS-05. | Blocks closure for visual-backup and volume-control rows. | `production/epics/accessibility-settings/EPIC.md` story 003 plus future audio-controls story. |
| A11Y-DEP-05 | Render calibration approach | Settings UX requires brightness/gamma controls and contrast preservation, but no renderer/shader/browser calibration approach is selected. | Technical decision for brightness/gamma implementation path and contrast re-verification under the adjustment range. | A11Y-BS-02. | Blocks brightness/gamma closure. | Future accessibility-settings video calibration story. |
| A11Y-DEP-06 | Tutorial/help prompt registry | Settings UX specifies Help replay/reset and prompt categories. No registry, ownership decision, or persistence evidence exists. SAU-012 supplies the DRAFT_INITIAL objective overlay retrieval path for A11Y-ST-18. | Prompt registry, owner decision for tutorial/help copy, and dismiss/replay/reset persistence evidence. | A11Y-ST-15. | Blocks tutorial persistence; no longer blocks DRAFT_INITIAL clear-objective closure. | `production/epics/accessibility-settings/EPIC.md` story 005. |

## Rows Remaining As Accepted Risk / Future Accessibility Debt

QA-COND-0005 is accepted risk for Sprint 6 friend-game scope. The following
source rows remain unresolved accessibility debt and must be revisited before
any public/external release, but they no longer block S6-06 under the producer
waiver above:

- A11Y-ST-01, A11Y-ST-02, A11Y-ST-03.
- A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-ST-08.
- A11Y-ST-10, A11Y-ST-11, A11Y-ST-15.
- A11Y-ST-16, A11Y-ST-17, A11Y-ST-19.
- A11Y-BS-01, A11Y-BS-02, A11Y-BS-04, A11Y-BS-05.
- A11Y-NA-01.

Rows currently marked `must implement in Sprint 6`:

- None.

Rows currently marked `later sprint / blocked dependency`:

- A11Y-ST-04, A11Y-ST-05, A11Y-ST-06, A11Y-ST-07, A11Y-ST-08.
- A11Y-ST-15.
- A11Y-BS-02, A11Y-BS-04, A11Y-BS-05.

## Rows No Longer Blocking QA-COND-0005 Closure

Six source rows no longer block as individual sub-gaps:

- A11Y-ST-09 - PLACEMENT timer extension. It is implemented and evidenced by
  GSS-008.
- A11Y-ST-12 - Auction bid buttons: immediate preset commitments. It is
  implemented and evidenced by SAU-011.
- A11Y-ST-13 - Mana pools: distinct container shapes. It is implemented and
  evidenced by HUD-011.
- A11Y-ST-14 - PLACEMENT staged disclosure. It is implemented and evidenced by
  HAND-UI-014.
- A11Y-ST-18 - DRAFT_INITIAL: clear objective. It is implemented and evidenced
  by SAU-012.
- A11Y-BS-03 - Screen flash warning. It is implemented and evidenced by the
  photosensitivity warning plus flash-frequency audit. Objective destruction
  Prism White remains a documented residual follow-up if warning-only
  disposition is insufficient before release.

Two dependency rows are no longer empty prerequisites:

- A11Y-DEP-01 - Settings/accessibility screen foundation. It is implemented and
  evidenced by A11Y Settings 001.
- A11Y-DEP-02 - Preference persistence foundation. It is implemented and
  evidenced by A11Y Settings 001.

This does not close QA-COND-0005 as a whole.

## Producer Decisions Recorded For Sprint 6

The producer decision above accepts the remaining Standard-tier accessibility
exposure for friend-game scope only. The row-specific decisions below remain
useful future planning notes, not Sprint 6 blockers after this accepted-risk
waiver.

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
| A11Y-BS-04 | Either implement safe-phase pause and unsafe-phase pause request behavior, or producer reclassifies pause-anywhere behavior out of the gate with multiplayer constraints recorded. |
| A11Y-BS-05 | Either implement independent Music/SFX/UI volume controls or producer accepts risk/reclassifies the audio-controls dependency if no full audio pipeline ships in this gate. |
| A11Y-NA-01 | Producer must sign the not-applicable decision if the formal audit confirms no voiced dialogue or spoken instructions ship. |

## Closure Rule

QA-COND-0005 can only be marked Verified/Closed after all source rows are
implemented and evidenced, accepted as risk with producer signoff, reclassified
out of the relevant release gate with producer signoff and follow-up path, or
otherwise formally dispositioned without an unverified Standard-tier blocker.
For Sprint 6 friend-game scope, this register records accepted risk rather than
verified accessibility completion.

Current closure result:

- QA-COND-0005 is Accepted Risk for Sprint 6 friend-game scope.
- A11Y-ST-09, A11Y-ST-12, A11Y-ST-13, A11Y-ST-14, and A11Y-ST-18
  are closed sub-gaps.
- A11Y-BS-03 is a closed warning/audit sub-gap, with objective-destruction
  Prism White retained as a residual follow-up if strict local-rule compliance
  is required before release.
- HUD-owned portions of A11Y-ST-01 and A11Y-ST-03 are evidenced by HUD-012, but
  the parent rows still block closure until remaining non-HUD / auction-price
  exposure is evidenced or dispositioned.
- Producer accepted-risk signoff is present for remaining Standard-tier
  exposure under friend-game scope only.
- Rows marked `evidence-only required`, `must implement in Sprint 6`, or
  `later sprint / blocked dependency` remain future accessibility debt and must
  be revisited before any public/external release candidate.
