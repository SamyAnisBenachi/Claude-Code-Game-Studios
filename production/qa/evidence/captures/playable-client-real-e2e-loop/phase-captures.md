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

## Phase Summary

| Phase | Capture | Result |
|---|---|---|
| Server launch | `attempt-5-server.stderr.log` | Reached after repairs |
| Client A launch | `attempt-5-client-a.stderr.log` | Reached after repairs |
| Client B launch | `attempt-5-client-b.stderr.log` | Reached after repairs |
| Fresh hello | `playable_client_real_e2e_loop_test` | Reached by automated real-Lightyear smoke; not directly logged by live native clients |
| Create room | none | Not reached in live process evidence |
| Join room | none | Not reached in live process evidence |
| Class select/confirm | none | Not reached in live process evidence |
| Server-confirmed session entry | none | Not reached |
| DRAFT_INITIAL | none | Not reached |
| DRAFT_SHOP | none | Not reached |
| Auction | none | Not reached |
| Placement submit | none | Not reached |
| Placement reveal | none | Not reached |
| Resolution replay | none | Not reached |
| Next loop | none | Not reached |
| Game-over or nearest endpoint | `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log` | Nearest live endpoint is native two-client launch with MissingComponent repaired |
