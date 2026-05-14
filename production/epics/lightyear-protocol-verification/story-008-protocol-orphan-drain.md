# Story 008: S13-PROTO-ORPHAN-DRAIN-001 -- Drain or Delete 8 S2C + 1 C2S Protocol Orphans

> **Epic**: Lightyear Protocol & Verification Spike
> **Story ID**: S13-PROTO-ORPHAN-DRAIN-001
> **Status**: Draft -- Sprint 13 candidate; NOT activated; Sprint 12 is the
> active sprint
> **Layer**: Foundation / Protocol
> **Type**: Decision-first per-message (drain vs delete-with-rationale) +
> Integration (drain wiring) + Config/Data (protocol deletion if chosen)
> **Sprint**: Sprint 13 candidate (per PROMPT 803 §6 line 142; NOT activated)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint
(`status: active` per `production/sprint-status.yaml` at
`origin/main@b5eef0d`) and must not be changed by this authoring run.
Activation of Sprint 13 happens via a separate `/sprint-plan sprint-13`
prompt after Sprint 12 close-out.

PROMPT 804 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md` or any other active sprint
  file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify `production/qa/qa-plan-sprint-12.md` or any other QA-plan file.
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

Sprint 10 disposition (`closed-with-conditions` per PROMPT 763) and
Sprint 11 disposition (`closed-with-conditions` per PROMPT 792) remain
unchanged. PROMPT 761 Polish->Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** Each orphan disposition (drain vs delete) lands a server-
authoritative drain (client-side draining server-broadcast S2C state
read-only into client view) or removes the unused message; no
disposition allows the client to mutate authoritative state outside the
existing shared phase sink / snapshot / S2C consumer pattern. ADR-002 +
ADR-008 + ADR-009 + ADR-011 + ADR-012 binding for every drain that
lands.

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-1** Protocol-registered-but-orphan-on-receive (HIGH): 8 S2C
  messages defined+registered but with NO client `MessageReceiver`
  drain; 1 C2S with no server handler stub. Evidence anchors:
  `shared/src/protocol.rs:71,81,82,85,92,93,103,105,107`;
  `server/src/main.rs:138-143` TODO.
- **§4 Lane A "8 S2C orphans (no client drain)"**:
  - `S2CHeartbeat` (`shared/src/protocol.rs:107`)
  - `S2COpponentDisconnected` (`shared/src/protocol.rs:92`)
  - `S2COpponentReconnected` (`shared/src/protocol.rs:93`)
  - `S2CPoolUpdate` (`shared/src/protocol.rs:85`)
  - `S2CPrismRespawned` (`shared/src/protocol.rs:82`)
  - `S2CPrismRewardDropped` (`shared/src/protocol.rs:81`)
  - `S2CSangMepriseReveal` (`shared/src/protocol.rs:105`)
  - `S2CSessionCancelled` (`shared/src/protocol.rs:103`)
- **§4 Lane A "1 C2S orphan"**: `C2SRequestSnapshot`
  (`shared/src/protocol.rs:71`) -- no server-side `MessageReceiver`;
  `server/src/main.rs:138-143` carries a TODO acknowledging the
  Lightyear handler wiring is incomplete here.
- **§5 Must row 2 (S13-PROTO-ORPHAN-DRAIN-001)**: "Add
  `MessageReceiver`/`MessageSender` for the 8 S2C orphans + the 1 C2S
  orphan (or delete them from the protocol with rationale)".
- **§9 Product Decisions Needed (3) (4)**: each orphan needs an
  explicit "drain vs delete" decision. Some are *probably* intentional
  latent capability (e.g., `S2COpponentReconnected`); each needs a
  written rationale.

---

## Problem Class / Prevention Target

**Defect class (DC-1)**: A message type is added to
`shared/src/protocol.rs` and registered, but no production code path
on the receiving side ever drains it. Symptoms are silent data loss
(S2C orphans) or a silent C2S no-op (server-side orphan). The
compiler cannot detect this; today's only mitigation is manual audit.

**Prevention target**: Each orphan reaches a binding decision recorded
in this story (or, if split, in a per-message follow-on story). The
decision is one of:

- **Path A -- Drain**: add a `MessageReceiver<T>` (or
  `MessageSender<T>` for the C2S handler) in the appropriate
  client/server module; wire the drain to a no-op handler with a log
  line + TODO if no immediate consumer exists; rationale records why
  the message is latent capability rather than active.
- **Path B -- Delete**: remove the message from
  `shared/src/protocol.rs`, remove its channel registration, remove
  any orphaned sender, and the rationale records why the message is
  not needed (e.g., never produced by any system, never planned to
  be).

The `S13-PROTO-INVARIANT-001` test (Story 007 in this epic) is the
machine-checked gate that this story's outcome restores to PASS. After
this story lands, the workspace invariant test passes with at most a
documented allowlist (each allowlist entry has an inline rationale +
follow-on reference).

---

## Context

### Existing surface (per PROMPT 803 §4 Lane A and §3 DC-1)

**8 S2C orphans -- defined and channel-registered, but no client drain
exists**:

| Orphan | `shared/src/protocol.rs:LINE` | Likely intent (advisory, not binding) |
|--------|--------------------------------|----------------------------------------|
| `S2CHeartbeat` | `:107` | UnreliableChannel keep-alive (ADR-008); client may want to ack or just observe. Likely "drain to log". |
| `S2COpponentDisconnected` | `:92` | Opponent connection lost mid-game; UI surface (e.g., "Opponent disconnected -- waiting...") is the obvious consumer. Pairs with `S2COpponentReconnected`. Likely "drain to UI". |
| `S2COpponentReconnected` | `:93` | Opponent reconnected; UI clears the disconnect modal. Pairs with `S2COpponentDisconnected`. Likely "drain to UI". |
| `S2CPoolUpdate` | `:85` | Card-pool refresh broadcast; client may want to update its pool view if a future feature exposes pool state to player. May be latent capability for ladder telemetry. |
| `S2CPrismRespawned` | `:82` | Prism (objective-adjacent entity) lifecycle event; consumer is presentation layer. |
| `S2CPrismRewardDropped` | `:81` | Reward drop event; consumer is presentation layer / economy view. |
| `S2CSangMepriseReveal` | `:105` | "Sang Méprise" reveal mechanism per ADR pending; this orphan is likely awaiting the reveal mechanism ADR cited as pending in `.claude/docs/technical-preferences.md`. |
| `S2CSessionCancelled` | `:103` | Session cancellation broadcast; consumer is lobby/session UI for graceful exit. Pairs with reconnect handling. |

**1 C2S orphan -- defined and channel-registered, but no server handler
exists**:

| Orphan | `shared/src/protocol.rs:LINE` | Likely intent |
|--------|--------------------------------|---------------|
| `C2SRequestSnapshot` | `:71` | Client-initiated snapshot request; the corresponding server handler is acknowledged TODO at `server/src/main.rs:138-143`. The server's reconnect path (`server/src/core/session/reconnect.rs`) already broadcasts snapshots; the question is whether `C2SRequestSnapshot` is a separate ad-hoc path the client can invoke (e.g., on stale-data heuristics) or whether the server's automatic snapshot at reconnect is the only path. Likely "drain with TODO" OR "delete". |

### Drain target locations (advisory, not binding)

- **Presentation layer drains**: `client/src/presentation/mod.rs` already
  hosts the shared phase sink (`apply_phase_changed_message` at
  ~`:163-222`) and is the canonical home for new S2C drains that update
  presentation state. New drains should follow this pattern
  (single-drainer rule per ADR-008).
- **UI surface drains**: lobby/session UI drains can live in
  `client/src/ui/lobby.rs` (mirror of existing `S2CClassLocked` drain
  at `:326-335`).
- **Network layer drains**: `client/src/network/mod.rs` is the
  canonical home for heartbeat / transport-adjacent drains.
- **Server-side C2S handler**: `server/src/network/` or
  `server/src/core/session/` is the canonical home for the
  `C2SRequestSnapshot` handler (if Path A). The reconnect snapshot
  builder at `server/src/core/session/snapshot.rs` is the reusable
  primitive.

### Idempotency interactions

- **DC-6 (Reconnect / snapshot late-message idempotency)**: drains
  added by this story should follow the `C2SAcknowledgeResult`
  idempotency precedent (`tests/integration/session/result_acknowledgement_contract_test.rs:91-96`).
  This is enforced more broadly under `S13-LATE-MSG-DEDUPE-001`
  (Sprint 13 candidate Story 020 in `playable-client`); coordinate to
  avoid double-implementation.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/network-protocol.md` Table A is the canonical
  message inventory. If Path B (delete) is chosen for any orphan, the
  GDD Table A entry must also be deleted with cross-reference.
