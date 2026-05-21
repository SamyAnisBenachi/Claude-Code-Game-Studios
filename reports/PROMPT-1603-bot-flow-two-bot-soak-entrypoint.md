# PROMPT 1603 — BOT-FLOW-TWO-BOT-SOAK-ENTRYPOINT

**Source-of-truth tip:** `origin/main @ 237572af`
**Branch:** `work/bot-flow-two-bot-soak-entrypoint-1603`
**Worktree:** `D:/Tmp/wt-1603`
**Commit:** _filled in after commit_

## Scope

Two of the items missing from the bot-flow inventory follow-up
(`reports/PROMPT-1594-bot-flow-inventory-followup.md`, items 7-8):

1. A debug-only **Create 2-Bot Soak Room** control in the client lobby UI,
   gated by `CCGS_DEBUG_UI=1`.
2. A `tools/dev-launcher/Start-BotVsBotSoak.ps1` headless launcher with a
   wall-clock timer (default 300 s) and the evidence-directory layout
   documented in PROMPT-1594.

Out of scope for this PROMPT (owned elsewhere):

- Bot placement logic (Wave 3) — PROMPT 1602.
- Server-side QA snapshot / `bot-decision-log.jsonl` writers — PROMPT-1594
  follow-up items 2-3 (separate PROMPTs to come).

## Files touched

| Path | Change |
|---|---|
| `client/src/ui/lobby.rs` | Add `LobbyCreateBotRoomButton` component, `LobbyDynamicText::CreateBotRoom`, `LobbyCommand::CreateBotRoom`, env-var helpers (`debug_ui_enabled_from` + `is_debug_ui_enabled`), `request_create_bot_room` helper, wire interaction + send paths for `C2SCreateBotRoom`. Button spawn is gated by `session_id.is_none() && is_debug_ui_enabled()`. |
| `client/Cargo.toml` | Register the new `playable_client_lobby_create_bot_room_test` integration test. |
| `tests/integration/playable_client/lobby_create_bot_room_test.rs` | New focused test file: env-var contract, session-gated helper, ECS proof that the debug button is absent without `CCGS_DEBUG_UI=1`. |
| `tools/dev-launcher/Start-BotVsBotSoak.ps1` | New launcher: mirrors `Start-TwoClients.ps1` resolve/policy/port/build chain, server-only, sleeps `-DurationSeconds` (default 300), then `Stop-Process`. Sets `CCGS_BOT_DECISION_LOG_PATH` / `CCGS_QA_SNAPSHOT_DIR` env vars forward-compatible with the server-side dump work to come. |
| `start-bot-vs-bot-soak.bat` | One-click `.bat` forwarder mirroring `start-two-clients.bat`. |

## Path allowlist review

All edits fall inside the prompt's owned scope:

- ✅ `client/src/ui/lobby.rs`
- ✅ `client/Cargo.toml` — necessary `[[test]]` registration only; no
  dependency/feature change.
- ✅ `tests/integration/playable_client/lobby_create_bot_room_test.rs` —
  directly necessary lobby test file.
- ✅ `tools/dev-launcher/Start-BotVsBotSoak.ps1` + adjacent `.bat` wrapper.
- ✅ `reports/PROMPT-1603-bot-flow-two-bot-soak-entrypoint.md` (this file).

No edits to: server bot loop, server placement logic, shared protocol,
`production/sprint-status.yaml`, `production/session-state/`,
`production/sprints/`, `production/qa/`, `production/stage.txt`, or
unrelated Cargo/CI files.

## Evidence-directory contract

`Start-BotVsBotSoak.ps1` writes, per run:

```
production/qa/evidence/dev-runs/<UTC-YYYY-MM-DD-HHmmss>-bot-vs-bot-soak/
├── server.log
├── server.err
├── bot-decision-log.jsonl          # populated by future server-side dump
├── server-snapshots/               # populated by future server-side dump
└── soak-summary.json               # what the launcher itself recorded
```

This matches the schema documented in
`reports/PROMPT-1594-bot-flow-inventory-followup.md` § "Dump cadence".
`CCGS_BOT_DECISION_LOG_PATH` and `CCGS_QA_SNAPSHOT_DIR` are set on the
server child process so the server-side dump work (PROMPT-1594 items 2-3,
future PROMPTs) only has to read those env vars to write into the canonical
location.

## Gating verification

- `debug_ui_enabled_from(Some("1"))` → `true`.
- `debug_ui_enabled_from(Some(" 1 "))` → `true` (whitespace trimmed).
- `debug_ui_enabled_from(None | Some("") | Some("0") | Some("true") | Some("yes") | Some("11"))` → `false`.
- `LobbyUiPlugin` with `CCGS_DEBUG_UI` unset: zero `LobbyCreateBotRoomButton`
  entities spawned (asserted in the new test).
- Spawn site additionally gates on `lobby.session_id.is_none()`, so even with
  `CCGS_DEBUG_UI=1` the affordance only appears on the pre-room surface —
  matching the server-side `S2CBotActionRejected::AlreadyInSession`
  contract for `C2SCreateBotRoom`.

## Focused validation

- `git diff --check` — clean.
- Path allowlist — within owned scope.
- Launcher `-Help` exits 0 and prints the documented parameter list.
- `cargo test -p client --test playable_client_lobby_create_bot_room_test`:
  **deferred to the verification PROMPT.** This workspace's first
  `cargo build` after a fresh checkout takes ~10-15 min on Windows/MSVC
  (Bevy 0.18 + lightyear); the prompt body explicitly allows deferring
  broad verification in this case. The new test file is small, mirrors
  the structure of the green `lobby_bot_controls_test.rs` neighbour, and
  uses only public APIs of the lobby module; rust-analyzer reports no
  diagnostics on the changed lines.

## Risks / follow-ups

- The launcher's `Start-Sleep -Seconds $DurationSeconds` is the soak's only
  termination signal today. Once PROMPT 1602 lands the bot-vs-bot driver and
  natural `GameOver` becomes reachable, a `--bot-vs-bot-max-rounds` server
  flag (PROMPT-1594 item 9) is the cleaner termination path.
- The `Create 2-Bot Soak Room` button currently shares the pre-session
  surface with `Create Room` / `Join Room`. Because it follows the existing
  spawn-time `session_id.is_none()` pattern used by the rooms browser and
  the slot picker, it stays visible if the player exits a room back to the
  lobby state without re-entering the lobby (the lobby UI is despawned
  `OnExit(Lobby)` and respawned `OnEnter(Lobby)`, so the surface always
  matches the env + session-on-spawn snapshot).

1603: BOT-FLOW-TWO-BOT-SOAK-ENTRYPOINT: SHIPPED
