# Smoke Check Report: Sprint 12 (Polish / friend-game scope)

**Date**: 2026-05-14
**Sprint**: Sprint 12 (Polish stage — `active` per PROMPT 798 activation, all 5 Must Have rows `done` per PROMPT 814)
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-12.md` (authored by PROMPT 799, 2026-05-14)
**Prompt**: PROMPT 815 — Sprint 12 Smoke Check
**HEAD at smoke entry**: `7e55952c4837ea6aad361d2a946b94e768cfee9b` (`qa(s12): /story-done batch for 5 Sprint 12 Must Have rows (PROMPT 814)`)
**HEAD == origin/main**: yes (`origin/main` was at the same SHA after `git fetch origin`)
**Smoke worktree**: `D:\_DEV\claude-code-game-studios-worktrees\sprint-12-smoke-check`
**Branch**: `qa/sprint-12-smoke-check`
**Smoke environment**: Windows 11 / D: drive. After disk-pressure cleanup (see §Disk Pressure Policy Invocation), 225 GB free at start of full test re-run.
**Scope**: Sprint 12 Polish / friend-game smoke check only.

---

## Verdict: PASS-WITH-WARNINGS

All Sprint 12 Must Have rows are `done` on `origin/main@7e55952` (PROMPT 814 /story-done batch). The full workspace builds, formatting is clean, and the test suite functionally passes at parity with the PROMPT 813 baseline (1135 passed / 0 failed / 0 ignored).

The single warning is **environmental, not a code regression**:

- `cargo test --workspace --tests --no-fail-fast` could not spawn one test binary, `spawn_range_live_update_contract-*.exe`, due to a Windows AppCompat installer-detection heuristic that requires UAC elevation for executables whose names contain the substring `update`. This is a Windows shell layer; the binary itself compiles and is well-formed.
- When the binary is renamed (`spawn_range_live_upd_contract.exe`, dropping the `update` substring) and executed directly, all five tests inside pass: `5 passed; 0 failed; 0 ignored`. See §Windows AppCompat Workaround.
- Net functional totals (cargo aggregate plus direct-run of the renamed binary): **1135 passed / 0 failed / 0 ignored** — exact parity with PROMPT 813's reported baseline.

Per `/smoke-check` verdict rules: **PASS** if the suite ran cleanly; **PASS-WITH-WARNINGS** when results are at parity with the baseline but a known, documented, owner-identifiable environmental issue prevents one binary from launching under cargo test. This run matches the latter condition. The underlying tests pass; the cargo runner cannot spawn one binary on this Windows host due to AppCompat.

This report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no two-client GAME_OVER closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no Sprint 12 close-out
- no Polish→Release gate-check retry (PROMPT 761 `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`)

PROMPT 815 did **not** run `/gate-check`, `/team-qa`, `/release-check`, `/story-done`, `/dev-story`, or `/qa-plan`. PROMPT 815 did **not** issue any QA sign-off, did **not** close Sprint 12, did **not** advance stage from Polish.

---

## Preflight

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git rev-parse HEAD` (smoke worktree) | `7e55952c4837ea6aad361d2a946b94e768cfee9b` |
| `git rev-parse origin/main` | `7e55952c4837ea6aad361d2a946b94e768cfee9b` (matches HEAD) |
| `git status --short` (smoke worktree) | clean — no modifications, no untracked files |
| D: free space at smoke entry | 82 GB free (1.3 TB total — 94% used); hit disk-pressure mid-run, see §Disk Pressure Policy Invocation |
| D: free space after cleanup | 225 GB free (above 5 GB minimum and above the PROMPT 790 reference of ~222 GB) |
| Root-checkout dirt preserved untouched | yes — `M .claude/settings.json`, `A production/session-state/autonomous-monitor-task.md`, `?? Dtmpworkspace-test-output.txt` were not staged, unstaged, modified, deleted, or relied on |

