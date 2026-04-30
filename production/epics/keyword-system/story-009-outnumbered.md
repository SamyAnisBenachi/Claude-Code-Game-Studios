# Story 009: OUTNUMBERED Board Count Evaluation

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-010 — OUTNUMBERED: per-player global board count, strict less-than comparison; re-evaluated per sub-step
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, Part 6 OUTNUMBERED Evaluation)
**ADR Decision Summary**: `eval_outnumbered_system` called by combat resolution at each sub-step boundary. Formula 3: `outnumbered(player) = count(alive_units(player)) < count(alive_units(opponent))`. Traps excluded from count. `OutnumberedFlipped` emitted ONLY when the boolean transitions (bandwidth-efficient). `outnumbered_active: bool` cached in `UnitKeywordState` per unit.

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `eval_outnumbered_system` is called inline by `resolve_combat` (exclusive system) at each sub-step boundary. Uses `Res<BoardUnitCounts>` resource for efficient O(1) count lookup — not a per-frame full-board scan.

**Control Manifest Rules (Feature layer)**:
- Required: `eval_outnumbered_system` called at sub-step boundaries, not per-attack (ADR-018)
- Forbidden: Never compute OUTNUMBERED mid-sub-step (e.g., mid-DEATH chain in SS4) — evaluated at sub-step boundaries only (KW-040)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-027a: GIVEN Player A has 3 units and Player B has 3 units at SS2 entry, WHEN OUTNUMBERED is evaluated for Player A, THEN result is false (equal counts — strict less-than, not ≤)
- [ ] KW-027b: GIVEN Player A has 2 units and Player B has 4 units at SS2 entry, WHEN OUTNUMBERED is evaluated for Player A, THEN result is true
- [ ] KW-040: GIVEN a DEATH trigger chain changes board counts mid-chain in SS4, WHEN OUTNUMBERED is evaluated for SS5, THEN the count reflects the full board state AFTER all SS4 deaths resolve — NOT any intermediate count during the chain
- [ ] KW-054: GIVEN Player A has 2 units and Player B has 4 at SS3 entry (Player A OUTNUMBERED); Player A's FIRST STRIKE kills 3 opponent units in SS3 (opponent count → 1), WHEN SS5 is evaluated, THEN OUTNUMBERED for Player A is false (2 > 1 — no longer outnumbered)
- [ ] Board count definition: Minions + Structures (alive); Traps excluded (face-down); Fields excluded (no HP, not fighting). Confirmed max = 10 per player (5 Minions + 5 Structures)
- [ ] `OutnumberedFlipped` event emitted ONLY when boolean transitions (true→false or false→true) — NOT emitted when value is unchanged
- [ ] `outnumbered_active: bool` in `UnitKeywordState` updated on each unit with OUTNUMBERED keyword at each sub-step boundary

---

## Implementation Notes

*Derived from ADR-018 Part 6 and GDD Formula 3:*

**eval_outnumbered_system signature (ADR-018 Part 6):**
```rust
pub fn eval_outnumbered_system(
    mut units: Query<(&UnitBoardOwner, &mut UnitKeywordState)>,
    board_counts: Res<BoardUnitCounts>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
)
```

**Formula 3 (GDD):**
```
outnumbered(player) = count(alive_units(player)) < count(alive_units(opponent))
```
- `alive_units` = Minions + Structures; excludes Traps and Fields
- Strict less-than: equal counts → false

**BoardUnitCounts resource:** must exist (owned by board-lane-system epic — coordinate with that epic to ensure counts are maintained per-player and accessible as a resource). `eval_outnumbered_system` reads counts from this resource rather than scanning the board each call.

**OutnumberedFlipped emission pattern (bandwidth-efficient):**
```rust
let was_outnumbered = kw_state.outnumbered_active;
let now_outnumbered = board_counts.count(player) < board_counts.count(opponent);
kw_state.outnumbered_active = now_outnumbered;
if was_outnumbered != now_outnumbered {
    keyword_triggered.write(KeywordTriggered {
        payload: OutnumberedFlipped { player_id, active: now_outnumbered },
        sub_step,
        source_unit_id: None, // board-global event
    });
}
```

**KW-040 — mid-DEATH chain boundary:** `eval_outnumbered_system` is called once at the START of each sub-step (not inside the SS4 DEATH chain drain loop). The count read at SS5 entry reflects ALL SS4 deaths resolved.

**OUTNUMBERED indicator note (GDD):** the visual indicator must be per-unit (on each unit carrying the OUTNUMBERED keyword), not per-lane. This is a client-side rendering concern, not a server concern.

---

## Out of Scope

- Story 009 does NOT implement the board unit count tracking itself — that belongs to board-lane-system epic
- Story 054 cross-test (OUTNUMBERED flip after FIRST STRIKE in SS3) — also requires Story 003 (FIRST STRIKE) to be Done

---

## QA Test Cases

- **AC-1**: KW-027a — Equal counts → not outnumbered
  - Given: Player A = 3 units; Player B = 3 units at SS2 entry
  - When: eval_outnumbered_system called
  - Then: `outnumbered(PlayerA) = 3 < 3 = false`; `outnumbered_active` set to false; no OutnumberedFlipped event emitted (if was already false)
  - Edge cases: strict less-than — 3 < 3 is false; 3 ≤ 3 would be true but that would be wrong

- **AC-2**: KW-027b — Fewer units → outnumbered
  - Given: Player A = 2 units; Player B = 4 units at SS2 entry
  - When: eval_outnumbered_system called
  - Then: `outnumbered(PlayerA) = 2 < 4 = true`; `outnumbered_active` set to true on all Player A units with OUTNUMBERED keyword; OutnumberedFlipped { player_id: A, active: true } emitted
  - Edge cases: only units with OUTNUMBERED keyword have `outnumbered_active` updated

- **AC-3**: KW-040 — OUTNUMBERED evaluated AFTER full SS4 chain resolves
  - Given: SS4 DEATH chain kills 3 Player B units; Player B count drops from 5 to 2 during chain; Player A has 3 units
  - When: eval_outnumbered_system called at SS5 entry (after SS4 fully resolves)
  - Then: `outnumbered(PlayerA) = 3 < 2 = false`; OUTNUMBERED inactive for Player A
  - Edge cases: do NOT call eval_outnumbered mid-chain (intermediate count of 4 or 3 must not affect SS5 evaluation)

- **AC-4**: KW-054 — OUTNUMBERED flips after FIRST STRIKE kills in SS3
  - Given: Player A = 2 units (OUTNUMBERED at SS3 entry, opponent has 4); Player A FIRST STRIKE kills 3 opponent units in SS3 (opponent → 1)
  - When: eval_outnumbered_system called at SS5 entry
  - Then: `outnumbered(PlayerA) = 2 < 1 = false`; OutnumberedFlipped { player_id: A, active: false } emitted
  - Edge cases: SS3 kills are reflected in BoardUnitCounts BEFORE SS5 evaluation

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/outnumbered_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 003 (FIRST STRIKE — for KW-054 cross-test)
- Depends on: board-lane-system epic (BoardUnitCounts resource must exist)
- Unlocks: None directly
