# Story 018: Auction Featured Card Leading / Losing State Visual

> **Epic**: Shop / Auction UI
> **Status**: Draft; producer-decision-4 captured by PROMPT 967 (2026-05-16);
> `/story-readiness` re-run required before `/dev-story`
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Slug**: `S12-UX-AUCTION-LEAD-LOSS-STATE-001`
> **Authoring**: PROMPT 881 (2026-05-14) on worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-auction-draft-layout-story-authoring`,
> branch `story/s14-auction-draft-layout-story-authoring`, source-of-truth
> `origin/main@51e6228` (PROMPT 871 `qa(s13): /story-done S13-TWO-CLIENT-RUNTIME-HARNESS-001`).

## Status / No-Claim Banner

This story file began as **authoring only** and now also carries the
PROMPT 967 producer-decision-4 paperwork capture. It is a Sprint 14
candidate row drawn from the
[UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
Tier 1 Should adjacent row (effort 0.5d; pairs with rank 10
`S11-UX-AUCTION-FEATURED-CARD`; net-new; producer decision captured by
PROMPT 967 before `/dev-story`) and the
[PROMPT 802 Expert UI Layout Audit](../../../reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md)
§3.6 A7. Authoring this story:

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
- PROMPT 967 **does** resolve PROMPT 802 producer-decision-4
  (visual language for leading / losing) as a static token-colored
  border-frame extension of the Story 016 `AuctionFeaturedCardFrame`.
  This paperwork-only decision capture does **not** invoke
  `/dev-story`; `/story-readiness` must re-run before implementation.

Sprint 14 activation, if and when it happens, must re-state every
accept-risk disposition above on the activation artifact, and must
not flip any of them to `closed` without a separate scoped sprint and
gate-check evidence.

---

## Context

**Sprint Gate**: Sprint 14+ Polish UI Clean-Pass candidate. Scope is
**layout / composition / visual state** for the featured auction-up
card leading / losing indicator only. This is **not** final-art /
asset-production work and **not** an auction logic change.

**Source audit**: PROMPT 802 §3.6 A7 -- the leading / losing
feedback state (which player is currently winning the auction) is
text-rendered, with no color / border state on the featured card
itself. Friend-game viewers cannot tell at a glance whether the
local player is currently winning the auction; the affordance is
buried in a text label that competes with the bid cluster, gold
counters, and timer for attention.

**Producer decision captured** (PROMPT 802 §9 producer-decision-4):
PROMPT 967 selects **Option A -- static token-colored border-frame**
on the featured card. The implementation extends the Story 016
`AuctionFeaturedCardFrame` primitive and changes only the frame color
by auction state:

- **Leading**: `SEMANTIC_SUCCESS` (`#27AE60`) border/frame color.
- **Losing**: `SEMANTIC_ERROR` (`#EB5757`) border/frame color.
- **Neutral / pre-bid**: retain the existing Story 016 `ACCENT`
  (`#F2C94C`) frame color.

No color pulse, required tween, animated chevron, leader badge, or
new art is selected. If implementation discovers a true tied state,
it may use `SEMANTIC_WARNING` (`#F2994A`) as the tied-state token,
but the story acceptance contract remains the three authored states
(leading / losing / neutral). The existing leader-state text remains
the colorblind fallback.

**Friend-game scope**: this story is for the friend-game product
showcase. Standard-tier accessibility (color-only feedback,
colorblind modes) remains **out of scope** under `QA-COND-0005`.
The text rendering of the leader state is **preserved** alongside
the new visual indicator so colorblind users continue to read the
existing text fallback; this story does **not** claim Standard-tier
colorblind conformance, but it **does** preserve the text fallback.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**UI Clean-Pass Roadmap**: `docs/ux/ui-clean-pass-roadmap.md` Tier 1
Should adjacent row (pairs with rank 10
`S11-UX-AUCTION-FEATURED-CARD`)
**Source Audit**: `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §3.6 A7
**Requirement**: `TR-SAU-005`, `TR-SAU-006`
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
- Do not drain `MessageReceiver<S2CAuctionBidAccepted>` or
  `MessageReceiver<S2CAuctionBidRejected>` in this story's
  leading / losing state subsystem unless already done by Story 005
  / Story 006. Read the shared auction state resource populated by
  Story 005 / Story 006.
- Send `C2SPlaceBid` only as player intent. Never mutate
  authoritative price, leader, gold, reservation, ownership, or
  protocol state from local focus, hover, click, or keyboard input.
- Use Bevy 0.18 Required Components API for all UI entities: `Node`,
  `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`,
  `ImageNode`, and `ChildOf` where needed. Do not use `NodeBundle`,
  `TextBundle`, `UiImage::new()`, `Parent`,
  `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Preserve pre-pooled auction panel, featured card, bid cluster, and
  leader-state text entities. Do not rebuild the featured-card frame
  per frame.
