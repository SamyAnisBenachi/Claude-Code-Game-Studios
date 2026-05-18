# Story 007: S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001 — Remove Dead `RoundState.auction_safety_timer`

> **Epic**: Round State Machine
> **Story ID**: S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001
> **Status**: Draft — Sprint 18 candidate (NOT activated by this authoring run)
> **Layer**: Core
> **Type**: Config/Data (struct field removal) + docs sync (ADR-009, EPIC.md, story-001)
> **Sprint**: Sprint 18 candidate (Sprint 17 remains the active sprint at the authoring source-of-truth; activation of Sprint 18 happens via a separate `/sprint-plan sprint-18` prompt, NOT this story)
> **Authored**: 2026-05-18 by PROMPT 1305 (branch `work/s18-server-dead-state-hygiene-story-authoring-1305`)
> **Authoring source-of-truth**: `origin/main@6239c9ee636ae9c71fac92ad9ee31d898925f9b8` (PROMPT 1300 windows dev launcher canonical-main repair integration)
> **Source audit**: `reports/PROMPT-1298-server-dead-state-hygiene-audit.md` §3 F-09

---

## Status / No-Claim Banner

This story is authored as a Sprint 18 candidate. PROMPT 1305 (this
authoring run) does **NOT**:

- Activate Sprint 18.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-18.md` or any other sprint file.
- Modify `production/stage.txt`.
- Modify any `production/session-state/*` file.
- Modify `production/qa/**` or `production/gate-checks/**`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any file under `docs/architecture/**` (ADR amendments are tracked as
  deliverables inside the implementation prompt that lands this story; the
  authoring run only records the planned ADR edits).
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan` on this story.

This story does **not** claim: release readiness, sprint close-out,
gate-check pass, or any production state advance.

If Sprint 18 is already active on a later `origin/main` than the source-of-
truth above, this story file should still be respected as candidate-grade
material; the activation status is recorded on `production/sprint-status.yaml`
(not edited here).

---

## Source Finding (PROMPT 1298 F-09)

`reports/PROMPT-1298-server-dead-state-hygiene-audit.md` §3 F-09:

- **Field**: `RoundState.auction_safety_timer: Option<Timer>` (definition
  `server/src/core/rsm/state.rs:42`, initialised `None` at `state.rs:65`).
- **Writers** (origin/main, exhaustive):
  - `server/src/core/rsm/state.rs:65` — `auction_safety_timer: None` in
    `RoundState::new`.
  - `server/src/core/rsm/transitions.rs:514` — `rsm.auction_safety_timer =
    None;` inside the `request.game_over` arm of `advance_phase`.
- **Readers** (origin/main, exhaustive):
  - `server/tests/rsm_scaffold_test.rs:19` —
    `assert!(state.auction_safety_timer.is_none());` (initialisation-only
    invariant).
- **Tick coverage**: `tick_rsm_timers` (`transitions.rs:363-368`) explicitly
  does **not** tick `auction_safety_timer`; the `DraftAuction` arm is in the
  inert `Lobby | DraftAuction | GameOver` branch.
- **Status**: Dead state. Never set to `Some(Timer::new(...))` anywhere in the
  workspace; never ticks; only "clear" write is on `GameOver`.

**Live auction safety mechanism (must be preserved)**: the live auction
settlement clock lives at `server/src/feature/auction/state.rs:31` as
`AuctionState.live_bidding_deadline_elapsed_ms` (wall-clock anchor via
`Time<Real>`). This field — and the auction-system tick that drives it
(PROMPT 1091, AUDIT-1076-12) — is the only safety net the auction needs and
**must not be touched** by this story. The RSM-level `auction_safety_timer`
is structurally redundant scaffolding that this story removes.

---

## Problem Class / Prevention Target

**Defect class (PROMPT 1298 audit row F-09)**: a `RoundState` field is
written but never meaningfully read — initialisation `None`, "clear" `None`
on GameOver, and one test assertion on the initialisation `None`. There is
no `Some(_)` writer and no tick path. A future contributor reading
`RoundState::new` or the GameOver arm might believe the auction phase has an
RSM-level safety net; in practice, the only auction safety net is
`AuctionState.live_bidding_deadline_elapsed_ms`.

**Prevention target**: remove the dead field, the two `None` writes, the
initialisation-only test assertion, and the ADR/EPIC/story doc references
that still claim `auction_safety_timer` is part of the canonical phase-state.
After this story lands, the only documented and implemented auction safety
mechanism is `AuctionState.live_bidding_deadline_elapsed_ms`.

---

## Context

### Existing surface (PROMPT 1298 F-09 verbatim)

- `RoundState.auction_safety_timer: Option<Timer>` — defined at
  `server/src/core/rsm/state.rs:42`.
- Initialised `None` at `state.rs:65` (`RoundState::new`).
- Cleared `None` at `server/src/core/rsm/transitions.rs:514` inside
  `advance_phase`'s game-over arm.
- Asserted `is_none()` at `server/tests/rsm_scaffold_test.rs:19`.
- Not ticked: `tick_rsm_timers` at `transitions.rs:363-368` routes
  `RoundPhase::DraftAuction` to the inert branch.

### Live auction safety mechanism (PRESERVE)

- `AuctionState.live_bidding_deadline_elapsed_ms` at
  `server/src/feature/auction/state.rs:31` — owns the live auction
  settlement clock; consumed by the auction tick system (`server/src/feature/
  auction/system.rs`) and the auction settlement path. Cross-reference
  PROMPT 1091 / AUDIT-1076-12 in commit history if needed.
- This story must not touch any file under `server/src/feature/auction/**`
  and must not modify `AuctionState`.

### Docs that reference the dead field (must be amended)

- `docs/architecture/adr-009-rsm-phase-state.md` — phase-state diagram
  and struct snippet at approximately `:123` and `:183` reference
  `auction_safety_timer` as a canonical phase-state field. The implementation
  prompt must remove these references and add an inline note that the
  auction safety net lives in `AuctionState.live_bidding_deadline_elapsed_ms`
  per the feature/auction module.
- `production/epics/round-state-machine/EPIC.md` — Deliverables section
  lists `auction_safety_timer: Option<Timer>` in the `RoundState` resource
  bullet (the line that enumerates RSM resource fields). This bullet must be
  removed in the same commit set as the code deletion.
- `production/epics/round-state-machine/story-001-state-and-events-scaffold.md`
  — Acceptance Criteria lists `auction_safety_timer: Option<Timer>` as a
  required field on `RoundState`. The AC line must be edited to drop the
  field reference and to add an inline note that the live auction safety
  mechanism lives in `AuctionState.live_bidding_deadline_elapsed_ms`. The
  story-001 status remains Complete; this is a doc-sync edit, NOT a
  re-implementation of story-001.

### Tests on the canonical auction safety net (preserved coverage)

The story does NOT add new tests for the canonical auction safety mechanism.
Coverage of `AuctionState.live_bidding_deadline_elapsed_ms` is already
landed under the auction-system epic (tests under `tests/unit/auction/**`
and `tests/integration/auction/**`) and under the PROMPT 1091 timer-anchor
work. The implementation prompt should verify (read-only) that at least one
test currently asserts auction safety behaviour against
`live_bidding_deadline_elapsed_ms` before deleting the
`rsm_scaffold_test.rs:19` assertion, and quote the test path in the worker
report.

### Engine

- **Engine**: Bevy 0.18 (Rust). The deletion is field removal on a Bevy
  `Resource` (`RoundState`) — purely a struct edit + initialiser edit + one
  match-arm write removal.
- **Lightyear**: 0.26 — not involved; `auction_safety_timer` is server-only
  state and never crossed the wire.

### Mandatory skills

- **`liv-bevy-018`** — mandatory for the `.rs` edits (Bevy `Resource` field
  removal, match-arm cleanup).
- **`liv-bevy-lightyear`** — NOT required (no protocol surface touched).

### Control Manifest Rules (Core layer)

- Required: `RoundState` remains the single source of truth for phase,
  round number, and timers; `advance_phase` remains the sole writer of
  `ResMut<RoundState>` — ADR-009. This story does not change that
  invariant; it removes a field that no writer ever set to a non-default
  value.
- Required: the canonical auction safety net (`AuctionState.
  live_bidding_deadline_elapsed_ms`) remains untouched and continues to be
  the auction's only safety mechanism.
- Forbidden: Adding a replacement `auction_safety_timer` field, or moving
  the existing field to another resource, or wiring a new RSM-level
  `DraftAuction` timer. The decision recorded here is REMOVE, not RENAME or
  RELOCATE.
- Forbidden: Touching any file under `server/src/feature/auction/**` (the
  live auction safety net is preserved by leaving the auction module
  alone).
- Forbidden: Touching `tests/integration/auction/**` or `tests/unit/auction/
  **` (live auction safety coverage is preserved by leaving auction tests
  alone).

---

## Story Classification

**Story type**: Config/Data (server `Resource` struct field removal) +
docs sync (ADR-009 + EPIC + story-001 references).

This is **NOT** a:

- Pure refactor story (a real semantic surface — the doc claim that the
  RSM owns auction safety — is corrected).
- Logic / Integration story (no new behaviour; one removed match-arm write
  is a no-op since the value is already `None`).
- Cross-system story (touches only `server/src/core/rsm/**` code + one
  server test + RSM-owned docs).

---

## Acceptance Criteria

*Each AC is a verifiable post-condition checked by the implementation
prompt that lands this story.*

- [ ] **AC1**: `server/src/core/rsm/state.rs` no longer contains a field
      named `auction_safety_timer` on the `RoundState` struct. Verified by
      `grep -n "auction_safety_timer" server/src/core/rsm/state.rs`
      returning zero matches.
- [ ] **AC2**: `server/src/core/rsm/state.rs` no longer contains the
      `auction_safety_timer: None,` initialiser inside `RoundState::new` (or
      `Default` impl, whichever the file uses). Verified by `grep -rn
      "auction_safety_timer" server/src/core/rsm/` returning zero matches.
- [ ] **AC3**: `server/src/core/rsm/transitions.rs` no longer contains
      `rsm.auction_safety_timer = None;` inside `advance_phase`'s game-over
      arm at approximately `:514`. Verified by `grep -n
      "auction_safety_timer" server/src/core/rsm/transitions.rs` returning
      zero matches.
- [ ] **AC4**: `server/tests/rsm_scaffold_test.rs:19`'s
      `assert!(state.auction_safety_timer.is_none());` line is deleted, and
      no other test in `server/tests/` or `tests/` references
      `auction_safety_timer`. Verified by `grep -rn
      "auction_safety_timer" server/tests/ tests/` returning zero matches.
- [ ] **AC5**: `tick_rsm_timers` in `server/src/core/rsm/transitions.rs`
      (approximately `:363-368`) is updated so the match arm covering
      `RoundPhase::DraftAuction` continues to route to the inert branch
      (no behavioural change). Verified by reading the arm in the diff:
      `Lobby | DraftAuction | GameOver => false,` (or equivalent) remains
      intact.
- [ ] **AC6** (**PRESERVATION — BLOCKING**): `server/src/feature/auction/
      state.rs` continues to define `AuctionState.live_bidding_deadline_
      elapsed_ms` and no edit lands under `server/src/feature/auction/**`
      as part of this story. Verified by `git diff origin/main..HEAD --
      server/src/feature/auction/` showing zero changes, AND by `grep -n
      "live_bidding_deadline_elapsed_ms" server/src/feature/auction/state.
      rs` continuing to return at least one match. The auction system tick
      that consumes this field (`server/src/feature/auction/system.rs`
      auction tick) remains unchanged.
- [ ] **AC7** (**PRESERVATION — BLOCKING**): At least one existing test
      under `tests/unit/auction/**` or `tests/integration/auction/**`
      continues to assert auction safety behaviour against
      `AuctionState.live_bidding_deadline_elapsed_ms` (or the equivalent
      auction-tick deadline path). The implementation prompt must quote
      the test path(s) in its worker report; if no such test exists, the
      story is BLOCKED and the deletion does NOT land until a coverage
      gap is recorded as a follow-on story.
- [ ] **AC8**: `docs/architecture/adr-009-rsm-phase-state.md` is amended
      to remove `auction_safety_timer` from the phase-state diagram and
      the struct snippet (approximately `:123` and `:183`), and adds an
      inline note that the auction safety net lives in
      `AuctionState.live_bidding_deadline_elapsed_ms` under the
      feature/auction module. Verified by `grep -n "auction_safety_timer"
      docs/architecture/adr-009-rsm-phase-state.md` returning zero matches.
- [ ] **AC9**: `production/epics/round-state-machine/EPIC.md` is amended
      to remove `auction_safety_timer: Option<Timer>` from the `RoundState`
      resource bullet in the Deliverables section. Verified by `grep -n
      "auction_safety_timer" production/epics/round-state-machine/EPIC.md`
      returning zero matches.
- [ ] **AC10**: `production/epics/round-state-machine/story-001-state-and-
      events-scaffold.md` is amended to drop the `auction_safety_timer:
      Option<Timer>` reference from the Acceptance Criteria bullet that
      enumerates `RoundState` fields, AND adds an inline note that the
      live auction safety mechanism lives in
      `AuctionState.live_bidding_deadline_elapsed_ms`. Story-001 status
      remains `Complete`; this is doc sync only. Verified by `grep -n
      "auction_safety_timer" production/epics/round-state-machine/story-001-
      state-and-events-scaffold.md` returning zero matches AND
      `live_bidding_deadline_elapsed_ms` appearing in an inline doc note.
- [ ] **AC11**: `cargo check --workspace` is green and zero new warnings
      land on `server/src/core/rsm/**` or `server/tests/rsm_scaffold_test.
      rs`. (Run by the implementation prompt under the project's
      Windows/MSVC Cargo resource policy; the authoring run does NOT run
      Cargo.)
- [ ] **AC12**: `docs/architecture/control-manifest.md` is searched for
      any rule referencing `auction_safety_timer` and any such rule is
      removed in the same commit set. Verified by `grep -n
      "auction_safety_timer" docs/architecture/control-manifest.md`
      returning zero matches.

---

## Out of Scope

- **Live auction safety mechanism** — `AuctionState.live_bidding_deadline_
  elapsed_ms` and its auction-system tick are explicitly preserved
  (AC6, AC7) and **must not be modified**. Any change to that mechanism
  is a separate auction-system story.
- **F-05 / F-06** — the two other PROMPT 1298 findings are owned by
  S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001 and
  S18-PROTO-CLASSCHOICE-DROP-001 respectively. Each is dispositioned in
  its own story; this story addresses F-09 only.
- **F-07** (`submissions_received` stale-clear) — separate story
  `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` per PROMPT 1287 §5 / SA-4. Not
  affected by this story.
- **`resolution_safety_timer`** — actively ticked and consumed (RSM-38);
  PROMPT 1298 confirmed it is NOT dead state. Out of scope.
- **Sprint activation** and any sprint-status / session-state /
  stage.txt edits — handled by the orchestrator outside this story.
- **QA plan amendments** — Sprint 18 QA plan authoring is a separate
  `/qa-plan` invocation; this story is implementation-grade material
  consumed by that plan, not the plan itself.

---

## QA Test Cases

*Behavioural test cases; the implementation prompt verifies each.*

- **AC1-AC5 (struct + match-arm + test removal)**
  - **Given**: `origin/main@6239c9e` + the working branch after the
    implementation prompt lands.
  - **When**: `grep -rn "auction_safety_timer" server/src/ server/tests/
    tests/` runs.
  - **Then**: Zero matches across the entire workspace.

- **AC6 (live auction safety preserved)**
  - **Given**: the working branch after this story lands.
  - **When**: `git diff origin/main..HEAD -- server/src/feature/auction/`
    runs.
  - **Then**: No changes in `server/src/feature/auction/**` and
    `grep -n "live_bidding_deadline_elapsed_ms" server/src/feature/
    auction/state.rs` returns at least one match.

- **AC7 (coverage preservation)**
  - **Given**: the working branch + the auction test suite at
    `origin/main@6239c9e`.
  - **When**: the implementation prompt enumerates auction-safety tests in
    `tests/unit/auction/**` and `tests/integration/auction/**`.
  - **Then**: at least one test path is quoted in the worker report that
    asserts auction safety behaviour against
    `live_bidding_deadline_elapsed_ms` (or the auction-tick deadline path).
    If zero such tests exist, the story is BLOCKED and the deletion does
    NOT land.

- **AC8-AC10, AC12 (doc sync)**
  - **Given**: the working branch.
  - **When**: `grep -rn "auction_safety_timer" docs/architecture/ production/
    epics/round-state-machine/` runs.
  - **Then**: Zero matches.

- **AC11 (build green)**
  - **Given**: the working branch on Windows/MSVC with the project's Cargo
    resource policy applied.
  - **When**: the implementation prompt runs `cargo check --workspace`.
  - **Then**: exit 0 with zero new warnings on `server/src/core/rsm/**`.

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**:

1. Quoted output of `grep -rn "auction_safety_timer" server/src/ server/
   tests/ tests/ docs/architecture/ production/epics/round-state-machine/
   docs/architecture/control-manifest.md` showing zero matches (AC1–AC5,
   AC8–AC10, AC12).
2. Quoted output of `git diff origin/main..HEAD -- server/src/feature/
   auction/` showing zero changes (AC6).
3. Quoted output of `grep -n "live_bidding_deadline_elapsed_ms" server/src/
   feature/auction/state.rs` showing ≥1 match (AC6).
4. The path(s) of the existing auction-safety test(s) that anchor coverage
   on `live_bidding_deadline_elapsed_ms` (AC7).
5. `cargo check --workspace` exit status + relevant warnings excerpt for
   `server/src/core/rsm/**` (AC11).
6. Evidence file path: `tests/evidence/rsm-story-007-auction-safety-timer-
   remove.md`.

**Status**: Not yet created (authoring run only). Created by the
implementation prompt.

---

## Dependencies

- **Depends on**: none. The PROMPT 1287 §4.3 Lane A2 parallel-lane map
  marks this finding as standalone (no cross-lane file collision risk).
- **Unlocks**: cleanup of the ADR-009 phase-state diagram (removes one
  source of reader confusion); no other story is blocked.
- **Sprint 18 lane**: PROMPT 1287 §4.3 Lane A2 (server hygiene) — parallel
  with §4.3 Lane A1 (other dead-state hygiene) and §3.x HUD/UI lanes.

---

## Parallel Safety Notes

- ✅ **Safe** to schedule in parallel with any other Sprint 18 lane that
  does NOT modify `server/src/core/rsm/state.rs`,
  `server/src/core/rsm/transitions.rs`, `server/tests/rsm_scaffold_test.
  rs`, `docs/architecture/adr-009-rsm-phase-state.md`,
  `docs/architecture/control-manifest.md`,
  `production/epics/round-state-machine/EPIC.md`, or
  `production/epics/round-state-machine/story-001-state-and-events-
  scaffold.md`.
- ⚠ **Single-writer rule** on `server/src/core/rsm/state.rs` and
  `transitions.rs` — serialise with any other Sprint 18 lane that mutates
  RSM state (notably `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` / SA-4 per
  PROMPT 1287 §5). The orchestrator schedules these waves; this story
  does not implement the schedule.

---

## Notes for the Implementation Prompt

- Read PROMPT 1298 §3 F-09 verbatim before editing; the deletion footprint
  is exactly the file set quoted in this story's Context section.
- Verify AC7 by reading the auction-system test suite **before** deleting
  the `rsm_scaffold_test.rs:19` assertion. If no test currently anchors on
  `live_bidding_deadline_elapsed_ms`, surface the gap as a follow-on
  story (`S18-AUCTION-SAFETY-TEST-EVIDENCE-001` or similar) and BLOCK this
  story until the gap is filled. Do not delete coverage that has no
  equivalent.
- The `rsm_scaffold_test.rs:19` assertion is the only test reference;
  there is no integration test on `auction_safety_timer` to migrate.
- Activate the `liv-bevy-018` skill for the `.rs` edits.
- Do not introduce a replacement RSM-level auction timer. Doing so
  reintroduces the same dead-state defect class and contradicts the
  PROMPT 1298 §3 F-09 recommendation.
