# Story 004: SessionReady Observer & Shop Refresh Subscriber

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: TR-CDP-04, TR-CDP-09 (pool initialized at `SessionReady`; `ShopRefreshNeeded` subscriber draws 3 or 9 slots; `CardPoolPlugin` registers all resources and systems)

**ADRs Governing Implementation**:
- ADR-006: Card Data Schema and Pool State Architecture — `on_session_ready_init` Observer initialises `PlayerPools` from `Res<SessionConfig>` + `Res<CardCatalog>` + `Res<GameConfig>`; soft error on `pool_copies_override <= 0`
- ADR-010: RSM Phase Event Bus — `on_shop_refresh_needed` subscribes to `ShopRefreshNeeded { player }` event (one per player per DRAFT entry); subscriber must run `.after(advance_phase)` to guarantee `RoundState.phase` is updated before the slot-count check
- ADR-012: Session Ready Delivery — `SessionReady` is the canonical trigger for all per-session resource initialization; `on_session_ready_init` must be an Observer (not a `startup` system) so it fires once per session, not once at app startup

**ADR Decision Summary**: `CardPoolPlugin` inserts all four resources (`PlayerPools`, `ShopSlots`, `InitialDraftOffering`, `ManualRefreshCount`) and registers `on_session_ready_init` as an Observer for `SessionReady`. The `ShopRefreshNeeded` subscriber determines slot count from `Res<RoundState>.phase` — 9 for `Phase::DraftInitial`, 3 for `Phase::DraftShop`. Per-player fan-out (N messages per frame for 1v1 to 3v3) is handled by sequential iteration via `MessageReader<ShopRefreshNeeded>::read()`. After each player's shop is drawn, `ManualRefreshCount[player]` is reset to 0.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**:
- `MessageReader::read()` (Bevy 0.18 — `EventReader` no longer exists; `EventReader` was removed in 0.17). `liv-bevy-018` skill mandatory.
- `Observer` API: in Bevy 0.18, observers are registered via `app.observe(on_session_ready_init)`. The callback receives `Trigger<SessionReady>` (verify exact `Trigger` vs `On<E>` parameter — post-cutoff API). Consult `liv-bevy-018` REFERENCE.md.
- `ShopRefreshNeeded` is a Bevy **Message** (`#[derive(Message)]`) with a two-frame lifetime. The subscriber must run every frame to avoid missing messages. Scheduling `.after(advance_phase)` from ADR-010 ensures correct phase visibility.
- `bevy_asset_loader` compatibility: `Res<CardCatalog>` is assumed to be inserted by the `game-config-pipeline` epic before `SessionReady` fires. Guard with `Option<Res<CardCatalog>>` if load ordering is uncertain.

**Control Manifest Rules (Core layer)**:
- Required: `on_session_ready_init` iterates `SessionConfig.team_map.keys()` in deterministic (sorted) order to maintain RNG audit log replay correctness (ADR-005).
- Required: `on_shop_refresh_needed` processes messages sequentially in emission order — Bevy's `MessageReader` provides FIFO ordering within a frame.
- Required: `ManualRefreshCount[player] = 0` reset happens inside `on_shop_refresh_needed` before `refresh_shop()` is called, so that a DRAFT_INITIAL entry starts with a clean counter.
- Required: `CardPoolPlugin` registers the `on_shop_refresh_needed` system with `.after(advance_phase)` using the ordering label exported by the Round State Machine epic.
- Forbidden: `on_session_ready_init` must NEVER abort or panic due to a bad `pool_copies_override` value — soft error and continue per ADR-006.
- Forbidden: No direct writes to `PlayerPools` outside `on_session_ready_init` and the systems delivering on `ShopRefreshNeeded` events in this story.

---

## Acceptance Criteria

- [ ] `server/src/core/pool/system.rs` exists and defines:
  - `on_session_ready_init(trigger: Trigger<SessionReady>, mut pools: ResMut<PlayerPools>, mut shop_slots: ResMut<ShopSlots>, mut offering: ResMut<InitialDraftOffering>, mut refresh_count: ResMut<ManualRefreshCount>, catalog: Res<CardCatalog>, config: Res<GameConfig>, session: Res<SessionConfig>)` — initializes all four resources for each player in the session
  - `on_shop_refresh_needed(mut events: MessageReader<ShopRefreshNeeded>, mut pools: ResMut<PlayerPools>, mut shop_slots: ResMut<ShopSlots>, mut offering: ResMut<InitialDraftOffering>, mut refresh_count: ResMut<ManualRefreshCount>, round_state: Res<RoundState>, catalog: Res<CardCatalog>, family_index: Res<FamilyIndex>, mut rng: ResMut<ServerRng>, config: Res<GameConfig>)` — processes per-player shop refresh messages — TODO(liv-bevy-018): verify MessageReader<T> type name