- Preserve Story 004 (auction activation / preparing state),
  Story 005 (bid buttons / affordability / in-flight),
  Story 006 (accepted / rejected feedback),
  Story 011 (bid target size / focus evidence), and Story 013
  (card-text accessibility) contracts.
- If
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md)
  has landed, this story **extends the featured-card frame primitive
  authored there**. It does not re-author the frame geometry.

---

## Scope

### In Scope

- Add a leading / losing visual state on the featured auction-up
  card that is observable at a glance from a friend-game showcase
  capture. The visual language is the PROMPT 967 producer-picked
  static token-colored border frame on the Story 016
  `AuctionFeaturedCardFrame`.
- Drive the state from the existing shared auction-state resource
  populated by Story 005 / Story 006: when the local player matches
  the current leader, the featured card carries the "leading" state;
  when the local player does not match the current leader, the
  featured card carries the "losing" state; before any bid is
  placed, the featured card carries a neutral / pre-bid state.
- Preserve the existing text-rendered leader label. The text state
  is the colorblind fallback and must remain readable.
- Drive the state through the ADR-021 `StateSync` system phase,
  consuming the existing shared resource; do not add a parallel
  drain.
- Establish smooth state transitions where appropriate. PROMPT 967
  does not require a tween; if a worker elects to add one, it must
  not exceed the ADR-021 3 ms phase-boundary guardrail and must
  remain inside the `AnimationTick` phase.
- Add or update test-observable UI state so automated tests can
  assert leading vs losing vs neutral state against stable marker
  components, observe the visual indicator's binding to the shared
  auction-state resource, and assert the text fallback persistence.
- Capture browser/WASM evidence at both 1920 x 1080 and 1366 x 768
  showing the three states (leading / losing / neutral) with the
  featured card, bid cluster, gold counters, timer, HUD non-
  occlusion, and hand-tray non-occlusion.

### Out of Scope

- Do **not** re-litigate the PROMPT 967 producer decision during
  `/dev-story`; implement the static token-colored border-frame
  decision unless `/story-readiness` identifies a contradiction.
- Do **not** finalize replacement chrome art (`PAW-TD-003-a`
  accept-risk preserved). The visual state is a layout / composition
  primitive (token-colored border), not new chrome art.
- Do **not** change bid increment amounts, current price calculation,
  free-gold calculation, server validation, settlement behavior, or
  protocol surface.
- Do **not** change Story 005 / Story 011 bid target size, focus
  ring, or one-send semantics.
- Do **not** add `ZIndex` / `GlobalZIndex` here; this story consumes
  the layer ordering produced by Tier 0 story `S11-TD-UI-ZINDEX-LAYERS`
  (rank 1). If
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md)
  has landed, this story extends its featured-card frame primitive.
- Do **not** remove the existing leader-state text. It is preserved
  as the colorblind fallback.
- Do **not** claim Standard-tier colorblind conformance or close
  `QA-COND-0005`.
- Implementation work must not modify `production/sprint-status.yaml`,
  `production/session-state/**`, `AGENTS.md`, or unrelated epics.
  PROMPT 967 is the separate authorised paperwork capture that
  updates shared trackers for this producer decision only.
