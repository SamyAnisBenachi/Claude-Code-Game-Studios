# Sprint 15 -- ACTIVATED (PROMPT 997; Polish stage)

> **PROMPT 997 paperwork-only Sprint 15 activation (2026-05-17)**. Source-of-truth
> at activation: `origin/main@8294f9a02ee14a09e8add43db181f0e4ce890816` (PROMPT
> 995 ui interaction state primitives story-authoring integration tip).
> Worktree: `D:/_DEV/claude-code-game-studios-worktrees/sprint-15-activation-997`.
> Branch: `activate/sprint-15-997`.
>
> **Status**: `active` (flipped on `production/sprint-status.yaml` top-level
> from `sprint: 14 / status: closed-with-conditions` to `sprint: 15 / status:
> active`). **Stage**: `Polish` UNCHANGED (`production/stage.txt` NOT modified
> by PROMPT 997).
>
> Sprint 15 active row set (5 rows; ~2.75d estimated effort against 8d
> available capacity): **Must Have** S11-HUD-TIMER-EYEBALL-VISUAL-001
> (Sprint 13 -> 14 -> 15 human-operator-blocked carry; promoted Should ->
> Must) + S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP
> (paperwork-only sprint-status.yaml row-status flip); **Should Have**
> S12-UX-HAND-DRAG-STATE-VISUALS-001 (story 020, hand-ui surface, 0.5d) +
> S11-UX-BOARD-RENDERING-SPEC (story 013, doc-only, 0.75d); **Nice to
> Have** S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 (story 008, design-
> tokens module, 1.0d).
>
> PROMPT 988 DRAFT plan body below is preserved verbatim; activation does
> NOT rewrite the plan. PROMPT 997 changed only the top banner above this
> line, `production/sprint-status.yaml` (top-level flips +
> stories: block + sprint_15_activation: block append +
> next_sprint_15_draft: removal; sprint_14_closeout: and all prior closeout
> blocks preserved unchanged), `production/session-state/active.md` and
> `production/session-state/codex-orchestrator-state.md` (PROMPT 997
> activation banners prepended).
>
> **PROMPT 761 `Polish->Release` gate-check `FAIL` preserved** at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry**
> attempted by PROMPT 997; **NO retry** in scope for Sprint 15. Sprint 15
> is NOT a `Polish->Release` sprint.
>
> **Carried conditions preserved verbatim** (none closed by PROMPT 997):
> S11-HUD-TIMER-EYEBALL-VISUAL-001 human-operator-blocked carry, S8-QA-001-W1
> OPEN, QA-COND-0005 accepted-risk (friend-game scope), QA-COND-0006
> accepted-risk, PAW-TD-*-a accepted-risk across PAW-002..PAW-006, PROMPT
> 683-era runtime divergence preserved (no third same-scope retest per
> TQ-S12-C2), TQ-S12-C1..C7 preserved verbatim, Sprint 12 story 019
> cannot-reproduce preserved (no underlying drag-runtime bug fix claim),
> Sprint 14 disposition closed-with-conditions preserved unchanged
> (PROMPT 987), all 16 closed Sprint 14 /story-done closures preserved.
>
> **Explicit non-claims by PROMPT 997 activation**: NO public release
> readiness, NO RC readiness, NO full game completion, NO broad / Standard-
> tier accessibility completion (QA-COND-0005), NO playtest validation
> (QA-COND-0006), NO full playable-client manual QA, NO two-client
> GAME_OVER closure (S8-QA-001-W1 OPEN), NO final-art completion
> (PAW-TD-*-a), NO Polish->Release gate-check retry (PROMPT 761 FAIL
> preserved), NO stage advance from Polish to Release, NO underlying
> drag-runtime bug fix, NO closure of S11-HUD-TIMER-EYEBALL-VISUAL-001
> (closure remains gated on human-operator screenshot capture; no LLM
> /story-done authorised), NO closure of S11-CLIENT-CONNECTION-LOST-
> OBSERVABILITY-001 (the row-status flip is a separate Sprint 15 Must
> Have paperwork-only prompt, NOT performed by activation), NO closure
> of S8-QA-001-W1 / TQ-S12-C7, NO Sprint 14 row reopen, NO Sprint 15
> /dev-story / /story-done / /story-readiness / /smoke-check / /team-qa /
> /gate-check / /release-check / /qa-plan by PROMPT 997.
>
> **PROMPT 997 paperwork-only activation scope**: NO `/dev-story`, NO
> `/story-readiness`, NO `/story-done`, NO `/smoke-check`, NO `/team-qa`,
> NO `/gate-check`, NO `/release-check`, NO `/qa-plan`, NO production/
> qa/qa-plan-sprint-15.md authored, NO stage advance, NO implementation,
> NO CI run, NO `cargo` / `trunk` invocation, NO touch of `client/` /
> `server/` / `shared/` / `tests/` / `Cargo.toml` / `Cargo.lock` /
> `.cargo/` / `.github/` / `production/stage.txt` / `production/qa/*` /
> `production/gate-checks/*`. Files changed only: `production/sprints/
> sprint-15.md` (this banner prepended; plan body unchanged), `production/
> sprint-status.yaml`, `production/session-state/active.md`, `production/
> session-state/codex-orchestrator-state.md`.

---

# Sprint 15 -- DRAFT (NOT ACTIVATED; Polish stage)

