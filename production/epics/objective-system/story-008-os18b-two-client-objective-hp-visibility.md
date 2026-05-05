# Story 008: OS-18b Two-Client Objective HP Visibility Evidence

> **Epic**: Objective System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirement**: `OS-18b` - two `take_damage()` calls targeting the same objective in the same RESOLUTION sub-step must produce the correct final `ObjectiveHp`, exactly one consequence path, and no client-visible intermediate HP replication update.

**Trace Validation (2026-05-05)**: `OS-18b` is an Objective GDD acceptance gap left open by Story 007 completion notes. The ECS-side no-duplicate consequence path is already covered by Objective System tests, but live two-client transport visibility remains unproven. This story closes open QA condition [QA-COND-0003](../../qa/bugs/QA-COND-0003-os-18b-two-client-objective-hp-visibility.md) by requiring two-client evidence.

**Story Ownership**: Objective System owns this story because OS-18b is an Objective GDD / Story 007 acceptance gap. The Lightyear harness is the evidence mechanism only.

**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](docs/architecture/adr-001-objective-identity-unicast.md); [ADR-010: RSM Phase Event Bus](docs/architecture/adr-010-rsm-event-bus.md)

**ADR Decision Summary**: `ObjectiveHp` is public replicated state visible to both clients, while objective identity remains server-only except for targeted owner unicast. Objective destruction is queued during RESOLUTION sub-step processing and emitted at the RESOLUTION-end sync boundary, not mid-sub-step.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: This story verifies Lightyear transport visibility with a live two-client harness. It must not introduce gameplay behavior changes unless the harness exposes a real transport bug in the ObjectiveHp replication or RESOLUTION-end sync path.

**Control Manifest Rules (Feature / Foundation layers)**:
- Required: `ObjectiveHp` is replicated public state; `ObjectiveIdentity` is never replicated as an ECS component.
- Required: Objective damage flows through `take_damage()`; no harness-only path may mutate HP directly.
- Required: RESOLUTION-end network output remains ordered on `ReliableChannel`.
- Forbidden: Do not emit or expose intermediate sub-step Objective HP states to clients.
- Guardrail: Evidence must use the existing Lightyear protocol/channel patterns; do not add a new channel or protocol workaround for this story.

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to OS-18b evidence:*

- [ ] **OS-18b final server HP**: GIVEN one live two-client server tick/sub-step and one target objective with known HP, WHEN two `take_damage()` calls hit that same objective before the replication flush, THEN the final authoritative server `ObjectiveHp` equals the saturated result of both calls.
- [ ] **OS-18b consequence path once**: GIVEN the same tick/sub-step, WHEN the first or combined damage reaches 0 HP, THEN the Objective System consequence path runs exactly once for that objective.
- [ ] **OS-18b destruction event once**: GIVEN the same tick/sub-step reaches 0 HP, WHEN RESOLUTION-end sync emits destruction output, THEN the destruction event for that objective is visible exactly once.
- [ ] **OS-18b both clients final-only HP**: GIVEN both clients are connected for the same tick/sub-step, WHEN objective HP replication is observed, THEN both clients observe only the final `ObjectiveHp` value.
- [ ] **OS-18b no intermediate HP**: GIVEN the first `take_damage()` call would create an intermediate HP value, WHEN client observations are inspected, THEN neither client observes that intermediate HP value.
- [ ] **OS-18b no duplicate final HP**: GIVEN the final HP value is replicated, WHEN client observations are inspected for the target objective, THEN neither client observes duplicate final HP updates in the same tick/sub-step.
- [ ] **QA-COND-0003 closure evidence**: GIVEN the harness passes, WHEN the story is closed, THEN `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.md` records the command, server assertions, both client observation sequences, and whether the optional raw log was captured.

---

## Readiness Criteria

