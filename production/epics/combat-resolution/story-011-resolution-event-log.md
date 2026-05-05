# Story 011: ResolutionEvent Log Completeness

> **Epic**: Combat Resolution
> **Status**: Complete
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-014` (CR-30 PlacementReveal atomic timing) and `TR-CR-015` (CR-32 ResolutionEvent content completeness and ordering)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture (Decision 3) + ADR-008: Lightyear Channel Config
**ADR Decision Summary**: `S2CResolutionEvent` is a single reliable-broadcast Lightyear message sent after all 6 sub-steps complete. It contains a `Vec<ResolutionEvent>` in chronological `(sub_step, trigger_index)` order. `S2CPhaseChanged(DRAFT_SHOP)` must not be observable before `S2CResolutionEvent`. `S2CPlacementReveal` is a separate message broadcast before SS1.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: OQ-D ordering invariant (ADR-008): `S2CResolutionEvent` and `S2CPhaseChanged` both travel on `ReliableChannel`. The `OrderedReliable` channel guarantees FIFO — enqueue `S2CResolutionEvent` before `ResolutionComplete` is written; RSM enqueues `S2CPhaseChanged` only AFTER receiving `ResolutionComplete`. See Lightyear 0.26 Verification Checklist item 10 (`OrderedReliable` guarantees FIFO).

**Control Manifest Rules (Feature layer)**:
- Required: `S2CResolutionEvent.events` contains exactly one `SubStepBegin` per executed sub-step; one `CombatDamage` per damage application; one `UnitRemoved` per killed unit; one `GoldAwarded` per gold event; one `KeywordTriggered` per APPEARANCE/DEATH/COUNTERATTACK/FINAL_BLOW activation; all in chronological order
- Forbidden: Never send `S2CPhaseChanged` before `S2CResolutionEvent` on the reliable channel; never omit a non-lethal `CombatDamage` entry (CR-32 requires ALL damage applications including non-lethal)

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-30 / TR-CR-014**: GIVEN `S2CPlacementReveal` is broadcast, WHEN RESOLUTION begins, THEN PlacementReveal is sent before any sub-step 1 effects execute AND contains both players' full placements in one atomic message.
- [ ] **CR-32 / TR-CR-015 - protocol schema**: GIVEN the ADR-017 resolution log is serialized, WHEN `S2CResolutionEvent.events` is inspected, THEN the shared protocol contains typed entries for every required CR-32 replay category: `SubStepBegin` (GDD `SubStepEntry`), `CombatDamage`, `UnitRemoved` (GDD `UnitRemovedRecord`), `GoldAwarded`, and `KeywordTriggered`.
- [ ] **CR-32 / TR-CR-015 - content completeness**: GIVEN RESOLUTION completes all executed sub-steps, WHEN `S2CResolutionEvent` is built, THEN it contains exactly one `SubStepBegin` entry per executed sub-step, one `CombatDamage` record per damage application including non-lethal and SHIELD-blocked hits, one `UnitRemoved` record per killed unit, one `GoldAwarded` record per gold event, and one `KeywordTriggered` record per APPEARANCE/DEATH/COUNTERATTACK/FINAL_BLOW activation.
- [ ] **CR-32 / TR-CR-015 - ordering**: GIVEN the event log contains entries from multiple sub-steps and triggers, WHEN the batch is serialized, THEN entries remain in chronological `(sub_step, trigger_index)` order with no post-hoc reordering that can move same-trigger effects across their emission order.
- [ ] **CR-32 / TR-CR-015 - phase delivery**: GIVEN RESOLUTION completes, WHEN server network output and RSM phase output are observed at frame boundaries, THEN one complete `S2CResolutionEvent` reliable broadcast is enqueued before `ResolutionComplete` is written, and `S2CPhaseChanged(DRAFT_SHOP)` is not observable before that full batch.

---

## Implementation Notes

*Derived from ADR-017 Decision 3 and GDD CR-32 content requirement:*

This story is an integration/protocol completion story. It must not change gameplay rules or individual sub-step behavior, but it may need to implement missing protocol/log serialization so the `ResolutionLog` built across Stories 001-010 is correctly serialized, broadcast in one message, and ordered correctly.

**Readiness repair note (2026-05-05)**: Current code review found that `shared/src/protocol.rs` does not yet expose the full ADR-017/CR-32 `ResolutionEvent` schema, `resolve_combat` currently enqueues an empty `S2CResolutionEvent`, and objective resolution sync can emit a separate `S2CResolutionEvent` after `ResolutionComplete`. `/dev-story` must treat those as in-scope integration gaps: consolidate RESOLUTION replay data into one complete combat-owned batch before `ResolutionComplete`, then verify ordering against the RSM phase change.

If implementation touches Bevy combat systems, shared protocol, or Lightyear send/registration code, future `/dev-story` must use `liv-bevy-018` and `liv-bevy-lightyear`.

**Log population checklist** — verify each event type is emitted by the correct story:

| Event type | Emitted by | Required by CR-32 |
|---|---|---|
| `SubStepBegin { sub_step }` | Story 001 (scaffold) | ✅ one per sub-step |
| `UnitPlaced { unit_id, player, lane, cell }` | Story 003 (SS1) | ✅ |
| `CombatDamage { attacker, defender, damage, shield_blocked, sub_step }` | Stories 005, 007, 008 | ✅ ALL damage events including 0-damage |
| `UnitMoved { unit_id, from, to, sub_step }` | Story 004 (SS2+SS5) | ✅ |
| `UnitChangedLane { unit_id, from_lane, to_lane, sub_step }` | Story 003 (SS1) | ✅ |
| `UnitRemoved { unit_id, lane, cell }` | Story 006 (SS4) + Story 007 (SS6 cleanup) | ✅ one per killed unit |
| `KeywordTriggered { unit_id, keyword, sub_step }` | Stories 003, 005, 006, 007 | ✅ APPEARANCE, DEATH, COUNTERATTACK, FINAL_BLOW |
| `GoldAwarded { player, amount, reason }` | Stories 006, 009 | ✅ kill gold + objective gold |
| `ObjectiveDamage { attacker, lane, damage, hp_after }` | Story 009 | ✅ |
| `ObjectiveDestroyed { lane, owner, is_fake }` | Story 009 | ✅ |
| `GameOver { loser, reason }` | Story 009 / Story 001 (budget abort) | ✅ |

**Ordering verification**: All log entries pushed via `log.push(...)` during sub-step execution must remain in chronological order because the log is append-only and sub-steps run sequentially. No sorting pass should be needed for normal combat entries. Integration test: assert that for any two entries `A` and `B` in the log, if `A.sub_step < B.sub_step`, then `A` precedes `B` in `log.events`; same-sub-step entries retain `trigger_index`/emission order.

**OQ-D ordering**: The integration test must verify that in the same Bevy frame, `S2CResolutionEvent` is enqueued BEFORE `ResolutionComplete` is written. The RSM only reads `ResolutionComplete` on the next tick. `S2CPhaseChanged` is sent in the RSM tick that processes `ResolutionComplete` — which is always a later Bevy frame than `S2CResolutionEvent`.

**SHIELD-blocked damage (CR-32)**: A `CombatDamage` entry IS required even when SHIELD absorbs damage — with `was_blocked_by_shield: true` and `damage_amount: 0` (or the pre-absorption amount). The absence of a damage number is the player-facing signal that SHIELD blocked; the log entry is still required for replay correctness.

---

## Out of Scope

- Stories 001–010: Individual event emissions (this story verifies completeness, not individual sub-step logic)
- Board Rendering GDD: Client-side animation replay of the log (owned by that system)
- Gameplay rule changes, balance changes, or new combat mechanics

**Performance budget**: No new gameplay algorithm cost is expected beyond serializing already-emitted log entries. The RESOLUTION batch must remain within the ADR-017 server RESOLUTION budget (`<= 15 ms`) and the ADR-002 per-round message target (`< 1 KB/round/player`) unless the implementation records a measured exception for review.

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-30** (PlacementReveal content and timing):
  - Given: Two players with 2 placed units each
  - When: `resolve_combat` runs
  - Then: `S2CPlacementReveal.placements` contains all 4 units for both players; the message is enqueued before any `UnitPlaced` entry in `S2CResolutionEvent`

- **CR-32** (SubStepBegin entries):
  - Given: A full RESOLUTION with actions in all 6 sub-steps
  - When: `S2CResolutionEvent.events` is inspected
  - Then: exactly 6 `SubStepBegin` entries, one for each sub-step (indices 1–6)

- **CR-32** (CombatDamage completeness including non-lethal):
  - Given: RESOLUTION where unit A deals 3 damage to unit B (non-lethal, B survives) and then a second attack deals 2 more damage
  - When: log inspected
  - Then: 2 separate `CombatDamage` records for the A→B pair; neither omitted

- **CR-32** (SHIELD-blocked damage recorded):
  - Given: SHIELD unit attacked by FS unit in SS3
  - When: log inspected
  - Then: `CombatDamage { was_blocked_by_shield: true }` present; damage animation hint present for client

- **CR-32** (S2CPhaseChanged ordering):
  - Given: Integration test with RSM and Combat Resolution connected
  - When: RESOLUTION completes; RSM advances to DRAFT_SHOP
  - Then: `S2CResolutionEvent` was enqueued in frame N; `S2CPhaseChanged(DRAFT_SHOP)` was enqueued in frame N+1 or later (verified by checking message queue state at frame boundaries)

- **CR-32** (single complete batch):
  - Given: Objective damage/destruction and gold awards occur during SS6
  - When: RESOLUTION completes
  - Then: objective, gold, damage, removal, keyword, and sub-step entries appear in the same `S2CResolutionEvent`; no second RESOLUTION log batch is emitted after `ResolutionComplete`

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/combat/resolution_event_log_test.rs` — must exist and pass

