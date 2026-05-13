# Sprint 11 D-5 `#[ignore]` Triage — Evidence

> **Story**: `S11-TD-IGNORED-D5-TRIAGE-001`
> **Authored**: 2026-05-13 (PROMPT 787)
> **Source-of-truth**: `origin/main@798ecc0` (PROMPT 786 — `/story-done` for
> `S11-ROUTE-READABILITY-CARRY-001`)
> **Scope**: triage / documentation only. Read-only against `origin/main`. No
> test files modified by this prompt; no production code touched.
> **Owner**: qa-lead + per-test owner (named per row).

## Purpose

Owner-named per-test disposition for the **11 D-5 `#[ignore]` tests** surfaced
by Sprint 10 smoke retry-7 W1 (`production/qa/smoke-sprint-10-2026-05-12-retry-7.md`
lines 59-74). Each row classifies the test, names a system/owner, attaches the
landed evidence or proposed follow-on, and records a next action.

Tests resolved by `S11-TD-FIXTURE-HAND-UI-ONENTER-001` (PROMPT 779 worker,
PROMPT 784 integration at `d7f4103`, PROMPT 785 `/story-done` at `a8af79a`) are
**linked**, not re-litigated.

## Verification baseline

Read-only `grep '#\[ignore' tests/ --include='*.rs'` against worktree at
`origin/main@798ecc0` returns **5** owner-named ignored tests, in 5 distinct
files. The 6 cluster tests resolved by Story-011 are no longer `#[ignore]`-tagged
on `main`. Original total of 11 is fully accounted for below: **6 resolved + 5
retained** = **11**.

| Pre-Sprint-11 ignored (retry-7 W1) | Post-Sprint-11 ignored (`origin/main@798ecc0`) | Delta |
|---|---|---|
| 11 | 5 | -6 |

The -6 delta matches the 6 fixture-cascade tests un-`#[ignore]`d by Story-011
(see "Cluster A — Resolved" below). Workspace pass-count delta `+6` at
integration time (1123 → 1129) is recorded in
`production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`.

---

## Cluster A — Resolved by `S11-TD-FIXTURE-HAND-UI-ONENTER-001` (6 tests)

Root cause: `spawn_hand_ui` (`client/src/ui/hand/mod.rs:2738`) early-returns on
`Option<Res<PlaceholderAssets>>::None`. `MinimalPlugins` fixtures added
`HandUiPlugin` but did NOT also seed `PlaceholderAssets`, so the spawn silently
skipped and downstream `HandUiEntities` / fan-slot queries failed. Resolved by
the test-only helper `client::asset_wiring::enter_in_session_via_fixture`
(`client/src/asset_wiring.rs:420-453`) called from each repaired fixture in
place of the ad-hoc `NextState + update` block.

