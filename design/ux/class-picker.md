# UX Spec: Class Picker

> **Status**: Complete — pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-05-01
> **Journey Phase(s)**: LOBBY — Anticipation
> **Template**: UX Spec
> **Input Methods**: Mouse + Keyboard (primary: mouse click). No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (`design/accessibility-requirements.md`)
> **Source GDD**: `design/gdd/class-system.md` § UI Requirements
> **Related spec**: `design/ux/main-menu.md` (lobby container — this spec is the deep-dive on the class picker component)

---

## Purpose & Player Need

The class picker serves the player's need to **commit to a strategic identity** before the information war begins. The player arrives wanting to choose the class whose tempo signature matches their intended playstyle — they need enough information to make a meaningful choice (one-line tempo description + signature cards), but not so much that the lobby becomes a tutorial. The choice is permanent for the session: the class is the longest gear in the player's clockwork, wound here and ticked for the next 10–15 minutes.

**What goes wrong if this is hard to use:** A player who picks a class without understanding it discovers mid-game that they built Xelor's reserve but don't know how to spend it. A player who accidentally confirms the wrong class (misclick on carousel) cannot undo it — the frustration poisons the opening rounds. A player who cannot tell which class matches their instincts abandons the picker at random rather than with intent.

**Pillar alignment**: Simple surface — each class identity fits in one phrase, first-read legible. Deep emergence — six clockworks means six different games on the same board; the picker is the first moment of strategic differentiation.

**OQ-MM-1 resolution**: Show the one-line tempo signature and the 4 signature Krosmic card names (with hover tooltips per card). No paragraph summaries — the one-liner is sufficient for the Simple surface pillar. Tooltip depth is available on demand for players who want it.

---

## Player Context on Arrival

**When encountered**: Every session — the class picker is unavoidable. There is no default class; every player must actively choose before the game can start.

**What they were just doing**: They clicked Create Room or joined via a friend's room code. Their attention is on two things: (1) the room code they need to share or just used, and (2) getting ready to play.

**Emotional state**: *Anticipation edged with appraisal* (art bible § LOBBY mood). Composed, watchful, deliberate. Not rushed — the 90-second lobby timer is generous. If the opponent hasn't connected yet, the player is in a waiting state that naturally encourages browsing the class options. If the opponent is already connected, there's mild social pressure to confirm quickly.

**Voluntary vs. sent**: The player arrives voluntarily by loading the URL. The class picker is not a screen they navigate to — it is the lobby itself. It is always in front of them. This means the picker must work both as a first-look browsable overview (new players exploring all 6) and as a fast-confirm surface (returning players who already know what they want).

**First-time vs. returning distinction**:
- **First-time player**: Needs to read the tempo signatures. Will browse all 6. Hover tooltips on Krosmic cards are critical for them — 4 card names without context are meaningless.
- **Returning player**: Arrives knowing their class. Carousel should default to their last-played class (if stored in localStorage) or Iop (simplest tempo, most legible on first encounter) as the fallback. They confirm in under 5 seconds.

---

## Navigation Position

This spec describes the class picker component embedded within the Lobby screen. It is not a separate navigation destination.

```
[Browser URL load]
        ↓
  TITLE SCREEN
  └── Create Room / Join Room
              ↓
        LOBBY SCREEN  ←── This spec lives here
        ├── Room code display          (main-menu.md)
        ├── Player slots               (main-menu.md)
        ├── [CLASS PICKER COMPONENT]   (this spec)
        │   ├── Class carousel
        │   ├── Class info panel
        │   └── Confirm Class button
        ├── Cancel button              (main-menu.md)
        └── Lobby timer               (main-menu.md)
              ↓
        IN-GAME HUD (on game start)
```

The class picker is always visible from the moment the player enters the lobby until the game starts. It is not collapsed, tabbed, or revealed by a separate action. There are no alternate entry paths to the picker — the only way to reach it is through the lobby.

---

## Entry & Exit Points

**Entry:**

