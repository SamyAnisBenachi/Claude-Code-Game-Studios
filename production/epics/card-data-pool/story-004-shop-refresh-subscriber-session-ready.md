# Story 004: ShopRefreshNeeded Subscriber + SessionReady Init

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirements**: `TR-CDP-04` (PlayerPool per-player init), `TR-CDP-09` (ShopRefreshNeeded subscriber)
*(TR-IDs are informal — `docs/architecture/tr-registry.yaml` is unpopulated.)*

**ADRs Governing Implementation**:
- [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md) — pool initialization from `SessionReady`; per-player isolation
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md) — `ShopRefreshNeeded { player }` is a `#[derive(Message)]` type; subscriber uses `MessageReader<ShopRefreshNeeded>`; must run `.after(advance_phase)`; `SessionReady` uses Observer pattern

**ADR Decision Summary**: `SessionReady` fires as a Bevy Observer (same-frame trigger); Card Pool observes it to initialize `PlayerPools`. `ShopRefreshNeeded { player }` is a buffered Bevy Message read via `MessageReader` — NOT `EventReader`. System must be scheduled `.after(advance_phase)`. Per-player fan-out: RSM writes one `ShopRefreshNeeded` per player; Card Pool processes N messages per frame.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM (post-cutoff Bevy 0.17+ Message API; Observer pattern)
**Engine Notes**:
- `EventWriter`/`EventReader` no longer exist in Bevy 0.17+. Use `MessageWriter<T>`/`MessageReader<T>` + `app.add_message::<T>()`.
- `SessionReady` uses `#[derive(Event)]` + `app.observe(on_session_ready_init)` — NOT `add_message`.
- `query.single()` returns `Result` in Bevy 0.16+ — use `let Ok(x) = query.single() else { return; }`.
- Verify exact `MessageReader::read()` iterator API against Bevy 0.18 docs before implementing. `liv-bevy-018` skill is mandatory.

**Control Manifest Rules (Core layer)**:
- Required: `SessionReady` must be observed via `app.observe()`. Never use `MessageReader<SessionReady>` — it will never fire.
- Required: `ShopRefreshNeeded` subscriber must run `.after(advance_phase)` in the scheduling chain.
- Required: `SessionConfig` must be present in `World` before `SessionReady` fires (ADR-012 contract).
- Forbidden: `EventReader<ShopRefreshNeeded>` — compile error on Bevy 0.18.
- Forbidden: Spawning `SessionReady` as a buffered Message — it is an Observer Event.
- Guardrail: Per-player fan-out must complete within single frame (Bevy 2-frame message lifetime).

---

## Acceptance Criteria

*From ADR-006 and ADR-010 subscriber contracts, scoped to this story:*

- [ ] **AC-1**: GIVEN `SessionReady` Observer fires in a Bevy `World` with `Res<SessionConfig>` containing 2 player IDs, `Res<CardCatalog>` (fixture), and `Res<GameConfig>`, WHEN `on_session_ready_init` runs, THEN `Res<PlayerPools>` is inserted containing entries for both player IDs, each with `copies_remaining.len() == catalog.len()` and all values `>= 1`.
- [ ] **AC-2**: GIVEN `ShopRefreshNeeded { player: A }` and `ShopRefreshNeeded { player: B }` written to the Bevy `World` in the same frame, WHEN `on_shop_refresh_needed` processes both, THEN `ShopSlots[A]` and `ShopSlots[B]` are independently populated — Player A's draw does NOT change `PlayerPool[B].copies_remaining`.
- [ ] **AC-3**: GIVEN `ShopRefreshNeeded { player: P }` with `Res<RoundState>.phase == DraftInitial`, WHEN `on_shop_refresh_needed` runs, THEN `InitialDraftOffering[P]` is populated with up to 9 entries AND `ShopSlots[P]` is NOT written for this player.
- [ ] **AC-4**: GIVEN `ShopRefreshNeeded { player: P }` with `Res<RoundState>.phase == DraftShop`, WHEN `on_shop_refresh_needed` runs, THEN `ShopSlots[P]` is populated with up to 3 entries AND `InitialDraftOffering[P]` is NOT written for this player.
- [ ] **AC-5**: GIVEN `CardPoolPlugin` registered in a minimal `App::new()` with mock `Res<CardCatalog>` and `Res<GameConfig>` inserted before startup, WHEN `app.update()` runs, THEN no panic occurs (plugin registration is valid).

