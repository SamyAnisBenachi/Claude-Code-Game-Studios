# Sprint 17 -- CLOSED-WITH-CONDITIONS (Polish stage; PROMPT 1279, 2026-05-18)

> **PROMPT 1279 paperwork-only Sprint 17 close-out disposition (2026-05-18)**.
> Source-of-truth at close-out:
> `origin/main@946ca392c94a4988e9c6b4483848233fe6323061`
> (PROMPT 1276 board-rendering message init main-land; includes PROMPT
> 1272 ignored-test baseline `23d1c1b`, PROMPT 1275 shop prepool formula
> `35a95d5`, and PROMPT 1274 UI clean-pass lint `c94514f`).
>
> **Status flip**: `production/sprint-status.yaml` top-level
> `status: active -> closed-with-conditions`. `sprint: 17` and
> `stage: Polish` are preserved. `production/stage.txt` is not touched.
> Sprint 18 is **not** activated.
>
> **Disposition basis**: PROMPT 1278 Team-QA is
> `APPROVED-WITH-CONDITIONS`. Sprint 17 closes with 7 of 9 rows done,
> 1 row still `in_progress` (`S17-UI-HUD-OPP-MANA-CLEANUP-001`; AC3
> source repair on `origin/main@c842668`, but no final `/story-done`
> paperwork), and 1 row still human-operator-blocked
> (`S11-HUD-TIMER-EYEBALL-VISUAL-001`; no LLM `/story-done` authorised).
>
> **Smoke evidence conditions**: the initial PROMPT 1264 smoke failed and
> the late repair chain landed on `origin/main` through PROMPT 1272 /
> 1274 / 1275 / 1276. PROMPT 1278 accepted the prompt-provided PROMPT 1277
> PASS/PASS-WITH-WARNINGS disposition with local rerun artifacts, but no
> durable tracked `reports/PROMPT-1277*` or
> `production/qa/smoke-sprint-17*` artifact exists on `origin/main`.
> That missing durable smoke artifact remains a close-out condition.
> The two PROMPT 1278 smoke warnings are carried forward: the
> `hand_ui_phase_transition_auto_submit_short_circuit` /
> `invalid_submit_state` warning after one staged card at Placement ->
> Resolution, and the `RSM disconnect timer breach: grace window exceeded`
> warning after a later DraftShop disconnect.
>
> **Non-claims preserved**: no release readiness, no RC readiness, no full
> game completion, no broad accessibility completion, no playtest
> validation, no final-art completion, no full playable-client manual QA,
> no two-client GAME_OVER closure, no Polish->Release retry, no stage
> advance, no closure of `S8-QA-001-W1`, `QA-COND-0005`,
> `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, PROMPT 683-era runtime
> divergence, Sprint 12 story 019 cannot-reproduce disposition, or PROMPT
> 1054 P1 UI snapshot visual retest blocked-human state.

---

# Sprint 17 -- ACTIVATED (Polish stage; PROMPT 1099, 2026-05-18)

> **PROMPT 1099 paperwork-only Sprint 17 activation (2026-05-18)**.
> Source-of-truth at activation:
> `origin/main@bc3db291fb2e9b840c986b68ea8899664bba94b6`
> (PROMPT 1097 paperwork-only main integration tip:
> `integrate(s17): merge PROMPT 1095 net-new Sprint 17 story authoring
> batch into main (PROMPT 1097 paperwork-only)`). Strict fast-forward
> descendant of `origin/main@e6a6e11` (PROMPT 1090 Sprint 17 plan draft
> commit base) and of `origin/main@fec13ff` (PROMPT 1088 Sprint 16
> close-out main integration tip).
>
> **Status flip**: `production/sprint-status.yaml` top-level
> `sprint: 16 -> 17`; `status: closed-with-conditions -> active`;
> `stage: Polish` **UNCHANGED**. `production/stage.txt` **NOT touched**
> (remains `Polish`). PROMPT 761 `Polish->Release` gate-check `FAIL`
> preserved verbatim at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO
> retry** in scope for Sprint 17. Sprint 17 is **NOT** a
> `Polish->Release` activation.
>
> **Stage**: `Polish` (UNCHANGED).
>
> **Dates (locked at activation)**: 2026-08-27 -> 2026-09-09 (10
> workdays; same provisional window as PROMPT 1090 draft).
>
> **Five draft conditional Must Have rows DROPPED at PROMPT 1099
> activation** because their repairs already landed on
> `origin/main` between PROMPT 1090 draft commit (`e6a6e11`) and
> PROMPT 1099 activation (`bc3db29`):
>
> - `S17-UI-MODAL-BLACK-SLAB-001` -- AUDIT-1076-01 (P0) discharged
>   via PROMPT 1080 worker (`cbc11b2`) + PROMPT 1083 integration
>   (`e4bbca3`) + PROMPT 1094 stack merge (`a6ecc47`).
> - `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` -- AUDIT-1076-04 (P1) +
>   AUDIT-1076-13 (P2) discharged via PROMPT 1085 worker (`8a67460`)
>   + PROMPT 1094 stack merge (`6b5eb8e`).
> - `S17-UI-PLACEMENT-PERSPECTIVE-001` -- AUDIT-1076-09 (P1 UX) +
>   PROMPT 1079 client residual risk #2 discharged via PROMPT 1086
>   worker (`d87939c`) + PROMPT 1092 integration (`e6a6e11`).
> - `S17-UI-LOBBY-CLASS-ART-CONFIRM-001` -- AUDIT-1076-06 (P1) +
>   AUDIT-1076-07 (P1) discharged via PROMPT 1081 worker (`7f10b42`)
>   + PROMPT 1087 integration (`eec2a91`) + PROMPT 1089 refresh
>   (`d51e246`). Placeholder PNGs only; real-art production
>   deferred to Sprint 18+ under `PAW-TD-*-a` accept-risk.
> - `S17-SERVER-AUCTION-TIMER-001` -- AUDIT-1076-12 (P2) discharged
>   via PROMPT 1091 worker (`4b5d751` `fix(server/auction): anchor
>   LiveBidding deadline to Time<Real> (PROMPT 1091)`) + PROMPT 1091
>   integration (`e3c91d5`).
>
> **Sprint 17 final 9-row active set at PROMPT 1099 activation**
> (2 Must Have + 4 Should Have + 3 Nice to Have; ~2.65d / 7.5d
> available capacity):
>
> **Must Have (2 rows; ~1.0d)**
>
> - `S11-HUD-TIMER-EYEBALL-VISUAL-001` -- 0.25d, conditional
>   human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 -> 17 carry.
>   Story file: `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`.
>   Closure remains gated on human-operator screenshot capture; **no
>   LLM `/story-done` is authorised**.
> - `S17-UI-CARD-DISPLAY-ART-HELPER-001` -- 0.75d, non-conditional.
>   Story file:
>   `production/epics/ui-clean-pass/story-017-card-display-art-helper-bundle.md`.
>   Bundled SOURCE-1077-01 / 02 / 03 / 04 (P0 / P0 / P1 / P1)
>   structural card-display-art helper fix.
>
> **Should Have (4 rows; ~1.25d)**
>
> - `S17-UI-HUD-OPP-MANA-CLEANUP-001` -- 0.5d
>   (`production/epics/hud/story-018-opp-figurine-mana-cleanup.md`;
>   AUDIT-1076-10 / 16 / 17).
> - `S17-UI-CARD-SLOT-INSET-WIRING-001` -- 0.25d
>   (`production/epics/ui-clean-pass/story-018-card-slot-inset-wiring.md`;
>   SOURCE-1077-06).
> - `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` -- 0.25d
>   (`production/epics/ui-clean-pass/story-019-qa-snapshot-marker-split.md`;
>   SOURCE-1077-08 / 09 / 16).
> - `S17-UI-BID-BUTTON-PHASE-RACE-001` -- 0.25d
>   (`production/epics/shop-auction-ui/story-019-bid-button-phase-race.md`;
>   SOURCE-1077-10).
>
> **Nice to Have (3 rows; ~0.4d)**
>
> - `S17-OPS-VULKAN-VALIDATION-GATING-001` -- 0.15d
>   (`production/epics/devops/story-007-vulkan-validation-gating.md`;
>   AUDIT-1076-18).
> - `S17-SERVER-START-OF-TURN-DEBUG-001` -- 0.1d
>   (`production/epics/server/story-003-start-of-turn-debug-downgrade.md`;
>   AUDIT-1076-15).
> - `S17-UI-HAND-B0004-CLEANUP-001` -- 0.15d
>   (`production/epics/hand-ui/story-021-hand-fan-root-b0004-hierarchy.md`;
>   AUDIT-1076-14).
>
> **All 9 story files exist on `origin/main` at activation HEAD
> `bc3db29`** (PROMPT 1097 integrated the 8 net-new Sprint 17
> story files authored by PROMPT 1095; story-014 HUD timer file
> unchanged on `origin/main` since Sprint 13).
>
> **Next launchable prompt**: `PROMPT 1100 -- /qa-plan sprint-17`.
> The Sprint 17 QA plan
> (`production/qa/qa-plan-sprint-17.md`) MUST be authored via
> `/qa-plan sprint-17` **after PROMPT 1099 activation lands on
> `origin/main`** and **before** any `/dev-story` runs against
> Sprint 17 stories. Per-row `/story-readiness` reruns against
> Sprint 17 activation HEAD `bc3db29` are also required before
> `/dev-story` for each of the 8 net-new Sprint 17 active rows
> (the S11-HUD-TIMER-EYEBALL-VISUAL-001 row was already READY since
> Sprint 13 because the story file is unchanged on `origin/main`).
>
> **Non-claims preserved verbatim at PROMPT 1099 activation** (full
> list in the PROMPT 1090 DRAFT banner below + the
> `sprint_17_activation.explicitly_not_claimed` block in
> `production/sprint-status.yaml`): NO public release readiness, NO
> RC readiness, NO full game completion, NO `QA-COND-0005`
> Standard-tier accessibility advancement, NO `QA-COND-0006`
> playtest validation advancement, NO full playable-client manual
> QA, NO `S8-QA-001-W1` closure (remains OPEN; Sprint 13 story 017
> AC12 forbid-auto-closure preserved through Sprint 13 / 14 / 15 /
> 16 / 17), NO `PAW-TD-*-a` final-art completion, NO
> `Polish->Release` gate-check retry (PROMPT 761 `FAIL` preserved),
> NO stage advance from Polish to Release, NO underlying
> drag-runtime bug fix (Sprint 12 story 019 `cannot-reproduce`
> preserved), NO `TQ-S12-C7` closure, NO LLM closure of
> `S11-HUD-TIMER-EYEBALL-VISUAL-001`, NO closure of PROMPT 1054 P1
> UI snapshot visual retest by an LLM (`BLOCKED-HUMAN-OPERATOR`
> preserved), NO closure of any PROMPT 1076 / PROMPT 1077 finding
> for which a concrete repair is not on `origin/main` at
> activation, NO closure of any of the 24 PROMPT 1022 QA snapshot
> audit findings, NO Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row
> reopen, NO Sprint 16 close-out reopen or re-author.
>
> **Files changed by PROMPT 1099**:
>
> - `production/sprint-status.yaml` (top-level `sprint: 16 -> 17`;
>   `status: closed-with-conditions -> active`; `stage: Polish`
>   PRESERVED; `goal` / `start` / `end` / `scope` / `generated` /
>   `updated` refreshed; `stories:` block replaced with 9-row
>   active set; `next_sprint_17_draft:` block at EOF replaced with
>   `sprint_17_activation:` block; all prior
>   `sprint_N_closeout:` / `sprint_N_activation:` /
>   `sprint_N_story_done:` blocks preserved verbatim above
>   `sprint_17_activation:`).
> - `production/sprints/sprint-17.md` (this PROMPT 1099 ACTIVATED
>   banner prepended above the PROMPT 1090 DRAFT banner; plan body
>   NOT rewritten).
> - `production/session-state/active.md` (PROMPT 1099 banner
>   prepended above PROMPT 1090 banner).
> - `production/session-state/codex-orchestrator-state.md` (PROMPT
>   1099 paragraph prepended above PROMPT 1090 paragraph).
> - `reports/PROMPT-1099-sprint-17-activation.md` (mandatory final
>   report; `reports/` is gitignored; not staged or committed).
>
> **Files explicitly NOT touched by PROMPT 1099**: `client/`,
> `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`,
> `.cargo/`, `.github/`, `Trunk.toml`, `production/stage.txt`,
> `production/qa/**`, `production/gate-checks/**`,
> `production/epics/**`, `production/sprints/sprint-16.md` /
> `sprint-15.md` / `sprint-14.md` / `sprint-13.md` / `sprint-12.md`
> / `sprint-11.md` / `sprint-10.md`,
> `production/sprint-status.yaml` `sprint_16_closeout:` /
> `sprint_16_activation:` / `sprint_16_story_done:` and all prior
> `sprint_N_*` blocks, `.octogent/`, `.claude/settings.json`,
> `.claude/scheduled_tasks.lock`. No cargo / trunk / CI command
> invoked. **Cargo policy: N/A** for this paperwork-only activation.
>
> **Branch / push**: PROMPT 1099 commits the activation paperwork
> on branch `activate/sprint-17-1099` from base
> `origin/main@bc3db291fb2e9b840c986b68ea8899664bba94b6`. Push
> target: worker branch only; never `main`. Orchestrator integrates
> separately.

---

# Sprint 17 -- DRAFT (Polish stage; Sprint 16 closed-with-conditions)

> **PROMPT 1090 paperwork-only Sprint 17 plan draft (2026-05-18)**.
> Source-of-truth at authoring: `origin/main@fec13ffc3723d9d68afdda4b6e4bf62af5d6da2a`
> (PROMPT 1088 Sprint 16 close-out main integration tip:
> `integrate(s16): merge Sprint 16 close-out paperwork into main (PROMPT 1088)`).
> Worktree: `D:/_DEV/claude-code-game-studios-worktrees/sprint-17-plan-draft-1090`.
> Branch: `sprint-plan/sprint-17-draft-1090`.
>
> **Status**: `draft -- authored 2026-05-18 by PROMPT 1090`. **Sprint 17
> is NOT activated by this draft.** Top-level `sprint: 16`, `status:
> closed-with-conditions`, `stage: Polish` preserved verbatim in
> `production/sprint-status.yaml`. Activation of Sprint 17 is a separate
> explicit prompt that mirrors the PROMPT 826 / PROMPT 897 / PROMPT 997
> / PROMPT 1064 pattern: it flips `sprint: 16 -> 17` and `status:
> closed-with-conditions -> active`, replaces the `stories:` block with
> the Sprint 17 active set, replaces the `next_sprint_17_draft:` block
> at EOF with a `sprint_17_activation:` block, and adds an ACTIVATED
> banner to this file.
>
> **Stage**: `Polish` (UNCHANGED). `production/stage.txt` NOT modified
> by this draft and MUST NOT be modified by activation. PROMPT 761
> `Polish->Release` gate-check `FAIL` evidence preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO
> retry** is in scope for Sprint 17 and MUST NOT be attempted by
> activation. Sprint 17 is **NOT** a `Polish->Release` sprint.
>
> **Provisional start / end (locked at activation)**: 2026-08-27 ->
> 2026-09-09 (10 workdays). Continuous follow-on to Sprint 16
> (2026-08-13 -> 2026-08-26).
>
> **Sprint 16 disposition at draft time**: `closed-with-conditions` per
> PROMPT 1082 close-out + PROMPT 1088 main integration. 3 of 4 active
> rows closed (Must 0/1 + Should 1/1 + Nice 2/2). Sole open row
> `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have, story 014, 0.25d) is a
> human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 carry; closure
> remains gated on real human-operator screenshot capture across
> `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases per
> the story AC matrix. No LLM `/story-done` is authorised. Allowed to
> carry forward to Sprint 17 if no human-operator slot opens.
>
> **In-flight repair tracks at draft time (not yet on `origin/main`)**:
>
> - **PROMPT 1080 / 1083** -- DraftInitial modal black-slab repair.
>   Worker `cbc11b2` on `origin/work/client-phase-modal-black-slab-1080`;
>   integration `e4bbca3` on `origin/integrate/client-phase-modal-black-slab-1083`.
>   PROMPT 1083 integration verified `PASS` (10 focused test bins).
>   NOT yet integrated into `origin/main` at draft time.
> - **PROMPT 1085** -- client shop / auction surface paint + intent
>   repair. Worker branch only `origin/work/client-shop-auction-surface-paint-1085`;
>   no integration prompt landed at draft time.
> - **PROMPT 1086** -- client placement perspective + invalid-drop
>   feedback repair. Worker branch only
>   `origin/work/client-placement-perspective-submit-1086`; no
>   integration prompt landed at draft time.
> - **PROMPT 1087 / 1089** -- lobby class art + Confirm button
>   integration. PROMPT 1087 integration `eec2a91` on
>   `origin/integrate/lobby-class-art-confirm-1087` (49/49 PASS, 5
>   files M3/A2). PROMPT 1089 main push attempt blocked
>   (`NEEDS-REFRESH`: integration ancestor of `origin/main@fec13ff` is
>   `false` because of PROMPT 1082 + PROMPT 1088 close-out paperwork
>   commits; non-FF would lose close-out paperwork). NOT yet
>   integrated into `origin/main` at draft time. **Treated as
>   candidate / conditional**; Sprint 17 plan does NOT assume any of
>   PROMPT 1080 / 1083 / 1085 / 1086 / 1087 is on `origin/main` at
>   activation time.
> - **PROMPT 1079 / 1084** -- server placement buffer log + spawn
>   integration. ALREADY on `origin/main` (`c5b0d04` worker + `dd7f5d3`
>   integration). Sprint 17 plan treats AUDIT-1076-02 / AUDIT-1076-03
>   (server-side placement buffer race + spawn-loop counter
>   mismatch) as **partially landed**; remaining client-side
>   placement perspective + invalid-drop feedback gap (AUDIT-1076-09)
>   is still open and is the Sprint 17 candidate.
>
> **Sprint 14 / Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 + Sprint
> 15 + Sprint 16 closeouts** preserved unchanged. `S8-QA-001-W1`
> OPEN, `QA-COND-0005` + `QA-COND-0006` accepted-risk, `PAW-TD-*-a`
> accept-risk across PAW-002..PAW-006, `TQ-S12-C1..C7` preserved
> verbatim (`TQ-S12-C7` NOT closed), PROMPT 683-era runtime divergence
> question preserved (no third same-scope retest per `TQ-S12-C2`),
> Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed
> (`cannot-reproduce` preserved).
>
> **Sprint 17 explicitly does NOT claim**: public release readiness,
> release-candidate readiness, full game completion, broad / Standard-
> tier accessibility completion (`QA-COND-0005` accepted-risk),
> playtest / fun-hypothesis validation (`QA-COND-0006` accepted-risk),
> full playable-client manual QA, two-client `GAME_OVER` closure
> (`S8-QA-001-W1` remains OPEN), final-art / asset-production
> completion (`PAW-TD-*-a` accepted-risk preserved across
> PAW-002..PAW-006), `Polish->Release` gate-check retry (PROMPT 761
> `FAIL` preserved), stage advance from Polish to Release, underlying
> drag-runtime bug fix (Sprint 12 story 019 `cannot-reproduce`
> preserved, NOT bug-fixed), Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10
> row reopen, `S8-QA-001-W1` closure, `TQ-S12-C7` closure, closure of
> the Sprint 16 `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-
> blocked carry by an LLM, closure of any PROMPT 1076 or PROMPT 1077
> audit finding for which a concrete repair is not already on
> `origin/main` at activation time, closure of any of the 24 PROMPT
> 1022 QA snapshot audit findings, or full UI clean-pass repair
> beyond the candidate rows below.
>
> **Late-breaking update at PROMPT 1090 commit time (2026-05-18)**:
> Between draft authoring and commit, `origin/main` advanced from
> `fec13ff` to **`e6a6e11b7c3359e076dd1e3c71d47015fa1cf739`** (5
> additional post-Sprint 16-close-out repair commits: `7f10b42`
> PROMPT 1081 lobby class art worker; `eec2a91` PROMPT 1087 lobby
> integration; `d87939c` PROMPT 1086 client placement perspective
> fix; `d51e246` PROMPT 1089 refresh merge; `e6a6e11` PROMPT 1092
> client placement perspective + submit feedback integration). The
> draft branch was rebased onto `e6a6e11` before commit. The
> conditional-row mechanism in this draft accommodates exactly
> this scenario: a row is pulled forward as candidate Must Have
> only if its in-flight repair is NOT on `origin/main` at
> activation. Factual update at PROMPT 1090 commit time:
>
> - **AUDIT-1076-09 client placement perspective + invalid-drop
>   feedback (PROMPT 1086)** is **NOW on `origin/main`** via
>   PROMPT 1092 integration (`e6a6e11`) + worker fix (`d87939c`).
>   The conditional Must Have row `S17-UI-PLACEMENT-PERSPECTIVE-001`
>   (1.0d) is **pre-dropped at PROMPT 1090 commit time**; the
>   Sprint 17 activation prompt (PROMPT 1093) should drop it from
>   the active set.
> - **AUDIT-1076-06 lobby class art + AUDIT-1076-07 Confirm-class
>   button (PROMPT 1087 / 1089)** are **NOW on `origin/main`** via
>   PROMPT 1087 integration (`eec2a91`) + PROMPT 1089 refresh merge
>   (`d51e246`) + PROMPT 1081 worker fix (`7f10b42`). The
>   conditional Must Have row `S17-UI-LOBBY-CLASS-ART-CONFIRM-001`
>   (0.25d-1.5d) is **pre-dropped at PROMPT 1090 commit time**;
>   placeholder PNGs only -- real-art production remains deferred
>   to Sprint 18+ under `PAW-TD-*-a` accept-risk.
>
> **Remaining conditional Must Have rows at PROMPT 1090 commit
> time**: `S17-UI-MODAL-BLACK-SLAB-001` (PROMPT 1080 worker
> `cbc11b2` + PROMPT 1083 integration `e4bbca3` **NOT on
> `origin/main`** at `e6a6e11`) and
> `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (PROMPT 1085 worker
> `origin/work/client-shop-auction-surface-paint-1085` only; **NOT
> on `origin/main`** at `e6a6e11`; depends on modal repair landing
> first).
>
> **Sprint 17 Must Have scope at PROMPT 1090 commit time**:
> ~2.75d-3.5d (`S11-HUD-TIMER-EYEBALL-VISUAL-001` 0.25d
> conditional human-operator-blocked + `S17-UI-MODAL-BLACK-SLAB-001`
> 0.25d-1.0d conditional + `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001`
> 1.5d conditional + `S17-UI-CARD-DISPLAY-ART-HELPER-001` 0.75d
> non-conditional) -- well within Sprint 17 7.5d available
> capacity, with comfortable room for the Should Have + Nice to
> Have list.
>
> **Next launchable prompts at PROMPT 1090 commit time**: PROMPT
> 1091c (client placement perspective integration) and PROMPT
> 1091d (lobby class art main-push refresh) in the original list
> are **SUPERSEDED** by the landed PROMPT 1086 + PROMPT 1087 / 1089
> commits and may be dropped from the launch sequence. PROMPT
> 1091a (modal main-push refresh) and PROMPT 1091b (shop / auction
> surface paint integration) remain the priority next-launchable
> prompts. Story authoring for `S17-UI-CARD-DISPLAY-ART-HELPER-001`
> Must Have + the Should Have + Nice to Have rows remains PROMPT
> 1092 (the illustrative prompt body in §"Required Sprint 17
> Story Docs" below still applies, with
> `S17-UI-PLACEMENT-PERSPECTIVE-001` +
> `S17-UI-LOBBY-CLASS-ART-CONFIRM-001` removed from the list).
>
> The `source_of_truth_at_authoring:` field in this banner and in
> `production/sprint-status.yaml` `next_sprint_17_draft:` is
> preserved verbatim at `fec13ff` (the tip at which the audit
> content was synthesised); a separate `source_of_truth_at_commit:`
> field records `e6a6e11` for traceability. No row revisions to
> the Sprint 17 plan body are required by this late-breaking
> update; the conditional-row mechanism handles the actual row
> drops at PROMPT 1093 activation.
>
> **PROMPT 1090 paperwork-only draft scope**: NO `/dev-story`, NO
> `/story-readiness`, NO `/story-done`, NO `/smoke-check`, NO
> `/team-qa`, NO `/gate-check`, NO `/release-check`, NO `/qa-plan`,
> NO Sprint 17 activation, NO Sprint 16 row reopen, NO
> `production/qa/qa-plan-sprint-17.md` authored, NO stage advance, NO
> implementation, NO CI run, NO `cargo` / `trunk` invocation, NO
> touch of `client/` / `server/` / `shared/` / `tests/` / `Cargo.toml`
> / `Cargo.lock` / `.cargo/` / `.github/`. Files allowed: this file
> (NEW), `production/sprint-status.yaml` (`next_sprint_17_draft:`
> block appended at EOF only; top-level `sprint: 16 / status:
> closed-with-conditions / stage: Polish` preserved verbatim; all
> prior closeout / activation / story-done blocks NOT modified),
> `production/session-state/active.md` (PROMPT 1090 banner
> prepended), `production/session-state/codex-orchestrator-state.md`
> (PROMPT 1090 section prepended).

---

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`.
  Sprint 17 does NOT advance stage. Sprint 17 is NOT a
  `Polish->Release` sprint.
- Sprint 16 is `closed-with-conditions` per PROMPT 1082 close-out +
  PROMPT 1088 main integration. The PROMPT 1075 Sprint 16 smoke
  (`PASS-WITH-WARNINGS`) and PROMPT 1078 Sprint 16 Team-QA
  (`APPROVED-WITH-CONDITIONS`) evidence files live on
  `origin/qa/sprint-16-smoke-check-1075` / `origin/qa/sprint-16-team-qa-1078`
  branches and were NOT integrated under `production/qa/` on
  `origin/main` by PROMPT 1082 / 1088. Sprint 17 does NOT re-author
  or re-integrate Sprint 16 QA evidence files.
- Sprint 17 is a **focused runtime / UI repair sprint** prioritising
  runtime-playability blockers and visible UX fixes ahead of broad
  UI architecture debt. The PROMPT 1076 latest user-test log /
  snapshot deep audit (18 findings: 3 P0 + 9 P1 + 4 P2 + 5 P3) and
  PROMPT 1077 UI / state source consistency deep audit (16
  findings: 2 P0 + 4 P1 + 9 P2 + 1 P3) are the canonical inputs.
- The PROMPT 1076 P0 server-side findings (AUDIT-1076-02 placement
  buffer race + AUDIT-1076-03 spawn-loop counter mismatch) are
  **partially landed** on `origin/main` via PROMPT 1079 (`c5b0d04`
  `fix(server-placement): log rejections, fix LaneWide spawn count,
  filter reveal`) + PROMPT 1084 (`dd7f5d3` integration). The
  client-side placement perspective + invalid-drop feedback gap
  (AUDIT-1076-09) is **NOT** covered by 1079 / 1084 and remains a
  Sprint 17 candidate.
- The PROMPT 1076 P0 / P1 client-side findings (AUDIT-1076-01 modal
  black slab + AUDIT-1076-04 shop / auction surfaces never paint +
  AUDIT-1076-06 lobby class art + AUDIT-1076-07 lobby confirm
  button) are addressed by in-flight repair tracks (PROMPT 1080 /
  1083 / 1085 / 1087) that are NOT yet on `origin/main` at draft
  time. Sprint 17 plan treats each as a **conditional Must Have
  candidate**: pulled forward as a Sprint 17 Must Have row only if
  the corresponding repair is NOT on `origin/main` at activation
  time. If the repair lands before activation, the row is dropped
  from Sprint 17 (Must Have shrinks) and the repair is treated as
  Sprint 16 -> Sprint 17 spill-over already discharged.
- The PROMPT 1077 P0 structural findings (SOURCE-1077-01 slot-well
  chrome lost when card art is missing + SOURCE-1077-02 duplicate
  `apply_card_display_art` definitions) are a **structural Sprint
  17 candidate** owned by `client/src/asset_wiring.rs` +
  `client/src/ui/shop_auction/mod.rs` + `client/src/ui/hand/mod.rs`.
  Single bundled row recommended (dedup + leak fix + existence
  check) because the two findings must land together to avoid
  re-introducing the bug after dedup.
- This draft pulls a **deliberately small** Sprint 17 scope. Per the
  PROMPT 1090 instruction ("Keep Sprint 17 small and executable"),
  the plan covers (1) the Sprint 16 carry (HUD timer eyeball check,
  conditional), (2) up to 4 conditional Must Have runtime / UI
  repair rows that are NOT yet on `origin/main` at activation, (3)
  one Must Have card-display-art helper dedup + leak fix +
  existence-check row (PROMPT 1077 P0 bundle), and (4) up to 5
  Should Have rows for visible UX cleanup and structural debt
  reduction. Heavier candidates (full per-surface card-slot
  primitive migration, real-art production for lobby portraits /
  player slot / room code chip / board sprites, the 24 PROMPT 1022
  audit findings, the Sprint 11/12/13 server hardening backlog,
  long-tail PROMPT 1076 P3 rows) are deliberately **deferred to
  Sprint 18+ backlog**, not promoted into Sprint 17.
- Sequencing is governed by the canonical reconciliation roadmap at
  `docs/ux/ui-clean-pass-roadmap.md` (PROMPT 838) and by the Sprint
  16 draft / activation / close-out evidence (PROMPT 1024 plan /
  PROMPT 1064 activation / PROMPT 1082 close-out / PROMPT 1088 main
  integration). The PROMPT 1076 and PROMPT 1077 audit findings are
  **new inputs** beyond the canonical roadmap; sequencing treats
  the P0 runtime-playability blockers as the top priority and the
  PROMPT 1077 P0 structural fix as a sibling because it gates clean
  card art across every Sprint 17 surface.
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 17 QA plan exists at draft time. A Sprint 17 QA plan
  (`production/qa/qa-plan-sprint-17.md`) MUST be authored via
  `/qa-plan sprint-17` **after** Sprint 17 activation **and after**
  each Sprint 17 story file passes `/story-readiness` against
  activation HEAD. No `/dev-story` is authorised before the QA plan
  exists. PROMPT 1090 does NOT author the QA plan.
- Sprint 17 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad /
  Standard-tier accessibility completion (`QA-COND-0005`), playtest /
  fun-hypothesis validation (`QA-COND-0006`), full playable-client
  manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), final-
  art / asset-production completion (`PAW-TD-*-a`),
  `Polish->Release` gate-check retry, stage advance from Polish to
  Release, or underlying drag-runtime bug fix (Sprint 12 story 019
  remains `closed-with-conditions / cannot-reproduce`). None of
  these can be added to Sprint 17 by activation; each requires its
  own scope and gate evidence.

## Entry Conditions (must be true at activation)

- `production/sprint-status.yaml` top-level reads `sprint: 16`,
  `status: "closed-with-conditions"`, `stage: Polish`.
- `production/stage.txt` reads `Polish` (UNCHANGED).
- PROMPT 761 `Polish->Release` gate-check `FAIL` evidence preserved
  at `production/gate-checks/gate-polish-release-2026-05-12.md`.
- Sprint 16 disposition `closed-with-conditions` per PROMPT 1082 +
  PROMPT 1088 preserved unchanged. Sprint 15 / Sprint 14 / Sprint
  13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts preserved
  unchanged.
- `S8-QA-001-W1` OPEN. `QA-COND-0005` + `QA-COND-0006` accepted-
  risk. `PAW-TD-*-a` accept-risk across PAW-002..PAW-006.
  `TQ-S12-C1..C7` preserved verbatim (`TQ-S12-C7` NOT closed).
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` disposition at Sprint 17
  activation determines the Must Have row count: if still `ready`
  (human-operator-blocked), the row is carried into Sprint 17 as
  Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; if closed on
  `origin/main` by a separate prompt before Sprint 17 activation,
  the row is dropped from Sprint 17.
- The four in-flight repair tracks (PROMPT 1080 / 1083 modal;
  PROMPT 1085 shop / auction surface paint; PROMPT 1086 client
  placement perspective; PROMPT 1087 / 1089 lobby class art + confirm
  button) are each independently audited at activation time. For
  every track NOT on `origin/main`, the corresponding conditional
  Must Have row below is promoted to active. For every track
  already on `origin/main`, the corresponding row is dropped (the
  repair is treated as already discharged).
- The Sprint 17 candidate story files for any net-new row do NOT
  yet exist on `origin/main` at draft time. **Story authoring
  prompts are a prerequisite to Sprint 17 activation** for each
  net-new row. The HUD timer Must Have carry uses the existing
  story 014 file unchanged.

If any entry condition fails, Sprint 17 does NOT activate; producer
must revise scope before activation.

## Sprint Goal

Sprint 17 is a **focused runtime / UI repair sprint**: it
prioritises runtime-playability blockers and visible UX fixes
identified by the PROMPT 1076 latest user-test log / snapshot deep
audit and the PROMPT 1077 UI / state source consistency deep audit.
It is **NOT** a release sprint and is not gated on Sprint 16 manual
QA closure; the Sprint 16 close-out conditions carry forward
unchanged. The goal is:

1. **Discharge the four in-flight runtime / UI repair tracks** that
   were authored against Sprint 16 evidence but did NOT land on
   `origin/main` before Sprint 16 close-out:
   - PROMPT 1080 / 1083 modal black-slab repair (AUDIT-1076-01).
   - PROMPT 1085 client shop / auction surface paint + intent
     repair (AUDIT-1076-04 + AUDIT-1076-13).
   - PROMPT 1086 client placement perspective + invalid-drop
     feedback repair (AUDIT-1076-09 + PROMPT 1079 client residual
     risk #2).
   - PROMPT 1087 / 1089 lobby class art + Confirm-class button
     integration (AUDIT-1076-06 + AUDIT-1076-07).
   Each is a **conditional Must Have row**: pulled forward only if
   NOT on `origin/main` at activation time.
2. **Land the PROMPT 1077 P0 structural card-display-art bundle**
   (SOURCE-1077-01 slot-well chrome preservation +
   SOURCE-1077-02 duplicate-helper dedup + SOURCE-1077-03 leak fix
   + SOURCE-1077-04 existence check). One bundled Must Have row;
   the four findings must land together to avoid re-introducing
   the empty-slot bug after dedup.
3. **Close the Sprint 13 -> 14 -> 15 -> 16 -> 17 human-operator-
   blocked HUD timer carry** (`S11-HUD-TIMER-EYEBALL-VISUAL-001`)
   if it has not already closed on `origin/main` by a separate
   prompt before Sprint 17 activation. Closure remains gated on
   human-operator action.
4. **Land visible UX cleanup Should Have rows** identified by
   PROMPT 1076 P2 / P3 and PROMPT 1077 P1 / P2:
   - HUD opponent figurine / OPP label / mana duplicate cleanup
     (AUDIT-1076-10 + AUDIT-1076-16 + AUDIT-1076-17).
   - Auction timer state-machine latency repair (AUDIT-1076-12)
     **conditional** -- pulled only if NOT covered by an
     in-flight track at activation.
   - Card-slot primitive image / text inset wiring
     (SOURCE-1077-06; per-surface migration siblings remain
     separate Sprint 17+ candidates).
   - QA snapshot marker split + visibility-aware counts
     (SOURCE-1077-08 + SOURCE-1077-09 + SOURCE-1077-16).
   - Bid-button phase-entry race cleanup (SOURCE-1077-10).
5. **Discharge two small Nice to Have hygiene rows**:
   - Vulkan validation-layer warning gating (AUDIT-1076-18).
   - `start_of_turn_dispatch_system not yet implemented` warn ->
     debug downgrade (AUDIT-1076-15).
6. **Hand UI `B0004` hierarchy warning cleanup** (AUDIT-1076-14)
   is a Nice to Have candidate; producer may keep or drop based
   on Sprint 17 capacity at activation.

Sprint 17 does NOT claim release readiness, broad accessibility
completion, full playable-client manual QA, playtest validation,
final-art / asset-production completion, `S8-QA-001-W1` closure,
full game completion, two-client GAME_OVER closure, a Polish->Release
retry, closure of the underlying drag-runtime bug from Sprint 12
story 019, closure of any of the 24 PROMPT 1022 audit findings, or
closure of any PROMPT 1076 / PROMPT 1077 finding for which a
concrete repair is not already on `origin/main` at activation
time. The 24 PROMPT 1022 audit findings, the 12 Tier 2 cosmetic-
capture future candidates, the Sprint 11/12/13 server hardening
backlog, the full per-surface card-slot primitive migration of
hand / draft-grid / auction-featured / board-staged-ghost surfaces,
and real-art production for lobby portraits / player slot / room
code chip / board sprites remain deferred to Sprint 18+ **explicitly**.

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10..16)
- Buffer (25%): 2.5 days reserved for (a) per-row `/story-readiness`
  re-runs against Sprint 17 activation HEAD; (b) the net-new
  story-authoring prompts that must precede `/dev-story` for any
  candidate row whose story file does not yet exist; (c) `/qa-plan
  sprint-17` authoring; (d) integration / `/story-done` paperwork
  serialisation overhead given the high row count; (e) producer /
  human-operator scheduling friction on the human-operator-blocked
  Must Have carry if still open at activation.
- Available: **7.5 effective planned days**
- Planned Must Have scope (upper bound, all conditional rows
  promoted): **~5.0 estimated days**
  - HUD timer carry 0.25d (human-operator-blocked)
  - Modal repair main-push paperwork 0.25d **OR** worker re-run
    1.0d if PROMPT 1080 / 1083 cannot be fast-forwarded
  - Shop / auction surface paint + intent repair 1.5d
  - Client placement perspective + invalid-drop feedback 1.0d
  - Lobby class art + Confirm-class main-push paperwork 0.25d
    **OR** worker re-run 1.5d if PROMPT 1087 / 1089 cannot be
    fast-forwarded (placeholder asset md5 gaps may need rework)
  - PROMPT 1077 P0 card-display-art bundle 0.75d
- Planned Should Have scope: **~1.75 estimated days**
  - HUD opponent figurine + OPP label + mana duplicate cleanup
    0.5d
  - Auction timer state-machine latency repair (conditional) 0.5d
  - Card-slot primitive image / text inset wiring 0.25d
  - QA snapshot marker split + visibility-aware counts 0.25d
  - Bid-button phase-entry race cleanup 0.25d
- Planned Nice to Have scope: **~0.4 estimated days**
  - Vulkan validation gating 0.15d
  - `start_of_turn_dispatch_system` warn -> debug 0.1d
  - Hand UI B0004 hierarchy warning cleanup 0.15d
- Total implementation effort (Must + Should + Nice, upper bound):
  **~7.15 days against 7.5 days available**. Tight but within
  capacity if conditional Must Have rows drop because their
  in-flight repairs land on `origin/main` before activation.
- **Lower-bound scenario**: if all four in-flight repairs (PROMPT
  1080 / 1083, 1085, 1086, 1087 / 1089) land on `origin/main`
  before activation, Sprint 17 Must Have scope shrinks to (HUD
  timer carry 0.25d + PROMPT 1077 P0 bundle 0.75d) = ~1.0d, and
  the full Should + Nice list lands comfortably within capacity.
- If burn-down comes in significantly under capacity, a producer
  may pull a single additional row from the Sprint 17 Backlog
  section (priority: per-surface card-slot primitive migration of
  one of hand-fan / draft-grid / auction-featured / board-staged-
  ghost surfaces; OR a single P1 PROMPT 1022 finding split into a
  single-surface story; do NOT pull multiple migration siblings
  or multiple P1 findings simultaneously without separate sprint
  scoping).

---

## Tasks

> All IDs below are **draft Sprint 17 candidate** tickets. They are
> NOT yet active `production/sprint-status.yaml` rows. Promotion to
> active rows happens at activation via a separate explicit prompt
> (mirrors the PROMPT 826 / PROMPT 897 / PROMPT 997 / PROMPT 1064
> pattern), after Sprint 16 close-out paperwork landed on
> `origin/main` (DONE per PROMPT 1088 at `fec13ff`). Conditional
> rows below are pulled forward only if the corresponding in-flight
> repair is NOT on `origin/main` at activation time; if the repair
> has landed before activation, the row is dropped from Sprint 17.

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Conditional? | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------------|--------|------------------------------|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD Timer Eyeball Visual Check (Sprint 13 -> 14 -> 15 -> 16 -> 17 carry; **human-operator-blocked**) -- manual 2-client run validating timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases. **Conditional row: dropped at Sprint 17 activation if closed on `origin/main` by a separate prompt before activation.** | UI programmer + **human operator** | 0.25 | yes (dropped if closed on `origin/main` before activation) | **Sprint 16 carry** per PROMPT 1082 close-out + PROMPT 1088 main integration; originally Sprint 10 smoke retry-7 W2 -> Sprint 11 -> Sprint 12 -> Sprint 13 -> Sprint 14 -> Sprint 15 -> Sprint 16 -> Sprint 17 carry. Story file at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` (PROMPT 822 author / PROMPT 823 READY / PROMPT 1000 readiness rerun READY). | Story 014 `/story-readiness` confirms READY against Sprint 17 activation HEAD (expected READY unless the story file has been touched). Manual 2-client run + screenshot evidence in `production/qa/evidence/sprint-17-hud-timer-visual-check/` (NEW). Cosmetic verification only; no production-code change unless an actual visual regression is found and a follow-on story is authored. **Closure remains gated on human-operator screenshot capture** -- no LLM `/story-done` is authorised. Does NOT claim `QA-COND-0005` Standard-tier accessibility completion. `PAW-TD-*-a` preserved. If human-operator time cannot be scheduled within Sprint 17, the row carries forward as a Sprint 17 -> Sprint 18 carry. |
| S17-UI-MODAL-BLACK-SLAB-001 | Client phase-modal black-slab repair -- DraftInitial modal must hide outside DraftInitial; restore Placement / Shop / Auction / Resolution surface paint behind the modal | UI programmer | 0.25 (paperwork) **OR** 1.0 (worker re-run) | yes (dropped if PROMPT 1080 / 1083 on `origin/main` at activation) | AUDIT-1076-01 (P0) -- giant ~850 x 340 px black modal in screen centre obscures the board through every InSession phase. In-flight tracks: PROMPT 1080 worker `cbc11b2` on `origin/work/client-phase-modal-black-slab-1080`; PROMPT 1083 integration `e4bbca3` on `origin/integrate/client-phase-modal-black-slab-1083` (10/10 focused tests PASS at integration time). NOT on `origin/main` at draft time. | If PROMPT 1083 integration can be fast-forwarded into `origin/main`: paperwork-only main-push prompt. If not (likely; PROMPT 1082 + 1088 close-out paperwork commits intervene): worker re-run rebasing `cbc11b2` onto `fec13ff` and re-running the 10 focused tests, then new integration prompt. Files: `client/src/ui/shop_auction/mod.rs`, `tests/integration/shop_auction_ui/draft_initial_centered_modal_layout_test.rs`. No protocol-shape change; no new server-authoritative state. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-SHOP-AUCTION-SURFACE-PAINT-001 | Client shop / auction surface paint + intent repair -- DraftShop / DraftAuction surfaces must paint shop tiles + auction featured card + bid controls when data arrives; shop tile click must initiate a purchase intent (or show an affordance), not silently send `C2SActivateCard` | UI programmer | 1.5 | yes (dropped if PROMPT 1085 on `origin/main` at activation) | AUDIT-1076-04 (P1) + AUDIT-1076-13 (P2) -- shop / auction surfaces render as empty black slabs even though `drain_shop_slots slots_len=3` and `drain_auction_card card_id=CardId(106)` arrive on the client; shop card clicks fire repeated `C2SActivateCard` storms with no `C2SPurchaseCard` follow-up. In-flight track: PROMPT 1085 worker on `origin/work/client-shop-auction-surface-paint-1085` (no integration prompt at draft time). NOT on `origin/main` at draft time. **Depends on S17-UI-MODAL-BLACK-SLAB-001 landing first** -- the modal repair is a prerequisite because the shop and auction surfaces paint behind the same modal root. | If PROMPT 1085 worker is current and clean: integration prompt + main-push paperwork. If worker is stale or incomplete: worker re-run against `fec13ff` + modal repair tip + integration + main-push. Files: `client/src/ui/shop_auction/shop_slot*.rs`, `client/src/ui/shop_auction/auction_*.rs`, `client/src/ui/design_tokens/z_layers.rs`, `client/src/ui/shop_auction/shop_slot_click*.rs`. New / updated integration tests in `tests/integration/shop_auction_ui/` for shop tile paint, auction featured card paint, and shop tile purchase-intent click handler. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-PLACEMENT-PERSPECTIVE-001 | Client placement perspective + invalid-drop feedback repair -- staged placements must render at the user's own-board perspective; staged-but-not-submitted placements must show explicit feedback before the 10 s placement window expires | UI programmer | 1.0 | yes (dropped if PROMPT 1086 on `origin/main` at activation) | AUDIT-1076-09 (P1 UX) + PROMPT 1079 client residual risk #2 -- run-7 Round 1: Player 2 staged 2 cards but `C2SSubmitPlacement` was never sent; staged placements silently dropped on phase transition with no `auto_submit_on_phase_end` log. In-flight track: PROMPT 1086 worker on `origin/work/client-placement-perspective-submit-1086` (no integration prompt at draft time). NOT on `origin/main` at draft time. AUDIT-1076-02 + AUDIT-1076-03 (server side) ARE on `origin/main` via PROMPT 1079 + PROMPT 1084. | If PROMPT 1086 worker is current and clean: integration prompt + main-push paperwork. If worker is stale: re-run against `fec13ff`. Files: `client/src/ui/hand/staging*.rs`, `client/src/ui/shop_auction/placement_submit*.rs`. New / updated integration tests in `tests/integration/placement_ui/` for staged-but-not-submitted feedback path and own-board placement perspective. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-LOBBY-CLASS-ART-CONFIRM-001 | Lobby class art + Confirm-class button integration -- 7 class-portrait placeholder PNGs + slot-panel placeholder + room-code-chip placeholder + 7 board-sprite placeholders authored and wired; Confirm-class button rendered as a visually distinct, hit-targetable primary-action button | UI programmer + art-director | 0.25 (paperwork) **OR** 1.5 (worker re-run) | yes (dropped if PROMPT 1087 / 1089 on `origin/main` at activation) | AUDIT-1076-06 (P1) + AUDIT-1076-07 (P1) -- class picker shows generic `?` cards for all 7 classes; Confirm-class button reads as a dim text band "Confirm your class to continue" with no border / fill / arrow / label distinguishing it from a status message. In-flight tracks: PROMPT 1087 integration `eec2a91` on `origin/integrate/lobby-class-art-confirm-1087` (49/49 PASS, 5 files M3/A2 at integration time); PROMPT 1089 main-push attempt blocked (`NEEDS-REFRESH`; non-FF would lose PROMPT 1082 + 1088 close-out paperwork). NOT on `origin/main` at draft time. | If PROMPT 1087 integration can be rebased or refreshed onto `fec13ff`: refresh + main-push paperwork (per PROMPT 1089 "next: refresh-worker-merge-or-rebase"). If not: worker re-run + integration + main-push. Placeholder asset md5 gaps (lobby-portrait-png x7 + slot-panel + room-code-chip + board-sprite x7) noted by PROMPT 1087 must be resolved before close-out unless explicitly deferred to a Sprint 17+ real-art row. Files: `client/src/ui/lobby/class_picker*.rs`, `client/src/ui/lobby/confirm_class*.rs`, `client/src/ui/lobby/intent_chain*.rs`, `client/src/asset_wiring.rs`, `assets/art/lobby/` placeholder PNGs, new lobby UI integration tests. **No final-art claim** -- placeholders only; `PAW-TD-*-a` accept-risk preserved. `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-CARD-DISPLAY-ART-HELPER-001 | Card display art helper / chrome preservation + dedup + leak fix + existence check (PROMPT 1077 P0 bundle) -- lift `apply_card_display_art` and `clear_card_display_art` from the duplicate `shop_auction/mod.rs` + `hand/mod.rs` sites into a shared owner; preserve slot-well chrome (`ImageNode`) when card art is missing; replace `Box::leak` with a non-leaking lifetime strategy; check art asset exists on disk before returning a path | UI programmer | 0.75 | no | SOURCE-1077-01 (P0) slot-well chrome lost when card art is missing + SOURCE-1077-02 (P0) duplicate `apply_card_display_art` definitions + SOURCE-1077-03 (P1) per-render `Box::leak` in `resolve_card_display_art` + SOURCE-1077-04 (P1) `resolve_card_display_art` returns path without existence check. Bundled because dedup MUST land in the same commit as the slot-well chrome fix or the dedup re-introduces the empty-slot bug; the leak fix and existence check share `client/src/asset_wiring.rs:505-518` and are cheaper to land together. | Single bundled story file authored under `production/epics/ui-clean-pass/story-XXX-card-display-art-helper.md` (slug TBD by story-authoring prompt). Helper lifted to a single owner -- recommended location alongside `resolve_card_display_art` in `client/src/asset_wiring.rs` or in a new `client/src/ui/design_tokens/card_art.rs` module. Slot-well chrome preserved as a separate component (`CardArtImageNode` child vs slot-well `ImageNode`) so missing card art does NOT remove the slot chrome. `resolve_card_display_art` returns a non-leaking type (`Cow<'static, str>` or `String` plus a registry lookup that returns `&'static str` for cached paths) -- exact strategy TBD by story-authoring prompt. Existence check probes the asset registry or filesystem before returning; missing assets fall through to a documented placeholder. New / updated integration tests in `tests/integration/ui_clean_pass/` and `tests/integration/shop_auction_ui/` assert (1) slot-well chrome survives missing card art, (2) helper exists at a single owner site (no duplicate symbol), (3) per-render allocations are bounded (no leak path on a 1000-card stress run), (4) `resolve_card_display_art` returns the placeholder path for `art_id = "missing"`. No protocol-shape change; no new server-authoritative state. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |

**Must Have subtotal**: **~5.0 estimated days upper bound** (all
conditional rows promoted, worker re-runs assumed); **~1.0
estimated days lower bound** (all conditional rows dropped, HUD
timer 0.25d + PROMPT 1077 P0 bundle 0.75d only). The mix depends
on which in-flight repairs land on `origin/main` before activation.

### Should Have

| ID | Task | Agent/Owner | Est. Days | Conditional? | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------------|--------|------------------------------|
| S17-UI-HUD-OPP-MANA-CLEANUP-001 | HUD opponent figurine + OPP label + mana duplicate cleanup -- opponent figurine strip and OPP label must repaint after `S2CClassesRevealed`; the floating "Reserve 0 + / Current 2" mana microbadge must be removed in favour of the canonical `MANA 2 / 10` HUD strip | UI programmer | 0.5 | no | AUDIT-1076-10 (P2) + AUDIT-1076-16 (P3) -- top-left opponent-figurines strip renders 4-5 small grey circles each with a `?` overlay even after `S2CClassesRevealed`; HUD `OPP ?` survives reveal. AUDIT-1076-17 (P3) -- "Reserve 0 + / Current 2" microbadge floats above the board while HUD shows `MANA 2 / 10`. | Story file authored under `production/epics/hud/story-XXX-opp-mana-cleanup.md` (slug TBD). Opponent figurine + OPP label subscribe to `S2CClassesRevealed` reducer and repaint with class crest. Mana microbadge removed or merged into the HUD strip. Integration test asserts opponent figurine repaint on classes-revealed and absence of duplicate mana display. Files: `client/src/ui/hud/opponent_figurine*.rs`, `client/src/ui/hud/opp_label*.rs`, `client/src/ui/hud/mana_*.rs`, `client/src/state/mod.rs` (apply_classes_revealed reducer). No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-SERVER-AUCTION-TIMER-001 | Auction state-machine timer latency repair -- `S2CAuctionCard.timer_duration_ms=20000` and `S2CAuctionBidAccepted.new_timer_ms=20000` must drive settle at <=30s after last bid, not <=149s phase-timer expiry | server programmer + network programmer | 0.5 | yes (dropped if already on `origin/main` at activation via a separate repair track) | AUDIT-1076-12 (P2) -- run-7: bid `+4` at 23:50:07 -> `settle_expired_auction` at 23:52:36 (~149 s). Either the `Resolving` transition is gated on a longer phase timer or the per-bid extension is mis-multiplied. | Story file authored under `production/epics/server-auction/story-XXX-auction-timer-state-machine.md` (slug TBD). Files: `server/src/game/auction*.rs`, `server/src/core/rsm/transitions.rs` (`DraftAuction` phase tick). New unit / integration tests assert settle-time within `timer_duration_ms + extension_ms` of the last bid regardless of surrounding phase timer. No protocol-shape change (timer fields already in protocol). `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-CARD-SLOT-INSET-WIRING-001 | Card-slot primitive image / text inset wiring -- `card_slot_node(kind)` must build full image inset + text inset + padding + `GlobalZIndex` so per-surface consumers stop re-authoring child layout; this completes Sprint 16 Phase 1 primitive ratification | UI programmer | 0.25 | no | SOURCE-1077-06 (P1) -- `card_slot_node(kind)` builds outer rectangle only; every other surface still re-authors child layout. Sprint 16 PROMPT 1067 / 1073 / 1074 closed primitive Phase 1 (shop_slot migration only); per-surface migration of hand-fan / draft-grid / auction-featured / board-staged-ghost remain Sprint 17+ siblings. This row does NOT migrate consumer surfaces; it ratifies the primitive so per-surface migration siblings can land cleanly. | Story file authored under `production/epics/ui-clean-pass/story-XXX-card-slot-inset-wiring.md` (slug TBD). Files: `client/src/ui/design_tokens/card_slot.rs`. New integration test in `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` (or a sibling) asserts inset / padding / z-index wiring at the primitive level. No consumer surface migrated by this row. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. Per-surface migration siblings (`S17-UI-CARD-SLOT-MIGRATION-HAND-001`, `-DRAFT-GRID-001`, `-AUCTION-FEATURED-001`, `-BOARD-GHOST-001`) remain Sprint 17+ Backlog candidates. |
| S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 | QA snapshot marker split + visibility-aware counts -- `HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity` markers split per sub-surface; `qa_snapshot.rs` counts filter by `Visibility::Inherited`/`Visible` so pre-spawned hidden roots stop double-counting | UI programmer + QA tooling | 0.25 | no | SOURCE-1077-08 (P2) + SOURCE-1077-09 (P2) + SOURCE-1077-16 (P3) -- universal markers + visibility-blind counts produce misleading QA snapshot JSON; `ConnectionLostOverlayRoot` and `ResultScreenRoot` spawned at `Startup` are counted before they paint; snapshot ID format aliases across two concurrent clients. | Story file authored under `production/epics/ui-clean-pass/story-XXX-qa-snapshot-marker-split.md` (slug TBD). Files: `client/src/presentation/qa_snapshot.rs`, marker definitions in `client/src/ui/hud/`, `client/src/ui/hand/`, `client/src/ui/shop_auction/`, `client/src/presentation/connection_lost_overlay.rs`, `client/src/presentation/board_rendering.rs`. New / updated integration tests assert marker per-sub-surface granularity and visibility-aware counts. Snapshot ID format includes a client-disambiguating prefix. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-BID-BUTTON-PHASE-RACE-001 | Bid-button phase-entry race cleanup -- bid-button text must not render as empty `""` or baked `"?"` during the phase-entry race before `S2CAuctionCard` arrives | UI programmer | 0.25 | no | SOURCE-1077-10 (P2) -- bid-button text spawn-state is `Text::new("")` populated only after `S2CAuctionCard` arrives; `ui_bid_button_disabled.png` has `"?"` baked in. Phase-entry race produces visible empty / `"?"` buttons. | Story file authored under `production/epics/ui-clean-pass/story-XXX-bid-button-phase-race.md` (slug TBD). Files: `client/src/ui/shop_auction/auction_bid_buttons*.rs` (or equivalent), `assets/ui/ui_bid_button_disabled.png` if the baked `"?"` is replaced. New / updated integration test in `tests/integration/shop_auction_ui/` asserts bid-button text is `Loading…` (or equivalent) before `S2CAuctionCard` arrives, then numeric bid amounts after. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |

**Should Have subtotal**: ~1.75 estimated days. Land only if Must
Have closure is on track.

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Conditional? | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------------|--------|------------------------------|
| S17-OPS-VULKAN-VALIDATION-GATING-001 | Vulkan validation-layer warning gating -- `InstanceFlags::VALIDATION requested, but unable to find layer: VK_LAYER_KHRONOS_validation` warnings on every client launch silenced or gated on a `cargo` feature so prod / CI logs stay clean | UI programmer + devops-engineer | 0.15 | no | AUDIT-1076-18 (P3) -- 3 Vulkan validation warnings on every client launch (run-7 client-a:2-5, client-b:2-5). Dev-only, harmless on the test machine. | Story file authored under `production/epics/devops/story-XXX-vulkan-validation-gating.md` (slug TBD). Files: `client/src/main.rs` (or wherever `App::new()` configures the WGPU plugin) gates validation on `cfg!(debug_assertions)` and / or `--features wgpu-validation`. Smoke harness confirms zero Vulkan validation lines on the next Sprint 17 smoke. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-SERVER-START-OF-TURN-DEBUG-001 | `start_of_turn_dispatch_system not yet implemented` warn -> debug downgrade -- known-deferred WARN spam on every round entry demoted to `debug!` until keyword dispatch is implemented | server programmer | 0.1 | no | AUDIT-1076-15 (P3) -- 6 WARN lines per session (server.log:67, 109, 161, 240, 293, 337). Known-deferred work; reduces noise so real warnings stand out. | Story file authored under `production/epics/server-game/story-XXX-start-of-turn-debug-downgrade.md` (slug TBD). Files: `server/src/game/mod.rs` or `server/src/game/dispatch*.rs`. Smoke harness confirms zero `not yet implemented` WARN lines on the next Sprint 17 smoke; same lines visible at `debug!` level. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S17-UI-HAND-B0004-CLEANUP-001 | Hand UI `B0004` hierarchy warning cleanup -- Hand UI Fan Root entity with `GlobalTransform` component must not have a parent (HandBar) without `GlobalTransform` | UI programmer | 0.15 | no | AUDIT-1076-14 (P3) -- 1 `B0004` warning per client per InSession entry (run-7 client-a:94, client-b:116). Structural ECS warning; may explain brittleness in hand layout under window resize but no functional bug evident yet. | Story file authored under `production/epics/ui-clean-pass/story-XXX-hand-b0004-hierarchy.md` (slug TBD). Files: `client/src/ui/hand/fan_root*.rs`, `client/src/ui/hand/hand_bar*.rs`. Hand UI hierarchy aligned so HandBar carries `GlobalTransform` OR FanRoot is reparented. Smoke harness confirms zero `B0004` lines on the next Sprint 17 smoke. No protocol-shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |

**Nice to Have subtotal**: ~0.4 estimated days. Land only if Must
Have + Should Have closure is on track. Trivially small; each row
pays back per-session developer / smoke friction.

---

## Carryover from Sprint 16

| Source row (Sprint 16) | Disposition into Sprint 17 |
|------------------------|----------------------------|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 16 Must Have, `ready` after PROMPT 1072 + PROMPT 1074 closure of the other 3 rows -- the only un-closed row of Sprint 16; human-operator-blocked cosmetic visual check; Sprint 13 -> 14 -> 15 -> 16 carry) | **Conditional Sprint 16 -> Sprint 17 carry**: pulled forward as Sprint 17 **Must Have** human-operator-blocked carry **only if** the row remains `ready` on `origin/main` at Sprint 17 activation. If a producer-scheduled human-operator capture session closes the row on `origin/main` between this draft and Sprint 17 activation, the row is dropped from Sprint 17 (Must Have shrinks by 0.25d). Disposition preserved unchanged: closure remains gated on human screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases; no LLM `/story-done` is authorised; PROMPT 822 / 823 / 894 / 987 / 988 / 997 / 1064 / 1082 disposition preserved. Story file unchanged at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`. Evidence target: `production/qa/evidence/sprint-17-hud-timer-visual-check/` (NEW). |
| All 3 closed Sprint 16 rows (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001` Should, `S15-OPS-APPCOMPAT-MANIFEST-001` Nice, `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` Nice) | Preserved unchanged on `origin/main` per PROMPT 1072 + PROMPT 1074 closures + PROMPT 1082 close-out + PROMPT 1088 main integration. None reopened or revisited by Sprint 17. |
| Sprint 16 close-out paperwork (PROMPT 1082 + PROMPT 1088) | Preserved unchanged on `origin/main`. Sprint 17 does NOT reopen, re-author, or re-integrate Sprint 16 close-out. |
| Sprint 16 QA evidence on QA branches (PROMPT 1075 smoke `PASS-WITH-WARNINGS` on `origin/qa/sprint-16-smoke-check-1075`; PROMPT 1078 Team-QA `APPROVED-WITH-CONDITIONS` on `origin/qa/sprint-16-team-qa-1078`) | Preserved unchanged on their respective QA branches. Sprint 17 does NOT integrate these evidence files under `production/qa/` on `origin/main`. |
| In-flight Sprint 16-era repair tracks NOT on `origin/main` at draft time (PROMPT 1080 / 1083 modal; PROMPT 1085 shop / auction; PROMPT 1086 placement perspective; PROMPT 1087 / 1089 lobby class art) | **Conditional Sprint 16 -> Sprint 17 spill-over**: each track is represented as a conditional Must Have row above (S17-UI-MODAL-BLACK-SLAB-001, S17-UI-SHOP-AUCTION-SURFACE-PAINT-001, S17-UI-PLACEMENT-PERSPECTIVE-001, S17-UI-LOBBY-CLASS-ART-CONFIRM-001). If the track lands on `origin/main` before Sprint 17 activation, the corresponding row is dropped. If the worker is stale, a worker re-run is scoped at activation time. |

(All Sprint 16 closed rows + all 4 closed Sprint 15 rows + all 16
closed Sprint 14 rows + all closed Sprint 13 / Sprint 12 / Sprint
11 / Sprint 10 rows are preserved unchanged on `origin/main`. None
are reopened or revisited by Sprint 17.)

## Conditions Carried Forward Unchanged (NOT closed by Sprint 17)

Sprint 17 explicitly preserves and does NOT claim closure for any
of:

- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap
  remains **OPEN**. Sprint 13 story 017 AC12 forbid-auto-closure
  preserved through Sprint 13 / 14 / 15 / 16. Sprint 17 candidate
  rows do not touch the two-client GAME_OVER surface. Sprint 17
  activation MUST NOT silently close `S8-QA-001-W1`.
- **`QA-COND-0005`** -- Standard-tier accessibility remains
  **accepted-risk** (friend-game scope only). Sprint 17 repair /
  cleanup rows are friend-game visual polish / ops hygiene only.
  The L5 `LOBBY_BUTTON_HEIGHT = 30.0` defect (PROMPT 802 §3.1 L5)
  remains accepted-risk under `QA-COND-0005`. Sprint 17 does NOT
  pursue WCAG contrast ratios, >=44 px hit-targets, full keyboard
  navigation, screen reader support, colorblind modes, or text
  scaling.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation
  remains **accepted-risk / deferred**. Sprint 17 repair / cleanup
  rows do NOT advance playtest validation even where the surface
  becomes visibly playable.
- **`PAW-TD-*-a`** -- placeholder-art accept-risk across
  PAW-002..PAW-006 remains in place. Sprint 17 lobby class art row
  authors **placeholder** PNGs only (per PROMPT 1087 evidence);
  real-art production for lobby portraits / player slot / room
  code chip / board sprites is **explicitly deferred** to Sprint
  18+.
- **PROMPT 683-era runtime divergence question** -- folded into
  Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`
  (after second time-box exhaustion). Sprint 17 does NOT claim
  this question closed. **A third same-scope retest is NOT
  authorised** per `TQ-S12-C2`.
- **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved
  at `production/gate-checks/gate-polish-release-2026-05-12.md`.
  **NO retry** is in scope for Sprint 17. Stage remains `Polish`.
- **Sprint 12 story 019 underlying drag-runtime bug** -- NOT
  claimed fixed. Sprint 17 repair / cleanup rows are additive
  client-UI / server-timer work and do not reproduce or fix the
  underlying drag-runtime behaviour.
- **`TQ-S12-C1..C7`** -- all 7 Sprint 12 Team-QA conditions
  preserved verbatim. `TQ-S12-C7` explicitly NOT closed by any
  Sprint 17 row.
- **Sprint 16 / Sprint 15 / Sprint 14 / Sprint 13 / Sprint 12 /
  Sprint 11 / Sprint 10 closeouts** -- preserved unchanged. Sprint
  16 `closed-with-conditions` per PROMPT 1082 + PROMPT 1088;
  Sprint 15 `closed-with-conditions` per PROMPT 1056; Sprint 14
  `closed-with-conditions` per PROMPT 987; Sprint 13
  `closed-with-conditions` per PROMPT 894; Sprint 12
  `closed-with-conditions` per PROMPT 817; Sprint 11
  `closed-with-conditions` per PROMPT 792; Sprint 10
  `closed-with-conditions` per PROMPT 763.
- **All closed `/story-done` closures** across Sprint 10 -> 16
  preserved unchanged on `origin/main`. Sprint 17 does NOT reopen
  any of them.
- **24 PROMPT 1022 audit findings** -- preserved as report-only
  inputs. None are pulled as Sprint 17 active rows. None are
  claimed closed by Sprint 17. Each future Sprint 18+ row that
  pulls one or more findings requires its own story file authored
  via a separate story-authoring prompt, its own `/story-readiness`
  pass, and its own QA plan reference.
- **PROMPT 1054 P1 UI snapshot visual retest
  `BLOCKED-HUMAN-OPERATOR`** -- preserved deferred. Sprint 17
  does NOT close this retest by an LLM.
- **Sprint 16 PROMPT 1075 smoke + PROMPT 1078 Team-QA evidence**
  preserved on QA branches; Sprint 17 does NOT re-integrate.

If any condition above changes during Sprint 17, it requires its
own separate story file and explicit disposition -- it cannot be
silently folded into another Sprint 17 row.

## Wider Sprint 17 Backlog (NOT scheduled into this draft; deferred to Sprint 18+)

The following candidates remain in the broader backlog and are
**NOT scheduled** into this Sprint 17 draft. They may be pulled by
a producer revision before activation (priority: a single per-
surface card-slot primitive migration row; OR a single P1 PROMPT
1022 finding split into a single-surface story; OR a single small-
asset placeholder real-art row), or deferred further to Sprint 18+:

### Per-surface card-slot primitive migration siblings (Sprint 16+ family)

Sprint 16 closed `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` Phase 1
(primitive module + shop_slot migration only). Per-surface migration
of the four remaining consumer surfaces remains an explicit Sprint
17+ family per the Sprint 16 story 009 Parallelization and Phase
Breakdown section:

- `S17-UI-CARD-SLOT-MIGRATION-HAND-001` -- migrate
  `client/src/ui/hand/mod.rs::hand_fan_card_node` to
  `card_slot_node(CardSlotKind::HandFan)`.
- `S17-UI-CARD-SLOT-MIGRATION-DRAFT-GRID-001` -- migrate draft
  initial keep-9 modal grid to `card_slot_node(CardSlotKind::
  DraftGrid)`.
- `S17-UI-CARD-SLOT-MIGRATION-AUCTION-FEATURED-001` -- migrate
  `client/src/ui/shop_auction/mod.rs::auction_featured_card_node`
  to `card_slot_node(CardSlotKind::AuctionFeatured)`.
- `S17-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001` -- migrate board
  staged-ghost rendering to `card_slot_node(CardSlotKind::
  BoardStagedGhost)`.

Each is owned by `client/src/ui/hand/` or
`client/src/ui/shop_auction/` or `client/src/presentation/
board_rendering.rs` and requires its own story file before
activation. Producer may pull one of the four into Sprint 17 if
Sprint 17 capacity allows; do NOT pull more than one per sprint
without separate sprint scoping.

### Real-art production for lobby portraits + player slot + room code chip + board sprites

Lobby class art Must Have row (S17-UI-LOBBY-CLASS-ART-CONFIRM-001)
authors **placeholder** PNGs only. Real-art production for 7 lobby
portraits + slot-panel chrome + room-code-chip + 7 board sprites is
deferred to Sprint 18+. Each is owned by `art-director` +
`sound-designer` (where appropriate) and requires its own story
file. `PAW-TD-*-a` accept-risk preserved.

### PROMPT 1022 QA snapshot audit findings (24 total; deferred to Sprint 18+)

`reports/PROMPT-1022-qa-snapshot-visual-state-audit.md` produced 24
findings across 5 P1, 6 P2, 5 P3, 6 state-mismatch / instrumentation,
and 2 snapshot-tool. **None are pulled as Sprint 17 active rows.**

### PROMPT 1077 remaining structural findings (12 total; deferred to Sprint 18+)

Sprint 17 Must Have row S17-UI-CARD-DISPLAY-ART-HELPER-001 absorbs
SOURCE-1077-01 / 02 / 03 / 04 (P0 / P0 / P1 / P1). The following 12
SOURCE-1077-* findings remain backlog candidates:

- **SOURCE-1077-05** (P1) -- `interaction_states::*` token set is
  referenced only by tests + card_slot doc comments; no surface
  consumes it.
- **SOURCE-1077-07** (P2) -- four parallel card-label string
  formats across draft / shop / shop-footer / auction; no shared
  formatter; Rarity uses Debug `{:?}` in two sites.
- **SOURCE-1077-11** (P2) -- `ShopAuctionUiMode::from_phase` vs
  `HandUiMode::from_phase` divergence; auction activation
  branch dependent on `auction_state.card_id`.
- **SOURCE-1077-12** (P2) -- `BoardRuntimeAssets` insertion gated
  on `asset_server.is_some()`; tests fall through to legacy
  flat-colour `Sprite::from_color` ("grey square" reachable in
  test-fixture environments).
- **SOURCE-1077-13** (P2) --
  `hand_ui_chrome_composition_test.rs` re-declares production
  constants as test-locals instead of importing them.
- **SOURCE-1077-14** (P2) -- card-rendering integration tests run
  without `AssetServer`; ImageNode survival never asserted under
  the real `Some(asset_server)` branch.
- **SOURCE-1077-15** (P3) -- test card fixtures use `art_id:
  format!("test_{id}")` so disk asset never exists; production
  art-loading path is untested.

Each is owned by `client/src/asset_wiring.rs`,
`client/src/ui/design_tokens/`, `client/src/presentation/
board_rendering.rs`, or `tests/integration/`. Producer may pull
one P1 finding into Sprint 17 if capacity allows; do NOT pull
multiple findings without separate sprint scoping.

### Server hardening backlog (Sprint 11/12/13 carryover)

- `S11-TD-NET-001`, `S11-TD-NET-002`, `S11-TD-NET-003` -- server
  hardening test parity. Defer to a focused server-hardening
  sprint.
- `S11-TD-PRISM-COV-001` -- Cluster 2C advisory coverage gap on
  `S2CPrismRewardDropped` + `S2CPrismRespawned`.
- `S11-TD-HARNESS-MESSAGES-001` -- 4 harness bins downstream from
  PROMPT 690 needing `add_message::<PlayerTeamMapUpdated>`.
- `S11-TD-HARNESS-HANDUI-ENTITIES-001` -- 2 harness bins downstream
  from PROMPT 690 needing `HandUiEntities`.
- `S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001` -- split
  from PROMPT 680 PARTIAL closure.
- `S11-TD-FIXTURE-MESSAGES-002` -- wider exhaustive `add_message`
  sweep (Option B from PROMPT 708).
- `S11-TD-CI-NORMALIZE-COMMENTS-001` -- teach `normalize_source()`
  to strip Rust comments (Option B from PROMPT 674 FAIL report).

### PROMPT 803 §5 Should / Nice rows not pulled into Sprint 13/14/15/16

- `S13-LOBBY-CONFIRMCLASS-SENDER-001`,
  `S13-COOCCUPANCY-INVARIANT-001`,
  `S13-PHASE-IDEMPOTENCY-CLIENT-001`,
  `S13-ADR012-LOBBY-OPTIMISM-001`,
  `S13-S2C-SUCCESS-LOG-001`,
  `S13-OBSERVABLE-PRODUCER-AUDIT-001`,
  `S13-PLUGIN-REGISTRATION-INVARIANT-001`,
  `S13-IGNORE-ATTRIBUTE-DRIFT-001`,
  `S13-MANUAL-RUNBOOK-AUTOMATION-001` (gated on Sprint 13 story
  017 outcome; NOT authorised to advance `S8-QA-001-W1` in Sprint
  17),
  `S13-PROTO-MESSAGE-ID-001`.

### Tier 2 cosmetic captures bundle

12 already-tracked future-candidate slugs per PROMPT 802 §9
producer-decision-5 (preserved through Sprint 14 + Sprint 15 +
Sprint 16 deferral). Bundled candidate:
`S17-UX-CAPTURES-CLEAN-PASS-001` if a producer activates it. **Not
pulled into Sprint 17 draft.**

### Sprint 16 audit long-tail (PROMPT 1076 P3 not absorbed by Sprint 17 candidates)

AUDIT-1076-05 (P1 giant blurry `?` glyph behind modal during
DraftShop / DraftAuction / Placement) -- likely owned by
`client/src/presentation/board_rendering.rs` (objective unknown
asset) or `client/src/ui/hand/` (opponent hand back). Not pulled
into Sprint 17 because it may resolve as a side-effect of the
modal repair (S17-UI-MODAL-BLACK-SLAB-001); reassess after that
row lands. AUDIT-1076-08 (P1 "TO PLACE ART" placeholder on hand
card) -- card art binding gap likely resolved as a side-effect of
the PROMPT 1077 P0 bundle (S17-UI-CARD-DISPLAY-ART-HELPER-001)
existence check; reassess after that row lands. AUDIT-1076-11
(P2 Resolution phase has zero visualisation) -- degenerate with
AUDIT-1076-03 (server spawn loop, already on `origin/main` via
PROMPT 1079 + 1084); reassess after Sprint 17 modal repair + a
fresh user-test capture.

---

## Required Sprint 17 Story Docs

PROMPT 1090 (this draft) does NOT author any new story files.

The Sprint 17 Must Have HUD timer carry row is paperwork-carry-only:

- `S11-HUD-TIMER-EYEBALL-VISUAL-001` -- story file at
  `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
  ALREADY EXISTS on `origin/main` from Sprint 13 (PROMPT 822
  author / PROMPT 823 READY / PROMPT 1000 readiness rerun READY).
  No new story file required.

The other Must Have rows require story-authoring prompts BEFORE
Sprint 17 activation. Recommended next-launchable prompt sequence
after this draft lands on `origin/main`:

1. **PROMPT 1091 -- main-push refresh prompts (paperwork or worker
   re-run) for the four in-flight tracks**. Each track is its own
   prompt because the worker / integration state differs:
   - **PROMPT 1091a** -- PROMPT 1083 modal integration main-push
     refresh: rebase `e4bbca3` onto `fec13ff`, re-run the 10
     focused tests, push a new integration branch, paperwork-only
     main-push. If rebase conflicts, escalate to a worker re-run.
   - **PROMPT 1091b** -- PROMPT 1085 shop / auction surface paint
     integration prompt (worker `origin/work/client-shop-auction-
     surface-paint-1085` -> integration branch + tests + main-push
     paperwork). Sequenced AFTER PROMPT 1091a because the shop /
     auction repair depends on the modal repair landing.
   - **PROMPT 1091c** -- PROMPT 1086 client placement perspective
     integration prompt (worker `origin/work/client-placement-
     perspective-submit-1086` -> integration branch + tests +
     main-push paperwork). File-disjoint from the modal repair; may
     run in parallel with PROMPT 1091a.
   - **PROMPT 1091d** -- PROMPT 1087 / 1089 lobby class art main-
     push refresh: rebase `eec2a91` onto current `origin/main`,
     resolve placeholder-asset md5 gaps from PROMPT 1087 evidence,
     re-run the 49 tests, paperwork-only main-push. If rebase
     conflicts, escalate to a worker re-run.
2. **PROMPT 1092 -- story-authoring prompts for the Sprint 17 Must
   Have card-display-art helper bundle (S17-UI-CARD-DISPLAY-ART-
   HELPER-001) + the five Should Have rows + the three Nice to
   Have rows**. One prompt may author multiple sibling story files
   if file-disjoint, or one prompt per story if scope is unclear.
   Each story file MUST embed the source audit finding ID
   (AUDIT-1076-* or SOURCE-1077-*) and the minimal repair surface
   from the audit. Each story file MUST pass `/story-readiness`
   against Sprint 17 activation HEAD before `/dev-story` runs.
3. **PROMPT 1093 -- Sprint 17 activation**. Mirrors PROMPT 1064
   activation pattern: flip `production/sprint-status.yaml`
   top-level `sprint: 16 -> 17` and `status: closed-with-conditions
   -> active`; replace `stories:` block with Sprint 17 active set
   (depends on PROMPT 1091a-d outcomes -- conditional rows that
   landed before activation are dropped); replace
   `next_sprint_17_draft:` block at EOF with a
   `sprint_17_activation:` block; preserve `stage: Polish`
   verbatim. NO `production/stage.txt` touch. NO
   `production/gate-checks/*` touch. NO `production/qa/qa-plan-
   sprint-17.md` authoring (separate prompt). Refer to PROMPT 1064
   precedent (`c908f73`) for the exact paperwork shape.
4. **PROMPT 1094 -- Sprint 17 QA plan (`/qa-plan sprint-17`)**.
   Authored ONLY after Sprint 17 activation (PROMPT 1093). MUST
   reference each Sprint 17 active row's test evidence type
   (Logic / Integration / Visual / UI / Config-Data) per
   `.claude/docs/coding-standards.md` "Test Evidence by Story
   Type" matrix. NO `/dev-story` is authorised before the QA plan
   exists on `origin/main`.

The illustrative prompt body for PROMPT 1091a (representative of
the four PROMPT 1091a-d main-push refresh prompts) is:

```
PROMPT 1091a -- Client Phase Modal Black-Slab Main-Push Refresh

Paperwork-only main-push refresh for the PROMPT 1083 modal
integration. NOT a worker re-run unless rebase conflicts force one.

Repo: D:/_DEV/Work/Claude-Code-Game-Studios

Worktree:
D:/_DEV/claude-code-game-studios-worktrees/client-phase-modal-main-push-1091a

Branch (fresh integration off origin/main):
integrate/client-phase-modal-black-slab-1091a

Source of truth:
origin/main = fec13ffc3723d9d68afdda4b6e4bf62af5d6da2a
(PROMPT 1088 Sprint 16 close-out main integration tip; must contain
PROMPT 1082 close-out + Sprint 16 stories block update).

Inputs:
- origin/integrate/client-phase-modal-black-slab-1083 tip e4bbca3
  (PROMPT 1083 integration, 10 focused tests PASS at integration time)
- origin/work/client-phase-modal-black-slab-1080 tip cbc11b2
  (PROMPT 1080 worker)

Steps:
1. git fetch origin
2. git worktree add D:/_DEV/claude-code-game-studios-worktrees/
   client-phase-modal-main-push-1091a -b
   integrate/client-phase-modal-black-slab-1091a origin/main
3. cd into the worktree; git merge --no-ff
   origin/integrate/client-phase-modal-black-slab-1083 or
   git cherry-pick the two-file diff identified in
   reports/PROMPT-1083-client-phase-modal-black-slab-integration.md
   (client/src/ui/shop_auction/mod.rs +
   tests/integration/shop_auction_ui/draft_initial_centered_modal_layout_test.rs).
4. Re-run the 10 focused tests from PROMPT 1083 §"Focused test suite":
   cargo test -p client --test
   shop_auction_ui_draft_initial_centered_modal_layout_test ; ditto
   for the other 9 binaries.
5. git diff --check ; git diff --cached --check ; cargo fmt
   -p shared -- --check ; cargo fmt -p server -- --check ;
   cargo fmt -p client -- --check ; cargo fmt -p two-client-runtime
   -- --check.
6. Push the new integration branch only. Do not push main.

Cargo policy: set
CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0,
CARGO_INCREMENTAL=0, RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'.
Apply Windows / MSVC Cargo policy first if Cargo invocation is
required.

Deliverable: report at
reports/PROMPT-1091a-Client-Phase-Modal-Main-Push-Refresh.md with
branch, commit, changed files, focused test results, fmt results,
and final status line:
1091a: CLIENT-PHASE-MODAL-MAIN-PUSH-REFRESH: STATUS

Allowed writes: client/, server/, shared/, tests/ ONLY in the
worktree branch (Sprint 17 is NOT activated yet; Sprint 16 stories
block remains untouched). Forbidden writes: production/stage.txt,
production/sprint-status.yaml top-level sprint/status/stage,
production/qa/qa-plan-sprint-17.md, story-done row flips.

Final line: 1091a: CLIENT-PHASE-MODAL-MAIN-PUSH-REFRESH: STATUS
```

Analogous prompt bodies for PROMPT 1091b (shop / auction surface
paint integration), PROMPT 1091c (placement perspective
integration), and PROMPT 1091d (lobby class art main-push refresh)
follow the same shape with worker / integration branches swapped
in and the file-disjointness checks updated. Each is a separate
prompt so the orchestrator can serialise main-push paperwork and
respect the "Keep one shared-status writer active at a time"
rule.

---

## Non-Claims (verbatim preservation)

Sprint 17 MUST NOT claim any of the following at activation or at
close-out:

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`
  remains accepted-risk).
- Playtest / fun-hypothesis validation (`QA-COND-0006` remains
  accepted-risk).
- Full playable-client manual QA.
- Two-client `GAME_OVER` closure (`S8-QA-001-W1` remains OPEN;
  Sprint 13 story 017 AC12 forbid-auto-closure preserved through
  Sprint 13 / 14 / 15 / 16 / 17).
- Final-art / asset-production completion (`PAW-TD-*-a` accepted-
  risk preserved across PAW-002..PAW-006; Sprint 17 lobby class
  art row uses placeholder PNGs only).
- `Polish->Release` gate-check retry (PROMPT 761 `FAIL`
  preserved; **NO retry**).
- Stage advance from Polish to Release (`production/stage.txt`
  remains `Polish`).
- Underlying drag-runtime bug fix (Sprint 12 story 019
  `cannot-reproduce` preserved; **NOT bug-fixed**).
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` by an LLM
  `/story-done` (closure remains gated on human-operator
  screenshot capture).
- Pixel-level closure of PROMPT 1054 P1 UI snapshot visual retest
  (`BLOCKED-HUMAN-OPERATOR` preserved).
- Pixel-level QA snapshot capture for the Sprint 16 card-slot
  primitive shop-panel bundles at 1366x768 / 1920x1080 (story 009
  AC6 PARTIAL preserved).
- Closure of `S8-QA-001-W1` / `TQ-S12-C7`.
- Closure of the four per-surface card-slot migration siblings
  (`S17-UI-CARD-SLOT-MIGRATION-HAND-001` /
  `-DRAFT-GRID-001` / `-AUCTION-FEATURED-001` / `-BOARD-GHOST-001`)
  -- those remain Sprint 17+ Backlog.
- Closure of any of the 24 PROMPT 1022 QA snapshot audit findings.
- Closure of any PROMPT 1076 finding for which a concrete repair
  is not on `origin/main` at activation. Specifically: Sprint 17
  conditional Must Have rows are dropped if their in-flight repair
  lands before activation; Sprint 17 does NOT claim closure of an
  audit finding by reference to a non-merged integration branch.
- Closure of any PROMPT 1077 finding outside the Must Have
  S17-UI-CARD-DISPLAY-ART-HELPER-001 bundle (which covers
  SOURCE-1077-01 / 02 / 03 / 04) and Should Have
  S17-UI-CARD-SLOT-INSET-WIRING-001 (SOURCE-1077-06) +
  S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 (SOURCE-1077-08 / 09 / 16)
  + S17-UI-BID-BUTTON-PHASE-RACE-001 (SOURCE-1077-10).
- Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 row reopen.
- Sprint 16 close-out paperwork reopen or re-author.

If any of the above MUST be advanced, it requires its own scope,
its own story file, its own `/story-readiness` pass, and its own
QA plan reference -- never silently folded into another Sprint 17
row.

---

> **End of Sprint 17 DRAFT (PROMPT 1090, 2026-05-18).**
> Activation prompt is a separate explicit instruction. Until
> activation, top-level `sprint: 16 / status: closed-with-conditions
> / stage: Polish` remain verbatim in
> `production/sprint-status.yaml`; this file's banner is the only
> Sprint 17 marker on `origin/main`.
