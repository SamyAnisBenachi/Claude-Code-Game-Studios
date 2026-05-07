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

## Runtime

- Environment type: controlled in-process real Lightyear server plus two primary
  client apps from the same worker tree.
- Target: local WebSocket route using one real server app and two real primary
  client apps.
- Local hardware, usernames, machine identifiers, process ids, raw room codes,
  transient ports, and unsafe branch-local metadata are redacted from committed
  evidence.
- Browser target: not exercised in this automated PLAYABLE-004 evidence pass.

## Capture Manifest

Capture directory:
`production/qa/evidence/captures/sprint-8-friend-game-loop/`

- `playable-004-result-endpoint-trace.json`: sanitized controlled real-Lightyear
  trace summary for the Sprint 8 endpoint expansion.

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

## Defects And Gaps

| ID | Severity | Owner/System | Status | Friend-game Impact | Workaround |
|---|---|---|---|---|---|
| PLAYABLE-004-D1 | Low evidence gap | Manual/browser evidence | Deferred | Automated in-process real-Lightyear endpoint is covered; browser/native manual captures are not part of this pass | Use S8-QA-001 for manual friend-game smoke package if requested |
| QA-COND-0005 | Accepted risk | Accessibility evidence | Still accepted risk | This story does not close broad Standard-tier accessibility evidence | Keep non-claim explicit |
| QA-COND-0006 | Accepted risk | Playtest validation | Still accepted risk | This story is not a playtest and does not validate fun hypothesis | Keep non-claim explicit |

## Verification Results

- `cargo test -p server --test playable_client_friend_game_result_endpoint_test`: PASS, 1 passed. Endpoint reached `GAME_OVER` through real C2S/S2C route.

Additional required regression commands are recorded in the final branch handoff
for this implementation pass.

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
