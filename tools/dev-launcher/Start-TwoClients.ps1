# Start-TwoClients.ps1 -- Button 2 of the dev two-button launcher.
#
# What it does:
#   1. Resolves repo root (parent-of-parent of this script).
#   2. Applies the documented Windows/MSVC Cargo resource policy.
#   3. Picks a server port (default 5000, override with -Port). If the port is
#      busy, auto-bumps to the next free port unless -StrictPort is passed.
#   4. Builds server + client if the binaries are missing under the resolved
#      CARGO_TARGET_DIR (cheap no-op when Update-LatestMain.ps1 just ran).
#   5. Creates a timestamped evidence directory under
#      production/qa/evidence/dev-runs/<UTC-YYYY-MM-DD-HHMMSS>/.
#   6. Starts `server.exe` with SERVER_PORT set, redirecting stdout/stderr to
#      server.log in the evidence dir. Waits up to 8 s for the bind line.
#   7. Starts two `client.exe` processes with SERVER_URL=ws://127.0.0.1:<port>,
#      redirecting stdout/stderr to client_a.log / client_b.log.
#   8. Prints PIDs, log paths, and the server URL. Saves a launch-summary.json.
#
# What it does NOT do:
#   - It does not fetch, pull, merge, push, or modify git in any way.
#   - It does not run tests, story-done, smoke, or any QA workflow.
#   - It does not edit production/ or session-state files.
#   - It does not wait for the clients to exit -- they remain running for
#     manual two-client testing. Close them with the X button or via Stop-Process.
#
# Usage from PowerShell:
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1 -Port 5050
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1 -StrictPort
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1 -Help
#
# One-click: double-click `start-two-clients.bat` at the repo root.

[CmdletBinding()]
param(
    [int]$Port = 5000,
    [switch]$StrictPort,
    [switch]$Release,
    [int]$ServerWaitSeconds = 8,
    [switch]$DryRun,
    [switch]$Help,
    [string]$PlayRepoRoot = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Section { param([string]$Title) Write-Host ""; Write-Host "==== $Title ====" -ForegroundColor Cyan }

function Show-Help {
    @"
Start-TwoClients.ps1 -- launch one server + two native clients for manual testing.

PARAMETERS
  -Port N                Server bind port (default 5000). Auto-bumps if busy
                         unless -StrictPort is passed.
  -StrictPort            Do NOT auto-bump; fail if -Port is busy.
  -Release               Use the release-profile binaries.
  -ServerWaitSeconds N   How long to wait for the server bind line (default 8).
  -DryRun                Print every step but do not start any process.
  -PlayRepoRoot P        Absolute path to the dedicated play/build checkout
                         (the directory under which the build runs and the
                         evidence dir is created). Falls back to
                         `$env:CCGS_PLAY_REPO_ROOT, `$env:CCGS_CANONICAL_MAIN_ROOT,
                         then 'D:\_DEV\ccgs-play-main', then the script's own
                         parent-of-parent if the dedicated path does not exist.
  -Help                  Show this help and exit.

OUTPUT (per run)
  production/qa/evidence/dev-runs/<UTC-stamp>/
    server.log
    client_a.log
    client_b.log
    launch-summary.json

EXIT CODES
  0 success (processes are running),
  1 generic failure,
  2 port unavailable with -StrictPort.
"@ | Write-Host
}

if ($Help) { Show-Help; exit 0 }

# ---- 1. Resolve launcher root + play root --------------------------------
$ScriptDir    = Split-Path -Parent $MyInvocation.MyCommand.Path
$ToolsDir     = Split-Path -Parent $ScriptDir
$LauncherRoot = Split-Path -Parent $ToolsDir

# PROMPT 1402: load the build-provenance helper module so we can write a
# build.json beside server.log/client_a.log/client_b.log. Failing to import
# the helper must never block a launch -- we degrade to no-build.json.
$BuildProvenanceModule = Join-Path $ScriptDir 'BuildProvenance.psm1'
$BuildProvenanceLoaded = $false
try {
    Import-Module $BuildProvenanceModule -Force -ErrorAction Stop
    $BuildProvenanceLoaded = $true
} catch {
    Write-Warning "BuildProvenance helper failed to import ($BuildProvenanceModule): $($_.Exception.Message). Continuing without build.json."
}

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
    # Last-resort fallback to the launcher root so a tester who never ran
    # Update-LatestMain.ps1 still gets a usable Start session. This is the
    # only path that can step on the orchestrator checkout, so we surface
    # it loudly.
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

# ---- 4. Build if missing -------------------------------------------------
Write-Section "Binary check"
$profileDir = if ($Release) { 'release' } else { 'debug' }
$serverBin  = Join-Path $env:CARGO_TARGET_DIR "$profileDir\server.exe"
$clientBin  = Join-Path $env:CARGO_TARGET_DIR "$profileDir\client.exe"

$needServer = -not (Test-Path $serverBin)
$needClient = -not (Test-Path $clientBin)
if ($needServer -or $needClient) {
    Write-Host "Missing binaries detected -- running cargo build."
    $base = @('build')
    if ($Release) { $base += '--release' }
    if ($needServer) {
        $cargoArgs = $base + @('-p','server')
        Write-Host "cargo $($cargoArgs -join ' ')"
        if (-not $DryRun) {
            & cargo @cargoArgs
            if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "cargo build server failed."; exit 1 }
        }
    }
    if ($needClient) {
        $cargoArgs = $base + @('-p','client','--bin','client')
        Write-Host "cargo $($cargoArgs -join ' ')"
        if (-not $DryRun) {
            & cargo @cargoArgs
            if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "cargo build client failed."; exit 1 }
        }
    }
}
Write-Host "Server binary: $serverBin (exists=$(Test-Path $serverBin))"
Write-Host "Client binary: $clientBin (exists=$(Test-Path $clientBin))"

