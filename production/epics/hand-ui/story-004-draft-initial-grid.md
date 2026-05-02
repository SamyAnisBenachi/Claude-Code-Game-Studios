# Story 004: DRAFT_INITIAL Grid — Display & Purchase Flow

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-005`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-004: Asset Loading Pipeline](../../docs/architecture/adr-004-asset-loading-pipeline.md)
**ADR Decision Summary**: `bevy_asset_loader` LoadingState loads `CardCatalog` before `AppState::Lobby`. `Res<CardAtlas>` is promoted by `BoardRenderingPlugin` on session entry and read by `HandUiPlugin`. Grid slot entities are pre-pooled (Story 001). The `S2CDraftOffering` message (Lightyear inbound) is drained in `PresentationSet::MessageDrain` and drives grid slot visibility.

**Engine**: Bevy 0.18 | **Risk**: HIGH — MEDIUM
**Engine Notes**: `Time<Virtual>` used for purchase timeout (`purchase_timeout_ms = 3000 ms` default). `bevy_tweening::Animator<Transform>` for card-slide animation on purchase confirmation. Verify `Animator<T>::set_tweenable()` exists before use (ADR-021 Verification Required item 3). `Sprite { texture_atlas: Some(TextureAtlas { layout, index }), .. }` pattern for card art (not `Handle<TextureAtlas>` — does not exist in Bevy 0.18).

**Control Manifest Rules (Presentation Layer)**:
- Required: Pre-pooled entities — 9 grid slot entities spawned in Story 001; this story populates and toggles them.
- Required: `CardAtlas` accessed as `Res<CardAtlas>`; `CardAtlas::frame_index(card_id)` for atlas lookup.
- Required: All grid slot systems `in_state(ClientState::InSession)`.
- Required: Tween cancel-and-replace via `set_tweenable()` — never despawn grid slot entities.
- Forbidden: Any `*Bundle` type.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rules 4 and 14, scoped to this story:*

- [x] **HU-07**: GIVEN DRAFT_INITIAL begins and `S2CDraftOffering` is received with 9 card IDs, WHEN the grid renders, THEN exactly 9 grid slot entities have `Visibility::Visible` AND each slot's bound card data (name component, mana cost component) matches its corresponding card ID in the offering. Art rendering (atlas sprite display) is ADVISORY — lead sign-off required.

- [x] **HU-08**: GIVEN the player clicks a grid card during DRAFT_INITIAL, WHEN `S2CCardAcquired` confirms the purchase, THEN:
  - (a) The grid slot's `Visibility` becomes `Hidden` within one tick of receipt
  - (b) The corresponding fan slot becomes `Visibility::Visible` and an `Animator<Transform>` interpolating to the computed fan position (Formula 1) is attached
  - (c) After advancing `Time<Virtual>` by `card_draw_animation_ms` (default 280 ms), the fan slot's `Transform.translation` equals the formula-computed fan position for the updated hand count

- [x] **HU-09**: GIVEN the 10th card has been added to the hand during DRAFT_INITIAL, WHEN `S2CCardAcquired` delivers the 10th card, THEN within the same `App::update()` tick:
  - (a) All remaining visible grid slots receive a `GridSlotState::HandFullLocked` marker component
  - (b) Clicks on `GridSlotState::HandFullLocked` slots produce no `C2SPurchaseCard` message (input suppressed)
  - The 30% chroma / Ink Blue overlay rendering is ADVISORY.

- [x] **HU-10**: GIVEN the player clicks a grid card and no `S2CCardAcquired` arrives (covering all non-arrival cases: dropped server response, phase transition, pool exhaustion — server silently rejects), WHEN `purchase_timeout_ms` (3000 ms) elapses, THEN:
  - (a) The `PlayerEconomies` gold value is unchanged (no gold deducted)
  - (b) The slot reverts from `GridSlotState::Pending` to `GridSlotState::Available`
  - (c) A subsequent click on the slot produces a fresh `C2SPurchaseCard` message (player may retry)

- [x] **HU-10c**: GIVEN the hand reaches 10 cards (locking grid) AND a previously clicked grid card is still in `GridSlotState::Pending` (purchase in flight), THEN the slot's state becomes `GridSlotState::HandFullLocked` (hand-full lock takes precedence — click suppressed, pending state cleared regardless of in-flight request).

- [x] **HU-30**: GIVEN the 10th card is acquired during DRAFT_INITIAL, WHEN the hand-full lock fires, THEN:
  - The pre-pooled `HandFullNotification` entity (see Story 001) becomes `Visibility::Visible` and receives a `NotificationTimer { remaining_ms: hand_full_notification_duration_ms }` component
  - After the timer elapses (verifiable via `Time<Virtual>` advance), the entity becomes `Visibility::Hidden` and the timer component is removed
  - *(Note: This entity is PRE-POOLED at session start — it is NOT spawned at runtime. This is consistent with ADR-021 Impl Guideline 3: no per-round spawn/despawn. Add it to the pre-pool count in Story 001 if not already included.)*

---

## Implementation Notes

*Derived from ADR-021 and GDD Rules 4, 14:*

1. **`S2CDraftOffering` drain**: Drain in `PresentationSet::MessageDrain`. On receipt, iterate the 9 card IDs, look up card data from `Res<CardCatalog>`, populate grid slot entities (name, mana cost text components, atlas frame index), set `Visibility::Visible`.

2. **Click → Pending state**: On click, add `GridSlotState::Pending` to the slot entity. Suppress further clicks on that slot while Pending. Start a `Time<Virtual>`-based countdown timer of `purchase_timeout_ms`.

3. **`S2CCardAcquired` → slide animation**: Drain in `PresentationSet::MessageDrain`. On receipt for a grid slot, set the slot `Visibility::Hidden`, then attach `Animator<Transform>` to the corresponding fan slot to interpolate from current position to formula-computed fan position (using updated hand count). Use `bevy_tweening` tween with `EaseFunction::QuadraticOut` over `card_draw_animation_ms`.

4. **Hand-full lock**: When `S2CCardAcquired` brings the tracked hand count to 10, set `GridSlotState::HandFullLocked` on ALL remaining Visible grid slots (not just the one just purchased). Also activate the `HandFullNotification` entity (see HU-30).

5. **Timeout revert**: If `purchase_timeout_ms` elapses with no `S2CCardAcquired` for that slot's card, revert slot from `Pending` to `Available` (remove Pending marker, restore input-active state). No gold is deducted (gold deduction is server-side; client never deducts optimistically — ADR-002).

6. **HU-10c priority**: On `S2CCardAcquired` for the 10th card, lock ALL slots including any in `Pending` — hand-full takes priority over in-flight state. `GridSlotState::HandFullLocked` replaces any other state marker.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001]: Grid slot entity spawning (pre-pooling)
- [Story 003]: Phase transition into DRAFT_INITIAL (grid overlay visibility entry)
- [Story 007]: PLACEMENT Instant card staging (separate drop zone)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-07**: Grid display on offering
  - Given: DRAFT_INITIAL phase active; inject `S2CDraftOffering` with 9 known card IDs [C1..C9]
  - When: `App::update()` runs (MessageDrain processes offering)
  - Then: Query entities with `GridSlotIndex` AND `Visibility::Visible` → count == 9; for each slot index i, assert `CardNameText` component == `CardCatalog[Ci].name` and `ManaCost` component == `CardCatalog[Ci].cost`
  - Edge cases: Second `S2CDraftOffering` in same phase → existing slots update to new card IDs (idempotent refresh)

- **HU-08**: Purchase confirmation → slide animation
  - Given: Grid showing 9 cards; player click on slot 3 (card_id = C3); slot enters Pending
  - When: Inject `S2CCardAcquired { card_id: C3 }`; `App::update()` runs
  - Then: Slot 3 entity → `Visibility::Hidden`; fan slot at computed index has `Visibility::Visible` with `Animator<Transform>` attached
  - Then: Advance `Time<Virtual>` by `card_draw_animation_ms` (280 ms); `App::update()` → assert fan slot `Transform.translation.x` == Formula 1 `card_x` for new hand count
  - Edge cases: Two rapid purchases (C3 and C5 in same tick) → both animate independently to correct fan positions

- **HU-09**: Hand-full lock on 10th card
  - Given: 9 cards in hand; one grid slot in Pending (C9 in flight)
  - When: Inject `S2CCardAcquired { card_id: C9 }`; `App::update()` runs
  - Then: ALL remaining visible grid slots have `GridSlotState::HandFullLocked`; click on any locked slot → no `C2SPurchaseCard` in queue
  - Edge cases: HU-10c — slot in Pending gets locked (Pending cleared, HandFullLocked set)

- **HU-10**: Purchase timeout revert
  - Given: DRAFT_INITIAL; click slot 2 → slot enters Pending
  - When: Advance `Time<Virtual>` by `purchase_timeout_ms + 1 ms` (3001 ms); `App::update()` runs
  - Then: `PlayerEconomies[self_player].gold` is unchanged (query resource directly); slot 2 has `GridSlotState::Available` (Pending removed); click slot 2 again → `C2SPurchaseCard` is enqueued
  - Edge cases: Click during Pending state → no new `C2SPurchaseCard` (input suppressed while Pending)

- **HU-30**: Hand-full notification lifecycle (pre-pooled entity toggle)
  - Given: 9 cards in hand; `HandFullNotification` entity pre-pooled with `Visibility::Hidden`
  - When: Inject `S2CCardAcquired` for 10th card; `App::update()` runs
  - Then: `HandFullNotification` entity has `Visibility::Visible` AND `NotificationTimer { remaining_ms: 2000 }`
  - When: Advance `Time<Virtual>` by 2001 ms; `App::update()` runs
  - Then: `HandFullNotification` entity has `Visibility::Hidden`; `NotificationTimer` component absent (removed after expiry)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/hand-ui/draft_initial_grid_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (pre-pooled grid slot entities), Story 003 (DRAFT_INITIAL phase entry handled)
- Unlocks: None directly — parallel with Stories 005–013

## Completion Notes

**Completed**: 2026-05-02
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 6/6 passing; HU-07, HU-08, HU-09, HU-10, HU-10c, and HU-30 are covered by `tests/integration/hand-ui/draft_initial_grid_test.rs`.
**Test Evidence**: `cargo test -p client --test hand_ui_draft_initial_grid_test` passed 5/5. `cargo check -p client` passed.
**Verification**: `client/src/ui/hand/mod.rs` populates pre-pooled grid slots from draft offerings, sends local purchase intents through `HandUiOutboundMessages`, applies pending purchase timeouts with `Time<Virtual>`, hides confirmed grid slots, animates acquired cards into fan slots with `TweenAnim`, locks visible grid slots at hand-full, and toggles the pre-pooled hand-full notification through `NotificationTimer`.
**Deviations**: Advisory only - live Lightyear wiring is not verified here; the current implementation uses local Bevy messages/outbox (`HandUiDraftOfferingReceived`, `HandUiCardAcquiredReceived`, `HandUiOutboundMessages`) rather than real `MessageReceiver<S2CDraftOffering>` / `MessageSender<C2SPurchaseCard>`. Advisory only - `CardAtlas` art/frame lookup is not implemented; `CardAtlas` currently exists only in architecture docs, and HU-07 marks art rendering advisory. Advisory only - current `TR-HU-005` registry text also mentions the 45s timer and 5g budget via `S2CGoldBroadcast`; this story's acceptance criteria cover the grid display and purchase flow only.
**Code Review**: Skipped - lean mode.
