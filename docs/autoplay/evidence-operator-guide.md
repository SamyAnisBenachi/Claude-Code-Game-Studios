# Autoplay & Bot QA — Evidence Operator Guide

_Maintained doc — supersedes the scratch guide in `reports/PROMPT-1624-autoplay-evidence-operator-guide.md`._

---

## 1. Prerequisites

| Check | Command / Action |
|---|---|
| On latest `origin/main` | `git fetch origin && git status` |
| PowerShell 5.1+ | `$PSVersionTable.PSVersion` — Major ≥ 5 |
| Python 3.8+ on PATH | `python --version` |
| Interactive desktop session | A visible Windows desktop is required; the Bevy window must open |

> The launcher `Run-AutoplaySmoke.ps1` uses `[DateTime]::UtcNow` for timestamps
> (PS5.1-compatible since PROMPT 1619). Never use `Get-Date -AsUTC` in wrappers.

---

## 2. Running the Smoke Suite

### Quickstart

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1
```

Builds the client (`--features autoplay-remote`), launches it, waits for the RPC
port, runs the `smoke` recipe, then kills the client.

### Named recipe

```powershell
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe lobby-create
```

### Launcher parameters

| Parameter | Default | Purpose |
|---|---|---|
| `-Port` | `15873` | RPC TCP port |
| `-ArtifactDir` | auto: `production/qa/evidence/autoplay-runs/<TIMESTAMP>` | Override output folder |
| `-Python` | `python` | Python executable |
| `-DriverTicks` | `10` | Max driver ticks (0 = recipe-driven) |
| `-DriverHz` | `10.0` | Driver tick rate in Hz |
| `-ClientStartupSecs` | `60` | Seconds to wait for RPC port bind |

### Coordinate overrides

Recipes use fractional viewport coords (0.0–1.0). Override per-session via env vars:

```powershell
$env:CCGS_AUTOPLAY_LOBBY_CREATE_BTN  = "0.5,0.55"
$env:CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN = "0.5,0.85"
$env:CCGS_AUTOPLAY_CLASS_FIRST_CARD  = "0.25,0.45"
$env:CCGS_AUTOPLAY_CLASS_CONFIRM_BTN = "0.5,0.85"
$env:CCGS_AUTOPLAY_SHOP_FIRST_SLOT   = "0.2,0.4"
$env:CCGS_AUTOPLAY_SHOP_CONFIRM_BTN  = "0.5,0.85"
$env:CCGS_AUTOPLAY_AUCTION_BID_BTN   = "0.5,0.6"
$env:CCGS_AUTOPLAY_AUCTION_READY_BTN = "0.5,0.85"
$env:CCGS_AUTOPLAY_HAND_FIRST_CARD   = "0.15,0.85"
$env:CCGS_AUTOPLAY_BOARD_FIRST_CELL  = "0.5,0.5"
$env:CCGS_AUTOPLAY_SUBMIT_BTN        = "0.5,0.85"
```

Malformed values emit a `local.note` and fall back to defaults — they do not crash.

### Bot-vs-Bot full-game recipe

```powershell
# Terminal 1
pwsh -File tools/dev-launcher/Start-BotVsBotSoak.ps1

# Terminal 2 — once the soak server is ready
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe full-game
```

If `CCGS_AUTOPLAY_BOT_ROOM_READY` is unset, the recipe emits `local.block` and
the driver exits with code 4 — expected behaviour.

---

## 3. Evidence Output Layout

```
production/qa/evidence/autoplay-runs/
└── YYYYMMDD-HHMMSS-Z/          ← UTC timestamp, always -Z suffix
    ├── launcher-status.json     ← overall outcome + exit codes  ← START HERE
    ├── driver.log               ← human-readable progress log
    ├── driver-timeline.jsonl    ← one JSON row per driver tick
    ├── checkpoints.jsonl        ← phase-gate entries
    ├── capabilities.json        ← RPC capability probe at startup
    ├── process.log              ← client stdout + stderr
    └── screenshots/
        ├── 001.png
        ├── 001.json             ← sidecar: { requested_at, captured_at, reason }
        └── …
