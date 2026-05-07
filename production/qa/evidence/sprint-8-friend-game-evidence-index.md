# Sprint 8 Friend-Game Evidence Index

Date: 2026-05-07
Source: current `main` at `efbc9e4ed58128f385fb8bceea18302a4c830c17`
Scope: Sprint 8 internal friend-game evidence indexing only

## Summary

Sprint 8 internal friend-game evidence is indexed here for audit convenience.
This document does not create new QA sign-off, does not run Sprint close-out,
and does not change `production/sprint-status.yaml` or
`production/session-state/active.md`.

S8-QA-001 records **PASS WITH WARNINGS**. The automated controlled
real-Lightyear evidence covers the active loop and the `GAME_OVER` result
endpoint, but a full manually driven two-window or browser two-client
`GAME_OVER` route was not captured. Manual/browser game-over is not claimed.

PLAYABLE-004 reached this endpoint route:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(endpoint) -> RESOLUTION -> GAME_OVER`.

LOOP-001 active-loop evidence covers:

`DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

## Evidence Documents

| Item | Status In Current Main | Evidence |
|---|---|---|
| S8-QA-001 Friend-Game Manual Smoke Expansion Package | PASS WITH WARNINGS | [sprint-8-friend-game-loop-evidence.md](sprint-8-friend-game-loop-evidence.md); [../smoke-sprint-8-2026-05-07.md](../smoke-sprint-8-2026-05-07.md) |
| PLAYABLE-004 Friend-Game Result Endpoint Expansion | Complete | [sprint-8-friend-game-loop-evidence.md](sprint-8-friend-game-loop-evidence.md); [../../epics/playable-client/story-004-friend-game-result-endpoint-expansion.md](../../epics/playable-client/story-004-friend-game-result-endpoint-expansion.md) |
| LOOP-001 DRAFT_SHOP / Auction / Placement / Resolution Loop Polish | Complete | [sprint-8-friend-game-loop-evidence.md](sprint-8-friend-game-loop-evidence.md); [../../epics/playable-client/story-005-draft-shop-auction-placement-resolution-loop-polish.md](../../epics/playable-client/story-005-draft-shop-auction-placement-resolution-loop-polish.md) |
| SAU-007 Auction Settlement and Shop Transition | Complete | [shop-auction-ui-settlement-transition-evidence.md](shop-auction-ui-settlement-transition-evidence.md); [../../epics/shop-auction-ui/story-007-auction-settlement-and-shop-transition.md](../../epics/shop-auction-ui/story-007-auction-settlement-and-shop-transition.md) |
| CONTENT-001A supporting content slice | Supporting only | Current main contains the runtime card variety floor in [../../../assets/data/cards.json](../../../assets/data/cards.json) and [../../../server/tests/content_runtime_card_variety_floor_test.rs](../../../server/tests/content_runtime_card_variety_floor_test.rs); no standalone story file or full content-production claim is made. |
| ASSET-LOOP-001 supporting asset wiring | Supporting only | Current main contains active-loop display fallback coverage in [../../../tests/integration/hand-ui/draft_initial_grid_test.rs](../../../tests/integration/hand-ui/draft_initial_grid_test.rs), [../../../tests/integration/shop_auction_ui/shop_panel_test.rs](../../../tests/integration/shop_auction_ui/shop_panel_test.rs), and [../../../tests/integration/shop_auction_ui/auction_activation_test.rs](../../../tests/integration/shop_auction_ui/auction_activation_test.rs); reconciled under LOOP-001 with no standalone asset approval or asset-production claim. |
| QA-COND-0005 Standard-tier accessibility gaps | Carried accepted risk | [../bugs/QA-COND-0005-standard-tier-accessibility-gaps.md](../bugs/QA-COND-0005-standard-tier-accessibility-gaps.md) |
| QA-COND-0006 Playtest/fun-hypothesis evidence | Carried accepted risk/deferred | [../bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md](../bugs/QA-COND-0006-playtest-fun-hypothesis-evidence.md) |

## Capture Directories And Key Artifacts

Primary Sprint 8 capture directory:

- [captures/sprint-8-friend-game-loop/](captures/sprint-8-friend-game-loop/)

Key Sprint 8 capture artifacts:

- [playable-004-result-endpoint-trace.json](captures/sprint-8-friend-game-loop/playable-004-result-endpoint-trace.json) - controlled real-Lightyear two-primary-client trace reaching `GAME_OVER`.
- [loop-001-active-loop-polish-trace.json](captures/sprint-8-friend-game-loop/loop-001-active-loop-polish-trace.json) - controlled active-loop polish trace for repeated loop stability and stale-state cleanup.
- [s8-qa-001-manual-smoke-summary.json](captures/sprint-8-friend-game-loop/s8-qa-001-manual-smoke-summary.json) - S8-QA-001 scoped smoke package summary.
- [s8-qa-001-command-summary.md](captures/sprint-8-friend-game-loop/s8-qa-001-command-summary.md) - command summary for the S8-QA-001 package.
- [s8-qa-001-server-process.txt](captures/sprint-8-friend-game-loop/s8-qa-001-server-process.txt) - bounded server process note.
- [s8-qa-001-client-a-log.md](captures/sprint-8-friend-game-loop/s8-qa-001-client-a-log.md) - nearest host/client A capture note from covered real-Lightyear traces.
- [s8-qa-001-client-b-log.md](captures/sprint-8-friend-game-loop/s8-qa-001-client-b-log.md) - nearest joiner/client B capture note from covered real-Lightyear traces.

Supporting prior friend-game baseline:

- [sprint-7-friend-game-evidence-index.md](sprint-7-friend-game-evidence-index.md)
- [playable-client-real-e2e-loop.md](playable-client-real-e2e-loop.md)
- [captures/playable-client-real-e2e-loop/](captures/playable-client-real-e2e-loop/)

## Bounded Warnings

- S8-QA-001-W1 remains a bounded manual/browser smoke evidence gap: no new full
  manually driven two-window or browser route log was captured for the complete
  room creation, join, class confirmation, draft/shop, auction, placement,
  resolution, and `GAME_OVER` route.
- S8-QA-001-W2 records the smoke command ambiguity: the prompt listed
  `cargo test -p client --test` without a test target, and the scoped package
  used `cargo test -p client --tests` as the valid equivalent.
- Browser HUD capture for PLAYABLE-004 was not run and is not claimed.
- CONTENT-001A remains supporting content context only; the committed runtime
  card variety floor is not a full card-production or balance-completion claim.
- ASSET-LOOP-001 remains supporting display fallback coverage under LOOP-001
  only; no asset approval, asset production completion, or standalone asset
  sign-off is claimed.

## Carried Conditions

QA-COND-0005 remains accepted risk for friend-game scope only. Sprint 8 evidence
does not verify Standard-tier accessibility completion and does not close the
remaining accessibility debt for any public, external, commercial, or broader
release-candidate scope.

QA-COND-0006 remains accepted-risk/deferred. Sprint 8 friend-game evidence and
the S8-QA-001 smoke package are not playtest evidence, do not validate or revise
the fun hypothesis, and do not close the future playtest evidence requirement.

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
- No asset production approval.
- No full game completion.
