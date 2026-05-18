# Story 022: S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001 — Mana Preview During Placement Drag

> **Epic**: Hand UI
> **Story ID**: `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001`
> **Status**: Draft — future Sprint 18 candidate; NOT activated
> **Layer**: Presentation — Hand UI (drag-state read) + HUD (reactive mana label preview)
> **Type**: Logic + Integration test (reactive UI mutation under simulated drag)
> **Authored**: 2026-05-18 by PROMPT 1136
> **Authoring worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s18-hand-mana-affordance-stories`
> **Authoring branch**: `work/s18-hand-mana-affordance-stories`
> **Authoring source-of-truth**: `origin/main@05192b5f830c5d5b17ed7af07df37f56187130fc`
> (PROMPT 1125 `story-done(s17): close S17-OPS-VULKAN-VALIDATION-GATING-001`)

---

## Status / No-Claim Banner

This story is authored by PROMPT 1136 as a **future Sprint 18** candidate
covering the **missing-feature** classification from
`reports/PROMPT-1127-placement-drag-drop-mana-preview-diagnostic.md` §R2
(*"Mana preview during drag"*). Sprint 18 does **NOT** exist as a plan
file on `origin/main` at authoring time (`production/sprints/sprint-18.md`
is absent). This story is candidate planning only.

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
- Modify any `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/`
  file.
- Re-author or edit any **Complete** Sprint 15 / 17 hand-ui story
  (Stories 005 / 006 / 007 / 008 / 010 / 011 / 020 / 021).

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- closure of `S8-QA-001-W1`
- closure of the **R1 drag-pipeline-dead-in-shipped-build bug**
  (PROMPT 1127 §R1; `ui_picking` feature gate; build-config regression).
  R1 is a **separate repair** and a **separate story / prompt**; this
  story is the **missing-feature** authoring for R2 only.
- closure of the **R3 idle-hand playable-affordance missing-feature**
  (PROMPT 1127 §R3). R3 is authored as a sibling future-Sprint-18
  candidate at `story-023-hand-idle-playable-affordance.md`.
- closure of `AUDIT-1076-02 / AUDIT-1076-03` server-side placement loss.
- final-art / asset-production completion (`PAW-TD-*-a`).
- advance of stage from `Polish` to `Release`.

ADR-002 + ADR-012 + ADR-021 binding preserved: this story is **read-only
over client-side mirrors of authoritative state** (`Res<PlayerEconomyView>`,
`Res<ActivePlacementDrag>`, `Res<HandCardCatalog>`, `Res<PendingPlacements>`).
It introduces no new server-authoritative state, no new Lightyear
message, no new protocol shape, and no client-side authority over
stage / activate / submit. Server-side mana deduction remains
authoritative under `BLS-011` and the existing `economy_api::apply_explicit_mana_split`
commit path; the preview is purely a HUD display projection that
resets to authoritative values on drag end / cancel.

---

## Source Finding

**PROMPT 1127 diagnostic** (`reports/PROMPT-1127-placement-drag-drop-mana-preview-diagnostic.md`
§2 *"R2 — Mana preview during drag"*):

> **Status: MISSING-FEATURE**. There is no code path anywhere in
> `client/`, `server/`, or `shared/` that projects "if I drop this
> card here, my current/reserve mana would become X".

Specifically, the diagnostic established (PROMPT 1127 §2.1):

- `client/src/presentation/shared/economy_view.rs` (`PlayerEconomyView`):
  `current_mana` / `reserve_mana` / `mana_cap` / `gold` are updated only
  by `S2CGoldUpdate` and `S2CGameSnapshot` — no reactive coupling to
  `ActivePlacementDrag`.
- `client/src/ui/hud/mod.rs::sync_mana_text_system`: mana bar renders
  the static authoritative resource each frame; no query against drag
  state.
- `client/src/ui/hand/mod.rs`: zero hits for `preview`, `would_cost`,
  `projected`.
- `client/src/ui/hand/drag_state_visuals.rs::slot_is_affordable`
  (lines 352-368): returns a binary playable / not-playable boolean
  for Minions; never projects a numeric delta.
- `shared/src/protocol.rs`: no `C2SPreviewPlacement` / `S2CPreviewMana`
  exchange exists or is required (the projection is purely client-side).

**Design coverage gap** (PROMPT 1127 §2.2):

- Story 006 (placement-drag-highlights) covers **board cell / unit /
  objective** highlights during drag; zero ACs for mana feedback.
- Story 010 (submit-prevalidation) validates **at Submit click only**
  per `TR-HU-008`; not during drag.
- Story 011 (reserve-mana-strip) operates on **already-staged** cards,
  not on a card mid-drag.
- `design/gdd/hand-ui.md` Rules 1-14 do not mention mana preview.

**Classification**: MISSING-FEATURE, not a regression. The user's
expectation is correct for a Hearthstone-genre game but is outside
the entire current story tree. This story is the **formal authoring
pass** before any future `/dev-story` implementation prompt can be
emitted for the feature.

---

## Problem Class / Prevention Target

**Defect class**: During PLACEMENT drag of a Minion card, the player has
no on-screen indication of what their `current_mana` and `reserve_mana`
will look like after dropping the card. The HUD mana labels (PROMPT
PRES-002 / Story 010 dependency surface) display only the authoritative
`PlayerEconomyView` values, which remain unchanged until the server
processes the eventual `C2SSubmitPlacement` batch. As a result:

1. Players cannot judge affordability of a sequence of drags without
   mentally subtracting cost from current mana every step. With
   `mana_cap` rising over rounds and `reserve_mana` being a distinct
   pool, this cognitive load is the highest in the drag-to-stage
   flow.
2. The reserve-mana-strip (Story 011) only appears **after** a card
   is staged, leaving the drag window itself silent about the split.
3. The drop-target highlight (Story 006) communicates *where* a card
   can go but not *whether the player has the resources to commit it*.
4. The disabled-overlay (Story 020 `Disabled` state) signals
   unaffordability with a binary dim tint, but does not surface
   numeric deltas during an attempted drag of a different
   (affordable) card.

**Prevention target**: introduce a HUD-reactive mana preview that
renders projected `current_mana` and `reserve_mana` values during
an active PLACEMENT drag of a Minion card, sourced purely from
already-extant client state. The preview activates on
`ActivePlacementDrag::is_active()` transitioning to true (drag start)
and resets to authoritative `PlayerEconomyView` values on drag end,
drag cancel, or drop completion. The preview uses a **default
reserve-amount = 0 spend** (matching Story 011 `spawn_reserve_strip`
defaults: `reserve_amount = 0` and `current_mana_spend = cost`); any
subsequent reserve-strip adjustments to staged cards are handled by
the existing Story 011 surface, not by this drag-time preview.

---

## Context

### Existing surface

- **`client/src/presentation/shared/economy_view.rs`**
  - `pub struct PlayerEconomyView { gold, current_mana, reserve_mana, mana_cap, initialized, last_update_source }`
    (Resource, derived from `S2CGoldUpdate` and `S2CGameSnapshot`).
  - `pub fn apply_player_economy_view(...)` mirrors this resource into
    HUD `GoldDisplayState` / `ManaDisplayState` once per change.
- **`client/src/ui/hud/mod.rs`**
  - `ManaDisplayState { current_mana, mana_cap, reserve_mana, is_populated }`
    is the HUD-side mirror; `sync_mana_text_system` paints `Text`
    nodes for the current/reserve mana labels.
  - `apply_player_economy_view` (line 1829) copies `PlayerEconomyView`
    into `ManaDisplayState`; `mana_display_differs_from_view`
    (line 1842) is the change-detection comparator.
- **`client/src/ui/hand/mod.rs`**
  - `pub struct ActivePlacementDrag { card, card_id, owner_id, target_kind, cursor_world_position }`
    (Resource) with `is_active()` (true iff `card.is_some() && target_kind.is_some()`).
  - `HandCardCatalog` resource (re-exported and consumed by
    `client/src/ui/hand/drag_state_visuals.rs`) maps `CardId →
    CardDef` (`shared::card::CardDef { card_type, cost, ... }`).
  - `PendingPlacements { placements: Vec<PlacedCardSubmit> }` —
    already-staged cards' aggregated `current_mana_spend` /
    `reserve_mana_spend` are read from this resource for
    multi-card preview (the projection must subtract already-staged
    spend on top of the in-flight card's cost).
- **`client/src/ui/hand/drag_state_visuals.rs::slot_is_affordable`**
  (lines 352-368): `available = current_mana + reserve_mana;
  available >= card.cost`. Only `CardType::Minion` is checked;
  Instants / passives return `true`. This story REUSES the same
  affordability test but extends it into a numeric projection.
- **`shared/src/card.rs::CardType`**
  - Variants: `Minion`, `Spell`, `Trap`, `Structure`, `Field`, `Order`.
  - Per GDD Rule 6 mana split semantics, only `Minion` placements
    consume `current_mana_spend + reserve_mana_spend == card.cost`
    via `economy_api::apply_explicit_mana_split` on the server. Other
    card types are out of scope for this preview (see AC8).
- **`shared/src/protocol.rs::PlacedCardSubmit`** (line 289):
  `{ card_id, target, current_mana_spend, reserve_mana_spend }`
  with `current_mana_spend + reserve_mana_spend == card.cost` per
  GDD Rule 10. The default at stage time is
  `current_mana_spend = card.cost, reserve_mana_spend = 0` (Story 011
  `spawn_reserve_strip` default). The drag-time preview MUST use the
  same default so the projection and the staged value coincide.
- **`design/gdd/hand-ui.md` Rule 6** (PLACEMENT drag-to-stage flow):
  no mana-preview text. **Light GDD addition** is in scope for the
  paired `/dev-story` — one row in the drag-to-stage flow table
  documenting the HUD mana preview. The numeric tokens
  (overdraw colour, baseline) reuse the existing §7 `SEMANTIC_ERROR`
  / `SEMANTIC_SUCCESS` palette already in use by `drag_state_visuals.rs`.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` Rule 6 (PLACEMENT drag-to-stage)
  and Rule 10 (Submit with pre-validation). Light addition: one row
  in Rule 6 documenting that the HUD mana labels project the
  in-flight card's cost during an active drag and reset on drag end.
  Owner: `/dev-story` paired with the implementation prompt may
  perform the edit, OR it may be deferred to a small follow-on doc
  story. Either resolution is acceptable per Sprint 15 / 17 precedent
  (Story 020 paired the GDD addition; Story 014 deferred it).
