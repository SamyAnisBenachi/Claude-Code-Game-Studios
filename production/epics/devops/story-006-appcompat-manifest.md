# Story 006: S15-OPS-APPCOMPAT-MANIFEST-001 -- Windows AppCompat Manifest for `spawn_range_live_update_contract` Test Binary

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S15-OPS-APPCOMPAT-MANIFEST-001
> **Status**: Done -- closed by PROMPT 1072 on 2026-05-17 on basis of PROMPT 1068 worker (`ed58e3d98ab0ddc851dfcf6fdad84680a0006937`) + PROMPT 1071 integration (`488a9cd96e6bce2b9ecca7db172ab1be1acc8878`) + evidence at `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
> **Layer**: DevOps -- Cargo test-binary configuration (Windows manifest embed)
> **Type**: Implementation / ops hygiene -- bounded build-system change scoped
> to one Cargo test target
> **Sprint**: Sprint 16 Nice to Have row per `production/sprints/sprint-16.md`
> §"Nice to Have" (sourced from Sprint 14 PROMPT 983 smoke rerun §"Windows
> AppCompat Workaround" Option B; preserved into Sprint 15 deferred backlog
> by PROMPT 988 plan §"Smoke evidence hygiene"). Activated by PROMPT 1064
> on `origin/main@6f9308c`.
> **Authored**: 2026-05-17 by PROMPT 1057
> **Authoring source-of-truth**: `origin/main@8bec9dc` (PROMPT 1055 state
> banner record, P1 UI snapshot retest human-block; latest `origin/main`
> at authoring time)
> **Closed**: 2026-05-17 by PROMPT 1072
> **Closure source-of-truth**: `origin/main@bd374ddf4360f01838830cef22bc36e3763fb8c1`
> (PROMPT 1070 final integration tip merging the dead-code warning cleanup
> on top of PROMPT 1071 AppCompat integration `488a9cd`)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16 candidate. Sprint 16 is **NOT**
activated by PROMPT 1057. Sprint 15 close-out disposition (still
pending at authoring time) and all prior sprint dispositions are not
changed by this authoring run.

PROMPT 1057 (this authoring run) does NOT:

- Activate Sprint 16.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-16.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`,
  or any build script.
- Embed any Windows manifest under this authoring run.
- Close or reopen the sibling story 005
  (`S13-OPS-WIN-APPCOMPAT-NOTE-001` -- doc-only Windows AppCompat
  heuristic note, already DONE on `origin/main` per PROMPT 888).
- Retry the PROMPT 761 Polish->Release gate-check.
- Run any Cargo / Trunk / CI command.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), final-art / asset-production completion, or
closure of any `TQ-S12-C*` Sprint 12 Team-QA condition (in particular
`TQ-S12-C7` AppCompat heuristic remains informational and is **NOT
closed** by this story; closure of `TQ-S12-C7`, if it ever happens, is
a separate `/team-qa` paperwork run).

Sprint 10 / Sprint 11 / Sprint 12 / Sprint 13 / Sprint 14 / Sprint 15
dispositions unchanged. PROMPT 761 Polish->Release gate-check FAIL
evidence preserved at
`production/gate-checks/gate-polish-release-2026-05-12.md`.

**This story is a bounded ops-hygiene implementation.** The implementation
prompt embeds (or otherwise robustly suppresses) the AppCompat
installer-detection heuristic for exactly one Cargo test target. **No
gameplay behaviour changes. No release claim. No QA-condition closure.**

---

## Source Finding

- **`production/sprints/sprint-16.md` Nice to Have row
  `S15-OPS-APPCOMPAT-MANIFEST-001`** (the sprint draft row this story
  formalises). The row prescribes "embed a Windows manifest with
  `level=\"asInvoker\"`" on the test binary and explicitly identifies
  PROMPT 988 plan §"Smoke evidence hygiene" as the deferral path.
- **Sprint 14 PROMPT 983 smoke rerun report**
  `production/qa/smoke-sprint-14-2026-05-16-rerun.md`
  §"Windows AppCompat Workaround". Documents:
  - The exact Cargo error chain (`os error 740 -- "The requested
    operation requires elevation."`).
  - The per-run rename workaround (`cp ... srluc_appcompat_renamed.exe`
    then 5 direct invocations) that has been used by PROMPT 815 /
    PROMPT 982 / PROMPT 983 to unblock smoke.
  - Option B forward-looking path: "rename the test source file or add
    a Windows manifest via `embed-resource` / `winres` to the `shared`
    test target. **Not authorised** by PROMPT 983 (scope forbids
    `tests/` modifications). Could be filed as a Sprint 14 candidate
    Nice-to-Have if the host environment continues to flag this
    binary." -- that filing is this story.
