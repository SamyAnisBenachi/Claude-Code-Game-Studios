# UX Spec: Result Screen / Game Over

> **Status**: Complete - pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-05-05
> **Journey Phase(s)**: GAME_OVER / post-match
> **Template**: UX Spec
> **Input Methods**: Mouse + Keyboard (primary: mouse click). No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (`design/accessibility-requirements.md`)
> **Source GDDs**: `design/gdd/round-state-machine.md`, `design/gdd/objective-system.md`, `design/gdd/hud.md`, `design/gdd/network-protocol.md`
> **Related UX Specs**: `design/ux/main-menu.md`, `design/ux/hud.md`, `design/ux/interaction-patterns.md`

---

## Purpose & Player Need

The result screen is the terminal match readout shown after the Round State Machine enters `GAME_OVER`. It serves four player needs:

1. Understand the outcome immediately: win, loss, draw, or resolution failure.
2. See why the match ended: objective loss, disconnection, mutual destruction, or safety timeout.
3. Resolve the hidden-information promise by revealing the objective map and summarising real/fake outcomes.
4. Choose the next action: request a rematch or leave the ended session.

The player arrives wanting closure first and action second. The first read must answer "Did I win?" The second read must answer "What was true behind the bluff?" The third read can show resources and tactical stats that help the player process the match.

If this screen is hard to use, the game fails to pay off its deception mechanic. Players may understand that the match ended but miss the real/fake reveal, misread a draw as a loss, or leave without knowing whether a rematch was offered.

---

## Player Context on Arrival

Players arrive automatically; this screen is never opened voluntarily from a menu. The immediate prior context is usually the final RESOLUTION, after one or both players cross the loss threshold of two destroyed real objectives. Other arrivals include disconnect-triggered GAME_OVER and resolution safety timeout.

Expected emotional states:

| Result | Player State | Design Response |
|---|---|---|
| Win | Relief, confirmation, curiosity about the hidden objective map | Warm result headline, then objective reveal summary |
| Loss | Frustration, need for fairness and clarity | Direct cause text, no taunting, clear objective facts |
| Draw | Ambiguity, need to understand simultaneity | Neutral headline and explicit "both players met the condition" copy |
| Disconnection | Possible annoyance or confusion | Plain connection reason; do not style as tactical victory if the cause was network loss |
| Resolution timeout | Trust risk | Separate "Result unavailable" treatment; communicate that no winner was declared |

The HUD remains visible beneath the result overlay in FROZEN mode. It is a final-state record, not the result surface. The result screen must not mutate HUD state or add real/fake glyphs to HUD dots.

No `design/player-journey.md` exists yet, so this spec assumes the post-match journey phase from the RSM, Objective System, HUD, and main-menu specs.

---

## Navigation Position

```text
IN-GAME HUD / BOARD
  -> RESOLUTION
      -> GAME_OVER phase
          -> RESULT SCREEN overlay
              -> Rematch requested flow
              -> Return to Lobby / Menu
```

This screen is a terminal overlay above the frozen in-game scene. It is not part of the main-menu stack until the player chooses an exit action.

`design/ux/main-menu.md` OQ-MM-4 is resolved by this spec: GAME_OVER routes first to the result screen. Exiting the result screen returns to the main-menu/lobby flow, not directly into a new match.

---

## Entry & Exit Points

### Entry

| Entry Source | Trigger | Player Carries This Context |
|---|---|---|
| Final RESOLUTION | `S2CGameOver { loser: Some(player), reason: ObjectivesDestroyed }` plus `S2CPhaseChanged(GAME_OVER)` | Final round, loser, reason, frozen HUD state, objective snapshots |
| Mutual objective destruction | `S2CGameOver { loser: None, reason: Draw }` | Both players met loss condition in the same RESOLUTION |
| Disconnect loss | `S2CGameOver { loser: Some(player), reason: Disconnection }` | One player exceeded disconnect grace |
| Mutual disconnection | `S2CGameOver { loser: None, reason: Draw }` | Both players disconnected beyond grace |
| Resolution safety timeout | `S2CGameOver { loser: None, reason: ResolutionTimeout }` | Resolution did not complete within safety timeout |
| Reconnect at GAME_OVER | `S2CGameSnapshot.phase == GAME_OVER` | Final board/HUD state can rebuild, but result payload may be missing unless `S2CGameOver` is re-sent or included in snapshot |

### Exit

