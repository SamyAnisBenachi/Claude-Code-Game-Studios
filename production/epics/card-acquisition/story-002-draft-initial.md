# Story 002: Draft Initial — 9-Card Offering

> **Epic**: Card Acquisition
> **Status**: Complete
> **Layer**: Feature (M2)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/card-acquisition.md`
**Requirement**: `TR-CA-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-015: Card Acquisition Shop State Machine Architecture
**ADR Decision Summary**: On `ShopRefreshTrigger::DraftInitial`, the system calls `draw_initial_draft(class, 9, seed)` to obtain 9 distinct card IDs, populates `displayed_this_draft`, sets `phase = ShopPhase::DraftInitial`, and sends `S2CDraftOffering` via Lightyear reliable unicast. Manual refresh is rejected in `DraftInitial` by the phase gate.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `MessageReader<ShopRefreshTriggered>` — Bevy internal bus; NOT Lightyear's `MessageReceiver<T>`
- Lightyear reliable unicast for `S2CDraftOffering`: `server.send_message_to_target::<ReliableChannel, S2CDraftOffering>(msg, NetworkTarget::Single(client_id))` — verify exact API against `docs.rs/lightyear/0.26` via `liv-bevy-lightyear` skill before implementing
- CA21 tests unicast scoping; if Lightyear 0.26 send API is not yet verified in a `World`-based test, reclassify CA21 as an integration test requiring `App::new()` with Lightyear plugin

**Control Manifest Rules (Feature layer — from ADR-015):**
- Required: `S2CDraftOffering` sent on `ReliableChannel` via Lightyear unicast (ADR-008)
- Required: `draw_initial_draft()` called exactly once per DRAFT_INITIAL entry per player
- Forbidden: Manual refresh (`C2SRefreshShop`) in `DraftInitial` phase — silently discarded

---

## Acceptance Criteria

*From GDD `design/gdd/card-acquisition.md`, scoped to this story:*

- [x] **CA3** — GIVEN DRAFT_INITIAL begins, WHEN `draw_initial_draft()` completes and `S2CDraftOffering` is sent, THEN the offering contains exactly 9 distinct card IDs with no duplicates within the 9.
- [x] **CA4** — GIVEN a player has 5g at DRAFT_INITIAL and buys one 3g Rare, WHEN the timer expires, THEN the player's gold at round 1 DRAFT start is 2g (5−3 carried over; unused budget is not forfeited).
- [x] **CA5** — GIVEN DRAFT_INITIAL is active, WHEN `C2SRefreshShop` is received, THEN server silently discards the message, gold unchanged, `refresh_count_this_draft` unchanged, no S2C error response.
- [x] **CA21** — GIVEN DRAFT_INITIAL begins for a 2-player game, WHEN `S2CDraftOffering` is sent, THEN exactly one `S2CDraftOffering` is received by the target player and zero are received by the opponent (unicast verified at network layer).

---

## Implementation Notes

*Derived from ADR-015 Decision:*

**`ShopRefreshTrigger::DraftInitial` branch** in `card_acquisition_tick_system` step 1:

```rust
ShopRefreshTrigger::DraftInitial => {
    // 1. Draw 9 distinct cards via Pool API (calls draw_initial_draft)
    let offered = card_pool.draw_initial_draft(
        session_config.class_map[player_id],
        9,
        &mut server_rng,
    );
    // 2. Populate displayed_this_draft and current offering
    shop_state.displayed_this_draft.extend(offered.iter().copied());
    shop_state.phase = ShopPhase::DraftInitial;
    // 3. Send S2CDraftOffering via Lightyear reliable unicast
    //    VERIFY: exact Lightyear 0.26 send API before implementing
}
```

**CA4 — gold carry-over**: Unspent gold at DRAFT_INITIAL timer end is NOT reset. The Economy System does not zero gold on phase exit. Card Acquisition does not call `refund_gold` at timer expiry. Gold naturally carries over because the economy only debits on confirmed purchase (`spend_gold`).

**CA5 — refresh rejection in DraftInitial**: The main tick loop drains `MessageReceiver<C2SRefreshShop>` only when `phase == ShopActive`. When `phase == DraftInitial`, drain-and-discard `C2SRefreshShop` silently (step 4 of the system execution order). `refresh_count_this_draft` is NOT incremented.

**CA21 — unicast scope**: `S2CDraftOffering` must be sent with `NetworkTarget::Single(client_id)` (or Lightyear 0.26 equivalent). Do NOT broadcast. Use `liv-bevy-lightyear` skill to confirm the correct send method on the server side.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: `ShopStates` / `PlayerHands` resource definitions
- Story 003: Personal shop auto-refresh draw pipeline (3-slot, dedup, 50/50 split)
- Story 005: Full purchase flow including `is_available` check and CA18 rollback

---

## QA Test Cases

- **CA3**: 9 distinct cards in offering
  - Given: `draw_initial_draft()` returns a `Vec<CardId>` of length 9 (test fixture with ≥9 eligible cards)
  - When: `ShopRefreshTriggered { trigger: DraftInitial }` processed by tick system
  - Then: `shop_state.displayed_this_draft.len() == 9`; `S2CDraftOffering.card_ids` has exactly 9 unique entries
  - Edge cases: fewer than 9 cards in pool (returns however many exist — no padding)

- **CA4**: Unspent gold carries over
  - Given: player starts DRAFT_INITIAL with 5g; purchases one 3g card during DraftInitial
  - When: DRAFT_INITIAL timer expires / phase transitions to PLACEMENT
  - Then: `economy.gold(player) == 2` at next DRAFT entry (not reset)
  - Edge cases: zero purchases (5g carried); full spend (0g carried)

- **CA5**: C2SRefreshShop silently discarded in DraftInitial
  - Given: `shop_state.phase == ShopPhase::DraftInitial`
  - When: `C2SRefreshShop` injected into `MessageReceiver`
  - Then: `refresh_count_this_draft == 0` (unchanged); economy.gold unchanged; `displayed_this_draft` unchanged; no S2C message staged
  - Edge cases: multiple `C2SRefreshShop` messages in the same frame

- **CA21**: S2CDraftOffering is unicast, not broadcast
  - Given: 2-player session; both players connected
  - When: DRAFT_INITIAL fires for Player A
  - Then: Player A receives exactly one `S2CDraftOffering`; Player B receives zero
  - Edge cases: reconnect scenario — re-sent `S2CDraftOffering` still unicast to reconnecting player only

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/card_acquisition/draft_initial_test.rs` — must exist and pass

**Status**: [x] Verified locally with `cargo test -p server --test card_acquisition_draft_initial_test`

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) must be Done — `ShopStates`, `PlayerHands`, `ShopRefreshTriggered` must be defined
- Unlocks: None directly — Story 004 (refresh cost) also depends on Story 003; Story 005 (purchase) depends on Stories 001 + 003

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 4/4 passing (CA3, CA4, CA5, CA21)
**Deviations**: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; no blocking drift found. `TR-CA-002` currently lists CA3/CA4, while this story also closes CA5 and CA21 from the current Card Acquisition GDD.
**Test Evidence**: Integration evidence at `tests/integration/card_acquisition/draft_initial_test.rs`; `cargo test -p server --test card_acquisition_draft_initial_test` passed 5/5 tests.
**Code Review**: Skipped - lean review mode; local implementation review found no blocking GDD/ADR deviations.