| Entry Source | Trigger | Player carries this context |
|---|---|---|
| Lobby created | Player clicks Create Room; lobby initialises | No class selected; carousel at default position (last-played class or Iop) |
| Lobby joined | Player clicks Join Room; `S2CJoinAck` received | No class selected; carousel at default position |
| Ready retracted (own) | Player retracts their Ready before both players commit | Previously selected class restored in carousel; picker re-enabled; confirm button re-enabled |

**Exit:**

| Exit Destination | Trigger | Notes |
|---|---|---|
| Picker locked (own Ready confirmed) | Player clicks Confirm Class; `C2SConfirmClass` sent | Class frozen; carousel disabled; confirm button replaced by locked indicator. Reversible — player can retract until opponent also commits. |
| Picker permanently locked (game start) | `S2CClassesRevealed` fires; both players committed | Irreversible. Picker enters read-only display. Lobby transitions to game start. |
| Lobby cancelled | `S2CSessionCancelled` received, or player clicks Cancel | Picker state discarded; player returns to title screen. |

**One-way note**: Once `S2CClassesRevealed` fires, the class choice cannot be changed for any reason — the picker exits permanently into read-only state and the lobby transitions to the game.

---

## Layout Specification

### Information Hierarchy

Priority order within a single displayed class entry:

1. **Class portrait / figurine** — identity anchor; visual recognition before any text is read
2. **Class name** — large, unambiguous ("Xelor", "Sadida", etc.)
3. **Tempo signature** — one-line description; primary decision-support text for new players
4. **4 Krosmic card names** — tooltip-gated depth for players who want it
5. **Carousel position indicator** — "3 of 6" dot nav; communicates how many classes exist
6. **Confirm Class button** — commitment action; never the first element the eye lands on

### Layout Zones

**Option C — Portrait-centered card frame.** A single card fills the picker zone: portrait centred at top, info below, confirm button beneath the card frame. Carousel arrows float left and right of the card. Card uses a Krosmaga-style card frame background with a class-tinted border — thematically consistent with the game's visual language and the strongest visual identity of the three options considered.

### Component Inventory

| Component | Zone | Type | Content | Interactive | Pattern |
|---|---|---|---|---|---|
| Class card frame | Full picker zone | Decorative container | Krosmaga-style card frame background; class-tinted border | No | New — flag for library: Card Frame Container |
| Class portrait | Top of card | Image display | Full character illustration for current class | No | PTN-DSP-007 (Class Figurine Display) |
| Class name | Below portrait | Display text | e.g. "Xelor" — large, prominent | No | — |
| Tempo signature | Below class name | Display text | One-line description, e.g. "Reserve tempo — slow accumulator, fires once" | No | — |
| Krosmic card list | Below tempo | List of 4 text items | Card names; each is a hover target for tooltip | Hover → tooltip | PTN-FDB-004 (Hover Tooltip) |
| Carousel arrow — prev | Left of card | Icon button | ◀ | Yes | PTN-INP-002 variant (flanking card frame) |
| Carousel arrow — next | Right of card | Icon button | ▶ | Yes | Same |
| Position indicator | Bottom of card | Dot nav (6 dots) | Filled dot = current class; empty = others | No | New — flag for library: Dot Position Indicator |
| Confirm Class button | Below card frame | Primary button | "Confirm Class" / locked: "✓ Locked In" | Yes | PTN-NAV-001 (Primary Action Button) |
| Retract Ready link | Below confirm button | Text link | "Change class" — visible only in own-locked state, before both players commit | Yes | PTN-NAV-002 (Text/Icon Link Button) |

**New patterns to add to library**: Card Frame Container, Dot Position Indicator.

### ASCII Wireframe

```
┌─────────────────────────────────────────┐
│                                         │
│   ◀        ┌─────────────┐        ▶    │
│            │  [PORTRAIT] │             │
│            │             │             │
│            │   XELOR     │             │
│            │─────────────│             │
│            │ Reserve tempo — slow      │
│            │ accumulator, fires once   │
│            │─────────────│             │
│            │ ◆ Rollback              ⓘ │
│            │ ◆ Garde-Temps           ⓘ │
│            │ ◆ Miss Nuit             ⓘ │
│            │ ◆ Dévouement            ⓘ │
│            │─────────────│             │
│            │  ○ ○ ● ○ ○ ○│             │
│            └─────────────┘             │
│                                         │
│          [ CONFIRM CLASS ]              │
│                                         │
└─────────────────────────────────────────┘
```

