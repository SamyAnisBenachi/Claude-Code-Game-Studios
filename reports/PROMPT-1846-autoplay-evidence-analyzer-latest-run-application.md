# PROMPT 1846 — Autoplay Evidence Analyzer: Latest Run Application

**Date:** 2026-05-28  
**Analyzer:** `tools/autoplay/analyze_evidence_run.py` (introduced PROMPT 1833)  
**Source branch:** origin/main@b856eef4  

---

## Evidence Directories Analyzed

| Run | Directory | Timestamp | Duration |
|-----|-----------|-----------|----------|
| Run 1 | `production/qa/evidence/autoplay-runs/20260528-051148-Z` | 05:28:42–05:29:14 Z | ~32 s |
| Run 2 | `production/qa/evidence/autoplay-runs/20260528-063609-Z` | 06:52:32–06:53:08 Z | ~36 s |
| Run 3 | `production/qa/evidence/autoplay-runs/20260528-090613-Z` | 09:06:15–09:06:55 Z | ~40 s |

All three runs also have a composite-run entry under `production/qa/evidence/composite-runs/`.

---

## Analyzer Output — Run 1: 20260528-051148-Z

```
=== analyze_evidence_run: 20260528-051148-Z ===

Launcher / Driver:
  launcher_outcome : 'ok'
  driver_exit_code : 0
  client_exit_code : None

Capture Labels:
  families seen    : (none)
  FROZEN lines     : 0

Screenshots:
  root win32_tick  : 0 PNG(s)
  bevy screenshots : 15 PNG(s)
  total            : 15

pixel_hash:
  total captures   : 0
  distinct count   : 0
  distinct values  : (none)
  frozen pattern   : False

VERDICT: PARTIAL
REASON : screenshots present but no recognised capture label in driver.log
```

**Root cause:** The driver in this run did not activate `win32_capture`, `win32_printwindow`, or `desktop_bitblt`. It used only the legacy `foreground:` strategy (`SetForegroundWindow returned 0 — trying BringWindowToTop`). Screenshots were produced solely via the Bevy RPC `autoplay/screenshot` path (writing to `screenshots/`). No `pixel_hash` values were logged, so distinctness cannot be verified programmatically.

**Checkpoints reached:** All 15 — `lobby-loaded` → `bot-added` → `lobby-confirmed` → `class-select-loaded` → `class-confirmed` → `shop-loaded` → `shop-slot-clicked` → `auction-loaded` → `auction-ready` → `placement-loaded` → `placement-dragged` → `placement-submitted` → `resolution-started` → `resolution-complete` → `vs-bot-post-resolution`.

**Signoff verdict: INSUFFICIENT** — No win32 capture labels and no pixel_hash data. Cannot machine-verify screenshot distinctness. Bevy RPC screenshots exist but lack hashes. Requires human visual inspection of all 15 PNGs to confirm distinct game states.

---

## Analyzer Output — Run 2: 20260528-063609-Z

```
=== analyze_evidence_run: 20260528-063609-Z ===

Launcher / Driver:
  launcher_outcome : 'ok'
  driver_exit_code : 0
  client_exit_code : None

Capture Labels:
  families seen    : ['win32_capture']
  FROZEN lines     : 0

Screenshots:
  root win32_tick  : 15 PNG(s)
  bevy screenshots : 15 PNG(s)
  total            : 30

pixel_hash:
  total captures   : 15
  distinct count   : 1
  distinct values  : ['0x26207c4c']
  frozen pattern   : True

VERDICT: PARTIAL
REASON : all 15 pixel_hash captures share the same value (0x26207c4c) — renderer may be frozen
```

**Root cause:** `win32_capture` was active and produced 15 PNGs at `1296×759` resolution. However all 15 pixel_hash values are identical (`0x26207c4c`), meaning the captured frames never changed across the entire run — from lobby through post-resolution. This strongly corroborates the human observation that the GUI was opened too small / clipped: `win32_capture` was successfully reading the window buffer each time but the rendered content was never updating (frozen renderer or the visible game area was a static clipped region). The driver logged no `FROZEN` lines, meaning the frozen pattern was not detected at capture time — the fallback to `desktop_bitblt` was not triggered.

**Checkpoints reached:** All 15 (same recipe as Run 1 — recipe completed at tick 260).

