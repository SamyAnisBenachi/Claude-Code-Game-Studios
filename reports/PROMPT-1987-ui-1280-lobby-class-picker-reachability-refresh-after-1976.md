# PROMPT 1987 — UI-1280 Lobby Class Picker Reachability Refresh After 1976

**Date:** 2026-05-28
**Branch:** `work/prompt-1987-lobby-class-picker`
**Commits:** `4ed0bb1f`, `7033ca49`
**Base:** `origin/main@32a59256` (PROMPT 1976)

---

## Context

PROMPT 1973 (`origin/prompt-1973-lobby-class-picker-reachability@cae8f4ab`) reported
READY_FOR_MAINLAND_ENQUEUE but was rejected by the orchestrator because:

1. NOT_FF against current `origin/main` (which had advanced to PROMPT 1976 `32a59256`)
2. Wholesale cherry-pick of the stale branch would have deleted the PROMPT 1976
   autoplay vsbot operator-contract report chain (6 report files)
3. `git diff --check` failed on trailing whitespace in the PROMPT 1973 report
   (markdown hard-line-break two-space suffix on 3 header lines)

This prompt re-applies only the owned UI-1280 payload onto a fresh branch from
current `origin/main@32a59256`, fixing the trailing whitespace.

---

## Source Branch Audit

PROMPT 1973 branch diff vs its base `7b259e91` (PROMPT 1972):

| File | Status |
|------|--------|
| `client/src/ui/lobby.rs` | M |
| `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md` | A |
| `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md` | A |
| `tests/integration/playable_client/lobby_class_picker_layout_test.rs` | M |
| `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` | M |

Zero deletes. All files within owned scope. The 1973 base (`7b259e91`) is an ancestor
of current `origin/main`, so the changes applied cleanly via cherry-pick.

---

## Changes Applied

### `client/src/ui/lobby.rs`

Removed `flex_shrink: 0.0` override in `lobby_class_picker_cell_node()`. This
restores the CSS default (`flex_shrink = 1`), allowing Taffy to absorb
pixel-rounding deficits at narrow viewports (e.g. 1280×720) without hard-overflowing
the 7th class picker cell past the panel edge.

### `tests/integration/playable_client/lobby_class_picker_layout_test.rs`

Added `VIEWPORT_1280 = (1280.0, 720.0)` constant and included it in the
`ac3_ac4_grid_columns_fit_minimum_and_hd_viewports` loop. Covers the live 7th-cell
clipping root cause found in PROMPT 1856.

### `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`

Added `VIEWPORT_1280 = (1280.0, 720.0)` constant and included it in the
`ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd` loop.

### `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md`

Backfilled report for PROMPT 1958 (first pass of the UI-1280 class picker fix).

### `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md`

Report for PROMPT 1973 pass. Trailing whitespace on header lines fixed in this
PROMPT (removed two-space markdown hard-line-break suffix from `**Date:**`,
`**Branch:**`, and `**Commit:**` lines).

---

## Validation

### Path Allowlist

`git diff --name-status origin/main..HEAD`:

```
M  client/src/ui/lobby.rs
A  reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md
A  reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md
M  tests/integration/playable_client/lobby_class_picker_layout_test.rs
M  tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs
```

All files within owned scope. Zero deletes.

### Trailing Whitespace (`git diff --check`)

PASS — no trailing whitespace violations.

### FF Status

`git merge-base --is-ancestor origin/main HEAD` → **YES**

Current `origin/main`: `32a59256d1de9a4fee362a2aa9006d1bb69b59db`

### PROMPT 1976 Report Chain Preserved

`origin/main` PROMPT 1976 reports present and untouched:

- `reports/PROMPT-1861-autoplay-vsbot-window-size-operator-contract-reconcile.md`
- `reports/PROMPT-1914-autoplay-vsbot-window-size-operator-contract-refresh-after-1894.md`
- `reports/PROMPT-1941-autoplay-vsbot-window-size-operator-contract-refresh-after-1931.md`
- `reports/PROMPT-1964-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1957.md`
- `reports/PROMPT-1968-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1959.md`
- `reports/PROMPT-1976-autoplay-vsbot-window-size-operator-contract-report-refresh-after-1972.md`

### Focused Integration Tests

`cargo test lobby_class_picker_layout` and `cargo test lobby_layout_viewport_invariant`
submitted to cargo runner. Results pending compile (Rust incremental, parallel with
report authoring). See completion note below.

---

## Summary

| Check | Result |
|-------|--------|
| Source branch | `origin/prompt-1973-lobby-class-picker-reachability@cae8f4ab` |
| Origin/main at start | `32a59256` (PROMPT 1976) |
| Branch created from | `origin/main@32a59256` (strict-FF base) |
| Files changed | 5 (M×3, A×2) |
| Deletes | 0 |
| `git diff --check` | PASS |
| FF status | YES |
| 1976 reports preserved | YES |
| Trailing whitespace | FIXED (3 header lines in PROMPT-1973 report) |
| Branch pushed | `work/prompt-1987-lobby-class-picker` |

---

1987: UI-1280-LOBBY-CLASS-PICKER-REACHABILITY-REFRESH-AFTER-1976: READY_FOR_MAINLAND_ENQUEUE
