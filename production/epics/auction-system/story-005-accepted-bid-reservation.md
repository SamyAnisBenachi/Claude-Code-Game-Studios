# Story 005: Accepted Bid — Gold Reservation Handoff & Timer Reset

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-004`, `TR-AUC-005`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: When a bid passes all 5 validation conditions, the acceptance path executes atomically within one system body: `release_gold_reservation(prev_leader)` then `reserve_gold(new_leader, amount)` sequentially, with no system boundary between them. Timer reset uses Formula 3 (`min(remaining + reset_ms, cap_ms)`). Tick delta is clamped to 1000ms before `saturating_sub` to prevent lag spikes from consuming the full timer.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- Bevy 0.18 `Time::delta().as_millis()` returns `u128`; cast with `u32::try_from(...).unwrap_or(u32::MAX)` — never use `as u32` (Clippy CI failure risk per ADR-013)
- `S2CAuctionBidAccepted` is a reliable broadcast — all players receive it. Confirm `NetworkTarget::All` variant in Lightyear 0.26 (verification checklist item 8: confirmed ✅)
- `S2CGoldBroadcast` is fired by the Economy System on `reserve_gold` and `release_gold_reservation` calls — the Auction System does NOT send it directly. M7-a and M7-c test that the Economy System fires it correctly in response to Auction System API calls
- `liv-bevy-018` + `liv-bevy-lightyear` skills mandatory on `system.rs`
- Time delta injection in tests: use Bevy's `app.world.resource_mut::<Time>().advance_by(Duration::from_millis(N))` — confirm the exact Bevy 0.18 API before writing AU22 test

**Control Manifest Rules (Feature Layer)**:
- Required: Release-before-reserve ordering — `api::release_gold_reservation(prev_leader)` THEN `api::reserve_gold(new_leader, amount)` in same function body — no `await`, `yield`, or system boundary between them
- Required: First bid skip — if `current_leader == None` before this bid, skip `release_gold_reservation` entirely (do not call with None; this is a no-op path, not an error)
- Forbidden: Do NOT access gold fields directly — all gold operations go through `economy/api.rs` functions on `ResMut<PlayerEconomies>`
- Guardrail: Timer reset must never exceed `auction_timer_seconds * 1000` — the `min()` cap is mandatory
- Note: ADR-013 is incorporated in control manifest v2026-05-01. Current protocol/GDD naming uses `C2SPlaceBid`; any `MessageReceiver<C2SAuctionBid>` wording in ADR-derived references maps to `MessageReceiver<C2SPlaceBid>` at implementation time.

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [ ] **AU4**: `GIVEN` Player A has `gold=10, reserved_gold=5` (current leader) and Player B has `gold=10, reserved_gold=0`, `WHEN` Player B's bid of 6g is accepted, `THEN` `Player_A.reserved_gold == 0` AND `Player_B.reserved_gold == 6`
- [ ] **AU20**: `GIVEN` Player A is current leader with `reserved_gold = A_amt`, `WHEN` Player B's bid is accepted, `THEN` pre-bid snapshot (`A=A_amt, B=0`) and post-bid snapshot (`A=0, B=B_amt`) are both explicitly asserted in the test
- [ ] **M7-a**: `GIVEN` an accepted bid, `WHEN` the Economy System fires `S2CGoldBroadcast` for the new leader, `THEN` the broadcast includes `reserved_gold == bid_amount` (not 0 or absent)
- [ ] **M7-c**: `GIVEN` Player A is outbid by Player B, `WHEN` `release_gold_reservation(Player_A)` fires, `THEN` `S2CGoldBroadcast { player_id: Player_A, reserved_gold: 0 }` is dispatched — distinct from and in addition to the M7-a broadcast for Player B. Both must fire on the same outbid event
- [ ] **AU5**: `GIVEN` an accepted bid with `timer_remaining_ms=3000`, `auction_timer_seconds=20`, `auction_timer_reset_seconds=5` (inject as test constants), `THEN` `timer_remaining_ms = min(3000+5000, 20000) = 8000`
- [ ] **AU6**: `GIVEN` an accepted bid with `timer_remaining_ms=17000`, same config, `THEN` `timer_remaining_ms = min(17000+5000, 20000) = 20000` (capped)
- [ ] **AU22**: `GIVEN` LIVE_BIDDING with `timer_remaining_ms=T` and `tick_delta_ms=5000` (lag spike injected), `WHEN` the timer decrement step runs, `THEN` `timer_remaining_ms` decrements by at most 1000ms, not 5000ms. Assert `new_timer == T.saturating_sub(1000)`

---

## Implementation Notes

*Derived from ADR-013 Implementation Guidelines — Step 3 acceptance path and Step 4:*

```rust
// Step 3 continued: Accepted bid (all 5 conditions passed)
// Step 1 of Rule 5: release previous leader's reservation
if let Some(prev_leader) = auction.current_leader {
    if let Some(econ) = economies.0.get_mut(&prev_leader) {
        api::release_gold_reservation(econ);  // fires S2CGoldBroadcast for prev_leader
    }
}
// Step 2 of Rule 5: reserve new leader's gold
if let Some(econ) = economies.0.get_mut(&bidder) {
    let _ = api::reserve_gold(econ, amount);  // pre-validated by can_afford_bid; fires S2CGoldBroadcast for bidder
}
// Step 3 of Rule 5: update state
auction.current_price = amount;
auction.current_leader = Some(bidder);
// Step 4 of Rule 5: timer reset (Formula 3)
let reset_ms = game_config.auction_timer_reset_seconds * 1000;
let cap_ms   = game_config.auction_timer_seconds * 1000;
auction.timer_remaining_ms = (auction.timer_remaining_ms + reset_ms).min(cap_ms);
// Step 5 of Rule 5: broadcast accepted bid
// send S2CAuctionBidAccepted { bidder, amount, new_timer_ms: auction.timer_remaining_ms }

