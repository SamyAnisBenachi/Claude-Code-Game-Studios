# Sprint 18 -- DRAFT (Polish stage; Sprint 17 closed-with-conditions, evidence reconcile pending)

> **PROMPT 1285 paperwork-only Sprint 18 plan draft (2026-05-18).**
> Source-of-truth at authoring: `origin/main@d73e25e49519a214f8fb0fefa1e78351ccd74795`
> (PROMPT 1281 `cargo fmt --check` drift repair, landed by PROMPT 1283).
> Strict fast-forward descendant of `origin/main@4f98fe5ffa4b7edbb9beb8da542e17da451ee302`
> (PROMPT 1279 Sprint 17 close-out-with-conditions disposition).
> Worktree: `D:/tmp/sprint-18-plan-draft-1285`.
> Branch: `work/sprint-18-plan-draft-1285`.
>
> **Status**: `draft -- authored 2026-05-18 by PROMPT 1285`.
> **Sprint 18 is NOT activated by this draft.** Top-level `sprint: 17`,
> `status: closed-with-conditions`, `stage: Polish` in
> `production/sprint-status.yaml` are preserved verbatim. Activation is a
> separate explicit prompt that flips `sprint: 17 -> 18` and
> `status: closed-with-conditions -> active`, preserves `stage: Polish`,
> attaches a `sprint_18_activation:` block, and prepends an ACTIVATED
> banner to this file. PROMPT 1285 does **not** edit
> `production/sprint-status.yaml`, `production/stage.txt`,
> `production/session-state/*`, `production/qa/**`,
> `production/gate-checks/**`, `production/epics/**`, source code, tests,
> Cargo files, CI, or any sprint file other than this one.
>
> **Stage**: `Polish` (UNCHANGED). PROMPT 761 `Polish->Release` gate-check
> `FAIL` preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry**
> is in scope for Sprint 18 and MUST NOT be attempted by activation.
> Sprint 18 is **NOT** a `Polish->Release` sprint.
>
> **Dates (provisional)**: 2026-09-10 -> 2026-09-23 (10 workdays;
> contiguous with the locked Sprint 17 window 2026-08-27 -> 2026-09-09).
> Activation may relock these against the activation HEAD.

---

## 0. Activation Blockers (must clear before Sprint 18 activation)

This draft is authored ahead of the activation gate. Activation MUST NOT
proceed until all of the following are resolved on `origin/main`:

1. **PROMPT 1284 Sprint 17 post-fmt smoke rerun** -- treated as pending
   for activation-sequencing purposes by this draft. PROMPT 1284 is the
   serial gate that supplies fresh smoke evidence for the Sprint 17
   close-out, and its result must be reconciled into the closeout block
   before Sprint 18 is activated. No Sprint 18 prompt may consume PROMPT
   1284 evidence as a committed fact until the closeout is reconciled.
