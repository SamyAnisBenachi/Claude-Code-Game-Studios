# PROMPT 2018 — Bot/Autoplay Current-State Closure Audit After PROMPT 2016

**Date:** 2026-05-28
**Branch:** `work/PROMPT-2018-bot-autoplay-closure-audit`
**Source-of-truth:** `origin/main @ 3966d1c1` (PROMPT 2016)
**Scope:** Read-only audit — no source edits, no sprint/production writes.

---

## Why This Report Exists

The user asked: why has bot/autoplay taken many agents, what state is it in,
what blockers remain, and whether orchestrator state needs updating. This is a
factual closure audit against `origin/main@3966d1c1` answering those questions
directly, with no report churn recommendation.

---

## 1. Bot State

### 1.1 Server-side Bot AI

The bot AI (Rust/Bevy server logic) is fully implemented and merged to main in
earlier sprints. The server runs a game loop against a bot opponent; all bot
decisions are server-authoritative. No open implementation items on the bot AI
itself.

**Status: DONE — on main.**

### 1.2 Bot Soak (Server-only, No GUI)

A headless server-vs-server bot soak was implemented in earlier work (PROMPT
1671 range). No open items.

**Status: DONE — on main. Not dependent on autoplay tooling.**

---

## 2. Autoplay State

### 2.1 Driver Foundation

`tools/autoplay/driver.py` is the core Python RPC driver. It connects to the
Bevy autoplay RPC server, replays recipe action sequences, captures screenshots,
and logs structured checkpoints.

**On main (`3966d1c1`):**

| Component | File | Status |
|---|---|---|
| Core driver | `tools/autoplay/driver.py` | DONE — on main |
| Recipe registry | `tools/autoplay/recipes/__init__.py` | DONE — on main |
| Evidence analyzer | `tools/autoplay/analyze_evidence_run.py` | DONE — on main (includes `win32_quality` composite verdict) |
| Composite validator | `tools/autoplay/validate_composite_run.py` | DONE — on main (hard-FAIL on resize/frozen) |
| Win32 capture | `tools/autoplay/win_capture.py` | DONE — on main (PrintWindow + desktop BitBlt fallback) |
| Screenshot poll | `tools/autoplay/screenshot_poll.py` | DONE — on main |
| Operator runbook | `tools/autoplay/Run-AutoplaySmoke.ps1` | DONE — on main |

### 2.2 Viewport / Window Guards

Three guard layers are on main:

| Guard | Activation | Where | Status |
|---|---|---|---|
| **AC-VPT-01** — Startup window-size floor | `enforce_autoplay_window_size_system` Bevy startup system reads `CCGS_WINDOW_WIDTH`/`CCGS_WINDOW_HEIGHT` env vars | `client/src/autoplay.rs` | DONE — on main (PROMPT 1912) |
| **AC-VPT-02 / AC-VPT-08** — Mid-run drift + OOB abort | `EXIT_VIEWPORT_GUARD=5`; inlined `_check_window_minimum`, `_check_window_drift`, `_validate_cursor_coords` per tick | `tools/autoplay/driver.py` | DONE — on main (PROMPT 1880/1894) |
| **Viewport shrink guard module** | `check_viewport_size`, `check_click_target`, `check_before_input` — standalone utility | `tools/autoplay/viewport_shrink_guard.py` | DONE as standalone — **NOT imported by driver.py**. Driver has its own inlined equivalent. Module is available for recipe/future use. (PROMPT 2009) |

**Integration note:** `viewport_shrink_guard.py` provides the same semantics as
the inlined driver guards but is a clean standalone module. It is tested in
isolation (31 tests — all pass). It does NOT replace driver.py's inlined logic
in the current codebase; driver.py imports only `recipes`, `screenshot_poll`,
`win_capture`, and `win_foreground`. This is an architectural loose end but
does NOT block a clean autoplay run — the driver inlined guards are active.

### 2.3 Window-Resize Verdict

`analyze_evidence_run.py` and `validate_composite_run.py` detect and hard-FAIL
on mid-run DWM window-resize events (`WINDOW-RESIZE-DETECTED`), frozen
PrintWindow captures (`WIN32-ALL-FROZEN`), and window height below minimum
(`WINDOW-HEIGHT-TOO-SMALL`). Verdict logic and test suite are on main via
PROMPT 1994 (reapply of PROMPT 1979).

**On main (`3966d1c1`):**