> **PROMPT 988 paperwork-only sprint plan draft (2026-05-16)**.
> Source-of-truth at authoring: `origin/main@7ceba822cd99bb3119292bda0a100817f3103335` (PROMPT 987
> Sprint 14 close-out disposition tip; `close-out(s14): Sprint 14 close-out disposition closed-with-conditions`).
> Worktree: `D:/_DEV/claude-code-game-studios-worktrees/sprint-15-plan-draft-988`.
> Branch: `sprint-plan/sprint-15-draft`.
>
> **Status**: `draft -- authored 2026-05-16 by PROMPT 988`. **Sprint 15 is NOT activated by this draft.**
> Activation is a separate explicit prompt that mirrors the PROMPT 826 / PROMPT 897 pattern:
> flips `production/sprint-status.yaml` top-level `sprint: 14 -> 15` and
> `status: closed-with-conditions -> active`, appends a `sprint_15_activation:`
> block, and adds an ACTIVATED banner to this file. PROMPT 988 itself does NOT
> activate Sprint 15.
>
> **Stage**: `Polish` (UNCHANGED). `production/stage.txt` NOT modified by this draft and
> MUST NOT be modified by activation. PROMPT 761 `Polish->Release` gate-check `FAIL`
> evidence preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`;
> **NO retry** is in scope for Sprint 15 and MUST NOT be attempted by activation.
> Sprint 15 is **NOT** a `Polish->Release` sprint.
>
> **Provisional start / end (locked at activation)**: 2026-07-30 -> 2026-08-12 (10
> workdays). Continuous follow-on to Sprint 14 (2026-07-16 -> 2026-07-29).
>
> **Sprint 14 disposition preserved unchanged**: `closed-with-conditions` per
> PROMPT 987, recorded under `sprint_14_closeout:` block in `production/sprint-status.yaml`.
> 16 of 17 Sprint 14 rows closed; the only un-closed row is
> `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Should Have, story 014, 0.25d,
> human-operator-blocked cosmetic visual check), explicitly **carried forward
> into Sprint 15** as a Sprint 13 -> Sprint 14 -> Sprint 15 carry (originally
> Sprint 10 smoke retry-7 W2).
>
> **Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts** preserved
> unchanged. `S8-QA-001-W1` OPEN, `QA-COND-0005` + `QA-COND-0006`
> accepted-risk, `PAW-TD-*-a` accept-risk across PAW-002..PAW-006,
> `TQ-S12-C1..C7` preserved verbatim, PROMPT 683-era runtime divergence
> question preserved (no third same-scope retest per `TQ-S12-C2`), Sprint 12
> story 019 underlying drag-runtime bug NOT claimed fixed (cannot-reproduce
> preserved), `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row
> remains as-is (row-status flip is a Sprint 15 Must Have paperwork-only
> candidate row per below; NOT closed by this draft).
>
> **Sprint 15 explicitly does NOT claim**: public release readiness,
> release-candidate readiness, full game completion, broad / Standard-tier
> accessibility completion, playtest validation, full playable-client manual QA,
> two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN), final-art /
> asset-production completion, `Polish->Release` gate-check retry, stage
> advance from Polish to Release, underlying drag-runtime bug fix (Sprint 12
> story 019 closed cannot-reproduce, NOT bug-fixed), Sprint 14 row reopen
> (any of the 16 closed Sprint 14 rows), `S8-QA-001-W1` closure,
> `TQ-S12-C7` closure, or full UI clean-pass repair beyond the candidate rows
> below (Tier 3 rank 13 deferred; Tier 2 cosmetic captures deferred unless a
> producer activates them separately).
>
> **PROMPT 988 paperwork-only draft scope**: NO `/dev-story`, NO
> `/story-readiness`, NO `/story-done`, NO `/smoke-check`, NO `/team-qa`,
> NO `/gate-check`, NO `/release-check`, NO `/qa-plan`, NO Sprint 15
> activation, NO `production/qa/qa-plan-sprint-15.md` authored, NO stage
> advance, NO implementation, NO CI run, NO `cargo` / `trunk` invocation,
> NO touch of `client/` / `server/` / `shared/` / `tests/`. Files allowed:
> this file (NEW), `production/sprint-status.yaml`
> (`next_sprint_15_draft:` block appended only; Sprint 14 closeout blocks
> NOT modified), `production/session-state/active.md` (PROMPT 988 banner
> prepended), `production/session-state/codex-orchestrator-state.md`
> (PROMPT 988 section prepended).

---

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`. Sprint 15
  does NOT advance stage. Sprint 15 is NOT a `Polish->Release` sprint.
- Sprint 14 is `closed-with-conditions` per PROMPT 987 (commit
  `7ceba82` on `origin/main`); Must Have 9/9 done, Should Have 3/4 done
  (one row human-operator-blocked carry), Nice to Have 4/4 done; 16 of 17
  rows closed. The single un-closed row is `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  (Should Have, story 014, 0.25d, human-operator-blocked cosmetic visual
  check); it is **carried forward** into Sprint 15 planning here.
- This draft pulls a **deliberately small** Sprint 15 scope. Per PROMPT 988
  instruction "Prefer a small, executable plan over a broad mega-sprint",
  the plan covers (1) the Sprint 14 carry, (2) one small paperwork-only
  row that closes a Sprint 13 evidence-vs-status gap, (3) the two
  smallest remaining UI clean-pass roadmap candidates that are
  file-disjoint and have no producer-decision blocker, and (4) a single
  Tier 0 Should-priority adjacent row as Nice to Have. Heavier candidates
  (Tier 3 rank 13 multi-surface refactor; Tier 2 cosmetic captures bundle;
  the long server-hardening backlog from Sprint 11/12/13) are deliberately
  **deferred to Sprint 16+ backlog**, not promoted into Sprint 15.
- Sequencing is governed by the canonical reconciliation roadmap at
  `docs/ux/ui-clean-pass-roadmap.md` (PROMPT 838) and by the Sprint 14
  close-out disposition at
  `production/qa/evidence/sprint-14-close-out-disposition.md` (PROMPT 987).
  Tier 0 (ranks 1-6) and the headline Tier 1 surfaces (ranks 7, 10, 12)
  are already DONE in Sprint 14; the remaining Tier 1 Must rows (ranks 8,
  9, 11) plus their paired Nice rows are also DONE. Sprint 15 closes out
  the Tier 1 Should-priority adjacent rows that were NOT pulled into
  Sprint 14, and authors the Tier 3 doc-only spec (rank 14). Tier 3 rank
  13 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`, 1.5d multi-surface refactor)
  is **deferred to Sprint 16+** -- it would be a mega-sprint expansion
  here.
- The HUD timer eyeball visual check (`S11-HUD-TIMER-EYEBALL-VISUAL-001`,
  story 014) is carried forward from Sprint 14 as a Must Have row.
  Disposition: **human-operator-blocked**; cosmetic visual check requires
  human screenshot capture across `DraftInitial` 45s / `DraftShop` 30s /
  `Placement` 10-12s phases and cannot be auto-closed by an LLM
  `/story-done`. Closure remains gated on human-operator screenshot
  capture per the Sprint 13 closeout (PROMPT 894) and Sprint 14 closeout
  (PROMPT 987) carry plan. **Promoted from Should Have to Must Have in
  Sprint 15 to surface it as a sprint-level blocker the producer must
  schedule human-operator time for, rather than carrying it indefinitely.**
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 15 QA plan exists at draft time. A Sprint 15 QA plan
  (`production/qa/qa-plan-sprint-15.md`) MUST be authored via `/qa-plan
  sprint-15` **after** Sprint 15 activation **and after** each Sprint 15
  story file passes `/story-readiness` against activation HEAD. No
  `/dev-story` is authorised before the QA plan exists. PROMPT 988 does
  NOT author the QA plan.
