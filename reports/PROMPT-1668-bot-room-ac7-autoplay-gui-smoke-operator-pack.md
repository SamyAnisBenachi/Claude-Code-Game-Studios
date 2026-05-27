# PROMPT 1668 — BOT-ROOM-AC7 & AUTOPLAY-VS-BOT GUI Smoke Operator Pack

**Date:** 2026-05-27  
**Source-of-truth at authoring:** `origin/main@e4249f07`  
**Audience:** Human QA operator running the next live GUI session.  
**Prior context:**
- PROMPT 1661 — readiness audit; AC7 flagged HUMAN/LIVE GUI GATE — OPEN
- PROMPT 1663 — autoplay dry-run PASS post-1662; live run classified HUMAN_SIGNOFF_REQUIRED
- PROMPT 1665 — AC8 ruled OUT OF SCOPE for story-001; story-001 path to done is AC7 only

---

## Overview

This pack covers two parallel QA tracks that must both complete in the same session:

| Track | Criterion | Story Gate |
|-------|-----------|------------|
| **A** | BOT-ROOM-PARTICIPANT-001 **AC7** — human plays one full friend-game round vs bot without server panic | Closes story-001 (`/story-done` AC7 row) |
| **B** | AUTOPLAY-VS-BOT live GUI smoke — `Start-AutoplayVsBot.ps1 -Recipe full-game` emits composite evidence with exit 0 | Closes AUTOPLAY-VS-BOT-QA-001 (`/story-done` AC5 row) |

Track A and Track B share the same soak server. Run the soak server once; run both tracks against it.

---

## Shared Prerequisites

| Check | Command | Pass condition |
|-------|---------|----------------|
| On latest main | `git fetch origin && git log --oneline -3` | Top commit is `e4249f07` or newer |
| PowerShell version | `$PSVersionTable.PSVersion` | Major ≥ 5 |
| Python on PATH | `python --version` | 3.8 or newer |
| Interactive desktop | — | You are at a visible Windows desktop; not in a CI/headless session |
| Repo root CWD | `cd D:\_DEV\Work\Claude-Code-Game-Studios` | All subsequent commands run from here |

---

## Track A — BOT-ROOM-PARTICIPANT-001 AC7: Human vs Bot Live Session

**AC7 exact wording** (story-001 line 82–83):
> A real human client can complete a friend-game round against the bot  
> (DRAFT_INITIAL → DRAFT_SHOP → AUCTION → PLACEMENT → RESOLUTION → next-loop)  
> without server panic.

### A-1 Environment Variables

Set in the terminal before launching any process:

```powershell
$env:CCGS_BOT_DECISION_LOG_PATH = "$PWD\production\qa\evidence\dev-runs\bot-decision-log-ac7-$(Get-Date -Format 'yyyyMMdd-HHmmss').jsonl"
$env:CCGS_QA_SNAPSHOT = "1"
$env:CCGS_QA_SNAPSHOT_DIR = "$PWD\production\qa\evidence\dev-runs"
$env:CCGS_BOT_QA_SNAPSHOT = "1"
$env:CCGS_DEBUG_UI = "1"
$env:CCGS_BOT_DEBUG_UI = "1"
```

### A-2 Launch the Soak Server (Terminal 1)

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 `
    -DurationSeconds 900 `
    -MaxRounds 2
```

**Expected output (within ~30 s after build):**

```
==== Evidence dir ====
Evidence dir: production\qa\evidence\dev-runs\<UTC>-bot-vs-bot-soak\
==== Server ====
Server bound on port 5000
...
```

Wait until you see `Server bound on port 5000` (or similar) before proceeding.

> If port 5000 is busy the launcher auto-bumps to the next free port and logs
> `Chosen port: 5001`. Note the actual port — you need it for Track B.

### A-3 Launch the Human Client (Terminal 2)

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios
$env:SERVER_URL = "ws://127.0.0.1:5000"   # adjust if port bumped in A-2
$env:CCGS_QA_SNAPSHOT = "1"
$env:CCGS_DEBUG_UI = "1"
cargo run -p client
```

