# Sprint 11 Hand UI `OnEnter(InSession)` Fixture-Cascade Repair — Evidence

> **Story**: `S11-TD-FIXTURE-HAND-UI-ONENTER-001`
> **Story file**: `production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`
> **Implementation prompt**: PROMPT 779
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\S11-HAND-UI-ONENTER`
> **Branch**: `work/s11-hand-ui-onenter-fixture-repair`
> **Base**: `origin/main@d36bbbd` (PROMPT 774 — Sprint 11 QA plan)
> **Authored**: 2026-05-13

## Diagnosis

`spawn_hand_ui` (`client/src/ui/hand/mod.rs:2738`) early-returns when
`Option<Res<PlaceholderAssets>>` is `None`:

```rust
pub fn spawn_hand_ui(
    mut commands: Commands,
    existing: Option<Res<HandUiEntities>>,
    placeholder: Option<Res<PlaceholderAssets>>,
) {
    if existing.is_some() {
        return;
    }
    let Some(placeholder) = placeholder.as_ref() else {
        return;
    };
    // ... spawn fan_root, fan_slots, grid_slots, reserve_strips, drag_sprite, submit_button ...
}
```

In production, `client::asset_wiring::AssetWiringPlugin` adds
`insert_placeholder_assets` on `OnEnter(ClientState::InSession)`, and
`HandUiPlugin` schedules `spawn_hand_ui.after(insert_placeholder_assets)` so
the resource is present when the spawn runs.

The 6 affected `MinimalPlugins` fixtures added `HandUiPlugin` (which registers
the `OnEnter(InSession)` spawn schedule) but did NOT also add
`AssetWiringPlugin` (which inserts the resource), did NOT add `AssetPlugin`
(without which `insert_placeholder_assets` cannot run because `Res<AssetServer>`
is absent), AND did NOT insert `placeholder_assets_for_tests()` directly.
Result: when the fixture flipped `NextState::<ClientState>::Pending(InSession)`
and pumped `app.update()`, the `OnEnter` schedule ran, `spawn_hand_ui` saw
`placeholder.is_none()`, and silently early-returned. No panic — the fixture
appeared to enter the session cleanly. The cascade surfaced downstream as
"entity not found" / "resource not found" failures in the assertion code:

| Failing query | Symptom |
|---|---|
| `app.world().resource::<HandUiEntities>()` | "Resource HandUiEntities does not exist" |
| `query.iter().find_map(\|(e, idx)\| (idx.0 == slot_index).then_some(e))` | "fan slot should exist" panic |
| `entities.grid_slots.iter().filter_map(...)` | empty vec, equality assertion fails |

### Diagnosis cross-check

Confirmed by running the previously-ignored
`test_reserve_strip_input_does_not_mutate_player_economy_view` from
`tests/integration/presentation/shared_economy_view_test.rs` against the
pre-repair `app_with_hand_ui_in_placement` fixture. The fixture transitions
to `InSession`, `app.update()` runs, but the subsequent `fan_slot(...)`
helper panics with `"fan slot should exist"` — the exact shape predicted
by the diagnosis.

### Classification under the S10-TD-001 cascade taxonomy

This is **Layer 3** of the S10-TD-001 cascade, not Layer 4. Per the story
narrative (`production/epics/playable-client/story-011-hand-ui-onenter-fixture-repair.md`,
Context section):

- Layer 3 (already known at story authoring): "`b92aa97` made `spawn_hand_ui`
  early-return on `Option<Res<PlaceholderAssets>>::None`; `MinimalPlugins`
  fixtures had to insert a `PlaceholderAssets` resource via
  `placeholder_assets_for_tests()`."

The 6 fixtures in this cluster were authored before Layer 3 was documented
and missed the `placeholder_assets_for_tests()` insertion. The repair below
brings them into compliance with the Layer 3 contract. **No Layer 4 gap
(missing run-condition resource, missing schedule registration, missing
`StateTransition` cycle) was surfaced.** This is paperwork-grade fixture
hygiene, not a deeper production-runtime concern.

### Production-runtime impact assessment

**None observed.** The production code path is:

- `AssetWiringPlugin` inserts `PlaceholderAssets` via `insert_placeholder_assets`
  on `OnEnter(InSession)`.
- `HandUiPlugin` registers `spawn_hand_ui.after(insert_placeholder_assets)`.

This ordering guarantees `PlaceholderAssets` exists before `spawn_hand_ui`
runs in production. The bug is fixture-side only. No follow-on production-fix
story is required.

---

## Repair: helper + fixture changes

### Helper (mirroring `placeholder_assets_for_tests()` precedent)

Added `pub fn enter_in_session_via_fixture(app: &mut App)` to
`client/src/asset_wiring.rs` immediately above `AssetWiringPlugin`. The helper:

1. Inserts `placeholder_assets_for_tests()` if absent.
2. Sets `NextState::<ClientState>::Pending(InSession)`.
3. Pumps `app.update()` twice — first applies state transition + runs
   `OnEnter(InSession)` systems (queues spawn commands); second flushes
   deferred commands so downstream queries resolve.

Public surface mirrors `placeholder_assets_for_tests` exactly: plain `pub fn`,
no `#[cfg(test)]` gate (integration test binaries don't see `#[cfg(test)]`
items from a consumed library — the precedent function follows this rule too).

