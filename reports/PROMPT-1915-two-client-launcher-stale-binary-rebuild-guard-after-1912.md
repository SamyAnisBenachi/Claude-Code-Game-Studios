# PROMPT 1915 — Two-Client Launcher Stale-Binary Rebuild Guard

**Branch:** `feat/1915-two-client-stale-guard`
**Base:** origin/main after PROMPT 1912
**Status:** Originally shipped on branch; not fast-forwardable onto later main (re-applied as PROMPT 1939)

## Summary

Added source-vs-binary staleness detection to `Start-TwoClients.ps1` so the
launcher automatically rebuilds stale binaries rather than silently launching
an outdated build.

## Changes

### `tools/dev-launcher/Start-TwoClients.ps1`

**New parameters:**
- `-ForceRebuild` — always rebuild both binaries before launch
- `-SkipStalenessCheck` — skip mtime comparison; only rebuild if binary is absent

**New helper functions:**
- `Get-NewestSourceTime` — scans `*.rs`, `Cargo.toml`, `Cargo.lock` under the
  play root and returns the newest `LastWriteTimeUtc`
- `Test-BinaryStale` — returns `$true` when the binary mtime is older than the
  newest source file, or when the binary is missing

**Section 4 logic (Build if missing or stale):**
- Default path: runs staleness check and rebuilds any binary whose mtime is
  behind the newest source file; prints lag in seconds
- `-ForceRebuild`: marks both binaries for rebuild unconditionally
- `-SkipStalenessCheck`: skips mtime scan; only rebuilds missing binaries
- Post-build existence guard: fails with a clear error if cargo exits 0 but
  the binary is still absent (e.g. wrong `CARGO_TARGET_DIR`)

**Improved error messages** on build failure include the cargo exit code and
explicitly say "Aborting launch to prevent running a stale binary."

## Validation

- PowerShell parser: PASS (`[Parser]::ParseFile` — no parse errors)
- `git diff --check`: PASS (no trailing whitespace)
- Diff scope: only `tools/dev-launcher/Start-TwoClients.ps1` modified

## Notes

This report was backfilled when PROMPT 1939 re-applied this change onto a
later main because the original `feat/1915-two-client-stale-guard` branch was
not fast-forwardable after PROMPT 1931 mainlined additional changes.
