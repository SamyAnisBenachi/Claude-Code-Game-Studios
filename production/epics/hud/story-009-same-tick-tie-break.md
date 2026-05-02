# Story 009: Same-Tick Gold Tie-Break (Plugin-Level Integration)

> **Epic**: HUD
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `handle_gold_broadcast_system` is scheduled `.before(handle_gold_update_system)` within `MessageDrain` set via `app.configure_sets` or explicit `.before()` constraint declared in `HudPlugin::build()`. This ordering ensures that when both messages arrive in the same ECS tick, `S2CGoldUpdate` wins for `GoldDisplayState.gold` on the own gold label. The ordering is declared at plugin registration time — not enforced by calling order in a single system.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: System ordering declared via `app.add_systems(Update, handle_gold_update_system.after(handle_gold_broadcast_system).in_set(PresentationSet::MessageDrain))`. This ordering can only be verified by running both systems inside a registered `App` (i.e., with `App::new()` + `HudPlugin` + `PresentationPlugin`). A `World::new()` unit test that manually calls systems in order does NOT verify plugin system ordering — it only verifies the individual system logic.

**Control Manifest Rules (Presentation Layer)**:
- Required: `handle_gold_broadcast_system` runs `.before(handle_gold_update_system)` — this ordering is a code contract enforced in `HudPlugin::build()`. Both systems in `MessageDrain` set.
- Forbidden: Never verify plugin system ordering via `World::new()` — use `App::new()` with `HudPlugin` registered.
- Guardrail: Own `GoldDisplayState.gold` always reflects the `S2CGoldUpdate` value when both messages arrive same tick.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [x] **HUD-20** (BLOCKING): GIVEN `S2CGoldUpdate{gold=15, current_mana=0, reserve_mana=0, mana_cap=10}` and `S2CGoldBroadcast{player_id=local_id, gold=12, reserved_gold=0}` arrive in the same ECS tick, WHEN all HUD `MessageDrain` systems complete, THEN `GoldDisplayState.gold` on the own gold label entity reads `15.0` (from `S2CGoldUpdate`), confirming that the `.before()` ordering declared in `HudPlugin::build()` is in effect. **This test MUST use `App::new()` with `HudPlugin` registered, NOT `World::new()`.**
- [x] **Own `reserved_gold` unaffected by `S2CGoldBroadcast.gold`** (BLOCKING): In the same-tick scenario, `S2CGoldBroadcast{player_id=local_id, gold=12, reserved_gold=0}` MUST NOT overwrite `GoldDisplayState.gold`. The broadcast's `gold` field is ignored for the own label — only `reserved_gold` is written.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- In `HudPlugin::build()`, declare system ordering explicitly:
  ```rust
  app.add_systems(
      Update,
      (
          handle_gold_broadcast_system
              .before(handle_gold_update_system)
              .in_set(PresentationSet::MessageDrain),
          handle_gold_update_system
              .in_set(PresentationSet::MessageDrain),
      )
  );
  ```
- `handle_gold_broadcast_system`: when `player_id == local_id`, writes ONLY `GoldDisplayState.reserved_gold`. Does NOT write `GoldDisplayState.gold`. The `gold` field from `S2CGoldBroadcast` is intentionally unused for the own label.
- `handle_gold_update_system`: writes `GoldDisplayState.gold` (and mana fields). Does NOT write `GoldDisplayState.reserved_gold`.
- Result: if both arrive same tick — broadcast runs first (writes `reserved_gold = 0`), update runs second (writes `gold = 15`). Final state: `GoldDisplayState{ gold: 15.0, reserved_gold: 0.0 }`. Correct.
- The correctness of same-tick ordering is **structural** (separate fields, no conflict) per GDD Rule 11 field-split proof. The `.before()` is belt-and-suspenders, not the primary mechanism.
- Test must use `App::new()` because it is verifying that the `.before()` declaration in `HudPlugin::build()` is actually in effect. Without `HudPlugin` registered, no system ordering is declared and the test verifies nothing meaningful about production behaviour.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: Individual system logic for `handle_gold_update_system` and `handle_gold_broadcast_system`
- [Story 006]: ECONOMY_AUCTION display where `reserved_gold` is shown

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-20**: Same-tick tie-break via App::new()
  - Given: `App::new()` with `HudPlugin` registered (full plugin, system ordering active); HUD in ECONOMY_BASIC mode
  - When: Both `S2CGoldUpdate{gold=15, current_mana=0, reserve_mana=0, mana_cap=10}` and `S2CGoldBroadcast{player_id=local_id, gold=12, reserved_gold=0}` inserted into Lightyear message queues for the same tick; `App::update()` called once
  - Then: `GoldDisplayState.gold == 15.0` on own gold label entity; `GoldDisplayState.reserved_gold == 0.0`
  - Edge cases: Verify with `S2CGoldBroadcast{gold=99, reserved_gold=5}` — `GoldDisplayState.gold` must still be 15.0 (not 99); `reserved_gold` must be 5.0 (from broadcast)

**Own reserved_gold isolation**: broadcast.gold not written to own label
  - Given: Own `GoldDisplayState.gold = 10.0`; only `S2CGoldBroadcast{player_id=local_id, gold=3, reserved_gold=2}` in tick
  - When: `MessageDrain` runs
  - Then: `GoldDisplayState.gold == 10.0` (unchanged); `GoldDisplayState.reserved_gold == 2.0` (from broadcast)
  - Edge cases: Opponent broadcast `player_id=opponent_id` → does NOT write own `GoldDisplayState` at all

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/hud/same_tick_tie_break_test.rs` — must use `App::new()` with `HudPlugin` registered; must exist and pass

**Status**: [x] Created and passing (`cargo test -p client --test same_tick_tie_break_test`)

---

## Dependencies

- Depends on: Story 002 (`GoldDisplayState`, both handler systems implemented), Story 006 (format context — though tie-break test uses ECONOMY_BASIC)
- Unlocks: None (standalone integration correctness story)

## Completion Notes

**Completed**: 2026-05-02
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 2/2 passing; HUD-20 and own reserved_gold isolation are covered by `tests/integration/hud/same_tick_tie_break_test.rs`.
**Test Evidence**: `cargo test -p client --test same_tick_tie_break_test` passed 3/3. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
**Verification**: `HudPlugin::build()` registers `handle_gold_broadcast_system.before(handle_gold_update_system)` in `HudSystemSet::MessageDrain`. `handle_gold_broadcast_system` writes only `GoldDisplayState.reserved_gold` for the local label, while `handle_gold_update_system` writes local `GoldDisplayState.gold`; the integration test uses `App::new()` with `HudPlugin` registered.
**Notes**: Advisory only - the test injects `HudGoldBroadcastMessage` / `HudGoldUpdateMessage` directly into Bevy messages after the Lightyear drain seam. It verifies plugin-level handler ordering and field ownership, but does not instantiate real Lightyear receiver entities. Lean mode skipped external QA/code-review gates.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged per user instruction; no explicit `HUD-009` row exists in `production/sprint-status.yaml`.
**Next Recommended**: HUD Story 008 Reconnect Snapshot Rebuild (`production/epics/hud/story-008-reconnect-snapshot-rebuild.md`) after readiness check, or continue the serialized closure queue for HUD Story 010 Numeric Tween Animation if its implementation is ready for closure.