### Per-fixture repair

| # | Test name | File | Fixture fn | Disposition |
|---|---|---|---|---|
| 1 | `test_placement_exit_clears_stale_hand_timer_submit_and_pending_state` | `tests/integration/playable_client/active_loop_ui_state_test.rs` | `hand_app_in_placement` (line 321) | `#[ignore]` removed (line 227); fixture now calls `enter_in_session_via_fixture(&mut app)` in place of manual `NextState + run_update` block — passes |
| 2 | `test_one_card_acquired_fanout_updates_hand_and_draft_pending_purchase` | `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` | `app_in_phase` (line 31) | `#[ignore]` removed (line 90); fixture now calls helper — passes |
| 3 | `test_one_draft_offering_fanout_updates_hand_grid_and_shop_grid` | `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` | `app_in_phase` (line 31) | `#[ignore]` removed (line 72); fixture now calls helper — passes |
| 4 | `test_shop_purchase_reconciles_hand_size_slots_and_shared_economy` | `tests/integration/playable_client/draft_shop_hand_bridge_test.rs` | `app_in_phase` (line 31) | `#[ignore]` removed (line 129); fixture now calls helper — passes |
| 5 | `test_hand_pointer_controls_stage_unstage_and_submit_placement` | `tests/integration/playable_client/native_operator_controls_test.rs` | `hand_app` (line 304) | `#[ignore]` removed (line 214); fixture now calls helper — passes |
| 6 | `test_reserve_strip_input_does_not_mutate_player_economy_view` | `tests/integration/presentation/shared_economy_view_test.rs` | `app_with_hand_ui_in_placement` (line 87) | `#[ignore]` removed (line 67); fixture now calls helper — passes |

### 7th-sibling-test resolution

