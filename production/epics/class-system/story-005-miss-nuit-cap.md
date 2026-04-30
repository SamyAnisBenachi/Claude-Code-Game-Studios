# Story 005: Miss Nuit Per-Round Cap

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Class effects are plain Rust functions called from within the RESOLUTION system body — NOT standalone Bevy systems. Miss Nuit's trigger fires during PLACEMENT commit (sub-step 1) when an opponent card is played from hand. The `miss_nuit_reserve_gained_this_round: u32` tracker is owned by Economy System (not PlayerSessions), reset to 0 at each DRAFT phase entry.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `miss_nuit_reserve_gained_this_round` lives in Economy System's resource (confirm exact field name with Economy ADR before implementing).
- Trigger fires on `opponent_card_played_event` during PLACEMENT commit sub-step 1 — this is an intra-RESOLUTION event, not a Bevy `Message`. Confirm the exact hook point with the combat-resolution GDD sub-step spec.
- ADR-014 is NOT yet in the control manifest.

**Control Manifest Rules (Feature Layer)**:
- Required: Class effect functions take plain parameters — NOT Bevy system params — ADR-014 §4
- Required: Feature systems react to RSM Messages; never observe RoundState directly — ADR-010
- Guardrail: Server tick budget ≤ 5ms — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, CS-11 formula:*

- [ ] **CS-AC-11** GIVEN Miss Nuit is in play, WHEN opponent plays 3 cards (spell or minion) in one round, THEN `Xelor.reserve` increases by exactly 2 (`miss_nuit_cap=2` enforced, not 3).
- [ ] **CS-AC-12** GIVEN Miss Nuit is in play, WHEN opponent spawns 3 token units via DEATH triggers in one round (not card-plays from hand), THEN `Xelor.reserve` is unchanged (token spawns do not qualify as card plays).

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Formula CS-11:*

**CS-11 formula**:
```rust
// Fires on each opponent_card_played_event during PLACEMENT commit (sub-step 1)
if miss_nuit.is_alive AND NOT miss_nuit.is_silenced
        AND miss_nuit_reserve_gained_this_round < miss_nuit_cap:
    self.reserve += 1
    miss_nuit_reserve_gained_this_round += 1
```

**File location**: `server/src/core/resolution/effects.rs`

```rust
/// Called once per qualifying opponent card play from hand.
/// Caller is responsible for filtering: only Spell and Minion cards played from hand qualify.
/// Token spawns, prism-grants, and Drheller-draws must NOT call this function.
pub fn try_apply_miss_nuit_trigger(
    sessions: &mut PlayerSessions,
    economy: &mut EconomyState,     // owns miss_nuit_reserve_gained_this_round
    config: &GameConfig,
    xelor_player_id: PlayerId,
    miss_nuit_entity: Entity,
    board: &BoardState,
) {
    if !board.is_alive(miss_nuit_entity) { return; }
    if board.has_status(miss_nuit_entity, StatusEffect::Silence) { return; }
    if economy.miss_nuit_reserve_gained_this_round(xelor_player_id) >= config.miss_nuit_cap {
        return;
    }
    let p = sessions.players.get_mut(&xelor_player_id).expect("miss nuit: player not in session");
    p.reserve += 1;
    economy.increment_miss_nuit_counter(xelor_player_id);
}
```

**What counts as a qualifying card play (binding)**:
- **Counts**: Spell cast or Minion summon committed at PLACEMENT sub-step 1 (card was in opponent's hand and is now leaving it).
- **Does NOT count**: Token spawns (Mummy, Madoll, Bow Meow, etc.); free card grants from Lane 3 prism; Drheller-style triggered draws.
- **Does NOT count**: Xelor's own card plays — only **opponent** plays trigger Miss Nuit.

**Edge case — SILENCE or destruction mid-round**: Subsequent opponent plays after Miss Nuit is silenced or destroyed do not trigger. Trigger is gated on `is_alive AND NOT is_silenced` at the moment the opponent's card commits. Reserve already awarded before SILENCE lands is NOT retroactively revoked.

**Reset**: `miss_nuit_reserve_gained_this_round` resets to 0 at DRAFT phase entry (owned by Economy System).

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 003: Gelure/Xelorium/Rollback reserve mutations — separate reserve-gaining paths
- Economy System: `miss_nuit_reserve_gained_this_round` field definition and DRAFT-entry reset — Economy ADR stories
- Combat Resolution: SILENCE status effect application — Keyword System + Combat Resolution epics
- UI: Xelorium drain animation, reserve counter updates mid-RESOLUTION — Presentation layer

---

## QA Test Cases

*Logic story — automated test specs using `World::new()`.*

- **AC CS-AC-11 — Cap enforcement (3 plays, cap=2)**:
  - Given: `PlayerSessions` Xelor player `reserve = 0`; Miss Nuit alive and unsilenced in board; `miss_nuit_cap = 2`; `miss_nuit_reserve_gained_this_round = 0`
  - When: `try_apply_miss_nuit_trigger(...)` called 3 times (simulating 3 opponent card plays)
  - Then: `reserve = 2` (capped); `miss_nuit_reserve_gained_this_round = 2` (not 3)
  - Edge cases: cap=1 → only first play grants reserve; fourth call with existing reserve=2, gained=2 → no-op

- **AC CS-AC-12 — Token spawn exclusion**:
  - Given: same setup; opponent triggers 3 DEATH effects spawning Mummy tokens
  - When: token spawn path does NOT call `try_apply_miss_nuit_trigger` (caller is responsible for filtering)
  - Then: `reserve = 0` unchanged; `miss_nuit_reserve_gained_this_round = 0` unchanged
  - Verify: confirm that the token spawn system (Story 002 / board/spawn.rs) does NOT invoke `try_apply_miss_nuit_trigger`

- **SILENCE mid-round**:
  - Given: Xelor `reserve = 0`; Miss Nuit alive; opponent plays 1 card (reserve → 1); opponent SILENCEs Miss Nuit; opponent plays 1 more card
  - When: `try_apply_miss_nuit_trigger` called after SILENCE
  - Then: `reserve = 1` (only first play counted; second play after SILENCE does not trigger)

- **Miss Nuit destroyed mid-round**:
  - Given: Miss Nuit alive for first opponent play (reserve → 1); Miss Nuit HP → 0 (destroyed); opponent plays 1 more card
  - When: `try_apply_miss_nuit_trigger` called with dead Miss Nuit entity
  - Then: `reserve = 1` (destruction stops further gains)

- **Own-card-play exclusion**:
  - Caller (RESOLUTION system) must NOT call `try_apply_miss_nuit_trigger` when the Xelor player's own cards commit. Verify calling site filter.

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/miss_nuit_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (PlayerSessions, `reserve` field) — must be DONE
- Depends on: `economy-system` epic (adds `miss_nuit_reserve_gained_this_round` tracker + reset logic) — must be DONE
- Depends on: `combat-resolution` epic (RESOLUTION sub-step 1 hook for opponent card plays; SILENCE status on Miss Nuit entity) — must be DONE for integration
- Unlocks: No direct story dependency; completes Xelor's passive income path alongside Stories 003–004
