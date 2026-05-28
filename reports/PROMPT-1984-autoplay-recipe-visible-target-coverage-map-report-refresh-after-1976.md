# PROMPT 1984 — Autoplay Recipe Visible-Target Coverage Map — Refresh After PROMPT 1976

**Date**: 2026-05-28
**Branch**: `report/autoplay-recipe-visible-target-coverage-map-1984-r2`
**Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-1984`
**Base commit**: `32ca23e8` (origin/main, PROMPT 1988)
**Source-of-truth main**: origin/main at `32ca23e8` (PROMPT 1988 tier-border backfill after 1985)
**Recovers**: PROMPT 1967 (NOT_FF against current main — stale branch rooted at `2bf3960d` before
  PROMPT 1959/1972/1976/1985/1988 landed)
**Worker**: Claude Code — PROMPT 1984 report-only refresh (r2 after main advanced past r1)

---

## 1. Recovery Context

PROMPT 1967 (`report/autoplay-recipe-visible-target-coverage-map-1967`) produced five
report files (PROMPT 1848 + 1909 + 1924 + 1949 backfills, plus PROMPT 1967 refresh) and
reported `READY_FOR_MAINLAND_ENQUEUE`. However, the orchestrator rejected the branch because:

1. **NOT strict-FF** against current `origin/main`.
2. The stale 1967 branch is rooted at `2bf3960d` (PROMPT 1957) and would **delete or
   not include** reports that have since landed on main (PROMPT 1959, 1972, 1976, 1985, 1988).
3. A wholesale cherry-pick or merge of the stale branch would bring missing commits
   and risk clobbering the current report chain.

A first refresh attempt (r1, branch `report/autoplay-recipe-visible-target-coverage-map-1984`)
was committed and pushed rooted at `32a59256` (PROMPT 1976), but origin/main advanced again
before the branch could be enqueued for mainland (PROMPT 1985 and PROMPT 1988 landed),
making r1 NOT_FF with 7 deletes. This r2 branch is rooted at the new tip `32ca23e8`.

This worker:
1. Creates a clean branch rooted at current `origin/main` @ `32ca23e8`.
2. Copies the PROMPT 1848, 1909, 1924, 1949, and 1967 report payloads from the stale
   remote commit (`39b03ca4`) via `git show` — no cherry-pick, no merge.
3. Adds this refresh report (PROMPT 1984) documenting what has changed in the autoplay
   visible-target fragility picture since the 1967 base.

**No code, tests, or tooling were modified.** Scope is report files only.

---

## 2. Files Added in This Branch

| File | Action | Source |
|---|---|---|
| `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` | Added (backfill) | stale `39b03ca4` via `git show` |
| `reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md` | Added (backfill) | stale `39b03ca4` via `git show` |
| `reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md` | Added (backfill) | stale `39b03ca4` via `git show` |
| `reports/PROMPT-1949-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1943.md` | Added (backfill) | stale `39b03ca4` via `git show` |
| `reports/PROMPT-1967-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1957.md` | Added (backfill) | stale `39b03ca4` via `git show` |
| `reports/PROMPT-1984-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1976.md` | Added (this file) | New |

All files added via direct file extraction (`git show <commit>:<path>`).

---

## 3. Delta Since PROMPT 1967 Base

The PROMPT 1967 report was based on `origin/main` at `2bf3960d` (PROMPT 1957 krosmaga
auction tier-border report). Five commits landed on main before the current `32ca23e8` base:

| PROMPT | Commit | Change | Autoplay fragility impact |
|---|---|---|---|
| 1959 | `7fc1706e` | Reapply krosmaga-ui-stage3 slices reports after 1920 mainland | None — report-only |
| 1972 | `7b259e91` | Reapply PROMPT 1841/1889/1911/1946/1956 signoff-pack reports after 1959 | None — report-only |
| 1976 | `32a59256` | Backfill 1861/1914/1941/1964/1968 operator contract + refresh after 1972 | None — report-only |
| 1985 | `b354bee6` | Bot/autoplay story readiness report refresh after 1976 | None — report-only (story readiness, not recipe/driver) |
| 1988 | `32ca23e8` | Reapply PROMPT 1933/1961/1974/1986 tier-border reports after 1985 | None — report-only |

**No changes to `tools/autoplay/recipes/`, `tools/autoplay/driver.py`,
`client/src/autoplay.rs`, or any other autoplay-relevant source file landed
between PROMPT 1957 and PROMPT 1988.**

The autoplay fragility register is **unchanged** from PROMPT 1967.

---

## 4. Autoplay Fragility Status (Unchanged from PROMPT 1967)

### FRAG-01 — Bottom-strip clicks at fy=0.92

**Status: LOW (unchanged from 1967)**

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

| FRAG | Risk (1848) | Risk (1924) | Risk (1949) | Risk (1967) | Risk (1984) | Status |
|---|---|---|---|---|---|---|
| FRAG-01 bottom-strip fy=0.92 | CRITICAL | LOW | LOW | LOW | **LOW** | Unchanged — R-01 coords fix still open |
| FRAG-02 Add Bot no visibility proof | HIGH | HIGH | HIGH | HIGH | **HIGH** | Unchanged — open |
| FRAG-03 time-based phase waits | HIGH | HIGH | HIGH | HIGH | **HIGH** | Unchanged — R-02 poll_phase still needed |
| FRAG-04 fy=0.85 confirm cluster | MEDIUM-HIGH | MEDIUM-HIGH | MEDIUM-HIGH | MEDIUM-HIGH | **MEDIUM-HIGH** | Unchanged — coupled to FRAG-03 |
| FRAG-05 driver.py fallback silent | LOW | LOW | LOW | LOW | **LOW** | Unchanged — R-03 still open |
| FRAG-06 window size read once | LOW | NONE | NONE | NONE | **NONE** | Unchanged — mitigated |

---

## 6. Open Repair Items (Inherited from PROMPT 1848, confirmed open at 1984)

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
```

Six adds, no deletes, no modifications to any non-report file.

```
git merge-base --is-ancestor origin/main HEAD
# exit 0 — strict-FF requirement satisfied

git diff --check origin/main..HEAD
# (no output) — no trailing whitespace or conflict markers
```

---

## 8. Branch and Commit

- **Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-1984`
- **Branch**: `report/autoplay-recipe-visible-target-coverage-map-1984-r2`
- **Base**: `origin/main` @ `32ca23e8` (PROMPT 1988)
- **Source reports from**: stale commit `39b03ca4` (`origin/report/autoplay-recipe-visible-target-coverage-map-1967`) via `git show`

---

1984: AUTOPLAY-RECIPE-VISIBLE-TARGET-COVERAGE-MAP-REPORT-REFRESH-AFTER-1976: READY_FOR_MAINLAND_ENQUEUE
