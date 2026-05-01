# Story 001: State Scaffold — ShopStates, PlayerHands, Phase Machine

> **Epic**: Card Acquisition
> **Status**: Complete
> **Layer**: Feature (M2)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/card-acquisition.md`
**Requirements**: `TR-CA-001`, `TR-CA-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-015: Card Acquisition Shop State Machine Architecture
**ADR Decision Summary**: `ShopStates` + `PlayerHands` are server-only `Resource`s. A single system — `card_acquisition_tick_system` — is the sole `ResMut<ShopStates>` writer and sole drainer of the two Lightyear C2S receivers. `ShopPhase` provides an explicit phase gate that discards C2S messages received in the wrong phase.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `#[derive(Resource, Default)]` — no `#[derive(Reflect)]` needed for server-only logic resources
- `HashSet<CardId>` inside a `Resource` compiles without `#[derive(Reflect)]` or `#[derive(Clone)]`
- `#[derive(Message, Clone)]` for `ShopRefreshTriggered` — Bevy 0.17+ Message/Event split; `EventWriter`/`EventReader` do NOT exist in 0.18
- `MessageReader<T>` is Bevy's internal bus; `MessageReceiver<T>` is Lightyear's C2S network API — never confuse them
- `app.add_message::<ShopRefreshTriggered>()` required in plugin setup

**Control Manifest Rules (Feature layer — from ADR-015):**
- Required: `ResMut<ShopStates>` appears in exactly one system — code review gate on every CA PR
- Required: `MessageReceiver<C2SPurchaseCard>` and `MessageReceiver<C2SRefreshShop>` each drained by exactly one system
- Required: `CardAcquisitionSet::Tick.after(RsmSet::Tick)` — RSM produces `ShopRefreshTriggered`; CA consumes it in the same frame
- Forbidden: No `EventWriter` / `EventReader` (removed in Bevy 0.17) — use `MessageWriter` / `MessageReader`
- Guardrail: `PlayerHands` written by CA in DRAFT only; Prism and Objective write in RESOLUTION only — never in same phase

---

## Acceptance Criteria

*From GDD `design/gdd/card-acquisition.md`, scoped to this story:*

- [ ] **CA1** — GIVEN a player's hand has 9 cards, WHEN they purchase a card during DRAFT_SHOP, THEN `hand.len() == 10` and gold is decremented by `card_cost`.
- [ ] **CA2** — GIVEN a player's hand has 10 cards, WHEN they attempt to purchase any card, THEN purchase is rejected, gold unchanged, slot remains displayed and re-attemptable.
- [ ] **CA7** — GIVEN a player is in DRAFT_AUCTION state (`ShopPhase::AuctionLock`), WHEN they send `C2SPurchaseCard` or `C2SRefreshShop`, THEN both are silently discarded by the server, gold unchanged, hand unchanged, no S2C error response.

---

## Implementation Notes

*Derived from ADR-015 Decision and Key Interfaces:*

Define these three files in `server/src/card_acquisition/` (or `server/feature/acquisition/`):

**`state.rs`** — `ShopStates` resource:
```rust
#[derive(Resource, Default)]
pub struct ShopStates {
    pub players: HashMap<PlayerId, PlayerShopState>,
}

#[derive(Default)]
pub struct PlayerShopState {
    pub phase: ShopPhase,
    pub displayed_this_draft: HashSet<CardId>,
    pub current_slots: [Option<CardId>; 3],
    pub refresh_count_this_draft: u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopPhase {
    #[default] Inactive,
    DraftInitial,
    AuctionLock,
    ShopActive,
}
```

**`hands.rs`** — `PlayerHands` resource:
```rust
#[derive(Resource, Default)]
pub struct PlayerHands {
    pub hands: HashMap<PlayerId, Vec<CardId>>,
}
impl PlayerHands {
    pub fn hand_len(&self, player: PlayerId) -> usize { ... }
    pub fn push_card(&mut self, player: PlayerId, card_id: CardId) { ... }
}
```

