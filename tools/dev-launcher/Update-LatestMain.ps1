# Update-LatestMain.ps1 -- Button 1 of the dev two-button launcher.
#
# PROMPT 1309: this script now operates against a **dedicated play/build
# checkout** that is structurally separate from the orchestrator/launcher
# checkout. The launcher EXE passes the dedicated path via -PlayRepoRoot;
# command-line callers can pass the same flag or rely on
# $env:CCGS_PLAY_REPO_ROOT / the documented default `D:\_DEV\ccgs-play-main`.
# The orchestrator/launcher repo root (the one that owns the scripts) is
# NEVER switched or reset by this script.
#
# What it does (in order):
#   1. Resolves two roots:
#        $LauncherRoot -- where this script lives (parent-of-parent dir).
#        $PlayRoot     -- the dedicated play/build checkout, from
#                         -PlayRepoRoot, $env:CCGS_PLAY_REPO_ROOT,
#                         $env:CCGS_CANONICAL_MAIN_ROOT, or the documented
#                         default. May initially be missing.
#   2. If $PlayRoot is missing and $LauncherRoot is a git repo, materialises
#      $PlayRoot as a linked git worktree off $LauncherRoot:
#        git -C $LauncherRoot worktree add $PlayRoot main
#      (creating local `main` from origin/main if necessary). When `main` is
#      already checked out by another worktree (typically the launcher root
#      itself), falls back to a dedicated `play-main` local branch that
#      tracks origin/main, so the dedicated checkout still mirrors origin/main
#      without colliding with the launcher root's checkout.
#   3. Inside $PlayRoot, aborts unless the working tree is clean (override
#      with -Force). If the dedicated checkout is on a non-main branch,
#      attempts `git switch main` only if clean; refuses otherwise.
#   4. `git fetch origin` and `git merge --ff-only origin/main`. Aborts on
#      non-FF unless -Force (which then performs git reset --hard, DESTRUCTIVE).
#   5. Applies the documented Windows/MSVC Cargo resource policy:
#        CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
#        CARGO_PROFILE_DEV_DEBUG='0'
#        CARGO_PROFILE_TEST_DEBUG='0'
#        CARGO_INCREMENTAL='0'
#        RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
#   6. Checks D: free space. If under 40 GB AND -AllowCacheClean is passed,
#      cleans verified-stale subdirectories under the resolved Cargo target
#      directory (only that exact tree -- never repo source, reports,
#      production, .git, or evidence).
#   7. Builds the `server` and `client` binaries (debug by default, release
#      with -Release) inside $PlayRoot.
#
# What it does NOT do:
#   - It does not start the server or any client.
#   - It does not push, force-push, or modify any remote branch.
#   - It does not edit production/, qa/, story/sprint trackers, or evidence.
#   - It does not switch branches in the launcher/orchestrator checkout.
#   - It does not run tests.
#
# Usage from PowerShell:
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 -Release
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 -PlayRepoRoot D:\_DEV\ccgs-play-main
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 -Help
#
# One-click: double-click `update-latest-main.bat` at the repo root.

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$Release,
    [switch]$AllowCacheClean,
    [switch]$DryRun,
    [switch]$Help,
    [string]$PlayRepoRoot = ''
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
Update-LatestMain.ps1 -- fetch origin, fast-forward main, rebuild server + client
                        inside a dedicated play/build checkout.

PARAMETERS
  -Force             Allow operation on a dirty tree (stash discarded) and allow
                     non-FF main to be hard-reset to origin/main. DESTRUCTIVE.
                     Never affects the launcher/orchestrator checkout.
  -Release           Build in release mode (default is debug for faster turn).
  -AllowCacheClean   If D: free space is under 40 GB, allow cleanup of stale
                     subdirectories under the resolved CARGO_TARGET_DIR.
  -DryRun            Print every step but do not run git, cargo, or rm.
  -PlayRepoRoot P    Absolute path to the dedicated play/build checkout. If
                     omitted, falls back to `$env:CCGS_PLAY_REPO_ROOT, then
                     `$env:CCGS_CANONICAL_MAIN_ROOT, then the documented
                     default 'D:\_DEV\ccgs-play-main'. If the resolved path
                     does not exist, this script will create it as a git
                     worktree off the launcher repo root.
  -Help              Show this help and exit.

