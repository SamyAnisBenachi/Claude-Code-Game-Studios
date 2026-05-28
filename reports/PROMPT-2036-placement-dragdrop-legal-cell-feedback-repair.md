# PROMPT 2036 — Placement DragDrop Legal Cell Feedback Repair

Status: PARTIAL (highest-impact gap closed; deeper repairs intentionally deferred).
Branch: `work/PROMPT-2036`
Worktree: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2036`
Base: `origin/main@8f7d3502`

## Root cause identified

The targeting overlay infrastructure (`presentation/board_rendering/targeting_overlay.rs`)
was **already wired** to paint cyan valid-cell rings, a red invalid marker,
and a ghost-unit preview at the current target cell — but its sole input,
`GhostPlacementChanged`, was only emitted **after a drop resolved**
(`handle_placement_drop_resolved_system` in `ui/hand/mod.rs:3588`). During the
actual drag, no live `GhostPlacementChanged` fired, so:

- Valid spawn cells were never highlighted in-flight (UX-002).
- The cursor could hover an invalid cell with zero visual rejection
  before the user committed; only the server's post-confirm
  `S2CPlacementRejected` surfaced the failure (UX-003, P0-014).
- The ghost-unit preview never followed the cursor during the drag.
- The `BoardCellHighlighted` markers computed by
  `apply_placement_drag_highlights_system` were never paired with a live
  cell-tinting sink in `presentation::board_rendering` — but they were
  already useful via the targeting overlay if a target existed.

In short: **the overlay machinery existed; the producer did not.**

## Before / After behaviour

| Behaviour | Before | After |
|---|---|---|
| Cursor over a valid spawn cell during a Minion drag | No visual change | Cyan `TargetingValidRing` halo painted on every valid cell + endpoint ring on the hovered cell + ghost unit preview spawned under cursor |
| Cursor over a board cell outside spawn range | No visual change | Red `TargetingInvalidMarker` painted on the hovered cell |
| Cursor leaves the board while drag is live | Stale (last cell still implied by HUD) | `GhostPlacementChanged { target: None }` is flushed → overlay despawns |
| LaneWide drag | No live target preview | LaneWide ghost wash + lane highlight repaint as cursor moves between lanes |
| TargetUnit / TargetObj / Instant drags | Unchanged (existing pathways own them) | Unchanged — live producer is silent for these kinds to avoid stepping on the existing target-unit hover and fan-plate Instant pathways |
| Drop resolution at release | Sent `target=None` if cursor was off the board | Same; live producer also flushes its own `target=None` so the overlay does not leak after drag end |

## Changes

1. **`client/src/ui/hand/mod.rs`** — added
   `produce_live_ghost_placement_during_drag_system` (registered in the
   `HandUiSystemSet::Input` chain immediately after
   `handle_placement_cursor_moved_system`). The system is read-only over
   `ActivePlacementDrag` / `HandUiMode` / `BoardLayout`, tracks the last
   emitted `(CardId, Option<PlayTarget>)` in `Local`, dedupes repeated
   moves to the same cell, and flushes a `target=None` exactly once when
   the drag ends or the cursor leaves the board after having been on it.
   It deliberately stays silent for TargetUnit / TargetObj / Instant
   target kinds so the existing dedicated pathways
   (`apply_placement_drag_highlights_system`, target-unit hover, fan-plate
   Instant overlay) are not disturbed.

2. **`tests/unit/hand-ui/placement_live_ghost_target_test.rs`** — new
   focused unit test file with four arrange/act/assert tests:
   - `test_live_ghost_emits_board_cell_when_minion_drag_cursor_enters_cell`
   - `test_live_ghost_emits_none_when_minion_drag_cursor_leaves_board`
   - `test_live_ghost_does_not_emit_for_target_unit_drag_kind`
   - `test_live_ghost_dedupes_repeated_moves_to_same_cell`

3. **`client/Cargo.toml`** — added `[[test]]` registration for the new
   binary.

## Validation

- Path allowlist: all edits stay inside `client/src/ui/hand/**` (in
  scope: `client/src/ui/**placement**`), `tests/unit/hand-ui/` (directly
  related placement tests), and the test registration in
  `client/Cargo.toml`. No server, autoplay, sprint-status, or session-state
  paths touched.
- `git diff --check` on owned files: clean. (`.claude/settings.json` has
  pre-existing whitespace warnings but is not part of this PROMPT's
  scope.)
- `cargo check -p client --test hand_ui_placement_live_ghost_target_test`:
  finished successfully (test crate compiles cleanly against the lib).
- `cargo test` for the new binary could not be run to completion: the
  target/ directory exhausted disk space mid-link
  (`rustc-LLVM ERROR: IO failure on output stream: no space on device`).
  The same disk exhaustion will affect any cargo test invocation on this
  host until cleared. Per the prompt's "Run only focused cheap validation;
  defer broad Cargo suites" guidance, the compile-pass gate is treated as
  the validation surface here.
- An unrelated `hud_phase_transitions_test` shows a pre-existing
  `ScoreboardDotState missing field 'known'` compile error in another
  test crate that this PROMPT does not own and does not modify.

## Remaining gaps (out of scope this PROMPT)

The following user-observation items from the unplayable bug-register
(2026-05-28) were not fully closed by this minimal repair and should be
queued as follow-up tickets:

- **"Card does not stick to cursor"** — the cursor-tracking producers
  (`produce_drag_cursor_moved_from_pointer_move_system` +
  `produce_drag_cursor_moved_from_window_system`) are already registered
  and write both `cursor_screen_position` and `cursor_world_position`
  every frame the drag is active. If lag is still observed, the most
  likely remaining cause is camera/viewport scaling drift between WASM
  canvas CSS size and the bevy `Window.physical_resolution`. Needs a
  reproduction on the live build with browser DPR + viewport metrics
  captured.
- **"Dropped card can become stuck"** — pointer-release fallback through
  `mouse_buttons.just_released(MouseButton::Left)` exists at
  `produce_drag_ended_from_pointer_release_system`. If a stuck state is
  still observed, candidate causes are: window focus loss while the
  mouse button is down, or a `HandUiMode` transition mid-drag that the
  drag-end consumer's `Staging` guard skips. A targeted reproduction
  with focus + mode tracing would isolate this.
- **"Invalid cells are visually allowed then confirm rejects"** — this
  PROMPT paints the red `TargetingInvalidMarker` on board cells outside
  the local spawn-range mirror. It does NOT block the drop itself —
  client-authoritative drop gating would require either a new ECS
  predicate (occupancy / owner / mana-affordability projected client-
  side) or a server pre-validation round trip, both of which exceed the
  "interaction and feedback" scope this PROMPT was constrained to.
  Recommend a follow-up PROMPT to add `apply_submit_validation`-style
  pre-flight on the live drag cursor (not just on submit).
- **"No UI feedback explaining placement state"** — the cyan/red overlays
  are a visual affordance, not a text explanation. The
  `S2CPlacementRejected` enum (`shared/src/protocol.rs:309`) already
  distinguishes `WrongPhase / InvalidTarget / SpawnRangeRejected /
  OccupancyRejected / InsufficientMana / OwnerMismatch`. Surfacing this
  enum through a transient banner (reuse `PhaseBannerPlugin` at
  `client/src/ui/phase_banner.rs`) is a small, low-risk follow-up.

2036: PLACEMENT-DRAGDROP-LEGAL-CELL-FEEDBACK-REPAIR: PARTIAL
