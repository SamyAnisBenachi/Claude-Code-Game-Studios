# Sprint 10 Evidence Index (Aggregator)

> **Created**: 2026-05-13 (PROMPT 771 — Sprint 11 draft Must Have carry `S11-EVIDENCE-INDEX-CARRY-001`,
> deferred from Sprint 10 nice-to-have `S10-N1` per PROMPT 763 close-out).
> **Source-of-truth at authoring**: `origin/main@8869a54`.
> **Sprint 10 disposition**: `closed-with-conditions` per PROMPT 763 at
> `origin/main@9af992f` (2026-05-13) — recorded in
> `production/sprint-status.yaml` `sprint_10_closeout:`.
> **Stage**: `Polish`. `production/stage.txt` unchanged.
> **Sprint 11 disposition**: `draft / not_active` (PROMPT 764). This index
> does **not** activate Sprint 11, does **not** mutate
> `production/sprint-status.yaml`, does **not** mutate
> `production/sprints/sprint-11.md`, does **not** mutate
> `production/stage.txt`, and does **not** run `/dev-story`, `/story-done`,
> `/smoke-check`, `/team-qa`, `/gate-check`, or `/qa-plan`.
> **PROMPT 761 Polish→Release gate-check `FAIL`**: preserved unchanged at
> `production/gate-checks/gate-polish-release-2026-05-12.md` — do not retry
> until release-scope artefacts exist.

This aggregator collates the per-story evidence already on `origin/main` for
Sprint 10 and records each story's status, integration commit, evidence path,
and friend-game-lite no-claim language in one place. It is read-only over the
underlying evidence — it does not modify, supersede, or reclassify any
existing artefact. Authoritative status remains `production/sprint-status.yaml`.

---

## Sprint 10 Headline

| Field | Value |
|---|---|
| Sprint window | 2026-05-21 → 2026-06-03 (per plan; actual close-out paperwork landed 2026-05-13) |
| Scope | Polish / friend-game-lite paperwork only — close PAW-002..006, drain test-fixture + plugin-registration tech debt, start targeted visual-UI chrome |
| Must-Have done | 6 / 6 |
| Should-Have done | 2 / 3 (S10-TD-003 deferred) |
| Nice-to-Have done | 0 / 2 (S10-N1 + S10-N2 deferred) |
| Smoke (retry-7) | **PASS WITH WARNINGS** — `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` (1123/1123 effective; 11 ignored D-5 tests; HUD timer eyeball check deferred) |
| Team-QA | **APPROVED WITH CONDITIONS** — `production/qa/team-qa-sprint-10-2026-05-11.md` (5 conditions; C-5 carries deferred items into Sprint 11 planning) |
| Polish→Release gate (PROMPT 761) | **FAIL** — `production/gate-checks/gate-polish-release-2026-05-12.md` (0/13 required artefacts present; Sprint 10 is scoped friend-game-lite, not release) |
| Stage after close-out | Polish (unchanged) |

---

## Per-Story Status (Sprint 10)

### Must Have

| ID | Story | Status | Closed | Integration commit | Primary evidence path |
|---|---|---|---|---|---|
| S10-PAW-001 | PAW-002..PAW-006 `/story-done` close-out batch | done | 2026-05-10 | PAW-002 `40a9f72`, PAW-003 `792a9d8`, PAW-004 `a7e397a`, PAW-005 `7782c6f`, PAW-006 `724470e` (all on `main`) | `tests/integration/presentation/hand_ui_asset_wiring_test.rs`, `shop_auction_asset_wiring_test.rs`, `hud_asset_wiring_test.rs`, `board_asset_wiring_test.rs`, `lobby_asset_wiring_test.rs` |
| S10-TD-001 | Test-fixture cascade-fail repair | done | 2026-05-10 | `7075da7` / `4b0c456` / `c11d1b6` / `bb51463` / `7c8f400` (pre-cascade prep `bbdbcd6` + `24e8095` grandfathered) | `tests/integration/auction/pool_integration_test.rs` (pattern); fixture suite green; AC4 evidence doc deferred (no `sprint-10-test-fixture-repair.md` on disk) |
| S10-TD-002 | Plugin-registration audit and dead-plugin sweep | done | 2026-05-10 | Audit `0648deb`, resolutions `bbdb91e` + `8932d8c` + `f06271a` | `production/qa/evidence/sprint-10-plugin-registration-audit.md` |
| S10-CARRY-001 | Sprint 9 carry-over consolidation | done | 2026-05-10 | S9-AUDIO-001 integrated at `9c00e06`; Sprint 10 activation flip `8ff4f84` + `e35b955` | `production/sprint-status.yaml` `carried_conditions:` block + `previous_sprint_closeout.carried_into_sprint_10` (no separate evidence doc — sprint-status.yaml IS the evidence per AC) |
| S10-POLISH-001 | HUD visual chrome — timer + class figurines + RESOLUTION dim | done | 2026-05-10 | `b780f0e` (cherry-pick of `1a1ae4f`) on `main` | `production/qa/evidence/sprint-10-hud-chrome-evidence.md` + `tests/integration/hud/hud_resolution_dim_test.rs` (8/8 PASS) |
| S10-POLISH-002 | Shop/Auction panel chrome wiring | done | 2026-05-10 | per PROMPT 621 / `2026-05-10` | `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md` + `tests/integration/shop_auction_ui/chrome_wiring_test.rs` (4/4 PASS) |

