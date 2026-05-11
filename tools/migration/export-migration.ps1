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
    [string]$RepoRoot         = (Resolve-Path "$PSScriptRoot\..\..").Path,
    [string]$ClaudeHome       = (Join-Path $env:USERPROFILE ".claude"),
    [string]$ClaudeJsonMaster = (Join-Path $env:USERPROFILE ".claude.json"),
    [string]$CodexHome        = (Join-Path $env:USERPROFILE ".codex"),
    [string]$OutDir           = "D:\",
    [int]   $RecentJsonlDays  = 7
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
Write-Host "[1/5] Project gitignored state..." -ForegroundColor Yellow
CopyIfExists (Join-Path $RepoRoot "production\session-state") (Join-Path $staging "project\production\session-state")
CopyIfExists (Join-Path $RepoRoot "production\session-logs")  (Join-Path $staging "project\production\session-logs")
CopyIfExists (Join-Path $RepoRoot ".claude\settings.local.json") (Join-Path $staging "project\.claude\settings.local.json")
CopyIfExists (Join-Path $RepoRoot "CLAUDE.local.md")            (Join-Path $staging "project\CLAUDE.local.md")
CopyIfExists (Join-Path $RepoRoot ".agents")                    (Join-Path $staging "project\.agents")
CopyIfExists (Join-Path $RepoRoot ".codex-tmp")                 (Join-Path $staging "project\.codex-tmp")
CopyIfExists (Join-Path $RepoRoot "expansions")                 (Join-Path $staging "project\expansions")

