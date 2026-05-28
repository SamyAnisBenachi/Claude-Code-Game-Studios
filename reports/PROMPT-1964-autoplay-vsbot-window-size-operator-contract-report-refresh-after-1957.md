# PROMPT 1964 — AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REPORT-REFRESH-AFTER-1957

**Type:** Report-only backfill — branch refresh
**Date:** 2026-05-28
**Status:** READY_FOR_MAINLAND_ENQUEUE
**Backfills:**
  - `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md`
  - `reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md`
  - `reports/PROMPT-1941-autoplay-vsbot-window-size-operator-contract-refresh-after-1931.md`
**Supersedes stale branch:** `origin/report/autoplay-vsbot-window-contract-1941` @ `143df7f8`
**Current refresh base:** `origin/main` @ `2bf3960d` (PROMPT 1957)
**Branch:** `report/autoplay-vsbot-window-contract-1964`

---

## Purpose

This report tracks the clean re-landing of the autoplay vs-bot window-size operator
contract report chain (PROMPTs 1861, 1914, 1941) onto current `origin/main@2bf3960d`.

### Refresh Chain Summary

| PROMPT | Role | Branch tip | Status |
|--------|------|-----------|--------|
| 1861 | Original operator contract reconcile (supersedes 1847) | `cee93efe` | Landed on 1964 branch |
| 1914 | Refresh of 1861 after PROMPTs 1856/1876/1894 landed | `b6a65b65` | Landed on 1964 branch |
| 1941 | Intended refresh after PROMPTs 1912/1931 landed | `143df7f8` | SUPERSEDED — branch was stale at NOT_FF |
| 1964 | This report — clean re-landing after PROMPT 1957 | `HEAD` | READY_FOR_MAINLAND_ENQUEUE |

### Why 1941 Was NOT Mergeable

Branch `origin/report/autoplay-vsbot-window-contract-1941` (`143df7f8`) was reported
READY_FOR_MAINLAND_ENQUEUE but was never merged. By PROMPT 1964, main had advanced to
`2bf3960d` and the 1941 branch was NOT_FF. A wholesale merge would have:

1. Deleted all reports from PROMPTs 1932, 1937, 1939, 1943, 1950, and 1957 that
   existed on main but not in the 1941 branch's tree.
2. Clobbered PROMPT 1957's krosmaga auction tier-border report and client changes.
3. Introduced stale drift in client files from the 1941 branch's history.

PROMPT 1964 rebuilds the backfill from scratch using `git worktree add` on
`origin/main@2bf3960d`, then writes only the four owned report files directly.

---

## What PROMPT 1861 Established (Key Truths — Still Valid)

PROMPT 1861 was an operator contract/runbook **correction** that superseded PROMPT 1847.
Its key findings remain valid through PROMPT 1957:

1. **None of the three 2026-05-28 autoplay runs is a clean automated PASS.**
   - `20260528-051148-Z` — PARTIAL (no pixel_hash data, Bevy RPC screenshots only)
   - `20260528-063609-Z` — PARTIAL (all 15 hashes frozen/identical; not usable as evidence)
   - `20260528-090613-Z` — PARTIAL (mid-run window resize + frozen PrintWindow)

2. **Run 090613-Z is conditional human-review evidence only** — disqualified by
   mid-run resize (720 → 505 → 1076 px at tick 115), stale coordinates post-resize,
   PrintWindow frozen 11/15, and `cursor_logical=None` at resize ticks.

3. **Window / viewport preflight remains required** per Section 4 of the 1861
   operator contract.

4. **AC-VPT-01..08** (PROMPT 1844 §8) define the minimum bar for a future clean
   automated PASS.

---

## AC-VPT Status at origin/main@2bf3960d (PROMPT 1957)

| AC | Requirement | Responsible | Status |
|---|---|---|---|
| AC-VPT-01 | Driver aborts if initial window < [1280, 720] | 1857 → 1880/1894 | **LANDED** |
| AC-VPT-02 | Driver detects mid-run resize > ±10 px; aborts/marks NEEDS_HUMAN_GUI | 1857 → 1880/1894 | **LANDED** |
| AC-VPT-03 | Driver warns/aborts if `cursor_logical=None` before `mouse_down` | 1857 → 1880/1894 | **LANDED** |
| AC-VPT-04 | Recipe rebuilt after resize | future | Not scheduled |
| AC-VPT-05 | Composite records `frozen_all`; verdict downgraded to `NEEDS_HUMAN_GUI` | 1850 | Not confirmed landed |
| AC-VPT-06 | ≥ 3 distinct hashes required for mechanical PASS | 1850 | Not confirmed landed |
| AC-VPT-07 | Composite records `initial_window_size`, `window_resize_events`; non-zero downgrades | 1850 | Not confirmed landed |
| AC-VPT-08 | Launcher sets `CCGS_WINDOW_WIDTH/HEIGHT=1280/720`; Rust enforces at Startup | 1842 → 1912 | **LANDED** |

No viewport guard or composite verdict PROMPTs landed between 1931 and 1957.
The AC-VPT table is unchanged from the 1941 refresh era; only AC-VPT-04 (future)
and AC-VPT-05/06/07 (PROMPT 1850, not yet confirmed) remain open.

---

## Files Changed vs origin/main

```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md
A  reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md
A  reports/PROMPT-1941-autoplay-vsbot-window-size-operator-contract-refresh-after-1931.md
A  reports/PROMPT-1964-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1957.md
```

No deletions. No changes to `tools/**`, `client/**`, `server/**`, `tests/**`,
`production/**`, `Cargo.*`, or any existing reports. PROMPT 1920 card inspect
reports and PROMPT 1957 auction tier-border report/test are preserved intact on main.

---

## Validation

### Path allowlist review — PASS

All four changed files are within the owned scope defined by PROMPT 1964:
- `reports/PROMPT-1861-*` ✓
- `reports/PROMPT-1914-*` ✓
- `reports/PROMPT-1941-*` ✓
- `reports/PROMPT-1964-*` ✓

No forbidden paths (`client/**`, `tests/**`, `tools/**`, `production/**`,
`Cargo.*`, stage/QA/session-state files, unrelated reports) touched.

### git diff --check — PASS

No trailing-whitespace errors detected in any of the four report files.

### FF-readiness — PASS

```
git merge-base --is-ancestor origin/main HEAD
→ exit 0
```

Branch `report/autoplay-vsbot-window-contract-1964` is a strict fast-forward
from `origin/main@2bf3960d`.

### No Cargo/Python tests required

Report-only payload; no test suite applicable.

---

## Branch / Commit Details

| Field | Value |
|-------|-------|
| Branch | `report/autoplay-vsbot-window-contract-1964` |
| Base (origin/main) | `2bf3960d` — PROMPT 1957 krosmaga auction tier-border |
| Stale predecessor | `origin/report/autoplay-vsbot-window-contract-1941` @ `143df7f8` (NOT_FF — superseded) |
| FF status | FF-READY |
| Commit message | `docs(reports): PROMPT 1964 — backfill 1861/1914/1941 operator contract + refresh after 1957` |

---

1964: AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REPORT-REFRESH-AFTER-1957: READY_FOR_MAINLAND_ENQUEUE
