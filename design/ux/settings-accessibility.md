# UX Spec: Settings / Accessibility

> **Status**: Complete - pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-05-05
> **Journey Phase(s)**: Title screen, lobby, safe in-game pause windows, game over
> **Template**: UX Spec
> **Input Methods**: Mouse + Keyboard (primary: mouse click). No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (`design/accessibility-requirements.md`)
> **Source Docs**: `design/accessibility-requirements.md`, `design/ux/main-menu.md`, `design/ux/interaction-patterns.md`, `design/ux/hud.md`, `design/ux/shop-auction-ui.md`, `design/art/art-bible.md`

---

## Purpose & Player Need

The Settings / Accessibility screen lets players make the game physically readable, controllable, and comfortable before the first timed decision and during safe match boundaries.

The player arrives wanting to:

1. Remove barriers before play: colorblind mode, reduced motion, UI scale, brightness/gamma, audio levels, and input bindings.
2. Recover dismissed help: tutorial prompts and rules references remain available after first dismissal.
3. Adjust live comfort settings without losing match context or creating a multiplayer timing exploit.

If this screen is missing or hard to use, the Standard accessibility tier is not met. The highest-risk failures are: players cannot extend the 10-second PLACEMENT timer, color-coded information stays inaccessible, input conflicts block keyboard play, or dismissed tutorials become permanently unrecoverable.

---

## Player Context on Arrival

| Arrival Context | Player State | Design Response |
|---|---|---|
| First app launch from title screen | Curious, checking setup before creating/joining a room | Open to Accessibility by default on first visit; make core comfort controls visible without scrolling |
| Lobby via Settings button or pause/settings shortcut | Waiting, low pressure, possibly configuring before opponent arrives | Full Settings panel available; return to exact lobby state on close |
| DRAFT_INITIAL or DRAFT_SHOP pause window | Actively playing but not in the 10-second placement deadline | Settings can open as a blocking overlay; local-only settings apply immediately; timer-affecting settings apply at next server-safe boundary |
| PLACEMENT, DRAFT_AUCTION, or RESOLUTION | Time-critical or non-interruptible multiplayer phase | Full panel does not open immediately. Esc shows "Pause requested - settings available next phase boundary." |
| GAME_OVER | Reviewing outcome, safe to change preferences for next match | Full panel available; Help tab includes match-relevant tutorial entries |

No `design/player-journey.md` exists yet, so player journey assumptions come from the existing UX specs and GDD phase descriptions.

---

## Navigation Position

```text
TITLE SCREEN
  -> Settings button
      -> SETTINGS / ACCESSIBILITY
          -> Back to Title

LOBBY
  -> Settings button or Esc pause overlay
      -> SETTINGS / ACCESSIBILITY
          -> Back to Lobby

IN-GAME HUD
  -> Esc / pause command in safe phases
      -> Pause Overlay
          -> Settings
              -> Back to Pause Overlay / HUD

IN-GAME HUD during PLACEMENT, DRAFT_AUCTION, RESOLUTION
  -> Esc / pause command
      -> Pause Requested Indicator
          -> Opens Settings at next safe boundary

HELP / HOW TO PLAY
  -> Settings link or Reset Tutorials action
      -> SETTINGS / ACCESSIBILITY, Help tab
          -> Back to Help source
```

The screen is a top-level destination from the title screen and a modal destination from live game contexts. It never destroys session state by itself.

---

## Entry & Exit Points

### Entry

| Entry Source | Trigger | Player Carries This Context |
|---|---|---|
| Title screen | Click Settings icon/button from `design/ux/main-menu.md` | Return destination = title; no active session |
| Lobby | Click Settings or press Esc, then Settings | Room code, own class state, opponent slot state, lobby timer |
| DRAFT_INITIAL / DRAFT_SHOP | Press Esc or pause command, then Settings | Current phase, card/shop state, remaining timer, ready/retract state |
| PLACEMENT / DRAFT_AUCTION / RESOLUTION | Press Esc or pause command | Request flag only; full Settings opens at next safe phase boundary |
| GAME_OVER | Click Settings from result/pause area | Match result context and next-match preferences |
| Help / How to Play | Click settings/reset tutorial link | Help source and selected help topic |

### Exit

