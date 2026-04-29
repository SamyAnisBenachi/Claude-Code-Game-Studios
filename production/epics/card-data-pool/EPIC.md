# Epic: Card Data & Pool

> **Layer**: Core
> **GDD**: design/gdd/card-data-pool.md
> **Architecture Module**: `server/core/pool/` (full module — `state.rs`, `api.rs`, `system.rs`, `plugin.rs`); the `CardCatalog` lookup type is consumed from `shared/` (Foundation epic)
> **Status**: Ready
> **Stories**: To be created — see Story Breakdown Hint below

## Overview

Implements the session-scoped player card pools and the weighted-draw shop refresh that drives every DRAFT entry. The immutable `CardCatalog` (loaded by the Foundation epic) holds card definitions for the lifetime of the server. This epic owns `PlayerPool { copies_remaining: HashMap<CardId, u32> }` per player (mutable, session-scoped, initialised on `SessionReady`), the `distribute()` mutation as the sole authority on `copies_remaining` (never below 0), the rarity- and class-aware weighted draw (Formula 2), the `ShopSlots` per-player resource that holds the current 3-card shop, and the manual-refresh cost-escalation counter that resets on each DRAFT phase. The system subscribes to `ShopRefreshNeeded` from Epic 1's RSM event bus (one event per player per DRAFT entry) — Card Pool draws three cards per player, sends `S2CShopSlots` unicast on `ReliableChannel`, and for DRAFT_INITIAL specifically populates the 9-card initial offering. All randomness flows through `Res<ServerRng>` (Foundation `server-rng` epic) — Card Pool owns no RNG source. Returns `Option<CardId>` from every draw function (never panics on empty pool — the GDD edge case "no eligible cards remain" must surface gracefully to the caller).

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-006: Card Data Schema and Pool State Architecture | `CardCatalog` immutable + lifetime-of-server; `PlayerPool` mutable + session-scoped per player; `distribute()` is sole pool mutation; all draws return `Option<T>` (never panic); `total_acquired(id)` derived from `initial_count - copies_remaining`; Epic/Legendary copy counts are compile-time consts (1 each) | LOW (data model) |
| ADR-010: RSM Phase Event Bus | Card Pool is the canonical `ShopRefreshNeeded { player }` subscriber; F2 emission ordering guarantees Card Pool runs after Economy (so post-income gold is visible if any future weight depends on gold) | MEDIUM (post-cutoff Bevy event API) |

## Engine Risk: MEDIUM

The data model itself is LOW risk (plain `HashMap`, `serde`, no Bevy-version coupling). The MEDIUM-risk surface area:

1. **`bevy_asset_loader` 0.18 compatibility** — `CardCatalog` loading happens at server startup (Foundation epic territory) but if a `CardCatalogAsset` wrapper is used here for shop-side type validation, `#[derive(Asset, TypePath)]` is required as of Bevy 0.18. Verify against pinned `bevy_asset_loader` version.
2. **`MessageReader::read()` — `EventReader` no longer exists in Bevy 0.17+.** Card Pool subscribes to `ShopRefreshNeeded` (a `#[derive(Message)]` type) via `MessageReader<ShopRefreshNeeded>`. `liv-bevy-018` enforces the correct API.
3. **Per-player fan-out** — `ShopRefreshNeeded` is written once per player per DRAFT entry. The subscriber must handle N messages per frame (2 for 1v1, up to 6 for 3v3). Bevy's two-frame message lifetime means the subscriber must run every frame; missing a message is a silent shop-fail bug. Subscriber must be `.after(advance_phase)`.

