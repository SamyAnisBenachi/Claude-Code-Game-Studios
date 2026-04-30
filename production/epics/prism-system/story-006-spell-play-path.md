# Story 006: Spell Card Play Path Integration

> **Epic**: Prism System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/prism-system.md`
**Requirements**: `TR-PRI-001`, `TR-PRI-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API
**ADR Decision Summary**: The Prism System delivers `prism_strike` (Lane 1/5) and `prism_reserve` (Lane 2/4) spell cards to the player's hand. The Prism System does NOT own play resolution — spell effects are resolved by the Objective System (`take_damage`) for `prism_strike` and the Economy System (`add_reserve`) for `prism_reserve`. This story verifies the full end-to-end pipeline: Prism delivers card → player plays card during DRAFT → effect resolves in the owning system. `prism_strike_damage` and `prism_strike_mana_cost` are read from `Res<GameConfig>` — never hardcoded.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `take_damage(lane, attacker_player, amount)` — Objective System's sole damage interface (ADR-010 / TR-OBJ-010); must be called via the Objective System API, not by direct `ObjectiveHp` mutation
- `add_reserve(player, 1)` — Economy System API (not a formula — it is a direct call on `PlayerEconomies`)
- `prism_reserve` is playable during DRAFT only (GDD Rule 13); play-phase validation is owned by Card Acquisition's spell play validator — Prism System does not gate this
- Self-targeting `prism_strike` (targeting own objective) is legal by design (GDD OQ5 resolved); no server-side rejection for `TargetObj { player_id: self }`
- Config validation: the control manifest (ADR-004) requires `app_exit.write(AppExit::Error(NonZeroU8::MIN))` — **not `panic!`** — for out-of-range config values. PS-22 (ADVISORY) uses this pattern, not `unwrap()` / `panic!()`

