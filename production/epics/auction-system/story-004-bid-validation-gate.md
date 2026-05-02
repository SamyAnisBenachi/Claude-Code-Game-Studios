# Story 004: Bid Validation — 5-Condition Rejection Gate

> **Epic**: Auction System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-003`, `TR-AUC-010`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: The bid drain loop (Step 3 of `auction_tick_system`) runs only in LIVE_BIDDING. It drains Lightyear's `MessageReceiver<C2SAuctionBid>` in arrival order. For each bid, it validates 5 conditions in sequence and unicasts `S2CAuctionBidRejected` with a specific `BidRejectedReason` variant on the first failing condition. No state changes on rejection.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- **Critical API boundary**: Lightyear C2S messages use `MessageReceiver<C2SAuctionBid>` (Lightyear system param, components on entities). Do NOT use Bevy's `MessageReader<C2SAuctionBid>` for network bids
- **`receiver.receive()` returns `impl Iterator<Item = C2SAuctionBid>`** — no `receive_messages()` method (see Lightyear 0.26 verification checklist item 6)
- Abstract the receiver behind a helper `fn drain_bids(receiver: ...) -> impl Iterator<Item = C2SAuctionBid>` to keep bid validation logic testable without a live Lightyear session (per ADR-013)
- Unicast rejection: use `ServerMultiMessageSender::send::<S2CAuctionBidRejected, ReliableChannel>(&msg, &server, &NetworkTarget::Single(bidder_peer_id))` — confirm exact API from Lightyear 0.26 verification checklist item 9
- `liv-bevy-018` + `liv-bevy-lightyear` skills mandatory on `system.rs`
- **Pre-implementation gate (Epic / OQ9)**: Confirm whether LIVE_BIDDING with `timer_remaining_ms == 0` is a reachable state before implementing AU12. Owner: Gameplay Programmer. If OQ9 resolves as "unreachable," close AU12 with a note

**Control Manifest Rules (Feature Layer)**:
- Required: Phase-gate pattern — `if auction.phase != AuctionPhase::LiveBidding { return; }` — invalid phase → silent discard (no rejection message sent for non-LIVE_BIDDING bids arriving in IDLE, see AU18)
- Required: `can_afford_bid(player, amount)` checks `gold - reserved_gold >= amount` — Auction System does not access gold fields directly; calls economy API
- Forbidden: `MessageReceiver<C2SAuctionBid>` must appear in exactly one system (`auction_tick_system`) — code review gate on every PR
- Note: ADR-013 is Accepted but not yet incorporated in the control manifest (v2026-04-30 lists it as pending)

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [x] **AU2**: `GIVEN` LIVE_BIDDING and `bidder == current_leader`, `WHEN` `C2SAuctionBid` arrives, `THEN` `S2CAuctionBidRejected { reason: AlreadyLeader }` unicast to bidder; no state changes
- [x] **AU3**: `GIVEN` LIVE_BIDDING and `bidder.hand_size == 10`, `WHEN` `C2SAuctionBid` arrives, `THEN` `S2CAuctionBidRejected { reason: HandFull }` unicast; no state changes
- [x] **AU16**: `GIVEN` LIVE_BIDDING and `bidder.gold.saturating_sub(bidder.reserved_gold) < bid_amount`, `WHEN` `C2SAuctionBid { amount: bid_amount }` arrives, `THEN` `S2CAuctionBidRejected { reason: InsufficientGold }` unicast; no state changes
- [x] **AU17**: `GIVEN` LIVE_BIDDING with `current_price = P`, `WHEN` `C2SAuctionBid { amount: P }` arrives (at-price), `THEN` `S2CAuctionBidRejected { reason: AmountTooLow }` unicast; no state changes. Also verify `amount = P - 1`
- [x] **AU12** *(conditional on OQ9)*: `GIVEN` OQ9 resolves "AuctionExpired is reachable" AND the system is in LIVE_BIDDING with `timer_remaining_ms = 0` (injected), `WHEN` `C2SAuctionBid` arrives, `THEN` `S2CAuctionBidRejected { reason: AuctionExpired }` unicast; no state changes. *If OQ9 resolves "unreachable," close this AC with a note — no test written*
- [x] **AU18**: `GIVEN` the Auction System is in IDLE and a stale `C2SAuctionBid` arrives, `THEN` no `S2CAuctionBidRejected` queued, no state changes, no error logged (silent discard — distinct from `AuctionExpired`)
- [x] **AU13**: `GIVEN` two `C2SAuctionBid` messages with the same `amount = P` injected in the receiver queue before the system runs, `WHEN` the bid drain processes them in arrival order, `THEN` first bid accepted (`current_price` raised to P), second rejected with `AmountTooLow` (P is not >= P+1)

---

## Implementation Notes

*Derived from ADR-013 Implementation Guidelines — Step 3 (validation/rejection path):*

```rust
// Step 3: Drain bid receiver (only in LiveBidding)
if auction.phase == AuctionPhase::LiveBidding {
    for bid in drain_bids(&mut bids) {  // abstract helper; returns impl Iterator
        let bidder = bid.bidder;
        let amount = bid.amount;

        // Condition 1: Phase + timer check
        if auction.timer_remaining_ms == 0 {
            // Conditional on OQ9 — timer may already be 0 before RESOLVING fires
            send_rejection(&mut sender, bidder, BidRejectedReason::AuctionExpired);
            continue;
        }
        // Condition 2: Amount must exceed current_price by at least 1
        if amount < auction.current_price + 1 {
            send_rejection(&mut sender, bidder, BidRejectedReason::AmountTooLow);
            continue;
        }
        // Condition 3: Bidder cannot be the current leader
        if Some(bidder) == auction.current_leader {
            send_rejection(&mut sender, bidder, BidRejectedReason::AlreadyLeader);
            continue;
        }
        // Condition 4: Bidder must have sufficient free gold
        let player_econ = match economies.0.get(&bidder) { ... };
        if !api::can_afford_bid(player_econ, amount) {
            send_rejection(&mut sender, bidder, BidRejectedReason::InsufficientGold);
            continue;
        }
        // Condition 5: Bidder's hand must not be full
        let player_hand = match hands.get(&bidder) { ... };
        if player_hand.size >= 10 {
            send_rejection(&mut sender, bidder, BidRejectedReason::HandFull);
            continue;
        }

        // All conditions passed — proceed to acceptance (Story 005)
        accept_bid(&mut auction, &mut economies, &mut sender, bidder, amount, &game_config);
    }
}
```

**AU18 (stale IDLE bid) vs AU12 (AuctionExpired):** These are distinct behaviors. In IDLE, the outer `if auction.phase == AuctionPhase::LiveBidding` check prevents the bid drain loop from running at all — no rejection is sent. In LIVE_BIDDING with `timer == 0`, the bid drain runs but condition 1 rejects with `AuctionExpired`. The silent IDLE discard is architecturally guaranteed by the phase gate, not by explicit code in the loop.

**`BidRejectedReason` enum:** 5 variants — `InsufficientGold`, `AmountTooLow`, `AuctionExpired`, `AlreadyLeader`, `HandFull`. All confirmed in `network-protocol.md` (2026-04-29). Check condition order: GDD Rule 4 lists them in the table order above. Reject on the first failing condition — do not accumulate reasons.

---

## Out of Scope

- Accepted bid path (release → reserve → timer reset → S2CAuctionBidAccepted) — Story 005
- Pool integration and Legendary stratification — Story 008
- Plugin registration — Story 007

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new cases.*

**AC AU2 — AlreadyLeader rejection:**
```
Test: self-bid rejected with AlreadyLeader
  Given: AuctionState { phase: LiveBidding, current_price: 5,
                        current_leader: Some(PlayerId(1)), timer_remaining_ms: 8000 }
         C2SAuctionBid { bidder: PlayerId(1), amount: 6 } injected
  When: auction_tick_system bid drain runs
  Then: S2CAuctionBidRejected { reason: AlreadyLeader } unicast to PlayerId(1) only
        current_price still == 5; current_leader still == Some(PlayerId(1))
  Edge cases: current_leader == Some(PlayerId(2)) bidding → NOT AlreadyLeader
