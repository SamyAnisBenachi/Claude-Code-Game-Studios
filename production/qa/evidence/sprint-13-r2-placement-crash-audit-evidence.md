# Sprint 13 -- R2 Placement Intermittent Runtime Crash -- Audit Evidence

> **Story**: `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`
> (see `production/epics/server/story-002-r2-placement-crash-audit.md`)
> **Sprint**: Sprint 13 (Nice to Have row)
> **Authoring source-of-truth**: worker base `origin/main@3cf5e41`
> **Worker prompt**: PROMPT 874
> **Story type**: Audit / Diagnostic -- audit log expansion only.
> **Result**: PASS (audit instrumentation landed; no fix; no behaviour change).

---

## Status / No-Claim Banner (verbatim restatement)

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

PROMPT 874 (this implementation prompt) is the worker landing for the
audit instrumentation only. It does NOT modify `client/`, `shared/`, or
any sprint-tracker file. The change is strictly server-side tracing
emissions inside two files:

- `server/src/core/rsm/transitions.rs`
- `server/src/feature/board/placement.rs`

---

## AC1 -- Audit-target sites enumerated (file:line evidence)

Grep on the worker base commit (`origin/main@3cf5e41`) for
`Phase::Placement` in `server/src/` produced the following authoritative
state mutation surface around the `Phase::Placement` round-2 transition:

### `server/src/core/rsm/transitions.rs`

| Site | File:line (pre-edit) | Authoritative mutation |
|------|----------------------|------------------------|
| `rsm_input_reader` placement_submitted reader | `transitions.rs:90-102` | `rsm.submissions_received.insert(...)`; conditional `pending.request(PhaseAdvanceRequest::new(RoundPhase::Placement))` |
| `advance_phase` entry | `transitions.rs:335-365` | branch dispatch on `rsm.phase`; capture of `from_phase` |
| `advance_phase` `RoundPhase::DraftInitial` arm (Placement R1 entry) | `transitions.rs:421-444` | `rsm.phase = RoundPhase::Placement`; timers cleared; `rsm.placement_timer` (re)installed; `PlacementPhaseEntered` + `BroadcastPhaseChanged` written |
| `advance_phase` `RoundPhase::DraftShop` arm (Placement R2+ entry) | `transitions.rs:471-497` | same shape as DraftInitial arm; this is the **canonical R2 Placement entry path** for non-auction rounds (`is_auction_round(2) == false`) |
| `advance_phase` `RoundPhase::Placement` arm (Placement exit) | `transitions.rs:498-522` | `rsm.phase = RoundPhase::Resolution`; placement timer cleared; resolution safety timer installed; `ResolutionPhaseEntered`, `BeginResolution`, `BroadcastPhaseChanged` written |

The round-2 (R2) Placement traversal is:

```text
R1 Resolution
  -> advance_phase RoundPhase::Resolution arm (round_number += 1, now 2)
  -> rsm.phase = RoundPhase::DraftShop (since is_auction_round(2) == false)
  -> rsm_input_reader observes ready signals or draft_shop_timer fires
  -> advance_phase RoundPhase::DraftShop arm
  -> rsm.phase = RoundPhase::Placement           [R2 Placement ENTRY]
  -> rsm_input_reader observes placement_submitted (or placement timer fires)
  -> advance_phase RoundPhase::Placement arm
  -> rsm.phase = RoundPhase::Resolution           [R2 Placement EXIT]
```

### `server/src/feature/board/placement.rs`

| Site | File:line (pre-edit) | Authoritative mutation |
|------|----------------------|------------------------|
| `placement_buffer_open` | `placement.rs:278-290` | `PendingPlacements::submissions.clear()` on `PlacementPhaseEntered` |
| `handle_placement_submission` | `placement.rs:385-431` | `PendingPlacements::submissions.insert(...)` (via `process_placement_submission`); writes `PlacementSubmitted` |
| `close_placement_phase` | `placement.rs:505-588` | (R2 Placement EXIT handler) mana deduction via `PlayerEconomies` mutation; `BoardGrid::lanes` mutation; `BoardOccupancy::{minion_slots, traps, structures, fields}` mutation; unit entity spawn (`Commands::spawn` with `Replicate`); writes `S2CPlacementReveal` and `PlacementCommitted` |
| `spawn_committed_placement` (called from `close_placement_phase`) | `placement.rs:895-957` | unit entity spawn + grid/occupancy update |

