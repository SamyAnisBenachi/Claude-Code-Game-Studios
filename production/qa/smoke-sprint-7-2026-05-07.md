# Smoke Check Report: Sprint 7

**Date**: 2026-05-07
**Sprint**: Sprint 7
**Engine**: Bevy 0.18 + Lightyear 0.26
**QA Plan**: `production/qa/qa-plan-sprint-7-2026-05-06.md`
**Argument**: `sprint`
**Commit Under Smoke**: `a5ce9d490caf3d4621c3569f11e8fe958a533b60`
**Smoke Environment**: local Windows Cargo with documented no-PDB settings
`CARGO_PROFILE_DEV_DEBUG=0` and `RUSTFLAGS=-C link-arg=/DEBUG:NONE`

---

## Verdict: PASS WITH WARNINGS

Sprint 7 smoke passes on clean `origin/main` at
`a5ce9d490caf3d4621c3569f11e8fe958a533b60`.

All required automated smoke commands passed. The verdict remains **PASS WITH
WARNINGS** because this is friend-game smoke only and the carried Sprint 7 scope
warnings remain in force.

This report does not run or claim `/team-qa`, `/gate-check`, Sprint 8 planning,
`/story-done`, public release readiness, broad accessibility completion,
playtest/fun-hypothesis validation, full playable-client manual QA, game-over
coverage, or full game completion.

---

## Environment

- Root branch: `main`, aligned with `origin/main`.
- Test directory: found at `tests/`.
- CI configured: yes, `.github/workflows/tests.yml` references Cargo test and
  check commands.
- Smoke test source: `production/qa/qa-plan-sprint-7-2026-05-06.md` plus
  `tests/smoke/critical-paths.md`.
- Required context read:
  - `production/sprints/sprint-7.md`
  - `production/sprint-status.yaml`
  - `production/qa/qa-plan-sprint-7-2026-05-06.md`
  - `production/qa/evidence/sprint-7-friend-game-evidence-index.md`
  - `production/qa/evidence/playable-client-real-e2e-loop.md`

Note: the final prompt bullet `cargo test -p server --test` was missing a test
target name. For smoke coverage, it was interpreted as the Sprint 7 QA-plan
server smoke target that exists on `main`:
`cargo test -p server --test e2e_websocket_test`.

---

## Automated Smoke Commands

**Status**: PASS

Summary: 20 automated tests passed, 0 failed, plus workspace check and diff
check passed.

| Command | Result |
|---|---|
| `cargo test -p client --test playable_client_lobby_entry_test` | PASS, 5 passed |
| `cargo test -p server --test playable_client_lobby_entry_server_test` | PASS, 3 passed |
| `cargo test -p client --test playable_client_draft_shop_hand_bridge_test` | PASS, 4 passed |
| `cargo test -p server --test playable_client_draft_ready_bridge_test` | PASS, 3 passed |
| `cargo test -p server --test playable_client_real_e2e_loop_test` | PASS, 4 passed |
| `cargo test -p server --test e2e_websocket_test` | PASS, 1 passed |
| `cargo check --workspace` | PASS |
| `git diff --check` | PASS |

Failed checks:

- None.

---

## Sprint 7 Smoke Scope Status

