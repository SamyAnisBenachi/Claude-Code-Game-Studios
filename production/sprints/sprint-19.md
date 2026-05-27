# Sprint 19 -- DRAFT (Polish stage; Sprint 18 active)

> **PROMPT 1723 paperwork-only Sprint 19 plan draft (2026-05-27).**
> Source-of-truth at authoring: `origin/main@7f9c605e` (PROMPT 1708
> confirmed main-landed; PROMPT 1711 wave-map source-of-truth).
> Worktree: `.claude/worktrees/prompt-1723-sprint-19-plan-draft`.
> Branch: `worktree-prompt-1723-sprint-19-plan-draft`.
>
> **Status**: `draft -- authored 2026-05-27 by PROMPT 1723`.
> **Sprint 19 is NOT activated by this draft.** Top-level `sprint: 18`,
> `status: active`, `stage: Polish` in
> `production/sprint-status.yaml` are preserved verbatim. Activation is a
> separate explicit prompt (PROMPT 1726 per PROMPT 1711 §6) that flips
> `sprint: 18 -> 19` and `status: active -> active` (new sprint active),
> preserves `stage: Polish`, attaches a `sprint_19_activation:` block, and
> prepends an ACTIVATED banner to this file. PROMPT 1723 does **not** edit
> `production/sprint-status.yaml`, `production/stage.txt`,
> `production/session-state/*`, `production/qa/**`,
> `production/gate-checks/**`, `production/epics/**`, source code, tests,
> Cargo files, CI, or any sprint file other than this one.
>
> **Stage**: `Polish` (UNCHANGED). PROMPT 761 `Polish->Release` gate-check
> `FAIL` preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry**
> is in scope for Sprint 19 and MUST NOT be attempted by activation.
> Sprint 19 is **NOT** a `Polish->Release` sprint.
>
> **Dates (provisional)**: 2026-09-24 -> 2026-10-07 (10 workdays;
> contiguous with the locked Sprint 18 window 2026-09-10 -> 2026-09-23).
> Activation may relock these against the activation HEAD.

---

## 0. Activation Blockers (must clear before Sprint 19 activation)

This draft is authored ahead of the activation gate. Activation MUST NOT
proceed until all of the following are resolved:

1. **Sprint 18 close-out** -- All Sprint 18 Must Have rows must reach
   `Done` or explicit `carry-forward` disposition. Sprint 19 activation
   is gated on `PROMPT 1725 SPRINT-18-CLOSE-OUT` completing (per PROMPT
   1711 §2 Wave C-1). No Sprint 19 activation before Sprint 18 is
   formally closed or closed-with-conditions.

2. **Sprint 19 plan draft landed (this file)** -- PROMPT 1723 produces
   it on a worker branch; the orchestrator integrates separately. This
   file MUST be on `origin/main` before activation.

3. **Sprint 19 QA plan absent and out of scope here** --
   `production/qa/qa-plan-sprint-19.md` is absent on `origin/main` and
   is **NOT** authored by PROMPT 1723. Authoring the QA plan is a
   separate, sequenced post-activation prompt (`/qa-plan sprint-19`),
   mirroring the Sprint 17 PROMPT 1100 / Sprint 18 pattern. No
   `/dev-story` may run against Sprint 19 rows before the QA plan exists
   on `origin/main`.

4. **A-2 bot evidence gap fixes (P0/P1) landed** -- PROMPT 1711 §A-2
   identifies four evidence gaps (P0-A placement coords, P0-B
   final_state winner/reason, P0-C last_decision_at_ms frozen, P1-A
   legal_action_count null) that must land on `origin/main` before
   `BOT-SOAK-ENTRYPOINT-001` can have a valid `/story-done`. Activation
   of Sprint 19 may proceed before these land, but those rows cannot
   close without them.

5. **Human gates B-1/B-2 for BOT-ROOM-PARTICIPANT-001** -- AC7 evidence
   requires a live two-client full test (B-1) and a live human-vs-bot
   smoke run (B-2) before `/story-done` can close
   `BOT-ROOM-PARTICIPANT-001`. Client rebuild from `7f9c605e` is
   required before either gate (run `Update-LatestMain.ps1`).

---

## 1. Goal

Sprint 19 is a focused **Polish-stage bot-and-autoplay epic closure +
Krosmaga UI cohesion sprint**: close the bot + autoplay QA-automation
substrate that has been landing on `origin/main` across PROMPT 1430 –
1682, discharge the outstanding Sprint 18 carry conditions (Timer
Eyeball Visual human gate, Interaction State Migration Wave-2), and
advance the Krosmaga rendering fidelity wave for hand-fan layout,
board play-area targeting feedback, card rendering fidelity, and
result/mulligan overlay chrome.