`ⓘ` = hover target; triggers PTN-FDB-004 tooltip with card effect summary. Filled dot `●` = current class (position 3 of 6 shown). Carousel arrows flank the card frame. "Change class" retract link appears below the confirm button only in the own-locked state.

---

## States & Variants

| State / Variant | Trigger | What Changes |
|---|---|---|
| **Default — browsing** | Lobby entered; no class confirmed | Carousel active; all components interactive; confirm button enabled once any class is focused |
| **Own class confirmed (soft lock)** | Player clicks Confirm Class; `C2SConfirmClass` sent | Carousel arrows disabled; card frame gains lock overlay (padlock icon or gold border pulse); confirm button changes to "✓ Locked In" (disabled); "Change class" retract link appears below |
| **Own Ready retracted** | Player clicks "Change class"; `C2SRetractReady` sent | Carousel re-enabled; card frame returns to browsing state; confirm button re-enabled; retract link hidden |
| **Opponent confirmed (waiting)** | `S2CClassLocked` received | No change to own picker; opponent slot in lobby header updates to "Ready ✓" — class still hidden |
| **Both confirmed — reveal** | `S2CClassesRevealed` received | Picker enters permanent read-only state; card frame animates reveal (see Transitions); carousel arrows hidden; confirm/retract hidden; both class portraits visible simultaneously in player slots |
| **Empty / loading** | Class data not yet loaded from `SessionConfig` | Card frame shows skeleton placeholder (portrait grey box, name and tempo as shimmer lines); confirm button disabled; carousel arrows disabled |
| **Lobby cancelled** | `S2CSessionCancelled` received | Picker state discarded; lobby overlay fires (main-menu.md); picker not rendered on return to title |

**OQ-MM-2 resolution**: Opponent's class is not previewed before they confirm. Their slot shows "Ready ✓" once locked, but class name and portrait remain hidden until `S2CClassesRevealed`. This preserves the reveal moment as the first beat of the information war.

---

## Interaction Map

*Input methods: Mouse + Keyboard. No gamepad. No touch. WASM browser.*

| Element | Action | Input | Immediate Feedback | Outcome |
|---|---|---|---|---|
| Carousel arrow ◀ (prev) | Click / Left arrow key | Mouse click or `←` (when ◀ or ▶ has focus) | Arrow highlights; card frame cross-fades to previous class (200ms); dot indicator updates | Previous class displayed; name, tempo, Krosmics update; wraps class 1 → class 6 |
| Carousel arrow ▶ (next) | Click / Right arrow key | Mouse click or `→` (when ◀ or ▶ has focus) | Arrow highlights; card frame cross-fades to next class (200ms); dot indicator updates | Next class displayed; wraps class 6 → class 1 |
| Krosmic card name (hover) | Hover | Mouse hover (150ms delay) | PTN-FDB-004 tooltip fades in above the name (flip below if near viewport top); card effect summary text | Tooltip visible while hovered; dismissed on mouse-out |
| Krosmic card name (keyboard) | Focus | Tab to name | Same tooltip triggered on focus | Dismissed on blur |
| Confirm Class button | Click / Enter | Mouse click or keyboard Enter | Scale 0.96 press (80ms); transitions to "✓ Locked In"; card frame gains lock visual | Sends `C2SConfirmClass { class_id }`; picker enters soft-lock state |
| Confirm Class button — disabled | Attempt click | Mouse click | No response; cursor default | No action |
| "Change class" retract link | Click / Enter | Mouse click or keyboard Enter | Brief highlight; lock overlay dissolves; card returns to browsing state | Sends `C2SRetractReady`; picker re-enters browsing state |
| Dot position indicator | — | Not interactive | — | Display only |

**Carousel wrap**: Navigating past class 6 wraps to class 1 and vice versa — no dead ends.

