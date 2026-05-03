# Story 005: PLACEMENT Entry — Submit Button & Core Stage/Unstage

> **Epic**: Hand UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md)
**ADR Decision Summary**: Client is a read-only view. No optimistic updates — staging is local pending state only; actual play is committed by `C2SSubmitPlacement` only when Submit fires after pre-validation. `GhostPlacementChanged` is a Bevy-internal message (NOT a Lightyear message) written by Hand UI and read by Board Rendering.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `MessageWriter<GhostPlacementChanged>` / `MessageReader<GhostPlacementChanged>` — Bevy-internal buffered messages registered via `app.add_message::<GhostPlacementChanged>()`. NOT Lightyear's `MessageSender`/`MessageReceiver`. `C2SSubmitPlacement` uses Lightyear's `MessageSender<C2SSubmitPlacement>`. The drag sprite is a bevy_ui `Node` — not a world-space `Sprite` — to preserve z-order above board during drag (ADR-021 Impl Guideline 8).

**Control Manifest Rules (Presentation Layer)**:
- Required: Drag-sprite preview is bevy_ui `Node` — NOT world-space `Sprite`.
- Required: `GhostPlacementChanged` is a Bevy-internal `#[derive(Message)]` — NOT Lightyear. Use `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()`.
- Required: No optimistic client updates — `C2SSubmitPlacement` is sent exactly once, after pre-validation passes.
- Required: All placement systems `in_state(ClientState::InSession)`.
- Forbidden: Double-submit — once Submit button enters `Inactive`, no second `C2SSubmitPlacement` is written.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md` Rules 6, 8, 10, scoped to this story:*

- [ ] **HU-11**: GIVEN PLACEMENT begins (`Res<CurrentClientPhase>` = PLACEMENT), WHEN Hand UI enters STAGING state, THEN:
  - Submit button entity has `Visibility::Visible`
  - Its text component reads exactly `"Submit (0 cards)"`
  - Its interaction component is `Active` from the first frame of PLACEMENT

- [ ] **HU-13**: GIVEN the player stages a card by dropping it on a valid board target, WHEN the drop is confirmed, THEN:
  - (a) A `GhostPlacementChanged { target: Some(<resolved PlayTarget variant>), card_id: Some(card_id) }` Bevy-internal message is written
  - (b) The fan slot for that card enters `FanSlotState::Ghost` marker component
  - (c) The Submit button text updates to `"Submit (N cards)"` where N is the new staged count
  - (d) The staged card's reserve strip entity becomes `Visibility::Visible` (see Story 011)
  - Note: The 40% chroma / 50% opacity ghost rendering is ADVISORY (lead sign-off).
  - Note: "Submit (1 cards)" — GDD is silent on singular; use "1 cards" consistently unless designer specifies. **(Designer flag: confirm singular/plural format.)**

- [ ] **HU-14**: GIVEN the player drops a dragged card on an unhighlighted (invalid) target or outside the board, WHEN the drop fires, THEN:
  - (a) The drag sprite returns to `Visibility::Hidden`
  - (b) The original fan slot returns to `FanSlotState::Active` marker component
  - (c) No `GhostPlacementChanged` message is written to the Bevy message bus

- [ ] **HU-16**: GIVEN the player clicks Submit with 0 staged cards, THEN:
  - `C2SSubmitPlacement { placements: [] }` is sent via Lightyear
  - The Submit button enters `Inactive` interaction state
  - Its text updates to `"Submitted"`
  - No confirmation modal entity is spawned

- [ ] **HU-17**: GIVEN the player clicks Submit once and the button becomes `Inactive` with text `"Submitted"`, WHEN the player attempts to click Submit again (e.g. via rapid double-click), THEN no second `C2SSubmitPlacement` message is written to the Lightyear send queue.

---

## Implementation Notes

*Derived from ADR-021 and GDD Rules 6, 8, 10:*

1. **PLACEMENT entry**: On PLACEMENT phase entry (from `PresentationSet::PhaseTransition`), set HandUiMode to STAGING: show Submit button with text "Submit (0 cards)" and `Active` interaction state; initialize local `PendingPlacements` vec as empty.

2. **Drag-to-stage core state machine**: On valid drop, add card to local `PendingPlacements` vec, write `GhostPlacementChanged` (Bevy-internal), set fan slot to `FanSlotState::Ghost`, increment submit count in button text.

3. **Submit on invalid drop**: No state change. Drag sprite hides. Fan slot reverts to `FanSlotState::Active`. No `GhostPlacementChanged` written.

4. **Submit button lock**: Once `C2SSubmitPlacement` is sent, immediately set interaction to `Inactive` and update text to "Submitted". No re-send path exists — check interaction state before enqueuing any `C2SSubmitPlacement` message.

5. **`GhostPlacementChanged` type** (register in `HandUiPlugin::build()`):
   ```rust
   #[derive(Message, Clone)]
   pub struct GhostPlacementChanged {
       pub target: Option<PlayTarget>,
       pub card_id: Option<CardId>,
   }
   app.add_message::<GhostPlacementChanged>();
   ```
   Board Rendering reads this in `PresentationSet::MessageDrain`.

6. **Reserve strip visibility**: When a card stages, the reserve strip entity for that card slot (see Story 011) becomes `Visibility::Visible`. This story sets the visibility; Story 011 implements the strip's logic.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 006]: Drag highlight sets by card type (BoardCellHighlighted, TargetUnitHover)
- [Story 007]: Instant card staging to fan plate
- [Story 008]: Un-staging via board ghost click/drag
- [Story 009]: Grace window (timer expiry during active drag) and urgency state
- [Story 010]: Submit pre-validation (mana/reserve overdraw checks)
- [Story 011]: Reserve mana split strip logic (+/- controls)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-11**: Submit button on PLACEMENT entry
  - Given: `App` in `ClientState::InSession`; `CurrentClientPhase` = DRAFT_SHOP
  - When: `CurrentClientPhase` updated to PLACEMENT; `App::update()` runs
  - Then: Submit button entity has `Visibility::Visible`; text component == `"Submit (0 cards)"`; interaction state == `Active`
  - Edge cases: Rapid PLACEMENT entry → always "Submit (0 cards)" regardless of previous staged state (local queue reset on entry)

- **HU-13**: Valid drop → stage + GhostPlacementChanged
  - Given: PLACEMENT active; hand with Minion card at slot 0; `BoardCellHighlighted` markers set for cells [1,2] in lane 1
  - When: Simulate drop of card 0 on cell (lane=1, cell=2) (valid highlighted cell)
  - Then: `GhostPlacementChanged { target: Some(BoardCell { lane:1, cell:2 }), card_id: Some(slot_0_id) }` message present in Bevy message bus; slot 0 has `FanSlotState::Ghost`; Submit text == `"Submit (1 cards)"`; reserve strip for slot 0 has `Visibility::Visible`
  - Edge cases: Stage second card → Submit text == `"Submit (2 cards)"`

- **HU-14**: Invalid drop → cancel drag
  - Given: PLACEMENT; Minion card lifted (drag sprite visible); cursor over non-highlighted cell
  - When: Simulate drop on non-highlighted cell
  - Then: Drag sprite has `Visibility::Hidden`; slot 0 has `FanSlotState::Active`; message bus contains no `GhostPlacementChanged`
  - Edge cases: Drop outside board entirely → same result; drop on occupied cell (no highlight) → same result

- **HU-16**: 0-card submit
  - Given: PLACEMENT; 0 staged cards; Submit button `Active` with text `"Submit (0 cards)"`
  - When: Click Submit button
  - Then: Lightyear outbound queue contains `C2SSubmitPlacement { placements: [] }`; Submit button interaction == `Inactive`; text == `"Submitted"`; no entity with ConfirmationModal marker component
  - Edge cases: Submit with text `"Submit (0 cards)"` at 9 seconds remaining → timer continues, Submit locked

- **HU-17**: Double-submit prevention
  - Given: PLACEMENT; Submit button `Inactive` (already submitted)
  - When: Click Submit button again (even if via programmatic trigger)
  - Then: Lightyear outbound queue still contains only the original `C2SSubmitPlacement` (no second entry)
  - Edge cases: Click 10× rapidly → still only 1 `C2SSubmitPlacement` in queue

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/placement_submit_core_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (pre-pooled fan slots), Story 002 (fan positions), Story 003 (PLACEMENT phase entry)
- Unlocks: Story 006 (drag highlights), Story 007 (Instant staging), Story 008 (un-staging), Story 009 (timer + grace window), Story 010 (pre-validation), Story 011 (reserve strip)

## Completion Notes

**Completed**: 2026-05-03
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 5/5 passing; HU-11, HU-13, HU-14, HU-16, and HU-17 are covered by `tests/unit/hand-ui/placement_submit_core_test.rs`.
**Test Evidence**: `cargo test -p client --test hand_ui_placement_submit_core_test` passed 5/5. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
**Verification**: `client/src/ui/hand/mod.rs` registers `GhostPlacementChanged` as a Bevy-internal message, resets `PendingPlacements` and activates the Submit button on PLACEMENT entry, stages valid drops into `PendingPlacements`, emits ghost messages, restores invalid drops without ghost messages, and locks the Submit button after the first `C2SSubmitPlacement` send.
**Notes**: Advisory only - the unit test verifies the Hand UI local message/outbox seam and optional `MessageSender<C2SSubmitPlacement>` path, not a full live Lightyear transport session. Advisory only - current `TR-HU-002` also mentions cursor-to-cell mapping via `BoardLayout`; this story intentionally scopes that work to core stage/submit behavior, while drag highlight and target mapping are deferred to Story 006. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
**Tech Debt**: None logged.
**Sprint Status**: Unchanged per user instruction; no matching Hand UI Story 005 row exists in `production/sprint-status.yaml`.
**Next Recommended**: Hand UI Story 006 PLACEMENT Drag Highlights (`production/epics/hand-ui/story-006-placement-drag-highlights.md`) after readiness check.
