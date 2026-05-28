# PROMPT 1867 — QA-SNAPSHOT-OBSERVABILITY-GAP-REFRESH-AFTER-1844

**Date:** 2026-05-28
**Author:** PROMPT-1867 worker
**Source tree:** `origin/main@bb90d7c2` (PROMPT 1844 — autoplay vs-bot viewport/click-target evidence audit)
**Scope:** Observability gap audit refresh — report only, no source edits.

> **IMPORTANT — Scope of this document:**
> This report is an **observability gap audit** of the QA snapshot system, refreshed
> to reflect the state of `origin/main` as of PROMPT 1844. It documents what data
> fields the snapshot captures and what it does NOT capture. It is **not** proof of
> GUI correctness, autoplay PASS/FAIL, or correct bot behaviour. Live run verdicts
> must come from `analyze_evidence_run.py` (PROMPT 1833) and the viewport/click
> audit (PROMPT 1844).

---

## 0. Relationship to Prior Audit (PROMPT 1839)

PROMPT 1839 (`reports/PROMPT-1839-qa-snapshot-observability-gap-refresh.md`)
performed this same gap audit against `origin/main@71484998`. That branch
(`origin/wt-1839-qa-obs-gap`) is **not** fast-forward-mergeable over current main
without reverting PROMPT 1833 (evidence analyzer + tests) and PROMPT 1844 (viewport/
click audit report). This 1867 report therefore backfills 1839 onto the correct base
and adds findings from 1833 and 1844.

**What changed between PROMPT 1839 base and current main:**

| PROMPT | Commit | Impact on gap audit |
|---|---|---|
| 1833 | `b856eef4` | Added `tools/autoplay/analyze_evidence_run.py` (evidence distinctness analyzer) + 21 tests. Directly addresses GAP-11 in the tools layer (verdict logic for FROZEN, NEEDS_HUMAN_GUI). Does NOT close the snapshot struct gap. |
| 1844 | `bb90d7c2` | Identified mid-run window resize bug (090613 run), stale win32 PrintWindow frames, and baked-at-tick-1 coordinate issue. Introduced 8 acceptance criteria (AC-VPT-01 through AC-VPT-08). No source code changes — report only. |

---

## 1. Snapshot Field Inventory (as of current main)

The snapshot system inventory from PROMPT 1839 remains accurate; no source changes
landed in PROMPT 1833 or 1844 that touch `client/src/presentation/qa_snapshot.rs`
or `server/src/feature/bot/qa_snapshot.rs`.

For the full field-by-field inventory, see:
`reports/PROMPT-1839-qa-snapshot-observability-gap-refresh.md` §§ 1.1–1.5.

Key structures unchanged:
- `QASnapshotData` (client, qa_snapshot.rs:391)
- `ExtrasSnapshot` and all sub-fields (qa_snapshot.rs:1127)
- `BotQaSnapshot` (server, bot/qa_snapshot.rs:236)
- `DebugBotOverlayState` (debug_bot_overlay.rs:100) — still NOT read by snapshot system

---

## 2. Coverage Verdicts (refreshed)

All coverage verdicts from PROMPT 1839 hold unchanged because no snapshot struct
changes landed between 1839 and 1867:

| UI / Game Area | Verdict | Key gaps |
|---|---|---|
| Phase / timer | COVERED | — |
| Shop offers | PARTIAL | Card class/rarity per slot (GAP-4) |
| Auction leader/status | PARTIAL | Auctioned card class/rarity (GAP-5) |
| Placement drag target / accepted ACK / rejected recovery | PARTIAL | ACK is heuristic-only (GAP-7) |
| Board units | PARTIAL | Unit class_id missing (GAP-6) |
| Resolution / gameover | PARTIAL | Result screen outcome fields absent (GAP-1/2); round lost mid-playback (GAP-8); recovery signal private (GAP-9) |
| Bot / autoplay debug overlay | MISSING | Overlay state entirely absent (GAP-3); autoplay step not captured (GAP-11) |
| **Viewport / window size** | **MISSING (new)** | Snapshot does not record whether the window resized during an autoplay run (see §3 below) |

