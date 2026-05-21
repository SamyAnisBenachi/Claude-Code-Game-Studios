# Epic: Bot & Autoplay (QA Automation Substrate)

> **Layer**: Polish / Cross-Cutting QA Automation
> **GDD**: N/A (operational QA automation; no mechanic GDD owns these rows)
> **Architecture Module**: `client/src/autoplay.rs`, `tools/autoplay/**`,
> `docs/autoplay/**`, `.claude/skills/ccgs-autoplay/**`,
> `server/src/game/bot/**` (server-side bot decision/action path),
> bot-room protocol surfaces shared with `playable-client` epic
> **Status**: Draft -- ledger placeholder for bot + autoplay workstreams that
> have been landing on `origin/main` without a formal story owner since
> PROMPT 1430 / 1439 / 1531 / 1582 / 1583 / 1595 / 1601 / 1602 / 1603.
> NOT activated. Sprint 18 active set NOT modified by this epic's creation.
> **Stories**: 5 candidate stories (1 Sprint 18 carry-tracking, 4 Sprint 19
> candidates). NOT activated.

## Overview

This epic is a thin ledger index for the QA-automation substrate that has
been growing on `origin/main` outside any system epic. Two workstreams
share the substrate:

1. **Bot** -- a server-driven autonomous opponent that joins a real game
   room as a participant, picks a class, bids in auctions, places units,
   and emits decision telemetry. Foundations landed via PROMPT 1430
   (protocol/room foundations), PROMPT 1439 (foundation scaffold
   main-land), PROMPT 1531 / 1582 (participant action loop Waves 1 + 2),
   PROMPT 1583 (lobby ready auto-confirm), PROMPT 1602 (Wave 3 placement
   heuristic, currently main-tip at `origin/main@576fbe8c`). Server-side
   QA snapshot + streamed bot-decision log shipped via PROMPT 1597.
2. **Autoplay** -- a client-side, real-UI input automation harness that
   drives the primary client through real Bevy / Lightyear flows the same
   way a human operator would. NOT semantic game-state mutation. NOT
   server-side. Bootstrap landed via PROMPT 1595 (first slice) and
   PROMPT 1601 (main-ready refresh: `client/src/autoplay.rs`,
   project-local skill `skills/ccgs-autoplay`, `tools/autoplay/**`,
   docs). Docs refresh via PROMPT 1606.

The two surfaces are deliberately separate but composable: a bot can sit
on one side of a friend-game room while autoplay drives the other side
through the real client. The "Autoplay-vs-Bot" QA flow (story 004 in this
epic) is the planned consumer of both surfaces.

