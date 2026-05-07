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

No browser/manual two-client smoke was run in this worker pass. S8-QA-001
remains blocked until the orchestrator pulls the manual friend-game smoke
package.

## Defects And Gaps

| ID | Severity | Owner/System | Status | Friend-game Impact | Workaround |
|---|---|---|---|---|---|
| PLAYABLE-004-D1 | Low evidence gap | Manual/browser evidence | Deferred | Automated in-process real-Lightyear endpoint is covered; browser/native manual captures are not part of this pass | Use S8-QA-001 for manual friend-game smoke package if requested |
| QA-COND-0005 | Accepted risk | Accessibility evidence | Still accepted risk | This story does not close broad Standard-tier accessibility evidence | Keep non-claim explicit |
| QA-COND-0006 | Accepted risk | Playtest validation | Still accepted risk | This story is not a playtest and does not validate fun hypothesis | Keep non-claim explicit |

## Verification Results

- `cargo test -p server --test playable_client_friend_game_result_endpoint_test`: PASS, 1 passed. Endpoint reached `GAME_OVER` through real C2S/S2C route.
- `cargo test -p client --test hud_game_over_freeze_test`: PASS, 2 passed.
  Captures available HUD frozen/result behavior for the same commit.
- `cargo test -p server --test playable_client_active_loop_polish_test`: PASS,
  4 passed.
- `cargo test -p client --test playable_client_active_loop_ui_state_test`: PASS,
  4 passed.
- `cargo test -p server --test playable_client_real_e2e_loop_test`: PASS,
  4 passed.
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
- `git diff --check origin/main...HEAD`: PASS.

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
