# Sprint 16 -- ACTIVATED (Polish stage; Sprint 15 closed-with-conditions)

> **PROMPT 1064 ACTIVATED 2026-05-17.** Sprint 16 was activated from the
> PROMPT 1024 draft below by PROMPT 1064 paperwork-only activation against
> `origin/main@6f9308c9defcdeb1a29b20eecfa21d5ba8a3bbec` (PROMPT 1056
> Sprint 15 close-out tip: `closeout(s15): close Sprint 15 with deferred
> human visual conditions (PROMPT 1056)`). Activation mirrors the PROMPT
> 826 / 897 / 997 precedent: flipped top-level `sprint: 15 -> 16` and
> `status: closed-with-conditions -> active` in
> `production/sprint-status.yaml`; replaced the `stories:` block with the
> Sprint 16 4-row active set (1 Must Have + 1 Should Have + 2 Nice to
> Have); replaced the `next_sprint_16_draft:` block at EOF with a
> `sprint_16_activation:` block; preserved `stage: Polish` verbatim
> (`production/stage.txt` NOT touched). PROMPT 761 `Polish->Release`
> gate-check `FAIL` preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry**
> is in scope for Sprint 16. Sprint 16 is **NOT** a `Polish->Release`
> sprint.
>
> **Sprint 16 active rows (status: ready)**:
>
> 1. `S11-HUD-TIMER-EYEBALL-VISUAL-001` -- Must Have, ready,
>    human-operator-blocked Sprint 13 -> 14 -> 15 -> 16 carry; carried
>    unchanged from Sprint 15 close-out per the 2026-05-17 orchestrator
>    decision to defer human visual testing later; **MUST NOT block
>    non-human Sprint 16 development lanes**; no LLM `/story-done`
>    authorised; story file
>    `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
>    (PROMPT 822 author / PROMPT 823 + PROMPT 1000 READY).
> 2. `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` -- Should Have, ready, story
>    `production/epics/ui-clean-pass/story-009-ui-card-slot-primitive.md`
>    (PROMPT 1044 / PROMPT 1045 authored + integrated; PROMPT 1060 /
>    PROMPT 1063 repair + integrated); `/story-readiness` rerun against
>    activation HEAD pending before `/dev-story`.
> 3. `S15-OPS-APPCOMPAT-MANIFEST-001` -- Nice to Have, ready, story
>    `production/epics/devops/story-006-appcompat-manifest.md`
>    (PROMPT 1057 author + PROMPT 1062 integration);
>    `/story-readiness` rerun against activation HEAD pending before
>    `/dev-story`.
> 4. `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` -- Nice to Have, ready,
>    story
>    `production/epics/ui-clean-pass/story-016-workspace-dead-code-warning.md`
>    (PROMPT 1058 author + PROMPT 1061 integration);
>    `/story-readiness` rerun against activation HEAD pending before
>    `/dev-story`.
>
> **PROMPT 1064 explicitly does NOT claim**: public release readiness,
> release-candidate readiness, full game completion, broad /
> Standard-tier accessibility completion (`QA-COND-0005` remains
> accepted-risk), playtest validation (`QA-COND-0006` remains
> accepted-risk), full playable-client manual QA, two-client
> `GAME_OVER` closure (`S8-QA-001-W1` remains OPEN), final-art /
> asset-production completion (`PAW-TD-*-a` accepted-risk preserved),
> `Polish->Release` gate-check retry (PROMPT 761 `FAIL` preserved),
> stage advance from Polish to Release, underlying drag-runtime bug
> fix (Sprint 12 story 019 `cannot-reproduce` preserved), closure of
> `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-later carry; no LLM
> `/story-done` authorised; closure remains gated on human-operator
> screenshot capture), pixel-level closure of PROMPT 1054 P1 UI
> snapshot visual retest (`BLOCKED-HUMAN-OPERATOR` preserved), Sprint
> 15 row reopen (4 closed Sprint 15 rows preserved on origin/main),
> Sprint 14 / 13 / 12 / 11 / 10 row reopen, or closure of any of the
> 24 PROMPT 1022 QA snapshot audit findings (those remain report-only
> inputs to future story authoring; none are Sprint 16 active rows).
>
> **PROMPT 1064 paperwork-only activation scope**: changed only
> `production/sprint-status.yaml`, this file (`production/sprints/sprint-16.md`
> banner only; plan body NOT rewritten),
> `production/session-state/active.md` (PROMPT 1064 banner prepended
> above PROMPT 1056 banner), `production/session-state/codex-orchestrator-state.md`
> (PROMPT 1064 section prepended above PROMPT 1056 section), and
> `reports/PROMPT-1064-Sprint-16-Activation.md` (final report;
> `reports/` is gitignored). NO `client/` / `server/` / `shared/` /
> `tests/` / `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/` /
> `Trunk.toml` touch. NO `production/stage.txt` touch. NO `production/qa/*`
> touch. NO `production/gate-checks/*` touch. NO story file under
> `production/epics/` touch. NO `/story-readiness`, `/dev-story`,
> `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
> `/release-check`, `/qa-plan` run by PROMPT 1064; NO cargo/trunk
> invocation. Sprint 16 QA plan does NOT exist at activation; must be
> authored via `/qa-plan sprint-16` BEFORE any `/dev-story` runs and
> BEFORE any Sprint 16 close-out claim.

---

> **PROMPT 1024 paperwork-only Sprint 16 plan draft (2026-05-17)**.
> Source-of-truth at authoring: `origin/main@a53a33820789b0c4dd8d390963db5a3ef59250f9`
> (PROMPT 1020 Sprint 15 QA-snapshot auto-capture + F9 integration tip:
> `integrate(s15): apply QA snapshot auto-capture + F9 shortcut (PROMPT 1020)`).
> Worktree: `D:/_DEV/claude-code-game-studios-worktrees/sprint-16-plan-draft`.
> Branch: `sprint-plan/sprint-16-draft`.
>
> **Status**: `draft -- authored 2026-05-17 by PROMPT 1024`. **Sprint 16 is NOT
> activated by this draft.** **Sprint 15 remains `active`**; this draft does NOT
> close Sprint 15. Activation of Sprint 16 is a separate explicit prompt that
> mirrors the PROMPT 826 / PROMPT 897 / PROMPT 997 pattern: it requires Sprint 15
> close-out to have landed on `origin/main` first (which has NOT happened at
> draft time), then flips `production/sprint-status.yaml` top-level
> `sprint: 15 -> 16` and `status: closed-with-conditions -> active`, appends a
> `sprint_16_activation:` block, and adds an ACTIVATED banner to this file.
> PROMPT 1024 itself does NOT activate Sprint 16 and does NOT close Sprint 15.
>
> **Stage**: `Polish` (UNCHANGED). `production/stage.txt` NOT modified by this
> draft and MUST NOT be modified by activation. PROMPT 761 `Polish->Release`
> gate-check `FAIL` evidence preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry** is in
> scope for Sprint 16 and MUST NOT be attempted by activation. Sprint 16 is
> **NOT** a `Polish->Release` sprint.
>
> **Provisional start / end (locked at activation)**: 2026-08-13 -> 2026-08-26
> (10 workdays). Continuous follow-on to Sprint 15 (2026-07-30 -> 2026-08-12).
>
> **Sprint 15 disposition at draft time**: `active`. Per
> `production/sprint-status.yaml` and `production/session-state/active.md` at
> draft time, Sprint 15 is **effectively development-complete**: 4 of 5 active
> rows are closed (S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP Must,
> S12-UX-HAND-DRAG-STATE-VISUALS-001 Should, S11-UX-BOARD-RENDERING-SPEC Should,
> S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 Nice). The single un-closed row is
> `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have, story 014, 0.25d,
> human-operator-blocked Sprint 13 -> 14 -> 15 carry; promoted Should -> Must
> in Sprint 15 by PROMPT 988). Closure remains gated on real human-operator
> screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement`
> 10-12s phases per the story AC matrix. No LLM `/story-done` is authorised.
>
> **Live QA / debugging / snapshot phase is non-blocking for Sprint 16
> planning.** Per the 2026-05-17 orchestrator instruction, the QA snapshot
> audit and tooling integration tracks running in parallel at draft time are
> **explicitly NOT blockers** for Sprint 16 planning and are NOT represented
> as Sprint 15 close-out:
>
> - **PROMPT 1019 / 1020** (`integrate(s15): apply QA snapshot auto-capture + F9 shortcut`)
>   landed on `origin/main@a53a338`. Auto-capture + F9 shortcut available in dev
>   builds.
> - **PROMPT 1022** QA snapshot visual + state audit (read-only): produced
>   `reports/PROMPT-1022-qa-snapshot-visual-state-audit.md` (CONCERNS verdict;
>   24 findings across 5 P1, 6 P2, 5 P3, 6 state-mismatch / instrumentation,
>   2 snapshot-tool). Report-only; not integrated as production-source change.
>   Findings inform future story authoring but are **NOT pulled as Sprint 16
>   rows in this draft** because no story files exist yet.
> - **PROMPT 1021 / 1023** QA snapshot default-dev integration (code-only):
>   landed on `origin/main` (commits `10057dc` worker / `7b663df` integration).
>   QA snapshot is enabled by default in dev builds.
>
> None of these tracks constitute Sprint 15 close-out evidence. None advance
> Sprint 15 stage. None modify the Sprint 15 `S11-HUD-TIMER-EYEBALL-VISUAL-001`
> human-operator-blocked carry disposition.
>
> **Sprint 15 close-out remains pending** at draft time. Whether Sprint 15
> closes-with-conditions, carries the HUD timer row into Sprint 16, or closes
> outright after a human-operator capture session is a producer / orchestrator
> decision that is **NOT made by this draft**. This draft assumes the most
> likely outcome -- Sprint 15 closes-with-conditions with the HUD timer row
> carried into Sprint 16 -- and structures Sprint 16 around that assumption,
> with explicit conditional language if the row closes on `origin/main` by a
> separate prompt before Sprint 16 activation.
>
> **Sprint 14 / Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts**
> preserved unchanged. `S8-QA-001-W1` OPEN, `QA-COND-0005` + `QA-COND-0006`
> accepted-risk, `PAW-TD-*-a` accept-risk across PAW-002..PAW-006,
> `TQ-S12-C1..C7` preserved verbatim, PROMPT 683-era runtime divergence
> question preserved (no third same-scope retest per `TQ-S12-C2`), Sprint 12
> story 019 underlying drag-runtime bug NOT claimed fixed (cannot-reproduce
> preserved).
>
> **Sprint 16 explicitly does NOT claim**: public release readiness,
> release-candidate readiness, full game completion, broad / Standard-tier
> accessibility completion, playtest validation, full playable-client manual QA,
> two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN), final-art /
> asset-production completion, `Polish->Release` gate-check retry, stage
> advance from Polish to Release, underlying drag-runtime bug fix (Sprint 12
> story 019 closed cannot-reproduce, NOT bug-fixed), Sprint 14 / Sprint 13 /
> Sprint 12 / Sprint 11 / Sprint 10 row reopen, `S8-QA-001-W1` closure,
> `TQ-S12-C7` closure, Sprint 15 close-out, **closure of any of the 24
> PROMPT 1022 audit findings** (these are inputs to future story authoring,
> not Sprint 16 rows), closure of the Sprint 15 `S11-HUD-TIMER-EYEBALL-VISUAL-001`
> human-operator-blocked row by an LLM, or full UI clean-pass repair beyond
> the candidate rows below.
>
> **PROMPT 1024 paperwork-only draft scope**: NO `/dev-story`, NO
> `/story-readiness`, NO `/story-done`, NO `/smoke-check`, NO `/team-qa`,
> NO `/gate-check`, NO `/release-check`, NO `/qa-plan`, NO Sprint 16
> activation, NO Sprint 15 close-out, NO `production/qa/qa-plan-sprint-16.md`
> authored, NO stage advance, NO implementation, NO CI run, NO `cargo` /
> `trunk` invocation, NO touch of `client/` / `server/` / `shared/` /
> `tests/` / `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/`. Files
> allowed: this file (NEW), `production/sprint-status.yaml`
> (`next_sprint_16_draft:` block appended only at EOF; Sprint 15 active rows
> and all prior closeout blocks NOT modified; top-level `sprint: 15 /
> status: active / stage: Polish` preserved verbatim),
> `production/session-state/active.md` (PROMPT 1024 banner prepended),
> `production/session-state/codex-orchestrator-state.md` (PROMPT 1024
> section prepended).

---

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`. Sprint 16
  does NOT advance stage. Sprint 16 is NOT a `Polish->Release` sprint.
