# Live GUI Autoplay Smoke — Result Template

Copy this file to `production/qa/evidence/autoplay-runs/<run-stamp>/RESULT.md` and
fill each section immediately after your run. Fields marked `<!-- fill -->` are
required; fields marked `<!-- optional -->` help reviewers but may be left blank.

> For how to run the suite and interpret exit codes, see
> [evidence-operator-guide.md](evidence-operator-guide.md).

---

## Run Metadata

| Field | Value |
|---|---|
| **Date (UTC)** | <!-- fill: YYYY-MM-DD HH:MM UTC --> |
| **Operator** | <!-- fill: your name / GH handle --> |
| **Artifact directory** | <!-- fill: production/qa/evidence/autoplay-runs/YYYYMMDD-HHMMSS-Z/ --> |
| **Baseline commit** | <!-- fill: git rev-parse --short HEAD --> |
| **Branch / worktree** | <!-- fill: branch name or "main" --> |

---

## Command

```powershell
<!-- fill: exact command(s) run, e.g. -->
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Recipe full-game
```

---

## Environment Variables

List every `CCGS_*` or other relevant env var that was set for this run.
Leave blank if none were set beyond defaults.

```powershell
<!-- fill: e.g.
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
$env:CCGS_BOT_DEBUG_UI            = "1"
$env:CCGS_BOT_QA_SNAPSHOT         = "1"
$env:CCGS_QA_SNAPSHOT             = "1"
-->
```

---

## launcher-status.json Summary

Paste the full JSON or the most relevant fields:

```json
<!-- fill: e.g.
{
  "outcome": "success",
  "launcher_exit_code": 0,
  "driver_exit_code": 0,
  "client_exit_code": 0,
  "started_at": "2026-05-27T...",
  "finished_at": "2026-05-27T...",
  "recipe": "full-game"
}
-->
```

---

## Recipe Checkpoint Log

Paste the contents of `checkpoints.jsonl`, or a filtered subset of the key
`checkpoint` and `block` entries. Omit `note` entries if the log is large.

```jsonl
<!-- fill: e.g.
{"tick":5,"kind":"checkpoint","label":"lobby-loaded","elapsed_secs":2.15,"screenshot":true}
{"tick":12,"kind":"checkpoint","label":"lobby-confirmed","elapsed_secs":4.81,"screenshot":true}
...
-->
```

| Checkpoint | Present? | Screenshot? | Notes |
|---|---|---|---|
| `lobby-loaded` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `lobby-confirmed` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `class-select-loaded` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `class-confirmed` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `shop-loaded` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `shop-slot-clicked` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `auction-loaded` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `auction-ready` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `placement-loaded` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `placement-dragged` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `placement-submitted` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |
| `full-game-resolution` | <!-- ✓ / ✗ / N/A --> | <!-- ✓ / ✗ --> | |

---

## Screenshots

List each screenshot captured (from `screenshots/`) with a brief description
of what it shows. Attach / embed images if the report is shared in a wiki or
PR comment.

| File | Checkpoint | What it shows | Anomalies |
|---|---|---|---|
| `screenshots/001.png` | <!-- fill --> | <!-- fill --> | <!-- fill or "none" --> |
| `screenshots/002.png` | <!-- fill --> | <!-- fill --> | <!-- fill or "none" --> |
| <!-- add rows as needed --> | | | |

---

## Bot Decision Log

<!-- optional — only relevant for full-game or bot-vs-bot runs -->

| Field | Value |
|---|---|
| **Log file path** | <!-- fill: production/qa/evidence/dev-runs/bot-decision-YYYYMMDD-HHMMSS.jsonl --> |
| **Total entries** | <!-- fill: wc -l on the file --> |
| **Auction decisions look sensible?** | <!-- Yes / No / Not checked --> |
| **Placement decisions look sensible?** | <!-- Yes / No / Not checked --> |
| **Notable log entries** | <!-- paste any suspicious or interesting lines --> |

---

## QA Snapshot

<!-- optional — only relevant if CCGS_QA_SNAPSHOT=1 was set -->

| Field | Value |
|---|---|
| **Snapshot file** | <!-- fill: production/qa/evidence/snapshot-YYYYMMDD-HHMMSS.json --> |
| **Screenshot paired?** | <!-- Yes / No --> |
| **Phase at capture** | <!-- fill: e.g. "auction phase, round 2" --> |
| **ECS state looks correct?** | <!-- Yes / No + brief note --> |

---

## Bot Debug Overlay (F8)

<!-- optional — only relevant if CCGS_BOT_DEBUG_UI=1 was set -->

| Field | Value |
|---|---|
| **Overlay visible?** | <!-- Yes / No --> |
| **Hand entries shown correctly?** | <!-- Yes / No / Not checked --> |
| **`last_bid_valuation` appeared?** | <!-- Yes / No / Not checked --> |
| **Notes** | <!-- any visual anomalies in the overlay panel --> |

---

## Verdict

<!-- fill: choose exactly one -->

- [ ] **PASS** — all expected checkpoints reached; all screenshots show correct UI; no anomalies
- [ ] **FAIL** — at least one checkpoint missing or launcher/driver exit code non-zero
- [ ] **BLOCKED** — recipe emitted `local.block`; upstream prerequisite not met (expected)
- [ ] **PARTIAL** — some checkpoints reached but run did not complete cleanly

**Exit codes:**

| Component | Exit code | Expected |
|---|---|---|
| Launcher | <!-- fill --> | `0` for PASS |
| Driver | <!-- fill --> | `0` for PASS, `4` for BLOCKED |
| Client | <!-- fill --> | `0` for clean exit |

**One-line verdict:**

```
<!-- fill: e.g. "PASS — full-game recipe reached full-game-resolution on first run" -->
<!-- fill: e.g. "FAIL — lobby-confirmed missing; Confirm button coordinate wrong" -->
<!-- fill: e.g. "BLOCKED — CCGS_AUTOPLAY_BOT_ROOM_READY not set; expected" -->
```

---

## Failure Analysis

<!-- fill only if verdict is FAIL -->

| Field | Value |
|---|---|
| **First missing checkpoint** | <!-- fill --> |
| **Relevant log excerpt** | <!-- paste from driver.log or process.log --> |
| **Root cause (hypothesis)** | <!-- fill --> |
| **Coordinate override needed?** | <!-- Yes / No → which env var? --> |
| **Build error?** | <!-- paste first `error[E` line from process.log if any --> |

---

## Follow-Up Prompts / Actions

List any follow-up work surfaced by this run. Reference the ORCHESTRATOR-QUEUE
or spawn a PROMPT for each item.

| # | Action | Priority | PROMPT # |
|---|---|---|---|
| 1 | <!-- fill: e.g. "Fix lobby-confirm coordinate; CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN wrong" --> | <!-- P1/P2/P3 --> | <!-- if known --> |
| 2 | | | |
| <!-- add rows as needed --> | | | |

If no follow-up actions: `None — run clean.`

---

_Template version: PROMPT 1643 — 2026-05-27_
_Guide: [evidence-operator-guide.md](evidence-operator-guide.md)_