```

**AC AU3 — HandFull rejection:**
```
Test: bid rejected when hand is full
  Given: AuctionState { phase: LiveBidding, current_price: 3, current_leader: None,
                        timer_remaining_ms: 10000 }
         Player 2: hand_size == 10
         C2SAuctionBid { bidder: PlayerId(2), amount: 4 } injected
  When: bid drain runs
  Then: S2CAuctionBidRejected { reason: HandFull } unicast to PlayerId(2)
        No state changes
  Edge cases: hand_size == 9 → bid NOT rejected for HandFull (can still proceed)
```

**AC AU16 — InsufficientGold rejection:**
```
Test: bid rejected when free gold < bid amount
  Given: AuctionState in LiveBidding, current_price = 5
         Player 3: gold = 8, reserved_gold = 5 (free_gold = 3)
         C2SAuctionBid { bidder: PlayerId(3), amount: 6 }
  When: bid drain runs
  Then: S2CAuctionBidRejected { reason: InsufficientGold } unicast
        No state changes
  Edge cases: free_gold == amount (exact match) → NOT rejected
              free_gold == amount - 1 → rejected
```

**AC AU17 — AmountTooLow rejection:**
```
Test: at-price and below-price bids rejected
  Given: AuctionState in LiveBidding, current_price = 7
  Subtest A: C2SAuctionBid { amount: 7 } (at current_price)
  Subtest B: C2SAuctionBid { amount: 6 } (below current_price)
  When: bid drain runs for each
  Then: S2CAuctionBidRejected { reason: AmountTooLow } in both
  Edge cases: amount == 8 (current_price + 1) → NOT rejected for AmountTooLow
