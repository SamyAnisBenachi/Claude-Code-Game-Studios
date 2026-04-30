---
name: objective-system QL-STORY-READY review
description: Gate review results for 7 objective-system stories — verdicts, key gaps, and test file assignments
type: project
---

QL-STORY-READY gate performed 2026-04-30 on Objective System epic (7 stories: 001–007).

## Verdicts

- Story 001 (State Model): ADEQUATE
- Story 002 (Fake Assignment & Config Guards): ADEQUATE
- Story 003 (Identity Unicast Delivery): ADEQUATE — advisory flag: zero BLOCKING criteria; pre-sprint discussion required on test strategy for OS-17 (Option A: pure-function unit test on message assembly; Option B: defer to Lightyear harness)
- Story 004 (Damage Interface): ADEQUATE
- Story 005 (Destruction Consequence Path): ADEQUATE — test specs must scope OS-13a and OS-18a as queue-population only (not broadcast timing, which is Story 007)
- Story 006 (D4 Fake Reward Draw): ADEQUATE (revised 2026-04-30) — OS-27 resolved via named constant FAKE_REWARD_POOL_FILTER; test asserts all four fields are None; code review gate verifies constant is the one passed to draw_random
- Story 007 (ResolutionPhaseEntered & RESOLUTION-end Sync): ADEQUATE

## Story 006 Resolution (2026-04-30)

OS-27 was unblocked by lead programmer proposal: extract filter as named constant FAKE_REWARD_POOL_FILTER in the objective module. Unit test asserts all four fields (rarity, class, card_type, max_cost) are None. Code review gate verifies the constant is the one passed at the draw_random call site. No runtime argument inspection required. This approach is ADEQUATE under the no-mocks rule — the observation target is a compile-time constant, not a runtime call argument.

FAKE_REWARD_POOL_FILTER must be pub(crate) or pub, defined as a named const/static (not a function or lazy_static), so the code review gate is unambiguous. If PoolFilter gains new fields, the test fails at compile time — drift is surfaced automatically.

## Story 003 Advisory Note

OS-17 is the only criterion; it is ADVISORY (two-client live Lightyear test). If lead programmer determines the message assembly can be extracted as a pure function, elevate OS-17 to BLOCKING and write a unit test in tests/unit/objective/unicast_dispatch_test.rs. This decision should happen before sprint start.

## Key Scoping Notes Absorbed into Test Specs

- OS-13a appears in both Story 005 and Story 007. Story 005 scope: assert event is queued/held internally. Story 007 scope: assert queued events are broadcast at ResolutionPhaseEntered. Tests must NOT assert broadcast in Story 005.
- OS-18a appears in both Story 005 and Story 007. Story 005 scope: lane 1 consequence fires before lane 3 in the internal queue order. Story 007 scope: broadcast order to clients is ascending lane order.
- OS-25 (Garde-Temps) notes "confirm when Class System GDD is authored" — this is a spec note only, not a gap; the criterion is testable as written using take_damage(lane, attacker, objective_hp).
- GDD OS-17 criterion text references "ClientId" but ADR-001 and the story packet use "PeerId". Story packet is authoritative. GDD text is stale — programmer must use PeerId.

## Test File Assignments

- Story 001: tests/unit/objective/objective_state_test.rs
- Story 002: tests/unit/objective/fake_assignment_test.rs
- Story 003: (no blocking test file — ADVISORY only; future: tests/unit/objective/unicast_dispatch_test.rs if Option A adopted)
- Story 004: tests/unit/objective/damage_interface_test.rs
- Story 005: tests/unit/objective/consequence_path_test.rs
- Story 006: tests/unit/objective/fake_reward_test.rs (full — OS-27 resolved; all ACs BLOCKING)
- Story 007: tests/integration/objective/resolution_sync_test.rs

**Why:** Story 006 blocked same pattern as card-data-pool Story 006 — observable call arguments require either an inspector pattern on the real struct or demotion to integration test. No mocks allowed.

**How to apply:** When Story 006 is revised and returns for re-review, check that OS-27 criterion names the specific observation mechanism. That is the only unresolved gap.
