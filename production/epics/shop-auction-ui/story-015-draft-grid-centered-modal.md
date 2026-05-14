# Story 015: Draft Initial Grid Centered Modal Layout

> **Epic**: Shop / Auction UI
> **Status**: Draft (Sprint 14+ candidate; NOT activated by this story authoring)
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05
> **Slug**: `S11-UX-DRAFT-GRID-CENTERED-MODAL`
> **Authoring**: PROMPT 881 (2026-05-14) on worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-auction-draft-layout-story-authoring`,
> branch `story/s14-auction-draft-layout-story-authoring`, source-of-truth
> `origin/main@51e6228` (PROMPT 871 `qa(s13): /story-done S13-TWO-CLIENT-RUNTIME-HARNESS-001`).

## Status / No-Claim Banner

This story file is **authoring only**. It is a Sprint 14+ candidate row
drawn from the
[UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
rank 9 (Tier 1 Must) and the
[PROMPT 802 Expert UI Layout Audit](../../../reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md)
§3.4 D1. Authoring this story:

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
  (`PAW-TD-003-a` accept-risk preserved).
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
**layout / composition / readability** for the DRAFT_INITIAL panel
only. This is **not** final-art / asset-production work.

**Source audit**: PROMPT 802 §3.4 D1 -- the draft grid is rendered
inside `draft_offering_panel` which uses `bottom_panel_node()`, the
**same node** as the shop panel, visibility-toggled. The grid lives
at the *bottom* of the screen during DRAFT_INITIAL, not as a centered
modal. The first-session "select up to 9 cards" moment reads as a
half-screen empty prototype instead of a focused decision surface.
This is a friend-game first-impression defect.

**Friend-game scope**: this story is for the friend-game product
showcase. Standard-tier accessibility (WCAG contrast ratios, >=44px
hit-targets, full keyboard navigation, screen reader support,
colorblind modes, text scaling) remains **out of scope** under
`QA-COND-0005`. Hit-target conformance is **not** advanced by this
row. Where existing accessibility evidence (Story 011, Story 012,
Story 013) overlaps the surface, it must not regress.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**UI Clean-Pass Roadmap**: `docs/ux/ui-clean-pass-roadmap.md` rank 9
**Source Audit**: `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §3.4 D1
**Requirement**: `TR-SAU-006` (panel transitions and input gating)
**ADR Governing Implementation**:
[ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md),
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
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI.
  Read `Res<CurrentClientPhase>` populated by `phase_sink_system`.
- Do not add a new C2S message, server state, or protocol field for
  the layout change.