| Exit Destination | Trigger | Notes |
|---|---|---|
| Title screen | Back / Esc from title-launched settings | Preferences persist before close |
| Lobby | Back / Esc from lobby-launched settings | Room/lobby state is unchanged |
| Pause overlay | Back / Esc from in-game settings | Returns to pause overlay first, not directly to gameplay, if the pause overlay was the source |
| HUD | Close from non-pausing settings context | Returns to live HUD; no gameplay state mutation except preferences |
| Help overlay | Back from Help-tab deep link | Returns to prior help/rules overlay |

Nested capture states have priority over screen close: Esc cancels a keybind capture before it closes Settings.

---

## Layout Specification

### Information Hierarchy

1. Current category title and short state summary: Accessibility, Controls, Audio, Video, Help.
2. High-impact accessibility controls: colorblind mode, reduced motion, PLACEMENT timer multiplier, UI/HUD scale.
3. Sensory comfort controls: brightness/gamma and audio buses.
4. Control reliability: input remapping, conflict status, reset defaults.
5. Tutorial/help persistence: replay dismissed prompts, reset tutorial flags, open rules.
6. Lower-risk actions: reset category, reset all settings, storage/error messages.

### Layout Zones

Desktop/browser layout uses a two-column settings panel centered over a dimmed background. It is a panel, not a decorative card stack.

```text
+----------------------------------------------------------------+
| Back / Close                         SETTINGS                  |
+---------------------+------------------------------------------+
| Accessibility       | ACCESSIBILITY                            |
| Controls            |                                          |
| Audio               | [Colorblind Mode      Off v] [Preview]   |
| Video               | [Reduced Motion       On/Off]            |
| Help                | [Placement Timer      1x v]              |
|                     | [Menu UI Scale        100% ----o----]    |
|                     | [HUD UI Scale         100% ----o----]    |
|                     |                                          |
|                     | [Reset Accessibility]                    |
+---------------------+------------------------------------------+
| Status: Saved locally / Applies next safe phase / Error text    |
+----------------------------------------------------------------+
```

Responsive behavior:

- At widths below the two-column breakpoint, the category rail becomes a top tab row.
- Content scrolls inside the right/content pane only. Header, Back, and status footer remain fixed.
- UI scale changes must not push Back/Close or category navigation out of reach at 150% scale.

### Component Inventory

| Component | Type | Content | Interactive | Pattern |
|---|---|---|---|---|
| Back / Close button | Text/icon button | "Back" or close icon depending on source | Yes | PTN-NAV-002 variant |
| Category rail / tabs | Navigation list | Accessibility, Controls, Audio, Video, Help | Yes | New: Settings Category Navigation |
| Status footer | Inline status | Saved, pending, applies next phase, storage error | No | PTN-FDB-002 style without input |
| Colorblind mode selector | Segmented select/dropdown | Off, Protanopia, Deuteranopia, Tritanopia | Yes | New: Segmented Preference Control |
| Colorblind preview strip | Data preview | Player rings, class icon pair, auction track, objective dot states, ATK/HP gems | No | New: Accessibility Preview Strip |
| Reduced motion toggle | Toggle | Off/On | Yes | New: Binary Setting Toggle |
| PLACEMENT timer multiplier | Segmented select | 0.5x, 1x, 1.5x, 2x, 3x | Yes | New: Segmented Preference Control |
| Menu UI scale slider | Slider + numeric value | 75% to 150% | Yes | New: Slider with Numeric Value |
| HUD UI scale slider | Slider + numeric value | 75% to 150% | Yes | New: Slider with Numeric Value |
| Brightness slider | Slider + numeric value | -50% to +50% | Yes | New: Slider with Preview |
| Gamma slider | Slider + numeric value | -50% to +50% adjustment | Yes | New: Slider with Preview |
| Display calibration preview | Data preview | Dark/mid/bright tiles plus card/HUD sample | No | New: Accessibility Preview Strip |
| Audio bus sliders | Slider row | Music, SFX, UI, optional Master | Yes | New: Slider with Numeric Value |
| Mute toggle per bus | Icon button/toggle | Mute/unmute | Yes | New: Binary Setting Toggle |
| Input binding row | Action row | Action name, primary binding, alternate binding, reset | Yes | New: Keybind Capture Row |
| Keybind capture modal | Modal | "Press a key or mouse button" | Yes | New: Keybind Capture Modal |
| Conflict message | Inline error | Duplicate binding, reserved browser shortcut, invalid input | No | PTN-FDB-002 |
| Help topic list | List | Tutorial prompts, How to Play, phase help | Yes | New: Help Topic List |
| Reset tutorials button | Secondary/destructive-safe button | "Show Tutorials Again" | Yes | PTN-NAV-001 secondary variant |
| Reset category button | Secondary button | Reset current category to defaults | Yes | Confirmation Modal gap |

