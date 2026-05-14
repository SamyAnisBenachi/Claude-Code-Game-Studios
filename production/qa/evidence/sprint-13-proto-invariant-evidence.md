# Sprint 13 — Protocol Completeness Invariant Evidence

> **Story**: `production/epics/lightyear-protocol-verification/story-007-protocol-completeness-invariant.md`
> **Story ID**: `S13-PROTO-INVARIANT-001`
> **Implementation prompt**: PROMPT 845 (`/dev-story` worker)
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-protocol-completeness-invariant`
> **Worker branch**: `work/s13-protocol-completeness-invariant`
> **Source-of-truth at start**: `origin/main@fe74fb0` (PROMPT 844 closure
> `qa(s13): /story-done S11-HU-PHASE-IDEMPOTENCY-001 (PROMPT 844)`).

---

## Status / No-Claim Banner (verbatim from story 007)

PROMPT 845 (this implementation run) does NOT:

- Activate Sprint 13 or change its disposition. (Sprint 13 is already
  active per `production/sprint-status.yaml`; PROMPT 845 does not flip
  `sprint:` / `stage:` / per-row status.)
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other active sprint
  file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan file.
- Run `/story-readiness`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify production code under `client/src/**`, `server/src/**`, or
  `shared/src/**`. AC7 is satisfied: only `tests/invariants/` (NEW) and
  `shared/Cargo.toml` (scope-capped `[[test]]` registration) are added.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 disposition (`closed-with-conditions` per PROMPT 763), Sprint
11 disposition (`closed-with-conditions` per PROMPT 792), and Sprint 12
disposition (`closed-with-conditions` per PROMPT 817) remain unchanged.
PROMPT 761 Polish→Release gate-check FAIL evidence at
`production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The invariant test is read-only over `shared/src/protocol.rs`
and over `client/src/` / `server/src/`; it does not call any Bevy or
Lightyear runtime API and does not mutate any protocol shape, channel
binding, or message-handling behaviour. ADR-002 binding preserved.

---

## What landed (PROMPT 845 commit scope)

1. **NEW** `tests/invariants/protocol_completeness_test.rs` (`protocol_completeness_invariant` workspace test target).
   - One ignored test `protocol_completeness_assert_send_and_drain_sites`
     gating the invariant proper; `#[ignore]` is removed by
     `S13-PROTO-ORPHAN-DRAIN-001` per the story Implementation Notes
     Wave 4.
   - One enabled test `protocol_manifest_parser_discovers_registered_messages`
     guarding against silent parser regressions (asserts ≥16 C2S + ≥34
     S2C registered messages and that every registered name has a
     `pub struct` declaration).
2. **MODIFIED** `shared/Cargo.toml` — a single `[[test]]` block added,
   scope-capped per AC8, with an inline rationale comment cross-referencing
   `S13-PROTO-ORPHAN-DRAIN-001`. No other production-side change.
3. **NEW** `production/qa/evidence/sprint-13-proto-invariant-evidence.md`
   (this file).

`git diff --name-only origin/main...HEAD` returns exactly these three
paths; the worker commit diff under `client/src/**`, `server/src/**`,
and `shared/src/**` is empty. AC7 satisfied.

---

## Test design summary

- **Source-text inspection**, not runtime Lightyear introspection. The
  test reads `shared/src/protocol.rs` via `include_str!` and walks
  `client/src/` and `server/src/` with `std::fs` to find consumer files.
- **Manifest discovery** scans `register_c2s::<T>(...)` /
  `register_s2c::<T>(...)` lines verbatim. The 50 currently-registered
  message types (16 C2S + 34 S2C) are enumerated automatically; the
  parser smoke test asserts these floors stay stable.
- **Send-site detection** looks for either the Lightyear SystemParam
  `MessageSender<T>` OR the canonical broadcast call form
  `send::<T, _>` on `ServerMultiMessageSender` (this codebase mostly
  uses the latter on the server). The detector follows `as Alias`
  imports per-file so that
  `server/src/network/economy_dispatch.rs` (which imports
  `S2CGoldBroadcast as ProtocolGoldBroadcast`) and
  `server/src/feature/auction/system.rs` (`S2CAuctionCard as
  ProtocolS2CAuctionCard`) are correctly counted as send-sites for
  their canonical protocol types.
- **Drain-site detection** looks for `MessageReceiver<T>` SystemParam
  references, with the same alias-following.
- **`#[cfg(test)]` mod blocks** are erased before scanning so that
  references inside per-file unit-test modules do not satisfy the
  production-side reference requirement. Files under `tests/` are
  not scanned (only `client/src/` and `server/src/`).
- **Channel choice is not exempt**: `C2SHeartbeat` /
  `S2CHeartbeat` (`UnreliableChannel`) are required to have
  send- + drain-sites identically to `ReliableChannel` messages
  per AC4 / story Control Manifest.

The implementation uses Lightyear 0.26 / Bevy 0.18 API shapes
exclusively (`liv-bevy-018` + `liv-bevy-lightyear` skills). The test
does not construct a Bevy `App`; it has zero `bevy` or `lightyear`
crate imports.

---

## Pre-`S13-PROTO-ORPHAN-DRAIN-001` test output (verbatim, PROMPT 845)

Command (Cargo policy applied):

```
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p shared --test protocol_completeness_invariant \
    protocol_completeness_assert_send_and_drain_sites \
    -- --ignored --nocapture
```

Verbatim output (stripped of build-system noise):

```
running 1 test

thread 'protocol_completeness_assert_send_and_drain_sites' panicked at
shared\..\tests\invariants\protocol_completeness_test.rs:384:9:
Protocol completeness invariant violated: 13 violation(s) across 50
registered message type(s).

Each entry: <MessageType>  (declaration:line)
    missing-side: actionable remediation hint.
Cross-link:
reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md
§3 DC-1, §3 DC-15, §4 Lane A.

C2SClassChoice  (shared/src/protocol.rs:421)
    missing client-side send-site: add a `MessageSender<C2SClassChoice>`
    SystemParam (or call `sender.send::<C2SClassChoice, _>(...)`) under
    client/src/, or delete the type from the protocol with a rationale.

C2SRequestSnapshot  (shared/src/protocol.rs:472)
    missing server-side drain: add a
    `MessageReceiver<C2SRequestSnapshot>` SystemParam under server/src/,
    or delete the type from the protocol with a rationale.

S2CHeartbeat  (shared/src/protocol.rs:817)
    missing server-side send-site: add a `MessageSender<S2CHeartbeat>`
    SystemParam or call `sender.send::<S2CHeartbeat, _>(...)` on a
    `ServerMultiMessageSender` under server/src/, or delete the type
    from the protocol with a rationale.

S2CHeartbeat  (shared/src/protocol.rs:817)
    missing client-side drain: add a `MessageReceiver<S2CHeartbeat>`
    SystemParam under client/src/, or delete the type from the protocol
    with a rationale.

S2COpponentDisconnected  (shared/src/protocol.rs:593)
    missing server-side send-site: add a
    `MessageSender<S2COpponentDisconnected>` SystemParam or call
    `sender.send::<S2COpponentDisconnected, _>(...)` on a
    `ServerMultiMessageSender` under server/src/, or delete the type
    from the protocol with a rationale.

S2COpponentDisconnected  (shared/src/protocol.rs:593)
    missing client-side drain: add a
    `MessageReceiver<S2COpponentDisconnected>` SystemParam under
    client/src/, or delete the type from the protocol with a rationale.

S2COpponentReconnected  (shared/src/protocol.rs:599)
    missing client-side drain: add a
    `MessageReceiver<S2COpponentReconnected>` SystemParam under
    client/src/, or delete the type from the protocol with a rationale.

S2CPoolUpdate  (shared/src/protocol.rs:548)
    missing server-side send-site: add a `MessageSender<S2CPoolUpdate>`
    SystemParam or call `sender.send::<S2CPoolUpdate, _>(...)` on a
    `ServerMultiMessageSender` under server/src/, or delete the type
    from the protocol with a rationale.

S2CPoolUpdate  (shared/src/protocol.rs:548)
    missing client-side drain: add a `MessageReceiver<S2CPoolUpdate>`
    SystemParam under client/src/, or delete the type from the protocol
    with a rationale.

S2CPrismRespawned  (shared/src/protocol.rs:533)
    missing client-side drain: add a
    `MessageReceiver<S2CPrismRespawned>` SystemParam under client/src/,
    or delete the type from the protocol with a rationale.

S2CPrismRewardDropped  (shared/src/protocol.rs:527)
    missing client-side drain: add a
    `MessageReceiver<S2CPrismRewardDropped>` SystemParam under
    client/src/, or delete the type from the protocol with a rationale.

S2CSangMepriseReveal  (shared/src/protocol.rs:664)
    missing client-side drain: add a
    `MessageReceiver<S2CSangMepriseReveal>` SystemParam under
    client/src/, or delete the type from the protocol with a rationale.

S2CSessionCancelled  (shared/src/protocol.rs:654)
    missing client-side drain: add a
    `MessageReceiver<S2CSessionCancelled>` SystemParam under
    client/src/, or delete the type from the protocol with a rationale.


test protocol_completeness_assert_send_and_drain_sites ... FAILED

failures:
    protocol_completeness_assert_send_and_drain_sites

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured;
             1 filtered out; finished in 0.06s
```

---

## Cross-check vs PROMPT 803 §4 Lane A "9 named orphans"

| # | Orphan (PROMPT 803) | Surfaced by PROMPT 845 test | Verdict |
|---|---|---|---|
| 1 | `S2CHeartbeat` (defined-but-never-drained client + never-sent server) | ✔ both sides | PASS |
| 2 | `S2COpponentDisconnected` (defined-but-never-drained client; server stub absent) | ✔ both sides | PASS |
| 3 | `S2COpponentReconnected` (defined-but-never-drained client) | ✔ client drain | PASS |
| 4 | `S2CPoolUpdate` (defined-but-never-drained client; never-sent server) | ✔ both sides | PASS |
| 5 | `S2CPrismRespawned` (defined-but-never-drained client) | ✔ client drain | PASS |
| 6 | `S2CPrismRewardDropped` (defined-but-never-drained client) | ✔ client drain | PASS |
| 7 | `S2CSangMepriseReveal` (defined-but-never-drained client) | ✔ client drain | PASS |
| 8 | `S2CSessionCancelled` (defined-but-never-drained client) | ✔ client drain | PASS |
| 9 | `C2SRequestSnapshot` (defined-but-no-server-handler) | ✔ server drain | PASS |

All 9 named orphans surfaced. AC6 satisfied (the 9 are a floor, not a
ceiling, per the story text).

**One additional orphan discovered by PROMPT 845** (story explicitly
allows this — "the 9 named orphans are a floor, not a ceiling"):

- `C2SClassChoice` (`shared/src/protocol.rs:421`) — server has the drain
  (`MessageReceiver<C2SClassChoice>` at `server/src/lobby/handler.rs:15`,
  `apply_class_choice` at `:40`), but no `client/src/` file references
  `MessageSender<C2SClassChoice>` or `send::<C2SClassChoice, _>`. The
  client lobby uses `C2SSelectClass` (`client/src/ui/lobby.rs:514`) and
  `C2SConfirmClass` (`:515`) instead. This is the same DC-1 defect
  class as the 9 named orphans and is rolled into the
  `S13-PROTO-ORPHAN-DRAIN-001` scope (either add a client send-site, or
  delete the type with a rationale — disposition decided by that story).

---

## Post-`S13-PROTO-ORPHAN-DRAIN-001` re-run (slot reserved)

> *Not populated by PROMPT 845.*
>
> When `S13-PROTO-ORPHAN-DRAIN-001` lands, that prompt's worker:
>
> 1. Adds the missing senders / drains (or deletes orphan types with an
>    inline rationale per the story Control Manifest).
> 2. Removes the `#[ignore = "S13-PROTO-ORPHAN-DRAIN-001 pending"]`
>    attribute on
>    `protocol_completeness_assert_send_and_drain_sites`.
> 3. Re-runs the command above (without `--ignored`) and appends the
>    PASS output verbatim to this section. If any allowlist exception
>    is required, the rationale comment lives next to the allowlist in
>    the test file and is referenced here.

---

## Regression commands actually run (PROMPT 845)

All commands executed inside the worktree
`D:\_DEV\claude-code-game-studios-worktrees\s13-protocol-completeness-invariant`
with the Cargo policy env vars set as documented above.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 — clean |
| `cargo test -p shared --test protocol_completeness_invariant` | exit 0 — 1 passed (parser smoke), 1 ignored (drain-pending) |
| `cargo test -p shared --test protocol_completeness_invariant protocol_completeness_assert_send_and_drain_sites -- --ignored --nocapture` | exit 101 — orphan list captured above (EXPECTED pre-`S13-PROTO-ORPHAN-DRAIN-001`) |
| `git diff --check` | exit 0 — no whitespace defects |
| `git diff --check origin/main...HEAD` | exit 0 — no whitespace defects vs origin/main |

`cargo check --workspace --all-targets` was **not** run per the PROMPT
845 dispatch directive ("Required checks unless story says otherwise"
— story 007 prescribes targeted invariant test, fmt, and the diff
check). Workspace compile is not gated by this story and is deferred
to Sprint 13 end-of-sprint integration smoke per the QA-plan-sprint-13
no-full-workspace-tests-by-default policy.

---

## ADR / GDD / protocol surfaces — no change

- `shared/src/protocol.rs` — not modified (verified by `git diff` empty
  on this path).
- `client/src/**`, `server/src/**` — not modified (verified by
  `git diff` empty on these paths).
- ADR-002 / ADR-008 / ADR-009 / ADR-012 — no binding affected; the test
  is read-only and runtime-free.
- `design/gdd/network-protocol.md` Table A — not modified; the test's
  manifest enumeration is sourced from the live `register_c2s` /
  `register_s2c` calls, not from Table A. Future Table A updates do
  not require coordinated changes to this test.

"no optimistic client-side authority is introduced or implied" —
verbatim restatement satisfying AC9 evidence requirement.

---

## Cross-links

- `production/epics/lightyear-protocol-verification/story-007-protocol-completeness-invariant.md`
  — owning story (AC1–AC11 enumerated).
- `production/epics/lightyear-protocol-verification/EPIC.md` — epic
  parent and Story 003 (`MessageSender` / `MessageReceiver` placement
  precedent).
- `reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`
  §3 DC-1, §3 DC-15, §4 Lane A — defect-class origin and 9 named
  orphans floor list.
- `tests/evidence/lightyear-026-verification.md` — Lightyear 0.26 API
  surface verification (`MessageSender<T>`, `MessageReceiver<T>`,
  `ServerMultiMessageSender::send::<T, C>`).
- *Next-story slot*: `S13-PROTO-ORPHAN-DRAIN-001` — owns the actual
  orphan repair (drain implementation, sender implementation, or
  documented deletion with rationale).
