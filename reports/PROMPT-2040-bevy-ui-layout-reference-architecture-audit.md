# PROMPT 2040 — Bevy UI Layout Reference Architecture Audit

**Branch:** work/PROMPT-2040
**Source-of-truth:** origin/main @ 8f7d3502
**Worktree HEAD:** a295db2a
**Engine:** Bevy 0.18 (per `docs/engine-reference/bevy/VERSION.md`)
**Skill applied:** liv-bevy-018 (Required Components, GlobalZIndex, Flex-first layout, no pre-0.15 Bundle patterns).
**Scope:** Audit/report only. No game-code edits.

---

## 1. Executive Summary

The Bevy UI architecture is **fundamentally sound** and follows Bevy 0.18 idioms (Required Components Node API, GlobalZIndex layering, design-token primitives, Flex-first layout). The user's "whole UI feels incorrectly anchored" symptom is **NOT** a wholesale architectural failure. It traces to a **small number of localized regressions** layered on top of an otherwise idiomatic foundation that already received targeted repairs (PROMPT 802 result-screen baseline, PROMPT 933 modal mirror, PROMPT 1180 PlayArea reparent, PROMPT 1398 lobby body wrap).

**Verdict per surface:**

| Surface | Verdict | Reason |
|---|---|---|
| Lobby root + panel | SALVAGEABLE | Flex column + max-width/percent + body flex_grow already idiomatic (PROMPT 1398). |
| Class picker grid | SALVAGEABLE | Flex grid, no hardcoded pixel positions for cells. |
| Shop / Auction modals | SALVAGEABLE | Mirrors result-screen modal pattern; card_slot primitive abstracts geometry. |
| Draft initial 3×N modal | SALVAGEABLE | Flex-based grid via summed-constant width. |
| Hand fan strip | SALVAGEABLE | Absolute edge-anchor at root, Flex slots inside; viewport-aware metrics. |
| **Draft grid (in-modal)** | **NEEDS REFACTOR** | Hardcoded `Val::Px(96 + col*132)` / `Val::Px(28 + row*66)` absolute layout (`hand/mod.rs:4957-4959`). |
| **Hand drag ghost preview** | **NEEDS REWRITE** | World-space `Sprite`, not `bevy_ui::Node` — z-order ambiguity vs board overlays. |
| Placement action panel | SALVAGEABLE | Absolute-anchored to PlayArea (post-1180), Flex internals. |
| HUD strips (header / objective / timer) | SALVAGEABLE | Edge-anchored Absolute root, stateless. |
| Result screen | CORRECT (baseline) | Reference pattern other modals copy. |
| Board (world layer) | CORRECT | Sprite/Transform world-space; outside bevy_ui by ADR-021 §R2. |

**No staged architectural rewrite is justified.** A 3-phase repair plan (below) closes the systemic gaps.

---

## 2. Module Owner Map

Client UI lives under `client/src/ui/` with companions in `client/src/presentation/`:

| File | Owns |
|---|---|
| `client/src/ui/mod.rs` | UI plugin wiring, exports of `PlayArea`/`PhaseBanner` primitives |
| `client/src/ui/lobby.rs` | Lobby root + class picker grid + slot panels + room browser |
| `client/src/ui/hand/mod.rs` | Hand fan, draft grid, placement action panel, drag ghost |
| `client/src/ui/shop_auction/mod.rs` | Shop slots, auction featured card, draft-initial modal |
| `client/src/ui/hud/mod.rs` | Phase timer, round/gold/mana, objective dots |
| `client/src/ui/design_tokens/card_slot.rs` | Card geometry primitive (5 `CardSlotKind` variants) |
| `client/src/ui/design_tokens/z_layers.rs` | 8 `GlobalZIndex` constants (BACKGROUND → DEBUG) |
| `client/src/ui/design_tokens/play_area.rs` | Middle-band flex container (PROMPT 1180 reparent target) |
| `client/src/ui/design_tokens/viewport_matrix.rs` | Safety viewport matrix: 1280×720 / 1366×768 / 1920×1080 |
| `client/src/ui/design_tokens/modal_panel.rs` | Centered modal template |
| `client/src/ui/design_tokens/strips.rs` | HeaderBar / FooterBar / HandBar anchor constants |
| `client/src/ui/shared.rs` | `BoardLayout` coordinate model, `HudObjectiveUpdate` |
| `client/src/ui/phase_banner.rs` | Transient phase-change label (MODAL layer) |
| `client/src/presentation/result_screen.rs` | Result modal — **baseline reference** for all modal patterns |
| `client/src/presentation/board_rendering.rs` | World-space board sprites (NOT bevy_ui by ADR-021) |
| `client/src/presentation/targeting_overlay.rs` | World-space hit-test for sprite-layer picking |

---

## 3. Per-Surface Findings

