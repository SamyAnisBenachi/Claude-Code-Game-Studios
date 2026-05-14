# Epic: Server (Operational Hardening)

> **Layer**: Server / Operational
> **GDD**: design/gdd/network-protocol.md, design/gdd/card-data-pool.md, design/gdd/round-state-machine.md (cross-cut)
> **Architecture Module**: `server/src/`, `server/src/game/`, `server/src/network/`, `server/src/main.rs`
> **Status**: Draft -- Sprint 13 candidate index for server-side operational
> hardening rows that are too narrow to fit a single-system epic and have no
> GDD-mechanic owner
> **Stories**: 2 Sprint 13 candidate stories (Sprint 12 close-out deferrals);
> NOT activated

## Overview

This epic is a thin index for server-side operational hardening rows that
do not fit cleanly under any existing system epic (auction, combat,
card-data-pool, etc.) and are too narrow to warrant a full-system epic of
their own. The first two rows are Sprint 12 close-out deferrals carried
into Sprint 13 planning:

- `S11-SERVER-POOL-INIT-LOG-GUARD-001` -- `server::game::init_pool` info
  log emits before the initialization guard fires; gate log emission on
  the guard to bring cold-path log volume back into budget.
- `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001` -- intermittent
  `Phase::Placement` round-2 runtime crash audit; **audit only, no fix
  lands**. If a repro is captured during Sprint 13, a follow-on story is
  authored with the precise repro.

Neither story claims release readiness, full game completion, or any
condition closure (`S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
`PAW-TD-*-a`). PROMPT 761 Polish->Release gate-check `FAIL` evidence
remains preserved.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Server owns all authoritative state; operational changes do not weaken this | HIGH |
| [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md) | `server/`, `shared/`, `client/` boundaries preserved | MEDIUM |
| [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Operational changes do not alter channel routing | HIGH |
| [ADR-009: Round State Machine](../../../docs/architecture/adr-009-round-state-machine.md) | `Phase::Placement` round-2 transition semantics unchanged by these stories | HIGH |

## Requirements

| Source | Requirement |
|--------|-------------|
| Sprint 12 close-out (PROMPT 817) `sprint_12_closeout.deferred_into_sprint_13_planning` | `S11-SERVER-POOL-INIT-LOG-GUARD-001` and `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001` deferred from Sprint 12 Should Have / Nice to Have |
| Sprint 11 Wave 12 backlog (parallel to W5 `ee27fb6` `acquisition_tick` fix) | Pattern for log-emit-before-guard fix |
| Sprint 11 Wave 12 backlog (12:07 R2 capture; not reproduced 13:28) | Intermittent runtime crash audit candidate |

## Scope

### In Scope

- Narrow operational fixes that gate log emission on the relevant
  guard.
- Audit-only diagnostics around `Phase::Placement` round-2 transitions
  that may surface a repro.
- Story-authored evidence documents under
  `production/qa/evidence/sprint-13-*/`.

### Out of Scope

- Any change to server authority model or round-state-machine semantics.
- Any change to placement validation logic.
- Any fix for the R2 Placement intermittent crash without a captured
  repro (audit only).
- Release-scope claims, manual-QA closure, accessibility completion,
  playtest evidence, final-art completion.
- `S8-QA-001-W1` closure, `QA-COND-0005` / `QA-COND-0006` closure,
  `PAW-TD-*-a` accept-risk closure.
- Polish->Release gate-check retry.

## Control Manifest Rules

- Server remains authoritative; no client-side authority is introduced.
- `shared/` remains protocol/data only.
- Log gating fixes use the same pattern as `ee27fb6` `acquisition_tick`
  fix (guard before emission).
- Audit stories emit diagnostic logs only; no behaviour change lands
  under an audit story.

## Dependency Map

| Dependency | Use |
|------------|-----|
| Card Data Pool | `init_pool` log lives in server card-pool init path |
| Round State Machine | `Phase::Placement` round-2 transition is the crash audit focus |
| Lightyear Channel Config | Unchanged by either story |

## Stories

| # | Story | Type | Status | Sprint 13 Slug |
|---|-------|------|--------|----------------|
| 001 | [Server `init_pool` Log Emit Before Guard](story-001-init-pool-log-guard.md) | Logic | Draft -- Sprint 13 candidate (Should Have), NOT activated | S11-SERVER-POOL-INIT-LOG-GUARD-001 |
| 002 | [R2 Placement Intermittent Runtime Crash Audit](story-002-r2-placement-crash-audit.md) | Audit / Diagnostic | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 |

## Definition of Done

- Each story passes `/story-readiness` against Sprint 13 activation
  HEAD before `/dev-story` is run.
- Each story's evidence document records its specific acceptance
  criteria with no-claim restatement preserved.
- No release-scope, manual-QA, accessibility, playtest, final-art, or
  S8-QA-001-W1 closure claims are made by either story.
