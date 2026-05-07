# PLAYABLE-003 Real End-to-End Loop Evidence

This is internal friend-game evidence only. It is not public release readiness, not broad accessibility completion, not playtest validation, not fun-hypothesis validation, not QA sign-off, and not full playable-client manual QA.

## Version Control

- Worker branch: `work/playable-003-draftinitial-draftshop-evidence`
- Base commit at prompt 296 repair start: `54681615f725cf1d7be19fd70dc27f133ebcf39f`
- Prompt 296 scope: continue PLAYABLE-003 only; do not create PLAYABLE-004; do not run `/story-done`.
- Prior room/session repair branch: `work/playable-003-room-session-evidence-repair`
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
- Prompt 296 found the next missing server transition: room/session readiness created `SessionConfig`, but `PlayerSessions` stayed empty, so the card pool/acquisition subscribers could not resolve player class identity and did not emit `S2CDraftOffering`. `server/src/core/session/system.rs` now populates server-authoritative `PlayerSessions` from `SessionConfig` during both direct session-ready and room-session-ready promotion before `SessionReady` observers consume class identity.
- `tests/integration/playable_client/real_e2e_loop_test.rs` now extends the controlled two-primary-client Lightyear test through real `S2CDraftOffering`, real `C2SPurchaseCard`, authoritative `S2CCardAcquired`/`S2CGoldUpdate`, `C2SSignalReady` ready/retract/ready, real `C2SSubmitPlacement`, server `S2CPlacementReveal`/`S2CResolutionEvent`, and `S2CPhaseChanged(DraftShop)`.

## Capture Manifest

Capture directory: `production/qa/evidence/captures/playable-client-real-e2e-loop/`

- `attempt-1-*.stderr.log`: sanitized initial failure summaries for invalid client command and missing server `CardCatalog` startup resource.
- `attempt-2-*.stderr.log`: sanitized failure summaries for missing server `SessionConfig` startup resource and duplicate native client `StatesPlugin`.
- `server.stderr.log`, `client-a.stderr.log`, `client-b.stderr.log`: sanitized pre-repair live summaries showing native launch followed by `MissingComponent(ComponentId(320))` on both clients.
- `attempt-5-server.stderr.log`, `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log`: sanitized post-repair live summaries showing server and two native clients launched with no `MissingComponent` logged during the capture window.
- `attempt-*-live-process-summary.json`, `live-process-summary.json`: sanitized process summaries with pids and transient port redacted.
- `prompt-290-room-session-trace.json`: sanitized controlled Lightyear trace for host create-room, joiner join-room, class select/confirm, class reveal, and server-confirmed `DraftInitial` session entry.
- `prompt-296-draft-shop-trace.json`: sanitized controlled Lightyear trace for DraftInitial offering, purchase/acquisition/economy, ready/retract, placement submit, resolution, and server-confirmed `DraftShop`.
- `phase-captures.md`: reached and unreached phase notes.

## No Harness Statement

No direct production `World` injection or harness-injected game state was used for the live process attempts. The live attempts used one real local server process and two real primary native client processes launched from the worker checkout. The automated prompt 290 and prompt 296 room/session tests use in-process Lightyear server/client apps as controlled primary-client evidence: they drive real C2S messages through Lightyear `MessageSender`s and observe real S2C messages through Lightyear `MessageReceiver`s. They do not mutate authoritative client ECS state, do not inject fake S2C snapshots, and do not claim completion of the full manual two-client game loop.

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

Prompt 296 controlled DraftShop evidence:

1. Both clients received server-authored `S2CDraftOffering` with nine offered cards in `DraftInitial`.
2. Host and joiner each sent real `C2SPurchaseCard` using card IDs from their server offering.
3. The server accepted the purchases, updated authoritative `PlayerHands`, and sent `S2CCardAcquired { source: DraftInitial }` plus purchase `S2CGoldUpdate`.
4. Host sent `C2SSignalReady { retract: false }`, then `C2SSignalReady { retract: true }`; the server RSM observed the retract path before final readiness.
5. Joiner sent `C2SSignalReady { retract: false }`, then host sent final ready.
6. The server RSM's actual rule advanced `DraftInitial` to `Placement`, not directly to `DraftShop`.
7. Both clients received `S2CPhaseChanged(Placement)` and sent real empty `C2SSubmitPlacement` messages.
8. The server advanced through `Resolution`, emitted `S2CPlacementReveal` and `S2CResolutionEvent`, then progressed to `DraftShop`.
9. Both clients received `S2CPhaseChanged(DraftShop)`.

