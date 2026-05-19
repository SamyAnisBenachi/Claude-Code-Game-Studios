# PROMPT 1441 - S19-BOT-PROTOCOL-ROOM-FOUNDATIONS-INTEGRATION-REFRESH

Status: READY_FOR_MAIN_LAND
Branch: `work/s19-bot-protocol-room-foundations-1441`
Base main: `499b5e10b19de435967b384c39d6115badbcf033`
Source worker branch: `origin/work/s19-bot-protocol-room-foundations-1430`
Source worker commit: `8860f3fce93872eae80af9950bbb49e82d8829ce`

## Integration Method

Created an isolated integration worktree from freshly fetched `origin/main`,
then applied PROMPT 1430 with a no-commit cherry-pick. The cherry-pick applied
cleanly with no conflicts. The old PROMPT 1430 report was removed from the
refresh changeset because this prompt only permits the PROMPT 1441 report.

The resulting patch was reviewed as an additive protocol/session refresh. No
landed current-main files from prompts 1432/1433/1438/1439/1434 were deleted or
reverted.

## Changed Files

- `shared/src/protocol.rs`
- `server/src/core/session/state.rs`
- `server/src/core/session/system.rs`
- `server/src/core/session/plugin.rs`
- `server/src/core/session/mod.rs`
- `client/src/network/mod.rs`
- `server/tests/session_scaffold_test.rs`
- `server/tests/placement_timer_multiplier_test.rs`
- `tests/integration/session/room_create_join_test.rs`
- `tests/integration/session/room_list_test.rs`
- `tests/integration/session/lobby_to_draft_initial_test.rs`
- `tests/integration/session/game_over_teardown_test.rs`
- `tests/unit/session/rng_init_failure_test.rs`
- `tests/unit/session/session_ready_test.rs`
- `tests/unit/session/single_fire_test.rs`
- `tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs`
- `tests/integration/playable_client/lobby_entry_test.rs`
- `tests/integration/playable_client/lobby_room_browser_test.rs`
- `tests/integration/playable_client/lobby_confirm_button_reachable_test.rs`
- `tests/integration/playable_client/lobby_confirm_button_stateful_style_test.rs`
- `tests/integration/playable_client/lobby_class_portrait_confirm_repair_test.rs`
- `reports/PROMPT-1441-s19-bot-protocol-room-foundations-integration-refresh.md`

The hand/playable-client test files are compile fixture backfills only:
`SessionSlot` literals now set `is_bot: false`, and `RoomListEntry` literals now
set `bot_count: 0` and `has_human_opponent: true`. No UI behavior was changed.

## Contract Preserved

- Added `BotKind::Default`, `BotActionRejectedReason`, `C2SCreateBotRoom`,
  `C2SAddBot`, `C2SRemoveBot`, and `S2CBotActionRejected` to the shared
  reliable protocol manifest.
- Added `SessionSlot.is_bot` with serde defaulting and mirrored it through room
  and slot protocol payloads.
- Added `RoomListEntry.bot_count` and `RoomListEntry.has_human_opponent` with
  backwards-compatible serde defaults.
- Added server C2S drainers for create-bot-room, add-bot, and remove-bot.
- Added pure helpers `create_bot_room`, `add_bot_to_room`, and
  `remove_bot_from_room`.
- `C2SCreateBotRoom` creates a waiting room and seats a synthetic bot in the
  first opposing-team open slot.
- `C2SAddBot` and `C2SRemoveBot` are owner-only and lobby-waiting-only; rejection
  is unicast via `S2CBotActionRejected`; success broadcasts `S2CSlotUpdated` to
  human occupants.
- Synthetic bot player IDs reserve the high-bit range and are tracked in
  `ActiveSessions` for cleanup.
- Lobby heartbeat timeout checks skip bot slots.
- `join_room` clears `is_bot` when a human occupies a reopened slot.
- Added client network compile proof sites for the new C2S senders and S2C
  rejection receiver.

## Preservation Checks

- Changed-file allowlist: passed.
- `git diff --cached --check`: passed.
- Worker diff had no file deletions; refreshed patch has no deletes.
- Forbidden current-main UI, board-picking, bot foundation, placement submit,
  `production/**`, `stage.txt`, `.github/**`, `.claude/**`, and `Cargo.lock`
  files were not touched.

## Verification

Windows/MSVC Cargo policy was applied:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Targeted checks:

- `cargo test -p shared --lib` - passed, 13 tests.
- `cargo test -p shared --test protocol_completeness_invariant` - passed, 2 tests.
- `cargo test -p server --test room_create_join_test --test room_list_test` -
  passed, 10 + 7 tests.
- `cargo test -p server --test session_ready_test --test single_fire_test --test rng_init_failure_test --test lobby_to_draft_initial_test --test session_scaffold_test --test placement_timer_multiplier_test` -
  passed, 1 + 9 + 1 + 4 + 9 + 1 tests.
- `cargo check -p client` - passed with pre-existing deprecation warnings.

Tooling note: the first non-escalated Cargo command was blocked by access denied
on the mandated target directory. The first server test build was then blocked
by a running `server.exe` holding `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe`.
That process was stopped, `cargo clean -p server` was run against the mandated
target directory, and the targeted server tests passed on rerun.

No full workspace tests were run.

## Push And Main Status

Local integration is committed on `work/s19-bot-protocol-room-foundations-1441`.
The worker branch was pushed to `origin/work/s19-bot-protocol-room-foundations-1441`.
Main has not been pushed from this worker. This branch is ready for orchestrator
main-land.

## Next Bot Work

The next bot slice can consume the protocol/session room foundations to wire a
client UI entry point or the server bot behavior scaffold. This refresh does not
implement bot AI, class selection, gameplay actions, or visible lobby UX.

1441: S19-BOT-PROTOCOL-ROOM-FOUNDATIONS-INTEGRATION-REFRESH: READY_FOR_MAIN_LAND
