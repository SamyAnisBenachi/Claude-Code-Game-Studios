# PROMPT 1836 — Krosmaga UI Stage 3: Next Implementation Slices

**Date**: 2026-05-28
**Worktree source**: `D:/_DEV/Work/Claude-Code-Game-Studios/tmpwt-1836-ui-stage3-slices`
**Source-of-truth branch**: main (HEAD at time of audit)
**Audit type**: Read-only — no source files modified

---

## Section 1: Current UI State Summary

### What UI Systems Exist Today

The client UI layer lives under `client/src/ui/` and `client/src/presentation/`. The
full module inventory is:

#### `client/src/ui/` — Bevy-UI overlay layer

| File/Directory | Purpose | Status |
|---|---|---|
| `mod.rs` | Re-exports all UI sub-modules; wires `PlayAreaPlugin`, `PhaseBannerPlugin` | Done |
| `card_inspect.rs` | Shared enlarged-card inspect primitive (320×520px shell, art window, title, stats, keyword, rules text) | Done |
| `design_tokens/` | Full token set: z_layers, typography, spacing, strips, play_area, overlays, interaction_states, card_slot, viewport_matrix, modal_panel, cta_row, scroll_region, status_chip, text_fit | Done (Sprint 14–18) |
| `hand/mod.rs` | Hand fan layout (10-slot arc), drag-and-drop placement, reserve mana strips, placement action panel, idle playable-affordance overlays, card art sync, stat labels | Done |
| `hand/drag_state_visuals.rs` | Per-slot drag-state overlay differentiation (Idle/Hover/Drag/DropTarget/Disabled) | Done |
| `hand/inspect.rs` | Right-click card inspect consumer for hand fan and DRAFT_INITIAL grid | Done |
| `hud/mod.rs` | HUD top-strip: gold, mana bar/diamond, phase pill, round counter, phase timer bar + numeric countdown, opponent figurine, objective dot rows, resolution dim overlay | Done (Sprint 14–18) |
| `hud/mana_preview.rs` | Mana preview projection for reserve/current split during placement | Done |
| `lobby.rs` | Centred lobby modal (Option A layout), class-picker portraits, room-code chip, player-slot panels, bot controls, confirm CTA | Done |
| `phase_banner.rs` | Transient centred phase-transition banner (auto-despawns after 1.4s) | Done (PROMPT 1404) |
| `photosensitivity_warning.rs` | One-time photosensitivity warning overlay | Done |
| `settings/mod.rs` | In-game settings panel (accessibility prefs: color-blind mode, placement timer multiplier, UI scale) | Done |
| `shared.rs` | Shared types: `BoardLayout`, `LaneCell`, `HudObjectiveUpdate`, `BOARD_CELL_COUNT`, `BOARD_LANE_COUNT` | Done |
| `shop_auction/mod.rs` | Shop slots (3), draft initial 3×3 modal, auction panel (featured card, bid buttons, pass, free-gold, settlement overlay, toast) | Done |
| `shop_auction/inspect.rs` | Right-click card inspect consumer for shop/auction surfaces | Done (PROMPT 1530) |

#### `client/src/presentation/` — world-space + cross-cutting layer

| File/Directory | Purpose | Status |
|---|---|---|
| `board_rendering.rs` | Board grid, lane surfaces/rails, unit sprites, objective sprites, HP bars, status icons, ghost preview, fog-of-war, spawn-range highlights | Done |
| `board_rendering/rendering_constants.rs` | Z-layer, cell/unit/objective/HP-bar size constants; Stage 2 board polish constants (PROMPT 1695) | Done |
| `board_rendering/targeting_overlay.rs` | Targeting dim-wash, valid-ring, endpoint-ring, invalid-marker overlays | Done |
| `board_rendering/perf_harness.rs` | Board rendering performance harness | Done |
| `result_screen.rs` | Game-over result screen: outcome accent stripe, per-lane objective scoreboard, step-through reveal, return-to-lobby CTA | Done |
| `connection_lost_overlay.rs` | Connection lost overlay | Done |
| `debug_bot_overlay.rs` | Bot debug overlay (dev only) | Done |
| `mod.rs` | `PresentationPlugin` wiring, `PresentationSet` system sets | Done |
| `qa_snapshot.rs` | QA snapshot observability fields | Done |
| `shared/economy_view.rs` | `PlayerEconomyView` resource | Done |
| `card_animations/` | Tween queue, placement animators, damage numbers, acquisition animation | Done |

