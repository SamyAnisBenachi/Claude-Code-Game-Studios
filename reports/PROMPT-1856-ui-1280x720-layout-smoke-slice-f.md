# PROMPT 1856 — UI-1280X720-LAYOUT-SMOKE-SLICE-F

**Date**: 2026-05-28
**Scope**: 1280×720 layout smoke audit — clipped CTAs, offscreen elements, cramped panels, hand fan occlusion, result/mulligan modal overflow, text overflow in compact panels
**Evidence type**: Static code review (`client/src/ui/**`) + live screenshots from autoplay run `20260528-090613-Z` (window at 1280×1076, NOT 1280×720)
**Status**: Report-only — no production UI edits made

---

## Viewport Context

| Name | Size | Role |
|------|------|------|
| Dev floor (`SAFETY_VIEWPORT_DEV_FLOOR`) | 1280×720 | Target of this audit |
| Prod min (`SAFETY_VIEWPORT_PROD_MIN`) | 1366×768 | Minimum supported per spec §8 |
| HD baseline | 1920×1080 | Design source viewport |

**PlayArea geometry at 1280×720**:
- HeaderBar: 60px (top)
- FooterBar + HandBar: 40 + 180 = 220px (bottom)
- PlayArea height = 720 − 60 − 220 = **440px**
- PlayArea width = 1280px (full)

---

## Ranked Issue List

### F1 — Lobby class picker 7th cell clipped at 1280px window width [P2 HIGH] 🔴 LIVE CONFIRMED

**Evidence**: Autoplay screenshot `bitblt_tick_000051.png` (1280×1076 window) shows 7 class-picker cells in the lobby; the 7th cell (Neutral) is clipped at the right edge of the modal panel. Later screenshots at a smaller window size show all 7 cells fully visible.

**Code analysis** (`client/src/ui/lobby.rs`):
```
LOBBY_CLASS_PICKER_GRID_COLUMNS = 7
LOBBY_CLASS_PICKER_CELL_WIDTH_PX = 108px
column_gap = SPACING_SM = 8px
Required width = 7×108 + 6×8 = 756 + 48 = 804px

Panel outer = min(88% × 1280, 860) = 860px
Panel padding = 2 × SPACING_LG = 48px (2 × 24)
Content width = 860 − 48 = 812px
Margin = 812 − 804 = 8px
```

The theoretical margin is only **8px**. At native DPI (scale=1.0) it barely fits; any UI scale factor (>1.0 on HiDPI displays) makes cells wider in logical pixels and triggers visible clipping. The live screenshot confirms clipping occurs at 1280px width with this monitor configuration.

**Test gap**: `lobby_class_picker_layout_test.rs::ac3_ac4_grid_columns_fit_minimum_and_hd_viewports` tests at 1366×768 and 1920×1080 only — **1280×720 is not covered**.

**Suggested next prompt**: `PROMPT-LOBBY-CLASS-PICKER-1280-FIX` — reduce cell width by 2px (106px) or tighten column gap to 6px to add 14px safety margin; add 1280×720 case to `ac3_ac4`.

---

### F2 — Result screen `Overflow::clip()` silently hides content at 720px [P2 HIGH] 🟡 INFERRED

**Evidence**: Static code review — `client/src/presentation/result_screen.rs` line ~791:
```rust
overflow: Overflow::clip(),
max_height: Val::Percent(92.0),   // = 662.4px at 720px viewport
```

**Analysis**: At 720px height, the result screen container has a hard ceiling of **662.4px**. Content taller than 662px (scoreboard rows, match statistics, extended player stats) is silently clipped without a scroll affordance. No `scroll_y()` fallback exists.

Compare: `draft_initial_modal_panel_node()` uses `Overflow::scroll_y()` correctly. Result screen regresses to `clip()`.

**Suggested next prompt**: `PROMPT-RESULT-SCREEN-OVERFLOW-SCROLL` — change result screen outer panel to `Overflow::scroll_y()` and add a 1280×720 content-budget test.

---

### F3 — Hand fan strip overflows 80px into PlayArea [P2 HIGH] 🟡 INFERRED

**Evidence**: Static code (`client/src/ui/hand/mod.rs`, `client/src/ui/design_tokens/strips.rs`, `client/src/ui/design_tokens/play_area.rs`):
```
HAND_FAN_STRIP_HEIGHT_PX = 260px     (HandFanRoot local height)
HAND_BAR_HEIGHT_PX       = 180px     (strip footprint per spec §9)
Overflow                 = 260 − 180 = 80px above HandBar into PlayArea
```

