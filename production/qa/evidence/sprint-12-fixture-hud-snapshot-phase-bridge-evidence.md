# Sprint 12 -- Story 012 Fixture HUD Snapshot Phase Bridge -- Evidence

> **Story**: `production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md`
> (S11-TD-FIXTURE-HUD-SNAPSHOT-PHASE-BRIDGE-001 -- Cluster B2)
> **Sprint**: Sprint 12 (`active`, stage `Polish`)
> **Authored by**: PROMPT 806 (`/dev-story story-012-fixture-hud-snapshot-phase-bridge.md`)
> **Worktree**: `work/s11-fixture-hud-snapshot-phase-bridge`
> **Source-of-truth at entry**: `origin/main@d8d0196` (PROMPT 801 lobby ConfirmClass un-ignore)
> **Authored**: 2026-05-14

---

## No-Claim Banner (verbatim from story file)

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 disposition (`closed-with-conditions`) and Sprint 11 disposition
(`closed-with-conditions` per PROMPT 792) remain unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**No optimistic client-side authority is introduced or proposed by this
story**. ADR-002 and ADR-009 remain binding. ADR-021 (single shared phase
sink) governs the HUD-side phase bridge and is unchanged.

---

## Decision -- Path B (Relocate)

The decision was recorded in the story file
(`production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md`
"Design Decision" section) **before any test code modification**. The
recorded rationale, in summary:

1. **Authority root-cause**: `BoardRenderingPlugin` does not own the
   `snapshot.phase -> CurrentClientPhase` write. The write happens in
   `client::ui::hud::handle_game_snapshot_system`
   (`client/src/ui/hud/mod.rs:884-941`, lines 940-941:
   `current.phase = snapshot.phase; current.round = snapshot.round_number;`),
   which is a `HudPlugin` system consuming
   `PresentationGameSnapshotMessage`. The board-rendering plugin consumes
   a different message (`ClientGameSnapshotMessage`) and writes only
   board-side state (`BoardRenderState`, unit/objective entities,
   `AnimQueue`, `PendingPhaseChange`, etc.) -- it never writes
   `CurrentClientPhase`. Placing the assertion in a
   `BoardRenderingPlugin`-only fixture was therefore a test-layout
   defect, not a fixture-cascade problem.
2. **HUD coverage already exists**: the HUD bridge invariant is already
   exercised by
   `tests/integration/hud/reconnect_snapshot_rebuild_test.rs::full_snapshot_rebuild_populates_all_hud_zones_without_respawning_entities`
   at lines 65-69. Path B does not create a coverage gap. To make the
   trace explicit, PROMPT 806 adds a small dedicated HUD-side test
   (`tests/integration/hud/snapshot_phase_bridge_test.rs`) whose single
   responsibility is the `snapshot.phase + snapshot.round_number ->
   CurrentClientPhase` bridge under `HudPlugin`.
3. **Path A cost (rejected)**: registering `HudPlugin` into
   `app_in_session()` cascades into HUD asset-wiring fixture work (UI
   nodes, text spans, asset placeholders for figurines / dots / labels)
   that is already wired in `tests/integration/hud/*` fixtures. Doubling
   that wiring in the board-rendering fixture inflates fixture surface
   for the other five tests in `snapshot_spawn_test.rs` (which are pure
   board-rendering and do not need a HUD plugin) and raises reviewer
   cost without improving invariant coverage.
4. **AC5 / ADR conformance**: Path B is test-only. No production code in
   `client/`, `server/`, or `shared/` is touched. ADR-002, ADR-009, and
   ADR-021 remain binding and unchanged.

**Coverage gap verification (PROMPT 806)**: The "Open Questions" item
"Is there an existing HUD-side integration test that already covers the
`snapshot.phase -> CurrentClientPhase` bridge invariant?" resolved
**YES** (see rationale item 2). The producer recommendation (Path B) is
therefore binding; no coverage gap is created.