- [ ] `server/src/core/pool/plugin.rs` exists and defines `CardPoolPlugin` which:
  - Inserts `PlayerPools`, `ShopSlots`, `InitialDraftOffering`, `ManualRefreshCount` as default-empty resources at app startup
  - Registers `on_session_ready_init` as an Observer for the `SessionReady` event
  - Registers `on_shop_refresh_needed` as an `Update` system scheduled `.after(advance_phase)` ordering label
  - Registers a teardown subscriber on `GameOverEmitted` to clear all four per-session resources
- [ ] `on_session_ready_init` behavior:
  - For each `PlayerId` in `session.team_map.keys()` (sorted ascending for RNG determinism), calls `PlayerPool::initialize(catalog, config)`
  - Inserts initialized pool into `PlayerPools.0`
  - Inserts empty `Vec<CardId>` into `ShopSlots.0` and `InitialDraftOffering.0` for each player
  - Inserts `0u32` into `ManualRefreshCount.0` for each player
  - Logs a warning (not panic) for any `pool_copies_override <= 0` encountered during initialization
- [ ] `on_shop_refresh_needed` behavior per event:
  - Determines `slot_count`: 9 if `round_state.phase == Phase::DraftInitial`, 3 if `Phase::DraftShop`
  - Resets `ManualRefreshCount[player] = 0`
  - Calls `refresh_shop(pool, catalog, family_index, rng, config, slot_count)`
  - For `DraftInitial`: writes result to `InitialDraftOffering[player]`; enqueues `S2CDraftOffering { player, offering: Vec<CardId> }`
  - For `DraftShop`: writes result to `ShopSlots[player]`; enqueues `S2CShopSlots { player, slots: Vec<CardId> }`
  - If drawn slots < `slot_count` (partial fill): fills remaining positions as `None` in the outbound message; logs a debug warning
- [ ] Per-player fan-out: GIVEN two `ShopRefreshNeeded` events for `Player A` and `Player B` in the same frame, WHEN `on_shop_refresh_needed` processes both, THEN `ShopSlots[A]` and `ShopSlots[B]` are both populated; draws are independent (Player A's pool draw does not affect Player B's pool)
- [ ] Slot count selection: GIVEN `round_state.phase == Phase::DraftInitial`, WHEN event fires, THEN `slot_count == 9`; GIVEN `round_state.phase == Phase::DraftShop`, THEN `slot_count == 3`
- [ ] Counter reset: GIVEN `ManualRefreshCount[player] = 3` (from prior DRAFT_SHOP), WHEN `ShopRefreshNeeded` fires for that player on next DRAFT entry, THEN `ManualRefreshCount[player] == 0` after the system runs
- [ ] Teardown on `GameOverEmitted`: GIVEN a completed session, WHEN `GameOverEmitted` fires, THEN `PlayerPools.0.is_empty()`, `ShopSlots.0.is_empty()`, `InitialDraftOffering.0.is_empty()`, `ManualRefreshCount.0.is_empty()`
- [ ] `CardPoolPlugin` registers cleanly in a headless Bevy `App::new()` startup test with mock `CardCatalog` and `GameConfig` inserted as resources
- [ ] Integration test: `ShopRefreshNeeded { player: A }` + `ShopRefreshNeeded { player: B }` in the same frame → both shops drawn; both `S2CShopSlots` events enqueued for correct targets
- [ ] `cargo check -p server` passes after adding `system.rs` and `plugin.rs`

---

## Implementation Notes

*Derived from EPIC.md §Deliverables, ADR-006 §Decision, ADR-010 §Subscriber Contracts, ADR-012 §Session Ready Delivery:*

**Observer registration pattern (Bevy 0.18):**
```rust
// plugin.rs
impl Plugin for CardPoolPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PlayerPools::default())
            .insert_resource(ShopSlots::default())
            .insert_resource(InitialDraftOffering::default())
            .insert_resource(ManualRefreshCount::default())
            .observe(on_session_ready_init)
            .add_systems(Update, on_shop_refresh_needed.after(AdvancePhaseSet))
            .add_systems(Update, on_manual_refresh.after(AdvancePhaseSet));  // Story 005
    }
}
```
`AdvancePhaseSet` is the scheduling label exported by the Round State Machine epic. Verify its exact type (`SystemSet` or label string) against the RSM epic before implementing.

**RNG determinism — player iteration order:** In `on_session_ready_init`, sort `session.team_map.keys()` before iterating. `PlayerId` must implement `Ord` or `PartialOrd` for this. If `PlayerId = u32`, standard sort order applies. This ensures the audit log records seed consumption in the same order on every server restart given identical inputs.