- Sprint 15 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
  validation (`QA-COND-0006`), full playable-client manual QA, two-client
  GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
  completion (`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage
  advance from Polish to Release, or underlying drag-runtime bug fix
  (Sprint 12 story 019 remains `closed-with-conditions / cannot-reproduce`).
  None of these can be added to Sprint 15 by activation; each requires its
  own scope and gate evidence.

## Entry Conditions (must be true at activation)

- `production/sprint-status.yaml` top-level reads `sprint: 14`,
  `status: "closed-with-conditions"` (already true at draft time per
  PROMPT 987).
- `production/stage.txt` reads `Polish` (UNCHANGED).
- PROMPT 761 `Polish->Release` gate-check `FAIL` evidence preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
- Sprint 14 disposition (`closed-with-conditions` per PROMPT 987) preserved
  unchanged. Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts
  preserved unchanged.
- `S8-QA-001-W1` OPEN. `QA-COND-0005` + `QA-COND-0006` accepted-risk.
  `PAW-TD-*-a` accept-risk across PAW-002..PAW-006. PROMPT 683-era runtime
  divergence question preserved (no third same-scope retest per
  `TQ-S12-C2`). `TQ-S12-C1..C7` preserved verbatim.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 13 -> Sprint 14 -> Sprint 15
  human-operator-blocked carry) preserved on
  `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
  (authored by PROMPT 822; `/story-readiness` READY per PROMPT 823;
  carried forward by PROMPT 894 and PROMPT 987 closeouts).
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row remains
  un-flipped (evidence closure landed in Sprint 13 via
  `S13-CONN-LOST-UX-001` per PROMPT 891; row-status flip is a Sprint 15
  Must Have paperwork-only candidate row per below).
- Smoke evidence (PROMPT 983 `PASS-WITH-WARNINGS` at
  `production/qa/smoke-sprint-14-2026-05-16-rerun.md`) and Team-QA
  evidence (PROMPT 984 `APPROVED-WITH-CONDITIONS` at
  `production/qa/team-qa-sprint-14-2026-05-16.md`) preserved on
  `origin/main` per PROMPT 986.
- The Sprint 15 candidate story files for Should/Nice rows below do NOT
  yet exist on `origin/main` at draft time. **Story authoring prompts are
  a prerequisite to Sprint 15 activation** for the three net-new
  candidate stories. The two Must Have rows are paperwork-only (Sprint 14
  carry + backlog row-status flip) and do not require new story authoring.

If any entry condition fails, Sprint 15 does NOT activate; producer must
revise scope before activation.

## Sprint Goal

Sprint 15 is a **UI clean-pass closeout sprint** for the remaining
Tier 1 Should-priority adjacent rows and the Tier 3 doc-only spec, plus
the Sprint 13 -> 14 -> 15 human-operator-blocked HUD timer eyeball visual
check carry and a small paperwork-only row-status flip. It is NOT a
release sprint. The goal is:

1. **Close the Sprint 13 -> 14 -> 15 human-operator-blocked carry**
   (`S11-HUD-TIMER-EYEBALL-VISUAL-001`) by scheduling explicit human-operator
   screenshot capture time. The story file is already READY; closure remains
   gated on human-operator action. Promotion to Must Have surfaces the
   schedule risk that has carried this row through four sprints.
2. **Close the Sprint 13 evidence-vs-status gap** by flipping the backlog
   row-status for `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`
   (evidence closure landed in `S13-CONN-LOST-UX-001` per PROMPT 891 but
   the row-status flip was deferred). Paperwork-only.
3. **Land the two smallest remaining UI clean-pass roadmap candidates**
   that are file-disjoint and producer-decision-unblocked:
   `S12-UX-HAND-DRAG-STATE-VISUALS-001` (Tier 1 Should adjacent; hand UI
   only) and `S11-UX-BOARD-RENDERING-SPEC` (Tier 3 rank 14 doc-only spec
   depending only on the now-DONE rank 6 global UI design spec).
4. **Author one Tier 0 Should-priority adjacent row** as Nice to Have:
   `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (hover / focus / pressed
   / disabled primitive set; pair with the now-DONE rank 6 global UI
   design spec). Sprint 14 Tier 1 surfaces are tolerable without it but
   degrade to per-site button styling; landing this row removes that
   degradation risk for any post-Sprint-15 UI work.

Sprint 15 does NOT claim release readiness, broad accessibility
completion, full playable-client manual QA, playtest validation,
final-art / asset-production completion, S8-QA-001-W1 closure, full game
completion, two-client GAME_OVER closure, a Polish->Release retry,
closure of the underlying drag-runtime bug from Sprint 12 story 019, or
the Tier 3 rank 13 multi-surface card-slot primitive refactor (deferred
to Sprint 16+).

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10/11/12/13/14)
- Buffer (20%): 2 days reserved for (a) per-row `/story-readiness` re-runs
  against Sprint 15 activation HEAD; (b) the three net-new story-authoring
  prompts (Should #3, Should #4, Nice #5) that must precede `/dev-story`;
  (c) `/qa-plan sprint-15` authoring; (d) producer / human-operator
  scheduling friction on the human-operator-blocked Must Have carry.
- Available: **8 effective planned days**
- Planned Must Have scope: **~0.50 estimated days** (Sprint 14 carry 0.25d
  human-operator-blocked + backlog row-status flip 0.25d paperwork-only).
  These are intentionally non-implementation rows; the implementation work
  is in Should + Nice.
- Should Have scope: **~1.25 estimated days** (hand drag state visuals
  0.5d + board rendering spec 0.75d). Both are file-disjoint and
  producer-decision-unblocked.
- Nice to Have scope: **~1.00 estimated days** (interaction state
  primitives 1.0d). Authoring + implementation; lands only if Must + Should
  closure is on track.
- Total implementation effort: **~2.75 days against 8 days available**.
  This is a deliberately small plan; the remaining 5.25 days of capacity
  absorbs story-authoring overhead, `/story-readiness` reruns, `/qa-plan
  sprint-15` authoring, integration / `/story-done` paperwork, and
  human-operator scheduling for the Must Have carry. If burn-down comes in
  under capacity, a producer may pull a single additional row from the
  Sprint 15 Backlog section (priority: row-status paperwork; do NOT pull
  the Tier 3 rank 13 multi-surface refactor without a separate scope
  decision).

---

## Tasks

> All IDs below are **draft Sprint 15 candidate** tickets. They are NOT yet
> active `production/sprint-status.yaml` rows. Promotion to active rows
> happens at activation via `/sprint-plan sprint-15` (or an equivalent
> activation prompt), mirroring the PROMPT 826 / PROMPT 897 pattern.
> All slug provenance and rank references are against
> `docs/ux/ui-clean-pass-roadmap.md` and the Sprint 14 close-out
> disposition at
> `production/qa/evidence/sprint-14-close-out-disposition.md`.

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD Timer Eyeball Visual Check (Sprint 13 -> 14 -> 15 carry; **human-operator-blocked**) -- manual 2-client run validating timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases | UI programmer + **human operator** | 0.25 | **Sprint 14 carry** per PROMPT 987 close-out `remaining_open_human_blocked` + `carried_into_sprint_15_planning`; originally Sprint 10 smoke retry-7 W2 -> Sprint 11 -> Sprint 12 -> Sprint 13 -> Sprint 14 -> Sprint 15 carry. Story file at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` (PROMPT 822 author / PROMPT 823 READY). **Promoted Should -> Must in Sprint 15** to surface the four-sprint carry as a sprint-level blocker requiring explicit human-operator scheduling. | Story 014 `/story-readiness` confirms READY against Sprint 15 activation HEAD (expected READY per PROMPT 823 on `origin/main` unless the story file has been touched). Manual 2-client run + screenshot evidence in `production/qa/evidence/sprint-15-hud-timer-visual-check/` (NEW). Cosmetic verification only; no production-code change unless an actual visual regression is found and a follow-on story is authored. **Closure remains gated on human-operator screenshot capture** -- no LLM `/story-done` is authorised. Does NOT claim `QA-COND-0005` Standard-tier accessibility completion. `PAW-TD-*-a` preserved. If human-operator time cannot be scheduled within Sprint 15, the row carries forward as a Sprint 15 -> Sprint 16 carry; this is acceptable but the producer must document the cause in Sprint 15 close-out. |
| S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP | Paperwork-only row-status flip for `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row -- evidence closure landed in `S13-CONN-LOST-UX-001` per PROMPT 891 `/story-done`; the row-status flip itself was deferred at Sprint 13 closeout (PROMPT 894 `conditions_carried_forward_unchanged`) and Sprint 14 closeout (PROMPT 987 `explicitly_not_claimed`). | producer + qa-lead | 0.25 | Sprint 13 closeout `conditions_carried_forward_unchanged` (PROMPT 894); Sprint 14 closeout `explicitly_not_claimed` (PROMPT 987). | Backlog row in `production/sprint-status.yaml` for `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` flipped from its current pre-Sprint-13 disposition to `closed-by-evidence` (or equivalent disposition that names PROMPT 891 / `S13-CONN-LOST-UX-001` as the closure evidence). Closure notes reference Sprint 13 story 021 evidence per AC8 design. **No code change, no test change, no production code touched.** Paperwork-only. Does NOT change Sprint 13 / 12 / 11 / 10 disposition. Does NOT claim S8-QA-001-W1 closure (different surface). `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` accept-risk preserved verbatim. |

**Must Have subtotal**: ~0.50 estimated days. Both rows are
intentionally non-implementation: the Sprint 14 carry is
human-operator-blocked manual screenshot capture only; the
`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` flip is paperwork-only
serialized through `production/sprint-status.yaml`. Together they
discharge two long-carried open items without consuming code-modification
capacity. Sprint 15 implementation capacity flows into the Should + Nice
rows below.

### Should Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S12-UX-HAND-DRAG-STATE-VISUALS-001 | Hand Drag-State Visuals (Tier 1 Should adjacent) -- visual differentiation for hand-card drag states (idle / hover / drag / drop-target / disabled) using Tier 0 token primitives | UI programmer + ux-designer | 0.5 | Roadmap §"Tier 1 Should-Priority Adjacent Rows" (independent of the 14 main-rank rows; orthogonal to ranks 7-12; touches hand UI only; PROMPT 802 §3.3 HA3). **Story file NEW; must be authored before activation** at `production/epics/hand-ui/story-XXX-hand-drag-state-visuals.md` (slug TBD by story-authoring prompt; mirrors PROMPT 879 / 880 / 881 pattern for Sprint 14 candidate authoring). | Story file authored on `origin/main` via a separate story-authoring prompt before Sprint 15 activation. `/story-readiness` passes against Sprint 15 activation HEAD. Drag-state visuals consume the Tier 0 token primitives (z-layers from `S11-TD-UI-ZINDEX-LAYERS` DONE in Sprint 14; typography from `S11-TD-UI-FONT-CONSTANTS` DONE; overlay alpha from `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` DONE; flex strips from `S11-TD-UI-FLEX-STRIPS` DONE; global UI spec from `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` DONE). New integration test in `tests/integration/hand_ui/` (path TBD) asserting drag-state visual properties using ECS marker / color / z-layer assertions. **Read-only over client-side ephemeral drag state** (ADR-012 binding; no new server-authoritative state, no protocol-shape change). No final-art replacement (`PAW-TD-*-a` preserved across PAW-002..PAW-006). `QA-COND-0005` + `QA-COND-0006` preserved (drag-state visuals do NOT advance Standard-tier accessibility or playtest validation). |
| S11-UX-BOARD-RENDERING-SPEC | Board Rendering Spec (Tier 3 rank 14, doc-only) -- author canonical board rendering spec at `docs/ux/board-rendering-spec.md` covering cell rendering, unit placement, range overlays, status icon legend, ghost preview opacity | ux-designer + art-director + producer | 0.75 | Roadmap rank 14 (Tier 3, Should, PROMPT 685 row 6, re-validated by PROMPT 802 §3.7 B1 / §4 Tier 3.2); **depends on rank 6 `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` DONE** (Sprint 14 PROMPT 922). **Story file NEW; must be authored before activation** at `production/epics/board-rendering/story-XXX-board-rendering-spec.md` (slug TBD by story-authoring prompt). | Story file authored on `origin/main` via a separate story-authoring prompt before Sprint 15 activation. `/story-readiness` passes against Sprint 15 activation HEAD. Spec authored at `docs/ux/board-rendering-spec.md` (NEW) and references the now-existing parent `docs/ux/global-ui-design-spec.md` (Sprint 14 PROMPT 911 / 912 / 922). UX-designer + art-director + producer sign off in commit message or evidence doc. Spec covers: Status / No-Claim Banner mirroring the global UI design spec banner; cell rendering rules; unit placement rules; range overlay rules; status icon legend (`S11-UX-BOARD-STATUS-ICON-LEGEND-001` future-candidate is folded as a spec section, NOT as a separate story); ghost preview opacity rules (`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` future-candidate folded as a spec section, NOT as a separate story); accept-risk dispositions preserved verbatim. **Doc-only**; no production-code change under `client/` / `server/` / `shared/` / `tests/` by this row. `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` preserved. |

**Should Have subtotal**: ~1.25 estimated days. Both rows are
file-disjoint (hand-ui surface vs `docs/ux/`) and producer-decision-unblocked
(no PROMPT 802 §9 producer-decision references in either row). Each
requires a story-authoring prompt before activation and a
`/story-readiness` pass before `/dev-story`.

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 | UI Interaction State Primitives (Tier 0 Should adjacent) -- author hover / focus / pressed / disabled primitive set; pair with the now-DONE rank 6 global UI design spec | UI programmer + ux-designer | 1.0 | Roadmap §"Tier 0 Should-Priority Adjacent Row" (net-new, PROMPT 802 §3.9 G7). **Story file NEW; must be authored before activation** at `production/epics/ui-clean-pass/story-XXX-ui-interaction-state-primitives.md` (slug TBD by story-authoring prompt). | Story file authored on `origin/main` via a separate story-authoring prompt before Sprint 15 activation. `/story-readiness` passes against Sprint 15 activation HEAD. Interaction-state primitive module authored under `client/src/ui/design_tokens/` (alongside the Sprint 14 DONE Tier 0 modules `z_layers.rs` / typography / flex strips / overlay alpha). Hover / focus / pressed / disabled visual states exported as named tokens consumed by Tier 1 button surfaces. Migration of existing Sprint 14 button surfaces (lobby buttons per `S11-UX-LOBBY-BUTTON-HITTARGETS` DONE; auction bid buttons per `S11-UX-AUCTION-FEATURED-CARD` DONE; HUD action buttons per `S11-UX-HUD-TOP-STRIP-LAYOUT` DONE) **out of scope for Sprint 15** -- the primitive module is authored and the spec body in `docs/ux/global-ui-design-spec.md` is amended to reference it, but per-surface migration is a follow-on story. New integration test in `tests/integration/ui_clean_pass/` (path TBD) asserts primitive module shape (named tokens, default values, no inline literal regressions). `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` preserved (no Standard-tier hit-target conformance, no playtest validation, no final-art replacement). |

**Nice to Have subtotal**: ~1.00 estimated days. Lands only if Must
Have + Should Have closure is on track. Authoring the primitive module
without per-surface migration is the minimum viable Sprint 15
deliverable; per-surface migration deferred to Sprint 16+ to keep
Sprint 15 small.

---

## Carryover from Sprint 14

| Source row (Sprint 14) | Disposition into Sprint 15 |
|------------------------|----------------------------|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 14 Should Have, `ready` after PROMPT 987 close-out -- the only un-closed row of Sprint 14; human-operator-blocked cosmetic visual check; originally Sprint 13 carry into Sprint 14) | Pulled forward as Sprint 15 **Must Have** human-operator-blocked carry (**promoted Should -> Must in Sprint 15** to surface the multi-sprint carry as a sprint-level blocker requiring explicit human-operator scheduling). Closure remains gated on human screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases. No LLM `/story-done` is authorised; PROMPT 822 / PROMPT 823 / PROMPT 894 / PROMPT 987 disposition preserved. Story file unchanged at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`. Evidence target shifts from `production/qa/evidence/sprint-14-hud-timer-visual-check/` (NEW, unpopulated) to `production/qa/evidence/sprint-15-hud-timer-visual-check/` (NEW). |
| `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row-status (cited in `S13-CONN-LOST-UX-001` evidence per AC8 design; row-status flip deferred at Sprint 13 closeout PROMPT 894 and Sprint 14 closeout PROMPT 987) | Pulled forward as Sprint 15 **Must Have** paperwork-only row-status flip. Discharges the long-carried evidence-vs-status gap without consuming code-modification capacity. |

(All 16 closed Sprint 14 rows are preserved unchanged on `origin/main`
per the PROMPT 987 closeout disposition. None are reopened or revisited
by Sprint 15.)

## Conditions Carried Forward Unchanged (NOT closed by Sprint 15)

Sprint 15 explicitly preserves and does NOT claim closure for any of:

- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains
  **OPEN**. Sprint 13 story 017 AC12 forbid-auto-closure was preserved
  through Sprint 13 and Sprint 14; Sprint 15 candidate stories do not
  touch the two-client GAME_OVER surface. Sprint 15 activation MUST NOT
  silently close `S8-QA-001-W1`.
- **`QA-COND-0005`** -- Standard-tier accessibility remains **accepted-risk**
  (friend-game scope only). Sprint 15 hand-drag-state visuals, board
  rendering spec, and interaction state primitives are **friend-game
  visual polish only** per roadmap §"Friend-Game Scope vs Standard-Tier-
  Accessibility Scope Boundary". The L5 `LOBBY_BUTTON_HEIGHT = 30.0`
  defect (PROMPT 802 §3.1 L5) remains accepted-risk under `QA-COND-0005`;
  the Sprint 14 `S11-UX-LOBBY-BUTTON-HITTARGETS` row DONE was friend-game
  scope only and did NOT close `QA-COND-0005`. Sprint 15 does NOT pursue
  WCAG contrast ratios, ≥44px hit-targets, full keyboard navigation,
  screen reader support, colorblind modes, or text scaling.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. Sprint 15 UI clean-pass closeout does
  NOT advance playtest validation even when the surface is visibly
  polished.
- **`PAW-TD-*-a`** -- placeholder-art accept-risk across PAW-002..PAW-006
  remains in place. Sprint 15 layout / composition / primitive work
  does NOT advance placeholder-art resolution; PROMPT 802 §7 places
  final-art work explicitly out of audit scope.
- **PROMPT 683-era runtime divergence question** -- folded into Sprint 12
  story 019 `closed-with-conditions / cannot-reproduce` (after second
  time-box exhaustion). Sprint 15 does NOT claim this question closed.
  **A third same-scope retest is NOT authorised** per `TQ-S12-C2`.
- **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. **NO retry**
  is in scope for Sprint 15. Stage remains `Polish`.
- **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed
  fixed. Sprint 15 hand-drag-state visuals work does not reproduce or
  fix the underlying drag-runtime behaviour (`S12-UX-HAND-DRAG-STATE-VISUALS-001`
  is layout / visual state work over already-extant client-side drag
  ephemeral state; per ADR-012 it adds no new server-authoritative state
  and does not modify the runtime drag pipeline).
- **`TQ-S12-C1..C7`** -- all 7 Sprint 12 Team-QA conditions preserved
  verbatim. `TQ-S12-C7` explicitly NOT closed by any Sprint 15 row.
- **Sprint 14 / Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts**
  -- preserved unchanged. Sprint 14 disposition `closed-with-conditions`
  per PROMPT 987; Sprint 13 `closed-with-conditions` per PROMPT 894;
  Sprint 12 `closed-with-conditions` per PROMPT 817; Sprint 11
  `closed-with-conditions` per PROMPT 792; Sprint 10
  `closed-with-conditions` per PROMPT 763.
- **All 16 closed Sprint 14 `/story-done` closures** (PROMPT 903 / 908 /
  909 / 919 / 921 / 922 / 931 / 939 / 942 / 953 / 956 / 960 / 962 / 972
  / 974 / 976) preserved unchanged on `origin/main`. Sprint 15 does NOT
  reopen any of them.

If any condition above changes during Sprint 15, it requires its own
separate story file and explicit disposition -- it cannot be silently
folded into another Sprint 15 row.

## Wider Sprint 15 Backlog (NOT scheduled into this draft; deferred)

The following candidates remain in the broader backlog and are **NOT
scheduled** into this Sprint 15 draft. They may be pulled by a producer
revision before activation, or deferred further to Sprint 16+:

### Deliberately deferred to Sprint 16+ (size or coordination overhead)

- **`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`** (Roadmap rank 13, Tier 3, Should,
  **1.5d**, net-new, PROMPT 802 §3.3 HA1 / §3.3 HA5 / §4 Tier 3.1) --
  refactor touches **hand + shop + auction together** (per PROMPT 802 §8);
  while Tier 1 surfaces are now stable (all Sprint 14 ranks 7-12 DONE),
  pulling this row into Sprint 15 would inflate the sprint into a
  mega-sprint (1.5d + ~0.5d authoring + ~0.5d integration friction). Defer
  to Sprint 16+ as the headline Must row of a focused refactor sprint.
- **Tier 2 cosmetic captures bundle** -- 12 already-tracked future
  candidates per PROMPT 802 §9 producer-decision-5 (`S11-UX-LOBBY-ROOM-CODE-EYEBALL-001`,
  `S11-UX-LOBBY-OPP-SLOT-DISAMBIGUATION-001`, `S11-DRAFT-INITIAL-OVERLAY-EYEBALL-001`,
  `S11-UX-SHOP-SLOT-AFFORDANCE-001`, `S11-UX-SHOP-INLINE-GOLD-READ-ORDER-001`,
  `S11-UX-AUCTION-SETTLEMENT-VISUAL-EYEBALL-001` (depends on `S8-QA-001-W1`
  out of UI clean-pass scope), `S11-HU-DRAG-FEEDBACK-DIFFERENTIATION-001`,
  `S11-UX-RESULT-RETURN-TO-LOBBY-001`, `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`,
  `S11-UX-BOARD-STATUS-ICON-LEGEND-001`, `S11-UX-HUD-TIMER-URGENCY-VISUAL-001`).
  Each is 0.10d-0.25d manual capture work. Bundled candidate: `S15-UX-CAPTURES-CLEAN-PASS-001`
  (per roadmap §"Tier 2 Cosmetic / Eyeball Captures") if a producer
  activates it. **Not pulled into Sprint 15 draft.** Two of these
  (`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` + `S11-UX-BOARD-STATUS-ICON-LEGEND-001`)
  are folded as spec sections into Sprint 15 Should Have
  `S11-UX-BOARD-RENDERING-SPEC` rather than as separate captures.

### Server hardening test parity (Sprint 11/12 backlog)

- **`S11-TD-NET-001`, `S11-TD-NET-002`, `S11-TD-NET-003`** -- server
  hardening test parity from Sprint 11/12 backlog. Defer to a focused
  server-hardening sprint.
- **`S11-TD-PRISM-COV-001`** -- Cluster 2C advisory coverage gap on
  `S2CPrismRewardDropped` + `S2CPrismRespawned` (per-row drain-or-delete
  disposition recorded in Sprint 13 story 008 closure; advisory test
  coverage follow-on).
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

### PROMPT 803 §5 Should/Nice rows not pulled into Sprint 13

- `S13-LOBBY-CONFIRMCLASS-SENDER-001`, `S13-COOCCUPANCY-INVARIANT-001`,
  `S13-PHASE-IDEMPOTENCY-CLIENT-001`, `S13-ADR012-LOBBY-OPTIMISM-001`,
  `S13-S2C-SUCCESS-LOG-001`, `S13-OBSERVABLE-PRODUCER-AUDIT-001`,
  `S13-PLUGIN-REGISTRATION-INVARIANT-001`, `S13-IGNORE-ATTRIBUTE-DRIFT-001`,
  `S13-MANUAL-RUNBOOK-AUTOMATION-001` (gated on Sprint 13 story 017
  outcome; NOT authorised to advance `S8-QA-001-W1` in Sprint 15),
  `S13-PROTO-MESSAGE-ID-001`. All carried forward from Sprint 13 / 14
  backlog unchanged.

### Smoke evidence hygiene (Sprint 14 PROMPT 983 by-products)

- **`S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`** (candidate; net-new):
  remove `count_with_image_node` pre-existing dead-code warning at
  `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43`
  surfaced by Sprint 14 PROMPT 983 smoke. NOT pulled into Sprint 15
  draft; it is a Nice-to-Have hygiene row that touches `tests/` and is
  forbidden under PROMPT 988 paperwork-only draft scope. If a producer
  pulls it into Sprint 15 at activation, it would replace one of the
  Sprint 15 Nice rows or be added as an additional Nice row; estimated
  effort 0.1d.
- **`S15-OPS-APPCOMPAT-MANIFEST-001`** (candidate; net-new): embed a
  Windows manifest with `level="asInvoker"` on the `spawn_range_live_update_contract`
  test binary (Option B from PROMPT 983 §"Windows AppCompat Workaround").
  Removes the per-run rename workaround. NOT pulled into Sprint 15 draft;
  ops / build-system change touches `tests/` or `Cargo.toml` configuration
  and is forbidden under PROMPT 988 paperwork-only draft scope. Producer
  may pull at activation as Nice; estimated effort 0.1d-0.25d.

---

## Required Sprint 15 Story Docs

PROMPT 988 (this draft) does NOT author any new story files.

The Sprint 15 Must Have rows are paperwork-only:

- `S11-HUD-TIMER-EYEBALL-VISUAL-001` -- story file at
  `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
  ALREADY EXISTS on `origin/main` from Sprint 13 (PROMPT 822 author /
  PROMPT 823 `/story-readiness` READY); carried forward unchanged.
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` -- no story file
  required; this is a `production/sprint-status.yaml` paperwork-only
  row-status flip.

The three Sprint 15 Should + Nice rows require **new story files** to be
authored on `origin/main` via separate story-authoring prompts BEFORE
Sprint 15 activation, mirroring the Sprint 14 PROMPT 878 / 879 / 880 /
881 authoring pattern:

| Planned ID | Required story file | Source-of-truth (current) |
|------------|---------------------|---------------------------|
| S12-UX-HAND-DRAG-STATE-VISUALS-001 | `production/epics/hand-ui/story-XXX-hand-drag-state-visuals.md` (slug TBD; NEW) | NOT on main; needs story-authoring prompt before Sprint 15 activation. PROMPT 802 §9 producer-decision-5 candidate (visual language for drag-states) MAY apply -- producer to confirm at story-authoring time. |
| S11-UX-BOARD-RENDERING-SPEC | `production/epics/board-rendering/story-XXX-board-rendering-spec.md` (slug TBD; NEW) | NOT on main; needs story-authoring prompt before Sprint 15 activation. Doc-only spec; no producer-decision blocker. |
| S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 | `production/epics/ui-clean-pass/story-XXX-ui-interaction-state-primitives.md` (slug TBD; NEW) | NOT on main; needs story-authoring prompt before Sprint 15 activation. PROMPT 802 §9 producer-decision candidate (visual language for interaction states) MAY apply -- producer to confirm at story-authoring time. |

The Sprint 14 closed story files (16 rows) remain on `origin/main`
unchanged; Sprint 15 does NOT touch any of them and MUST NOT reopen any
of them.

## Explicitly NOT Claimed by Sprint 15 Draft

PROMPT 988 (this draft) does NOT claim, and Sprint 15 activation MUST NOT
claim, any of the following:

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
- **Sprint 15 activation** (this is a draft, not an activation)
- Sprint 15 sprint-status `active` top-level row
- underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`; third same-scope retest
  NOT authorised per `TQ-S12-C2`)
