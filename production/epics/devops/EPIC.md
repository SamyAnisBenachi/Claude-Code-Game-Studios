# Epic: DevOps (Operational Hardening)

> **Layer**: DevOps / Build / Tooling / Documentation
> **GDD**: N/A (operational infra; no mechanic GDD)
> **Architecture Module**: `Cargo.toml` (workspace + members), `docs/setup/`,
> `docs/architecture/` (operational notes), `.octogent/` (orchestrator docs)
> **Status**: Draft -- Sprint 13 candidate index for DevOps operational
> rows; NOT activated
> **Stories**: 5 Sprint 13 candidate stories (Sprint 12 close-out deferrals
> + Windows AppCompat note); NOT activated

## Overview

This epic is a thin index for DevOps / build / tooling / documentation
rows surfaced by Sprint 11 / Sprint 12 close-outs and by the PROMPT 815
disk-pressure invocation. All five rows are **documentation- or
investigation-tier**; none lands a build-script change, profile change,
tooling change, or production-source change in this sprint.

- `S11-TD-CARGO-DISK-USAGE-001` -- Cargo workspace disk-usage reduction
  strategy note; investigation only, no build-script change.
- `S11-TD-CARGO-PDB-LIMIT-001` -- Cargo PDB-size pressure investigation;
  recommend Windows `split-debuginfo` / `strip` profile knobs as a
  follow-on, no profile change here.
- `S11-OPS-ORCHESTRATOR-LOCK-001` -- orchestrator-root concurrent-session
  lock pattern documented at `.octogent/orchestrator-lock.md` (or
  equivalent); no code lands.
- `S11-OPS-GH-CLI-001` -- `gh` CLI installation note in
  `docs/setup/dev-environment.md`.
- `S13-OPS-WIN-APPCOMPAT-NOTE-001` -- Windows AppCompat heuristic +
  manifest/rename workaround note at `docs/setup/dev-environment.md`
  (informational from TQ-S12-C7).

Sprint 13 does **not** advance stage. PROMPT 761 Polish->Release
gate-check `FAIL` evidence is preserved. None of these stories closes
`S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or `PAW-TD-*-a`.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md) | `shared/`, `server/`, `client/` boundaries preserved by any workspace change | MEDIUM |

## Requirements

| Source | Requirement |
|--------|-------------|
| Sprint 12 close-out (PROMPT 817) `sprint_12_closeout.deferred_into_sprint_13_planning` | Four DevOps rows deferred from Sprint 12 Nice to Have / Should Have |
| PROMPT 815 Sprint 12 smoke disk-pressure invocation | Cleaned 25 GB + ~200 GB worker `target/` directories; re-affirms disk-usage strategy candidate |
| 2026-05-13 override rule "only one shared-status writer at a time" | Reinforces orchestrator-lock pattern candidate |
| TQ-S12-C7 informational (PROMPT 815/816/817 evidence) | Windows AppCompat heuristic on the substring `update` in `spawn_range_live_update_contract-*.exe` |

## Scope

### In Scope

- Documentation-tier notes under `docs/setup/`, `docs/architecture/`,
  `.octogent/`.
- Investigation notes documenting trade-offs and recommending a
  single follow-on story per investigation.

### Out of Scope

- Build-script changes, profile changes, CI workflow changes, or
  tooling installation in this sprint.
- Production-source changes under `client/`, `server/`, `shared/`,
  `tests/`.
- Polish->Release gate-check retry.
- `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`
  closure claims.

## Control Manifest Rules

- Investigation / documentation stories land notes only; no code lands.
- Each investigation story names exactly one recommended follow-on
  story (or explicitly defers naming until evidence is captured).

## Dependency Map

| Dependency | Use |
|------------|-----|
| Cargo workspace | Disk-usage and PDB-size investigations read the existing workspace state |
| Orchestrator runtime docs (`docs/octogent-integration.md` and `.octogent/`) | Lock pattern is documented alongside existing orchestrator docs |

## Stories

| # | Story | Type | Status | Sprint 13 Slug |
|---|-------|------|--------|----------------|
| 001 | [Cargo Workspace Disk-Usage Reduction Strategy](story-001-cargo-workspace-disk-usage.md) | Investigation -- doc only | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S11-TD-CARGO-DISK-USAGE-001 |
| 002 | [Cargo PDB-Size Pressure Investigation](story-002-cargo-pdb-limit.md) | Investigation -- doc only | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S11-TD-CARGO-PDB-LIMIT-001 |
| 003 | [Orchestrator-Root Concurrent-Session Lock Pattern](story-003-orchestrator-lock.md) | Documentation only | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S11-OPS-ORCHESTRATOR-LOCK-001 |
| 004 | [`gh` CLI Installation Note](story-004-gh-cli-setup.md) | Documentation only | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S11-OPS-GH-CLI-001 |
| 005 | [Windows AppCompat Heuristic + Workaround Note](story-005-win-appcompat-note.md) | Documentation only | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S13-OPS-WIN-APPCOMPAT-NOTE-001 |

## Definition of Done

- Each story passes `/story-readiness` against Sprint 13 activation
  HEAD before `/dev-story` is run.
- Each story's note artifact is authored at the named path and
  records the trade-offs and recommended follow-on (if any).
- No build / tooling / CI / production-source change lands under any
  of these stories.