- **ADR-002** (Client-Server Authority): every drain is read-only on
  the client; the client never mutates authoritative state from the
  drain. Server-side C2S handler treats request as advisory; server
  remains authoritative on the snapshot contents.
- **ADR-003** (Cargo Workspace Structure): all protocol changes live
  in `shared/src/protocol.rs`; both client and server consume the
  same module.
- **ADR-008** (Lightyear Channel Config): Path B deletions remove
  channel bindings; Path A drains use the existing reliable /
  unreliable channel assignment (no channel changes land in this
  story).
- **ADR-011** (Reconnect Snapshot): the `C2SRequestSnapshot` handler
  (Path A) must reuse the existing snapshot builder at
  `server/src/core/session/snapshot.rs`; no new snapshot construction
  path is added.
- **ADR-012** (SessionReady Delivery): drains for session lifecycle
  S2C messages (`S2CSessionCancelled`) must integrate with the
  SessionReady observer's flush ordering; no new ordering is
  introduced.

### Engine

- **Engine**: Bevy 0.18 (Rust). All new drains follow the Bevy 0.18
  `Update`-scheduled system pattern with `MessageReceiver<T>` system
  params (Lightyear 0.26 API).
- **Lightyear**: 0.26 (Bevy 0.18 compatible). `MessageReceiver<T>` /
  `MessageSender<T>` are the canonical Lightyear 0.26 system params.

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code touching Bevy.
- **`liv-bevy-lightyear`** -- mandatory for all protocol / network
  code. Lightyear 0.26 API is post-training-cutoff; cross-reference
  `docs/engine-reference/bevy/VERSION.md` and the Lightyear release
  notes before adding `MessageReceiver` / `MessageSender` system
  params or modifying `register_protocol(app)`.