- closure of `S8-QA-001-W1`
- closure of `TQ-S12-C7`
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked
  carry; promoted to Must Have but closure remains gated on human-operator
  screenshot capture; no LLM `/story-done` authorised)
- Sprint 14 row reopen (any of the 16 closed Sprint 14 rows)
- full UI clean-pass repair beyond the Sprint 15 candidate rows above
  (Tier 3 rank 13 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` deferred to
  Sprint 16+; Tier 2 cosmetic captures bundle deferred unless a producer
  pulls them at activation)
- creation of `production/qa/qa-plan-sprint-15.md` -- this is **explicitly
  out of Sprint 15 draft scope**; the QA plan MUST be authored separately
  via `/qa-plan sprint-15` after activation and before any Sprint 15
  `/dev-story`
- creation of any Sprint 15 story file by this draft (the three Should +
  Nice candidates require separate story-authoring prompts)
- `sprint_15_activation:` block in `production/sprint-status.yaml`
  (PROMPT 988 only appends a `next_sprint_15_draft:` block; activation is
  a separate prompt)

## Suggested First Parallel Batch (post-activation)

The orchestrator that activates Sprint 15 should batch the post-activation
work as follows. Per CLAUDE.md / `.claude/docs/coordination-rules.md`,
"Launch only actually ready work; do not invent parallelism to satisfy a
quota."

### Parallel batch (file-disjoint, ready post-activation; can launch concurrently)

After Sprint 15 activation, after the three net-new story files are
authored on main, after each story file passes `/story-readiness`, and
after `/qa-plan sprint-15` lands:

| Story | File scope | Parallel-safe with |
|---|---|---|
| `S12-UX-HAND-DRAG-STATE-VISUALS-001` | `client/src/ui/hand_ui/` + `tests/integration/hand_ui/` (NEW path) + `production/epics/hand-ui/story-XXX-*.md` | Board rendering spec (different file scope: hand-ui vs docs/ux); Interaction state primitives (hand-ui surface vs design_tokens module) |
| `S11-UX-BOARD-RENDERING-SPEC` | `docs/ux/board-rendering-spec.md` (NEW) + `production/epics/board-rendering/story-XXX-*.md` | All other rows (doc-only; no `client/` / `server/` / `shared/` / `tests/` touch) |
| `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` | `client/src/ui/design_tokens/interaction_states.rs` (NEW) + `docs/ux/global-ui-design-spec.md` amendment + `tests/integration/ui_clean_pass/` (NEW path) + `production/epics/ui-clean-pass/story-XXX-*.md` | Hand drag state visuals (design_tokens module vs hand-ui surface); Board rendering spec (design_tokens module vs board doc) |

Pairwise file-disjoint by surface: hand-ui (drag visuals) / board-rendering
doc / design-tokens module (interaction states). All three depend on
Sprint 14 Tier 0 modules which are already on main.

### Serial / human-blocked (do NOT include in the parallel batch)

| Item | Reason |
|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | **Human-operator-blocked**. Requires manual 2-client run + screenshot capture; no LLM `/story-done` authorised. Schedule producer + human-operator slot; track separately. |
| `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` | **Shared-status writer**. Edits `production/sprint-status.yaml`; must be serialized with any `/story-done` paperwork against Sprint 15 per CLAUDE.md / `.claude/docs/coordination-rules.md` ("Keep one shared-status writer active at a time"). Sequence inside the closeout / `/story-done` queue, NOT in the parallel implementation batch. |
| Per-story `/story-readiness` reruns (3 rows) | Read-only but serialize through the producer / orchestrator; cannot run before each Sprint 15 story file lands on main. |
| `/qa-plan sprint-15` authoring | Single shared-status writer; must complete before any `/dev-story` per CLAUDE.md / QA-plan precedent. |

### Pre-activation prerequisites (no parallelism; must complete before activation prompt is launched)

1. Three story-authoring prompts (one per Should + Nice candidate), each
   in its own worktree on its own branch (`story-authoring/sprint-15-hand-drag-state-visuals`,
   `story-authoring/sprint-15-board-rendering-spec`,
   `story-authoring/sprint-15-ui-interaction-state-primitives`). File-disjoint by surface
   so these three can themselves run as a parallel batch.
2. Integration of the three story-authoring branches to `origin/main`
   (mirroring Sprint 14 PROMPT 893 four-way merge precedent; can land as
   one `--no-ff` per branch or one consolidated merge).
3. Sprint 15 activation prompt (mirrors PROMPT 826 / PROMPT 897) flips
   `production/sprint-status.yaml` top-level and appends a
   `sprint_15_activation:` block. PROMPT 988 itself does NOT do this.

## Sequencing Notes

Per `docs/ux/ui-clean-pass-roadmap.md` and PROMPT 988 small-plan framing:

1. **Sprint 14 Tier 0 (ranks 1-6) is DONE on main.** Sprint 15 Should +
   Nice rows can consume Tier 0 token modules directly without
   re-authoring primitives.
2. **Sprint 14 Tier 1 headline (ranks 7, 10, 12) is DONE on main.**
   Sprint 15 hand-drag-state visuals row does not touch any of these
   surfaces; the board rendering spec row is doc-only.
3. **Sprint 14 remaining Tier 1 Must rows (ranks 8, 9, 11) are DONE on
   main.** Sprint 15 does not touch any of these surfaces.
4. **The Sprint 14 carry (`S11-HUD-TIMER-EYEBALL-VISUAL-001`) is
   parallel-safe with all other rows** (cosmetic visual check only;
   no shared host module; no code change unless a regression is found)
   but is human-operator-blocked; sequence outside the parallel
   implementation batch.
5. **Tier 0 Should adjacent row** (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`)
   touches `client/src/ui/design_tokens/` -- a host module already shared
   by Sprint 14 Tier 0 ranks 1-3 + 5 which are all DONE. The new
   `interaction_states.rs` file is additive (not a refactor); no
   collision expected. Spec amendment in `docs/ux/global-ui-design-spec.md`
   is additive (new section referencing the new primitive module).
