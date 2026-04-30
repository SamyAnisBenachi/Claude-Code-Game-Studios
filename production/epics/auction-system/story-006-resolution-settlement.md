# Story 006: Resolution & Settlement — Case A/B & Post-Settlement Invariants

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-006`, `TR-AUC-010`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: When `timer_remaining_ms == 0` in LIVE_BIDDING, `auction_tick_system` transitions to RESOLVING synchronously. Case A (winner exists): `spend_reserved_gold`, add card to hand, unicast `S2CCardAcquired`, broadcast `S2CAuctionSettled { winner: Some(...) }`, write `AuctionSettled` Bevy Message, return to IDLE. Case B (no bids): broadcast `S2CAuctionSettled { winner: None, amount: 0 }`, write `AuctionSettled`, return to IDLE.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `AuctionSettled` is a Bevy internal buffered message (`#[derive(Message)]`) written via `MessageWriter<AuctionSettled>`, NOT `#[derive(Event)]` + `EventWriter`
- `S2CCardAcquired` is a reliable unicast to the winner only — confirm `NetworkTarget::Single(winner_peer_id)` variant in Lightyear 0.26 (verification checklist item 7: `NetworkTarget::Single(PeerId)` ✅)
- `liv-bevy-018` + `liv-bevy-lightyear` skills mandatory on `system.rs`
- **Pre-implementation gate (Epic OQ4)**: Verify exact function name `spend_reserved_gold` in `economy/api.rs` before implementation — may differ from GDD description
- **Pre-implementation gate (Epic gate 1)**: Confirm `AcquisitionSource::AuctionWon` vs `CardSource::AuctionWon` enum name in `S2CCardAcquired` source field (from `network-protocol.md`)

**Control Manifest Rules (Feature Layer)**:
- Required: `spend_reserved_gold` must be followed by `debug_assert!(player.gold >= player.reserved_gold)` before subtraction; in release builds add an explicit `if player.gold < player.reserved_gold { log_critical_error!(...) }` guard
- Required: `AuctionSettled` Bevy Message must be written in BOTH Case A and Case B paths — the RSM reads it to transition to DRAFT_SHOP
- Required: RESOLVING must complete within the same server tick as LIVE_BIDDING exit — no deferred resolution
- Forbidden: Do NOT call `spend_reserved_gold` in Case B (no bids) — there is no reservation to spend
- Note: ADR-013 is Accepted but not yet incorporated in the control manifest (v2026-04-30 lists it as pending)

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [ ] **AU7-a**: `GIVEN` `timer_remaining_ms == 0` with `current_leader != None` and `hand_size < 10`, `WHEN` resolution fires, `THEN`: `leader.gold` decremented by bid amount, `leader.reserved_gold == 0` (zeroed, not just decremented), `hand_size` increases by 1, `S2CCardAcquired` unicast to leader, `S2CAuctionSettled { winner: Some(leader), amount }` broadcast, `AuctionSettled` Bevy Message written, `auction_state.phase == IDLE`
- [ ] **AU7-b**: `GIVEN` `timer_remaining_ms == 0` with `current_leader != None` and `leader.hand_size == 10` (injected artificially), `WHEN` resolution fires, `THEN`: `spend_reserved_gold` still called (gold deducted), `hand_size` remains 10 (card NOT added), no `S2CCardAcquired` queued, `S2CAuctionSettled { winner: Some(leader), amount }` broadcast, server error logged. *(DEPENDENCY: requires `hand_size` field on player struct — mark DEFERRED if struct not yet implemented)*
- [ ] **AU8**: `GIVEN` `timer_remaining_ms == 0` with `current_leader == None` (no bids), `WHEN` resolution fires, `THEN`: no gold values change for any player, `S2CAuctionSettled { winner: None, amount: 0 }` broadcast, `AuctionSettled` Bevy Message written, `auction_state.phase == IDLE`
- [ ] **AU14** *(Integration)*: `GIVEN` a complete prior auction ending in a win (reserved gold spent, reservation zeroed), `WHEN` a new `AuctionPhaseEntered` triggers SELECTING for the next round, `THEN` `reserved_gold == 0` for all players at SELECTING entry — no stale reservation carried from prior auction

