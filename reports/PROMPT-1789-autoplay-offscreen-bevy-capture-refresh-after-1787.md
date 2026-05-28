# PROMPT 1789 — Autoplay Offscreen Bevy Capture Refresh (after 1787)

**Date:** 2026-05-28  
**Branch:** `integrate/autoplay-offscreen-capture-1789`  
**Worktree:** `tmpwt-1789`

---

## Summary

Cherry-picked the PROMPT 1780 offscreen Bevy render target payload (`cc2b923d`,
originally authored as `ba8f0b19`) cleanly onto the latest `origin/main`
(`8dadb857`), producing a strict fast-forward-ready branch. Focused compile
passed with exit code 0.

The refresh was required because PROMPT 1784's branch
(`worktree-1784-autoplay-offscreen-integration @ cc2b923d`) was based on
`origin/main@49c7805b`, which is four commits behind the current HEAD
(`8dadb857`) after PROMPT 1785 and PROMPT 1787 landed.

No changes to `client/src/autoplay.rs` were introduced between `49c7805b`
and `8dadb857`, so the cherry-pick landed cleanly with zero conflicts.

---

## Base & Source

| Field | Value |
|---|---|
| Base SHA (origin/main) | `8dadb857` |
| Cherry-pick source | `cc2b923d` (PROMPT 1784 payload = PROMPT 1780 `ba8f0b19`) |
| Cherry-pick result | `e16b24ed` |
| Branch | `integrate/autoplay-offscreen-capture-1789` |
| FF-ready over origin/main | **YES** (`git merge-base --is-ancestor origin/main HEAD` → 0) |

---

## Files Changed

- `client/src/autoplay.rs` — 94 lines added, 3 deleted (identical to original PROMPT 1780 payload)

No other files modified. Owned scope respected (no `tools/autoplay/**`, no
`production/**`, no `Cargo.toml`, no sprint/QA/session files).

---

## Validation

### git diff --check
```
PASSED — zero output (no whitespace errors)
```

### FF-readiness
```
git log --oneline origin/main..HEAD
→ e16b24ed feat(autoplay): PROMPT 1780 — offscreen Bevy render target for screenshot capture

git merge-base --is-ancestor origin/main HEAD
→ exit 0 (fast-forward ready)
```

### Focused compile
```
cargo build -p client --features autoplay-remote
→ Finished `dev` profile [optimized + debuginfo] target(s) in 7m 13s (exit code 0)
```

Warnings: 101 pre-existing deprecation notices from `HudEntity`, `HandUiEntity`,
`ShopAuctionUiEntity` markers (SOURCE-1077-08, tracked separately). Zero new
warnings in `autoplay.rs` itself.

---

## Bevy 0.18 API Review (liv-bevy-018)

| Pattern | Status | Notes |
|---|---|---|
| `MessageWriter<MouseWheel>` | PASS | Correct 0.17+ scroll API; no `EventWriter` |
| `windows.single()` returns `Result` | PASS | Wrapped in `if let Ok(...)` (lines 325, 514) |
| `windows.single_mut()` returns `Result` | PASS | Wrapped in `if let Ok(...)` (line 422) |
| Required Components spawn tuple | PASS | `(Camera2d, Camera { order: 1, .. }, RenderTarget::Image(...))` |
| No `Camera2dBundle` / bundle patterns | PASS | Required Components style only |
| `Screenshot::image(handle)` / `Screenshot::primary_window()` | PASS | Correct 0.18 API |
| `Option<Res<AutoplayOffscreenTarget>>` | PASS | Correct optional resource pattern |
| `RenderAssetUsages::default()` | PASS | Correct import path |
| `TextureUsages` flags | PASS | `TEXTURE_BINDING | COPY_DST | COPY_SRC | RENDER_ATTACHMENT` |
| No `unwrap()` in production paths | PASS | All fallible ops use `if let Ok/Err` |
| No deprecated Bundle patterns | PASS | No `Camera2dBundle`, `SpriteBundle`, etc. |

---

## UI Capture Limitation Assessment

The offscreen camera (`AutoplayOffscreenCamera`, order 1) renders the **game
scene** only. Bevy UI follows `IsDefaultUiCamera`, which targets the primary
window camera (order 0, the default). This means:

- **Game sprites, board, units** → captured by offscreen camera → appear in PNG
- **bevy_ui overlays, HUD, shop panels** → rendered to primary window only → **NOT captured by offscreen path**

This limitation was documented in the original PROMPT 1780 payload commit and in
the code comment at `autoplay.rs:354`. It is **unchanged** by this integration.

**Impact on evidence quality:** Screenshots from the offscreen path will show the
game scene background without UI chrome. This is sufficient to distinguish frames
(non-identical captures) but QA reviewers should note that HUD/shop state is not
visible in the captured PNG. The `CCGS_QA_SNAPSHOT=1` in-game snapshot mechanism
(separate from autoplay screenshots) remains the authoritative source for UI state
evidence.

**Workaround path (not in scope):** Making UI render to the offscreen target would
require either (a) adding `IsDefaultUiCamera` to the offscreen camera or (b) a
post-process blit pass. Deferred per PROMPT 1784 scope decision.

---

## FF-Readiness Proof

```
git log --oneline origin/main..HEAD
→ e16b24ed feat(autoplay): PROMPT 1780 — offscreen Bevy render target for screenshot capture

git merge-base --is-ancestor origin/main HEAD
→ exit 0 (fast-forward ready)
```

One commit ahead of `origin/main@8dadb857`. Merge to main via
`git merge --ff-only` will succeed.

---

## Next Action

- Branch `integrate/autoplay-offscreen-capture-1789` is FF-ready on demand.
- No live GUI smoke was conducted in this prompt — live verification is the
  subject of PROMPT 1788 (foreground title fix path).
- If PROMPT 1788 live verification shows screenshot distinctness is achieved via
  foreground repair alone, this branch may be superseded. If identical captures
  persist, this branch should be merged to main.

---

1789: AUTOPLAY-OFFSCREEN-BEVY-CAPTURE-REFRESH-AFTER-1787: SHIPPED_NEEDS_LIVE_VERIFY
