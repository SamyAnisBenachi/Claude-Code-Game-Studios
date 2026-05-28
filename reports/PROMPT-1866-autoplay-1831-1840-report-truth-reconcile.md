# PROMPT 1866 — AUTOPLAY-1831-1840-REPORT-TRUTH-RECONCILE

**Date:** 2026-05-28
**Status:** DONE
**Branch:** `wt/1866-report-truth-reconcile` from `origin/main @ bb90d7c2`
**Re-applied by:** PROMPT 1877 (re-lands on top of origin/main @ 2ce3dc6b, preserving PROMPT 1845/1858/1846/1859/1872 reports)
**Scope:** Read-only forensic reconciliation — no source code edits.

---

## 1. Purpose

PROMPT 1840 backfilled `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md`
and produced `reports/PROMPT-1840-autoplay-vsbot-1831-report-backfill-reconcile.md`.
Both documents conclude that run `20260528-090613-Z` constitutes an authoritative
`DONE / PASS` for PROMPT 1831's task, citing it as the canonical post-1818
evidence.

Subsequent forensic audits — PROMPT 1844 (viewport/click-target evidence audit)
and PROMPT 1846 (evidence analyzer application) — identified material issues in
that run that invalidate the automated PASS conclusion.

This report reconciles the record, establishes the corrected verdict, and
documents what remediation is required before an automated PASS can be claimed.

---

## 2. What PROMPT 1840 Claimed

| Datum | 1840 claim |
|-------|-----------|
| Evidence dir | `production/qa/evidence/autoplay-runs/20260528-090613-Z` |
| Source of truth | `DONE / PASS` — report + evidence dir |
| Driver exit code | `0` ✅ |
| Checkpoints | 15/15 ✅ |
| Distinct desktop_bitblt hashes | 10 ✅ (≥ 3 requirement met) |
| Relay mismatch | `NEEDS_HUMAN_GUI` was transient preflight gate, not final outcome |
| Final verdict | Authoritative DONE / PASS — relay line discarded as misleading |

The relay mismatch explanation in PROMPT 1840 is **correct and preserved**: the
`NEEDS_HUMAN_GUI` relay line reflected the preflight gate state before the
operator launched the game. It is a transient launcher state, not a final
outcome. This fact stands.

The error in PROMPT 1840 is that it elevated run `090613` to "authoritative
DONE/PASS" without accounting for the mid-run window resize and its effect on
click-target validity.

---

## 3. What PROMPT 1844 Found (Superseding Evidence)

PROMPT 1844 performed a forensic audit of all three autoplay runs present at
`production/qa/evidence/autoplay-runs/` and found:

### 3.1 Mid-Run Window Resize (BLOCKER)

Run `20260528-090613-Z` experienced a DWM-triggered window resize mid-run:

```
tick=1    window_size = [1280, 720]   ← recipe built at this size
tick=114  window_size = [1280, 720]   ← last stable pre-resize tick
tick=115  window_size = [1280, 505]   ← resize START (window shrank)
tick=127  window_size = [1280, 1076]  ← resize END (stabilized at 1076)
```

The autoplay driver builds click coordinates **once at tick 1** for a 1280×720
window. After tick 127, all click targets are baked for 720px height but the
actual window is 1076px tall.

### 3.2 Post-Resize Click Targets Invalid (BLOCKER)

Critical post-resize clicks land at wrong fractional positions:

| Checkpoint | Tick | Coord (baked for 720) | Actual fraction in 1076 | Expected fraction |
|---|---|---|---|---|
| auction-ready bid | 127–128 | y=612 | 56.9% | 85% |
| placement drag start | 149–150 | y=662 | 61.5% | 92% |
| placement drag end | 155 | y=396 | 36.8% | 55% |
| placement submit | 166–167 | y=662 | 61.5% | 92% |

If bevy_ui reflowed for the taller window (the expected behavior), all
placement and submit clicks missed their targets.

### 3.3 win32_printwindow Frozen for All 11 Captures (MAJOR)

Every `win32_printwindow` capture in run `090613` returned a frozen/stale hash.
`desktop_bitblt` fallback compensated and produced 10–12 distinct hashes
(confirming real visual state changes) — but the primary capture path was
non-functional throughout. The 10 distinct hashes cited by PROMPT 1831 as "PASS"
evidence came entirely from the fallback, not the primary capture mechanism.

### 3.4 Checkpoints Are Time-Based (INFO)

All 15 checkpoints fired at their expected tick counts — but checkpoints count
ticks, not game state transitions. A checkpoint firing does not confirm:
- That the preceding click landed on the correct UI element
- That the game responded to the click as intended
- That the UI was in the expected state at that tick

The run reaching `vs-bot-post-resolution` proves the server completed a full
game loop. It does not prove bot UI interactions were correct post-resize.

---

## 4. What PROMPT 1846 Found (Corroborating Evidence)

The automated evidence analyzer (`tools/autoplay/analyze_evidence_run.py`)
returned **PARTIAL** for run `090613`:

```
VERDICT: PARTIAL
REASON : FROZEN label appeared 11 time(s) in driver.log
```

Key metrics:
- `win32_printwindow` attempted 15 times; frozen 11 times
- `desktop_bitblt` fallback triggered 11 times; all produced distinct hashes
- Total distinct pixel_hashes: 12 out of 26 captures
- Analyzer verdict: PARTIAL (not PASS)

PROMPT 1846 explicitly concludes:

> "No run achieves PASS verdict from the analyzer. … Run 3 (090613): Strongest
> evidence with 12 distinct hashes and all checkpoints, but win32_printwindow
> is frozen 11/15 times. The desktop_bitblt fallback produces real evidence,
> but the analyzer correctly returns PARTIAL."

---

