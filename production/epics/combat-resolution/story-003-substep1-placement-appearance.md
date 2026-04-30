# Story 003: Sub-step 1 — Placement Commit + APPEARANCE Triggers

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-???` (TR-CR-004 — unregistered)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: Sub-step 1 is a sequential function call within `resolve_combat`. All PlacementBuffer cards enter the board simultaneously; APPEARANCE triggers fire immediately; DEATH from APPEARANCE effects is deferred until all SS1 APPEARANCE effects complete; cross-lane triggers (CHANGE LANE, Strich) execute after SS1 and before SS2.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Entity spawning in Bevy 0.18 uses Required Components — `commands.spawn((Sprite::from_image(handle), Transform::default()))` — NEVER `SpriteBundle`. Since `resolve_combat` is an exclusive system using `world: &mut World`, spawning uses `world.spawn(...)` or deferred via `Commands` from `world.commands()`. `despawn()` replaces `despawn_recursive()` since Bevy 0.16.

**Control Manifest Rules (Feature layer)**:
- Required: `S2CPlacementReveal` enqueued BEFORE any `world.spawn(...)` for placed units (ADR-007 ordering rule); APPEARANCE trigger chains are sequential, not simultaneous
- Forbidden: Never spawn ECS entities before `S2CPlacementReveal` is enqueued; never use `SpriteBundle`, `despawn_recursive()`
- Guardrail: SS1 must complete within the overall 15ms RESOLUTION budget; APPEARANCE chains have no budget of their own but contribute to the 10,000-iteration counter

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-24**: GIVEN a unit with an APPEARANCE ability enters play in sub-step 1, WHEN sub-step 1 executes, THEN the APPEARANCE ability fires before sub-step 2 begins
- [ ] **CR-38**: GIVEN unit A's APPEARANCE trigger deals lethal damage to unit B in sub-step 1, AND unit C also has an APPEARANCE trigger in sub-step 1, WHEN sub-step 1 executes, THEN unit C's APPEARANCE fires before unit B's DEATH trigger; unit B's DEATH trigger fires only after ALL sub-step 1 APPEARANCE effects complete
- [ ] **CR-39**: GIVEN a unit with a CHANGE LANE trigger activates in sub-step 1, WHEN all sub-step 1 effects complete, THEN the CHANGE LANE executes before sub-step 2 begins; the unit's new lane position is used for sub-step 2 CHARGE X movement
- [ ] **CR-40**: GIVEN unit A's APPEARANCE trigger applies STUN to unit B (which has CHARGE X) in sub-step 1, WHEN sub-step 2 executes, THEN unit B does NOT advance via CHARGE X (STUN suppresses sub-step 2); WHEN sub-step 5 executes, THEN unit B does NOT advance (STUN suppresses sub-step 5)

---

## Implementation Notes

*Derived from ADR-017 Architecture Diagram and GDD Sub-step 1 rules:*

```
SS1 execution order (within apply_placements):
1. Commit PlacementBuffer → spawn all placed unit entities across all lanes
2. Collect all units with APPEARANCE keyword
3. Fire all APPEARANCE triggers (in lane order for determinism)
   - Each trigger may: deal damage, apply STUN, apply SILENCE, etc.
   - If any unit's HP reaches 0 → mark for deferred DEATH (DO NOT remove yet)
4. After ALL APPEARANCE triggers complete:
   - Fire DEATH triggers for units marked in step 3 (sequential in lane order)
5. Execute cross-lane triggers (CHANGE LANE, Strich) that fired in steps 3-4
   - These execute AFTER all SS1 APPEARANCE and DEATH effects
   - BEFORE SS2 CHARGE X begins
6. Update UnitSnapshot positions after cross-lane moves
```

**STUN propagation**: When APPEARANCE applies STUN to a unit, set `snapshot.is_stunned = true` immediately. The STUN flag is checked in SS2 (`execute_charge_x`) and SS5 (`execute_movement`) — if `is_stunned`, skip the unit entirely.

**Iteration counter**: Each APPEARANCE trigger fired and each DEATH trigger fired increments `iter_count`. This guards against pathological APPEARANCE → DEATH → APPEARANCE chains.

---

## Out of Scope

- Story 001: The scaffold that calls `apply_placements()`
- Story 004: SS2 CHARGE X and SS5 movement (which respect the STUN set here)
- Story 006: SS4 formal dead removal (DEATH from SS3/SS6 kills; SS1 DEATH chains are handled within this story's SS1 pass)

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-24** (APPEARANCE fires before SS2):
  - Given: Synthetic board with 1 placed unit with APPEARANCE effect (deals 1 damage to adjacent enemy)
  - When: SS1 executes
  - Then: `KeywordTriggered { keyword: APPEARANCE }` appears in ResolutionLog before any SS2 entries

- **CR-38** (DEATH deferred until all APPEARANCE complete):
  - Given: Units A (APPEARANCE kills B), C (APPEARANCE buffs self), B (0 HP after A's trigger)
  - When: SS1 executes
  - Then: log order = A's APPEARANCE → C's APPEARANCE → B's DEATH (not A's APPEARANCE → B's DEATH → C's APPEARANCE)

- **CR-39** (CHANGE LANE before SS2):
  - Given: Unit with CHANGE LANE trigger in APPEARANCE
  - When: SS1 completes, SS2 begins
  - Then: unit's lane in SS2 is the post-CHANGE-LANE lane; `UnitChangedLane` entry in log is before first `SubStepBegin(2)` entry

- **CR-40** (STUN suppresses SS2 + SS5):
  - Given: Unit B has CHARGE X=2; unit A's APPEARANCE applies STUN to B
  - When: SS1 completes, SS2 runs, SS5 runs
  - Then: B's cell position is unchanged after SS2; B's cell position is unchanged after SS5

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/substep1_placement_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (`apply_placements` function signature defined in scaffold)
- Unlocks: Story 004 (movement uses board state established by SS1), Story 005 (SS3 attacks units placed in SS1)
