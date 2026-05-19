# BuildProvenance.psm1 -- dev-launcher build provenance helper.
#
# PROMPT 1402 (AUDIT-1392-P05 / AUDIT-1392-O02): every dev-run evidence
# directory must carry a `build.json` so a captured test binary can be tied
# back to a specific origin/main commit. Without this, several user-reported
# defects cannot be distinguished from stale-binary drift.
#
# Design constraints:
#   - Pure, testable core. `New-CcgsBuildProvenance` takes the git/build
#     metadata as parameters (no command execution), returns a PSObject
#     shaped exactly as the on-disk `build.json`. This is what the Pester
#     test suite exercises with mocked inputs.
#   - Thin discovery wrappers. `Read-CcgsGitProvenance`, `Read-CcgsBinaryInfo`,
#     and `Get-CcgsCargoEnvSnapshot` collect inputs from the running session
#     and feed them into the pure core. They tolerate missing `git`, missing
#     binaries, and non-repo paths.
#   - No new process spawns beyond the `git` calls already made by the
#     existing launcher scripts.
#
# Exposed (Export-ModuleMember at the bottom):
#   New-CcgsBuildProvenance      -- pure builder, the unit-tested core.
#   ConvertTo-CcgsBuildProvenanceJson -- consistent JSON formatting.
#   Write-CcgsBuildProvenance    -- writes <evidence-dir>/build.json.
#   Read-CcgsGitProvenance       -- discover git branch/sha/status from a path.
#   Read-CcgsBinaryInfo          -- discover {exists,size,mtime} for a binary.
#   Get-CcgsCargoEnvSnapshot     -- capture relevant CARGO_*/RUSTFLAGS env vars.
#   Read-CcgsLastBuildProvenance -- read sidecar emitted by Update-LatestMain.

Set-StrictMode -Version Latest

$script:CcgsBuildProvenanceSchemaVersion = 1
$script:CcgsLastBuildSidecarName         = 'last-build-provenance.json'

function Read-CcgsGitProvenance {
    <#
    .SYNOPSIS
        Reads git branch, HEAD sha + subject, status --short --branch, and a
        boolean is_clean from `Path`. Tolerates missing `git` or non-repo paths
        by returning a populated object with `error` set.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$Path
    )

    $result = [ordered]@{
        path                = $Path
        branch              = $null
        head_sha            = $null
        head_short          = $null
        head_subject        = $null
        head_committed_at   = $null
        status_short_branch = $null
        is_clean            = $null
        dirty_files         = @()
        error               = $null
    }

    if (-not (Test-Path $Path)) {
        $result.error = "path does not exist: $Path"
        return [pscustomobject]$result
    }
    if (-not (Test-Path (Join-Path $Path '.git'))) {
        $result.error = "no .git directory at $Path (not a worktree)"
        return [pscustomobject]$result
    }

    try {
        $branch = (& git -C $Path rev-parse --abbrev-ref HEAD 2>$null)
        if ($LASTEXITCODE -eq 0 -and $branch) { $result.branch = $branch.Trim() }

        $sha = (& git -C $Path rev-parse HEAD 2>$null)
        if ($LASTEXITCODE -eq 0 -and $sha) {
            $result.head_sha   = $sha.Trim()
            $result.head_short = $result.head_sha.Substring(0, [Math]::Min(12, $result.head_sha.Length))
        }

        $subject = (& git -C $Path log -1 --pretty=%s HEAD 2>$null)
        if ($LASTEXITCODE -eq 0 -and $subject) { $result.head_subject = $subject.Trim() }

        $when = (& git -C $Path log -1 --pretty=%cI HEAD 2>$null)
        if ($LASTEXITCODE -eq 0 -and $when) { $result.head_committed_at = $when.Trim() }

        $statusLines = (& git -C $Path status --short --branch 2>$null)
        if ($LASTEXITCODE -eq 0) {
            $joined = if ($null -eq $statusLines) { '' } else { ($statusLines -join "`n") }
            $result.status_short_branch = $joined
            $dirty = @()
            foreach ($line in @($statusLines)) {
                if ($null -eq $line) { continue }
                $trim = $line.ToString()
                # `--branch` adds a header line beginning with `##`. Anything
                # else with content indicates a working-tree change.
                if ($trim.StartsWith('##')) { continue }
                if ($trim.Trim().Length -eq 0) { continue }
                $dirty += $trim
            }
            $result.dirty_files = $dirty
            $result.is_clean    = ($dirty.Count -eq 0)
        }
    } catch {
        $result.error = "git invocation failed: $($_.Exception.Message)"
    }

    return [pscustomobject]$result
}

