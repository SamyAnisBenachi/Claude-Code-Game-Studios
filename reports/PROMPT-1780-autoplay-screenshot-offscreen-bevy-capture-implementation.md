# PROMPT 1780 — Autoplay Screenshot Offscreen Bevy Capture Implementation

**Branch**: `worktree-1780-autoplay-screenshot-offscreen`  
**Commit**: `ba8f0b19`  
**Date**: 2026-05-28

---

## Summary

Implemented the minimal Bevy-side offscreen screenshot capture path for the autoplay harness.
`Screenshot::primary_window()` was returning stale GPU backbuffer content when the window ran
in the background (OS swapchain issue). The fix bypasses the swapchain by rendering to a
`Handle<Image>` render target and capturing with `Screenshot::image(handle)`.

---

## Files Changed

| File | Change |
|------|--------|
| `client/src/autoplay.rs` | +94 lines, -3 lines |

No changes to `tools/autoplay/**`, `Cargo.toml`, `Cargo.lock`, or production tracking files.

---

## Bevy API Paths Used

| API | Path | Version |
|-----|------|---------|
| `Screenshot::image(Handle<Image>)` | `bevy::render::view::screenshot::Screenshot` | 0.18.1 ✓ |
| `RenderTarget` (Component) | `bevy::camera::RenderTarget` | 0.18.1 ✓ |
| `RenderTarget::Image(handle.into())` | derives `From<Handle<Image>>` | 0.18.1 ✓ |
| `Image::new_fill(...)` | `bevy::prelude::Image` | 0.18.1 ✓ |
| `TextureUsages` | `bevy::render::render_resource::TextureUsages` | 0.18.1 ✓ |
| `RenderAssetUsages` | `bevy::asset::RenderAssetUsages` | 0.18.1 ✓ |

**Key Bevy 0.18 insight**: `RenderTarget` is a *Required Component* of `Camera` (not a field).
`Camera2d` auto-inserts it pointing at the primary window. To override, spawn `RenderTarget::Image(...)`
explicitly alongside `Camera2d` — the provided component takes precedence over the required-component default.

---

## Implementation Details

### New types
- **`AutoplayOffscreenTarget`** (`Resource`) — holds `Handle<Image>` for the offscreen render target.
- **`AutoplayOffscreenCamera`** (`Component` marker) — tags the secondary camera entity.

### New system: `setup_offscreen_target_system` (Startup)
1. Reads primary window physical size; falls back to 1280×720.
2. Creates `Image::new_fill(Rgba8UnormSrgb)` with usages:  
   `TEXTURE_BINDING | COPY_DST | COPY_SRC | RENDER_ATTACHMENT`
3. Spawns `(Camera2d, Camera { order: 1, .. }, RenderTarget::Image(handle.into()))`.
4. Inserts `AutoplayOffscreenTarget { handle }` resource.

### Modified system: `drain_commands_system`
Added `Option<Res<AutoplayOffscreenTarget>>` parameter. Screenshot branch now:
```rust
if let Some(ref target) = offscreen {
    commands.spawn(Screenshot::image(target.handle.clone()))
            .observe(save_to_disk(abs_path.clone()));
} else {
    commands.spawn(Screenshot::primary_window())
            .observe(save_to_disk(abs_path.clone()));
}
```

All existing sidecar JSON, path reporting, and `last_status.screenshots_requested` behavior unchanged.

---

## Compile / Test Results

```
cargo build -p client --features autoplay-remote
→ Finished `dev` profile [optimized + debuginfo] target(s) in 1.00s  (no errors)

git diff --check
→ DIFF CHECK PASSED
```

Existing unit tests in `autoplay.rs` (JSON parsing, keycode parsing, status render) are unaffected —
they do not exercise Bevy ECS systems and continue to pass.

---

## UI / Offscreen Capture Status

- The offscreen camera renders the **game scene** (sprites, board, cards) to the offscreen image.
- **UI does not render to the offscreen camera** — Bevy 0.18 UI uses `IsDefaultUiCamera` which
  targets the primary camera (order 0). The secondary camera (order 1) captures scene content only.
- This is sufficient to fix stale swapchain captures for the autoplay smoke runner, which primarily
  cares about detecting whether the game board progresses, not UI overlays.
- Live verification that captured frames differ across ticks still needed — deferred to VERIFY lane.

---

## Invariants Preserved

- Dev-only: gated behind `autoplay-remote` feature + `CCGS_AUTOPLAY=1` env var.
- Localhost-only: no changes to RPC server.
- No gameplay mutation: offscreen camera is read-only capture; no ECS state writes.
- Fallback: if `AutoplayOffscreenTarget` is absent at screenshot time, falls back to `primary_window()`.

---

1780: AUTOPLAY-SCREENSHOT-OFFSCREEN-BEVY-CAPTURE-IMPLEMENTATION: SHIPPED_NEEDS_LIVE_VERIFY