Evidence document:
`production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
(PROMPT 779 worker, integrated at `d7f4103` by PROMPT 784, `/story-done` verdict
at `a8af79a` by PROMPT 785).

Pattern doc: `docs/architecture/test-fixture-patterns.md`.

| # | Test name | Test path | State on `origin/main@798ecc0` | Owner/system | Disposition | Evidence link | Next action |
|---|---|---|---|---|---|---|---|
| A1 | `test_placement_exit_clears_stale_hand_timer_submit_and_pending_state` | `tests/integration/playable_client/active_loop_ui_state_test.rs` | unignored / pass | client gameplay programmer — `HandUiPlugin` fixture cluster | `resolved-by-S11-TD-FIXTURE-HAND-UI-ONENTER-001` | hand-ui onenter fixture evidence, "Per-fixture repair" row 1; integration commit `d7f4103` | none — closed |
| A2 | `test_one_card_acquired_fanout_updates_hand_and_draft_pending_purchase` | `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` | unignored / pass | client gameplay programmer — `HandUiPlugin` fixture cluster | `resolved-by-S11-TD-FIXTURE-HAND-UI-ONENTER-001` | hand-ui onenter fixture evidence, "Per-fixture repair" row 2; integration commit `d7f4103` | none — closed |
| A3 | `test_one_draft_offering_fanout_updates_hand_grid_and_shop_grid` | `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` | unignored / pass | client gameplay programmer — `HandUiPlugin` fixture cluster | `resolved-by-S11-TD-FIXTURE-HAND-UI-ONENTER-001` | hand-ui onenter fixture evidence, "Per-fixture repair" row 3; integration commit `d7f4103` | none — closed |
| A4 | `test_shop_purchase_reconciles_hand_size_slots_and_shared_economy` | `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` | unignored / pass | client gameplay programmer — `HandUiPlugin` fixture cluster | `resolved-by-S11-TD-FIXTURE-HAND-UI-ONENTER-001` | hand-ui onenter fixture evidence, "Per-fixture repair" row 4; integration commit `d7f4103` | none — closed |
| A5 | `test_hand_pointer_controls_stage_unstage_and_submit_placement` | `tests/integration/playable_client/native_operator_controls_test.rs` | unignored / pass | client gameplay programmer — `HandUiPlugin` fixture cluster | `resolved-by-S11-TD-FIXTURE-HAND-UI-ONENTER-001` | hand-ui onenter fixture evidence, "Per-fixture repair" row 5; integration commit `d7f4103` | none — closed |
| A6 | `test_reserve_strip_input_does_not_mutate_player_economy_view` | `tests/integration/presentation/shared_economy_view_test.rs` | unignored / pass | client gameplay programmer — `HandUiPlugin` fixture cluster | `resolved-by-S11-TD-FIXTURE-HAND-UI-ONENTER-001` | hand-ui onenter fixture evidence, "Per-fixture repair" row 6; integration commit `d7f4103` | none — closed |

**Cluster A totals**: 6 of 11 original D-5 ignored tests resolved; 0 follow-up
required.

---

## Cluster B — Retained `#[ignore]` (5 tests, distinct root causes)

Each row carries the original PROMPT 750 D-5 owner-named comment unchanged on
`origin/main@798ecc0`. Each disposition names an owner, a proposed follow-up
story slug, and an explicit decision gate. No row authorises immediate
implementation under `S11-TD-IGNORED-D5-TRIAGE-001`; each follow-up requires
its own story file + `/story-readiness` in a separate prompt before
`/dev-story` can begin.

