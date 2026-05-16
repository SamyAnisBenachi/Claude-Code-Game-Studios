# Smoke Check Report: Sprint 14 (Polish stage) -- Rerun after UI drift repair

**Date**: 2026-05-16
**Sprint**: Sprint 14 (Polish stage -- `active` per PROMPT 897 activation)
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-14.md`
**Prompt**: PROMPT 983 -- Sprint 14 Smoke Check Rerun After UI Drift Repair
**HEAD at smoke entry**: `f94f4893cae3690372c5a12f81145de42bb4d94e` (`integrate(s14): merge work/s14-smoke-repair-ui-drift (PROMPT 982)`)
**HEAD == origin/main**: yes (`origin/main` was at the same SHA after `git fetch origin`)
**Smoke worktree**: `D:/tmp/ccgs-prompt-983-smoke` (fresh detached worktree off `origin/main`)
**Worker branch (for evidence push)**: `qa/sprint-14-smoke-rerun-983`
**Smoke environment**: Windows 11 / D: drive. 828 GB free at smoke entry and at smoke exit (no disk-pressure invocation needed).
**Scope**: Sprint 14 smoke rerun after PROMPT 978/979/982 UI drift repair.

---

## Verdict: PASS-WITH-WARNINGS

The full workspace builds, formatting is clean, the cargo aggregate emits 213 binaries / 1350 passed / 0 failed / 0 ignored / 0 measured / 0 filtered, and both PROMPT 978/979 targeted UI drift tests pass at their post-repair shape:

- `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test -- --nocapture` -- **8/8 PASS** (parity with PROMPT 982 integration totals).
- `cargo test -p client --test ui_clean_pass_z_layers_test -- --nocapture` -- **6/6 PASS** (parity with PROMPT 982 integration totals).

The single warning is **environmental, not a code regression**, identical to PROMPT 815 / PROMPT 982 classification:

- `cargo test --workspace --tests --no-fail-fast` could not spawn the test binary `spawn_range_live_update_contract-72565efffef603cf.exe` due to a Windows Application Compatibility installer-detection heuristic that intercepts executables whose filename contains the substring `update` and demands UAC elevation (Windows OS error 740 -- "The requested operation requires elevation."). This is a Windows AppCompat shim layer issue; the binary itself compiles and is well-formed.
- When the same binary is renamed (`srluc_appcompat_renamed.exe`, dropping the `update` substring) and executed directly, all 5 tests inside pass consistently: 5 consecutive runs all returned `ok. 5 passed; 0 failed; 0 ignored`. See [Windows AppCompat Workaround](#windows-appcompat-workaround).
- Net functional totals (cargo aggregate plus direct-run of the renamed binary): **1355 passed / 0 failed / 0 ignored** across 214 effective binaries.

Per `/smoke-check` verdict rules: **PASS** if the suite ran cleanly; **PASS-WITH-WARNINGS** when results are at parity with the baseline but a known, documented, owner-identifiable environmental issue prevents one binary from launching under cargo test. This rerun matches the latter condition. The underlying tests pass; the cargo runner cannot spawn one binary on this Windows host due to AppCompat.

This report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` OPEN, unchanged)
- no two-client GAME_OVER closure
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no Polish->Release gate-check retry (PROMPT 761 `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`)
- no Sprint 14 close-out

PROMPT 983 did **not** run `/gate-check`, `/team-qa`, `/release-check`, `/story-done`, `/dev-story`, `/qa-plan`, or `/story-readiness`. PROMPT 983 did **not** issue any QA sign-off, did **not** close Sprint 14, did **not** advance stage from Polish, did **not** flip any sprint-status row, did **not** edit any story file, did **not** edit `production/stage.txt`, did **not** edit any Sprint 14 sprint plan or QA plan file.

---

## Preflight

| Step | Result |
|---|---|
| `git fetch origin` | OK (no output) |
| `git rev-parse HEAD` (smoke worktree) | `f94f4893cae3690372c5a12f81145de42bb4d94e` |
| `git rev-parse origin/main` | `f94f4893cae3690372c5a12f81145de42bb4d94e` (matches HEAD) |
| `git status --short` (smoke worktree) | clean -- no modifications, no untracked files |
| D: free space at smoke entry | **828 GB free** (well above 40 GB stop-threshold; no cleanup needed) |
| `CARGO_TARGET_DIR` | `D:\_DEV\cargo-target\ccgs-msvc` |
| `CARGO_PROFILE_DEV_DEBUG` | `0` |
| `CARGO_PROFILE_TEST_DEBUG` | `0` |
| `CARGO_INCREMENTAL` | `0` |
| `RUSTFLAGS` | `-C debuginfo=0 -C link-arg=/DEBUG:NONE` |
| Root-checkout dirt preserved untouched | yes -- `M .claude/settings.json`, `M AGENTS.md`, `M CODEX.md`, paperwork-only sprint-state files, `?? D\357\200\272tmp...` stranded files, `?? production/session-state/autonomous-monitor-task.md`, `?? tools/gcs-orchestrator/docs/ARCHITECTURE.md` were not staged, unstaged, modified, deleted, or relied on by this smoke run |

