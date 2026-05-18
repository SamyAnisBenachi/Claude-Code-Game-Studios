# Story 009: S18-PROTOCOL-SNAPSHOT-REAL-WIRE-TESTS-001 -- Real-Wire Snapshot Test Helper + 4 Test Migrations

> **Epic**: Lightyear Protocol & Verification Spike
> **Story ID**: `S18-PROTOCOL-SNAPSHOT-REAL-WIRE-TESTS-001`
> **Status**: Draft -- future Sprint 18 candidate; NOT activated. No sprint plan currently activates this row. Sprint 17 remains `active` per `production/sprint-status.yaml`. `production/sprints/sprint-18.md` does not exist at authoring time. `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/stage.txt`, and every `production/session-state/*` file are NOT modified by this authoring run.
> **Layer**: Test Infrastructure / Protocol Verification -- snapshot-driven systems must exercise the real Lightyear receive/sink path
> **Type**: Logic (test infrastructure + 4 test migrations) -- NO production-code change
> **Sprint**: Sprint 18 candidate (story-authoring lane SA-5 per `reports/PROMPT-1287-sprint-18-parallel-lane-readiness-map.md` §3.11 Lane W10). Authoring does NOT activate Sprint 18.
> **Authored**: 2026-05-18 by PROMPT 1295 (`S18-STORY-AUTHORING-WAVE-B`)
> **Authoring worktree**: `D:\tmp\wt-1295`
> **Authoring branch**: `work/s18-story-authoring-wave-b-1295`
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 `plan(s18): draft Sprint 18 plan`)
> **Source audit**: `reports/PROMPT-1202-multiplayer-protocol-state-consistency-bug-audit.md` §2 row F-08 (P2 -- anti-pattern flagged with explicit historical regression evidence: PROMPT 1086 fix → PROMPT 1130 NEW-1130-01 ~6-week gap).

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

**No production-code change is introduced or proposed by this story.** ADR-002 + ADR-008 + ADR-009 + ADR-010 + ADR-011 + ADR-021 binding preserved. The story changes only:

1. `tests/helpers/` (NEW helper module: `real_wire_snapshot.rs`).
2. The four flagged test files (migrate off direct `PresentationGameSnapshotMessage` injection).
3. (Optionally; worker discretion) a CI grep gate that fences future bypass writes.

No `shared/src/protocol.rs` edit. No `server/src/` edit. No `client/src/` edit. No new C2S / S2C message, no new channel binding, no new authoritative state, no new presentation-layer system. **The production wire from `S2CGameSnapshot` → `game_snapshot_sink_system` → `PresentationGameSnapshotMessage` is unchanged** -- this story only verifies that wire under test.

---

## Source Finding (PROMPT 1202)

`reports/PROMPT-1202-multiplayer-protocol-state-consistency-bug-audit.md` §2 row F-08 ("Snapshot-driven systems tested by direct-message-injection bypass real wire"):

> **Pattern**: tests insert a fake `PresentationGameSnapshotMessage` into the bus via `world.write_message(...)` or `world.resource_mut::<Messages<…>>.write(...)`. The system-under-test (e.g. `apply_placement_board_view_from_snapshot_system`, `hud_opp_figurine_repaint_system`) reads the bus and produces the assert-worthy output. **No production code path that emits `PresentationGameSnapshotMessage` is exercised** -- `game_snapshot_sink_system` is not in any of these test apps' schedules.
>
> **Concrete sites** (verified by `grep -rn "PresentationGameSnapshotMessage" tests/`):
>
> - `tests/integration/hand-ui/placement_perspective_snapshot_test.rs:285`
> - `tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs:389` (one of seven subtests)
> - `tests/integration/hud/hud_opp_class_recipient_mismatch_test.rs:57`
> - `tests/integration/hud/hud_opp_figurine_test.rs:301`
>
> **Historical evidence**: PROMPT 1086 fix's only test injected `PresentationGameSnapshotMessage` directly -- the system worked under test but never fired in production because `S2CGameSnapshot` was reconnect-only. PROMPT 1130 NEW-1130-01 audit caught this six weeks later.
>
> **Repair surface**: introduce a shared test helper `tests/helpers/fake_snapshot_arrival.rs` that:
>
> 1. Inserts the `S2CGameSnapshot` into the `MessageReceiver` of a single-client test App via lightyear's test-friendly receiver-poke API, OR
> 2. Builds a two-app fixture (server + 1 client) and writes the snapshot through the real `ServerMultiMessageSender` so the receive path is exercised end-to-end.
>
> **Coverage acceptance**: every consumer test that today writes `PresentationGameSnapshotMessage` directly should call the new helper instead. Migration is mechanical; the new helper guarantees that a regression in `game_snapshot_sink_system` (e.g. removed from the schedule, or filter changed) trips every dependent test.

Cross-references: `reports/PROMPT-1287-sprint-18-parallel-lane-readiness-map.md` §3.11 Lane W10 + §5 row SA-5. Historical context: `reports/PROMPT-1086-*` (the original "silent dead in prod" fix) and `reports/PROMPT-1130-*` NEW-1130-01 (the audit that caught it six weeks later).

