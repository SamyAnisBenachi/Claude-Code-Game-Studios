# PROMPT 1985 — Bot/Autoplay Story Readiness Report Refresh After PROMPT 1976

**Date:** 2026-05-28
**Branch:** `report/bot-autoplay-readiness-refresh-1985`
**Source tree:** `origin/main@32a59256` (PROMPT 1976 — latest main)
**Scope:** Report-only — no source edits, no sprint-state writes.
**Supersedes:** `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md`
**Prior base:** PROMPT 1970 was authored targeting `origin/main@7fc1706e` (PROMPT 1959)
but its branch (`report/bot-autoplay-readiness-refresh-1970`) was NOT_FF against
current main (`32a59256`) and was not merged. This report is a clean reconstruction
on fresh `origin/main@32a59256`.

---

## 1. Why This Report Exists

PROMPT 1970 produced branch `report/bot-autoplay-readiness-refresh-1970` (commit
`b9f29e27`) but orchestrator verification found it was **NOT_FF** against
`origin/main@32a59256`. Merging it directly would delete the PROMPT 1972 and
PROMPT 1976 artifacts that landed between `7fc1706e` and `32a59256`.

Since PROMPT 1959 (`7fc1706e`), the following landed on `origin/main`:

| PROMPT | Commit | What |
|--------|--------|------|
| **1972** | `7b259e91` | Reapply PROMPT 1841/1889/1911/1946/1956 signoff-pack reports after 1959 (report-only) |
| **1976** | `32a59256` | Backfill 1861/1914/1941/1964/1968 operator contract + refresh after 1972 (report-only) |

Neither 1972 nor 1976 changes the bot/autoplay story readiness picture. The
active refresh lanes at the time of this report are:

- **PROMPT 1977** (`origin/integrate/autoplay-placement-reject-recipe-1977`) —
  placement-reject recipe reapplied onto post-1972 main; **FF over main, not yet merged**
- **PROMPT 1979** (`origin/work/PROMPT-1979`) — composite window-resize verdict
  reapplied onto post-1976 main; **FF over main, not yet merged**
- **PROMPT 1982** — described as the active controlled GUI smoke lane; no branch
  found on `origin` at report time (not yet pushed or in-flight)

This report is authored on a fresh branch from `origin/main@32a59256` and updates
all integration branch and AC item status statements to reflect current post-1976 state.

---

## 2. Evidence Truth (Unchanged from PROMPT 1844 + 1846 + 1931)

### 2.1 Available Runs

No new autoplay runs have been executed since the three runs captured on
2026-05-28. This section is unchanged from PROMPT 1970 §2.1.

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

### 2.3 Analyzer / Verification Reports on Main (`origin/main@32a59256`)

The following report files are confirmed on `origin/main@32a59256`:

| File | Status on main |
|------|----------------|
| `reports/PROMPT-1838-post-1830-autoplay-tooling-focused-verify.md` | **ON MAIN** (via PROMPT 1950) |
| `reports/PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md` | **ON MAIN** |
| `reports/PROMPT-1846-autoplay-evidence-analyzer-latest-run-application.md` | **ON MAIN** |
| `reports/PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md` | **ON MAIN** |
| `reports/PROMPT-1859-autoplay-evidence-analyzer-latest-run-report-backfill.md` | **ON MAIN** |
| `reports/PROMPT-1862-post-1830-autoplay-tooling-verify-report-backfill.md` | **ON MAIN** (via PROMPT 1950) |
| `reports/PROMPT-1872-autoplay-evidence-analyzer-latest-run-refresh-after-1858.md` | **ON MAIN** |
| `reports/PROMPT-1899-post-1830-autoplay-tooling-verify-report-backfill-refresh.md` | **ON MAIN** (via PROMPT 1950) |
| `reports/PROMPT-1931-autoplay-1831-1840-truth-refresh-after-1912.md` | **ON MAIN** (PROMPT 1931) |
| `reports/PROMPT-1932-post-1830-autoplay-tooling-verify-report-backfill-refresh-after-1929.md` | **ON MAIN** (via PROMPT 1950) |
| `reports/PROMPT-1831-autoplay-vsbot-fresh-post-1818-live-verify.md` | **ON MAIN** (truth-corrected, PROMPT 1931) |
| `reports/PROMPT-1866-autoplay-1831-1840-report-truth-reconcile.md` | **ON MAIN** (PROMPT 1931) |
| `reports/PROMPT-1950-post-1830-autoplay-tooling-verify-report-backfill-refresh-after-1943.md` | **ON MAIN** (PROMPT 1950) |