The fan root strip is 260px tall but the HandBar strip it sits in is only 180px. The HandFanRoot overflows 80px upward into the PlayArea coordinate space. At 720px viewport:
- HandBar top = 720 − 180 = 540 (viewport y)
- Fan root top = 720 − 260 = 460 (viewport y)
- Fan cards at `fan_base_y = 160` in local space → viewport y ≈ 460 + 10 = 470

PlayArea bottom = 720 − 220 = 500. Shop/auction bottom panel (`bottom_panel_node()`) fills PlayArea with `left:0, right:0, top:0, bottom:0`. At viewport y=470–500, fan card tops overlap the shop panel's bottom 30px.

This is a known z-layer interaction but the 80px fan spillover into PlayArea has no documented clip guard. At 720px it is more acute than at 1080px (same 80px overlap, but smaller PlayArea to absorb it).

**Suggested next prompt**: `PROMPT-FAN-PLAYAREA-OVERLAP-GUARD` — add z-layer ordering test or PlayArea bottom padding equal to fan overflow height; or document the z-order contract explicitly.

---

### F4 — Auction readability panel: hardcoded 552px left offset, no 1280×720 safety test [P3 MEDIUM] 🟡 INFERRED

**Evidence** (`client/src/ui/shop_auction/mod.rs`):
```
AUCTION_READABILITY_CARD_LEFT_PX     = 112px  (featured card)
AUCTION_FEATURED_CARD_WIDTH_PX       = 380px  → right edge at 492px
AUCTION_READABILITY_INFO_LEFT_PX     = 552px  (gap = 60px)
AUCTION_READABILITY_INFO_WIDTH_PX    = 468px  → right edge at 1020px
Bid buttons (index 0–3): 552 + n×124px → rightmost ends at 1032px
```

At 1280px viewport (PlayArea = full width): 1032px < 1280px ✓. All elements fit with 248px right margin. However:
- No responsive adaptation if viewport is narrower than 1280px
- No integration test covering this geometry at the 1280×720 dev floor
- `auction_featured_card_layout_test.rs` and `auction_free_gold_counters_layout_test.rs` exist but their viewport coverage is unknown

**Suggested next prompt**: `PROMPT-AUCTION-LAYOUT-1280-SAFETY-TEST` — verify existing auction tests cover 1280×720; add static assertion that rightmost bid button right-edge < viewport width at every `SAFETY_VIEWPORT_MATRIX` row.

---

### F5 — PlayArea top overlaps LaneBar by 60px if LaneBar is spawned [P3 MEDIUM] 🟡 INFERRED

**Evidence** (`client/src/ui/design_tokens/play_area.rs`, `strips.rs`):
```
PlayArea.top = HEADER_BAR_HEIGHT_PX = 60px
LaneBar: top = HEADER_BAR_HEIGHT_PX (60), height = 60px → occupies viewport y=60–120
```

PlayArea starts at y=60 (same as LaneBar top). If LaneBar is ever spawned, it occupies the first 60px of PlayArea, visually covering shop/draft panels' top band. Currently `strips.rs` notes LaneBar as "documented-only" and not yet spawned by any shipped plugin — but there is no spawn-guard to prevent future duplication.

**Suggested next prompt**: `PROMPT-LANEBAR-PLAYAREA-OFFSET-RECONCILE` — either update PlayArea.top to `HEADER_BAR_HEIGHT_PX + LANE_BAR_HEIGHT_PX` when LaneBar is spawned, or add a build-time assertion that only one of the two occupies y=60–120.

---

### F6 — Drag sprite has no viewport-edge clamp [P3 MEDIUM] 🟡 INFERRED

**Evidence** (`apply_fan_layout_system` and `sync_hand_drag_sprite_position_system`, `client/src/ui/hand/mod.rs`):

The drag sprite `Node.left` / `Node.top` track the cursor screen position verbatim with no edge clamping. At 1280×720, if the cursor approaches the right edge (x > 1172, i.e., 1280 − 108) or bottom edge (y > 570), the drag card (108×150px) overflows the viewport and clips without visual feedback to the user.

**Suggested next prompt**: `PROMPT-DRAG-SPRITE-EDGE-CLAMP` — clamp `left` to `(0.0)..=(viewport_width − HAND_CARD_DISPLAY_WIDTH_PX)` and `top` similarly in `sync_hand_drag_sprite_position_system`.

---

### F7 — Draft initial modal scroll indicator missing [P4 LOW] 🟡 INFERRED

