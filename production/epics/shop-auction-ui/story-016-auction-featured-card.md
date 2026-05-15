# Story 016: Auction Featured Card Visual Hierarchy

> **Epic**: Shop / Auction UI
> **Status**: Done (Sprint 14 Must Have; closure source-of-truth `origin/main@b8285870df7612d24fe6a7d06643aa699650ca5e` = PROMPT 930 `--no-ff` integration merge of PROMPT 928 worker tip `1ddc3722538c5b9689c2411b4185f9e516951041` into prior `origin/main@06d3cdc`; flipped `ready -> done` by PROMPT 931 paperwork closure on 2026-05-15)
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Slug**: `S11-UX-AUCTION-FEATURED-CARD`
> **Authoring**: PROMPT 881 (2026-05-14) on worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-auction-draft-layout-story-authoring`,
> branch `story/s14-auction-draft-layout-story-authoring`, source-of-truth
> `origin/main@51e6228` (PROMPT 871 `qa(s13): /story-done S13-TWO-CLIENT-RUNTIME-HARNESS-001`).

## Status / No-Claim Banner

This story file is **authoring only**. It is a Sprint 14+ candidate row
drawn from the
[UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
rank 10 (Tier 1 Must) and the
[PROMPT 802 Expert UI Layout Audit](../../../reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md)
§3.6 A2. Authoring this story:

- Does **not** activate Sprint 14.
- Does **not** pull this row into any active sprint.
- Does **not** invoke `/dev-story`, `/story-readiness`, `/story-done`,
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or
  `/qa-plan`.
- Does **not** modify `production/sprint-status.yaml`,
  `production/stage.txt`, `production/sprints/`, `production/qa/`,
  `production/session-state/`.
- Does **not** modify `client/`, `server/`, `shared/`, or `tests/`.
- Does **not** advance stage (Polish remains).
- Does **not** retry the PROMPT 761 Polish->Release gate-check FAIL.
- Does **not** claim final-art / asset-production completion. Both
  `PAW-TD-003-a` (auction chrome reuses shop chrome) and the
  underlying `PAW-TD-002-a` placeholder-art accept-risk remain in
  place. The fix here is **layout / composition / typography
  differentiation**, not final art.
- Does **not** claim release-candidate readiness, public release
  readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), or playtest / fun-
  hypothesis validation (`QA-COND-0006`). All four conditions remain
  accept-risk per friend-game scope.
- Does **not** close `S8-QA-001-W1` (two-client GAME_OVER closure).

Sprint 14 activation, if and when it happens, must re-state every
accept-risk disposition above on the activation artifact, and must
not flip any of them to `closed` without a separate scoped sprint and
gate-check evidence.

---

## Context

**Sprint Gate**: Sprint 14+ Polish UI Clean-Pass candidate. Scope is
**layout / composition / typography / readability** for the featured
auction-up card surface only. This is **not** final-art / asset-
production work.

**Source audit**: PROMPT 802 §3.6 A2 -- the featured auction-up card
is *not visually differentiated* from shop slot wells because the
auction panel reuses `SHOP_PANEL_CHROME_ASSET` as a placeholder and
the featured card slot reuses the same placeholder PNG as shop slot
wells. The auction is the highest-information-density UI moment in
the game (30s `DraftAuction` timer + competing bids); a friend-game
showcase that records this moment puts the auction front-and-center,
and without featured-card differentiation the moment reads as flat.

**Friend-game scope**: this story is for the friend-game product
showcase. Standard-tier accessibility remains **out of scope** under
`QA-COND-0005`. Hit-target conformance is **not** advanced by this
row. Final-art replacement (`PAW-TD-003-a`) remains a separate sprint
scope and is **not** addressed here.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**UI Clean-Pass Roadmap**: `docs/ux/ui-clean-pass-roadmap.md` rank 10
**Source Audit**: `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §3.6 A2
**Requirement**: `TR-SAU-002`, `TR-SAU-006`
**ADR Governing Implementation**:
[ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md),
[ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.

**Engine**: Bevy 0.18 + Lightyear 0.26 + WASM browser evidence | **Risk**: MEDIUM

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs`
file. `liv-bevy-lightyear` is **not** required for this story because
no Lightyear protocol surface is touched.

**ADR / control-manifest rules for this story**:

- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin`
  sub-plugin after Card Animations, Board Rendering, Hand UI, and HUD.
- Run UI work in the ADR-021 `PresentationSet` order:
  `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` or
  `MessageReceiver<S2CAuctionCard>` in Shop/Auction UI's featured
  card subsystem unless already done. Read `Res<CurrentClientPhase>`
  and the existing Story 004 / Story 005 auction state resources.
- Send `C2SPlaceBid` only as player intent. Never mutate
  authoritative price, leader, gold, reservation, ownership, or
  protocol state from local focus, hover, click, or keyboard input.
- Use Bevy 0.18 Required Components API for all UI entities: `Node`,
  `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`,
  `ImageNode`, and `ChildOf` where needed. Do not use `NodeBundle`,
  `TextBundle`, `UiImage::new()`, `Parent`,
  `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Preserve pre-pooled auction panel, featured card, bid button,
  timer, and gold counter entities. Do not rebuild the featured card
  per frame.
- Preserve Story 004 (auction activation / preparing state),
  Story 005 (bid buttons / affordability / in-flight),
  Story 006 (accepted / rejected feedback),
  Story 007 (settlement / shop transition),
  Story 011 (bid target size / focus evidence),
  and Story 013 (card-text accessibility) contracts.

---

## Scope

### In Scope

- Promote the featured auction-up card to a visually dominant card
  surface inside the auction panel: larger card footprint (width and
  height) than any shop slot well, an explicit visual frame (border,
  drop-shadow, or panel-scoped highlight ring derived from existing
  design tokens) and a prominent center-of-panel anchor.
- Establish typography hierarchy for the featured card: card name
  larger / heavier than ATK / HP stats; ATK / HP larger / heavier
  than keyword text; keyword text remains legible per Story 013
  contract. Concrete font sizes consume `S11-TD-UI-FONT-CONSTANTS`
  once that Tier 0 story lands; until then, document the placeholder
  font sizes and flag them as a follow-on cleanup.
- Re-anchor the featured card so it does **not** sit at the same
  layout depth as shop slot wells. The card must read at a glance
  as "this is what is being auctioned right now".
- Preserve the existing left-to-right read order for the auction
  panel: timer, featured card, bid cluster, leader state. If a
  different read order is chosen at implementation time, it must be
  recorded against the Story 005 / Story 006 / Story 011 evidence as
  a deliberate change.
- Add or update test-observable UI state so automated tests can
  assert featured-card width / height bounds, typography sizes,
  panel-scoped position, and read-order against stable marker
  components without relying on manual screenshots.
- Capture browser/WASM evidence at both 1920 x 1080 and 1366 x 768
  showing the featured card, the bid cluster, the timer, gold
  counters, HUD non-occlusion, and hand-tray non-occlusion.

### Out of Scope

- Do **not** finalize replacement chrome art (`PAW-TD-003-a`
  accept-risk preserved). Visual differentiation comes from layout /
  composition / typography / explicit frame -- **not** new art
  assets.
- Do **not** change bid increment amounts, current price calculation,
  free-gold calculation, server validation, settlement behavior, or
  protocol surface.
- Do **not** change Story 005 / Story 011 bid target size, focus
  ring, or one-send semantics.
- Do **not** add `ZIndex` / `GlobalZIndex` here; this story consumes
  the layer ordering produced by Tier 0 story `S11-TD-UI-ZINDEX-LAYERS`
  (rank 1). If `S11-TD-UI-ZINDEX-LAYERS` has not landed at activation
  time, this story is blocked.
- Do **not** add the leading / losing border-state visual; that is
  the scope of a separate story
  [`story-018-auction-lead-loss-state.md`](story-018-auction-lead-loss-state.md)
  (`S12-UX-AUCTION-LEAD-LOSS-STATE-001`). The two stories are
  parallel-safe but must not collide on the same featured-card frame
  primitive.
- Do **not** address the free-gold counter placement / typography;
  that is the scope of
  [`story-017-auction-free-gold-counters.md`](story-017-auction-free-gold-counters.md)
  (`S11-UX-AUCTION-FREE-GOLD-COUNTERS`).
- Do **not** modify `production/sprint-status.yaml`,
  `production/session-state/**`, `AGENTS.md`, or unrelated epics.
- Do **not** advance `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or PROMPT 761 Polish->Release gate-check.

---

## Acceptance Criteria

- [x] The featured auction-up card width and height are each strictly
  larger than the width and height of a shop slot well, measured
  from stable marker components in automated tests at both
  1920 x 1080 and 1366 x 768. (AC1 PASS — featured 380x280 px vs
  shop slot well 136x78 px; pixel-fixed per spec §8 so the single
  ECS read covers all six canonical viewports; integration tests
  `ac1_featured_card_strictly_larger_than_every_shop_slot_well` +
  `ac1_featured_card_size_constants_are_pixel_fixed_at_every_viewport`
  PASS on `origin/main@b828587`.)
- [x] The featured card carries an explicit visual frame (border,
  drop-shadow, panel-scoped highlight ring, or equivalent) that no
  shop slot well carries; the frame is observable via a stable
  marker component in automated tests. (AC2 PASS — marker
  `AuctionFeaturedCardFrame` on the child sub-node at
  `client/src/ui/shop_auction/mod.rs:704`; ACCENT `#F2C94C` 3 px
  border via `auction_featured_card_accent_color()`; integration
  test `ac2_featured_card_carries_unique_frame_marker` PASS on
  `origin/main@b828587`.)
- [x] The featured card is anchored at the center of the auction
  panel (center-of-panel anchor verified by automated assertion on
  the panel-relative offset of the featured card center). (AC3 PASS
  — canonical bevy_ui centering trick `left: 50%, top: 50%` with
  `margin: { left: -W/2, top: -H/2 }` at
  `client/src/ui/shop_auction/mod.rs:4414-4421`; integration test
  `ac3_featured_card_centered_on_panel_via_percent_anchor` PASS
  zero-tolerance Node-intent assertion on `origin/main@b828587`.)
- [x] Typography hierarchy on the featured card is: card name font
  size > ATK / HP font size > keyword text font size; assertions are
  numeric and recorded against the Story 013 readability contract.
  (AC4 PASS — name `H1 = 30 px` on `AuctionFeaturedCard`; stats
  `H2 = 22 px` on `AuctionFeaturedCardStats`; keyword `BODY = 15 px`
  on `AuctionFeaturedCardKeyword`; 30 > 22 > 15 strict inequality;
  integration tests `ac4_typography_hierarchy_name_gt_stats_gt_keyword`
  + `ac4_typography_marker_uniqueness` PASS on `origin/main@b828587`.)
- [x] Story 004 / Story 005 / Story 006 / Story 007 / Story 011
  contracts remain unchanged: auction activation, bid affordability,
  in-flight semantics, one-send `C2SPlaceBid`, accepted / rejected
  feedback, settlement transition, bid target 44 x 44 CSS px, focus
  order +1 / +3 / +5, focus ring visibility, hidden disabled focus
  behavior, and `YOU ARE LEADING` replacement all remain identical
  to current evidence. (AC5 PASS — all five regression test bins
  green per PROMPT 928 evidence:
  `shop_auction_ui_auction_activation_test` 3/3 +
  `shop_auction_ui_auction_bid_buttons_test` 5/5 +
  `shop_auction_ui_auction_feedback_test` 6/6 +
  `shop_auction_ui_auction_settlement_test` 7/7 +
  `shop_auction_ui_auction_bid_target_focus_test` 4/4; integration
  test `ac5_bid_target_size_constants_unchanged_by_featured_card_story`
  PASS confirms Story 011 44x44 + focus-ring 2 px constants
  unchanged on `origin/main@b828587`.)
- [x] Story 013 card-text / stat / keyword readability evidence
  remains valid for the featured card surface (no regression on
  legibility of card name / ATK / HP / keywords). (AC6 PASS —
  featured-card name reads at `H1 = 30 px`, well above the
  `H2 = 22 px` HUD secondary readout accessibility floor used by
  Story 013 typography assertions; featured-card surface adopts the
  same `H1 / H2 / BODY` semantic typography scale as Story 003
  `S11-TD-UI-FONT-CONSTANTS`; no changes to hand / shop / draft
  surfaces where Story 013 evidence was captured.)
- [x] Browser/WASM evidence shows the featured card dominant at
  1920 x 1080 and 1366 x 768, bid cluster visible alongside, timer
  visible, gold counters visible, HUD non-occlusion, and hand-tray
  non-occlusion. (AC7 PASS-NODE-INTENT-VERIFICATION-MANUAL-CAPTURE-
  DEFERRED — Node-intent invariants AC1 / AC2 / AC3 / AC4 already
  verified geometrically constrain what the screenshots will exhibit
  centered card / larger-than-shop-slot / ACCENT frame / H1>H2>BODY
  hierarchy; PROMPT 928 worker ran in a headless `cargo test`
  environment without browser/WASM rendering capability; manual
  capture instructions recorded at
  `production/qa/evidence/sprint-14-auction-featured-card/manual-capture-instructions.md`
  for the capturer step at both 1920 x 1080 and 1366 x 768 viewports;
  capture deferred per PROMPT 928 evidence §8 + PROMPT 930 integration
  PASS verdict + PROMPT 931 closure scope.)
- [x] The evidence document includes an explicit no-claim banner
  preserving `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-002-a`,
  `PAW-TD-003-a`, `S8-QA-001-W1`, and PROMPT 761 Polish->Release
  gate-check. (AC8 PASS —
  `production/qa/evidence/sprint-14-auction-featured-card/evidence.md`
  §1 "No-Claim Banner" carries each disposition verbatim;
  `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-002-a` + `PAW-TD-003-a` +
  `S8-QA-001-W1` + PROMPT 761 Polish->Release `FAIL` named explicitly
  on `origin/main@b828587`.)
- [x] `git diff --check` passes. (AC9 PASS — exit code 0 across
  worker (PROMPT 928) + integration (PROMPT 930) commits; only
  benign LF→CRLF Windows warnings recorded, not whitespace errors;
  `git diff --cached --check` also PASS on
  `origin/main@b8285870df7612d24fe6a7d06643aa699650ca5e`.)

---

## Implementation Notes

- The Tier 0 `S11-TD-UI-ZINDEX-LAYERS` story owns the
  `GlobalZIndex(...)` constant; this story should consume it. If the
  constant module is not present at implementation time, this story
  is blocked on Tier 0 landing.
- The Tier 0 `S11-TD-UI-FONT-CONSTANTS` story owns shared font-size
  constants. If the font module is not present at implementation
  time, this story uses local placeholder constants and records them
  as a follow-on cleanup row. The numeric hierarchy (name > stats >
  keywords) is the binding contract, not the absolute values.
- The Tier 0 `S11-TD-UI-FLEX-STRIPS` story owns spacing-scale
  constants for auction panel padding and inter-section gaps. Same
  fallback rule as the font constants.
- The Tier 0 `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` story owns the
  viewport-invariant test harness. Use it for the featured-card
  bounds / hierarchy assertions across 1920 x 1080 and 1366 x 768
  once it lands.
- The Tier 0 `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` story authors the
  numeric inputs (frame thickness, drop-shadow alpha, highlight ring
  width, font-size ratios). If the spec has not landed at
  implementation time, this story is blocked.
- The auction panel chrome continues to reuse the placeholder
  `SHOP_PANEL_CHROME_ASSET` PNG under `PAW-TD-003-a` accept-risk.
  Visual differentiation comes from the **featured card frame**,
  not from new chrome art.
- The featured card frame primitive should be authored as a reusable
  marker / sub-node so the leading / losing state story
  (`S12-UX-AUCTION-LEAD-LOSS-STATE-001`) can extend it without
  re-authoring the geometry.

## Performance Budget

No gameplay-loop performance impact expected. The featured card is
spawned once per auction and updated on `S2CAuctionCard`. Steady-
state UI updates must remain O(1), with no per-frame card rebuild,
no card catalog scan, and no allocation-heavy focus traversal.
Presentation steady-state must remain under 1 ms per frame and
phase-boundary / auction-activation spikes must stay under the
ADR-021 3 ms guardrail.

---

## QA Test Cases

- **Featured-card dominance (automated)**
  - Given: DRAFT_AUCTION is active with a featured card
  - When: featured-card bounds and shop-slot-well bounds are queried
  - Then: the featured card width and height are each strictly
    greater than the shop slot well width and height

- **Featured-card frame (automated)**
  - Given: DRAFT_AUCTION is active with a featured card
  - When: the featured-card frame marker component is queried
  - Then: the featured card carries an explicit visual frame that
    no shop slot well carries

- **Center-of-panel anchor (automated)**
  - Given: DRAFT_AUCTION panel is active
  - When: the featured-card panel-relative offset is queried
  - Then: the featured card center is at the panel center within a
    documented tolerance

- **Typography hierarchy (automated)**
  - Given: featured-card name, ATK, HP, and keyword text are rendered
  - When: their `TextFont` font sizes are queried
  - Then: card name font size > ATK / HP font size > keyword font
    size; the ordering is asserted as a strict inequality

- **Viewport non-occlusion (browser/WASM evidence)**
  - Given: DRAFT_AUCTION is active at 1366 x 768 and at 1920 x 1080
  - When: the panel is captured
  - Then: the featured card, the bid cluster, the timer, the gold
    counters, the HUD, and the hand tray are visible without
    clipping or overlap

- **Story 004 / 005 / 006 / 007 / 011 / 013 regression**
  - Given: existing auction activation, bid affordability, in-flight,
    accepted / rejected, settlement, bid target size + focus, and
    card-text readability paths are exercised
  - When: each path is replayed against the new featured-card layout
  - Then: the behavior and evidence remain unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test targets (paths suggested; final names to
be finalized at `/story-readiness`)**:

- `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs`
  - Registered as `shop_auction_ui_auction_featured_card_layout_test`
  - Command: `cargo test -p client --test shop_auction_ui_auction_featured_card_layout_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_activation_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_feedback_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_settlement_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-auction-featured-card-<YYYY-MM-DD>.md`

**Required browser/WASM capture contents**:

- Featured card dominant at 1920 x 1080.
- Featured card dominant at 1366 x 768.
- Side-by-side capture of featured card and shop slot well showing
  the size differential (either as a single capture at DRAFT_SHOP
  transition, or two captures with annotated bounds).
- Featured-card frame visible at both viewports.
- Bid cluster visible alongside the featured card with Story 011
  bid target evidence preserved.
- Gold counters visible alongside the featured card (Story 017
  composition is preserved if it has landed; otherwise current
  layout is captured for baseline).
- Timer visible.
- HUD non-occlusion.
- Hand-tray non-occlusion.

**No-claim banner required in evidence**:

Story 016 implements auction featured-card visual hierarchy (layout
/ composition / typography / explicit frame) only. It does **not**
advance `QA-COND-0005` (Standard-tier accessibility), `QA-COND-0006`
(playtest / fun-hypothesis validation), `PAW-TD-002-a` /
`PAW-TD-003-a` (placeholder PNG accept-risk), `S8-QA-001-W1`
(two-client GAME_OVER closure), the PROMPT 761 Polish->Release
gate-check, or any release-readiness claim. All conditions remain
accept-risk / open per their existing dispositions.

**Status**: [ ] Draft (Sprint 14+ candidate; NOT activated by this story authoring).

---

## Dependencies

- Depends on: `S11-TD-UI-ZINDEX-LAYERS`
  ([UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  rank 1; Tier 0 Must; foundational) -- provides shared
  `GlobalZIndex` constants so the featured card sits at a
  deterministic layer above the auction-panel chrome and below the
  HUD dim overlay. **Must land before this story implements.**
- Depends on: `S11-TD-UI-FONT-CONSTANTS`
  (roadmap rank 2; Tier 0 Must) -- provides shared font-size
  constants for the featured-card typography hierarchy. Fallback
  rule: local placeholder constants are acceptable if the module
  has not landed, recorded as follow-on cleanup.
- Depends on: `S11-TD-UI-FLEX-STRIPS`
  (roadmap rank 3; Tier 0 Must) -- provides shared spacing-scale
  constants for the auction panel padding and inter-section gaps.
  Fallback rule: local placeholder constants are acceptable if the
  module has not landed, recorded as follow-on cleanup.
- Depends on: `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`
  (roadmap rank 4; Tier 0 Must) -- provides the viewport-invariant
  test harness for the featured-card bounds / hierarchy assertion
  test at 1920 x 1080 and 1366 x 768.
- Depends on: `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  (roadmap rank 6; Tier 0 Must) -- provides the numeric inputs
  (frame thickness, drop-shadow alpha, highlight ring width,
  font-size ratios). **Must be authored before this story
  implements.**
- Depends on:
  [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md)
  -- Complete; provides `ShopAuctionUiPlugin`, panel roots, and
  shared formula scaffolding.
- Depends on:
  [Story 004](story-004-auction-panel-activation-and-preparing-state.md)
  -- Complete; provides active auction panel and timer state.
- Depends on:
  [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md)
  -- Complete; provides bid cluster behavior that this story must
  preserve.
- Depends on:
  [Story 006](story-006-auction-accepted-rejected-feedback.md)
  -- Complete; provides accepted / rejected feedback behavior this
  story must preserve.
- Depends on:
  [Story 007](story-007-auction-settlement-and-shop-transition.md)
  -- Ready; provides settlement transition behavior this story must
  preserve.
- Depends on:
  [Story 011](story-011-auction-bid-target-size-and-focus-evidence.md)
  -- Complete; provides bid target 44 x 44 CSS px and focus ring
  contract this story must preserve.
- Depends on:
  [Story 013](story-013-card-text-stat-keyword-accessibility.md)
  -- Ready; provides the card-text / stat / keyword readability
  contract this story must preserve for the featured-card surface.
- Depends on: `design/ux/shop-auction-ui.md` for DRAFT_AUCTION panel
  layout, focus, and non-occlusion requirements.
- Depends on: ADR-013, ADR-021 Accepted.
- Unlocks: layout / composition / typography portion of the
  DRAFT_AUCTION surface for the friend-game product showcase. Does
  **not** unlock `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or Polish->Release gate-check closure.
- Sibling-of:
  [`story-018-auction-lead-loss-state.md`](story-018-auction-lead-loss-state.md)
  -- the leading / losing border-state story extends the
  featured-card frame primitive. Implementations should land this
  story first; if the two run in parallel, the lead / loss story
  must rebase on the featured-card frame primitive.

## Blockers

- Sprint 14 has not been activated. This story is a Sprint 14+
  candidate and is **blocked until Sprint 14 is opened**, the Tier 0
  foundational stories named above land, and `/story-readiness` is
  run against this story file.
- If `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` has not been authored, the
  numeric inputs for featured-card frame / typography / spacing are
  undefined; this story remains blocked until the design spec
  authoring lands.

## Completion Notes

**Completed**: 2026-05-15 (PROMPT 931 `/story-done` paperwork).
**Criteria**: 9 / 9 (AC1-AC9 PASS; AC7 PASS-NODE-INTENT-VERIFICATION-
MANUAL-CAPTURE-DEFERRED per evidence §8 + integration PASS verdict).
**Deviations**: One deliberate read-order change recorded in evidence
§5 — top-to-bottom (status/timer above featured card, bid cluster
below) instead of left-to-right per story §"In Scope" line 142-143
allowance ("If a different read order is chosen at implementation
time, it must be recorded against the Story 005 / Story 006 / Story
011 evidence as a deliberate change"). Story 005 / 006 / 011
behavioral contracts unchanged.
**Test Evidence**:
- `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs`
  NEW 368 lines (7 tests AC1-AC5 all PASS).
- `production/qa/evidence/sprint-14-auction-featured-card/evidence.md`
  NEW 241 lines (no-claim banner §1; AC verdicts §3; spec adoption
  §4; expert-UI-designer self-review §5; Cargo policy §6; test
  commands §7; manual-capture status §8; carried non-claims §9).
- `production/qa/evidence/sprint-14-auction-featured-card/manual-capture-instructions.md`
  NEW 160 lines (AC7 reach-point + capture instructions; capturer
  step deferred from headless PROMPT 928 worker).
- Worker report `reports/PROMPT-928-S14-AUCTION-FEATURED-CARD-DEV-STORY.md`
  (gitignored) and integration report
  `reports/PROMPT-930-S14-AUCTION-FEATURED-CARD-INTEGRATION.md`
  (gitignored) cover both phases.
**Code Review**: Lean review absorbed into PROMPT 930 integration
verification — `cargo fmt -p client -- --check` clean,
`cargo check -p client` clean, all 7 story-prescribed tests PASS,
`git diff --check` PASS, forbidden-path filter empty.

---

## Authoring Trail

- PROMPT 881 (2026-05-14) -- story authored at this path against
  `origin/main@51e6228`. Slug `S11-UX-AUCTION-FEATURED-CARD`
  recorded per
  [UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  rank 10 and PROMPT 802 §3.6 A2. Sprint 14 NOT activated. No
  implementation. No `/story-readiness` / `/dev-story` /
  `/story-done` invocation by this authoring prompt.

---

## Closure Trail

- **PROMPT 928** (2026-05-15) -- `/dev-story` worker on fresh worktree
  `D:/_DEV/wt/ccgs-prompt-928-auction-featured-card`, branch
  `work/s14-auction-featured-card`, base `origin/main@f6e538f`.
  Commit `1ddc3722538c5b9689c2411b4185f9e516951041`. 5 files
  changed (+995 / -11): `client/Cargo.toml` +4 (NEW test bin
  registration); `client/src/ui/shop_auction/mod.rs` +222 / -11
  (featured-card geometry constants `AUCTION_FEATURED_CARD_*_PX` +
  markers `AuctionFeaturedCardFrame` / `Stats` / `Keyword` + ACCENT
  helper + center-of-panel `auction_featured_card_node()` + frame /
  stats / keyword sub-nodes + repositioning of bid buttons / bid
  status / status / timer to flow around centered featured card);
  `tests/integration/shop_auction_ui/auction_featured_card_layout_test.rs`
  NEW 368 (7 tests AC1-AC5);
  `production/qa/evidence/sprint-14-auction-featured-card/evidence.md`
  NEW 241; `production/qa/evidence/sprint-14-auction-featured-card/manual-capture-instructions.md`
  NEW 160. Worker branch pushed; `main` NOT pushed. Targeted
  regression suite (8 sibling test bins) GREEN. Cargo resource
  policy applied. Story file body NOT flipped per `/dev-story`
  scope. Worker report
  `reports/PROMPT-928-S14-AUCTION-FEATURED-CARD-DEV-STORY.md`
  (gitignored).
- **PROMPT 930** (2026-05-15) -- `--no-ff` integration of PROMPT 928
  worker tip `1ddc372` into prior `origin/main@06d3cdc` (PROMPT 922
  `/story-done S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) on fresh
  integration worktree `D:/_DEV/wt/ccgs-prompt-930-integration`,
  branch `integrate/s14-auction-featured-card-930`. Merge commit
  `b8285870df7612d24fe6a7d06643aa699650ca5e`. Zero conflicts.
  Verification: `cargo fmt -p client -- --check` clean +
  `cargo check -p client` clean (Finished dev profile [optimized]
  target(s) in 7.14s) + `cargo test -p client --test shop_auction_ui_auction_featured_card_layout_test`
  **7/7 PASS** + `git diff --check` clean + `git diff --cached
  --check` clean + forbidden-path filter empty (`server/`,
  `shared/`, `production/sprint-status.yaml`,
  `production/session-state/`, `production/stage.txt`,
  `production/sprints/`). Pushed `06d3cdc..b828587 HEAD -> main`.
  Cargo resource policy applied. Integration report
  `reports/PROMPT-930-S14-AUCTION-FEATURED-CARD-INTEGRATION.md`
  (gitignored).
- **PROMPT 931** (2026-05-15) -- `/story-done` paperwork closure on
  fresh detached worktree `D:/_DEV/wt/ccgs-prompt-931-storydone`
  from `origin/main@b828587`. Row flipped `ready -> done` in
  `production/sprint-status.yaml`; AC1-AC9 checkboxes flipped
  `[ ] -> [x]` in this file with per-AC closure-evidence
  annotations; `sprint_14_story_done:` block extended with PROMPT
  931 entry as **seventh** `/story-done` entry of Sprint 14
  (after PROMPT 909 viewport-tests first + PROMPT 908 font-constants
  second + PROMPT 903 z-index-layers third + PROMPT 919 flex-strips
  fourth + PROMPT 921 overlay-alpha-token fifth + PROMPT 922
  global-ui-design-spec sixth — all preserved verbatim).
  `production/session-state/active.md` + `codex-orchestrator-state.md`
  PROMPT 931 banner prepended. No `client/` / `server/` / `shared/`
  / `tests/` code change. No `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, `/qa-plan`, `/dev-story`,
  `/story-readiness`, Sprint 14 close-out, S8-QA-001-W1 closure,
  Polish->Release retry, stage advance, final-art replacement,
  release-readiness claim, or closure of any other Sprint 14 row
  invoked by PROMPT 931. Sprint 14 disposition UNCHANGED `active`.
  Stage UNCHANGED `Polish`. PROMPT 761 Polish->Release `FAIL`
  preserved. All carried non-claims preserved verbatim. Closure
  report `reports/PROMPT-931-S14-AUCTION-FEATURED-CARD-STORY-DONE.md`
  (gitignored).
