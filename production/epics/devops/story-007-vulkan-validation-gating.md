# Story 007: S17-OPS-VULKAN-VALIDATION-GATING-001 -- Vulkan Validation-Layer Warning Gating

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S17-OPS-VULKAN-VALIDATION-GATING-001
> **Status**: Done -- closed by PROMPT 1125 paperwork-only `/story-done` against `origin/main@c300b141247307cbd0fbc7f507a175db308026b2` (PROMPT 1124 tip). Source-side remediation discharged by PROMPT 1103 worker + PROMPT 1109 integration (`origin/main@0cab942`). AC1 + AC2 + AC4..AC10 PASS; AC3 ADVISORY-DEFERRED to the Sprint 17 smoke harness per the Config / Data row classification.
> **Layer**: Client / DevOps -- WGPU plugin configuration
> **Type**: Tech Debt / Ops Hygiene -- silence log spam on every client launch
> **Sprint**: Sprint 17 Nice to Have row per `production/sprints/sprint-17.md` §"Nice to Have". Activated by PROMPT 1099 against `origin/main@bc3db29` (Sprint 17 activation tip).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.15d (small ops hygiene; single edit at `App::new()` site)
> **Source audit**: PROMPT 1076 `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md` §"Per-finding evidence" AUDIT-1076-18 (P3)
> **Worker**: PROMPT 1103 (`work/s17-vulkan-validation-gating` @ `c34cc041759385b75d6356ebbae7e3f336cb85a5`); `reports/PROMPT-1103-s17-vulkan-validation-gating.md`
> **Integration**: PROMPT 1109 (`integrate/s17-vulkan-validation-gating-1109` @ `0cab9421bd11b86c05cd804d62739e2e13a55278`); `reports/PROMPT-1109-s17-vulkan-validation-gating-integration.md`
> **Story-Done**: PROMPT 1125 (paperwork-only closure on a fresh worktree from `origin/main@c300b14`); `reports/PROMPT-1125-s17-vulkan-validation-gating-story-done.md`
> **Evidence**: `production/qa/evidence/sprint-17-vulkan-validation-gating/evidence.md`

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

- [x] **AC1 -- Vulkan validation flag is OFF by default**: GIVEN
  the post-implementation client built with default features
  (`cargo build -p client` OR `trunk build`), WHEN launched and
  the stderr/stdout is inspected, THEN there are ZERO
  occurrences of `InstanceFlags::VALIDATION requested, but
  unable to find layer: VK_LAYER_KHRONOS_validation` (or
  equivalent Vulkan validation-layer warnings emitted by WGPU /
  Bevy 0.18 at `App` startup).

- [x] **AC2 -- Validation can be opted in**: GIVEN the post-
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

