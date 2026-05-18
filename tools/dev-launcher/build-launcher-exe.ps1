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
    [switch]$Help
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
  -Release  Build with --release. Default is debug.
  -DryRun   Print every step but do not run cargo.
  -Help     Show this help and exit.

WHAT IT DOES
  1. Resolves repo root from this script's location.
  2. Applies the documented Windows / MSVC Cargo resource policy
     (CARGO_TARGET_DIR, CARGO_PROFILE_*_DEBUG, CARGO_INCREMENTAL, RUSTFLAGS).
  3. cargo build -p dev-launcher-app --bin ccgs-dev-launcher [--release].
  4. Prints the resolved EXE path.

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

# ---- Report -------------------------------------------------------------
$profileDir = if ($Release) { 'release' } else { 'debug' }
$exePath = Join-Path $env:CARGO_TARGET_DIR "$profileDir\ccgs-dev-launcher.exe"

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