`liv-bevy-018` skill is mandatory on every `.rs` file. `liv-bevy-lightyear` is mandatory wherever `S2CShopSlots` and `S2CDraftOffering` send code lives (the network dispatch system).

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from the ADR "GDD Requirements Addressed" sections.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-CDP-01 | Card data types defined in shared crate; consumed identically by server and client | ADR-006 ✅ (consumed from `shared/src/card.rs`) |
| TR-CDP-02 | `CardCatalog` immutable; loaded once at startup; never mutated mid-session | ADR-006 ✅ |
| TR-CDP-03 | Card definition schema with all required fields | Foundation `workspace-and-shared-types` epic ✅ |
| TR-CDP-04 | `PlayerPool` per-player; session-scoped; initialised at `SessionReady` from rarity defaults + per-card overrides; cleared at session teardown | ADR-006 ✅ |
| TR-CDP-05 | `distribute()` is sole mutation; `copies_remaining` never below 0 | ADR-006 ✅ |
| TR-CDP-06 | Weighted draw (Formula 2): rarity weights × class match × copies_remaining; weights from `GameConfig` | ADR-006 ✅ |
| TR-CDP-07 | All draw functions return `Option<CardId>` — never panic | ADR-006 ✅ |
| TR-CDP-08 | `total_acquired(id)` is derived; no separate stored field | ADR-006 ✅ |
| TR-CDP-09 | `ShopRefreshNeeded` subscriber draws 3 cards per player; for DRAFT_INITIAL, draws 9-card offering | ADR-010 ✅ |

## Scope

### Deliverables

**`server/src/core/pool/state.rs`**
- `PlayerPool { copies_remaining: HashMap<CardId, u32> }` — `#[derive(Clone, Debug)]`. Held inside `PlayerPools(HashMap<PlayerId, PlayerPool>)` resource.
- `PlayerPools` resource: initialised on `SessionReady` (Epic 2). For each player, populate `copies_remaining` with: rarity defaults from `GameConfig.pool_copies_common/uncommon/rare`, `EPIC_POOL_COPIES = 1`, `LEGENDARY_POOL_COPIES = 1`, then apply any `CardData.pool_copies_override` (per-card). Soft error (warn + use rarity default + continue) if `pool_copies_override <= 0` — NEVER abort startup per ADR-004.
- `ShopSlots(HashMap<PlayerId, Vec<CardId>>)` resource — current 3-card shop per player. Replaced atomically on each `ShopRefreshNeeded`.
- `InitialDraftOffering(HashMap<PlayerId, Vec<CardId>>)` resource — the 9-card DRAFT_INITIAL selection. Cleared at end of DRAFT_INITIAL.
- `ManualRefreshCount(HashMap<PlayerId, u32>)` resource — count of paid manual refreshes this DRAFT phase per player. Reset to 0 at every DRAFT entry. The `n`th refresh in a phase costs `refresh_base_cost + (n - 1)` gold (1g, 2g, 3g, …).
- `PoolError` enum: `CardNotInCatalog | CopiesExhausted | EmptyFilteredPool` — returned where appropriate.
- `PoolFilter` struct: `class: Option<ClassId>, family: Option<FamilyId>, max_rarity: Option<Rarity>` — used by typed-draw functions.

**`server/src/core/pool/api.rs`** (sole-mutation discipline)

```rust
/// SOLE pool mutation function. Removes one copy of `card_id` from `pool`.
/// Returns Err(CopiesExhausted) if `copies_remaining[card_id] == 0`.
/// Never panics. Never decrements below 0.
pub fn distribute(pool: &mut PlayerPool, card_id: CardId) -> Result<(), PoolError>;

/// Weighted draw. Filters first, then weights by rarity × copies_remaining.
/// Returns None on empty filtered pool. Caller decides how to handle empty.
/// Calls `rng.next_seed(RngEvent::DrawShopSlot { ... })` exactly once for
/// the random selection — no internal RNG.
pub fn draw(
    pool: &PlayerPool,
    catalog: &CardCatalog,
    filter: PoolFilter,
    rng: &mut ServerRng,
    config: &GameConfig,
) -> Option<CardId>;

/// Convenience wrappers — all built on `draw`:
pub fn draw_class_card(pool, catalog, class, rng, config) -> Option<CardId>;
pub fn draw_neutral(pool, catalog, rng, config) -> Option<CardId>;
pub fn draw_family_card(pool, catalog, family, rng, config) -> Option<CardId>;

/// Atomic shop refresh: draws N cards (typically 3), distributes each on success.
/// Returns the drawn vector. Caller (system layer) handles the unicast.
pub fn refresh_shop(
    pool: &mut PlayerPool,
    catalog: &CardCatalog,
    rng: &mut ServerRng,
    config: &GameConfig,
    slot_count: usize,           // 3 for normal DRAFT, 9 for DRAFT_INITIAL
) -> Vec<CardId>;

/// On acquisition (shop purchase, auction win, free pick): removes one copy.
pub fn acquire_card(pool: &mut PlayerPool, card_id: CardId)
    -> Result<(), PoolError>;

/// Derived (no separate stored field):
pub fn total_acquired(pool: &PlayerPool, catalog: &CardCatalog, id: CardId) -> u32;
```

