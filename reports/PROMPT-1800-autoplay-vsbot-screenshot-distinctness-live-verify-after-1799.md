# PROMPT 1800 — Autoplay VS-Bot Screenshot Distinctness Live Verify After 1799

**Date:** 2026-05-28
**Worktree:** `D:\tmp\tmpwt-1800-screenshot-distinctness-verify`
**Branch:** `worker/1800-screenshot-distinctness-verify`
**Base:** `origin/main @ 4eb69de4` (PROMPT 1799)
**Status:** FAIL

---

## Summary

Two live vs-bot harness runs were executed. Both returned `outcome: ok` at the
composite-summary level (driver exit 0, all checkpoints hit), but the Bevy RPC
screenshots are **byte-identical** across all 15 captures in each run. The
hardened validator (PROMPT 1799) correctly detects this as `IDENTICAL-SCREENSHOTS`
and exits 1.

Root cause: the live machine's working checkout
(`D:\_DEV\Work\Claude-Code-Game-Studios`) is stale at commit `c6be1a33`
(PROMPT 1789), **12 commits behind** `origin/main@4eb69de4`. It includes PROMPT
1780 (offscreen camera always-on) but NOT PROMPT 1795 (restore
`Screenshot::primary_window()` as default). The offscreen camera renders a
secondary scene without the UI layer, producing near-identical dim frames
regardless of the game phase.

---

## Run 1 — Stale Main Checkout (PROMPT 1789 binary)

### Command

```powershell
cd D:/_DEV/Work/Claude-Code-Game-Studios
/c/Windows/System32/WindowsPowerShell/v1.0/powershell -ExecutionPolicy Bypass `
  -File tools/dev-launcher/Start-AutoplayVsBot.ps1 `
  -SkipSoakLaunch -Port 5000 -RpcPort 15873 -Recipe vs-bot `
  -DriverTimeoutSecs 300 -ClientStartupSecs 90
```

### Evidence Path

| Dir | Path |
|-----|------|
| Composite | `D:\_DEV\Work\Claude-Code-Game-Studios\production\qa\evidence\composite-runs\2026-05-28-051148-autoplay-vs-bot\` |
| Autoplay | `D:\_DEV\Work\Claude-Code-Game-Studios\production\qa\evidence\autoplay-runs\20260528-051148-Z\` |

### Harness Outcome

```json
{
  "schema": "autoplay_vs_bot_composite_summary_v1",
  "outcome": "ok",
  "recipe": "vs-bot",
  "smoke_exit_code": 0,
  "live_pass_status": "NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator sign-off for live PASS evidence"
}
```

Driver exit: 0. All vs-bot checkpoints hit:

```
lobby-loaded @tick 1, bot-added @tick 26, lobby-confirmed @tick 38,
class-select-loaded @tick 47, class-confirmed @tick 68,
shop-loaded @tick 77, shop-slot-clicked @tick 89, auction-loaded @tick 109,
auction-ready @tick 134, placement-loaded @tick 143, placement-dragged @tick 160,
placement-submitted @tick 172, resolution-started @tick 181,
resolution-complete @tick 246, vs-bot-post-resolution @tick 255
```

### Screenshot Evidence

| Metric | Value |
|--------|-------|
| PNG count | 15 |
| Unique MD5 hashes | **1** (all identical) |
| Hash | `8c5ee59d2d0380c9fd942e5578d90bb0` |
| Size | 86,080 bytes each |
| Resolution | 1280×720 |
| Mean brightness | 26.2/255 |
| Near-black? | No (threshold = 15) |
| Distinct across time? | **No** — all 15 phases identical |

Files: `000000.png` through `000057.png` — all 15 identical.

### Bevy RPC Screenshot Assessment

**UNUSABLE.** All 15 captures are byte-identical. The Bevy client at this
checkout uses the offscreen render-target (PROMPT 1780 unconditional), which
captures a secondary scene without the Bevy UI layer. Regardless of game phase
(lobby / draft / placement / resolution), the rendered content is the same dim
empty scene.

`SetForegroundWindow returned 0` on every foreground attempt — the window could
not be brought to the foreground, which worsens the frozen-frame problem but is
not the root cause (offscreen camera is the primary issue).

### Win32 Supplemental Captures

**ABSENT.** PROMPT 1794 (Win32 GDI/PrintWindow capture wiring in `driver.py`)
is not in the stale checkout. No `win32_tick_*.png` files were produced. No
win32 log messages appear in `driver.log`.

### Validator Output (post-1799 validator from worktree)

```
[validate_composite_run] FAIL: ...composite-runs\2026-05-28-051148-autoplay-vs-bot
  1 check(s) failed:
  FAIL: IDENTICAL-SCREENSHOTS: all 15 PNG(s) in screenshots/ share the same content
  hash (8c5ee59d2d0380c9fd942e5578d90bb0). Renderer may have produced frozen/
  repeated frames. Files: ['000000.png', ..., '000057.png']