- **Sibling story 005**: `production/epics/devops/story-005-win-appcompat-note.md`
  (`S13-OPS-WIN-APPCOMPAT-NOTE-001`, DONE on `origin/main` per PROMPT
  888). Story 005 is **background only**: it documented the heuristic
  and the chosen workaround at `docs/setup/dev-environment.md`. Story
  005 explicitly named `S13-OPS-WIN-APPCOMPAT-MANIFEST-001` as a
  candidate follow-on (see story 005 AC2 PASS evidence at
  `docs/setup/dev-environment.md:262-278`). The Sprint 16 row renames
  the slug `S13-OPS-WIN-APPCOMPAT-MANIFEST-001` -> `S15-OPS-APPCOMPAT-MANIFEST-001`
  to match the deferral chain (Sprint 13 surfacing -> Sprint 14 smoke
  reaffirmation -> Sprint 15 deferred backlog -> Sprint 16 candidate).
  This story is the implementation follow-on. **Story 005 is NOT
  reopened.** Story 005 closure evidence at
  `origin/main@807c3e7377cf58330fc5e2f9b61fbdd6eb9127a1` is preserved
  unchanged.
- **Cargo target inventory** (read-only verification at
  `origin/main@8bec9dc`):
  - `shared/Cargo.toml` defines `[[test]] name = "spawn_range_live_update_contract"`
    at `path = "../tests/unit/protocol/spawn_range_live_update_contract_test.rs"`.
  - The only source file for the test is
    `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
  - The binary that the heuristic intercepts is
    `D:\_DEV\cargo-target\ccgs-msvc\debug\deps\spawn_range_live_update_contract-<hash>.exe`
    (per PROMPT 983 evidence).

---

## Problem Class / Prevention Target

**Defect class** (verbatim from Sprint 14 PROMPT 983 smoke
§"Windows AppCompat Workaround"): the Windows Application Compatibility
shim layer intercepts executables whose **filename** contains the
substrings `update`, `install`, `setup`, or `patch` and demands UAC
elevation unless an embedded application manifest declares
`<requestedExecutionLevel level=\"asInvoker\"/>`. Cargo-emitted rustc
test binaries do not currently embed such a manifest, so on
elevation-prompting Windows hosts the test target named
`spawn_range_live_update_contract` (whose binary filename contains
`update`) fails to spawn with OS error 740 unless renamed at invocation
time.

**Prevention target**: a robust mechanism that ensures the
`spawn_range_live_update_contract` test binary launches without OS error
740 on the same Windows host class for 5 consecutive runs **without**
the per-run rename workaround. The default mechanism is an embedded
Windows manifest with `requestedExecutionLevel level="asInvoker"` on
the test binary, but the implementing worker chooses among the
following mechanism options at `/dev-story` time and records the
chosen option + rationale in story evidence:

- **Mechanism (a) -- `embed-resource` crate via `build.rs`**: add a
  small `build.rs` to the `shared` crate (or to a dedicated
  test-binary build helper) that compiles a Windows resource file
  (`.rc` + `.manifest` XML pair) embedding the `asInvoker` manifest
  into the test target on `cfg(target_os = "windows")` only.
  Trade-off: introduces a `build.rs` dependency in `shared/`; adds
  one Cargo build dependency (`embed-resource` ~0.6 MB sources).
- **Mechanism (b) -- `winresource` (formerly `winres`) crate via
  `build.rs`**: similar to (a) but uses the `winresource` crate,
  which has a slightly higher-level API for setting the manifest.
  Trade-off: same `build.rs` cost; alternative crate choice.
- **Mechanism (c) -- `embed-manifest` crate via `build.rs`**: the
  most narrowly scoped crate; only handles application-manifest
  embedding and nothing else. Trade-off: smallest dependency
  surface; same `build.rs` cost.
