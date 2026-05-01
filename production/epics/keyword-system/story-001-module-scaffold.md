# Story 001: Keyword System Module Scaffold

> **Epic**: Keyword System
> **Status**: Complete
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-001 through TR-KW-012 (infrastructure for all keyword requirements)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture) + ADR-022 (Timing Trigger Observer Architecture)
**ADR Decision Summary**: All 6 persistent keyword states co-located in a single `UnitKeywordState` component per board unit entity. Module tree at `server/feature/keyword/` with `effects.rs` called by combat resolution. Five timing triggers dispatched via global Bevy Observers; COUNTERATTACK and INJURED dispatched inline. Protocol types in `protocol/src/keyword.rs`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `#[derive(Component)]` with `Option<Entity>` field — entity handle semantics Bevy 0.18; verify `&Entities` system param for alive-check API
- `#[derive(Message)]` + `MessageWriter<T>` / `MessageReader<T>` — Bevy 0.17+ Message/Event split (post-cutoff); `app.add_message::<KeywordTriggered>()` must be in `KeywordPlugin::build()` or first write panics
- `#[derive(Event)]` + `app.observe()` + `Trigger<T>` param type — Bevy 0.17+ Observer split; confirm `Trigger<T>` not `On<T>` before coding
- `world.trigger_targets(event, entity)` — confirm this is valid `World` method in Bevy 0.18 (vs. `world.commands().trigger_targets() + world.flush()`)

**Control Manifest Rules (Feature layer)**:
- Required: Every Feature system reacting to phase changes must subscribe to the relevant RSM event — never observe `RoundState` directly (ADR-010)
- Required: Dependency direction `feature/ → core/ → foundation/` only; no reverse imports (ADR-003)
- Forbidden: Never let Feature systems call Core/Foundation systems directly — communicate upward via events (ADR-010)
- Forbidden: Never serialize `bodyguard_protects: Option<Entity>` into protocol types — use stable `EntityId` in protocol (ADR-002)

---

## Acceptance Criteria

*This is a scaffold story — no behavioral GDD ACs. The scaffold is "done" when all downstream stories can compile against its interface.*

- [x] `server/feature/keyword/` module tree exists with all files: `mod.rs`, `components.rs`, `events.rs`, `observers.rs`, `resources.rs`, `effects.rs`, `state_eval.rs`, `movement.rs`
- [x] `UnitKeywordState` component defined in `components.rs` with all 6 fields: `shield_active: bool`, `stun_active: bool`, `silenced_until_round: Option<u32>`, `leader_bonus_atk: u8`, `leader_bonus_hp: u8`, `bodyguard_protects: Option<Entity>`, `outnumbered_active: bool`
- [x] `KeywordPlugin::build()` registers: 5 global observers (`on_unit_appeared`, `on_unit_died`, `on_final_blow_dealt`, `on_start_of_turn`, `on_end_of_turn`), `ChainDeathBuffer` resource, `start_of_turn_dispatch_system`, `app.add_message::<KeywordTriggered>()`
- [x] `events.rs` defines: `UnitAppeared { sub_step: u8 }`, `UnitDied { attacker: Option<Entity> }`, `FinalBlowDealt { killed: Entity, sub_step: u8 }`, `StartOfTurnTriggered`, `EndOfTurnTriggered` — all `#[derive(Event)]`
- [x] `resources.rs` defines: `ChainDeathBuffer(pub VecDeque<(Entity, Option<Entity>)>)` with `#[derive(Resource, Default)]`
- [x] `protocol/src/keyword.rs` defines: `KeywordKind`, `KeywordPayload`, `InjuredGrantedKeyword` (4 variants: `FirstStrike, Counterattack, Range, Shield`), `DisplacementEvent`, `DisplacementKind` — all `#[derive(Serialize, Deserialize)]`
- [x] All effect functions in `effects.rs` stubbed with `todo!()`: `apply_first_strike`, `check_shield_absorb`, `apply_bodyguard_bond`, `apply_repel`, `apply_attract`, `apply_teleport`, `apply_change_lane`, `check_irremovable`, `check_counterattack_proximity`, `apply_counterattack`
- [x] All observer handler bodies in `observers.rs` stubbed with `todo!()`: `on_unit_appeared`, `on_unit_died`, `on_final_blow_dealt`, `on_start_of_turn`, `on_end_of_turn`, `start_of_turn_dispatch_system`
- [x] State eval functions in `state_eval.rs` stubbed with `todo!()`: `leader_snapshot_system`, `eval_outnumbered_system`, `eval_injured_bonuses`, `bodyguard_cleanup_system`
- [x] Movement formulas in `movement.rs` stubbed with `todo!()`: `repel_destination(target_cell: u8, owner: PlayerSide, x: u8) -> u8`, `attract_destination(caster_cell: u8, target_cell: u8, x: u8) -> u8`
- [x] Integration smoke test confirms `app.add_message::<KeywordTriggered>()` is registered and a test `MessageWriter::write()` does not panic

