# GCS Orchestrator — Onboarding

Stack for running a long-lived Codex orchestrator session (`019dddb4-...`)
backed by `codex app-server` (JSON-RPC over WebSocket on `:9787`), with
a persistent relay daemon, status sidecar, and a Textual TUI viewer.

## What you get after a clean install

| Component | Port | Purpose |
|---|---|---|
| `codex app-server` | 9787 (WS) | Hosts the orchestrator thread |
| `gcs-app-supervisor` | (none) | Wraps app-server with restart + sleep recovery |
| Relay daemon | 9789 (TCP) | Persistent WS reuse for worker DONE relays |
| Status sidecar | 9788 (HTTP) | Read-only health/last-turn/queue/version/metrics |
| `gcs-viewer-tui` | (TUI) | Live transcript + token gauge + speaker labels |

All co-located under one Windows service (`gcs-app-server`) once the
NSSM install is done.

## Fresh-install steps

### 0. Prerequisites

- Windows 10/11
- Python 3.10+ on PATH (`python --version`)
- Node.js with `@openai/codex` installed globally (`npm install -g @openai/codex`)
- Octogent running on port 8787 (separate stack — see Octogent docs)

### 1. Bootstrap

```powershell
cd D:\_DEV\Work\Claude-Code-Game-Studios\tools\gcs-orchestrator\scripts
PowerShell -ExecutionPolicy Bypass -File bootstrap.ps1
```

What it does:
- Verifies Python 3.10+
- `pip install -e .[tui]` the package (gets you all `gcs-*` entry points)
- Copies `templates/gcs.toml.example` → `~/.codex/gcs.toml`
- Probes Codex CLI + app-server health

### 2. Edit your `~/.codex/gcs.toml`

Set `session_id` to your orchestrator thread UUID. Find it in
`~/.codex/sessions/YYYY/MM/DD/rollout-*-<uuid>.jsonl` — the suffix UUID
is your session.

### 3. (Optional, recommended) NSSM service install

Auto-start at boot, survive logoff:

```powershell
PowerShell -ExecutionPolicy Bypass -File bootstrap.ps1 -InstallNssmService
```

Requires admin. Creates Windows service `gcs-app-server` that runs
`gcs-app-supervisor` (which spawns + monitors `codex app-server`).

## Daily operations

### Launch the viewer

```cmd
gcs-viewer-tui
```

- `Esc` interrupts the in-flight turn
- `Ctrl+Y` copies the last agent message to clipboard
- `Ctrl+Q` quits
- `Ctrl+P` opens Textual's command palette (includes "Save screenshot")

### Check status without the TUI

```cmd
curl http://127.0.0.1:9788/status
curl http://127.0.0.1:9788/last-turn
curl http://127.0.0.1:9788/queue
curl http://127.0.0.1:9788/version
curl http://127.0.0.1:9788/metrics
```

### Search the rollout (~90 MB JSONL)

```cmd
gcs-rollout-grep "lobby confirm state"
gcs-rollout-grep --role assistant --since 24h "PROMPT-833"
```

### Worker DONE timeline

```cmd
gcs-relay-history --since 1h
gcs-relay-history --status failed
```

### Orphan / silent-loss detection

```cmd
gcs-reconcile --since 24h
gcs-reconcile --since 7d --status orphan-report
gcs-reconcile --json | jq .
```

### Manual relay (debug)

```cmd
echo "DONE PROMPT-999: TEST" | python -m gcs_orchestrator.relay <thread-id> -
```

## Troubleshooting

### "app-server unreachable"

```cmd
# Is the service up?
Get-Service gcs-app-server

# Probe directly
curl http://127.0.0.1:9787/readyz

# Restart manually
nssm restart gcs-app-server
# OR run supervisor manually for foreground debug
gcs-app-supervisor
```

### "relay daemon failed, fallback engaged"

The relay falls back to one-shot subprocess if the daemon is unreachable.
Check supervisor.log for `relay daemon NOT started: ...` or
`daemon transport connect failed`.

### Viewer shows truncated text / mis-wrapped output

Should not happen on the current viewer — markup escaping + line buffering
+ blank collapse are all enabled. If you see weird wrapping, save a SVG
screenshot (`Ctrl+P` → "Save screenshot") and check `D:\Downloads`.

### Rollout corruption

```cmd
gcs-rollout-check %USERPROFILE%\.codex\sessions\... --full
gcs-rollout-check ... --repair
```

`--repair` truncates the rollout to the last valid newline and writes
the chopped bytes to a `.dropped` sidecar for forensics.

## File / data locations

| Path | Contents |
|---|---|
| `~/.codex/gcs.toml` | Unified config (this stack's settings) |
| `~/.codex/sessions/.../rollout-*-<uuid>.jsonl` | Full conversation log (90+ MB) |
| `%LOCALAPPDATA%/gcs-app-relay/` | Per-machine state |
| ↳ `relay.log` | Relay activity (verbose) |
| ↳ `daemon.log` | Relay daemon activity |
| ↳ `supervisor.log` | Supervisor activity |
| ↳ `metrics.jsonl` | Per-turn latency records (size-rotated) |
| ↳ `receipts/` | sha256-keyed idempotency receipts |
| ↳ `turn.lock` | Cross-process serialization lock |
| `~/.codex/dispatch-audit.jsonl` | Dispatcher decision audit log |
| `~/.codex/gcs-dispatch.log` | Dispatcher activity (verbose) |
| `~/.codex/gcs-spawn-watchdog.log` | Per-spawn reliability watchdog |
| `D:\_DEV\Work\Claude-Code-Game-Studios\reports\` | Worker DONE reports |

## Architecture diagram

See `docs/octogent-integration.md` Section 9-bis in the main repo for the
full architecture, boot procedure, and rollback steps.