**Tooltip placement**: Above the card name by default; flips below if picker is near viewport top. Never obscures portrait or confirm button.

**Tab order in soft-lock state**: Carousel arrows non-interactive. Tab order collapses to: "Change class" link → Cancel button (lobby shell).

---

## Events Fired

| Player Action | Event Fired | Payload / Data | Notes |
|---|---|---|---|
| Navigate carousel (prev/next) | `C2SSelectClass` (optional) | `{ class_id }` | Client-only preview; no server commitment. Not required before `C2SConfirmClass`. Reserved for opponent preview feature if added later — not in current scope. |
| Click Confirm Class | `C2SConfirmClass` | `{ class_id }` | **Commits class to server.** Modifies persistent session state — class locked until retracted. |
| Click "Change class" (retract) | `C2SRetractReady` | `{ player_id }` (implicit from connection) | Reverses the lock. Valid only before `S2CClassesRevealed`; server rejects if both players already committed. |
| Hover Krosmic card name | None | — | Pure client-side tooltip; no server or analytics event. |
| *(inbound)* Both players committed | `S2CClassesRevealed` received | `{ player_a_class, player_b_class }` | Not fired by this component. Triggers permanent lock and reveal animation. |

**Idempotency requirement**: `C2SConfirmClass` and `C2SRetractReady` both modify server-side session state. Double-sending `C2SConfirmClass` with the same `class_id` must be a no-op, not an error.

---

## Transitions & Animations

**Screen enter (lobby → picker visible):**
Card frame slides up from below (150ms, ease-out). Carousel arrows fade in (100ms, 50ms delay after card). No bounce — composed arrival matches the LOBBY mood.

**Carousel navigation:**
Card frame cross-fades to the next class (200ms, opacity 0→1 on incoming frame, outgoing fades simultaneously). Portrait cross-fades within the frame — does not slide, to avoid implying physical ordering. Dot indicator updates instantly.

**Confirm Class (soft lock):**
Gold border pulse on the card frame (one 300ms pulse — scale 1.0 → 1.02 → 1.0, border brightens then settles). Padlock icon fades in over portrait (150ms). Confirm button text swaps to "✓ Locked In" (instant swap). Carousel arrows fade to 30% opacity and become non-interactive.

**Retract Ready ("Change class"):**
Padlock icon fades out (100ms). Gold border pulse reverses (150ms desaturate). Carousel arrows return to full opacity (100ms). Confirm button text swaps back to "Confirm Class".

**Class reveal (`S2CClassesRevealed`):**
The first moment of the information war — highest animation weight in the lobby.
- Own card frame: desaturate-to-resaturate flash (art bible § LOBBY "cold assessment" beat) — 400ms total
- Opponent portrait slides in from right into their player slot simultaneously with own portrait finalising on left — both reach final position on the same frame
- Hold both portraits visible 1.5s before lobby transitions to game start
- Carousel arrows and confirm/retract controls hidden immediately on reveal trigger (no fade)

**Reduced-motion alternative** (accessibility-requirements.md Standard tier):
All cross-fades become instant cuts. Gold border pulse replaced by static gold border. Class reveal is an instant cut to both portraits — no slide, no desaturate flash. The 1.5s hold before game start is preserved.

---

## Data Requirements

| Data | Source System | Read / Write | Notes |
|---|---|---|---|
| Class list (6 entries) | `SessionConfig` / static game data | Read | Loaded at app startup; not fetched per-lobby. Contains: `class_id`, `name`, `tempo_signature`, `krosmic_card_names[4]`, `portrait_asset_path` |
| Krosmic card effect summaries | `cards.json` (Card Data & Pool) | Read | Tooltip text per card name — one line per card, not full card text. Loaded at startup alongside class list. |
| Last-played class (carousel default) | `localStorage` (client-only) | Read / Write | Written on `C2SConfirmClass`; read on lobby entry to set carousel start position. Falls back to Iop if absent. No server involvement. |
| Own class selection (pre-confirm) | Local client state | Read / Write | Carousel position only; not persisted until `C2SConfirmClass` sent. |
| Own lock state | Server (`S2CGameSnapshot.player.class_locked`) | Read | Drives soft-lock state. Server is source of truth; client mirrors it. |
| Opponent lock state | `S2CClassLocked { player_id }` | Read | Drives "Ready ✓" display on opponent slot. Opponent class remains hidden. |
| Both classes (reveal) | `S2CClassesRevealed { player_a_class, player_b_class }` | Read | Drives permanent lock + reveal animation. Class IDs map to local class metadata for portrait lookup. |