- **Mechanism (d) -- Cargo `[[test]] name = "..."` rename**: change
  `shared/Cargo.toml` `[[test]] name = "spawn_range_live_update_contract"`
  to a name that does not contain the substring `update` (e.g.
  `spawn_range_live_refresh_contract`, `spawn_range_live_contract`,
  or similar). Trade-off: no build-system change, no new dependency,
  but the Cargo target name no longer matches the source-of-truth
  invariant being tested (the live-`update` protocol contract), so
  semantic drift between the target name and the protocol it
  validates is the cost. The chosen rename must preserve the
  evidence trail to `S13-PROTO-INVARIANT-001` and to the existing
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
  source file. **The source file itself is NOT renamed under this
  story** (renaming the source file is out of scope -- it would
  cascade into every other place the file is referenced; the Cargo
  `[[test]] name` attribute can decouple the target binary name from
  the source file name without renaming the file).
- **Mechanism (e) -- Equivalent robust mechanism not enumerated
  above**: the implementing worker MAY choose an equivalent mechanism
  (e.g. a custom Cargo `[package.metadata.winres]`-style hook, a
  workspace-level `build.rs`, or a vendored manifest binding) **only
  if** it satisfies the same prevention target (no OS error 740 across
  5 consecutive runs without the rename workaround) and records the
  chosen mechanism + rationale in story evidence.

The implementing worker picks exactly one mechanism (a) / (b) / (c) /
(d) / (e) -- not multiple -- and records the rationale in story
evidence. The mechanism choice is bounded to the
`spawn_range_live_update_contract` test target's configuration; **no
production-source change lands** under any mechanism choice (no
change to `client/`, `server/`, `shared/src/`, or
`tests/unit/protocol/spawn_range_live_update_contract_test.rs`).

If mechanism (a) / (b) / (c) is chosen, the new Cargo build dependency
is scoped to `[target.'cfg(windows)'.build-dependencies]` (or the
nearest Cargo equivalent) so non-Windows builds incur zero extra
dependency cost. If mechanism (d) is chosen, no new dependency is
added.

---

## Context

### Existing surface

- **`shared/Cargo.toml`** at `origin/main@8bec9dc` lines 17-19:
  ```toml
  [[test]]
  name = "spawn_range_live_update_contract"
  path = "../tests/unit/protocol/spawn_range_live_update_contract_test.rs"
  ```
  This is the Cargo target whose emitted binary filename contains the
  trigger substring `update`. The implementing worker modifies this
  block (mechanism (d)) **or** adds a sibling build-system mechanism
  alongside it (mechanism (a) / (b) / (c) / (e)).
- **`tests/unit/protocol/spawn_range_live_update_contract_test.rs`**:
  the test source file; **NOT renamed under this story** (source-file
  rename is out of scope and would cascade into Story 005 evidence,
  `S13-PROTO-INVARIANT-001` evidence, and the Sprint 13 invariant test
  ecosystem).
- **`docs/setup/dev-environment.md`** (already on `origin/main` per
  Story 005 / PROMPT 873 / PROMPT 887): existing H2 section
  `## Windows AppCompat heuristic for Cargo test binaries` at
  lines 160-307 documents the heuristic and the per-run rename
  workaround. **This story does NOT delete that section.** The
  implementing prompt MAY (and SHOULD) append a follow-up subsection
  noting that the manifest mechanism has landed and that the per-run
  rename workaround is now retained only as a fallback for non-MSVC
  builds or for hosts where the manifest-embed mechanism is not
  available. **The rename workaround documentation is NOT
  deleted.**
- **Smoke harness** (per Sprint 14 PROMPT 983 smoke rerun
  `production/qa/smoke-sprint-14-2026-05-16-rerun.md`): currently
  invokes the rename workaround at smoke time. After this story
  lands, future smoke runs (Sprint 16 `/smoke-check` and later)
  invoke the test target directly under `cargo test --workspace
  --tests --no-fail-fast` with no rename step. The implementing
  prompt updates the smoke harness invocation guidance (in the next
  smoke report's "How to reproduce" section, or in the smoke runbook
  if one exists, or in `docs/setup/dev-environment.md`'s AppCompat
  section as a follow-up) to drop the rename step. **This story
  does NOT modify any existing smoke report on disk** (the smoke
  reports are evidence artifacts and are immutable); the next
  smoke report authored after this story lands will record the new
  behaviour.

### Engine / skills

- **Engine**: N/A directly (Windows OS heuristic / Cargo build-system
  concern). Bevy 0.18 / Lightyear 0.26 are not involved.
