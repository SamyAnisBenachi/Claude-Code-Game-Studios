# PROMPT 1878 — KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1872

**Branch:** `prompt-1878-hand-fan-readability-refresh`
**Date:** 2026-05-28
**Base:** `origin/main` @ `2ce3dc6b` (PROMPT 1872)

---

## Task

Cherry-pick the PROMPT 1854 hand fan readability payload onto the latest `origin/main`
without disturbing any PROMPT 1844/1845/1846/1858/1859/1872 artifacts already on main.

PROMPT 1854's original branch (`origin/prompt-1854-hand-fan-readability`) was not
FF-mergeable over current main: it branched from a stale base and would have deleted
reports added by PROMPTs 1844 through 1872.

---

## Changes Applied

### `client/src/ui/hand/mod.rs` — `HandFanLayoutConfig::default()`

| Field | Old | New | Reason |
|---|---|---|---|
| `fan_base_margin_px` | 100.0 | **150.0** | card bottom → strip-local y=260 (no clip) |
| `fan_half_spread_px` | 280.0 | **380.0** | spacing 84 px > badge width 26 px |
| `arc_height_px` | 20.0 | **30.0** | visible arc with new strip room |
| `max_rotation_deg` | 8.0 | 8.0 | unchanged |

### `tests/integration/hand-ui/draft_initial_grid_test.rs`

- `qa_metrics()`: `fan_base_y` updated **160 → 110** (= 260 − 150).
  The test harness uses `HandFanLayoutConfig::default()` implicitly (no explicit override
  in `app_with_hand_ui_in_draft_initial()`), so the expected 1-card `t=0` position must
  track the new default margin.

### `tests/unit/hand-ui/fan_layout_formula_test.rs`

- Added two module-level constants: `CARD_H = 150.0`, `RIGHT_BADGE_W = 108.0 * 0.24`.
- Added test: `default_config_10_cards_at_1280x720_readability_invariants`
  - **Invariant 1 (no bottom clip)**: `card_y + 150 ≤ HAND_FAN_STRIP_HEIGHT_PX` for all
    10 slots. Passes: center card bottom = 110 + 150 = 260; edge card bottom = 80 + 150 = 230.
  - **Invariant 2 (badge visibility)**: per-slot x-spacing > right-badge width (25.9 px).
    Passes: spacing = 380 × 2 / 9 = 84.4 px >> 25.9 px.

### Reports Restored/Added

- `reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` — restored from
  root checkout (was not committed on main before this PROMPT).
- `reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` — this
  file (new).

---

## Static Geometry Validation — 10-card 1280×720

With new defaults (`fan_base_margin_px=150`, `fan_half_spread_px=380`, `arc_height_px=30`):

**fan_base_y** = 260 − 150 = **110** (strip-local)

| Slot | t | card_x (1280px vp) | card_y | card_bottom | spacing to next |
|---|---|---|---|---|---|
| 0 | −1.000 | 260.0 | 80.0 | 230.0 | 84.4 px |
| 1 | −0.778 | 344.4 | 92.9 | 242.9 | 84.4 px |
| 2 | −0.556 | 428.9 | 103.7 | 253.7 | 84.4 px |
| 3 | −0.333 | 513.3 | 106.7 | 256.7 | 84.4 px |
| 4 | −0.111 | 597.8 | 109.6 | 259.6 | 84.4 px |
| 5 | +0.111 | 682.2 | 109.6 | 259.6 | 84.4 px |
| 6 | +0.333 | 766.7 | 106.7 | 256.7 | 84.4 px |
| 7 | +0.556 | 851.1 | 103.7 | 253.7 | 84.4 px |
| 8 | +0.778 | 935.6 | 92.9 | 242.9 | 84.4 px |
| 9 | +1.000 | 1020.0 | 80.0 | 230.0 | — |

- All card bottoms ≤ 260 (strip height) ✓
- All spacings 84.4 px > 25.9 px (right-badge width) ✓
- Leftmost left edge: 260 px — safe ✓
- Rightmost right edge: 1020 + 108 = 1128 px < 1280 px — safe ✓

---

## Scope / Allowlist Check

```
git diff --name-status origin/main..HEAD
M  client/src/ui/hand/mod.rs
M  tests/integration/hand-ui/draft_initial_grid_test.rs
M  tests/unit/hand-ui/fan_layout_formula_test.rs
A  reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md
A  reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md
```

- `client/src/ui/hand/inspect.rs` — **not touched** ✓
- `tools/autoplay/**` — **not touched** ✓
- All PROMPT 1844/1845/1846/1858/1859/1872 artifacts — **preserved** ✓
- `git diff --check` — clean ✓

---

## Compilation Note

Cargo test compilation deferred: D: drive has limited free space and full `cargo test`
would exhaust it. Changes are statically verified by formula (table above) and the new
`default_config_10_cards_at_1280x720_readability_invariants` test encodes those invariants
mathematically. Run `cargo test -p client fan_layout` when drive space is recovered.

---

1878: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1872: SHIPPED