- Sprint 15 remains `active` at draft time. 4 of 5 Sprint 15 rows are closed
  on `origin/main` (PROMPT 1009 batch + PROMPT 1010 paperwork). The single
  un-closed row is `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have; human-
  operator-blocked Sprint 13 -> 14 -> 15 carry). Sprint 16 planning explicitly
  preserves this row as a **conditional Sprint 15 -> Sprint 16 carry**: if it
  closes on `origin/main` between this draft and Sprint 16 activation (e.g.
  via a producer-scheduled human-operator capture session), the carry is
  dropped from Sprint 16 at activation; if it remains open, it is pulled
  forward into Sprint 16 as a Must Have row unchanged.
- The live QA snapshot tooling phase (PROMPT 1019 / 1020 + PROMPT 1021 /
  1023 + PROMPT 1022 audit) is **explicitly treated as parallel side-track
  work**, not as a blocker for Sprint 16 planning. The audit findings inform
  future story authoring but are NOT pulled as Sprint 16 active rows here;
  pulling 24 audit findings into Sprint 16 would inflate the sprint into a
  mega-sprint and conflict with the "small, executable plan" framing.
- This draft pulls a **deliberately small** Sprint 16 scope. Per the 2026-
  05-17 PROMPT 1024 instruction ("Prefer a small, executable plan over a
  broad mega-sprint"), the plan covers (1) the Sprint 15 carry (HUD timer
  eyeball check, conditional), (2) the deferred Tier 3 rank 13 UI primitive
  refactor that Sprint 15 explicitly deferred (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`,
  1.5d), and (3) two small ops / test-harness hygiene rows pre-identified
  by Sprint 14 PROMPT 983 smoke + Sprint 15 deferred backlog. Heavier
  candidates (24 PROMPT 1022 audit findings; server-hardening backlog from
  Sprint 11/12/13; per-surface migration of the Sprint 15 interaction-state
  primitive module to Tier 1 button surfaces) are deliberately **deferred
  to Sprint 17+ backlog**, not promoted into Sprint 16.
- Sequencing is governed by the canonical reconciliation roadmap at
  `docs/ux/ui-clean-pass-roadmap.md` (PROMPT 838) and by the Sprint 15
  draft / activation evidence (PROMPT 988 plan / PROMPT 997 activation).
  Tier 0 (ranks 1-6) and Tier 1 (ranks 7-12) are DONE on `origin/main`.
  Tier 1 Should-priority adjacent rows that were not pulled into Sprint 14
  landed in Sprint 15 (`S12-UX-HAND-DRAG-STATE-VISUALS-001` DONE;
  `S11-UX-BOARD-RENDERING-SPEC` Tier 3 rank 14 doc-only spec DONE). The
  Tier 0 Should-priority adjacent row `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`
  also landed in Sprint 15. The **last remaining canonical roadmap row** is
  Tier 3 rank 13 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` -- explicitly deferred
  from Sprint 15 to Sprint 16 by PROMPT 988 plan section "Deliberately
  deferred to Sprint 16+" because pulling it would inflate Sprint 15 into a
  mega-sprint (1.5d + ~0.5d authoring + ~0.5d integration friction). It is
  the headline Sprint 16 candidate.
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 16 QA plan exists at draft time. A Sprint 16 QA plan
  (`production/qa/qa-plan-sprint-16.md`) MUST be authored via `/qa-plan
  sprint-16` **after** Sprint 16 activation **and after** each Sprint 16
  story file passes `/story-readiness` against activation HEAD. No
  `/dev-story` is authorised before the QA plan exists. PROMPT 1024 does
  NOT author the QA plan.
- Sprint 16 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
  validation (`QA-COND-0006`), full playable-client manual QA, two-client
  GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
  completion (`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage
  advance from Polish to Release, or underlying drag-runtime bug fix
  (Sprint 12 story 019 remains `closed-with-conditions / cannot-reproduce`).
  None of these can be added to Sprint 16 by activation; each requires its
  own scope and gate evidence.

