# Story 004: STUN State Management

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-007 — STUN suppresses all actions in SS2/SS3/SS5/SS6; `UnitKeywordState.stun_active` flag; one-RESOLUTION scope
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture)
**ADR Decision Summary**: `stun_active: bool` is stored in `UnitKeywordState`. Set immediately when STUN is applied (via an APPEARANCE trigger or card effect). Cleared at RESOLUTION end. The wire protocol carries `KeywordPayload::StunApplied { duration_rounds: u8 }` for forward-compatibility; server MUST always emit `duration_rounds = 1` — any value > 1 is a server bug in the current design.

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `stun_active` cleared at RESOLUTION end — ensure the clear runs in the same exclusive system context as `resolve_combat` or in a dedicated PostUpdate system that runs after RESOLUTION completes. Do not use a timed or frame-deferred clear.

**Control Manifest Rules (Feature layer)**:
- Required: `stun_active` clear at RESOLUTION end must be structural (explicit system call), not time-based (ADR-009)
- Forbidden: Never design multi-round STUN — `duration_rounds` field exists for forward-compatibility only; server must always emit 1

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria, Combat Keywords section:*

- [ ] KW-015a: GIVEN a STUNned unit is in the path of an enemy attack in SS3 or SS6, WHEN the attack resolves, THEN the STUNned unit takes incoming damage according to the normal damage formula; it does not attack or advance
- [ ] KW-015b: GIVEN a unit was STUNned during RESOLUTION R, WHEN RESOLUTION R+1 begins, THEN the STUN state is cleared; the unit participates in SS2, SS3, SS5, and SS6 normally
- [ ] KW-034: GIVEN a HASTE unit has STUN applied in SS1, WHEN RESOLUTION proceeds, THEN the STUNned HASTE unit skips SS2, SS3, SS5, and SS6; HASTE does not partially override STUN

---

## Implementation Notes

*Derived from ADR-018 Part 1 and GDD Detailed Design — STUN:*

**STUN suppression scope:**
- SS2 (CHARGE X movement) — STUN suppresses
- SS3 (FIRST STRIKE attacks) — STUN suppresses
- SS5 (standard movement) — STUN suppresses
- SS6 (standard attacks) — STUN suppresses
- Unit remains on board and takes damage normally from enemy attacks
- STUN applied in SS1 overrides HASTE (KW-034 — canonical HASTE+STUN test)

**Wire protocol note (ADR-018 Part 7):**
`KeywordPayload::StunApplied { duration_rounds: u8 }` — server MUST always emit `duration_rounds = 1`. The `u8` field exists for forward-compatibility only. Any emission of `duration_rounds > 1` is a server bug. Multi-round STUN is not designed.

**State lifecycle:**
1. STUN applied → `kw_state.stun_active = true` immediately
2. Emit `KeywordTriggered { payload: StunApplied { duration_rounds: 1 }, sub_step }`
3. Combat resolution checks `kw_state.stun_active` at each sub-step gate and skips actions if true
4. RESOLUTION end → `kw_state.stun_active = false` for all units

**Combat resolution integration pattern:**
```rust
// In execute_ss2(), execute_ss3(), execute_ss5(), execute_ss6():
let kw_state = world.get::<UnitKeywordState>(unit).expect("...");
if kw_state.stun_active { continue; } // skip all SS actions
// ...proceed with normal logic...
```

---

## Out of Scope

- Story 003: HASTE keyword mechanics (STUN+HASTE interaction covered here as KW-034 cross-test)
- Story 006: SILENCE keyword (separate state management, different scope)
- STUN application source (APPEARANCE trigger, card effect) — handled in Stories 012–016

---

## QA Test Cases

- **AC-1**: KW-015a — STUNned unit takes damage but cannot act
  - Given: STUNned unit A (ATK=3, HP=5) in SS6; enemy unit B (ATK=2) attacks A
  - When: SS6 resolves
  - Then: unit A HP = 5 - 2 = 3 (takes damage normally); unit A does NOT attack B (stun_active suppresses SS6 attacks); unit A does NOT advance in SS5
  - Edge cases: STUN does not provide immunity to damage from any source

- **AC-2**: KW-015b — STUN clears at RESOLUTION end
  - Given: unit STUNned during RESOLUTION R (stun_active = true at RESOLUTION R end)
  - When: RESOLUTION R+1 begins (leader_snapshot_system + SS1 start)
  - Then: `stun_active = false`; unit participates in SS2, SS3 (if FIRST STRIKE), SS5, SS6 normally in round R+1
  - Edge cases: STUN clear must happen BEFORE SS1 of round R+1 begins (not after SS1)

- **AC-3**: KW-034 — STUN overrides HASTE (canonical cross-keyword test)
  - Given: HASTE unit placed in SS1; STUN applied in SS1 via APPEARANCE trigger
  - When: RESOLUTION proceeds
  - Then: `stun_active = true`; unit skips SS2, SS3, SS5, SS6 entirely; HASTE does not grant any partial participation
  - Edge cases: HASTE without STUN correctly grants SS2/SS5/SS6 participation (tested in Story 003 KW-013)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/stun_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold)
- Depends on: Story 003 (HASTE keyword, for KW-034 cross-test)
- Unlocks: No direct story unlocks (STUN application is used by Story 016 displacement keywords for STUN-on-trap-traversal)