> Building cold takes 5–15 min; warm takes ~1–3 min. Wait for the Bevy window.

### A-4 Operator Steps (UI interaction)

Perform each step in the Bevy client window. The bot drives its own side — you interact as player 1.

| Step | Action | Expected UI state |
|------|--------|-------------------|
| 1 | Click **Create Room** | Lobby screen loads; room code visible |
| 2 | Wait ≤ 10 s | Bot joins automatically (server-side join via `Add-Bot`). Room shows two participants. |
| 3 | Click **Confirm / Ready** | Transition to **class-select** screen |
| 4 | Click any class card | Card highlights |
| 5 | Click **Confirm** | Transition to **DRAFT_INITIAL / DRAFT_SHOP** screen (shop phase) |
| 6 | Click one shop slot | Item highlights |
| 7 | Click **Confirm** / proceed | Transition to **AUCTION** screen |
| 8 | Click the bid CTA and **Ready** | Ready state set; waiting for bot ready |
| 9 | Wait for bot to ready | Transition to **PLACEMENT** screen |
| 10 | Drag one unit from hand to board | Unit appears on board |
| 11 | Click **Submit** | Placement submitted; waiting for bot submit |
| 12 | Wait for bot submit | Transition to **RESOLUTION** screen; combat animation plays |
| 13 | Wait for resolution to complete | Transition to next round lobby or game-over screen |

> **Do not force-quit the client during steps 8–13.** A clean resolution cycle is the key AC7 gate.

### A-5 During the Run — What to Capture

1. **Press F9** (or click the QA Snapshot button, if visible) at each phase transition: after lobby join, after class confirm, after auction ready, after placement submit, after resolution complete. This writes `snapshot.json` + screenshot to `production/qa/evidence/dev-runs/`.
2. **Press F8** once to open the bot debug overlay. Take a screenshot of the overlay showing bot hand and decision entries.
3. After round completes (or game over), **do not close the client** until step A-6 is complete.

### A-6 Evidence to Collect

After round completion (no server panic):

| Evidence artifact | Location | Content |
|------------------|----------|---------|
| `server.log` | `production/qa/evidence/dev-runs/<run-stamp>-bot-vs-bot-soak/server.log` | Full server stdout — confirm no `panicked at` line |
| `server.err` | Same directory | Stderr — confirm no `panicked at` |
| `bot-decision-log.jsonl` | `$env:CCGS_BOT_DECISION_LOG_PATH` | At least one entry per phase (bid, placement) for the bot |
| `snapshot.json` (×N) | `production/qa/evidence/dev-runs/` | One snapshot per F9 press |
| F9 screenshots | Same directory as snapshots | PNG per press |
| F8 overlay screenshot | Manually saved | Bot hand + decision log visible |

### A-7 Pass / Fail Criteria (AC7)

| Criterion | Pass | Fail |
|-----------|------|------|
| Server does not crash | `server.log` / `server.err` contain **zero** `panicked at` lines | Any `panicked at` line |
| Full phase sequence observed | UI progressed through DRAFT_SHOP → AUCTION → PLACEMENT → RESOLUTION at minimum | Any phase transition missing without an error reason |
| Bot participated | `bot-decision-log.jsonl` has ≥ 1 entry with `phase: "AuctionBid"` and ≥ 1 entry with `phase: "Placement"` | Log missing or empty |
| QA snapshots captured | At least one `snapshot.json` for a phase at or after PLACEMENT | No snapshots captured |
| No client panic | Client window stays open through resolution; no Windows crash dialog | Client window closes unexpectedly |

**AC7 PASS** = all five criteria above are satisfied.

---

## Track B — AUTOPLAY-VS-BOT Live GUI Smoke