Story authoring left an open question: PROMPT 762 candidate-backlog capture
references "7x `spawn_hand_ui` not firing on `OnEnter(InSession)`" but smoke
retry-7 enumerates only 6 explicitly Hand UI tagged tests. Diagnosis
confirms: **no 7th sibling test in the workspace shares this root cause.**
Cross-checked via `grep -rn "PROMPT 750 D-5" tests/ --include="*.rs"`. Of the
11 ignored D-5 tests, 6 carry the `spawn_hand_ui not firing` / `HandUiEntities
never spawned` / `fan slots never spawned` owner-comment shape — all 6 are in
this story's primary scope and now un-ignored. The remaining 5 ignored tests
have distinct, unrelated owner-comment causes and stay ignored (see "Sibling
ignored tests" below).

The PROMPT 762 "7x" count was a counting artifact between smoke retry-5 and
retry-7 (the 7th D-5 test that was in the same cluster at retry-5 may have
been deleted or re-dispositioned under PROMPT 759's sweep, leaving 6 by
retry-7). Sprint 11 row `S11-TD-FIXTURE-HAND-UI-ONENTER-001` AC language
should be updated at next sprint-paperwork pass to reflect 6, not 7. **No
correction is made here** — this story does not modify
`production/sprints/sprint-11.md` (AC7 forbids it).

### Sibling ignored tests (out of primary scope per story, verified unchanged)

The 5 remaining workspace-level `#[ignore]`d tests after this repair, each with
its owner-named disposition comment intact and pointing at a distinct
non-`spawn_hand_ui` cause:

| Test | File | Owner-named cause |
|---|---|---|
| `br_8e_board_ghost_pointer_messages_leave_ghost_owned_by_hand_ui` | `tests/integration/board_rendering/ghost_preview_bridge_test.rs:147` | `GhostDragStartEvent` producer system not present in `BoardRenderingPlugin`-only fixture — different cluster, `S11-TD-FIXTURE-D-RESIDUALS-001` |
| `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives` | `tests/integration/board_rendering/snapshot_spawn_test.rs:39` | `HudPlugin` snapshot.phase bridge fixture gap — separate Sprint 11 candidate |
| `test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands` | `tests/integration/playable_client/native_operator_controls_test.rs:105` | `ConfirmClass` intent chain after `SelectClass` — separate Sprint 11 candidate |
| `test_cooccupancy_index_two_panics_with_offending_index` | `tests/unit/board_rendering/status_icons_test.rs:167` | Production `co_occupancy_offset` no longer panics on index 2 — separate Sprint 11 candidate |
| `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes` | `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:25` | `ShopAuctionUiEntity` count drift 57→66 — `S11-TD-FIXTURE-D-RESIDUALS-001` |

All 5 carry their original D-5 owner-comment dispositions and remain blocked
on their respective separate Sprint 11 candidates / triage stories. No silent
`#[ignore]` retention.

---

## Pre/post test counts

### Per-binary (the 4 binaries containing the affected fixtures)

| Test binary | Pre-repair pass count | Pre-repair ignored | Post-repair pass count | Post-repair ignored |
|---|---|---|---|---|
| `playable_client_active_loop_ui_state_test` | 3 passed + 1 ignored | 1 | 4 passed | 0 |
| `playable_client_draft_shop_hand_bridge_test` | 2 passed + 3 ignored | 3 | 5 passed | 0 |
| `playable_client_native_operator_controls_test` | 3 passed + 2 ignored | 2 | 4 passed + 1 ignored | 1 (lobby ConfirmClass — out of scope) |
| `shared_economy_view_test` | 2 passed + 1 ignored | 1 | 3 passed | 0 |
| **Cluster total** | **10 passed + 7 ignored** | **7** | **16 passed + 1 ignored** | **1** |

Note: pre-repair ignored count column for the cluster is 7, post-repair is 1.
The single remaining ignored test (lobby ConfirmClass) is the sibling-cluster
test recorded under "Sibling ignored tests" above — different root cause,
out of this story's scope.

### Workspace (`cargo test --workspace --tests --no-fail-fast`)

| Metric | Pre-repair (smoke retry-7 baseline, `production/qa/smoke-sprint-10-2026-05-12-retry-7.md`) | Post-repair |
|---|---|---|
| Passed | 1123 | 1129 |
| Failed | 0 | 0 |
| Ignored | 11 | 5 |

Delta: **+6 passed**, **-6 ignored** — exactly the 6 un-ignored tests in this
story's primary scope. AC2 satisfied.

---

## Production source diff audit

`git diff origin/main...HEAD -- 'server/src/**' 'shared/src/**'` shows zero
changes.

`git diff origin/main...HEAD -- 'client/src/**'` shows changes confined to
`client/src/asset_wiring.rs` only — exactly the `enter_in_session_via_fixture`
test-only helper that mirrors the `placeholder_assets_for_tests()` precedent
named explicitly in story AC6:

> "GIVEN the diff of the repair commit set, WHEN the diff is filtered to
> `server/src/`, `client/src/` (excluding `#[cfg(test)]`-gated helper fns of
> the `placeholder_assets_for_tests()` precedent), and `shared/src/`, THEN
> zero production-code changes are present."

`placeholder_assets_for_tests` is `pub fn`, not `#[cfg(test)]`-gated — the AC's
"`#[cfg(test)]`-gated" phrase is slightly imprecise; the intent is "test-only
helper fns of this precedent." `enter_in_session_via_fixture` is purely
test-only, has no production callers, and lives next to its precedent. AC6
satisfied per intent.

No production runtime code path is altered. No follow-on production-fix story
is required.

---

## Sprint 11 disposition preservation audit (AC7)

```
git diff origin/main...HEAD -- production/sprint-status.yaml
git diff origin/main...HEAD -- production/sprints/sprint-11.md
git diff origin/main...HEAD -- production/stage.txt
```

All three diffs are empty. Sprint 11 disposition unchanged (`active`, Polish
stage). Sprint 10 disposition unchanged (`closed-with-conditions`). No release
claim, no `Polish->Release` retry, no manual-QA sign-off, no accessibility
completion claim, no playtest validation claim. AC7 satisfied.

---

## Pattern documentation (AC4)

Authored at `docs/architecture/test-fixture-patterns.md` (new). Single page
covering:

- Why the doc exists (silent-skip failure class).
- When to use the helper (any `MinimalPlugins` fixture adding `HandUiPlugin`
  that needs the spawn to actually fire).
- What goes wrong without it (the `spawn_hand_ui` / `placeholder.is_none()`
  early-return chain).
- Helper signature, behavior, pre-conditions.
- Minimal example.
- Side effects (does not also set `RoundPhase`; image handles are
  `Handle::default`).
- Related precedent (`placeholder_assets_for_tests` from S10-TD-001 Layer 3).

Doc cross-links back to this story id and to story-009. AC4 satisfied.

---

## Acceptance criteria sign-off

| AC | Status | Reference |
|---|---|---|
| AC1 — Per-test disposition for the 6 cluster tests | PASS | "Per-fixture repair" table above |
| AC2 — Workspace ignored count drops by N (= 6) OR every remaining ignored test has owner-named disposition | PASS | Workspace pre/post + sibling-test table above |
| AC3 — Reusable fixture helper authored, called from every repaired fixture, no duplicated entry boilerplate | PASS | `client::asset_wiring::enter_in_session_via_fixture`; 4 fixtures call it |
| AC4 — Pattern documentation | PASS | `docs/architecture/test-fixture-patterns.md` |
| AC5 — `cargo test -p client --no-fail-fast` passes for repaired set | PASS | 390 passed, 0 failed, 5 ignored (workspace-level 5 ignored = 5 sibling tests in non-`client` binaries… actually `cargo test -p client` reports only its own binaries; 5 ignored at workspace, 5 also visible from client crate because all 5 sibling tests live in `client`-owned test binaries). All 6 cluster tests un-`#[ignore]`d and pass. |
| AC6 — No production code modified (test-helper exception only) | PASS | Diff confined to `client/src/asset_wiring.rs` adding the helper |
| AC7 — Sprint 11 disposition preserved | PASS | Three diffs empty |
| AC8 — Evidence document slot reserved (this file) | PASS | This file |

---

## Verification commands run

```
cargo test --test shared_economy_view_test                      # 3 passed
cargo test --test playable_client_active_loop_ui_state_test     # 4 passed
cargo test --test playable_client_draft_shop_hand_bridge_test   # 5 passed
cargo test --test playable_client_native_operator_controls_test # 4 passed, 1 ignored (lobby ConfirmClass — out of scope)
cargo test -p client --no-fail-fast                             # 390 passed, 0 failed, 5 ignored
cargo test --workspace --tests --no-fail-fast                   # 1129 passed, 0 failed, 5 ignored
cargo fmt --check                                               # clean
git diff --check origin/main...HEAD                             # clean
```

---

## What this story does NOT claim

This story is **test-fixture / paperwork-grade tech-debt repair**. It does NOT
claim:

- Public release readiness.
- Release-candidate readiness.
- Full playable-client manual QA.
- Full game completion.
- Broad Standard-tier accessibility completion.
- Playtest / fun-hypothesis validation.
- Final-art / asset-production completion.

It does NOT close any S8 / Sprint 9 / Sprint 10 carried condition. It does NOT
modify `production/sprint-status.yaml`, `production/sprints/sprint-11.md`,
or `production/stage.txt`. Sprint 11 remains `active`, stage remains `Polish`.

Sprint 11 row `S11-TD-FIXTURE-HAND-UI-ONENTER-001` flip to `done` and the
matching `/story-done` re-fire are explicitly deferred to a separate prompt
per story Implementation Notes and PROMPT 779 forbidden-action list.