**Invariants enforced by the API:**
- `copies_remaining` is mutated ONLY via `distribute()` and `acquire_card()`. Code review gate: any direct write to the `HashMap` outside `pool/api.rs` is a CHANGES REQUIRED.
- All draws are `Option<CardId>` returns — never panic on empty pool.
- The RNG is borrowed `&mut` and used exactly once per draw. Audit trail (Foundation `server-rng` epic) records the consumption.

**`server/src/core/pool/system.rs`**
- `on_session_ready_init` — Observer for `SessionReady` (registered by GSS plugin in Epic 2 — coordinated): initialise `PlayerPools` from `Res<SessionConfig>.team_map.keys()` × `Res<CardCatalog>` × `Res<GameConfig>`. Apply rarity defaults + `pool_copies_override` per card with soft-error logging.
- `on_shop_refresh_needed` — `MessageReader<ShopRefreshNeeded>` subscriber. For each message:
  1. Determine slot count: 9 if `Res<RoundState>.phase == DraftInitial`, 3 otherwise.
  2. Call `refresh_shop(player, ...)` — atomic distribute-and-collect.
  3. Write to `ShopSlots[player]` (or `InitialDraftOffering[player]` for DRAFT_INITIAL).
  4. Reset `ManualRefreshCount[player] = 0`.
  5. Enqueue `S2CShopSlots { slots: Vec<CardId> }` unicast on `ReliableChannel`. For DRAFT_INITIAL, enqueue `S2CDraftOffering { offering: Vec<CardId> }` instead.
  6. If shop draw returns < `slot_count` cards (filtered pool exhausted): fill remaining slots with `None` placeholders in the message; client renders empty slot. Soft fail per ADR-006.
- `on_manual_refresh` — `MessageReader<C2SShopRefresh>` subscriber. Phase-gate (`phase == DraftShop`); compute cost from `ManualRefreshCount`; call Economy's `validate_spend` (`from_reserve_only=false`) + `apply_spend` for the gold; then call `refresh_shop` for 3 slots; increment `ManualRefreshCount`. Send fresh `S2CShopSlots`.
- `on_card_acquired` — `MessageReader<CardAcquired>` (emitted by shop purchase / auction win / objective free-pick): call `acquire_card(pool, card_id)` to remove the copy. Enqueues `S2CHandUpdate` to the owner (defined in shared protocol; this epic only enqueues).

**`server/src/core/pool/plugin.rs`**
- `CardPoolPlugin`: registers `PlayerPools`, `ShopSlots`, `InitialDraftOffering`, `ManualRefreshCount`; subscribes `on_shop_refresh_needed` `.after(advance_phase)` (Epic 1 ordering contract); registers `on_session_ready_init` Observer; teardown subscriber on `GameOverEmitted` to clear all per-session resources.

**Network dispatch wiring**
- A system in `server/src/network/` reads `MessageReader<S2CShopSlots>` and sends unicast on `ReliableChannel`.
- A system reads `MessageReader<S2CDraftOffering>` and sends unicast on `ReliableChannel`.
- Message types defined in `shared/src/protocol.rs` (`workspace-and-shared-types` Foundation epic).

