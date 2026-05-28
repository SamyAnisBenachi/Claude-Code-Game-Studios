# PROMPT 1970 — Bot/Autoplay Story Readiness Report Refresh After PROMPT 1959

**Date:** 2026-05-28
**Branch:** `report/bot-autoplay-readiness-refresh-1970`
**Source tree:** `origin/main@7fc1706e` (PROMPT 1959 — latest main)
**Scope:** Report-only — no source edits, no sprint-state writes.
**Supersedes:** `reports/PROMPT-1935-bot-autoplay-story-readiness-report-refresh-after-1931.md`
**Prior base:** PROMPT 1935 was authored targeting `origin/main@79031021` (PROMPT 1931)
but its branch (`report/bot-autoplay-readiness-refresh-1935`) was NOT_FF against
current main and was not merged. This report is a clean reconstruction on fresh
`origin/main@7fc1706e`.

---

## 1. Why This Report Exists

PROMPT 1935 produced branch `report/bot-autoplay-readiness-refresh-1935` (commit
`799eb078`) but orchestrator verification found it was **NOT_FF** against
`origin/main@7fc1706e`. Merging it would delete PROMPT 1957 and 1959 artifacts
and the PROMPT 1957 test artifact that landed after `79031021`.

Since PROMPT 1931 (`79031021`), the following landed on `origin/main`:

| PROMPT | Commit | What |
|--------|--------|------|
| **1937** | `b58cdd66` | QA snapshot observability gap report refresh (report-only) |
| **1939** | `be40e0c6` | Two-client launcher stale-binary rebuild guard (feat) |
| **1943** | `e62c431e` | Two-client retest reports backfill (PROMPT 1883/1903 reports) |
| **1950** | `241e33a8` | Autoplay tooling verify reports backfill (PROMPT 1838/1862/1899/1932 reports) |
| **1852** | `49aeb4f0` | feat(ui/card-inspect): keyword glossary definitions panel |
| **1868** | `097a7b74` | Card inspect hover glossary integration refresh (report) |
| **1920** | `1c4981a6` | Card inspect hover glossary refresh after 1912 (report) |
| **1957** | `449688dd` + `2bf3960d` | feat+report: krosmaga auction tier-border asset binding |
| **1959** | `7fc1706e` | Krosmaga UI stage3 slices report backfill refresh after 1920 |

None of the above change the bot/autoplay story readiness picture. The PENDING
integration branches (`integrate/autoplay-composite-window-resize-verdict-1875`
and `integrate/autoplay-placement-reject-recipe-1881`) remain unmerged.

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

### 2.3 Analyzer / Verification Reports on Main

The following report files are confirmed on `origin/main@7fc1706e`:

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

---

## 3. AC Item Status Against Current Main (`origin/main@7fc1706e`)

### 3.1 AC-VPT-01 — Minimum Window Size Gate

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-window-size-default-1893` (PROMPT 1893) |
| **Landed via** | PROMPT 1912 (`e02d132f`) |
| **Status** | **MERGED TO MAIN** — unchanged from PROMPT 1935 |
| **Verified on main** | `client/src/autoplay.rs`: `enforce_autoplay_window_size_system` at startup; `CCGS_WINDOW_WIDTH`/`CCGS_WINDOW_HEIGHT` env-var constants present |
| **Verified on main** | `tools/autoplay/Run-AutoplaySmoke.ps1`: env-var guards present |
| **Gap** | Startup-size floor enforced. Mid-run DWM resize prevention is AC-VPT-02/08 scope (also on main). |

### 3.2 AC-VPT-02 + AC-VPT-08 — Click-Target Viewport Guard

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-click-viewport-guard-1880` (PROMPT 1880) |
| **Landed via** | PROMPT 1880 source commit (`e8a40f81`) + PROMPT 1894 report (`71484fc4`) |
| **Status** | **MERGED TO MAIN** — unchanged from PROMPT 1935 |
| **Verified on main** | `tools/autoplay/driver.py`: `EXIT_VIEWPORT_GUARD = 5`, `_check_window_minimum`, `_check_window_drift`, `_validate_cursor_coords`, `_check_post_foreground_window` — all present |
| **Verified on main** | `tests/tools/autoplay/test_driver_click_viewport_guard.py` — present (66 tests) |
| **Checkpoint types on main** | `viewport_drift`, `viewport_shrink_abort`, `viewport_guard_cursor_none`, `viewport_guard_oob` |

