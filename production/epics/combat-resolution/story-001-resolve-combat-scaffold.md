# Story 001: resolve_combat Scaffold + Safety Timeout

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: TR-CR-013 (scaffold + idle-exit + iteration budget), TR-CR-014 (PlacementReveal timing), TR-CR-015 (ResolutionEvent ordering invariant), TR-CR-001 (sub-step 1 placement effects — partial)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: `resolve_combat(world: &mut World)` is an exclusive Bevy system registered via `add_systems(Update, resolve_combat)`. It reads `MessageReader<BeginResolution>` (exits if none), executes sub-steps 1–6 as sequential function calls, writes `MessageWriter<ResolutionComplete>`, and broadcasts `S2CResolutionEvent` via Lightyear. The 60s RSM safety timeout and a 10,000-iteration internal budget guard against infinite loops.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Exclusive system auto-detection via `&mut World` is stable since before Bevy 0.14 — no breaking changes through 0.18. `MessageWriter<T>`/`MessageReader<T>` are Bevy-internal buffered messages registered via `app.add_message::<T>()` — distinct from Lightyear's `MessageSender<T>`/`MessageReceiver<T>` used for network protocol. Do NOT confuse the two: `BeginResolution` and `ResolutionComplete` use Bevy-internal `MessageWriter`; `S2CResolutionEvent` uses Lightyear `MessageSender`. Verify against `docs/engine-reference/bevy/VERSION.md` and the Lightyear 0.26 verification checklist in `control-manifest.md` before implementing.

**Control Manifest Rules (Feature layer)**:
- Required: `app.add_message::<BeginResolution>()` and `app.add_message::<ResolutionComplete>()` for Bevy-internal bus; `S2CResolutionEvent` registered via Lightyear protocol plugin; `S2CPlacementReveal` enqueued on `ReliableChannel` BEFORE any entity spawning
- Forbidden: Never use `EventWriter<T>`/`EventReader<T>` (removed in Bevy 0.17+); never call RSM functions directly from `resolve_combat`
- Guardrail: Server RESOLUTION batch ≤ 15ms; `resolve_combat` exits in < 1ms when `BeginResolution` is absent

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-30 (structural)**: `S2CPlacementReveal` is enqueued on `ReliableChannel` before any sub-step 1 effect executes — enforced by execution order within `resolve_combat`, not by Bevy system ordering
- [ ] **CR-32 (ordering invariant)**: `S2CResolutionEvent` is enqueued before `S2CPhaseChanged(DRAFT_SHOP)` — enforced by `resolve_combat` writing `ResolutionComplete` only AFTER `S2CResolutionEvent` is enqueued, and the RSM broadcasting `S2CPhaseChanged` only after `ResolutionComplete` arrives
- [ ] **CR-41 (iteration budget)**: If the internal iteration counter exceeds 10,000 total iterations (across all sub-step loops in a single RESOLUTION), `resolve_combat` aborts and the RSM broadcasts `S2CGameOver { loser: None, reason: Draw }`; `ResolutionComplete` is NOT written in this case
- [ ] When no `BeginResolution` message is present, `resolve_combat` returns immediately without touching any ECS state

---

## Implementation Notes

*Derived from ADR-017 Decision 1 and Key Interfaces:*

```rust
// server/src/feature/combat/mod.rs
pub fn resolve_combat(world: &mut World) {
    // 1. Read BeginResolution — exit immediately if absent
    let has_trigger = world.resource_mut::<MessageReader<BeginResolution>>()
        .read().next().is_some();
    if !has_trigger { return; }

    // 2. Enqueue S2CPlacementReveal BEFORE spawning any entities
    // ... build payload from PendingPlacements ...
    world.resource_mut::<MessageSender<S2CPlacementReveal>>()
        .send(..., NetworkTarget::All);

    // 3. Take UnitSnapshot of all units + LEADER bonuses
    let mut snapshots = snapshot_units(world);

    // 4. Initialise iteration counter
    let mut iter_count: u32 = 0;

    // 5. Execute sub-steps 1–6 as sequential function calls
    let mut log = ResolutionLog::default();
    apply_placements(world, &mut snapshots, &mut log, &mut iter_count);
    execute_charge_x(world, &mut snapshots, &mut log, &mut iter_count);
    execute_first_strike(world, &mut snapshots, &mut log, &mut iter_count);
    remove_dead(world, &mut snapshots, &mut log, &mut iter_count);
    execute_movement(world, &mut snapshots, &mut log, &mut iter_count);
    execute_combat(world, &mut snapshots, &mut log, &mut iter_count);

    // 6. Broadcast S2CResolutionEvent FIRST
    world.resource_mut::<MessageSender<S2CResolutionEvent>>()
        .send(S2CResolutionEvent { round, events: log.into_vec() }, NetworkTarget::All);

    // 7. Write ResolutionComplete AFTER broadcast
    world.resource_mut::<MessageWriter<ResolutionComplete>>().write(ResolutionComplete);
}
```

**Iteration budget guard** — pass `iter_count` into every sub-step loop. If it exceeds 10_000, return an error variant. The calling frame in `resolve_combat` checks the result and, on overflow, writes `GameOver { reason: Draw }` to the RSM event bus instead of `ResolutionComplete`.

**Schedule placement**: `resolve_combat` must run AFTER `placement_buffer_close_system` and AFTER `MessageSendSystems` for `BeginResolution` to be readable. See ADR-009 system schedule order: `AuctionSystem → CombatResolutionSystem → rsm_tick_system → MessageSendSystems`.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 002: Actual `apply_combat_modifier_stack` logic
- Story 003: SS1 placement commit and APPEARANCE triggers
- Stories 004–010: Individual sub-step implementations
- Story 011: Full ResolutionEvent log population and content verification

---

## QA Test Cases

*(Lean mode — QL-STORY-READY skipped; test cases authored inline)*

- **CR-30 (structural)**:
  - Given: `resolve_combat` world with `BeginResolution` present and `PendingPlacements` with 1 unit
  - When: `resolve_combat` executes
  - Then: `S2CPlacementReveal` message is enqueued before any `UnitPlaced` event in the ResolutionLog; assert log index of `UnitPlaced` > 0 with PlacementReveal confirmed sent
  - Edge cases: empty `PendingPlacements` — PlacementReveal still sent (empty payload)

- **CR-32 (ordering invariant)**:
  - Given: World where `resolve_combat` completes normally
  - When: RSM receives `ResolutionComplete`
  - Then: `S2CResolutionEvent` was enqueued in the same frame as `ResolutionComplete` write, and `S2CPhaseChanged` can only be enqueued in the NEXT RSM tick after `ResolutionComplete` arrives
  - Edge cases: test that `ResolutionComplete` write happens AFTER `S2CResolutionEvent` send call

- **CR-41 (idle exit)**:
  - Given: World with NO `BeginResolution` message
  - When: `resolve_combat` runs
  - Then: `ResolutionComplete` is NOT written; `S2CResolutionEvent` is NOT sent; no ECS mutations occur

- **CR-41 (iteration budget)**:
  - Given: Synthetic sub-step that increments `iter_count` past 10,000
  - When: `resolve_combat` detects budget exceeded
  - Then: `GameOver { loser: None, reason: Draw }` is broadcast; `ResolutionComplete` is NOT written

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/combat/resolve_combat_scaffold_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None — first story in this epic
- Unlocks: Story 002 (UnitSnapshot struct used by modifier stack), Story 003 (SS1 runs inside resolve_combat)
