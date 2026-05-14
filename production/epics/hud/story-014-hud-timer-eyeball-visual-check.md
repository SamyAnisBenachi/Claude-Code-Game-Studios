# Story 014: S11-HUD-TIMER-EYEBALL-VISUAL-001 -- HUD Timer Eyeball Visual Check

> **Epic**: HUD
> **Story ID**: S11-HUD-TIMER-EYEBALL-VISUAL-001
> **Status**: Draft -- Sprint 13 candidate (Should Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: HUD / Presentation (visual verification only)
> **Type**: Visual/Feel -- manual two-client smoke check + screenshot evidence
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; originally a
> Sprint 10 smoke retry-7 W2 carry into Sprint 11); NOT activated
> **Authored**: 2026-05-14 by PROMPT 819 (worktree
> `D:\_DEV\claude-code-game-studios-worktrees\sprint-13-missing-story-authoring`)
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run.

PROMPT 819 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The HUD timer is read-only over server-authoritative phase
state via `Res<CurrentClientPhase>` (ADR-021 binding); this story
verifies its visual rendering only and does not modify it.

---

## Source Finding

- Sprint 10 smoke retry-7 produced Warning W2: HUD timer eyeball
  visual check was deferred to a manual two-client run.
- Sprint 11 close-out carried W2 forward as Should Have row
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to Sprint
  13 planning (see
  `sprint_12_closeout.deferred_into_sprint_13_planning.should_have`
  in `production/sprint-status.yaml`).
- PROMPT 818 Sprint 13 draft plan pulls this row as a Should Have
  candidate at 0.25 estimated days.

---

## Problem Class / Prevention Target

**Defect class**: Visual regression risk in the HUD timer rendering
that automated tests cannot detect. The HUD timer is driven by
server-authoritative phase state (`S2CPhaseChanged` -> shared phase
sink -> `Res<CurrentClientPhase>` -> HUD timer system per ADR-021).
The numeric countdown values are server-authoritative; the **visual
rendering** of those values (text position, font size, color,
visibility transitions) is purely presentation and not asserted by
the existing logic tests.

**Prevention target**: A manual two-client smoke run captures
screenshots of the HUD timer in each of the three phases with
countdowns:

- `DraftInitial` (45 s countdown)
- `DraftShop` (30 s countdown)
- `Placement` (10-12 s countdown depending on tuning)

The evidence package records the captures alongside a written
eyeball verdict (PASS / FAIL / NEEDS-FOLLOW-ON). If the eyeball
verdict surfaces an actual visual regression, a follow-on story is
authored with the precise repro; **no production-source code change
lands under this story**.

---

## Context

### Existing surface

- **`client/src/ui/hud/`** (per ADR-021): HUD timer text entity is
  pre-pooled at session start and updated each frame from
  `Res<CurrentClientPhase>` and the phase-timer field of the latest
  `S2CPhaseChanged`.
- **`design/gdd/hud.md`** TR-HUD-003: phase label strings and timer
  countdown rendering.
