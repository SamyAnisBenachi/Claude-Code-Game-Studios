# PROMPT 1973 — UI-1280 Lobby Class Picker Reachability Refresh After 1959

**Date:** 2026-05-28
**Branch:** `prompt-1973-lobby-class-picker-reachability`
**Commit:** `ba8349f1`
**Base:** `origin/main@7b259e91` (PROMPT 1972, strict-FF over 7fc1706e PROMPT 1959)

---

## Context

PROMPT 1958 (`origin/prompt-1958-lobby-class-picker-refresh@5739f0f4`) reported
READY_FOR_MAINLAND_ENQUEUE but was found NOT_FF against current `origin/main`
(which had advanced to PROMPT 1959 `7fc1706e` and then PROMPT 1972 `7b259e91`).
Wholesale merging 1958 would have deleted the PROMPT 1972 autoplay signoff-pack
reports. This prompt re-applies only the owned 1958 payload onto a fresh branch
from current `origin/main`.

---

## Changes Applied (file-level transplant from 1958)

### `client/src/ui/lobby.rs`
- Removed hard `flex_shrink: 0.0` override from the class picker cell Node.
- CSS default `flex_shrink: 1` now in effect, allowing Taffy to absorb
  pixel-rounding deficits at narrow viewports (1280×720) without clipping the
  7th cell past the panel edge. Cells retain preferred width at all well-sized
  viewports.

### `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
- Added `VIEWPORT_1280: (f32, f32) = (1280.0, 720.0)` constant.
- Extended `ac3_ac4_grid_columns_fit_minimum_and_hd_viewports` assertion loop
  to include `1280×720` alongside the existing `1366×768` and `1920×1080`
  viewports.

### `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`
- Added `VIEWPORT_1280: (f32, f32) = (1280.0, 720.0)` constant.
- Extended `ac3_ac4_panel_fits_within_viewport_at_minimum_and_hd` assertion
  loop to include `1280×720`.

### `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md`
- Backfilled from stale 1958 branch (was absent from origin/main).

---

## Validation Results

### Required Checks

| Check | Result |
|---|---|
| `git merge-base --is-ancestor origin/main HEAD` | **PASS** |
| `git diff --name-status origin/main..HEAD` — only owned files, no out-of-scope deletions | **PASS** |
| `git diff --check origin/main..HEAD` — no whitespace errors | **PASS** |

### Diff Scope (clean)

```
M  client/src/ui/lobby.rs
A  reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md
M  tests/integration/playable_client/lobby_class_picker_layout_test.rs
M  tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs
```

No deletions. No out-of-scope files touched.

### Cargo Tests

```
playable_client_lobby_class_picker_layout_test:  7 passed / 0 failed
playable_client_lobby_layout_viewport_invariant_test: 12 passed / 0 failed
```

Both test suites **GREEN**. The `ac3_ac4_grid_columns_fit_minimum_and_hd_viewports`
test now covers `1280×720` and passes — confirming the flex_shrink=1 change
allows the 7-column grid to fit within the panel at the narrowest supported
viewport.

### Pre-existing diagnostics (out-of-scope, not introduced by this PR)

101 compiler warnings in `client` lib (deprecated `HudEntity`, `HandUiEntity`,
`ShopAuctionUiEntity` markers, one `E0063` in
`tests/unit/hud/phase_transitions_test.rs`). None are in owned files. Not
introduced by this PR.

---

## Preservation Check

| Commit | Content | Status |
|---|---|---|
| PROMPT 1920 / `1c4981a6` | card-inspect hover glossary | Preserved (rebase base) |
| PROMPT 1957 / `449688dd`+`2bf3960d` | auction tier-border asset binding | Preserved |
| PROMPT 1959 / `7fc1706e` | Krosmaga UI Stage3 reports | Preserved |
| PROMPT 1972 / `7b259e91` | autoplay vsbot signoff-pack reports (1841/1889/1911/1946/1956/1972) | Preserved (rebase base) |

---

1973: UI-1280-LOBBY-CLASS-PICKER-REACHABILITY-REFRESH-AFTER-1959: READY_FOR_MAINLAND_ENQUEUE
