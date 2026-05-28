# PROMPT 1870 — Bot/Autoplay Story Readiness Refresh After PROMPT 1858

**Date:** 2026-05-28  
**Worker:** PROMPT-1870  
**Source tree:** origin/main@5c91918d  
**Scope:** Reconciled story-readiness refresh — no source edits.  
**Supersedes:** `reports/PROMPT-1863-bot-autoplay-story-readiness-reconcile-after-1844-1846.md`  
**Superseded report base:** origin/main@bb90d7c2

---

## 1. Why This Report Exists

PROMPT 1863 shipped a useful story-readiness reconcile on
`origin/report/bot-autoplay-readiness-reconcile-1863` but that branch is
**not FF-ready** over current main — a direct merge would delete the PROMPT 1845
and PROMPT 1858 backfill artifacts that landed after 1863 was authored.

Since PROMPT 1863, the following landed on main:
- **PROMPT 1858** (`5c91918d`): backfilled `PROMPT-1845-post-1833-evidence-analyzer-focused-verify.md`
  and `PROMPT-1858-post-1833-evidence-analyzer-verify-report-backfill.md` to root `reports/`.

Additionally, two new integration branches are now ready but **not yet on main**:
- `origin/integrate/autoplay-window-size-default-1865` — AC-VPT-01 window size enforcement
- `origin/integrate/autoplay-click-viewport-guard-1857` — AC-VPT-02 + AC-VPT-08 viewport guard with blocking semantics

This report is authored on a fresh branch from `origin/main@5c91918d` and preserves
the 1863 structure/conclusions while updating integration branch status.

---

## 2. Current Evidence Truth (Unchanged from PROMPT 1844 + 1846)

### 2.1 The Three Runs on Record

| Run | Window size | Checkpoints | Analyzer verdict | Automated PASS? |
|---|---|---|---|---|
| `20260528-051148-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — no capture labels, no pixel_hash | NO |
| `20260528-063609-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — all 15 hashes identical (frozen renderer) | NO |
| `20260528-090613-Z` | `[1280,720]` → `[1280,1076]` mid-run | 15/15 | **PARTIAL** — 11/15 PrintWindow captures frozen; 11 distinct bitblt hashes | NO (conditional human-review only) |

No new runs have been executed since PROMPT 1846. This section is unchanged from PROMPT 1863.

### 2.2 Run `090613` Classification (UNCHANGED — CRITICAL)

Run `090613` is the **best available human-review evidence** but is **not a clean automated PASS**:

- **Mid-run DWM window resize** (ticks 115–127): `SW_RESTORE` snap-restore animated the window
  from `[1280,720]` to `[1280,1076]`.
- **720-baked click coordinates**: Recipe built at tick 1 for `[1280,720]`. Post-resize
  clicks at `placement-dragged` (tick 160) and `placement-submitted` (tick 172) used
  coordinates that map to wrong fractions in the 1076-height window:
  - `HAND_FIRST_CARD (0.35, 0.92)` → `y=662` → 61.5% of 1076 (target: 92%)
  - `SUBMIT_BTN (0.85, 0.92)` → `y=662` → 61.5% of 1076 (target: 92%)
- **PrintWindow all-frozen**: All 11 `win32_printwindow` captures triggered frozen
  detection; `desktop_bitblt` fallback produced 11 distinct hashes but the primary
  path was non-operational throughout.
- **Time-based checkpoints only**: All 15 checkpoints are tick-based. Passage does not
  confirm clicks landed on correct UI elements.

**Correct citation for `090613`:** "Conditional human-review evidence — bitblt PNGs show
distinct visual state changes; requires human inspector to verify UI was not clipped and
bot actions landed on visible elements."

**Prohibited citation:** Any sentence claiming `090613` or PROMPT 1831 as a clean
automated PASS, a clean smoke PASS, or as proof of correct bot UI interaction.

### 2.3 Blocker Summary for Automated PASS

None of the three available runs satisfies a clean automated PASS. Required before
AUTOPLAY-VS-BOT-QA-001 can close:

- **AC-VPT-01** (BLOCKING): Minimum window size gate — abort run if initial window < `[1280,720]`.
- **AC-VPT-02** (BLOCKING): Mid-run resize detection — abort with `EXIT_VIEWPORT_GUARD=5`
  if resize > ±10 px detected after recipe build.
- **AC-VPT-06** (BLOCKING): Minimum screenshot requirements — distinct pixel_hash per phase
  transition, zero stale-frame captures claiming PASS.

---

## 3. Integration Branch Status Update (New in 1870)

