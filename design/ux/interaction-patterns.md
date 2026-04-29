# Interaction Pattern Library

> **Status**: Complete — pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-04-29
> **Template**: Interaction Pattern Library
> **Source Specs**: `design/ux/main-menu.md`, `design/ux/hud.md`
> **Input Methods**: Mouse + Keyboard. No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (`design/accessibility-requirements.md`)

---

## Overview

This library catalogs the reusable interaction patterns extracted from all Lanes and Lies UX specs. When designing a new screen or flow, reference these patterns by ID rather than reinventing behavior. New patterns introduced by future specs are added here at the time of spec completion.

**Total patterns: 18**

---

## Pattern Catalog

| ID | Pattern | Category | Used In |
|---|---|---|---|
| PTN-NAV-001 | Primary Action Button | Navigation | main-menu |
| PTN-NAV-002 | Text / Icon Link Button | Navigation | main-menu |
| PTN-INP-001 | Text Input + Action Button Pair | Input | main-menu |
| PTN-INP-002 | Carousel Browser | Input | main-menu |
| PTN-INP-003 | Clipboard Copy Button | Input | main-menu |
| PTN-FDB-001 | Button Loading Spinner | Feedback | main-menu |
| PTN-FDB-002 | Inline Error Message | Feedback | main-menu |
| PTN-FDB-003 | Animated Number Delta | Feedback | hud |
| PTN-FDB-004 | Hover Tooltip | Feedback | hud |
| PTN-DSP-001 | Resource Counter | Data Display | hud |
| PTN-DSP-002 | Segmented Resource Bar | Data Display | hud |
| PTN-DSP-003 | Diamond Resource Display | Data Display | hud |
| PTN-DSP-004 | Phase Label | Data Display | hud |
| PTN-DSP-005 | Urgency Countdown Timer | Data Display | hud, main-menu (lobby) |
| PTN-DSP-006 | Status Dot | Data Display | hud |
| PTN-DSP-007 | Class Figurine Display | Data Display | hud, main-menu (lobby) |
| PTN-DSP-008 | Horizontal Card Row | Data Display | hud |
| PTN-OVR-001 | Player Slot | Overlay / Modal | main-menu (lobby) |

---

## Patterns

### PTN-NAV-001 — Primary Action Button

**Category**: Navigation
**Used In**: `design/ux/main-menu.md` (Create Room, Confirm Class, Join Room)

**Description**: The primary call-to-action button for a screen or phase. Used when there is one clear action the player should take next. Visually dominant — heavier weight and stronger color than secondary/ghost buttons on the same screen. Only one primary button should exist per visible context.

**Specification**:
- Visual: Filled background, Arcane Gold `#F5C842` label, Void `#1A1520` background or class-colored fill per context
- Size: Minimum 44×44 CSS px click target
- Focus: 2px Prism White outline ring on keyboard focus
- Disabled state: 40% opacity; not interactive (cursor default); reason communicated by context (e.g., empty input = Join Room disabled)
- Loading state: Button text replaced by spinner — see PTN-FDB-001; button disabled while server responds
- Input: Mouse click or keyboard Enter
- Feedback: Brief visual press (scale 0.96, 80ms) on click; loading spinner if async operation

**When to Use**: One clear, high-commitment action per visible context (Create Room, Confirm Class, Submit Bid).

**When NOT to Use**: Destructive or reversible actions (use ghost/text button). More than one primary action per context (demote the secondary to a ghost button).

**Reference**: `design/ux/main-menu.md` — Create Room button, Confirm Class button

---

### PTN-NAV-002 — Text / Icon Link Button

**Category**: Navigation
**Used In**: `design/ux/main-menu.md` (Settings ⚙, How to Play ?)

**Description**: A low-visual-weight navigation button for tertiary or quaternary actions. May be icon-only, text-only, or icon + text. Never competes with primary/secondary buttons for attention. Typically positioned at screen corners or footer.

**Specification**:
- Visual: No fill, no border. Icon or text only. Muted color (Prism White or Ivory at 70% opacity).
- Size: Minimum 44×44 CSS px click target even if the visible icon is smaller
- Focus: 2px Prism White outline ring on keyboard focus
- Hover: Opacity rises to 100%; subtle underline or background tint
- Input: Mouse click or keyboard Enter via Tab navigation
- No disabled state — if the destination is unavailable, hide the element entirely

**When to Use**: Settings, Help, About, Cancel-type navigation that should not compete with the primary action.

