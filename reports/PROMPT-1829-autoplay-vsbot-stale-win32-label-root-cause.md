# PROMPT 1829 — AUTOPLAY-VSBOT-STALE-WIN32-LABEL-ROOT-CAUSE

**Status:** DIAGNOSED — no source repair needed  
**Date:** 2026-05-28  
**Worktree:** `tmpwt-1829-stale-win32-label-root-cause`  
**Branch:** `wt/1829-stale-win32-label-root-cause`  
**HEAD:** `ae06e9b1` (origin/main)

---

## 1. Objective

Determine why the latest vs-bot autoplay evidence run
(`production/qa/evidence/autoplay-runs/20260528-063609-Z`) shows pre-1818
`win32_capture=OK` log labels instead of the post-1818
`win32_printwindow=OK/FROZEN` labels, and whether the frozen-frame detection
added by PROMPT 1818 is actually live in the current code.

---

## 2. Root Cause — Evidence Predates the Fix

The evidence run at `20260528-063609-Z` was executed at **06:36:09 UTC** on
2026-05-28. The PROMPT 1818 commit (`d8b41463`) — which renamed the log label
from `win32_capture=OK` to `win32_printwindow=OK` and introduced the
frozen-frame fallback — was merged at **07:52:42 UTC** on the same day.

The run predates the fix by **76 minutes**. The `driver.py` that executed was
the pre-1818 version. There is no stale copy, no path mismatch, and no pyc
cache issue — the run simply used the code as it existed at that time.

The same conclusion holds for `20260528-051148-Z` (05:11:48 UTC), which is
even earlier.

### Timing proof

| Event | UTC timestamp |
|---|---|
| Run `20260528-051148-Z` | 2026-05-28 05:11:48 |
| Run `20260528-063609-Z` | 2026-05-28 06:36:09 |
| PROMPT 1818 commit `d8b41463` | 2026-05-28 07:52:42 |
| PROMPT 1824 commit `a0a96360` (env gate) | after 1818 |
| Current HEAD `ae06e9b1` | latest |

---

## 3. Current Code Audit (HEAD = ae06e9b1)

All script paths resolve via `LauncherRoot` = `D:\_DEV\Work\Claude-Code-Game-Studios`
(the play-root fallback chain in `Start-AutoplayVsBot.ps1` falls through to
launcher root because `D:\_DEV\ccgs-play-main` exists but contains no
`Cargo.toml`).

### 3.1 `tools/autoplay/driver.py` — post-1818 label (correct)

`driver.py:335`:
```python
log(f"tick={tick} win32_printwindow={'OK' if _win32_ok else 'FAILED'} path={_win32_shot.name}")
```

`driver.py:339–343` — frozen-frame detection:
```python
if _bitblt_reason == "frozen_printwindow":
    log(
        f"tick={tick} win32_printwindow=FROZEN "
        f"hash={last_win32_hash} — triggering desktop_bitblt fallback"
    )
```

Both the label and the frozen-fallback logic are present and correct.

### 3.2 Script path chain (no stale copy)

| Script | Resolved path |
|---|---|
| `Start-AutoplayVsBot.ps1` | `D:\_DEV\Work\Claude-Code-Game-Studios\tools\dev-launcher\` |
| `Run-AutoplaySmoke.ps1` | `Join-Path $LauncherRoot 'tools\autoplay\Run-AutoplaySmoke.ps1'` |
| `driver.py` | `Join-Path $PSScriptRoot "driver.py"` inside Run-AutoplaySmoke.ps1 |

All three resolve to the same repo. No secondary checkout is consulted.

### 3.3 Stale-pyc guard (correct, double-layered)

`Start-AutoplayVsBot.ps1` §6b clears:
- `$LauncherRoot\tools\autoplay\__pycache__`
- `$LauncherRoot\tools\autoplay\recipes\__pycache__`

`Run-AutoplaySmoke.ps1` additionally clears the same dirs via `$PSScriptRoot`.

Both guards fire before the driver process is started. The `-B` flag is also
passed to Python, suppressing re-cache writes.

---

## 4. All-Identical Pixel Hash Observation

Every win32 capture in `20260528-063609-Z` reports `pixel_hash=0x26207c4c` and
`47377 bytes` — 15 frames, identical. This is the frozen-DWM-buffer symptom
that PROMPT 1818 was designed to detect and fall back from. Because the run
used the pre-1818 driver, no fallback fired. With the post-1818 driver the
frozen-frame detection would have triggered on tick 2 (second screenshot) when
the hash repeated, and `desktop_bitblt` would have been attempted.

---

## 5. Why PROMPT 1827's Finding Was Accurate But Misread

PROMPT 1827 observed "driver.log still uses old `win32_capture=OK` labels" and
concluded the frozen fallback "did not trigger." Both observations are factually
correct for the evidence run it examined. The misread was treating the evidence
as a post-1818 run — it was not. No post-1818 live vs-bot run exists in the
evidence directory.

---

## 6. No Repair Needed

- `tools/autoplay/driver.py` at HEAD is correct (post-1818 labels + frozen fallback).
- `tools/autoplay/Run-AutoplaySmoke.ps1` is correct.
- `tools/dev-launcher/Start-AutoplayVsBot.ps1` is correct.
- No stale copy, no path mismatch, no pyc cache issue.

No source file was modified in this worktree.

---

## 7. Next Step — Fresh Live Evidence Run (VERIFY worker)

A post-1818 vs-bot live run has never been captured. The next worker should:

1. Run: `pwsh -File tools/dev-launcher/Start-AutoplayVsBot.ps1 -Recipe vs-bot`
   (or equivalent with `CCGS_AUTOPLAY_BOT_ROOM_READY=1` preset).
2. Confirm `driver.log` contains `win32_printwindow=OK` on the first screenshot
   tick.
3. Confirm either `win32_printwindow=FROZEN ... desktop_bitblt=OK/FAILED` on
   subsequent ticks (if DWM is still frozen), OR multiple distinct `pixel_hash`
   values across the run (if DWM is now compositing correctly).
4. Archive the evidence and sign off AUTOPLAY-VS-BOT-QA-001 live gate if the
   run passes.

---

1829: AUTOPLAY-VSBOT-STALE-WIN32-LABEL-ROOT-CAUSE: DIAGNOSED
