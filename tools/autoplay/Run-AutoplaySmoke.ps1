# PROMPT 1595 -- autoplay BRP smoke launcher.
#
# Launches a single visible client with the dev-only `autoplay-remote` Cargo
# feature and CCGS_AUTOPLAY=1, waits for the RPC port to come up, runs the
# Python driver against the `smoke` recipe, then shuts the client down.
#
# This is the smallest practical autoplay smoke. It proves:
#   - the client builds with `--features autoplay-remote`,
#   - the RPC server binds on the expected port,
#   - capabilities + status + input + clear_input + screenshot all succeed,
#   - artifacts land in the expected directory.
#
# Out-of-scope here:
#   - exercising bot-vs-driven gameplay (deferred; see docs/autoplay.md)
#   - headless mode (no headless Cargo feature exists yet)
#
# Usage from the repo root (works with both PowerShell 5.1 and 7+):
#   powershell -File tools/autoplay/Run-AutoplaySmoke.ps1
#   pwsh       -File tools/autoplay/Run-AutoplaySmoke.ps1
#   pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Port 15874 -ArtifactDir D:/Tmp/autoplay-test
#   pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe full-game

[CmdletBinding()]
param(
    [int]$Port = 15873,
    [string]$ArtifactDir = "",
    [string]$Recipe = "smoke",
    [string]$Python = "python",
    [int]$DriverTicks = 0,
    [double]$DriverHz = 10.0,
    [int]$DriverTimeoutSecs = 300,
    [int]$ClientStartupSecs = 60
)

$ErrorActionPreference = 'Stop'

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Set-Location $repoRoot

if ([string]::IsNullOrWhiteSpace($ArtifactDir)) {
    $stamp = [DateTime]::UtcNow.ToString("yyyyMMdd-HHmmss") + "-Z"
    $ArtifactDir = Join-Path "production/qa/evidence/autoplay-runs" $stamp
}

New-Item -ItemType Directory -Path $ArtifactDir -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $ArtifactDir "screenshots") -Force | Out-Null

$logPath = Join-Path $ArtifactDir "process.log"
$launcherStatusPath = Join-Path $ArtifactDir "launcher-status.json"

Write-Host "[autoplay-smoke] repo=$repoRoot port=$Port artifact_dir=$ArtifactDir"

# Build first so the client launch does not have to wait inside the timeout window.
Write-Host "[autoplay-smoke] cargo build -p client --features autoplay-remote"
$build = Start-Process -FilePath "cargo" -ArgumentList @(
    "build","-p","client","--features","autoplay-remote"
) -NoNewWindow -PassThru -Wait
if ($build.ExitCode -ne 0) {
    Write-Error "cargo build failed with exit code $($build.ExitCode)"
    exit 2
}

# Launch the client. Env vars activate the autoplay harness; the launcher
# captures stdout+stderr to process.log so the report has a paper trail.
$startedAt = [DateTime]::UtcNow.ToString("o")
$env:CCGS_AUTOPLAY = "1"
$env:CCGS_AUTOPLAY_PORT = "$Port"
$env:CCGS_AUTOPLAY_ARTIFACT_DIR = (Resolve-Path $ArtifactDir).Path

Write-Host "[autoplay-smoke] launching client (env CCGS_AUTOPLAY=1 CCGS_AUTOPLAY_PORT=$Port)"
$client = Start-Process -FilePath "cargo" -ArgumentList @(
    "run","-p","client","--features","autoplay-remote"
) -NoNewWindow -PassThru -RedirectStandardOutput $logPath -RedirectStandardError "$logPath.err"

# Wait for the RPC port to accept connections, capped at $ClientStartupSecs.
$deadline = (Get-Date).AddSeconds($ClientStartupSecs)
$ready = $false
while ((Get-Date) -lt $deadline -and -not $client.HasExited) {
    try {
        $tcp = New-Object System.Net.Sockets.TcpClient
        $tcp.Connect("127.0.0.1", $Port)
        $tcp.Close()
        $ready = $true
        break
    } catch {
        Start-Sleep -Milliseconds 500
    }
}
if (-not $ready) {
    Write-Warning "[autoplay-smoke] client did not bind RPC port within $ClientStartupSecs s; aborting"
    if (-not $client.HasExited) {
        $client.Kill() | Out-Null
    }
    $finishedAt = [DateTime]::UtcNow.ToString("o")
    Set-Content -Path $launcherStatusPath -Value (@{
        schema           = "autoplay_launcher_status_v1"
        outcome          = "rpc_port_never_bound"
        port             = $Port
        artifact_dir     = $ArtifactDir
        started_at       = $startedAt
        finished_at      = $finishedAt
        client_exit_code = $client.ExitCode
        log_path         = "process.log"
    } | ConvertTo-Json -Depth 4)
    exit 3
}

Write-Host "[autoplay-smoke] RPC port bound; running driver (recipe=$Recipe ticks=$DriverTicks hz=$DriverHz)"
$driverPath = Join-Path $PSScriptRoot "driver.py"
$driver = Start-Process -FilePath $Python -ArgumentList @(
    $driverPath,
    "--port", $Port,
    "--artifact-dir", $ArtifactDir,
    "--recipe", $Recipe,
    "--ticks", $DriverTicks,
    "--hz", $DriverHz,
    "--timeout", $DriverTimeoutSecs
) -NoNewWindow -PassThru -Wait
$driverExit = $driver.ExitCode

Write-Host "[autoplay-smoke] driver exit=$driverExit; stopping client"
if (-not $client.HasExited) {
    $client.Kill() | Out-Null
    Start-Sleep -Seconds 1
}

$finishedAt = [DateTime]::UtcNow.ToString("o")
Set-Content -Path $launcherStatusPath -Value (@{
    schema           = "autoplay_launcher_status_v1"
    outcome          = if ($driverExit -eq 0) { "ok" } else { "driver_failed" }
    port             = $Port
    artifact_dir     = $ArtifactDir
    started_at       = $startedAt
    finished_at      = $finishedAt
    driver_exit_code = $driverExit
    client_exit_code = $client.ExitCode
    log_path         = "process.log"
} | ConvertTo-Json -Depth 4)

Write-Host "[autoplay-smoke] artifacts under $ArtifactDir"
exit $driverExit