**Tests**
- `tests/unit/pool/` — every Acceptance Criterion from `card-data-pool.md` (CDP-1 through CDP-N — refer to GDD §8).
- `distribute()` behaviour: never below 0; returns Err when called on exhausted card; idempotent for the empty case.
- `draw` filter cases: class match, neutral filter, family filter, max_rarity filter; empty-filtered-pool returns `None` (not panic).
- `refresh_shop` for 3 slots and 9 slots; partial-fill case (pool exhausted to < slot_count).
- `total_acquired` derived correctness: for any `(initial, distributed)` pair, `total_acquired = initial - copies_remaining`.
- `pool_copies_override <= 0` soft-error: warning logged, rarity default used, no panic.
- Manual refresh cost escalation across 3 refreshes in same DRAFT: 1g, 2g, 3g; resets to 1g at next DRAFT entry.
- DRAFT_INITIAL 9-card offering vs DRAFT_SHOP 3-card refresh: both work, `S2CDraftOffering` vs `S2CShopSlots` correctly chosen.
- Per-player isolation: Player A's pool draw does not affect Player B's `copies_remaining`.
- Integration test: `ShopRefreshNeeded { player: A }` + `ShopRefreshNeeded { player: B }` in same frame → two independent shops drawn, both unicast on correct targets.
- RNG audit log integration: each draw produces one `RngEvent::DrawShopSlot` audit entry (verified via `Res<ServerRng>.audit_log`).

### Out of Scope (owned by other epics)

- `CardCatalog` loading from `cards.json`: Foundation epic — `game-config-pipeline` (or sister loading epic). This epic uses `Res<CardCatalog>` read-only.
- `CardData`, `CardId`, `Rarity`, `ClassId`, `FamilyId`, `CardType` type definitions: `workspace-and-shared-types` Foundation epic.
- `GameConfig` field definitions including pool weights, rarity defaults, `interest_threshold_gold`: `game-config-pipeline` Foundation epic.
- `ServerRng` Resource type and `next_seed()` API: `server-rng` Foundation epic.
- `ShopRefreshNeeded` event definition + RSM emission: Epic 1 — Round State Machine.
- `SessionConfig` and `SessionReady`: Epic 2 — Game Session System.
- Economy spend (`validate_spend`, `apply_spend`): Epic 3 — Economy System.
- Auction System weighted-draw on the shared neutral pool: M2 — Auction epic. The `draw` API here is reused; the Auction epic adds an auction-pool variant if needed.
- Hand management (10-card hand cap, hand-full rejection on auction): M2 — Card Acquisition. This epic drives shop and pool; hand state belongs to Card Acquisition.
- `S2CHandUpdate` send wiring (this epic emits on `acquire_card` but the message type is defined in shared and the actual hand state is M2): coordinated.

### Implementation Notes

**RNG consumption order discipline (ADR-005)** — The strict per-event ordering matters because the audit log compares replayed sessions to recorded outputs. For DRAFT_INITIAL, `refresh_shop` is called once per player in ascending `player_id` order; within each player's call, `slot_index` is ascending 0..9 (or 0..3). For DRAFT_SHOP, same pattern. Manual refresh during DRAFT_SHOP uses the same `RngEvent::DrawShopSlot` event type — the audit log distinguishes by timestamp, not event type. The audit log will catch any consumption-order regression in CI replay tests (Foundation `server-rng` epic owns the harness).

**Empty-filtered-pool surface behaviour** — Per ADR-006: never panic. For shop refresh, drawing 3 cards but the filtered pool only has 2: the third slot is `None`. Client renders as an empty slot or "—". The next refresh attempt on a DRAFT entry might draw differently (since pool changes between rounds), so this is recoverable, not fatal. Log a warning at debug level with the filter used.

**`pool_copies_override` semantics** — `CardData.pool_copies_override: Option<i32>` (signed for "use default" sentinel). `Some(n) where n > 0` overrides rarity default. `Some(n) where n <= 0` is a soft error: log warn, use rarity default, continue. `None` uses rarity default. This matches the GDD's "soft error" guarantee and ADR-006's "never abort startup" rule.

**Concurrency: per-player isolation** — `PlayerPools.get_mut(player_id)` returns one pool; the borrow checker prevents simultaneous mutation of two players in the same system. For per-player fan-out (the `ShopRefreshNeeded` subscriber processing N events), iterate sequentially — Bevy's borrow rules prevent parallel mutation anyway. No locks needed.

