# UX Spec: Main Menu & Lobby Flow

> **Status**: Complete — pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-04-29
> **Journey Phase(s)**: Game entry → Matchmaking → Pre-game lobby
> **Template**: UX Spec
> **Input Methods**: Mouse + Keyboard (primary: mouse click). No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard

---

## Purpose & Player Need

This screen serves three consecutive player needs in sequence:

1. **Land and orient** — player arrives at the game and needs to understand what to do within 10 seconds (create a room or join one)
2. **Connect with an opponent** — player needs to share a room code or enter one to find a match
3. **Commit to a class** — player selects and confirms their class before the information war begins

The player arrives wanting to play. The screen must remove every obstacle between them and DRAFT_INITIAL.

**What goes wrong if this is hard to use:** A player who cannot figure out how to share a room code with their friend abandons before the first game. A player who accidentally confirms the wrong class and cannot change it feels cheated before a card is played.

---

## Player Context on Arrival

**First visit:** Player has just loaded the WASM app in their browser. They have zero context. The game title and two clear actions (Create / Join) must tell them everything they need to know within 5 seconds.

**Returning player:** Arrives with intent — they already know what they want to do. The last-used action (Create or Join) should be visually prominent but not the only option.

**Emotional state:** Anticipation and mild uncertainty. They're about to enter an information war — the lobby experience should hint at this without being opaque. The opponent's class reveal (at the end of this flow) is the first moment of the information game: design the lobby to make that reveal feel significant.

**Arrival context:** Player arrives voluntarily by loading the URL. No prior game state carried over (stateless browser session).

---

## Navigation Position

```
[Browser URL load]
        ↓
  TITLE SCREEN
  ├── Create Room ──► LOBBY (own code, class select, ready)
  │                       └──► [Game starts] ──► IN-GAME HUD
  ├── Join Room ────► LOBBY (entered code, class select, ready)
  │                       └──► [Game starts] ──► IN-GAME HUD
  ├── Settings ──────► SETTINGS SCREEN (separate spec)
  └── How to Play ──► RULES OVERLAY (modal over title)
```

The title screen is the root of all navigation. The lobby is a direct child accessible only after Create or Join. There is no back-navigation from the lobby to the title screen — exiting the lobby cancels the session and returns to title.

---

## Entry & Exit Points

**Entry:**

| Entry Source | Trigger | Player carries this context |
|---|---|---|
| Browser URL load | App startup | No prior state |
| Lobby cancelled (own disconnect or timeout) | Server sends `S2CSessionCancelled` | Returns with no carried state; title resets to default |
| GAME_OVER (future: post-match return) | [Not yet designed] | Would carry match result — flag as OQ |

**Exit:**

| Exit Destination | Trigger | Notes |
|---|---|---|
| IN-GAME HUD | `SessionReady` fires; RSM enters DRAFT_INITIAL | One-way — cannot return to lobby during game |
| Settings screen | Settings button click | Overlays or replaces; returns to title on close |
| Rules overlay | How to Play click | Modal overlay; closes back to title |
| Lobby cancelled → title | Own disconnect in lobby, opponent disconnect, lobby timeout | Session destroyed; player returns to title screen |

---

## Layout Specification

### Information Hierarchy

**Title screen — priority order:**
1. Game title "Lanes and Lies" — identity anchor; first read
2. Create Room button — primary action (most players will create)
3. Join Room (input + button) — secondary action
4. Settings — tertiary, small, corner
5. How to Play — quaternary, discoverable

**Lobby screen — priority order:**
1. Room code (large, copyable) — the thing you need to share/enter
2. Opponent slot status (empty / connected / class locked)
3. Own class selection area
4. Ready/Confirm button
5. Cancel / leave room

### Layout Zones

**Title Screen:**

```
┌──────────────────────────────────────────────────┐
│                                                  │
│          LANES AND LIES                          │
│          [subtitle / tagline]                    │
│                                                  │
│          [ CREATE ROOM ]                         │
│                                                  │
│          [ ______ ]  [ JOIN ROOM ]               │
│          room code input                         │
│                                                  │
│                                                  │
│  [?] How to Play              [⚙] Settings      │
└──────────────────────────────────────────────────┘
```

