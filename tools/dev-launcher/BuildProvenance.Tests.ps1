# BuildProvenance.Tests.ps1 -- Pester tests for tools/dev-launcher/BuildProvenance.psm1.
#
# PROMPT 1402: covers the pure builder (New-CcgsBuildProvenance), JSON
# serialization round-trip, the file writer (Write-CcgsBuildProvenance), and
# the discovery helpers (Read-CcgsBinaryInfo, Read-CcgsLastBuildProvenance,
# Get-CcgsCargoEnvSnapshot). All inputs are mocked or built in a temp dir;
# no actual cargo invocation, no actual repo access is performed.
#
# Compatible with Pester 3.4.0 (shipped with Windows PowerShell 5.1).
#
# Run:
#   powershell -ExecutionPolicy Bypass -Command "Invoke-Pester -Path tools\dev-launcher\BuildProvenance.Tests.ps1"

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$module = Join-Path $here 'BuildProvenance.psm1'

Describe 'BuildProvenance helper' {
    BeforeEach {
        Get-Module BuildProvenance -ErrorAction SilentlyContinue | Remove-Module -Force
        Import-Module $module -Force
    }

    Context 'New-CcgsBuildProvenance pure builder' {
        It 'returns the documented schema_version' {
            $git = [pscustomobject]@{
                path = 'D:\fake'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main...origin/main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $obj = New-CcgsBuildProvenance -Context 'launch' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\fake' -Git $git -BuildProfile 'debug'
            $obj.schema_version | Should Be 1
        }

        It 'serializes generated_at_utc to ISO 8601 with offset' {
            $git = [pscustomobject]@{
                path = 'D:\fake'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main...origin/main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $when = [datetime]::Parse('2026-05-19T12:34:56Z').ToUniversalTime()
            $obj = New-CcgsBuildProvenance -Context 'launch' -GeneratedAtUtc $when -RepoRoot 'D:\r' -Git $git -BuildProfile 'debug'
            $obj.generated_at_utc | Should Match '^2026-05-19T12:34:56'
        }

        It 'propagates all top-level required fields' {
            $git = [pscustomobject]@{
                path = 'D:\fake'; branch = 'work/foo'; head_sha = 'b' * 40; head_short = 'b' * 12;
                head_subject = 'subj'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = "## work/foo`n M file.rs"; is_clean = $false; dirty_files = @(' M file.rs'); error = $null
            }
            $srv = [pscustomobject]@{ path = 'D:\srv.exe'; exists = $true; size_bytes = 100; modified_at = '2026-05-19T00:00:00Z' }
            $cli = [pscustomobject]@{ path = 'D:\cli.exe'; exists = $true; size_bytes = 200; modified_at = '2026-05-19T00:00:00Z' }
            $env = [pscustomobject]@{ CARGO_TARGET_DIR = 'D:\t'; RUSTFLAGS = '-C debuginfo=0' }
            $obj = New-CcgsBuildProvenance -Context 'rebuild' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T00:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\repo' -RepoRootSource 'dedicated default' `
                -IsLauncherRoot $false -AutoSwitchedOrDedicated $true `
                -Git $git -BuildProfile 'debug' `
                -BuildCommands @('cargo build -p server','cargo build -p client --bin client') `
                -TargetDir 'D:\t' -ServerBinary $srv -ClientBinary $cli -CargoEnv $env `
                -LauncherScript 'tools/dev-launcher/Update-LatestMain.ps1' `
                -LastRebuild $null

            $obj.context                           | Should Be 'rebuild'
            $obj.repo.root_resolved                | Should Be 'D:\repo'
            $obj.repo.root_source                  | Should Be 'dedicated default'
            $obj.repo.is_launcher_root             | Should Be $false
            $obj.repo.auto_switched_or_dedicated   | Should Be $true
            $obj.git.branch                        | Should Be 'work/foo'
            $obj.git.head_sha                      | Should Be ('b' * 40)
            $obj.git.is_clean                      | Should Be $false
            $obj.build.profile                     | Should Be 'debug'
            $obj.build.target_dir                  | Should Be 'D:\t'
            $obj.build.command_lines.Count         | Should Be 2
            $obj.build.command_lines[0]            | Should Be 'cargo build -p server'
            $obj.build.binaries.server.size_bytes  | Should Be 100
            $obj.build.binaries.client.size_bytes  | Should Be 200
            $obj.build.last_rebuild                | Should BeNullOrEmpty
            $obj.cargo_env.CARGO_TARGET_DIR        | Should Be 'D:\t'
            $obj.launcher.script_path              | Should Be 'tools/dev-launcher/Update-LatestMain.ps1'
        }

        It 'embeds last_rebuild verbatim when provided' {
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'c' * 40; head_short = 'c' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $rebuildObj = [pscustomobject]@{
                generated_at_utc = '2026-05-19T11:00:00.0000000Z'
                git              = [pscustomobject]@{ branch = 'main'; head_short = 'deadbee'; head_sha = 'deadbee' * 5 + 'd' * 5 }
            }
            $obj = New-CcgsBuildProvenance -Context 'launch' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'debug' -LastRebuild $rebuildObj
            $obj.build.last_rebuild.git.head_short | Should Be 'deadbee'
            $obj.build.last_rebuild.generated_at_utc | Should Be '2026-05-19T11:00:00.0000000Z'
        }

        It 'rejects invalid build profile' {
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            { New-CcgsBuildProvenance -Context 'launch' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'nightly' } | Should Throw
        }
    }

    Context 'ConvertTo-CcgsBuildProvenanceJson + Write-CcgsBuildProvenance' {
        $tempDir = $null

        BeforeEach {
            $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("ccgs-buildprov-{0}" -f ([guid]::NewGuid().ToString('N')))
            New-Item -ItemType Directory -Path $tempDir | Out-Null
        }
        AfterEach {
            if ($tempDir -and (Test-Path $tempDir)) {
                Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }

        It 'writes build.json into the evidence dir and returns its path' {
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $obj = New-CcgsBuildProvenance -Context 'launch' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'debug'
            $written = Write-CcgsBuildProvenance -EvidenceDir $tempDir -Payload $obj
            $written | Should Not BeNullOrEmpty
            (Test-Path $written) | Should Be $true
            # Round-trip JSON to confirm shape survives serialization.
            $raw = Get-Content -LiteralPath $written -Raw
            $parsed = $raw | ConvertFrom-Json
            $parsed.schema_version  | Should Be 1
            $parsed.context         | Should Be 'launch'
            $parsed.git.branch      | Should Be 'main'
            $parsed.build.profile   | Should Be 'debug'
        }

        It 'writes UTF-8 without BOM' {
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $obj = New-CcgsBuildProvenance -Context 'launch' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'debug'
            $written = Write-CcgsBuildProvenance -EvidenceDir $tempDir -Payload $obj
            $bytes = [System.IO.File]::ReadAllBytes($written) | Select-Object -First 3
            $hasBom = ($bytes.Count -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
            $hasBom | Should Be $false
        }

        It 'returns $null and warns when evidence dir is missing' {
            $missing = Join-Path $tempDir 'does-not-exist'
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $obj = New-CcgsBuildProvenance -Context 'launch' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'debug'
            $written = Write-CcgsBuildProvenance -EvidenceDir $missing -Payload $obj -WarningAction SilentlyContinue
            $written | Should BeNullOrEmpty
        }

        It 'accepts a custom file name (sidecar)' {
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $obj = New-CcgsBuildProvenance -Context 'rebuild' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'release'
            $written = Write-CcgsBuildProvenance -EvidenceDir $tempDir -Payload $obj -FileName 'last-build-provenance.json'
            (Split-Path -Leaf $written) | Should Be 'last-build-provenance.json'
        }
    }

    Context 'Read-CcgsBinaryInfo' {
        $tempDir = $null
        BeforeEach {
            $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("ccgs-buildprov-bin-{0}" -f ([guid]::NewGuid().ToString('N')))
            New-Item -ItemType Directory -Path $tempDir | Out-Null
        }
        AfterEach {
            if ($tempDir -and (Test-Path $tempDir)) {
                Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }

        It 'reports a real file with non-null size and mtime' {
            $f = Join-Path $tempDir 'server.exe'
            Set-Content -LiteralPath $f -Value 'fake-bytes' -NoNewline
            $info = Read-CcgsBinaryInfo -Path $f
            $info.exists      | Should Be $true
            $info.size_bytes  | Should BeGreaterThan 0
            $info.modified_at | Should Not BeNullOrEmpty
        }

        It 'reports exists=$false for a missing file without throwing' {
            $f = Join-Path $tempDir 'no-such.exe'
            $info = Read-CcgsBinaryInfo -Path $f
            $info.exists      | Should Be $false
            $info.size_bytes  | Should BeNullOrEmpty
            $info.modified_at | Should BeNullOrEmpty
        }
    }

    Context 'Read-CcgsLastBuildProvenance' {
        $tempDir = $null
        BeforeEach {
            $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("ccgs-buildprov-side-{0}" -f ([guid]::NewGuid().ToString('N')))
            New-Item -ItemType Directory -Path $tempDir | Out-Null
        }
        AfterEach {
            if ($tempDir -and (Test-Path $tempDir)) {
                Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }

        It 'returns $null when the sidecar is missing' {
            (Read-CcgsLastBuildProvenance -TargetProfileDir $tempDir) | Should BeNullOrEmpty
        }

        It 'returns $null on malformed JSON without throwing' {
            $bad = Join-Path $tempDir 'last-build-provenance.json'
            Set-Content -LiteralPath $bad -Value '{not json'
            (Read-CcgsLastBuildProvenance -TargetProfileDir $tempDir) | Should BeNullOrEmpty
        }

        It 'parses a valid sidecar round-trip' {
            $git = [pscustomobject]@{
                path = 'D:\r'; branch = 'main'; head_sha = 'a' * 40; head_short = 'a' * 12;
                head_subject = 's'; head_committed_at = '2026-05-19T00:00:00Z';
                status_short_branch = '## main'; is_clean = $true; dirty_files = @(); error = $null
            }
            $obj = New-CcgsBuildProvenance -Context 'rebuild' `
                -GeneratedAtUtc ([datetime]::Parse('2026-05-19T12:00:00Z').ToUniversalTime()) `
                -RepoRoot 'D:\r' -Git $git -BuildProfile 'debug'
            Write-CcgsBuildProvenance -EvidenceDir $tempDir -Payload $obj -FileName 'last-build-provenance.json' | Out-Null

            $parsed = Read-CcgsLastBuildProvenance -TargetProfileDir $tempDir
            $parsed                | Should Not BeNullOrEmpty
            $parsed.context        | Should Be 'rebuild'
            $parsed.schema_version | Should Be 1
        }
    }

    Context 'Get-CcgsCargoEnvSnapshot' {
        It 'always returns the documented key set, with $null for unset keys' {
            # Snapshot + temporarily clear a well-known key.
            $key = 'CARGO_TARGET_DIR'
            $orig = [System.Environment]::GetEnvironmentVariable($key)
            try {
                [System.Environment]::SetEnvironmentVariable($key, $null)
                $snap = Get-CcgsCargoEnvSnapshot
                $snap.PSObject.Properties.Name -contains $key | Should Be $true
                $snap.$key                                    | Should BeNullOrEmpty
                $snap.PSObject.Properties.Name -contains 'RUSTFLAGS' | Should Be $true
            } finally {
                if ($null -ne $orig) {
                    [System.Environment]::SetEnvironmentVariable($key, $orig)
                }
            }
        }

        It 'surfaces a set env var verbatim' {
            $key = 'CCGS_PLAY_REPO_ROOT'
            $orig = [System.Environment]::GetEnvironmentVariable($key)
            try {
                [System.Environment]::SetEnvironmentVariable($key, 'D:\fake\play')
                $snap = Get-CcgsCargoEnvSnapshot
                $snap.$key | Should Be 'D:\fake\play'
            } finally {
                [System.Environment]::SetEnvironmentVariable($key, $orig)
            }
        }
    }

    Context 'Read-CcgsGitProvenance error tolerance' {
        It 'returns an error string for a non-existent path' {
            $g = Read-CcgsGitProvenance -Path 'D:\definitely-not-a-real-path-3xqcv'
            $g.error | Should Not BeNullOrEmpty
            $g.branch | Should BeNullOrEmpty
        }

        It 'returns an error string for an existing non-git path' {
            $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("ccgs-buildprov-notgit-{0}" -f ([guid]::NewGuid().ToString('N')))
            New-Item -ItemType Directory -Path $tempDir | Out-Null
            try {
                $g = Read-CcgsGitProvenance -Path $tempDir
                $g.error | Should Not BeNullOrEmpty
                $g.branch | Should BeNullOrEmpty
            } finally {
                Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    }
}
