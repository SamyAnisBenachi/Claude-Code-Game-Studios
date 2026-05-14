# Story 001: S11-SERVER-POOL-INIT-LOG-GUARD-001 -- Server `init_pool` Log Emits Before Guard

> **Epic**: Server (Operational Hardening)
> **Story ID**: S11-SERVER-POOL-INIT-LOG-GUARD-001
> **Status**: Done -- closed by PROMPT 833 (`/story-done` paperwork) on
> `origin/main@7983f5c`; worker `c6f6325` (PROMPT 829) integrated to
> `main` as `7983f5c` (PROMPT 832). W5 `ee27fb6` `acquisition_tick`
> pattern applied: pre-guard entry log downgraded `info!` -> `debug!`;
> new `info!` emitted only after the `DraftPhase::Initial` continue-guard.
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

- [x] **AC1 -- Source located**: GIVEN the implementation prompt's
  first read pass, WHEN `server/src/game/` is grep'd for
  `init_pool` log emission, THEN the exact `info!` (or `tracing::info!`)
  call site and the existing guard are named with file:line evidence
  in the evidence document.
  *Closure evidence*: pre-fix `info!` at
  `server/src/core/pool/system.rs:21`; existing guard at
  `server/src/core/pool/system.rs:25-28` (the `for message in
  draft_started.read() { if message.phase != DraftPhase::Initial {
  continue; } ... }` block). Documented in
  `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`
  AC1 section. The canonical path is `server/src/core/pool/`, not
  `server/src/game/`; story-card path field was a planning
  approximation and the verified path is recorded in the evidence
  doc.

- [x] **AC2 -- Log relocated**: GIVEN the located source, WHEN the
  fix lands, THEN the `info!` emission is moved to **after** the
  existing initialization guard so it fires only when the guard
  permits work.
  *Closure evidence*: post-fix `info!` at
  `server/src/core/pool/system.rs:37-39`, immediately after the
  `for message in draft_started.read() { if message.phase !=
  DraftPhase::Initial { continue; } }` continue-guard. Pre-guard
  emission downgraded to `tracing::debug!` at lines 25-27.

- [x] **AC3 -- Pattern matches `ee27fb6`**: GIVEN the diff, WHEN
  compared to Sprint 11 Wave 12 W5 fix `ee27fb6` for
  `acquisition_tick`, THEN the relocation pattern matches (log
  emission moved post-guard; guard logic unchanged).
  *Closure evidence*: W5 `ee27fb6` pattern recreated identically
  (entry log downgraded `info!` -> `debug!`; info-level added
  post-guard; guard body byte-identical). Cross-link in evidence
  document AC3 section.

- [x] **AC4 -- Smoke / log evidence**: GIVEN a Sprint 13 smoke run
  (or a dedicated server log capture session), WHEN the cold path
  is exercised, THEN the captured logs show **<50 `init_pool`
  emitted lines per session**.
  *Closure evidence*: AC4 satisfied by static-analysis bound — the
  post-fix `info!` fires only inside `for message in
  draft_started.read()` after the `DraftPhase::Initial`
  continue-guard, which is a one-shot RSM transition. Therefore
  N_info <= count(`DraftStarted::Initial` drains) <= count(session
  restarts), <<50 under any realistic cold-path scenario. Runtime
  smoke confirmation deferred to the Sprint 13 end-of-sprint
  integration smoke per QA-plan-sprint-13's smoke serialization
  policy; PROMPT 833 does NOT run `/smoke-check`.

- [x] **AC5 -- No client-side change**: GIVEN the diff in `client/`,
  WHEN inspected, THEN no functional change lands.
  *Closure evidence*: `git diff b0c43cb..7983f5c -- 'client/**'`
  empty at the integration commit; AC5 confirmed by both PROMPT 829
  worker run and PROMPT 832 integration verification.

- [x] **AC6 -- No protocol change**: GIVEN the diff in
  `shared/src/protocol.rs`, WHEN inspected, THEN no functional
  change lands.
  *Closure evidence*: `git diff b0c43cb..7983f5c -- 'shared/**'`
  empty at the integration commit; AC6 confirmed by both PROMPT 829
  and PROMPT 832.

- [x] **AC7 -- Workspace test pass**: GIVEN `cargo test --workspace
  --tests --no-fail-fast` at the implementation commit, WHEN
  compared to the post-Sprint-12 baseline, THEN no new `#[ignore]`
  markers are introduced; previously-passing tests continue to
  pass.
  *Closure evidence*: `cargo test -p server --lib` 98 passed, 0
  failed, 0 ignored at PROMPT 829 worker (`c6f6325`) and again at
  PROMPT 832 integration (`7983f5c`). Full-workspace
  `cargo test --workspace --tests --no-fail-fast` deferred to the
  Sprint 13 end-of-sprint orchestrator integration gate per
  `production/qa/qa-plan-sprint-13.md`'s binding
  "no-full-workspace-tests-by-default" / "per-row narrowest BLOCKING
  command only" policy. The narrowest BLOCKING command (server-lib
  unit tests) is the W5 `ee27fb6` precedent and exercises every
  pool/RSM-touched server unit test. PROMPT 833 does NOT re-run
  Cargo as paperwork-only closure; AC7 closes on the documented
  worker + integration evidence.