#### `client/src/asset_wiring.rs` — asset path constants and binding helpers

All card-frame, stat-badge, rarity-icon, class-type-icon, shop/auction chrome,
HUD figurine/timer/dot, board character, and lobby portrait paths are declared
here. Several slots have "NO ANALOGUE on disk" and are repointed to the universal
placeholder (`ui_unit_placeholder_default_board.png`):

- `STAT_BADGE_AR_ASSET` — no Armour badge sprite on disk
- `BID_BUTTON_HOVER_ASSET` — no hover-state bid-button sprite on disk
- `HUD_PHASE_TIMER_BAR_ASSET` — no dedicated timer bar sprite on disk
- `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` — no destroyed-dot sprite on disk

### What Is Clearly Shipped / Done

The following areas are fully implemented and tested:

1. **Design token foundation** (Sprint 14–18): all 14 `design_tokens/` modules
   exist with stable named constants. No inline bare literals in surface code.
2. **Hand fan** with arc layout, drag-and-drop, card art (`CardSlotArtImage`),
   stat labels (ATK/HP/MP/AR), idle playable-affordance overlays, reserve mana
   strips, placement action panel, placement timer.
3. **Shop / auction panel** with all phases (DraftInitial grid, shop slots,
   auction featured card, bid buttons, settlement, toast). Stage 2 readability
   pass landed (PROMPT 1697): timer font H3, bid border 2px, auction copy strings
   corrected.
4. **Board rendering** with unit/objective sprites, HP bars, status icons, ghost
   preview, lane surfaces/rails, targeting overlays, spawn-range highlights.
   Stage 2 board polish landed (PROMPT 1695): chrome margins, lane-surface height
   ratio, rail alpha raised.
5. **HUD** with gold/mana/phase/round readouts, figurine (local + opponent),
   phase timer bar + numeric countdown, objective dot rows, RESOLUTION dim overlay.
6. **Lobby** with centred modal (Option A layout), class portrait picker with
   class-type icon overlays, room-code chip, player-slot panels.
7. **Card inspect** (`card_inspect.rs`) shared primitive — wired for hand/draft
   (PROMPT 1520) and shop/auction (PROMPT 1530). Right-click opens enlarged card
   with art, title, cost, ATK/HP, keyword, rules text.
8. **Phase-transition banner** (PROMPT 1404): transient centred label on every
   major phase change.
9. **Result screen** with outcome accent palette (Victory/Defeat/Draw), per-lane
   objective scoreboard, step-through reveal animation, return-to-lobby CTA.
10. **Asset provenance tooling** (`tools/asset-provenance/`): Stage 3 validator
    with 62 tests, dev-proxy-pack-stage3-candidate.json covering 17 LIDs.

### What Is Partially Done or Clearly Missing

The audit identified the following gaps, ordered by visual impact:

**A. Hover/inspect glossary tooltip** (story 028 `S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001`):
The card inspect primitive renders title, cost, ATK/HP, and a `keyword` field,
but there is no in-context keyword glossary panel. The `keyword` field in
`CardInspectView` is a single `Option<String>` — it can display one keyword
label but not expand it into a readable definition panel anchored near the card.
Story 028 is drafted but NOT activated.

**B. Card-inspect glossary content binding**: `card_inspect.rs` has
`CARD_INSPECT_KEYWORD_FONT_PX` and a `keyword` slot, but the `build_card_inspect_view_from_card`
function in `hand/inspect.rs` must map `Keyword` enums to human-readable names
and definitions. The mapping exists for enum-to-label, but full in-context definition
panels do not exist.

**C. Asset binding gaps** — four "NO ANALOGUE" slots still point to the universal
placeholder: Armour badge, bid-button hover state, HUD phase timer bar sprite,
and objective-dot destroyed state. These are cosmetic but visible to QA.

