# PROMPT 1947 — KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1943

**Branch:** `integrate/krosmaga-hand-fan-readability-1947`
**Date:** 2026-05-28
**Base:** `origin/main` @ `e62c431e` (PROMPT 1943)

---

## Context

PROMPT 1910 (`origin/integrate/krosmaga-hand-fan-readability-1910`) was rejected by the
orchestrator because it was NOT_FF against current main, deleted already-landed reports,
and carried stale unrelated changes in `client/src/autoplay.rs`,
`tools/autoplay/Run-AutoplaySmoke.ps1`, and `tools/dev-launcher/Start-TwoClients.ps1`.

This PROMPT reapplies only the hand fan readability payload cleanly from
`origin/main@e62c431e` (PROMPT 1943), without touching autoplay/tooling files.

---

## Files Changed

| Status | File |
|--------|------|
| M | `client/src/ui/hand/mod.rs` |
| M | `tests/integration/hand-ui/draft_initial_grid_test.rs` |
| M | `tests/unit/hand-ui/fan_layout_formula_test.rs` |
| A | `reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` |
| A | `reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` |
| A | `reports/PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md` |
| A | `reports/PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md` |

No deletions. No changes to `client/src/autoplay.rs`, `tools/**`, `production/**`,
or Cargo files.

---

## Payload Summary

### `client/src/ui/hand/mod.rs` — `HandFanLayoutConfig::default()`

| Field | Old | New | Reason |
|---|---|---|---|
| `fan_base_margin_px` | 100.0 | **150.0** | card bottom → strip-local y=260 (no clip) |
| `fan_half_spread_px` | 280.0 | **380.0** | spacing 84 px > badge width 26 px |
| `arc_height_px` | 20.0 | **30.0** | visible arc with new strip room |
| `max_rotation_deg` | 8.0 | 8.0 | unchanged |

Fixes two readability bugs at 1280×720 with a full 10-card hand:
- **ATK/HP bottom-clip**: old margin placed card bottom 50 px below viewport edge.
- **AR/HP badge occlusion**: old spread caused 46 px overlap, hiding right-side badges.

### `tests/integration/hand-ui/draft_initial_grid_test.rs`

- `qa_metrics()`: `fan_base_y` updated **160 → 110** (= 260 − 150 per new default margin).
- Updated comment to document the PROMPT 1854 change.

### `tests/unit/hand-ui/fan_layout_formula_test.rs`

- Added `CARD_H = 150.0` and `RIGHT_BADGE_W = 108.0 * 0.24` module constants.
- Added `default_config_10_cards_at_1280x720_readability_invariants` test with two invariants:
  1. No card bottom clips below `HAND_FAN_STRIP_HEIGHT_PX` (ATK/HP badge visibility).
  2. Adjacent card spacing exceeds right-badge width (AR/HP badge visibility).

### Reports Carried Forward

- `PROMPT-1854`: original slice-D report.
- `PROMPT-1878`: first refresh report (was on 1910 branch, not yet on main).
- `PROMPT-1910`: previous refresh attempt report (carried from root checkout).
- `PROMPT-1947`: this file.

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

- All card bottoms ≤ 260 ✓
- All spacings 84.4 px > 25.9 px (right-badge width) ✓
- Leftmost left edge: 260 px ✓
- Rightmost right edge: 1128 px < 1280 px ✓

---

## Validation

### Allowlist check

```
git diff --name-status origin/main..HEAD
M  client/src/ui/hand/mod.rs
M  tests/integration/hand-ui/draft_initial_grid_test.rs
M  tests/unit/hand-ui/fan_layout_formula_test.rs
A  reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md
A  reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md
A  reports/PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md
A  reports/PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md
```

- No deletions ✓
- `client/src/autoplay.rs` — not touched ✓
- `tools/**` — not touched ✓
- `production/**` — not touched ✓
- Cargo files — not touched ✓
- `git diff --check` — clean ✓

### Compilation

Cargo test deferred: D: drive space constraints prevent full `cargo test` suite.
Changes are statically verified by formula. Run `cargo test -p client fan_layout`
when drive space is available.

---

1947: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1943: READY_FOR_MAINLAND_ENQUEUE
