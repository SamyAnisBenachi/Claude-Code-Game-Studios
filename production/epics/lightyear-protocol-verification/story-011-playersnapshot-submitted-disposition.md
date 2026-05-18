# Story 011: S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001 — Drop-vs-Wire Decision for `PlayerSnapshot.submitted`

> **Epic**: Lightyear Protocol & Verification Spike
> **Story ID**: S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001
> **Status**: Draft — Sprint 18 candidate (NOT activated by this authoring run)
> **Layer**: Foundation / Protocol — disposition-first; implementation footprint depends on the chosen path
> **Type**: Decision-first (Path A — Drop / Path B — Wire) + Config/Data (protocol field removal — Path A) OR Integration (HUD wiring — Path B) + docs sync as required
> **Sprint**: Sprint 18 candidate (Sprint 17 remains the active sprint at the authoring source-of-truth; activation of Sprint 18 happens via a separate `/sprint-plan sprint-18` prompt, NOT this story)
> **Authored**: 2026-05-18 by PROMPT 1305 (branch `work/s18-server-dead-state-hygiene-story-authoring-1305`)
> **Authoring source-of-truth**: `origin/main@6239c9ee636ae9c71fac92ad9ee31d898925f9b8` (PROMPT 1300 windows dev launcher canonical-main repair integration)
> **Source audit**: `reports/PROMPT-1298-server-dead-state-hygiene-audit.md` §3 F-05

---

## Epic Ownership — Why `lightyear-protocol-verification`

The PROMPT 1298 §3 F-05 finding allowed either `lightyear-protocol-
verification` or HUD (if the wire variant is chosen). This story is placed
under `lightyear-protocol-verification` because the story is **decision-
first**: the deliverable is the drop-vs-wire decision and its rationale,
not (yet) HUD code. The disposition convention established by
`story-008-protocol-orphan-drain.md` in this same epic is the controlling
precedent — protocol-level decisions are recorded here, regardless of
where the eventual implementation lives.

If Path B (Wire) is chosen, the implementation footprint extends into the
`hud` epic. In that case the implementation prompt may either:

- **Option B-1 — Keep this story as the umbrella** (recommended) and let
  the HUD edit live in `client/src/ui/hud/mod.rs` under this story's
  acceptance criteria. The HUD edit is a follow-on commit in the same
  Sprint 18 wave, after `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` lands.
- **Option B-2 — Split into a follow-on HUD story** under the `hud` epic
  (e.g., `S18-HUD-OPPONENT-SUBMITTED-PIP-001`), with the protocol
  retention recorded here and the HUD wiring authored separately. The
  producer (or implementation prompt) records the split decision in
  this story's "Path B Split Decision" subsection if Path B is chosen.

