# PROMPT 1788 — AUTOPLAY-VSBOT-SCREENSHOT-DISTINCTNESS-LIVE-VERIFY-AFTER-1787

**Date:** 2026-05-28
**Worktree:** `D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1788-screenshot-distinctness` (branch `prompt-1788-screenshot-distinctness`)
**Tested commit:** `8dadb857` — origin/main after PROMPT 1787 (foreground window title mainland refresh)
**Build note:** Target directory junctioned from `tmpwt-1782/target`; incremental build took 1m 56s (no Rust source changed between 49c7805b → 8dadb857)

---

## Command Executed

```powershell
powershell.exe -ExecutionPolicy Bypass `
  -File "D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1788-screenshot-distinctness/tools/dev-launcher/Start-AutoplayVsBot.ps1" `
  -Recipe vs-bot `
  -SoakReadySecs 60 `
  -ClientStartupSecs 90 `
  -DriverTimeoutSecs 360 `
  -Python "D:/_APPS/Python312/python.exe" `
  -PlayRepoRoot "D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1788-screenshot-distinctness"
```

**Exit code:** `0`
**Build:** Incremental (target junction to tmpwt-1782) — 1m 56s
**Soak server:** Bound on port 5000 within 60s
**Driver:** recipe=vs-bot, 260 ticks, exit 0

---

## Evidence Paths

| Path | Contents |
|------|----------|
| `tmpwt-1788-screenshot-distinctness/production/qa/evidence/autoplay-runs/20260528-040849-Z/` | checkpoints.jsonl, driver.log, launcher-status.json, driver-timeline.jsonl, screenshots/ (15 PNGs + 15 JSON sidecars) |
| `tmpwt-1788-screenshot-distinctness/production/qa/evidence/composite-runs/2026-05-28-040848-autoplay-vs-bot/` | composite-summary.json (outcome: ok), autoplay-run-path.txt |

---

## Claim 1: Foreground helper logs `matched window title='Lanes and Lies'`

**Result: ✅ PASS — COMPLETE FIX from PROMPT 1786/1787**

All 15 checkpoint screenshots are preceded by the foreground match log:

```
foreground: matched window title='Lanes and Lies' hwnd=0x00600fd0
foreground: SetForegroundWindow OK hwnd=0x00600fd0
```

This appears identically for every one of the 15 screenshot invocations across all game phases. **Zero** instances of the previous `no CCGS/Bevy window found` error.

**Comparison to PROMPT 1782 (pre-fix):**
| Run | Foreground outcome |
|-----|--------------------|
| PROMPT 1782 (49c7805b) | `no CCGS/Bevy window found among 21 visible windows` — 0/15 matched |
| PROMPT 1788 (8dadb857) | `matched window title='Lanes and Lies'` + `SetForegroundWindow OK` — 15/15 matched |

PROMPT 1786's `_WINDOW_TITLE_HINTS` expansion to include `"lanes and lies"` resolves the foreground discovery failure completely.

---

## Claim 2: `status.frame` advances across checkpoints

**Result: ✅ PASS**

| Tick | Checkpoint | Frame | Elapsed |
|------|-----------|-------|---------|
| 1 | lobby-loaded | 2 | 0.016s |
| 26 | bot-added | 537 | 2.594s |
| 38 | lobby-confirmed | 819 | 3.766s |
| 47 | class-select-loaded | 1038 | 4.688s |
| 68 | class-confirmed | 1548 | 6.813s |
| 77 | shop-loaded | 1771 | 7.750s |
| 89 | shop-slot-clicked | 2057 | 8.938s |
| 109 | auction-loaded | 2528 | 10.906s |
| 134 | auction-ready | 3128 | 13.406s |
| 143 | placement-loaded | 3338 | 14.297s |
| 160 | placement-dragged | 3747 | 16.016s |
| 172 | placement-submitted | 4058 | 17.328s |
| 181 | resolution-started | 4280 | 18.250s |
| 246 | resolution-complete | 5823 | 24.672s |
| 255 | vs-bot-post-resolution | 6016 | 25.516s |

Frame delta: 6016 − 2 = **6014 frames** over 25.5s ≈ **236 fps** — WinitSettings::game() working correctly.

---

## Claim 3: Screenshot PNG hashes not all identical within the run

**Result: ❌ FAIL — byte-identical despite foreground fix**

### Hash table (all 15 screenshots)

| File | Size (bytes) | SHA-256 (first 32 hex) | Written at (UTC) |
|------|-------------|------------------------|------------------|
| 000000.png (lobby-loaded) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:08:55 |
| 000007.png (bot-added) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:08:58 |
| 000011.png (lobby-confirmed) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:08:59 |
| 000013.png (class-select-loaded) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:00 |
| 000020.png (class-confirmed) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:02 |
| 000022.png (shop-loaded) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:03 |
| 000026.png (shop-slot-clicked) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:04 |
| 000030.png (auction-loaded) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:06 |
| 000037.png (auction-ready) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:09 |
| 000039.png (placement-loaded) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:10 |
| 000048.png (placement-dragged) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:11 |
| 000052.png (placement-submitted) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:13 |
| 000054.png (resolution-started) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:14 |
| 000055.png (resolution-complete) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:20 |
| 000057.png (vs-bot-post-resolution) | 85955 | 1D033581E166E4900DC223544FE86432 | 04:09:21 |

**Unique hashes: 1 / 15 — all byte-identical.**

**Note:** Files have different write timestamps (3–6 second gaps), confirming each was independently written to disk. The problem is not a copy — the Bevy screenshot RPC writes a distinct file each time, but with identical content.

---

## Claim 4: Screenshots nonblank and visually distinct

**Result: ⚠️ PARTIAL — non-black, but NOT distinct**

### Pixel analysis (sampled every 4px)

| File | Dimensions | Non-dark pixels (>20 threshold) | Max channel |
|------|-----------|----------------------------------|-------------|
| 000000.png | 1280×720 | 33,615 / 57,600 (58.4%) | R=250, G=245, B=255 |

All 15 share the same hash, so all share the same pixel data.

**Critical improvement from PROMPT 1782:**

| Run | Non-dark % | Content |
|-----|-----------|---------|
| PROMPT 1782 (pre-fix) | 1.6% | Desktop/terminal (wrong window captured) |
| PROMPT 1788 (post-fix) | 58.4% | Real game UI content ✅ |

The foreground fix correctly redirected screenshot capture from the desktop/terminal to the Bevy game window. However, all 15 captures return the same game frame — the Bevy `autoplay/screenshot` RPC appears to return the same GPU backbuffer on every call regardless of which render frame is active.

---

## Claim 5: Composite validator passes

**Result: ✅ PASS**

```
[validate_composite_run] PASS: ...composite-runs/2026-05-28-040848-autoplay-vs-bot
```

The PROMPT 1785 BOM fix (already on main at 8dadb857) resolved the path mismatch that failed in PROMPT 1782.

---

## Root Cause Analysis

### What PROMPT 1786/1787 fixed (confirmed working)

The `_WINDOW_TITLE_HINTS` expansion to include `"lanes and lies"` (the actual Bevy `WindowPlugin` title set in `client/src/main.rs`) completely resolves the foreground discovery failure:
- 15/15 foreground matches in this run (vs. 0/15 in PROMPT 1782)
- `SetForegroundWindow OK` on every screenshot
- Screenshot content: real game UI (58% non-dark) vs. desktop capture (1.6% non-dark)

### New root cause: Bevy screenshot RPC returns same frame each time

Despite the game advancing 6014 frames and the foreground being actively set before each capture, all 15 screenshots share identical bytes. The files are written at different times (distinct timestamps) confirming the RPC is serviced independently, but the Bevy screenshot system returns the same GPU buffer content each time.

**Hypotheses (in priority order):**
1. **Bevy screenshot asynchrony**: `Screenshot::primary_window()` captures asynchronously; the RPC may return the result of the *previous* screenshot before the new one is ready, causing all calls to return the initial captured frame.
2. **Stale render target**: After `SetForegroundWindow`, the GPU compositor has not yet produced a new composited frame when the Bevy screenshot fires. The screenshot captures a cached pre-composited surface.
3. **RPC handler caches result**: The `autoplay/screenshot` RPC handler in Rust may be returning a cached path + data that only updates under specific conditions.

**Next fix needed:** Investigate `client/src/autoplay/screenshot.rs` (or equivalent) to determine if the screenshot queue drains correctly and if the result returned is always the current frame rather than a cached one.

---

## Progressive Fix Audit

| PROMPT | Fix | Outcome |
|--------|-----|---------|
| 1774 | WinitSettings::game() | Frame counter advances ✅ |
| 1766 | Frame-advance barrier in driver | Barrier works ✅ |
| 1776/1781/1786/1787 | win_foreground.py + "lanes and lies" hint | Window found, SetForegroundWindow OK ✅ |
| 1785 | BOM fix in validator | Composite validator passes ✅ |
| **Remaining** | Bevy screenshot RPC returns stale frame | Screenshots byte-identical ❌ |

---

## Overall Disposition

| Check | Result |
|-------|--------|
| Foreground matched `Lanes and Lies` | ✅ PASS (15/15 — complete fix) |
| `status.frame` advances across checkpoints | ✅ PASS |
| Screenshot PNG hashes not all identical | ❌ FAIL (1/15 unique) |
| Screenshots nonblank | ✅ PASS (58% non-dark — real game UI) |
| Screenshots visually distinct | ❌ FAIL (byte-identical) |
| Composite validator | ✅ PASS |

**Overall verdict: FAIL**

PROMPT 1786/1787 successfully fixed the foreground window discovery problem. Screenshots now capture real game UI content. The remaining failure is a new root cause: the Bevy `autoplay/screenshot` RPC returns the same frame on every invocation despite the game rendering new frames. This is a separate Rust-side bug that requires investigation of the screenshot queue/drain logic.

---

1788: AUTOPLAY-VSBOT-SCREENSHOT-DISTINCTNESS-LIVE-VERIFY-AFTER-1787: FAIL
