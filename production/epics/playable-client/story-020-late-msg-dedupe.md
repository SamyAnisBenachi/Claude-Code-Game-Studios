# Story 020: S13-LATE-MSG-DEDUPE-001 -- Client-Side Idempotency for Late / Duplicate Reliable S2C Messages

> **Epic**: Playable Client
> **Story ID**: S13-LATE-MSG-DEDUPE-001
> **Status**: Done -- /story-done verdict PASS per PROMPT 884 on
> 2026-05-14 against integrated evidence on `origin/main@6163cd3` (PROMPT
> 883 integration merge of PROMPT 872 worker commit `dfe5f21` into prior
> `origin/main@c379625`). AC1-AC9 + AC11 + AC12 all PASS; AC10
> PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK (full workspace tests
> intentionally NOT run per Sprint 13 QA-plan no-full-workspace-tests-by-
> default policy + worker prompt scope; targeted check 17/17 pass +
> 7/7 unit pass). Sprint 13 disposition UNCHANGED `active`; stage
> UNCHANGED `Polish`; PROMPT 761 Polish->Release gate-check FAIL
> preserved. PROMPT 884 was a paperwork-only closure.
> **Layer**: Reconnect / Idempotency
> **Type**: Integration -- per-drain dedupe state + new integration tests
> **Sprint**: Sprint 13 (activated 2026-05-14 per PROMPT 826)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint
(`status: active`) and must not be changed by this authoring run.

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

Sprint 10 / Sprint 11 dispositions unchanged. PROMPT 761 Polish->Release
gate-check FAIL evidence preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** Dedupe state is purely defensive on the receive side: when a
duplicate S2C reliable broadcast arrives, the drain detects the
duplicate and no-ops. The client remains read-only over server-
authoritative state; the dedupe state is part of the read-only
projection. ADR-002 + ADR-008 + ADR-011 binding.

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-6** Reconnect / snapshot late-message idempotency (HIGH):
  Late `S2CGameOver`, `S2CClassLocked`, `S2CPlaceUnit`-class messages
  are phase-gated but not dedupe-guarded. Only `C2SAcknowledgeResult`
  has explicit idempotency. Evidence anchor:
  `tests/integration/session/result_acknowledgement_contract_test.rs:91-96`
  (positive) vs missing analogue elsewhere.
- **§4 Lane C "DC-6 late-message idempotency"**: only
  `C2SAcknowledgeResult` has explicit duplicate-noop coverage.
  Duplicate `S2CGameOver` or `S2CClassLocked` arriving after
  reconnect re-send is *not* dedupe-guarded on client. Test that
  would have caught: a fuzz/late-message matrix test -- does not
  exist.