- Do **not** advance `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or PROMPT 761 Polish->Release gate-check.

---

## Acceptance Criteria

- [ ] The featured auction-up card carries a leading visual state
  whenever the shared auction-state resource reports the local
  player as the current leader, observable via a stable marker
  component in automated tests.
- [ ] The featured auction-up card carries a losing visual state
  whenever the shared auction-state resource reports another player
  as the current leader and the local player has placed at least one
  bid, observable via a stable marker component.
- [ ] The featured auction-up card carries a neutral / pre-bid
  visual state whenever no bid has been placed yet, observable via a
  stable marker component.
- [ ] The leading / losing / neutral states are mutually exclusive
  and the active state is asserted by a strict equality test against
  the marker component.
- [ ] The text-rendered leader state is preserved alongside the
  visual indicator and remains readable in all three states.
- [ ] State transitions are driven by the ADR-021 `StateSync` phase
  reading from the existing shared auction-state resource (no
  parallel `MessageReceiver` drain added).
- [ ] Any tween used to express the state remains inside the
  `AnimationTick` phase and the phase-boundary spike stays under
  the ADR-021 3 ms guardrail.
- [ ] Story 004 / Story 005 / Story 006 / Story 011 / Story 013
  contracts remain unchanged.
- [ ] Browser/WASM evidence shows the three states (leading,
  losing, neutral) at 1920 x 1080 and 1366 x 768; alongside the
  featured card, bid cluster, gold counters, timer; HUD non-
  occlusion and hand-tray non-occlusion preserved; the colorblind-
  fallback text remains readable in every captured state.
- [ ] The evidence document includes an explicit no-claim banner
  preserving `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-002-a`,
  `PAW-TD-003-a`, `S8-QA-001-W1`, and PROMPT 761 Polish->Release
  gate-check, plus an explicit "Standard-tier colorblind
  conformance not claimed" line.
- [ ] `git diff --check` passes.

---

## Implementation Notes

- The producer decision is recorded by PROMPT 967 before this story
  enters `/dev-story`: extend the Story 016 `AuctionFeaturedCardFrame`
  as a static token-colored border frame. Use `SEMANTIC_SUCCESS` for
  leading, `SEMANTIC_ERROR` for losing, and the existing `ACCENT`
  frame for neutral / pre-bid. Do not introduce a required pulse,
  badge, chevron, or new art.
- The Tier 0 `S11-TD-UI-ZINDEX-LAYERS` story owns `GlobalZIndex`
  layer constants; this story consumes them.
- The Tier 0 `S11-TD-UI-FONT-CONSTANTS` story owns shared font-size
  constants. PROMPT 967 does not pick a leader badge, so no new badge
  label font size is required.
- The Tier 0 `S11-TD-UI-FLEX-STRIPS` story owns spacing-scale
  constants for any border/frame thickness derived from the design
  spec. Prefer the existing Story 016 3 px frame thickness unless
  `/story-readiness` requires a different tokenized value.
- The Tier 0 `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` story owns the
  viewport-invariant test harness used by this story's
  leading / losing / neutral state assertion test at 1920 x 1080 and
  1366 x 768.
- The Tier 0 `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` story authors the
  color tokens consumed here. Badge size and tween duration inputs
  are intentionally unused by the PROMPT 967 decision.
- If
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md)
  has landed, this story extends the Story 016 featured-card frame
  primitive. The two stories must not collide on the same primitive;
  Story 016 should land first. If the two run in parallel, this
  story rebases on the Story 016 frame primitive.
- The colorblind-fallback text is the existing leader-state text
  rendered by Story 005 / Story 006. Preserving it is sufficient for
  this story; broader colorblind / Standard-tier work is out of
  scope.

## Performance Budget

No gameplay-loop performance impact expected. The leading / losing /
neutral state is set on `S2CAuctionBidAccepted` /
`S2CAuctionBidRejected` via the existing shared auction-state
resource. Steady-state UI updates must remain O(1). Any tween used
to express the state must stay under the ADR-021 3 ms phase-
boundary guardrail. Presentation steady-state must remain under 1
ms per frame.

---

## QA Test Cases

- **Leading state (automated)**
  - Given: the shared auction-state resource reports the local
    player as the current leader
  - When: the featured card marker component is queried
  - Then: the featured card carries the leading state and the
    losing / neutral states are absent

- **Losing state (automated)**
  - Given: the shared auction-state resource reports another player
    as the current leader and the local player has placed at least
    one bid
  - When: the featured card marker component is queried
  - Then: the featured card carries the losing state and the
    leading / neutral states are absent

- **Neutral state (automated)**
  - Given: no bid has been placed yet
  - When: the featured card marker component is queried
  - Then: the featured card carries the neutral state and the
    leading / losing states are absent

- **State transitions (automated)**
  - Given: a sequence of `S2CAuctionBidAccepted` and
    `S2CAuctionBidRejected` messages exercising leader changes
  - When: the marker component is observed across frames
  - Then: the active state transitions strictly between leading /
    losing / neutral with no intermediate "two states active" frame

- **Colorblind fallback (automated + browser/WASM evidence)**
  - Given: any of the three states is active
  - When: the leader-state text is queried
  - Then: the text remains readable, is not removed, and matches the
    Story 005 / Story 006 evidence

- **Tween budget (automated)**
  - Given: a state transition triggers a tween
  - When: the tween's phase-boundary frame time is measured
  - Then: the spike stays under 3 ms per ADR-021

- **Viewport non-occlusion (browser/WASM evidence)**
  - Given: each of the three states is active at 1366 x 768 and at
    1920 x 1080
  - When: the panel is captured
  - Then: the featured card, the bid cluster, the gold counters, the
    timer, the colorblind-fallback text, the HUD, and the hand tray
    are visible without clipping or overlap

- **Story 004 / 005 / 006 / 011 / 013 regression**
  - Given: existing auction activation, bid affordability, accepted
    / rejected, bid target size + focus, and card-text readability
    paths are exercised
  - When: each path is replayed against the new leading / losing /
    neutral state visual
  - Then: the behavior and evidence remain unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test targets (paths suggested; final names to
be finalized at `/story-readiness`)**:

- `tests/integration/shop_auction_ui/auction_lead_loss_state_test.rs`
  - Registered as `shop_auction_ui_auction_lead_loss_state_test`
  - Command: `cargo test -p client --test shop_auction_ui_auction_lead_loss_state_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_activation_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_feedback_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-auction-lead-loss-state-<YYYY-MM-DD>.md`

**Required browser/WASM capture contents**:

- Featured card in the leading state at 1920 x 1080 and 1366 x 768.
- Featured card in the losing state at 1920 x 1080 and 1366 x 768.
- Featured card in the neutral / pre-bid state at 1920 x 1080 and
  1366 x 768.
- Colorblind-fallback leader-state text readable in every captured
  state.
- Bid cluster visible alongside in each state.
- Gold counters visible alongside in each state (Story 017
  composition preserved if it has landed; otherwise current layout
  is captured for baseline).
- Timer visible.
- HUD non-occlusion.
- Hand-tray non-occlusion.
- The producer-picked visual language (border vs color pulse vs
  badge) is recorded once at the head of the evidence document.

**No-claim banner required in evidence**:

Story 018 implements auction leading / losing / neutral state visual
on the featured card only. It does **not** claim Standard-tier
colorblind conformance and does **not** advance `QA-COND-0005`
(Standard-tier accessibility), `QA-COND-0006` (playtest / fun-
hypothesis validation), `PAW-TD-002-a` / `PAW-TD-003-a`
(placeholder PNG accept-risk), `S8-QA-001-W1` (two-client GAME_OVER
closure), the PROMPT 761 Polish->Release gate-check, or any
release-readiness claim. All conditions remain accept-risk / open
per their existing dispositions.

**Status**: [ ] Draft; producer-decision-4 resolved by PROMPT 967.
`/story-readiness` must re-run before `/dev-story`.

---

## Dependencies

- Depends on: `S11-TD-UI-ZINDEX-LAYERS`
  ([UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  rank 1; Tier 0 Must; foundational) -- provides shared
  `GlobalZIndex` constants. **Must land before this story
  implements.**
- Depends on: `S11-TD-UI-FONT-CONSTANTS`
  (roadmap rank 2; Tier 0 Must) -- provides shared font-size
  constants if the leader-badge candidate is picked.
- Depends on: `S11-TD-UI-FLEX-STRIPS`
  (roadmap rank 3; Tier 0 Must) -- provides shared spacing-scale
  constants for badge / outline thickness.
- Depends on: `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`
  (roadmap rank 4; Tier 0 Must) -- provides the viewport-invariant
  test harness used by this story's leading / losing / neutral state
  assertion test.
- Depends on: `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  (roadmap rank 6; Tier 0 Must) -- provides the numeric inputs
  (outline thickness, color tokens, badge size, tween durations).
  **Must be authored before this story implements.**
