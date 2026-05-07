# PLAYABLE-003 Real End-to-End Loop Evidence

This is internal friend-game evidence only. It is not public release readiness, not broad accessibility completion, not playtest validation, not fun-hypothesis validation, not QA sign-off, and not full playable-client manual QA.

## Version Control

- Worker branch: `work/playable-003-room-session-evidence-repair`
- Base commit at room/session repair start: `b00cc3b123d04d5ea7c49f29b85985dc5c3f8599`
- Rebased onto current `origin/main`: `5993be5` (`docs: repair ECO-004 readiness`) before worker push.
- Prior sanitized replacement history: created from `origin/main` and imported only final sanitized file contents, without carrying earlier local capture-history commits.
- Repair/evidence commit: recorded in the worker response after commit and push.
- Worker dirty status during prompt 290 capture: repair files plus sanitized evidence changes.

## Runtime

- Environment type: Windows native server plus two Windows native primary clients
- Local hardware, username, machine identifiers, process ids, raw timestamps, local filesystem paths, and transient port values are redacted from committed evidence
- Rust: `rustc 1.95.0`
- Cargo: `cargo 1.95.0`
- Trunk: `trunk 0.21.14`
- Browser target: not exercised in this repair pass
- Native target: `cargo run -p server` plus two `cargo run -p client --bin client` processes

## Commands

- Server: `SERVER_PORT=<PORT> cargo run -p server`
- Client A: `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
- Client B: `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client`
- Client URL if WASM: not applicable; native clients used `SERVER_URL=ws://localhost:<PORT>`

## Root Cause And Repair

`MissingComponent(ComponentId(320))` was caused by a Lightyear protocol net-id mismatch. The server build enabled Lightyear `replication`, which registers protocol-check and replication metadata messages before the shared gameplay protocol. The native client build did not enable the same protocol-affecting feature, so it decoded the first server metadata payload as a C2S gameplay message. In the pre-repair client app, `ComponentId(320)` mapped to the missing client-side `MessageReceiver<C2SHello>` path, which is intentionally absent for a client-to-server-only message.

Repair:

- `client/Cargo.toml` now enables Lightyear `replication` so client/server message and metadata net-id tables align.
- `server/Cargo.toml` now registers the story-declared `playable_client_real_e2e_loop_test`.
- `tests/integration/playable_client/real_e2e_loop_test.rs` verifies the protocol-affecting feature alignment and a real Lightyear fresh-hello handshake without faking full manual completion.
- Prompt 290 repair adds a server-authored ready-room promotion path: when a room-backed lobby has all slots filled and all classes confirmed, the server builds `SessionConfig`, initializes reconnect/session resources, marks the room `GameActive`, triggers `SessionReady`, and enters the RSM `DraftInitial` path. This keeps room/session entry server-authored and does not inject gameplay state into client ECS.
- `tests/integration/playable_client/real_e2e_loop_test.rs` now includes a controlled two-primary-client Lightyear test that sends the real `C2SCreateRoom`, `C2SJoinRoom`, `C2SSelectClass`, and `C2SConfirmClass` messages and observes the corresponding server S2C messages through Lightyear receivers.

## Capture Manifest

Capture directory: `production/qa/evidence/captures/playable-client-real-e2e-loop/`

- `attempt-1-*.stderr.log`: sanitized initial failure summaries for invalid client command and missing server `CardCatalog` startup resource.
- `attempt-2-*.stderr.log`: sanitized failure summaries for missing server `SessionConfig` startup resource and duplicate native client `StatesPlugin`.
- `server.stderr.log`, `client-a.stderr.log`, `client-b.stderr.log`: sanitized pre-repair live summaries showing native launch followed by `MissingComponent(ComponentId(320))` on both clients.
- `attempt-5-server.stderr.log`, `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log`: sanitized post-repair live summaries showing server and two native clients launched with no `MissingComponent` logged during the capture window.
- `attempt-*-live-process-summary.json`, `live-process-summary.json`: sanitized process summaries with pids and transient port redacted.
- `prompt-290-room-session-trace.json`: sanitized controlled Lightyear trace for host create-room, joiner join-room, class select/confirm, class reveal, and server-confirmed `DraftInitial` session entry.
- `phase-captures.md`: reached and unreached phase notes.

## No Harness Statement

No direct production `World` injection or harness-injected game state was used for the live process attempts. The live attempts used one real local server process and two real primary native client processes launched from the worker checkout. The automated prompt 290 room/session test uses in-process Lightyear server/client apps as controlled primary-client evidence: it drives real C2S messages through Lightyear `MessageSender`s and observes real S2C messages through Lightyear `MessageReceiver`s. It does not mutate authoritative gameplay state directly and does not claim completion of the full manual two-client game loop.

## Reached Endpoint

Post-repair live process attempt:

1. Server launch reached and stayed alive through the capture window.
2. Client A native launch reached and no `MissingComponent` was logged.
3. Client B native launch reached and no `MissingComponent` was logged.
4. Production client logs still do not emit enough handshake, room, session-entry, or phase-transition evidence to prove the full real primary-client path.

Automated smoke evidence:

1. `cargo test -p server --test playable_client_real_e2e_loop_test` reached a real Lightyear fresh hello.
2. The server mapped the peer to `PlayerId(1)`.
3. The client received `S2CHandshake`.

Prompt 290 controlled room/session evidence:

1. Host and joiner both fresh-started through real Lightyear `C2SHello`/`S2CHandshake`.
2. Host sent `C2SCreateRoom`; server returned `S2CRoomCreated`.
3. The room code was captured from `S2CRoomCreated` and reused by the joiner.
4. Joiner sent `C2SJoinRoom`; server returned `S2CJoinAck`; host received `S2CSlotUpdated`.
5. Host sent `C2SSelectClass(Iop)` and `C2SConfirmClass(Iop)`.
6. Joiner sent `C2SSelectClass(Cra)` and `C2SConfirmClass(Cra)`.
7. Both clients received `S2CClassesRevealed`.
8. The server promoted the ready room to `GameActive`, inserted `SessionConfig`, entered `RoundState::DraftInitial`, and both clients received `S2CPhaseChanged(DraftInitial)`.

Room creation, join, class select/confirm, class reveal, and server-confirmed session entry are now verified by controlled real-Lightyear primary-client evidence. `DRAFT_SHOP`, auction, placement submit, placement reveal, resolution replay, next loop, and game-over remain unverified from the full friend-game loop and are not claimed here.

## Phase Checklist

| Phase | Result | Evidence |
|---|---|---|
| Server launch | Reached after repair | `attempt-5-server.stderr.log`, `attempt-5-live-process-summary.json` |
| Client A launch | Reached after repair | `attempt-5-client-a.stderr.log`, `attempt-5-live-process-summary.json` |
| Client B launch | Reached after repair | `attempt-5-client-b.stderr.log`, `attempt-5-live-process-summary.json` |
| Fresh hello | Reached by automated real-Lightyear smoke and prompt 290 controlled two-client trace | `playable_client_real_e2e_loop_test`, `prompt-290-room-session-trace.json` |
| Create room | Reached by controlled real-Lightyear primary-client trace | `prompt-290-room-session-trace.json` |
| Join room | Reached by controlled real-Lightyear primary-client trace | `prompt-290-room-session-trace.json` |
| Class select/confirm | Reached by controlled real-Lightyear primary-client trace | `prompt-290-room-session-trace.json` |
| Server-confirmed session entry | Reached by controlled real-Lightyear primary-client trace | Server `RoomSession::GameActive`, `SessionConfig`, `RoundState::DraftInitial`, client `S2CPhaseChanged(DraftInitial)` |
| DRAFT_INITIAL | Reached by controlled real-Lightyear primary-client trace | `prompt-290-room-session-trace.json` |
| DRAFT_SHOP | Not reached in prompt 290 room/session repair | Full loop evidence remains future PLAYABLE-003 scope |
| Auction | Not reached in live process evidence | Blocked before session entry |
| Placement submit | Not reached in live process evidence | Blocked before session entry |
| Placement reveal | Not reached in live process evidence | Blocked before session entry |
| Resolution replay | Not reached in live process evidence | Blocked before session entry |
| Next loop | Not reached in live process evidence | Blocked before session entry |
| Game-over or nearest endpoint | Nearest endpoint documented | Controlled room/session endpoint reaches `DRAFT_INITIAL`; full loop not verified |

## Defects

| ID | Severity | Owner/System | Status | Friend-game Impact | Evidence |
|---|---|---|---|---|---|
| PLAYABLE-003-D1 | Blocker | Auction startup scheduling | Repaired in this branch | Server could not launch from `cargo run -p server` without loaded `CardCatalog` | `attempt-1-server.stderr.log` |
| PLAYABLE-003-D2 | Blocker | Board displacement scheduling | Repaired in this branch | Server could not remain alive before session setup | `attempt-2-server.stderr.log` |
| PLAYABLE-003-D3 | Blocker | Native client startup | Repaired in this branch | Primary native client could not launch | `attempt-2-client-a.stderr.log`, `attempt-2-client-b.stderr.log` |
| PLAYABLE-003-D4 | Blocker | Lightyear primary-client receive path | Repaired in this branch | Two real primary clients logged `MissingComponent(ComponentId(320))` before protocol feature alignment | `client-a.stderr.log`, `client-b.stderr.log`, `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log` |
| PLAYABLE-003-D5 | Blocker | PLAYABLE-003 automated coverage | Repaired in this branch | Story-declared test target was absent | `cargo test -p server --test playable_client_real_e2e_loop_test` |
| PLAYABLE-003-D6 | Major evidence gap | Real two-client room/session flow capture | Repaired for room/session scope | Host create, joiner join, class reveal, and session entry are now covered by controlled real-Lightyear evidence | `prompt-290-room-session-trace.json` |
| PLAYABLE-003-D7 | Major evidence gap | Full friend-game loop capture | Open | Draft/shop, auction, placement, resolution, next loop, and game-over are still not proven from the full two-client path | Phase checklist |

## Verification Results

Pre-repair verification passed for the previously listed PLAYABLE-003 regressions, but the story-declared E2E test target was absent and the live client logs found `MissingComponent(ComponentId(320))`.

Prompt 290 verification results:

- `cargo fmt -p client -p server -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test -p client --test playable_client_lobby_entry_test`: PASS, 5 passed.
- `cargo test -p server --test playable_client_lobby_entry_server_test`: PASS, 3 passed.
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`: PASS, 4 passed.
- `cargo test -p server --test playable_client_draft_ready_bridge_test`: PASS, 3 passed.
- `cargo test -p server --test playable_client_real_e2e_loop_test`: PASS, 3 passed.
- `cargo test -p server --test session_ready_test`: PASS, 4 passed.
- `cargo test -p server --test room_create_join_test`: PASS, 7 passed.
- `cargo test -p server --test class_reveal_test`: PASS, 8 passed.
- `cargo test -p server --test class_lifecycle_test`: PASS, 9 passed.
- `git diff --check origin/main...HEAD`: PASS.

Because the full real primary-client game loop remains unverified from live two-client evidence, this evidence still does not claim full friend-game loop completion.