**D. Shop/auction panel chrome differentiation**: PROMPT 802 §3.6 A2 noted that
the auction featured-card reuses shop chrome under `PAW-TD-003-a`. The `SHOP_PANEL_CHROME_ASSET`
and `AUCTION_PANEL_CHROME_ASSET` constants now point to distinct files
(`art/ui/shop/ui_shop_panel_chrome.png` vs `art/ui/auction/ui_auction_panel_bg.png`),
but the auction panel background uses a flat colour path (`ui_auction_panel_bg.png`)
while `assets/art/ui/shop_auction/` has distinct border-tier assets not yet wired
(`ui_auction_border_tier1_hud.png` … `ui_auction_border_tier4_hud.png`). These
tier-keyed borders (presumably bid-tier indicators) have no consumer in
`shop_auction/mod.rs`.

**E. Bid-button hover state** is missing a real asset (falls back to universal
placeholder). The `BidButtonChromeState::Hover` path is declared in `asset_wiring.rs`
but `BID_BUTTON_HOVER_ASSET` points to `ui_unit_placeholder_default_board.png`.

**F. Mulligan / result-screen chrome polish**: The result screen has the outcome
accent palette and per-lane scoreboard, but `docs/ux/ui-clean-pass-roadmap.md` lists
`S11-UX-RESULT-RETURN-TO-LOBBY-001` as an already-tracked future candidate. No
result-screen mulligan (card redo) surface exists because the game has no mulligan
phase — the "mulligan" concept in the task description maps to the DRAFT_INITIAL
keep-9 grid (already implemented) and potentially a post-round card-retire flow
(not in the GDD scope).

**G. Responsive 1280×720 layout validation**: `viewport_matrix.rs` defines the
`1280×720` / `1366×768` / `1920×1080` safety matrix, and several harness HTML
files exist for validation, but no automated viewport-resize regression test
confirms the layout contract is enforced continuously. The layout contract document
(`docs/ux/global-ui-layout-contract.md`) notes the `button_vs_chip_lint_test.rs`
was shipped but the contract is a "target state" not yet fully satisfied.

**H. Auction tier-border wiring**: `assets/art/ui/shop_auction/` contains
`ui_auction_border_tier1_hud.png` through `ui_auction_border_tier4_hud.png`
and gem assets (`ui_gem_epic_default_*.png`, `ui_gem_rare_default_*.png`,
`ui_gem_legendary_default_*.png`) with no matching constants or consumer in
`asset_wiring.rs` or `shop_auction/mod.rs`. These are unbound.

---

## Section 2: 6 Concrete Implementation Slices

### SLICE-A: Hover Glossary Tooltip on Card Inspect

**Description**: Extend `card_inspect.rs` so the `keyword` section expands into a
multi-line glossary panel when a card has keyword(s). Map each `Keyword` enum
variant to a short definition string. Panel appears anchored below the keyword
label within the inspect card shell, scrolling if long.

**Files owned** (touch only these):
- `client/src/ui/card_inspect.rs` — add glossary node layout + constants
- `client/src/ui/hand/inspect.rs` — extend `build_card_inspect_view_from_card` to populate keyword definitions
- `client/src/ui/shop_auction/inspect.rs` — re-uses the shared primitive; verify existing wiring passes keyword through

**Files NOT touched**: `shop_auction/mod.rs`, `hand/mod.rs`, `result_screen.rs`,
`board_rendering.rs`, `asset_wiring.rs`, `lobby.rs`, all `design_tokens/` modules (read-only),
all `presentation/` except `result_screen.rs` (not touched), `hud/mod.rs`

**Estimated scope**: Medium (0.5–0.75d). The `card_inspect` primitive already has the
layout zones; this is additive content (keyword → definition map + extra node rows).

**Ready now?** YES. The shared primitive is stable, both inspect consumers are wired, and
no server/protocol change is needed. The `shared::card` keyword registry (`Keyword` enum)
is the only input source needed.

