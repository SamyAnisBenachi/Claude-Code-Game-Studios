# PROMPT 1537 — Bot Lobby Add-Bot UI Wiring

- **Branch:** `worker/prompt-1537-bot-lobby-add-bot-ui`
- **Base:** `origin/main@38975b51dcce63d649bf6d9bf0ecbf2ecfe84b1d`
- **Worktree:** `D:/Tmp/wt-1537`
- **Status:** SHIPPED

## Scope

Wire the minimal client lobby UI affordances for adding a bot to a room, using
existing protocol/server capabilities only. Two surfaces:

1. **`Play vs Bot`** — pre-session CTA in the create/join row. Writes
   `C2SCreateBotRoom { mode: OneVOne, bot_kind: Default }`.
2. **`Add Bot`** — in-room CTA below the slot panels. Writes
   `C2SAddBot { slot, bot_kind: Default }` targeting the first empty
   non-bot slot.

Bot local/single-player remains out of scope.

## Protocol surface used (pre-existing, not modified)

- `C2SCreateBotRoom`  (shared/src/protocol.rs:487)
- `C2SAddBot`  (shared/src/protocol.rs:496)
- `S2CBotActionRejected`  (shared/src/protocol.rs:731)
- `BotKind`  (shared/src/protocol.rs:263)

No shared protocol changes were made.

## Files touched

| File | Change |
|------|--------|
| `client/src/ui/lobby.rs` | Add `LobbyPlayVsBotButton`, `LobbyAddBotButton` markers + spawn nodes; `LobbyCommand::{PlayVsBot, AddBot}`; `request_play_vs_bot`/`request_add_bot` helpers; `refresh_bot_buttons_visibility_system`; `drain_lobby_bot_rejections_system`; `bot_action_in_flight` latch on `LobbyInputState`; new `LobbyDynamicText::{PlayVsBot, AddBot}` variants; `lobby_add_bot_target_slot` selector. |
| `client/Cargo.toml` | Register `playable_client_lobby_bot_affordances_test`. |
| `tests/integration/playable_client/lobby_bot_affordances_test.rs` | New — 9 tests covering selector, send paths, in-flight latch, no-op preconditions. |
| `tests/integration/playable_client/lobby_entry_test.rs` | Add `bot_action_in_flight: false` to the explicit `LobbyInputState` literal so the struct-init test stays exhaustive. |

## UI contract

- Visibility is the single source of truth — buttons are hidden (`Display::None`)
  rather than rendered as disabled-looking labels, per the prompt instruction
  "avoid disabled button-looking labels".
  - `Play vs Bot`: visible iff `lobby.session_id.is_none()`.
  - `Add Bot`: visible iff `lobby.session_id.is_some()` **and**
    `lobby_add_bot_target_slot(&lobby).is_some()`.
- Pending state is communicated by the button label, not a disabled state:
  - `Play vs Bot` → `Starting vs bot...`
  - `Add Bot` → `Adding bot...`
  - Label flips back on `S2CRoomCreated` / `S2CSlotUpdated` /
    `S2CBotActionRejected`.
- A single shared latch `LobbyInputState.bot_action_in_flight` prevents either
  button from stacking duplicate requests while one is pending.
- Rejections surface via `S2CBotActionRejected` into the lobby status banner.

## Eligibility rule for `Add Bot`

`lobby_add_bot_target_slot` picks the first slot where `player_id.is_none() &&
!is_bot`. Server remains authoritative on owner-check + final eligibility; a
race against another player's join is recovered through `S2CBotActionRejected`
(latch cleared, status banner updated).

## Validation

- `git diff --check`: clean.
- Path allowlist: only `client/Cargo.toml`, `client/src/ui/lobby.rs`,
  `tests/integration/playable_client/lobby_bot_affordances_test.rs`,
  `tests/integration/playable_client/lobby_entry_test.rs`. All inside the
  prompt's owned scope (`client lobby UI modules and direct lobby UI tests` +
  this report).
- Focused tests:
  - `cargo test -p client --test playable_client_lobby_bot_affordances_test`
    → **9 passed / 0 failed**.
  - `cargo test -p client --test playable_client_lobby_entry_test`
    → **6 passed / 0 failed** (regression check on the literal
    `LobbyInputState` site).
- Broader Cargo suite intentionally not run (prompt: "Do not run broad Cargo
  suites").

## Owned-scope check

No edits outside the lobby UI surface:

- No server / bot action-loop files touched (PROMPT 1531 territory).
- No `shared/src/protocol.rs` modifications.
- No sprint/session/state paperwork edits.
- No `Cargo.toml` changes outside `client/Cargo.toml`'s test registration.

## Final line

1537: BOT-LOBBY-ADD-BOT-UI-WIRING: SHIPPED