- Use Bevy 0.18 Required Components API for all UI entities: `Node`,
  `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`,
  `ImageNode`, and `ChildOf` where needed. Do not use `NodeBundle`,
  `TextBundle`, `UiImage::new()`, `Parent`,
  `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Preserve pre-pooled DRAFT_INITIAL panel, grid, timer, Ready, and
  objective overlay entities. Do not rebuild the grid per frame.
- Preserve Story 002 (DRAFT_INITIAL purchase/ready behavior), Story
  012 (objective overlay), and Story 013 (card-text accessibility)
  contracts.

---

## Scope

### In Scope

- Re-host the DRAFT_INITIAL panel root as a centered modal-panel
  layout (`Display::Flex`, `align_items: Center`, `justify_content:
  Center`, viewport-anchored via `Val::Percent` width with a
  `max_width` cap) in the style of `result_screen.rs` (the lone
  surface PROMPT 802 §3.8 calls out as `ACCEPTABLE`).
- Separate the DRAFT_INITIAL panel root from the
  `bottom_panel_node()` shared with the shop panel, so DRAFT_INITIAL
  no longer shares spawn-order Z with the shop panel. Visibility-
  toggle preservation between DRAFT_INITIAL and DRAFT_SHOP is kept,
  but the two panels live as siblings under their own roots.
- Re-host the 3 x 3 offering grid inside the centered modal panel
  with stable column widths, stable row heights, and explicit
  spacing constants (consumed from `S11-TD-UI-FLEX-STRIPS` and
  `S11-TD-UI-FONT-CONSTANTS` once those Tier 0 stories land; until
  then, document the placeholder constants used in implementation
  and flag them as a follow-on cleanup).
- Preserve the Story 012 objective overlay copy, dismissal,
  retrieval, and non-occlusion contract. The overlay must remain
  panel-scoped and must not block HUD, hand tray, or any card slot.
- Preserve Story 002 click behavior, insufficient-gold handling,
  confirmed purchased-slot state, Ready / Retract Ready, and no
  optimistic ownership.
- Preserve Story 013 card-text / stat / keyword readability for
  DRAFT_INITIAL slot card text.
- Add or update test-observable UI state so automated tests can
  assert the panel root layout properties (display mode, alignment,
  width / max-width, child grid arrangement) without relying on
  manual screenshots.
- Capture browser/WASM evidence at both 1920 x 1080 and 1366 x 768
  showing the centered modal, the 3 x 3 grid, the timer, Ready /
  Retract Ready, HUD non-occlusion, and hand-tray non-occlusion.

### Out of Scope

- Do **not** change DRAFT_INITIAL offering generation, sort order,
  purchase validation, hand cap, gold display, Ready / Retract Ready
  messages, or phase transition behavior.
- Do **not** add a new C2S message, server state, or protocol field
  for the layout change.
- Do **not** add `ZIndex` / `GlobalZIndex` here; this story consumes
  the `ZIndex` / `GlobalZIndex` layer ordering produced by Tier 0
  story `S11-TD-UI-ZINDEX-LAYERS` (rank 1) once it lands. If
  `S11-TD-UI-ZINDEX-LAYERS` has not landed at activation time, this
  story is blocked.
- Do **not** finalize replacement chrome art (`PAW-TD-002-a` /
  `PAW-TD-003-a` accept-risk preserved). This is layout / composition
  only.
- Do **not** modify `production/sprint-status.yaml`,
  `production/session-state/**`, `AGENTS.md`, or unrelated epics.
- Do **not** advance `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or PROMPT 761 Polish->Release gate-check.
- Do **not** include Sprint 13 close-out work.

---

## Acceptance Criteria

- [ ] The DRAFT_INITIAL panel root uses `Display::Flex` with
  `align_items: Center` and `justify_content: Center`, anchored to
  the viewport via `width: Val::Percent(N)` with a `max_width:
  Val::Px(M)` cap (concrete N / M values are recorded in
  implementation and reflected in the centered-modal assertion test).
- [ ] At 1920 x 1080 the DRAFT_INITIAL panel is horizontally and
  vertically centered with both side margins and top / bottom
  margins greater than zero (no longer anchored to the bottom edge).
- [ ] At 1366 x 768 the DRAFT_INITIAL panel is horizontally and
  vertically centered, all nine offering slots are fully visible
  inside the panel, and no slot, label, timer, Ready, or Retract
  Ready control clips against the viewport.
- [ ] The DRAFT_INITIAL panel root is a sibling of the DRAFT_SHOP
  panel root (no longer the shared `bottom_panel_node()` parent).
  Visibility toggling between DRAFT_INITIAL and DRAFT_SHOP preserves
  the Story 002 / Story 003 behavior; both panels are never visible
  simultaneously.
- [ ] The 3 x 3 offering grid renders with stable column widths and
  row heights and visible inter-slot spacing; no slot overlaps another
  slot, the timer, the Ready control, or the objective overlay.
- [ ] The Story 012 objective overlay still appears on DRAFT_INITIAL
  activation, the copy is unchanged, dismissal / retrieval still work,
  and the overlay still does not occlude the HUD, the hand tray, or
  any card slot.
- [ ] Story 002 purchase behavior is unchanged: valid affordable slot
  clicks still send exactly one `C2SPurchaseCard { card_id }`,
  insufficient gold still does not send, confirmed purchases still
  show the existing bought state, and Ready / Retract Ready still
  send `C2SSignalReady { retract: false / true }`.
- [ ] Story 013 card-text / stat / keyword readability evidence
  remains valid for DRAFT_INITIAL slot card text (no regression).
- [ ] Browser/WASM evidence shows DRAFT_INITIAL panel centered at
  1920 x 1080 and 1366 x 768, the 3 x 3 grid, the timer, Ready /
  Retract Ready, HUD non-occlusion, and hand-tray non-occlusion.
- [ ] The evidence document includes an explicit no-claim banner
  preserving `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-002-a`,
  `PAW-TD-003-a`, `S8-QA-001-W1`, and PROMPT 761 Polish->Release
  gate-check.
- [ ] `git diff --check` passes.

---

## Implementation Notes

- The centered-modal pattern is already in use in
  `client/src/presentation/result_screen.rs:488` and is called out as
  the lone acceptable surface in PROMPT 802 §3.8. Mirror that pattern
  (`Display::Flex`, `width: Val::Percent(88)`, `max_width:
  Val::Px(860)`, `max_height: Val::Percent(92)`, `GlobalZIndex(...)`)
  rather than inventing a new layout idiom.
- The Tier 0 `S11-TD-UI-ZINDEX-LAYERS` story owns the
  `GlobalZIndex(...)` constant; this story should consume it. If the
  constant module is not present at implementation time, this story
  is blocked on Tier 0 landing.
- The Tier 0 `S11-TD-UI-FLEX-STRIPS` story owns spacing-scale
  constants. If the spacing module is not present at implementation
  time, this story uses local placeholder constants and records them
  as a follow-on cleanup row.
- The Tier 0 `S11-TD-UI-FONT-CONSTANTS` story owns shared font-size
  constants. Same fallback rule as the spacing constants.
- The Tier 0 `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` story owns the
  viewport-invariant test harness. This story's centered-modal
  assertion test should be authored against that harness once it
  lands; if it has not landed at implementation time, a one-off
  assertion test is acceptable as long as it survives the Tier 0
  migration.
- Splitting DRAFT_INITIAL out from `bottom_panel_node()` must not
  break the Story 003 DRAFT_SHOP layout; the DRAFT_SHOP panel
  continues to live in its existing position until a separate Sprint
  14+ shop-layout story addresses it.

## Performance Budget

No gameplay-loop performance impact expected. The panel is spawned
once at session start and the centered-modal change is a one-time
layout property update. Steady-state UI updates must remain O(1),
with no per-frame grid rebuild, no card catalog scan, and no
allocation-heavy focus traversal. Presentation steady-state must
remain under 1 ms per frame and phase-boundary panel toggles must
stay under the ADR-021 3 ms guardrail.

---

## QA Test Cases

- **Centered-modal layout properties (automated)**
  - Given: DRAFT_INITIAL is active and the offering grid is populated
  - When: the DRAFT_INITIAL panel root layout properties are queried
  - Then: `Display::Flex`, `align_items: Center`,
    `justify_content: Center`, viewport-anchored `width: Val::Percent(N)`,
    and `max_width: Val::Px(M)` are asserted against the documented
    constants

- **Grid composition (automated)**
  - Given: DRAFT_INITIAL is active with nine offered cards
  - When: the 3 x 3 grid children are enumerated
  - Then: column widths and row heights are stable, inter-slot
    spacing is greater than zero, and no slot overlaps another slot
    or the timer / Ready / objective overlay

- **Viewport non-occlusion (browser/WASM evidence)**
  - Given: DRAFT_INITIAL is active at 1366 x 768 and at 1920 x 1080
  - When: the panel is captured
  - Then: all nine slots, the timer, Ready, Retract Ready, the
    objective overlay, the HUD, and the hand tray are visible without
    clipping or overlap

- **Story 002 / Story 012 / Story 013 regression**
  - Given: existing Story 002 purchase / Ready / Retract Ready /
    PLACEMENT dismissal paths, Story 012 objective overlay, and
    Story 013 card-text readability are exercised
  - When: each path is replayed against the new centered-modal panel
  - Then: the behavior and evidence remain unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test targets (paths suggested; final names to
be finalized at `/story-readiness`)**:

- `tests/integration/shop_auction_ui/draft_initial_centered_modal_layout_test.rs`
  - Registered as `shop_auction_ui_draft_initial_centered_modal_layout_test`
  - Command: `cargo test -p client --test shop_auction_ui_draft_initial_centered_modal_layout_test`
- Regression: `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- Regression: `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
- Regression: `cargo test -p client --test shop_auction_ui_shop_panel_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-draft-grid-centered-modal-<YYYY-MM-DD>.md`

**Required browser/WASM capture contents**:

- Centered DRAFT_INITIAL panel at 1920 x 1080.
- Centered DRAFT_INITIAL panel at 1366 x 768.
- 3 x 3 grid fully visible inside the panel at both viewports.
- Timer, Ready, Retract Ready, and objective overlay non-occlusion.
- HUD non-occlusion (top strip and bottom strip visible alongside
  the centered panel).
- Hand-tray non-occlusion.
- DRAFT_INITIAL -> DRAFT_SHOP visibility toggle screenshots showing
  both panels never simultaneously visible.

**No-claim banner required in evidence**:

Story 015 implements DRAFT_INITIAL centered-modal layout only. It does
**not** advance `QA-COND-0005` (Standard-tier accessibility),
`QA-COND-0006` (playtest / fun-hypothesis validation), `PAW-TD-002-a`
/ `PAW-TD-003-a` (placeholder PNG accept-risk), `S8-QA-001-W1`
(two-client GAME_OVER closure), the PROMPT 761 Polish->Release
gate-check, or any release-readiness claim. All conditions remain
accept-risk / open per their existing dispositions.

**Status**: [ ] Draft (Sprint 14+ candidate; NOT activated by this story authoring).

---

## Dependencies

- Depends on: `S11-TD-UI-ZINDEX-LAYERS`
  ([UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  rank 1; Tier 0 Must; foundational) -- provides shared
  `GlobalZIndex` constants so the centered DRAFT_INITIAL panel sits at
  a deterministic layer above the playfield and below the HUD dim
  overlay. **Must land before this story implements.**
- Depends on: `S11-TD-UI-FLEX-STRIPS`
  (roadmap rank 3; Tier 0 Must) -- provides shared spacing-scale
  constants for the centered modal padding, inter-slot gap, and grid
  gutters. If not landed at implementation time, this story uses
  local placeholder constants and records a follow-on cleanup row.
- Depends on: `S11-TD-UI-FONT-CONSTANTS`
  (roadmap rank 2; Tier 0 Must) -- provides shared font-size
  constants for the DRAFT_INITIAL header, timer, and slot label.
  Same fallback rule as `S11-TD-UI-FLEX-STRIPS`.
- Depends on: `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`
  (roadmap rank 4; Tier 0 Must) -- provides the viewport-invariant
  test harness used by this story's centered-modal assertion test
  at 1920 x 1080 and 1366 x 768.
- Depends on: `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  (roadmap rank 6; Tier 0 Must) -- provides the global UI design spec
  whose numeric values drive the Tier 0 token modules above. If the
  spec is not authored at implementation time, this story is blocked.
- Depends on:
  [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md)
  -- Complete; provides `ShopAuctionUiPlugin`, panel roots, and
  shared formula scaffolding.
- Depends on:
  [Story 002](story-002-draft-initial-grid-purchase-ready.md)
  -- Complete; provides active DRAFT_INITIAL grid, purchase, Ready /
  Retract Ready, and PLACEMENT dismissal behavior this story must
  preserve.
- Depends on:
  [Story 003](story-003-shop-panel-slots-refresh-purchase-ready.md)
  -- Complete; sibling DRAFT_SHOP layout that must not regress when
  DRAFT_INITIAL is split out of `bottom_panel_node()`.
- Depends on:
  [Story 012](story-012-draft-initial-clear-objective-overlay.md)
  -- Complete; provides the Story 012 objective overlay contract
  this story must preserve.
- Depends on:
  [Story 013](story-013-card-text-stat-keyword-accessibility.md)
  -- Ready; provides the card-text / stat / keyword readability
  contract for DRAFT_INITIAL slot card text this story must preserve.
- Depends on: `design/ux/shop-auction-ui.md` for DRAFT_INITIAL panel
  layout, focus, and non-occlusion requirements.
- Depends on: ADR-015, ADR-021 Accepted.
- Unlocks: layout / composition / readability portion of the
  DRAFT_INITIAL surface for the friend-game product showcase. Does
  **not** unlock `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, or Polish->Release gate-check closure.

## Blockers

- Sprint 14 has not been activated. This story is a Sprint 14+
  candidate and is **blocked until Sprint 14 is opened**, the Tier 0
  foundational stories named above land, and `/story-readiness` is
  run against this story file.
- If `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` has not been authored, the
  numeric inputs for centered-modal width / max-width / padding /
  font sizes are undefined; this story remains blocked until the
  design spec authoring lands.

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
  `origin/main@51e6228`. Slug `S11-UX-DRAFT-GRID-CENTERED-MODAL`
  recorded per
  [UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
  rank 9 and PROMPT 802 §3.4 D1. Sprint 14 NOT activated. No
  implementation. No `/story-readiness` / `/dev-story` /
  `/story-done` invocation by this authoring prompt.
