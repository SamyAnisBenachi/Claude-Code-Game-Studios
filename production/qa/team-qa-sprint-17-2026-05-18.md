# PROMPT 1278 - Sprint 17 Team-QA

## Status line

1278: SPRINT-17-TEAM-QA: APPROVED-WITH-CONDITIONS

## Verdict

APPROVED-WITH-CONDITIONS for Sprint 17 Team-QA review of record.

This is not Sprint 17 close-out, not Sprint 18 activation, not a release-readiness claim, and not a Polish -> Release gate retry.

## Source of truth

| Field | Value |
|---|---|
| Review date | 2026-05-18 |
| Command intent | `/team-qa sprint` per `production/qa/qa-plan-sprint-17.md` |
| Source under review | `origin/main@946ca392c94a4988e9c6b4483848233fe6323061` |
| Source commit | `Integrate board rendering damage message registration` |
| Clean review worktree | `D:/Tmp/gcs-prompt-1278-team-qa` |
| Clean worktree status | detached HEAD at `946ca39`, no tracked modifications |
| Stage | `Polish` preserved |
| Sprint status | `sprint: 17`, `status: active`, `stage: "Polish"` |
| Cargo by PROMPT 1278 | None. This Team-QA is paperwork/review-of-record only. |

The root checkout was dirty on `integrate/windows-dev-launcher-visual-polish-1261` at prompt entry. PROMPT 1278 did not treat that checkout as source of truth and did not stage, revert, or modify the pre-existing dirty code files.

## Sprint 17 Row State

At `origin/main@946ca39`, `production/sprint-status.yaml` records:

| Row | Priority | Status |
|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | Must | `ready` - human-operator-blocked carry |
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` | Must | `done` |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` | Should | `in_progress` - PROMPT 1112 partial disposition remains |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` | Should | `done` |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` | Should | `done` |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` | Should | `done` |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` | Nice | `done` |
| `S17-SERVER-START-OF-TURN-DEBUG-001` | Nice | `done` |
| `S17-UI-HAND-B0004-CLEANUP-001` | Nice | `done` |

Count: 7 of 9 Sprint 17 rows are `done`, 1 is `in_progress`, and 1 is the long-running human-operator-blocked carry. The hand-reserve microbadge source repair that addresses the carried AC3 surface is on `origin/main` via `c842668`, but the parent Sprint 17 HUD opp/mana row remains `in_progress` because no final `/story-done` paperwork has closed or explicitly carried that row.

## Smoke Evidence

PROMPT 1278 was instructed to use PROMPT 1277 smoke rerun evidence after a PASS/PASS-WITH-WARNINGS result. I found no durable tracked file named `reports/PROMPT-1277*` and no `production/qa/smoke-sprint-17*` report in the repository. I therefore treated the prompt-provided PROMPT 1277 disposition plus the local rerun artifacts left in the checkout as the available smoke evidence:

- `production/qa/evidence/dev-runs/2026-05-18-200005/launch-summary.json`
- `production/qa/evidence/dev-runs/2026-05-18-200005/{server.log,client_a.log,client_b.log}`
- `qa-snapshots/2-000000-1779110939048/{snapshot.json,screenshot.png}`

The 20:00 dev run launched one server and two native clients, reached a real two-client session, class reveal, `DraftInitial`, `Placement`, `Resolution`, `DraftShop`, and a second `Placement` transition. No panic, fatal, or error lines were found in the 20:00 log scan. Two warnings are carried as conditions:

- `client_a.log`: `hand_ui_phase_transition_auto_submit_short_circuit` with `invalid_submit_state` after one staged card at the round-1 `Placement -> Resolution` transition.
- `server.log`: `RSM disconnect timer breach: grace window exceeded` after a later client disconnect in `DraftShop`.

The earlier 13:28 dev run reached room creation/join but timed out in lobby and is not used as the positive smoke path. The QA snapshot bundle is a Lobby snapshot only; it confirms snapshot capture works but does not close any visual acceptance item.

## Smoke Repair Chain on `origin/main`

The late Sprint 17 smoke repair commits are all ancestors of `origin/main@946ca39`:

- `23d1c1b` - `test: make dev launcher sidecar test hermetic` (PROMPT 1271 / 1272)
- `35a95d5` - `test: account for shop auction prepool formula nodes` (PROMPT 1269 / 1275)
- `c94514f` - `fix: clear ui clean pass lint violations` (PROMPT 1270 / 1274)
- `946ca39` - `Integrate board rendering damage message registration` (PROMPT 1268 / 1273 / 1276)

Focused evidence from the corresponding reports:

- PROMPT 1268 / 1276: board-rendering message registration repair; targeted board-rendering tests and `cargo check -p client` PASS.
- PROMPT 1269: shop-auction prepool formula updated for the intentional 88-node shape; targeted shop-auction tests PASS.
- PROMPT 1270: UI clean-pass lint repairs; targeted lint, hand, lobby, and HUD tests PASS; `cargo check -p client` PASS.
- PROMPT 1271: dev-launcher ignored host-filesystem test converted to hermetic default-suite test; `cargo test -p dev-launcher-app --tests` PASS 31/31, 0 ignored.

Direct `rg '#\[ignore\]' -g '*.rs'` on the clean worktree found only documentation/comment references, not active `#[ignore]` attributes.

## Conditions

This approval is conditioned on the following preserved facts:

1. Sprint 17 remains active. PROMPT 1278 does not close it.
2. `S11-HUD-TIMER-EYEBALL-VISUAL-001` remains human-operator-blocked. No LLM `/story-done` is authorized for that row; real two-client screenshot evidence is still required.
3. `S17-UI-HUD-OPP-MANA-CLEANUP-001` remains `in_progress` in `production/sprint-status.yaml`. The AC3 source-side hand-reserve cleanup is on `origin/main@c842668`, but the parent row still needs explicit `/story-done` paperwork or producer-approved carry handling before close-out.
4. PROMPT 1277 durable smoke report artifact is missing from tracked repo files. The Team-QA accepts the prompt-provided PASS/PASS-WITH-WARNINGS disposition with local rerun artifacts, but records this as a documentation gap.
5. PROMPT 1277 local rerun warnings are accepted as non-blocking for Team-QA but must not be hidden: the invalid placement auto-submit short-circuit and disconnect grace timer breach are carried as smoke warnings.
6. PROMPT 761 Polish -> Release gate-check remains FAIL at `production/gate-checks/gate-polish-release-2026-05-12.md`; no retry is attempted.
7. Stage remains `Polish`; no Sprint 18 activation is performed by this report.
8. `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 683-era runtime divergence, Sprint 12 story 019 cannot-reproduce disposition, PROMPT 1054 P1 UI snapshot visual retest blocked-human state, and all prior closed-with-conditions sprint dispositions are preserved.

## Non-Claims

PROMPT 1278 does not claim:

- Sprint 17 close-out.
- Sprint 18 activation.
- release readiness or release-candidate readiness.
- full game completion.
- broad/Standard-tier accessibility completion.
- playtest/fun-hypothesis validation.
- full playable-client manual QA.
- two-client GAME_OVER closure.
- final-art completion.
- Polish -> Release gate retry.
- stage advance from Polish.
- closure of the HUD timer human visual carry.
- closure of the HUD opp/mana parent row.
- closure of any remaining PROMPT 1022, PROMPT 1076, or PROMPT 1077 findings outside the concrete repairs already on `origin/main`.

## Recommendation

Sprint 17 may proceed to a separate close-out decision only as `closed-with-conditions` or equivalent carry-aware paperwork. Before that close-out, the producer should either:

- run final `/story-done` paperwork for `S17-UI-HUD-OPP-MANA-CLEANUP-001` now that the AC3 source-side hand-reserve cleanup landed on `origin/main`, or
- explicitly carry that parent-row paperwork gap forward with the Sprint 17 close-out conditions.

No repair prompt is required solely for the late smoke repair chain; the relevant fixes are on `origin/main` and the available smoke rerun evidence is acceptable with the warnings above.

## Files Changed by PROMPT 1278

- `production/qa/team-qa-sprint-17-2026-05-18.md` - this Team-QA report.
- `reports/PROMPT-1278-sprint-17-team-qa.md` - mandatory final report copy.
- `reports/PROMPT-1278-sprint-17-team-qa.summary.txt` - relay summary.

No production code, tests, sprint status, sprint plan, stage file, gate-check, release artifact, or Sprint 18 activation file was modified by PROMPT 1278.

1278: SPRINT-17-TEAM-QA: APPROVED-WITH-CONDITIONS