**Status**: [x] Exists and passes (`cargo test -p server --test resolution_event_log_test`)

---

## Dependencies

- Depends on: Story 003 (Complete - SS1 placement/APPEARANCE emits placement and keyword trace data)
- Depends on: Story 004 (Complete - SS2/SS5 movement emits movement trace data)
- Depends on: Story 005 (Complete - SS3 FIRST STRIKE emits damage and FINAL_BLOW trace data)
- Depends on: Story 006 (Complete - SS4 dead removal emits removal, DEATH, and kill-gold trace data)
- Depends on: Story 007 (Complete - SS6 melee, SHIELD, and COUNTERATTACK emit damage/keyword trace data)
- Depends on: Story 008 (Complete - RANGE targeting emits ranged damage trace data)
- Depends on: Story 009 (Complete - objective damage/GAME_OVER emits objective and reward trace data)
- Depends on: Story 010 (Complete - persistent keyword states closed at `7e0a213`)
- Unlocks: Epic closed via `/story-done` after this story passes

---

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 5/5 passing.
**Verification**: Current `main` includes worker commit `06d5b1744c39e3f6be97ffedb11c3dd99e489c12` and integrated implementation commit `73ad695`. CR-30 / TR-CR-014 and CR-32 / TR-CR-015 are covered by `tests/integration/combat/resolution_event_log_test.rs`; adjacent combat/objective regression coverage also passed for objective damage/gameover, objective resolution sync, RANGE targeting, SS4 dead removal, and SS6 SHIELD/COUNTERATTACK.
**Test Evidence**: `cargo test -p server --test resolution_event_log_test` passed 3/3. Requested adjacent regression command passed 27/27. `cargo check --workspace` passed. `git diff --check` passed.
**Deviations**: None blocking. Advisory only - story manifest version is 2026-05-01 while current control manifest is 2026-05-05; no conflicting current GDD, ADR-017, ADR-008, Bevy 0.18, or Lightyear 0.26 rule found during closure.
**Scope**: Implementation scope stayed within the COMBAT-011 integration boundary; objective resolution sync was touched only to consolidate objective events into the single combat-owned `S2CResolutionEvent` batch before `ResolutionComplete`.
**Code Review**: Lean mode; QL-TEST-COVERAGE and LP-CODE-REVIEW gates skipped because `production/review-mode.txt` is unset and `/story-done` defaults to lean mode.
**Tech Debt**: None logged.