### Control Manifest Rules (Foundation / Protocol scope)

- Required: Each orphan reaches a written drain-vs-delete decision
  with rationale in this story file (or in a per-message follow-on
  story, see "Per-Orphan Decisions" below).
- Required: Path A drains follow the single-drainer rule per ADR-008
  (Lightyear `MessageReceiver<T>` is drained exactly once in
  production code).
- Required: Path A drains are read-only on the client; they update
  presentation state via existing patterns (phase sink, snapshot
  sink, S2C consumers) and never mutate authoritative state.
- Required: Path B deletions remove the message from
  `shared/src/protocol.rs`, remove its channel binding, remove any
  orphaned senders, AND update `design/gdd/network-protocol.md`
  Table A in the same commit set.
- Required: Each drain landed in Path A has at least one integration
  test asserting the drain is invoked when the corresponding S2C /
  C2S message is sent. The test landing pattern follows existing
  precedents (e.g., `tests/integration/session/*`).
- Forbidden: Adding optimistic client-side authority for any S2C
  drain (ADR-002 binding).
- Forbidden: Modifying channel bindings (reliable vs unreliable) for
  any retained message. Channel binding changes are out of scope.
- Forbidden: Adding new C2S / S2C messages in this story. Only the
  9 named orphans are dispositioned here.

---

## Story Classification

**Story type**: Composite -- decision-first per orphan + Integration
(drain wiring for Path A) + Config/Data (protocol deletion for Path B).

This is **NOT** a:

- Pure evidence-only story (real code lands).
- Pure refactor story (each disposition is a feature-level decision).
- Single-message story (9 orphans dispositioned in one umbrella).

---

## Producer Decision (umbrella vs per-message split)

The implementation prompt (or a separate producer prompt) MUST record
exactly one of the following before any code change is staged.

- [ ] **Umbrella (this story)**: keep all 9 orphan dispositions in
      this story; close each cluster (8 S2C + 1 C2S) under
      `S13-PROTO-ORPHAN-DRAIN-001`. Rationale:
      _<implementation prompt fills in: e.g., the orphan set is small
      and shares the same decision shape; batching reduces re-review
      cost and lets the invariant test flip to PASS in one commit
      set>_

- [ ] **Split into per-message stories**: author up to 9 follow-on
      story files (one per orphan); close this umbrella story as the
      producer-decision-record artefact. Each split story inherits
      the no-claim banner, evidence-path conventions, and decision-
      first discipline from this story. Rationale:
      _<implementation prompt fills in: e.g., `S2CSangMepriseReveal`
      depends on the pending reveal-mechanism ADR; separating
      clarifies that block>_