| # | Test name | Test path:line | State on `origin/main@798ecc0` | Owner/system | Disposition | Evidence link / current comment | Proposed follow-up story slug | Next action / decision gate |
|---|---|---|---|---|---|---|---|---|
| B1 | `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` | `tests/integration/board_rendering/ghost_preview_bridge_test.rs:147` | still ignored | board-rendering test-infra owner — `BoardRenderingPlugin` fixture, `GhostDragStartEvent` producer gap | `needs-repair-story` (fixture expansion: add `HandUiPlugin` pointer-to-drag bridge or scope the assertion to a `HandUiPlugin` fixture) | `#[ignore = "PROMPT 750 D-5 follow-on: GhostDragStartEvent producer system not present in BoardRenderingPlugin-only fixture …"]`; sprint-11 plan rolls this into `S11-TD-FIXTURE-D-RESIDUALS-001` (Should Have) per `production/sprints/sprint-11.md:123` and "Wider Sprint 11 Backlog" entry referencing PROMPT 762 candidate #6 | `S11-TD-FIXTURE-D-RESIDUALS-001` (existing Sprint 11 Should-Have) **or** new `S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001` if separated | author standalone story file or fold into `S11-TD-FIXTURE-D-RESIDUALS-001` story authoring; do not modify test under this triage prompt |
| B2 | `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` | `tests/integration/board_rendering/snapshot_spawn_test.rs:39` | still ignored | board-rendering test-infra owner + HUD plugin owner — `HudPlugin` snapshot.phase bridge fixture gap | `needs-design-decision` (either expand fixture to include `HudPlugin` so the bridge runs, OR relocate the `snapshot.phase -> CurrentClientPhase` assertion into a dedicated HUD test) | `#[ignore = "PROMPT 750 D-5: assertion expects HudPlugin to bridge snapshot.phase -> CurrentClientPhase, but HudPlugin is not in this fixture …"]`; sprint-11 plan flags this as PROMPT 762 candidate #5 under "Wider Sprint 11 Backlog" (`production/sprints/sprint-11.md:194`) | `S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001` (NEW) | author standalone story file; decision gate is "expand fixture vs relocate assertion" — qa-lead should record the decision in the story file before `/dev-story` |
| B3 | `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` | `tests/integration/playable_client/native_operator_controls_test.rs:106` | still ignored | client gameplay programmer (lobby input system) + ux-designer (intent chain) — lobby `ConfirmClass` after `SelectClass` chain | `needs-repair-story` (production lobby input system: investigate why `ConfirmClass` intent is not emitted alongside `SelectClass` after the D-1 fix — input chain stops at `SelectClass`) | `#[ignore = "PROMPT 750 D-5 follow-on: ConfirmClass intent not emitted alongside SelectClass — input chain stops at SelectClass; needs lobby input system investigation (revealed after D-1 fix)"]`; sprint-11 plan flags this as PROMPT 762 candidate #7 under "Wider Sprint 11 Backlog" (`production/sprints/sprint-11.md:190`) | `S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001` (NEW) | author standalone story file; decision gate is "production fix vs test redesign" — likely production fix per owner-comment language ("input chain stops at SelectClass"); do not modify production lobby input under this triage prompt |
| B4 | `test_cooccupancy_index_two_panics_with_offending_index` | `tests/unit/board_rendering/status_icons_test.rs:167` | still ignored | board-rendering owner (production `co_occupancy_offset`) + qa-lead | `needs-design-decision` (production `co_occupancy_offset` no longer panics on index 2; either restore panic guard in production, OR update the test to assert non-panic behaviour) | `#[ignore = "PROMPT 750 D-5: production co_occupancy_offset no longer panics on index 2 — needs design decision: restore panic guard or update test to assert non-panic behavior"]`; sprint-11 plan flags this as PROMPT 762 candidate #3 under "Wider Sprint 11 Backlog" (`production/sprints/sprint-11.md:191`) | `S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001` (NEW) | author standalone story file; **decision gate is binary**: panic-guard restored in production code (with test re-armed `#[should_panic(expected = "unit_index=2")]`) OR test rewritten to assert non-panic behaviour. Resolution **must not** silently delete the `#[should_panic]` invariant without an explicit production-design write-up. |
| B5 | `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` | `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25` | still ignored | shop-auction-ui scaffold owner — `ShopAuctionUiEntity` count drift (actual=66, formula expects=57; +9 delta) | `needs-design-decision` (scaffold owner must reconcile formula vs spawn count: either update the formula to match actual prepooled entities, OR trim the spawn so it matches the formula) | `#[ignore = "PROMPT 750 D-5: ShopAuctionUiEntity count drift — actual=66, formula expects=57 (9 entity delta); needs scaffold owner to either update formula or trim spawn"]`; sprint-11 plan partially folds this into `S11-TD-FIXTURE-D-RESIDUALS-001` (Should Have) per `production/sprints/sprint-11.md:123` and PROMPT 762 candidate #4 (`production/sprints/sprint-11.md:192`) | `S11-TD-FIXTURE-D-RESIDUALS-001` (existing Sprint 11 Should-Have, already names this test) **or** new `S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001` if separated | author standalone story file or fold into `S11-TD-FIXTURE-D-RESIDUALS-001` story authoring; decision gate is "update formula vs trim spawn" — scaffold owner records decision in the story file before `/dev-story` |

**Cluster B totals**: 5 of 11 original D-5 ignored tests remain ignored; 5
follow-up dispositions named; 0 silently dropped.

---

## Roll-up

| Category | Count | Tests |
|---|---|---|
| Resolved by `S11-TD-FIXTURE-HAND-UI-ONENTER-001` | 6 | A1, A2, A3, A4, A5, A6 |
| Retain-ignore-with-owner / needs-repair-story | 2 | B1, B3 |
| Retain-ignore-with-owner / needs-design-decision | 3 | B2, B4, B5 |
| Delete-candidate-with-rationale | 0 | — |
| **Total accounted for** | **11** | matches original retry-7 W1 list |

Original 11 D-5 ignored tests are fully accounted for. None silently dropped.

