# Story 006: SILENCE + INJURED State System

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-005 (INJURED re-evaluated at sub-step boundaries, not retroactive), TR-KW-006 (SILENCE strips INJURED-granted keywords; silenced_until_round Option<u32>)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (ECS State Architecture) + ADR-022 (eval_injured_bonuses inline dispatch)
**ADR Decision Summary**: `silenced_until_round: Option<u32>` in `UnitKeywordState` (NOT `Option<u8>` — u32 matches round_number type per NP R6). INJURED is a derived state (`current_hp < max_hp`) — NOT a stored field; computed at each sub-step boundary by `eval_injured_bonuses()` which is called inline by combat resolution. SILENCE strips all keyword-granted effects including INJURED bonuses.

**BLOCKED**: ADR-018 Proposed. Also BLOCKED on OQ-KS-new: `silence_duration_rounds: u8` structured field must be added to `cards.json` schema in card-data-pool.md before SILENCE can be implemented. Story 001 and Story 014 must be Done (Story 014 owns `eval_injured_bonuses`; this story wires SILENCE-stripping into it).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `max_hp` lives only in `UnitBoardState` snapshot (reconnect recovery), NOT in the `UnitStats` replicated component. Client caches `UnitBoardState.max_hp` from snapshot; server derives INJURED from live `UnitStats.hp` vs. a cached `max_hp` field that must exist on the server-side entity component.

**Control Manifest Rules (Feature layer)**:
- Required: INJURED is a derived state, never stored as a boolean flag — always compare `current_hp < max_hp` (ADR-018)
- Forbidden: Never store `max_hp` only in `UnitStats` replicated component — `max_hp` must be accessible server-side for derivation (coordinate with combat-resolution.md UnitStats component design)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-007: GIVEN unit X has max_HP=4, current_HP=4, and gains FIRST STRIKE when INJURED; X receives 2 damage in SS3 (reducing HP to 2), WHEN SS3 resolves, THEN X does NOT receive the INJURED-granted FIRST STRIKE during SS3; INJURED activates at the SS3→SS4 boundary, granting the bonus from SS4 onward
- [ ] KW-008a: GIVEN a unit is INJURED and gains FIRST STRIKE from INJURED, WHEN SILENCE is applied, THEN the INJURED-granted FIRST STRIKE is stripped; the unit no longer attacks in SS3
- [ ] KW-008b: GIVEN a unit is INJURED and then SILENCEd, WHEN SILENCE applies, THEN the `injured` state (computed from HP) remains true — SILENCE does not clear the INJURED state
- [ ] KW-023: GIVEN a SILENCEd unit has COUNTERATTACK, DEATH trigger, FIRST STRIKE, and is INJURED, WHEN SILENCE applies, THEN all keyword hooks are stripped; INJURED state persists (HP comparison still true)
- [ ] KW-036: GIVEN a WALL unit is SILENCEd, WHEN SS5 resolves, THEN the SILENCEd WALL loses its blocking behavior; advancing enemies no longer halt at its cell; unit still has MP=0 (card stat) and does not self-move
- [ ] KW-044: GIVEN unit X has FIRST STRIKE and COUNTERATTACK and is SILENCEd for exactly 1 RESOLUTION during round R, WHEN round R+1 RESOLUTION begins, THEN FIRST STRIKE and COUNTERATTACK are active again (SILENCE has expired; `silenced_until_round` = R, current_round = R+1)
- [ ] KW-045: GIVEN unit X with FIRST STRIKE and COUNTERATTACK is SILENCEd at the end of SS3 (after SS3 has resolved), WHEN SS6 resolves, THEN unit X does NOT use FIRST STRIKE in SS6 AND does NOT fire COUNTERATTACK; SS3 damage already dealt is NOT reversed

---

## Implementation Notes

*Derived from ADR-018 Part 1, GDD Replication Contract, and GDD Edge Cases:*

**silenced_until_round field:**
- `silenced_until_round: Option<u32>` — `Some(R)` means silenced until end of round R inclusive; client renders SILENCE outline while `current_round <= silenced_until_round`
- Set when SILENCE card effect fires: `silenced_until_round = Some(current_round + silence_duration_rounds - 1)` where `silence_duration_rounds` comes from the card's structured field (OQ-KS-new — must be added to cards.json schema)
- Check at each sub-step: `if let Some(r) = kw_state.silenced_until_round { if current_round <= r { /* silenced */ } }`
- Cleared naturally — no explicit clear needed; the Option check handles expiry