Pattern library gaps to update later:

- Settings Category Navigation
- Segmented Preference Control
- Binary Setting Toggle
- Slider with Numeric Value
- Accessibility Preview Strip
- Keybind Capture Row
- Keybind Capture Modal
- Help Topic List
- Confirmation Modal, already anticipated by `design/ux/interaction-patterns.md`

### Category Content

#### Accessibility

| Setting | Control | Default | Requirement |
|---|---|---|---|
| Colorblind mode | Segmented selector | Off | Provides Protanopia, Deuteranopia, and Tritanopia modes. Shape/icon backups remain always-on and are not toggleable. |
| Reduced motion | Toggle | Off | Removes repeated pulses, frame flicker, large panel movement, desaturate/slide reveal effects, and nonessential scale motion. Does not remove necessary unit movement that communicates board state. |
| PLACEMENT timer multiplier | Segmented selector | 1x | Options: 0.5x, 1x, 1.5x, 2x, 3x. At 3x, the 10-second PLACEMENT window becomes 30 seconds. |
| Menu UI scale | Slider/stepper | 100% | 75% to 150%. Applies to menu/settings/lobby UI. |
| HUD UI scale | Slider/stepper | 100% | 75% to 150%. Independent from menu scale per accessibility requirements. |

Colorblind preview must include:

- Player A circle base ring vs Player B diamond base ring.
- Sacrier red vs Cra green class icon pair.
- Auction escalation track with text price values.
- Objective dot active, damaged, destroyed states.
- ATK orange diamond vs HP teal gem.
- Damage number vs healing number direction cue.

#### Controls

Controls are grouped by context:

| Group | Example Actions |
|---|---|
| Global | Confirm, Cancel/Back, Pause, Open Settings, Open Help |
| Menu/Lobby | Create Room, Join Room, Copy Room Code, Browse Class Previous/Next, Confirm Class |
| Shop/Auction | Buy/Select Slot, Refresh Shop, Ready/Retract Ready, Bid +1, Bid +3, Bid +5 |
| Placement | Select Card, Navigate Board Cell, Confirm Cell, Submit Placement, Undo/Unstage |
| Board/Hand | Previous/Next Card, Inspect Card, Dismiss Tooltip |

Binding rules:

- Every player-facing keyboard and mouse action is rebindable.
- No two active actions may share the same key or mouse binding simultaneously.
- Reserved browser/system shortcuts are rejected with an inline explanation.
- A binding row supports Primary and Alternate bindings.
- Reset action restores only that row; Reset Controls restores the whole Controls category.
- The settings screen itself remains keyboard-operable even if gameplay bindings are incomplete.

#### Audio

| Bus | Control | Default | Requirement |
|---|---|---|---|
| Music | Slider 0% to 100% + mute | 80% | Controls background music only |
| SFX | Slider 0% to 100% + mute | 100% | Controls combat, board, auction, and card SFX |
| UI | Slider 0% to 100% + mute | 100% | Controls button, tooltip, panel, countdown UI cues |
| Master | Slider 0% to 100% + mute | 100% | Optional umbrella control; if omitted, the three required buses must remain |

All gameplay-critical audio cues must retain visual backups. Muting UI or SFX must not remove timer numbers, bid text, objective dot shape changes, or combat number direction cues.

#### Video

| Setting | Control | Default | Requirement |
|---|---|---|---|
| Brightness | Slider | 0% | Range -50% to +50%; preview updates live |
| Gamma | Slider | 0% adjustment | Range -50% to +50% adjustment; preview updates live |
| Reset Display | Button | - | Restores brightness/gamma to defaults |

Brightness/gamma controls must not reduce UI text contrast below the Standard tier target. If the rendering path applies brightness/gamma globally, UI contrast must be re-verified after implementation.

#### Help