If Path A (Drop) is chosen, the entire deliverable lives under this
story; no HUD epic is touched.

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
- Modify any file under `docs/architecture/**` (ADR / control-manifest
  edits are deliverables of the implementation prompt that lands this
  story; the authoring run only records the planned edits).
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan` on this story.
- **Pre-commit to drop-vs-wire.** The disposition is **decision-first** —
  the AC below makes the choice an explicit deliverable of the
  implementation prompt (with UX-designer + network-programmer co-sign),
  not of the authoring run.

This story does **not** claim: release readiness, sprint close-out,
gate-check pass, or any production state advance.

---

## Source Finding (PROMPT 1298 F-05)

`reports/PROMPT-1298-server-dead-state-hygiene-audit.md` §3 F-05:

- **Field**: `PlayerSnapshot.submitted: bool` (wire field) at
  `shared/src/protocol.rs:730-744`.
- **Writers**:
  - `server/src/core/session/snapshot.rs:110` —
    `submitted: submitted_for_player(world, player_id),` inside
    `build_player_snapshot`.
  - `server/src/core/session/snapshot.rs:229-234` —
    `submitted_for_player` helper that reads
    `RoundState.submissions_received.contains(&player_id)`.
  - `client/src/presentation/board_rendering/perf_harness.rs:556` —
    constructs a synthetic `PlayerSnapshot { ..., submitted: false }`
    for the perf harness (test-only fixture, not a production
    producer or reader).
- **Readers**:
  - `tests/unit/session/snapshot_secret_strip_test.rs:201` —
    `assert!(own.submitted);` (server-side unit test of the snapshot
    builder).
  - **No client production read.** All `.submitted` references in
    `client/src/ui/hand/mod.rs` and `client/src/presentation/qa_snapshot.
    rs` read the **client-local** `PlacementTimer.submitted` resource, a
    different field on a different type. The wire field is silently
    discarded after deserialisation.
- **Status**: Partially wired — server writes, server-side test asserts
  the write, no client production consumer. Dead on the wire.

### Coupling with F-07 (`submissions_received` stale-clear) — load-bearing for Path B

PROMPT 1298 §3 F-05 documents that `RoundState.submissions_received` is
cleared **only** on entry into a new `Placement` phase (`server/src/core/
rsm/transitions.rs:573, :641, :800`) — never on `Placement → Resolution`
exit. A snapshot built during `DraftAuction` or `DraftShop` between rounds
carries `submissions_received` flags from the prior placement.

Today this is hidden because no client production code reads
`PlayerSnapshot.submitted`. If Path B (Wire) is chosen, this stale-state
bug becomes player-visible — for example, an "opponent submitted ✓"
indicator in the placement HUD would persist into the next round's draft
phase.

The cross-lane fix for the stale clear is `S18-RSM-SUBMISSIONS-RECEIVED-
CLEAR-001` (PROMPT 1287 §5 SA-4). Path B of this story is **BLOCKED**
until that fix lands. Path A has no such dependency.

---

## Problem Class / Prevention Target

**Defect class (PROMPT 1298 audit row F-05)**: a wire field is populated
by the server, asserted by a server-side unit test, but never consumed
by the client. The wire bytes are silently discarded. The compiler cannot
detect this; the only safety net is manual audit.

**Prevention target**: reach a binding drop-vs-wire decision for
`PlayerSnapshot.submitted` (with UX-designer + network-programmer
co-sign per PROMPT 1298 §3 F-05 "Recommendation"). Either:

- **Path A — Drop the wire field** and re-anchor the server-side test
  directly on `RoundState.submissions_received`. The simplest path;
  removes wire bloat (~1 byte per snapshot per player); has no F-07
  dependency.
- **Path B — Wire the field to a HUD indicator** ("opponent submitted ✓")
  via `S2CGameSnapshot.players[i].submitted`. Requires
  `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` (the F-07 fix) to land FIRST.

The disposition itself — Path A vs Path B with rationale and co-sign —
is the primary deliverable.

---

## Per-Item Decisions (decision-first per the story-008 precedent)

### Disposition for `PlayerSnapshot.submitted` (the wire field)

- [ ] **Path A — Drop**. PROMPT 1298 §3 F-05 default recommendation
      ("least surface area until proven necessary"). Rationale template
      (must be filled by the implementation prompt if Path A is chosen):
      "Dropped because no client consumer is in scope for the friend-
      game milestone; the test assertion at `tests/unit/session/
      snapshot_secret_strip_test.rs:201` is re-anchored onto
      `RoundState.submissions_received` directly, preserving coverage of
      the snapshot builder's submission-tracking logic. UX-designer
      co-sign: [name]. Network-programmer co-sign: [name]."

- [ ] **Path B — Wire to HUD opponent-submitted indicator**. Rationale
      template (must be filled by the implementation prompt if Path B is
      chosen): "Wired because [specific UX value proposition cited by
      UX-designer]; HUD edit lives in `client/src/ui/hud/mod.rs` and
      surfaces an 'opponent submitted ✓' pip during the placement phase.
      Required prerequisite: `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`
      MUST be Done before this story's Path B implementation lands.
      UX-designer co-sign: [name]. Network-programmer co-sign: [name]."

- [ ] **Path C — Defer**. Recorded if the UX-designer and network-
      programmer cannot reach co-sign in the implementation prompt's
      review window. Rationale template: "Deferred to Sprint 19 pending
      co-sign on the UX value question; field retained on the wire for
      one more sprint with allowlist entry in `tests/invariants/
      protocol_completeness_test.rs` (S13-PROTO-INVARIANT-001 fixture)."

**Default recommendation (advisory, not binding)**: **Path A — Drop**,
per PROMPT 1298 §3 F-05.

The implementation prompt records exactly one of A / B / C with
rationale + co-sign(s) before staging any change. The decision IS the
first deliverable.

### If Path B is chosen — Path B Split Decision

The implementation prompt records one of:

- [ ] **Option B-1 — Umbrella under this story**. HUD edit lives in
      `client/src/ui/hud/mod.rs` under this story's AC; landing commit
      includes both the protocol retention rationale and the HUD wire-up.
      Recommended for small HUD edits (single pip).
- [ ] **Option B-2 — Split into a follow-on HUD story** (`S18-HUD-
      OPPONENT-SUBMITTED-PIP-001`) under the `hud` epic. Recommended if
      the HUD edit grows into a multi-state indicator (e.g., own +
      opponent + animated submit transition). The follow-on story
      inherits the F-07 prerequisite from this story's AC.

---

## Context

### Existing surface (PROMPT 1298 F-05 verbatim)

- `shared/src/protocol.rs:730-744` — `PlayerSnapshot` struct with
  `submitted: bool` field. The field sits between `mana_cap: u8` and
  `hand: Vec<CardId>` in the struct definition (audit anchor).
- `server/src/core/session/snapshot.rs:110` — write site inside
  `build_player_snapshot`.
- `server/src/core/session/snapshot.rs:229-234` — `submitted_for_player`
  helper.
- `tests/unit/session/snapshot_secret_strip_test.rs:201` — server-side
  test assertion `assert!(own.submitted);`.
- `client/src/presentation/board_rendering/perf_harness.rs:556` —
  synthetic `submitted: false` fixture (test-harness only).

### Live submissions-tracking source (PRESERVE regardless of path)

- `server/src/core/rsm/state.rs` — `RoundState.submissions_received:
  HashSet<PlayerId>` is the canonical source of truth for "who has
  submitted in the current placement window".
- `server/src/core/rsm/transitions.rs:573, :641, :800` — clear sites on
  entry to `Placement` (the F-07 finding: there is no clear site on
  `Placement → Resolution` exit).
- The placement-submission update path that adds to
  `submissions_received` is owned by the RSM submission-tracking system
  (see `production/epics/round-state-machine/story-003-timers-and-input-
  reader.md`). This story does NOT modify that path.

### Cross-lane fix `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` (F-07)

- Owned by `round-state-machine` epic per PROMPT 1287 §5 SA-4.
- Adds a clear site for `RoundState.submissions_received` on
  `Placement → Resolution` exit (or equivalent — exact placement is the
  F-07 story's deliverable).
- **Path B prerequisite**: this F-07 fix MUST be Done before Path B of
  the current story's implementation lands. Path A has no such
  dependency.
- This story does NOT implement the F-07 fix and does NOT assert AC on
  the clear site; it only declares the prerequisite.

### Engine

- **Engine**: Bevy 0.18 (Rust). Either path involves edits to
  `shared/src/protocol.rs` (Path A: field removal; Path B: no protocol
  change, only HUD wire-up).
- **Lightyear**: 0.26 — the wire field crosses the client/server
  boundary inside `S2CGameSnapshot.players[i]`.

### Mandatory skills

- **`liv-bevy-018`** — mandatory for any `.rs` edit under either path.
- **`liv-bevy-lightyear`** — mandatory for the protocol surface edit
  (Path A) and for any `S2CGameSnapshot` consumer review (Path B);
  cross-reference `docs/engine-reference/bevy/VERSION.md`.

### Control Manifest Rules (Foundation + Presentation scope)

- Required: Path B (Wire) implementation MUST NOT land until
  `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` is Done. The HUD indicator
  must NOT show stale "opponent submitted" state on subsequent draft
  phases. This is a hard gate enforced by AC-B-PREREQ below.
- Required: The disposition decision (Path A / Path B / Path C) is
  recorded in the implementation prompt's worker report with two co-
  sign annotations (UX-designer + network-programmer). The co-sign is
  the binding artifact, not the recommendation in this story.
- Required: If Path A is chosen, the existing server-side test
  assertion at `tests/unit/session/snapshot_secret_strip_test.rs:201`
  is re-anchored onto `RoundState.submissions_received` directly so
  that coverage of the snapshot builder's submission-tracking-write
  logic is not lost. Coverage MUST NOT regress.
- Required: ADR-002 (Client-Server Authority) is preserved regardless
  of path. The HUD consumer in Path B is read-only; the client never
  mutates `submitted` state from the wire field.
- Forbidden: Path B implementation landing in a Sprint 18 wave that
  does not include — and merge before — `S18-RSM-SUBMISSIONS-RECEIVED-
  CLEAR-001`.
- Forbidden: Adding a client-local mirror of `submitted` that is updated
  by anything other than the `S2CGameSnapshot` drain (no optimistic
  client-side authority — ADR-002 binding).
- Forbidden: Touching `client/src/ui/hud/mod.rs` in this story if Path
  A or Path C is chosen.

---

## Story Classification

**Story type**: Decision-first (drop vs wire) +

- *(Path A)* Config/Data — protocol field removal + test re-anchor.
- *(Path B)* Integration — HUD wire-up; gated on F-07.
- *(Path C)* Pure decision — allowlist entry, no code change.

This is **NOT** a:

- Pre-committed deletion story (the choice is made by the
  implementation prompt with co-sign, not by this authoring run).
- F-07 fix story (the `submissions_received` clear-on-exit fix is
  `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`, separate).
- HUD-only story (the disposition decision is the primary deliverable;
  HUD wire-up is a conditional follow-on).

---

## Acceptance Criteria

*Each AC is a verifiable post-condition checked by the implementation
prompt that lands this story.*

### Disposition decision — BLOCKING for any code change

- [ ] **AC-DECISION (BLOCKING)**: The implementation prompt records the
      disposition decision in its worker report as exactly one of:
      - **Path A — Drop** (with rationale, UX-designer co-sign,
        network-programmer co-sign).
      - **Path B — Wire** (with rationale, UX-designer co-sign,
        network-programmer co-sign, AND the chosen Path B split
        option B-1 or B-2).
      - **Path C — Defer** (with rationale; allowlist entry added to
        the `S13-PROTO-INVARIANT-001` fixture if it exists; follow-on
        sprint named).
      The decision is BLOCKING for any subsequent code change AC below.

### Path B prerequisite (only checked if Path B is chosen)

- [ ] **AC-B-PREREQ (BLOCKING for Path B only)**:
      `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` is Done on the merge base
      of this story's implementation branch. Verified by reading
      `production/epics/round-state-machine/EPIC.md` Stories table (or
      the equivalent index location) and confirming the F-07 fix
      story's status is `Complete` or `Done`. **If the F-07 story is
      not Done, Path B of this story is BLOCKED and the implementation
      prompt MUST either (a) downgrade to Path A, (b) re-decide to
      Path C, or (c) hold the implementation until F-07 lands.** Path
      B implementation MUST NOT land while F-07 is open.

### Path A acceptance criteria (only checked if Path A is chosen)

- [ ] **AC-A1**: `shared/src/protocol.rs` `PlayerSnapshot` struct
      (audit-reference `:730-744`) no longer contains a `submitted: bool`
      field. Verified by `grep -n "submitted" shared/src/protocol.rs`
      returning zero matches **inside the `PlayerSnapshot` struct body**
      (the implementation prompt should diff the struct definition; the
      keyword `submitted` may legitimately appear elsewhere in the file
      in unrelated contexts, so the grep is scoped).
- [ ] **AC-A2**: `server/src/core/session/snapshot.rs` no longer
      contains a `submitted` field initialiser inside
      `build_player_snapshot` (audit-reference `:110`) AND no longer
      contains the `submitted_for_player` helper function (audit-
      reference `:229-234`). Verified by `grep -n "submitted_for_player\|
      submitted:" server/src/core/session/snapshot.rs` returning zero
      matches.
