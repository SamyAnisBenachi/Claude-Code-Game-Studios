# Story 015: COUNTERATTACK Inline + INJURED Cross-Keywords + START / END OF TURN Observers

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-004` — COUNTERATTACK inline dispatch (any non-RANGE melee attack against this unit; no proximity restriction). `TR-KW-???` — START OF TURN and END OF TURN timing triggers have no registered TR-ID. Run `/architecture-review` to register missing TRs before marking this story Done.
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-022 (Part 5 — COUNTERATTACK inline dispatch; Part 1 — START OF TURN and END OF TURN observers)
**ADR Decision Summary**: COUNTERATTACK dispatched inline from `resolve_combat` after damage resolves in SS3/SS6. Fires on any non-RANGE melee attack (same-cell OR collision-halted adjacent-cell contact); RANGE excluded. STUN suppresses COUNTERATTACK. START OF TURN dispatched via `start_of_turn_dispatch_system` (normal Bevy system reading `MessageReader<DraftPhaseEntered>`); END OF TURN via `EndOfTurnTriggered` Observer fired on all alive units after SS6.

**BLOCKED**: ADR-018 is Proposed. ADR-022 is Accepted. Stories 001 (scaffold — registers `on_start_of_turn`, `on_end_of_turn` observer stubs, `start_of_turn_dispatch_system`), 005 (SHIELD — for KW-057 SHIELD via INJURED), 011 (RANGE — for KW-056 RANGE via INJURED), and 014 (APPEARANCE + INJURED inline — `eval_injured_bonuses` must be implemented) must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `check_and_apply_counterattack(world, defender, attacker, sub_step)` — inline function called from `resolve_combat` after each attack resolves; no Observer for COUNTERATTACK
- `commands.trigger_targets(StartOfTurnTriggered, entity)` — deferred dispatch for START OF TURN (acceptable in DRAFT phase); fires when Commands flush after `start_of_turn_dispatch_system`; `apply_deferred` must be scheduled after this system (ADR-022 Part 6)
- `MessageReader<DraftPhaseEntered>` — `DraftPhaseEntered` must be registered with `app.add_message::<DraftPhaseEntered>()` in the RSM plugin (not `KeywordPlugin`) per ADR-022 Verification item 5
- `world.trigger_targets(EndOfTurnTriggered, alive_unit)` — synchronous Observer firing at end of SS6 inside `resolve_combat`

**Control Manifest Rules (Feature layer)**:
- Required: COUNTERATTACK guard: check `stun_active` BEFORE keyword presence check — a STUNned unit does not COUNTERATTACK regardless of keyword (KW-058, GDD)
- Required: COUNTERATTACK chain terminates after one exchange — if original attacker has COUNTERATTACK, it retaliates once; the defender does NOT counter-of-counter (KW-048)
- Required: COUNTERATTACK retaliation uses pre-retaliation HP snapshots for each bilateral pair when multiple attackers are involved (KW-049)
- Forbidden: Never trigger COUNTERATTACK from RANGE attacks — RANGE is the sole excluded attack type (KW-005)
- Forbidden: Never trigger COUNTERATTACK when the defender is STUNned (KW-058, design decision D4)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

### COUNTERATTACK Inline Dispatch
- [ ] KW-005: GIVEN a RANGE attacker deals damage to a unit with COUNTERATTACK, WHEN the RANGE attack resolves, THEN COUNTERATTACK does NOT fire — RANGE is the sole excluded attack type; the COUNTERATTACK unit does not retaliate
- [ ] KW-006: GIVEN a unit with COUNTERATTACK has SHIELD active; in SS6 it receives melee damage, WHEN SHIELD absorbs all incoming damage (unit takes 0 HP damage), THEN COUNTERATTACK still fires against the attacker — the unit was attacked regardless of SHIELD absorption; SHIELD consumption and COUNTERATTACK dispatch are independent
- [ ] KW-048: GIVEN unit A (ATK=5, COUNTERATTACK) and unit B (ATK=3, COUNTERATTACK) fight in SS6 — unit A attacks unit B; WHEN B's COUNTERATTACK fires against A, THEN A's COUNTERATTACK does NOT fire a second time; the chain terminates after one COUNTERATTACK exchange; final HP reflects A's initial attack resolved by B's defense, then B's COUNTERATTACK resolved by A's defense — no further retaliation
- [ ] KW-049: GIVEN unit X (ATK=4, COUNTERATTACK) is attacked simultaneously in SS6 by attacker A (ATK=3) and attacker B (ATK=2); WHEN SS6 resolves, THEN X retaliates against A for 4 ATK-effective damage AND X retaliates against B for 4 ATK-effective damage; the HP snapshot used for X's outgoing COUNTERATTACK damage to each attacker is taken before any retaliation damage is applied to X (pre-retaliation snapshot for each bilateral pair)
- [ ] KW-058: GIVEN a STUNned unit receives melee damage in SS3 or SS6, WHEN the attack resolves, THEN COUNTERATTACK does NOT fire — STUN suppresses all keyword hooks including reactive triggers

### INJURED Cross-Keywords (via eval_injured_bonuses from Story 014)
- [ ] KW-055: GIVEN unit X's card grants COUNTERATTACK when INJURED; X has ATK=3 and current_HP=2 (max_HP=4, INJURED=true from prior sub-step); in SS6 attacker A deals 1 damage to X, WHEN SS6 resolves, THEN X's INJURED-granted COUNTERATTACK fires against A for 3 ATK-effective damage; the INJURED-granted COUNTERATTACK was active because X was INJURED before SS6 began
- [ ] KW-056: GIVEN unit X has RANGE granted via INJURED (INJURED=true from prior sub-step); WHEN INJURED is active at SS6 entry, THEN X attacks the nearest enemy within RANGE (from the `max_range` in the INJURED-RANGE card definition) without advancing; X does NOT trigger COUNTERATTACK from the opposing unit — RANGE attackers cannot trigger COUNTERATTACK regardless of INJURED
- [ ] KW-057: GIVEN unit X gains SHIELD via INJURED at the SS3→SS4 boundary (was damaged in SS3); WHEN SS6 attacker A deals damage to X, THEN SHIELD (granted at the SS3→SS4 boundary) absorbs the SS6 attack; the granted SHIELD is active from SS4 onward; it is NOT retroactive to SS3

### START / END OF TURN Observers
- [ ] KW-009a: GIVEN a unit with START OF TURN is alive at DRAFT phase entry; mana ramp and gold income have been applied (RSM Rule 3), WHEN `DraftPhaseEntered` message is received by `start_of_turn_dispatch_system`, THEN START OF TURN fires for that unit; the effect is applied after mana ramp and gold — not before
- [ ] KW-009b: GIVEN unit X is placed in SS1 of round R and is alive at the end of round R's RESOLUTION, WHEN round R+1's DRAFT phase entry fires `DraftPhaseEntered`, THEN START OF TURN fires for X in round R+1 (not in round R — cards placed this round get their first START OF TURN on round R+1)
- [ ] KW-010a: GIVEN a unit with END OF TURN is alive when SS6 completes, WHEN RESOLUTION ends, THEN END OF TURN fires for that unit before the RSM round counter increments
- [ ] KW-010b: GIVEN a unit with END OF TURN entered play on round R (SS1) and survives SS6 of round R, WHEN RESOLUTION ends for round R, THEN END OF TURN fires — a unit that entered play this round is eligible for END OF TURN on the same round it entered

---

## Implementation Notes

*Derived from ADR-022 Part 5 (COUNTERATTACK/INJURED inline) and Part 6 (START OF TURN) and Part 1 (END OF TURN observer):*

**check_and_apply_counterattack (called inline from resolve_combat after SS3/SS6 damage):**
```rust
fn check_and_apply_counterattack(
    world: &mut World,
    defender: Entity,
    attacker: Entity,
    sub_step: u8,
    attack_is_range: bool,  // true for RANGE attacks — COUNTERATTACK excluded
    chain_depth: u8,        // 0 = initial attack; 1 = COUNTERATTACK retaliation; stop at 1
) {
    if attack_is_range { return; }  // KW-005: RANGE excluded

    let Ok(kw_state) = world.get::<UnitKeywordState>(defender) else { return; };
    if kw_state.stun_active { return; }  // KW-058: STUN suppresses COUNTERATTACK first
    if !kw_state.has_keyword(SimpleKeyword::Counterattack) { return; }
    if chain_depth >= 1 { return; }  // KW-048: chain terminates after one exchange

    keyword::effects::apply_counterattack(world, defender, attacker, sub_step);
    // apply_counterattack may call check_and_apply_counterattack with chain_depth=1
    // on the original attacker (if it has COUNTERATTACK); chain_depth=1 returns immediately
}
```

**KW-049 — simultaneous multi-attacker pre-retaliation snapshot:**
- Take HP snapshot of X BEFORE any retaliation damage is applied
- Apply X's COUNTERATTACK damage to A using the snapshot (A's defense stack vs X's ATK)
- Apply X's COUNTERATTACK damage to B using the same snapshot
- Apply A's retaliation damage to X
- Apply B's retaliation damage to X
- Order within each pair determined by combat resolution ordering rules

**start_of_turn_dispatch_system (ADR-022 Part 6 — normal Bevy system):**
```rust
pub fn start_of_turn_dispatch_system(
    mut reader: MessageReader<DraftPhaseEntered>,
    units: Query<(Entity, &UnitKeywordState)>,
    mut commands: Commands,
) {
    for _event in reader.read() {
        for (entity, kw_state) in units.iter() {
            if kw_state.has_keyword(SimpleKeyword::StartOfTurn) {
                commands.trigger_targets(StartOfTurnTriggered, entity);  // deferred
            }
        }
    }
}
// NOTE: apply_deferred must run after this system for commands to flush
```

**KW-009b — round R+1 gate:** Units placed in SS1 of round R are on the board in round R's RESOLUTION. `start_of_turn_dispatch_system` fires on `DraftPhaseEntered` — which is DRAFT entry for round R+1. Units placed in round R are present in the board query by then. This structural ordering satisfies KW-009b without additional logic.

**on_end_of_turn (called after SS6 in resolve_combat, before ResolutionComplete):**
```rust
// In resolve_combat, after SS6, before writing ResolutionComplete:
let alive_units: Vec<Entity> = query_alive_units(world);
for alive_unit in alive_units {
    world.trigger_targets(EndOfTurnTriggered, alive_unit);
}
// KW-010b: unit that entered play this round is alive_units → eligible
```

---

## Out of Scope

- Story 014: `eval_injured_bonuses` implementation (must be Done before KW-055/056/057 are testable)
- Story 011: RANGE targeting implementation (must be Done before KW-056 RANGE-via-INJURED is testable)
- Story 005: SHIELD `check_shield_absorb` (must be Done before KW-057 SHIELD-via-INJURED is testable)
- Story 006: SILENCE stripping INJURED-granted COUNTERATTACK/RANGE/SHIELD (separate story)
- Story 004: STUN state implementation (must be Done — KW-058 tests STUN suppresses COUNTERATTACK)

---

## QA Test Cases

*Automated test specs (Integration story):*

- **KW-005**: RANGE attack does not trigger COUNTERATTACK
  - Given: Player B RANGE unit at Cell 3 attacks Player A unit at Cell 6 (with COUNTERATTACK); `attack_is_range=true`
  - When: `check_and_apply_counterattack` called with `attack_is_range=true`
  - Then: function returns immediately; no COUNTERATTACK damage applied to Player B unit

- **KW-006**: COUNTERATTACK fires despite SHIELD absorption
  - Given: Unit X (COUNTERATTACK, SHIELD active); attacker A deals 5 damage in SS6
  - When: SHIELD check fires first → absorbs all damage (X takes 0 HP); then COUNTERATTACK check runs
  - Then: COUNTERATTACK fires → A takes X's ATK post-modifier damage; X's HP unchanged; X's SHIELD consumed

- **KW-048**: COUNTERATTACK chain terminates
  - Given: Unit A (ATK=5, COUNTERATTACK) vs unit B (ATK=3, COUNTERATTACK) in SS6
  - When: A attacks B → B's COUNTERATTACK fires (chain_depth=1) → A's COUNTERATTACK check runs (chain_depth=1) → returns immediately
  - Then: A HP reduced by B's COUNTERATTACK; B HP reduced by A's initial attack; NO further exchange; total damage events = 2 (initial + one counter)

- **KW-049**: Simultaneous multi-attacker COUNTERATTACK snapshot
  - Given: Unit X (ATK=4, COUNTERATTACK, HP=10); attacker A (ATK=3); attacker B (ATK=2); all attack X simultaneously in SS6
  - When: SS6 resolves; pre-retaliation snapshot HP_X=10 taken
  - Then: X retaliates against A for 4 damage (post-modifier); X retaliates against B for 4 damage (using same HP_X=10 snapshot, not post-A-retaliation HP); X's final HP = max(0, 10 − 3 − 2) = 5

- **KW-058**: STUN suppresses COUNTERATTACK
  - Given: Unit X (COUNTERATTACK, `stun_active=true`); attacker A deals melee damage in SS6
  - When: `check_and_apply_counterattack` called for X
  - Then: `stun_active` check fires first → COUNTERATTACK does not fire; A takes no retaliation damage

- **KW-009b**: START OF TURN fires on round R+1, not R
  - Given: Unit X (START OF TURN effect) placed in SS1 of round R; round R RESOLUTION completes
  - When: Round R+1 DRAFT entry fires `DraftPhaseEntered`
  - Then: `start_of_turn_dispatch_system` finds X in board query; `commands.trigger_targets(StartOfTurnTriggered, X)` dispatched; effect applies in round R+1 (not round R)

- **KW-010b**: END OF TURN fires for units that entered this round
  - Given: Unit X (END OF TURN effect) entered play in SS1 of round R; survives SS6
  - When: After SS6, `on_end_of_turn` fires for all alive units
  - Then: X is in `alive_units` list; END OF TURN fires for X in round R

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/keyword/counterattack_sot_eot_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 004 (STUN — KW-058), Story 005 (SHIELD — KW-057), Story 011 (RANGE — KW-056), Story 014 (INJURED inline — KW-055/056/057) must be Done
- Unlocks: Story 006 (SILENCE + INJURED depends on COUNTERATTACK and eval_injured_bonuses both working)