**Lobby Screen (after Create or Join):**

```
┌──────────────────────────────────────────────────┐
│  ← Cancel                                        │
│                                                  │
│  ROOM: [ X8K2PQ ]  [📋 Copy]                    │
│                                                  │
│  ┌──────────────┐    ┌──────────────┐            │
│  │   YOU        │    │   OPPONENT   │            │
│  │  [Class art] │    │  [Waiting…]  │            │
│  │  Xelor       │    │  ?           │            │
│  │  ✓ Confirmed │    │  (empty)     │            │
│  └──────────────┘    └──────────────┘            │
│                                                  │
│  ◀ ▶  Browse Classes                             │
│                                                  │
│             [ CONFIRM CLASS ]                    │
│                                                  │
│  Lobby closes in 90s  ████████░░  72s            │
└──────────────────────────────────────────────────┘
```

### Component Inventory

**Title Screen:**

| Component | Type | Content | Interactive |
|---|---|---|---|
| Game title | Display text | "Lanes and Lies" | No |
| Create Room button | Primary button | "Create Room" | Yes — sends `C2SCreateRoom` |
| Room code input | Text input | Placeholder: "Room code" | Yes — accepts 6-char input |
| Join Room button | Secondary button | "Join Room" | Yes — sends `C2SJoinRoom` after input |
| How to Play | Text link / icon button | "?" or "How to Play" | Yes — opens rules overlay |
| Settings | Icon button | ⚙ | Yes — opens settings |

**Lobby Screen:**

| Component | Type | Content | Interactive |
|---|---|---|---|
| Room code display | Large text + copy button | 6-char code (e.g. `X8K2PQ`) | Yes — copy to clipboard |
| Own player slot | Card panel | Class art, class name, status (browsing / confirmed) | No (status display) |
| Opponent slot | Card panel | "Waiting…" / connected / class locked state | No (status display) |
| Class browser | Carousel / prev-next | Class portrait + name + brief description | Yes — left/right arrows or click |
| Confirm Class button | Primary button | "Confirm Class" | Yes — sends `C2SConfirmClass` |
| Cancel button | Text/ghost button | "← Cancel" | Yes — leaves room, returns to title |
| Lobby timer | Progress bar + countdown | Time remaining until lobby timeout (90s default) | No |

---

## States & Variants

**Title Screen States:**

| State | Trigger | What Changes |
|---|---|---|
| Default | App load | All elements visible, no input populated |
| Join Room active | Player clicks Join Room input | Input field focused; Join button becomes primary |
| Joining (loading) | Player clicks Join Room | Brief spinner on Join button; disabled while server responds |
| Join error — room not found | Server returns `RoomNotFound` | Error message below input: "Room not found. Check the code." Input re-enabled. |
| Join error — room full | Server returns `SessionFull` | Error: "Room is full." |
| Join error — session in progress | Server returns `SessionInProgress` | Error: "Game already started." |
| Create Room loading | Player clicks Create Room | Brief spinner; button disabled |

**Lobby States:**

| State | Trigger | What Changes |
|---|---|---|
| Creator waiting | Room created; own slot filled | Opponent slot shows "Waiting for opponent…" with subtle pulse |
| Joiner connected | Opponent joins; `S2CSlotUpdated` | Opponent slot shows connected state (no class revealed yet) |
| Own class browsing | Default lobby state | Class carousel active |
| Own class confirmed | Own `C2SConfirmClass` sent | Own slot shows confirmed state; Confirm button shows checkmark; locked state |
| Opponent class confirmed | `S2CClassLocked` received | Opponent slot shows "Ready" (class still hidden) |
| Both classes confirmed | `S2CClassesRevealed` received | REVEAL MOMENT: both classes animate in simultaneously — see Transitions |
| Lobby cancelled — opponent left | `S2CSessionCancelled { reason: PlayerDisconnected }` | Overlay: "Opponent disconnected. Room closed." → returns to title |
| Lobby cancelled — timeout | `S2CSessionCancelled { reason: LobbyTimeout }` | Overlay: "Lobby timed out." → returns to title |
| Lobby cancelled — server RNG fail | `S2CSessionCancelled` | Overlay: "Session failed to start. Please try again." → returns to title |

