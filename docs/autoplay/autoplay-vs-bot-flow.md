# Autoplay-vs-Bot Composite Flow

_Added: PROMPT 1644 — 2026-05-27_

---

## Overview

The autoplay-vs-bot composite flow coordinates two independent processes — a
headless bot server and a recipe-driven autoplay client — into a single
evidence-producing run. The `Start-AutoplayVsBot.ps1` launcher owns the
orchestration; it delegates to the existing child launchers rather than
reimplementing their logic.

```
Operator terminal
│
└─ Start-AutoplayVsBot.ps1
   ├─ [precondition] Interactive desktop session check → exit 10 if headless
   ├─ [precondition] Child launcher files present → exit 11 if missing
   │
   ├─ [optional] Start-BotVsBotSoak.ps1 (background PowerShell job)
   │    └─ server.exe listening on <PORT>
   │
   ├─ wait for soak server port to bind (up to -SoakReadySecs)
   │    └─ exit 12 if not bound in time
   │
   ├─ set env vars:
   │    CCGS_AUTOPLAY_BOT_ROOM_READY=1
   │    SERVER_PORT=<PORT>
   │    SERVER_URL=ws://127.0.0.1:<PORT>
   │
   ├─ Run-AutoplaySmoke.ps1 -Recipe <recipe>
   │    ├─ cargo build -p client --features autoplay-remote
   │    ├─ launch Bevy client (visible window)
   │    └─ python driver.py --recipe <recipe>
   │
   ├─ stop background soak job
   └─ write composite-summary.json
```

---

## Prerequisites

| Check | Command / Detail |
|---|---|
| Interactive desktop | Launcher exits 10 (`BLOCKED-HUMAN-GUI`) if not interactive |
| `Start-BotVsBotSoak.ps1` | Required unless `-SkipSoakLaunch` is passed |
| `Run-AutoplaySmoke.ps1` | Always required |
| Python 3.8+ on PATH | Used by the autoplay driver |
| `CCGS_AUTOPLAY_BOT_ROOM_READY` | Set automatically by the composite launcher |

---

## Quickstart

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios (or the play/build checkout)
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1
```

Launches the bot soak server, waits for it to bind, runs the `full-game`
recipe, and writes evidence to `production/qa/evidence/composite-runs/`.

### Dry run (no processes launched)

```powershell
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -DryRun
```

### Use an already-running bot server

```powershell
# Terminal 1 — start the soak server separately
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1

# Terminal 2 — composite launcher, skip soak launch
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -SkipSoakLaunch -Port 5000
```

### Non-default recipe

```powershell
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -Recipe smoke
```

---

## Parameters

| Parameter | Default | Description |
|---|---|---|
| `-Port` | `5000` | Bot server bind port |
| `-StrictPort` | off | Fail if port busy (no auto-bump) |
| `-Release` | off | Build/run release-profile server |
| `-RpcPort` | `15873` | Autoplay RPC port for the client |
| `-Recipe` | `full-game` | Recipe passed to `Run-AutoplaySmoke.ps1` |
| `-SkipSoakLaunch` | off | Assume server already running; skip `Start-BotVsBotSoak.ps1` |
| `-SoakReadySecs` | `20` | Max seconds to wait for soak server to bind |
| `-SoakDurationSeconds` | `300` | Soak window forwarded to child launcher |
| `-ClientStartupSecs` | `60` | Max seconds to wait for client RPC bind |
| `-Python` | `python` | Python executable |
| `-DryRun` | off | Print every step without launching |
| `-PlayRepoRoot` | _(auto)_ | Explicit play/build checkout path |

---

## Evidence Output

```
production/qa/evidence/composite-runs/
└── YYYY-MM-DD-HHMMSS-autoplay-vs-bot/
    ├── composite-summary.json     ← outcome + child exit codes  ← START HERE
    └── autoplay-run-path.txt      ← path to the autoplay artifact dir
```

The autoplay artifacts (screenshots, driver log, checkpoints) land under
`production/qa/evidence/autoplay-runs/<TIMESTAMP>/` as usual — the
composite run references that path in `autoplay-run-path.txt` rather than
duplicating the artifacts.

### `composite-summary.json` fields

| Field | Meaning |
|---|---|
| `outcome` | `ok`, `blocked-recipe-guard`, `blocked-human-gui`, `blocked-precondition`, `blocked-soak-timeout`, `smoke_failed_exit_N` |
| `smoke_exit_code` | Exit code from `Run-AutoplaySmoke.ps1` |
| `autoplay_artifact_dir` | Path to autoplay run artifacts |
| `live_pass_status` | Always `NOT-CLAIMED` — live PASS requires human sign-off |

---

## Exit Codes

| Code | Label | Meaning |
|---|---|---|
| `0` | success | Recipe completed cleanly |
| `1` | failure | Generic error (port, file I/O, process error) |
| `4` | blocked-recipe-guard | Driver emitted `local.block` (recipe guard fired) |
| `10` | BLOCKED-HUMAN-GUI | Non-interactive session; Bevy needs a visible desktop |
| `11` | BLOCKED-PRECONDITION | Soak server absent and `Start-BotVsBotSoak.ps1` missing, or `Run-AutoplaySmoke.ps1` missing |
| `12` | BLOCKED-PRECONDITION | Soak server did not bind within `-SoakReadySecs` |

---

## Live PASS Gate

This harness implements the **runner/scaffold** for `AUTOPLAY-VS-BOT-QA-001`
(Story 004). It does **not** claim or record a live PASS.

A live PASS for Story 004 requires:

1. Operator runs `Start-AutoplayVsBot.ps1` (or equivalent) in an interactive
   session with a visible desktop.
2. Recipe `full-game` completes with exit 0.
3. `composite-summary.json` → `outcome: ok`.
4. `checkpoints.jsonl` contains `full-game-resolution`.
5. Operator attaches `composite-summary.json` + screenshot pairs to the story
   evidence and signs off.

Until steps 1–5 are completed by a human operator, `GAP-01` and `GAP-02`
remain open and the story remains in `Draft`.

---

## Relation to Other Launchers

| Launcher | Purpose | Relation |
|---|---|---|
| `Start-BotVsBotSoak.ps1` | Headless bot server, wall-clock timer | Child of this launcher (or run separately with `-SkipSoakLaunch`) |
| `Run-AutoplaySmoke.ps1` | Build client + drive recipe | Child of this launcher |
| `Start-TwoClients.ps1` | Two human-driven clients | Independent; not used here |
| `Start-AutoplayVsBot.ps1` | **This file** — composite orchestrator | Coordinates the two above |

---

_Last updated: PROMPT 1644 — 2026-05-27_
