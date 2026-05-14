# Story 008: S13-PROTO-ORPHAN-DRAIN-001 -- Drain or Delete 8 S2C + 1 C2S Protocol Orphans

> **Epic**: Lightyear Protocol & Verification Spike
> **Story ID**: S13-PROTO-ORPHAN-DRAIN-001
> **Status**: **Done** (PROMPT 856 `/story-done` closure, 2026-05-14) --
> verdict **PASS-WITH-ALLOWLIST** (3-row documented allowlist: Sang
> Méprise reveal drain deferred per Path C, C2SClassChoice
> disposition deferred, S2COpponentDisconnected send-site out-of-scope
> per story disposition); each allowlist entry has inline rationale +
> follow-on story reference inside
> `tests/invariants/protocol_completeness_test.rs`. Flipped from
> `Draft` (Sprint 13 candidate) to `Done` after worker (PROMPT 852
> `9c0923f3f83652af27dd67fba9ceb8c155b3fd12` on
> `work/s13-protocol-orphan-drain` from base `origin/main@25573e6`)
> + integration (PROMPT 855 merge commit
> `ecec3760af02401902e5959da38dad1bba4f2421` on `origin/main`, merge
> of worker tip `9c0923f` into prior `origin/main@3199c01`) reached
> the integration tip. PROMPT 856 source-of-truth: `origin/main@ecec376`.
> **Layer**: Foundation / Protocol
> **Type**: Decision-first per-message (drain vs delete-with-rationale) +
> Integration (drain wiring) + Config/Data (protocol deletion if chosen)
> **Sprint**: Sprint 13 (activated at `origin/main@b5eef0d->25573e6`;
> Must Have row `S13-PROTO-ORPHAN-DRAIN-001`)
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

- [x] **Umbrella (this story)**: keep all 9 orphan dispositions in
      this story; close each cluster (8 S2C + 1 C2S) under
      `S13-PROTO-ORPHAN-DRAIN-001`. Rationale (PROMPT 821, 2026-05-14
      producer decision):
      The 9-orphan set is small, shares one decision shape
      (drain-or-delete-with-rationale), and the only orphan with an
      open ADR dependency (`S2CSangMepriseReveal`) is dispositioned
      below as Path C (defer to a per-message split story) without
      blocking the other eight. Batching the remaining 8 dispositions
      under a single umbrella lets the `S13-PROTO-INVARIANT-001`
      invariant test (Story 007) flip to PASS in one Sprint 13 wave
      with at most a single documented allowlist entry for the Sang
      Méprise reveal. ADR-002 / ADR-008 / ADR-011 / ADR-012 are
      preserved verbatim by every per-orphan decision below: no
      optimistic client-side authority is introduced; no channel
      bindings change; Path A drains for messages already produced on
      the server (`S2COpponentReconnected`, `S2CSessionCancelled`,
      `S2CPrismRespawned`, `S2CPrismRewardDropped`) consume those
      existing sends without altering the reconnect / SessionReady
      ordering; Path B deletions target three messages with neither
      live sender nor consumer plan and remove their GDD entries
      atomically.

- [ ] **Split into per-message stories**: author up to 9 follow-on
      story files (one per orphan); close this umbrella story as the
      producer-decision-record artefact. Each split story inherits
      the no-claim banner, evidence-path conventions, and decision-
      first discipline from this story. Rationale:
      _Not chosen — see umbrella rationale above. The single per-
      message split that IS required is the `S2CSangMepriseReveal`
      Path C deferral; that one-row split is recorded under
      "Per-Orphan Decisions" without converting the entire umbrella
      to a split._

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
      _Not chosen._
- [x] **Path B -- Delete**. Rationale (PROMPT 821):
      No producer exists anywhere in the workspace — `S2CHeartbeat`
      is defined as an empty struct at `shared/src/protocol.rs:817`
      and registered at `shared/src/protocol.rs:107`, but no
      `MessageSender<S2CHeartbeat>` call site exists in `server/src/`
      or anywhere else (verified by grep at PROMPT 821). The GDD's
      Rule 8 disconnect-detection contract specifies
      `C2SHeartbeat` (client → server, unreliable channel, `5000ms`
      cadence) as the sole application-layer liveness probe — see
      `design/gdd/network-protocol.md` Rule 8 and the C2S table
      entry for `C2SHeartbeat`. There is no documented server →
      client heartbeat in any rule, table, or ADR; `S2CHeartbeat`
      appears to be a vestigial mirror artefact. ADR-008 compliance
      preserved: the `UnreliableChannel` binding at
      `shared/src/protocol.rs:107` is removed in the same commit set
      that removes the type definition at `shared/src/protocol.rs:817`
      (channel-binding removal for a deleted message is the only
      kind of channel change allowed under ADR-008). Atomic-deletion
      pre-condition satisfied: no GDD Table A row exists to remove;
      no sender call site exists to remove. ADR-002 / ADR-011 /
      ADR-012 unaffected (S2CHeartbeat has no role in the authority
      model, reconnect snapshot, or SessionReady ordering).

#### `S2COpponentDisconnected` -- `shared/src/protocol.rs:92`

