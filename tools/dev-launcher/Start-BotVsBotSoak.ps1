# Start-BotVsBotSoak.ps1 -- PROMPT 1603 — bot-vs-bot soak QA launcher.
#
# What it does:
#   1. Resolves the play/build repo root via the same fallback chain as
#      Start-TwoClients.ps1 (-PlayRepoRoot, $env:CCGS_PLAY_REPO_ROOT,
#      $env:CCGS_CANONICAL_MAIN_ROOT, the documented default
#      `D:\_DEV\ccgs-play-main`, then the launcher root).
#   2. Applies the shared Windows/MSVC Cargo resource policy.
#   3. Picks a server port (default 5000, override with -Port). Auto-bumps to
#      the next free port unless -StrictPort is passed.
#   4. Builds `server.exe` if missing under the resolved CARGO_TARGET_DIR.
#   5. Creates a timestamped evidence directory under
#      production/qa/evidence/dev-runs/<UTC-YYYY-MM-DD-HHMMSS>-bot-vs-bot-soak/
#      with the layout documented in
#      `reports/PROMPT-1594-bot-flow-inventory-followup.md` (server.log,
#      server.err, bot-decision-log.jsonl, server-snapshots/).
#   6. Starts `server.exe` with SERVER_PORT set and the bot-flow QA env vars
#      (`CCGS_BOT_DECISION_LOG_PATH`, `CCGS_QA_SNAPSHOT_DIR`) pointed at the
#      evidence subdirectories so any future server-side dump code can drop
#      its artefacts in the canonical place.
#   7. Sleeps for -DurationSeconds (default 300 = 5 min), then stops the
#      server cleanly with Stop-Process and saves a soak-summary.json.
#
# What it does NOT do:
#   - It does not spawn a client. Bot-vs-bot is headless; the
#     `Create 2-Bot Soak Room` button (PROMPT 1603) is the manual entry
#     point for an operator-driven variant of this flow, not the soak path.
#   - It does not implement bot placement logic. PROMPT 1602 owns that;
#     this launcher only owns the wall-clock timer + evidence layout.
#   - It does not fetch, pull, merge, push, or modify git in any way.
#   - It does not run tests, story-done, smoke, or any QA workflow.
#   - It does not edit production/ or session-state files; the evidence
#     directory it creates is the only write surface.
#
# Usage from PowerShell:
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 -DurationSeconds 60
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 -Port 5050 -Release
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 -Help
#
# One-click: double-click `start-bot-vs-bot-soak.bat` at the repo root.

