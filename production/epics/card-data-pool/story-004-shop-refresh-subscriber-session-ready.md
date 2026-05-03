# Story 004: PlayerPools Draft Entry Init + ShopRefreshTriggered Handoff

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Readiness Refresh

2026-05-03: Revalidated against control manifest version 2026-05-01.
This sprint-status path is canonical for Card Data & Pool Story 004. The
duplicate `story-004-session-ready-observer-shop-subscriber.md` file is stale
and is not the implementation source for this story.

Current architecture supersedes the original `ShopRefreshNeeded` wording:

- `SessionReady` has exactly one observer, the RSM `on_session_ready` handler.
  Card Pool must not register a second `SessionReady` observer.
- RSM emits `DraftStarted` and per-player `ShopRefreshTriggered` messages on
  draft entry.
- Card Pool initializes `PlayerPools` from `DraftStarted { phase: Initial }`
  after `advance_phase` and before Card Acquisition consumes
  `ShopRefreshTriggered` in the same frame.
- Card Acquisition owns draft offerings, shop slots, deduplication, network
  dispatch, and manual refresh cost escalation. Card Pool owns only the
  per-player pool resource lifecycle and pool API.

QL-STORY-READY skipped - Lean mode.

---

## Context

**GDD**: `design/gdd/card-data-pool.md`

**Requirements**:
- `TR-CDP-007`: `CardCatalog` is immutable for server lifetime; `PlayerPool`
  is session-scoped per player in `PlayerPools`.
- `TR-CDP-004`: Pool API supplies `draw_initial_draft`, shop draw helpers,
  `is_available`, and `distribute` for downstream systems.
- `TR-CA-002` and `TR-CA-003`: Card Acquisition consumes
  `ShopRefreshTriggered` to build draft offerings and shop slots from the pool.

**ADRs Governing Implementation**:
- [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md) -
  `PlayerPools: HashMap<PlayerId, PlayerPool>` is per-session mutable state.
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md) -
  `DraftStarted` and `ShopRefreshTriggered` are Bevy buffered Messages.
- [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md) -
  only the RSM observes `SessionReady`; downstream systems react to
  `DraftStarted`.
- [ADR-015: Card Acquisition Shop State Machine Architecture](../../../docs/architecture/adr-015-card-acquisition-shop-state.md) -
  Card Acquisition is the `ShopRefreshTriggered` consumer and owns shop state.

**ADR Decision Summary**: `CardPoolPlugin` registers default-empty pool
resources plus a `DraftStarted` subscriber. On `DraftPhase::Initial`, the
subscriber rebuilds `PlayerPools` from `Res<SessionConfig>`,
`Res<CardCatalog>`, and `Res<GameConfig>`. The subscriber runs every frame in
`Update`, scheduled after `advance_phase`. Card Acquisition's tick set is
scheduled after the Card Pool lifecycle set so the same-frame
`ShopRefreshTriggered` branch sees initialized pools.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM

**Engine Notes**:
- Use `#[derive(Message)]` messages with `MessageReader<T>` and
  `MessageReader::read()`. Do not use `EventReader`.
- Use `DraftPhase::Initial` from `shared::protocol` to select the one-time
  pool initialization path.
- No `SessionReady` observer is added in this story. The only valid observer
  remains RSM `on_session_ready`.
- `liv-bevy-018` is mandatory for all Bevy `.rs` changes.

**Control Manifest Rules (Core layer)**:
- Required: `SessionReady` is observed exactly once by RSM; other systems react
  to `DraftStarted`.
- Required: all RSM subscribers are scheduled after `advance_phase`.
- Required: Core modules do not import Feature modules. If Card Acquisition
  needs to run after Card Pool lifecycle, the Feature plugin depends on an
  exported Core pool system set.
- Forbidden: `EventReader<SessionReady>`, `MessageReader<SessionReady>`, and
  `app.add_message::<SessionReady>()`.
- Forbidden: old `ShopRefreshNeeded` type usage in new code.

---

## Acceptance Criteria

- [ ] **AC-1**: GIVEN `DraftStarted { phase: DraftPhase::Initial }`,
  `Res<SessionConfig>` with 2 player IDs, `Res<CardCatalog>` fixture, and
  `Res<GameConfig>`, WHEN the Card Pool draft-entry subscriber runs, THEN
  `Res<PlayerPools>` contains entries for both players, each
  `copies_remaining.len() == catalog.cards.len()`, and every copy count is
  `>= 1`.
- [ ] **AC-2**: GIVEN an existing non-empty `PlayerPools` resource from a prior
  session, WHEN the next `DraftStarted { phase: DraftPhase::Initial }` is
  processed, THEN the old pool map is cleared before inserting the new
  session's players.
- [ ] **AC-3**: GIVEN `DraftStarted { phase: DraftPhase::Auction }` or
  `DraftStarted { phase: DraftPhase::Shop }`, WHEN the Card Pool subscriber
  runs, THEN `PlayerPools` is unchanged. Later draft entries do not rebuild
  per-player pools.
- [ ] **AC-4**: GIVEN a minimal app with RSM, Card Pool, and Card Acquisition
  plugins, WHEN `DraftStarted { phase: Initial }` and
  `ShopRefreshTriggered { trigger: DraftInitial }` are written in the same
  frame, THEN Card Pool initialization runs before Card Acquisition tick
  consumes the refresh trigger.
