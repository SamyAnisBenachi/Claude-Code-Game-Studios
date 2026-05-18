# Story 007: S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001 -- Clear `submissions_received` on Placement→Resolution Exit

> **Epic**: Round State Machine
> **Story ID**: `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`
> **Status**: Draft -- future Sprint 18 candidate; NOT activated. No sprint plan currently activates this row. Sprint 17 remains `active` per `production/sprint-status.yaml`. `production/sprints/sprint-18.md` does not exist at authoring time. `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, and every `production/session-state/*` file are NOT modified by this authoring run.
> **Layer**: Core / Server -- Round State Machine transition table (Placement→Resolution exit hygiene)
> **Type**: Logic -- one-line transition fix + regression test
> **Sprint**: Sprint 18 candidate (story-authoring lane SA-4 per `reports/PROMPT-1287-sprint-18-parallel-lane-readiness-map.md` §3.10 Lane W9). Authoring does NOT activate Sprint 18.
> **Authored**: 2026-05-18 by PROMPT 1295 (`S18-STORY-AUTHORING-WAVE-B`)
> **Authoring worktree**: `D:\tmp\wt-1295`
> **Authoring branch**: `work/s18-story-authoring-wave-b-1295`
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 `plan(s18): draft Sprint 18 plan`)
> **Source audit**: `reports/PROMPT-1202-multiplayer-protocol-state-consistency-bug-audit.md` §2 row F-07 (P2 -- latent today; visible the moment any client reads `PlayerSnapshot.submitted`).

---

## Status / No-Claim Banner

This story is authored by PROMPT 1295 as a **future Sprint 18 candidate**. PROMPT 1295 is a branch-only story-authoring run.

PROMPT 1295 (this authoring run) does **NOT**:

- Activate Sprint 18.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-17.md`, `production/sprints/sprint-18.md` (does not exist at authoring), or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/` / `Trunk.toml` file.
- Retry the PROMPT 761 Polish→Release gate-check.

This story does **NOT** claim: public release readiness, release-candidate readiness, full game completion, broad / Standard-tier accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production completion (`PAW-TD-*-a`), closure of any other AUDIT-1131-* / AUDIT-1076-* / HUNT-1201-* finding, or stage advance.

Sprint 17 disposition (`active`) and all prior Sprint dispositions are preserved unchanged. PROMPT 761 Polish→Release gate-check FAIL evidence at `production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**ADR-002 + ADR-009 + ADR-010 binding preserved.** The repair is a single-line addition to the SOLE writer of `ResMut<RoundState>` (`server/src/core/rsm/transitions.rs::advance_phase`) inside the Placement→Resolution arm. No new authoritative state surface, no new C2S / S2C message, no new protocol shape, no new resource, no schedule reordering. The repair removes a latent stale-state foot-gun documented in PROMPT 1202 F-07; it does NOT change any observable Lightyear wire output today.

---

## Source Finding (PROMPT 1202)

`reports/PROMPT-1202-multiplayer-protocol-state-consistency-bug-audit.md` §2 row F-07 ("`submissions_received` never cleared on Placement→Resolution"):

> **Reset sites**: `transitions.rs:462, 530, 687` -- all are *entry* to a new Placement. Never cleared on *exit* from Placement.
>
> **Read site**: `server/src/core/session/snapshot.rs:229-234` (`submitted_for_player`).
>
> **Symptom**: if a client requests a snapshot during DraftAuction or DraftShop *between* rounds (e.g. via the `C2SRequestSnapshot` desync-recovery path while a phase chip animation is mid-transition), every player who submitted in the previous Placement will be marked `submitted=true` in the response, even though the current phase is no longer Placement.
>
> **Latent because**: F-05 -- the client doesn't read the field today. The moment F-05 is addressed (by wiring the field to UI), this becomes a visible bug.
>
> **Repair surface (minimal, single-line)**: add `rsm.submissions_received.clear();` to the Placement→Resolution arm at `transitions.rs:573` (right after `rsm.placement_timer = None;`). Pair with a regression test asserting `submissions_received.is_empty()` after the `advance_phase` from Placement.

Audit line anchor `transitions.rs:573` is the line number at audit-source-of-truth `origin/main@efb698e` (PROMPT 1202 head). At PROMPT 1295 authoring source-of-truth `origin/main@1345c6b`, the same statement has relocated to `server/src/core/rsm/transitions.rs:686` (`rsm.placement_timer = None;` inside the `RoundPhase::Placement =>` arm of `advance_phase`). The `/dev-story` worker MUST re-anchor by **the Placement→Resolution arm + the `placement_timer = None;` adjacent statement**, NOT by the stale line number. The intent of the repair is unchanged.

Cross-reference: `reports/PROMPT-1287-sprint-18-parallel-lane-readiness-map.md` §3.10 Lane W9 and §5 row SA-4.

---

## Problem

`RoundState.submissions_received: HashSet<PlayerId>` is the server-authoritative set of players whose `C2SSubmitPlacement` has been accepted in the current Placement phase. It is **cleared on Placement entry**:

- `RoundPhase::Lobby` → `DraftInitial` → `Placement` enter branch (`enter_draft_initial` and the Lobby arm; current main lines `:397`, `:509`, `:687`, `:800`, `:802` after refactor).
- `RoundPhase::DraftInitial` → `Placement` arm (`server/src/core/rsm/transitions.rs:573` at current main).
- `RoundPhase::DraftShop` → `Placement` arm (`server/src/core/rsm/transitions.rs:641` at current main).

It is **never cleared on Placement→Resolution exit**. The `RoundPhase::Placement =>` arm at `transitions.rs:676-720` (current main) currently mutates:

```rust
rsm.phase = RoundPhase::Resolution;
rsm.placement_timer = None;
rsm.placement_deadline_grace_timer = None;
rsm.resolution_safety_timer = config
    .as_ref()
    .map(|config| once_timer(config.resolution_max_duration_seconds));
```

`submissions_received` retains the per-player `PlayerId`s from the previous Placement. The next-round entry (DraftInitial → Placement at `:573` or DraftShop → Placement at `:641`) re-clears it before that round's submissions arrive, so the **field's observable read in the next Placement phase is correct**. The bug surfaces only in the **between-rounds window** (DraftAuction or DraftShop in the auction-round path; DraftShop in the non-auction-round path):

| Window | `RoundState.phase` | `submissions_received` contents | Snapshot `PlayerSnapshot.submitted` for each player |
|---|---|---|---|
| End of Placement N (just submitted) | `Resolution` | `{P1, P2}` (carried over) | `true` for both -- correct (phase IS Resolution, but the previous Placement just ended) |
| Resolution body | `Resolution` | `{P1, P2}` (still carried) | `true` for both -- **stale-but-harmless** while phase is Resolution |
| Resolution → DraftAuction (auction round) | `DraftAuction` | `{P1, P2}` (stale) | `true` for both -- **WRONG**: no Placement is in progress |
| DraftAuction → DraftShop | `DraftShop` | `{P1, P2}` (stale) | `true` for both -- **WRONG** |
| Resolution → DraftShop (non-auction round) | `DraftShop` | `{P1, P2}` (stale) | `true` for both -- **WRONG** |
| DraftShop → Placement (N+1 entry) | `Placement` | `{}` (cleared by entry arm) | `false` for both -- correct (clean Placement N+1) |

The bug is **observable today only via** `C2SRequestSnapshot` → `handle_request_snapshot` → `submitted_for_player` (`server/src/core/session/snapshot.rs:229-234`):

```rust
fn submitted_for_player(world: &World, player_id: PlayerId) -> bool {
    world
        .get_resource::<RoundState>()
        .map(|rsm| rsm.submissions_received.contains(&player_id))
        .unwrap_or(false)
}
```

A snapshot requested in the between-rounds window returns `PlayerSnapshot.submitted=true` for players whose submission belongs to the **previous** Placement. The audit flags this as P2 because:

1. **Latent at HEAD**: F-05 (`PlayerSnapshot.submitted` is unread on the client) means no current client surface displays the stale flag. The on-wire payload is correct in shape, just carries stale truth.
2. **Becomes a foot-gun the moment any consumer reads `submitted`**: any future client surface that wires the field (e.g. a "waiting on opponent" placement-HUD indicator) will misrender opponent state during DraftAuction / DraftShop. This includes any retroactive consumer in the test fleet that reads the field via `S2CGameSnapshot`.
3. **Reset-on-entry hides the bug from in-round symptoms**: every clean Placement entry re-clears the set, so the bug is invisible to single-round tests. The audit found it only via a static-state read of the transition table (PROMPT 1202 §4) plus the snapshot-builder read-site walk.

**Class**: per-phase server-authoritative state surface that is reset on the wrong side of a state boundary. Cleanup-on-entry is the correct discipline for **inbound** state (a future submission writer needs an empty set to start counting), but `submissions_received` is logically scoped to a single Placement and **its lifetime should end at Placement exit**, not at the next Placement entry, so that any read of the set outside Placement (snapshot, future UI consumer, future test) reports the truth: "no Placement is currently accepting submissions."

---

## Contract (server / RSM state-source ownership)

| Concern | Owner | Repair scope |
|---|---|---|
| `RoundState` resource (the SOLE `ResMut<RoundState>` writer is `advance_phase`) | Server / RSM | One-line addition to the Placement→Resolution arm |
| `submissions_received: HashSet<PlayerId>` field | Server / RSM | Lifecycle clarified: cleared on Placement entry (existing) AND on Placement→Resolution exit (new) |
| `submitted_for_player` snapshot read (`server/src/core/session/snapshot.rs:229`) | Server / session snapshot | Unchanged. The read continues to consult `rsm.submissions_received`; after this repair, the read returns `false` for both players during the between-rounds window. |
| `S2CGameSnapshot` shape | Protocol | Unchanged. `PlayerSnapshot.submitted: bool` shape preserved. |
| Client-side consumers of `PlayerSnapshot.submitted` | Client | **Unchanged.** F-05 is OUT OF SCOPE for this story. If/when a client surface wires the field, this repair guarantees the read is correct in the between-rounds window. |
| RSM phase-gate pattern | RSM | Unchanged. The phase-gate pattern in C2S handlers and `rsm_input_reader` continues to read `RoundState.phase`, NOT `submissions_received`. |
| F2 emission ordering inside the Placement→Resolution arm | RSM | Unchanged. The new `rsm.submissions_received.clear();` is a **state mutation**, not a `MessageWriter::write()` call -- it is appended to the substep mutation block (alongside `placement_timer = None`, `placement_deadline_grace_timer = None`, `resolution_safety_timer = ...`) and runs BEFORE the `MessageWriter` calls (`resolution_entered.write`, `begin_resolution.write`, `broadcast.write`). It does NOT alter the F2 emission order (which is event-order, not state-mutation-order). |

This story introduces **zero new protocol shape**, **zero new authoritative state**, and **zero new resource**. It corrects the lifecycle of an existing field.

---

## Acceptance Criteria

All criteria are independently checkable.

### Repair scope

- [ ] **AC1 -- One-line repair in `advance_phase` Placement→Resolution arm**: GIVEN `server/src/core/rsm/transitions.rs` at the future implementation source-of-truth, WHEN `advance_phase` runs the `RoundPhase::Placement =>` match arm, THEN the arm mutates `rsm.submissions_received.clear();` immediately adjacent to the existing `rsm.placement_timer = None;` statement (worker MAY place it before or after; `before` is recommended for consistency with the audit anchor). The arm continues to set `rsm.phase = RoundPhase::Resolution;`, `rsm.placement_deadline_grace_timer = None;`, and `rsm.resolution_safety_timer = ...` as it does today. No other arm is modified.

- [ ] **AC2 -- The repair is the SOLE production-code change**: GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN the only production source modified is `server/src/core/rsm/transitions.rs` (the one-line addition inside the `RoundPhase::Placement =>` arm). No edits to `shared/src/protocol.rs`, no edits to `client/src/`, no edits to any other file under `server/src/`. No new resource, no new `MessageWriter` call, no new `MessageReader` consumer, no new `Message` type. No `Cargo.toml` / `Cargo.lock` change.

- [ ] **AC3 -- F2 emission order preserved**: GIVEN the modified Placement→Resolution arm, WHEN `advance_phase` runs, THEN the linear `MessageWriter::write()` call order inside the arm is unchanged: `resolution_entered.write(ResolutionPhaseEntered { ... })` → `begin_resolution.write(BeginResolution { ... })` → `broadcast.write(BroadcastPhaseChanged { phase: Resolution, ... })`. The new `clear()` runs in the state-mutation block BEFORE the emission block. The integration test asserts no emission-order change relative to a pre-repair baseline (a snapshot of the message order may be embedded in the test, or the test may simply assert all three emissions occur and `broadcast.write` is the last call).

- [ ] **AC4 -- Existing reset-on-entry sites preserved**: GIVEN the modified transition table, WHEN `advance_phase` runs the existing Placement-entry arms (`Lobby → DraftInitial`, `DraftInitial → Placement`, `DraftShop → Placement`, `enter_draft_initial`), THEN the existing `rsm.submissions_received.clear();` statements (currently at `transitions.rs:573`, `:641`, `:800`, plus `enter_draft_initial` paths) remain in place. The repair is **additive**: this story does NOT remove any existing reset site. Belt-and-braces clears at Placement entry are explicit defence-in-depth against any future regression in this story's exit-clear.

### Regression test (BLOCKING)

- [ ] **AC5 -- Regression test in `server/tests/rsm_transitions_test.rs`**: A new test drives the RSM through `Lobby → DraftInitial → Placement → Resolution` using the existing test scaffold pattern (look for existing `advance_phase` invocations under `server/tests/rsm_transitions_test.rs` for the canonical setup: insert `RoundState`, insert `SessionConfig`, insert / mock other resources `advance_phase` reads). The test:
  1. Drives the RSM to `RoundPhase::DraftInitial`, then advances to `RoundPhase::Placement` via the `DraftInitial → Placement` path.
  2. Simulates **both players submitting** by inserting `PlayerId` entries into `RoundState.submissions_received` (e.g. via `world.resource_mut::<RoundState>().submissions_received.insert(PlayerId(1)); ... insert(PlayerId(2));`).
  3. Asserts `RoundState.submissions_received` contains both `PlayerId`s before the Placement→Resolution advance.
  4. Calls `advance_phase` to trigger the `RoundPhase::Placement =>` arm.
  5. Asserts `RoundState.phase == RoundPhase::Resolution`.
  6. **Asserts `RoundState.submissions_received.is_empty()`** -- THE BLOCKING ASSERTION.
  7. Asserts `RoundState.placement_timer.is_none()` (existing behaviour; included as a co-regression).
  8. Asserts `RoundState.placement_deadline_grace_timer.is_none()` (existing behaviour; co-regression).
  Test name: recommended `placement_to_resolution_clears_submissions_received` (or equivalent per local test-naming pattern). Worker discretion within `server/tests/rsm_transitions_test.rs`.

- [ ] **AC6 -- Test fails on `origin/main@<pre-repair HEAD>`**: GIVEN the test from AC5 is added to a pre-repair worktree, WHEN `cargo test -p server --test rsm_transitions_test` runs, THEN the test FAILS at the `is_empty()` assertion (the field still contains `{P1, P2}` because the exit-clear has not yet been added). This is the **proof-of-prevention** condition. The `/dev-story` worker MAY commit the test alone first (failing) and then the one-line repair (passing) as two ordered commits, OR may land both in the same commit; either pattern is acceptable provided the test fails before the repair and passes after.

- [ ] **AC7 -- Test passes on `origin/main@<post-repair HEAD>`**: GIVEN the test from AC5 + the one-line repair from AC1 in the same worktree, WHEN `cargo test -p server --test rsm_transitions_test` runs, THEN the new test PASSES along with all pre-existing tests in `server/tests/rsm_transitions_test.rs`. No existing test regresses.

### Scope guards

- [ ] **AC8 -- No protocol shape change**: GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN `shared/src/protocol.rs` is **NOT** modified. No new `S2C*` or `C2S*` message variants. No new fields on `S2CGameSnapshot`, `PlayerSnapshot`, `S2CPhaseChanged`, or any other message. No new `Message` type, no new channel binding, no new `RoundPhase` variant, no new `DraftPhase` variant. The on-wire payload shape on `origin/main@1345c6b` is preserved bit-for-bit.

- [ ] **AC9 -- No client-side behaviour change**: GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN no source under `client/src/` is modified. No new `MessageReceiver<T>` drain, no new presentation-layer system, no new HUD / Hand UI / Board surface. F-05 (`PlayerSnapshot.submitted` unread) is explicitly OUT OF SCOPE for this story.

- [ ] **AC10 -- No new resource, no new schedule wiring**: GIVEN `cargo check --workspace`, WHEN run, THEN no new `Resource` is registered under `server/src/core/rsm/plugin.rs` or anywhere else. No new `SystemSet`, no new system, no new schedule order, no new plugin. The `RsmPlugin` registration is byte-identical to its pre-repair state apart from the one-line addition inside `advance_phase`.

- [ ] **AC11 -- ADR-009 + ADR-010 invariants preserved**: GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN:
  - **ADR-009 sole-writer invariant**: `grep -rE "ResMut<RoundState>" server/src/ | grep -v transitions.rs` continues to return zero matches. `advance_phase` remains the sole writer.
  - **ADR-010 buffered-message invariant**: `grep -rE "EventWriter|EventReader|Events<|add_event" server/src/core/rsm/` continues to return zero matches. The repair adds no message types; the addition is a state mutation.
  - **ADR-010 F2 emission order invariant**: the linear `MessageWriter::write()` call order inside `advance_phase` match arms is unchanged. `BroadcastPhaseChanged` continues to be the LAST `.write()` call in every arm that broadcasts.

- [ ] **AC12 -- ADR-002 binding preserved**: GIVEN the repair, WHEN inspected, THEN no client-side optimistic authority is introduced. The repair is server-authoritative state hygiene. No `C2S*` handler is modified. No phase-gate pattern is altered.

- [ ] **AC13 -- Windows / MSVC Cargo resource policy compliance**: GIVEN the implementation runs on Windows / MSVC per project convention, WHEN `cargo test -p server --test rsm_transitions_test` is invoked, THEN the test respects the project's Cargo resource policy (run sequentially / under the conventional `-- --test-threads=1` or equivalent if required by the local harness). The test does not introduce a long-running fixture; it is a `World::new()`-class unit test extension and should complete in well under a second.

- [ ] **AC14 -- No QA-plan / smoke / gate-check artifact modification**: GIVEN the implementation worker branch diff, WHEN inspected, THEN no file under `production/qa/`, `production/gate-checks/`, `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/sprints/sprint-18.md`, `production/stage.txt`, `production/session-state/*` is modified by `/dev-story` of this story. `/story-done` of this story may update only this story file's status banner and Closure Trail section. QA-plan binding (Sprint 18 QA plan, when authored) is established by a separate `/qa-plan` prompt -- not this story.

### Authoring-only scope (PROMPT 1295)

- [ ] **AC15 -- PROMPT 1295 authoring-only scope contained**: GIVEN PROMPT 1295 worker branch (`work/s18-story-authoring-wave-b-1295`) diff, WHEN inspected, THEN the only files modified by PROMPT 1295 are:
  - `production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md` (NEW; this file)
  - `production/epics/round-state-machine/EPIC.md` (index update only -- appending the story-007 row)
  - `production/epics/lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md` (NEW; sibling Wave-B story)
  - `production/epics/lightyear-protocol-verification/EPIC.md` (index update only -- appending the story-009 row)
  - `reports/PROMPT-1295-s18-story-authoring-wave-b.md` (NEW; the worker report)
  No code under `client/`, `server/`, `shared/`, `tests/`. No GDD edit. No ADR edit. No sprint plan edit. No QA artifact edit. No production session-state edit. No `production/sprint-status.yaml` edit. No `production/stage.txt` edit. No Cargo / Trunk / CI edit. No skill / agent edit.

- [ ] **AC16 -- Worker branch scope contained for the future `/dev-story` worker**: GIVEN the future implementation worker branch (slug recommendation: `work/s18-rsm-submissions-received-clear-<prompt-N>`), WHEN inspected, THEN it pushes only the worker branch -- never `main`. Files changed at worker time are scoped to:
  - `server/src/core/rsm/transitions.rs` (the one-line addition inside the `RoundPhase::Placement =>` arm)
  - `server/tests/rsm_transitions_test.rs` (the new regression test from AC5)
  - This story file's Closure Trail section (status flip + evidence pointers; performed by `/story-done` after worker DONE)
  No other file is modified by the implementation worker.

---

## Dependencies (must hold before `/dev-story` on this story)

| Dependency | Slug / Story | Why blocking |
|---|---|---|
| `RoundState` resource + `submissions_received: HashSet<PlayerId>` field | Story 001 (Complete) | The field this story is correcting; existing scaffold is the substrate. |
| `advance_phase` SOLE writer + `RoundPhase::Placement =>` match arm | Story 002 (Complete) | The arm being modified must exist at HEAD. |
| `server/tests/rsm_transitions_test.rs` test harness | Story 002 / 003 (Complete -- harness lands alongside `advance_phase` tests) | The new regression test extends this file. |
| Sprint 18 plan file `production/sprints/sprint-18.md` | Sprint 18 plan authoring prompt | The story may be authored before the Sprint 18 plan exists (PROMPT 1295 is branch-only), but `/dev-story` of this story SHOULD wait for Sprint 18 activation (status flip from Draft to Ready) -- otherwise paperwork drift between story-status and sprint-plan-status. **Soft dependency**; if Sprint 18 activates without flipping this story, the worker may still implement, but `/story-done` of the worker output must align the two. |

**Optional but recommended** (not blocking):

- Coordination with the future F-05 closure story (`PlayerSnapshot.submitted` client read site). The two stories are **parallel-safe**: this story corrects the server-side write lifecycle; F-05 wires the client-side read. The order does not matter for correctness, but **landing this story BEFORE any F-05 closure is preferred** so that a F-05 closure tests against the correct between-rounds value (`submitted=false`) rather than the stale value (`submitted=true`).
- Light entry in `docs/architecture/tr-registry.yaml` -- when the registry is populated (currently noted as not-yet-populated in `production/epics/round-state-machine/EPIC.md`), this repair maps to an extension of `TR-RSM-03` ("Timers and per-Placement state hygiene on `RoundState`") or a new `TR-RSM-11` ("`submissions_received` lifecycle ends at Placement→Resolution exit"). The TR-registry edit is performed by the `/dev-story` implementation prompt or a sibling `/architecture-review` prompt -- NOT by PROMPT 1295.

---

## Test Evidence

**Story Type**: Logic.

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type", Logic stories deliver **automated unit test -- must pass** as BLOCKING evidence. The regression test from AC5-AC7 is the required evidence.

**Evidence location**: `server/tests/rsm_transitions_test.rs` (extension). A short evidence note SHOULD be added to `tests/evidence/` (e.g. `tests/evidence/rsm-story-007-check.md`) by the `/dev-story` implementation prompt summarising:

1. Pre-repair test FAIL output (the `is_empty()` assertion fails).
2. Post-repair test PASS output.
3. Confirmation that all pre-existing `rsm_transitions_test.rs` tests continue to PASS.

CI link / run ID (or local `cargo test` invocation transcript on Windows / MSVC) recorded in the evidence note.

---

## Implementation Notes (advisory; `/dev-story` may deviate with rationale)

- **Placement of the new statement**: recommended directly before `rsm.placement_timer = None;` inside the `RoundPhase::Placement =>` arm, matching the audit anchor wording ("right after `rsm.placement_timer = None;`" in the audit text is interchangeable with "right before" -- the two statements are independent state mutations). Worker discretion; the BLOCKING constraint is that the clear happens inside the arm before the `MessageWriter::write()` emission block.
- **`HashSet::clear()` is `O(n)` on the capacity** but `n ≤ 2` for Lanes and Lies (two-player game); the cost is trivially bounded.
- **No `mem::take`** or other replacement strategy is needed. `.clear()` is the canonical idiom and matches the three existing reset-on-entry sites.
- **Logging**: the existing `tracing::info!` at the arm entry (`"advance_phase: Placement->Resolution entry (audit: R2 placement transition audit)"`) already logs `submissions_received = rsm.submissions_received.len()` at line `:681`. After the repair, that log line continues to report the pre-clear count (which is the meaningful diagnostic -- "how many submissions did this Placement see"). The worker MAY add a second `tracing::debug!` after the clear that confirms the post-clear `len() == 0`, but this is OPTIONAL.
- **Independence from F-05**: the audit notes that F-07 is "latent because F-05" (the client doesn't read the field). This story does NOT close F-05; it only ensures that the server-side state is correct WHEN F-05 is eventually closed. F-05 closure is a sibling story owned elsewhere (per PROMPT 1202 §2 row F-05 and the audit's repair-surface notes).
- **Independence from F-09**: F-09 (`auction_safety_timer` is dead state) is a separate sibling audit row and is OUT OF SCOPE for this story. Do NOT remove the field or change its lifecycle as part of this repair.

---

## Out of Scope

*Handled by neighbouring stories or sibling audit closures -- do NOT implement here:*

- F-05 closure (`PlayerSnapshot.submitted` client read site -- separate sibling story owner)
- F-01 closure (`S2COpponentDisconnected` send site)
- F-02 closure (`C2SActivateCard` server handler decision)
- F-03 closure (`S2CSessionSettingsUpdated` initial broadcast on join)
- F-04 closure (`S2CSangMepriseReveal` client drain -- ADR-gated)
- F-06 closure (`C2SClassChoice` dead-code C2S decision)
- F-08 closure (real-wire snapshot test helper -- sibling Wave-B story `S18-PROTOCOL-SNAPSHOT-REAL-WIRE-TESTS-001` at `production/epics/lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md`)
- F-09 closure (`auction_safety_timer` dead-state removal)
- Any TR-registry population pass (separate `/architecture-review` prompt)
- Any GDD edit to `design/gdd/round-state-machine.md` (the repair matches the GDD's existing implicit semantics -- "`submissions_received` is per-Placement state"; no GDD wording change is required)

---

## QA Test Cases (informal -- for `/qa-plan` to formalise when Sprint 18 activates)

- **AC1 / AC5 -- Submissions cleared on Placement→Resolution exit**
  - **Given**: RSM in `RoundPhase::Placement` with `submissions_received = {PlayerId(1), PlayerId(2)}`
  - **When**: `advance_phase` runs and takes the `RoundPhase::Placement =>` arm
  - **Then**: `phase == RoundPhase::Resolution` AND `submissions_received.is_empty() == true` AND `placement_timer.is_none() == true` AND `placement_deadline_grace_timer.is_none() == true`

- **AC4 -- Reset-on-entry preserved**
  - **Given**: RSM in `RoundPhase::DraftShop` with `submissions_received = {}` (cleared by this story's exit-clear)
  - **When**: `advance_phase` runs the `DraftShop → Placement` arm and the worker (or a test fixture) writes `submissions_received.insert(PlayerId(1))` before invocation
  - **Then**: After the entry arm runs, `submissions_received` is once again `{}` (the entry-clear remains in place; this story does NOT remove it)

- **AC3 -- F2 emission order preserved**
  - **Given**: a `MessageReader` test fixture observing `ResolutionPhaseEntered`, `BeginResolution`, and `BroadcastPhaseChanged`
  - **When**: `advance_phase` takes the Placement→Resolution arm
  - **Then**: all three messages are observed AND `BroadcastPhaseChanged` is the last of the three in arrival order

- **AC8 -- No protocol shape change** (CI grep gate)
  - **Given**: `git diff <activation HEAD>..HEAD -- shared/src/protocol.rs`
  - **When**: inspected
  - **Then**: zero lines changed

- **AC11 -- ADR-009 sole-writer invariant preserved** (CI grep gate)
  - **Given**: `grep -rE "ResMut<RoundState>" server/src/ | grep -v transitions.rs`
  - **When**: run on post-repair source
  - **Then**: zero matches

- **Between-rounds snapshot assertion (optional follow-on test, NOT BLOCKING for this story)**
  - **Given**: a `C2SRequestSnapshot` issued during `RoundPhase::DraftShop` immediately after a Placement N where both players submitted
  - **When**: `handle_request_snapshot` runs and `submitted_for_player` is invoked for each player
  - **Then**: `submitted_for_player(PlayerId(1)) == false` AND `submitted_for_player(PlayerId(2)) == false`
  - **Note**: this is the audit's recommended T-05 follow-on test (`server/tests/snapshot_post_round_test.rs` new). It is OPTIONAL for this story's BLOCKING evidence. The `/dev-story` worker MAY include it; if so, it lives in a separate test file (`server/tests/snapshot_post_round_test.rs`) and does NOT merge with the AC5 test.

---

## Closure Trail (filled by `/story-done` after worker DONE)

- Worker branch:                  *(filled at worker DONE)*
- Worker commit:                  *(filled at worker DONE)*
- Worker source-of-truth base:    *(filled at worker DONE)*
- Integration / merge commit:     *(filled at integration DONE)*
- `/story-done` PROMPT N:         *(filled at /story-done)*
- Evidence note:                  `tests/evidence/rsm-story-007-check.md` *(target path)*
- CI run ID / local cargo log:    *(filled at /story-done)*
- Code review verdict:            *(filled at /story-done; lean-mode acceptable per local convention)*
- Deviations from this story:     *(none expected; record any here)*

---

## Cross-References

- Source audit: `reports/PROMPT-1202-multiplayer-protocol-state-consistency-bug-audit.md` §2 row F-07
- Lane map: `reports/PROMPT-1287-sprint-18-parallel-lane-readiness-map.md` §3.10 Lane W9 + §5 row SA-4
- Sibling Wave-B story: `production/epics/lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md` (F-08 closure)
- Sibling audit rows OUT OF SCOPE: F-01 / F-02 / F-03 / F-04 / F-05 / F-06 / F-09 (each owned separately)
- Authoring report: `reports/PROMPT-1295-s18-story-authoring-wave-b.md`
- Governing ADRs: ADR-009 (RSM Phase State as ECS Resource) + ADR-010 (RSM Phase Event Bus). Both binding; neither modified.
