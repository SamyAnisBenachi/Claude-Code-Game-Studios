# Story 008: Pool Integration — draw_auction_card, distribute, Legendary Stratification

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: At SELECTING entry, `draw_auction_card()` is called and `distribute()` is called unconditionally at draw time — the card is permanently consumed from the shared neutral pool regardless of auction outcome. Pool-empty guard fires `AuctionSettled { winner: None }` immediately without sending `S2CAuctionCard`. Legendary cards are filtered out if `round_number < GameConfig.legendary_pool_entry_round`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- This story uses `App::new()` with both `AuctionPlugin` and `CardDataPoolPlugin` registered — the full integration test harness
- `draw_auction_card()` accepts eligible rarities derived from `round_number` vs `legendary_pool_entry_round` — the filter lives inside the pool API call, not in the Auction System directly
- Pool draw functions return `Option<T>` and never panic — pool-empty returns `None`, Auction System handles it (AU15)
- `liv-bevy-018` skill mandatory; `liv-bevy-lightyear` for any networking code added during integration

**Control Manifest Rules (Feature Layer)**:
- Required: `distribute()` called unconditionally at draw time, NOT conditionally on auction outcome — card is consumed from pool at draw, not at settlement
- Required: `draw_auction_card()` must receive the eligible rarity set as a parameter (based on `round_number`) — pool owns NO round-number awareness; filter is passed in by caller
- Forbidden: Do NOT return a Legendary card when `round_number < legendary_pool_entry_round` — this is enforced by the `eligible_rarities` parameter passed to `draw_auction_card()`
- Note: ADR-013 is Accepted but not yet incorporated in the control manifest (v2026-04-30 lists it as pending)

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

// Build eligible rarity set based on round_number
let eligible_rarities = if round >= game_config.legendary_pool_entry_round {
    vec![Rarity::Rare, Rarity::Epic, Rarity::Legendary]
} else {
    vec![Rarity::Rare, Rarity::Epic]  // No Legendary before entry round
};

let draw_result = card_pool.draw_auction_card(&eligible_rarities, rng_seed);

match draw_result {
    None => {
        // Pool empty for eligible rarities — fire immediate AuctionSettled{None}
        settled_writer.write(AuctionSettled { winner: None, final_price: 0, card_id: CardId(0) });
        auction.phase = AuctionPhase::Idle;
        return;  // Skip LIVE_BIDDING setup
    }
    Some((card_id, rarity)) => {
        // distribute() was called by draw_auction_card() unconditionally at draw time
        // The card is permanently consumed from the pool
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

**`distribute()` responsibility**: The `card-data-pool` epic defines whether `distribute()` is called inside `draw_auction_card()` or by the caller. Confirm with `card-data-pool` story-001 — the GDD says "calls `distribute(card_id)` immediately at draw time." If `distribute()` is the caller's responsibility, the Auction System must call it explicitly after a successful draw.

**Pool-empty with only ineligible rarities**: If the pool has Legendary copies but `round_number < legendary_pool_entry_round`, the Legendary cards are ineligible. The pool draw returns `None` for the eligible rarities — AU15 path fires. This is tested in AU21's edge case.

**AU8-pool parameterization**: Rust does not have built-in parameterized tests — write three separate test functions sharing a `fn auction_fixture() -> App` builder that sets up AuctionPlugin + CardDataPoolPlugin + EconomyPlugin with a Rare card having `copies_remaining = 3`.

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
         Card pool fixture: ALL eligible rarities (Rare, Epic) have copies_remaining == 0
         AuctionPhaseEntered injected
  When: auction_tick_system runs
  Then: AuctionSettled { winner: None, amount: 0 } Bevy Message fires in same invocation
        auction_state.phase == AuctionPhase::Idle
        No S2CAuctionCard queued
        Messages<AuctionSettled> contains exactly one event (not deferred to next frame)
  Edge cases: Pool has copies_remaining = 0 for Rare/Epic but has Legendary copies AND
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
- Depends on: `card-data-pool` story-001 DONE (provides `draw_auction_card()`, `distribute()`, and `SharedNeutralPool` resource)
- Unlocks: Epic complete — all 10 TR-AUC requirements covered