The default producer recommendation (advisory, not binding) is
**umbrella**, because (a) the 9-orphan set is small; (b) batched review
of the drain-or-delete decision shape reduces re-review cost;
(c) flipping the `S13-PROTO-INVARIANT-001` test to PASS in one Sprint
13 wave is desirable. The split path is available when an individual
orphan blocks (e.g., `S2CSangMepriseReveal` waiting on a pending ADR).

---

## Per-Orphan Decisions

If the umbrella path is chosen, all 9 dispositions are recorded here.
If the split path is chosen, each follow-on story records its own
disposition.

### S2C orphans (8 rows)

#### `S2CHeartbeat` -- `shared/src/protocol.rs:107` (UnreliableChannel)

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

#### `S2COpponentDisconnected` -- `shared/src/protocol.rs:92`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

#### `S2COpponentReconnected` -- `shared/src/protocol.rs:93`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

#### `S2CPoolUpdate` -- `shared/src/protocol.rs:85`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

#### `S2CPrismRespawned` -- `shared/src/protocol.rs:82`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

#### `S2CPrismRewardDropped` -- `shared/src/protocol.rs:81`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

#### `S2CSangMepriseReveal` -- `shared/src/protocol.rs:105`

- [ ] **Path A -- Drain (with TODO)**. Rationale + drain location:
      _<implementation prompt fills in; e.g., "drain to a no-op log
      handler pending the Sang Méprise reveal mechanism ADR">_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_
- [ ] **Path C -- Defer to a per-message split story**. Rationale:
      _<implementation prompt fills in; e.g., "block on the pending
      reveal-mechanism ADR; track under a separate Sprint 14
      candidate row">_

#### `S2CSessionCancelled` -- `shared/src/protocol.rs:103`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete**. Rationale:
      _<implementation prompt fills in>_

### C2S orphan (1 row)

#### `C2SRequestSnapshot` -- `shared/src/protocol.rs:71` + `server/src/main.rs:138-143` TODO

- [ ] **Path A -- Add server handler that reuses the reconnect snapshot
      builder at `server/src/core/session/snapshot.rs`**. Rationale +
      handler location:
      _<implementation prompt fills in>_
- [ ] **Path B -- Delete the C2S message from the protocol** (server's
      automatic snapshot at reconnect is the only path). Rationale:
      _<implementation prompt fills in>_

---

## Acceptance Criteria

All criteria are independently checkable. Most are GIVEN/WHEN/THEN.

- [ ] **AC1 -- Per-orphan decisions recorded**: GIVEN the umbrella-vs-
      split decision, WHEN the relevant story file(s) are read at the
      decision commit, THEN every orphan has exactly one path checked
      (with rationale text under it; per-row `_<implementation prompt
      fills in>_` placeholders replaced). The decision-recording
      commit precedes any code change.

- [ ] **AC2 -- Path A drains land with single-drainer discipline**:
      GIVEN the chosen path for each Path A orphan, WHEN the
      implementation commit set is reviewed, THEN exactly one
      production-code `MessageReceiver<T>` (or `MessageSender<T>` for
      the C2S handler) drain exists for each Path A orphan; no
      second drainer is introduced. ADR-008 binding.

- [ ] **AC3 -- Path B deletions are atomic across protocol + GDD +
      senders**: GIVEN the chosen path for each Path B orphan, WHEN
      the implementation commit set is reviewed, THEN: (a) the message
      type is removed from `shared/src/protocol.rs`; (b) the channel
      binding is removed; (c) any orphaned senders are removed;
      (d) `design/gdd/network-protocol.md` Table A is updated;
      (e) test files referencing the deleted type are updated or
      removed. The diff must show all five sub-changes in the same
      commit (or the same commit series with a clear ordering).

- [ ] **AC4 -- `S13-PROTO-INVARIANT-001` test flips to PASS**: GIVEN
      the implementation commit set, WHEN `cargo test --workspace
      --tests --no-fail-fast` is run, THEN
      `tests/invariants/protocol_completeness_test.rs` passes (or
      passes with a documented allowlist where each allowlist entry
      has an inline rationale + follow-on story reference for any
      orphan deferred under "Per-Orphan Decisions" Path C).