---

## Implementation Notes

*From ADR-006 and ADR-010 Implementation Guidelines:*

**`on_session_ready_init`** — Observer handler:
```rust
fn on_session_ready_init(
    _trigger: Trigger<SessionReady>,
    catalog:  Res<CardCatalog>,
    config:   Res<GameConfig>,
    session:  Res<SessionConfig>,
    mut pools: ResMut<PlayerPools>,
) {
    for player_id in session.team_map.keys() {
        let pool = PlayerPool::initialize(&catalog, &config);
        pools.pools.insert(*player_id, pool);
    }
}
```
Registration: `app.observe(on_session_ready_init)` in `CardPoolPlugin::build()`.

**`on_shop_refresh_needed`** — `MessageReader` subscriber:
```rust
fn on_shop_refresh_needed(
    mut reader:   MessageReader<ShopRefreshNeeded>,
    round_state:  Res<RoundState>,
    mut pools:    ResMut<PlayerPools>,
    catalog:      Res<CardCatalog>,
    family_index: Res<FamilyIndex>,
    mut rng:      ResMut<ServerRng>,
    config:       Res<GameConfig>,
    mut shop_slots:     ResMut<ShopSlots>,
    mut draft_offering: ResMut<InitialDraftOffering>,
    mut refresh_count:  ResMut<ManualRefreshCount>,
) {
    for msg in reader.read() {
        let slot_count = if round_state.phase == RoundPhase::DraftInitial { 9 } else { 3 };
        let player_id = msg.player;

        let Some(pool) = pools.pools.get_mut(&player_id) else {
            tracing::warn!(?player_id, "ShopRefreshNeeded for unknown player — skipping");
            continue;
        };

        let cards = refresh_shop(pool, &catalog, &family_index, &mut rng, &config, slot_count);

        if round_state.phase == RoundPhase::DraftInitial {
            draft_offering.0.insert(player_id, cards);
        } else {
            shop_slots.0.insert(player_id, cards);
        }

        // Reset manual refresh counter on automatic DRAFT refresh
        refresh_count.0.insert(player_id, 0);
    }
}
```

**Scheduling** in `CardPoolPlugin::build()`:
```rust
app.add_systems(Update, on_shop_refresh_needed.after(advance_phase));
```

**`CardPoolPlugin`** must register:
- `app.init_resource::<PlayerPools>()`
- `app.init_resource::<ShopSlots>()`
- `app.init_resource::<InitialDraftOffering>()`
- `app.init_resource::<ManualRefreshCount>()`
- `app.add_message::<ShopRefreshNeeded>()` — only if RSM plugin hasn't registered it already (coordinate with RSM epic)
- `app.observe(on_session_ready_init)`

**Message lifetime**: Bevy buffered messages exist for 2 frames. The `on_shop_refresh_needed` system must run every frame in `Update`. Missing a `ShopRefreshNeeded` message is a silent shop-fail bug — do NOT gate this system on a condition that might skip frames.

**Soft-error on missing player**: If `ShopRefreshNeeded.player` has no `PlayerPool` entry (pool not yet initialized), log a warning and continue — do NOT panic. This handles the edge case where a message arrives before `SessionReady` fires.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 003]: `refresh_shop()` function — must be DONE first
- [Story 005]: `on_manual_refresh` system and `ManualRefreshCount` escalation logic
- [Story 006]: Network dispatch — sending `S2CShopSlots` / `S2CDraftOffering` after population
- Epic 1 (RSM): `ShopRefreshNeeded` message type definition and emission from `advance_phase` — coordinate: this story registers the subscriber only; if the type isn't defined yet, stub it locally and coordinate with RSM story

