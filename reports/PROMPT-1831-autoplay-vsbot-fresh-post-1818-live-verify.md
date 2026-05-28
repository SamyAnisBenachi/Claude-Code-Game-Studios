<!-- CORRECTION NOTICE — added by PROMPT 1866, re-applied by PROMPT 1877 (2026-05-28) -->
<!-- The original report below was written at the time of the run and reflects    -->
<!-- the worker's understanding at that moment. Subsequent audits (PROMPT 1844,   -->
<!-- PROMPT 1846) identified material caveats that prevent treating run            -->
<!-- 20260528-090613-Z as an automated PASS. See PROMPT-1866 report for full      -->
<!-- reconciliation. Key corrections:                                              -->
<!--  - The "PASS" verdict is DOWNGRADED to CONDITIONAL/HUMAN-REVIEW.             -->
<!--  - Run 090613 had a mid-run window resize (720→505→1076) that invalidated    -->
<!--    all post-resize click targets (auction-ready, placement, submit phases).  -->
<!--  - win32_printwindow was frozen for ALL 11 captures; bitblt was the actual   -->
<!--    evidence path — the PASS language overstated the capture quality.         -->
<!--  - Checkpoints are time-based, not state-verified; 15/15 does not confirm   -->
<!--    that bot clicks landed on correct UI elements after the resize.           -->
<!-- CORRECTED STATUS: CONDITIONAL — human operator visual review required.       -->
<!-- Do NOT cite this run as an automated signoff for AUTOPLAY-VS-BOT-QA-001.    -->

# PROMPT 1831 — AUTOPLAY-VSBOT-FRESH-POST-1818-LIVE-VERIFY

**Date:** 2026-05-28  
**Original Status:** DONE — post-1818 live evidence confirmed *(see correction notice above)*  
**Corrected Status:** CONDITIONAL — human-review required (PROMPT 1866, re-applied PROMPT 1877)  
**Branch at execution:** main @ `71484998`  
**Evidence dir:** `production/qa/evidence/autoplay-runs/20260528-090613-Z`

---

## ⚠️ Correction Summary (PROMPT 1866 / PROMPT 1877)

The original report below claimed **Result: PASS** based on run `20260528-090613-Z`.
PROMPT 1844 and PROMPT 1846 subsequently identified the following blockers:

| Issue | Severity | Detail |
|-------|----------|--------|
| Mid-run window resize (720 → 505 → 1076) | BLOCKER | Ticks 115–127; post-resize click targets at wrong fractions |
| win32_printwindow frozen ALL 11 captures | MAJOR | `desktop_bitblt` was the real evidence path; PrintWindow frames are stale |
| Post-resize click coordinates invalid | BLOCKER | Placement drag/submit at y=662 landed at 61.5% of 1076px instead of 92% |
| Checkpoints are time-based, not state-verified | INFO | 15/15 does not confirm click accuracy |

**Corrected verdict:** Run `090613` is **CONDITIONAL** evidence — useful for human review
of `desktop_bitblt` frames, but does NOT constitute an automated PASS for
`AUTOPLAY-VS-BOT-QA-001`. The PROMPT 1844 acceptance criteria (AC-VPT-01 through
AC-VPT-08) must be satisfied before an automated PASS can be claimed.

PROMPT 1877 note: This correction was first produced by PROMPT 1866 on branch
`origin/wt/1866-report-truth-reconcile`. PROMPT 1871 re-applied it on top of
`origin/main @ 5c91918d` but that branch (`origin/wt/1871-truth-refresh`) could
not be FF-merged to origin/main because it predates PROMPT 1872 (which added the
1846/1859 reports) and would delete those reports. PROMPT 1877 re-applies the same
correction content cleanly on top of the current origin/main (`2ce3dc6b`) without
touching any existing reports.

---

## Original Report (preserved as-is)

---

## Result: PASS *(see correction notice — downgraded to CONDITIONAL)*

A fresh vs-bot run was executed against the post-1818 driver on `main`. All
validation criteria met. The `win32_printwindow=FROZEN` + `desktop_bitblt=OK`
fallback chain is working correctly in a live game session.

---

## Live Run Summary

| Check | Result |
|-------|--------|
| launcher-status outcome | `ok` ✅ |
| driver_exit_code | `0` ✅ |
| Capture label | `win32_printwindow=OK` / `win32_printwindow=FROZEN` (post-1818) ✅ |
| Old label absent | No `win32_capture=OK` in driver.log ✅ |
| desktop_bitblt fallback | `desktop_bitblt=OK reason=frozen_printwindow` at 11 ticks ✅ |
| Distinct desktop_bitblt hashes | **10 unique hashes** ✅ (requirement: ≥ 3) |
| Screenshots | 15 screenshots in `screenshots/` ✅ |
| All checkpoints reached | All 13 (lobby-loaded → vs-bot-post-resolution) ✅ |

