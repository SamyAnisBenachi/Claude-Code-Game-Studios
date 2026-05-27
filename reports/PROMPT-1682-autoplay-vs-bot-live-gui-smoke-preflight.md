# PROMPT 1682 — AUTOPLAY-VS-BOT-LIVE-GUI-SMOKE-PREFLIGHT

**Date:** 2026-05-27  
**Source-of-truth at authoring:** `origin/main@aa9f4ae54f67c263a73da0659b06ef1ce09cb25e`  
**Audience:** Human QA operator running the next live GUI autoplay-vs-bot session.  
**Prior context:**
- PROMPT 1668 — full operator pack for AC7 + AC5; source `origin/main@e4249f07`
- PROMPT 1678 — headless bot-vs-bot soak PASS; bot placement + draft auto-pick landed
- This prompt: static preflight audit + narrow defect repair before the live GUI run

---

## Preflight Audit Summary

### Files Audited

| File | Status |
|------|--------|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | **DEFECT FOUND + FIXED** |
| `tools/autoplay/Run-AutoplaySmoke.ps1` | **DEFECT FOUND + FIXED** |
| `tools/autoplay/driver.py` | OK — no changes needed |
| `tools/autoplay/validate_composite_run.py` | OK — no changes needed |
| `tools/autoplay/recipes/full_game.py` | OK — no changes needed |
| `reports/PROMPT-1668-bot-room-ac7-autoplay-gui-smoke-operator-pack.md` | OK — still valid; see delta below |

---

## Defect Found: DriverTicks=10 Causes Silent Early Exit on Full-Game Recipe

### Root cause

`Run-AutoplaySmoke.ps1` had `$DriverTicks = 10` as its default. This value is
passed to `driver.py` as `--ticks 10`.

`driver.py` uses `--ticks` as a hard tick cap:
```python
ticks_cap = args.ticks if args.ticks > 0 else (last_recipe_tick + 2 if last_recipe_tick else 10)
```

With `--ticks 10 > 0`, the cap is 10 ticks regardless of recipe length.

The `full-game` recipe chains five sub-recipes (lobby → class → draft-auction →
placement → resolution) and produces approximately **235 ticks** total. The driver
exited after tick 10 (≈ 1 second at 10 hz) — well before any phase after the
lobby Create click.

### Consequence

The driver exited 0 (no error; just reached the tick cap), so `Start-AutoplayVsBot.ps1`
also exited 0 and wrote `"outcome": "ok"` to `composite-summary.json`.  
However, `validate_composite_run.py` would then fail with missing required checkpoints
(`lobby-confirmed`, `class-select-loaded`, `placement-loaded`, `placement-submitted`)
because only the first 10 ticks of the lobby sub-recipe had fired.

### Second defect: timeout hardcoded to 30s

`Run-AutoplaySmoke.ps1` hardcoded `"--timeout", "30"` when invoking `driver.py`.
At 10 hz with ~235 ticks, the recipe needs ~23.5 s of tick loop time. 30 s is marginal
(no safety for RPC round-trips, screenshot writes, or a slow host). The value was also
not configurable from `Start-AutoplayVsBot.ps1`, preventing operators from tuning it.

### Additional finding: DriverTicks not forwarded

`Start-AutoplayVsBot.ps1` did not pass `-DriverTicks` or any timeout value when
invoking `Run-AutoplaySmoke.ps1`, so operators had no way to correct the 10-tick cap
from the top-level launcher.

---

## Fix Applied (branch: prompt-1682-autoplay-preflight)

### `tools/autoplay/Run-AutoplaySmoke.ps1`

| Change | Before | After |
|--------|--------|-------|
| `$DriverTicks` default | `10` | `0` (follow recipe length) |
| `$DriverTimeoutSecs` | not a param; `"30"` hardcoded | new param, default `300` |
| `--timeout` passed to driver | `"30"` literal | `$DriverTimeoutSecs` |

