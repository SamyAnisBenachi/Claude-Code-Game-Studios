# Story 003: Xelor Reserve Formulas — Gelure, Xelorium, Rollback

> **Epic**: Class System
> **Status**: Complete
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

*Derived from ADR-014 Decision §4, ADR-019 §Key Interfaces, and GDD Formulas CS-1, CS-2, CS-3:*

**File location**: `server/src/core/resolution/effects.rs` (class effect helpers called from `resolve_resolution` system body)

**Resource architecture**: Xelor reserve formulas touch **economy state only** — NOT `PlayerSessions`.
Per ADR-014, `PlayerSessions` owns only `class: ClassId` and `class_locked: bool`. Per ADR-019,
`current_mana` and `reserve_mana` live in `PlayerEconomy` inside `ResMut<PlayerEconomies>`.
All mutations go through `server/src/core/economy/api.rs` — direct field assignment outside
that module is forbidden (ADR-019 constraint).

Relevant `api.rs` functions used here:
- `api::add_reserve(economy: &mut PlayerEconomy, amount: u32)` — adds to `reserve_mana`
- `api::discard_current_mana(economy: &mut PlayerEconomy)` — sets `current_mana = 0`
- `api::apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve_only: bool)` — with
  `from_reserve_only = true`, deducts from `reserve_mana` only; call with `cost = economy.reserve_mana`
  to consume all reserve

**CS-1 — Gelure**:
```rust
/// CS-1: reserve_mana += current_mana; current_mana = 0
/// No PlayerSessions access needed — economy fields only (ADR-019).
pub fn apply_gelure(economies: &mut PlayerEconomies, player_id: PlayerId) {
    let Some(economy) = economies.0.get_mut(&player_id) else {
        warn!("apply_gelure: player {:?} not in economies", player_id);
        return;
    };
    let amount = economy.current_mana;  // copy before mutable call
    api::add_reserve(economy, amount);
    api::discard_current_mana(economy);
}
```
Edge case: `current_mana = 0` → `add_reserve(0)` is a no-op; `discard_current_mana` on an
already-zero field is harmless. No special case needed.

**CS-2 — Xelorium**:
```rust
/// CS-2: caster.reserve_mana += opponent.current_mana; opponent.current_mana = 0
/// Called AFTER Xelorium's 4-mana cost is deducted by the calling site (Economy Rule 4).
/// Takes explicit opponent_id — no PlayerSessions dependency (ADR-014 §4 pattern).
/// Reads stolen amount and zeroes opponent in one borrow to avoid double-borrow conflict.
pub fn apply_xelorium(
    economies: &mut PlayerEconomies,
    caster_id: PlayerId,
    opponent_id: PlayerId,
) {
    let stolen = if let Some(e) = economies.0.get_mut(&opponent_id) {
        let m = e.current_mana;
        api::discard_current_mana(e);
        m
    } else {
        warn!("apply_xelorium: opponent {:?} not in economies", opponent_id);
        0
    };
    if let Some(e) = economies.0.get_mut(&caster_id) {
        api::add_reserve(e, stolen);
    }
}
```
Calling site must deduct Xelorium's 4-mana cost via `api::apply_spend(caster_economy, 4, false)`
BEFORE calling `apply_xelorium`. The steal reads opponent's post-cost `current_mana`.
Edge case: opponent `current_mana = 0` → `stolen = 0`; `add_reserve(0)` no-op; caster still pays cost.

**CS-3 — Rollback**:
```rust
/// CS-3: n = reserve_mana; reserve_mana = 0; friendly Minions charge n cells (STUN excluded)
/// Note field name: reserve_mana (ADR-019 PlayerEconomy), not reserve.
pub fn apply_rollback(
    economies: &mut PlayerEconomies,
    board: &mut BoardState,
    player_id: PlayerId,
) {
    // Scoped block releases economy borrow before board access.
    let n = {
        let Some(economy) = economies.0.get_mut(&player_id) else {
            warn!("apply_rollback: player {:?} not in economies", player_id);
            return;
        };
        let reserve = economy.reserve_mana;  // copy before mutable call
        api::apply_spend(economy, reserve, true);  // from_reserve_only=true; cost=reserve zeros it
        reserve as i32
    };

    let direction = if player_id == board.player_a_id { 1i32 } else { -1i32 };
    for unit in board.friendly_minions_mut(player_id) {
        if unit.has_status(StatusEffect::Stun) { continue; }
        let new_cell = (unit.cell as i32 + direction * n).clamp(1, 8) as u8;
        unit.cell = new_cell;
    }
}
```
- `reserve_mana = 0` unconditionally: `api::apply_spend(economy, 0, true)` when n=0 is a no-op.
- No `validate_spend` call needed: consuming exactly what is present cannot fail.
- Only Minion-type units charge; Structures and Traps excluded by `friendly_minions_mut`.
- HASTE units placed this round ARE eligible (GDD CS-AC-08 note, OQ-CS-3 closed).

