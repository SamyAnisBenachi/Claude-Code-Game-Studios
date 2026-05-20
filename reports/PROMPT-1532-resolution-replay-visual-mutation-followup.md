# PROMPT 1532 — Resolution Replay Visual Mutation Follow-up

**Status**: SHIPPED — per-event visual mutation now applied client-side
for `UnitMoved`, `UnitChangedLane`, `UnitPlaced` (dedupe), `ObjectiveDamage`,
and `ObjectiveDestroyed`, with `tracing::debug!` instrumentation across
the replay applier.

**Worktree**: `D:/_DEV/Work/Claude-Code-Game-Studios/.claude/worktrees/agent-a1456f6d8a8c43f44`
**Branch**: `worker/prompt-1532-resolution-replay-visual-mutation-followup`
**Base**: `5358aed1` (matches the source-of-truth `origin/main@5358aed1`).

---

## What landed

PROMPT 1532 extends the per-group replay cadence shipped by PROMPT 1521
(`apply_resolution_replay_group_system` /
`apply_replay_event_visual_feedback`) so the applier no longer only emits
ephemeral feedback (damage numbers, kill markers) — it now mutates the
client's authoritative-presentation entities (`BoardUnit` / `LaneCell` /
`Transform` / `StandingObjectiveHp`) in lockstep with each `AnimGroup`'s
emergence.

### Code (`client/src/presentation/board_rendering.rs`)

1. **System signature broadened**. `apply_resolution_replay_group_system`
   now also receives:
   - `Res<BoardLayout>` (for `cell_to_world` projection during moves),
   - `Res<BoardRenderingConfig>` (for `hp_bar_visual` recompute),
   - `Query<(Entity, &BoardUnit, &mut LaneCell, &mut Transform), ...>` (write access
     for movement),
   - `Query<(Entity, &StandingObjective, &mut StandingObjectiveHp, Option<&Children>), ...>`
     (write access for objective HP / destruction),
   - `Query<(&mut Transform, &mut Sprite), With<HpBarFill>>` (so the HP-bar
     fill child re-renders immediately, matching `update_hp_bars_system`'s
     existing `apply_hp_fill_visual` contract for units).
   All write queries carry `Without<BoardCellNode>` and disjoint markers
   (`Without<StandingObjective>` / `Without<BoardUnit>`) to satisfy Bevy
   0.18's overlapping-query borrow checker.

2. **`apply_replay_event_visual_feedback`** now handles five new match
   arms in addition to the prior `CombatDamage` / `UnitDied` / `UnitRemoved`:
   - **`UnitMoved`**: looks up the board unit by `unit_id`, snaps its
     `LaneCell` to `(lane, to_cell)` and its `Transform` to
     `board_layout.cell_to_world(lane, to_cell)`.
   - **`UnitChangedLane`**: preserves the unit's current `cell`, mutates
     `lane` and `Transform.y` via the same projection.
   - **`UnitPlaced`**: dedupes against any existing `BoardUnit` carrying
     the same `unit_id` (the placement-reveal path or a board-snapshot
     rebuild has already spawned it). The applier deliberately does **not**
     spawn a missing entity — see "Protocol gap report" below.
   - **`ObjectiveDamage`**: drives `StandingObjectiveHp.hp_current` to the
     server-reported `objective_hp_after`, clamped to `hp_max`. The HP-bar
     fill child is updated in place via `apply_hp_fill_visual`.
   - **`ObjectiveDestroyed`**: zeroes HP via the same path, then despawns
     the matching `StandingObjective` entity (`commands.entity(...).despawn()`).

3. **`apply_unit_lane_cell_mutation` helper** centralises the lane/cell
   mutation logic so both `UnitMoved` and `UnitChangedLane` share the same
   bounds-check + projection + transform-snap + debug-trace path.

4. **`apply_objective_hp_change` helper** centralises objective HP updates
   so `ObjectiveDamage` and `ObjectiveDestroyed` share the same lookup +
   clamp + bar-fill repaint path.

5. **Debug tracing**. Every applier branch emits a `tracing::debug!`
   record on `target: "client::resolution_replay"` carrying
   `group_index`, `sub_step`, `trigger_index`, plus event-specific fields
   (unit_id, prev/new lane+cell, prev/new hp, etc). Mutation order is
   inspectable end-to-end by enabling that target.

No movement tween was introduced — the current commit lands the visible
correctness (the unit ends up at the right cell at group-start time);
tween polish can ride on top of `bevy_tweening` in a later PROMPT without
touching the applier contract.

### Tests (`tests/integration/board_rendering/resolution_replay_visual_mutation_test.rs`, new)

