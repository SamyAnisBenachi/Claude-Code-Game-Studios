# PROMPT 1820 — AUTOPLAY-VSBOT-LIVE-SCREENSHOT-VERIFY-AFTER-1818-1819

**Status:** NEEDS_HUMAN_GUI
**Date:** 2026-05-28
**Worktree:** `tmpwt-1820-live-screenshot-verify`
**Branch:** `worktree-1820-live-screenshot-verify`
**HEAD:** `3c3aa6d72819247daade62e3bf3c441317b7090c`
**Origin/main:** same — confirmed via `git fetch --dry-run`

---

## 1. Objective

Verify that fresh screenshots produced by the post-1818/1819 driver:
- use the new `win32_printwindow=OK/FAILED/FROZEN` log labels, and
- trigger the desktop BitBlt fallback when consecutive captures are hash-identical,
- resulting in ≥3 distinct hashes across the run.

---

## 2. Pre-Run Checks

### 2.1 HEAD / Origin Sync

```
local  HEAD : 3c3aa6d7  docs(reports): PROMPT 1818 — frozen PrintWindow BitBlt fallback implementation report
origin/main : 3c3aa6d7  (same — git fetch --dry-run returned nothing)
```

No PROMPT 1819 commit exists in the repo. The task description refers to
"PROMPT 1819 static verification PASS" — this was a static code-review
confirmation that the 1818 implementation was logically correct. No new commit
or live-run report was created for 1819. The most recent code-change commit is
`d8b41463` (PROMPT 1818 feat).

### 2.2 Python Path

```
D:/_APPS/Python312/python.exe  (from relay protocol in PROMPT-1820 task)
```

### 2.3 Launcher Command (for human GUI run)