2. **Sprint 17 closeout evidence reconcile** -- PROMPT 1279 closed
   Sprint 17 against stale smoke evidence (the
   `sprint_17_closeout.smoke_evidence.missing_artifact_condition` block
   in `production/sprint-status.yaml` records "No durable tracked
   `reports/PROMPT-1277*` artifact and no tracked
   `production/qa/smoke-sprint-17*` report were found on `origin/main`").
   A follow-up reconcile prompt must re-bind the Sprint 17 closeout to
   the PROMPT 1284 smoke artifact (or successor) on `origin/main`
   before Sprint 18 activation. Until that reconcile lands, the
   "closed-with-conditions" disposition for Sprint 17 is provisional
   from an evidence standpoint, even though the top-level status flip
   itself stands.
3. **Sprint 18 plan draft landed (this file)** -- PROMPT 1285 produces
   it on a worker branch; the orchestrator integrates separately. This
   file MUST be on `origin/main` before activation.
4. **Sprint 18 QA plan absent and out of scope here** --
   `production/qa/qa-plan-sprint-18.md` is absent on `origin/main` and
   is **NOT** authored by PROMPT 1285. Authoring the QA plan is a
   separate, sequenced post-activation prompt (`/qa-plan sprint-18`),
   mirroring the Sprint 17 PROMPT 1100 pattern. No `/dev-story` may
   run against Sprint 18 rows before the QA plan exists on
   `origin/main`.

If any of (1) or (2) regresses (e.g. PROMPT 1284 returns FAIL or BLOCKED
and is not subsequently superseded by a PASS / PASS-WITH-WARNINGS
rerun), activation is blocked and this draft must be revised before
activation proceeds.

---

## 1. Goal

Sprint 18 is a focused **Polish-stage UI cohesion + observability
sprint**: convert the documented Sprint 18 candidate story files into
closed rows, finish the auction-won card disposition unblock (PROMPT
1262 / story 020), and complete the ui-clean-pass "wave 2" cohort
(stories 020 through 027) plus the hand-ui mana-preview / idle-playable
affordance follow-ups. Several rows are already implemented on
`origin/main` and need only `/story-done` paperwork after activation;
others remain unimplemented and need `/story-readiness` then
`/dev-story` against the Sprint 18 activation tip.

Sprint 18 is **NOT** a Polish->Release activation and makes **NO**
release / RC / full-game / accessibility-Standard-tier / playtest
validation claims (see Section 7 below).

---

## 2. Capacity and Active Set

**Provisional capacity**: ~7.5d (10 workdays minus QA / reserve buffer).
**Total active scope**: ~5.6d before any carry adjustments. Activation
may trim the Should / Nice tiers if capacity is reassessed.

### 2.1 Must Have (4 rows; ~2.25d)

| ID | Story file | Est. | Source / Notes |
|---|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` | 0.25d | Sprint 13 -> 14 -> 15 -> 16 -> 17 -> 18 carry. Human-operator-blocked; no LLM `/story-done` authorised. Carried from Sprint 17 close-out per `sprint_17_closeout.rows_carried_forward[0]`. |
| `S18-AUCTION-WON-CARD-DISPOSITION-001` | `production/epics/shop-auction-ui/story-020-auction-won-card-disposition.md` | 0.75d | PROMPT 1262 unblock target. PROMPT 1263 verdict: blocked only by Sprint 18 activation. Highest-leverage P0 implementation surface on the Sprint 18 candidate roster. Branch `integrate/auction-won-card-disposition-1141` outstanding per PROMPT 1287 §0. |
| `S18-UI-PLAY-AREA-CONTAINER-001` | `production/epics/ui-clean-pass/story-020-ui-play-area-container.md` | 0.75d | PROMPT 1180 Lane A. Structurally enables shop / auction / hand / placement-action panels to stop colliding with the HandBar + FooterBar. Sequencing-sensitive: this row before `S18-UI-CARD-ART-AND-LABEL-STRIP-001`. |
| `S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001` | `production/epics/ui-clean-pass/story-027-ui-layout-contract-doc-and-lint.md` | 0.5d | Effectively implemented per PROMPT 1232 / PROMPT 1263 (PROMPT 1188). Activation-time `/story-readiness` then `/story-done` paperwork only; if any AC is not satisfied at the activation tip, escalate as a thin implementation prompt. |

### 2.2 Should Have (6 rows; ~2.5d)

| ID | Story file | Est. | Source / Notes |
|---|---|---|---|
| `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001` | `production/epics/hand-ui/story-022-hand-mana-preview-during-drag.md` | 0.5d | Implementation landed on `origin/main` per PROMPT 1287 §2 inventory (commit `8d0a3d3`, PROMPT 1228). Activation-time `/story-readiness` then `/story-done` paperwork only. |
| `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001` | `production/epics/hand-ui/story-023-hand-idle-playable-affordance.md` | 0.5d | Implementation landed on `origin/main` per PROMPT 1287 §2 inventory (commits `50b66ad` + `4c75cec`, PROMPT 1239 + PROMPT 1243). Activation-time `/story-readiness` then `/story-done` paperwork only. |
| `S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001` | `production/epics/ui-clean-pass/story-021-ui-viewport-invariant-live-harness.md` | 0.25d | Implementation landed on `origin/main` per PROMPT 1287 §2 inventory (commit `671c677`, PROMPT 1185). Activation-time `/story-readiness` then `/story-done` paperwork only. |
| `S18-UI-CARD-ART-AND-LABEL-STRIP-001` | `production/epics/ui-clean-pass/story-022-ui-card-art-and-label-strip.md` | 0.5d | PROMPT 1180 Lane C. **Unimplemented** at this draft tip per PROMPT 1287 §2 note. Sequenced after `S18-UI-PLAY-AREA-CONTAINER-001`. |
| `S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001` | `production/epics/ui-clean-pass/story-023-obs-snapshot-layout-fields.md` | 0.5d | Implementation landed on `origin/main` per PROMPT 1287 §2 inventory (commit `e68ac4f`, PROMPT 1229). Activation-time `/story-readiness` then `/story-done` paperwork only. |
| `S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001` | `production/epics/ui-clean-pass/story-024-ui-settings-panel-flex-relayout.md` | 0.25d | Listed as "effectively implemented on `origin/main`" by PROMPT 1232 / PROMPT 1263, but no explicit commit captured in PROMPT 1287 §2 inventory. Treat as **verify-only** before `/story-done`; if AC gaps remain, escalate as a thin implementation prompt. |

### 2.3 Nice to Have (3 rows; ~0.85d)

| ID | Story file | Est. | Source / Notes |
|---|---|---|---|
| `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` | `production/epics/ui-clean-pass/story-025-ui-interaction-state-migration-wave-2.md` | 0.35d | PROMPT 1190 J-class polish row. Unimplemented at this draft tip. May trim at activation if Must/Should capacity overruns. |
| `S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001` | `production/epics/ui-clean-pass/story-026-ui-overlay-panel-overflow-hardening.md` | 0.3d | PROMPT 1190 J-class polish row. Unimplemented at this draft tip. May trim at activation if Must/Should capacity overruns. |
| `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` | **story-authoring-needed** -- no story file on `origin/main` at this draft tip | 0.2d | Preferred follow-up for the Sprint 17 `S17-UI-HUD-OPP-MANA-CLEANUP-001` AC3 hand-reserve microbadge parent-row paperwork gap. PROMPT 1263 §3 records this as "candidate slug only, not an existing story file". **Activation MUST NOT include this row until a story file is authored under `production/epics/hand-ui/` (most likely slot) and landed on `origin/main`.** A story-authoring prompt is the prerequisite; if not authored before activation, drop from the Sprint 18 active set and re-evaluate at Sprint 19 planning. |

---

## 3. Carried Conditions

The following Sprint 17 close-out conditions are carried forward verbatim and **MUST NOT** be claimed closed by Sprint 18 activation or by any Sprint 18 row outside an explicit, scoped repair prompt:

- `S8-QA-001-W1` -- remains OPEN.
- `QA-COND-0005` -- Standard-tier accessibility remains accepted-risk / friend-game scope only.
- `QA-COND-0006` -- playtest / fun-hypothesis validation remains accepted-risk / deferred.
- `PAW-TD-*-a` -- placeholder-art accepted-risk remains in place.
- `TQ-S12-C1..C7` -- preserved; `TQ-S12-C7` is not closed.
- PROMPT 683-era runtime divergence + Sprint 12 story 019 `cannot-reproduce` -- preserved; no underlying drag-runtime bug fix is claimed.
- PROMPT 1054 P1 UI snapshot visual retest -- remains `BLOCKED-HUMAN-OPERATOR`.
- PROMPT 761 `Polish->Release` gate-check -- remains FAIL; no retry in Sprint 18.
- `S17-UI-HUD-OPP-MANA-CLEANUP-001` -- parent-row paperwork gap carried (AC3 hand-reserve microbadge source repair is on `origin/main` per `sprint_17_closeout.rows_carried_forward[1]`, but no final `/story-done` paperwork closed the Sprint 17 parent row). **Sprint 18 does NOT silently close this row.** The preferred discharge path is the new `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` follow-up (Section 2.3 above), which is itself blocked on story-authoring.
- All prior Sprint 10 through Sprint 17 `closed-with-conditions` dispositions remain preserved.

---

## 4. Conditional / Not Yet Landed Inputs

These items are mentioned in adjacent planning documents (PROMPT 1280
Krosmaga dev-proxy pack-boundary, etc.) but their story files are
**NOT on `origin/main` at the source-of-truth tip used by this draft
(`d73e25e`)**. They are recorded here as **conditional only** and MUST
NOT be treated as committed Sprint 18 scope until their story files
land on `origin/main`:

- PROMPT 1280 Krosmaga style UI implementation wave (dev-proxy pack
  boundary, hand-fan layout, board-rendering playarea targeting,
  shop-auction card-product layout, presentation-layer result/mulligan
  overlay chrome, ui-clean-pass card rendering fidelity / hover
  glossary). Untracked story files exist in some local worktrees but
  are absent from `origin/main`. Activation MUST verify each story
  file's presence on `origin/main` before including any of these rows
  in the Sprint 18 active set. If not landed, defer to Sprint 19.

---

## 5. Activation Sequence (informational; not executed by this draft)

The required serial gates after this draft lands:

1. PROMPT 1284 Sprint 17 post-fmt smoke completes and is reconciled.
2. Sprint 17 closeout evidence reconcile prompt lands on `origin/main`
   (re-binds `sprint_17_closeout.smoke_evidence` to the new artifact).
3. This Sprint 18 draft is integrated to `origin/main` (Sprint 18 plan
   file lands but no activation flip).
4. Sprint 18 activation prompt flips top-level `sprint: 17 -> 18` and
   `status: closed-with-conditions -> active`, preserves `stage:
   Polish`, attaches a `sprint_18_activation:` block at EOF of
   `production/sprint-status.yaml`, and prepends an ACTIVATED banner
   to this file.
5. `/qa-plan sprint-18` authors `production/qa/qa-plan-sprint-18.md`
   against the Sprint 18 activation tip.
6. Per-row `/story-readiness` reruns against the Sprint 18 activation
   tip for each active row before any `/dev-story` runs.
7. `/story-done` paperwork (no `/dev-story` needed) for the rows
   already implemented on `origin/main` (Section 2.2 inventory and
   Section 2.1 row 4) once the QA plan and readiness reruns clear.
8. `/dev-story` for the rows in Section 2.1 (auction-won card,
   play-area container) and Section 2.2 (card art + label strip) once
   readiness clears.
9. (Conditional) `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` story
   authoring + readiness + `/dev-story` if the row is retained.

---

## 6. Files Changed by PROMPT 1285

- `production/sprints/sprint-18.md` (this file; **CREATED**).
- `reports/PROMPT-1285-sprint-18-plan-draft-branch-prep.md` (mandatory
  final report; `reports/` is gitignored; not staged or committed by
  this commit).

**Files explicitly NOT touched by PROMPT 1285**: `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`,
`Trunk.toml`, `production/stage.txt`, `production/sprint-status.yaml`,
`production/sprints/sprint-1.md` through `sprint-17.md`,
`production/qa/**`, `production/gate-checks/**`,
`production/epics/**`, `production/session-state/**`, `.octogent/`,
`.claude/`. No cargo / trunk / CI command invoked by PROMPT 1285;
Cargo policy: N/A for this paperwork-only draft.

---

## 7. Non-Claims (preserved verbatim through Sprint 18 activation)

Sprint 18 activation and any prompt operating under the Sprint 18 plan
MUST preserve all of the following non-claims. PROMPT 1285 makes
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
- NO Sprint 10 through Sprint 17 row reopen.
- NO Sprint 17 close-out reopen, re-author, or silent overwrite.
- NO retroactive closure of any row not implemented on `origin/main`
  at the activation tip.

---

## 8. Branch / Push Policy

PROMPT 1285 commits the draft on branch
`work/sprint-18-plan-draft-1285` from base
`origin/main@d73e25e49519a214f8fb0fefa1e78351ccd74795`. Push target:
worker branch only; **never `main`**. Orchestrator integrates
separately. If branch push is blocked at the remote, the commit is
kept locally and the exact branch/commit is reported.