| File | Status |
|---|---|
| `tools/autoplay/analyze_evidence_run.py` | Window-resize verdict — DONE |
| `tools/autoplay/validate_composite_run.py` | Hard-FAIL conditions — DONE |
| `tests/tools/autoplay/test_window_resize_verdict.py` | On main (PROMPT 1994) |

**Status: DONE — on main.**

### 2.4 Recipes

All registered recipes in `tools/autoplay/recipes/REGISTRY`:

| Recipe | Key | Status |
|---|---|---|
| `idle` | idle | On main |
| `smoke` | smoke | On main |
| `lobby-create` | lobby-create | On main |
| `add-bot-lobby` | add-bot-lobby | On main |
| `class-select` | class-select | On main |
| `draft-auction-probe` | draft-auction-probe | On main |
| `round-loop` | round-loop | On main |
| `placement-drag-probe` | placement-drag-probe | On main |
| `resolution-observe` | resolution-observe | On main |
| `game-over-observe` | game-over-observe | On main |
| `full-game` | full-game | On main |
| `vs-bot` | vs-bot | On main |
| `placement-reject-probe` | placement-reject-probe | **NEW — on main (PROMPT 2013)** |

**`placement_reject_probe` observable limitations** (documented, not bugs):
- No `autoplay/status` phase signal to confirm `S2CPlacementRejected` received;
  rejection can only be confirmed via screenshot review.
- `BOARD_DEEP_CELL` target is heuristic (fy=0.30) — may accidentally be valid
  for some class/round combos.
- Override: `CCGS_AUTOPLAY_BOARD_DEEP_CELL=fx,fy` env var available.

**Status: All recipes DONE — on main.**

### 2.5 Open Recipe Fragility Items (From PROMPT 1848 Coverage Map)

These are identified fragility items that have NOT been implemented:

| ID | Description | Files | Priority |
|---|---|---|---|
| FRAG-02 / R-01 | `HAND_FIRST_CARD` fy=0.92 and `SUBMIT_BTN` fy=0.92 are at 92% of window height — high risk of click miss at 720px or at post-resize tall windows | `tools/autoplay/recipes/_coords.py` | MEDIUM |
| FRAG-03 / R-02 | No `poll_phase(label, max_ticks)` pseudo-action — recipes use fixed-tick settling gaps; if any phase takes longer the next recipe clicks into the wrong overlay | `tools/autoplay/recipes/_builder.py` + `driver.py` | MEDIUM |

Neither fragility causes a hard build failure. Both cause click misses in
specific timing/layout conditions. Neither was in scope for any currently landed
PROMPT. They remain identified but unimplemented.

---

## 3. Evidence State

### 3.1 Known Runs (All From 2026-05-28)