6. **Tier 3 rank 14 doc-only spec** (`S11-UX-BOARD-RENDERING-SPEC`) is
   parallel-safe with all other rows (doc-only; no `client/` / `server/`
   / `shared/` / `tests/` touch).
7. **Tier 1 Should adjacent row** (`S12-UX-HAND-DRAG-STATE-VISUALS-001`)
   touches hand UI only -- per roadmap "independent of the 14;
   orthogonal to ranks 7-12; touches hand UI only".
8. **No PROMPT 802 §9 producer-decisions are blocking for the chosen
   rows.** Decisions 1-4 are RESOLVED on Sprint 14 (decisions 2 / 3 / 4
   per PROMPT 911 + 922 + 933 + 935 + 967). Decisions 5 (cosmetic captures
   bundle) and 6 (post-Tier-1 polish bar) are NOT relevant to the chosen
   Sprint 15 rows. If a producer pulls the deferred Tier 2 cosmetic
   captures bundle into Sprint 15 at activation, decision 5 must be
   confirmed.
9. **PROMPT 802 §3.9 G8 lobby file collision risk** -- inapplicable to
   Sprint 15 (no row touches `client/src/ui/lobby.rs`).
10. **Sprint 14 HUD top + bottom strip collision precedent** -- inapplicable
    to Sprint 15 (no row touches `client/src/ui/hud/mod.rs`).

