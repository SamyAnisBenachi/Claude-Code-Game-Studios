# PROMPT 1907 — Bot/Autoplay Story Readiness Report Refresh After PROMPT 1876

**Date:** 2026-05-28
**Branch:** `report/bot-autoplay-readiness-refresh-1907`
**Source tree:** `origin/main@c35750d8` (PROMPT 1856 — latest main)
**Scope:** Report-only — no source edits, no sprint-state writes.
**Supersedes:** `reports/PROMPT-1891-bot-autoplay-story-readiness-refresh-after-1872.md`
**Prior base:** PROMPT 1891 was authored on `origin/main@2ce3dc6b`

---

## 0. Backfill Note

This prompt also backfills `reports/PROMPT-1891-bot-autoplay-story-readiness-refresh-after-1872.md`
from `origin/report/bot-autoplay-readiness-refresh-1891` (`7996c8a2`) onto current
main. The 1891 branch was NOT_FF against main@`c35750d8`; its source content is
preserved here without modification.

Source branch for 1891 backfill: `origin/report/bot-autoplay-readiness-refresh-1891`
Source commit: `7996c8a2`

---

## 1. Why This Report Exists

PROMPT 1891 shipped a useful readiness refresh on
`origin/report/bot-autoplay-readiness-refresh-1891` but that branch was
**not FF-ready** over current main (`c35750d8`) — merging it directly would
delete the PROMPT 1856/1876 artifacts that landed between `2ce3dc6b` and
`c35750d8`.

Since PROMPT 1891 (base `2ce3dc6b`), the following landed on `origin/main`:
- **PROMPT 1876** (`674ba870`): Re-applied PROMPT 1837/1874 evidence UX block
  onto post-1872 main — added `tools/dev-launcher/Start-AutoplayVsBot.ps1`
  (dev-launcher autoplay VS-bot launch script) and a companion report.
- **PROMPT 1856** (`c35750d8`): UI layout smoke slice-F report — added
  `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md`.

Neither PROMPT 1876 nor PROMPT 1856 changes autoplay driver code, AC item
status, or integration branch status. The evidence truth, story status, and
merge queue from PROMPT 1891 are authoritative and reproduced verbatim below.

---

## 2. Evidence Truth (Unchanged from PROMPT 1844 + 1846 + 1872)

### 2.1 Available Runs

No new autoplay runs have been executed since the three runs captured on 2026-05-28.
This section is identical to PROMPT 1891 §2.1.

| Run | Window size | Checkpoints | Analyzer verdict | Automated PASS? |
|---|---|---|---|---|
| `20260528-051148-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — no capture labels, no pixel_hash | NO |
| `20260528-063609-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — all 15 hashes identical (frozen renderer) | NO |
| `20260528-090613-Z` | `[1280,720]` → `[1280,1076]` mid-run | 15/15 | **PARTIAL** — 11/15 PrintWindow captures frozen; 11 distinct bitblt hashes | NO (conditional human-review only) |

### 2.2 Run `090613` Classification (UNCHANGED — CRITICAL)

Run `090613` is the **best available human-review evidence** but is **not a clean
automated PASS**. The PROMPT 1891 §2.2 classification stands verbatim:

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

## 3. AC Item Status Against Current Main (`origin/main@c35750d8`)

### 3.1 AC-VPT-01 — Minimum Window Size Gate

| Field | Value |
|---|---|
| **Refresh branch** | `integrate/autoplay-window-size-default-1879` (PROMPT 1879) |
| **Base** | `origin/main@2ce3dc6b` |
| **FF-ready over current main** | Rebase onto `c35750d8` required before FF merge |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | `enforce_autoplay_window_size_system` startup system in `client/src/autoplay.rs`; env-var `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT` guards in `Run-AutoplaySmoke.ps1` |
| **Gap** | Startup-size floor enforced; mid-run DWM resize NOT prevented (that is AC-VPT-02 scope) |
| **Merge action** | Rebase onto `c35750d8`, then `git merge --ff-only` |