---

## 3. New Findings from PROMPT 1833 and PROMPT 1844

### 3.1 GAP-12 — Window Size Change Not Tracked in Snapshot (from PROMPT 1844)

PROMPT 1844 established that run `20260528-090613-Z` suffered a mid-run window resize
(720 → 505 → 1076 px) triggered by `win_foreground.py`'s `SW_RESTORE` call on a
DWM-snapped window. The QA snapshot (`snapshot.json`) records the window size at the
moment of capture (`window.width`, `window.height`) but does NOT record:

- Whether the window was resized since the previous snapshot.
- The initial window size at recipe build time.
- A `window_resize_events` counter for the run.

**Impact:** A post-resize snapshot will show the post-resize dimensions. Without an
initial-size field and a resize counter, automated analysis cannot distinguish a
clean run (stable 720) from a corrupted run (720 → 1076) from snapshot data alone.

**Field gap:**

| Missing field | Where it should live | Notes |
|---|---|---|
| `window.resize_count` | `WindowInfo` (qa_snapshot.rs:408) | Total WM_SIZE events since app launch |
| `window.initial_logical_width/height` | `WindowInfo` | Size at Bevy app startup / `AutoplayPlugin` init |
| `window.is_stable` | `WindowInfo` | `true` if no resize events since last snapshot |

**Current workaround:** The driver log (`driver-timeline.jsonl`) records
`window_size` per tick, enabling post-hoc detection of resize events. The
`analyze_evidence_run.py` tool (PROMPT 1833) does NOT yet parse the timeline for
resize events — it relies only on `launcher-status.json` and `driver.log`.

**Severity:** MAJOR for automated PASS determination (see AC-VPT-02 from PROMPT 1844).

---

### 3.2 GAP-13 — Frozen Renderer Not Detectable from Snapshot Alone (from PROMPT 1833 / 1844)

PROMPT 1833's `analyze_evidence_run.py` detects frozen win32 captures by comparing
`pixel_hash` values across consecutive captures in `driver.log`. PROMPT 1844 confirmed
that all 11 win32_printwindow captures in run `090613` shared the same hash (frozen).

The QA snapshot (`snapshot.json`) does NOT record:

- `screenshot.is_stale` — whether the screenshot file is a re-capture of a prior frame.
- `screenshot.pixel_hash` — the MD5 hash of the screenshot PNG.
- `screenshot.capture_method` — which capture backend produced the PNG (`win32_printwindow`, `desktop_bitblt`, `bevy_native`, etc.).

**Impact:** A snapshot JSON without these fields cannot be used to determine whether
the screenshot file it references is a live frame or a frozen/stale frame.

**Field gap:**

| Missing field | Where it should live | Notes |
|---|---|---|
| `screenshot.capture_method` | `ScreenshotInfo` (qa_snapshot.rs:403) | Enum token: `win32_printwindow`, `desktop_bitblt`, `bevy_native`, `rpc_screenshot` |
| `screenshot.pixel_hash` | `ScreenshotInfo` | MD5/SHA256 of the PNG at snapshot time |
| `screenshot.is_frozen` | `ScreenshotInfo` | True if hash matches previous snapshot's hash |

**Current workaround:** `analyze_evidence_run.py` parses `driver.log` for
`pixel_hash=` lines and `FROZEN` labels, providing a run-level verdict. Per-snapshot
frozen detection is not available from JSON alone.

**Severity:** MAJOR. The screenshot field in `snapshot.json` currently records only
path, status, and timestamps — insufficient to validate visual evidence quality.

---

### 3.3 GAP-14 — Click-Target Accuracy Not Captured in Snapshot (from PROMPT 1844)

PROMPT 1844 identified that the autoplay driver bakes click coordinates at recipe
build time (tick 1) and does not update them on window resize. The snapshot does not
record:

- The fractional click target intended for the current tick (`intended_fx`, `intended_fy`).
- Whether the last click was inside the window bounds at the time of firing.
- The autoplay recipe's build-time window size.

