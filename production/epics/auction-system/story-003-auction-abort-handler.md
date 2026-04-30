# Story 003: AbortAuction — Cleanup Handler & RESOLVING Guard

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: `auction_tick_system` reads `MessageReader<AbortAuction>` as Step 2 of its execution body. On receipt: release any gold reservation held by the current leader, return to IDLE. Do NOT write `AuctionSettled` — the RSM has already committed to GAME_OVER. RESOLVING is uninterruptible: `AbortAuction` arriving during RESOLVING is a no-op (settlement completes normally).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `AbortAuction` is a Bevy internal buffered message registered via `app.add_message::<AbortAuction>()` and read via `MessageReader<AbortAuction>` — NOT `EventReader` (removed in 0.17+)
- `AbortAuction` was absent from ADR-010's event catalog but has been added as part of ADR-013 authoring — confirm it is registered on the event bus before implementing
- `liv-bevy-018` + `liv-bevy-lightyear` skills mandatory on `system.rs`
- **Pre-implementation gate (Epic)**: The RSM GDD must document `auction_max_duration_seconds` safety timeout that triggers `AbortAuction`. Confirm this is present in `design/gdd/round-state-machine.md` before writing this story

**Control Manifest Rules (Feature Layer)**:
- Required: Auction System reacts to RSM events (`AbortAuction` from `round-state-machine`) via `MessageReader<T>` — never imports RSM modules directly
- Forbidden: Do NOT fire `AuctionSettled` on `AbortAuction` — the RSM has already committed to GAME_OVER; a spurious `AuctionSettled` would cause the RSM to attempt a DRAFT_SHOP transition after GAME_OVER
- Note: ADR-013 is Accepted but not yet incorporated in the control manifest (v2026-04-30 lists it as pending)

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [ ] **AU9**: `GIVEN` the Auction System is in LIVE_BIDDING with `current_leader == Some(Player_A)` and `Player_A.reserved_gold == 5`, `WHEN` `AbortAuction` is received, `THEN` `Player_A.reserved_gold == 0`, `auction_state.phase == IDLE`, and `Messages<AuctionSettled>` contains zero events
- [ ] **AU19-b**: `GIVEN` the Auction System is in SELECTING, `WHEN` `AbortAuction` is received, `THEN` Auction System returns to IDLE and `AuctionSettled` is NOT fired (reservation vacuously absent in SELECTING)
- [ ] **AU19-a**: `GIVEN` the Auction System is artificially placed in RESOLVING state with a current leader (injected directly), `WHEN` `AbortAuction` is received, `THEN` `AbortAuction` is a no-op — RESOLVING completes normally: gold deducted, card added to hand, `AuctionSettled` IS fired, system transitions to IDLE

---

## Implementation Notes

*Derived from ADR-013 Implementation Guidelines — Step 2 of `auction_tick_system`:*

```rust
// Step 2: Handle AbortAuction (must run BEFORE bid drain — code order enforces this)
for _msg in abort.read() {
    match auction.phase {
        AuctionPhase::Resolving => {
            // RESOLVING is uninterruptible — AbortAuction is a no-op here.
            // Settlement will complete in Step 5 of this same tick.
            // Do nothing.
        }
        AuctionPhase::Idle => {
            // No active auction — no-op. The RSM may send AbortAuction speculatively.
        }
        _ => {
            // Selecting or LiveBidding: cancel and clean up.
            if let Some(leader) = auction.current_leader {
                if let Some(econ) = economies.0.get_mut(&leader) {
                    api::release_gold_reservation(econ);
                }
            }
            auction.phase = AuctionPhase::Idle;
            auction.card_id = None;
            auction.current_price = 0;
            auction.current_leader = None;
            auction.timer_remaining_ms = 0;
            // Do NOT write to settled_writer here.
        }
    }
}
```

**`release_gold_reservation` pre-condition check:** Before calling this function, check `current_leader != None`. If `current_leader == None` (no bids placed yet), there is no reservation to release — skip the call entirely. Do not pass `None` to the economy API.

**AU19-a is a regression guard:** RESOLVING is effectively unreachable from `AbortAuction` under correct Bevy scheduling (the RSM commits GAME_OVER after RESOLVING completes). The test guards against future schedule refactoring that would make RESOLVING interruptible.

---

## Out of Scope

- AuctionPhaseEntered handling (Step 1) — Story 002
- Bid drain and validation (Step 3) — Story 004
- Timer decrement (Step 4) — Story 005
- Resolution and settlement (Step 5) — Story 006
- Plugin registration — Story 007

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new cases.*

**AC AU9 — AbortAuction in LIVE_BIDDING releases reservation and returns to IDLE:**
```
Test: AbortAuction cleans up reservation in LIVE_BIDDING
  Given: World with:
         AuctionState { phase: LiveBidding, card_id: Some(CardId(2)),
                        current_price: 6, current_leader: Some(PlayerId(1)),
                        timer_remaining_ms: 5000 }
         PlayerEconomies: Player 1 { gold: 10, reserved_gold: 5 }
         AbortAuction message injected
         Messages<AuctionSettled> resource registered and empty
  When: auction_tick_system runs once
  Then: Player 1.reserved_gold == 0
        auction_state.phase == AuctionPhase::Idle
        Messages<AuctionSettled> still contains zero events
  Edge cases: current_leader == None (no bids placed) — system still returns to IDLE;
              release step is skipped; no panic; reserved_gold remains 0 for all
```

**AC AU19-b — AbortAuction in SELECTING returns to IDLE, no AuctionSettled:**
```
Test: AbortAuction in SELECTING is a clean no-op except state return
  Given: AuctionState { phase: Selecting, card_id: None, current_price: 0,
                        current_leader: None, timer_remaining_ms: 0 }
         All players have reserved_gold == 0
         AbortAuction injected
  When: auction_tick_system runs once
  Then: auction_state.phase == AuctionPhase::Idle
        Messages<AuctionSettled> contains zero events
        No player has non-zero reserved_gold
```

**AC AU19-a — AbortAuction in RESOLVING is a no-op; settlement completes:**
```
Test: RESOLVING is uninterruptible — AbortAuction does not interrupt settlement
  Given: AuctionState { phase: Resolving, card_id: Some(CardId(9)),
                        current_price: 7, current_leader: Some(PlayerId(2)),
                        timer_remaining_ms: 0 }
         Player 2: { gold: 10, reserved_gold: 7, hand_size: 3 }
         AbortAuction injected
         Messages<AuctionSettled> empty
  When: auction_tick_system runs once
  Then: Player 2.gold == 3 (10 - 7 via spend_reserved_gold)
        Player 2.reserved_gold == 0
        Player 2.hand_size == 4 (card awarded)
        Messages<AuctionSettled> contains exactly one event
        auction_state.phase == AuctionPhase::Idle
  Note: This test requires Step 5 (resolution) to be implemented; write it after Story 006
        or stub Step 5 locally in this test. Mark the story test as pending Step 5 if needed.
```

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/auction/auction_abort_handler_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 DONE (`AuctionState`, `AuctionPhase` defined)
- Depends on: `economy-system` story-001 DONE (provides `release_gold_reservation` in `economy/api.rs`)
- Depends on: `round-state-machine` story-001 DONE (provides `AbortAuction` Bevy Message)
- Note: AU19-a test requires Story 006 (resolution) to be implemented first — write it last among Story 003 tests
- Unlocks: Story 007 (Plugin Registration) — AbortAuction handler must be in place before plugin is final