**AC5 exact wording** (AUTOPLAY-VS-BOT-QA-001):
> Composite flow reaches at least one full RESOLUTION (autoplay-driven client + bot server).

### B-1 Prerequisite — Soak Server Running

Track A's soak server (Terminal 1) must still be running. Confirm with:

```powershell
# Quick port check
Test-NetConnection -ComputerName 127.0.0.1 -Port 5000 -InformationLevel Quiet
```

Expected: `True`. If false, restart the soak server (Track A step A-2).

### B-2 Launch the Composite Harness (Terminal 3)

From the repo root:

```powershell
cd D:\_DEV\Work\Claude-Code-Game-Studios

powershell -ExecutionPolicy Bypass `
    -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
    -Recipe full-game `
    -SkipSoakLaunch `
    -Port 5000 `
    -SoakDurationSeconds 600 `
    -ClientStartupSecs 90
```

> `-SkipSoakLaunch` tells the launcher that the soak server is already running
> (Track A launched it). Use the actual port if it was bumped in Track A.
>
> `-ClientStartupSecs 90` gives the Bevy client an extra 30 s margin for a warm build.

**Alternative — composite harness launches everything itself (Track B standalone):**

If Track A is not being run concurrently, drop `-SkipSoakLaunch` and `-Port`:

```powershell
powershell -ExecutionPolicy Bypass `
    -File tools\dev-launcher\Start-AutoplayVsBot.ps1 `
    -Recipe full-game `
    -ClientStartupSecs 90
```

The launcher will start `Start-BotVsBotSoak.ps1` as a background job and wait
up to 20 s for port bind before handing off to the autoplay smoke.

### B-3 Expected Terminal Output Sequence

```
==== Roots ====
Launcher repo root: D:\_DEV\Work\Claude-Code-Game-Studios
Play/build root:    D:\_DEV\Work\Claude-Code-Game-Studios  (source: ...)

==== Desktop session check ====
UserInteractive: True

==== Child launcher check ====
Start-BotVsBotSoak.ps1 : found (...)
Run-AutoplaySmoke.ps1  : found (...)

==== Evidence dir ====
Evidence dir: production\qa\evidence\composite-runs\<UTC>-autoplay-vs-bot\

==== Soak server port ====
SkipSoakLaunch=true -- assuming server already listening on port 5000.

==== Autoplay smoke (recipe=full-game) ====
CCGS_AUTOPLAY_BOT_ROOM_READY = 1
SERVER_PORT                  = 5000
SERVER_URL                   = ws://127.0.0.1:5000
Autoplay artifact dir:       production\qa\evidence\autoplay-runs\<YYYYMMDD-HHMMSS>-Z\
powershell -ExecutionPolicy Bypass -File ... Run-AutoplaySmoke.ps1 -Port 15873 -Recipe full-game ...

... (build output / Bevy window opens) ...

Run-AutoplaySmoke.ps1 exited: 0

==== Composite summary ====
Composite summary: production\qa\evidence\composite-runs\<UTC>-autoplay-vs-bot\composite-summary.json
Composite run COMPLETE (recipe=full-game exit=0).
NOTE: This is NOT a live PASS for AUTOPLAY-VS-BOT-QA-001. An operator must review artifacts and sign off.
```

**Final exit code must be 0.**

> Do NOT click in the autoplay-driven Bevy window while the recipe runs.
> The driver injects input; human clicks race with recipe steps.

### B-4 Expected Evidence Files

After exit 0, the following must exist:

```
production/qa/evidence/composite-runs/<UTC>-autoplay-vs-bot/
    composite-summary.json              ← top-level outcome
    autoplay-run-path.txt               ← path to autoplay artifact dir

production/qa/evidence/autoplay-runs/<YYYYMMDD-HHMMSS>-Z/
    launcher-status.json                ← overall verdict + exit codes
    checkpoints.jsonl                   ← phase-gate log — CRITICAL
    driver.log                          ← human-readable progress
    driver-timeline.jsonl               ← per-tick log
    process.log                         ← Bevy stdout/stderr
    screenshots/
        001.png, 001.json               ← lobby-loaded checkpoint
        002.png, 002.json               ← lobby-confirmed
        ...                             ← one pair per checkpoint
```