### Should Have

| ID | Story | Status | Closed / Disposition | Integration commit | Primary evidence path |
|---|---|---|---|---|---|
| S10-POLISH-003 | Lobby visual chrome — class carousel + portraits + slot indicators | done | 2026-05-10 | `084129c` (cherry-pick of `fd2e0a6`) on `main` | `production/qa/evidence/sprint-10-lobby-chrome-evidence.md` + `tests/integration/session/lobby_chrome_wiring_test.rs` (5/5 PASS); AC-5 ADVISORY pre-existing PAW-006 `lobby_asset_wiring_test.rs` compile break (12 × E0596) — candidate Sprint 11 story `S11-TD-PAW-006-COMPILE-001` |
| S10-TD-003 | Doc hygiene tech debt sweep | **deferred → Sprint 11 planning** | PROMPT 763 (2026-05-13) | partial: `App::add_message` idempotency correction already on `main` at PROMPT 770 time (Bevy 0.18 fact verified at `bevy_app-0.18.1/src/sub_app.rs:358`); outstanding ADR-011 `TR-NP-04` → `TR-NP-006` corrections + Network Protocol Rule 7 `ADR-011` breadcrumb landed on `main` by PROMPT 770 at `0d19690` under draft Sprint 11 story `S11-DOC-HYGIENE-CARRY-001` (Sprint 11 row not flipped to `done` — that is a Sprint 11 activation-time decision) | none authored under S10 scope — Sprint 11 carry tracked in `production/sprint-status.yaml` `next_sprint:` block and PROMPT 770 disposition in `production/session-state/codex-orchestrator-state.md` |
| ECO-004 | Kill and Objective Awards reward-loop polish | done | 2026-05-10 | `9fb8e60` (cherry-pick of `bb1b104`) on `main` (PROMPT 650 closure) | `tests/integration/economy/reward_loop_awards_test.rs`; pre-existing AuctionSettled/ResolutionComplete fixture cluster surfaced as Sprint 11 candidate |

### Nice to Have

| ID | Story | Status / Disposition | Notes |
|---|---|---|---|
| S10-N1 | Sprint 10 evidence index | **deferred → Sprint 11 planning** (PROMPT 763) → carried as draft `S11-EVIDENCE-INDEX-CARRY-001` (PROMPT 764) → **this file** authored under PROMPT 771 | Sprint 11 row not flipped to `done`; that is a Sprint 11 activation-time decision |
| S10-N2 | Friend-game route readability notes | **deferred → Sprint 11 planning** (PROMPT 763) → carried as draft `S11-ROUTE-READABILITY-CARRY-001` (PROMPT 764) | No `sprint-10-readability*.md` exists under `production/ux/`, `design/ux/`, or `production/qa/`. Authoring deferred. |

### PAW-002..PAW-006 sub-table (rolled up under S10-PAW-001)

