# Story 016: S15-TD-WORKSPACE-DEAD-CODE-WARNING-001 -- Workspace Dead-Code Warning Cleanup

> **Epic**: UI Clean-Pass
> **Story ID**: S15-TD-WORKSPACE-DEAD-CODE-WARNING-001
> **Status**: Draft -- Sprint 16 candidate, NOT activated
> **Layer**: Test hygiene -- workspace warning cleanup
> **Type**: Tech Debt -- test hygiene (single-helper cleanup in one test file)
> **Sprint**: Sprint 16 candidate (Nice to Have row per
> `production/sprints/sprint-16.md` §"Nice to Have" / §"Capacity"). NOT
> activated by this authoring run. Sprint 15 disposition (`active` with 4
> closed Should/Nice rows + 1 human-operator-blocked Must Have carry per
> PROMPT 1009 / PROMPT 1054) is preserved unchanged; Sprint 14 disposition
> (`closed-with-conditions`, `Polish` stage, PROMPT 987) is preserved
> unchanged.
> **Authored**: 2026-05-17 by PROMPT 1058
> **Authoring source-of-truth**: `origin/main@8bec9dca624a191fbc7c12409b2ea4690a1040ab`
> (PROMPT 1055 `chore(state): record P1 UI snapshot retest human block`)
> **Estimated effort**: ~0.1d (single-helper cleanup; one test file +
> targeted `cargo check` verification)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16 candidate. **No sprint is activated
by this authoring run.** PROMPT 1058 does NOT:

- Activate Sprint 16.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`,
  `production/sprints/sprint-14.md`,
  `production/sprints/sprint-15.md`,
  `production/sprints/sprint-16.md`, or any other sprint plan file.
- Modify `production/stage.txt`.
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Run `cargo`, `trunk`, or any CI command.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or
  `Trunk.toml`.
- Commit to `main` directly.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), Standard-tier hit-target conformance
(≥44px), playtest / fun-hypothesis validation (`QA-COND-0006`), full
playable-client manual QA, two-client GAME_OVER closure
(`S8-QA-001-W1`), final-art / asset-production completion
(`PAW-TD-*-a`), `Polish->Release` gate-check retry, or any stage
advance.

---

## Overview

Sprint 14 PROMPT 983 smoke run (`production/qa/smoke-sprint-14-2026-05-16-rerun.md`
§"`cargo check --workspace --all-targets`" / lines ~91-104) surfaced one
pre-existing dead-code warning under `cargo check`:

```
warning: function `count_with_image_node` is never used
  --> tests\integration\presentation\hand_ui_asset_wiring_test.rs:43:4
   |
