# Epic: Auction System

> **Layer**: Feature (M2)
> **GDD**: design/gdd/auction-system.md
> **Architecture Module**: `server/feature/auction/` (`state.rs`, `snapshot.rs`, `system.rs`, `plugin.rs`)
> **Status**: Ready
> **Stories**: Not yet created — run `/create-stories auction-system`

## Overview

Implements the server-authoritative state machine for the game's signature mechanic: an open ascending auction that runs every third round (DRAFT_AUCTION phase). The epic owns `AuctionState` — a single Bevy `Resource` holding phase, current card, price, leader, and timer — along with `auction_tick_system`, the sole writer of that resource. Per-frame execution order within `auction_tick_system` is strictly enforced by code order: inbound control messages first (`AuctionPhaseEntered`, `AbortAuction`), then Lightyear C2S bid drain (`MessageReceiver<C2SAuctionBid>`), then timer decrement with a 1000ms lag-spike clamp, then RESOLVING transition and settlement. Economy gold operations (reserve, release, spend) are invoked via `api.rs` functions on `ResMut<PlayerEconomies>` to enforce the release-before-reserve invariant atomically within one system run. The `auction_snapshot()` function is a pure function on `&AuctionState`, used by the reconnect handler. The system is scheduled via `AuctionSet::Tick.before(RsmSet::Tick)` so that `AuctionSettled` is readable by the RSM in the same frame.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-013: Auction System State Machine and Bid Processing Architecture | Single `AuctionState` resource + single `auction_tick_system` sole writer; bid drain before timer decrement via code order; release-before-reserve atomicity via sequential `api.rs` calls within one function body; `auction_snapshot()` as pure `&AuctionState` function | HIGH |

## Engine Notes

- **`MessageReceiver<C2SAuctionBid>`** is Lightyear's C2S receiver — distinct from Bevy's `MessageReader<T>`. Do NOT use `MessageReader<C2SAuctionBid>` for Lightyear network messages.
- **Timer cast**: use `u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` — not `as u32`.
- **`liv-bevy-018` + `liv-bevy-lightyear`** skills are mandatory on every `.rs` file in this epic.
- **Verification required** (from ADR-013): confirm Lightyear 0.26 C2S receiver system param is `MessageReceiver<T>` with `receiver.receive_messages()` before implementing the bid drain loop. Abstract behind a helper returning `impl Iterator<Item = C2SAuctionBid>` to keep bid-validation logic testable regardless of Lightyear API shape.

## Pre-Implementation Gates

Confirm all four before any story begins:

1. **CardSource naming** — Reconcile `CardSource::AuctionWon` vs `AcquisitionSource` enum name in `network-protocol.md` (`S2CCardAcquired` source field)
2. **RSM AbortAuction trigger** — RSM GDD must document `auction_max_duration_seconds` safety timeout that triggers `AbortAuction`
3. **OQ9 resolution** — Confirm whether LIVE_BIDDING with `timer_remaining_ms == 0` is a reachable state before implementing AU12 (`AuctionExpired` rejection)
4. **`spend_reserved_gold` API name** — Verify exact function name in `economy-system` before story implementation (OQ4)

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-AUC-001 | State machine lifecycle: IDLE guard; AuctionPhaseEntered → LIVE_BIDDING; non-IDLE duplicate → logged error, no state change | ADR-013 ✅ |
| TR-AUC-002 | Announcement ordering: S2CAuctionCard enqueued before S2CPhaseChanged(DRAFT_AUCTION) server-side | ADR-013 ✅ |
| TR-AUC-003 | Bid validation: 5-condition gate (phase+timer, amount, not-leader, gold, hand); BidRejectedReason unicast on rejection | ADR-013 ✅ |
| TR-AUC-004 | Gold reservation atomicity: release_prev before reserve_new within single system body; no two-player simultaneous reservation; S2CGoldBroadcast includes reserved_gold for both affected players | ADR-013 ✅ |
| TR-AUC-005 | Timer mechanics: Formula 3 reset on accepted bid; 1000ms tick-delta clamp; saturating_sub; bid drain before timer decrement | ADR-013 ✅ |
| TR-AUC-006 | Resolution: Case A winner (spend_reserved_gold + card to hand + S2CCardAcquired + S2CAuctionSettled{Some}); Case B no-bid (no gold + S2CAuctionSettled{None}); AuctionSettled Bevy Message in both cases | ADR-013 ✅ |
| TR-AUC-007 | Pool integration: draw_auction_card() + distribute() unconditional at draw; pool-empty immediate AuctionSettled{None}; Legendary excluded before legendary_pool_entry_round | ADR-013 ✅ |
| TR-AUC-008 | AbortAuction: release reservation + IDLE + no AuctionSettled; RESOLVING is uninterruptible | ADR-013 ✅ |
| TR-AUC-009 | Reconnect snapshot: auction_snapshot() pure fn on &AuctionState; None in IDLE; last_accepted_bid == 0 sentinel (not starting_price); snapshot system before auction_tick_system | ADR-013 ✅ |
| TR-AUC-010 | Same-tick + post-settlement ordering: duplicate same-tick bids first-wins; stale post-settlement bid silent discard; reserved_gold == 0 for all at SELECTING entry | ADR-013 ✅ |

