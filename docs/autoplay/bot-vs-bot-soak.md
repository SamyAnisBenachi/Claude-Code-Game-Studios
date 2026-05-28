# Bot-vs-Bot Soak — Standalone Runbook

_Added: PROMPT 1770 — 2026-05-28_
_Satisfies: BOT-SOAK-ENTRYPOINT-001 AC7_

---

## Purpose

`tools/dev-launcher/Start-BotVsBotSoak.ps1` launches a fully headless bot-vs-bot
soak: no human client, no recipe driver. The server runs two bots autonomously via
`BotLobbyPlugin` + `BotActionLoopPlugin`. The `bot-soak-trigger` binary acts as the
human-proxy client that creates the room, then idles while the bots play.

Use this runbook to:
- Verify bot round-loop correctness in isolation (no UI noise)
- Produce per-round QA snapshot evidence for story acceptance
- Run a bounded soak (via `-MaxRounds`) as part of a CI-adjacent gate
- Provide the soak server for a composite autoplay-vs-bot run (see
  [autoplay-vs-bot-flow.md](autoplay-vs-bot-flow.md))

---

## Environment Gating (AC6)

The bot soak path is **debug/dev-only** and protected by a server-side env gate
(`CCGS_BOT_SOAK_ENABLED`). The gate was added in commit `686f2c3f` (PROMPT 1743).

The launcher script **does not** set `CCGS_BOT_SOAK_ENABLED` automatically — the
operator must have it present in their shell environment before launching:

```powershell
$env:CCGS_BOT_SOAK_ENABLED = "1"
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1
```

Without this var the server will reject the bot-room creation request. The server's
unit tests (7 tests, commit `686f2c3f`) verify the gate logic directly.

> **Debug builds only.** Never set `CCGS_BOT_SOAK_ENABLED` in a production deploy.
> The env var must not appear in Railway / Vercel environment config.

---

## Repo / Play Root Assumptions

The launcher resolves the build/play checkout through the following fallback chain
(highest priority first):

| Priority | Source | Path |
|---|---|---|
| 1 | `-PlayRepoRoot` argument | Explicit override |
| 2 | `$env:CCGS_PLAY_REPO_ROOT` | Env var |
| 3 | `$env:CCGS_CANONICAL_MAIN_ROOT` | Legacy alias |
| 4 | Dedicated default | `D:\_DEV\ccgs-play-main` |
| 5 | Launcher parent-of-parent | Fallback; emits a warning |

The resolved root must contain `Cargo.toml` at its top level (workspace root check).
If not, the launcher exits 1.

**Recommended setup**: keep a dedicated `D:\_DEV\ccgs-play-main` checkout on latest
`origin/main` and rebuild binaries there. The two-checkout model avoids polluting
the working checkout with stale binaries.

---

## Quick Start

```powershell
# Minimal run — 5-minute soak, default port 5000
$env:CCGS_BOT_SOAK_ENABLED = "1"
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1
```

```powershell
# One-click launcher at repo root (wraps the above)
start-bot-vs-bot-soak.bat
```

```powershell
# Bounded by round count (exits cleanly on MaxRoundsReached)
$env:CCGS_BOT_SOAK_ENABLED = "1"
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 `
    -MaxRounds 3 -DurationSeconds 180
```

```powershell
# Force-rebuild both binaries + use port 5050
$env:CCGS_BOT_SOAK_ENABLED = "1"
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 `
    -Rebuild -Port 5050
```

```powershell
# Dry-run: prints every step without starting any process
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 `
    -DryRun
```

---

## Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `-Port` | int | `5000` | Server bind port. Auto-bumps to next free port (up to +50) unless `-StrictPort` is passed. |
| `-StrictPort` | switch | off | Fail immediately (exit 2) if `-Port` is occupied. |
| `-Release` | switch | off | Build and run the release-profile binary instead of debug. |
| `-ServerWaitSeconds` | int | `8` | How long to poll for the server port to become occupied after launch. |
| `-DurationSeconds` | int | `300` | Overall soak wall-clock limit in seconds. Passed to `bot-soak-trigger` as `--overall-timeout-secs`. |
| `-MaxRounds` | int | `0` | If > 0, sets `CCGS_BOT_MAX_ROUNDS=N` so the server emits `GameOver (MaxRoundsReached)` after N completed rounds. `bot-soak-trigger` exits cleanly when it receives `S2CGameOver`. `0` = unlimited (timer-only). |
| `-Rebuild` | switch | off | Force `cargo build` for both `server` and `bot-soak-trigger` even if binaries are up-to-date. Use after `git pull`. |
| `-DryRun` | switch | off | Print all steps (including build commands) without starting any process. Safe to combine with `-Rebuild`. |
| `-PlayRepoRoot` | string | `''` | Explicit override for the build/play checkout root (see §Repo / Play Root Assumptions). |
| `-Help` | switch | off | Print inline help and exit. |

---

## Environment Variables Set by the Launcher

The launcher injects these into the server process environment before starting it:

| Variable | Value | Purpose |
|---|---|---|
| `SERVER_PORT` | chosen port | Server bind port |
| `SERVER_URL` | `ws://127.0.0.1:<port>` | WebSocket URL for trigger client |
| `CCGS_BOT_QA_SNAPSHOT` | `1` | Activates `BotQaSnapshotPlugin` in the server |
| `CCGS_BOT_QA_SNAPSHOT_DIR` | `<evidence>/server-snapshots/` | Per-round snapshot output directory |
| `CCGS_BOT_DECISION_LOG_PATH` | `<evidence>/bot-decision-log.jsonl` | Decision log output file |
| `CCGS_BOT_MAX_ROUNDS` | N (only when `-MaxRounds N` > 0) | Round-count bound; cleared when disabled |

