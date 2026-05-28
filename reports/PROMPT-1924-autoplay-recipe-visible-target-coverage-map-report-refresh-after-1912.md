# PROMPT 1924 — Autoplay Recipe Visible-Target Coverage Map — Refresh After PROMPT 1912

**Date**: 2026-05-28
**Branch**: `report/autoplay-recipe-visible-target-coverage-map-1924`
**Worktree**: `D:\tmp\wt-1924-coverage-report`
**Base commit**: `2ce3dc6b` (origin/main, PROMPT 1872)
**Source-of-truth main**: origin/main at/after `1c945fd2` (PROMPT 1912 whitespace cleanup)
**Recovers**: PROMPT 1909 (NOT_FF against current main — stale branch deleted from remote)
**Worker**: Claude Code — PROMPT 1924 report-only refresh

---

## 1. Recovery Context

PROMPT 1909 (`report/autoplay-recipe-visible-target-coverage-map-1909`) produced two
report files (PROMPT 1848 backfill + PROMPT 1909 recovery) but the branch was never
merged to `origin/main`. A subsequent mainland push (`1c945fd2`, PROMPT 1912 whitespace
cleanup) made the stale branch NOT_FF, requiring a clean reapply.

This worker:
1. Copies the PROMPT 1848 and PROMPT 1909 report payloads from the stale local branch
   (accessible via `git show`) onto the current `origin/main`.
2. Adds this refresh report (PROMPT 1924) documenting what has changed in the autoplay
   visible-target fragility picture since the 1909 base.

**No code, tests, or tooling were modified.** Scope is report files only.

---

## 2. Files Added in This Branch

| File | Action | Source |
|---|---|---|
| `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` | Added (backfill) | `report/autoplay-recipe-visible-target-coverage-map-1909` stale branch |
| `reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md` | Added (backfill) | `report/autoplay-recipe-visible-target-coverage-map-1909` stale branch |
| `reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md` | Added (this file) | New |

All files added with `git add -f` (reports/ is gitignored).

---

## 3. Autoplay Fragility Status After PROMPT 1912

The PROMPT 1848 report identified 6 fragility classes (FRAG-01 through FRAG-06).
Three relevant PLOMPTs have landed on main since the 1848/1909 baseline:

| PROMPT | Commits | Change |
|---|---|---|
| 1880 / 1894 | `e8a40f81`, `71484fc4` | driver.py viewport guard: 4-layer abort system |
| 1912 | `e02d132f`, `fe2a9e88`, `1c945fd2` | autoplay.rs `enforce_autoplay_window_size_system` + Run-AutoplaySmoke.ps1 env defaults |
| 1916 | `96d2ee97` | Focused verify: 66/66 tests pass on `1c945fd2` |

---

### FRAG-01 — CRITICAL: fy=0.92 bottom-strip clicks

**Original**: `HAND_FIRST_CARD (0.35, 0.92)` → (448, 662) and
`SUBMIT_BTN (0.85, 0.92)` → (1088, 662) at 1280×720. Only 58px from bottom.

**Status after PROMPT 1912: PARTIALLY MITIGATED**

Two independent enforcement layers now prevent sub-nominal window launches:

1. **PROMPT 1880/1894 driver guard** — `_check_window_minimum()` in `driver.py`
   aborts with exit code 5 (`EXIT_VIEWPORT_GUARD`) before the recipe is built
   if `window_logical_size` is missing or below 1280×720.

2. **PROMPT 1912 Bevy Startup system** — `enforce_autoplay_window_size_system` in
   `client/src/autoplay.rs` sets `window.resolution.set(new_w, new_h)` at Startup
   to enforce `max(current, 1280×720)`. Reads `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT`
   env vars (defaults to 1280×720 if unset). `Run-AutoplaySmoke.ps1` sets these env
   vars defensively before the cargo launch.

**Residual exposure**: `_coords.py` still has `FracPoint(0.35, 0.92)` and
`FracPoint(0.85, 0.92)`. At exactly 720px height, y=662 leaves 58px of margin. The
R-01 repair (lower fy from 0.92 → 0.88, +29px headroom) is still open. With the floor
enforced, this is now a LOW-priority cosmetic hardening rather than a CRITICAL
live-failure path.

**Recommended next action**: R-01 remains valid and low-risk; assign to the next
`_coords.py` owner.

---

### FRAG-02 — HIGH: Add Bot button (debug-only, no visibility proof)

**Status: UNCHANGED — open**

No change to `add-bot-lobby` recipe or `LOBBY_ADD_BOT_BTN (0.5, 0.72)` coordinate.
The env-guard (`CCGS_DEBUG_UI=1`) and the `CCGS_AUTOPLAY_LOBBY_ADD_BOT_BTN` override
path are unchanged. Measurement protocol documentation (R-04) not yet written.

---

