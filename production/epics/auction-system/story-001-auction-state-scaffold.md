# Story 001: AuctionState Types & Snapshot Scaffold

> **Epic**: Auction System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/auction-system.md`
**Requirement**: `TR-AUC-009`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-013: Auction System State Machine and Bid Processing Architecture](docs/architecture/adr-013-auction-system-state.md)
**ADR Decision Summary**: Auction state lives in a single `AuctionState` resource (parallel to `RoundState`). The `auction_snapshot()` function is a pure function on `&AuctionState` — no ECS query, no system parameter, trivially testable with `World::new()`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `#[derive(Resource)]` is the correct Bevy 0.18 pattern for singleton server state — no entity/component needed
- `liv-bevy-018` skill is mandatory when editing these files
- No Lightyear APIs are used in this story (pure type definitions)

**Control Manifest Rules (Feature Layer)**:
- Required: `CardId` and `PlayerId` are newtypes from `shared/protocol.rs` — import from there, never redeclare
- Forbidden: Do not add `#[derive(Component)]` to `AuctionState` — it is a singleton `Resource`
- Guardrail: `AuctionState` struct must be `< 64 bytes` (currently ~40 bytes per ADR-013 performance analysis)
- Note: ADR-013 is Accepted but not yet in the control manifest (manifest v2026-04-30 lists it as pending). Embed rules from ADR-013 directly.

---

## Acceptance Criteria

*From GDD `design/gdd/auction-system.md`, scoped to this story:*

- [ ] **AU10-a**: `auction_snapshot()` returns `None` exactly when `phase == AuctionPhase::Idle`
- [ ] **AU10-b**: `auction_snapshot()` in `LIVE_BIDDING` with no bids placed returns `Some(AuctionSnapshot)` where `last_accepted_bid == 0` (sentinel — NOT `starting_price`)
- [ ] **AU10-c**: `auction_snapshot()` in `LIVE_BIDDING` with bids placed returns `Some(AuctionSnapshot)` where `last_accepted_bid == last_accepted_bid_amount` and `current_leader == Some(leader_id)`
- [ ] **AU10-d**: `timer_remaining_ms` in snapshot matches the injected value exactly (no transformation)
- [ ] **AU10-e**: `AuctionState::default()` starts with `phase == AuctionPhase::Idle` and all fields zeroed/None

---

## Implementation Notes

*Derived from ADR-013 Implementation Guidelines:*

**`server/src/feature/auction/state.rs`** — implement exactly as specified in ADR-013:
```rust
#[derive(Resource)]
pub struct AuctionState {
    pub phase: AuctionPhase,
    pub card_id: Option<CardId>,
    pub current_price: u32,
    pub current_leader: Option<PlayerId>,
    pub timer_remaining_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionPhase { Idle, Selecting, LiveBidding, Resolving }

impl Default for AuctionState { /* Idle, None, 0, None, 0 */ }
```

**`server/src/feature/auction/snapshot.rs`** — `auction_snapshot()` is a free function on `&AuctionState`. It returns `None` when `phase == AuctionPhase::Idle`. For all other phases it returns `Some(AuctionSnapshot)` using `state.card_id.map(...)` — note that `card_id` is `None` during `SELECTING` before `draw_auction_card()` completes (SELECTING is synchronous, so this edge is unreachable at runtime, but the function must not panic if `card_id` is `None` in a non-IDLE state).

**`last_accepted_bid` sentinel**: `0` means "no bids placed yet" — NOT `starting_price`. The client computes minimum valid bid as `last_accepted_bid + 1` when bids have been placed, and `starting_price + 1` when `last_accepted_bid == 0`. Using `starting_price` as the sentinel would cause the client to miscalculate the first valid bid.

**No `starting_price` field in `AuctionSnapshot`**: The ADR-013 snapshot struct does NOT include `starting_price` (the GDD snapshot spec does). Verify against `design/gdd/network-protocol.md` `AuctionSnapshot` definition at implementation time and add the field if the protocol spec requires it.

---

## Out of Scope

- `auction_tick_system` implementation — Story 002
- `AuctionPlugin` registration — Story 007
- Any Lightyear networking — Stories 002–008

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new cases.*

**AC AU10-a — snapshot returns None in Idle:**
```
Test: auction_snapshot returns None when phase is Idle
  Given: AuctionState { phase: Idle, card_id: None, current_price: 0,
                        current_leader: None, timer_remaining_ms: 0 }
  When: auction_snapshot(&state) is called
  Then: result is None
```

**AC AU10-b — snapshot returns Some with last_accepted_bid == 0 sentinel (no bids):**
```
Test: snapshot returns Some with 0 sentinel when no bids placed
  Given: AuctionState { phase: LiveBidding, card_id: Some(CardId(7)),
                        current_price: 3, current_leader: None,
                        timer_remaining_ms: 12000 }
  When: auction_snapshot(&state) is called
  Then: Some(AuctionSnapshot) where
        card_id == CardId(7)
        last_accepted_bid == 0   // NOT 3 (starting_price)
        current_leader == None
        timer_remaining_ms == 12000
  Edge cases: timer_remaining_ms = 0 (timer just hit zero) — snapshot still returns Some
              (RESOLVING fires in step 5 of same tick, not here)
```

**AC AU10-c — snapshot returns Some with correct bid and leader (bids placed):**
```
Test: snapshot reflects last accepted bid and leader
  Given: AuctionState { phase: LiveBidding, card_id: Some(CardId(3)),
                        current_price: 7, current_leader: Some(PlayerId(1)),
                        timer_remaining_ms: 5500 }
  When: auction_snapshot(&state) is called
  Then: last_accepted_bid == 7
        current_leader == Some(PlayerId(1))
        timer_remaining_ms == 5500
```

**AC AU10-d — timer value is not transformed:**
```
Test: timer_remaining_ms passes through unmodified
  Given: AuctionState with timer_remaining_ms = T (various values: 0, 1, 19999, 20000)
  When: auction_snapshot(&state) is called for each T
  Then: result.timer_remaining_ms == T in each case
```

**AC AU10-e — AuctionState::default() is Idle with zeroed fields:**
```
Test: default state is Idle
  Given: AuctionState::default()
  When: each field is inspected
  Then: phase == AuctionPhase::Idle
        card_id == None
        current_price == 0
        current_leader == None
        timer_remaining_ms == 0
```

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/auction/auction_state_scaffold_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `workspace-and-shared-types` story-002 DONE (provides `CardId`, `PlayerId` in `shared/protocol.rs`)
- Unlocks: Story 002 (Auction Phase Entry), Story 003 (AbortAuction Handler)