- [ ] **AC-A3**: `tests/unit/session/snapshot_secret_strip_test.rs` is
      amended so the existing assertion at audit-reference `:201`
      (`assert!(own.submitted);`) is replaced with a direct assertion
      on `RoundState.submissions_received` membership for the same
      player, preserving coverage of the snapshot builder's submission-
      tracking-write logic. Verified by reading the test diff and
      confirming the new assertion drives `RoundState.submissions_
      received.contains(&player_id) == true` (or the equivalent path
      that exercises the same write).
- [ ] **AC-A4**: `client/src/presentation/board_rendering/perf_harness.
      rs:556` synthetic fixture is amended to drop the
      `submitted: false` field if the struct definition no longer has
      that field. Verified by `cargo check --workspace` green AND
      `grep -n "submitted" client/src/presentation/board_rendering/perf_
      harness.rs` showing no remaining reference to the removed wire
      field.
- [ ] **AC-A5**: Workspace-wide `grep -rn "\.submitted\b" client/src/
      shared/src/ server/src/ tests/` is enumerated by the
      implementation prompt; each remaining match is confirmed to be a
      reference to the unrelated `PlacementTimer.submitted` resource
      (client-local) or another unrelated field — NOT a reference to
      the now-removed `PlayerSnapshot.submitted` wire field. The
      implementation prompt quotes each remaining match with a
      one-line classification in the worker report.