| Help Item | Behavior |
|---|---|
| How to Play | Opens the same rules content reachable from the title screen |
| Tutorial Library | Lists every tutorial prompt the player has seen or dismissed |
| Replay Prompt | Opens a selected tutorial prompt in read-only mode |
| Show Tutorials Again | Clears tutorial dismissal flags so contextual prompts can appear again |
| Per-prompt reset | Allows one prompt's dismissal flag to be cleared without resetting all tutorials |

Required persisted prompt categories:

- DRAFT_INITIAL "one-time offering / no refresh" explanation.
- DRAFT_AUCTION bid and timer explanation.
- DRAFT_SHOP refresh/ready explanation.
- PLACEMENT staged disclosure: select card -> lane -> cell -> mana split/submit.
- Objective dot real/fake and destroyed-state explanation.
- Mana and reserve mana explanation.

---

## States & Variants

| State / Variant | Trigger | What Changes |
|---|---|---|
| First visit | No stored settings | Accessibility category opens first; all defaults visible |
| Returning visit | Stored last category exists | Opens to last visited category unless deep-linked from Help/Controls |
| Preference changed | Any local setting changed | Setting applies immediately where safe; footer shows "Saved" after persistence succeeds |
| Storage unavailable | Browser localStorage/profile write fails | Inline warning: "Settings could not be saved in this browser session." Runtime value still applies until refresh |
| Active match safe phase | Settings opened from DRAFT_INITIAL or DRAFT_SHOP pause | Local visual/audio/input settings apply immediately; timer multiplier shows "Applies next PLACEMENT" |
| Unsafe phase request | Esc pressed during PLACEMENT, DRAFT_AUCTION, RESOLUTION | No full panel; HUD shows pause/settings request indicator until next safe boundary |
| Keybind capture | Player activates a binding cell | Modal traps focus, waits for key/mouse input, Esc cancels capture |
| Keybind conflict | Captured binding already used | Binding is not saved; conflicting action is named in an inline error |
| Reserved shortcut | Captured binding is browser/system reserved | Binding is rejected with an inline explanation |
| Reduced motion enabled | Toggle On | Motion-heavy previews and all referenced UI transitions switch to cut/fade alternatives |
| Colorblind mode changed | Selector changed | Preview strip updates immediately; game palette updates immediately in menu and at next safe render boundary in-match |
| UI scale changed | Slider moved | Content reflows live; current panel remains in bounds and focused control stays visible |
| Reset category pending | Reset clicked | Confirmation modal appears; keyboard focus moves into modal |
| Help prompt replay | Help item selected | Read-only prompt opens as an overlay inside the Help category; Back returns to Help list |

---

## Interaction Map

Input scope: Mouse click primary. Keyboard Tab, Shift+Tab, Enter, Space, Arrow keys, and Esc supported.

| Element | Action | Input | Immediate Feedback | Outcome |
|---|---|---|---|---|
| Back / Close | Return to source | Click, Enter, Esc | Button highlights; panel closes | Returns to source destination |
| Category rail/tab | Change category | Click, Enter, Arrow Up/Down or Left/Right | Active category highlight moves | Content pane updates; last category stored |
| Colorblind selector | Choose mode | Click, Enter, Arrow keys | Selected option highlights; preview updates | Saves `colorblind_mode` |
| Reduced motion toggle | Toggle | Click, Space, Enter | Toggle changes state; preview switches motion policy | Saves `reduced_motion` |
| Timer multiplier selector | Choose multiplier | Click, Enter, Arrow keys | Value updates; status says when it applies | Saves `placement_timer_multiplier`; may notify active session policy |
| UI scale slider | Adjust scale | Drag, Arrow keys, Page Up/Down | Numeric value and layout update live | Saves menu or HUD scale |
| Brightness/gamma slider | Adjust image calibration | Drag, Arrow keys, Page Up/Down | Preview tiles update live | Saves display adjustment |
| Audio bus slider | Adjust volume | Drag, Arrow keys, Page Up/Down | Optional UI tick plays on UI bus if unmuted | Saves bus volume |
| Mute button | Toggle mute | Click, Space, Enter | Icon/state changes | Saves muted state |
| Input binding cell | Start capture | Click, Enter | Capture modal appears | Awaits key/mouse input |
| Keybind capture modal | Capture binding | Press key or mouse button | Modal shows captured input | Saves if valid and conflict-free |
| Keybind capture modal | Cancel capture | Esc or Cancel button | Modal closes | Existing binding unchanged |
| Reset row/category | Reset | Click, Enter | Confirmation for category/all; row resets immediately | Saves default binding/category values |
| Help topic row | Open topic | Click, Enter | Topic overlay opens inside panel | No gameplay event |
| Show Tutorials Again | Reset tutorial flags | Click, Enter, confirm | Footer shows saved | Clears tutorial dismissal flags |

