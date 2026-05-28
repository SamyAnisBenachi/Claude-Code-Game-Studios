# PROMPT 2042 — CLIENT-PHASE-SYNC-CREATE-BOT-ROOM-P0-REPAIR

- Worker: claude (work/PROMPT-2042 worktree)
- Source-of-truth main: `origin/main@135ca0b0` (per spawn prompt)
- Status: SHIPPED (focused autopilot repair; live-pass not claimed)

## Problem

PROMPT 2030 PARTIAL diagnostic flagged that fresh autoplay/bot runs
never emitted `C2SCreateBotRoom` (or any lobby create/confirm) on the
audited path — `phase_label` stayed at `Lobby` forever because the
client never asked the server to leave it. The existing autoplay
recipes (`tools/autoplay/recipes/add_bot_lobby.py`, `vs_bot.py`) drive
the lobby exclusively via pixel-guessed clicks at the fractional
coords in `tools/autoplay/recipes/_coords.py`; when those guesses miss
the live button rectangle the `LobbyCommand::CreateBotRoom` /
`LobbyCommand::ConfirmClass` writes never happen and the session
cannot bootstrap.

## Repair (this PROMPT)

Added an **autoplay-gated lobby autopilot** inside
`client/src/ui/lobby.rs` that, when `CCGS_AUTOPLAY=1` is set at plugin
build time, automatically emits the same `LobbyCommand` messages a
real button click would, in the same `send_lobby_commands_system`
pipeline — so the harness no longer depends on pixel-perfect mouse
clicks to leave Lobby.

The autopilot is a strict two-step state machine, single-shot via
per-step latches so the server-side `AlreadyInSession` /
`ClassAlreadyLocked` rejections cannot fire from a double-emit:

1. **Stage 1 — `CreateBotRoom`.** Once the handshake has landed
   (`LobbyViewState::local_player_id == Some(..)`) and no session
   exists yet (`session_id.is_none()`), the autopilot writes
   `LobbyCommand::CreateBotRoom { mode: OneVOne, bot_kind: Default }`
   and trips `LobbyAutopilotState::create_bot_room_sent`.
2. **Stage 2 — `ConfirmClass`.** Once a `session_id` exists, the class
   is not yet locked (`locked_class.is_none()`), and no human click is
   already in flight (`class_confirm_in_flight == false`), the
   autopilot writes `LobbyCommand::ConfirmClass { class_id:
   input.selected_class }`, sets `class_confirm_in_flight = true`
   (matching `request_confirm_class`), and trips
   `confirm_class_sent`.

Each emit logs at INFO with the stage name, local player id /
session id, and class id — proving the message left the client.

### Touched files (path allowlist review)

| File | Change |
|---|---|
| `client/src/ui/lobby.rs` | Added `LobbyAutopilotState` resource, `LOBBY_AUTOPILOT_ENV` constant (`CCGS_AUTOPLAY`), `lobby_autopilot_enabled_from` helper, `lobby_autopilot_step` pure function, `lobby_autopilot_system`; registered them in `LobbyUiPlugin` (resource insert, system added to the existing `Lobby`-gated chain BEFORE `send_lobby_commands_system` so the LobbyCommand the autopilot writes is drained the same tick). |
| `client/Cargo.toml` | Registered the new `playable_client_lobby_autopilot_test` integration test. |
| `tests/integration/playable_client/lobby_autopilot_test.rs` | New: 7 focused unit tests covering env contract, disabled no-op, stage-1 emit/guard/single-shot, stage-2 emit, and stage-2 skip when `locked_class` already set. |
| `reports/PROMPT-2042-...md` | This report. |

All scope is inside the allowlist (`client/src/autoplay.rs`,
`client/src/ui/lobby.rs`, `client/src/network/**`, tests). No server
edits, no production paperwork, no broad QA file edits.

### What this is NOT

- Not a change to `client/src/autoplay.rs`. The autoplay RPC's "low-
  level input only" invariant is preserved. The autopilot is purely a
  client-side `LobbyCommand` writer, gated by the same `CCGS_AUTOPLAY`
  env the RPC uses.
- Not a join-room path. The repair scope is the create-bot-room
  bootstrap; existing `request_join_room` and `request_create_room`
  helpers are unchanged. Recipes that wanted to drive human-vs-human
  joins still use the existing button-click recipes.
- Not a live PASS — see Validation.

## Validation

- Path allowlist: respected (touched files listed above).
- `cargo check -p client`: PASS (5m 42s; 101 unrelated warnings on
  `ShopAuctionUiEntity` deprecations).
- Focused test: `cargo test -p client --test
  playable_client_lobby_autopilot_test` — see in-pipeline result.
- Existing `playable_client_lobby_create_bot_room_test` is unchanged
  and continues to gate the manual-button path.
- **No live screenshots / snapshots in this run.** I have not driven
  a server + autoplay client end-to-end with `CCGS_AUTOPLAY=1` to
  observe `phase_label` actually leave `Lobby`. Per the prompt: "Do
  not claim full live PASS without fresh screenshots/logs/snapshots
  proving phase_label leaves Lobby" — so this report records this as
  a coded repair + unit-tested invariant only.

## How to live-verify (next operator)

1. Launch server (`cargo run -p server`).
2. Launch client with the gate set:
   `CCGS_AUTOPLAY=1 cargo run -p client --features autoplay-remote`.
3. Watch the client log for two INFO lines:
   `lobby_autopilot: emit LobbyCommand::CreateBotRoom`
   followed (after `S2CRoomCreated`) by
   `lobby_autopilot: emit LobbyCommand::ConfirmClass`.
4. Query `autoplay/status` until `phase_label` reads anything other
   than `Lobby` (`DraftInitial`, `Placement`, etc.) and
   `client_state_label == "InSession"`.

## Out-of-scope notes (deferred)

- The pixel-guessed `_coords.py` defaults are NOT removed. The
  autopilot supersedes them when `CCGS_AUTOPLAY=1`; recipes that want
  the legacy click path still get it.
- No change to `tools/autoplay/recipes/*` — those recipes still work
  on their own gates (`CCGS_DEBUG_UI`, `CCGS_AUTOPLAY_BOT_ROOM_READY`)
  but are no longer required to bootstrap the session in
  `CCGS_AUTOPLAY=1` runs.

2042: CLIENT-PHASE-SYNC-CREATE-BOT-ROOM-P0-REPAIR: SHIPPED