**Calling site** (in `resolve_resolution`):

Add `ResMut<PlayerEconomies>` to the system params. Derive `opponent_id` from the economies map
before the card-effect dispatch loop (1v1: the other key in the HashMap):

```rust
fn resolve_resolution(
    mut sessions:   ResMut<PlayerSessions>,   // still needed for class identity checks
    mut economies:  ResMut<PlayerEconomies>,  // required for Xelor reserve formulas
    mut board:      ResMut<BoardState>,
    // ... other params
    mut placements: MessageReader<PlacementsCommitted>,
) {
    for placement_batch in placements.read() {
        let caster_id = placement_batch.player_id;
        let opponent_id = economies.0.keys()
            .copied()
            .find(|&p| p != caster_id)
            .expect("resolve_resolution: session has no opponent");

        for card_play in &placement_batch.cards {
            match card_play.effect {
                CardEffect::Gelure => apply_gelure(&mut economies, caster_id),
                CardEffect::Xelorium => {
                    // Deduct cost before steal (Economy Rule 4)
                    if let Some(e) = economies.0.get_mut(&caster_id) {
                        api::apply_spend(e, XELORIUM_MANA_COST, false);
                    }
                    apply_xelorium(&mut economies, caster_id, opponent_id);
                }
                CardEffect::Rollback => apply_rollback(&mut economies, &mut board, caster_id),
                // ...
            }
        }
    }
}
```
- Sub-step 1: `apply_gelure` and `apply_xelorium` (cost deducted inline before steal)
- Sub-step 2: `apply_rollback` (after sub-step 1 completes)
- `apply_gelure` fires at placement-commit time; confirm exact sub-step against combat-resolution GDD.
- `XELORIUM_MANA_COST`: define as `const u32 = 4` in this file; sourced from `cards.json` card data.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 004: Garde-Temps reserve gate — separate formula, separate story
- Story 005: Miss Nuit per-round trigger — separate formula
- Economy System: `reserve_mana`, `current_mana` field definitions in `PlayerEconomy` — owned by `PlayerEconomies` resource (ADR-019); this story calls those fields via `api.rs` but does not define them
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

**Status**: [x] Created and passing

---

## Completion Notes

**Completed**: 2026-05-02
**Criteria**: 6/6 passing.
**Deviations**:
- Advisory: story manifest version `2026-04-30` is older than current control manifest version `2026-05-01`.
- Advisory: story notes still point to `server/src/core/resolution/effects.rs`; implementation lives in `server/src/feature/class/resolution/effects.rs`, matching the current feature-layer organization.
- Advisory: no separate `tests/evidence/class-story-003-tests.md` exists; the story's required Logic evidence is the unit test file itself.
**Test Evidence**: `tests/unit/class/xelor_reserve_test.rs` passed 6/6; `cargo check -p server` passed; `cargo fmt -p server -- --check` passed.
**Code Review**: Skipped - lean review mode.
**Sprint Status**: Not updated - no `CS-003` row exists in `production/sprint-status.yaml`.

---

## Dependencies

- Depends on: Story 001 (PlayerSessions with `reserve` + `current_mana` fields — added by Economy ADR, but PlayerSessions scaffold is from Story 001) — must be DONE
- Depends on: `economy-system` story-001 (adds `reserve: u32, current_mana: u32` to `PlayerSessionData`) — must be DONE
- Depends on: `combat-resolution` epic story-001 (RESOLUTION system body scaffold — `resolve_resolution` function; this story adds helpers called from it) — must exist before integration
- Unlocks: Story 004 (Garde-Temps — builds on reserve math); Story 005 (Miss Nuit — builds on reserve mutation pattern)
