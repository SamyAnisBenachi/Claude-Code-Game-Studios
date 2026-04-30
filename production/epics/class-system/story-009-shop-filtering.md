# Story 009: Class Card Shop Filtering

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: `sessions.class_of(player_id)` (O(1) HashMap lookup) provides the `ClassId` parameter to `PlayerPool::draw_class_card(class, ...)` from ADR-006. Cross-class draw legality — `PlayerPool::draw_random(filter: &PoolFilter)` with `filter.class = None` — class filter not applied. No runtime play-time gate on `card_class` field: once a card is in a player's hand by any means, it is playable.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `ClassId` is already in `shared/src/card.rs` (ADR-006) — import from there, never redefine.
- `PlayerPool::draw_class_card(class: ClassId, ...)` API is defined by ADR-006; this story wires the ClassId parameter from `PlayerSessions` to that call.
- ADR-014 is NOT yet in the control manifest.

**Control Manifest Rules (Feature Layer)**:
- Required: All pool draw functions return `Option<T>` and never panic — ADR-006
- Required: `ClassId` imported from `shared::card::ClassId` — never redefine — ADR-014
- Forbidden: Never apply the shop's 50/50 class filter to triggered draws (Drheller, prism Lane 3) — GDD Rule (cross-class draw exception)
- Forbidden: Never block a card play at the server's play-acceptance gate based on `card_class != player.class` — GDD Rule (no runtime play gate)
- Guardrail: Card pool draw: O(1) HashMap lookup + CDF selection; negligible cost — ADR-006

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, Detailed Rules §Class card filtering boundary:*

- [ ] **CS-AC-26** GIVEN a player whose class is Sadida, WHEN shop slots are generated, THEN the class slot samples exclusively from the Sadida card library (25 cards); no Iop/Cra/Sacrier/Xelor/Ecaflip cards may appear in the class slot.
- [ ] **CS-AC-27** GIVEN a player triggers a Drheller DEATH draw, WHEN the draw resolves, THEN the drawn card may be any card from the full pool with no class filter applied (cross-class card in hand is legal and playable).
- [ ] **CS-AC-27b** GIVEN a player holds a cross-class card obtained via Drheller DEATH draw, WHEN that player attempts to play the cross-class card, THEN the server accepts the play (does not reject with any class-restriction error); the card's effect resolves normally.

---

## Implementation Notes

*Derived from ADR-014 Decision §5 ("Key Interfaces") and ADR-006:*

**Class slot generation** — in shop refresh system (Card Acquisition epic):
```rust
// Called by ShopRefreshTriggered subscriber (Card Acquisition epic)
let class_id = sessions.class_of(player_id);  // from PlayerSessions (Story 001)
let class_card = pool.draw_class_card(class_id, seed_from_rng);  // ADR-006 API
// → only Sadida cards returned if class_id == ClassId::Sadida
```

This story verifies the wiring between `sessions.class_of()` and `pool.draw_class_card()`. The shop generation system itself is implemented by Card Acquisition epic; this story adds the class filter integration test.

**Cross-class draw path** — Drheller DEATH trigger and prism Lane 3:
```rust
// No class filter — draw_random returns any card from the full pool
let drawn = pool.draw_random(&PoolFilter { class: None, .. }, seed_from_rng);
// Result may be any CardClass including one different from player.class
// Once in hand, the card is playable with no runtime gate
```

**No runtime play-time gate** — in C2SSubmitPlacement handler:
```rust
// Do NOT add this check:
// if card.card_class != player.class { return reject; }
// Cross-class cards in hand are ALWAYS accepted by the server.
// The only valid gate is that the card is in the player's hand (ownership check).
```

This is explicitly documented so implementers do not treat cross-class card play as a bug requiring a server-side filter.

**Boundary rule**: Class System defines *what cards qualify as a player's class library*; it does NOT define the shop-slot generation algorithm. The 50/50 class-vs-neutral roll, weighting, and refresh belong to Card Acquisition GDD. This story only wires the `ClassId` parameter — it does not reimplement the generation algorithm.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 001: `class_of()` implementation in PlayerSessions — must be DONE
- Card Data & Pool epic: `draw_class_card()` and `draw_random()` implementations (ADR-006)
- Card Acquisition epic: shop slot generation system, 50/50 class-vs-neutral weighting, refresh logic, ShopRefreshTriggered subscriber
- Prism System epic: Lane 3 draw (uniform pool, no class filter)

---

## QA Test Cases

*Logic story — automated test specs using `World::new()` with stubbed CardCatalog + PlayerPool.*

- **AC CS-AC-26 — Class slot samples only from player's class library**:
  - Given: `PlayerSessions` with Sadida-class player; `PlayerPool` containing Sadida class cards and neutral cards; `draw_class_card(ClassId::Sadida, seed)` returns a random card from Sadida pool
  - When: shop slot generation calls `sessions.class_of(sadida_player)` then `pool.draw_class_card(class_id, seed)` for 100 iterations with varying seeds
  - Then: every drawn card has `card.card_class == ClassId::Sadida`; no Iop/Cra/Sacrier/Xelor/Ecaflip cards returned
  - Edge cases: late-game Sadida class pool exhausted → `draw_class_card` returns `None`; calling code handles `None` gracefully (slot remains empty or falls back per Card Acquisition logic)

- **AC CS-AC-27 — Drheller DEATH draw bypasses class filter**:
  - Given: Cra-class player; Drheller DEATH trigger fires; `pool.draw_random(&PoolFilter { class: None, .. }, seed)` called
  - When: draw resolves
  - Then: returned card may be any `CardClass` (Iop, Cra, Sadida, etc. — all legal); drawn card added to player's hand
  - Verify: the Drheller trigger path does NOT call `draw_class_card(ClassId::Cra, ...)` — it must call `draw_random` with `class: None`

- **AC CS-AC-27b — Cross-class card in hand is playable**:
  - Given: Cra player's hand contains an Iop card (obtained via Drheller draw)
  - When: C2SSubmitPlacement containing the Iop card is processed by server
  - Then: server does NOT reject with class-restriction error; card accepted; effect resolves normally
  - Verify: `C2SSubmitPlacement` handler has NO `card_class != player.class` rejection branch

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/shop_filtering_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (PlayerSessions + `class_of()`) — must be DONE
- Depends on: `card-data-pool` epic story-001 (CardCatalog + `draw_class_card` API in PlayerPool — ADR-006) — must be DONE
- Depends on: `card-acquisition` epic (shop slot generation system that calls `draw_class_card`) — must be DONE for integration; unit test can stub the pool
- Unlocks: No direct downstream story; completes class-to-shop integration bridge
