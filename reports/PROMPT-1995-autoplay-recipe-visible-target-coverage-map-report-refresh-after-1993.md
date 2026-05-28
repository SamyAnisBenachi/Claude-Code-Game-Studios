# PROMPT 1995 — Autoplay Recipe Visible-Target Coverage Map — Refresh After PROMPT 1993

**Date**: 2026-05-28
**Branch**: `report/autoplay-recipe-visible-target-coverage-map-1995`
**Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-1995-reports`
**Base commit**: `56839ef1` (origin/main, PROMPT 1993)
**Source-of-truth main**: origin/main at `56839ef1` (PROMPT 1993 game-completion next-wave map refresh after 1991)
**Recovers**: PROMPT 1984 r2 (NOT_FF against current main — stale branch rooted at `32ca23e8` before
  PROMPT 1991/1993 landed; r2 would delete hand-fan reports 1854/1878/1910/1947/1955/1963/1981/1991
  plus game-completion reports 1978/1993, and also modified client/src/ui/hand/mod.rs and hand UI tests)
**Worker**: Claude Code — PROMPT 1995 report-only refresh

---

## 1. Recovery Context

PROMPT 1984 r2 (`report/autoplay-recipe-visible-target-coverage-map-1984-r2`) produced six
report files (PROMPT 1848 + 1909 + 1924 + 1949 + 1967 backfills, plus PROMPT 1984 refresh)
and reported `READY_FOR_MAINLAND_ENQUEUE`. However, the orchestrator rejected the branch because:

1. **NOT strict-FF** against current `origin/main`.
2. The stale 1984-r2 branch is rooted at `32ca23e8` (PROMPT 1988) and would **delete or not
   include** reports that have since landed on main (PROMPT 1991, 1993).
3. The r2 branch also carries modifications to `client/src/ui/hand/mod.rs` and hand UI test
   files — these must not be reused wholesale; report content only is safe to carry forward.

This worker:
1. Creates a clean branch rooted at current `origin/main` @ `56839ef1`.
2. Copies the PROMPT 1848, 1909, 1924, 1949, 1967, and 1984 report payloads from the stale
   remote branch (`origin/report/autoplay-recipe-visible-target-coverage-map-1984-r2`) via
   `git show` — no cherry-pick, no merge, no source/test file carry-over.
3. Adds this refresh report (PROMPT 1995) documenting what has changed in the autoplay
   visible-target fragility picture since the 1984-r2 base.

**No code, tests, or tooling were modified.** Scope is report files only.

---

## 2. Files Added in This Branch

| File | Action | Source |
|---|---|---|
| `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` | Added (backfill) | stale r2 branch via `git show` |
| `reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md` | Added (backfill) | stale r2 branch via `git show` |
| `reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md` | Added (backfill) | stale r2 branch via `git show` |
| `reports/PROMPT-1949-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1943.md` | Added (backfill) | stale r2 branch via `git show` |
| `reports/PROMPT-1967-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1957.md` | Added (backfill) | stale r2 branch via `git show` |
| `reports/PROMPT-1984-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1976.md` | Added (backfill) | stale r2 branch via `git show` |
| `reports/PROMPT-1995-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1993.md` | Added (this file) | New |

All files added via direct file extraction (`git show <branch>:<path>`).

---

## 3. Delta Since PROMPT 1984 Base

The PROMPT 1984-r2 report was based on `origin/main` at `32ca23e8` (PROMPT 1988 tier-border
backfill). Two commits landed on main before the current `56839ef1` base:

| PROMPT | Commit | Change | Autoplay fragility impact |
|---|---|---|---|
| 1991 | `17b68aac` | Reapply hand fan readability Stage3-D on post-1988 main (`client/src/ui/hand/mod.rs`, hand UI tests + reports) | None — hand UI layout only; no autoplay recipe/driver changes |
| 1993 | `56839ef1` | Reapply game-completion next-wave map PROMPT 1978 report after 1991 mainland | None — report-only |

**No changes to `tools/autoplay/recipes/`, `tools/autoplay/driver.py`,
`client/src/autoplay.rs`, or any other autoplay-relevant source file landed
between PROMPT 1988 and PROMPT 1993.**

The autoplay fragility register is **unchanged** from PROMPT 1984.

---

## 4. Autoplay Fragility Status (Unchanged from PROMPT 1984)

### FRAG-01 — Bottom-strip clicks at fy=0.92

**Status: LOW (unchanged from 1984)**

Two enforcement layers remain in place from PROMPT 1880/1894 (driver viewport guard,
`_check_window_minimum()`) and PROMPT 1912 (Bevy `enforce_autoplay_window_size_system`
in `client/src/autoplay.rs`). The `_coords.py` values `HAND_FIRST_CARD (0.35, 0.92)`
and `SUBMIT_BTN (0.85, 0.92)` have not been updated. R-01 (lower fy from 0.92 to 0.88,
+29px headroom) remains open as low-priority hardening.

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

All four CTAs (`LOBBY_CONFIRM_BTN`, `CLASS_CONFIRM_BTN`, `SHOP_CONFIRM_BTN`,
`AUCTION_READY_BTN`) still share `FracPoint(0.5, 0.85)`. Coupled fix remains:
resolve FRAG-03 first.

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

| FRAG | Risk (1848) | Risk (1924) | Risk (1949) | Risk (1967) | Risk (1984) | Risk (1995) | Status |
|---|---|---|---|---|---|---|---|
| FRAG-01 bottom-strip fy=0.92 | CRITICAL | LOW | LOW | LOW | LOW | **LOW** | Unchanged — R-01 coords fix still open |
| FRAG-02 Add Bot no visibility proof | HIGH | HIGH | HIGH | HIGH | HIGH | **HIGH** | Unchanged — open |
| FRAG-03 time-based phase waits | HIGH | HIGH | HIGH | HIGH | HIGH | **HIGH** | Unchanged — R-02 poll_phase still needed |
| FRAG-04 fy=0.85 confirm cluster | MEDIUM-HIGH | MEDIUM-HIGH | MEDIUM-HIGH | MEDIUM-HIGH | MEDIUM-HIGH | **MEDIUM-HIGH** | Unchanged — coupled to FRAG-03 |
| FRAG-05 driver.py fallback silent | LOW | LOW | LOW | LOW | LOW | **LOW** | Unchanged — R-03 still open |
| FRAG-06 window size read once | LOW | NONE | NONE | NONE | NONE | **NONE** | Unchanged — mitigated |

---

## 6. Open Repair Items (Inherited from PROMPT 1848, confirmed open at 1995)

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
A  reports/PROMPT-1967-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1957.md
A  reports/PROMPT-1984-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1976.md
A  reports/PROMPT-1995-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1993.md
```

Seven adds, no deletes, no modifications to any non-report file.

```
git merge-base --is-ancestor origin/main HEAD
# exit 0 — strict-FF requirement satisfied

git diff --check origin/main..HEAD
# (no output) — no trailing whitespace or conflict markers
```

---

## 8. Branch and Commit

- **Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-1995-reports`
- **Branch**: `report/autoplay-recipe-visible-target-coverage-map-1995`
- **Base**: `origin/main` @ `56839ef1` (PROMPT 1993)
- **Source reports from**: `origin/report/autoplay-recipe-visible-target-coverage-map-1984-r2` via `git show`

---

1995: AUTOPLAY-RECIPE-VISIBLE-TARGET-COVERAGE-MAP-REPORT-REFRESH-AFTER-1993: READY_FOR_MAINLAND_ENQUEUE
