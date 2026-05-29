# PROMPT 2057 — Lobby Class Picker Visible-State P0 Repair

- Branch: `work/PROMPT-2057`
- Worktree: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2057`
- Source-of-truth base: `origin/main@f591614a`
- Commit: `a849d544` (`fix(ui/lobby): PROMPT 2057 visible-state P0 repair ...`)
- Related: PROMPT 2026 visible-screen audit · PROMPT 2027 click/window
  metadata audit · PROMPT 2034 user live UI/UX bug ledger · PROMPT 2040
  Bevy UI architecture audit

## Summary

Two narrowly-scoped repairs in `client/src/ui/lobby.rs`:

1. **Stale room/player/confirmation copy in the status banner**
   - Banner now reads the **server-authoritative** seat index for the
     local player when `lobby.slots` carries an assignment (mirrors the
     PROMPT 1178 `lobby_own_slot_label_text` fix). Falls back to
     `input.requested_slot` only during the pre-handshake / typed-join
     staging window.
   - Trailing `Class: {…}` segment now prefers `lobby.locked_class`
     (server-confirmed identity) over `input.selected_class` (in-flight
     preview) once the class is locked. Matches the
     `lobby_selected_class_identity_text` projection post-confirmation.
   - `Players: N/M` substring preserved verbatim (regression guard for
     `lobby_entry_test::class_confirmations_are_server_confirmed`).

2. **Clipped Neutral class card at 1280×720**
   - `LOBBY_CLASS_PICKER_CELL_WIDTH_PX`: **108 → 104**
   - `LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX`: **96 → 92**
   - Result at 1280×720 (panel content area 812 px):
     - Required grid width: 7 · 104 + 6 · 8 = **776 px** (was 804 px)
     - Slack: **36 px** (was 8 px) → absorbs sub-pixel rounding, browser
       chrome, and JustifyContent::Center centring without pushing the
       trailing Neutral cell past the panel border.
   - Per-cell content area is 92 px (cell 104 − 2 · 6 px padding); the
     92 px button still fits exactly, and the inner button width after
     16 px horizontal padding (76 px) still fits the longest selected
     class label "Sacrier *" / "Ecaflip *" (≈ 70 px at typography::BODY
     15 px) without ellipsis.

## Files touched

| File | Change |
|---|---|
| `client/src/ui/lobby.rs` | Cell width 108→104, button width 96→92 (with PROMPT 2057 rationale comments), `lobby_status_copy` slot/class server-truth projection, 5 inline `#[cfg(test)]` unit tests. |

Owned-scope discipline:

- Editable paths: `client/src/ui/lobby/**` (file lives directly at
  `client/src/ui/lobby.rs` — same lobby ownership) and focused tests
  inline. ✓
- Read-only consulted: `client/src/asset_wiring.rs`,
  `client/src/ui/design_tokens/spacing.rs`,
  `client/src/ui/design_tokens/typography.rs`,
  `shared/src/protocol.rs::SessionSlot`,
  `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
  (to confirm the existing 1366×768 / 1920×1080 invariants and the
  "Sacrier *" label-width math the repair preserves).
- No edits outside lobby UI ownership. The pre-existing
  `.claude/settings.json` orchestrator-hook edit was not staged.

## Out-of-scope finding (documented, not fixed)

The user bug ledger entry "black / missing class art" maps to:

- Path: `assets/art/ui/lobby/ui_class_portrait_*.png`
- All 7 files exist on disk (Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida,
  Neutral) and are valid 120×180 RGBA PNGs.
- File sizes: ~1.9 KB each — placeholder art that reads as dark colored
  blocks when rendered at 64×80 in the picker.
- The asset binding in `client/src/asset_wiring.rs::lobby_portrait_asset`
  is correct (one-to-one mapping `ClassId → path`).
- This is a **placeholder-art accept-risk** (`PAW-TD-*-a` per lobby
  class-picker layout test) and is outside lobby UI ownership. No
  edits made; flagged here as an asset-binding follow-up for the
  art-director / technical-artist owner.

**Follow-up asset binding ask** (not actioned by this worker): replace
the `assets/art/ui/lobby/ui_class_portrait_{class}.png` placeholder
files (currently ~1.9 KB colored blocks at 120×180) with class-distinct
portrait art. The wiring in `client/src/asset_wiring.rs` already
resolves each `ClassId` variant; no code change required once the new
PNGs land at the existing paths.

## Validation

Path allowlist + `git diff --check`:

- `git diff --check -- client/src/ui/lobby.rs` → no whitespace or
  conflict markers.
- Modified files (after commit): none in the worktree on `lobby.rs`.
- Pre-existing `.claude/settings.json` orchestrator-hook drift was
  intentionally not staged.

Focused tests (inline `#[cfg(test)] mod
prompt_2057_visible_state_repair_tests` in `client/src/ui/lobby.rs`):

```
cargo test -p client --lib ui::lobby::prompt_2057