# -- 2. Claude global pieces (for HANDOVER, not blind copy) -----------------
Write-Host ""
Write-Host "[2/5] Claude global state (for handover merge)..." -ForegroundColor Yellow
# Master config at user-home root (MCP servers, project trust, OAuth, session counters).
# Without it the target treats the project as untrusted on first launch.
CopyIfExists $ClaudeJsonMaster (Join-Path $staging "claude\.claude.json")
# Memory files: the canonical "what to remember" — the other Claude will merge these.
CopyIfExists (Join-Path $ClaudeHome "projects\D---DEV-claude-code-game-studios\memory") `
             (Join-Path $staging "claude\memory")
# Agent-level memory (creative-director, producer, technical-director).
CopyIfExists (Join-Path $ClaudeHome "agent-memory") (Join-Path $staging "claude\agent-memory")
# Liv-* skills (subscribed catalog) — used by the project workflow.
CopyIfExists (Join-Path $ClaudeHome "skills") (Join-Path $staging "claude\skills")
# liv-skills subscription manifest — without it /liv-sync misbehaves.
CopyIfExists (Join-Path $ClaudeHome "liv-skills.json") (Join-Path $staging "claude\liv-skills.json")
# Plugins / marketplaces (installed via /plugin).
CopyIfExists (Join-Path $ClaudeHome "plugins") (Join-Path $staging "claude\plugins")
# Scheduled tasks (/schedule cron state).
CopyIfExists (Join-Path $ClaudeHome "tasks")                (Join-Path $staging "claude\tasks")
CopyIfExists (Join-Path $ClaudeHome "scheduled_tasks.lock") (Join-Path $staging "claude\scheduled_tasks.lock")
# Settings + hook scripts (as REFERENCE, not to overwrite the other PC's settings).
CopyIfExists (Join-Path $ClaudeHome "settings.json")    (Join-Path $staging "claude\settings.json")
CopyIfExists (Join-Path $ClaudeHome "CLAUDE.md")        (Join-Path $staging "claude\CLAUDE.md")
CopyIfExists (Join-Path $ClaudeHome "stop-sound.ps1")   (Join-Path $staging "claude\stop-sound.ps1")
CopyIfExists (Join-Path $ClaudeHome "ask-sound.ps1")    (Join-Path $staging "claude\ask-sound.ps1")
CopyIfExists (Join-Path $ClaudeHome "notify-sound.ps1") (Join-Path $staging "claude\notify-sound.ps1")
# Skip: ~/.claude/file-history/, shell-snapshots/, paste-cache/, .credentials.json (auth, per-device)

# -- 2b. Recent conversation transcripts (for /resume on target) ------------
Write-Host ""
Write-Host "[2b/5] Recent .jsonl transcripts (last $RecentJsonlDays days) + most-recent subagents..." -ForegroundColor Yellow
$projDir = Join-Path $ClaudeHome "projects\D---DEV-claude-code-game-studios"
if (Test-Path $projDir) {
    $cutoff = (Get-Date).AddDays(-$RecentJsonlDays)
    $recentJsonl = Get-ChildItem -Path $projDir -Filter "*.jsonl" -File -ErrorAction SilentlyContinue `
        | Where-Object { $_.LastWriteTime -ge $cutoff }
    $dstJsonl = Join-Path $staging "claude\projects-recent"
    New-Item -ItemType Directory -Path $dstJsonl -Force | Out-Null
    foreach ($f in $recentJsonl) {
        Copy-Item $f.FullName (Join-Path $dstJsonl $f.Name) -Force
    }
    Write-Host ("  [ok]   {0} recent .jsonl files (>= {1:yyyy-MM-dd})" -f $recentJsonl.Count, $cutoff)
    # Most-recent session's subagent transcripts.
    $recentSessionDir = Get-ChildItem -Path $projDir -Directory -ErrorAction SilentlyContinue `
        | Where-Object { $_.Name -ne "memory" } `
        | Sort-Object LastWriteTime -Descending `
        | Select-Object -First 1
    if ($recentSessionDir) {
        CopyIfExists $recentSessionDir.FullName (Join-Path $staging ("claude\projects-recent\" + $recentSessionDir.Name))
    }
} else {
    Write-Host "  [skip] $projDir absent"
}

# -- 3. Codex (fresh on target -> safe to blind-copy, MINUS build cache) ----
Write-Host ""
Write-Host "[3/5] Codex state (excluding build cache + 3.3 GB telemetry DB)..." -ForegroundColor Yellow
if (Test-Path $CodexHome) {
    $dst = Join-Path $staging "codex"
    New-Item -ItemType Directory -Path $dst -Force | Out-Null
    # /XD: exclude memories/ (build cache, multi-GB) and .tmp/
    # /XF: exclude logs_2.sqlite* (3.3 GB telemetry — regenerates locally)
    & robocopy $CodexHome $dst /E `
        /XD (Join-Path $CodexHome "memories") (Join-Path $CodexHome ".tmp") `
        /XF "logs_2.sqlite" "logs_2.sqlite-shm" "logs_2.sqlite-wal" `
        /R:1 /W:1 /NFL /NDL /NJH /NJS | Out-Null
    Write-Host "  [ok]   $CodexHome (memories/, .tmp/, logs_2.sqlite* excluded)"
} else {
    Write-Host "  [skip] $CodexHome absent"
}

# -- 4. Zip it up -----------------------------------------------------------
Write-Host ""
Write-Host "[4/5] Compressing -> $zipOut" -ForegroundColor Yellow
if (Test-Path $zipOut) { Remove-Item $zipOut -Force }
Compress-Archive -Path (Join-Path $staging "*") -DestinationPath $zipOut -CompressionLevel Optimal

$sizeMB = [math]::Round((Get-Item $zipOut).Length / 1MB, 1)
Write-Host ""
Write-Host "[5/5] Worktrees notice..." -ForegroundColor Yellow
$worktreeRoot = "$RepoRoot-worktrees"
if (Test-Path $worktreeRoot) {
    $count = (Get-ChildItem $worktreeRoot -Directory -ErrorAction SilentlyContinue).Count
    Write-Host "  !! Detected $count external worktrees at $worktreeRoot" -ForegroundColor Magenta
    Write-Host "     NOT migrated. If any have uncommitted work, commit/push or copy manually." -ForegroundColor Magenta
}

Write-Host ""
Write-Host "== DONE ==" -ForegroundColor Green
Write-Host "Archive: $zipOut  ($sizeMB MB)"
Write-Host "Staging kept at: $staging  (delete manually when verified)"
Write-Host ""
Write-Host "Next: copy this zip to the target PC, then follow tools\migration\IMPORT-README.md"
