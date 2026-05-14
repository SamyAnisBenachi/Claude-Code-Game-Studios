# Story 002: S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 -- R2 Placement Intermittent Runtime Crash Audit

> **Epic**: Server (Operational Hardening)
> **Story ID**: S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: Server -- `Phase::Placement` round-2 transition path
> **Type**: Audit / Diagnostic -- audit log expansion only; no fix lands
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; Sprint 11
> Wave 12 backlog 12:07 capture, not reproduced 13:28); NOT activated
> **Authored**: 2026-05-14 by PROMPT 819
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

**This story is audit-only. NO FIX LANDS under this story.** If a
repro is captured during Sprint 13, a follow-on story is authored
with the precise repro and recommended remediation scope.

---

## Source Finding

- Sprint 11 Wave 12 backlog: an intermittent server-side runtime
  crash was captured at 12:07 during a R2 `Phase::Placement`
  transition (round 2 placement), but the issue was **not
  reproduced** at the 13:28 retry.
- Sprint 11 close-out (`S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`
  row) deferred the audit as Nice to Have pending a repro.
- Sprint 12 close-out (PROMPT 817) carried the row forward to
  Sprint 13 planning.

---

## Problem Class / Prevention Target

**Defect class**: Intermittent server-side crash on the
`Phase::Placement` round-2 transition. No repro is currently
available; the captured stack / log evidence is insufficient to
diagnose the failure mode.

**Prevention target (audit-only)**: Add **diagnostic logs**
around the `Phase::Placement` round-2 transition (entry, exit,
intermediate substates) so that the next occurrence captures
enough evidence to enable diagnosis. Specifically:

- Tracing emission on entry to `Phase::Placement` (round, player
  count, alive units, placement intent buffer state).
- Tracing emission on exit (transition target, dwell time,
  outstanding intents).
- Tracing emission around each substep within the round-2
  transition that involves authoritative state mutation.

**No fix lands in this story.** If a repro is captured during
Sprint 13, a separate follow-on story is authored to scope the
actual remediation; otherwise the audit logs remain as
permanent diagnostic infrastructure.

---

## Context

### Existing surface

- **`server/src/game/` (canonical path verified by implementing
  worker)**: the `Phase::Placement` transition handler in the
  round state machine.
- **`design/gdd/round-state-machine.md`**: phase semantics for
  `Phase::Placement` and round-2 transition rules.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/round-state-machine.md` (phase semantics).
- **ADR-002** (Client-Server Authority): authoritative state
  unchanged.
- **ADR-009** (Round State Machine): phase semantics unchanged.
- **TR registry**: no new TR (audit-only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust). Server-side.
- **Mandatory skills**: `liv-bevy-018` (any `.rs` edit). May
  coordinate with the Sprint 13 Must Have row
  `S13-OBS-TRACING-TARGETS-001` (story 018) for consistent
  module-path-scoped tracing targets.

### Control Manifest Rules

- Required: All emissions are diagnostic (info / debug / trace);
  no behaviour change lands.
- Required: Emissions use module-path-scoped tracing targets
  consistent with Sprint 13 story 018 conventions
  (e.g., `target: "server::game"` or sub-target).
- Required: If a repro is captured, a follow-on story is authored
  with precise repro evidence.
- Forbidden: Changing the `Phase::Placement` transition logic.
- Forbidden: Adding panic-guards or fallbacks that mask the crash
  rather than diagnose it.
- Forbidden: Suppressing the crash output.

---

## Story Classification

**Story type**: Audit / Diagnostic -- audit log expansion only.

This is **NOT** a:

- Bug fix.
- Behaviour change.
- Test for fixed behaviour.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Audit-target sites named**: GIVEN the implementation
  prompt's first read pass, WHEN `server/src/game/` is grep'd for
  the `Phase::Placement` round-2 transition handler, THEN every
  authoritative state mutation site within that transition is
  enumerated with file:line evidence in the evidence document.

- [ ] **AC2 -- Audit logs added**: GIVEN the enumerated sites,
  WHEN the audit logs land, THEN each entry / exit / intermediate
  substep emits a tracing event at info or debug level with
  module-path-scoped target consistent with Sprint 13 story 018
  conventions (e.g., `target: "server::game::placement"`).

- [ ] **AC3 -- No behaviour change**: GIVEN the diff, WHEN
  reviewed, THEN no function body other than added tracing
  emissions is modified. Workspace tests continue to pass with
  the same assertions.

- [ ] **AC4 -- No fix for the underlying crash**: GIVEN the diff,
  WHEN reviewed, THEN no panic-guard, fallback path, or
  defensive `?`/`unwrap_or_else` is added to mask the crash. The
  crash, if it recurs, must produce the same observable failure
  mode plus the new audit evidence.

- [ ] **AC5 -- Workspace test pass**: GIVEN `cargo test
  --workspace --tests --no-fail-fast` at the implementation
  commit, WHEN compared to the post-Sprint-12 baseline, THEN no
  new `#[ignore]` markers are introduced; previously-passing
  tests continue to pass.

- [ ] **AC6 -- Repro-capture watch documented**: GIVEN the
  evidence document, WHEN inspected, THEN it explicitly states
  that during Sprint 13 the qa-tester / observer watches for
  any R2 Placement crash recurrence and, if captured, authors a
  follow-on story with the precise repro. **No fix is
  implemented under this story even if a repro is captured.**

- [ ] **AC7 -- No client-side or protocol change**: GIVEN the
  diff in `client/` and `shared/src/protocol.rs`, WHEN
  inspected, THEN no functional change lands.

- [ ] **AC8 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [ ] **AC9 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`
  (NEW). Records the enumerated audit-target sites, the diff
  summary, repro-watch protocol, no-fix restatement, no-claim
  restatement, cross-link to Sprint 11 Wave 12 12:07 capture,
  cross-link to Sprint 13 story 018 tracing-targets convention.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `server/src/game/` (canonical path verified by implementing worker) | Audit logs added around `Phase::Placement` round-2 transition. No behaviour change. |
| `production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md` | NEW evidence document per AC9. |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for any `.rs` edit.

---

## Evidence Path

`production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- Enumerated audit-target sites (file:line evidence).
- Diff summary for added audit logs.
- Repro-watch protocol (who watches, where the next capture lands,
  what triggers the follow-on story).
- Explicit restatement that no fix lands under this story.
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to Sprint 11 Wave 12 12:07 R2 Placement capture.
- Cross-link to Sprint 13 story 018 tracing-targets convention
  (when story 018 lands).

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'shared/**'`
  (verifies AC7: zero client / protocol change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- Any fix for the R2 Placement intermittent crash.
- Any change to the `Phase::Placement` transition logic.
- Any panic-guard / fallback / defensive coding that masks the
  crash.
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Sequences **after** Sprint 13 Must Have row
  `S13-OBS-TRACING-TARGETS-001` (story 018) so that the audit logs
  use the same module-path-scoped target convention. If story 018
  is not landed first, the worker uses a placeholder target
  (e.g., `target: "server::game"`) and updates it in a follow-on
  prompt once story 018 lands.
- Sequences **after** Sprint 13 Must Have row
  `S13-OBS-WALLCLOCK-TIMESTAMPS-001` (story 019(S13)) so the audit
  logs carry ISO-8601 wall-clock timestamps useful for
  multi-process correlation if the crash recurs.
- No file-scope collision with Sprint 13 Must Have stories 007,
  008, 016, 017.