## Cargo Resource Policy (this draft)

**Not applied** -- PROMPT 988 is a paperwork-only draft. No `cargo`
command was invoked. `$env:CARGO_TARGET_DIR`,
`$env:CARGO_PROFILE_DEV_DEBUG`, `$env:CARGO_PROFILE_TEST_DEBUG`,
`$env:CARGO_INCREMENTAL`, `$env:RUSTFLAGS` were not set. Cargo resource
policy was not applied because no Cargo command was needed.

Sprint 15 implementation prompts (post-activation `/dev-story` workers
and integration merges) MUST apply the binding Windows / MSVC Cargo
resource policy per the Sprint 13 / Sprint 14 precedent (PROMPT 829 /
833 / 884 / 902 / 906 / 907 / 912 / 917 / 918 / 930 / 938 / 941 / 951 /
955 / 959 / 961 / 970 / 973 / 975 worker + integration prompts):

- `$env:CARGO_TARGET_DIR = "D:\_DEV\cargo-target\ccgs-msvc"`
- `$env:CARGO_PROFILE_DEV_DEBUG = "0"`
- `$env:CARGO_PROFILE_TEST_DEBUG = "0"`
- `$env:CARGO_INCREMENTAL = "0"`
- `$env:RUSTFLAGS = "-C debuginfo=0 -C link-arg=/DEBUG:NONE"`