---

## Commands and Results

### `cargo fmt --check`

```
cargo fmt --check
```

**Result**: PASS (exit 0, no output — no formatting drift on the workspace).

### `cargo check --workspace`

```
cargo check --workspace
```

**Result**: PASS — `Finished \`dev\` profile [optimized + debuginfo] target(s) in 1m 22s`. Zero compilation errors. No new warnings beyond those already present on the integrated tip.

### `cargo test --workspace --tests --no-fail-fast`

```
cargo test --workspace --tests --no-fail-fast
```

**Result**: PASS-WITH-WARNINGS (1 binary failed to spawn; functional total 1135/0/0 verified via §Windows AppCompat Workaround).

Cargo-aggregate totals across the 189 binaries that emitted a `test result:` line:

| Metric | Count |
|---|---|
| passed | **1130** |
| failed | **0** |
| ignored | **0** |
| measured | 0 |
| filtered out | 0 |
| binaries with emitted result | 189 |

The 1 binary that failed to spawn:

- Binary: `target\debug\deps\spawn_range_live_update_contract-<hash>.exe`
- Source: `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
- Cargo error: `could not execute process ... The requested operation requires elevation. (os error 740)`
- Root cause: Windows Application Compatibility installer-detection heuristic intercepts spawn of any executable whose filename contains the substrings `setup`, `install`, `update`, or `patch` and demands UAC elevation unless an embedded application manifest declares `level="asInvoker"`. Cargo-emitted rustc test binaries do not embed such a manifest.
- The test source file (`spawn_range_live_update_contract_test.rs`) is named after the live-update protocol contract it validates; that string ends up in the cargo bin name verbatim.

Directly executing the same binary after copying it to a non-`update` name (`D:\tmp\spawn_range_live_upd_contract.exe`) ran cleanly:

```
running 5 tests
test test_spawn_range_changed_is_ordered_after_fake_objective_destroyed_in_same_batch ... ok
test test_s2c_resolution_event_remains_registered_on_reliable_channel ... ok
test test_spawn_range_is_not_registered_as_standalone_protocol_message ... ok
test test_spawn_range_changed_schema_round_trips_through_resolution_batch ... ok
test test_player_snapshot_spawn_range_cells_remains_public_recovery_field ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Functional total** (cargo aggregate + 5 from direct-run of UAC-blocked binary): **1135 passed / 0 failed / 0 ignored** — parity with PROMPT 813 baseline at `a3c624e` (also 1135 / 0 / 0).

### `git diff --check`

Output: empty (exit 0). No whitespace defects.

### `git diff --cached --check`

Output: empty (exit 0). No staged changes at smoke entry.

---

## Windows AppCompat Workaround

The Windows installer-detection heuristic is documented behavior of the Application Compatibility shim layer. It triggers on filenames containing `setup`, `install`, `update`, or `patch` regardless of file content, unless the executable has an embedded application manifest with `<requestedExecutionLevel level="asInvoker"/>`. Cargo-emitted rustc test binaries do not currently embed such a manifest.

Two workaround paths exist for future smoke runs on this Windows host:

1. **Per-run workaround (used here)**: copy the binary to a filename that does not contain `update`, then execute directly. This isolates the AppCompat trigger entirely. Used in this smoke check to verify the test contents pass.

2. **Code-side workaround (out of smoke scope; would require Cargo manifest changes)**: rename the test source file or add a Windows manifest via `embed-resource` / `winres` to the test crate. This is **not** authorised by PROMPT 815 (scope forbids `tests/` modifications). Could be filed as a Sprint 13 candidate Nice-to-Have if the host environment continues to flag this binary.

The discrepancy with PROMPT 813's clean cargo aggregate (1135/0/0) is consistent with Windows AppCompat caching: identical code can produce different spawn outcomes between runs depending on whether the AppCompat shim has cached an installer-detection decision for the previous binary hash. The current run's binary hash differs from PROMPT 813's; the cache miss caused the shim to re-evaluate and intercept on the filename heuristic.