- **Mandatory skills for `/dev-story`**:
  - `liv-bevy-018`: NOT required (no Bevy code change).
  - `liv-bevy-lightyear`: NOT required (no Lightyear code change).
  - The implementing worker is the **devops-engineer** agent (per
    `production/sprints/sprint-16.md` row owner). The UI programmer
    column on the sprint-16.md row is the secondary reviewer (the
    sprint row pairs devops-engineer + UI programmer; UI programmer
    is the cross-check, not the primary).

### Control Manifest Rules

- Required: Cargo build emits a `spawn_range_live_update_contract`
  test binary that launches without OS error 740 on the same Windows
  host class for 5 consecutive runs without the rename workaround.
- Required: the chosen mechanism (a / b / c / d / e) is recorded in
  story evidence with the rationale.
- Required: any new Cargo dependency is gated behind
  `cfg(target_os = "windows")` (or the nearest Cargo equivalent)
  so non-Windows hosts incur zero extra dependency cost.
- Required: the smoke-harness rename workaround is either dropped or
  retained-as-documented-fallback (see AC4); the documentation under
  `docs/setup/dev-environment.md` is updated to reflect the new
  primary path while preserving the rename workaround as fallback
  documentation.
- Forbidden: renaming the test **source** file
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
- Forbidden: any change under `client/`, `server/`, `shared/src/`,
  or any other test source file.
- Forbidden: any production-gameplay behaviour change.
- Forbidden: any release claim, any QA-COND-* closure, any gate-check
  retry, any stage advance.
- Forbidden: closing or reopening story 005
  (`S13-OPS-WIN-APPCOMPAT-NOTE-001`).
- Forbidden: closing or reopening `TQ-S12-C7` (the Team-QA Sprint 12
  AppCompat condition); it remains informational.

---

## Story Classification

**Story type**: Implementation / ops hygiene -- bounded build-system
change scoped to one Cargo test target's manifest configuration (or
target rename if mechanism (d) is chosen).

This **IS** a:

- Cargo test-target configuration change (mechanism (a) / (b) / (c) /
  (e): add build-script + dependency; mechanism (d): rename the Cargo
  `[[test]] name` attribute).
- Build-system change scoped to a single test target.
- (Optionally) a docs follow-up at `docs/setup/dev-environment.md`'s
  AppCompat section to reflect the new primary path.

This is **NOT** a:

- Production-source change.
- Test source-file rename.
- Gameplay behaviour change.
- Release claim.
- Closure of any QA condition, any sprint disposition, or any
  prior story.

---

## Acceptance Criteria

All criteria are independently checkable. AC1 is the readiness gate
for `/dev-story`; AC2-AC6 are the implementation/closure gates for
`/story-done`.

- [x] **AC1 -- Story readiness passes before Sprint 16 `/dev-story`**:
  Verdict READY rerun against Sprint 16 activation HEAD prior to
  PROMPT 1068 `/dev-story` (deferred mechanism advisory preserved as
  non-blocking per the story body). Closure evidence: PROMPT 1068
  worker (`ed58e3d`) proceeded only after a documented READY rerun.

- [x] **AC2 -- Windows manifest embedded (or equivalent robust
  mechanism)**: Mechanism (d) chosen -- Cargo `[[test]] name = "..."`
  renamed from `spawn_range_live_update_contract` to
  `spawn_range_live_refresh_contract` (no trigger substring
  `update` / `install` / `setup` / `patch` / `uninst`). Source file
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
  NOT renamed (path attribute preserves it verbatim). Rationale
  recorded in
  `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
  §"Chosen Mechanism + Rationale" on `origin/main@bd374dd`.

- [x] **AC3 -- Test binary runs without OS error 740 for 5
  consecutive runs without rename workaround**: 5 consecutive
  invocations of
  `cargo test -p shared --test spawn_range_live_refresh_contract`
  captured in
  `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
  on `origin/main@bd374dd`. All 5 PASS with zero
  `os error 740` / "requested operation requires elevation"
  diagnostics across the run set.

- [x] **AC4 -- Smoke harness rename workaround removed or retained
  only as documented fallback**: `docs/setup/dev-environment.md`'s
  `## Windows AppCompat heuristic for Cargo test binaries` section
  updated with a follow-up subsection noting that mechanism (d) is
  now the primary path and that the per-run rename workaround is
  retained only as documented fallback. Existing Story 005
  AppCompat note text preserved unchanged.