- [ ] **AC5 -- Integration tests cover at least one Path A drain
      per cluster**: GIVEN the Path A set chosen, WHEN integration
      tests are listed, THEN at least one new integration test
      asserts each newly added drain is invoked when its
      corresponding S2C/C2S message is sent. The tests follow the
      `tests/integration/session/result_acknowledgement_contract_test.rs`
      precedent. (One test per cluster suffices; e.g., one test for
      lifecycle S2C drains, one for prism / pool drains, one for
      heartbeat drain.)

- [ ] **AC6 -- No optimistic client-side authority introduced**:
      GIVEN the implementation diff, WHEN the diff is reviewed for
      any client-side mutation of authoritative state outside the
      shared phase sink, snapshot drainers, and S2C consumers, THEN
      no such mutation is present. ADR-002 binding. *Evidence*: text
      search for "no optimistic" in the evidence document.

- [ ] **AC7 -- No channel-binding changes for retained messages**:
      GIVEN the implementation diff in `shared/src/protocol.rs`,
      WHEN channel bindings (reliable vs unreliable) are inspected
      for any retained message, THEN no channel binding has changed.
      ADR-008 binding.

- [ ] **AC8 -- Workspace test count and ignored count behave
      predictably**: GIVEN `cargo test --workspace --tests
      --no-fail-fast` at the implementation commit, WHEN compared to
      the post-`S13-PROTO-INVARIANT-001` baseline, THEN the
      `protocol_completeness_test` reports PASS (no new `#[ignore]`
      markers introduced). The 5 Sprint 11 retained Cluster B
      `#[ignore]` tests remain unchanged in count unless Sprint 12
      Must Have rows have already retired them (in which case the
      delta is documented).

- [ ] **AC9 -- If split chosen, per-message follow-on stories
      authored**: GIVEN the split decision, WHEN
      `production/epics/lightyear-protocol-verification/` is listed,
      THEN one follow-on story per split orphan exists with the
      no-claim banner, evidence-path conventions, and decision-first
      discipline inherited from this story. This umbrella story
      closes as the producer-decision-record artefact.

- [ ] **AC10 -- Sprint 12 disposition preserved**: GIVEN the
      implementation commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, `production/stage.txt`,
      and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
      none of them are modified under this story. Sprint 12
      activation disposition is preserved. Stage remains `Polish`.
      Sprint 11 disposition (`closed-with-conditions`) is unchanged.
      Sprint 10 disposition (`closed-with-conditions`) is unchanged.

- [ ] **AC11 -- Evidence document slot reserved**: GIVEN this story
      file, WHEN the evidence-doc path is checked, THEN a slot is
      reserved at
      `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
      (umbrella) or per-split-story (split). Authoring of the
      evidence file(s) is deferred to the implementation prompt(s).

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `shared/src/protocol.rs` | Path B deletions (per-orphan); channel binding line removal. No additions. Possibly a `#[allow(dead_code)]` removal where applicable. |
| `client/src/network/mod.rs` | New `MessageReceiver<S2C*>` drains for Path A S2C orphans whose target home is the network layer (e.g., `S2CHeartbeat`). |
| `client/src/presentation/mod.rs` | New drains for Path A S2C orphans whose target home is presentation (e.g., `S2CPrismRespawned`, `S2CPrismRewardDropped`, `S2CPoolUpdate`). |
| `client/src/ui/lobby.rs` (or session UI module) | New drains for Path A S2C orphans whose target home is session UI (e.g., `S2COpponentDisconnected`, `S2COpponentReconnected`, `S2CSessionCancelled`). |
| `server/src/network/mod.rs` (or `server/src/core/session/`) | New `MessageReceiver<C2SRequestSnapshot>` handler if Path A chosen for the C2S orphan; reuses the snapshot builder at `server/src/core/session/snapshot.rs`. Removes the TODO at `server/src/main.rs:138-143`. |
| `tests/integration/network/*.rs` or `tests/integration/session/*.rs` | New integration tests asserting each Path A drain is invoked when its corresponding S2C/C2S message is sent. |
| `design/gdd/network-protocol.md` Table A | Updated for any Path B deletions (delete row + add deletion note). |
| `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md` | NEW evidence document per AC11. |
| This story file (decision-recording commit) | "Producer Decision" + "Per-Orphan Decisions" sections updated. **Commit precedes code change.** |
| Per-split follow-on stories (split only) | New story files per split orphan, inheriting from this umbrella. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code changes.
- **`liv-bevy-lightyear`** -- mandatory for all protocol / network
  changes. Lightyear 0.26 API drain wiring uses
  `MessageReceiver<T>` system params; the implementing worker must
  cross-reference Lightyear 0.26 release notes and
  `docs/engine-reference/bevy/VERSION.md` before adding new drains
  or removing channel bindings.

