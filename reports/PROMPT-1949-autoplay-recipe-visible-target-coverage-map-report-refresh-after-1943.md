# PROMPT 1949 — Autoplay Recipe Visible-Target Coverage Map — Refresh After PROMPT 1943

**Date**: 2026-05-28
**Branch**: `report/autoplay-recipe-visible-target-coverage-map-1949`
**Worktree**: `/tmp/wt-1949-coverage-report`
**Base commit**: `e62c431e` (origin/main, PROMPT 1943)
**Source-of-truth main**: origin/main at `e62c431e` (PROMPT 1943 two-client retest backfill)
**Recovers**: PROMPT 1924 (NOT_FF against current main — stale branch never merged)
**Worker**: Claude Code — PROMPT 1949 report-only refresh

---

## 1. Recovery Context

PROMPT 1924 (`report/autoplay-recipe-visible-target-coverage-map-1924`) produced three
report files (PROMPT 1848 backfill + PROMPT 1909 recovery + PROMPT 1924 refresh) but the
branch was never merged to `origin/main`. Two subsequent mainland pushes (PROMPT 1929
result-screen polish, PROMPT 1931 truth correction, PROMPT 1939 stale-binary guard,
PROMPT 1943 two-client retest) made the stale branch NOT_FF. The orchestrator rejected
the 1924 branch because it was NOT_FF, would have deleted already-landed reports, and
carried a stale `tools/dev-launcher/Start-TwoClients.ps1` change.

This worker:
1. Copies the PROMPT 1848, 1909, and 1924 report payloads from the stale local worktree
   (accessible via `git show 04e240a3`) onto the current `origin/main`.
2. Adds this refresh report (PROMPT 1949) documenting what has changed in the autoplay
   visible-target fragility picture since the 1924 base.

**No code, tests, or tooling were modified.** Scope is report files only.

---

## 2. Files Added in This Branch

| File | Action | Source |
|---|---|---|
| `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` | Added (backfill) | `git show 04e240a3` (stale 1924 branch commit) |
| `reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md` | Added (backfill) | `git show 04e240a3` (stale 1924 branch commit) |
| `reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md` | Added (backfill) | `git show 04e240a3` (stale 1924 branch commit) |
| `reports/PROMPT-1949-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1943.md` | Added (this file) | New |

All files added with `git add -f` (reports/ is gitignored).

---

## 3. Delta Since PROMPT 1924 Base

The PROMPT 1924 report was based on `origin/main` at `1c945fd2` (PROMPT 1912 whitespace
cleanup). Since then, four commits landed on main before the current PROMPT 1943 base:

| PROMPT | Commit | Change | Autoplay fragility impact |
|---|---|---|---|
| 1929 | `63f3b575` | Result screen chrome polish (SLICE-E reports) | None — UI-only reports |
| 1931 | `79031021` | Autoplay truth correction backfill (1831/1840 reports) | None — report-only |
| 1939 | `be40e0c6` | `Start-TwoClients.ps1` stale-binary rebuild guard | None — launcher tooling, not autoplay recipes |
| 1943 | `e62c431e` | Two-client full-flow retest report backfill (1883/1903) | None — report-only |

**No changes to `tools/autoplay/recipes/`, `tools/autoplay/driver.py`, `client/src/autoplay.rs`,
or any other autoplay-relevant source file landed between PROMPT 1912 and PROMPT 1943.**

The autoplay fragility register is **unchanged** from PROMPT 1924.

---

## 4. Autoplay Fragility Status (Unchanged from PROMPT 1924)

### FRAG-01 — Bottom-strip clicks at fy=0.92

**Status: LOW (unchanged from 1924)**

Two enforcement layers remain in place from PROMPT 1880/1894 (driver viewport guard) and
PROMPT 1912 (Bevy `enforce_autoplay_window_size_system`). The `_coords.py` values
`HAND_FIRST_CARD (0.35, 0.92)` and `SUBMIT_BTN (0.85, 0.92)` have not been updated.
R-01 (lower fy from 0.92 to 0.88) remains open as low-priority hardening.

---

### FRAG-02 — Add Bot button (debug-only, no visibility proof)

**Status: HIGH — UNCHANGED — open**