**Worker prompt template**:
```
Extend `client/src/ui/card_inspect.rs` and `client/src/ui/hand/inspect.rs` to add a
keyword-glossary expansion panel inside the card inspect shell. For each `Keyword` variant
in `shared::card`, add a short (1-2 sentence) definition string. Render one definition row
per keyword below the existing keyword label in the inspect card. Keep the glossary panel
inside the existing 320×520px shell (use `scroll_region` primitive if content overflows).
Add a unit test in `tests/integration/ui_clean_pass/` asserting at least one keyword
produces a non-empty definition row. Do NOT touch shop_auction/mod.rs, lobby.rs, hud/mod.rs,
board_rendering.rs, or any sprint/session-state files.
```

---

### SLICE-B: Auction Tier-Border Asset Binding

**Description**: Wire the four `ui_auction_border_tier*.png` assets (and the gem assets)
in `assets/art/ui/shop_auction/` into `asset_wiring.rs` and bind them to the auction
featured-card or bid-button chrome in `shop_auction/mod.rs`. These files exist on disk
but have no constants or consumer — they are currently invisible dead assets.

**Files owned** (touch only these):
- `client/src/asset_wiring.rs` — add `AUCTION_BORDER_TIER1_ASSET` … `AUCTION_BORDER_TIER4_ASSET` constants
- `client/src/ui/shop_auction/mod.rs` — wire border asset to bid-tier indicator or featured-card chrome

**Files NOT touched**: `hand/mod.rs`, `hud/mod.rs`, `lobby.rs`, `result_screen.rs`,
`board_rendering.rs`, all `design_tokens/` modules, `card_inspect.rs`, `card_animations/`

**Estimated scope**: Small (0.25–0.5d). Constants already declared (analogously) for
shop chrome; this is pattern-matching work.

**Ready now?** YES. The assets exist on disk; `asset_wiring.rs` patterns are established.
No design decision is needed — the `ui_auction_border_tier*.png` naming implies bid-tier
chrome (Tier 1 = cheapest bid range, Tier 4 = highest). Worker should confirm with art
directory naming before finalising.

**Worker prompt template**:
```
In `client/src/asset_wiring.rs`, add path constants for the four
`art/ui/shop_auction/ui_auction_border_tier{1..4}_hud.png` assets and the
`ui_gem_{rare,epic,legendary}_default_{24,32}.png` assets. In `client/src/ui/shop_auction/mod.rs`,
bind the tier-border to the auction bid button or featured-card chrome (choose whichever
reads most clearly as a bid-tier indicator). Add a snapshot test confirming the asset path
is reachable (exists on disk). Do NOT touch hand/mod.rs, hud/mod.rs, lobby.rs,
result_screen.rs, or any sprint/session-state files.
```

---

### SLICE-C: Missing Placeholder Repairs (Armour Badge, Bid-Hover, Timer Bar, Destroyed Dot)

**Description**: Author the four missing asset slots that still fall back to the universal
placeholder (`ui_unit_placeholder_default_board.png`): the Armour stat badge
(`STAT_BADGE_AR_ASSET`), the bid-button hover state (`BID_BUTTON_HOVER_ASSET`), the HUD
phase timer bar (`HUD_PHASE_TIMER_BAR_ASSET`), and the objective-dot destroyed state
(`HUD_OBJECTIVE_DOT_DESTROYED_ASSET`). For each, either:
- Bind to an existing on-disk asset that is semantically close (preferred — see notes below), OR
- Create a minimal placeholder PNG in the correct directory so the slot is visually distinct
  from the generic board unit placeholder.

Disk candidates: `ui_button_reserve_minus_active_hud.png` could serve as a temporary AR badge
substitute; `ui_bid_button_active.png` exists but not a hover-state variant; `ui_icon_checkmark_faint_hud.png`
could serve for destroyed dot. This slice is primarily about eliminating the most jarring
visual placeholder regression visible in QA screenshots.

**Files owned** (touch only these):
- `client/src/asset_wiring.rs` — update the four "NO ANALOGUE" constants
- Potentially create new PNGs in `assets/art/ui/hand/` or `assets/art/ui/auction/` for
  genuine gaps that need new art (Armour badge, bid hover)