---

## QA Test Cases

*Written by QA Lead at story creation. Implement against these — do not invent new test cases.*

- **AC-1** — `test_session_ready_initializes_player_pools`
  - Given: Bevy `World` with `Res<SessionConfig>` (2 players: A, B), `Res<CardCatalog>` (fixture with 5 cards), `Res<GameConfig>` (defaults)
  - When: `commands.trigger(SessionReady)` called; `app.update()` runs; `on_session_ready_init` fires
  - Then: `Res<PlayerPools>` inserted; `pools[A].copies_remaining.len() == 5`; `pools[B].copies_remaining.len() == 5`; all copies >= 1
  - Edge cases: Session with 1 player → pool for 1 player created; trigger fired twice → second trigger is a no-op (guard in plugin or idempotent init)

- **AC-2** — `test_per_player_pool_isolation`
  - Given: `PlayerPools` with Player A (5 eligible cards, `copies_remaining=4`) and Player B (same cards); `ShopRefreshNeeded` written for both A and B in same frame
  - When: `on_shop_refresh_needed` processes both messages
  - Then: `ShopSlots[A]` and `ShopSlots[B]` both populated; `PlayerPool[A].copies_remaining` decreased by A's draw count; `PlayerPool[B].copies_remaining` decreased by B's draw count independently — no cross-contamination
  - Edge cases: Same seed for A and B → they may draw the same card type but from their own independent pools

- **AC-3** — `test_draft_initial_writes_to_offering_not_shop`
  - Given: `Res<RoundState>.phase == RoundPhase::DraftInitial`; `ShopRefreshNeeded { player: P }` written; `PlayerPool[P]` has >= 9 eligible cards
  - When: `on_shop_refresh_needed` processes message
  - Then: `InitialDraftOffering[P]` populated with up to 9 entries; `ShopSlots[P]` is either not present or unchanged from prior state
  - Edge cases: Partial initial offering (< 9 eligible) → `InitialDraftOffering[P].len() < 9`; no panic

- **AC-4** — `test_draft_shop_writes_to_shop_slots_not_offering`
  - Given: `Res<RoundState>.phase == RoundPhase::DraftShop`; `ShopRefreshNeeded { player: P }` written; `PlayerPool[P]` has >= 3 eligible cards
  - When: `on_shop_refresh_needed` processes message
  - Then: `ShopSlots[P]` populated with up to 3 entries; `InitialDraftOffering[P]` is not modified
  - Edge cases: Pool has 1 eligible card → `ShopSlots[P].len() == 1`; no panic

- **AC-5** — `test_plugin_registers_cleanly`
  - Given: `App::new()` with `CardPoolPlugin` added; `Res<CardCatalog>` (empty fixture) and `Res<GameConfig>` inserted before build
  - When: `app.update()` called once
  - Then: No panic; `Res<PlayerPools>` exists in world (initialized to empty); `Res<ShopSlots>` exists; `Res<ManualRefreshCount>` exists
  - Edge cases: Plugin added twice → should not double-register resources (use `init_resource` not `insert_resource`)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/pool/session_ready_test.rs` — must exist and all tests must pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (refresh_shop) must be **DONE**
- Depends on: Epic 1 (RSM) — `ShopRefreshNeeded` message type and `advance_phase` scheduling — must be **DONE** or stubbed locally for tests
- Depends on: Epic 2 (Game Session System) — `SessionReady` Observer event and `SessionConfig` — must define `SessionReady` (coordinate: use the same type, don't define a local one)
- Depends on: Foundation `server-rng` epic — `Res<ServerRng>` must be available
- Unlocks: Story 005 (Manual Refresh — depends on `ShopSlots` and `ManualRefreshCount` being managed); Story 006 (Network Dispatch — reads `ShopSlots` / `InitialDraftOffering`)
