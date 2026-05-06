# Story 013: Card Text, Stat, and Keyword Accessibility

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-04 / QA-COND-0005

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility evidence register row `A11Y-ST-02` states that card cost, ATK, HP, and keyword text do not yet have browser/WASM measurement evidence. This story owns the Shop/Auction UI implementation slice for DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, and settlement card surfaces. The final cross-surface browser/WASM evidence is owned by Presentation Layer Story 005.

**Primary sources**:

- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
- `design/accessibility-requirements.md`
- `design/gdd/card-data-pool.md`
- `design/gdd/shop-auction-ui.md`
- `design/ux/shop-auction-ui.md`
- `design/ux/interaction-patterns.md`

**Accessibility requirement**:

- `design/accessibility-requirements.md` Standard-tier row `Minimum text size - card text (cost, ATK, HP, keyword)` requires stat badges for ATK and HP to have an 18 px minimum floor and keyword text to have a 14 px minimum floor. This story applies the same 18 CSS px floor to visible card cost numerals because `A11Y-ST-02` explicitly includes card cost in the evidence gap.

**GDD and UX trace**:

- `design/gdd/card-data-pool.md` card definition schema exposes display fields including `cost`, `atk`, `hp`, `keywords`, and `effect_text`.
- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 2 requires a 3 x 3 card grid sorted by rarity and cost.
- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 4 keeps purchased slots in position with a bought overlay, so readable card text must remain bounded when the overlay is present.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 1 requires the auction panel to render the auction card art, rarity badge, and starting price as current price after activation data is available.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 2 requires read-only shop footer card costs to remain visible at 30 percent opacity during auction.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 9 requires settlement card movement and outcome overlays.
- `design/gdd/shop-auction-ui.md` DRAFT_SHOP Rule 2 requires three horizontal shop slots, each showing card art, name, rarity badge, and cost.
- `design/ux/shop-auction-ui.md` requires card keyword text to remain readable on hover or zoom and body text contrast of at least 4.5:1 where sampled against its rendered background.
- `design/ux/shop-auction-ui.md` requires no required UI overlap at `1366x768` and `1920x1080`; this story prepares Shop/Auction UI card surfaces for the final Presentation Story 005 browser/WASM evidence pass.

**TR IDs**:

- `TR-CDP-010` for draft, shop, and auction card payloads arriving before client phase/UI use.
- `TR-SAU-003` for auction settlement display.
- `TR-SAU-004` for locked DRAFT_AUCTION shop footer slots.
- `TR-SAU-006` for Shop/Auction panel transitions and presentation coverage.

