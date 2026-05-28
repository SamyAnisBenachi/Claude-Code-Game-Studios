# PROMPT 1750 — Autoplay-vs-Bot Live Blocker Reduction Audit

_Audited: 2026-05-28 | Branch: `audit/1750-autoplay-vs-bot-live-blocker` | Base: `origin/main@ba7bb394`_

---

## Scope

Scripts inspected:
- `tools/dev-launcher/Start-AutoplayVsBot.ps1` (PROMPT 1644 composite harness)
- `tools/dev-launcher/Start-BotVsBotSoak.ps1` (PROMPT 1603 headless server launcher)
- `tools/autoplay/Run-AutoplaySmoke.ps1` (PROMPT 1595 client build + driver runner)
- `tools/autoplay/recipes/vs_bot.py` (PROMPT 1747 recipe)
- `tools/autoplay/recipes/add_bot_lobby.py`
- `tools/autoplay/driver.py` (env propagation path)

---

## Environment Snapshot

| Item | State |
|------|-------|
| D: free space | **~14 GB** (15,017,992,192 bytes) — disk pressure resolved |
| `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe` | EXISTS, built 2026-05-27 23:48:07 UTC |
| `D:\_DEV\cargo-target\ccgs-msvc\debug\bot-soak-trigger.exe` | EXISTS, built 2026-05-27 23:49:15 UTC |
| `D:\_DEV\cargo-target\ccgs-msvc\debug\client.exe` | EXISTS, built 2026-05-21 (without `autoplay-remote` feature — needs rebuild) |
| `D:\_DEV\ccgs-play-main` | EXISTS but **no Cargo.toml** (stub/evidence dir) |

---

## Blocker Matrix

### BLOCKER 1 — `CCGS_DEBUG_UI=1` not threaded through by composite launcher (**FIXED**)

**Root cause:** `Start-AutoplayVsBot.ps1` sets `CCGS_AUTOPLAY_BOT_ROOM_READY`, `SERVER_PORT`, `SERVER_URL` before calling `Run-AutoplaySmoke.ps1`, but does **not** set `CCGS_DEBUG_UI=1`.

**Effect chain:**
1. Operator runs `Start-AutoplayVsBot.ps1 -Recipe vs-bot`
2. `Run-AutoplaySmoke.ps1` builds and launches the Bevy client (inherits env — no `CCGS_DEBUG_UI`)
3. `driver.py` line 187: `ctx = RecipeContext(..., env=dict(os.environ))` — Python reads its own env
4. `vs_bot.py` line 68: `if ctx.env.get("CCGS_DEBUG_UI") != "1": → local.block`
5. Driver exits 4 → composite outcome `blocked-recipe-guard`
6. The Add Bot button is also never rendered in the Bevy client lobby (same guard in `client/src/ui/lobby.rs`)

**Fix applied** — `Start-AutoplayVsBot.ps1` now sets `$env:CCGS_DEBUG_UI = '1'` when `$Recipe -eq 'vs-bot'` (env-vars block, step 7). This propagates to both the client process and the Python driver via Windows env inheritance.

---

### BLOCKER 2 — Soak child launched without `-PlayRepoRoot` passthrough (**FIXED**)

**Root cause:** `Start-AutoplayVsBot.ps1` correctly handles the `D:\_DEV\ccgs-play-main` stub (directory exists, no Cargo.toml → falls back to launcher root). But it launches `Start-BotVsBotSoak.ps1` as a background job **without** `-PlayRepoRoot`, so the soak script runs its own fallback resolution independently.

**`Start-BotVsBotSoak.ps1` fallback (lines 176-183):** when `D:\_DEV\ccgs-play-main` exists it sets `$PlayRoot = $DefaultPlayRoot` unconditionally (no Cargo.toml check at this stage). Then line 192 finds no Cargo.toml → `exit 1`.

**Effect chain:**
1. Soak job starts, immediately exits 1 (bad root)
2. Port 5000 never binds
3. After `SoakReadySecs` (20s): composite launcher exits 12 (`BLOCKED-PRECONDITION: soak server did not bind`)

**Fix applied** — `Start-AutoplayVsBot.ps1` now passes `-PlayRepoRoot $RepoRoot` to `$soakArgs`, propagating the already-resolved workspace root to the child. The DRY RUN display string was updated to match.

