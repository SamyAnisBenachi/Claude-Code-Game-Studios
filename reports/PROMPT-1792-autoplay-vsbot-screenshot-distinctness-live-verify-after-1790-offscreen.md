# PROMPT 1792 — AUTOPLAY-VSBOT-SCREENSHOT-DISTINCTNESS-LIVE-VERIFY-AFTER-1790-OFFSCREEN

**Date:** 2026-05-28
**Worktree:** `D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1792-screenshot-distinctness` (detached HEAD `bf9f2bf1`)
**Target commit:** `bf9f2bf1` — origin/main after PROMPT 1790 (offscreen Bevy capture code-payload refresh)
**Binary source:** `autoplay.rs` identical to `tmpwt-1789` (diff: no output); binary at `tmpwt-1789/target/debug/client.exe` (built 2026-05-28T05:10:43+01:00), junctioned to `tmpwt-1792/target`
**Binary features confirmed:** `["autoplay-remote", "default", "ui_picking"]` (from `client-521bf1599e7fad1d/bin-client.json`)
**Build time:** 2m 04s (recompile due to shared crate touching; not a clean cache hit)

---

## Command Executed

```powershell
powershell.exe -ExecutionPolicy Bypass `
  -File "D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1792-screenshot-distinctness/tools/dev-launcher/Start-AutoplayVsBot.ps1" `
  -Recipe vs-bot `
  -SoakReadySecs 60 `
  -ClientStartupSecs 90 `
  -DriverTimeoutSecs 360 `
  -Python "D:/_APPS/Python312/python.exe" `
  -PlayRepoRoot "D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1792-screenshot-distinctness"
