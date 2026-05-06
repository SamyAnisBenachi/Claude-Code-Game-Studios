# Story 015: Card Text, Stat, and Keyword Accessibility

> **Epic**: Hand UI
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
HP, or keyword text floors. This story owns the Hand UI portion of that row.

**GDD**: `design/gdd/hand-ui.md`
**UX Spec**: `design/ux/hand-ui.md`
**Accessibility Source**: `design/accessibility-requirements.md`
**Requirement**: `TR-HU-001`, `TR-HU-002`, `TR-HU-003`, `TR-HU-005`,
`TR-HU-008`, A11Y-ST-02

**ADR Governing Implementation**:
[ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md),
[ADR-002: Client-Server Authority Model](../../../docs/architecture/adr-002-client-server-authority.md),
[ADR-019: Economy System Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)

**Control Manifest**: `docs/architecture/control-manifest.md` version
`2026-05-05`.

**GDD trace**:

- `design/gdd/hand-ui.md` Rule 1 and acceptance criterion HU-01 require 10
  pre-pooled hand fan card slots and 9 pre-pooled DRAFT_INITIAL grid slots,
  with card visibility toggled from server-authoritative hand state.
- `design/gdd/hand-ui.md` Rules 5, 5d, 6, 7, 8, 10, and 13 require the hand
  fan to render during DRAFT_SHOP, DRAFT_AUCTION read-only, and PLACEMENT
  staging states while preserving card identity, fan ghosts, Instant staging,
  Submit count, and reserve/current split controls.
- `design/gdd/hand-ui.md` Visual Anatomy VA-1 defines a 120x180 card display,
  cost in the top-left, ATK in the top-right orange badge, HP below ATK in a
  teal gem, and readable card identity at 10-card hand compression.
- `design/gdd/hand-ui.md` acceptance criteria HU-05, HU-06, HU-13, HU-18,
  HU-19, HU-21, HU-21b, and HU-21c define the Hand UI states whose behavior
  must be preserved while typography, badge, or measurement instrumentation is
  repaired.

**UX and accessibility trace**:

- `design/accessibility-requirements.md` Standard-tier row `Minimum text size -
  card text (cost, ATK, HP, keyword)` requires ATK and HP stat badges to have
  an 18px minimum floor and keyword text to have a 14px minimum floor.
- A11Y-ST-02 explicitly includes card cost, so this story applies the same
  18 CSS px floor to visible Hand UI card cost numerals or badges.
- `design/ux/hand-ui.md` requires the bottom hand fan, including 10-card
  compression, DRAFT_AUCTION read-only state, PLACEMENT staged ghosts, and
  hover or zoom inspection to preserve card cost, ATK, HP, and card text
  readability.
- `design/ux/hand-ui.md` states that card cost, ATK, HP, and type or rarity
  must remain readable at 10-card hand compression, and that ATK orange and HP
  teal remain reserved stat colors.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM evidence | **Risk**: HIGH

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and
`liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:

- `HandUiPlugin` remains the third `PresentationPlugin` sub-plugin after Card
  Animations and Board Rendering.
- Hand UI reads phase through `Res<CurrentClientPhase>` only. It must not drain
  `MessageReceiver<S2CPhaseChanged>`.
- Hand UI reads `Res<PlayerEconomyView>` for own current and reserve mana. It
  must not drain economy S2C messages directly.
- Card fan, drag sprite, fan ghosts, reserve strips, and evidence-only overlays
  remain bevy_ui presentation surfaces where UI is required.
- UI work runs in ADR-021 `PresentationSet` order and remains session-scoped
  under `ClientState::InSession`.
- Client-side card readability remediation is presentation-only. It must not
  mutate authoritative card catalog, hand ownership, phase, current mana,
  reserve mana, or placement state.

---

## Scope

### In Scope

- Verify or remediate A11Y-ST-02 card text floors for Hand UI-owned card
  surfaces:
  - DRAFT_SHOP hand fan cards at rest and hover or zoom.
  - DRAFT_AUCTION read-only hand fan cards.
  - PLACEMENT hand fan cards, active selection state, staged fan ghosts, and
    Instant fan ghosts.
  - DRAFT_INITIAL acquired-card feedback in the hand fan after authoritative
    purchase confirmation.
- Measure visible cost, ATK, HP, and keyword text at browser/WASM viewports
  `1366x768` and `1920x1080`.
- Apply a minimum floor of `18 CSS px` or browser-equivalent rendered pixels to
  visible cost, ATK, and HP numerals or badge text.
- Apply a minimum floor of `14 CSS px` or browser-equivalent rendered pixels to
  visible keyword text.
- Record explicit not-applicable entries for card types or states that do not
  have ATK, HP, or visible keyword text.
- Verify that Hand UI card text does not clip, truncate into unreadability, or
  overlap another required card element, sibling fan card, fan ghost, reserve
  strip, Submit cluster, timer, HUD zone, tooltip, or evidence overlay.
- Verify body text contrast for sampled Hand UI keyword text and stat numerals
  against their final browser/WASM composited backgrounds. Sampled pairs must
  meet at least `4.5:1`.
- Add or update focused automated coverage that exposes Hand UI card text
  metrics, card bounds, and overlap checks without relying only on manual
  visual judgment.
- Capture Hand UI browser/WASM evidence and update the exact evidence document
  listed in `## Test Evidence`.
