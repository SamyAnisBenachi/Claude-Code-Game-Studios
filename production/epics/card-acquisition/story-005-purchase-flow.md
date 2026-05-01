# Story 005: Purchase Flow, Dead Slot, and CA18 Atomicity

> **Epic**: Card Acquisition
> **Status**: In Progress
> **Owner**: codex-ca-005-purchase-flow
> **Layer**: Feature (M2)
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-acquisition.md`
**Requirements**: `TR-CA-008`, `TR-CA-009`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-015: Card Acquisition Shop State Machine Architecture
**ADR Decision Summary**: Purchase flow executes as sequential calls within one system body: phase gate → `hand_len < 10` → `pool.is_available()` → `economy.spend_gold()` → `card_pool.distribute()`. If `distribute()` returns `Err(DistributeError::Exhausted)` after `spend_gold()` succeeded, `economy.refund_gold()` is called immediately in the same function body before returning — no cross-frame messaging, no `await`. Gold must never remain deducted after a failed distribute.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `MessageReceiver<C2SPurchaseCard>` — Lightyear C2S network receiver; confirm exact API with `liv-bevy-lightyear` skill
- All calls (`spend_gold`, `distribute`, `refund_gold`) are synchronous Rust function calls in the same system body — no Bevy command queue, no cross-frame messaging
- `Query::single()` returns `Result` in Bevy 0.16+ — not used here (Resources accessed directly)

**Control Manifest Rules (Feature layer — from ADR-015):**
- Required: `economy.refund_gold(player_id, cost)` called before ANY return path following `spend_gold()` + failed `distribute()` — mandatory CA18 invariant
- Required: Three pre-purchase checks executed in order: (1) phase gate, (2) `hand_len < 10`, (3) `pool.is_available(card_id)`
- Required: Rejected purchase leaves slot displayed (dead slot persists until manual refresh)
- Forbidden: Cross-frame message path between `spend_gold` and `refund_gold` — must be sequential in same function body

**Performance Budget**: Purchase processing must be O(queued purchase messages) per frame. Each message may inspect only the player's capped hand state, the three current shop slots, and constant-time economy/card-pool state needed for the selected card. Do not scan the full card pool, all players, or all hands in the gameplay-loop path.

---

## Acceptance Criteria

*From GDD `design/gdd/card-acquisition.md`, scoped to this story:*

- [ ] **CA13** — GIVEN card X sits in slot 1 with `copies_remaining=1` AND the Prism or Objective System concurrently distributes the last copy (`copies_remaining` → 0), WHEN the player attempts to purchase card X, THEN purchase is rejected (TOCTOU guard via `pool.is_available()`), gold unchanged, slot 1 remains displayed (dead slot).
- [ ] **CA14** — GIVEN a player purchases the card in slot 2 successfully, WHEN purchase completes, THEN card is in `player.hand`, gold decremented by `card_cost`, and slot 2 is no longer in the shop display.
- [ ] **CA18** — GIVEN all three Rule 6 checks pass AND `spend_gold()` succeeds AND `distribute()` returns `Err(DistributeError::Exhausted)` (injected fault), WHEN this occurs, THEN `refund_gold(player, cost)` is called returning gold to its pre-purchase value, card NOT added to hand, server-side error logged, slot remains displayed.
- [ ] **CA20** — GIVEN a player sends `C2SPurchaseCard` during DRAFT_SHOP with 0.1s remaining AND the DRAFT_SHOP timer expires before the server processes the message, THEN the purchase is silently discarded (phase transition wins) and gold is unchanged.

---

## Implementation Notes

*Derived from ADR-015 Decision — Purchase Atomicity section:*

**Purchase handler** (step 3 in tick system, `ShopActive` or `DraftInitial`):

```rust
for msg in purchase_messages.receive_messages() {
    let player_id = msg.client_id;
    let card_id = msg.card_id;

    // Phase gate (CA20): if phase != ShopActive/DraftInitial, discard
    // Hand check (CA2, CA13): if hand_len >= 10, reject
    // Availability check (CA13): if !pool.is_available(card_id), leave slot as dead slot
    // Spend:
    if economy.spend_gold(player_id, card_cost).is_ok() {
        match card_pool.distribute(card_id) {
            Ok(()) => {
                hands.push_card(player_id, card_id);
                shop_state.current_slots[slot_idx] = None;
                // send S2CShopSlots update
            }
            Err(DistributeError::Exhausted) => {
                // CA18 MANDATORY rollback — no system boundary between these lines
                economy.refund_gold(player_id, card_cost);
                error!("TOCTOU: distribute failed for {:?}, gold refunded", card_id);
                // Slot remains displayed — dead slot
            }
        }
    }
}
```

**CA13 TOCTOU setup**: Test fixture with `copies_remaining = 0` (pre-depleted) — simulates the race where another system distributed the last copy before the purchase was processed. `pool.is_available()` returns `false`. Purchase rejected. Slot stays.

**CA18 fault injection**: For OQ4 — the test approach is to use an explicit error-injection path. Either (a) a `CardPool` test double that returns `Err(DistributeError::Exhausted)` unconditionally, or (b) the real `PlayerPool` with `copies_remaining = 0` bypassing `is_available` (should not happen in production but is the inject path for testing CA18). The key assertion is: gold after == gold before.

**CA20 phase race**: The phase gate check uses `shop_state.phase` — by the time the timer fires, `rsm_tick_system` has advanced the phase, writing a `ShopRefreshTriggered` (or equivalent phase-close signal). Since `card_acquisition_tick_system` runs **after** `rsm_tick_system` (ADR-015 scheduling), by the time CA processes the stale `C2SPurchaseCard`, the phase is already `Inactive`. The phase gate discards it.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: `PlayerHands`, `ShopStates` definitions
- Story 003: Slot fill pipeline (this story only removes a slot after successful purchase)
- Story 004: Manual refresh cost (a separate `C2SRefreshShop` path)
- Story 006: External bypass (Prism/Objective) write path to `PlayerHands`

---

## QA Test Cases

- **CA13**: TOCTOU dead slot
  - Given: `current_slots[0] = Some(card_x)`; `pool.is_available(card_x) == false` (copies exhausted)
  - When: `C2SPurchaseCard { card_id: card_x }` processed in `ShopActive`
  - Then: economy.gold unchanged; `hand.len()` unchanged; `current_slots[0] == Some(card_x)` (slot still displayed)
  - Edge cases: all 3 slots are dead simultaneously

- **CA14**: Successful purchase removes slot
  - Given: `current_slots[1] = Some(card_y)`; pool has `copies_remaining >= 1`; economy has sufficient gold; hand.len() < 10
  - When: `C2SPurchaseCard { card_id: card_y }` processed
  - Then: `hands.hand_len() == prior_len + 1`; economy.gold decreased by `card_cost`; `current_slots[1] == None`
  - Edge cases: buying the only copy (copies_remaining → 0 after)

- **CA18**: Rollback on distribute failure
  - Given: economy.spend_gold succeeds (gold sufficient); `distribute()` returns `Err(DistributeError::Exhausted)` (injected)
  - When: CA18 code path executes
  - Then: economy.gold == gold_before_purchase (refund confirmed); `hands.hand_len()` unchanged; `current_slots` unchanged (dead slot); error log entry present
  - Edge cases: refund amount exactly matches spend amount; refund called exactly once

- **CA20**: Phase transition discards stale purchase
  - Given: `ShopRefreshTriggered { trigger: ShopOpen }` already processed (phase advanced to `Inactive` or next state) this frame; stale `C2SPurchaseCard` arrives
  - When: CA tick processes the message after RSM has already closed the DRAFT_SHOP phase
  - Then: economy.gold unchanged; hand unchanged; no S2C message staged
  - Edge cases: multiple stale messages in the same frame

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/card_acquisition/purchase_atomicity_test.rs` — must exist and pass
*(Integration test must inject `Err(DistributeError::Exhausted)` via pool test double and assert gold is fully restored)*

**Status**: [x] Created and verified locally with `cargo test -p server --test card_acquisition_purchase_atomicity_test` on 2026-05-01

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) and Story 003 (`draw-pipeline`) must both be Done — purchase flow reads `ShopStates` and modifies `PlayerHands`; `current_slots` updated during purchase depend on draw pipeline having populated them
- Unlocks: None — this story is the last logic story; the epic definition of done requires all CA stories to be complete
