# PROMPT 1976 — AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REPORT-REFRESH-AFTER-1972

**Type:** Report-only backfill — branch refresh
**Date:** 2026-05-28
**Status:** READY_FOR_MAINLAND_ENQUEUE
**Backfills:**
  - `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md`
  - `reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md`
  - `reports/PROMPT-1941-autoplay-vsbot-window-size-operator-contract-refresh-after-1931.md`
  - `reports/PROMPT-1964-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1957.md`
  - `reports/PROMPT-1968-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1959.md`
**Supersedes stale branch:** `origin/report/autoplay-vsbot-window-contract-1968` @ `c50f0323`
**Current refresh base:** `origin/main` @ `7b259e91` (PROMPT 1972)
**Branch:** `report/autoplay-vsbot-window-contract-1976`

---

## Purpose

PROMPT 1968 shipped the clean backfill of the autoplay vs-bot window-size operator
contract report chain (PROMPTs 1861, 1914, 1941, 1964) onto
`origin/main@7fc1706e` (PROMPT 1959) as branch
`origin/report/autoplay-vsbot-window-contract-1968` @ `c50f0323`.

Before that branch was merged, PROMPT 1972 landed on `origin/main`
(`7b259e91` — "reapply PROMPT 1841/1889/1911/1946/1956 signoff-pack reports after 1959").
This made the 1968 branch NOT fast-forward against current `origin/main`:
a wholesale merge of the stale 1968 branch would delete PROMPT 1972's six signoff-pack
reports that exist on main but not in the 1968 branch tree.

PROMPT 1976 rebuilds the backfill cleanly from `origin/main@7b259e91` (PROMPT 1972),
writing only the six owned report files into a fresh worktree branch.
No implementation files are touched.

---

## Refresh Chain Summary

| PROMPT | Role | Branch tip | Status |
|--------|------|-----------|--------|
| 1861 | Original operator contract reconcile (supersedes 1847) | `cee93efe` | Carried forward — landed on 1976 branch |
| 1914 | Refresh of 1861 after PROMPTs 1856/1876/1894 landed | `b6a65b65` | Carried forward — landed on 1976 branch |
| 1941 | Intended refresh after PROMPTs 1912/1931 landed | `143df7f8` | SUPERSEDED — carried forward as historical record |
| 1964 | Refresh after PROMPT 1957 | `e9955805` | SUPERSEDED — carried forward as historical record |
| 1968 | Refresh after PROMPT 1959 | `c50f0323` | SUPERSEDED — branch was NOT_FF against 7b259e91 |
| 1976 | This report — clean re-landing after PROMPT 1972 | `HEAD` | READY_FOR_MAINLAND_ENQUEUE |

### Why 1968 Was NOT Mergeable

Branch `origin/report/autoplay-vsbot-window-contract-1968` (`c50f0323`) was reported
READY_FOR_MAINLAND_ENQUEUE after PROMPT 1959, but was never merged to main before
PROMPT 1972 landed. PROMPT 1972 added six signoff-pack reports to main that do not
exist in the 1968 branch tree:

- `reports/PROMPT-1841-autoplay-vsbot-1831-evidence-signoff-pack.md`
- `reports/PROMPT-1889-autoplay-vsbot-1841-signoff-pack-refresh-after-1872.md`
- `reports/PROMPT-1911-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1894.md`
- `reports/PROMPT-1946-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1943.md`
- `reports/PROMPT-1956-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1920.md`
- `reports/PROMPT-1972-autoplay-vsbot-1841-signoff-pack-report-refresh-after-1959.md`

A wholesale merge of `origin/report/autoplay-vsbot-window-contract-1968` would have
deleted all six of these PROMPT 1972 signoff-pack reports that exist on main but not
in the 1968 branch tree.

PROMPT 1976 rebuilds from scratch using a new worktree from `origin/main@7b259e91`,
writing only the six owned report files directly.

---

## What PROMPT 1861 Established (Key Truths — Still Valid)

PROMPT 1861 was an operator contract/runbook **correction** that superseded PROMPT 1847.
Its key findings remain valid through PROMPT 1972:

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

## AC-VPT Status at origin/main@7b259e91 (PROMPT 1972)

PROMPT 1972 landed only signoff-pack report files. No viewport guard or composite
verdict code changes occurred between PROMPT 1959 and PROMPT 1972.
The AC-VPT status table is unchanged from PROMPT 1968.

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

AC-VPT-01/02/03/08 are confirmed landed on main.
AC-VPT-05/06/07 (PROMPT 1850 composite verdict downgrade) remain unconfirmed.
AC-VPT-04 (recipe rebuild after resize) is not scheduled.

**Implication:** Post-1894 driver will abort with `EXIT_VIEWPORT_GUARD=5` on the
failure modes seen in 090613-Z (mid-run resize, cursor-None click). The Rust Startup
system (post-1912) additionally enforces minimum window geometry at process start.
The remaining open gap is AC-VPT-05/06/07: until PROMPT 1850 is confirmed landed,
a composite verdict of PASS must be treated as unverified against the full bar.

---

## Files Changed vs origin/main

```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md
A  reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md
A  reports/PROMPT-1941-autoplay-vsbot-window-size-operator-contract-refresh-after-1931.md
A  reports/PROMPT-1964-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1957.md
A  reports/PROMPT-1968-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1959.md
A  reports/PROMPT-1976-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1972.md
```

No deletions. No changes to `tools/**`, `client/**`, `server/**`, `tests/**`,
`production/**`, `Cargo.*`, or any existing reports. PROMPT 1972 signoff-pack
reports are preserved intact on main.

---

## Validation

### Path allowlist review — PASS

All six changed files are within the owned scope defined by PROMPT 1976:
- `reports/PROMPT-1861-*` ✓
- `reports/PROMPT-1914-*` ✓
- `reports/PROMPT-1941-*` ✓
- `reports/PROMPT-1964-*` ✓
- `reports/PROMPT-1968-*` ✓
- `reports/PROMPT-1976-*` ✓

No forbidden paths (`client/**`, `tests/**`, `tools/**`, `production/**`,
`Cargo.*`, stage/QA/session-state files, unrelated reports) touched.

### git diff --check — PASS

No trailing-whitespace errors detected in any of the six report files.

### FF-readiness — PASS

```
git merge-base --is-ancestor origin/main HEAD
→ exit 0
```

Branch `report/autoplay-vsbot-window-contract-1976` is a strict fast-forward
from `origin/main@7b259e91`.

### No Cargo/Python tests required

Report-only payload; no test suite applicable.

---

## Branch / Commit Details

| Field | Value |
|-------|-------|
| Branch | `report/autoplay-vsbot-window-contract-1976` |
| Base (origin/main) | `7b259e91` — PROMPT 1972 signoff-pack reports reapply after 1959 |
| Stale predecessor | `origin/report/autoplay-vsbot-window-contract-1968` @ `c50f0323` (NOT_FF — superseded) |
| FF status | FF-READY |
| Commit message | `docs(reports): PROMPT 1976 — backfill 1861/1914/1941/1964/1968 operator contract + refresh after 1972` |

---

1976: AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REPORT-REFRESH-AFTER-1972: READY_FOR_MAINLAND_ENQUEUE
