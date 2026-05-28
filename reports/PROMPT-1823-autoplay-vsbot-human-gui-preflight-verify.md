# PROMPT 1823 — AUTOPLAY-VSBOT-HUMAN-GUI-PREFLIGHT-VERIFY

**Verdict:** CONCERNS
**Date:** 2026-05-28
**Worktree:** `tmpwt-1823-vsbot-preflight`
**Branch:** `prompt/1823-vsbot-preflight`
**HEAD (worktree):** `3c3aa6d72819247daade62e3bf3c441317b7090c`
**origin/main:** `822c48733e4313b5c29a82176af93eaf4e66c34a`

> HEAD is one commit behind origin/main. The delta is PROMPT 1822 (reports
> backfill only — `docs/reports): no autoplay source code changed. All
> checks below apply equally to both SHAs.

---

## 1. Checklist Results

### 1.1 HEAD / origin/main SHA

| Item | Value |
|---|---|
| Worktree HEAD | `3c3aa6d7` — PROMPT 1818 frozen PrintWindow BitBlt fallback report |
| origin/main | `822c4873` — PROMPT 1822 reports backfill (docs only, no code change) |
| Divergence impact | None — autoplay tools unchanged between the two commits |

**Result: PASS**

---

### 1.2 Run-AutoplaySmoke.ps1 Exists and Supports `-Recipe vs-bot`

- File present: `tools/autoplay/Run-AutoplaySmoke.ps1` ✅
- `-Recipe` is a declared parameter (line 27, default `"smoke"`)
- The script passes `--recipe $Recipe` directly to `driver.py` (line 131)
- `driver.py` looks up `args.recipe` in `REGISTRY`
- `vs_bot.NAME = "vs-bot"` is registered in `recipes/__init__.py` (line 83)
- `--list-recipes` dry-run output confirms `vs-bot` appears in the registry ✅

```
vs-bot    Composite recipe (add-bot-lobby -> class -> draft/auction ->
          placement -> resolution soak). Requires CCGS_DEBUG_UI=1 AND
          CCGS_AUTOPLAY_BOT_ROOM_READY=1; emits BLOCKED otherwise.
```

**Result: PASS**

---

### 1.3 Python Path

| Path | Exists | Version |
|---|---|---|
| `D:/_APPS/Python312/python.exe` | YES | Python 3.12.10 |
| `python` (PATH) | YES | resolves to same binary at `D:\_APPS\Python312\python.exe` |

Both the explicit path and the default `python` alias resolve to the same
Python 3.12.10 installation. The script's default `-Python python` is safe
to use; `-Python "D:/_APPS/Python312/python.exe"` is also valid and
unambiguous.

**Result: PASS**

---

### 1.4 Stale-PYC Guard

Located in `Run-AutoplaySmoke.ps1` lines 111–124 (PROMPT 1802 / PROMPT 1814):

```powershell
# Clears before every driver invocation:
tools/autoplay/__pycache__/
tools/autoplay/recipes/__pycache__/

# Also passes -B to Python:
$env:PYTHONDONTWRITEBYTECODE = '1'
```

Expected log lines during next run (launcher stdout, before driver start):

```
[autoplay-smoke] clearing stale pyc: <repo>\tools\autoplay\__pycache__
[autoplay-smoke] clearing stale pyc: <repo>\tools\autoplay\recipes\__pycache__
[autoplay-smoke] PYTHONDONTWRITEBYTECODE=1 (stale-pyc guard active)
```

**Result: PASS**

---

### 1.5 Post-1818 Markers in Driver

`tools/autoplay/driver.py` contains all required post-1818 markers:

| Marker | Line(s) | Present |
|---|---|---|
| `win32_printwindow=` (label prefix) | 335, 341 | ✅ |
| `win32_printwindow=FROZEN` | 341 | ✅ |
| `desktop_bitblt` (fallback import + log) | 57, 346, 348 | ✅ |
| `frozen_printwindow` (reason string) | 91, 100, 105, 339 | ✅ |
| `_frozen_win32_check` / consecutive hash logic | 81–105 | ✅ |

`tools/autoplay/win_capture.py` exposes:
- `capture_game_window_desktop_bitblt` (post-1813 function) at line 411 ✅
- `win32_printwindow` logging prefix at lines 380–407 ✅

**Result: PASS**

---

### 1.6 Port 15873 / Client Window Status

```
netstat -an | Select-String ":15873"  →  (no output)
```

Port 15873 is **not bound**. No autoplay client is running. This is the
expected state for a preflight check; it is not a failure. The human GUI
run will start a fresh client.

**Result: PASS (expected — no client running)**

---

### 1.7 Dry-Run / Help Mode

`Run-AutoplaySmoke.ps1` has **no `-DryRun`, `-WhatIf`, or `-Help` parameter**.
`[CmdletBinding()]` is declared but `SupportsShouldProcess` is not used.

The closest available dry-run is the `--list-recipes` flag on the driver:

```powershell
"D:/_APPS/Python312/python.exe" -B tools/autoplay/driver.py --list-recipes
```

This was executed and returned cleanly, confirming `vs-bot` is in the
registry. **No script-level dry-run exists.**

**Result: N/A (no dry-run mode; driver --list-recipes confirms registry)**

---

### 1.8 Optional Focused Build Check

Skipped. The preflight scope is read-only verification. A `cargo check -p
client --bin client --features autoplay-remote` would require the full Rust
toolchain to activate and is an optional verify lane only. The PROMPT 1821
operator pack already documents this as the recommended pre-run step (Section
4.2). No environment blocker prevents running it if the operator chooses.

**Result: SKIPPED (non-blocking; see corrected operator pack Section 4 below)**

---

## 2. CONCERNS — vs-bot Recipe Env Var Gap

> **Severity: MEDIUM — will cause BLOCKED exit without action.**

The `vs-bot` recipe has two hard env var gates enforced in
`tools/autoplay/recipes/vs_bot.py` lines 68–92:

| Env Var | Required value | Purpose |
|---|---|---|
| `CCGS_DEBUG_UI` | `"1"` | Exposes the Add Bot button in the lobby UI |
| `CCGS_AUTOPLAY_BOT_ROOM_READY` | `"1"` | Signals that a bot peer is available to seat |

**What happens without them:**

1. The launcher builds and starts the client normally.
2. The RPC port binds (port 15873).
3. The driver starts, loads `vs-bot`, runs the initial `vs-bot-precheck` checkpoint.
4. The recipe emits `local.block` — the driver exits with **code 4** immediately.
5. No gameplay is driven. No frozen-fallback evidence is produced.
6. The `launcher-status.json` records `"outcome": "driver_failed"` (exit ≠ 0).

**Gap in PROMPT 1821 operator pack:**

Section 4.3 of PROMPT 1821 showed:

```powershell
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

This command does **not** set `CCGS_DEBUG_UI` or `CCGS_AUTOPLAY_BOT_ROOM_READY`.
The `Run-AutoplaySmoke.ps1` script also does not set them. Running the 1821
command verbatim will produce a BLOCKED exit.

`CCGS_AUTOPLAY_BOT_ROOM_READY=1` also implies the server-side bot soak room
must be started first (via `Start-BotVsBotSoak.ps1`, PROMPT 1607). The vs_bot.py
BLOCK hint states: *"Set CCGS_AUTOPLAY_BOT_ROOM_READY=1 after launching
`Start-BotVsBotSoak.ps1` (PROMPT 1607)."*

---

## 3. Corrected Human Command Block

### 3.1 Prerequisites (before running the smoke script)

1. Start the bot soak room (server-side prerequisite):
   ```powershell
   # Run this in a separate terminal and leave it running:
   pwsh -File tools/autoplay/Start-BotVsBotSoak.ps1
   ```

2. Confirm no other process holds port 15873:
   ```powershell
   netstat -an | Select-String ":15873"
   # Expected: no output
   ```

3. Optional — confirm build is clean before the timed run:
   ```powershell
   cargo check -p client --bin client --features autoplay-remote 2>&1 | Select-Object -Last 10
   # Expected: ends with "Finished"
   ```

### 3.2 Full Live Run Command (corrected)

From `D:\_DEV\Work\Claude-Code-Game-Studios` in a PowerShell terminal:

```powershell
$env:CCGS_DEBUG_UI = "1"
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

> `python` (default `-Python` value) resolves to `D:\_APPS\Python312\python.exe`
> (Python 3.12.10) — no `-Python` override needed unless PATH changes.

### 3.3 Alternative: Driver-Only (client already running)

If a vs-bot client is already up with `CCGS_AUTOPLAY=1` on port 15873:

```powershell
cd D:\_DEV\Work\Claude-Code-Game-Studios
$env:CCGS_DEBUG_UI = "1"
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
$stamp = (Get-Date -Format "yyyyMMdd-HHmmss") + "-Z"
$artDir = "production/qa/evidence/autoplay-runs/$stamp"
New-Item -ItemType Directory -Path $artDir -Force | Out-Null
"D:/_APPS/Python312/python.exe" -B tools/autoplay/driver.py `
    --port 15873 `
    --recipe vs-bot `
    --artifact-dir $artDir
```

---

## 4. Post-Run Verification (from PROMPT 1821 Section 6 — unchanged)

| # | Check | Pass condition | Fail signal |
|---|---|---|---|
| 1 | Build | `cargo build` exits 0 | exit code ≠ 0 |
| 2 | RPC bind | `[autoplay-smoke] RPC port bound` in launcher stdout | timeout |
| 3 | Stale-pyc guard | `clearing stale pyc` + `PYTHONDONTWRITEBYTECODE=1` lines | lines absent |
| 4 | Post-1818 label | `driver.log` contains `win32_printwindow=OK/FAILED/FROZEN` | `win32_capture=OK` → pre-1818 bytecode |
| 5 | Frozen fallback *(if window frozen)* | `win32_printwindow=FROZEN` + `desktop_bitblt=OK reason=frozen_printwindow` | FROZEN without desktop_bitblt |
| 6 | Distinctness | ≥3 distinct SHA-256 across `win32_tick_*.png` or `bitblt_tick_*.png` | 1 hash across all |
| 7 | Artifact dir | `production/qa/evidence/autoplay-runs/<stamp>/` with `driver.log`, PNGs, `screenshots/` | dir empty |
| 8 | Launcher status | `launcher-status.json` → `"outcome": "ok"` | `rpc_port_never_bound` or `driver_failed` |

**Extra check (new — env var gate):**

| # | Check | Pass condition | Fail signal |
|---|---|---|---|
| 0 | vs-bot not BLOCKED | `driver.log` does NOT contain `local.block` for `vs-bot-precheck` | BLOCKED at `vs-bot-precheck` → `CCGS_DEBUG_UI` or `CCGS_AUTOPLAY_BOT_ROOM_READY` not set |

---

## 5. Summary

| Check | Result |
|---|---|
| Worktree / branch | `tmpwt-1823-vsbot-preflight` / `prompt/1823-vsbot-preflight` |
| HEAD (worktree) | `3c3aa6d7` |
| origin/main | `822c4873` (1822 reports backfill — no code change) |
| `Run-AutoplaySmoke.ps1` exists | PASS |
| `-Recipe vs-bot` supported | PASS — `vs-bot` registered in REGISTRY |
| Python `D:/_APPS/Python312/python.exe` | PASS — Python 3.12.10 |
| `python` in PATH | PASS — resolves to same binary |
| Stale-pyc guard | PASS — launcher lines 111–124 |
| Post-1818 markers in driver.py | PASS — all 4 markers confirmed |
| Port 15873 bound | NOT BOUND (expected — no client running) |
| Dry-run mode | N/A — `--list-recipes` confirms `vs-bot` in registry |
| **vs-bot env var gap** | **CONCERN — CCGS_DEBUG_UI=1 and CCGS_AUTOPLAY_BOT_ROOM_READY=1 must be set before run; 1821 operator pack omitted them** |
| Corrected command block | YES — Section 3 above |

**Overall verdict: CONCERNS** — all infrastructure is in place; the run
will succeed if and only if the two env vars are set and the bot soak room
is running. Without them the driver exits BLOCKED after the first checkpoint
and produces no live evidence.

---

1823: AUTOPLAY-VSBOT-HUMAN-GUI-PREFLIGHT-VERIFY: CONCERNS
