# Start-AutoplayVsBot.ps1 -- PROMPT 1644 — autoplay-vs-bot composite harness v1.
#
# What it does:
#   1. Resolves the play/build repo root via the same fallback chain as
#      Start-BotVsBotSoak.ps1 (-PlayRepoRoot, $env:CCGS_PLAY_REPO_ROOT,
#      $env:CCGS_CANONICAL_MAIN_ROOT, the documented default
#      `D:\_DEV\ccgs-play-main`, then the launcher root).
#   2. Checks that an interactive desktop session exists; emits
#      BLOCKED-HUMAN-GUI and exits 10 if running non-interactively, because
#      the Bevy client requires a visible window.
#   3. Validates that the bot server preconditions are met:
#        a) Either a soak server is already running on the configured port
#           (when -SkipSoakLaunch is passed), OR
#        b) Start-BotVsBotSoak.ps1 is present so it can be launched in a
#           background job.
#      Emits BLOCKED-PRECONDITION and exits 11 if neither condition is met.
#   4. If -DryRun: prints every step without launching any process.
#   5. Otherwise:
#        a) If -SkipSoakLaunch is NOT set: starts Start-BotVsBotSoak.ps1
#           as a background PowerShell job and waits up to -SoakReadySecs
#           for the server port to bind.
#        b) Sets CCGS_AUTOPLAY_BOT_ROOM_READY=1 and all canonical autoplay
#           env vars, then delegates to Run-AutoplaySmoke.ps1 -Recipe full-game
#           (or the recipe supplied by -Recipe).
#   6. Creates a timestamped evidence directory under
#      production/qa/evidence/composite-runs/<UTC-YYYY-MM-DD-HHMMSS>-autoplay-vs-bot/
#      and writes a composite-summary.json with the outcome.
#   7. Stops the background soak job (if this launcher started it) on exit.
#
# What it does NOT do:
#   - It does not build Rust; Run-AutoplaySmoke.ps1 owns the cargo build step.
#   - It does not run tests, story-done, smoke, or any QA workflow gate.
#   - It does not claim live PASS for AUTOPLAY-VS-BOT-QA-001; that gate
#     remains HUMAN-GATE until an operator runs and signs off the full-game run.
#   - It does not fetch, pull, merge, push, or modify git in any way.
#   - It does not edit production/session-state or sprint files.
#
# Usage from the repo root:
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -DryRun
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -SkipSoakLaunch -Port 5000
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -Recipe full-game -SoakDurationSeconds 600
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -Help
#
# Exit codes:
#   0   success (Run-AutoplaySmoke.ps1 exited 0)
#   1   generic failure (missing files, I/O error, smoke launcher failed)
#   4   driver emitted local.block (BLOCKED — upstream recipe guard fired)
#  10   BLOCKED-HUMAN-GUI (non-interactive session; Bevy window cannot open)
#  11   BLOCKED-PRECONDITION (soak server not running and Start-BotVsBotSoak.ps1 missing)
#  12   BLOCKED-PRECONDITION (soak server did not bind within -SoakReadySecs)

