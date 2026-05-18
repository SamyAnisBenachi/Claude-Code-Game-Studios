# Story 007: S17-OPS-VULKAN-VALIDATION-GATING-001 -- Vulkan Validation-Layer Warning Gating

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S17-OPS-VULKAN-VALIDATION-GATING-001
> **Status**: Draft -- Sprint 17 Nice to Have candidate; NOT activated by this authoring run
> **Layer**: Client / DevOps -- WGPU plugin configuration
> **Type**: Tech Debt / Ops Hygiene -- silence log spam on every client launch
> **Sprint**: Sprint 17 Nice to Have row per `production/sprints/sprint-17.md` §"Nice to Have". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.15d (small ops hygiene; single edit at `App::new()` site)
> **Source audit**: PROMPT 1076 `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md` §"Per-finding evidence" AUDIT-1076-18 (P3)

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
`S11-HUD-TIMER-EYEBALL-VISUAL-001`, closure of any of the 24 PROMPT
1022 audit findings, closure of any SOURCE-1077-* finding, or
closure of any AUDIT-1076-* finding outside AUDIT-1076-18.

**No optimistic client-side authority is introduced or proposed.**
No protocol shape change. No new server-authoritative state. No
new C2S / S2C message. The change is a one-line guard on the WGPU
plugin's validation feature flag.

Sprint 16 disposition `closed-with-conditions` preserved unchanged.
PROMPT 761 Polish->Release gate-check `FAIL` preserved.
`PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`, `TQ-S12-C1..C7`
preserved verbatim.

---

## Source Finding

### AUDIT-1076-18 (P3) — Vulkan validation-layer noise

- **Audit location**:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-18.
- **Severity**: P3.
- **Evidence**: 3 Vulkan validation-layer warnings on every client
  launch (run-7 client-a:2-5, client-b:2-5):
  `InstanceFlags::VALIDATION requested, but unable to find layer:
  VK_LAYER_KHRONOS_validation`.
- **Behaviour**: dev-only, harmless on the test machine (the
  validation layer is not installed by default on Windows /
  end-user machines), but emits log spam on every launch.
- **Minimal repair surface** (audit recommendation):
  `client/src/main.rs` (or wherever `App::new()` configures the
  WGPU plugin) gates validation on `cfg!(debug_assertions)` and /
  or a `--features wgpu-validation` Cargo feature.
- **Sprint 17 plan rationale**: per
  `production/sprints/sprint-17.md` §"Nice to Have" row
  `S17-OPS-VULKAN-VALIDATION-GATING-001`: "3 Vulkan validation
  warnings on every client launch (run-7 client-a:2-5,
  client-b:2-5). Dev-only, harmless on the test machine."

---

## Problem Class / Prevention Target

**Defect class**: a WGPU plugin configured to request a Vulkan
validation layer that is not installed on the test / dev / prod
machine, producing the same 3 warning lines on every client
launch. Real warnings get harder to spot when the log is
pre-populated with three known-deferred lines.

**Prevention target**: gate the Vulkan validation request behind
either `cfg!(debug_assertions)` OR an explicit
`--features wgpu-validation` Cargo feature, so the validation flag
is OFF by default (no warning) and the developer can opt in when
they want validation diagnostics.

---

## Context

### Existing surface

- **`client/src/main.rs`** (or wherever the WGPU plugin is
  configured — the implementing worker re-verifies). The
  `App::new()` site currently configures the WGPU plugin with
  `InstanceFlags::VALIDATION` (or equivalent flag) set
  unconditionally.
- **`client/Cargo.toml`** — the Cargo feature list for the client
  crate. A new optional feature `wgpu-validation` MAY be added
  here.
- **`Cargo.toml`** (workspace root) — only touched if the new
  feature needs to propagate (likely unnecessary for a client-
  local feature).
- **Existing `cfg!(debug_assertions)` usage in the client** —
  re-verify pattern.

### GDD / ADR / TR trace

- **GDD**: not applicable; this is ops hygiene, not GDD-spec'd.
- **ADR-021** (Presentation Layer Architecture): no change.
- **ADR-002** (Client-Server Authority): no change.
- **ADR-003** (Cargo Workspace Structure): minor — a new Cargo
  feature is well within `client/Cargo.toml` scope; workspace
  boundary preserved.
- **TR registry**: no new TR.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on the `.rs` edit (the
  `App::new()` site is Bevy ECS configuration code). No Lightyear
  edits — `liv-bevy-lightyear` NOT required.

