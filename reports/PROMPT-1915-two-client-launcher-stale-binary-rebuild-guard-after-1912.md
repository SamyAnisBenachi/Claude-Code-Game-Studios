# PROMPT 1915 — Two-Client Launcher Stale-Binary Rebuild Guard

**Date:** 2026-05-28  
**Worktree:** `D:\tmp\wt-1915-two-client-stale-guard`  
**Branch:** `feat/1915-two-client-stale-guard`  
**Commit:** `8d3615eb`

## Problem

`Start-TwoClients.ps1` section 4 only rebuilt when binaries were **missing**. After
source changes (e.g. a Lightyear protocol type rename) the script would silently
launch stale `server.exe` + `client.exe`, causing protocol mismatches. PROMPT 1883
hit this exact failure: the two-client retest was PARTIAL because the binaries
predated recent source changes.

## Changes Made

**File:** `tools/dev-launcher/Start-TwoClients.ps1` — section 4 "Binary check"

### New parameters

| Param | Default | Effect |
|---|---|---|
| `-ForceRebuild` | off | Always rebuild both binaries before launch |
| `-SkipStalenessCheck` | off | Bypass source-vs-binary comparison (missing binaries still rebuild) |

### Staleness check logic

1. Scans `*.rs`, `Cargo.toml`, `Cargo.lock` under `$RepoRoot` (recursive) for the
   newest `LastWriteTimeUtc`.
2. Compares each binary's mtime against that timestamp.
3. If the binary is older, sets `$needServer`/`$needClient = $true` and prints a
   yellow warning with the lag in seconds.
4. If no source files are found, staleness check is skipped with a warning.

### Rebuild failure guard

Existing `exit 1` on non-zero `cargo` exit code was kept. Added an additional
**post-build existence guard**: if cargo exits 0 but the binary path is still absent
(misconfigured `CARGO_TARGET_DIR`), the script exits 1 with a clear diagnostic
rather than attempting to launch a missing binary.

### Preserved behavior

- All existing params (`-Port`, `-StrictPort`, `-Release`, `-DryRun`, etc.) unchanged.
- Operator output style unchanged — same `==== Section ====` headers.
- `-DryRun` prints what would be built but does not invoke cargo.
- Missing-binary path still works as before.
- BuildProvenance module integration untouched.

## Validation

### Static / parser check
```
PowerShell syntax OK
```
(`[System.Management.Automation.Language.Parser]::ParseFile` — zero parse errors)

### `git diff --check`
No trailing-whitespace issues.

### Dry-run executions

**Normal staleness scan** (`-DryRun -PlayRepoRoot ...`):
```
==== Binary check ====
Staleness check: scanning source files under D:/_DEV/Work/Claude-Code-Game-Studios ...
Newest source file mtime (UTC): 2026-05-28 05:53:09
server.exe is STALE (binary 16783s behind newest source) -- will rebuild.
client.exe is STALE (binary 578821s behind newest source) -- will rebuild.
Rebuilding server and client...
cargo build -p server
cargo build -p client --bin client
```

**ForceRebuild** (`-DryRun -ForceRebuild`):
```
-ForceRebuild specified -- marking server and client for rebuild.
Rebuilding server and client...
```

**SkipStalenessCheck** (`-DryRun -SkipStalenessCheck`):
```
-SkipStalenessCheck specified -- skipping source-vs-binary staleness check.
```

## Scope compliance

- Edited only: `tools/dev-launcher/Start-TwoClients.ps1`
- All forbidden files untouched: `Start-AutoplayVsBot.ps1`, `autoplay/**`,
  `client/src/**`, `server/src/**`, `production/**`

## Path allowlist review

All file access in the new code:
- `Get-ChildItem -Path $Root ...` — reads from `$RepoRoot` (the play/build root, already trusted)
- `Get-Item $BinPath` — reads from `$env:CARGO_TARGET_DIR` (already used throughout)
- No new network, registry, or external tool calls

## Deferred

- Heavy compile/runtime verification (actual `cargo build` + two-client launch + Lightyear
  handshake check) deferred to a follow-up VERIFY prompt as per task instructions.

---

1915: TWO-CLIENT-LAUNCHER-STALE-BINARY-REBUILD-GUARD-AFTER-1912: SHIPPED
