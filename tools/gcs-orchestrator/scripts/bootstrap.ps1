# GCS Orchestrator bootstrap (Windows).
#
# Fresh install on a new PC, or repair an existing install. Idempotent.
#
# Steps (each one is safe to re-run):
#   1. Verify Python 3.10+ is on PATH
#   2. pip install -e the gcs-orchestrator package (with [tui] extras)
#   3. Copy templates/gcs.toml.example -> ~/.codex/gcs.toml if absent
#   4. Probe for Codex CLI; warn if not found
#   5. Probe for app-server health on ws://127.0.0.1:9787
#   6. Suggest next steps (NSSM install, viewer launch, etc.)
#
# Run:
#   PowerShell -ExecutionPolicy Bypass -File bootstrap.ps1
#
# Optional flags:
#   -InstallNssmService   also install the NSSM service (requires admin)
#   -SkipPipInstall       skip pip install (useful when iterating on the
#                         package and you already did a manual install)

param(
    [switch]$InstallNssmService,
    [switch]$SkipPipInstall
)

$ErrorActionPreference = "Stop"
$PackageDir = (Resolve-Path "$PSScriptRoot\..").Path
$CodexHome = "$env:USERPROFILE\.codex"

function Write-Step($n, $msg) {
    Write-Host ""
    Write-Host "[$n/6] $msg" -ForegroundColor Cyan
}

function Write-Ok($msg)    { Write-Host "  OK: $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "  WARN: $msg" -ForegroundColor Yellow }
function Write-Err($msg)   { Write-Host "  ERR: $msg" -ForegroundColor Red }

# ---- 1. Python ----
Write-Step 1 "Verify Python 3.10+"
$pyv = & python --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Err "python not on PATH. Install Python 3.10+ from python.org and retry."
    exit 2
}
Write-Ok $pyv
$verNum = ($pyv -replace "Python\s+", "") -split "\."
if ([int]$verNum[0] -lt 3 -or ([int]$verNum[0] -eq 3 -and [int]$verNum[1] -lt 10)) {
    Write-Err "Python $($verNum[0]).$($verNum[1]) is below required 3.10"
    exit 2
}

# ---- 2. pip install -e .[tui] ----
Write-Step 2 "pip install -e gcs-orchestrator[tui]"
if ($SkipPipInstall) {
    Write-Warn "Skipped (-SkipPipInstall)"
} else {
    Push-Location $PackageDir
    try {
        & python -m pip install -e ".[tui]" --quiet
        if ($LASTEXITCODE -ne 0) {
            Write-Err "pip install failed"
            exit 2
        }
        Write-Ok "Package installed (editable mode, [tui] extras)"
    } finally {
        Pop-Location
    }
}

# Verify entry points are on PATH
$entryPoints = @("gcs-relay", "gcs-viewer-tui", "gcs-app-supervisor", "gcs-relay-daemon",
                 "gcs-app-status", "gcs-rollout-grep", "gcs-reconcile")
foreach ($ep in $entryPoints) {
    $found = Get-Command $ep -ErrorAction SilentlyContinue
    if ($found) {
        Write-Ok "$ep -> $($found.Source)"
    } else {
        Write-Warn "$ep not on PATH (pip install may have failed silently)"
    }
}

# ---- 3. Copy gcs.toml template ----
Write-Step 3 "~/.codex/gcs.toml"
$gcsToml = Join-Path $CodexHome "gcs.toml"
$template = Join-Path $PackageDir "templates\gcs.toml.example"
if (Test-Path $gcsToml) {
    Write-Ok "$gcsToml already exists (preserved)"
} elseif (-not (Test-Path $template)) {
    Write-Warn "Template not found at $template — config will use defaults"
} else {
    if (-not (Test-Path $CodexHome)) {
        New-Item -ItemType Directory -Path $CodexHome | Out-Null
    }
    Copy-Item $template $gcsToml
    Write-Ok "Created $gcsToml from template (edit it for your session_id)"
}

# ---- 4. Codex CLI ----
Write-Step 4 "Codex CLI"
$codex = Get-Command codex -ErrorAction SilentlyContinue
if ($codex) {
    Write-Ok "codex -> $($codex.Source)"
    & codex --version 2>&1 | ForEach-Object { Write-Host "    $_" }
} else {
    $npmCodex = "$env:USERPROFILE\AppData\Roaming\npm\codex.cmd"
    if (Test-Path $npmCodex) {
        Write-Ok "codex (via npm install location) -> $npmCodex"
    } else {
        Write-Warn "codex not found. Install: npm install -g @openai/codex"
    }
}

# ---- 5. app-server health probe ----
Write-Step 5 "app-server health (127.0.0.1:9787/readyz)"
try {
    $r = Invoke-WebRequest -Uri "http://127.0.0.1:9787/readyz" -UseBasicParsing -TimeoutSec 3 -ErrorAction Stop
    if ($r.StatusCode -eq 200) {
        Write-Ok "app-server is UP"
    } else {
        Write-Warn "app-server returned HTTP $($r.StatusCode)"
    }
} catch {
    Write-Warn "app-server is DOWN. Start it manually with:"
    Write-Host "    codex app-server --listen ws://127.0.0.1:9787" -ForegroundColor Gray
    Write-Host "  or run gcs-app-supervisor (wraps it with restart + monitoring)" -ForegroundColor Gray
}

# ---- 6. NSSM service install ----
Write-Step 6 "NSSM Windows service"
if ($InstallNssmService) {
    $nssmScript = Join-Path $PackageDir "scripts\install-nssm-service.ps1"
    if (-not (Test-Path $nssmScript)) {
        Write-Err "install-nssm-service.ps1 not found"
    } else {
        Write-Host "  Running NSSM install (needs admin)..." -ForegroundColor Gray
        & $nssmScript
    }
} else {
    $svc = Get-Service gcs-app-server -ErrorAction SilentlyContinue
    if ($svc) {
        Write-Ok "Service gcs-app-server exists: $($svc.Status)"
    } else {
        Write-Warn "Service gcs-app-server not installed."
        Write-Host "    To install (requires admin):" -ForegroundColor Gray
        Write-Host "    PowerShell -ExecutionPolicy Bypass -File bootstrap.ps1 -InstallNssmService" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "Bootstrap complete." -ForegroundColor Green
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  - Edit ~/.codex/gcs.toml to set your session_id"
Write-Host "  - Launch viewer:        gcs-viewer-tui"
Write-Host "  - Status:               curl http://127.0.0.1:9788/status"
Write-Host "  - Search rollout:       gcs-rollout-grep '<query>'"
Write-Host "  - Reconcile orphans:    gcs-reconcile --since 24h"
Write-Host "  - Docs:                 $PackageDir\docs\onboarding.md"