Five focused integration tests, registered as a new `[[test]]` target in
`client/Cargo.toml`:

| Test                                                                | Verifies                                                                                |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `test_unit_moved_event_snaps_board_unit_to_new_cell`                | `UnitMoved` updates `LaneCell` and `Transform.x/y` via `cell_to_world`.                 |
| `test_unit_changed_lane_event_updates_lane_preserves_cell`          | `UnitChangedLane` changes `lane`, preserves `cell`, repositions the transform.          |
| `test_unit_placed_event_dedupes_against_existing_board_unit`        | `UnitPlaced` is a no-op when a matching `BoardUnit` already exists; no double-spawn.    |
| `test_objective_damage_event_updates_standing_objective_hp`         | `ObjectiveDamage` writes `objective_hp_after` to `StandingObjectiveHp.hp_current`.      |
| `test_objective_destroyed_event_despawns_standing_objective`        | `ObjectiveDestroyed` despawns the matching `StandingObjective` entity.                  |

## Validation

Focused tests (no broad suites; per worker rules):

- `cargo check -p client` — clean compile (101 pre-existing warnings, 0 errors).
- `cargo test -p client --test board_rendering_resolution_replay_per_group_cadence_test`
  — **2/2 PASS** (PROMPT 1521 cadence regression check).
- `cargo test -p client --test board_rendering_resolution_replay_visual_mutation_test`
  — **5/5 PASS** (new PROMPT 1532 coverage).
- `git diff --check` — clean.
- Path allowlist:
  - `client/src/presentation/board_rendering.rs` — owned scope.
  - `client/Cargo.toml` — owned scope (test registration).
  - `tests/integration/board_rendering/resolution_replay_visual_mutation_test.rs` — owned scope.
  - No edits to `server/**`, `shared/src/protocol.rs`, shop_auction, hand UI,
    bot code, sprint/session/QA paperwork.

## Protocol gap report

**No protocol gap blocking the work in scope.** The five `ResolutionEvent`
variants this PROMPT consumes (`UnitMoved`, `UnitChangedLane`, `UnitPlaced`,
`ObjectiveDamage`, `ObjectiveDestroyed`) all already carry sufficient data
in the existing `shared/src/protocol.rs` definitions for the visual
mutations landed here.

One deliberately deferred case is worth surfacing — **not a gap, an
authority decision**:

- **`UnitPlaced` for a unit that is NOT yet on the board.** The replay
  payload has `unit_id`, `player`, `lane`, `cell` but **no card identity,
  no stats, no source-class** — i.e. it cannot reconstruct a full
  `BoardUnit` + `BoardUnitOwner` + `BoardUnitCard` + `BoardUnitStats` +
  sprite assembly the way `S2CPlacementReveal` or a `BoardSnapshotEntity`
  rebuild can. The applier therefore emits a `debug!` trace and waits for
  the next snapshot rebuild (`BoardRebuildRequested`) to materialise the
  unit. This is consistent with the existing "client-as-view" rule:
  authoritative entity composition flows from snapshots, not from
  resolution-event reconstruction.

  If a future PROMPT wants `UnitPlaced` to spawn-on-the-fly during
  replay, that is a **protocol decision** (extend `ResolutionEvent::UnitPlaced`
  with `card_id` / `source_class` / `stats`) and belongs in the server-owned
  scope, not in this client-only follow-up.

## Authority / no-claim

The client still receives `S2CResolutionEvent` from the
server-authoritative resolver and only mutates **presentation** state
(`BoardUnit`, `LaneCell`, `Transform`, `StandingObjectiveHp`). No
client-side combat recomputation. No new C2S game-logic messages. No
release-readiness, final-art, broad combat redesign, or sprint closeout
claim.

## Files changed

- `client/Cargo.toml` — register the new `[[test]]` target.
- `client/src/presentation/board_rendering.rs` —
  `apply_resolution_replay_group_system` signature broadened;
  `apply_replay_event_visual_feedback` extended with five new match arms;
  `apply_unit_lane_cell_mutation` and `apply_objective_hp_change`
  helpers added; per-event `tracing::debug!` instrumentation; obsolete
  `find_board_unit_by_id` helper removed (replaced by inline
  `iter()/iter_mut()` finds to satisfy Bevy 0.18 Query lifetime rules).
- `tests/integration/board_rendering/resolution_replay_visual_mutation_test.rs` — new.
- `reports/PROMPT-1532-resolution-replay-visual-mutation-followup.md` — this report.

1532: RESOLUTION-REPLAY-VISUAL-MUTATION-FOLLOWUP: SHIPPED