Sprint 19 is **NOT** a Polish->Release activation and makes **NO**
release / RC / full-game / accessibility-Standard-tier / playtest
validation claims (see Section 7 below).

---

## 2. Capacity and Active Set

**Provisional capacity**: ~7.5d (10 workdays minus QA / reserve buffer).
**Total active scope**: ~5.55d before any carry adjustments. Activation
may trim the Should / Nice tiers if capacity is reassessed.

### 2.1 Must Have (2 rows; ~0.5d)

| ID | Story file | Est. | Source / Notes |
|---|---|---|---|
| `BOT-ROOM-PARTICIPANT-001` | `production/epics/bot-and-autoplay/story-001-bot-room-participant.md` | 0.25d | Implementation on `origin/main` across PROMPTs 1430/1439/1531/1582/1583/1602. **Paperwork-only**: requires `/story-readiness` + `/story-done`. Gated on Sprint 19 activation (per story-001 ledger placeholder) + B-2 live human-vs-bot smoke pass (AC7 evidence). |
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` | 0.25d | Sprint 13 -> 14 -> 15 -> 16 -> 17 -> 18 -> 19 carry. Human-operator-blocked; no LLM `/story-done` authorised. Carried from Sprint 18 close-out. |

### 2.2 Should Have (5 rows; ~2.75d)

| ID | Story file | Est. | Source / Notes |
|---|---|---|---|
| `BOT-SOAK-ENTRYPOINT-001` | `production/epics/bot-and-autoplay/story-002-bot-vs-bot-soak-entrypoint.md` | 0.75d | Worker shipped via PROMPT 1603; integration PROMPT 1607 main-land status as of draft tip. **Prereq**: A-2 P0 evidence gap fixes (PROMPTs 1719–1722) landed on `origin/main`. Requires fresh bounded soak run with enhanced evidence before `/story-done`. |
| `AUTOPLAY-RECIPE-LIBRARY-001` | `production/epics/bot-and-autoplay/story-003-autoplay-recipe-library-v1.md` | 0.5d | Bootstrap on `origin/main`; recipe library extension is the next slice. **Prereq**: B-3 human live full-game run via `Run-AutoplaySmoke.ps1 -Recipe full-game` (AC2 evidence). Gated on `Start-AutoplayVsBot.ps1` harness running. |
| `BOT-DEBUG-OVERLAY-001` | `production/epics/bot-and-autoplay/story-005-bot-debug-overlay.md` | 0.75d | Data contract via PROMPT 1604. **Prereq**: B-4 human AC5 toggle-policy ruling (PROMPT 1670 options doc) + remaining `/dev-story` impl + verify pass. |
| `S19-KROSMAGA-HAND-FAN-LAYOUT-001` | `production/epics/hand-ui/story-024-krosmaga-hand-fan-layout.md` | 0.5d | Unimplemented at draft tip. **Gated on `S18-UI-PLAY-AREA-CONTAINER-001` reaching Done** (A-1 row 1, PROMPT 1712). Sequencing-sensitive: do not initiate `/dev-story` before gate clears. |
| `S19-BOARD-PLAYAREA-TARGETING-001` | `production/epics/board-rendering/story-014-krosmaga-playarea-targeting-feedback.md` | 0.25d | Implementation landed on `origin/main` per PROMPT 1390. **Gated on `S18-UI-PLAY-AREA-CONTAINER-001` Done** (same gate as hand-fan). Activation-time `/story-readiness` then `/story-done` paperwork only once gate clears. |

### 2.3 Nice to Have (6 rows; ~2.3d)

| ID | Story file | Est. | Source / Notes |
|---|---|---|---|
| `AUTOPLAY-VS-BOT-QA-001` | `production/epics/bot-and-autoplay/story-004-autoplay-vs-bot-qa-flow.md` | 0.5d | **Composite gate**: `BOT-ROOM-PARTICIPANT-001` Done + `BOT-SOAK-ENTRYPOINT-001` Done + `AUTOPLAY-RECIPE-LIBRARY-001` Done + B-1 live two-client composite run. Activation-gated on C-6 per PROMPT 1711 wave chain. |
| `S19-KROSMAGA-CARD-RENDERING-FIDELITY-001` | `production/epics/ui-clean-pass/story-028-krosmaga-card-rendering-fidelity-hover-glossary.md` | 0.5d | **Gated on `S18-UI-CARD-ART-AND-LABEL-STRIP-001` Done** (A-1 row 4, PROMPT 1715). Do not initiate before gate clears. |
| `S19-KROSMAGA-RESULT-MULLIGAN-CHROME-001` | `production/epics/presentation-layer/story-007-krosmaga-result-mulligan-overlay-chrome.md` | 0.5d | Sprint 19 candidate per PROMPT 1711 §A-3. No upstream story gate identified; may begin after Sprint 19 activation + QA plan. |
| `PAW-DEV-PROXY-PACK-BOUNDARY-001` | `production/epics/presentation-asset-wiring/story-007-krosmaga-dev-proxy-pack-boundary.md` | 0.25d | Implementation on `origin/main` per PROMPT 1369. **Paperwork-only**: `/story-readiness` + `/story-done`. |
| `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` | `production/epics/ui-clean-pass/story-025-ui-interaction-state-migration-wave-2.md` | 0.35d | Unimplemented Nice-to-Have carry from Sprint 18 active set. `/dev-story` targeted by PROMPT 1724 (A-4 wave). **Conditional**: if PROMPT 1724 ships impl before Sprint 19 activation, this becomes paperwork-only; if PROMPT 1724 is blocked, row scope carries forward. Do not initiate duplicate `/dev-story` while PROMPT 1724 is active. |
| `S19-UI-HAND-RESERVE-STRIP-CLEANUP-001` | `production/epics/hand-ui/story-027-hand-reserve-strip-cleanup.md` | 0.2d | Sprint 18 dropped-row candidate (story file absent at Sprint 18 activation; dropped per Section 2.3 constraint; story-027 now exists on `origin/main`). Preferred follow-up for `S17-UI-HUD-OPP-MANA-CLEANUP-001` AC3 hand-reserve microbadge parent-row paperwork gap. May trim at activation if capacity is tight. |

---

## 3. Carried Conditions

The following conditions are carried forward from Sprint 18 verbatim and
**MUST NOT** be claimed closed by Sprint 19 activation or by any Sprint
19 row outside an explicit, scoped repair prompt:

- `S8-QA-001-W1` -- remains OPEN.
- `QA-COND-0005` -- Standard-tier accessibility remains accepted-risk / friend-game scope only.
- `QA-COND-0006` -- playtest / fun-hypothesis validation remains accepted-risk / deferred.
- `PAW-TD-*-a` -- placeholder-art accepted-risk remains in place.
- `TQ-S12-C1..C7` -- preserved; `TQ-S12-C7` is not closed.
- PROMPT 683-era runtime divergence + Sprint 12 story 019 `cannot-reproduce` -- preserved.
- PROMPT 1054 P1 UI snapshot visual retest -- remains `BLOCKED-HUMAN-OPERATOR`.
- PROMPT 761 `Polish->Release` gate-check -- remains FAIL; no retry in Sprint 19.
- `S17-UI-HUD-OPP-MANA-CLEANUP-001` -- parent-row paperwork gap carried.
  Preferred discharge path is `S19-UI-HAND-RESERVE-STRIP-CLEANUP-001`
  (Section 2.3). Sprint 19 does NOT silently close this row.
- All prior Sprint 10 through Sprint 18 `closed-with-conditions` dispositions remain preserved.

---

## 4. Human-Gated Items

These cannot be automated. No batch, no headless path. Operator must be
at keyboard with client windows open. **Client must be rebuilt from
`7f9c605e` before any B-gate run** (run `Update-LatestMain.ps1`).

| Gate | Story | Prereq | What to run |
|------|-------|--------|-------------|
| **B-1** Live two-client full test | `AUTOPLAY-VS-BOT-QA-001` AC composite | Rebuild client | PROMPT 1699 runbook (L/A/P/R checklists) |
| **B-2** Human-vs-bot live smoke | `BOT-ROOM-PARTICIPANT-001` AC7 | Rebuild client + server running | `Start-AutoplayVsBot.ps1` manual observation |
| **B-3** Autoplay live full-game run | `AUTOPLAY-RECIPE-LIBRARY-001` AC2 | `Run-AutoplaySmoke.ps1 -Recipe full-game`; harness running | Recipe full-game path |
| **B-4** Bot Debug Overlay AC5 ruling | `BOT-DEBUG-OVERLAY-001` AC5 | PROMPT 1670 AC5 options doc | Human chooses toggle-on-or-off policy |
| **B-5** HUD Timer Eyeball Visual | `S11-HUD-TIMER-EYEBALL-VISUAL-001` | Live client required | Direct visual verification of timer eyeball |

**Client rebuild command** (run before B-1, B-2, B-3, or B-5):
```powershell
cd D:\_DEV\Work\Claude-Code-Game-Studios
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1
```

---

## 5. Deferred to Sprint 20+

| Item | Reason | Earliest gate |
|------|--------|---------------|
| `BOT-DISCONNECT-REJOIN-006` | Hard gate on `BOT-ROOM-PARTICIPANT-001` Done | Sprint 20+ |
| Krosmaga Stage 3 full rendering (real assets) | PAW placeholder art; art production work | Art milestone |
| `QA-COND-0005` Standard accessibility | Full accessibility audit required | Human |
| `QA-COND-0006` Playtest fun-hypothesis | Human playtest sessions required | Human |
| `S8-QA-001-W1` Full QA regression sweep | Human operator | Human |
| `PAW-TD-*-a` Placeholder art replacement | Art production | Art milestone |
| Polish → Release gate retry | PROMPT 761 FAIL preserved; human decision + full gate-check rerun | Human decision |
| `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` (if trimmed) | May drop at activation if capacity tight | Sprint 20 |

---

## 6. Activation Sequence (informational; not executed by this draft)

The required serial gates after this draft lands (PROMPT 1711 §2 wave C):

1. Sprint 18 A-1 paperwork sweep (PROMPTs 1712–1718) completes — all 7 rows Done or carry-forward.
2. Sprint 18 close-out prompt (PROMPT 1725) lands on `origin/main` (C-1).
3. Sprint 19 activation prompt (PROMPT 1726) flips `sprint: 18 -> 19`,
   preserves `stage: Polish`, attaches `sprint_19_activation:` block,
   and prepends ACTIVATED banner to this file.
4. `/qa-plan sprint-19` authors `production/qa/qa-plan-sprint-19.md`
   against the Sprint 19 activation tip.
5. Per-row `/story-readiness` reruns against the Sprint 19 activation tip
   for each active row before any `/dev-story` runs.
6. Human gates B-1/B-2 (client rebuild required) to close
   `BOT-ROOM-PARTICIPANT-001` AC7 evidence.
7. `/story-done` for paperwork-only rows:
   `BOT-ROOM-PARTICIPANT-001`, `S19-BOARD-PLAYAREA-TARGETING-001`,
   `PAW-DEV-PROXY-PACK-BOUNDARY-001`.
8. `/dev-story` for unimplemented rows:
   `S19-KROSMAGA-HAND-FAN-LAYOUT-001`, `BOT-DEBUG-OVERLAY-001`
   (after B-4 AC5 ruling), `S19-KROSMAGA-RESULT-MULLIGAN-CHROME-001`.
9. A-2 evidence gap fixes (PROMPTs 1719–1722) confirmed landed, then
   fresh bounded soak for `BOT-SOAK-ENTRYPOINT-001` (C-4 gate).
10. B-3 live full-game autoplay run for `AUTOPLAY-RECIPE-LIBRARY-001` (C-5 gate).
11. Composite closure: `AUTOPLAY-VS-BOT-QA-001` after C-3 + C-4 + C-5 + B-1 (C-6 gate).

---

## 7. Non-Claims (preserved verbatim through Sprint 19 activation)

Sprint 19 activation and any prompt operating under the Sprint 19 plan
MUST preserve all of the following non-claims. PROMPT 1723 makes
**NONE** of these claims:

- NO public release readiness.
- NO release-candidate readiness.
- NO full game completion.
- NO `QA-COND-0005` Standard-tier accessibility advancement.
- NO `QA-COND-0006` playtest / fun-hypothesis validation advancement.
- NO full playable-client manual QA.
- NO `S8-QA-001-W1` closure.
- NO `PAW-TD-*-a` final-art completion.
- NO `Polish->Release` gate-check retry.
- NO stage advance from `Polish`.
- NO LLM closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`.
- NO silent closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent row.
- NO closure of any PROMPT 1022 / PROMPT 1076 / PROMPT 1077 finding
  outside concrete repairs already on `origin/main`.