The operator must separately set `CCGS_BOT_SOAK_ENABLED=1` before launching (see
§Environment Gating).

---

## Binary Build Logic

The launcher builds two binaries:

| Binary | Cargo package | Source dirs watched |
|---|---|---|
| `server.exe` | `-p server` | `server/src/`, `shared/src/`, `Cargo.toml`, `Cargo.lock` |
| `bot-soak-trigger.exe` | `-p two-client-runtime --bin bot-soak-trigger` | `tools/two-client-runtime/src/`, `shared/src/`, `Cargo.toml`, `Cargo.lock` |

Freshness logic (PROMPT 1679):
- If any watched source file is **newer** than the binary → rebuilds automatically.
- `-Rebuild` forces a build regardless of file timestamps. Always use after `git pull`.
- CARGO_TARGET_DIR is fixed to `D:\_DEV\cargo-target\ccgs-msvc` (MSVC policy).

---

## Evidence Output Layout

Each run creates a timestamped directory:

```
production/qa/evidence/dev-runs/
└── <UTC-YYYY-MM-DD-HHMMSS>-bot-vs-bot-soak/
    ├── soak-summary.json           ← launcher outcome; start here
    ├── server.log                  ← server stdout
    ├── server.err                  ← server stderr
    ├── bot-decision-log.jsonl      ← per-decision BotDecisionLog entries
    ├── server-snapshots/           ← per-round QA snapshots (BotQaSnapshotPlugin)
    └── bot-soak-trigger/
        ├── final_state.json        ← trigger's exit summary
        ├── bot-soak-trigger.log    ← trigger stdout
        └── bot-soak-trigger.err    ← trigger stderr
```

### soak-summary.json fields (key fields)

| Field | What it tells you |
|---|---|
| `trigger_exit_code` | The outcome of `bot-soak-trigger` — `0` = soak passed |
| `trigger_exit_code_source` | How the exit code was obtained (`process`, `final_state.json`, `timeout-forced`, ...) |
| `started_utc` / `stopped_utc` | ISO-8601 timestamps |
| `server_port` | Port the server actually bound |
| `server_build_reason` | Why server was (re)built: `missing`, `source-newer`, `forced`, `up-to-date` |
| `trigger_build_reason` | Why trigger was (re)built |
| `ccgs_bot_max_rounds` | `null` if round-count bound was disabled |
| `dry_run` | `true` if no process was started |

### bot-soak-trigger/final_state.json fields

Written by the trigger binary before exit:

| Field | Meaning |
|---|---|
| `exit_code` | Integer exit code (0 = clean game-over or max-rounds; non-zero = timeout or error) |
| `endpoint_reached` | Which protocol endpoint caused exit (e.g. `game_over`, `max_rounds_reached`, `overall_timeout`) |

> **Null exit-code reconciliation**: On Windows, `Start-Process -PassThru` can return
> `null` for `ExitCode` even after `WaitForExit` succeeds (handle-release race). The
> launcher automatically reconciles from `final_state.json` when this happens. The
> `trigger_exit_code_source` field in `soak-summary.json` records which source was used.

---

## Exit Codes

| Code | Source | Meaning |
|---|---|---|
| `0` | launcher | Soak ran to completion; `bot-soak-trigger` exited 0 |
| `1` | launcher | Generic failure: build failed, no free port, file I/O error, or trigger exited non-zero |
| `2` | launcher | Port unavailable and `-StrictPort` was passed |
| `3` | launcher | Server failed to bind within `-ServerWaitSeconds` |

Launcher exit 0 means the trigger reached its `S2CGameOver` endpoint cleanly (via
`-MaxRounds`) or ran for the full `-DurationSeconds` and exited 0 on its own. A
launcher exit 1 without a build or port error means the trigger itself failed —
read `bot-soak-trigger.log` and `bot-soak-trigger.err`, and check `final_state.json`
for `endpoint_reached`.

---

## Max Rounds Behavior

When `-MaxRounds N` is passed:

1. The launcher sets `CCGS_BOT_MAX_ROUNDS=N` in the server environment.
2. The server's `BotRoundCountPlugin` counts completed rounds. After round N it
   emits `GameOverReason::MaxRoundsReached` and shuts down cleanly.
3. `bot-soak-trigger` receives `S2CGameOver` and exits 0 before the
   `-DurationSeconds` wall-clock timer fires.
4. The launcher exits 0.

