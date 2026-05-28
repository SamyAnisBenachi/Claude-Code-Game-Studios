# PROMPT 1935 — Bot/Autoplay Story Readiness Report Refresh After PROMPT 1931

**Date:** 2026-05-28
**Branch:** `report/bot-autoplay-readiness-refresh-1935`
**Source tree:** `origin/main@79031021` (PROMPT 1931 — latest main at authoring time)
**Scope:** Report-only — no source edits, no sprint-state writes.
**Supersedes:** `reports/PROMPT-1907-bot-autoplay-story-readiness-report-refresh-after-1876.md`
**Prior base:** PROMPT 1907 was authored on `origin/main@c35750d8`
**Backfill note:** This file was authored on branch `report/bot-autoplay-readiness-refresh-1935`
which was NOT_FF against `origin/main` at the time of review. The file is being added
to `origin/main` via PROMPT 1970 (backfill at base `7fc1706e`).

---

## 1. Why This Report Exists

PROMPT 1907 shipped a readiness refresh on
`origin/report/bot-autoplay-readiness-refresh-1907` but that branch was
**not FF-ready** over current main (`79031021`) — merging it directly would
delete the PROMPT 1894/1912/1929/1931 artifacts that landed between `c35750d8`
and `79031021`.

Since PROMPT 1907 (base `c35750d8`), the following landed on `origin/main`:

| PROMPT | Commit | What |
|--------|--------|------|
| **1880/1894** | `e8a40f81` / `71484fc4` | AC-VPT-02/08 click-target viewport guard — `driver.py` + `test_driver_click_viewport_guard.py` |
| **1912** | `e02d132f` + `fe2a9e88` + `1c945fd2` | AC-VPT-01 window-size default repair — `client/src/autoplay.rs`, `Run-AutoplaySmoke.ps1`, reports |
| **1929** | `63f3b575` | Result screen chrome polish SLICE-E report (unrelated to bot autoplay) |
| **1931** | `79031021` | PROMPT 1831/1840 truth correction re-applied on top of `1c945fd2` (6 report files) |

---

## 2. Evidence Truth (Unchanged from PROMPT 1844 + 1846 + 1931)

### 2.1 Available Runs

No new autoplay runs have been executed since the three runs captured on
2026-05-28. This section is identical to PROMPT 1907 §2.1.

| Run | Window size | Checkpoints | Analyzer verdict | Automated PASS? |
|-----|-------------|-------------|------------------|-----------------|
| `20260528-051148-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — no capture labels, no pixel_hash | NO |
| `20260528-063609-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — all 15 hashes identical (frozen renderer) | NO |
| `20260528-090613-Z` | `[1280,720]` → `[1280,1076]` mid-run | 15/15 | **PARTIAL** — 11/15 PrintWindow captures frozen; 11 distinct bitblt hashes | NO (conditional human-review only) |

### 2.2 Run `090613` Classification (UNCHANGED — CRITICAL)

Run `090613` is the **best available human-review evidence** but is **not a
clean automated PASS**. The PROMPT 1931 §4.2 classification stands verbatim:

- **Mid-run DWM window resize** (ticks 115–127): window snapped from `[1280,720]`
  to `[1280,1076]`.
- **Click coordinates baked at 720 height**: post-resize placement and submit
  clicks landed at 61.5% of 1076-height window (target: 92%).
- **PrintWindow all-frozen**: 11 frozen lines; `desktop_bitblt` fallback produced
  11 distinct hashes but the primary path was non-operational throughout.
- **Time-based checkpoints only**: passage does not confirm clicks landed on
  correct UI elements.

**Correct citation:** "Conditional human-review evidence — bitblt PNGs show
distinct visual state changes; requires human inspection to confirm UI was not
clipped and bot actions landed on visible elements."

**Prohibited citation:** Any sentence claiming `090613` or PROMPT 1831 as a
clean automated PASS, a clean smoke PASS, or as proof of correct bot UI
interaction.

### 2.3 Analyzer Reports on Main

The following analyzer report files are confirmed on `origin/main@79031021`:

| File | Status on main |
|------|----------------|
| `reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md` | **ON MAIN** |
| `reports/PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md` | **ON MAIN** |
| `reports/PROMPT-1872-autoplay-evidence-analyzer-latest-run-refresh-after-1858.md` | **ON MAIN** |
| `reports/PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md` | **ON MAIN** |
| `reports/PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md` | **ON MAIN** |
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | **ON MAIN** (truth-corrected, PROMPT 1931) |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | **ON MAIN** (PROMPT 1931) |
| `reports/PROMPT-1931-autoplay-1831-1840-truth-refresh-after-1912.md` | **ON MAIN** (PROMPT 1931) |

