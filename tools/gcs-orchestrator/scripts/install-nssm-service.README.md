# NSSM Windows Service for `gcs-app-supervisor`

The supervisor wraps `codex app-server` and restarts it on crash, HTTP
probe failure, or JSON-RPC liveness probe failure. Running it as a
Windows service via NSSM lets it survive logoff/logon, auto-start at
boot, and have proper rotated logging.

## One-time setup

1. Install NSSM from https://nssm.cc/download. Put `nssm.exe` somewhere
   on PATH (e.g. `C:\Windows\System32`).

2. Ensure the gcs-orchestrator package is installed in your Python:

   ```cmd
   pip install -e D:\_DEV\Work\Claude-Code-Game-Studios\tools\gcs-orchestrator
   ```

3. Run the install script **as Administrator**:

   ```powershell
   cd D:\_DEV\Work\Claude-Code-Game-Studios\tools\gcs-orchestrator\scripts
   PowerShell -ExecutionPolicy Bypass -File install-nssm-service.ps1
   ```

   This creates and starts the `gcs-app-server` service.

## What gets configured

| Setting | Value |
|---|---|
| Service name | `gcs-app-server` |
| Executable | `D:\_APPS\Python312\python.exe -m gcs_orchestrator.supervisor` |
| Start mode | `SERVICE_AUTO_START` (boot start) |
| Restart delay | 5 s after exit |
| Exit policy | `Default Restart` (always restart on any exit code) |
| Stop method | Console signal first, 30 s grace, then kill |
| Stdout/stderr | `%LOCALAPPDATA%/gcs-app-relay/service-logs/` (10 MB rotation, daily) |
| Supervisor log | `%LOCALAPPDATA%/gcs-app-relay/supervisor.log` (rotated by supervisor itself) |

The supervisor's own probes (HTTP `/readyz`, JSON-RPC `initialize`) detect
stuck-but-alive states that NSSM's process-liveness can't.

## Verifying

```cmd
:: Service status
Get-Service gcs-app-server

:: Recent supervisor activity
Get-Content $env:LOCALAPPDATA\gcs-app-relay\supervisor.log -Tail 20

:: Quick health probe
curl http://127.0.0.1:9787/readyz
```

## Daily operations

```cmd
:: Stop the service
nssm stop gcs-app-server

:: Start the service
nssm start gcs-app-server

:: Restart (manual)
nssm restart gcs-app-server

:: Edit settings (opens GUI)
nssm edit gcs-app-server

:: Remove entirely
nssm stop gcs-app-server
nssm remove gcs-app-server confirm
```

## Behavior on PC sleep/wake

The supervisor wires up a `PowerEventWatcher` that detects wall-clock
drift relative to monotonic time (a sleep signature). On resume, it
forces a probe cycle immediately rather than waiting for the next
30-second tick. If the probes fail (half-open WS sockets after wake),
the supervisor restarts the app-server within ~30 s.

## Rollback (if the service misbehaves)

```cmd
nssm stop gcs-app-server
nssm remove gcs-app-server confirm

:: Then run app-server manually as before
codex app-server --listen ws://127.0.0.1:9787
```