**Files NOT touched**: `shop_auction/mod.rs`, `hand/mod.rs`, `hud/mod.rs`, `lobby.rs`,
`result_screen.rs`, `board_rendering.rs`, all `design_tokens/` modules

**Estimated scope**: Small (0.25d) for constant rebinding only; Medium (0.5d) if new
placeholder PNGs need to be created.

**Ready now?** YES for rebinding to existing on-disk analogues. YES for new minimal placeholder
PNGs if the worker can author them (no art-lead approval needed for dev-proxy-scope assets
under the provenance boundary story `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`).

**Worker prompt template**:
```
In `client/src/asset_wiring.rs`, update the four "NO ANALOGUE on disk" constants:
STAT_BADGE_AR_ASSET, BID_BUTTON_HOVER_ASSET, HUD_PHASE_TIMER_BAR_ASSET,
HUD_OBJECTIVE_DOT_DESTROYED_ASSET. For each, either point to the closest semantically
appropriate existing on-disk asset (prefer assets already in `assets/art/ui/hand/`,
`assets/art/ui/auction/`, or `assets/art/board/`) or create a minimal 1-colour
placeholder PNG in the correct art directory. Document each choice in a comment above
the constant. Do NOT change any functional system logic, any spawn system, or any
sprint/session-state files.
```

---

### SLICE-D: Hand Fan Card Readability — Arc Tightening and Card-Frame Size Audit

**Description**: Audit and tighten the hand fan card rendering at 1280×720.
`HAND_CARD_DISPLAY_WIDTH_PX = 108.0` and `HAND_CARD_DISPLAY_HEIGHT_PX = 150.0` with a
`fan_half_spread_px = 280.0` at 10 cards. At 1280×720 with 10 cards the leftmost and
rightmost fan slots extend ≈ 280px from centre, placing them at x=360 and x=920 —
inside the viewport. However the card height at 150px combined with the arc places the
top edge of the tallest slot (centre) at `260 - 100 - 75 = 85px` from the strip top,
leaving limited art window. This slice audits the fan at the minimum viewport (1280×720)
with a full 10-card hand, compares against `HAND_FAN_SLOT_COUNT` = 10 maximum, and
adjusts `fan_base_margin_px`, `arc_height_px`, and `max_rotation_deg` if cards are
visually crowded or partially clipped.

**Files owned** (touch only these):
- `client/src/ui/hand/mod.rs` — adjust `HandFanLayoutConfig::default()` constants if needed
- `client/src/ui/hand/drag_state_visuals.rs` — if overlay sizing needs updating (read-only review)
- `tests/integration/hand_ui/` — update or add layout assertion tests

**Files NOT touched**: `shop_auction/mod.rs`, `hud/mod.rs`, `lobby.rs`, `result_screen.rs`,
`board_rendering.rs`, all `design_tokens/` modules, `card_inspect.rs`, `asset_wiring.rs`

**Estimated scope**: Small-to-Medium (0.25–0.5d). Most changes are numeric constants;
integration tests cover the arc formula.

**Ready now?** YES. `compute_fan_slot_layout` and `metrics_for_viewport` are pure functions
with well-covered unit tests. Changes are isolated to `HandFanLayoutConfig::default()` and
test assertions.

**Worker prompt template**:
```
Run the existing hand-fan integration tests and the `hand-ui-placement-staged-disclosure-harness`
HTML harness with a full 10-card hand at 1280×720. Audit whether the fan cards are readable
(no clipping, arc spread not too wide, card rotation not exceeding comfortable readability).
If adjustments are needed, edit only `HandFanLayoutConfig::default()` in
`client/src/ui/hand/mod.rs` (fan_base_margin_px, fan_half_spread_px, arc_height_px,
max_rotation_deg). Update any affected integration test assertions. Do NOT touch shop_auction,
hud, lobby, result_screen, board_rendering, design_tokens, or asset_wiring.
```

---

### SLICE-E: Result Screen Chrome Polish — Step-Through Pacing and Outcome Accent Legibility

