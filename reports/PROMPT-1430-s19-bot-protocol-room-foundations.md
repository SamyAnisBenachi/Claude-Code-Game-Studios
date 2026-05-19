# PROMPT 1430 - S19-BOT-PROTOCOL-ROOM-FOUNDATIONS

Status: DONE
Branch: `work/s19-bot-protocol-room-foundations-1430`
Commit: pending commit at report write time; final commit is recorded in the worker relay/final summary after commit.

## Partial Diff Disposition

Reused and completed the previous partial diff. The existing changes to
`shared/src/protocol.rs`, `server/src/core/session/state.rs`,
`server/src/core/session/system.rs`, and the session fixture backfills were
coherent: they added `SessionSlot.is_bot`, bot protocol message types, reliable
registration, and room-list bot metadata. This pass kept that direction and
finished the missing server-side room/session behavior, invariant proof sites,
tests, report, and commit prep.

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
- `reports/PROMPT-1430-s19-bot-protocol-room-foundations.md`

## Protocol And Session Contract Added

- Added `BotKind::Default`, `BotActionRejectedReason`, `C2SCreateBotRoom`,
  `C2SAddBot`, `C2SRemoveBot`, and `S2CBotActionRejected` to the shared
  reliable protocol manifest.
- Added `SessionSlot.is_bot` with serde defaulting, mirrored from server
  session slots into `S2CRoomCreated`, `S2CJoinAck`, and `S2CSlotUpdated`.
- Added `RoomListEntry.bot_count` and `RoomListEntry.has_human_opponent` with
  backwards-compatible serde defaults.
- Added server C2S drainers for create-bot-room, add-bot, and remove-bot.
- Added pure session helpers:
  - `create_bot_room`
  - `add_bot_to_room`
  - `remove_bot_from_room`
- `C2SCreateBotRoom` creates a fresh waiting room and seats a synthetic bot in
  the first opposing-team open slot.
- `C2SAddBot` and `C2SRemoveBot` are owner-only, lobby-waiting-only operations.
  Rejections are unicast as `S2CBotActionRejected`; successful changes broadcast
  `S2CSlotUpdated` to human occupants.
- Synthetic bot player ids reserve the high-bit range and are tracked in
  `ActiveSessions` so room cleanup can remove them.
- Lobby heartbeat timeout checks skip `is_bot` slots. Bots do not fake network
  heartbeats.
- `join_room` clears `is_bot` when a human later occupies a reopened slot.
- Added non-UI client network compile proof sites for the new C2S senders and
  S2C rejection receiver so the protocol completeness invariant remains green.

## Verification

Windows/MSVC Cargo policy was applied for every Cargo command:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Formatting:

- `cargo fmt` was attempted first and failed with Windows `os error 206`
  (filename or extension too long).
- `rustfmt` was run directly on the touched Rust files and succeeded.

Targeted tests/checks:

- `cargo test -p shared --lib` - passed, 13 tests.
- `cargo test -p shared --test protocol_completeness_invariant` - passed, 2 tests.
- `cargo test -p server --test room_create_join_test --test room_list_test` -
  passed, 10 + 7 tests.
- `cargo test -p server --test session_ready_test --test single_fire_test --test rng_init_failure_test --test lobby_to_draft_initial_test --test session_scaffold_test --test placement_timer_multiplier_test` -
  passed, 1 + 9 + 1 + 4 + 9 + 1 tests.
- `cargo check -p client` - passed after clearing stale shared/client package
  artifacts from the shared target directory with `cargo clean -p shared -p client`.

Notes:

- The first non-escalated Cargo command was blocked by target-directory access
  denied on `.cargo-lock`; all Cargo verification then ran with approved
  escalation against the mandated target directory.
- The first `cargo check -p client` picked an old `shared` rmeta from the
  shared target cache and failed to see the new protocol types. Clearing only
  `shared` and `client` artifacts fixed the cache issue; the rerun passed.
- No full workspace tests were run.

## Dependency Status Relative To PROMPT 1435

This slice is protocol/session-only and does not depend on the PROMPT 1428 bot
foundation scaffold or PROMPT 1435 integration landing on `main`. It does not
import `server/src/feature/bot/*` types. The future bot foundation can consume
the `is_bot` room slot contract and synthetic `PlayerId` occupancy later.

## Explicit Non-Claims

- No bot AI heuristic was implemented.
- No client UX, lobby button, HUD tag, shop/auction copy, or result-screen UI
  was implemented.
- No local-vs-bot playable flow exists yet.
- Bots do not select/confirm classes or submit gameplay actions in this slice.
- No sprint/story paperwork, `production/**`, `stage.txt`, `.github/**`, or
  release/stage files were touched.

1430: S19-BOT-PROTOCOL-ROOM-FOUNDATIONS: DONE