```

**Validator exit code: 1.** The validator correctly catches the
`IDENTICAL-SCREENSHOTS` condition. This confirms the validator hardening
(PROMPT 1799) is working.

---

## Run 2 — Worktree at origin/main (PROMPT 1795 binary)

A second run was attempted from the worktree
(`D:\tmp\tmpwt-1800-screenshot-distinctness-verify`) which is at
`origin/main@4eb69de4` and includes PROMPT 1795 (restore `primary_window`
default) and PROMPT 1794 (Win32 capture in driver.py).

### Command

```powershell
cd D:/tmp/tmpwt-1800-screenshot-distinctness-verify
powershell -ExecutionPolicy Bypass -File tools/dev-launcher/Start-AutoplayVsBot.ps1 `
  -SkipSoakLaunch -Port 5000 -RpcPort 15874 -Recipe vs-bot `
  -DriverTimeoutSecs 300 -ClientStartupSecs 120 `
  -PlayRepoRoot D:/tmp/tmpwt-1800-screenshot-distinctness-verify
```

### Outcome

**INCONCLUSIVE — client did not produce log evidence within observation window.**

- Cargo build: completed successfully in 5m38s
  (`D:\tmp\tmpwt-1800-screenshot-distinctness-verify\target\debug\client.exe` created at 06:40 UTC)
- Port 15874 (RPC): never opened during observation
- `process.log`: never appeared in evidence dir
- `launcher-status.json`: never appeared
- Composite summary: not written
- Observation window: ~25 minutes since build completion

The 120-second client startup window in Run-AutoplaySmoke.ps1 has certainly
elapsed without the RPC port binding. The most likely cause is that the Bevy
client from the cold worktree target directory requires GPU shader compilation
on first launch, which can take several minutes. No error output was captured
(cargo run stdout redirected to `process.log` which did not appear, suggesting
the binary exited before Bevy fully initialized).

The Run 2 evidence is insufficient to validate the PROMPT 1795 fix.

---

## Root Cause Analysis

| Item | State |
|------|-------|
| `D:\_DEV\Work\Claude-Code-Game-Studios` checkout | At `c6be1a33` (PROMPT 1789) — **stale** |
| Origin/main current commit | `4eb69de4` (PROMPT 1799) |
| Commits behind | 12 (PROMPTs 1780→1793→1794→1795→1796→1797→1798→1799) |
| PROMPT 1780 (offscreen camera unconditional) | **ACTIVE** in stale checkout |
| PROMPT 1795 (restore `primary_window` default) | **NOT PRESENT** in stale checkout |
| PROMPT 1794 (Win32 capture in driver.py) | **NOT PRESENT** in stale checkout |
| PROMPT 1793 (file-ready poll) | **NOT PRESENT** in stale checkout |
| PROMPT 1799 (validator hardening) | In origin/main; validator ran from worktree correctly |

The screenshots from the stale checkout use `Screenshot::image(handle)` against
an offscreen `Image` render target that has no Bevy UI layer. Every frame
captures the same dim secondary scene. The near-black threshold (15/255) is not
triggered because the scene has some ambient content (brightness 26.2/255), but
all 15 captures are byte-identical.

---

## Bevy RPC Screenshot Usability

**NOT USABLE.** All 15 Bevy RPC screenshots are identical (md5 `8c5ee59d...`).

## Win32 Supplemental Captures Usability

**NOT PRESENT.** The stale checkout does not have PROMPT 1794. No win32 captures produced.

## Validator Correctness

**PASS.** The post-1799 validator from `origin/main`:
- Correctly detected `IDENTICAL-SCREENSHOTS` (all 15 identical hashes)
- Returned exit code 1
- Did not falsely report near-black (brightness 26.2 > threshold 15)
- All structural checks (schema, outcome, run-path, checkpoints) passed

---

## What Needs to Happen Before This Can Pass

1. **Pull origin/main into the live checkout**: `git pull origin main` from
   `D:\_DEV\Work\Claude-Code-Game-Studios` to bring it to `4eb69de4`. This
   activates PROMPT 1795 (primary_window default), 1793 (file-ready poll),
   1794/1797 (Win32 capture), and 1796/1798/1799 (validator hardening).
2. Re-run `Start-AutoplayVsBot.ps1` from the updated checkout with
   `-SkipSoakLaunch` (server on port 5000 remains available).
3. Confirm screenshots are distinct (at least 2 unique MD5 hashes).
4. Confirm at least some brightness above 15/255.
5. Confirm validator exits 0.

Until the checkout is updated and a fresh run succeeds, screenshot distinctness
is **FAIL**.

---

## Artifact Paths

| Artifact | Path |
|----------|------|
| Run 1 composite evidence | `production/qa/evidence/composite-runs/2026-05-28-051148-autoplay-vs-bot/` (in stale main checkout) |
| Run 1 autoplay artifacts | `production/qa/evidence/autoplay-runs/20260528-051148-Z/` (in stale main checkout) |
| Run 2 worktree composite dir | `D:\tmp\tmpwt-1800-screenshot-distinctness-verify\production\qa\evidence\composite-runs\2026-05-28-053449-autoplay-vs-bot\` |
| Run 2 worktree autoplay dir | `D:\tmp\tmpwt-1800-screenshot-distinctness-verify\production\qa\evidence\autoplay-runs\20260528-053449-Z\` (no files) |

---

1800: AUTOPLAY-VSBOT-SCREENSHOT-DISTINCTNESS-LIVE-VERIFY-AFTER-1799: FAIL
