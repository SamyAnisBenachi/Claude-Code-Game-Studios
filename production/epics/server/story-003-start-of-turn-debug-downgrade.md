# Story 003: S17-SERVER-START-OF-TURN-DEBUG-001 -- `start_of_turn_dispatch_system` `warn!` -> `debug!` Downgrade

> **Epic**: Server (Operational Hardening)
> **Story ID**: S17-SERVER-START-OF-TURN-DEBUG-001
> **Status**: Draft -- Sprint 17 Nice to Have candidate; NOT activated by this authoring run
> **Layer**: Server / Operational -- log level downgrade for known-deferred keyword dispatch
> **Type**: Tech Debt / Ops Hygiene -- silence known-deferred WARN spam
> **Sprint**: Sprint 17 Nice to Have row per `production/sprints/sprint-17.md` §"Nice to Have". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.1d (single-line log level change in `server/src/game/` keyword dispatch path)
> **Source audit**: PROMPT 1076 `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md` §"Per-finding evidence" AUDIT-1076-15 (P3)

---

## Target Epic Justification

This story is filed under `production/epics/server/` rather than
`production/epics/round-state-machine/`. Justification:

- The `start_of_turn_dispatch_system` is in
  `server/src/game/` keyword dispatch path (per PROMPT 1076
  AUDIT-1076-15 minimal repair surface: "`server::game` keyword
  dispatch (deferred). Minimal repair surface: drop to `debug!`
  until implemented"). It is NOT in `server/src/core/rsm/`.
- The round-state-machine epic
  (`production/epics/round-state-machine/`) owns
  `server/src/core/rsm/state.rs` / `events.rs` / `transitions.rs`
  / `system.rs` / `plugin.rs` per its EPIC.md Architecture
  Module line. The keyword dispatch path is downstream of the
  RSM (in `server/src/game/`) and is owned by the Server
  Operational Hardening epic per its EPIC.md scope.
- The Server epic's EPIC.md §Overview explicitly scopes "server-
  side operational hardening rows that do not fit cleanly under
  any existing system epic ... and are too narrow to warrant a
  full-system epic of their own" — log-level adjustment for a
  deferred dispatch path is exactly this shape.
- Sprint 17 plan row source allows either epic ("Target epic:
  production/epics/server/ or production/epics/round-state-machine/
  if existing ownership points there. Choose one and justify.").
  The server epic is the natural ownership.

---

## Status / No-Claim Banner

This story is a Sprint 17 Nice to Have **candidate** authored by
PROMPT 1095. **No sprint is activated by this authoring run.**
PROMPT 1095 does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-17.md`, `production/sprints/sprint-16.md`,
`production/stage.txt`, any `production/session-state/*` file, any
QA-plan / smoke / Team-QA / gate-check / release-check artifact
under `production/qa/`, any code under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`,
`.github/`, or `Trunk.toml`. PROMPT 1095 does NOT run
`/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`,
`trunk`, or any CI command.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
validation (`QA-COND-0006`), full playable-client manual QA,
two-client GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-
production completion (`PAW-TD-*-a`), `Polish->Release` gate-check
retry, stage advance from Polish to Release, closure of the Sprint
12 story 019 underlying drag-runtime bug, closure of
`S11-HUD-TIMER-EYEBALL-VISUAL-001`, closure of any of the 24
PROMPT 1022 audit findings, closure of any SOURCE-1077-* finding,
or closure of any AUDIT-1076-* finding outside AUDIT-1076-15.

**This story does NOT implement `start_of_turn_dispatch_system`.**
The dispatch logic remains deferred; only the log level for the
"not yet implemented" message is changed. No new server-
authoritative state, no protocol change, no behaviour change.

Sprint 16 disposition preserved unchanged. Sprint 15 / 14 / 13 /
12 / 11 / 10 dispositions preserved unchanged. PROMPT 761 Polish
->Release gate-check `FAIL` preserved. `PAW-TD-*-a`,
`QA-COND-0005`, `QA-COND-0006`, `TQ-S12-C1..C7` preserved
verbatim.

---

## Source Finding

### AUDIT-1076-15 (P3) — `start_of_turn_dispatch_system not yet implemented` spam

- **Audit location**:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-15.
- **Severity**: P3.
- **Evidence**: server.log lines 67, 109, 161, 240, 293, 337 in
  run-7 (6 WARN lines per session, one per round entry).
- **Behaviour**: every round entry logs
  `start_of_turn_dispatch_system not yet implemented` at WARN
  level on the server.
- **Likely owner** (per audit): `server::game` keyword dispatch
  (deferred work).
- **Minimal repair surface** (audit recommendation): drop to
  `debug!` until the system is implemented.

---

## Problem Class / Prevention Target

**Defect class**: a known-deferred code path logs at WARN level,
emitting one log line per round per session. The line is benign
(deferred work; the keyword dispatch system has not been
implemented yet) but its WARN level makes real warnings harder
to spot in server logs.

**Prevention target**: downgrade the log level from `warn!` to
`debug!` so the message no longer appears in normal-server logs
but remains capturable when a developer raises the log level
filter to debug. The message text and the location of the call
site remain unchanged; only the macro changes.

---

## Context

### Existing surface

- **`server/src/game/mod.rs`** or **`server/src/game/dispatch*.rs`**
  (or similar — the implementing worker re-verifies the exact
  file at activation HEAD; the audit says "server::game keyword
  dispatch" without naming the exact file). The current call
  emits `warn!("start_of_turn_dispatch_system not yet
  implemented");` (or equivalent text).
- **Logging infrastructure**: the server uses Bevy's `tracing`
  facility (per Bevy 0.18 defaults). `debug!` macro is
  immediately available alongside `warn!`.
- **No related test**: no current test asserts the log line at a
  specific level. This story does NOT add a positive test for
  the log level downgrade; the BLOCKING gate is the Sprint 17
  smoke (AC2 below).

### GDD / ADR / TR trace

- **GDD**: not applicable; log-level hygiene is not GDD-spec'd.
- **ADR-009** (Round State Machine Phase State) and **ADR-010**
  (RSM Phase Event Bus): no change. The `start_of_turn_dispatch_system`
  is downstream of the RSM and not in `server/core/rsm/`.
- **ADR-002** (Client-Server Authority): no change.
- **TR registry**: no new TR.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on the `.rs` edit. No
  Lightyear edits — `liv-bevy-lightyear` NOT required.

### Control Manifest Rules

- Required: the `start_of_turn_dispatch_system not yet
  implemented` message is emitted at `debug!` level (not
  `warn!`).
- Required: the message text and call-site location are
  unchanged.
- Required: the `start_of_turn_dispatch_system` system body is
  unchanged (still a stub / deferred / no-op or its current
  pre-implementation shape). This row does NOT implement the
  dispatch.
- Required: smoke harness confirms zero `WARN` lines with text
  `start_of_turn_dispatch_system not yet implemented` on the
  next Sprint 17 smoke at the default log filter
  (`info`-and-above).
- Required: same lines remain visible at `debug!` level when the
  log filter is raised (e.g.
  `RUST_LOG=debug cargo run -p server`).
- Required: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved.
- Forbidden: implementing `start_of_turn_dispatch_system`. The
  dispatch logic remains deferred; only the log level is
  changed.
- Forbidden: changing the message text.
- Forbidden: removing the log line entirely (it is still useful
  at debug level when developers raise the filter).
- Forbidden: modifying `client/`, `shared/`, or anything under
  `tests/integration/client*` / `tests/integration/lightyear*`.
- Forbidden: modifying `server/core/rsm/`. This row is in
  `server/src/game/` only.
- Forbidden: closure of any AUDIT-1076-* finding outside
  AUDIT-1076-15.
- Forbidden: closure of any SOURCE-1077-* finding.

---

## Story Classification

**Story type**: **Config / Data** (single-line log macro
substitution).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix, Config / Data rows require smoke check PASS
(ADVISORY gate). Evidence is captured by the Sprint 17 smoke
prompt, NOT by this row's `/dev-story` worker. The implementing
worker MAY run a manual `cargo run -p server` locally and
inspect stderr/stdout (optional evidence) but the BLOCKING gate
is the post-implementation Sprint 17 smoke.

This is **NOT** a:

- Logic story (no formula / state-machine / reducer change).
- Integration story (single-module config change; no multi-system
  wiring).
- Visual / UI / accessibility / final-art story.

---

## Dependencies and Parallelism

### Prerequisites

- None on `origin/main`.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| Every other Sprint 17 row | **YES** | this row touches `server/src/game/` only. Disjoint from every other Sprint 17 client-side row. |

This is the **most parallel-safe Sprint 17 row** alongside
`S17-OPS-VULKAN-VALIDATION-GATING-001` and
`S17-UI-CARD-SLOT-INSET-WIRING-001`.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Log macro downgraded from `warn!` to `debug!`**:
  GIVEN the post-implementation `server/src/game/` source, WHEN
  `grep -rn 'start_of_turn_dispatch_system not yet implemented'
  server/src/` is run, THEN the surrounding macro is `debug!`,
  not `warn!`. The message text and the location of the call
  site are unchanged. The `start_of_turn_dispatch_system`
  function body is unchanged.

- [ ] **AC2 -- Sprint 17 smoke confirms zero WARN lines at
  default filter**: GIVEN the post-implementation server binary
  is launched as part of the Sprint 17 smoke harness (a later
  Sprint 17 prompt, NOT this row's `/dev-story` worker), WHEN
  the smoke captures server stderr/stdout at default `info`
  filter level, THEN zero `WARN` lines with text
  `start_of_turn_dispatch_system not yet implemented` appear.
  Smoke evidence path:
  `production/qa/smoke-sprint-17-*.md` (NEW; authored by the
  Sprint 17 smoke prompt, NOT by this story). This AC is
  satisfied by the smoke run.

- [ ] **AC3 -- Same lines visible at `debug!` level**: GIVEN the
  post-implementation server binary launched with
  `RUST_LOG=debug` (or equivalent log filter raise), WHEN
  stderr/stdout is inspected, THEN the
  `start_of_turn_dispatch_system not yet implemented` lines
  DO appear, now at `DEBUG` level. The message text is
  unchanged. This AC is verified by the implementing worker as
  optional local evidence (not a BLOCKING gate).

- [ ] **AC4 -- `start_of_turn_dispatch_system` body unchanged**:
  GIVEN `git diff <activation HEAD>..HEAD` for the worker's
  commit, WHEN inspected, THEN the only change to
  `server/src/game/` is the single-line `warn!` -> `debug!`
  substitution. The system body, registration, schedule, and
  per-round-entry trigger are unchanged. This row does NOT
  implement the dispatch.

- [ ] **AC5 -- No protocol or client change**: GIVEN the same
  diff, WHEN inspected, THEN there are zero changes under
  `client/`, `shared/`, `tests/integration/client*`,
  `tests/integration/lightyear*`, or `tests/integration/server/`
  (this row has no test bin). The implementation is server-side
  only and is bounded to `server/src/game/`.

- [ ] **AC6 -- No RSM change**: GIVEN the same diff, WHEN
  inspected, THEN there are zero changes under
  `server/src/core/rsm/`. The RSM epic boundary is preserved.

- [ ] **AC7 -- No accept-risk closure claimed**: GIVEN the
  commit message and any evidence document, WHEN inspected,
  THEN they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other
  accept-risk disposition. Release readiness, accessibility
  completion, playtest validation, final-art completion, stage
  advance, and Polish->Release gate-check retry are explicitly
  out of scope.

- [ ] **AC8 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/*`, `production/gate-checks/*`, and
  `docs/architecture/adr-*.md` are diffed, THEN none are modified
  by this story's `/dev-story` worker.

- [ ] **AC9 -- Worker branch scope contained**: GIVEN the worker
  branch (slug recommendation:
  `work/s17-start-of-turn-debug-downgrade`), WHEN inspected,
  THEN it pushes only the worker branch — never `main`. The
  only file changed at worker time is the keyword dispatch file
  under `server/src/game/`.

- [ ] **AC10 -- Cargo resource policy applied for every Cargo
  command**: future implementation MUST set the Cargo resource
  policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
  every `cargo check` / `cargo build` / `cargo test` invocation
  on Windows / MSVC. Story authoring (PROMPT 1095) does NOT
  invoke Cargo.

---

## Implementation Notes

### Owned files (likely change set)

| Path | Expected change |
|------|-----------------|
| `server/src/game/mod.rs` OR `server/src/game/dispatch*.rs` (re-verify exact location at activation HEAD) | Single-line `warn!(...)` -> `debug!(...)` substitution at the `start_of_turn_dispatch_system not yet implemented` call site. |
| `production/qa/evidence/sprint-17-start-of-turn-debug/evidence.md` (NEW, optional, by `/dev-story` worker) | Optional evidence document. |

### Forbidden files

- Everything under `client/`, `shared/`.
- Everything under `server/src/core/rsm/` (RSM epic boundary
  preserved).
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files.
- `docs/architecture/adr-*.md`.
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.

### Cargo resource policy

Per the binding Sprint 15+ QA plan precedent, every `cargo`
invocation on Windows / MSVC MUST set the five env vars under
AC10.

### Target citations

- Sprint 17 plan row source:
  `production/sprints/sprint-17.md` §"Nice to Have" row
  `S17-SERVER-START-OF-TURN-DEBUG-001`.
- Source audit:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-15.

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a
   fresh worktree (suggested slug
   `work/s17-start-of-turn-debug-downgrade`).
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time call site by running
   `grep -rn 'start_of_turn_dispatch_system not yet implemented'
   server/src/`. Confirm the exact file and line. Confirm the
   surrounding macro is `warn!` and the message text matches the
   audit.
4. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear`.
5. Make the single-line substitution. Do NOT change message text
   or call-site location.
6. Set the Cargo resource policy env vars per AC10 before every
   `cargo check` / `cargo build` invocation.
7. Run `cargo check -p server` under the Cargo resource policy;
   confirm zero new warnings on the touched file. Optionally
   run `cargo run -p server` locally and confirm the WARN is
   gone at default filter and present at debug filter — this is
   NOT a BLOCKING gate; the BLOCKING gate is the Sprint 17
   smoke (AC2).
8. Push the worker branch (never `main`).
9. Stop. Closure paperwork is later prompts' scope.

The worker MUST NOT:

- Implement `start_of_turn_dispatch_system`. The dispatch logic
  remains deferred.
- Change the message text or call-site location.
- Remove the log line entirely.
- Modify `client/`, `shared/`, or anything under
  `server/src/core/rsm/`.
- Modify `tests/integration/server/`, `tests/integration/client*`,
  or `tests/integration/lightyear*`.
- Modify Cargo / Trunk / CI files.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation
  (this row has no `cargo test` requirement).
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of any AUDIT-1076-* finding outside
  AUDIT-1076-15.
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art
  completion, or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to the file this
worker owns under `server/src/game/`. The worker MUST NOT block
on workspace-wide compilation errors introduced by other
in-flight Sprint 17 workers' branches. This row is file-disjoint
with every other Sprint 17 row.

### Relay / reporting expectation for future workers

Final status line:

```
N: S17-SERVER-START-OF-TURN-DEBUG-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Closure Trail

Closure trail is appended by future `/story-readiness`,
`/dev-story`, and `/story-done` prompts. No closure trail is
authored by PROMPT 1095.

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-
  operator-blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.
- `start_of_turn_dispatch_system` implementation remains
  **deferred** (this row does NOT implement it).

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Closure of any AUDIT-1076-* finding outside AUDIT-1076-15.
- Closure of any SOURCE-1077-* finding.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Sprint 17 close-out.
- Implementation of `start_of_turn_dispatch_system` (the dispatch
  logic remains deferred).
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`003: S17-SERVER-START-OF-TURN-DEBUG-001: DRAFT`
