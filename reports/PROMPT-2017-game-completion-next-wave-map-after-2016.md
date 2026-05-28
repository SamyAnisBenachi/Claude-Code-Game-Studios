# PROMPT 2017 — Game-Completion Next-Wave Map — After PROMPT 2016

**Date**: 2026-05-28
**Branch**: `work/PROMPT-2017`
**Worktree**: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2017`
**Source-of-truth main**: `origin/main@3966d1c1` (PROMPT 2016 — POST-2015 result-screen overflow
  post-mainland verify report)
**Worker**: Claude Code — PROMPT 2017 planning/audit report (read-only)

---

## 1. Executive State

**Stage**: `Polish` (locked; `production/stage.txt` unchanged)
**Sprint**: 18 (active; activated PROMPT 1301 against `origin/main@1345c6b8`)
**Sprint 18 progress**: **8 of 12 active rows DONE** (66.7%)
**Stage advance gate**: PROMPT 761 `Polish → Release` gate-check **FAIL — NO RETRY authorized**
**Tests (last smoke)**: 263 suites / 1861 tests / 0 failed (Sprint 17 evidence; PROMPT 1284 / 1289 baseline)

**In plain terms**: The game's core implementation (lanes, combat, auction, economy, hand UI, board
rendering, HUD, shop/auction UI, result screen, networking) is substantially complete on
`origin/main`. Sprint 18 is a Polish-stage UI cohesion + observability sprint — the last
identified slice before Sprint 18 can be closed. Sprint 19 is planned for bot/autoplay QA
automation. The release gate is technically blocked (PROMPT 761 FAIL); clearing it requires an
explicit user-driven gate-check retry.

**Recent chain (PROMPT 2005 → 2016)** was entirely housekeeping: reapply operations that brought
stale branches (lobby class picker fix, autoplay viewport shrink guard, placement-reject recipe,
result-screen overflow guard) back onto a main that had advanced past their roots. No new features
landed in this chain.

---

## 2. Done / Landed

### Sprint 18 Closed Rows (8/12)

| Row ID | Priority | Closed By | Basis |
|---|---|---|---|
| `S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001` | Must Have | PROMPT 1337 | PROMPT 1188 impl (`c2eaab0`) + PROMPT 1208 lint-baseline bump + PROMPT 1334 AC9 cross-link |
| `S18-UI-PLAY-AREA-CONTAINER-001` | Must Have | PROMPT 1712 | PROMPT 1328 worker (`475269b7`) + PROMPT 1339/1343 replay; `play_area_budget_test` 13/13 PASS |
| `S18-AUCTION-WON-CARD-DISPOSITION-001` | Must Have | PROMPT 1713 | PROMPT 1347/1409/1513/1518 impl chain; all AC PASS |
| `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001` | Should Have | PROMPT 1357 | PROMPT 1239 + 1243 impl (`50b66ad` + `4c75cec`); paperwork-only |
| `S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001` | Should Have | PROMPT 1331 | PROMPT 1187 impl (`8eeb94e`); paperwork-only (attribution corrected PROMPT 1324) |
| `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001` | Should Have | PROMPT 1716 | PROMPT 1228 + 1336 + 1344 impl (`8d0a3d3` lineage); paperwork-only |
| `S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001` | Should Have | PROMPT 1717 | PROMPT 1185 + 1333 impl (`671c677`); paperwork-only |
| `S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001` | Nice to Have | PROMPT 1718 | PROMPT 1349 worker (`f7cfa422`) + PROMPT 1371 integration + PROMPT 1375 main-land |

**Dropped at activation**: `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` — no story file on
`origin/main` at activation; recorded as `not activated / story-authoring-needed` in
`sprint_18_activation.dropped_rows`; re-evaluation deferred to Sprint 19.

### Core Game Implementation (all prior sprints)

All 30+ production epics outside `bot-and-autoplay` and `ui-clean-pass` wave 2 are in a
completed state on `origin/main`. This includes: `auction-system`, `board-lane-system`,
`board-rendering`, `card-acquisition`, `card-animations`, `card-data-pool`, `class-system`,
`combat-resolution`, `economy-system`, `game-config-pipeline`, `game-session-system`,
`hand-ui` (stories 001–019), `hud` (stories 001–013), `keyword-system`,
`lightyear-protocol-verification`, `objective-system`, `playable-client`,
`presentation-layer`, `prism-system`, `round-state-machine`, `server`, `server-rng`,
`shop-auction-ui` (core stories), `workspace-and-shared-types`.

Source: `production/epics/` directory inventory; `production/sprint-status.yaml` story-done
blocks Sprint 10 through Sprint 18.

---

## 3. Remaining P0 (Must Have — blocks Sprint 18 closure)

### P0-A: `S11-HUD-TIMER-EYEBALL-VISUAL-001` — Human-Operator-Blocked Carry

- **Status**: READY (implemented); no LLM `/story-done` authorized
- **Carried from**: Sprint 13 → 14 → 15 → 16 → 17 → 18
- **Blocker**: Requires human-operator screenshot capture of the HUD phase timer visual
  and sign-off. The implementation is on `origin/main` but visual verification is advisory
  and explicitly not delegable to an LLM.
- **Story file**: `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
- **Resolution**: Human operator runs the game, captures the HUD timer at the relevant
  phase, attaches screenshot to the story file, then a thin paperwork-only PROMPT runs
  `/story-done` citing the operator's screenshot evidence.