SAFETY
  The launcher/orchestrator checkout is NEVER switched or reset by this script.
  Inside the dedicated play root only:
    Dirty tree                -> abort unless -Force.
    Not on main / play-main   -> attempt 'git switch main' (then 'play-main'
                                 if main is checked out elsewhere) only if
                                 clean; abort unless -Force.
    Non-FF current branch     -> abort unless -Force.
  Low disk and not -AllowCacheClean -> warn, continue without cleanup.
  Cleanup is restricted to the resolved CARGO_TARGET_DIR only.

EXIT CODES
  0 success, 1 generic failure, 2 unsafe state without -Force.
"@ | Write-Host
}

if ($Help) { Show-Help; exit 0 }

# ---- 1. Resolve launcher root + play root -------------------------------
$ScriptDir    = Split-Path -Parent $MyInvocation.MyCommand.Path
$ToolsDir     = Split-Path -Parent $ScriptDir
$LauncherRoot = Split-Path -Parent $ToolsDir

# PROMPT 1402: load the build-provenance helper module so we can emit a
# `last-build-provenance.json` sidecar next to the binaries after build.
# Start-TwoClients.ps1 reads this sidecar to surface rebuild SHA + branch
# even when it does not rebuild itself.
$BuildProvenanceModule = Join-Path $ScriptDir 'BuildProvenance.psm1'
$BuildProvenanceLoaded = $false
try {
    Import-Module $BuildProvenanceModule -Force -ErrorAction Stop
    $BuildProvenanceLoaded = $true
} catch {
    Write-Warning "BuildProvenance helper failed to import ($BuildProvenanceModule): $($_.Exception.Message). Continuing without last-build-provenance.json sidecar."
}

$DefaultPlayRoot = 'D:\_DEV\ccgs-play-main'
$PlayRoot       = ''
$PlayRootSource = ''
if ($PSBoundParameters.ContainsKey('PlayRepoRoot') -and $PlayRepoRoot.Trim().Length -gt 0) {
    $PlayRoot       = $PlayRepoRoot.Trim()
    $PlayRootSource = '-PlayRepoRoot argument'
} elseif ($env:CCGS_PLAY_REPO_ROOT) {
    $PlayRoot       = $env:CCGS_PLAY_REPO_ROOT.Trim()
    $PlayRootSource = '$env:CCGS_PLAY_REPO_ROOT'
} elseif ($env:CCGS_CANONICAL_MAIN_ROOT) {
    $PlayRoot       = $env:CCGS_CANONICAL_MAIN_ROOT.Trim()
    $PlayRootSource = '$env:CCGS_CANONICAL_MAIN_ROOT (alias)'
} else {
    $PlayRoot       = $DefaultPlayRoot
    $PlayRootSource = 'documented dedicated default'
}

Write-Section "Roots"
Write-Host "Launcher repo root: $LauncherRoot"
Write-Host "Play/build root:    $PlayRoot"
Write-Host "Play/build source:  $PlayRootSource"

if (-not (Test-Path (Join-Path $LauncherRoot '.git'))) {
    Write-Host -ForegroundColor Red "Launcher root has no .git directory at $LauncherRoot."
    exit 1
}
if (-not (Test-Path (Join-Path $LauncherRoot 'Cargo.toml'))) {
    Write-Host -ForegroundColor Red "No Cargo.toml at $LauncherRoot -- this does not look like the CCGS workspace."
    exit 1
}