| Run ID | Window | Checkpoints | Verdict | Usable as? |
|---|---|---|---|---|
| `20260528-051148-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — no `pixel_hash`, no capture labels | No |
| `20260528-063609-Z` | `[1280,720]` stable | 15/15 | **PARTIAL** — all 15 hashes frozen/identical | No |
| `20260528-090613-Z` | `[1280,720]` → `[1280,1076]` mid-run | 15/15 | **PARTIAL** — DWM resize + 11 frozen PrintWindow lines | Conditional human-review only |

**There is no clean automated PASS run.** All three known runs were executed
before AC-VPT-01 (startup floor) was on main (PROMPT 1912). The full guard
stack (AC-VPT-01 + AC-VPT-02/08) has never been exercised against a live run.

### 3.2 Run `090613` Classification (Standing — Unchanged)

Run `090613` is the best available human-review evidence but is **not** a clean
automated PASS:

- Mid-run DWM window resize: `[1280,720]` → `[1280,1076]` (ticks 115–127).
- Click coordinates baked at 720px height; post-resize placement/submit clicks
  landed at ~61.5% of the 1076px window (expected ~92%).
- PrintWindow all-frozen (11/11 lines); `desktop_bitblt` fallback produced 11
  distinct pixel hashes.

**Correct citation for run `090613`:** "Conditional human-review evidence —
bitblt PNGs show distinct visual state changes; requires human inspection to
confirm UI was not clipped and bot actions landed on visible elements."

**`090613` must not be cited as:** "clean automated PASS", "smoke PASS", or
"proof of correct bot UI interaction."

---

## 4. Test Suite State (All on main@3966d1c1)

| Test file | Count | Last run | Result |
|---|---|---|---|
| `test_driver_click_viewport_guard.py` | 66 | PROMPT 1948 | 66/66 PASS — no code changes since |
| `test_viewport_shrink_guard.py` | 31 | PROMPT 2009 | 31/31 PASS |
| `test_recipe_static.py` | 83 | PROMPT 2013 | 83/83 PASS |
| `test_window_resize_verdict.py` | present | PROMPT 1994 | PASS (count not restated in 1994 report) |
| `test_analyze_evidence_run.py` | present | PROMPT 1994 | PASS |
| `test_validate_composite_run.py` | present | PROMPT 1994 | PASS |
| `test_win32_capture.py` | present | Earlier PROMPT | PASS |
| `test_win_foreground.py` | present | Earlier PROMPT | PASS |
| `test_driver_screenshot_barrier.py` | present | Earlier PROMPT | PASS |

All static tests pass. No test depends on a live GUI run. No Cargo build
required for any autoplay test suite.

---

## 5. Remaining Blockers

### 5.1 AUTOPLAY-VS-BOT-QA-001 (The Core Live-Run Blocker)

**Status: BLOCKED — operator environment gate.**

No fresh autoplay run has been executed with the full guard stack on main
(AC-VPT-01 + AC-VPT-02/08 + composite verdict). Until a run exits with:
- `driver.py` exit 0
- `analyze_evidence_run.py` verdict: PASS (≥3 distinct pixel hashes, 0 FROZEN
  lines, window stable at `[1280,720]`, `EXIT_VIEWPORT_GUARD` never triggered)
- Human review of bitblt/Bevy PNGs confirming actions landed on visible elements

...the story cannot be marked DONE.

**Blocker type: Operator environment gate.** Not a code blocker. The code is
complete. The operator must:
1. Start the Bevy client with `CCGS_WINDOW_WIDTH=1280 CCGS_WINDOW_HEIGHT=720`
   (or via `Run-AutoplaySmoke.ps1`)
2. Ensure DWM does not resize the window mid-run (no snapping, no display scale
   change, no other window management intervention)
3. Run `python driver.py --recipe vs-bot` against the live session
4. Review the analyzer output and bitblt PNGs

### 5.2 viewport_shrink_guard.py Integration Gap

`viewport_shrink_guard.py` is a clean extracted module that is not yet imported
by `driver.py`. The driver has its own inlined equivalent. This is an
architectural loose end, not a blocker for QA-001.

**Blocker type: Technical debt (optional). Does not block any QA gate.**

### 5.3 Recipe Fragility Items (FRAG-02, FRAG-03)

FRAG-02 (`HAND_FIRST_CARD` / `SUBMIT_BTN` fy=0.92) and FRAG-03 (no
`poll_phase` phase-gating) are known risks but have not caused run failures
in the available evidence. Fixing FRAG-02 would require a `_coords.py` edit;
fixing FRAG-03 would require driver + recipe changes.

**Blocker type: Product/tooling — medium priority, not blocking QA-001 today.**

---

## 6. Human QA Gates

| Gate | Condition | Status |
|---|---|---|
| **Fresh run PASS verdict** | `analyze_evidence_run.py` returns PASS on a new run | NOT MET — no fresh run executed |
| **Human visual review** | Operator inspects bitblt/Bevy PNGs from the passing run | NOT MET — blocked on gate above |
| **AUTOPLAY-VS-BOT-QA-001 sign-off** | Analyzer PASS + human review → story marked DONE | NOT MET |

These gates are **human-only** — no agent can satisfy them. They require an
operator with a running dev environment.

---

## 7. Why Bot/Autoplay Has Taken Many Agents

The large agent count is explained by three compounding factors:

**1. Rebase churn from concurrent main advancement.**
Each time a bot/autoplay branch was ready to merge, another PROMPT had already
advanced `origin/main`. The branch became NOT_FF, and a "refresh after N" agent
was required to rebase the reports onto the new main. This created chains of
8-10+ refresh agents per report cluster (operator contract: 8 refreshes;
coverage map: 10+ refreshes; signoff pack: 6+ refreshes).

**2. Three evidence lanes running in parallel.**
- Viewport/window guard (source code) lane
- Placement-reject recipe (source code) lane
- Evidence analyzer/verdict (source code) lane
- Five+ report chains for each lane (stale → refresh → re-merge → reject →
  refresh again)

**3. The core evidence gap is operator-gated.**
All three known runs were PARTIAL. Every story-readiness and signoff-pack report
has had to restate "BLOCKED — no clean run" while the code work continued
around it. This created a large number of "readiness refresh" reports that could
not progress past BLOCKED until a human runs the driver.

**Summary:** The agent count is high because rebase cycles compounded across
five parallel report chains. The underlying implementation work was modest; the
report backfill and rebase overhead dominated.

---

## 8. Next Necessary Prompts

Only one is needed. No report churn is recommended.

### Required (Unblocks QA-001):

**Operator action, not a code PROMPT:**

> Operator runs: `tools/autoplay/Run-AutoplaySmoke.ps1` (or equivalent) with
> `CCGS_WINDOW_WIDTH=1280 CCGS_WINDOW_HEIGHT=720 CCGS_QA_SNAPSHOT=1`.
> Collect run evidence directory. If `analyze_evidence_run.py` returns PASS,
> the operator reviews screenshots and signs off AUTOPLAY-VS-BOT-QA-001.

If the run fails (window resize detected, frozen captures, or click misses), one
diagnostic PROMPT may be needed to identify the root cause. But the run must be
attempted first.

### Optional (If wanted before QA-001):

| PROMPT | What | Priority |
|---|---|---|
| `viewport_shrink_guard.py` driver integration | Replace inlined driver guards with `viewport_shrink_guard` module import | LOW — not blocking |
| FRAG-02 coord fix | Lower `HAND_FIRST_CARD`/`SUBMIT_BTN` fy from 0.92 → 0.88 in `_coords.py` | MEDIUM — reduces click miss risk |

### Do NOT queue:

See §9.

---

## 9. Stop-Doing List

The following PROMPT patterns should be retired immediately:

| Pattern | Why to stop |
|---|---|
| **Story readiness refresh** (PROMPT 1935/1970/1985) | Each one restates "BLOCKED — no clean run" with no new information. Will continue to be stale until a fresh run happens. Cannot contribute to QA-001 sign-off. |
| **Operator contract refresh** (PROMPT 1861/1914/1941/1964/1968/1976) | The operator contract is stable. Env vars, window requirements, and run procedure are documented and unchanged. No further refreshes needed unless the contract changes. |
| **Signoff pack refresh** (PROMPT 1841/1889/1911/1946/1956/1972) | There is no new evidence to sign off. Refreshes cannot upgrade PARTIAL runs to PASS. |
| **Coverage map refresh** (PROMPT 1848/1909/1924/1949/1967/1984/1995/2000/2007/2011) | The coverage map (FRAG-01 through FRAG-07) is complete and stable. Only new recipe changes should trigger a refresh. |
| **Viewport/window-guard verify refresh** (PROMPT 1916/1948/1966/1980) | The guard implementation has not changed since PROMPT 1894. Test count (66) and results (66/66 PASS) are stable. No further verify reports needed until code changes. |

---

## 10. Closure Summary

| Area | Status | Notes |
|---|---|---|
| Bot AI (server-side) | **DONE** | On main; no open items |
| Bot soak (headless) | **DONE** | On main; no open items |
| Autoplay driver foundation | **DONE** | On main; all guards active |
| Startup window-size floor (AC-VPT-01) | **DONE** | `enforce_autoplay_window_size_system` on main (PROMPT 1912) |
| Mid-run drift + OOB guards (AC-VPT-02/08) | **DONE** | `EXIT_VIEWPORT_GUARD` inlined in driver (PROMPT 1880/1894) |
| Window-resize verdict + hard-FAIL | **DONE** | `analyze_evidence_run.py` + `validate_composite_run.py` on main (PROMPT 1994) |
| Mid-run viewport shrink guard module | **DONE (standalone)** | `viewport_shrink_guard.py` on main (PROMPT 2009); NOT imported by driver — architectural loose end |
| Placement-reject recipe | **DONE** | `placement_reject_probe.py` in REGISTRY on main (PROMPT 2013) |
| Recipe coverage map | **DONE (documented)** | FRAG-02/03 identified, not fixed |
| Test suite | **DONE** | 31 + 66 + 83 + verdict suite — all PASS |
| Report chains (all five) | **DONE (landed)** | All backfills on main via PROMPT 2016 |
| **AUTOPLAY-VS-BOT-QA-001 live PASS** | **BLOCKED** | Operator environment gate — no fresh run with full guard stack |
| Human QA sign-off | **BLOCKED** | Blocked on live PASS above |

**The bot/autoplay work is code-complete. The only remaining gate is a
human-operated fresh run of the autoplay driver against a live game session.**

---

## 11. Validation

```
git diff --name-status origin/main..HEAD
A  reports/PROMPT-2018-bot-autoplay-current-state-closure-audit-after-2016.md
```

One file added, zero deletes, zero source/test/production edits.

`git diff --check origin/main..HEAD` — clean (no trailing whitespace).

---

2018: BOT-AUTOPLAY-CURRENT-STATE-CLOSURE-AUDIT-AFTER-2016: SHIPPED
