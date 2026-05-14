# Story 007: S13-PROTO-INVARIANT-001 -- Protocol Completeness Invariant Test

> **Epic**: Lightyear Protocol & Verification Spike
> **Story ID**: S13-PROTO-INVARIANT-001
> **Status**: Done -- closed by PROMPT 851 `/story-done` on
> `origin/main@c1b7753` (worker `96c1600` PROMPT 845 on
> `work/s13-protocol-completeness-invariant` from base
> `origin/main@fe74fb0` + integration merge commit `25573e6` PROMPT 849
> fast-forward push to `origin/main`; subsequent PROMPT 850 closure
> commit `c1b7753` for the sibling Must Have row
> `S13-OBS-TRACING-TARGETS-001` lands after PROMPT 849 and does not
> modify story-007 scope). AC1-AC11 all satisfied per worker +
> integration evidence (see Closure Trail below).
> **Layer**: Test Infrastructure / Invariant Gate
> **Type**: Logic (workspace invariant test) -- no production-code change
> **Sprint**: Sprint 13 active (activated by PROMPT 826; Must Have row)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.
> **Closure source-of-truth**: `origin/main@c1b7753` (PROMPT 850
> closure commit on top of PROMPT 849 integration merge `25573e6`
> which fast-forward-pushed PROMPT 845 worker commit `96c1600`
> together with PROMPT 847 unrelated `9e32fbe`).

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint (`status:
active` per `production/sprint-status.yaml` at `origin/main@b5eef0d`) and
must not be changed by this authoring run. Activation of Sprint 13
happens via a separate `/sprint-plan sprint-13` prompt after Sprint 12
close-out.

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
story.** The invariant test is read-only over `shared/src/protocol.rs`
and over `client/` / `server/` source; it does not mutate or change any
protocol shape or any client/server message-handling behaviour. ADR-002
+ ADR-008 binding for any follow-on implementation that lands the drain
or sender additions discovered by this test (those land under separate
Sprint 13 stories -- see `S13-PROTO-ORPHAN-DRAIN-001`).

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-1** Protocol-registered-but-orphan-on-receive (HIGH): 8 S2C
  messages defined+registered but with NO client `MessageReceiver`
  drain; 1 C2S with no server handler stub. Evidence anchors:
  `shared/src/protocol.rs:71,81,82,85,92,93,103,105,107`;
  `server/src/main.rs:138-143` TODO.
- **§3 DC-2** Plugin-defined-but-not-added (MED): no current orphans
  found; risk surface remains because there is no static check. (This
  story's invariant test optionally extends to plugin registration in a
  follow-on test; see "Out of Scope" below.)
- **§3 DC-15** CI gate weakness: no static check enforces (a) every
  protocol message has both a sender and a drain, (b) every plugin
  defined is added, (c) `#[should_panic]` and `#[ignore]` attribute
  counts are stable. Each defect class above is detectable only by
  ad-hoc audit.
- **§4 Lane A "Existing tests that should have caught"**: a "protocol
  completeness" invariant test -- does not exist.
- **§5 Must row 1 (S13-PROTO-INVARIANT-001)**: "Protocol completeness
  invariant test (every defined C2S/S2C has >=1 send-site and >=1
  drain-site)" -- new test under `tests/invariants/` (NEW directory).
- **§6 PROMPT-N+1 dispatch slot**: paperwork-only story-authoring for
  this row; parallel-safe with the other Sprint 13 candidate stories
  because it touches a disjoint epic / story-file path.

---

## Problem Class / Prevention Target

**Defect class (DC-1 + DC-15)**: A message type is added to
`shared/src/protocol.rs` and registered via `register_protocol(app)`
(or equivalent), but no production code path on the receiving side
ever drains it (`MessageReceiver<T>` is missing) or, in the C2S
direction, no production code path on the server ever receives /
handles it. The compiler does not detect this -- the message
serialises fine and the channel binding is fine; the drain just never
happens. Symptoms are silent data loss (S2C orphans) or a silent C2S
no-op (server-side orphan).

**Prevention target**: A workspace test that fails if any C2S or S2C
message type defined in `shared/src/protocol.rs` lacks either:

- at least one `MessageSender<T>` reference (sender side), AND
- at least one `MessageReceiver<T>` reference (receiver side),

across the workspace's production code (i.e., excluding `tests/` and
`#[cfg(test)]`-gated code). The test must be deterministic and run as
part of the standard `cargo test --workspace` invocation so that CI
catches recurrences before merge.

The test must surface, for each violation, the exact message type name,
the file:line where the type is declared, and an actionable hint
("missing client-side drain" vs "missing server-side handler"). The
test must be readable at-a-glance so that adding a new message type
prompts a clear "add drain" / "add handler" instruction without
requiring the author to reread this story.

---

## Context

### Existing surface

- `shared/src/protocol.rs` defines all C2S* and S2C* message types and
  the `register_protocol(app)` function that registers them and their
  channel bindings (per ADR-008 reliable / unreliable split).
- `client/src/network/` houses client-side `MessageSender<C2S*>` and
  `MessageReceiver<S2C*>` system params (per
  `production/epics/lightyear-protocol-verification/EPIC.md` Story 003).
- `server/src/network/` houses the mirror set: `MessageReceiver<C2S*>`
  handlers and `MessageSender<S2C*>` broadcasts.
- Today no workspace test enforces that every message defined in
  `shared/src/protocol.rs` has both a `MessageSender<T>` reference (on
  the sending side) and a `MessageReceiver<T>` reference (on the
  receiving side). The 8 S2C orphans and the 1 C2S orphan found in
  PROMPT 803 §3 DC-1 are detectable only by manual audit today.