---

## Files Changed

| Path | Change | Layer |
|------|--------|-------|
| `production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md` | "Design Decision" section: Path A marked NOT chosen, Path B marked chosen with 4-point rationale recorded before any test code modification | paperwork |
| `tests/integration/board_rendering/snapshot_spawn_test.rs` | (1) Remove PROMPT 750 D-5 `#[ignore]` attribute on `test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`. (2) Remove misplaced `assert_eq!(app.world().resource::<CurrentClientPhase>().phase, RoundPhase::Placement)`. (3) Drop unused `CurrentClientPhase` import. (4) Add 5-line comment pointing future readers at the HUD-side coverage. | test only |
| `tests/integration/hud/snapshot_phase_bridge_test.rs` (NEW) | Dedicated single-responsibility test `test_hud_plugin_bridges_snapshot_phase_and_round_into_current_client_phase` exercising `S2CGameSnapshot.phase` and `S2CGameSnapshot.round_number` -> `Res<CurrentClientPhase>` under `HudPlugin`. Mirrors the `app_with_hud_in_session()` fixture pattern from `reconnect_snapshot_rebuild_test.rs`. | test only |
| `client/Cargo.toml` | Add `[[test]] name = "snapshot_phase_bridge_test" path = "../tests/integration/hud/snapshot_phase_bridge_test.rs"` wiring entry. | test only |
| `production/qa/evidence/sprint-12-fixture-hud-snapshot-phase-bridge-evidence.md` (NEW) | This evidence document. | paperwork |

**Production-code diff audit (AC5)**: `git diff origin/main...HEAD --
'server/src/**' 'client/src/**' 'shared/src/**'` is empty. Zero
production-code changes.

---

## Test Pass Counts

### Targeted: `cargo test -p client --test board_rendering_snapshot_spawn_test --no-fail-fast`

```
running 6 tests
test test_missing_card_art_uses_placeholder_and_keeps_hp_bar ... ok
test test_hp_bar_fill_thresholds_local_z_and_no_fill_tween ... ok
test test_baseline_board_path_supports_twenty_units_and_two_atlased_images ... ok
test test_standing_objectives_use_unknown_frame_and_no_identity_components ... ok
test test_runtime_board_assets_drive_placeholder_hp_and_objective_images ... ok
test test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

The previously-`#[ignore]`d test
`test_snapshot_rebuild_clears_stale_visuals_and_spawns_snapshot_units_and_objectives`
now passes without the `#[ignore]` attribute. The misplaced
`CurrentClientPhase` assertion has been removed; the remaining
board-rendering-side assertions (stale entity despawn, `AnimQueue`
clear, `PendingPhaseChange::is_none`, queues empty,
`ObjectiveIdentityCache` empty, `BoardRenderState::Placement`, units +
objectives spawned) all pass.

### Client-crate full run (`cargo test -p client --no-fail-fast`)