**When NOT to Use**: Actions that modify game state directly. Primary calls-to-action.

**Reference**: `design/ux/main-menu.md` — Settings button, How to Play button

---

### PTN-INP-001 — Text Input + Action Button Pair

**Category**: Input
**Used In**: `design/ux/main-menu.md` (Room code input + Join Room button)

**Description**: A text input field paired inline with an action button. The button is disabled until the input contains valid content. On narrow viewports, the pair stacks vertically.

**Specification**:
- Input field: Rounded rectangle, Ink Blue `#1E2E40` fill, Prism White `#EAE8E2` text, 2px Stone Grey border. Placeholder text at 50% opacity.
- Character constraints: Applied at input (max length enforced client-side; normalization e.g. auto-uppercase applied on keystroke)
- Button: Secondary button style when a separate primary button exists on the same screen; becomes primary style when it is the dominant action
- Disabled when: Input is empty or below minimum valid length
- Focus interaction: Clicking into the input shifts visual emphasis toward the paired button
- Keyboard: Enter in the input field activates the paired button
- Input: Keyboard text entry; mouse click or Enter to activate button

**When to Use**: Any flow where a short user-entered code or string gates a server action.

**When NOT to Use**: Long-form text entry (use a full form layout). Multi-field forms.

**Reference**: `design/ux/main-menu.md` — Room code input + Join Room pair

---

### PTN-INP-002 — Carousel Browser

**Category**: Input
**Used In**: `design/ux/main-menu.md` (Class browser in lobby)

**Description**: A single-item viewer with previous/next navigation arrows. The current item fills the display area; adjacent items are not visible. Selection is committed via a separate explicit action (PTN-NAV-001), not by browsing itself.

**Specification**:
- Display area: Dominant item (portrait, name, brief description)
- Arrows: Left/right chevron buttons flanking the display area
- Browse action: Arrow button click or keyboard left/right arrow keys
- Preview vs. commit: Browsing is preview-only. A separate Confirm action commits the selection.
- Animation: Item transitions with horizontal slide or crossfade (200ms). Reduced-motion: instant cut.
- Accessibility: Current item name announced on change; arrow buttons labeled "Previous [item]" / "Next [item]"
- Open question: Wrap behavior (circular vs. stop at boundary) — see OQ-PAT-1

**When to Use**: Browsing a small ordered list (3–20 items) where the player should focus on one item at a time.

**When NOT to Use**: Large unordered lists (use a grid or list). Comparisons requiring two items visible simultaneously.

**Reference**: `design/ux/main-menu.md` — Class browser

---

### PTN-INP-003 — Clipboard Copy Button

**Category**: Input
**Used In**: `design/ux/main-menu.md` (Room code copy)

**Description**: A copy-to-clipboard icon button adjacent to a read-only text value. On click, the value is copied and a brief tooltip confirms success. The underlying text is always selectable as a fallback.

**Specification**:
- Icon: Clipboard or copy icon (📋 or equivalent SVG). Button labeled "Copy room code" for screen readers — not just "Copy".
- Click feedback: "Copied!" tooltip fades in immediately, holds for 1.5s minimum, fades out (1.5s meets WCAG 2.1 timing requirement)
- Tooltip position: Above or to the right of the button; never obscures the source text
- Failure state: If clipboard API is unavailable (browser permission denied), tooltip shows "Select and copy manually" and the text field gains focus
- The source text must always be selectable — copy button is supplementary
- Accessibility: 1.5s display minimum; button label describes the content being copied

**When to Use**: Read-only values the player needs to share or transfer (room codes, invite links, reference IDs).

**When NOT to Use**: Long text blocks where selection is more practical.

**Reference**: `design/ux/main-menu.md` — Room code copy button

---

### PTN-FDB-001 — Button Loading Spinner

**Category**: Feedback
**Used In**: `design/ux/main-menu.md` (Create Room, Join Room)

**Description**: When a button triggers an async server operation, its label is replaced with a spinner and the button is disabled. Communicates that the operation is in progress and prevents double-submission.

**Specification**:
- Trigger: Immediately on button click, before server responds
- Visual: Button text replaced by a rotating spinner icon (24px). Button background retains filled state. Button non-interactive.
- Duration: Spinner shows until server responds (success or error)
- On success: Transition to next state — spinner is not visible in the new state
- On error: Spinner removed, button re-enabled, error shown via PTN-FDB-002
- Timeout: If no server response after 10s, re-enable button and show "Connection error. Try again."