- [x] **AC5 -- Cargo resource policy is required for any future
  Cargo command**: PROMPT 1068 `/dev-story` worker set the Sprint
  13-15 binding Cargo resource policy env vars
  (`CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'`,
  `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
  `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before each
  `cargo` invocation; attestation recorded in
  `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`.
  PROMPT 1072 paperwork closure invoked no Cargo / Trunk command.

- [x] **AC6 -- No production gameplay behaviour change, no release
  claim, no QA-COND closure**: PROMPT 1068 worker diff scoped to
  `shared/Cargo.toml` `[[test]] name` rename + `docs/setup/dev-environment.md`
  AppCompat follow-up subsection + evidence artifact at
  `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`.
  Zero touch under `client/`, `server/`, `shared/src/`, or
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
  `production/stage.txt` remains `Polish`; `production/sprint-status.yaml`
  top-level fields preserved through PROMPT 1071 integration tip
  `488a9cd` and PROMPT 1070 merge tip `bd374dd`. No release readiness,
  `S8-QA-001-W1` closure, `QA-COND-0005` / `QA-COND-0006` advancement,
  `PAW-TD-*-a` resolution, or `TQ-S12-C7` closure claimed by this row.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `shared/Cargo.toml` | Either (mechanism (a) / (b) / (c) / (e)) add `[target.'cfg(windows)'.build-dependencies]` for the chosen manifest-embed crate **OR** (mechanism (d)) modify the `[[test]] name = "spawn_range_live_update_contract"` line to a name that does not contain the substring `update` while leaving the `path = "../tests/unit/protocol/spawn_range_live_update_contract_test.rs"` source-file pointer unchanged. |
| `shared/build.rs` (NEW under mechanism (a) / (b) / (c) / (e); NOT touched under mechanism (d)) | Build script invoking the chosen crate (`embed-resource` / `winresource` / `embed-manifest` / equivalent) to embed the `asInvoker` manifest into the test-target output on `cfg(target_os = "windows")` only. |
| `shared/resources/appcompat-asinvoker.manifest` (NEW under mechanism (a) / (b) / (c); NOT touched under mechanism (d); path TBD by worker) | XML manifest file with `<requestedExecutionLevel level="asInvoker"/>`. Path / filename TBD by worker; the implementing prompt records the chosen path in story evidence. |
| `docs/setup/dev-environment.md` (existing on `origin/main` per Story 005) | Appended follow-up subsection under the existing `## Windows AppCompat heuristic for Cargo test binaries` section noting that the manifest mechanism is now the primary path and the rename workaround is retained only as documented fallback. **Existing Story 005 AppCompat content NOT deleted.** |
| `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md` (NEW; path may shift to `sprint-17-...` if the row carries forward) | Evidence artifact: 5 consecutive cargo test invocations captured without the rename workaround; chosen mechanism + rationale recorded. |
| This story file | Status update on `/story-done`. |

---

## Required Skills

- **Engine skills**: NOT required (no Bevy / Lightyear change).
- **Cargo / build-system literacy**: required for mechanism (a) /
  (b) / (c) / (e) -- the worker must understand
  `[target.'cfg(windows)'.build-dependencies]` scoping and how Cargo
  invokes `build.rs` for test targets.
- **Windows manifest literacy**: required for mechanism (a) / (b) /
  (c) / (e) -- the worker must produce a well-formed
  `application/v1` manifest XML with the correct
  `<requestedExecutionLevel level="asInvoker"/>` element.
- **Implementing agent**: `devops-engineer` (primary per
  `production/sprints/sprint-16.md` Nice to Have row); UI programmer
  is the cross-check column on the sprint row but is NOT the
  primary owner.

---

## Evidence Path

The combined Cargo test-target configuration delta (+ optional
`build.rs` + manifest XML, depending on mechanism) **is** the
artifact for this story. The evidence file at
`production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
(NEW under `/dev-story`) records:

**Required evidence content** (deferred to implementation prompt):

- Chosen mechanism (a / b / c / d / e) + rationale.
- Pre-impl base hash on `origin/main`.
- Impl commit hash.
- 5 consecutive cargo invocations of the test target without the
  rename workaround, with stdout `test result: ok. 5 passed; 0
  failed; 0 ignored; 0 measured; 0 filtered out` for all 5 runs
  and zero `os error 740` diagnostics across all runs.
- Cargo resource-policy authorisation reference (per AC5).
- Diff stat showing zero touch under `client/`, `server/`,
  `shared/src/`, and `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
- Smoke-harness implication note (drops the rename step from the
  next smoke run's invocation guidance; existing
  `production/qa/smoke-sprint-14-2026-05-16-rerun.md` is NOT
  retro-edited).

---

## Regression Commands Expected

For the implementation prompt (`/dev-story`):

- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'server/**' 'shared/src/**' 'tests/unit/protocol/spawn_range_live_update_contract_test.rs' 'production/stage.txt' 'production/gate-checks/**'`
  (verifies AC6: zero touch under production gameplay surface and
  zero touch of the protected disposition artifacts)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`
- `cargo test -p shared --test spawn_range_live_update_contract`
  (or under the renamed target name if mechanism (d) chosen),
  invoked 5 times consecutively without the rename workaround
  (AC3 evidence capture); each invocation must request Cargo
  resource policy authorisation (AC5).
- `cargo check --workspace --all-targets` (sanity check that the
  added `build.rs` / dependency does not break the wider workspace
  build).

No `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or
`/qa-plan` is run under either the authoring prompt (PROMPT 1057)
or the implementation prompt (`/dev-story`). Smoke harness
implications land in the next `/smoke-check` invocation that runs
after this story integrates to `origin/main`.

---

## Out of Scope

- Renaming the test **source** file
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
- Any production-source change under `client/`, `server/`,
  `shared/src/`.
- Any other Cargo test target's manifest configuration (this story
  is scoped to `spawn_range_live_update_contract` only; other test
  targets do not currently trigger the AppCompat heuristic and are
  not in scope).
- Closure of `TQ-S12-C7` Sprint 12 AppCompat informational
  condition (preserved as informational; closure, if ever, is a
  separate `/team-qa` paperwork run).
- Closure or reopening of Story 005
  (`S13-OPS-WIN-APPCOMPAT-NOTE-001`); Story 005 remains DONE on
  `origin/main` and is referenced here as background only.
- Sprint 16 activation; closure of `S8-QA-001-W1`; advance of
  `QA-COND-0005` / `QA-COND-0006`; advance of `PAW-TD-*-a`;
  Polish->Release gate-check retry.
- Final-art or asset-production work.
- Smoke harness rewrite (only the rename-step removal is in
  scope; the wider smoke harness invocation remains unchanged).
- Retro-editing prior smoke / Team-QA / close-out evidence files
  (those are immutable evidence artifacts).
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt (PROMPT 1057).

---

## Dependency Notes Against Sprint 16 Active Scope

- **Build-system scope vs Sprint 16 UI scope**: this story touches
  `shared/Cargo.toml` (and optionally `shared/build.rs` plus a
  manifest XML file). Sprint 16 Should Have row
  `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` touches
  `client/src/ui/design_tokens/` or `client/src/ui/primitives/`
  (TBD) -- file-disjoint by surface. No conflict risk with the
  card-slot primitive story.
- **No conflict with Sprint 16 Must Have carry**: the conditional
  Must Have row `S11-HUD-TIMER-EYEBALL-VISUAL-001` is a
  human-operator manual visual check at the HUD-timer surface;
  it touches no Cargo file and no test target. File-disjoint by
  surface.
- **No conflict with Sprint 16 Nice to Have peer row**: the peer
  Nice to Have row `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`
  touches `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43`.
  File-disjoint by surface (different test file under a
  different test directory; different Cargo crate -- this story
  edits `shared/Cargo.toml`; the dead-code warning row edits a
  test under the integration tree which is owned by the
  workspace root or its respective crate, not `shared/`).
- **Cargo invariant tests**: the `protocol_completeness_invariant`
  `[[test]]` block at `shared/Cargo.toml:28-30` is **unaffected**
  by this story (this story modifies only the
  `spawn_range_live_update_contract` `[[test]]` block at
  `shared/Cargo.toml:17-19` plus optionally the
  `[target.'cfg(windows)'.build-dependencies]` table). `S13-PROTO-INVARIANT-001`
  evidence preserved.
- **Smoke gate**: after this story lands and integrates, the next
  `/smoke-check` invocation should drop the per-run rename step
  from its invocation script and document the change in its own
  report. This story does NOT retroactively edit prior smoke
  reports.

---

## Authoring / Implementation / Closure Trail

- **PROMPT 1057** (2026-05-17, doc-only story authoring at
  `origin/main@8bec9dc`): Story authored as Sprint 16 Nice to Have
  candidate. **Sprint 16 NOT activated** by PROMPT 1057. Sprint 15
  close-out disposition (still pending at authoring time) and all
  prior sprint dispositions unchanged. No Cargo / Trunk / CI
  command invoked. No production-source change. Files touched by
  PROMPT 1057: this story file (NEW) +
  `production/epics/devops/EPIC.md` (row added for story 006).
- **(Future) PROMPT TBD `/story-readiness`**: expected verdict
  `READY` against Sprint 16 activation HEAD (advisories on the
  deferred mechanism choice (a)-(e) are intentional and do NOT
  block readiness).
- **(Future) PROMPT TBD `/dev-story`**: implementing worker
  chooses one mechanism (a) / (b) / (c) / (d) / (e), lands the
  Cargo / build-system delta on a worker branch, captures 5
  consecutive cargo invocations as evidence, and pushes the
  worker branch only.
- **(Future) PROMPT TBD integration**: `--no-ff` merge of worker
  branch into `origin/main`.
- **(Future) PROMPT TBD `/story-done`**: paperwork-only closure
  flipping this story's AC1-AC6 checkboxes and the sprint-status
  row.

### Conditions carried forward unchanged by PROMPT 1057

- Sprint 15 close-out disposition (pending at authoring time).
- Sprint 14 disposition `closed-with-conditions` per PROMPT 987.
- Sprint 13 disposition `closed-with-conditions` per PROMPT 894.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817.
- Sprint 11 disposition `closed-with-conditions` per PROMPT 792.
- Sprint 10 disposition `closed-with-conditions` per PROMPT 763.
- Stage `Polish` (production/stage.txt NOT modified).
- PROMPT 761 Polish->Release gate-check FAIL preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` (friend-game scope) + `QA-COND-0006` (playtest
  deferred) accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim, including `TQ-S12-C7`
  AppCompat informational condition (this story does NOT close
  `TQ-S12-C7`).
- Sprint 10 / Sprint 11 / Sprint 12 / Sprint 13 / Sprint 14 /
  Sprint 15 closeouts preserved unchanged (where each has landed).
- Story 005 (`S13-OPS-WIN-APPCOMPAT-NOTE-001`) DONE on
  `origin/main@807c3e7` preserved unchanged.
- PROMPT 683-era runtime divergence question preserved (folded
  into Sprint 12 story 019 `cannot-reproduce` closure).
- All `/story-done` closures across Sprint 10 -> Sprint 15
  preserved unchanged on `origin/main`.
- 24 PROMPT 1022 audit findings preserved as report-only inputs;
  none pulled by this story.
- Live QA snapshot tooling phase (PROMPT 1019 / 1020 / 1021 /
  1023) preserved unchanged on `origin/main`.

### Explicitly NOT claimed by PROMPT 1057

- Sprint 16 activation.
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`
  remains accepted-risk).
- Playtest / fun-hypothesis validation (`QA-COND-0006` remains
  accepted-risk).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN).
- Final-art / asset-production completion (`PAW-TD-*-a` preserved).
- Polish->Release gate-check retry.
- Stage advance from Polish to Release.
- Closure of `TQ-S12-C7` Sprint 12 AppCompat informational
  condition.
- Closure or reopening of Story 005
  (`S13-OPS-WIN-APPCOMPAT-NOTE-001`).
- Any Cargo / Trunk / CI command invocation under the authoring
  prompt.
- Activation of Sprint 16 peer rows (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`,
  `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`); each has its own
  story-authoring / readiness / implementation trail and is
  authored under its own prompt.

