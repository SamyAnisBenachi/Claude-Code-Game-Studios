# Sprint 16 -- S15-OPS-APPCOMPAT-MANIFEST-001 Implementation Evidence

> **Story**: `S15-OPS-APPCOMPAT-MANIFEST-001`
> **Story file**: `production/epics/devops/story-006-appcompat-manifest.md`
> **Sprint**: Sprint 16 Nice to Have row (active per PROMPT 1064 `c908f73`).
> **Implementation prompt**: PROMPT 1068 (`/dev-story`).
> **Authored**: 2026-05-17.
> **Worker**: devops-engineer role; single-context worker; no spawned
> CCGS subagents; no `/story-readiness`, `/story-done`, `/smoke-check`,
> `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` invoked
> under this implementation prompt.
> **Source-of-truth at implementation**:
> `origin/main@7a78b257cdcd8b76f439f7264b31648b3ae1261c` (Sprint 16
> QA plan landing tip, PROMPT 1066 `qa(s16): author Sprint 16 QA plan`).
> **Worker branch**: `work/s16-appcompat-manifest`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s16-appcompat-manifest`

---

## Chosen Mechanism + Rationale

**Mechanism (d) -- Cargo `[[test]] name = "..."` rename.**

The Cargo `[[test]]` block at `shared/Cargo.toml` lines 17-19 on
`origin/main@7a78b25` (verified on the implementation worktree at
identical line numbers) was renamed:

```toml
# BEFORE
[[test]]
name = "spawn_range_live_update_contract"
path = "../tests/unit/protocol/spawn_range_live_update_contract_test.rs"

# AFTER (mechanism (d))
[[test]]
name = "spawn_range_live_refresh_contract"
path = "../tests/unit/protocol/spawn_range_live_update_contract_test.rs"
```

The Cargo-emitted test binary filename is therefore
`spawn_range_live_refresh_contract-<hash>.exe`, which contains no
AppCompat installer-detection trigger substring (`update`,
`install`, `setup`, `patch`, `uninst`).

**The source file `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
is NOT renamed.** The `path` attribute still pointers it verbatim,
preserving:

- The `S13-PROTO-INVARIANT-001` evidence chain
  (`production/qa/evidence/sprint-13-proto-invariant-evidence.md`).
- The Story 005 (`S13-OPS-WIN-APPCOMPAT-NOTE-001`) closure
  evidence on `origin/main@807c3e7`.
- All 23 cross-references to `spawn_range_live_update_contract`
  across `production/sprints/`, `production/qa/`, `production/epics/`,
  `production/sprint-status.yaml`, `production/session-state/`,
  `docs/setup/dev-environment.md`, and the Sprint 13 / 14 / 15 / 16
  authoring trail.

### Rationale for choosing mechanism (d) over (a) / (b) / (c) / (e)

The story's mechanism enumeration (`production/epics/devops/story-006-appcompat-manifest.md`
§"Problem Class / Prevention Target") presents 5 options. Mechanism
(d) was chosen on the following grounds, recorded here per the
story's evidence requirement:

1. **Zero new build dependency**: mechanisms (a) `embed-resource`, (b)
   `winresource`, and (c) `embed-manifest` each introduce a new
   `[target.'cfg(windows)'.build-dependencies]` entry plus the
   transitive dependency closure of that crate. Mechanism (d) adds
   zero crates -- no crates.io fetch, no `Cargo.lock` delta beyond
   the Cargo manifest itself, no resource-compiler invocation, and
   no MSVC linker resource-handling path.
2. **Zero `build.rs`**: mechanisms (a) / (b) / (c) / (e) require a new
   `shared/build.rs` (or equivalent) build script. Mechanism (d)
   does not. A new `build.rs` is itself a new build-system surface
   that must be maintained and that complicates the `shared` crate's
   compile graph (`shared/` currently has no `build.rs` -- see
   `shared/Cargo.toml`). Avoiding the build-script introduction
   preserves the simplicity of the `shared` crate's build.
3. **Zero new resource file**: mechanisms (a) / (b) / (c) require a
   Windows `.manifest` (XML) and typically a `.rc` resource file as
   well. Mechanism (d) requires no new files at all.