- **`design/gdd/economy-system.md`**: server-authoritative mana
  deduction unchanged; preview is purely client-side cosmetic.
- **`design/gdd/hud.md`**: mana labels surface owns the projection;
  reserve diamond reads projected `reserve_mana` during drag if and
  only if the in-flight card's projected `reserve_mana_spend > 0`
  (default 0 → reserve diamond unchanged at drag time; preview only
  affects `current_mana` text by default — see AC2 and AC3).
- **ADR-021** (Presentation Layer Architecture): preserved. Mana
  preview is a HUD reactive read of `Res<PlayerEconomyView>` +
  `Res<ActivePlacementDrag>` + `Res<HandCardCatalog>` +
  `Res<PendingPlacements>`. No new pre-pooled entity; the existing
  HUD mana labels (`ManaLabel`, `ReserveManaLabel`) are reused.
- **ADR-002** (Client-Server Authority): preserved. Preview never
  mutates `PlayerEconomyView`. The authoritative pool is restored
  via `apply_player_economy_view` on every `S2CGoldUpdate` /
  `S2CGameSnapshot` and (additionally) on every drag-end / drag-cancel
  transition (see AC4).
- **ADR-009** (RSM Phase State): preserved. Preview gates on
  `Res<CurrentClientPhase> == Phase::Placement` AND
  `Res<ActivePlacementDrag>::is_active()`; no `MessageReceiver<S2CPhaseChanged>`
  drain (EPIC.md §"Key ADR-021 Constraints" rule 3).
- **ADR-012** (SessionReady Delivery): preserved. No new server-
  authoritative state surface, no resource that must be present
  before a trigger fires, no SessionReady path change. Preview reads
  client mirrors only.
- **ADR-019** (Mana split): the projection's default split
  (`current_mana_spend = cost, reserve_mana_spend = 0`) matches
  `economy_api::apply_explicit_mana_split` first-current-then-reserve
  fallback for a default `reserve_amount = 0` strip value (Story 011
  HU-25 / HU-26 / HU-27 default). When `cost > current_mana`, the
  server's authoritative behaviour spills overflow into reserve
  (validate_explicit_mana_split path); the preview MUST mirror this
  spill for visual consistency (see AC5 overdraw treatment).
- **TR registry**: extends `TR-HU-002` (PLACEMENT drag-to-stage state
  machine) and `TR-HU-008` (submit pre-validation) with a **new TR**:
  `TR-HU-009` — *"Mana preview during PLACEMENT drag"*. The TR
  registry edit (`docs/architecture/tr-registry.yaml`) is performed
  by the `/dev-story` implementation prompt, not by this authoring
  run.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits under
  `client/src/ui/hand/`, `client/src/ui/hud/`, or
  `client/src/presentation/shared/`, and for the integration test.
  The `/dev-story` implementation prompt MUST activate this skill
  before editing.