- [ ] **AC-A6**: `cargo check --workspace` is green and zero new
      warnings land. (Run by the implementation prompt under the
      project's Windows/MSVC Cargo resource policy.)

### Path B acceptance criteria (only checked if Path B is chosen)

- [ ] **AC-B1**: `shared/src/protocol.rs` `PlayerSnapshot` continues to
      contain the `submitted: bool` field (Path B retains the wire
      surface). Verified by reading the struct definition.
- [ ] **AC-B2**: `client/src/ui/hud/mod.rs` (or the chosen HUD module
      per Option B-1 / B-2) consumes `snapshot.players[i].submitted`
      and renders an "opponent submitted ✓" pip (or equivalent
      UX-designer-specified indicator) during the placement phase only.
      Verified by reading the diff and confirming the read site exists
      AND is gated on `phase == Placement` (per ADR-009 phase-gate
      pattern for client UI). The drain is read-only — no client-local
      mirror is written from anything other than the `S2CGameSnapshot`
      drain.
- [ ] **AC-B3**: An integration test under `tests/integration/hud/` (or
      the equivalent test location for HUD consumers) asserts that an
      `S2CGameSnapshot` received during `DraftShop` after a Placement
      round shows `submitted = false` for both players in the consumed
      view. This test is the validation gate for the F-07 cross-lane
      fix (`S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`) — if the F-07 fix
      regresses, this test catches it. Verified by the test passing in
      CI on the merge base of this story's implementation.
- [ ] **AC-B4**: `cargo check --workspace` is green and zero new
      warnings land.
- [ ] **AC-B5**: If Option B-2 (split into HUD follow-on story) is
      chosen, the follow-on story file (`S18-HUD-OPPONENT-SUBMITTED-PIP-
      001` or producer-named equivalent) is authored under
      `production/epics/hud/` with the F-07 prerequisite inherited and
      a back-reference to this story. The follow-on story file is
      created by the implementation prompt; this authoring run does
      not pre-author it.

### Path C acceptance criteria (only checked if Path C is chosen)

- [ ] **AC-C1**: An allowlist entry naming `PlayerSnapshot.submitted` is
      added to `tests/invariants/protocol_completeness_test.rs`
      (S13-PROTO-INVARIANT-001 fixture) if that fixture exists. If the
      fixture does not exist, AC-C1 is a no-op and the deferral
      rationale lives in this story's "Per-Item Decisions" section
      and in the implementation prompt's worker report.
- [ ] **AC-C2**: A follow-on Sprint 19 story slot is named in the
      worker report (e.g., `S19-PROTO-PLAYERSNAPSHOT-SUBMITTED-
      DISPOSITION-001`). Authoring of the follow-on story is out of
      scope for the implementation prompt under Path C.

### Cross-path acceptance criteria

- [ ] **AC-X1** (cross-path): Whichever path is chosen, the worker
      report includes:
      - The disposition decision (A / B / C).
      - The UX-designer co-sign annotation (name or pseudonym).
      - The network-programmer co-sign annotation (name or pseudonym).
      - A quoted output of `git diff origin/main..HEAD -- server/src/
        core/rsm/` showing no changes from this story (RSM is preserved
        across all paths; F-07 fix is a separate lane).
- [ ] **AC-X2** (cross-path): `production/epics/lightyear-protocol-
      verification/EPIC.md` Stories table is updated to add a row for
      this story with the final status. (The authoring run adds the
      Draft row; the implementation prompt updates the status to
      Complete / Done on landing.)

---

## Out of Scope

- **F-07 fix** (`S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`) — this story
  declares the prerequisite for Path B but does not implement the
  fix. The fix lives under `production/epics/round-state-machine/`.
- **F-06 / F-09** — the two other PROMPT 1298 findings are owned by
  `S18-PROTO-CLASSCHOICE-DROP-001` and
  `S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001` respectively.
- **Own-player "submitted" indicator** in placement HUD — the
  client-local `PlacementTimer.submitted` resource at
  `client/src/ui/hand/mod.rs` (and the `qa_snapshot.rs` reference)
  already surfaces the local player's own submitted state. The wire
  field disposition discussed here is about the OPPONENT's submitted
  state visible to the local player. Anything touching
  `PlacementTimer.submitted` is out of scope.
- **Sprint activation** and any sprint-status / session-state /
  stage.txt edits — handled by the orchestrator outside this story.
- **QA plan amendments** — Sprint 18 QA plan authoring is a separate
  `/qa-plan` invocation.

---

## QA Test Cases

*Behavioural test cases; the implementation prompt verifies each.*

- **AC-DECISION**
  - **Given**: PROMPT 1298 §3 F-05 + the UX-designer + network-
    programmer review pair.
  - **When**: the implementation prompt records the disposition.
  - **Then**: exactly one of Path A / Path B / Path C is checked with
    rationale + two co-sign annotations.

- **AC-B-PREREQ (Path B only)**
  - **Given**: the working branch on `origin/main@<merge-base>`.
  - **When**: the implementation prompt reads `production/epics/round-
    state-machine/EPIC.md` (or the F-07 story file directly).
  - **Then**: the F-07 story status is `Complete` / `Done`. If not, the
    HUD edit MUST NOT be staged.

- **AC-A1-A6 (Path A)**
  - **Given**: the working branch after the Path A implementation
    prompt lands.
  - **When**: workspace greps for `submitted` references and
    `cargo check --workspace` run.
  - **Then**: zero `PlayerSnapshot.submitted` references remain; the
    re-anchored test passes; build is green.

- **AC-B1-B5 (Path B)**
  - **Given**: the working branch + Done F-07 fix on the merge base.
  - **When**: the new integration test under `tests/integration/hud/`
    runs.
  - **Then**: the test asserts `submitted = false` for both players in
    a snapshot received during `DraftShop` after a Placement round.

- **AC-C1-C2 (Path C)**
  - **Given**: the worker report.
  - **When**: the implementation prompt records the deferral.
  - **Then**: allowlist entry added (if invariant fixture exists);
    Sprint 19 follow-on slot named.

---

## Test Evidence

**Story Type**: Decision-first + (Path-dependent footprint)
**Required evidence**:

1. The disposition record (Path A / Path B / Path C) with rationale and
   both co-sign annotations (AC-DECISION).
2. *(Path B only)* Quoted snippet from
   `production/epics/round-state-machine/EPIC.md` showing F-07 story
   `Done` status (AC-B-PREREQ).
3. *(Path A)* Quoted output of `grep -rn "\.submitted\b" client/src/
   shared/src/ server/src/ tests/` with each remaining match
   classified (AC-A5); `cargo check --workspace` exit status (AC-A6);
   evidence file `tests/evidence/lyp-story-011-playersnapshot-submitted-
   drop.md`.
4. *(Path B)* Quoted diff snippet of the HUD wire-up site (AC-B2); new
   integration test path + pass status (AC-B3); `cargo check --workspace`
   exit status (AC-B4); evidence file `tests/evidence/lyp-story-011-
   playersnapshot-submitted-wire.md`.
5. *(Path C)* Allowlist diff snippet (if invariant fixture exists);
   follow-on Sprint 19 story slot name; evidence file
   `tests/evidence/lyp-story-011-playersnapshot-submitted-defer.md`.

**Status**: Not yet created (authoring run only). Created by the
implementation prompt.

---

## Dependencies

- **Path A**: no dependencies. Standalone on `origin/main@6239c9e`.
- **Path B**: depends on `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`
  (PROMPT 1287 §5 SA-4) being **Done** on the merge base of this
  story's implementation. AC-B-PREREQ enforces.
- **Path C**: no dependencies; allowlist + Sprint 19 follow-on.

- **Unlocks**:
  - *(Path A)* future protocol slimming on `S2CGameSnapshot`; cleaner
    `S13-PROTO-INVARIANT-001` fixture.
  - *(Path B)* HUD opponent-submitted indicator landed; F-07 regression
    test in place.
  - *(Path C)* nothing immediately; deferral entry created for the
    `S13-PROTO-INVARIANT-001` invariant.

- **Sprint 18 lane**: PROMPT 1287 §4.3 Lane A2 (server hygiene) +
  *(Path B only)* PROMPT 1287 §3.2 HUD-touching wave. The HUD wave
  carries a single-writer rule on `client/src/ui/hud/mod.rs`;
  serialise with any other HUD lane in the same sprint.

---

## Parallel Safety Notes

- **Path A**: Safe to run in parallel with most other Sprint 18 lanes.
  ⚠ Single-writer rule on `shared/src/protocol.rs` — serialise with
  `S18-PROTO-CLASSCHOICE-DROP-001` and any other protocol-touching
  lane in the same wave. ⚠ Single-writer rule on
  `server/src/core/session/snapshot.rs` — serialise with any other
  Sprint 18 lane that mutates the snapshot builder.
- **Path B**: ⚠ Single-writer rule on `client/src/ui/hud/mod.rs` —
  serialise with any other HUD-touching Sprint 18 lane. ⚠ Hard
  dependency on `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` landing first.
- **Path C**: ✅ Safe.

---

## Notes for the Implementation Prompt

- Read PROMPT 1298 §3 F-05 verbatim before deciding. The two viable
  paths and the F-07 coupling are quoted there.
- **The decision is the first deliverable.** Do not stage any code
  change before the UX-designer + network-programmer co-sign is
  recorded in the worker report. Path A is the default recommendation
  per PROMPT 1298 §3 F-05; Path B requires explicit justification of
  the UX value proposition.
- **Path B is HARD-BLOCKED on F-07.** Before staging any HUD edit,
  read `production/epics/round-state-machine/EPIC.md` (or the F-07
  story file directly) and confirm the F-07 story status is `Done` /
  `Complete`. If not, downgrade to Path A or Path C, or hold the
  implementation until F-07 lands. Do not introduce a HUD indicator
  that will show stale state.
- Activate `liv-bevy-018` for `.rs` edits. Activate `liv-bevy-lightyear`
  for any `S2CGameSnapshot` / `register_protocol` review.
- *(Path A only)* Re-anchor the snapshot-builder test on
  `RoundState.submissions_received` directly; do not delete the test —
  the underlying logic (the snapshot builder reads the submissions
  set) is still load-bearing and must remain covered.
- *(Path B only)* The integration test under `tests/integration/hud/`
  (AC-B3) is the validation gate for F-07. If the F-07 fix regresses,
  this test catches it. Author the test even if it duplicates
  coverage from the F-07 story — defence in depth on a player-visible
  stale-state bug is justified.
- Do not pre-commit to any path in this story file. The authoring
  recommendation is **Path A**, but the binding decision is the
  implementation prompt's.
