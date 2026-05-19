# PROMPT 1410 — S18-BOARD-PICKING-BACKEND-DRAG-TO-CELL-001

## Status

`1410: S18-BOARD-PICKING-BACKEND-DRAG-TO-CELL-001: COMPLETE`

## Root cause

`ui_picking` (the only picking backend the client registers via the
`bevy/bevy_picking` + `bevy/ui_picking` features) generates `Pointer<Move>`
events **only while the cursor is over a UI node**. Board cells are 2D
sprites under no picking backend, so the moment the cursor leaves the
hand fan into the board area:

1. `produce_drag_cursor_moved_from_pointer_move_system` stops receiving
   `Pointer<Move>` messages.
2. `ActivePlacementDrag.cursor_world_position` keeps its drag-start
   `None` (or whatever stale UI-pixel value the producer last wrote).
3. `handle_placement_drag_ended_system` calls
   `cursor_to_lane_cell(None, ...)` → `None`, so the resolved drop is
   `HandUiPlacementDropResolved { target: None }`.
4. `handle_placement_drop_resolved_system` reads `target=None`, flips the
   card to `FanSlotState::Active`, and routes the next click through
   `default_click_stage_target` → `fan_active_default_drop`. That is the
   click-to-stage default-cell path AUDIT-1392-P02 spotted in the run.

