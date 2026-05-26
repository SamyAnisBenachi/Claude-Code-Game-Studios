# PROMPT 1632 — Bot Debug Overlay Minor UX Polish

**Date**: 2026-05-26
**Branch**: main (applied directly; no merge-conflict risk — 3-file touch with no cross-dependencies)
**Status**: SHIPPED

---

## Summary

Three non-GUI polish issues from the PROMPT 1623 UX contract audit. All changes are
minimal doc/constant fixes; no production logic was altered.

---

## Fix 1 — z_layers.rs DEBUG docstring corrected

**File**: `client/src/ui/design_tokens/z_layers.rs`

**Problem**: Both the module-level table and the `DEBUG` const doc said
`"not shipped in release builds"`. That is factually wrong — the overlay is
runtime/env gated (`CCGS_DEBUG_UI=1`), not compile-excluded.

**Change**:
- Table row: updated to `"default-off; runtime-gated via CCGS_DEBUG_UI env var — can be enabled in release with CCGS_DEBUG_UI=1"`.
- Const doc: updated to `"Default-off in release — enabled at runtime via CCGS_DEBUG_UI=1 (see DebugBotOverlayConfig)"`.

No runtime behaviour changes.

---

## Fix 2 — Client render cap aligned to server tail cap (12 → 16)

**File**: `client/src/presentation/debug_bot_overlay.rs`

**Problem**: `DEBUG_BOT_OVERLAY_TAIL_RENDER_CAP = 12` while the server cap
`DEBUG_BOT_DECISION_TAIL_CAP = 16` (in `server/src/feature/bot/debug_push.rs`).
The server sends up to 16 entries; the client silently drops entries 13–16.

**Resolution chosen**: Align client cap upward to 16 (lowest-risk path — single
constant change, no render-logic changes needed, no `+N more` indicator required).

**Change**: `DEBUG_BOT_OVERLAY_TAIL_RENDER_CAP: usize = 12` → `16`.
Updated const doc to reference the server constant to make the alignment intent explicit.

No test assertions reference the old value of 12, so no test updates were needed.

---

## Fix 3 — File-wide `#![allow(dead_code)]` in debug_push.rs

**File**: `server/src/feature/bot/debug_push.rs`

**Assessment**: The file-wide `#![allow(dead_code)]` at line 39 cannot be safely
narrowed or removed without a compiler run. All major items are `pub` and appear
to be wired into production call paths, but the server is a binary crate and Rust
emits dead_code warnings even for `pub` items unreachable from `main`. Without
compiling we cannot enumerate which specific items (if any) would fire.

**Decision**: Leave the attribute in place per task fallback rule.
A follow-up compiler-run verification pass (`cargo check -p server`) would confirm
whether the attribute can be dropped entirely or narrowed to specific constants.

---

## Validation

```
git diff --check
```

Whitespace warnings are exclusively in `.claude/settings.json` (pre-existing,
outside owned scope). The three owned files produce no `--check` warnings.

Focused tests: no cheap unit tests directly test the changed constants by value.
The `render_overlay_body_*` tests in `debug_bot_overlay.rs` verify payload
rendering against payload content — they remain valid at the new cap of 16.
Broad `cargo test` deferred (CI gate handles it).

---

## Files changed

| File | Change |
|---|---|
| `client/src/ui/design_tokens/z_layers.rs` | Docstring: "not shipped in release" → runtime-gated wording (2 sites) |
| `client/src/presentation/debug_bot_overlay.rs` | `TAIL_RENDER_CAP` 12 → 16; updated const doc |
| `server/src/feature/bot/debug_push.rs` | No change (see Fix 3 above) |
| `reports/PROMPT-1632-bot-debug-overlay-minor-ux-polish.md` | This file |

---

1632: BOT-DEBUG-OVERLAY-MINOR-UX-POLISH: SHIPPED