$LauncherRootNorm = [System.IO.Path]::GetFullPath($LauncherRoot)
$PlayRootNorm     = [System.IO.Path]::GetFullPath($PlayRoot)
if ($LauncherRootNorm -ieq $PlayRootNorm) {
    Write-Warning "Play/build root equals the launcher root ($LauncherRootNorm). Operating on a single checkout -- the dedicated-checkout safety net is disabled."
}

# ---- 1b. Materialise play root as a worktree if missing ------------------
# Branch name used when the launcher root already has `main` checked out. Git
# refuses to check out the same branch in two worktrees, so the dedicated
# play/build checkout uses a separate local branch that tracks `origin/main`.
$PlayBranchFallback = 'play-main'

# Detect stub: path exists but has no .git (e.g. leftover from a pruned worktree).
# git worktree add refuses to populate a non-empty directory, so we must abort early
# with actionable instructions rather than proceeding to an opaque .git check failure.
if ((Test-Path $PlayRoot) -and -not (Test-Path (Join-Path $PlayRoot '.git'))) {
    Write-Host -ForegroundColor Red ""
    Write-Host -ForegroundColor Red "ERROR: '$PlayRoot' exists but is not a git checkout (no .git directory)."
    Write-Host -ForegroundColor Yellow "This is a leftover stub directory. Choose one of:"
    Write-Host ""
    Write-Host "  Option A — use the launcher checkout directly (fastest):"
    Write-Host "    powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 ``"
    Write-Host "        -PlayRepoRoot '$LauncherRoot'"
    Write-Host ""
    Write-Host "  Option B — delete the stub so this script auto-creates a linked worktree:"
    Write-Host "    Remove-Item -Recurse -Force '$PlayRoot'"
    Write-Host "    (then re-run Update-LatestMain.ps1 without -PlayRepoRoot)"
    Write-Host ""
    Write-Host "  Option C — set the env var permanently:"
    Write-Host "    `$env:CCGS_PLAY_REPO_ROOT = '$LauncherRoot'"
    exit 1
}

if (-not (Test-Path $PlayRoot)) {
    Write-Section "Create play/build worktree"
    Write-Host "Path $PlayRoot does not exist -- creating as a linked git worktree."
    if ($DryRun) {
        Write-Host "[dry-run] git -C $LauncherRoot fetch origin"
        Write-Host "[dry-run] git -C $LauncherRoot worktree add $PlayRoot main (or -B $PlayBranchFallback origin/main if main is already checked out)"
    } else {
        # Make sure origin/main is up to date so the new worktree starts on a
        # current ref. This fetch is read-only and does not modify any branch.
        git -C $LauncherRoot fetch origin
        if ($LASTEXITCODE -ne 0) {
            Write-Host -ForegroundColor Red "git fetch failed in launcher root; cannot create worktree."
            exit 1
        }
        # Detect whether `main` is already checked out by another worktree (the
        # most common case: the launcher root IS that worktree). If yes, we
        # can't `worktree add ... main` and must use a dedicated branch.
        $mainCheckedOutElsewhere = $false
        $worktreeListRaw = git -C $LauncherRoot worktree list --porcelain 2>$null
        if ($LASTEXITCODE -eq 0 -and $worktreeListRaw) {
            foreach ($line in ($worktreeListRaw -split "`n")) {
                if ($line.Trim() -eq 'branch refs/heads/main') {
                    $mainCheckedOutElsewhere = $true
                    break
                }
            }
        }

        git -C $LauncherRoot show-ref --verify --quiet refs/heads/main
        $localMainExists = ($LASTEXITCODE -eq 0)

        if ($mainCheckedOutElsewhere) {
            Write-Host "Local 'main' is already checked out in another worktree (likely the launcher root)."
            Write-Host "Creating dedicated branch '$PlayBranchFallback' tracking origin/main for the play/build checkout."
            Write-Host "Attempting: git -C $LauncherRoot worktree add -B $PlayBranchFallback $PlayRoot origin/main"
            git -C $LauncherRoot worktree add -B $PlayBranchFallback $PlayRoot origin/main
        } elseif ($localMainExists) {
            Write-Host "Attempting: git -C $LauncherRoot worktree add $PlayRoot main"
            git -C $LauncherRoot worktree add $PlayRoot main
        } else {
            Write-Host "Local 'main' not found; creating from origin/main."
            git -C $LauncherRoot worktree add -B main $PlayRoot origin/main
        }
        if ($LASTEXITCODE -ne 0) {
            Write-Host -ForegroundColor Red "git worktree add failed. Resolve the conflict (likely 'main' is checked out elsewhere) and retry."
            exit 1
        }
        Write-Host "Worktree created at $PlayRoot."
    }
}