**`S2CShopSlots` / `S2CDraftOffering` enqueue pattern:** These are Bevy buffered Messages (not direct network sends). Story 006's network dispatch system reads these messages via `MessageReader<T>` and sends the actual Lightyear messages. In this story, enqueuing means `MessageWriter<S2CShopSlots>.write(S2CShopSlots { player_id, slots })` — TODO(liv-bevy-018): verify MessageWriter type name. The message types derive `Message` and are registered via `app.add_message::<T>()`. Types defined in `shared/src/protocol.rs`.

**Partial fill message encoding:** When `refresh_shop()` returns fewer cards than `slot_count`, the outbound `S2CShopSlots` message should use `Vec<Option<CardId>>` (not `Vec<CardId>`) to encode empty slots as `None`. If the protocol type uses `Vec<CardId>`, discuss with the network programmer whether to use a sentinel value or extend the type — do not silently truncate.

**Teardown system:** Register on `GameOverEmitted` (an observer or a system reading the event). Clear by calling `.0.clear()` on each resource, not by removing and re-inserting resources — the resource handles remain valid for the next session initialization.

---

## Out of Scope

- Story 001: Resource type definitions (`PlayerPools`, `ShopSlots`, `InitialDraftOffering`, `ManualRefreshCount`)
- Story 002: Weighted draw logic
- Story 003: `refresh_shop()`, `draw_initial_draft()`, `FamilyIndex`
- Story 005: `on_manual_refresh` system — referenced in `CardPoolPlugin.build()` but implemented in Story 005
- Story 006: Network dispatch — this story enqueues `S2CShopSlots` and `S2CDraftOffering` as ECS events; Story 006 sends them over Lightyear
- `SessionConfig` and `SessionReady` event definitions — Game Session System epic
- `RoundState` resource and `Phase` enum — Round State Machine epic
- `ShopRefreshNeeded` event definition and emission — Round State Machine epic (Epic 1)

---

## QA Test Cases

- **SessionReady initializes all players**
  - Given: a mock `SessionConfig` with 2 players (IDs 1, 2); mock `CardCatalog` with 5 cards; valid `GameConfig`
  - When: `SessionReady` trigger fires
  - Then: `PlayerPools.0.contains_key(&1)` and `PlayerPools.0.contains_key(&2)`; each pool has 5 entries in `copies_remaining`

- **Per-player fan-out in same frame**
  - Given: both players initialized; `round_state.phase = DraftShop`
  - When: `ShopRefreshNeeded { player: 1 }` and `ShopRefreshNeeded { player: 2 }` emitted in same frame
  - Then: `ShopSlots.0[&1].len() == 3`; `ShopSlots.0[&2].len() == 3`; card IDs in player 1's shop not constrained to differ from player 2's (independent pools)

- **Slot count 9 for DraftInitial, 3 for DraftShop**
  - When phase is `DraftInitial`: assert `refresh_shop` called with `slot_count = 9`
  - When phase is `DraftShop`: assert called with `slot_count = 3`

- **ManualRefreshCount reset on DRAFT entry**
  - Given: `ManualRefreshCount[player] = 5`
  - When: `ShopRefreshNeeded { player }` fires
  - Then: `ManualRefreshCount[player] == 0`

- **Teardown on GameOverEmitted**
  - Given: session fully initialized with 2 players
  - When: `GameOverEmitted` fires
  - Then: all four resources have empty HashMaps

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/pool/session_init_shop_refresh_test.rs` — all acceptance criteria passing; covers `on_session_ready_init`, per-player fan-out, slot count selection, `ManualRefreshCount` reset, teardown on `GameOverEmitted`, `CardPoolPlugin` headless app startup
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (provides `PlayerPools`, `ShopSlots`, `InitialDraftOffering`, `ManualRefreshCount`, `PlayerPool::initialize()`)
- Depends on: Story 002 (provides `draw_class_card`, `draw_neutral_family`, `draw_family_card`)
- Depends on: Story 003 (provides `refresh_shop()`, `draw_initial_draft()`, `FamilyIndex`, `build_family_index()`)
- Depends on: `round-state-machine` epic — `ShopRefreshNeeded` event type, `RoundState` resource, `Phase` enum, `AdvancePhaseSet` scheduling label
- Depends on: `game-session-system` epic — `SessionReady` event, `SessionConfig` resource, `GameOverEmitted` event
- Depends on: `workspace-and-shared-types` Story 004 — `S2CShopSlots` and `S2CDraftOffering` event types in `shared/src/protocol.rs`
- Depends on: `server-rng` Story 001 — `ServerRng` resource
- Unlocks: Story 005 (manual refresh subscriber — depends on initialized `PlayerPools` and `ManualRefreshCount`)
- Unlocks: Story 006 (network dispatch — depends on `S2CShopSlots` and `S2CDraftOffering` events being enqueued)
