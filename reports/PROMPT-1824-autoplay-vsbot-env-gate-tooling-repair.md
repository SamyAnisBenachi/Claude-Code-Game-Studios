# PROMPT 1824 — AUTOPLAY-VSBOT-ENV-GATE-TOOLING-REPAIR

**Status:** SHIPPED
**Date:** 2026-05-28
**Worktree:** `tmpwt-1824-vsbot-env-gate`
**Branch:** `prompt/1824-vsbot-env-gate`
**Base (origin/main):** `822c4873`

---

## 1. Problem Summary

PROMPT 1823 preflight confirmed that `Run-AutoplaySmoke.ps1 -Recipe vs-bot` would
immediately exit BLOCKED (driver exit 4) when invoked directly, because two required
env vars were not set:

| Env var | Required value | Effect when absent |
|---|---|---|
| `CCGS_DEBUG_UI` | `"1"` | Bevy client hides the Add Bot button in lobby UI |
| `CCGS_AUTOPLAY_BOT_ROOM_READY` | `"1"` | vs-bot recipe emits `local.block`, driver exits 4 |

The PROMPT 1821 operator command (`pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot`)
omitted both vars. The driver would BLOCK after completing the full 60-second cargo
build + client launch cycle — wasting ~2 minutes before the failure was surfaced.

`Start-AutoplayVsBot.ps1` (the canonical composite entrypoint, PROMPT 1644) already
set both vars correctly at lines 329 and 336. The gap was `Run-AutoplaySmoke.ps1`
invoked directly, which had no vs-bot-specific env gate logic.

---

## 2. Fix Applied — `tools/autoplay/Run-AutoplaySmoke.ps1`

Inserted a vs-bot env gate block immediately after the initial `Write-Host` (line 51)
and **before** the `cargo build` step. This means:

- The gate fires at the cheapest possible point — before any Rust compilation.
- `CCGS_DEBUG_UI` is auto-set to `"1"` when absent (safe; it just controls a UI flag).
- `CCGS_AUTOPLAY_BOT_ROOM_READY` is **not** auto-set — it requires a real running bot
  room, so the script fails fast with explicit remediation instructions including:
  1. How to launch `Start-BotVsBotSoak.ps1` in a separate terminal.
  2. The exact env + re-run commands to paste.
  3. The one-command alternative (`Start-AutoplayVsBot.ps1`) that handles everything.
- Exit code on blocked: `4` — matches the driver's `local.block` exit, keeping the
  `launcher-status.json` `"outcome": "driver_failed"` semantics consistent.
- All other recipes are unaffected; the block is gated on `$Recipe -eq 'vs-bot'`.

### Diff summary

```
tools/autoplay/Run-AutoplaySmoke.ps1  +27 lines (env gate block only)
```

---

## 3. PROMPT 1823 Report Recovery

Copied from tmpwt-1823-vsbot-preflight to root `reports/`:

```
reports/PROMPT-1823-autoplay-vsbot-human-gui-preflight-verify.md
```

---

## 4. Validation

| Check | Result |
|---|---|
| `git diff --check` | PASS — no whitespace errors |
| `python -B tools/autoplay/driver.py --list-recipes` (vs-bot) | PASS — `vs-bot` still in registry |
| Non-vs-bot recipes unchanged | PASS — gate block is `if ($Recipe -eq 'vs-bot')` only |
| PROMPT 1823 report copied to `reports/` | PASS |

---

## 5. Behavior Before / After

### Before (PROMPT 1821 operator command verbatim)

```
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

1. cargo build — 60–90 s
2. Client launches, RPC port binds
3. Driver starts, runs `vs-bot-precheck`
4. Recipe emits `local.block` — driver exits 4 immediately
5. `launcher-status.json` → `"outcome": "driver_failed"`
6. No useful evidence produced. Failure reason not obvious from launcher output.

### After (same command, same invocation)

```
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```

1. vs-bot env gate fires immediately (< 1 s):
   - `CCGS_DEBUG_UI` absent → auto-set to `1`, log line emitted
   - `CCGS_AUTOPLAY_BOT_ROOM_READY` absent → BLOCKED message printed with full
     remediation steps, exit 4
2. cargo build never starts. No wasted build time.

### After (with bot room running)

```powershell
# Terminal A: start bot soak room
pwsh -File tools/dev-launcher/Start-BotVsBotSoak.ps1

# Terminal B: run vs-bot smoke
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
# CCGS_DEBUG_UI is auto-set; CCGS_AUTOPLAY_BOT_ROOM_READY is already set
# → gate passes, build starts, driver runs normally
```

### Canonical one-command path (preferred, no manual env setup)

```powershell
pwsh -File tools/dev-launcher/Start-AutoplayVsBot.ps1 -Recipe vs-bot
```

`Start-AutoplayVsBot.ps1` (PROMPT 1644) already sets both env vars and launches the
bot soak room automatically. This remains the recommended operator path.

---

## 6. Files Modified / Created

| File | Action |
|---|---|
| `tools/autoplay/Run-AutoplaySmoke.ps1` | Modified — added vs-bot env gate block (+27 lines) |
| `reports/PROMPT-1823-autoplay-vsbot-human-gui-preflight-verify.md` | Recovered from tmpwt-1823-vsbot-preflight |
| `reports/PROMPT-1824-autoplay-vsbot-env-gate-tooling-repair.md` | This report |

---

## 7. Remaining Human GUI Prerequisites

To execute a live vs-bot run after this repair:

1. Start bot soak room in a separate terminal:
   ```powershell
   pwsh -File tools/dev-launcher/Start-BotVsBotSoak.ps1
   ```
2. Confirm port binding (wait for `Server listening on port ...` message).
3. Set env var and run:
   ```powershell
   $env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
   pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
   ```
   (Or simply use `Start-AutoplayVsBot.ps1 -Recipe vs-bot` which handles step 1–3.)

---

1824: AUTOPLAY-VSBOT-ENV-GATE-TOOLING-REPAIR: SHIPPED
