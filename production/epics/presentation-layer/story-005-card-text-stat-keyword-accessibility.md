# Story 005: Card Text, Stat, and Keyword Accessibility Evidence

> **Epic**: Presentation Layer
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-04 / QA-COND-0005

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility evidence register row `A11Y-ST-02` states that card cost, ATK, HP, and keyword text do not yet have browser/WASM measurement evidence. This story is the final cross-surface evidence pass after the owner implementation slices in Hand UI Story 015 and Shop/Auction UI Story 013.

**Primary sources**:

- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
- `design/accessibility-requirements.md`
- `design/gdd/card-data-pool.md`
- `design/gdd/hand-ui.md`
- `design/gdd/shop-auction-ui.md`
- `design/ux/hand-ui.md`
- `design/ux/shop-auction-ui.md`
- `design/ux/interaction-patterns.md`

**Accessibility requirement**:

- `design/accessibility-requirements.md` Standard-tier row `Minimum text size - card text (cost, ATK, HP, keyword)` requires stat badges for ATK and HP to have an 18 px minimum floor and keyword text to have a 14 px minimum floor. This story applies the same 18 CSS px floor to visible card cost numerals because `A11Y-ST-02` explicitly includes card cost in the evidence gap.

**GDD and UX trace**:

- `design/gdd/card-data-pool.md` card definition schema exposes display fields including `cost`, `atk`, `hp`, `keywords`, and `effect_text`.
- `design/gdd/hand-ui.md` Rule 1 requires pre-pooled hand fan slots and Rule 3 requires hand/fan visibility across DRAFT_SHOP, DRAFT_AUCTION, and PLACEMENT.
- `design/gdd/hand-ui.md` VA-1 defines hand card cost, ATK, HP, type, rarity, and hover zoom anatomy.
- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 2 requires a 3 x 3 card grid sorted by rarity and cost.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 1 requires the auction panel to render the auction card art, rarity badge, and current price.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 2 requires read-only shop footer card costs to remain visible at 30 percent opacity.
- `design/gdd/shop-auction-ui.md` DRAFT_SHOP Rule 2 requires three horizontal shop slots, each showing card art, name, rarity badge, and cost.
- `design/ux/hand-ui.md` requires card cost, ATK, HP, and type or rarity to remain readable at 10-card hand compression.
- `design/ux/shop-auction-ui.md` requires card keyword text to remain readable on hover or zoom, body text contrast of at least 4.5:1 where card text is sampled against its rendered background, and no required UI overlap at `1366x768` and `1920x1080`.

**TR IDs**:

- `TR-CDP-010` for draft, shop, and auction card payloads arriving before client phase/UI use.
- `TR-HU-001` for pre-pooled hand fan card slots.
- `TR-HU-005` for DRAFT_INITIAL card acquisition flow into the hand fan.
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

**ADR Decision Summary**: Final A11Y-ST-02 evidence is presentation-only. The evidence pass verifies rendered client presentation across Hand UI and Shop/Auction UI without mutating authoritative card data, card ownership, economy, auction, phase, or protocol state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file. The final evidence pass may add test-only browser/WASM capture hooks, measurement exports, or capture scripts, but normal production UI must continue to use Bevy 0.18 Required Components API. Do not use `NodeBundle`, `TextBundle`, `SpriteBundle`, `UiImage::new()`, `Parent`, or `Color::rgba()`.

**Control Manifest Rules (2026-05-05)**:

- Required: client presentation is a read-only view of server-authoritative state.
- Required: presentation work runs in the ADR-021 order `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: `PresentationPlugin` registration order remains Card Animations, Board Rendering, Hand UI, HUD, Shop/Auction UI.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: economy presentation reads `PlayerEconomyView` rather than adding new independent economy message drains.
- Required: UI overlays such as HUD, hand fan, shop panels, and auction bid box use `bevy_ui`; board content remains world-space.
- Forbidden: client presentation must not assert or mutate authoritative game state.
- Guardrail: presentation steady-state stays below 1 ms per frame and phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Capture final browser/WASM evidence for A11Y-ST-02 across all required card surfaces:
  - DRAFT_INITIAL 3 x 3 grid cards.
  - Hand/fan cards at rest in DRAFT_SHOP.
  - Hand/fan cards in DRAFT_AUCTION read-only state.
  - Hand/fan cards in PLACEMENT active and staged ghost states.
  - DRAFT_SHOP shop cards.
  - DRAFT_AUCTION read-only shop footer cards.
  - DRAFT_AUCTION auction featured card in preparing and active states.
  - Auction settlement card view.
- Measure browser/WASM output at `1366x768` and `1920x1080`.
- Verify cost, ATK, and HP text floors of at least 18 CSS px or browser-equivalent rendered pixels.
- Verify keyword text floor of at least 14 CSS px or browser-equivalent rendered pixels for every accepted readable keyword state.
- Record explicit not-applicable entries for card types or states that do not have ATK, HP, or visible keyword text.
- Verify that card text does not clip, truncate into unreadability, or overlap card-internal, fan-level, panel-level, HUD, hand tray, shop control, auction control, timer, tooltip, overlay, or evidence overlay bounds.
- Verify sampled browser/WASM foreground and composited background color pairs for visible cost, ATK, HP, and keyword text meet at least 4.5:1 contrast.
- Consolidate Hand UI Story 015 and Shop/Auction UI Story 013 implementation evidence into the final A11Y-ST-02 evidence document.
- State whether A11Y-ST-02 is implemented and evidenced, or identify the exact measured surface and field that still fails.

### Out of Scope

- No Hand UI typography, badge, fan, hover, zoom, drag sprite, staged ghost, or reserve strip remediation in this story. Hand UI Story 015 owns those implementation changes.
- No Shop/Auction UI typography, badge, grid, shop slot, footer, auction card, or settlement card remediation in this story. Shop/Auction UI Story 013 owns those implementation changes.
- No new game logic, protocol payload, card catalog, card stat, keyword, price, ownership, economy, bid, purchase, placement, phase, or server-authority change.
- No broader UI scaling preference implementation.
- No colorblind palette implementation.
- No Settings or Accessibility screen work.
- No full A11Y-ST-01 HUD text evidence or A11Y-ST-03 global contrast evidence beyond the card text pairs sampled for A11Y-ST-02.
- No changes to `production/sprint-status.yaml`.
- No changes to `production/session-state/**`.
- No changes to project asset files.
- No changes to `AGENTS.md`.
- Do not close QA-COND-0005 from this story alone.

---

## Acceptance Criteria

- [ ] **Owner implementation evidence is present**: GIVEN Hand UI Story 015 and Shop/Auction UI Story 013 have completed their implementation evidence notes, WHEN this evidence story starts, THEN both owner evidence notes are referenced in `production/qa/evidence/presentation-card-text-accessibility.md`.
- [ ] **DRAFT_INITIAL grid evidence exists**: GIVEN browser/WASM evidence runs at `1366x768` and `1920x1080`, WHEN the DRAFT_INITIAL 3 x 3 grid is rendered with fixture cards, THEN the evidence records cost, ATK, HP, keyword text, card bounds, and text bounds for every visible grid card field that applies.
- [ ] **Hand/fan evidence exists**: GIVEN browser/WASM evidence runs at both required viewports, WHEN the hand/fan is rendered with 10 cards in DRAFT_SHOP, DRAFT_AUCTION read-only, PLACEMENT active, and PLACEMENT staged ghost states, THEN the evidence records cost, ATK, HP, keyword text, card bounds, and text bounds for each visible fan card field that applies.
- [ ] **Shop card evidence exists**: GIVEN browser/WASM evidence runs at both required viewports, WHEN DRAFT_SHOP shows three shop cards and DRAFT_AUCTION read-only footer shows three locked shop cards at 30 percent opacity, THEN the evidence records cost, ATK, HP, keyword text, card bounds, and text bounds for every visible shop/footer field that applies.
- [ ] **Auction card evidence exists**: GIVEN browser/WASM evidence runs at both required viewports, WHEN the auction featured card is shown in preparing, active, and settlement states, THEN the evidence records cost, ATK, HP, keyword text, card bounds, and text bounds for every visible auction-card field that applies.
- [ ] **Cost floor passes**: GIVEN any visible card cost numeral or badge in the measured surfaces, WHEN text-size measurements are reviewed, THEN the cost text measures at least 18 CSS px or browser-equivalent rendered pixels at both viewports.
- [ ] **ATK floor passes**: GIVEN any visible card ATK numeral or badge in the measured surfaces, WHEN text-size measurements are reviewed, THEN the ATK text measures at least 18 CSS px or browser-equivalent rendered pixels at both viewports.
- [ ] **HP floor passes**: GIVEN any visible card HP numeral or badge in the measured surfaces, WHEN text-size measurements are reviewed, THEN the HP text measures at least 18 CSS px or browser-equivalent rendered pixels at both viewports.
- [ ] **Keyword floor passes**: GIVEN any visible keyword text in the measured surfaces, WHEN text-size measurements are reviewed, THEN the keyword text measures at least 14 CSS px or browser-equivalent rendered pixels at both viewports.
- [ ] **Accepted readable states are explicit**: GIVEN a card surface hides keyword text at rest and exposes it through hover or zoom, WHEN the evidence table is reviewed, THEN the rest state and hover or zoom accepted readable state are both recorded.
- [ ] **Not-applicable fields are explicit**: GIVEN a fixture card has no ATK, no HP, or no visible keyword text, WHEN the evidence table is reviewed, THEN that field is marked `N/A - field not present on this card type or state` rather than being silently omitted.
- [ ] **Long and dense card text remains readable**: GIVEN fixture cards include at least one long card name, at least one two-keyword card, at least one zero-cost card, and at least one card with two-digit cost, ATK, or HP, WHEN browser/WASM captures are reviewed, THEN visible cost, ATK, HP, and keyword text remains inside its card or badge bounds without clipping or unreadable truncation.
- [ ] **No card-internal overlap**: GIVEN the measured text bounds and card bounds, WHEN overlap checks run, THEN cost, ATK, HP, keyword text, card name, rarity indicator, and visible card art zones do not overlap in a way that makes required text unreadable.
- [ ] **No surface-level overlap**: GIVEN the measured card and UI bounds, WHEN overlap checks run, THEN card text does not overlap sibling cards, fan ghosts, reserve split controls, shop controls, auction controls, Ready controls, timers, tooltips, HUD chips, settlement overlays, objective overlays, or browser evidence overlays.
- [ ] **Contrast sample passes for card text**: GIVEN sampled browser/WASM foreground and composited background colors for cost, ATK, HP, and keyword text, WHEN contrast ratios are computed, THEN each sampled text/background pair meets at least 4.5:1.
- [ ] **Existing behavior is preserved**: GIVEN existing Hand UI and Shop/Auction UI regression commands run after the owner implementation stories, WHEN final evidence capture is performed, THEN purchase, refresh, read-only footer, auction preparing, auction active, settlement, hand/fan visibility, fan staged ghost, and input suppression behavior remains unchanged.
- [ ] **Focused accessibility test passes**: `cargo test -p client --test card_text_accessibility_test` passes. The target must be backed by `tests/integration/presentation/card_text_accessibility_test.rs` and registered as `card_text_accessibility_test`.
- [ ] **Browser/WASM evidence document exists**: `production/qa/evidence/presentation-card-text-accessibility.md` records the browser, build target, commit, capture command, fixture cards, viewport table, text-size table, overlap table, contrast sample table, screenshot capture directory, pass/fail verdict, and QA-COND-0005 impact statement.
- [ ] **Capture directory is populated**: `production/qa/evidence/captures/presentation-card-text-accessibility/` contains the browser/WASM captures referenced by the evidence document for both required viewports and every required card surface.
- [ ] **A11Y-ST-02 impact is explicit**: The evidence document states whether A11Y-ST-02 is implemented and evidenced by this story, or which measured surface still fails the cost, ATK, HP, keyword, non-overlap, or readability checks.
- [ ] **QA-COND-0005 remains open**: The evidence document states that this story contributes only the A11Y-ST-02 card text accessibility row and that QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, dependency-blocked, or accepted as risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Treat this as a final evidence and integration-verification story. Production typography and card layout remediation belongs in the owner implementation stories.
- If final browser/WASM evidence finds a failing owner surface, record the failing surface and return to the owning story area for repair rather than silently expanding this story into broad remediation.
- Use fixture cards that cover minion, structure, spell or instant, zero-cost, two-digit stat, long-name, no-keyword, and multi-keyword cases.
- Capture both required viewports for every surface. Evidence should identify the browser, build target, commit, capture command, fixture source, viewport, and UI scale.
- If a card surface intentionally hides text at rest and shows it only on hover or zoom, the evidence must show both the hidden/rest state and the readable hover or zoom state.
- If a card type has no ATK or HP, do not add fake stat text to satisfy the measurement table. Record the field as not applicable.
- If browser metrics are exported through a debug overlay, measurement log, or test-only component, keep that instrumentation out of normal shipping UI.
- Do not reduce DRAFT_AUCTION footer opacity below the GDD-specified value as an evidence workaround.

## Performance Budget

No measurable gameplay-loop performance impact is expected from final evidence capture or test-only measurement instrumentation. Any production-facing measurement hook must preserve the ADR-021 presentation guardrails: steady-state presentation work remains below 1 ms per frame and phase-boundary spikes remain below 3 ms. The implementation must not add per-frame entity creation, extra Lightyear message drains, card catalog scans, texture uploads, or persistent debug overlays.

---

## QA Test Cases

- **DRAFT_INITIAL browser evidence**
  - Given: Browser/WASM DRAFT_INITIAL renders a 3 x 3 fixture at `1366x768` and `1920x1080`.
  - When: text metrics are exported and screenshots are captured.
  - Then: visible cost, ATK, HP, and accepted keyword fields meet their floors, absent fields are recorded, and no required text overlaps grid controls or overlays.

- **Hand/fan browser evidence**
  - Given: The hand/fan renders 10 cards in DRAFT_SHOP, DRAFT_AUCTION read-only, PLACEMENT active, and PLACEMENT staged ghost states.
  - When: text metrics and bounds are exported.
  - Then: visible cost, ATK, HP, and accepted keyword fields meet their floors and remain readable despite fan overlap, dimming, hover, zoom, and ghost treatments.

- **Shop and auction browser evidence**
  - Given: DRAFT_SHOP slots, DRAFT_AUCTION footer slots, auction preparing, active auction card, and settlement card views are rendered.
  - When: text metrics and bounds are exported.
  - Then: visible cost, ATK, HP, and accepted keyword fields meet their floors in every state where the field is intended to be readable.

- **Overlap and contrast guard**
  - Given: Browser/WASM text bounds, card bounds, adjacent UI bounds, and foreground/background color samples are captured for every required surface.
  - When: overlap checks and contrast calculations run.
  - Then: no required card text overlaps another element in a way that makes it unreadable and sampled text/background pairs meet at least 4.5:1.

- **Behavior preservation**
  - Given: final card text evidence capture hooks are present.
  - When: existing Hand UI and Shop/Auction UI regression commands run.
  - Then: purchase, refresh, auction, settlement, hand/fan visibility, staged ghost, and input gating behavior remains unchanged.

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/presentation/card_text_accessibility_test.rs`
  - Registered as `card_text_accessibility_test`
  - Command: `cargo test -p client --test card_text_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test hand_ui_card_text_stat_keyword_accessibility_test`
- `cargo test -p client --test shop_auction_ui_card_text_stat_keyword_accessibility_test`
- `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
- `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
- `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- `cargo check -p client`
- `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/presentation-card-text-accessibility.md`

**Required browser/WASM capture artifact directory**:

- `production/qa/evidence/captures/presentation-card-text-accessibility/`

**Required browser/WASM evidence contents**:

- Browser, build target, commit, capture command, fixture source, and UI scale.
- Viewports: `1366x768` and `1920x1080`.
- Fixture cards covering minion, structure, spell or instant, zero-cost, two-digit stat, long-name, no-keyword, and multi-keyword cases.
- Owner implementation evidence references for Hand UI Story 015 and Shop/Auction UI Story 013.
- Surface table covering DRAFT_INITIAL grid cards, hand/fan cards, DRAFT_SHOP shop cards, DRAFT_AUCTION footer cards, and auction featured card views.
- Text-size table for cost, ATK, HP, and keyword text by surface and viewport.
- Explicit not-applicable entries for fields absent from a card type or state.
- Overlap table comparing text bounds, card bounds, and adjacent UI bounds.
- Contrast sample table for visible cost, ATK, HP, and keyword text.
- Browser/WASM capture links under the required capture directory.
- Focused automated test command output summary.
- Regression command output summary.
- A11Y-ST-02 impact statement.
- QA-COND-0005 impact statement confirming the condition remains Open.

**QA-COND-0005 impact statement required in evidence**:

Story 005 implements and evidences A11Y-ST-02 for card cost, ATK, HP, and keyword text floors across DRAFT_INITIAL grid, hand/fan, shop, and auction card views. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, dependency-blocked, or accepted as risk.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Hand UI Story 015](../hand-ui/story-015-card-text-stat-keyword-accessibility.md) - Ready; provides the Hand UI implementation slice before this final evidence pass executes.
- Depends on: [Shop/Auction UI Story 013](../shop-auction-ui/story-013-card-text-stat-keyword-accessibility.md) - Ready; provides the Shop/Auction UI implementation slice before this final evidence pass executes.
- Depends on: `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`, which identifies A11Y-ST-02 as an evidence-only required row blocking QA-COND-0005 closure.
- Depends on: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`, which remains Open and defines the closure guard.
- Depends on: ADR-002, ADR-013, ADR-015, ADR-019, and ADR-021 Accepted.
- Unlocks: A11Y-ST-02 can move from evidence-only required to implemented and evidenced after this story is implemented and `production/qa/evidence/presentation-card-text-accessibility.md` passes QA review. This does not unlock QA-COND-0005 closure by itself.

## Blockers

None.

## No Open Questions

None.
