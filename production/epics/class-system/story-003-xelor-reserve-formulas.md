# Story 003: Xelor Reserve Formulas — Gelure, Xelorium, Rollback

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Class effect formulas are plain Rust functions taking `&mut PlayerSessions` (and other `&mut` state) as parameters — NOT standalone Bevy systems, NOT buffered Messages within a RESOLUTION tick. This preserves RESOLUTION sub-step ordering (Xelorium at sub-step 1, Rollback at sub-step 2) without frame-delay risk. Functions are called synchronously from within the RESOLUTION system body.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `ResMut<PlayerSessions>` in RESOLUTION system — if multiple systems share this resource, ALL must be in an explicit `.before()`/`.after()` ordering chain within the `SystemSet`. Bevy's multi-threaded executor panics in debug builds on shared `ResMut<T>` without ordering.
- `sessions.players` is a `HashMap<PlayerId, PlayerSessionData>`. Access pattern: `sessions.players.get_mut(&player_id).expect("msg")`. The Economy ADR will have added `reserve: u32` and `current_mana: u32` fields to `PlayerSessionData` before these formulas can be implemented.
- ADR-014 is NOT yet in the control manifest. These implementation patterns come from ADR-014 §4 directly.

**Control Manifest Rules (Feature Layer)**:
- Required: All RESOLUTION systems sharing `ResMut<PlayerSessions>` must have explicit `.before()`/`.after()` ordering — ADR-014 §4
- Required: Class effect functions take `&mut PlayerSessions` as plain parameters — never as Bevy system params — ADR-014 §4
- Forbidden: Never use `EventWriter`/`EventReader` — use `MessageWriter::write()` — ADR-009
- Guardrail: RESOLUTION batch budget ≤ 15ms — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, scoped to CS-1 (Gelure), CS-2 (Xelorium), CS-3 (Rollback):*

