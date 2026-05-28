# PROMPT 1877 — AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1872

**Date:** 2026-05-28
**Status:** SHIPPED
**Branch:** `wt/1877-truth-refresh` from `origin/main @ 2ce3dc6b`
**Scope:** Reports-only backfill — no source code, tests, tool, or sprint-state changes.

---

## 1. Purpose

The corrected truth reconciliation for PROMPT 1831/1840 has been attempted twice:

- **PROMPT 1866** (`wt/1866-report-truth-reconcile`, base `bb90d7c2`) — correct
  content, but would have deleted PROMPT 1845/1858 reports on merge.
- **PROMPT 1871** (`wt/1871-truth-refresh`, base `5c91918d`) — re-applied on
  top of 1845/1858, but PROMPT 1872 then landed (`2ce3dc6b`) adding reports
  1846/1859. The 1871 branch is not FF-mergeable and would delete those reports.

This prompt is the final application: it re-applies the same correction content
cleanly on top of `origin/main @ 2ce3dc6b` — the commit that includes all
existing reports — without deleting any of them.

---

## 2. Why 1871 Could Not Be Main-Landed After 1872

| Fact | Detail |
|------|--------|
| 1871 base commit | `origin/main @ 5c91918d` (PROMPT 1858 backfill) |
| Current main HEAD | `2ce3dc6b` (PROMPT 1872 — reapply 1846/1859 analyzer reports) |
| Commit between 1871 base and HEAD | `5c91918d` → `2ce3dc6b` (PROMPT 1872: adds PROMPT-1846 and PROMPT-1859 reports) |
| 1871 diff would delete | `reports/PROMPT-1846-*` and `reports/PROMPT-1859-*` (and transitively the 1872 report) |
| Safe action | Re-apply the same 1831/1866/1871 correction content as new files on top of `2ce3dc6b` |

---

## 3. What Was Done

### 3.1 Files Written

| File | Action | Status |
|------|--------|--------|
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | Created (new on main — absent from origin/main before this commit) | ✅ |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | Created (new on main) | ✅ |
| `reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md` | Created (superseded status documented) | ✅ |
| `reports/PROMPT-1877-autoplay-1831-1840-truth-refresh-after-1872.md` | Created (this file) | ✅ |
| `reports/PROMPT-1833-autoplay-evidence-distinctness-analyzer.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md` | **NOT TOUCHED** | ✅ |
| `reports/PROMPT-1872-autoplay-evidence-analyzer-latest-run-refresh-after-1858.md` | **NOT TOUCHED** | ✅ |

### 3.2 Correction Applied to PROMPT-1831 Report

The 1831 report received the correction notice block originally authored by PROMPT 1866:
- HTML comment header marking PASS → CONDITIONAL
- Inline correction notes at the Capture Chain and Checkpoints sections
- Final status line: `CONDITIONAL (corrected by PROMPT 1866, re-applied PROMPT 1877)`
- Attribution updated from PROMPT 1871 → PROMPT 1877 for the final application

### 3.3 PROMPT-1866 Reconcile Report

The full 1866 reconcile report is preserved verbatim, with Section 8 updated to
document the full chain: 1866 → 1871 → 1877 (why each prior application was
not main-landed, and that 1877 is the final clean application).

### 3.4 PROMPT-1871 Report

Preserved as-is but with status updated to `SUPERSEDED by PROMPT 1877` and a
section explaining why 1871 was also not main-landed after 1872 landed.

---

## 4. Truth Established (Final — supersedes 1866 and 1871 records)

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

**NOT satisfied** as of origin/main @ `2ce3dc6b`. The PROMPT 1844 blocking
acceptance criteria (AC-VPT-01 through AC-VPT-08) are not landed on main.
A clean re-run with window-size lock enforced is required for an automated PASS.

---

## 5. Validation

### 5.1 git diff --check

```
(no whitespace errors)
```

### 5.2 git diff --name-status origin/main..HEAD

```
A  reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md
A  reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md
A  reports/PROMPT-1871-autoplay-1831-1840-truth-refresh-after-1858.md
A  reports/PROMPT-1877-autoplay-1831-1840-truth-refresh-after-1872.md
```

No deletions. All PROMPT 1833/1844/1845/1858/1846/1859/1872 report artifacts
are untouched.

### 5.3 No automated PASS claim

Neither the 1831 report nor this report claims run `20260528-090613-Z` as a
clean automated PASS. The word "PASS" appears only in correction headers marking
it as superseded/downgraded, and in the "NOT an automated PASS" assertions.

---

## 6. Branch and Commit

- **Worktree:** `D:\tmp\wt-1877-truth-refresh`
- **Branch:** `wt/1877-truth-refresh`
- **Base:** `origin/main @ 2ce3dc6b`

---

1877: AUTOPLAY-1831-1840-TRUTH-REFRESH-AFTER-1872: SHIPPED
