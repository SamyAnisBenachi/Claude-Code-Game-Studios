# PROMPT 1863 — Bot/Autoplay Story Readiness: Reconciled Report After PROMPT 1844 and 1846

**Date:** 2026-05-28  
**Worker:** PROMPT-1863  
**Source tree:** origin/main@bb90d7c2  
**Scope:** Reconciled story-readiness report — no source edits.  
**Supersedes:** PROMPT-1834 report (partially stale — see §1 below)

---

## 1. Why This Report Exists / PROMPT 1834 Staleness

PROMPT 1834 (`reports/PROMPT-1834-bot-autoplay-story-readiness-after-1830.md`, branch
`origin/wt/1834-bot-autoplay-readiness @ 30a67570`) was reported DONE but has three
problems that prevent it from landing or from being used as-is:

| Problem | Detail |
|---|---|
| Not FF-ready | Branch diverges from current main; a direct merge would delete PROMPT 1833/1844 files |
| Evidence claim stale | PROMPT 1834 treats PROMPT 1831 / run `20260528-090613-Z` as PASS-style live evidence |
| Superseded by 1844/1846 | Those two audits re-classified ALL three runs as PARTIAL / INSUFFICIENT for automated PASS |

This report preserves the useful story-by-story structure from PROMPT 1834 and applies the
corrections established by PROMPT 1844 and PROMPT 1846 as the current evidence truth.

---

## 2. Current Evidence Truth (PROMPT 1844 + 1846 Findings)

### 2.1 The Three Runs on Record

| Run | Window size | Checkpoints | Analyzer verdict | Automated PASS? |
|---|---|---|---|---|
| `20260528-051148-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — no capture labels, no pixel_hash | NO |
| `20260528-063609-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — all 15 hashes identical (frozen renderer) | NO |
| `20260528-090613-Z` | `[1280,720]` → `[1280,1076]` mid-run | 15/15 | **PARTIAL** — 11/15 PrintWindow captures frozen; 11 distinct bitblt hashes | NO (conditional human-review only) |

### 2.2 Run `090613` Classification

Run `090613` is the **best available human-review evidence** but is **not a clean automated PASS**
for the following reasons established by PROMPT 1844:

- **Mid-run DWM window resize** (tick 115–127): Win32 `ShowWindow(SW_RESTORE)` triggered a
  snap-restore animation, resizing from `[1280,720]` to `[1280,1076]`.
- **720-baked click coordinates**: Recipe was built once at tick 1 for `[1280,720]`. Post-resize
  clicks at `placement-dragged` (tick 160) and `placement-submitted` (tick 172) used coordinates
  that map to wrong fractions in the 1076-height window:
  - `HAND_FIRST_CARD (0.35, 0.92)` → `y=662` → 61.5% of 1076 (should be 92%)
  - `SUBMIT_BTN (0.85, 0.92)` → `y=662` → 61.5% of 1076 (should be 92%)
- **PrintWindow all-frozen**: All 11 `win32_printwindow` captures triggered frozen detection;
  `desktop_bitblt` fallback was used and produced 11 distinct hashes. This is functional but
  means the primary capture path was non-operational for the entire run.
- **Time-based checkpoints only**: All 15 checkpoints are tick-based, not state-verified. Passage
  of all 15 checkpoints does **not** confirm that clicks landed on correct UI elements.

**Correct citation for `090613`:** "Conditional human-review evidence — bitblt PNGs show distinct
visual state changes; requires human inspector to verify UI was not clipped and bot actions landed
on visible elements."

**Incorrect citation (PROHIBITED):** Any sentence claiming `090613` or PROMPT 1831 as a clean
automated PASS, a clean smoke PASS, or as proof of correct bot UI interaction.

### 2.3 Runs `051148` and `063609` Classification

`051148`: Window stable throughout; Bevy RPC screenshots (15) present; no capture labels / no
pixel_hash data → **INSUFFICIENT** for automated PASS. Human visual review of 15 PNGs could
potentially qualify as evidence but no such review is on record.

`063609`: Window stable but all 15 win32_capture hashes identical (`0x26207c4c`) — frozen renderer
throughout. This run cannot be used to claim visible GUI transitions. **INSUFFICIENT**.

### 2.4 Blocker Summary for Automated PASS

None of the three available runs satisfies a clean automated PASS. A fresh run after the following
repairs is required before AUTOPLAY-VS-BOT-QA-001 can be closed:

- **AC-VPT-01** (BLOCKING): Minimum window size gate — abort run if initial window < `[1280,720]`.
- **AC-VPT-02** (BLOCKING): Mid-run resize detection — abort (or flag `NEEDS_HUMAN_GUI`) if resize
  detected after recipe build.
- **AC-VPT-06** (BLOCKING): Minimum screenshot requirements — distinct pixel_hash per phase
  transition, zero stale-frame captures claiming PASS.

(Full AC list in PROMPT 1844 §8.)

---

## 3. Story-by-Story Status

### Story 001 — Autoplay Tooling Baseline

| Field | Value |
|---|---|
| **Status** | DONE (main-landed) |
| **Evidence** | PROMPT 1818 (frozen PrintWindow bitblt fallback), PROMPT 1833 (evidence analyzer), PROMPT 1838 (post-1830 tooling verify) |
| **Notes** | Core autoplay infrastructure on main. `analyze_evidence_run.py` landed @ b856eef4. |

No changes from PROMPT 1834 assessment; this story is stable.

---

### Story 002 — VS-Bot Recipe Smoke Pass (automated)