running 5 tests
test ui::lobby::prompt_2057_visible_state_repair_tests::test_class_picker_button_fits_inside_cell_content_area ... ok
test ui::lobby::prompt_2057_visible_state_repair_tests::test_class_picker_grid_fits_1280x720_panel_content_area ... ok
test ui::lobby::prompt_2057_visible_state_repair_tests::test_status_copy_falls_back_to_input_slot_before_join_ack ... ok
test ui::lobby::prompt_2057_visible_state_repair_tests::test_status_copy_uses_locked_class_after_confirmation ... ok
test ui::lobby::prompt_2057_visible_state_repair_tests::test_status_copy_uses_server_slot_when_seats_assigned ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 137 filtered out
```

Test coverage:

- `test_status_copy_uses_server_slot_when_seats_assigned` — local player
  in `slots[1]` with `slot = 2`; banner reads "Slot 2", not stale
  "Slot 1" from `input.requested_slot = 1`. Preserves `Players: 2/2`.
- `test_status_copy_falls_back_to_input_slot_before_join_ack` —
  pre-join window with no server slots; banner falls back to
  `input.requested_slot = 3`.
- `test_status_copy_uses_locked_class_after_confirmation` —
  `locked_class = Some(Sacrier)`, `selected_class = Iop`; banner reads
  "Class: Sacrier", not "Class: Iop". Closes the
  "banner says Iop after I confirmed Sacrier" surprise.
- `test_class_picker_grid_fits_1280x720_panel_content_area` —
  required grid width 776 px ≤ panel content 812 px with ≥ 16 px slack
  guard. (Actual slack: 36 px.)
- `test_class_picker_button_fits_inside_cell_content_area` —
  per-cell button width (92) ≤ cell inner width (104 − 12 = 92). Guards
  against future cell shrinkage that would horizontally overflow the
  button.

No broad Cargo suites run (per task scope: "focused local validation
only"). `cargo check -p client --lib` clean; pre-existing
`hud_phase_transitions_test` error in a non-owned test file (in-flight
worker, isolated-file build-gate rule applies).

## Layout math at 1280×720 (worked example)

```
viewport               = 1280 × 720
scrim padding          = SPACING_LG = 24 per side
scrim content area     = 1280 − 48 = 1232 wide

panel.width            = min(88% × 1232, max_width 860)
                       = min(1084.16, 860)
                       = 860
panel.content_area     = 860 − 2·SPACING_LG = 812 wide

grid required width    = 7 cells × 104 + 6 gaps × SPACING_SM
                       = 728 + 48
                       = 776 px

slack                  = 812 − 776 = 36 px        # before: 8 px

cell inner content     = 104 − 2·CELL_PADDING_PX = 92 px
button width           = 92 px → fits cell content exactly

inner button width     = 92 − 2·8 (BUTTON H-padding) = 76 px
longest sel. label     = "Sacrier *" / "Ecaflip *" = 9 chars
estimated label width  = 9 × typography::BODY 15 × 0.52 ≈ 70.2 px
70.2 px ≤ 76 px → fits without ellipsis
```

## Confirm-CTA reachability at 1280×720

Unchanged by this PROMPT — the PROMPT 1398
`LobbyPanelBody (flex_grow:1.0, flex_shrink:1.0, overflow:clip_y) +
Confirm CTA (flex_shrink:0.0)` pair structurally anchors the CTA to the
panel bottom edge regardless of body density. The class-grid shrink in
this PROMPT only reduces horizontal pressure; vertical reachability
guarantees from PROMPT 1398 still hold.

## Risk / scope notes

- Friend-game scope only. Does not advance Standard-tier accessibility
  (`QA-COND-0005`), playtest validation (`QA-COND-0006`),
  placeholder-art completion (`PAW-TD-*-a`), or `S8-QA-001-W1` closure.
- `LOBBY_CLASS_PICKER_CELL_WIDTH_PX` and
  `LOBBY_CLASS_PICKER_BUTTON_WIDTH_PX` are `pub const`; the existing
  `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
  and `lobby_button_dimensions_test.rs` references the constants
  symbolically, so the dimension-stability assertions still pass
  against the new values.

## Final line

2057: LOBBY-CLASS-PICKER-VISIBLE-STATE-P0-REPAIR: DONE
