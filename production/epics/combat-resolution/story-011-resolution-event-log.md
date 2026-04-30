# Story 011: ResolutionEvent Log Completeness

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-???` (TR-CR-001 full, TR-CR-015 full — unregistered)

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

- [ ] **CR-30**: GIVEN `S2CPlacementReveal` is broadcast, WHEN RESOLUTION begins, THEN PlacementReveal is sent before any sub-step 1 effects execute AND contains both players' full placements in one atomic message
- [ ] **CR-32**: GIVEN RESOLUTION completes all 6 sub-steps, WHEN RESOLUTION_COMPLETE fires, THEN `S2CResolutionEvent` MUST contain:
  - Exactly one `SubStepEntry` per executed sub-step
  - One `CombatDamage` record per damage application (including non-lethal hits)
  - One `UnitRemovedRecord` per killed unit
  - One `GoldAwarded` record per gold event
  - One `KeywordTriggered` record per APPEARANCE/DEATH/COUNTERATTACK/FINAL_BLOW activation
  - All in chronological `(sub_step, trigger_index)` order
  - `S2CPhaseChanged(DRAFT_SHOP)` must NOT be observed by any client before `S2CResolutionEvent` is received

---

## Implementation Notes

*Derived from ADR-017 Decision 3 and GDD CR-32 content requirement:*

This story is an integration verification story — it does not add new behaviour but verifies that the `ResolutionLog` built across Stories 001–010 is correctly serialized, broadcast in one message, and ordered correctly.

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

**Ordering verification**: All log entries pushed via `log.push(...)` during sub-step execution are already in chronological order — the log is append-only and sub-steps run sequentially. No reordering needed. Integration test: assert that for any two entries `A` and `B` in the log, if `A.sub_step < B.sub_step`, then `A` precedes `B` in `log.events`.

**OQ-D ordering**: The integration test must verify that in the same Bevy frame, `S2CResolutionEvent` is enqueued BEFORE `ResolutionComplete` is written. The RSM only reads `ResolutionComplete` on the next tick. `S2CPhaseChanged` is sent in the RSM tick that processes `ResolutionComplete` — which is always a later Bevy frame than `S2CResolutionEvent`.

**SHIELD-blocked damage (CR-32)**: A `CombatDamage` entry IS required even when SHIELD absorbs damage — with `was_blocked_by_shield: true` and `damage_amount: 0` (or the pre-absorption amount). The absence of a damage number is the player-facing signal that SHIELD blocked; the log entry is still required for replay correctness.

---

## Out of Scope

- Stories 001–010: Individual event emissions (this story verifies completeness, not individual sub-step logic)
- Board Rendering GDD: Client-side animation replay of the log (owned by that system)

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

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/combat/resolution_event_log_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Stories 003–010 (all event-emitting sub-steps must be implemented before log completeness can be verified)
- Unlocks: Epic closed via `/story-done` after this story passes