**Control Manifest Rules (Feature layer — from ADR-016, ADR-004):**
- Required: `prism_strike_damage` and `prism_strike_mana_cost` read from `Res<GameConfig>` — no hardcoded values in any system body
- Required: `take_damage()` via Objective System interface — not direct `ResMut<ObjectiveHp>` mutation from outside Objective System
- Forbidden: Card Acquisition system in call chain for prism card play validation (Prism System delivers; CA validates play-time; Objective/Economy resolve effect — three distinct systems, none owning another's job)
- Forbidden: `panic!` in production paths — use `app_exit.write(AppExit::Error(NonZeroU8::MIN))` for config validation failures (ADR-004)

---

## Acceptance Criteria

*From GDD `design/gdd/prism-system.md`, scoped to this story:*

- [ ] **PS-04** — GIVEN a player has a `prism_reserve` spell card in hand AND the current phase is DRAFT, WHEN the player plays it, THEN that player's reserve mana increases by exactly 1 AND the player's mana pool balance is unchanged (net 0 deduction — verify by reading mana before and after play).
- [ ] **PS-06** — GIVEN a player has a `prism_strike` spell card in hand, WHEN the player plays it targeting any objective (real or fake, including their own — self-targeting is legal), THEN that objective takes exactly `prism_strike_damage` (default 1) damage, the player's mana pool decreases by exactly `prism_strike_mana_cost` (default 3), and no lane position requirement applies. Test with a target that has no units in its lane to confirm position bypass.
- [ ] **PS-18** — GIVEN `GameConfig` is loaded with `prism_strike_damage = 2` (non-default), WHEN `prism_strike` is played, THEN the objective takes exactly 2 damage — confirming the value is read from config, not hardcoded.
- [ ] **PS-19** — GIVEN `GameConfig` is loaded with `prism_strike_mana_cost = 1` (safe range minimum), WHEN `prism_strike` is played, THEN the player's mana pool decreases by exactly 1 — confirming the cost is read from config and not hardcoded to the default of 3.
- [ ] **PS-22** *(ADVISORY)* — GIVEN `game_config.ron` has `prism_strike_damage` set outside the documented safe range (< 1 or > 3), WHEN the server reads the config at startup, THEN the server exits with `AppExit::Error` and logs an explicit message identifying the invalid field. Note: implementation of the validation gate is shared with the Game Config Pipeline epic (ADR-004); this AC verifies that the Prism-specific validation is wired up. Confirm whether this validation lives in GameConfig's startup system or in PrismPlugin's session-start.

---

## Implementation Notes

*Derived from GDD Rules 13 and the referenced operations table (Section F):*

**prism_reserve play path** (PS-04):
- Card definition in `cards.json`: `{ "id": "prism_reserve", "cost": 0, "target": "Instant", "card_type": "Spell" }`
- Play validation (Card Acquisition): cost = 0 → always affordable; target = Instant → no target selection needed; phase gate: DRAFT only
- Effect resolution (Economy System): `player_economies.add_reserve(player, 1)` — no current_mana deduction (cost is 0)
- Card consumed from hand after successful play

**prism_strike play path** (PS-06):
- Card definition in `cards.json`: `{ "id": "prism_strike", "cost": "{GameConfig.prism_strike_mana_cost}", "target": "TargetObj", "card_type": "Spell" }`
- Play validation (Card Acquisition): cost = `game_config.prism_strike_mana_cost` (read at play time); target = `TargetObj { player_id, lane }` — any lane, any player (no position restriction); phase gate: DRAFT only
- Effect resolution (Objective System): `objective_system.take_damage(lane, attacker_player, game_config.prism_strike_damage)` — bypasses lane position check; saturating_sub on HP
- Mana deducted from current_mana (then reserve_mana if needed per Economy System auto-split rule)

**Self-targeting** (PS-06 edge case): `TargetObj { player_id: attacker }` is valid. Objective System resolves `objective_damage()` without checking if attacker == defender for the damage calculation. Gold reward `+3g` is NOT awarded for self-targeted objective destruction (per TR-OBJ-004: `if attacker_player != defender_player`).

**Config validation** (PS-22): The `prism_strike_damage` field should be validated in `GameConfig`'s startup validation system alongside other dangerous values. If it's ≤ 0 or > 3 (outside documented safe range), call `app_exit.write(AppExit::Error(NonZeroU8::MIN))` with a log message. This is an ADVISORY AC — mark as DEFERRED if the Game Config Pipeline epic handles this validation generically.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Stories 001–005: All Prism System collection logic (`resolve_prism_draws`)
- Objective System epic: `take_damage` formula implementation (this story consumes it; Objective System must be Done first)
- Economy System epic: `add_reserve` implementation (same — must be Done first)
- Card Acquisition epic: spell play validation, mana deduction, hand slot management
- Game Config Pipeline epic: `prism_strike_damage` out-of-range startup abort (if covered generically there)

---

## QA Test Cases

- **PS-04**: prism_reserve play → +1 reserve, no mana change
  - Given: `player_a.reserve_mana = R`; `player_a.current_mana = M`; `player_a.hand` contains `prism_reserve` card; phase = DRAFT
  - When: player plays `prism_reserve` (via card play system)
  - Then: `player_a.reserve_mana == R + 1`; `player_a.current_mana == M` (unchanged); `prism_reserve` removed from hand
  - Edge cases: reserve_mana at 0 before play (0 → 1); reserve_mana already high (uncapped per economy-system.md OQ2 resolution)

- **PS-06**: prism_strike play → damage + mana cost, no lane position required
  - Given: `player_a.current_mana = M`; `player_a.hand` contains `prism_strike`; target objective in lane L has HP = H; no units in lane L
  - When: player plays `prism_strike` targeting `TargetObj { player_id: player_b, lane: L }`
  - Then: objective HP == `max(0, H - prism_strike_damage)` (default prism_strike_damage=1); `player_a.current_mana == M - prism_strike_mana_cost` (default 3); `prism_strike` removed from hand; no lane position check triggered
  - Edge cases: self-targeting (`player_id: player_a`) → damage applies; if own real objective reaches 0 HP, RSM evaluates GAME_OVER; targeting fake objective → fake reward fires (Objective System logic, not tested here)

- **PS-18**: prism_strike_damage from GameConfig (non-default)
  - Given: `GameConfig { prism_strike_damage: 2, .. }` (inserted via `world.insert_resource(cfg)` in test); valid prism_strike in hand; target objective HP = 5
  - When: prism_strike played
  - Then: objective HP == 3 (`5 - 2`); confirms value is not hardcoded to 1
  - Edge cases: `prism_strike_damage = 3` (max safe range); `prism_strike_damage = 1` (default)

- **PS-19**: prism_strike_mana_cost from GameConfig (non-default)
  - Given: `GameConfig { prism_strike_mana_cost: 1, .. }`; `player_a.current_mana = 5`; prism_strike in hand
  - When: prism_strike played
  - Then: `player_a.current_mana == 4` (`5 - 1`); confirms cost not hardcoded to 3
  - Edge cases: mana exactly equal to cost (0 remaining after play); mana less than cost (rejected by Card Acquisition play validator — no damage applied)

- **PS-22** *(ADVISORY)*: Out-of-range config → startup exit
  - Given: `game_config.ron` contains `prism_strike_damage: 5` (> safe range maximum of 3)
  - When: server startup config validation runs
  - Then: `AppExit::Error` sent; explicit log message names `prism_strike_damage` as the invalid field; server does not reach `SessionReady`
  - Note: if this validation is handled by the Game Config Pipeline epic generically, mark this test as DEFERRED here

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/prism/spell_play_path_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (`deterministic-lanes`) must be Done — prism_strike and prism_reserve must be deliverable to hand
- Depends on (external epic — must be Done): Objective System epic stories that implement `take_damage()` via the play-spell path
- Depends on (external epic — must be Done): Economy System epic stories that implement `add_reserve()`
- Depends on (external epic — must be Done): Card Acquisition epic stories that implement spell play validation (mana deduction, target resolution, phase gate for Instant/TargetObj cards)
- Unlocks: None — leaf story; this is the last story in the Prism System epic
