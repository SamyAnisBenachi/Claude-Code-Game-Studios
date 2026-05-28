# PROMPT 1937 — QA-SNAPSHOT-OBSERVABILITY-GAP-REPORT-REFRESH-AFTER-1931

**Date:** 2026-05-28
**Author:** PROMPT-1937 worker
**Branch:** `report/qa-snapshot-observability-gap-refresh-1937`
**Source tree:** `origin/main@79031021681c3ca72a02564bd1482cab99771015` (PROMPT 1931)

---

## Purpose

PROMPT 1900 shipped branch `origin/report/qa-snapshot-observability-gap-refresh-1900`
containing the three observability gap report files from PROMPT 1839 and PROMPT 1867.
That branch was based on `origin/main@c35750d8` (PROMPT 1856) and is **NOT
fast-forward-mergeable** against current `origin/main@79031021` (PROMPT 1931).
Direct landing would delete reports and revert source changes from PROMPT 1880, 1894,
and 1912.

This worker creates a clean branch from current `origin/main` that backfills only the
four owned report files — touching nothing else.

---

## Source Branch Details

| Field | Value |
|---|---|
| Source branch | `origin/report/qa-snapshot-observability-gap-refresh-1900` |
| Source tip | `origin/report/qa-snapshot-observability-gap-refresh-1900` HEAD |
| Source base | `origin/main@c35750d8` (PROMPT 1856) — stale vs current main |
| Report files carried | `PROMPT-1839-...`, `PROMPT-1867-...`, `PROMPT-1900-...` |

---

## What Changed Between PROMPT 1900 Base and Current Main

The following commits landed between `c35750d8` (PROMPT 1856, 1900 base) and
`79031021` (PROMPT 1931, current main):

| PROMPT | Commit | Source files changed | Impact on gap audit |
|---|---|---|---|
| 1880/1894 | `e8a40f81` | `tools/autoplay/driver.py`, `tests/tools/autoplay/test_driver_click_viewport_guard.py` | Viewport guard (AC-VPT-02, AC-VPT-08) in driver; partially addresses GAP-14 and GAP-12 at the driver layer — snapshot struct unchanged |
| 1912 | `e02d132f` | `client/src/autoplay.rs`, `tools/autoplay/Run-AutoplaySmoke.ps1` | AC-VPT-01 `enforce_autoplay_window_size_system` startup system; default window 1280x720; partially addresses GAP-12 at client startup — snapshot struct unchanged |
| 1912 (reports) | `fe2a9e88`, `1c945fd2` | reports only | PROMPT-1893/1879/1912 report backfills |
| 1929 | `63f3b575` | reports only | Result screen chrome polish SLICE-E refresh |
| 1931 | `79031021` | reports only | Autoplay 1831/1840 truth refresh after 1912 |

**No changes to `client/src/presentation/qa_snapshot.rs` or
`server/src/feature/bot/qa_snapshot.rs` landed in any of these commits.**
The snapshot struct inventory from PROMPT 1867 remains accurate against current main.

---

## Snapshot Field Inventory (as of current main)

The snapshot system inventory from PROMPT 1839 and PROMPT 1867 remains fully accurate.
No snapshot struct modifications landed between `c35750d8` and `79031021`.

For the field-by-field inventory see:
- `reports/PROMPT-1839-qa-snapshot-observability-gap-refresh.md` §§ 1.1–1.5
- `reports/PROMPT-1867-qa-snapshot-observability-gap-refresh-after-1844.md` §§ 1–2

Key structures unchanged:
- `QASnapshotData` (client, qa_snapshot.rs:391)
- `ExtrasSnapshot` and all sub-fields (qa_snapshot.rs:1127)
- `BotQaSnapshot` (server, bot/qa_snapshot.rs:236)
- `DebugBotOverlayState` (debug_bot_overlay.rs:100) — still NOT read by snapshot system
- `ResultScreenViewState` (result_screen.rs:85) — still NOT read by snapshot system

---

## Coverage Verdicts (current main)

All coverage verdicts from PROMPT 1867 hold unchanged:

| UI / Game Area | Verdict | Key gaps |
|---|---|---|
| Phase / timer | COVERED | — |
| Shop offers | PARTIAL | Card class/rarity per slot (GAP-4) |
| Auction leader / status | PARTIAL | Auctioned card class/rarity (GAP-5) |
| Placement drag target / accepted ACK / rejected recovery | PARTIAL | ACK is heuristic-only (GAP-7) |
| Board units | PARTIAL | Unit class_id missing (GAP-6) |
| Resolution / gameover | PARTIAL | Result screen outcome fields absent (GAP-1/2); round lost mid-playback (GAP-8); recovery signal private (GAP-9) |
| Bot / autoplay debug overlay | MISSING | Overlay state entirely absent (GAP-3); autoplay step not captured (GAP-11) |
| Viewport / window size | MISSING (from 1867) | Snapshot does not record resize-event count or initial window size (GAP-12) |

---

## Gap Status Update (all 14 gaps)

| GAP | Area | Severity | Status as of PROMPT 1937 |
|---|---|---|---|
| GAP-1 | Result screen — `S2CGameOver` payload missing | HIGH | Open — `ResultScreenViewState` still not read by snapshot |
| GAP-2 | Result screen — local win/loss/draw not projected | HIGH | Open |
| GAP-3 | Bot debug overlay state entirely absent | HIGH | Open — `DebugBotOverlayState` still not read |
| GAP-4 | Card class/rarity not in shop slot snapshot | MEDIUM | Open |
| GAP-5 | Auctioned card class/rarity not in auction state | MEDIUM | Open |
| GAP-6 | Board unit class_id absent | MEDIUM | Open |
| GAP-7 | Placement ACK is heuristic-only | HIGH | Open — `S2CPlacementAck` still not shipped |
| GAP-8 | AnimQueue round number lost mid-playback | LOW | Open (out-of-scope PROMPT 1586) |
| GAP-9 | Resolution recovery signal private | LOW | Open (out-of-scope PROMPT 1586) |
| GAP-10 | Card art aspect-ratio diagnostics absent | LOW | Open (proposed PROMPT 1533, not shipped) |
| GAP-11 | Autoplay recipe step state not in snapshot | HIGH | Open — struct gap open; tools-layer only (PROMPT 1833) |
| GAP-12 | Window resize events not tracked in snapshot | MAJOR | **Partially addressed at client startup layer** — `enforce_autoplay_window_size_system` (PROMPT 1912, `client/src/autoplay.rs`) enforces 1280x720 on startup, reducing resize risk. `WindowInfo` struct still lacks `resize_count`, `initial_logical_width/height`, `is_stable` fields. |
| GAP-13 | Frozen renderer not detectable from snapshot JSON alone | MAJOR | Open — `analyze_evidence_run.py` (PROMPT 1833) covers run-level; per-snapshot `capture_method`, `pixel_hash`, `is_frozen` fields still absent from `ScreenshotInfo` |
| GAP-14 | Click-target accuracy not captured in snapshot | ADVISORY | **Partially addressed at driver layer** — `check_window_drift` + `validate_cursor_coords` + out-of-bounds abort (PROMPT 1880/1894, `tools/autoplay/driver.py`). Snapshot `AutoplayStepSnapshot` struct still absent (GAP-11 prerequisite). |

### Gap Progress Summary

- **Fully closed:** 0 of 14
- **Partially addressed (driver/tools/startup layer only; snapshot struct unchanged):** 3 (GAP-11, GAP-12, GAP-14)
- **Open / unchanged:** 11

---

## Files Changed

| File | Action |
|---|---|
| `reports/PROMPT-1839-qa-snapshot-observability-gap-refresh.md` | Added (backfilled from `origin/report/qa-snapshot-observability-gap-refresh-1900`) |
| `reports/PROMPT-1867-qa-snapshot-observability-gap-refresh-after-1844.md` | Added (backfilled from `origin/report/qa-snapshot-observability-gap-refresh-1900`) |
| `reports/PROMPT-1900-qa-snapshot-observability-gap-refresh-report-refresh.md` | Added (backfilled from `origin/report/qa-snapshot-observability-gap-refresh-1900`) |
| `reports/PROMPT-1937-qa-snapshot-observability-gap-report-refresh-after-1931.md` | Added (this report) |

No deletes. No modifications to existing files. No `tools/**`, `client/**`, `server/**`,
`production/**`, `tests/**`, or `Cargo` files touched.

---

## Validation

See post-commit validation section below.

---

1937: QA-SNAPSHOT-OBSERVABILITY-GAP-REPORT-REFRESH-AFTER-1931: SHIPPED