### Test-evidence path precedent

- The Lightyear verification evidence at
  `tests/evidence/lightyear-026-verification.md` is the precedent for
  doc-and-test evidence in this epic.
- `tests/integration/network/` already contains protocol-level
  integration tests (e.g., `os18b_two_client_objective_hp_visibility_test.rs`).
- A new `tests/invariants/` directory does not exist on `origin/main`
  at `b5eef0d` -- this story creates it (test-only path; no production
  code touched).

### GDD / ADR / TR trace

- **GDD**: `design/gdd/network-protocol.md` Table A defines the
  C2S/S2C message inventory. No GDD change is required by this story.
- **ADR-003** (Cargo Workspace Structure): `shared/` is the single
  registration site; both client and server consume the same protocol
  module. The invariant test reads from `shared/src/protocol.rs`
  directly.
- **ADR-008** (Lightyear Channel Config): `ReliableChannel` owns all
  game-state messages; `UnreliableChannel` carries heartbeat + auction
  timer ticks. The invariant test must tolerate the unreliable channel
  membership for `C2SHeartbeat` / `S2CHeartbeat` (i.e., these still
  need senders and drains; channel choice is independent of the
  send/drain symmetry property).
- **TR registry**: the invariant gates TR-NP-SYMM ("Server and client
  use identical message type definitions from `shared/`; divergence is
  a compile error") in spirit -- divergence in routing (defined but
  not drained) is not currently a compile error; this test makes it a
  test-time error.

### Engine

- **Engine**: Bevy 0.18 (Rust). The test is a pure Rust workspace test;
  no Bevy `App::run()` is required. Optional: the test may construct an
  `App` with `register_protocol(app)` invoked to verify channel
  binding parity, but this is not the primary mechanism.
- **Lightyear**: 0.26 (Bevy 0.18 compatible). The test inspects
  `MessageSender<T>` / `MessageReceiver<T>` system-param references
  via source-text grep, not via runtime Lightyear introspection.

### Mandatory skills

- **`liv-bevy-018`** -- this is a Bevy `.rs` test file. All test code
  follows Bevy 0.18 idioms (no pre-0.15 `Bundle` patterns; no
  `apply_deferred`; use `App::update()` if any `App` is constructed).
- **`liv-bevy-lightyear`** -- the test reads the Lightyear protocol
  manifest in `shared/src/protocol.rs` and the lightyear 0.26 API
  shapes (`MessageSender<T>`, `MessageReceiver<T>`,
  `register_message::<T>(...)` registration calls). The Lightyear 0.26
  API is post-training-cutoff; the implementing worker must
  cross-reference `docs/engine-reference/bevy/VERSION.md` and the
  Lightyear release notes before writing the grep / reflection logic.

### Control Manifest Rules (Foundation, test-scope)

- Required: The test is deterministic; it does not depend on test
  execution order, random seeds, or external resources.
- Required: The test runs under `cargo test --workspace`; no special
  feature flag is needed to enable it.
- Required: Each violation is reported individually with file:line of
  the message-type declaration site and a one-line actionable hint.
- Required: The test treats `C2SHeartbeat` (UnreliableChannel) and
  `S2CHeartbeat` (UnreliableChannel) identically to the
  ReliableChannel messages: both must have at least one send-site and
  one drain-site reference. Channel membership is not exempt.
- Forbidden: Modifying `shared/src/protocol.rs`, `client/src/network/`,
  `server/src/network/`, or any other production code to make the test
  pass -- the test's purpose is to surface today's 8+1 orphans;
  removing the orphans is the job of `S13-PROTO-ORPHAN-DRAIN-001`.
- Forbidden: Quieting the test with allowlists for known orphans
  without an inline rationale comment + a follow-on issue or story
  cross-reference. The default disposition is "fail loudly".

---

## Story Classification

**Story type**: Logic -- workspace invariant test. The test is the only
deliverable; no production-code change lands.

This is **NOT** a:

- Repair story (orphans found by this test are repaired under
  `S13-PROTO-ORPHAN-DRAIN-001`).
- Refactor story (no rewrite of protocol or network code).
- Documentation-only story (a real test file lands).

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Test file exists at the canonical path**:
  `tests/invariants/protocol_completeness_test.rs` exists on the
  implementation branch. The file is registered as a workspace test
  (i.e., the `tests/` runner finds it; if a custom `Cargo.toml`
  `[[test]]` block is needed because `tests/invariants/` is a new
  subdirectory, it lands in the workspace Cargo.toml under
  scope-capped controls -- see AC8).
  **Closure evidence (PROMPT 851)**: file `tests/invariants/protocol_completeness_test.rs`
  exists on `origin/main` at `25573e6` (421 lines; landed via PROMPT 845
  worker commit `96c1600`). Registered as `[[test]] name =
  "protocol_completeness_invariant"` in `shared/Cargo.toml` (the
  scope-capped block authored per AC8 with an inline rationale comment
  cross-referencing `S13-PROTO-ORPHAN-DRAIN-001`). PASS.

- [x] **AC2 -- Test enumerates every C2S* and S2C* message type from
  `shared/src/protocol.rs`**: The test discovers the message-type
  inventory by parsing the source file (via `include_str!` + token-
  scanning) OR by reading a single canonical list constant exposed
  from `shared/src/protocol.rs` for this purpose. The discovered
  inventory matches the human-readable list in
  `design/gdd/network-protocol.md` Table A modulo any documented
  deviations.
  **Closure evidence (PROMPT 851)**: parsing approach (a) chosen --
  `const PROTOCOL_SOURCE: &str = include_str!("../../shared/src/protocol.rs");`
  plus a token scan for `register_c2s::<...>` / `register_s2c::<...>`
  lines (see `discover_registered_messages()` and `find_decl_line()`).
  Companion enabled test `protocol_manifest_parser_discovers_registered_messages`
  asserts the floors `c2s >= 16` and `s2c >= 34` (50 types
  discovered at closure tip). Per evidence doc §"What landed"
  bullet 1.2: every registered name has a `pub struct` declaration
  in `shared/src/protocol.rs`. PASS.

