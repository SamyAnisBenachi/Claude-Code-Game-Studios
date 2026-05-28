# PROMPT 1821 — AUTOPLAY-VSBOT-1820-REPORT-RECOVERY-HUMAN-GUI-PACK

**Status:** SHIPPED
**Date:** 2026-05-28
**Worktree:** `tmpwt-1821-report-recovery`
**Branch:** `prompt-1821-report-recovery`
**HEAD:** `3c3aa6d72819247daade62e3bf3c441317b7090c`

---

## 1. PROMPT 1820 Report Recovery

### 1.1 Was the report found?

**YES — RECOVERED.**

The PROMPT 1820 report was located at:

```
D:\_DEV\Work\Claude-Code-Game-Studios\tmpwt-1820-live-screenshot-verify\reports\PROMPT-1820-autoplay-vsbot-live-screenshot-verify-after-1818-1819.md
```

A summary file was also present:

```
D:\_DEV\Work\Claude-Code-Game-Studios\tmpwt-1820-live-screenshot-verify\reports\PROMPT-1820-autoplay-vsbot-live-screenshot-verify-after-1818-1819.summary.txt
```

**Root cause of orchestrator miss:** The report was written to the worktree-local
`reports/` directory (`tmpwt-1820-live-screenshot-verify/reports/`) rather than the
repo-root `reports/`. The orchestrator searched only root `reports/`, `.claude/worktrees`,
external dirs, and `D:\tmp` — not the sibling `tmpwt-*` worktrees on disk.

### 1.2 PROMPT 1820 Report Summary

| Field | Value |
|---|---|
| Status | `NEEDS_HUMAN_GUI` |
| Date | 2026-05-28 |
| Branch | `worktree-1820-live-screenshot-verify` |
| HEAD | `3c3aa6d72819247daade62e3bf3c441317b7090c` |
| PROMPT 1818 code present | PASS — `_frozen_win32_check` + `win32_printwindow=` labels confirmed in `driver.py` |
| Post-1818 live run exists | NO — both existing runs (`20260528-051148-Z`, `20260528-063609-Z`) predate commit `d8b41463` |
| PROMPT 1819 static-verify | No commit or report file; 1819 was an in-conversation static code-review PASS only |
| Game process when 1820 ran | NOT RUNNING — client was not active |

---

## 2. Post-1818 Live Evidence

### 2.1 Evidence Index

| Run directory | UTC start | Post-1818? | Notes |
|---|---|---|---|
| `production/qa/evidence/autoplay-runs/20260528-051148-Z` | 05:11 UTC | **No** | Before `d8b41463` (07:52 UTC) |
| `production/qa/evidence/autoplay-runs/20260528-063609-Z` | 06:36 UTC | **No** | Before `d8b41463` (07:52 UTC); most recent run |

**No post-1818 live run exists.** PROMPT 1818 feat commit (`d8b41463`) landed at
`2026-05-28 08:52:42 +0100` (= `07:52:42 UTC`). Both runs predate it.

### 2.2 Most Recent Pre-1818 Run — 20260528-063609-Z

Artifacts present (15 win32 captures):

```
production/qa/evidence/autoplay-runs/20260528-063609-Z/
  capabilities.json
  checkpoints.jsonl
  driver-timeline.jsonl
  driver.log                  ← no win32_printwindow= lines (pre-1818 labels)
  launcher-status.json
  process.log / process.log.err
  screenshots/                ← 15 files, all hash b987b7a7ecb7 (frozen)
  win32_tick_000005.png … win32_tick_000259.png  ← 15 files, all hash 58f3d88ad004 (frozen)
  status.json
```

Log format used: `win32_capture=OK` (pre-1818 label — confirms 1818 code was not active).

Distinctness: 1/15 distinct hash across win32 captures. All frozen.

---

## 3. Stale-PYC Guard Status

The stale-pyc guard is active in `Run-AutoplaySmoke.ps1` (PROMPT 1802 / PROMPT 1814),
lines 111–124:

```powershell
# Clears before every run:
tools/autoplay/__pycache__/
tools/autoplay/recipes/__pycache__/

# Also passes -B to Python (no bytecode written/read):
$env:PYTHONDONTWRITEBYTECODE = '1'
```

A residual `tools/autoplay/__pycache__/` exists from before PROMPT 1820 ran.
The launcher will clear it automatically before the next driver invocation.

**Expected stale-pyc guard log line during next run:**

```
[autoplay-smoke] clearing stale pyc: <repo>\tools\autoplay\__pycache__
[autoplay-smoke] PYTHONDONTWRITEBYTECODE=1 (stale-pyc guard active)
```

