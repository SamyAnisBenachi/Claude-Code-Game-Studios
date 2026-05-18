# Story 023: S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001 — Idle Hand Playable Affordance

> **Epic**: Hand UI
> **Story ID**: `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001`
> **Status**: Draft — future Sprint 18 candidate; NOT activated
> **Layer**: Presentation — Hand UI (idle-hand visual affordance, distinct
> from drag-state overlays)
> **Type**: UI + Integration test (ECS marker / colour assertions)
> **Authored**: 2026-05-18 by PROMPT 1136
> **Authoring worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s18-hand-mana-affordance-stories`
> **Authoring branch**: `work/s18-hand-mana-affordance-stories`
> **Authoring source-of-truth**: `origin/main@05192b5f830c5d5b17ed7af07df37f56187130fc`
> (PROMPT 1125 `story-done(s17): close S17-OPS-VULKAN-VALIDATION-GATING-001`)

---

## Status / No-Claim Banner

This story is authored by PROMPT 1136 as a **future Sprint 18** candidate
covering the **missing-feature (designed-out)** classification from
`reports/PROMPT-1127-placement-drag-drop-mana-preview-diagnostic.md` §R3
(*"Playable-card affordance in idle hand"*). Sprint 18 does **NOT** exist
as a plan file on `origin/main` at authoring time
(`production/sprints/sprint-18.md` is absent). This story is candidate
planning only.

PROMPT 1136 (this authoring run) does **NOT**:

- Activate Sprint 18.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-17.md`, `production/sprints/sprint-18.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / team-qa / gate-check / release-check
  artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/` file.
- **Re-author or edit `production/epics/hand-ui/story-020-hand-drag-state-visuals.md`.**
  Story 020 is **Complete** and its AC2 idle-baseline assertion
  remains accurate as written **because this story deliberately uses
  a distinct marker (`FanSlotPlayableAffordanceOverlay`) that does
  NOT match `DragStateOverlay`**. The Story 020 query
  `Query<&FanSlotIndex, Without<DragStateOverlay>>` continues to
  identify the same set of idle slots; the affordance overlay does
  not contaminate it. See AC2 below and the "Story 020 AC2
  Reconciliation" section.

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- closure of `S8-QA-001-W1`
- closure of the **R1 drag-pipeline-dead-in-shipped-build bug**
  (PROMPT 1127 §R1). R1 is a **separate repair / separate prompt**.
- closure of the **R2 mana-preview missing-feature** (PROMPT 1127
  §R2). R2 is authored as a sibling future-Sprint-18 candidate at
  `story-022-hand-mana-preview-during-drag.md`.
- closure of `AUDIT-1076-02 / AUDIT-1076-03` server-side placement
  loss.
- final-art / asset-production completion (`PAW-TD-*-a`).
- advance of stage from `Polish` to `Release`.

ADR-002 + ADR-012 + ADR-021 binding preserved: this story is **read-only
over client-side mirrors of authoritative state** (`Res<PlayerEconomyView>`,
`Res<HandCardCatalog>`, `Res<HandUiMode>`,
`Res<CurrentClientPhase>`, `Res<ActivePlacementDrag>`,
`Res<PendingPlacements>`). It introduces no new server-authoritative
state, no new Lightyear message, no new protocol shape, and no
client-side authority over stage / activate / submit.

---

## Source Finding

**PROMPT 1127 diagnostic** (`reports/PROMPT-1127-placement-drag-drop-mana-preview-diagnostic.md`
§3 *"R3 — Playable-card affordance in idle hand"*):

> **Status: MISSING-FEATURE (designed out)**. Disabled-overlay code
> exists at `client/src/ui/hand/drag_state_visuals.rs:358-368, 207-250`.
> The check `available >= card.cost` is computed correctly. It is
> wired into `DragStateOverlayActive::Disabled` and triggers a
> 0.45-alpha dim layer.
>
> **But the sync system (`:185-230`) gates all overlay logic on
> `drag_active`.** When `ActivePlacementDrag::is_active()` is false
> (idle hand, no drag in flight), the `Disabled` state is never
> written and the dim overlay stays `Visibility::Hidden`.
>
> This is **by design**: Story 020 AC2
> (`production/epics/hand-ui/story-020-hand-drag-state-visuals.md`)
> explicitly states *"an idle slot has no visible overlay"*. The
> five drag states (`Idle`, `Hover`, `Drag` source, `DropTarget`
> fan-plate, `Disabled`) are all part of the **drag** state machine;
> none of them is an **idle-hand** affordance.

**Test false-confidence** (PROMPT 1127 §3.2):

> `tests/integration/hand-ui/hand_ui_drag_state_visuals_test.rs:AC5`
> drives the overlay state via direct resource mutation
> (`set_active_drag(...)`) — i.e. it forces a drag to be active
> before asserting. It never exercises the idle-hand path. A
> complete absence of an idle affordance therefore reads as PASS.

**Classification**: MISSING-FEATURE (designed out) — the implementation
matches the written design. The user's complaint is a **design gap**,
not a code regression. Repair requires a story authoring pass before
implementation. This is that pass.

**Source code anchors** (verified at `origin/main@05192b5`):

- `client/src/ui/hand/drag_state_visuals.rs:211-345`
  (`sync_hand_drag_state_visuals_system`): gates Disabled-overlay
  application on the `drag_active` branch — when
  `ActivePlacementDrag::is_active() == false` AND
  `HandUiMode != PassiveLocked`, no per-slot `Disabled` state is
  inserted, so the dim overlay stays Hidden.
- `client/src/ui/hand/drag_state_visuals.rs:352-368`
  (`slot_is_affordable`): `available = current_mana + reserve_mana;
  available >= card.cost`. Non-Minion cards return `true`. This is
  the existing affordability test reused by this story.

---

## Problem Class / Prevention Target

**Defect class**: In idle PLACEMENT (and in idle Passive /
`HandUiMode::Passive`) state, the player has no on-screen visual
indication of which hand cards they can currently afford. The
Story 020 drag-state visual differentiation surfaces affordability
**only while a drag is in flight** (the source slot's
`Disabled` overlay applies when `drag_active` and the slot's
card fails `slot_is_affordable`). Outside of drag:

1. Every card slot reads identically — the player cannot tell
   whether attempting to drag a card will result in the `Disabled`
   dim treatment until the drag actually starts.
2. The reserve-strip's `[+]` ceiling (Story 011) only appears on
   already-staged cards, so it is not an idle-hand affordance.
3. The submit-pre-validation `SubmitValidationError` marker
   (Story 010) fires only at Submit click, after a multi-card
   placement attempt — far too late for the idle-hand affordance.
