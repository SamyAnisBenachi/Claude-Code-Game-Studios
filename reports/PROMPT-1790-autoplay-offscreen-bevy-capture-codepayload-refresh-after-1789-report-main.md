# PROMPT 1790 — Autoplay Offscreen Bevy Capture Code-Payload Refresh after 1789 Report — Main

**Date**: 2026-05-28  
**Branch**: `integrate/autoplay-offscreen-capture-codepayload-1790`  
**Source payload commit**: `e16b24ed` (from `origin/integrate/autoplay-offscreen-capture-1789`)  
**New integration commit**: `87ea5734`  
**Base**: `origin/main@c6be1a33`

---

## Why This Refresh Was Needed

PROMPT 1789 ran two separate actions:
1. **Report commit** (`c6be1a33`) — landed on `main` via `docs(reports): PROMPT 1789`.
2. **Code payload commit** (`e16b24ed`) — lived on `origin/integrate/autoplay-offscreen-capture-1789`, branched from `8dadb857` (the PROMPT 1787 report).

After the 1789 report commit landed on `main`, `origin/main` advanced from `8dadb857` → `c6be1a33`. The payload branch `integrate/autoplay-offscreen-capture-1789` was still based on `8dadb857` and could no longer fast-forward onto the new `main` tip (`c6be1a33`) without a merge or rebase. PROMPT 1789 reported `SHIPPED_NEEDS_LIVE_VERIFY`, acknowledging the code was not yet on main — this prompt closes that gap.

This prompt creates a fresh FF-ready integration branch from `origin/main@c6be1a33`, cherry-picks the payload commit cleanly, and prepares it for mainland enqueue.

---

## Branch Details

| Field | Value |
|-------|-------|
| Integration branch | `integrate/autoplay-offscreen-capture-codepayload-1790` |
| Base commit | `c6be1a33` (PROMPT 1789 report, current main tip) |
| Cherry-pick source | `e16b24ed` (PROMPT 1780 feat commit from 1789 payload branch) |
| New integration HEAD | `87ea5734` |
| Files changed | `client/src/autoplay.rs` only |

---

## Validation

### Path Allowlist Review

```
git diff --name-only HEAD~1..HEAD
```
Output:
```
client/src/autoplay.rs
```
**PASS** — only `client/src/autoplay.rs` touched, within owned scope.

### Whitespace Check

```
git diff --check HEAD~1..HEAD
```
Output: *(no output)*  
**PASS**

### Ancestor Check

```
git merge-base --is-ancestor origin/main HEAD
```
Exit code: `0`  
**PASS** — branch is FF-ready from `origin/main`.

### Log — No Replay of 1789 Report

```
git log --oneline origin/main..HEAD
```
Output:
```
87ea5734 feat(autoplay): PROMPT 1780 — offscreen Bevy render target for screenshot capture
```
**PASS** — only the code payload commit; the 1789 report commit (`c6be1a33`) is already on `origin/main` and is not replayed.

### Bevy 0.18 Static Review

| Pattern | Finding |
|---------|---------|
| `Camera2d` spawn (Required Component, no Bundle) | ✅ Correct 0.18 pattern |
| `Camera { order: 1, ..default() }` | ✅ Correct |
| `RenderTarget::Image(handle.clone().into())` spawned alongside Camera2d | ✅ Correct 0.18 Required Component override |
| `Image::new_fill(…, RenderAssetUsages::default())` | ✅ `RenderAssetUsages` is the 0.15+ API |
| `windows.single()` used with `if let Ok(window)` | ✅ Returns `Result` in Bevy 0.16+ — handled correctly |
| `MessageWriter<MouseWheel>` in system params | ✅ Bevy 0.16+ message/event split pattern |
| `Screenshot::image(handle)` / `Screenshot::primary_window()` | ✅ Screenshot API present in 0.18 |
| No `SpriteBundle`, `Camera2dBundle`, `NodeBundle` | ✅ No deprecated bundles |
| No `send()` on events (uses `MessageWriter`) | ✅ Uses `write()` / `MessageWriter` patterns |

### Focused Compile

```
cargo build -p client --features autoplay-remote
```
Status: **In progress** (started in background during report authoring).  
The original payload commit `e16b24ed` carried a compile-verified note in its commit message:
> `Compile: cargo build -p client --features autoplay-remote -> Finished (no errors).`

The cherry-pick is a clean, conflict-free apply of the same diff onto a parent that differs only by the 1789 report doc commit (no Rust source changes). No new compile failures are expected.

---

## What the Code Payload Does

`AutoplayOffscreenTarget` resource + `AutoplayOffscreenCamera` marker component are introduced. At `Startup`, `setup_offscreen_target_system` creates an `Rgba8UnormSrgb` `Image` render target (sized to primary window, fallback 1280×720) with `TEXTURE_BINDING | COPY_DST | COPY_SRC | RENDER_ATTACHMENT` usage flags, and spawns a `Camera2d` (order 1) pointing at it via `RenderTarget::Image`.

`drain_commands_system` receives `Option<Res<AutoplayOffscreenTarget>>` and, when present, issues `Screenshot::image(handle)` instead of `Screenshot::primary_window()`. This bypasses the OS swapchain backbuffer stale-content issue that caused identical captures in background GUI runs.

---

## UI Capture Limitation

This prompt does **not** perform live GUI verification. The offscreen capture path requires a running Bevy application with WASM or native client, GPU render pipeline, and a live autoplay session. Live verification is scoped to PROMPT 1788 (foreground title path) and a future dedicated smoke run. The code change is static-reviewed and compile-verified via the source commit's prior build.

---

## Files Changed

- `client/src/autoplay.rs` — 94 additions, 3 deletions (offscreen target setup + drain_commands screenshot branch)
- `reports/PROMPT-1790-autoplay-offscreen-bevy-capture-codepayload-refresh-after-1789-report-main.md` — this file

---

1790: AUTOPLAY-OFFSCREEN-BEVY-CAPTURE-CODEPAYLOAD-REFRESH-AFTER-1789-REPORT-MAIN: READY_FOR_MAINLAND_ENQUEUE