### 3.3 Composite Window-Resize Verdict

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-composite-window-resize-verdict-1875` (PROMPT 1875) |
| **Base** | `origin/main@2ce3dc6b` |
| **Status** | **PENDING — not merged to main** — unchanged from PROMPT 1935 |
| **Verified absent** | `tools/autoplay/analyze_evidence_run.py`: no `win32_quality` / `window_resize_verdict` field on `origin/main@7fc1706e` |
| **Verified absent** | `tests/tools/autoplay/test_window_resize_verdict.py`: not present in `tests/tools/autoplay/` tree |
| **What it adds** | Window size tracking + win32 quality verdict in `analyze_evidence_run.py`; integrity guards in `validate_composite_run.py`; 25-test suite |
| **Rebase needed** | Yes — branch was based on `2ce3dc6b`; must rebase onto `7fc1706e` before FF merge |
| **Merge action** | Rebase onto `7fc1706e`, then `git merge --ff-only` |

### 3.4 Placement-Reject Recipe

| Field | Value |
|-------|-------|
| **Source branch** | `integrate/autoplay-placement-reject-recipe-1881` (PROMPT 1881) |
| **Base** | `origin/main@2ce3dc6b` |
| **Status** | **PENDING — not merged to main** — unchanged from PROMPT 1935 |
| **Verified absent** | `tools/autoplay/recipes/placement_reject_probe.py`: not in `tools/autoplay/recipes/` tree on `origin/main@7fc1706e` |
| **Verified absent** | `BOARD_DEEP_CELL` coord: not in `tools/autoplay/recipes/_coords.py` on main |
| **Verified absent** | Only `placement_drag_probe` in `recipes/__init__.py`; `placement_reject_probe` absent |
| **What it adds** | `tools/autoplay/recipes/placement_reject_probe.py` (121 lines); `placement_reject_probe` REGISTRY entry; `BOARD_DEEP_CELL` coord |
| **Rebase needed** | Yes — branch was based on `2ce3dc6b`; must rebase onto `7fc1706e` before FF merge |
| **Merge action** | Rebase onto `7fc1706e`, then `git merge --ff-only` |

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
| **Delta since PROMPT 1935** | No change — story remains DONE; tooling verify backfill reports (1838/1862/1899/1932) now on main via PROMPT 1950 |

---

### Story 002 — AUTOPLAY-VS-BOT-QA-001 (bot game clean run)

| Field | Value |
|-------|-------|
| **Status** | **BLOCKED — no fresh run with repaired driver yet** — unchanged from PROMPT 1935 |
| **Blocker** | No run achieves automated PASS. All three 2026-05-28 runs are PARTIAL. No fresh run executed with the repaired AC-VPT-01/02/08 driver. |
| **AC-VPT-01 repair** | **ON MAIN** since PROMPT 1912 |
| **AC-VPT-02/08 repair** | **ON MAIN** since PROMPT 1880/1894 |
| **Composite verdict tool** | PROMPT 1875 — pushed to origin; rebase needed for `7fc1706e`; **not on main** |
| **Delta since PROMPT 1935** | No change — no new run executed, composite verdict branch still pending |

**Path to DONE:**
1. Rebase + merge `integrate/autoplay-composite-window-resize-verdict-1875` onto `7fc1706e`
   (verdict tool, 25 tests — AC-VPT-06 coverage)
2. Execute fresh autoplay run; driver must exit 0, analyzer must return PASS verdict
   (zero FROZEN lines, ≥3 distinct hashes, window stable at `[1280,720]` throughout,
   `EXIT_VIEWPORT_GUARD` never triggered)

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|-------|-------|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** — unchanged from PROMPT 1935 |
| **Recipe branch** | `origin/integrate/autoplay-placement-reject-recipe-1881` |
| **main status** | `placement_reject_probe.py` absent from `origin/main@7fc1706e` |
| **Delta since PROMPT 1935** | No change |

**Path to DONE:** Rebase `integrate/autoplay-placement-reject-recipe-1881` onto
`7fc1706e`, FF merge, then run the recipe against a live game to produce pass evidence.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Full bot game live-pass signoff)

| Field | Value |
|-------|-------|
| **Status** | **BLOCKED — no automated PASS yet** — unchanged from PROMPT 1935 |
| **Blocking condition** | Analyzer returns PARTIAL for all three available runs (PROMPT 1846 §6) |
| **Human-review evidence** | Run `090613` — conditional only; bitblt PNGs show distinct content; requires human inspection to confirm UI not clipped and bot actions landed correctly |
| **Dependencies** | Story 002 must reach PASS first (fresh run with repaired driver + clean analyzer verdict) |
| **Delta since PROMPT 1935** | No change |

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
| `integrate/autoplay-composite-window-resize-verdict-1875` | 1875 | Window verdict in analyzer/validator | **NO** | Rebase onto `7fc1706e` needed |
| `integrate/autoplay-placement-reject-recipe-1881` | 1881 | `placement_reject_probe` recipe | **NO** | Rebase onto `7fc1706e` needed |
| `report/bot-autoplay-readiness-refresh-1935` | 1935 | Prior readiness report | **NO (branch)** | NOT_FF vs `7fc1706e`; this 1970 report is the replacement |

---

## 6. Validation Checklist

| Check | Result |
|-------|--------|
| Branch based on fresh `origin/main@7fc1706e`, not on stale 1935 branch | PASS — worktree created from `origin/main` |
| `git merge-base --is-ancestor origin/main HEAD` | PASS — verified before writing |
| No sentence claims PROMPT 1831 / run `090613` as clean automated PASS | PASS — §2.2 explicitly classifies as conditional human-review only |
| C0/human-review caveat preserved from PROMPT 1831/1840 | PASS — §2.2 |
| Report references PROMPT 1844 + 1846 as current evidence truth | PASS — §2 |
| PROMPT 1931 truth correction landing noted | PASS — §1, §2.3 |
| AC-VPT-01 on main — verified by code inspection | PASS — §3.1 |
| AC-VPT-02/08 on main — verified by code inspection | PASS — §3.2 |
| Composite verdict status against `7fc1706e` stated explicitly | PASS — §3.3 |
| Placement-reject recipe status against `7fc1706e` stated explicitly | PASS — §3.4 |
| Story 004 (AUTOPLAY-VS-BOT-QA-001) shown as BLOCKED | PASS — §4 Story 004 |
| Existing landed reports not deleted | PASS — report-only branch, two files added |
| Report-only (no sprint/story/tools/source files touched) | PASS |
| `git diff --name-status origin/main..HEAD` shows only owned report files | PASS — two files added, zero deletions |

---

## 7. Open Items and Merge Queue

| Item | Priority | Branch | Status |
|------|----------|--------|--------|
| Rebase + merge `integrate/autoplay-composite-window-resize-verdict-1875` onto `7fc1706e` | BLOCKING | origin — rebase needed | FF merge pending |
| Execute fresh autoplay run with repaired driver; expect PASS verdict | GATE | — | Blocked on composite verdict merge |
| Human review of bitblt/Bevy PNGs from fresh run | GATE | — | Blocked on clean run |
| Rebase + merge `integrate/autoplay-placement-reject-recipe-1881` (Story 003) onto `7fc1706e` | NORMAL | origin — rebase needed | FF merge pending |
| Verify AC-VPT-06 (distinct pixel_hash per phase, zero frozen) in fresh run | GATE | — | Blocked on merges + fresh run |

---

1970: BOT-AUTOPLAY-STORY-READINESS-REPORT-REFRESH-AFTER-1959: READY_FOR_MAINLAND_ENQUEUE
