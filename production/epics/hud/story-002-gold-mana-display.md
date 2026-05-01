# Story 002: Gold and Mana Display (ECONOMY_BASIC)

> **Epic**: HUD
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-001`, `TR-HUD-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: All S2C message draining happens in the `MessageDrain` set. Gold labels use a `GoldDisplayState { gold: f32, reserved_gold: f32, is_populated: bool }` backing component. A change-detection system in `StateSync` reads `GoldDisplayState` and writes the formatted `Text` string. `handle_gold_broadcast_system` runs `.before(handle_gold_update_system)` within `MessageDrain` — the tie-break ordering is a code contract enforced at plugin registration.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `MessageReceiver<S2CGoldUpdate>` (Lightyear) — drain only once per frame (first call consumes all). `query.single()` returns `Result` in Bevy 0.16+ — use `let Ok(label) = query.single() else { return; }`. `Text` component on parent entity, `TextSpan` on child entity for gold label. `Visibility` is a standalone component; toggle via `commands.entity(e).insert(Visibility::Hidden)`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Single `MessageReceiver<T>` drain per S2C message type per frame. Backing `f32` field in `GoldDisplayState`; string derived in `StateSync` via change detection.
- Forbidden: Never drain `MessageReceiver<S2CGoldUpdate>` or `MessageReceiver<S2CGoldBroadcast>` in more than one system. Never format strings inside the `MessageDrain` set.
- Guardrail: Client S2C processing ≤ 2 ms/frame.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [ ] **HUD-03** (BLOCKING): GIVEN HUD in ECONOMY_BASIC mode, WHEN `S2CGoldUpdate{gold=8, current_mana=6, mana_cap=10, reserve_mana=2}` and `S2CGoldBroadcast{player_id=opponent_id, gold=6, reserved_gold=0}` arrive, THEN own gold label `Text` reads `"8g"`, opponent gold label `Text` reads `"6g"`, mana label `Text` reads `"6 / 10"`, reserve label `Text` reads `"+2 reserve"` and is `Visibility::Visible`.
- [ ] **HUD-04** (BLOCKING): GIVEN HUD in any visible mode, WHEN only `S2CGoldUpdate` arrives (no other messages), THEN ONLY own gold/mana/reserve `GoldDisplayState` fields and their derived `Text` strings change; opponent label `Text`, phase label `Text`, round counter `Text`, and all dot state flags retain their prior values.
- [ ] **HUD-21** (BLOCKING): GIVEN mana label reads `"4 / 8"`, WHEN `S2CGoldUpdate{current_mana=4, mana_cap=10, reserve_mana=0}` arrives, THEN mana label `Text` updates to `"4 / 10"`; reserve label `Visibility::Hidden`.
- [ ] **HUD-25** (BLOCKING): GIVEN `S2CPhaseChanged(DRAFT_INITIAL)` received but no `S2CGoldUpdate` yet AND no `S2CGoldBroadcast` for opponent yet, THEN own gold label `Text` reads `"--g"`, mana label `Text` reads `"-- / --"`, opponent gold label `Text` reads `"--g"`. A subsequent `S2CGoldUpdate{gold=0}` MUST then produce `"0g"` (not `"--g"`).
- [ ] **HUD-31** (BLOCKING): GIVEN `S2CGoldUpdate{current_mana=0, mana_cap=0}` arrives, THEN mana label `Text` renders `"0 / 0"` without panic; a warning is logged.
- [ ] **Multi-update collapse** (BLOCKING): GIVEN 3 `S2CGoldUpdate` messages arrive in the same ECS tick (values: gold=7, gold=9, gold=11), WHEN all `MessageDrain` systems complete, THEN `GoldDisplayState.gold == 11.0`; no intermediate tween is initiated for the 7 or 9 values.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- `handle_gold_broadcast_system` (in `MessageDrain`, runs first): drains `MessageReceiver<S2CGoldBroadcast>`. If `player_id == opponent_id`: write full `GoldDisplayState{gold, reserved_gold, is_populated: true}` to opponent entity. If `player_id == local_id`: write ONLY `GoldDisplayState.reserved_gold` (do NOT touch `.gold` — that field belongs to `S2CGoldUpdate`).
- `handle_gold_update_system` (in `MessageDrain`, runs after): drains `MessageReceiver<S2CGoldUpdate>`. When multiple messages in batch, apply `.last()` only (multi-update collapse). Write `GoldDisplayState.gold`, `is_populated = true`, mana numerator/denominator to own entity. Write reserve label visibility.
- Change-detection system in `StateSync` (runs every frame): if `GoldDisplayState` is changed, format string and write to `Text`/`TextSpan`.
  - `is_populated == false` → `Text = "--g"` (own and opponent)
  - ECONOMY_BASIC: `Text = "{gold}g"` where `gold = state.gold as u32`
  - Reserve label: `Visibility::Visible` when `reserve_mana > 0`, else `Visibility::Hidden`