---

## Capture Chain Behavior

`win32_printwindow` returned the same frozen hash (`0xb4db8636`) on most
frames — this is the frozen-PrintWindow condition PROMPT 1818 was designed to
handle. The frozen-frame detector triggered `desktop_bitblt` as the fallback on
every frozen tick, producing live, distinct frames each time:

```
tick=5   win32_printwindow=OK    (first frame, no prior hash → not yet frozen)
tick=30  win32_printwindow=OK    (first repeat hash → frozen detected next tick)
tick=51  win32_printwindow=FROZEN hash=0874d30f... → desktop_bitblt=OK (0x6ba13736)
tick=72  win32_printwindow=FROZEN → desktop_bitblt=OK (0x281d3775)
tick=81  win32_printwindow=FROZEN → desktop_bitblt=OK (0x5ef6083d)
tick=93  win32_printwindow=FROZEN → desktop_bitblt=OK (0xd4e70842)
tick=113 win32_printwindow=FROZEN → desktop_bitblt=OK (0xd4e70842)
tick=138 win32_printwindow=OK    (window resized: 1296x759 → 1296x1115, new hash)
tick=147 win32_printwindow=FROZEN → desktop_bitblt=OK (0xef7ef1dd)
tick=164 win32_printwindow=FROZEN → desktop_bitblt=OK (0x34b8206f)
tick=176 win32_printwindow=FROZEN → desktop_bitblt=OK (0xf9e702e5)
tick=185 win32_printwindow=FROZEN → desktop_bitblt=OK (0x8941261d)
tick=250 win32_printwindow=FROZEN → desktop_bitblt=OK (0xaa0544a1)
tick=259 win32_printwindow=FROZEN → desktop_bitblt=OK (0x9dcf44a0)
```

The `desktop_bitblt` pixel hashes are all distinct (10 unique values) —
confirming live, non-frozen frames were captured at each game phase transition.

> ⚠️ **Correction (PROMPT 1866/1877):** The tick=138 "window resized: 1296x759 → 1296x1115"
> entry reflects the mid-run DWM resize. After this point all click coordinates baked at
> 720 height are wrong for the 1076-height window. The 10 distinct hashes confirm the
> capture fallback worked, but they do not confirm that bot clicks landed on correct
> UI elements.

---

## Checkpoints Reached

```
lobby-loaded → bot-added → lobby-confirmed → class-select-loaded →
class-confirmed → shop-loaded → shop-slot-clicked → auction-loaded →
auction-ready → placement-loaded → placement-dragged → placement-submitted →
resolution-started → resolution-complete → vs-bot-post-resolution
```

All 15 checkpoints reached. Recipe completed at tick=260, driver exited 0.

> ⚠️ **Correction (PROMPT 1866/1877):** Checkpoints are time-based counters, not
> state-verified signals. Reaching all 15 proves the server advanced the game loop;
> it does not prove the bot's UI clicks after the resize landed on the correct elements.

---

## Launcher-Status

```json
{
  "artifact_dir": "...autoplay-runs/20260528-090613-Z",
  "started_at":   "2026-05-28T09:06:15.2205321Z",
  "finished_at":  "2026-05-28T09:06:55.7056807Z",
  "driver_exit_code": 0,
  "outcome": "ok"
}
```

---

## Context: Pre-1818 Runs

Both earlier runs from the same day were confirmed pre-1818:

| Run | Labels | Distinct hashes | Verdict |
|-----|--------|----------------|---------|
| `20260528-051148-Z` | None (pre-capture-code) | — | Pre-1818 |
| `20260528-063609-Z` | `win32_capture=OK` | 1 (all frozen) | Pre-1818; frozen confirmed |
| `20260528-090613-Z` | `win32_printwindow=OK/FROZEN` + `desktop_bitblt=OK` | 10 | Post-1818; **CONDITIONAL** (see above) |

---

## Note on AUTOPLAY-VS-BOT-QA-001

Per the launcher script disclaimer: *"This is NOT a live PASS for
AUTOPLAY-VS-BOT-QA-001. An operator must review artifacts and sign off."*
This report documents the post-1818 capture chain works correctly. The QA gate
sign-off remains a human operator step.

> **Reinforced by PROMPT 1866/1877:** The AUTOPLAY-VS-BOT-QA-001 automated PASS gate
> has NOT been satisfied. Human review of `bitblt_tick_*.png` files and Bevy
> RPC screenshots from run 090613 is required, with explicit verification that
> bot clicks (especially placement and submit phases) landed on correct UI
> elements despite the window resize.

---

1831: AUTOPLAY-VSBOT-FRESH-POST-1818-LIVE-VERIFY: CONDITIONAL (corrected by PROMPT 1866, re-applied PROMPT 1877)
