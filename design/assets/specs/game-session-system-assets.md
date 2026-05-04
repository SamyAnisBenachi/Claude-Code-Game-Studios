# Asset Specs - System: Game Session / Lobby / Reconnect / Outcome

> **Source**: design/gdd/game-session-system.md; design/ux/main-menu.md; design/ux/interaction-patterns.md
> **Art Bible**: design/art/art-bible.md
> **Generated**: 2026-05-04
> **Status**: 20 assets specced / 0 approved / 0 in production / 0 done
> **Asset IDs**: ASSET-195 through ASSET-214

---

## Scope Notes

Game Session owns the pre-game lobby surface and the player-facing states around connection, cancellation, reconnect, and post-match handoff. The Game Session GDD's Visual/Audio and UI sections are still marked "To be designed"; therefore several rows below are intentionally placeholder ownership rows.

Result-screen rows are blocked pending result-screen UX. They exist only so the manifest tracks ownership and does not lose the requirement.

---

## Lobby / Main Menu Assets

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-195 | Title / Lobby Backdrop | UI Background | 1024x1024 tiled PNG-32 or procedural bands | atlas_ui_hud or standalone | Placeholder |
| ASSET-196 | Room Code Display Chip | UI | 9-slice PNG-32 or Bevy UI material | atlas_ui_hud | Needed |
| ASSET-197 | Room Code Copy Icon | UI Icon | 24x24 PNG-32 | atlas_ui_hud | Needed |
| ASSET-198 | Join Room Input Frame | UI | 9-slice PNG-32 or Bevy UI material | atlas_ui_hud | Needed |
| ASSET-199 | Lobby Player Slot Panel States | UI | Empty/connected/confirmed/locked states | atlas_ui_hud | Needed |
| ASSET-200 | Class Browser Carousel Arrows | UI Icon | 32x32 PNG-32, left/right variants | atlas_ui_hud | Needed |
| ASSET-201 | Lobby Timer Progress Bar Material | UI Material | PTN-DSP-005 lobby variant | N/A | Needed |
| ASSET-202 | Lobby Cancel Confirmation Overlay | UI Overlay | Bevy UI material / optional icon | atlas_ui_hud | Placeholder |
| ASSET-203 | Lobby Inline Error Flash Material | UI Material | Crimson-Amber 150ms flash | N/A | Needed |
| ASSET-204 | Button Loading Spinner | UI Icon / Animation | 24x24 PNG-32 or procedural spinner | atlas_ui_hud | Needed |
| ASSET-205 | Simultaneous Class Reveal Flash | VFX / UI Material | desaturate-to-resaturate flash | N/A | Needed |
| ASSET-206 | Session Cancelled - Opponent Left Overlay | UI Overlay | Runtime text + panel material | N/A | Placeholder |
| ASSET-207 | Session Cancelled - Timeout Overlay | UI Overlay | Runtime text + panel material | N/A | Placeholder |

### Visual Direction

- Lobby mood is anticipation, not marketing page spectacle. Use warm-neutral lighting and Ink Blue structure, with class color only after reveal.
- Opponent class must not leak before `S2CClassesRevealed`. Slot states before reveal must remain neutral.
- Class reveal panels arrive simultaneously and reach final position on the same frame. Reduced-motion mode replaces the slide/flash with an instant cut.
- Error states must include text. Crimson-Amber flash is feedback only, never the sole error communication.

---

## Reconnect / Outcome Ownership Rows

| Asset ID | Name | Category | Dimensions / Format | Atlas | Status |
|---|---|---|---|---|---|
| ASSET-208 | Reconnect Snapshot Rebuild Overlay | UI Overlay | Runtime panel + spinner | N/A | Placeholder |
| ASSET-209 | Opponent Disconnected Grace Overlay | UI Overlay | Runtime panel + timer text | N/A | Placeholder |
| ASSET-210 | Opponent Reconnected Toast | UI / Toast | Runtime text + small status icon | atlas_ui_hud | Placeholder |
| ASSET-211 | GAME_OVER Result Panel Placeholder | UI Overlay | Blocked pending result-screen UX | N/A | Blocked |
| ASSET-212 | Outcome Badge Placeholder Set | UI | Victory / defeat / draw placeholder ownership | N/A | Blocked |
| ASSET-213 | Post-Match Action Button Placeholder | UI | Rematch / return buttons, UX not designed | N/A | Blocked |
| ASSET-214 | Post-Match Objective Reveal Placeholder | UI / VFX | Objective identity reveal ownership only | N/A | Blocked |

### Blocked Notes

- ASSET-211 through ASSET-214 must not be commissioned until a result-screen UX spec exists.
- Current UX sources explicitly mark GAME_OVER post-match flow as not designed.
- These rows reserve ownership only; exact layout, copy, animation, and art treatment remain unresolved.