- Preserve existing fan layout, card acquisition feedback, read-only auction
  fan behavior, PLACEMENT staging, fan ghost, reserve strip, Submit
  pre-validation, Instant staging, and input suppression semantics.

### Out of Scope

- DRAFT_INITIAL 3x3 grid slot rendering. That surface is owned by
  Shop/Auction UI Story 013.
- DRAFT_SHOP shop slot cards, DRAFT_AUCTION featured cards, DRAFT_AUCTION shop
  footer cards, auction preparing cards, and settlement cards. Those surfaces
  are owned by Shop/Auction UI Story 013.
- Final cross-surface A11Y-ST-02 browser/WASM evidence aggregation. That is
  owned by Presentation Layer Story 005 after the split implementation stories
  are complete.
- Changes to authoritative card catalog fields, card costs, ATK, HP, keywords,
  rarity, effect text, pool distribution, hand ownership, economy, auction,
  phase timing, network protocol payloads, or server authority.
- Broader UI scaling preferences, Settings or Accessibility screen work,
  colorblind palette implementation, full A11Y-ST-01 HUD text evidence, or
  A11Y-ST-03 global contrast evidence beyond sampled Hand UI card text pairs.
- Changes to `production/sprint-status.yaml`, `production/session-state/**`,
  project asset files, or `AGENTS.md`.
- QA-COND-0005 closure. This story contributes only the Hand UI slice of
  A11Y-ST-02.

---

## Acceptance Criteria

- [ ] **Hand fan text metrics exist**: GIVEN browser/WASM evidence runs at
  `1366x768` and `1920x1080`, WHEN the hand fan renders 0, 1, 2, 5, and 10
  card cases, THEN the evidence records visible cost, ATK, HP, keyword text,
  card bounds, and text bounds for every field that applies.
- [ ] **DRAFT_SHOP fan readability passes**: GIVEN the DRAFT_SHOP hand fan
  renders 10 cards and hover or zoom inspection is available, WHEN text-size
  measurements are reviewed, THEN visible cost, ATK, and HP fields are at least
  `18 CSS px`, visible keyword text is at least `14 CSS px`, and the accepted
  readable state is stated for each field.
- [ ] **DRAFT_AUCTION read-only fan readability passes**: GIVEN the
  DRAFT_AUCTION read-only fan is visible with input suppressed, WHEN
  browser/WASM measurements are reviewed, THEN dimming or read-only treatment
  does not reduce visible cost, ATK, HP, or keyword text below the required
  floors in the accepted readable state.
- [ ] **PLACEMENT fan readability passes**: GIVEN PLACEMENT fan cards are
  active, selected, staged as fan ghosts, or staged as Instant fan ghosts, WHEN
  browser/WASM measurements are reviewed, THEN required card text remains
  inside card or badge bounds and meets the required text floors in each state.