| PAW ID | Story file | Integration commit | Merge commit | Test evidence |
|---|---|---|---|---|
| PAW-002 | `production/epics/presentation-asset-wiring/story-002-hand-ui-card-frames.md` | `40a9f72` | `69a03cc` | `tests/integration/presentation/hand_ui_asset_wiring_test.rs` |
| PAW-003 | `production/epics/presentation-asset-wiring/story-003-shop-auction-chrome.md` | `792a9d8` | — | `tests/integration/presentation/shop_auction_asset_wiring_test.rs` |
| PAW-004 | `production/epics/presentation-asset-wiring/story-004-hud-figurines-timer-dots.md` | `a7e397a` | `2132129` | `tests/integration/presentation/hud_asset_wiring_test.rs` |
| PAW-005 | `production/epics/presentation-asset-wiring/story-005-board-unit-sprites.md` | `7782c6f` | `ece5f48` | `tests/integration/presentation/board_asset_wiring_test.rs` |
| PAW-006 | `production/epics/presentation-asset-wiring/story-006-lobby-portraits.md` | `724470e` | `bb80b47` | `tests/integration/presentation/lobby_asset_wiring_test.rs` |

Per `production/sprint-status.yaml` `presentation_asset_wiring.stories[*].tech_debt`,
every PAW row carries `PAW-TD-*-a: accept-risk — placeholder PNGs, not final
art (friend-game scope)` and `PAW-TD-*-b: accept-risk — no manual
browser/native visual capture (friend-game scope)`. PAW-004 additionally
carries `PAW-TD-004-b: accept-risk — opponent objective dots start as
Unknown; no Alive/Unknown disambiguation art (friend-game scope)`. These
accept-risk waivers are friend-game scope only — they are not a final-art
or public-release-art claim.

---

## Deferred Items (NOT silently dropped)

| ID | Original sprint | Disposition | Sprint 11 draft carry ID |
|---|---|---|---|
| S10-TD-003 | Sprint 10 Should-Have | DEFERRED to Sprint 11 planning (PROMPT 763) | `S11-DOC-HYGIENE-CARRY-001` (Sprint 11 draft Must Have; PROMPT 770 landed ADR-011 + Rule 7 corrections at `0d19690` on `main`, Sprint 11 row not flipped) |
| S10-N1 | Sprint 10 Nice-to-Have | DEFERRED to Sprint 11 planning (PROMPT 763) | `S11-EVIDENCE-INDEX-CARRY-001` (Sprint 11 draft Must Have; PROMPT 771 authored this aggregator under the draft Sprint 11 row) |
| S10-N2 | Sprint 10 Nice-to-Have | DEFERRED to Sprint 11 planning (PROMPT 763) | `S11-ROUTE-READABILITY-CARRY-001` (Sprint 11 draft Must Have; not yet authored) |

All three are also recorded in `production/qa/team-qa-sprint-10-2026-05-11.md`
Condition C-5 and `production/gate-checks/gate-polish-release-2026-05-12.md`
Recommendation 1. Sprint 11 activation (and any Sprint 11 row flip to `done`)
remains a separate prompt — neither this aggregator nor PROMPT 770 mutated
`production/sprint-status.yaml` or `production/sprints/sprint-11.md`.

---

## Carried Conditions (unchanged by this aggregator)

| Condition | Status | Notes |
|---|---|---|
| S8-QA-001-W1 — manual/browser two-client GAME_OVER gap | **OPEN** | Carried from Sprint 8 through Sprints 9 and 10. Full manually-driven two-client GUI route through GAME_OVER has not been captured. Resolution path: human operator executes `production/qa/evidence/manual-friend-game-evidence-runbook.md`. |
| QA-COND-0005 — Standard-tier accessibility | **Accepted risk** | Friend-game scope only; Standard-tier accessibility completion explicitly **not** verified. |
| QA-COND-0006 — playtest / fun-hypothesis validation | **Accepted-risk / deferred** | No playtest evidence; no fun-hypothesis validation. |
| 11 ignored D-5 tests (smoke retry-7 Warning W1) | **Carried** | Enumerated in `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` lines 60-74. Sprint 11 draft folds these into `S11-TD-IGNORED-D5-TRIAGE-001` and `S11-TD-FIXTURE-HAND-UI-ONENTER-001`. |
| HUD timer eyeball visual check (smoke retry-7 Warning W2) | **Deferred** | Automated integration coverage in `tests/integration/hud/hud_phase_timer_bar_test.rs` (4/4 PASS at `112ac83`); a brief native eyeball run is recommended before public-facing demo. Sprint 11 draft tracks as `S11-HUD-TIMER-EYEBALL-VISUAL-001`. |
| Placeholder / friend-game art scope | **Accepted risk** | PAW-TD-*-a accept-risk on placeholder PNGs across PAW-002..PAW-006. No final-art / public-release-art claim. |

---

## Non-Claims