## Entry Conditions (must be true at activation)

- `production/sprint-status.yaml` top-level reads `sprint: 15`,
  `status: "closed-with-conditions"` (this requires Sprint 15 close-out to
  have landed on `origin/main` via a separate paperwork prompt mirroring
  the PROMPT 817 / 894 / 987 pattern; **NOT performed by this draft**).
- `production/stage.txt` reads `Polish` (UNCHANGED).
- PROMPT 761 `Polish->Release` gate-check `FAIL` evidence preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
- Sprint 15 disposition (`closed-with-conditions` expected per the same
  pattern as Sprint 10 / 11 / 12 / 13 / 14) preserved unchanged once
  Sprint 15 close-out lands. Sprint 14 / Sprint 13 / Sprint 12 / Sprint
  11 / Sprint 10 closeouts preserved unchanged.
- `S8-QA-001-W1` OPEN. `QA-COND-0005` + `QA-COND-0006` accepted-risk.
  `PAW-TD-*-a` accept-risk across PAW-002..PAW-006. PROMPT 683-era runtime
  divergence question preserved (no third same-scope retest per
  `TQ-S12-C2`). `TQ-S12-C1..C7` preserved verbatim.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` disposition at Sprint 16 activation
  determines the Must Have row count: if still `ready` (human-operator-
  blocked), the row is carried into Sprint 16 as Sprint 13 -> 14 -> 15 ->
  16 carry; if closed on `origin/main` by a separate prompt before Sprint
  16 activation, the row is dropped from Sprint 16 and Must Have shrinks
  by one row.
- The Sprint 16 candidate story files for the Should + Nice rows below do
  NOT yet exist on `origin/main` at draft time. **Story authoring prompts
  are a prerequisite to Sprint 16 activation** for the three net-new
  candidate rows. The HUD timer Must Have carry uses the existing story
  014 file unchanged.

If any entry condition fails, Sprint 16 does NOT activate; producer must
revise scope before activation.

## Sprint Goal

Sprint 16 is a **deferred-backlog closeout sprint**: it discharges the last
remaining canonical UI clean-pass roadmap row (Tier 3 rank 13, the only
roadmap row not yet DONE) plus two small ops / test-harness hygiene rows
identified by Sprint 14 PROMPT 983 smoke. It is **NOT** a release sprint
and explicitly is not gated on Sprint 15 manual QA closure; the live QA
snapshot tooling phase runs in parallel and informs (but does not block)
Sprint 16. The goal is:

1. **Close the Sprint 13 -> 14 -> 15 -> 16 human-operator-blocked HUD
   timer carry** (`S11-HUD-TIMER-EYEBALL-VISUAL-001`) if it has not
   already closed on `origin/main` by a separate prompt before Sprint 16
   activation. The story file is already READY; closure remains gated on
   human-operator action.
2. **Land the deferred Tier 3 rank 13 UI primitive refactor**
   (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) per `docs/ux/ui-clean-pass-roadmap.md`
   rank 13 / §"Sequencing Rules" / §"Tier 3 (ranks 13-14) lands last".
   Refactor touches hand + shop + auction together (per PROMPT 802 §8);
   was explicitly deferred from Sprint 15 by PROMPT 988 plan section
   "Deliberately deferred to Sprint 16+". With Tier 1 surfaces stable
   (Sprint 14 ranks 7-12 DONE) and Sprint 15 hand-drag-state visuals DONE,
   the refactor's dependencies are satisfied.
3. **Discharge two small ops / test-harness hygiene rows** pre-identified
   by Sprint 14 PROMPT 983 smoke and Sprint 15 deferred backlog:
   - **`S15-OPS-APPCOMPAT-MANIFEST-001`** -- embed a Windows manifest with
     `level="asInvoker"` on the `spawn_range_live_update_contract` test
     binary (Option B from PROMPT 983 §"Windows AppCompat Workaround"),
     removing the per-run rename workaround.
   - **`S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`** -- remove the
     `count_with_image_node` pre-existing dead-code warning at
     `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43`
     surfaced by Sprint 14 PROMPT 983 smoke and preserved through Sprint
     15.

Sprint 16 does NOT claim release readiness, broad accessibility
completion, full playable-client manual QA, playtest validation, final-
art / asset-production completion, S8-QA-001-W1 closure, full game
completion, two-client GAME_OVER closure, a Polish->Release retry,
closure of the underlying drag-runtime bug from Sprint 12 story 019, or
closure of any of the 24 PROMPT 1022 audit findings (those inform future
sprint planning, not Sprint 16). The 24 PROMPT 1022 audit findings, the
12 Tier 2 cosmetic-capture future candidates, and the long server-
hardening backlog from Sprint 11/12/13 are deferred to Sprint 17+
**explicitly**.

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10/11/12/13/14/15)
- Buffer (20%): 2 days reserved for (a) per-row `/story-readiness` re-runs
  against Sprint 16 activation HEAD; (b) the three net-new story-authoring
  prompts (Should #1 card-slot, Nice #2 AppCompat manifest, Nice #3
  dead-code warning) that must precede `/dev-story`; (c) `/qa-plan
  sprint-16` authoring; (d) producer / human-operator scheduling friction
  on the human-operator-blocked Must Have carry if still open at activation.
- Available: **8 effective planned days**
- Planned Must Have scope: **~0.25 estimated days** (conditional Sprint 15
  carry HUD timer 0.25d human-operator-blocked; drops to 0d if the row
  closes on `origin/main` before activation). This is intentionally non-
  implementation; the implementation work is in Should + Nice.
- Should Have scope: **~1.5 estimated days** (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`
  1.5d multi-surface refactor). Headline Sprint 16 development row.
- Nice to Have scope: **~0.35 estimated days** (AppCompat manifest 0.25d +
  dead-code warning 0.1d). Both land only if Must + Should closure is on
  track.
- Total implementation effort: **~1.85 days against 8 days available**.
  This is a deliberately small plan; the remaining ~6 days of capacity
  absorbs story-authoring overhead, `/story-readiness` reruns, `/qa-plan
  sprint-16` authoring, integration / `/story-done` paperwork, and
  human-operator scheduling for the Must Have carry. If burn-down comes
  in significantly under capacity, a producer may pull a single additional
  row from the Sprint 16 Backlog section (priority: one of the PROMPT
  1022 P1 audit findings split into a single-surface story; do NOT pull
  all five P1 findings simultaneously without separate sprint scoping;
  do NOT pull the per-surface migration of the Sprint 15 interaction-
  state primitive module without a separate scope decision).

---

## Tasks

