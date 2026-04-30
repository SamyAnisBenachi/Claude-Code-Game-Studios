# Story 004: Garde-Temps Reserve Gate

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Class effects are plain Rust functions called from within the RESOLUTION system body — NOT standalone Bevy systems. Garde-Temps uses `take_damage()` (not `destroy()`) routed through `objective-system.md` Rule 9 for the lethal strike — consistent with the Objective System's damage interface. The per-game cap counter `garde_temps_used_this_game` is owned by Game Session System (GSS), not `PlayerSessions`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `garde_temps_used_this_game: u32` lives in a GSS resource (or is a field added to `PlayerSessionData` by the GSS ADR — confirm with GSS implementation). It is initialized at `on_lobby_to_draft_initial` transition and persists for the full game session.
- Garde-Temps' `mana_cost` field in `cards.json` is 0 (or absent). The Economy System's "from reserve" path (Economy Rule 4) is the only valid payment route. Server validates `reserve >= 20` BEFORE accepting the placement — never deduct optimistically.
- ADR-014 is NOT yet in the control manifest. These patterns come from ADR-014 §4 directly.

**Control Manifest Rules (Feature Layer)**:
- Required: Class effect functions take `&mut PlayerSessions` as plain parameters — ADR-014 §4
- Required: Phase-gate pattern: validate conditions before any state mutation — ADR-002
- Forbidden: Never deduct reserve/mana before validating all gate conditions — ADR-002
- Guardrail: RESOLUTION batch budget ≤ 15ms — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, CS-4 formula:*

- [ ] **CS-AC-09** GIVEN Xelor player with `reserve=15` (below `garde_temps_reserve_cost=20`), WHEN Garde-Temps play is submitted, THEN server rejects with insufficient-reserve error; no mana deducted; `reserve=15` unchanged.
- [ ] **CS-AC-10** GIVEN Xelor player with `reserve=22`, WHEN Garde-Temps is accepted, THEN `reserve=2` and the chosen enemy objective HP=0.
- [ ] **CS-AC-10b** GIVEN Xelor player with `reserve=22` and `garde_temps_used_this_game=1` (`garde_temps_per_game_cap=1`), WHEN Garde-Temps play is submitted, THEN server rejects with "per-game cap reached" error; `reserve=22` unchanged; `garde_temps_used_this_game=1` unchanged.

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Formula CS-4:*

**CS-4 formula**:
```
target_valid = (chosen_enemy_objective.is_alive = true)
playable = (self.reserve >= garde_temps_cost)
        AND target_valid
        AND (self.garde_temps_used_this_game < garde_temps_per_game_cap)
if playable:
  self.reserve_new = self.reserve - garde_temps_cost
  self.garde_temps_used_this_game += 1
  take_damage(lane=chosen_enemy_objective.lane, attacker=self, amount=chosen_enemy_objective.hp)
else:
  reject_play  -- reserve/mana untouched; reject if reserve < cost, target dead, or cap reached
```

**File location**: `server/src/core/resolution/effects.rs`

```rust
pub fn apply_garde_temps(
    sessions: &mut PlayerSessions,
    gss_state: &mut GssSessionState,  // owns garde_temps_used_this_game
    objectives: &mut ObjectiveState,
    config: &GameConfig,
    player_id: PlayerId,
    target_lane: u8,
) -> Result<(), GardeTempsError> {
    let p = sessions.players.get_mut(&player_id)
        .expect("apply_garde_temps: player not in session");
    let used = gss_state.garde_temps_used_per_player[&player_id];
    let target_alive = objectives.is_alive(player_id.opponent(), target_lane);

    if p.reserve < config.garde_temps_reserve_cost {
        return Err(GardeTempsError::InsufficientReserve);
    }
    if !target_alive {
        return Err(GardeTempsError::TargetAlreadyDestroyed);
    }
    if used >= config.garde_temps_per_game_cap {
        return Err(GardeTempsError::PerGameCapReached);
    }

    // All conditions pass — mutate state
    p.reserve -= config.garde_temps_reserve_cost;
    gss_state.garde_temps_used_per_player
        .entry(player_id).and_modify(|c| *c += 1);
    let lethal_hp = objectives.hp(player_id.opponent(), target_lane);
    objectives.take_damage(player_id.opponent(), target_lane, player_id, lethal_hp);
    Ok(())
}
```