[CmdletBinding()]
param(
    [int]$Port           = 5000,
    [switch]$StrictPort,
    [switch]$Release,
    [int]$RpcPort        = 15873,
    [string]$Recipe      = 'full-game',
    [switch]$SkipSoakLaunch,
    [int]$SoakReadySecs  = 20,
    [int]$SoakDurationSeconds = 300,
    [int]$ClientStartupSecs = 60,
    [string]$Python      = 'python',
    [switch]$DryRun,
    [switch]$Help,
    [string]$PlayRepoRoot = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Section { param([string]$Title) Write-Host ""; Write-Host "==== $Title ====" -ForegroundColor Cyan }

function Show-Help {
    @"
Start-AutoplayVsBot.ps1 -- composite autoplay-vs-bot QA launcher (PROMPT 1644).

Coordinates a headless bot server (Start-BotVsBotSoak.ps1) with a recipe-driven
autoplay client (Run-AutoplaySmoke.ps1) and writes a composite evidence summary.

PARAMETERS
  -Port N                Bot server bind port (default 5000).
  -StrictPort            Do NOT auto-bump port; fail if -Port is busy.
  -Release               Use release-profile server binary.
  -RpcPort N             Autoplay RPC port for the client (default 15873).
  -Recipe NAME           Recipe passed to Run-AutoplaySmoke.ps1 (default: full-game).
  -SkipSoakLaunch        Do NOT launch Start-BotVsBotSoak.ps1; assume the server
                         is already running on -Port.
  -SoakReadySecs N       Seconds to wait for the soak server to bind (default 20).
  -SoakDurationSeconds N Wall-clock soak window forwarded to the soak launcher
                         (default 300). Ignored when -SkipSoakLaunch is set.
  -ClientStartupSecs N   Max seconds to wait for the client RPC port (default 60).
  -Python EXE            Python executable (default: python).
  -DryRun                Print every step without launching any process.
  -PlayRepoRoot P        Absolute path to the dedicated play/build checkout.
                         Falls back to `$env:CCGS_PLAY_REPO_ROOT,
                         `$env:CCGS_CANONICAL_MAIN_ROOT, 'D:\_DEV\ccgs-play-main',
                         then the launcher root.
  -Help                  Show this help and exit.

BLOCKED EXIT CODES
  10   BLOCKED-HUMAN-GUI       -- non-interactive; Bevy needs a visible desktop.
  11   BLOCKED-PRECONDITION    -- soak server absent and Start-BotVsBotSoak.ps1 missing.
  12   BLOCKED-PRECONDITION    -- soak server did not bind within -SoakReadySecs.

OUTPUT (per run)
  production/qa/evidence/composite-runs/<UTC>-autoplay-vs-bot/
      composite-summary.json    # top-level outcome + child exit codes
      autoplay-run/             # symlinked or path-ref to autoplay artifact dir
"@ | Write-Host
}

if ($Help) { Show-Help; exit 0 }

# ---- helpers ----------------------------------------------------------------
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

function Test-PortListening {
    param([int]$P)
    try {
        $tcp = [System.Net.Sockets.TcpClient]::new()
        $tcp.Connect('127.0.0.1', $P)
        $tcp.Close()
        $true
    } catch { $false }
}

# ---- 1. Resolve roots -------------------------------------------------------
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
    Write-Warning "No dedicated play/build checkout found; using launcher root. Set CCGS_PLAY_REPO_ROOT or run Update-LatestMain.ps1."
}

$RepoRoot = $PlayRoot
Set-Location $RepoRoot

Write-Section "Roots"
Write-Host "Launcher repo root: $LauncherRoot"
Write-Host "Play/build root:    $RepoRoot  (source: $PlayRootSource)"
if (-not (Test-Path (Join-Path $RepoRoot 'Cargo.toml'))) {
    Write-Host -ForegroundColor Red "BLOCKED-PRECONDITION: No Cargo.toml at $RepoRoot -- does not look like the CCGS workspace."
    exit 11
}

# ---- 2. Interactive-session guard -------------------------------------------
Write-Section "Desktop session check"
$isInteractive = [Environment]::UserInteractive
Write-Host "UserInteractive: $isInteractive"
if (-not $isInteractive -and -not $DryRun) {
    Write-Host -ForegroundColor Red "BLOCKED-HUMAN-GUI: The Bevy client requires a visible desktop. Run this launcher from an interactive PowerShell terminal, not from a CI/CD headless agent or scheduled task."
    exit 10
}

# ---- 3. Locate child launchers ----------------------------------------------
Write-Section "Child launcher check"
$soakScript   = Join-Path $LauncherRoot 'tools\dev-launcher\Start-BotVsBotSoak.ps1'
$smokeScript  = Join-Path $LauncherRoot 'tools\autoplay\Run-AutoplaySmoke.ps1'

$soakPresent  = Test-Path $soakScript
$smokePresent = Test-Path $smokeScript
Write-Host "Start-BotVsBotSoak.ps1 : $(if ($soakPresent)  { 'found' } else { 'MISSING' }) ($soakScript)"
Write-Host "Run-AutoplaySmoke.ps1  : $(if ($smokePresent) { 'found' } else { 'MISSING' }) ($smokeScript)"

if (-not $smokePresent) {
    Write-Host -ForegroundColor Red "BLOCKED-PRECONDITION: Run-AutoplaySmoke.ps1 not found at $smokeScript."
    exit 11
}
if (-not $SkipSoakLaunch -and -not $soakPresent) {
    Write-Host -ForegroundColor Red "BLOCKED-PRECONDITION: -SkipSoakLaunch was not set and Start-BotVsBotSoak.ps1 is missing at $soakScript."
    exit 11
}