---

## 4. Human GUI Operator Pack

### 4.1 Prerequisites

1. Game client source must compile (`autoplay-remote` feature).
2. No other process on port `15873`.
3. Working directory: `D:\_DEV\Work\Claude-Code-Game-Studios`.

### 4.2 Build Pre-Flight (DryRun equivalent)

Run this first to confirm the build is clean before doing the live run:

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios:
cargo check -p client --bin client --features autoplay-remote 2>&1 | Select-Object -Last 20
```

Expected: ends with `Finished` (warnings OK, errors NOT OK).

If it fails, fix the build before proceeding. Do not run the smoke script against a
broken build — the launcher will fail at the build step and produce no evidence.

### 4.3 Full Live Run Command

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios, in a terminal:
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

**What the launcher does (in order):**

1. `cargo build -p client --bin client --features autoplay-remote`
2. Sets `CCGS_AUTOPLAY=1`, `CCGS_AUTOPLAY_PORT=15873`
3. Launches the client via `cargo run`
4. Polls port 15873 until the RPC server binds (up to 60 s)
5. Clears `tools/autoplay/__pycache__` and `tools/autoplay/recipes/__pycache__`
6. Sets `PYTHONDONTWRITEBYTECODE=1`
7. Runs `python -B driver.py --port 15873 --recipe vs-bot ...`
8. Emits artifacts to:
   ```
   production/qa/evidence/autoplay-runs/<yyyyMMdd-HHmmss-Z>/
   ```

### 4.4 Alternative: Separate Terminal (client already running)

If a vs-bot game is already running in the native client:

```powershell
# Terminal A (skip if client already running — this is the launcher path above)
# cargo run -p client --bin client --features autoplay-remote
# (with CCGS_AUTOPLAY=1 and CCGS_AUTOPLAY_PORT=15873)

# Terminal B — driver only:
cd D:\_DEV\Work\Claude-Code-Game-Studios
$stamp = (Get-Date -Format "yyyyMMdd-HHmmss") + "-Z"
$artDir = "production/qa/evidence/autoplay-runs/$stamp"
New-Item -ItemType Directory -Path $artDir -Force | Out-Null
"D:/_APPS/Python312/python.exe" -B tools/autoplay/driver.py `
    --port 15873 `
    --recipe vs-bot `
    --artifact-dir $artDir
```

---

## 5. Expected Log Markers (post-1818)

All markers appear in `<artifact-dir>/driver.log`.

### 5.1 Win32 PrintWindow — Normal (OK)

```
tick=N win32_printwindow=OK path=win32_tick_NNNNNN.png
```

Appears when PrintWindow succeeds and the frame hash differs from the previous capture.

### 5.2 Win32 PrintWindow — Frozen frame detected

```
tick=N win32_printwindow=FROZEN hash=<md5hex> — triggering desktop_bitblt fallback
```

Appears when consecutive PrintWindow hashes are identical (frozen frame).
The `_frozen_win32_check()` helper at `driver.py:81` triggers this path.

### 5.3 Desktop BitBlt Fallback (after frozen detection)

```
tick=N desktop_bitblt=OK reason=frozen_printwindow path=bitblt_tick_NNNNNN.png
```

or on BitBlt failure:

```
tick=N desktop_bitblt=FAILED reason=frozen_printwindow path=bitblt_tick_NNNNNN.png
```

### 5.4 Win32 PrintWindow — API failure

```
tick=N win32_printwindow=FAILED path=win32_tick_NNNNNN.png
```

Appears when PrintWindow returns a non-OK result (window minimised, no handle).

### 5.5 Stale-PYC Guard (in launcher stdout, not driver.log)

```
[autoplay-smoke] clearing stale pyc: <path>\tools\autoplay\__pycache__
[autoplay-smoke] clearing stale pyc: <path>\tools\autoplay\recipes\__pycache__
[autoplay-smoke] PYTHONDONTWRITEBYTECODE=1 (stale-pyc guard active)
```

### 5.6 Screenshot File Paths

| Capture type | Path pattern |
|---|---|
| Win32 PrintWindow | `<artifact-dir>/win32_tick_NNNNNN.png` |
| Desktop BitBlt fallback | `<artifact-dir>/bitblt_tick_NNNNNN.png` |
| RPC screenshot | `<artifact-dir>/screenshots/NNNNNN.png` |

---

## 6. Pass/Fail Checklist for Human GUI Run

Run through this list after the run completes:

| # | Check | Pass condition | Fail signal |
|---|---|---|---|
| 1 | **Build** | `cargo build` exits 0 | exit code ≠ 0 → fix build |
| 2 | **RPC bind** | `[autoplay-smoke] RPC port bound` appears in launcher stdout | timeout → client not starting |
| 3 | **Stale-pyc guard** | `clearing stale pyc` + `PYTHONDONTWRITEBYTECODE=1` lines present in launcher stdout | lines absent → guard broken (check launcher lines 111–124) |
| 4 | **Post-1818 label** | `driver.log` contains `win32_printwindow=OK` or `win32_printwindow=FAILED` or `win32_printwindow=FROZEN` | `win32_capture=OK` → running pre-1818 bytecode |
| 5 | **Frozen fallback** *(if window was frozen)* | `driver.log` contains `win32_printwindow=FROZEN` + `desktop_bitblt=OK reason=frozen_printwindow` | `FROZEN` without `desktop_bitblt` → fallback logic not executing |
| 6 | **Distinctness** | ≥3 distinct SHA-256 hashes across all `win32_tick_*.png` OR `bitblt_tick_*.png` | 1 distinct hash across all captures → stuck/frozen; 1818 fallback may not have triggered enough ticks |
| 7 | **Artifact dir** | `production/qa/evidence/autoplay-runs/<stamp>/` exists and contains `driver.log`, `win32_tick_*.png`, `screenshots/` | dir empty or missing → driver crashed before first tick |
| 8 | **Launcher status** | `launcher-status.json` → `"outcome": "success"` | `"outcome": "rpc_port_never_bound"` → client startup failure |

**Minimum PASS bar:** Items 1–4 must pass. Items 5–6 are conditional on whether the
window actually exhibits frozen-frame behavior during the run.

---

## 7. Follow-Up Prompt After Human GUI Run

Once the run completes, paste the contents of `<artifact-dir>/driver.log` into a
new agent window with this prompt:

---

**PROMPT 1822 — AUTOPLAY-VSBOT-POST-1818-LIVE-VERIFY**

Context:
- PROMPT 1818 implemented the frozen PrintWindow → desktop BitBlt fallback.
- PROMPT 1820 and 1821 confirmed no post-1818 live run existed; a human GUI run
  was triggered per PROMPT 1821 operator pack.
- The run artifact dir is `production/qa/evidence/autoplay-runs/<stamp>/`.

Task:
- Read `<artifact-dir>/driver.log` and verify the following:
  1. At least one `win32_printwindow=OK` or `win32_printwindow=FROZEN` log line
     is present (confirms post-1818 code ran).
  2. If `win32_printwindow=FROZEN` appears: exactly one `desktop_bitblt=OK
     reason=frozen_printwindow` line follows each FROZEN line.
  3. SHA-256 hash distinctness across `win32_tick_*.png` files:
     - If no FROZEN lines: ≥3 distinct hashes expected (window was rendering).
     - If FROZEN lines present: ≥1 distinct BitBlt hash across `bitblt_tick_*.png`.
  4. Stale-pyc guard markers present in launcher stdout (from `process.log`).
- Write test evidence to `production/qa/evidence/autoplay-runs/<stamp>/verify-1822.md`.
- Verdict: PASS / FAIL / NEEDS_HUMAN_GUI (with specific reason).
- Commit and push the evidence file from a dedicated worktree.

Owned scope:
- Read-only: `production/qa/evidence/autoplay-runs/<stamp>/`.
- Write only: `production/qa/evidence/autoplay-runs/<stamp>/verify-1822.md`,
  `reports/PROMPT-1822-autoplay-vsbot-post-1818-live-verify.md`.
- Forbidden: source files, sprint files, session-state.

---

## 8. Summary

| Item | Result |
|---|---|
| PROMPT 1820 report found | YES — at `tmpwt-1820-live-screenshot-verify/reports/PROMPT-1820-*.md` |
| Root cause of orchestrator miss | Report written to worktree-local `reports/`; orchestrator searched only root `reports/` |
| Post-1818 live evidence exists | NO — no run after `d8b41463` (07:52 UTC 2026-05-28) |
| PROMPT 1818 code verified present | YES — `_frozen_win32_check`, `win32_printwindow=` labels at driver.py:81/335/341 |
| Stale-pyc guard active | YES — Run-AutoplaySmoke.ps1 lines 111–124 |
| Human GUI commands documented | YES — Section 4 above |
| Pass/fail checklist | YES — Section 6 above (8 items) |
| Follow-up prompt drafted | YES — PROMPT 1822 in Section 7 |

---

1821: AUTOPLAY-VSBOT-1820-REPORT-RECOVERY-HUMAN-GUI-PACK: SHIPPED
