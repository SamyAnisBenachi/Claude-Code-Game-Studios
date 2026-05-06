# Story 013: Card Text, Stat, and Keyword Accessibility

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-04 / QA-COND-0005

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier
accessibility remediation.

**QA condition**:
`production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
remains Open. The Sprint 6 accessibility evidence register row `A11Y-ST-02`
states that there is no browser/WASM measurement evidence for card cost, ATK,
HP, or keyword text floors. This story owns the Shop/Auction UI portion of
that row.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`, `design/ux/interaction-patterns.md`
**Accessibility Source**: `design/accessibility-requirements.md`
**Requirement**: `TR-CDP-010`, `TR-SAU-003`, `TR-SAU-004`, `TR-SAU-006`,
A11Y-ST-02

**ADR Governing Implementation**:
[ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md),
[ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md),
[ADR-019: Economy System Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md),
[ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md),
[ADR-002: Client-Server Authority Model](../../../docs/architecture/adr-002-client-server-authority.md)

**Control Manifest**: `docs/architecture/control-manifest.md` version
`2026-05-05`.

**GDD trace**:

- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 2 requires a 3x3 card
  grid sorted by rarity and cost.
- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL visual treatment defines card
  display size, hover zoom, purchased-slot treatment, and stable slot
  positioning.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 1 requires the auction
  panel to render auction card art, rarity badge, and starting price/current
  price after `S2CAuctionCard` and phase data are available.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 2 requires read-only shop
  footer card costs to remain visible at 30 percent opacity during auction.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION settlement rules require local
  win, opponent win, and no-bid settlement states to preserve terminal auction
  presentation and transition to DRAFT_SHOP.
- `design/gdd/shop-auction-ui.md` DRAFT_SHOP Rule 2 requires three horizontal
  shop slots, each showing card art, name, rarity badge, and cost.
- `design/gdd/shop-auction-ui.md` acceptance criteria SAU-DI6, SAU-DI10,
  SAU-DA1, SAU-DA3, SAU-EG4c, SAU-EG6a, SAU-V5, SAU-DS1, SAU-DS4, SAU-DS9,
  SAU-DS10, SAU-SET1a, and SAU-SET2 define the Shop/Auction UI states whose
  behavior must be preserved while typography, badge, or measurement
  instrumentation is repaired.

**UX and accessibility trace**:

- `design/accessibility-requirements.md` Standard-tier row `Minimum text size -
  card text (cost, ATK, HP, keyword)` requires ATK and HP stat badges to have
  an 18px minimum floor and keyword text to have a 14px minimum floor.
- A11Y-ST-02 explicitly includes card cost, so this story applies the same
  18 CSS px floor to visible Shop/Auction UI card cost numerals or badges.
- `design/ux/shop-auction-ui.md` requires DRAFT_INITIAL cards, DRAFT_SHOP
  cards, DRAFT_AUCTION featured cards, and DRAFT_AUCTION read-only footer cards
  to show card identity and cost without overlapping HUD, hand tray, controls,
  timers, or panel boundaries.
- `design/ux/shop-auction-ui.md` Accessibility requires body text contrast of
  at least `4.5:1`, and requires card keyword text to remain readable on hover
  or zoom.
- `design/ux/interaction-patterns.md` PTN-INP-005 defines purchasable card
  presentation as card art, name, rarity, cost, and purchase state in a stable
  slot.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM evidence | **Risk**: HIGH

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and
`liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:

- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin` sub-plugin after
  Card Animations, Board Rendering, Hand UI, and HUD.
- Shop/Auction UI reads phase through `Res<CurrentClientPhase>` only. It must
  not drain `MessageReceiver<S2CPhaseChanged>`.
- Shop/Auction UI reads `Res<PlayerEconomyView>` for local own-gold and
  affordability state. It must not drain economy S2C messages directly.
- DRAFT_INITIAL, DRAFT_AUCTION, DRAFT_SHOP, footer, settlement, and
  evidence-only overlays remain bevy_ui presentation surfaces.
- UI work runs in ADR-021 `PresentationSet` order and remains session-scoped
  under `ClientState::InSession`.
- Client-side card readability remediation is presentation-only. It must not
  mutate authoritative card catalog, shop slots, auction card, price, leader,
  gold, reservation, card ownership, phase, or protocol state.

