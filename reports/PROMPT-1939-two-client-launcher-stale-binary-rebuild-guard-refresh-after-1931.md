# PROMPT 1939 — Two-Client Launcher Stale-Binary Rebuild Guard (Refresh after 1931)

**Branch:** `feat/1939-two-client-stale-guard`
**Base:** origin/main @ 79031021 (after PROMPT 1931)
**Status:** SHIPPED

## Context

`origin/feat/1915-two-client-stale-guard` shipped the staleness-guard feature
but was not fast-forwardable onto main after PROMPT 1931 landed reports and
tests. A direct cherry-pick would have deleted those recent additions. This
prompt re-applies only the launcher change on the current main.

## Changes Applied

### `tools/dev-launcher/Start-TwoClients.ps1`

Recovered the exact 1915 payload by inspecting the old branch with `git show`.
No wholesale merge or cherry-pick was performed.

**New parameters added:**
| Parameter | Purpose |
|---|---|
| `-ForceRebuild` | Always rebuild server and client before launch |
| `-SkipStalenessCheck` | Skip mtime scan; only rebuild when binary is absent |

**New helper functions added:**
| Function | Purpose |
|---|---|
| `Get-NewestSourceTime` | Returns newest mtime among `*.rs`, `Cargo.toml`, `Cargo.lock` under play root |
| `Test-BinaryStale` | Returns `$true` when binary is absent or older than newest source |

**Section 4 behaviour change** (was "Build if missing", now "Build if missing or stale"):
- Default: staleness check runs automatically; stale binaries trigger rebuild with
  a lag-in-seconds diagnostic message
- `-ForceRebuild`: skips mtime check entirely and rebuilds unconditionally
- `-SkipStalenessCheck`: skips mtime check; preserves previous behaviour
- Post-build existence guard: if `cargo` exits 0 but the binary is still absent,
  the launcher aborts with a diagnostic rather than starting nothing

**Help text** updated to document the two new flags.

**Header comment** step 4 updated: "Builds server + client if the binaries are
missing or stale (newer source files exist)."

### `reports/PROMPT-1915-two-client-launcher-stale-binary-rebuild-guard-after-1912.md`

Backfilled the original 1915 report which was never mainlined.

### `reports/PROMPT-1939-two-client-launcher-stale-binary-rebuild-guard-refresh-after-1931.md`

This file.

## Validation

| Check | Result |
|---|---|
| PowerShell parser (`[Parser]::ParseFile`) | PASS |
| `git diff --check` | PASS — no trailing whitespace |
| `git diff --name-status origin/main..HEAD` | Only owned files: `tools/dev-launcher/Start-TwoClients.ps1` + 2 reports |
| No deletions of recent reports/tests | PASS — no existing files deleted |
| Forbidden files untouched | PASS — `Start-AutoplayVsBot.ps1`, `tools/autoplay/**`, `client/src/autoplay.rs`, `production/**`, `tests/**` all clean |

## Files Changed

```
M  tools/dev-launcher/Start-TwoClients.ps1
A  reports/PROMPT-1915-two-client-launcher-stale-binary-rebuild-guard-after-1912.md
A  reports/PROMPT-1939-two-client-launcher-stale-binary-rebuild-guard-refresh-after-1931.md
```

1939: TWO-CLIENT-LAUNCHER-STALE-BINARY-REBUILD-GUARD-REFRESH-AFTER-1931: SHIPPED