- **Lightyear**: not applicable. This story does not touch Lightyear
  protocol, `S2C*` messages, `C2S*` messages, or any networking
  surface. `liv-bevy-lightyear` is **NOT** required and **NOT**
  activated.

### Control Manifest Rules

- **Required**: The preview state is held in a new client-side
  `Res<ManaPreviewState>` (or equivalent component on the HUD
  `ManaLabel` entity) that is read-only over `PlayerEconomyView`,
  `ActivePlacementDrag`, `HandCardCatalog`, `PendingPlacements`. No
  mutation of `PlayerEconomyView` is permitted under any condition.
- **Required**: The preview is computed in `PresentationSet::StateSync`
  on the same frame as `sync_mana_text_system` so the projection and
  the authoritative reset are deterministically ordered.
- **Required**: On every `ActivePlacementDrag::is_active()` transition
  to false (drag end / cancel / drop), the HUD `ManaDisplayState`
  MUST be re-synced from `PlayerEconomyView` via
  `apply_player_economy_view` (or equivalent path) **before** the
  next `sync_mana_text_system` runs. No stale projection may survive
  one frame past drag end.
- **Required**: The projection uses the **default split** at drag
  time: `current_mana_spend = min(cost, current_mana)`,
  `reserve_mana_spend = cost - current_mana_spend` (spills into
  reserve if and only if `cost > current_mana`). This mirrors
  `economy_api::apply_explicit_mana_split` server fallback and the
  Story 011 `reserve_amount = 0` default.
- **Required**: For non-Minion card types (Spell, Trap, Structure,
  Field, Order, Instant), the preview is **explicitly suppressed**:
  the HUD remains at authoritative `PlayerEconomyView` values during
  the drag. Rationale: those card types do not consume mana under
  the current `economy_api::apply_explicit_mana_split` flow (per
  `slot_is_affordable` line 364 fast-path), so projecting a delta
  would be misleading. See AC8.
- **Required**: Aggregation across already-staged cards. The
  projection MUST subtract the sum of `placements[i].current_mana_spend`
  and `placements[i].reserve_mana_spend` over `PendingPlacements`
  BEFORE adding the in-flight card's projected spend. This keeps
  the preview consistent with Story 010 / `TR-HU-008` pre-validation
  arithmetic.
- **Required**: `liv-bevy-018` skill applies to all `.rs` edits.
- **Required**: ADR-021 plugin registration order preserved
  (`HandUiPlugin` remains #3, `HudPlugin` retains its registration
  order; the preview's `Res<ManaPreviewState>` insert order does
  NOT alter sub-plugin order).
- **Required**: Integration test asserts that the HUD `ManaLabel`
  `Text::0` value mutates between drag start and drag end under a
  simulated `ActivePlacementDrag::start(...)` direct resource write.
  No `Pointer<*>` event synthesis is required (avoids dependence
  on the R1 picking-pipeline repair).
- **Forbidden**: Mutating `PlayerEconomyView` from any drag-preview
  system (verified by grep for `ResMut<PlayerEconomyView>` in new
  systems).
- **Forbidden**: Introducing a new Lightyear message, a new server-
  authoritative state surface, or any C2S preview round-trip.
- **Forbidden**: Reproducing the R1 drag-pipeline-dead bug or
  blocking on its repair. The integration test for this story drives
  drag state via direct resource insertion (see AC9), independent of
  the `bevy_picking` feature gate.
- **Forbidden**: Touching any file outside the host paths listed in
  "Likely Files" (hand-UI, HUD, and tests/integration/hand-ui/ or
  tests/integration/hud/). No edits to `shared/`, `server/`, or any
  protocol-shape file.
- **Forbidden**: Repairing R3 idle-hand affordance inside this story
  (R3 is `story-023-hand-idle-playable-affordance.md`).
- **Forbidden**: Re-authoring **Complete** Sprint 15 / 17 hand-ui
  stories 005 / 006 / 010 / 011 / 020. AC2 / AC3 / AC8 are
  **additive** to those stories and do not regress them; no edits to
  their files are required by this story or its eventual
  `/dev-story` implementation.

---

## Story Classification

**Story type**: Logic + Integration test (reactive UI mutation).

This is **NOT** a:

- Networking / protocol story (no `S2C*` / `C2S*` edit; no
  `shared/src/protocol.rs` diff).
- Visual / feel story (the projection is a deterministic, testable
  arithmetic value; the HUD `Text` node is asserted by integration
  test rather than by manual screenshot).
- Runtime-bug-repair story (R1 drag pipeline + R1.b cursor coord-space
  + AUDIT-1076-02/03 server-side placement loss are **separate**
  prompts).
- Accessibility story (`QA-COND-0005` preserved).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type",
Logic stories deliver an automated unit test (BLOCKING) and
Integration stories deliver an integration test (BLOCKING). This
story carries both forms:

1. **Unit-test slice (BLOCKING)**: pure projection arithmetic
   (`project_mana_preview(view, drag_card, pending) → (current_proj,
   reserve_proj, overdrawn)`) tested in isolation under
   `tests/unit/hand-ui/mana_preview_projection_test.rs`.
2. **Integration-test slice (BLOCKING)**: full HUD `ManaLabel`
   `Text::0` mutation under simulated drag in
   `tests/integration/hand-ui/mana_preview_during_drag_test.rs`.

Both BLOCKING gates per the story-type matrix.

---

## Dependencies (must be Done before /dev-story on this story)

| Dependency | Slug / Story | Why blocking |
|---|---|---|
| `PlayerEconomyView` resource | `PRES-002` (Complete at `8b10c6e`) | The projection's authoritative source-of-truth pool reads `current_mana` / `reserve_mana` / `mana_cap` from this resource. |
| HUD mana labels reactive | `HUD-MANA-LABELS` (existing on `origin/main`; `sync_mana_text_system` at `client/src/ui/hud/mod.rs:1882`) | The projection writes through `ManaDisplayState` which `sync_mana_text_system` paints to `Text::0`. |
| `ActivePlacementDrag` resource | Story 017 (`S11-HU-CARD-DRAG-MVP-001`) | Source of the drag-start / drag-end transitions; `is_active()` and `card_id` are read by the projection system. |
| `HandCardCatalog` resource + `slot_is_affordable` | Story 020 (`S12-UX-HAND-DRAG-STATE-VISUALS-001`, Complete) | The projection's `cost` lookup reuses the same `HandCardCatalog::cards.get(&card_id).cost` path; the overdraw branch reuses `slot_is_affordable` semantics for Minion-only gating. |
| `PendingPlacements` resource + `current_mana_spend` / `reserve_mana_spend` fields | Story 005 + Story 011 (both Complete) | The projection subtracts already-staged aggregated spend. |
| Server-authoritative `apply_explicit_mana_split` | `ECO-007` (Complete at `a564d99`) | Defines the canonical default split (`current` first, then `reserve` spillover); the preview must mirror this fallback so the staged-card display matches the dropped result. |
| `CurrentClientPhase` resource | ADR-009 + Story 003 (Complete) | The projection gates on `Phase::Placement`; otherwise the preview is suppressed. |
| **R1 repair** (PROMPT 1127 §1) | **NOT required at the integration-test level** — the test drives drag via direct resource insertion. | R1 is required only for end-to-end manual evidence; the BLOCKING integration test does NOT depend on `bevy_picking` runtime. See AC9. |