```

**Exit code:** `0`
**Soak server:** Bound on port 5000 within 60s
**Driver:** recipe=vs-bot, 260 ticks, exit 0

---

## Evidence Paths

| Path | Contents |
|------|----------|
| `tmpwt-1792/production/qa/evidence/autoplay-runs/20260528-043102-Z/` | checkpoints.jsonl, driver.log, launcher-status.json, process.log, screenshots/ (15 PNGs + 15 JSON sidecars) |
| `tmpwt-1792/production/qa/evidence/composite-runs/2026-05-28-043102-autoplay-vs-bot/` | composite-summary.json (outcome: ok), autoplay-run-path.txt |

---

## Claim 1: Foreground helper logs `matched window title='Lanes and Lies'`

**Result: ✅ PASS — unchanged from PROMPT 1788**

All 15 checkpoint screenshots are preceded by the foreground match log:

```
foreground: matched window title='Lanes and Lies' hwnd=0x005e0250
foreground: SetForegroundWindow returned 0 — trying BringWindowToTop hwnd=0x005e0250
```

Appears identically for every one of the 15 screenshot invocations across all game phases. The PROMPT 1786 `_WINDOW_TITLE_HINTS` fix remains intact.

---

## Claim 2: `status.frame` advances across checkpoints

**Result: ✅ PASS**

| Tick | Checkpoint | Frame | Elapsed |
|------|-----------|-------|---------|
| 1 | lobby-loaded | 2 | 0.016s |
| 26 | bot-added | 552 | 2.516s |
| 38 | lobby-confirmed | 828 | 3.688s |
| 47 | class-select-loaded | 1027 | 4.563s |
| 68 | class-confirmed | 1531 | 6.672s |
| 77 | shop-loaded | 1752 | 7.579s |
| 89 | shop-slot-clicked | 2047 | 8.813s |
| 109 | auction-loaded | 2530 | 10.844s |
| 134 | auction-ready | 3124 | 13.329s |
| 143 | placement-loaded | 3335 | 14.235s |
| 160 | placement-dragged | 3749 | 15.938s |
| 172 | placement-submitted | 4041 | 17.157s |
| 181 | resolution-started | 4263 | 18.079s |
| 246 | resolution-complete | 5854 | 24.735s |
| 255 | vs-bot-post-resolution | 6062 | 25.641s |

Frame delta: 6062 − 2 = **6060 frames** over 25.6s ≈ **237 fps** — `WinitSettings::game()` working correctly.

---

## Claim 3: Offscreen render target created and active

**Result: ✅ CONFIRMED ACTIVE — but captures black content**

Process log confirms:

```
2026-05-28T04:33:11.3437257Z INFO client::autoplay: AutoplayPlugin: offscreen render target created (1280x720); screenshots will use Image path
```

Bevy's `Screenshot::image(handle)` path IS being exercised instead of `Screenshot::primary_window()`. This is the PROMPT 1780 code running.

---

## Claim 4: Screenshot PNG hashes not all identical within the run

**Result: ❌ FAIL — byte-identical, near-black content**

### Hash table (all 15 screenshots)

| File | Size (bytes) | SHA-256 (first 32 hex) | Written at (UTC) |
|------|-------------|------------------------|------------------|
| 000000.png (lobby-loaded) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:12 |
| 000007.png (bot-added) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:14 |
| 000011.png (lobby-confirmed) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:15 |
| 000013.png (class-select-loaded) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:16 |
| 000020.png (class-confirmed) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:18 |
| 000022.png (shop-loaded) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:19 |
| 000026.png (shop-slot-clicked) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:20 |
| 000030.png (auction-loaded) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:23 |
| 000037.png (auction-ready) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:25 |
| 000039.png (placement-loaded) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:26 |
| 000048.png (placement-dragged) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:28 |
| 000052.png (placement-submitted) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:29 |
| 000054.png (resolution-started) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:30 |
| 000055.png (resolution-complete) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:36 |
| 000057.png (vs-bot-post-resolution) | 15061 | dc4684f0d18ca5220803004c3ae660e7 | 04:33:37 |

**Unique hashes: 1 / 15 — all byte-identical.**

---

## Claim 5: Screenshots nonblank and visually distinct

**Result: ❌ FAIL — near-black, no game content**

### Pixel analysis (via Python PIL-free parser)

| File | Dimensions | Non-dark pixels (>20 threshold) | Key pixels |
|------|-----------|----------------------------------|------------|
| 000000.png | 1280×720 | **1 / ~57600 sampled (0.002%)** | Row 0 pixel 0: RGB(43, 44, 47); all others: (0, 0, 0) |

All 15 share the same hash and pixel data. The offscreen camera captures a near-black image.

**Critical regression from PROMPT 1788:**

| Run | Approach | File size | Non-dark % | Content |
|-----|----------|-----------|------------|---------|
| PROMPT 1788 (8dadb857) | `Screenshot::primary_window()` | 85,955 bytes | 58.4% | Real game UI ✅ |
| PROMPT 1792 (bf9f2bf1) | `Screenshot::image(offscreen)` | 15,061 bytes | ~0% | Near-black ❌ |

The offscreen capture is a regression: it captures less content than the primary_window approach it replaced.

---

## Claim 6: Composite validator passes

**Result: ✅ PASS**

```
[validate_composite_run] PASS: ...composite-runs/2026-05-28-043102-autoplay-vs-bot
```

Structural/schema checks pass.

---

## Root Cause Analysis

### What PROMPT 1780/1790 implemented

`setup_offscreen_target_system` spawns a secondary `Camera2d` at render order 1 with `RenderTarget::Image(handle)`. `drain_commands_system` uses `Screenshot::image(handle)` instead of `Screenshot::primary_window()`.

### Why offscreen capture fails to capture game content

In Bevy 0.18, Bevy UI rendering is gated by `IsDefaultUiCamera`. The `IsDefaultUiCamera` marker component is only on the primary camera (order 0, window target). The secondary offscreen camera has no `IsDefaultUiCamera`, so **zero UI content is rendered into the offscreen image**.

This game is almost entirely Bevy UI — menus, HUD, shop/auction panels, lobby, placement board overlay. The 2D sprite entities (board tiles, unit sprites) are minimal and possibly also culled since the camera viewport may not be configured correctly.

Result: the offscreen camera renders only the clear color (RGB 43, 44, 47 background) into the image.

### Why the primary_window approach also fails (byte-identical issue)

The pre-offscreen `Screenshot::primary_window()` DID capture real game UI (58% non-dark pixels) but all 15 captures were identical. The root cause identified in PROMPT 1788:
- Bevy's `Screenshot::primary_window()` captures the GPU swapchain backbuffer asynchronously
- Despite the game rendering new frames (frame counter advances correctly), the screenshot captures the same backbuffer content each time
- Likely cause: the compositor presents the same frame to the swapchain multiple times, or the async screenshot resolves against a cached buffer

### Fix direction required

The offscreen approach is not viable for a UI-heavy game. The fix must either:
1. **Re-enable `Screenshot::primary_window()`** and add `IsDefaultUiCamera` to the offscreen camera (so it captures UI), then address the swapchain stale-frame issue through a different mechanism
2. **Use WinAPI capture** (BitBlt / PrintWindow) instead of Bevy's screenshot API to capture the composed window content
3. **Force a GPU flush/present** before screenshot to ensure a fresh backbuffer is read

---

## Progressive Fix Audit

| PROMPT | Fix | Outcome |
|--------|-----|---------|
| 1774 | WinitSettings::game() | Frame counter advances ✅ |
| 1766 | Frame-advance barrier in driver | Barrier works ✅ |
| 1776/1781/1786/1787 | win_foreground.py + "lanes and lies" hint | Window found, SetForegroundWindow OK ✅ |
| 1785 | BOM fix in validator | Composite validator passes ✅ |
| 1788 | Live verify: primary_window captures | 58% non-dark, but all identical ❌ |
| **1780/1790** | Offscreen Bevy camera — screenshots | Near-black (0% non-dark), all identical ❌❌ |

---

## Overall Disposition

| Check | Result |
|-------|--------|
| Foreground matched `Lanes and Lies` | ✅ PASS (15/15) |
| `status.frame` advances across checkpoints | ✅ PASS (6060 frame delta) |
| Offscreen render target created and active | ✅ CONFIRMED |
| Screenshot PNG hashes not all identical | ❌ FAIL (1 unique hash / 15) |
| Screenshots nonblank | ❌ FAIL (~0% non-dark — near-black) |
| Screenshots visually distinct | ❌ FAIL (byte-identical) |
| Composite validator | ✅ PASS |

**Overall verdict: FAIL**

The offscreen Bevy render target (PROMPT 1780/1790) does NOT fix the screenshot distinctness gate and introduces a content regression. Screenshots are now near-black (no UI content captured) rather than capturing real game UI as the primary_window approach did. The distinctness problem remains unsolved.

---

1792: AUTOPLAY-VSBOT-SCREENSHOT-DISTINCTNESS-LIVE-VERIFY-AFTER-1790-OFFSCREEN: FAIL