function Read-CcgsBinaryInfo {
    <#
    .SYNOPSIS
        Returns a uniform PSObject describing a built artifact. Never throws
        on missing files: `exists=$false` and the rest are $null.
    #>
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)][string]$Path)

    $info = [ordered]@{
        path        = $Path
        exists      = $false
        size_bytes  = $null
        modified_at = $null
    }
    if (Test-Path $Path) {
        try {
            $item = Get-Item -LiteralPath $Path
            $info.exists      = $true
            $info.size_bytes  = [int64]$item.Length
            $info.modified_at = $item.LastWriteTimeUtc.ToString('o')
        } catch {
            # Leave as not-exists if Get-Item fails.
        }
    }
    return [pscustomobject]$info
}

function Get-CcgsCargoEnvSnapshot {
    <#
    .SYNOPSIS
        Captures the subset of process env vars relevant to the documented
        Windows/MSVC Cargo resource policy. Keys are always present (null if
        the env var is unset) so the JSON shape is stable.
    #>
    [CmdletBinding()]
    param()
    $keys = @(
        'CARGO_TARGET_DIR',
        'CARGO_PROFILE_DEV_DEBUG',
        'CARGO_PROFILE_TEST_DEBUG',
        'CARGO_INCREMENTAL',
        'RUSTFLAGS',
        'CCGS_PLAY_REPO_ROOT',
        'CCGS_CANONICAL_MAIN_ROOT',
        'CCGS_CANONICAL_REPO_ROOT'
    )
    $out = [ordered]@{}
    foreach ($k in $keys) {
        $value = [System.Environment]::GetEnvironmentVariable($k)
        if ([string]::IsNullOrEmpty($value)) { $out[$k] = $null } else { $out[$k] = $value }
    }
    return [pscustomobject]$out
}

function Read-CcgsLastBuildProvenance {
    <#
    .SYNOPSIS
        Reads the sidecar emitted by Update-LatestMain.ps1 (if present) and
        returns the parsed PSObject. Returns $null if the file is missing or
        unreadable; callers should treat that as "no rebuild record".
    #>
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)][string]$TargetProfileDir)
    if (-not $TargetProfileDir) { return $null }
    $sidecar = Join-Path $TargetProfileDir $script:CcgsLastBuildSidecarName
    if (-not (Test-Path $sidecar)) { return $null }
    try {
        $text = Get-Content -LiteralPath $sidecar -Raw -ErrorAction Stop
        if ([string]::IsNullOrWhiteSpace($text)) { return $null }
        return ($text | ConvertFrom-Json -ErrorAction Stop)
    } catch {
        return $null
    }
}

