# Smoke Check Report: Sprint 15 (Polish stage) -- UI Clean-Pass Closeout Sprint

**Date**: 2026-05-17
**Sprint**: Sprint 15 (Polish stage -- `active` per PROMPT 997 activation; merged to `origin/main` by PROMPT 1001)
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-15.md` (PROMPT 1002)
**Prompt**: PROMPT 1012 -- Sprint 15 Smoke Check
**HEAD at smoke entry**: `f3e635d657589ce41b7b1e9667207e0830bfedb0` (`story-done(s15): close S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP (PROMPT 1010)`)
**HEAD == origin/main**: yes (`origin/main` was at the same SHA after `git fetch origin`)
**Smoke worktree**: `D:/Tmp/ccgs-prompt-1012-smoke` (fresh worktree off `origin/main`)
**Worker branch (for evidence push)**: `qa/sprint-15-smoke-check-1012`
**Smoke environment**: Windows 11 / D: drive. 826 GB free at smoke entry and smoke exit (no disk-pressure invocation, no Cargo target cleanup invoked).
**Scope**: Sprint 15 end-of-sprint smoke (Wave 7 per QA plan §"QA Sequencing"). Tests latest `origin/main`. Does NOT close Sprint 15. Does NOT close the human-operator HUD timer eyeball row.

---

## Verdict: PASS-WITH-WARNINGS

The full workspace builds, formatting is clean, the cargo aggregate emits **216 binaries / 1384 passed / 0 failed / 0 ignored / 0 measured / 0 filtered out**, and both Sprint 15 new integration test bins pass at parity with their `/dev-story` worker + integration results:

- `cargo test -p client --test hand_ui_drag_state_visuals_test` (story 020) -- **11/11 PASS** (parity with PROMPT 1003 worker + PROMPT 1008 integration totals).
- `cargo test -p client --test ui_clean_pass_interaction_state_primitives_test` (story 008) -- **8/8 PASS** (parity with PROMPT 1005 worker + PROMPT 1007 integration totals).
- Sprint 14 prior baselines re-verified at parity: `shop_auction_ui_plugin_scaffold_formulas_test` 8/8 PASS, `ui_clean_pass_z_layers_test` 6/6 PASS.

The single warning is **environmental, not a code regression**, identical to PROMPT 815 / PROMPT 979 / PROMPT 982 / PROMPT 983 classification:

- `cargo test --workspace --tests --no-fail-fast` could not spawn the test binary `spawn_range_live_update_contract-72565efffef603cf.exe` due to a Windows Application Compatibility installer-detection heuristic that intercepts executables whose filename contains the substring `update` and demands UAC elevation (Windows OS error 740 -- "The requested operation requires elevation."). This is a Windows AppCompat shim layer issue; the binary itself compiles and is well-formed.
- When the same binary is renamed (`srluc_appcompat_renamed_1012.exe`, dropping the `update` substring) and executed directly, all 5 tests inside pass consistently: 5 consecutive runs all returned `ok. 5 passed; 0 failed; 0 ignored`. See [Windows AppCompat Workaround](#windows-appcompat-workaround).
- Net functional totals (cargo aggregate plus direct-run of the renamed binary): **1389 passed / 0 failed / 0 ignored** across **217 effective binaries**.

Per `/smoke-check` verdict rules: **PASS** if the suite ran cleanly; **PASS-WITH-WARNINGS** when results are at parity with the baseline but a known, documented, owner-identifiable environmental issue prevents one binary from launching under cargo test, AND there is a known external pending integration not landed on `origin/main` at smoke time. This run matches both conditions:

1. The AppCompat-blocked binary is identical to the Sprint 11 / 12 / 14 precedents (PROMPT 790 / 815 / 982 / 983); classification unchanged.
2. **PROMPT 998 placement-timer audio crash repair (`origin/integrate/placement-timer-audio-crash-repair-998@c508d9d`) is NOT reachable from `origin/main` at smoke time** (verified via `git merge-base --is-ancestor c508d9d origin/main` -> not an ancestor). The repair branch is READY on a worker integration branch but the harness push-policy gates direct `main` merges. **This smoke does NOT claim PROMPT 996 / PROMPT 998 audio crash repair is landed.** See [External Pending Integration -- PROMPT 998](#external-pending-integration----prompt-998).

This report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged, accepted-risk preserved)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged, accepted-risk preserved)
- no full playable-client manual QA
- no two-client GAME_OVER closure (`S8-QA-001-W1` OPEN, unchanged)
- no final-art / asset-production completion (`PAW-TD-*-a` accepted-risk preserved across PAW-002..PAW-006)
- no Polish->Release gate-check retry (PROMPT 761 `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`)
- no Sprint 15 close-out
- no closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (still `ready`, human-operator-blocked Sprint 13 -> 14 -> 15 carry; no LLM `/story-done` authorised; human-operator screenshot capture remains pending)
- no closure of `TQ-S12-C7`
- no stage advance from `Polish`
- no Sprint 12 story 019 underlying drag-runtime bug claim of fix (cannot-reproduce preserved; TQ-S12-C2 no-third-retest binding preserved)
- no Sprint 14 / 13 / 12 / 11 / 10 row reopen
- no closure of PROMPT 996 / PROMPT 998 audio crash repair (external pending integration; not on `origin/main` at smoke time)

PROMPT 1012 did **not** run `/gate-check`, `/team-qa`, `/release-check`, `/story-done`, `/dev-story`, `/qa-plan`, or `/story-readiness`. PROMPT 1012 did **not** issue any QA sign-off, did **not** close Sprint 15, did **not** advance stage from Polish, did **not** flip any sprint-status row, did **not** edit any story file, did **not** edit `production/stage.txt`, did **not** edit any Sprint 15 sprint plan or QA plan file, did **not** modify `client/` / `server/` / `shared/` / `tests/`, did **not** modify session-state files (PROMPT 983 immediate-precedent smoke also declined; local smoke pattern does not require a banner).

---

## Preflight

| Step | Result |
|---|---|
| `git fetch origin` | OK (no output) |
| `git rev-parse HEAD` (smoke worktree) | `f3e635d657589ce41b7b1e9667207e0830bfedb0` |
| `git rev-parse origin/main` | `f3e635d657589ce41b7b1e9667207e0830bfedb0` (matches HEAD) |
| `git status --short` (smoke worktree) | clean -- no modifications, no untracked files |
| D: free space at smoke entry | **826 GB free** (well above 40 GB stop-threshold; no cleanup needed) |
| `CARGO_TARGET_DIR` | `D:\_DEV\cargo-target\ccgs-msvc` |
| `CARGO_PROFILE_DEV_DEBUG` | `0` |
| `CARGO_PROFILE_TEST_DEBUG` | `0` |
| `CARGO_INCREMENTAL` | `0` |
| `RUSTFLAGS` | `-C debuginfo=0 -C link-arg=/DEBUG:NONE` |
| Cargo resource policy applied before any cargo command | **yes** |
| Stale target cleanup invoked | **no** (D: free >> 40 GB threshold throughout) |
| `git merge-base --is-ancestor c508d9d origin/main` (PROMPT 998 reachability) | exit 1 -- **NOT an ancestor**. PROMPT 998 audio crash repair NOT landed on `origin/main` at smoke time. |
| Root-checkout dirt preserved untouched | yes -- the smoke worktree is at `D:/Tmp/ccgs-prompt-1012-smoke`, separate from the root checkout `D:/_DEV/Work/Claude-Code-Game-Studios`. Root-checkout modifications not staged, unstaged, or relied on by this smoke run. |

---

## Cargo Policy (verified before workspace Cargo)

The smoke run set and verified the Cargo policy environment **before** invoking any workspace-wide Cargo target, per PROMPT 1012 instruction (binding policy from PROMPT 815 / 833 / 844 / ... / 982 / 983 precedents and reaffirmed by PROMPT 1002 §"Cargo Resource Policy on Windows/MSVC"):

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

No PDB/debuginfo emitted; all linker outputs link with `/DEBUG:NONE`. No incremental compilation. D: free space remained at 826 GB at smoke exit; no Cargo target cleanup invoked.

---

## Commands and Results

### `cargo fmt --all -- --check`

**Result**: PASS (exit 0, no output). No formatting drift on the workspace.

### `cargo check --workspace --all-targets`

**Result**: PASS -- `Finished \`dev\` profile [optimized] target(s) in 57.76s`. Zero compilation errors. One pre-existing dead-code warning (carried unchanged from PROMPT 983 baseline; not introduced by any Sprint 15 row; not a regression):