### 3.2 AC-VPT-02 + AC-VPT-08 — Click-Target Viewport Guard

| Field | Value |
|---|---|
| **Refresh branch** | `integrate/autoplay-click-viewport-guard-1880` (PROMPT 1880) |
| **Base** | `origin/main@2ce3dc6b` |
| **FF-ready over current main** | Rebase onto `c35750d8` required before FF merge |
| **Test suite** | 66/66 unit tests pass (focused run; no GUI, no Cargo) |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | `EXIT_VIEWPORT_GUARD=5` exit code; viewport drift/OOB guards in `tools/autoplay/driver.py`; structured checkpoint types |
| **Merge action** | Rebase onto `c35750d8`, then `git merge --ff-only` |

### 3.3 Composite Window-Resize Verdict

| Field | Value |
|---|---|
| **Refresh branch** | `integrate/autoplay-composite-window-resize-verdict-1875` (PROMPT 1875) |
| **Base** | `origin/main@2ce3dc6b` |
| **FF-ready over current main** | Rebase onto `c35750d8` required before FF merge |
| **Test suite** | 25/25 tests pass |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | Window size tracking + win32 quality verdict in `tools/autoplay/analyze_evidence_run.py`; integrity guards in `validate_composite_run.py`; 25-test suite |
| **Merge action** | Rebase onto `c35750d8`, then `git merge --ff-only` |

### 3.4 Placement-Reject Recipe

| Field | Value |
|---|---|
| **Refresh branch** | `integrate/autoplay-placement-reject-recipe-1881` (PROMPT 1881) |
| **Base** | `origin/main@2ce3dc6b` |
| **FF-ready over current main** | Rebase onto `c35750d8` required before FF merge |
| **Registry** | 13 recipes; `placement-reject-probe` present |
| **Status** | **PENDING — not merged to main** |
| **What it adds** | `tools/autoplay/recipes/placement_reject_probe.py` (121 lines); REGISTRY entry; `BOARD_DEEP_CELL` coord |
| **Merge action** | Rebase onto `c35750d8`, then `git merge --ff-only` |

---

## 4. Story Status Table

### Story 001 — Autoplay Driver Foundation

| Field | Value |
|---|---|
| **Status** | **DONE (main)** — core driver, recipe framework, composite harness on main since early sprints |
| **Evidence analyzer** | `tools/autoplay/analyze_evidence_run.py` on main since PROMPT 1833 (`b856eef4`) |
| **Window-resize verdict extension** | In `integrate/autoplay-composite-window-resize-verdict-1875` — rebase needed, not yet main |

---

### Story 002 — AUTOPLAY-VS-BOT-QA-001 (bot game clean run)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — pending repairs + fresh run** |
| **Blocker** | No run achieves automated PASS. All three 2026-05-28 runs are PARTIAL. |
| **AC-VPT-01 repair** | PROMPT 1879 — pushed to origin; rebase needed for `c35750d8`; **not on main** |
| **AC-VPT-02/08 repair** | PROMPT 1880 — pushed to origin; rebase needed; 66/66 tests pass; **not on main** |
| **Composite verdict tool** | PROMPT 1875 — pushed to origin; rebase needed; 25/25 tests; **not on main** |
| **Remaining repair gap** | None of the AC-VPT branches has been merged to main. No fresh autoplay run with repaired driver. |

**Path to DONE:**
1. Rebase + merge `integrate/autoplay-window-size-default-1879` onto `c35750d8` (AC-VPT-01)
2. Rebase + merge `integrate/autoplay-click-viewport-guard-1880` onto `c35750d8` (AC-VPT-02/08, 66 tests)
3. Rebase + merge `integrate/autoplay-composite-window-resize-verdict-1875` onto `c35750d8` (verdict tool, 25 tests)
4. Execute fresh run; driver must exit 0, analyzer must return PASS verdict

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|---|---|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** |
| **Recipe branch** | `origin/integrate/autoplay-placement-reject-recipe-1881` |
| **main status** | `placement_reject_probe.py` absent from `origin/main@c35750d8` |
| **Registry** | 13 recipes including `placement-reject-probe` on the branch |