PROMPT 1057 did NOT run: `/smoke-check`, `/team-qa`, `/gate-check`,
`/release-check`, `/dev-story`, `/story-readiness`, `/qa-plan`, or
any Cargo command. PROMPT 1057 did NOT touch: `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`,
`.github/`, `Trunk.toml`, `production/stage.txt`,
`production/sprints/`, `production/qa/`, `production/sprint-status.yaml`,
`production/gate-checks/`, `production/session-state/`, or any
story file other than this one + `production/epics/devops/EPIC.md`
(row addition only; no existing row status changed).

### Closure trail (added 2026-05-17 by PROMPT 1072)

- **PROMPT 1064** (2026-05-17, paperwork-only Sprint 16 activation at
  `origin/main@6f9308c`): flipped `production/sprint-status.yaml`
  top-level `sprint: 15 -> 16` + `status: closed-with-conditions ->
  active`; this row entered as Nice to Have `ready` on the Sprint 16
  active set.
- **PROMPT 1066** (2026-05-17, paperwork-only Sprint 16 QA plan at
  `origin/main@c908f73`): authored `production/qa/qa-plan-sprint-16.md`
  naming this row's targeted Windows/MSVC validation gates (AC3
  BLOCKING 5 consecutive `cargo test` invocations without rename
  workaround / no `os error 740`; AC5 Cargo resource policy mandatory).
