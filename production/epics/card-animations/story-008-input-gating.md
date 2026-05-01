# Story 008: Input-gating: timer bar, drag latency, bid button state, de-hover cancel-replace

> **Epic**: Card Animations
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-animations.md`
**Requirement**: `TR-CAN-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: Input-gating rules: bid buttons gated on settlement only, NOT on animation completion; card slides non-blocking; drag sprite is a bevy_ui `Node` (NOT a world-space Sprite) for correct z-ordering above board content; 0-frame Bevy-side latency from input event to animation start.

**Engine**: Bevy 0.18 + bevy_tweening 0.18 | **Risk**: HIGH
**Engine Notes**: OQ-CA-01 (`AnimatorState` enum name) affects CA-13, CA-23, CA-24 `Playing` state assertions — confirm via Story 001 cargo check before writing test file. WASM platform input latency (16–50 ms from physical mouse-down to Bevy event receipt) is a known floor not addressable at this layer — ACs validate Bevy-side latency only. Hand drag-sprite preview is `Node` + `ImageNode::new(handle)` — NOT `UiImage::new()` (deprecated in Bevy 0.16). `ChildOf(parent)` for hierarchy (NOT `set_parent`).

**Control Manifest Rules (Presentation Layer)**:
- Required: Hand drag-sprite preview is a bevy_ui `Node` (NOT world-space `Sprite`). 0-frame Bevy-side latency from input to `Animator` insertion. `set_tweenable` for cancel-replace (no despawn/respawn). De-hover replacement tween minimum 40 ms floor. At most one card scaling toward 1.12× (hover target) at any time. `PickingBehavior` only inside `#[cfg(feature = "ui_picking")]` guard.
- Forbidden: Animation gating phase transition. Animation gating bid re-enable (buttons re-enable on message receipt, NOT on tween completion). `Transform.translation` written while `Animator<Transform>` active. `UiImage::new()` — use `ImageNode::new()`. `commands.entity(e).set_parent(p)` — use `commands.entity(e).insert(ChildOf(p))`.
- Guardrail: ≤ 2 UI regions animating simultaneously per phase transition. Presentation steady-state < 1 ms per frame.

---

## Acceptance Criteria

*From GDD `design/gdd/card-animations.md`, scoped to this story:*

- [x] **CA-13** — GIVEN `TimerBarEaseRequested` is emitted, WHEN one `App::update()` tick runs, THEN the timer bar entity's `Animator` is in `AnimatorState::Playing` in the same frame as event receipt (0-frame latency). **[BLOCKING]**
- [ ] **CA-13b** — GIVEN `TimerBarEaseRequested` is processing a tween, WHEN the tween is in flight, THEN bid preset buttons are in enabled state. *(Manual UI walkthrough; screenshot evidence.)* **[ADVISORY - DEFERRED]**
- [ ] **CA-22** — The "≤ 2 animated UI regions per phase transition" rule is enforced by upstream event sequencing design — not a Card Animations runtime guard. Verified by: (a) code review of upstream plugin event-emit timing confirming sequencing invariants, (b) manual playtest walkthrough at DRAFT_INITIAL entry confirming panel and card-draw animations do not fire simultaneously. **[ADVISORY - DEFERRED]**
- [x] **CA-23** — GIVEN a card in hand during PLACEMENT phase, WHEN a drag-start input event is processed by `App::update()`, THEN the drag sprite entity has an `Animator` in `AnimatorState::Playing` in the same tick (0-frame Bevy-side latency). **[BLOCKING]**
- [x] **CA-24** — GIVEN card A has an active `Animator<Transform>` in `Playing` (de-hover return-to-1.0× tween), WHEN a hover-in event fires on card B before A's tween completes, THEN: (a) card A's `Animator<Transform>` remains in `Playing` — new return-to-1.0× tween installed via `set_tweenable` (cancel-replace, NOT stop), AND (b) card A's `Transform.scale.x` is in range (1.0, 1.12] exclusive (no instant snap to 1.0 — scale still at intermediate value), AND (c) card B's `Animator<Transform>` is in `Playing`, AND (d) query `With<HandCard>` + `With<Animator<Transform>>` filtered for `Playing` returns exactly 2 entities (A returning, B hovering). **[BLOCKING]**