| Field | Value |
|---|---|
| **Status** | **BLOCKED — pending repairs + fresh run** |
| **Blocker** | No run achieves PASS verdict. All three 2026-05-28 runs are PARTIAL (PROMPT 1846 §3–5). |
| **Repair blockers** | AC-VPT-01, AC-VPT-02, AC-VPT-06 (see §2.4 above). |
| **PROMPT 1842 status** | Branch `origin/work/1842-window-size-repair` adds launcher env vars but Rust-side DWM resize guard not verified on main. |
| **PROMPT 1843 status** | No worktree found; viewport guard not landed on main (PROMPT 1844 §6). |

**Correction from PROMPT 1834:** PROMPT 1834 cited `090613` smoke_exit=0 as a pass signal.
This is incorrect. `smoke_exit=0` reflects driver exit code, not state verification. Run
`090613` is PARTIAL with post-resize click accuracy unverified. Do not cite it as a PASS.

**Path to DONE:**
1. Land AC-VPT-01 (initial window size gate in driver)
2. Land AC-VPT-02 (mid-run resize detection in driver)
3. Ensure win32_printwindow / bitblt returns distinct hashes for each phase
4. Execute fresh run; analyzer must return PASS verdict (zero FROZEN lines, ≥3 distinct hashes,
   window stable at `[1280,720]` throughout)

---

### Story 003 — Placement-Reject Recovery Recipe

| Field | Value |
|---|---|
| **Status** | **IMPLEMENTED (integration branch), NOT yet main-landed** |
| **Recipe branch** | `origin/integrate/autoplay-placement-reject-recipe-1860` |
| **Recipe HEAD** | `9cf5c181` (PROMPT 1860) |
| **FF-ready** | Yes — single commit ahead of main@bb90d7c2, no conflicts (PROMPT 1849 validation) |
| **main status** | `placement_reject_probe.py` is **absent** from `origin/main@bb90d7c2` |

**PROMPT 1832/1849/1860 lineage:**

- **PROMPT 1832** — original implementation on `origin/wt-1832-placement-reject-recipe @ 3bbfbec1`.
  Based on pre-1833 main; not FF-compatible due to missing PROMPT 1833 files.
- **PROMPT 1849** — integration refresh: cherry-pick of 3 recipe files onto current main (b856eef4).
  Validation: PROMPT 1833 files preserved, no conflicts, MAINLAND_ENQUEUE recommended.
- **PROMPT 1860** — mainland refresh after PROMPT 1844 landed (bb90d7c2). Branch at `9cf5c181`,
  FF-ready. Report backfill confirmed via summary file.

**Files in the recipe (not yet on main):**

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
| **Dependencies** | Story 002 must reach PASS first (window size gate + resize detection + clean run) |

**Correct assessment (supersedes PROMPT 1834):**

PROMPT 1844 established that:
- All post-resize clicks in run `090613` (ticks 128–260) used 720-baked coordinates against a
  1076-height window. Placement and submit clicks may have missed their targets.
- All three available runs show insufficient evidence for automated PASS.

PROMPT 1846 confirmed:
- Analyzer verdict for all three runs: PARTIAL.
- No run satisfies the PASS condition (distinct pixel_hashes ≥ 3, zero FROZEN lines, stable
  window at full resolution from tick 1).
- The composite summary correctly states `NOT-CLAIMED` for live PASS status on all three runs.

**Path to DONE:**
1. Story 002 repairs + clean run (window stable, distinct hashes, zero frozen)
2. Analyzer returns PASS on the fresh run
3. Human reviewer inspects bitblt/Bevy PNGs for that run and signs off
4. AUTOPLAY-VS-BOT-QA-001 can then be marked DONE

---

## 4. Validation Checklist

| Check | Result |
|---|---|
| No sentence claims PROMPT 1831 / run `090613` as clean automated PASS | PASS — §2.2 explicitly classifies it as conditional human-review only |
| Report references PROMPT 1844 as current evidence truth | PASS — §2.1, §2.2, §3 Story 002, §3 Story 004 all cite PROMPT 1844 |
| Report references PROMPT 1846 as current evidence truth | PASS — §2.1, §2.3, §3 Story 002, §3 Story 004 all cite PROMPT 1846 |
| Story 004 (AUTOPLAY-VS-BOT-QA-001) shown as BLOCKED | PASS — §3 Story 004 |
| Story 003 placement-reject recipe status reflects 1832/1849/1860 lineage | PASS — §3 Story 003 |
| Story 003 notes recipe not yet on main | PASS — integration branch `origin/integrate/autoplay-placement-reject-recipe-1860`, absent from main@bb90d7c2 |
| Report-only (no sprint/story/status files touched) | PASS — single file in reports/ |
| Diff is reports-only | PASS — see worktree state |

---

## 5. Open Items and Repair Queue

| Item | Priority | Owner |
|---|---|---|
| Land AC-VPT-01: initial window size abort gate in `driver.py` | BLOCKING | tools-programmer |
| Land AC-VPT-02: mid-run resize detection + abort/flag in `driver.py` | BLOCKING | tools-programmer |
| Verify PROMPT 1842 Rust-side DWM resize guard on main | MAJOR | gameplay-programmer |
| Land PROMPT 1843 viewport guard (currently absent from main) | MAJOR | tools-programmer |
| Merge `origin/integrate/autoplay-placement-reject-recipe-1860` to main (FF-ready) | NORMAL | producer |
| Execute fresh autoplay run post-repairs; expect PASS verdict from analyzer | GATE | qa-tester |
| Human review of `bitblt_tick_*.png` and Bevy RPC screenshots from fresh run | GATE | qa-lead |

---

1863: BOT-AUTOPLAY-STORY-READINESS-RECONCILE-AFTER-1844-1846: SHIPPED