> All IDs below are **draft Sprint 16 candidate** tickets. They are NOT yet
> active `production/sprint-status.yaml` rows. Promotion to active rows
> happens at activation via a separate explicit prompt (mirrors the
> PROMPT 826 / PROMPT 897 / PROMPT 997 pattern), after Sprint 15 close-out
> lands on `origin/main`. All slug provenance and rank references are
> against `docs/ux/ui-clean-pass-roadmap.md` and the Sprint 15 draft
> (PROMPT 988) "Deliberately deferred to Sprint 16+" section.

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD Timer Eyeball Visual Check (Sprint 13 -> 14 -> 15 -> 16 carry; **human-operator-blocked**) -- manual 2-client run validating timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases. **Conditional row: dropped at Sprint 16 activation if closed on `origin/main` by a separate prompt before activation.** | UI programmer + **human operator** | 0.25 | **Sprint 15 carry** per PROMPT 988 plan + PROMPT 997 activation; originally Sprint 10 smoke retry-7 W2 -> Sprint 11 -> Sprint 12 -> Sprint 13 -> Sprint 14 -> Sprint 15 -> Sprint 16 carry. Story file at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` (PROMPT 822 author / PROMPT 823 READY / PROMPT 1000 readiness rerun READY). The QA snapshot tooling that landed in Sprint 15 (PROMPT 1019/1020/1021/1023) provides the auto-capture path the human operator may use, but does NOT auto-close this row. | Story 014 `/story-readiness` confirms READY against Sprint 16 activation HEAD (expected READY per PROMPT 823 / 1000 unless the story file has been touched). Manual 2-client run + screenshot evidence in `production/qa/evidence/sprint-16-hud-timer-visual-check/` (NEW; or `production/qa/evidence/sprint-15-hud-timer-visual-check/` if a Sprint 15 close-out chose to keep the carry under the Sprint 15 evidence path; producer choice at activation). Cosmetic verification only; no production-code change unless an actual visual regression is found and a follow-on story is authored. **Closure remains gated on human-operator screenshot capture** -- no LLM `/story-done` is authorised. Does NOT claim `QA-COND-0005` Standard-tier accessibility completion. `PAW-TD-*-a` preserved. If human-operator time cannot be scheduled within Sprint 16, the row carries forward as a Sprint 16 -> Sprint 17 carry; this is acceptable but the producer must document the cause in Sprint 16 close-out. |

**Must Have subtotal**: ~0.25 estimated days (conditional; drops to 0d if
the row closes on `origin/main` before Sprint 16 activation). The Must
Have row is intentionally non-implementation: the carry is human-
operator-blocked manual screenshot capture only. Sprint 16 implementation
capacity flows into the Should + Nice rows below.

### Should Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S12-TD-UI-CARD-SLOT-PRIMITIVE-001 | UI Card-Slot Primitive (Tier 3 rank 13, **last remaining canonical UI clean-pass roadmap row**) -- author a shared card-slot primitive consumed by hand / shop / auction surfaces, replacing per-surface inline slot rendering | UI programmer + ux-designer | 1.5 | Roadmap rank 13 (Tier 3, Should, **net-new**, PROMPT 802 §3.3 HA1 / §3.3 HA5 / §4 Tier 3.1); **deferred from Sprint 15 by PROMPT 988** plan section "Deliberately deferred to Sprint 16+" because pulling it would inflate Sprint 15 into a mega-sprint (1.5d + ~0.5d authoring + ~0.5d integration friction). Tier 1 surfaces (ranks 7-12) DONE in Sprint 14; Sprint 15 hand-drag-state visuals DONE; the refactor's dependencies are satisfied. **Story file NEW; must be authored before activation** at `production/epics/ui-clean-pass/story-XXX-ui-card-slot-primitive.md` (slug TBD by story-authoring prompt; mirrors PROMPT 991 / 992 / 993 pattern for Sprint 15 candidate authoring). | Story file authored on `origin/main` via a separate story-authoring prompt before Sprint 16 activation. `/story-readiness` passes against Sprint 16 activation HEAD. Card-slot primitive module authored under `client/src/ui/design_tokens/` or `client/src/ui/primitives/` (path TBD by story-authoring prompt; either location is consistent with existing Sprint 14 Tier 0 modules). Primitive consumed by **at least one** of hand / shop / auction surfaces in Sprint 16 (full per-surface migration of all three may exceed Sprint 16 capacity and may carry to Sprint 17; producer + UI programmer scope at story-authoring time). New integration test in `tests/integration/ui_clean_pass/` (or equivalent path; TBD) asserts primitive module shape and at least one consumer-surface migration. **Refactor of Sprint 14 Tier 1 surfaces (ranks 7-12 DONE) is incremental, not a full rewrite**: existing inline slot rendering must continue to work during the migration; primitive is introduced additively and consumer migration is per-surface. No protocol-shape change; no new server-authoritative state. `PAW-TD-*-a` preserved across PAW-002..PAW-006 (no final-art replacement). `QA-COND-0005` + `QA-COND-0006` preserved (card-slot primitive does NOT advance Standard-tier accessibility or playtest validation). |

**Should Have subtotal**: ~1.5 estimated days. The headline Sprint 16
development row. Requires a story-authoring prompt before activation and
a `/story-readiness` pass before `/dev-story`.

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S15-OPS-APPCOMPAT-MANIFEST-001 | Windows AppCompat manifest for `spawn_range_live_update_contract` test binary -- embed a Windows manifest with `level="asInvoker"` on the test binary to disable AppCompat installer-detection heuristic (OS error 740) | devops-engineer + UI programmer | 0.25 | Sprint 14 PROMPT 983 smoke §"Windows AppCompat Workaround" Option B; preserved into Sprint 15 deferred backlog by PROMPT 988 plan §"Smoke evidence hygiene". Removes the per-run rename workaround the smoke harness currently uses. **Story file NEW; must be authored before activation** at `production/epics/ops/story-XXX-appcompat-manifest.md` (slug TBD by story-authoring prompt). | Story file authored on `origin/main` via a separate story-authoring prompt before Sprint 16 activation. `/story-readiness` passes against Sprint 16 activation HEAD. Windows manifest embedded on the `spawn_range_live_update_contract-*.exe` test binary path (likely via `embed-resource` crate or `Cargo.toml` build script directive; exact mechanism TBD by story-authoring prompt). Test binary launches without AppCompat OS error 740 across 5 consecutive runs without the rename workaround. Smoke harness updated to drop the per-run rename workaround (or keep it as a documented fallback for non-MSVC builds). Cargo manifest delta scoped to test-binary configuration; no production-code shape change. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |
| S15-TD-WORKSPACE-DEAD-CODE-WARNING-001 | Workspace dead-code warning cleanup -- remove the `count_with_image_node` pre-existing dead-code warning at `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43` surfaced by Sprint 14 PROMPT 983 smoke | UI programmer | 0.1 | Sprint 14 PROMPT 983 smoke pre-existing warning preserved through Sprint 15; PROMPT 988 plan §"Smoke evidence hygiene" candidate. Trivial hygiene cleanup. **Story file NEW; must be authored before activation** at `production/epics/ui-clean-pass/story-XXX-workspace-dead-code-warning.md` (or similar; slug TBD by story-authoring prompt). | Story file authored on `origin/main` via a separate story-authoring prompt before Sprint 16 activation. `/story-readiness` passes against Sprint 16 activation HEAD. Either the `count_with_image_node` helper is removed (if truly unused) or it is wired into an existing test assertion (if the original author intended it to be live). `cargo check` and the smoke binary set produce zero dead-code warnings across `tests/integration/presentation/`. No production-code shape change; no test-coverage regression. `PAW-TD-*-a` + `QA-COND-0005` + `QA-COND-0006` preserved. |

**Nice to Have subtotal**: ~0.35 estimated days. Both land only if Must
Have + Should Have closure is on track. Trivially small; the AppCompat
manifest pays back per-run developer friction while the dead-code warning
is a single-line / single-function cleanup.

---

## Carryover from Sprint 15

| Source row (Sprint 15) | Disposition into Sprint 16 |
|------------------------|----------------------------|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 15 Must Have, `ready` after PROMPT 1009 + PROMPT 1010 closure of the other 4 rows -- the only un-closed row of Sprint 15; human-operator-blocked cosmetic visual check; Sprint 13 -> 14 -> 15 carry) | **Conditional Sprint 15 -> Sprint 16 carry**: pulled forward as Sprint 16 **Must Have** human-operator-blocked carry **only if** the row remains `ready` on `origin/main` at Sprint 16 activation. If a producer-scheduled human-operator capture session closes the row on `origin/main` between this draft and Sprint 16 activation, the row is dropped from Sprint 16 (Must Have shrinks to 0 rows). Disposition preserved unchanged: closure remains gated on human screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases; no LLM `/story-done` is authorised; PROMPT 822 / PROMPT 823 / PROMPT 894 / PROMPT 987 / PROMPT 988 / PROMPT 997 disposition preserved. Story file unchanged at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`. Evidence target shifts from `production/qa/evidence/sprint-15-hud-timer-visual-check/` (NEW, unpopulated at draft time) to `production/qa/evidence/sprint-16-hud-timer-visual-check/` (NEW) **if** Sprint 15 close-out preserves the carry path; producer chooses at activation. |
| All 4 closed Sprint 15 rows (`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` Must, `S12-UX-HAND-DRAG-STATE-VISUALS-001` Should, `S11-UX-BOARD-RENDERING-SPEC` Should, `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` Nice) | Preserved unchanged on `origin/main` per PROMPT 1009 + PROMPT 1010 closures. None reopened or revisited by Sprint 16. |
| Sprint 15 QA snapshot tooling tracks (PROMPT 1019/1020/1021/1023 landed; PROMPT 1022 audit report-only) | Preserved unchanged on `origin/main`. Sprint 16 does NOT reopen, re-integrate, or re-audit these tracks. The 24 PROMPT 1022 findings are inputs to future story authoring (deferred to Sprint 17+); they are NOT Sprint 16 active rows. |