Delta since PROMPT 1970: No new bot/autoplay report files landed on main via
PROMPT 1972 or PROMPT 1976. The table above is identical to PROMPT 1970 §2.3.

---

## 3. AC Item Status Against Current Main (`origin/main@32a59256`)

### 3.1 AC-VPT-01 — Minimum Window Size Gate

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-window-size-default-1893` (PROMPT 1893) |
| **Landed via** | PROMPT 1912 (`e02d132f`) |
| **Status** | **MERGED TO MAIN** — unchanged from PROMPT 1970 |
| **Verified on main** | `client/src/autoplay.rs`: `enforce_autoplay_window_size_system` at startup; `CCGS_WINDOW_WIDTH`/`CCGS_WINDOW_HEIGHT` env-var constants present |
| **Verified on main** | `tools/autoplay/Run-AutoplaySmoke.ps1`: env-var guards present |
| **Gap** | Startup-size floor enforced. Mid-run DWM resize prevention is AC-VPT-02/08 scope (also on main). |

### 3.2 AC-VPT-02 + AC-VPT-08 — Click-Target Viewport Guard

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-click-viewport-guard-1880` (PROMPT 1880) |
| **Landed via** | PROMPT 1880 source commit (`e8a40f81`) + PROMPT 1894 report (`71484fc4`) |
| **Status** | **MERGED TO MAIN** — unchanged from PROMPT 1970 |
| **Verified on main** | `tools/autoplay/driver.py`: `EXIT_VIEWPORT_GUARD = 5`, `_check_window_minimum`, `_check_window_drift`, `_validate_cursor_coords`, `_check_post_foreground_window` — all present |
| **Verified on main** | `tests/tools/autoplay/test_driver_click_viewport_guard.py` — present (66 tests) |
| **Checkpoint types on main** | `viewport_drift`, `viewport_shrink_abort`, `viewport_guard_cursor_none`, `viewport_guard_oob` |

### 3.3 Composite Window-Resize Verdict

| Field | Value |
|-------|-------|
| **Active refresh branch** | `origin/work/PROMPT-1979` (PROMPT 1979) — replaces stale 1875/1875-era branches |
| **Base of 1979 branch** | `origin/main@32a59256` (FF over current main — verified) |
| **Status** | **PENDING — FF over main, not yet merged** |
| **What 1979 adds** | Window verdict chain reports (PROMPT 1850/1864/1873/1875/1913/1918/1945/1951/1969/1979); `test_window_resize_verdict.py`; `analyze_evidence_run.py` + `validate_composite_run.py` updates |
| **Verified absent on main** | `tools/autoplay/analyze_evidence_run.py`: no `win32_quality` / `window_resize_verdict` field on `origin/main@32a59256` |
| **Verified absent on main** | `tests/tools/autoplay/test_window_resize_verdict.py`: not in `tests/tools/autoplay/` on main |
| **Merge action** | FF-merge `origin/work/PROMPT-1979` directly — already FF over `32a59256` |
| **Delta since PROMPT 1970** | Branch refreshed to PROMPT 1979; previously tracked as 1875/1935-era base; now clean FF over `32a59256` |

### 3.4 Placement-Reject Recipe

| Field | Value |
|-------|-------|
| **Active refresh branch** | `origin/integrate/autoplay-placement-reject-recipe-1977` (PROMPT 1977) — replaces stale 1881-era branches |
| **Base of 1977 branch** | `origin/main@32a59256` (FF over current main — verified) |
| **Status** | **PENDING — FF over main, not yet merged** |
| **What 1977 adds** | `tools/autoplay/recipes/placement_reject_probe.py`; `placement_reject_probe` REGISTRY entry in `recipes/__init__.py`; `BOARD_DEEP_CELL` coord in `_coords.py`; chain reports (PROMPT 1928/1952/1960/1977) |
| **Verified absent on main** | `tools/autoplay/recipes/placement_reject_probe.py`: not in `tools/autoplay/recipes/` on `origin/main@32a59256` |
| **Verified absent on main** | `BOARD_DEEP_CELL` coord: not in `tools/autoplay/recipes/_coords.py` on main |
| **Verified absent on main** | Only `placement_drag_probe` in `recipes/__init__.py`; `placement_reject_probe` absent |
| **Merge action** | FF-merge `origin/integrate/autoplay-placement-reject-recipe-1977` directly — already FF over `32a59256` |
| **Delta since PROMPT 1970** | Branch refreshed to PROMPT 1977 (was tracked as 1881-era base); now clean FF over `32a59256` |