---

## Interaction Map

*Input: Mouse click (primary). Keyboard Tab + Enter supported for all interactive elements.*

**Title Screen:**

| Element | Action | Input | Feedback | Outcome |
|---|---|---|---|---|
| Create Room button | Click / Enter | Mouse click or keyboard Enter | Button shows loading spinner; disabled | Sends `C2SCreateRoom { mode: OneVOne }`; transitions to Lobby on `S2CRoomCreated` |
| Room code input | Type | Keyboard | Characters appear; auto-uppercased; max 6 chars | Populates join code |
| Join Room button | Click / Enter | Mouse click or Enter | Disabled if input empty; spinner on click | Sends `C2SJoinRoom { room_code, requested_slot: 1 }`; transitions on `S2CJoinAck` |
| Settings button | Click | Mouse click | Highlight | Opens Settings screen |
| How to Play | Click | Mouse click | Highlight | Opens rules modal overlay |

**Lobby Screen:**

| Element | Action | Input | Feedback | Outcome |
|---|---|---|---|---|
| Copy room code | Click clipboard icon | Mouse click | Brief "Copied!" tooltip (1.5s) | Room code copied to clipboard |
| Class prev/next arrows | Click | Mouse click | Class portrait animates in; name updates | Previews next/previous class |
| Confirm Class button | Click | Mouse click | Button shows checkmark; own slot locks | Sends `C2SConfirmClass { class_id }`; own slot shows confirmed state |
| Cancel button | Click | Mouse click | Brief confirm if opponent is connected ("Leave room?") | Disconnects from session; returns to title screen |

---

## Events Fired

| Player Action | Server Message / Event | Payload |
|---|---|---|
| Click Create Room | `C2SCreateRoom` | `{ mode: OneVOne }` |
| Click Join Room | `C2SJoinRoom` | `{ room_code: String, requested_slot: 1 }` |
| Browse class (preview only) | `C2SSelectClass` | `{ class_id }` — optional server-side; no UI commitment |
| Confirm class | `C2SConfirmClass` | `{ class_id }` |
| Click Cancel in lobby | *(transport disconnect or server-side cleanup)* | — |

*Note: `C2SSelectClass` is optional preview — the server does not require it before `C2SConfirmClass`. Client may send it for opponent's preview status if that feature is added, but it carries no commitment.*

---

## Transitions & Animations

**Title → Lobby (Create Room):** Cross-fade 200ms. Room code fades in from center. Class browser slides up from bottom.

**Title → Lobby (Join Room):** Same as Create Room transition.

**Class reveal moment** (`S2CClassesRevealed` received — both players confirmed):
This is the first high-stakes moment of the information war. Both class portraits animate in simultaneously:
- Own class: slides in from left, desaturate-to-resaturate flash (art bible §2 LOBBY mood)
- Opponent class: slides in from right, same flash
- Both panels reach final position at the same frame — simultaneous reveal
- Hold for 1.5s, then transition to game start loading

**Lobby → Game start:** Board expands to fill screen (200ms scale from center); Lobby fades out.

**Error states:** Red flash on the relevant input/button (150ms), then error text fades in below.

**Lobby cancellation overlay:** Semi-opaque dark overlay fades in (150ms), cancel message fades in, "Return to Menu" button appears. No animation on auto-return — user controls the timing.

---

## Data Requirements

| Data | Source | Read/Write | Notes |
|---|---|---|---|
| Room code | Server (`S2CRoomCreated`) | Read | Display and copy — never editable by client |
| Room code input | Local client state | Write → `C2SJoinRoom` | Validated: 6 chars, alphanumeric, before sending |
| Class list | `SessionConfig` / game data | Read | Static list of available classes; loaded at startup |
| Own class selection | Local client state | Write → `C2SConfirmClass` | Preview state (`C2SSelectClass`) is client-only until confirmed |
| Opponent slot status | `S2CSlotUpdated`, `S2CClassLocked`, `S2CClassesRevealed` | Read | Never shows opponent class until `S2CClassesRevealed` |
| Lobby deadline timer | Local client countdown | Read | `lobby_timeout_seconds` from GameConfig (default 90s); client counts down from connection time |
| Session cancellation reason | `S2CSessionCancelled.reason` | Read | Drives error message copy |

