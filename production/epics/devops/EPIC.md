# Epic: DevOps (Operational Hardening)

> **Layer**: DevOps / Build / Tooling / Documentation
> **GDD**: N/A (operational infra; no mechanic GDD)
> **Architecture Module**: `Cargo.toml` (workspace + members), `docs/setup/`,
> `docs/architecture/` (operational notes), `.octogent/` (orchestrator docs)
> **Status**: Draft -- Sprint 13 candidate index for DevOps operational
> rows; NOT activated. Story 005 closed on `origin/main` per PROMPT 888
> (2026-05-15). Story 006 added 2026-05-17 by PROMPT 1057 as a Sprint 16
> candidate (NOT activated).
> **Stories**: 5 Sprint 13 candidate stories (Sprint 12 close-out deferrals
> + Windows AppCompat note) + 1 Sprint 16 candidate story (AppCompat
> manifest follow-on); Sprint 13 stories activated by PROMPT 826; Sprint
> 16 NOT activated.

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
  (informational from TQ-S12-C7). **DONE** on `origin/main` per PROMPT
  888 (2026-05-15).
- `S15-OPS-APPCOMPAT-MANIFEST-001` -- **DONE** on `origin/main` per
  PROMPT 1072 (`/story-done` 2026-05-17 on `origin/main@bd374dd`).
  Activated in Sprint 16 by PROMPT 1064. Implementation via Mechanism
  (d) Cargo `[[test]] name` rename
  `spawn_range_live_update_contract -> spawn_range_live_refresh_contract`
  (PROMPT 1068 worker `ed58e3d`, integrated PROMPT 1071 `488a9cd`,
  carried forward by PROMPT 1070 `bd374dd`). Source file
  `spawn_range_live_update_contract_test.rs` NOT renamed. 5 consecutive
  cargo runs without rename workaround / no `os error 740` recorded
  in `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`.

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
| Sprint 14 PROMPT 983 smoke rerun §"Windows AppCompat Workaround" Option B | Per-run rename workaround used at smoke time; manifest-embed candidate filed as Sprint 15 deferred and pulled into Sprint 16 candidate (Story 006). |

## Scope

### In Scope

- Documentation-tier notes under `docs/setup/`, `docs/architecture/`,
  `.octogent/`.
- Investigation notes documenting trade-offs and recommending a
  single follow-on story per investigation.
- (Sprint 16 candidate, Story 006 only) A bounded Cargo test-target
  configuration change embedding a Windows `asInvoker` manifest (or
  equivalent robust mechanism) on the
  `spawn_range_live_update_contract` test binary. Scope limited to
  `shared/Cargo.toml` + optional `shared/build.rs` + optional
  manifest XML file; no production-source change under `client/`,
  `server/`, `shared/src/`.

### Out of Scope

- Build-script changes, profile changes, CI workflow changes, or
  tooling installation in this sprint (except the Sprint 16 candidate
  Story 006 bounded Cargo test-target configuration change above,
  which is the explicit exception and is itself NOT activated by the
  story-authoring run).
- Production-source changes under `client/`, `server/`, `shared/`,
  `tests/`.
- Polish->Release gate-check retry.
- `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`
  closure claims.

## Control Manifest Rules

- Investigation / documentation stories land notes only; no code lands.
- Each investigation story names exactly one recommended follow-on
  story (or explicitly defers naming until evidence is captured).
- (Story 006 only) Implementation is scoped to one Cargo test target's
  manifest configuration; production-source paths under `client/`,
  `server/`, `shared/src/`, and the
  `spawn_range_live_update_contract_test.rs` source file are
  off-limits. Any new Cargo build dependency is gated behind
  `cfg(target_os = "windows")` (or nearest Cargo equivalent).

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
| 006 | [Windows AppCompat Manifest for `spawn_range_live_update_contract` Test Binary](story-006-appcompat-manifest.md) | Implementation / ops hygiene -- bounded Cargo test-target configuration change | Done -- Sprint 16 Nice to Have (closed PROMPT 1072 on `origin/main@bd374dd`; PROMPT 1068 worker `ed58e3d` + PROMPT 1071 integration `488a9cd`) | S15-OPS-APPCOMPAT-MANIFEST-001 |

## Definition of Done

- Each story passes `/story-readiness` against the relevant sprint
  activation HEAD (Sprint 13 for stories 001-005; Sprint 16 for story
  006) before `/dev-story` is run.
- Each story 001-005 note artifact is authored at the named path and
  records the trade-offs and recommended follow-on (if any).
- For stories 001-005: no build / tooling / CI / production-source
  change lands.
- For story 006: a bounded Cargo test-target configuration change
  lands (one `[[test]]` block plus optionally one `build.rs` and one
  manifest XML file), with zero touch under `client/`, `server/`,
  `shared/src/`, or
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