---

## Scope

### In Scope

- Verify or remediate A11Y-ST-02 card text floors for Shop/Auction UI-owned
  card surfaces:
  - DRAFT_INITIAL 3x3 offering grid cards, including available, pending,
    purchased, locked, hand-full, and hover or zoom states.
  - DRAFT_SHOP shop slot cards, including available, pending, purchased,
    empty/dead, hand-full, and hover or zoom states.
  - DRAFT_AUCTION featured card in preparing and active states.
  - DRAFT_AUCTION read-only shop footer cards at the specified 30 percent
    opacity.
  - DRAFT_AUCTION settlement card views for local win, opponent win, and no-bid
    terminal states once settlement rendering is available.
- Measure visible cost, ATK, HP, and keyword text at browser/WASM viewports
  `1366x768` and `1920x1080`.
- Apply a minimum floor of `18 CSS px` or browser-equivalent rendered pixels to
  visible cost, ATK, and HP numerals or badge text.
- Apply a minimum floor of `14 CSS px` or browser-equivalent rendered pixels to
  visible keyword text.
- Record explicit not-applicable entries for card types or states that do not
  have ATK, HP, or visible keyword text.
- Verify that Shop/Auction UI card text does not clip, truncate into
  unreadability, or overlap another required card element, sibling card, HUD
  zone, hand tray, panel header, timer, Ready control, refresh control, auction
  bid control, settlement overlay, tooltip, or evidence overlay.
- Verify body text contrast for sampled Shop/Auction UI keyword text and stat
  numerals against their final browser/WASM composited backgrounds. Sampled
  pairs must meet at least `4.5:1`.
- Add or update focused automated coverage that exposes Shop/Auction UI card
  text metrics, card bounds, and overlap checks without relying only on manual
  visual judgment.
- Capture Shop/Auction UI browser/WASM evidence and update the exact evidence
  document listed in `## Test Evidence`.
- Preserve existing DRAFT_INITIAL purchase, Ready/Retract Ready, objective
  overlay, DRAFT_SHOP purchase, refresh, hand-full, DRAFT_AUCTION preparing,
  bid target/focus, in-flight, read-only footer, settlement, and transition
  semantics.

### Out of Scope

- Hand fan cards, PLACEMENT fan cards, staged fan ghosts, and Instant fan
  ghosts. Those surfaces are owned by Hand UI Story 015.
- Final cross-surface A11Y-ST-02 browser/WASM evidence aggregation. That is
  owned by Presentation Layer Story 005 after the split implementation stories
  are complete.
- Changes to authoritative card catalog fields, card costs, ATK, HP, keywords,
  rarity, effect text, pool distribution, shop slot contents, auction bidding,
  purchase validation, refresh validation, gold, reservation, hand ownership,
  phase timing, network protocol payloads, or server authority.
- Broader UI scaling preferences, Settings or Accessibility screen work,
  colorblind palette implementation, full A11Y-ST-01 HUD text evidence, or
  A11Y-ST-03 global contrast evidence beyond sampled Shop/Auction UI card text
  pairs.
- Changes to `production/sprint-status.yaml`, `production/session-state/**`,
  project asset files, or `AGENTS.md`.
- QA-COND-0005 closure. This story contributes only the Shop/Auction UI slice
  of A11Y-ST-02.

---

## Acceptance Criteria

- [ ] **DRAFT_INITIAL grid metrics exist**: GIVEN browser/WASM evidence runs at
  `1366x768` and `1920x1080`, WHEN the DRAFT_INITIAL 3x3 offering grid renders
  with available, pending, purchased, locked, hand-full, and hover or zoom
  states, THEN the evidence records visible cost, ATK, HP, keyword text, card
  bounds, and text bounds for every field that applies.
- [ ] **DRAFT_SHOP slot metrics exist**: GIVEN browser/WASM evidence runs at
  both required viewports, WHEN DRAFT_SHOP renders three shop slot cards in
  available, pending, purchased, empty/dead, hand-full, and hover or zoom
  states, THEN the evidence records visible cost, ATK, HP, keyword text, card
  bounds, and text bounds for every field that applies.
