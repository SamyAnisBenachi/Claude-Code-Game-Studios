# S9-QA-001 Manual Game-Over Evidence

> **Story**: production/epics/playable-client/story-007-manual-browser-game-over-evidence-closure.md
> **Status**: PARTIAL — automated regressions pass; manual GUI route not executed (see MANUAL-FG-001)
> **Date**: 2026-05-08
> **Commit**: d2ac17ccb2c19be18dd1b2de63d2f2e968c235c8 (HEAD == origin/main)
> **Branch**: main
> **Prompt**: 449

## Executive Summary

All automated regression tests for the S9 result flow pass (16/16). The server
binary starts cleanly. The full manually driven two-client GUI route through
`GAME_OVER`, result screen, and Return to Lobby acknowledgement was not
captured because this evidence run was executed by a non-interactive AI agent
that cannot operate Bevy windowed client applications.

`S8-QA-001-W1` is **not closed** by this run. The manual/browser warning remains
open pending a human-operator-executed run.

---

## Preconditions Check

| Gate | Status | Detail |
|---|---|---|
| Source baseline recorded | PASS | `git rev-parse HEAD` = d2ac17ccb2c19be18dd1b2de63d2f2e968c235c8; matches origin/main |
| Dirty state documented | PASS | 3 staged sprint-management files present; no source or test files dirty |
| S9-RS-001 complete | PASS | main@b87e694; /story-done Prompt 412 |
| S9-RS-002 complete | PASS | main@8d963d5; /story-done Prompt 423 |
| S9-RS-003 complete | PASS | main@40b7599; /story-done Prompt 441 |
| S9-NATIVE-001 complete | PASS (with warning) | main@1e8e1dd; /story-done Prompt 433; native visual window confirmation not claimed |
| S8 carried warnings checked | PASS | S8-QA-001-W1, QA-COND-0005, QA-COND-0006 remain carried |
| Output folder created | PASS | `production/qa/evidence/captures/sprint-9-manual-game-over/` |

---

## Regression Evidence

All commands run from commit d2ac17ccb2c19be18dd1b2de63d2f2e968c235c8.

| Command | Result |
|---|---|
| `cargo test -p server --test result_acknowledgement_contract_test` | **PASS — 5/5** |
| `cargo test -p server --test result_acknowledgement_cleanup_handshake_test` | **PASS — 3/3** |
| `cargo test -p client --test result_screen_mvp_test` | **PASS — 6/6** |
| `cargo test -p client --test result_screen_return_to_lobby_test` | **PASS — 2/2** |
| `cargo check --workspace` | **PASS** |
| `git diff --check` | **PASS** (exit 0) |

**Total automated regression**: 16/16 tests pass.

### Test Breakdown

**result_acknowledgement_contract_test (5/5)**
- `ack_drain_is_session_owned_not_network_log_only`
- `acknowledgement_marks_only_sender_and_duplicate_is_noop`
- `all_ack_cleanup_removes_result_session_tokens_and_deferred_queues`
- `invalid_phase_unknown_sender_and_non_participant_ack_are_silent_discards`
- `timeout_cleanup_uses_same_terminal_cleanup_path`

**result_acknowledgement_cleanup_handshake_test (3/3)**
- `duplicate_ack_before_all_ack_is_idempotent_and_then_terminal_cleanup_removes_retention`
- `stale_acknowledgements_are_silent_discards_until_game_over_retention_exists`
- `timeout_cleanup_uses_the_same_retention_cleanup_without_requiring_acknowledgements`

**result_screen_mvp_test (6/6)**
- `objective_summary_keeps_alive_opponent_objectives_unknown_without_reveal`
- `outcome_copy_uses_server_authored_result_data`
- `result_screen_has_single_game_over_receiver_and_no_snapshot_receiver`
- `overlay_renders_game_over_result_and_hides_rematch`
- `reduced_motion_disables_entry_and_row_motion`
- `snapshot_only_game_over_uses_pending_fallback_and_return_action`

**result_screen_return_to_lobby_test (2/2)**
- `duplicate_return_to_lobby_activation_sends_one_ack_and_cleans_local_result_ui`
- `disconnected_transport_fallback_returns_to_lobby_without_mutating_server_phase_view`

---

## Server Startup Evidence

| Field | Result |
|---|---|
| Binary | target/msvc-local/debug/server.exe |
| Command | `SERVER_PORT=5000 target/msvc-local/debug/server.exe` |
| Duration | 8 seconds (killed by timeout; not a crash) |
| Exit code | 124 (SIGTERM from timeout) |
| Panic observed | None |
| Crash on startup | None |

**Note**: Bevy logging writes to the Windows console API in this environment;
stdout file redirect produced an empty log. This is a capture tooling limitation.
The binary ran cleanly for 8 seconds. No build errors, no immediate crash.

---

## Manual Route Evidence

### Blocker: MANUAL-FG-001