- **Workaround available?** No — the carry condition is explicit: "no LLM `/story-done`
  authorised." Sprint 18 can close with this row formally waived/deferred to Sprint 19 if
  the user decides the human sign-off is not a blocker for closeout.

### P0-B: `Polish → Release` Gate-Check Retry

- **Status**: BLOCKED — PROMPT 761 gate-check returned FAIL; NO RETRY authorized in
  any sprint through Sprint 18.
- **File**: `production/gate-checks/gate-polish-release-2026-05-12.md`
- **Resolution**: Requires explicit user decision to authorize a retry. This is not a
  worker-dispatchable prompt — it is a deliberate human gate.
- **Impact**: Until this gate passes, `production/stage.txt` cannot flip to `Release` and
  all Release-stage work (certification, store submission, final QA gates) is blocked.

---

## 4. Remaining P1 (Should Have / Nice to Have — Sprint 18 open rows)

### P1-A: `S18-UI-CARD-ART-AND-LABEL-STRIP-001` (Should Have)

- **Status**: Unimplemented; **now unblocked** (sequencer `S18-UI-PLAY-AREA-CONTAINER-001`
  closed by PROMPT 1712)
- **Story file**: `production/epics/ui-clean-pass/story-022-ui-card-art-and-label-strip.md`
- **Source**: PROMPT 1180 Lane C
- **Estimated effort**: ~0.5d
- **Required path**: `/story-readiness` against `origin/main@3966d1c1` → `/dev-story`
- **File ownership**: `client/src/ui/` (card rendering strip components)

### P1-B: `S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001` (Should Have)

- **Status**: Implementation already on `origin/main` (commit `e68ac4f`, PROMPT 1229);
  only `/story-readiness` + `/story-done` paperwork needed
- **Story file**: `production/epics/ui-clean-pass/story-023-obs-snapshot-layout-fields.md`
- **Estimated effort**: ~0.1d (paperwork-only)
- **Required path**: `/story-readiness` verify → `/story-done`

### P1-C: `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` (Nice to Have)

- **Status**: Unimplemented; PROMPT 1190 J-class polish
- **Story file**: `production/epics/ui-clean-pass/story-025-ui-interaction-state-migration-wave-2.md`
- **Estimated effort**: ~0.35d
- **Required path**: `/story-readiness` → `/dev-story`
- **File ownership**: `client/src/ui/` (interaction state primitives)
- **Note**: Nice to Have tier — can be deferred to Sprint 19 without blocking Sprint 18
  Must/Should closure if capacity is tight.

### P1-D: Autoplay Hardening (open repair register)

From `reports/PROMPT-2000-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1994.md`
and `reports/PROMPT-2007-autoplay-recipe-visible-target-coverage-map-refresh-after-1980.md`:

| Repair | Risk | File | Status |
|---|---|---|---|
| R-01: Lower HAND/SUBMIT fy from 0.92 → 0.88 | LOW | `tools/autoplay/recipes/_coords.py` | Open |
| R-02: Add `poll_phase(label, max_ticks)` to `_builder.py` / `driver.py` | HIGH (FRAG-03) | `tools/autoplay/_builder.py`, `driver.py` | Open |
| R-03: Log warning on `window_logical_size` fallback | LOW | `tools/autoplay/driver.py` line 226–229 | Open |
| R-04: Document Add Bot button coordinate measurement protocol | HIGH (FRAG-02) | `docs/autoplay.md` | Open |

R-02 and R-04 are HIGH-risk: their absence means the autoplay bot test recipes depend on
time-based waits (not phase-label polling) and the Add Bot button has no verified coordinate
baseline. Both block reliable `BOT-SOAK-ENTRYPOINT-001` and `AUTOPLAY-VS-BOT-QA-001` execution
in Sprint 19.

### P1-E: `S17-UI-HUD-OPP-MANA-CLEANUP-001` Parent-Row Paperwork Gap

- Source repair (AC3 hand-reserve microbadge) is on `origin/main`
- No final `/story-done` paperwork closed the Sprint 17 parent row
- Preferred discharge path is `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` story authoring,
  but no story file exists yet on `origin/main`
- Sprint 19 candidate; requires story authoring prompt first

---

## 5. Human QA Gates (not dispatchable to LLM workers)