## 5. Corrected Verdicts

### 5.1 Run 20260528-090613-Z

| Dimension | PROMPT 1831/1840 verdict | Corrected verdict (PROMPT 1866) |
|-----------|--------------------------|----------------------------------|
| Automated PASS | PASS ✅ | **NOT PASS — CONDITIONAL** |
| Capture quality | "10 distinct hashes" ✅ | Hashes from fallback only; PrintWindow 100% frozen |
| Click accuracy | Implied correct | **Invalid post-resize (ticks 128–260)** |
| Checkpoint validity | 15/15 = verified ✅ | Time-based only; does not verify click accuracy |
| QA gate | Human sign-off noted | Human sign-off required + click-target repair first |

**Corrected run verdict:** `CONDITIONAL — human-review evidence only. Not an automated PASS for AUTOPLAY-VS-BOT-QA-001.`

### 5.2 PROMPT 1831

| Aspect | Original | Corrected |
|--------|----------|-----------|
| `NEEDS_HUMAN_GUI` relay | Correctly explained as preflight gate | Confirmed: still a transient gate, not final outcome |
| Run outcome | DONE / PASS | **CONDITIONAL** — superseded by 1844/1846 |
| Post-1818 capture chain | Working correctly (fallback active) | **Confirmed** — fallback IS working |
| Automated PASS claim | Asserted | **Withdrawn** — resize invalidates post-tick-127 clicks |

PROMPT 1831's core technical finding — that the post-1818 `desktop_bitblt`
fallback chain is active and produces distinct pixel hashes — is **valid and
preserved**. The error is in the PASS conclusion, not in the capture chain
diagnosis.

### 5.3 PROMPT 1840

| Aspect | 1840 claim | Corrected |
|--------|-----------|-----------|
| Relay mismatch root cause | Correctly identified | Confirmed correct |
| Run 090613 as authoritative PASS | Asserted | **Incorrect — not an automated PASS** |
| 1840 branch main-land | Implied ready | **Do NOT main-land** — overclaims PASS |

PROMPT 1840's relay mismatch analysis is preserved. The backfill of the 1831
report is preserved (file now has a correction header). The DONE/PASS verdict
for 090613 is withdrawn.

---

## 6. What Is True Post-Reconciliation

1. **The `NEEDS_HUMAN_GUI` relay from PROMPT 1831 was a transient preflight gate.**
   It reflected the launcher waiting for a live game window, not a final failure.
   The operator resolved it; the run proceeded. This is confirmed by 1840 and
   not changed by this reconciliation.

2. **Run `20260528-090613-Z` is the strongest available evidence for human review.**
   It has 12 distinct pixel_hashes (from `desktop_bitblt`), 15 Bevy RPC screenshots,
   all 15 checkpoints, and driver exit code 0. Runs `051148` and `063609` are
   weaker (no hash verification / frozen-all hashes respectively).

3. **Run `090613` is NOT an automated PASS.**
   The mid-run window resize (DWM-triggered at tick 115) invalidated click targets
   for the auction-ready, placement, and submit phases. The analyzer returns
   PARTIAL. The composite report correctly states `NOT-CLAIMED`.

4. **The post-1818 capture chain (PrintWindow + BitBlt fallback) is confirmed working.**
   This is the positive technical finding PROMPT 1831 established and it stands.
   The fallback activation is real and the distinct hashes prove live content was
   captured. This is separate from the click-target validity question.

5. **An automated PASS requires the repairs in PROMPT 1844 AC-VPT-01 through AC-VPT-08.**
   Specifically BLOCKING gates: AC-VPT-01 (min window size gate), AC-VPT-02
   (mid-run resize detection → abort), and AC-VPT-06 (distinct hash per
   checkpoint). These are not landed on `main` as of `2ce3dc6b`.

---

## 7. Files Modified by This Prompt

| File | Action | Rationale |
|------|--------|-----------|
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | Added correction header + inline correction notes | Preserve original content; clearly mark PASS → CONDITIONAL |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | Created (this file) | Reconciliation record |

The `origin/wt/1840-report-backfill` branch is **not merged**. Its relay
mismatch analysis (preserved in commit history) is accurate; its PASS verdict
is superseded by this report.

---

## 8. Disposition of origin/wt/1866 and origin/wt/1871

The branch `origin/wt/1866-report-truth-reconcile` was built from
`origin/main @ bb90d7c2` and would have deleted the PROMPT 1845 and 1858
reports when merged. PROMPT 1871 re-applied the correction on top of
`origin/main @ 5c91918d`, but that branch (`origin/wt/1871-truth-refresh`)
similarly could not be FF-merged once PROMPT 1872 landed (it would have deleted
the 1846/1859/1872 reports).

PROMPT 1877 is the final application: it re-applies the same correction content
cleanly on top of the current origin/main (`2ce3dc6b`, which includes all
1845/1858/1846/1859/1872 reports) without disturbing any of them.

---

## 9. Required Next Steps (Not In Scope of This Prompt)

| Step | Priority | Tracking |
|------|----------|---------|
| Implement AC-VPT-01: abort if initial window < 1280×720 | BLOCKING | PROMPT 1844 |
| Implement AC-VPT-02: abort on mid-run resize > ±10px | BLOCKING | PROMPT 1844 |
| Implement AC-VPT-06: distinct hash per checkpoint required for PASS | BLOCKING | PROMPT 1844 |
| Re-run vs-bot with window size lock enforced | Required for fresh PASS evidence | New run needed |
| Human visual review of 090613 bitblt PNGs (11 files) | CONDITIONAL PASS path | PROMPT 1846 |

---

1866: AUTOPLAY-1831-1840-REPORT-TRUTH-RECONCILE: DONE (re-landed by PROMPT 1877)
