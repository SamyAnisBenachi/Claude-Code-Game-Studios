# PROMPT 2056 — Placement Drag/Drop Cursor Legality P0 Repair

Source-of-truth at start: `origin/main@f591614a`.
Worktree: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2056`.
Branch: `work/PROMPT-2056`.

## Problem statement (user live report)

> "Drag/drop is approximate, card does not stay attached to cursor, valid
> cells are unclear, invalid placements are allowed visually then rejected
> late, and there is no useful UI feedback."

## Investigation

The placement drag/drop pipeline already received several rounds of repair
(PROMPT 1210 coord-space split, PROMPT 1410 board-picking window cursor
producer, PROMPT 1456 placement board hit-test, PROMPT 1696 ghost centering,
PROMPT 2036/2046 partial repairs). Cursor follow + valid-cell highlight are
in place:

- `produce_drag_cursor_moved_from_window_system`
  (`client/src/ui/hand/mod.rs:3372–3406`) reads `Window::cursor_position()`
  every frame the drag is active, emits both `world_position` and
  `screen_position`, runs alongside the `Pointer<Move>` producer so the
  cursor keeps streaming once it leaves the UI and enters the board area
  (no picking backend).
- `handle_placement_cursor_moved_system` (`mod.rs:3140–3173`) overwrites
  `ActivePlacementDrag.cursor_world_position` / `cursor_screen_position`
  with the latest sample; system ordering places the producers + handler
  in the `Input` set and the sprite-follow + highlight systems in
  `StateSync`, so the ghost trails the cursor same-tick.
- `sync_hand_drag_sprite_position_system` (`mod.rs:3456–3474`) writes
  `Node.left` / `Node.top` centered on `cursor_screen_position`.
- `minion_highlight_cells` (`mod.rs:5777–5802`) computes the green
  valid-cell set: spawn cell ∩ ¬occupied ∩ ¬objective ∩ ¬already-staged.

The remaining defect this PROMPT scoped was the **legality gap at drop
resolution**. `handle_placement_drag_ended_system` (`mod.rs:3175–3277`)
for `PlacementTargetKind::Minion` did:

```rust
cursor_to_lane_cell(cursor, layout)              // any in-bounds (lane, cell)
    .map(|(lane, cell)| PlayTarget::BoardCell { lane, cell })
