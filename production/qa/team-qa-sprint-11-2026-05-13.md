# Team-QA Report: Sprint 11 (Polish / friend-game scope)

| Field | Value |
|---|---|
| **Date** | 2026-05-13 |
| **Sprint** | Sprint 11 — `active` (Polish stage; activated by PROMPT 773) |
| **Stage** | `Polish` (unchanged) |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | Friend-game / Polish slice — explicitly NOT public release readiness |
| **Skill** | `/team-qa sprint` (qa-lead + producer roles, root checkout, no worktree) |
| **Prompt** | PROMPT 791 — Sprint 11 Team QA / QA Sign-Off |
| **Commit Under Review (HEAD)** | `1617352274531a63f96db1e44ec8a755214ceb7e` (`qa(smoke): Sprint 11 smoke check`, PROMPT 790 paperwork commit) |
| **HEAD == origin/main** | yes (verified after `git fetch origin`) |
| **Working-copy state** | one unstaged modification at `.claude/settings.json` preserved untouched per operating contract; no staged changes; no new untracked files |
| **Review mode** | Lean (no `production/review-mode.txt` override) |

---

## Verdict: PASS-WITH-WARNINGS

Sprint 11 Must Have scope is complete (6/6 `done` on `origin/main@1617352`). The Sprint 11 smoke check is `PASS-WITH-WARNINGS` (1129 passed / 0 failed / 5 ignored across 189 binaries). All 5 ignored tests match the **documented Cluster B set** in `production/qa/evidence/sprint-11-ignored-d5-triage.md` with owner-named dispositions and named follow-up story slugs or decision gates. No undocumented ignored test and no undocumented failure surfaced. Carried conditions (`S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, placeholder-art `PAW-TD-*-a` accept-risk, HUD-timer eyeball check W2) are preserved unchanged.

This Team-QA report makes **no claim** of (preserved non-claims):

- no public release readiness
- no release-candidate readiness
- no full game completion
- no broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- no playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- no full playable-client manual QA (`S8-QA-001-W1` unchanged)
- no final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- no Polish→Release retry — PROMPT 761 Polish→Release gate-check `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`
- no Sprint 11 close-out (this Team-QA sign-off is a precondition to a separate close-out decision, not the close-out itself)
- no `S8-QA-001-W1` closure

PROMPT 791 did **not** run `/dev-story`, `/smoke-check`, `/gate-check`, `/release-check`, or `/story-done`. PROMPT 791 did **not** modify production code under `client/`, `server/`, `shared/`, or `tests/`. PROMPT 791 did **not** modify `production/sprint-status.yaml`, `production/sprints/sprint-11.md`, `production/stage.txt`, `.claude/settings.json`, or `reports/`.

---

## Verification (Preflight)

| Step | Result |
|---|---|
| `git fetch origin` | OK |
| `git rev-parse HEAD` | `1617352274531a63f96db1e44ec8a755214ceb7e` |
| `git rev-parse origin/main` | `1617352274531a63f96db1e44ec8a755214ceb7e` (matches HEAD) |
| `git status --short` | ` M .claude/settings.json` (preserved untouched; pre-existing modification) |
| `production/stage.txt` | `Polish` (unchanged) |
| `production/sprint-status.yaml` sprint row | `sprint: 11`, `status: active`, `stage: Polish` |
| PROMPT 761 Polish→Release gate-check FAIL evidence | preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` |
| Sprint 11 smoke evidence | exists at `production/qa/smoke-sprint-11-2026-05-13.md` — verdict `PASS-WITH-WARNINGS` |
| Sprint 11 QA plan | exists at `production/qa/qa-plan-sprint-11.md` |

---

## Sprint 11 Must Have Completion (6/6 `done`)

Verified by reading `production/sprint-status.yaml` `stories:` block. Each Must Have row carries `status: done`, `completed: "2026-05-13"`, and a `/story-done` verdict note with AC verification recorded in-line.

| ID | Title | Status | Closed | Integration commit | `/story-done` prompt | Evidence |
|---|---|---|---|---|---|---|
| S11-DRAG-RUNTIME-RETEST-001 | Drag-and-drop runtime divergence retest, S1–S5 truth-table lock | done | 2026-05-13 | `0fc05c3` (worker) / `3ca1aff` (integration) / `1fd0b95` (`/story-done`) | PROMPT 783 | `production/qa/evidence/sprint-11-drag-runtime-evidence.md`; follow-on diagnostic story `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md` |
| S11-TD-FIXTURE-HAND-UI-ONENTER-001 | `spawn_hand_ui` `OnEnter(InSession)` fixture-cascade repair (6 tests un-`#[ignore]`d) | done | 2026-05-13 | `d7f4103` (integration) / `a8af79a` (`/story-done`) | PROMPT 785 | `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`; pattern doc `docs/architecture/test-fixture-patterns.md` |
| S11-TD-IGNORED-D5-TRIAGE-001 | Owner-named `#[ignore]` triage for 11 D-5 tests from smoke retry-7 | done | 2026-05-13 | `1d96281` (worker integration) / `18758b2` (`/story-done`) | PROMPT 789 | `production/qa/evidence/sprint-11-ignored-d5-triage.md` |
| S11-DOC-HYGIENE-CARRY-001 | ADR-011 `TR-NP-04 → TR-NP-006` + Rule 7 ADR-011 breadcrumb (S10-TD-003 carry) | done | 2026-05-13 | `0d19690` (deliverable) | PROMPT 780 | `docs/architecture/adr-011-reconnect-snapshot.md:173,810`; `design/gdd/network-protocol.md` Rule 7 |
| S11-EVIDENCE-INDEX-CARRY-001 | Sprint 10 evidence aggregator index (S10-N1 carry) | done | 2026-05-13 | `348084b` (deliverable) | PROMPT 781 | `production/qa/evidence/sprint-10-evidence-index.md` |
| S11-ROUTE-READABILITY-CARRY-001 | Friend-game route readability notes (S10-N2 carry) | done | 2026-05-13 | `d3ee8df` (deliverable) | PROMPT 786 | `production/qa/evidence/sprint-10-route-readability-notes.md` |

**Must Have closure**: **6 / 6 done.**

Should Have rows (`S11-TD-FIXTURE-D-RESIDUALS-001`, `S11-HU-PHASE-IDEMPOTENCY-001`, `S11-SERVER-POOL-INIT-LOG-GUARD-001`, `S11-HUD-TIMER-EYEBALL-VISUAL-001`) and Nice to Have rows (`S11-TD-CARGO-DISK-USAGE-001`, `S11-TD-CARGO-PDB-LIMIT-001`, `S11-OPS-ORCHESTRATOR-LOCK-001`, `S11-OPS-GH-CLI-001`, `S11-LOBBY-UX-CONFIRM-STATE-001`, `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`) remain `blocked` — no story files authored, `/story-readiness` pending. This is consistent with the Sprint 11 plan's stated friend-game / Polish scope-cap and is not a regression.

---

## Smoke Check Summary (PROMPT 790, `production/qa/smoke-sprint-11-2026-05-13.md`)

| Command | Result |
|---|---|
| `cargo fmt --check` | PASS (exit 0, no output) |
| `cargo check --workspace` | PASS (`Finished dev profile [optimized + debuginfo] target(s) in 1m 15s`) |
| `cargo test --workspace --tests --no-fail-fast` | **1129 passed / 0 failed / 5 ignored** across 189 binaries |
| `git diff --check` | informational CRLF advisory on `.claude/settings.json` only (NOT a whitespace error) |
| `git diff --cached --check` | empty |

Smoke verdict: `PASS-WITH-WARNINGS`. Warning = the 5 documented D-5 ignored tests with owner-named follow-up slugs (Cluster B in the triage evidence), not a regression. Smoke pass-count delta `+6` vs Sprint 10 retry-7 baseline (1123 → 1129) is attributable to `S11-TD-FIXTURE-HAND-UI-ONENTER-001` un-`#[ignore]`-ing 6 Cluster A fixture tests at integration commit `d7f4103`.

---

## Warning Table — 5 Retained Ignored Tests (Cluster B)

Cross-reference: `production/qa/evidence/sprint-11-ignored-d5-triage.md` Cluster B section (lines 72–90). Each row remains `still ignored` on `origin/main@1617352` with the original PROMPT 750 D-5 owner-named comment preserved and a named follow-up story slug or decision gate.

| # | Test (binary :: name) | Disposition | Follow-up story slug / decision gate |
|---|---|---|---|
| B1 | `tests/integration/board_rendering/ghost_preview_bridge_test.rs :: br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` | `needs-repair-story` — `GhostDragStartEvent` producer system absent in `BoardRenderingPlugin`-only fixture | `S11-TD-FIXTURE-D-RESIDUALS-001` (umbrella) OR new split `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` |
| B2 | `tests/integration/board_rendering/snapshot_spawn_test.rs :: test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` | `needs-design-decision` — `HudPlugin` snapshot.phase bridge fixture gap; decide between expand fixture or relocate assertion | new `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001` |
| B3 | `tests/integration/playable_client/native_operator_controls_test.rs :: test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` | `needs-repair-story` — `ConfirmClass` intent not emitted alongside `SelectClass` after D-1 fix; lobby input system investigation | new `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` |
| B4 | `tests/unit/board_rendering/status_icons_test.rs :: test_cooccupancy_index_two_panics_with_offending_index` | `needs-design-decision` — production `co_occupancy_offset` no longer panics on idx 2; restore guard vs rewrite test | new `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001` |
| B5 | `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs :: shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` | `needs-design-decision` — `ShopAuctionUiEntity` count drift (actual=66, formula expects=57; +9 delta); update formula vs trim spawn | `S11-TD-FIXTURE-D-RESIDUALS-001` (umbrella) OR new split `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` |

**Total ignored**: 5 (Cluster B only) — matches the cargo aggregate. Cluster A (6 tests) was un-`#[ignore]`d and merged at `d7f4103` and is NOT in this table. No undocumented ignored test surfaced.

These 5 warnings are documented follow-ups, not regressions, and do not block Team-QA sign-off at the friend-game / Polish tier. Each follow-up slug requires its own story file + `/story-readiness` in a separate prompt before `/dev-story` can begin — no row authorises immediate implementation.

---

## Carried Conditions Table (preserved unchanged)

| Condition | Status before Sprint 11 | Status after Sprint 11 Team-QA | Notes |
|---|---|---|---|
| `S8-QA-001-W1` — manual / browser two-client GAME_OVER gap | OPEN | **OPEN (unchanged)** | No manual / browser two-client GAME_OVER route executed in Sprint 11. Sign-off does NOT close this. |
| `QA-COND-0005` — Standard-tier accessibility | accepted-risk / friend-game scope | **accepted-risk / friend-game scope (unchanged)** | Sprint 11 explicitly does not pursue Standard-tier accessibility completion. Sign-off does NOT close this. |
| `QA-COND-0006` — playtest / fun-hypothesis validation | accepted-risk / deferred | **accepted-risk / deferred (unchanged)** | Sprint 11 evidence is friend-game / fixture / paperwork; no playtest evidence. Sign-off does NOT close this. |
| 11 ignored D-5 tests (smoke retry-7 W1) | pending owner review | **6 resolved (Cluster A) / 5 retained (Cluster B) with named follow-ups (per triage evidence)** | None silently dropped; 5 remain as warnings under documented follow-up slugs / decision gates. |
| HUD timer eyeball visual check (W2) | deferred | **deferred (unchanged)** | `S11-HUD-TIMER-EYEBALL-VISUAL-001` Should-Have row remains `blocked` (no story file authored). |
| Placeholder / friend-game art scope | `PAW-TD-*-a` accept-risk | **accept-risk (unchanged)** | Placeholder PNGs across PAW-002..PAW-006. No final-art / asset-production-completion claim. |
| `PROMPT 761 Polish→Release` gate-check | FAIL (0/13 required artefacts) | **FAIL preserved (unchanged)** | No retry attempted. Evidence preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. |
| Sprint 10 disposition | `closed-with-conditions` | **`closed-with-conditions` (unchanged)** | Recorded under `sprint_10_closeout:` in `production/sprint-status.yaml`. |

---

## QA Recommendation

**Ready for Sprint 11 close-out with conditions.**

Sprint 11 Must Have scope is complete on `origin/main@1617352`, the workspace test suite passes cleanly (1129/0/5), and the 5 ignored tests are all documented Cluster B retainers under the landed triage evidence. The friend-game / Polish acceptance criteria specified in `production/sprints/sprint-11.md` "Definition of Done for this Sprint" are met for the closure-relevant subset:

- [x] All Must Have tasks completed and integrated.
- [x] All Must Have tasks pass acceptance criteria (per `/story-done` verdicts at commits `1fd0b95`, `a8af79a`, `18758b2`, `0d19690`, `348084b`, `d3ee8df` / `798ecc0`).
- [x] Sprint 11 QA plan exists at `production/qa/qa-plan-sprint-11.md` (PROMPT 774).
- [x] All Logic/Integration stories have passing unit/integration tests (workspace 1129 passed / 0 failed).
- [x] `cargo test` workspace passes without regression vs Sprint 10 close-out baseline (1123 → 1129 = +6 tests un-`#[ignore]`d; 0 failures).
- [x] `/smoke-check sprint` recorded as `PASS-WITH-WARNINGS` with documented warnings (`production/qa/smoke-sprint-11-2026-05-13.md`).
- [x] `/team-qa sprint` produced this `APPROVED WITH CONDITIONS` / `PASS-WITH-WARNINGS` sign-off report.
- [x] No S1 or S2 bugs in delivered Must Have features (verified against Cluster B disposition table; none classify as S1/S2 regressions).
- [x] `production/sprint-status.yaml` reflects every Must Have story as `done`.
- [x] `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006` retain their pre-Sprint-11 disposition.
- [x] `production/stage.txt` remains `Polish`.
- [x] No public release readiness, release-candidate readiness, full playable-client manual QA, full game completion, broad Standard-tier accessibility completion, playtest / fun-hypothesis validation, or full asset / content production is claimed.

**Conditions attached to this approval** (mirroring Sprint 10 sign-off pattern):

- **TQ-S11-C1** — Sprint 11 close-out is a separate orchestrator decision. This Team-QA report does NOT close Sprint 11. Closing requires either (a) pulling each Should Have / Nice to Have row into active scope and closing it, or (b) explicit deferral with a written reason captured in `sprint_11_closeout:` paperwork analogous to `sprint_10_closeout:`.
- **TQ-S11-C2** — The 5 Cluster B ignored tests must be tracked as Sprint 12 (or later) backlog candidates. The triage evidence already names follow-up slugs; producer-side decision required on whether to keep `S11-TD-FIXTURE-D-RESIDUALS-001` as the umbrella row or split per-test. No row authorises immediate implementation under this sign-off.
- **TQ-S11-C3** — `S8-QA-001-W1` remains OPEN. Sign-off does NOT include manual / browser two-client GAME_OVER evidence. Any future release-scope claim must address it separately.
- **TQ-S11-C4** — `QA-COND-0005` (Standard-tier accessibility) and `QA-COND-0006` (playtest / fun-hypothesis validation) remain accepted-risk / deferred. Sign-off does NOT include accessibility or playtest evidence.
- **TQ-S11-C5** — PROMPT 761 `Polish→Release` gate-check `FAIL` remains preserved. Do NOT retry the Polish→Release gate-check until release-scope artefacts (final art, manual-QA sign-off, accessibility completion, playtest evidence) actually exist on `main`. Sign-off does NOT authorise a retry.
- **TQ-S11-C6** — Placeholder / friend-game art scope (`PAW-TD-*-a`) remains accept-risk. Sign-off does NOT include final-art or asset-production-completion evidence.

**Not ready for**: Polish→Release advancement, public release readiness, release-candidate readiness, full game completion, broad accessibility completion, playtest / fun-hypothesis validation, full playable-client manual QA, final-art / asset-production completion, or `S8-QA-001-W1` closure.

---

## Checks Run (read-only)

- `git fetch origin` — OK.
- `git rev-parse HEAD` vs `git rev-parse origin/main` — both `1617352`; matches.
- `git status --short` — only ` M .claude/settings.json` (preserved untouched).
- `git log --oneline -10` — recent commits include `1617352 qa(smoke): Sprint 11 smoke check`, `18758b2 story-done(s11): close ignored D-5 triage`, `1d96281 docs(qa): triage Sprint 11 D-5 ignored tests`, `798ecc0 story-done(s11): close route readability carry`, `a8af79a story-done(s11): close hand-ui OnEnter fixture repair`, `d7f4103 test(hand-ui): repair OnEnter(InSession) fixture cascade per S11-TD-FIXTURE-HAND-UI-ONENTER-001`, `1fd0b95 docs(sprint): /story-done S11-DRAG-RUNTIME-RETEST-001 per PROMPT 783`.
- Read `production/sprint-status.yaml` — Sprint 11 row `status: active`, `stage: Polish`; 6/6 Must Have rows `status: done`.
- Read `production/sprints/sprint-11.md` — Definition-of-Done items mapped above.
- Read `production/qa/qa-plan-sprint-11.md` — Sprint 11 QA plan exists; story-file gate table consistent with current state.
- Read `production/qa/smoke-sprint-11-2026-05-13.md` — verdict `PASS-WITH-WARNINGS`; aggregates `1129 / 0 / 5`.
- Read `production/qa/evidence/sprint-11-ignored-d5-triage.md` — Cluster A (6) + Cluster B (5) = 11; Cluster B owner-named dispositions and follow-up slugs verified.
- Read `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md` — `enter_in_session_via_fixture` helper at `client/src/asset_wiring.rs:420-453`; pattern doc at `docs/architecture/test-fixture-patterns.md`; per-fixture repair table covers 6/6 Cluster A tests.
- Read `production/qa/evidence/sprint-10-evidence-index.md` — Sprint 10 aggregator landed at `8869a54` per the file (effective integration commit on main is `348084b`).
- Read `production/qa/evidence/sprint-10-route-readability-notes.md` (via cross-reference in `sprint-status.yaml`) — coverage of 8 routes with classified observations, non-claims explicit.
- Read `production/gate-checks/gate-polish-release-2026-05-12.md` — `FAIL` verdict preserved (0/13 required artefacts present).
- Read `production/qa/team-qa-sprint-10-2026-05-11.md` — Sprint 10 sign-off `APPROVED WITH CONDITIONS`; pattern adopted here.
- Read `production/stage.txt` — `Polish` (unchanged).
- No `cargo` commands executed by PROMPT 791. Smoke results cited authoritatively from PROMPT 790 evidence file.

---

## Files Changed by PROMPT 791

- `production/qa/team-qa-sprint-11-2026-05-13.md` (this file — NEW)
- `production/session-state/active.md` (banner prepended — paperwork only, no state mutation beyond the banner)
- `production/session-state/codex-orchestrator-state.md` (operating-rules banner refreshed; PROMPT 791 disposition section prepended)
- `reports/PROMPT-791.md` (mandatory final report file; NOT staged or committed)

Explicitly **not** touched by PROMPT 791:

- `.claude/settings.json` (the dirty in-tree modification is preserved as-is, NOT staged, NOT committed)
- `client/`, `server/`, `shared/`, `tests/`
- `production/sprint-status.yaml`
- `production/stage.txt`
- `production/sprints/sprint-11.md`
- `production/qa/qa-plan-sprint-11.md`
- `production/qa/smoke-sprint-11-2026-05-13.md`
- `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
- `production/qa/evidence/sprint-10-evidence-index.md`
- `production/qa/evidence/sprint-10-route-readability-notes.md`
- `production/gate-checks/gate-polish-release-2026-05-12.md`
- any `reports/` file other than `reports/PROMPT-791.md`

---

## Cross-references

- Sprint 11 plan: `production/sprints/sprint-11.md`
- Sprint 11 QA plan: `production/qa/qa-plan-sprint-11.md`
- Sprint 11 smoke: `production/qa/smoke-sprint-11-2026-05-13.md` (PASS-WITH-WARNINGS)
- D-5 triage evidence: `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- Hand-UI OnEnter fixture repair evidence: `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
- Sprint 10 evidence index: `production/qa/evidence/sprint-10-evidence-index.md`
- Sprint 10 route readability notes: `production/qa/evidence/sprint-10-route-readability-notes.md`
- Polish→Release gate-check FAIL (preserved): `production/gate-checks/gate-polish-release-2026-05-12.md`
- Sprint 10 Team-QA pattern reference: `production/qa/team-qa-sprint-10-2026-05-11.md`
- Sprint status authoritative: `production/sprint-status.yaml`
- Stage authoritative: `production/stage.txt` (`Polish`)

---

**End of report — PROMPT 791 / Sprint 11 Team-QA verdict: PASS-WITH-WARNINGS (APPROVED WITH CONDITIONS — ready for Sprint 11 close-out with conditions; NOT a release sign-off).**
