# PROMPT 1467 -- Connection-Lost Auction Overlay Observability Repair

Status: BLOCKED on branch push approval; implementation and local commit are complete.

## Implementation Summary

- Replaced the connection-lost overlay's boolean-only state with an explicit projection:
  - `cause`: `none`, `local_transport_disconnected`, or `opponent_disconnected`
  - `disconnected_player_id`
  - `grace_remaining_ms`
  - `input_blocking`
- Local Lightyear transport loss now renders blocking copy:
  - headline: `Connection Interrupted`
  - body explains the local client is reconnecting, game input is blocked, and auction/shop state remains visible behind the notice.
- Opponent disconnect state now projects from the existing `OpponentConnectionView` without adding protocol semantics:
  - headline: `Opponent Reconnecting`
  - body includes the disconnected player, reconnect grace countdown in seconds, and states local input is not blocked.
- Non-blocking opponent notices are laid out as a compact top-right status panel with no scrim and `Pickable { should_block_lower: false, is_hoverable: false }`.
- Blocking local transport-loss notices retain the centered panel, dim backdrop, and blocking `Pickable` mode.
- Added structured tracing fields for overlay cause, disconnected player, grace remaining, and input-blocking mode on state transitions.
- Added additive QA snapshot diagnostics under `extras.connection_lost`:
  - `cause`
  - `local_is_disconnected`
  - `disconnected_player_id`
  - `grace_remaining_ms`
  - `blocking_input`

## Files Changed

- `client/src/presentation/connection_lost_overlay.rs`
- `client/src/presentation/qa_snapshot.rs`
- `tests/integration/playable_client/connection_lost_overlay_test.rs`
- `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs`
- `reports/PROMPT-1467-connection-lost-auction-overlay-observability-repair.md`

## Scope Notes

- Did not touch `client/src/ui/shop_auction/**`.
- Did not touch `client/src/ui/hud/**`.
- Did not touch `client/src/ui/hand/**`.
- Did not touch `client/src/presentation/board_rendering.rs`.
- Did not touch server/shared protocol.
- QA snapshot changes are strictly additive and preserve existing PROMPT 1458/1460 fields.
- Existing reconnect behavior is preserved; no new protocol semantics were invented.

## Tests

Cargo/MSVC policy was applied before every Cargo command:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Commands run:

```powershell
cargo fmt
cargo test -p client --test connection_lost_overlay_test --test qa_snapshot_overlay_test prompt1467
cargo test -p client --test connection_lost_overlay_test --test qa_snapshot_overlay_test
```

Results:

- `connection_lost_overlay_test`: 19 passed, 0 failed.
- `qa_snapshot_overlay_test`: 29 passed, 0 failed.
- Rust warnings: existing deprecated universal UI marker warnings from HUD/hand/shop snapshot count paths; no new test failures.
- Worker worktree note: `cargo fmt` hit Windows path-length error in `D:\_DEV\claude-code-game-studios-worktrees\PROMPT-1467`, so the four changed Rust files were formatted with `rustfmt` directly. The worker-targeted tests passed from a fresh `D:\_DEV\cargo-target\ccgs-msvc-1467` target directory after the shared target initially served stale artifacts.

## Branch / Commit / Push

- Worker branch: `work/connection-lost-auction-overlay-observability-repair-1467`
- Worker worktree: `D:\_DEV\claude-code-game-studios-worktrees\PROMPT-1467`
- Local commit: `683ebecef45ecea4053fec90f74c05f88e74dc58`
- Push command attempted: `git push origin work/connection-lost-auction-overlay-observability-repair-1467`
- Push blocker: approval reviewer rejected external network export to unverified remote `origin`; no workaround attempted.

## Remaining Live QA Needed

- Fresh two-client QA should confirm the opponent-disconnect notice appears as a non-blocking top-right status over active auction/shop state.
- Fresh local transport-drop QA should confirm the blocking local reconnect notice still blocks input while keeping the auction/shop state readable underneath.
- Snapshot review should confirm `extras.connection_lost` correlates with `ui_counts.connection_lost_overlay_visible`, `extras.opponent_connection`, server/client disconnect logs, and auction state.

1467: CONNECTION-LOST-AUCTION-OVERLAY-OBSERVABILITY-REPAIR: BLOCKED