---

## Implementation Notes

*Derived from ADR-018 Parts 1–4 and ADR-022 Parts 2–4:*

**UnitKeywordState (ADR-018 Part 1)** — monolithic component chosen over individual components to avoid up to 720 archetype migrations per RESOLUTION round (10 units × 6 states × 12 sub-step boundaries). Co-locate all 6 fields. Field `silenced_until_round: Option<u32>` uses `u32` to match `round_number` type (NP R6 authoritative — previous versions used `Option<u8>`, which is wrong).

**BODYGUARD bond** — `bodyguard_protects: Option<Entity>` is a typed Bevy handle, NOT a lane index. Stable across CHANGE LANE. NEVER serialize into `protocol/` types — protocol uses `EntityId` (session-scoped u32). The `bodyguard_cleanup_system` in PostUpdate clears stale refs using `&Entities` (Bevy 0.18 alive-check system param — verify exact symbol path before coding).

**Module interface boundary (ADR-018 Part 2)** — `server/feature/combat/` calls into `server/feature/keyword::effects::*` functions as plain function calls with query references. The keyword module does NOT schedule its own systems against the combat sub-step timeline — that timeline is owned entirely by combat resolution.

**Protocol types (ADR-018 Part 4)** — `KeywordPayload` uses one variant per keyword event. `DisplacementEvent` has `was_blocked: bool` (IRREMOVABLE rejected) and `to_cell` reflecting actual final position after Trap interruption. Note: the NP GDD renamed `keyword`→`kind` and `was_blocked: bool`→`block_reason: Option<DisplacementBlockReason>` — coordinate with network-protocol.md before finalising field names.

**Observer registration (ADR-022 Part 4)** — global observers registered once in `KeywordPlugin::build()` via `app.observe(handler)`. Every observer handler MUST guard-check keyword presence as its first operation and return early if the entity lacks the relevant keyword — global observers fire for ALL entities receiving the trigger.

**`ChainDeathBuffer` (ADR-022 Part 3)** — `VecDeque<(Entity, Option<Entity>)>` where tuple is `(dying_entity, attacker)`. Cleared at SS4 start before seeding. The explicit queue avoids recursive `world.trigger_targets()` inside observer handlers (unverified borrow semantics in Bevy 0.18).

---

## Out of Scope

- Story 002: movement formula implementations (currently stubbed with `todo!()`)
- Stories 003–011: keyword effect function bodies (currently stubbed)
- Stories 012–015: observer handler bodies (currently stubbed)
- Story 016: displacement keyword implementations (currently stubbed)

---

## QA Test Cases

*Scaffold story — integration smoke test only.*

- **Smoke test: Plugin registration**
  - Given: `KeywordPlugin` added to a test `App`
  - When: app is built and a `MessageWriter<KeywordTriggered>` writes one message
  - Then: no panic; message is readable via `MessageReader<KeywordTriggered>` in the same frame
  - Edge cases: missing `app.add_message::<KeywordTriggered>()` registration → panic with descriptive message (not silent failure)

- **Smoke test: ChainDeathBuffer initial state**
  - Given: `KeywordPlugin` added to a test `App`
  - When: `ChainDeathBuffer` resource is read at startup
  - Then: `ChainDeathBuffer.0.is_empty() == true`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/integration/keyword/plugin_smoke_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: None (first story in epic)
- Depends on (pre-impl gates — must resolve before opening):
  - ADR-018 Accepted
  - ADR-022 Accepted
  - ADR-006 amendment merged (extended `SimpleKeyword` enum)
  - ADR-022 Verification Required × 5 resolved
- Unlocks: Stories 002–016 (all downstream stories depend on scaffold)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 11/11 passing
**Deviations**: Advisory only - protocol keyword types live in `shared/src/keyword.rs` because this workspace uses `shared/`, `server/`, and `client/` crates rather than a standalone `protocol/` crate; current GDD/ADR/control-manifest wording supports the implementation. `docs/architecture/tr-registry.yaml` has stale wording for TR-KW-006 and TR-KW-012; registry was not edited during closure.
**Test Evidence**: Logic story evidence at `tests/integration/keyword/plugin_smoke_test.rs`; `cargo test -p server --test keyword_plugin_smoke_test` passed with 2/2 tests; `cargo check --workspace` passed with warnings only.
**Code Review**: Skipped - lean review mode.