```
warning: function `count_with_image_node` is never used
  --> client\..\tests\integration\presentation\hand_ui_asset_wiring_test.rs:43:4
   |
43 | fn count_with_image_node<M: Component>(app: &mut App) -> usize {
   |    ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
warning: `client` (test "hand_ui_asset_wiring_test") generated 1 warning
```

This warning predates Sprint 15 (preserved from Sprint 14 close-out PROMPT 983 baseline). PROMPT 1012 scope forbids `tests/` edits; the warning is **not** addressed by this smoke run.

### `cargo test --workspace --tests --no-fail-fast`

**Result**: PASS-WITH-WARNINGS (1 binary failed to spawn under cargo; functional total verified via the [Windows AppCompat Workaround](#windows-appcompat-workaround)).

Cargo-aggregate totals across the 216 binaries that emitted a `test result:` line:

| Metric | Count |
|---|---|
| passed | **1384** |
| failed | **0** |
| ignored | **0** |
| measured | 0 |
| filtered out | 0 |
| binaries with emitted result | 216 |

The 1 binary that failed to spawn:

- Binary: `D:\_DEV\cargo-target\ccgs-msvc\debug\deps\spawn_range_live_update_contract-72565efffef603cf.exe`
- Source: `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
- Cargo error chain:

```
error: test failed, to rerun pass `-p shared --test spawn_range_live_update_contract`

Caused by:
  could not execute process `D:\_DEV\cargo-target\ccgs-msvc\debug\deps\spawn_range_live_update_contract-72565efffef603cf.exe` (never executed)

Caused by:
  The requested operation requires elevation. (os error 740)

error: 1 target failed:
    `-p shared --test spawn_range_live_update_contract`
```

- Root cause: Windows Application Compatibility installer-detection heuristic intercepts spawn of any executable whose filename contains the substrings `setup`, `install`, `update`, or `patch` and demands UAC elevation unless an embedded application manifest declares `level="asInvoker"`. Cargo-emitted rustc test binaries do not embed such a manifest.
- The test source file (`spawn_range_live_update_contract_test.rs`) is named after the live-update protocol contract it validates; that string ends up in the cargo bin name verbatim.

**Functional total** (cargo aggregate + 5 from direct-run of UAC-blocked binary, see workaround): **1389 passed / 0 failed / 0 ignored** across **217 effective binaries**.

### Sprint 15 row test bins (parity rerun via the workspace test invocation above)

#### Story 020 -- `cargo test -p client --test hand_ui_drag_state_visuals_test`

**Result**: PASS -- 11/11.

```
running 11 tests
... (11 tests, all ok) ...

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Parity with PROMPT 1003 worker + PROMPT 1008 integration. AC9 BLOCKING integration test passes at workspace level.

#### Story 008 -- `cargo test -p client --test ui_clean_pass_interaction_state_primitives_test`

**Result**: PASS -- 8/8.

```
running 8 tests
... (8 tests, all ok) ...

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Parity with PROMPT 1005 worker + PROMPT 1007 integration. AC8 BLOCKING integration test passes at workspace level.

### Sprint 14 prior-baseline reruns (parity verification)

#### `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test`

**Result**: PASS -- 8/8 (parity with PROMPT 982 / PROMPT 983 baseline).

```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

#### `cargo test -p client --test ui_clean_pass_z_layers_test`

**Result**: PASS -- 6/6 (parity with PROMPT 982 / PROMPT 983 baseline).

```
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### `git diff --check`

Output: empty (exit 0). No whitespace defects.

### `git diff --cached --check`

Output: empty (exit 0). No staged changes at smoke entry.

---

## Windows AppCompat Workaround

The Windows installer-detection heuristic is documented behavior of the Application Compatibility shim layer. It triggers on filenames containing `setup`, `install`, `update`, or `patch` regardless of file content, unless the executable has an embedded application manifest with `<requestedExecutionLevel level="asInvoker"/>`. Cargo-emitted rustc test binaries do not currently embed such a manifest.

PROMPT 1012 followed the PROMPT 815 / 979 / 982 / 983 precedent and used the per-run rename workaround:

```bash
cp D:/_DEV/cargo-target/ccgs-msvc/debug/deps/spawn_range_live_update_contract-72565efffef603cf.exe \
   D:/tmp/srluc_appcompat_renamed_1012.exe

# 5 consecutive direct invocations
D:/tmp/srluc_appcompat_renamed_1012.exe
D:/tmp/srluc_appcompat_renamed_1012.exe
D:/tmp/srluc_appcompat_renamed_1012.exe
D:/tmp/srluc_appcompat_renamed_1012.exe
D:/tmp/srluc_appcompat_renamed_1012.exe
```

All 5 runs returned identical results:

```
running 5 tests
test test_s2c_resolution_event_remains_registered_on_reliable_channel ... ok
test test_spawn_range_changed_is_ordered_after_fake_objective_destroyed_in_same_batch ... ok
test test_player_snapshot_spawn_range_cells_remains_public_recovery_field ... ok
test test_spawn_range_is_not_registered_as_standalone_protocol_message ... ok
test test_spawn_range_changed_schema_round_trips_through_resolution_batch ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**5/5 runs PASS -- each 5/5 tests** -- identical AppCompat classification and identical pass record to PROMPT 815 (Sprint 12 smoke), PROMPT 790 (Sprint 11 smoke), PROMPT 979 (UI drift repair worker), PROMPT 982 (UI drift repair integration), and PROMPT 983 (Sprint 14 smoke rerun). Classification: **environment/tool-only failure** -- not a code regression, not a product-code regression, not authored by any Sprint 15 row.

Two workaround paths exist (forward-looking, **not** invoked by PROMPT 1012 scope):

1. **Per-run workaround (used here)**: copy the binary to a filename that does not contain `update`, then execute directly. Isolates the AppCompat trigger entirely.
2. **Code-side workaround (out of smoke scope; would require `tests/` or build-system changes)**: rename the test source file or add a Windows manifest via `embed-resource` / `winres` to the `shared` test target. **Not authorised** by PROMPT 1012 (scope forbids `tests/` modifications). Could be filed as a Sprint 16+ candidate Nice-to-Have if the host environment continues to flag this binary.

---

## External Pending Integration -- PROMPT 998

**PROMPT 998 placement-timer audio crash repair integration** (`origin/integrate/placement-timer-audio-crash-repair-998@c508d9d`) is READY on a worker integration branch but is **NOT** reachable from `origin/main` at PROMPT 1012 smoke time.

Verification at smoke entry:

```
$ git rev-parse origin/integrate/placement-timer-audio-crash-repair-998
c508d9d... (parent: 299696d "fix(client): disarm unsupported timer urgency audio crash (PROMPT 996)")

$ git merge-base --is-ancestor c508d9d origin/main
$ echo $?
1                              # NOT an ancestor

$ git log --oneline origin/main..origin/integrate/placement-timer-audio-crash-repair-998
c508d9d integrate(s15): merge PROMPT 996 placement timer audio crash repair (PROMPT 998 redo on activated main)
299696d fix(client): disarm unsupported timer urgency audio crash (PROMPT 996)
```

This smoke **explicitly does NOT claim the audio crash repair is landed**. It is recorded as a **warning / carry condition** for Wave 8 (Team-QA) and Sprint 15 close-out planning. The two-commit chain (PROMPT 996 worker `299696d` + PROMPT 998 integration `c508d9d`) sits on the integration branch pending orchestrator-side `main` merge.

The smoke binary count and functional test totals reported here therefore reflect `origin/main@f3e635d` only -- they do NOT include any test changes that might land alongside the audio crash repair integration. If a follow-on prompt merges `c508d9d` into `origin/main`, a fresh smoke is required to confirm no regression in the integrated state.

---

## Failures and Blockers

- **0 test failures** in the cargo aggregate (1384 / 0 / 0 / 0 / 0 across 216 binaries).
- **0 build errors** (`cargo check --workspace --all-targets` PASS).
- **0 formatting drift** (`cargo fmt --all -- --check` PASS).
- **0 `git diff --check` whitespace defects** (working and cached both clean).
- **1 binary spawn failure** documented in [Windows AppCompat Workaround](#windows-appcompat-workaround). Functional pass total verified at parity with baseline. **Not a code regression**; not blocking; identical classification to PROMPT 790 / 815 / 979 / 982 / 983.
- **0 disk-space blockers** at any point in the smoke run (826 GB free at entry and exit).
- **0 Sprint 15 regression** -- both new integration test bins (story 020 + story 008) PASS at workspace-level parity with their `/dev-story` worker + integration results.
- **0 Sprint 14 prior-baseline regression** -- both PROMPT 978/979/982/983 targeted tests PASS at parity.
- **0 tooling blocks** otherwise. No `cargo fmt --check` drift. No deprecation panic. No linker / PDB pressure.

### Carried warnings / non-blocking observations

- **PROMPT 998 audio crash repair (`c508d9d`) is NOT on `origin/main`**. Documented as warning / carry condition; smoke does not claim the repair is landed; future smoke required after merge.
- **`S11-HUD-TIMER-EYEBALL-VISUAL-001` remains `status: ready`** (human-operator-blocked Sprint 13 -> 14 -> 15 carry; promoted Should -> Must in Sprint 15). The evidence-slot reservation runbook authored by PROMPT 1011 (worker branch `prompt-1011-hud-timer-human-capture-prep@b4c1b79`) is **not** merged to `origin/main` at smoke time. The row's closure is gated on a real human-operator capture session -- **no LLM `/story-done` is authorised**, and this smoke does NOT claim the row closed.

---

## Sprint 15 status and row summary (preserved by PROMPT 1012)

Sprint 15 is `active` (Polish-stage; activated by PROMPT 997, integration to `origin/main` by PROMPT 1001). Smoke entry HEAD = `origin/main@f3e635d` = PROMPT 1010 row-flip /story-done tip.

| # | Row ID | Priority | Status | Closing prompt | Notes |
|---|--------|----------|--------|----------------|-------|
| 1 | `S11-HUD-TIMER-EYEBALL-VISUAL-001` | must-have | **ready** | (none yet -- human-operator-blocked) | Sprint 13 -> 14 -> 15 carry; promoted Should -> Must; no LLM /story-done authorised; PROMPT 1011 authored evidence-slot reservation but did NOT close the row |
| 2 | `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` | must-have | done (2026-05-17) | PROMPT 1010 | Paperwork-only row-status flip; closure rationale cites PROMPT 891 / `S13-CONN-LOST-UX-001` evidence |
| 3 | `S12-UX-HAND-DRAG-STATE-VISUALS-001` | should-have | done (2026-05-17) | PROMPT 1009 | hand-ui drag-state visuals (story 020); 11-test BLOCKING integration test PASS |
| 4 | `S11-UX-BOARD-RENDERING-SPEC` | should-have | done (2026-05-17) | PROMPT 1009 | Doc-only spec at `docs/ux/board-rendering-spec.md`; ratification sign-off captured |
| 5 | `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` | nice-to-have | done (2026-05-17) | PROMPT 1009 | UI interaction state primitives (story 008); 8-test BLOCKING integration test PASS |

**Sprint 15 progress after PROMPT 1010**: 1 of 2 Must Have done + 2 of 2 Should Have done + 1 of 1 Nice to Have done = **4 of 5 rows closed**. The single outstanding row is `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked). PROMPT 1012 smoke does NOT advance this count.

### Sprint 15 close-out status

- **Sprint 15**: `active` (preserved by PROMPT 1012). NOT closed by this smoke.
- **Stage**: `Polish` (unchanged). PROMPT 761 Polish->Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry attempted by PROMPT 1012.
- **Sprint 14 close-out**: `closed-with-conditions` per PROMPT 987 -- preserved unchanged.
- **Sprint 13 close-out**: `closed-with-conditions` per PROMPT 894 -- preserved unchanged.
- **Sprint 12 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 11 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 10 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 15 close-out**: **NOT** performed by this prompt. Smoke is one of several gates required before close-out; Team-QA (Wave 8) and Sprint 15 close-out (Wave 9) remain pending in separate prompts per QA plan sequencing.

---

## Ignored tests (documented)

**Total `#[ignore]`d in workspace**: **0** (consistent with Sprint 12 / 13 / 14 baseline retired in PROMPT 814; preserved unchanged by all Sprint 15 rows).

---

## Preserved non-claims (BLOCKING -- verbatim from QA plan §"Required Non-Claims")

PROMPT 1012 / Sprint 15 smoke **explicitly does NOT claim** any of the following. Each is BLOCKING -- if any becomes true within Sprint 15 scope, it requires a SEPARATE story file with explicit disposition; it cannot be silently folded into this smoke or surfaced as a side-effect of running tests.

- **Stage `Polish`** preserved (`production/stage.txt` unchanged).
- **PROMPT 761 `Polish->Release` gate-check `FAIL`** preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; **no retry** in Sprint 15 scope, **no retry attempted** by PROMPT 1012.
- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains **OPEN**. Sprint 15 candidate stories did NOT touch the two-client GAME_OVER surface. The Sprint 13 story 017 AC12 forbid-auto-closure carries through Sprint 14 and Sprint 15 unchanged.
- **`QA-COND-0005`** Standard-tier accessibility remains **accepted-risk / friend-game scope only**. Sprint 15 UI clean-pass closeout rows are friend-game visual polish only; no WCAG contrast ratios, no >=44 Px hit-targets, no full keyboard navigation, no screen-reader support, no colour-blind modes, no text-scaling advanced by Sprint 15.
- **`QA-COND-0006`** playtest / fun-hypothesis validation remains **accepted-risk / deferred**. Sprint 15 visual review is qa-lead / qa-tester / ui-programmer / ux-designer eyeball only.
- **`PAW-TD-*-a`** placeholder-art accept-risk across PAW-002..PAW-006 -- preserved. Sprint 15 layout / composition / primitive work does NOT advance placeholder-art resolution.
- **`TQ-S12-C1..C7`** preserved verbatim. **TQ-S12-C2 binding**: no third same-scope retest of Sprint 12 `hand-ui/story-019-drag-runtime-retest-tighter-capture.md` is authorised. **TQ-S12-C7** explicitly NOT closed by any Sprint 15 row.
- **PROMPT 683-era runtime divergence question** -- preserved as folded into Sprint 12 story 019 `cannot-reproduce` closure.
- **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed fixed by Sprint 15. Story 020 (hand drag-state visuals) is layout / visual state work over already-extant client-side drag ephemeral state per ADR-012; it does NOT reproduce or fix the underlying drag-runtime bug.
- **No public release readiness**, **no release-candidate (RC) readiness**, **no full game completion**, **no broad / Standard-tier accessibility completion**, **no playtest validation**, **no full playable-client manual QA**, **no two-client GAME_OVER closure**, **no final-art / asset-production completion**, **no Polish->Release retry**, **no stage advance**, **no underlying drag-runtime bug fix**.
- **No closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`** by this smoke. The row remains `status: ready`, human-operator-blocked. Closure requires a human-operator screenshot capture session FIRST (per PROMPT 1011 runbook), then a paperwork `/story-done` prompt that references the evidence README. If the session cannot be scheduled in Sprint 15, the row carries Sprint 15 -> Sprint 16.
- **No closure of PROMPT 996 / PROMPT 998 audio crash repair disposition** -- external pending integration; not on `origin/main` at smoke time (`c508d9d` not reachable from `origin/main`).
- **No Sprint 14 row reopen** -- all 16 closed Sprint 14 `/story-done` closures (PROMPT 903 / 908 / 909 / 919 / 921 / 922 / 931 / 939 / 942 / 953 / 956 / 960 / 962 / 972 / 974 / 976) preserved unchanged on `origin/main`.
- **No Sprint 15 close-out** -- Sprint 15 remains `active`. Team-QA (Wave 8) and Sprint 15 close-out (Wave 9) remain pending in separate prompts per QA plan sequencing.

---

## Files changed by PROMPT 1012

- `production/qa/smoke-sprint-15-2026-05-17.md` (this file -- NEW)
- `reports/PROMPT-1012-Sprint-15-Smoke-Check.md` (mandatory final report; tracked separately under `reports/` which is gitignored)

Explicitly **not** touched (forbidden by PROMPT 1012 scope):

- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-15.md`
- `production/qa/qa-plan-sprint-15.md`
- any prior sprint plan, QA plan, smoke evidence, team-QA, or close-out artifact
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- any Sprint 15 story file (story 014 / 020 / 013 / 008)
- `production/session-state/active.md` and `production/session-state/codex-orchestrator-state.md` (PROMPT 1012 allowed-edits list permits a session-state banner *only if local smoke pattern requires it*; PROMPT 983 immediate-precedent smoke did NOT modify session-state; the local smoke pattern therefore declines to touch session-state)
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`
- WASM bundle build outputs, deployment configs
- the PROMPT 998 audio crash repair branch tip (`c508d9d`) -- no merge attempted; recorded only as a carry warning

---

## Verification commands (for re-run)

```bash
git worktree add D:/Tmp/ccgs-prompt-1012-smoke -b qa/sprint-15-smoke-check-1012 origin/main
cd D:/Tmp/ccgs-prompt-1012-smoke

# Cargo policy
export CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'

git fetch origin
git rev-parse HEAD                                       # expect f3e635d657589ce41b7b1e9667207e0830bfedb0
git rev-parse origin/main                                # expect f3e635d657589ce41b7b1e9667207e0830bfedb0
df -h /d                                                 # expect >40 GB free

# PROMPT 998 reachability
git merge-base --is-ancestor c508d9d origin/main         # expect exit 1 (NOT an ancestor)

cargo fmt --all -- --check                               # expect exit 0, no output
cargo check --workspace --all-targets                    # expect Finished dev profile, 1 pre-existing dead_code warning
cargo test --workspace --tests --no-fail-fast            # expect 216 binaries / 1384 / 0 / 0 + 1 spawn-blocked binary
git diff --check                                         # expect empty
git diff --cached --check                                # expect empty

# Workaround for AppCompat-blocked binary on Windows hosts:
cp D:/_DEV/cargo-target/ccgs-msvc/debug/deps/spawn_range_live_update_contract-*.exe D:/tmp/srluc_appcompat_renamed_1012.exe
D:/tmp/srluc_appcompat_renamed_1012.exe                  # expect 5 passed; 0 failed; 0 ignored (repeat 5x for parity)
```

---

## Cross-references

- Sprint 15 plan: `production/sprints/sprint-15.md` (PROMPT 988 draft + PROMPT 997 ACTIVATED banner; integration to `origin/main` by PROMPT 1001 merge `7a5965e`)
- Sprint 15 QA plan: `production/qa/qa-plan-sprint-15.md` (PROMPT 1002)
- Sprint 15 activation: PROMPT 997 (`ef68816` activation worktree -> PROMPT 1001 `main` merge `7a5965e`)
- Sprint 15 story-authoring: PROMPT 991 / 992 / 993 (workers) + PROMPT 995 integration (`8294f9a`)
- Sprint 15 `/dev-story` workers: PROMPT 1003 / 1004 / 1005 (workers) + PROMPT 1006 / 1007 / 1008 integrations
- Sprint 15 integrated /story-done batch: PROMPT 1009 (`3b6acec`)
- Sprint 15 paperwork-only row-flip: PROMPT 1010 (`f3e635d` = smoke entry tip)
- Sprint 15 HUD-timer human capture prep: PROMPT 1011 (`prompt-1011-hud-timer-human-capture-prep@b4c1b79`; NOT yet on `origin/main`)
- PROMPT 998 audio crash repair (external pending): `origin/integrate/placement-timer-audio-crash-repair-998@c508d9d` (NOT reachable from `origin/main`)
- Prior smoke (Sprint 14 rerun, PASS-WITH-WARNINGS reference): `production/qa/smoke-sprint-14-2026-05-16-rerun.md` (PROMPT 983, 1350 / 0 / 0)
- Polish->Release gate FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint status: `production/sprint-status.yaml`
- Stage: `production/stage.txt` (`Polish`)

---

## Next recommended step

**PROMPT N+1 -- `/team-qa sprint`** (Wave 8 per QA plan sequencing) -- after this smoke evidence lands on `origin/main`, the next eligible action is Sprint 15 Team-QA synthesis. The team-QA prompt synthesises per-row evidence (test pass logs from this smoke, story 020 / 008 / 013 ratification minutes, row-flip diff from PROMPT 1010, and the PROMPT 1011 evidence-slot reservation -- noting that the HUD timer eyeball row remains open pending human-operator capture) and authors `production/qa/team-qa-sprint-15-<date>.md`. Disposition expected: `APPROVED-WITH-CONDITIONS` (same accept-risk conditions carry forward; `S11-HUD-TIMER-EYEBALL-VISUAL-001` remains open).

**Parallel-safe / out-of-band**: The human-operator HUD timer eyeball capture session for `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Wave 5; per the PROMPT 1011 runbook at `production/qa/evidence/sprint-15-hud-timer-visual-check/README.md` on `origin/prompt-1011-hud-timer-human-capture-prep@b4c1b79`) remains pending. It is parallel-safe with Team-QA but the integration of PROMPT 1011 worker branch onto `origin/main` is a separate orchestrator step (Sprint 14 PROMPT 1011-precedent integration).

**Also pending (NOT advanced by this smoke)**: PROMPT 998 placement-timer audio crash repair integration to `origin/main`. The repair is READY on `origin/integrate/placement-timer-audio-crash-repair-998@c508d9d` but not yet merged. Recommend a separate paperwork-only integration prompt to land `c508d9d` on `origin/main`, followed by a confirmatory mini-smoke (targeted reruns + AppCompat workaround) before Sprint 15 close-out.

PROMPT 1012 explicitly defers all of `/team-qa`, `/gate-check`, `/release-check`, `/story-done`, `/dev-story`, `/qa-plan`, `/story-readiness`, the human-operator HUD timer capture, the PROMPT 998 audio repair integration, and Sprint-15 close-out to later prompts. This smoke does **not** advance stage from Polish, does **not** retry Polish->Release, does **not** close Sprint 15, does **not** claim release-readiness, broad accessibility, playtest validation, two-client GAME_OVER closure, or final-art completion.

---

**End of report -- PROMPT 1012 / Sprint 15 smoke verdict: PASS-WITH-WARNINGS (environment/tool-only AppCompat warning + external pending PROMPT 998 audio repair not yet on `origin/main`; identical AppCompat classification to PROMPT 815 / 979 / 982 / 983)**