| Exit Destination | Trigger | Notes |
|---|---|---|
| Rematch pending | Player activates Rematch | Requires protocol support not currently defined; see Open Questions |
| Main menu / lobby flow | Player activates Return to Lobby | Sends or has already sent `C2SAcknowledgeResult`; clears local ended-session UI state |
| Browser/session close | Player closes tab | Server cleanup follows `ack_timeout_ms` if no result acknowledgement is received |

---

## Layout Specification

### Information Hierarchy

1. Result headline: WIN, LOSS, DRAW, or NO RESULT.
2. Cause line: objective destruction, opponent disconnect, own disconnect, mutual destruction, mutual disconnect, or resolution timeout.
3. Objective reveal summary: both players' five objective lanes, real/fake identity, destroyed/alive state, final HP.
4. Final score/resources summary: final round, real objectives destroyed, fake objectives destroyed, final gold, current mana/mana cap, reserve mana, hand size if available.
5. Actions: Rematch, Return to Lobby.
6. Secondary system messages: waiting for opponent rematch, disconnect at result screen, result-data fallback.

The objective reveal summary is the main content after the headline. It must not be hidden behind tabs by default because it is the payoff for the bluff system.

### Layout Zones

The overlay sits above the frozen board and HUD. It may dim the board but must leave enough of the final-state HUD visible to reinforce continuity.

```text
+----------------------------------------------------------------+
| [frozen HUD remains visible behind overlay]                     |
|                                                                |
|             RESULT HEADLINE                                    |
|             Cause sentence / final round                       |
|                                                                |
|   YOUR OBJECTIVES                     OPPONENT OBJECTIVES       |
|   L1 REAL  destroyed  HP 0            L1 FAKE  alive      HP 2 |
|   L2 FAKE  destroyed  HP 0            L2 REAL  destroyed  HP 0 |
|   L3 REAL  alive      HP 3            L3 REAL  alive      HP 1 |
|   L4 FAKE  alive      HP 5            L4 FAKE  destroyed  HP 0 |
|   L5 REAL  destroyed  HP 0            L5 REAL  alive      HP 5 |
|                                                                |
|   FINAL SUMMARY                                                |
|   Round R12 | Real lost 2-1 | Fakes found 1-1 | Gold 8-5       |
|                                                                |
|                  [REMATCH]   [RETURN TO LOBBY]                 |
+----------------------------------------------------------------+
```

### Component Inventory

| Component | Type | Content | Interactive | Pattern |
|---|---|---|---|---|
| Result overlay root | Overlay panel | Dims board, contains all result elements | No | New: Result Overlay |
| Result headline | Display text | `VICTORY`, `DEFEAT`, `DRAW`, `NO RESULT` | No | PTN-DSP-004 variant |
| Cause line | Display text | Human-readable `GameOverReason` and final round | No | Inline status text |
| Objective reveal grid | Data table / lane rows | 5 lanes per player; real/fake, alive/destroyed, final HP | No | New: Objective Reveal Summary |
| Lane reveal row | Status row | Lane number, identity, destroyed/alive, HP, reward marker if fake destroyed | No | PTN-DSP-006 expanded |
| Final resources summary | Compact stat strip | Final round, gold, mana, reserve, real/fake counts | No | PTN-DSP-001 / PTN-DSP-002 variants |
| Rematch button | Primary action button | `Rematch` / waiting state | Yes when protocol-supported | PTN-NAV-001 |
| Return to Lobby button | Secondary action button | `Return to Lobby` | Yes | PTN-NAV-001 secondary variant |
| Rematch status text | Status label | Waiting, accepted, unavailable, opponent left | No | Inline status text |
| Result data fallback | Inline warning | Missing result payload or reveal data | No | PTN-FDB-002 style without input |

New patterns to add to `design/ux/interaction-patterns.md` later:

- Result Overlay
- Objective Reveal Summary
- Rematch Pending State

---

## States & Variants

