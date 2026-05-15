# Story 017: Auction Free-Gold Counters Layout and Readability

> **Epic**: Shop / Auction UI
> **Status**: Draft (Sprint 14+ candidate; NOT activated by this story authoring)
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Slug**: `S11-UX-AUCTION-FREE-GOLD-COUNTERS`
> **Authoring**: PROMPT 881 (2026-05-14) on worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-auction-draft-layout-story-authoring`,
> branch `story/s14-auction-draft-layout-story-authoring`, source-of-truth
> `origin/main@51e6228` (PROMPT 871 `qa(s13): /story-done S13-TWO-CLIENT-RUNTIME-HARNESS-001`).

## Status / No-Claim Banner

This story file is **authoring only**. It is a Sprint 14+ candidate row
drawn from the
[UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
Tier 1 Should adjacent row (effort 0.5d; pairs with rank 10
`S11-UX-AUCTION-FEATURED-CARD`) and the
[PROMPT 802 Expert UI Layout Audit](../../../reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md)
§3.6 A3. Authoring this story:

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
- Does **not** claim final-art / asset-production completion
  (`PAW-TD-002-a` / `PAW-TD-003-a` accept-risk preserved).
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
**layout / composition / typography / readability** for the auction
free-gold counters only (interest counter and refunded-bid counter).
This is **not** final-art / asset-production work and **not** an
economy logic change.

**Source audit**: PROMPT 802 §3.6 A3 -- the free-gold (interest /
refunded-bid) counters are wired but their on-screen placement
relative to the bid-button cluster has not been UX-reviewed. Players
must be able to read at a glance how much free gold they have, where
it came from, and whether they can afford the next bid increment
without scanning across the screen.

**Free-gold model**: per ADR-019 + `design/gdd/shop-auction-ui.md`,
`local_free_gold = gold - reserved_gold`. Interest accrual and
refunded-bid restoration update the value through `S2CGoldBroadcast`
and `S2CGoldUpdate`; the auction panel surfaces the resulting free-
gold counter alongside the bid cluster.

**Friend-game scope**: this story is for the friend-game product
showcase. Standard-tier accessibility remains **out of scope** under
`QA-COND-0005`. Hit-target conformance is **not** advanced by this
row. Final-art replacement (`PAW-TD-003-a`) remains a separate sprint
scope and is **not** addressed here.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**UI Clean-Pass Roadmap**: `docs/ux/ui-clean-pass-roadmap.md` Tier 1
Should adjacent row (pairs with rank 10
`S11-UX-AUCTION-FEATURED-CARD`)
**Source Audit**: `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §3.6 A3
**Requirement**: `TR-SAU-001`, `TR-SAU-006`
**ADR Governing Implementation**:
[ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md),
[ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md),
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
- Do not drain `MessageReceiver<S2CGoldUpdate>` or
  `MessageReceiver<S2CGoldBroadcast>` in the auction free-gold
  counter subsystem. Read `Res<PlayerEconomyView>` (or equivalent
  shared resource) populated by the existing economy bridge per
  Story 005 / Story 006.
- Send `C2SPlaceBid` only as player intent. Never mutate
  authoritative gold, reservation, or auction state from local
  focus, hover, click, or keyboard input.