# ---- 4. Evidence directory --------------------------------------------------
Write-Section "Evidence dir"
$stamp       = [DateTime]::UtcNow.ToString('yyyy-MM-dd-HHmmss')
$evidenceDir = Join-Path $RepoRoot "production/qa/evidence/composite-runs/$stamp-autoplay-vs-bot"
if (-not $DryRun) {
    New-Item -ItemType Directory -Force -Path $evidenceDir | Out-Null
}
Write-Host "Evidence dir: $evidenceDir"

$compositeSummaryPath = Join-Path $evidenceDir 'composite-summary.json'
$autoplayRunRef       = Join-Path $evidenceDir 'autoplay-run-path.txt'

# ---- 5. Port selection for soak server -------------------------------------
Write-Section "Soak server port"
$chosenPort = $Port
if ($SkipSoakLaunch) {
    Write-Host "SkipSoakLaunch=true -- assuming server already listening on port $chosenPort."
    if (-not (Test-PortListening $chosenPort) -and -not $DryRun) {
        Write-Host -ForegroundColor Yellow "Warning: port $chosenPort does not appear to be listening. Proceeding anyway (server may still be starting)."
    }
} else {
    # Auto-bump if busy (and not strict)
    if (-not (Test-PortFree $chosenPort)) {
        if ($StrictPort) {
            Write-Host -ForegroundColor Red "Port $chosenPort is in use and -StrictPort was passed."
            exit 1
        }
        Write-Warning "Port $chosenPort is in use. Searching for a free port..."
        $found = $false
        for ($p = $chosenPort + 1; $p -le $chosenPort + 50; $p++) {
            if (Test-PortFree $p) { $chosenPort = $p; $found = $true; break }
        }
        if (-not $found) {
            Write-Host -ForegroundColor Red "No free port found between $($Port+1) and $($Port+50)."
            exit 1
        }
    }
    Write-Host "Chosen soak server port: $chosenPort"
}

# ---- 6. Launch soak server (background job) ---------------------------------
$soakJob = $null
if (-not $SkipSoakLaunch -and -not $DryRun) {
    Write-Section "Starting soak server (background job)"
    $soakArgs = @(
        '-ExecutionPolicy', 'Bypass',
        '-File', $soakScript,
        '-Port', $chosenPort,
        '-DurationSeconds', $SoakDurationSeconds
    )
    if ($Release) { $soakArgs += '-Release' }
    Write-Host "pwsh $($soakArgs -join ' ')"
    $soakJob = Start-Job -ScriptBlock {
        param($pwshArgs)
        & powershell.exe @pwshArgs
    } -ArgumentList (, $soakArgs)
    Write-Host "Soak job ID: $($soakJob.Id)"

    Write-Host "Waiting up to ${SoakReadySecs}s for soak server to bind port $chosenPort..."
    $deadline = (Get-Date).AddSeconds($SoakReadySecs)
    $bound = $false
    while ((Get-Date) -lt $deadline) {
        if (Test-PortListening $chosenPort) { $bound = $true; break }
        Start-Sleep -Milliseconds 500
    }
    if (-not $bound) {
        Write-Host -ForegroundColor Red "BLOCKED-PRECONDITION: Soak server did not bind port $chosenPort within ${SoakReadySecs}s."
        Stop-Job  $soakJob -ErrorAction SilentlyContinue
        Remove-Job $soakJob -ErrorAction SilentlyContinue
        exit 12
    }
    Write-Host "Soak server bound on port $chosenPort."
} elseif (-not $SkipSoakLaunch -and $DryRun) {
    Write-Section "Starting soak server (DRY RUN -- skipped)"
    Write-Host "[DRY RUN] would launch: powershell -ExecutionPolicy Bypass -File $soakScript -Port $chosenPort -DurationSeconds $SoakDurationSeconds"
}

# ---- 7. Run autoplay smoke launcher -----------------------------------------
Write-Section "Autoplay smoke (recipe=$Recipe)"