---

### NOT A BLOCKER — D: disk pressure (previously PROMPT 1740)

D: currently has **~14 GB free**. Disk pressure that PROMPT 1740 identified has been resolved. `cargo build -p client --features autoplay-remote` should succeed with current free space. If disk becomes tight again, that is a human precondition (clear old target artifacts) rather than a script defect.

---

### NOT A BLOCKER — `Run-AutoplaySmoke.ps1` always rebuilds client

`Run-AutoplaySmoke.ps1` always runs `cargo build -p client --features autoplay-remote` (lines 54–61). It has no freshness guard or skip flag, unlike `Start-BotVsBotSoak.ps1` which has `Get-BinaryBuildReason`. The existing `client.exe` was built without the `autoplay-remote` feature, so the first run will do a full feature rebuild (~2–4 min). With 14 GB free this is slow but not blocking. A `-SkipClientBuild` flag would be a future improvement, not a now-fix.

---

### NOT A BLOCKER — Default recipe is `full-game`, not `vs-bot`

`Start-AutoplayVsBot.ps1` defaults to `-Recipe full-game`. Operator must pass `-Recipe vs-bot` explicitly to exercise the PROMPT 1747 recipe. This is by design (the composite harness predates the vs-bot recipe). Documented in operator commands below.

---

## Files Changed

| File | Change |
|------|--------|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | +`-PlayRepoRoot $RepoRoot` in `$soakArgs`; +DRY RUN display string update; +`CCGS_DEBUG_UI=1` when recipe is `vs-bot` |

No other files modified. `Start-BotVsBotSoak.ps1` and `Run-AutoplaySmoke.ps1` unchanged.

---

## Validation

- PowerShell parser check: **PARSE OK** (no syntax errors)
- Dry-run (`-DryRun -Recipe vs-bot`): confirmed
  - `CCGS_DEBUG_UI = 1 (set for vs-bot recipe)` printed
  - `[DRY RUN] would launch: ... Start-BotVsBotSoak.ps1 ... -PlayRepoRoot D:\tmp\tmpwt-1750-autoplay-audit` printed
  - All other sections intact

---

## Exact Operator Commands

### Option A — Full composite run (recommended; everything automatic)

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios (or play checkout root)
powershell -ExecutionPolicy Bypass `
  -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
  -Recipe vs-bot
```

This will:
1. Fall back to launcher root (warns about ccgs-play-main stub — expected)
2. Start `Start-BotVsBotSoak.ps1` with `-PlayRepoRoot` pointed at the correct workspace
3. Wait up to 20s for the soak server to bind on port 5000
4. Set `CCGS_DEBUG_UI=1` and `CCGS_AUTOPLAY_BOT_ROOM_READY=1`
5. Build the client with `--features autoplay-remote` (~2–4 min first time)
6. Launch the visible Bevy client + run the Python driver with `vs-bot` recipe
7. Write evidence to `production/qa/evidence/composite-runs/<UTC>-autoplay-vs-bot/`

### Option B — Pre-built server already running (skip soak launch)

```powershell
# Terminal 1 — start soak server in advance
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1

# Terminal 2 — composite launcher, skip soak
powershell -ExecutionPolicy Bypass `
  -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
  -Recipe vs-bot `
  -SkipSoakLaunch -Port 5000
```

### Dry run (safe, no processes)

```powershell
powershell -ExecutionPolicy Bypass `
  -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
  -Recipe vs-bot -DryRun
```

### Validate evidence after a run

```powershell
python tools/autoplay/validate_composite_run.py `
  production\qa\evidence\composite-runs\<YYYY-MM-DD-HHMMSS-autoplay-vs-bot>
```

---

## Live PASS Gate Reminder

A live PASS for `AUTOPLAY-VS-BOT-QA-001` (Story 004) still requires:
1. Operator runs Option A in an interactive terminal with visible desktop
2. Exit code 0, `outcome: ok` in `composite-summary.json`
3. `checkpoints.jsonl` contains `vs-bot-post-resolution` (or `vs-bot-complete` if GameOver opt-in)
4. Human operator attaches artifacts and signs off

`GAP-01` and `GAP-02` remain open until the operator completes the above.

---

1750: AUTOPLAY-VS-BOT-LIVE-BLOCKER-REDUCTION-AUDIT: SHIPPED