**SILENCE strips all keywords** — not just some:
- FIRST STRIKE, HASTE, CHARGE X, RANGE, WALL blocking, BODYGUARD protection, UNTARGETABLE, RESISTANCE, VULNERABILITY, ARMOR-PIERCING, SHIELD, LEADER bonus, OUTNUMBERED, ALL trigger hooks
- INJURED STATE cannot be silenced — it is derived from HP, not a keyword

**INJURED derivation:**
- `injured(unit) = unit.current_hp < unit.max_hp`
- `max_hp` must be cached server-side (separate from replicated `UnitStats.hp`) — coordinate with combat-resolution.md unit component design to ensure `max_hp` is accessible
- Evaluated by `eval_injured_bonuses()` at sub-step boundaries — not inline during SS3 damage computation (KW-007: bonus is NOT active during the sub-step the damage was received)

**SILENCE on WALL (KW-036):**
- SILENCEd WALL loses WALL keyword behavior (blocking anchor), but retains MP=0 card stat (unit physically cannot self-move)
- Enemies no longer halt at SILENCEd WALL's cell — combat resolution must check `silenced_until_round` before applying WALL collision logic

---

## Out of Scope

- Story 014: `eval_injured_bonuses()` full implementation; INJURED-granted SHIELD timing (KW-057)
- Story 007: WALL blocking behavior (SILENCE+WALL overlap tested here via KW-036)
- Story 008: LEADER+SILENCE interaction (KW-026 — LEADER snapshot; KW-039 — mid-RESOLUTION SILENCE)

---

## QA Test Cases

- **AC-1**: KW-007 — INJURED bonus not active during damage sub-step
  - Given: unit X (max_HP=4, current_HP=4) has FIRST STRIKE when INJURED; no other FIRST STRIKE
  - When: X receives 2 damage in SS3 (HP→2); SS3 resolves completely
  - Then: eval_injured_bonuses() at SS3→SS4 boundary sets INJURED bonus active; X does NOT attack in SS3 (was not INJURED at SS3 start); X attacks in SS6 via INJURED-granted FIRST STRIKE would be in Story 014
  - Edge cases: verify that `current_hp < max_hp` is false at SS3 start (before damage)

- **AC-2**: KW-008a — SILENCE strips INJURED-granted keywords
  - Given: unit X (INJURED, INJURED-granted FIRST STRIKE active)
  - When: SILENCE is applied to X
  - Then: INJURED-granted FIRST STRIKE is stripped; X does not attack in SS3; `silenced_until_round = Some(current_round)`
  - Edge cases: INJURED state (HP comparison) remains true after SILENCE

- **AC-3**: KW-008b — SILENCE does not clear INJURED state
  - Given: unit X (current_hp=2, max_hp=4, INJURED=true)
  - When: SILENCE applied
  - Then: `silenced_until_round` set; `current_hp` and `max_hp` unchanged; `injured(X) = 2 < 4 = true`
  - Edge cases: INJURED is a computed state — no field to clear

- **AC-4**: KW-023 — SILENCE strips all keyword hooks simultaneously
  - Given: SILENCEd unit has COUNTERATTACK, DEATH trigger, FIRST STRIKE, is INJURED
  - When: SILENCE applies (silenced_until_round set)
  - Then: COUNTERATTACK doesn't fire; DEATH doesn't fire when unit dies; FIRST STRIKE doesn't fire in SS3; INJURED state (HP comparison) still true
  - Edge cases: all four keyword hooks suppressed in one SILENCE application

- **AC-5**: KW-044 — SILENCE expires between rounds
  - Given: unit SILENCEd for 1 RESOLUTION in round R (`silenced_until_round = Some(R)`)
  - When: round R+1 begins (`current_round = R+1`)
  - Then: `R+1 > R` so SILENCE check fails → FIRST STRIKE and COUNTERATTACK active again
  - Edge cases: SILENCE with `silenced_until_round = Some(R)` must not persist into R+1

- **AC-6**: KW-045 — SILENCE applied mid-RESOLUTION suppresses remaining sub-steps
  - Given: unit X with FIRST STRIKE and COUNTERATTACK; SILENCE applied at end of SS3 (after SS3 completes)
  - When: SS6 resolves
  - Then: X does not attack in SS6 (SILENCE active); X does not fire COUNTERATTACK; SS3 damage already dealt is NOT reversed
  - Edge cases: SILENCE application is not retroactive — only affects actions from application point forward

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/silence_injured_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 007 (WALL mechanics — for KW-036 SILENCE+WALL test)
- Depends on: OQ-KS-new resolved (`silence_duration_rounds: u8` in cards.json schema)
- Unlocks: Story 008 (LEADER+SILENCE), Story 013 (COUNTERATTACK+SILENCE), Story 014 (eval_injured_bonuses)