Three integration branches exist that address 1863 open items. None are on main yet.

### 3.1 `origin/integrate/autoplay-window-size-default-1865` — AC-VPT-01

| Field | Value |
|---|---|
| **PROMPT** | PROMPT 1865 |
| **Base** | `origin/main@bb90d7c2` → now at `5c91918d` (1 report commit ahead — still FF-ready) |
| **Status** | FF-ready over current main |
| **What it adds** | `enforce_autoplay_window_size_system` in `client/src/autoplay.rs`; window defaults in `Run-AutoplaySmoke.ps1` |
| **AC addressed** | AC-VPT-01 (initial window size floor at `[1280,720]`) |
| **Gap** | Rust `window.resolution.set()` enforces startup size but does NOT prevent mid-run DWM resize — that is AC-VPT-02 scope |
| **Merge blocker** | None reported; can be merged when operator is ready |

**Verification needed after merge:** confirm `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT`
env vars are propagated to the Bevy process in the launcher env before any cargo invocation.

### 3.2 `origin/integrate/autoplay-click-viewport-guard-1857` — AC-VPT-02 + AC-VPT-08

| Field | Value |
|---|---|
| **PROMPT** | PROMPT 1857 (two commits: `b07d50b9` initial, `f2afa1bb` addendum) |
| **Base** | `origin/main@bb90d7c2` |
| **FF-ready over 5c91918d?** | Yes — 1858/1845 commits were report-only; no Python/Rust file conflicts |
| **Status** | Integration-ready; **66/66 tests pass** (pytest, Python 3.12.10) |
| **What it adds** | `_parse_window_size`, `_check_window_minimum`, `_check_window_drift`, `_validate_cursor_coords` with `EXIT_VIEWPORT_GUARD=5` abort semantics; post-foreground re-poll for AC-VPT-08 |
| **ACs addressed** | AC-VPT-02 (mid-run drift abort), AC-VPT-08 (post-foreground DWM shrink abort) |
| **Checkpoint emission** | `viewport_drift`, `viewport_shrink_abort`, `viewport_guard_cursor_none`, `viewport_guard_oob` |
| **Merge blocker** | None reported; can merge independently after 1865 or before (no conflict) |

This branch is the primary repair for the `090613` resize-during-foreground failure mode.
After it lands, the driver will abort with `rc=5` before any click targets are corrupted
by a DWM resize event.

### 3.3 `origin/integrate/autoplay-placement-reject-recipe-1860` — Story 003

Status unchanged from PROMPT 1863. See §4 Story 003.

---

## 4. Story-by-Story Status

### Story 001 — Autoplay Tooling Baseline

| Field | Value |
|---|---|
| **Status** | DONE (main-landed) |
| **Evidence** | PROMPT 1818 (frozen PrintWindow bitblt fallback landed `d8b41463`); PROMPT 1833 (`analyze_evidence_run.py` landed `b856eef4`); PROMPT 1845 verify (21/21 tests pass, backfilled in `5c91918d`) |
| **Notes** | Core autoplay infrastructure stable on main. 1858 backfilled 1845 report — no functional change. |

No changes from PROMPT 1863 assessment; this story is stable.

---

### Story 002 — VS-Bot Recipe Smoke Pass (automated)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — pending repairs + fresh run** |
| **Blocker** | No run achieves PASS verdict. All three 2026-05-28 runs are PARTIAL (PROMPT 1846 §3–5). |
| **Repair status (new in 1870)** | AC-VPT-01: integration branch 1865 — FF-ready, not yet on main. AC-VPT-02/AC-VPT-08: integration branch 1857 — 66/66 tests pass, FF-ready, not yet on main. |
| **Remaining repair gap** | Neither 1865 nor 1857 has been merged to main. No fresh autoplay run has been executed with the repaired driver. |

**Correction from PROMPT 1834 (preserved from 1863):** `090613` smoke_exit=0 is not a pass
signal. Run `090613` is PARTIAL with post-resize click accuracy unverified. Do not cite as a PASS.

**Path to DONE:**
1. Merge `origin/integrate/autoplay-window-size-default-1865` (AC-VPT-01, FF-ready)
2. Merge `origin/integrate/autoplay-click-viewport-guard-1857` (AC-VPT-02/AC-VPT-08, FF-ready)
3. Execute fresh run; driver must exit 0, analyzer must return PASS verdict (zero FROZEN
   lines, ≥3 distinct hashes, window stable at `[1280,720]` throughout, `EXIT_VIEWPORT_GUARD`
   never triggered)

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|---|---|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** |
| **Recipe branch** | `origin/integrate/autoplay-placement-reject-recipe-1860` |
| **Recipe HEAD** | `9cf5c181` (PROMPT 1860) |
| **FF-ready** | Yes — single commit ahead of main@bb90d7c2; `5c91918d` is report-only, no conflict |
| **main status** | `placement_reject_probe.py` absent from `origin/main@5c91918d` |