(All Sprint 15 closed rows + all 16 closed Sprint 14 rows + all closed
Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 rows are preserved unchanged
on `origin/main`. None are reopened or revisited by Sprint 16.)

## Conditions Carried Forward Unchanged (NOT closed by Sprint 16)

Sprint 16 explicitly preserves and does NOT claim closure for any of:

- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains
  **OPEN**. Sprint 13 story 017 AC12 forbid-auto-closure was preserved
  through Sprint 13, Sprint 14, and Sprint 15; Sprint 16 candidate rows
  do not touch the two-client GAME_OVER surface. Sprint 16 activation
  MUST NOT silently close `S8-QA-001-W1`.
- **`QA-COND-0005`** -- Standard-tier accessibility remains **accepted-risk**
  (friend-game scope only). Sprint 16 card-slot primitive refactor and
  AppCompat manifest and dead-code warning cleanup are **friend-game
  visual polish / ops hygiene only** per roadmap §"Friend-Game Scope vs
  Standard-Tier-Accessibility Scope Boundary". The L5 `LOBBY_BUTTON_HEIGHT
  = 30.0` defect (PROMPT 802 §3.1 L5) remains accepted-risk under
  `QA-COND-0005`. Sprint 16 does NOT pursue WCAG contrast ratios, ≥44px
  hit-targets, full keyboard navigation, screen reader support,
  colorblind modes, or text scaling.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. Sprint 16 card-slot primitive refactor
  does NOT advance playtest validation even when the surface is visibly
  polished.
- **`PAW-TD-*-a`** -- placeholder-art accept-risk across PAW-002..PAW-006
  remains in place. Sprint 16 layout / composition / primitive work
  does NOT advance placeholder-art resolution; PROMPT 802 §7 places
  final-art work explicitly out of audit scope.
- **PROMPT 683-era runtime divergence question** -- folded into Sprint 12
  story 019 `closed-with-conditions / cannot-reproduce` (after second
  time-box exhaustion). Sprint 16 does NOT claim this question closed.
  **A third same-scope retest is NOT authorised** per `TQ-S12-C2`.
- **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. **NO retry**
  is in scope for Sprint 16. Stage remains `Polish`.
- **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed
  fixed. Sprint 16 card-slot primitive refactor is additive UI work and
  does not reproduce or fix the underlying drag-runtime behaviour.
- **`TQ-S12-C1..C7`** -- all 7 Sprint 12 Team-QA conditions preserved
  verbatim. `TQ-S12-C7` explicitly NOT closed by any Sprint 16 row.
- **Sprint 15 / Sprint 14 / Sprint 13 / Sprint 12 / Sprint 11 / Sprint
  10 closeouts** -- preserved unchanged once each lands. Sprint 15
  close-out is **pending** at draft time; the expected disposition is
  `closed-with-conditions` (same accept-risk conditions carry forward)
  with `S11-HUD-TIMER-EYEBALL-VISUAL-001` carrying into Sprint 16 if
  still open. Sprint 14 disposition `closed-with-conditions` per PROMPT
  987; Sprint 13 `closed-with-conditions` per PROMPT 894; Sprint 12
  `closed-with-conditions` per PROMPT 817; Sprint 11
  `closed-with-conditions` per PROMPT 792; Sprint 10
  `closed-with-conditions` per PROMPT 763.
- **All closed `/story-done` closures** across Sprint 10 -> 15 preserved
  unchanged on `origin/main`. Sprint 16 does NOT reopen any of them.
- **24 PROMPT 1022 audit findings** -- preserved as report-only inputs.
  None are pulled as Sprint 16 active rows. None are claimed closed by
  Sprint 16. Each future Sprint 17+ row that pulls one or more findings
  requires its own story file authored via a separate story-authoring
  prompt, its own `/story-readiness` pass, and its own QA plan reference.
- **Live QA snapshot tooling phase** (PROMPT 1019 / 1020 / 1021 / 1023)
  preserved on `origin/main` unchanged. Sprint 16 does NOT re-integrate,
  re-audit, or re-tool the QA snapshot path.

If any condition above changes during Sprint 16, it requires its own
separate story file and explicit disposition -- it cannot be silently
folded into another Sprint 16 row.

## Wider Sprint 16 Backlog (NOT scheduled into this draft; deferred to Sprint 17+)

The following candidates remain in the broader backlog and are **NOT
scheduled** into this Sprint 16 draft. They may be pulled by a producer
revision before activation (priority: a single P1 PROMPT 1022 finding
split into a single-surface story; trivially-small ops hygiene), or
deferred further to Sprint 17+:

### PROMPT 1022 QA snapshot audit findings (24 total; deferred to Sprint 17+)

`reports/PROMPT-1022-qa-snapshot-visual-state-audit.md` produced 24
findings across 5 P1, 6 P2, 5 P3, 6 state-mismatch / instrumentation,
and 2 snapshot-tool. **None are pulled as Sprint 16 active rows.** Each
future Sprint 17+ row that pulls one or more findings requires its own
story file and its own `/story-readiness` pass. P1 findings (highest
priority for future scoping):

- **F-P1-01** DraftShop phase shows no shop UI (Surface E,
  `client/src/ui/shop_auction/mod.rs`).
- **F-P1-02** Photosensitivity Warning fires inside an active game
  session (Surface B, `client/src/ui/photosensitivity_warning.rs` + z
  -order policy).
- **F-P1-03** HUD glyph duplication / ghost overlay across every gameplay
  phase (Surface A, `client/src/ui/hud/mod.rs`).
- **F-P1-04** Auction bid buttons show "?" placeholder glyphs (Surface F,
  `client/src/ui/shop_auction/mod.rs`).
- **F-P1-05** DraftInitial keep-9 modal has no opaque backdrop (Surface
  C, `client/src/ui/shop_auction/mod.rs` shared with E and F).

P2 / P3 / state-mismatch / snapshot-tool findings deferred with the P1
findings for unified Sprint 17+ scoping. Surfaces C / E / F all share
`client/src/ui/shop_auction/mod.rs`, so a Sprint 17+ activation that
pulls more than one shop-auction surface must either sequence them or
split the module first.

### Per-surface migration of Sprint 15 interaction-state primitive module

The Sprint 15 `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` row authored
the primitive module under `client/src/ui/design_tokens/` but explicitly
deferred per-surface migration of existing Sprint 14 button surfaces
(lobby buttons per `S11-UX-LOBBY-BUTTON-HITTARGETS` DONE; auction bid
buttons per `S11-UX-AUCTION-FEATURED-CARD` DONE; HUD action buttons per
`S11-UX-HUD-TOP-STRIP-LAYOUT` DONE) to Sprint 16+ per story 008 AC10.
Per-surface migration is **NOT pulled into Sprint 16 draft**; it would
inflate the sprint. Producer may pull a single per-surface migration row
at activation if Sprint 16 burn-down comes in under capacity.

### Tier 2 cosmetic captures bundle

12 already-tracked future-candidate slugs per PROMPT 802 §9 producer-
decision-5 (preserved through Sprint 14 + Sprint 15 deferral). Bundled
candidate: `S16-UX-CAPTURES-CLEAN-PASS-001` (per roadmap §"Tier 2
Cosmetic / Eyeball Captures") if a producer activates it. **Not pulled
into Sprint 16 draft.**

### Server hardening test parity (Sprint 11/12 backlog)

- **`S11-TD-NET-001`, `S11-TD-NET-002`, `S11-TD-NET-003`** -- server
  hardening test parity from Sprint 11/12 backlog. Defer to a focused
  server-hardening sprint.
- **`S11-TD-PRISM-COV-001`** -- Cluster 2C advisory coverage gap on
  `S2CPrismRewardDropped` + `S2CPrismRespawned`.
- **`S11-TD-HARNESS-MESSAGES-001`** -- 4 harness bins downstream from
  PROMPT 690 needing `add_message::<PlayerTeamMapUpdated>`.
- **`S11-TD-HARNESS-HANDUI-ENTITIES-001`** -- 2 harness bins downstream
  from PROMPT 690 needing `HandUiEntities`.
- **`S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001`** -- split from
  PROMPT 680 PARTIAL closure.
- **`S11-TD-FIXTURE-MESSAGES-002`** -- wider exhaustive `add_message`
  sweep (Option B from PROMPT 708).
- **`S11-TD-CI-NORMALIZE-COMMENTS-001`** -- teach `normalize_source()` to
  strip Rust comments (Option B from PROMPT 674 FAIL report).

### PROMPT 803 §5 Should/Nice rows not pulled into Sprint 13/14/15

- `S13-LOBBY-CONFIRMCLASS-SENDER-001`, `S13-COOCCUPANCY-INVARIANT-001`,
  `S13-PHASE-IDEMPOTENCY-CLIENT-001`, `S13-ADR012-LOBBY-OPTIMISM-001`,
  `S13-S2C-SUCCESS-LOG-001`, `S13-OBSERVABLE-PRODUCER-AUDIT-001`,
  `S13-PLUGIN-REGISTRATION-INVARIANT-001`, `S13-IGNORE-ATTRIBUTE-DRIFT-001`,
  `S13-MANUAL-RUNBOOK-AUTOMATION-001` (gated on Sprint 13 story 017
  outcome; NOT authorised to advance `S8-QA-001-W1` in Sprint 16),
  `S13-PROTO-MESSAGE-ID-001`. All carried forward from Sprint 13 / 14 /
  15 backlog unchanged.

---

## Required Sprint 16 Story Docs

PROMPT 1024 (this draft) does NOT author any new story files.

The Sprint 16 Must Have row is paperwork-carry-only:

- `S11-HUD-TIMER-EYEBALL-VISUAL-001` -- story file at
  `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
  ALREADY EXISTS on `origin/main` from Sprint 13 (PROMPT 822 author /
  PROMPT 823 + 1000 `/story-readiness` READY); carried forward
  unchanged if still open on `origin/main` at Sprint 16 activation.