---

## Implementation Notes

*Derived from ADR-021 and GDD Rules C-3, C-5, C-13:*

1. **0-frame latency (GDD Rule C-5):** `TimerBarEaseRequested` handler and drag-start handler must be in `PresentationSet::MessageDrain` or `PresentationSet::AnimationTick`, running within the same `App::update()` tick as the triggering event. `CardAnimationsSet::React` ordered after `BoardRenderSet::ScheduleTweens` for same-frame delivery.

2. **Bid button state independence (GDD Rule C-3):** Timer bar tween and bid button enable/disable are independent. Bid buttons re-enable on `S2CAuctionBidAccepted` receipt — NOT on timer bar tween completion. The `TimerBarEaseRequested` handler must NOT touch bid button state.

3. **De-hover cancel-replace with 40 ms floor (GDD Rule C-13):** When cursor leaves card A, install return-to-1.0× tween via `set_tweenable`. If hover-in fires on card B before A's tween completes: install new return-to-1.0× tween on A from its current intermediate scale. **Minimum floor 40 ms regardless of remaining time** — prevents single-frame snap when cursor swipes rapidly across fan (remaining duration may be 5–15 ms at fast swipe speeds).

4. **At-most-one hover scaling invariant (GDD Rule C-13):** Only one card may be scaling toward 1.12× at any time. Cards returning to 1.0× (de-hover) do NOT count against this. New hover on card B → cancel-replace on any card that is scaling toward 1.12×.

5. **Drag sprite is bevy_ui `Node` (ADR-021 §8):** `Node` + `ImageNode::new(card_atlas.image.clone())`. NOT a world-space `Sprite`. Ensures correct z-ordering above board content during PLACEMENT drag.

6. **`≤ 2 UI regions` design contract (GDD Rule C-14):** DRAFT_INITIAL: panel slide fires tick 1; card-draws fire at t+350 ms (`anim_panel_slide_duration_ms`); `HandShowRequested` is instant `Visibility` change (no tween). Compliance verified in code review + manual playtest. Not a runtime runtime guard.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001](story-001-plugin-scaffold-custom-lenses.md): Plugin scaffold
- [Story 002](story-002-tween-cancel-replace-lifecycle.md): `BoardRebuildRequested` cancel; concurrent `Animator<T>`
- [Story 005](story-005-placement-reveal-parallelism.md): PLACEMENT phase animation lifecycle (CA-12, CA-21)

---

## QA Test Cases

*Written by qa-lead at story creation. CA-13, CA-23, CA-24 assert `AnimatorState::Playing` — confirm exact enum name via OQ-CA-01 (Story 001) before writing test file.*

**CA-13 — TimerBarEaseRequested produces Playing Animator in same frame (0-frame latency)**

- Given: World with `CardAnimationsPlugin`; pre-spawned timer bar entity; `MessageWriter<TimerBarEaseRequested>` writes one message
- When: `app.update()` called once
- Then: `world.get::<Animator<T>>(timer_bar_entity)` in `Playing` state
- Edge cases: `TimerBarEaseRequested` fired twice same frame — second replaces first (`set_tweenable`); exactly one `Animator` after update; timer bar entity missing from world — error logged, no panic

**CA-23 — Drag-start produces Playing Animator on drag sprite same frame (Bevy-side)**

- Given: World with `CardAnimationsPlugin` + `HandUiPlugin`; PLACEMENT phase active (`CurrentClientPhase.phase == Placement`); drag sprite entity pre-spawned (pre-pooled, `hand-ui.md` Rule 1); drag-start input event injected via `app.world`
- When: `app.update()` called once
- Then: `world.get::<Animator<T>>(drag_sprite_entity)` in `Playing` state (same tick as input processing)
- Edge cases: drag-start outside PLACEMENT phase — no `Animator` spawned, no panic; WASM platform latency is a known floor not under test

**CA-24 — De-hover cancel-replace: A stays animated, B starts, scale.x not snapped, exactly 2 Playing**