**Critical ordering rules from GDD**:
- Validation precedes all deductions. Server MUST validate `reserve >= garde_temps_cost` BEFORE accepting the placement. No optimistic deduction.
- Uses `take_damage()` NOT `destroy()` — consistent with objective-system.md Rule 9 and Edge Cases ("If Garde-Temps targets an objective: Treated as `take_damage(lane, attacker_player, objective_hp)`").
- `garde_temps_used_this_game` counter owned by GSS, initialized at LOBBY→DRAFT_INITIAL entry, persists for full game. Counter is never reset mid-game. Implementer must confirm exact field name with GSS ADR before writing integration test.

**Rejection case**: `reserve` NOT deducted on reject. Client UI should grey out already-destroyed lanes as invalid targets (client-side pre-validation; server is authoritative gate).

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 003: Gelure/Xelorium/Rollback formulas — separate reserve formulas
- Story 001: `PlayerSessions` scaffold and `reserve` field — provided by Economy ADR stories
- Objective System: `take_damage()` implementation — owned by `objective-system` epic
- GSS: `garde_temps_used_this_game` counter initialization and storage — owned by `game-session-system` epic
- UI: Garde-Temps gate visual feedback (reserve vs. mana distinction, exhausted-state display) — Presentation layer

---

## QA Test Cases

*Logic story — automated test specs using `World::new()`.*

- **AC CS-AC-09 — Insufficient reserve rejection**:
  - Given: `PlayerSessions` player with `reserve = 15`, `garde_temps_reserve_cost = 20` (from GameConfig); target objective alive; `garde_temps_used = 0`
  - When: `apply_garde_temps(...)` called with target_lane = 2
  - Then: returns `Err(GardeTempsError::InsufficientReserve)`; `reserve` still `15`; objective HP unchanged
  - Edge cases: `reserve = 19` → still rejected; `reserve = 20` → accepted

- **AC CS-AC-10 — Accepted play**:
  - Given: `reserve = 22`, target objective HP = 5 (alive), `garde_temps_used = 0`, `garde_temps_per_game_cap = 1`
  - When: `apply_garde_temps(...)` called
  - Then: returns `Ok(())`; `reserve = 2` (22 − 20); target objective HP = 0 (via take_damage with lethal amount); `garde_temps_used = 1`
  - Edge cases: `reserve = 20` exactly → accepted, `reserve_new = 0`

- **AC CS-AC-10b — Per-game cap reached rejection**:
  - Given: `reserve = 22`, target objective alive, `garde_temps_used = 1`, `garde_temps_per_game_cap = 1`
  - When: `apply_garde_temps(...)` called
  - Then: returns `Err(GardeTempsError::PerGameCapReached)`; `reserve = 22` unchanged; `garde_temps_used = 1` unchanged; objective HP unchanged
  - Edge cases: `garde_temps_per_game_cap = 2` → second play accepted; `garde_temps_used = 2` would be rejected

- **Already-destroyed target rejection**:
  - Given: `reserve = 25`, `garde_temps_used = 0`, target objective `is_alive = false`
  - When: `apply_garde_temps(...)` called
  - Then: returns `Err(GardeTempsError::TargetAlreadyDestroyed)`; `reserve = 25` unchanged

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/garde_temps_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (reserve math established in PlayerSessions; `apply_gelure`/`apply_xelorium` patterns) — must be DONE
- Depends on: `game-session-system` epic (provides `garde_temps_used_this_game` counter initialized at LOBBY→DRAFT_INITIAL) — must be DONE
- Depends on: `objective-system` epic (provides `take_damage()` API and `ObjectiveState.is_alive()`) — must be DONE for integration; unit test can stub `ObjectiveState`
- Unlocks: Story 006 (Sacrier effects also interact with Objective System — validates the take_damage interface is wired)