This aggregator and the Sprint 10 close-out it summarises explicitly do **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- full manual / browser two-client GAME_OVER route (S8-QA-001-W1 remains OPEN)
- final-art / asset-production completion
- a fresh smoke, `/team-qa`, `/gate-check`, `/dev-story`, `/story-done`, `/qa-plan`, or release-checklist run authored under PROMPT 771
- Sprint 11 activation (Sprint 11 remains `draft / not_active`)
- any new Sprint 10 story closure or status flip (sprint-status.yaml unchanged by this prompt)

---

## Evidence File Map (Sprint 10)

| File | Contents |
|---|---|
| `production/qa/evidence/sprint-10-hud-chrome-evidence.md` | S10-POLISH-001 HUD visual chrome — RESOLUTION dim + timer + figurines manual evidence (manual capture deferred per friend-game-lite paperwork pattern) |
| `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md` | S10-POLISH-002 Shop/Auction panel chrome wiring evidence (AC-3/AC-7 screenshot deferred) |
| `production/qa/evidence/sprint-10-lobby-chrome-evidence.md` | S10-POLISH-003 Lobby visual chrome evidence (manual capture deferred) |
| `production/qa/evidence/sprint-10-plugin-registration-audit.md` | S10-TD-002 audit doc (14/14 server + 14/14 client plugins reconciled; 0 silent dead plugins) |
| `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` | Sprint 10 smoke retry-7 — PASS WITH WARNINGS (1123/1123 effective; 11 D-5 ignored; HUD timer eyeball deferred) |
| `production/qa/team-qa-sprint-10-2026-05-11.md` | Sprint 10 team-QA report — APPROVED WITH CONDITIONS (5 conditions; C-5 carries S10-TD-003 / S10-N1 / S10-N2 into Sprint 11 planning) |
| `production/gate-checks/gate-polish-release-2026-05-12.md` | PROMPT 761 Polish→Release gate-check — FAIL (0/13 required artefacts present; stage remains Polish) |
| `production/qa/qa-plan-sprint-10-2026-05-10.md` | Sprint 10 QA plan referenced by smoke retry-7 |
| `tests/integration/presentation/{hand_ui,shop_auction,hud,board,lobby}_asset_wiring_test.rs` | PAW-002..PAW-006 integration test evidence |
| `tests/integration/hud/hud_resolution_dim_test.rs` | S10-POLISH-001 RESOLUTION dim integration test (8/8 PASS) |
| `tests/integration/shop_auction_ui/chrome_wiring_test.rs` | S10-POLISH-002 panel chrome integration test (4/4 PASS) |
| `tests/integration/session/lobby_chrome_wiring_test.rs` | S10-POLISH-003 lobby chrome integration test (5/5 PASS) |
| `tests/integration/economy/reward_loop_awards_test.rs` | ECO-004 reward-loop awards integration test |
| `production/sprint-status.yaml` | Authoritative Sprint 10 status — `sprint_10_closeout:` block + per-story rows |
| `production/sprints/sprint-10.md` | Sprint 10 plan + close-out banner |
| `production/sprints/sprint-11.md` | Sprint 11 **draft** plan (not active) — references this aggregator's predecessor carry `S11-EVIDENCE-INDEX-CARRY-001` |

---

## Authoring Disposition (PROMPT 771)

PROMPT 771 authored this aggregator under draft Sprint 11 story
`S11-EVIDENCE-INDEX-CARRY-001` (carried from deferred Sprint 10
nice-to-have `S10-N1` per PROMPT 763 close-out and PROMPT 764 Sprint 11
draft plan). PROMPT 771 did **not** run `/dev-story`, `/story-readiness`,
`/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/qa-plan`, or
any implementation; did **not** modify production code under `client/`,
`server/`, `shared/`, or `tests/`; did **not** mutate
`production/sprint-status.yaml`, `production/sprints/sprint-11.md`, or
`production/stage.txt`; did **not** modify `.octogent/`, `.gitignore`,
`.claude/settings.json`, `reports/`, or `.claude/scheduled_tasks.lock`;
did **not** activate Sprint 11; did **not** flip the Sprint 11
`S11-EVIDENCE-INDEX-CARRY-001` row to `done` (Sprint 11 activation-time
decision); and did **not** make any release, release-candidate,
full-game-completion, broad / Standard-tier accessibility,
playtest / fun-hypothesis, full-manual-QA, or final-art claim.