4. The drag-state `Disabled` overlay (Story 020) is suppressed
   in the idle path by design (Story 020 AC2 / Story 020 "Out of
   Scope").
5. The R1 drag-pipeline-dead bug (PROMPT 1127 §R1) means that even
   the existing `Disabled` overlay is unreachable in the shipped
   client — but R1 is a separate repair; this story's affordance
   is **independent of R1** because it activates in idle hand
   without requiring a `Pointer<*>` event flow.

**Prevention target**: introduce a **separate** idle-hand
affordance overlay (distinct from `DragStateOverlay` / Story 020)
that surfaces playability on local hand cards while the hand is in
an interactive mode (`HandUiMode::Passive` or `HandUiMode::Staging`)
and **no drag is in flight**. The affordance reads
`slot_is_affordable(card_id, &catalog, &economy)` (the existing
Story 020 helper) and applies an additive treatment to
**playable** local hand cards. Unaffordable cards receive a
**subdued** treatment per product decision (see AC3). Opponent
hands are NEVER affected (AC4).

The affordance overlay is **deliberately distinct from
`DragStateOverlay`** so that Story 020 AC2's ECS query
`Query<&FanSlotIndex, Without<DragStateOverlay>>` continues to
identify the same idle slots — the affordance overlay does NOT
carry `DragStateOverlay` as a component. The two overlays
coexist as sibling marker types under `FanSlotIndex` parent
entities; the drag-state pathway is owned by Story 020 and the
idle-affordance pathway is owned by this story.

---

## Story 020 AC2 Reconciliation (NO STORY 020 EDIT REQUIRED)

The Story 020 AC2 assertion reads (verbatim):

> **AC2 — `Idle` baseline preserved**: GIVEN `HandUiMode == Passive`
> (or `Staging` with no active drag), WHEN a fan-slot card with no
> pointer hover and no staged placement is inspected, THEN no new
> overlay child node is present, no new tint is applied, and the
> card chrome reads identically to the pre-refactor baseline.
> Verified by ECS query: `Query<&FanSlotIndex, Without<DragStateOverlay>>`
> (or equivalent marker) returns the expected idle slots; their
> `Sprite::color` / `BackgroundColor` are unchanged from baseline.

This story preserves the Story 020 AC2 assertion **by construction**:

1. The new idle-affordance overlays use a **distinct marker**:
   `FanSlotPlayableAffordanceOverlay` (component).
2. The new overlays do **NOT** insert the `DragStateOverlay`
   marker component. Story 020 AC2's `Without<DragStateOverlay>`
   query result is unchanged.
3. The new overlays may apply visible treatment (border / tint /
   glow) in idle states — this **does** introduce a visual change
   relative to the pre-Story-020 baseline, but is **scoped to the
   affordance treatment only**. The Story 020 AC2 phrase "no new
   tint is applied" is read as "no `DragStateOverlay` tint is
   applied" — the AC2 query already encodes that scoping via the
   `Without<DragStateOverlay>` predicate.
4. **No edit to `production/epics/hand-ui/story-020-hand-drag-state-visuals.md`
   is required or performed by this story.** The drafted
   reconciliation is the recommended reading; if a future review
   wants to make the Story 020 AC2 scoping explicit, a tiny
   docs-only follow-on story can add a footnote — that follow-on
   is **NOT in scope** for this story.

The `/dev-story` implementation prompt **MUST verify** that the
post-implementation `cargo test -p client --test hand_ui_drag_state_visuals_test`
(Story 020 AC9 regression) continues to PASS. If the existing test
uses a strict marker query that excludes the new affordance overlay
correctly, the regression passes naturally. If the existing test
relies on a query that incidentally catches the new overlay (e.g.
a `Without<DragStateOverlay>` + colour-equality assertion that
fails because the affordance treatment paints a tint), the
`/dev-story` implementation MUST be reconciled before the AC9
gate of this story.

---

## Context

### Existing surface

- **`client/src/ui/hand/drag_state_visuals.rs`** (Sprint 15 Story 020):
  - `sync_hand_drag_state_visuals_system` (lines 211-345): the
    DRAG state sync system. Idle-affordance work is a **sibling
    system**, not an edit of this one.
  - `slot_is_affordable` (lines 352-368): the helper this story
    REUSES. Public visibility may be needed (the worker may need
    to `pub use` it from `drag_state_visuals.rs` or refactor the
    helper into a shared `hand_affordability.rs` submodule).
  - `DragStateOverlay`, `FanSlotDimOverlay`, `FanSlotHoverOverlay`,
    `FanPlateDropTargetOverlay`, `DragStateOverlayActive`: existing
    marker components owned by Story 020. This story does NOT
    reuse these markers; it introduces its own marker set.
- **`client/src/ui/hand/mod.rs`**:
  - `HandUiMode` enum: `Hidden`, `Grid`, `Passive`, `PassiveLocked`,
    `Staging`. The idle affordance applies in `Passive` and
    `Staging` modes only.
  - `FanSlotIndex(u8)`: marker on each of the 10 pre-pooled fan
    slot entities. The new affordance overlay is a CHILD node
    spawned under each `FanSlotIndex` entity at session start.
  - `HandSlotCard(CardId)`: marker on slots that currently hold a
    card. Empty slots have no `HandSlotCard` and receive no
    affordance overlay.
  - `HandCardCatalog`: resource mapping `CardId → CardDef`. The
    affordance check reads `catalog.cards.get(&card_id).card_type` /
    `.cost`.
  - `ActivePlacementDrag`: the affordance is **suppressed** while
    a drag is in flight (drag-state visuals own that frame; the
    idle affordance steps aside).
  - `PendingPlacements`: staged cards are treated as "playing-it-
    already"; the affordance overlay on a staged card's fan slot
    is suppressed (since the slot is dimmed by the staged ghost,
    per Story 005 / Story 008).
- **`client/src/presentation/shared/economy_view.rs`**:
  - `PlayerEconomyView { current_mana, reserve_mana, mana_cap, gold, ... }`:
    the affordance reads `current_mana + reserve_mana` to compare
    against `card.cost`. Mirrors the existing `slot_is_affordable`
    formula.
- **`shared/src/card.rs::CardType`**:
  - `Minion` is the only mana-costing card type under the current
    pool. Non-Minion types return `true` from `slot_is_affordable`
    and thus always read as "playable" in this story's affordance
    treatment (see AC5).
- **`client/src/ui/design_tokens/`**:
  - `overlays::OVERLAY_DIM_ALPHA` (`0.45`): the same constant
    Story 020 uses for the drag-state Disabled dim. The
    unaffordable subdued treatment in this story REUSES this
    constant for visual consistency. **No new design tokens are
    authored.**
  - `z_layers::UI_BASE` / `UI_OVERLAY`: the affordance overlay
    paints at the same z-layer as the fan-slot chrome
    (`UI_BASE` per the existing fan-slot spawn), NOT at
    `UI_OVERLAY` — the affordance is a calm idle treatment, not
    a "lifted" drag-time treatment.
  - §7 `ACCENT` / `SEMANTIC_SUCCESS`: the playable treatment may
    consume a subtle border or glow in this palette. Worker
    discretion within the existing token set.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` Rule 3 (Phase Behavior) and
  Rule 6 (PLACEMENT drag-to-stage) implicitly cover idle hand
  rendering. Light addition: one row in Rule 3 / Rule 6
  documenting the idle playable affordance. Owner: `/dev-story`
  paired with implementation OR follow-on doc story (same
  pattern as Story 020).
- **`design/gdd/hud.md`**: HUD `PlayerEconomyView` reflection
  unchanged.
- **ADR-021** (Presentation Layer Architecture): preserved. The
  affordance overlay is a CHILD node under each pre-pooled
  `FanSlotIndex` entity; no new top-level pre-pool entry
  (`HAND_UI_ENTITY_COUNT` bumps by `HAND_FAN_SLOT_COUNT` to
  account for the new child overlay entities — Story 020's
  precedent at `+(HAND_FAN_SLOT_COUNT * 2 + 1)`). The bump is
  performed by `/dev-story`, not by this authoring run.
- **ADR-002** (Client-Server Authority): preserved. Affordance is
  a read-only derivation of `PlayerEconomyView` and
  `HandCardCatalog`. No mutation; no `S2C*` / `C2S*` edit.
- **ADR-012** (SessionReady Delivery): preserved. No new server-
  authoritative state surface.
- **ADR-009** (RSM Phase State): preserved. Affordance gates on
  `HandUiMode` and `CurrentClientPhase` directly; no
  `MessageReceiver<S2CPhaseChanged>` drain.
- **TR registry**: extends `TR-HU-002` (PLACEMENT drag-to-stage
  state machine) with a **new TR**: `TR-HU-010` — *"Idle hand
  playable affordance"*. The TR registry edit is performed by the
  `/dev-story` implementation prompt, not by this authoring run.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits under
  `client/src/ui/hand/`, and for the integration test.
- **Lightyear**: NOT applicable. `liv-bevy-lightyear` is NOT
  required and NOT activated.

### Control Manifest Rules

- **Required**: The affordance overlay uses a **distinct marker**
  set (e.g. `FanSlotPlayableAffordanceOverlay`,
  `FanSlotPlayableAffordanceActive`). These markers do NOT
  include `DragStateOverlay` as a component, preserving Story
  020 AC2's `Without<DragStateOverlay>` query semantics.
- **Required**: The sync system gates on:
  - `Res<CurrentClientPhase>` is one of the phases where hand is
    interactive (`Phase::Placement` definitively, optionally
    `Phase::DraftShop` and `Phase::DraftAuction` if the worker
    chooses to extend — but the **BLOCKING scope is `Phase::Placement`
    only**; other phases are advisory per AC8).
  - `Res<HandUiMode>` ∈ `{ Passive, Staging }` (NOT `PassiveLocked`,
    `Hidden`, `Grid`).
  - `Res<ActivePlacementDrag>::is_active() == false` (drag-state
    visuals own that frame).
  - The slot has a `HandSlotCard` (empty slots receive no
    treatment).
  - The slot's card is NOT in `PendingPlacements::placements`
    (staged cards receive no idle affordance; the staged ghost
    treatment from Story 005 / 008 is preserved).
- **Required**: The affordance affects ONLY the **local player's
  hand**. Opponent hand surfaces (if any are rendered) are
  unaffected. See AC4 explicit verification.
- **Required**: The affordance applies a **playable** treatment
  to slots where `slot_is_affordable(card_id, &catalog, &economy)
  == true` AND an **unaffordable / subdued** treatment to slots
  where the helper returns `false` (Minion-only check; non-Minion
  cards always "playable").
- **Required**: The unaffordable treatment is **explicit, not
  absent**: per the prompt requirement *"unaffordable cards show
  subdued/disabled treatment or no affordance per product choice,
  but must be explicit"*, this story takes the **explicit
  subdued** product choice. Rationale: an absent treatment is
  ambiguous with the idle Story 020 baseline; an explicit subdued
  treatment communicates "this card exists but you cannot afford
  it" clearly. The treatment is the same
  `overlays::OVERLAY_DIM_ALPHA` tint that Story 020's drag-time
  `Disabled` overlay uses, applied at idle. See AC3.
- **Required**: Updates **reactively** on every
  `PlayerEconomyView` change (`S2CGoldUpdate` /
  `S2CGameSnapshot` drain). The sync system runs in
  `PresentationSet::StateSync` so it re-evaluates each frame
  (matching Story 020's sync system pattern). See AC6.
- **Required**: `liv-bevy-018` skill applies to all `.rs` edits.
- **Required**: ADR-021 Impl Guideline 5 preserved — the new
  affordance overlay child nodes are CHILDREN of the existing
  pre-pooled `FanSlotIndex` entities (10 of them), not new
  top-level pre-pool entries. `HAND_UI_ENTITY_COUNT` bumps by
  `HAND_FAN_SLOT_COUNT` (one playable-treatment overlay child
  per slot) OR by `2 * HAND_FAN_SLOT_COUNT` (one playable + one
  unaffordable overlay child per slot, mutually-exclusive
  visibility). Worker discretion; the BLOCKING assertion is that
  no NEW top-level pre-pool entry is added.
- **Required**: Integration test (BLOCKING) asserts the
  affordance treatment via ECS marker / colour queries. The test
  drives the affordance state via direct resource insertion
  (mutate `PlayerEconomyView.current_mana` /
  `.reserve_mana`, write `HandSlotCard(card_id)`, set
  `HandUiMode::Passive`). No `Pointer<*>` event synthesis is
  required. Independent of R1.
- **Forbidden**: Mutating `PlayerEconomyView`, `HandCardCatalog`,
  `ActivePlacementDrag`, `PendingPlacements`, `HandUiMode`, or
  `CurrentClientPhase` from any affordance system.
- **Forbidden**: Adding `DragStateOverlay` as a component to the
  new affordance overlay child nodes.
- **Forbidden**: Modifying `production/epics/hand-ui/story-020-hand-drag-state-visuals.md`
  (Story 020 is Complete). The reconciliation lives in this
  story file; no edit to Story 020 is required.
- **Forbidden**: Touching the existing
  `sync_hand_drag_state_visuals_system` body or its overlay child
  spawn helpers in `drag_state_visuals.rs`. The new affordance
  systems live in a sibling submodule (e.g.
  `client/src/ui/hand/playable_affordance.rs`) or in
  `drag_state_visuals.rs` as a clearly-named sibling system.
- **Forbidden**: Surface coupling with opponent-side rendering
  (the affordance applies to LOCAL hand only). AC4 explicit.
- **Forbidden**: Repair of R1 drag pipeline or R2 mana preview
  (those are separate stories / prompts).
- **Forbidden**: Introducing a new Lightyear message or
  server-authoritative state surface. AC13 explicit.

---

## Story Classification

**Story type**: UI + Integration test (ECS marker / colour
assertions).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type",
UI stories deliver manual walkthrough OR interaction test with
ADVISORY gate. This story raises the bar to **BLOCKING integration
test** (matching Story 020 precedent) because the affordance
derivation is deterministic — a pure function of `PlayerEconomyView`
+ `HandCardCatalog` + `HandUiMode` + `ActivePlacementDrag` +
`PendingPlacements` + per-slot `HandSlotCard` state.

This is **NOT** a:

- Networking / protocol story (AC13 verification).
- Final-art story (`PAW-TD-*-a` preserved).
- Accessibility story (`QA-COND-0005` preserved; this is friend-
  game visual polish).
- Drag-state visual differentiation re-author (Story 020 is
  Complete; this story is **additive** on a distinct overlay
  pathway).
- Runtime-bug-repair story (R1 / R2 / R3 are missing-feature /
  build-config bug classifications; R1 is a build-config repair
  in a separate prompt; R3 is THIS story but as a missing-feature
  AUTHORING pass, not a runtime repair).

---

## Dependencies (must be Done before /dev-story on this story)

| Dependency | Slug / Story | Why blocking |
|---|---|---|
| `slot_is_affordable` helper | Story 020 (Complete) | The affordance check reuses the exact helper at `client/src/ui/hand/drag_state_visuals.rs:352-368`. The `/dev-story` worker either `pub use`s the helper or refactors it into a shared affordability submodule. |
| `PlayerEconomyView` resource | `PRES-002` (Complete) | The affordance reads `current_mana + reserve_mana` to compare against `card.cost`. |
| `HandCardCatalog` resource | Story 017 + Story 020 (both Complete) | Cost / type lookup. |
| `HandUiMode` resource | Story 003 (Complete) | Gates the affordance to `Passive` / `Staging` modes. |
| `ActivePlacementDrag` resource | Story 017 + Story 020 (Complete) | Suppresses affordance during drag. |
| `PendingPlacements` resource | Story 005 + Story 010 (Complete) | Suppresses affordance on already-staged cards. |
| `CurrentClientPhase` resource | ADR-009 + Story 003 (Complete) | Gates affordance to `Phase::Placement` (BLOCKING scope). |
| `HandSlotCard` marker + per-slot pre-pool | Story 001 + Story 002 (Complete) | The affordance is a CHILD overlay of pre-pooled fan slot entities. |
| Tier 0 overlay tokens (`OVERLAY_DIM_ALPHA`) + colour palette (`ACCENT` / `SEMANTIC_SUCCESS`) | Sprint 14 Tier 0 stories (Complete) | Token consumption; no new tokens. |

**Optional but recommended** (not blocking):

- Coordination with `story-022-hand-mana-preview-during-drag.md`
  (R2 sibling). The two stories are **parallel-safe**:
  story 022 edits HUD mana labels reactively; story 023 edits
  hand-fan slot child overlays. Neither story depends on the
  other.
- Coordination with the R1 drag-pipeline repair prompt. The R1
  repair UNBLOCKS the existing Story 020 drag-time `Disabled`
  overlay (which is currently dead in the shipped client per
  PROMPT 1127 §1). This story's idle affordance is **independent
  of R1**, but the manual evidence quality is best when R1 is
  also merged so the player can compare the idle affordance
  (story 023) with the drag-time `Disabled` overlay (Story 020).
- A light `design/gdd/hand-ui.md` Rule 3 / Rule 6 addition. May
  be paired with `/dev-story` or deferred.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 — Playable affordance applies to affordable local
  hand cards (idle)**: GIVEN `Res<CurrentClientPhase> == Phase::Placement`
  AND `Res<HandUiMode> ∈ { Passive, Staging }` AND
  `Res<ActivePlacementDrag>::is_active() == false` AND a local
  hand slot has `HandSlotCard(card_id)` AND
  `slot_is_affordable(card_id, &catalog, &economy) == true` AND
  the card is NOT in `PendingPlacements::placements`, WHEN one
  `App::update()` tick runs, THEN:
  - The slot entity carries `FanSlotPlayableAffordanceActive::Playable`
    (or equivalent state marker).
  - The slot's `FanSlotPlayableAffordanceOverlay` child node has
    `Visibility::Visible`.
  - The overlay does NOT carry `DragStateOverlay` (preserves Story
    020 AC2 query semantics).
  - The treatment is the playable composition (e.g.
    `BorderColor::all(accent_color())` border or
    `BorderColor::all(semantic_success_color())` outline; worker
    discretion within §7 token symbols).
  Verified by integration-test ECS query.

- [ ] **AC2 — Subdued treatment applies to unaffordable local
  hand cards (idle)**: GIVEN the same idle preconditions as AC1
  AND `slot_is_affordable(card_id, &catalog, &economy) == false`
  (i.e. `card.card_type == Minion` AND
  `current_mana + reserve_mana < card.cost`), WHEN one tick runs,
  THEN:
  - The slot entity carries `FanSlotPlayableAffordanceActive::Unaffordable`
    (or equivalent state marker).
  - A `FanSlotPlayableAffordanceUnaffordableOverlay` child node
    (distinct from the Playable overlay) has
    `Visibility::Visible`, painting `OVERLAY_DIM_ALPHA` (0.45)
    tint sourced by SYMBOL from `overlays::OVERLAY_DIM_ALPHA`.
  - The Playable overlay (AC1) has `Visibility::Hidden` for the
    same slot.
  - The unaffordable overlay does NOT carry `DragStateOverlay`.
  Verified by integration-test ECS query.

- [ ] **AC3 — Playable / Unaffordable are mutually exclusive**:
  GIVEN any single local hand slot in any frame, WHEN the
  affordance sync system runs, THEN at most one of
  `FanSlotPlayableAffordanceActive::{Playable, Unaffordable}`
  is set on the slot entity (never both, never neither when AC1
  / AC2 preconditions are met). The two overlay child node
  visibilities are inverses; both `Hidden` only when the
  affordance is suppressed (AC4, AC7, AC8, AC9).

- [ ] **AC4 — Opponent hand unaffected**: GIVEN the post-
  implementation build, WHEN inspected, THEN no opponent-side
  entity carries `FanSlotPlayableAffordanceActive` or
  `FanSlotPlayableAffordanceOverlay` / `FanSlotPlayableAffordanceUnaffordableOverlay`.
  The pre-pool surface for opponent hand (if any rendering
  surface exists for opponent fan slots — current scope is
  local-only) is not extended. Verified by:
  - grep across `client/src/ui/hand/` for any opponent-side query
    (`OwnerId::Opponent` etc.) attaching the new markers — zero
    hits.
  - the integration test asserts that an opponent-tagged dummy
    slot entity (synthesised in the test fixture) does NOT
    receive the new markers when the affordance system runs.

- [ ] **AC5 — Non-Minion card types treated as Playable**: GIVEN
  the same idle preconditions as AC1 AND the slot's card has
  `CardType ∈ { Spell, Trap, Structure, Field, Order, Instant }`,
  WHEN one tick runs, THEN `FanSlotPlayableAffordanceActive::Playable`
  is set (because `slot_is_affordable` returns `true` for non-
  Minion types per `drag_state_visuals.rs:363-365`). The
  Playable overlay is `Visible`; the Unaffordable overlay is
  `Hidden`. **Note**: this matches the current Minion-only mana
  semantics; if a future card pool adds mana-costing non-Minion
  types, the `slot_is_affordable` helper changes there and this
  story's behaviour follows naturally.

- [ ] **AC6 — Reactive updates on PlayerEconomyView change**:
  GIVEN the slot is currently in `FanSlotPlayableAffordanceActive::Unaffordable`
  (cost = 4, current = 2, reserve = 1, sum = 3 < 4), WHEN a fresh
  `S2CGoldUpdate` (or `S2CGameSnapshot`) drains and updates
  `PlayerEconomyView.current_mana` to 4 (sum = 5 >= 4) AND one
  more tick runs, THEN:
  - The slot flips to `FanSlotPlayableAffordanceActive::Playable`.
  - The Playable overlay becomes `Visible`; the Unaffordable
    overlay becomes `Hidden`.
  - No more than one frame's lag (the affordance sync system
    runs every frame in `PresentationSet::StateSync`).
  Verified by inserting the new economy values and observing
  the marker / visibility on the subsequent tick.

- [ ] **AC7 — Drag-active suppresses idle affordance**: GIVEN
  the same idle preconditions AND
  `Res<ActivePlacementDrag>::is_active() == true` (drag in
  flight), WHEN one tick runs, THEN both the Playable and
  Unaffordable overlays are `Visibility::Hidden` on ALL slots,
  AND `FanSlotPlayableAffordanceActive` is removed from all
  slots. The drag-state visuals (Story 020) own the frame.
  Verified by ECS query.

- [ ] **AC8 — Phase / mode gating**: GIVEN any of the following
  fails: (a) `Res<CurrentClientPhase> != Phase::Placement`;
  (b) `Res<HandUiMode> ∉ { Passive, Staging }` (i.e. `Hidden`,
  `Grid`, or `PassiveLocked`); WHEN one tick runs, THEN both
  Playable and Unaffordable overlays are `Visibility::Hidden`
  on ALL slots. The affordance is gated to the interactive
  PLACEMENT idle window only. **Note**: the BLOCKING scope is
  `Phase::Placement` only; `DraftShop` / `DraftAuction`
  extension is advisory and may be added by a follow-on story
  if product flagged.

- [ ] **AC9 — Staged-card suppression**: GIVEN the slot's
  `HandSlotCard(card_id)` is in `PendingPlacements::placements`
  (i.e. the card has been staged this PLACEMENT window), WHEN
  one tick runs, THEN both overlays are `Visibility::Hidden`
  on that slot, AND `FanSlotPlayableAffordanceActive` is
  removed. The staged-ghost dim treatment from Story 005 /
  Story 008 is preserved as the only visual on that slot.

- [ ] **AC10 — Empty slot suppression**: GIVEN a fan slot has
  NO `HandSlotCard` component (empty slot in the pre-pool),
  WHEN one tick runs, THEN both overlays are
  `Visibility::Hidden`. `FanSlotPlayableAffordanceActive` is
  absent. Verified by ECS query against the empty slots in the
  pre-pool.

- [ ] **AC11 — Integration test in
  `tests/integration/hand-ui/`**: GIVEN the post-implementation
  build, WHEN
  `cargo test -p client --test hand_ui_idle_playable_affordance_test`
  is run (file path TBD by `/dev-story`; likely
  `tests/integration/hand-ui/hand_ui_idle_playable_affordance_test.rs`),
  THEN it PASSES with at minimum 10 assertions covering AC1, AC2,
  AC3, AC4, AC5, AC6, AC7, AC8 (both sub-cases), AC9, AC10. The
  test drives state via direct resource insertion (mutate
  `PlayerEconomyView`, write `HandSlotCard`, set `HandUiMode`,
  insert / clear `ActivePlacementDrag` / `PendingPlacements`).
  No `Pointer<*>` event synthesis. Independent of R1 repair.

- [ ] **AC12 — Story 020 AC9 regression continues to PASS**:
  GIVEN the post-implementation build, WHEN
  `cargo test -p client --test hand_ui_drag_state_visuals_test`
  is run, THEN it PASSES (all 11 / 11 ECS-query assertions
  unchanged). The new affordance overlays do NOT contaminate
  Story 020's queries because they carry distinct markers
  (`FanSlotPlayableAffordanceOverlay` ≠ `DragStateOverlay`).
  **This AC is the contractual mechanism that preserves Story
  020 AC2 without editing Story 020.** If this AC fails, the
  `/dev-story` worker MUST resolve the regression (likely by
  ensuring the affordance markers / queries are properly
  disjoint from `DragStateOverlay`) before commit.

- [ ] **AC13 — ADR-002 + ADR-012 binding preserved**: GIVEN the
  post-implementation build, WHEN inspected, THEN:
  - The new systems read `Res<PlayerEconomyView>` (immutable),
    `Res<HandCardCatalog>` (immutable), `Res<HandUiMode>`
    (immutable), `Res<ActivePlacementDrag>` (immutable),
    `Res<PendingPlacements>` (immutable), `Res<CurrentClientPhase>`
    (immutable), per-slot `&FanSlotIndex` / `&HandSlotCard`
    (immutable).
  - They write only the per-slot marker components
    (`FanSlotPlayableAffordanceActive`) and the child overlay
    `Visibility`.
  - They do NOT add any `S2C*` / `C2S*` message;
    `shared/src/protocol.rs` diff is empty.
  - `liv-bevy-lightyear` is NOT activated.
  Verified by grep for new `ResMut<PlayerEconomyView>` (zero
  hits), `ResMut<HandCardCatalog>` (zero hits),
  `ResMut<ActivePlacementDrag>` (zero hits),
  `ResMut<PendingPlacements>` (zero hits), and by
  `git diff shared/src/protocol.rs` (empty).

- [ ] **AC14 — ADR-021 pre-pool discipline preserved**: GIVEN
  the post-implementation build, WHEN inspected, THEN:
  - `HAND_UI_ENTITY_COUNT` reflects the new overlay child
    entities (`+HAND_FAN_SLOT_COUNT` for a single overlay per
    slot, OR `+2 * HAND_FAN_SLOT_COUNT` for paired Playable /
    Unaffordable overlays; the bump matches the implementation
    decision).
  - No NEW top-level pre-pool entry is added (the overlays are
    CHILDREN of existing `FanSlotIndex` entities).
  - `hand_ui_plugin_scaffold_test` PASSES with the updated
    count.
  - `HandUiPlugin` and `HudPlugin` registration order is
    unchanged.

- [ ] **AC15 — Tween conflict-free**: GIVEN any tween installed
  by the affordance systems (worker discretion to add a subtle
  scale or alpha fade on the playable treatment), WHEN
  inspected, THEN it does NOT target a `Sprite` / `Node`
  component already targeted by an existing card lift /
  staging / drag-state tween. The `replace_tweenable` API is
  used for any install / cancel. (Worker may opt for a step
  change with no tween, as Story 020's hover overlay does.)

- [ ] **AC16 — No new Lightyear / protocol message**: GIVEN the
  post-implementation build, WHEN `git diff` is inspected for
  `shared/src/protocol.rs` / `shared/src/network/` /
  `client/src/network/` / `server/src/network/`, THEN no diff
  is present. `liv-bevy-lightyear` is NOT activated.

- [ ] **AC17 — Targeted regressions pass**: GIVEN the post-
  implementation build, WHEN run, THEN all PASS:
  - `cargo test -p client --lib`
  - `cargo test -p client --test hand_ui_drag_state_visuals_test`
    (Story 020 regression — AC12 above)
  - `cargo test -p client --test hand_ui_drag_to_board_cell_test`
  - `cargo test -p client --test hand_ui_drag_end_non_instant_test`
  - `cargo test -p client --test hand_ui_submit_prevalidation_test`
  - `cargo test -p client --test hand_ui_reserve_mana_strip_test`
  - `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
  - `cargo test -p client --test hand_ui_placement_timer_test`
  - `cargo test -p client --test hand_ui_placement_unstaging_test`
  - `cargo test -p client --test hand_ui_chrome_composition_test`
  - `cargo test -p client --test hand_ui_slot_onscreen_test`
  - `cargo test -p client --test hand_ui_viewport_sync_test`
  - `cargo test -p client --test ui_viewport_invariants_test`
  - `cargo test -p client --test hand_ui_plugin_scaffold_test`
    (entity-count assertion will need updating; AC14).

- [ ] **AC18 — No accept-risk closure claimed**: GIVEN the
  evidence, WHEN inspected, THEN it explicitly does NOT claim
  closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `TQ-S12-C1..C7`, R1 (drag-pipeline-dead bug),
  R2 (mana-preview missing-feature — sibling story 022), or
  `AUDIT-1076-02 / 03` (server-side placement loss).

- [ ] **AC19 — Sprint 17 / Sprint 18 disposition preserved**:
  GIVEN the story commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-17.md`,
  `production/sprints/sprint-18.md` (if extant),
  `production/stage.txt`, and PROMPT 761 gate-check artifact
  are diffed, THEN none of them are modified by this story.

- [ ] **AC20 — Hand UI EPIC count updated**: GIVEN the epic
  file `production/epics/hand-ui/EPIC.md`, WHEN updated by the
  `/story-done` paperwork, THEN the "Stories" table reflects
  this story 023 row consistently. PROMPT 1136 authoring
  **DOES** add the story 023 row in `Draft (future Sprint 18
  candidate)` status.

---

## Implementation Notes (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

The affordance system is a single sibling system, e.g.:

```rust
pub fn sync_hand_idle_playable_affordance_system(
    phase: Res<CurrentClientPhase>,
    mode: Res<HandUiMode>,
    active_drag: Res<ActivePlacementDrag>,
    pending_placements: Res<PendingPlacements>,
    economy: Res<PlayerEconomyView>,
    catalog: Res<HandCardCatalog>,
    slots: Query<(Entity, &FanSlotIndex, Option<&HandSlotCard>)>,
    mut playable_overlays: Query<
        (&ChildOf, &mut Visibility),
        With<FanSlotPlayableAffordanceOverlay>,
    >,
    mut unaffordable_overlays: Query<
        (&ChildOf, &mut Visibility),
        (
            With<FanSlotPlayableAffordanceUnaffordableOverlay>,
            Without<FanSlotPlayableAffordanceOverlay>,
        ),
    >,
    mut commands: Commands,
) {
    // Compute idle-active flag.
    let phase_ok = *phase == CurrentClientPhase::Placement;
    let mode_ok = matches!(*mode, HandUiMode::Passive | HandUiMode::Staging);
    let drag_inactive = !active_drag.is_active();
    let idle_active = phase_ok && mode_ok && drag_inactive;

    // Aggregate staged card_ids once.
    let staged_ids: Vec<_> = pending_placements
        .placements
        .iter()
        .map(|p| p.card_id)
        .collect();

    // Resolve per-slot affordance state.
    let mut slot_states: HashMap<Entity, FanSlotPlayableAffordanceActive> = HashMap::new();
    if idle_active {
        for (slot_entity, _slot_index, slot_card) in slots.iter() {
            let Some(card) = slot_card else { continue };
            if staged_ids.contains(&card.0) { continue; }
            let state = if slot_is_affordable(card.0, &catalog, &economy) {
                FanSlotPlayableAffordanceActive::Playable
            } else {
                FanSlotPlayableAffordanceActive::Unaffordable
            };
            slot_states.insert(slot_entity, state);
        }
    }

    // Patch overlays / markers.
    for (child_of, mut visibility) in &mut playable_overlays {
        let parent = child_of.parent();
        *visibility = if matches!(
            slot_states.get(&parent),
            Some(FanSlotPlayableAffordanceActive::Playable),
        ) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (child_of, mut visibility) in &mut unaffordable_overlays {
        let parent = child_of.parent();
        *visibility = if matches!(
            slot_states.get(&parent),
            Some(FanSlotPlayableAffordanceActive::Unaffordable),
        ) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (slot_entity, _, _) in slots.iter() {
        match slot_states.get(&slot_entity) {
            Some(state) => {
                commands.entity(slot_entity).insert(*state);
            }
            None => {
                commands
                    .entity(slot_entity)
                    .remove::<FanSlotPlayableAffordanceActive>();
            }
        }
    }
}
```

The system runs in `PresentationSet::StateSync`. Order it AFTER
`sync_hand_drag_state_visuals_system` so the drag-state's frame
ownership is unambiguous (when a drag is active, the affordance
system clears its own markers and the drag-state system runs
unaffected).

The new overlay child nodes are spawned in `spawn_hand_ui` next
to the existing Story 020 overlay children, e.g.:

```rust
pub fn spawn_fan_slot_playable_affordance_overlays(
    commands: &mut Commands,
    slot: Entity,
    slot_index: u8,
) {
    commands.spawn((
        Name::new(format!("Fan Slot {slot_index} Playable Affordance Overlay")),
        super::HandUiEntity,
        FanSlotPlayableAffordanceOverlay,
        playable_overlay_node(),
        BorderColor::all(accent_color()),   // §7 ACCENT — symbol
        Visibility::Hidden,
        ChildOf(slot),
    ));
    commands.spawn((
        Name::new(format!("Fan Slot {slot_index} Unaffordable Affordance Overlay")),
        super::HandUiEntity,
        FanSlotPlayableAffordanceUnaffordableOverlay,
        unaffordable_overlay_node(),
        BackgroundColor(dim_overlay_color()), // OVERLAY_DIM_ALPHA — symbol
        Visibility::Hidden,
        ChildOf(slot),
    ));
}
```

Neither overlay carries `DragStateOverlay`. AC2 (Story 020) query
is preserved.

`HAND_UI_ENTITY_COUNT` bumps by `2 * HAND_FAN_SLOT_COUNT` to
account for the two new child overlays per slot. `hand_ui_plugin_scaffold_test`
expectation is updated in the same commit.

---

## Likely Files (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/mod.rs` | Declare new submodule `playable_affordance` (or extend `drag_state_visuals`); spawn the two new overlay children under each `FanSlotIndex` at session start; register `sync_hand_idle_playable_affordance_system` in `HandUiSystemSet::StateSync` after `sync_hand_drag_state_visuals_system`; bump `HAND_UI_ENTITY_COUNT` by `2 * HAND_FAN_SLOT_COUNT` (or `+HAND_FAN_SLOT_COUNT` if the worker chooses single-overlay-toggled). |
| `client/src/ui/hand/playable_affordance.rs` (NEW, recommended) | Host the new marker components (`FanSlotPlayableAffordanceOverlay`, `FanSlotPlayableAffordanceUnaffordableOverlay`, `FanSlotPlayableAffordanceActive`) and the sync system. Re-exported from `mod.rs`. |
| `client/src/ui/hand/drag_state_visuals.rs` | Possibly: `pub use slot_is_affordable;` to expose the existing helper to the new sibling submodule (or refactor the helper to a shared location like `client/src/ui/hand/affordability.rs`). NO edit to `sync_hand_drag_state_visuals_system` body. |
| `tests/integration/hand-ui/hand_ui_idle_playable_affordance_test.rs` (NEW) | Integration test per AC11. ECS-query-driven; drives state via direct resource insertion. |
| `tests/integration/hand-ui/hand_ui_plugin_scaffold_test.rs` (existing) | Update `HAND_UI_ENTITY_COUNT` expectation to match the new pre-pool count. |
| `design/gdd/hand-ui.md` | Optional light addition: one row documenting the idle affordance. Worker discretion. |
| `docs/architecture/tr-registry.yaml` | Add `TR-HU-010 — Idle hand playable affordance` row. Worker discretion. |
| `production/epics/hand-ui/story-023-hand-idle-playable-affordance.md` | This file. Status flipped Draft → Ready by `/story-readiness` post Sprint 18 activation; Ready → Done by `/story-done`. |
| `production/epics/hand-ui/EPIC.md` | `/story-done` paperwork: refresh story-023 row to `Done`. PROMPT 1136 authoring adds the row in `Draft`. |

**Explicitly out of scope for the `/dev-story` worker** (any of these
constitutes a scope violation per the Forbidden Control Manifest
Rules):

- `production/epics/hand-ui/story-020-hand-drag-state-visuals.md`
  (Story 020 is Complete; NO EDIT).
- `shared/src/protocol.rs` — no protocol-shape edit.
- `server/` — no server-side affordance code.
- `client/src/network/` / `server/src/network/` — no networking.
- `client/src/ui/shop_auction/`, `client/src/ui/hud/`,
  `client/src/ui/lobby.rs` — out of host module.
- `client/src/presentation/board_rendering*` — board-side
  highlights are Story 006 ownership.
- `production/sprints/*`, `production/sprint-status.yaml`,
  `production/stage.txt`, PROMPT 761 gate-check artifact.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`.
- R1 / R2 / `AUDIT-1076-02 / 03` repair sites.

---

## Out of Scope

- **R1 — drag pipeline dead in shipped build**. Separate repair
  prompt. This story's AC11 test drives state via direct
  resource insertion to remain independent of R1.
- **R2 — mana preview during drag**. Sibling future-Sprint-18
  candidate at `story-022-hand-mana-preview-during-drag.md`.
- **AUDIT-1076-02 / AUDIT-1076-03** — server-side placement
  loss. Server-only fix; out of host module.
- **Editing Story 020** (`production/epics/hand-ui/story-020-hand-drag-state-visuals.md`).
  Story 020 is Complete and its AC2 idle-baseline query is
  preserved by construction (the new affordance markers are
  distinct from `DragStateOverlay`). See "Story 020 AC2
  Reconciliation" section above.
- **Opponent-hand affordance**. Local-only scope. AC4 explicit.
- **Affordance in DraftShop / DraftAuction phases**. BLOCKING
  scope is `Phase::Placement` only (AC8). Extension to other
  phases is a follow-on story.
- **Affordance for already-staged cards**. Suppressed; the
  staged ghost dim treatment from Story 005 / Story 008 is
  preserved (AC9).
- **Affordance for empty slots**. No treatment (AC10).
- **Affordance for non-Minion cards**. Treated as Playable
  because `slot_is_affordable` returns `true` for non-Minion
  types (AC5). If product wants per-type affordance variation,
  it is a follow-on story.
- **Tweens on the affordance treatment**. Worker discretion;
  default is step change (consistent with Story 020 hover
  overlay).
- **WCAG contrast verification** on the playable / unaffordable
  treatments. `QA-COND-0005` accepted-risk.
- **Final-art replacement** on the fan-slot card chrome.
  `PAW-TD-*-a` accepted-risk.
- **Sprint 18 activation**. PROMPT 1136 (this authoring) does
  NOT activate Sprint 18.
- **`/qa-plan sprint-18` authoring**. Owned by a separate prompt
  after Sprint 18 activation.
- **`/story-readiness` on this story**. Run as a separate prompt
  after Sprint 18 activation.
- **`/dev-story` on this story**. Run only after Sprint 18
  activation, after `/qa-plan sprint-18` (if any), and after
  `/story-readiness` passes against Sprint 18 activation HEAD.
- **Polish → Release gate-check retry**. PROMPT 761 FAIL
  preserved.
- **Stage advance from Polish to Release**. `production/stage.txt`
  remains `Polish`.

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements
against these — do not invent new test cases during implementation.*

- **AC1 — Playable affordance applies**:
  - Given: PLACEMENT; `HandUiMode::Passive`; drag inactive;
    slot 0 holds Minion `cost = 2`;
    `PlayerEconomyView { current: 3, reserve: 0, cap: 3 }`.
  - When: one tick.
  - Then: slot 0 has `FanSlotPlayableAffordanceActive::Playable`;
    Playable overlay Visible; Unaffordable overlay Hidden.

- **AC2 — Subdued / unaffordable treatment**:
  - Given: PLACEMENT; `HandUiMode::Passive`; drag inactive;
    slot 0 holds Minion `cost = 5`;
    `PlayerEconomyView { current: 1, reserve: 1, cap: 3 }`.
  - When: one tick.
  - Then: slot 0 has `FanSlotPlayableAffordanceActive::Unaffordable`;
    Unaffordable overlay Visible (OVERLAY_DIM_ALPHA tint);
    Playable overlay Hidden.

- **AC3 — Mutual exclusion**:
  - Given: any single slot in any test fixture.
  - When: the affordance sync runs.
  - Then: at most one of Playable / Unaffordable is Visible on
    that slot.

- **AC4 — Opponent unaffected**:
  - Given: a fixture with a synthesised opponent-tagged dummy
    slot entity bearing `HandSlotCard(opponent_card_id)`.
  - When: one tick.
  - Then: opponent slot has neither
    `FanSlotPlayableAffordanceActive` nor visible affordance
    overlay; local slot affordance behaves per AC1.

- **AC5 — Non-Minion**:
  - Given: PLACEMENT idle; slot 0 holds a `CardType::Spell`
    card with `cost = 99`; `PlayerEconomyView { current: 0,
    reserve: 0, cap: 3 }`.
  - When: one tick.
  - Then: slot 0 is Playable (non-Minion returns `true` from
    `slot_is_affordable`).

- **AC6 — Reactive update on economy change**:
  - Given: starting in AC2 state.
  - When: `S2CGoldUpdate` lifts `current_mana` to 5 (sum 6 >=
    5); one tick.
  - Then: slot 0 flips to Playable on the next tick.

- **AC7 — Drag-active suppression**:
  - Given: AC1 state.
  - When: `ActivePlacementDrag::start(...)` writes
    `is_active() == true`; one tick.
  - Then: all affordance overlays Hidden; no
    `FanSlotPlayableAffordanceActive` markers on any slot.

- **AC8 — Phase / mode gating**:
  - Given: `HandUiMode::PassiveLocked` AND PLACEMENT phase.
  - When: one tick.
  - Then: all affordance overlays Hidden.
  - Given: `Phase::DraftShop` AND `HandUiMode::Passive`.
  - When: one tick.
  - Then: all affordance overlays Hidden.

- **AC9 — Staged-card suppression**:
  - Given: slot 0 holds Minion `cost = 2`; affordable;
    `PendingPlacements::placements` contains a
    `PlacedCardSubmit { card_id: slot_0_card_id, ... }`.
  - When: one tick.
  - Then: slot 0 has no affordance overlay Visible.

- **AC10 — Empty slot suppression**:
  - Given: slot 9 has no `HandSlotCard` (empty in pre-pool).
  - When: one tick.
  - Then: slot 9 has no affordance overlay Visible.

- **AC12 — Story 020 regression preserved**:
  - When: `cargo test -p client --test hand_ui_drag_state_visuals_test`
    runs.
  - Then: PASS 11 / 11 (no regression).

---

## Test Evidence

**Story Type**: UI + Integration (paired BLOCKING test).

**Required evidence**:
- `tests/integration/hand-ui/hand_ui_idle_playable_affordance_test.rs`
  (NEW; BLOCKING) — AC11.
- Story 020 regression suite (`hand_ui_drag_state_visuals_test`)
  continues to PASS — AC12.
- `hand_ui_plugin_scaffold_test` PASSES with updated entity
  count — AC14.

**Status**: [ ] Created and passing (BLOCKED until `/dev-story`
runs post Sprint 18 activation).

**Advisory evidence**:
- `production/qa/evidence/sprint-18-hand-idle-affordance/README.md`
  (optional manual walkthrough; lead sign-off on the playable /
  unaffordable visual compositions). Trivially satisfied by the
  AC11 integration test alone.

---

## Performance Budget

Per ADR-021 Presentation steady-state budget of `< 1 ms` per frame.
The affordance system performs:

- `O(1)` resource reads.
- `O(n)` over `PendingPlacements::placements` (`n ≤ 10`) — set
  membership pass.
- `O(s)` over fan slots (`s = 10`) — one `HandCardCatalog` lookup
  per slot.
- `O(s)` over per-slot overlay child queries — one `Visibility`
  write per overlay.

Expected per-frame cost: `< 50 µs`. Well within ADR-021 budget.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker conflates this missing-feature story with the R1 drag-pipeline-dead bug repair. | Low | High | "No-Claim Banner" and "Out of Scope" disjoin R1; AC11 drives state via direct resource insertion. |
| Worker edits Story 020 to "amend AC2" instead of using a distinct marker. | Medium | High | "Story 020 AC2 Reconciliation" section forbids the edit; AC12 BLOCKING gate verifies Story 020's regression passes; the affordance marker design is documented. |
| Worker adds `DragStateOverlay` to the new affordance overlay child nodes (contaminating Story 020 query). | Low | High | AC1 / AC2 / Forbidden rules forbid; AC12 gate catches at test runtime. |
| Worker writes the projection through `ResMut<PlayerEconomyView>`. | Low | High | AC13 forbids; grep gate verifies. |
| Worker applies affordance to opponent hand surface. | Low | Medium | AC4 explicit; grep + integration-test fixture covers. |
| Worker omits the staged-card suppression and the staged ghost reads through the affordance tint. | Medium | Medium | AC9 explicit; integration-test asserts. |
| Worker bumps `HAND_UI_ENTITY_COUNT` incorrectly and breaks `hand_ui_plugin_scaffold_test`. | Medium | Low | AC14 explicit; integration-test asserts. |
| Worker introduces a tween that conflicts with Story 020's hover overlay or with the existing card lift / staging tweens. | Low | Medium | AC15 explicit; default is step change (no tween). |
| Worker invents a new Lightyear `S2CPlayability` round-trip. | Low | High | AC13 + AC16 forbid; `liv-bevy-lightyear` NOT activated. |
| `slot_is_affordable` helper visibility is `pub(crate)` not `pub`, blocking sibling-submodule access. | Medium | Low | Worker can `pub use` from `drag_state_visuals.rs` or refactor the helper into a shared `affordability.rs`. Either resolution is acceptable; AC12 / AC13 verify the outcome. |
| Worker activates Sprint 18 as a side effect of `/dev-story` paperwork. | Low | Medium | No-Claim Banner forbids. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for the PROMPT 1136 authoring run itself
(which is paperwork-only):

- `production/sprint-status.yaml` top-level `sprint:` field reads
  `18` (after Sprint 18 activation) and the row for
  `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001` is `ready` at the
  time `/dev-story` is dispatched.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-18.md` shows the ACTIVATED banner.
- PROMPT 761 Polish → Release gate-check FAIL evidence preserved.
- `production/qa/qa-plan-sprint-18.md` (if extant) references this
  story.
- `/story-readiness` on this story file returns READY against the
  Sprint 18 activation HEAD.
- Sprint 12 story 019 disposition preserved on `origin/main`.
- `git diff --check` and `git diff --cached --check` pass before
  commit.
- Story 020 file unchanged on `origin/main` (this story does NOT
  edit it).
- R1 repair status: this story does NOT require R1 to be merged
  (AC11 is R1-independent). Sequencing R1 first may improve the
  manual walkthrough evidence quality (since the player can
  trigger the Story 020 `Disabled` drag-time overlay to compare
  against the idle affordance), but is not a gate.

---

## Authoring Trail

- 2026-05-18 — PROMPT 1136 — Story file authored as future
  Sprint 18 candidate `S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001`.
  Worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s18-hand-mana-affordance-stories`,
  branch `work/s18-hand-mana-affordance-stories`, base
  `origin/main@05192b5f830c5d5b17ed7af07df37f56187130fc` (PROMPT
  1125 `story-done(s17): close S17-OPS-VULKAN-VALIDATION-GATING-001`).
  Files touched by this authoring run: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (story-list row added; count-
  deferral note updated). Sibling
  `production/epics/hand-ui/story-022-hand-mana-preview-during-drag.md`
  authored in the same run as the R2 missing-feature candidate
  (PROMPT 1127 §R2). Sprint 18 NOT activated. No code change. No
  edit to Story 020 file (the AC2 reconciliation is internal to
  this story file; AC12 BLOCKING gate verifies the regression).
  No `/dev-story`, `/story-readiness`, `/story-done`,
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`,
  `/qa-plan`, `cargo`, or `trunk` command run. ADR-002 + ADR-012 +
  ADR-021 binding preserved; Sprint 12 story 019 disposition
  preserved; PROMPT 761 gate-check FAIL preserved;
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `S8-QA-001-W1`,
  `TQ-S12-C1..C7` all preserved verbatim. R1 repair status
  unchanged (separate prompt). PROMPT 1127 R3 missing-feature
  (designed-out) is now formally documented as a Sprint 18
  candidate story; no implementation is performed by this
  authoring run.
