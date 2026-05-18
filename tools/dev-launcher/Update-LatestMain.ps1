# Update-LatestMain.ps1 -- Button 1 of the dev two-button launcher.
#
# What it does (in order):
#   1. Resolves the repo root (parent-of-parent of this script).
#   2. Aborts unless the current branch is `main` and the working tree is
#      clean (override with -Force).
#   3. `git fetch origin` and `git merge --ff-only origin/main`. Aborts if the
#      merge is not a fast-forward (override with -Force forces a hard reset,
#      DESTRUCTIVE).
#   4. Applies the documented Windows/MSVC Cargo resource policy:
#        CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
#        CARGO_PROFILE_DEV_DEBUG='0'
#        CARGO_PROFILE_TEST_DEBUG='0'
#        CARGO_INCREMENTAL='0'
#        RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
#   5. Checks D: free space. If under 40 GB AND -AllowCacheClean is passed,
#      cleans verified-stale subdirectories under the resolved Cargo target
#      directory (only that exact tree -- never repo source, reports,
#      production, .git, or evidence). Always prints the resolved target dir
#      and confirms it is safe before deleting anything.
#   6. Builds the `server` and `client` binaries (debug by default, release
#      with -Release).
#
# What it does NOT do:
#   - It does not start the server or any client.
#   - It does not push, force-push, or modify any remote branch.
#   - It does not edit production/, qa/, story/sprint trackers, or evidence.
#   - It does not run tests.
#
# Usage from PowerShell:
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 -Release
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 -Help
#
# One-click: double-click `update-latest-main.bat` at the repo root.

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$Release,
    [switch]$AllowCacheClean,
    [switch]$DryRun,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Section {
    param([string]$Title)
    Write-Host ""
    Write-Host "==== $Title ====" -ForegroundColor Cyan
}

function Show-Help {
    @"
Update-LatestMain.ps1 -- fetch origin, fast-forward main, rebuild server + client.

PARAMETERS
  -Force             Allow operation on a dirty tree (stash discarded) and allow
                     non-FF main to be hard-reset to origin/main. DESTRUCTIVE.
  -Release           Build in release mode (default is debug for faster turn).
  -AllowCacheClean   If D: free space is under 40 GB, allow cleanup of stale
                     subdirectories under the resolved CARGO_TARGET_DIR.
  -DryRun            Print every step but do not run git, cargo, or rm.
  -Help              Show this help and exit.

SAFETY
  Dirty tree -> abort unless -Force.
  Not on main -> abort unless -Force.
  Non-FF main -> abort unless -Force.
  Low disk and not -AllowCacheClean -> warn, continue without cleanup.
  Cleanup is restricted to the resolved CARGO_TARGET_DIR only.

EXIT CODES
  0 success, 1 generic failure, 2 unsafe state without -Force.
"@ | Write-Host
}

if ($Help) { Show-Help; exit 0 }

# ---- 1. Resolve repo root ------------------------------------------------
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ToolsDir  = Split-Path -Parent $ScriptDir
$RepoRoot  = Split-Path -Parent $ToolsDir
Set-Location $RepoRoot

Write-Section "Repo root"
Write-Host "Repo root: $RepoRoot"
if (-not (Test-Path (Join-Path $RepoRoot '.git'))) {
    Write-Host -ForegroundColor Red "Not a git repo (no .git at $RepoRoot)."
    exit 1
}
if (-not (Test-Path (Join-Path $RepoRoot 'Cargo.toml'))) {
    Write-Host -ForegroundColor Red "No Cargo.toml at $RepoRoot -- this does not look like the CCGS workspace."
    exit 1
}

# ---- 2. Branch + dirty checks --------------------------------------------
Write-Section "Git pre-checks"
$Branch = (git rev-parse --abbrev-ref HEAD).Trim()
Write-Host "Current branch: $Branch"
if ($Branch -ne 'main' -and -not $Force) {
    Write-Host -ForegroundColor Red "Refusing to fast-forward: current branch is '$Branch', not 'main'. Re-run with -Force only if you really want to switch to main first."
    exit 2
}

$Dirty = (git status --porcelain) -join "`n"
if ($Dirty -and -not $Force) {
    Write-Host -ForegroundColor Red "Refusing to fast-forward: working tree is dirty. Commit, stash, or re-run with -Force (DESTRUCTIVE -- discards changes)."
    Write-Host $Dirty
    exit 2
}
if ($Dirty -and $Force) {
    Write-Warning "Dirty tree detected and -Force passed -- will reset hard after fetch."
}

# ---- 3. Fetch + fast-forward ---------------------------------------------
Write-Section "git fetch origin"
if ($DryRun) {
    Write-Host "[dry-run] git fetch origin"
} else {
    git fetch origin
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git fetch failed."; exit 1 }
}

if ($Branch -ne 'main') {
    if ($DryRun) {
        Write-Host "[dry-run] git switch main"
    } else {
        git switch main
        if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git switch main failed."; exit 1 }
    }
}