Room creation, join, class select/confirm, class reveal, server-confirmed session entry, DraftInitial offering, purchase, authoritative hand/economy update, ready/retract, placement submit, resolution, and server-confirmed `DRAFT_SHOP` are now verified by controlled real-Lightyear primary-client evidence. Auction, non-empty placement gameplay, next combat loop after `DRAFT_SHOP`, game-over, and full manual two-client friend-game completion remain unverified and are not claimed here.

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
| DRAFT_INITIAL | Reached by controlled real-Lightyear primary-client trace | `prompt-290-room-session-trace.json`, `prompt-296-draft-shop-trace.json` |
| DRAFT_SHOP | Reached by controlled real-Lightyear prompt 296 trace | `prompt-296-draft-shop-trace.json`; server route is `DRAFT_INITIAL -> PLACEMENT -> RESOLUTION -> DRAFT_SHOP` |
| Auction | Not reached in prompt 296 evidence | Auction remains outside this repair scope |
| Placement submit | Reached by controlled real-Lightyear prompt 296 trace with empty submissions | `prompt-296-draft-shop-trace.json` |
| Placement reveal | Reached by controlled real-Lightyear prompt 296 trace with empty reveal | `prompt-296-draft-shop-trace.json` |
| Resolution replay | Partially reached: server emitted `S2CResolutionEvent`; no manual visual replay validation claimed | `prompt-296-draft-shop-trace.json` |
| Next loop | Not reached beyond `DRAFT_SHOP` | Auction and next placement loop remain unverified |
| Game-over or nearest endpoint | Nearest endpoint documented | Controlled endpoint reaches `DRAFT_SHOP`; full loop not verified |

## Defects

| ID | Severity | Owner/System | Status | Friend-game Impact | Evidence |
|---|---|---|---|---|---|
| PLAYABLE-003-D1 | Blocker | Auction startup scheduling | Repaired in this branch | Server could not launch from `cargo run -p server` without loaded `CardCatalog` | `attempt-1-server.stderr.log` |
| PLAYABLE-003-D2 | Blocker | Board displacement scheduling | Repaired in this branch | Server could not remain alive before session setup | `attempt-2-server.stderr.log` |
| PLAYABLE-003-D3 | Blocker | Native client startup | Repaired in this branch | Primary native client could not launch | `attempt-2-client-a.stderr.log`, `attempt-2-client-b.stderr.log` |
| PLAYABLE-003-D4 | Blocker | Lightyear primary-client receive path | Repaired in this branch | Two real primary clients logged `MissingComponent(ComponentId(320))` before protocol feature alignment | `client-a.stderr.log`, `client-b.stderr.log`, `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log` |
| PLAYABLE-003-D5 | Blocker | PLAYABLE-003 automated coverage | Repaired in this branch | Story-declared test target was absent | `cargo test -p server --test playable_client_real_e2e_loop_test` |
| PLAYABLE-003-D6 | Major evidence gap | Real two-client room/session flow capture | Repaired for room/session scope | Host create, joiner join, class reveal, and session entry are now covered by controlled real-Lightyear evidence | `prompt-290-room-session-trace.json` |
| PLAYABLE-003-D7 | Major evidence gap | Full friend-game loop capture | Open | Auction, non-empty placement gameplay, next loop after `DRAFT_SHOP`, and game-over are still not proven from the full two-client path | Phase checklist |
| PLAYABLE-003-D8 | Major evidence gap | Room session to draft offering bridge | Repaired in prompt 296 | Room-backed session entry did not populate `PlayerSessions`, preventing pool/acquisition subscribers from emitting `S2CDraftOffering` | `server/src/core/session/system.rs`, `prompt-296-draft-shop-trace.json` |

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

Prompt 296 verification results:

- `cargo fmt -p client -p server -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test -p server --test playable_client_real_e2e_loop_test`: PASS, 4 passed.
- `cargo test -p client --test playable_client_lobby_entry_test`: PASS, 5 passed.
- `cargo test -p server --test playable_client_lobby_entry_server_test`: PASS, 3 passed.
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`: PASS, 4 passed.
- `cargo test -p server --test playable_client_draft_ready_bridge_test`: PASS, 3 passed.
- `cargo test -p server --test card_acquisition_draft_initial_test`: PASS, 5 passed.
- `cargo test -p server --test card_acquisition_purchase_atomicity_test`: PASS, 6 passed.
- `cargo test -p server --test room_create_join_test`: PASS, 7 passed.
- `cargo test -p server --test class_reveal_test`: PASS, 8 passed.
- `cargo test -p server --test class_lifecycle_test`: PASS, 9 passed.
- `cargo test -p server --test session_ready_test`: PASS, 4 passed.
- `cargo test -p server --test lobby_to_draft_initial_test`: PASS, 1 passed.
- `cargo test -p server --test rsm_timers_test`: PASS, 10 passed.
- `cargo test -p server --test rsm_transitions_test`: PASS, 14 passed.
- `cargo test -p server --test pool_session_ready_test`: PASS, 5 passed.
- `cargo test -p server --test economy_draft_subscriber_test`: PASS, 7 passed.
- `cargo test -p server --test rsm_network_dispatch_test`: PASS, 3 passed.
- `git diff --check origin/main...HEAD`: PASS.

Because the full real primary-client game loop remains unverified from live two-client evidence, this evidence still does not claim full friend-game loop completion.