---

## Evidence Path

`production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Per-orphan disposition + rationale (transcribed from this story
  file's "Per-Orphan Decisions" section).
- Diff summary per orphan (file paths + line counts).
- Pre/post `cargo test --workspace --tests --no-fail-fast` pass +
  ignored counts.
- Pre/post `tests/invariants/protocol_completeness_test.rs` output
  (FAIL with 9 named orphans pre-impl; PASS post-impl).
- New integration-test pass evidence per cluster.
- No-claim restatement (verbatim from this story file's "Status /
  No-Claim Banner" section), including the explicit "no optimistic
  client-side authority" line.
- Cross-link to
  `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §3 DC-1 and §4 Lane A.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast 2>&1 | grep -iE "protocol_completeness|s2c|c2s_request"`
  (verifies the new drains and the invariant test all report PASS)
- `git log --oneline -- shared/src/protocol.rs client/src/ server/src/ tests/integration/`
  (verifies decision-recording commit precedes code-change commits)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'client/src/**' 'server/src/**' 'tests/**'`
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Authoring `tests/invariants/protocol_completeness_test.rs`**
  itself. That test lands under `S13-PROTO-INVARIANT-001` (Story 007).
  This story's responsibility is to make that test PASS (or pass with
  documented allowlists).
- **Adding new C2S / S2C messages** beyond the 9 named orphans.
- **Modifying channel bindings (reliable vs unreliable)** for any
  retained message. ADR-008 binding.
- **Channel infrastructure changes** (new channels, channel
  prioritisation, channel encryption).
- **Plugin registration invariant** (DC-2). Scoped to Sprint 14
  Nice-to-Have (`S13-PLUGIN-REGISTRATION-INVARIANT-001`).
- **`#[ignore]` / `#[should_panic]` attribute-drift invariant**
  (DC-15). Scoped to Sprint 14 Nice-to-Have.
- **Late-message dedupe** (DC-6). Scoped to Sprint 13 candidate
  `S13-LATE-MSG-DEDUPE-001` (Story 020 in `playable-client`).
- **Sprint 13 activation**. No `production/sprint-status.yaml` /
  `production/stage.txt` / `production/sprints/sprint-12.md` /
  `production/sprints/sprint-13.md` modification under this story.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  or any carried Sprint 10 / Sprint 11 / Sprint 12 condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Touches `shared/src/protocol.rs`, `client/src/network/`,
  `client/src/presentation/`, `client/src/ui/lobby.rs`,
  `server/src/network/`, `server/src/core/session/`,
  `tests/integration/network/`, `tests/integration/session/`**, and
  `design/gdd/network-protocol.md`. Sprint 12 Must Have rows touch:
  - Story 019 (hand-ui): no source files touched (runtime evidence).
  - Story 012 (HUD snapshot bridge): `tests/integration/board_rendering/`
    + optionally `client/src/` HUD code -- disjoint.
  - Story 013 (lobby ConfirmClass intent chain): `client/src/ui/lobby.rs`
    + `tests/integration/playable_client/` -- **POTENTIAL CONFLICT**
    if `S2CSessionCancelled` Path A drains land in lobby.rs in
    parallel with Sprint 12 story 013's lobby edits. Mitigation: this
    Sprint 13 story MUST NOT run in parallel with Sprint 12 story 013
    `/dev-story`; either Sprint 12 closes first OR the
    `S2CSessionCancelled` drain lives outside lobby.rs (e.g., session
    UI module or presentation module). The implementation prompt
    chooses the drain location to avoid lobby.rs collision if Sprint
    12 story 013 is still active.
  - Story 014 (cooccupancy panic guard): `client/src/presentation/board_rendering.rs`
    + `tests/unit/board_rendering/` -- disjoint.
  - Story 015 (fixture D residuals): `tests/integration/board_rendering/`
    + `tests/unit/shop_auction_ui/` + potentially
    `client/src/ui/shop_auction_ui/` -- disjoint.
- **No Sprint 12 invasion**: This story's implementation prompt
  (Sprint 13 candidate) MUST NOT land before Sprint 12 close-out
  unless the producer explicitly authorises a pull-forward via a
  separate prompt.