### `tools/dev-launcher/Start-AutoplayVsBot.ps1`

| Change | Before | After |
|--------|--------|-------|
| `$DriverTicks` param | not present | added, default `0` |
| `$DriverTimeoutSecs` param | not present | added, default `300` |
| `$smokeArgs` invocation | no `-DriverTicks`, no `-DriverTimeoutSecs` | forwards both |
| DryRun print | did not show ticks/timeout | shows `-DriverTicks $DriverTicks -DriverTimeoutSecs $DriverTimeoutSecs` |
| Help text | no DriverTicks docs | documents both params + explains why 0 is required for full-game |

### Validation

```
git diff --check            → (no output) — no whitespace errors
PowerShell Parse: Start-AutoplayVsBot.ps1  → PARSE OK
PowerShell Parse: Run-AutoplaySmoke.ps1    → PARSE OK
DryRun:  -DriverTicks 0 -DriverTimeoutSecs 300 visible in DryRun print
DryRun:  -Help output shows updated DriverTicks and DriverTimeoutSecs docs
git diff --stat: 2 files changed, 13 insertions(+), 4 deletions(-)
Path allowlist: tools/autoplay/ and tools/dev-launcher/ only — within scope
```

---

## Operator Instructions (Live GUI Run)

> **Read the 1668 operator pack first.** This section covers only the delta
> from PROMPT 1668 caused by the fixes above and the newer source-of-truth commit.

### Prerequisites — Updated Source Check

| Check | Command | Pass condition |
|-------|---------|----------------|
| On latest main | `git fetch origin && git log --oneline -3` | Top commit is `aa9f4ae5` or newer |
| Branch with fix merged | top commit includes `prompt-1682-autoplay-preflight` merge | After this PR merges |
| PowerShell version | `$PSVersionTable.PSVersion` | Major ≥ 5 |
| Python on PATH | `python --version` | 3.8 or newer |
| Interactive desktop | — | Visible Windows desktop |
| Repo root CWD | `cd D:\_DEV\Work\Claude-Code-Game-Studios` | All commands run from here |

### Track B — AUTOPLAY-VS-BOT Live GUI Smoke (Updated Command)

**Terminal 3 — no changes if using standalone mode (no pre-existing soak server):**

```powershell
cd D:\_DEV\Work\Claude-Code-Game-Studios

powershell -ExecutionPolicy Bypass `
    -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
    -Recipe full-game `
    -ClientStartupSecs 90
```

The fix is transparent — the launcher now passes `-DriverTicks 0 -DriverTimeoutSecs 300`
internally. No operator flag changes required.

**If running alongside Track A (soak server already up on port 5000):**

```powershell
powershell -ExecutionPolicy Bypass `
    -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
    -Recipe full-game `
    -SkipSoakLaunch `
    -Port 5000 `
    -ClientStartupSecs 90
```

### Expected DryRun output snippet (smoke confirmation step)

Before the live run, confirm the fix is active with:

```powershell
powershell -ExecutionPolicy Bypass `
    -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
    -DryRun
```

The dry-run output should contain the line:

```
[DRY RUN] would launch: powershell -ExecutionPolicy Bypass -File ...\Run-AutoplaySmoke.ps1
    -Port 15873 -Recipe full-game ... -DriverTicks 0 -DriverTimeoutSecs 300
