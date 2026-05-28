# PROMPT 1891 — Bot/Autoplay Story Readiness Refresh After PROMPT 1872

**Date:** 2026-05-28
**Branch:** `report/bot-autoplay-readiness-refresh-1891`
**Source tree:** `origin/main@2ce3dc6b` (PROMPT 1872 — latest main)
**Scope:** Report-only — no source edits, no sprint-state writes.
**Supersedes:** `reports/PROMPT-1870-bot-autoplay-story-readiness-refresh-after-1858.md`
**Prior base:** PROMPT 1870 was authored on `origin/main@5c91918d`

---

## 1. Why This Report Exists

PROMPT 1870 shipped a useful readiness refresh on
`origin/report/bot-autoplay-readiness-refresh-1870` but that branch is
**not FF-ready** over current main (`2ce3dc6b`) — merging it directly would
delete the PROMPT 1846/1859/1872 analyzer report artifacts that landed between
`5c91918d` and `2ce3dc6b`.

Since PROMPT 1870, the following landed on `origin/main`:
- **PROMPT 1872** (`2ce3dc6b`): Re-applied PROMPT 1846 and PROMPT 1859 analyzer
  reports as a clean cherry-pick, preserving PROMPT 1845/1858 artifacts.

And since PROMPT 1872, the following **integration branches** were authored but
**not yet merged to main**:
- PROMPT 1875 — composite window-resize verdict refresh
- PROMPT 1877 — 1831/1840 truth refresh after 1872
- PROMPT 1879 — AC-VPT-01 window-size default repair refresh
- PROMPT 1880 — AC-VPT-02/08 click-target viewport guard refresh
- PROMPT 1881 — placement-reject recipe refresh

This report is authored on a fresh branch from `origin/main@2ce3dc6b` and
updates all integration branch status statements to reflect the current state.

---

## 2. Evidence Truth (Unchanged from PROMPT 1844 + 1846 + 1872)

### 2.1 Available Runs

No new autoplay runs have been executed since the three runs captured on 2026-05-28.
This section is identical to PROMPT 1870 §2.1.

| Run | Window size | Checkpoints | Analyzer verdict | Automated PASS? |
|---|---|---|---|---|
| `20260528-051148-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — no capture labels, no pixel_hash | NO |
| `20260528-063609-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — all 15 hashes identical (frozen renderer) | NO |
| `20260528-090613-Z` | `[1280,720]` → `[1280,1076]` mid-run | 15/15 | **PARTIAL** — 11/15 PrintWindow captures frozen; 11 distinct bitblt hashes | NO (conditional human-review only) |

### 2.2 Run `090613` Classification (UNCHANGED — CRITICAL)

Run `090613` is the **best available human-review evidence** but is **not a clean
automated PASS**. The PROMPT 1870 §2.2 classification stands verbatim:

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

**Prohibited citation:** Any sentence claiming `090613` or PROMPT 1831 as a clean
automated PASS, a clean smoke PASS, or as proof of correct bot UI interaction.

### 2.3 Analyzer Reports on Main

PROMPT 1872 (`2ce3dc6b`) successfully landed the following analyzer report files
onto `origin/main`:

| File | Status on main |
|---|---|
| `reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md` | **ON MAIN** |
| `reports/PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md` | **ON MAIN** |
| `reports/PROMPT-1872-autoplay-evidence-analyzer-latest-run-refresh-after-1858.md` | **ON MAIN** |
| `reports/PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md` | **ON MAIN** |
| `reports/PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md` | **ON MAIN** |

---

## 3. AC Item Status Against Current Main (`origin/main@2ce3dc6b`)

### 3.1 AC-VPT-01 — Minimum Window Size Gate

| Field | Value |
|---|---|
| **Prior branch** | `integrate/autoplay-window-size-default-1865` (based on `bb90d7c2`) |
| **Refresh branch** | `integrate/autoplay-window-size-default-1879` (PROMPT 1879) |
| **Base** | `origin/main@2ce3dc6b` (latest main) |
| **Pushed to origin** | YES — `origin/integrate/autoplay-window-size-default-1879` @ `dd7b12cb` |
| **FF-ready over main** | YES — single commit ahead of `2ce3dc6b` |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | `enforce_autoplay_window_size_system` startup system in `client/src/autoplay.rs`; env-var `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT` guards in `Run-AutoplaySmoke.ps1` |
| **Gap** | Startup-size floor enforced; mid-run DWM resize NOT prevented (that is AC-VPT-02 scope) |
| **Merge action** | `git merge --ff-only integrate/autoplay-window-size-default-1879` |

### 3.2 AC-VPT-02 + AC-VPT-08 — Click-Target Viewport Guard