- [ ] **CS-AC-04** GIVEN Xelor player with `current_mana=5` and `reserve=2`, WHEN Gelure is played, THEN `current_mana=0` and `reserve=7`.
- [ ] **CS-AC-05** GIVEN Xelor player with `current_mana=8, reserve=3` and opponent with `current_mana=6, reserve=8`, WHEN Xelorium (cost=4 mana, deducted first per Economy Rule 4) resolves at RESOLUTION sub-step 1, THEN `Xelor.current_mana=4`, `Xelor.reserve=9`, `opponent.current_mana=0`, `opponent.reserve=8` (unchanged).
- [ ] **CS-AC-05b** GIVEN Xelor player with `current_mana=4` (exactly the cost of Xelorium) and opponent with `current_mana=6`, WHEN Xelorium is played, THEN `Xelor.current_mana=0` (cost deducted), `Xelor.reserve` increases by 6 (the steal receives post-cost `opponent.current_mana=6`), and the play is not rejected as insufficient-mana (exact-cost payment is valid).
- [ ] **CS-AC-06** GIVEN Xelor player with `reserve=4` and three friendly units at cells 2, 3, 5 on a board of cells [1–8] where Player A advances in the +1 direction, WHEN Rollback resolves, THEN `reserve=0` and units land at cells 6, 7, 8 (`clamp(2+4)=6`, `clamp(3+4)=7`, `clamp(5+4)=8`).
- [ ] **CS-AC-07** GIVEN Xelor player with `current_mana=N` (N ≥ Rollback's mana cost per `cards.json`), `reserve=0`, WHEN Rollback is played, THEN `reserve=0`, all friendly units advance 0 cells, and `current_mana = N − Rollback_mana_cost` (mana cost deducted normally; zero-reserve cast is valid and not rejected).
- [ ] **CS-AC-08** GIVEN Xelor player with `reserve=5`, one healthy unit at cell 2 and one STUNned unit at cell 4, WHEN Rollback resolves, THEN the healthy unit moves to cell 7; the STUNned unit does not move; `reserve=0`.

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Formulas CS-1, CS-2, CS-3:*

**File location**: `server/src/core/resolution/effects.rs` (class effect helpers called from `resolve_resolution` system body)

**CS-1 — Gelure**:
```rust
/// reserve_new = reserve + current_mana;  current_mana_new = 0
pub fn apply_gelure(sessions: &mut PlayerSessions, player_id: PlayerId) {
    let p = sessions.players.get_mut(&player_id)
        .expect("apply_gelure: player not in session");
    p.reserve += p.current_mana;
    p.current_mana = 0;
}
```
Edge case: `current_mana = 0` is a no-op transfer (mathematically correct; no special case needed).

**CS-2 — Xelorium**:
```rust
/// self.reserve += opponent.current_mana;  opponent.current_mana = 0
/// Called AFTER Xelorium's own mana cost is deducted (Economy Rule 4).
/// Caster's cost deduction must happen before this call — do not combine.
pub fn apply_xelorium(sessions: &mut PlayerSessions, caster_id: PlayerId) {
    let opponent_id = sessions.opponent_of(caster_id);  // helper returning PlayerId
    let stolen = sessions.players[&opponent_id].current_mana;
    sessions.players.get_mut(&caster_id).expect("caster").reserve += stolen;
    sessions.players.get_mut(&opponent_id).expect("opponent").current_mana = 0;
}
```
`opponent_of()` must be implemented on `PlayerSessions` (derives opponent PlayerId from session map — in 1v1 it is the other key in the HashMap). Calling site in RESOLUTION must deduct Xelorium's 4-mana cost BEFORE calling `apply_xelorium`.

**CS-3 — Rollback**:
```rust
/// n = self.reserve; reserve = 0; each friendly Minion charges n cells (STUN excluded)
pub fn apply_rollback(
    sessions: &mut PlayerSessions,
    board: &mut BoardState,
    player_id: PlayerId,
) {
    let p = sessions.players.get_mut(&player_id).expect("rollback: player not in session");
    let n = p.reserve as i32;
    p.reserve = 0;
    let direction = if player_id == board.player_a_id { 1i32 } else { -1i32 };
    for unit in board.friendly_minions_mut(player_id) {
        if unit.has_status(StatusEffect::Stun) { continue; }  // STUN-blocked units skip
        let new_cell = (unit.cell as i32 + direction * n).clamp(1, 8) as u8;
        unit.cell = new_cell;
    }
}
```
- Only Minion-type units charge; Structures and Traps are excluded by `friendly_minions_mut`.
- `reserve = 0` unconditionally, even if n = 0.
- HASTE units placed this round ARE eligible (HASTE removes summoning sickness; Rollback is movement, not action — GDD CS-AC-08 note, OQ-CS-3 closed).

**Calling site ordering** (in `resolve_resolution`):
- Sub-step 1: `apply_xelorium` (after cost deduction)
- Sub-step 2: `apply_rollback` (after sub-step 1 completes)
- `apply_gelure` fires at the card's placement-commit time, not a specific sub-step number — confirm with RESOLUTION sub-step spec from combat-resolution GDD.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 004: Garde-Temps reserve gate — separate formula, separate story
- Story 005: Miss Nuit per-round trigger — separate formula
- Economy System: `reserve`, `current_mana` field definitions in `PlayerSessionData` — provided by Economy ADR stories; this story calls those fields but does not define them
- RESOLUTION system framework (`resolve_resolution` function skeleton) — provided by combat-resolution epic; this story adds helper functions called from within it

---

## QA Test Cases

*Logic story — automated test specs. Extract formula logic as pure functions taking explicit `reserve: u32`, `current_mana: u32` inputs for unit testability (GDD CS-AC-22/23/24/25 pattern for Ecaflip; same approach applies here). World-level tests for Rollback (needs board state).*

- **AC CS-AC-04 — Gelure basic**:
  - Given: player A in `PlayerSessions` with `current_mana = 5, reserve = 2`
  - When: `apply_gelure(&mut sessions, player_a_id)` called
  - Then: `sessions.players[A].current_mana == 0`; `sessions.players[A].reserve == 7`
  - Edge cases: `current_mana = 0` → reserve unchanged; `reserve = u32::MAX` would overflow — verify saturating or use u64 intermediate if needed (check GDD: no explicit cap, but practical max via mana_cap = 10/12)

- **AC CS-AC-05 — Xelorium standard steal**:
  - Given: Xelor `current_mana = 4` (post-cost: was 8, cost 4 already deducted), `reserve = 3`; opponent `current_mana = 6, reserve = 8`
  - When: `apply_xelorium(&mut sessions, xelor_id)` called
  - Then: Xelor `reserve = 9`; opponent `current_mana = 0`; opponent `reserve = 8` (unchanged)

- **AC CS-AC-05b — Exact cost payment**:
  - Given: Xelor `current_mana = 4` (exactly covers Xelorium's cost); opponent `current_mana = 6`
  - When: cost deduction runs (`current_mana -= 4 = 0`), then `apply_xelorium` called
  - Then: Xelor `reserve += 6` (steals opponent's post-cost current_mana); play not rejected
  - Edge cases: opponent `current_mana = 0` → steal of 0; Xelor still pays own cost

- **AC CS-AC-06 — Rollback movement**:
  - Given: World with `PlayerSessions` (Xelor, reserve=4), 3 friendly Minions at cells 2, 3, 5; Player A (+1 direction); no STUNned units
  - When: `apply_rollback(&mut sessions, &mut board, xelor_id)` called
  - Then: `sessions.players[xelor_id].reserve == 0`; units at cells 6, 7, 8

- **AC CS-AC-07 — Rollback n=0**:
  - Given: Xelor `reserve = 0`; 2 friendly Minions at cells 3, 5
  - When: `apply_rollback` called
  - Then: `reserve = 0` (unchanged); units at cells 3, 5 (no movement)

- **AC CS-AC-08 — Rollback STUN exclusion**:
  - Given: Xelor `reserve = 5`; healthy unit at cell 2; STUNned unit at cell 4
  - When: `apply_rollback` called
  - Then: healthy unit moves to cell 7 (`clamp(2+5)`); STUNned unit stays at cell 4; `reserve = 0`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/xelor_reserve_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (PlayerSessions with `reserve` + `current_mana` fields — added by Economy ADR, but PlayerSessions scaffold is from Story 001) — must be DONE
- Depends on: `economy-system` story-001 (adds `reserve: u32, current_mana: u32` to `PlayerSessionData`) — must be DONE
- Depends on: `combat-resolution` epic story-001 (RESOLUTION system body scaffold — `resolve_resolution` function; this story adds helpers called from it) — must exist before integration
- Unlocks: Story 004 (Garde-Temps — builds on reserve math); Story 005 (Miss Nuit — builds on reserve mutation pattern)