- [ ] **DRAFT_AUCTION featured card metrics exist**: GIVEN browser/WASM
  evidence runs at both required viewports, WHEN the DRAFT_AUCTION featured
  card is shown in preparing and active states, THEN the evidence records
  visible cost, ATK, HP, keyword text, card bounds, and text bounds for every
  field that applies.
- [ ] **DRAFT_AUCTION footer metrics exist**: GIVEN browser/WASM evidence runs
  at both required viewports, WHEN the read-only shop footer shows three locked
  cards at 30 percent opacity, THEN the evidence records visible cost, ATK, HP,
  keyword text, card bounds, and text bounds for every field that applies.
- [ ] **Settlement card metrics exist**: GIVEN settlement card rendering is
  available from Story 007, WHEN local win, opponent win, and no-bid terminal
  states are captured, THEN the evidence records visible cost, ATK, HP,
  keyword text, card bounds, and text bounds for every settlement card field
  that applies.
- [ ] **Cost floor passes**: GIVEN any visible Shop/Auction UI card cost
  numeral or badge in the measured surfaces, WHEN text-size measurements are
  reviewed, THEN the cost text measures at least `18 CSS px` or
  browser-equivalent rendered pixels at both viewports.
- [ ] **ATK floor passes**: GIVEN any visible Shop/Auction UI card ATK numeral
  or badge in the measured surfaces, WHEN text-size measurements are reviewed,
  THEN the ATK text measures at least `18 CSS px` or browser-equivalent
  rendered pixels at both viewports.
- [ ] **HP floor passes**: GIVEN any visible Shop/Auction UI card HP numeral or
  badge in the measured surfaces, WHEN text-size measurements are reviewed,
  THEN the HP text measures at least `18 CSS px` or browser-equivalent rendered
  pixels at both viewports.
- [ ] **Keyword floor passes**: GIVEN any visible Shop/Auction UI keyword text
  in the measured surfaces, WHEN text-size measurements are reviewed, THEN the
  keyword text measures at least `14 CSS px` or browser-equivalent rendered
  pixels at both viewports.
- [ ] **Not-applicable fields are explicit**: GIVEN a fixture card has no ATK,
  no HP, or no visible keyword text in a Shop/Auction UI state, WHEN the
  evidence table is reviewed, THEN that field is marked `N/A - field not
  present on this card type or state` rather than being silently omitted.
- [ ] **Long and dense card text remains readable**: GIVEN fixture cards
  include at least one long card name, one two-keyword card, one zero-cost card,
  and one card with two-digit cost, ATK, or HP, WHEN browser/WASM captures are
  reviewed, THEN visible cost, ATK, HP, and keyword text remains inside its
  card or badge bounds without clipping or unreadable truncation.
- [ ] **No Shop/Auction card-internal overlap**: GIVEN measured text bounds and
  card bounds, WHEN overlap checks run, THEN cost, ATK, HP, keyword text, card
  name, rarity indicator, and visible card art zones do not overlap in a way
  that makes required text unreadable.
- [ ] **No Shop/Auction surface-level overlap**: GIVEN measured card and UI
  bounds, WHEN overlap checks run, THEN Shop/Auction UI card text does not
  overlap sibling cards, shop controls, auction controls, Ready controls,
  timers, tooltips, HUD chips, hand tray, settlement overlays, panel
  boundaries, or browser evidence overlays.
- [ ] **Contrast sample passes for Shop/Auction card text**: GIVEN sampled
  browser/WASM foreground and composited background colors for visible cost,
  ATK, HP, and keyword text, WHEN contrast ratios are computed, THEN each
  sampled text/background pair meets at least `4.5:1`.
- [ ] **Existing Shop/Auction UI behavior is preserved**: GIVEN existing
  Shop/Auction UI regression tests run after remediation, WHEN card typography
  or measurement instrumentation is applied, THEN DRAFT_INITIAL purchase,
  objective overlay, Ready/Retract Ready, DRAFT_SHOP purchase, refresh,
  hand-full, DRAFT_AUCTION preparing, bid target/focus, in-flight, read-only
  footer, settlement, and transition behavior remains unchanged.
- [ ] **Focused accessibility test passes**:
  `cargo test -p client --test shop_auction_ui_card_text_stat_keyword_accessibility_test`
  passes. The target must be backed by
  `tests/integration/shop_auction_ui/card_text_stat_keyword_accessibility_test.rs`.