Keyboard navigation order:

1. Back / Close.
2. Category rail from top to bottom.
3. Current category content from top to bottom, left to right within each row.
4. Status/footer actions, if any.

No disabled control receives keyboard focus. Hidden controls are removed from the focus order. Focus indicators use the project-standard 2px Prism White outline or an equivalent high-contrast ring.

---

## Events Fired

| Player Action | Event / Message Fired | Payload / Data |
|---|---|---|
| Open Settings | `UiSettingsOpened` | `{ source, initial_category }` |
| Close Settings | `UiSettingsClosed` | `{ destination, changed_keys[] }` |
| Change category | `UiSettingsCategoryChanged` | `{ category }` |
| Change colorblind mode | `PreferenceChanged` | `{ key: "colorblind_mode", value }` |
| Toggle reduced motion | `PreferenceChanged` | `{ key: "reduced_motion", value: bool }` |
| Change timer multiplier | `PreferenceChanged` | `{ key: "placement_timer_multiplier", value }` |
| Change menu UI scale | `PreferenceChanged` | `{ key: "menu_ui_scale", value_percent }` |
| Change HUD UI scale | `PreferenceChanged` | `{ key: "hud_ui_scale", value_percent }` |
| Change brightness/gamma | `PreferenceChanged` | `{ key: "brightness" or "gamma", value_percent }` |
| Change audio bus volume | `PreferenceChanged` | `{ key: "volume_music" / "volume_sfx" / "volume_ui" / "volume_master", value_percent }` |
| Start keybind capture | `InputBindingCaptureStarted` | `{ action_id, binding_slot }` |
| Save keybind | `InputBindingChanged` | `{ action_id, binding_slot, input }` |
| Reject keybind conflict | `InputBindingRejected` | `{ action_id, input, reason, conflicting_action_id? }` |
| Reset tutorial flags | `TutorialDismissalFlagsReset` | `{ scope: "all" or prompt_id }` |
| Replay help topic | None | Local read-only UI state |

Server-authoritative timer behavior is not fully specified in current architecture docs. If PLACEMENT timer multiplier affects server phase duration, the implementation needs a network/session preference message or lobby-ready payload extension. This spec treats that as an architecture dependency, not a UI-owned decision.

---

## Transitions & Animations

| Transition | Standard Motion | Reduced Motion |
|---|---|---|
| Settings opens from title/lobby | 120ms fade over dimmed source | Instant open or 80ms fade |
| Settings closes | 120ms fade out | Instant close or 80ms fade |
| Category switch | Content cross-fades 80ms; focus moves to heading | Instant content swap; focus still moves |
| Toggle/selector change | Static state change with small highlight | Same static state change |
| Slider movement | Thumb follows pointer/key; preview updates live | Same, no animated easing |
| Keybind capture modal open | 80ms fade | Instant |
| Help topic overlay | 80ms fade within content pane | Instant |

Prohibited motion:

- No sliding panel from off-screen.
- No repeating glow/pulse in Settings except optional short focus or confirmation highlight.
- No animation that hides text during a category switch.

---

## Data Requirements

| Data | Source System | Read / Write | Notes |
|---|---|---|---|
| Return destination | UI navigation state | Read / Write | Determines Back/Close behavior |
| Current game phase | RSM / phase sink | Read | Determines whether full Settings can open immediately |
| Colorblind mode | Local preferences | Read / Write | Off, Protanopia, Deuteranopia, Tritanopia |
| Reduced motion | Local preferences | Read / Write | Read by HUD, Shop/Auction UI, class picker, animation systems |
| Placement timer multiplier | Local preferences plus server/session policy | Read / Write | UI stores preference; server authority policy unresolved |
| Menu UI scale | Local preferences | Read / Write | Applies to title, lobby, settings, modal overlays |
| HUD UI scale | Local preferences | Read / Write | Applies to in-game HUD independently from menu scale |
| Brightness/gamma | Local preferences / render pipeline | Read / Write | Applies to render calibration; contrast re-verification required |
| Audio bus volumes | Audio mixer/bus settings | Read / Write | Music, SFX, UI; Master optional |
| Input bindings | Input preferences | Read / Write | Keyboard + mouse bindings; conflict-free |
| Tutorial dismissal flags | Local preferences | Read / Write | Drives contextual prompt visibility and Help replay state |
| Help/rules content | Help content data or static copy | Read | Includes How to Play and phase/tutorial explanations |
| Storage capability | Browser/runtime | Read | Detects localStorage/profile availability and failure state |