| State / Variant | Trigger | What Changes |
|---|---|---|
| Victory - objectives | `loser == opponent`, `reason == ObjectivesDestroyed` | Headline `VICTORY`; cause says opponent lost two real objectives |
| Defeat - objectives | `loser == local`, `reason == ObjectivesDestroyed` | Headline `DEFEAT`; cause says two of your real objectives were destroyed |
| Draw - mutual objectives | `loser == None`, `reason == Draw`, objective counters show both players crossed threshold | Headline `DRAW`; cause says both players lost real objectives in the same resolution |
| Victory - disconnection | `loser == opponent`, `reason == Disconnection` | Headline `VICTORY`; cause says opponent disconnected beyond grace |
| Defeat - disconnection | `loser == local`, `reason == Disconnection` | Headline `DEFEAT`; cause says your connection exceeded grace |
| Draw - mutual disconnect | `loser == None`, `reason == Draw`, no objective threshold explanation available | Headline `DRAW`; cause says both players disconnected |
| Resolution timeout | `reason == ResolutionTimeout` | Headline `NO RESULT`; cause says resolution timed out and no winner was declared |
| Result payload missing | `phase == GAME_OVER` but no cached or re-sent `S2CGameOver` | Show fallback headline `RESULT PENDING`; disable Rematch; Return to Lobby remains available |
| Reveal data partial | Objective identity or opponent `was_fake` reveal data missing | Show known lanes; unknown identity rows read `Unknown`; add inline warning |
| Rematch available | Both players connected and protocol supports rematch | Rematch button enabled |
| Rematch requested | Local player clicks Rematch | Button becomes `Waiting...`; Return remains available |
| Opponent requested rematch first | Inbound rematch request state | Rematch button becomes `Accept Rematch`; status text names opponent request |
| Rematch unavailable | Opponent disconnected, protocol unsupported, or session already acknowledged/cleaned | Rematch disabled or hidden; status text explains why |
| Reduced motion | Accessibility setting enabled | All reveal sequencing becomes instant or simple fade; no iris wipe, scale, flash, or card travel |

---

## Interaction Map

Input scope: Mouse click primary. Keyboard Tab and Enter must reach both action buttons. Escape focuses `Return to Lobby` but does not auto-exit, because leaving the result screen is a session action.

| Element | Action | Input | Immediate Feedback | Outcome |
|---|---|---|---|---|
| Rematch button | Request or accept rematch | Click or Enter | Button changes to waiting/accepted state | Sends rematch request if protocol exists; otherwise no send |
| Return to Lobby button | Leave result screen | Click or Enter | Button pressed state, then route transition | Sends `C2SAcknowledgeResult` if not already sent; clears ended-session UI; returns to main-menu/lobby flow |
| Objective reveal rows | Inspect | Keyboard focus optional; mouse hover optional | Row highlight only; no tooltip required | No state change |
| Result overlay | Initial render | None | Overlay receives focus on first interactive action | After first stable render, client may send `C2SAcknowledgeResult` if server cleanup requires it |

Keyboard focus order:

1. Result overlay heading, programmatically focused for screen-reader orientation if available.
2. Rematch button, if enabled or pending.
3. Return to Lobby button.
4. Optional objective rows only if the implementation supports row-level accessible inspection.

Disabled controls do not receive keyboard focus. If Rematch is unsupported, focus starts at Return to Lobby.

---

## Events Fired

| Player Action | Event / Message Fired | Payload / Data |
|---|---|---|
| Result screen fully rendered | `C2SAcknowledgeResult` | `{}`; exact timing must be reconciled with rematch support |
| Click Return to Lobby | `C2SAcknowledgeResult` if not already sent | `{}`; local route to main-menu/lobby flow |
| Click Rematch | TBD protocol gap | Proposed: `C2SRematchRequest {}` or equivalent |
| Accept opponent rematch request | TBD protocol gap | Proposed: same rematch request/accept message |
| Click disabled Rematch | None | Visual state only |
| Hover/focus objective row | None | Pure client-side inspection |

Persistent or server-authoritative state changes must not be committed optimistically. A rematch does not start until both clients receive a server-authoritative new lobby/session state.

---

## Transitions & Animations

### Entry

Standard motion:

- On `S2CPhaseChanged(GAME_OVER)`, HUD enters FROZEN mode immediately.
- Board settles for 300ms after final resolution effects if they are still draining.
- Overlay fades in over 150ms.
- Result headline appears first.
- Objective reveal grid appears after a 500ms minimum hold if the final objective destruction reveal has not already paid off that beat in board rendering.
- Both players' objective maps reveal simultaneously; lane rows can highlight left-to-right over 300ms total.

Reduced motion:

- Overlay appears via instant cut or 80ms fade.
- Objective rows appear all at once.
- No iris wipe, no overexposed bloom, no scale pulse, no repeated flashes.

### Result Tone

Art Bible alignment:

| Result | Visual Treatment |
|---|---|
| Victory | Warm gold fill light, saturated, but no celebratory HUD mutation |
| Defeat | Cool-blue grade, slightly desaturated, readable text contrast maintained |
| Draw | Neutral even light; no winner color language |
| Resolution timeout | Neutral warning treatment; do not use defeat palette |

