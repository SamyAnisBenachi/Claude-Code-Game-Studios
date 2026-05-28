# PROMPT 1931 — AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1912

**Date:** 2026-05-28
**Status:** SHIPPED
**Branch:** `wt/1931-truth-refresh` from `origin/main @ 1c945fd2`
**Scope:** Reports-only backfill — no source code, tests, tool, or sprint-state changes.

---

## 1. Purpose

The corrected truth reconciliation for PROMPT 1831/1840 has now been attempted
four times before this prompt:

- **PROMPT 1866** (`wt/1866-report-truth-reconcile`, base `bb90d7c2`) — correct
  content, would have deleted PROMPT 1845/1858 reports on merge.
- **PROMPT 1871** (`wt/1871-truth-refresh`, base `5c91918d`) — re-applied on
  top of 1845/1858; PROMPT 1872 then landed, making 1871 non-FF.
- **PROMPT 1877** (`wt/1877-truth-refresh`, base `2ce3dc6b`) — re-applied on
  top of 1846/1859/1872; PROMPTs 1876 and 1856 then landed, making 1877 non-FF.
- **PROMPT 1892** (`wt/1892-truth-refresh`, base `c35750d8`) — re-applied on
  top of 1876/1856; PROMPTs 1879, 1880, 1893, 1894, and 1912 then landed on
  origin/main (`1c945fd2`), advancing main past the 1892 branch base.

This prompt is the current application: it re-applies the same correction content
cleanly on top of `origin/main @ 1c945fd2` — the commit that includes PROMPT 1912
(autoplay window-size default repair after 1894) — without deleting any existing
reports or reverting any tool changes.

---

## 2. Why 1892 Could Not Be Main-Landed After 1912

| Fact | Detail |
|------|--------|
| 1892 base commit | `origin/main @ c35750d8` (PROMPT 1856 UI layout smoke) |
| Current main HEAD | `1c945fd2` (PROMPT 1912 whitespace cleanup) |
| Commits between 1892 base and HEAD | `e8a40f81` PROMPT 1880 click-target viewport guard (source); `71484fc4` PROMPT 1894 click-target viewport guard refresh; `e02d132f` PROMPT 1912 window-size default repair (source); `fe2a9e88` PROMPT 1912 report; `1c945fd2` PROMPT 1912 whitespace cleanup |
| Safe action | Re-apply the 1831/1866/1871/1877/1892 correction content as new files on top of `1c945fd2` |

---

## 3. What Was Done

### 3.1 Files Written

| File | Action | Status |
|------|--------|--------|
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | Copied from `origin/wt/1892-truth-refresh` (correction notice, CONDITIONAL verdict) | ✅ |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | Copied from `origin/wt/1892-truth-refresh` (reconciliation record) | ✅ |
| `reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md` | Copied from `origin/wt/1892-truth-refresh` (superseded status documented) | ✅ |
| `reports/PROMPT-1877-autoplay-1831-1840-truth-refresh-after-1872.md` | Copied from `origin/wt/1892-truth-refresh` (prior re-application record) | ✅ |
| `reports/PROMPT-1892-autoplay-1831-1840-truth-refresh-after-1856-1876.md` | Copied from `origin/wt/1892-truth-refresh` (prior re-application record) | ✅ |
| `reports/PROMPT-1931-autoplay-1831-1840-truth-refresh-after-1912.md` | Created (this file) | ✅ |
| `tools/autoplay/` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1879-*` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1880-*` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1893-*` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1894-*` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1912-*` | **NOT TOUCHED** | ✅ |

### 3.2 Content Source

All five ported reports (`1831`, `1866`, `1871`, `1877`, `1892`) are taken
verbatim from `origin/wt/1892-truth-refresh`. No content was modified. The 1931
report adds the next step in the chain: why 1892 could not be main-landed after
the 1880/1894/1912 commits.

---

## 4. Truth Established (Final — supersedes 1866, 1871, 1877, and 1892 records)

### 4.1 NEEDS_HUMAN_GUI (PROMPT 1831 relay)

**Correctly explained by PROMPT 1840 — preserved unchanged:**
`NEEDS_HUMAN_GUI` was a transient preflight gate state while the launcher waited
for the operator to open a live game window. It is not a final failure verdict.
The operator launched the game; the run proceeded. This fact stands.

### 4.2 Run 20260528-090613-Z

| Dimension | PROMPT 1831/1840 verdict | Corrected verdict (final) |
|-----------|--------------------------|---------------------------|
| Automated PASS | PASS ✅ | **CONDITIONAL — NOT an automated PASS** |
| Capture quality | "10 distinct hashes" | Hashes from `desktop_bitblt` fallback only; `win32_printwindow` frozen 11/11 |
| Click accuracy | Implied correct | **Invalid for ticks 128–260** — mid-run DWM resize (720→505→1076) |
| Checkpoint validity | 15/15 = verified | Time-based only — does not verify click accuracy or UI state |
| Analyzer verdict | Not checked | PARTIAL (FROZEN label 11 times; `tools/autoplay/analyze_evidence_run.py`) |

**Definitive run verdict:** `CONDITIONAL — strongest available human-review evidence,
but NOT an automated PASS for AUTOPLAY-VS-BOT-QA-001.`

Run `20260528-090613-Z` remains CONDITIONAL. It is human-review evidence only.
It does not satisfy the automated PASS gate. This verdict is preserved unchanged
from the 1892 record and must not be upgraded to PASS without a clean re-run
with window-size lock enforced.

### 4.3 Post-1818 Capture Chain

**Confirmed working.** The `desktop_bitblt` fallback chain is active. 10–12
distinct pixel hashes prove live, non-frozen content was captured at each phase
transition. This positive technical finding from PROMPT 1831 is preserved unchanged.
The capture-chain functionality is separate from the click-target validity question.

### 4.4 Automated PASS Gate

**NOT satisfied** as of origin/main @ `1c945fd2`. The PROMPT 1844 blocking
acceptance criteria (AC-VPT-01 through AC-VPT-08) are not cleared as of this
base. A clean re-run with window-size lock enforced is required for an automated
PASS.

---

## 5. Validation

### 5.1 Path Allowlist Review

All files written are under `reports/`. No source files, test files, tool files,
`production/sprint-status.yaml`, `production/session-state/`, `production/sprints/`,
`production/qa/`, `production/stage.txt`, or Cargo/CI files were touched.

### 5.2 git diff --check

No whitespace errors.

### 5.3 git diff --name-status origin/main..HEAD

```
A  reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md
A  reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md
A  reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md
A  reports/PROMPT-1877-autoplay-1831-1840-truth-refresh-after-1872.md
A  reports/PROMPT-1892-autoplay-1831-1840-truth-refresh-after-1856-1876.md
A  reports/PROMPT-1931-autoplay-1831-1840-truth-refresh-after-1912.md
```

No deletions. All existing report files untouched.

### 5.4 Ancestor Check

Branch `wt/1931-truth-refresh` is based on `origin/main @ 1c945fd2`. Origin/main
is a direct ancestor of this branch (additions only, no deletes).

---

## 6. Branch and Commit

- **Worktree:** `D:\tmp\wt-1931-truth-refresh`
- **Branch:** `wt/1931-truth-refresh`
- **Base:** `origin/main @ 1c945fd2`

---

1931: AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1912: SHIPPED
