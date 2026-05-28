# PROMPT 1981 — Krosmaga Hand Fan Readability Stage3-D Refresh After PROMPT 1976

**Date:** 2026-05-28
**Branch:** work/PROMPT-1981
**Base:** origin/main @ 32a59256d1de9a4fee362a2aa9006d1bb69b59db (PROMPT 1976)
**Prior ref:** origin/integrate/krosmaga-hand-fan-readability-1963 @ 80fc4687671fcf6a40f15cfb13183d192ccba358 (stale — NOT_FF vs current origin/main; would delete 1959/1972/1976 report chains)

---

## Context

PROMPT 1963 produced a clean Stage3-D payload and reported READY_FOR_MAINLAND_ENQUEUE, but
the orchestrator rejected it because origin/main advanced through PROMPT 1959, 1972, and 1976
(report-only commits) without the hand fan branch being merged. The stale 1963 branch tip
(`80fc4687`) shows 20+ D-entries (deleted reports) relative to current origin/main — wholesale
cherry-pick would destroy the current main report chains.

This PROMPT reapplies the identical Stage3-D payload via file-level transplant from the 1963
branch working tree, without carrying any stale deletes.

---

## Rebuild Strategy

origin/main advanced via 3 report-only commits since the 1963 branch base (2bf3960d):
- PROMPT 1959: krosmaga-ui-stage3 slices report backfill
- PROMPT 1972: autoplay vsbot 1841 signoff-pack reports
- PROMPT 1976: autoplay vsbot window-size operator contract reports

None of these commits touch any hand fan owned files. Strategy: file-level transplant of
the 1963 branch content for owned files only, skipping all stale D-entries.

---

## Scope Applied

### 1. `client/src/ui/hand/mod.rs` — `HandFanLayoutConfig::default()` tuning

| Field | Before | After | Reason |
|---|---|---|---|
| `fan_base_margin_px` | 100.0 | 150.0 | Lifts card strip so ATK/HP badges clear viewport edge at 1280×720 |
| `fan_half_spread_px` | 280.0 | 380.0 | Widens fan arc so 10 cards don't overlap right-side AR/HP badges |
| `arc_height_px` | 20.0 | 30.0 | Increases arc curvature to match wider spread |

These values were established and validated in PROMPT 1854 (original Stage3-D slice).

### 2. `tests/integration/hand-ui/draft_initial_grid_test.rs` — `qa_metrics()` update

- `fan_base_y`: 160.0 → 110.0 (= 260 − 150, tracking updated `fan_base_margin_px = 150`)
- Comment updated: `260 - 100 = 160` → `260 - 150 = 110`
- Added PROMPT 1854 Stage3-D note referencing the fan_base_y update.

### 3. `tests/unit/hand-ui/fan_layout_formula_test.rs` — Stage3-D readability invariants

Added `default_config_10_cards_at_1280x720_readability_invariants` test which asserts:
1. **No bottom clip**: `card_y + CARD_H ≤ HAND_FAN_STRIP_HEIGHT_PX` for every slot — ensures ATK/HP badges never clip below the strip edge.
2. **Right-badge visible**: spacing between adjacent card left edges > `RIGHT_BADGE_W` (108px × 24% ≈ 25.92px) — ensures AR/HP badges are not occluded by the neighboring card.

### 4. Reports carried (all untracked on origin/main)

| Report file | Action |
|---|---|
| `PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` | Carried (untracked on main) |
| `PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` | Carried (untracked on main) |
| `PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md` | Carried (untracked on main) |
| `PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md` | Carried (untracked on main) |
| `PROMPT-1955-krosmaga-hand-fan-readability-stage3-refresh-after-1920.md` | Carried (untracked on main) |
| `PROMPT-1963-krosmaga-hand-fan-readability-stage3-refresh-after-1957.md` | Carried (untracked on main) |
| `PROMPT-1981-krosmaga-hand-fan-readability-stage3-refresh-after-1976.md` | This report (new) |

---

## Files NOT Touched (preservation checklist)

| File | Status |
|---|---|
| `client/src/ui/card_inspect.rs` | Untouched — PROMPT 1920 glossary work preserved |
| `client/src/ui/hand/inspect.rs` | Untouched — PROMPT 1920 glossary work preserved |
| `client/src/asset_wiring.rs` | Untouched — PROMPT 1957 tier-border work preserved |
| `client/src/ui/shop_auction/mod.rs` | Untouched — PROMPT 1957 tier-border work preserved |
| `client/Cargo.toml` | Untouched |
| `tests/unit/asset_wiring/**` | Untouched — PROMPT 1957 test preserved |
| `production/**` | Untouched |
| `tools/**` | Untouched |
| `Cargo.lock` | Untouched |
| All existing main reports | None deleted or modified |

---

## Validation

### Path allowlist review — PASS

`git diff --name-status origin/main..HEAD` returns exactly:
```
M	client/src/ui/hand/mod.rs
A	reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md
A	reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md
A	reports/PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md
A	reports/PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md
A	reports/PROMPT-1955-krosmaga-hand-fan-readability-stage3-refresh-after-1920.md
A	reports/PROMPT-1963-krosmaga-hand-fan-readability-stage3-refresh-after-1957.md
A	reports/PROMPT-1981-krosmaga-hand-fan-readability-stage3-refresh-after-1976.md
M	tests/integration/hand-ui/draft_initial_grid_test.rs
M	tests/unit/hand-ui/fan_layout_formula_test.rs
```
All paths are within owned scope. Zero deletes. No forbidden paths touched.

### `git diff --check` — PASS

No trailing whitespace or other whitespace errors.

### `git merge-base --is-ancestor origin/main HEAD` — PASS

Branch `work/PROMPT-1981` is a strict fast-forward of
`origin/main@32a59256d1de9a4fee362a2aa9006d1bb69b59db` (PROMPT 1976).

### `git diff --name-status origin/main..HEAD` zero-delete check — PASS

All entries are M (modified) or A (added). Zero D (delete) entries.

### Focused tests — PASS

Command: `cargo test --test hand_ui_fan_layout_formula_test -p client`
Working directory: `D:/_DEV/Work/gcs-app-worktrees/lanesandlies/PROMPT-1981`

```
running 7 tests
test default_config_10_cards_at_1280x720_readability_invariants ... ok
test hu_02b_count_two_uses_full_normalized_span ... ok
test hu_03_single_card_early_return_centers_without_arc_or_tilt ... ok
test hu_02_count_five_positions_center_and_edges ... ok
test layout_system_applies_formula_to_visible_pooled_slots ... ok
test hu_03b_zero_cards_skips_formula_hides_slots_and_keeps_submit_active ... ok
test reserve_strip_uses_hand_fan_local_coordinates_above_card ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Note: full `cargo test -p client` deferred — pre-existing compile error in
`tests/unit/hud/phase_transitions_test.rs` (`missing field 'known' in ScoreboardDotState`)
unrelated to hand fan scope and pre-existing on origin/main.

---

1981: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1976: READY_FOR_MAINLAND_ENQUEUE