- Depends on: PROMPT 802 §9 producer-decision-4 -- **Resolved by
  PROMPT 967** as a static token-colored border-frame extension of
  `AuctionFeaturedCardFrame`: leading uses `SEMANTIC_SUCCESS`, losing
  uses `SEMANTIC_ERROR`, neutral / pre-bid retains `ACCENT`.
- Depends on:
  [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md)
  -- Complete; provides `ShopAuctionUiPlugin`, panel roots, and
  shared formula scaffolding.
- Depends on:
  [Story 004](story-004-auction-panel-activation-and-preparing-state.md)
  -- Complete; provides active auction panel and timer state.
- Depends on:
  [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md)
  -- Complete; provides bid cluster behavior, in-flight semantics,
  and the shared auction-state resource this story consumes.
- Depends on:
  [Story 006](story-006-auction-accepted-rejected-feedback.md)
  -- Complete; provides accepted / rejected feedback behavior and
  authoritative leader updates this story consumes.
- Depends on:
  [Story 011](story-011-auction-bid-target-size-and-focus-evidence.md)
  -- Complete; provides bid target 44 x 44 CSS px and focus ring
  contract this story must preserve.
- Depends on:
  [Story 013](story-013-card-text-stat-keyword-accessibility.md)
  -- Ready; provides the card-text / stat / keyword readability
  contract this story must preserve.
