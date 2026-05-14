# Story 002: S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 -- R2 Placement Intermittent Runtime Crash Audit

> **Epic**: Server (Operational Hardening)
> **Story ID**: S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001
> **Status**: Done -- closed by PROMPT 885 (`/story-done`) on
> `origin/main@dd9630bc572a466fd6b88be4be0d8f894cd34252` (PROMPT 877
> integration merge of PROMPT 874 worker commit
> `dc140896730ef6e3464ca78ed987c21c80ad0ffb`). Verdict **PASS**.
> AC1-AC9 all PASS against integrated evidence on `origin/main@dd9630b`.
> **Layer**: Server -- `Phase::Placement` round-2 transition path
> **Type**: Audit / Diagnostic -- audit log expansion only; no fix lands
> **Sprint**: Sprint 13 (Nice to Have); activated 2026-05-14 (PROMPT 826);
> closed 2026-05-14 (PROMPT 885 `/story-done`)
> **Authored**: 2026-05-14 by PROMPT 819
> **Implemented**: 2026-05-14 by PROMPT 874 (worker; `dc14089` on
> `work/s13-r2-placement-crash-audit` from base `origin/main@3cf5e41`)
> **Integrated**: 2026-05-14 by PROMPT 877 (`--no-ff` merge commit
> `dd9630b` on `origin/main` via merge of worker tip `dc14089` into prior
> `origin/main@51e6228`; clean fast-forward push `51e6228..dd9630b`)
> **Closed**: 2026-05-14 by PROMPT 885 (`/story-done` paperwork)
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)
> **Closure source-of-truth**: `origin/main@dd9630b` (PROMPT 877 integration
> merge; PROMPT 874 worker reachable as merge's second-parent)

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

- [x] **AC1 -- Audit-target sites named** — PASS (closure evidence
  PROMPT 885): evidence doc §AC1 (`production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`
  lines 57-114) enumerates six authoritative state mutation sites for the
  R2 Placement transition with file:line evidence at worker base
  `origin/main@3cf5e41`: `transitions.rs:90-102` (`rsm_input_reader`),
  `transitions.rs:335-365` (`advance_phase` entry), `transitions.rs:421-444`
  (DraftInitial→Placement R1 entry arm), `transitions.rs:471-497`
  (DraftShop→Placement R2 canonical entry arm),
  `transitions.rs:498-522` (Placement→Resolution exit arm), plus
  `placement.rs:278-290` (`placement_buffer_open`), `placement.rs:385-431`
  (`handle_placement_submission`), `placement.rs:505-588`
  (`close_placement_phase` — R2 EXIT handler, mutation-densest site
  with three pre-mutation guards and one mid-mutation guard), and
  `placement.rs:895-957` (`spawn_committed_placement` called from
  `close_placement_phase`).

- [x] **AC2 -- Audit logs added** — PASS (closure evidence PROMPT 885):
  `cd D:/_DEV/wt/ccgs-prompt-885-storydone && rg -c 'server::game::placement' server/src` at
  `origin/main@dd9630b` reports 13 occurrences in
  `server/src/core/rsm/transitions.rs` + 12 occurrences in
  `server/src/feature/board/placement.rs` = 25 total emissions, every
  one tagged `target: "server::game::placement"` (Sprint 13 story 018
  module-path-scoped convention; landed via PROMPT 847 `9e32fbe` +
  PROMPT 850 `/story-done` `c1b7753`). Identical count of 25 for
  parenthetical `(audit: R2 placement transition audit)`. All emit at
  `tracing::info!` / `tracing::debug!` / `tracing::warn!` per evidence
  doc §AC2 lines 117-187 enumeration (entries 1-19 covering
  rsm_input_reader per-submission/post-insert/quorum + advance_phase
  entry + DraftInitial/DraftShop/Placement arm entries+substeps +
  placement_buffer_open entry+post-clear + close_placement_phase
  entry+4-early-return-warns+committed-sequence+mana-deducted+S2C-reveal+spawn-loop+exit).

- [x] **AC3 -- No behaviour change** — PASS (closure evidence
  PROMPT 885): `git diff dd9630b^1..dd9630b --stat` reports exactly
  3 files / +507 / -1: `transitions.rs +85 / -0` (pure tracing
  additions); `placement.rs +73 / -1` (12 tracing additions + 1
  supporting `let reveal_placements_len = reveal.placements.len();`
  binding inside `close_placement_phase` introduced because `reveal`
  is moved into `sender.send`; the single `-1` line is the inline
  `reveal.placements.len()` expression in the existing
  `tracing::error!` call replaced by the new local — log content
  byte-identical per evidence doc §AC3 lines 191-203); evidence doc
  +350 / -0 (NEW). PROMPT 874 worker `cargo check -p server` PASS
  (Finished `dev` [optimized] in 7.26s, no warnings); PROMPT 877
  integration `cargo check -p server` PASS (15.71s, no warnings) +
  targeted placement/RSM regression `placement_timer_multiplier_test`
  6/0/0 + `rsm_placement_timer_multiplier_test` 2/0/0 +
  `rsm_transitions_test` 14/0/0 + `rsm_timers_test` 10/0/0 = 32/0/0.

- [x] **AC4 -- No fix for the underlying crash** — PASS (closure
  evidence PROMPT 885): no panic-guard, fallback path, defensive
  `?` / `unwrap_or_else`, or suppression added per evidence doc §AC4
  lines 205-214. The four pre-existing `close_placement_phase`
  early-return short-circuits (catalog-missing / server-or-sender-missing
  / economies-missing / mana-deduction-failed) get `tracing::warn!`
  emissions **immediately BEFORE** the existing `return;` statements;
  the `return;` statements themselves are byte-identical to the
  pre-edit state. If the underlying crash recurs, the same observable
  failure mode (panic / process abort / silent early return) will
  fire — now with audit-level tracing breadcrumb evidence captured up
  to the failure point.

- [x] **AC5 -- Workspace test pass** —
  PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK (closure evidence
  PROMPT 885): per PROMPT 877 integrator scope ("Do not run full
  workspace tests by default... run targeted placement/audit tests
  from the worker report") + Sprint 13 QA-plan binding
  no-full-workspace-tests-by-default policy
  (`production/qa/qa-plan-sprint-13.md` lines 599-602), the four
  targeted placement/RSM test files most directly exercising the
  audit-instrumented code passed on integration tip `dd9630b`:
  `cargo test -p server --test placement_timer_multiplier_test` 6/0/0
  + `... rsm_placement_timer_multiplier_test` 2/0/0
  + `... rsm_transitions_test` 14/0/0
  + `... rsm_timers_test` 10/0/0 = **32/0/0** aggregate
  (PROMPT 877 integration report Exact checks #8-#11). No new
  `#[ignore]` markers introduced (worker did not edit `tests/`; diff
  scope is `transitions.rs` + `placement.rs` + evidence doc only).
  Full-workspace `cargo test --workspace --tests --no-fail-fast`
  result claim deferred to Sprint 13 end-of-sprint integration smoke
  per binding policy.

- [x] **AC6 -- Repro-capture watch documented** — PASS (closure
  evidence PROMPT 885): evidence doc §AC6 lines 228-250 documents the
  qa-tester / observer protocol verbatim: (1) preserve operator log
  output verbatim with at least last 200 lines preceding the failure,
  (2) note the `round = ...` field from audit log emissions, (3)
  author a follow-on story under `production/epics/server/` with
  precise repro evidence + recommended remediation scope, (4) do NOT
  implement a fix under this story even if a repro is captured. The
  no-fix restatement is preserved verbatim at evidence doc lines 43-46
  and 246-250.

- [x] **AC7 -- No client-side or protocol change** — PASS (closure
  evidence PROMPT 885): `git diff dd9630b^1..dd9630b --stat -- 'client/' 'shared/'`
  is empty at integration commit on `origin/main`. The PROMPT 874
  worker diff against base `origin/main@3cf5e41` was also empty under
  `'client/**'` and `'shared/**'` per worker report (`git diff --stat
  origin/main -- 'client/**' 'shared/**'` empty pre-commit) and PROMPT
  877 integration check #17 also reports empty. No `shared/src/protocol.rs`
  modification; `close_placement_phase` re-uses the existing
  `ServerMultiMessageSender::send` signature without modifying message
  types — `liv-bevy-lightyear` skill not activated by worker because
  protocol/message-channel surface was untouched.

- [x] **AC8 -- Sprint 13 disposition preserved** — PASS (closure
  evidence PROMPT 885): `git diff dd9630b^1..dd9630b --stat -- 'production/sprint-status.yaml' 'production/sprints/sprint-13.md' 'production/stage.txt' 'production/gate-checks/'`
  is empty at integration commit on `origin/main` (verified pre-paperwork).
  Worker diff against base also empty per worker report. The PROMPT
  885 row-level `status: ready -> done` flip plus `completed: 2026-05-14`
  + worker/integration/story-done hashes + new notes lines in
  `production/sprint-status.yaml` is the permitted
  disposition-preserving `/story-done` paperwork edit (top-level
  `sprint:` / `status:` / `stage:` unchanged). `production/sprints/sprint-13.md`
  + `production/stage.txt` + PROMPT 761 gate-check artifact
  `production/gate-checks/gate-polish-release-2026-05-12.md` NOT
  modified by PROMPT 885.

- [x] **AC9 -- Evidence document slot reserved** — PASS (closure
  evidence PROMPT 885): `production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`
  exists at the canonical path on `origin/main@dd9630b` (NEW, 350
  lines, created by PROMPT 874 worker; reachable as
  `dd9630b^2:production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`).
  Contents: §AC1 enumerated sites with file:line evidence; §AC2 19-entry
  emission catalogue under `server::game::placement` target; §AC3
  no-behaviour-change diff summary; §AC4 no-fix restatement; §AC5
  worker-scope cargo check + integrator-deferred full-workspace
  expectation; §AC6 4-step repro-watch protocol with no-fix
  restatement; §AC7 zero `'client/**' 'shared/**'` diff; §AC8 zero
  forbidden-path diff; §AC9 self-reference; Diff summary; Targeted
  checks executed; Cross-links to Sprint 11 Wave 12 12:07 capture,
  story 018 tracing-targets convention, story 019 wall-clock
  timestamps; Worker prompt log. Not modified by PROMPT 885.

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

At closure time (PROMPT 885), both sequencing dependencies were
satisfied on `origin/main` before PROMPT 874 worker began:
`S13-OBS-TRACING-TARGETS-001` landed via PROMPT 847 commit `9e32fbe`
+ PROMPT 850 `/story-done` `c1b7753` (target string
`server::game::placement` used by the audit emissions is a
module-path-scoped sub-target consistent with story 018);
`S13-OBS-WALLCLOCK-TIMESTAMPS-001` landed via PROMPT 837 commit
`a8ec25f` + PROMPT 843 `/story-done` `534d9df` (audit emissions
inherit ISO-8601 UTC timestamps from the global tracing subscriber).

---

## Authoring / Implementation / Closure Trail

### PROMPT 819 (2026-05-14 — authoring)

- Story file authored at this canonical path by PROMPT 819
  (`/sprint-plan sprint-13` DRAFT batch alongside 10 sibling Sprint 13
  candidate story files).
- Authoring source-of-truth: `origin/main@be69f5c`.
- No `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, code under `client/` / `server/` /
  `shared/` / `tests/`, QA-plan / smoke / Team-QA / gate-check /
  release-check artifact, or `production/session-state/*` file
  modified by PROMPT 819 (per Status / No-Claim Banner above).
- No `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan` run.
- No Polish→Release gate-check retry attempted.

### PROMPT 823 (2026-05-14 — `/story-readiness` batch)

- This story batch-verified READY against Sprint 13 activation HEAD.
- Verdict recorded in `production/sprints/sprint-13.md` Required
  Sprint 13 Story Docs table + `production/qa/qa-plan-sprint-13.md`
  story-readiness table.
- No code or sprint-status modification.

### PROMPT 826 (2026-05-14 — Sprint 13 activation)

- Sprint 13 flipped DRAFT → ACTIVE; this row promoted into
  `production/sprint-status.yaml` Sprint 13 Nice to Have block with
  `status: ready` per `sprint_13_activation.story_files_referenced`
  block.
- Stage UNCHANGED (`Polish`); PROMPT 761 Polish→Release gate-check
  FAIL preserved.

### PROMPT 827 (2026-05-14 — Sprint 13 QA plan)

- `production/qa/qa-plan-sprint-13.md` authored; this row mapped to
  Logic (audit-only) story type with server gameplay programmer as
  reviewer; doc-only Nice to Have rows classified at
  `qa-plan-sprint-13.md` lines 483-495; smoke gate notes at line 602
  reserve "R2 Placement audit log emits enriched diagnostics around
  `Phase::Placement` round-2 transition" for the Sprint 13
  end-of-sprint smoke check.
- No code change; no sprint-status modification.

### PROMPT 874 (2026-05-14 — `/dev-story` worker; verdict PASS)

- Worker branch: `work/s13-r2-placement-crash-audit` (new; pushed).
- Worker base: `origin/main@3cf5e41` (PROMPT 870 integration tip at
  worker start).
- Worker commit: `dc140896730ef6e3464ca78ed987c21c80ad0ffb`.
- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\s13-r2-placement-crash-audit`.
- Skills activated: `liv-bevy-018` (any `.rs` edit per technical-preferences
  routing). `liv-bevy-lightyear` **NOT** activated (`close_placement_phase`
  re-uses `ServerMultiMessageSender::send` without modifying its
  signature; no Lightyear protocol surface touched).
- Files modified: `server/src/core/rsm/transitions.rs` (+85 / -0
  audit emissions only across `rsm_input_reader` + `advance_phase`
  entry / DraftInitial→Placement / DraftShop→Placement / Placement→Resolution
  arms), `server/src/feature/board/placement.rs` (+73 / -1 audit
  emissions across `placement_buffer_open` + `close_placement_phase`
  entry / 4 early-return warns / committed-sequence / mana-deducted
  / S2C-reveal / spawn-loop / exit — one supporting
  `let reveal_placements_len` binding to enable post-S2C audit
  substep without rebinding the moved variable),
  `production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`
  (NEW; canonical AC9 path; 350 lines).
- Cargo policy applied: `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
  + `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
  `CARGO_INCREMENTAL=0` + `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`.
- Targeted checks: `cargo fmt -p server -- --check` PASS;
  `cargo check -p server` PASS (7.26s); `git diff --check
  origin/main...HEAD` PASS; `git diff --stat origin/main --
  'client/**' 'shared/**'` empty (AC7); `git diff --stat origin/main
  -- 'production/sprint-status.yaml' 'production/sprints/sprint-13.md'
  'production/stage.txt' 'production/gate-checks/**'` empty (AC8).
- No `cargo test --workspace` (worker prompt explicitly forbade
  full-workspace tests).
- No `/story-readiness`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, `/qa-plan` run by PROMPT 874.
- Report: `reports/PROMPT-874-S13-R2-Placement-Crash-Audit.md`.

### PROMPT 877 (2026-05-14 — integration; verdict PASS)

- Integration branch: `integrate/s13-r2-placement-crash-audit-877`.
- Integration worktree: `D:\_DEV\claude-code-game-studios-worktrees\integration-s13-r2-placement-crash-audit-877`.
- Integration commit: `dd9630bc572a466fd6b88be4be0d8f894cd34252`
  (clean `--no-ff` merge of worker tip `dc14089` into prior
  `origin/main@51e6228`; strategy `ort`; 3 files / +507 / -1 —
  byte-identical to worker stat).
- First integration attempt `066ccf0` discarded after origin/main
  advanced mid-run from `7403e8f` to `51e6228` (PROMPT 871
  `/story-done` paperwork). Integration branch was
  `git reset --hard origin/main`-ed and worker tip re-merged; identical
  file content, only merge SHA changed.
- Targeted regression on integration tip: `cargo fmt -p server --
  --check` PASS; `cargo check -p server` PASS (15.71s); placement /
  RSM tests `placement_timer_multiplier_test` 6/0/0 +
  `rsm_placement_timer_multiplier_test` 2/0/0 +
  `rsm_transitions_test` 14/0/0 + `rsm_timers_test` 10/0/0 = 32/0/0.
- `git diff --check origin/main...HEAD` PASS;
  `git diff --cached --check` PASS; `git diff --stat origin/main...HEAD --
  'production/sprint-status.yaml' 'production/session-state/' 'production/stage.txt'
  'production/sprints/' 'production/gate-checks/'` empty (no
  forbidden-file touches); `git diff --stat origin/main...HEAD --
  'client/**' 'shared/**'` empty (worker AC7 preserved at
  integration).
- Fast-forward push `51e6228..dd9630b` to `origin/main`; no force,
  no conflict.
- Cargo policy applied identically to worker (same env vars).
- Skills used: `liv-bevy-018` review context; `liv-bevy-lightyear`
  not activated (Lightyear protocol surface untouched).
- No `/story-done` run by PROMPT 877. No `production/sprint-status.yaml`
  / `production/session-state/*` / `production/stage.txt` /
  smoke / team-qa / gate-check / release-check artifact modified.
- Report: `reports/PROMPT-877-S13-R2-Placement-Crash-Audit-Integration.md`.

### PROMPT 885 (2026-05-14 — `/story-done` closure; verdict PASS)

- Source-of-truth at closure: `origin/main@dd9630bc572a466fd6b88be4be0d8f894cd34252`
  (PROMPT 877 integration merge of PROMPT 874 worker commit `dc14089`
  into prior `origin/main@51e6228`; PROMPT 874 worker reachable as
  merge's second-parent).
- Worktree: `D:/_DEV/wt/ccgs-prompt-885-storydone` (fresh detached
  worktree from `origin/main` because root checkout had unrelated dirt
  — `M .claude/settings.json` + `M production/session-state/codex-orchestrator-state.md`
  + untracked files; root-checkout dirt NOT touched by PROMPT 885).
  Pattern matches PROMPT 884 precedent.
- HEAD at closure: `origin/main@ae0165a` (PROMPT 884 tip) →
  PROMPT 885 commit applied on top via this paperwork run; verified
  PROMPT 877 integration commit `dd9630b` is reachable on
  `origin/main` (`git merge-base --is-ancestor dd9630b origin/main`
  TRUE; `git log --oneline --grep="PROMPT 877"` returns
  `dd9630b integrate(s13): merge work/s13-r2-placement-crash-audit
  (server story 002 / PROMPT 874) (PROMPT 877)`).
- AC1-AC9 verification: all PASS against integrated evidence on
  `origin/main@dd9630b` (read-only grep + diff verification — see
  per-AC annotations above). AC5 closure verdict
  PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK per Sprint 13 QA-plan
  binding no-full-workspace-tests-by-default policy + PROMPT 877
  integrator scope (32/0/0 targeted placement/RSM tests at integration
  tip).
- Files changed by PROMPT 885 (paperwork only): this story file
  (Status header `Draft -> Done` + AC1-AC9 checkboxes `[ ] -> [x]`
  with per-AC closure-evidence annotations + Closure Trail section
  appended below Authoring Trail), `production/sprint-status.yaml`
  (top-level `updated:` annotation refreshed + row `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`
  flipped `status: ready -> done` with `completed: 2026-05-14`,
  `worker_prompt: 874`, `worker_branch: work/s13-r2-placement-crash-audit`,
  `worker_commit: dc140896730ef6e3464ca78ed987c21c80ad0ffb`,
  `integration_prompt: 877`, `integration_branch: integrate/s13-r2-placement-crash-audit-877`,
  `integration_commit: dd9630bc572a466fd6b88be4be0d8f894cd34252`,
  `story_done_prompt: 885`, `acceptance_evidence: production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`,
  plus new notes lines documenting PROMPT 874 / 877 / 885 dispositions
  + `sprint_13_story_done:` block extended with PROMPT 885 entry as
  the thirteenth `/story-done` block of Sprint 13),
  `production/session-state/active.md` (PROMPT 885 banner prepended
  above PROMPT 884 banner), `production/session-state/codex-orchestrator-state.md`
  (PROMPT 885 section prepended above the PROMPT 871 section).
- Cargo policy: **N/A** for PROMPT 885 — no `cargo` command invoked
  (paperwork-only closure; AC5 PASS verdict relies on PROMPT 877
  integrator's 32/0/0 targeted regression run, not a fresh
  PROMPT 885 run). PROMPT 874 worker + PROMPT 877 integration both
  applied the binding Windows/MSVC Cargo resource policy.
- Sprint 13 disposition UNCHANGED (`active`; PROMPT 885 is a per-row
  flip, NOT a Sprint 13 close-out). Stage UNCHANGED (`Polish`).
  PROMPT 761 Polish→Release gate-check `FAIL` preserved.
- Sprint 13 progress after PROMPT 885: 6 of 6 Must Have done (track
  COMPLETE per PROMPT 871); 4 of 6 Should Have done (UNCHANGED by
  PROMPT 885); **6 of 7 Nice to Have done** (PROMPT 885 closes the
  SIXTH Nice to Have row, after PROMPT 840 / 865 / 868 / 869+882
  carry / 876); total **16 of 19** rows closed at integration tip.
  Only `S11-HUD-TIMER-EYEBALL-VISUAL-001` + `S13-CONN-LOST-UX-001`
  (Should Have) and `S13-OPS-WIN-APPCOMPAT-NOTE-001` (Nice to Have)
  remain `ready`.
- Report: `reports/PROMPT-885-S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-STORY-DONE.md`
  (mandatory final report file; `reports/` is gitignored).
- No `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`,
  `/qa-plan`, `/dev-story`, `/story-readiness`, Sprint close-out,
  `S8-QA-001-W1` closure, or remediation-story authoring run by
  PROMPT 885. The audit instrumentation remains armed; any future R2
  Placement crash recurrence must trigger a follow-on story under
  `production/epics/server/` with the precise repro evidence — the
  remediation-fix gate is explicitly preserved.

## Conditions carried forward unchanged by PROMPT 885

- `S8-QA-001-W1` manual / browser two-client GAME_OVER gap remains
  OPEN.
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk
  (friend-game scope only).
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved (folded into
  Sprint 12 story 019 cannot-reproduce closure; third same-scope
  retest NOT authorised per `TQ-S12-C2`; not advanced by PROMPT 885).
- PROMPT 761 Polish→Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 885 scope.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed
  (closed cannot-reproduce, NOT bug-fixed).
- `TQ-S12-C1..C7` (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- All 13 prior Sprint 13 `/story-done` closures (PROMPT 833 / 835
  inline / 840 / 843 / 844 / 850 / 851 / 854 / 856 / 865 / 868 / 869
  via 882 carry / 871 / 876 / 884) preserved unchanged on
  `origin/main`; PROMPT 885 does NOT re-claim or modify them.

## Explicitly NOT claimed by PROMPT 885

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish→Release gate-check retry
- Stage advance from Polish to Release
- underlying R2 Placement runtime crash bug fix (the audit instruments
  the transition for future repro capture; **no fix lands under this
  story even if a repro occurs**)
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` or
  `S13-CONN-LOST-UX-001` (remaining ready Should Have rows)
- closure of `S13-OPS-WIN-APPCOMPAT-NOTE-001` (remaining ready Nice
  to Have row)
- Sprint 13 close-out (Sprint 13 remains `active`; only 16 of 19
  rows closed after PROMPT 885)
- full-workspace `cargo test --workspace --tests --no-fail-fast`
  result claim (per Sprint 13 QA-plan binding
  no-full-workspace-tests-by-default policy + PROMPT 874 worker scope
  + PROMPT 877 integrator scope; 32/0/0 targeted placement/RSM tests
  used)
- authoring of any follow-on remediation story (audit instrumentation
  remains armed; the follow-on remediation story is gated on a future
  R2 Placement crash repro capture)
