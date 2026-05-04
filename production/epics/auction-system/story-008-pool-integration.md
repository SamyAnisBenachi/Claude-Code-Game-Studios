# Story 008: Pool Integration — draw_auction_card, distribute, Legendary Stratification

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: At SELECTING entry, Auction uses the current read-only `PlayerPool::draw_auction_card(auction_pool, catalog, seed)` API to select an auction card, then Auction calls `distribute(card_id)` explicitly and unconditionally at draw time on a successful selection. The card is permanently consumed from the shared neutral auction pool regardless of auction outcome. Pool-empty guard fires `AuctionSettled { winner: None }` immediately without sending `S2CAuctionCard`. Legendary cards are excluded by Auction-side round eligibility before `GameConfig.legendary_pool_entry_round`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- This story uses `App::new()` with both `AuctionPlugin` and `CardDataPoolPlugin` registered — the full integration test harness
- Current Card Pool API: `PlayerPool::draw_auction_card(auction_pool, catalog, seed) -> Option<CardId>` is read-only and does **not** accept eligible rarities. It filters only Neutral Rare/Legendary cards with `copies_remaining > 0`
- Auction owns round eligibility and Legendary stratification: before the configured entry round, Auction must present a Rare-only auction-pool view or equivalent bounded filter to the draw path, then call `distribute(card_id)` only after a successful eligible selection
- Pool draw functions return `Option<T>` and never panic — pool-empty returns `None`, Auction System handles it (AU15)
- `liv-bevy-018` skill mandatory; `liv-bevy-lightyear` for any networking code added during integration

**Control Manifest Rules (Feature Layer)**:
- Required: `distribute()` called by Auction unconditionally at draw time after a successful `draw_auction_card()` result, NOT conditionally on auction outcome — card is consumed from pool at draw, not at settlement
- Required: `auction_tick_system` remains the sole `AuctionState` writer and keeps ADR-013 code order: handle `AuctionPhaseEntered`, handle `AbortAuction`, drain bids, decrement timer, settle expired auction
- Forbidden: Do NOT return or distribute a Legendary card when `round_number < legendary_pool_entry_round`; Auction owns this stratification because the current Card Pool API has no eligible-rarity parameter
- Note: `docs/architecture/control-manifest.md` version 2026-05-01 includes ADR-013 auction state, scheduling, and Lightyear receiver rules

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [ ] **AU8-pool**: `GIVEN` `draw_auction_card()` was called at SELECTING entry, `WHEN` the auction ends by any path (win, no-bid, or AbortAuction from LIVE_BIDDING), `THEN` the drawn card's `copies_remaining` in the shared neutral pool has been decremented by exactly 1 from its pre-draw value. Test parameterized over all three exit paths
- [ ] **AU15**: `GIVEN` the shared neutral auction pool has `copies_remaining == 0` across all eligible rarities, `WHEN` `AuctionPhaseEntered` is received, `THEN` `AuctionSettled { winner: None, amount: 0 }` Bevy Message fires in the same system invocation (not deferred), `auction_state.phase == IDLE`, and no `S2CAuctionCard` is queued
- [ ] **AU21**: `GIVEN` `round_number < GameConfig.legendary_pool_entry_round` (e.g., round 3 with `legendary_pool_entry_round = 6`), `WHEN` `draw_auction_card()` is called, `THEN` the drawn card's rarity is NOT Legendary — even if Legendary cards are present in the pool

---

## Implementation Notes

*Derived from ADR-013 and GDD Rule 2:*

```rust
// Step 1 of auction_tick_system (within AuctionPhaseEntered handler):
auction.phase = AuctionPhase::Selecting;

// Build an eligible auction-pool view based on round_number.
// Current PlayerPool::draw_auction_card() has no eligible-rarity parameter.
// Before legendary_pool_entry_round, Auction must exclude Legendary cards
// before selecting and before calling distribute().
let eligible_auction_pool =
    build_eligible_auction_pool(&auction_pool, &catalog, round, &game_config);

let draw_result = PlayerPool::draw_auction_card(&eligible_auction_pool, &catalog, rng_seed);

match draw_result {
    None => {
        // Pool empty for eligible rarities — fire immediate AuctionSettled{None}
        settled_writer.write(AuctionSettled { winner: None, final_price: 0, card_id: CardId(0) });
        auction.phase = AuctionPhase::Idle;
        return;  // Skip LIVE_BIDDING setup
    }
    Some(card_id) => {
        // Auction owns consumption. distribute() is called immediately at draw time.
        // The card is permanently consumed from the pool regardless of outcome.
        if auction_pool.distribute(card_id).is_err() {
            // Treat an unexpected distribute failure as pool-empty for this story's tests.
            settled_writer.write(AuctionSettled { winner: None, final_price: 0, card_id: CardId(0) });
            auction.phase = AuctionPhase::Idle;
            return;
        }
        let rarity = catalog
            .get(&card_id)
            .expect("draw_auction_card returned a card missing from CardCatalog")
            .rarity;
        let starting_price = match rarity { ... };
        auction.card_id = Some(card_id);
        auction.current_price = starting_price;
        auction.timer_remaining_ms = game_config.auction_timer_seconds * 1000;
        auction.current_leader = None;
        auction.phase = AuctionPhase::LiveBidding;
        // broadcast S2CAuctionCard
    }
}
```

**`distribute()` responsibility**: Current code makes `PlayerPool::draw_auction_card(...)` read-only. Auction System must call `distribute(card_id)` explicitly and immediately after a successful eligible draw. This is the only consumption point for AU8-pool; win, no-bid, and AbortAuction paths must not add a second decrement or return the card.

