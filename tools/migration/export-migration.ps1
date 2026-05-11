# Migration export script — runs on the SOURCE PC.
# Produces a single zip with all the non-Git state needed to resume work on another PC.
# Excludes huge build caches, ephemeral logs, and machine-specific Windows linker config.
#
# Usage (Developer PowerShell or any PS 5.1+):
#   cd D:\_DEV\claude-code-game-studios
#   .\tools\migration\export-migration.ps1
#
# Output: D:\ccgs-migration-YYYY-MM-DD.zip  (path printed at the end)

param(
    [string]$RepoRoot   = (Resolve-Path "$PSScriptRoot\..\..").Path,
    [string]$ClaudeHome = (Join-Path $env:USERPROFILE ".claude"),
    [string]$CodexHome  = (Join-Path $env:USERPROFILE ".codex"),
    [string]$OutDir     = "D:\"
)

$ErrorActionPreference = "Stop"
$stamp   = Get-Date -Format "yyyy-MM-dd"
$staging = Join-Path $env:TEMP "ccgs-migration-$stamp"
$zipOut  = Join-Path $OutDir "ccgs-migration-$stamp.zip"

if (Test-Path $staging) { Remove-Item $staging -Recurse -Force }
New-Item -ItemType Directory -Path $staging | Out-Null

function CopyIfExists($src, $dst, [string[]]$exclude = @()) {
    if (-not (Test-Path $src)) { Write-Host "  [skip] $src (absent)"; return }
    New-Item -ItemType Directory -Path (Split-Path $dst) -Force | Out-Null
    if ((Get-Item $src).PSIsContainer) {
        $args = @($src, $dst, "/E", "/COPYALL:DAT", "/R:1", "/W:1", "/NFL", "/NDL", "/NJH", "/NJS")
        foreach ($x in $exclude) { $args += @("/XD", $x); $args += @("/XF", $x) }
        & robocopy @args | Out-Null
    } else {
        Copy-Item $src $dst -Force
    }
    Write-Host "  [ok]   $src"
}

Write-Host "== CCGS migration export ==" -ForegroundColor Cyan
Write-Host "Repo:        $RepoRoot"
Write-Host "Claude home: $ClaudeHome"
Write-Host "Codex home:  $CodexHome"
Write-Host "Staging:     $staging"
Write-Host ""

# -- 1. Project-local non-Git state -----------------------------------------
Write-Host "[1/4] Project gitignored state..." -ForegroundColor Yellow
CopyIfExists (Join-Path $RepoRoot "production\session-state") (Join-Path $staging "project\production\session-state")
CopyIfExists (Join-Path $RepoRoot "production\session-logs")  (Join-Path $staging "project\production\session-logs")
CopyIfExists (Join-Path $RepoRoot ".claude\settings.local.json") (Join-Path $staging "project\.claude\settings.local.json")
CopyIfExists (Join-Path $RepoRoot "CLAUDE.local.md")            (Join-Path $staging "project\CLAUDE.local.md")
CopyIfExists (Join-Path $RepoRoot ".agents")                    (Join-Path $staging "project\.agents")
CopyIfExists (Join-Path $RepoRoot ".codex-tmp")                 (Join-Path $staging "project\.codex-tmp")
CopyIfExists (Join-Path $RepoRoot "expansions")                 (Join-Path $staging "project\expansions")

# -- 2. Claude global pieces (for HANDOVER, not blind copy) -----------------
Write-Host ""
Write-Host "[2/4] Claude global state (for handover merge)..." -ForegroundColor Yellow
# Memory files: the canonical "what to remember" — the other Claude will merge these.
CopyIfExists (Join-Path $ClaudeHome "projects\D---DEV-claude-code-game-studios\memory") `
             (Join-Path $staging "claude\memory")
# Liv-* skills (subscribed catalog) — used by the project workflow.
CopyIfExists (Join-Path $ClaudeHome "skills") (Join-Path $staging "claude\skills")
# Settings + hook scripts (as REFERENCE, not to overwrite the other PC's settings).
CopyIfExists (Join-Path $ClaudeHome "settings.json")    (Join-Path $staging "claude\settings.json")
CopyIfExists (Join-Path $ClaudeHome "CLAUDE.md")        (Join-Path $staging "claude\CLAUDE.md")
CopyIfExists (Join-Path $ClaudeHome "stop-sound.ps1")   (Join-Path $staging "claude\stop-sound.ps1")
CopyIfExists (Join-Path $ClaudeHome "ask-sound.ps1")    (Join-Path $staging "claude\ask-sound.ps1")
CopyIfExists (Join-Path $ClaudeHome "notify-sound.ps1") (Join-Path $staging "claude\notify-sound.ps1")
# We intentionally SKIP: ~/.claude/projects/*/*.jsonl (transcripts — big, low value)
#                       ~/.claude/file-history/, shell-snapshots/, paste-cache/

# -- 3. Codex (fresh on target -> safe to blind-copy, MINUS build cache) ----
Write-Host ""
Write-Host "[3/4] Codex state (excluding 'memories' build cache, several GB)..." -ForegroundColor Yellow
if (Test-Path $CodexHome) {
    $dst = Join-Path $staging "codex"
    New-Item -ItemType Directory -Path $dst -Force | Out-Null
    # robocopy with /XD memories  -> exclude that one huge directory
    & robocopy $CodexHome $dst /E /XD (Join-Path $CodexHome "memories") `
        (Join-Path $CodexHome ".tmp") /R:1 /W:1 /NFL /NDL /NJH /NJS | Out-Null
    Write-Host "  [ok]   $CodexHome (memories/ + .tmp/ excluded)"
} else {
    Write-Host "  [skip] $CodexHome absent"
}

# -- 4. Zip it up -----------------------------------------------------------
Write-Host ""
Write-Host "[4/4] Compressing -> $zipOut" -ForegroundColor Yellow
if (Test-Path $zipOut) { Remove-Item $zipOut -Force }
Compress-Archive -Path (Join-Path $staging "*") -DestinationPath $zipOut -CompressionLevel Optimal

$sizeMB = [math]::Round((Get-Item $zipOut).Length / 1MB, 1)
Write-Host ""
Write-Host "== DONE ==" -ForegroundColor Green
Write-Host "Archive: $zipOut  ($sizeMB MB)"
Write-Host "Staging kept at: $staging  (delete manually when verified)"
Write-Host ""
Write-Host "Next: copy this zip to the target PC, then follow tools\migration\IMPORT-README.md"
