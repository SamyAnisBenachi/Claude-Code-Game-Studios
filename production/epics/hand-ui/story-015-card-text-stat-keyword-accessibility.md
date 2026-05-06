# Story 015: Card Text, Stat, and Keyword Accessibility

> **Epic**: Hand UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-04 / QA-COND-0005

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility evidence register row `A11Y-ST-02` states that card cost, ATK, HP, and keyword text do not yet have browser/WASM measurement evidence. This story owns the Hand UI implementation slice for hand and fan card surfaces. The final cross-surface browser/WASM evidence is owned by Presentation Layer Story 005.

**Primary sources**:

- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
- `design/accessibility-requirements.md`
- `design/gdd/card-data-pool.md`
- `design/gdd/hand-ui.md`
- `design/ux/hand-ui.md`

**Accessibility requirement**:

- `design/accessibility-requirements.md` Standard-tier row `Minimum text size - card text (cost, ATK, HP, keyword)` requires stat badges for ATK and HP to have an 18 px minimum floor and keyword text to have a 14 px minimum floor. This story applies the same 18 CSS px floor to visible card cost numerals because `A11Y-ST-02` explicitly includes card cost in the evidence gap.

**GDD and UX trace**:

- `design/gdd/card-data-pool.md` card definition schema exposes display fields including `cost`, `atk`, `hp`, `keywords`, and `effect_text`.
- `design/gdd/hand-ui.md` Rule 1 requires pre-pooled 10 hand fan card slots and a reused drag sprite.
- `design/gdd/hand-ui.md` Rule 3 requires the fan to be visible in DRAFT_SHOP, read-only in DRAFT_AUCTION, and interactive in PLACEMENT.
- `design/gdd/hand-ui.md` Rule 8 and Rule 13 require staged fan ghosts and reserve/current split controls to preserve card identity during PLACEMENT.
- `design/gdd/hand-ui.md` VA-1 defines card cost, ATK, HP, type, rarity, and hover zoom anatomy for the hand card face.
- `design/ux/hand-ui.md` requires card cost, ATK, HP, and type or rarity to remain readable at 10-card hand compression.

**TR IDs**:

- `TR-HU-001` for pre-pooled hand fan card slots.
- `TR-HU-002` for PLACEMENT drag-to-stage state and cursor mapping.
- `TR-HU-003` for Instant card staging.
- `TR-HU-004` for reserve mana split controls attached to staged cards.
- `TR-HU-005` for DRAFT_INITIAL card acquisition flow into the hand fan.
- `TR-CDP-010` for server-provided draft and shop card payloads arriving before client phase/UI use.

`A11Y-ST-02` is a Sprint 6 accessibility evidence row, not a registered `TR-PRES-*` requirement.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: Hand card readability is presentation-only. Hand UI reads server-authoritative phase, hand, card, and economy state, then renders readable hand card surfaces. Remediation must not mutate authoritative hand ownership, card data, phase, mana, reserve mana, or placement state.

**Engine**: Bevy 0.18 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file. Hand card text and badges must remain Bevy 0.18 presentation UI or sprite-backed presentation elements using the Required Components API. Do not use `NodeBundle`, `TextBundle`, `SpriteBundle`, `UiImage::new()`, `Parent`, or `Color::rgba()`. Evidence or measurement helpers must remain test-only or evidence-only unless explicitly promoted to player-facing UI by a later story.

**Control Manifest Rules (2026-05-05)**:

- Required: client presentation is a read-only view of server-authoritative state.
- Required: Hand UI reads phase through `Res<CurrentClientPhase>` and never drains `MessageReceiver<S2CPhaseChanged>`.
- Required: Hand UI reads `Res<PlayerEconomyView>` and never drains economy S2C messages directly.
- Required: UI surfaces such as hand fan and drag sprite use `bevy_ui`; board content remains world-space.
- Required: `PresentationSet` order remains `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: `PresentationPlugin` registration order remains Card Animations, Board Rendering, Hand UI, HUD, Shop/Auction UI.
- Forbidden: client presentation must not assert or mutate authoritative game state.
- Guardrail: presentation steady-state stays below 1 ms per frame and phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Verify or remediate card cost, ATK, HP, and keyword text floors on Hand UI owned card surfaces:
  - DRAFT_SHOP passive hand/fan cards at rest.
  - DRAFT_SHOP hand/fan hover or zoom card view.
  - DRAFT_AUCTION read-only hand/fan cards.
  - PLACEMENT active hand/fan cards before staging.
  - PLACEMENT selected-card and drag-sprite presentation.
  - PLACEMENT staged fan ghosts.
  - DRAFT_INITIAL card-acquired fan landing state after a purchase confirmation.
- Apply a minimum floor of 18 CSS px or browser-equivalent rendered pixels to visible cost, ATK, and HP numerals or badge text.
- Apply a minimum floor of 14 CSS px or browser-equivalent rendered pixels to visible keyword text when keyword text is exposed on the hand surface, hover view, or zoom view.
- Add or update test-observable card text metrics, card bounds, text bounds, and overlap checks for Hand UI surfaces.
- Preserve existing Hand UI behavior: hand count, phase visibility, drag-to-stage, Instant staging, staged fan ghost, reserve split controls, submit pre-validation, activation lock behavior, and reconnect rebuild semantics.
- Produce the Hand UI implementation evidence note listed in `## Test Evidence`. Final browser/WASM cross-surface evidence remains owned by Presentation Layer Story 005.

### Out of Scope

- No Shop/Auction UI card surfaces.
- No DRAFT_INITIAL 3 x 3 grid slot rendering. Shop/Auction UI owns those slot entities and Story 013 owns their card text remediation.
- No auction featured card, auction preparing card, settlement card, DRAFT_SHOP shop slot, or DRAFT_AUCTION read-only shop footer remediation.
- No final cross-surface browser/WASM capture package. Presentation Layer Story 005 owns that evidence pass.
- No changes to authoritative card catalog fields, card costs, ATK, HP, keywords, rarity, effect text, or pool distribution.
- No changes to economy, auction bidding, purchase validation, hand ownership, placement legality, phase timing, network protocol payloads, or server authority.
- No broader UI scaling preference implementation.
- No colorblind palette implementation.
- No Settings or Accessibility screen work.
- No changes to `production/sprint-status.yaml`.
- No changes to `production/session-state/**`.
- No changes to project asset files.
- No changes to `AGENTS.md`.
- Do not close QA-COND-0005 from this story alone.

---

## Acceptance Criteria

