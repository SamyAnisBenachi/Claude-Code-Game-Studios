# PROMPT 2023 — POST-2019/2022 Game Completion Readiness Refresh

**Date**: 2026-05-28
**Branch**: `work/PROMPT-2023`
**Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2023`
**Source-of-truth**: `origin/main@e7b51e84` (PROMPT 2019 — S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 Done)
**Worker**: Claude Code — PROMPT 2023 read-only readiness refresh (no source edits)

---

## 1. Executive State

**Stage**: `Polish` (unchanged; `production/stage.txt` not modified)
**Sprint**: 18 (active since PROMPT 1301)
**Sprint 18 progress**: **9 of 12 active rows DONE** (75%)
**Stage advance gate**: PROMPT 761 `Polish → Release` gate-check **FAIL — NO RETRY authorized**
**Tests (last smoke baseline)**: 263 suites / 1861 tests / 0 failed (Sprint 17 evidence PROMPT 1284/1289)

**In plain terms**: The chain PROMPT 2019–2022 closed one Sprint 18 story (OBS snapshot
layout fields), added the `poll_phase` autoplay pseudo-action, documented the Add Bot
coordinate measurement protocol, and verified that `S18-UI-CARD-ART-AND-LABEL-STRIP-001`
is already implemented on main. Sprint 18 is now **9/12** with two automatable rows still
open: one paperwork-only and one unimplemented. The only Must Have row still open
(`S11-HUD-TIMER-EYEBALL-VISUAL-001`) remains human-operator-blocked, as it has been since
Sprint 13. The autoplay fragility register has been partially retired: FRAG-03 (no
`poll_phase`) and R-04 (Add Bot docs gap) are now closed on main.

---

## 2. What Changed: PROMPT 2019 → 2022

### PROMPT 2019 — S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 Story-Done

- Closed `S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001` (Should Have) via paperwork-only `/story-done`.
- Implementation was pre-landed at PROMPT 1186 (`d75db1af`); all 14 integration tests PASS.
- AC1–AC18 all PASS (Q-05/Q-06/Q-07 PASS-WITH-LIMITATIONS, documented with file:line refs).
- Sprint 18 coverage: **8/12 → 9/12**.
- Evidence file created: `production/qa/evidence/sprint-18-snapshot-layout-fields/evidence.md`.

### PROMPT 2020 — `poll_phase()` Pseudo-Action (FRAG-03 / R-02 CLOSED)

- Implemented `RecipeBuilder.poll_phase(label, max_ticks)` in `tools/autoplay/recipes/_builder.py`.
- Implemented `_poll_for_phase()` handler in `tools/autoplay/driver.py` with injected
  `_rpc`/`_sleep` params for unit-test isolation.
- 39 new tests in `tests/tools/autoplay/test_poll_phase.py` — all pass.
- Backward compatibility confirmed: 163 pre-existing tests (recipe_static + viewport_guard +
  screenshot_barrier) pass unchanged.
- **FRAG-03 is CLOSED.** Phase-aware polling is available; recipes are no longer forced to
  use brittle fixed-tick `wait()` calls.

### PROMPT 2021 — Add Bot Coordinate Measurement Protocol (R-04 CLOSED)

- New section `## Add Bot Coordinate Measurement Protocol` added to `docs/autoplay.md`.
- `docs/autoplay/evidence-operator-guide.md` updated with `CCGS_AUTOPLAY_ADD_BOT_BTN`
  coordinate override row, window-size preflight callout, and two new Common Failures rows.
- Documents: minimum window size (≥1280×720), 8-step capture procedure, re-measure triggers,
  evidence requirements for click-miss bug reports.
- **R-04 is CLOSED.** No further documentation refreshes needed for Add Bot coordinates.

### PROMPT 2022 — S18-UI-CARD-ART-AND-LABEL-STRIP-001 Slice Verify

- **Key finding**: The implementation was already on `origin/main@05014373` via PROMPT 1348
  (`26bc1204`) + PROMPT 1403 (`ac8c0a20`).
- All 14 ACs pass static verification: `CardSlotArtImage` / `CardSlotLabelStrip` markers
  defined and exported, `NodeImageMode::Auto` enforced, label strip opaque (α=0.92 ≥ 0.85),
  three consumer systems migrated (`sync_hand_fan_card_art_system`, `handle_draft_offering_system`,
  `auction_featured_card_node`).
