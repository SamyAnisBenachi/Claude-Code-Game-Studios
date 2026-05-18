# Story 011: S18-PROTO-CLASSCHOICE-DROP-001 — Drop Dead `C2SClassChoice` Protocol Path

> **Epic**: Class System
> **Story ID**: S18-PROTO-CLASSCHOICE-DROP-001
> **Status**: Draft — Sprint 18 candidate (NOT activated by this authoring run)
> **Layer**: Feature (M3) / Protocol cleanup
> **Type**: Decision-first (Path A drop / Path B retain-with-rationale) + Config/Data (protocol deletion) + docs sync (ADR-014, control-manifest)
> **Sprint**: Sprint 18 candidate (Sprint 17 remains the active sprint at the authoring source-of-truth; activation of Sprint 18 happens via a separate `/sprint-plan sprint-18` prompt, NOT this story)
> **Authored**: 2026-05-18 by PROMPT 1305 (branch `work/s18-server-dead-state-hygiene-story-authoring-1305`)
> **Authoring source-of-truth**: `origin/main@6239c9ee636ae9c71fac92ad9ee31d898925f9b8` (PROMPT 1300 windows dev launcher canonical-main repair integration)
> **Source audit**: `reports/PROMPT-1298-server-dead-state-hygiene-audit.md` §3 F-06
> **Supersedes placeholder**: `S14-PROTO-CLASSCHOICE-DISPOSITION-001` (PROMPT 1202 F-06 placeholder slug) and the `lightyear-protocol-verification/story-008-protocol-orphan-drain.md` allowlist row that deferred `C2SClassChoice` disposition.

---

## Epic Ownership — Why `class-system` and not `lightyear-protocol-verification`

The PROMPT 1298 §3 F-06 finding allowed either epic. This story is placed
under `class-system` because the deletion footprint is overwhelmingly
class-system:

- `server/src/lobby/handler.rs` (handler module owned by `class-system`
  per `production/epics/class-system/EPIC.md` Architecture Module field
  and per ADR-014 §"Architecture Module" — `server/feature/class/` +
  `server/src/lobby/handler.rs`).
- `docs/architecture/adr-014-class-system-architecture.md` (class-system
  governing ADR; the dead protocol path is named in §"Key Interfaces" and
  §"Forbidden Patterns" of this ADR).
- `docs/architecture/control-manifest.md` lines `:128` and `:164`
  (`C2SClassChoice` single-drain forbidden-pattern rule; semantically a
  class-system rule).
- `server/tests/class_lifecycle_test.rs` (class-system test surface).
- `shared/src/protocol.rs` (single-writer protocol file; touched by
  every protocol-disposition story regardless of owning epic).

The only `lightyear-protocol-verification` precedent is `story-008-
protocol-orphan-drain.md`, whose own allowlist explicitly deferred
`C2SClassChoice` disposition to a follow-on. This story is that follow-on.
Co-locating it with the class-system ADR amendment is cleaner than
splitting the ADR edit (class-system) from the protocol edit (`lightyear-
protocol-verification`); a single PR / commit set lands the full
deletion footprint.