- [ ] **AC-5**: GIVEN `GameOverEmitted` is written after a session, WHEN the
  Card Pool teardown subscriber runs, THEN `PlayerPools`, `ShopSlots`,
  `InitialDraftOffering`, and `ManualRefreshCount` are empty.
- [ ] **AC-6**: GIVEN `CardPoolPlugin` is added to `App::new()` with minimal
  Bevy plugins, WHEN the app updates once, THEN registration is valid and no
  second `SessionReady` observer or `ShopRefreshNeeded` message is registered.

---

## Implementation Notes

**New system module**: `server/src/core/pool/system.rs`

```rust
pub fn initialize_player_pools_on_draft_started(
    mut draft_started: MessageReader<DraftStarted>,
    session: Option<Res<SessionConfig>>,
    catalog: Option<Res<CardCatalog>>,
    config: Option<Res<GameConfig>>,
    mut pools: ResMut<PlayerPools>,
)
```

The system should ignore non-initial draft phases and missing resources without
panicking. On the first `DraftPhase::Initial` message, clear `pools.pools`, sort
`session.players()` by `PlayerId`, and insert a freshly initialized
`PlayerPool` for each player.

**Scheduling**:

- Export a core-owned `CardPoolSet::Lifecycle`.
- In `CardPoolPlugin`, configure `CardPoolSet::Lifecycle.after(advance_phase)`.
- Put `initialize_player_pools_on_draft_started` and the teardown subscriber in
  `CardPoolSet::Lifecycle`.
- In `CardAcquisitionPlugin`, schedule `CardAcquisitionSet::Tick` after
  `CardPoolSet::Lifecycle`. This keeps the dependency direction legal because
  Feature may import Core, but Core must not import Feature.

**Teardown**:

Use a `MessageReader<GameOverEmitted>` subscriber that calls `.clear()` on the
four Card Pool resources. Do not remove and reinsert resources.

---

## Out of Scope

- Manual refresh cost escalation and rejected refresh handling. That is Card
  Acquisition Story 004 and sprint Card Pool Story 005.
- Drawing draft offerings or shop slots from `ShopRefreshTriggered`. Current
  architecture assigns that to Card Acquisition.
- Network dispatch for `S2CDraftOffering` or `S2CShopSlots`.
- Adding a `SessionReady` observer in Card Pool.
- Editing duplicate stale Card Data & Pool Story 004 files.

---

## QA Test Cases

- **AC-1 / AC-2** - `test_draft_initial_initializes_player_pools`
  - Given: app with `CardPoolPlugin`, `SessionConfig` for players 1 and 2,
    catalog fixture, config fixture, and stale pool data.
  - When: write `DraftStarted { phase: DraftPhase::Initial }` and update.
  - Then: only players 1 and 2 exist in `PlayerPools`; each pool mirrors the
    catalog length with positive copy counts.
- **AC-3** - `test_non_initial_draft_does_not_reinitialize_pools`
  - Given: prefilled `PlayerPools`.
  - When: write `DraftStarted { phase: DraftPhase::Shop }` and update.
  - Then: the pool map is unchanged.
- **AC-4** - `test_pool_init_precedes_card_acquisition_refresh`
  - Given: app with `CardPoolPlugin` and `CardAcquisitionPlugin`.
  - When: write initial `DraftStarted` and `ShopRefreshTriggered` in one frame.
  - Then: the draft offering path can read initialized `PlayerPools`.
- **AC-5** - `test_game_over_clears_pool_session_resources`
  - Given: all four Card Pool resources contain data.
  - When: write `GameOverEmitted`.
  - Then: all four resources are empty.
- **AC-6** - `test_card_pool_plugin_registers_cleanly`
  - Given: `App::new()` with `MinimalPlugins` and `CardPoolPlugin`.
  - When: update once.
  - Then: plugin registration succeeds and `Messages<DraftStarted>` /
    `Messages<GameOverEmitted>` are available through RSM registration in
    integration tests.

---

## Test Evidence

**Story Type**: Integration

**Required evidence**:
- `tests/integration/pool/session_ready_test.rs`
- Cargo test target: `cargo test -p server --test pool_session_ready_test`

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Card Data & Pool Story 003 is complete; pool API and
  `refresh_shop` helpers exist.
- Depends on: Game Session System Story 004 is complete; `SessionReady`,
  `SessionConfig`, and RSM `on_session_ready` are implemented.
- Depends on: Round State Machine Story 003 is complete; `DraftStarted`,
  `ShopRefreshTriggered`, and `advance_phase` exist.
- Depends on: Card Acquisition Stories 002 and 003 are complete; the consumer
  of `ShopRefreshTriggered` exists and needs initialized `PlayerPools`.
- Unlocks: Card Pool Story 005 and later network/reconnect polish that assumes
  pool resources are session-scoped and cleared on game over.

---

## Performance Budget

No steady-state gameplay cost is expected. Initialization runs once per session
and is O(players * catalog size). The per-frame subscribers only drain small
message buffers and return immediately when no relevant message exists, staying
within the server steady-state budget of 5 ms.