- `cargo check -p client` clean (zero errors; only pre-existing deprecation warnings unrelated
  to this story).
- Both integration test targets exist and are registered in `client/Cargo.toml`:
  `card_art_aspect_fit_test` (5 tests) and `auction_featured_art_binding_test` (8 tests).
- **This story requires story-done paperwork only** — the implementation is on main.

---

## 3. Sprint 18 Current State (origin/main@e7b51e84)

### Closed Rows — 9 of 12

| Row ID | Priority | Closed By | Status |
|---|---|---|---|
| `S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001` | Must Have | PROMPT 1337 | DONE |
| `S18-UI-PLAY-AREA-CONTAINER-001` | Must Have | PROMPT 1712 | DONE |
| `S18-AUCTION-WON-CARD-DISPOSITION-001` | Must Have | PROMPT 1713 | DONE |
| `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001` | Should Have | PROMPT 1357 | DONE |
| `S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001` | Should Have | PROMPT 1331 | DONE |
| `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001` | Should Have | PROMPT 1716 | DONE |
| `S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001` | Should Have | PROMPT 1717 | DONE |
| `S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001` | Nice to Have | PROMPT 1718 | DONE |
| `S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001` | Should Have | **PROMPT 2019** | DONE ← new |

**Dropped at activation**: `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` — no story file on main at activation; story-authoring-needed; Sprint 19 candidate.

### Open Rows — 3 of 12

| Row ID | Priority | Current Status | Nature |
|---|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | Must Have | READY (impl on main); no LLM `/story-done` authorized | Human-operator-blocked carry Sprint 13→18 |
| `S18-UI-CARD-ART-AND-LABEL-STRIP-001` | Should Have | READY (impl on main; verified PROMPT 2022) | Paperwork-only — needs `/story-done` only |
| `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` | Nice to Have | READY (not implemented) | Needs `/dev-story`; PROMPT 1190 J-class polish |

**Earliest Sprint 18 Must/Should closure** (ignoring human gate): after
`S18-UI-CARD-ART-AND-LABEL-STRIP-001` story-done paperwork lands → Sprint 18 is 10/12 on
Must+Should (3/3 Must Have closed, 6/6 Should Have closed). Sprint 18 closeout can then
proceed with `S11-HUD-TIMER-EYEBALL-VISUAL-001` carried under formal waiver/deferred
conditions, consistent with Sprint 15/16/17 close-with-conditions precedents.

---

## 4. Core Game Implementation

All 30+ production epics outside `bot-and-autoplay` and `ui-clean-pass` wave 2 are in a
completed state on `origin/main`. This includes: `auction-system`, `board-lane-system`,
`board-rendering`, `card-acquisition`, `card-animations`, `card-data-pool`, `class-system`,
`combat-resolution`, `economy-system`, `game-config-pipeline`, `game-session-system`,
`hand-ui` (stories 001–019), `hud` (stories 001–013), `keyword-system`,
`lightyear-protocol-verification`, `objective-system`, `playable-client`,
`presentation-layer`, `prism-system`, `round-state-machine`, `server`, `server-rng`,
`shop-auction-ui` (core stories), `workspace-and-shared-types`.

**No open implementation items** for core game logic.

---

## 5. Bot / Autoplay Automation State

### Bot AI and Soak

| Area | Status |
|---|---|
| Server-side bot AI | DONE — on main |
| Headless bot soak | DONE — on main |

### Autoplay Tooling (post-PROMPT-2020/2021)

| Component | Status | Notes |
|---|---|---|
| Core driver (`driver.py`) | DONE | AC-VPT-01/02/08 guards active |
| Viewport shrink guard module | DONE (standalone) | `viewport_shrink_guard.py` — NOT imported by driver (arch loose end; not blocking) |
| Window-resize verdict | DONE | `analyze_evidence_run.py` + `validate_composite_run.py` (PROMPT 1994) |
| Placement-reject recipe | DONE | `placement_reject_probe.py` in REGISTRY (PROMPT 2013) |
| `poll_phase()` pseudo-action | **DONE** | PROMPT 2020 — FRAG-03 CLOSED |
| Add Bot coord measurement docs | **DONE** | PROMPT 2021 — R-04 CLOSED |
| All autoplay test suites | DONE | 39 + 31 + 66 + 83 + verdict suite — all PASS |

### Autoplay Fragility Register (post-2022 state)