---

## Accessibility

*Standard tier. Source: `design/accessibility-requirements.md`.*

| Requirement | How addressed |
|---|---|
| Keyboard navigation | All interactive elements reachable via Tab. Enter activates buttons. No mouse-only interactions. |
| Error messages | Text errors displayed below the triggering element, not color-only. |
| Room code copy | Clipboard copy is supplementary — room code text is always selectable/visible. |
| Class browser | Arrow key navigation supported alongside click. Class name is always text (not image-only). |
| Lobby cancellation messages | Error text explains what happened. Not silent dismissal. |
| Text contrast | All button labels and status text ≥ 4.5:1 on background. |
| Focus indicators | All interactive elements have visible focus ring (2px Prism White outline). |
| "Copied!" tooltip | 1.5s display minimum — meets minimum reading time; no interaction required to dismiss. |
| Lobby timer | Timer is both visual (progress bar) and text (countdown seconds) — not color-only. |
| Class reveal animation | Simultaneous reveal animation can be replaced with a cut in reduced-motion mode. |

---

## Localization Considerations

| Element | Max length risk | Notes |
|---|---|---|
| "Create Room" button | Medium | German/French ~40% longer: "Créer une partie" / "Raum erstellen" — button must expand or truncate gracefully |
| "Join Room" button | Medium | Same concern |
| "Confirm Class" button | Medium | French "Confirmer la classe" is long |
| Error messages | High | Error copy can be verbose in some languages; allow 2-line wrap in error display zones |
| Class names | Low | Dofus/Wakfu class names are French-origin and consistent across localizations |
| Room code | N/A | Always 6-char alphanumeric; not localized |

---

## Acceptance Criteria

- [ ] Title screen loads within 500ms of WASM app startup (excluding initial bundle download)
- [ ] Create Room button sends `C2SCreateRoom` and displays a 6-character room code on `S2CRoomCreated`
- [ ] Room code is copyable to clipboard via the copy button; "Copied!" tooltip appears and disappears after 1.5s
- [ ] Join Room button is disabled when the room code input is empty; enabled once input has ≥1 character
- [ ] Invalid room code (RoomNotFound) displays an error message below the input field without clearing the input
- [ ] Class browser allows cycling through all available classes; class name and portrait update on each step
- [ ] Confirm Class button is disabled until a class is selected; sends `C2SConfirmClass` on click
- [ ] Own slot shows "confirmed" state after `C2SConfirmClass` is sent; Confirm button becomes non-interactive
- [ ] Opponent slot shows "Waiting…" state when empty and updates on `S2CSlotUpdated`
- [ ] Opponent class is NOT revealed until `S2CClassesRevealed` is received — opponent slot shows "Ready" not the class
- [ ] Both class portraits animate in simultaneously on `S2CClassesRevealed` (no sequential reveal)
- [ ] `S2CSessionCancelled` displays a human-readable cancellation reason and a "Return to Menu" button
- [ ] Cancel button in lobby returns player to title screen (session destroyed)
- [ ] All interactive elements on both screens are reachable via keyboard Tab navigation
- [ ] All text elements meet 4.5:1 contrast ratio against their backgrounds

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-MM-1 | Should the class browser show a brief class description/ability hint per class to help new players choose? Or just name + portrait? | game-designer | Medium |
| OQ-MM-2 | Does the lobby show the opponent's class icon/color before they confirm (preview state), or only show "connected" with no class hint? GSS Rule 6 says `C2SSelectClass` is optional and client-only — only `C2SConfirmClass` is committed. | ux-designer | High |
| OQ-MM-3 | What is the cancellation UX when the creator leaves: instant return to title, or a short "Room closed" hold? The joiner needs to know the room is gone. | ux-designer | Medium |
| OQ-MM-4 | Post-game: does the player return to the title screen after GAME_OVER, or is there a post-match screen with result + rematch option? Not designed yet. | game-designer | Low |
| OQ-MM-5 | Should room codes be case-insensitive on input? (e.g., `x8k2pq` and `X8K2PQ` both work) The server generates uppercase — auto-uppercase the input client-side. | lead-programmer | Low |