This decision is recorded as a producer call (PROMPT 1305 authoring run) and
is reversible if the implementation prompt elects to split the disposition
across two stories (one ADR-amendment story under `class-system`, one
protocol-deletion story under `lightyear-protocol-verification`). The
recommendation here is umbrella-under-class-system.

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
- Modify any file under `docs/architecture/**` (ADR-014 amendment and
  control-manifest edits are deliverables of the implementation prompt
  that lands this story; the authoring run only records the planned
  edits).
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan` on this story.

This story does **not** claim: release readiness, sprint close-out,
ADR-014 supersession verdict, gate-check pass, or any production state
advance.

---

## Source Finding (PROMPT 1298 F-06)

`reports/PROMPT-1298-server-dead-state-hygiene-audit.md` §3 F-06:

- **Protocol type**: `C2SClassChoice { class: ClassId }` at
  `shared/src/protocol.rs:453`.
- **Channel registration**: `register_c2s::<C2SClassChoice>(registry,
  ProtocolChannel::Reliable);` at `shared/src/protocol.rs:61`.
- **Server-side drainer**: `handle_class_choice` system at
  `server/src/lobby/handler.rs:10-37`; registered in `ServerNetworkPlugin`
  at `server/src/network/mod.rs:39`.
- **Server-side helper**: `apply_class_choice` at
  `server/src/lobby/handler.rs:40-50`.
- **Tests**: `server/tests/class_lifecycle_test.rs:11, :88, :106, :123,
  :181` — 4+ test cases exercising `apply_class_choice` with synthetic
  `C2SClassChoice` payloads.
- **Client senders**: zero references to `MessageSender<C2SClassChoice>`
  anywhere in `client/**` (origin/main exhaustive grep).
- **Production class-update path** (the one clients actually use):
  - `C2SSelectClass` — sender `client/src/ui/lobby.rs:750`, `:817`
    (`send_lobby_commands_system`); drainer `server/src/core/session/
    system.rs:708-743`.
  - `C2SConfirmClass` — sender `client/src/ui/lobby.rs:751`, `:834`;
    drainer `server/src/core/session/system.rs:745-755`.

- **Status**: Dead protocol path. `MessageReceiver<C2SClassChoice>` will
  always be empty in production. `handle_class_choice` is a hot-path
  no-op every tick. `apply_class_choice` and the protocol type itself are
  reachable only from tests.

### ADR-014 staleness

ADR-014 (`docs/architecture/adr-014-class-system-architecture.md`)
specifies `C2SClassChoice` as the canonical LOBBY message and registers a
single-drain forbidden-pattern rule for `MessageReceiver<C2SClassChoice>`
(control-manifest.md `:164`). That ADR predates the two-step
`C2SSelectClass` / `C2SConfirmClass` lobby UX that lobby UI currently
depends on. The implemented protocol is the two-step variant; the ADR is
stale with respect to the implemented protocol.

---

## Problem Class / Prevention Target

**Defect class (PROMPT 1298 audit row F-06, same family as PROMPT 1202
F-06 and as DC-1 from PROMPT 803)**: a C2S message + handler + helper
function are wired through `ServerNetworkPlugin` and tested in isolation,
but no client production code ever produces the message. The compiler
cannot detect this; the only safety net is manual audit.

**Prevention target**: remove the dead protocol type, the dead server
handler module function, the dead helper function, the server system
registration, and the orphan tests. Amend ADR-014 + control-manifest to
reflect the implemented two-step (`C2SSelectClass` + `C2SConfirmClass`)
class-update protocol. After this story lands:

- `shared/src/protocol.rs` has no `C2SClassChoice` definition or
  registration.
- `server/src/lobby/handler.rs` has no `handle_class_choice` or
  `apply_class_choice` function; the file may be deleted entirely if it
  has no other production exports.
- `ServerNetworkPlugin` does not register `handle_class_choice`.
- `server/tests/class_lifecycle_test.rs` is deleted (or rewritten — see
  coverage-equivalence AC below).
- ADR-014 is amended (Path A: marked Superseded by an ADR-014-revision
  ADR; or Path B: amended in place with a revision header). The
  control-manifest single-drain rule for `MessageReceiver<C2SClassChoice>`
  is removed; equivalent single-drain rules for
  `MessageReceiver<C2SSelectClass>` and `MessageReceiver<C2SConfirmClass>`
  are added.

---

## Per-Item Decisions (decision-first per the story-008 precedent)

### Disposition for `C2SClassChoice` (the protocol type)

- [ ] **Path A — Drop**. Recommended (PROMPT 1305 authoring run):
      The type has zero client senders; the server drainer is a no-op
      every tick; the only consumers are tests that exercise
      `apply_class_choice` directly with synthetic payloads (not over the
      wire). Deletion is safe under the coverage-equivalence AC below.
- [ ] **Path B — Retain with allowlist rationale**. Not chosen unless the
      implementation prompt discovers an unknown latent consumer. Rationale
      template (must be filled if Path B is chosen): "Retained because
      [specific consumer name + file:line]. Allowlist entry added to
      `S13-PROTO-INVARIANT-001` invariant test fixture (`tests/invariants/
      protocol_completeness_test.rs`) with rationale + follow-on story
      reference."

**Default decision** (recorded here, may be revisited by the
implementation prompt if a latent consumer is discovered): **Path A —
Drop**.

### Disposition for `handle_class_choice` system

- [ ] **Path A — Delete**. Recommended. Function lives at `server/src/
      lobby/handler.rs:10-37`; deletion is straightforward and removes the
      hot-path no-op every tick.
- [ ] **Path B — Keep stub**. Not chosen.

**Default decision**: **Path A — Delete**.

### Disposition for `apply_class_choice` helper

- [ ] **Path A — Delete**. Recommended. Only test code calls
      `apply_class_choice`; with `C2SClassChoice` deleted, the helper has
      no caller.
- [ ] **Path B — Keep as private helper**. Not chosen.

**Default decision**: **Path A — Delete**.

### Disposition for `server/src/lobby/handler.rs` module file

- [ ] **Path A — Delete file**. Default if the file contains no other
      production exports after `handle_class_choice` and `apply_class_choice`
      are removed. The implementation prompt verifies emptiness before
      deletion.
- [ ] **Path B — Keep file empty (placeholder for future lobby
      handlers)**. Only if the implementation prompt elects to retain a
      stub for future lobby C2S handlers. Default is **Path A**; the
      implementation prompt may choose B with rationale.

**Default decision**: **Path A — Delete file**; if file becomes empty,
also delete the `pub mod handler;` declaration in `server/src/lobby/mod.rs`.

### Disposition for ADR-014

- [ ] **Path A — Mark Superseded by an ADR-014-revision ADR (e.g.,
      `adr-014a-class-system-two-step-lobby-protocol.md`)**. Preferred
      because it preserves history and avoids in-place rewrite of an
      Accepted ADR.
- [ ] **Path B — Revise in place with a Revision Note header**. Allowed.
      Less history-preserving but simpler.
- [ ] **Path C — Defer to technical-director ruling**. The implementation
      prompt MUST record which path the technical-director sign-off
      endorses before landing the ADR edit. If unsign-able, the story is
      BLOCKED on the ADR amendment but the protocol deletion may still
      land (decoupled).

**Default recommendation** (advisory, not binding): **Path A — Superseded
by revision ADR**.

### Disposition for `control-manifest.md` rules

- [ ] **Path A — Remove the two `C2SClassChoice` bullets at `:128` and
      `:164`; add equivalent single-drain rules for `MessageReceiver<
      C2SSelectClass>` (one drainer in `server/src/core/session/system.rs:
      708-743`) and `MessageReceiver<C2SConfirmClass>` (one drainer at
      `:745-755`)**. Recommended.
- [ ] **Path B — Remove the bullets without adding replacements**. Not
      chosen: the single-drain invariant for class-update messages is
      still load-bearing under ADR-002 / ADR-008; the replacements must
      be added in the same commit set.

**Default decision**: **Path A**.

### Disposition for `server/tests/class_lifecycle_test.rs`

- [ ] **Path A — Delete entire file** once the coverage-equivalence AC
      below is verified.
- [ ] **Path B — Rewrite to target `C2SSelectClass` / `C2SConfirmClass`
      handler paths** if the existing two-step-message tests at
      `server/src/core/session/system.rs:708-755` callers do not already
      cover the three scenarios named in the coverage-equivalence AC.

**Default decision**: **Path A — Delete file**, conditional on the
coverage-equivalence AC passing. If any scenario is missing, the
implementation prompt MUST author the missing test against the two-step
path FIRST (in the same commit set), then delete the dead file.

---

## Context

### Existing surface (PROMPT 1298 F-06 verbatim)

- `shared/src/protocol.rs:453` — `pub struct C2SClassChoice { pub class:
  ClassId }`.
- `shared/src/protocol.rs:61` — `register_c2s::<C2SClassChoice>(registry,
  ProtocolChannel::Reliable);`.
- `server/src/lobby/handler.rs:10-37` — `pub fn handle_class_choice(...)`.
- `server/src/lobby/handler.rs:40-50` — `pub fn apply_class_choice(
  sessions: &mut PlayerSessions, player_id: PlayerId, msg: C2SClassChoice)`.
- `server/src/network/mod.rs:7` — `use crate::lobby::handler::
  handle_class_choice;`.
- `server/src/network/mod.rs:39` — `handle_class_choice` in the
  `add_systems(Update, ...)` tuple of `ServerNetworkPlugin`.
- `server/tests/class_lifecycle_test.rs:11, :88, :106, :123, :181` — test
  cases against `apply_class_choice`.
- `docs/architecture/adr-014-class-system-architecture.md` — `C2SClassChoice`
  named in §"Key Interfaces", §"Lifecycle", §"Forbidden Patterns".
- `docs/architecture/control-manifest.md:128, :164` — two
  `C2SClassChoice`-specific bullets.

### Live two-step class-update protocol (PRESERVE)

The implemented production path is the two-step `C2SSelectClass` +
`C2SConfirmClass` lobby UX:

- `shared/src/protocol.rs` — `C2SSelectClass`, `C2SConfirmClass` (lines
  verified by the implementation prompt at edit time).
- `client/src/ui/lobby.rs:750, :817` — `MessageSender<C2SSelectClass>`
  send sites.
- `client/src/ui/lobby.rs:751, :834` — `MessageSender<C2SConfirmClass>`
  send sites.
- `server/src/core/session/system.rs:708-743` — `MessageReceiver<
  C2SSelectClass>` drainer.
- `server/src/core/session/system.rs:745-755` — `MessageReceiver<
  C2SConfirmClass>` drainer.

This story must not touch these files in a way that disrupts the
two-step protocol. The only edits to this surface that this story permits
are: (a) adding the new single-drain control-manifest rules naming
`C2SSelectClass` / `C2SConfirmClass`; and (b) any test files added under
Path B of the `class_lifecycle_test.rs` disposition (see coverage-
equivalence AC).

### Engine

- **Engine**: Bevy 0.18 (Rust). The deletion involves removing a Bevy
  system from `add_systems`, removing a Bevy `MessageReceiver<T>` system
  param, and deleting plain Rust helper functions.
- **Lightyear**: 0.26 — the protocol-registration call site
  (`shared/src/protocol.rs:61`) is removed.

### Mandatory skills

- **`liv-bevy-018`** — mandatory for the `.rs` edits (system removal from
  `add_systems`, `MessageReceiver<T>` removal).
- **`liv-bevy-lightyear`** — mandatory for the protocol edit
  (`register_c2s::<C2SClassChoice>` removal); cross-reference
  `docs/engine-reference/bevy/VERSION.md` and Lightyear 0.26 release
  notes before editing `register_protocol`.

### Control Manifest Rules (Foundation + Feature scope)

- Required: After this story lands, every C2S message type registered in
  `shared/src/protocol.rs` MUST have at least one production producer
  (client-side sender) AND at least one production consumer (server-side
  drainer). `C2SClassChoice` is the last known violator; its deletion
  restores the invariant.
- Required: Each protocol-level deletion under this story removes the
  channel binding in the same commit set that removes the type definition
  (the only ADR-008-compatible channel-binding change is "remove a
  deleted message's binding").
- Required: Replacement single-drain rules for `MessageReceiver<
  C2SSelectClass>` and `MessageReceiver<C2SConfirmClass>` are added to
  the control manifest in the same commit set that removes the
  `C2SClassChoice` rule. No drift between the manifest and the
  implemented two-step protocol.
- Forbidden: Adding a third class-update message ("`C2SSetClass`",
  "`C2SChooseClass`", etc.) under any name. The decision is REMOVE the
  dead path, not RENAME it.
- Forbidden: Loosening the single-drain invariant. Every
  `MessageReceiver<T>` for class-update messages MUST be drained in
  exactly one production system.
- Forbidden: Touching `client/src/ui/lobby.rs` `MessageSender<
  C2SSelectClass>` / `MessageSender<C2SConfirmClass>` send sites in this
  story (they are out of scope; their disposition is `Done` under the
  class-lifecycle epic story-001 implementation).

---

## Story Classification

**Story type**: Decision-first per item (drop vs retain for each of:
protocol type, server handler, helper, module file, ADR-014,
control-manifest rules, test file) + Config/Data (protocol deletion) +
Integration (system registration removal) + docs sync (ADR-014 +
control-manifest amendments).

This is **NOT** a:

- Pure refactor story (real semantic surface — the ADR contract and the
  control-manifest forbidden-pattern rule — is corrected).
- Pure deletion story (each item carries an explicit decision with
  rationale, following the `story-008-protocol-orphan-drain.md`
  per-orphan precedent).
- Lobby UX change story (the implemented two-step `C2SSelectClass` /
  `C2SConfirmClass` UX is preserved verbatim).

---

## Acceptance Criteria

*Each AC is a verifiable post-condition checked by the implementation
prompt that lands this story.*

### Coverage equivalence — BLOCKING gate before any deletion

- [ ] **AC1** (**BLOCKING — PRE-DELETION**): Before any deletion lands,
      the implementation prompt enumerates production-path coverage for
      each of the three scenarios named in `server/tests/class_lifecycle_
      test.rs` (PROMPT 1298 §3 F-06 "Tests — Verify before delete"):
      1. **Unlocked + valid (non-Neutral) class → `player.class` is
         updated.** A test must exist that drives
         `MessageReceiver<C2SSelectClass>` (or its drainer at
         `server/src/core/session/system.rs:708-743`) and asserts the
         `class` field update.
      2. **Locked + valid class → `player.class` is unchanged.** A test
         must exist that drives `MessageReceiver<C2SSelectClass>` against
         a `class_locked == true` player and asserts the `class` field is
         not mutated.
      3. **Unlocked + `ClassId::Neutral` → rejected.** A test must exist
         that drives `MessageReceiver<C2SSelectClass>` with a `Neutral`
         payload and asserts the message is rejected without mutating
         state.
      The implementation prompt quotes the test path(s) for each scenario
      in the worker report. If any scenario is missing equivalent
      coverage, **AC1 fails** and the deletion is BLOCKED. The missing
      coverage MUST be authored against the two-step (`C2SSelectClass` /
      `C2SConfirmClass`) path in the same commit set, BEFORE the
      `class_lifecycle_test.rs` deletion is staged.

### Protocol + server-side deletions

- [ ] **AC2**: `shared/src/protocol.rs` no longer contains
      `pub struct C2SClassChoice` or `register_c2s::<C2SClassChoice>`.
      Verified by `grep -n "C2SClassChoice" shared/src/protocol.rs`
      returning zero matches.
- [ ] **AC3**: `server/src/lobby/handler.rs` no longer contains
      `handle_class_choice` or `apply_class_choice`. If the file becomes
      empty, the file is deleted and the `pub mod handler;` (or
      equivalent) declaration is removed from `server/src/lobby/mod.rs`.
      Verified by `grep -rn "handle_class_choice\|apply_class_choice"
      server/src/` returning zero matches AND, if the file is deleted,
      `git ls-files | grep "server/src/lobby/handler.rs"` returning
      empty.
- [ ] **AC4**: `server/src/network/mod.rs` no longer references
      `handle_class_choice` in any `use` declaration or `add_systems`
      tuple. Verified by `grep -n "handle_class_choice" server/src/
      network/mod.rs` returning zero matches AND `cargo check
      --workspace` green.
- [ ] **AC5**: `server/tests/class_lifecycle_test.rs` is deleted (or
      rewritten per Path B of the test disposition; default is delete).
      Verified by `git ls-files | grep "server/tests/class_lifecycle_test.
      rs"` returning empty (Path A) OR by the file containing no
      reference to `C2SClassChoice` / `apply_class_choice` (Path B).
- [ ] **AC6**: Workspace-wide `grep -rn "C2SClassChoice\|handle_class_
      choice\|apply_class_choice" shared/ server/ client/ tests/` returns
      zero matches.

### Two-step protocol PRESERVATION (BLOCKING)

- [ ] **AC7** (**PRESERVATION — BLOCKING**): `client/src/ui/lobby.rs`
      continues to send `C2SSelectClass` at the existing call sites
      (audit-reference `:750, :817`) and `C2SConfirmClass` at
      (audit-reference `:751, :834`). Verified by `grep -n
      "C2SSelectClass\|C2SConfirmClass" client/src/ui/lobby.rs`
      continuing to return ≥4 matches (two per message) AND `git diff
      origin/main..HEAD -- client/src/ui/lobby.rs` showing no behavioural
      change to the send sites (line numbers may shift but the
      `MessageSender<T>.send(...)` calls must remain).
- [ ] **AC8** (**PRESERVATION — BLOCKING**): `server/src/core/session/
      system.rs` continues to drain `MessageReceiver<C2SSelectClass>` at
      the existing drain site (audit-reference `:708-743`) and
      `MessageReceiver<C2SConfirmClass>` at (audit-reference `:745-755`).
      Verified by `grep -n "MessageReceiver<C2SSelectClass>\|
      MessageReceiver<C2SConfirmClass>" server/src/core/session/system.rs`
      continuing to return ≥2 matches AND `git diff origin/main..HEAD --
      server/src/core/session/system.rs` showing no behavioural change to
      the drain logic.

### Docs sync

- [ ] **AC9**: `docs/architecture/adr-014-class-system-architecture.md`
      is amended per the ADR-disposition path chosen (Path A:
      Superseded by a new revision ADR; Path B: in-place revision; Path
      C: technical-director ruling required). The amendment removes
      `C2SClassChoice` from §"Key Interfaces", §"Lifecycle", and
      §"Forbidden Patterns", and replaces those references with
      `C2SSelectClass` + `C2SConfirmClass` describing the implemented
      two-step UX. Verified by `grep -n "C2SClassChoice" docs/
      architecture/adr-014-class-system-architecture.md` returning zero
      matches.
- [ ] **AC10**: `docs/architecture/control-manifest.md` has the two
      `C2SClassChoice`-specific bullets (audit-reference `:128`, `:164`)
      removed AND has two new single-drain bullets added that name
      `MessageReceiver<C2SSelectClass>` (one drainer in
      `server/src/core/session/system.rs:708-743`) and
      `MessageReceiver<C2SConfirmClass>` (one drainer at `:745-755`).
      Verified by `grep -n "C2SClassChoice" docs/architecture/control-
      manifest.md` returning zero matches AND `grep -n "MessageReceiver<
      C2SSelectClass>\|MessageReceiver<C2SConfirmClass>" docs/architecture/
      control-manifest.md` returning ≥2 matches.
- [ ] **AC11**: `production/epics/class-system/EPIC.md` is amended to
      drop `C2SClassChoice` from the §"Engine Notes" line ("`C2SClassChoice`
      uses `lightyear::prelude::Message` ..."), from the Pre-Implementation
      Gate 4 (`MessageReceiver<C2SClassChoice>` single-drain rule), from
      the §"Deliverables" bullets that name `C2SClassChoice` and
      `handle_class_choice`, and from the GDD-Requirements TR-CS-001
      ADR-coverage description. The references are replaced with the
      two-step `C2SSelectClass` / `C2SConfirmClass` equivalents where
      appropriate. Verified by `grep -n "C2SClassChoice" production/
      epics/class-system/EPIC.md` returning zero matches.
- [ ] **AC12**: `production/epics/class-system/story-001-class-lifecycle.
      md` is amended so its Acceptance Criteria, Implementation Notes,
      Engine Notes, and Control-Manifest-Rules sections no longer
      reference `C2SClassChoice` as the canonical class-update message.
      Story-001 status remains `Complete`; this is doc sync only.
      Verified by `grep -n "C2SClassChoice" production/epics/class-
      system/story-001-class-lifecycle.md` returning zero matches.

### Invariant test compatibility

- [ ] **AC13**: If `tests/invariants/protocol_completeness_test.rs`
      (`S13-PROTO-INVARIANT-001` per `lightyear-protocol-verification/
      story-007`) exists, any allowlist entry mentioning
      `C2SClassChoice` is removed in the same commit set. Verified by
      `grep -n "C2SClassChoice" tests/invariants/` returning zero
      matches. (If the invariant test file does not exist yet, this AC
      is a no-op.)

### Build

- [ ] **AC14**: `cargo check --workspace` is green and zero new warnings
      land on `shared/src/`, `server/src/`, or `client/src/`. (Run by the
      implementation prompt under the project's Windows/MSVC Cargo
      resource policy; the authoring run does NOT run Cargo.)

---

## Out of Scope

- **Two-step `C2SSelectClass` / `C2SConfirmClass` protocol** — explicitly
  preserved by AC7 / AC8 and **must not be modified** by this story.
- **`client/src/ui/lobby.rs`** send-site logic — preserved verbatim.
- **`server/src/core/session/system.rs`** drain-site logic — preserved
  verbatim (modulo coverage-equivalence test additions if AC1 forces
  authoring under Path B of the test disposition).
- **F-05 / F-09** — the two other PROMPT 1298 findings are owned by
  `S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001` and
  `S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001` respectively.
- **Sprint activation** and any sprint-status / session-state /
  stage.txt edits — handled by the orchestrator outside this story.
- **QA plan amendments** — Sprint 18 QA plan authoring is a separate
  `/qa-plan` invocation; this story is implementation-grade material
  consumed by that plan, not the plan itself.

---

## QA Test Cases

*Behavioural test cases; the implementation prompt verifies each.*

- **AC1 (coverage equivalence)**
  - **Given**: `origin/main@6239c9e` test suite.
  - **When**: the implementation prompt enumerates tests covering the
    three scenarios (unlocked-valid, locked-valid, unlocked-Neutral) for
    the `C2SSelectClass` / `C2SConfirmClass` drain path.
  - **Then**: at least one test path per scenario is quoted in the
    worker report. If any scenario is missing, the missing test is
    authored against the two-step path in the same commit set BEFORE
    deletion.

- **AC2-AC6 (deletion)**
  - **Given**: the working branch after the implementation prompt lands.
  - **When**: `grep -rn "C2SClassChoice\|handle_class_choice\|apply_class_
    choice" shared/ server/ client/ tests/` runs.
  - **Then**: Zero matches workspace-wide.

- **AC7-AC8 (preservation)**
  - **Given**: the working branch.
  - **When**: `grep -n` on the two-step protocol send/drain sites runs.
  - **Then**: All existing `C2SSelectClass` / `C2SConfirmClass` send and
    drain references remain present; behaviour is unchanged.

- **AC9-AC13 (docs + invariant sync)**
  - **Given**: the working branch.
  - **When**: `grep -rn "C2SClassChoice" docs/architecture/ production/
    epics/class-system/ tests/invariants/` runs.
  - **Then**: Zero matches across ADR-014, control-manifest,
    class-system EPIC.md, class-system story-001, and any invariant
    fixture.

- **AC14 (build green)**
  - **Given**: the working branch on Windows/MSVC with the project's
    Cargo resource policy applied.
  - **When**: the implementation prompt runs `cargo check --workspace`.
  - **Then**: exit 0 with zero new warnings.

---

## Test Evidence

**Story Type**: Decision-first + Config/Data + Integration + docs sync
**Required evidence**:

1. Quoted test path(s) for each of the three coverage-equivalence
   scenarios (unlocked-valid, locked-valid, unlocked-Neutral) on the
   `C2SSelectClass` / `C2SConfirmClass` drain path (AC1).
2. Quoted output of the workspace `grep -rn "C2SClassChoice\|handle_class_
   choice\|apply_class_choice" shared/ server/ client/ tests/` showing
   zero matches (AC2–AC6).
3. Quoted output of the two-step preservation greps on
   `client/src/ui/lobby.rs` and `server/src/core/session/system.rs`
   (AC7–AC8).
4. Quoted output of the docs-sync greps on `docs/architecture/adr-014-
   class-system-architecture.md`, `docs/architecture/control-manifest.md`,
   `production/epics/class-system/EPIC.md`, `production/epics/class-
   system/story-001-class-lifecycle.md` (AC9–AC12).
5. Quoted output of `grep -n "C2SClassChoice" tests/invariants/` (AC13).
6. `cargo check --workspace` exit status (AC14).
7. Evidence file path: `tests/evidence/class-story-011-classchoice-drop.
   md`.

**Status**: Not yet created (authoring run only). Created by the
implementation prompt.

---

## Dependencies

- **Depends on**: none on `origin/main@6239c9e`. The PROMPT 1287 §4.3
  Lane A2 parallel-lane map marks this finding as standalone modulo the
  single-writer constraint on `shared/src/protocol.rs` and
  `server/src/network/mod.rs`.
- **Unlocks**: the `S13-PROTO-INVARIANT-001` invariant test
  (`tests/invariants/protocol_completeness_test.rs`) can drop its
  `C2SClassChoice` allowlist row once this story lands. Per
  `lightyear-protocol-verification/story-008-protocol-orphan-drain.md`'s
  allowlist closure note, that drop is a follow-on commit to the
  invariant fixture.
- **Sprint 18 lane**: PROMPT 1287 §4.3 Lane A2 (server hygiene) — same
  lane as `S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001`, but a different
  file set so the two can run in parallel.

---

## Parallel Safety Notes

- ⚠ **Single-writer rule** on `shared/src/protocol.rs` — serialise with
  any other Sprint 18 lane that mutates the protocol registry (notably
  `S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001` if its
  implementation prompt elects the "drop" variant). The orchestrator
  schedules these waves; this story does not implement the schedule.
- ⚠ **Single-writer rule** on `server/src/network/mod.rs` — serialise
  with any other Sprint 18 lane that adds or removes systems from
  `ServerNetworkPlugin`.
- ⚠ **Single-writer rule** on `docs/architecture/adr-014-class-system-
  architecture.md` and `docs/architecture/control-manifest.md` —
  serialise with any other Sprint 18 doc-sync lane.
- ✅ Otherwise safe to run in parallel with the RSM auction-safety-timer
  lane (`S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001`) and with any
  client-UI lane that does not touch `client/src/ui/lobby.rs` class-
  selection send sites.

---

## Notes for the Implementation Prompt

- Read PROMPT 1298 §3 F-06 verbatim before editing; the deletion
  footprint is exactly the file set quoted in this story's Context
  section.
- **Verify AC1 (coverage equivalence) FIRST.** Do not stage any deletion
  until the three required scenarios are confirmed-covered on the
  two-step path. If a gap exists, author the missing test against
  `C2SSelectClass` / `C2SConfirmClass` in the same commit set BEFORE
  deleting `class_lifecycle_test.rs`.
- Make the ADR-014 decision (Path A vs B vs C) explicit in the worker
  report; cite the technical-director sign-off if Path C.
- Activate `liv-bevy-018` for `.rs` edits and `liv-bevy-lightyear` for
  the protocol registration removal at `shared/src/protocol.rs:61`.
- Do not introduce a replacement single-step class-update message.
  Doing so reintroduces the same dead-protocol-path defect class and
  contradicts the PROMPT 1298 §3 F-06 recommendation.
- If `server/src/lobby/handler.rs` becomes empty after deletion,
  delete the file AND the corresponding `pub mod handler;` declaration
  in `server/src/lobby/mod.rs`. If `server/src/lobby/mod.rs` becomes
  empty, delete that too (and the `pub mod lobby;` declaration in
  `server/src/lib.rs` or wherever the module is rooted). Walk the
  module tree as far as the emptiness propagates.