### Control Manifest Rules

- Required: the Vulkan validation flag is OFF by default. The
  default `cargo build` / `cargo run` / `trunk build` / `trunk
  serve` invocation MUST NOT request `VK_LAYER_KHRONOS_validation`.
- Required: the developer can opt in via either
  `cfg!(debug_assertions)` (i.e. it is ON in debug builds, OFF
  in release builds) OR a `--features wgpu-validation` Cargo
  feature. Implementing worker chooses one strategy and justifies.
- Required: smoke harness confirms zero `VK_LAYER_KHRONOS_validation`
  / `InstanceFlags::VALIDATION` warning lines on the next Sprint
  17 smoke against the default build profile.
- Required: no `Cargo.toml` workspace change. Any new feature
  lives in `client/Cargo.toml` only.
- Required: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved.
- Forbidden: removing Vulkan / WGPU validation capability
  entirely. The opt-in path must remain so developers can request
  validation when debugging GPU issues.
- Forbidden: modifying `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Forbidden: closure of any AUDIT-1076-* finding outside
  AUDIT-1076-18.
- Forbidden: closure of any SOURCE-1077-* finding.

---

## Story Classification

**Story type**: **Config / Data** (single-line feature-gate at
the `App::new()` configuration site; possibly accompanied by a
new Cargo feature line in `client/Cargo.toml`).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix, Config / Data rows require smoke check PASS
(ADVISORY gate). Evidence is captured by the Sprint 17 smoke
prompt — NOT by this story's `/dev-story` worker. The
implementing worker MAY run a manual `cargo run -p client` and
confirm the warning is absent from stderr/stdout (optional
evidence) but the BLOCKING gate is the post-implementation Sprint
17 smoke.

This is **NOT** a:

- Logic story (no formula / state-machine / reducer change).
- Integration story (single-module config change).
- Visual / feel story (no shader / VFX change).
- UI / accessibility / final-art story.

---

## Dependencies and Parallelism

### Prerequisites

- None on `origin/main`.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| Every other Sprint 17 row | **YES** | this row touches `client/src/main.rs` (or wherever `App::new()` configures the WGPU plugin) + optionally `client/Cargo.toml`. Disjoint from every other Sprint 17 row. |

This is the **most parallel-safe Sprint 17 row** alongside
`S17-UI-CARD-SLOT-INSET-WIRING-001` and
`S17-SERVER-START-OF-TURN-DEBUG-001`.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Vulkan validation flag is OFF by default**: GIVEN
  the post-implementation client built with default features
  (`cargo build -p client` OR `trunk build`), WHEN launched and
  the stderr/stdout is inspected, THEN there are ZERO
  occurrences of `InstanceFlags::VALIDATION requested, but
  unable to find layer: VK_LAYER_KHRONOS_validation` (or
  equivalent Vulkan validation-layer warnings emitted by WGPU /
  Bevy 0.18 at `App` startup).

- [ ] **AC2 -- Validation can be opted in**: GIVEN the post-
  implementation client, WHEN built either (a) in debug mode
  with `cfg!(debug_assertions)`, OR (b) with
  `cargo build -p client --features wgpu-validation` (per the
  implementing worker's strategy choice), THEN the Vulkan
  validation flag IS set on the WGPU plugin AND (on a machine
  where the validation layer is installed) the warning is gone
  but validation diagnostics fire normally. On a machine where
  the validation layer is NOT installed (the test / dev
  machine), the opt-in path will still emit the 3 warnings — that
  is expected behaviour for the opt-in path and is documented
  in the commit message / evidence file.

- [ ] **AC3 -- Sprint 17 smoke confirms zero validation
  warnings**: GIVEN the post-implementation default-build client
  is launched as part of the Sprint 17 smoke harness (a later
  Sprint 17 prompt, NOT this row's `/dev-story` worker), WHEN
  the smoke captures client stderr/stdout, THEN zero
  `VK_LAYER_KHRONOS_validation` lines appear. This AC is
  satisfied by the Sprint 17 smoke run, not by this row's
  `/dev-story` worker.

- [ ] **AC4 -- WGPU plugin still functions normally**: GIVEN
  the post-implementation default-build client, WHEN launched,
  THEN the existing rendering capability is unchanged: HUD
  paints, board renders, hand fan visible, etc. The
  `liv-bevy-018` review confirms no other WGPU plugin
  configuration was altered.

- [ ] **AC5 -- No new workspace Cargo dependency**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN the
  workspace root `Cargo.toml` is unchanged (or carries at most
  a new client feature entry that does not affect server /
  shared crates). The only Cargo file that MAY be modified is
  `client/Cargo.toml` (if the implementing worker chooses the
  feature-gated strategy).

- [ ] **AC6 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN
  there are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.

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
  `work/s17-vulkan-validation-gating`), WHEN inspected, THEN it
  pushes only the worker branch — never `main`. Files changed
  at worker time are scoped to `client/src/main.rs` (or
  equivalent `App::new()` site) and optionally
  `client/Cargo.toml`.

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
| `client/src/main.rs` (or wherever `App::new()` configures the WGPU plugin — re-verify at activation HEAD) | Gate the `InstanceFlags::VALIDATION` (or equivalent) on `cfg!(debug_assertions)` OR `cfg!(feature = "wgpu-validation")`. |
| `client/Cargo.toml` (optional, IF feature-gated strategy chosen) | Add `wgpu-validation = []` (or equivalent) to `[features]`. |
| `production/qa/evidence/sprint-17-vulkan-validation-gating/evidence.md` (NEW, by `/dev-story` worker; optional) | Optional evidence document showing local `cargo run` stderr/stdout free of the warning. |

### Forbidden files

- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`,
  `tests/unit/server/`, `tests/integration/lightyear*`,
  `tests/unit/lightyear*`.