Write-Section "Fast-forward main -> origin/main"
$AheadBehind = (git rev-list --left-right --count main...origin/main).Trim()
Write-Host "main vs origin/main (ahead/behind): $AheadBehind"
$parts = $AheadBehind -split '\s+'
$ahead  = [int]$parts[0]
$behind = [int]$parts[1]
if ($ahead -gt 0 -and -not $Force) {
    Write-Host -ForegroundColor Red "Refusing to fast-forward: local main is $ahead commit(s) ahead of origin/main. Push or rebase first, or re-run with -Force (DESTRUCTIVE)."
    exit 2
}
if ($behind -eq 0) {
    Write-Host "main is already up to date."
} elseif ($DryRun) {
    Write-Host "[dry-run] git merge --ff-only origin/main"
} elseif ($Force -and $ahead -gt 0) {
    Write-Warning "Force-resetting main to origin/main (discards $ahead local commit(s))."
    git reset --hard origin/main
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git reset --hard failed."; exit 1 }
} else {
    git merge --ff-only origin/main
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git merge --ff-only failed."; exit 1 }
}
Write-Host "main HEAD: $((git log -1 --oneline).Trim())"

# ---- 4. Cargo policy -----------------------------------------------------
Write-Section "Cargo resource policy"
$env:CARGO_TARGET_DIR        = 'D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL       = '0'
$env:RUSTFLAGS               = '-C debuginfo=0 -C link-arg=/DEBUG:NONE'
Write-Host "CARGO_TARGET_DIR        = $env:CARGO_TARGET_DIR"
Write-Host "CARGO_PROFILE_DEV_DEBUG = $env:CARGO_PROFILE_DEV_DEBUG"
Write-Host "CARGO_PROFILE_TEST_DEBUG = $env:CARGO_PROFILE_TEST_DEBUG"
Write-Host "CARGO_INCREMENTAL       = $env:CARGO_INCREMENTAL"
Write-Host "RUSTFLAGS               = $env:RUSTFLAGS"

# ---- 5. Disk handling ----------------------------------------------------
Write-Section "Disk check"
$drive = Get-PSDrive -Name D -ErrorAction SilentlyContinue
if ($null -ne $drive) {
    $freeGB = [math]::Round($drive.Free / 1GB, 2)
    Write-Host "D: free = $freeGB GB"
    if ($freeGB -lt 40) {
        Write-Warning "D: free space under 40 GB."
        if ($AllowCacheClean) {
            $target = $env:CARGO_TARGET_DIR
            $safeRoot = 'D:\_DEV\cargo-target\ccgs-msvc'
            Write-Host "Resolved Cargo target dir: $target"
            if ($target -ne $safeRoot) {
                Write-Warning "Resolved target dir differs from documented policy root '$safeRoot'. Skipping cleanup."
            } elseif (-not (Test-Path $target)) {
                Write-Host "Target dir does not exist -- nothing to clean."
            } else {
                # Only the documented Cargo target sub-tree is touched.
                $forbidden = @('production','reports','assets','docs','client','server','shared','tools','tests','design','prototypes','.git','evidence')
                $abort = $false
                foreach ($f in $forbidden) {
                    if ($target -like "*\$f*") { $abort = $true; break }
                }
                if ($abort) {
                    Write-Warning "Resolved target path '$target' contains a forbidden segment. Skipping cleanup."
                } else {
                    Write-Host "Cleaning stale subdirs under: $target"
                    $candidates = @('debug\incremental','release\incremental','debug\deps','debug\build','tmp')
                    foreach ($sub in $candidates) {
                        $p = Join-Path $target $sub
                        if (Test-Path $p) {
                            if ($DryRun) {
                                Write-Host "[dry-run] Remove-Item -Recurse -Force '$p'"
                            } else {
                                Write-Host "  - rm $p"
                                Remove-Item $p -Recurse -Force -ErrorAction Continue
                            }
                        }
                    }
                }
            }
        } else {
            Write-Host "Pass -AllowCacheClean to clean stale subdirs under the resolved CARGO_TARGET_DIR."
        }
    }
}

# ---- 6. Cargo build ------------------------------------------------------
Write-Section "Cargo build"
$cargoArgsServer = @('build','-p','server')
$cargoArgsClient = @('build','-p','client','--bin','client')
if ($Release) {
    $cargoArgsServer += '--release'
    $cargoArgsClient += '--release'
}

Write-Host "cargo $($cargoArgsServer -join ' ')"
if (-not $DryRun) {
    & cargo @cargoArgsServer
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "cargo build server failed."; exit 1 }
}

Write-Host "cargo $($cargoArgsClient -join ' ')"
if (-not $DryRun) {
    & cargo @cargoArgsClient
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "cargo build client failed."; exit 1 }
}

# ---- Done ----------------------------------------------------------------
Write-Section "Done"
$profileDir = if ($Release) { 'release' } else { 'debug' }
$serverBin = Join-Path $env:CARGO_TARGET_DIR "$profileDir\server.exe"
$clientBin = Join-Path $env:CARGO_TARGET_DIR "$profileDir\client.exe"
Write-Host "Server binary: $serverBin (exists=$(Test-Path $serverBin))"
Write-Host "Client binary: $clientBin (exists=$(Test-Path $clientBin))"
Write-Host ""
Write-Host "Next: launch with .\start-two-clients.bat (or tools\dev-launcher\Start-TwoClients.ps1)."
exit 0