The most mutation-dense site by far is `close_placement_phase` -- it
deducts mana, sends a reliable S2C message, spawns N entities into the
authoritative ECS world, mutates board grid + occupancy maps, and emits
`PlacementCommitted`. It also has three pre-mutation early-return
guards (`catalog`, `server`/`sender`, `economies`) and one
mid-mutation early-return guard (`deduct_committed_mana == false`). Any
of these can short-circuit the close without visible evidence in the
pre-audit logs.

Other non-Placement-arm sites that contain `Phase::Placement` matches
but are NOT part of the audit transition path (left untouched):

- `server/src/core/rsm/transitions.rs:91` -- guard `if rsm.phase != RoundPhase::Placement` in `rsm_input_reader` (covered by audit log on the reader entry).
- `server/src/core/rsm/transitions.rs:252,261,498,741` -- timer-tick `match`, name lookup, protocol mapping (no authoritative mutation specific to R2 transition).
- `server/src/core/session/reconnect.rs:1160` -- protocol mapping (reconnect projection; not Placement-transition path).
- `server/src/core/session/snapshot.rs:467,479` -- protocol mapping + timer projection for snapshot (read-only with respect to RSM).
- `server/src/network/rsm_dispatch.rs:52` -- protocol mapping (S2CPhaseChanged dispatcher; transitively logs phase=Placement via existing target `server::game`).
- `server/src/feature/board/placement.rs:448` -- `process_placement_submission` validation guard.

---

## AC2 -- Audit logs added (tracing emissions only)

All added emissions use the module-path-scoped target
`server::game::placement`, consistent with the convention established
by Sprint 13 Must Have story 018 (`S13-OBS-TRACING-TARGETS-001`,
landed on `origin/main` as commit `9e32fbe` + `c1b7753` /story-done).
Wall-clock ISO-8601 timestamps come from Sprint 13 Must Have story 019
(`S13-OBS-WALLCLOCK-TIMESTAMPS-001`, landed as `a8ec25f` + `534d9df`).

### `server/src/core/rsm/transitions.rs`

1. **`rsm_input_reader` -- per-submission audit (`tracing::debug!`)** on
   every observed `PlacementSubmitted`, with fields
   `player_id, phase, round, submissions_seen`.
2. **`rsm_input_reader` -- post-insert audit (`tracing::debug!`)** after
   `rsm.submissions_received.insert(...)`, with fields
   `player_id, round, submissions_received`.
3. **`rsm_input_reader` -- quorum-reached audit (`tracing::info!`)** when
   `all_players_seen(...)` triggers the `Placement -> Resolution`
   advance request, with fields `round, submissions_received`.
4. **`advance_phase` -- entry audit (`tracing::debug!`)** at top of fn
   with fields `from_phase, expected_source, request_is_game_over, round`.
5. **`advance_phase` -- DraftInitial -> Placement arm**: entry
   `tracing::info!` (before mutation), substep `tracing::debug!` after
   state mutated (`timer_ms, has_timer`), substep `tracing::debug!`
   after event dispatch.
6. **`advance_phase` -- DraftShop -> Placement arm** (R2 canonical):
   entry `tracing::info!` (with `auction_round` flag), substep
   `tracing::debug!` after state mutated, substep `tracing::debug!`
   after event dispatch.
7. **`advance_phase` -- Placement -> Resolution arm** (R2 exit canonical):
   entry `tracing::info!` (with `submissions_received`), substep
   `tracing::debug!` after state mutated, substep `tracing::debug!`
   after event dispatch.

### `server/src/feature/board/placement.rs`

8. **`placement_buffer_open` -- audit entry (`tracing::info!`)** with
   `previous_submissions` field; placed alongside (after) the existing
   `target: "server::game"` info-log.
9. **`placement_buffer_open` -- post-clear audit (`tracing::debug!`)**
   confirming the pending submissions clear completed.
10. **`close_placement_phase` -- entry audit (`tracing::info!`)** with
    `round, pending_submissions` fields.
11. **`close_placement_phase` -- catalog-missing early return
    (`tracing::warn!`)** observability for the existing `let ... else`
    short-circuit.
