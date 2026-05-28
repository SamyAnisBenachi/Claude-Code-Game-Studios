# PROMPT 1814 — AUTOPLAY-COMPOSITE-LAUNCHER-STALE-PYC-GUARD

**Date:** 2026-05-28
**Branch:** `fix/1814-stale-pyc-guard`
**Commit:** `4bf70c94`

---

## Audit Findings

### Existing guard (PROMPT 1802)

`tools/autoplay/Run-AutoplaySmoke.ps1` (lines 111–124) already has a full
stale-pyc guard:

- Clears `tools/autoplay/__pycache__` and `tools/autoplay/recipes/__pycache__`
- Sets `$env:PYTHONDONTWRITEBYTECODE = '1'`
- Passes `-B` to the Python driver invocation

This guard runs correctly in the smoke-only path.

### Gap in the composite path

`tools/dev-launcher/Start-AutoplayVsBot.ps1` — the vs-bot composite launcher —
had **no explicit stale-pyc guard section of its own**.  It delegates Python
execution to `Run-AutoplaySmoke.ps1` via a `powershell.exe` subprocess, so the
inner guard would still fire, but:

1. The composite launcher's own console output showed no `[stale-pyc-guard]`
   evidence — future live-verify reports scanning composite output could not
   confirm the guard ran.
2. `PYTHONDONTWRITEBYTECODE=1` was never set in the composite launcher's own
   process environment before the subprocess was started.
3. Any future Python invocation added at the composite layer would be unguarded.

---

## Change Made

**File:** `tools/dev-launcher/Start-AutoplayVsBot.ps1`

Added new section **6b — Stale-pyc guard** immediately before step 7
(Run autoplay smoke launcher), spanning 25 lines:

```powershell
# ---- 6b. Stale-pyc guard (PROMPT 1814) --------------------------------------
Write-Section "Stale-pyc guard"
$autoplayToolsDir   = Join-Path $LauncherRoot 'tools\autoplay'
$pycDirs = @(
    (Join-Path $autoplayToolsDir '__pycache__'),
    (Join-Path $autoplayToolsDir 'recipes\__pycache__')
)
foreach ($pycDir in $pycDirs) {
    if (Test-Path $pycDir) {
        if (-not $DryRun) {
            Remove-Item -Recurse -Force $pycDir -ErrorAction SilentlyContinue
            Write-Host "[stale-pyc-guard] cleared: $pycDir"
        } else {
            Write-Host "[DRY RUN] would clear stale pyc: $pycDir"
        }
    } else {
        Write-Host "[stale-pyc-guard] not present (skip): $pycDir"
    }
}
$env:PYTHONDONTWRITEBYTECODE = '1'
Write-Host "[stale-pyc-guard] PYTHONDONTWRITEBYTECODE=1 (composite stale-pyc guard active)"
```

### Command-line semantics: before vs after

| Aspect | Before | After |
|--------|--------|-------|
| `__pycache__` cleared at composite layer | No | Yes (via section 6b) |
| `PYTHONDONTWRITEBYTECODE` set in composite process | No | Yes |
| `-DryRun` support | n/a | Section 6b skips the Remove-Item and logs `[DRY RUN]` |
| Guard log line in composite output | Absent | `[stale-pyc-guard] …` lines visible |
| Python `-B` flag | Unaffected (owned by smoke script) | Unaffected |

---

## Validation

### Path allowlist review

Only `tools/dev-launcher/Start-AutoplayVsBot.ps1` was modified.  No Rust/Bevy
source, no gameplay code, no production sprint files touched.

### `git diff --check`

```
DIFF_CHECK_OK
```
(no trailing whitespace, no conflict markers)

### Static grep — guard strings present

Lines confirmed in the modified file:
- `312: Write-Host "[stale-pyc-guard] cleared: $pycDir"`
- `317: Write-Host "[stale-pyc-guard] not present (skip): $pycDir"`
- `321: Write-Host "[stale-pyc-guard] PYTHONDONTWRITEBYTECODE=1 (composite stale-pyc guard active)"`

### No live GUI run

Per task scope — no live GUI run performed in this prompt.

---

## Integration Readiness

- Branch `fix/1814-stale-pyc-guard` pushed to origin.
- 1 file changed, 25 insertions(+).
- Ready for integration (merge to main or cherry-pick by orchestrator).
- The inner guard in `Run-AutoplaySmoke.ps1` is untouched and remains the
  authoritative guard for the smoke-only path; this change adds belt-and-suspenders
  coverage at the composite layer.

---

1814: AUTOPLAY-COMPOSITE-LAUNCHER-STALE-PYC-GUARD: SHIPPED
