# Story 002: Movement Formulas — repel_destination + attract_destination

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-002 — CHARGE X bonus movement applied at sub-step 2; cells parameter clamped per Board/Lane F1
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, movement.rs section)
**ADR Decision Summary**: Movement keywords REPEL X and ATTRACT X are implemented as pure functions in `server/feature/keyword/movement.rs`. Both use i32 intermediate arithmetic to prevent u8 underflow, clamp to [1, 8], and return u8. No world access — pure function call from effects.rs.

**BLOCKED**: ADR-018 is Proposed — must be Accepted before opening this story. Story 001 (module scaffold) must also be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW (pure functions, no Bevy API)
**Engine Notes**: Pure functions — no Bevy API surface. Rust integer arithmetic only. Intermediate computation must use i32 to prevent u8 underflow/overflow.

**Control Manifest Rules (Feature layer)**:
- Required: Pure formula functions must be deterministic — no RNG, no world access (ADR-005)
- Forbidden: Never use `u8` arithmetic for intermediate displacement calculations — underflow produces incorrect clamped results in release mode

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Formulas and Acceptance Criteria, scoped to movement formula correctness:*

- [ ] KW-029a: `repel_destination(target_cell=2, owner=PlayerA, x=3)` → `clamp(2 + (-1)*3, 1, 8) = 1` (clamped at board edge)
- [ ] KW-029b: `repel_destination(target_cell=5, owner=PlayerB_pushed_by_PlayerA, x=2)` → WALL pushed toward Cell 8: `clamp(5 + (+1)*2, 1, 8) = 7`
- [ ] KW-030: `attract_destination(caster_cell=3, target_cell=7, x=6)` → `effective_pull = min(6, |3-7|) = 4`; `result = 7 + sign(3-7)*4 = 7 + (-1)*4 = 3` (co-located with caster; NOT past Cell 3)
- [ ] `repel_destination` with intermediate negative value (e.g., Player A at Cell 1, REPEL 6) → intermediate = `1 + (-1)*6 = -5`; clamped to 1 (must use i32 intermediate — u8 underflow would produce wrong result)
- [ ] `attract_destination(caster_cell=5, target_cell=5, x=3)` → `effective_pull = min(3, 0) = 0`; result = 5 (already co-located; `sign(0)` does not affect output)
- [ ] `repel_destination` output always in [1, 8] regardless of x value or target_cell
- [ ] `attract_destination` output always between target_cell and caster_cell inclusive (target never overshoots caster)
- [ ] Both functions are pure (same inputs always produce same output; no side effects)

---

## Implementation Notes

*Derived from ADR-018 Key Interfaces (movement.rs) and GDD Formulas 1–2:*

**Formula 1 — repel_destination (GDD Formula 1):**
```
repel_destination = clamp(target_cell + (−advance_dir(target.owner)) × X, 1, 8)
```
- `advance_dir(PlayerA) = +1`; `advance_dir(PlayerB) = -1`
- REPEL pushes toward own side: negate advance direction
- **Rust implementation:** compute intermediate in `i32` (or `i16`): `(target_cell as i32 + (-(advance_dir as i32)) * x as i32).clamp(1, 8) as u8`
- **Traversal iteration:** for Trap-trigger purposes (Story 016), iterate cells strictly between start cell and final destination — exclusive of start cell. A clamped-to-current-cell result traverses zero cells.

**Formula 2 — attract_destination (GDD Formula 2):**
```
effective_pull = min(X, |caster_cell − target_cell|)
attract_destination = target_cell + sign(caster_cell − target_cell) × effective_pull
```
- Precondition: caster and target must be in the same lane — caller enforces lane-locality before calling this function
- `sign(0)` edge case: when `caster_cell == target_cell`, `effective_pull = 0`, so multiplication is `0 × anything = 0` regardless of sign convention
- Use `i8::signum()` for the sign computation
- Output always in [1, 8] by construction (target cannot overshoot caster; caster is already in [1, 8])

**Function signatures (ADR-018 Key Interfaces):**
```rust
// server/feature/keyword/movement.rs
pub fn repel_destination(target_cell: u8, owner: PlayerSide, x: u8) -> u8;
pub fn attract_destination(caster_cell: u8, target_cell: u8, x: u8) -> u8;
```
Both are `pub` — called by `apply_repel()` and `apply_attract()` in `effects.rs` (Story 016).

---

## Out of Scope

- Story 016: `apply_repel()` and `apply_attract()` in `effects.rs` — call these formulas and handle Trap traversal, IRREMOVABLE check, and `DisplacementEvent` emission
- CHARGE X bonus movement (KW-028) — movement formula integration in the combat sub-step execution; tested in Story 016

---

## QA Test Cases

*Written at story creation. Implement tests in `tests/unit/keyword/movement_formulas_test.rs`.*

- **AC-1**: KW-029a — Player A unit REPEL 3 from Cell 2
  - Given: `target_cell = 2`, `owner = PlayerA` (advance_dir = +1), `x = 3`
  - When: `repel_destination(2, PlayerA, 3)` called
  - Then: returns `1` (clamped; intermediate = 2 + (-1)*3 = -1 → clamp to 1)
  - Edge cases: intermediate computation must not underflow u8 — use i32

- **AC-2**: KW-029b — WALL unit REPEL 2 (Player B unit pushed by Player A toward Cell 8)
  - Given: `target_cell = 5`, `owner = PlayerB` (advance_dir = -1; negated = +1), `x = 2`
  - When: `repel_destination(5, PlayerB, 2)` called
  - Then: returns `7` (5 + (+1)*2 = 7; no clamping needed)

- **AC-3**: KW-030 — ATTRACT 6 from Cell 3 to Cell 7 target
  - Given: `caster_cell = 3`, `target_cell = 7`, `x = 6`
  - When: `attract_destination(3, 7, 6)` called
  - Then: returns `3` (effective_pull = min(6, 4) = 4; 7 + sign(3-7)*4 = 7 - 4 = 3; co-located, not past caster)
  - Edge cases: result must equal caster_cell (3), not go past to 2 or 1

- **AC-4**: Repel clamp at Cell 1 (extreme underflow test)
  - Given: `target_cell = 1`, `owner = PlayerA`, `x = 6`
  - When: `repel_destination(1, PlayerA, 6)` called
  - Then: returns `1` (intermediate = 1 - 6 = -5; clamped to 1)
  - Edge cases: u8 arithmetic would produce 251 (wrapping) or panic (debug) — i32 intermediate required

- **AC-5**: ATTRACT co-location no-op
  - Given: `caster_cell = 5`, `target_cell = 5`, `x = 3`
  - When: `attract_destination(5, 5, 3)` called
  - Then: returns `5` (effective_pull = 0; no movement)

- **AC-6**: Repel clamp at Cell 8 (max board edge)
  - Given: `target_cell = 8`, `owner = PlayerB`, `x = 4`
  - When: `repel_destination(8, PlayerB, 4)` called
  - Then: returns `8` (intermediate = 8 + (+1)*4 = 12; clamped to 8)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/movement_formulas_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (module scaffold — `movement.rs` file exists)
- Unlocks: Story 016 (displacement keywords use these formulas)
