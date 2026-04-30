# Story 002: Auction Phase Entry — AuctionPhaseEntered & IDLE Guard

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-001`, `TR-AUC-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: `auction_tick_system` reads `MessageReader<AuctionPhaseEntered>` as Step 1 of its execution body. If phase is IDLE, the system transitions IDLE→SELECTING, calls `draw_auction_card()`, broadcasts `S2CAuctionCard`, then transitions to LIVE_BIDDING. Non-IDLE receipt is silently discarded with a server error log.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- Use `MessageReader<AuctionPhaseEntered>` (Bevy internal message bus, registered via `app.add_message::<AuctionPhaseEntered>()`). This is NOT `EventReader` (removed in Bevy 0.17+) and NOT Lightyear's `MessageReceiver<T>` (for network C2S)
- AU1-b-server asserts that `Messages<S2CPhaseChanged>` is empty after `AuctionPhaseEntered` is handled. In Bevy 0.18, read the message resource registered by `app.add_message::<S2CPhaseChanged>()` — the GDD shorthand `Events<S2CPhaseChanged>` uses pre-0.17 naming
- `liv-bevy-018` + `liv-bevy-lightyear` skills are mandatory on `system.rs`
- **Verification required (ADR-013 VR-1)**: Lightyear 0.26 C2S receiver system param is `MessageReceiver<T>` — confirmed. Do NOT use `MessageReader<C2SAuctionBid>` for the Lightyear bid receiver

**Control Manifest Rules (Feature Layer)**:
- Required: RSM sends `AuctionPhaseEntered` — Auction System subscribes via `MessageReader<AuctionPhaseEntered>` (never reads `RoundState` directly)
- Required: `BroadcastPhaseChanged` (which sends `S2CPhaseChanged`) is always emitted LAST in the RSM phase entry sequence AFTER `AuctionPhaseEntered` is fully handled — the ordering invariant is structural, not timing-based
- Forbidden: `EventWriter<T>` / `EventReader<T>` — removed in Bevy 0.17+; use `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()`
- Note: ADR-013 is Accepted but not yet incorporated in the control manifest (v2026-04-30 lists it as pending)

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [ ] **AU1-a**: `GIVEN` `AuctionPhaseEntered` is processed in IDLE state, `WHEN` `auction_tick_system` returns, `THEN` `auction_state.phase == LIVE_BIDDING`, `auction_state.card_id != None`, and `auction_state.current_price == starting_price_for_drawn_rarity` (3 for Rare, 4 for Epic, 5 for Legendary)
- [ ] **AU1-b-server**: `GIVEN` `AuctionPhaseEntered` is processed, `WHEN` the system returns, `THEN` `S2CAuctionCard` has been written to the outbound message queue AND the `Messages<S2CPhaseChanged>` resource contains zero items (RSM sends this AFTER the Auction System handles `AuctionPhaseEntered` — in a single-system test the RSM does not run)
- [ ] **AU23**: `GIVEN` the Auction System is in LIVE_BIDDING and `AuctionPhaseEntered` arrives (duplicate trigger), `WHEN` the system processes it, `THEN` phase remains LIVE_BIDDING, no `S2CAuctionCard` is queued, and a server error is logged

---

## Implementation Notes

*Derived from ADR-013 Implementation Guidelines — Step 1 of `auction_tick_system`:*

```rust
// Step 1: Handle AuctionPhaseEntered (code order within auction_tick_system body)
for _msg in phase_entered.read() {
    if auction.phase != AuctionPhase::Idle {
        tracing::error!("AuctionPhaseEntered received in non-Idle state {:?} — RSM bug", auction.phase);
        continue;
    }
    auction.phase = AuctionPhase::Selecting;
    let drawn = card_pool.draw_auction_card(...);
    match drawn {
        None => {
            // Pool empty — fire AuctionSettled{None} immediately, return to Idle
            // (pool-empty guard; see Story 008 for full pool integration)
        }
        Some(card_id) => {
            let rarity = card_catalog.get(&card_id).map(|c| c.rarity).unwrap_or(Rarity::Rare);
            let starting_price = match rarity {
                Rarity::Rare      => game_config.auction_floor_rare,
                Rarity::Epic      => game_config.auction_floor_epic,
                Rarity::Legendary => game_config.auction_floor_legendary,
            };
            auction.card_id = Some(card_id);
            auction.current_price = starting_price;
            auction.timer_remaining_ms = game_config.auction_timer_seconds * 1000;
            auction.current_leader = None;
            auction.phase = AuctionPhase::LiveBidding;
            // Broadcast S2CAuctionCard BEFORE S2CPhaseChanged (RSM invariant)
            // Use Lightyear MessageSender — confirm exact API from Lightyear 0.26 docs
        }
    }
}
```

