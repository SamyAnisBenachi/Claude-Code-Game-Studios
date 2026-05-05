# Story 005: Card Text, Stat, and Keyword Accessibility

> **Epic**: Presentation Layer
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
HP, or keyword text floors, and that QA-COND-0005 closure remains blocked until
that evidence exists.

**Primary Sources**:

- `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
- `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
- `design/accessibility-requirements.md`
- `design/gdd/card-data-pool.md`
- `design/gdd/hand-ui.md`
- `design/gdd/shop-auction-ui.md`
- `design/ux/hand-ui.md`
- `design/ux/shop-auction-ui.md`
- `design/ux/interaction-patterns.md`

**Accessibility Requirement**:

- `design/accessibility-requirements.md` Standard-tier row
  `Minimum text size - card text (cost, ATK, HP, keyword)` requires card stat
  badges for ATK and HP to have an 18px minimum floor, and keyword text to have
  a 14px minimum floor. This story applies the same 18 CSS px floor to the
  visible card cost badge or numeral because A11Y-ST-02 explicitly includes
  card cost in the evidence gap.

**GDD Requirements**:

- `design/gdd/card-data-pool.md` Card definition schema requires visible card
  display fields including `cost`, `atk`, `hp`, `keywords`, and `effect_text`.
- `design/gdd/hand-ui.md` Rule 1 requires 10 pre-pooled hand fan card slots
  and 9 pre-pooled DRAFT_INITIAL grid slots.
- `design/gdd/hand-ui.md` Rule 4 requires each DRAFT_INITIAL grid cell to show
  card art, name, mana cost, and rarity indicator.
- `design/gdd/hand-ui.md` Visual Anatomy VA-1 places cost in the top-left,
  ATK in a bottom-right orange badge, HP below ATK in a teal gem, and requires
  ATK and HP badges to maintain a minimum render floor at 10-card overlap.
- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 2 requires a 3x3 card
  grid sorted by rarity and cost.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 1 requires the auction
  panel to render the auction card art, rarity badge, and starting price as the
  current price after `S2CAuctionCard` and phase data are available.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 2 requires read-only shop
  footer card costs to remain visible at 30 percent opacity during auction.
- `design/gdd/shop-auction-ui.md` DRAFT_SHOP Rule 2 requires three horizontal
  shop slots, each showing card art, name, rarity badge, and cost.

**UX Requirements**:

- `design/ux/hand-ui.md` and `design/ux/shop-auction-ui.md` identify card
  readability, text contrast, and non-overlap as required browser/WASM evidence
  concerns for card surfaces.
- `design/ux/shop-auction-ui.md` Accessibility requires card keyword text to
  remain readable on hover or zoom, and requires body text contrast of at least
  4.5:1 where card text is sampled against its rendered background.
- `design/ux/shop-auction-ui.md` acceptance criteria require no required UI
  overlap at `1366x768` and `1920x1080`; this story applies that rule to card
  cost, ATK, HP, and keyword text across the card surfaces below.

**TR IDs**:

- `TR-CDP-010` for DRAFT_INITIAL and shop card payloads arriving before client
  phase/UI use.
- `TR-HU-001` for pre-pooled fan and DRAFT_INITIAL grid card slots.
- `TR-HU-005` for DRAFT_INITIAL 9-card display and budget presentation.
- `TR-SAU-003` for auction settlement display.
- `TR-SAU-004` for locked DRAFT_AUCTION shop footer slots.
- `TR-SAU-006` for Shop/Auction panel transitions and presentation coverage.