- Sibling-of:
  [`story-016-auction-featured-card.md`](story-016-auction-featured-card.md)
  -- this story extends the Story 016 featured-card frame primitive.
  **Story 016 should land first.** If the two run in parallel, this
  story must rebase on the Story 016 frame primitive.
- Pairs with:
  [`story-017-auction-free-gold-counters.md`](story-017-auction-free-gold-counters.md)
  -- composition preserved if Story 017 has landed; otherwise current
  layout captured for baseline.
- Depends on: `design/ux/shop-auction-ui.md` for DRAFT_AUCTION panel
  layout, focus, and non-occlusion requirements.
- Depends on: ADR-013, ADR-021 Accepted.
- Unlocks: layout / composition / visual state portion of the auction
  leading / losing affordance for the friend-game product showcase.
  Does **not** unlock `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or Polish->Release gate-check closure.

## Blockers

- Sprint 14 has not been activated. This story is a Sprint 14+
  candidate and is **blocked until Sprint 14 is opened**, the Tier 0
  foundational stories named above land, and `/story-readiness` is
  run against this story file.
- PROMPT 802 §9 producer-decision-4 is resolved by PROMPT 967. No
  producer-decision blocker remains; re-run `/story-readiness` against
  this updated story before `/dev-story`.
- `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` has landed and provides the
  `ACCENT`, `SEMANTIC_SUCCESS`, `SEMANTIC_ERROR`, and optional
  `SEMANTIC_WARNING` tokens consumed by this decision. No badge-size
  or required-tween input remains.

## Completion Notes

**Completed**: Not yet (Draft).
**Criteria**: 0 / 11 (story authoring only; no implementation).
**Deviations**: None at authoring time.
**Test Evidence**: To be captured at implementation time per the
Test Evidence section above.
**Code Review**: To be run at `/story-done` time per the lean review
mode default; PROMPT 881 authoring does **not** run code review.

---

## Authoring Trail

- PROMPT 881 (2026-05-14) -- story authored at this path against
  `origin/main@51e6228`. Slug `S12-UX-AUCTION-LEAD-LOSS-STATE-001`
  recorded per
  [UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  Tier 1 Should adjacent row (effort 0.5d; net-new) and PROMPT 802
  §3.6 A7. Sprint 14 NOT activated. No implementation. No
  `/story-readiness` / `/dev-story` / `/story-done` invocation by
  this authoring prompt. Producer-decision-4 visual-language choice
  NOT bound by this authoring prompt -- recorded on activation
  artifact.
- PROMPT 967 (2026-05-16) -- paperwork-only producer-decision-4
  capture. Decision: static token-colored border-frame extension of
  the Story 016 `AuctionFeaturedCardFrame`; leading =
  `SEMANTIC_SUCCESS`, losing = `SEMANTIC_ERROR`, neutral / pre-bid =
  existing `ACCENT`. No code, tests, Cargo, gate, smoke, release, or
  final-art claim. `/story-readiness` re-run is required next.