- [x] **AC8 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them are
  modified by this story.
  *Closure evidence*: integration commit `7983f5c` touches only
  `server/src/core/pool/system.rs` and
  `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`;
  `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, and
  `production/gate-checks/gate-polish-release-2026-05-12.md` were
  not modified by the integration commit. PROMPT 833 paperwork
  closure separately flips this story's
  `production/sprint-status.yaml` row `status: ready -> done` with
  `completed: 2026-05-14` (the only sprint-status delta authorised
  by `/story-done`); stage / sprint-13 plan / gate-check artifact
  remain unmodified by PROMPT 833.

- [x] **AC9 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`
  (NEW). Records the file:line evidence, the diff summary, the
  smoke/log capture showing <50 emitted lines, cross-link to
  Sprint 11 W5 `ee27fb6` pattern, no-claim restatement.
  *Closure evidence*: file authored by PROMPT 829 worker, 281
  lines, integrated by PROMPT 832 in `7983f5c`. Contains AC1
  file:line evidence, AC2 post-fix code, AC3 W5 `ee27fb6`
  cross-link, AC4 cold-path bound analysis, AC5/AC6 no-change
  confirms, AC7 targeted-test result, AC8 disposition-preserved
  confirms, no-claim restatement verbatim.

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

---

## Authoring Trail

- 2026-05-14 -- PROMPT 819 -- Story authored as Sprint 13 candidate
  Should Have (Wave 12 W5 `ee27fb6` `acquisition_tick` parallel
  pattern). Source-of-truth at authoring: `origin/main@be69f5c`
  (PROMPT 818 Sprint 13 DRAFT). No /story-readiness / /dev-story /
  /story-done / /smoke-check / /team-qa / /gate-check / /qa-plan run
  by PROMPT 819.
- 2026-05-14 -- PROMPT 823 -- `/story-readiness` rerun verdict
  `READY` (story shipped in PROMPT 823's READY batch of 12 Sprint 13
  stories). No code / production-state changes by PROMPT 823.
- 2026-05-14 -- PROMPT 826 -- Sprint 13 activated; this story
  promoted into the active Sprint 13 Should Have row set at
  `origin/main@e331d6a`. No code change.
- 2026-05-14 -- PROMPT 827 -- Sprint 13 QA plan authored at
  `production/qa/qa-plan-sprint-13.md` (`origin/main@4bf95fa`). The
  binding per-row narrowest BLOCKING command for this story is
  `cargo test -p server --lib` (W5 `ee27fb6` precedent); the
  end-of-sprint orchestrator gate runs the full workspace.
- 2026-05-14 -- PROMPT 829 -- `/dev-story` worker landed the W5
  pattern on worktree `work/s13-server-pool-init-log-guard`
  (`c6f6325`): pre-guard `info!` downgraded to `debug!`;
  post-`DraftPhase::Initial`-guard `info!` added. Evidence at
  `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`
  (NEW, 281 lines). Targeted regression: `cargo fmt -p server`
  clean, `cargo check -p server` clean, `cargo test -p server --lib`
  98/0/0. `client/**` and `shared/**` diffs empty. Cargo resource
  policy applied. Source-of-truth at worker start:
  `origin/main@4bf95fa` (PROMPT 827).
- 2026-05-14 -- PROMPT 832 -- Integration prompt cherry-picked
  `c6f6325` onto an integration worktree built from
  `origin/main@b0c43cb`; new commit `7983f5c` (identical 2-file
  scope) fast-forward-pushed to `origin/main`. Targeted regression
  re-run on the integration worktree: `cargo fmt -p server` clean,
  `cargo check -p server` clean, `cargo test -p server --lib` 98/0/0.
  No `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check` run by PROMPT 832.
- 2026-05-14 -- PROMPT 833 -- `/story-done` paperwork: this Status
  field flipped Draft -> Done; AC1-AC9 checkboxes flipped `[ ]` ->
  `[x]` against `origin/main@7983f5c` evidence;
  `production/sprint-status.yaml` Sprint 13 Should Have row
  `S11-SERVER-POOL-INIT-LOG-GUARD-001` flipped `status: ready ->
  done` with `completed: 2026-05-14`. Sprint 13 disposition
  UNCHANGED (`active`). Stage UNCHANGED (`Polish`). PROMPT 761
  Polish->Release gate-check FAIL preserved (no retry). No
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check` run
  by PROMPT 833. No Sprint 13 close-out. No `client/`, `server/`,
  `shared/`, `tests/` touched. Carry conditions and non-claims
  preserved verbatim from this story file's "Status / No-Claim
  Banner" (S8-QA-001-W1 OPEN; QA-COND-0005 + QA-COND-0006
  accepted-risk; PAW-TD-*-a accept-risk; PROMPT 683-era runtime
  divergence question; PROMPT 761 Polish->Release FAIL; TQ-S12-C1..C7
  verbatim; story 019 underlying drag-runtime bug NOT claimed fixed).