- NO Sprint 10 through Sprint 18 row reopen.
- NO Sprint 18 close-out reopen, re-author, or silent overwrite.
- NO retroactive closure of any row not implemented on `origin/main`
  at the activation tip.

---

## 8. Files Changed by PROMPT 1723

- `production/sprints/sprint-19.md` (this file; **CREATED**).
- `reports/PROMPT-1723-sprint-19-plan-draft.md` (mandatory final report;
  `reports/` is gitignored; not staged or committed by this commit).

**Files explicitly NOT touched by PROMPT 1723**: `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`,
`Trunk.toml`, `production/stage.txt`, `production/sprint-status.yaml`,
`production/sprints/sprint-1.md` through `sprint-18.md`,
`production/qa/**`, `production/gate-checks/**`,
`production/epics/**`, `production/session-state/**`, `.octogent/`,
`.claude/`. No cargo / trunk / CI command invoked by PROMPT 1723;
Cargo policy: N/A for this paperwork-only draft.

---

## 9. Branch / Push Policy

PROMPT 1723 commits the draft on branch
`worktree-prompt-1723-sprint-19-plan-draft` from base
`origin/main@7f9c605e`. Push target: worker branch only;
**never `main`**. Orchestrator integrates separately. If branch push is
blocked at the remote, the commit is kept locally and the exact
branch/commit is reported.