**Window dimensions:** 1296×759 throughout (consistent with Run 1 — below expected full resolution).

**Fallback label trustworthiness:** `win32_capture` label is present and legitimate, but the frozen hash pattern shows the captures are not representative of live game state transitions. The label alone cannot be used to claim GUI visibility.

**Signoff verdict: INSUFFICIENT** — Frozen renderer (all hashes identical). win32_capture captured the same static frame 15 times. This evidence does not prove a real GUI-visible pass; it is indistinguishable from a scenario where the game window is showing only a static loading screen or clipped blank area.

---

## Analyzer Output — Run 3: 20260528-090613-Z (Best Run)

```
=== analyze_evidence_run: 20260528-090613-Z ===

Launcher / Driver:
  launcher_outcome : 'ok'
  driver_exit_code : 0
  client_exit_code : None

Capture Labels:
  families seen    : ['desktop_bitblt', 'win32_capture', 'win32_printwindow']
  FROZEN lines     : 11

Screenshots:
  root win32_tick  : 15 PNG(s)
  bevy screenshots : 15 PNG(s)
  total            : 30

pixel_hash:
  total captures   : 26
  distinct count   : 12
  distinct values  : ['0xb4db8636', '0xbb3c81cc', '0x6ba13736', '0x281d3775',
                      '0x5ef6083d', '0xd4e70842', '0xef7ef1dd', '0x34b8206f',
                      '0xf9e702e5', '0x8941261d', '0xaa0544a1', '0x9dcf44a0']
  frozen pattern   : False

VERDICT: PARTIAL
REASON : FROZEN label appeared 11 time(s) in driver.log
```

**Capture mechanism breakdown:**
- `win32_printwindow` attempted 15 times; **frozen 11 times** (`hash=0874d30f…` repeated)
- `desktop_bitblt` fallback triggered 11 times for frozen printwindow captures; all 11 produced **distinct hashes**
- `win32_capture` label also present (used for `ShowWindow`/`SetForegroundWindow` steps before printwindow attempt)
- Total unique pixel_hashes observed: **12 out of 26 captures** — real visual state change confirmed

**Window dimension change mid-run:**
- Ticks 5–113: 1296×759
- Ticks 113+: 1296×**1115** (window expanded, likely when placement/board phase rendered)

This dimension expansion is the direct trigger for the FROZEN pattern: `win32_printwindow` returns a stale hash (`0874d30f…` or `ca2ab3e8…`) whenever the window resizes, causing `desktop_bitblt` fallback to activate. The bitblt captures at the larger dimensions show distinct hashes — real content.

**All 15 checkpoints reached.** Composite summary confirms: `"live_pass_status": "NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator sign-off for live PASS evidence"`.

**Signoff verdict: INSUFFICIENT for automated PASS — BEST EVIDENCE for human review.**

The 12 distinct pixel_hashes (from 26 captures) prove real visual state changes occurred across the run. The desktop_bitblt captures (11 PNGs named `bitblt_tick_*.png`) are the most trustworthy screenshots; they were written when win32_printwindow produced a frozen result and show differing content. However the analyzer's PARTIAL verdict is correct because the dominant capture path (win32_printwindow) is repeatedly frozen, meaning 15 of 26 captures may not reflect live content at that tick.

---

## Cross-Run Comparison

| Metric | Run 1 (051148-Z) | Run 2 (063609-Z) | Run 3 (090613-Z) |
|--------|-----------------|-----------------|-----------------|
| Launcher outcome | ok | ok | ok |
| Driver exit code | 0 | 0 | 0 |
| Checkpoints (of 15) | 15/15 | 15/15 | 15/15 |
| Capture labels | none | win32_capture | win32_printwindow + win32_capture + desktop_bitblt |
| Total screenshots | 15 (Bevy only) | 30 | 30 + 11 bitblt = 41 files |
| pixel_hash total | 0 | 15 | 26 |
| Distinct hashes | 0 | **1 (frozen)** | **12** |
| FROZEN log lines | 0 | 0 | 11 |
| Window size | n/a (Bevy RPC) | 1296×759 (fixed) | 1296×759 → 1296×1115 |
| Analyzer verdict | PARTIAL | PARTIAL | PARTIAL |
| Evidence quality | Weakest | Weak | Strongest |