This is **not** a code regression. The five tests inside the blocked binary pass when execution is allowed.

---

## Disk Pressure Policy Invocation

PROMPT 815 was authorised by its instructions to clean Cargo build artifacts if cargo checks/tests hit disk-full / linker / PDB pressure, with a preferred target of the smoke worktree's own `target/` or a clearly unused previous worker `target/`, and a requirement to print the resolved absolute path and verify it is under `target/` and not repo root, source, `.git`, production, reports, or evidence.

This authorisation was invoked twice during this smoke run:

**Cleanup 1** — at start of `cargo test`, the first `cargo test` invocation failed with `error: failed to write query cache to ... There is not enough space on the disk. (os error 112)`. `df -h /d` showed 0 GB free.

- Resolved path: `D:\_DEV\claude-code-game-studios-worktrees\class-d-diag\target`
- Verified under: `target/` of an old, no-longer-active worker worktree on branch `work/fixture-clientstate-init-state-001` (oldest worktree, May 12 timestamp; predates current Sprint 11 / Sprint 12 work). Not repo root, source, `.git`, production, reports, or evidence.
- Size freed: 25 GB
- Deletion command: `Remove-Item -LiteralPath 'D:\_DEV\claude-code-game-studios-worktrees\class-d-diag\target' -Recurse -Force`
- Post-cleanup: 25 GB free.

**Cleanup 2** — disk usage scan revealed `integration-s11-fixture-d-residuals/target` at 201 GB. That worktree (`integrate/s11-fixture-d-residuals`) carried the PROMPT 813 integration commit `a3c624e`, already on `origin/main` since that integration. Its `target/` was clearly unused build cache from a closed integration step.

- Resolved path: `D:\_DEV\claude-code-game-studios-worktrees\integration-s11-fixture-d-residuals\target`
- Verified under: `target/` of an integration worktree whose work is already on `origin/main`. Not repo root, source, `.git`, production, reports, or evidence.
- Size freed: ~200 GB
- Deletion command: `Remove-Item -LiteralPath 'D:\_DEV\claude-code-game-studios-worktrees\integration-s11-fixture-d-residuals\target' -Recurse -Force`
- Post-cleanup: 225 GB free.

Both cleanups followed the prompt's `Remove-Item -LiteralPath ... -Recurse -Force` form. Neither touched any source, production, evidence, reports, `.git`, or repo-root content. After the second cleanup, `cargo test --workspace --tests --no-fail-fast` was re-run successfully (the run that produced the 189-binary aggregate above; the spawn-failure for `spawn_range_live_update_contract` is unrelated to disk and is documented in §Windows AppCompat Workaround).

This cleanup is a Sprint 12 candidate Nice-to-Have signal: the existing nice-to-have rows `S11-TD-CARGO-DISK-USAGE-001` and `S11-TD-CARGO-PDB-LIMIT-001` (both `blocked` pending story files) are the right destination for any persistent fix.

---

## Ignored tests (documented)

**Total `#[ignore]`d in workspace**: **0**.

This is the Sprint 12 retirement of the Sprint 11 close-out Cluster B baseline of 5 retained D-5 `#[ignore]` tests. All five were retired by Sprint 12 Must Have stories:

- **B1 + B5 umbrella**: `S11-TD-FIXTURE-D-RESIDUALS-001` retired `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` (B1) and `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` (B5). Worker PROMPT 812 / integration PROMPT 813 / story-done PROMPT 814.
- **B2**: `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001` retired `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` via Path B (relocated the snapshot.phase HUD bridge assertion into a dedicated HUD-side test). Worker PROMPT 806 / integration PROMPT 809 / story-done PROMPT 814.
- **B3**: `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` retired `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` via fallback path (test rewritten to simulate `S2CJoinAck` + `session_id` lobby session gate; asserts `C2SSelectClass` + `C2SConfirmClass` traverse reliable channel). Worker PROMPT 801 / integration PROMPT 805 / story-done PROMPT 814.
- **B4**: `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001` retired `test_cooccupancy_index_two_panics_with_offending_index` via Path B (test rewritten to assert clamp invariant; `#[should_panic]` removed deliberately, not silently). Worker PROMPT 800 / integration PROMPT 805 / story-done PROMPT 814.

