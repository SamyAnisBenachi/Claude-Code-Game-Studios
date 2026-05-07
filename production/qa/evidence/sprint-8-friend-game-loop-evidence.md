# Sprint 8 Friend-Game Loop Evidence

This is internal friend-game evidence only. It is not public release readiness,
not broad accessibility completion, not playtest validation, not fun-hypothesis
validation, not QA sign-off, not full playable-client manual QA, and not full
game completion.

## Version Control

- Worker branch: `work/playable-004-friend-game-result-endpoint-expansion`
- Integrated base during implementation: `origin/main@d5f50f9f85224477500f1427ac00b3ff23ab0530`
- Worker commit: recorded in the final orchestration handoff for this branch. The
  evidence document is committed with the worker branch; a Git commit cannot
  contain its own final hash.
- Scope: PLAYABLE-004 only. No `/story-done`, smoke, team-qa, gate-check, Sprint
  status close-out, ASSET-LOOP files, public-release claims, or implementation
  outside the friend-game endpoint test/evidence scope.
- LOOP-001 worker branch:
  `work/loop-001-draft-shop-auction-placement-resolution-loop-polish`
- LOOP-001 integrated base during implementation:
  `origin/main@4f85035a93297f2ee5e0c26e8f7a2b0abf488005`
- LOOP-001 worker commit: recorded in the final orchestration handoff for this
  branch. The evidence document is committed with the worker branch; a Git
  commit cannot contain its own final hash.
- LOOP-001 scope: active friend-game loop polish only. No `/story-done`, smoke,
  team-qa, gate-check, Sprint 8 close-out, QA-COND-0005/0006 disposition change,
  public-release claim, broad accessibility claim, playtest claim, or unrelated
  content/asset work.

## Runtime

- Environment type: controlled in-process real Lightyear server plus two primary
  client apps from the same worker tree.
- Target: local WebSocket route using one real server app and two real primary
  client apps.
- HUD target: same-commit client HUD ECS regression for `GAME_OVER` frozen
  state. The endpoint harness does not instantiate the full rendered HUD app, so
  browser HUD capture remains outside this automated pass.
- Local hardware, usernames, machine identifiers, process ids, raw room codes,
  transient ports, and unsafe branch-local metadata are redacted from committed
  evidence.
- Browser target: not exercised in this automated PLAYABLE-004 evidence pass.

## Capture Manifest

Capture directory:
`production/qa/evidence/captures/sprint-8-friend-game-loop/`

- `playable-004-result-endpoint-trace.json`: sanitized controlled real-Lightyear
  trace summary for the Sprint 8 endpoint expansion.
- `loop-001-active-loop-polish-trace.json`: sanitized controlled
  real-Lightyear and ECS regression summary for repeated active-loop stability.

## Supporting Content Addendum

CONTENT-001A was already integrated on main by merge
`d5f50f9f85224477500f1427ac00b3ff23ab0530` with worker commit
`eed165e4837a64ace95c144028c28aad24f7cfb3` as Sprint 8 supporting content
under CONTENT-001. It adds 8 runtime-valid Neutral cards, from `CardId(101)`
through `CardId(108)`, and
`server/tests/content_runtime_card_variety_floor_test.rs`.

This note only reconciles the runtime card variety floor support slice; it does
not claim full card production, full balance completion, public release
readiness, QA sign-off, playtest validation, or fun-hypothesis validation.

## Reached Endpoint

The automated endpoint test reached actual game-over/result coverage.

