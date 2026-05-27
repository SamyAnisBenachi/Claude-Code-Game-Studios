# PROMPT 1671 -- BOT-SOAK-LAUNCHER-PORT-DETECTION-REPAIR

**Date:** 2026-05-27
**Source-of-truth at run:** `origin/main@346c5e9d` (rebased onto, merged as `4060737a`)
**Scope:** `tools/dev-launcher/Start-BotVsBotSoak.ps1` — port-detection bug repair.

---

## Summary

**Verdict: SHIPPED**

Replaced the `TcpListener.Start()` probe in `Test-PortFree` with a
`TcpClient.BeginConnect()` probe. Added a `HasExited` guard to the readiness
loop with distinct error messages for "server crashed" vs "server timed out".
Static parse: 0 errors. Probe unit tests: all 3 cases pass. Dry-run: clean.
Committed `4060737a`, merged to `main`.

---

## Root Cause

`Test-PortFree` created a `TcpListener` on `127.0.0.1:$P`. On Windows with
`SO_REUSEADDR` (the .NET default), this succeeds even when a server is already
bound on `0.0.0.0:$P` — the wildcard address does not block a loopback-specific
bind. The result: `Test-PortFree` always returned `$true` ("free") regardless of
server state. The post-start readiness loop polled this function and never saw
`$false`, so `$bound` never became `$true`, the loop expired, and the launcher
killed the running server and exited 3.

---

## Fix Applied

### `Test-PortFree` — replaced `TcpListener.Start()` with `TcpClient.BeginConnect()`

```powershell
function Test-PortFree {
    param([int]$P)
    # TCP connect probe: the old TcpListener.Start() approach was a false-negative
    # on Windows -- binding 127.0.0.1:$P succeeds even when a server is already
    # listening on 0.0.0.0:$P (SO_REUSEADDR + wildcard vs. specific-address semantics).
    # A connect attempt is reliable: ConnectionRefused means free; success means occupied.
    # Returns $true (free) / $false (occupied).
    $client = $null
    try {
        $client = [System.Net.Sockets.TcpClient]::new()
        $ar = $client.BeginConnect([System.Net.IPAddress]::Loopback, $P, $null, $null)
        $connected = $ar.AsyncWaitHandle.WaitOne(300)   # 300 ms timeout
        if ($connected -and $client.Connected) { return $false }
        return $true
    } catch {
        return $true
    } finally {
        if ($null -ne $client) { try { $client.Close() } catch {} }
    }
}
```

A connect attempt to `127.0.0.1:$P` is reliably refused if nothing is listening,
regardless of the server's bind address (`0.0.0.0` or `127.0.0.1`). The 300 ms
`WaitOne` timeout keeps each probe fast; `ConnectionRefused` returns in < 1 ms.

### Readiness loop — added `HasExited` guard and distinct error messages

```powershell
$exitedEarly = $false
while ((Get-Date) -lt $deadline) {
    if ($serverProc.HasExited) { $exitedEarly = $true; break }
    if (-not (Test-PortFree $chosen)) { $bound = $true; break }
    Start-Sleep -Milliseconds 250
}
if (-not $bound) {
    if ($exitedEarly) {
        Write-Host -ForegroundColor Red "Server failed to start: process exited (code $($serverProc.ExitCode)) before binding port $chosen."
    } else {
        Write-Host -ForegroundColor Red "Server timed out: port $chosen was not reachable within $ServerWaitSeconds s (process still running)."
        try { Stop-Process -Id $serverProc.Id -Force -ErrorAction Stop } catch {}
    }
    exit 3
}
Write-Host "Server bound port $chosen (PID $($serverProc.Id) alive)."
```

Distinguishes three outcomes:
- **Bound** → soak proceeds normally
- **Crashed** (`HasExited=true`) → no `Stop-Process` needed; message shows exit code
- **Timed out** (process alive, no bind) → `Stop-Process` to clean up

---

## Validation

### Static parse

```
PARSE OK: 0 syntax errors
```

### git diff --check

```
diff-check OK
```

### TCP connect probe unit tests

```powershell
# Test 1: no listener
No-listener (19876) -> free=True   [expect True]  ✓

# Test 2: TcpListener on 0.0.0.0:19877 (the bug case)
0.0.0.0 listener (19877) -> free=False  [expect False]  ✓

# Test 3: TcpListener on 127.0.0.1:19878
127.0.0.1 listener (19878) -> free=False  [expect False]  ✓
```

The fix correctly detects both wildcard (`0.0.0.0`) and loopback-only
(`127.0.0.1`) listeners. The old `TcpListener` approach returned `True` (free)
for test 2; the new `TcpClient` approach returns `False` (occupied) as required.

### Dry-run

```
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1
    -DryRun -PlayRepoRoot D:\_DEV\Work\Claude-Code-Game-Studios
```

All sections executed cleanly. Port 5000 correctly identified as free (no server
running). `-DryRun` path skips server start and soak sleep as designed.

---

## Files Modified

| File | Change |
|------|--------|
| `tools/dev-launcher/Start-BotVsBotSoak.ps1` | Replace `Test-PortFree` body; add `HasExited` guard + improved error messages to readiness loop |

---

## Files Created

| File | Purpose |
|------|---------|
| `reports/PROMPT-1671-bot-soak-launcher-port-detection-repair.md` | This report |

---

## Commit

```
4060737a fix(launcher): replace TcpListener port probe with TCP connect in Start-BotVsBotSoak.ps1
```

Branch: `fix/1671-bot-soak-port-detection` → merged to `main`

---

## Scope Boundary

- **Touched**: `tools/dev-launcher/Start-BotVsBotSoak.ps1`, `reports/PROMPT-1671-*.md`
- **Not touched**: Rust source, Cargo.toml, CI files, production/, session-state/,
  sprint files, any other launcher scripts

---

1671: BOT-SOAK-LAUNCHER-PORT-DETECTION-REPAIR: SHIPPED
