# Smoke Check Report: Sprint 8 Manual Friend-Game Package

**Date**: 2026-05-07
**Sprint**: Sprint 8
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-8-2026-05-07.md`
**Argument**: scoped S8-QA-001 manual friend-game smoke package
**Commit Under Smoke**: `3cc620cdeee6f5249e404703365b160ccbc34f6c`
**Smoke Environment**: local Windows Cargo with documented no-PDB settings
`CARGO_PROFILE_DEV_DEBUG=0` and `RUSTFLAGS=-C link-arg=/DEBUG:NONE`

---

## Verdict: PASS WITH WARNINGS

All required pre-smoke commands passed. The scoped internal friend-game route is
covered by controlled real-Lightyear evidence using one real server app and two
primary client apps in-process, including repeated loop coverage and
`GAME_OVER` endpoint coverage.

The verdict remains **PASS WITH WARNINGS** because a new fully manual
two-window or browser client run was not completed in this non-interactive
Codex shell session. Manual/browser game-over is not claimed.

This report does not run or claim `/dev-story`, `/story-done`, `/team-qa`,
`/gate-check`, Sprint 8 close-out, public release readiness, broad accessibility
completion, playtest/fun-hypothesis validation, full playable-client manual QA,
asset production approval, or full game completion.

---

## Environment

- Root branch before docs: `main`, aligned with `origin/main`.
- Test directory: found at `tests/`.
- CI configured: yes, `.github/workflows/tests.yml` exists and references Cargo
  test/check flows.
- Trunk: `trunk 0.21.14` found at `C:\Users\Sam\.cargo\bin\trunk.exe`.
- Native server command shape: `SERVER_PORT=<PORT> cargo run -p server`.
- Native client command shape:
  `SERVER_URL=ws://localhost:<PORT> cargo run -p client --bin client` for both
  primary clients.
- Default server port: `5000`; default client URL: `ws://localhost:5000`.

Required context read:

- `production/sprints/sprint-8.md`
- `production/sprint-status.yaml`
- `production/qa/qa-plan-sprint-8-2026-05-07.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/epics/playable-client/story-004-friend-game-result-endpoint-expansion.md`
- `production/epics/playable-client/story-005-draft-shop-auction-placement-resolution-loop-polish.md`

---

## Automated Pre-Smoke Commands

**Status**: PASS

Summary: required focused smoke targets passed, the valid all-client-tests
equivalent passed, workspace check passed, and diff check passed.

| Command | Result |
|---|---|
| `git rev-parse HEAD` | PASS, `3cc620cdeee6f5249e404703365b160ccbc34f6c` |
| `git status --short --branch` | PASS, `## main...origin/main` |
| `cargo test -p server --test playable_client_active_loop_polish_test` | PASS, 4 passed |
| `cargo test -p client --test playable_client_active_loop_ui_state_test` | PASS, 4 passed |
| `cargo test -p server --test playable_client_friend_game_result_endpoint_test` | PASS, 1 passed |
| `cargo test -p server --test playable_client_real_e2e_loop_test` | PASS, 4 passed |
| `cargo test -p client --tests` | PASS; used as the valid equivalent for the prompt's incomplete `cargo test -p client --test`; `--list` reports 292 client tests |
| `cargo check --workspace` | PASS |
| `git diff --check` | PASS |

Failed checks:

- None.

---

## Manual Smoke Checklist

