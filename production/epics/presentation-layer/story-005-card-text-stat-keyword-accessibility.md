# Story 005: A11Y-ST-02 Cross-Surface Browser/WASM Evidence

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
HP, or keyword text floors.

This story is the final cross-surface Browser/WASM evidence pass for
A11Y-ST-02. It starts after the split implementation stories land:

- Hand UI Story 015 owns Hand UI card fan, staged fan, and acquisition feedback
  remediation/evidence.
- Shop/Auction UI Story 013 owns DRAFT_INITIAL grid, DRAFT_SHOP shop,
  DRAFT_AUCTION featured, DRAFT_AUCTION footer, and settlement card
  remediation/evidence.

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

- `design/accessibility-requirements.md` Standard-tier row `Minimum text size -
  card text (cost, ATK, HP, keyword)` requires ATK and HP stat badges to have
  an 18px minimum floor and keyword text to have a 14px minimum floor.
- A11Y-ST-02 explicitly includes card cost, so this final evidence story
  verifies the same 18 CSS px floor for visible card cost numerals or badges.

**GDD Requirements**:

- `design/gdd/card-data-pool.md` Card definition schema requires visible card
  display fields including `cost`, `atk`, `hp`, `keywords`, and `effect_text`.
- `design/gdd/hand-ui.md` Rule 1 requires 10 pre-pooled hand fan card slots and
  9 pre-pooled DRAFT_INITIAL grid slots.
- `design/gdd/hand-ui.md` Visual Anatomy VA-1 places card cost, ATK, and HP
  badges on the card face and requires readable card identity at 10-card hand
  compression.
- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 2 requires a 3x3 card
  grid sorted by rarity and cost.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 1 requires the auction
  panel to render the auction card art, rarity badge, and starting price after
  `S2CAuctionCard` and phase data are available.
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 2 requires read-only shop
  footer card costs to remain visible at 30 percent opacity during auction.
- `design/gdd/shop-auction-ui.md` DRAFT_SHOP Rule 2 requires three horizontal
  shop slots, each showing card art, name, rarity badge, and cost.

**UX Requirements**:

- `design/ux/hand-ui.md` requires card cost, ATK, HP, and type or rarity to
  remain readable at 10-card hand compression.
- `design/ux/shop-auction-ui.md` requires card keyword text to remain readable
  on hover or zoom, body text contrast of at least `4.5:1`, and no required UI
  overlap at `1366x768` and `1920x1080`.