- **Coordinate with `S13-PROTO-INVARIANT-001` (Story 007 in this
  epic)**: Either land both stories in the same Sprint 13 wave (this
  story flips the invariant test to PASS), or land Story 007 first
  with its test `#[ignore]`-gated, then land this story with the
  ignore removed.
- **Coordinate with `S13-LATE-MSG-DEDUPE-001` (playable-client Story
  020)**: drains added here should follow the dedupe pattern that
  Story 020 establishes; if Story 020 lands later, this story's
  drains use the existing `C2SAcknowledgeResult` idempotency
  precedent until Story 020 generalises it.
- **No shared-status writer overlap**: Per the 2026-05-13 override,
  only one shared-status writer is active at a time.
  `production/sprint-status.yaml` is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out (separate prompt).
2. Sprint 13 is planned via `/sprint-plan sprint-13` (separate prompt).
3. This story passes `/story-readiness` (separate prompt).
4. Sprint 13 `/qa-plan sprint` is authored (separate prompt).
5. `/dev-story story-008-protocol-orphan-drain.md` is dispatched
   (separate prompt).

Expected implementation flow:

1. **Wave 1 -- Umbrella-vs-split decision**: the implementation prompt
   records the umbrella-vs-split decision in this story file's
   "Producer Decision" section. **This commit precedes any code
   change.**
2. **Wave 2 -- Per-orphan decisions**: each of the 9 orphan
   dispositions recorded with rationale.
3. **Wave 3 -- Code changes per chosen disposition**: Path A drains
   wired with single-drainer discipline (ADR-008); Path B deletions
   removed atomically across protocol + GDD + senders.
4. **Wave 4 -- Integration tests**: one new integration test per Path
   A cluster, following
   `tests/integration/session/result_acknowledgement_contract_test.rs`
   precedent.
5. **Wave 5 -- Invariant test flip**: re-run
   `tests/invariants/protocol_completeness_test.rs` (from Story 007);
   confirm PASS or PASS-WITH-DOCUMENTED-ALLOWLIST.
6. **Wave 6 -- Evidence**: populate
   `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Path B deletion of an "intentional latent capability" message regrets later | Medium | Medium | Per-orphan rationale required; deletion candidates favour messages with no observable consumer plan. `S2CSangMepriseReveal` is the highest-risk deletion candidate -- producer should default to Path A (drain with TODO) or Path C (defer to per-message split). |
| Lobby.rs collision with Sprint 12 story 013 | Medium | High | Coordinate dispatch: do not run this story's `/dev-story` while Sprint 12 story 013 is `in-progress`. Implementation prompt chooses drain locations to avoid lobby.rs if Sprint 12 story 013 is still active. |
| Path A drain accidentally mutates authoritative state (e.g., updates `ClientState` directly) | Low | High | AC6 + ADR-002 reviewer check; pattern templates from existing drains (e.g., `S2CClassLocked` at `client/src/ui/lobby.rs:326-335`) cited in evidence doc. |
| Integration tests for new drains are flaky | Low | Medium | Use deterministic seeds; follow `tests/integration/session/result_acknowledgement_contract_test.rs:91-96` precedent for idempotency assertions. |
| `S13-PROTO-INVARIANT-001` (Story 007) lands after this story and reveals more orphans than expected | Medium | Low | This story explicitly allows additional orphans discovered at implementation time; rationale records them. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate; this story stays `Draft` until activation. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the implementation
prompt, not for the worker:

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation; this story is referenced from the active row,
  OR the row is held with a written blocker.
- Sprint 12 close-out has landed (`sprint_12_closeout:` block exists
  in `production/sprint-status.yaml`).
- Sprint 12 story 013 (lobby ConfirmClass intent chain) is closed or
  the implementation prompt is dispatched after Sprint 12 close-out.
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for the Protocol Orphan Drain / Delete umbrella covering
  8 S2C + 1 C2S orphans from PROMPT 803 §4 Lane A. Sprint 12 is
  `active` (PROMPT 798) and is not modified by this authoring run.
  No code changes, no smoke / gate / QA / `/dev-story` / `/story-done` /
  `/story-readiness` / `/qa-plan` run. Source-of-truth at authoring:
  `origin/main@b5eef0d`. Worker branch:
  `work/s13-runtime-hardening-story-authoring`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.
