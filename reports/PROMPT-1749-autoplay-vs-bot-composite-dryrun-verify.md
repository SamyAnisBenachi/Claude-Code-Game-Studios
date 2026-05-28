# PROMPT 1749 — AUTOPLAY-VS-BOT-COMPOSITE-DRYRUN-VERIFY

**Date:** 2026-05-28
**Branch:** `work/autoplay-vs-bot-composite-dryrun-1749`
**Base:** `origin/main` @ `ba7bb394` (PROMPT 1747 — vs-bot recipe)

**Worktree:** `D:\_DEV\claude-code-game-studios-worktrees\tmpwt-1749-composite-dryrun`

---

## Task

Verify that `Start-AutoplayVsBot.ps1 -Recipe vs-bot -DryRun` accepts and propagates
`-Recipe vs-bot` to `Run-AutoplaySmoke.ps1` without launching any process or broad Cargo build.

---

## Script Inventory (origin/main)

| Script | Path | Status |
|---|---|---|
| `Start-AutoplayVsBot.ps1` | `tools/dev-launcher/Start-AutoplayVsBot.ps1` | ✅ Found |
| `Run-AutoplaySmoke.ps1` | `tools/autoplay/Run-AutoplaySmoke.ps1` | ✅ Found |
| `Start-BotVsBotSoak.ps1` | `tools/dev-launcher/Start-BotVsBotSoak.ps1` | ✅ Found |
| `vs_bot.py` recipe | `tools/autoplay/recipes/vs_bot.py` | ✅ Found |

---

## Parameter Analysis

`Start-AutoplayVsBot.ps1` declares:

```powershell
param(
    ...
    [string]$Recipe = 'full-game',
    [switch]$DryRun,
    ...
)
```

It passes `-Recipe $Recipe` verbatim to `Run-AutoplaySmoke.ps1` (line 319–327 of script):

```powershell
$smokeArgs = @(
    '-ExecutionPolicy', 'Bypass',
    '-File', $smokeScript,
    '-Port', $RpcPort,
    '-Recipe', $Recipe,
    ...
)
```

In `-DryRun` mode (line 336), it prints the full invocation instead of launching:

```powershell
[DRY RUN] would launch: powershell -ExecutionPolicy Bypass -File .../Run-AutoplaySmoke.ps1 -Port 15873 -Recipe $Recipe ...
```

---

## Recipe Registry Verification

`tools/autoplay/recipes/__init__.py` registers `vs_bot` module with:

```python
vs_bot.NAME: (vs_bot.DESCRIPTION, vs_bot.build),
```

`tools/autoplay/recipes/vs_bot.py` declares:

```python
NAME = "vs-bot"
```

The registry key is **exactly `"vs-bot"`** — same string passed via `-Recipe vs-bot`.
No hyphen-to-underscore translation is needed; driver.py looks up `args.recipe` directly in `REGISTRY`.

---

## Dry-Run Execution

**Command:**
```powershell
powershell -ExecutionPolicy Bypass `
  -File "tools\dev-launcher\Start-AutoplayVsBot.ps1" `
  -Recipe vs-bot -DryRun
```

**Full output:**
```
WARNING: 'D:\_DEV\ccgs-play-main' exists but contains no Cargo.toml. Ignoring stub and falling back to launcher root.
Set CCGS_PLAY_REPO_ROOT or -PlayRepoRoot to suppress this warning.

==== Roots ====
Launcher repo root: D:\_DEV\claude-code-game-studios-worktrees\tmpwt-1749-composite-dryrun
Play/build root:    D:\_DEV\claude-code-game-studios-worktrees\tmpwt-1749-composite-dryrun  (source: launcher root (default play root is a stub without Cargo.toml))

==== Desktop session check ====
UserInteractive: True

==== Child launcher check ====
Start-BotVsBotSoak.ps1 : found (D:\_DEV\...\tools\dev-launcher\Start-BotVsBotSoak.ps1)
Run-AutoplaySmoke.ps1  : found (D:\_DEV\...\tools\autoplay\Run-AutoplaySmoke.ps1)

==== Evidence dir ====
Evidence dir: D:\_DEV\...\production\qa\evidence\composite-runs\2026-05-28-004221-autoplay-vs-bot

==== Soak server port ====
Chosen soak server port: 5000

==== Starting soak server (DRY RUN -- skipped) ====
[DRY RUN] would launch: powershell -ExecutionPolicy Bypass -File .../Start-BotVsBotSoak.ps1 -Port 5000 -DurationSeconds 300

==== Autoplay smoke (recipe=vs-bot) ====
CCGS_AUTOPLAY_BOT_ROOM_READY = 1
SERVER_PORT                  = 5000
SERVER_URL                   = ws://127.0.0.1:5000
Autoplay artifact dir:       D:\_DEV\...\production\qa\evidence\autoplay-runs\20260528-004221-Z

==== Autoplay smoke (DRY RUN -- skipped) ====
[DRY RUN] would launch: powershell -ExecutionPolicy Bypass -File .../Run-AutoplaySmoke.ps1 -Port 15873 -Recipe vs-bot -ArtifactDir ... -Python python -ClientStartupSecs 60 -DriverTicks 0 -DriverTimeoutSecs 300

==== Composite summary ====
Composite summary: D:\_DEV\...\composite-runs\2026-05-28-004221-autoplay-vs-bot\composite-summary.json
[DRY RUN] Simulated outcome: COMPLETE (no processes launched; exit=0 assumed).
```

**Exit code: 0**

---

## Key Findings

| Check | Result |
|---|---|
| `-Recipe vs-bot` accepted by `Start-AutoplayVsBot.ps1` | ✅ PASS |
| `vs-bot` propagated to `Run-AutoplaySmoke.ps1 -Recipe vs-bot` | ✅ PASS — visible in DRY RUN output |
| `vs-bot` registry key matches `vs_bot.NAME` | ✅ PASS — `NAME = "vs-bot"` |
| DRY RUN completes without Cargo build or process launch | ✅ PASS — exit 0 |
| Both child launchers found | ✅ PASS |
| `ccgs-play-main` stub warning (no Cargo.toml) | ⚠️ WARNING only — falls back to launcher root correctly |
| Live GUI | NOT attempted — HUMAN-GATE per spec |

### Warning Note

`D:\_DEV\ccgs-play-main` exists as a directory but contains no `Cargo.toml`.
The script correctly detects this, warns, and falls back to the launcher root (which has `Cargo.toml`).
This is the documented fallback path; no fix required. Operator can silence by setting `CCGS_PLAY_REPO_ROOT`.

---

## Validation

```
git diff --check: (no staged/unstaged whitespace errors — report-only commit)
```

No source edits, no Cargo builds, no production/** edits. Report file only.

---

## Status

All dry-run validation criteria met. Recipe propagation confirmed end-to-end:
`-Recipe vs-bot` → `Run-AutoplaySmoke.ps1 -Recipe vs-bot` → `REGISTRY["vs-bot"]` → `vs_bot.build`.

Live GUI run remains HUMAN-GATE (AUTOPLAY-VS-BOT-QA-001 live PASS requires operator sign-off).

1749: AUTOPLAY-VS-BOT-COMPOSITE-DRYRUN-VERIFY: SHIPPED