- [ ] **Hand/fan cost floor passes**: GIVEN the Hand UI fan renders at 10-card compression in DRAFT_SHOP, DRAFT_AUCTION read-only, and PLACEMENT, WHEN hand card text metrics are exported, THEN every visible cost numeral or badge text measures at least 18 CSS px or browser-equivalent rendered pixels.
- [ ] **Hand/fan ATK floor passes**: GIVEN a visible minion or structure hand card has an ATK field, WHEN hand card text metrics are exported, THEN every visible ATK numeral or badge text measures at least 18 CSS px or browser-equivalent rendered pixels.
- [ ] **Hand/fan HP floor passes**: GIVEN a visible minion or structure hand card has an HP field, WHEN hand card text metrics are exported, THEN every visible HP numeral or badge text measures at least 18 CSS px or browser-equivalent rendered pixels.
- [ ] **Keyword floor passes on the accepted readable state**: GIVEN a card has one or more keywords, WHEN the accepted readable hand state is rendered at rest, hover, or zoom, THEN the visible keyword text measures at least 14 CSS px or browser-equivalent rendered pixels.
- [ ] **Accepted readable state is explicit**: GIVEN a hand card surface hides keyword text at rest because the 10-card fan is compressed, WHEN metrics are exported, THEN the record identifies hover or zoom as the accepted readable state and still records the rest state as not presenting keyword text.
- [ ] **Not-applicable fields are explicit**: GIVEN a card type has no ATK, no HP, or no visible keyword text in a Hand UI state, WHEN the Hand UI metric table is generated, THEN that field is recorded as `N/A - field not present on this card type or state`.
- [ ] **Long and dense cards remain inside bounds**: GIVEN fixture cards include a long name, a zero-cost card, a two-digit cost card, a two-digit ATK or HP card, a no-keyword card, and a multi-keyword card, WHEN Hand UI bounds checks run, THEN visible cost, ATK, HP, and keyword text remains inside the card, badge, hover view, or zoom bounds without clipping into unreadability.
- [ ] **No hand-card internal overlap**: GIVEN Hand UI text bounds and card bounds are exported, WHEN overlap checks run, THEN cost, ATK, HP, keyword text, card name, rarity indicator, and visible card art zones do not overlap in a way that makes required text unreadable.
- [ ] **No fan-surface overlap**: GIVEN the hand/fan renders 10 cards, staged fan ghosts, and reserve/current split strips, WHEN surface overlap checks run, THEN required card text does not overlap sibling cards, fan ghosts, reserve/current split controls, Submit, timer, HUD chips, or evidence overlays.
- [ ] **Read-only and ghost treatments preserve readability**: GIVEN DRAFT_AUCTION read-only fan cards and PLACEMENT staged fan ghosts are rendered, WHEN metrics and bounds are exported, THEN opacity, desaturation, and ghost treatment do not reduce visible cost, ATK, HP, or accepted keyword text below the required floors.
- [ ] **Drag sprite preserves stat readability**: GIVEN a PLACEMENT card is selected or dragged, WHEN the drag sprite is visible, THEN the drag presentation keeps visible cost, ATK, and HP text at the required 18 CSS px floor and does not hide the source fan slot state needed for layout stability.
- [ ] **Existing Hand UI behavior is preserved**: GIVEN the Hand UI accessibility changes are present, WHEN the existing Hand UI regression commands listed in `## Test Evidence` run, THEN phase visibility, fan layout, draft acquisition, drag highlights, Instant staging, un-staging, reserve split, submit pre-validation, timer, and staged disclosure behavior remains unchanged.
- [ ] **Focused Hand UI accessibility test passes**: `cargo test -p client --test hand_ui_card_text_stat_keyword_accessibility_test` passes. The target must be backed by `tests/integration/hand-ui/card_text_stat_keyword_accessibility_test.rs` and registered as `hand_ui_card_text_stat_keyword_accessibility_test`.
- [ ] **Hand UI implementation evidence exists**: `production/qa/evidence/hand-ui-card-text-stat-keyword-accessibility-2026-05-06.md` records the changed constants or components, fixture cards, automated text-size table, overlap table, regression command output summary, and hand-surface readiness statement for Presentation Layer Story 005.
- [ ] **A11Y-ST-02 impact is explicit**: The Hand UI evidence note states that this story implements the Hand UI slice of A11Y-ST-02 only, and that final cross-surface browser/WASM evidence remains owned by Presentation Layer Story 005.
- [ ] **QA-COND-0005 remains open**: The Hand UI evidence note states that QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, dependency-blocked, or accepted as risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep work local to Hand UI card rendering, hand/fan layout constants, text/badge sizing, hover or zoom surfaces, test fixtures, and evidence-only measurement hooks.
- Prefer adjusting existing typography constants, badge dimensions, fan text slots, hover or zoom presentation, and test-observable measurement components over changing card data or game logic.
- If a hand card surface intentionally hides keyword text at rest and exposes it on hover or zoom, the implementation must make that state test-observable and document it as the accepted player-readable state.
- Do not add fake ATK, HP, or keyword text to cards that do not have those fields. Record those fields as not applicable.
- If keyword text wraps in hover or zoom, the wrapped block must meet the 14 CSS px floor, remain inside the card surface, and avoid overlap with stats or card controls.
- Do not reduce DRAFT_AUCTION read-only fan opacity or PLACEMENT staged ghost opacity below the GDD-specified communication treatment as a readability shortcut. Preserve the treatment and make the readable state pass.
- Keep any measurement overlay, metric export, or test marker out of normal shipping UI unless a later UX story promotes it.

## Performance Budget

No gameplay-loop performance impact is expected from typography adjustments or evidence-only measurement instrumentation. Any production remediation must preserve the ADR-021 presentation guardrails: steady-state presentation work remains below 1 ms per frame and phase-boundary spikes remain below 3 ms. The implementation must not add per-frame entity creation, extra Lightyear message drains, card catalog scans, texture uploads, or persistent debug overlays.

---

## QA Test Cases

- **Hand/fan text measurement**
  - Given: The hand/fan renders 10 cards in DRAFT_SHOP, DRAFT_AUCTION read-only, and PLACEMENT.
  - When: text metrics and bounds are exported.
  - Then: visible cost, ATK, and HP fields meet the 18 CSS px floor, accepted keyword text meets the 14 CSS px floor, and absent fields are explicitly recorded as not applicable.

