# Story 020: S12-UX-HAND-DRAG-STATE-VISUALS-001 -- Hand-Card Drag-State Visual Differentiation (Read-Only Over Ephemeral Drag State)

> **Epic**: Hand UI
> **Story ID**: S12-UX-HAND-DRAG-STATE-VISUALS-001
> **Status**: Draft -- Sprint 15 candidate; **Sprint 15 NOT activated**;
> pending `/story-readiness` after Sprint 15 activation
> **Layer**: Presentation -- Hand UI (visual differentiation only)
> **Type**: UI -- layout/composition + integration test (ECS marker / color /
> z-layer assertions)
> **Sprint**: Sprint 15 Should Have (Tier 1 Should-priority adjacent row;
> independent of the 14 main-rank UI clean-pass surfaces; touches hand UI
> only per `docs/ux/ui-clean-pass-roadmap.md` "Tier 1 Should-Priority
> Adjacent Rows" table at 0.5d)
> **Authored**: 2026-05-16 by PROMPT 991 (worktree
> `D:\_DEV\claude-code-game-studios-worktrees\sprint-15-hand-drag-state-visuals-story-991`,
> branch `story-authoring/sprint-15-hand-drag-state-visuals`)
> **Authoring source-of-truth**: `origin/main@2c84d6e37f2ec58b729064b6dbe4c9b017e5ceb3`
> (PROMPT 990 `integrate(s15): merge Sprint 15 plan draft (PROMPT 990)`)

---

## Status / No-Claim Banner

This story file is authored by PROMPT 991 as the Sprint 15 Should Have
candidate `S12-UX-HAND-DRAG-STATE-VISUALS-001` named in the Sprint 15
draft plan at `production/sprints/sprint-15.md` §"Should Have". Sprint 15
itself is `draft -- authored 2026-05-16 by PROMPT 988` and is **NOT
activated** by PROMPT 991. Activation happens via a separate prompt
mirroring the PROMPT 826 / PROMPT 897 pattern.

PROMPT 991 (this authoring run) does **NOT**:

- Activate Sprint 15.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-15.md` or any other sprint plan
  file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/`
  file.
- Retry the PROMPT 761 Polish->Release gate-check.
- Touch the existing Sprint 12 drag-runtime story files (017 / 018 /
  019) or the closed `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  evidence.

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion (`PAW-TD-*-a`); drag-state
  visual differentiation reuses the existing PAW-* placeholder card
  chrome with **no final-art treatment introduced by this story**
- closure of any other Sprint 15 candidate row from `docs/ux/ui-clean-pass-roadmap.md`
- **fix of the underlying drag-runtime bug from Sprint 12 story 019**
  (`closed-with-conditions / cannot-reproduce` after the PROMPT 814
  second time-box exhaustion; `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  status preserved verbatim). This story is **layout / visual state**
  work over already-extant client-side drag ephemeral state; it does
  **NOT** reproduce, repair, or retest the Sprint 12 story 019 runtime
  divergence, and no third same-scope retest is authorised per
  `TQ-S12-C2`.
- closure of `TQ-S12-C7` or any other Sprint 12 Team-QA condition
- reopen of any closed Sprint 14 row
- advance of stage from `Polish` to `Release`

Sprint 10 / Sprint 11 / Sprint 12 / Sprint 13 / Sprint 14 dispositions
unchanged. PROMPT 761 Polish->Release gate-check FAIL evidence preserved
at `production/gate-checks/gate-polish-release-2026-05-12.md`. PROMPT 802
audit roadmap accept-risk boundaries (`PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`) preserved verbatim.

**No optimistic client-side authority is introduced or proposed by this
story.** The drag-state visual differentiation reads from the already-
extant `Res<ActivePlacementDrag>` and the pre-pooled `FanSlotIndex` /
`HandDragSprite` entities; it adds no new server-authoritative state,
no new Lightyear message, no new protocol shape, and no new client-side
authority over stage / activate / submit. **ADR-002 + ADR-012 binding
preserved**: read-only over client-side ephemeral drag state; no new
server-authoritative state and no protocol-shape change.

---

## Source Finding

- `docs/ux/ui-clean-pass-roadmap.md` §"Tier 1 Should-Priority Adjacent
  Rows (PROMPT 802 §6 second-batch candidates; not in the 14)" lists
  this slug at **0.5d, "Independent of the 14; orthogonal to ranks
  7-12; touches hand UI only"** with provenance "Net-new / PROMPT 802
  §3.3 HA3".
- `docs/ux/global-ui-design-spec.md` §"Spec Adoption Matrix" / "Tier 1
  Should-priority adjacent rows" row for `S12-UX-HAND-DRAG-STATE-VISUALS-001`
  reads: "§3 (`UiOverlay` drag ghost layer) + §6 overlay alpha. Drag
  ghost reads `OVERLAY_DIM_ALPHA`."
- `production/sprints/sprint-15.md` §"Should Have" table row for
  `S12-UX-HAND-DRAG-STATE-VISUALS-001` cites this story's role:
  "visual differentiation for hand-card drag states (idle / hover /
  drag / drop-target / disabled) using Tier 0 token primitives". The
  Sprint 15 plan explicitly names: "Story file NEW; must be authored
  before activation at `production/epics/hand-ui/story-XXX-hand-drag-state-visuals.md`".
- PROMPT 802 audit `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`
  §3.3 HA3 names the underlying hand-card visual-state defect class
  (no drag-state visual differentiation; hand cards read the same
  under idle / hover / drag / drop-target / disabled).
- PROMPT 685 row (legacy audit) is folded into the same slug per the
  reconciliation matrix in `docs/ux/ui-clean-pass-roadmap.md`.

---

## Problem Class / Prevention Target

**Defect class**: hand cards in the current playable client render
visually identically across the full drag-state machine (`Idle`,
`Hover`, `Drag`, valid `DropTarget`, `Disabled`). The `HandDragSprite`
pre-pooled entity exists and tracks the cursor (PROMPT 696 / story
017), and `Res<ActivePlacementDrag>` already exposes the ephemeral
drag state, but the static fan-slot card entities and the drag-sprite
do not paint distinguishing visual cues. This:

1. Forces friend-game players to memorise which card is "lifted" vs
   "hovered" vs "available", with no on-screen affordance for the
   difference. The card-fan reads as a static row of identical card
   chrome regardless of pointer state.
2. Provides no visual cue when a card is **disabled** (e.g. card is
   already staged, hand is locked because `HandUiMode != Staging`,
   or the card costs more mana than the player has). Players currently
   attempt drags that the existing system rejects silently.
3. Provides no positive feedback for a **valid drop-target** lane /
   cell during PLACEMENT drag, distinct from board ghost preview --
   the hand-side surface is silent about which fan slot the drag
   *originated* from while the drag is in flight.
4. Has no consumer for the Tier 0 `OVERLAY_DIM_ALPHA` /
   `OVERLAY_SCRIM_ALPHA` (§6) tokens nor the §3 `UiOverlay` z-layer
   on the hand-card surface, even though the global UI design spec
   (`docs/ux/global-ui-design-spec.md` §"Tier 1 Should-priority
   adjacent rows" row) explicitly names this story as the consumer
   of those Tier 0 primitives on the hand surface.

**Prevention target**: introduce visual differentiation for the five
drag states of a hand-card entity by reading the already-extant
`Res<ActivePlacementDrag>`, `HandUiMode`, and per-slot card-staging
state, and applying named visual treatments composed from the Tier 0
token primitives already on `origin/main` (z-layers, typography,
overlay alpha, spacing, colour palette). The states render as:

| Drag state | Trigger / source | Visual treatment (composition from Tier 0 primitives) |
|------------|-------------------|--------------------------------------------------------|
| `Idle` | Default. No pointer hover, no active drag, not disabled. | Baseline fan-slot card chrome unchanged. No tint, no scrim, no z-layer change. |
| `Hover` | Pointer is over the fan-slot card entity, no active drag in flight. | Lift cue (border / outline using §7 `ACCENT` token) AND/OR a subtle scale tween (existing `bevy_tweening` already available; reuse). No z-layer change. |
| `Drag` | `ActivePlacementDrag::is_active()` is true AND this slot's `Entity` matches `ActivePlacementDrag::card`. Source slot dims (out-of-fan visual) while the `HandDragSprite` (already pre-pooled per story 017 PROMPT 696) paints the lifted card at `UiOverlay` (§3) with `OVERLAY_DIM_ALPHA` (§6) treatment as the in-flight ghost. | Source slot: dim tint via `OVERLAY_DIM_ALPHA` overlay child node. `HandDragSprite`: full opacity, painted at z-layer `UiOverlay` (§3) above board content. |
| `DropTarget` | `ActivePlacementDrag::is_active()` is true AND the cursor is currently over a **fan-side** drop target (e.g. Instant fan-plate drop per story 007). Note: board-cell drop-target ghosting is owned by Board Rendering -- this story scopes the **fan-side** drop-target affordance only. | Fan-plate region tint via `OVERLAY_SCRIM_ALPHA` (§6) overlay child node OR a semantic-success outline using `SEMANTIC_SUCCESS` (§7). |
| `Disabled` | `HandUiMode::PassiveLocked` for the slot, OR the card is already staged in `PendingPlacements`, OR the slot card cannot be afforded (per existing submit pre-validation in story 010). | Card tint via `OVERLAY_DIM_ALPHA` (§6) overlay child node AND/OR desaturated fallback. Cursor reads `not-allowed`-equivalent affordance if and only if such an affordance is already supported (worker discretion). |

The treatment shapes above are **specifications**, not implementation;
the `/dev-story` worker chooses the precise composition of border /
outline / scrim / tint / scale within the bounds of the Tier 0
primitives. **The values themselves are already ratified** by the
global UI design spec (PROMPT 911 + 922) -- this story does NOT
re-author Tier 0 numeric values and does NOT introduce new tokens.

The drag-state visual differentiation is **read-only over server-
authoritative state and over client-side ephemeral drag state**. It
does NOT introduce any new server message, any new client-side
authority over stage / activate / submit, or any new client-side
optimism. ADR-002 binding preserved. ADR-012 binding preserved (no
new server-authoritative state surfaces are added; the drag-state
visuals are derived purely from the already-extant
`Res<ActivePlacementDrag>` ephemeral state and the pre-pooled
hand-UI entities).

---

## Context

### Existing surface

- **`client/src/ui/hand/mod.rs`** (per ADR-021): the canonical Hand
  UI plugin owns `HandUiPlugin`, the pre-pooled fan-slot entities
  carrying `FanSlotIndex(u8)` (10 slots), the pre-pooled
  `HandDragSprite` ghost entity (PROMPT 696 / story 017), and the
  resources `HandUiMode` (`Hidden` / `Grid` / `Passive` /
  `PassiveLocked` / `Staging`) and `ActivePlacementDrag`
  (`card: Option<Entity>`, `card_id: Option<CardId>`,
  `owner_id: Option<PlayerId>`, `target_kind: Option<PlacementTargetKind>`,
  `cursor_world_position: Option<Vec2>`). The hand-fan slots and
  drag sprite are toggled by `Visibility` only -- no per-round
  spawn / despawn (ADR-021 Impl Guideline 5 preserved).
- **`client/src/ui/design_tokens/`** (Tier 0 modules already on
  `origin/main`): `z_layers.rs` (§3 `UI_BASE`, `UI_OVERLAY`,
  `MODAL`, `TOAST` constants, PROMPT 902), `typography.rs` (§5
  Caption / Body / H3 / H2 / H1 / Display constants, PROMPT 908),
  `overlays.rs` (§6 `OVERLAY_DIM_ALPHA = 0.45`,
  `OVERLAY_SCRIM_ALPHA = 0.55`, `OVERLAY_TOAST_ALPHA = 0.80`),
  `spacing.rs` (§4 `SPACING_XS` .. `SPACING_XL`, story 004), and
  `strips.rs` (§9 `HeaderBar` / `FooterBar` / `HandBar` primitives).
  All values are ratified by `docs/ux/global-ui-design-spec.md` per
  PROMPT 911 / 922.
- **`client/src/ui/hand/mod.rs::ActivePlacementDrag`**: already
  exposes `is_active()` (true iff `card.is_some() && target_kind.is_some()`)
  and `cursor_world_position`. The drag-state visual systems read
  this resource; they do NOT mutate it.
- **`client/src/ui/hand/mod.rs::PendingPlacements`**: already exposes
  `placements: Vec<PlacedCardSubmit>` -- used to derive the
  "already-staged" disabled treatment.
- **PROMPT 968 / PROMPT 975 HUD opponent figurine** (story 017 in
  the HUD epic) sets the precedent for "Tier 1 Should-priority
  adjacent row, layout/composition only, read-only over server-
  authoritative state" pattern. This story mirrors that pattern on
  the hand-UI surface for ephemeral drag state.
- **PROMPT 802 §3.3 HA3** records the underlying audit defect.
- **`docs/ux/ui-clean-pass-roadmap.md`** sequences this slug as
  Tier 1 Should-priority adjacent at 0.5d, independent of the 14;
  parallel-safe with the other Sprint 15 Should + Nice rows
  (board rendering spec is doc-only; interaction state primitives
  touch `client/src/ui/design_tokens/`, a different file scope
  than `client/src/ui/hand/`).

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` -- hand-UI state machine and the
  placement drag-to-stage flow already document `HandUiMode`
  transitions and the `Idle -> Dragging -> Staged -> Committed`
  state path. This story adds **visual differentiation** for the
  drag states; a light GDD addition (one row in the visual-state
  table) is in scope for the `/dev-story` paired with the implementation
  edit, OR may be deferred to a small follow-on doc story per
  producer / implementation-prompt scope decision.
- **`design/ux/hand-ui.md`** lines ~210-230 describe the dragging /
  valid-target-hover / staged / un-staging state machine. This story
  surfaces the visual treatment for those states; no UX-spec change
  required because the spec already prescribes "Dragging card", "Valid
  board target hover", and "Staged board card" as distinct states.
- **ADR-021** (Presentation Layer Architecture): preserved unchanged.
  `HandUiPlugin` retains plugin registration order #3 (after
  `CardAnimationsPlugin` and `BoardRenderingPlugin`); pre-pooled
  entity count unchanged (10 fan slots + 9 DRAFT_INITIAL grid slots
  + 1 drag sprite); per-state visual differentiation is overlay /
  child-node additive composition, NOT a respawn / re-pool of the
  fan-slot entities (Impl Guideline 5 preserved). The drag sprite
  remains a bevy_ui `Node`, not a world-space `Sprite` (Impl
  Guideline 8 preserved).
- **ADR-002** (Client-Server Authority): preserved. Drag-state
  visuals are read-only over `Res<ActivePlacementDrag>` ephemeral
  state and the per-slot `HandSlotCard` / `PendingPlacements`
  state -- all of which are already on the client. No client-side
  optimism is added.
- **ADR-012** (SessionReady Delivery): preserved. Drag-state visual
  differentiation does not introduce any new server-authoritative
  state surface, any new resource that must be present before a
  trigger fires, or any change to the SessionReady delivery path.
  The story is read-only over already-extant client-side ephemeral
  drag state.
- **ADR-009** (RSM Phase State): preserved. Hand UI continues to
  read `Res<CurrentClientPhase>`; `MessageReceiver<S2CPhaseChanged>`
  is never drained directly (EPIC.md §"Key ADR-021 Constraints for
  This Epic" rule 3).
- **ADR-008** (Lightyear Channel Config): no new channel; this story
  does not introduce any new Lightyear message.
- **ADR-011** (Reconnect + Snapshot): drag-state visuals reset
  automatically on reconnect because `ActivePlacementDrag` and
  `HandUiMode` are already part of the per-session resource set
  that the existing reconnect rebuild path reinitialises (story 013).
- **ADR-023** (Placement Timer Accessibility Authority): unchanged
  by this story.
- **TR registry**: no new TR. This is a visual-treatment extension
  of TR-HU-002 (PLACEMENT drag-to-stage state machine) and a
  composition-time consumer of the §3 / §6 / §7 token sets already
  ratified by `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (PROMPT 911 /
  912 / 922).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits under
  `client/src/ui/hand/` or `tests/integration/hand-ui/`. The
  `/dev-story` implementation prompt MUST activate this skill
  before editing.
- **Lightyear**: not applicable to this story. The drag-state visual
  differentiation does NOT touch Lightyear protocol, `S2C*` messages,
  `C2S*` messages, or any networking surface. `liv-bevy-lightyear`
  is NOT required.
- **`bevy_tweening`**: optional. If the `/dev-story` worker chooses
  to add a subtle scale / opacity tween for the `Hover` state, the
  existing `bevy_tweening` dependency already used by the hand-UI
  card lift / staging tweens is reused (no new dependency). All
  tween installs must obey the EPIC.md §"Key ADR-021 Constraints"
  rule 6 (cancel-and-replace via `set_tweenable()`; no despawn +
  respawn of hand entities mid-animation).

### Control Manifest Rules

- **Required**: Drag-state visual treatments are composed from the
  existing Tier 0 token modules under `client/src/ui/design_tokens/`
  (z_layers / typography / overlays / spacing / colors). No new
  numeric design tokens are authored by this story.
- **Required**: All systems that update drag-state visuals are gated
  on `HandUiMode::Staging` OR on `ActivePlacementDrag::is_active()`
  (matching the story 017 PROMPT 696 producer-scope precedent).
  Systems that only update the `Disabled` treatment for non-Staging
  modes are gated on the corresponding `HandUiMode` value (e.g.
  `PassiveLocked`).
- **Required**: The `HandDragSprite` z-layer reads `UI_OVERLAY` (§3
  z_layers) explicitly; no hard-coded `GlobalZIndex(N)` literal is
  re-introduced on the drag-sprite spawn.
- **Required**: Overlay tints applied to the source / disabled fan
  slots read `OVERLAY_DIM_ALPHA` (§6 overlays); the fan-plate
  drop-target tint reads `OVERLAY_SCRIM_ALPHA` (§6). Source-of-truth
  references the constants, not the numeric literals.
- **Required**: `Res<ActivePlacementDrag>` is read-only inside the
  drag-state visual systems. Mutation of `active_drag` remains owned
  by the existing producers / consumers (PROMPT 696 story 017 and
  the existing `handle_placement_drag_*` systems).
- **Required**: `liv-bevy-018` skill applies to all `.rs` edits.
- **Required**: ADR-021 plugin registration order preserved
  (`HandUiPlugin` remains #3 inside `PresentationPlugin`).
- **Required**: ADR-021 Impl Guideline 5 preserved (pre-pooled
  entities only; no per-round spawn / despawn of fan-slot or
  drag-sprite entities for drag-state changes).
- **Required**: ADR-002 + ADR-012 binding preserved (read-only over
  client-side ephemeral drag state; no new server-authoritative
  state; no new protocol shape).
- **Required**: ADR-001 preserved (drag-state visuals carry no
  objective identity / `was_fake` data).
- **Required**: Integration test asserts the drag-state visual
  treatment via ECS marker / colour / z-layer queries (no rendered
  pixel snapshot tests).
- **Forbidden**: Introducing a new server-authoritative state for
  drag (no `S2CDragState`, no `C2SDragState`, no new Lightyear
  message).
- **Forbidden**: Reproducing, repairing, or retesting the Sprint 12
  story 019 underlying drag-runtime bug (closed
  `closed-with-conditions / cannot-reproduce` by PROMPT 814). The
  drag-runtime question is **out of scope**; a third same-scope
  retest is **not authorised** per `TQ-S12-C2`.
- **Forbidden**: Final-art replacement on the card chrome
  (`PAW-TD-*-a` preserved). Drag-state visuals reuse the existing
  placeholder card art.
- **Forbidden**: Standard-tier accessibility hit-target / WCAG
  contrast work (`QA-COND-0005` preserved). Drag-state visuals are
  friend-game visual polish only.
- **Forbidden**: Touching any file outside `client/src/ui/hand/`,
  `tests/integration/hand-ui/`, `design/gdd/hand-ui.md` (optional
  light addition only), and `production/epics/hand-ui/` for this
  story (per PROMPT 802 §8 hand-UI host-module discipline).
- **Forbidden**: Introducing a tween on the source fan-slot dim or
  the drop-target tint that would conflict with existing card lift
  / staging tweens (no overlapping tween targets on the same
  `Sprite` / `Node` component; reuse the existing
  `replace_tweenable` API from `client/src/ui/hand/mod.rs`).
- **Forbidden**: Modifying `Res<ActivePlacementDrag>` inside any
  new visual-update system.

---

## Story Classification

**Story type**: UI -- layout / composition + integration test (ECS
marker / colour / z-layer assertions).

This is **NOT** a:

- Logic story (no formula, no state-machine change beyond visual
  treatment lookup; the `Idle / Hover / Drag / DropTarget /
  Disabled` derivation is a pure read over already-extant state).
- Networking / protocol story (no Lightyear message change; no
  protocol-shape edit).
- Final-art story (PAW-* placeholder card chrome preserved).
- Accessibility story (`QA-COND-0005` preserved; this is friend-
  game visual polish only).
- Runtime-bug-repair story (Sprint 12 story 019 disposition
  preserved verbatim; no repair claimed).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type", UI stories deliver a **manual walkthrough doc OR
interaction test** with ADVISORY gate. This story raises the
evidence bar to a **required integration test in
`tests/integration/hand-ui/`** with ECS marker / colour / z-layer
assertions (see AC9 below) because the drag-state derivation is
deterministic and testable headlessly -- the visual treatment is
a pure function of `Res<ActivePlacementDrag>`, `HandUiMode`, and
the per-slot card-staging state.

---

## Dependencies (must be Done before /dev-story on this story)

| Dependency | Slug | Why blocking |
|---|---|---|
| Z-index layers | `S11-TD-UI-ZINDEX-LAYERS` (rank 1, Tier 0 Must -- Sprint 14 DONE PROMPT 902) | `HandDragSprite` reads `UI_OVERLAY` (§3) for paint order above board content during drag. Required for AC3 z-layer assertion. |
| Font constants | `S11-TD-UI-FONT-CONSTANTS` (rank 2, Tier 0 Must -- Sprint 14 DONE PROMPT 908) | Any caption added under a fan-slot drag-state treatment (e.g. cost-overdraw badge during `Disabled`) reads `typography::CAPTION` or `typography::BODY`. Reused only if the worker adds a caption; if not, this dependency is trivially satisfied. |
| Flex strip primitives | `S11-TD-UI-FLEX-STRIPS` (rank 3, Tier 0 Must -- Sprint 14 DONE PROMPT 918) | The hand-fan strip is hosted inside `HandBar` (§9, `180` px); drag-state visual overlays composed as children of the existing flex strip parent inherit its bounds. |
| Viewport invariants | `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4, Tier 0 Must -- Sprint 14 DONE PROMPT 909) | Drag-state visual treatments must not regress the viewport invariants asserted in `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS` (the 6-viewport matrix). |
| Overlay alpha tokens | `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` (rank 5, Tier 0 Must -- Sprint 14 DONE PROMPT 917) | Source fan-slot dim reads `OVERLAY_DIM_ALPHA`; fan-plate drop-target tint reads `OVERLAY_SCRIM_ALPHA`. Required for AC2 / AC4 colour assertions. |
| Global UI design spec | `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, Tier 0 Must -- Sprint 14 DONE PROMPT 922) | Spec adoption matrix row for `S12-UX-HAND-DRAG-STATE-VISUALS-001` names §3 (`UiOverlay`) + §6 overlay alpha as the source-of-truth sections. Required for AC1 token-consumption traceability. |
| HU-card-drag MVP producers | story 017 (`S11-HU-CARD-DRAG-MVP-001`, PROMPT 696) | `Pointer<Press>` / `Pointer<Move>` / `Pointer<Release>` producers + `HandDragSprite` sprite follow -- the in-flight drag state surfaced by this story is the state produced by story 017. Required for AC4 `Drag` state coverage. |

**Optional but recommended** (not blocking):

- A small `design/gdd/hand-ui.md` addition documenting the five
  drag-state visual treatments may be paired with the `/dev-story`
  implementation prompt; if not, it is a follow-on doc story.
- Coordination with `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`
  (Sprint 15 Nice to Have, Tier 0 Should-priority adjacent row;
  hover / focus / pressed / disabled primitive set under
  `client/src/ui/design_tokens/`): if the Nice row lands inside
  Sprint 15, the `Hover` state treatment in this story may reuse
  the new interaction-state primitive module (hover token); if
  the Nice row defers to Sprint 16+, this story implements the
  `Hover` cue inline using §7 `ACCENT` and `bevy_tweening`. The
  `/dev-story` worker picks whichever the integration HEAD makes
  available; both paths are acceptable.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Tier 0 token consumption**: GIVEN the post-refactor
  build, WHEN the new drag-state visual systems are inspected,
  THEN they reference the §3 (`UI_OVERLAY`), §6 (`OVERLAY_DIM_ALPHA`,
  `OVERLAY_SCRIM_ALPHA`), and §7 (`ACCENT`, `SEMANTIC_SUCCESS`)
  Tier 0 constants by symbol -- **not by numeric literal**. Verified
  by grep across `client/src/ui/hand/` for `OVERLAY_DIM_ALPHA`,
  `OVERLAY_SCRIM_ALPHA`, and `UI_OVERLAY` (or equivalent symbol
  imports) returning at least one hit each, and by no new
  inline numeric literals matching the canonical Tier 0 values
  (`0.45`, `0.55`, `UI_OVERLAY` integer = `400`) introduced in
  hand-UI source.

- [ ] **AC2 -- `Idle` baseline preserved**: GIVEN `HandUiMode == Passive`
  (or `Staging` with no active drag), WHEN a fan-slot card with no
  pointer hover and no staged placement is inspected, THEN no new
  overlay child node is present, no new tint is applied, and the
  card chrome reads identically to the pre-refactor baseline.
  Verified by ECS query: `Query<&FanSlotIndex, Without<DragStateOverlay>>`
  (or equivalent marker) returns the expected idle slots; their
  `Sprite::color` / `BackgroundColor` are unchanged from baseline.

- [ ] **AC3 -- `Drag` source-slot dim + ghost ascends to `UI_OVERLAY`**:
  GIVEN `ActivePlacementDrag::is_active()` is true AND
  `active_drag.card == Some(slot_entity)`, WHEN the fan-slot card
  and the `HandDragSprite` are inspected, THEN:
  - the source `FanSlotIndex(active_drag.slot)` entity carries a
    new child node with `BackgroundColor` reading
    `OVERLAY_DIM_ALPHA` (or equivalent named overlay-tint marker
    component);
  - the `HandDragSprite` entity carries `GlobalZIndex(UI_OVERLAY)`
    (the `400` named constant from §3 z_layers, **read by symbol**,
    not hard-coded literal);
  - the `HandDragSprite` `Visibility` is `Visible` (preserved from
    story 017 PROMPT 696).
  Verified by integration test query.

- [ ] **AC4 -- `DropTarget` (fan-plate) tint applied**: GIVEN
  `ActivePlacementDrag::is_active()` is true AND the cursor is over
  the fan-plate region (Instant-drop target per story 007), WHEN
  the fan-plate region is inspected, THEN it carries a new child
  node with `BackgroundColor` reading `OVERLAY_SCRIM_ALPHA`
  (or equivalent named overlay-tint marker), OR carries an outline
  reading `SEMANTIC_SUCCESS` (§7) -- worker decides between the
  two compositions, but the chosen composition is the one
  asserted by the integration test. **Out of scope** for this AC:
  board-cell drop-target ghosting (owned by Board Rendering, not
  Hand UI).

- [ ] **AC5 -- `Disabled` treatment applied**: GIVEN `HandUiMode == PassiveLocked`
  OR a card is already in `PendingPlacements::placements` OR the
  slot card cannot be afforded per existing submit pre-validation
  (story 010), WHEN the fan-slot card is inspected, THEN it
  carries a child node with `BackgroundColor` reading
  `OVERLAY_DIM_ALPHA` (or a desaturated `Sprite` colour-multiply,
  worker choice) AND no `Hover` outline is applied even if the
  pointer is over it. Verified by integration test query covering
  all three disable triggers.

- [ ] **AC6 -- `Hover` state (non-drag) treatment**: GIVEN
  `HandUiMode == Passive` OR `HandUiMode == Staging` AND
  `!ActivePlacementDrag::is_active()` AND the pointer is over a
  non-disabled fan-slot card, WHEN the slot is inspected, THEN it
  carries either a `BorderColor` reading `ACCENT` (§7) OR a `Transform`
  scale lifted slightly above 1.0 (e.g. 1.02-1.05; precise value
  is worker discretion within `bevy_tweening` reuse). The integration
  test asserts the chosen composition. **Out of scope** for this
  AC: pressed / focus visual states (owned by the parallel Sprint 15
  Nice candidate `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`).

- [ ] **AC7 -- State transitions are visually consistent across the
  pre-pooled entity set**: GIVEN the pre-pooled fan-slot entity
  count (10) is unchanged, WHEN the drag-state visual systems
  toggle treatments, THEN they do so via `Visibility` toggles on
  the per-slot overlay child nodes (or `BackgroundColor` swaps on
  named marker components) -- NOT via per-slot spawn / despawn
  of the parent `FanSlotIndex` entity. ADR-021 Impl Guideline 5
  preserved. Verified by ECS query: `FanSlotIndex` entity count
  remains `10` across all drag-state transitions in the integration
  test.

- [ ] **AC8 -- ADR-002 + ADR-012 binding preserved (read-only over
  ephemeral drag state; no new server-authoritative state)**: GIVEN
  the post-refactor build, WHEN the drag-state visual systems are
  inspected, THEN they read `Res<ActivePlacementDrag>`, `Res<HandUiMode>`,
  `Res<PendingPlacements>`, and per-slot `&HandSlotCard` / `&FanSlotIndex`
  / `&CardSlotPlayability` queries ONLY. They do NOT mutate
  `ActivePlacementDrag`, do NOT add any `S2C*` / `C2S*` message,
  and do NOT introduce any new `Res<...>` that surfaces server-
  authoritative drag state. Verified by grep across
  `client/src/ui/hand/` for new `ResMut<ActivePlacementDrag>` (zero
  hits) and by grep across `shared/src/protocol.rs` (no diff vs
  base) -- the integration test additionally asserts that the
  diff against `shared/src/protocol.rs` is empty.

- [ ] **AC9 -- Integration test in `tests/integration/hand-ui/` with
  ECS marker / colour / z-layer assertions**: GIVEN the post-refactor
  build, WHEN `cargo test -p client --test hand_ui_drag_state_visuals_test`
  is run (file path TBD by the `/dev-story` worker; likely
  `tests/integration/hand-ui/hand_ui_drag_state_visuals_test.rs`),
  THEN it PASSES. The test asserts each of the five drag states
  (`Idle`, `Hover`, `Drag`, `DropTarget`, `Disabled`) via ECS
  queries against marker components, `BackgroundColor`/`BorderColor`
  values, `GlobalZIndex`, and `Visibility`. No rendered-pixel
  snapshot. No `Pointer<...>` event synthesis beyond what story
  017's existing `hand_ui_drag_to_board_cell_test` already
  exercises -- the new test drives drag-state via direct resource
  insertion (`ActivePlacementDrag::start(...)`, `HandUiMode::set(...)`)
  and asserts the resulting visual treatment by ECS query.

- [ ] **AC10 -- ADR-021 plugin registration + pre-pool count
  preserved**: GIVEN the post-refactor build, WHEN `HandUiPlugin`
  is inspected, THEN it remains the third sub-plugin inside
  `PresentationPlugin` (after `CardAnimationsPlugin` and
  `BoardRenderingPlugin`); the pre-pooled entity counts are
  unchanged (10 fan slots + 9 DRAFT_INITIAL grid slots + 1 drag
  sprite + any newly-added child overlay nodes per slot which are
  spawned as **children** of the existing slot entities, not new
  top-level pre-pool entries).

- [ ] **AC11 -- Tween conflict-free**: GIVEN the post-refactor
  build, WHEN any tween installed by the drag-state visual systems
  is inspected, THEN it does NOT target a `Sprite` / `Node` component
  already targeted by an existing card lift / staging tween. The
  `replace_tweenable` API is used to install / cancel new tweens
  rather than despawn + respawn. Verified by grep for
  `cancel_tween_anim_in_place` / `replace_tweenable` usage in any
  new tween install path.

- [ ] **AC12 -- No new Lightyear / protocol message**: GIVEN the
  post-refactor build, WHEN `git diff` is inspected for paths
  matching `shared/src/protocol.rs` / `shared/src/network/` /
  `client/src/network/` / `server/src/network/`, THEN no diff is
  present. `liv-bevy-lightyear` is NOT activated for this story
  because no networking code is touched.

- [ ] **AC13 -- `Res<ActivePlacementDrag>` read-only inside new
  systems**: GIVEN the post-refactor build, WHEN any new system
  added by this story is inspected, THEN it takes
  `Res<ActivePlacementDrag>` (not `ResMut<...>`) and does NOT
  re-introduce a write path. Verified by grep across
  `client/src/ui/hand/` for new system signatures.

- [ ] **AC14 -- Sprint 12 story 019 disposition preserved**: GIVEN
  the story commit, WHEN `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`,
  `production/qa/evidence/sprint-11-drag-runtime-evidence.md`,
  `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md`,
  `production/qa/evidence/captures/sprint-11-drag-runtime/`, and
  the `production/sprint-status.yaml` Sprint 12 Must Have row
  `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001` are diffed, THEN
  none of them are modified by this story. The underlying drag-
  runtime bug is **NOT claimed fixed** by this story. `TQ-S12-C2`
  preserved (no third same-scope retest authorised).

- [ ] **AC15 -- Sprint 13/14/15 disposition preserved**: GIVEN the
  story commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  `production/sprints/sprint-15.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them are
  modified by this story. **Sprint 15 remains NOT activated by this
  authoring run.**

- [ ] **AC16 -- No accept-risk closure claimed**: GIVEN the
  implementation evidence, WHEN inspected, THEN it explicitly does
  NOT claim closure of `S8-QA-001-W1`, `QA-COND-0005`,
  `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, or any other
  accept-risk disposition. Friend-game visual polish only;
  Standard-tier accessibility is not pursued; final-art / asset
  replacement is not pursued.

- [ ] **AC17 -- Targeted regression passes**: GIVEN the post-refactor
  code, WHEN `cargo test -p client --lib` is run AND
  `cargo test -p client --test hand_ui_drag_to_board_cell_test`
  AND `cargo test -p client --test hand_ui_drag_end_non_instant_test`
  AND `cargo test -p client --test hand_ui_chrome_composition_test`
  AND `cargo test -p client --test hand_ui_slot_onscreen_test`
  AND `cargo test -p client --test hand_ui_viewport_sync_test`
  AND `cargo test -p client --test placement_staged_disclosure_accessibility_test`
  AND `cargo test -p client --test placement_timer_test` AND
  `cargo test -p client --test placement_unstaging_test` are run,
  THEN all PASS. Existing hand-UI integration tests must not regress.
  The 6-viewport invariant suite at
  `tests/integration/ui_viewport_invariants_test.rs` must also
  continue to PASS.

- [ ] **AC18 -- Hand UI EPIC count updated**: GIVEN the epic file
  `production/epics/hand-ui/EPIC.md`, WHEN updated by the
  `/story-done` paperwork at terminal disposition, THEN the
  "Stories" table reflects this story 020 row consistently with
  the existing rows-not-yet-folded note (see PROMPT 991 authoring
  EPIC update below for the current row entry; `/story-done`
  paperwork updates the row to Done and refreshes the count
  summary line).

- [ ] **AC19 -- Evidence slot reserved** (advisory; only required
  if the `/dev-story` worker opts to capture a manual visual
  walkthrough alongside the AC9 integration test):
  `production/qa/evidence/sprint-15-hand-drag-state-visuals/README.md`
  (NEW). Records the build commit, optional manual capture if any,
  no-claim restatement, and cross-links to PROMPT 802 §3.3 HA3 +
  `docs/ux/ui-clean-pass-roadmap.md` "Tier 1 Should-Priority
  Adjacent Rows" table + Sprint 15 plan §"Should Have" + this
  story file. Trivially satisfied if the worker relies on the AC9
  integration test alone -- the evidence README in that case
  records "AC9 integration test is the sole evidence; no manual
  visual capture executed."

---

## Likely Files (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

The `/dev-story` worker that implements this story will touch only
the hand-UI host module and the integration-test path. Out-of-host
edits are **forbidden** for this story per the Control Manifest
Rules.

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/mod.rs` | Add drag-state visual systems (one or more) registered in `HandUiSystemSet::StateSync`; add overlay child node spawn under each `FanSlotIndex` entity at session start (preserving pre-pool count discipline; the children are part of the slot's hierarchy, not new top-level pre-pooled entries); add marker components (e.g. `DragStateOverlay`, `FanSlotHoverState`, `FanSlotDisabledState`) for ECS-query-driven AC9 assertions; import Tier 0 token symbols (`OVERLAY_DIM_ALPHA`, `OVERLAY_SCRIM_ALPHA`, `UI_OVERLAY`, `ACCENT`, `SEMANTIC_SUCCESS`). |
| `client/src/ui/hand/` (additional submodule, if extracted) | Optional: extract drag-state visual logic into a dedicated submodule (e.g. `client/src/ui/hand/drag_state_visuals.rs`) re-exported from `mod.rs` for readability. Worker discretion. |
| `client/src/ui/design_tokens/colors.rs` (if not yet present) | This story does **not** author new color tokens; if the §7 `ACCENT` / `SEMANTIC_SUCCESS` / `SEMANTIC_ERROR` constants are not yet exposed as a named module by the Sprint 14 closure, the worker imports them from wherever the global UI design spec consumers landed them in Sprint 14. **No new numeric values introduced by this story.** |
| `tests/integration/hand-ui/hand_ui_drag_state_visuals_test.rs` | NEW. ECS-query-driven test for the five drag states. AC9 evidence. |
| `design/gdd/hand-ui.md` | Optional light addition: one row in the state-machine visual-treatment table documenting the five drag-state visual cues. Worker discretion -- may be deferred to a small follow-on doc story. |
| `production/epics/hand-ui/story-020-hand-drag-state-visuals.md` | This file. Status flipped Draft -> Ready by `/story-readiness`; Ready -> Done by `/story-done` post-implementation. |
| `production/epics/hand-ui/EPIC.md` | `/story-done` paperwork: update story-020 row in the "Stories" table from `Draft (Sprint 15 not activated)` to `Done`; refresh the count-deferral note. |

**Explicitly out of scope for the `/dev-story` worker** (any of these
would constitute a scope violation per the Forbidden rules above):

- `shared/src/protocol.rs` -- no protocol-shape edit.
- `server/` -- no server-side drag-state code.
- `client/src/presentation/board_rendering*` -- board-cell drop-target
  ghosting is owned by Board Rendering, not Hand UI; this story
  scopes the **fan-side** drop-target affordance only (AC4 scope guard).
- `client/src/ui/lobby.rs` / `client/src/ui/hud/` / `client/src/ui/shop_auction/`
  -- this story touches the hand-UI host module only (PROMPT 802 §8
  hand-UI host-module discipline).
- `production/sprints/*`, `production/sprint-status.yaml`,
  `production/stage.txt`, `production/qa/qa-plan-sprint-15.md`,
  PROMPT 761 gate-check artifact -- not touched by this story.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/` -- not touched.
- `production/epics/hand-ui/story-017-card-drag-mvp.md`,
  `story-018-drag-runtime-retest.md`,
  `story-019-drag-runtime-retest-tighter-capture.md` -- the prior
  drag-runtime stories are preserved unchanged; this story does
  NOT reopen any of them.

---

## Out of Scope

- **Sprint 12 story 019 underlying drag-runtime bug**. Sprint 12
  story 019 is `closed-with-conditions / cannot-reproduce` per
  PROMPT 814; no third same-scope retest is authorised per
  `TQ-S12-C2`. This story is layout / visual state work over the
  already-extant client-side drag ephemeral state. It does NOT
  reproduce, repair, or retest the Sprint 12 story 019 runtime
  divergence.
- **Board-cell drop-target ghosting** during PLACEMENT drag. The
  board-cell ghost preview is owned by Board Rendering
  (`client/src/presentation/board_rendering.rs`). This story
  scopes the fan-side drop-target affordance only (the fan-plate
  region used for Instant card drops per story 007).
- **Keyboard equivalent of drag-state visuals** (focus / Enter /
  arrow keys). Owned by the accessibility stories 014 / 015 and
  by Standard-tier accessibility scope (`QA-COND-0005`
  accepted-risk).
- **Pressed / focused button visual states** for non-card UI
  elements (lobby buttons, HUD buttons, auction bid buttons).
  Owned by the parallel Sprint 15 Nice candidate
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (Tier 0
  Should-priority adjacent row).
- **WCAG contrast verification** on the drag-state visual colour
  treatments. Standard-tier accessibility is `QA-COND-0005`
  accepted-risk. Friend-game visual polish only.
- **Final-art replacement** on the card chrome under any drag
  state. `PAW-TD-*-a` preserved across PAW-002..PAW-006.
  Placeholder card art retained.
- **`S8-QA-001-W1` closure** (two-client GAME_OVER manual gap).
  Out of surface for this story.
- **New Lightyear protocol message or new server-authoritative
  drag state**. ADR-002 + ADR-012 binding preserved.
- **GDD authoring beyond a light state-machine table addition**.
  A full hand-UI state-machine rewrite is out of scope; if the
  state-machine table addition exceeds one paragraph or one
  table row, the worker defers the GDD edit to a small follow-on
  doc story.
- **Activating Sprint 15**. PROMPT 991 (this authoring) does NOT
  activate Sprint 15. Activation is a separate prompt mirroring
  the PROMPT 826 / PROMPT 897 pattern.
- **`/qa-plan sprint-15` authoring**. Owned by a separate prompt
  after Sprint 15 activation per the Sprint 15 plan §"Entry
  Conditions".
- **`/story-readiness` on this story**. Run as a separate prompt
  after Sprint 15 activation per the Sprint 15 plan §"Suggested
  First Parallel Batch" pre-activation prerequisites list.
- **`/dev-story` on this story**. Run only after Sprint 15
  activation, after `/qa-plan sprint-15` lands, and after
  `/story-readiness` passes against Sprint 15 activation HEAD.
- **Polish->Release gate-check retry**. PROMPT 761 FAIL preserved
  at `production/gate-checks/gate-polish-release-2026-05-12.md`;
  no retry in scope.
- **Stage advance from Polish to Release**. `production/stage.txt`
  reads `Polish` and is NOT modified by this story (or by any
  future `/dev-story` or `/story-done` paperwork on this story).

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker conflates this story with the Sprint 12 story 019 drag-runtime bug and attempts a repair / retest. | Low | High | Forbidden rules and AC14 explicitly preserve Sprint 12 story 019 disposition; `TQ-S12-C2` cited; the no-claim banner restates "no runtime-bug repair claimed". |
| Worker introduces new server-authoritative drag state (e.g. `S2CDragState`) to "make the visual differentiation server-driven". | Low | High | ADR-002 + ADR-012 binding cited; AC8 + AC12 explicitly verify zero diff against `shared/src/protocol.rs`. |
| Worker adds new pre-pooled top-level entities for the drag-state overlays, regressing ADR-021 Impl Guideline 5 pre-pool discipline. | Low | Medium | AC7 + AC10 explicitly assert pre-pool count preservation; overlays must be **children** of existing slot entities, not new top-level pre-pool entries. |
| Worker introduces tween conflicts with existing card lift / staging tweens. | Medium | Medium | AC11 + Control Manifest Rule on `replace_tweenable` reuse cited; EPIC.md §"Key ADR-021 Constraints" rule 6 (cancel-and-replace, no despawn + respawn mid-animation) restated. |
| Worker pulls in `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (Sprint 15 Nice) before that story has landed, creating a cross-row coupling. | Medium | Low | Dependencies §"Optional but recommended" makes the coupling explicit; both paths (use Nice row's primitive module IF available, OR implement inline) are acceptable; the worker resolves at integration HEAD. |
| Worker tries to ratify a new numeric value (e.g. a new alpha or a new colour) instead of consuming Tier 0 tokens. | Low | Medium | AC1 verifies symbol-by-symbol consumption; no new numeric literals matching Tier 0 values introduced; new tokens require a separate story authoring run. |
| Worker touches a forbidden path (`client/src/ui/lobby.rs`, `client/src/ui/hud/`, `client/src/presentation/board_rendering*`, `shared/`, `server/`, `production/sprint-status.yaml`, etc.). | Low | Medium | Forbidden list enumerated; `/dev-story` worker can `git diff --stat` against base to verify scope before commit. |
| Worker authors a manual screenshot evidence path and skips the AC9 integration test. | Medium | Medium | AC9 is BLOCKING (integration test required); AC19 manual evidence is ADVISORY only. The story type asserts UI + integration-test pairing. |
| Worker is tempted to fix the Sprint 12 story 019 underlying runtime bug because "the visuals exposed a regression". | Low | Medium | If the AC9 integration test surfaces a runtime regression that is **not** the drag-state visual treatment itself, it is recorded in a separate follow-on story under `production/epics/hand-ui/` or `production/epics/playable-client/` -- NOT repaired inside this story. ADR-002 + ADR-012 binding preserved across the boundary. |
| `/story-readiness` against Sprint 15 activation HEAD fails because Sprint 15 activation introduces a base SHA change that invalidates Tier 0 token symbol locations. | Low | Low | Tier 0 token modules under `client/src/ui/design_tokens/` are stable since Sprint 14 PROMPT 902/906/908/917/918; Sprint 15 activation is paperwork-only and does not touch them. |
| Concurrent root-checkout race during authoring damages this story file. | Low | Low | PROMPT 991 authoring is a single new write on a dedicated worker branch (`story-authoring/sprint-15-hand-drag-state-visuals`) in an isolated worktree; root-checkout never touches the worker branch. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for the PROMPT 991 authoring run itself
(which is paperwork-only):

- `production/sprint-status.yaml` top-level `sprint:` field reads
  `15` (after Sprint 15 activation) and the row for
  `S12-UX-HAND-DRAG-STATE-VISUALS-001` is `ready` at the time
  `/dev-story` is dispatched.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-15.md` shows the ACTIVATED banner
  (added by Sprint 15 activation; not added by PROMPT 991).
- PROMPT 761 Polish->Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is
  preserved.
- `production/qa/qa-plan-sprint-15.md` exists and references this
  story (authored by a separate `/qa-plan sprint-15` prompt after
  Sprint 15 activation).
- `/story-readiness` on this story file returns READY against the
  Sprint 15 activation HEAD.
- Sprint 12 story 019 disposition preserved on `origin/main`
  (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
  status `Done` with `closed-with-conditions / cannot-reproduce`).
- `git diff --check` and `git diff --cached --check` pass before
  commit.

---

## Authoring Trail

- 2026-05-16 -- PROMPT 991 -- Story file authored as Sprint 15
  Should Have candidate `S12-UX-HAND-DRAG-STATE-VISUALS-001`.
  Worktree `D:\_DEV\claude-code-game-studios-worktrees\sprint-15-hand-drag-state-visuals-story-991`,
  branch `story-authoring/sprint-15-hand-drag-state-visuals`,
  base `origin/main@2c84d6e37f2ec58b729064b6dbe4c9b017e5ceb3`
  (PROMPT 990 Sprint 15 plan draft integration). Files touched by
  this authoring run: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (story-list row added; count
  deferral note updated). Sprint 15 NOT activated. No code change.
  No `/dev-story`, `/story-readiness`, `/story-done`,
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`,
  `/qa-plan`, `cargo`, or `trunk` command run. ADR-002 + ADR-012
  binding preserved; Sprint 12 story 019 disposition preserved;
  PROMPT 761 gate-check FAIL preserved; `QA-COND-0005`,
  `QA-COND-0006`, `PAW-TD-*-a`, `S8-QA-001-W1`, `TQ-S12-C1..C7`
  all preserved verbatim.