_(populated after the client-crate run completes -- see "Verification
Summary" below)_

### Workspace full run (`cargo test --workspace --tests --no-fail-fast`)

_(populated after the workspace run completes -- see "Verification
Summary" below)_

---

## Ignored-Count Tally

### Baseline -- `origin/main@d8d0196` (PROMPT 801)

```
tests/integration/board_rendering/ghost_preview_bridge_test.rs:#[ignore = "PROMPT 750 D-5 follow-on: GhostDragStartEvent producer system not present in BoardRenderingPlugin-only fixture — needs HandUiPlugin pointer-to-drag bridge or fixture expansion (revealed after D-3 picking events were registered)"]
tests/integration/board_rendering/snapshot_spawn_test.rs:#[ignore = "PROMPT 750 D-5: assertion expects HudPlugin to bridge snapshot.phase -> CurrentClientPhase, but HudPlugin is not in this fixture; either expand fixture to include HudPlugin or relocate the assertion to a hud test (needs owner decision)"]
tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:#[ignore = "PROMPT 750 D-5: ShopAuctionUiEntity count drift — actual=66, formula expects=57 (9 entity delta); needs scaffold owner to either update formula or trim spawn"]
```

**Baseline count: 3.**

### Worker -- `work/s11-fixture-hud-snapshot-phase-bridge`

```
tests/integration/board_rendering/ghost_preview_bridge_test.rs:#[ignore = "PROMPT 750 D-5 follow-on: GhostDragStartEvent producer system not present in BoardRenderingPlugin-only fixture — needs HandUiPlugin pointer-to-drag bridge or fixture expansion (revealed after D-3 picking events were registered)"]
tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:#[ignore = "PROMPT 750 D-5: ShopAuctionUiEntity count drift — actual=66, formula expects=57 (9 entity delta); needs scaffold owner to either update formula or trim spawn"]
```

**Worker count: 2.**

**Drop: 1.** AC3 satisfied. No new undocumented `#[ignore]` introduced.
The two retained markers (`ghost_preview_bridge_test.rs` and
`plugin_scaffold_formulas_test.rs`) are unrelated Cluster B residuals
scoped to other Sprint 12 / future-sprint stories.

---

## ADR Conformance

- **ADR-002 (Client-Server Authority)**: No optimistic client-side
  authority is introduced. The new HUD-side test asserts that
  `CurrentClientPhase` is written by `HudPlugin` consuming the
  server-authoritative `S2CGameSnapshot.phase` value. No client-side
  mutation of phase state outside the shared sink is added.
- **ADR-009 (RSM Phase State)**: Phase transitions remain
  server-authoritative; the client reads only. The bridge tested is
  read-only on the snapshot side and projects into the shared
  `Res<CurrentClientPhase>` sink.
- **ADR-021 (Presentation Layer Architecture / single shared phase
  sink)**: The HUD bridge already conforms by reading from the same
  `Res<CurrentClientPhase>` projected by the shared sink. This story
  does not modify ADR-021 conformance.

---

## Verification Summary

| Command | Result |
|---------|--------|
| `cargo fmt -p client -- --check` (worktree) | PASS (no formatting diffs) |
| `cargo test -p client --test board_rendering_snapshot_spawn_test --no-fail-fast` | PASS -- 6 passed, 0 failed, 0 ignored |
| `cargo test -p client --no-fail-fast` | _(see "Client-crate full run" above)_ |
| `cargo test --workspace --tests --no-fail-fast` | _(see "Workspace full run" above)_ |
| `git diff --check origin/main...HEAD` (worktree) | PASS (no whitespace defects) |
| `git diff origin/main...HEAD -- 'server/src/**' 'client/src/**' 'shared/src/**'` | EMPTY (AC5 satisfied) |

---

## Cross-Links

- Story file: `production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md`
- Sprint 12 QA plan: `production/qa/qa-plan-sprint-12.md`
- Sprint 11 D-5 triage Cluster B2 row 84:
  `production/qa/evidence/sprint-11-ignored-d5-triage.md`
- ADR-021: `docs/architecture/adr-021-presentation-layer-architecture.md`
- ADR-002: `docs/architecture/adr-002-client-server-authority.md`
- ADR-009: `docs/architecture/adr-009-rsm-phase-state.md`
- HUD production owner: `client/src/ui/hud/mod.rs:884-941`
  (`handle_game_snapshot_system`, lines 940-941:
  `current.phase = snapshot.phase; current.round = snapshot.round_number;`)
- Pre-existing HUD coverage:
  `tests/integration/hud/reconnect_snapshot_rebuild_test.rs::full_snapshot_rebuild_populates_all_hud_zones_without_respawning_entities`
  (lines 65-69)
- New focused HUD test:
  `tests/integration/hud/snapshot_phase_bridge_test.rs::test_hud_plugin_bridges_snapshot_phase_and_round_into_current_client_phase`