**Manual refresh counter reset on DRAFT_INITIAL** — DRAFT_INITIAL has no manual refresh (per GDD Rule 15: "Purchase cards (up to starting gold); signal ready" — no refresh action listed). Defensively reset `ManualRefreshCount` to 0 on DRAFT_INITIAL entry too, to ensure round 2's first DRAFT_SHOP starts clean. Phase-gate `on_manual_refresh` to `DraftShop` only; reject in any other phase.

## Definition of Done

- All deliverables above implemented and passing.
- All Acceptance Criteria from `card-data-pool.md` §8 have passing unit tests in `tests/unit/pool/`.
- `cargo check --workspace` green; zero warnings on `server/src/core/pool/**`.
- CI grep gate: direct `copies_remaining` mutation outside `pool/api.rs` returns zero matches:
  `grep -rE "copies_remaining\.\s*(insert|remove|entry)" server/src/ | grep -v "core/pool/api.rs"` returns zero matches.
- CI grep gate: `grep -rE "panic!|unwrap\(\)|expect\(" server/src/core/pool/api.rs` returns zero matches in draw paths (non-draw code may use `.expect("diagnostic")` per coding standards).
- An integration test demonstrates a complete DRAFT_INITIAL → DRAFT_SHOP → manual refresh → next-round DRAFT flow with assertions on `S2CShopSlots` / `S2CDraftOffering` payloads at every step.
- An integration test demonstrates pool exhaustion edge case: configure `copies_remaining` for a Legendary card to 1, acquire it once, attempt next draw with class filter that includes that Legendary — confirm it's no longer in the pool and the draw produces a non-Legendary result without error.
- An integration test confirms RNG audit log integration: 3 shop draws produce exactly 3 `RngEvent::DrawShopSlot` entries in `Res<ServerRng>.audit_log`.
- `CardPoolPlugin` registers cleanly in a headless Bevy `App` startup test with mock `CardCatalog` and `GameConfig`.

## Story Breakdown Hint

Suggested decomposition (final story list to be authored via `/create-stories`):

1. **State + API scaffold** (Config/Data + Logic) — `state.rs`, `api.rs`, `distribute()`, `acquire_card()`, `total_acquired()`; pure-function unit tests including `pool_copies_override <= 0` soft-error path.
2. **Weighted draw** (Logic) — `draw()` with full PoolFilter (class, family, max_rarity); Formula 2 weighting; empty-filtered-pool returns `None`; convenience wrappers; tests for each filter combination.
3. **`refresh_shop` + 3/9 slot variants** (Logic) — atomic distribute-and-collect; partial-fill on exhaustion; tests for both DRAFT_INITIAL (9) and DRAFT_SHOP (3) cases.
4. **`ShopRefreshNeeded` subscriber + `SessionReady` init** (Integration) — Observer for `SessionReady` initialises pools; `MessageReader<ShopRefreshNeeded>` subscriber calls `refresh_shop` and writes `ShopSlots`; per-player fan-out across 1v1 and 2v2.
5. **Manual refresh + cost escalation** (Logic + Integration) — `on_manual_refresh` with `ManualRefreshCount`; phase-gate to DRAFT_SHOP; calls Economy's `validate_spend` + `apply_spend`; counter reset on DRAFT entry; tests EC24/EC25/EC26 from Economy GDD covered jointly.
6. **Network dispatch wiring** (Integration) — `S2CShopSlots` and `S2CDraftOffering` unicast on `ReliableChannel`; `liv-bevy-lightyear` mandatory; integration test asserts correct messages on correct channel with correct targets.

## Next Step

Run `/create-stories production/epics/card-data-pool/EPIC.md` to author the story files. Story 1 (State + API scaffold) can begin in parallel with Epic 1, Epic 2, and Epic 3 Story 1 — they are independent. Story 4 onward is gated on Epic 1's `ShopRefreshNeeded` event and Epic 2's `SessionReady` Observer being defined.