### B-5 Composite Evidence Validation

After the run, validate the composite evidence directory:

```powershell
python tools/autoplay/validate_composite_run.py `
    production\qa\evidence\composite-runs\<UTC>-autoplay-vs-bot
```

Expected output: `[validate_composite_run] PASS: production\qa\evidence\composite-runs\<UTC>-autoplay-vs-bot`  
Expected exit code: **0**

### B-6 Checkpoint Verification

Open `checkpoints.jsonl` in the autoplay artifact directory. A successful `full-game` run
must contain **all** of the following `checkpoint` labels in order:

| Checkpoint | Phase |
|------------|-------|
| `lobby-loaded` | Lobby screen interactive |
| `lobby-confirmed` | Confirm CTA clicked |
| `class-select-loaded` | Class selection rendered |
| `class-confirmed` | First card + Confirm |
| `shop-loaded` | Shop phase mounted |
| `shop-slot-clicked` | First slot clicked |
| `auction-loaded` | Auction phase mounted |
| `auction-ready` | Ready button clicked |
| `placement-loaded` | Placement board appeared |
| `placement-dragged` | Drag from hand to board |
| `placement-submitted` | Submit button clicked |
| `resolution-started` | Resolution soak begins |
| `resolution-complete` | Resolution soak ended |
| `full-game-post-resolution` | Composite run ended (default terminal checkpoint) |

> If `full-game-post-resolution` is absent but `full-game-complete` is present,
> the optional GameOver soak ran — also a PASS.  
> Any `block` entry in `checkpoints.jsonl` = BLOCKED (not PASS).

### B-7 composite-summary.json Fields to Verify

Open `composite-summary.json` and confirm:

```json
{
  "schema":           "autoplay_vs_bot_composite_summary_v1",
  "outcome":          "ok",
  "recipe":           "full-game",
  "smoke_exit_code":  0,
  "dry_run":          false,
  "live_pass_status": "NOT-CLAIMED ..."
}
```

> `outcome: "ok"` and `smoke_exit_code: 0` are the two hard pass conditions.
> `live_pass_status` containing `NOT-CLAIMED` is expected — the script always
> emits this; it does **not** indicate failure. The operator's sign-off (this
> document) is the live PASS claim.

### B-8 Pass / Fail Criteria (AUTOPLAY-VS-BOT AC5)

| Criterion | Pass | Fail |
|-----------|------|------|
| Composite harness exits 0 | `Start-AutoplayVsBot.ps1` exit code = 0 | Non-zero exit |
| `composite-summary.json` outcome | `"outcome": "ok"` | `"outcome"` ≠ `"ok"` |
| All required checkpoints present | All 14 labels from §B-6 appear in `checkpoints.jsonl` | Any label missing |
| Resolution checkpoint reached | `resolution-started` and `resolution-complete` both present | Either missing |
| No `block` entries | `checkpoints.jsonl` contains zero rows with `"kind":"block"` | Any `block` row present |
| Composite validator exits 0 | `validate_composite_run.py` exits 0 | Exits 1 or 2 |

**AC5 PASS** = all six criteria above are satisfied.

---

## Story Gates Closed on PASS

| Story | AC | Gate condition | How to close |
|-------|----|----------------|--------------|
| BOT-ROOM-PARTICIPANT-001 | **AC7** | Track A PASS criteria (§A-7) | Attach evidence filelist + server.log to `/story-done` call |
| BOT-ROOM-PARTICIPANT-001 | **AC8** | OUT OF SCOPE — deferred to BOT-DISCONNECT-REJOIN-006 | Include AC8 scope ruling text from PROMPT 1665 §5 in `/story-done` PR |
| AUTOPLAY-VS-BOT-QA-001 | **AC5** | Track B PASS criteria (§B-8) | Attach `composite-summary.json` + `checkpoints.jsonl` + validator exit 0 |
| AUTOPLAY-VS-BOT-QA-001 | **AC2, AC3, AC4** | Infrastructure is wired; confirmed live by Track B run | Include evidence dir path in `/story-done` |
| AUTOPLAY-RECIPE-LIBRARY-001 | **AC2** | `full-game` recipe reached RESOLUTION | Track B PASS is sufficient for AC2 |
| AUTOPLAY-RECIPE-LIBRARY-001 | **AC6** | Failures surface failing step + QA snapshot | Confirmed by checkpoints.jsonl step log presence in Track B |

> AUTOPLAY-VS-BOT-QA-001 story-done is still gated on upstream stories 001 + 002 + 003 being
> story-done first. Track B PASS closes AC5 evidence but the story cannot reach `/story-done`
> until the upstream chain clears.

---

## Debrief — What to Record Regardless of Outcome

After any run — pass or fail — save the following in a `RESULT.md` file alongside the artifacts:

```markdown
## AC7 Live Run Result