| ID | Description | Priority | Status |
|---|---|---|---|
| FRAG-01 | Lobby loaded confirmation gap | LOW | Documented, no repair needed |
| ~~FRAG-02~~ (R-01) | `HAND_FIRST_CARD`/`SUBMIT_BTN` fy=0.92 → 0.88 coord fix | MEDIUM | **Still open** — `tools/autoplay/recipes/_coords.py` edit needed |
| ~~FRAG-03~~ (R-02) | No `poll_phase()` pseudo-action | ~~HIGH~~ | **CLOSED — PROMPT 2020** |
| FRAG-04 | `BOT_GAME_TIMEOUT_SECS` tuning gap | LOW | Documented |
| FRAG-05 | No structured screenshot check on checkpoint sequence | LOW | Documented |
| R-03 | Log warning on `window_logical_size` fallback | LOW | Still open |
| ~~R-04~~ | Add Bot coordinate measurement protocol | ~~HIGH~~ | **CLOSED — PROMPT 2021** |

**Remaining fragility requiring a worker**: Only FRAG-02/R-01 (fy coord fix, MEDIUM) and
R-03 (log warning, LOW) are unimplemented. Neither blocks `AUTOPLAY-VS-BOT-QA-001` execution;
they are click-miss risk reduction and diagnostics.

### Live-Run Gate (AUTOPLAY-VS-BOT-QA-001)

**Status: BLOCKED — operator environment gate.**

No fresh autoplay run has been executed with the full guard stack (AC-VPT-01 + AC-VPT-02/08
+ composite verdict). The three known runs from 2026-05-28 predate the full guard stack and
are all PARTIAL. The code is complete. Operator must:

1. Start Bevy client with `CCGS_WINDOW_WIDTH=1280 CCGS_WINDOW_HEIGHT=720 CCGS_QA_SNAPSHOT=1`
   (or `tools/autoplay/Run-AutoplaySmoke.ps1`)
2. Ensure DWM does not resize the window mid-run
3. Run `python driver.py --recipe vs-bot`
4. If `analyze_evidence_run.py` returns PASS, review bitblt/Bevy PNGs and sign off
   `AUTOPLAY-VS-BOT-QA-001`

---

## 6. Human GUI Evidence Gates