[CmdletBinding()]
param(
    [int]$Port = 5000,
    [switch]$StrictPort,
    [switch]$Release,
    [int]$ServerWaitSeconds = 8,
    [int]$DurationSeconds = 300,
    # PROMPT 1640: opt-in round-count bound. 0 = disabled (default).
    # When set to N > 0 the server exits cleanly after N completed rounds via
    # CCGS_BOT_MAX_ROUNDS env var; GameOverReason::MaxRoundsReached is emitted.
    [int]$MaxRounds = 0,
    [switch]$DryRun,
    [switch]$Help,
    [string]$PlayRepoRoot = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Section { param([string]$Title) Write-Host ""; Write-Host "==== $Title ====" -ForegroundColor Cyan }

function Show-Help {
    @"
Start-BotVsBotSoak.ps1 -- launch a headless bot-vs-bot soak run.

PARAMETERS
  -Port N                Server bind port (default 5000). Auto-bumps if busy
                         unless -StrictPort is passed.
  -StrictPort            Do NOT auto-bump; fail if -Port is busy.
  -Release               Use the release-profile server binary.
  -ServerWaitSeconds N   How long to wait for the server bind line (default 8).
  -DurationSeconds N     Wall-clock soak duration in seconds (default 300 = 5 min).
                         At expiry the launcher Stop-Process's the server.
  -MaxRounds N           Opt-in round-count bound (default 0 = disabled).
                         Sets CCGS_BOT_MAX_ROUNDS=N so the server triggers
                         GameOver (MaxRoundsReached) after N completed rounds.
                         Normal multiplayer sessions are not affected.
  -DryRun                Print every step but do not start any process.
  -PlayRepoRoot P        Absolute path to the dedicated play/build checkout.
                         Falls back to `$env:CCGS_PLAY_REPO_ROOT,
                         `$env:CCGS_CANONICAL_MAIN_ROOT, then
                         'D:\_DEV\ccgs-play-main', then the launcher's own
                         parent-of-parent if the dedicated path does not exist.
  -Help                  Show this help and exit.

OUTPUT (per run, under production/qa/evidence/dev-runs/<UTC>-bot-vs-bot-soak/)
  server.log
  server.err
  bot-decision-log.jsonl          # populated by future server-side dump code
  server-snapshots/               # populated by future server-side dump code
  soak-summary.json               # what the launcher itself recorded

EXIT CODES
  0 success (soak ran to completion),
  1 generic failure (build, port, file I/O),
  2 port unavailable with -StrictPort,
  3 server failed to bind within -ServerWaitSeconds.
"@ | Write-Host
}

if ($Help) { Show-Help; exit 0 }

# ---- 1. Resolve launcher root + play root --------------------------------
$ScriptDir    = Split-Path -Parent $MyInvocation.MyCommand.Path
$ToolsDir     = Split-Path -Parent $ScriptDir
$LauncherRoot = Split-Path -Parent $ToolsDir

$DefaultPlayRoot = 'D:\_DEV\ccgs-play-main'
$PlayRoot        = ''
$PlayRootSource  = ''
if ($PSBoundParameters.ContainsKey('PlayRepoRoot') -and $PlayRepoRoot.Trim().Length -gt 0) {
    $PlayRoot       = $PlayRepoRoot.Trim()
    $PlayRootSource = '-PlayRepoRoot argument'
} elseif ($env:CCGS_PLAY_REPO_ROOT) {
    $PlayRoot       = $env:CCGS_PLAY_REPO_ROOT.Trim()
    $PlayRootSource = '$env:CCGS_PLAY_REPO_ROOT'
} elseif ($env:CCGS_CANONICAL_MAIN_ROOT) {
    $PlayRoot       = $env:CCGS_CANONICAL_MAIN_ROOT.Trim()
    $PlayRootSource = '$env:CCGS_CANONICAL_MAIN_ROOT (alias)'
} elseif (Test-Path $DefaultPlayRoot) {
    $PlayRoot       = $DefaultPlayRoot
    $PlayRootSource = 'documented dedicated default'
} else {
    $PlayRoot       = $LauncherRoot
    $PlayRootSource = 'launcher root (no dedicated checkout configured)'
    Write-Warning "No dedicated play/build checkout configured; falling back to the launcher root. Run Update-LatestMain.ps1 (or set CCGS_PLAY_REPO_ROOT) to create the dedicated checkout."
}

$RepoRoot = $PlayRoot
Set-Location $RepoRoot

Write-Section "Roots"
Write-Host "Launcher repo root: $LauncherRoot"
Write-Host "Play/build root:    $RepoRoot"
Write-Host "Play/build source:  $PlayRootSource"
if (-not (Test-Path (Join-Path $RepoRoot 'Cargo.toml'))) {
    Write-Host -ForegroundColor Red "No Cargo.toml at $RepoRoot -- this does not look like the CCGS workspace."
    exit 1
}

# ---- 2. Cargo policy -----------------------------------------------------
Write-Section "Cargo resource policy"
$env:CARGO_TARGET_DIR        = 'D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL       = '0'
$env:RUSTFLAGS               = '-C debuginfo=0 -C link-arg=/DEBUG:NONE'
Write-Host "CARGO_TARGET_DIR = $env:CARGO_TARGET_DIR"

# ---- 3. Port selection ---------------------------------------------------
function Test-PortFree {
    param([int]$P)
    $listener = $null
    try {
        $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $P)
        $listener.Start()
        $true
    } catch { $false }
    finally { if ($null -ne $listener) { try { $listener.Stop() } catch {} } }
}