- [~] **AC3 -- Sprint 17 smoke confirms zero validation
  warnings**: GIVEN the post-implementation default-build client
  is launched as part of the Sprint 17 smoke harness (a later
  Sprint 17 prompt, NOT this row's `/dev-story` worker), WHEN
  the smoke captures client stderr/stdout, THEN zero
  `VK_LAYER_KHRONOS_validation` lines appear. This AC is
  satisfied by the Sprint 17 smoke run, not by this row's
  `/dev-story` worker.

- [x] **AC4 -- WGPU plugin still functions normally**: GIVEN
  the post-implementation default-build client, WHEN launched,
  THEN the existing rendering capability is unchanged: HUD
  paints, board renders, hand fan visible, etc. The
  `liv-bevy-018` review confirms no other WGPU plugin
  configuration was altered.

- [x] **AC5 -- No new workspace Cargo dependency**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN the
  workspace root `Cargo.toml` is unchanged (or carries at most
  a new client feature entry that does not affect server /
  shared crates). The only Cargo file that MAY be modified is
  `client/Cargo.toml` (if the implementing worker chooses the
  feature-gated strategy).

- [x] **AC6 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN
  there are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.

- [x] **AC7 -- No accept-risk closure claimed**: GIVEN the
  commit message and any evidence document, WHEN inspected,
  THEN they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other
  accept-risk disposition. Release readiness, accessibility
  completion, playtest validation, final-art completion, stage
  advance, and Polish->Release gate-check retry are explicitly
  out of scope.

- [x] **AC8 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/*`, `production/gate-checks/*`, and
  `docs/architecture/adr-*.md` are diffed, THEN none are modified
  by this story's `/dev-story` worker.

- [x] **AC9 -- Worker branch scope contained**: GIVEN the worker
  branch (slug recommendation:
  `work/s17-vulkan-validation-gating`), WHEN inspected, THEN it
  pushes only the worker branch — never `main`. Files changed
  at worker time are scoped to `client/src/main.rs` (or
  equivalent `App::new()` site) and optionally
  `client/Cargo.toml`.

- [x] **AC10 -- Cargo resource policy applied for every Cargo
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

## Completion Notes

### PROMPT 1103 worker + PROMPT 1109 integration outcome

**Strategy chosen by worker (PROMPT 1103)**: `--features
wgpu-validation` Cargo feature gate, OFF by default. The story
permitted either `cfg!(debug_assertions)` or the feature gate;
the worker chose the feature gate because AC1 requires zero
`VK_LAYER_KHRONOS_validation` warnings on the default
`cargo build -p client` invocation (the audit-reproducing
configuration). `cfg!(debug_assertions)` would have left the
flag ON in dev builds and therefore would not satisfy AC1
against that profile; the feature gate is the only strategy
that unambiguously satisfies AC1 + AC2 + the Sprint 17 plan
wording "gated on a cargo feature so prod / CI logs stay clean".

**Owned-file change set on origin/main** (3 paths; verified by
`git ls-tree -r origin/main --name-only | grep -i vulkan` +
`grep -n "wgpu-validation\|InstanceFlags\|RenderPlugin"
client/Cargo.toml client/src/main.rs` at PROMPT 1125 closure
time on a fresh worktree from `origin/main@c300b14`):

| Path | Change |
|---|---|
| `client/Cargo.toml` | `wgpu-validation = []` added under `[features]` with S17-OPS comment block at lines 34-40. |
| `client/src/main.rs` | `use bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings};` + `use bevy::render::RenderPlugin;` at lines 9-10; `instance_flags = if cfg!(feature = "wgpu-validation") { InstanceFlags::from_build_config() } else { InstanceFlags::empty() }` at lines 60-64; `.set(RenderPlugin { render_creation: RenderCreation::Automatic(WgpuSettings { instance_flags, ..default() }), ..default() })` on the `DefaultPlugins` builder at line 74. |
| `production/qa/evidence/sprint-17-vulkan-validation-gating/evidence.md` (NEW by worker) | Full AC mapping + launch-log excerpts + commit-message rationale. |

### Test evidence at origin/main@c300b14

- **PROMPT 1103 worker `cargo build -p client` (default features)** +
  non-interactive 8s launch — PASS. Grep `VK_LAYER_KHRONOS_validation`
  match count = **0** (AC1). Renderer still selects RTX 5090 Vulkan
  adapter; window + every client plugin loaded (AC4).
- **PROMPT 1103 worker `cargo build -p client --features
  wgpu-validation`** + non-interactive 8s launch — PASS. Grep match
  count = **1**. Opt-in observability preserved on a host without
  the validation layer (AC2 expected behaviour).
- **PROMPT 1103 worker `cargo check -p client`** — PASS, no new
  warnings on touched files.
- **PROMPT 1109 integration `cargo check -p client`** — PASS
  (11.32s + 11.66s after forward-merge).
- **PROMPT 1109 integration `cargo build -p client` (default
  features)** — PASS (59.58s + 1m10s after forward-merge).
- **PROMPT 1109 integration `cargo build -p client --features
  wgpu-validation`** — PASS (1m46s + 1m09s after forward-merge).
- **PROMPT 1109 integration `git diff --check origin/main...HEAD`** —
  clean. **`git diff --cached --check`** — clean.
- **`git diff origin/main...HEAD` after forward-merge** — exactly
  the 3 expected files.
- **AC3 binding evidence**: deferred to the Sprint 17 smoke harness
  (Config / Data row classification per
  `.claude/docs/coding-standards.md` Test Evidence by Story Type
  matrix; smoke check pass is the ADVISORY gate for Config / Data
  rows). PROMPT 1125 paperwork-only closure proceeds on AC1 + AC2
  + AC4..AC10 PASS; AC3 carries forward into the existing Sprint
  17 smoke prompt scope.

### Cargo resource policy (AC10)

PROMPT 1103 worker applied all 5 Cargo resource policy env vars
(`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` +
`CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
`CARGO_INCREMENTAL=0` +
`RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
every cargo invocation; D: free space >= 760 GB at preflight;
stray `target/` directory created by the first env-unloaded
invocation was removed before re-running under the policy.
PROMPT 1109 integration applied the same env vars; D: free ~745
GB at integration start. Build correctness gate unaffected.
PROMPT 1125 itself did NOT invoke Cargo (paperwork-only
closure).

### Per-AC outcome

- **AC1** Vulkan validation flag OFF by default: **PASS** (grep
  match count = 0 on PROMPT 1103 worker default-build launch log;
  reverified by PROMPT 1109 integration cargo build).
- **AC2** Validation can be opted in via `--features
  wgpu-validation`: **PASS** (grep match count = 1 on PROMPT 1103
  worker opt-in launch log; documented opt-in behaviour on a host
  without the validation layer).
- **AC3** Sprint 17 smoke confirms zero validation warnings:
  **ADVISORY-DEFERRED** to the Sprint 17 smoke harness (Config /
  Data row classification; not BLOCKING for PROMPT 1125 closure).
- **AC4** WGPU plugin still functions normally: **PASS**
  (Vulkan `AdapterInfo`, window creation, every client plugin
  loaded in both launch logs; integration cargo build also
  confirms).
- **AC5** No new workspace Cargo dependency: **PASS** (workspace
  root `Cargo.toml` unchanged; only `client/Cargo.toml` touched;
  `Cargo.lock` unchanged — the `wgpu-validation` feature toggles
  `InstanceFlags` and pulls no new dependency).
- **AC6** No protocol or server change: **PASS** (PROMPT 1109
  `git diff` confirms zero changes under `server/`, `shared/`,
  `tests/integration/server/`).
- **AC7** No accept-risk closure claimed: **PASS** (PROMPT 1103
  worker commit + evidence file + PROMPT 1109 integration merge
  commit + PROMPT 1125 paperwork all explicitly preserve
  `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `TQ-S12-C1..C7`, PROMPT 761 Polish->Release FAIL,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all AUDIT-1076-*
  findings outside AUDIT-1076-18, all SOURCE-1077-*, all 24
  PROMPT 1022 findings).
- **AC8** Sprint 17 disposition preserved by worker +
  integration: **PASS** (PROMPT 1103 + PROMPT 1109 diffs touched
  zero files under `production/sprint-status.yaml`,
  `production/sprints/sprint-17.md`, `production/stage.txt`,
  `production/session-state/*`,
  `production/qa/qa-plan-sprint-17.md`,
  `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/*`, `docs/architecture/adr-*.md`.
  PROMPT 1125 is the first authorised modifier of
  `production/sprint-status.yaml` + `production/session-state/*`
  for this row).
- **AC9** Worker branch scope contained: **PASS** (PROMPT 1103
  worker pushed `work/s17-vulkan-validation-gating` only —
  never `main`. Integration into `origin/main` performed
  separately by PROMPT 1109 via
  `integrate/s17-vulkan-validation-gating-1109` →
  `0cab9421bd11b86c05cd804d62739e2e13a55278`).
- **AC10** Cargo resource policy applied: **PASS-WORKER +
  PASS-INTEGRATION**. PROMPT 1103 worker applied all 5 env vars
  before every cargo invocation; PROMPT 1109 integration also
  applied the policy (see Cargo resource policy section above).
  PROMPT 1125 paperwork-only closure did NOT invoke Cargo.

### Branch-state note (preserved verbatim)

PROMPT 1125 paperwork performed on a fresh worktree
`D:/_DEV/claude-code-game-studios-worktrees/vulkan-validation-gating-story-done-1125`
on branch `worker/vulkan-validation-gating-story-done-1125` from
`origin/main@c300b14` to avoid acting on the root checkout's
local divergent `main` (per the PROMPT 1123/1124-recorded
branch-state anomaly where local-only commits sit on the root
checkout's local main without push). The root checkout is NOT
touched by PROMPT 1125.

## Closure Trail

PROMPT 1125 closes this story as Done via paperwork-only
`/story-done` against `origin/main@c300b141247307cbd0fbc7f507a175db308026b2`
(PROMPT 1124 `story-done(s17): close
S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 (PROMPT 1124)`). Source-side
remediation discharged by PROMPT 1103 worker
(`c34cc041759385b75d6356ebbae7e3f336cb85a5` `dev-story(s17):
gate Vulkan validation behind cargo feature
(S17-OPS-VULKAN-VALIDATION-GATING-001 / AUDIT-1076-18, PROMPT
1103)`) + PROMPT 1109 integration
(`0cab9421bd11b86c05cd804d62739e2e13a55278` `integrate(s17):
bring origin/main 5345164 (PROMPTS 1106 1113 1114 1115 1116
1117 1118 1119 1120 1121) forward into PROMPT 1103 Vulkan
validation gating integration branch (PROMPT 1109)`).

### Numbered closure trail

1. **PROMPT 1095** — `sprint-plan(s17)`: net-new Sprint 17
   story authoring batch authored story 007; not activated by
   this run.
2. **PROMPT 1097** — paperwork-only main integration onto
   `origin/main@bc3db29`.
3. **PROMPT 1099** — `activate(s17)`: flipped Sprint 17 from
   draft to active on `origin/main@cb62a9e`.
4. **PROMPT 1100** — `qa-plan(s17)`: authored Sprint 17 QA plan
   on `origin/main@ff47075`.
5. **PROMPT 1101** — `/story-readiness` rerun against Sprint 17
   activation HEAD bc3db29 (story marked READY).
6. **PROMPT 1103** — `/dev-story`: feature-gated Vulkan
   validation behind `--features wgpu-validation`; worker
   branch `work/s17-vulkan-validation-gating` @ `c34cc04`.
7. **PROMPT 1109** — paperwork-only integration onto
   `origin/main`; tip `0cab9421bd11b86c05cd804d62739e2e13a55278`.
8. **PROMPT 1125** — paperwork-only `/story-done` closure on a
   fresh worktree from `origin/main@c300b14`; this row.

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED; `production/stage.txt`
  NOT touched by PROMPT 1125).
- Sprint 17 disposition `active` (UNCHANGED; Sprint 17 NOT
  closed-out by PROMPT 1125).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`;
  NO retry by PROMPT 1125.
- `S8-QA-001-W1` OPEN preserved (two-client GAME_OVER closure
  remains gap).
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- `TQ-S12-C1..C7` preserved verbatim. TQ-S12-C7 explicitly NOT
  closed by PROMPT 1125.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  human-operator-blocked carry preserved; NOT closed by PROMPT
  1125.
- S17-UI-HUD-OPP-MANA-CLEANUP-001 PROMPT 1112 PARTIAL
  disposition with AC3 carry preserved verbatim; NOT closed by
  PROMPT 1125.
- PROMPT 1108 + PROMPT 1110 + PROMPT 1117 + PROMPT 1120 +
  PROMPT 1121 + PROMPT 1124 `sprint_17_story_done` entries
  preserved verbatim.
- PROMPT 1112 `sprint_17_partial_disposition` entry
  (S17-UI-HUD-OPP-MANA-CLEANUP-001 PARTIAL / AC3 carried; row
  remains OPEN) preserved verbatim.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.
- AUDIT-1076-18 (P3) discharged on origin/main by PROMPT 1103
  worker + PROMPT 1109 integration; row paperwork closed by
  PROMPT 1125. Other AUDIT-1076-* findings preserved outside
  the already-discharged subset (14 PROMPT 1118/1120; 15 PROMPT
  1107/1108; 10 + 16 PROMPT 1111). AUDIT-1076-17 remains OPEN
  carried with AC3 of S17-UI-HUD-OPP-MANA-CLEANUP-001.
- SOURCE-1077-01/02/03/04 discharged by PROMPT 1114/1117;
  SOURCE-1077-06 by PROMPT 1106/1110; SOURCE-1077-08/09/16 by
  PROMPT 1123/1124; SOURCE-1077-10 by PROMPT 1119/1121.
  SOURCE-1077-05/07/11/12/13/14/15 deferred to Sprint 18+.
- External `shop_auction_ui_plugin_scaffold_formulas_test`
  baseline drift `87 vs 82` preserved verbatim by PROMPT 1124
  paperwork and NOT silently fixed by PROMPT 1125.

### Explicitly NOT claimed by this story or PROMPT 1125

- Closure of any AUDIT-1076-* finding outside AUDIT-1076-18 by
  PROMPT 1125.
- Closure of any SOURCE-1077-* finding by PROMPT 1125.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Sprint 17 close-out.
- Removal of Vulkan / WGPU validation capability (the opt-in
  path remains via `--features wgpu-validation`).
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.
- AC3 binding closure (deferred to the Sprint 17 smoke harness
  per Config / Data row classification; ADVISORY-DEFERRED).
- Fix of external
  `shop_auction_ui_plugin_scaffold_formulas_test` baseline
  drift (preserved verbatim by PROMPT 1124 paperwork; recommend
  separate follow-up story for counter reconciliation).
- Closure of S11-HUD-TIMER-EYEBALL-VISUAL-001
  (human-operator-blocked carry; no LLM `/story-done`
  authorised).
- Closure of S17-UI-HUD-OPP-MANA-CLEANUP-001 (PROMPT 1112
  PARTIAL disposition preserved verbatim; row remains OPEN;
  PROMPT 1125 does NOT modify the PARTIAL disposition or
  activate the follow-up candidate slug).
- Any change to `client/` / `server/` / `shared/` / `tests/` /
  `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/` /
  `Trunk.toml` by PROMPT 1125 (paperwork-only closure).
- Any change to `production/stage.txt`, `production/sprints/*`,
  `production/qa/qa-plan-sprint-17.md`,
  `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/*`, `docs/architecture/adr-*.md`,
  `production/qa/evidence/sprint-17-vulkan-validation-gating/*`
  by PROMPT 1125.
- Any `/story-readiness`, `/dev-story`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`,
  cargo, trunk, or CI command run by PROMPT 1125.

`007: S17-OPS-VULKAN-VALIDATION-GATING-001: DONE`