- **Hover or zoom readable state**
  - Given: a compressed hand card has hidden or abbreviated keyword text at rest.
  - When: hover or zoom is rendered.
  - Then: keyword text is visible at the 14 CSS px floor and the metric record identifies hover or zoom as the accepted readable state.

- **Staged ghost readability**
  - Given: a PLACEMENT hand card is staged and displayed as a fan ghost.
  - When: metrics and bounds are exported.
  - Then: visible cost, ATK, HP, and accepted keyword text remain readable through the ghost treatment without overlapping reserve/current split controls.

- **Overlap guard**
  - Given: 10-card fan compression, staged ghosts, and reserve/current split controls are visible.
  - When: overlap checks compare card text, sibling cards, hand controls, HUD chips, Submit, and timer bounds.
  - Then: no required Hand UI card text overlaps another element in a way that makes it unreadable.

- **Behavior preservation**
  - Given: Hand UI card readability changes are present.
  - When: the Hand UI regression suite runs.
  - Then: draft acquisition, fan layout, drag/stage, Instant, un-stage, reserve strip, submit pre-validation, timer, and staged disclosure behavior remains unchanged.

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/hand-ui/card_text_stat_keyword_accessibility_test.rs`
  - Registered as `hand_ui_card_text_stat_keyword_accessibility_test`
  - Command: `cargo test -p client --test hand_ui_card_text_stat_keyword_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test hand_ui_fan_layout_formula_test`
- `cargo test -p client --test hand_ui_draft_initial_grid_test`
- `cargo test -p client --test hand_ui_placement_drag_highlights_test`
- `cargo test -p client --test hand_ui_placement_instant_staging_test`
- `cargo test -p client --test hand_ui_placement_unstaging_test`
- `cargo test -p client --test hand_ui_reserve_mana_strip_test`
- `cargo test -p client --test hand_ui_submit_prevalidation_test`
- `cargo test -p client --test hand_ui_placement_timer_test`
- `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
- `cargo check -p client`
- `git diff --check`

**Required implementation evidence document**:

- `production/qa/evidence/hand-ui-card-text-stat-keyword-accessibility-2026-05-06.md`

**Required evidence contents**:

- Commit, fixture card list, Hand UI surfaces covered, changed typography constants or components, and UI scale assumption.
- Text-size table for cost, ATK, HP, and keyword text by Hand UI surface.
- Explicit not-applicable entries for fields absent from a card type or state.
- Bounds and overlap table for 10-card fan, read-only fan, PLACEMENT selected state, drag sprite, staged ghost, and hover or zoom state.
- Regression command output summary.
- Hand UI readiness statement for Presentation Layer Story 005.
- A11Y-ST-02 impact statement.
- QA-COND-0005 impact statement confirming the condition remains Open.

**Final browser/WASM evidence owner**:

- Presentation Layer Story 005 captures the final cross-surface browser/WASM evidence at `production/qa/evidence/presentation-card-text-accessibility.md`.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold.md) - Complete; provides pre-pooled hand/fan slot ownership.
- Depends on: [Story 002](story-002-fan-layout-formula.md) - Complete; provides 10-card fan compression formula.
- Depends on: [Story 003](story-003-phase-state-machine.md) - Complete; provides phase visibility and passive/read-only/staging state machine.
- Depends on: [Story 004](story-004-draft-initial-grid.md) - Complete; provides DRAFT_INITIAL acquisition into the fan, while Shop/Auction UI remains owner of grid slot rendering.
- Depends on: [Story 005](story-005-placement-submit-core.md) - Complete; provides PLACEMENT staging baseline.
- Depends on: [Story 007](story-007-placement-instant-staging.md) - Complete; provides Instant staging baseline.
- Depends on: [Story 008](story-008-placement-unstaging.md) - Complete; provides staged ghost correction baseline.
- Depends on: [Story 011](story-011-reserve-mana-strip.md) - Complete; provides reserve/current split controls near staged ghosts.
- Depends on: [Story 014](story-014-placement-staged-disclosure-accessibility.md) - Complete; provides staged disclosure evidence baseline to preserve.
- Depends on: ADR-002, ADR-019, and ADR-021 Accepted.
- Unlocks: Hand UI slice for Presentation Layer Story 005 final A11Y-ST-02 browser/WASM evidence.

## Blockers

None.

## No Open Questions

None.