`A11Y-ST-02` is a Sprint 6 accessibility evidence row, not a registered
`TR-PRES-*` requirement.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-013: Auction System State Machine and Bid Processing Architecture](../../../docs/architecture/adr-013-auction-system-state.md)
- [ADR-019: Economy System Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: Card text accessibility is presentation-only. The
client reads server-authoritative phase, card, hand, shop, auction, and economy
state, then renders readable card surfaces. Remediation must not mutate
authoritative card data, gold, hand ownership, auction price, bid state, or
phase state.

**Engine**: Bevy 0.18 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file. Card
text and card badges must remain Bevy 0.18 presentation UI or sprite-backed
presentation elements that follow the Required Components API. Do not use
`NodeBundle`, `TextBundle`, `SpriteBundle`, `UiImage::new()`, `Parent`, or
`Color::rgba()`. If a browser measurement overlay or evidence harness is added,
keep it test-only or evidence-only.

**Control Manifest Rules (2026-05-05)**:

- Required: client presentation is a read-only view of server-authoritative
  state.
- Required: `PresentationPlugin` registration order remains Card Animations,
  Board Rendering, Hand UI, HUD, Shop/Auction UI.
- Required: presentation work runs in the ADR-021 order
  `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Required: UI overlays such as HUD, hand fan, shop panels, and auction bid box
  use `bevy_ui`; board content remains world-space.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: economy presentation reads `PlayerEconomyView` rather than adding
  new independent economy message drains.
- Forbidden: client presentation must not assert or mutate authoritative game
  state.
- Guardrail: presentation steady-state stays below 1 ms per frame and
  phase-boundary presentation spikes stay below 3 ms.

---

## Scope

### In Scope

- Verify or remediate A11Y-ST-02 card text floors across these card views:
  - DRAFT_INITIAL 3x3 grid cards.
  - Hand/fan cards at rest, hover or zoom, read-only DRAFT_AUCTION fan, and
    staged PLACEMENT fan ghosts.
  - DRAFT_SHOP shop cards.
  - DRAFT_AUCTION auction featured card, auction preparing card view, settlement
    card view, and read-only shop footer cards.
- Measure cost, ATK, HP, and keyword text at browser/WASM viewports `1366x768`
  and `1920x1080`.
- Apply a minimum floor of `18 CSS px` or browser-equivalent rendered pixels to
  visible cost, ATK, and HP numerals or badge text.
- Apply a minimum floor of `14 CSS px` or browser-equivalent rendered pixels to
  visible keyword text.
- Record explicit not-applicable entries for card types that do not have ATK,
  HP, or visible keyword text in a given fixture.
- Verify that card text does not clip, truncate into unreadability, or overlap
  another required card element, sibling card, HUD zone, hand fan element, shop
  control, auction control, timer, tooltip, or overlay.
- Verify body text contrast for sampled keyword text and stat numerals where
  the text is composited against the final browser/WASM background; sampled
  pairs must meet at least `4.5:1`.
- Add or update focused automated coverage that exposes card text metrics,
  card bounds, and overlap checks without relying only on manual visual
  judgment.
- Capture browser/WASM evidence and update the exact evidence document listed
  in `## Test Evidence`.
- Preserve existing card surface behavior, input gating, panel activation,
  purchase, bid, settlement, and hand/fan state semantics.

### Out of Scope

- No code implementation in this readiness pass.
- No changes to authoritative card catalog fields, card costs, ATK, HP,
  keywords, rarity, effect text, or pool distribution.
- No changes to economy, auction bidding, purchase validation, hand ownership,
  phase timing, network protocol payloads, or server authority.
- No broader UI scaling preference implementation.
- No colorblind palette implementation.
- No Settings or Accessibility screen work.
- No full A11Y-ST-01 HUD text evidence or A11Y-ST-03 global contrast evidence
  beyond the card text pairs sampled for this story.
- No changes to `production/sprint-status.yaml`.
- No changes to `production/session-state/**`.
- No changes to project asset files.
- No changes to `AGENTS.md`.
- Do not close QA-COND-0005 from this story alone.

---

## Acceptance Criteria

- [ ] **DRAFT_INITIAL grid evidence exists**: GIVEN browser/WASM evidence runs
  at `1366x768` and `1920x1080`, WHEN the DRAFT_INITIAL 3x3 grid is rendered
  with a fixture containing minion, spell or instant, structure, long-name, and
  multi-keyword cards, THEN the evidence records cost, ATK, HP, keyword text,
  card bounds, and text bounds for every visible grid card field that applies.
- [ ] **Hand/fan evidence exists**: GIVEN browser/WASM evidence runs at both
  required viewports, WHEN the hand/fan is rendered with 10 cards in DRAFT_SHOP,
  DRAFT_AUCTION read-only, and PLACEMENT staged ghost states, THEN the evidence
  records cost, ATK, HP, keyword text, card bounds, and text bounds for each
  visible fan card field that applies.
- [ ] **Shop card evidence exists**: GIVEN browser/WASM evidence runs at both
  required viewports, WHEN DRAFT_SHOP shows three shop cards and the DRAFT_AUCTION
  read-only footer shows three locked shop cards at its specified opacity, THEN
  the evidence records cost, ATK, HP, keyword text, card bounds, and text bounds
  for every visible shop/footer field that applies.
- [ ] **Auction card evidence exists**: GIVEN browser/WASM evidence runs at both
  required viewports, WHEN the auction featured card is shown in preparing,
  active, and settlement states, THEN the evidence records cost, ATK, HP,
  keyword text, card bounds, and text bounds for every visible auction-card
  field that applies.
- [ ] **Cost floor passes**: GIVEN any visible card cost numeral or badge in the
  measured surfaces, WHEN text-size measurements are reviewed, THEN the cost
  text measures at least `18 CSS px` or browser-equivalent rendered pixels at
  both viewports.
- [ ] **ATK floor passes**: GIVEN any visible card ATK numeral or badge in the
  measured surfaces, WHEN text-size measurements are reviewed, THEN the ATK text
  measures at least `18 CSS px` or browser-equivalent rendered pixels at both
  viewports.
- [ ] **HP floor passes**: GIVEN any visible card HP numeral or badge in the
  measured surfaces, WHEN text-size measurements are reviewed, THEN the HP text
  measures at least `18 CSS px` or browser-equivalent rendered pixels at both
  viewports.
- [ ] **Keyword floor passes**: GIVEN any visible keyword text in the measured
  surfaces, WHEN text-size measurements are reviewed, THEN the keyword text
  measures at least `14 CSS px` or browser-equivalent rendered pixels at both
  viewports.
- [ ] **Not-applicable fields are explicit**: GIVEN a fixture card has no ATK,
  no HP, or no visible keyword text, WHEN the evidence table is reviewed, THEN
  that field is marked `N/A - field not present on this card type or state`
  rather than being silently omitted.
- [ ] **Long and dense card text remains readable**: GIVEN fixture cards include
  at least one long card name, at least one two-keyword card, at least one
  zero-cost card, and at least one card with two-digit cost, ATK, or HP, WHEN
  browser/WASM captures are reviewed, THEN visible cost, ATK, HP, and keyword
  text remains inside its card or badge bounds without clipping or unreadable
  truncation.
- [ ] **No card-internal overlap**: GIVEN the measured text bounds and card
  bounds, WHEN overlap checks run, THEN cost, ATK, HP, keyword text, card name,
  rarity indicator, and visible card art zones do not overlap in a way that
  makes required text unreadable.
- [ ] **No surface-level overlap**: GIVEN the measured card and UI bounds, WHEN
  overlap checks run, THEN card text does not overlap sibling cards, fan ghosts,
  shop controls, auction controls, Ready controls, timers, tooltips, HUD chips,
  settlement overlays, or browser evidence overlays.
- [ ] **Contrast sample passes for card text**: GIVEN sampled browser/WASM
  foreground and composited background colors for cost, ATK, HP, and keyword
  text, WHEN contrast ratios are computed, THEN each sampled text/background
  pair meets at least `4.5:1`.
- [ ] **Existing behavior is preserved**: GIVEN existing Hand UI and
  Shop/Auction UI regression tests run after remediation, WHEN card typography
  or measurement instrumentation is applied, THEN purchase, refresh, read-only
  footer, auction preparing, auction active, settlement, hand/fan visibility,
  fan staged ghost, and input suppression behavior remains unchanged.
- [ ] **Focused accessibility test passes**:
  `cargo test -p client --test card_text_accessibility_test` passes. The target
  must be backed by `tests/integration/presentation/card_text_accessibility_test.rs`
  and registered as `card_text_accessibility_test`.
- [ ] **Browser/WASM evidence exists**:
  `production/qa/evidence/presentation-card-text-accessibility.md` records the
  browser, build target, commit, capture command, fixture cards, viewport table,
  text-size table, overlap table, contrast sample table, screenshot capture
  directory, pass/fail verdict, and QA-COND-0005 impact statement.
- [ ] **Capture directory is populated**:
  `production/qa/evidence/captures/presentation-card-text-accessibility/`
  contains the browser/WASM captures referenced by the evidence document for
  both required viewports and every required card surface.
- [ ] **A11Y-ST-02 impact is explicit**: The evidence document states whether
  A11Y-ST-02 is implemented and evidenced by this story, or which measured
  surface still fails the cost, ATK, HP, keyword, non-overlap, or readability
  checks.
- [ ] **QA-COND-0005 remains open**: The evidence document states that this
  story contributes only the A11Y-ST-02 card text accessibility row and that
  QA-COND-0005 remains Open until all remaining Standard-tier rows are
  implemented and evidenced, reclassified, dependency-blocked, or accepted as
  risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep work local to presentation card rendering, test fixtures, and evidence
  harnesses needed for A11Y-ST-02.
- Prefer adjusting existing typography constants, theme tokens, card layout
  slots, badge sizes, or hover/zoom surfaces over changing card data or game
  logic.
- If a card surface intentionally hides text at rest and shows it only on hover
  or zoom, the evidence must show both the hidden/rest state and the readable
  hover or zoom state. The evidence must state which state is the accepted
  player-readable state for that field.
- If a card type has no ATK or HP, do not add fake stat text to satisfy the
  measurement table. Record the field as not applicable for that card type.
- If keyword text wraps, the wrapped line block must still meet the `14 CSS px`
  floor, remain inside the card surface, and avoid overlap with stats or card
  controls.
- If browser metrics are exported through a debug overlay, measurement log, or
  test-only component, keep that instrumentation out of normal shipping UI.
- Do not reduce DRAFT_AUCTION footer opacity below the GDD-specified value as a
  readability workaround. If 30 percent opacity makes required text fail, choose
  a remediation that preserves read-only state while meeting A11Y-ST-02.
- Do not move the auction price counter into card-text scope. The auction price
  counter is owned by separate HUD or Shop/Auction evidence for A11Y-ST-01 and
  A11Y-ST-03.

---

## QA Test Cases

- **DRAFT_INITIAL card text measurement**
  - Given: Browser/WASM DRAFT_INITIAL renders a 3x3 fixture at `1366x768` and
    `1920x1080`
  - When: text metrics are exported
  - Then: every visible cost, ATK, and HP text field is at least `18 CSS px`,
    every visible keyword field is at least `14 CSS px`, and not-applicable
    stat fields are explicitly recorded

- **Hand/fan card text measurement**
  - Given: The hand/fan renders 10 cards in DRAFT_SHOP, DRAFT_AUCTION read-only,
    and PLACEMENT staged ghost states
  - When: text metrics and bounds are exported
  - Then: visible cost, ATK, HP, and keyword fields meet their floors and remain
    readable despite fan overlap, dimming, hover, zoom, and ghost treatments

- **Shop and auction card text measurement**
  - Given: DRAFT_SHOP slots, DRAFT_AUCTION footer slots, auction preparing, and
    active auction card views are rendered
  - When: text metrics and bounds are exported
  - Then: visible cost, ATK, HP, and keyword fields meet their floors in every
    state where the field is intended to be readable

- **Overlap guard**
  - Given: Browser/WASM card and text bounds are captured for every required
    surface
  - When: overlap checks compare required text fields against sibling elements
    and adjacent UI
  - Then: no required card text overlaps another element in a way that makes it
    unreadable

- **Contrast sample guard**
  - Given: Browser/WASM foreground/background samples are captured for visible
    cost, ATK, HP, and keyword text
  - When: contrast ratios are computed after compositing
  - Then: sampled pairs meet at least `4.5:1`

- **Behavior preservation**
  - Given: existing Hand UI and Shop/Auction UI behavior regressions run
  - When: card text accessibility changes are present
  - Then: purchase, refresh, auction, settlement, hand/fan visibility, staged
    ghost, and input gating behavior remain unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/presentation/card_text_accessibility_test.rs`
  - Registered as `card_text_accessibility_test`
  - Command: `cargo test -p client --test card_text_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- `cargo test -p client --test shop_auction_ui_auction_panel_test`
- `cargo test -p client --test shop_auction_ui_shop_panel_test`
- `cargo test -p client --test hand_ui_draft_initial_grid_test`
- `cargo test -p client --test hand_ui_fan_layout_test`
- `cargo check -p client`
- `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/presentation-card-text-accessibility.md`

**Required browser/WASM capture artifact directory**:

- `production/qa/evidence/captures/presentation-card-text-accessibility/`

**Required browser/WASM evidence contents**:

- Browser, build target, commit, capture command, and fixture source.
- Viewports: `1366x768` and `1920x1080`.
- UI scale used for the capture.
- Fixture cards covering minion, structure, spell or instant, zero-cost,
  two-digit stat, long-name, no-keyword, and multi-keyword cases.
- Surface table covering DRAFT_INITIAL grid cards, hand/fan cards, DRAFT_SHOP
  shop cards, DRAFT_AUCTION footer cards, and auction featured card views.
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

Story 005 implements and evidences A11Y-ST-02 for card cost, ATK, HP, and
keyword text floors across DRAFT_INITIAL grid, hand/fan, shop, and auction card
views. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open
until all remaining Standard-tier rows are implemented and evidenced,
reclassified, dependency-blocked, or accepted as risk.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Story dependencies: None.
- Source dependency: `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
  identifies A11Y-ST-02 as an evidence-only required row that blocks
  QA-COND-0005 closure.
- Source dependency:
  `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains
  Open and defines the closure guard.
- Source dependency: ADR-002, ADR-013, ADR-019, and ADR-021 are Accepted.
- Unlocks: A11Y-ST-02 can move from evidence-only required to implemented and
  evidenced after this story is implemented and
  `production/qa/evidence/presentation-card-text-accessibility.md` passes QA
  review. This does not unlock QA-COND-0005 closure by itself.

## Performance Budget

No measurable gameplay-loop performance impact is expected from typography
adjustments or evidence-only measurement instrumentation. Any production
remediation must preserve the ADR-021 presentation guardrails: steady-state
presentation work remains below 1 ms per frame and phase-boundary spikes remain
below 3 ms. The implementation must not add per-frame entity creation, extra
Lightyear message drains, card catalog scans, texture uploads, or persistent
debug overlays.

## QA-COND-0005 Impact

This story targets only A11Y-ST-02 card text accessibility. Completing it
reduces QA-COND-0005 by attaching card text-size, readability, non-overlap, and
contrast sample evidence, but QA-COND-0005 remains Open until every other
Standard-tier blocker has implementation and evidence, reclassification,
dependency-blocking, or accepted-risk disposition.

## No Open Questions

None.