- Given: 3 `HandCard` entities (A, B, C); card A has `Animator<Transform>` in `Playing` (de-hover return-to-1.0× tween, intermediate `scale.x` e.g. 1.08); hover-in event fires on card B
- When: `app.update()` called once
- Then:
  - (a) `world.get::<Animator<Transform>>(card_A)` in `Playing` (cancel-replaced, not stopped)
  - (b) `world.get::<Transform>(card_A).unwrap().scale.x` in range (1.0, 1.12] exclusive (no instant snap to 1.0)
  - (c) `world.get::<Animator<Transform>>(card_B)` in `Playing`
  - (d) query `With<HandCard>` + `With<Animator<Transform>>` filtered for `Playing` returns exactly 2 entities (A returning, B hovering)
  - (e) `world.get::<Transform>(card_C).map(|t| t.scale.x)` == `Some(1.0)` (card C untouched)
- Edge cases:
  - 40 ms floor: A's replacement tween duration ≥ 40 ms when remaining time < 40 ms (verify tween duration assertion)
  - Hover on C before A's replacement completes: A gets new replacement from new intermediate; C gets hover-in; still exactly 2 Playing

**CA-13b — Bid buttons enabled during timer bar tween (Visual/Feel)**

Manual check: Bid buttons enabled while timer bar ease-out plays
  - Setup: Client in DRAFT_AUCTION phase; place a bid (receives `S2CAuctionBidAccepted` + `TimerBarEaseRequested` fires)
  - Verify: Bid preset buttons are in clickable/enabled state while timer bar ease-out tween is in flight
  - Pass condition: Player can successfully click a bid button before the timer bar animation completes (screenshot evidence in `production/qa/evidence/input-gating-evidence.md`)

**CA-22 — ≤ 2 animated UI regions at DRAFT_INITIAL phase entry (Visual/Feel)**

Manual check: Panel slide and card-draws do not animate simultaneously at DRAFT_INITIAL entry
  - Setup: Game entering DRAFT_INITIAL phase from LOBBY
  - Verify: Auction panel slides in (region 1, ~350 ms); card-draw animations start only after ~350 ms delay (region 2); `HandShowRequested` is instant `Visibility` change (not counted as an animated region)
  - Pass condition: Exactly 1 animated region during t=0..350 ms (panel slide); exactly 1 animated region during t=350..630 ms (card draws); no overlap (screenshot evidence in `production/qa/evidence/input-gating-evidence.md`)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Integration: `tests/integration/card-animations/input_gating_test.rs` — must exist and pass
- Visual: `production/qa/evidence/input-gating-evidence.md` (CA-13b + CA-22 screenshots + lead sign-off)

**Status**: [x] Automated integration evidence created and passing; CA-13b and CA-22 manual UI walkthrough evidence deferred until bid-button UI and DRAFT_INITIAL animation sequencing UI exist.

---

## Dependencies

- Depends on: [Story 002](story-002-tween-cancel-replace-lifecycle.md) must be DONE (cancel-replace contract; OQ-CA-01 resolved for test harness)
- Unlocks: None

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 3/3 blocking passing (CA-13, CA-23, CA-24); 2 advisory manual criteria deferred (CA-13b, CA-22) until bid-button UI and DRAFT_INITIAL animation sequencing UI exist.
**Deviations**: Advisory only - story/GDD wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`, while the compiled workspace uses `TweenAnim` / `PlaybackState` / `TweenState` from `bevy_tweening 0.15`. Advisory only - worker commit `0d75fb0` is not an ancestor of `HEAD`, but it has the same stable patch-id as main integration commit `9308bf3`, and `9308bf3` is included in current `main`.
**Test Evidence**: Integration test file at `tests/integration/card-animations/input_gating_test.rs`; `cargo test -p client --test card_animations_input_gating_test` passed 6/6. Scaffold regression `cargo test -p client --test card_animations_plugin_scaffold_test` passed 8/8. `cargo check -p client` passed. Manual evidence file exists at `production/qa/evidence/input-gating-evidence.md` with CA-13b and CA-22 marked pending.
**Code Review**: Skipped - Lean mode.