**Optional but recommended** (not blocking):

- A light `design/gdd/hand-ui.md` Rule 6 addition. May be paired with
  the `/dev-story` implementation OR deferred to a follow-on doc
  story. Pattern matches Story 020 vs Story 014 (Story 020 paired;
  Story 014 deferred).
- Coordination with `story-023-hand-idle-playable-affordance.md`
  (R3 sibling): the idle-affordance story introduces a separate
  affordance-overlay marker, deliberately distinct from this story's
  drag-time HUD preview. The two are independent and may ship in
  parallel or in either order. Both stories cite this independence
  in their "Out of Scope" sections.

---

## Acceptance Criteria

All criteria are independently checkable. The integration test in AC9
is the single BLOCKING evidence path; the unit test in AC10 is the
companion BLOCKING evidence for the pure projection arithmetic.

- [ ] **AC1 — Preview activates on drag start (Minion)**: GIVEN
  `Res<CurrentClientPhase> == Phase::Placement` AND a Minion card
  with `card.cost = 4` is in the local hand AND
  `PlayerEconomyView { current_mana: 5, reserve_mana: 2, ... }`,
  WHEN `ActivePlacementDrag::start(card_id, ...)` transitions
  `is_active()` to true, THEN within the same frame
  `ManaDisplayState.current_mana == 1` AND
  `ManaDisplayState.reserve_mana == 2` (projection:
  `current_mana_spend = min(4, 5) = 4`, `reserve_mana_spend = 0`;
  authoritative current `5 - 4 = 1`; reserve unchanged at 2).
  Verified by querying `ManaDisplayState` after one `App::update()`
  tick following the resource insertion.

- [ ] **AC2 — Preview reflects projected current mana in HUD Text**:
  GIVEN AC1 preconditions, WHEN one `App::update()` tick runs after
  AC1, THEN the HUD `ManaLabel` `Text::0` value contains the substring
  `"1"` (the projected `current_mana`) AND the rendered string
  matches the existing `sync_mana_text_system` format
  (e.g. `"1 / 5"` if the format is `current / cap`, with `cap`
  unchanged at the authoritative `5`). The exact format is the one
  produced by `sync_mana_text_system` for the projected value — the
  story does NOT alter the format; it alters only the projected
  numeric input. Verified by reading `Text::0` from the `ManaLabel`
  entity.

- [ ] **AC3 — Reserve spillover when `cost > current_mana`**: GIVEN
  `PlayerEconomyView { current_mana: 1, reserve_mana: 3, ... }`
  AND a Minion card with `card.cost = 3` is drag-source, WHEN one
  tick runs, THEN `ManaDisplayState.current_mana == 0` AND
  `ManaDisplayState.reserve_mana == 1` (projection:
  `current_mana_spend = min(3, 1) = 1`, `reserve_mana_spend = 3 - 1 = 2`;
  authoritative reserve `3 - 2 = 1`). The HUD `ReserveManaLabel`
  `Text::0` updates accordingly. The reserve diamond visibility
  follows the existing `set_reserve_mana_visibility` rules (Hidden
  iff projected reserve == 0); when projected reserve > 0 (as here),
  it remains `Visible`.

- [ ] **AC4 — Preview resets on drag end / cancel / drop**: GIVEN
  a preview was active per AC1 with `ManaDisplayState.current_mana == 1`,
  WHEN `ActivePlacementDrag::clear()` is invoked (drag end, drag
  cancel, or successful drop), THEN within the same frame and
  before the next `sync_mana_text_system` paint:
  - `ManaDisplayState.current_mana == 5` (authoritative value
    restored from `PlayerEconomyView`).
  - `ManaDisplayState.reserve_mana == 2` (authoritative restored).
  - The HUD `ManaLabel` `Text::0` paints the authoritative value
    on the next `sync_mana_text_system` invocation.
  - `Res<ManaPreviewState>` (or equivalent marker) is cleared /
    inactive. No stale projection survives one frame past
    `ActivePlacementDrag::is_active() == false`.

- [ ] **AC5 — Overdraw indicator when cost exceeds combined pool**:
  GIVEN `PlayerEconomyView { current_mana: 1, reserve_mana: 1, ... }`
  AND a Minion card with `card.cost = 5` is drag-source, WHEN one
  tick runs, THEN:
  - `ManaDisplayState.current_mana == 0` (clamped at 0; `1 - 1 = 0`).
  - `ManaDisplayState.reserve_mana == 0` (clamped at 0;
    `1 - (5 - 1) = -3`, saturating to `0`).
  - A new marker `ManaPreviewOverdrawn` (component on the
    `ManaLabel` entity, or a field on `Res<ManaPreviewState>`) is
    present / true so the HUD can apply the `SEMANTIC_ERROR` (§7)
    colour treatment to the preview text. The exact paint
    treatment (text colour, background tint, optional
    `OVERLAY_DIM_ALPHA` flash) is **worker discretion**; the
    BLOCKING assertion is the marker / state-field presence, not
    a specific colour value. See AC11 for advisory visual.

- [ ] **AC6 — Multi-card preview subtracts already-staged spend**:
  GIVEN `PlayerEconomyView { current_mana: 6, reserve_mana: 0, ... }`
  AND `PendingPlacements::placements` contains one
  `PlacedCardSubmit { current_mana_spend: 3, reserve_mana_spend: 0, ... }`
  AND a Minion card with `card.cost = 2` is drag-source, WHEN one
  tick runs, THEN `ManaDisplayState.current_mana == 1` (projection:
  `available_after_staged = 6 - 3 = 3`; in-flight spend = 2; remaining
  current = `3 - 2 = 1`). The reserve-strip-mutated `reserve_mana_spend`
  values from Story 011 are similarly aggregated for the reserve
  pool: GIVEN `PlayerEconomyView { current_mana: 0, reserve_mana: 5, ... }`
  AND `PendingPlacements` contains one
  `PlacedCardSubmit { current_mana_spend: 0, reserve_mana_spend: 2 }`
  AND drag of a Minion with `cost = 2`, THEN projected
  `reserve_mana == 1` (5 − 2 − 2 = 1) and projected `current_mana == 0`.

