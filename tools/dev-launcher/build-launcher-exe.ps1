# build-launcher-exe.ps1 -- build the CCGS Dev Launcher Windows EXE
# (`tools/dev-launcher-app`) under the workspace's documented Cargo policy.
#
# Output: D:\_DEV\cargo-target\ccgs-msvc\<profile>\ccgs-dev-launcher.exe
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1 -Release
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1 -DryRun
#   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1 -Help

[CmdletBinding()]
param(
    [switch]$Release,
    [switch]$DryRun,
    [switch]$Help,
    [string]$CanonicalRepoRoot,
    [switch]$AllowWorkerWorktreeSidecar
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Section {
    param([string]$Title)
    Write-Host ""
    Write-Host "==== $Title ====" -ForegroundColor Cyan
}

if ($Help) {
    @"
build-launcher-exe.ps1 -- build the CCGS Dev Launcher EXE.

PARAMETERS
  -Release                       Build with --release. Default is debug.
  -DryRun                        Print every step but do not run cargo.
  -Help                          Show this help and exit.
  -CanonicalRepoRoot <path>      Explicit canonical repo root to write into
                                 the sidecar. Overrides every other source.
  -AllowWorkerWorktreeSidecar    Permit writing this checkout's worker
                                 worktree path into the sidecar. Without
                                 this flag, building from a non-main worker
                                 worktree refuses to write the sidecar (it
                                 would pin the EXE to the worker checkout
                                 and defeat Rebuild Latest Main).

WHAT IT DOES
  1. Resolves the build repo root from this script's location (the worktree
     that owns the code being compiled).
  2. Applies the documented Windows / MSVC Cargo resource policy
     (CARGO_TARGET_DIR, CARGO_PROFILE_*_DEBUG, CARGO_INCREMENTAL, RUSTFLAGS).
  3. cargo build -p dev-launcher-app --bin ccgs-dev-launcher [--release].
  4. Resolves the CANONICAL repo root that will be written into the sidecar
     (`ccgs-dev-launcher.repo-root.txt`):
       a. -CanonicalRepoRoot <path>              (explicit override)
       b. \$env:CCGS_CANONICAL_REPO_ROOT          (env override)
       c. the build repo root, IF it is on branch `main`
       d. D:\_DEV\Work\Claude-Code-Game-Studios   (documented default), if
                                                  it exists and is a valid
                                                  CCGS workspace
       e. else: refuse to write the sidecar unless
          -AllowWorkerWorktreeSidecar is passed.
  5. Writes the sidecar next to the built EXE pointing at the resolved
     canonical root.
  6. Prints the resolved EXE path.

WHAT IT DOES NOT DO
  - Does not run the EXE.
  - Does not run the underlying launcher scripts.
  - Does not modify git, production/, sprint state, or evidence.
"@ | Write-Host
    exit 0
}

# ---- Resolve repo root --------------------------------------------------
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ToolsDir  = Split-Path -Parent $ScriptDir
$RepoRoot  = Split-Path -Parent $ToolsDir
Set-Location $RepoRoot

Write-Section "Repo root"
Write-Host "Repo root: $RepoRoot"
if (-not (Test-Path (Join-Path $RepoRoot 'Cargo.toml'))) {
    Write-Host -ForegroundColor Red "No Cargo.toml at $RepoRoot -- this does not look like the CCGS workspace."
    exit 1
}
if (-not (Test-Path (Join-Path $RepoRoot 'tools\dev-launcher-app\Cargo.toml'))) {
    Write-Host -ForegroundColor Red "No tools\dev-launcher-app\Cargo.toml at $RepoRoot -- launcher crate missing."
    exit 1
}

# ---- Cargo policy -------------------------------------------------------
Write-Section "Cargo resource policy"
$env:CARGO_TARGET_DIR         = 'D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG  = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL        = '0'
$env:RUSTFLAGS                = '-C debuginfo=0 -C link-arg=/DEBUG:NONE'
Write-Host "CARGO_TARGET_DIR        = $env:CARGO_TARGET_DIR"
Write-Host "CARGO_PROFILE_DEV_DEBUG = $env:CARGO_PROFILE_DEV_DEBUG"
Write-Host "CARGO_PROFILE_TEST_DEBUG = $env:CARGO_PROFILE_TEST_DEBUG"
Write-Host "CARGO_INCREMENTAL       = $env:CARGO_INCREMENTAL"
Write-Host "RUSTFLAGS               = $env:RUSTFLAGS"