This epic is a **tracking ledger** -- it captures what is on `origin/main`
already, what is unimplemented, and what should be sequenced into Sprint
19 planning. It does NOT close any landed work, NOT advance stage, NOT
activate Sprint 19, and NOT modify the Sprint 18 active row set.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Bot decisions are server-authoritative; autoplay only emits the same C2S intents a human would | HIGH |
| [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md) | Bot logic lives under `server/`; autoplay lives under `client/`; no client → server import crosses the boundary | MEDIUM |
| [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Bot/autoplay traffic uses the same reliable/unreliable channels as a human participant | HIGH |
| [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md) | Bot-occupied rooms reach `SessionReady` through the same Observer path as human rooms | HIGH |

## Requirements

| Source | Requirement |
|--------|-------------|
| PROMPT 1594 followup bot-flow inventory | 10 missing bot-flow items, recommended ordering A..H; this epic captures the highest-leverage items |
| PROMPT 1601 main-land | Autoplay bootstrap substrate on `origin/main`: `client/src/autoplay.rs`, `tools/autoplay/**`, `docs/autoplay/**`, `skills/ccgs-autoplay` |
| PROMPT 1602 main-land | Bot Wave 3 placement heuristic on `origin/main@576fbe8c` |
| PROMPT 1603 worker | Bot-vs-bot soak entrypoint worker shipped; integration refresh PROMPT 1607 active, not yet confirmed main-landed |
| PROMPT 1604 data contract | Debug-only bot overlay data-push path defined; implementation deferred until 1602/1603 land |
| Sprint 18 plan §4 Conditional / Not Yet Landed Inputs | Pattern of recording out-of-sprint workstreams as ledger entries without activating them |

## Scope

### In Scope

- Tracking-only story files for the 5 workstreams below.
- One epic index row in `production/epics/index.md`.
- Sprint placement annotations under Sprint 18 §4 (Conditional / Not Yet
  Landed Inputs) and one orchestrator-state note. NO Sprint 19 file is
  created or activated.

### Out of Scope

- Any source-code change under `client/`, `server/`, `shared/`, `tests/`.
- Any Cargo / Trunk / CI invocation.
- Any modification of `production/stage.txt`, `production/qa/**`,
  `production/gate-checks/**`, or release artifacts.
- Closing PROMPT 1430 / 1439 / 1531 / 1582 / 1583 / 1595 / 1601 / 1602 /
  1603 / 1606 work as "done" against new Sprint 18 / 19 rows; those
  landed against orchestrator PROMPTs and are tracked there.
- Activating Sprint 19 or any of these candidate rows.
- Polish→Release gate-check retry.
- `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`
  closure claims.

## Control Manifest Rules

- Ledger-only authoring at this epic-creation step; no implementation
  lands here.
- Sprint 18 row set is NOT modified by this epic's creation; the active
  4 Must Have + 6 Should Have + 2 Nice to Have rows in
  `production/sprints/sprint-18.md` §2.1 / §2.2 / §2.3 are preserved
  verbatim.
- Bot decisions remain server-authoritative; autoplay never short-circuits
  C2S → S2C through direct state mutation.
- Debug-only surfaces (story 005 overlay) stay gated behind a dev/debug
  env or feature flag and never ship in a release build.

## Stories

| # | Story | Type | Status | Sprint Placement |
|---|---|---|---|---|
| 001 | [Bot Room Participant -- Join + Class Confirm + Action Loop Refresh](story-001-bot-room-participant.md) | Integration | Draft -- ledger placeholder for landed work | Sprint 18 carry-tracking (already implemented on `origin/main` via PROMPT 1430 / 1439 / 1531 / 1582 / 1583 / 1602; story-done paperwork deferred to Sprint 19) |
| 002 | [Bot-vs-Bot Soak Entrypoint + QA Harness](story-002-bot-vs-bot-soak-entrypoint.md) | Integration | Draft -- Sprint 19 candidate | Sprint 19 candidate (worker shipped via PROMPT 1603; integration PROMPT 1607 active) |
| 003 | [Autoplay Full-Game Recipe Library v1 (Real UI Input)](story-003-autoplay-recipe-library-v1.md) | Integration | Draft -- Sprint 19 candidate | Sprint 19 candidate (autoplay bootstrap on `origin/main` via PROMPT 1601; recipe library extension is the next slice) |
| 004 | [Autoplay-vs-Bot QA Flow](story-004-autoplay-vs-bot-qa-flow.md) | Integration | Draft -- Sprint 19 candidate (gated on 001 + 002 + 003) | Sprint 19 candidate (depends on 001/002/003) |
| 005 | [Debug-Only Bot Overlay -- Data Push Path](story-005-bot-debug-overlay.md) | Integration | Draft -- Sprint 19 candidate | Sprint 19 candidate (data contract defined by PROMPT 1604; gated behind `CCGS_DEBUG_UI=1` and F8 per orchestrator note) |

## Non-Claims

- NO Sprint 19 activation.
- NO closure of any landed bot/autoplay PROMPT.
- NO release / RC / full-game / accessibility / playtest validation claim.
- NO stage advance from `Polish`.
- NO `Polish->Release` gate-check retry.
- NO modification of Sprint 18 active row set.