Cross-references for retirement evidence:

- `production/qa/evidence/sprint-12-fixture-d-residuals-evidence.md`
- `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md`
- `production/qa/evidence/sprint-12-lobby-confirm-class-intent-chain-evidence.md`
- `production/qa/evidence/sprint-12-cooccupancy-panic-guard-evidence.md`
- `production/qa/evidence/sprint-11-ignored-d5-triage.md` (PROMPT 787 / 789 baseline)

Also note story 019 (`S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`) is `done` with `closed-with-conditions / cannot-reproduce` after the second time-box exhaustion in PROMPT 807. The underlying drag-runtime divergence is **not** claimed fixed; it is escalated to PROMPT 804 Sprint 13 candidate runtime-hardening stories. Evidence at `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md`.

---

## Failures and Blockers

- **0 test failures** in the cargo aggregate.
- **0 build errors**.
- **0 `git diff --check` whitespace defects** (working and cached both clean).
- **1 binary spawn failure** documented in §Windows AppCompat Workaround. Functional pass total verified at parity with baseline. **Not a code regression**; not blocking.
- **0 disk-space blockers** at end of smoke run (225 GB free after Cleanup 1 + Cleanup 2).
- **0 tooling blocks** otherwise. No `cargo fmt --check` drift. No deprecation panic.

---

## Sprint 12 disposition (preserved)

- **Sprint 12**: `active` (Polish-stage; activated by PROMPT 798). All 5 Must Have rows `done` per PROMPT 814 batch. Should-Have rows (4) and Nice-to-Have rows (5) remain `blocked` pending story files + `/story-readiness`. PROMPT 815 did **not** edit `production/sprints/sprint-12.md`, did **not** edit `production/sprint-status.yaml`, did **not** edit `production/stage.txt`, did **not** modify `.claude/settings.json`.
- **Stage**: `Polish`. PROMPT 761 Polish→Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry attempted by PROMPT 815.
- **Sprint 11**: `closed-with-conditions` per PROMPT 792, recorded under `sprint_11_closeout:` in `production/sprint-status.yaml`. Unchanged.
- **Sprint 10**: `closed-with-conditions` per PROMPT 763, recorded under `sprint_10_closeout:` in `production/sprint-status.yaml`. Unchanged.
- **Sprint 12 close-out**: **NOT** performed by this prompt. Smoke is one of several gates required before close-out; Team-QA and Sprint-12 close-out remain pending in separate prompts.

---

## Files changed by PROMPT 815

- `production/qa/smoke-sprint-12-2026-05-14.md` (this file — NEW)
- `production/session-state/active.md` (banner prepended)
- `production/session-state/codex-orchestrator-state.md` (PROMPT 815 disposition section prepended)
- `reports/PROMPT-815-Sprint-12-Smoke-Check.md` (mandatory final report; not staged or committed)

Explicitly **not** touched (forbidden by PROMPT 815 scope):

- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-12.md`
- `production/sprints/sprint-11.md`
- `production/qa/qa-plan-sprint-12.md`
- `production/qa/qa-plan-sprint-11.md`
- `production/qa/smoke-sprint-11-2026-05-13.md`
- `production/qa/team-qa-sprint-11-2026-05-13.md`
- `production/qa/evidence/sprint-11-ignored-d5-triage.md` and all other Sprint 11 / Sprint 12 evidence files
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- any Sprint 12 story file (`story-012`/`013`/`014`/`015`/`019`)
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`