---

## 3. AC Item Status Against Current Main (`origin/main@79031021`)

### 3.1 AC-VPT-01 — Minimum Window Size Gate

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-window-size-default-1893` (PROMPT 1893) |
| **Landed via** | PROMPT 1912 (`e02d132f`) |
| **Status** | **MERGED TO MAIN** |
| **Verified on main** | `client/src/autoplay.rs`: `enforce_autoplay_window_size_system` at startup; `CCGS_WINDOW_WIDTH`/`CCGS_WINDOW_HEIGHT` env-var constants confirmed |
| **Verified on main** | `tools/autoplay/Run-AutoplaySmoke.ps1`: env-var guards present |
| **Gap** | Startup-size floor enforced. Mid-run DWM resize still not prevented by this AC item — that is AC-VPT-02/08 scope (also now on main). |

### 3.2 AC-VPT-02 + AC-VPT-08 — Click-Target Viewport Guard

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-click-viewport-guard-1880` (PROMPT 1880) |
| **Landed via** | PROMPT 1880 source commit (`e8a40f81`) + PROMPT 1894 report (`71484fc4`) |
| **Status** | **MERGED TO MAIN** |
| **Verified on main** | `tools/autoplay/driver.py`: `EXIT_VIEWPORT_GUARD = 5`, `_check_window_minimum`, `_check_window_drift`, `_validate_cursor_coords`, `_check_post_foreground_window` — all present |
| **Verified on main** | `tests/tools/autoplay/test_driver_click_viewport_guard.py` — present (66 tests) |
| **Checkpoint types on main** | `viewport_drift`, `viewport_shrink_abort`, `viewport_guard_cursor_none`, `viewport_guard_oob` |

### 3.3 Composite Window-Resize Verdict

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-composite-window-resize-verdict-1875` (PROMPT 1875) |
| **Base** | `origin/main@2ce3dc6b` |
| **Status** | **PENDING — not merged to main** |
| **Verified absent** | `tools/autoplay/analyze_evidence_run.py`: no `win32_quality` / `window_resize_verdict` field found on `origin/main@79031021` |
| **Verified absent** | `tests/tools/autoplay/test_window_resize_verdict.py`: not present in `tests/tools/autoplay/` tree |
| **What it adds** | Window size tracking + win32 quality verdict in `analyze_evidence_run.py`; integrity guards in `validate_composite_run.py`; 25-test suite |
| **Rebase needed** | Yes — branch was based on `2ce3dc6b`; must rebase onto `79031021` before FF merge |

### 3.4 Placement-Reject Recipe

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-placement-reject-recipe-1881` (PROMPT 1881) |
| **Base** | `origin/main@2ce3dc6b` |
| **Status** | **PENDING — not merged to main** |
| **Verified absent** | `tools/autoplay/recipes/placement_reject_probe.py`: not in `tools/autoplay/recipes/` tree on `origin/main@79031021` |
| **Verified absent** | `BOARD_DEEP_CELL` coord: not in `tools/autoplay/recipes/_coords.py` on main |
| **Verified absent** | Registry: only `placement_drag_probe` in `recipes/__init__.py`; `placement_reject_probe` absent |
| **What it adds** | `tools/autoplay/recipes/placement_reject_probe.py` (121 lines); `placement_reject_probe` REGISTRY entry; `BOARD_DEEP_CELL` coord |
| **Rebase needed** | Yes — branch was based on `2ce3dc6b`; must rebase onto `79031021` before FF merge |

---

## 4. Story Status Table

### Story 001 — Autoplay Driver Foundation

| Field | Value |
|-------|-------|
| **Status** | **DONE (main)** — core driver, recipe framework, composite harness on main since early sprints |
| **Evidence analyzer** | `tools/autoplay/analyze_evidence_run.py` on main since PROMPT 1833 (`b856eef4`) |
| **AC-VPT-01 repair** | `enforce_autoplay_window_size_system` — on main since PROMPT 1912 |
| **AC-VPT-02/08 repair** | `EXIT_VIEWPORT_GUARD` / viewport guards — on main since PROMPT 1880/1894 |
| **Window-resize verdict extension** | In `integrate/autoplay-composite-window-resize-verdict-1875` — rebase needed, not yet main |

---

### Story 002 — AUTOPLAY-VS-BOT-QA-001 (bot game clean run)

