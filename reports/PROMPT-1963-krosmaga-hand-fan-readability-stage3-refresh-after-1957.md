# PROMPT 1963 — Krosmaga Hand Fan Readability Stage3-D Refresh After PROMPT 1957

**Date:** 2026-05-28
**Branch:** integrate/krosmaga-hand-fan-readability-1963
**Base:** origin/main @ 2bf3960def7a1e19c4157051c5e356bca13377f5 (PROMPT 1957)
**Prior ref:** origin/integrate/krosmaga-hand-fan-readability-1955 @ ed406d11 (stale — not strict-FF vs current main after 1957 landed)

---

## Context

PROMPT 1955 produced a clean Stage3-D payload on main@1c4981a6 (PROMPT 1920), but
origin/integrate/krosmaga-hand-fan-readability-1955 became stale: main advanced through
PROMPT 1957 (auction tier-border asset binding) without the hand fan branch being merged.
The old branch was not a strict fast-forward of the updated main. Additionally, the 1955
report carried trailing whitespace on its header lines that failed `git diff --check`.

This PROMPT rebuilds the identical Stage3-D payload cleanly from
origin/main@2bf3960def7a1e19c4157051c5e356bca13377f5 (PROMPT 1957) via clean cherry-pick
of the 1955 commit, with report whitespace corrected.

---

## Rebuild Strategy

PROMPT 1957 and PROMPT 1955 touch entirely disjoint file sets:

| PROMPT 1957 files | PROMPT 1955 files |
|---|---|
| `client/Cargo.toml` | `client/src/ui/hand/mod.rs` |
| `client/src/asset_wiring.rs` | `tests/integration/hand-ui/draft_initial_grid_test.rs` |
| `client/src/ui/shop_auction/mod.rs` | `tests/unit/hand-ui/fan_layout_formula_test.rs` |
| `tests/unit/asset_wiring/auction_tier_border_asset_test.rs` | reports (5 files) |
| reports/PROMPT-1957-*.md | — |

No conflicts. Strategy: `git cherry-pick ed406d11` onto fresh worktree from origin/main,
then fix 1955 report whitespace and add the 1963 report.

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

### 4. Reports carried / updated

| Report file | Action |
|---|---|
| `PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` | Carried (untracked on main) |
| `PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` | Carried (untracked on main) |
| `PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md` | Carried (untracked on main) |
| `PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md` | Carried (untracked on main) |
| `PROMPT-1955-krosmaga-hand-fan-readability-stage3-refresh-after-1920.md` | Carried + trailing whitespace fixed (header lines 3-5) |
| `PROMPT-1963-krosmaga-hand-fan-readability-stage3-refresh-after-1957.md` | This report (new) |

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

`git diff --name-only origin/main..HEAD` returns exactly:
```
client/src/ui/hand/mod.rs
reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md
reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md
reports/PROMPT-1910-krosmaga-hand-fan-readability-stage3-refresh-after-1894.md
reports/PROMPT-1947-krosmaga-hand-fan-readability-stage3-refresh-after-1943.md
reports/PROMPT-1955-krosmaga-hand-fan-readability-stage3-refresh-after-1920.md
reports/PROMPT-1963-krosmaga-hand-fan-readability-stage3-refresh-after-1957.md
tests/integration/hand-ui/draft_initial_grid_test.rs
tests/unit/hand-ui/fan_layout_formula_test.rs
```
All paths are within owned scope. No forbidden paths touched.

### `git diff --check` — PASS

No trailing whitespace or other whitespace errors. Header whitespace in PROMPT-1955
report corrected (two trailing spaces on lines 3-5 stripped).

### `git merge-base --is-ancestor origin/main <branch>` — PASS

Branch `integrate/krosmaga-hand-fan-readability-1963` is a strict fast-forward of
`origin/main@2bf3960def7a1e19c4157051c5e356bca13377f5` (PROMPT 1957).

### Focused tests — PASS

Command: `cargo test --test hand_ui_fan_layout_formula_test -p client`
Working directory: `D:/Tmp/wt-1963-hand-fan-readability`

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

1963: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1957: READY_FOR_MAINLAND_ENQUEUE