Exact reached route:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(endpoint) -> RESOLUTION -> GAME_OVER`.

Observed result endpoint:

- Both clients reproduced the Sprint 7 next-loop `DRAFT_SHOP` endpoint.
- Both clients extended beyond that endpoint with real `C2SSignalReady` and real
  `C2SSubmitPlacement` traffic.
- Both clients observed `S2CResolutionEvent` before terminal phase change.
- Both clients observed `ResolutionEvent::ObjectiveDestroyed`.
- Both clients observed authoritative `S2CGameOver`.
- Both clients observed `S2CPhaseChanged(GameOver)` after the resolution event.
- The `S2CGameOver` payload was a draw: `loser = None`, `reason = Draw`.
- HUD frozen/result state is available through the same-commit client HUD
  regression: `HudMode::Frozen`, phase label `GAME OVER`, round counter retained,
  final gold text retained, objective dot state frozen against post-game updates,
  and active gold tween removed on snap rerender.
- Server `ObjectiveCounters` reached at least two real objectives destroyed for
  both players.
- GSS teardown was observed: `RoundState::GameOver`, `LobbyState::GameOver`,
  `SessionConfig` removed, and `ServerRng` removed.

## No-Harness Statement

Completion proof does not use direct `World` injection, fake snapshot insertion,
harness card state, or direct server feature API calls as the endpoint proof.
The route uses a real local Lightyear server app and two real primary client apps
communicating through C2S/S2C messages. The deterministic catalog/config/RNG
fixtures are used to make the real friend-game route repeatable; they do not
short-circuit game-over or mutate result state directly.

## Ordering Observations

- `S2CPlacementReveal` is observed before the following resolution phase path.
- `S2CResolutionEvent` is observed before `S2CPhaseChanged(GameOver)`.
- The terminal phase is server-authored; clients do not locally force the result.

## LOOP-001 Active Loop Polish Addendum

LOOP-001 stabilized the active friend-game loop around stale UI state and
phase-boundary cleanup. Dedicated capture:
`production/qa/evidence/captures/sprint-8-friend-game-loop/loop-001-active-loop-polish-trace.json`.

The automated real-Lightyear route remains:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

Stabilized behavior:

- Repeated authoritative `DRAFT_SHOP` phase messages reset local ready/retract
  state, wait for fresh shop slots, and restart the server-provided shop timer.
- Auction feedback clears on phase boundaries: rejected-bid toasts, accepted-bid
  timer targets, keyboard focus, in-flight bid state, and old card/leader state
  do not leak into placement or the next auction.
- Late settlement messages after shop convergence do not resurrect auction UI,
  restart settlement overlays, or disturb the active shop timer/slots.
- Placement exit clears pending hand placements, submit validation markers,
  active drag state, submitted/grace flags, urgency state, and the visible hand
  timer before RESOLUTION.
- SAU-007 remains the owner of settlement presentation and card-acquisition
  feedback. LOOP-001 only suppresses stale/late settlement start after auction
  context is gone.
- `UnitPlaced` replay remains covered by the real-Lightyear route and by the
  existing PLAYABLE-004 result endpoint route.

No browser/manual two-client smoke was run in the PLAYABLE-004 or LOOP-001
worker passes. The S8-QA-001 scoped smoke package below records the current
manual friend-game smoke evidence status.

## S8-QA-001 Manual Friend-Game Smoke Package Addendum

S8-QA-001 was run as a scoped smoke/evidence package on
`main@3cc620cdeee6f5249e404703365b160ccbc34f6c`.

This package did not run `/dev-story`, `/story-done`, `/team-qa`,
`/gate-check`, Sprint 8 close-out, or new implementation. It only reviewed the
Sprint 8 plan/status/story context, reran the required smoke command set, and
packaged the manual friend-game evidence status.

Capture summary:
`production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-manual-smoke-summary.json`.

Command summary:
`production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-command-summary.md`.

Bounded server process note:
`production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-server-process.txt`.

Nearest host/client capture notes:

- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-client-a-log.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-client-b-log.md`

Manual execution status:

- Full native/browser two-client manual execution was not completed in this
  Codex shell session.
- Blocker: this workflow can run commands and inspect artifacts, but it cannot
  drive two interactive Bevy native client windows or browser clients through
  room creation, join, class confirmation, draft/shop, auction, placement,
  resolution, and result steps. Launching native clients here would require GUI
  interaction and would not produce a completed manual route log from this
  non-interactive workflow.
- A bounded server process note was captured. Client A and Client B capture
  notes were added from the nearest covered real-Lightyear host/joiner traces,
  but no new full manual browser/native route logs were captured.
- Trunk is installed (`trunk 0.21.14`), `client/index.html` exists, the default
  server port remains `5000`, and the manual command shape remains:
  `SERVER_PORT=<PORT> cargo run -p server`;
  `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client` for both
  primary clients.

Nearest covered evidence:

- `playable-004-result-endpoint-trace.json` covers one real local Lightyear
  server app plus two real primary client apps in-process reaching
  `GAME_OVER`.
- `loop-001-active-loop-polish-trace.json` covers repeated active-loop stability
  and UI stale-state cleanup.
- `production/qa/evidence/captures/playable-client-real-e2e-loop/phase-captures.md`
  records prior native launch attempts and the controlled real-Lightyear route
  baseline.

Manual/evidence checklist status:

| Checklist Item | Status | Evidence |
|---|---|---|
| Server log | WARN | Bounded server process note only; no full manual two-client server route log captured. |
| Client A log | WARN | Host/client A capture note added from nearest real-Lightyear trace; no new full manual browser/native client A log captured. |
| Client B log | WARN | Joiner/client B capture note added from nearest real-Lightyear trace; no new full manual browser/native client B log captured. |
| Commands / port / commit / target summary | PASS | This addendum, `s8-qa-001-manual-smoke-summary.json`, and `s8-qa-001-command-summary.md`. |
| Lobby create/join | PASS | Controlled real-Lightyear trace records `C2SCreateRoom`, `S2CRoomCreated`, `C2SJoinRoom`, and `S2CJoinAck`. |
| Class confirm | PASS | Controlled real-Lightyear trace records `C2SSelectClass`, `C2SConfirmClass`, `S2CClassLocked`, and `S2CClassesRevealed`. |
| DRAFT_INITIAL | PASS | Controlled real-Lightyear trace records `S2CPhaseChanged(DraftInitial)` and `S2CDraftOffering`. |
| DRAFT_SHOP | PASS | Controlled real-Lightyear traces reproduce Sprint 7 next-loop `DRAFT_SHOP` and later passes. |
| Auction | PASS | Controlled trace records `DRAFT_AUCTION`, `S2CAuctionCard`, `C2SPlaceBid`, `S2CAuctionBidAccepted`, `S2CAuctionSettled`, and auction acquisition. |
| Settlement-to-shop | PASS | SAU-007 settlement regression passed; LOOP-001 trace records stale settlement suppression after shop convergence. |
| Post-auction DRAFT_SHOP | PASS | Controlled trace records post-auction `DRAFT_SHOP`. |
| Non-empty placement | PASS | Controlled trace records real `C2SSubmitPlacement` with server-owned hand cards. |
| Resolution `UnitPlaced` | PASS | Controlled trace records `S2CResolutionEvent` containing `UnitPlaced` before following phase changes. |
| Second post-endpoint loop pass | PASS | PLAYABLE-004 endpoint trace plus LOOP-001 repeated-loop trace cover continued route beyond Sprint 7 endpoint. |
| GAME_OVER | PASS WITH WARNING | Automated real-Lightyear endpoint reaches `GAME_OVER`; manual/browser game-over is not claimed. |
| Defect table | PASS | See Defects And Gaps below. |

