# Story 005: Auction Reservation & Bid Validation

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-??? (covers TR-ECO-08: auction bid validation and gold reservation; EC21: bid affordability check; EC22: hand-full bid rejection; EC23: shop affordability with active reservation)

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: `reserve_gold`, `release_gold_reservation`, `can_afford_bid`, and `can_afford_shop` are pure API functions in `economy/api.rs` (defined in Story 001). This story authors the system-layer validation entry points that the Auction System (M2) will call: a `validate_auction_bid` function that checks both affordability (`can_afford_bid`) and the hand-full rule (`hand_size < 10`). The `SpendError::HandFull` variant is returned when `hand_size == 10`. Auction System (M2) is the caller; Economy System is the validator.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: This story adds no new Bevy systems — it extends the pure-function API in `economy/api.rs`. All functions are synchronous and take `&PlayerEconomy` or `&mut PlayerEconomy` parameters with no `EventReader`/`EventWriter` in scope. No post-cutoff API risk. `liv-bevy-018` mandatory on the file.

**Control Manifest Rules (Core layer)**:
- Required: `reserved_gold` never exceeds `gold`. Debug-mode assertion: `debug_assert!(economy.reserved_gold <= economy.gold)` after every `reserve_gold` call.
- Required: `reserve_gold` returns `Err(SpendError::InsufficientFunds)` (not panic) when `amount > gold - reserved_gold`.
- Required: `release_gold_reservation` uses `saturating_sub` — releasing an amount greater than `reserved_gold` clamps to 0; no panic, no negative values.
- Required: Hand-full check (`hand_size == 10`) is enforced in `validate_auction_bid`. This function is called by the Auction System. Economy owns the validation; Auction owns the call site.
- Forbidden: `reserved_gold` is not exposed as a public field — callers use `can_afford_bid` and `can_afford_shop` for reads; `reserve_gold`/`release_gold_reservation` for mutations.

---

## Acceptance Criteria

- [ ] `server/src/core/economy/api.rs` contains `validate_auction_bid` function:
  ```rust
  pub fn validate_auction_bid(
      economy: &PlayerEconomy,
      bid_amount: u32,
      hand_size: u32,
  ) -> Result<(), SpendError>
  ```
  - Returns `Err(SpendError::HandFull)` if `hand_size >= 10`
  - Returns `Err(SpendError::InsufficientFunds)` if `!can_afford_bid(economy, bid_amount)`
  - Returns `Ok(())` otherwise
- [ ] `reserve_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError>` (defined in Story 001, verified here):
  - Returns `Err(InsufficientFunds)` when `amount > gold - reserved_gold`
  - On `Ok`: `reserved_gold += amount`; `debug_assert!(reserved_gold <= gold)` passes
- [ ] `release_gold_reservation(economy: &mut PlayerEconomy, amount: u32)` (defined in Story 001, verified here):
  - Uses `saturating_sub`: `reserved_gold = reserved_gold.saturating_sub(amount)`; never panics regardless of input
- [ ] `can_afford_bid(economy: &PlayerEconomy, amount: u32) -> bool` returns `(gold - reserved_gold) >= amount` using `saturating_sub` for the difference
- [ ] `can_afford_shop(economy: &PlayerEconomy, cost: u32) -> bool` returns `(gold - reserved_gold) >= cost` — same formula as `can_afford_bid`, separate named function
- [ ] **EC21**: GIVEN `gold = 3`, `reserved_gold = 0`, WHEN `validate_auction_bid(economy, 5, hand_size=0)`, THEN returns `Err(InsufficientFunds)`; `gold` unchanged
- [ ] **EC22**: GIVEN `gold = 10`, `hand_size = 10`, WHEN `validate_auction_bid(economy, 1, hand_size=10)`, THEN returns `Err(HandFull)` (hand check fires before affordability check)
- [ ] **EC23**: GIVEN `gold = 8`, `reserved_gold = 5`, WHEN `can_afford_shop(economy, 4)`, THEN returns `false` (`8 - 5 = 3 < 4`); WHEN `can_afford_shop(economy, 3)`, THEN returns `true`
- [ ] Reservation lifecycle: GIVEN `gold = 10`, WHEN `reserve_gold(economy, 7)` (Ok); then `release_gold_reservation(economy, 7)`; THEN `reserved_gold = 0`
- [ ] Outbid release: GIVEN `reserved_gold = 5`, WHEN `release_gold_reservation(economy, 5)`, THEN `reserved_gold = 0`; then `can_afford_shop(economy, 8)` returns `true` (reservation released, full gold available)
- [ ] Auction win spend: GIVEN `gold = 8`, `reserved_gold = 5` (winning bid), WHEN `release_gold_reservation(economy, 5)` then `apply_spend_gold(economy, 5)` (using `apply_gold_award` with negative — or a separate `spend_gold` helper per M2 Auction System contract), THEN `gold = 3`, `reserved_gold = 0`
- [ ] `release_gold_reservation` overflow guard: GIVEN `reserved_gold = 2`, WHEN `release_gold_reservation(economy, 100)`, THEN `reserved_gold = 0` (no panic, saturating)
- [ ] `cargo check -p server` passes after all additions