- [ ] **Browser/WASM evidence exists**:
  `production/qa/evidence/shop-auction-ui-card-text-stat-keyword-accessibility.md`
  records browser, build target, commit, capture command, fixture cards,
  viewport table, text-size table, overlap table, contrast sample table,
  screenshot capture directory, pass/fail verdict, and QA-COND-0005 impact
  statement.
- [ ] **Capture directory is populated**:
  `production/qa/evidence/captures/shop-auction-ui-card-text-stat-keyword-accessibility/`
  contains the browser/WASM captures referenced by the evidence document for
  both required viewports and every required Shop/Auction UI card surface.
- [ ] **A11Y-ST-02 Shop/Auction impact is explicit**: The evidence document
  states whether the Shop/Auction UI portion of A11Y-ST-02 is implemented and
  evidenced, or which measured Shop/Auction UI surface still fails the cost,
  ATK, HP, keyword, non-overlap, or readability checks.
- [ ] **QA-COND-0005 remains open**: The evidence document states that this
  story contributes only the Shop/Auction UI slice of A11Y-ST-02 and that
  QA-COND-0005 remains Open until all remaining Standard-tier rows are
  implemented and evidenced, reclassified, dependency-blocked, or accepted as
  risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep work local to Shop/Auction UI card rendering, Shop/Auction UI test
  fixtures, and Shop/Auction UI evidence harnesses needed for A11Y-ST-02.
- Prefer adjusting existing typography constants, theme tokens, card layout
  slots, badge sizes, hover or zoom surfaces, footer treatment, settlement card
  treatment, or measurement markers over changing card data or game logic.
- If a card field is intentionally unreadable at rest and readable on hover or
  zoom, the evidence must show both states and state which state is accepted as
  the player-readable state for that field.
- If a card type has no ATK or HP, do not add fake stat text to satisfy the
  measurement table. Record the field as not applicable for that card type.
- If keyword text wraps, the wrapped line block must still meet the `14 CSS px`
  floor, remain inside the card surface, and avoid overlap with stats, card
  controls, panel controls, and sibling cards.
- Keep evidence-only measurement overlays, debug labels, exported bounds, or
  capture harnesses out of normal shipping UI unless they are also accepted
  player-facing accessibility affordances.
- Do not reduce DRAFT_AUCTION footer opacity below the GDD-specified 30 percent
  value as a readability workaround. If the 30 percent read-only footer fails,
  choose a remediation that preserves read-only state while meeting A11Y-ST-02.
- Do not move auction price counter remediation into card-text scope. The
  auction price counter is owned by separate HUD or Shop/Auction evidence for
  A11Y-ST-01 and A11Y-ST-03.

---

## QA Test Cases

- **DRAFT_INITIAL grid card text measurement**
  - Given: Browser/WASM DRAFT_INITIAL renders a 3x3 fixture at `1366x768` and
    `1920x1080`
  - When: text metrics are exported
  - Then: every visible cost, ATK, and HP field is at least `18 CSS px`, every
    visible keyword field is at least `14 CSS px`, and absent fields are
    explicitly recorded

- **DRAFT_SHOP card text measurement**
  - Given: Browser/WASM DRAFT_SHOP renders three shop slot cards with available,
    pending, purchased, empty/dead, and hand-full states
  - When: text metrics and bounds are exported
  - Then: visible cost, ATK, HP, and keyword fields meet their floors and remain
    readable without overlapping controls, HUD, hand tray, or panel boundaries

- **Auction card and footer measurement**
  - Given: DRAFT_AUCTION featured card, preparing card, active card, read-only
    footer cards, and settlement card states are rendered
  - When: text metrics and bounds are exported
  - Then: visible cost, ATK, HP, and keyword fields meet their floors in every
    state where the field is intended to be readable

- **Contrast sample guard**
  - Given: Browser/WASM foreground/background samples are captured for visible
    Shop/Auction UI cost, ATK, HP, and keyword text
  - When: contrast ratios are computed after compositing
  - Then: sampled pairs meet at least `4.5:1`