The audit names the helper `tests/helpers/fake_snapshot_arrival.rs`. The lane-readiness map (PROMPT 1287 §3.11) renames it `tests/helpers/real_wire_snapshot.rs` for clarity ("`real_wire_snapshot`" describes the helper's intent better than "`fake_snapshot_arrival`"). **This story adopts `tests/helpers/real_wire_snapshot.rs`** as the canonical name. The implementing worker MAY use an alternate filename inside `tests/helpers/` provided the name describes the intent (the helper exercises the REAL Lightyear receive path, NOT the bypass).

---

## Problem

`S2CGameSnapshot` is the server-authoritative full-state-rebuild payload (recovery / reconcile bus). Its production wire on `origin/main@1345c6b` is:

```
server::game::snapshot.rs → ServerMultiMessageSender → S2CGameSnapshot over ReliableChannel
                          ↓ (Lightyear receive)
client::network::game_snapshot_receiver_system → S2CGameSnapshot message bus
                          ↓ (drained by)
client::presentation::game_snapshot_sink_system (presentation/mod.rs:288-325)
                          ↓ (fans out to)
ClientGameSnapshotMessage (board)  AND  PresentationGameSnapshotMessage (UI / HUD / Hand)
                          ↓ (drained by)
apply_placement_board_view_from_snapshot_system (hand-ui)
hud_opp_figurine_repaint_system (hud)
hud_opp_class_label_system (hud)
opp_figurine_label_mana_repaint_system (hud)
... and other PresentationGameSnapshotMessage consumers
```

`game_snapshot_sink_system` is the **only production path** that ever writes `PresentationGameSnapshotMessage`. The fan-out includes downstream filters (e.g. the system may early-return when the snapshot's `current_phase` is `Lobby`, or when the local player is unknown, or when the round number has not changed). Any regression in:

- `game_snapshot_sink_system` itself being **dropped from the schedule** (Plugin removal, `App::add_systems` deletion).
- The system's **gating filter changing** (e.g. early return on a different precondition).
- The `S2CGameSnapshot` → `MessageReceiver<S2CGameSnapshot>` drain path being broken upstream.
- The Lightyear `register_protocol(app)` for `S2CGameSnapshot` being removed or its channel binding changed.

... will **silently dead** the production snapshot fan-out without tripping a single test in the fleet today.

### The bypass pattern

The four flagged test files all write into the bus directly, bypassing `game_snapshot_sink_system`:

| File | Line | Pattern |
|---|---|---|
| `tests/integration/hand-ui/placement_perspective_snapshot_test.rs` | `:285` | `world.write_message(PresentationGameSnapshotMessage(snapshot));` |
| `tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs` | `:389` | `world.write_message(PresentationGameSnapshotMessage(snapshot));` (one of seven subtests) |
| `tests/integration/hud/hud_opp_class_recipient_mismatch_test.rs` | `:57` | `world.resource_mut::<Messages<PresentationGameSnapshotMessage>>().write(PresentationGameSnapshotMessage(snapshot));` |
| `tests/integration/hud/hud_opp_figurine_test.rs` | `:301` | `world.resource_mut::<Messages<PresentationGameSnapshotMessage>>().write(PresentationGameSnapshotMessage(snapshot));` |

`grep -rn "PresentationGameSnapshotMessage" tests/` at `origin/main@1345c6b` confirms these four call sites are the audit's named bypass set. (Additional sites that use `PresentationGameSnapshotMessage` exist -- e.g. `tests/integration/hud/hud_resolution_dim_test.rs`, `tests/integration/hud/hud_scoreboard_dot_image_refresh_test.rs`, `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`, `tests/integration/hud/reconnect_snapshot_rebuild_test.rs`, `tests/integration/hud/snapshot_phase_bridge_test.rs`, `tests/integration/playable_client/draft_shop_hand_bridge_test.rs`, `tests/integration/presentation/result_screen_mvp_test.rs` -- per the `grep` at HEAD. These extra sites are NOT in the audit's named four. They MAY be migrated as a secondary scope per AC11 below, but only the audit's four are BLOCKING for this story.)

The two-client integration tests in `tests/integration/playable_client/` (e.g. the friend-game tests) **do** exercise the real wire end-to-end; they are NOT in this story's migration scope. They cover the layer-cake regression (Lightyear → drain → bus → consumer) only at the full-loop level. The gap is that **unit-level** snapshot consumer tests bypass the bus origin.

### Historical evidence (PROMPT 1086 → PROMPT 1130 ~6-week gap)

The audit names a concrete historical regression: PROMPT 1086 landed a fix in `game_snapshot_sink_system` (the only test wrote `PresentationGameSnapshotMessage` directly), the test passed, the fix shipped, **and the production wire was silently dead** because `S2CGameSnapshot` was reconnect-only at the time and `game_snapshot_sink_system` was not in any production schedule that fired during normal play. PROMPT 1130 NEW-1130-01 audited the area six weeks later and caught the dead wire. The audit calls this "the same anti-pattern that hid PROMPT 1130 NEW-1130-01 for ~6 weeks."

**Class**: a test fleet pattern where unit / integration tests of consumers of an internal message bus inject the bus message directly, bypassing the production producer of that bus message. The test verifies the consumer's behaviour-on-message but does NOT verify that the producer (or its upstream wire) emits the message in the first place. A regression in the producer or its upstream wire silently dead the production path without tripping any consumer test.

**Prevention target**: every consumer of `PresentationGameSnapshotMessage` (and, by extension, `ClientGameSnapshotMessage`) MUST exercise the message via the production producer chain (`S2CGameSnapshot` → Lightyear receive → `game_snapshot_sink_system` → bus). A regression in `game_snapshot_sink_system` MUST trip every dependent test.

---

## Contract (test-infrastructure scope -- NO PRODUCTION-CODE CHANGE)

| Concern | Owner | Repair scope |
|---|---|---|
| `tests/helpers/real_wire_snapshot.rs` (NEW helper module) | Test infrastructure | Introduce a single public helper function (or small helper struct) that injects an `S2CGameSnapshot` through the real Lightyear receive path on the App-under-test |
| `tests/integration/hand-ui/placement_perspective_snapshot_test.rs` | Hand UI tests | Migrate `:285` off `world.write_message(PresentationGameSnapshotMessage(...))` to the helper |
| `tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs` | Hand UI tests | Migrate `:389` (one subtest of seven) off the direct write to the helper |
| `tests/integration/hud/hud_opp_class_recipient_mismatch_test.rs` | HUD tests | Migrate `:57` off `world.resource_mut::<Messages<...>>().write(...)` to the helper |
| `tests/integration/hud/hud_opp_figurine_test.rs` | HUD tests | Migrate `:301` off the direct write to the helper |
| `client::presentation::game_snapshot_sink_system` schedule + filter | Client / presentation | **Unchanged.** This story does NOT modify the system; it only verifies that the system is exercised under test |
| `shared/src/protocol.rs::S2CGameSnapshot` shape + channel binding | Protocol | **Unchanged.** No new field, no channel reassignment |
| `client::network::game_snapshot_receiver_system` Lightyear drain | Client / network | **Unchanged.** The helper integrates with this drain via Lightyear's test-friendly receiver API (or via a two-app fixture; worker discretion -- see Implementation Notes) |
| `server::game::snapshot.rs::ServerMultiMessageSender` write path | Server / game | **Unchanged.** If the worker chooses the two-app fixture approach (option 2 in the audit), the helper uses this sender; if the worker chooses the receiver-poke approach (option 1), the sender is not invoked from the helper |
| `tests/integration/playable_client/` two-client tests | Integration | **Unchanged.** Already exercise the real wire end-to-end; outside this story's migration scope |
| CI grep gate fencing future bypass writes | Test infrastructure / DevOps | **Optional / worker discretion** -- if added, lives in a single CI script under `tools/ci/` or in an inline `tests/invariants/` invariant test |

This story introduces **zero new production code**, **zero new protocol shape**, **zero new authoritative state**, and **zero new C2S / S2C message**. It is a test-infrastructure story whose entire blast radius is `tests/` (plus an optional CI grep gate).

---

## Acceptance Criteria

All criteria are independently checkable.

### Helper module (`tests/helpers/real_wire_snapshot.rs`)

- [ ] **AC1 -- Helper module exists at the canonical path**: GIVEN the implementation worker branch, WHEN inspected, THEN `tests/helpers/real_wire_snapshot.rs` exists (or an alternate filename inside `tests/helpers/` chosen by the worker with a name that describes the real-wire intent; e.g. `snapshot_real_wire.rs`, `fake_snapshot_arrival.rs` per the audit's wording). The module is `pub` to the workspace test harness so it is callable from all four migrated test files.

- [ ] **AC2 -- Helper exposes a single primary entry point**: GIVEN the helper module, WHEN inspected, THEN it exposes a single public function (or method on a small builder struct) whose signature is approximately:

  ```rust
  pub fn inject_snapshot_real_wire(app: &mut App, snapshot: S2CGameSnapshot);
  ```

  OR a builder pattern (worker discretion):

  ```rust
  pub struct RealWireSnapshot { /* ... */ }
  impl RealWireSnapshot {
      pub fn new(snapshot: S2CGameSnapshot) -> Self { /* ... */ }
      pub fn deliver(self, app: &mut App) { /* ... */ }
  }
  ```

  The function MUST drive the snapshot through `S2CGameSnapshot → MessageReceiver<S2CGameSnapshot> → game_snapshot_sink_system → PresentationGameSnapshotMessage`. After the helper returns, a subsequent `app.update()` MUST cause the snapshot's fan-out into `PresentationGameSnapshotMessage` (and `ClientGameSnapshotMessage` for board consumers) and downstream into all `PresentationGameSnapshotMessage` consumers in the App's schedule.

- [ ] **AC3 -- Helper chooses ONE of two strategies (worker decision)**:
  - **Strategy A (receiver-poke)**: insert the `S2CGameSnapshot` directly into the App's `MessageReceiver<S2CGameSnapshot>` using a Lightyear-test-friendly receiver API (e.g. a `MessageReceiver::push` / `MessageReceiver::test_inject` / equivalent helper exposed by Lightyear 0.26 for unit tests). If no such API exists in Lightyear 0.26, the worker MAY introduce a small bypass at the `MessageReceiver` boundary -- BUT NOT at the `PresentationGameSnapshotMessage` boundary. The bypass MUST sit ABOVE `game_snapshot_sink_system` in the production chain so the sink system is exercised.
  - **Strategy B (two-app fixture)**: spin up a minimal server App + minimal client App, write the `S2CGameSnapshot` through `ServerMultiMessageSender` on the server App, drive a few `app.update()` ticks on both Apps so the message round-trips, and assert the client App's downstream consumers fire.
  - The worker chooses one strategy and documents the choice in the helper module's doc comment. Strategy A is recommended for performance (no two-App overhead); Strategy B is recommended for fidelity (exercises the entire production wire including Lightyear serde + transport).

- [ ] **AC4 -- Helper integrates with `liv-bevy-018` + `liv-bevy-lightyear` skill conventions**: GIVEN the helper module's `.rs` file, WHEN reviewed, THEN it uses Bevy 0.18 + Lightyear 0.26 API patterns correctly (no `EventWriter` / `EventReader` / `Events<T>` / `add_event`; uses `MessageWriter` / `MessageReader` / `add_message` per ADR-010 + the migration guide). The `liv-bevy-018` skill applies to this module and the `liv-bevy-lightyear` skill applies because the module touches the Lightyear receive path.

- [ ] **AC5 -- Helper carries an inline rationale doc comment**: GIVEN the helper module, WHEN the file's leading doc-comment block is read, THEN it states (in worker's words):
  - The audit reference (`PROMPT 1202 §2 row F-08`).
  - The historical reference (`PROMPT 1086 silent-dead-in-prod fix + PROMPT 1130 NEW-1130-01 audit ~6-week gap`).
  - The contract this helper enforces ("a regression in `game_snapshot_sink_system` MUST trip every dependent test").
  - The chosen strategy (A or B per AC3) and a one-paragraph rationale.

### Test migrations (BLOCKING)

- [ ] **AC6 -- `tests/integration/hand-ui/placement_perspective_snapshot_test.rs:285` migrated**: GIVEN the file, WHEN diffed against `origin/main@1345c6b`, THEN the line `world.write_message(PresentationGameSnapshotMessage(snapshot));` is REPLACED with a call to the AC2 helper (e.g. `inject_snapshot_real_wire(&mut app, snapshot);` or the equivalent builder invocation). The test's assertions are unchanged (the test continues to assert the same downstream consumer behaviour). The test PASSES after the migration.

- [ ] **AC7 -- `tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs:389` migrated**: GIVEN the file, WHEN diffed, THEN the one subtest (of seven) that injects `PresentationGameSnapshotMessage` directly at `:389` is REPLACED with a call to the helper. The other six subtests in the file are NOT modified (they do not bypass the wire; they cover other paths). The migrated subtest's assertions are unchanged. The full test file PASSES after the migration.

- [ ] **AC8 -- `tests/integration/hud/hud_opp_class_recipient_mismatch_test.rs:57` migrated**: GIVEN the file, WHEN diffed, THEN the `Messages<PresentationGameSnapshotMessage>::write(...)` call at `:57` (and any adjacent `resource_mut` boilerplate at `:56`) is REPLACED with a call to the helper. The test's assertions are unchanged. The test PASSES after the migration.

- [ ] **AC9 -- `tests/integration/hud/hud_opp_figurine_test.rs:301` migrated**: GIVEN the file, WHEN diffed, THEN the `Messages<PresentationGameSnapshotMessage>::write(...)` call at `:301` (and any adjacent `resource_mut` boilerplate at `:300`) is REPLACED with a call to the helper. The test's assertions are unchanged. The test PASSES after the migration.

- [ ] **AC10 -- Regression-on-removal is observable**: GIVEN the four migrated tests AND a hypothetical temporary regression where `game_snapshot_sink_system` is removed from the client App's schedule (or its gating filter is inverted), WHEN `cargo test` is run on the migrated test files, THEN at least three of the four tests FAIL (the assertion-worthy downstream consumer behaviour no longer fires because the bus message is never produced). This is the **proof-of-prevention** condition. The `/dev-story` worker MUST demonstrate this in the evidence note (AC15) by temporarily commenting out the sink system in a sibling worktree, running the migrated tests, and observing failures. The temporary regression is then reverted.

- [ ] **AC11 -- Direct-bypass helper is removed or fenced for the four named sites**: GIVEN the four migrated tests, WHEN diffed, THEN no direct `world.write_message(PresentationGameSnapshotMessage(...))` or `world.resource_mut::<Messages<PresentationGameSnapshotMessage>>().write(...)` call remains in any of the four. **OUT OF SCOPE for migration**: the additional sites enumerated in the Problem section (`tests/integration/hud/hud_resolution_dim_test.rs`, `tests/integration/hud/hud_scoreboard_dot_image_refresh_test.rs`, `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`, `tests/integration/hud/reconnect_snapshot_rebuild_test.rs`, `tests/integration/hud/snapshot_phase_bridge_test.rs`, `tests/integration/playable_client/draft_shop_hand_bridge_test.rs`, `tests/integration/presentation/result_screen_mvp_test.rs`). The worker MAY migrate these as a secondary pass (call it a "Phase 2" inside the same worker branch or carry to a sibling follow-on story) but is NOT BLOCKED by them. The BLOCKING migration set is the audit's four named sites only.

### Optional fence (advisory)

- [ ] **AC12 -- Optional CI grep gate**: GIVEN the implementation worker branch, WHEN reviewed, THEN the worker MAY add a single CI grep gate (in `tools/ci/` or as an invariant test under `tests/invariants/`) that fences future regressions of the bypass pattern. The gate's recommended form:

  ```
  grep -rE "(write_message|\.write)\\(PresentationGameSnapshotMessage" tests/ \
    | grep -v "tests/helpers/" \
    | grep -v "// allowed-bypass:" \
    > /dev/null && exit 1 || exit 0
  ```

  (Worker may use an alternate shape; the spirit is "fail CI if any future test injects `PresentationGameSnapshotMessage` directly outside the helper, unless the call site explicitly opts-in with an `// allowed-bypass:` comment naming a reason"). This is **ADVISORY**, not BLOCKING. Producer / worker may defer the fence to a sibling follow-on story if scope is tight.

### Scope guards

- [ ] **AC13 -- NO production-code change**: GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN **zero lines** are changed under `client/src/`, `server/src/`, `shared/src/`. No `Cargo.toml` change beyond an optional `[dev-dependencies]` addition for any new test-only crate (worker discretion; should be unnecessary if the helper uses only Bevy 0.18 + Lightyear 0.26 + the existing test harness). No `shared/src/protocol.rs` edit. No `register_protocol(app)` change. No `S2CGameSnapshot` shape change. No `PresentationGameSnapshotMessage` shape change.

- [ ] **AC14 -- ADR invariants preserved**: GIVEN the changes, WHEN inspected, THEN:
  - **ADR-008 (Channel config)**: `S2CGameSnapshot` continues to route over `ReliableChannel`. No reassignment.
  - **ADR-009 (RSM Phase State)**: no client-side optimistic phase derivation introduced.
  - **ADR-010 (RSM Phase Event Bus)**: no new `Message` type, no `EventWriter` / `EventReader` regression. The helper uses `MessageWriter::write` / `MessageReader::read` patterns or Lightyear test APIs only.
  - **ADR-011 (Reconnect snapshot)**: the helper is compatible with the reconnect-bus path; if the worker chooses Strategy A (receiver-poke), the helper does NOT distinguish reconnect from normal-flow snapshots (consistent with the new normal-flow `S2CGameSnapshot` wire that landed in PROMPT 1130).
  - **ADR-021 (Presentation Layer Architecture)**: `game_snapshot_sink_system` and its `PresentationSet` slot are unchanged. No new schedule wiring.

- [ ] **AC15 -- Windows / MSVC Cargo resource policy compliance**: GIVEN the implementation runs on Windows / MSVC per project convention, WHEN `cargo test --test placement_perspective_snapshot_test --test placement_board_view_team_map_bootstrap_test --test hud_opp_class_recipient_mismatch_test --test hud_opp_figurine_test` (or the project's conventional pattern for running a subset of integration tests) is invoked, THEN the tests respect the Cargo resource policy (run sequentially under `-- --test-threads=1` if required by the local harness; honour the project's `cargo` invocation convention -- see `production/session-state/codex-orchestrator-state.md` for canonical invocation). The migrated tests do not introduce a long-running fixture; the helper should complete each invocation in well under a second under Strategy A, and within a few seconds under Strategy B.

- [ ] **AC16 -- Migration is mechanical**: GIVEN the diff of each of the four migrated test files, WHEN inspected, THEN the per-file diff is small (typically 5-30 lines per file, mostly an `use` import for the helper + the one-line site replacement + possibly an adjacent setup line if Strategy A requires the test to mount the receive plugin). Test assertions are byte-identical to pre-migration. No new test cases are added by the migration (the proof-of-prevention assertion in AC10 is demonstrated in the evidence note, not added as a new permanent test case unless the worker chooses to add one as a permanent invariant -- see AC17).

- [ ] **AC17 -- Optional permanent regression test (advisory)**: GIVEN the implementation worker branch, WHEN reviewed, THEN the worker MAY add a single permanent test under `tests/integration/presentation/` (or equivalent) that asserts the helper's contract directly: "given an App with the production client plugin, when an `S2CGameSnapshot` is injected via the helper, then `PresentationGameSnapshotMessage` appears in the bus after `app.update()`." This test acts as a sentinel: if `game_snapshot_sink_system` is removed from the schedule, this single test fails. This is **ADVISORY**, not BLOCKING. The four migrated tests already cover the regression-on-removal observability per AC10; this sentinel is a "small permanent canary" the worker may add if scope permits.

### Authoring-only scope (PROMPT 1295)

- [ ] **AC18 -- PROMPT 1295 authoring-only scope contained**: GIVEN PROMPT 1295 worker branch (`work/s18-story-authoring-wave-b-1295`) diff, WHEN inspected, THEN the only files modified by PROMPT 1295 are:
  - `production/epics/lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md` (NEW; this file)
  - `production/epics/lightyear-protocol-verification/EPIC.md` (index update only -- appending the story-009 row)
  - `production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md` (NEW; sibling Wave-B story)
  - `production/epics/round-state-machine/EPIC.md` (index update only -- appending the story-007 row)
  - `reports/PROMPT-1295-s18-story-authoring-wave-b.md` (NEW; the worker report)
  No code under `client/`, `server/`, `shared/`, `tests/`. No GDD edit. No ADR edit. No sprint plan edit. No QA artifact edit. No production session-state edit. No `production/sprint-status.yaml` edit. No `production/stage.txt` edit. No Cargo / Trunk / CI edit. No skill / agent edit.

- [ ] **AC19 -- Worker branch scope contained for the future `/dev-story` worker**: GIVEN the future implementation worker branch (slug recommendation: `work/s18-protocol-snapshot-real-wire-tests-<prompt-N>`), WHEN inspected, THEN it pushes only the worker branch -- never `main`. Files changed at worker time are scoped to:
  - `tests/helpers/real_wire_snapshot.rs` (NEW)
  - `tests/integration/hand-ui/placement_perspective_snapshot_test.rs` (migration at `:285`)
  - `tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs` (migration at `:389`)
  - `tests/integration/hud/hud_opp_class_recipient_mismatch_test.rs` (migration at `:57`)
  - `tests/integration/hud/hud_opp_figurine_test.rs` (migration at `:301`)
  - Optionally: `tools/ci/` (the AC12 grep gate) and/or `tests/integration/presentation/` (the AC17 sentinel) and/or `Cargo.toml` `[dev-dependencies]` (only if the chosen helper strategy requires it; should be unnecessary)
  - This story file's Closure Trail section (status flip + evidence pointers; performed by `/story-done` after worker DONE)
  No production source under `client/src/`, `server/src/`, `shared/src/` is modified by the implementation worker.

---

## Dependencies (must hold before `/dev-story` on this story)

| Dependency | Slug / Story | Why blocking |
|---|---|---|
| `shared/src/protocol.rs::S2CGameSnapshot` shape | Story 002 (Complete) + reconnect-snapshot work (Complete) | The helper writes `S2CGameSnapshot` payloads; its shape must be stable. |
| `client::presentation::game_snapshot_sink_system` (the system the helper exercises) | Existing presentation-layer work (Complete) | The helper validates this system; the system must exist at HEAD. |
| `client::network::game_snapshot_receiver_system` Lightyear drain | Story 003 (Complete) + Story 004 (Complete) | The helper integrates with this drain (or replaces it under Strategy A); the drain must exist at HEAD. |
| Lightyear 0.26 receiver-poke / two-app-fixture API surface | Story 001 verification spike (Complete) | The helper depends on Lightyear's test-friendly API; the verification spike confirmed the API exists. |
| The four flagged test files at the audit's named line offsets | HEAD `origin/main@1345c6b` | The migration is byte-anchored to the audit's named line offsets (`:285`, `:389`, `:57`, `:301`). If a prior commit moves these lines, the worker re-anchors by the call-site pattern, NOT by the stale line number (same convention as the sibling Wave-B story's audit-line drift). |
| Sprint 18 plan file `production/sprints/sprint-18.md` | Sprint 18 plan authoring prompt | The story may be authored before the Sprint 18 plan exists (PROMPT 1295 is branch-only), but `/dev-story` of this story SHOULD wait for Sprint 18 activation (status flip from Draft to Ready). **Soft dependency**. |

**Optional but recommended** (not blocking):

- Coordination with the sibling Wave-B story `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` (`production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md`). The two stories are **parallel-safe**: this story is tests-only; the sibling is a server-side one-line repair. Neither depends on the other.
- Coordination with any future F-05 closure story (`PlayerSnapshot.submitted` client read site). If F-05 lands, the migrated tests in this story MAY want to additionally assert the `submitted` field flows through the snapshot helper correctly. This is OUT OF SCOPE for the initial migration but is a natural follow-on if F-05 closes.
- Light entry in `docs/architecture/tr-registry.yaml` -- when the registry is populated, this story maps to a new `TR-NP-015` ("Snapshot-driven consumer tests exercise the real Lightyear receive/sink path") or an extension of `TR-NP-ALL29`. The TR-registry edit is performed by a separate `/architecture-review` prompt -- NOT by PROMPT 1295.

---

## Test Evidence

**Story Type**: Logic (test infrastructure).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type", Logic stories deliver **automated unit test -- must pass** as BLOCKING evidence. The four migrated tests passing post-migration AND the regression-on-removal observation from AC10 are the required evidence.

**Evidence location**: a new note at `tests/evidence/protocol-story-009-real-wire-snapshot.md` (created by `/dev-story`) recording:

1. The four migrated tests passing on the post-migration worktree (`cargo test --test placement_perspective_snapshot_test --test placement_board_view_team_map_bootstrap_test --test hud_opp_class_recipient_mismatch_test --test hud_opp_figurine_test` output transcript or CI run ID).
2. The proof-of-prevention regression (AC10): in a sibling experimental worktree, `game_snapshot_sink_system` is temporarily disabled and the four tests are re-run; at least three of the four FAIL. The transcript is captured in the evidence note. The temporary regression is then reverted (NOT pushed to main).
3. Confirmation that pre-existing tests in the four files (and their sibling tests in `tests/integration/hand-ui/` / `tests/integration/hud/`) continue to PASS.
4. The chosen helper strategy (A or B per AC3) and a one-paragraph rationale.

CI link / run ID (or local `cargo test` invocation transcript on Windows / MSVC) recorded in the evidence note.

---

## Implementation Notes (advisory; `/dev-story` may deviate with rationale)

- **Strategy A (receiver-poke) is recommended as the default**: it is faster, simpler, and keeps the test surface scoped. The helper inserts the `S2CGameSnapshot` into the App's `MessageReceiver<S2CGameSnapshot>` directly. The `liv-bevy-lightyear` skill should be consulted for the canonical Lightyear 0.26 test-injection idiom. If Lightyear 0.26 does not expose a public test-injection helper, the worker may construct a minimal local helper that writes into the resource that `MessageReceiver` reads from, provided the helper sits ABOVE `game_snapshot_sink_system` in the production chain.
- **Strategy B (two-app fixture)** is acceptable if Strategy A is infeasible (e.g. Lightyear 0.26's `MessageReceiver` is opaque to test code in a way that prevents Strategy A from working). The two-app fixture has higher fidelity but takes longer per test invocation and is more boilerplate. Worker discretion.
- **Helper module location**: `tests/helpers/real_wire_snapshot.rs` is the canonical location. If `tests/helpers/` does not exist as a discoverable test-helper module yet, the worker MAY need to add a minimal `tests/helpers/mod.rs` (or follow the project's existing convention for shared test helpers -- check `tests/integration/` for any pre-existing `helpers.rs` siblings). Worker discretion within `tests/`.
- **No new test cases added by migration**: the migration is mechanical -- replace the direct bus write with the helper call. Test assertions are byte-identical. If the worker wants to ADD coverage for the regression-on-removal scenario as a permanent test, that goes in the AC17 optional sentinel test, NOT in the migrated tests.
- **`tracing` instrumentation**: the helper MAY add a single `tracing::debug!` log on injection ("real-wire snapshot helper: injecting S2CGameSnapshot at tick N for round X"), but this is OPTIONAL.
- **Independence from F-08 sibling rows**: the audit's other F-* rows are out of scope. The sibling Wave-B story `S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` closes F-07; this story closes F-08. F-01 / F-02 / F-03 / F-04 / F-05 / F-06 / F-09 are owned by separate stories and are NOT modified by this story.
- **PROMPT 1086 / PROMPT 1130 historical context**: when the worker writes the helper's doc comment (AC5), they SHOULD reference both prompts by number. The audit's framing -- "this is the same anti-pattern that hid PROMPT 1130 NEW-1130-01 for ~6 weeks" -- is the canonical historical anchor.

---

## Out of Scope

*Handled by neighbouring stories or sibling audit closures -- do NOT implement here:*

- F-01 closure (`S2COpponentDisconnected` send site)
- F-02 closure (`C2SActivateCard` server handler decision)
- F-03 closure (`S2CSessionSettingsUpdated` initial broadcast on join)
- F-04 closure (`S2CSangMepriseReveal` client drain -- ADR-gated)
- F-05 closure (`PlayerSnapshot.submitted` client read site)
- F-06 closure (`C2SClassChoice` dead-code C2S decision)
- F-07 closure (`submissions_received` Placement→Resolution clear -- sibling Wave-B story at `production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md`)
- F-09 closure (`auction_safety_timer` dead-state removal)
- Migration of the additional `PresentationGameSnapshotMessage` test sites enumerated in the Problem section (HUD resolution dim, HUD scoreboard dot, etc.) -- these are not in the audit's named four and may be carried as a sibling Phase 2 follow-on
- Any `ClientGameSnapshotMessage` consumer test migrations (the board-side fan-out) -- the audit's F-08 names only the UI / HUD-side `PresentationGameSnapshotMessage` consumers; board consumers MAY follow the same pattern via a sibling follow-on but are NOT in this story's BLOCKING scope
- Any TR-registry population pass (separate `/architecture-review` prompt)
- Any GDD edit to `design/gdd/network-protocol.md` (the helper validates the existing wire; no GDD wording change is required)
- Any change to `game_snapshot_sink_system`, `S2CGameSnapshot`, or `PresentationGameSnapshotMessage` (these are the things the helper validates -- changing them is OUT OF SCOPE)

---

## QA Test Cases (informal -- for `/qa-plan` to formalise when Sprint 18 activates)

- **AC2 / AC6-AC9 -- Helper injects via the real wire AND all four migrations PASS**
  - **Given**: a client App with the production plugin set, AND the helper module from AC1, AND the four migrated test files
  - **When**: `cargo test --test placement_perspective_snapshot_test --test placement_board_view_team_map_bootstrap_test --test hud_opp_class_recipient_mismatch_test --test hud_opp_figurine_test` is invoked
  - **Then**: all four test files PASS, and the bus messages exercised inside each test originated from `game_snapshot_sink_system` (not from a direct `world.write_message(PresentationGameSnapshotMessage)` call)

- **AC10 -- Regression-on-removal is observable**
  - **Given**: a sibling experimental worktree where `game_snapshot_sink_system` is temporarily disabled in the client plugin
  - **When**: `cargo test` runs the four migrated tests
  - **Then**: at least three of the four FAIL because the downstream consumer assertions reference state that was never populated (the bus message never arrived)

- **AC11 -- No remaining direct-bypass writes in the four named sites**
  - **Given**: `grep -nE "(write_message|\\.write)\\(PresentationGameSnapshotMessage" tests/integration/hand-ui/placement_perspective_snapshot_test.rs tests/integration/hand-ui/placement_board_view_team_map_bootstrap_test.rs tests/integration/hud/hud_opp_class_recipient_mismatch_test.rs tests/integration/hud/hud_opp_figurine_test.rs`
  - **When**: run on post-migration source
  - **Then**: zero matches (the helper call has replaced every direct write)

- **AC13 -- No production-code change** (CI grep gate)
  - **Given**: `git diff <activation HEAD>..HEAD -- client/src/ server/src/ shared/src/`
  - **When**: inspected
  - **Then**: zero lines changed

- **AC14 -- ADR-008 channel-binding invariant preserved**
  - **Given**: the post-migration source
  - **When**: `register_protocol(app)` is inspected
  - **Then**: `S2CGameSnapshot` continues to register on `ReliableChannel` with no reassignment

- **Optional AC12 fence test**
  - **Given**: a hypothetical future PR that adds a `world.write_message(PresentationGameSnapshotMessage(...))` call outside `tests/helpers/`
  - **When**: CI runs the AC12 grep gate
  - **Then**: CI FAILS with an error pointing at the violating call site

---

## Closure Trail (filled by `/story-done` after worker DONE)

- Worker branch:                  *(filled at worker DONE)*
- Worker commit:                  *(filled at worker DONE)*
- Worker source-of-truth base:    *(filled at worker DONE)*
- Integration / merge commit:     *(filled at integration DONE)*
- `/story-done` PROMPT N:         *(filled at /story-done)*
- Evidence note:                  `tests/evidence/protocol-story-009-real-wire-snapshot.md` *(target path)*
- CI run ID / local cargo log:    *(filled at /story-done)*
- Code review verdict:            *(filled at /story-done; lean-mode acceptable per local convention)*
- Chosen helper strategy:         A (receiver-poke) | B (two-app fixture) -- *(filled at worker DONE; with rationale)*
- AC10 regression-on-removal demo: *(transcript / observation captured in the evidence note)*
- Deviations from this story:     *(record any here; none expected for the BLOCKING scope)*

---

## Cross-References

- Source audit: `reports/PROMPT-1202-multiplayer-protocol-state-consistency-bug-audit.md` §2 row F-08
- Historical anti-pattern: `reports/PROMPT-1086-*` (silent-dead-in-prod fix) + `reports/PROMPT-1130-*` NEW-1130-01 (audit caught the dead wire 6 weeks later)
- Lane map: `reports/PROMPT-1287-sprint-18-parallel-lane-readiness-map.md` §3.11 Lane W10 + §5 row SA-5
- Sibling Wave-B story: `production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md` (F-07 closure)
- Sibling audit rows OUT OF SCOPE: F-01 / F-02 / F-03 / F-04 / F-05 / F-06 / F-07 / F-09 (each owned separately)
- Authoring report: `reports/PROMPT-1295-s18-story-authoring-wave-b.md`
- Governing ADRs: ADR-008 (Lightyear Channel Config) + ADR-003 (Cargo Workspace Structure) + ADR-009 + ADR-010 + ADR-011 + ADR-021. All binding; none modified.
