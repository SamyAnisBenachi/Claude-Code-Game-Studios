# Story 001: S11-SERVER-POOL-INIT-LOG-GUARD-001 -- Server `init_pool` Log Emits Before Guard

> **Epic**: Server (Operational Hardening)
> **Story ID**: S11-SERVER-POOL-INIT-LOG-GUARD-001
> **Status**: Draft -- Sprint 13 candidate (Should Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: Server -- card-pool init path
> **Type**: Logic -- log-gating fix + integration / smoke evidence
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; parallel to
> Sprint 11 Wave 12 backlog W5 `ee27fb6` `acquisition_tick` fix); NOT
> activated
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

**No client-side authority is introduced or proposed by this story.**
The fix is server-side only: gate the `init_pool` info-level log on
the existing initialization guard. ADR-002 binding.

---

## Source Finding

- Sprint 11 close-out (`S11-SERVER-POOL-INIT-LOG-GUARD-001` row)
  flagged that the server's `init_pool` info-level log emits before
  the initialization guard fires, producing redundant log volume
  on every cold-path traversal.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to Sprint
  13 planning.
- Pattern: parallel to Sprint 11 Wave 12 W5 fix `ee27fb6`
  (`acquisition_tick` log gated on its guard). Apply the same
  pattern.

---

## Problem Class / Prevention Target

**Defect class**: A server `info!`-level log is emitted **before**
the function's idempotency guard fires. On a re-entrant code path
(e.g., session restart, fixture re-run, double-init under tests),
the log emits one line per re-entry even though the guard
short-circuits the work. Symptoms: log noise budget for the cold
path is blown; diagnostic captures contain spurious "init_pool"
lines that do not correspond to actual init events.

**Prevention target**: Move the `info!` emission **after** the
guard check, so that the log fires only on the frame the guard
permits the work. The fix is mechanical and small (one log
relocation in `server/src/game/` or canonical equivalent).
Verification target: smoke / log evidence captures **<50 emitted
`init_pool` lines per session** on the cold path.

---

## Context

### Existing surface

- **`server/src/game/` (canonical path verified by implementing
  worker)**: the `init_pool` function and its existing
  initialization guard.
- **Pattern reference**: Sprint 11 Wave 12 W5 fix `ee27fb6` for
  `acquisition_tick`, which gated the log on the existing guard.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/card-data-pool.md` (pool init lives in the
  card-data-pool system on the server).
- **ADR-002** (Client-Server Authority): no client-side authority
  added.
- **ADR-003** (Cargo Workspace Structure): `shared/` boundary
  preserved.
- **TR registry**: no new TR (log-gating fix only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust). Server-side.
- **Mandatory skills**: `liv-bevy-018` (any `.rs` edit).

### Control Manifest Rules

- Required: `info!` emission lands **after** the existing init
  guard.
- Required: Pattern matches Sprint 11 W5 `ee27fb6`
  `acquisition_tick` fix.
- Required: Smoke / log evidence captures <50 emitted lines per
  session on the cold path.
- Forbidden: Modifying the initialization guard itself.
- Forbidden: Suppressing the log entirely (it remains
  diagnostically useful; only its emission frequency is fixed).
- Forbidden: Any change outside `server/src/game/`.

---

## Story Classification

**Story type**: Logic -- log-gating fix.

This is **NOT** a:

- Client-side change.
- Protocol change.
- Behaviour change.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Source located**: GIVEN the implementation prompt's
  first read pass, WHEN `server/src/game/` is grep'd for
  `init_pool` log emission, THEN the exact `info!` (or `tracing::info!`)
  call site and the existing guard are named with file:line evidence
  in the evidence document.

- [ ] **AC2 -- Log relocated**: GIVEN the located source, WHEN the
  fix lands, THEN the `info!` emission is moved to **after** the
  existing initialization guard so it fires only when the guard
  permits work.

- [ ] **AC3 -- Pattern matches `ee27fb6`**: GIVEN the diff, WHEN
  compared to Sprint 11 Wave 12 W5 fix `ee27fb6` for
  `acquisition_tick`, THEN the relocation pattern matches (log
  emission moved post-guard; guard logic unchanged).

- [ ] **AC4 -- Smoke / log evidence**: GIVEN a Sprint 13 smoke run
  (or a dedicated server log capture session), WHEN the cold path
  is exercised, THEN the captured logs show **<50 `init_pool`
  emitted lines per session**.

- [ ] **AC5 -- No client-side change**: GIVEN the diff in `client/`,
  WHEN inspected, THEN no functional change lands.

- [ ] **AC6 -- No protocol change**: GIVEN the diff in
  `shared/src/protocol.rs`, WHEN inspected, THEN no functional
  change lands.

- [ ] **AC7 -- Workspace test pass**: GIVEN `cargo test --workspace
  --tests --no-fail-fast` at the implementation commit, WHEN
  compared to the post-Sprint-12 baseline, THEN no new `#[ignore]`
  markers are introduced; previously-passing tests continue to
  pass.

- [ ] **AC8 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them are
  modified by this story.

- [ ] **AC9 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`
  (NEW). Records the file:line evidence, the diff summary, the
  smoke/log capture showing <50 emitted lines, cross-link to
  Sprint 11 W5 `ee27fb6` pattern, no-claim restatement.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `server/src/game/` (canonical path verified by implementing worker) | One log emission relocated to post-guard. |
| `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md` | NEW evidence document per AC9. |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for the `.rs` edit.

---

## Evidence Path

`production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- File:line evidence of the pre-fix `info!` emission and the
  existing guard.
- Diff summary for the log relocation.
- Smoke / log capture confirming <50 emitted `init_pool` lines per
  session on the cold path.
- Cross-link to Sprint 11 Wave 12 W5 fix `ee27fb6` for pattern
  reference.
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to Sprint 12 close-out deferral row.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'shared/**'`
  (verifies AC5 + AC6: zero client / protocol change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- Modifying the initialization guard logic.
- Suppressing or removing the `info!` log entirely.
- Any change outside `server/src/game/`.
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Touches `server/src/game/`. No file-scope collision with the 8
  PROMPT 804 Sprint 13 candidate stories (007, 008, 016, 017, 018,
  019(S13), 020, 021), which target `shared/src/protocol.rs`,
  `client/src/`, `server/src/main.rs`, and `tests/`. Sequences
  independently.
- Wider backlog row `S13-S2C-SUCCESS-LOG-001` (DC-3) is the same
  log-emission class but a different surface; not folded.
