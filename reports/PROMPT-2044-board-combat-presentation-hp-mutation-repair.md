# PROMPT 2044 — Board / Combat Presentation HP Mutation Repair

Status: SHIPPED
Branch: `work/PROMPT-2044`
Worktree: `D:/_DEV/Work/gcs-app-worktrees/lanesandlies/PROMPT-2044`
Base: `origin/main@135ca0b0` (worktree branched from `a295db2a` after the BUG-02
debounce-race fix; both reachable from current main tip).

## Problem

`reports/PROMPT-2039-board-unit-combat-presentation-audit-repair-map.md` flagged
that `apply_replay_event_visual_feedback` (`client/src/presentation/board_rendering.rs`,
the `CombatDamage` arm around the old line 1728–1754) writes a
`DamageNumberSpawnRequested` toast but **never mutates `BoardUnitStats` on the
defender entity**. Result: during Resolution replay the floating damage number
flashes, but HP-derived presentation (HP labels, future bar fill) keeps the
pre-combat value until the next full snapshot rebuild — making spawned units
appear unaffected by combat for the entire replay window.

Two adjacent silent-failure modes were also called out:

- `spawn_revealed_placement_unit` silently `return`s when
  `visible_unit_cell` is `None` (OOB lane/cell on a reveal payload).
- `spawn_revealed_placement_units` already `warn!`s on missing `BoardLayout`
  / `CardAtlas`, so that path is already diagnosable.

## Fix

`client/src/presentation/board_rendering.rs`

1. **HP mutation on `CombatDamage`.** Added a disjoint
   `Query<&mut BoardUnitStats, (With<BoardUnit>, Without<BoardCellNode>, Without<StandingObjective>)>`
   parameter to `apply_resolution_replay_group_system` and threaded it into
   `apply_replay_event_visual_feedback`. The `CombatDamage` arm now destructures
   `defender_hp_after` and, after locating the defender entity via the existing
   `board_units` query, writes
   `stats.hp_current = defender_hp_after.min(stats.hp_max)`. Clamping against
   `hp_max` defends against a malformed / stale payload pushing the bar above
   100%. The damage-number toast write is preserved and unchanged.
2. **OOB diagnostic on reveal drop.** `spawn_revealed_placement_unit` now emits
   a `warn!` (target `client::placement_reveal`) when `visible_unit_cell`
   returns `None`, surfacing the silent-drop mode named by PROMPT 2039 with
   one log line per dropped unit (unit_id + owner_id + location).

No changes outside this file in `client/src/`.

## Tests

`tests/integration/board_rendering/resolution_combat_feedback_test.rs`

Three new focused tests around the new contract:

- `test_combat_damage_mutates_defender_board_unit_stats_hp` — defender starts
  at 8/8 HP, replay applies `defender_hp_after = 5`, assert `hp_current == 5`
  and `hp_max == 8` (only current mutates).
- `test_combat_damage_zero_damage_does_not_mutate_defender_hp` — shield-blocked
  `damage_amount = 0` early-return must leave `BoardUnitStats` untouched.
- `test_combat_damage_clamps_hp_after_to_hp_max` — `defender_hp_after = 99`
  against `hp_max = 5` must clamp to 5.

Shared helpers refactored: `spawn_board_unit_with_transform` now delegates to a
new `spawn_board_unit_with_stats` so existing tests retain a sensible default
(`5/5 HP`, atk 2) while the HP-mutation tests can pick exact values.

### Run

```text
cargo test -p client --test board_rendering_resolution_combat_feedback_test
...
running 10 tests
test test_combat_damage_with_zero_damage_is_skipped ... ok
test test_combat_damage_clamps_hp_after_to_hp_max ... ok
test test_unit_removed_event_spawns_kill_marker ... ok
test test_combat_damage_for_unknown_defender_is_silently_skipped ... ok
test test_combat_damage_event_emits_damage_number_spawn_request ... ok
test test_combat_damage_zero_damage_does_not_mutate_defender_hp ... ok
test test_unrelated_resolution_events_emit_no_combat_feedback ... ok
test test_unit_died_event_spawns_kill_marker_at_unit_position ... ok
test test_combat_damage_mutates_defender_board_unit_stats_hp ... ok
test test_kill_marker_despawns_after_ttl ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo check -p client` also clean (pre-existing deprecation warnings on
`HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity` are SOURCE-1077-08 carry-over
and out of scope).

## Scope discipline

- Owned files only: `client/src/presentation/board_rendering.rs`,
  `tests/integration/board_rendering/resolution_combat_feedback_test.rs`,
  this report.
- Forbidden paths untouched: `server/src/**`, `client/src/ui/hand/**`,
  `client/src/ui/shop_auction/**`, `production/sprint-status.yaml`,
  `production/session-state/**`, QA paperwork.
- `git diff --check` clean for owned files. `.claude/settings.json` shows a
  pre-existing trailing-whitespace local edit from the session-start hook,
  not authored here and left out of the commit.
- No `cargo test --workspace` run. Focused crate test binary only.

## Out of scope / deferred

- The PROMPT 2039 finding that the `BoardLayout` / `CardAtlas` reveal-abort is
  silent is already covered by `warn!` lines in `spawn_revealed_placement_units`
  (lines ~1262 / ~1267); no new diagnostic added there.
- No HP-bar fill child mutation on unit HP change — bar geometry currently
  rebuilds on snapshot. A follow-up worker can mirror the
  `apply_objective_hp_change` pattern (update HpBarFill children) if a future
  audit shows the unit HP bar visually lags between snapshot rebuilds.
- No combat gameplay closure asserted — this is a presentation repair, not a
  combat-loop verify. Fresh live/snapshot evidence remains required before
  closing the broader BUG register entry.

2044: BOARD-COMBAT-PRESENTATION-HP-MUTATION-REPAIR: SHIPPED