**Path to DONE:** Rebase `integrate/autoplay-placement-reject-recipe-1881` onto `c35750d8`, merge (FF).

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Full bot game live-pass signoff)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — no automated PASS yet** |
| **Blocking condition** | Analyzer returns PARTIAL for all three available runs |
| **Human-review evidence** | Run `090613` — conditional only; requires human inspection |
| **Dependencies** | Story 002 must reach PASS first |

**Path to DONE:**
1. Story 002 path completed (merges + fresh run with repaired driver)
2. Analyzer returns PASS on the fresh run
3. Human reviewer inspects bitblt/Bevy PNGs for that run and signs off
4. AUTOPLAY-VS-BOT-QA-001 can then be marked DONE

---

## 5. Integration Branch Summary Table

| Branch | PROMPT | What | Pushed | FF over `c35750d8` | On main |
|---|---|---|---|---|---|
| `integrate/autoplay-window-size-default-1879` | 1879 | AC-VPT-01 startup size floor | YES | Rebase needed | NO |
| `integrate/autoplay-click-viewport-guard-1880` | 1880 | AC-VPT-02/08 drift + OOB guards | YES | Rebase needed | NO |
| `integrate/autoplay-composite-window-resize-verdict-1875` | 1875 | Window verdict in analyzer/validator | YES | Rebase needed | NO |
| `integrate/autoplay-placement-reject-recipe-1881` | 1881 | `placement_reject_probe` recipe | YES | Rebase needed | NO |
| `report/bot-autoplay-readiness-refresh-1891` | 1891 | Prior readiness report | YES | NO (not FF over `c35750d8`) | NO |

**Note on 1891 branch:** Not FF-ready over current main (`c35750d8`) — merging directly
would delete PROMPT 1876/1856 artifacts. This report (1907) supersedes 1891.

---

## 6. Validation

| Check | Result |
|---|---|
| Source branch for 1891 backfill identified | PASS — `origin/report/bot-autoplay-readiness-refresh-1891` @ `7996c8a2` |
| Current main preserved as ancestor | PASS — branch based on `origin/main@c35750d8` |
| No tools/source/sprint files touched | PASS — report-only |
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` not reverted | PASS — report-only branch |
| No existing landed reports deleted | PASS — two files added only |
| `git diff --name-status origin/main..HEAD` shows only two owned report files | PASS (see §7) |
| No sentence claims run `090613` as clean automated PASS | PASS — §2.2 unchanged |
| Integration branches noted as needing rebase before FF merge to `c35750d8` | PASS — §3.x |

---

## 7. Diff Validation

Expected `git diff --name-status origin/main..HEAD`:
```
A	reports/PROMPT-1891-bot-autoplay-story-readiness-refresh-after-1872.md
A	reports/PROMPT-1907-bot-autoplay-story-readiness-report-refresh-after-1876.md
```

No deletes. No modifications to existing files.

---

## 8. Commit Details

| Field | Value |
|---|---|
| **Branch** | `report/bot-autoplay-readiness-refresh-1907` |
| **Based on** | `origin/main@c35750d8` |
| **Source 1891 report from** | `origin/report/bot-autoplay-readiness-refresh-1891` @ `7996c8a2` |
| **Files added** | 2 (both in `reports/`) |
| **Files modified/deleted** | 0 |
| **FF ancestor check** | `git merge-base --is-ancestor origin/main HEAD` → exits 0 |

---

1907: BOT-AUTOPLAY-STORY-READINESS-REPORT-REFRESH-AFTER-1876: SHIPPED
