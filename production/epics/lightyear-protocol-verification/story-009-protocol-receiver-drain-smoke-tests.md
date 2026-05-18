# Story 009: S18-PROTOCOL-RECEIVER-DRAIN-SMOKE-TESTS-001 -- Cover Real MessageReceiver Drain Path (F-08 Partial)

> **Epic**: Lightyear Protocol Verification
> **Story ID**: `S18-PROTOCOL-RECEIVER-DRAIN-SMOKE-TESTS-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Tests-only -- protocol drain smoke harness (`tests/integration/lightyear-protocol-verification/`)
> **Type**: Integration (real-wire receiver-drain smoke)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPT**: 1246 -- `test(s18-protocol-receiver-drain-smoke-tests): cover real MessageReceiver drain path`
> **Implementing commit**: `2c76959`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1202 F-08 (anti-pattern: bypassing real `MessageReceiver<T>` drain via direct `Presentation*Message` writes)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `2c76959`**. PROMPT 1296
makes **no** code, test, Cargo, CI, sprint, QA, or session-state
mutations. Sprint 18 is **NOT activated** by this authoring run. All
standard non-claims preserved verbatim.

This story is a **partial** F-08 close: it adds smoke coverage that
exercises real receiver-drain paths for a representative subset of
message types. The broader F-08 work (migrating all snapshot bypass
tests to a shared `real_wire_snapshot.rs` helper) remains a separate
candidate (`S18-PROTOCOL-SNAPSHOT-REAL-WIRE-TESTS-001`) per PROMPT
1287 Lane W10.

---

## Source Finding

**PROMPT 1202 F-08 / PROMPT 1086 anti-pattern history**: integration
tests for HUD / hand-ui / placement etc. were bypassing the real
`MessageReceiver<T>` drain by writing directly into the presentation-
side mirror (e.g. `PresentationGameSnapshotMessage`). A regression in
the real `*_sink_system` drain path therefore would not fail these
tests. F-08 listed this anti-pattern across four hot-spot test files
and recommended a `real_wire_snapshot.rs` helper.

PROMPT 1246 (`test`) authored a focused smoke layer that exercises the
**real** Lightyear `MessageReceiver<T>` drain for a representative
subset of message types so any regression in the drain wiring fails
this smoke before reaching the bypass tests.

---

## Landed Evidence (commit `2c76959`, PROMPT 1246)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `tests/integration/lightyear-protocol-verification/protocol_receiver_drain_test.rs` (NEW) | 614 LOC smoke covering real receiver drain for a representative subset of S2C messages. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Real-wire smoke target exists**:
  `tests/integration/lightyear-protocol-verification/protocol_receiver_drain_test.rs`
  is present on `origin/main` and runs under the project's standard
  test invocation.
- [ ] **AC2 -- Real MessageReceiver drained**: the smoke test sends a
  message via the real Lightyear send path (server-side
  `ServerMultiMessageSender` or equivalent) and asserts that the
  matching client-side `MessageReceiver<T>` drains it through the
  real `*_sink_system` -- not through a direct
  `Presentation*Message` write.
- [ ] **AC3 -- Representative subset covered**: at minimum the smoke
  covers the S2C message types most regressed by past breakages
  (per PROMPT 1086 / F-08 narrative). The exact list is whatever the
  worker put under test in `2c76959`; the AC binds to *coverage
  existence*, not a specific enumeration.
- [ ] **AC4 -- PASS at activation tip**: the smoke test PASSES at the
  Sprint 18 activation tip with no skipped or `#[ignore]` cases.
- [ ] **AC5 -- No production code change**: `client/src/`, `server/src/`,
  and `shared/src/` diffs are empty in `2c76959` (tests-only commit).
- [ ] **AC6 -- F-08 partial-close status**: this story is recorded as
  the partial close of F-08; the broader snapshot-helper migration
  (`S18-PROTOCOL-SNAPSHOT-REAL-WIRE-TESTS-001`, PROMPT 1287 Lane W10)
  remains open and is NOT claimed closed by this story.
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Migration of the four bypass tests listed in PROMPT 1202 F-08 (HUD
  opp-class-recipient-mismatch, HUD opp-figurine, placement
  perspective snapshot, placement board-view team-map bootstrap).
  Owned by `S18-PROTOCOL-SNAPSHOT-REAL-WIRE-TESTS-001`.
- Authoring a generic `tests/helpers/real_wire_snapshot.rs` helper.
  Owned by the W10 follow-on.
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/lightyear-protocol-verification/EPIC.md` (table
  row added). Implementation landed via PROMPT 1246 at `2c76959`
  prior to this authoring; this stub does not re-author or alter that
  work.