- [x] **Path A -- Drain**. Rationale + drain location (PROMPT 821):
      GDD `design/gdd/network-protocol.md` Rule 8 explicitly mandates
      this broadcast: *"The protocol broadcasts
      `S2COpponentDisconnected { grace_remaining_ms }` on
      `OnDisconnected` so the remaining player sees the countdown."*
      The "Zero idle time" pillar row in the Player Fantasy table
      lists `S2COpponentDisconnected` as a load-bearing rule. The
      type definition (`shared/src/protocol.rs:593`) carries the
      `grace_remaining_ms: u32` field documented in Rule 8. The
      server-side sender is currently absent from the workspace
      (verified by grep at PROMPT 821: no `MessageSender<S2COpponent
      Disconnected>` call site in `server/src/`); landing the server
      sender is out-of-scope for this story (separate Sprint 13 or
      14 row, not yet authored — flag as a follow-on). Path B
      (delete) would contradict GDD Rule 8 and is forbidden. Drain
      location: **session UI surface module** outside
      `client/src/ui/lobby.rs` to avoid the Sprint 12 story 013
      lobby.rs collision risk flagged in this story's "Risks" table;
      candidate home is `client/src/presentation/mod.rs` next to the
      shared phase sink (`apply_phase_changed_message` ~`:163-222`)
      or a new disconnect-modal module under `client/src/ui/`. The
      drain is read-only (renders a "Opponent disconnected, grace
      remaining: N ms" indicator); never mutates authoritative state
      (ADR-002 binding). Channel binding unchanged
      (`ReliableChannel` per `shared/src/protocol.rs:92`; ADR-008
      binding). Single-drainer rule per ADR-008.
- [ ] **Path B -- Delete**. Rationale:
      _Not chosen — would contradict GDD `network-protocol.md` Rule
      8 ("The protocol broadcasts `S2COpponentDisconnected
      { grace_remaining_ms }` on `OnDisconnected`...") and remove a
      load-bearing rule for the "Zero idle time" pillar._

#### `S2COpponentReconnected` -- `shared/src/protocol.rs:93`

- [x] **Path A -- Drain**. Rationale + drain location (PROMPT 821):
      Live producer already exists in the reconnect flow at
      `server/src/core/session/reconnect.rs:54-58, 231, 502-513`
      (sent through the `send_reconnect_dispatches` /
      `send_deferred_message` path; `S2COpponentReconnected` is one
      of the messages broadcast to the *remaining* player when their
      opponent re-establishes a transport identity). ADR-011 §"Verified
      Implementation Checklist" item 8 + item 9 reference this
      send. Pairs symmetrically with `S2COpponentDisconnected`
      (above) for the "Zero idle time" pillar (`design/gdd/network-
      protocol.md` Player Fantasy table row 4). Path B (delete) is
      forbidden because deletion would silently drop the live
      reconnect broadcast and break ADR-011's reconnect-notification
      contract. Drain location: **same session UI module** chosen
      for `S2COpponentDisconnected`, paired in a single drain system
      that clears the "Opponent disconnected" indicator on receipt.
      Drain is read-only (UI state only); never mutates authoritative
      state (ADR-002 binding). Channel binding unchanged
      (`ReliableChannel` per `shared/src/protocol.rs:93`; ADR-008
      binding). Single-drainer rule per ADR-008.
- [ ] **Path B -- Delete**. Rationale:
      _Not chosen — live producer exists at
      `server/src/core/session/reconnect.rs:54-58, 231, 502-513`;
      deletion would silently drop the reconnect broadcast and
      contradict ADR-011's reconnect-notification contract._

#### `S2CPoolUpdate` -- `shared/src/protocol.rs:85`

- [ ] **Path A -- Drain**. Rationale + drain location:
      _Not chosen._
- [x] **Path B -- Delete**. Rationale (PROMPT 821):
      No producer exists in the workspace — type defined at
      `shared/src/protocol.rs:548`, channel-registered at
      `shared/src/protocol.rs:85`, but no `MessageSender<S2CPoolUpdate>`
      call site in `server/src/` (verified by grep at PROMPT 821).
      No GDD rule, ADR, or table entry requires it: the card pool
      lives entirely on the server (per `design/gdd/network-protocol.md`
      Rule 1 + Rule 6 — private per-player state is unicast on
      demand, public pool state is not broadcast); no client-facing
      pool view is in scope for the friend-game milestone or any
      named Sprint 13 / Sprint 14 row. The hypothesised "ladder-
      telemetry" use case in this story's "Context" table is
      advisory speculation, not a roadmap commitment. Path A drain
      to a no-op TODO would create a never-fired receiver and
      complicate the `S13-PROTO-INVARIANT-001` allowlist; cleaner
      to delete and re-add the message if a future feature row
      authors a pool-broadcast story. Atomic-deletion: type
      definition at `shared/src/protocol.rs:548` removed; channel
      binding at `shared/src/protocol.rs:85` removed; no sender to
      remove (none exists); GDD `network-protocol.md` Table A has no
      row for `S2CPoolUpdate` (verified at PROMPT 821 by reading the
      GDD's C2S + S2C inventory tables), so the GDD-update
      sub-condition is satisfied by a no-op. ADR-002 / ADR-008 /
      ADR-011 / ADR-012 unaffected.

#### `S2CPrismRespawned` -- `shared/src/protocol.rs:82`

- [x] **Path A -- Drain**. Rationale + drain location (PROMPT 821):
      Live producer in `server/src/feature/prism/system.rs:513-531`
      (`stage_prism_respawned`: broadcasts `S2CPrismRespawned
      { player_id }` on `NetworkTarget::All` over `ReliableChannel`).
      The prism system is an active feature (ADR-016, prism system
      architecture) — Path B (delete) would silently drop a live
      server broadcast and orphan the sender, contradicting ADR-008's
      no-orphaned-sender requirement (and AC3 of this story). Drain
      location: `client/src/presentation/mod.rs` next to the shared
      phase sink at ~`:163-222`, since prism respawn is a
      presentation-layer concern (rendering the prism entity back
      into the lane visualisation). Drain is read-only — it updates
      a presentation-side `PrismVisualState` and never mutates
      authoritative state (ADR-002 binding). Channel binding
      unchanged (`ReliableChannel`; ADR-008 binding). Single-drainer
      rule per ADR-008.
- [ ] **Path B -- Delete**. Rationale:
      _Not chosen — live producer in
      `server/src/feature/prism/system.rs:513-531`; deletion would
      orphan the sender (forbidden by ADR-008 / AC3) and break
      ADR-016 (prism system architecture)._

#### `S2CPrismRewardDropped` -- `shared/src/protocol.rs:81`

- [x] **Path A -- Drain**. Rationale + drain location (PROMPT 821):
      Live producer in `server/src/feature/prism/system.rs:467-497`
      (`stage_reward_dropped`: emits `S2CPrismRewardDropped
      { player_id, lane }` to `NetworkTarget::All` on
      `ReliableChannel`) plus the reconnect-deferred sender at
      `server/src/core/session/reconnect.rs:755-779` that replays
      pending drops on session restore. Tied to ADR-016 (prism
      system) and the economy/reward pillar; Path B (delete) would
      silently break the reward broadcast and the reconnect replay.
      Drain location: `client/src/presentation/mod.rs` (same
      presentation-layer home as `S2CPrismRespawned`), paired in a
      single prism-event drain system to keep the prism presentation
      surface coherent. Drain is read-only — it updates a
      presentation-side reward indicator and never mutates
      authoritative gold / economy state (ADR-002 binding: server
      remains sole authority on gold via `S2CGoldUpdate` and
      `S2CGoldBroadcast`, which are already drained elsewhere).
      Channel binding unchanged (`ReliableChannel`; ADR-008 binding).
      Single-drainer rule per ADR-008.
- [ ] **Path B -- Delete**. Rationale:
      _Not chosen — live producer set in
      `server/src/feature/prism/system.rs:467-497` and the reconnect
      replay at `server/src/core/session/reconnect.rs:755-779`;
      deletion would orphan both senders (forbidden by ADR-008 /
      AC3) and break the prism reward broadcast contract._

#### `S2CSangMepriseReveal` -- `shared/src/protocol.rs:105`

- [ ] **Path A -- Drain (with TODO)**. Rationale + drain location:
      _Not chosen — see Path C._
- [ ] **Path B -- Delete**. Rationale:
      _Not chosen — live producer exists in
      `server/src/core/session/reconnect.rs:54, 479-490` and the
      builder at `server/src/core/session/reconnect.rs:998-1005`
      (`sang_meprise_reveal_message`). Deletion would silently break
      the reconnect path and the reveal contract referenced in
      ADR-011 §"Verified Implementation Checklist" item 14 (Sang
      Méprise reveal restore is included in snapshot and re-send
      flow). Forbidden by ADR-011 binding._
- [x] **Path C -- Defer to a per-message split story**. Rationale
      (PROMPT 821):
      `.claude/docs/technical-preferences.md` lists "Sang Méprise
      reveal mechanism" among the **Pending ADRs needed**, and the
      client-side reveal-rendering contract depends on that ADR's
      authority decisions (when does the reveal animation play,
      what UI surface owns it, what visibility rules guard the
      reveal payload). Choosing Path A (no-op log drain) now risks
      having to rip out the drain and redo it once the reveal ADR
      lands, which costs more than the cost of keeping the
      `S13-PROTO-INVARIANT-001` allowlist single-row. Path B is
      forbidden (live producer; see above). Path C therefore: this
      umbrella story records the deferral; a separate Sprint 14
      candidate row (proposed identifier
      `S14-PROTO-SANG-MEPRISE-DRAIN-001`) authors the consumer-side
      drain *after* the Sang Méprise reveal-mechanism ADR is
      Accepted. The `S13-PROTO-INVARIANT-001` invariant test (Story
      007) lands with `S2CSangMepriseReveal` on its allowlist; the
      allowlist entry's rationale cites this umbrella decision and
      the pending ADR (single-row exception per the umbrella
      rationale). PROMPT 821 does NOT author the Sprint 14 candidate
      story (paperwork-only run; story-file authoring is a separate
      paperwork prompt).

#### `S2CSessionCancelled` -- `shared/src/protocol.rs:103`

- [x] **Path A -- Drain**. Rationale + drain location (PROMPT 821):
      Live producer set: `server/src/core/session/system.rs:2075`
      builds the message and
      `server/src/core/session/system.rs:2143` sends it via
      `MessageSender::send::<S2CSessionCancelled, ReliableChannel>`;
      `server/src/core/session/state.rs:126, 234, 240, 252` queue
      it through the session-state machinery; and
      `server/src/core/session/reconnect.rs:581-593`
      (`send_deferred_message`) replays it for reconnecting clients.
      Session-cancellation semantics are owned by
      `design/gdd/game-session-system.md`; ADR-012 (SessionReady
      Delivery) requires that lifecycle S2C messages integrate with
      the SessionReady observer's flush ordering without introducing
      new ordering — this story's drain reads the message after the
      observer flush and never re-orders. Path B (delete) is
      forbidden because deletion would silently drop the live cancel
      broadcast and the deferred replay. Drain location: **session
      UI surface module outside `client/src/ui/lobby.rs`** to avoid
      the Sprint 12 story 013 lobby.rs collision risk flagged in
      this story's "Risks" table and "Dependency Notes Against
      Sprint 12 Active Scope" — candidate home is
      `client/src/presentation/mod.rs` (next to phase sink) or a
      dedicated session-lifecycle module under `client/src/network/`.
      Drain renders a graceful-exit indicator and routes the client
      state machine to a post-session screen; never mutates server-
      authoritative state (ADR-002 binding). Channel binding
      unchanged (`ReliableChannel`; ADR-008 binding). Single-drainer
      rule per ADR-008.
- [ ] **Path B -- Delete**. Rationale:
      _Not chosen — live producer set:
      `server/src/core/session/system.rs:2075, 2143`,
      `server/src/core/session/state.rs:126, 234, 240, 252`, and
      `server/src/core/session/reconnect.rs:581-593` (deferred
      replay). Deletion would orphan all of them (forbidden by
      ADR-008 / AC3) and break the session-cancellation broadcast
      contract owned by `design/gdd/game-session-system.md`._

### C2S orphan (1 row)

#### `C2SRequestSnapshot` -- `shared/src/protocol.rs:71` + `server/src/main.rs:138-143` TODO

- [x] **Path A -- Add server handler that reuses the reconnect snapshot
      builder at `server/src/core/session/snapshot.rs`**. Rationale +
      handler location (PROMPT 821):
      `design/gdd/network-protocol.md` Table A explicitly specifies
      `C2SRequestSnapshot { }` with payload "Client-initiated desync
      recovery. Server responds with `S2CGameSnapshot` unicast (same
      path as reconnect). Rate-limited: server ignores if a snapshot
      was sent to this client within the last `snapshot_cooldown_ms`
      (default 5000ms). This is a recovery tool — clients must not
      poll for snapshots. Resolves OQ-BR-06." The GDD distinguishes
      this client-initiated recovery path from the server-driven
      reconnect snapshot (ADR-011 §"Snapshot-first reconnect
      sequence"): reconnect snapshot fires automatically on
      transport identity recovery; `C2SRequestSnapshot` fires on
      client-side stale-data heuristics (e.g., a Sprint 13 candidate
      Story 021 `S13-CONN-LOST-UX-001` "request fresh state" button
      surfacing this path). Path B (delete) would contradict GDD
      Table A and break OQ-BR-06's resolution; rejected. Handler
      location: `server/src/core/session/` — new system
      `handle_request_snapshot` that reads
      `MessageReceiver<C2SRequestSnapshot>`, calls the existing
      snapshot builder at `server/src/core/session/snapshot.rs` (no
      new construction path; ADR-011 binding), enforces the
      `snapshot_cooldown_ms` rate-limit per the GDD, and unicasts
      `S2CGameSnapshot` via `NetworkTarget::Single(PeerId)` (ADR-011
      reconnect-snapshot pattern). The TODO at
      `server/src/main.rs:138-143` is removed in the same commit
      set. ADR-002 binding (server remains authoritative on snapshot
      contents; the C2S message is advisory only — server can
      ignore via rate-limit). ADR-008 channel binding unchanged
      (`ReliableChannel` per `shared/src/protocol.rs:71`). ADR-011
      reuse: the existing snapshot builder is the only authoring
      path; no new snapshot construction lands. ADR-012 unaffected
      (SessionReady ordering not involved — this handler runs in
      `IN_GAME` phases only per GDD Table A Valid Phase(s)).
- [ ] **Path B -- Delete the C2S message from the protocol** (server's
      automatic snapshot at reconnect is the only path). Rationale:
      _Not chosen — GDD Table A entry for `C2SRequestSnapshot`
      explicitly distinguishes it from the reconnect-driven
      snapshot path (the reconnect path is server-initiated on
      transport-identity recovery; `C2SRequestSnapshot` is client-
      initiated on stale-data heuristics, e.g., the Sprint 13
      candidate Story 021 conn-lost UX). Deletion would close OQ-BR-
      06 by amputation, not resolution, and would remove the only
      named recovery path the client has between `OnDisconnected`
      and full reconnect. Forbidden by GDD Table A binding._

---

## Acceptance Criteria

All criteria are independently checkable. Most are GIVEN/WHEN/THEN.

- [x] **AC1 -- Per-orphan decisions recorded**: GIVEN the umbrella-vs-
      split decision, WHEN the relevant story file(s) are read at the
      decision commit, THEN every orphan has exactly one path checked
      (with rationale text under it; per-row `_<implementation prompt
      fills in>_` placeholders replaced). The decision-recording
      commit precedes any code change.
      **PASS** (PROMPT 856 verification): Umbrella path chosen per
      PROMPT 821 producer decision (line 267, `[x]` Umbrella with
      multi-paragraph rationale). All 9 named orphans + the
      additional `C2SClassChoice` row have exactly one path checked
      in this story's "Per-Orphan Decisions" section: 5×Path A
      (`S2COpponentDisconnected`, `S2COpponentReconnected`,
      `S2CPrismRespawned`, `S2CPrismRewardDropped`,
      `S2CSessionCancelled`) + 2×Path B (`S2CHeartbeat`,
      `S2CPoolUpdate`) + 1×Path C (`S2CSangMepriseReveal`, deferred
      to `S14-PROTO-SANG-MEPRISE-DRAIN-001`) + 1×Path A
      (`C2SRequestSnapshot` server handler). PROMPT 821 paperwork
      commit (decision-recording) precedes PROMPT 852 worker commit
      `9c0923f` (implementation) on `work/s13-protocol-orphan-drain`.

- [x] **AC2 -- Path A drains land with single-drainer discipline**:
      GIVEN the chosen path for each Path A orphan, WHEN the
      implementation commit set is reviewed, THEN exactly one
      production-code `MessageReceiver<T>` (or `MessageSender<T>` for
      the C2S handler) drain exists for each Path A orphan; no
      second drainer is introduced. ADR-008 binding.
      **PASS** (PROMPT 856 verification): single-drainer source guards
      land in `tests/integration/presentation/protocol_orphan_drain_test.rs::{lifecycle_cluster_drains_are_registered_exactly_once_in_production, prism_cluster_drains_are_registered_exactly_once_in_production}`
      (NEW; 227 lines on `origin/main@ecec376` via integration merge)
      + `tests/integration/session/request_snapshot_handler_test.rs::handle_request_snapshot_is_sole_production_drain`
      (NEW; 154 lines on `origin/main@ecec376` via integration merge).
      All 5 Path A S2C drains land in
      `client/src/presentation/mod.rs::{drain_opponent_connection_messages, drain_prism_lifecycle_messages, drain_session_lifecycle_messages}`;
      the C2S handler lands in
      `server/src/core/session/snapshot_request.rs::handle_request_snapshot`.
      Evidence: `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
      §AC verification matrix row AC2 + targeted `cargo test`
      outputs (6/6 + 5/5 PASS recorded on PROMPT 852 worker tip).

- [x] **AC3 -- Path B deletions are atomic across protocol + GDD +
      senders**: GIVEN the chosen path for each Path B orphan, WHEN
      the implementation commit set is reviewed, THEN: (a) the message
      type is removed from `shared/src/protocol.rs`; (b) the channel
      binding is removed; (c) any orphaned senders are removed;
      (d) `design/gdd/network-protocol.md` Table A is updated;
      (e) test files referencing the deleted type are updated or
      removed. The diff must show all five sub-changes in the same
      commit (or the same commit series with a clear ordering).
      **PASS** (PROMPT 856 verification): Path B deletions for
      `S2CHeartbeat` + `S2CPoolUpdate` land atomically in PROMPT 852
      worker commit `9c0923f`: (a) type defs removed from
      `shared/src/protocol.rs` (-17 lines net in protocol.rs per the
      ecec376 integration diff); (b) channel-binding `register_s2c::<T>`
      lines removed; (c) no senders existed for either type
      (verified at PROMPT 821 by grep, recorded in this story's
      per-orphan rationales); (d) `design/gdd/network-protocol.md`
      updated (8 lines net change; `S2CPoolUpdate` Table A row +
      §VI + §VIII + §IX cross-references updated; `S2CHeartbeat` had
      no GDD row to remove per the story's PROMPT 821 rationale);
      (e) `tests/integration/network/e2e_websocket_test.rs` updated
      to drop the S2CHeartbeat unreliable assertions (-41 / +0
      block per integration diff). All sub-changes are in worker
      commit `9c0923f` and remain reachable on
      `origin/main@ecec376`.

- [x] **AC4 -- `S13-PROTO-INVARIANT-001` test flips to PASS**: GIVEN
      the implementation commit set, WHEN `cargo test --workspace
      --tests --no-fail-fast` is run, THEN
      `tests/invariants/protocol_completeness_test.rs` passes (or
      passes with a documented allowlist where each allowlist entry
      has an inline rationale + follow-on story reference for any
      orphan deferred under "Per-Orphan Decisions" Path C).
      **PASS-WITH-DOCUMENTED-ALLOWLIST** (PROMPT 856 verification):
      `tests/invariants/protocol_completeness_test.rs::protocol_completeness_assert_send_and_drain_sites`
      had its `#[ignore]` removed by PROMPT 852 worker commit
      `9c0923f`; the targeted `cargo test -p shared --test
      protocol_completeness_invariant` evidence in PROMPT 852's
      doc reports **2/2 PASS** (parser-smoke + send-and-drain
      asserter). The 3-row `ALLOWLIST` const lives inline at
      `tests/invariants/protocol_completeness_test.rs:296-345` on
      `origin/main@ecec376` and each entry names its follow-on:
      (i) `S2CSangMepriseReveal` / `MissingSide::Drain` (Path C
      deferral, follow-on `S14-PROTO-SANG-MEPRISE-DRAIN-001`);
      (ii) `C2SClassChoice` / `MissingSide::Send` (out-of-scope
      surfacing per PROMPT 845 invariant test discovery; follow-on
      `S14-PROTO-CLASSCHOICE-DISPOSITION-001`);
      (iii) `S2COpponentDisconnected` / `MissingSide::Send`
      (server-broadcast send-site explicitly out-of-scope per
      Story 008 per-orphan rationale; follow-on not yet authored).
      Verified by PROMPT 856 reading the integration tip
      `tests/invariants/protocol_completeness_test.rs` at
      `git show ecec376`.

- [x] **AC5 -- Integration tests cover at least one Path A drain
      per cluster**: GIVEN the Path A set chosen, WHEN integration
      tests are listed, THEN at least one new integration test
      asserts each newly added drain is invoked when its
      corresponding S2C/C2S message is sent. The tests follow the
      `tests/integration/session/result_acknowledgement_contract_test.rs`
      precedent. (One test per cluster suffices; e.g., one test for
      lifecycle S2C drains, one for prism / pool drains, one for
      heartbeat drain.)
      **PASS** (PROMPT 856 verification): Three Path A clusters
      each covered. Lifecycle cluster (`S2COpponentDisconnected`,
      `S2COpponentReconnected`, `S2CSessionCancelled`): covered by
      `tests/integration/presentation/protocol_orphan_drain_test.rs::{s2c_opponent_disconnect_and_reconnect_pair_apply_to_connection_view, s2c_session_cancelled_applies_to_session_lifecycle_view, lifecycle_cluster_drains_are_registered_exactly_once_in_production}`.
      Prism cluster (`S2CPrismRespawned`, `S2CPrismRewardDropped`):
      covered by `..::s2c_prism_respawned_and_reward_dropped_apply_to_lifecycle_view`
      + `..::prism_cluster_drains_are_registered_exactly_once_in_production`.
      Snapshot-request cluster (`C2SRequestSnapshot`): covered by
      `tests/integration/session/request_snapshot_handler_test.rs::{snapshot_request_cooldown_blocks_inside_window_and_releases_after_threshold, snapshot_request_cooldown_tracks_each_player_independently, game_session_plugin_installs_snapshot_request_cooldowns_resource, handle_request_snapshot_is_sole_production_drain}`.
      Both new test files are registered as `[[test]]` targets in
      `client/Cargo.toml` (+7 lines) and `server/Cargo.toml` (+6
      lines) per the PROMPT 852 integration diff at `ecec376`.

- [x] **AC6 -- No optimistic client-side authority introduced**:
      GIVEN the implementation diff, WHEN the diff is reviewed for
      any client-side mutation of authoritative state outside the
      shared phase sink, snapshot drainers, and S2C consumers, THEN
      no such mutation is present. ADR-002 binding. *Evidence*: text
      search for "no optimistic" in the evidence document.
      **PASS** (PROMPT 856 verification): The phrase
      "no optimistic client-side authority" is preserved verbatim
      in `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
      (No-Claim Banner restatement at lines 11-25 and AC6 row at
      line 115). The three new client-side resources
      (`OpponentConnectionView`, `PrismLifecycleView`,
      `SessionLifecycleView`) added in `client/src/state/mod.rs`
      are read-only presentation views; the new drain systems in
      `client/src/presentation/mod.rs` write only to those views
      and never mutate `CurrentClientPhase`,
      `ClientObjectiveIdentities`, `PlayerEconomyView`, or any
      other authoritative-mirror resource. Server-side
      `handle_request_snapshot` remains sole authority on snapshot
      contents (reuses `build_game_snapshot` per ADR-011 binding;
      the C2S message is advisory and rate-limited).

- [x] **AC7 -- No channel-binding changes for retained messages**:
      GIVEN the implementation diff in `shared/src/protocol.rs`,
      WHEN channel bindings (reliable vs unreliable) are inspected
      for any retained message, THEN no channel binding has changed.
      ADR-008 binding.
      **PASS** (PROMPT 856 verification): The PROMPT 852 worker
      commit `9c0923f` net change to `shared/src/protocol.rs` is
      17 lines and consists entirely of: (i) `S2CHeartbeat` type
      def + its `register_s2c::<S2CHeartbeat>` channel-binding line
      removed; (ii) `S2CPoolUpdate` type def + its
      `register_s2c::<S2CPoolUpdate>` channel-binding line removed.
      No `register_c2s::<T>` / `register_s2c::<T>` call for any
      retained message had its `ProtocolChannel::Reliable` /
      `ProtocolChannel::Unreliable` argument changed. Evidence
      doc AC7 row records the manual diff inspection;
      PROMPT 856 confirms the integration-tip `protocol.rs` matches.

- [x] **AC8 -- Workspace test count and ignored count behave
      predictably**: GIVEN `cargo test --workspace --tests
      --no-fail-fast` at the implementation commit, WHEN compared to
      the post-`S13-PROTO-INVARIANT-001` baseline, THEN the
      `protocol_completeness_test` reports PASS (no new `#[ignore]`
      markers introduced). The 5 Sprint 11 retained Cluster B
      `#[ignore]` tests remain unchanged in count unless Sprint 12
      Must Have rows have already retired them (in which case the
      delta is documented).
      **PASS-WITH-NARROW-EXCEPTION** (PROMPT 856 verification): the
      load-bearing assertion ("`protocol_completeness_test` reports
      PASS; no new `#[ignore]` markers introduced") is verified by
      reading `tests/invariants/protocol_completeness_test.rs` at
      `origin/main@ecec376`: the previously-ignored
      `protocol_completeness_assert_send_and_drain_sites` test has
      its `#[ignore]` REMOVED (a net `#[ignore]` count DECREASE,
      not an increase), and no new `#[ignore]` attribute was
      introduced anywhere in the PROMPT 852 worker diff. The
      comparison to the post-`S13-PROTO-INVARIANT-001` baseline is
      satisfied. The "5 Sprint 11 retained Cluster B `#[ignore]`
      tests remain unchanged in count" sub-clause is not
      re-verified by PROMPT 856 (no full-workspace test run per
      Cargo policy + QA-plan no-full-workspace-tests-by-default
      policy); PROMPT 852's evidence doc explicitly defers that
      sub-clause to sprint integration time. The narrow exception
      clause is: PROMPT 856 does not run
      `cargo test --workspace --tests --no-fail-fast` (out of scope
      for `/story-done` paperwork; Cargo not run).

- [x] **AC9 -- If split chosen, per-message follow-on stories
      authored**: GIVEN the split decision, WHEN
      `production/epics/lightyear-protocol-verification/` is listed,
      THEN one follow-on story per split orphan exists with the
      no-claim banner, evidence-path conventions, and decision-first
      discipline inherited from this story. This umbrella story
      closes as the producer-decision-record artefact.
      **N/A** (PROMPT 856 verification): Umbrella path chosen per
      PROMPT 821 producer decision (this story's "Producer
      Decision" section line 267 `[x]` Umbrella); the split path
      `[ ]` is explicitly not chosen. The only per-message
      deferral is `S2CSangMepriseReveal` Path C (recorded inline
      within the umbrella, NOT a full split). The Sprint 14
      candidate follow-on story file
      `S14-PROTO-SANG-MEPRISE-DRAIN-001` is explicitly **not
      authored by PROMPT 821 / 852 / 856** per the story's Path C
      rationale ("PROMPT 821 does NOT author the Sprint 14
      candidate story (paperwork-only run; story-file authoring is
      a separate paperwork prompt)"). Follow-on authoring remains
      a separate paperwork prompt; PROMPT 856 honors that
      separation. AC9 status is therefore N/A by design.

- [x] **AC10 -- Sprint 12 disposition preserved**: GIVEN the
      implementation commit, WHEN `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, `production/stage.txt`,
      and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
      none of them are modified under this story. Sprint 12
      activation disposition is preserved. Stage remains `Polish`.
      Sprint 11 disposition (`closed-with-conditions`) is unchanged.
      Sprint 10 disposition (`closed-with-conditions`) is unchanged.
      **PASS** (PROMPT 856 verification): PROMPT 852 worker commit
      `9c0923f` + PROMPT 855 integration merge `ecec376` both
      excluded `production/sprint-status.yaml`,
      `production/sprints/sprint-12.md`, `production/stage.txt`,
      and `production/qa/qa-plan-sprint-12.md` from their diffs
      (verified by `git show --stat ecec376` showing only 17 files
      changed, none under `production/sprint*` or
      `production/stage.txt` or `production/qa/qa-plan-sprint-12.md`).
      Sprint 12 disposition (`closed-with-conditions` per PROMPT
      817) preserved unchanged. Sprint 11 disposition
      (`closed-with-conditions` per PROMPT 792) preserved unchanged.
      Sprint 10 disposition (`closed-with-conditions` per PROMPT
      763) preserved unchanged. Stage = `Polish` preserved
      (PROMPT 856 does not touch `production/stage.txt`).
      PROMPT 761 Polish→Release gate-check FAIL evidence at
      `production/gate-checks/gate-polish-release-2026-05-12.md`
      preserved (not in diff). NOTE: PROMPT 856 (`/story-done`
      paperwork) does flip the Sprint 13 row for
      `S13-PROTO-ORPHAN-DRAIN-001` from `ready` to `done` in
      `production/sprint-status.yaml`; that is the
      `/story-done`-prescribed paperwork update and is distinct
      from the AC10 binding (which is about the implementation
      diff, not the `/story-done` paperwork diff).

- [x] **AC11 -- Evidence document slot reserved**: GIVEN this story
      file, WHEN the evidence-doc path is checked, THEN a slot is
      reserved at
      `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
      (umbrella) or per-split-story (split). Authoring of the
      evidence file(s) is deferred to the implementation prompt(s).
      **PASS** (PROMPT 856 verification): the evidence document at
      `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
      (NEW; 160 lines on `origin/main@ecec376` via PROMPT 855
      integration merge of PROMPT 852 worker commit `9c0923f`) is
      authored by PROMPT 852 and contains every required section:
      No-Claim Banner (verbatim), per-orphan disposition table
      (10 rows including the `C2SClassChoice` allowlisted row),
      changed-files table, targeted verification table, pre/post
      invariant summary, AC verification matrix (AC1-AC11),
      cross-references, Cargo policy applied, and out-of-scope
      list. PROMPT 856 does NOT modify this evidence document
      (verified by `git diff` showing it untouched in PROMPT 856's
      paperwork commit).

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

- 2026-05-14 -- PROMPT 821 -- Per-orphan decision-recording
  paperwork. Umbrella-vs-split decision flipped to `[x]` Umbrella
  with multi-paragraph rationale. All 9 named orphan dispositions
  recorded inline (5×Path A + 2×Path B + 1×Path C +
  1×Path A C2S handler). Paperwork-only run; precedes any code
  change per the story's Wave 1 ordering.

- 2026-05-14 -- PROMPT 852 -- Implementation worker commit
  `9c0923f3f83652af27dd67fba9ceb8c155b3fd12` on
  `work/s13-protocol-orphan-drain` from base `origin/main@25573e6`.
  Worktree: `D:\_DEV\claude-code-game-studios-worktrees\s13-protocol-orphan-drain`.
  Landed: Path A drains for `S2COpponentDisconnected`,
  `S2COpponentReconnected`, `S2CPrismRespawned`,
  `S2CPrismRewardDropped`, `S2CSessionCancelled` in
  `client/src/presentation/mod.rs`; Path A server handler for
  `C2SRequestSnapshot` in
  `server/src/core/session/snapshot_request.rs`
  (216 lines NEW; reuses `build_game_snapshot` per ADR-011 with
  new `GameConfig::snapshot_cooldown_ms` default 5000ms);
  Path B atomic deletions for `S2CHeartbeat` + `S2CPoolUpdate`
  across `shared/src/protocol.rs` + `design/gdd/network-protocol.md`
  + `tests/integration/network/e2e_websocket_test.rs`;
  `tests/invariants/protocol_completeness_test.rs` un-ignored
  with 3-row inline `ALLOWLIST`; two new integration test files
  (227 + 154 lines NEW); evidence document
  `production/qa/evidence/sprint-13-proto-orphan-drain-evidence.md`
  (160 lines NEW). Targeted Cargo evidence: `cargo fmt --all
  --check` clean; `cargo check -p {shared,server,client}` clean;
  `cargo test -p shared --test protocol_completeness_invariant`
  2/2 PASS; `cargo test -p client --test
  presentation_protocol_orphan_drain_test` 6/6 PASS; `cargo test
  -p server --test request_snapshot_handler_test` 5/5 PASS.

- 2026-05-14 -- PROMPT 855 -- Integration merge commit
  `ecec3760af02401902e5959da38dad1bba4f2421` on `origin/main`,
  fast-forwarding from `3199c01` (PROMPT 851 `/story-done`
  closure of Story 007) into the integration tip. Merge of
  PROMPT 852 worker tip `9c0923f` into prior `origin/main@3199c01`;
  17 files changed, +1082 / -118 lines. All PROMPT 852 evidence
  reachable on `origin/main@ecec376`. Story 007's
  `S13-PROTO-INVARIANT-001` invariant test confirmed still in
  PASS-WITH-DOCUMENTED-ALLOWLIST state after Story 008
  integration (the `#[ignore]` removal landed in PROMPT 852
  and remains in effect on `origin/main@ecec376`).

- 2026-05-14 -- PROMPT 856 -- `/story-done` paperwork closure for
  `S13-PROTO-ORPHAN-DRAIN-001`. Source-of-truth: `origin/main@ecec376`.
  Worktree: shared root checkout `D:\_DEV\Work\Claude-Code-Game-Studios`
  (no new worktree; serialized shared-status writer per 2026-05-13
  override). HEAD verified == `origin/main@ecec376` before any edit.
  PROMPT 855 integration merge `ecec376` confirmed on `origin/main`.
  PROMPT 852 worker commit `9c0923f` confirmed reachable on
  `origin/main` (one commit before integration merge).
  This story file flipped `Status: Draft -> Done` with verdict
  **PASS-WITH-ALLOWLIST** (3-row documented allowlist per AC4 inline
  in `tests/invariants/protocol_completeness_test.rs:296-345`).
  AC1-AC11 all flipped `[ ] -> [x]` with per-AC closure-evidence
  annotations cross-referencing PROMPT 852 worker commit + PROMPT
  855 integration merge + evidence document.
  `production/sprint-status.yaml` Sprint 13 Must Have row
  `S13-PROTO-ORPHAN-DRAIN-001` flipped `status: ready -> done` with
  `completed: 2026-05-14`, `worker_prompt: 852`,
  `worker_commit: 9c0923f3f83652af27dd67fba9ceb8c155b3fd12`,
  `integration_prompt: 855`,
  `integration_commit: ecec3760af02401902e5959da38dad1bba4f2421`,
  `story_done_prompt: 856`. `production/session-state/active.md`
  + `production/session-state/codex-orchestrator-state.md`
  prepended with PROMPT 856 banner. Story 007 invariant remains
  in PASS-WITH-DOCUMENTED-ALLOWLIST state post-integration
  (verified by `git show ecec376:tests/invariants/protocol_completeness_test.rs`
  showing no `#[ignore]` on `protocol_completeness_assert_send_and_drain_sites`
  and the 3-row inline ALLOWLIST intact). No Cargo run (out of
  scope for `/story-done` paperwork; story-done policy did not
  require targeted recheck because PROMPT 852 evidence + PROMPT
  855 integration verification already cover all ACs). No smoke /
  team-qa / gate-check / release-check / sprint close-out run.
  Carried non-claims preserved verbatim (public release
  readiness, RC readiness, full game completion, broad / Standard-
  tier accessibility completion (`QA-COND-0005`), playtest / fun-
  hypothesis validation (`QA-COND-0006`), full playable-client
  manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`),
  final-art / asset-production completion, full-workspace
  `cargo test`, `S2COpponentDisconnected` server-broadcast send-
  site, `S2CSangMepriseReveal` client drain Path C deferral,
  `C2SClassChoice` drain-vs-delete disposition). Sprint 12
  disposition (`closed-with-conditions` per PROMPT 817) preserved.
  Sprint 11 disposition (`closed-with-conditions` per PROMPT 792)
  preserved. Sprint 10 disposition (`closed-with-conditions` per
  PROMPT 763) preserved. Stage = `Polish` preserved.