Root-checkout dirt at `D:\_DEV\Work\Claude-Code-Game-Studios` (`M .claude/settings.json`, `A production/session-state/autonomous-monitor-task.md`, untracked `Dtmpworkspace-test-output.txt`) was **not** touched, staged, unstaged, deleted, or relied on by this smoke check. The smoke worktree was a freshly created, clean checkout of `origin/main`.

---

## Verification commands (for re-run)

```bash
git worktree add -b qa/sprint-12-smoke-check D:/_DEV/claude-code-game-studios-worktrees/sprint-12-smoke-check origin/main
cd D:/_DEV/claude-code-game-studios-worktrees/sprint-12-smoke-check

git fetch origin
git rev-parse HEAD                                       # expect 7e55952c4837ea6aad361d2a946b94e768cfee9b
git rev-parse origin/main                                # expect 7e55952c4837ea6aad361d2a946b94e768cfee9b
df -h /d                                                 # expect >5 GB free (cleanup other worktree target/ dirs if not)

cargo fmt --check                                        # expect exit 0, no output
cargo check --workspace                                  # expect Finished dev profile
cargo test --workspace --tests --no-fail-fast            # expect 189 binaries / 1130 / 0 / 0 + 1 spawn-blocked binary (see workaround)
git diff --check                                         # expect empty
git diff --cached --check                                # expect empty

# Workaround for AppCompat-blocked binary on Windows hosts:
cp target/debug/deps/spawn_range_live_update_contract-*.exe /tmp/spawn_range_live_upd_contract.exe
/tmp/spawn_range_live_upd_contract.exe                   # expect 5 passed; 0 failed; 0 ignored
```

---

## Cross-references

- Sprint 12 plan: `production/sprints/sprint-12.md`
- Sprint 12 QA plan: `production/qa/qa-plan-sprint-12.md`
- Sprint 12 story files (5 Must Have, all `done`):
  - `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  - `production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md`
  - `production/epics/playable-client/story-013-lobby-confirm-class-intent-chain.md`
  - `production/epics/playable-client/story-014-cooccupancy-panic-guard-decision.md`
  - `production/epics/playable-client/story-015-fixture-d-residuals.md`
- Sprint 11 D-5 triage (baseline for Cluster B retirement): `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- Prior smoke: `production/qa/smoke-sprint-11-2026-05-13.md` (PROMPT 790, PASS-WITH-WARNINGS, 1129/0/5)
- Polish→Release gate FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint status: `production/sprint-status.yaml`
- Stage: `production/stage.txt` (`Polish`)
- PROMPT 813 integration test totals (baseline reference): 1135 / 0 / 0 at `a3c624e`
- PROMPT 814 `/story-done` batch report: `reports/PROMPT-814-S11-Sprint-12-Must-Have-Story-Done-Batch.md`

---

## Next recommended step

PROMPT 816 — **`/team-qa` for Sprint 12** (qa-lead + qa-tester team review against this smoke evidence). Run on a new worktree from latest `origin/main` after this PROMPT 815 smoke evidence lands. PROMPT 816 should:

- consume this smoke verdict (PASS-WITH-WARNINGS — environmental, not code)
- review the four Sprint 12 evidence files and the story-019 cannot-reproduce disposition
- emit a Sprint 12 Team-QA sign-off file under `production/qa/team-qa-sprint-12-YYYY-MM-DD.md`
- **not** advance stage from Polish, **not** retry Polish→Release, **not** close Sprint 12 (close-out is a separate prompt)
- **not** claim release-readiness, broad accessibility, playtest validation, or two-client GAME_OVER closure

After PROMPT 816 Team-QA lands, the Sprint 12 close-out gate becomes the next eligible step (still does not advance stage).

---

**End of report — PROMPT 815 / Sprint 12 smoke verdict: PASS-WITH-WARNINGS**