### FRAG-03 — HIGH: Time-based phase waits (no phase_label polling)

**Status: UNCHANGED — open**

`autoplay/status` still exposes `phase_label` and `client_state_label`; recipes
still use static `wait(N)` counts without polling the label. The PROMPT 1880 drift
guard (AC-VPT-02) aborts on window-size drift mid-run but does NOT add phase-label
gating between recipe steps. R-02 (`poll_phase` pseudo-action in `_builder.py` /
`driver.py`) is still the recommended repair.

---

### FRAG-04 — MEDIUM-HIGH: fy=0.85 confirm/ready cluster

**Status: UNCHANGED — coupled to FRAG-03**

All four CTAs (`LOBBY_CONFIRM_BTN`, `CLASS_CONFIRM_BTN`, `SHOP_CONFIRM_BTN`,
`AUCTION_READY_BTN`) still share `FracPoint(0.5, 0.85)` → (640, 612). The fix
remains: resolve FRAG-03 first with phase-label polling so any mismatched overlay
is rejected before the click fires.

---

### FRAG-05 — LOW: window_logical_size fallback with no warning in driver.py

**Status: PARTIALLY IMPROVED — driver warning still open**

PROMPT 1912's `enforce_autoplay_window_size_system` logs the applied window dimensions
(`tracing::info!` in Rust at the Startup boundary). This provides observability on the
Bevy side when the fallback fires. The Python driver.py fallback path (lines 226–229,
silent fall-back to `[1280.0, 720.0]`) still produces no warning log.

R-03 (one-line `log()` call in `driver.py` for the fallback path) is still open.

---

### FRAG-06 — LOW: window_logical_size read once at recipe-build time

**Status: MITIGATED by AC-VPT-02**

PROMPT 1880's mid-run drift guard (`_check_window_drift()`) re-polls
`autoplay/status` every tick after recipe init and aborts with exit code 5 if the
window drifts more than `_WIN_DRIFT_PX = 10.0px` from the build-time snapshot. The
original FRAG-06 risk (stale coordinates after resize) is now handled by aborting the
run rather than silently proceeding with wrong coordinates. No further repair needed.

---

## 4. Updated Fragility Register Summary

| FRAG | Risk (1848) | Risk (1924) | Status |
|---|---|---|---|
| FRAG-01 bottom-strip fy=0.92 | CRITICAL | **LOW** | Mitigated by 1880 + 1912 window floor; R-01 coords fix still open |
| FRAG-02 Add Bot no visibility proof | HIGH | HIGH | Open — no change |
| FRAG-03 time-based phase waits | HIGH | HIGH | Open — R-02 poll_phase still needed |
| FRAG-04 fy=0.85 confirm cluster | MEDIUM-HIGH | MEDIUM-HIGH | Open — coupled to FRAG-03 |
| FRAG-05 driver.py fallback silent | LOW | LOW | Partially improved by Rust log; driver.py R-03 still open |
| FRAG-06 window size read once | LOW | **NONE** | Mitigated by AC-VPT-02 abort-on-drift |

---

## 5. Open Repair Items (Inherited from PROMPT 1848)

| ID | Fragility | Priority | File(s) | Change | Status |
|---|---|---|---|---|---|
| R-01 | FRAG-01 | LOW (was CRITICAL) | `tools/autoplay/recipes/_coords.py` | Lower `HAND_FIRST_CARD` and `SUBMIT_BTN` fy from 0.92 → 0.88 | Open |
| R-02 | FRAG-03 | HIGH | `tools/autoplay/recipes/_builder.py`, `tools/autoplay/driver.py` | `poll_phase(label, max_ticks)` pseudo-action | Open |
| R-03 | FRAG-05 | LOW | `tools/autoplay/driver.py` | Add `log()` warning on `window_logical_size` fallback path | Open |
| R-04 | FRAG-02 | MEDIUM | `docs/autoplay.md` | Document Add Bot button coordinate measurement protocol | Open |

---

## 6. Validation

```
git merge-base --is-ancestor origin/main HEAD
# exit 0 — passes

git diff --check origin/main..HEAD
# (no output) — passes

git diff --name-status origin/main..HEAD
A  reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md
A  reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md
A  reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md
```

Three adds, no deletes, no modifications to any non-report file. Strict-FF-ready.

---

## 7. Branch and Commit

- **Worktree**: `D:\tmp\wt-1924-coverage-report`
- **Branch**: `report/autoplay-recipe-visible-target-coverage-map-1924`
- **Base**: `origin/main` @ `2ce3dc6b` (PROMPT 1872)
- **Source reports from**: stale local branch `report/autoplay-recipe-visible-target-coverage-map-1909`

---

1924: AUTOPLAY-RECIPE-VISIBLE-TARGET-COVERAGE-MAP-REPORT-REFRESH-AFTER-1912: SHIPPED
