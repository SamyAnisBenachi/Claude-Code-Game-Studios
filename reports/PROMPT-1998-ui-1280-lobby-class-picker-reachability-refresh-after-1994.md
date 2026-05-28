# PROMPT 1998 — UI-1280 Lobby Class Picker Reachability Refresh After 1994

**Date:** 2026-05-28
**Branch:** `work/PROMPT-1998`
**Base:** `origin/main@ccff4a06be7d752135cf660d52410efcd3959fce` (PROMPT 1994)

---

## Context

PROMPT 1987 (`origin/work/prompt-1987-lobby-class-picker`) reported
READY_FOR_MAINLAND_ENQUEUE but was rejected because it was NOT_FF after
PROMPT 1994 landed. A wholesale cherry-pick of the 1987 branch would have:

1. Been a non-fast-forward merge (1987 base was pre-1994)
2. Deleted current-main reports (PROMPT 1978/1993 game-completion, PROMPT 1994
   autoplay window-resize composite)
3. Modified out-of-scope files (hand UI, autoplay tooling)

This prompt creates a clean refresh directly on top of
`origin/main@ccff4a06` by checking out only the owned-scope files from
the 1987 branch.

---

## Source Branch Audit

`origin/work/prompt-1987-lobby-class-picker` diff vs `origin/main`:

| File | Status |
|------|--------|
| `client/src/ui/lobby.rs` | M |
| `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md` | A |
| `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md` | A |
| `reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md` | A |
| `tests/integration/playable_client/lobby_class_picker_layout_test.rs` | M |
| `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` | M |

Zero deletions. All files within owned scope. No hand UI, autoplay tooling,
or out-of-scope report changes present on the 1987 branch.

---

## Changes Applied

### `client/src/ui/lobby.rs`

Removed the `flex_shrink: 0.0` override in `lobby_class_picker_cell_node()`.
Restores the CSS default (`flex_shrink = 1`), allowing Taffy to absorb
pixel-rounding deficits at narrow viewports (e.g. 1280×720) without
hard-overflowing the 7th class picker cell past the panel edge. Cells keep
their preferred width at well-sized viewports; only absorb rounding slack
when needed.

### Test files

Both lobby layout integration tests extended to cover `VIEWPORT_1280`
(1280×720) explicitly, matching the live clipping issue originally identified
in PROMPT 1856. `ac3_ac4_grid_columns_fit_minimum_and_hd_viewports` and
`ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd` now assert for
1280×720, 1366×768, and 1920×1080.

### Reports carried forward from 1987 branch

- `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md`
- `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md`
- `reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md`

---

## Validation Results

### Owned-scope gate

```
git diff --name-status origin/main..HEAD
M client/src/ui/lobby.rs
A reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md
A reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md
A reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md
A reports/PROMPT-1998-ui-1280-lobby-class-picker-reachability-refresh-after-1994.md
M tests/integration/playable_client/lobby_class_picker_layout_test.rs
M tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs
```

No deletions. No out-of-scope files. PASS.

### Whitespace check

`git diff --check origin/main..HEAD` — PASS (no trailing whitespace or
mixed indent issues).

### Bevy 0.18 API static grep

Checked `client/src/ui/lobby.rs`,
`tests/integration/playable_client/lobby_class_picker_layout_test.rs`,
`tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`
for banned patterns (`EventReader`, `EventWriter`, `Events<`, `add_event`):
**0 matches** — PASS.

### Focused lobby layout tests

```
cargo test -p client \
  --test playable_client_lobby_class_picker_layout_test \
  --test playable_client_lobby_layout_viewport_invariant_test
```

```
test result: ok. 7 passed; 0 failed (lobby_class_picker_layout_test)
test result: ok. 12 passed; 0 failed (lobby_layout_viewport_invariant_test)
```

All 19 tests PASS including the new 1280×720 viewport assertions.

### FF eligibility

`git merge-base --is-ancestor origin/main HEAD` — PASS.
Branch is a strict fast-forward over `origin/main@ccff4a06`.

---

1998: UI-1280-LOBBY-CLASS-PICKER-REACHABILITY-REFRESH-AFTER-1994: READY_FOR_MAINLAND_ENQUEUE