- [ ] **DRAFT_INITIAL acquisition fan feedback is readable**: GIVEN a
  DRAFT_INITIAL purchase is confirmed and the acquired card animates or settles
  into the hand fan, WHEN metrics are captured after the authoritative hand
  update is visible, THEN visible cost, ATK, HP, and keyword fields meet the
  required floors or are explicitly marked not applicable.
- [ ] **Cost floor passes**: GIVEN any visible Hand UI card cost numeral or
  badge in the measured surfaces, WHEN text-size measurements are reviewed,
  THEN the cost text measures at least `18 CSS px` or browser-equivalent
  rendered pixels at both viewports.
- [ ] **ATK floor passes**: GIVEN any visible Hand UI card ATK numeral or
  badge in the measured surfaces, WHEN text-size measurements are reviewed,
  THEN the ATK text measures at least `18 CSS px` or browser-equivalent
  rendered pixels at both viewports.
- [ ] **HP floor passes**: GIVEN any visible Hand UI card HP numeral or badge
  in the measured surfaces, WHEN text-size measurements are reviewed, THEN the
  HP text measures at least `18 CSS px` or browser-equivalent rendered pixels
  at both viewports.
- [ ] **Keyword floor passes**: GIVEN any visible Hand UI keyword text in the
  measured surfaces, WHEN text-size measurements are reviewed, THEN the keyword
  text measures at least `14 CSS px` or browser-equivalent rendered pixels at
  both viewports.
- [ ] **Not-applicable fields are explicit**: GIVEN a fixture card has no ATK,
  no HP, or no visible keyword text in a Hand UI state, WHEN the evidence table
  is reviewed, THEN that field is marked `N/A - field not present on this card
  type or state` rather than being silently omitted.
- [ ] **Dense fan state remains readable**: GIVEN the hand fan contains at
  least one long card name, one two-keyword card, one zero-cost card, and one
  card with two-digit cost, ATK, or HP, WHEN the 10-card compressed fan is
  captured, THEN visible cost, ATK, HP, and keyword text remains readable
  without clipping or unreadable truncation.
- [ ] **No Hand UI card-internal overlap**: GIVEN measured text bounds and card
  bounds, WHEN overlap checks run, THEN cost, ATK, HP, keyword text, card name,
  rarity indicator, and visible card art zones do not overlap in a way that
  makes required text unreadable.
- [ ] **No Hand UI surface-level overlap**: GIVEN measured card and UI bounds,
  WHEN overlap checks run, THEN Hand UI card text does not overlap sibling fan
  cards, staged fan ghosts, reserve strips, Submit controls, timers, HUD chips,
  tooltips, hand-full notifications, no-valid-target overlays, or browser
  evidence overlays.
- [ ] **Contrast sample passes for Hand UI card text**: GIVEN sampled
  browser/WASM foreground and composited background colors for visible cost,
  ATK, HP, and keyword text, WHEN contrast ratios are computed, THEN each
  sampled text/background pair meets at least `4.5:1`.
- [ ] **Existing Hand UI behavior is preserved**: GIVEN existing Hand UI
  regression tests run after remediation, WHEN card typography or measurement
  instrumentation is applied, THEN DRAFT_INITIAL acquisition feedback,
  DRAFT_SHOP fan display, DRAFT_AUCTION read-only fan, PLACEMENT staging, fan
  ghost, Instant staging, reserve strip, Submit pre-validation, and input
  suppression behavior remains unchanged.
- [ ] **Focused accessibility test passes**:
  `cargo test -p client --test hand_ui_card_text_stat_keyword_accessibility_test`
  passes. The target must be backed by
  `tests/integration/hand-ui/card_text_stat_keyword_accessibility_test.rs`.
- [ ] **Browser/WASM evidence exists**:
  `production/qa/evidence/hand-ui-card-text-stat-keyword-accessibility.md`
  records browser, build target, commit, capture command, fixture cards,
  viewport table, text-size table, overlap table, contrast sample table,
  screenshot capture directory, pass/fail verdict, and QA-COND-0005 impact
  statement.