The three Sprint 16 Should + Nice rows require **new story files** to be
authored on `origin/main` via separate story-authoring prompts BEFORE
Sprint 16 activation, mirroring the Sprint 15 PROMPT 991 / 992 / 993
authoring pattern:

| Planned ID | Required story file | Source-of-truth (current) |
|------------|---------------------|---------------------------|
| S12-TD-UI-CARD-SLOT-PRIMITIVE-001 | `production/epics/ui-clean-pass/story-XXX-ui-card-slot-primitive.md` (slug TBD; NEW) | NOT on main; needs story-authoring prompt before Sprint 16 activation. PROMPT 802 §3.3 HA1 / §3.3 HA5 producer-decision candidate (card-slot visual language) MAY apply -- producer to confirm at story-authoring time. |
| S15-OPS-APPCOMPAT-MANIFEST-001 | `production/epics/ops/story-XXX-appcompat-manifest.md` (slug TBD; NEW; ops epic NEW if not yet present) | NOT on main; needs story-authoring prompt before Sprint 16 activation. Doc-only spec + manifest embed; no producer-decision blocker. |
| S15-TD-WORKSPACE-DEAD-CODE-WARNING-001 | `production/epics/ui-clean-pass/story-XXX-workspace-dead-code-warning.md` (or under a `tests/` or `ops/` epic if producer prefers; slug TBD; NEW) | NOT on main; needs story-authoring prompt before Sprint 16 activation. No producer-decision blocker. |

The Sprint 15 closed story files (4 rows) and all Sprint 14 closed story
files (16 rows) remain on `origin/main` unchanged; Sprint 16 does NOT
touch any of them and MUST NOT reopen any of them.

## Explicitly NOT Claimed by Sprint 16 Draft

PROMPT 1024 (this draft) does NOT claim, and Sprint 16 activation MUST
NOT claim, any of the following:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion (`PAW-TD-*-a`)
- `Polish->Release` gate-check retry (PROMPT 761 FAIL preserved; NO retry
  authorised)
- stage advance from `Polish` to `Release`
- **Sprint 15 close-out** (this draft does NOT close Sprint 15; close-out
  is a separate prompt and is a prerequisite for Sprint 16 activation)