**When to Use**: Any primary button that triggers a server round-trip before the result is known.

**When NOT to Use**: Instant local actions (navigation, toggles). Optimistic UI patterns where the result is assumed successful.

**Reference**: `design/ux/main-menu.md` — Create Room loading state, Join Room loading state

---

### PTN-FDB-002 — Inline Error Message

**Category**: Feedback
**Used In**: `design/ux/main-menu.md` (RoomNotFound, SessionFull, SessionInProgress)

**Description**: An error message displayed inline below the element that caused the error. Always text-based — never color-only. The triggering element receives a brief red-flash visual at the moment of failure.

**Specification**:
- Position: Directly below the triggering element (input field or button), left-aligned
- Visual: 14px minimum text, Crimson-Amber `#C44B28` label. Error copy describes what happened and how to recover.
- Trigger animation: Red flash on the triggering element (150ms, fades to normal)
- Input re-enabled: Input re-enabled after error; value is NOT cleared
- Dismissal: Clears on the next input change or next submission attempt
- Never color-only: Error state requires a text label, not only a red color
- Accessibility: Error message is associated with its input element (aria-describedby pattern)

**When to Use**: Server validation errors, connection errors, invalid state transitions.

**When NOT to Use**: Warnings or informational messages that are not errors. Global app-level errors (use a toast or modal instead).

**Reference**: `design/ux/main-menu.md` — RoomNotFound, SessionFull, SessionInProgress error states

---

### PTN-FDB-003 — Animated Number Delta

**Category**: Feedback
**Used In**: `design/ux/hud.md` (gold counter, reserve mana numeral)

**Description**: When a numeric value changes, it animates from the old value to the new value rather than jumping. Count-up for gains, count-down for losses. Always communicates the direction of change.

**Specification**:
- Count-up (gain): Numeric value increments visually from old to new over max 400ms
- Count-down (loss): Numeric value decrements visually from old to new over max 400ms
- Large deltas: Accelerate proportionally — a +20 change should not take the full 400ms
- Phase resets (e.g., mana refill at DRAFT entry): Use count-up animation from previous value to new baseline
- Direction reinforcement (optional): Brief gold flash (+) on gain; brief blue flash (–) on loss
- Do not animate: State syncs after reconnect/disconnect (avoid ghost animations from server reconciliation)
- Open question: Rapid-succession delta behavior — see OQ-PAT-2

**When to Use**: Countable resources that change during gameplay where the delta communicates tactical information (gold earned, HP lost, mana spent).

**When NOT to Use**: Values set once (round number). Values updating > 3×/sec (use a static display or smooth meter instead).

**Reference**: `design/ux/hud.md` — Gold counter, reserve mana diamond

---

### PTN-FDB-004 — Hover Tooltip

**Category**: Feedback
**Used In**: `design/ux/hud.md` (interest threshold indicator on gold counter)

**Description**: A small informational overlay that appears on pointer hover. Used for supplementary information that is helpful but not required for core decisions. Never reveals information that should be persistently visible — if the player always needs it, make it a HUD element.

**Specification**:
- Trigger: Pointer enter event (mouse hover). Not visible by default.
- Appearance: Fades in 80ms. Positioned above or right of the element — never obscures the hovered element.
- Content: One to three lines of text. No interactive elements inside the tooltip.
- Dismissal: Fades out 80ms on pointer leave. Instantly dismissed if the element becomes non-interactive.
- Minimum display time: 1.5s if pointer moves away quickly — never flickers
- Accessibility: Same tooltip content accessible on keyboard focus (not hover-only)

**When to Use**: Definitions, secondary metrics, contextual hints for information already visible in reduced form.

**When NOT to Use**: Critical gameplay information that must always be visible. Interactive actions (tooltips are read-only).

**Reference**: `design/ux/hud.md` — Interest threshold tooltip on gold counter

---

### PTN-DSP-001 — Resource Counter

**Category**: Data Display
**Used In**: `design/ux/hud.md` (own gold, opponent gold)

**Description**: A large numeral representing a countable resource, accompanied by an icon that identifies the resource without relying on color or position alone. Always visible; never occluded.

**Specification**:
- Numeral: Heavy/bold weight. Min 40px for high-stakes counters (gold, auction price); min 20px for secondary resource counters.
- Icon: Resource-specific icon to the left of the numeral. Provides non-color identification.
- Color: Resource-type specific per art bible §4 semantic vocabulary (gold = Arcane Gold `#F5C842`)
- Updates: Animated via PTN-FDB-003 on change
- Layout: Horizontal — icon left, numeral right. Fixed-width numeral zone so value changes do not cause layout reflow.
- Opponent variant: Same size and weight as own counter — equal visual importance