**Impact:** Even if a snapshot captures `extras.input.pointer_screen` (current cursor
position) and `window.width/height`, there is no field indicating what the autoplay
driver *intended* to click vs what it *actually* sent. Post-hoc correlation requires
reading `driver-timeline.jsonl`.

**Field gap:**

| Missing field | Where it should live | Notes |
|---|---|---|
| `autoplay.recipe_build_window` | `AutoplayStepSnapshot` (GAP-11, not yet added) | `[w, h]` at build time |
| `autoplay.last_click_target` | `AutoplayStepSnapshot` | `{fx, fy, logical_x, logical_y, in_bounds}` |
| `autoplay.window_resize_detected` | `AutoplayStepSnapshot` | `true` if current window differs from build-time window |

**Current workaround:** Manual correlation between `driver-timeline.jsonl` tick
entries and screenshot timestamps. AC-VPT-02 (PROMPT 1844) mandates driver-side
logging of resize events.

**Severity:** ADVISORY for pure observability; MAJOR if snapshot is used for automated
click accuracy verification.

---

### 3.4 PROMPT 1833 / 1844 Status vs. Gap Map

| Gap | Status after PROMPT 1833 | Status after PROMPT 1844 |
|---|---|---|
| GAP-11 (autoplay step not in snapshot) | Partially addressed in the tools layer: `analyze_evidence_run.py` gives run-level verdict, but the snapshot struct itself is unchanged. | No change. |
| GAP-3 (bot debug overlay absent) | No change. | No change. |
| GAP-1/2 (result screen outcome) | No change. | No change. |
| GAP-12 (window resize tracking) | Not previously identified; now documented. | Concrete evidence from 090613 run. |
| GAP-13 (frozen renderer in snapshot) | `analyze_evidence_run.py` addresses at run level. Per-snapshot detection still absent. | Confirmed: all 11 win32 captures frozen in 090613. |
| GAP-14 (click-target accuracy) | Not previously identified. | Concrete evidence: post-resize clicks at wrong fractions in 090613. |

---

## 4. Updated Gap List (all 14 gaps)

> Gaps 1–11 are carried forward from PROMPT 1839 without changes to their descriptions.
> Gaps 12–14 are new as of this refresh.

| GAP | Area | Severity | Status |
|---|---|---|---|
| GAP-1 | Result screen — `S2CGameOver` payload missing | HIGH | Open |
| GAP-2 | Result screen — local win/loss/draw not projected | HIGH | Open |
| GAP-3 | Bot debug overlay state entirely absent | HIGH | Open |
| GAP-4 | Card class/rarity not in shop slot snapshot | MEDIUM | Open |
| GAP-5 | Auctioned card class/rarity not in auction state | MEDIUM | Open |
| GAP-6 | Board unit class_id absent | MEDIUM | Open |
| GAP-7 | Placement ACK is heuristic-only | HIGH | Open (S2CPlacementAck proposed PROMPT 1533, not shipped) |
| GAP-8 | AnimQueue round number lost mid-playback | LOW | Open (out-of-scope PROMPT 1586) |
| GAP-9 | Resolution recovery signal private | LOW | Open (out-of-scope PROMPT 1586) |
| GAP-10 | Card art aspect-ratio diagnostics absent | LOW | Open (proposed PROMPT 1533, not shipped) |
| GAP-11 | Autoplay recipe step state not in snapshot | HIGH | Partially addressed at tools layer (PROMPT 1833) — struct gap open |
| GAP-12 | Window resize events not tracked in snapshot | MAJOR (new) | Open — workaround via driver-timeline.jsonl |
| GAP-13 | Frozen renderer not detectable from snapshot JSON alone | MAJOR (new) | Partially addressed at tools layer (PROMPT 1833) — per-snapshot detection absent |
| GAP-14 | Click-target accuracy not captured in snapshot | ADVISORY (new) | Open — workaround via driver-timeline.jsonl |

---

## 5. Evidence Qualification Note (from PROMPT 1844 and 1846)

**PROMPT 1844** established that run `20260528-090613-Z` is NOT a clean PASS:
- Mid-run window resize triggered by win32 foreground operations.
- Post-resize click targets at wrong fractions (placement, submit buttons).
- All win32_printwindow captures frozen (fallback to desktop_bitblt working).