- `design/ux/interaction-patterns.md` PTN-INP-005 defines purchasable card
  presentation as card art, name, rarity, cost, and purchase state in a stable
  slot.

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
- [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md)
- [ADR-019: Economy System Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: A11Y-ST-02 is presentation-only. The client reads
server-authoritative phase, card, hand, shop, auction, and economy state, then
renders readable card surfaces. Final evidence must not mutate authoritative
card data, gold, hand ownership, auction price, bid state, or phase state.

**Engine**: Bevy 0.18 + browser/WASM target | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file. This
story should normally be evidence-only. If the final browser/WASM evidence
finds a missing test hook or measurement export, repair that hook only in the
owning split story area and preserve Bevy 0.18 Required Components API rules.

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

- Run the final A11Y-ST-02 browser/WASM evidence pass after Hand UI Story 015
  and Shop/Auction UI Story 013 have landed.
- Verify the combined card readability matrix across these card views:
  - DRAFT_INITIAL 3x3 grid cards.
  - DRAFT_INITIAL acquired-card feedback in the Hand UI fan.
  - DRAFT_SHOP hand fan cards.
  - DRAFT_AUCTION read-only hand fan cards.
  - PLACEMENT active fan cards, staged fan ghosts, and Instant fan ghosts.
  - DRAFT_SHOP shop cards.
  - DRAFT_AUCTION auction preparing and active featured card views.
  - DRAFT_AUCTION read-only shop footer cards.
  - DRAFT_AUCTION settlement card views.
- Measure cost, ATK, HP, and keyword text at browser/WASM viewports `1366x768`
  and `1920x1080`.
- Verify a minimum floor of `18 CSS px` or browser-equivalent rendered pixels
  for visible cost, ATK, and HP numerals or badge text.
- Verify a minimum floor of `14 CSS px` or browser-equivalent rendered pixels
  for visible keyword text.
- Verify explicit not-applicable entries for card types that do not have ATK,
  HP, or visible keyword text in a given fixture or state.
- Verify that card text does not clip, truncate into unreadability, or overlap
  another required card element, sibling card, HUD zone, hand fan element, shop
  control, auction control, timer, tooltip, settlement overlay, or evidence
  overlay.
- Verify body text contrast for sampled keyword text and stat numerals where
  text is composited against the final browser/WASM background. Sampled pairs
  must meet at least `4.5:1`.
- Combine Hand UI Story 015 and Shop/Auction UI Story 013 evidence into the
  final evidence document listed in `## Test Evidence`.
- Record a single A11Y-ST-02 pass/fail impact statement for QA-COND-0005.

### Out of Scope

- New Hand UI card typography remediation unless Hand UI Story 015 first
  failed and is repaired in its owning scope.
- New Shop/Auction UI card typography remediation unless Shop/Auction UI Story
  013 first failed and is repaired in its owning scope.
- Changes to authoritative card catalog fields, card costs, ATK, HP, keywords,
  rarity, effect text, pool distribution, economy, auction bidding, purchase
  validation, hand ownership, phase timing, network protocol payloads, or
  server authority.
- Broader UI scaling preferences, Settings or Accessibility screen work,
  colorblind palette implementation, full A11Y-ST-01 HUD text evidence, or
  A11Y-ST-03 global contrast evidence beyond sampled card text pairs.
- Changes to `production/sprint-status.yaml`, `production/session-state/**`,
  project asset files, or `AGENTS.md`.
- QA-COND-0005 closure. This story contributes only the final A11Y-ST-02
  evidence row.

---

## Acceptance Criteria

- [ ] **Split implementation inputs are present**: GIVEN this story begins,
  WHEN implementation history and story files are reviewed, THEN Hand UI Story
  015 and Shop/Auction UI Story 013 are both implemented with their required
  evidence documents and capture directories present.
- [ ] **Combined surface matrix exists**: GIVEN final browser/WASM evidence
  runs at `1366x768` and `1920x1080`, WHEN the evidence document is reviewed,
  THEN it includes every in-scope Hand UI and Shop/Auction UI card surface
  listed in `## Scope`.
- [ ] **Cost floor passes cross-surface**: GIVEN any visible card cost numeral
  or badge in the measured surfaces, WHEN final text-size measurements are
  reviewed, THEN the cost text measures at least `18 CSS px` or
  browser-equivalent rendered pixels at both viewports.
- [ ] **ATK floor passes cross-surface**: GIVEN any visible card ATK numeral or
  badge in the measured surfaces, WHEN final text-size measurements are
  reviewed, THEN the ATK text measures at least `18 CSS px` or
  browser-equivalent rendered pixels at both viewports.
- [ ] **HP floor passes cross-surface**: GIVEN any visible card HP numeral or
  badge in the measured surfaces, WHEN final text-size measurements are
  reviewed, THEN the HP text measures at least `18 CSS px` or
  browser-equivalent rendered pixels at both viewports.
- [ ] **Keyword floor passes cross-surface**: GIVEN any visible keyword text in
  the measured surfaces, WHEN final text-size measurements are reviewed, THEN
  the keyword text measures at least `14 CSS px` or browser-equivalent rendered
  pixels at both viewports.
- [ ] **Not-applicable fields are explicit**: GIVEN a fixture card has no ATK,
  no HP, or no visible keyword text, WHEN the final evidence table is reviewed,
  THEN that field is marked `N/A - field not present on this card type or
  state` rather than being silently omitted.
- [ ] **Long and dense card text remains readable cross-surface**: GIVEN
  fixture cards include at least one long card name, one two-keyword card, one
  zero-cost card, and one card with two-digit cost, ATK, or HP, WHEN final
  browser/WASM captures are reviewed, THEN visible cost, ATK, HP, and keyword
  text remains inside its card or badge bounds without clipping or unreadable
  truncation across all in-scope surfaces.
- [ ] **No card-internal overlap cross-surface**: GIVEN measured text bounds
  and card bounds, WHEN final overlap checks run, THEN cost, ATK, HP, keyword
  text, card name, rarity indicator, and visible card art zones do not overlap
  in a way that makes required text unreadable.
- [ ] **No surface-level overlap cross-surface**: GIVEN measured card and UI
  bounds, WHEN final overlap checks run, THEN card text does not overlap sibling
  cards, fan ghosts, reserve strips, shop controls, auction controls, Ready
  controls, timers, tooltips, HUD chips, hand tray, settlement overlays, panel
  boundaries, or browser evidence overlays.
- [ ] **Contrast sample passes cross-surface**: GIVEN sampled browser/WASM
  foreground and composited background colors for cost, ATK, HP, and keyword
  text, WHEN contrast ratios are computed, THEN each sampled text/background
  pair meets at least `4.5:1`.
- [ ] **Focused accessibility tests pass**: The Hand UI and Shop/Auction UI
  focused accessibility test commands from Story 015 and Story 013 both pass,
  and their command summaries are included in the final evidence document.
- [ ] **Regression summaries are included**: The final evidence document
  includes the Hand UI and Shop/Auction UI regression command summaries from
  Story 015 and Story 013.
- [ ] **Browser/WASM evidence exists**:
  `production/qa/evidence/presentation-card-text-accessibility.md` records the
  browser, build target, commit, capture command, fixture cards, viewport table,
  text-size table, overlap table, contrast sample table, screenshot capture
  directory, pass/fail verdict, and QA-COND-0005 impact statement.
- [ ] **Capture directory is populated**:
  `production/qa/evidence/captures/presentation-card-text-accessibility/`
  contains the browser/WASM captures referenced by the final evidence document
  for both required viewports and every required card surface.
- [ ] **A11Y-ST-02 impact is explicit**: The final evidence document states
  whether A11Y-ST-02 is implemented and evidenced across all in-scope card
  surfaces, or which measured surface still fails the cost, ATK, HP, keyword,
  non-overlap, or readability checks.
- [ ] **QA-COND-0005 remains open**: The final evidence document states that
  this story contributes only the A11Y-ST-02 card text accessibility row and
  that QA-COND-0005 remains Open until all remaining Standard-tier rows are
  implemented and evidenced, reclassified, dependency-blocked, or accepted as
  risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Treat this as the final Browser/WASM evidence and aggregation story, not the
  first implementation story.
- Use the focused evidence from Hand UI Story 015 and Shop/Auction UI Story 013
  as inputs. Do not duplicate ownership by moving remediation into
  Presentation Layer.
- If final capture finds a failing surface, repair that surface in the owning
  split story scope first, then rerun this final evidence story.
- Keep any browser measurement overlay, evidence harness, or bounds export
  test-only or evidence-only.
- Do not reduce DRAFT_AUCTION footer opacity below the GDD-specified 30 percent
  value as a readability workaround.
- Do not move auction price counter remediation into card-text scope. The
  auction price counter is owned by separate HUD or Shop/Auction evidence for
  A11Y-ST-01 and A11Y-ST-03.

---

## QA Test Cases

- **Cross-surface card text measurement**
  - Given: Browser/WASM evidence fixtures render every in-scope card surface at
    `1366x768` and `1920x1080`
  - When: final text metrics are exported
  - Then: every visible cost, ATK, and HP field is at least `18 CSS px`, every
    visible keyword field is at least `14 CSS px`, and absent fields are
    explicitly recorded

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

- **Evidence aggregation**
  - Given: Hand UI Story 015 and Shop/Auction UI Story 013 evidence documents
    exist
  - When: the final Presentation Layer evidence document is reviewed
  - Then: it includes links or summaries for both split stories, the combined
    surface matrix, final pass/fail verdict, and QA-COND-0005 impact statement

---

## Test Evidence

**Story Type**: UI

**Required source evidence inputs**:

- `production/qa/evidence/hand-ui-card-text-stat-keyword-accessibility.md`
- `production/qa/evidence/shop-auction-ui-card-text-stat-keyword-accessibility.md`

**Required automated test targets**:

- `tests/integration/hand-ui/card_text_stat_keyword_accessibility_test.rs`
  - Registered as `hand_ui_card_text_stat_keyword_accessibility_test`
  - Command:
    `cargo test -p client --test hand_ui_card_text_stat_keyword_accessibility_test`
- `tests/integration/shop_auction_ui/card_text_stat_keyword_accessibility_test.rs`
  - Registered as `shop_auction_ui_card_text_stat_keyword_accessibility_test`
  - Command:
    `cargo test -p client --test shop_auction_ui_card_text_stat_keyword_accessibility_test`

**Required regression commands**:

- Hand UI regression commands from Story 015.
- Shop/Auction UI regression commands from Story 013.
- `cargo check -p client`
- `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/presentation-card-text-accessibility.md`

**Required browser/WASM capture artifact directory**:

- `production/qa/evidence/captures/presentation-card-text-accessibility/`

**Required browser/WASM evidence contents**:

- Browser, build target, commit, capture command, fixture source, and UI scale.
- Viewports: `1366x768` and `1920x1080`.
- Fixture cards covering minion, structure, spell or instant, zero-cost,
  two-digit stat, long-name, no-keyword, and multi-keyword cases.
- Combined surface table covering all Hand UI and Shop/Auction UI card surfaces
  listed in `## Scope`.
- Text-size table for cost, ATK, HP, and keyword text by surface and viewport.
- Explicit not-applicable entries for fields absent from a card type or state.
- Overlap table comparing text bounds, card bounds, and adjacent UI bounds.
- Contrast sample table for visible cost, ATK, HP, and keyword text.
- Browser/WASM capture links under the required capture directory.
- Focused automated test command output summaries.
- Regression command output summaries.
- Links or summaries for the Hand UI Story 015 evidence input and the
  Shop/Auction UI Story 013 evidence input.
- A11Y-ST-02 impact statement.
- QA-COND-0005 impact statement confirming the condition remains Open.

**QA-COND-0005 impact statement required in evidence**:

Story 005 implements the final cross-surface A11Y-ST-02 Browser/WASM evidence
pass for card cost, ATK, HP, and keyword text floors after Hand UI Story 015
and Shop/Auction UI Story 013 land. It does not close QA-COND-0005 by itself.
QA-COND-0005 remains Open until all remaining Standard-tier rows are
implemented and evidenced, reclassified, dependency-blocked, or accepted as
risk.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Hand UI Story 015](../hand-ui/story-015-card-text-stat-keyword-accessibility.md)
  - Ready; provides Hand UI A11Y-ST-02 remediation and focused evidence.
