# PLAYABLE-003 Phase Capture Notes

This capture set is internal friend-game evidence only. Local paths, usernames, hardware details, process ids, raw timestamps, and transient ports are redacted.

## Attempt 1

- Command set:
  - `SERVER_PORT=<PORT> cargo run -p server`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client`
- Result:
  - Server failed before launch completion in `auction_tick_system` because `CardCatalog` did not exist.
  - Clients were not valid primary-client launches because Cargo required `--bin client`.
- Files:
  - `attempt-1-server.stderr.log`
  - `attempt-1-client-a.stderr.log`
  - `attempt-1-client-b.stderr.log`

## Attempt 2

- Command set:
  - `SERVER_PORT=<PORT> cargo run -p server`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
- Result:
  - Server failed before lobby in `apply_attract_displacements` because `SessionConfig` did not exist.
  - Clients failed at startup because `StatesPlugin` was added twice.
- Files:
  - `attempt-2-server.stderr.log`
  - `attempt-2-client-a.stderr.log`
  - `attempt-2-client-b.stderr.log`

## Attempt 3

- Command set:
  - `SERVER_PORT=<PORT> cargo run -p server`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
- Result:
  - Server process remained alive through the client capture window.
  - Client A and Client B launched native windows.
  - Both clients logged `lightyear_messages::receive` `MissingComponent(ComponentId(320))`.
  - Root cause was later traced to client/server Lightyear feature mismatch: server registered replication/protocol-check messages, client did not.
- Files:
  - `server.stderr.log`
  - `client-a.stderr.log`
  - `client-b.stderr.log`
  - `live-process-summary.json`

## Attempt 5

- Command set:
  - `SERVER_PORT=<PORT> cargo run -p server`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
  - `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
- Result:
  - Server process remained alive through the post-repair client capture window.
  - Client A launched a native window and did not log `MissingComponent`.
  - Client B launched a native window and did not log `MissingComponent`; it exited after the hidden capture window closed.
  - Production live logs still do not prove room creation, join, class confirmation, session entry, draft/shop, placement, resolution, next loop, or game-over.
- Files:
  - `attempt-5-server.stderr.log`
  - `attempt-5-client-a.stderr.log`
  - `attempt-5-client-b.stderr.log`
  - `attempt-5-live-process-summary.json`

## Automated Smoke

- Command: `cargo test -p server --test playable_client_real_e2e_loop_test`
- Result:
  - Client/server Lightyear feature alignment verified.
  - Real Lightyear client sent `C2SHello`.
  - Real Lightyear server mapped the peer to `PlayerId(1)`.
  - Real Lightyear client received `S2CHandshake`.
- Limit:
  - This smoke does not fake or claim full manual two-client completion.

## Prompt 290 Controlled Room/Session Repair

- Command: `cargo test -p server --test playable_client_real_e2e_loop_test`
- Focused test: `real_lightyear_two_client_room_session_reaches_class_reveal_and_session_entry`
- Result:
  - Host and joiner both completed real Lightyear fresh hello/handshake.
  - Host sent `C2SCreateRoom`; server returned `S2CRoomCreated`.
  - Room code was captured from the server response and redacted as `<ROOM_CODE_6_CHARS>`.
  - Joiner sent `C2SJoinRoom`; server returned `S2CJoinAck`.
  - Host received `S2CSlotUpdated`.
  - Host and joiner both sent `C2SSelectClass` and `C2SConfirmClass`.
  - Both clients received `S2CClassLocked` and `S2CClassesRevealed`.
  - Server promoted the ready room to `GameActive`, built `SessionConfig`, entered `RoundState::DraftInitial`, and both clients received `S2CPhaseChanged(DraftInitial)`.
- Files:
  - `prompt-290-room-session-trace.json`
- Limit:
  - This repair proves the controlled room/session endpoint only. It does not claim `DRAFT_SHOP`, auction, placement, resolution, next-loop, game-over, full playable-client manual QA, or public release readiness.

## Phase Summary

| Phase | Capture | Result |
|---|---|---|
| Server launch | `attempt-5-server.stderr.log` | Reached after repairs |
| Client A launch | `attempt-5-client-a.stderr.log` | Reached after repairs |
| Client B launch | `attempt-5-client-b.stderr.log` | Reached after repairs |
| Fresh hello | `playable_client_real_e2e_loop_test`, `prompt-290-room-session-trace.json` | Reached by automated real-Lightyear smoke and controlled two-client trace |
| Create room | `prompt-290-room-session-trace.json` | Reached by controlled real-Lightyear primary-client trace |
| Join room | `prompt-290-room-session-trace.json` | Reached by controlled real-Lightyear primary-client trace |
| Class select/confirm | `prompt-290-room-session-trace.json` | Reached by controlled real-Lightyear primary-client trace |
| Server-confirmed session entry | `prompt-290-room-session-trace.json` | Reached: server room `GameActive`, `SessionConfig`, `RoundState::DraftInitial`, client `S2CPhaseChanged(DraftInitial)` |
| DRAFT_INITIAL | `prompt-290-room-session-trace.json` | Reached by controlled real-Lightyear primary-client trace |
| DRAFT_SHOP | none | Not reached in prompt 290 room/session repair |
| Auction | none | Not reached |
| Placement submit | none | Not reached |
| Placement reveal | none | Not reached |
| Resolution replay | none | Not reached |
| Next loop | none | Not reached |
| Game-over or nearest endpoint | `prompt-290-room-session-trace.json` | Nearest controlled endpoint is `DRAFT_INITIAL`; full loop not verified |