# ---- Build --------------------------------------------------------------
Write-Section "Cargo build"
$cargoArgs = @('build','-p','dev-launcher-app','--bin','ccgs-dev-launcher')
if ($Release) { $cargoArgs += '--release' }
Write-Host "cargo $($cargoArgs -join ' ')"

if (-not $DryRun) {
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host -ForegroundColor Red "cargo build failed."
        exit 1
    }
}

# ---- Canonical repo root ------------------------------------------------
# PROMPT 1290: when the build is invoked from a worker worktree on a
# non-main branch, we MUST NOT silently write the worktree path into the
# sidecar -- that pins the EXE to a worker checkout and the `Rebuild Latest
# Main` button then refuses with "current branch is 'work/...'".
#
# Resolution order (first hit wins):
#   1. -CanonicalRepoRoot <path>              (explicit override)
#   2. $env:CCGS_CANONICAL_REPO_ROOT          (env override)
#   3. $BuildRepoRoot (this checkout), if it is on branch `main`
#   4. D:\_DEV\Work\Claude-Code-Game-Studios (default), if valid
#   5. else: refuse to write the sidecar unless -AllowWorkerWorktreeSidecar.
Write-Section "Canonical sidecar root"
$BuildBranch = ''
try {
    $BuildBranch = (& git -C $RepoRoot rev-parse --abbrev-ref HEAD 2>$null).Trim()
} catch {
    $BuildBranch = ''
}
Write-Host "Build branch: $BuildBranch"

function Test-IsValidCcgsRepo {
    param([string]$Path)
    if (-not $Path) { return $false }
    if (-not (Test-Path $Path)) { return $false }
    if (-not (Test-Path (Join-Path $Path 'Cargo.toml'))) { return $false }
    if (-not (Test-Path (Join-Path $Path 'tools\dev-launcher'))) { return $false }
    if (-not (Test-Path (Join-Path $Path '.git'))) { return $false }
    return $true
}

$BuildRepoRoot    = $RepoRoot
$DefaultCanonical = 'D:\_DEV\Work\Claude-Code-Game-Studios'
$CanonicalRoot    = $null
$CanonicalSource  = ''

if ($PSBoundParameters.ContainsKey('CanonicalRepoRoot') -and $CanonicalRepoRoot.Trim().Length -gt 0) {
    if (Test-IsValidCcgsRepo -Path $CanonicalRepoRoot) {
        $CanonicalRoot   = $CanonicalRepoRoot
        $CanonicalSource = '-CanonicalRepoRoot override'
    } else {
        Write-Host -ForegroundColor Red "-CanonicalRepoRoot '$CanonicalRepoRoot' is not a valid CCGS workspace."
        exit 2
    }
}

if (-not $CanonicalRoot -and $env:CCGS_CANONICAL_REPO_ROOT) {
    $envCandidate = $env:CCGS_CANONICAL_REPO_ROOT.Trim()
    if ($envCandidate -and (Test-IsValidCcgsRepo -Path $envCandidate)) {
        $CanonicalRoot   = $envCandidate
        $CanonicalSource = '$env:CCGS_CANONICAL_REPO_ROOT'
    } elseif ($envCandidate) {
        Write-Warning "CCGS_CANONICAL_REPO_ROOT='$envCandidate' is not a valid CCGS workspace; ignoring."
    }
}

if (-not $CanonicalRoot -and $BuildBranch -eq 'main' -and (Test-IsValidCcgsRepo -Path $BuildRepoRoot)) {
    $CanonicalRoot   = $BuildRepoRoot
    $CanonicalSource = 'build checkout on main'
}

if (-not $CanonicalRoot -and (Test-IsValidCcgsRepo -Path $DefaultCanonical)) {
    $CanonicalRoot   = $DefaultCanonical
    $CanonicalSource = 'documented default (D:\_DEV\Work\Claude-Code-Game-Studios)'
}