---

## Implementation Notes

*Derived from EPIC.md §api.rs and economy-system.md Rule 7, Edge Cases:*

**`validate_auction_bid` is the Auction System's single call point:** The Auction System (M2) calls this function before processing a `C2SAuctionBid` message. It checks both the hand-full rule and gold sufficiency in one call, returning a typed error that the Auction System broadcasts back to the client via `S2CAuctionBidRejected`. Economy is the authority on both checks; Auction is only the messenger.

**Hand-full check ordering:** Per GDD Rule 7, hand-full check comes first. A player with a full hand and insufficient gold returns `HandFull`, not `InsufficientFunds` — the hand-full error is more actionable for the player (they need to play cards before re-bidding; adding gold won't help).

**Auction bid reservation flow (M2 wiring pattern):** The flow the Auction System implements using this story's API:
1. New highest bid arrives: `validate_auction_bid(economy, new_bid, hand_size)` → if `Ok`, `reserve_gold(economy, new_bid)`
2. Prior highest bidder outbid: `release_gold_reservation(prior_highest_bidder_economy, prior_bid_amount)`
3. Auction won: `release_gold_reservation(winner_economy, winning_bid)` then `apply_spend` equivalent (deduct gold) — the release-then-spend pattern ensures `reserved_gold` returns to 0 before the spend
4. Auction lost / timed out: `release_gold_reservation(all_active_bidders, their_reservation_amount)`

For M1 unit testing, this flow is tested with manual setup — no live Auction System.

**`apply_spend` for gold vs. mana:** The existing `apply_spend` API operates on mana pools. Gold deduction at auction win uses `apply_gold_award` with negative is not valid (u32 types). The correct pattern for auction win gold deduction is:
```rust
// In api.rs — add this function for gold spending (separate from mana spend):
pub fn spend_gold(economy: &mut PlayerEconomy, amount: u32) {
    economy.gold = economy.gold.saturating_sub(amount);
}
```
This aligns with the GDD's `spend_gold(player, bid_amount)` language. Add `spend_gold` to `api.rs` as part of this story. The M2 Auction System calls it after a win.

**`reserved_gold` visibility to clients:** The client's HUD shows raw `gold` (not `gold - reserved_gold`). The server is the only authority on the `reserved_gold` ceiling for shop validation. `S2CGoldUpdate` includes `reserved_gold` only if the HUD GDD requires it — check `design/ux/hud.md` before adding it to the message. Per the current GDD, `S2CGoldUpdate` is sufficient without `reserved_gold`.

---

## Out of Scope

- Auction System (M2): `C2SAuctionBid` message handling, bid acceptance, auction timer, `S2CAuctionBidRejected` — Auction System epic owns these; Economy only validates
- Story 006: Network dispatch wiring
- Shop refresh cost escalation counter — owned by Card Data & Pool epic; Economy validates the cost passed in, does not own the counter

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **EC21: Bid rejected when gold insufficient**
  - Given: `PlayerEconomy { gold: 3, reserved_gold: 0, .. }`
  - When: `validate_auction_bid(&economy, 5, 0)`
  - Then: Returns `Err(InsufficientFunds)`; economy state unchanged

- **EC22: Bid rejected when hand full (hand check before affordability)**
  - Given: `PlayerEconomy { gold: 100, reserved_gold: 0, .. }`
  - When: `validate_auction_bid(&economy, 1, 10)` (hand_size = 10)
  - Then: Returns `Err(HandFull)` — not `InsufficientFunds`

- **EC23: Shop purchase rejected when reservation reduces available gold**
  - Given: `PlayerEconomy { gold: 8, reserved_gold: 5, .. }`
  - When: `can_afford_shop(&economy, 4)`
  - Then: Returns `false` (effective gold = 3)
  - When: `can_afford_shop(&economy, 3)`
  - Then: Returns `true` (effective gold = 3 >= 3)

- **Reservation lifecycle: reserve → release → shop available**
  - Given: `gold = 10, reserved_gold = 0`
  - When: `reserve_gold(&mut e, 7)` → Ok; `can_afford_shop(&e, 4)` → false (10-7=3<4); `release_gold_reservation(&mut e, 7)`; `can_afford_shop(&e, 4)` → true (10>=4)
  - Then: All assertions pass

- **Release overflow safety**
  - Given: `reserved_gold = 2`
  - When: `release_gold_reservation(&mut e, 100)`
  - Then: `reserved_gold == 0`; no panic

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/economy/auction_reservation_test.rs` — covers EC21, EC22, EC23, reservation lifecycle, release overflow, bid sequence simulation
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (all API functions `reserve_gold`, `release_gold_reservation`, `can_afford_bid`, `can_afford_shop` defined; `SpendError` enum with `HandFull` variant exists)
- Note: This story adds `validate_auction_bid` and `spend_gold` to `api.rs` — these extend Story 001's module, no new files needed
- Unlocks: M2 Auction System (can implement bid handling knowing Economy API is complete and tested)