**Pool-empty with only ineligible rarities**: If the pool has Legendary copies but `round_number < legendary_pool_entry_round`, the Legendary cards are ineligible. The eligible auction-pool view has no drawable cards, `draw_auction_card()` returns `None`, and AU15 fires. This is tested in AU21's edge case.

**AU8-pool parameterization**: Rust does not have built-in parameterized tests — write three separate test functions sharing a `fn auction_fixture() -> App` builder that sets up AuctionPlugin + CardDataPoolPlugin + EconomyPlugin with a Rare card having `copies_remaining = 3`.

**Performance / No-Impact Note**:
- Pool integration must stay in the bounded SELECTING branch of `auction_tick_system`: at most one bounded eligibility pass over the auction-pool/catalog data, one `draw_auction_card()` call, and one `distribute()` call on success
- Do not add retry-until-success loops, background/deferred pool work, asset scans, network fan-out, or other expensive work inside `auction_tick_system`
- Expected cost remains within the server steady-state budget from the control manifest (`<= 5 ms` per frame tick); empty-pool and ineligible-Legendary paths must return immediately to IDLE after writing the required `AuctionSettled` message

---

## Out of Scope

- `draw_auction_card()` implementation — owned by `card-data-pool` epic
- Pool weighted draw for shop cards — `card-data-pool` epic
- Auction UI rendering — `shop-auction-ui` epic (Presentation layer, M2)

---

## QA Test Cases

*Written by qa-lead at story creation. All tests use `App::new()` with AuctionPlugin + CardDataPoolPlugin + EconomyPlugin registered.*

**AC AU8-pool Path 1 — copies_remaining decremented after win:**
```
Test: pool distribution is permanent after auction win
  Given: App::new() with AuctionPlugin + CardDataPoolPlugin + EconomyPlugin
         Card pool fixture: CardId(10) Rare, copies_remaining = 3 (pre-draw)
         AuctionPhaseEntered { round: 3 } injected → LIVE_BIDDING
         Player 1 places winning bid; set timer_remaining_ms = 0; resolution fires
  When: Full auction cycle completes (win path)
  Then: copies_remaining for CardId(10) == 2
        (decremented at draw time via distribute(), not at settlement — win doesn't add a second decrement)
```

**AC AU8-pool Path 2 — copies_remaining decremented after no-bid expiry:**
```
Test: pool distribution is permanent after no-bid outcome
  Given: Same pool fixture; auction enters LIVE_BIDDING; no bids placed
         Set timer_remaining_ms = 0; Case B resolution fires
  When: Full cycle completes (no-bid path)
  Then: copies_remaining for CardId(10) == 2 (decremented at draw time)
        S2CAuctionSettled { winner: None, amount: 0 } broadcast
        Card is NOT returned to pool (permanent depletion regardless of bid outcome)
```

**AC AU8-pool Path 3 — copies_remaining decremented after AbortAuction:**
```
Test: pool distribution is permanent even when auction aborted
  Given: Same pool fixture; auction enters LIVE_BIDDING
         AbortAuction injected before timer expires
  When: auction_tick_system handles AbortAuction
  Then: copies_remaining for CardId(10) == 2 (decremented at draw time)
        AuctionSettled Bevy Message NOT fired (AbortAuction is silent)
```

**AC AU15 — Pool empty fires immediate AuctionSettled{None}:**
```
Test: empty pool triggers immediate no-card outcome
  Given: App::new() with AuctionPlugin + CardDataPoolPlugin
         Card pool fixture: all currently eligible auction cards have copies_remaining == 0
         (before legendary_pool_entry_round, Rare is the only eligible auction rarity)
         AuctionPhaseEntered injected
  When: auction_tick_system runs
  Then: AuctionSettled { winner: None, amount: 0 } Bevy Message fires in same invocation
        auction_state.phase == AuctionPhase::Idle
        No S2CAuctionCard queued
        Messages<AuctionSettled> contains exactly one event (not deferred to next frame)
  Edge cases: Pool has copies_remaining = 0 for Rare but has Legendary copies AND
              round_number < legendary_pool_entry_round → Legendary is ineligible →
              treated as pool empty → same AU15 outcome
```

**AC AU21 — Legendary stratification enforced before entry round:**
```
Test: round < threshold draws only non-Legendary cards
  Given: App::new() with AuctionPlugin + CardDataPoolPlugin + GameConfig
         GameConfig { legendary_pool_entry_round: 6 }
         Card pool fixture: Rare cards (copies_remaining >= 3)
                            AND Legendary cards (copies_remaining >= 2)
         AuctionPhaseEntered { round: 3 } injected (round 3 < threshold 6)
  When: auction_tick_system handles AuctionPhaseEntered (SELECTING entry)
  Then: AuctionState.card_id is set to a Rare card (NOT a Legendary card)
        Legendary cards' copies_remaining are unchanged (not drawn)
  Edge cases: round = 5 (threshold - 1) → still non-Legendary
              round = 6 (= threshold) → Legendary is now eligible (can be drawn)
              round = 7 (threshold + 1) → Legendary eligible
              Pool has ONLY Legendary + round < threshold → AU15 path fires
              (pool is empty for eligible rarities)
```

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/auction/pool_integration_test.rs` — must exist and pass (uses `App::new()` with AuctionPlugin + CardDataPoolPlugin + EconomyPlugin)

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 007 DONE (`AuctionPlugin` fully registered — required for `App::new()` integration tests)
- Depends on: `card-data-pool` story-001 DONE (provides `PlayerPool` and `distribute()`)
- Depends on: `card-data-pool` story-002 DONE (provides current read-only `PlayerPool::draw_auction_card(auction_pool, catalog, seed)`)
- Unlocks: Epic complete — all 10 TR-AUC requirements covered
