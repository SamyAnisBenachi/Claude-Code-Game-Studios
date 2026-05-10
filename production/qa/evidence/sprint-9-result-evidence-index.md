# Sprint 9 Result Evidence Index

> **Created**: 2026-05-08 (S9-QA-002 partial — S9-QA-001 blocker recorded)
> **Last updated**: 2026-05-08

This index records the Sprint 9 result flow evidence status, S9-QA-001 blocker
disposition, and carried Sprint 8 conditions. It is created per S9-QA-002 scope
now that S9-QA-001 has produced a blocker record.

---

## S9 Story Completion Status

| Story | Status | Commit | Evidence doc |
|---|---|---|---|
| S9-RS-001 Result ack/data contract | Complete | main@b87e694 | result-acknowledgement-cleanup-handshake-evidence.md (via regression) |
| S9-RS-002 Result Screen MVP | Complete | main@8d963d5 | result-screen-mvp-evidence.md |
| S9-RS-003 Cleanup handshake | Complete | main@40b7599 | result-acknowledgement-cleanup-handshake-evidence.md |
| S9-NATIVE-001 Native operator controls | Complete (warning) | main@1e8e1dd | native-friend-game-operator-controls-evidence.md |
| S9-QA-001 Manual GAME_OVER evidence | **PARTIAL/BLOCKED** | d2ac17c | sprint-9-manual-game-over-evidence.md |
| S9-QA-002 Evidence index cleanup | **PARTIAL** | d2ac17c | this file |

---

## S9-QA-001 Manual Route Status

| Check | Status |
|---|---|
| Automated regressions (16 tests) | PASS |
| cargo check --workspace | PASS |
| git diff --check | PASS |
| Server startup clean | PASS (partial — stdout capture failed; no panic) |
| Full manually driven two-client GUI route | **NOT CAPTURED** |
| GAME_OVER observation (both clients) | Not reached |
| Result screen observation | Not reached |
| Return to Lobby acknowledgement | Not reached |

**Blocker**: MANUAL-FG-001 (S2) — Non-interactive AI agent cannot operate
Bevy windowed client applications. Human operator required.

**Last reached step**: Server startup clean. Client windows not launched.

---

## S8-QA-001-W1 Disposition

**REMAINS OPEN.** Sprint 8 carried warning is not closed. The full manually
driven browser or native two-client route through GAME_OVER has not been
captured as of this index update.

Resolution path: Human operator executes the route per
`manual-friend-game-evidence-runbook.md` and updates
`sprint-9-manual-game-over-evidence.md` with full artifact evidence.

---

## Carried Conditions

| Condition | Status | Notes |
|---|---|---|
| S8-QA-001-W1 | **Carried/open** | Full manual/browser GAME_OVER route not captured |
| QA-COND-0005 | **Accepted risk** | Friend-game scope only; Standard-tier accessibility not verified |
| QA-COND-0006 | **Accepted-risk/deferred** | No playtest evidence or fun-hypothesis validation |

---

## Non-Claims

- No public release readiness.
- No release-candidate readiness.
- No full game completion.
- No broad Standard-tier accessibility completion.
- No playtest validation.
- No fun-hypothesis validation.
- No full playable-client manual QA.
- No full regression campaign.
- No Sprint 8 close-out or Sprint 9 close-out.
- No smoke, QA sign-off, gate-check, /dev-story, /story-done, team-qa,
  or CI watch run.

---

## Evidence File Map

| File | Contents |
|---|---|
| `sprint-9-manual-game-over-evidence.md` | S9-QA-001 primary evidence; regression results; blocker record |
| `captures/sprint-9-manual-game-over/command-summary.md` | Baseline, commands, regression table |
| `captures/sprint-9-manual-game-over/server-summary.md` | Server startup evidence |
| `captures/sprint-9-manual-game-over/defects.md` | MANUAL-FG-001 formal record |
| `captures/sprint-9-manual-game-over/route-summary.json` | Machine-readable route/blocker disposition |
| `captures/sprint-9-manual-game-over/screenshots/` | Empty — route not reached |
| `result-screen-mvp-evidence.md` | S9-RS-002 automated/unit evidence |
| `result-acknowledgement-cleanup-handshake-evidence.md` | S9-RS-003 integration test evidence |
| `native-friend-game-operator-controls-evidence.md` | S9-NATIVE-001 operator controls evidence |
| `sprint-9-manual-evidence-harness-prep.md` | Harness prep and gate checklist (planning) |
| `manual-friend-game-evidence-runbook.md` | Route execution runbook for human operator |

---

## Accepted-Risk Closure Note (2026-05-10)

S9-QA-001 was closed accepted-risk friend-game-lite on 2026-05-10 per PROMPT 572 user authorization. Closure evidence remains the existing automated regressions 16/16 pass at e26e240; no manual two-client GUI route was executed. S8-QA-001-W1 remains OPEN and explicitly carried. QA-COND-0005 (Standard-tier accessibility) and QA-COND-0006 (playtest fun-hypothesis validation) remain accepted-risk and are not closed by this entry. No public release readiness, full game completion, broad accessibility completion, full playable-client manual QA, or playtest validation is claimed.