Files in the recipe not yet on main:

| Path | Change |
|---|---|
| `tools/autoplay/recipes/__init__.py` | adds `placement_reject_probe` import + REGISTRY entry |
| `tools/autoplay/recipes/_coords.py` | adds `BOARD_DEEP_CELL` coord (0.5, 0.30) |
| `tools/autoplay/recipes/placement_reject_probe.py` | full recipe (121 lines) |

**Path to DONE:** Merge `origin/integrate/autoplay-placement-reject-recipe-1860` to main
(FF merge, no conflicts). Then run the recipe against a live game to produce pass evidence.

---

### Story 004 — AUTOPLAY-VS-BOT-QA-001 (Full bot game live-pass signoff)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — no automated PASS yet** |
| **Blocking condition** | Analyzer returns PARTIAL for all three available runs (PROMPT 1846 §6) |
| **Human-review evidence** | Run `090613` — conditional only; bitblt PNGs show distinct content; requires human inspection to confirm UI not clipped and bot actions landed correctly |
| **Dependencies** | Story 002 must reach PASS first (both AC-VPT repair branches on main + clean fresh run) |

**Path to DONE:**
1. Story 002 path completed (merges + fresh run with repaired driver)
2. Analyzer returns PASS on the fresh run
3. Human reviewer inspects bitblt/Bevy PNGs for that run and signs off
4. AUTOPLAY-VS-BOT-QA-001 can then be marked DONE

---

## 5. Validation Checklist

| Check | Result |
|---|---|
| No sentence claims PROMPT 1831 / run `090613` as clean automated PASS | PASS — §2.2 explicitly classifies as conditional human-review only |
| Report references PROMPT 1844 as current evidence truth | PASS — §2, §4 Story 002, §4 Story 004 |
| Report references PROMPT 1846 as current evidence truth | PASS — §2.1, §4 Story 002, §4 Story 004 |
| Story 004 (AUTOPLAY-VS-BOT-QA-001) shown as BLOCKED | PASS — §4 Story 004 |
| Story 003 placement-reject recipe status reflects 1860 lineage | PASS — §4 Story 003 |
| PROMPT 1845/1858 artifacts not deleted | PASS — this report adds one file only |
| 1865 integration branch status noted | PASS — §3.1 |
| 1857 integration branch status noted | PASS — §3.2 |
| Report-only (no sprint/story/tools/source files touched) | PASS |
| `git diff --check` | clean — see §6 |

---

## 6. Diff Validation

Branch: `report/bot-autoplay-readiness-refresh-1870`  
Base: `origin/main@5c91918d`  
Changes: `reports/PROMPT-1870-bot-autoplay-story-readiness-refresh-after-1858.md` (add)

No deletions of PROMPT 1845 or PROMPT 1858 artifacts. No changes to tools, tests,
source code, or sprint/session-state files.

---

## 7. Open Items and Repair Queue (Updated from 1863)

| Item | Priority | Branch | Status |
|---|---|---|---|
| Merge `origin/integrate/autoplay-window-size-default-1865` (AC-VPT-01) | BLOCKING | `integrate/autoplay-window-size-default-1865` | FF-ready, awaiting merge |
| Merge `origin/integrate/autoplay-click-viewport-guard-1857` (AC-VPT-02/AC-VPT-08) | BLOCKING | `integrate/autoplay-click-viewport-guard-1857` | FF-ready, 66/66 tests pass, awaiting merge |
| Merge `origin/integrate/autoplay-placement-reject-recipe-1860` (Story 003) | NORMAL | `integrate/autoplay-placement-reject-recipe-1860` | FF-ready, awaiting merge |
| Execute fresh autoplay run post-repairs; expect PASS verdict from analyzer | GATE | — | Blocked on above merges |
| Human review of `bitblt_tick_*.png` and Bevy RPC screenshots from fresh run | GATE | — | Blocked on clean run |
| Verify AC-VPT-06 (distinct pixel_hash per phase, zero frozen) in fresh run | GATE | — | Blocked on 1857 merge + fresh run |

---

1870: BOT-AUTOPLAY-STORY-READINESS-REFRESH-AFTER-1858: SHIPPED