# ---- 5. Evidence dir -----------------------------------------------------
Write-Section "Evidence dir"
$stamp = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd-HHmmss')
$evidenceDir = Join-Path $RepoRoot "production/qa/evidence/dev-runs/$stamp"
if (-not $DryRun) {
    New-Item -ItemType Directory -Force -Path $evidenceDir | Out-Null
}
Write-Host "Evidence dir: $evidenceDir"

$serverLog  = Join-Path $evidenceDir 'server.log'
$clientALog = Join-Path $evidenceDir 'client_a.log'
$clientBLog = Join-Path $evidenceDir 'client_b.log'

# ---- 5b. Build provenance (PROMPT 1402) ----------------------------------
# Write `build.json` beside the log files so a captured test binary can be
# tied back to a specific origin/main commit. If the helper module did not
# load, or the evidence dir doesn't exist (DryRun), log the reason and
# continue -- never block the launch on provenance failures.
Write-Section "Build provenance"
if (-not $BuildProvenanceLoaded) {
    Write-Host "build.json: BuildProvenance helper not loaded -- skipped."
} elseif ($DryRun) {
    Write-Host "build.json: -DryRun in effect, evidence dir was not created -- skipped."
} elseif (-not (Test-Path $evidenceDir)) {
    Write-Host "build.json: evidence dir '$evidenceDir' missing -- skipped (launch continues)."
} else {
    $launcherRel = 'tools/dev-launcher/Start-TwoClients.ps1'
    $launcherRootNorm = ''
    $playRootNorm     = ''
    try {
        $launcherRootNorm = [System.IO.Path]::GetFullPath($LauncherRoot)
        $playRootNorm     = [System.IO.Path]::GetFullPath($RepoRoot)
    } catch { }
    $isLauncherRoot   = ($launcherRootNorm -ne '' -and $launcherRootNorm -ieq $playRootNorm)
    $autoSwitchedOrDedicated = (-not $isLauncherRoot)

    $gitProv      = Read-CcgsGitProvenance -Path $RepoRoot
    $serverInfo   = Read-CcgsBinaryInfo   -Path $serverBin
    $clientInfo   = Read-CcgsBinaryInfo   -Path $clientBin
    $cargoEnv     = Get-CcgsCargoEnvSnapshot
    $targetProfDir = Join-Path $env:CARGO_TARGET_DIR $profileDir
    $lastRebuild  = Read-CcgsLastBuildProvenance -TargetProfileDir $targetProfDir

    # Build commands the launcher would invoke for missing-binary recovery.
    # If neither was missing this run, an empty array is honest about it.
    $buildCmds = @()
    if ($needServer) {
        if ($Release) { $buildCmds += 'cargo build --release -p server' } else { $buildCmds += 'cargo build -p server' }
    }
    if ($needClient) {
        if ($Release) { $buildCmds += 'cargo build --release -p client --bin client' } else { $buildCmds += 'cargo build -p client --bin client' }
    }

    $payload = New-CcgsBuildProvenance `
        -Context 'launch' `
        -GeneratedAtUtc (Get-Date).ToUniversalTime() `
        -RepoRoot $RepoRoot `
        -RepoRootSource $PlayRootSource `
        -IsLauncherRoot $isLauncherRoot `
        -AutoSwitchedOrDedicated $autoSwitchedOrDedicated `
        -Git $gitProv `
        -BuildProfile $profileDir `
        -BuildCommands $buildCmds `
        -TargetDir $env:CARGO_TARGET_DIR `
        -ServerBinary $serverInfo `
        -ClientBinary $clientInfo `
        -CargoEnv $cargoEnv `
        -LauncherScript $launcherRel `
        -LastRebuild $lastRebuild

    $written = Write-CcgsBuildProvenance -EvidenceDir $evidenceDir -Payload $payload
    if ($written) {
        Write-Host "build.json written: $written"
        if ($gitProv.head_short) { Write-Host "  HEAD = $($gitProv.head_short) on '$($gitProv.branch)' (clean=$($gitProv.is_clean))" }
        if ($lastRebuild -and $lastRebuild.git -and $lastRebuild.git.head_short) {
            Write-Host "  Last rebuild = $($lastRebuild.git.head_short) on '$($lastRebuild.git.branch)' at $($lastRebuild.generated_at_utc)"
        } else {
            Write-Host "  No prior rebuild sidecar found at $targetProfDir\last-build-provenance.json"
        }
    }
}

# ---- 6. Start server -----------------------------------------------------
Write-Section "Start server"
$env:SERVER_PORT = "$chosen"
$serverUrl = "ws://127.0.0.1:$chosen"
$env:SERVER_URL  = $serverUrl
Write-Host "SERVER_PORT = $env:SERVER_PORT"
Write-Host "SERVER_URL  = $serverUrl"

$serverProc = $null
if (-not $DryRun) {
    $serverProc = Start-Process `
        -FilePath $serverBin `
        -WorkingDirectory $RepoRoot `
        -RedirectStandardOutput $serverLog `
        -RedirectStandardError  "$serverLog.err" `
        -WindowStyle Hidden `
        -PassThru
    Write-Host "Server PID: $($serverProc.Id) -- logs at $serverLog"

    # Wait until the chosen port is bound, up to ServerWaitSeconds.
    $deadline = (Get-Date).AddSeconds($ServerWaitSeconds)
    $bound = $false
    while ((Get-Date) -lt $deadline) {
        if (-not (Test-PortFree $chosen)) { $bound = $true; break }
        Start-Sleep -Milliseconds 250
    }
    if (-not $bound) {
        Write-Warning "Server did not appear to bind port $chosen within $ServerWaitSeconds s. Continuing anyway -- check $serverLog for errors."
    } else {
        Write-Host "Server bound port $chosen."
    }
}

# ---- 7. Start two clients ------------------------------------------------
Write-Section "Start clients"
$clientProcA = $null
$clientProcB = $null
if (-not $DryRun) {
    $clientProcA = Start-Process `
        -FilePath $clientBin `
        -WorkingDirectory $RepoRoot `
        -RedirectStandardOutput $clientALog `
        -RedirectStandardError  "$clientALog.err" `
        -PassThru
    Write-Host "Client A PID: $($clientProcA.Id) -- logs at $clientALog"

    Start-Sleep -Milliseconds 250

    $clientProcB = Start-Process `
        -FilePath $clientBin `
        -WorkingDirectory $RepoRoot `
        -RedirectStandardOutput $clientBLog `
        -RedirectStandardError  "$clientBLog.err" `
        -PassThru
    Write-Host "Client B PID: $($clientProcB.Id) -- logs at $clientBLog"
}

# ---- 8. Summary ----------------------------------------------------------
Write-Section "Summary"
$summary = [ordered]@{
    started_utc   = (Get-Date).ToUniversalTime().ToString('o')
    repo_root     = $RepoRoot
    evidence_dir  = $evidenceDir
    server_port   = $chosen
    server_url    = $serverUrl
    server_pid    = if ($null -ne $serverProc) { $serverProc.Id } else { $null }
    client_a_pid  = if ($null -ne $clientProcA) { $clientProcA.Id } else { $null }
    client_b_pid  = if ($null -ne $clientProcB) { $clientProcB.Id } else { $null }
    server_log    = $serverLog
    client_a_log  = $clientALog
    client_b_log  = $clientBLog
    server_bin    = $serverBin
    client_bin    = $clientBin
    profile       = $profileDir
    dry_run       = [bool]$DryRun
}
$summaryPath = Join-Path $evidenceDir 'launch-summary.json'
if (-not $DryRun) {
    $summary | ConvertTo-Json -Depth 4 | Set-Content -Path $summaryPath -Encoding utf8
}
Write-Host "Summary written to: $summaryPath"
Write-Host ""
Write-Host "Two clients are now running. Close them when done with manual testing."
Write-Host "Tail logs with:"
Write-Host "  Get-Content -Wait $serverLog"
Write-Host "  Get-Content -Wait $clientALog"
Write-Host "  Get-Content -Wait $clientBLog"
exit 0
