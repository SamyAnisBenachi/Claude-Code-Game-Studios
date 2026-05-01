# Story 011: RANGE + RESISTANCE / ARMOR-PIERCING / VULNERABILITY

> **Epic**: Keyword System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-011` — RANGE target selection nearest-enemy; equidistant tie-break via `range_equidistant_select` RNG slot (ADR-005 seed slot registered)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

> ⚠️ RESISTANCE, VULNERABILITY, and ARMOR-PIERCING have no TR-ID in the registry. Stories use `TR-KW-???` placeholders. Run `/architecture-review` to register missing TRs before marking this story Done.

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, `effects.rs` section)
**ADR Decision Summary**: Keyword effect functions live in `server/feature/keyword/effects.rs` and are called by `server/feature/combat/` as plain function calls. RANGE target selection, RESISTANCE reduction, VULNERABILITY increase, and ARMOR-PIERCING AR-zero-out are all stateless — evaluated per-attack from the unit's card definition via `CardCatalog`, not stored in `UnitKeywordState`.

**Readiness Refresh (2026-05-01)**: Revalidated against control manifest version 2026-05-01. ADR-018 is Accepted, the ADR-006 keyword schema amendment is reflected in the current manifest, and Story 001 is Complete. The stale ADR-018 blocker is cleared. The `TR-KW-???` placeholders for RESISTANCE, VULNERABILITY, and ARMOR-PIERCING remain a traceability follow-up before Story Done; they do not change this story's implementation behavior.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: RANGE target selection iterates the lane's cell positions — reads `BoardState` via `world.get::<UnitBoardPosition>(entity)`. No `EventWriter`/`EventReader` — RNG tie-break uses `ServerRng` resource via `world.resource_mut::<ServerRng>()` inside the exclusive `resolve_combat` system (ADR-017 pattern). Verify `world.resource_mut` access pattern against Bevy 0.18 exclusive-system API.

**Control Manifest Rules (Feature layer)**:
- Required: RANGE target selection uses `range_equidistant_select` seed slot from `ServerRng` — not `thread_rng`, not hardcoded first-target (ADR-005)
- Required: `apply_first_strike()` and RANGE attacks called by `server/feature/combat/` — keyword module does not schedule against combat timeline (ADR-018)
- Forbidden: Never bypass BODYGUARD protection check for Spell/Order targeting; RANGE is NOT a Spell/Order — RANGE bypass of BODYGUARD is by design (GDD)
- Forbidden: Never let UNTARGETABLE prevent RANGE attacks — UNTARGETABLE blocks Spell/Order targeting only

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria, Combat Keywords section:*

- [ ] KW-016: GIVEN a RANGE 1-X unit has an enemy BODYGUARD protecting another unit within range, WHEN the RANGE unit selects its target, THEN RANGE selects by proximity (nearest cell in the forward direction within X cells); BODYGUARD's Spell/Order protection does NOT intercept RANGE targeting
- [ ] KW-017: GIVEN a unit with RANGE and FIRST STRIKE, WHEN RESOLUTION executes, THEN the unit attacks in SS3 AND again in SS6; SHIELD consumed in SS3 does NOT protect the same target unit in SS6 (confirmed by KW-037 in Story 005)
- [ ] KW-022: GIVEN a defender with RESISTANCE 2 is attacked by a unit with ARMOR-PIERCING, WHEN the modifier stack resolves, THEN RESISTANCE 2 reduces `ATK_effective` by 2 first; ARMOR-PIERCING sets `AR_defender = 0` independently; RESISTANCE is NOT bypassed by ARMOR-PIERCING
- [ ] KW-061: GIVEN a RANGE unit has an enemy WALL as the nearest enemy within range, WHEN SS6 (or SS3 for FIRST STRIKE) resolves, THEN the RANGE unit attacks the WALL, not any unit beyond it; WALL's movement-blocking behavior (SS5 halt) does not affect RANGE target selection

---

## Implementation Notes

*Derived from ADR-018 effects.rs interface and GDD Detailed Design:*

**RANGE target selection algorithm:**
- `apply_range_attack(attacker: Entity, world: &mut World)` — scans the attacker's lane for enemy units
- Nearest-enemy: sort by `|attacker_cell − target_cell|`; smallest distance wins
- WALL counts as a valid RANGE target (GDD: "If WALL is the nearest enemy, RANGE attacks WALL")
- BODYGUARD does NOT intercept (GDD: "RANGE attacks bypass BODYGUARD") — no BODYGUARD check in RANGE path
- UNTARGETABLE does NOT prevent RANGE attacks — only Spell/Order targeting
- Equidistant tie-break: consume `range_equidistant_select` seed slot from `ServerRng` to deterministically pick among tied targets; ascending lane order for inter-player RNG ordering

**RANGE + FIRST STRIKE (KW-017):**
- RANGE+FS unit attacks in SS3 (via `apply_first_strike` path with RANGE targeting) AND again in SS6 (standard RANGE pass)
- Two separate attack events — SHIELD consumed in SS3 by any attack is gone in SS6 (see Story 005 KW-037)
- No special dual-attack flag needed: `resolve_combat` checks `has_keyword(SimpleKeyword::FirstStrike)` in SS3 and `RangeX { .. }` in SS6 independently

**RESISTANCE / VULNERABILITY / ARMOR-PIERCING (KW-022):**
```
Modifier stack (steps 1-9, per combat-resolution.md):
  Step N: ATK_effective = ATK_raw - resistance_value + vulnerability_value  (both from CardCatalog)
  Step N+1: net_damage = max(0, ATK_effective - AR_defender)
  where AR_defender = 0 if attacker has ARMOR-PIERCING

