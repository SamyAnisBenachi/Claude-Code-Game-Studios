# PROMPT 1471 -- Connection-Lost Overlay Observability Main Refresh

Status: READY_FOR_MAIN_LAND on branch `work/connection-lost-overlay-observability-main-refresh-1471`.

## Implementation Summary

- Created a fresh worker worktree from `origin/main` at `7e308a831505bbc91f906a78955832a0b85eecd5`.
- Cherry-picked source commit `db0754b965bb0801ea876fd02dc5fbe350f35ccc` onto that main refresh branch.
- The cherry-pick applied cleanly; no conflict resolution was needed.
- Rebasing was then required because `origin/main` advanced during the task; the final branch is based on `a1e9e287e248b301506b2b0b20d7bc722e0f0641`.
- Preserved the current `origin/main` HUD readability, placement rejection recovery, grid overlay, placement visibility, and QA snapshot forensic field work.
- Reapplied only the connection-lost overlay observability repair:
  - explicit overlay cause, disconnected player id, grace remaining, and input-blocking state;
  - local transport loss remains a blocking centered overlay;
  - opponent disconnect projects from `OpponentConnectionView` as a non-blocking top-right status;
  - QA snapshot diagnostics add `extras.connection_lost.cause` and `local_is_disconnected` while preserving existing snapshot fields;
  - targeted integration tests cover blocking local loss, non-blocking opponent grace, and snapshot payload distinctions.

## Files Changed

- `client/src/presentation/connection_lost_overlay.rs`
- `client/src/presentation/qa_snapshot.rs`
- `tests/integration/playable_client/connection_lost_overlay_test.rs`
- `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs`
- `reports/PROMPT-1467-connection-lost-auction-overlay-observability-repair.md`
- `reports/PROMPT-1471-connection-lost-overlay-observability-main-refresh.md`

## Scope Notes

- Did not touch `client/src/ui/hud/**`.
- Did not touch `client/src/ui/hand/**`.
- Did not touch `client/src/ui/shop_auction/**`.
- Did not touch board rendering, server, shared, protocol, sprint/session paperwork, or staged files in the root checkout.
- Root checkout was dirty on another worker branch and was not modified.

## Verification

MSVC Cargo policy was applied before the targeted Cargo test command:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Commands run:

```powershell
git diff --check origin/main...HEAD
cargo test -p client --test connection_lost_overlay_test --test qa_snapshot_overlay_test
```

Results:

- `git diff --check origin/main...HEAD`: passed.
- `connection_lost_overlay_test`: 19 passed, 0 failed.
- `qa_snapshot_overlay_test`: 29 passed, 0 failed.
- Cargo emitted existing deprecated universal UI marker warnings from HUD/hand/shop snapshot count paths; no test failures.

## Branch / Commit

- Worker branch: `work/connection-lost-overlay-observability-main-refresh-1471`
- Worker worktree: `D:\_DEV\claude-code-game-studios-worktrees\PROMPT-1471`
- Base: `origin/main` at `a1e9e287e248b301506b2b0b20d7bc722e0f0641`
- Source commit replayed: `db0754b965bb0801ea876fd02dc5fbe350f35ccc`
- Local commits:
  - `2f4ca757` -- rebased PROMPT-1467 observability replay
  - `HEAD` -- PROMPT-1471 refresh report
- Push attempted: `git push origin work/connection-lost-overlay-observability-main-refresh-1471`
- Push blocker: approval reviewer rejected external data transfer to unverified remote `origin`; no workaround attempted.

1471: CONNECTION-LOST-OVERLAY-OBSERVABILITY-MAIN-REFRESH: READY_FOR_MAIN_LAND