**Evidence** (`draft_initial_modal_panel_node()`, `client/src/ui/shop_auction/mod.rs`):
```rust
overflow: Overflow::scroll_y()  // correct
```

The modal uses `scroll_y()` correctly (contrast with F2 above), but Bevy 0.18's `Overflow::scroll_y()` does not render a visible scrollbar indicator by default. If content at 1280×720 causes the modal to exceed its 662px max-height and scroll, the user has no visual affordance that scroll is available.

**Suggested next prompt**: `PROMPT-SCROLL-AFFORDANCE-DRAFT-MODAL` — add a custom scrollbar child node, or document the accept-risk if the content always fits at 720px.

---

## Surfaces with No Issues Found (static code)

| Surface | Analysis | Result |
|---------|----------|--------|
| Lobby modal outer (result/lobby panel) | 88% × 1280 = 860px cap, 92% height = 662px, `Overflow::clip()` | Fits at 720px ✓ |
| Photosensitivity/connection-lost overlay | Narrow kind: max 520px, 16px padding, 720px fits ✓ | OK |
| HUD header strip | 60px height fixed, full width | OK |
| HUD footer strip | 40px height fixed, full width | OK |
| Shop footer slots | `left = 92 + n×154`, 4 slots → rightmost right-edge = 554+136=690px < 1280px | OK |
| Placement action panel | Inside PlayArea, not checked in detail — owned by PROMPT 1855 | Defer |
| Mana preview / HUD mana bar | 104×28px, anchored to HUD strip | OK |

---

## Coverage Gaps

1. **No autoplay run at exactly 1280×720**: `status.json` reports `window_logical_size: [1280, 1076]`. All screenshots in this audit are at 1280×1076, not 1280×720. A 720px height run is required to validate the vertical budget for the result screen (F2), auction featured card height in PlayArea (F3), and draft initial modal scrolling (F7).

2. **Mulligan modal not found**: No standalone `mulligan*.rs` file in `client/src/ui/` or `client/src/presentation/`. Mulligan phase transitions may be handled by the draft-initial modal path. Needs confirmation from a live run.

3. **HiDPI / UI scale factor**: The live screenshot clipping at 1280×1076 (F1) is consistent with a UI scale >1.0. Bevy's default `UiScaleMode` can apply monitor DPI scaling. Fixed-pixel constants (108px cell, 860px panel) may render larger in HiDPI logical space. This affects all fixed-px UI at 1280×720 on HiDPI monitors.

---

## Suggested Next Prompt Splits (ranked)

| Priority | Prompt slug | Finding | Owner |
|----------|-------------|---------|-------|
| P2 | `PROMPT-LOBBY-CLASS-PICKER-1280-FIX` | F1 — cell clipping confirmed live | UI programmer |
| P2 | `PROMPT-RESULT-SCREEN-OVERFLOW-SCROLL` | F2 — clip→scroll_y migration | UI programmer |
| P2 | `PROMPT-FAN-PLAYAREA-OVERLAP-GUARD` | F3 — fan/PlayArea z-layer contract | Lead programmer |
| P3 | `PROMPT-AUCTION-LAYOUT-1280-SAFETY-TEST` | F4 — add 1280×720 safety test | QA / UI programmer |
| P3 | `PROMPT-DRAG-SPRITE-EDGE-CLAMP` | F6 — viewport clamp | UI programmer |
| P4 | `PROMPT-LANEBAR-PLAYAREA-OFFSET-RECONCILE` | F5 — LaneBar collision guard | Engine programmer |
| OPP | `PROMPT-1280x720-LIVE-RUN` | NEEDS_HUMAN_GUI — run autoplay at 720px height | Human / QA |

---

## NEEDS_HUMAN_GUI

To complete the live phase-screenshot portion of this audit, a human must:

1. Launch the client with window fixed to `1280×720` (set `Window::resolution` or OS-resize to exactly 720px inner height)
2. Run through: Lobby → DraftInitial → DraftAuction → Placement → Result screen
3. Capture screenshots at each phase entry
4. Look for: result screen bottom CTA reachability, auction bid buttons fully visible, draft modal bottom footer in view, placement action panel CTA not occluded

---

## Validation

- `git diff --check`: No production files modified. This is a report-only lane.
- Evidence: static code review + autoplay screenshots (1280×1076 window, not 1280×720)
- F1 confirmed via live screenshot. F2–F7 inferred from constants and node builders.

1856: UI-1280X720-LAYOUT-SMOKE-SLICE-F: SHIPPED
