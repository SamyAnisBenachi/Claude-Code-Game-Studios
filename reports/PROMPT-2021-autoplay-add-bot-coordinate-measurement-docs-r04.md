# PROMPT 2021 — Autoplay Add Bot Coordinate Measurement Docs R04

**Date:** 2026-05-28
**Status:** SHIPPED
**Branch:** work/PROMPT-2021

---

## Summary

Added a concrete Add Bot coordinate measurement protocol and viewport/window-size
preflight rules to the autoplay documentation, as recommended by PROMPT 2017 (R-04).

---

## Files Modified

| File | Change |
|---|---|
| `docs/autoplay.md` | New top-level section `## Add Bot Coordinate Measurement Protocol` + new `CCGS_AUTOPLAY_ADD_BOT_BTN` row in env vars table |
| `docs/autoplay/evidence-operator-guide.md` | `CCGS_AUTOPLAY_ADD_BOT_BTN` added to coordinate overrides block + window-size preflight callout + two new Add Bot rows in §8 Common Failures + updated last-updated datestamp |

---

## New Documentation Content

### `docs/autoplay.md` — §Add Bot Coordinate Measurement Protocol

The new section covers:

1. **Why this matters** — when the client window opens too small, the Add Bot
   button may be clipped below the visible viewport. The recipe does not emit
   `local.block` in this case (the button exists; it is just unreachable), so
   the `bot-added` checkpoint silently disappears.

2. **Minimum window size and preflight checks** — actionable table:
   - Client physical width ≥ 1280 px (read from `status.json` → `window_width`)
   - Client physical height ≥ 720 px (read from `status.json` → `window_height`)
   - `CCGS_DEBUG_UI=1` required before client launch
   - Add Bot button must be fully visible in the `lobby-loaded` checkpoint screenshot

3. **Coordinate capture procedure** — eight concrete steps:
   - Launch client with `CCGS_DEBUG_UI=1`, reach lobby
   - Request screenshot via `driver.py --one-shot screenshot`
   - Open PNG in any pixel-coordinate-aware viewer
   - Record Add Bot button centre `(px, py)`
   - Read `window_width` / `window_height` from `status.json` (not monitor resolution)
   - Compute `fx = px / window_width`, `fy = py / window_height`
   - Validate both values in `[0.0, 1.0]` (out-of-range → button is off-screen)
   - Set `CCGS_AUTOPLAY_ADD_BOT_BTN = "fx,fy"` and re-run

4. **Re-measure triggers** — OS DPI change, resolution change, WindowPlugin config
   change, lobby UI layout change, different machine.

5. **Evidence to collect when clicks miss** — five mandatory artefacts for bug
   reports: `lobby-loaded` screenshot, post-click screenshot, `status.json` window
   dimensions, `driver.log` cursor coords, `checkpoints.jsonl`.

6. **Add Bot-specific failures quick-reference** — five rows covering the most
   common failure modes with concrete fixes.

### `docs/autoplay.md` — env vars table

New row added:

| Var | Default | Purpose |
|---|---|---|
| `CCGS_AUTOPLAY_ADD_BOT_BTN` | per `_coords.DEFAULTS` | Fractional screen position for the Add Bot button in `add-bot-lobby`; re-measure when window size or DPI changes |

### `docs/autoplay/evidence-operator-guide.md` — §2 Coordinate overrides

- Added `CCGS_AUTOPLAY_ADD_BOT_BTN = "0.5,0.65"` line (with inline comment) to the
  PowerShell override block.
- Added a `> Add Bot window-size preflight` callout block immediately below the
  override block, stating the ≥ 1280 × 720 requirement and pointing to the full
  re-measure procedure in `docs/autoplay.md`.

### `docs/autoplay/evidence-operator-guide.md` — §8 Common Failures

Two new rows added:

| Symptom | Cause | Fix |
|---|---|---|
| Driver exits 4 with `add-bot-lobby` | `CCGS_DEBUG_UI=1` not set | Expected — set before launch |
| `bot-added` checkpoint absent; lobby unchanged | Window too small — button clipped | Confirm `window_height` ≥ 720 px from `status.json`; maximise and re-run |
| `bot-added` checkpoint absent; click on wrong element | Default coords misaligned for current window/DPI | Re-measure; see §Add Bot Coordinate Measurement Protocol |

---

## Validation

- **Path allowlist:** Only `docs/autoplay.md` and `docs/autoplay/evidence-operator-guide.md`
  modified. No source code, production status/story files, or unrelated docs touched.
- **`git diff --check`:** Clean — no trailing whitespace or mixed line-ending issues.
- **Static doc review:** Both docs now explicitly mention:
  - The too-small window / offscreen-click failure mode
  - The minimum visible-target requirement (≥ 1280 × 720 px physical)
  - The operator re-measure procedure
  - `CCGS_AUTOPLAY_ADD_BOT_BTN` as the override env var

---

2021: AUTOPLAY-ADD-BOT-COORDINATE-MEASUREMENT-DOCS-R04: SHIPPED