```

**AC AU12 — AuctionExpired rejection (DEFERRED pending OQ9):**
```
[DEFERRED — Do not implement until OQ9 is resolved by Gameplay Programmer]
If OQ9 resolves "AuctionExpired is reachable":
  Given: AuctionState { phase: LiveBidding, timer_remaining_ms: 0 } (injected directly)
         C2SAuctionBid injected
  When: bid drain runs before Step 5 RESOLVING check
  Then: S2CAuctionBidRejected { reason: AuctionExpired } unicast
        No state changes
If OQ9 resolves "unreachable": Close this AC with a note in the story. No test written.
```

**AC AU18 — Stale IDLE bid silently discarded:**
```
Test: bid in IDLE produces no response and no log
  Given: AuctionState { phase: Idle }
         C2SAuctionBid injected
  When: auction_tick_system runs
  Then: No S2CAuctionBidRejected queued
        No state changes
        No error logged (silent discard — not AuctionExpired)
```

**AC AU13 — Same-tick duplicate bids at same amount: first wins:**
```
Test: two bids at same amount in same tick — first accepted, second rejected
  Given: AuctionState { phase: LiveBidding, current_price: 5, current_leader: None,
                        timer_remaining_ms: 10000 }
         Player 1: gold=20, reserved_gold=0, hand_size=3
         Player 2: gold=20, reserved_gold=0, hand_size=3
         MessageReceiver queue (insertion order):
           [0] C2SAuctionBid { bidder: PlayerId(1), amount: 6 }
           [1] C2SAuctionBid { bidder: PlayerId(2), amount: 6 }  // same amount
  When: bid drain processes full queue in one system run
  Then: First bid accepted: current_price raised to 6, current_leader = Some(PlayerId(1))
        Second bid rejected: S2CAuctionBidRejected { reason: AmountTooLow } to PlayerId(2)
  Edge cases: Reverse insertion order → PlayerId(2) accepted, PlayerId(1) rejected
```

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/auction/bid_validation_gate_test.rs` — must exist and pass

**Status**: [x] Created and passing (`cargo test -p server --test auction_bid_validation_gate_test`: 9 passed, 0 failed)

---

## Dependencies

- Depends on: Story 002 DONE (establishes LIVE_BIDDING state and `auction_tick_system` structure)
- Depends on: `economy-system` story-005 DONE (provides `can_afford_bid` in `economy/api.rs`)
- Depends on: `workspace-and-shared-types` story-002 DONE (provides `C2SAuctionBid`, `BidRejectedReason`, `S2CAuctionBidRejected`)
- Unlocks: Story 005 (Accepted Bid — reservation handoff and timer reset)

## Completion Notes

**Completed**: 2026-05-02
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 7/7 passing; AU2, AU3, AU16, AU17, AU12, AU18, and AU13 are covered by `tests/unit/auction/bid_validation_gate_test.rs`.
**Test Evidence**: Logic unit test at `tests/unit/auction/bid_validation_gate_test.rs`; `cargo test -p server --test auction_bid_validation_gate_test` passed 9/9. Adjacent regression bundle `cargo test -p server --test auction_phase_entry_test --test auction_abort_handler_test --test auction_reservation_test` passed 14 executable tests with 1 ignored future settlement guard. `cargo fmt -p server -- --check` and `cargo check -p server` passed.
**Deviations**:
- Advisory: story manifest v2026-04-30 is older than current control manifest v2026-05-01.
- Advisory: story/TR/ADR wording still references `C2SAuctionBid`; current GDD, protocol, and implementation use `C2SPlaceBid`.
- Advisory: implementation drains `C2SPlaceBid` and silently discards non-`LiveBidding` bids inside `process_bid_batch`; AU18 behavior matches the requirement with no rejection and no state change.
**Code Review**: Skipped by lean review mode; local Bevy 0.18 and Lightyear review found no blocking issue.
**Scope**: Implementation commit `5bd635e` touched the auction system, protocol registration/types, the generic network logger, Cargo test registration, and the required unit test. No scope creep found for this story.
**Sprint Status**: Unchanged per user instruction; no explicit `AUC-004` / Auction Story 004 row exists in `production/sprint-status.yaml`.
**Tech Debt**: None logged.
**Next recommended**: Auction Story 005 Accepted Bid (`production/epics/auction-system/story-005-accepted-bid-reservation.md`) after readiness check, or continue the serialized closure queue for already implemented stories.