43 | fn count_with_image_node<M: Component>(app: &mut App) -> usize {
   |    ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
warning: `client` (test "hand_ui_asset_wiring_test") generated 1 warning
```

The warning **predates** PROMPT 983 and PROMPT 982 (per the smoke note);
PROMPT 983 scope explicitly forbade `tests/` edits and so left the
warning in place. The warning was preserved through Sprint 15 (none of
the Sprint 15 rows touched
`tests/integration/presentation/hand_ui_asset_wiring_test.rs`), and is
named in the Sprint 16 plan draft (`production/sprints/sprint-16.md`
§"Nice to Have" row 2 / §"Pre-activation paperwork inventory" / §"Per-row
file-disjoint sketch") as the single-line, single-function Nice to Have
cleanup row.

Inspection of `tests/integration/presentation/hand_ui_asset_wiring_test.rs`
on `origin/main@8bec9dc` confirms:

- The helper `count_with_image_node<M: Component>(app: &mut App) -> usize`
  is defined at line 43 of the test file.
- No call site uses this helper anywhere in the workspace
  (`grep -rn count_with_image_node tests/` matches only the definition).
- A sibling helper `count_child_of_with<M: Component>(app: &mut App)
  -> usize` at line 48 is the one actually invoked by every
  `test_fan_slot_chrome_*_image_node_present` test (lines ~59-100+).
  `count_child_of_with` filters by `(With<M>, With<ImageNode>)` and
  counts the `&ChildOf` of those matches; this is the assertion shape
  every existing chrome-presence test uses.

The dead helper appears to be a leftover from an earlier authoring of
the PAW-002-f chrome ImageNode presence tests where the helper iterated
`&ImageNode` directly with a single `With<M>` filter; the surviving
helper iterates `&ChildOf` with `(With<M>, With<ImageNode>)`. The
surviving helper is strictly the more correct one for the PAW-002-f
"chrome child entity carries `ImageNode`" assertion shape.

The implementing worker MUST make a deliberate per-helper decision:

- **Option A -- remove the helper**: delete the `count_with_image_node`
  definition (and only that helper) if it has no call site and the
  intent it encoded is fully covered by `count_child_of_with`. No test
  coverage is lost; the test file's existing assertions remain on
  `count_child_of_with` exactly as authored.
- **Option B -- wire the helper into a meaningful assertion**: only if
  the worker discovers an actual coverage gap that the dead helper was
  meant to close (e.g. a "the chrome marker entity itself carries an
  `ImageNode`" invariant that the surviving `count_child_of_with`
  helper does not check). In that case, the worker adds **one** test
  function that calls `count_with_image_node` against an existing
  PAW-002-f marker, asserts the expected count, and documents in the
  test doc-comment why the new assertion is independent of the
  existing `count_child_of_with` coverage. No silent re-purposing
  of the helper.

The default is Option A (remove). Option B requires explicit producer
sign-off at `/dev-story` time and a documented rationale in the
commit message.

This story is **test-hygiene only**: it does not change product
behaviour, does not alter the PAW-002-f test surface coverage shape,
does not migrate, refactor, or rename any test file, and does not touch
any production code under `client/`, `server/`, or `shared/`.

---

## Scope

### In Scope

- A single edit to
  `tests/integration/presentation/hand_ui_asset_wiring_test.rs` that
  either (a) removes the `count_with_image_node` helper definition at
  line 43 (Option A, default) OR (b) wires the helper into exactly one
  new test function with documented rationale (Option B, producer
  sign-off required).
- A targeted verification (named in the `/dev-story` worker's commit
  message) that the dead-code warning is gone:
  - `cargo check --workspace --all-targets` (under the Cargo resource
    policy below) returns exit 0 and emits zero
    `warning: function \`count_with_image_node\` is never used`
    lines.
  - The bin's existing tests (`cargo test -p client --test
    hand_ui_asset_wiring_test`) continue to PASS unchanged
    (functional regression check on the four `test_fan_slot_chrome_*`
    tests already in the file).
- Preservation of the existing `count_child_of_with` helper, its
  signature, and every `test_fan_slot_chrome_*` test that uses it.
- A short note in the `/dev-story` worker's commit message stating
  which option (A or B) was chosen and why.

### Out of Scope

- Any change to `client/`, `server/`, `shared/`.
- Any change to any other file under `tests/integration/presentation/`
  or any other `tests/` subdirectory.
- Any change to `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or
  `Trunk.toml`.
- Adoption of a workspace-wide `#![deny(dead_code)]` or `cargo clippy`
  policy (separate Sprint 17+ candidate row if desired; out of scope
  here).
- Reformat / rename / refactor of any helper or test in the file
  beyond the single deletion or single new test function described
  above.
- Sprint 16 activation.
- Any QA-plan / smoke / Team-QA / gate-check / release-check artifact
  authoring.
- Stage advance, release-readiness claim, or closure of any
  `QA-COND-*` accept-risk row.
- `PAW-TD-*-a` advancement (placeholder-art accept-risk preserved).

---

## Acceptance Criteria

All BLOCKING.

- [ ] **AC1 -- `/story-readiness` PASS before Sprint 16 `/dev-story`**:
  This story file passes `/story-readiness` against the Sprint 16
  activation HEAD (commit hash recorded by the activation prompt). All
  required headings are present (Status / Overview / Scope / Acceptance
  Criteria / Implementation Notes / Worker Contract). Story ID matches
  Sprint 16 plan row. No open design question remains for the
  `/dev-story` worker. Verification: `/story-readiness` verdict line
  reads READY.

- [ ] **AC2 -- Helper handled deliberately, no silent test coverage
  reduction**: The implementing `/dev-story` worker either (a) removes
  the `count_with_image_node` helper definition entirely (Option A,
  default) **or** (b) wires it into exactly one new test function with
  a documented rationale in the test doc-comment AND the commit
  message (Option B, producer sign-off). The four existing
  `test_fan_slot_chrome_*_image_node_present` tests (`HandCardFrame`,
  `StatBadgeAtk`, `StatBadgeHp`, `StatBadgeMp`) remain functionally
  unchanged: their assertion bodies, expected counts, and call site
  to `count_child_of_with` are preserved verbatim. Verification:
  `git diff origin/main...HEAD -- tests/integration/presentation/hand_ui_asset_wiring_test.rs`
  shows only the deletion of the `count_with_image_node` block
  (Option A) or the deletion of the helper plus the addition of a
  single new test function that names the helper in its body (Option
  B); no other diff hunks inside the file.

- [ ] **AC3 -- Dead-code warning is gone (targeted check)**: A
  targeted `cargo check --workspace --all-targets` invocation (under
  the Cargo resource policy below) returns exit 0 and its stderr / stdout
  contains zero occurrences of
  `warning: function \`count_with_image_node\` is never used`.
  Worker captures the relevant `cargo check` tail (or the relevant
  `--message-format=human` line range) in the commit message or a
  worker-tracked log line. **The full workspace `cargo test` is NOT
  required for this row** (per Sprint 15 QA Policy §"Test Scope Per
  Prompt Type" line 104: "Normal implementation workers ... run
  story-prescribed targeted tests only"). A targeted `cargo check`
  is the policy-correct verification.

- [ ] **AC4 -- Existing hand UI asset wiring coverage remains intact**:
  `cargo test -p client --test hand_ui_asset_wiring_test --no-fail-fast`
  PASSES with the same count of green tests as on `origin/main` at
  Sprint 16 activation HEAD (expected: 4 PAW-002-f chrome-presence
  tests minimum; worker confirms exact count from the activation
  HEAD before running). No `#[ignore]` is introduced. No test is
  removed or renamed. Worker MAY add **at most one** new test
  function (Option B path only).

- [ ] **AC5 -- Cargo resource policy is required for any future Cargo
  command**: The future `/dev-story` worker MUST set the Sprint 14 \
  Sprint 15 binding Cargo resource policy environment variables for
  every `cargo` invocation on Windows / MSVC (mirrors the binding
  precedent from PROMPT 815 disk-pressure incident; preserved across
  Sprint 14 / 15 worker + integration prompts per `production/qa/qa-plan-sprint-15.md`
  §"Cargo Resource Policy on Windows/MSVC"):

  ```powershell
  $env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
  $env:CARGO_PROFILE_DEV_DEBUG='0'
  $env:CARGO_PROFILE_TEST_DEBUG='0'
  $env:CARGO_INCREMENTAL='0'
  $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
  ```

  Verification: worker commit message or worker-tracked log includes
  a one-line attestation that these env vars were set before each
  `cargo check` / `cargo test` invocation.

- [ ] **AC6 -- No production behavior change, no release claim, no
  `QA-COND-*` closure**: `git diff origin/main...HEAD --stat` against
  the worker branch shows NO change under `client/src/`, `server/src/`,
  or `shared/src/`. NO release-readiness claim is made. NO
  `QA-COND-0005` Standard-tier accessibility advancement is claimed.
  NO `QA-COND-0006` playtest validation advancement is claimed. NO
  `PAW-TD-*-a` placeholder-art accept-risk advancement is claimed. NO
  closure of `S8-QA-001-W1` two-client GAME_OVER row is claimed. NO
  `Polish->Release` retry is attempted. NO stage advance is performed.
  Verification:
  `git diff origin/main...HEAD --stat -- 'client/src/' 'server/src/' 'shared/src/' 'production/sprint-status.yaml' 'production/sprints/' 'production/stage.txt' 'production/session-state/' 'production/qa/'`
  is empty.

- [ ] **AC7 -- Worker branch scope contained**: The implementing
  `/dev-story` worker pushes a single worker branch (suggested slug
  `work/s16-workspace-dead-code-warning` or producer-renamed at
  activation) with at most these changed paths:
  - `tests/integration/presentation/hand_ui_asset_wiring_test.rs`
    (one deletion + optionally one added test function).
  - `production/epics/ui-clean-pass/story-016-workspace-dead-code-warning.md`
    (status-block update for closure paperwork by `/story-done`,
    if and when that runs in a later prompt).

  No edit to any other path. Worker MUST NOT push to `main`. If
  branch push is blocked, worker leaves a local commit ready and
  reports the exact commit hash + blocker per the standard worker
  protocol.

---

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `tests/integration/presentation/hand_ui_asset_wiring_test.rs` | One deletion at line 43 (Option A, default) of the `count_with_image_node` helper block; OR Option B per AC2 (producer sign-off required). |

### Forbidden files

- Everything under `client/`, `server/`, `shared/`.
- All other files under `tests/` (including all other files under
  `tests/integration/presentation/`).
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/qa/evidence/*`.
- All other `production/epics/*` story files (no cross-epic edit).
- All `docs/` files.
- `.claude/`, `AGENTS.md`, `CLAUDE.md`.

### Target citations

- Sprint 16 plan row source: `production/sprints/sprint-16.md` §"Nice
  to Have" row 2 (`S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`); §"Sprint
  16 plan rationale" line 3 (`Discharge two small ops / test-harness
  hygiene rows`) bullet 2 (lines ~236-240); §"Pre-activation paperwork
  inventory" row 3 (line ~514); §"Per-row file-disjoint sketch" row
  for this slug (line ~591); §"Capacity" Nice to Have subtotal
  (lines ~325-329).
- Warning source: `production/qa/smoke-sprint-14-2026-05-16-rerun.md`
  §"`cargo check --workspace --all-targets`" (lines ~89-104).
- Target test file: `tests/integration/presentation/hand_ui_asset_wiring_test.rs`.
- Target symbol: `count_with_image_node` at line 43, defined as
  `fn count_with_image_node<M: Component>(app: &mut App) -> usize`.
- Surviving sibling helper (DO NOT touch): `count_child_of_with` at
  line 48, defined as
  `fn count_child_of_with<M: Component>(app: &mut App) -> usize`.

### Cargo resource policy (binding for all Sprint 16 Cargo invocations)

Per `production/qa/qa-plan-sprint-15.md` §"Cargo Resource Policy on
Windows/MSVC" binding precedent (preserved across PROMPT 815 / 833 / 844 / 851 / 872 /
884 / 889 / 902 / 906 / 907 / 912 / 917 / 918 / 930 / 938 / 941 / 951 /
955 / 959 / 961 / 970 / 973 / 975 / 982 / 983), every `cargo` invocation
on Windows / MSVC MUST set:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

The Sprint 16 `/qa-plan` prompt is expected to restate this policy
verbatim; this story file MUST NOT amend the policy.

---

## Parallelization and Dependencies

| Sibling story | Parallel-safe? |
|---|---|
| **Story 009 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`** (Sprint 16 Should Have headline) | **YES**, file-disjoint: story 009 owns `client/src/ui/` design-token primitives + at least one of hand / shop / auction consumer-surface migrations; this story owns one helper in `tests/integration/presentation/hand_ui_asset_wiring_test.rs`. No shared file. |
| **`S15-OPS-APPCOMPAT-MANIFEST-001`** (Sprint 16 Nice to Have row 1) | **YES**, file-disjoint: appcompat manifest owns `Cargo.toml` test-binary configuration + (likely) `build.rs` or equivalent + the `spawn_range_live_update_contract-*` binary's test directory. This story owns one helper in `hand_ui_asset_wiring_test.rs`. No shared file. |
| **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** (Sprint 16 Must Have human-operator-blocked carry) | **YES**, doc / manual-evidence row vs single-helper test cleanup. No code-file overlap. |

### Dependencies

- **Prerequisite**: None. The dead-code warning has existed on
  `origin/main` since at least PROMPT 983 (Sprint 14 close-out smoke
  rerun) and has been preserved unchanged through Sprint 15 close-out.
  No upstream Sprint 16 row needs to land before this one.
- **Unblocks**: A future Sprint 17+ workspace `#![deny(dead_code)]` or
  `cargo clippy` policy row (NOT scheduled by this story; named here
  only as the conceptual downstream).

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 16 activation HEAD on a fresh
   worktree (suggested branch slug:
   `work/s16-workspace-dead-code-warning`).
2. Read this story file end-to-end before any code change.
3. Read `tests/integration/presentation/hand_ui_asset_wiring_test.rs`
   in full to verify the current state of `count_with_image_node` at
   line 43 and `count_child_of_with` at line 48.
4. Run `grep -rn count_with_image_node tests/ client/ server/ shared/`
   to re-confirm zero call sites at Sprint 16 activation HEAD.
5. Apply the chosen option:
   - **Option A (default)**: delete the `count_with_image_node` helper
     block (lines 43-46 inclusive on `origin/main@8bec9dc`; exact
     range to be re-verified at activation HEAD) and any orphan blank
     line introduced by the deletion. No other diff.
   - **Option B (producer sign-off required)**: delete the helper
     block AND add one new test function that calls
     `count_with_image_node` against an existing PAW-002-f marker
     with documented rationale in the test doc-comment.
6. Set the Cargo resource policy env vars per AC5 / §"Cargo resource
   policy" before any `cargo` invocation.
7. Run `cargo check --workspace --all-targets` and confirm zero
   occurrences of the target dead-code warning line in stderr / stdout
   (AC3).
8. Run `cargo test -p client --test hand_ui_asset_wiring_test
   --no-fail-fast` and confirm PASS with the unchanged green-test
   count (AC4).
9. Push `work/s16-workspace-dead-code-warning` (worker branch only,
   never `main`).
10. Stop. Closure paperwork (`/story-done`, integration `/no-ff`
    merge) is a later prompt's scope.

The worker MUST NOT:

- Modify any file under `client/`, `server/`, `shared/`.
- Modify any file under `tests/` other than
  `tests/integration/presentation/hand_ui_asset_wiring_test.rs`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`,
  `Trunk.toml`.
- Modify any file under `production/sprint-status.yaml`,
  `production/sprints/`, `production/stage.txt`,
  `production/session-state/`, `production/qa/`.
- Modify any file under `docs/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace --tests --no-fail-fast`
  invocation (forbidden for normal `/dev-story` workers per Sprint 15
  QA Policy §"Test Scope Per Prompt Type"; targeted `cargo check` +
  the named per-bin `cargo test` is the policy-correct shape).
- Run `trunk` or any CI command.
- Push to `main`.
- Make any release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, or final-art completion
  claim.

---

`016: S15-TD-WORKSPACE-DEAD-CODE-WARNING-001: DRAFT`