### Proposed follow-up story slugs

Two slugs are already named in `production/sprints/sprint-11.md`; up to four
are new candidates suggested here. Authoring each story file and running
`/story-readiness` is **out of scope for this triage prompt** — these are
producer decisions that must happen in separate prompts.

1. **`S11-TD-FIXTURE-D-RESIDUALS-001`** — existing Sprint 11 Should-Have row
   (`production/sprints/sprint-11.md:123`). Already names B1 (board ghost-drag
   producer fixture gap) and B5 (shop-auction-ui count drift). If retained as
   the umbrella row, both B1 and B5 dispositions land under it.
2. **`S11-TD-FIXTURE-BOARD-GHOST-DRAG-PRODUCER-001`** (NEW; optional split of B1
   out of `S11-TD-FIXTURE-D-RESIDUALS-001`).
3. **`S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001`** (NEW; covers B2 — PROMPT
   762 candidate #5).
4. **`S11-LOBBY-CONFIRM-CLASS-INTENT-CHAIN-001`** (NEW; covers B3 — PROMPT 762
   candidate #7).
5. **`S11-TD-COOCCUPANCY-PANIC-GUARD-DECISION-001`** (NEW; covers B4 — PROMPT
   762 candidate #3).
6. **`S11-TD-SHOP-AUCTION-UI-COUNT-DRIFT-001`** (NEW; optional split of B5 out
   of `S11-TD-FIXTURE-D-RESIDUALS-001`).

Net new story slugs proposed: **3 mandatory** (items 3, 4, 5) plus **2
optional splits** (items 2, 6) if the producer prefers per-test rows over the
existing `S11-TD-FIXTURE-D-RESIDUALS-001` umbrella. Whether to keep the
residuals row as the umbrella row or split per-test is a producer call deferred
to a separate `/sprint-plan sprint-11 --add ...` prompt.

---

## Checks run

- Read-only grep for `#[ignore` across `tests/` against worktree
  `origin/main@798ecc0` — 5 matches in 5 files, owner-named comments preserved.
- Cross-referenced each Cluster A test against
  `production/qa/evidence/sprint-11-hand-ui-onenter-fixture-evidence.md`
  "Per-fixture repair" table — 6/6 cluster tests confirmed un-`#[ignore]`d at
  `d7f4103`.
- Cross-referenced each Cluster B test against
  `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` lines 59-74 — 5/5
  retained tests confirmed against original retry-7 W1 list.
- Cross-referenced Cluster B follow-up story slugs against
  `production/sprints/sprint-11.md` "Wider Sprint 11 Backlog" (lines 188-198)
  and "Should Have" `S11-TD-FIXTURE-D-RESIDUALS-001` (line 123).
- No `cargo test` runs performed (per task spec — read-only triage).

## Files changed by this prompt

Only `production/qa/evidence/sprint-11-ignored-d5-triage.md` (this file).

No test files modified. No production code modified. No
`production/sprint-status.yaml`, `production/sprints/sprint-11.md`,
`production/stage.txt`, `production/session-state/*`, `.claude/settings.json`,
or `reports/**` touched by this triage prompt.

---

## What this triage does NOT claim

This triage doc is **read-only documentation / planning artefact**. It does NOT
claim:

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged).
- Playtest / fun-hypothesis validation (`QA-COND-0006` unchanged).
- Full playable-client manual QA (`S8-QA-001-W1` unchanged).
- Final-art / asset-production completion (`PAW-TD-*-a` accept-risk unchanged).
- Sprint 11 close-out.
- Closure of any individual Cluster B ignored test — each requires its own
  follow-up story + `/story-readiness` + `/dev-story` in separate prompts.

It does NOT authorise immediate implementation of any follow-up story. It does
NOT flip `S11-TD-IGNORED-D5-TRIAGE-001` to `done` in
`production/sprint-status.yaml` (that paperwork is a separate `/story-done`
prompt). It does NOT modify any test code under `tests/`.

Sprint 11 disposition remains `active` (Polish stage). Sprint 10 disposition
remains `closed-with-conditions`. Stage remains `Polish`.