if (-not $DryRun) {
    if (-not (Test-Path (Join-Path $PlayRoot '.git'))) {
        Write-Host -ForegroundColor Red "Play/build root has no .git after creation: $PlayRoot."
        exit 1
    }
    if (-not (Test-Path (Join-Path $PlayRoot 'Cargo.toml'))) {
        Write-Host -ForegroundColor Red "Play/build root has no Cargo.toml: $PlayRoot (not a CCGS workspace)."
        exit 1
    }
}

$RepoRoot = $PlayRoot
if ($DryRun -and -not (Test-Path $RepoRoot)) {
    Write-Host "[dry-run] would Set-Location $RepoRoot (skipped: path not materialised)"
    Write-Host ""
    Write-Host "[dry-run] Stopping here -- subsequent git/cargo steps require an existing play/build root."
    exit 0
}
Set-Location $RepoRoot

# ---- 2. Branch + dirty checks (play root only) --------------------------
Write-Section "Git pre-checks (play/build root)"
$Branch = (git rev-parse --abbrev-ref HEAD).Trim()
Write-Host "Current branch (play/build root): $Branch"

# The play/build checkout normally lives on `main`, but when the launcher root
# already holds `main` we create the worktree on a dedicated `play-main`
# branch tracking origin/main. Both are considered canonical for rebuild.
$CanonicalPlayBranches = @('main', $PlayBranchFallback)
$Dirty = (git status --porcelain) -join "`n"
if ($CanonicalPlayBranches -notcontains $Branch) {
    if ($Dirty -and -not $Force) {
        Write-Host -ForegroundColor Red "Play/build root is on '$Branch' AND its working tree is dirty -- refusing to switch."
        Write-Host -ForegroundColor Red "Commit/stash inside $PlayRoot, or re-run with -Force (DESTRUCTIVE)."
        Write-Host $Dirty
        exit 2
    }
    # Prefer switching to `main`; fall back to the dedicated `play-main`
    # branch when `main` is already checked out elsewhere.
    Write-Host "Play/build root is on '$Branch' (clean) -- switching to a canonical branch."
    if ($DryRun) {
        Write-Host "[dry-run] git switch main (or $PlayBranchFallback if main is checked out elsewhere)"
    } else {
        git switch main
        if ($LASTEXITCODE -ne 0) {
            Write-Host "git switch main failed (likely already checked out elsewhere) -- trying '$PlayBranchFallback'."
            git switch $PlayBranchFallback
            if ($LASTEXITCODE -ne 0) {
                Write-Host "git switch $PlayBranchFallback failed -- creating it from origin/main."
                git switch -c $PlayBranchFallback origin/main
                if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git switch to a canonical branch failed."; exit 1 }
            }
            $Branch = $PlayBranchFallback
        } else {
            $Branch = 'main'
        }
    }
}

if ($Dirty -and -not $Force) {
    Write-Host -ForegroundColor Red "Refusing to fast-forward: play/build tree is dirty. Commit, stash, or re-run with -Force (DESTRUCTIVE -- discards changes)."
    Write-Host $Dirty
    exit 2
}
if ($Dirty -and $Force) {
    Write-Warning "Dirty tree in play/build root detected and -Force passed -- will reset hard after fetch."
}

# ---- 3. Fetch + fast-forward (play root only) ---------------------------
Write-Section "git fetch origin"
if ($DryRun) {
    Write-Host "[dry-run] git fetch origin"
} else {
    git fetch origin
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git fetch failed."; exit 1 }
}

