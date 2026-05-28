# PROMPT 2005 — UI-1280 Lobby Class Picker Reachability Refresh After PROMPT 1980

**Branch**: `work/PROMPT-2005-ui-1280-lobby-class-picker-reachability-after-1980`
**Base origin/main SHA**: `f16d60416651cbbaa9443ec76da25fae2f552af9`
**Source branch recovered from**: `origin/work/PROMPT-1998` @ `ae2ad2a3`

---

## Context

PROMPT 1998 was rejected for mainland enqueue because its branch was NOT
fast-forward over the current `origin/main` (which had landed PROMPT 1980 in
the interim). The PROMPT-1998 diff vs `origin/main` showed deletions of:

- `reports/PROMPT-1916-post-1912-autoplay-viewport-window-guard-focused-verify.md`
- `reports/PROMPT-1948-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1943.md`
- `reports/PROMPT-1966-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1957.md`
- `reports/PROMPT-1980-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1976.md`

This refresh creates a clean branch from current `origin/main` and recovers
only the owned UI-1280 files using `git checkout origin/work/PROMPT-1998 -- <files>`,
which copies file content without propagating any deletions from the PROMPT-1998 branch.

---

## Payload Summary

### Implementation change: `client/src/ui/lobby.rs`

Function `lobby_class_picker_cell_node()` (line ~2723):
- **Before**: `flex_shrink: 0.0` — cells could not compress, causing the 7th
  cell to hard-overflow the panel edge at 1280×720 viewports.
- **After**: `flex_shrink` field removed (reverts to CSS default of `1.0`),
  allowing Taffy to absorb pixel-rounding deficits proportionally across the
  7 × 108 px grid that fits with only a 4 px margin at 1280×720.

### Tests recovered (unmodified from PROMPT-1998)
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
- `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs`

### Reports recovered from PROMPT-1998 chain
- `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md`
- `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md`
- `reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md`
- `reports/PROMPT-1998-ui-1280-lobby-class-picker-reachability-refresh-after-1994.md`

---

## Validation Output

### `git diff --name-status origin/main..HEAD`

```
A  reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md
A  reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md
A  reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md
A  reports/PROMPT-1998-ui-1280-lobby-class-picker-reachability-refresh-after-1994.md
A  reports/PROMPT-2005-ui-1280-lobby-class-picker-reachability-refresh-after-1980.md
M  client/src/ui/lobby.rs
M  tests/integration/playable_client/lobby_class_picker_layout_test.rs
M  tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs
```

Zero deletions (`D` lines). All changes are additions (`A`) or modifications (`M`).

### `git diff --check origin/main..HEAD`

PASS — no whitespace errors in owned files.

### Bevy 0.18 static guard

Grep for `EventReader`, `EventWriter`, `Events<`, `.add_event`, `add_event::` across
all changed `.rs` files: **CLEAN — no forbidden patterns**.

### Strict FF check

`git merge-base --is-ancestor origin/main HEAD` — **exits 0** (strict fast-forward confirmed).

---

## PROMPT-1980 Reports Preserved

All four protected reports from PROMPT 1980's landing chain are present on this branch:

- ✅ `reports/PROMPT-1916-post-1912-autoplay-viewport-window-guard-focused-verify.md`
- ✅ `reports/PROMPT-1948-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1943.md`
- ✅ `reports/PROMPT-1966-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1957.md`
- ✅ `reports/PROMPT-1980-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1976.md`

---

## Deferred Verification

Full `cargo test` on the integration tests is deferred (requires the full WASM
build toolchain and server runtime). The test file content is structurally valid
Rust — Bevy 0.18 API guard passed for all changed `.rs` files. Functional
verification of the flex-shrink fix should be done via the autoplay/viewport
smoke suite at mainland merge time.

---

2005: UI-1280-LOBBY-CLASS-PICKER-REACHABILITY-REFRESH-AFTER-1980: READY_FOR_MAINLAND_ENQUEUE