| Gate | Status | Path to Resolution |
|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` visual sign-off | BLOCKED-HUMAN-OPERATOR (Sprint 13→18 carry) | Human captures HUD phase timer screenshot; then thin PROMPT runs `/story-done` citing screenshot |
| PROMPT 1054 P1 UI snapshot visual retest | BLOCKED-HUMAN-OPERATOR | Multi-sprint carry; requires human visual comparison of UI snapshots |
| `AUTOPLAY-VS-BOT-QA-001` fresh run + human visual review | BLOCKED-HUMAN-OPERATOR | Operator runs `Run-AutoplaySmoke.ps1`; `analyze_evidence_run.py` must return PASS |
| `S8-QA-001-W1` Two-client GAME_OVER closure | OPEN | Requires two-client live test with manual verification |

---

## 7. Release / Polish Gates

| Gate | Status | Notes |
|---|---|---|
| `Polish → Release` gate-check retry | BLOCKED-EXPLICIT-USER-GATE | PROMPT 761 FAIL, 2026-05-12; NO retry authorized in any sprint through S18; requires explicit user authorization |
| `QA-COND-0005` Standard accessibility tier | ACCEPTED-RISK | Friend-game scope |
| `QA-COND-0006` Playtest / fun-hypothesis | ACCEPTED-RISK / DEFERRED | No timeline |
| `PAW-TD-*-a` Placeholder art completion | ACCEPTED-RISK | Release-stage; gate blocked |
| `TQ-S12-C7` | OPEN / CARRIED | 6+ sprint carry; no discharge path defined |

---

## 8. Next Launchable Worker Prompts

### Group A — Paperwork-Only (parallel-safe; no source edits)

**PROMPT A1 — `S18-UI-CARD-ART-AND-LABEL-STRIP-001` story-done (paperwork-only)**

- Scope: `/story-readiness` verify against `origin/main@e7b51e84` confirming PROMPT 2022
  AC audit (all 14 ACs PASS), then `/story-done`.
- Implementation on main: PROMPT 1348 (`26bc1204`) + PROMPT 1403 (`ac8c0a20`).
- Owned files: `production/epics/ui-clean-pass/story-022-ui-card-art-and-label-strip.md`,
  `production/sprint-status.yaml` (story-done block append only), `reports/`.
- Forbidden: `client/**`, `server/**`, `shared/**`, `tests/**`, `Cargo.toml`, stage.txt.
- Validation: `git diff --check`; no Cargo invocation needed.
- Expected outcome: Should Have row closed; Sprint 18 coverage 10/12; all Must+Should closed.
- **This is the highest-priority automatable PROMPT. Once A1 lands, Sprint 18 can close
  (with S11-HUD-TIMER-EYEBALL-VISUAL-001 carried under formal waiver).**

### Group B — Sprint 18 Closeout (sequence after A1)

**PROMPT B1 — Sprint 18 Smoke + Team-QA + Closeout**

- Scope: `/smoke-check` → `/team-qa` → Sprint 18 close-out disposition
- Prerequisite: A1 must land on main first.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` carries under deferred-with-conditions waiver (human
  operator evidence; consistent with Sprint 15/16/17 close-with-conditions precedents).
- `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` (Nice to Have) deferred to Sprint 19.
- Expected outcome: Sprint 18 `status: active → closed-with-conditions`; Sprint 19 activation
  ready.

### Group C — Autoplay Hardening (parallel-safe; tools/autoplay only)

These can run in parallel with Group A and with each other.

**PROMPT C1 — Autoplay FRAG-02/R-01: Coord fix (`HAND_FIRST_CARD`/`SUBMIT_BTN` fy=0.92→0.88)**

- Scope: Edit `tools/autoplay/recipes/_coords.py` to lower `HAND_FIRST_CARD` and
  `SUBMIT_BTN` fy from `0.92` to `0.88`; update any affected test assertions.
- Owned files: `tools/autoplay/recipes/_coords.py`,
  `tests/tools/autoplay/test_recipe_static.py` (if coordinate values are asserted there).
- No Cargo; focused pytest run to confirm static recipe tests still pass.
- Expected outcome: FRAG-02 click-miss risk at 720px window height downgraded from MEDIUM
  to LOW/CLOSED.

**PROMPT C2 — Autoplay R-03: Log warning on `window_logical_size` fallback**

- Scope: Add `logging.warning(...)` in `tools/autoplay/driver.py` at the
  `window_logical_size` fallback path (line ~226–229 as noted in PROMPT 2017/2018).
- Owned files: `tools/autoplay/driver.py` only.
- Validation: `git diff --check`; no test changes needed.
- Expected outcome: Fallback path now visible in driver logs; diagnostic gap closed.

### Group D — Sprint 19 Preparation (sequence after B1)

**PROMPT D1 — Sprint 19 Plan Draft + Activation**

- Prerequisite: Sprint 18 closeout (B1) on main.
- Scope: Author Sprint 19 plan with primary focus on:
  - `BOT-SOAK-ENTRYPOINT-001` and `AUTOPLAY-VS-BOT-QA-001` execution (operator-gated but
    story-authored and ready for human attempt)
  - `S11-HUD-TIMER-EYEBALL-VISUAL-001` waiver or final carry decision
  - `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` and `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001`
    story authoring
- No source edits; production planning files only.

### Group E — Human-Gated (not dispatchable; requires operator action)

**E1 — `S11-HUD-TIMER-EYEBALL-VISUAL-001` visual sign-off**

- Action: Human launches game, captures HUD phase timer screenshot across DraftInitial 45s /
  DraftShop 30s / Placement phases, attaches to
  `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`.
- After screenshot attached: thin PROMPT runs `/story-done` citing the operator screenshot.
- **No LLM worker can substitute for this step.**

**E2 — `AUTOPLAY-VS-BOT-QA-001` fresh run**

- Action: Operator runs `tools/autoplay/Run-AutoplaySmoke.ps1` with
  `CCGS_WINDOW_WIDTH=1280 CCGS_WINDOW_HEIGHT=720 CCGS_QA_SNAPSHOT=1`.
- If `analyze_evidence_run.py` returns PASS: review bitblt PNGs, sign off story.
- If FAIL: one diagnostic PROMPT to identify root cause (viewport resize, frozen captures,
  click miss).

**E3 — `Polish → Release` gate-check retry authorization**

- Action: User explicitly authorizes a gate-check retry (or decides to accept Polish as
  final stage for this milestone).
- Once authorized: dispatch `/gate-check` with full evidence sweep.

---

## 9. Priority Summary Table

| PROMPT | Group | Parallelizable with | Blocking |
|---|---|---|---|
| A1 (`S18-CARD-ART` story-done) | A | C1, C2 | Sprint 18 Must+Should closure; Sprint 18 closeout trigger |
| B1 (Sprint 18 Smoke+Close) | B | — | Sprint 19 activation |
| C1 (FRAG-02 coord fix) | C | A1, C2 | `AUTOPLAY-VS-BOT-QA-001` click-miss risk |
| C2 (R-03 fallback log) | C | A1, C1 | Diagnostics quality |
| D1 (Sprint 19 plan) | D | — | All Sprint 19 work |
| E1 (HUD timer human sign-off) | E | All | Final Sprint 18 Must Have row closure |
| E2 (Autoplay fresh run) | E | All | `AUTOPLAY-VS-BOT-QA-001` |
| E3 (gate-check retry auth) | E | All | `Polish → Release` stage advance |

**Critical path**: `A1 → B1 → D1` is the minimum sequence to close Sprint 18 and activate
Sprint 19. C1/C2 are parallel risk reduction. E1/E2/E3 are human-only and cannot be
accelerated by workers.

---

## 10. Risks

| Risk | Severity | Notes |
|---|---|---|
| Sprint 18 never formally closes — S11-HUD-TIMER carry (Sprint 13→18) | MEDIUM | User can authorize formal waiver/deferred-with-conditions closure; prior precedent exists in Sprints 15/16/17 |
| `Polish → Release` gate stays indefinitely blocked | HIGH | No retry authorized and no retry sprint defined; code drift builds between PROMPT 761 failure evidence (2026-05-12) and current codebase state |
| AUTOPLAY-VS-BOT-QA-001 never attempted | HIGH | Code is complete; only barrier is operator environment gate; risk grows as gap between last code change and first live-run attempt widens |
| FRAG-02 click miss at 720px window | MEDIUM | fy=0.92 leaves 58px below SUBMIT/HAND at 720px — marginal; fix is a two-line `_coords.py` change |
| `S8-QA-001-W1` two-client closure deferred indefinitely | LOW | Carried through 6+ sprints; no discharge path; not currently blocking Sprint 18 closeout |

---

## 11. Evidence Cited

| Source | What It Confirmed |
|---|---|
| `origin/main@e7b51e84` (`production/sprint-status.yaml`) | Sprint 18 active; 9 story-done blocks (PROMPT 1337/1331/1357/1712/1713/1716/1717/1718/2019); `S18-UI-CARD-ART-AND-LABEL-STRIP-001` status: ready; `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` status: unimplemented |
| `reports/PROMPT-2017-game-completion-next-wave-map-after-2016.md` | Baseline state (8/12); open rows; autoplay R-01..R-04 register |
| `reports/PROMPT-2018-bot-autoplay-current-state-closure-audit-after-2016.md` | Bot AI DONE; autoplay code-complete; FRAG-02/03 open; live-run operator gate; stop-doing list |
| `reports/PROMPT-2019-s18-obs-snapshot-layout-fields-story-done-paperwork.md` | S18-OBS story DONE; Sprint 18 → 9/12; AC1–AC18 PASS |
| `reports/PROMPT-2020-autoplay-poll-phase-pseudo-action-r02.md` | `poll_phase()` shipped; 39 new tests PASS; 163 existing tests PASS; FRAG-03 CLOSED |
| `reports/PROMPT-2021-autoplay-add-bot-coordinate-measurement-docs-r04.md` | Add Bot measurement protocol documented in `docs/autoplay.md` + `evidence-operator-guide.md`; R-04 CLOSED |
| `reports/PROMPT-2022-s18-ui-card-art-and-label-strip-slice.md` | S18-UI-CARD-ART-AND-LABEL-STRIP-001 implementation on main (PROMPT 1348/1403); all 14 ACs PASS; `cargo check -p client` clean; story-done paperwork only needed |
| `git log --oneline origin/main` (last 5) | Confirmed e7b51e84 as current tip; PROMPT 2019 DONE commit; PROMPT 2020/2021 autoplay refreshes; PROMPT 2022 card art report |

---

2023: POST-2019-2022-GAME-COMPLETION-READINESS-REFRESH: SHIPPED