| Scope Item | Status | Evidence |
|---|---|---|
| Workspace compiles for the current target set | PASS | `cargo check --workspace` passed. |
| Server starts without panic and accepts real Lightyear/WebSocket clients | PASS | `playable_client_real_e2e_loop_test` and `e2e_websocket_test` passed. |
| Primary client boots through normal client or browser/WASM entry | PASS WITH WARNING | Client entry behavior is covered by focused integration tests and committed evidence. This smoke does not claim full playable-client manual QA. |
| Fresh client hello maps identity and records `S2CHandshake` | PASS | `playable_client_lobby_entry_test`, `playable_client_lobby_entry_server_test`, and real-loop evidence cover this path. |
| Two clients create/join room, confirm class, and enter session from server-confirmed state | PASS | `playable_client_lobby_entry_server_test` and `playable_client_real_e2e_loop_test` passed. |
| DRAFT_INITIAL offering appears from `S2CDraftOffering` | PASS | `playable_client_draft_shop_hand_bridge_test` and real-loop evidence cover this path. |
| Purchase updates hand and economy only after authoritative server messages | PASS | `playable_client_draft_shop_hand_bridge_test` passed. |
| Ready signals reach server draft-ready authority and all-ready progression is server-owned | PASS | `playable_client_draft_ready_bridge_test` passed. |
| DRAFT_SHOP slots, purchase, refresh, and ready controls use real messages where reached | PASS | Focused draft/shop bridge tests passed; PLAYABLE-003 evidence reaches next-loop DRAFT_SHOP. |
| Placement submit, placement reveal, resolution event ordering, and next-loop endpoint are evidenced | PASS | `playable_client_real_e2e_loop_test` passed and evidence records next-loop DRAFT_SHOP after post-auction placement/resolution. |
| No harness-injected state used for Must Have completion evidence | PASS | PLAYABLE-003 evidence states no harness-injected completion path. |
| QA-COND-0005 and QA-COND-0006 remain accepted-risk context only | PASS WITH WARNING | No disposition changes were made. |
| Public release, broad accessibility, playtest/fun-hypothesis, and full playable-client manual QA claims are avoided | PASS WITH WARNING | This report explicitly carries the non-claims. |

---

## Evidence Reviewed

Sprint 7 Must Have evidence is complete for:

- PLAYABLE-001: Primary Client Bootstrap + Fresh Lobby Entry.
- PLAYABLE-002: Live Draft/Shop/Hand Bridge.
- PLAYABLE-003: Real End-to-End Loop Verification.
- S7-N1: Friend-game evidence index cleanup.

Verified friend-game endpoint from the evidence index:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

The reached endpoint is next-loop `DRAFT_SHOP` after post-auction
placement/resolution.

Game-over was not reached and is not claimed.

---

## Carried Warnings And Conditions

- This is friend-game smoke only.
- No public release readiness.
- No broad accessibility completion.
- No playtest/fun-hypothesis validation.
- No full playable-client manual QA.
- No full game completion.
- No game-over coverage is claimed.
- QA-COND-0005 and QA-COND-0006 remain accepted-risk/deferred context.
- PLAYABLE-003 evidence is internal friend-game evidence only, not public QA,
  QA sign-off, playtest validation, fun-hypothesis validation, or full manual QA.

---

## Manual Smoke Checks

No new manual playable-client QA was run or claimed by this smoke report.

Sprint 7 friend-game evidence already records the scoped two-client route to the
nearest reached endpoint. This smoke report verifies the committed automated
targets and carries the manual-scope warnings forward.

---

## Commands Run

```powershell
git fetch origin
git status --short --branch
git rev-parse HEAD
git rev-parse origin/main
Get-Content -Path production\sprints\sprint-7.md
Get-Content -Path production\sprint-status.yaml
Get-Content -Path production\qa\qa-plan-sprint-7-2026-05-06.md
Get-Content -Path production\qa\evidence\sprint-7-friend-game-evidence-index.md
Get-Content -Path production\qa\evidence\playable-client-real-e2e-loop.md
rg --files tests | rg "playable_client|e2e_websocket|smoke|critical"
rg -n "PDB|no-PDB|DEBUG:NONE|split-debuginfo|linker|MSVC|RUSTFLAGS" . -g "*.md" -g "*.ps1" -g "*.toml"
rg -n "cargo test|cargo check|test" .github\workflows
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo test -p client --test playable_client_lobby_entry_test
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo test -p server --test playable_client_lobby_entry_server_test
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo test -p client --test playable_client_draft_shop_hand_bridge_test
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo test -p server --test playable_client_draft_ready_bridge_test
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo test -p server --test playable_client_real_e2e_loop_test
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo test -p server --test e2e_websocket_test
$env:CARGO_PROFILE_DEV_DEBUG='0'; $env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'; cargo check --workspace
git diff --check
```

---

## Changed Files

- `production/qa/smoke-sprint-7-2026-05-07.md`

No source code, QA-COND disposition files, `production/sprint-status.yaml`,
`/story-done` records, QA sign-off, gate-check reports, team-qa reports, or
Sprint 8 planning files were changed.
