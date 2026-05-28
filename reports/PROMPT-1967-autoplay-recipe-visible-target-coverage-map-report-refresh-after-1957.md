# PROMPT 1967 — Autoplay Recipe Visible-Target Coverage Map — Refresh After PROMPT 1957

**Date**: 2026-05-28
**Branch**: `report/autoplay-recipe-visible-target-coverage-map-1967`
**Worktree**: `D:\tmp\wt-1967-coverage-report`
**Base commit**: `2bf3960d` (origin/main, PROMPT 1957)
**Source-of-truth main**: origin/main at `2bf3960d` (PROMPT 1957 krosmaga auction tier-border report)
**Recovers**: PROMPT 1949 (NOT_FF against current main — stale branch carried drift in client files,
  deleted already-landed PROMPT 1957 auction tier-border test/report)
**Worker**: Claude Code — PROMPT 1967 report-only refresh

---

## 1. Recovery Context

PROMPT 1949 (`report/autoplay-recipe-visible-target-coverage-map-1949`) produced four
report files (PROMPT 1848 + 1909 + 1924 backfills, plus PROMPT 1949 refresh) and reported
`READY_FOR_MAINLAND_ENQUEUE`. However, the orchestrator rejected the branch because:

1. **NOT strict-FF** against current `origin/main` (`2bf3960d`).
2. **Deletes already-landed reports** — the stale 1949 branch predates PROMPT 1957 and
   would have clobbered `reports/PROMPT-1957-krosmaga-auction-tier-border-asset-binding-refresh-after-1920.md`.
3. **Deletes PROMPT 1957 feat commit** — the stale branch did not include
   `449688dd` (PROMPT 1957 tier-border asset binding feat) or `1c4981a6` (PROMPT 1920
   card inspect hover glossary refresh).
4. **Carries drift in client files** — the old worktree checkout had stale
   `tools/dev-launcher/Start-TwoClients.ps1` edits included in the diff.

This worker:
1. Copies the PROMPT 1848, 1909, 1924, and 1949 report payloads from the stale remote
   branch (`origin/report/autoplay-recipe-visible-target-coverage-map-1949`) via
   `git show` onto the current `origin/main`.
2. Adds this refresh report (PROMPT 1967) documenting what has changed in the autoplay
   visible-target fragility picture since the 1949 base.

**No code, tests, or tooling were modified.** Scope is report files only.

---

## 2. Files Added in This Branch

| File | Action | Source |
|---|---|---|
| `reports/PROMPT-1848-autoplay-recipe-visible-target-coverage-map.md` | Added (backfill) | stale `origin/report/autoplay-recipe-visible-target-coverage-map-1949` via `git show` |
| `reports/PROMPT-1909-autoplay-recipe-visible-target-coverage-map-report-recovery.md` | Added (backfill) | stale `origin/report/autoplay-recipe-visible-target-coverage-map-1949` via `git show` |
| `reports/PROMPT-1924-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1912.md` | Added (backfill) | stale `origin/report/autoplay-recipe-visible-target-coverage-map-1949` via `git show` |
| `reports/PROMPT-1949-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1943.md` | Added (backfill) | stale `origin/report/autoplay-recipe-visible-target-coverage-map-1949` via `git show` |
| `reports/PROMPT-1967-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1957.md` | Added (this file) | New |

All files added with `git add -f` (reports/ is gitignored).

---

## 3. Delta Since PROMPT 1949 Base

The PROMPT 1949 report was based on `origin/main` at `e62c431e` (PROMPT 1943 two-client
retest backfill). Since then, seven commits landed on main before the current PROMPT 1957 base:

| PROMPT | Commit | Change | Autoplay fragility impact |
|---|---|---|---|
| 1937 | `b58cdd66` | QA snapshot observability gap report refresh | None — report-only |
| 1950 | `241e33a8` | Reapply PROMPT 1838/1862/1899/1932 autoplay tooling verify reports | None — report-only (tooling verify, not recipe/driver) |
| 1852 | `49aeb4f0` | Add keyword glossary definitions panel (UI feat) | None — UI, not autoplay |
| 1868 | `097a7b74` | Card inspect hover glossary report backfill | None — report-only |
| 1920 | `1c4981a6` | Card inspect hover glossary refresh report | None — report-only |
| 1957 feat | `449688dd` | Reapply PROMPT 1853 tier-border asset binding onto post-1920 main | None — auction UI, not autoplay recipes |
| 1957 report | `2bf3960d` | PROMPT 1957 tier-border asset binding refresh report | None — report-only |

**No changes to `tools/autoplay/recipes/`, `tools/autoplay/driver.py`,
`client/src/autoplay.rs`, or any other autoplay-relevant source file landed
between PROMPT 1943 and PROMPT 1957.**

The autoplay fragility register is **unchanged** from PROMPT 1949.

---

## 4. Autoplay Fragility Status (Unchanged from PROMPT 1949)

### FRAG-01 — Bottom-strip clicks at fy=0.92

**Status: LOW (unchanged from 1949)**

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

| FRAG | Risk (1848) | Risk (1924) | Risk (1949) | Risk (1967) | Status |
|---|---|---|---|---|---|
| FRAG-01 bottom-strip fy=0.92 | CRITICAL | LOW | LOW | **LOW** | Unchanged — R-01 coords fix still open |
| FRAG-02 Add Bot no visibility proof | HIGH | HIGH | HIGH | **HIGH** | Unchanged — open |
| FRAG-03 time-based phase waits | HIGH | HIGH | HIGH | **HIGH** | Unchanged — R-02 poll_phase still needed |
| FRAG-04 fy=0.85 confirm cluster | MEDIUM-HIGH | MEDIUM-HIGH | MEDIUM-HIGH | **MEDIUM-HIGH** | Unchanged — coupled to FRAG-03 |
| FRAG-05 driver.py fallback silent | LOW | LOW | LOW | **LOW** | Unchanged — R-03 still open |
| FRAG-06 window size read once | LOW | NONE | NONE | **NONE** | Unchanged — mitigated |

---

## 6. Open Repair Items (Inherited from PROMPT 1848, confirmed open at 1967)

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
```

Five adds, no deletes, no modifications to any non-report file.

```
git merge-base --is-ancestor origin/main HEAD
# exit 0 — strict-FF requirement satisfied

git diff --check origin/main..HEAD
# (no output) — no trailing whitespace or conflict markers
```

---

## 8. Branch and Commit

- **Worktree**: `D:\tmp\wt-1967-coverage-report`
- **Branch**: `report/autoplay-recipe-visible-target-coverage-map-1967`
- **Base**: `origin/main` @ `2bf3960d` (PROMPT 1957)
- **Source reports from**: stale remote branch `origin/report/autoplay-recipe-visible-target-coverage-map-1949` via `git show`

---

1967: AUTOPLAY-RECIPE-VISIBLE-TARGET-COVERAGE-MAP-REPORT-REFRESH-AFTER-1957: READY_FOR_MAINLAND_ENQUEUE