**Severity**: S2 — blocks full manual route execution.

**Description**: This evidence run was executed by a non-interactive AI agent
(Claude Code). The agent can run shell commands, cargo tests, and observe
process exit codes, but cannot interact with running Bevy windowed client
applications. The following route steps require a human operator with visual
access to two simultaneously running game windows:

- Room create/join (click, text input, slot selection)
- Class confirm (UI selection and button click)
- DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION (card purchase, bid, ready UI)
- Placement (card staging and submit UI)
- Resolution loop observation (visual)
- GAME_OVER (visual observation of result screen)
- Return to Lobby (button click, acknowledgement timing observation)
- Screenshots and video capture

**Last reached step**: Server startup clean (server ran without panic).
Client windows were not launched. No GUI interaction was attempted.

**Owner**: QA tester (human operator required).

**Workaround**: Execute this run with a human operator using the existing
runbook: `production/qa/evidence/manual-friend-game-evidence-runbook.md`
and harness prep: `production/qa/evidence/sprint-9-manual-evidence-harness-prep.md`.
No product implementation work is needed — only human operator time.

**Recommended next story**: No new implementation story required. S9-QA-001
is fully ready at the implementation level. The next step is a human-operator
re-execution of this evidence run.

---

## Route Steps — Status

| Route step | Status | Notes |
|---|---|---|
| Baseline confirmed | PASS | HEAD == origin/main |
| Server launch (startup check) | PARTIAL | 8s clean run; stdout capture failed; no panic |
| Client A launch | NOT REACHED | Requires human operator |
| Client B launch | NOT REACHED | Requires human operator |
| Room create / join | NOT REACHED | |
| Class confirm | NOT REACHED | |
| DRAFT_INITIAL (purchase, ready) | NOT REACHED | |
| DRAFT_SHOP | NOT REACHED | |
| DRAFT_AUCTION | NOT REACHED | |
| Placement | NOT REACHED | |
| Resolution | NOT REACHED | |
| GAME_OVER | NOT REACHED | |
| Result screen | NOT REACHED | |
| Return to Lobby / ack | NOT REACHED | |

---

## Result-Screen UX Checks

All checks BLOCKED — result screen not reached.

Authoritative UX check coverage for this screen exists in:
- `production/qa/evidence/result-screen-mvp-evidence.md` (automated/unit level)
- `tests/integration/presentation/result_screen_mvp_test.rs` (6 tests, all passing)

Browser/manual viewport claim (1366×768, 1920×1080, 150% UI scale) is not
made by automated tests or this run. That claim requires a human-operator
visual run.

---

## Screenshots and Video

None captured. Capture directory created at:
`production/qa/evidence/captures/sprint-9-manual-game-over/screenshots/`

---

## Carried Conditions and Non-Claims

### S8-QA-001-W1

**REMAINS OPEN.** The full manually driven browser or native two-client
friend-game route through `GAME_OVER` was not captured by this run.
S8-QA-001-W1 is not closed.

### QA-COND-0005

**REMAINS CARRIED** as accepted risk for friend-game scope only. This run does
not verify Standard-tier accessibility completion. Result-screen MVP automated
evidence covers keyboard focus MVP, reduced-motion MVP, and photosensitivity
static behavior only. Broad Standard-tier accessibility completion is not
claimed.

### QA-COND-0006

**REMAINS ACCEPTED-RISK/DEFERRED.** This run is not playtest evidence,
fun-hypothesis validation, or a playtest report.

### Explicit Non-Claims

- No public release readiness claimed.
- No release-candidate readiness claimed.
- No full game completion claimed.
- No broad Standard-tier accessibility completion claimed.
- No playtest validation claimed.
- No fun-hypothesis validation claimed.
- No full playable-client manual QA claimed.
- No full regression campaign claimed.
- No Sprint 8 close-out, Sprint 9 close-out, smoke, QA sign-off, gate-check,
  `/dev-story`, `/story-done`, or CI watch run.
- No store, deployment, or launch readiness claimed.

---

## Capture Artifacts

| Artifact | Status |
|---|---|
| `captures/sprint-9-manual-game-over/command-summary.md` | Created |
| `captures/sprint-9-manual-game-over/server-startup.log` | Empty (Windows console API capture limitation) |
| `captures/sprint-9-manual-game-over/server-summary.md` | Created |
| `captures/sprint-9-manual-game-over/defects.md` | Created — MANUAL-FG-001 recorded |
| `captures/sprint-9-manual-game-over/route-summary.json` | Created — all route steps NOT_REACHED after server startup |
| `captures/sprint-9-manual-game-over/screenshots/` | Directory created; no screenshots (route not reached) |
| `captures/sprint-9-manual-game-over/client-a-summary.md` | Not created — client not launched |
| `captures/sprint-9-manual-game-over/client-b-summary.md` | Not created — client not launched |