4. **Cross-platform clean**: mechanism (d) is OS-agnostic at the
   Cargo manifest level -- the rename applies identically on
   Windows, Linux, macOS, and WASM builds. Mechanisms (a) / (b) / (c)
   / (e) require Windows-conditional gating
   (`[target.'cfg(windows)'.build-dependencies]` and
   `cfg(target_os = "windows")` build-script branches).
5. **Robustness across Windows host classes**: mechanism (d) prevents
   the AppCompat heuristic from ever seeing a triggering filename
   on any Windows host class (MSVC, GNU, future toolchains). The
   manifest-embed mechanisms only suppress the heuristic on hosts
   where the AppCompat heuristic respects the embedded manifest's
   `asInvoker` declaration -- which is the documented behaviour, but
   mechanism (d) sidesteps the heuristic entirely rather than
   negotiating with it.
6. **Pre-authorised by project documentation**: Story 005 (`S13-OPS-WIN-APPCOMPAT-NOTE-001`)
   already documents this exact rename target name
   `spawn_range_live_refresh_contract` as the canonical permanent-fix
   option (`docs/setup/dev-environment.md` lines 217-234 -- "Rename
   the Cargo test target ... could be renamed to
   `spawn_range_live_refresh_contract` to avoid `update`"). Choosing
   mechanism (d) with this name reuses the already-documented
   project-internal canonical rename.
7. **Cargo policy alignment**: the Cargo policy preamble required by
   the implementation prompt (`CARGO_PROFILE_DEV_DEBUG=0`,
   `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
   `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`,
   `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`) optimizes for
   minimal build artifacts and minimal disk pressure. Adding a new
   `build.rs` + new build-dependency closure works against that
   posture; the rename does not.

### Trade-off acknowledged

Mechanism (d)'s documented cost is semantic drift between the Cargo
test-target name (`spawn_range_live_refresh_contract`) and the
protocol contract being tested (the "live update" protocol contract
for `SpawnRangeChanged` events). This drift is mitigated by:

- The `path` attribute still pointers the canonical source file
  `tests/unit/protocol/spawn_range_live_update_contract_test.rs` --
  anyone investigating the test target follows the path back to the
  source-of-truth filename.
- A multi-line `# S15-OPS-APPCOMPAT-MANIFEST-001 (PROMPT 1068,
  mechanism (d)):` comment block above the `[[test]]` entry in
  `shared/Cargo.toml` records the rationale inline at the Cargo
  manifest site.
- The `## Windows AppCompat heuristic for Cargo test binaries`
  section in `docs/setup/dev-environment.md` gets a new
  `### S15-OPS-APPCOMPAT-MANIFEST-001 follow-up` subsection
  documenting the rename and the path forward.
- The Sprint 16 sprint plan, Sprint 13 Story 005 doc, Sprint 13
  story 006 source-finding section, and this evidence file all
  reference the original target name verbatim, ensuring the
  evidence trail through the Sprint 12 / 13 / 14 / 15 / 16 deferral
  chain is preserved.

This trade-off is accepted; the test semantics, source file, and
protocol-invariant claim are unchanged.

---

## Pre-Impl Base Hash on origin/main

`origin/main@7a78b257cdcd8b76f439f7264b31648b3ae1261c`

Verified via `git rev-parse origin/main` at the start of the
implementation run. PROMPT 1066 `qa(s16): author Sprint 16 QA plan
(PROMPT 1066)` is the Sprint 16 QA plan landing tip; PROMPT 1064
`c908f73c1e72c4be17b1ed1ada37d1e6` is the Sprint 16 activation
commit; PROMPT 1065 `/story-readiness` batch verdict
`ALL-NON-HUMAN-READY` (report at `reports/PROMPT-1065-S16-Story-Readiness-Batch.md`)
is preserved against the activation HEAD.

## Impl Commit Hash

Recorded at commit time below. The single implementation commit on
worker branch `work/s16-appcompat-manifest` is titled
`fix(devops): prevent AppCompat elevation for spawn range test (PROMPT 1068)`.

## Files Changed by Implementation Commit

- `shared/Cargo.toml` -- rename of the `[[test]] name` attribute
  from `spawn_range_live_update_contract` to
  `spawn_range_live_refresh_contract`; `path` attribute unchanged;
  inline rationale comment added.
- `docs/setup/dev-environment.md` -- new subsection
  `### S15-OPS-APPCOMPAT-MANIFEST-001 follow-up (Sprint 16; PROMPT 1068)`
  appended within the existing `## Windows AppCompat heuristic for
  Cargo test binaries` H2 section, before the section's closing
  `---` separator. Existing Story 005 / PROMPT 887 content is NOT
  deleted; new subsection appends only.
- `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
  -- THIS file (NEW).

## Files NOT Changed (per story §"Forbidden" + AC6)

- `tests/unit/protocol/spawn_range_live_update_contract_test.rs` --
  the test source file is unchanged.
- `client/`, `server/`, `shared/src/` -- no production gameplay
  source change.
- `tests/integration/`, `tests/unit/` (other than the path attribute
  reference above), `tests/invariants/` -- no other test file
  changed.
- `production/sprint-status.yaml` -- top-level fields unchanged; no
  row status flip under this prompt (that is `/story-done` work,
  forbidden here).
- `production/sprints/sprint-16.md`, `production/sprints/sprint-15.md`,
  and all prior sprint plan files -- unchanged.
- `production/stage.txt` -- unchanged (remains `Polish`).
- `production/qa/qa-plan-sprint-16.md` -- unchanged (PROMPT 1066
  artifact preserved verbatim).
- `production/session-state/*` -- not modified by this worker.
- `production/gate-checks/` -- unchanged.
- `production/qa/smoke-sprint-14-2026-05-16-rerun.md` and all prior
  smoke / Team-QA / close-out evidence artifacts -- preserved
  verbatim (immutable evidence artifacts).
- `production/epics/devops/story-006-appcompat-manifest.md` -- NOT
  edited under this implementation prompt (the story file remains
  in its `Draft -- Sprint 16 candidate, NOT activated` status as
  authored by PROMPT 1057 + integrated by PROMPT 1062; `/story-done`
  paperwork is a separate prompt and is NOT run here).
- `Cargo.lock` -- not modified (the rename does not alter the
  workspace dependency graph).
- `.cargo/`, `.github/`, `Trunk.toml` -- unchanged.

## Tests Run -- Pass / Fail

### Repeated test gate (AC3): 5 consecutive `cargo test` invocations

All 5 invocations executed with the Cargo policy preamble applied
(see "Cargo resource policy applied" below) and **without** any
per-run `cp ... srluc_appcompat_renamed.exe` rename workaround.
The renamed test target was invoked directly as
`cargo test -p shared --test spawn_range_live_refresh_contract`
(the renamed target name; AC3 explicitly authorises invoking under
the renamed target name when mechanism (d) is chosen).

| Run | Timestamp | Test result | Exit code | `os error 740` / "requires elevation" |
|-----|-----------|-------------|-----------|---------------------------------------|
| 1   | 2026-05-17T16:37:04+01:00 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 0 | none |
| 2   | 2026-05-17T16:37:05+01:00 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 0 | none |
| 3   | 2026-05-17T16:37:05+01:00 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 0 | none |
| 4   | 2026-05-17T16:37:05+01:00 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 0 | none |
| 5   | 2026-05-17T16:37:06+01:00 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 0 | none |

Aggregate: **5 / 5 invocations PASSED**; **0 / 5 invocations FAILED**;
**0 occurrences of `os error 740`**; **0 occurrences of "requires
elevation" / "The requested operation requires elevation"** across
all 5 runs.

The Cargo-emitted test binary across all 5 runs was
`D:/_DEV/cargo-target/ccgs-msvc\debug\deps\spawn_range_live_refresh_contract-dcf81efb462e9aa5.exe`
(verified via the `Running ..\tests\unit\protocol\spawn_range_live_update_contract_test.rs (...)`
line in each run's stdout). The binary filename contains no
AppCompat trigger substring; the heuristic does not fire.

### Test list (5 tests inside the test binary, unchanged from the
existing source file)

- `test_spawn_range_changed_is_ordered_after_fake_objective_destroyed_in_same_batch ... ok`
- `test_s2c_resolution_event_remains_registered_on_reliable_channel ... ok`
- `test_spawn_range_is_not_registered_as_standalone_protocol_message ... ok`
- `test_spawn_range_changed_schema_round_trips_through_resolution_batch ... ok`
- `test_player_snapshot_spawn_range_cells_remains_public_recovery_field ... ok`

(Test names and contents are unchanged; the source file is unchanged.)

### Optional sanity check: `cargo check --workspace --all-targets`

NOT run. Justification: the change is a Cargo `[[test]] name`
rename only; the `shared` crate's `cargo test` build succeeded
on the very first invocation under the renamed target (mid-build
log line: `Compiling shared v0.1.0` then `Finished test profile
[optimized] target(s) in 3.15s`), and the 5 subsequent invocations
all reused the cached build with sub-second incremental times
(`Finished test profile [optimized] target(s) in 0.13s`). The
broader workspace `cargo check` was not invoked under this
prompt because:

- the rename does not affect any production-source dependency
  closure (the `[[test]]` block is local to the `shared` crate's
  test-target list);
- adding a workspace-wide `cargo check` would consume substantial
  D: disk budget and time without informing AC2 / AC3 / AC6;
- AC3's evidence requirement is satisfied without it.

If a downstream consumer requires the workspace sanity check, it
can be re-run under a separate prompt; the rename is a
Cargo-manifest-only change with zero behavior risk to the
workspace's other crates / test targets / build graph.

### `git diff --check` cleanliness

- `git diff --check origin/main...HEAD` -- **CLEAN** (zero
  whitespace / line-ending diagnostics).
- `git diff --cached --check` (run pre-commit) -- **CLEAN**.

### Cargo invariant test

The `protocol_completeness_invariant` `[[test]]` block at
`shared/Cargo.toml` lines 28-30 (now lines 41-43 post-rename due
to the inline rationale comment expansion above the renamed
`[[test]]` block) is **NOT modified**. `S13-PROTO-INVARIANT-001`
evidence preserved.

---

## Cargo Resource Policy Applied: YES

The Cargo policy preamble was applied as a single environment-variable
block before every Cargo invocation in the implementation session:

```bash
export CARGO_TARGET_DIR='D:/_DEV/cargo-target/ccgs-msvc'
export CARGO_PROFILE_DEV_DEBUG='0'
export CARGO_PROFILE_TEST_DEBUG='0'
export CARGO_INCREMENTAL='0'
export RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

(The story's PowerShell-syntax form `$env:CARGO_TARGET_DIR='...'` is
equivalent to the bash `export` form used here; the implementation
session was the Bash tool's subshell -- the resulting environment is
identical and the Cargo target-dir, profile-debug-suppression,
incremental-build-suppression, and `RUSTFLAGS` debuginfo-suppression
+ MSVC linker `/DEBUG:NONE` flag are all in effect.)

Verified by stdout echo of the env vars before the test-gate loop:

```
CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

All 5 test-gate invocations + the initial `--no-run` dry-run share
the same environment.

---

## Disk Cleanup Performed: NO

D: free space at implementation start: **802 GB free / 1.3 TB total
(39% used)**. This is well above the 40 GB cleanup threshold required
by the disk policy preamble. No cleanup was performed under this
prompt; the shared cargo target dir
`D:\_DEV\cargo-target\ccgs-msvc` was used as-is.

D: free space post-test-gate: not separately measured because no
cleanup was performed; the 5 cargo invocations reused the cached
build artifacts with negligible disk delta.

Paths cleaned: NONE.

---

## Smoke-Harness Implication Note (AC4)

The next `/smoke-check` invocation that runs after this story
integrates to `origin/main` should:

1. Drop the per-run `cp ... srluc_appcompat_renamed.exe` rename
   step from its invocation script.
2. Invoke the renamed test target directly as
   `cargo test -p shared --test spawn_range_live_refresh_contract`
   under the Cargo policy preamble.
3. Reference this evidence file at
   `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
   in its "How to reproduce" section as the canonical record of the
   rename mechanism.

This implementation prompt does **NOT** retro-edit any prior smoke
report. `production/qa/smoke-sprint-14-2026-05-16-rerun.md`,
`production/qa/smoke-sprint-12-2026-05-14.md`, and all earlier
smoke reports are preserved verbatim as immutable evidence
artifacts.

The follow-up subsection in `docs/setup/dev-environment.md`
(`### S15-OPS-APPCOMPAT-MANIFEST-001 follow-up (Sprint 16; PROMPT 1068)`)
documents the smoke implication for future reproductions and
explicitly retains the per-run rename workaround as documented
fallback for non-MSVC builds, hosts where the AppCompat heuristic
catches an incidentally-named third-party binary, and forensic /
archeological reproduction of the Sprint 12 / 13 / 14 evidence.

---

## Diff-Stat Against Production Gameplay Surface (AC6)

`git diff origin/main...HEAD --stat` is contained entirely within:

- `shared/Cargo.toml` (Cargo manifest -- not gameplay source).
- `docs/setup/dev-environment.md` (documentation -- not gameplay
  source).
- `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`
  (NEW evidence file -- not gameplay source).

Zero touch under:

- `client/**` -- verified.
- `server/**` -- verified.
- `shared/src/**` -- verified.
- `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
  -- verified (the test source file is unchanged).
- `production/stage.txt` -- verified (remains `Polish`).
- `production/sprint-status.yaml` top-level fields -- verified
  (`sprint: 16`, `status: active`, `stage: Polish` unchanged).
- `production/gate-checks/**` -- verified (PROMPT 761
  Polish->Release gate-check FAIL evidence preserved at
  `gate-polish-release-2026-05-12.md`).
- `production/sprints/sprint-16.md`, `production/sprints/*.md` --
  verified unchanged.
- `production/qa/qa-plan-sprint-16.md` -- verified unchanged
  (PROMPT 1066 artifact preserved).
- `production/qa/smoke-*.md`, `production/qa/team-qa-*.md` --
  verified unchanged.
- `production/session-state/*` -- verified unchanged.
- `production/epics/devops/story-006-appcompat-manifest.md` --
  verified unchanged (no AC checkbox flip; `/story-done` is a
  separate prompt and is NOT run under this implementation
  prompt).
- `Cargo.lock` -- verified unchanged.
- `.cargo/`, `.github/`, `Trunk.toml` -- verified unchanged.

---

## Explicit Non-Claims

This implementation prompt (PROMPT 1068) does **NOT** claim and does
**NOT** advance any of:

- Public release readiness; release-candidate readiness; full game
  completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`
  remains accepted-risk / friend-game scope only).
- Playtest / fun-hypothesis validation (`QA-COND-0006` remains
  accepted-risk / deferred).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN).
- Final-art / asset-production completion (`PAW-TD-*-a` preserved).
- Polish -> Release gate-check retry (the PROMPT 761 FAIL is
  preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`;
  no retry attempted; stage remains `Polish`).
- Stage advance from `Polish` to `Release`.
- Closure of `TQ-S12-C7` Sprint 12 Team-QA AppCompat informational
  condition (preserved as informational; closure, if ever, is a
  separate `/team-qa` paperwork run).
- Closure or reopening of Story 005
  (`S13-OPS-WIN-APPCOMPAT-NOTE-001`); Story 005 remains DONE on
  `origin/main@807c3e7` and is referenced here as background only.
- Closure of any other Sprint 16 story row -- specifically NOT
  flipping the `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  human-operator-blocked carry, `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`
  Should Have row, or `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`
  Nice to Have peer row.
- Sprint 16 close-out disposition; Sprint 15 close-out re-claim;
  Sprint 14 disposition re-claim; or any earlier sprint
  disposition.
- HUD timer visual / QA snapshot human-visual closure (those
  remain human-operator-blocked per PROMPT 1055 / PROMPT 1054).
- `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, `/qa-plan`, or `/story-readiness`
  on any Sprint 16 story row under this implementation prompt
  (this prompt is `/dev-story` for story 006 only -- it implements
  the AC2 / AC3 mechanism choice and provides AC3 evidence + AC4
  doc append + AC6 zero-production-touch diff; it does NOT run any
  of those companion skills as paperwork).
- Authorisation of a Sprint 17 row.
- Workspace `cargo check --all-targets` sanity sweep (deferred --
  see "Optional sanity check" above).

---

## /story-done Handoff Note

This evidence file is the canonical reference for the `/story-done`
paperwork prompt that follows. The paperwork prompt will flip the
AC1-AC6 checkboxes in
`production/epics/devops/story-006-appcompat-manifest.md` once it
inspects this evidence file + the worker-branch diff + the
integration-merge tip. AC1 readiness is already covered by PROMPT 1065's
`ALL-NON-HUMAN-READY` verdict on `origin/main@c908f73`; AC2 + AC3 +
AC4 + AC5 + AC6 are covered by this evidence file in conjunction
with the worker-branch diff and the post-integration `git log` /
`git diff` on `origin/main`.