12. **`close_placement_phase` -- server/sender-missing early return
    (`tracing::warn!`)** observability for the existing `(Ok(server), Some(sender))` short-circuit.
13. **`close_placement_phase` -- committed-sequence collected
    (`tracing::debug!`)** with `committed_players, committed_placements_len`.
14. **`close_placement_phase` -- economies-missing early return
    (`tracing::warn!`)** observability for the existing `economies.as_deref_mut()` short-circuit.
15. **`close_placement_phase` -- mana-deduction-failed early return
    (`tracing::warn!`)** observability for the existing
    `!deduct_committed_mana(...)` short-circuit.
16. **`close_placement_phase` -- mana-deducted substep
    (`tracing::debug!`)** after `PlacementCommitTraceEntry::ManaDeducted`.
17. **`close_placement_phase` -- S2C-reveal substep (`tracing::debug!`)**
    after `PlacementCommitTraceEntry::PlacementRevealEnqueued`, with
    `reveal_placements` count.
18. **`close_placement_phase` -- spawn-loop substep (`tracing::debug!`)**
    after all `spawn_committed_placement` calls, with `spawned_units`
    count.
19. **`close_placement_phase` -- exit audit (`tracing::info!`)** after
    `PlacementCommitted` event + `pending.submissions.clear()`, with
    `committed_players, spawned_units` counts.

Every added emission carries `target: "server::game::placement"` and a
trailing parenthetical `(audit: R2 placement transition audit)` to make
the audit instrumentation immediately greppable in operator log output.

---

## AC3 -- No behaviour change

No function signature, return type, control flow, or non-tracing
expression was altered. The only non-tracing edit is a single
`let reveal_placements_len = reveal.placements.len();` binding inside
`close_placement_phase` to allow the post-S2C `tracing::debug!` substep
to reference the placements count after `reveal` is moved into
`sender.send`. The existing `tracing::error!` on send failure was
updated to consume the new local instead of `reveal.placements.len()`,
preserving identical log content.

`cargo check -p server` passes (Finished `dev` profile [optimized] in
7.26s, no warnings). `cargo fmt -p server -- --check` passes silently.

## AC4 -- No fix for the underlying crash

No panic-guard, fallback path, defensive `?`/`unwrap_or_else`, or
suppression was added. The four pre-existing early-return short-circuits
in `close_placement_phase` were instrumented with `tracing::warn!`
emissions *before* the existing `return;` statements. The `return`
statements themselves are unchanged; control flow is identical to
pre-edit. If the underlying crash recurs, the same observable failure
mode (panic / process abort / silent early return) will fire, but now
with audit-level tracing evidence captured up to the failure point.

## AC5 -- Workspace test pass

PROMPT 874 ran the story-prescribed targeted checks only
(`cargo fmt -p server -- --check` PASS; `cargo check -p server` PASS).
The full workspace test run (`cargo test --workspace --tests
--no-fail-fast`) is out of scope for the worker prompt per the
PROMPT 874 instruction *"Verification: Run only story-prescribed
targeted checks. Do not run full workspace tests."* The integrating
prompt should run the full workspace test as part of `/story-readiness`
or `/story-done`. No new `#[ignore]` markers were introduced (no test
file edits in this commit).

## AC6 -- Repro-capture watch documented

During Sprint 13, the qa-tester / observer watches operator logs for
any process abort, panic, or unexpected termination that occurs while
the server is in `RoundPhase::Placement`. The new audit logs
(target `server::game::placement`, message bodies tagged
`(audit: R2 placement transition audit)`) provide the substep
breadcrumb trail leading up to the failure.

If a repro is captured:

1. **Preserve the operator log output verbatim** with at least the
   last 200 lines preceding the failure -- the new audit substeps
   should provide visibility into which substep was last entered and
   which authoritative state mutation was last applied.
2. **Note the round number** from the `round = ...` field on the
   audit log emissions.
3. **Author a follow-on story** under
   `production/epics/server/` with the precise repro evidence and
   recommended remediation scope.
4. **Do NOT implement a fix under this story.** This story is
   audit-only. The follow-on story is the gate for any code
   remediation.

## AC7 -- No client-side or protocol change

`git diff --stat origin/main -- 'client/**' 'shared/**'` produces
zero output. Confirmed pre-commit by the worker.

