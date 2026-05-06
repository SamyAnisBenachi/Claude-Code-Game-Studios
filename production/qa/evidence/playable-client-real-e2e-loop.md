# PLAYABLE-003 Real End-to-End Loop Evidence

This is internal friend-game evidence only. It is not public release readiness, not broad accessibility completion, not playtest validation, not fun-hypothesis validation, not QA sign-off, and not full playable-client manual QA.

## Version Control

- Worker branch: `work/playable-003-real-e2e-sanitized`
- Base commit at story start: `38ca69c7b6e43d821653df1f79d30715077fbc5f`
- Sanitized replacement history: created from `origin/main` and imported only final sanitized file contents, without carrying earlier local capture-history commits.
- Repair/evidence commit: recorded in the worker response after commit and push.
- Worker dirty status during post-repair capture: repair files plus sanitized evidence changes.

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

## Capture Manifest

Capture directory: `production/qa/evidence/captures/playable-client-real-e2e-loop/`

- `attempt-1-*.stderr.log`: sanitized initial failure summaries for invalid client command and missing server `CardCatalog` startup resource.
- `attempt-2-*.stderr.log`: sanitized failure summaries for missing server `SessionConfig` startup resource and duplicate native client `StatesPlugin`.
- `server.stderr.log`, `client-a.stderr.log`, `client-b.stderr.log`: sanitized pre-repair live summaries showing native launch followed by `MissingComponent(ComponentId(320))` on both clients.
- `attempt-5-server.stderr.log`, `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log`: sanitized post-repair live summaries showing server and two native clients launched with no `MissingComponent` logged during the capture window.
- `attempt-*-live-process-summary.json`, `live-process-summary.json`: sanitized process summaries with pids and transient port redacted.
- `phase-captures.md`: reached and unreached phase notes.

## No Harness Statement

No direct production `World` injection or harness-injected game state was used for the live process attempts. The live attempts used one real local server process and two real primary native client processes launched from the worker checkout. The added automated smoke test uses in-process Lightyear apps only to guard launch/protocol reachability and does not claim completion of the real manual two-client path.

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

Fresh hello is therefore covered by the new smoke test, but create room, join room, class select/confirm, server-confirmed session entry, `DRAFT_INITIAL`, `DRAFT_SHOP`, auction, placement submit, placement reveal, resolution replay, next loop, and game-over remain unverified from the real primary-client live process path.

## Phase Checklist

| Phase | Result | Evidence |
|---|---|---|
| Server launch | Reached after repair | `attempt-5-server.stderr.log`, `attempt-5-live-process-summary.json` |
| Client A launch | Reached after repair | `attempt-5-client-a.stderr.log`, `attempt-5-live-process-summary.json` |
| Client B launch | Reached after repair | `attempt-5-client-b.stderr.log`, `attempt-5-live-process-summary.json` |
| Fresh hello | Reached by automated real-Lightyear smoke; not directly logged by live native clients | `playable_client_real_e2e_loop_test` |
| Create room | Not reached in live process evidence | Requires controlled two-client input/capture driver or manual evidence |
| Join room | Not reached in live process evidence | Requires controlled two-client input/capture driver or manual evidence |
| Class select/confirm | Not reached in live process evidence | Requires controlled two-client input/capture driver or manual evidence |
| Server-confirmed session entry | Not reached in live process evidence | Blocked by missing controlled real-client flow evidence |
| DRAFT_INITIAL | Not reached in live process evidence | Blocked before session entry |
| DRAFT_SHOP | Not reached in live process evidence | Blocked before session entry |
| Auction | Not reached in live process evidence | Blocked before session entry |
| Placement submit | Not reached in live process evidence | Blocked before session entry |
| Placement reveal | Not reached in live process evidence | Blocked before session entry |
| Resolution replay | Not reached in live process evidence | Blocked before session entry |
| Next loop | Not reached in live process evidence | Blocked before session entry |
| Game-over or nearest endpoint | Nearest endpoint documented | Native clients launch with MissingComponent repaired; full loop not verified |

## Defects

| ID | Severity | Owner/System | Status | Friend-game Impact | Evidence |
|---|---|---|---|---|---|
| PLAYABLE-003-D1 | Blocker | Auction startup scheduling | Repaired in this branch | Server could not launch from `cargo run -p server` without loaded `CardCatalog` | `attempt-1-server.stderr.log` |
| PLAYABLE-003-D2 | Blocker | Board displacement scheduling | Repaired in this branch | Server could not remain alive before session setup | `attempt-2-server.stderr.log` |
| PLAYABLE-003-D3 | Blocker | Native client startup | Repaired in this branch | Primary native client could not launch | `attempt-2-client-a.stderr.log`, `attempt-2-client-b.stderr.log` |
| PLAYABLE-003-D4 | Blocker | Lightyear primary-client receive path | Repaired in this branch | Two real primary clients logged `MissingComponent(ComponentId(320))` before protocol feature alignment | `client-a.stderr.log`, `client-b.stderr.log`, `attempt-5-client-a.stderr.log`, `attempt-5-client-b.stderr.log` |
| PLAYABLE-003-D5 | Blocker | PLAYABLE-003 automated coverage | Repaired in this branch | Story-declared test target was absent | `cargo test -p server --test playable_client_real_e2e_loop_test` |
| PLAYABLE-003-D6 | Major evidence gap | Real two-client flow capture | Open | Full friend-game path is still not proven from two real primary clients | Phase checklist |

## Verification Results

Pre-repair verification passed for the previously listed PLAYABLE-003 regressions, but the story-declared E2E test target was absent and the live client logs found `MissingComponent(ComponentId(320))`.

Post-repair verification results:

- `cargo fmt -p client -p server -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test -p client --test playable_client_lobby_entry_test`: PASS, 5 passed.
- `cargo test -p server --test playable_client_lobby_entry_server_test`: PASS, 3 passed.
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`: PASS, 4 passed.
- `cargo test -p server --test playable_client_draft_ready_bridge_test`: PASS, 3 passed.
- `cargo test -p server --test e2e_websocket_test`: PASS, 1 passed.
- `cargo test -p server --test auction_fifo_ordering_test`: PASS, 2 passed.
- `cargo test -p server --test os18b_two_client_objective_hp_visibility_test`: PASS, 1 passed.
- `cargo test -p server --test playable_client_real_e2e_loop_test`: PASS, 2 passed.
- `git diff --check origin/main...HEAD`: PASS.

Because the full real primary-client game loop remains unverified from live two-client evidence, this story is still not complete for friend-game scope.