- `mana_cap=0` guard: format `"0 / 0"`, emit `warn!("HUD: mana_cap=0 received — server invariant violated")`.
- Overfull mana (`current_mana > mana_cap`): render as-is (e.g. `"5 / 3"`). Do not clamp.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 006]: ECONOMY_AUCTION parenthetical format `"Xg (Yr)"` and TextSpan content updates
- [Story 009]: Same-tick tie-break verified via `App::new()` with HudPlugin registered
- [Story 010]: Numeric tween animation of `GoldDisplayState` `f32` fields

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-03**: Basic display format correctness
  - Given: HUD initialized, `HudMode = ECONOMY_BASIC`, own and opponent gold `is_populated = false`
  - When: `S2CGoldUpdate{gold=8, current_mana=6, mana_cap=10, reserve_mana=2}` processed; then `S2CGoldBroadcast{player_id=opponent_id, gold=6, reserved_gold=0}` processed; `StateSync` runs
  - Then: Own gold `Text == "8g"`; opponent gold `Text == "6g"`; mana `Text == "6 / 10"`; reserve `Text == "+2 reserve"` and `Visibility::Visible`
  - Edge cases: `reserve_mana=0` → reserve label `Visibility::Hidden`; `current_mana == mana_cap` → `"10 / 10"` not `"MAX"`

**HUD-04**: Per-message update isolation
  - Given: Own gold `GoldDisplayState.gold = 5.0`; opponent gold `Text = "3g"`; phase label `Text = "DRAFT"`
  - When: Only `S2CGoldUpdate{gold=10, current_mana=4, mana_cap=8, reserve_mana=0}` processed; `StateSync` runs
  - Then: Own `GoldDisplayState.gold == 10.0`; opponent `GoldDisplayState.gold` unchanged; phase label `Text` still `"DRAFT"`
  - Edge cases: Confirm via component query — not just text read

**HUD-21**: Mana cap denominator update
  - Given: `GoldDisplayState` mana numerator=4, mana label reads `"4 / 8"`
  - When: `S2CGoldUpdate{current_mana=4, mana_cap=10, reserve_mana=0}` processed
  - Then: Mana label `Text == "4 / 10"`; reserve label `Visibility::Hidden`
  - Edge cases: `current_mana=5, mana_cap=3` (overfull) → `Text == "5 / 3"`

**HUD-25**: Cold-start placeholder display
  - Given: HUD initialized; `is_populated = false` on own and opponent entities
  - When: `StateSync` runs before any economy message
  - Then: Own gold `Text == "--g"`; mana `Text == "-- / --"`; opponent gold `Text == "--g"`
  - Edge cases: After `S2CGoldUpdate{gold=0}` → `Text == "0g"` and `is_populated = true`

**HUD-31**: mana_cap=0 guard
  - Given: HUD initialized
  - When: `S2CGoldUpdate{current_mana=0, mana_cap=0}` processed
  - Then: No panic; mana label `Text == "0 / 0"`; `warn!` logged
  - Edge cases: `mana_cap=1, current_mana=0` → `"0 / 1"` (normal path, no warning)

**Multi-update collapse**: Last-value-wins on lag burst
  - Given: `GoldDisplayState.gold = 5.0`
  - When: 3 `S2CGoldUpdate` in same tick: gold=7, gold=9, gold=11
  - Then: `GoldDisplayState.gold == 11.0`; no intermediate GoldDisplayState values of 7 or 9 committed
  - Edge cases: Burst of exactly 1 message → normal update

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/hud/gold_mana_display_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (entity pool + `GoldDisplayState` component + `HudEntities` resource)
- Unlocks: Story 006 (ECONOMY_AUCTION format builds on this), Story 009 (tie-break uses these systems)