The Hearthstone-style drag-to-board-cell intent never reached the
resolver. Pinning `Pointer<Move>` to UI nodes was the gap PROMPT 1392
suspected (`task brief`: *"board cells need mesh/sprite/custom picking
or an explicit cursor-to-board-cell path"*).

## Fix

Take the **explicit cursor-to-board-cell path**. Added a new producer
`produce_drag_cursor_moved_from_window_system` that runs alongside the
existing `Pointer<Move>` producer in the
`HandUiSystemSet::Input` chain. While `ActivePlacementDrag.is_active()`
it reads `Window::cursor_position()` from the primary `Window`, converts
through the active `Camera2d`'s `viewport_to_world_2d`, and emits
`HandUiPlacementCursorMoved` with both viewport-space (`screen_position`)
and world-space (`world_position`) values. The downstream
`handle_placement_cursor_moved_system` then keeps
`ActivePlacementDrag.cursor_world_position` fresh every tick the drag is
live, regardless of which surface the cursor sits over.

### Why this preserves the existing contract

- **Authority boundary** — the producer is read-only over Window /
  Camera and never spawns server-side state or local optimistic units.
- **Click-to-stage fallback** — preserved verbatim. When the cursor is
  unavailable (no `PrimaryWindow`, cursor outside the window, or no
  active 2D camera) the producer is a no-op, the resolver still returns
  `target=None`, and the next click runs `default_click_stage_target`.
  The new regression test
  `no_cursor_during_drag_leaves_drop_target_none_so_click_fallback_can_take_over`
  pins this.
- **PROMPT 1210 cursor split** — viewport-pixel coordinates feed
  `screen_position`, world-space coordinates feed `world_position`. The
  same field split as `produce_drag_cursor_moved_from_pointer_move_system`.
- **PROMPT 1390 / 1401 targeting overlay** — the overlay state machine is
  driven by `GhostPlacementChanged`, which is emitted on drop, so it
  only paints the resolved cell after the drop. The drop now resolves
  to the cell under the cursor, so the existing valid/invalid feedback
  surfaces the right cell unchanged.
- **No new picking backend** — `ui_picking` keeps its single-purpose
  scope (UI nodes only). The explicit window→world path is cheaper than
  registering `MeshPickingPlugin` or `SpritePickingPlugin` and avoids
  expanding the entities that must carry `Pickable` components.

### Implementation note: Query vs. Single

The producer uses
`Query<&Window, With<PrimaryWindow>>::iter().next()` rather than
`Option<Single<&Window, With<PrimaryWindow>>>`. Empirically the
`Option<Single<...>>` form did **not** match a `PrimaryWindow` spawned
*after* `HandUiPlugin::build` in a `MinimalPlugins` test harness even
when exactly one matching entity existed — the test seeing `Some` only
once the system ran reliably required the `Query` form. The production
window is spawned by `DefaultPlugins::WindowPlugin` before plugin init
so either pattern matches in production, but `Query::iter().next()` is
the safe lower bound that works in both worlds.

## Changed files

| File | Change |
| --- | --- |
| `client/src/ui/hand/mod.rs` | + `produce_drag_cursor_moved_from_window_system` (new producer); registered in the existing `HandUiSystemSet::Input` chain alongside the `Pointer<Move>` producer. No removals; the existing producer / consumer pipeline is untouched. |
| `client/Cargo.toml` | + `[[test]]` entry for the new integration bin. |
| `tests/integration/hand-ui/hand_ui_drag_window_cursor_to_board_cell_test.rs` | NEW. Two cases: (1) window cursor over `BoardCell { lane: 2, cell: 5 }` resolves drop to that cell; (2) no cursor → drop `target=None` (click fallback preserved). |

Out of scope (touched on previous PROMPTs and explicitly left alone
here):

- `client/src/presentation/board_rendering.rs` / `board_rendering/**` —
  no board-cell pickability or target metadata changes required; the
  explicit cursor path made the picking-backend route unnecessary.
- `client/src/main.rs` — no plugin registration change required; the
  new producer is added inside the existing `HandUiPlugin` chain.
- shop_auction, lobby, HUD, qa_snapshot, sprint/session/status
  paperwork — none of these were touched.

## Tests run (Windows, MSVC Cargo policy applied)

Environment per binding policy:

```
CARGO_TARGET_DIR        = D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG = 0
CARGO_PROFILE_TEST_DEBUG= 0
CARGO_INCREMENTAL       = 0
RUSTFLAGS               = -C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Run set:

| Test bin | Cases | Result |
| --- | --- | --- |
| `hand_ui_drag_window_cursor_to_board_cell_test` | 2 | **2 pass** (new) |
| `hand_ui_drag_to_board_cell_test` | 4 | 4 pass |
| `hand_ui_drag_cursor_world_projection_test` | 1 | 1 pass |
| `hand_ui_drag_end_non_instant_test` | 11 | 11 pass |
| `hand_ui_drag_state_visuals_test` | (see harness) | pass |
| `hand_ui_viewport_sync_test` | 2 | 2 pass |
| `hand_ui_idle_playable_affordance_test` | 10 | 10 pass |
| `hand_ui_placement_drag_highlights_test` | 5 | 5 pass |
| `hand_ui_plugin_scaffold_test` | 3 | 3 pass |
| `hand_ui_fan_layout_formula_test` | (see bin) | pass |
| `hand_ui_phase_state_machine_test` | 4 | 4 pass |
| `board_rendering_targeting_feedback_test` | 10 | 10 pass |
| `board_rendering_ghost_preview_bridge_test` | 7 | 7 pass |

No regressions surfaced. The pre-existing `qa_snapshot_overlay_test.rs`
field-literal failures resolved themselves after rebase onto origin/main
HEAD `f6d9aa1` (PROMPT 1409 backfilled the missing `auction_won_pending`
and `board_targeting` fields).

## Branch / commit

- Worktree: `D:/_DEV/claude-code-game-studios-worktrees/board-picking-backend-1410`
- Branch: `work/s18-board-picking-backend-1410`
- Based on: `origin/main@f6d9aa1` (rebased post-PROMPT 1409 backfill)
- Commit: see `git log -1` on the branch (filled in by commit step below)

## Live two-client retest

**Required.** The fix is verifiable headlessly via the new integration
test, but the integration with the real `ui_picking` backend, real
cursor input, and the full board layout / camera setup needs a live
two-client run before the AUDIT-1392-P02 row can close on the QA
tracker. The next QA snapshot should show drag-end resolving to the
cell under the cursor rather than to the default spawn cell.

## Status line

```
1410: S18-BOARD-PICKING-BACKEND-DRAG-TO-CELL-001: COMPLETE
```