**Architectural note**: The picker never owns or writes authoritative session state. `localStorage` is the only client-side persistence, and it is advisory (carousel default only). All lock decisions are server-authoritative.

**Optimistic update behaviour**: Client applies soft-lock state optimistically on `C2SConfirmClass` send without waiting for server acknowledgment. If the server rejects the message (edge case — e.g. race condition with `S2CClassesRevealed`), the client reverts to browsing state and the lobby shell surfaces a session-level error.

**Krosmic tooltip null handling**: If a card's effect summary is absent from `cards.json`, the tooltip displays the card name only with no effect text — no broken or empty tooltip bubble.

---

## Accessibility

*Standard tier. Source: `design/accessibility-requirements.md`.*

| Requirement | How addressed |
|---|---|
| **Keyboard navigation** | Tab order (browsing): ◀ arrow → ▶ arrow → Krosmic name 1 → 2 → 3 → 4 → Confirm Class button. Tab order (soft-lock): "Change class" link → Cancel button (lobby shell). No mouse-only interactions. |
| **Carousel keyboard control** | `←` / `→` arrow keys navigate the carousel when focus is on either the ◀ or ▶ arrow button. The card frame itself is not a focusable tab stop — arrow key navigation is accessed through the arrow buttons. |
| **Krosmic tooltips — keyboard** | Tooltip triggers on focus (Tab), not hover-only. Dismissed on blur. Tooltip text readable by screen reader (`role="tooltip"`, referenced by `aria-describedby` on the card name element). |
| **Class name always text** | Class name and tempo signature always rendered as text, never image-only. Portrait is decorative (`alt=""`); class identity is communicated by the text name. |
| **Color-independent class identity** | Class icon/figurine always shown alongside class color (art bible §7.5). No class identified by color alone. Colorblind modes do not break class recognition. |
| **Confirm button — disabled state** | Communicated by 40% opacity and `aria-disabled="true"` — not color alone. |
| **Lock state — non-color indicator** | Padlock icon communicates locked state. Gold border is supplementary, not the sole indicator. |
| **Dot position indicator** | Current dot carries `aria-label="Class 3 of 6"` (or equivalent). Filled dot is also slightly larger than empty dots — shape backup, not color-only. |
| **Text contrast** | Class name, tempo signature, Krosmic card names: minimum 4.5:1 on card frame background. Confirm button label: minimum 4.5:1. |
| **Focus indicators** | All interactive elements have 2px Prism White outline ring on keyboard focus (consistent with PTN-NAV-001 and site-wide focus style). |
| **Reduced-motion** | All animations have instant-cut alternatives — see Transitions & Animations. Toggle in settings. |
| **No time pressure** | Class selection has no picker-specific timer. The 90s lobby timer is a shared session timer. Players have the full lobby window to browse and confirm. |

---

## Localization Considerations

| Element | Max length risk | Notes |
|---|---|---|
| Class name ("Xelor", "Sadida", etc.) | Low | French-origin names; consistent across all localizations. No expansion risk. |
| Tempo signature (one-liner) | **High** | English baseline ~47 chars. German/French translations can run 40–60% longer. Reserve two lines of height in the card frame layout — this is the design constraint, not one-line English. Truncate with ellipsis only as last resort; prefer wrap. **HIGH PRIORITY for localization engineer.** |
| Krosmic card names (4 items) | Medium | Proper nouns, mostly stable. Allow ~30% expansion per name. Each name stays on one line; truncate with ellipsis if needed, full name available in tooltip. |
| Krosmic tooltip text (effect summary) | Medium | Allow two-line wrap inside tooltip bubble — tooltip width can flex. Do not truncate tooltip text. |
| "Confirm Class" button label | Medium | French "Confirmer la classe" and German equivalents are significantly longer than English. Button expands horizontally or wraps to two lines before truncating. Minimum 44px height preserved. |
| "✓ Locked In" label | Low | Short confirmation string — most translations similarly compact. |
| "Change class" retract link | Low | Short action string — low expansion risk. |
| Dot indicator aria-label ("Class 3 of 6") | Low | Numeric pattern localizes cleanly. |

