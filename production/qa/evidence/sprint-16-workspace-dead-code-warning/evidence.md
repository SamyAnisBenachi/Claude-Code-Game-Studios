# Sprint 16 -- Workspace Dead-Code Warning Cleanup Evidence

**Story**: `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`
**Story file**: `production/epics/ui-clean-pass/story-016-workspace-dead-code-warning.md`
**Worker prompt**: PROMPT 1069
**Worker branch**: `work/s16-workspace-dead-code-warning`
**Base origin/main**: `7a78b257cdcd8b76f439f7264b31648b3ae1261c`
  (PROMPT 1066 `qa(s16): author Sprint 16 QA plan`)
**Date**: 2026-05-17

## Option Chosen: Option A -- Delete

### Rationale

The dead helper `count_with_image_node<M: Component>` at
`tests/integration/presentation/hand_ui_asset_wiring_test.rs:43` was
unused on `origin/main@7a78b25`. Workspace grep
(`tests/integration/presentation/hand_ui_asset_wiring_test.rs` is the
only call surface for either helper; both helpers had zero matches in
any other file under `tests/`) confirmed no call site existed for
`count_with_image_node` anywhere in the workspace.

The sibling helper `count_child_of_with<M: Component>` (still defined
at the same section header) is strictly more correct for the PAW-002-f
chrome-presence assertion shape: it filters by `(With<M>, With<ImageNode>)`
and counts the `&ChildOf` of those matches, which simultaneously
asserts (a) the chrome entity carries `ImageNode`, (b) the chrome
entity carries the marker `M`, and (c) the chrome entity has a parent.
Every `test_fan_slot_chrome_*_image_node_present` test in the file
calls `count_child_of_with`. No coverage gap remained that
`count_with_image_node` was meant to close, so Option A (deletion) was
the correct choice per the story default. Option B (wire helper into a
new assertion) was not used; no new test function was added.

### Net Change

Single hunk in
`tests/integration/presentation/hand_ui_asset_wiring_test.rs`:
deletion of the five-line `count_with_image_node` function definition
and its trailing blank line. The `// ── Helpers ──` section header is
preserved (the surviving `count_child_of_with` helper still occupies
that section). No production code is touched.

## Verification Results

### Disk Preflight

D: free space before `cargo check --workspace --all-targets`:
**~860 GB free** (860,441,366,528 bytes), well above the 40 GB
threshold. No disk cleanup performed.

### Cargo Resource Policy Attestation

Applied for every `cargo` invocation in this run (per Sprint 14 /
Sprint 15 binding precedent, PROMPT 815+):

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

### `cargo check --workspace --all-targets`

PASS. Exit 0.

Final two lines:
```
    Checking two-client-runtime v0.1.0 (...\tools\two-client-runtime)
    Finished `dev` profile [optimized] target(s) in 21.00s
```

Symbol filter (`findstr count_with_image_node` on the full stderr/stdout
output): **0 matches**. The `warning: function \`count_with_image_node\`
is never used` line is gone.

### `cargo test -p client --test hand_ui_asset_wiring_test --no-fail-fast`

PASS. 10 tests run; 10 passed; 0 failed; 0 ignored.

```
test test_fan_slot_chrome_stat_badge_atk_image_node_present ... ok
test test_fan_slot_chrome_stat_badge_mp_image_node_present ... ok
test test_fan_slot_chrome_stat_badge_hp_image_node_present ... ok
test test_fan_slot_chrome_card_frame_image_node_present ... ok
test test_fan_slot_chrome_card_frame_handle_non_default ... ok
test test_fan_slot_chrome_rarity_icon_image_node_present ... ok
test test_fan_slot_chrome_stat_badge_ar_image_node_present ... ok
test test_fan_slot_chrome_type_icon_image_node_present ... ok
test test_fan_slot_chrome_stat_badge_handles_non_default ... ok
test test_fan_slot_chrome_children_parent_is_fan_slot ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Baseline test count on `origin/main@7a78b25`: 10 `#[test]` items in the
same file (verified via `git show origin/main:.../hand_ui_asset_wiring_test.rs | grep -c '^#\[test\]'`).
Post-change count: 10. **No `#[ignore]` introduced. No test removed
or renamed.** No new test function added (Option A path).

### Git Diff Hygiene

- `git diff` (unstaged): single hunk, deletion only, in
  `tests/integration/presentation/hand_ui_asset_wiring_test.rs`.
- `git diff --check`: clean (no whitespace errors).

## Non-Claims

This evidence document does NOT claim any of the following:

- Sprint 16 closure
- Polish->Release stage advance or retry
- Release-candidate or full-game readiness
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (HUD timer human visual)
- Closure of `QA-COND-0005` (Standard-tier accessibility) or
  `QA-COND-0006` (playtest / fun hypothesis)
- Closure of `S8-QA-001-W1` (two-client GAME_OVER)
- `PAW-TD-*-a` placeholder-art advancement
- Smoke check, Team-QA, gate-check, or release-check sign-off
- Story-done closure for story 016 (left for a later `/story-done`
  prompt; story status not flipped here)
- Edits to `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-sprint-16.md`, or any other Sprint 16
  activation / close-out artifact
- Any change to `client/`, `server/`, or `shared/` production code
- Any change to `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or
  `Trunk.toml`