- **Sprint 16 activation** (this is a draft, not an activation)
- Sprint 16 sprint-status `active` top-level row
- underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`; third same-scope retest
  NOT authorised per `TQ-S12-C2`)
- closure of `S8-QA-001-W1`
- closure of `TQ-S12-C7`
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked
  carry; preserved unchanged; closure remains gated on human-operator
  screenshot capture; no LLM `/story-done` authorised by this draft)
- closure of any of the 24 PROMPT 1022 QA snapshot audit findings (those
  are report-only inputs to future story authoring; none are Sprint 16
  active rows)
- Sprint 15 row reopen (any of the 4 closed Sprint 15 rows)
- Sprint 14 row reopen (any of the 16 closed Sprint 14 rows)
- Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 row reopen
- full UI clean-pass repair beyond the Sprint 16 candidate rows above
  (PROMPT 1022 audit findings deferred to Sprint 17+; Tier 2 cosmetic
  captures bundle deferred unless a producer pulls them at activation;
  per-surface migration of Sprint 15 interaction-state primitive module
  deferred)
- creation of `production/qa/qa-plan-sprint-16.md` -- this is **explicitly
  out of Sprint 16 draft scope**; the QA plan MUST be authored separately
  via `/qa-plan sprint-16` after activation and before any Sprint 16
  `/dev-story`
- creation of any Sprint 16 story file by this draft (the three Should +
  Nice candidates require separate story-authoring prompts)
- `sprint_16_activation:` block in `production/sprint-status.yaml`
  (PROMPT 1024 only appends a `next_sprint_16_draft:` block at EOF;
  activation is a separate prompt)
- `sprint_15_closeout:` block in `production/sprint-status.yaml` (PROMPT
  1024 does NOT modify Sprint 15 active row content; Sprint 15 close-out
  is a separate prompt that runs before Sprint 16 activation)
- modification or re-integration of PROMPT 1019 / 1020 / 1021 / 1023 QA
  snapshot tooling tracks (preserved verbatim on `origin/main`)

## Suggested First Parallel Batch (post-activation)

The orchestrator that activates Sprint 16 should batch the post-activation
work as follows. Per CLAUDE.md / `.claude/docs/coordination-rules.md`,
"Launch only actually ready work; do not invent parallelism to satisfy a
quota."

### Parallel batch (file-disjoint, ready post-activation; can launch concurrently)

After Sprint 16 activation, after the three net-new story files are
authored on main, after each story file passes `/story-readiness`, and
after `/qa-plan sprint-16` lands:

| Story | File scope | Parallel-safe with |
|---|---|---|
| `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` | `client/src/ui/design_tokens/` or `client/src/ui/primitives/` (NEW path; TBD) + `client/src/ui/hand/` and/or `client/src/ui/shop_auction/` (incremental, additive consumer migration) + `tests/integration/ui_clean_pass/` (NEW path) + `production/epics/ui-clean-pass/story-XXX-*.md` | AppCompat manifest (test-binary configuration vs UI module); dead-code warning cleanup (test-helper deletion vs UI module). **Caveat**: if the card-slot primitive's incremental consumer migration touches `client/src/ui/shop_auction/mod.rs`, sequence against any future PROMPT 1022 P1 row that also touches the same module (none in Sprint 16 scope). |
| `S15-OPS-APPCOMPAT-MANIFEST-001` | `Cargo.toml` test-binary configuration + (likely) `build.rs` or equivalent + `tests/integration/board_rendering/` or wherever the `spawn_range_live_update_contract-*` binary lives + `production/epics/ops/story-XXX-*.md` | All other rows (test-binary configuration vs UI surface; no `client/` UI module touch). |
| `S15-TD-WORKSPACE-DEAD-CODE-WARNING-001` | `tests/integration/presentation/hand_ui_asset_wiring_test.rs` (1-2 lines around `count_with_image_node` at line 43) + `production/epics/ui-clean-pass/story-XXX-*.md` | All other rows (single-file change; no shared module). |

Pairwise file-disjoint by surface: UI primitive module / Cargo test-
binary configuration / test-helper deletion in a different test file.
All three depend on Sprint 14 + Sprint 15 modules which are already on
main.

### Serial / human-blocked (do NOT include in the parallel batch)

| Item | Reason |
|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | **Human-operator-blocked**. Requires manual 2-client run + screenshot capture; no LLM `/story-done` authorised. Schedule producer + human-operator slot; track separately. If closed on `origin/main` before Sprint 16 activation, dropped from Sprint 16 entirely. |
| Per-story `/story-readiness` reruns (3 rows) | Read-only but serialize through the producer / orchestrator; cannot run before each Sprint 16 story file lands on main. |
| `/qa-plan sprint-16` authoring | Single shared-status writer; must complete before any `/dev-story` per CLAUDE.md / QA-plan precedent. |

### Pre-activation prerequisites (no parallelism; must complete before activation prompt is launched)

1. **Sprint 15 close-out paperwork prompt** -- mirrors PROMPT 817 / 894 /
   987 pattern; flips top-level `status: active -> closed-with-conditions`;
   appends `sprint_15_closeout:` block. Carries `S11-HUD-TIMER-EYEBALL-
   VISUAL-001` forward if still open. PROMPT 1024 does NOT do this.
2. Three story-authoring prompts (one per Should + Nice candidate), each
   in its own worktree on its own branch
   (`story-authoring/sprint-16-card-slot-primitive`,
   `story-authoring/sprint-16-appcompat-manifest`,
   `story-authoring/sprint-16-dead-code-warning`). File-disjoint by
   surface so these three can themselves run as a parallel batch.
3. Integration of the three story-authoring branches to `origin/main`
   (mirroring Sprint 15 PROMPT 995 four-way merge precedent; can land as
   one `--no-ff` per branch or one consolidated merge).
4. Sprint 16 activation prompt (mirrors PROMPT 826 / PROMPT 897 / PROMPT
   997) flips `production/sprint-status.yaml` top-level and appends a
   `sprint_16_activation:` block. PROMPT 1024 itself does NOT do this.

## Sequencing Notes

Per `docs/ux/ui-clean-pass-roadmap.md` and PROMPT 1024 small-plan framing:

1. **Sprint 14 Tier 0 (ranks 1-6) is DONE on main.** Sprint 16 card-slot
   primitive consumes Tier 0 token modules directly without re-authoring
   primitives.
2. **Sprint 14 Tier 1 (ranks 7-12) is DONE on main.** Sprint 16
   card-slot primitive's incremental consumer-surface migration is
   parallel-safe with the existing Tier 1 surfaces (no module-level
   collision because migration is additive, not a rewrite).
3. **Sprint 15 Tier 1 Should-adjacent rows (hand drag visuals + board
   spec) are DONE on main.** Sprint 16 card-slot primitive's hand-
   surface consumer migration coexists with Sprint 15 hand-drag-state
   visuals because the latter is read-only over ephemeral drag state
   while the former is a structural slot primitive (different concern).
4. **Sprint 15 Tier 0 Should-adjacent row (interaction state primitives)
   is DONE on main.** Sprint 16 card-slot primitive may consume the
   interaction state primitives if applicable to its surface (producer +
   ux-designer call at story-authoring time).
5. **The Sprint 15 carry (`S11-HUD-TIMER-EYEBALL-VISUAL-001`) is
   parallel-safe with all other Sprint 16 rows** (cosmetic visual check
   only; no shared host module; no code change unless a regression is
   found) but is human-operator-blocked; sequence outside the parallel
   implementation batch.
6. **Tier 3 rank 13 card-slot primitive** (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`)
   touches `client/src/ui/design_tokens/` or `client/src/ui/primitives/`
   (a host module either already present from Sprint 14 Tier 0 ranks or
   new) -- additive, not a refactor of existing tokens; consumer-surface
   migration is incremental (one surface in Sprint 16; remaining surfaces
   may carry to Sprint 17).
7. **AppCompat manifest** (`S15-OPS-APPCOMPAT-MANIFEST-001`) touches the
   `spawn_range_live_update_contract` test binary's configuration only;
   no UI surface touch; orthogonal to the card-slot primitive.
8. **Dead-code warning cleanup** (`S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`)
   touches one test file at a known line; orthogonal to all other rows.
9. **No PROMPT 802 §9 producer-decisions are blocking for the chosen
   Sprint 16 rows.** Decisions 1-4 are RESOLVED on Sprint 14 (decisions
   2 / 3 / 4 per PROMPT 911 + 922 + 933 + 935 + 967). Decisions 5
   (cosmetic captures bundle) and 6 (post-Tier-1 polish bar) are NOT
   relevant to the chosen Sprint 16 rows. If a producer pulls the
   deferred Tier 2 cosmetic captures bundle into Sprint 16 at activation,
   decision 5 must be confirmed.
10. **PROMPT 802 §3.9 G8 lobby file collision risk** -- inapplicable to
    Sprint 16 (no row touches `client/src/ui/lobby.rs`).
11. **Live QA snapshot tooling tracks** (PROMPT 1019/1020/1021/1023 +
    PROMPT 1022 audit) -- preserved verbatim on `origin/main`. Sprint 16
    does NOT re-integrate, re-audit, or re-tool the QA snapshot path.
    The 24 PROMPT 1022 findings inform future Sprint 17+ scoping but are
    not Sprint 16 active rows.

## Cargo Resource Policy (this draft)

**Not applied** -- PROMPT 1024 is a paperwork-only draft. No `cargo`
command was invoked. `$env:CARGO_TARGET_DIR`,
`$env:CARGO_PROFILE_DEV_DEBUG`, `$env:CARGO_PROFILE_TEST_DEBUG`,
`$env:CARGO_INCREMENTAL`, `$env:RUSTFLAGS` were not set. Cargo resource
policy was not applied because no Cargo command was needed.