| Field | Value |
|---|---|
| **Prior branch** | `integrate/autoplay-click-viewport-guard-1857` (based on `bb90d7c2`) |
| **Refresh branch** | `integrate/autoplay-click-viewport-guard-1880` (PROMPT 1880) |
| **Base** | `origin/main@2ce3dc6b` (latest main) |
| **Pushed to origin** | YES — `origin/integrate/autoplay-click-viewport-guard-1880` @ `4dfdb28c` |
| **FF-ready over main** | YES — single commit ahead of `2ce3dc6b` |
| **Test suite** | 66/66 unit tests pass (focused run; no GUI, no Cargo) |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | `EXIT_VIEWPORT_GUARD=5` exit code; `_check_window_minimum`, `_check_window_drift`, `_validate_cursor_coords`, `_check_post_foreground_window` guards in `tools/autoplay/driver.py`; structured `viewport_drift` / `viewport_shrink_abort` / `viewport_guard_cursor_none` / `viewport_guard_oob` checkpoints |
| **Merge action** | `git merge --ff-only integrate/autoplay-click-viewport-guard-1880` |

### 3.3 Composite Window-Resize Verdict

| Field | Value |
|---|---|
| **Prior branch** | `integrate/autoplay-composite-window-resize-verdict-1873` (based on `5c91918d`) |
| **Refresh branch** | `integrate/autoplay-composite-window-resize-verdict-1875` (PROMPT 1875) |
| **Base** | `origin/main@2ce3dc6b` (latest main) |
| **Pushed to origin** | YES — `origin/integrate/autoplay-composite-window-resize-verdict-1875` @ `4cccb1e5` |
| **FF-ready over main** | YES — commits ahead of `2ce3dc6b` |
| **Test suite** | 25/25 tests pass |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | Window size tracking + win32 quality verdict in `tools/autoplay/analyze_evidence_run.py`; integrity guards in `tools/autoplay/validate_composite_run.py`; `tests/tools/autoplay/test_window_resize_verdict.py` (25 tests) |
| **Merge action** | `git merge --ff-only integrate/autoplay-composite-window-resize-verdict-1875` |

### 3.4 Placement-Reject Recipe

| Field | Value |
|---|---|
| **Prior branch** | `integrate/autoplay-placement-reject-recipe-1860` (based on `bb90d7c2`) |
| **Refresh branch** | `integrate/autoplay-placement-reject-recipe-1881` (PROMPT 1881) |
| **Base** | `origin/main@2ce3dc6b` (latest main) |
| **Pushed to origin** | YES — `origin/integrate/autoplay-placement-reject-recipe-1881` @ `b189d252` |
| **FF-ready over main** | YES — commits ahead of `2ce3dc6b` |
| **Registry** | 13 recipes; `placement-reject-probe` present |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | `tools/autoplay/recipes/placement_reject_probe.py` (121 lines); `placement_reject_probe` REGISTRY entry in `__init__.py`; `BOARD_DEEP_CELL` coord in `_coords.py` |
| **Merge action** | `git merge --ff-only integrate/autoplay-placement-reject-recipe-1881` |

---

## 4. Story Status Table

### Story 001 — Autoplay Driver Foundation

| Field | Value |
|---|---|
| **Status** | **DONE (main)** — core driver, recipe framework, composite harness on main since early sprints |
| **Evidence analyzer** | `tools/autoplay/analyze_evidence_run.py` on main since PROMPT 1833 (`b856eef4`) |
| **Window-resize verdict extension** | In `integrate/autoplay-composite-window-resize-verdict-1875` — FF-ready, not yet main |

---

### Story 002 — AUTOPLAY-VS-BOT-QA-001 (bot game clean run)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — pending repairs + fresh run** |
| **Blocker** | No run achieves automated PASS. All three 2026-05-28 runs are PARTIAL (PROMPT 1846 §3–5). |
| **AC-VPT-01 repair** | PROMPT 1879 — pushed to `origin/integrate/autoplay-window-size-default-1879`; FF-ready; **not on main** |
| **AC-VPT-02/08 repair** | PROMPT 1880 — pushed to `origin/integrate/autoplay-click-viewport-guard-1880`; FF-ready; 66/66 tests pass; **not on main** |
| **Composite verdict tool** | PROMPT 1875 — pushed to `origin/integrate/autoplay-composite-window-resize-verdict-1875`; FF-ready; 25/25 tests; **not on main** |
| **Remaining repair gap** | None of the AC-VPT branches has been merged to main. No fresh autoplay run has been executed with the repaired driver. |

**Path to DONE:**
1. Merge `integrate/autoplay-window-size-default-1879` (AC-VPT-01, FF-ready)
2. Merge `integrate/autoplay-click-viewport-guard-1880` (AC-VPT-02/08, FF-ready, 66 tests)
3. Merge `integrate/autoplay-composite-window-resize-verdict-1875` (verdict tool, FF-ready, 25 tests)
4. Execute fresh run; driver must exit 0, analyzer must return PASS verdict (zero FROZEN
   lines, ≥3 distinct hashes, window stable at `[1280,720]` throughout,
   `EXIT_VIEWPORT_GUARD` never triggered)

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|---|---|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** |
| **Recipe branch** | `origin/integrate/autoplay-placement-reject-recipe-1881` |
| **Recipe HEAD** | `b189d252` (PROMPT 1881 report) / `23c3e901` (PROMPT 1881 feat) |
| **FF-ready** | YES — based on main@`2ce3dc6b`; no conflict |
| **main status** | `placement_reject_probe.py` absent from `origin/main@2ce3dc6b` |
| **Registry** | 13 recipes including `placement-reject-probe` on the branch |