- [x] **AC3 -- For every message type, the test verifies at least one
  send-site reference**: For C2S types, the test searches the
  workspace source for `MessageSender<C2SX>` references in production
  code under `client/src/`. For S2C types, the test searches for
  `MessageSender<S2CX>` references under `server/src/`. The search
  excludes `tests/` and `#[cfg(test)]`-gated blocks.
  **Closure evidence (PROMPT 851)**: `has_send_site()` in the test
  file looks for either `MessageSender<T>` SystemParam OR the
  canonical `send::<T,` broadcast-call shape on
  `ServerMultiMessageSender`. Per-file alias scan via
  `extract_aliases_in()` follows `T as Alias` imports so that
  `server/src/network/economy_dispatch.rs`
  (`S2CGoldBroadcast as ProtocolGoldBroadcast`) and
  `server/src/feature/auction/system.rs`
  (`S2CAuctionCard as ProtocolS2CAuctionCard`) are correctly counted.
  `read_stripped()` invokes `strip_cfg_test_blocks()` to erase
  brace-balanced `#[cfg(test)] mod ... { ... }` blocks before
  scanning; files under `tests/` are not scanned (only `client/src/`
  and `server/src/`). PASS.

- [x] **AC4 -- For every message type, the test verifies at least one
  drain-site reference**: For C2S types, the test searches the
  workspace source for `MessageReceiver<C2SX>` references in
  production code under `server/src/`. For S2C types, the test
  searches for `MessageReceiver<S2CX>` references under
  `client/src/`. The search excludes `tests/` and
  `#[cfg(test)]`-gated blocks.
  **Closure evidence (PROMPT 851)**: `has_drain_site()` in the test
  file looks for `MessageReceiver<T>` SystemParam references with
  the same alias-following machinery used by AC3. C2S types are
  searched in `server_files`; S2C types in `client_files`. `#[cfg(test)]`
  blocks erased via `strip_cfg_test_blocks()`; `tests/` not scanned.
  AC4 unreliable-channel non-exemption is honoured: `C2SHeartbeat`
  and `S2CHeartbeat` are required to have send + drain sites even
  though they ride `UnreliableChannel` (see evidence doc §"Test
  design summary" bullet 6). PASS.

- [x] **AC5 -- Violation report is actionable**: When the test fails,
  the failure message lists every violating message type, the
  `shared/src/protocol.rs:LINE` of its declaration, the side that
  lacks the reference (client or server, send or drain), and a
  one-line remediation hint ("add MessageReceiver<S2CFooBar> drain in
  client/src/..." or "add MessageSender<C2SFoo> in client/src/..." or
  "delete from protocol with rationale").
  **Closure evidence (PROMPT 851)**: per evidence doc §"Pre-
  `S13-PROTO-ORPHAN-DRAIN-001` test output (verbatim, PROMPT 845)",
  every violating entry prints
  `<MessageType>  (shared/src/protocol.rs:LINE)\n    missing-side:
  <one-line remediation hint>` and the report's preamble cross-links
  `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §3 DC-1, §3 DC-15, §4 Lane A. Hint text differentiates "missing
  client-side send-site", "missing client-side drain", "missing
  server-side send-site", "missing server-side drain" and each
  variant proposes either the canonical SystemParam / call-site
  addition OR `delete the type from the protocol with a rationale`.
  PASS.

- [x] **AC6 -- Test surfaces today's PROMPT 803 §4 Lane A orphans**:
  Run against `origin/main` at the implementation commit's parent
  (i.e., before `S13-PROTO-ORPHAN-DRAIN-001` lands), the test FAILS
  with at minimum the 8 S2C orphans (`S2CHeartbeat`,
  `S2COpponentDisconnected`, `S2COpponentReconnected`,
  `S2CPoolUpdate`, `S2CPrismRespawned`, `S2CPrismRewardDropped`,
  `S2CSangMepriseReveal`, `S2CSessionCancelled`) AND the 1 C2S orphan
  (`C2SRequestSnapshot`) listed in
  `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §4 Lane A. The implementation prompt's evidence file records the
  test output verbatim. The test is allowed to additionally surface
  any other orphans discovered at implementation time; the 9 named
  orphans are a floor, not a ceiling.
  **Closure evidence (PROMPT 851)**: per evidence doc §"Cross-check
  vs PROMPT 803 §4 Lane A '9 named orphans'", all 9 named orphans
  surfaced (table maps each PROMPT 803 §4 row to the PROMPT 845
  test output and records "PASS" for each). One additional orphan
  surfaced -- `C2SClassChoice` (`shared/src/protocol.rs:421`) --
  allowed by the "floor, not ceiling" clause; client lobby uses
  `C2SSelectClass` / `C2SConfirmClass` instead and the disposition
  (drain / send / delete) rolls into `S13-PROTO-ORPHAN-DRAIN-001`.
  PROMPT 849 integration verification re-ran the ignored invariant
  test at the integration tip and got the identical 13-violation
  output across 10 unique message types (per `reports/PROMPT-849-S13-PROTOCOL-COMPLETENESS-INVARIANT-INTEGRATION.md`
  "Invariant / orphan caveats preserved" section). PASS.

- [x] **AC7 -- Test does not modify any production code**: The
  implementation commit's diff under `client/src/**`, `server/src/**`,
  and `shared/src/**` is empty (modulo a single canonical
  message-name-list constant exposed from `shared/src/protocol.rs`
  IF AC2's parsing approach requires it; that constant addition is
  the only allowed production-side change and is scope-capped to
  declarative metadata).
  **Closure evidence (PROMPT 851)**: `git diff --name-only 96c1600^1
  96c1600` returns exactly three paths:
  `production/qa/evidence/sprint-13-proto-invariant-evidence.md`,
  `shared/Cargo.toml`, and `tests/invariants/protocol_completeness_test.rs`.
  No file under `client/src/**`, `server/src/**`, or `shared/src/**`
  modified -- AC2's parsing approach was (a) `include_str!` source
  scanning, so no name-list constant was added; the only `shared/*`
  edit is `shared/Cargo.toml` `[[test]]` registration (scope-capped
  per AC8). PASS.

- [x] **AC8 -- Test runs under `cargo test --workspace`**: After the
  implementation lands, `cargo test --workspace --tests --no-fail-fast`
  invokes the new test (verified by grepping the test runner output
  for the test function name). If `Cargo.toml` workspace member
  registration is required for the new `tests/invariants/`
  subdirectory, the change is scope-capped to a single `[[test]]`
  block or `tests/invariants/Cargo.toml` shim with a rationale comment
  cross-referencing this story.
  **Closure evidence (PROMPT 851)**: `shared/Cargo.toml` carries a
  single new `[[test]] name = "protocol_completeness_invariant"
  path = "../tests/invariants/protocol_completeness_test.rs"` block
  with an inline rationale comment cross-referencing
  `S13-PROTO-ORPHAN-DRAIN-001` and pointing at the
  `production/qa/evidence/sprint-13-proto-invariant-evidence.md`
  orphan list. Worker (PROMPT 845) ran `cargo test -p shared --test
  protocol_completeness_invariant -- --nocapture` -- exit 0, 1
  passed (`protocol_manifest_parser_discovers_registered_messages`),
  1 ignored (`protocol_completeness_assert_send_and_drain_sites`).
  Integration (PROMPT 849) re-ran the same command at integration
  tip with identical output (per integration report row 5 +
  row 11). Full-workspace `cargo test --workspace --tests --no-fail-fast`
  intentionally NOT run per QA-plan-sprint-13 binding
  no-full-workspace-tests-by-default policy (orchestrator
  end-of-sprint integration gate covers full workspace). PASS.

- [x] **AC9 -- No optimistic client-side authority introduced**: The
  test reads source files; it does not call any client/server
  runtime API. ADR-002 binding. *Evidence*: text search for "no
  optimistic" in the evidence document.
  **Closure evidence (PROMPT 851)**: test file imports are limited
  to `std::fs` + `std::path::{Path, PathBuf}` (per
  `tests/invariants/protocol_completeness_test.rs:37-38`). Zero
  `bevy` or `lightyear` crate imports. No Bevy `App` is constructed
  and no Lightyear runtime API is called. Evidence doc carries the
  verbatim "**No optimistic client-side authority is introduced or
  proposed by this story.**" line in §"Status / No-Claim Banner
  (verbatim from story 007)" and the verbatim restatement "no
  optimistic client-side authority is introduced or implied" in
  §"ADR / GDD / protocol surfaces -- no change". ADR-002 + ADR-008
  binding preserved. PASS.

- [x] **AC10 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`, and
  `production/qa/qa-plan-sprint-12.md` are diffed, THEN none of them
  are modified under this story. Sprint 12 activation disposition is
  preserved. Stage remains `Polish`. Sprint 11 disposition
  (`closed-with-conditions`) is unchanged. Sprint 10 disposition
  (`closed-with-conditions`) is unchanged.
  **Closure evidence (PROMPT 851)**: `git diff --name-only 96c1600^1
  96c1600` does NOT include any of `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, `production/qa/qa-plan-sprint-12.md`, or
  `production/qa/qa-plan-sprint-13.md`. Sprint 12
  `closed-with-conditions` per PROMPT 817 preserved. Sprint 11
  `closed-with-conditions` per PROMPT 792 preserved. Sprint 10
  `closed-with-conditions` per PROMPT 763 preserved. Stage UNCHANGED
  `Polish`. PROMPT 761 Polish->Release gate-check `FAIL` at
  `production/gate-checks/gate-polish-release-2026-05-12.md`
  preserved. (Note: PROMPT 845 / 849 ran during Sprint 13's active
  window; the AC10 preservation extends to Sprint 13's `sprint:` /
  `status:` / `stage:` top-level fields as well -- none touched by
  the integration commit.) The PROMPT 851 row-level
  `status: ready -> done` + `completed: 2026-05-14` flip in
  `production/sprint-status.yaml` is the permitted disposition-
  preserving paperwork edit. PASS.

- [x] **AC11 -- Evidence document slot reserved**: A slot is reserved
  at `production/qa/evidence/sprint-13-proto-invariant-evidence.md`
  (NEW; populated by the implementation prompt). The evidence file
  records pre-`S13-PROTO-ORPHAN-DRAIN-001` test output (FAIL with the
  9 named orphans) and a re-run after the drain story lands (PASS
  except for any explicit allowlist with rationale).
  **Closure evidence (PROMPT 851)**: evidence document exists NEW
  on `origin/main` via PROMPT 845 worker commit `96c1600` (then on
  origin/main via PROMPT 849 integration merge `25573e6`): 330 lines
  at `production/qa/evidence/sprint-13-proto-invariant-evidence.md`.
  Records: no-claim restatement (verbatim from story banner with
  the "no optimistic" phrase preserved in §"Status / No-Claim
  Banner"); pre-drain test output verbatim (13 violations across
  10 unique types; all 9 PROMPT 803 §4 Lane A named orphans
  surfaced plus 1 additional `C2SClassChoice`); regression commands
  actually run with Cargo resource policy applied; AC1-AC11
  sectioned evidence; cross-link to
  `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §3 DC-1, §3 DC-15, §4 Lane A; slot reserved §"Post-
  `S13-PROTO-ORPHAN-DRAIN-001` re-run (slot reserved)" for the
  PASS rerun that lands when the drain story removes the
  `#[ignore]` attribute. PASS.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `tests/invariants/protocol_completeness_test.rs` | NEW. Workspace invariant test that enumerates C2S/S2C messages from `shared/src/protocol.rs` and verifies send-site + drain-site references across `client/src/` and `server/src/` production code. |
| `tests/invariants/mod.rs` or `tests/invariants/Cargo.toml` (if needed) | NEW (only if required to register the new subdirectory as a workspace test target; scope-capped per AC8). |
| `shared/src/protocol.rs` (OPTIONAL, scope-capped) | OPTIONAL: if the test approach uses a canonical name-list constant rather than source parsing, that constant is added here (declarative metadata only, no behaviour change). |
| `Cargo.toml` (workspace) | OPTIONAL: `[[test]]` block adding the new `tests/invariants/` target, ONLY if the default test runner does not discover it automatically. |
| `production/qa/evidence/sprint-13-proto-invariant-evidence.md` | NEW. Evidence document with pre/post test output, no-claim restatement, cross-link to PROMPT 803 §4 Lane A. |
| This story file (decision-recording / status-update commits) | Per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for any read/review/edit of Bevy
  `.rs` code. The test file is `.rs` and uses Bevy idioms.
- **`liv-bevy-lightyear`** -- mandatory for protocol / network code
  reading. The test inspects `shared/src/protocol.rs` registration
  calls and grep-scans for `MessageSender<T>` / `MessageReceiver<T>`
  system-param references, which are Lightyear 0.26 API surfaces.

---

## Evidence Path

`production/qa/evidence/sprint-13-proto-invariant-evidence.md`
(NEW; populated by the implementation prompt).

**Required evidence content** (deferred to implementation prompt):

- Pre-implementation `cargo test --workspace --tests --no-fail-fast`
  output filtered to the new test (FAIL with the 9 named orphans from
  PROMPT 803 §4 Lane A; additional orphans recorded as discovered).
- Post-implementation re-run after `S13-PROTO-ORPHAN-DRAIN-001` lands
  (PASS or PASS-WITH-DOCUMENTED-ALLOWLIST).
- Test file diff summary (file count, line count, key assertions
  enumerated).
- No-claim restatement (verbatim from this story file's "Status /
  No-Claim Banner" section).
- Cross-link to
  `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §3 DC-1 + DC-15 and §4 Lane A.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test --workspace --tests --no-fail-fast 2>&1 | grep -i "protocol_completeness"`
  (verifies the new test is discovered and reports its outcome)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

The test must PASS-with-known-orphans-allowlist OR FAIL-with-actionable-
report after `S13-PROTO-ORPHAN-DRAIN-001` decisions land. Until that
story lands, the test FAILing with the 9 known orphans is the expected
result and is recorded as evidence (not as a CI gate failure -- the
Sprint 13 dispatch ordering can be `S13-PROTO-INVARIANT-001` lands first
with the test ignored or expected-fail, then `S13-PROTO-ORPHAN-DRAIN-001`
lands and the test flips to PASS; OR both land in the same Sprint 13
window with a single PR that resolves the orphans and lands the test
together).

---

## Out of Scope

- **Adding drains/senders for the 8+1 orphans named in PROMPT 803 §4
  Lane A**. That work lands under `S13-PROTO-ORPHAN-DRAIN-001` and is
  not scoped to this story.
- **Plugin registration invariant test** (DC-2). Scoped to a separate
  Sprint 14 Nice-to-Have row (`S13-PLUGIN-REGISTRATION-INVARIANT-001`
  per PROMPT 803 §5 Nice). The implementing worker MAY add a stub or
  TODO inside the new test file referencing the future plugin
  invariant, but the plugin check itself is out of scope here.
- **`#[ignore]` / `#[should_panic]` attribute-drift invariant test**
  (DC-15). Scoped to a separate Sprint 14 Nice-to-Have row
  (`S13-IGNORE-ATTRIBUTE-DRIFT-001` per PROMPT 803 §5 Nice).
- **Lightyear API surface compile-time verification**. The test uses
  source-text inspection, not runtime Lightyear API introspection.
- **No production-code change** beyond the optional scope-capped
  name-list constant in `shared/src/protocol.rs` (AC7). The test does
  not modify, add, remove, or rename any C2S/S2C message type, any
  channel binding, or any send/drain system.
- **No Sprint 13 activation**. No `production/sprint-status.yaml` /
  `production/stage.txt` / `production/sprints/sprint-12.md` /
  `production/sprints/sprint-13.md` modification under this story.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or
  any carried Sprint 10 / Sprint 11 / Sprint 12 condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Disjoint files**: This story's anticipated file set
  (`tests/invariants/`, optional `shared/src/protocol.rs` declarative
  constant, optional `Cargo.toml` workspace edit, new evidence file)
  is disjoint from every Sprint 12 Must Have row's anticipated file
  set:
  - Sprint 12 story 019 (`production/epics/hand-ui/story-019-...`) is
    runtime evidence only and touches no source files.
  - Sprint 12 story 012 (HUD snapshot phase bridge fixture) touches
    `tests/integration/board_rendering/` and optionally `client/src/`
    HUD code -- disjoint from `tests/invariants/` and from
    `shared/src/protocol.rs`.
  - Sprint 12 story 013 (lobby ConfirmClass intent chain) touches
    `client/src/ui/lobby.rs` and `tests/integration/playable_client/`
    -- disjoint from this story.
  - Sprint 12 story 014 (cooccupancy panic guard decision) touches
    `client/src/presentation/board_rendering.rs` and
    `tests/unit/board_rendering/status_icons_test.rs` -- disjoint.
  - Sprint 12 story 015 (fixture D residuals umbrella) touches
    `tests/integration/board_rendering/ghost_preview_bridge_test.rs`
    and `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`
    -- disjoint.
- **No Sprint 12 invasion**: This story's implementation prompt
  (Sprint 13 candidate) MUST NOT land before Sprint 12 close-out
  unless the producer explicitly authorises a pull-forward via a
  separate prompt. The default Sprint 13 dispatch ordering is to land
  this story together with `S13-PROTO-ORPHAN-DRAIN-001` in the same
  Sprint 13 wave.
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
5. `/dev-story story-007-protocol-completeness-invariant.md` is
   dispatched (separate prompt).

Expected implementation flow:

1. **Wave 1 -- Source scanning approach decision**: The implementation
   prompt chooses between (a) parsing `shared/src/protocol.rs` source
   text directly, (b) adding a canonical name-list constant to
   `shared/src/protocol.rs` and reading it, or (c) using a build-time
   `proc-macro` to enumerate types. The trade-off is scope-cap vs
   maintenance cost; (b) is the recommended default because it is
   declarative and keeps the test simple.
2. **Wave 2 -- Test scaffolding**: Author
   `tests/invariants/protocol_completeness_test.rs` with a single
   `#[test] fn protocol_completeness_assert_send_and_drain_sites()` or
   similar canonical name. Set up the source-text grep helpers
   (over `client/src/`, `server/src/`).
3. **Wave 3 -- Run test, record orphans**: Run the test against the
   pre-`S13-PROTO-ORPHAN-DRAIN-001` source tree. The test should FAIL
   with at least the 9 named orphans. Record output verbatim in the
   evidence file.
4. **Wave 4 -- Disposition vs `S13-PROTO-ORPHAN-DRAIN-001`**: If
   `S13-PROTO-ORPHAN-DRAIN-001` is landing in the same Sprint 13 wave,
   the test flips to PASS in that wave; this story's evidence file
   records the FAIL output as a snapshot at the pre-drain commit. If
   not, this story's test is marked `#[ignore = "S13-PROTO-ORPHAN-DRAIN-001
   pending"]` for the Sprint 13 wave with an explicit follow-on
   reference; the `#[ignore]` is removed in the drain-story commit.
5. **Wave 5 -- Evidence**: Populate
   `production/qa/evidence/sprint-13-proto-invariant-evidence.md`
   per "Evidence Path" above.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Source-text parsing approach is brittle to formatting changes in `shared/src/protocol.rs` | Medium | Low-Medium | Default to approach (b) -- canonical name-list constant -- which is declarative and survives formatting changes. |
| Test discovers orphans beyond the 9 named in PROMPT 803 §4 Lane A | Medium | Low | Expected -- recorded in evidence; either rolled into `S13-PROTO-ORPHAN-DRAIN-001` scope or escalated as a separate row. |
| Test surfaces false positives (e.g., a sender lives in a build-feature-gated module not visible to the grep) | Low-Medium | Medium | Document each allowlist exception with an inline rationale comment + cross-reference to a follow-on; default disposition is "fail loudly". |
| `Cargo.toml` workspace edit causes CI build regression | Low | Medium | Scope-cap to a single `[[test]]` block with a rationale comment; verify via `cargo check --workspace --all-targets` before merge. |
| Implementation lands without `S13-PROTO-ORPHAN-DRAIN-001` and the test fails in CI permanently | Medium | Medium | Either land both stories in the same Sprint 13 wave, or `#[ignore]`-gate the test with a one-commit follow-on requirement. |
| Sprint 12 active scope is disturbed | Low | High | This story explicitly excludes all Sprint 12 surfaces (see "Dependency Notes Against Sprint 12 Active Scope"). |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the implementation
prompt, not for the worker:

- `production/sprint-status.yaml` `sprint:` field reads `12` (Sprint 12
  active) until Sprint 12 close-out lands. This story is NOT in the
  active row set.
- `production/stage.txt` reads `Polish` and is unchanged.
- Sprint 12 Must Have rows are not delayed by this story's
  authoring -- this is paperwork-only and parallel-safe.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.
- The PROMPT 761 Polish->Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is
  preserved.

---

## Authoring / Implementation / Closure Trail

- **PROMPT 804 (2026-05-14)** -- Story file authored as a Sprint 13
  candidate for the Protocol Completeness Invariant Test. Sprint 12
  is `active` (PROMPT 798) and is not modified by this authoring run.
  No code changes, no smoke / gate / QA / `/dev-story` / `/story-done` /
  `/story-readiness` / `/qa-plan` run. Source-of-truth at authoring:
  `origin/main@b5eef0d`. Worker branch:
  `work/s13-runtime-hardening-story-authoring`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.
- **PROMPT 808 (2026-05-14)** -- Integration of the PROMPT 804
  authoring batch onto `origin/main` as `55b25be` (8-story
  paperwork integration covering 007 / 008 / 016-021).
- **PROMPT 823 (2026-05-14)** -- `/story-readiness` batch verdict
  **READY** for this story file (12 newly authored Sprint 13 story
  files reviewed).
- **PROMPT 826 (2026-05-14)** -- Sprint 13 activated (flipped
  top-level `sprint: 12 -> 13` and `status:
  closed-with-conditions -> active`). This story row promoted into
  Sprint 13 active Must Have at status `ready`.
- **PROMPT 845 (2026-05-14)** -- `/dev-story` worker implementation
  on `work/s13-protocol-completeness-invariant` from base
  `origin/main@fe74fb0` (PROMPT 844 closure). Worker commit
  `96c16003024d836cf4c24b0eeb35cdeb78e2cb20`:
  - `tests/invariants/protocol_completeness_test.rs` (NEW; 421
    lines): manifest scanner (`discover_registered_messages` over
    `register_c2s::<...>` / `register_s2c::<...>` lines), alias-aware
    send/drain detection (`MessageSender<T>` / `MessageReceiver<T>`
    + `send::<T,` broadcast-call), `#[cfg(test)]` block stripper.
    Two `#[test]` functions: enabled parser-smoke (`protocol_manifest_parser_discovers_registered_messages`)
    asserts `>= 16 C2S + >= 34 S2C`; ignored invariant
    (`protocol_completeness_assert_send_and_drain_sites`) gated by
    `#[ignore = "S13-PROTO-ORPHAN-DRAIN-001 pending --
    pre-drain orphan list captured in production/qa/evidence/
    sprint-13-proto-invariant-evidence.md; remove this attribute
    in the drain-story commit"]`.
  - `shared/Cargo.toml`: one `[[test]] name =
    "protocol_completeness_invariant" path = "../tests/invariants/
    protocol_completeness_test.rs"` block with an inline rationale
    comment cross-referencing `S13-PROTO-ORPHAN-DRAIN-001` and the
    evidence file. Scope-cap per AC8.
  - `production/qa/evidence/sprint-13-proto-invariant-evidence.md`
    (NEW; 330 lines): AC1-AC11 closure evidence with verbatim
    no-claim restatement, pre-drain test output (13 violations
    across 10 unique types), regression commands with Cargo
    resource policy, cross-link to PROMPT 803 §3 DC-1 / §3 DC-15 /
    §4 Lane A.
  - Targeted regression: `cargo fmt --all -- --check` (EXIT=0);
    `cargo test -p shared --test protocol_completeness_invariant
    -- --nocapture` (EXIT=0, 1 passed / 1 ignored); `cargo test
    -p shared --test protocol_completeness_invariant
    protocol_completeness_assert_send_and_drain_sites --
    --ignored --nocapture` (EXIT=101, 13 violations expected
    pre-drain); `git diff --check origin/main...HEAD` (EXIT=0);
    `git diff --check` (EXIT=0); `git diff --cached --check`
    (EXIT=0 pre-commit). Cargo resource policy applied
    (`CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc`,
    `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
    `CARGO_INCREMENTAL=0`, `RUSTFLAGS=-C debuginfo=0 -C
    link-arg=/DEBUG:NONE`). `cargo check --workspace
    --all-targets` intentionally NOT run per the worker dispatch
    directive and the QA-plan-sprint-13 no-full-workspace-tests-
    by-default policy.
  - AC verdicts at worker tip: AC1-AC11 all PASS per
    `reports/PROMPT-845-S13-PROTOCOL-COMPLETENESS-INVARIANT.md`
    "AC verification" table.
- **PROMPT 847 (2026-05-14)** -- Unrelated Sprint 13 observability
  story (`S13-OBS-TRACING-TARGETS-001`) landed on `origin/main` as
  `9e32fbe` during the PROMPT 849 integration window. No file
  overlap with PROMPT 845 (per integration report row 10:
  PROMPT 847 touched `client/src/{ui/hand/mod.rs,
  presentation/board_rendering.rs, card_animations/input_gating.rs}`
  + `server/src/{feature/*, network/*}` -- disjoint from
  `tests/invariants/` and `shared/Cargo.toml`).
- **PROMPT 849 (2026-05-14)** -- Integration to `origin/main`.
  Integration worktree
  `D:\_DEV\claude-code-game-studios-worktrees\integration-s13-protocol-completeness-invariant-849`;
  branch `integrate/s13-protocol-completeness-invariant-849` built
  from `origin/main@fe74fb0`; merged worker tip `96c1600` via
  fast-forward; merged advanced `origin/main` (now at PROMPT 847
  `9e32fbe`) into integration branch producing merge commit
  `25573e6d550c916eba22130791142ab9986d2dde`; fast-forward pushed
  to `origin/main` (non-force). Integration verification re-ran
  the worker's targeted commands at integration tip with identical
  output (13-violation orphan caveat preserved exactly); AC7 /
  AC8 / AC10 zero-touch verified. `cargo check --workspace
  --all-targets` not run (worker report explicitly stated not
  required; targeted `cargo test` invocation proves the new test
  compiles and discovers cleanly).
- **PROMPT 850 (2026-05-14)** -- Sibling `/story-done` for the
  parallel Must Have row `S13-OBS-TRACING-TARGETS-001` (playable-
  client story 018); commit `c1b7753` on `origin/main` landed
  between PROMPT 849 and PROMPT 851. Preserves the PROMPT 845 /
  849 work for story 007 unchanged on `origin/main` (no file
  overlap with story 007's integration scope).
- **PROMPT 851 (2026-05-14)** -- `/story-done` paperwork closure
  at root checkout against `origin/main@c1b7753` (serialized
  shared-status writer per 2026-05-13 override; matches
  PROMPT 850 / PROMPT 844 / PROMPT 843 / PROMPT 840 / PROMPT 835 /
  PROMPT 833 paperwork pattern). AC1-AC11 all verified against
  integrated evidence on `origin/main`. Files modified:
  - This story file (Status flipped Draft -> Done with PROMPT 851
    closure context; AC1-AC11 checkboxes `[ ]` -> `[x]` with
    per-AC closure-evidence annotations; Authoring / Implementation /
    Closure Trail rewritten with PROMPT 804 / 808 / 823 / 826 / 845 /
    847 / 849 / 851 entries + Conditions carried forward unchanged +
    Explicitly NOT claimed sub-sections).
  - `production/sprint-status.yaml` (Sprint 13 Must Have row
    `S13-PROTO-INVARIANT-001` flipped `status: ready -> done` with
    `completed: 2026-05-14`, `worker_prompt: 845`, `worker_commit:
    96c16003024d836cf4c24b0eeb35cdeb78e2cb20`, `integration_prompt:
    849`, `integration_commit:
    25573e6d550c916eba22130791142ab9986d2dde`, `story_done_prompt:
    851`, `test_evidence: tests/invariants/protocol_completeness_test.rs`,
    `acceptance_evidence: production/qa/evidence/sprint-13-proto-invariant-evidence.md`;
    top-level `updated:` annotation refreshed for PROMPT 851;
    `sprint_13_story_done:` block extended with PROMPT 851 entry as
    a sibling to the prior PROMPT 833 + 840 + 843 + 844 + 850
    entries).
  - `production/session-state/active.md` (PROMPT 851 banner
    prepended above PROMPT 850 banner).
  - `production/session-state/codex-orchestrator-state.md` (PROMPT
    851 section prepended above PROMPT 850 section).
- **Cargo policy**: N/A for PROMPT 851 itself (paperwork-only
  closure; no cargo command invoked). Worker (PROMPT 845) and
  integration (PROMPT 849) targeted regression runs applied the
  binding Windows/MSVC Cargo resource policy at their respective
  checkpoints.

## Conditions carried forward unchanged

- S8-QA-001-W1 manual/browser two-client GAME_OVER gap remains
  OPEN. Story 017 (two-client runtime harness) AC12 forbid-auto-
  closure: explicitly does NOT close S8-QA-001-W1 by itself.
- QA-COND-0005 Standard-tier accessibility accepted-risk
  (friend-game scope only).
- QA-COND-0006 playtest / fun-hypothesis validation accepted-risk
  / deferred.
- PAW-TD-*-a placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved (folded
  into Sprint 12 story 019 cannot-reproduce closure; third
  same-scope retest NOT authorised per TQ-S12-C2). PROMPT 851 does
  NOT re-attempt the Sprint 12 capture.
- PROMPT 761 Polish->Release gate-check FAIL preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 851 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed (closed cannot-reproduce, NOT bug-fixed).
- TQ-S12-C1..C7 (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved unchanged.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- Prior `/story-done` closures preserved unchanged on
  `origin/main`: PROMPT 833 (`S11-SERVER-POOL-INIT-LOG-GUARD-001`),
  PROMPT 835 (`S11-LOBBY-UX-CONFIRM-STATE-001`), PROMPT 840
  (`S13-UI-AUDIT-ROADMAP-PREP-001`), PROMPT 843
  (`S13-OBS-WALLCLOCK-TIMESTAMPS-001`), PROMPT 844
  (`S11-HU-PHASE-IDEMPOTENCY-001`), PROMPT 850
  (`S13-OBS-TRACING-TARGETS-001`).
- **Pre-`S13-PROTO-ORPHAN-DRAIN-001` orphan caveat preserved.**
  The ignored invariant
  `protocol_completeness_assert_send_and_drain_sites` continues to
  panic with 13 violations across 10 unique message types at
  `origin/main@c1b7753` (all 9 PROMPT 803 §4 Lane A named orphans
  + `C2SClassChoice`). The `#[ignore]` attribute is the documented
  pre-drain disposition per the story Implementation Notes Wave 4;
  removal of the attribute is owned by the `S13-PROTO-ORPHAN-DRAIN-001`
  drain-story commit (a future `/dev-story` prompt) -- PROMPT 851
  does NOT modify the test file or remove the `#[ignore]`
  attribute.

## Explicitly NOT claimed by PROMPT 851

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from Polish to Release
- underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed)
- full UI clean-pass repair
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`
- **Protocol Orphan Drain implementation** (`S13-PROTO-ORPHAN-DRAIN-001`
  remains `ready`; not started, not closed, not modified by
  PROMPT 851)
- removal of the `#[ignore]` attribute on
  `protocol_completeness_assert_send_and_drain_sites` (owned by
  the drain-story commit)
- Sprint 13 close-out (Sprint 13 remains active; 7 of 19 rows
  closed after PROMPT 851 -- 3 of 6 Must Have, 3 of 6 Should
  Have, 1 of 7 Nice to Have)
- full-workspace `cargo test --workspace --tests --no-fail-fast`
  result claim (deferred to orchestrator end-of-sprint
  integration gate per QA-plan-sprint-13 no-full-workspace-tests-
  by-default policy)
- Plugin Registration Invariant test (`S13-PLUGIN-REGISTRATION-INVARIANT-001`
  Sprint 14+ row -- not authored, not landed)
- `#[ignore]` / `#[should_panic]` attribute-drift invariant test
  (`S13-IGNORE-ATTRIBUTE-DRIFT-001` Sprint 14+ row -- not authored,
  not landed)