**Announcement ordering invariant (GDD Rule 3 + AU1-b-server):** The Auction System handles `AuctionPhaseEntered` fully before the RSM emits `S2CPhaseChanged(DRAFT_AUCTION)`. This is enforced structurally by the RSM's `advance_phase` code order (from ADR-010): `AuctionPhaseEntered` fires → Auction System handles it in the same tick → `BroadcastPhaseChanged` fires last. The unit test for AU1-b-server verifies the server-side queue state in isolation; the network-layer guarantee is tested by AU1-b-network (BLOCKED — see Story 007).

**Note on `draw_auction_card()` stub:** This story calls `draw_auction_card()`. At implementation time, the full pool integration (Legendary stratification, distribute()) is tested in Story 008. For Stories 002–006, use a fixture that returns a test card — the real integration is validated separately.

---

## Out of Scope

- AbortAuction handling (Step 2) — Story 003
- Bid validation and drain (Step 3) — Story 004
- Timer decrement (Step 4) — Story 005
- Resolution (Step 5) — Story 006
- `draw_auction_card()` pool integration and Legendary stratification — Story 008
- Plugin registration and system scheduling — Story 007
- AU1-b-network (Lightyear FIFO ordering guarantee) — Story 007, BLOCKED pending ADR-008

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new cases.*

**AC AU1-a — AuctionPhaseEntered in IDLE transitions to LIVE_BIDDING:**
```
Test: phase entry initialises state correctly
  Given: World with AuctionState::default() (Idle)
         CardPool with at least one Rare card (copies_remaining >= 1)
         GameConfig with auction_timer_seconds = 20, auction_floor_rare = 3
         AuctionPhaseEntered message injected into the Bevy message bus
  When: auction_tick_system runs once
  Then: auction_state.phase == AuctionPhase::LiveBidding
        auction_state.card_id != None
        auction_state.current_price == 3  // Rare starting_price
  Edge cases: Epic card drawn → current_price == 4
              Legendary card drawn (eligible round) → current_price == 5
```

**AC AU1-b-server — S2CAuctionCard queued; S2CPhaseChanged NOT queued:**
```
Test: S2CAuctionCard enqueued before RSM sends S2CPhaseChanged
  Given: Same world setup as AU1-a
         Messages<S2CPhaseChanged> resource registered and empty
  When: auction_tick_system runs once (RSM system does NOT run in this test)
  Then: S2CAuctionCard is in the outbound queue
        Messages<S2CPhaseChanged> resource is still empty (zero items)
  Note: Use the same method to inspect the outbound queue as the Lightyear
        0.26 abstract helper defined in ADR-013 VR-1.
```

**AC AU23 — duplicate AuctionPhaseEntered in LIVE_BIDDING is discarded:**
```
Test: duplicate trigger is silently rejected with server error log
  Given: AuctionState { phase: LiveBidding, card_id: Some(CardId(5)),
                        current_price: 4, current_leader: None,
                        timer_remaining_ms: 8000 }
         AuctionPhaseEntered injected into message bus
         Tracing subscriber capturing error-level logs
  When: auction_tick_system runs once
  Then: phase still == AuctionPhase::LiveBidding
        card_id still == Some(CardId(5))
        current_price still == 4
        No S2CAuctionCard queued
        At least one error-level log event captured
  Edge cases: Same guard applies in Selecting and Resolving states (add subtests)
```

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/auction/auction_phase_entry_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 DONE (`AuctionState`, `AuctionPhase`, `AuctionSnapshot` types defined)
- Depends on: `economy-system` story-001 DONE (provides `reserve_gold`, `release_gold_reservation`, `can_afford_bid` in `economy/api.rs`)
- Depends on: `card-data-pool` story-001 DONE (provides `draw_auction_card()` and `distribute()`)
- Depends on: `round-state-machine` story-001 DONE (provides `AuctionPhaseEntered` Bevy Message on ADR-010 event bus)
- Depends on: `workspace-and-shared-types` story-002 DONE (provides `C2SAuctionBid`, `S2CAuctionCard`, `S2CAuctionSettled` in `shared/protocol.rs`)
- Unlocks: Story 003 (AbortAuction Handler), Story 004 (Bid Validation Gate)