**Path to DONE:** Merge `integrate/autoplay-placement-reject-recipe-1881` to main
(FF merge, no conflicts). Then run the recipe against a live game to produce pass evidence.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Full bot game live-pass signoff)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — no automated PASS yet** |
| **Blocking condition** | Analyzer returns PARTIAL for all three available runs (PROMPT 1846 §6) |
| **Human-review evidence** | Run `090613` — conditional only; bitblt PNGs show distinct content; requires human inspection to confirm UI not clipped and bot actions landed correctly |
| **Dependencies** | Story 002 must reach PASS first (all AC-VPT repair branches on main + clean fresh run) |

**Path to DONE:**
1. Story 002 path completed (merges + fresh run with repaired driver)
2. Analyzer returns PASS on the fresh run
3. Human reviewer inspects bitblt/Bevy PNGs for that run and signs off
4. AUTOPLAY-VS-BOT-QA-001 can then be marked DONE

---

## 5. Integration Branch Summary Table

| Branch | PROMPT | What | Pushed | FF-ready | On main |
|---|---|---|---|---|---|
| `integrate/autoplay-window-size-default-1879` | 1879 | AC-VPT-01 startup size floor | YES | YES | NO |
| `integrate/autoplay-click-viewport-guard-1880` | 1880 | AC-VPT-02/08 mid-run drift + OOB guards | YES | YES | NO |
| `integrate/autoplay-composite-window-resize-verdict-1875` | 1875 | Window verdict in analyzer/validator | YES | YES | NO |
| `integrate/autoplay-placement-reject-recipe-1881` | 1881 | `placement_reject_probe` recipe | YES | YES | NO |
| `report/bot-autoplay-readiness-refresh-1870` | 1870 | Prior readiness report | YES | NO (not FF over 2ce3dc6b) | NO |

**Note on 1870 branch:** The prior readiness report branch is published on origin but is
not FF-ready over current main (`2ce3dc6b`) — merging it directly would delete the
PROMPT 1846/1859/1872 analyzer artifacts. This report (1891) supersedes 1870 and must
be used as the authoritative readiness document.

---

## 6. Validation Checklist

| Check | Result |
|---|---|
| No sentence claims PROMPT 1831 / run `090613` as clean automated PASS | PASS — §2.2 explicitly classifies as conditional human-review only |
| Report references PROMPT 1844 as current evidence truth | PASS — §2 |
| Report references PROMPT 1846 as current evidence truth | PASS — §2.1, §2.3, Story 002 |
| PROMPT 1872 landing correctly noted | PASS — §1, §2.3 |
| AC-VPT-01 status against current main stated explicitly | PASS — §3.1 |
| AC-VPT-02/08 status against current main stated explicitly | PASS — §3.2 |
| Placement-reject recipe status against current main stated explicitly | PASS — §3.4 |
| Story 004 (AUTOPLAY-VS-BOT-QA-001) shown as BLOCKED | PASS — §4 Story 004 |
| PROMPT 1845/1858/1872 artifacts not deleted | PASS — report-only branch, one file added |
| Report-only (no sprint/story/tools/source files touched) | PASS |
| Worktree used (not root checkout) | PASS — `D:/tmp/wt-1891-bot-autoplay-readiness` |

---

## 7. Open Items and Merge Queue

| Item | Priority | Branch | Status |
|---|---|---|---|
| Merge `integrate/autoplay-window-size-default-1879` (AC-VPT-01) | BLOCKING | pushed to origin | FF-ready, awaiting merge |
| Merge `integrate/autoplay-click-viewport-guard-1880` (AC-VPT-02/08) | BLOCKING | pushed to origin | FF-ready, 66/66 tests, awaiting merge |
| Merge `integrate/autoplay-composite-window-resize-verdict-1875` (verdict tool) | BLOCKING | pushed to origin | FF-ready, 25/25 tests, awaiting merge |
| Merge `integrate/autoplay-placement-reject-recipe-1881` (Story 003) | NORMAL | pushed to origin | FF-ready, awaiting merge |
| Execute fresh autoplay run post-repairs; expect PASS verdict from analyzer | GATE | — | Blocked on above merges |
| Human review of bitblt/Bevy PNGs from fresh run | GATE | — | Blocked on clean run |
| Verify AC-VPT-06 (distinct pixel_hash per phase, zero frozen) in fresh run | GATE | — | Blocked on merges + fresh run |

---

1891: BOT-AUTOPLAY-STORY-READINESS-REFRESH-AFTER-1872: SHIPPED