```

If you see `-DriverTicks 10` or if `-DriverTicks` is absent, the fix is not on your branch.

### Tick count reference (full-game recipe with default env vars)

| Phase | Sub-recipe | Approx. last tick (absolute) |
|-------|-----------|------------------------------|
| Lobby | lobby-create | ~23 |
| Class | class-select | ~74 |
| Draft/Auction | draft-auction-probe | ~129 |
| Placement | placement-drag-probe | ~160 |
| Resolution soak | resolution-observe (60 ticks) | ~229 |
| Tail checkpoint | full-game-post-resolution | ~235 |

At 10 hz, 235 ticks ≈ **23.5 seconds** of active tick loop time.  
The 300-second timeout provides a 12.7× safety margin for RPC overhead and screenshot writes.

---

## Pass / Fail Criteria

> Unchanged from PROMPT 1668 §B-8. Reproducing for reference.

| Criterion | Pass | Fail |
|-----------|------|------|
| Composite harness exits 0 | `Start-AutoplayVsBot.ps1` exit code = 0 | Non-zero exit |
| `composite-summary.json` outcome | `"outcome": "ok"` | `"outcome"` ≠ `"ok"` |
| All required checkpoints present | All 14 labels from §B-6 (1668 pack) appear in `checkpoints.jsonl` | Any label missing |
| Resolution checkpoint reached | `resolution-started` and `resolution-complete` both present | Either missing |
| No `block` entries | `checkpoints.jsonl` contains zero rows with `"kind":"block"` | Any `block` row present |
| Composite validator exits 0 | `validate_composite_run.py` exits 0 | Exits 1 or 2 |

**AC5 PASS** = all six criteria satisfied.

---

## Evidence Files Expected After a PASS Run

```
production/qa/evidence/composite-runs/<UTC>-autoplay-vs-bot/
    composite-summary.json              ← outcome:"ok", smoke_exit_code:0
    autoplay-run-path.txt               ← path to autoplay artifact dir

production/qa/evidence/autoplay-runs/<YYYYMMDD-HHMMSS>-Z/
    launcher-status.json
    checkpoints.jsonl                   ← 14+ checkpoint rows, zero block rows
    driver.log
    driver-timeline.jsonl
    process.log
    screenshots/
        001.png + 001.json  (lobby-loaded)
        002.png + 002.json  (lobby-confirmed)
        ...
        013.png + 013.json  (resolution-complete)
        014.png + 014.json  (full-game-post-resolution)
```

### Post-run validator command

```powershell
python tools/autoplay/validate_composite_run.py `
    production\qa\evidence\composite-runs\<UTC>-autoplay-vs-bot
```

Expected: `[validate_composite_run] PASS: ...` with exit 0.

---

## Failure Triage Paths

| Symptom | Diagnosis | Fix |
|---------|-----------|-----|
| Driver exits at tick 10 / `checkpoints.jsonl` has only `lobby-loaded` | Old `Run-AutoplaySmoke.ps1` (pre-fix) still in play | Confirm fix branch is merged; `git log --oneline -1` |
| `checkpoints.jsonl` missing after driver exits 0 | Tick cap hit but no checkpoint reached; driver.log shows "reached tick cap 10" | Same as above |
| DryRun output shows `-DriverTicks 10` | Fix not merged | Merge `prompt-1682-autoplay-preflight` or cherry-pick |
| Composite harness exits 4 | `CCGS_AUTOPLAY_BOT_ROOM_READY` not set | Fix: add `-SkipSoakLaunch` (already running soak) OR omit it and let launcher start soak |
| Composite harness exits 10 | Non-interactive session | Run from a visible Windows terminal |
| Composite harness exits 11 | Cargo.toml not found | Confirm CWD is repo root |
| Composite harness exits 12 | Soak server did not bind in time | Add `-SoakReadySecs 40`; check server build time |
| Driver timeout after 300 s | Very slow host / screenshot accumulation | Reduce recipe to omit resolution soak: `$env:CCGS_AUTOPLAY_FULL_GAME_RESOLUTION=0`, or increase `-DriverTimeoutSecs 600` |
| validate_composite_run.py exits 1 with MISSING CHECKPOINTS | Recipe ran but phase gating missed | Read `driver.log` for tick cap or timeout line; attach `checkpoints.jsonl` to bug report |

---

1682: AUTOPLAY-VS-BOT-LIVE-GUI-SMOKE-PREFLIGHT: SHIPPED