---

## Cargo Policy (verified before workspace Cargo)

The smoke run set and verified the Cargo policy environment **before** invoking any workspace-wide Cargo target, per PROMPT 983 instruction:

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

No PDB/debuginfo emitted; all linker outputs link with `/DEBUG:NONE`. No incremental compilation. D: free space remained at 828 GB at smoke exit; no Cargo target cleanup invoked.

---

## Commands and Results

### `cargo fmt --all -- --check`

**Result**: PASS (exit 0, no output). No formatting drift on the workspace.

### `cargo check --workspace --all-targets`

**Result**: PASS -- `Finished \`dev\` profile [optimized] target(s) in 20.06s`. Zero compilation errors. One pre-existing dead-code warning (not introduced by PROMPT 978/979/982 repair, not a regression):

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

This warning predates PROMPT 983 and PROMPT 982. PROMPT 983 scope forbids `tests/` edits; the warning is **not** addressed by this smoke run. Could be filed as a future Sprint 14 candidate Nice-to-Have if it persists.

### `cargo test --workspace --tests --no-fail-fast`

**Result**: PASS-WITH-WARNINGS (1 binary failed to spawn under cargo; functional total verified via the [Windows AppCompat Workaround](#windows-appcompat-workaround)).

Cargo-aggregate totals across the 213 binaries that emitted a `test result:` line:

| Metric | Count |
|---|---|
| passed | **1350** |
| failed | **0** |
| ignored | **0** |
| measured | 0 |
| filtered out | 0 |
| binaries with emitted result | 213 |

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

**Functional total** (cargo aggregate + 5 from direct-run of UAC-blocked binary, see workaround): **1355 passed / 0 failed / 0 ignored** across 214 effective binaries.

### Targeted PROMPT 978/979 reruns

#### `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test -- --nocapture`

**Result**: PASS -- 8/8.

```
running 8 tests
test auction_border_tier_maps_gdd_price_ranges ... ok
test bid_labels_render_total_commitment_with_secondary_increment ... ok
test local_free_gold_saturates_reserved_gold_without_underflow ... ok
test shop_auction_ui_plugin_registers_in_minimal_client_app_without_panic ... ok
test shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes ... ok
test shop_auction_ui_roots_are_stable_during_session_updates ... ok
test shop_auction_ui_phase_visibility_reads_current_phase_resource ... ok
test shop_auction_ui_plugin_is_registered_fifth_through_presentation_plugin ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

The `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` test verifies the post-PROMPT 979 prepool formula (`AUCTION_FREE_GOLD_COUNTER_COUNT * 3 + 1` term reconciled with the 7 `ShopAuctionUiEntity` entities from the auction free-gold counter group). Parity with PROMPT 982 integration result.

#### `cargo test -p client --test ui_clean_pass_z_layers_test -- --nocapture`

**Result**: PASS -- 6/6.

```
running 6 tests
test ac6_layer_constants_survive_pairwise_distinctness_under_arbitrary_permutation ... ok
test module_exports_minimum_gap_constant_for_future_intermediate_layers ... ok
test ac8_module_doc_names_adr_021_and_presentation_plugin_load_order ... ok
test ac7_production_migration_sites_reference_design_tokens ... ok
test ac5_grep_guard_no_inline_global_z_index_literals_outside_design_tokens ... ok
test ac6_paint_order_matches_named_layers_under_out_of_order_spawn ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

The `ac7_production_migration_sites_reference_design_tokens` test verifies the post-PROMPT 979 grep guard expecting both `z_layers::UI_OVERLAY` and `z_layers::MODAL` references in `ui/lobby.rs` (alongside existing `z_layers::UI_BASE`). Parity with PROMPT 982 integration result.

### `git diff --check`

Output: empty (exit 0). No whitespace defects.

### `git diff --cached --check`

Output: empty (exit 0). No staged changes at smoke entry.

---

## Windows AppCompat Workaround

The Windows installer-detection heuristic is documented behavior of the Application Compatibility shim layer. It triggers on filenames containing `setup`, `install`, `update`, or `patch` regardless of file content, unless the executable has an embedded application manifest with `<requestedExecutionLevel level="asInvoker"/>`. Cargo-emitted rustc test binaries do not currently embed such a manifest.

PROMPT 983 followed the PROMPT 979 / PROMPT 982 precedent and used the per-run rename workaround:

```bash
cp D:/_DEV/cargo-target/ccgs-msvc/debug/deps/spawn_range_live_update_contract-72565efffef603cf.exe \
   D:/tmp/srluc_appcompat_renamed.exe

# 5 consecutive direct invocations
D:/tmp/srluc_appcompat_renamed.exe
D:/tmp/srluc_appcompat_renamed.exe
D:/tmp/srluc_appcompat_renamed.exe
D:/tmp/srluc_appcompat_renamed.exe
D:/tmp/srluc_appcompat_renamed.exe
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

**5/5 runs PASS -- each 5/5 tests** -- identical AppCompat classification and identical pass record to PROMPT 815 (Sprint 12 smoke), PROMPT 790 (Sprint 11 smoke), PROMPT 979 (UI drift repair worker), and PROMPT 982 (UI drift repair integration). Classification: **environment/tool-only failure** -- not a code regression, not a product-code regression, not authored by PROMPT 978/979/982.

Two workaround paths exist (forward-looking, **not** invoked by PROMPT 983 scope):

1. **Per-run workaround (used here)**: copy the binary to a filename that does not contain `update`, then execute directly. Isolates the AppCompat trigger entirely.
2. **Code-side workaround (out of smoke scope; would require `tests/` or build-system changes)**: rename the test source file or add a Windows manifest via `embed-resource` / `winres` to the `shared` test target. **Not authorised** by PROMPT 983 (scope forbids `tests/` modifications). Could be filed as a Sprint 14 candidate Nice-to-Have if the host environment continues to flag this binary.

---

## Failures and Blockers

- **0 test failures** in the cargo aggregate (1350 / 0 / 0 / 0 / 0 across 213 binaries).
- **0 build errors** (`cargo check --workspace --all-targets` PASS).
- **0 formatting drift** (`cargo fmt --all -- --check` PASS).
- **0 `git diff --check` whitespace defects** (working and cached both clean).
- **1 binary spawn failure** documented in [Windows AppCompat Workaround](#windows-appcompat-workaround). Functional pass total verified at parity with baseline. **Not a code regression**; not blocking.
- **0 disk-space blockers** at any point in the smoke run (828 GB free at entry and exit).
- **0 PROMPT 978 regression** -- both targeted tests PASS at the post-repair shape introduced by PROMPT 979 / PROMPT 982.
- **0 tooling blocks** otherwise. No `cargo fmt --check` drift. No deprecation panic. No linker / PDB pressure.

---

## Ignored tests (documented)

**Total `#[ignore]`d in workspace**: **0** (consistent with Sprint 12 / Sprint 13 baseline retired in PROMPT 814).

---

## Sprint 14 disposition (preserved by PROMPT 983)

- **Sprint 14**: `active` (Polish-stage; activated by PROMPT 897). 8 of 9 Must Have rows `done` per session-state banner; 1 Must Have row (`S11-UX-HUD-TOP-STRIP-LAYOUT`) outstanding; 0 of 4 Should Have rows done; 0 of 4 Nice to Have rows done. PROMPT 983 did **not** edit `production/sprints/sprint-14.md`, did **not** edit `production/sprint-status.yaml`, did **not** edit `production/stage.txt`, did **not** modify `.claude/settings.json`, did **not** touch any story file, did **not** touch `production/qa/qa-plan-sprint-14.md`, did **not** touch session-state files.
- **Stage**: `Polish`. PROMPT 761 Polish->Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry attempted by PROMPT 983.
- **Sprint 13 close-out**: `closed-with-conditions` per PROMPT 894 -- preserved unchanged.
- **Sprint 12 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 11 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 10 close-out**: `closed-with-conditions` -- preserved unchanged.
- **Sprint 14 close-out**: **NOT** performed by this prompt. Smoke is one of several gates required before close-out; Team-QA and Sprint-14 close-out remain pending in separate prompts.

---

## Files changed by PROMPT 983

- `production/qa/smoke-sprint-14-2026-05-16-rerun.md` (this file -- NEW)
- `reports/PROMPT-983-Sprint-14-Smoke-Check-Rerun-After-UI-Drift-Repair.md` (mandatory final report; tracked separately under `reports/`)

Explicitly **not** touched (forbidden by PROMPT 983 scope):

- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-14.md`
- `production/qa/qa-plan-sprint-14.md`
- any prior sprint plan, QA plan, smoke evidence, team-QA, or close-out artifact
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- any Sprint 14 story file
- `production/session-state/active.md` and `production/session-state/codex-orchestrator-state.md` (PROMPT 983 allowed-edits list permits a session-state banner *only if existing checkpoint convention requires it*; PROMPT 982 immediate precedent did not modify session-state; smoke checkpoint rerun therefore declines to touch session-state)
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`
- WASM bundle build outputs, deployment configs

Root-checkout dirt at `D:\_DEV\Work\Claude-Code-Game-Studios` (`M .claude/settings.json`, `M AGENTS.md`, `M CODEX.md`, paperwork-only sprint-state modifications, stranded `D\357\200\272tmp...` files, `?? production/session-state/autonomous-monitor-task.md`, `?? tools/gcs-orchestrator/docs/ARCHITECTURE.md`) was **not** touched, staged, unstaged, deleted, or relied on by this smoke check. The smoke worktree was a freshly created, clean checkout of `origin/main@f94f489`.

---

## Verification commands (for re-run)

```bash
git worktree add D:/tmp/ccgs-prompt-983-smoke f94f4893cae3690372c5a12f81145de42bb4d94e
cd D:/tmp/ccgs-prompt-983-smoke

# Cargo policy
export CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'

git fetch origin
git rev-parse HEAD                                       # expect f94f4893cae3690372c5a12f81145de42bb4d94e
git rev-parse origin/main                                # expect f94f4893cae3690372c5a12f81145de42bb4d94e
df -h /d                                                 # expect >40 GB free

cargo fmt --all -- --check                               # expect exit 0, no output
cargo check --workspace --all-targets                    # expect Finished dev profile, 1 pre-existing dead_code warning
cargo test --workspace --tests --no-fail-fast            # expect 213 binaries / 1350 / 0 / 0 + 1 spawn-blocked binary (see workaround)
cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test -- --nocapture   # expect 8/8
cargo test -p client --test ui_clean_pass_z_layers_test -- --nocapture                     # expect 6/6
git diff --check                                         # expect empty
git diff --cached --check                                # expect empty

# Workaround for AppCompat-blocked binary on Windows hosts:
cp D:/_DEV/cargo-target/ccgs-msvc/debug/deps/spawn_range_live_update_contract-*.exe D:/tmp/srluc_appcompat_renamed.exe
D:/tmp/srluc_appcompat_renamed.exe                       # expect 5 passed; 0 failed; 0 ignored (repeat 5x for parity with PROMPT 982)
```

---

## Cross-references

- Sprint 14 plan: `production/sprints/sprint-14.md`
- Sprint 14 QA plan: `production/qa/qa-plan-sprint-14.md`
- Sprint 14 activation: PROMPT 897 / 2026-05-15
- Sprint 14 UI drift smoke (previous, FAIL): PROMPT 978 (no committed report file)
- Sprint 14 UI drift worker repair: `D:/_DEV/Work/Claude-Code-Game-Studios/reports/PROMPT-979-*.md` (PROMPT 979 worker on `work/s14-smoke-repair-ui-drift`)
- Sprint 14 UI drift integration: `reports/PROMPT-982-s14-smoke-repair-ui-drift-integration.md` (merged `origin/main@c385682` -> `origin/main@f94f489`)
- Prior smoke (Sprint 12, PASS-WITH-WARNINGS reference): `production/qa/smoke-sprint-12-2026-05-14.md` (PROMPT 815, 1135 / 0 / 0)
- Polish->Release gate FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint status: `production/sprint-status.yaml`
- Stage: `production/stage.txt` (`Polish`)

---

## Next recommended step

**PROMPT 984 (or successor)** -- after this smoke evidence lands on `origin/main`, the next eligible action is either:

1. **Resume Sprint 14 Must-Have completion** -- the outstanding Must Have row `S11-UX-HUD-TOP-STRIP-LAYOUT` is the gating row for Sprint 14 Should-Have / Nice-to-Have ladder activation; subject to `/story-readiness` rerun against this smoke HEAD.
2. **`/story-readiness` rerun for `production/epics/shop-auction-ui/story-018-auction-lead-loss-state.md`** -- per the PROMPT 967 disposition in session-state, story 018 is producer-decision-RESOLVED and awaits a readiness rerun against `origin/main@f94f489`.

PROMPT 983 explicitly defers all of `/team-qa`, `/gate-check`, `/release-check`, `/story-done`, `/dev-story`, `/qa-plan`, and Sprint-14 close-out to later prompts. This smoke rerun does **not** advance stage from Polish, does **not** retry Polish->Release, does **not** close Sprint 14, does **not** claim release-readiness, broad accessibility, playtest validation, two-client GAME_OVER closure, or final-art completion.

---

**End of report -- PROMPT 983 / Sprint 14 smoke rerun verdict: PASS-WITH-WARNINGS (environment/tool-only AppCompat warning, identical classification to PROMPT 815 / 982)**