### 3.1 Lobby (SALVAGEABLE)
- `LobbyRoot`: full-viewport flex with `z_layers::UI_OVERLAY` scrim. ✓
- `LobbyPanel`: `width: 88%`, `max_width: 860px`, `max_height: 92%`, `z_layers::MODAL`. ✓
- `LobbyPanelBody`: `flex_grow: 1`, `flex_shrink: 1`, `min_height: 0` — guarantees Confirm CTA reachable at 1280×720 safety floor (PROMPT 1398 fix). ✓
- Class picker: Flex grid, named constants `LOBBY_CLASS_PICKER_CELL_WIDTH_PX` / `..._HEIGHT_PX`, no per-cell absolute positions. ✓

### 3.2 Shop / Auction (SALVAGEABLE)
- Draft-initial modal mirrors result-screen literals (88% width / 860 max / 92% max height). ✓
- Grid width = `col_width * 3 + gap * 2` — derived, not hardcoded position. ✓
- Slots use `card_slot_node(CardSlotKind::Shop|AuctionFeatured)` primitive. ✓
- Featured card: 380×280 via `CardSlotKind::AuctionFeatured`. ✓

### 3.3 Hand fan (SALVAGEABLE)
- `HandFanRoot`: `PositionType::Absolute`, `left:0 right:0 bottom:0`, `height: HAND_FAN_STRIP_HEIGHT_PX` (260 px). ✓
- Slot positions: **relative inside fan_root**, computed from PrimaryWindow size (no hardcoded viewport assumption). ✓
- Drag overlay layer: `z_layers::UI_OVERLAY`. ✓

### 3.4 Draft grid (NEEDS REFACTOR — CRITICAL)
- `hand/mod.rs:4953-4965` builds the in-modal draft grid via:
  ```rust
  PositionType::Absolute,
  left: Val::Px(96.0 + column as f32 * 132.0),
  top:  Val::Px(28.0  + row    as f32 * 66.0),
  ```
- Every cell is hand-placed with hardcoded pixel offsets. If the parent modal resizes (window-resize, accessibility text scale, different `CardSlotKind`), cells overflow or clip.
- **Why this is the user's symptom:** the draft grid is the one place where absolute pixel math leaks into per-child positioning. It will literally look "misanchored" on any non-design-target viewport.

### 3.5 Placement action panel (SALVAGEABLE)
- `PositionType::Absolute` anchored 16 px from PlayArea bottom-right (PROMPT 1180 reparent). ✓
- Internal layout: Flex column. ✓
- Comment at `hand/mod.rs:5032-5035` still references "HandBar bottom" — stale after 1180 reparent. LOW-severity comment hygiene only.

### 3.6 Hand drag ghost (NEEDS REWRITE — HIGH)
- Drag preview is a world-space `Sprite`, not a `bevy_ui::Node`.
- Consequence: z-order vs board overlays / placement preview is governed by `Transform.z` against world geometry, **not** by bevy_ui `GlobalZIndex` layering. This is the root cause of the "drag ghost feels in the wrong layer" symptom and crosses the ADR-021 §R2 Sprite/UI boundary in the wrong direction.
- Fix: migrate ghost to `Node` + `ImageNode`, parent it to `UI_OVERLAY` layer, drive its position from cursor in screen-space.

### 3.7 HUD (SALVAGEABLE)
- Top strip Absolute-anchored `top:0 left:0 right:0`, height = `HEADER_BAR_HEIGHT_PX` (60 px). ✓
- All data drives from `CurrentClientPhase`, `BoardLayout`, `HudObjectiveUpdate` messages. ✓
- Inline `Color::srgba(...)` literals still mixed with design-token constants (`hud/mod.rs:109-120`). LOW-severity tokenization drift.

### 3.8 Result screen (CORRECT — baseline)
- Authoritative modal pattern; do not touch.

### 3.9 Board rendering (CORRECT, out of UI scope)
- World-space `Sprite` + `Transform.z` against `z_layers::WORLD`/`UNITS` — by design (ADR-021 §R2).

---

## 4. Systemic Findings (severity-ranked)

| # | Sev | Finding | File(s) | Recommended 0.18 idiom |
|---|---|---|---|---|
| F1 | CRITICAL | Draft grid uses absolute pixel offsets per cell | `hand/mod.rs:4953-4965` | Replace with `Display::Flex` + `flex_wrap: Wrap` + `row_gap`/`column_gap` on a `Relative` parent. Cells become responsive. |
| F2 | HIGH | Drag ghost is world-space `Sprite`, crossing UI/world boundary | `hand/mod.rs` drag spawn | Use `Node { position_type: Absolute, .. }` + `ImageNode`, place on `UI_OVERLAY` `GlobalZIndex`, drive `left`/`top` from cursor in window-space. |
| F3 | HIGH | No enforced minimum viewport for hand fan; below 1280×720 safety floor the fan can overflow | `hand/mod.rs` fan root, `viewport_matrix.rs:85` | Gate fan spawn on `viewport_matrix::within_safety_floor()`; otherwise render a "viewport too small" notice. Already the documented safety floor — just enforce it. |
| F4 | MEDIUM | Placement panel anchor offsets (16/16) are not asserted to stay inside PlayArea at 1280×720 | `hand/mod.rs:5029-5049` + missing `play_area_budget_test.rs` | Add `World`-based test asserting panel rect ⊂ PlayArea rect at all three safety viewports. |
| F5 | MEDIUM | Modal centering is Flex-based, not Transform-translate — correct, but uncovered by window-resize regression test | `lobby.rs`, `shop_auction/mod.rs` | Add `World`-based test that runs UI schedule, resizes `Window`, runs UI schedule again, asserts node `ComputedNode.size` updates. |
| F6 | MEDIUM | Some bevy_ui surfaces still use hand-rolled hit checks instead of `Interaction` component | search hand drag + targeting overlay | Where the surface IS bevy_ui (i.e. not board), prefer `Interaction` + `RelativeCursorPosition`. Keep custom hit-test only on world-space board. |
| F7 | LOW | Stale comments after PROMPT 1180 reparent | `hand/mod.rs:5032-5035` | Update comment to reference PlayArea. |
| F8 | LOW | Inline color literals not yet tokenized in HUD | `hud/mod.rs:109-120` | Migrate to named constants in `design_tokens/`. |
| F9 | LOW | Z-layer ladder has 100-unit gaps but only 8 layers used | `z_layers.rs` | Intentional headroom; no action. |