- [ ] Harness target is `tests/integration/network/os18b_two_client_objective_hp_visibility_test.rs`.
- [ ] Evidence output target is `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.md`.
- [ ] Optional raw log target is `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.log`.
- [ ] The story remains evidence-harness only; do not require gameplay behavior changes unless the harness exposes a real transport bug.
- [ ] `QA-COND-0003` remains open until evidence exists; this story is the planned closure path.

---

## Implementation Notes

Build a live two-client Lightyear harness that drives the authoritative Objective System path. A useful primary fixture is:

1. Start one server app and two client connections using the existing network integration test pattern.
2. Spawn or arrange one opponent objective with known HP, such as HP 3.
3. In one server tick/sub-step, call `take_damage()` twice against the same objective before clients process replicated updates. Example: damage 2, then damage 2, producing final HP 0 with intermediate HP 1.
4. Advance the server/client apps only through the intended replication/sync boundary.
5. Record the authoritative final HP, consequence count, destruction event count, and each client's observed `ObjectiveHp` sequence for the target entity.

The expected client sequence for the primary fixture is exactly one final value per client, such as `[0]`. Values such as `[1, 0]`, `[0, 0]`, or client A seeing `[0]` while client B sees `[]` fail the story.

If the harness exposes a real transport or scheduling bug, document it in the evidence file and fix the bug under a follow-up implementation change. Do not pre-emptively alter gameplay semantics as part of the evidence story.

---

## Out of Scope

- Implementing this harness in the story-docs change.
- Editing `production/qa/bugs/QA-COND-0003-os-18b-two-client-objective-hp-visibility.md` before evidence exists.
- Changing Objective System gameplay behavior unless a later harness run proves a real transport bug.
- HUD, Board Rendering, animation, screenshot polish, or manual visual-only proof.
- Editing `production/sprint-status.yaml`, `production/session-state/**`, `design/assets/**`, or unrelated accessibility ADR/docs.

---

## QA Test Cases

*Implement against these cases when the harness story is picked up.*

- **AC-1: final server HP after two same-sub-step damage calls**
  - Given: Two live clients connected to one server; target objective starts at HP 3.
  - When: The server processes `take_damage(target_lane, attacker, 2)` followed by `take_damage(target_lane, attacker, 2)` in the same server tick/sub-step.
  - Then: The authoritative server `ObjectiveHp` for that objective is 0.
  - Edge cases: Damage values may vary, but at least one fixture must include a client-invisible intermediate HP value.

- **AC-2: consequence and destruction output are not duplicated**
  - Given: The same fixture reaches HP 0.
  - When: The Objective System consequence path and RESOLUTION-end sync complete.
  - Then: Consequence handling for the target objective runs exactly once, and the destruction event for that target objective is emitted/observed exactly once.
  - Edge cases: The second `take_damage()` call must be proven to be a no-op after HP reaches 0.

- **AC-3: both clients observe final ObjectiveHp only**
  - Given: Both clients subscribe to the replicated target objective.
  - When: Replication is flushed after the two damage calls.
  - Then: Client A and Client B each observe exactly one `ObjectiveHp` update for the target objective, and the value equals the final server HP.
  - Edge cases: Fail if either client observes the intermediate HP value, misses the final HP value, or receives the final HP value more than once.

- **AC-4: evidence file is sufficient to close QA-COND-0003**
  - Given: The harness passes locally or in CI.
  - When: The developer writes `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.md`.
  - Then: The evidence includes the exact command, git commit or branch under test, server assertion summary, both client observation sequences, pass/fail verdict, and raw log path if captured.

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/network/os18b_two_client_objective_hp_visibility_test.rs` - live two-client Lightyear harness proving OS-18b visibility.
- `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.md` - closure evidence for `QA-COND-0003`.

**Optional evidence**:
- `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.log` - raw harness log if useful for audit or debugging.

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 (`take_damage()` interface), Story 005 (destruction consequence path), Story 007 (RESOLUTION-end sync and OS-18b advisory note), and existing Lightyear network integration test infrastructure.
- Unlocks: Closure of `QA-COND-0003` once the harness and evidence file exist and pass.
