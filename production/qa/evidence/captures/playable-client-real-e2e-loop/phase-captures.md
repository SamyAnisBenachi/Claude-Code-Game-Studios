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

## Prompt 296 Controlled DraftShop Repair

- Command: `cargo test -p server --test playable_client_real_e2e_loop_test`
- Focused test: `real_lightyear_two_client_draft_initial_purchase_ready_reaches_draft_shop`
- Result:
  - Host and joiner both completed the prompt 290 real Lightyear room/session path.
  - Room-backed `SessionConfig` was bridged into server-authoritative `PlayerSessions` before `SessionReady` observers consumed class identity.
  - Both clients received `S2CDraftOffering` with nine server-offered cards.
  - Host and joiner each sent real `C2SPurchaseCard` using a card ID from the received offering.
  - Server accepted purchase, updated authoritative `PlayerHands`, and emitted `S2CCardAcquired(source=DraftInitial)` plus purchase `S2CGoldUpdate`.
  - Host sent ready, retract, then ready again through real `C2SSignalReady`; joiner sent real ready; server RSM observed the retract path.
  - The real RSM route advanced all-ready `DraftInitial` to `Placement`, then both clients sent real empty `C2SSubmitPlacement`.
  - Server emitted `S2CPlacementReveal`, `S2CResolutionEvent`, and then `S2CPhaseChanged(DraftShop)`.
- Files:
  - `prompt-296-draft-shop-trace.json`
- Limit:
  - This repair proves the controlled real-Lightyear endpoint through `DRAFT_SHOP`. It does not claim auction, non-empty placement gameplay, next loop after `DRAFT_SHOP`, game-over, full playable-client manual QA, playtest validation, or public release readiness.

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
| DRAFT_INITIAL | `prompt-290-room-session-trace.json`, `prompt-296-draft-shop-trace.json` | Reached by controlled real-Lightyear primary-client trace |
| DRAFT_INITIAL offering | `prompt-296-draft-shop-trace.json` | Reached: both clients received server-authored `S2CDraftOffering(card_count=9)` |
| Purchase/acquisition/economy | `prompt-296-draft-shop-trace.json` | Reached: clients sent real `C2SPurchaseCard`; server emitted `S2CCardAcquired` and `S2CGoldUpdate` |
| Ready/retract | `prompt-296-draft-shop-trace.json` | Reached: real `C2SSignalReady` ready/retract/ready observed by server RSM |
| DRAFT_SHOP | `prompt-296-draft-shop-trace.json` | Reached via real server route `DRAFT_INITIAL -> PLACEMENT -> RESOLUTION -> DRAFT_SHOP` |
| Auction | none | Not reached |
| Placement submit | `prompt-296-draft-shop-trace.json` | Reached with real empty `C2SSubmitPlacement` from both clients |
| Placement reveal | `prompt-296-draft-shop-trace.json` | Reached with server-authored empty `S2CPlacementReveal` |
| Resolution replay | `prompt-296-draft-shop-trace.json` | Protocol reached `S2CResolutionEvent`; no manual visual replay validation claimed |
| Next loop | none | Not reached beyond `DRAFT_SHOP` |
| Game-over or nearest endpoint | `prompt-296-draft-shop-trace.json` | Nearest controlled endpoint is `DRAFT_SHOP`; full loop not verified |