// Step 4 (outer): timer decrement (runs after bid drain, every tick)
let raw_delta_ms = u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX);
let safe_delta   = raw_delta_ms.min(1000u32);  // 1000ms lag-spike clamp (GDD Rule 6)
auction.timer_remaining_ms = auction.timer_remaining_ms.saturating_sub(safe_delta);
```

**M7-a / M7-c responsibility split**: `S2CGoldBroadcast` is triggered by the Economy System's `reserve_gold` and `release_gold_reservation` functions, not by the Auction System directly. This story tests that those Economy API calls fire the broadcast correctly. Confirm with the Economy System story owner that the Economy tests cover the same cases before writing duplicate tests.

**Timer cap interaction**: AU5 and AU6 test that the `min(remaining + reset, cap)` formula is evaluated atomically — not "add then cap in the next frame." The reset and cap are computed together in one expression. The timer decrement (Step 4) fires after the bid acceptance and may reduce the value by one tick's worth — account for this in tests by capturing the timer value immediately after Step 3 completes, before Step 4 runs, or run Step 3 and Step 4 in separate isolated test contexts.

---

## Out of Scope

- Bid validation/rejection path (already handled) — Story 004
- Resolution and settlement — Story 006
- Pool integration — Story 008
- Plugin registration — Story 007

---

## QA Test Cases

*Written by qa-lead at story creation. Use `App::new()` with AuctionPlugin + EconomyPlugin registered for M7-a and M7-c. AU4/AU5/AU6/AU22 can use `World::new()` if Economy state is fixture-built directly.*

**AC AU4 — Gold reservation handoff on outbid:**
```
Test: prev leader's reservation released; new leader's gold reserved
  Given: App::new() with AuctionPlugin + EconomyPlugin
         AuctionState { phase: LiveBidding, current_price: 5,
                        current_leader: Some(PlayerId(1)), timer_remaining_ms: 10000 }
         Player 1 (PlayerId(1)): gold=10, reserved_gold=5
         Player 2 (PlayerId(2)): gold=10, reserved_gold=0, hand_size=3
         C2SPlaceBid { amount: 6 } injected for PlayerId(2) (passes all 5 conditions)
  When: app.update() runs auction_tick_system
  Then: Player 1.reserved_gold == 0
        Player 2.reserved_gold == 6
  Edge cases: First bid (current_leader was None) — release step skipped;
              Player 1 reserved_gold unchanged before the first bid