## AC8 -- Sprint 13 disposition preserved

`git diff --stat origin/main -- 'production/sprint-status.yaml'
'production/sprints/sprint-13.md' 'production/stage.txt'
'production/gate-checks/**'` produces zero output. Confirmed pre-commit
by the worker. PROMPT 761 Polish->Release gate-check FAIL evidence
preserved; not retried.

## AC9 -- Evidence document slot reserved

This document. NEW at the canonical path
`production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`.

---

## Diff summary

```text
git diff --stat origin/main...HEAD

 server/src/core/rsm/transitions.rs    | 85 +++++++++++++++++++++++++++++++++++
 server/src/feature/board/placement.rs | 73 +++++++++++++++++++++++++++++-
 2 files changed, 157 insertions(+), 1 deletion(-)
```

The single deletion is the inline `placements_len = reveal.placements.len()`
expression inside the existing `tracing::error!` call -- replaced by the
new local binding `reveal_placements_len` (used both by the unchanged
error log and the new audit substep log).

## Targeted checks executed (PROMPT 874)

| Check | Result |
|-------|--------|
| `cargo fmt -p server -- --check` | PASS (silent) |
| `cargo check -p server` | PASS (Finished `dev` profile [optimized] in 7.26s) |
| `git diff --check origin/main...HEAD` | PASS (silent; no whitespace defects) |
| `git diff --stat origin/main -- 'client/**' 'shared/**'` | PASS (zero output -- AC7) |
| `git diff --stat origin/main -- 'production/sprint-status.yaml' 'production/sprints/sprint-13.md' 'production/stage.txt' 'production/gate-checks/**'` | PASS (zero output -- AC8) |

Out of scope per the PROMPT 874 instruction *"Verification: Run only
story-prescribed targeted checks. Do not run full workspace tests."*:
`cargo test --workspace --tests --no-fail-fast`.

## Cross-links

- Source finding: Sprint 11 Wave 12 backlog 12:07 R2 Placement capture
  (not reproduced at 13:28 retry).
- Sprint 13 story 018 tracing-targets convention:
  `production/epics/observability/story-018-tracing-targets.md`
  (landed via PROMPT 847 commit `9e32fbe` and PROMPT 850
  `/story-done` `c1b7753`). Target string `server::game::placement` is a
  module-path-scoped sub-target consistent with story 018.
- Sprint 13 story 019 wall-clock timestamps:
  `production/epics/observability/story-019-wallclock-timestamps.md`
  (landed via PROMPT 837 commit `a8ec25f` and PROMPT 843
  `/story-done` `534d9df`). The audit emissions inherit ISO-8601 UTC
  timestamps from the global subscriber.
- Story file: `production/epics/server/story-002-r2-placement-crash-audit.md`.
- No-fix-restatement: This story is audit-only. NO FIX LANDS. If a
  repro is captured during Sprint 13, a follow-on story is authored
  with the precise repro and recommended remediation scope.

---

## Worker prompt log (PROMPT 874)

- Base commit: `origin/main@3cf5e41` (PROMPT 870 integration tip).
- Branch: `work/s13-r2-placement-crash-audit` (new, tracking
  `origin/main`).
- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\s13-r2-placement-crash-audit`.
- Files modified:
  - `server/src/core/rsm/transitions.rs` (added 7 audit emissions
    across `rsm_input_reader` + `advance_phase`).
  - `server/src/feature/board/placement.rs` (added 12 audit emissions
    across `placement_buffer_open` + `close_placement_phase`; one
    supporting `let reveal_placements_len` binding to enable
    post-send audit).
  - `production/qa/evidence/sprint-13-r2-placement-crash-audit-evidence.md`
    (NEW; this document).
- No edits to `client/**`, `shared/**`, `tests/**`,
  `production/sprint-status.yaml`, `production/sprints/**`,
  `production/stage.txt`, `production/gate-checks/**`,
  `production/session-state/**`.
- Cargo policy applied: `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`,
  `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
  `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`. No disk cleanup
  required.
- Skills activated: `liv-bevy-018` (any `.rs` edit, per
  technical-preferences routing). `liv-bevy-lightyear` not activated;
  `close_placement_phase` does call `ServerMultiMessageSender::send`
  but the existing call signature is unchanged -- no networking-API
  modification was needed.