RESISTANCE is NOT bypassed by ARMOR-PIERCING:
  ARMOR-PIERCING zeroes AR (step N+1 input)
  RESISTANCE reduces ATK_effective (step N output) — independent of AR
  The two modifiers operate at different stack levels
```

---

## Out of Scope

- Story 005 (KW-037): SHIELD consumed in SS3 doesn't protect SS6 — already written
- Story 007 (KW-021): UNTARGETABLE vs RANGE — already written
- Story 012: DEATH chain from RANGE kills — in the DEATH observer story
- Story 015: COUNTERATTACK NOT triggered by RANGE attackers (KW-056, KW-005)

---

## QA Test Cases

*Automated test specs (Logic story):*

- **KW-016**: RANGE bypasses BODYGUARD
  - Given: Lane with Player A RANGE unit at Cell 2; Player B BODYGUARD unit at Cell 4 protecting unit at Cell 5; RANGE max_range=4
  - When: SS6 RANGE target selection runs
  - Then: RANGE attacks Cell 4 (nearest enemy = BODYGUARD unit); BODYGUARD protection not consulted (RANGE is not a Spell/Order); no KW-021-style intercept

- **KW-017**: RANGE+FIRST STRIKE dual attack
  - Given: Player A RANGE+FIRST STRIKE unit at Cell 3; Player B unit at Cell 5 with SHIELD; max_range=3
  - When: SS3 resolves (FIRST STRIKE pass) then SS6 resolves (standard RANGE pass)
  - Then: SS3 attack fires; SHIELD absorbs SS3 attack and is consumed; SS6 attack fires full damage (SHIELD gone); target HP after SS6 = HP-after-SS3 minus SS6 net damage
  - Edge cases: if SS3 kills the target, no SS6 attack (target gone)

- **KW-022**: RESISTANCE + ARMOR-PIERCING modifier stack
  - Given: Attacker ATK=5, ARMOR-PIERCING; defender HP=10, AR=3, RESISTANCE 2
  - When: damage resolves
  - Then: `ATK_effective = 5 − 2 = 3`; `AR_defender = 0` (ARMOR-PIERCING); `net_damage = max(0, 3 − 0) = 3`; target HP = 7
  - Edge cases: `RESISTANCE > ATK_raw` → `ATK_effective = 0`; `net_damage = 0`

- **KW-061**: RANGE attacks WALL if nearest
  - Given: Player A RANGE unit at Cell 2; Player B WALL at Cell 4; Player B standard unit at Cell 6; max_range=5
  - When: RANGE target selection runs in SS6
  - Then: nearest enemy is WALL at Cell 4 (distance 2); standard unit at Cell 6 (distance 4) is farther; RANGE attacks WALL; WALL deals 0 damage in response; unit at Cell 6 is not targeted

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/range_resistance_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold) must be Done; ADR-006 amendment merged (Keyword::RangeX, ResistanceX, VulnerabilityX variants)
- Unlocks: Story 015 (KW-056 — RANGE granted via INJURED depends on RANGE targeting working); Story 017 (cross-keyword FIRST STRIKE × WALL)
