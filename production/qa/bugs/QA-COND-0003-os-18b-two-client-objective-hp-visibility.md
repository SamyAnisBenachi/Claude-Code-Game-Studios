# QA-COND-0003: OS-18b Two-Client Objective HP Visibility

| Field | Value |
|---|---|
| ID | QA-COND-0003 |
| Kind | QA Condition |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Closed |
| Action State | N/A - Closed |
| Reported | 2026-05-05 |
| Source | Sprint 5 QA sign-off and Production-to-Polish gate check |

## Summary

QA-COND-0003 is resolved by the OS-008 live two-client ObjectiveHp visibility
harness and evidence report. The harness proves both connected clients observe
only the final public `ObjectiveHp` value after two same-sub-step damage calls,
with no intermediate HP and no duplicate final HP update.

## Source Evidence

- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records OS-18b live
  two-client objective HP replication visibility as advisory.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries OS-18b live two-client visibility forward as an open QA condition.

## Expected Closure Evidence

Satisfied by the second closure path: an automated end-to-end transport test
proves the live two-client visibility contract.

- A live two-client capture showing objective HP visibility after
  resolution-end sync for both clients.
- An automated end-to-end transport test that proves the same visibility
  contract.
- A documented reclassification explaining why live two-client visibility is no
  longer required for this gate.

## Closure Evidence

Captured 2026-05-05 from `D:\_DEV\claude-code-game-studios`.

- Integrated OS-008 implementation commit: `f097606`
  (`S6-05 evidence: OS-18b two-client objective HP visibility`).
- Harness: `tests/integration/network/os18b_two_client_objective_hp_visibility_test.rs`.
- Evidence report:
  `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.md`.
- `cargo test -p server --test os18b_two_client_objective_hp_visibility_test -- --nocapture`
  passed: 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out.
- Server assertions recorded final authoritative `ObjectiveHp` as `0`, queued
  exactly one `ObjectiveDestroyed` consequence, incremented the real objective
  destroyed counter once, and emitted exactly one RESOLUTION-end
  `ObjectiveDestroyed` message.
- Client A observed post-damage `ObjectiveHp` sequence `[0]`.
- Client B observed post-damage `ObjectiveHp` sequence `[0]`.
- Neither client observed intermediate HP `1` or duplicate final HP `[0, 0]`.

## Current Blocker Status

Closed. QA-COND-0003 is no longer Sprint 6 validation debt after the OS-008
two-client harness and evidence report verified final-only ObjectiveHp
visibility for both clients.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit objective, HUD, board rendering, or networking code.
- Does not claim live two-client evidence exists.