- [ ] **Capture directory is populated**:
  `production/qa/evidence/captures/hand-ui-card-text-stat-keyword-accessibility/`
  contains the browser/WASM captures referenced by the evidence document for
  both required viewports and every required Hand UI card surface.
- [ ] **A11Y-ST-02 Hand UI impact is explicit**: The evidence document states
  whether the Hand UI portion of A11Y-ST-02 is implemented and evidenced, or
  which measured Hand UI surface still fails the cost, ATK, HP, keyword,
  non-overlap, or readability checks.
- [ ] **QA-COND-0005 remains open**: The evidence document states that this
  story contributes only the Hand UI slice of A11Y-ST-02 and that QA-COND-0005
  remains Open until all remaining Standard-tier rows are implemented and
  evidenced, reclassified, dependency-blocked, or accepted as risk.
- [ ] **Whitespace gate passes**: `git diff --check` passes.

---

## Implementation Notes

- Keep work local to Hand UI card rendering, Hand UI test fixtures, and Hand UI
  evidence harnesses needed for A11Y-ST-02.
- Prefer adjusting existing typography constants, theme tokens, card layout
  slots, badge sizes, hover or zoom surfaces, or Hand UI measurement markers
  over changing card data or game logic.
- If a card field is intentionally unreadable at rest and readable on hover or
  zoom, the evidence must show both states and state which state is accepted as
  the player-readable state for that field.
- If a card type has no ATK or HP, do not add fake stat text to satisfy the
  measurement table. Record the field as not applicable for that card type.
- If keyword text wraps, the wrapped line block must still meet the `14 CSS px`
  floor, remain inside the card surface, and avoid overlap with stats, fan
  neighbors, reserve strips, and card controls.
- Keep evidence-only measurement overlays, debug labels, exported bounds, or
  capture harnesses out of normal shipping UI unless they are also accepted
  player-facing accessibility affordances.
- Do not reduce DRAFT_AUCTION fan opacity as a readability workaround unless
  the read-only state remains visually distinguishable and existing input
  suppression semantics remain unchanged.

---

## QA Test Cases

- **Compressed fan card text measurement**
  - Given: Browser/WASM hand fan fixtures render 0, 1, 2, 5, and 10 cards at
    `1366x768` and `1920x1080`
  - When: text metrics are exported
  - Then: every visible cost, ATK, and HP field is at least `18 CSS px`, every
    visible keyword field is at least `14 CSS px`, and absent fields are
    explicitly recorded

- **DRAFT_SHOP and DRAFT_AUCTION fan states**
  - Given: The hand fan renders in DRAFT_SHOP and DRAFT_AUCTION read-only states
  - When: rest, hover or zoom, and read-only treatments are captured
  - Then: visible card text remains inside card or badge bounds and meets the
    required floors in the accepted readable state

- **PLACEMENT fan and ghost states**
  - Given: PLACEMENT fan cards are active, selected, staged as board-target
    ghosts, and staged as Instant fan ghosts
  - When: text metrics and bounds are exported
  - Then: visible cost, ATK, HP, and keyword fields meet their floors and do
    not overlap reserve strips, Submit controls, fan ghosts, HUD, or overlays

- **Contrast sample guard**
  - Given: Browser/WASM foreground/background samples are captured for visible
    Hand UI cost, ATK, HP, and keyword text
  - When: contrast ratios are computed after compositing
  - Then: sampled pairs meet at least `4.5:1`

- **Behavior preservation**
  - Given: existing Hand UI regressions run after typography or measurement
    changes
  - When: DRAFT_INITIAL acquisition, DRAFT_SHOP fan, DRAFT_AUCTION read-only
    fan, PLACEMENT staging, Instant staging, reserve strip, and Submit
    pre-validation paths are exercised
  - Then: existing behavior remains unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test target**:

- `tests/integration/hand-ui/card_text_stat_keyword_accessibility_test.rs`
  - Registered as `hand_ui_card_text_stat_keyword_accessibility_test`
  - Command:
    `cargo test -p client --test hand_ui_card_text_stat_keyword_accessibility_test`

**Required regression commands**:

- `cargo test -p client --test hand_ui_fan_layout_test`
- `cargo test -p client --test hand_ui_draft_initial_grid_test`
- `cargo test -p client --test hand_ui_placement_submit_core_test`
- `cargo test -p client --test hand_ui_placement_instant_staging_test`
- `cargo test -p client --test hand_ui_placement_unstaging_test`
- `cargo test -p client --test hand_ui_reserve_mana_strip_test`
- `cargo test -p client --test hand_ui_submit_prevalidation_test`
- `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
- `cargo check -p client`
- `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/hand-ui-card-text-stat-keyword-accessibility.md`

**Required browser/WASM capture artifact directory**:

- `production/qa/evidence/captures/hand-ui-card-text-stat-keyword-accessibility/`

**Required browser/WASM evidence contents**:

- Browser, build target, commit, capture command, fixture source, and UI scale.
- Viewports: `1366x768` and `1920x1080`.
- Fixture cards covering minion, structure, spell or instant, zero-cost,
  two-digit stat, long-name, no-keyword, and multi-keyword cases.
- Surface table covering DRAFT_INITIAL acquired-card fan feedback, DRAFT_SHOP
  hand fan, DRAFT_AUCTION read-only hand fan, and PLACEMENT active/staged fan
  states.
- Text-size table for cost, ATK, HP, and keyword text by Hand UI surface and
  viewport.
- Explicit not-applicable entries for fields absent from a card type or state.
- Overlap table comparing text bounds, card bounds, fan neighbors, reserve
  strips, Submit controls, HUD zones, and adjacent overlays.
- Contrast sample table for visible cost, ATK, HP, and keyword text.
- Browser/WASM capture links under the required capture directory.
- Focused automated test command output summary.
- Regression command output summary.
- A11Y-ST-02 Hand UI impact statement.
- QA-COND-0005 impact statement confirming the condition remains Open.

**QA-COND-0005 impact statement required in evidence**:

Story 015 implements and evidences the Hand UI slice of A11Y-ST-02 for card
cost, ATK, HP, and keyword text floors across Hand UI-owned card fan surfaces.
It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all
remaining Standard-tier rows are implemented and evidenced, reclassified,
dependency-blocked, or accepted as risk.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [Story 002](story-002-fan-layout-formula.md) - Complete; provides
  fan layout formulas and 10-card compression behavior.
- Depends on: [Story 004](story-004-draft-initial-grid.md) - Complete; provides
  DRAFT_INITIAL acquired-card fan feedback that this story must preserve.
- Depends on: [Story 005](story-005-placement-submit-core.md) - Complete;
  provides PLACEMENT Submit and staging baseline.
- Depends on: [Story 007](story-007-placement-instant-staging.md) - Complete;
  provides Instant fan staging semantics.
- Depends on: [Story 008](story-008-placement-unstaging.md) - Complete;
  provides fan ghost correction paths.
- Depends on: [Story 010](story-010-submit-prevalidation.md) - Complete;
  provides Submit pre-validation behavior that this story must preserve.
- Depends on: [Story 011](story-011-reserve-mana-strip.md) - Complete; provides
  reserve/current split controls that must not overlap Hand UI card text.
- Depends on: [Story 014](story-014-placement-staged-disclosure-accessibility.md)
  - Complete; provides A11Y-ST-14 staged disclosure evidence that this story
  must preserve.
- Depends on: ADR-002, ADR-019, and ADR-021 Accepted.
- Unlocks: Presentation Layer Story 005 final A11Y-ST-02 cross-surface
  browser/WASM evidence after Shop/Auction UI Story 013 is also implemented.

## Blockers

None.

## Performance Budget

No gameplay-loop performance impact is expected from Hand UI typography
adjustments or evidence-only measurement instrumentation. Any production
remediation must preserve ADR-021 presentation guardrails: steady-state
presentation work remains below 1 ms per frame and phase-boundary spikes remain
below 3 ms. The implementation must not add per-frame entity creation, extra
Lightyear message drains, card catalog scans, texture uploads, or persistent
debug overlays.

## No Open Questions

None.