**When to Use**: Countable resources with tactical significance that must be readable at a glance under time pressure.

**When NOT to Use**: Non-numeric states (use PTN-DSP-006). Unbounded values with no meaningful maximum visible to the player.

**Reference**: `design/ux/hud.md` — Own gold counter, opponent gold counter

---

### PTN-DSP-002 — Segmented Resource Bar

**Category**: Data Display
**Used In**: `design/ux/hud.md` (current mana bar)

**Description**: A horizontal bar divided into discrete segments, each representing one unit of a capped resource. Filled segments = available; empty segments = spent. Communicates both current value and maximum simultaneously through visual segment count.

**Specification**:
- Segments: Each segment = 1 unit. Filled = teal `#2AA8C4`. Empty = Ink Blue `#1E2E40`. Void `#1A1520` gap between segments.
- Cap label: Numeric label above bar showing current maximum (e.g., `10`). Updates with animation when cap changes.
- Drain: Segments empty instantly on spend — no animation (spend is an action, not an event)
- Refill: Segments fill via count-up animation left-to-right on phase reset/ramp
- Cap change: New segment slides in from right with gold flash (200ms)
- Accessibility: Segment count = numeric value label above or beside bar. Shape independent of color.

**When to Use**: Capped, discrete resources where the player needs current value and maximum simultaneously (mana, action points, charges).

**When NOT to Use**: Continuous/analog values (use a smooth fill bar). Uncapped resources (use PTN-DSP-003 or PTN-DSP-001).

**Reference**: `design/ux/hud.md` — Current mana bar

---

### PTN-DSP-003 — Diamond Resource Display

**Category**: Data Display
**Used In**: `design/ux/hud.md` (reserve mana)

**Description**: A diamond-shaped container with a numeric value inside. Used to distinguish a resource from a segmented bar (PTN-DSP-002) using shape rather than color alone. The diamond shape communicates "stored / persistent / different from the active resource."

**Specification**:
- Shape: Diamond (rotated square), blue gradient fill with Prism White inner glow
- Numeral: Heavy white text centered inside the diamond
- Size: Comparable to adjacent mana bar height — rendered in the same UI cluster
- Updates — gain: Pulse animation (scale 115%, return over 150ms)
- Updates — spend: Numeral counts down via PTN-FDB-003; no scale change (spend is a cost, not a reward)
- Persist indicator: Small loop/cycle glyph below diamond — confirms "carries to next round"
- No max indicator: Resource is uncapped; do not add a cap label
- Accessibility: Shape alone differentiates from the bar. Color is supplementary.

**When to Use**: Uncapped carry-over resources that must be visually distinct from the main capped resource in the same cluster.

**When NOT to Use**: Capped resources (use PTN-DSP-002). More than one diamond display in the same visual cluster (shape collision).

**Reference**: `design/ux/hud.md` — Reserve mana diamond

---

### PTN-DSP-004 — Phase Label

**Category**: Data Display
**Used In**: `design/ux/hud.md` (phase name + round number)

**Description**: A persistent, always-visible text label showing the current game phase and round number. Never relies on animation alone to communicate phase changes — the text is always present at some opacity.

**Specification**:
- Content: Phase name (e.g., `PLACEMENT`) + round number (e.g., `Round 4`)
- Hierarchy: Phase name larger/heavier than round number
- Color: Ivory `#F7F0DC`, regular weight, centered
- Transition: On phase change — old label crossfades to new label (80ms out, 80ms in simultaneously). Never fully hidden during transition.
- Always present: Minimum 40% opacity even during phase-specific HUD dimming (e.g., RESOLUTION)
- Accessibility: Text label always present. Phase change is never communicated by animation alone.

**When to Use**: Any game with distinct named phases where the player must know their current phase at all times.

**When NOT to Use**: Phases communicated solely by world context with no explicit label needed.

**Reference**: `design/ux/hud.md` — Phase label + round number

---

### PTN-DSP-005 — Urgency Countdown Timer

**Category**: Data Display
**Used In**: `design/ux/hud.md` (phase timer), `design/ux/main-menu.md` (lobby timer)

**Description**: A countdown timer that communicates urgency through a color ramp as the deadline approaches. Both a numeric text value and a visual indicator are always present — never color-only.