---

## 4. Story Status Table

### Story 001 — Autoplay Driver Foundation

| Field | Value |
|-------|-------|
| **Status** | **DONE (main)** — core driver, recipe framework, composite harness on main since early sprints |
| **Evidence analyzer** | `tools/autoplay/analyze_evidence_run.py` on main since PROMPT 1833 (`b856eef4`) |
| **AC-VPT-01 repair** | `enforce_autoplay_window_size_system` — on main since PROMPT 1912 |
| **AC-VPT-02/08 repair** | `EXIT_VIEWPORT_GUARD` / viewport guards — on main since PROMPT 1880/1894 |
| **Window-resize verdict extension** | In `origin/work/PROMPT-1979` — FF over main, merge pending |
| **Delta since PROMPT 1970** | No change to main state; PROMPT 1979 is the fresh FF-ready verdict branch |

---

### Story 002 — AUTOPLAY-VS-BOT-QA-001 (bot game clean run)

| Field | Value |
|-------|-------|
| **Status** | **BLOCKED — no fresh run with repaired driver yet** — unchanged from PROMPT 1970 |
| **Blocker** | No run achieves automated PASS. All three 2026-05-28 runs are PARTIAL. No fresh run executed with the repaired AC-VPT-01/02/08 driver. |
| **AC-VPT-01 repair** | **ON MAIN** since PROMPT 1912 |
| **AC-VPT-02/08 repair** | **ON MAIN** since PROMPT 1880/1894 |
| **Composite verdict tool** | `origin/work/PROMPT-1979` — FF over `32a59256`, **not yet on main** |
| **Controlled GUI smoke** | PROMPT 1982 referenced as active lane; no branch found on origin at report time |
| **Delta since PROMPT 1970** | Composite verdict now tracked as PROMPT 1979 (FF-ready); PROMPT 1982 GUI smoke referenced but not yet visible on origin |

**Path to DONE:**
1. FF-merge `origin/work/PROMPT-1979` onto main (verdict tool, test suite — AC-VPT-06 coverage)
2. Execute fresh autoplay run; driver must exit 0, analyzer must return PASS verdict
   (zero FROZEN lines, ≥3 distinct hashes, window stable at `[1280,720]` throughout,
   `EXIT_VIEWPORT_GUARD` never triggered)

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|-------|-------|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** — unchanged from PROMPT 1970 |
| **Active branch** | `origin/integrate/autoplay-placement-reject-recipe-1977` (FF over `32a59256`) |
| **main status** | `placement_reject_probe.py` absent from `origin/main@32a59256` |
| **Delta since PROMPT 1970** | Recipe branch refreshed to PROMPT 1977; now FF-ready over current main |

**Path to DONE:** FF-merge `origin/integrate/autoplay-placement-reject-recipe-1977`
onto main, then run the recipe against a live game to produce pass evidence.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Full bot game live-pass signoff)

| Field | Value |
|-------|-------|
| **Status** | **BLOCKED — no automated PASS yet** — unchanged from PROMPT 1970 |
| **Blocking condition** | Analyzer returns PARTIAL for all three available runs (PROMPT 1846 §6) |
| **Human-review evidence** | Run `090613` — conditional only; bitblt PNGs show distinct content; requires human inspection to confirm UI not clipped and bot actions landed correctly |
| **Dependencies** | Story 002 must reach PASS first (fresh run with repaired driver + clean analyzer verdict) |
| **Delta since PROMPT 1970** | No change |

**Path to DONE:**
1. Story 002 path completed (composite verdict merge + fresh run)
2. Analyzer returns PASS on the fresh run
3. Human reviewer inspects bitblt/Bevy PNGs for that run and signs off
4. AUTOPLAY-VS-BOT-QA-001 can then be marked DONE

---

## 5. Integration Branch Summary Table