function New-CcgsBuildProvenance {
    <#
    .SYNOPSIS
        Pure builder. Assembles the on-disk `build.json` PSObject from already-
        gathered inputs. No process spawn, no filesystem access. This is the
        function the Pester suite exercises directly.

    .PARAMETER Context
        'launch' (Start-TwoClients evidence dir) or 'rebuild' (Update-LatestMain
        sidecar). Free-form string accepted; the wrappers pass one of those two.

    .PARAMETER GeneratedAtUtc
        The UTC `[datetime]` representing now. Injectable for deterministic tests.

    .PARAMETER RepoRoot, RepoRootSource, IsLauncherRoot, AutoSwitchedOrDedicated
        Repo root provenance the launcher resolved before calling this.

    .PARAMETER Git
        PSObject from Read-CcgsGitProvenance (or an equivalent test fixture).

    .PARAMETER BuildProfile
        'debug' or 'release'.

    .PARAMETER BuildCommands
        Array of strings naming the cargo invocations the launcher used
        (e.g. `cargo build -p server`).

    .PARAMETER TargetDir
        Resolved $env:CARGO_TARGET_DIR (or fallback) at gathering time.

    .PARAMETER ServerBinary, ClientBinary
        PSObjects from Read-CcgsBinaryInfo (or fixtures) for the two binaries.

    .PARAMETER CargoEnv
        PSObject from Get-CcgsCargoEnvSnapshot.

    .PARAMETER LauncherScript
        Path of the .ps1 that triggered this call (Start-TwoClients.ps1 or
        Update-LatestMain.ps1). Repo-relative when available.

    .PARAMETER LastRebuild
        PSObject from Read-CcgsLastBuildProvenance, or $null when no sidecar
        exists. Surfaced verbatim in the output so consumers can compare
        running-binary HEAD vs source-tree HEAD.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$Context,
        [Parameter(Mandatory = $true)][datetime]$GeneratedAtUtc,
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [string]$RepoRootSource = '',
        [bool]$IsLauncherRoot = $false,
        [bool]$AutoSwitchedOrDedicated = $false,
        [Parameter(Mandatory = $true)][psobject]$Git,
        [Parameter(Mandatory = $true)][ValidateSet('debug','release')][string]$BuildProfile,
        [string[]]$BuildCommands = @(),
        [string]$TargetDir = '',
        [psobject]$ServerBinary = $null,
        [psobject]$ClientBinary = $null,
        [psobject]$CargoEnv = $null,
        [string]$LauncherScript = '',
        [psobject]$LastRebuild = $null
    )

    $utc = $GeneratedAtUtc.ToUniversalTime()

    $obj = [ordered]@{
        schema_version    = $script:CcgsBuildProvenanceSchemaVersion
        generated_at_utc  = $utc.ToString('o')
        context           = $Context
        repo              = [ordered]@{
            root_resolved              = $RepoRoot
            root_source                = $RepoRootSource
            is_launcher_root           = $IsLauncherRoot
            auto_switched_or_dedicated = $AutoSwitchedOrDedicated
        }
        git               = $Git
        build             = [ordered]@{
            profile        = $BuildProfile
            command_lines  = @($BuildCommands)
            target_dir     = $TargetDir
            binaries       = [ordered]@{
                server = $ServerBinary
                client = $ClientBinary
            }
            last_rebuild   = $LastRebuild
        }
        cargo_env         = $CargoEnv
        launcher          = [ordered]@{
            script_path = $LauncherScript
        }
    }
    return [pscustomobject]$obj
}

function ConvertTo-CcgsBuildProvenanceJson {
    <#
    .SYNOPSIS
        Consistent JSON formatter for build.json payloads. -Depth 8 is enough
        to cover the deepest leaf (build.binaries.server.modified_at).
    #>
    [CmdletBinding()]
    param([Parameter(Mandatory = $true, ValueFromPipeline = $true)][psobject]$Payload)
    process {
        return ($Payload | ConvertTo-Json -Depth 8)
    }
}

function Write-CcgsBuildProvenance {
    <#
    .SYNOPSIS
        Writes a build.json next to evidence logs. Returns the full path on
        success; returns $null and writes a host warning when writing fails
        (the caller must not block the launch on this).
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$EvidenceDir,
        [Parameter(Mandatory = $true)][psobject]$Payload,
        [string]$FileName = 'build.json'
    )
    if (-not (Test-Path $EvidenceDir)) {
        Write-Warning "build.json: evidence dir does not exist: $EvidenceDir -- skipped"
        return $null
    }
    $path = Join-Path $EvidenceDir $FileName
    try {
        $json = ConvertTo-CcgsBuildProvenanceJson -Payload $Payload
        # UTF-8 without BOM keeps the file diff-friendly across editors.
        $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
        [System.IO.File]::WriteAllText($path, $json, $utf8NoBom)
        return $path
    } catch {
        Write-Warning "build.json: failed to write '$path' -- $($_.Exception.Message)"
        return $null
    }
}

Export-ModuleMember -Function @(
    'New-CcgsBuildProvenance',
    'ConvertTo-CcgsBuildProvenanceJson',
    'Write-CcgsBuildProvenance',
    'Read-CcgsGitProvenance',
    'Read-CcgsBinaryInfo',
    'Get-CcgsCargoEnvSnapshot',
    'Read-CcgsLastBuildProvenance'
)