```powershell
# From repo root:
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

Stale-pyc guard is active inside the launcher (PROMPT 1814): clears
`tools/autoplay/__pycache__` and `tools/autoplay/recipes/__pycache__` before
invoking `python -B driver.py`.

### 2.4 1818 Code Confirmation

The `_frozen_win32_check()` helper **is present** in current `tools/autoplay/driver.py`:

```
driver.py:81   def _frozen_win32_check(
driver.py:100      return True, "win32_printwindow_failed", last_hash
driver.py:104      if last_hash is not None and current_hash == last_hash:
driver.py:105          return True, "frozen_printwindow", current_hash
driver.py:335      log(f"tick={tick} win32_printwindow={'OK' if _win32_ok else 'FAILED'} ...")
driver.py:341      f"tick={tick} win32_printwindow=FROZEN "
driver.py:342      f"hash={last_win32_hash} — triggering desktop_bitblt fallback"
```

New log labels expected in a post-1818 live run:
- `tick=N win32_printwindow=OK path=win32_tick_NNNNNN.png`
- `tick=N win32_printwindow=FROZEN hash=<md5> — triggering desktop_bitblt fallback`
- `tick=N desktop_bitblt=OK reason=frozen_printwindow path=bitblt_tick_NNNNNN.png`

---

## 3. Existing Run Evidence

### 3.1 Run Index

| Run dir | UTC start | BST start | Post-1818? |
|---|---|---|---|
| `20260528-051148-Z` | 05:11 | 06:11 | No (before `d8b41463` at 07:52 UTC) |
| `20260528-063609-Z` | 06:36 | 07:36 | No (before `d8b41463` at 07:52 UTC) |

PROMPT 1818 feat commit timestamp: `2026-05-28 08:52:42 +0100` = `07:52:42 UTC`.
Both existing runs predate that commit. No post-1818 live run exists.

### 3.2 Run 20260528-063609-Z — Pre-1818, All Frozen

This is the most recent run. Log labels use the pre-1818 format (`win32_capture=OK`
not `win32_printwindow=OK`), confirming the 1818 code was not active.

**win32 capture hashes (SHA-256 prefix, 15 captures):**

| File | SHA-256[:12] |
|---|---|
| win32_tick_000005.png | `58f3d88ad004` |
| win32_tick_000030.png | `58f3d88ad004` |
| win32_tick_000042.png | `58f3d88ad004` |
| win32_tick_000051.png | `58f3d88ad004` |
| win32_tick_000072.png | `58f3d88ad004` |
| win32_tick_000081.png | `58f3d88ad004` |
| win32_tick_000093.png | `58f3d88ad004` |
| win32_tick_000113.png | `58f3d88ad004` |
| win32_tick_000138.png | `58f3d88ad004` |
| win32_tick_000147.png | `58f3d88ad004` |
| win32_tick_000164.png | `58f3d88ad004` |
| win32_tick_000176.png | `58f3d88ad004` |
| win32_tick_000185.png | `58f3d88ad004` |
| win32_tick_000250.png | `58f3d88ad004` |
| win32_tick_000259.png | `58f3d88ad004` |

**Distinct win32 hashes: 1 / 15 — all frozen.**

DWM pixel_hash in driver log: `0x26207c4c` for every tick — consistent with a
static/minimised window returning stale DWM buffer.

**RPC screenshot hashes (screenshots/*.png, 15 files):**

| File | SHA-256[:12] |
|---|---|
| 000000.png | `b987b7a7ecb7` |
| 000007.png | `b987b7a7ecb7` |
| (all 15) | `b987b7a7ecb7` |

**Distinct RPC hashes: 1 / 15 — also all frozen.**

No `win32_printwindow=FROZEN` or `desktop_bitblt` lines appear in either run's
`driver.log`. The 1818 frozen fallback was never exercised.

---

## 4. Live Verification Attempt

### 4.1 Game Process Check

```bash
tasklist | grep -i "lanes|game|bevy|winit"
→ no matching processes
```

**The game client is not running.** No RPC endpoint to connect to.

### 4.2 Autoplay Driver Dry-Run

Not executed — no client window present. Running the driver without a live client
will immediately fail at `capabilities` RPC with a connection refused error.

---

## 5. Stale-PYC Guard Status

The stale-pyc guard in `Run-AutoplaySmoke.ps1` (PROMPT 1814) clears
`tools/autoplay/__pycache__` before every run and passes `-B` to Python so
bytecode is never re-written. Confirmed present at lines 112–124 of the launcher.

A `__pycache__` directory currently exists under `tools/autoplay/`:

```
tools/autoplay/__pycache__/  (present — will be cleared by launcher before next run)
```

This is the pre-1820 residual cache. The launcher will clear it before using the
post-1818 source.

---

## 6. NEEDS_HUMAN_GUI: Exact Commands

Start the game client and run the vs-bot autoplay recipe from the repo root:

```powershell
# 1. From D:\_DEV\Work\Claude-Code-Game-Studios, in a new terminal:
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

The launcher will:
1. `cargo build -p client --bin client --features autoplay-remote`
2. Launch the client with `CCGS_AUTOPLAY=1 CCGS_AUTOPLAY_PORT=15873`
3. Clear `tools/autoplay/__pycache__` (stale-pyc guard)
4. Run `python -B driver.py --recipe vs-bot`
5. Emit artifacts to `production/qa/evidence/autoplay-runs/<stamp>/`

**To confirm post-1818 behaviour is active, look in `driver.log` for:**

```
tick=N win32_printwindow=OK path=win32_tick_NNNNNN.png       ← new label (1818)
tick=N win32_printwindow=FROZEN hash=<md5> — triggering desktop_bitblt fallback
tick=N desktop_bitblt=OK reason=frozen_printwindow path=bitblt_tick_NNNNNN.png
```

If FROZEN never appears, PrintWindow is returning distinct frames — which also
proves the frozen-frame bug is gone for this run. Either way, expect ≥3 distinct
SHA-256 values across `win32_tick_*.png` OR `bitblt_tick_*.png`.

**If the client does not build, check:**
```powershell
cargo check -p client --features autoplay-remote 2>&1 | tail -20
```

---

## 7. Summary

| Check | Result |
|---|---|
| HEAD / origin sync | PASS — both at `3c3aa6d7` |
| PROMPT 1818 code present in driver.py | PASS — `_frozen_win32_check` + `win32_printwindow=` labels confirmed |
| Stale-pyc guard active in launcher | PASS — Run-AutoplaySmoke.ps1 lines 112–124 |
| Post-1818 live run with new log labels | MISSING — no run exists after `d8b41463` (07:52 UTC) |
| PROMPT 1819 static-verify report | MISSING — no commit or report file for 1819 |
| Pre-1818 run distinctness (20260528-063609-Z) | FAIL — 1/15 distinct, all frozen |
| Game process running now | NO |
| Live test executable without human GUI | NO |

**Overall verdict: NEEDS_HUMAN_GUI**

The 1818 code is present and correct; it has never been exercised by a live run.
A human must launch the client and run `Run-AutoplaySmoke.ps1 -Recipe vs-bot`
to produce post-1818 evidence.

---

1820: AUTOPLAY-VSBOT-LIVE-SCREENSHOT-VERIFY-AFTER-1818-1819: NEEDS_HUMAN_GUI
