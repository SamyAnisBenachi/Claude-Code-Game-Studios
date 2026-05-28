# PROMPT 1825 — POST-1824-VSBOT-ENV-GATE-VERIFY

**Status:** PASS
**Date:** 2026-05-28
**Worktree:** `D:/_DEV/Work/Claude-Code-Game-Studios/tmptmpwt-1825-post-1824-verify`
**Branch:** `prompt/1825-post-1824-vsbot-env-gate-verify`
**HEAD:** `a0a96360` (feat(autoplay): PROMPT 1824 — vs-bot env gate in Run-AutoplaySmoke.ps1)

---

## 1. Landing Status

**PROMPT 1824 is LANDED on origin/main.**

```
git log --oneline origin/main -3
a0a96360 feat(autoplay): PROMPT 1824 — vs-bot env gate in Run-AutoplaySmoke.ps1
822c4873 docs(reports): PROMPT 1822 — backfill PROMPT 1820 and 1821 reports to root reports/
3c3aa6d7 docs(reports): PROMPT 1818 — frozen PrintWindow BitBlt fallback implementation report
```

Branch `origin/prompt/1824-vsbot-env-gate` confirmed present and merged.

---

## 2. Static Check — `tools/autoplay/Run-AutoplaySmoke.ps1`

| Check | Result | Lines |
|---|---|---|
| Gate scoped to `$Recipe -eq 'vs-bot'` only | PASS | 57 |
| `CCGS_DEBUG_UI` auto-set to `'1'` + logged when absent | PASS | 58–61 |
| Missing `CCGS_AUTOPLAY_BOT_ROOM_READY` exits before cargo build | PASS | 62–77 (cargo build at line 80) |
| Exit code `4` on missing bot-room gate | PASS | 76 |
| Remediation mentions `Start-BotVsBotSoak.ps1` | PASS | 68 |
| Remediation mentions env var set + re-run | PASS | 70–73 |
| Remediation mentions `Start-AutoplayVsBot.ps1 -Recipe vs-bot` | PASS | 73–74 |

---

## 3. Live Gate Test

**Command:**
```powershell
powershell -ExecutionPolicy Bypass -NonInteractive -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
```
(Run with `CCGS_AUTOPLAY_BOT_ROOM_READY` unset, `CCGS_DEBUG_UI` unset)

**Output captured:**
```
[autoplay-smoke] repo=D:\_DEV\Work\Claude-Code-Game-Studios\tmptmpwt-1825-post-1824-verify port=15873 artifact_dir=production\qa\evidence\autoplay-runs\20260528-083457-Z
[autoplay-smoke] vs-bot: CCGS_DEBUG_UI not set -- auto-set to 1

[autoplay-smoke] BLOCKED: CCGS_AUTOPLAY_BOT_ROOM_READY is not set.
  The vs-bot recipe requires a running bot soak room before the driver starts.
  To fix:
    1. In a separate terminal, start the bot soak room:
         pwsh -File tools/dev-launcher/Start-BotVsBotSoak.ps1
    2. Wait for it to print 'Server listening on port ...'
    3. In this terminal, set the env var and re-run:
         $env:CCGS_AUTOPLAY_BOT_ROOM_READY = '1'
         pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe vs-bot
  Alternatively Start-AutoplayVsBot.ps1 handles both automatically:
         pwsh -File tools/dev-launcher/Start-AutoplayVsBot.ps1 -Recipe vs-bot
```

**Exit code:** `4`

**Key observations:**
- `CCGS_DEBUG_UI` auto-set logged before the BLOCKED message — no operator action needed
- BLOCKED fires before cargo build line (80+) — zero compile time wasted
- Exit code 4 matches the expected driver `local.block` value
- Remediation text is actionable and complete

---

## 4. `--list-recipes` Check

**Command:**
```
python tools/autoplay/driver.py --list-recipes
```

**`vs-bot` entry:**
```
vs-bot    Composite recipe (add-bot-lobby -> class -> draft/auction -> placement -> resolution soak).
          Requires CCGS_DEBUG_UI=1 AND CCGS_AUTOPLAY_BOT_ROOM_READY=1; emits BLOCKED otherwise.
          Resolution observation on by default; GameOver opt-in via CCGS_AUTOPLAY_VS_BOT_GAMEOVER=1.
```

**Result:** PASS — `vs-bot` is listed and description is accurate.

---

## 5. Non-vs-bot Recipe Unchanged

Static check: the gate block at lines 57–78 is wrapped in `if ($Recipe -eq 'vs-bot')`.
All other recipes (smoke, full-game, etc.) skip the block entirely. PASS.

---

## 6. Human Command Path — Safe to Use

The human command path is now safe. The previous failure mode (silent BLOCK after a
full 2-minute cargo build + client launch cycle) cannot occur:

| Scenario | Old behavior | New behavior |
|---|---|---|
| `Run-AutoplaySmoke.ps1 -Recipe vs-bot` with no env vars | 2-min build, then driver BLOCK exit 4 | Instant BLOCK at line 62, exit 4, clear remediation |
| `CCGS_DEBUG_UI` absent | Driver BLOCK (Add Bot hidden) | Auto-set silently, continues |
| `CCGS_AUTOPLAY_BOT_ROOM_READY` absent | Driver BLOCK after full build | Fast-exit before build, remediation printed |
| `Start-AutoplayVsBot.ps1 -Recipe vs-bot` | Sets both vars; still correct | Unchanged; sets `CCGS_DEBUG_UI` at line 336, `CCGS_AUTOPLAY_BOT_ROOM_READY=1` at line 329 |

---

## 7. Safe Re-run Command

```powershell
# Terminal 1 — start the bot soak room:
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1

# Terminal 2 — once soak prints "Server listening on port ...":
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = '1'
powershell -ExecutionPolicy Bypass -File tools\autoplay\Run-AutoplaySmoke.ps1 -Recipe vs-bot

# Or: one-command via composite launcher (handles both automatically):
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -Recipe vs-bot
```

---

1825: POST-1824-VSBOT-ENV-GATE-VERIFY: PASS