**Description**: The result screen has the Krosmaga-style outcome accent palette
(`OUTCOME_ACCENT_VICTORY/DEFEAT/DRAW/NEUTRAL`) and a step-through reveal system, but
the `S11-UX-RESULT-RETURN-TO-LOBBY-001` candidate is un-activated and `ResultScreenStepState`
drives the reveal pacing. This slice audits the result screen at 1280×720 (the minimum
viewport) and confirms:
- The outcome accent stripe is readable (width, alpha, contrast against the dark panel).
- The per-lane objective scoreboard rows are not clipped at 1280px wide.
- The return-to-lobby CTA (`CTA_PRIMARY_BG` gold button) is reachable without scrolling
  at the minimum viewport.
- The `ResultScreenMotionState` step-through timing feels Krosmaga-paced (not too fast
  to read, not too slow to be satisfying).
If gaps are found, apply targeted constant adjustments to `result_screen.rs` only.

**Files owned** (touch only these):
- `client/src/presentation/result_screen.rs` — chrome constant adjustments, CTA visibility fixes

**Files NOT touched**: `hand/mod.rs`, `shop_auction/mod.rs`, `hud/mod.rs`, `lobby.rs`,
`board_rendering.rs`, all `design_tokens/` modules, `card_inspect.rs`, `asset_wiring.rs`

**Estimated scope**: Small (0.25d) if findings are minor constant tweaks; Medium (0.5d) if
layout restructuring is needed.

**Ready now?** YES. `result_screen.rs` is self-contained; `ResultScreenPlugin` owns all
its systems. The existing integration test patterns cover phase transition and snapshot
caching.

**Worker prompt template**:
```
Audit `client/src/presentation/result_screen.rs` against a 1280×720 viewport:
(1) Confirm the outcome accent stripe (OUTCOME_ACCENT_VICTORY/DEFEAT/DRAW/NEUTRAL) is
    visually distinct at 1280×720 — adjust border width or alpha if too faint.
(2) Confirm the per-lane objective scoreboard (OBJECTIVE_LANES = 5 rows) does not clip
    at 1280px panel width.
(3) Confirm the return-to-lobby CTA (CTA_PRIMARY_BG gold button) is visible without
    scrolling at the minimum viewport.
(4) If the step-through pacing feels wrong, adjust the timing constant in
    ResultScreenMotionState.
Edit only `client/src/presentation/result_screen.rs`. Do NOT touch hand, shop_auction,
hud, lobby, board_rendering, design_tokens, card_inspect, or asset_wiring.
```

---

### SLICE-F: Responsive 1280×720 Layout Smoke — Viewport-Invariant Live Harness Coverage

**Description**: The Sprint 18 story 021 (`S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001`)
shipped a live-spawn harness for the hand UI (`hand-ui-placement-staged-disclosure-harness.html`),
board rendering perf harness, and shop/auction bid-target/objective harnesses. However the
`global-ui-layout-contract.md` acknowledges the contract is a "target state" not yet fully
satisfied. This slice creates one focused viewport-matrix smoke test that instantiates the
three primary surfaces (hand fan, shop/auction panel, HUD) in a `MinimalPlugins` + headless
world and confirms the layout invariants from §C1–C6 of the contract (no off-screen primary
CTA, no `Overflow::visible` outside approved sites, image-fitting via `NodeImageMode`, no
absolute `PositionType` except approved modal/overlay sites).

**Files owned** (touch only these):
- `tests/integration/ui_clean_pass/` — new test file `viewport_1280x720_layout_smoke_test.rs`
- `client/src/ui/` (read-only imports only — no edits)

**Files NOT touched**: Any `client/src/` production file, `shop_auction/mod.rs`,
`hand/mod.rs`, `hud/mod.rs`, `lobby.rs`, `result_screen.rs`, `board_rendering.rs`,
`design_tokens/`, `card_inspect.rs`, `asset_wiring.rs`

**Estimated scope**: Medium (0.5–0.75d). Follows the existing ECS `World`-based test
patterns established throughout the test suite.

**Ready now?** YES. All design-token primitives and the `viewport_matrix` module are stable.
The test pattern (MinimalPlugins + plugin scaffold + World queries) is established in dozens
of existing integration tests.