Objective identity reveal follows the Art Bible GAME_OVER mood: conclusive, open, and unambiguous. The reveal may use an iris-open wipe in standard motion, but the reduced-motion mode replaces it with static identity labels.

### Exit

| Transition | Standard Motion | Reduced Motion |
|---|---|---|
| Return to Lobby | Overlay fades out 150ms; route to main-menu/lobby flow | Instant route or 80ms fade |
| Rematch request | Button label changes immediately; small status text appears | Same, no pulse |
| Rematch accepted | Route through server-authoritative new lobby/session transition | Same |

The result screen must never delay server cleanup or a new session for animation polish.

---

## Data Requirements

| Data | Source System | Read / Write | Notes |
|---|---|---|---|
| Game result | `S2CGameOver` | Read | `{ loser: Option<PlayerId>, round, reason }`; primary source for headline and cause |
| GAME_OVER phase | `S2CPhaseChanged` or `S2CGameSnapshot.phase` | Read | Opens overlay and freezes HUD |
| Local player id | Session/client state | Read | Determines win/loss from `loser` |
| Final round | `S2CGameOver.round` and/or snapshot `round_number` | Read | Display as `R<round>`; mismatch is a server bug to log |
| Objective map - own side | `S2CGameSnapshot.players[].objectives` plus `S2CObjectiveIdentities` cache | Read | Own real/fake identity available; destroyed/alive and HP from snapshot |
| Objective map - opponent side | `S2CGameSnapshot.players[].opponent_objectives` | Read | `was_fake: Some(bool)` only when destroyed; alive opponent identity remains hidden unless result payload expands reveal authority |
| Full post-game objective reveal | TBD data contract | Read | Required if GAME_OVER should reveal all still-alive opponent real/fake identities |
| Final gold/reserved gold | `S2CGameSnapshot.PlayerSnapshot` or frozen HUD state | Read | Summary only; do not recompute from deltas |
| Final mana/reserve/mana cap | `S2CGameSnapshot.PlayerSnapshot` or frozen HUD state | Read | Summary only |
| Final hand size | `S2CGameSnapshot.PlayerSnapshot.hand` for own player; opponent hand unavailable unless post-game stats add it | Read | Optional; omit opponent hand size if unavailable |
| Rematch availability | TBD protocol/session state | Read | Needed to enable/disable Rematch |
| Result acknowledgement | `C2SAcknowledgeResult` | Write | Existing protocol message; cleanup handshake only |
| Rematch request | TBD protocol | Write | Not currently defined in network protocol |
| Return route | Local UI router | Write | Clears result overlay and returns to main-menu/lobby flow |

Architectural constraints:

- The result screen may read frozen HUD state, but it must not mutate HUD entities.
- The result screen must not infer hidden opponent objective identities unless the server explicitly provides a post-game reveal payload.
- If the server intends full objective map reveal at GAME_OVER, the protocol needs either an expanded `S2CGameOver`, a `S2CPostGameSummary`, or a GAME_OVER-specific snapshot projection that includes all objective identities.

---

## Accessibility

Standard tier. Source: `design/accessibility-requirements.md`.

| Requirement | UX Requirement |
|---|---|
| Keyboard navigation | Rematch and Return to Lobby reachable by Tab and activated by Enter. Escape moves focus to Return to Lobby but does not trigger it. |
| Initial focus | On screen open, focus the result heading or first action according to implementation capability; do not leave focus on a hidden gameplay control. |
| Focus indicators | Buttons and any focusable objective rows use a 2px high-contrast focus ring. |
| Text contrast | Headline, cause, row labels, and action buttons meet at least 4.5:1 contrast. |
| Color-independent outcome | Result uses text labels and cause copy, not only warm/cool palette. |
| Color-independent objective identity | Objective rows use explicit `REAL` / `FAKE` text and distinct icons/shapes; color is supplementary. |
| Reduced motion | Objective reveal, overlay entry, bloom/flash, and row highlight sequencing have static or simple-fade alternatives. |
| Photosensitivity | No repeated flash above 3 flashes/sec. Any objective destruction burst or GAME_OVER bloom must be a single burst only. |
| Motor accessibility | No timed action on the result screen. Rematch does not expire visually without an explicit status message. |
| Screen reader future path | If Bevy/browser accessible roles become available, expose result headline, cause, each objective row, and both buttons as named semantic UI elements. |

---

## Localization Considerations

