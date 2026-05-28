# PROMPT 1892 — AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1856-1876

**Date:** 2026-05-28  
**Status:** SHIPPED  
**Branch:** `wt/1892-truth-refresh` from `origin/main @ c35750d8`  
**Scope:** Reports-only backfill — no source code, tests, tool, or sprint-state changes.

---

## 1. Purpose

The corrected truth reconciliation for PROMPT 1831/1840 has been attempted three times:

- **PROMPT 1866** (`wt/1866-report-truth-reconcile`, base `bb90d7c2`) — correct
  content, but would have deleted PROMPT 1845/1858 reports on merge.
- **PROMPT 1871** (`wt/1871-truth-refresh`, base `5c91918d`) — re-applied on
  top of 1845/1858, but PROMPT 1872 then landed (`2ce3dc6b`) adding reports
  1846/1859. The 1871 branch was not FF-mergeable without deleting those reports.
- **PROMPT 1877** (`wt/1877-truth-refresh`, base `2ce3dc6b`) — re-applied on
  top of 1846/1859/1872, but PROMPTs 1876 and 1856 then landed on origin/main
  (`674ba870` and `c35750d8`), advancing main past the 1877 branch base.

This prompt is the final application: it re-applies the same correction content
cleanly on top of `origin/main @ c35750d8` — the commit that includes PROMPT
1876 (dev-launcher evidence UX) and PROMPT 1856 (UI 1280×720 layout smoke) —
without deleting any existing reports or reverting any tool changes.

---

## 2. Why 1877 Could Not Be Main-Landed After 1856/1876

| Fact | Detail |
|------|--------|
| 1877 base commit | `origin/main @ 2ce3dc6b` (PROMPT 1872 analyzer reports) |
| Current main HEAD | `c35750d8` (PROMPT 1856 UI layout smoke) |
| Commits between 1877 base and HEAD | `674ba870` (PROMPT 1876: dev-launcher evidence UX + `tools/dev-launcher/Start-AutoplayVsBot.ps1`) and `c35750d8` (PROMPT 1856: `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md`) |
| 1877 diff would delete | `reports/PROMPT-1856-*`, `reports/PROMPT-1876-*`, and revert `tools/dev-launcher/Start-AutoplayVsBot.ps1` changes |
| Safe action | Re-apply the 1831/1866/1871/1877 correction content as new files on top of `c35750d8` |

---

## 3. What Was Done

### 3.1 Files Written

| File | Action | Status |
|------|--------|--------|
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | Created with correction notice (CONDITIONAL verdict) | ✅ |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | Created (reconciliation record) | ✅ |
| `reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md` | Created (superseded status documented) | ✅ |
| `reports/PROMPT-1877-autoplay-1831-1840-truth-refresh-after-1872.md` | Created (prior re-application record) | ✅ |
| `reports/PROMPT-1892-autoplay-1831-1840-truth-refresh-after-1856-1876.md` | Created (this file) | ✅ |
| `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1876-dev-launcher-autoplay-evidence-ux-refresh-after-1872.md` | **NOT TOUCHED** | ✅ |
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | **NOT TOUCHED** | ✅ |

### 3.2 Content Source

All four ported reports (`1831`, `1866`, `1871`, `1877`) are taken verbatim from
`origin/wt/1877-truth-refresh`. No content was modified; the 1877 report already
documents why 1877 could not be main-landed after 1872 (Section 2 in that report).
This 1892 report adds the next step in the chain: why 1877 could not be main-landed
after 1856/1876.

---

## 4. Truth Established (Final — supersedes 1866, 1871, and 1877 records)

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

### 4.3 Post-1818 Capture Chain

**Confirmed working.** The `desktop_bitblt` fallback chain is active. 10–12
distinct pixel hashes prove live, non-frozen content was captured at each phase
transition. This positive technical finding from PROMPT 1831 is preserved unchanged.
The capture-chain functionality is separate from the click-target validity question.

### 4.4 Automated PASS Gate

**NOT satisfied** as of origin/main @ `c35750d8`. The PROMPT 1844 blocking
acceptance criteria (AC-VPT-01 through AC-VPT-08) are not landed on main.
A clean re-run with window-size lock enforced is required for an automated PASS.

---

## 5. Validation

### 5.1 Path Allowlist Review

All files written are under `reports/`. No source files, test files, tool files,
`production/sprint-status.yaml`, `production/session-state/`, or
`production/stage.txt` were touched.

### 5.2 git diff --check

No whitespace errors.

### 5.3 git diff --name-status origin/main..HEAD

```
A  reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md
A  reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md
A  reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md
A  reports/PROMPT-1877-autoplay-1831-1840-truth-refresh-after-1872.md
A  reports/PROMPT-1892-autoplay-1831-1840-truth-refresh-after-1856-1876.md
```

No deletions. All existing report files untouched. PROMPT 1856, PROMPT 1876,
and `tools/dev-launcher/Start-AutoplayVsBot.ps1` remain exactly as on origin/main.

### 5.4 Ancestor Check

Branch `wt/1892-truth-refresh` is based on `origin/main @ c35750d8`. Origin/main
is a direct ancestor of this branch (no deletions, only additions).

---

## 6. Branch and Commit

- **Worktree:** `D:\_DEV\Work\tmpwt-1892-truth-refresh`
- **Branch:** `wt/1892-truth-refresh`
- **Base:** `origin/main @ c35750d8`

---

1892: AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1856-1876: SHIPPED