- `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- Workspace root `Cargo.toml` (except in the unlikely case that
  a new feature must propagate from the workspace; the worker
  pauses and escalates in that case).
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
  `S17-OPS-VULKAN-VALIDATION-GATING-001`.
- Source audit:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-18.

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a
   fresh worktree (suggested slug
   `work/s17-vulkan-validation-gating`).
2. Read this story file end-to-end before any code change.
3. Re-verify the `App::new()` WGPU plugin configuration site.
   It is in `client/src/main.rs` at audit time, but the
   implementing worker confirms.
4. Pick the gating strategy (`cfg!(debug_assertions)` vs
   `--features wgpu-validation`). Justify in the commit
   message. Default recommendation:
   `cfg!(debug_assertions)` because it requires no Cargo edit
   and matches Sprint 17 plan row "gated on a `cargo` feature
   so prod / CI logs stay clean" wording (debug_assertions is
   off in release builds, which is the prod-like config).
5. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear`.
6. Set the Cargo resource policy env vars per AC10 before every
   `cargo check` / `cargo build` invocation.
7. Run `cargo check -p client` under the Cargo resource policy;
   confirm zero new warnings on the touched file. Optionally
   run `cargo run -p client` locally (subject to the worker's
   ability to launch a graphical client) and capture stderr
   showing the warning is gone — this is NOT a BLOCKING gate;
   the BLOCKING gate is the Sprint 17 smoke (AC3).
8. Push the worker branch (never `main`).
9. Stop. Closure paperwork is later prompts' scope.

The worker MUST NOT:

- Modify `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Modify workspace root `Cargo.toml` unless explicitly
  required by the chosen feature strategy (pause and escalate
  if so).
- Remove Vulkan / WGPU validation capability entirely. The
  opt-in path must remain.
- Modify `.github/`, `Trunk.toml`, `.cargo/`, or any CI
  configuration.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation
  (this row has no `cargo test` requirement; the BLOCKING gate
  is the Sprint 17 smoke).
- Run `trunk build --release` or any CI command beyond a local
  smoke `cargo check` / `cargo run`.
- Push to `main`.
- Claim closure of any AUDIT-1076-* finding outside
  AUDIT-1076-18.
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art
  completion, or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to the file this
worker owns (`client/src/main.rs` + optionally
`client/Cargo.toml`). The worker MUST NOT block on workspace-
wide compilation errors introduced by other in-flight Sprint 17
workers' branches. This row is file-disjoint with every other
Sprint 17 row.

### Relay / reporting expectation for future workers

Final status line:

```
N: S17-OPS-VULKAN-VALIDATION-GATING-001: STATUS
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

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Closure of any AUDIT-1076-* finding outside AUDIT-1076-18.
- Closure of any SOURCE-1077-* finding.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Sprint 17 close-out.
- Removal of Vulkan / WGPU validation capability (the opt-in
  path remains).
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`007: S17-OPS-VULKAN-VALIDATION-GATING-001: DRAFT`