---

## Implementation Notes

*Derived from ADR-013 Implementation Guidelines — Step 5 of `auction_tick_system`:*

```rust
// Step 5: RESOLVING — runs if LIVE_BIDDING && timer_remaining_ms == 0
if auction.phase == AuctionPhase::LiveBidding && auction.timer_remaining_ms == 0 {
    auction.phase = AuctionPhase::Resolving;

    match auction.current_leader {
        Some(winner) => {
            // Case A: Winner exists
            let bid_amount = auction.current_price;
            if let Some(econ) = economies.0.get_mut(&winner) {
                // Invariant check (production guard, not just debug_assert):
                if econ.gold < econ.reserved_gold {
                    tracing::error!("CRITICAL: gold < reserved_gold at resolution — session corrupt");
                }
                debug_assert!(econ.gold >= econ.reserved_gold, "gold invariant violated");
                api::spend_reserved_gold(econ);  // deducts reserved_gold from gold; zeroes reservation
            }
            let card_id = auction.card_id.expect("card_id must be set in RESOLVING");
            if let Some(hand) = hands.get_mut(&winner) {
                if hand.size < 10 {
                    hand.add(card_id);
                    // unicast S2CCardAcquired { card_id, source: AcquisitionSource::AuctionWon }
                } else {
                    tracing::error!("Resolution: winner hand full — unreachable under correct RSM; card discarded");
                }
            }
            // broadcast S2CAuctionSettled { winner: Some(winner), amount: bid_amount }
            settled_writer.write(AuctionSettled { winner: Some(winner), final_price: bid_amount, card_id });
        }
        None => {
            // Case B: No bids placed — no gold changes, no card awarded
            // broadcast S2CAuctionSettled { winner: None, amount: 0 }
            settled_writer.write(AuctionSettled { winner: None, final_price: 0, card_id: auction.card_id.unwrap_or(CardId(0)) });
        }
    }

    // Return to IDLE
    *auction = AuctionState::default();
}
```

**AU7-b hand_size dependency:** This story requires a `hand_size` field on a player struct. This may not yet exist if `card-acquisition` or `hand-ui` stories have not been implemented. At story implementation time, confirm which struct owns `hand_size` before writing the AU7-b test — if the struct is not yet implemented, mark AU7-b test as DEFERRED and add a comment in the test file.

**AU14 is an Integration test**: Run with `App::new()` registering both Auction System and Economy plugins. The test requires two sequential `app.update()` cycles — one for the prior auction (through RESOLVING→IDLE), then a new `AuctionPhaseEntered`. Evidence goes to `tests/integration/auction/`.

---

## Out of Scope

- Pool `copies_remaining` decrement at SELECTING (verified by AU8-pool) — Story 008
- `AbortAuction` during RESOLVING (no-op guard) — Story 003
- Plugin registration — Story 007

---

## QA Test Cases

*Written by qa-lead at story creation. AU7-a/AU7-b/AU8 can use `World::new()` (Logic); AU14 requires `App::new()` (Integration).*

**AC AU7-a — Case A: winner with room in hand:**
```
Test: winner gets gold deducted and card added to hand
  Given: World with AuctionState { phase: LiveBidding, card_id: Some(CardId(4)),
                                   current_price: 7, current_leader: Some(PlayerId(1)),
                                   timer_remaining_ms: 0 }
         Player 1: gold=10, reserved_gold=7, hand_size=3
         Messages<AuctionSettled> resource registered and empty
  When: auction_tick_system runs (timer==0 → RESOLVING fires in Step 5)
  Then: Player 1.gold == 3 (10 - 7)
        Player 1.reserved_gold == 0 (zeroed, not just decremented)
        Player 1.hand_size == 4 (card added)
        S2CCardAcquired { source: AcquisitionSource::AuctionWon } unicast to Player 1
        S2CAuctionSettled { winner: Some(PlayerId(1)), amount: 7 } broadcast
        Messages<AuctionSettled> contains exactly one event
        auction_state.phase == AuctionPhase::Idle
  Edge cases: reserved_gold == 0 before spend → debug_assert! fires (should not happen);
              verify production guard logs critical error
```