- [ ] **AC7 — Preview suppressed outside `Phase::Placement`**: GIVEN
  `Res<CurrentClientPhase> != Phase::Placement` (e.g.
  `Phase::DraftShop`) AND any `ActivePlacementDrag::is_active() == true`
  scenario constructed for tests, WHEN one tick runs, THEN the HUD
  `ManaDisplayState` paints authoritative `PlayerEconomyView` values
  unchanged. No projection is applied. Rationale: drag is suppressed
  in non-PLACEMENT phases per GDD Rule 5d ("DRAFT_SHOP drag-start
  suppression"); the preview must not contradict that by partially
  surfacing a projection.

- [ ] **AC8 — Non-Minion card types: preview explicitly suppressed**:
  GIVEN a drag-source card with `CardType::Spell` (or `Trap`,
  `Structure`, `Field`, `Order`, or `Instant`) AND any
  `PlayerEconomyView` / cost combination, WHEN one tick runs with
  `ActivePlacementDrag::is_active() == true`, THEN the HUD
  `ManaDisplayState` paints authoritative `PlayerEconomyView` values
  unchanged. No projection is applied for non-Minion card types.
  The `ManaPreviewOverdrawn` marker is absent. Rationale: only
  `CardType::Minion` consumes mana under `economy_api::apply_explicit_mana_split`
  in the current card pool; projecting a delta for other card types
  would be misleading. This mirrors the existing
  `slot_is_affordable` line 364 fast-path (`card.card_type !=
  CardType::Minion → true`). **If a future card pool introduces
  mana-costing non-Minion types, a separate follow-on story extends
  this AC; the current story scope is Minion-only.**

- [ ] **AC9 — Integration test in `tests/integration/hand-ui/`**:
  GIVEN the post-implementation build, WHEN
  `cargo test -p client --test mana_preview_during_drag_test` is
  run (file path TBD by the `/dev-story` worker; likely
  `tests/integration/hand-ui/mana_preview_during_drag_test.rs`),
  THEN it PASSES with at minimum 8 assertions covering AC1, AC2,
  AC3, AC4, AC5, AC6 (two sub-cases), AC7, and AC8. The test
  drives drag state via direct resource insertion
  (`ActivePlacementDrag::start(...)` or equivalent setter) and
  asserts `ManaDisplayState` field values and `ManaLabel` /
  `ReserveManaLabel` `Text::0` strings via ECS query. No
  `Pointer<*>` event synthesis. No `DefaultPickingPlugins`
  installation required (independent of R1 repair).

- [ ] **AC10 — Unit test for projection arithmetic in
  `tests/unit/hand-ui/mana_preview_projection_test.rs`**: GIVEN
  the post-implementation build, WHEN
  `cargo test -p client --test mana_preview_projection_test` is
  run, THEN it PASSES with at minimum 6 assertions covering the
  pure projection function:
  - `project((curr=5, res=2, cap=5), cost=4, staged=Σ0)` →
    `(1, 2, overdrawn=false)`
  - `project((curr=1, res=3, cap=3), cost=3, staged=Σ0)` →
    `(0, 1, overdrawn=false)` (spillover)
  - `project((curr=1, res=1, cap=2), cost=5, staged=Σ0)` →
    `(0, 0, overdrawn=true)` (clamped saturating)
  - `project((curr=6, res=0, cap=6), cost=2, staged=(3,0))` →
    `(1, 0, overdrawn=false)` (multi-card current)
  - `project((curr=0, res=5, cap=5), cost=2, staged=(0,2))` →
    `(0, 1, overdrawn=false)` (multi-card reserve)
  - `project((curr=3, res=0, cap=3), cost=0, staged=Σ0)` →
    `(3, 0, overdrawn=false)` (zero-cost card; no change)

- [ ] **AC11 — Advisory overdraw colour treatment**: GIVEN AC5
  preconditions, WHEN the HUD paints the preview, THEN the
  `ManaLabel` `Text` (or its surrounding `BackgroundColor`) reads
  the §7 `SEMANTIC_ERROR` palette colour (`#9C2000` or
  `#E2452E` token, whichever the §7 spec ratifies; consume by
  symbol, never by numeric literal — per Story 020 AC1 precedent).
  This AC is **ADVISORY** (lead sign-off acceptable in lieu of
  ECS assertion); the BLOCKING assertion is AC5's marker / state-
  field presence.

- [ ] **AC12 — ADR-021 plugin registration + entity counts
  preserved**: GIVEN the post-implementation build, WHEN inspected,
  THEN:
  - `HandUiPlugin` and `HudPlugin` remain at their existing
    registration positions inside `PresentationPlugin`.
  - `HAND_UI_ENTITY_COUNT` is unchanged (no new pre-pooled top-level
    entity; the preview lives on the existing `ManaLabel` entity
    as a `Component` or in a `Resource`).
  - The HUD entity set (`HudEntities`) is unchanged (or extended
    only by a `mana_preview_marker` field if absolutely necessary,
    but **preferred**: store the preview as a `Component` on the
    existing `ManaLabel` entity to avoid `HudEntities` churn).

- [ ] **AC13 — ADR-002 + ADR-012 binding preserved**: GIVEN the
  post-implementation build, WHEN the new systems are inspected,
  THEN:
  - They read `Res<PlayerEconomyView>` (immutable), `Res<ActivePlacementDrag>`
    (immutable), `Res<HandCardCatalog>` (immutable), `Res<PendingPlacements>`
    (immutable), `Res<CurrentClientPhase>` (immutable).
  - They write `ResMut<ManaDisplayState>` (or, equivalently,
    `ResMut<ManaPreviewState>` which `sync_mana_text_system` reads).
  - They do NOT write `ResMut<PlayerEconomyView>`.
  - They do NOT add any `S2C*` / `C2S*` message; `shared/src/protocol.rs`
    diff is empty.
  - `liv-bevy-lightyear` is NOT activated.
  Verified by grep across `client/src/ui/hand/`, `client/src/ui/hud/`,
  `client/src/presentation/shared/` for new `ResMut<PlayerEconomyView>`
  (zero hits) and by `git diff shared/src/protocol.rs` (empty).

- [ ] **AC14 — Authoritative-restore precedence over preview**:
  GIVEN a drag is active AND a preview projection is applied AND
  a fresh `S2CGoldUpdate` is received mid-drag (e.g. server-side
  income tick), WHEN `drain_gold_update_receiver_system` runs,
  THEN:
  - `PlayerEconomyView` updates to the new authoritative values.
  - The preview projection re-computes against the new authoritative
    baseline (re-applying the same drag delta).
  - `ManaDisplayState` reflects the new authoritative pool minus the
    in-flight delta on the next `sync_mana_text_system` tick.
  - No stale preview value is rendered for one frame.
  Rationale: authoritative state always wins; preview is a derived
  view, not a replacement.

- [ ] **AC15 — Targeted regressions pass**: GIVEN the post-
  implementation build, WHEN the following tests run, THEN all
  PASS (existing hand-UI / HUD coverage must not regress):
  - `cargo test -p client --lib`
  - `cargo test -p client --test hand_ui_submit_prevalidation_test`
  - `cargo test -p client --test hand_ui_reserve_mana_strip_test`
  - `cargo test -p client --test hand_ui_drag_state_visuals_test`
  - `cargo test -p client --test hand_ui_drag_to_board_cell_test`
  - `cargo test -p client --test hud_mana_text_test` (if extant)
  - `cargo test -p client --test placement_perspective_snapshot_test`
  - The 6-viewport invariant suite at
    `tests/integration/ui_viewport_invariants_test.rs`.

- [ ] **AC16 — No accept-risk closure claimed**: GIVEN the
  implementation evidence, WHEN inspected, THEN it explicitly does
  NOT claim closure of `S8-QA-001-W1`, `QA-COND-0005`,
  `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, R1
  (drag-pipeline-dead bug), R3 (idle-hand affordance — sibling
  story 023), or `AUDIT-1076-02 / 03` (server-side placement loss).

- [ ] **AC17 — Sprint 17 / Sprint 18 disposition preserved**: GIVEN
  the story commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-17.md`, `production/sprints/sprint-18.md`
  (if present at integration time), `production/stage.txt`, and PROMPT
  761 gate-check artifact are diffed, THEN none of them are
  modified by this story or by the `/dev-story` paperwork closure
  beyond the standard `/story-done` row flip (which happens only
  AFTER Sprint 18 is independently activated by a separate prompt).

- [ ] **AC18 — Hand UI EPIC count updated**: GIVEN the epic file
  `production/epics/hand-ui/EPIC.md`, WHEN updated by the
  `/story-done` paperwork at terminal disposition, THEN the
  "Stories" table reflects this story 022 row consistently with the
  existing rows-not-yet-folded note. PROMPT 1136 authoring **DOES**
  add the story 022 row to the table as `Draft (future Sprint 18
  candidate)`; `/story-done` paperwork later flips it to `Done`.

---

## Implementation Notes (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

The preview computation is a pure function:

```rust
// Returns (current_proj, reserve_proj, overdrawn).
// staged_current = sum of current_mana_spend over PendingPlacements.
// staged_reserve = sum of reserve_mana_spend over PendingPlacements.
fn project_mana_preview(
    view: &PlayerEconomyView,
    drag_card: Option<(CardId, &CardDef)>,
    staged_current: u32,
    staged_reserve: u32,
) -> (u32, u32, bool) {
    let baseline_current = view.current_mana.saturating_sub(staged_current);
    let baseline_reserve = view.reserve_mana.saturating_sub(staged_reserve);

    let Some((_, card)) = drag_card else {
        return (baseline_current, baseline_reserve, false);
    };
    if card.card_type != CardType::Minion {
        return (baseline_current, baseline_reserve, false);
    }

    let cost = card.cost;
    let from_current = cost.min(baseline_current);
    let from_reserve_demand = cost.saturating_sub(from_current);
    let from_reserve = from_reserve_demand.min(baseline_reserve);
    let overdrawn = from_current + from_reserve < cost;

    (
        baseline_current.saturating_sub(from_current),
        baseline_reserve.saturating_sub(from_reserve),
        overdrawn,
    )
}
```

The projection is consumed by a new system, e.g.
`sync_mana_preview_during_drag_system`, registered in
`PresentationSet::StateSync` **before** `sync_mana_text_system` so
the projection is the value `sync_mana_text_system` paints.

The reset path on drag end is the same system: when
`ActivePlacementDrag::is_active() == false`, the system writes the
authoritative baseline (`view.current_mana`, `view.reserve_mana`)
into `ManaDisplayState`, restoring it before the next paint.

The system gate is:

```rust
.in_set(PresentationSet::StateSync)
.before(sync_mana_text_system)
.run_if(in_state(ClientState::InSession))
.run_if(resource_equals(CurrentClientPhase::Placement))
```

Optionally, a `Res<ManaPreviewState>` may carry the `overdrawn` flag
for the `ManaPreviewOverdrawn` marker check in AC5. If the worker
prefers, store the flag as a `Component` on the `ManaLabel` entity
(`MaybeOverdrawn(true|false)`); both are acceptable. The AC9 test
queries whichever form the worker chose.

---

## Likely Files (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/mod.rs` | Optionally re-export `ActivePlacementDrag::is_active` and `HandCardCatalog` to the HUD module if not already accessible. **No structural change**; the existing public exposures should suffice. |
| `client/src/ui/hud/mod.rs` | Add `sync_mana_preview_during_drag_system` (new), register it before `sync_mana_text_system` in `PresentationSet::StateSync`. Add `ManaPreviewState` resource (or `MaybeOverdrawn` component on `ManaLabel`). Add a small helper `project_mana_preview` (pure function; ideally extracted to `client/src/ui/hud/mana_preview.rs` for testability). |
| `client/src/ui/hud/mana_preview.rs` (NEW, optional) | Host the pure projection function and the `ManaPreviewState` resource definition. Re-exported from `client/src/ui/hud/mod.rs`. |
| `tests/unit/hand-ui/mana_preview_projection_test.rs` (NEW) | Unit-test the pure projection arithmetic per AC10. |
| `tests/integration/hand-ui/mana_preview_during_drag_test.rs` (NEW) | Integration test per AC9. Drives drag via direct resource insertion. |
| `design/gdd/hand-ui.md` | Optional light addition: one row in Rule 6 documenting the HUD mana preview. Worker discretion — may be deferred to a follow-on doc story. |
| `docs/architecture/tr-registry.yaml` | Add `TR-HU-009 — Mana preview during PLACEMENT drag` row. Worker discretion — may be deferred. |
| `production/epics/hand-ui/story-022-hand-mana-preview-during-drag.md` | This file. Status flipped `Draft → Ready` by `/story-readiness` post Sprint 18 activation; `Ready → Done` by `/story-done` post-implementation. |
| `production/epics/hand-ui/EPIC.md` | `/story-done` paperwork: refresh story-022 row to `Done` and update count-deferral note. PROMPT 1136 authoring adds the row in `Draft` status. |

**Explicitly out of scope for the `/dev-story` worker** (any of these
constitutes a scope violation per the Forbidden Control Manifest
Rules):

- `shared/src/protocol.rs` — no protocol-shape edit; AC13.
- `server/` — no server-side preview code.
- `client/src/network/` / `server/src/network/` — no networking
  surface change.
- `client/src/ui/shop_auction/` — Shop/Auction UI does not consume
  the preview.
- `client/src/ui/lobby.rs` — out of host module.
- `client/src/presentation/board_rendering*` — board-side highlights
  remain owned by Board Rendering (Story 006); the preview is HUD-
  only.
- `production/sprints/*`, `production/sprint-status.yaml`,
  `production/stage.txt`, PROMPT 761 gate-check artifact — not
  touched by this story.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/` — not touched.
- The R1 repair files (`client/Cargo.toml` `default = []` flag,
  `client/src/main.rs` `DefaultPickingPlugins`,
  `client/src/ui/hand/mod.rs` cursor coord-space lines 405 / 2537-2551 /
  3182-3208) — those are a separate prompt for R1; this story
  must not touch them.
- `story-023-hand-idle-playable-affordance.md` — sibling R3 story;
  parallelism, not coupling.

---

## Out of Scope

- **R1 — drag pipeline dead in shipped build**. The `ui_picking`
  feature gate, the `DefaultPickingPlugins` absence, and the
  cursor-coord-space mismatch are a separate repair prompt.
  This story's AC9 integration test deliberately drives drag via
  direct resource insertion to remain independent of R1.
- **R3 — idle-hand playable affordance**. Sibling future-Sprint-18
  candidate at `story-023-hand-idle-playable-affordance.md`.
  Parallel-safe with this story.
- **AUDIT-1076-02 / AUDIT-1076-03** — server-side placement loss /
  submission survival bugs (PROMPT 1076 / 1127 §1.6). Server-only
  fix; out of host module for this story.
- **S2CPlacementRejected** — there is no S2C message communicating
  server-side validation failure to the client today (PROMPT 1127
  §1.6). Adding such a message is a separate networking-protocol
  story; this story is HUD-display-only.
- **Reserve-strip-mutated reserve_mana_spend visual indicator**.
  Story 011 already shows the per-staged-card reserve `[+]` / `[-]`
  control with its value text; this story's preview operates on
  the in-flight (un-staged) drag card's default split only. A
  player who wants a non-default split on a staged card uses the
  Story 011 strip after staging.
- **Animation / tween on the projected mana number**. The preview
  is a step change (instantaneous on drag start; instantaneous reset
  on drag end). A tween for the projected delta is out of scope;
  worker discretion to add a follow-on Polish story if the manual
  evidence run flags the step change as jarring.
- **Multi-Minion drag-sequence preview** (preview both the current
  drag and a planned next drag). Out of scope; the preview applies
  only to the **single** card currently held by `ActivePlacementDrag::card`.
- **Hover-preview without drag** (mana projection while hovering
  a hand card with no drag in flight). Out of scope; the trigger is
  strictly `ActivePlacementDrag::is_active() == true`. If the
  product team later wants a "hover preview", a separate story
  extends AC1.
- **WCAG contrast verification** on `SEMANTIC_ERROR` overdraw text.
  `QA-COND-0005` accepted-risk.
- **Final-art / asset replacement** on the HUD mana label chrome.
  `PAW-TD-*-a` accepted-risk.
- **Sprint 18 activation**. PROMPT 1136 (this authoring) does NOT
  activate Sprint 18.
- **`/qa-plan sprint-18` authoring**. Owned by a separate prompt
  after Sprint 18 activation.
- **`/story-readiness` on this story**. Run as a separate prompt
  after Sprint 18 activation.
- **`/dev-story` on this story**. Run only after Sprint 18
  activation, after `/qa-plan sprint-18` (if any), and after
  `/story-readiness` passes against Sprint 18 activation HEAD.
- **Polish → Release gate-check retry**. PROMPT 761 FAIL preserved.
- **Stage advance from Polish to Release**. `production/stage.txt`
  remains `Polish`.

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against
these — do not invent new test cases during implementation.*

- **AC1 + AC2 — Preview activates and paints projected current
  mana**:
  - Given: PLACEMENT phase; Minion `card.cost = 4` in hand;
    `PlayerEconomyView { current_mana: 5, reserve_mana: 2, mana_cap: 5 }`;
    no staged placements.
  - When: `ActivePlacementDrag::start(card_id, ...)` writes
    `is_active() == true`; one `App::update()` tick.
  - Then: `ManaDisplayState.current_mana == 1`;
    `ManaDisplayState.reserve_mana == 2`; HUD `ManaLabel.Text::0`
    paints the projected current mana on the next
    `sync_mana_text_system` tick.

- **AC3 — Reserve spillover**:
  - Given: PLACEMENT; Minion `cost = 3`;
    `PlayerEconomyView { current: 1, reserve: 3, cap: 3 }`.
  - When: drag start; one tick.
  - Then: `ManaDisplayState.current_mana == 0`;
    `ManaDisplayState.reserve_mana == 1`; reserve diamond
    `Visibility::Visible`.

- **AC4 — Reset on drag end**:
  - Given: drag is active per AC1.
  - When: `ActivePlacementDrag::clear()` (drag end / cancel / drop);
    one tick.
  - Then: `ManaDisplayState.current_mana == 5`;
    `ManaDisplayState.reserve_mana == 2`; `ManaPreviewOverdrawn`
    marker absent; HUD paints authoritative values on next tick.

- **AC5 — Overdraw indicator**:
  - Given: PLACEMENT; Minion `cost = 5`;
    `PlayerEconomyView { current: 1, reserve: 1, cap: 5 }`.
  - When: drag start; one tick.
  - Then: `ManaDisplayState.current_mana == 0`;
    `ManaDisplayState.reserve_mana == 0`; `ManaPreviewOverdrawn`
    marker present (or `Res<ManaPreviewState>.overdrawn == true`).

- **AC6 — Multi-card subtraction**:
  - Given (current branch): PLACEMENT; one staged
    `PlacedCardSubmit { current_mana_spend: 3, reserve_mana_spend: 0 }`;
    Minion `cost = 2`;
    `PlayerEconomyView { current: 6, reserve: 0, cap: 6 }`.
  - Then: projected `current_mana == 1`.
  - Given (reserve branch): PLACEMENT; one staged
    `PlacedCardSubmit { current_mana_spend: 0, reserve_mana_spend: 2 }`;
    Minion `cost = 2`;
    `PlayerEconomyView { current: 0, reserve: 5, cap: 5 }`.
  - Then: projected `current_mana == 0`; projected
    `reserve_mana == 1`.

- **AC7 — Suppressed outside PLACEMENT**:
  - Given: `Phase::DraftShop`; any drag scenario constructed for
    testability.
  - When: one tick.
  - Then: `ManaDisplayState` reflects authoritative
    `PlayerEconomyView`; no projection applied.

- **AC8 — Non-Minion suppression**:
  - Given: PLACEMENT; drag-source card has `CardType::Spell`;
    `PlayerEconomyView { current: 5, reserve: 2, cap: 5 }`;
    any cost value.
  - When: drag start; one tick.
  - Then: `ManaDisplayState.current_mana == 5`;
    `ManaDisplayState.reserve_mana == 2`; `ManaPreviewOverdrawn`
    marker absent.

- **AC14 — Authoritative S2CGoldUpdate mid-drag**:
  - Given: drag active per AC1 with projected current `1`.
  - When: `S2CGoldUpdate { current_mana: 7, reserve_mana: 2, ... }`
    is drained.
  - Then: `PlayerEconomyView.current_mana == 7`; projected
    `ManaDisplayState.current_mana == 3` (`7 - 4 = 3`); no stale
    `1` paint persists.

---

## Test Evidence

**Story Type**: Logic + Integration (paired BLOCKING tests).

**Required evidence**:
- `tests/unit/hand-ui/mana_preview_projection_test.rs` — must exist
  and pass (AC10; pure projection arithmetic).
- `tests/integration/hand-ui/mana_preview_during_drag_test.rs` — must
  exist and pass (AC9; reactive UI mutation with simulated drag via
  direct resource insertion).

**Status**: [ ] Created and passing (BLOCKED until `/dev-story`
runs post Sprint 18 activation).

**Advisory evidence**:
- `production/qa/evidence/sprint-18-hand-mana-preview/README.md`
  (optional manual capture; AC11 advisory overdraw colour
  walkthrough).

---

## Performance Budget

Per ADR-021 Presentation steady-state budget of `< 1 ms` per frame.
The projection system performs:

- `O(1)` resource reads (`PlayerEconomyView`, `ActivePlacementDrag`,
  `CurrentClientPhase`).
- `O(n)` over `PendingPlacements::placements` where `n ≤ 10` (hand
  size cap) — integer sums only.
- `O(1)` `HandCardCatalog::cards.get(&card_id)` lookup.
- One write to `ManaDisplayState` fields.

Expected per-frame cost: `< 10 µs`. Well within ADR-021 budget.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker conflates this missing-feature story with the R1 drag-pipeline-dead bug repair. | Low | High | "No-Claim Banner" and "Out of Scope" explicitly disjoin R1 and R2; AC9 drives drag via direct resource insertion to remove the R1 dependency. |
| Worker writes the projection through `ResMut<PlayerEconomyView>`, regressing ADR-002 client-authority. | Low | High | AC13 explicitly forbids; grep gate listed in AC13 verification. |
| Worker projects mana for non-Minion card types and breaks Story 010 / Story 011 invariants. | Low | Medium | AC8 explicit suppression; reuses existing `slot_is_affordable` Minion-only fast-path. |
| Worker reset path leaks a stale projection one frame past drag end. | Medium | Medium | AC4 explicitly asserts same-frame reset; system ordering before `sync_mana_text_system` is in Implementation Notes. |
| Worker introduces a tween on the projected number, conflicting with existing HUD mana tweens (PROMPT 974 / `is_hud_tween_active`). | Medium | Medium | Implementation Notes call out step-change behaviour; tween is explicitly out of scope. |
| Worker invents a new Lightyear `C2SPreviewPlacement` round-trip. | Low | High | AC13 + ADR-002 forbid; `liv-bevy-lightyear` NOT activated. |
| Worker activates Sprint 18 as a side effect of `/dev-story` paperwork. | Low | Medium | No-Claim Banner forbids; `/story-done` paperwork serialization is the orchestrator's contract, not the worker's. |
| Worker touches `production/epics/hand-ui/story-020-hand-drag-state-visuals.md` to "amend AC2" for the new affordance overlay (cross-pollination with story 023). | Low | Low | Story 020 AC2 query (`Without<DragStateOverlay>`) is preserved as authored; the sibling story 023 introduces a **distinct** affordance marker (`FanSlotPlayableAffordanceOverlay`) that does NOT match `DragStateOverlay`, so Story 020 AC2 stays valid without edit. This story does not edit Story 020 either. |
| Worker discovers an R1.b coord-space bug while implementing AC9 and is tempted to repair it. | Low | Medium | R1.b is owned by the separate R1 repair prompt; AC9 deliberately bypasses cursor-to-cell mapping by inserting `ActivePlacementDrag` directly. If a runtime cursor bug surfaces in manual evidence, it is recorded in a separate follow-on story, NOT repaired here. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for the PROMPT 1136 authoring run itself
(which is paperwork-only):

- `production/sprint-status.yaml` top-level `sprint:` field reads
  `18` (after Sprint 18 activation) and the row for
  `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001` is `ready` at the time
  `/dev-story` is dispatched.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-18.md` shows the ACTIVATED banner (added
  by Sprint 18 activation; not added by PROMPT 1136).
- PROMPT 761 Polish → Release gate-check FAIL evidence preserved.
- `production/qa/qa-plan-sprint-18.md` (if extant) references this
  story.
- `/story-readiness` on this story file returns READY against the
  Sprint 18 activation HEAD.
- Sprint 12 story 019 disposition preserved on `origin/main`.
- `git diff --check` and `git diff --cached --check` pass before
  commit.
- R1 repair status: this story does NOT require R1 to be merged
  before `/dev-story` runs (AC9 is R1-independent). However, the
  manual evidence at AC11 may benefit from R1 being merged first;
  the orchestrator may choose to sequence R1 before this story to
  maximise end-to-end evidence quality.

---

## Authoring Trail

- 2026-05-18 — PROMPT 1136 — Story file authored as future
  Sprint 18 candidate `S18-UI-HAND-MANA-PREVIEW-DURING-DRAG-001`.
  Worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s18-hand-mana-affordance-stories`,
  branch `work/s18-hand-mana-affordance-stories`, base
  `origin/main@05192b5f830c5d5b17ed7af07df37f56187130fc` (PROMPT
  1125 `story-done(s17): close S17-OPS-VULKAN-VALIDATION-GATING-001`).
  Files touched by this authoring run: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (story-list row added; count-
  deferral note updated). Sibling
  `production/epics/hand-ui/story-023-hand-idle-playable-affordance.md`
  authored in the same run as the R3 missing-feature candidate
  (PROMPT 1127 §R3). Sprint 18 NOT activated. No code change. No
  `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`,
  or `trunk` command run. ADR-002 + ADR-012 + ADR-021 binding
  preserved; Sprint 12 story 019 disposition preserved; PROMPT 761
  gate-check FAIL preserved; `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `S8-QA-001-W1`, `TQ-S12-C1..C7` all preserved
  verbatim. R1 repair status unchanged (separate prompt). PROMPT
  1127 R2 missing-feature is now formally documented as a Sprint 18
  candidate story; no implementation is performed by this authoring
  run.