**`messages.rs`** — `ShopRefreshTriggered` Bevy Message:
```rust
#[derive(Message, Clone)]
pub struct ShopRefreshTriggered { pub player_id: PlayerId, pub trigger: ShopRefreshTrigger }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopRefreshTrigger { DraftInitial, AuctionLock, ShopOpen, ShopUnlock }
```

**`plugin.rs`** — register resources + schedule:
```rust
app.init_resource::<ShopStates>();
app.init_resource::<PlayerHands>();
app.add_message::<ShopRefreshTriggered>();
app.configure_sets(Update, CardAcquisitionSet::Tick.after(RsmSet::Tick));
app.configure_sets(Update, CardAcquisitionSet::Tick.after(AuctionSet::Tick));
```

**Phase gate for CA7**: `card_acquisition_tick_system` drains `MessageReceiver<C2SPurchaseCard>` and `MessageReceiver<C2SRefreshShop>` in ALL phases — but only routes to handlers when `phase == ShopActive` or `phase == DraftInitial`. In `AuctionLock` and `Inactive`, drain-and-discard both receivers silently.

**CA1/CA2 enforcement**: The `hand_len() < 10` check is the second pre-purchase check (after phase gate). If false: return immediately, no state change.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: Draft Initial draw logic (`draw_initial_draft`, `S2CDraftOffering`)
- Story 003: Shop draw pipeline (slot fill, dedup, 50/50 split)
- Story 004: Manual refresh cost formula and counter reset
- Story 005: Full purchase flow (spend_gold, distribute, CA18 rollback)
- Story 006: External bypass writes to `PlayerHands` from Prism/Objective

---

## QA Test Cases

- **CA1**: Hand at 9, purchase attempt
  - Given: `PlayerHands` has 9 cards for player; `ShopStates` in `ShopActive`; slot contains a valid `card_id`; economy has sufficient gold
  - When: CA tick system processes `C2SPurchaseCard { card_id }`
  - Then: `hands.hand_len(player) == 10`; economy.gold decremented by `card_cost`
  - Edge cases: hand at exactly 9 (boundary); purchase of last card in hand

- **CA2**: Hand at 10, purchase rejected
  - Given: `PlayerHands` has 10 cards for player; `ShopStates` in `ShopActive`; slot valid
  - When: CA tick system processes `C2SPurchaseCard { card_id }`
  - Then: `hands.hand_len(player) == 10` (unchanged); economy.gold unchanged; slot still in `current_slots`
  - Edge cases: hand exactly at cap; multiple rejection attempts

- **CA7**: AuctionLock discards both C2S message types
  - Given: `PlayerShopState.phase == ShopPhase::AuctionLock`
  - When: CA tick system receives `C2SPurchaseCard` AND `C2SRefreshShop` in the same frame
  - Then: `PlayerHands` unchanged; `ShopStates` unchanged; economy unchanged; no S2C message staged
  - Edge cases: both message types in the same frame; `Inactive` phase (same discard behaviour)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/card_acquisition/state_scaffold_test.rs` — must exist and pass

**Status**: [x] Verified - `cargo test -p server --test card_acquisition_state_scaffold_test` passed on 2026-05-01

---

## Dependencies

- Depends on: None — this is the foundational story for the epic
- Unlocks: Stories 002, 003, 005, 006 (all depend on `ShopStates` + `PlayerHands` existing)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 3/3 passing (CA1, CA2, CA7)
**Deviations**:
- Advisory: story manifest v2026-04-30 is older than current control manifest v2026-05-01.
- Advisory: implementation orders `CardAcquisitionSet::Tick` after concrete systems `advance_phase` and `auction_tick_system`; the story/control text names `RsmSet::Tick` and `AuctionSet::Tick`, which do not exist in the current codebase. Behavior matches the intended scheduling order.
**Test Evidence**: Logic unit test at `tests/unit/card_acquisition/state_scaffold_test.rs`; `cargo test -p server --test card_acquisition_state_scaffold_test` passed 6/6.
**Code Review**: Skipped - Lean mode.