| Field | Value |
|-------|-------|
| **Status** | **BLOCKED — no fresh run with repaired driver yet** |
| **Blocker** | No run achieves automated PASS. All three 2026-05-28 runs are PARTIAL (PROMPT 1846 §3–5). No fresh run executed with the repaired AC-VPT-01/02/08 driver. |
| **AC-VPT-01 repair** | **ON MAIN** since PROMPT 1912 |
| **AC-VPT-02/08 repair** | **ON MAIN** since PROMPT 1880/1894 |
| **Composite verdict tool** | PROMPT 1875 — pushed to origin; rebase needed for `79031021`; **not on main** |
| **Remaining repair gap** | Composite verdict tool not yet on main. No fresh autoplay run executed with repaired driver. |

**Path to DONE:**
1. Rebase + merge `integrate/autoplay-composite-window-resize-verdict-1875` onto `79031021`
2. Execute fresh autoplay run; driver must exit 0, analyzer must return PASS verdict

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|-------|-------|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** |
| **Recipe branch** | `origin/integrate/autoplay-placement-reject-recipe-1881` |
| **main status** | `placement_reject_probe.py` absent from `origin/main@79031021` |

**Path to DONE:** Rebase `integrate/autoplay-placement-reject-recipe-1881` onto
`79031021`, FF merge, then run the recipe against a live game to produce pass evidence.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Full bot game live-pass signoff)

| Field | Value |
|-------|-------|
| **Status** | **BLOCKED — no automated PASS yet** |
| **Blocking condition** | Analyzer returns PARTIAL for all three available runs (PROMPT 1846 §6) |
| **Human-review evidence** | Run `090613` — conditional only; bitblt PNGs show distinct content; requires human inspection |
| **Dependencies** | Story 002 must reach PASS first |

---

## 5. Integration Branch Summary Table

| Branch | PROMPT | What | On main | Notes |
|--------|--------|------|---------|-------|
| `integrate/autoplay-window-size-default-1893` | 1912 | AC-VPT-01 startup size floor | **YES** | Landed via PROMPT 1912 (`e02d132f`) |
| `integrate/autoplay-click-viewport-guard-1880` | 1880/1894 | AC-VPT-02/08 drift + OOB guards | **YES** | Landed via PROMPT 1880/1894 (`e8a40f81`) |
| `integrate/autoplay-composite-window-resize-verdict-1875` | 1875 | Window verdict in analyzer/validator | **NO** | Rebase onto `79031021` needed |
| `integrate/autoplay-placement-reject-recipe-1881` | 1881 | `placement_reject_probe` recipe | **NO** | Rebase onto `79031021` needed |
| `report/bot-autoplay-readiness-refresh-1907` | 1907 | Prior readiness report | NO (branch) | Report file IS on main; branch not FF |

---

## 6. Validation Checklist

| Check | Result |
|-------|--------|
| No sentence claims PROMPT 1831 / run `090613` as clean automated PASS | PASS — §2.2 explicitly classifies as conditional human-review only |
| C0/human-review caveat preserved from PROMPT 1831/1840 | PASS — §2.2 |
| Report references PROMPT 1844 as current evidence truth | PASS — §2 |
| Report references PROMPT 1846 as current evidence truth | PASS — §2.1 |
| PROMPT 1931 truth correction landing noted | PASS — §1, §2.3 |
| AC-VPT-01 now on main — verified by code inspection | PASS — §3.1 |
| AC-VPT-02/08 now on main — verified by code inspection | PASS — §3.2 |
| Composite verdict status against current main stated explicitly | PASS — §3.3 |
| Placement-reject recipe status against current main stated explicitly | PASS — §3.4 |
| Story 004 (AUTOPLAY-VS-BOT-QA-001) shown as BLOCKED | PASS — §4 Story 004 |
| Existing landed reports not deleted | PASS — report-only branch, one file added |
| Report-only (no sprint/story/tools/source files touched) | PASS |

---

## 7. Open Items and Merge Queue

| Item | Priority | Branch | Status |
|------|----------|--------|--------|
| Rebase + merge `integrate/autoplay-composite-window-resize-verdict-1875` onto `79031021` | BLOCKING | origin — rebase needed | FF merge pending |
| Execute fresh autoplay run with repaired driver; expect PASS verdict | GATE | — | Blocked on composite verdict merge |
| Human review of bitblt/Bevy PNGs from fresh run | GATE | — | Blocked on clean run |
| Rebase + merge `integrate/autoplay-placement-reject-recipe-1881` (Story 003) | NORMAL | origin — rebase needed | FF merge pending |

---

1935: BOT-AUTOPLAY-STORY-READINESS-REPORT-REFRESH-AFTER-1931: SHIPPED