Write-Section "Port selection"
$chosen = $Port
if (-not (Test-PortFree $chosen)) {
    if ($StrictPort) {
        Write-Host -ForegroundColor Red "Port $chosen is in use and -StrictPort was passed."
        exit 2
    }
    Write-Warning "Port $chosen is in use. Searching for the next free port..."
    $found = $false
    for ($p = $chosen + 1; $p -le $chosen + 50; $p++) {
        if (Test-PortFree $p) { $chosen = $p; $found = $true; break }
    }
    if (-not $found) {
        Write-Host -ForegroundColor Red "No free port found between $($Port+1) and $($Port+50)."
        exit 1
    }
}
Write-Host "Chosen port: $chosen"

# ---- 4. Build server if missing -----------------------------------------
Write-Section "Binary check"
$profileDir = if ($Release) { 'release' } else { 'debug' }
$serverBin  = Join-Path $env:CARGO_TARGET_DIR "$profileDir\server.exe"

if (-not (Test-Path $serverBin)) {
    Write-Host "Missing server binary -- running cargo build."
    $base = @('build', '-p', 'server')
    if ($Release) { $base += '--release' }
    Write-Host "cargo $($base -join ' ')"
    if (-not $DryRun) {
        & cargo @base
        if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "cargo build server failed."; exit 1 }
    }
}
Write-Host "Server binary: $serverBin (exists=$(Test-Path $serverBin))"

# ---- 5. Evidence dir -----------------------------------------------------
Write-Section "Evidence dir"
$stamp = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd-HHmmss')
$evidenceDir   = Join-Path $RepoRoot "production/qa/evidence/dev-runs/$stamp-bot-vs-bot-soak"
$snapshotsDir  = Join-Path $evidenceDir 'server-snapshots'
if (-not $DryRun) {
    New-Item -ItemType Directory -Force -Path $evidenceDir  | Out-Null
    New-Item -ItemType Directory -Force -Path $snapshotsDir | Out-Null
}
Write-Host "Evidence dir:   $evidenceDir"
Write-Host "Snapshots dir:  $snapshotsDir"

$serverLog          = Join-Path $evidenceDir 'server.log'
$serverErr          = Join-Path $evidenceDir 'server.err'
$botDecisionLogPath = Join-Path $evidenceDir 'bot-decision-log.jsonl'

# ---- 6. Start server -----------------------------------------------------
Write-Section "Start server"
$env:SERVER_PORT = "$chosen"
$serverUrl       = "ws://127.0.0.1:$chosen"
$env:SERVER_URL  = $serverUrl
# These env vars are the canonical contract the server-side QA snapshot work
# (PROMPT-1594 follow-up items 2-3) will read. Set them now so the evidence
# layout is stable and forward-compatible: today the server may ignore them,
# tomorrow it will write into bot-decision-log.jsonl / server-snapshots/ on
# its own.
$env:CCGS_BOT_DECISION_LOG_PATH = $botDecisionLogPath
$env:CCGS_QA_SNAPSHOT_DIR       = $snapshotsDir
if ($MaxRounds -gt 0) {
    $env:CCGS_BOT_MAX_ROUNDS = "$MaxRounds"
} else {
    Remove-Item Env:CCGS_BOT_MAX_ROUNDS -ErrorAction SilentlyContinue
}
Write-Host "SERVER_PORT                  = $env:SERVER_PORT"
Write-Host "SERVER_URL                   = $serverUrl"
Write-Host "CCGS_BOT_DECISION_LOG_PATH   = $env:CCGS_BOT_DECISION_LOG_PATH"
Write-Host "CCGS_QA_SNAPSHOT_DIR         = $env:CCGS_QA_SNAPSHOT_DIR"
Write-Host "CCGS_BOT_MAX_ROUNDS          = $(if ($MaxRounds -gt 0) { $MaxRounds } else { '(disabled)' })"