Without `-MaxRounds` the soak runs until `-DurationSeconds` elapses (default 5 min),
at which point the launcher calls `Stop-Process` on the server and checks the
trigger's exit code.

---

## Snapshot and Decision Log Locations

| Output | Location | How to enable |
|---|---|---|
| Per-round QA snapshots | `<evidence>/server-snapshots/` | Always enabled by the launcher (`CCGS_BOT_QA_SNAPSHOT=1`) |
| Bot decision log | `<evidence>/bot-decision-log.jsonl` | Always enabled by the launcher (`CCGS_BOT_DECISION_LOG_PATH` wired) |
| Bot debug overlay | Press **F8** in a running client | Set `CCGS_BOT_DEBUG_UI=1` (server) + `CCGS_DEBUG_UI=1` (client) — see §Observability |

The snapshot dir and decision log path are wired at launch; the server writes to
them autonomously via `BotQaSnapshotPlugin`. No operator action needed beyond
running the launcher.

---

## Observability

### Bot debug overlay (F8)

Useful when combining the soak server with a human-controlled client:

```powershell
$env:CCGS_BOT_SOAK_ENABLED = "1"
$env:CCGS_BOT_DEBUG_UI     = "1"   # server: enables S2CDebugBotStatePush
$env:CCGS_DEBUG_UI          = "1"   # client: enables F8 corner panel
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1
```

Then launch a client separately. Press **F8** to toggle the overlay. Does not
affect game state.

### Live server log

While the soak runs, tail `server.log` from the evidence directory:

```powershell
Get-Content -Wait -Path production\qa\evidence\dev-runs\<stamp>-bot-vs-bot-soak\server.log
```

Look for `vs-bot-post-resolution` and round-completion markers to confirm the bots
are cycling through rounds.

---

## Operator Checklist

Run through this list before marking evidence as valid for a story acceptance gate.

```
[ ] CCGS_BOT_SOAK_ENABLED=1 is set in the current shell
[ ] Working directory is the CCGS workspace root (Cargo.toml present)
[ ] On latest origin/main (git fetch + git status clean)
[ ] After git pull: use -Rebuild to guarantee fresh binaries
[ ] Chose -MaxRounds N for a bounded run (recommended for CI-adjacent gates)
[ ] Run completed: launcher exit code = 0
[ ] soak-summary.json: trigger_exit_code = 0
[ ] bot-soak-trigger/final_state.json: endpoint_reached confirms clean exit
[ ] server-snapshots/ is non-empty (BotQaSnapshotPlugin wrote at least one snapshot)
[ ] bot-decision-log.jsonl is non-empty (decision log stream active)
[ ] server.log: no ERROR or panicked at lines
[ ] Evidence directory archived or noted in the story evidence doc
```

---

## Common Failures

| Symptom | Likely cause | Fix |
|---|---|---|
| Server exits immediately after launch | `CCGS_BOT_SOAK_ENABLED` not set | Set `$env:CCGS_BOT_SOAK_ENABLED = "1"` before running the launcher |
| Launcher exits 3 (port timeout) | Server crashed before binding | Check `server.err` for `panicked at` or `error` lines |
| Launcher exits 2 (strict port) | Chosen port already in use | Remove `-StrictPort` (auto-bump) or pick a different `-Port` |
| Launcher exits 1 (build failure) | Compile error | Run `cargo check -p server` / `cargo check -p two-client-runtime` manually |
| `server-snapshots/` is empty | Server built before `CCGS_BOT_QA_SNAPSHOT` feature | Use `-Rebuild` to force a fresh build |
| `trigger_exit_code` = -1 or `timeout-forced` | Trigger hung and was force-killed | Check `bot-soak-trigger.err`; try `-MaxRounds 2 -DurationSeconds 120` for a shorter bounded run |
| `trigger_exit_code_source` = `final_state.json(parse-error)` | Trigger binary crashed without clean exit | Read raw `bot-soak-trigger.log` and `bot-soak-trigger.err` |
| Stale binaries after git pull | Freshness guard may miss some changes | Always use `-Rebuild` after `git pull` |

---

## Relation to Other Launchers

```
Start-BotVsBotSoak.ps1          ← THIS RUNBOOK — headless bot-vs-bot only
    │
    ├── used standalone: this runbook
    └── used as child of Start-AutoplayVsBot.ps1:
            → see docs/autoplay/autoplay-vs-bot-flow.md

Run-AutoplaySmoke.ps1           ← recipe-driven client; no bot server
    → see docs/autoplay/evidence-operator-guide.md

Start-AutoplayVsBot.ps1         ← composite: soak server + autoplay client
    → see docs/autoplay/autoplay-vs-bot-flow.md
```

`Start-BotVsBotSoak.ps1` does NOT:
- Spawn a human-facing client (headless only)
- Run tests, story-done, smoke, or any QA workflow automation
- Fetch, pull, merge, push, or modify git
- Edit `production/` or session-state files (except the evidence directory it creates)

---

_Script source: `tools/dev-launcher/Start-BotVsBotSoak.ps1`_
_Story: BOT-SOAK-ENTRYPOINT-001 (Sprint 19 candidate)_