**Worker prompt template**:
```
Create `tests/integration/ui_clean_pass/viewport_1280x720_layout_smoke_test.rs`.
Using `World::new()` + `MinimalPlugins` + `HandUiPlugin`, instantiate the hand fan in a
1280×720 context and assert:
(1) All `FanSlotIndex` entities have `Node.left` within [0, 1280] and `Node.top` within [0, 720].
(2) No `FanSlotIndex` entity has `PositionType::Absolute` without an approved marker component.
(3) The submit button (`HandSubmitButton`) entity has a `Visibility` reachable from the root.
Mirror this pattern for at least one `ShopAuctionUiPlugin` surface (shop slot Node bounds).
Do NOT edit any production source file. Do NOT touch sprint/session-state files.
```

---

## Section 3: Blockers

The following are **true structural blockers** (not precautionary): items that a worker
cannot proceed without, not items that would merely be nice to have first.

### Genuine Blockers

**None for SLICE-A through SLICE-F as described above.** Each slice was designed to
be independently launchable from the current `main` baseline.

### Near-Blockers (Soft Dependencies — Not Blocking)

| Slice | Soft Dependency | Impact if Ignored |
|-------|----------------|-------------------|
| SLICE-A (Keyword Glossary) | `shared::card::Keyword` enum must have stable variant list | Worker should confirm no keyword variants are WIP before authoring definitions |
| SLICE-B (Tier Border) | Art intent for tier-border meaning (bid tier 1–4?) is assumed from naming convention; no GDD spec confirms | If assumption is wrong, the binding site may need moving after art-lead review |
| SLICE-C (Placeholder Repair) | For new PNGs: dev-proxy provenance boundary (`S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`) must gate any real Krosmaga art; minimal colour-fill placeholders are always safe | Minor: work within existing provenance rules |
| SLICE-E (Result Screen) | `S2CGameOver` must be reachable in a two-client run for visual validation | Existing harness tests cover the resource path; live QA needs a two-client run |
| SLICE-F (Layout Smoke) | `HandUiPlugin` initialisation in `MinimalPlugins` context must not hit missing dependencies | Established pattern from existing integration tests; worker verifies `init_resource` chain |

### Items That Are NOT Blockers (Common Misconceptions)

- Final art production (`PAW-TD-*-a`) is NOT required for any slice. All slices work
  with existing on-disk dev-proxy assets.
- `QA-COND-0005` (Standard-tier accessibility) is NOT a blocker. Friend-game scope
  boundary is preserved across all slices.
- A live two-client session is NOT required for SLICE-A through SLICE-D or SLICE-F.
  SLICE-E benefits from it for visual validation only.
- Slice authoring does NOT require activating story 028
  (`S19-UI-CARD-RENDERING-FIDELITY-HOVER-GLOSSARY-001`) formally. SLICE-A implements
  the core of its scope as a standalone targeted fix.

---

## Section 4: File Conflict Map

### Slices That Can Run in Parallel (Zero File Overlap)

All six slices were intentionally scoped to touch disjoint file sets. The following
pairs are confirmed parallel-safe:

| Pair | Shared files | Parallel-safe? |
|------|-------------|----------------|
| SLICE-A + SLICE-B | None | YES |
| SLICE-A + SLICE-C | None | YES |
| SLICE-A + SLICE-D | None | YES |
| SLICE-A + SLICE-E | None | YES |
| SLICE-A + SLICE-F | None (SLICE-F reads but never edits production files) | YES |
| SLICE-B + SLICE-C | `client/src/asset_wiring.rs` | CONFLICT — sequential required |
| SLICE-B + SLICE-D | None | YES |
| SLICE-B + SLICE-E | None | YES |
| SLICE-B + SLICE-F | None | YES |
| SLICE-C + SLICE-D | None | YES |
| SLICE-C + SLICE-E | None | YES |
| SLICE-C + SLICE-F | None | YES |
| SLICE-D + SLICE-E | None | YES |
| SLICE-D + SLICE-F | `client/src/ui/hand/mod.rs` — SLICE-F reads it, SLICE-D edits it | SOFT CONFLICT — run SLICE-D first or branch independently |
| SLICE-E + SLICE-F | None | YES |