**No findings of:** broken `add_event` / `EventReader` patterns, pre-0.15 Bundles (`NodeBundle`, `TextBundle`, `ButtonBundle`), deprecated hierarchy APIs (`set_parent`, `despawn_recursive`), `Query::single()` un-`Result`-handled, or `UiImage` (replaced by `ImageNode` already). The codebase is on the 0.18 API surface.

---

## 5. Correlation With Reported Bug IDs

The file `production/qa/bugs/current-unplayable-bug-register-2026-05-28.md` was not directly inspected by the audit pass; the symptom families V1-013 / UX-011 / UX-012 map plausibly onto findings as follows (to be confirmed by QA when register entries are read):

- "Whole UI feels misanchored" → likely a composite of **F1** (draft grid pixel offsets) + **F2** (drag ghost layer crossing) + **F3** (no enforced viewport floor). Fixing F1+F2+F3 is expected to resolve the user's perceived "everything is wrong" complaint without rewrites elsewhere.
- "Card chrome looks wrong / clipped" → most likely F1 (draft grid) at non-design viewports.
- "Drag preview appears under board pieces" → F2.
- Lobby Confirm CTA reachability — already closed by PROMPT 1398; **do not re-open** unless a new repro shows up.

---

## 6. Staged Repair Plan

**Phase 1 — Unblock perceived misanchoring (CRITICAL)**
- F1: Refactor `hand/mod.rs:4953-4965` draft grid to Flex + wrap + gap. Delete the four hardcoded literals.
- F3: Add safety-floor gate on fan spawn using `viewport_matrix::within_safety_floor()`.
- Test: `World`-based UI schedule run at 1280×720, 1366×768, 1920×1080 asserting no draft cell rect exits modal body rect.
- Est: 4–6 h. High confidence — Flex is the idiomatic 0.18 path.

**Phase 2 — Fix drag layer crossing (HIGH)**
- F2: Reimplement drag ghost as `Node + ImageNode` parented to a `UI_OVERLAY` root. Replace world-space `Sprite` spawn.
- Test: `World`-based test asserting ghost entity has `GlobalZIndex` and `Node` and no `Sprite`.
- Est: 4–6 h. Medium risk — touches drag animation hand-off.

**Phase 3 — Lock the safety net (MEDIUM)**
- F4 + F5 + F6: introduce `tests/integration/ui/play_area_budget_test.rs` covering placement panel containment, modal re-center on window resize, and `Interaction` usage for bevy_ui surfaces.
- Est: 3–5 h. Low risk — proven harness pattern.

**Phase 4 — Hygiene (LOW)**
- F7 + F8 stale comments + color literal tokenization. Single small PR.
- Est: 1–2 h.

**Total budget:** ~12–19 h for full surface stabilization. No surface needs a ground-up rewrite.

---

## 7. References Used

- `docs/engine-reference/bevy/VERSION.md` — pinned Bevy 0.18, lightyear 0.26, knowledge-gap warning for 0.15–0.18 APIs.
- `docs/architecture/adr-021-*` — Presentation Layer Architecture, §R2 Sprite/UI boundary, Implementation Guidelines 8 (UI ghost vs world sprite).
- liv-bevy-018 skill — Required Components Node API, `GlobalZIndex`, `Display::Flex` + `flex_wrap`/`gap`, `ImageNode`, `Interaction`, `RelativeCursorPosition`.
- Bevy 0.18 official references (cited inside VERSION.md):
  - https://bevy.org/news/bevy-0-18/
  - https://bevy.org/learn/migration-guides/0-17-to-0-18/
  - https://bevy.org/learn/migration-guides/0-16-to-0-17/
- Repo-local established patterns: `presentation/result_screen.rs` (modal baseline), `design_tokens/play_area.rs` (PROMPT 1180 reparent), `design_tokens/viewport_matrix.rs` (safety viewport matrix).

---

2040: BEVY-UI-LAYOUT-REFERENCE-ARCHITECTURE-AUDIT: SHIPPED