- Depends on: [Shop/Auction UI Story 013](../shop-auction-ui/story-013-card-text-stat-keyword-accessibility.md)
  - Ready; provides Shop/Auction UI A11Y-ST-02 remediation and focused evidence.
- Depends on: `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`
  identifying A11Y-ST-02 as an evidence-only required row that blocks
  QA-COND-0005 closure.
- Depends on:
  `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md`
  remaining Open and defining the closure guard.
- Depends on: ADR-002, ADR-013, ADR-015, ADR-019, and ADR-021 Accepted.
- Unlocks: A11Y-ST-02 can move from evidence-only required to implemented and
  evidenced in the Sprint 6 accessibility evidence register after this story is
  implemented and
  `production/qa/evidence/presentation-card-text-accessibility.md` passes QA
  review. This does not unlock QA-COND-0005 closure by itself.

## Blockers

None.

## Performance Budget

No measurable gameplay-loop performance impact is expected from final evidence
aggregation. If evidence-only measurement instrumentation is needed, it must
preserve ADR-021 presentation guardrails: steady-state presentation work remains
below 1 ms per frame and phase-boundary spikes remain below 3 ms. The story
must not add per-frame entity creation, extra Lightyear message drains, card
catalog scans, texture uploads, or persistent debug overlays.

## QA-COND-0005 Impact

This story targets only the final A11Y-ST-02 card text accessibility evidence
row. Completing it reduces QA-COND-0005 by attaching card text-size,
readability, non-overlap, and contrast sample evidence, but QA-COND-0005
remains Open until every other Standard-tier blocker has implementation and
evidence, reclassification, dependency-blocking, or accepted-risk disposition.

## No Open Questions

None.