- **`production/qa/smoke-sprint-10-*.md`** retry-7 W2: the original
  deferral.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hud.md` TR-HUD-003 (phase label + timer
  presentation).
- **ADR-021** (Presentation Layer Architecture): HUD timer renders
  from `Res<CurrentClientPhase>`; no direct `MessageReceiver<...>`
  call inside HUD timer system.
- **ADR-002** (Client-Server Authority): timer values are
  server-authoritative; client only renders.
- **TR registry**: no new TR (visual verification of existing TR).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust). No `.rs` edits expected under this
  story.
- **Mandatory skills (if a follow-on landed)**: `liv-bevy-018`,
  `liv-bevy-lightyear` (per `.claude/docs/technical-preferences.md`).
- **This story**: skills not invoked because no code change is
  authored.

### Control Manifest Rules

- Required: Manual run produces screenshot evidence per phase.
- Required: Eyeball verdict (PASS / FAIL / NEEDS-FOLLOW-ON) is
  written in the evidence document.
- Required: No production-source change lands.
- Required: ADR-002 + ADR-021 preserved (HUD is read-only).
- Forbidden: Introducing a client-side timer authority or local
  countdown loop.
- Forbidden: Modifying any `client/src/ui/hud/*.rs` file under this
  story (any visual fix is its own follow-on story).

---

## Story Classification

**Story type**: Visual/Feel -- manual two-client smoke check +
screenshot evidence.

This is **NOT** a:

- Logic story (no formula or state machine change).
- Integration story (no new code).
- Implementation story (any actual fix is a follow-on).

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Manual two-client run executed**: GIVEN a Sprint 13
  build at the activation HEAD, WHEN two real clients (browser/WASM
  or native) connect to a real server through the friend-game route,
  THEN the run reaches `DraftInitial`, `DraftShop`, and `Placement`
  phases at least once.

- [ ] **AC2 -- Screenshot per phase captured**: GIVEN the run reaches
  each target phase, WHEN the timer is mid-countdown, THEN a
  screenshot is captured showing the HUD timer for each of the three
  phases. Captures land under
  `production/qa/evidence/sprint-13-hud-timer-visual-check/` (NEW).

- [ ] **AC3 -- Eyeball verdict recorded**: GIVEN all three
  screenshots, WHEN the qa-tester or ui-programmer reviews them
  visually, THEN a verdict PASS / FAIL / NEEDS-FOLLOW-ON is written
  in the evidence document along with brief observations.

- [ ] **AC4 -- No production-source change lands**: GIVEN the story
  commit, WHEN diffed, THEN no file under `client/src/ui/hud/`,
  `client/src/`, `server/src/`, `shared/src/`, or `tests/` is
  modified. Only the new evidence document and (optionally) this
  story file's status are touched.

- [ ] **AC5 -- Follow-on story authored only on FAIL**: GIVEN the
  verdict is FAIL or NEEDS-FOLLOW-ON, WHEN the evidence document is
  finalised, THEN a follow-on story file is authored under the same
  HUD epic with the precise visual regression and a recommended
  remediation scope. If the verdict is PASS, no follow-on story is
  authored.

- [ ] **AC6 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, `production/qa/qa-plan-sprint-13.md` (when
  it exists), and PROMPT 761 gate-check artifact are diffed, THEN
  none of them are modified by this story.

- [ ] **AC7 -- No condition closure claimed**: GIVEN the evidence
  document, WHEN inspected, THEN it explicitly does **not** claim
  closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or
  `PAW-TD-*-a`. Standard-tier accessibility is not pursued by this
  story.

- [ ] **AC8 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-hud-timer-visual-check/README.md`
  (NEW). Records the build commit, the three screenshots, the eyeball
  verdict, no-claim restatement, and a cross-link to the Sprint 10
  smoke retry-7 W2 origin.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `production/qa/evidence/sprint-13-hud-timer-visual-check/README.md` | NEW evidence document. |
| `production/qa/evidence/sprint-13-hud-timer-visual-check/draft-initial-timer.png` (or .jpg) | NEW screenshot capture. |
| `production/qa/evidence/sprint-13-hud-timer-visual-check/draft-shop-timer.png` | NEW screenshot capture. |
| `production/qa/evidence/sprint-13-hud-timer-visual-check/placement-timer.png` | NEW screenshot capture. |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- None (no `.rs` edits expected under this story).
- If a follow-on fix story is authored as a result of an eyeball
  FAIL, **that** follow-on story invokes `liv-bevy-018` mandatorily.

---

## Evidence Path

`production/qa/evidence/sprint-13-hud-timer-visual-check/README.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- Build commit hash and branch.
- Three screenshots (one per target phase) at native resolution.
- Eyeball verdict (PASS / FAIL / NEEDS-FOLLOW-ON) with brief
  observations.
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to Sprint 10 smoke retry-7 W2 origin.
- Cross-link to `design/gdd/hud.md` TR-HUD-003.

---

## Regression Commands Expected

For the implementation prompt:

- `git diff <pre-impl-sha>..<impl-sha> -- 'client/src/**' 'server/src/**' 'shared/src/**' 'tests/**'`
  (verifies AC4: zero production-source change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

No `cargo test` is required by this story (no code change). If a
follow-on fix story is authored, that follow-on owns its own
regression command list.

---

## Out of Scope

- Any actual visual fix to the HUD timer.
- Standard-tier accessibility verification of the HUD timer
  (`QA-COND-0005` remains accepted-risk).
- Full HUD manual-QA review (this is a single-element eyeball check).
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Story landed under the existing HUD epic (no new epic created).
- No file collision with Sprint 13 Must Have rows.
- Sequences any time during Sprint 13 once the build at activation
  HEAD is available; depends on no other Sprint 13 row.