$autoplayStamp      = [DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss') + '-Z'
$autoplayArtifactDir = Join-Path $RepoRoot "production/qa/evidence/autoplay-runs/$autoplayStamp"

$env:CCGS_AUTOPLAY_BOT_ROOM_READY = '1'
$env:SERVER_PORT                  = "$chosenPort"
$env:SERVER_URL                   = "ws://127.0.0.1:$chosenPort"
Write-Host "CCGS_AUTOPLAY_BOT_ROOM_READY = 1"
Write-Host "SERVER_PORT                  = $chosenPort"
Write-Host "SERVER_URL                   = ws://127.0.0.1:$chosenPort"
Write-Host "Autoplay artifact dir:       $autoplayArtifactDir"

if (-not $DryRun) {
    Set-Content -Path $autoplayRunRef -Value $autoplayArtifactDir -Encoding utf8
}

$smokeExit = 0
if (-not $DryRun) {
    $smokeArgs = @(
        '-ExecutionPolicy', 'Bypass',
        '-File', $smokeScript,
        '-Port', $RpcPort,
        '-Recipe', $Recipe,
        '-ArtifactDir', $autoplayArtifactDir,
        '-Python', $Python,
        '-ClientStartupSecs', $ClientStartupSecs
    )
    Write-Host "powershell $($smokeArgs -join ' ')"
    $smokeProc = Start-Process -FilePath 'powershell.exe' `
        -ArgumentList $smokeArgs `
        -NoNewWindow -PassThru -Wait
    $smokeExit = $smokeProc.ExitCode
    Write-Host "Run-AutoplaySmoke.ps1 exited: $smokeExit"
} else {
    Write-Section "Autoplay smoke (DRY RUN -- skipped)"
    Write-Host "[DRY RUN] would launch: powershell -ExecutionPolicy Bypass -File $smokeScript -Port $RpcPort -Recipe $Recipe -ArtifactDir $autoplayArtifactDir"
}

# ---- 8. Stop soak job -------------------------------------------------------
if ($null -ne $soakJob) {
    Write-Section "Stop soak server job"
    Stop-Job   $soakJob -ErrorAction SilentlyContinue
    Remove-Job $soakJob -ErrorAction SilentlyContinue
    Write-Host "Soak job stopped."
}

# ---- 9. Composite summary ---------------------------------------------------
Write-Section "Composite summary"

$outcome = switch ($smokeExit) {
    0  { 'ok' }
    4  { 'blocked-recipe-guard' }
    10 { 'blocked-human-gui' }
    11 { 'blocked-precondition' }
    12 { 'blocked-soak-timeout' }
    default { "smoke_failed_exit_$smokeExit" }
}

$summary = [ordered]@{
    schema               = 'autoplay_vs_bot_composite_summary_v1'
    prompt               = 'PROMPT-1644'
    outcome              = $outcome
    recipe               = $Recipe
    soak_port            = $chosenPort
    rpc_port             = $RpcPort
    skip_soak_launch     = [bool]$SkipSoakLaunch
    soak_duration_secs   = $SoakDurationSeconds
    smoke_exit_code      = $smokeExit
    autoplay_artifact_dir = $autoplayArtifactDir
    evidence_dir         = $evidenceDir
    dry_run              = [bool]$DryRun
    generated_utc        = [DateTime]::UtcNow.ToString('o')
    live_pass_status     = 'NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator sign-off for live PASS evidence'
    notes                = 'PROMPT 1644 composite harness v1; live GUI PASS gate remains HUMAN-GATE per GAP-01/GAP-02.'
}

if (-not $DryRun) {
    $summary | ConvertTo-Json -Depth 4 | Set-Content -Path $compositeSummaryPath -Encoding utf8
}
Write-Host "Composite summary: $compositeSummaryPath"

# Human-readable outcome
if ($smokeExit -eq 0) {
    Write-Host -ForegroundColor Green "Composite run COMPLETE (recipe=$Recipe exit=0)."
    Write-Host -ForegroundColor Yellow "NOTE: This is NOT a live PASS for AUTOPLAY-VS-BOT-QA-001. An operator must review artifacts and sign off."
} elseif ($smokeExit -eq 4) {
    Write-Host -ForegroundColor Yellow "Composite run BLOCKED-RECIPE-GUARD (driver exit 4 -- local.block fired in recipe)."
} elseif ($smokeExit -in 10, 11, 12) {
    Write-Host -ForegroundColor Red "Composite run BLOCKED (exit=$smokeExit). See BLOCKED-* message above."
} else {
    Write-Host -ForegroundColor Red "Composite run FAILED (smoke exit=$smokeExit). Review $autoplayArtifactDir\launcher-status.json."
}

exit $smokeExit