**Specification**:
- Numeral: Heavy weight. Always visible as seconds remaining.
- Color ramp:
  - > 15s remaining: Ivory `#F7F0DC` (calm)
  - 6–15s remaining: Auction Amber `#D9A940` (attention)
  - ≤ 5s remaining: Crimson-Amber `#C44B28` (urgent)
- Background: Semi-opaque panel behind the numeral — never rendered directly over animated board content
- Stop behavior: Shows `0` when deadline passes or all players complete early; does not hide
- Absence: Hidden entirely when no active countdown (e.g., RESOLUTION phase)
- Accessibility: Color ramp is supplementary to the numeric value. Reduced-motion: remove pulse/scale animations; keep color shift.
- PLACEMENT multiplier: Duration multiplied by accessibility setting (×0.5–×3). Color ramp stays proportional to remaining time fraction, not absolute seconds.

**Lobby variant** (`design/ux/main-menu.md`): Adds a horizontal progress bar below the numeral. Bar depletes left-to-right. Same color ramp. Both bar and numeral always present.

**When to Use**: Any timed player deadline where urgency escalation helps the player anticipate and manage pressure.

**When NOT to Use**: Informational countdowns with no player action required.

**Reference**: `design/ux/hud.md` — Phase timer; `design/ux/main-menu.md` — Lobby timer with progress bar

---

### PTN-DSP-006 — Status Dot

**Category**: Data Display
**Used In**: `design/ux/hud.md` (objective dots × 10)

**Description**: A small circular indicator representing the state of a discrete game entity. State changes are communicated by both color and shape — never color alone. Permanent destruction is shown through a shape transformation, not only a color change.

**Specification**:
- Size: 12px circle at rest
- States:
  - Active (real, own): Arcane Gold fill, solid
  - Active (fake, own): Ivory fill with subtle `?` texture — visible to owner only
  - Active (opponent view): Neutral stone fill — real and fake are pixel-identical from the opponent's perspective
  - HP loss: Scale pulse 130% → 100%, 150ms, per HP loss event
  - Destroyed: Cracks, dims to dark stone, shrinks to 80% — permanently inert shape change
- Symmetry rule: Real and fake dots must be visually and behaviorally identical before destruction. No size, animation, or pulse differences. (Art bible §9.3 prohibition.)
- Accessibility: Destruction state communicated by shape change (cracked/shrunken), not only color change.

**When to Use**: A small discrete set (≤ 10) of game entities with an alive/damaged/destroyed lifecycle.

**When NOT to Use**: More than ~10 dots in a single cluster (readability degrades). Entities with complex multi-step states requiring richer iconography.

**Reference**: `design/ux/hud.md` — Objective dots

---

### PTN-DSP-007 — Class Figurine Display

**Category**: Data Display
**Used In**: `design/ux/hud.md` (own + opponent class figurines), `design/ux/main-menu.md` (lobby class selection)

**Description**: A class character portrait with an associated HP pedestal or status label. Represents a player's chosen class identity alongside their current health. In the lobby it is the item being selected; in-game it is a persistent status indicator.

**Specification**:
- Art: Class figurine illustration per art bible §5. ~80–120px height in-game.
- HP display (in-game): Large numeral on a pedestal below the figurine. Flinch animation (brief shake) on HP loss.
- Idle animation: Slow looping class-specific idle (art bible §5.4). Must not distract from board activity.
- In-game position: Own figurine top-left, opponent top-right — flanks board like corner anchors
- Lobby — selecting state: Large browseable portrait with class name + brief description. Browse arrows active.
- Lobby — confirmed state: Checkmark overlay on slot; browse arrows hidden; slot locked (see PTN-OVR-001)
- Lobby — opponent before reveal: "Ready" text shown; class hidden until `S2CClassesRevealed`
- Reveal animation (`S2CClassesRevealed`): Both portraits animate in simultaneously — own slides from left, opponent from right, same-frame arrival

**When to Use**: Representing player identity + current health as a persistent in-game presence.

**When NOT to Use**: Contexts where the figurine would obstruct gameplay-critical board cells.

**Reference**: `design/ux/hud.md` — Class figurines; `design/ux/main-menu.md` — Lobby class select + opponent slot

---

### PTN-DSP-008 — Horizontal Card Row

**Category**: Data Display
**Used In**: `design/ux/hud.md` (hand card tray)

**Description**: A horizontally laid-out row of playable cards representing the player's current hand. Cards scale up on hover for full detail. Selected cards receive a distinct gold highlight. No ghost frames for empty slots.