if (-not $CanonicalRoot) {
    if ($AllowWorkerWorktreeSidecar) {
        $CanonicalRoot   = $BuildRepoRoot
        $CanonicalSource = '-AllowWorkerWorktreeSidecar (worker worktree path)'
        Write-Warning "Writing worker-worktree path into sidecar by explicit override -- Rebuild Latest Main will refuse this checkout."
    } else {
        Write-Host -ForegroundColor Red ""
        Write-Host -ForegroundColor Red "Refusing to write the launcher sidecar."
        Write-Host -ForegroundColor Red "  Build checkout: $BuildRepoRoot"
        Write-Host -ForegroundColor Red "  Build branch:   $BuildBranch"
        Write-Host -ForegroundColor Red "  Default canonical '$DefaultCanonical' is not a valid CCGS workspace."
        Write-Host -ForegroundColor Red ""
        Write-Host -ForegroundColor Red "To recover, do ONE of the following:"
        Write-Host -ForegroundColor Red "  - Re-run with  -CanonicalRepoRoot <absolute-path-to-canonical-checkout-on-main>"
        Write-Host -ForegroundColor Red "  - Set         `$env:CCGS_CANONICAL_REPO_ROOT to your canonical checkout"
        Write-Host -ForegroundColor Red "  - Run this script from your canonical checkout while it is on branch 'main'"
        Write-Host -ForegroundColor Red "  - Pass -AllowWorkerWorktreeSidecar (intentional worker pin, dev-only)"
        Write-Host -ForegroundColor Red ""
        exit 2
    }
}

Write-Host "Canonical repo root: $CanonicalRoot"
Write-Host "Resolved via: $CanonicalSource"

# ---- Sidecar ------------------------------------------------------------
# The EXE may run from the external Cargo target dir
# (e.g. D:\_DEV\cargo-target\ccgs-msvc\debug), which sits outside the repo
# tree. Walking up from the EXE never reaches the repo. We write a sidecar
# file next to the EXE so the launcher can resolve repo root deterministically.
$profileDir   = if ($Release) { 'release' } else { 'debug' }
$exeDir       = Join-Path $env:CARGO_TARGET_DIR $profileDir
$exePath      = Join-Path $exeDir 'ccgs-dev-launcher.exe'
$sidecarPath  = Join-Path $exeDir 'ccgs-dev-launcher.repo-root.txt'

Write-Section "Sidecar (repo root)"
Write-Host "Sidecar path: $sidecarPath"
Write-Host "Sidecar contents (first non-blank line): $CanonicalRoot"
if (-not $DryRun) {
    if (-not (Test-Path $exeDir)) {
        New-Item -ItemType Directory -Path $exeDir -Force | Out-Null
    }
    $sidecarBody = @(
        "# ccgs-dev-launcher.repo-root.txt"
        "# Generated by tools\dev-launcher\build-launcher-exe.ps1"
        "# Consumed by tools/dev-launcher-app/src/main.rs at startup."
        "# Format: first non-blank, non-comment line is the absolute repo root."
        "# Canonical resolution: $CanonicalSource"
        $CanonicalRoot
    ) -join "`r`n"
    # Write as UTF-8 *without* BOM. PowerShell 5.x `Set-Content -Encoding UTF8`
    # prepends a UTF-8 BOM (0xEF 0xBB 0xBF), which then attaches to the first
    # line of the file. The Rust parser strips that BOM defensively, but we
    # also want the on-disk bytes to be clean so cat/Get-Content/diff/etc.
    # show the comment header without a leading "" glyph. Using
    # [System.IO.File]::WriteAllText with UTF8Encoding($false) emits no BOM
    # on both Windows PowerShell 5.x and PowerShell Core 7+.
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($sidecarPath, $sidecarBody, $utf8NoBom)
    if (Test-Path $sidecarPath) {
        $firstBytes = [System.IO.File]::ReadAllBytes($sidecarPath) | Select-Object -First 3
        $hasBom = ($firstBytes.Count -ge 3 -and $firstBytes[0] -eq 0xEF -and $firstBytes[1] -eq 0xBB -and $firstBytes[2] -eq 0xBF)
        if ($hasBom) {
            Write-Warning "Sidecar written WITH UTF-8 BOM -- parser tolerates this but writer should be no-BOM. Check encoding."
        } else {
            Write-Host "Sidecar written: yes (UTF-8 no-BOM)"
        }
    } else {
        Write-Warning "Sidecar not present after write."
    }
}

# ---- Report -------------------------------------------------------------
Write-Section "Done"
Write-Host "EXE path: $exePath"
if (-not $DryRun) {
    if (Test-Path $exePath) {
        $info = Get-Item $exePath
        Write-Host "Size: $([math]::Round($info.Length / 1MB, 2)) MB"
    } else {
        Write-Warning "EXE not found at expected path."
    }
}
Write-Host ""
Write-Host "Run with: $exePath"
exit 0
