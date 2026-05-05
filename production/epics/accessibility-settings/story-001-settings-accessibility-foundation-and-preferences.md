# Story 001: Settings / Accessibility Foundation and Preferences

> **Epic**: Accessibility Settings
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**Primary Design Sources**:

- `design/accessibility-requirements.md`: Standard-tier rows for colorblind mode, UI scaling, motion reduction, full input remapping, PLACEMENT timer extension, tutorial persistence, brightness/gamma, independent volume controls, and final browser evidence.
- `design/ux/settings-accessibility.md`: Settings / Accessibility shell, category navigation, Accessibility category controls, keyboard/focus order, preference persistence, safe/unsafe phase entry rules, and local storage warning behavior.
- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`: QA-COND-0005 remains Open after GSS-008; Settings / Accessibility foundation is the first recommended follow-up slice.

**GDD Trace**:

- `design/gdd/game-session-system.md` Rule 14 and GSS-42/GSS-43/GSS-44 are already implemented by GSS-008. This story surfaces those multiplayer-safe timer values in UI without reimplementing GSS authority.
- `design/gdd/network-protocol.md` NP-59/NP-60 are already implemented by GSS-008. This story may send `C2SSetPlacementTimerMultiplier` before `SessionReady` and display the neutral `S2CSessionSettingsUpdated` effective value.
- `design/gdd/hand-ui.md` PLACEMENT timer contract requires client timer display to use server-provided phase/snapshot duration. This story must not introduce a client-local active timer multiplier.

**TR IDs**: N/A. No registered `TR-AS-*` requirement exists yet. This story traces directly to the accessibility requirements draft, the Settings / Accessibility UX spec, and the GSS/network/Hand UI GDD timer contracts above.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
- [ADR-023: Placement Timer Accessibility Authority](../../../docs/architecture/adr-023-placement-timer-accessibility-authority.md)

**ADR Decision Summary**: Settings is client presentation UI and local preference state only. Multiplayer PLACEMENT timer authority remains in GSS/RSM; Settings can request the allowed lobby/session multiplier values and display the neutral effective session value, but active phase duration is always driven by server-provided timer data.

**Engine**: Bevy 0.18 + Lightyear 0.26 + WASM browser storage | **Risk**: HIGH

**Engine Notes**:

- Use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.
- UI must use Bevy 0.18 Required Components API: `Node`, `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`, `ChildOf`, and `ImageNode` where needed. Do not use `NodeBundle`, `TextBundle`, `UiImage::new()`, `Parent`, or `commands.entity(e).set_parent(...)`.
- Any `PickingBehavior` insertion must be behind `#[cfg(feature = "ui_picking")]`.
- The wasm persistence path may use `window.localStorage` through the existing wasm dependency family. If the implementation needs `web-sys` `Storage`, add that feature to `client/Cargo.toml` as part of this story instead of adding a new crate.
- Native/debug builds must compile without browser storage by using the same preference resource with an in-memory storage adapter.

**Control Manifest Rules (2026-05-05)**:

- Required: UI always uses bevy_ui for panels, HUD, hand fan, shop panels, and auction bid box.
- Required: `PresentationSet` order is `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: `S2CPhaseChanged` is drained only by the shared `phase_sink_system`; Settings reads `CurrentClientPhase` or existing phase/session view resources.
- Required: HUD and Hand UI display PLACEMENT timer duration from server phase/snapshot data and must not locally multiply countdown duration.
- Required: Display PLACEMENT timer multiplier as a neutral room/session setting.
- Forbidden: Never calculate active PLACEMENT countdown from local Settings in presentation code.
- Forbidden: Never expose `0.5x` as a multiplayer Standard-tier PLACEMENT timer value.
- Forbidden: Never attribute the effective timer multiplier to a player.
- Guardrail: Presentation steady-state stays below 1 ms per frame; phase-boundary UI toggles stay below a 3 ms spike.

---

## Scope

### In Scope

- Create a client Settings / Accessibility UI module and plugin surface, likely under `client/src/ui/settings/`.
- Add the Settings plugin to the client presentation composition in a way that preserves the ADR-021 presentation ordering contract.
- Create an `AccessibilityPreferences` or equivalent client resource with defaults for:
  - `colorblind_mode`: Off, Protanopia, Deuteranopia, Tritanopia.
  - `reduced_motion`: false.
  - `placement_timer_multiplier_request`: 1x default, with 1x, 1.5x, 2x, 3x allowed.
  - `menu_ui_scale_percent`: default 100, range 75 through 150.
  - `hud_ui_scale_percent`: default 100, range 75 through 150.
- Create a storage abstraction for those preferences:
  - Browser/WASM path persists to localStorage when available.
  - Storage-unavailable path keeps runtime values active and surfaces a clear saved-failed warning in Settings.
  - Native/debug path compiles with an in-memory adapter.
- Build the Settings / Accessibility shell with Back/Close, category navigation, content pane, status footer, and the Accessibility category controls listed above.
- Implement deterministic keyboard/focus baseline: Back/Close, category navigation, current category content, footer actions.
- Implement the PLACEMENT timer selector UI against the existing GSS-008 protocol surface:
  - Shows exactly 1x, 1.5x, 2x, and 3x.
  - Sends `C2SSetPlacementTimerMultiplier` only when the current context is LOBBY before `SessionReady`.
  - Shows active-session values as read-only after `SessionReady`; changes are stored as next-session preferences only.
  - Displays neutral effective room/session value from `SessionSettingsView` without requester attribution.
- Add test registrations in `client/Cargo.toml` for the exact test targets listed in this story.

### Out of Scope

- Do not close QA-COND-0005 from this story alone.
- Do not implement full colorblind palette application or gameplay color-only backup fixes.
- Do not implement reduced-motion consumers in HUD, Hand UI, Shop/Auction UI, Board Rendering, or Card Animations.
- Do not implement full keyboard/mouse remapping, conflict blocking, or reserved browser shortcut rejection beyond foundation placeholders needed by the shell.
- Do not implement Help/tutorial prompt registry, replay, reset, or persistence beyond reserving a future preferences surface.
- Do not implement brightness/gamma rendering calibration or audio bus routing.
- Do not capture final browser text-size, contrast, UI-scale, or closure evidence.
- Do not modify sprint status, session-state files, the project asset directory, or `AGENTS.md`.

---

## Acceptance Criteria

- [ ] `client/src/ui/settings/` exists and exposes a Settings / Accessibility plugin or module that can be registered from the client presentation layer without reordering existing ADR-021 sub-plugins.
- [ ] `AccessibilityPreferences` or equivalent resource exists with defaults: colorblind Off, reduced motion false, placement timer request 1x, menu UI scale 100 percent, HUD UI scale 100 percent.
- [ ] Preference validation clamps or rejects invalid scale values outside 75 through 150 and rejects any multiplayer PLACEMENT timer value outside 1x, 1.5x, 2x, and 3x.
- [ ] Browser/WASM persistence writes and reads the Story 001 preference fields through a single storage abstraction; when storage is unavailable or write fails, runtime values still apply and the Settings status footer reports the save warning.
- [ ] Native/debug builds compile without browser APIs by using the same preference resource with an in-memory storage adapter.
- [ ] Settings opens from a title/lobby safe-context entry function or message and closes back to its source without mutating room/session state.
- [ ] During PLACEMENT, DRAFT_AUCTION, and RESOLUTION, the Story 001 shell does not open the full Settings panel directly; it records or displays a pause/settings request state for the next safe boundary.
- [ ] The Settings shell includes Back/Close, category navigation, content pane, and status footer entities with stable components or markers that tests can query.
- [ ] Accessibility category exposes colorblind mode selector, reduced motion toggle, PLACEMENT timer selector, menu UI scale control, and HUD UI scale control.
- [ ] The PLACEMENT timer selector shows exactly 1x, 1.5x, 2x, and 3x. It does not show 0.5x, custom values, requester names, player IDs, or player-specific accessibility labels.
- [ ] Before `SessionReady` in LOBBY, changing the timer selector to 1.5x, 2x, or 3x writes one `C2SSetPlacementTimerMultiplier` intent for the selected value and updates the stored local request.
- [ ] After `SessionReady`, changing the timer preference stores the next-session preference only and does not write a C2S timer request or alter active `SessionSettingsView`.
- [ ] The visible effective timer value is read from the existing neutral session settings view or snapshot-derived state and never computed by locally multiplying the active PLACEMENT countdown.
- [ ] Keyboard focus order is deterministic: Back/Close, category navigation, current category content, footer actions. Hidden controls are absent from the focus order.
- [ ] Every visible interactive Settings control can be reached by keyboard and activated by Enter or Space where applicable; Esc closes the panel or cancels the active capture state if one exists later.
- [ ] Focus indicators use a high-contrast visible focus marker or component state that can be verified in tests.
- [ ] Menu UI scale and HUD UI scale are stored independently in the preference resource. Story 001 does not need every consumer to apply those values, but the Settings shell must apply menu scale to its own panel or expose a tested application hook for menu-scale consumers.
- [ ] Colorblind mode and reduced-motion preferences update the shared preference resource immediately. Palette application and animation-consumer behavior remain future stories.
- [ ] `cargo test -p client --test accessibility_settings_preferences_test` passes.
- [ ] `cargo test -p client --test accessibility_settings_shell_test` passes.
- [ ] `cargo test -p client --test accessibility_settings_timer_selector_test` passes.
- [ ] `cargo test -p client --test presentation_plugin_scaffold_test` remains green.
- [ ] `cargo check -p client` passes.
- [ ] `git diff --check` passes.

---

## Implementation Notes

**Likely files touched**:

- `client/src/ui/mod.rs`
- `client/src/ui/settings/mod.rs`
- `client/src/presentation/mod.rs`
- `client/src/state/mod.rs` or a new client preference module if that better fits the codebase
- `client/Cargo.toml` for exact test registrations and any required `web-sys` feature addition
- `tests/unit/accessibility_settings/preferences_test.rs`
- `tests/integration/accessibility_settings/settings_shell_test.rs`
- `tests/integration/accessibility_settings/timer_selector_test.rs`
- `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`

**Preference defaults**:

- Colorblind mode: Off.
- Reduced motion: false.
- Placement timer multiplier request: `PlacementTimerMultiplier::X1`.
- Menu UI scale percent: 100.
- HUD UI scale percent: 100.

**Storage keys**:

Use a single namespaced root such as `lanes_and_lies.accessibility_preferences.v1`. Store a versioned serialized payload rather than separate ad hoc keys so future migrations are explicit.

**Timer authority**:

The timer selector is a request UI. It is not the active timer source. Active PLACEMENT duration remains server-owned through `S2CPhaseChanged.timer_duration_ms` and reconnect snapshot data. The UI may show local request and neutral effective value side by side, but it must not say which player requested the effective value.

**Settings context model**:

Represent source context explicitly, such as Title, Lobby, SafeInGame, UnsafeInGame, GameOver, or Help. Story 001 only needs enough context to prove safe contexts open the panel, unsafe contexts queue a request, and Back/Esc returns to the source.

**Performance Budget**:

No gameplay-loop performance impact expected. Settings is a modal UI surface opened by player action, and steady-state hidden Settings systems should do no per-frame tree rebuild. Any visible Settings update should remain within the ADR-021 presentation guardrail: below 1 ms steady-state UI work and below a 3 ms phase-boundary spike.

---

## QA Test Cases

- **Preference defaults and validation**
  - Given: A fresh client app with no stored accessibility preferences
  - When: `AccessibilityPreferences` is initialized
  - Then: colorblind mode is Off, reduced motion is false, placement timer request is 1x, menu UI scale is 100, and HUD UI scale is 100
  - Edge cases: Scale values below 75 or above 150 are clamped or rejected consistently; timer values outside the multiplayer Standard-tier set are rejected

- **Preference persistence and storage failure**
  - Given: Browser storage adapter is available
  - When: The player changes colorblind mode, reduced motion, timer request, menu scale, and HUD scale
  - Then: A versioned preference payload is written under the namespaced storage key and reloads into the same resource values
  - Edge cases: If storage write fails, runtime values remain active and the Settings status footer reports the save warning

- **Settings shell safe and unsafe entry**
  - Given: Current context is Title, Lobby, or a safe in-game phase
  - When: Settings is requested
  - Then: The Settings panel becomes visible and Back/Esc returns to the exact source context without mutating room/session state
  - Edge cases: During PLACEMENT, DRAFT_AUCTION, or RESOLUTION, requesting Settings does not show the full panel and instead records or displays a pause/settings request state

- **Timer selector authority boundary**
  - Given: LOBBY before `SessionReady`
  - When: The player selects 3x
  - Then: Exactly one `C2SSetPlacementTimerMultiplier { multiplier: X3 }` intent is written and the local request preference becomes 3x
  - Edge cases: After `SessionReady`, selecting 3x updates next-session preference only and writes no C2S request

- **Timer selector values and neutral display**
  - Given: The Accessibility category renders
  - When: Timer selector options are queried
  - Then: The only options are 1x, 1.5x, 2x, and 3x; no 0.5x or custom option exists
  - Edge cases: Effective value display reads from neutral session state and contains no requester identity

- **Keyboard focus order**
  - Given: The Settings panel is visible
  - When: Keyboard navigation traverses the panel
  - Then: Focus order is Back/Close, category navigation, current category content, footer actions; hidden controls are not focusable
  - Edge cases: Enter or Space activates visible controls; Esc closes the panel unless a later capture state has priority

---

## Test Evidence

**Story Type**: UI

**Required automated test targets**:

- `tests/unit/accessibility_settings/preferences_test.rs`
  - Registered as `accessibility_settings_preferences_test`
  - Command: `cargo test -p client --test accessibility_settings_preferences_test`
- `tests/integration/accessibility_settings/settings_shell_test.rs`
  - Registered as `accessibility_settings_shell_test`
  - Command: `cargo test -p client --test accessibility_settings_shell_test`
- `tests/integration/accessibility_settings/timer_selector_test.rs`
  - Registered as `accessibility_settings_timer_selector_test`
  - Command: `cargo test -p client --test accessibility_settings_timer_selector_test`
- Regression: `cargo test -p client --test presentation_plugin_scaffold_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required evidence document**:

- `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`

The evidence document must list the exact commands above, their pass/fail result,
the implemented preference fields, the storage behavior, and the QA-COND-0005
impact statement below.

**QA-COND-0005 impact statement required in evidence**:

Story 001 reduces QA-COND-0005 risk by creating the Settings / Accessibility
preference foundation and timer selector UI. It does not close QA-COND-0005.
The bug remains Open until the remaining Standard-tier rows are implemented and
browser/WASM-evidenced, reclassified, or accepted as risk.

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `production/epics/game-session-system/story-008-placement-timer-multiplier-authority.md` (Complete) for implemented GSS-008 timer authority and protocol values.
- Depends on: `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` (Complete) for `PresentationPlugin`, `PresentationSet`, and phase sink.
- Depends on: `production/epics/presentation-layer/story-002-shared-economy-view.md` (Complete) for current presentation shared-resource patterns.
- Depends on: ADR-002, ADR-021, and ADR-023 Accepted.
- Unlocks: colorblind modes and color-only backups; reduced motion, flash audit, and gameplay-critical visual backups; input remapping and hold audit; help/tutorial persistence; browser accessibility evidence and QA-COND-0005 closure story.