**Layout rule**: Reserve two lines of vertical space for the tempo signature in all layout calculations. One-line English is the best case, not the constraint.

---

## Acceptance Criteria

**Performance**
- [ ] Lobby with class picker renders within 500ms of `S2CJoinAck` / `S2CRoomCreated` (class list and portraits loaded at app startup, not on lobby entry)
- [ ] Carousel navigation between classes completes the cross-fade within 200ms with no dropped frames at 60 FPS

**Navigation / lifecycle**
- [ ] Carousel navigates through all 6 classes; wraps from class 6 → class 1 and class 1 → class 6
- [ ] Clicking Confirm Class sends `C2SConfirmClass { class_id }`, disables the carousel, and displays locked state (padlock icon + "✓ Locked In")
- [ ] "Change class" retract link is visible only in soft-lock state and sends `C2SRetractReady` on click; picker returns to browsing state with carousel re-enabled
- [ ] Once `S2CClassesRevealed` is received, the picker is permanently read-only — carousel arrows, confirm button, and retract link are all removed; no player action can re-enable them

**Opponent reveal**
- [ ] Opponent slot shows "Ready ✓" after `S2CClassLocked` but does NOT show the opponent's class name or portrait until `S2CClassesRevealed`
- [ ] On `S2CClassesRevealed`, both class portraits animate in simultaneously — own from left, opponent from right — reaching final positions on the same frame

**Empty / loading state**
- [ ] If class metadata has not loaded when the lobby is entered, the card frame renders a skeleton placeholder; carousel and confirm button are non-interactive until data is available

**Accessibility**
- [ ] All interactive elements (carousel arrows, Krosmic card names, confirm button, retract link) are reachable via Tab in logical order
- [ ] Krosmic card name tooltips trigger on keyboard focus (not hover-only) and dismiss on blur
- [ ] Class name and tempo signature are always rendered as text — removing CSS does not leave the class unidentified
- [ ] Locked state is communicated by padlock icon, not color alone

**Core purpose — class information**
- [ ] Each class entry displays: class portrait, class name, one-line tempo signature, and exactly 4 Krosmic card names
- [ ] Hovering (or focusing) a Krosmic card name displays a tooltip with that card's effect summary; tooltip does not obscure the portrait or confirm button
- [ ] Last-confirmed class is restored as carousel default on next lobby entry (from `localStorage`); falls back to Iop if absent

**Layout / viewport**
- [ ] Card picker layout remains usable at viewport widths ≥ 320px — card frame, carousel arrows, and confirm button do not overlap or overflow at minimum supported width

---

## Open Questions

| # | Question | Owner | Priority | Status |
|---|---|---|---|---|
| OQ-CP-1 | Should `C2SSelectClass` be sent on every carousel navigation step for future opponent-preview feature, or suppressed entirely until `C2SConfirmClass`? Current spec suppresses it — revisit if opponent class preview is added. | ux-designer / lead-programmer | Low | Open |
| OQ-CP-2 | Two new interaction patterns flagged for the library: **Card Frame Container** and **Dot Position Indicator**. Add to `design/ux/interaction-patterns.md` before class picker stories begin. | ux-designer | Medium | Open — pattern library update needed |
| OQ-MM-1 | *(Resolved here)* Show one-line tempo signature + 4 Krosmic names with hover tooltips. No paragraph summaries. | — | — | **Closed** |
| OQ-MM-2 | *(Resolved here)* Opponent class NOT shown before both commit. Opponent slot shows "Ready ✓" only. | — | — | **Closed** |