### Recommended Execution Order

**Batch 1 (fully parallel)**: SLICE-A + SLICE-D + SLICE-E
- Zero file overlap between all three.
- SLICE-B and SLICE-C share `asset_wiring.rs` — exclude from Batch 1.

**Batch 2 (after Batch 1 merges)**: SLICE-B + SLICE-C sequentially (or one combined worker
to avoid `asset_wiring.rs` conflict), then SLICE-F after SLICE-D lands.

**Minimum critical path**: SLICE-B → SLICE-C (sequential on `asset_wiring.rs`) = 0.5–1.0d;
SLICE-F after SLICE-D = 0.75–1.25d. All others can overlap.

### Visual Dependency (Not a Code Conflict)

SLICE-A's glossary tooltip is most visible when SLICE-C's placeholder repairs are in
place (so the card art window in the inspect panel shows a real asset rather than the
generic board placeholder). This is a visual quality dependency, not a code one —
SLICE-A can ship and be useful before SLICE-C lands.

---

## Appendix: Complete UI File Inventory

### `client/src/ui/` (15 files / 5 directories)

```
card_inspect.rs           — shared enlarged-card shell primitive
design_tokens/
  card_slot.rs            — card-slot geometry catalog (5 kinds)
  cta_row.rs              — CTA row flex primitive
  interaction_states.rs   — hover/focus/pressed/disabled tokens
  mod.rs                  — token module re-exports
  modal_panel.rs          — modal content-budget primitive
  overlays.rs             — overlay alpha tokens (Dim/Scrim/Toast)
  play_area.rs            — in-session middle-band flex container
  scroll_region.rs        — body scroll region primitive
  spacing.rs              — named spacing scale (XS/SM/MD/LG/XL)
  status_chip.rs          — status chip vs CTA button distinction
  strips.rs               — flex strip primitives (Header/Lane/Hand/Footer bars)
  text_fit.rs             — text wrap policy primitive
  typography.rs           — typography scale (Caption/Body/H3/H2/H1/Display)
  viewport_matrix.rs      — safety viewport matrix (1280×720 / 1366×768 / 1920×1080)
  z_layers.rs             — named GlobalZIndex constants
hand/
  drag_state_visuals.rs   — drag-state overlay child nodes (5 states)
  inspect.rs              — hand/draft right-click inspect consumer
  mod.rs                  — hand fan + placement drag + reserve mana (6257 lines)
hud/
  mana_preview.rs         — mana preview projection
  mod.rs                  — HUD strip systems (figurines, timer, dots)
settings/
  mod.rs                  — accessibility settings panel
shop_auction/
  inspect.rs              — shop/auction right-click inspect consumer
  mod.rs                  — shop/auction panel systems
lobby.rs                  — lobby modal + class picker
mod.rs                    — UI layer re-exports + PlayAreaPlugin + PhaseBannerPlugin
phase_banner.rs           — transient phase-transition banner
photosensitivity_warning.rs — one-time photosensitivity overlay
shared.rs                 — shared BoardLayout, LaneCell, objective update types
```

### `client/src/presentation/` (12 files / 3 directories)

```
board_rendering.rs        — board grid, units, objectives, HP bars, ghost preview
board_rendering/
  perf_harness.rs         — performance harness
  rendering_constants.rs  — Z layers + size constants
  targeting_overlay.rs    — targeting dim/ring overlays
card_animations/
  animators.rs
  damage_numbers.rs
  events.rs
  input_gating.rs
  lenses.rs
  mod.rs
  placement.rs
  queue.rs
connection_lost_overlay.rs
debug_bot_overlay.rs
mod.rs                    — PresentationPlugin + PresentationSet
qa_snapshot.rs
result_screen.rs          — game-over result surface
shared/
  economy_view.rs
```

---

1836: KROSMAGA-UI-STAGE3-NEXT-IMPLEMENTATION-SLICES: SHIPPED