| Element | Risk | Requirement |
|---|---|---|
| Result headline | Low | Single-word labels have ample space |
| Cause line | High | Supports two-line wrap without pushing action buttons below minimum viewport |
| Objective row labels | Medium | Use compact localizable tokens; allow `REAL` / `FAKE` replacements to expand |
| Rematch button | Medium | Button supports two-line label at narrow desktop widths |
| Return to Lobby button | High | Long in French/German; button can expand or wrap to two lines |
| Resolution timeout copy | High | Must remain clear but concise; avoid implementation jargon in localized copy |
| Resource summary | Low | Numeric-heavy; localize resource labels, not game currency suffix unless global economy UI changes |

All numeric values are game values, not locale-specific dates or currencies. Round uses the existing HUD format `R<round_number>`.

---

## Acceptance Criteria

- [ ] Result screen opens within 300ms of receiving `S2CPhaseChanged(GAME_OVER)` and a usable `S2CGameOver` payload.
- [ ] Given `loser == opponent` and `reason == ObjectivesDestroyed`, the headline reads `VICTORY` and the cause identifies opponent real-objective loss.
- [ ] Given `loser == local` and `reason == ObjectivesDestroyed`, the headline reads `DEFEAT` and the cause identifies local real-objective loss.
- [ ] Given `loser == None` and `reason == Draw`, the headline reads `DRAW` and no player is presented as winner.
- [ ] Given `reason == ResolutionTimeout`, the headline reads `NO RESULT` and the copy explains that resolution timed out without declaring a winner.
- [ ] Objective reveal summary displays five lanes for each player with final HP and alive/destroyed state.
- [ ] Destroyed opponent objectives display their revealed real/fake identity from `OpponentObjectiveSnapshot.was_fake`.
- [ ] If full opponent identity reveal data is unavailable for alive lanes, those lanes show `Unknown` rather than client-inferred identity.
- [ ] Final resource summary displays final round, final gold, mana/mana cap, reserve mana, and real/fake objective counts when source data is available.
- [ ] HUD remains visible and frozen behind the overlay; the result screen does not add real/fake markers to HUD dots.
- [ ] Return to Lobby sends or has already sent `C2SAcknowledgeResult` and routes back to the main-menu/lobby flow.
- [ ] Rematch is disabled or hidden when rematch protocol support is unavailable, with explanatory status text.
- [ ] Reconnect at GAME_OVER either reconstructs the result screen from authoritative result payload or shows `RESULT PENDING` with Return to Lobby still usable.
- [ ] All interactive elements are reachable via keyboard Tab in logical order and have visible focus indicators.
- [ ] Reduced-motion mode removes iris wipe, bloom flash, scale pulse, and row sequencing while preserving all result and reveal information.
- [ ] At 1366x768, 1920x1080, and 150% UI scale, headline, cause, objective rows, resource summary, and buttons do not overlap.

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-RS-1 | Full post-game reveal data: should GAME_OVER reveal all opponent objective identities, including still-alive lanes? Current `S2CGameSnapshot` only reveals destroyed opponent objective identities via `was_fake`. | Network Protocol + Objective System | High |
| OQ-RS-2 | GAME_OVER reconnect result payload: should `S2CGameSnapshot` include `game_over: Option<{ loser, round, reason }>` or should the server re-send `S2CGameOver` on reconnect during GAME_OVER? | Network Protocol | High |
| OQ-RS-3 | Rematch protocol: is rematch same-session negotiation before cleanup, or a convenience action that creates a fresh lobby/room after `C2SAcknowledgeResult`? | Game Session System + Network Protocol | High |
| OQ-RS-4 | Result acknowledgement timing: should `C2SAcknowledgeResult` be sent on first stable render, on Return to Lobby, or after rematch negotiation is resolved? This affects server cleanup and rematch feasibility. | Game Session System | High |
| OQ-RS-5 | Should final score include hand size/card counts and reward history, or stay limited to objectives and resources for M2? | UX Designer + Producer | Medium |

---

## Cross-Reference Check

| Check | Result |
|---|---|
| RSM GAME_OVER data | Covered: headline and cause map from `S2CGameOver { loser, round, reason }` |
| Objective reveal | Partially covered: destroyed objective reveals are covered; full alive-lane reveal requires new server data |
| HUD freeze | Covered: overlay sits above frozen HUD and does not mutate HUD real/fake display |
| Main-menu return | Covered: resolves OQ-MM-4 by routing GAME_OVER to result screen first, then main-menu/lobby flow |
| Interaction patterns | New pattern gaps flagged: Result Overlay, Objective Reveal Summary, Rematch Pending State |
| Accessibility | Covered for keyboard focus, text labels, reduced motion, and photosensitivity |
