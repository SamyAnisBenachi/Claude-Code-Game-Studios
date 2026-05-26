# PROMPT 1644 — Autoplay-vs-Bot Composite Harness v1

**Date:** 2026-05-27
**Worker:** Claude Sonnet 4.6 (PROMPT-1644)
**Branch:** `prompt-1644-autoplay-vs-bot-composite-harness`
**Source-of-truth base:** `origin/main@e67a3488`

---

## Summary

Implemented the first-slice composite harness for autoplay-vs-bot QA
(`AUTOPLAY-VS-BOT-QA-001` Story 004). The harness coordinates the existing
`Start-BotVsBotSoak.ps1` server and `Run-AutoplaySmoke.ps1` client launchers
into a single evidence-producing run. No live GUI flow was executed in this
worker — the harness is a scaffold; live PASS remains a human-gate.

---

## Changed Files

| File | Status | Description |
|---|---|---|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | **NEW** | Composite PowerShell launcher |
| `docs/autoplay/autoplay-vs-bot-flow.md` | **NEW** | Flow documentation + operator quickstart |
| `reports/PROMPT-1644-autoplay-vs-bot-composite-harness-v1.md` | **NEW** | This report |

---

## Implementation Detail

### `tools/dev-launcher/Start-AutoplayVsBot.ps1`

- **Style:** Follows `Start-BotVsBotSoak.ps1` exactly — `[CmdletBinding()]`,
  `Set-StrictMode -Version Latest`, `$ErrorActionPreference = 'Stop'`,
  `Write-Section` helper, per-section ordering.
- **PS5.1 compatibility:** Uses `[DateTime]::UtcNow` (not `Get-Date -AsUTC`).
- **Conservative BLOCKED exits:**
  - `10 BLOCKED-HUMAN-GUI` — non-interactive session detected via
    `[Environment]::UserInteractive`.
  - `11 BLOCKED-PRECONDITION` — required child scripts missing, or
    workspace Cargo.toml absent.
  - `12 BLOCKED-PRECONDITION` — soak server did not bind within
    `-SoakReadySecs`.
- **Port handling:** Identical auto-bump logic to `Start-BotVsBotSoak.ps1`;
  honours `-StrictPort`.
- **Soak launch:** Background PowerShell job (not `Start-Process -Wait`) so
  the composite launcher can poll port bind without blocking forever.
- **Env vars set for child:** `CCGS_AUTOPLAY_BOT_ROOM_READY=1`,
  `SERVER_PORT`, `SERVER_URL`.
- **Evidence:** `production/qa/evidence/composite-runs/<UTC>-autoplay-vs-bot/`
  with `composite-summary.json` (schema `autoplay_vs_bot_composite_summary_v1`)
  and `autoplay-run-path.txt` referencing the autoplay artifact dir.
- **live_pass_status field:** Always `NOT-CLAIMED` — explicit non-claim
  embedded in every composite-summary.json.
- **-DryRun:** All process launches skipped; all paths printed; evidence dir
  not created.
- **Scope:** Does NOT build Rust, does NOT run tests/story-done/smoke gate,
  does NOT touch `production/session-state/**`, sprints, or tracker files.

### `docs/autoplay/autoplay-vs-bot-flow.md`

- ASCII flow diagram of the composite orchestration.
- Prerequisites table, quickstart examples, parameter reference,
  evidence output layout, exit code table.
- Live PASS Gate section explicitly states the 5-step human sign-off
  process required to close GAP-01/GAP-02 and advance Story 004 to Done.
- Relation-to-other-launchers table for cross-reference.

---

## Validation

| Check | Result |
|---|---|
| PowerShell static parse (`[Parser]::ParseFile`) | **PASS — 0 errors** |
| `git diff --check` (whitespace) | **PASS — no whitespace errors** (CRLF conversion warnings are expected on Windows) |
| Markdown path references | **PASS — all referenced paths exist or are documented as future** |
| No broad Cargo | **PASS — no `cargo` invocations in this worker** |
| No GUI launched | **PASS — DryRun not run; no Bevy window opened** |
| Owned scope respected | **PASS — no edits to recipes/**, Rust source, Cargo files, session-state, sprints** |

---

## Precondition Gaps Addressed

| Gap | Before | After |
|---|---|---|
| GAP-09 | `Start-AutoplayVsBot.ps1` missing | **Scaffold implemented** |

GAP-01 (live GUI smoke) and GAP-02 (full-game composite run) remain
**OPEN — HUMAN-GATE**. This harness provides the runner; a human operator
must execute it and sign off evidence to close those gaps.

---

## Explicit Non-Claim

> **Live PASS for `AUTOPLAY-VS-BOT-QA-001` is NOT claimed.**
> No GUI was launched in this worker. The composite harness is a scaffold.
> Story 004 remains in `Draft` status. Closing the live PASS gate requires
> an operator to run `Start-AutoplayVsBot.ps1` in an interactive session,
> verify `outcome: ok` in `composite-summary.json`, confirm
> `full-game-resolution` in `checkpoints.jsonl`, and attach evidence.

---

1644: AUTOPLAY-VS-BOT-COMPOSITE-HARNESS-V1: SHIPPED