- Use Bevy 0.18 Required Components API for all UI entities: `Node`,
  `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`,
  `ImageNode`, and `ChildOf` where needed. Do not use `NodeBundle`,
  `TextBundle`, `UiImage::new()`, `Parent`,
  `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Preserve pre-pooled auction panel, bid cluster, free-gold counter,
  and timer entities. Do not rebuild the counter labels per frame.
- Preserve Story 005 (bid buttons / affordability / in-flight),
  Story 006 (accepted / rejected feedback / two-message gold gate),
  Story 011 (bid target size / focus evidence), and Story 013
  (card-text accessibility) contracts.

---

## Scope

### In Scope

- Compose the free-gold counters as a single readable group near the
  bid cluster: interest-derived free-gold and refunded-bid-derived
  free-gold are visually associated with the bid affordability
  decision, not scattered across unrelated panel regions.
- Establish typography hierarchy for the counters: counter numeric
  value font size > counter label font size; both consume
  `S11-TD-UI-FONT-CONSTANTS` once that Tier 0 story lands.
- Establish clear visual grouping (shared row or column container,
  consistent inter-counter spacing, optional grouping background)
  derived from `S11-TD-UI-FLEX-STRIPS` once that Tier 0 story lands.
- Preserve the existing free-gold calculation
  (`gold - reserved_gold`); this story does **not** change the
  numeric value or the message-drain pathway.
- Preserve the Story 005 / Story 006 affordability gate: counter
  values stay in sync with the bid-button affordability state.
- Preserve the Story 006 two-message gold gate: re-enable behavior
  is unchanged.
- Add or update test-observable UI state so automated tests can
  assert counter positions relative to the bid cluster, counter
  typography sizes, and counter readability against stable marker
  components without relying on manual screenshots.
- Capture browser/WASM evidence at both 1920 x 1080 and 1366 x 768
  showing the free-gold counters next to the bid cluster, the
  featured card, the timer, HUD non-occlusion, and hand-tray
  non-occlusion.

### Out of Scope

- Do **not** change the free-gold calculation, the economy resource
  ownership, or the message-drain pathway.
- Do **not** change bid increment amounts, current price calculation,
  server validation, settlement behavior, or protocol surface.
- Do **not** change Story 005 / Story 011 bid target size, focus
  ring, or one-send semantics.
- Do **not** finalize replacement chrome art (`PAW-TD-003-a`
  accept-risk preserved). This is layout / composition / typography
  only.
- Do **not** add `ZIndex` / `GlobalZIndex` here; this story consumes
  the layer ordering produced by Tier 0 story `S11-TD-UI-ZINDEX-LAYERS`
  (rank 1).
- Do **not** add the leading / losing border-state visual; that is
  the scope of
  [`story-018-auction-lead-loss-state.md`](story-018-auction-lead-loss-state.md).
- Do **not** redesign the featured-card surface; that is the scope of
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md).
- Do **not** modify `production/sprint-status.yaml`,
  `production/session-state/**`, `AGENTS.md`, or unrelated epics.
- Do **not** advance `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or PROMPT 761 Polish->Release gate-check.

---

## Acceptance Criteria

- [ ] The free-gold counters (interest-derived and refunded-bid-
  derived) are composed as a single readable group with a shared
  container and consistent inter-counter spacing, observable via a
  stable marker component in automated tests.
- [ ] The free-gold counter group is anchored within the auction
  panel adjacent to the bid cluster (panel-relative offset asserted
  against a documented tolerance) so that the affordability decision
  reads left-to-right (or top-to-bottom) without screen-scanning.
- [ ] Counter typography hierarchy is: numeric value font size >
  label font size; assertions are numeric.
- [ ] At 1920 x 1080 and 1366 x 768 the counters are fully visible,
  do not clip against the panel, do not overlap the bid cluster, the
  featured card, the timer, or any settlement state.
- [ ] The free-gold value remains `gold - reserved_gold` per ADR-019;
  the numeric value matches existing Story 005 / Story 006 evidence
  on every captured frame.
- [ ] Story 005 / Story 006 / Story 011 contracts remain unchanged:
  bid affordability gate, accepted / rejected feedback, two-message
  gold re-enable gate, one-send `C2SPlaceBid`, bid target 44 x 44
  CSS px, focus order +1 / +3 / +5, focus ring visibility, hidden
  disabled focus behavior, and `YOU ARE LEADING` replacement all
  remain identical to current evidence.
- [ ] Browser/WASM evidence shows the free-gold counter group at
  1920 x 1080 and 1366 x 768, alongside the bid cluster, the
  featured card, and the timer; HUD non-occlusion and hand-tray
  non-occlusion preserved.
- [ ] The evidence document includes an explicit no-claim banner
  preserving `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-002-a`,
  `PAW-TD-003-a`, `S8-QA-001-W1`, and PROMPT 761 Polish->Release
  gate-check.
- [ ] `git diff --check` passes.

---

## Implementation Notes

- The Tier 0 `S11-TD-UI-ZINDEX-LAYERS` story owns `GlobalZIndex`
  layer constants; this story consumes them. If the constant module
  is not present at implementation time, this story is blocked.
- The Tier 0 `S11-TD-UI-FONT-CONSTANTS` story owns shared font-size
  constants. Fallback rule: local placeholder constants are
  acceptable if the module has not landed, recorded as follow-on
  cleanup. The numeric hierarchy (value > label) is the binding
  contract.
- The Tier 0 `S11-TD-UI-FLEX-STRIPS` story owns spacing-scale
  constants for the counter group container and inter-counter
  spacing. Fallback rule: local placeholder constants are acceptable
  if the module has not landed, recorded as follow-on cleanup.
- The Tier 0 `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` story owns the
  viewport-invariant test harness. Use it for the counter-group
  position / typography assertions across 1920 x 1080 and 1366 x 768.
- The Tier 0 `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` story authors the
  numeric inputs (counter group padding, inter-counter gap, label /
  value font sizes). If the spec has not landed, this story is
  blocked.
- If
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md)
  has not landed, the counter group anchor should not assume a
  promoted featured-card geometry; the auction-panel relative offset
  must be documented against current geometry and re-asserted
  against the new geometry once Story 016 lands.

## Performance Budget

No gameplay-loop performance impact expected. Counters update on
`S2CGoldBroadcast` / `S2CGoldUpdate` via the existing economy
bridge. Steady-state UI updates must remain O(1), with no per-frame
counter rebuild and no allocation-heavy text edit. Presentation
steady-state must remain under 1 ms per frame and phase-boundary
spikes must stay under the ADR-021 3 ms guardrail.

---

## QA Test Cases

- **Counter group composition (automated)**
  - Given: DRAFT_AUCTION is active and the bid cluster is visible
  - When: the free-gold counter group container is queried
  - Then: the counter group is a single container with the interest
    counter and refunded-bid counter as siblings, observable via a
    stable marker component

- **Counter typography hierarchy (automated)**
  - Given: counter labels and counter numeric values are rendered
  - When: their `TextFont` font sizes are queried
  - Then: numeric value font size > label font size; the ordering is
    asserted as a strict inequality

- **Counter adjacency to bid cluster (automated)**
  - Given: DRAFT_AUCTION panel is active
  - When: the counter-group center and the bid-cluster center are
    queried
  - Then: the panel-relative offset between the two centers is
    within a documented adjacency tolerance

- **Free-gold value preservation**
  - Given: a sequence of `S2CGoldBroadcast` and `S2CGoldUpdate`
    messages exercising interest accrual and refunded-bid restoration
  - When: the counter values are observed across frames
  - Then: the values match `gold - reserved_gold` on every frame and
    Story 005 / Story 006 affordability gate behavior is unchanged

- **Viewport non-occlusion (browser/WASM evidence)**
  - Given: DRAFT_AUCTION is active at 1366 x 768 and at 1920 x 1080
  - When: the panel is captured
  - Then: the counter group, the bid cluster, the featured card, the
    timer, the HUD, and the hand tray are visible without clipping
    or overlap

- **Story 005 / 006 / 011 / 013 regression**
  - Given: existing bid-button affordability, accepted / rejected,
    two-message gold re-enable, bid target size + focus, and card-
    text readability paths are exercised
  - When: each path is replayed against the new counter group layout
  - Then: the behavior and evidence remain unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test targets (paths suggested; final names to
be finalized at `/story-readiness`)**:

- `tests/integration/shop_auction_ui/auction_free_gold_counters_layout_test.rs`
  - Registered as `shop_auction_ui_auction_free_gold_counters_layout_test`
  - Command: `cargo test -p client --test shop_auction_ui_auction_free_gold_counters_layout_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_feedback_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-auction-free-gold-counters-<YYYY-MM-DD>.md`

**Required browser/WASM capture contents**:

- Free-gold counter group adjacent to bid cluster at 1920 x 1080.
- Free-gold counter group adjacent to bid cluster at 1366 x 768.
- Counter values visible alongside the bid affordability state in
  both the affordable and unaffordable bid-button states.
- Counter values visible alongside `BIDDING...` in-flight state.
- Counter values visible alongside `YOU ARE LEADING` replacement
  state.
- Featured card and timer visible alongside.
- HUD non-occlusion.
- Hand-tray non-occlusion.

**No-claim banner required in evidence**:

Story 017 implements auction free-gold counter layout / composition
/ typography / readability only. It does **not** advance
`QA-COND-0005` (Standard-tier accessibility), `QA-COND-0006`
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
  `GlobalZIndex` constants. **Must land before this story
  implements.**
- Depends on: `S11-TD-UI-FONT-CONSTANTS`
  (roadmap rank 2; Tier 0 Must) -- provides shared font-size
  constants for the counter value / label hierarchy.
- Depends on: `S11-TD-UI-FLEX-STRIPS`
  (roadmap rank 3; Tier 0 Must) -- provides shared spacing-scale
  constants for the counter group container and inter-counter gap.
- Depends on: `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`
  (roadmap rank 4; Tier 0 Must) -- provides the viewport-invariant
  test harness for the counter-group bounds / hierarchy assertion
  test at 1920 x 1080 and 1366 x 768.
- Depends on: `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  (roadmap rank 6; Tier 0 Must) -- provides the numeric inputs
  (counter group padding, inter-counter gap, label / value font
  sizes).
- Depends on:
  [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md)
  -- Complete; provides `ShopAuctionUiPlugin`, panel roots, and
  shared formula scaffolding.
- Depends on:
  [Story 004](story-004-auction-panel-activation-and-preparing-state.md)
  -- Complete; provides active auction panel and timer state.
- Depends on:
  [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md)
  -- Complete; provides bid cluster behavior and free-gold
  affordability gate this story must preserve.
- Depends on:
  [Story 006](story-006-auction-accepted-rejected-feedback.md)
  -- Complete; provides accepted / rejected feedback behavior and
  two-message gold re-enable gate this story must preserve.
- Depends on:
  [Story 011](story-011-auction-bid-target-size-and-focus-evidence.md)
  -- Complete; provides bid target 44 x 44 CSS px and focus ring
  contract this story must preserve.
- Depends on:
  [Story 013](story-013-card-text-stat-keyword-accessibility.md)
  -- Ready; provides the card-text / stat / keyword readability
  contract this story must preserve adjacent to the counters.
- Depends on: `design/ux/shop-auction-ui.md` for DRAFT_AUCTION panel
  layout, focus, and non-occlusion requirements.
- Depends on: ADR-013, ADR-019, ADR-021 Accepted.
- Pairs with:
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md)
  -- if Story 016 has landed, the counter group anchor consumes the
  promoted featured-card geometry. If Story 016 has not landed at
  implementation time, the anchor is documented against current
  geometry and re-asserted against the new geometry later.
- Unlocks: layout / composition / typography portion of the auction
  free-gold counters for the friend-game product showcase. Does
  **not** unlock `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or Polish->Release gate-check closure.

## Blockers

- Sprint 14 has not been activated. This story is a Sprint 14+
  candidate and is **blocked until Sprint 14 is opened**, the Tier 0
  foundational stories named above land, and `/story-readiness` is
  run against this story file.
- If `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` has not been authored, the
  numeric inputs for counter group padding / gap / font sizes are
  undefined; this story remains blocked until the design spec
  authoring lands.

## Completion Notes

**Completed**: Not yet (Draft).
**Criteria**: 0 / 9 (story authoring only; no implementation).
**Deviations**: None at authoring time.
**Test Evidence**: To be captured at implementation time per the
Test Evidence section above.
**Code Review**: To be run at `/story-done` time per the lean review
mode default; PROMPT 881 authoring does **not** run code review.

---

## Authoring Trail

- PROMPT 881 (2026-05-14) -- story authored at this path against
  `origin/main@51e6228`. Slug `S11-UX-AUCTION-FREE-GOLD-COUNTERS`
  recorded per
  [UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  Tier 1 Should adjacent row (effort 0.5d) and PROMPT 802 §3.6 A3.
  Sprint 14 NOT activated. No implementation. No `/story-readiness`
  / `/dev-story` / `/story-done` invocation by this authoring prompt.