- **§5 Should row 1 (S13-LATE-MSG-DEDUPE-001)**: "Add `(round,
  message-id)` dedupe set on client drains for `S2CGameOver`,
  `S2CClassLocked`, `S2CPlaceUnit` so duplicate reliable redelivery
  is idempotent". Likely files:
  `client/src/presentation/result_screen.rs`,
  `client/src/ui/lobby.rs`, `client/src/ui/hand/*`.
- **§6 PROMPT-N+4 (paired with S13-CONN-LOST-UX-001)**: paperwork-
  only story-authoring.

---

## Problem Class / Prevention Target

**Defect class (DC-6)**: When a client reconnects mid-game (or the
server's reconnect flow re-sends authoritative state), late or
duplicate reliable S2C messages may arrive on the client drain. The
existing `C2SAcknowledgeResult` flow has explicit idempotency: a
duplicate acknowledge is a no-op (`tests/integration/session/result_acknowledgement_contract_test.rs:91-96`).
No analogous dedupe exists for other reliable S2C drains.

Symptoms: a duplicate `S2CGameOver` after reconnect re-send could
trigger the result-screen sequence twice (e.g., re-playing a
result-screen entry animation); a duplicate `S2CClassLocked` could
confuse the lobby state machine; a duplicate `S2CPlaceUnit`-class
message could double-render placement. The reliable channel mostly
dedupes at transport, but reconnect-snapshot replay
(`server/src/core/session/reconnect.rs:198-233`) is a known path
where the same message identity can re-arrive.

**Prevention target**: Add `(round, message-id)` (or equivalent
canonical idempotency key) dedupe state on each client drain that
handles a reliable S2C message whose duplicate would have a
user-visible side effect. The drainers covered by this story:

- `S2CGameOver` drain in `client/src/presentation/result_screen.rs`.
- `S2CClassLocked` drain in `client/src/ui/lobby.rs:326-335`.
- `S2CPlaceUnit` (and related placement-reveal) drains in
  `client/src/ui/hand/*` and `client/src/presentation/board_rendering.rs`.

Each drain maintains a small dedupe set scoped to the session
lifetime. On reconnect, the set is preserved (server-side reconnect
state already preserves `EndedSessionResultState.final_snapshots` at
`server/src/core/session/state.rs:162`; the client mirrors the
session-lifetime scope).

Each drain's dedupe behaviour is covered by a new integration test
following the `result_acknowledgement_contract_test.rs:91-96`
precedent.

---

## Context

### Existing surface

- **`tests/integration/session/result_acknowledgement_contract_test.rs:91-96`**:
  the canonical idempotency-test precedent. Verifies that a duplicate
  `C2SAcknowledgeResult` is a no-op.
- **`server/src/core/session/reconnect.rs:198-233`**: server-side
  reconnect flush order (snapshot → ObjectiveIdentities →
  PhaseChanged → deferred-queue replay; all reliable).
- **`server/src/core/session/reconnect.rs:836-893`**: GameOver
  reconnect re-sends snapshot + `S2CGameOver`. This is the canonical
  late-message scenario.
- **`server/src/core/session/state.rs:162`**:
  `EndedSessionResultState.final_snapshots` retention.
- **`client/src/presentation/result_screen.rs:324-334`**: result
  screen disconnect reason copy (per PROMPT 803 §4 Lane E). The
  `S2CGameOver` drain lives near here.
- **`client/src/ui/lobby.rs:326-335`**: `S2CClassLocked` drain.
- **`client/src/ui/hand/*`**: hand UI placement drains.
- **`client/src/presentation/board_rendering.rs`**: board placement
  drains.

### Message-id source

The reliable S2C messages do not currently carry an explicit
message-id field. The dedupe key is constructed from `(round,
message-type, message-content-hash)` or `(round, sequence-num)` as
the implementation prompt decides. The future
`S13-PROTO-MESSAGE-ID-001` (Sprint 14 Nice per PROMPT 803 §5 Nice)
adds an explicit sequence-id field; until then this story uses the
canonical content-hash or round+content-key approach.

### Dedupe scope

- **Session-lifetime scope**: the dedupe set is created at session
  entry and cleared at session exit (`OnExit(InSession)` or
  equivalent).
- **Bounded size**: each dedupe set is bounded (e.g., last 32
  message-ids per type) to avoid unbounded memory growth in long
  sessions.

### GDD / ADR / TR trace

- **No GDD change**: this is a client-side idempotency hardening.
- **ADR-002** (Client-Server Authority): the dedupe state is part
  of the read-only client projection; it does not mutate
  authoritative state.
- **ADR-008** (Lightyear Channel Config): no channel-binding
  change; reliable delivery is preserved.
- **ADR-011** (Reconnect Snapshot): the dedupe state must survive
  the reconnect flow correctly -- i.e., when the snapshot replays
  late messages, the dedupe set already contains the keys from the
  pre-reconnect session (because the messages were drained once on
  the pre-reconnect connection). If the dedupe scope is session-
  lifetime and the reconnect preserves the session, the set is
  preserved automatically; the implementing worker confirms via
  Test 4 below.
- **TR registry**: no new TR.

### Engine

- **Engine**: Bevy 0.18 (Rust). Dedupe state is a Bevy `Resource`
  per drain or a single `ClientIdempotencyState` resource grouping
  the dedupe sets.
- **Lightyear**: 0.26. No protocol change.

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for `.rs` code edits.
- **`liv-bevy-lightyear`** -- mandatory for protocol / drain code
  edits.

### Control Manifest Rules (Reconnect / Idempotency scope)

- Required: Each newly dedupe-guarded drain follows the
  `C2SAcknowledgeResult` precedent: on duplicate detection, the
  drain logs at DEBUG level and returns early without side effect.
- Required: The dedupe key is `(round, canonical-message-key)`
  where `canonical-message-key` is reproducible across reconnect.
- Required: Dedupe state is session-scoped and cleared on
  `OnExit(InSession)` (or equivalent).
- Required: Each newly dedupe-guarded drain has a new integration
  test asserting duplicate is a no-op.
- Required: No protocol-shape change.
- Forbidden: Adding optimistic client-side authority.
- Forbidden: Modifying server-side reconnect logic.
- Forbidden: Editing Sprint 12 story files or evidence paths.

---

## Story Classification

**Story type**: Integration -- per-drain dedupe state + per-drain
integration tests.

This is **NOT** a:

- Protocol-shape change (covered by Sprint 14 candidate
  `S13-PROTO-MESSAGE-ID-001`).
- Server-side change.
- Sprint 12 expansion.

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- `S2CGameOver` dedupe-guarded**: GIVEN the diff under
  `client/src/presentation/result_screen.rs` (and any
  presentation drain location), WHEN the `S2CGameOver` drain is
  inspected, THEN it consults a `(round, key)` dedupe set; on
  duplicate, it logs DEBUG and returns. The duplicate path is
  covered by a new integration test.

- [x] **AC2 -- `S2CClassLocked` dedupe-guarded**: same for
  `client/src/ui/lobby.rs` `S2CClassLocked` drain (currently at
  `:326-335`).

- [x] **AC3 -- `S2CPlaceUnit` / placement-class drains
  dedupe-guarded**: same for the placement-reveal drains in
  `client/src/ui/hand/*` and/or `client/src/presentation/board_rendering.rs`.
  Implementation prompt enumerates the exact placement-class
  messages and dedupe-guards each.

- [x] **AC4 -- Reconnect-replay test**: GIVEN a new integration
  test (or extension of existing reconnect tests under
  `tests/integration/shop_auction_ui/reconnect_late_message_test.rs`
  or `tests/integration/session/`), WHEN the test sends a
  duplicate `S2CGameOver` after a reconnect re-send, THEN the
  client's result-screen sequence runs exactly once (no double-
  apply, no double-animation, no double-event emission).

- [x] **AC5 -- Dedupe set is session-lifetime-scoped**: GIVEN the
  diff, WHEN the dedupe state lifecycle is inspected, THEN the
  set is constructed at session entry and cleared at session
  exit. The reconnect path preserves the session and thus
  preserves the set.

- [x] **AC6 -- Bounded size**: GIVEN the diff, WHEN the dedupe
  set's growth policy is inspected, THEN it is bounded (e.g.,
  last 32 keys per type). The bound is documented inline.

- [x] **AC7 -- No protocol-shape change**: GIVEN the diff under
  `shared/src/protocol.rs`, WHEN inspected, THEN no C2S/S2C
  message type definition is changed (the message-id field is
  scoped to Sprint 14 candidate `S13-PROTO-MESSAGE-ID-001`).

- [x] **AC8 -- No server-side change**: GIVEN the diff under
  `server/`, WHEN inspected, THEN no functional behaviour change
  lands. Server-side reconnect logic is unchanged.

- [x] **AC9 -- No optimistic client-side authority introduced**:
  GIVEN the implementation diff, WHEN reviewed for any
  client-side mutation of authoritative state outside the
  shared phase sink, snapshot drainers, and S2C consumers,
  THEN no such mutation is present. ADR-002 binding.
  *Evidence*: text search for "no optimistic" in the evidence
  document.

- [x] **AC10 -- Workspace test pass + ignored count behave
  predictably**: GIVEN `cargo test --workspace --tests
  --no-fail-fast` at the implementation commit, WHEN compared to
  the post-Sprint-12 baseline, THEN no new `#[ignore]` markers
  are introduced; the new dedupe tests pass; previously-passing
  tests continue to pass.

- [x] **AC11 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  and `production/qa/qa-plan-sprint-12.md` are diffed, THEN
  none of them are modified under this story.

- [x] **AC12 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md`
  (NEW). Records per-message dedupe diff summary, new
  integration-test pass evidence, dedupe key construction
  rationale, no-claim restatement, cross-link to PROMPT 803 §3
  DC-6 + §4 Lane C.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/presentation/result_screen.rs` | `S2CGameOver` drain dedupe-guarded. |
| `client/src/ui/lobby.rs` | `S2CClassLocked` drain dedupe-guarded. |
| `client/src/ui/hand/*.rs` | Placement-class drains dedupe-guarded. |
| `client/src/presentation/board_rendering.rs` | Placement-class drains dedupe-guarded. |
| `client/src/state/mod.rs` or new `client/src/idempotency.rs` | NEW resource (`ClientIdempotencyState`) or per-drain dedupe state. |
| `tests/integration/session/late_msg_dedupe_test.rs` (or extension of existing test files) | NEW integration tests asserting duplicate-no-op per drain. |
| `tests/integration/shop_auction_ui/reconnect_late_message_test.rs` | Extended to cover the new dedupe path (or unchanged if a new test file is preferred). |
| `production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md` | NEW evidence document per AC12. |
| This story file | Status updates per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code edits.
- **`liv-bevy-lightyear`** -- mandatory for protocol / drain code
  edits.

---

## Evidence Path

`production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Per-message dedupe diff summary.
- Dedupe key construction rationale (e.g., "we used `(round,
  blake3-hash-of-message-bytes)` because the reliable channel
  guarantees byte-identity on replay").
- New integration-test pass evidence per drain.
- `cargo test --workspace --tests --no-fail-fast` pre/post output.
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no optimistic client-side authority").
- Cross-link to PROMPT 803 §3 DC-6 + §4 Lane C and to
  `tests/integration/session/result_acknowledgement_contract_test.rs:91-96`
  (the precedent test).

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test late_msg_dedupe -- --nocapture`
  (or the new test name)
- `cargo test -p client --test reconnect_late_message -- --nocapture`
  (or the canonical reconnect test file name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'client/src/**' 'tests/**' 'shared/src/**' 'server/src/**'`
  (verifies AC7 + AC8: zero protocol-shape change; zero server-side
  change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Adding a message-id sequence-num field to the protocol**.
  Scoped to Sprint 14 candidate `S13-PROTO-MESSAGE-ID-001`.
- **Server-side dedupe** (server is already authoritative; no
  server-side dedupe is needed). ADR-002 binding.
- **Phase-idempotency on `S2CPhaseChanged`** (DC-5). Scoped to
  Sprint 13 candidate `S13-PHASE-IDEMPOTENCY-CLIENT-001` (per
  PROMPT 803 §5 Should).
- **Sprint 13 activation**.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  or any carried Sprint condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Touches `client/src/ui/lobby.rs`** (`S2CClassLocked` drain
  dedupe). Sprint 12 Story 013 (lobby ConfirmClass intent chain)
  also touches `client/src/ui/lobby.rs`. **POTENTIAL CONFLICT**.
  Mitigation: this Sprint 13 story MUST NOT run in parallel with
  Sprint 12 Story 013; sequence after Sprint 12 close-out.
- **Touches `client/src/presentation/result_screen.rs`** -- not
  touched by Sprint 12 Must Have rows.
- **Touches `client/src/ui/hand/*`** -- Sprint 12 Story 019
  (drag-runtime tighter-capture) reads these files but doesn't
  modify them (evidence-only story). Sequence still
  Sprint-12-closes-first to keep the diff clean.
- **Touches `client/src/presentation/board_rendering.rs`** --
  Sprint 12 Story 014 (cooccupancy panic guard) touches this file.
  **POTENTIAL CONFLICT**. Mitigation: sequence after Sprint 12
  close-out.
- **No Sprint 12 invasion**: this story's implementation MUST NOT
  land before Sprint 12 close-out unless the producer explicitly
  authorises a pull-forward via a separate prompt.
- **Coordinate with `S13-PROTO-ORPHAN-DRAIN-001` (Story 008 in
  `lightyear-protocol-verification`)**: drains added by that
  story should follow the dedupe pattern established here.
- **No shared-status writer overlap**: `production/sprint-status.yaml`
  is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out.
2. Sprint 13 is planned via `/sprint-plan sprint-13`.
3. This story passes `/story-readiness`.
4. Sprint 13 `/qa-plan sprint` is authored.
5. `/dev-story story-020-late-msg-dedupe.md` is dispatched.

Expected implementation flow:

1. **Wave 1 -- Dedupe-key design**: implementation prompt decides
   the canonical key shape (e.g., `(round, message-type, hash)`).
   Documented in the evidence doc.
2. **Wave 2 -- Resource + per-drain edits**: introduce
   `ClientIdempotencyState` resource (or per-drain state); add
   dedupe check to each of the three message-class drains.
3. **Wave 3 -- Integration tests**: one new test per drain
   asserting duplicate-no-op, following the
   `result_acknowledgement_contract_test.rs:91-96` precedent.
4. **Wave 4 -- Reconnect-preservation test**: assert that the
   dedupe set is preserved across reconnect (i.e., reconnect-
   replay duplicates are detected).
5. **Wave 5 -- Evidence**: populate evidence file.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Lobby.rs collision with Sprint 12 Story 013 | High | High | Sequence: Sprint 12 closes first. |
| `board_rendering.rs` collision with Sprint 12 Story 014 | High | High | Sequence: Sprint 12 closes first. |
| Dedupe key collision (false positives) | Medium | Medium | Default key shape includes content hash; if false positives observed, refine to include a true sequence-id (defers to Sprint 14 candidate `S13-PROTO-MESSAGE-ID-001`). |
| Dedupe set memory growth in long sessions | Low | Low | AC6 bounded size. |
| Server-side reconnect change later breaks dedupe semantics | Low | Medium | AC8 forbids server-side change; future server-side changes must coordinate with this story's pattern. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate. |

---

## Verification (orchestrator-side, before worker dispatch)

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation; Sprint 12 close-out has landed.
- Sprint 12 Stories 013 (lobby) and 014 (board-rendering) are
  `done`.
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for Client-Side Late-Message Idempotency per PROMPT 803
  §3 DC-6 / §5 Should row 1. Sprint 12 is `active` (PROMPT 798) and
  is not modified by this authoring run. No code changes, no
  smoke / gate / QA / `/dev-story` / `/story-done` / `/story-readiness` /
  `/qa-plan` run. Source-of-truth at authoring: `origin/main@b5eef0d`.
  Worker branch: `work/s13-runtime-hardening-story-authoring`.
  Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.

## Closure Trail

- 2026-05-14 -- PROMPT 872 -- Worker implementation on
  `work/s13-late-msg-dedupe` from base `origin/main@3cf5e41`. Worker
  commit `dfe5f217aa99b4b5cd3c995a71db2c74f15a1135` adds session-
  lifetime dedupe rings on the three reliable S2C drains
  (`S2CGameOver` -> `apply_game_over_drain`; `S2CClassLocked` ->
  `apply_class_locked_drain`; `S2CPlacementReveal` ->
  `filter_placement_reveal_for_dedupe`) backed by new resource
  `ClientIdempotencyState` (3x `DedupeRing<K>`, `DEDUPE_BOUND = 32`
  per ring, oldest-key eviction), new plugin `ClientIdempotencyPlugin`
  (installs resource + registers `OnExit(ClientState::InSession)`
  clear), and new integration test
  `tests/integration/session/late_msg_dedupe_test.rs` (17 tests).
  9 files changed (+1095 / -28). No protocol-shape change, no
  server-side change, no optimistic client-side authority. Cargo
  resource policy applied (`CARGO_TARGET_DIR=D:\\_DEV\\cargo-target\\ccgs-msvc`).
  Targeted regression: `cargo fmt -p client -- --check` clean +
  `cargo check -p client` clean + `cargo test -p client --test
  late_msg_dedupe_test` 17/17 pass + `cargo test -p client --lib
  state::idempotency` 7/7 pass. Full-workspace tests intentionally
  NOT run per Sprint 13 QA-plan binding no-full-workspace-tests-by-
  default policy + worker prompt scope.

- 2026-05-14 -- PROMPT 883 -- Integration. `--no-ff` merge of worker
  tip `dfe5f21` into prior `origin/main@c379625` (post-PROMPT 876)
  producing merge commit
  `6163cd30d31821aa178444c48b854f086c97f4f0` on origin/main; zero
  conflicts. First integration attempt `adf429d` discarded after
  origin/main advanced under PROMPT 882 + PROMPT 876 mid-run; fresh
  worktree on new tip re-merged cleanly. Merge-introduced delta
  matches worker stat exactly (9 files / +1095 / -28). Forbidden
  paths (`production/sprint-status.yaml`,
  `production/session-state/`, `production/stage.txt`) untouched by
  worker + integration. AC1-AC12 evidence recorded at
  `production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md`
  + `reports/PROMPT-883-S13-Late-Msg-Dedupe-Integration.md`. No
  `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check` run.

- 2026-05-14 -- PROMPT 884 -- `/story-done` paperwork closure.
  Verdict PASS. Source-of-truth at closure:
  `origin/main@6163cd30d31821aa178444c48b854f086c97f4f0`. AC1-AC9
  + AC11 + AC12 all PASS verified against the integrated diff:
  AC1 PASS (`apply_game_over_drain` at
  `client/src/presentation/result_screen.rs:693,705` consults
  `ClientIdempotencyState::game_over` + `S2CGameOver` integration
  tests `s2c_game_over_drain_first_apply_caches_then_duplicate_is_noop`
  + `s2c_game_over_drain_consults_dedupe_ring_in_production_source`);
  AC2 PASS (`apply_class_locked_drain` at
  `client/src/ui/lobby.rs:335,405` consults
  `ClientIdempotencyState::class_locked` + tests
  `s2c_class_locked_drain_first_apply_locks_then_duplicate_is_noop`
  + `s2c_class_locked_drain_consults_dedupe_ring_in_production_source`);
  AC3 PASS (`filter_placement_reveal_for_dedupe` at
  `client/src/presentation/board_rendering.rs:1072,1105` consults
  `ClientIdempotencyState::placement_reveal` + tests
  `s2c_placement_reveal_drain_first_apply_returns_message_then_duplicate_is_noop`
  + `s2c_placement_reveal_drain_consults_dedupe_ring_in_production_source`;
  story's "`S2CPlaceUnit`" naming maps to the protocol's
  `S2CPlacementReveal` placement-class S2C message);
  AC4 PASS (test
  `ac4_game_over_reconnect_replay_runs_result_screen_sequence_exactly_once`);
  AC5 PASS (`reset_client_idempotency_on_session_exit_system`
  wired to `OnExit(ClientState::InSession)` in
  `ClientIdempotencyPlugin::build` at
  `client/src/state/idempotency.rs:236,250` + tests
  `ac5_clear_for_session_exit_resets_all_drain_rings` +
  `ac5_session_exit_system_is_wired_to_on_exit_in_session`;
  reconnect path does not exit `InSession` so dedupe state is
  preserved across reconnect per ADR-011);
  AC6 PASS (`pub const DEDUPE_BOUND: usize = 32` documented
  inline at `client/src/state/idempotency.rs:50` + tests
  `ac6_dedupe_ring_evicts_oldest_when_bound_exceeded` +
  `ac6_dedupe_bound_documented_inline` +
  `state::idempotency::tests::dedupe_ring_evicts_oldest_when_bound_exceeded`);
  AC7 PASS (`git diff 6163cd3^1..6163cd3 -- 'shared/src/protocol.rs'`
  empty + test `ac7_no_new_message_id_field_in_protocol`);
  AC8 PASS (`git diff 6163cd3^1..6163cd3 -- 'server/'` empty);
  AC9 PASS (read-only dedupe projection; test
  `ac9_dedupe_state_is_a_read_only_projection_no_optimistic_authority`
  + "No optimistic client-side authority is introduced" phrase
  preserved verbatim in evidence doc);
  AC10 PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK (per PROMPT 872
  + Sprint 13 QA-plan no-full-workspace-tests-by-default policy:
  `cargo test -p client --test late_msg_dedupe_test` 17/17 pass +
  `cargo test -p client --lib state::idempotency` 7/7 pass + nearby
  regression set green; no new `#[ignore]` markers; full-workspace
  cargo test deferred to Sprint 13 end-of-sprint integration smoke);
  AC11 PASS (`git diff 6163cd3^1..6163cd3 -- 'production/sprint-status.yaml'
  'production/sprints/sprint-12.md' 'production/stage.txt'
  'production/qa/qa-plan-sprint-12.md'` empty across worker +
  integration; PROMPT 884 row-level flip is the permitted
  disposition-preserving paperwork edit);
  AC12 PASS (`production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md`
  NEW 138 lines on origin/main via PROMPT 883 integration; not
  modified by PROMPT 884). Expected worker report at
  `reports/PROMPT-872-S13-LATE-MSG-DEDUPE-001-Client-Side-Dedupe.md`
  is missing per PROMPT 883 §Worker artifacts; per PROMPT 884 task
  rubric the missing worker report is documented as non-blocking
  because the integration report + worker commit message body +
  evidence document collectively cover all twelve ACs. Paperwork-
  only run: no /dev-story, no /story-readiness, no /smoke-check,
  no /team-qa, no /gate-check, no /release-check, no /qa-plan, no
  Sprint 13 close-out, no stage advance, no `S8-QA-001-W1` closure
  invoked by PROMPT 884. No Cargo invoked by PROMPT 884; Cargo
  resource policy N/A for the closure run itself (PROMPT 872 worker
  applied the binding Windows/MSVC Cargo resource policy for its
  targeted regression invocations; PROMPT 883 integration did not
  invoke cargo for the merge-only operation). No client/, server/,
  shared/, tests/ touched by PROMPT 884. No production/stage.txt,
  production/sprints/sprint-13.md, production/sprints/sprint-12.md,
  production/qa/qa-plan-sprint-13.md, production/qa/qa-plan-sprint-12.md,
  production/qa/evidence/sprint-13-late-msg-dedupe-evidence.md, or
  production/gate-checks/* touched by PROMPT 884. All carried
  non-claims preserved: `S8-QA-001-W1` OPEN, `QA-COND-0005` +
  `QA-COND-0006` accepted-risk, `PAW-TD-*-a` accept-risk,
  PROMPT 683-era runtime divergence question (folded into Sprint 12
  story 019 cannot-reproduce closure; third same-scope retest NOT
  authorised per TQ-S12-C2), Story 019 (Sprint 12 hand-ui) underlying
  drag-runtime bug NOT claimed fixed, `TQ-S12-C1..C7` verbatim,
  Sprint 12 / Sprint 11 / Sprint 10 closeouts unchanged. Sprint 13
  progress after PROMPT 884: 6 of 6 Must Have done (track COMPLETE
  per PROMPT 871); **4 of 6 Should Have done** (this row + the
  three prior Should Have closures via PROMPT 833 + 844 + and the
  PROMPT 835 inline closure); 5 of 7 Nice to Have done; **total
  14 of 19** rows closed. Sprint 13 disposition UNCHANGED `active`.
  Stage UNCHANGED `Polish`. PROMPT 761 Polish->Release gate-check
  FAIL preserved. No connection-lost UX implementation by PROMPT
  884 (forbidden per task scope; S13-CONN-LOST-UX-001 row remains
  `ready`).
