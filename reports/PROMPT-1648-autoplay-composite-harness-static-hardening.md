# PROMPT 1648 — AUTOPLAY-COMPOSITE-HARNESS-STATIC-HARDENING

**Date**: 2026-05-27  
**Branch**: work/autoplay-composite-harness-hardening-1648  
**Commit**: ccf5b73d  
**Source**: origin/main@178a8471

---

## Validation Results

| Check | Result |
|---|---|
| `git diff --check` | PASS (no whitespace errors) |
| `-Help` | PASS (updated exit-11 text rendered correctly) |
| `-DryRun` | PASS (all sections complete, correct DryRun qualifiers) |

---

## Static Findings

### BUG-1 — Exit-code 11 comment listed only one of three trigger conditions  
**Location**: header comment line 50; `Show-Help` BLOCKED EXIT CODES block line 104  
**Severity**: Low (misleading to operators debugging a BLOCKED-11)  
**Detail**: Exit 11 fires for three distinct conditions:  
1. No `Cargo.toml` at the resolved play root (section 1)  
2. `Run-AutoplaySmoke.ps1` not found (section 3)  
3. `-SkipSoakLaunch` not set and `Start-BotVsBotSoak.ps1` missing (section 3)  
The original comment only named condition 3.  
**Fix**: Both the file-header comment and the `-Help` output now enumerate all three.

### BUG-2 — DryRun smoke preview omitted `-Python` and `-ClientStartupSecs`  
**Location**: line 321 (original)  
**Severity**: Low (operator sees an incomplete preview of the launch command)  
**Detail**: The live path passes 7 args to `Run-AutoplaySmoke.ps1`; the DryRun message
only echoed 5, silently dropping `-Python` and `-ClientStartupSecs`.  
**Fix**: DryRun message now mirrors the full argument list exactly.

### BUG-3 — DryRun composite outcome printed the live-run success message  
**Location**: lines 368–370 (original)  
**Severity**: Low (confusing — "Composite run COMPLETE" appeared even when nothing ran)  
**Detail**: Section 9 outcome branch was not guarded by `$DryRun`. The green
"Composite run COMPLETE" message was identical in dry-run and live-run, making it
impossible to distinguish a simulated result from a real one.  
**Fix**: When `$DryRun` is true the message reads
`[DRY RUN] Simulated outcome: COMPLETE (no processes launched; exit=0 assumed).`

### NOTE-1 — Cosmetic: two section headers for smoke in DryRun  
**Location**: lines 285 + 320 (original numbering)  
**Status**: Noted only; not repaired (harmless cosmetic issue, out of task scope)  
**Detail**: `Write-Section "Autoplay smoke (recipe=$Recipe)"` runs unconditionally,
then the DryRun branch emits a second `Write-Section "Autoplay smoke (DRY RUN -- skipped)"`.
This produces two `====` headers for one conceptual section. Not a reliability concern.

### NOTE-2 — Outcome switch maps exit codes 10/11/12 from smoke child  
**Location**: lines 335–342  
**Status**: Noted only; no change made  
**Detail**: `$smokeExit` switch includes cases for 10/11/12 but those codes are only
reachable if `Run-AutoplaySmoke.ps1` itself returns them. At the point this switch
runs, the composite harness has already exited 10/11/12 directly on its own BLOCKED
conditions. The mapping is dead code for those codes but harmless and defensive.

---

## Changes Made

**File**: `tools/dev-launcher/Start-AutoplayVsBot.ps1`  
**Diff summary** (11 insertions, 5 deletions):

1. Header comment exit-11 line → expanded to list all 3 conditions.
2. `Show-Help` exit-11 line → expanded to list all 3 conditions.
3. DryRun smoke message → added `-Python $Python -ClientStartupSecs $ClientStartupSecs`.
4. Section-9 outcome branch → added `if ($DryRun)` guard with distinct DryRun message.

No Rust source, no sprint/session-state, no docs/autoplay.md touched.

---

## Post-Fix Validation

```
-Help  : PASS — exit-11 description now shows all 3 conditions.
-DryRun: PASS — smoke preview now shows full 7-arg command.
           PASS — outcome line reads "[DRY RUN] Simulated outcome: COMPLETE".
           PASS — no misleading "Composite run COMPLETE" in dry-run.
git diff --check: PASS
```

---

1648: AUTOPLAY-COMPOSITE-HARNESS-STATIC-HARDENING: SHIPPED
