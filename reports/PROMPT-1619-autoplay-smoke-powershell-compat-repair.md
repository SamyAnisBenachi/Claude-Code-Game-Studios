# PROMPT 1619 — Autoplay Smoke PowerShell Compatibility Repair

**Date**: 2026-05-26  
**Branch**: `work/autoplay-smoke-powershell-compat-1619`  
**Commit**: `434d93a0`  
**Base**: `origin/main@37306162`  
**Status**: SHIPPED

---

## Problem

`tools/autoplay/Run-AutoplaySmoke.ps1` used `Get-Date -AsUTC` in four places.
The `-AsUTC` flag was introduced in PowerShell 7.1 and does not exist in
Windows PowerShell 5.1, which is the target machine's shell. This caused an
immediate parameter-not-found error when launching the smoke runner on PS5.1.

Affected lines (pre-repair):

| Line | Expression | Issue |
|------|-----------|-------|
| 37 | `(Get-Date -AsUTC -Format "yyyyMMdd-HHmmss") + "-Z"` | PS7+ only |
| 61 | `(Get-Date -AsUTC).ToString("o")` | PS7+ only |
| 90 | `(Get-Date -AsUTC).ToString("o")` | PS7+ only |
| 123 | `(Get-Date -AsUTC).ToString("o")` | PS7+ only |

---

## Fix

Replaced all four call sites with `[DateTime]::UtcNow`, which:
- Works in Windows PowerShell 5.1, PowerShell 6, and PowerShell 7+
- Returns a `DateTime` with `Kind = Utc`, identical semantic to `-AsUTC`
- `.ToString("o")` on a UTC DateTime produces the same ISO-8601 round-trip
  string as the original expression
- `.ToString("yyyyMMdd-HHmmss")` produces the same directory stamp format

Also updated the usage comment block to document both `powershell` and `pwsh`
invocations, making it clear the script is compatible with both versions.

No other files were changed. No Cargo, client, server, or shared code was
touched.

---

## Validation

| Check | Result |
|-------|--------|
| `git diff --check` | PASS (0 whitespace errors) |
| PowerShell 5.1 AST parse (`Parser::ParseFile`) | PASS — 0 syntax errors |
| PowerShell 7 parse | SKIP — not installed on this machine |
| Grep for remaining `-AsUTC` | PASS — 0 occurrences in file |
| Path allowlist (only `tools/autoplay/Run-AutoplaySmoke.ps1`) | PASS |
| Bevy GUI not launched | PASS |

PowerShell 5.1 parse command used:
```powershell
[System.Management.Automation.Language.Parser]::ParseFile(
    'D:/Tmp/wt-1619/tools/autoplay/Run-AutoplaySmoke.ps1',
    [ref]$tokens,
    [ref]$errors
)
# Result: 0 errors
```

---

## Integration Readiness

Ready for fast-forward integration onto `origin/main`. The diff is a 7-line
purely-textual change to one tooling script with no Rust code impact.

**Test steps after integration:**
1. On a Windows PowerShell 5.1 machine, from repo root:  
   `powershell -ExecutionPolicy Bypass -File tools/autoplay/Run-AutoplaySmoke.ps1`
2. Confirm the artifact directory is created under  
   `production/qa/evidence/autoplay-runs/YYYYMMDD-HHMMSS-Z/`
3. Confirm the cargo build and RPC wait proceed without "parameter not found" errors.

---

1619: AUTOPLAY-SMOKE-POWERSHELL-COMPAT-REPAIR: SHIPPED
