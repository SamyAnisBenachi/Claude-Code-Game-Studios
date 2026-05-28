# PROMPT 1914 — AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REFRESH-AFTER-1894

**Type:** Report-only backfill — branch refresh  
**Date:** 2026-05-28  
**Status:** SHIPPED  
**Backfills:** `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md`  
**Source branch (1861):** `origin/report/autoplay-vsbot-window-contract-1861` @ `cee93efe`  
**Refresh base:** `origin/main` @ `71484fc4` (PROMPT 1894)  
**Target branch:** `report/autoplay-vsbot-window-contract-1914`

---

## Purpose

PROMPT 1861 shipped the authoritative operator contract / runbook correction for the
autoplay vs-bot window size issue onto branch
`origin/report/autoplay-vsbot-window-contract-1861`. That branch was NOT fast-forward
mergeable against current `origin/main` after PROMPTs 1856, 1876, and 1894 landed.
A direct merge would have deleted reports from those three PROMPTs and reverted
`tools/autoplay/driver.py` and `tools/dev-launcher/Start-AutoplayVsBot.ps1`.

This task cherry-picks **only the PROMPT 1861 report file** onto current
`origin/main@71484fc4`, creating a clean FF-ready branch
`report/autoplay-vsbot-window-contract-1914`.

No implementation files are touched.

---

## What PROMPT 1861 Established (Key Truths Preserved)

PROMPT 1861 was an operator contract/runbook **correction** that superseded PROMPT 1847.
Its key findings remain valid after PROMPTs 1856/1876/1894:

1. **None of the three 2026-05-28 autoplay runs is a clean automated PASS.**
   - `20260528-051148-Z` — PARTIAL (no pixel_hash data, Bevy RPC screenshots only)
   - `20260528-063609-Z` — PARTIAL (all 15 hashes frozen/identical; not usable as evidence)
   - `20260528-090613-Z` — PARTIAL (mid-run window resize + frozen PrintWindow)

2. **Run 090613-Z is conditional human-review evidence only** — it is NOT a clean
   automated PASS. It is disqualified by: mid-run resize (720 → 505 → 1076 px at
   tick 115), 720-baked coordinates applied post-resize, PrintWindow frozen 11/15,
   and `cursor_logical=None` at resize ticks.

3. **Window / viewport preflight remains required** — the operator must verify
   1280×720 minimum geometry, no DWM-snapped state, and no full-screen overlay before
   each run.

4. **Acceptance criteria AC-VPT-01..08** (from PROMPT 1844 §8) define the minimum bar
   that a future clean automated PASS must satisfy.

---

## State of Repairs at PROMPT 1894 (current main)

| AC | Requirement | PROMPT | Status at origin/main@71484fc4 |
|---|---|---|---|
| AC-VPT-01 | Driver aborts if initial window < [1280, 720] | 1857 → landed as **1880/1894** | **LANDED** — `driver.py` `check_window_minimum` + `EXIT_VIEWPORT_GUARD=5` |
| AC-VPT-02 | Driver detects mid-run resize > ±10 px; aborts/marks NEEDS_HUMAN_GUI | 1857 → **1880/1894** | **LANDED** — `check_window_drift` in `driver.py` |
| AC-VPT-03 | Driver warns/aborts if `cursor_logical=None` before `mouse_down` | 1857 → **1880/1894** | **LANDED** — `cursor_none` abort in `driver.py` |
| AC-VPT-05 | Composite records `frozen_all`; verdict downgraded to `NEEDS_HUMAN_GUI` | 1850 | Not confirmed landed |
| AC-VPT-06 | ≥ 3 distinct hashes required for mechanical PASS | 1850 | Not confirmed landed |
| AC-VPT-07 | Composite records `initial_window_size`, `window_resize_events`; non-zero downgrades | 1850 | Not confirmed landed |
| AC-VPT-08 | Launcher sets `CCGS_WINDOW_WIDTH/HEIGHT=1280/720`; Rust enforces at Startup | 1842 | Not confirmed landed |
| AC-VPT-04 | Recipe rebuilt after resize (future work) | future | Not scheduled |

PROMPT 1894 (`e8a40f81`) landed 66 passing viewport guard tests
(`tests/tools/autoplay/test_driver_click_viewport_guard.py`) and the full
`driver.py` guard implementation (AC-VPT-01/02/03). The tests cover minimum-size
abort, drift detection, cursor-None abort, and OOB click abort.

**Implication:** Future autoplay runs under the post-1894 driver will abort with
`EXIT_VIEWPORT_GUARD=5` on the exact failure modes seen in 090613-Z (mid-run
resize, cursor-None click), rather than silently continuing.

---

## Files Changed vs origin/main

| Status | File |
|--------|------|
| A | `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md` |
| A | `reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md` |

No deletions. No changes to `tools/**`, `client/**`, `server/**`, `tests/**`,
`production/**`, `Cargo.*`, or any existing reports.

---

## Source Branch Details

| Field | Value |
|-------|-------|
| Source branch | `origin/report/autoplay-vsbot-window-contract-1861` |
| Source tip commit | `cee93efe` |
| Source commit message | `docs(reports): PROMPT 1861 — autoplay vs-bot window contract reconcile (1844/1846)` |
| Report backfilled | `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md` |

---

## Validation

### Path allowlist review

```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md
A  reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md
```

PASS — only the two owned report files added; no deletions; no forbidden paths.

### git diff --check

No trailing-whitespace errors in either report file.

### FF-readiness

```
git merge-base --is-ancestor origin/main HEAD
→ exit 0 (FF-READY)
```

### Branch / Commit

| Field | Value |
|-------|-------|
| Branch | `report/autoplay-vsbot-window-contract-1914` |
| Base (origin/main) | `71484fc4` |
| FF status | FF-ready |

---

1914: AUTOPLAY-VSBOT-WINDOW-SIZE-OPERATOR-CONTRACT-REFRESH-AFTER-1894: SHIPPED