## Epic Dependencies

This epic requires the following stories to be **DONE** before implementation begins:

| Dependency | Why |
|------------|-----|
| `economy-system` story-001 (State & Pure API Scaffold) | Provides `reserve_gold`, `release_gold_reservation`, `spend_reserved_gold`, `can_afford_bid` in `economy/api.rs` |
| `economy-system` story-005 (Auction Reservation & Bid Validation) | Confirms economy-side auction reservation logic is implemented and tested |
| `card-data-pool` story-001 (Pool State Core API) | Provides `draw_auction_card()` and `distribute()` |
| `round-state-machine` story-001 (State & Events Scaffold) | Provides `AuctionPhaseEntered` and `AbortAuction` Bevy Messages on the ADR-010 event bus |
| `workspace-and-shared-types` story-002 (Shared Card Types) | Provides `CardId`, `PlayerId`, `C2SAuctionBid`, `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionBidRejected`, `S2CAuctionSettled`, `S2CCardAcquired` in `shared/protocol.rs` |

## Scope

### Deliverables

**`server/src/feature/auction/state.rs`**
- `AuctionState` resource: `{ phase: AuctionPhase, card_id: Option<CardId>, current_price: u32, current_leader: Option<PlayerId>, timer_remaining_ms: u32 }`
- `AuctionPhase` enum: `Idle | Selecting | LiveBidding | Resolving`
- `impl Default for AuctionState` — starts in `AuctionPhase::Idle` with zero values

**`server/src/feature/auction/snapshot.rs`**
- `AuctionSnapshot` struct: `{ card_id: CardId, last_accepted_bid: u32, current_leader: Option<PlayerId>, timer_remaining_ms: u32 }` — `last_accepted_bid == 0` means no bids placed yet (sentinel; NOT starting_price)
- `pub fn auction_snapshot(state: &AuctionState) -> Option<AuctionSnapshot>` — pure function, returns `None` in Idle

**`server/src/feature/auction/system.rs`**
- `auction_tick_system` — sole `ResMut<AuctionState>` writer; sole drainer of `MessageReceiver<C2SAuctionBid>`
- Execution order within function body (code order enforces invariants):
  1. Handle `MessageReader<AuctionPhaseEntered>` — IDLE guard → SELECTING → `draw_auction_card()` → S2CAuctionCard → LIVE_BIDDING
  2. Handle `MessageReader<AbortAuction>` — release reservation if leader → IDLE, no AuctionSettled
  3. Drain `MessageReceiver<C2SAuctionBid>` (if LIVE_BIDDING) — validate 5 conditions; on accept: `release_gold_reservation(prev)` → `reserve_gold(new, amount)` → update state → reset timer → S2CAuctionBidAccepted; on reject: S2CAuctionBidRejected
  4. `timer_remaining_ms = timer_remaining_ms.saturating_sub(u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX).min(1000))`
  5. If LIVE_BIDDING and `timer_remaining_ms == 0` → RESOLVING → settlement (Case A or B) → `MessageWriter<AuctionSettled>` → IDLE

**`server/src/feature/auction/plugin.rs`**
- `AuctionPlugin`: registers `AuctionState` as resource (default); registers `auction_tick_system`; configures `AuctionSet::Tick.before(RsmSet::Tick)`; reconnect snapshot system scheduled before `auction_tick_system`

### Out of Scope

- Economy gold state and API (`economy/api.rs`) — owned by `economy-system` epic
- `draw_auction_card()` implementation — owned by `card-data-pool` epic
- `AuctionPhaseEntered` / `AbortAuction` message definitions — owned by `round-state-machine` epic
- Network message type definitions (`C2SAuctionBid`, `S2CAuction*`) — owned by `workspace-and-shared-types` epic
- Shop/Auction UI panel rendering — owned by Presentation layer `shop-auction-ui` epic [M2]
- Reconnect snapshot handler (system that calls `auction_snapshot()` and populates `S2CGameSnapshot.auction_state`) — owned by `game-session-system` epic

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All 23 BLOCKING Acceptance Criteria from `design/gdd/auction-system.md` (AU1-a through AU23, M7-a, M7-c, AU8-pool) verified by passing unit/integration tests
- All Logic stories have passing test files in `tests/unit/auction/`
- All Integration stories have passing test files in `tests/integration/auction/`
- `cargo check --workspace` green; zero warnings on `server/src/feature/auction/**`
- Code review gate: `ResMut<AuctionState>` appears in exactly one system (`auction_tick_system`)
- Code review gate: `MessageReceiver<C2SAuctionBid>` (or confirmed equivalent) appears in exactly one system
- `auction_tick_system` is scheduled before `rsm_tick_system` — verified by Bevy schedule graph dump
- CI grep gate: `grep -rE "EventWriter|EventReader|Events<|add_event" server/src/feature/auction/` returns zero matches

## Stories

Not yet created — run `/create-stories auction-system`