```

It accepted **any** in-bounds cell — occupied, objective, out-of-spawn-range,
already-staged — and emitted `HandUiPlacementDropResolved { target: Some(...) }`.
Downstream, `handle_placement_drop_resolved_system` (`mod.rs:3513–3620`)
unconditionally staged the placement, the submit reached the server, and
the server rejected with `PlacementRejectedReason::InvalidTarget`
(`mod.rs:5418–5420`). The user perceived this as: "the card visually
commits and then snaps back from a server round-trip with no
explanation". The green valid-cell overlay was already correct; the drop
gate just did not enforce the same predicate.

## Fix

Single narrow client-side legality gate inserted between
`cursor_to_lane_cell` and the `PlayTarget::BoardCell` constructor for the
`Minion` branch. The predicate is a sibling helper
`is_valid_minion_drop_cell` that mirrors `minion_highlight_cells` so the
green-highlight semantics and the drop-acceptance semantics share the
same definition of "valid". When the release lands on a cell that is not
green-highlighted, the resolved target is `None`, the existing
`handle_placement_drop_resolved_system` returns the card to
`FanSlotState::Active`, the staged-count readout does not advance, no
submit goes out, and the user sees the card snap straight back to the
fan the instant they release.

The other target kinds were already gated (`TargetObj` requires an alive
opponent objective in that lane; `TargetUnit` requires `cursor_over_unit`
to hit a `PlacementTargetUnit`; `Instant` requires the cursor over the
fan plate; `LaneWide` accepts any in-bounds cell which matches GDD intent
— the spell is intentionally lane-wide so any lane hit is legal).
`Minion` was the only gap.

### Files touched (owned scope)

| File | Change |
|---|---|
| `client/src/ui/hand/mod.rs` | Added `is_valid_minion_drop_cell` helper near `staged_minion_cells`. Extended `handle_placement_drag_ended_system` signature with `Res<PlacementBoardView>`, `Res<HandCardCatalog>`, `Res<PendingPlacements>`, and a `board_cells` query. Added `.filter(...)` on the `Minion` branch that invokes the helper. |
| `tests/unit/hand-ui/placement_drop_legality_test.rs` | New focused unit-test file. Five tests exercise each rejection branch + the happy path. |
| `client/Cargo.toml` | Registered the new `[[test]]` entry `hand_ui_placement_drop_legality_test`. |

No edits to `client/src/ui/board_rendering.rs` (forbidden under PROMPT
2055 scope). No edits to production state, sprints, qa, or unrelated
modules.

## Validation

- `git diff --check HEAD -- client/src/ui/hand/mod.rs client/Cargo.toml tests/unit/hand-ui/placement_drop_legality_test.rs` → clean.
- Path allowlist self-review: only files in owned scope (`client/src/ui/hand/**`,
  `tests/unit/**`) were edited; `client/Cargo.toml` `[[test]]` entry is the
  minimum needed to make the test file discoverable and is directly tied to
  the owned-scope change.
- `cargo test -p client --test hand_ui_placement_drop_legality_test` →
  **5 passed, 0 failed** (`finished in 0.05s`).
  - `test_minion_drop_on_valid_spawn_cell_resolves_to_board_cell_target` —
    happy path returns `Some(PlayTarget::BoardCell { lane, cell })`.
  - `test_minion_drop_on_occupied_cell_resolves_to_none` — occupied cell
    rejected client-side.
  - `test_minion_drop_on_objective_cell_resolves_to_none` — objective
    cell rejected client-side.
  - `test_minion_drop_on_out_of_spawn_range_cell_resolves_to_none` —
    `is_spawn_cell` predicate matches the highlight overlay.
  - `test_minion_drop_on_already_staged_minion_cell_resolves_to_none` —
    a `PendingPlacements` entry on the same cell blocks a second Minion
    drop on the same cell during the same turn.
- `cargo test -p client --test hand_ui_placement_drag_highlights_test` →
  **5 passed, 0 failed** (regression baseline preserved).
- `cargo test -p client --test hand_ui_placement_instant_staging_test
  --test hand_ui_placement_submit_core_test --test
  hand_ui_submit_prevalidation_test` → **3 + 7 + 8 = 18 passed, 0
  failed** (instant drag/drop, submit flow, mana prevalidation all
  unaffected).

Total focused tests: **23 passed, 0 failed** across the hand UI suite.

No broad cargo run.

## Scope deltas vs. the PROMPT's full ask

The PROMPT also listed:

1. *"The dragged card/ghost follows the cursor predictably"* — the
   existing pipeline (PROMPT 1210 + 1410 + 1696) is structurally
   correct: window cursor producer runs every frame, cursor handler
   overwrites `ActivePlacementDrag` in the `Input` set, sprite-follow
   reads `cursor_screen_position` in `StateSync` same tick, centered on
   the cursor. No change made; if the user still perceives the ghost as
   lagging, the likely remaining axis is render-target / viewport
   pixel-ratio mismatch on the `Camera::viewport_to_world_2d` path, which
   lives in board layout / camera setup — outside this scope.
2. *"Valid/invalid target cells are visibly differentiated during
   drag"* — valid cells already receive `BoardCellHighlighted` (green
   overlay) every tick during a drag via
   `apply_placement_drag_highlights_system`. A red "invalid hover"
   visual on the cell under the cursor would require editing
   `client/src/ui/board_rendering.rs`, which is the explicitly forbidden
   path under PROMPT 2055 verification. **Precise repair map** for a
   follow-up PROMPT once 2055 lands:
   1. Add `pub struct BoardCellInvalidHover;` next to `BoardCellHighlighted`
      in `client/src/ui/hand/mod.rs` (`mod.rs:872`).
   2. In `apply_placement_drag_highlights_system` (`mod.rs:3890–3991`),
      when `target_kind == Minion` and `cursor_to_lane_cell` resolves to
      a cell that is *not* in `desired_highlights`, mark that cell with
      `BoardCellInvalidHover` and remove the marker from any other cell.
      Add a sibling `sync_board_cell_invalid_hover` helper modelled on
      `sync_board_cell_highlights`.
   3. In `client/src/ui/board_rendering.rs` (out of this PROMPT's scope),
      paint cells carrying `BoardCellInvalidHover` with a red tint /
      border that contrasts with the green `BoardCellHighlighted`
      overlay. (PROMPT 2055 verification path.)

   With the legality gate from this PROMPT already landed, the
   user-visible benefit of the red-hover visual is mostly anticipatory
   ("I can tell I won't be able to drop here before I release") — the
   confusing late-server-rejection cycle is already gone.

3. *"Invalid drops are blocked or explained before a confusing server
   rejection"* — DELIVERED. The release on an illegal cell now snaps
   the card back to the fan in the same tick, never reaches the server,
   never produces a `PlacementRejectedReason::InvalidTarget` toast.

## Risk

- The gate uses the same predicate (`is_spawn_cell` + occupancy +
  objective + `staged_minion_cells`) as the highlight overlay; if the
  two ever drift it is a single helper to update.
- The signature of `handle_placement_drag_ended_system` grew three
  `Res<>` params and one query. All three resources are initialized by
  `HandUiPlugin` before the system runs in production; tests confirm
  the fixture also provides them.
- No behavioural change for the `Instant`, `TargetObj`, `LaneWide`,
  `TargetUnit` branches.

## Test plan for the user

1. **Start the client** (worker branch `work/PROMPT-2056`, full app
   reload). Launch a placement-phase scenario with a Minion card in hand.
2. **Drag the Minion onto an *empty spawn cell*** → green highlight
   shows, release → card commits, staged-count increments. *Expected:
   identical to before this PROMPT.*
3. **Drag the Minion onto an *occupied cell* (a friendly unit already
   there)** → cell stays un-highlighted, release → card snaps back to
   the fan **without** any server round-trip or rejection toast.
   *Expected: card returns to fan instantly.*
4. **Drag the Minion onto an *objective cell*** → same as (3).
5. **Drag the Minion onto a cell *outside spawn range*** → same as (3).
6. **Stage a Minion on cell A, then drag a second Minion onto cell A** →
   same as (3).
7. **Spell cards (Instant / TargetObj / TargetUnit) and Field cards
   (LaneWide)** → behaviour unchanged.

## Files

- `client/src/ui/hand/mod.rs` (+65 lines: signature, filter, helper).
- `client/Cargo.toml` (+8 lines: new `[[test]]` block).
- `tests/unit/hand-ui/placement_drop_legality_test.rs` (new, 281 lines).
- `reports/PROMPT-2056-placement-dragdrop-cursor-legality-p0-repair.md`
  (this file).

2056: PLACEMENT-DRAGDROP-CURSOR-LEGALITY-P0-REPAIR: SHIPPED