| Branch | PROMPT | What | On main | Notes |
|--------|--------|------|---------|-------|
| `integrate/autoplay-window-size-default-1893` | 1912 | AC-VPT-01 startup size floor | **YES** | Landed via PROMPT 1912 (`e02d132f`) |
| `integrate/autoplay-click-viewport-guard-1880` | 1880/1894 | AC-VPT-02/08 drift + OOB guards | **YES** | Landed via PROMPT 1880/1894 (`e8a40f81`) |
| `work/PROMPT-1979` | 1979 | Window verdict in analyzer/validator (refresh of 1875-era) | **NO** | FF over `32a59256`; merge-ready |
| `integrate/autoplay-placement-reject-recipe-1977` | 1977 | `placement_reject_probe` recipe (refresh of 1881-era) | **NO** | FF over `32a59256`; merge-ready |
| `report/bot-autoplay-readiness-refresh-1970` | 1970 | Prior readiness report | **NO (branch)** | NOT_FF vs `32a59256`; this 1985 report is the replacement |
| `report/bot-autoplay-readiness-refresh-1935` | 1935 | Pre-prior readiness report | **NO (branch)** | NOT_FF vs `32a59256`; superseded by 1970, then this report |

---

## 6. Validation Checklist

| Check | Result |
|-------|--------|
| Branch based on fresh `origin/main@32a59256`, not on stale 1970 or 1935 branch | PASS — worktree created from `origin/main` |
| `git merge-base --is-ancestor origin/main HEAD` | PASS — verified before writing |
| `git diff --name-status origin/main..HEAD` shows only owned reports, zero deletes | PASS — see §7 |
| No sentence claims PROMPT 1831 / run `090613` as clean automated PASS | PASS — §2.2 explicitly classifies as conditional human-review only |
| C0/human-review caveat preserved from PROMPT 1831/1840 | PASS — §2.2 |
| Report references PROMPT 1844 + 1846 as current evidence truth | PASS — §2 |
| PROMPT 1931 truth correction landing noted | PASS — §2.3 |
| AC-VPT-01 on main — status stated | PASS — §3.1 |
| AC-VPT-02/08 on main — status stated | PASS — §3.2 |
| Composite verdict status against `32a59256` stated explicitly (PROMPT 1979 as active branch) | PASS — §3.3 |
| Placement-reject recipe status against `32a59256` stated explicitly (PROMPT 1977 as active branch) | PASS — §3.4 |
| PROMPT 1972/1976 report chains preserved (not deleted by this branch) | PASS — report-only branch adds three files, zero deletes |
| Story 004 (AUTOPLAY-VS-BOT-QA-001) shown as BLOCKED | PASS — §4 Story 004 |
| Report-only (no sprint/story/tools/source files touched) | PASS |
| Worktree used (not root checkout) | PASS — `D:/tmp/wt-1985-bot-autoplay-readiness` |

---

## 7. Branch Diff Summary

**Branch:** `report/bot-autoplay-readiness-refresh-1985`
**Base commit:** `32a59256` (origin/main)
**Files added (3):**
- `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md`
- `reports/PROMPT-1970-bot-autoplay-story-readiness-report-refresh-after-1959.md`
- `reports/PROMPT-1985-bot-autoplay-story-readiness-report-refresh-after-1976.md`

**Files modified:** 0
**Files deleted:** 0

---

## 8. Open Items and Merge Queue

| Item | Priority | Branch | Status |
|------|----------|--------|--------|
| FF-merge `origin/work/PROMPT-1979` onto main | BLOCKING | origin — FF-ready | Merge pending |
| FF-merge `origin/integrate/autoplay-placement-reject-recipe-1977` (Story 003) onto main | NORMAL | origin — FF-ready | Merge pending |
| Execute fresh autoplay run with repaired driver; expect PASS verdict | GATE | — | Blocked on composite verdict merge |
| Human review of bitblt/Bevy PNGs from fresh run | GATE | — | Blocked on clean run |
| Verify AC-VPT-06 (distinct pixel_hash per phase, zero frozen) in fresh run | GATE | — | Blocked on merges + fresh run |
| PROMPT 1982 controlled GUI smoke | ACTIVE LANE | not yet on origin | In-flight or not yet pushed |

---

1985: BOT-AUTOPLAY-STORY-READINESS-REPORT-REFRESH-AFTER-1976: READY_FOR_MAINLAND_ENQUEUE