## Provisional Next Launchable Prompts (after this draft lands)

PROMPT 988 (this draft) lands as a Sprint 15 sprint-plan draft on
`origin/main` via a separate paperwork-only integration prompt (not
performed by PROMPT 988 itself; PROMPT 988 commits to its own branch
`sprint-plan/sprint-15-draft` and pushes the branch). After this draft
lands:

1. **Sprint 15 story-authoring prompts** (three; runnable as a parallel
   batch on file-disjoint worktrees):
   - `story-authoring/sprint-15-hand-drag-state-visuals` ->
     `production/epics/hand-ui/story-XXX-hand-drag-state-visuals.md` (NEW).
   - `story-authoring/sprint-15-board-rendering-spec` ->
     `production/epics/board-rendering/story-XXX-board-rendering-spec.md` (NEW).
   - `story-authoring/sprint-15-ui-interaction-state-primitives` ->
     `production/epics/ui-clean-pass/story-XXX-ui-interaction-state-primitives.md` (NEW).
2. **Sprint 15 story-authoring integration prompts** -- one `--no-ff`
   merge per branch or one consolidated merge (mirrors PROMPT 893
   precedent for Sprint 14 four-way authoring batch).
3. **Sprint 15 activation prompt** -- flips top-level `sprint: 14 -> 15`
   and `status: closed-with-conditions -> active`; appends
   `sprint_15_activation:` block; prepends ACTIVATED banner to this file
   (mirrors PROMPT 826 / PROMPT 897 pattern).