- Date (UTC):
- Operator:
- Server commit: origin/main@e4249f07 (or later)
- PASS / FAIL / BLOCKED:
- Server panic (yes/no): [y/n] — copy first panicked line if yes
- Phase sequence completed: [DRAFT_SHOP → AUCTION → PLACEMENT → RESOLUTION / stopped at ___]
- Bot decision log entries: [present / absent / count ___]
- QA snapshots captured: [count ___]
- Notes:

## AUTOPLAY-VS-BOT Track B Result

- Composite exit code:
- outcome field in composite-summary.json:
- Last checkpoint in checkpoints.jsonl:
- validate_composite_run.py exit:
- PASS / FAIL / BLOCKED:
- Blocker label (if exit 4):
- Notes:
```

---

## Troubleshooting Quick-Reference

| Symptom | Diagnosis | Fix |
|---------|-----------|-----|
| Bot never joins (lobby stays at 1 participant) | Soak server not running or port mismatch | Check Terminal 1; confirm `SERVER_URL` matches soak port |
| Server panic in `server.log` | AC7 FAIL — record panic line | File as defect; do not call `/story-done` |
| Track B exits with code 4 | `CCGS_AUTOPLAY_BOT_ROOM_READY` was not set, or soak server dropped | Ensure `-SkipSoakLaunch` + soak server still alive; retry |
| Track B exits with code 10 | Non-interactive session | Run from a real Windows terminal, not scheduled task |
| Track B exits with code 11 | `Cargo.toml` not found at resolved play root | Confirm CWD is `D:\_DEV\Work\Claude-Code-Game-Studios` |
| Track B exits with code 12 | Soak server did not bind within SoakReadySecs | Increase `-SoakReadySecs 40` if server is slow to start |
| Bevy window opens but RPC never binds (exit 3) | `CCGS_AUTOPLAY=1` not set or `autoplay-remote` feature missing | Run-AutoplaySmoke.ps1 sets these automatically; check `process.log` for `Listening on 127.0.0.1:15873` |
| Missing checkpoint `lobby-confirmed` | Wrong button coordinates | Set `$env:CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN = "0.50,0.85"` (adjust Y) and retry |
| Checkpoint stops at `placement-submitted`, no `resolution-started` | Bot placement did not arrive in time | Increase `CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS`; check `server.log` for errors |
| `validate_composite_run.py` exits 1 | Schema / checkpoint mismatch | Read validator output; `WARN: ARTIFACT DIR NOT FOUND` = known warn, not fail |

---

1668: BOT-ROOM-AC7-AUTOPLAY-GUI-SMOKE-OPERATOR-PACK: SHIPPED