S8-QA-001 verdict: **PASS WITH WARNINGS**. All required commands passed and the
scoped route is evidenced through controlled real-Lightyear artifacts, but the
new full manual two-window/browser route log remains a bounded evidence gap.

## Defects And Gaps

| ID | Severity | Owner/System | Status | Friend-game Impact | Workaround |
|---|---|---|---|---|---|
| S8-QA-001-W1 | Low evidence gap | Manual/browser smoke workflow | Bounded warning | Core route is covered by controlled real-Lightyear tests and traces, but no new manually driven two-window or browser route log was captured in this session | Use the committed controlled traces for S8 smoke; run an out-of-band interactive two-client session later if full manual client QA is required |
| S8-QA-001-W2 | Low tooling ambiguity | Smoke command list | Recorded | Prompt listed `cargo test -p client --test`, which is incomplete without a test target | Interpreted as `cargo test -p client --tests`; valid equivalent passed |
| PLAYABLE-004-D1 | Low evidence gap | Manual/browser evidence | Superseded by S8-QA-001-W1 | Automated in-process real-Lightyear endpoint is covered; browser/native manual captures remain bounded by S8-QA-001-W1 | Keep the bounded warning explicit |
| QA-COND-0005 | Accepted risk | Accessibility evidence | Still accepted risk | This story does not close broad Standard-tier accessibility evidence | Keep non-claim explicit |
| QA-COND-0006 | Accepted risk | Playtest validation | Still accepted risk | This story is not a playtest and does not validate fun hypothesis | Keep non-claim explicit |

## Verification Results

- S8-QA-001 command gate on `main@3cc620cdeee6f5249e404703365b160ccbc34f6c`:
  PASS WITH WARNINGS.
- `cargo test -p server --test playable_client_friend_game_result_endpoint_test`: PASS, 1 passed. Endpoint reached `GAME_OVER` through real C2S/S2C route.
- `cargo test -p client --test hud_game_over_freeze_test`: PASS, 2 passed.
  Captures available HUD frozen/result behavior for the same commit.
- `cargo test -p server --test playable_client_active_loop_polish_test`: PASS,
  4 passed.
- `cargo test -p client --test playable_client_active_loop_ui_state_test`: PASS,
  4 passed.
- `cargo test -p server --test playable_client_real_e2e_loop_test`: PASS,
  4 passed.
- `cargo test -p client --tests`: PASS. This was used as the valid equivalent
  for the S8-QA-001 prompt's incomplete `cargo test -p client --test` command;
  `cargo test -p client --tests -- --list` reports 292 client tests.
- `cargo test -p client --test shop_auction_ui_auction_settlement_test`: PASS,
  7 passed.
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`:
  PASS, 4 passed.
- `cargo test -p client --test hand_ui_placement_timer_test`: PASS, 5 passed.
- `cargo test -p client --test hand_ui_phase_state_machine_test`: PASS,
  3 passed.
- `cargo test -p client --test shop_auction_ui_shop_panel_test`: PASS, 9 passed.
- `cargo test -p client --test shop_auction_ui_auction_feedback_test`: PASS,
  6 passed.
- `cargo test -p client --test shop_auction_ui_auction_activation_test`: PASS,
  7 passed.
- `cargo test -p client --test board_rendering_resolution_anim_queue_test`:
  PASS, 5 passed.
- `cargo test -p client --test hud_phase_transitions_test`: PASS, 5 passed.
- `cargo check --workspace`: PASS.
- `cargo fmt -p client -p server -- --check`: PASS.
- `git diff --check`: PASS.

## Non-Claims

- No public release readiness.
- No store readiness.
- No deployment readiness.
- No release-candidate readiness.
- No broad accessibility completion.
- No closure of QA-COND-0005.
- No playtest validation.
- No fun-hypothesis validation.
- No closure of QA-COND-0006.
- No full playable-client manual QA.
- No full regression campaign.
- No full game completion.