```

**AC AU20 — Pre-bid and post-bid snapshot assertions:**
```
Test: explicit before/after snapshot of reservation invariant
  Given: Same setup as AU4 with A_amt = 5 (Player 1 has reserved_gold = 5 before bid)
  Capture pre-bid: Player 1.reserved_gold = 5, Player 2.reserved_gold = 0
  When: Player 2's bid accepted
  Then: Post-bid: Player 1.reserved_gold == 0, Player 2.reserved_gold == 6
        Assert both snapshots explicitly in the test body (not derived)
```

**AC M7-a — S2CGoldBroadcast for new leader includes reserved_gold == bid_amount:**
```
Test: reserve_gold triggers S2CGoldBroadcast with reserved_gold field
  Given: Same app setup as AU4; Player 2 bid of 6g accepted
  When: Economy API reserve_gold(Player_2, 6) is called by auction_tick_system
  Then: S2CGoldBroadcast { player_id: PlayerId(2), gold: 10, reserved_gold: 6 } fired
        reserved_gold field == bid_amount (6), not 0 or unset
  Edge cases: Verify gold field is total gold, not free_gold
```

**AC M7-c — S2CGoldBroadcast for outbid player has reserved_gold == 0:**
```
Test: release_gold_reservation triggers S2CGoldBroadcast for outbid player
  Given: Same test run as M7-a (Player 1 outbid by Player 2)
  When: Economy API release_gold_reservation(Player_1) called
  Then: S2CGoldBroadcast { player_id: PlayerId(1), gold: 10, reserved_gold: 0 } fired
        Broadcast for Player 1 fires IN ADDITION TO (not instead of) Player 2 broadcast
        Both broadcasts present in same test run
```

**AC AU5 — Timer reset, non-capped case:**
```
Test: timer reset formula when sum is below cap
  Given: AuctionState { phase: LiveBidding, current_price: 3, timer_remaining_ms: 3000 }
         GameConfig { auction_timer_seconds: 20, auction_timer_reset_seconds: 5 }
           (inject as test constants, not read from asset loader)
         Accepted bid triggers timer reset
  When: Step 3 acceptance path runs
  Then: timer_remaining_ms immediately after reset == min(3000+5000, 20000) == 8000
        (before Step 4 decrement — isolate or account for tick decrement)
  Edge cases: timer = 0 + reset = 5000 → min(5000, 20000) = 5000 (not capped)
```

**AC AU6 — Timer reset, capped case:**
```
Test: timer reset formula when sum exceeds cap
  Given: AuctionState { timer_remaining_ms: 17000 }
         Same GameConfig as AU5
  When: Accepted bid triggers reset
  Then: timer_remaining_ms after reset == min(17000+5000, 20000) == 20000
  Edge cases: timer = 15001 → min(20001, 20000) = 20000 (capped)
              timer = 15000 → min(20000, 20000) = 20000 (exactly at cap)
              timer = 14999 → min(19999, 20000) = 19999 (not capped)
```

**AC AU22 — Lag spike clamp: delta capped at 1000ms:**
```
Test: tick_delta = 5000ms results in only 1000ms decrement
  Given: AuctionState { phase: LiveBidding, timer_remaining_ms: 12000 }
         No bids in queue
         Bevy Time resource advanced by 5000ms (tick_delta = 5000)
  When: auction_tick_system runs (Step 4: timer decrement)
  Then: timer_remaining_ms == 12000 - 1000 == 11000
        Timer is NOT decremented by 5000
  Edge cases: tick_delta = 1000 → decremented by 1000 (clamp not needed)
              tick_delta = 999  → decremented by 999 (clamp not triggered)
              tick_delta = 1001 → decremented by 1000 (clamp triggered)
              timer = 500, tick_delta = 5000 → 500.saturating_sub(1000) = 0 (saturating)
```

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/auction/accepted_bid_reservation_test.rs` — must exist and pass (uses `App::new()` with AuctionPlugin + EconomyPlugin for M7-a, M7-c; unit tests for AU4/AU5/AU6/AU20/AU22)

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 DONE (bid drain loop structure in place; acceptance path follows validation)
- Depends on: `economy-system` story-005 DONE (provides `reserve_gold`, `release_gold_reservation`, and their `S2CGoldBroadcast` firing in `economy/api.rs`)
- Unlocks: Story 006 (Resolution & Settlement — timer reaching 0 triggers RESOLVING)