`add-bot-lobby` recipe and `LOBBY_ADD_BOT_BTN (0.5, 0.72)` coordinate are unchanged.
Measurement protocol documentation (R-04) not yet written.

---

### FRAG-03 — Time-based phase waits (no phase_label polling)

**Status: HIGH — UNCHANGED — open**

`autoplay/status` still exposes `phase_label`; recipes still use static `wait(N)`.
R-02 (`poll_phase` pseudo-action in `_builder.py` / `driver.py`) still needed.

---

### FRAG-04 — fy=0.85 confirm/ready cluster

**Status: MEDIUM-HIGH — UNCHANGED — coupled to FRAG-03**

All four CTAs still share `FracPoint(0.5, 0.85)`. Coupled fix remains: resolve FRAG-03
first.

---

### FRAG-05 — driver.py fallback silent

**Status: LOW — UNCHANGED — partially improved**

Rust-side logging from PROMPT 1912 `enforce_autoplay_window_size_system` still the only
observability. Python driver.py fallback path (lines 226-229) still produces no warning.
R-03 (one-line `log()` call) still open.

---

### FRAG-06 — window_logical_size read once at recipe-build time

**Status: NONE — UNCHANGED — mitigated by AC-VPT-02**

PROMPT 1880 mid-run drift guard (`_check_window_drift()`) still in place. No further
repair needed.

---

## 5. Updated Fragility Register Summary

| FRAG | Risk (1848) | Risk (1924) | Risk (1949) | Status |
|---|---|---|---|---|
| FRAG-01 bottom-strip fy=0.92 | CRITICAL | LOW | **LOW** | Unchanged — R-01 coords fix still open |
| FRAG-02 Add Bot no visibility proof | HIGH | HIGH | **HIGH** | Unchanged — open |
| FRAG-03 time-based phase waits | HIGH | HIGH | **HIGH** | Unchanged — R-02 poll_phase still needed |
| FRAG-04 fy=0.85 confirm cluster | MEDIUM-HIGH | MEDIUM-HIGH | **MEDIUM-HIGH** | Unchanged — coupled to FRAG-03 |
| FRAG-05 driver.py fallback silent | LOW | LOW | **LOW** | Unchanged — R-03 still open |
| FRAG-06 window size read once | LOW | NONE | **NONE** | Unchanged — mitigated |

---

## 6. Open Repair Items (Inherited from PROMPT 1848, confirmed open at 1949)

| ID | Fragility | Priority | File(s) | Change | Status |
|---|---|---|---|---|---|
| R-01 | FRAG-01 | LOW | `tools/autoplay/recipes/_coords.py` | Lower `HAND_FIRST_CARD` and `SUBMIT_BTN` fy from 0.92 to 0.88 | Open |
| R-02 | FRAG-03 | HIGH | `tools/autoplay/recipes/_builder.py`, `tools/autoplay/driver.py` | `poll_phase(label, max_ticks)` pseudo-action | Open |
| R-03 | FRAG-05 | LOW | `tools/autoplay/driver.py` | Add `log()` warning on `window_logical_size` fallback path | Open |
| R-04 | FRAG-02 | MEDIUM | `docs/autoplay.md` | Document Add Bot button coordinate measurement protocol | Open |

---

## 7. Validation

```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md
A  reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md
A  reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md
A  reports/PROMPT-1949-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1943.md
```

Four adds, no deletes, no modifications to any non-report file.

```
git merge-base --is-ancestor origin/main HEAD
# exit 0 -- strict-FF requirement satisfied

git diff --check origin/main..HEAD
# (no output) -- no trailing whitespace or conflict markers
```

---

## 8. Branch and Commit

- **Worktree**: `/tmp/wt-1949-coverage-report`
- **Branch**: `report/autoplay-recipe-visible-target-coverage-map-1949`
- **Base**: `origin/main` @ `e62c431e` (PROMPT 1943)
- **Source reports from**: stale local commit `04e240a3` (1924 branch)

---

1949: AUTOPLAY-RECIPE-VISIBLE-TARGET-COVERAGE-MAP-REPORT-REFRESH-AFTER-1943: READY_FOR_MAINLAND_ENQUEUE