Sprint 16 implementation prompts (post-activation `/dev-story` workers
and integration merges) MUST apply the binding Windows / MSVC Cargo
resource policy per the Sprint 13 / Sprint 14 / Sprint 15 precedent
(PROMPT 829 / 833 / 884 / 902 / 906 / 907 / 912 / 917 / 918 / 930 / 938
/ 941 / 951 / 955 / 959 / 961 / 970 / 973 / 975 / 1003 / 1004 / 1005 /
1006 / 1007 / 1008 worker + integration prompts):

- `$env:CARGO_TARGET_DIR = "D:\_DEV\cargo-target\ccgs-msvc"`
- `$env:CARGO_PROFILE_DEV_DEBUG = "0"`
- `$env:CARGO_PROFILE_TEST_DEBUG = "0"`
- `$env:CARGO_INCREMENTAL = "0"`
- `$env:RUSTFLAGS = "-C debuginfo=0 -C link-arg=/DEBUG:NONE"`

## Provisional Next Launchable Prompts (after this draft lands)

PROMPT 1024 (this draft) lands as a Sprint 16 sprint-plan draft on
`origin/main` via a separate paperwork-only integration prompt (not
performed by PROMPT 1024 itself; PROMPT 1024 commits to its own branch
`sprint-plan/sprint-16-draft` and pushes the branch). After this draft
lands:

1. **(Optional / human-scheduled)** Sprint 15 `S11-HUD-TIMER-EYEBALL-
   VISUAL-001` human-operator screenshot capture session. May happen
   before or after Sprint 15 close-out; closes the row on `origin/main`
   if the capture is approved; drops the row from Sprint 16 Must Have
   if closed before Sprint 16 activation.
2. **Sprint 15 close-out paperwork prompt** -- mirrors PROMPT 817 / 894
   / 987 pattern; flips top-level `status: active -> closed-with-
   conditions`; appends `sprint_15_closeout:` block. Carries `S11-HUD-
   TIMER-EYEBALL-VISUAL-001` into Sprint 16 if still open. **Prerequisite
   for Sprint 16 activation.**
3. **Sprint 16 story-authoring prompts** (three; runnable as a parallel
   batch on file-disjoint worktrees):
   - `story-authoring/sprint-16-card-slot-primitive` ->
     `production/epics/ui-clean-pass/story-XXX-ui-card-slot-primitive.md` (NEW).
   - `story-authoring/sprint-16-appcompat-manifest` ->
     `production/epics/ops/story-XXX-appcompat-manifest.md` (NEW).
   - `story-authoring/sprint-16-dead-code-warning` ->
     `production/epics/ui-clean-pass/story-XXX-workspace-dead-code-warning.md` (NEW).
4. **Sprint 16 story-authoring integration prompts** -- one `--no-ff`
   merge per branch or one consolidated merge (mirrors PROMPT 893 / 995
   precedent).
5. **Sprint 16 activation prompt** -- flips top-level `sprint: 15 -> 16`
   and `status: closed-with-conditions -> active`; appends
   `sprint_16_activation:` block; prepends ACTIVATED banner to this file
   (mirrors PROMPT 826 / PROMPT 897 / PROMPT 997 pattern).
6. **`/qa-plan sprint-16`** -- after activation; before any `/dev-story`.
7. **Per-story `/story-readiness` reruns** against Sprint 16 activation
   HEAD for each of the (3 or 4) Sprint 16 story files (Must `story-014-
   hud-timer-eyeball-visual-check.md` carry IF still open, plus the
   three NEW Should + Nice files).
8. **Sprint 16 human-operator screenshot capture session** for
   `S11-HUD-TIMER-EYEBALL-VISUAL-001` IF the row is still open at
   activation; otherwise skipped.
9. **Sprint 16 `/dev-story` runs** for the three Should + Nice rows
   (parallel-safe per "Suggested First Parallel Batch" section above).
10. **Sprint 16 integration prompts** per row (mirrors Sprint 15 pattern).
11. **Sprint 16 `/story-done` paperwork** per row (serialized
    shared-status writer).
12. **Sprint 16 smoke-check** (`/smoke-check`) at end of sprint.
13. **Sprint 16 Team-QA** (`/team-qa sprint`) after smoke.
14. **Sprint 16 close-out disposition** (paperwork-only close-out
    prompt; mirrors PROMPT 817 / 894 / 987 pattern). Expected disposition:
    `closed-with-conditions` (NOT `closed`) -- the same accept-risk
    conditions carry forward.

---

## Files Changed By PROMPT 1024

| File | Status | Notes |
|---|---|---|
| `production/sprints/sprint-16.md` | NEW | This draft. Sprint 16 NOT activated. Sprint 15 NOT closed. |
| `production/sprint-status.yaml` | MODIFIED | `next_sprint_16_draft:` block appended at EOF; `updated:` annotation refreshed with PROMPT 1024 prefix preserving PROMPT 1010 narrative as `# Previous:` comment chain. **No row flips.** Sprint 15 active rows NOT modified (top-level `sprint: 15 / status: active / stage: Polish` preserved). `sprint_15_activation:` block NOT modified. `sprint_15_story_done:` PROMPT 1009 + PROMPT 1010 entries preserved verbatim. All Sprint 14 / 13 / 12 / 11 / 10 closeout blocks preserved verbatim. |
| `production/session-state/active.md` | MODIFIED | PROMPT 1024 banner prepended above PROMPT 1010 banner. |
| `production/session-state/codex-orchestrator-state.md` | MODIFIED | PROMPT 1024 section prepended above PROMPT 1010 section. |

Explicitly **NOT** touched by PROMPT 1024 (forbidden by task scope):

- `client/`, `server/`, `shared/`, `tests/` -- no production or test code edits.
- `Cargo.toml` / `Cargo.lock` / `.cargo/` / `Trunk.toml` -- not modified.
- `.github/` -- not modified.
- `production/stage.txt` -- no stage advance.
- `production/qa/qa-plan-sprint-16.md` -- NOT authored by PROMPT 1024.
- `production/qa/qa-plan-sprint-15.md` / `qa-plan-sprint-14.md` / etc. -- NOT modified.
- `production/qa/smoke-*.md` -- NOT modified.
- `production/qa/team-qa-*.md` -- NOT modified.
- `production/qa/evidence/*` -- NOT modified.
- `production/gate-checks/*` -- no gate-check retry or edit.
- `production/sprints/sprint-15.md` / `sprint-14.md` / `sprint-13.md` / `sprint-12.md` / `sprint-11.md` / `sprint-10.md` -- NOT modified.
- Sprint 15 active rows in `production/sprint-status.yaml` -- NOT modified (in particular `S11-HUD-TIMER-EYEBALL-VISUAL-001` remains `ready` human-operator-blocked).
- Sprint 14 / 13 / 12 / 11 / 10 story files under `production/epics/` -- NOT modified or reopened.
- Sprint 15 closed story files under `production/epics/` -- NOT modified or reopened.
- Sprint 16 candidate story files (the three Should + Nice rows) -- NOT authored by PROMPT 1024; authoring is a separate set of prompts.
- Release artifacts, release-checklist, launch-checklist, changelog, patch notes -- NOT modified or created.
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock` -- NOT modified.
- `reports/PROMPT-1022-qa-snapshot-visual-state-audit.md`, `reports/PROMPT-1019-*`, `reports/PROMPT-1020-*`, `reports/PROMPT-1021-*`, `reports/PROMPT-1023-*` -- NOT modified (`reports/` is gitignored; preserved as authored by their respective prompts).

PROMPT 1024 did NOT run `/dev-story`, `/smoke-check`, `/team-qa`,
`/gate-check`, `/release-check`, `/story-done`, `/story-readiness`,
`/qa-plan`, or any `cargo` / `trunk` command. PROMPT 1024 did NOT
activate Sprint 16. PROMPT 1024 did NOT close Sprint 15. PROMPT 1024 did
NOT advance stage from Polish.

---

**Final status line**

1024: SPRINT-16-PLAN-DRAFT: drafted-not-activated