| Checklist Item | Status | Evidence |
|---|---|---|
| Server log | WARN | Bounded server process note only; no full manual two-client server route log captured. |
| Client A log | WARN | Host/client A capture note added from nearest real-Lightyear trace; no new full manual browser/native client A log captured. |
| Client B log | WARN | Joiner/client B capture note added from nearest real-Lightyear trace; no new full manual browser/native client B log captured. |
| Commands / port / commit / target summary | PASS | This report, `s8-qa-001-manual-smoke-summary.json`, and `s8-qa-001-command-summary.md`. |
| Lobby create/join | PASS | `playable-004-result-endpoint-trace.json` records real C2S/S2C room flow. |
| Class confirm | PASS | `playable-004-result-endpoint-trace.json` records class select/confirm and class reveal. |
| DRAFT_INITIAL | PASS | `playable-004-result-endpoint-trace.json` records `S2CPhaseChanged(DraftInitial)` and `S2CDraftOffering`. |
| DRAFT_SHOP | PASS | `playable-004-result-endpoint-trace.json` and `loop-001-active-loop-polish-trace.json`. |
| Auction | PASS | `playable-004-result-endpoint-trace.json` records auction card, bid, accepted, settled, and acquisition. |
| Settlement-to-shop | PASS | `shop_auction_ui_auction_settlement_test` and LOOP-001 trace. |
| Post-auction DRAFT_SHOP | PASS | `playable-004-result-endpoint-trace.json`. |
| Non-empty placement | PASS | `playable-004-result-endpoint-trace.json` records real non-empty submit payloads. |
| Resolution `UnitPlaced` | PASS | `playable-004-result-endpoint-trace.json` and LOOP-001 trace. |
| Second post-endpoint loop pass | PASS | PLAYABLE-004 endpoint trace plus LOOP-001 repeated-loop trace. |
| GAME_OVER | PASS WITH WARNING | Automated real-Lightyear endpoint reaches `GAME_OVER`; manual/browser game-over is not claimed. |
| Defect table | PASS | See Warnings And Conditions. |

Manual execution blocker:

This shell session can run commands and inspect artifacts, but it cannot drive
two interactive Bevy native client windows or browser clients through room
creation, join, class confirmation, draft/shop, auction, placement, resolution,
and result steps. Launching native clients here would require GUI interaction
and would not produce a completed manual route log from this non-interactive
workflow.

---

## Evidence Artifacts

- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-manual-smoke-summary.json`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-command-summary.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-server-process.txt`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-client-a-log.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-client-b-log.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/playable-004-result-endpoint-trace.json`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/loop-001-active-loop-polish-trace.json`
- `production/qa/evidence/captures/playable-client-real-e2e-loop/phase-captures.md`

Reached route from PLAYABLE-004 automated endpoint evidence:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(endpoint) -> RESOLUTION -> GAME_OVER`.

LOOP-001 repeated-loop evidence remains:

`DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

---

## Warnings And Conditions

| ID | Severity | Owner/System | Status | Friend-game Impact | Workaround |
|---|---|---|---|---|---|
| S8-QA-001-W1 | Low evidence gap | Manual/browser smoke workflow | Bounded warning | Core route is covered by controlled real-Lightyear tests and traces, but no new manually driven two-window or browser route log was captured in this session. | Use the committed controlled traces for S8 smoke; run an out-of-band interactive two-client session later if full manual client QA is required. |
| S8-QA-001-W2 | Low tooling ambiguity | Smoke command list | Recorded | Prompt listed `cargo test -p client --test`, which is incomplete without a test target. | Interpreted as `cargo test -p client --tests`; valid equivalent passed. |
| QA-COND-0005 | Accepted risk | Accessibility evidence | Still accepted risk for friend-game scope only | This smoke does not verify broad Standard-tier accessibility completion. | Keep non-claim explicit until a separate accessibility scope verifies it. |
| QA-COND-0006 | Accepted risk | Playtest validation | Still accepted-risk/deferred | This smoke is not playtest evidence and does not validate the fun hypothesis. | Keep non-claim explicit until separate production playtests exist. |

---

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
- No asset production approval.

---

## Changed Files

- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-manual-smoke-summary.json`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-command-summary.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-server-process.txt`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-client-a-log.md`
- `production/qa/evidence/captures/sprint-8-friend-game-loop/s8-qa-001-client-b-log.md`
- `production/qa/smoke-sprint-8-2026-05-07.md`

No source code, `production/sprint-status.yaml`, `/story-done` records,
QA sign-off, gate-check reports, team-qa reports, Sprint 8 close-out, or asset
approval files were changed.