| Gate | Status | Condition |
|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` visual sign-off | BLOCKED-HUMAN-OPERATOR | Screenshot capture of HUD phase timer by human operator |
| PROMPT 1054 P1 UI snapshot visual retest | BLOCKED-HUMAN-OPERATOR | Multi-sprint carry; requires human visual comparison of UI snapshots |
| `Polish → Release` gate-check retry (PROMPT 761) | BLOCKED-EXPLICIT-USER-GATE | User must explicitly authorize the retry; no worker can dispatch this |
| `QA-COND-0005` Standard accessibility tier | ACCEPTED-RISK | Friend-game scope only; no timeline |
| `QA-COND-0006` Playtest / fun-hypothesis validation | ACCEPTED-RISK / DEFERRED | No timeline |
| `S8-QA-001-W1` Two-client GAME_OVER closure | OPEN | Requires two-client live test with manual verification |
| `TQ-S12-C7` | OPEN | Part of TQ-S12-C1..C7 preserved block; no concrete repair on main |
| `PAW-TD-*-a` Placeholder art completion | ACCEPTED-RISK | Final art not in scope until Release stage unlocked |

---

## 6. Recommended Parallel Prompt Queue (Next 6–10 Prompts)

### Group A — Paperwork-Only (parallel-safe; no source edits)

Can run in parallel with each other and with Groups B/C.

**PROMPT A1 — `S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001` story-done (paperwork-only)**
- Scope: `/story-readiness` verify against `origin/main@3966d1c1`, then `/story-done`
- Owned files: `production/epics/ui-clean-pass/story-023-obs-snapshot-layout-fields.md`,
  `production/sprint-status.yaml` (story-done block append only)
- Implementation on main: commit `e68ac4f` (PROMPT 1229)
- Validation: `git diff --check`; no Cargo invocation needed
- Expected outcome: Should Have row closed; Sprint 18 coverage 9/12

**PROMPT A2 — `BOT-ROOM-PARTICIPANT-001` story-done (Sprint 19 carry tracking)**
- Scope: `/story-readiness` verify + `/story-done` for bot-room-participant (already
  implemented on `origin/main`)
- Owned files: `production/epics/bot-and-autoplay/story-001-bot-room-participant.md`,
  `production/sprint-status.yaml`
- Validation: `git diff --check`; no Cargo invocation
- Expected outcome: Sprint 19 carry-tracking story closed before Sprint 19 activates

### Group B — Implementation Workers (serialize B1 then B2; or parallel if file audit confirms no overlap)

**PROMPT B1 — `S18-UI-CARD-ART-AND-LABEL-STRIP-001` dev-story**
- Scope: `/story-readiness` → `/dev-story` (Lane C from PROMPT 1180)
- Owned files: `client/src/ui/` card rendering strip components; story file;
  sprint-status.yaml
- Constraint: `S18-UI-PLAY-AREA-CONTAINER-001` (sequencer) is NOW DONE (PROMPT 1712);
  this row is unblocked
- Validation: `cargo test --workspace --tests --no-fail-fast` against owned files; `git diff --check`
- Expected outcome: Should Have row closed; Sprint 18 coverage 10/12 (after A1)

**PROMPT B2 — `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` dev-story**
- Scope: `/story-readiness` → `/dev-story` (PROMPT 1190 J-class polish)
- Owned files: `client/src/ui/` interaction state primitives (verify no overlap with B1
  before parallel launch)
- Tier: Nice to Have — can defer to Sprint 19 if Sprint 18 capacity is tight
- Validation: `cargo test --workspace --tests --no-fail-fast` against owned files
- Expected outcome: Nice to Have row closed; Sprint 18 coverage 11/12 (after A1 + B1)

### Group C — Autoplay Hardening (parallel-safe; tools/autoplay only)

Can run in parallel with Groups A and B; all edits stay in `tools/autoplay/` or `docs/`.

**PROMPT C1 — Autoplay R-02: `poll_phase()` pseudo-action**
- Scope: Add `poll_phase(label, max_ticks)` pseudo-action to `tools/autoplay/_builder.py`
  and `tools/autoplay/driver.py`; update `tests/tools/autoplay/` unit tests
- Owned files: `tools/autoplay/_builder.py`, `tools/autoplay/driver.py`,
  `tests/tools/autoplay/` (adds new test; does not touch existing test files)
- No Cargo/Trunk invocation needed; Python test only
- Expected outcome: FRAG-03 (phase-wait reliability) downgraded from HIGH to LOW/CLOSED

**PROMPT C2 — Autoplay R-04: Add Bot measurement protocol documentation**
- Scope: Author `docs/autoplay.md` section documenting how to measure and verify the
  `LOBBY_ADD_BOT_BTN (0.5, 0.72)` fractional coordinate baseline; include screenshot
  annotation procedure
- Owned files: `docs/autoplay.md` only
- No code changes; no Cargo invocation
- Expected outcome: FRAG-02 (Add Bot visibility proof) has documented protocol; unblocks
  `BOT-SOAK-ENTRYPOINT-001` pre-conditions for Sprint 19

### Group D — Human-Gated (not dispatchable; requires operator action)

**D1 — `S11-HUD-TIMER-EYEBALL-VISUAL-001` visual sign-off**
- Action required: Human operator launches game, navigates to a live round, captures
  HUD phase timer screenshot, attaches to story file
- After screenshot is attached: a thin PROMPT can run `/story-done` citing the screenshot
- No LLM worker can substitute for this step

**D2 — `Polish → Release` gate-check retry authorization**
- Action required: User explicitly authorizes a gate-check retry (or decides to accept
  Polish as the final stage for this milestone)
- Once authorized: dispatch `/gate-check` with full evidence sweep

### Summary Table

| PROMPT | Group | Can parallelize with | Blocks |
|---|---|---|---|
| A1 (S18-OBS story-done) | A | B1, B2, C1, C2 | Sprint 18 closeout prep |
| A2 (BOT-ROOM story-done) | A | A1, B1, B2, C1, C2 | Sprint 19 activation |
| B1 (CARD-ART dev-story) | B | A1, A2, C1, C2 | Sprint 18 Must/Should closure |
| B2 (INTERACTION-STATE dev-story) | B | A1, A2, C1, C2 | Sprint 18 Nice-to-Have closure |
| C1 (poll_phase R-02) | C | A1, A2, B1, B2, C2 | BOT-SOAK + AUTOPLAY-VS-BOT in S19 |
| C2 (Add Bot R-04 docs) | C | A1, A2, B1, B2, C1 | BOT-SOAK pre-conditions |
| D1 (human timer sign-off) | D | All | Final 1-of-12 Sprint 18 row |
| D2 (gate-check retry auth) | D | All | Polish → Release stage advance |

**Earliest Sprint 18 closeout** (ignoring human gates): after A1 + B1 land + either B2 lands
or user waives the Nice to Have. Then `/smoke-check` → `/team-qa` → Sprint 18 closeout prompt.

**Sprint 19 activation prerequisite**: Sprint 18 closeout (or formal waved-with-conditions),
Sprint 19 plan draft, then activation prompt.

---

## 7. Risks

### R1 — Sprint 18 Never Formally Closes (MEDIUM)

`S11-HUD-TIMER-EYEBALL-VISUAL-001` has been carried Sprint 13 → 18. If the human visual
sign-off is never performed, the Must Have set (3 of 4 rows done) technically leaves Sprint 18
with an unclosed Must Have. Mitigation: user can authorize a formal "deferred carry with
conditions" closure that waives the visual-only row, consistent with prior sprint close-with-
conditions precedents.

### R2 — Polish → Release Gate Stays Indefinitely Blocked (HIGH)

No retry is authorized and no sprint is defined as a gate-check sprint. Without a user-driven
decision to retry, the project cannot advance past Polish. The gate-check failure (PROMPT 761)
date was 2026-05-12; the failure rationale is in `production/gate-checks/gate-polish-release-2026-05-12.md`.
The longer this stays unrevisited, the more code drift builds up between the failure evidence
and the current codebase state.

### R3 — FRAG-02 / FRAG-03 Block Sprint 19 Bot QA (HIGH)

Both `BOT-SOAK-ENTRYPOINT-001` and `AUTOPLAY-VS-BOT-QA-001` depend on reliable phase
detection (R-02) and verified Add Bot button coordinates (R-04). Without C1 + C2, Sprint 19
bot QA recipes will hit time-based wait failures in any session that diverges from the
hardcoded timing assumptions.

### R4 — Carried Conditions Staleness (MEDIUM)

`S8-QA-001-W1`, `TQ-S12-C7`, PROMPT 1054, Sprint 12 drag-runtime `cannot-reproduce` have
been carried through 6+ sprints with no concrete discharge path. Each sprint closeout copies
these forward. They are tracked in `production/sprint-status.yaml` conditions_carried_forward
blocks. If the Release gate is retried, these will be reviewed; none of them currently blocks
Sprint 18 closure.

### R5 — `reapply` Chain Accumulation (LOW)

PROMPT 2005 → 2016 was entirely reapply operations for branches that drifted behind main.
This pattern is sustainable but generates report churn. The underlying cause is that worker
branches are authored at one main tip but land on a later tip, making them NOT strict-FF.
The current merge policy (strict-FF only) prevents history rewriting but requires reapply
overhead. No action needed for game completion; noted for process awareness.

---

## 8. Evidence Cited

| File | What It Confirmed |
|---|---|
| `production/sprint-status.yaml` (line 1 goal block, lines 8232–9162) | Sprint 18 active set; all 8 story-done closures (PROMPT 1337/1331/1357/1712/1713/1716/1717/1718); 4 open rows per `rows_not_closed_by_prompt_1718` |
| `production/sprints/sprint-18.md` (§1–§4-bis) | Sprint goal, capacity, active set, dropped row, Section 4-bis bot epic note |
| `reports/PROMPT-2000-autoplay-recipe-visible-target-coverage-map-report-refresh-after-1994.md` | FRAG-01..05 status (unchanged from 1995 baseline); R-01..R-04 open status |
| `reports/PROMPT-2007-autoplay-recipe-visible-target-coverage-map-refresh-after-1980.md` | Recovery chain; confirms FRAG register unchanged; PROMPT 1980 series preserved |
| `git log --oneline -20 origin/main` | PROMPT 2005 → 2016 chain confirmed as reapply/report-only; no new feature commits since PROMPT 1991 (hand fan readability Stage3-D) |
| `production/stage.txt` | Confirmed `Polish`; NOT modified by any PROMPT since Sprint 18 activation |

---

2017: GAME-COMPLETION-NEXT-WAVE-MAP-AFTER-2016: SHIPPED