**PROMPT 1846** applied `analyze_evidence_run.py` to existing runs and reported
`PARTIAL` (frozen win32 renderer) for at least one run. The tool's verdict of `PARTIAL`
qualifies the run as **not a final automated PASS**; a human GUI review is required
per AC-VPT-05 and AC-VPT-06 (PROMPT 1844).

**Current evidence qualification for the vs-bot recipe:**

| Run | `analyze_evidence_run.py` verdict | PROMPT 1844 assessment | Qualified as |
|---|---|---|---|
| `20260528-051148-Z` | PASS (frozen hash on old win32 backend) | Clean baseline (no resize) | Conditional PASS — human review advised |
| `20260528-063609-Z` | PARTIAL (frozen win32 renderer) | Clean baseline (no resize) | PARTIAL — win32 capture stale; desktop_bitblt evidence usable |
| `20260528-090613-Z` | PARTIAL (frozen win32 renderer) | Resize bug run — not clean | NOT a PASS — resize corrupted post-resize clicks |

**None of the three runs constitutes a final automated PASS** until:
1. AC-VPT-02 (mid-run resize detection + abort) is implemented.
2. AC-VPT-06 (minimum screenshot requirements per checkpoint) is verified.

---

## 6. Proposed Next Lanes

The Lane A–F proposals from PROMPT 1839 remain valid. The following additions apply
to the three new gaps:

### Lane G — Window Resize Tracking (GAP-12)

**Owner**: `client/src/presentation/qa_snapshot.rs` + `client/src/autoplay.rs` or a
new `WindowResizeTracker` resource.

**What to add**:
- A `WindowResizeTracker` resource (incremented on `WindowResized` events).
- Fields in `WindowInfo`: `resize_count: u32`, `initial_logical_width: u32`,
  `initial_logical_height: u32`, `is_stable: bool`.
- Autoplay side: driver (`tools/autoplay/driver.py`) AC-VPT-02 abort logic owns the
  driver-level guard; the snapshot field is the client-side corroboration.

---

### Lane H — Screenshot Capture Quality Fields (GAP-13)

**Owner**: `client/src/presentation/qa_snapshot.rs` only.

**What to add**:
- Extend `ScreenshotInfo` (qa_snapshot.rs:403) with:
  `capture_method: Option<String>`, `pixel_hash: Option<String>`, `is_frozen: Option<bool>`.
- The capture method is already known at screenshot write time (RPC path vs. F9 path).
- The pixel hash requires reading the PNG file at snapshot time — acceptable for QA
  mode, costly if done on every frame. Should be computed only when `CCGS_QA_SNAPSHOT=1`.

---

### Lane I — Autoplay Click-Target Fields (GAP-14)

**Owner**: `client/src/presentation/qa_snapshot.rs` + `client/src/autoplay.rs`.

**What to add** (contingent on GAP-11 autoplay struct being created first):
- Extend `AutoplayStepSnapshot` with `recipe_build_window: [u32; 2]`,
  `last_click_target: Option<ClickTargetSnapshot>`, `window_resize_detected: bool`.
- The autoplay module must expose these via a public resource that qa_snapshot reads.

---

## 7. Summary

This refresh confirms that the PROMPT 1839 gap map is accurate against current main
(`origin/main@bb90d7c2`). No snapshot struct changes landed in PROMPT 1833 or 1844.

Three new gaps (GAP-12, GAP-13, GAP-14) are added based on concrete evidence from
the 090613 run analysis. The tools-layer additions from PROMPT 1833 partially address
GAP-11 and GAP-13 at the run-level verdict layer, but the corresponding `snapshot.json`
struct fields are still absent.

**All 14 gaps remain open in the snapshot struct.** Closing GAP-12 and GAP-13 is
recommended before declaring any automated vs-bot run as a mechanical PASS.

---

1867: QA-SNAPSHOT-OBSERVABILITY-GAP-REFRESH-AFTER-1844: SHIPPED