- **PROMPT 1068** `/dev-story` (worker
  `ed58e3d98ab0ddc851dfcf6fdad84680a0006937`): chose Mechanism (d)
  Cargo `[[test]] name` rename
  `spawn_range_live_update_contract -> spawn_range_live_refresh_contract`,
  preserving the source file path. Captured 5 consecutive
  `cargo test` runs without rename workaround in
  `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`.
- **PROMPT 1071** integration commit
  `488a9cd96e6bce2b9ecca7db172ab1be1acc8878`: merged the PROMPT 1068
  worker into `origin/main` via `--no-ff`. Zero touch under `client/`,
  `server/`, `shared/src/`, or the test source file.
- **PROMPT 1070** final integration tip
  `bd374ddf4360f01838830cef22bc36e3763fb8c1` (merge of `origin/main`
  into the dead-code warning row's integration branch) carries this
  row's `488a9cd` commit forward on `origin/main`.
- **PROMPT 1072** (this `/story-done`, 2026-05-17, paperwork-only at
  `origin/main@bd374dd`): flipped this story's AC1-AC6 checkboxes to
  `[x]`; flipped the sprint-status row to `status: done` with
  `completed: 2026-05-17`; recorded the closure trail. No Cargo /
  Trunk / CI command. No production source edit.

### Conditions carried forward unchanged by PROMPT 1072

- Sprint 16 disposition: `active` (UNCHANGED; Sprint 16 NOT
  closed-out by this row).
- Stage `Polish` (UNCHANGED; `production/stage.txt` NOT touched).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; NO
  retry attempted.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` (friend-game scope) + `QA-COND-0006` (playtest
  deferred) accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim (TQ-S12-C7 explicitly NOT
  closed by this row).
- Story 005 (`S13-OPS-WIN-APPCOMPAT-NOTE-001`) DONE on
  `origin/main@807c3e7` preserved unchanged.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- All four closed Sprint 15 rows preserved unchanged on
  `origin/main`.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 13->14->15->16
  carry preserved as human-operator-blocked; NOT closed by this row.
- Card-slot row `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` remains Sprint 16
  Should Have ready; NOT closed by this row (PROMPT 1067 still
  running at closure time).

### Explicitly NOT claimed by PROMPT 1072

- Sprint 16 close-out.
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have
  human-operator-blocked Sprint 13->14->15->16 carry; closure remains
  gated on human-operator screenshot capture; no LLM /story-done
  authorised).
- Closure of `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` (Should Have).
- Public release readiness; release-candidate readiness; full game
  completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`
  remains accepted-risk).
- Playtest / fun-hypothesis validation (`QA-COND-0006` remains
  accepted-risk).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN).
- Final-art / asset-production completion (`PAW-TD-*-a` preserved).
- Polish->Release gate-check retry; stage advance from Polish to
  Release.
- Closure of `TQ-S12-C7` Sprint 12 AppCompat informational
  condition.
- Any Cargo / Trunk / CI command invocation under PROMPT 1072.

PROMPT 1072 did NOT run: `/smoke-check`, `/team-qa`, `/gate-check`,
`/release-check`, `/dev-story`, `/story-readiness`, or `/qa-plan`.
PROMPT 1072 did NOT touch: `client/`, `server/`, `shared/`, `tests/`,
`Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`,
`docs/setup/dev-environment.md`, `production/qa/evidence/*`,
`production/qa/qa-plan-sprint-16.md`, `production/sprints/*`, or
`production/stage.txt`.

`006: S15-OPS-APPCOMPAT-MANIFEST-001: DONE`