Preference storage should be abstracted behind a single profile/settings resource so browser `localStorage`, native debug profile files, or future account storage do not leak into UX implementation.

---

## Accessibility

Standard tier. Source: `design/accessibility-requirements.md`.

| Requirement | UX Requirement |
|---|---|
| Keyboard navigation | Entire Settings screen is reachable with keyboard alone. Tab order is deterministic; Arrow keys operate category lists, selectors, and sliders. |
| Focus indicators | Every focused interactive element has a visible 2px Prism White outline or equivalent high-contrast ring. |
| Colorblind modes | Protanopia, Deuteranopia, and Tritanopia modes are selectable. Shape/icon backups remain always-on. |
| Color-independent communication | Preview and setting labels confirm that color is never the only carrier for player side, class identity, objective destruction, ATK/HP, damage/heal, or auction urgency. |
| Reduced motion | Toggle provides alternatives for auction entrance, bid pulse, timer pulse, class reveal, panel motion, frame flicker, and nonessential scale effects. |
| PLACEMENT timer extension | Multiplier selector supports 0.5x, 1x, 1.5x, 2x, 3x; 3x turns 10 seconds into 30 seconds. |
| UI scaling | Menu and HUD scale from 75% to 150%; layout must remain usable at all values. |
| Brightness/gamma | Sliders expose -50% to +50% adjustment with a live preview; UI contrast cannot fall below 4.5:1 for body text. |
| Input remapping | Every player-facing keyboard/mouse action can be rebound; conflicts are blocked and explained. |
| Audio buses | Music, SFX, and UI volume sliders persist independently; critical audio cues retain visual backups. |
| Tutorial persistence | All dismissed tutorial prompts remain available from Help and can be reset. |
| Screen reader considerations | Menu/settings semantic labels should be added if Bevy/browser accessibility support is available. In-game board screen reader support remains outside current scope per accessibility requirements. |
| Timing safety | Full Settings does not open during PLACEMENT, DRAFT_AUCTION, or RESOLUTION; pause request is queued to a safe boundary. |

---

## Localization Considerations

| Element | Risk | Requirement |
|---|---|---|
| Category labels | Medium | Category rail supports 40% text expansion or top-tab wrapping without overlap |
| Selector options | Low | Short values like `1x`, `2x`, `3x` stable; mode names may expand |
| Colorblind mode names | Medium | Allow long medical terms to wrap or use tooltip/description text |
| Input action names | High | Controls table must allow two-line action labels without hiding binding buttons |
| Conflict messages | High | Inline error area supports two lines and does not push focused row offscreen |
| Help topic titles | Medium | List rows support wrap and maintain 44px minimum target height |
| Reset confirmation copy | Medium | Confirmation modal supports two-line title/body |
| Audio bus names | Low | Music/SFX/UI labels are short; localized equivalents still fit one row |

Numbers use simple game formatting, not locale-specific currency/date formatting. Percent values retain `%` unless localization policy changes globally.

---

## Asset / Spec Implications

Asset implications:

- Category icons for Accessibility, Controls, Audio, Video, and Help, if iconography is used.
- Accessibility preview strip assets or reusable mini-render samples: player base rings, class icon pair, auction escalation chips, objective dot states, ATK/HP gem samples, damage/heal direction samples.
- Slider, toggle, segmented selector, keybind row, capture modal, and high-contrast focus-ring UI assets in the existing Ink Blue / Arcane Gold / Prism White visual language.
- Audio bus icons and mute/unmute icons.
- Help/tutorial prompt iconography, including reset/replay affordances.

Spec implications:

