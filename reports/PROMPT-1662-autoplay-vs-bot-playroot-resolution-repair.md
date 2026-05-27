# PROMPT 1662 — AUTOPLAY-VS-BOT-PLAYROOT-RESOLUTION-REPAIR

**Date:** 2026-05-27  
**Branch:** `fix/1662-autoplay-vsbot-playroot-resolution`  
**Worktree:** `D:\_DEV\Work\ccgs-wt-1662-playroot-repair`  
**Source-of-truth at start:** `origin/main@9fa54ea7`

---

## Summary

**SHIPPED.** One-line logic fix to `tools/dev-launcher/Start-AutoplayVsBot.ps1` makes the
default play-root selection require a `Cargo.toml` at `D:\_DEV\ccgs-play-main` before
accepting it as the play root. When the stub directory exists but contains no workspace,
the script now falls back to the launcher root (the actual repo) and emits a clear warning
instead of blocking with exit 11.

---

## Root Cause (from PROMPT 1660 precheck)

`D:\_DEV\ccgs-play-main` is a stub directory containing only `production/qa/` evidence
artifacts. It was presumably created as a target for composite-run evidence, not as a
full Rust workspace checkout.

`Start-AutoplayVsBot.ps1` previously checked:

```powershell
} elseif (Test-Path $DefaultPlayRoot) {
    $PlayRoot = $DefaultPlayRoot   # selected stub — WRONG
```

This selected the stub, then immediately failed at the Cargo.toml guard:

```
BLOCKED-PRECONDITION: No Cargo.toml at D:\_DEV\ccgs-play-main
exit 11
```

---

## Fix Applied

**File:** `tools/dev-launcher/Start-AutoplayVsBot.ps1`

**Before (line 157):**
```powershell
} elseif (Test-Path $DefaultPlayRoot) {
    $PlayRoot       = $DefaultPlayRoot
    $PlayRootSource = 'documented dedicated default'
}
```

**After:**
```powershell
} elseif ((Test-Path $DefaultPlayRoot) -and (Test-Path (Join-Path $DefaultPlayRoot 'Cargo.toml'))) {
    $PlayRoot       = $DefaultPlayRoot
    $PlayRootSource = 'documented dedicated default'
} elseif (Test-Path $DefaultPlayRoot) {
    # Directory exists but has no Cargo.toml — treat as a stub/evidence-only directory and fall back.
    $PlayRoot       = $LauncherRoot
    $PlayRootSource = 'launcher root (default play root is a stub without Cargo.toml)'
    Write-Warning "'$DefaultPlayRoot' exists but contains no Cargo.toml. Ignoring stub and falling back to launcher root. Set CCGS_PLAY_REPO_ROOT or -PlayRepoRoot to suppress this warning."
}
```

The existing final `else` branch (no directory at all → launcher root) is preserved unchanged.

---

## Validation Results

### 1. PowerShell Syntax Check

```
[System.Management.Automation.Language.Parser]::ParseFile(...)
PARSE OK: no syntax errors
```

### 2. git diff --check

```
git diff --check: PASS
```
No trailing whitespace or line-ending issues.

### 3. Dry-run without -PlayRepoRoot (the previously-broken case)

**Command:**
```powershell
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -DryRun -Recipe full-game
```

**Before fix output:**
```
[DRY RUN] would block in a real run: no Cargo.toml at D:\_DEV\ccgs-play-main
```
(Exit 11 in real run)

**After fix output:**
```
WARNING: 'D:\_DEV\ccgs-play-main' exists but contains no Cargo.toml. Ignoring stub
and falling back to launcher root. Set CCGS_PLAY_REPO_ROOT or -PlayRepoRoot to suppress this warning.

==== Roots ====
Launcher repo root: D:\_DEV\Work\ccgs-wt-1662-playroot-repair
Play/build root:    D:\_DEV\Work\ccgs-wt-1662-playroot-repair  (source: launcher root (default play root is a stub without Cargo.toml))

[...all sections reached, DRY RUN COMPLETE, exit 0]
```

No exit 11. Script reaches all sections and exits 0.

### 4. Dry-run with -PlayRepoRoot override

**Command:**
```powershell
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -DryRun -Recipe full-game -PlayRepoRoot "D:\_DEV\Work\Claude-Code-Game-Studios"
```

**Output:**
```
Play/build root:    D:\_DEV\Work\Claude-Code-Game-Studios  (source: -PlayRepoRoot argument)
```

Explicit `-PlayRepoRoot` still wins over all fallback paths. Evidence lands correctly under the repo's `production/qa/evidence/` tree.

---

## Behavior Summary (Before / After)

| Scenario | Before | After |
|---|---|---|
| Default (no param/env, stub exists) | Exit 11 — BLOCKED-PRECONDITION | Warning + fallback to launcher root; exits 0 in DryRun |
| Default (stub has Cargo.toml — future valid checkout) | Would select stub (correct) | Selects stub (unchanged) |
| Default (no `D:\_DEV\ccgs-play-main` at all) | Falls back to launcher root | Falls back to launcher root (unchanged) |
| `-PlayRepoRoot` explicit | Wins (correct) | Wins (unchanged) |
| `$env:CCGS_PLAY_REPO_ROOT` env var | Wins (correct) | Wins (unchanged) |

---

## Files Changed

| File | Change |
|---|---|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | Add Cargo.toml validation to default-path guard; stub fallback with warning |
| `reports/PROMPT-1662-autoplay-vs-bot-playroot-resolution-repair.md` | This report |

---

## Not Changed (out of scope)

- `docs/autoplay/autoplay-vs-bot-flow.md` — operator commands in the docs already show `-PlayRepoRoot` as the recommended form; no doc update required.
- `tools/dev-launcher-app/src/main.rs` — no wiring change needed.
- Exit codes, interactive-session guard, port selection, soak/smoke delegation — all preserved exactly.

---

## Live GUI Gate

Still HUMAN-GATE (by design). This fix unblocks the tooling so a real run can proceed.
A full live run with operator sign-off is required to close GAP-01/GAP-02 per
AUTOPLAY-VS-BOT-QA-001 AC5.

---

1662: AUTOPLAY-VS-BOT-PLAYROOT-RESOLUTION-REPAIR: SHIPPED