- **Behavior preservation**
  - Given: existing Shop/Auction UI regressions run after typography or
    measurement changes
  - When: DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, footer, settlement, and
    transition paths are exercised
  - Then: existing behavior remains unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/shop_auction_ui/card_text_stat_keyword_accessibility_test.rs`
  - Registered as `shop_auction_ui_card_text_stat_keyword_accessibility_test`
  - Command:
    `cargo test -p client --test shop_auction_ui_card_text_stat_keyword_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- `cargo test -p client --test shop_auction_ui_shop_panel_test`
- `cargo test -p client --test shop_auction_ui_auction_activation_test`
- `cargo test -p client --test shop_auction_ui_auction_panel_test`
- `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
- `cargo test -p client --test shop_auction_ui_auction_settlement_test`
- `cargo check -p client`
- `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-card-text-stat-keyword-accessibility.md`

**Required browser/WASM capture artifact directory**:

- `production/qa/evidence/captures/shop-auction-ui-card-text-stat-keyword-accessibility/`

**Required browser/WASM evidence contents**:

- Browser, build target, commit, capture command, fixture source, and UI scale.
- Viewports: `1366x768` and `1920x1080`.
- Fixture cards covering minion, structure, spell or instant, zero-cost,
  two-digit stat, long-name, no-keyword, and multi-keyword cases.
- Surface table covering DRAFT_INITIAL grid cards, DRAFT_SHOP shop cards,
  DRAFT_AUCTION featured cards, DRAFT_AUCTION footer cards, and settlement card
  states.
- Text-size table for cost, ATK, HP, and keyword text by Shop/Auction UI
  surface and viewport.
- Explicit not-applicable entries for fields absent from a card type or state.
- Overlap table comparing text bounds, card bounds, adjacent cards, panel
  controls, HUD zones, hand tray, auction controls, settlement overlays, and
  adjacent overlays.
- Contrast sample table for visible cost, ATK, HP, and keyword text.
- Browser/WASM capture links under the required capture directory.
- Focused automated test command output summary.
- Regression command output summary.
- A11Y-ST-02 Shop/Auction UI impact statement.
- QA-COND-0005 impact statement confirming the condition remains Open.

**QA-COND-0005 impact statement required in evidence**:

Story 013 implements and evidences the Shop/Auction UI slice of A11Y-ST-02 for
card cost, ATK, HP, and keyword text floors across DRAFT_INITIAL grid,
DRAFT_SHOP shop, DRAFT_AUCTION featured, DRAFT_AUCTION footer, and settlement
card views. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open
until all remaining Standard-tier rows are implemented and evidenced,
reclassified, dependency-blocked, or accepted as risk.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Story 002](story-002-draft-initial-grid-purchase-ready.md) -
  Complete; provides active DRAFT_INITIAL grid, purchase, bought-slot, Ready,
  and PLACEMENT dismissal behavior.
- Depends on: [Story 003](story-003-shop-panel-slots-refresh-purchase-ready.md)
  - Complete; provides DRAFT_SHOP card slot, refresh, purchase, and hand-full
  behavior.
- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md)
  - Complete; provides auction panel activation, preparing state, featured card
  ownership, and read-only footer boundary.
- Depends on: [Story 007](story-007-auction-settlement-and-shop-transition.md)
  - Ready; provides settlement card views and auction-to-shop transition
  surfaces. Implement Story 007 before this story if settlement card rendering
  is still absent.
- Depends on: [Story 011](story-011-auction-bid-target-size-and-focus-evidence.md)
  - Complete; provides auction bid target and focus evidence that must be
  preserved.
- Depends on: [Story 012](story-012-draft-initial-clear-objective-overlay.md)
  - Complete; provides DRAFT_INITIAL objective overlay behavior that must not
  occlude grid card readability.
- Depends on: ADR-002, ADR-013, ADR-015, ADR-019, and ADR-021 Accepted.
- Unlocks: Presentation Layer Story 005 final A11Y-ST-02 cross-surface
  browser/WASM evidence after Hand UI Story 015 is also implemented.

## Blockers

None.

## Performance Budget

No gameplay-loop performance impact is expected from Shop/Auction UI typography
adjustments or evidence-only measurement instrumentation. Any production
remediation must preserve ADR-021 presentation guardrails: steady-state
presentation work remains below 1 ms per frame and phase-boundary spikes remain
below 3 ms. The implementation must not add per-frame entity creation, extra
Lightyear message drains, card catalog scans, texture uploads, or persistent
debug overlays.

## No Open Questions

None.