- `design/ux/interaction-patterns.md` should add the new settings controls listed in Component Inventory before implementation stories begin.
- Architecture needs a preference storage contract covering browser `localStorage`, native debug persistence, and future profile/account storage.
- Network/session design must resolve how PLACEMENT timer multiplier affects authoritative multiplayer timers.
- Audio implementation needs named mixer buses: Music, SFX, UI, and optionally Master.
- Help/tutorial content needs a maintained prompt registry so dismiss/reset/replay behavior is data-driven rather than hardcoded per screen.

---

## Acceptance Criteria

- [ ] Settings opens from the title screen Settings button and returns to the title screen without changing room/session state.
- [ ] Settings opens from lobby/pause safe contexts and returns to the exact source context on Back/Esc.
- [ ] During PLACEMENT, DRAFT_AUCTION, and RESOLUTION, pressing Esc does not open the full Settings panel; it shows a pause/settings request indicator and opens at the next safe boundary.
- [ ] Accessibility category exposes colorblind mode, reduced motion, PLACEMENT timer multiplier, menu UI scale, and HUD UI scale controls.
- [ ] Colorblind selector includes Off, Protanopia, Deuteranopia, and Tritanopia, and the preview strip updates immediately when changed.
- [ ] Reduced-motion mode removes repeated pulses, frame flicker, panel slide/expand motion, class reveal slide/desaturate, and nonessential scale motion while preserving readable state changes.
- [ ] PLACEMENT timer multiplier options include 0.5x, 1x, 1.5x, 2x, and 3x; 3x maps the 10-second PLACEMENT window to 30 seconds.
- [ ] Menu UI scale and HUD UI scale each support 75% to 150% and remain independently configurable.
- [ ] At 1366x768 and 1920x1080, at 75%, 100%, and 150% UI scale, no settings text, button, slider, category label, or status message overlaps another required UI element.
- [ ] Brightness and gamma controls each support -50% to +50% adjustment and show a live calibration preview.
- [ ] Music, SFX, and UI audio buses each have independent 0% to 100% volume controls and mute states that persist.
- [ ] Controls category allows every player-facing keyboard/mouse action to be rebound with Primary and Alternate slots.
- [ ] Attempting to bind an already-used key/mouse input is rejected and names the conflicting action.
- [ ] Reserved browser/system shortcuts are rejected with an inline explanation and do not overwrite the existing binding.
- [ ] Keybind capture modal traps focus until a valid input is captured, the player cancels, or an error returns focus to the source row.
- [ ] Help category lists dismissed tutorial prompts, can replay them read-only, and can reset all tutorial dismissal flags.
- [ ] All interactive Settings elements are reachable via keyboard Tab/Shift+Tab and activatable with Enter or Space where appropriate.
- [ ] Focus order follows Back/Close, category navigation, current category content, footer actions.
- [ ] Preferences persist across browser refresh when storage is available.
- [ ] If preference storage is unavailable, the current runtime setting still applies and the screen shows a clear save warning.

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-SA-1 | How should PLACEMENT timer multiplier work in multiplayer: highest requested multiplier across both players, room-host setting, per-player queueing, or another server-authoritative policy? | UX Designer + Lead Programmer + Producer | High |
| OQ-SA-2 | Should the 0.5x timer option ship, or should the minimum be 1x as raised by `design/ux/hud.md` OQ-HUD-5? | Producer | Medium |
| OQ-SA-3 | What is the canonical persistence layer for settings: browser `localStorage`, a player preferences resource, or a future profile/account store? | Lead Programmer | High |
| OQ-SA-4 | Can Bevy 0.18/browser builds expose semantic names/roles for menu and Settings controls, or is screen reader support deferred entirely? | Lead Programmer | Medium |
| OQ-SA-5 | Which browser/system shortcuts are forbidden for input remapping on the WASM target? | UI Programmer | Medium |
| OQ-SA-6 | Are brightness/gamma implemented as a WebGL post-process, shader uniform, CSS/canvas filter, or renderer-level calibration step? UI contrast verification depends on this. | Technical Artist + UI Programmer | Medium |
| OQ-SA-7 | Should full Settings be available during DRAFT_AUCTION if both players are waiting on server settlement, or is DRAFT_AUCTION always treated as unsafe until the next phase? | Producer + UX Designer | Low |
| OQ-SA-8 | Should audio include a Master bus in addition to the required Music/SFX/UI buses? | Audio Director | Low |
| OQ-SA-9 | Who owns final tutorial/help copy and the prompt registry structure? | Writer + UX Designer | Medium |
