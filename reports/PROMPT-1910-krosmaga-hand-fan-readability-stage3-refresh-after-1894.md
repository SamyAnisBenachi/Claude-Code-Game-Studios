# PROMPT 1910 — KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1894

**Branch:** `integrate/krosmaga-hand-fan-readability-1910`
**Commit:** `84d4213a4391a2218c9127f75b62034a9006f9c7`
**Date:** 2026-05-28
**Source branch:** `origin/prompt-1878-hand-fan-readability-refresh`
**Source commit:** `a389296c26f1b03951c87540045d02f88827e4bf`
**Base:** `origin/main@71484fc471d69966fe01de7e49890dbac5cdb79e` (PROMPT 1894)

---

## Context

PROMPT 1878 (`origin/prompt-1878-hand-fan-readability-refresh`) was not FF-ready against
current `origin/main` after PROMPT 1876, PROMPT 1856, and PROMPT 1894 landed. A direct
merge would have deleted reports/PROMPT-1856, PROMPT-1876, PROMPT-1880, PROMPT-1894,
reverted `tests/tools/autoplay/test_driver_click_viewport_guard.py`, reverted
`tools/autoplay/driver.py` viewport guard work, and reverted
`tools/dev-launcher/Start-AutoplayVsBot.ps1`.

This PROMPT reapplies only the 1878 payload onto the current `origin/main` tip.

---

## Files Changed

| Status | File |
|--------|------|
| M | `client/src/ui/hand/mod.rs` |
| A | `reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md` |
| A | `reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md` |
| M | `tests/integration/hand-ui/draft_initial_grid_test.rs` |
| M | `tests/unit/hand-ui/fan_layout_formula_test.rs` |

**5 files changed, 290 insertions(+), 5 deletions(-)**

No deletions. No changes to `tools/autoplay/**` or `tools/dev-launcher/**`.

---

## Payload Summary

**`client/src/ui/hand/mod.rs`**
- `HandFanLayoutConfig::default()`: `fan_base_margin_px` 100→150, `fan_half_spread_px`
  280→380, `arc_height_px` 20→30.
- Fixes ATK/HP bottom-clip and AR/HP badge occlusion at 1280×720 with a full 10-card hand.

**`tests/integration/hand-ui/draft_initial_grid_test.rs`**
- `qa_metrics()`: `fan_base_y` 160→110 (= 260 − 150 per the new default margin).
- Added comment: `PROMPT 1854 (STAGE3-D): fan_base_y updated 160→110`.

**`tests/unit/hand-ui/fan_layout_formula_test.rs`**
- Added `default_config_10_cards_at_1280x720_readability_invariants` test.
- Two invariants: (1) no card bottom clips below strip height; (2) adjacent card
  spacing > right-badge width.

**Reports**
- `PROMPT-1854` and `PROMPT-1878` reports carried forward from 1878 branch (unchanged content).

---

## FF Status

```
git merge-base --is-ancestor origin/main integrate/krosmaga-hand-fan-readability-1910
→ exit 0 (PASS — origin/main IS ancestor)
```

Branch is strict-FF-ready against `origin/main@71484fc4`.

---

## Validation

### Path allowlist check

```
git diff --name-status origin/main..integrate/krosmaga-hand-fan-readability-1910
M       client/src/ui/hand/mod.rs
A       reports/PROMPT-1854-krosmaga-hand-fan-readability-stage3-slice-d.md
A       reports/PROMPT-1878-krosmaga-hand-fan-readability-stage3-refresh-after-1872.md
M       tests/integration/hand-ui/draft_initial_grid_test.rs
M       tests/unit/hand-ui/fan_layout_formula_test.rs
```

- No deletes ✓
- No `tools/autoplay/**` changes ✓
- No `tools/dev-launcher/**` changes ✓
- No `reports/PROMPT-1856-*`, `PROMPT-1876-*`, `PROMPT-1880-*`, `PROMPT-1894-*` touched ✓

### Blob identity check

All 3 source file blobs match the PROMPT 1878 branch exactly (verified via `diff`):
- `client/src/ui/hand/mod.rs`: blob `42cb13cd` ✓
- `tests/integration/hand-ui/draft_initial_grid_test.rs`: blob `2738c9e1` ✓
- `tests/unit/hand-ui/fan_layout_formula_test.rs`: blob `09afc3c2` ✓

### Disk note

D: drive was 100% full during this task. Worktree was created with `--no-checkout`;
all file operations used `git hash-object -w`, `GIT_INDEX_FILE=C:/tmp/gcs-1910.index`
index operations, `git write-tree`, and `git commit-tree` to bypass the index.lock
constraint on D:. No working tree files were written to D:.

---

## Push

```
git push origin integrate/krosmaga-hand-fan-readability-1910
→ * [new branch] integrate/krosmaga-hand-fan-readability-1910 -> integrate/krosmaga-hand-fan-readability-1910
```

---

1910: KROSMAGA-HAND-FAN-READABILITY-STAGE3-REFRESH-AFTER-1894: SHIPPED