4. **`/qa-plan sprint-15`** -- after activation; before any `/dev-story`.
5. **Per-story `/story-readiness` reruns** against Sprint 15 activation
   HEAD for each of the 4 Sprint 15 story files (Must `story-014-hud-timer-eyeball-visual-check.md`
   carry, plus the three NEW Should + Nice files).
6. **Sprint 15 row-flip paperwork prompt** for
   `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row-status
   (Must Have; paperwork-only; serializes through
   `production/sprint-status.yaml`).
7. **Sprint 15 human-operator screenshot capture session** for
   `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Must Have; human-operator-blocked;
   schedule producer + human-operator slot).
8. **Sprint 15 `/dev-story` runs** for the three Should + Nice rows
   (parallel-safe per "Suggested First Parallel Batch" section above).
9. **Sprint 15 integration prompts** per row (mirrors Sprint 14 pattern).
10. **Sprint 15 `/story-done` paperwork** per row (serialized
    shared-status writer).
11. **Sprint 15 smoke-check** (`/smoke-check`) at end of sprint.
12. **Sprint 15 Team-QA** (`/team-qa sprint`) after smoke.
13. **Sprint 15 close-out disposition** (paperwork-only close-out
    prompt; mirrors PROMPT 817 / 894 / 987 pattern). Expected disposition:
    `closed-with-conditions` (NOT `closed`) -- the same accept-risk
    conditions carry forward.

---

## Files Changed By PROMPT 988

| File | Status | Notes |
|---|---|---|
| `production/sprints/sprint-15.md` | NEW | This draft. Sprint 15 NOT activated. |
| `production/sprint-status.yaml` | MODIFIED | `next_sprint_15_draft:` block appended at EOF; `updated:` annotation refreshed with PROMPT 988 prefix preserving PROMPT 987 narrative as `# Previous:` comment chain. No row flips. `sprint_14_closeout:` block NOT modified. Sprint 14 top-level `sprint: 14 / status: closed-with-conditions / stage: Polish` preserved. |
| `production/session-state/active.md` | MODIFIED | PROMPT 988 banner prepended above PROMPT 987 banner. |
| `production/session-state/codex-orchestrator-state.md` | MODIFIED | PROMPT 988 section prepended above PROMPT 987 section. |

Explicitly **NOT** touched by PROMPT 988 (forbidden by task scope):

- `client/`, `server/`, `shared/`, `tests/` -- no production or test code edits.
- `production/stage.txt` -- no stage advance.
- `production/qa/qa-plan-sprint-15.md` -- NOT authored by PROMPT 988.
- `production/qa/qa-plan-sprint-14.md` -- NOT modified.
- `production/gate-checks/*` -- no gate-check retry or edit.
- `production/sprints/sprint-14.md` / `sprint-13.md` / `sprint-12.md` / `sprint-11.md` / `sprint-10.md` -- NOT modified.
- Sprint 14 / 13 / 12 / 11 / 10 story files under `production/epics/` -- NOT modified or reopened.
- Sprint 15 candidate story files (the three Should + Nice rows) -- NOT authored by PROMPT 988; authoring is a separate set of prompts.
- Release artifacts, release-checklist, launch-checklist, changelog, patch notes -- NOT modified or created.
- `.claude/settings.json`, `.octogent/`, `.claude/scheduled_tasks.lock`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `Trunk.toml`, `.github/` -- NOT modified.

PROMPT 988 did NOT run `/dev-story`, `/smoke-check`, `/team-qa`,
`/gate-check`, `/release-check`, `/story-done`, `/story-readiness`,
`/qa-plan`, or any `cargo` / `trunk` command. PROMPT 988 did NOT
activate Sprint 15. PROMPT 988 did NOT advance stage from Polish.

---

**Final status line**

988: SPRINT-15-PLAN-DRAFT: drafted-not-activated