---

## Fallback Label Trustworthiness Assessment

| Label | Trustworthy? | Notes |
|-------|-------------|-------|
| `win32_capture` (Run 2) | Label only — content untrustworthy | All 15 hashes identical; window likely showing static/clipped frame |
| `win32_printwindow` (Run 3) | Partially — frozen 11/15 times | Frozen hash `0874d30f…` repeats on every tick after window resize; bitblt compensates |
| `desktop_bitblt` (Run 3) | **Most trustworthy** | All 11 fallback captures show distinct hashes; these represent real live desktop pixels |
| Bevy RPC screenshots | Present but unverifiable | No pixel_hash logged; require human visual comparison |

**Conclusion on fallback labels:** The `desktop_bitblt` label in Run 3 is the only capture mechanism producing reliably distinct hashes. The `win32_printwindow` frozen pattern is a known artifact of window resizing — the PrintWindow API returns a stale DIB when the window layout has changed but the compositor hasn't flushed. The `win32_capture` label in Run 2 is insufficient because the frozen-all-identical hash pattern means it captured the same frame regardless of game phase.

---

## Can the Evidence Prove a Real GUI-Visible Pass?

**No run achieves PASS verdict from the analyzer. Summary of why:**

- **Run 1:** No win32 capture path at all; Bevy RPC only; no pixel_hash verification possible.
- **Run 2:** win32_capture active but frozen renderer — all 15 captures identical; bot likely clicking on clipped/offscreen coordinates.
- **Run 3:** Strongest evidence with 12 distinct hashes and all checkpoints, but win32_printwindow is frozen 11/15 times. The desktop_bitblt fallback produces real evidence, but the analyzer correctly returns PARTIAL because the frozen-dominant path cannot be claimed as a clean pass.

**The human observation ("opened too small; full UI was clipped; bot clicked blank/offscreen areas") is consistent with Run 2 behavior** — where all 15 captures share the same hash regardless of game phase, and the window was 1296×759 throughout. Run 3 shows the window eventually expanding to 1296×1115, which suggests the game did render a larger view, but the frozen printwindow artifact prevented clean capture of those frames.

**For human signoff consideration:** The `bitblt_tick_*.png` files in Run 3 (11 files, at both 759 and 1115 heights) show 11 distinct pixel hashes and represent the closest thing to live GUI evidence in the dataset. A human reviewer inspecting those 11 bitblt files alongside the 15 Bevy RPC screenshots from the same run could potentially sign off on AUTOPLAY-VS-BOT-QA-001, subject to visual confirmation that the UI is not clipped and bot actions landed on visible elements.

---

## Recommendations

1. **Window sizing:** The 1296×759 starting size is too small — UI elements are likely clipped. A pre-run window maximize step should be added to the autoplay launcher before `capabilities OK` is reported.
2. **Frozen printwindow:** The freeze occurs reliably after window resize events. The `desktop_bitblt` fallback is working correctly but should not be the primary path. Consider making `desktop_bitblt` the primary capture method (not fallback) when window dimensions are unstable.
3. **pixel_hash gap in Run 1:** The driver in Run 1 did not invoke any win32 capture path. This suggests the driver version used pre-dates the capture integration; all future runs should verify `win32_capture` or `win32_printwindow` appears in the first tick.
4. **Automated PASS gate:** None of the three runs satisfies the PASS condition. The composite summary correctly states `NOT-CLAIMED`. A PASS would require: distinct pixel_hashes >= 3, zero FROZEN lines (or bitblt-primary instead), and window at full intended resolution from tick 1.

---

## Per-Run Signoff Table

| Run | Sufficient for automated PASS? | Sufficient for human-reviewed PASS? | Notes |
|-----|-------------------------------|-------------------------------------|-------|
| 20260528-051148-Z | NO | Possibly — needs human visual review of 15 Bevy PNGs | No pixel_hash data |
| 20260528-063609-Z | NO | NO | Frozen renderer; all hashes identical |
| 20260528-090613-Z | NO | **Conditionally** — bitblt PNGs show distinct content; human must verify UI not clipped | Best evidence; review `bitblt_tick_*.png` |

---

1846: AUTOPLAY-EVIDENCE-ANALYZER-LATEST-RUN-APPLICATION: PARTIAL