`A11Y-ST-02` is a Sprint 6 accessibility evidence row, not a registered `TR-PRES-*` requirement.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md)
- [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: Shop/Auction card readability is presentation-only. Shop/Auction UI reads server-authoritative phase, card acquisition, auction, and economy state, then renders readable card surfaces. Remediation must not mutate authoritative card data, shop slots, auction price, bid state, gold, reserved gold, ownership, or phase state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file. Shop/Auction card text and badges must remain Bevy 0.18 presentation UI or sprite-backed presentation elements using the Required Components API. Do not use `NodeBundle`, `TextBundle`, `SpriteBundle`, `UiImage::new()`, `Parent`, or `Color::rgba()`. Evidence or measurement helpers must remain test-only or evidence-only unless explicitly promoted to player-facing UI by a later story.

**Control Manifest Rules (2026-05-05)**:

- Required: client presentation is a read-only view of server-authoritative state.
- Required: Shop/Auction UI reads phase through `Res<CurrentClientPhase>` and never drains `MessageReceiver<S2CPhaseChanged>`.
- Required: Shop/Auction UI reads `Res<PlayerEconomyView>` and never drains economy S2C messages directly.
- Required: UI surfaces such as shop panels, auction panels, and card slots use `bevy_ui`; board content remains world-space.
- Required: `PresentationSet` order remains `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: `PresentationPlugin` registration order remains Card Animations, Board Rendering, Hand UI, HUD, Shop/Auction UI.
- Forbidden: client presentation must not assert or mutate authoritative game state.
- Guardrail: presentation steady-state stays below 1 ms per frame and phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Verify or remediate card cost, ATK, HP, and keyword text floors on Shop/Auction UI owned card surfaces:
  - DRAFT_INITIAL 3 x 3 offering grid cards.
  - DRAFT_INITIAL purchased-slot bought overlay state.
  - DRAFT_SHOP three shop slots.
  - DRAFT_AUCTION read-only shop footer cards at the GDD-specified 30 percent opacity.
  - DRAFT_AUCTION auction featured card in preparing and active states.
  - Auction settlement card view and outcome overlay adjacency.
- Apply a minimum floor of 18 CSS px or browser-equivalent rendered pixels to visible cost, ATK, and HP numerals or badge text.
- Apply a minimum floor of 14 CSS px or browser-equivalent rendered pixels to visible keyword text when keyword text is exposed on shop, auction, hover, or zoom surfaces.
- Add or update test-observable card text metrics, card bounds, text bounds, and overlap checks for Shop/Auction UI surfaces.
- Preserve existing Shop/Auction UI behavior: DRAFT_INITIAL activation, purchase, bought overlay, Ready/Retract Ready, DRAFT_SHOP slots, refresh, purchase, DRAFT_AUCTION preparing, bid controls, affordability, in-flight feedback, accepted/rejected responses, settlement, and panel transitions.
- Produce the Shop/Auction UI implementation evidence note listed in `## Test Evidence`. Final browser/WASM cross-surface evidence remains owned by Presentation Layer Story 005.

### Out of Scope

- No Hand UI card fan surfaces.
- No PLACEMENT hand/fan active card, drag sprite, or staged fan ghost remediation.
- No final cross-surface browser/WASM capture package. Presentation Layer Story 005 owns that evidence pass.
- No auction bid button target-size or focus remediation. Story 011 already owns A11Y-ST-12 and must remain preserved.
- No DRAFT_INITIAL objective overlay implementation. Story 012 already owns A11Y-ST-18 and must remain preserved.
- No changes to authoritative card catalog fields, card costs, ATK, HP, keywords, rarity, effect text, or pool distribution.
- No changes to economy, auction bidding, purchase validation, hand ownership, phase timing, network protocol payloads, or server authority.
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

- [ ] **DRAFT_INITIAL grid cost floor passes**: GIVEN the Shop/Auction UI DRAFT_INITIAL grid renders all 9 offering cards, WHEN grid card text metrics are exported, THEN every visible cost numeral or badge text measures at least 18 CSS px or browser-equivalent rendered pixels.
- [ ] **DRAFT_INITIAL grid stat and keyword floors pass**: GIVEN the DRAFT_INITIAL fixture includes minion, structure, spell or instant, zero-cost, long-name, no-keyword, and multi-keyword cards, WHEN grid card text metrics are exported, THEN visible ATK and HP fields measure at least 18 CSS px, visible keyword text measures at least 14 CSS px, and absent fields are recorded as not applicable.
- [ ] **Purchased-slot overlay preserves readability**: GIVEN a DRAFT_INITIAL slot has a confirmed bought overlay, WHEN bounds checks run, THEN the overlay does not clip or overlap visible cost, ATK, HP, or accepted keyword text into unreadability.
- [ ] **DRAFT_SHOP slot floors pass**: GIVEN DRAFT_SHOP renders three shop cards, WHEN shop card text metrics are exported, THEN visible cost, ATK, HP, and accepted keyword text meet their required floors and remain inside card or badge bounds.
- [ ] **Read-only auction footer floors pass**: GIVEN DRAFT_AUCTION renders locked shop footer cards at 30 percent opacity, WHEN footer card metrics are exported, THEN cost, ATK, HP, and accepted keyword text that remain intended-readable meet their floors and do not overlap footer locks or adjacent controls.
- [ ] **Auction featured card floors pass**: GIVEN the auction featured card is rendered in preparing and active states, WHEN auction card text metrics are exported, THEN visible cost, ATK, HP, and accepted keyword text meet their required floors.
- [ ] **Settlement card readability is preserved**: GIVEN auction settlement renders the card movement or settled card view with outcome overlay adjacency, WHEN bounds checks run, THEN outcome overlays do not clip, cover, or overlap required card text into unreadability.
- [ ] **Accepted readable state is explicit**: GIVEN a shop or auction card surface hides keyword text at rest because a hover or zoom view is the accepted readable state, WHEN metrics are exported, THEN the record identifies hover or zoom as the accepted readable state and records the rest state as not presenting keyword text.
- [ ] **Not-applicable fields are explicit**: GIVEN a card type has no ATK, no HP, or no visible keyword text in a Shop/Auction UI state, WHEN the Shop/Auction UI metric table is generated, THEN that field is recorded as `N/A - field not present on this card type or state`.
- [ ] **No card-internal overlap**: GIVEN Shop/Auction UI text bounds and card bounds are exported, WHEN overlap checks run, THEN cost, ATK, HP, keyword text, card name, rarity indicator, and visible card art zones do not overlap in a way that makes required text unreadable.
- [ ] **No panel-level overlap**: GIVEN DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, and settlement card bounds are exported, WHEN overlap checks run, THEN required card text does not overlap sibling cards, shop controls, refresh, Ready/Retract Ready, bid controls, timer, objective overlay retrieval, outcome overlays, HUD chips, hand tray, or evidence overlays.
- [ ] **Existing Shop/Auction behavior is preserved**: GIVEN the Shop/Auction accessibility changes are present, WHEN the existing Shop/Auction UI regression commands listed in `## Test Evidence` run, THEN DRAFT_INITIAL activation, purchase, bought overlay, objective overlay, Ready/Retract Ready, DRAFT_SHOP slot and refresh, DRAFT_AUCTION preparing, bid, feedback, settlement, and transition behavior remains unchanged.
- [ ] **Focused Shop/Auction accessibility test passes**: `cargo test -p client --test shop_auction_ui_card_text_stat_keyword_accessibility_test` passes. The target must be backed by `tests/integration/shop_auction_ui/card_text_stat_keyword_accessibility_test.rs` and registered as `shop_auction_ui_card_text_stat_keyword_accessibility_test`.
- [ ] **Shop/Auction implementation evidence exists**: `production/qa/evidence/shop-auction-ui-card-text-stat-keyword-accessibility-2026-05-06.md` records the changed constants or components, fixture cards, automated text-size table, overlap table, regression command output summary, and Shop/Auction surface readiness statement for Presentation Layer Story 005.
- [ ] **A11Y-ST-02 impact is explicit**: The Shop/Auction UI evidence note states that this story implements the Shop/Auction UI slice of A11Y-ST-02 only, and that final cross-surface browser/WASM evidence remains owned by Presentation Layer Story 005.
- [ ] **QA-COND-0005 remains open**: The Shop/Auction UI evidence note states that QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, dependency-blocked, or accepted as risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep work local to Shop/Auction UI card rendering, panel card typography constants, card slot dimensions, badge sizing, hover or zoom surfaces, test fixtures, and evidence-only measurement hooks.
- Prefer adjusting existing typography constants, badge dimensions, slot layout constraints, hover or zoom presentation, and test-observable measurement components over changing card data or game logic.
- Preserve DRAFT_AUCTION footer opacity at the GDD-specified 30 percent value. If footer readability fails, adjust text treatment or accepted readable state without silently changing the read-only footer contract.
- Preserve Story 011 bid target and focus behavior. Card text changes must not alter bid button dimensions, focus order, one-send semantics, in-flight disable, or `BIDDING...` feedback.
- Preserve Story 012 DRAFT_INITIAL objective overlay behavior. Card text changes must not cover the objective overlay retrieval affordance or cause objective overlay clicks to target cards incorrectly.
- If keyword text wraps in hover or zoom, the wrapped block must meet the 14 CSS px floor, remain inside the card surface, and avoid overlap with stats or card controls.
- Do not add fake ATK, HP, or keyword text to cards that do not have those fields. Record those fields as not applicable.
- Keep any measurement overlay, metric export, or test marker out of normal shipping UI unless a later UX story promotes it.

## Performance Budget

No gameplay-loop performance impact is expected from typography adjustments or evidence-only measurement instrumentation. Any production remediation must preserve the ADR-021 presentation guardrails: steady-state presentation work remains below 1 ms per frame and phase-boundary spikes remain below 3 ms. The implementation must not add per-frame entity creation, extra Lightyear message drains, card catalog scans, texture uploads, or persistent debug overlays.

---

## QA Test Cases

- **DRAFT_INITIAL card text measurement**
  - Given: The DRAFT_INITIAL panel renders a 3 x 3 fixture.
  - When: text metrics are exported.
  - Then: every visible cost, ATK, and HP text field is at least 18 CSS px, every accepted keyword field is at least 14 CSS px, and absent fields are explicitly recorded as not applicable.

- **DRAFT_SHOP and footer card measurement**
  - Given: DRAFT_SHOP slots and DRAFT_AUCTION read-only footer slots render fixture cards.
  - When: text metrics and bounds are exported.
  - Then: visible cost, ATK, HP, and accepted keyword fields meet their floors and remain readable through normal and read-only treatments.

- **Auction card measurement**
  - Given: The auction featured card renders in preparing, active, and settlement-adjacent states.
  - When: metrics and bounds are exported.
  - Then: visible cost, ATK, HP, and accepted keyword fields meet their floors in every state where the field is intended to be readable.

- **Overlap guard**
  - Given: DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, and settlement card bounds are captured.
  - When: overlap checks compare required card text with sibling cards, shop controls, auction controls, overlays, HUD, and hand tray bounds.
  - Then: no required Shop/Auction card text overlaps another element in a way that makes it unreadable.

- **Behavior preservation**
  - Given: Shop/Auction UI card readability changes are present.
  - When: the Shop/Auction UI regression suite runs.
  - Then: draft offering, shop slots, auction activation, bid, feedback, settlement, objective overlay, target focus, and transition behavior remains unchanged.

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/shop_auction_ui/card_text_stat_keyword_accessibility_test.rs`
  - Registered as `shop_auction_ui_card_text_stat_keyword_accessibility_test`
  - Command: `cargo test -p client --test shop_auction_ui_card_text_stat_keyword_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test`
- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
- `cargo test -p client --test shop_auction_ui_shop_panel_test`
- `cargo test -p client --test shop_auction_ui_auction_activation_test`
- `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`
- `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- `cargo test -p client --test shop_auction_ui_auction_feedback_test`
- `cargo check -p client`
- `git diff --check`

**Required implementation evidence document**:

- `production/qa/evidence/shop-auction-ui-card-text-stat-keyword-accessibility-2026-05-06.md`

**Required evidence contents**:

- Commit, fixture card list, Shop/Auction UI surfaces covered, changed typography constants or components, and UI scale assumption.
- Text-size table for cost, ATK, HP, and keyword text by Shop/Auction UI surface.
- Explicit not-applicable entries for fields absent from a card type or state.
- Bounds and overlap table for DRAFT_INITIAL grid, DRAFT_INITIAL bought overlay, DRAFT_SHOP slots, DRAFT_AUCTION read-only footer, auction featured card, and settlement card view.
- Regression command output summary.
- Shop/Auction UI readiness statement for Presentation Layer Story 005.
- A11Y-ST-02 impact statement.
- QA-COND-0005 impact statement confirming the condition remains Open.

**Final browser/WASM evidence owner**:

- Presentation Layer Story 005 captures the final cross-surface browser/WASM evidence at `production/qa/evidence/presentation-card-text-accessibility.md`.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md) - Complete; provides panel roots and pure formula scaffolding.
- Depends on: [Story 002](story-002-draft-initial-grid-purchase-ready.md) - Complete; provides DRAFT_INITIAL grid, purchase, Ready/Retract Ready, and panel dismissal behavior.
- Depends on: [Story 003](story-003-shop-panel-slots-refresh-purchase-ready.md) - Complete; provides DRAFT_SHOP slot and refresh behavior.
- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md) - Complete; provides auction preparing and active panel card ownership.
- Depends on: [Story 007](story-007-auction-settlement-and-shop-transition.md) - Ready; provides settlement and shop transition behavior that this story must preserve when implemented.
- Depends on: [Story 011](story-011-auction-bid-target-size-and-focus-evidence.md) - Complete; provides A11Y-ST-12 bid target and focus behavior that must be preserved.
- Depends on: [Story 012](story-012-draft-initial-clear-objective-overlay.md) - Complete; provides A11Y-ST-18 objective overlay behavior that must be preserved.
- Depends on: ADR-002, ADR-013, ADR-015, ADR-019, and ADR-021 Accepted.
- Unlocks: Shop/Auction UI slice for Presentation Layer Story 005 final A11Y-ST-02 browser/WASM evidence.

## Blockers

None.

## No Open Questions

None.
