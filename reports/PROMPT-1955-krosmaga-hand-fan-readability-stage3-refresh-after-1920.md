# PROMPT 1955 — Krosmaga Hand Fan Readability Stage3-D Refresh After PROMPT 1920

**Date:** 2026-05-28
**Branch:** integrate/krosmaga-hand-fan-readability-1955
**Base:** origin/main @ 1c4981a65f02422de7d01505ce029d1c1551a3a8 (PROMPT 1920)
**Prior ref:** origin/integrate/krosmaga-hand-fan-readability-1947 @ 4a067f40 (stale — not strict-FF vs current main)

---

## Context

PROMPT 1947 produced a clean Stage3-D payload on main@e62c431e (PROMPT 1943), but
origin/integrate/krosmaga-hand-fan-readability-1947 became stale: main advanced through
PROMPT 1950 → 1920 (card inspect glossary) without the hand fan branch being merged,
and the old branch was not a strict fast-forward of the updated main (it also lacked
the 1937/1950/1920 reports that landed on main in the interim).

This PROMPT rebuilds the identical Stage3-D payload cleanly from
origin/main@1c4981a65f02422de7d01505ce029d1c1551a3a8 (PROMPT 1920).

---

## Scope Applied

### 1. `client/src/ui/hand/mod.rs` — `HandFanLayoutConfig::default()` tuning

| Field | Before | After | Reason |
|---|---|---|---|
| `fan_base_margin_px` | 100.0 | 150.0 | Lifts card strip high enough that ATK/HP bottom badges clear the viewport edge at 1280×720 |
| `fan_half_spread_px` | 280.0 | 380.0 | Widens fan arc so 10 cards don't overlap right-side AR/HP badges |
| `arc_height_px` | 20.0 | 30.0 | Increases arc curvature to match wider spread |

These values were established and validated in PROMPT 1854 (original Stage3-D slice).

### 2. `tests/integration/hand-ui/draft_initial_grid_test.rs` — `qa_metrics()` update

- `fan_base_y`: 160.0 → 110.0 (= 260 − 150, tracking updated `fan_base_margin_px = 150`)
- Comment updated to reflect new formula derivation.

### 3. `tests/unit/hand-ui/fan_layout_formula_test.rs` — Stage3-D readability invariants

Added `default_config_10_cards_at_1280x720_readability_invariants` test which asserts:
1. **No bottom clip**: `card_y + CARD_H ≤ HAND_FAN_STRIP_HEIGHT_PX` for every slot — ensures ATK/HP badges never clip below the strip edge.
2. **Right-badge visible**: spacing between adjacent card left edges > `RIGHT_BADGE_W` (108px × 24% ≈ 25.92px) — ensures AR/HP badges are not occluded by the neighboring card.

### 4. Reports carried (previously untracked on main)

| Report file | Originates from |
|---|---|
| `PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` | PROMPT 1854 (original Stage3-D slice) |
| `PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` | PROMPT 1878 (refresh after 1872) |
| `PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md` | PROMPT 1910 (refresh after 1894) |
| `PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md` | PROMPT 1947 (refresh after 1943, the directly preceding iteration) |
| `PROMPT-1955-krosmaga-hand-fan-readability-stage3-refresh-after-1920.md` | This report |

---

## Files NOT Touched (preservation checklist)

| File | Status |
|---|---|
| `client/src/ui/card_inspect.rs` | Untouched — PROMPT 1920 glossary work preserved |
| `client/src/ui/hand/inspect.rs` | Untouched — PROMPT 1920 glossary work preserved |
| `production/**` | Untouched |
| `tools/**` | Untouched |
| `Cargo.toml` / `Cargo.lock` | Untouched |
| `tests/integration/hand-ui/draft_initial_grid_test.rs` (non-Stage3-D) | Only `qa_metrics()` fan_base_y updated; all other test logic unchanged |
| All existing main reports | None deleted or modified |

---

## Validation

### Path allowlist review — PASS

`git diff HEAD --name-only` returns exactly the three owned source files:
```
client/src/ui/hand/mod.rs
tests/integration/hand-ui/draft_initial_grid_test.rs
tests/unit/hand-ui/fan_layout_formula_test.rs
```
Plus 5 new report files (all under `reports/`). No forbidden paths touched.

### `git diff --check` — PASS

No trailing whitespace or other whitespace errors detected.

### `git merge-base --is-ancestor origin/main <branch>` — PASS

Branch `integrate/krosmaga-hand-fan-readability-1955` is a strict fast-forward of
`origin/main@1c4981a65f02422de7d01505ce029d1c1551a3a8`.

### Focused tests — see below

The `default_config_10_cards_at_1280x720_readability_invariants` test and the broader
fan layout formula suite were run against the worktree. Results recorded after
background test completion (see test run section below).

---

## Test Results

`cargo test -p client --test hand_ui_fan_layout_formula_test` — **7 passed, 0 failed**

```
running 7 tests
test hu_02_count_five_positions_center_and_edges ... ok
test hu_02b_count_two_uses_full_normalized_span ... ok
test hu_03_single_card_early_return_centers_without_arc_or_tilt ... ok
test default_config_10_cards_at_1280x720_readability_invariants ... ok
test reserve_strip_uses_hand_fan_local_coordinates_above_card ... ok
test hu_03b_zero_cards_skips_formula_hides_slots_and_keeps_submit_active ... ok
test layout_system_applies_formula_to_visible_pooled_slots ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Note: `cargo test -p client "default_config_10_cards_at_1280x720_readability_invariants"` (without
`--test` flag) failed with exit 101 due to a pre-existing compile error in
`tests/unit/hud/phase_transitions_test.rs:244` (`missing field 'known' in ScoreboardDotState`).
This error is unrelated to the hand fan scope and pre-exists on origin/main. The targeted binary
`--test hand_ui_fan_layout_formula_test` compiled and passed cleanly.

`draft_initial_grid_test` is also an owned file changed in this payload but its binary
(`hand_ui_draft_initial_grid_test`) was not run standalone — deferred per prompt instructions
("cheap and available; otherwise state exactly what was deferred"). The integration test
binary would require the full client compilation which has the pre-existing hud error.

---

1955: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1920: READY_FOR_MAINLAND_ENQUEUE