**AC AU7-b — Case A: winner with full hand (card discarded):**
```
Test: hand-full at resolution — gold deducted, card NOT added
  Given: Same as AU7-a but Player 1.hand_size = 10 (injected artificially)
  When: auction_tick_system runs
  Then: Player 1.gold == 3 (spend_reserved_gold still called — gold deducted)
        Player 1.reserved_gold == 0
        Player 1.hand_size == 10 (unchanged — card discarded)
        No S2CCardAcquired queued
        S2CAuctionSettled { winner: Some(PlayerId(1)), amount: 7 } IS broadcast
        Server error is logged (unreachable under correct RSM but guard is present)
  [DEFERRED if hand_size struct not yet implemented — mark with // DEFERRED: awaiting
   card-acquisition or hand-ui story that defines PlayerHand.hand_size]
```

**AC AU8 — Case B: no bids, no gold changes:**
```
Test: no-bid resolution — no gold changes, AuctionSettled{None} fired
  Given: AuctionState { phase: LiveBidding, card_id: Some(CardId(2)),
                        current_price: 3, current_leader: None, timer_remaining_ms: 0 }
         Player 1: gold=8, reserved_gold=0
         Player 2: gold=12, reserved_gold=0
  When: auction_tick_system runs
  Then: Player 1.gold == 8 (unchanged)
        Player 2.gold == 12 (unchanged)
        All reserved_gold == 0 (unchanged)
        S2CAuctionSettled { winner: None, amount: 0 } broadcast
        Messages<AuctionSettled> contains exactly one event
        auction_state.phase == AuctionPhase::Idle
  Edge cases: No S2CCardAcquired queued in Case B
```

**AC AU14 — reserved_gold == 0 at SELECTING entry of second auction (Integration):**
```
Test: post-settlement stale reservation invariant across two auctions
  Given: App::new() with AuctionPlugin + EconomyPlugin registered
  Phase 1 — run complete auction to IDLE:
    AuctionPhaseEntered → LIVE_BIDDING
    Player 2 bid of 5g accepted (Player 2.reserved_gold = 5)
    Set timer_remaining_ms = 0 → RESOLVING → Player 2 wins
    Player 2.gold decremented; Player 2.reserved_gold == 0; auction_state.phase = Idle
  Phase 2 — trigger next auction:
    New AuctionPhaseEntered injected
    Run auction_tick_system once (handles SELECTING entry)
  When: SELECTING entry is reached for the second auction
  Then: Player 1.reserved_gold == 0 (was outbid in prior auction and released)
        Player 2.reserved_gold == 0 (won prior auction; reservation was spent)
        reserved_gold == 0 for ALL players before any new bids placed
  [Place in tests/integration/auction/resolution_settlement_test.rs]
```

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/unit/auction/resolution_settlement_test.rs` — AU7-a, AU7-b, AU8 (unit tests, `World::new()`)
- `tests/integration/auction/resolution_settlement_test.rs` — AU14 (Integration, `App::new()` with two plugins)

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 DONE (timer reaches 0 after bid processing; full acceptance path complete)
- Depends on: `economy-system` story-005 DONE (provides `spend_reserved_gold` / confirmed API name per OQ4)
- Depends on: `workspace-and-shared-types` story-002 DONE (provides `AcquisitionSource`, `S2CCardAcquired`, `S2CAuctionSettled`)
- Unlocks: Story 007 (Plugin Registration & Scheduling — all auction_tick_system steps must be in place)