**Specification**:
- Default scale: 75% card size
- Hover: Scale to 100%, lift 12px, full card detail visible. Transition: 80ms ease-out.
- Selected: Arcane Gold outline pulse, lifts to 100% scale
- Layout: Left-anchored, horizontal. At max hand size (10): 90% spacing minimum (slight overlap). No empty slot placeholders.
- Phase dimming: Cards dim to 50% during RESOLUTION (board takes focus). Full opacity all other phases.
- Input: Mouse click to select. Keyboard: Tab reaches first card; arrow keys navigate within row (see OQ-PAT-4).

**When to Use**: A player's hand of playable items in a card game session.

**When NOT to Use**: Non-card item collections. Collections larger than ~15 (overflow handling complex; consider a scrollable grid).

**Reference**: `design/ux/hud.md` — Hand card row

---

### PTN-OVR-001 — Player Slot

**Category**: Overlay / Modal
**Used In**: `design/ux/main-menu.md` (own slot + opponent slot in lobby)

**Description**: A bordered card panel representing one player's presence and readiness in a lobby or waiting room. Transitions through states: empty → connected → browsing → confirmed. Own and opponent slots are symmetric in size and layout.

**Specification**:
- Container: Bordered card, fixed size, vertically centered content
- States:
  - Empty: "Waiting for opponent…" label with subtle pulse animation on the slot border
  - Connected: Opponent indicator shown; class concealed
  - Browsing (own): Class figurine + name shown; Confirm button (PTN-NAV-001) active
  - Confirmed (own): Checkmark overlay on slot; browse arrows hidden; slot locked
  - Confirmed (opponent): "Ready" text only — class still hidden until `S2CClassesRevealed`
  - Revealed: Both class portraits animate in simultaneously on `S2CClassesRevealed` (see PTN-DSP-007 reveal animation)
- Symmetry: Own and opponent slots identical in size and shape — no visual hierarchy between the two players
- Accessibility: All slot state transitions communicated by text label in addition to visual change

**When to Use**: Two-player lobby flows requiring both readiness states visible simultaneously.

**When NOT to Use**: Single-player flows. 3+ players (extend with an N-slot variant; this pattern is sized for 2).

**Reference**: `design/ux/main-menu.md` — Own player slot, opponent slot

---

## Gaps & Patterns Needed

Anticipated patterns for planned systems with no spec yet. Document when the relevant screen is authored.

| Gap | Anticipated by | Notes |
|---|---|---|
| Auction Bid Input | DRAFT_AUCTION HUD (future spec) | Numeric input with increment/decrement controls; confirmation step required (accessibility §motor — PTN-FDB-001 covers the submit spinner) |
| Shop Item Card | DRAFT_SHOP HUD (future spec) | Purchasable card with cost overlay; disabled/unaffordable state |
| Board Cell + Spawn Highlight | PLACEMENT HUD (future spec) | Interactive grid cell; spawn-range highlight state; fog-of-war state for opponent half |
| Movement Arrow | PLACEMENT HUD (future spec) | Per-cell directional arrow overlay showing committed unit movement |
| Confirmation Modal | Lobby cancel (OQ-MM-3), Settings | Two-action overlay — confirm / cancel. Needed for "Leave room?" when opponent is connected. |
| Notification Toast | Post-auction result, round summary | Non-blocking transient message that auto-dismisses after ~2s |
| Score / Result Panel | GAME_OVER screen (not yet designed) | Win/loss display with fake objective reveals |

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-PAT-1 | PTN-INP-002 Carousel Browser — does it wrap (index 0 after last class) or stop at boundaries? Not specified in main-menu.md. | ux-designer | Low |
| OQ-PAT-2 | PTN-FDB-003 Animated Number Delta — behavior when multiple deltas fire in rapid succession (e.g., multi-unit RESOLUTION). Queue and sequence? Merge into single jump? | ux-designer | Medium |
| OQ-PAT-3 | PTN-OVR-001 Player Slot — the simultaneous reveal animation needs a detailed timing spec (stagger? exact duration per portrait?). Flag for the GAME_OVER screen spec. | ux-designer | Low |
| OQ-PAT-4 | PTN-DSP-008 Horizontal Card Row — keyboard navigation: left/right arrow keys to navigate cards, or Tab only? Arrow keys are more natural but may conflict with board cell navigation during PLACEMENT. | ux-designer | Medium |