$serverProc = $null
if (-not $DryRun) {
    $serverProc = Start-Process `
        -FilePath $serverBin `
        -WorkingDirectory $RepoRoot `
        -RedirectStandardOutput $serverLog `
        -RedirectStandardError  $serverErr `
        -WindowStyle Hidden `
        -PassThru
    Write-Host "Server PID: $($serverProc.Id) -- logs at $serverLog"

    $deadline = (Get-Date).AddSeconds($ServerWaitSeconds)
    $bound = $false
    while ((Get-Date) -lt $deadline) {
        if (-not (Test-PortFree $chosen)) { $bound = $true; break }
        Start-Sleep -Milliseconds 250
    }
    if (-not $bound) {
        Write-Host -ForegroundColor Red "Server did not bind port $chosen within $ServerWaitSeconds s. See $serverLog / $serverErr."
        try { Stop-Process -Id $serverProc.Id -Force -ErrorAction Stop } catch {}
        exit 3
    }
    Write-Host "Server bound port $chosen."
}

# ---- 7. Soak wall-clock timer -------------------------------------------
Write-Section "Soak"
$soakStartUtc = (Get-Date).ToUniversalTime()
Write-Host "Soak start (UTC): $($soakStartUtc.ToString('o'))"
Write-Host "Soak duration:    ${DurationSeconds}s"
if (-not $DryRun) {
    # NOTE: PROMPT 1602 owns the bot-vs-bot driver wiring. Until it lands, a
    # soak of "two bots in one server, no client" cannot yet self-bootstrap;
    # this launcher sleeps the configured window to provide the canonical
    # wall-clock harness and evidence layout, so future PROMPTs only need
    # to plug in the server-side driver.
    Start-Sleep -Seconds $DurationSeconds
}
$soakEndUtc = (Get-Date).ToUniversalTime()

# ---- 8. Stop server cleanly ---------------------------------------------
Write-Section "Stop server"
if ($null -ne $serverProc) {
    try {
        Stop-Process -Id $serverProc.Id -Force -ErrorAction Stop
        Write-Host "Server PID $($serverProc.Id) stopped."
    } catch {
        Write-Warning "Stop-Process failed for PID $($serverProc.Id): $($_.Exception.Message)"
    }
}

# ---- 9. Summary ---------------------------------------------------------
Write-Section "Summary"
$summary = [ordered]@{
    started_utc                 = $soakStartUtc.ToString('o')
    stopped_utc                 = $soakEndUtc.ToString('o')
    duration_seconds            = $DurationSeconds
    repo_root                   = $RepoRoot
    evidence_dir                = $evidenceDir
    server_snapshots_dir        = $snapshotsDir
    bot_decision_log_path       = $botDecisionLogPath
    server_port                 = $chosen
    server_url                  = $serverUrl
    server_pid                  = if ($null -ne $serverProc) { $serverProc.Id } else { $null }
    server_log                  = $serverLog
    server_err                  = $serverErr
    server_bin                  = $serverBin
    profile                     = $profileDir
    ccgs_bot_decision_log_path  = $env:CCGS_BOT_DECISION_LOG_PATH
    ccgs_qa_snapshot_dir        = $env:CCGS_QA_SNAPSHOT_DIR
    ccgs_bot_max_rounds         = if ($MaxRounds -gt 0) { $MaxRounds } else { $null }
    dry_run                     = [bool]$DryRun
    notes                       = "PROMPT 1603 launcher; max-rounds bound via PROMPT 1640 (CCGS_BOT_MAX_ROUNDS)."
}
$summaryPath = Join-Path $evidenceDir 'soak-summary.json'
if (-not $DryRun) {
    $summary | ConvertTo-Json -Depth 4 | Set-Content -Path $summaryPath -Encoding utf8
}
Write-Host "Summary written to: $summaryPath"
exit 0