```

| File | When to read | What it tells you |
|---|---|---|
| `launcher-status.json` | Always — start here | Top-level verdict, exit codes for client + driver |
| `checkpoints.jsonl` | After seeing exit code | Which phases completed; where a block/note landed |
| `screenshots/` | After checkpoints | Visual state at each phase |
| `driver.log` | When a phase failed | Timestamped RPC call sequence |
| `driver-timeline.jsonl` | Deep debugging only | Every tick; cross-reference with checkpoints |
| `process.log` | Client crash / startup failure | Raw Bevy output; look for `ERROR` or `panicked at` |
| `capabilities.json` | Startup failures | What RPC surface the client offered |

---

## 4. Interpreting PASS / FAIL / BLOCKED

### Launcher exit codes

| Code | Meaning | Verdict |
|---|---|---|
| `0` | Recipe completed cleanly | **PASS** |
| `2` (launcher) | `cargo build` / `cargo run` failed | **FAIL** — build issue |
| `3` (launcher) | Client never bound RPC port | **FAIL** — Bevy startup stall |
| `1` (driver) | RPC error during run | **FAIL** — runtime issue |
| `2` (driver) | RPC server never answered within startup grace | **FAIL** — client launch issue |
| `4` (driver) | Recipe emitted `local.block` | **BLOCKED** — upstream prerequisite unmet |

> Check `launcher-status.json` → `driver_exit_code` **and** `client_exit_code` together.
> A non-zero `client_exit_code` with zero `driver_exit_code` = client crashed after
> recipe finished — inspect `process.log`.

### Phase checkpoints

Each recipe defines named phase gates:

| Checkpoint | Recipe | Meaning |
|---|---|---|
| `lobby-loaded` | `lobby-create` | Lobby screen interactive |
| `lobby-confirmed` | `lobby-create` | Confirm CTA clicked |
| `class-select-loaded` | `class-select` | Class selection rendered |
| `class-confirmed` | `class-select` | First card selected + Confirm clicked |
| `shop-loaded` | `draft-auction-probe` | Shop phase mounted |
| `shop-slot-clicked` | `draft-auction-probe` | First slot received click |
| `auction-loaded` | `draft-auction-probe` | Auction phase mounted |
| `auction-ready` | `draft-auction-probe` | Ready button clicked |
| `placement-loaded` | `placement-drag-probe` | Placement board appeared |
| `placement-dragged` | `placement-drag-probe` | Drag from hand to board completed |
| `placement-submitted` | `placement-drag-probe` | Submit button clicked |
| `full-game-resolution` | `full-game` | Resolution phase reached end |

**Reading `checkpoints.jsonl`:**

```jsonl
{"tick":5,"kind":"checkpoint","label":"lobby-loaded","elapsed_secs":2.15,"screenshot":true}
{"tick":12,"kind":"checkpoint","label":"lobby-confirmed","elapsed_secs":4.81}
{"tick":20,"kind":"note","label":"coord-override-parse-ok","elapsed_secs":7.00}
{"tick":30,"kind":"block","label":"missing-bot-room-ready","elapsed_secs":11.2}
```

- `checkpoint` → phase gate reached; screenshot in `screenshots/`
- `note` → informational annotation
- `block` → recipe stopped; driver exits 4

**Missing checkpoint = failure.** If `lobby-confirmed` never appears after
`lobby-loaded`, the Confirm button click did not register.

### Build phase diagnostics

Open `process.log`, search for `error[E`:

- Zero matches = build passed (warnings are expected)
- Any `error[` line → FAIL; launcher exits 2

### RPC timeout diagnostics

`launcher-status.json` → `"outcome": "rpc_timeout"`:

- Check `process.log` for `Listening on 127.0.0.1:15873`
- Confirm `CCGS_AUTOPLAY=1` was set
- Confirm build used `--features autoplay-remote`

---

## 5. Observability Tools

### Bot debug overlay (F8)

**Enable:**
```powershell
$env:CCGS_BOT_DEBUG_UI = "1"   # server: enables S2CDebugBotStatePush messages
$env:CCGS_DEBUG_UI     = "1"   # client: enables the F8-toggled corner panel
```

Press **F8** in the client window. The top-right panel shows the bot's hand,
last N decision log entries, and `last_bid_valuation`. Server pushes at 500 ms
intervals. Read-only — does not affect game state.

Autoplay does not press F8 automatically. For a combined run, toggle the overlay
manually before starting the recipe.

### Bot decision logs

**Enable:**
```powershell
$env:CCGS_BOT_QA_SNAPSHOT = "1"
```

Output: JSONL file of `BotDecisionLog` entries in
`production/qa/evidence/dev-runs/`. After a `full-game` run, cross-reference
decision log tick timestamps with `checkpoints.jsonl` to align bot actions with
driven-client phase state.

### QA snapshot (F9 / in-game button)

**Enable:**
```powershell
$env:CCGS_QA_SNAPSHOT = "1"
```

Usage options:
1. **Manual:** Press **F9** — captures `snapshot.json` + screenshot to `production/qa/evidence/`
2. **In-game button:** Visible when `CCGS_QA_SNAPSHOT=1`
3. **Recipe-triggered:** Add `autoplay/input --keys-down F9` to a custom recipe step

Snapshots provide structured ECS state; autoplay screenshots provide visual state.
Use both for comprehensive checkpoint evidence.

---

## 6. Human Operator Checklist

### Bot-vs-Bot QA (human watching)

1. Launch `Start-BotVsBotSoak.ps1` (terminal 1) with `CCGS_BOT_DEBUG_UI=1` and `CCGS_BOT_QA_SNAPSHOT=1`
2. Launch the client with `CCGS_DEBUG_UI=1` and `CCGS_QA_SNAPSHOT=1`
3. **Watch — do not interact.** Bots drive all phases
4. Press **F8** at any time to toggle the bot overlay
5. Press **F9** (or in-game button) to capture snapshots
6. After the round, review `production/qa/evidence/dev-runs/`

### Autoplay QA (recipe-driven client)

1. Close all other Bevy windows
2. Run `pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe <name>`
3. **Do not click in the client window** while the recipe runs — human input races with recipe input
4. Watch passively; recipe progress appears in the terminal
5. On exit: read `launcher-status.json` → `checkpoints.jsonl` → screenshot pairs
6. Wrong coordinates? Set the relevant `CCGS_AUTOPLAY_*` env var and re-run

### After any run

- Archive the run directory if you need to keep it (`autoplay-runs/` is not gitignored)
- Attach `launcher-status.json` + screenshot pairs to the PROMPT report as evidence
- If BLOCKED for a legitimate reason, note the `local.block` label so reviewers understand

---

## 7. Recipe Quick-Reference

| Recipe | Phase gates | Requires | Notes |
|---|---|---|---|
| `smoke` | — | — | Proves RPC infra only |
| `idle` | — | — | Status-polling soak |
| `lobby-create` | `lobby-loaded`, `lobby-confirmed` | — | |
| `class-select` | `class-select-loaded`, `class-confirmed` | — | |
| `draft-auction-probe` | `shop-loaded`, `shop-slot-clicked`, `auction-loaded`, `auction-ready` | — | |
| `placement-drag-probe` | `placement-loaded`, `placement-dragged`, `placement-submitted` | — | |
| `full-game` | all above + `full-game-resolution` | `CCGS_AUTOPLAY_BOT_ROOM_READY=1` | Complete loop; needs bot peer |

---

## 8. Common Failures

| Symptom | Likely cause | Fix |
|---|---|---|
| Launcher exits 2 immediately | Build failed | Run `cargo check -p client --features autoplay-remote` |
| Launcher exits 3 (RPC timeout) | Client opened but RPC port never bound | Check `process.log` for `Listening on 127.0.0.1:15873`; confirm `CCGS_AUTOPLAY=1` |
| Driver exits 2 (startup grace) | Client built but RPC not started | Same as above; verify `autoplay-remote` feature in `Cargo.toml` |
| Driver exits 4 with `full-game` | `CCGS_AUTOPLAY_BOT_ROOM_READY` unset | Set the var or launch `Start-BotVsBotSoak.ps1` first |
| Checkpoint missing (e.g., no `lobby-confirmed`) | Wrong button coordinates | Set `CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN` to correct fractional position |
| `local.note` about parse failure in checkpoints | Malformed `CCGS_AUTOPLAY_*` env var | Fix format: `"fx,fy"` with floats in [0.0, 1.0] |
| Artifact directory has garbled timestamp | PS < 5.1 or stale launcher | Confirm `$PSVersionTable.PSVersion.Major` ≥ 5 and latest `origin/main` |
| Client crashes after recipe completes | Bevy panic post-run | Check `process.log` for `panicked at`; separate from autoplay infra |

---

_Last updated: PROMPT 1637 — 2026-05-27 (materialized from PROMPT 1624 scratch guide)_