Write-Section "Fast-forward $Branch -> origin/main"
# Compare HEAD (whichever canonical branch we landed on) against origin/main.
$AheadBehind = (git rev-list --left-right --count HEAD...origin/main).Trim()
Write-Host "$Branch vs origin/main (ahead/behind): $AheadBehind"
$parts = $AheadBehind -split '\s+'
$ahead  = [int]$parts[0]
$behind = [int]$parts[1]
if ($ahead -gt 0 -and -not $Force) {
    Write-Host -ForegroundColor Red "Refusing to fast-forward: local $Branch is $ahead commit(s) ahead of origin/main. Push or rebase first, or re-run with -Force (DESTRUCTIVE)."
    exit 2
}
if ($behind -eq 0) {
    Write-Host "$Branch is already up to date."
} elseif ($DryRun) {
    Write-Host "[dry-run] git merge --ff-only origin/main"
} elseif ($Force -and $ahead -gt 0) {
    Write-Warning "Force-resetting $Branch to origin/main (discards $ahead local commit(s))."
    git reset --hard origin/main
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git reset --hard failed."; exit 1 }
} else {
    git merge --ff-only origin/main
    if ($LASTEXITCODE -ne 0) { Write-Host -ForegroundColor Red "git merge --ff-only failed."; exit 1 }
}
Write-Host "$Branch HEAD: $((git log -1 --oneline).Trim())"

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

# ---- Build provenance sidecar (PROMPT 1402) -----------------------------
Write-Section "Build provenance sidecar"
if (-not $BuildProvenanceLoaded) {
    Write-Host "last-build-provenance.json: BuildProvenance helper not loaded -- skipped."
} elseif ($DryRun) {
    Write-Host "last-build-provenance.json: -DryRun in effect -- skipped."
} else {
    $targetProfDir = Join-Path $env:CARGO_TARGET_DIR $profileDir
    if (-not (Test-Path $targetProfDir)) {
        Write-Host "last-build-provenance.json: target dir '$targetProfDir' does not exist (build may have failed earlier) -- skipped."
    } else {
        $launcherRel = 'tools/dev-launcher/Update-LatestMain.ps1'
        $launcherRootNorm = ''
        $playRootNorm     = ''
        try {
            $launcherRootNorm = [System.IO.Path]::GetFullPath($LauncherRoot)
            $playRootNorm     = [System.IO.Path]::GetFullPath($RepoRoot)
        } catch { }
        $isLauncherRoot   = ($launcherRootNorm -ne '' -and $launcherRootNorm -ieq $playRootNorm)
        $autoSwitchedOrDedicated = (-not $isLauncherRoot)

        $gitProv    = Read-CcgsGitProvenance -Path $RepoRoot
        $serverInfo = Read-CcgsBinaryInfo -Path $serverBin
        $clientInfo = Read-CcgsBinaryInfo -Path $clientBin
        $cargoEnv   = Get-CcgsCargoEnvSnapshot

        $buildCmds = @()
        $buildCmds += ('cargo ' + ($cargoArgsServer -join ' '))
        $buildCmds += ('cargo ' + ($cargoArgsClient -join ' '))

        $payload = New-CcgsBuildProvenance `
            -Context 'rebuild' `
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
            -LastRebuild $null

        $written = Write-CcgsBuildProvenance -EvidenceDir $targetProfDir -Payload $payload -FileName 'last-build-provenance.json'
        if ($written) {
            Write-Host "last-build-provenance.json written: $written"
            if ($gitProv.head_short) {
                Write-Host "  HEAD = $($gitProv.head_short) on '$($gitProv.branch)' (clean=$($gitProv.is_clean))"
            }
        }
    }
}

Write-Host ""
Write-Host "Next: launch with .\start-two-clients.bat (or tools\dev-launcher\Start-TwoClients.ps1)."
exit 0
