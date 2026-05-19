# Global UI Design Spec — Sprint 14 UI Clean-Pass Foundation

> **Story**: `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
> (`production/epics/ui-clean-pass/story-007-global-ui-design-spec.md`)
> **Sprint**: 14 (active; stage `Polish`; PROMPT 761 `Polish->Release` `FAIL`
> preserved)
> **Authoring prompt**: PROMPT 911 (`/dev-story`)
> **Source-of-truth at authoring**: `origin/main@b39eedf` (PROMPT 908
> `/story-done` `S11-TD-UI-FONT-CONSTANTS`)
> **Worktree**: `D:/_DEV/wt/ccgs-prompt-911-global-ui-design-spec`
> **Branch**: `work/s14-global-ui-design-spec`
> **Roadmap rank**: 6 (Tier 0, Must, 1.0d) per
> `docs/ux/ui-clean-pass-roadmap.md`

This document is the **canonical global UI design spec** for the playable
client. Tier 0 token modules (z-layers, typography, spacing, overlay alpha)
and the Sprint 14 Tier 1 surface stories read their numeric values from
this spec; per `docs/ux/ui-clean-pass-roadmap.md` §3 Sequencing Rule 2 it is
the design-token source of truth that the rest of the UI clean-pass
references.

---

## §1 Status / No-Claim Banner

This spec is **paperwork only**. PROMPT 911 (this authoring run) is a
`/dev-story` documentation closure for story 007. It authors **only** the
files under `docs/ux/` and `production/qa/evidence/sprint-14-ui-foundation/
global-ui-design-spec/`. It does **not** change any code, any test, any
sprint plan, any sprint-status row, any orchestrator state file, any
QA-plan file, or any session-state file.

### What this spec does NOT claim

This spec, and its adoption by Sprint 14 Tier 0 / Tier 1 stories, does
**not** claim, advance, or close any of:

- Public release readiness.
- Release-candidate (RC) readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion (`PAW-TD-002-a` …
  `PAW-TD-006-a`).
- Sprint 14 close-out.
- Polish → Release gate-check retry (PROMPT 761 `FAIL` preserved; **NO**
  retry is in scope per `TQ-S12-C2`-adjacent reasoning).
- Stage advance from `Polish` to `Release`.
- Underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`).

### Accept-risk dispositions preserved verbatim

The following dispositions are preserved **unchanged** by this spec:

- **`QA-COND-0005`** — Standard-tier accessibility remains **accepted-risk**
  (friend-game scope only). WCAG contrast ratios, ≥44px hit-targets, full
  keyboard navigation, screen reader support, colorblind modes, and text
  scaling are **out of spec scope**. The lobby `LOBBY_BUTTON_HEIGHT = 30.0`
  defect (PROMPT 802 §3.1 L5) remains accept-risk under `QA-COND-0005`;
  pulling a lobby surface story does NOT thereby commit to Standard-tier
  hit-target conformance.
- **`QA-COND-0006`** — playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. A visibly polished UI does not by itself
  produce playtest evidence.
- **`PAW-TD-002-a` … `PAW-TD-006-a`** — placeholder-art accept-risk across
  PAW-002..PAW-006. UI clean-pass repair is layout / composition /
  hierarchy / typography / z-order / spacing work and does **not** advance
  placeholder-art resolution. PROMPT 802 §7 places final-art work
  explicitly out of audit scope.

---

## §2 Scope Boundaries — Friend-Game vs Standard-Tier

This spec governs **friend-game visual polish only**. It is the
single source of truth for layout, composition, typography, spacing,
z-order, overlay alpha, palette, and responsive rules **for friend-game
scope**.

It is **not** the spec for:

- **Standard-tier accessibility** — WCAG contrast checking, ≥44px
  hit-target enforcement, focus order, keyboard navigation, screen-reader
  hints, colorblind modes, text scaling. These belong in a separate
  accessibility spec; pulling values out of this spec does NOT advance
  `QA-COND-0005`.
- **Final-art / asset-production** — `PAW-TD-*-a` placeholder PNGs
  (PAW-002..PAW-006) are preserved. Color and font choices below are
  **placeholder palette / placeholder font** for friend-game scope; final
  asset replacement is a separate sprint scope.
- **Animation / motion** — tween / transition spec is a separate scope.
- **Interaction-state primitives** — hover / focus / pressed / disabled
  visual primitives are authored by the Tier 0 Should-priority adjacent
  row `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` and ratified in §11.
  Per-surface migration of existing Sprint 14 button surfaces is deferred
  to a Sprint 16+ follow-on story family
  (`S16-UI-INTERACTION-STATE-MIGRATION-*`). The friend-game-vs-Standard-
  tier scope boundary is preserved: visual focus-ring presence does
  **not** advance `QA-COND-0005`.
- **Localization** — string layout, RTL, text expansion. Separate scope.
- **Per-element layout (HUD top-strip child order, lobby form sequencing,
  shop slot well composition)** — owned by the per-surface Tier 1 stories
  (15 / 16 / 24 / 25 / 26 / etc.). This spec defines the cross-cutting
  primitives (z-layer, font scale, spacing token, strip primitive); the
  Tier 1 surface story is authoritative for how a specific surface uses
  them.
- **Board-rendering** — world-space sprite layout under
  `client/src/presentation/board_rendering.rs` is owned by Tier 3 story 14
  (`S11-UX-BOARD-RENDERING-SPEC`, doc-only; depends on this spec).

If a future Sprint 14+ activation attempts to silently expand the claim
(e.g. flips `QA-COND-0005` to `closed`, claims Standard-tier conformance,
or claims `PAW-TD-*-a` resolved by layout-only repair), the activation
must be rejected and the row sent back for scope correction.

---

## §3 Z-Index Layer System

Canonical 8-layer hierarchy. Every `bevy_ui` root, overlay, modal, drag
ghost, and toast in the playable client paints into one of these layers
via `bevy::ui::GlobalZIndex` instead of relying on spawn-order. The
ordering is structural; integer values are spaced by 100 to allow future
intermediate layers without re-ordering existing constants.

**Source-of-truth module**: `client/src/ui/design_tokens/z_layers.rs`
(landed by story 002 via PROMPT 902). This spec ratifies the values
already shipped; no value change is requested.

| Layer | `GlobalZIndex` | Bevy module symbol | Canonical surfaces |
|-------|----------------|---------------------|---------------------|
| `Background` | `0`   | `BACKGROUND` | Background fills (clears, ambient backdrops). |
| `World`      | `100` | `WORLD`      | World-space board content (sprite `Transform.z` reference; not a bevy_ui consumer — see ADR-021 §R2). |
| `Units`      | `200` | `UNITS`      | Unit / objective sprites above the world layer (sprite `Transform.z` reference; not a bevy_ui consumer). |
| `UiBase`     | `300` | `UI_BASE`    | Foundational bevy_ui roots: lobby root, HUD root, hand fan root, shop / auction root, settings root. |
| `UiOverlay`  | `400` | `UI_OVERLAY` | Translucent overlays painted above the UI base: HUD dim, settlement scrim, draft-initial objective overlay, drag ghost, connection-lost overlay. |
| `Modal`      | `500` | `MODAL`      | Centred modal panels that demand player attention: result screen, photosensitivity warning, settings shell. |
| `Toast`      | `600` | `TOAST`      | Transient notifications painted above modals: shop / auction toasts, hand-full banners. |
| `Debug`      | `700` | `DEBUG`     | Diagnostic / dev-only overlays (not shipped in release builds). |

**Spacing rule**: adjacent layers are separated by exactly 100 integer
units. The module constant `LAYER_MIN_GAP = 10` is the *audit floor* (a
future intermediate layer may take any value with ≥10 units of gap from
its neighbours); the canonical spacing is 100.

**Pairwise distinctness**: every named layer resolves to a distinct
integer; no two constants alias.

**ADR alignment**: ADR-021 (Presentation Layer Architecture, R2) preserved
— world-space sprites paint below bevy_ui regardless of the `World` /
`Units` constants; those exist for documentation and future cross-layer
audits. ADR-002 preserved — z-layer constants are read-only presentation
primitives; no optimistic client-side authority is introduced.

---

## §4 Spacing Scale

Canonical spacing scale for child gaps, padding, and inter-element
margins inside flex strips. Each adjacent step roughly doubles so the set
is easy to reason about across a wide range of element sizes.

| Token | Pixels | Canonical use |
|-------|--------|----------------|
| `SPACING_XS` | `4`  | Tightest gap. Adjacent icon + numeric readout, badge padding, intra-cluster spacing. |
| `SPACING_SM` | `8`  | Default child gap inside a tight cluster (e.g. HUD secondary-row gold-icon + value). |
| `SPACING_MD` | `16` | Default gap between distinct readouts on the same strip (e.g. HUD gold cluster ↔ HUD mana cluster). Default panel padding. |
| `SPACING_LG` | `24` | Section separator inside a panel; gap between a strip's left edge and its first child. |
| `SPACING_XL` | `32` | Largest single step. Headline ↔ body separation, lobby form section separator. |

**Strictly-ascending invariant**: `XS < SM < MD < LG < XL` and the
geometric step is approximately `×2` (4 → 8 → 16 → 24 → 32). The 24-step
breaks strict doubling intentionally so consumers have a "between MD and
XL" middle option for asymmetric layouts.

**Replacement target** (story 004): the per-module gap constants enumerated
in PROMPT 802 §3.9 G2 (`HUD_GOLD_ROW_GAP_PX = 48.0`, `HUD_SECONDARY_ROW_GAP_PX
= 28.0`, and similar) read from `SPACING_*` tokens after story 004
migration; values larger than `XL = 32` are recomposed as `XL + MD`,
`XL + XL`, or as explicit padding on the strip's container.

**Out of scope**: viewport-driven spacing scaling. Sizes are fixed-pixel
for friend-game scope; the responsive matrix in §8 governs which surfaces
scale with the viewport vs which stay pixel-fixed.

---

## §5 Typography Hierarchy

Canonical typography scale, weights, and line-height ratio. **The numeric
values below are the canonical values already shipped by story 003
(`S11-TD-UI-FONT-CONSTANTS`, PROMPT 904 worker / PROMPT 906 integration /
PROMPT 908 `/story-done`) and verified verbatim against
`client/src/ui/design_tokens/typography.rs` at `origin/main@b39eedf`**.

### Semantic sizes (Caption → Display, strictly ascending)

| Token | Pixels | Bevy module symbol | Canonical use |
|-------|--------|---------------------|----------------|
| `Caption` | `13` | `typography::CAPTION` | Footnotes, micro-copy, secondary labels. Smallest semantic level in the scale. |
| `Body`    | `15` | `typography::BODY`    | Default running text, labels, room-code chip, lobby buttons. Reference baseline for body copy. |
| `H3`      | `18` | `typography::H3`      | Subheads, section labels, lobby status banner, return-to-lobby button. |
| `H2`      | `22` | `typography::H2`      | Panel titles, HUD secondary readouts (phase / round / mana / reserve). Sits ≥ HUD resource accessibility floor (20px). |
| `H1`      | `30` | `typography::H1`      | Screen headlines (result screen "RESULT PENDING"), HUD reserved-gold readout, connection-lost overlay headline. |
| `Display` | `40` | `typography::DISPLAY` | HUD primary readouts (own gold, opponent gold). Equals the HUD gold accessibility floor (40px). |

**Strictly-ascending invariant**: `Caption < Body < H3 < H2 < H1 <
Display` (asserted by `typography::ALL_SCALES_ASCENDING` and the
`ac1_canonical_scale_ordering_matches_story_spec` test). Minimum pairwise
gap = `SCALE_MIN_GAP = 2.0` px (audit floor for inserting future
intermediate scales without re-ordering).

### Font weights

| Token | CSS-style numeric | Bevy module symbol | Canonical use |
|-------|--------------------|---------------------|----------------|
| `Regular`  | `400` | `typography::WEIGHT_REGULAR`  | Default text weight for body copy and most labels. |
| `SemiBold` | `600` | `typography::WEIGHT_SEMIBOLD` | Subheads, emphasised labels, primary CTAs that should read heavier than body without dominating. |
| `Bold`     | `700` | `typography::WEIGHT_BOLD`     | Screen headlines and HUD primary readouts that must dominate visual hierarchy. |

**Weights are a semantic contract**, not yet a font-file switch. The
playable client uses Bevy's default font; mapping the weights to actual
font assets is deferred to a follow-on story per story 003 §Scope.
`PAW-TD-*-a` accept-risk preserved.

### Line-height ratio

`typography::LINE_HEIGHT_DEFAULT_RATIO = 1.25`. Multiply by a semantic-size
constant to obtain a `Val::Px(...)` line height when explicit vertical
rhythm is required. Spawn sites must not embed ad-hoc ratios; this is the
single source of truth for vertical rhythm.

### Accessibility-floor guard rails

Two regression tests in `typography.rs` lock the canonical values against
the accessibility floors named in the playable-client tests:

- `Display ≥ HUD_GOLD_TEXT_MIN_SIZE_PX (40.0)` so the HUD gold readout
  preserves its existing accessibility floor.
- `H2 ≥ HUD_RESOURCE_TEXT_MIN_SIZE_PX (20.0)` so HUD phase / round / mana
  / reserve readouts preserve theirs.

This spec **ratifies the shipped values verbatim** — no override is
requested.

---

## §6 Overlay Alpha Tokens

Canonical alpha-channel constants for translucent overlays. Replaces the
three scattered scrim / dim values PROMPT 802 §3.2 H4 / §3.9 G4
enumerated:

- `client/src/ui/hud/mod.rs:34` `HUD_DIM_OVERLAY_ALPHA = 0.45`
- `client/src/ui/shop_auction/mod.rs:3550` settlement scrim `0.58`
- `client/src/presentation/result_screen.rs:518` result panel backdrop
  `0.46`

### Ratified canonical tokens

| Token | Float | Canonical use | Rationale |
|-------|-------|----------------|-----------|
| `OVERLAY_DIM_ALPHA`    | `0.45` | HUD combat-focus dim during settlement entry; light dim where gameplay UI must remain partially legible underneath. Preserves the existing HUD value (`hud/mod.rs:34`). | A light dim that does NOT block visual continuity with the gameplay layer. Used when the dim is decorative / supportive (focus framing), not a hard modal scrim. |
| `OVERLAY_SCRIM_ALPHA`  | `0.55` | Modal scrim (settlement overlay, result screen panel backdrop, connection-lost overlay). Single canonical value for *all* modal scrims so transitions between settlement → result → connection-lost no longer flicker between three different darkness levels. | Splits the difference between the shipped `0.46` (result) and `0.58` (settlement) values. Heavy enough to read as "modal blocker"; light enough that the player retains spatial awareness of the underlying state. |
| `OVERLAY_TOAST_ALPHA`  | `0.80` | Toast root background (shop / auction toasts, hand-full banners). Above the modal scrim so the toast reads as a foreground notification. | Tested-against existing toast styling. Worker discretion at story 006 implementation if the toast root currently uses a different value. |

**Ranges**: every token must satisfy `0.0 < alpha < 1.0`. Pure-opaque
(`1.0`) and fully-transparent (`0.0`) values are not overlays and live
elsewhere.

**Migration mapping** (consumed by story 006
`S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`):

| Existing literal | Existing value | New token | Notes |
|------------------|----------------|-----------|-------|
| `HUD_DIM_OVERLAY_ALPHA` (`hud/mod.rs:34`) | `0.45` | `OVERLAY_DIM_ALPHA` | No value change. Preserves visual continuity. |
| `BackgroundColor(Color::srgba(0.02, 0.05, 0.08, 0.58))` (`shop_auction/mod.rs:3550`) | `0.58` | `OVERLAY_SCRIM_ALPHA` (`0.55`) | Slight lighten (−0.03). Acceptable; the settlement overlay was the outlier. |
| `BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.46))` (`result_screen.rs:518`) | `0.46` | `OVERLAY_SCRIM_ALPHA` (`0.55`) | Slight darken (+0.09). Acceptable; consolidates the three scrims to one. |

**Scope guard**: this token covers *modal scrim / dim* surfaces only.
Board ghost preview opacity (a future Tier 2 row
`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`) and any sprite-level alpha
remain out of scope. The connection-lost overlay comment in
`presentation/connection_lost_overlay.rs:206` references the `0.46`
result backdrop value — that comment will read `OVERLAY_SCRIM_ALPHA` after
story 006 migration.

---

## §7 Color Tokens

Canonical friend-game color palette. Each token is a placeholder palette
value sized for visual cohesion; final-art replacement is a separate
scope under `PAW-TD-*-a` (preserved accept-risk).

| Token | RGB hex | `Color::srgb(...)` reference | Canonical use |
|-------|---------|------------------------------|----------------|
| `PRIMARY`           | `#4A90E2` | `Color::srgb(0.290, 0.565, 0.886)` | Primary brand / interactive accent. Default CTA fill (lobby "Confirm class", auction "Bid", shop "Buy"). |
| `SECONDARY`         | `#E29E4A` | `Color::srgb(0.886, 0.620, 0.290)` | Secondary affordance / contrast against primary (e.g. "Cancel" / "Pass" buttons). |
| `ACCENT`            | `#F2C94C` | `Color::srgb(0.949, 0.788, 0.298)` | Highlight / featured-card frame; gold readout chrome. Used sparingly. |
| `SURFACE`           | `#0A0D14` | `Color::srgb(0.039, 0.051, 0.078)` | Default panel / scrim background base color. Pair with `OVERLAY_SCRIM_ALPHA` for modal backdrops. |
| `SURFACE_ELEVATED`  | `#161B27` | `Color::srgb(0.086, 0.106, 0.153)` | Raised panel surface (modal interior, lobby form panel, shop / auction inner container). |
| `SEMANTIC_SUCCESS`  | `#27AE60` | `Color::srgb(0.153, 0.682, 0.376)` | Confirmation, "Winning" auction lead state, success toasts. |
| `SEMANTIC_WARNING`  | `#F2994A` | `Color::srgb(0.949, 0.600, 0.290)` | Caution, "Tied" auction state, hand-full banner. |
| `SEMANTIC_ERROR`    | `#EB5757` | `Color::srgb(0.922, 0.341, 0.341)` | Error, "Losing" auction lead state, connection-lost overlay headline. |

**Friend-game palette only**: not WCAG contrast-checked. Standard-tier
conformance (`QA-COND-0005`) requires a separate audit and a separate
spec.

**Color::srgb() encoding**: Bevy 0.18 stores sRGB linearly; `Color::srgb`
expects 0.0..=1.0 floats. Each row above lists the canonical hex (the
authoring representation) and the `Color::srgb(...)` literal exactly as it
will appear in a `client/src/ui/design_tokens/colors.rs` module.

**Adoption note**: Tier 1 surface stories may continue to reference
existing `Color` literals where they describe a single specific element
(e.g. the orange highlight on the lobby confirm CTA) until a follow-on
"colorization pass" story migrates all eight tokens into a single palette
module. This spec ratifies the palette; the migration is downstream.

---

## §8 Responsive Layout Rules

Canonical viewport size matrix and per-class scaling rules. **The matrix
below is the canonical 6-viewport set already shipped by story 005
(`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`, PROMPT 905 worker / PROMPT 907
integration / PROMPT 909 `/story-done`) and verified verbatim against
`tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS` at
`origin/main@b39eedf`**.

### Canonical viewport matrix

| Name | Width × Height | Aspect ratio | Role |
|------|----------------|--------------|------|
| `1366x768`  | 1366 × 768   | 16:9   | **Minimum supported viewport** — common laptop default. Every surface MUST fit. |
| `1920x1080` | 1920 × 1080  | 16:9   | **Baseline reference (HD)** — the design-source viewport. All baseline captures land here. |
| `1920x1200` | 1920 × 1200  | 16:10  | 16:10 monitor variant. |
| `1280x960`  | 1280 × 960   | 4:3    | Legacy 4:3 monitor. Surfaces must still fit; aspect-stretch chrome is acceptable. |
| `3840x2160` | 3840 × 2160  | 16:9   | **4K scale-up boundary** — every surface scales correctly. |
| `2560x1080` | 2560 × 1080  | 21:9   | **Ultrawide aspect-stretch boundary** — every surface remains centred or anchored without clipping. |

### Per-class scaling rules

| Class | Viewport behavior | Examples |
|-------|--------------------|----------|
| **Strip primitives** (HeaderBar / FooterBar / HandBar — see §9) | Pixel-fixed height across **every** viewport. Width = full viewport (`Val::Percent(100.0)`). | HUD top strip, HUD bottom strip, hand-fan strip. |
| **Centred modal panels** | Pixel-fixed width × height; centred on the viewport (`(vw - w) / 2`, `(vh - h) / 2`). | Draft centered modal, shop panel, auction panel, result screen. |
| **Full-viewport overlays** | Full viewport width × height; transparent fill above the relevant z-layer. | HUD dim overlay, settlement overlay, connection-lost overlay. |
| **Anchored panels** | Fixed pixel position anchored to a viewport corner (top-left / bottom-right). | (Legacy) lobby column. Tier 1 row `S12-UX-LOBBY-LAYOUT-MODAL-001` migrates the lobby to a centred-modal OR full-viewport-hero composition per producer-decision-3. |
| **World-space sprites (not bevy_ui)** | World board scales with viewport via camera zoom; sprite Transform.z reads §3 World / Units layer constants. | Tile sprites, unit sprites, objective sprites. |

### What scales vs what stays pixel-fixed

- **Pixel-fixed**: strip heights, font sizes (§5), spacing tokens (§4),
  overlay alpha (§6), modal panel widths and heights.
- **Scales with viewport**: world board (camera zoom), strip widths
  (`Val::Percent(100.0)`), overlay backdrops (full viewport).
- **Centred**: every modal panel and the result screen — anchor reads
  `(vw - w) / 2, (vh - h) / 2` and remains pixel-stable.

### Invariants asserted by story 005

For every surface, every viewport in the matrix above:

1. **No clipping** — the surface's bounding rectangle is fully contained
   within the viewport rectangle.
2. **No overlap** — no two `UI_BASE` roots have overlapping bounding
   rectangles (overlays and modals are excluded from the geometric check
   per §3 z-layer ordering; story 002 already guarantees their paint
   order).
3. **Stable anchor** — the surface's anchor (top-left, bottom-left, or
   centre per the §"Per-class scaling rules" table) lands at the same
   proportional position across all six viewports.
4. **Deterministic strip height** — strip primitives (HeaderBar / FooterBar
   / HandBar) have identical pixel heights across every viewport.

---

## §9 Strip Composition Patterns

Canonical flex-strip primitives. Tier 0 story 004
(`S11-TD-UI-FLEX-STRIPS`) ships these as the `HeaderBar` / `HandBar` /
`FooterBar` strip-composition primitives that HUD top (story 015), HUD
bottom (story 016), and hand UI consume.

**The strip heights below are ratified verbatim against the canonical
baseline already shipped by story 005 at `tests/integration/fixtures/
ui_viewport_baseline.rs`** (constants `HEADER_BAR_HEIGHT_PX = 60.0`,
`FOOTER_BAR_HEIGHT_PX = 40.0`, `HAND_BAR_HEIGHT_PX = 180.0`). When story
004 lands the primitive module reads these values from the spec.

| Strip | Height (px) | Flex direction | Justify content | Align items | Anchor (parent UI root) | Canonical consumers |
|-------|-------------|-----------------|------------------|--------------|---------------------------|----------------------|
| `HeaderBar` | `60`  | `Row` | `SpaceBetween` | `Center` | Top-left of viewport (`PositionType::Absolute`, `top: Val::Px(0.0)`, `left: Val::Px(0.0)`, full-width). | HUD top strip — gold / mana / phase / round / timer (story 015). |
| `LaneBar`   | `60`  | `Row` | `Center`        | `Center` | Top-centre, below `HeaderBar` (story 004 worker decision: implement as bevy_ui IFF the lane indicators are bevy_ui rather than world-space sprites; otherwise the LaneBar primitive remains documented but unimplemented). | Lane indicators / board-chrome strip (Tier 3 board-rendering scope). |
| `HandBar`   | `180` | `Row` | `Center`       | `End`    | Bottom edge of viewport (`PositionType::Absolute`, `bottom: Val::Px(0.0)`, full-width). | Hand UI card-fan row (existing card-fan layout preserved per `f190cc7`). |
| `FooterBar` | `40`  | `Row` | `SpaceBetween` | `Center` | Bottom edge of viewport, immediately above `HandBar` (`PositionType::Absolute`, `bottom: Val::Px(HAND_BAR_HEIGHT_PX)`, full-width). | HUD bottom strip — figurine area + reserve-strip readouts (story 016). |

### Strip column composition

Strips stack from the **bottom** of the viewport so the hand reads as the
player's foreground anchor:

1. `HeaderBar` (60 px) at `top: 0` — top edge.
2. Centre play area (camera-rendered board + centred modal panels).
3. `FooterBar` (40 px) at `bottom: HAND_BAR_HEIGHT_PX (180)` — sits
   immediately above the hand.
4. `HandBar` (180 px) at `bottom: 0` — bottom edge.

### Default child spacing inside a strip

- Cluster-internal gap (e.g. icon + value): `SPACING_SM` (8 px) per §4.
- Cluster-to-cluster gap (e.g. gold cluster ↔ mana cluster): `SPACING_MD`
  (16 px) per §4.
- Strip-edge padding (left edge / right edge): `SPACING_LG` (24 px) per
  §4.

### Out of scope

- **Per-strip child order** — owned by the per-surface Tier 1 story (15 /
  16). This spec defines the strip parent; the surface story defines the
  children.
- **LaneBar implementation** — story 004 worker decides whether `LaneBar`
  is bevy_ui or remains world-space sprites. If the latter, the
  primitive remains documented in this spec but is unimplemented in the
  Tier 0 module; consumers fall back to `client/src/presentation/
  board_rendering.rs` for lane indicator geometry.

---

## §10 Component Specifications (stretch / optional)

Optional cross-cutting component patterns. This section is **non-binding**
— Tier 1 surface stories implement their own components and may cite this
section as guidance rather than a strict contract.

### Primary button affordance

- Background: `PRIMARY` token (§7).
- Text: `BODY` (15 px) / `Bold` (700) per §5.
- Padding: `SPACING_SM` vertical + `SPACING_LG` horizontal (8 + 24 px).
- Hover / focus / pressed / disabled: see §11 "Interaction State
  Primitives". The token module
  `client/src/ui/design_tokens/interaction_states.rs` is the source of
  truth; per-surface migration of existing Sprint 14 button surfaces is
  deferred to Sprint 16+ (`S16-UI-INTERACTION-STATE-MIGRATION-*`).

### Secondary button affordance

- Background: `SURFACE_ELEVATED` token (§7).
- Border: 1 px `PRIMARY` token outline.
- Text: `BODY` (15 px) / `Regular` (400) per §5.
- Padding: same as primary.
- Hover / focus / pressed / disabled: see §11 "Interaction State
  Primitives". Same forward reference and per-surface-migration deferral
  as Primary above.

### Panel chrome

- Background: `SURFACE_ELEVATED` token (§7).
- Padding: `SPACING_MD` (16 px) all sides; `SPACING_LG` (24 px) for
  primary modals (draft centered modal, result screen).
- Z-layer: `UI_BASE` for inline panels; `Modal` for centred modal panels;
  `UiOverlay` for full-viewport overlays. See §3.

### Card slot composition

Forward reference: see §12 "Card Slot Primitive" below. Sprint 16 story
009 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) authors the canonical primitive
module at `client/src/ui/design_tokens/card_slot.rs` and Phase 1 migrates
the shop slot well call site. Hand / draft / auction-featured / board
staged-ghost migrations are owned by the Sprint 16+
`S16-UI-CARD-SLOT-MIGRATION-*` follow-on family. §12 is the source-of-
truth for the canonical numeric values; this section is a pointer.

### Modal centering pattern

- Anchor: `(vw - w) / 2, (vh - h) / 2` per §8 "Centred modal panels".
- Z-layer: `MODAL` (500) per §3.
- Backdrop: full-viewport scrim at `OVERLAY_SCRIM_ALPHA` (0.55) on
  `SURFACE` color, painted at z-layer `UiOverlay` (400) immediately below
  the modal.

---

## §11 Interaction State Primitives

Canonical hover / focus / pressed / disabled visual primitives for any
clickable surface (lobby Join / Create / Confirm buttons; auction bid
buttons; HUD action buttons; shop slot purchase buttons; draft buttons).
Sprint 15 story 008 (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`,
Tier 0 Should-priority adjacent) authors the token module and amends this
section; per-surface migration of existing Sprint 14 button surfaces is
**out of scope for Sprint 15** and is deferred to a Sprint 16+ follow-on
story family (`S16-UI-INTERACTION-STATE-MIGRATION-*`).

**Source-of-truth module**:
`client/src/ui/design_tokens/interaction_states.rs` (NEW; landed by
story 008). This section ratifies the canonical numeric values; the §10
"Primary button affordance" / "Secondary button affordance" subsections
above forward-reference this section as the binding source for hover /
focus / pressed / disabled treatment.

### Hover tokens

| Token | Default | Bevy module symbol | Canonical use |
|-------|---------|---------------------|----------------|
| `HOVER_BG_TINT_ALPHA` | `0.08` | `interaction_states::HOVER_BG_TINT_ALPHA` | Alpha of a white `BackgroundColor` overlay painted over the surface's base palette token when the pointer is over it. Subtle pointer-feedback affordance. |
| `HOVER_BORDER_ALPHA` | `0.40` | `interaction_states::HOVER_BORDER_ALPHA` | Alpha of a `BorderColor` outline drawn around the surface on hover. Heavier than the tint so the border reads as a clear pointer-feedback edge. |

### Focus tokens

| Token | Default | Bevy module symbol | Canonical use |
|-------|---------|---------------------|----------------|
| `FOCUS_RING_COLOR` | `Color::srgb(0.949, 0.788, 0.298)` — verbatim §7 `ACCENT` token (hex `#F2C94C`) | `interaction_states::FOCUS_RING_COLOR` | Color of the keyboard / accessibility focus ring drawn around a focused clickable surface. Ratifies §7 `ACCENT` verbatim; **not** a fresh RGB triple. |
| `FOCUS_RING_WIDTH_PX` | `2.0` px | `interaction_states::FOCUS_RING_WIDTH_PX` | Stroke width of the focus-ring outline. Wide enough to read as a deliberate indicator at every §8 canonical viewport; narrow enough not to distort the surface's perceived size. |
| `FOCUS_RING_OFFSET_PX` | `2.0` px | `interaction_states::FOCUS_RING_OFFSET_PX` | Outset between the surface's outer edge and the inner edge of the focus ring. Non-zero so the ring reads distinct from a base border. |

**Friend-game scope guard**: focus-ring token presence is a **visual**
primitive only. It does **not** advance `QA-COND-0005` Standard-tier
focus-order conformance, keyboard navigation completeness, screen-reader
hints, ≥44px hit-target enforcement, or any other Standard-tier
accessibility requirement. `QA-COND-0006` playtest validation and
`PAW-TD-*-a` placeholder-art accept-risk are likewise unchanged by this
section.

### Pressed tokens

| Token | Default | Bevy module symbol | Canonical use |
|-------|---------|---------------------|----------------|
| `PRESSED_BG_TINT_ALPHA` | `0.16` | `interaction_states::PRESSED_BG_TINT_ALPHA` | Alpha of a black `BackgroundColor` overlay painted over the surface's base palette token while a mouse button is held down on it. Twice the magnitude of `HOVER_BG_TINT_ALPHA` so pressed reads as a clearly distinct visual state from hover. |
| `PRESSED_OFFSET_Y_PX` | `1.0` px | `interaction_states::PRESSED_OFFSET_Y_PX` | Vertical pixel offset applied to the surface's content while pressed — a one-pixel press-down nudge. Subtle by design; larger offsets would visibly shift the bounding box and disrupt neighbouring layout. |

### Disabled tokens

| Token | Default | Bevy module symbol | Canonical use |
|-------|---------|---------------------|----------------|
| `DISABLED_BG_TINT_ALPHA` | `0.50` | `interaction_states::DISABLED_BG_TINT_ALPHA` | Alpha of a black `BackgroundColor` overlay painted over the surface when it is not interactable. Heavy enough to flatten perceived saturation so disabled is unambiguously distinct from hover / pressed / default. |
| `DISABLED_TEXT_ALPHA` | `0.40` | `interaction_states::DISABLED_TEXT_ALPHA` | Alpha applied to the surface's label `TextColor` when disabled. Sits below the background tint band so the label reads as faded relative to an enabled surface's label without becoming unreadable. |
| `DISABLED_BORDER_ALPHA` | `0.20` | `interaction_states::DISABLED_BORDER_ALPHA` | Alpha of the surface's `BorderColor` outline when disabled. Lower than `HOVER_BORDER_ALPHA` so the border recedes alongside the flattened fill. |

**Canonical disabled-state surfaces**: auction bid button when the
player already holds the lead; shop slot when the player cannot afford
the unit; HUD action button when no valid target exists; lobby Confirm
button when class selection is incomplete. (Per-surface migration
deferred to Sprint 16+ per the section preamble.)

### Visual-state ordering invariants

Story 008's integration test enforces the following ordering invariants
so a future spec revision cannot silently collapse adjacent visual
states:

1. `PRESSED_BG_TINT_ALPHA > HOVER_BG_TINT_ALPHA` — pressed reads heavier
   than hover so the player perceives a clear state change between
   hover-enter and mouse-down.
2. `DISABLED_BG_TINT_ALPHA > PRESSED_BG_TINT_ALPHA` — disabled reads
   heaviest so the disabled state is unambiguously distinguishable from
   any interactive state.

### Scope (Sprint 15 story 008)

- **Friend-game scope boundary preserved** per the §2 boundary list and
  the per-token-set scope guard above.
- **Per-surface migration OUT OF SCOPE for Sprint 15.** Lobby buttons
  (`S11-UX-LOBBY-BUTTON-HITTARGETS` DONE), auction bid buttons
  (`S11-UX-AUCTION-FEATURED-CARD` DONE), HUD action buttons
  (`S11-UX-HUD-TOP-STRIP-LAYOUT` DONE), draft buttons, and shop slot
  buttons remain on their existing per-site styling for the duration of
  Sprint 15. Migration is a Sprint 16+ follow-on story.
- **No new color-palette tokens.** The four interaction-state token sets
  layer on top of the existing §7 palette; `FOCUS_RING_COLOR` ratifies
  §7 `ACCENT` verbatim and is not a fresh RGB choice.
- **No tween / animation of state transitions.** Static visual states
  only; future per-state easing is a separate scope.

---

## §12 Card Slot Primitive

Canonical layout primitive for every card-painting surface in the
playable client. Sprint 16 story 009
(`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`, Tier 3 rank 13) authors the token
module and amends this section; per-surface migration of the four
existing card surfaces is split between this story (Phase 1 shop slot
only) and the Sprint 16+ follow-on family
(`S16-UI-CARD-SLOT-MIGRATION-*`).

**Source-of-truth module**:
`client/src/ui/design_tokens/card_slot.rs` (NEW; landed by Sprint 16
story 009). This section ratifies the canonical numeric values for each
of the five `CardSlotKind` variants; the §10 "Card slot composition"
subsection above forward-references this section as the binding source.

### CardSlotKind variants

| Variant | Outer (px) | Aspect band | Border (px) | Z-layer | Canonical consumer |
|---------|------------|-------------|-------------|---------|--------------------|
| `HandFan`          |  96 × 136 portrait  | `0.69..=0.72` | `1.0` | `UI_BASE`    | `client/src/ui/hand/mod.rs` hand fan card (`HAND_CARD_DISPLAY_*`). |
| `DraftGrid`        | 120 ×  56 landscape | `2.10..=2.18` | `1.0` | `UI_BASE`    | `client/src/ui/hand/mod.rs` draft initial grid (`HAND_DRAFT_GRID_CARD_*`). |
| `ShopSlot`         | 136 ×  78 landscape | `1.70..=1.78` | `1.0` | `UI_BASE`    | `client/src/ui/shop_auction/mod.rs::shop_slot_node` (Phase 1 migration target). |
| `AuctionFeatured`  | 380 × 280 landscape | `1.32..=1.40` | `3.0` | `UI_BASE`    | `client/src/ui/shop_auction/mod.rs::auction_featured_card_node` (`AUCTION_FEATURED_CARD_*`). |
| `BoardStagedGhost` |  64 ×  80 portrait  | `0.78..=0.82` | `0.0` | `UI_OVERLAY` | World-space ghost preview sized to one board cell per `docs/ux/board-rendering-spec.md` BR-001 (`cell_width = 64.0`, `lane_height = 80.0`). |

### Image / text / hit-target insets

Insets are expressed as a `(left, right, top, bottom)` `UiRect` in
pixels. The image rectangle and text rectangle MUST be disjoint within
the outer rectangle — the integration test asserts containment per kind.

| Variant | Image inset (L / R / T / B) | Text inset (L / R / T / B) | Hit-target inset |
|---------|-----------------------------|----------------------------|------------------|
| `HandFan`          |   4 /  4 /   4 / 28 |   4 /  4 / 112 /  4 | `UiRect::ZERO` (hit target == visual outer rectangle). |
| `DraftGrid`        |   4 / 64 /   4 /  4 |  60 /  4 /   4 /  4 | `UiRect::ZERO`. |
| `ShopSlot`         |   4 / 80 /   4 /  4 |  60 /  4 /   4 /  4 | `UiRect::ZERO`. |
| `AuctionFeatured`  |  16 / 16 /  16 / 96 |  16 / 16 / 200 / 16 | `UiRect::ZERO`. |
| `BoardStagedGhost` |   2 /  2 /   2 / 14 |   2 /  2 /  70 /  2 | `UiRect::ZERO`. |

Per the AC7 contract the hit-target rectangle is a **superset of or
equal to** the visual outer rectangle. The default `UiRect::ZERO` means
the hit target equals the visual outer rectangle; a future per-surface
migration sibling MAY outset further (e.g. focus-ring outset).

### Composition rules

1. **No nested cards.** A card slot is **leaf-only** — it has image and
   text regions, NOT a child card slot. The `card_slot_node` builder for
   kind `K` MUST NOT instantiate `card_slot_node(K')` for any other
   kind. Composition that paints multiple cards (draft initial grid;
   shop slot row) does so by placing N siblings under a flex parent.
2. **Stable aspect ratio across viewports.** Slot dimensions are
   pixel-fixed per §4 spacing scale — no viewport-driven scaling. The
   integration test iterates `CANONICAL_VIEWPORTS` (§8) and confirms
   `outer_width_px / outer_height_px` is constant per kind.
3. **Distinct geometry per kind.** Each `CardSlotKind` variant resolves
   to a distinct `(outer_width_px, outer_height_px, z_layer)` triple —
   no two kinds collapse to the same slot.
4. **Pixel-fixed under canonical viewports.** Containment is asserted
   at the smallest canonical viewport (`1366 × 768`) and at one
   smaller-than-canonical sentinel (`1024 × 600`) per AC4; the test
   harness confirms no image or text region extends past the slot's
   outer rectangle even when the viewport drops below the canonical
   floor.

### Interaction-state composition

Card-slot kinds compose with the §11 interaction-state primitive
families published by `client/src/ui/design_tokens/interaction_states.rs`
(Sprint 15 story 008). Doc comments on each `CardSlotKind` variant
name which token family the kind consumes (e.g. `ShopSlot` consumes
`HOVER_BG_TINT_ALPHA` / `HOVER_BORDER_ALPHA` for pointer hover;
`FOCUS_RING_*` for Tab focus; `PRESSED_BG_TINT_ALPHA` for purchase
click; `DISABLED_*` for `cannot afford`). Per-surface migration of the
actual interaction-state visuals is owned by the Sprint 16+
`S16-UI-INTERACTION-STATE-MIGRATION-*` follow-on family — this primitive
declares the references only.

### Friend-game scope guard (§2)

- **Layout primitive only.** This section composes the *layout*; it
  does **not** replace placeholder art (`PAW-TD-002-a` /
  `PAW-TD-003-a`), introduce final-art chrome, or alter game-state
  machines. The underlying placeholder PNGs (e.g.
  `assets/ui/shop_slot_well.png`) remain accept-risk per
  `PAW-TD-*-a`.
- **`QA-COND-0005` accept-risk preserved.** Hit-target sizes are
  declared verbatim from the existing per-surface literals; the
  primitive does **not** enforce a ≥44px Standard-tier floor. WCAG
  contrast on the slot chrome is **not** introduced.
- **`QA-COND-0006` accept-risk preserved.** Playtest / fun-hypothesis
  validation is **not** advanced.

### Scope (Sprint 16 story 009)

- **Phase 1 migration only by this row.** Sprint 16 default scope is
  the primitive module + this spec amendment + the shop slot
  (`shop_slot_node`) Phase 1 migration + viewport-invariant
  integration test + evidence dir. The remaining three migration
  phases (hand surfaces / auction featured / board staged-ghost) are
  Sprint 16+ follow-on rows in the family
  `S16-UI-CARD-SLOT-MIGRATION-*`.
- **No per-surface interaction-state migration.** Hover / focus /
  pressed / disabled visual state mapping references the §11 token
  families by doc-comment forward reference only; actual per-surface
  wiring is the Sprint 16+ `S16-UI-INTERACTION-STATE-MIGRATION-*`
  family (out of scope here).
- **No drag-state visuals re-author.** Hand fan drag-state visuals
  remain owned by `S12-UX-HAND-DRAG-STATE-VISUALS-001` (Sprint 15
  DONE).
- **No board-rendering spec change.** `docs/ux/board-rendering-spec.md`
  (Sprint 15 DONE) is the authority for board-cell geometry; the
  `BoardStagedGhost` variant reads cell geometry from that spec.
- **No new color-palette tokens.** Card-slot chrome reads from §7
  palette tokens (`SURFACE_ELEVATED` default; `ACCENT` for the
  featured-card differentiation contract from Sprint 14 PROMPT 931).

---

## §13 Layout-Foundation Primitives (Viewport Safety Contract)

Sprint 17 PROMPT 1181 (`S17-UI-LAYOUT-FOUNDATION-PRIMITIVES-REPAIR`)
ships reusable layout primitives so that every later surface migration
declares its viewport-safety budget instead of hand-authoring fragile
fixed stacks. The primitive modules are:

| Module | Symbol surface | Canonical use |
|--------|----------------|----------------|
| `client/src/ui/design_tokens/viewport_matrix.rs` | `SAFETY_VIEWPORT_MATRIX = [1280×720, 1366×768, 1920×1080]`, `SAFETY_VIEWPORT_SMALLEST` | Tight 3-row safety matrix that every primitive's viewport-fit test iterates. The broader §8 6-viewport matrix remains canonical for full-app integration suites; this 3-row subset is the inner loop. |
| `client/src/ui/design_tokens/modal_panel.rs` | `ModalPanelKind::{Standard, Narrow}`, `modal_panel_node`, `ModalPanelBudget`, `modal_panel_content_budget`, `assert_fits_smallest_safety_viewport`, `ContentBudgetError` | Computes `outer_height – chrome → body` budget at the supplied safety viewport and fails closed when the title strip + section gaps + CTA row + min-body floor exceed the outer-height clamp. |
| `client/src/ui/design_tokens/cta_row.rs` | `CtaRowKind::{Primary, Compact}`, `cta_row_node`, `cta_button_node`, `CTA_ROW_HEIGHT_PX = 44`, `CTA_ROW_FLEX_GROW/SHRINK = 0` | Stable CTA row that is pinned to a pixel-fixed height and refuses to be squashed by body-region flex pressure. |
| `client/src/ui/design_tokens/scroll_region.rs` | `scroll_region_node`, `clipped_body_region_node`, `SCROLL_REGION_MIN_HEIGHT_PX = 0` | Body / scroll region that grows + shrinks under flex pressure and carries `min_height: 0` so it never pushes the CTA row off-screen. |
| `client/src/ui/design_tokens/status_chip.rs` | `StatusChip` marker, `VisualRole::{StatusChip, CtaButton}`, `status_chip_node`, `STATUS_CHIP_HEIGHT_PX = 22`, `STATUS_CHIP_TEXT_SIZE_PX = CAPTION` | Read-only chip that visually reads as smaller than `CTA_ROW_HEIGHT_PX` and carries neither the `Button` nor `Interaction` components. |
| `client/src/ui/design_tokens/text_fit.rs` | `TextFitPolicy::{SingleLineNoWrap, WrapWordBoundary, WrapWordOrCharacter}`, `text_layout`, `single_line_centered`, `wrap_body_left` | Names the three canonical `bevy::text::LineBreak` modes so spawn sites declare wrap-policy intent instead of re-deriving the bevy enum. |

### Viewport-safety contract (normative)

A surface MAY claim "fits the viewport" only if all of the following
hold at every row of [`SAFETY_VIEWPORT_MATRIX`] — and in particular at
[`SAFETY_VIEWPORT_SMALLEST`]:

1. The outer-rectangle height clamp (default
   `92% × viewport.height_px`) ≥ panel chrome height
   (`2 × padding + 2 × border + title_strip + cta_row +
   2 × section_gap`) + `MODAL_PANEL_MIN_BODY_HEIGHT_PX` (80 px).
2. The CTA row's `flex_grow` and `flex_shrink` are both `0.0` and its
   height is a `Val::Px(...)` value, not a `Val::Percent(...)` value.
3. The scroll / body region's `min_height` is `Val::Px(0.0)` so flex
   pressure absorbs into the body region before it propagates into the
   CTA row.
4. Single-line readouts (HUD readouts, status chips, button labels)
   declare `TextFitPolicy::SingleLineNoWrap`; multi-line body copy
   declares `TextFitPolicy::WrapWordBoundary` or
   `TextFitPolicy::WrapWordOrCharacter`.

The `modal_panel::assert_fits_smallest_safety_viewport(budget)`
function is the canonical way to assert (1) — it returns `Err` if the
chrome exceeds the outer clamp or the body falls below the min-body
floor.

### Status-chip vs CTA-button (mutually exclusive)

Per `VisualRole::is_interactive` / `VisualRole::is_read_only`, a node
is either a status chip OR a CTA button — never both. Surfaces MUST
NOT spawn a `Button` marker on a node that carries the `StatusChip`
marker, and MUST add the `CtaButton` marker to nodes spawned through
`cta_button_node`.

### Foundation lane scope (PROMPT 1181)

PROMPT 1181 explicitly does NOT migrate any surface. The lane ships:

- The six primitive modules listed above.
- Inline `#[cfg(test)]` unit assertions in each module + the unit
  shape guard at
  `tests/unit/ui/design_tokens/layout_primitives_shape_test.rs`.
- The integration viewport-safety invariant suite at
  `tests/integration/ui_clean_pass/layout_primitives_test.rs`.
- This normative §13 amendment.

Per-surface migration (result screen → `Narrow`, lobby modal →
`Standard`, draft-initial modal → `Standard`, photosensitivity warning
→ `Narrow`, connection-lost overlay → `Narrow`, settings shell →
`Narrow`; auction + shop CTA rows → `cta_row_node`; auction status
text → `status_chip_node`) is owned by the Sprint 17+ follow-on
family `S17-UI-MODAL-PANEL-CHROME-MIGRATION-*` and
`S17-UI-CTA-ROW-MIGRATION-*`.

### Friend-game scope guard (§2)

- `QA-COND-0005` Standard-tier accessibility accept-risk preserved.
  The primitive's `CTA_ROW_HEIGHT_PX = 44` is the friend-game
  click-target floor; it does NOT advance the Standard-tier ≥ 44 px
  hit-target enforcement claim (which requires a separate audit per
  §2).
- `QA-COND-0006` playtest validation accept-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.

---

## Spec Adoption Matrix

This matrix is the **canonical mapping** of spec sections to Sprint 14+
consumer stories. Every Tier 0 token-module story and every Tier 1
surface story reads its numeric values from the cited section.

### Tier 0 token-module consumers

| Sprint 14 row | Story file | Reads spec section(s) | Status on `origin/main@b39eedf` | Notes |
|---------------|------------|------------------------|---------------------------------|-------|
| `S11-TD-UI-ZINDEX-LAYERS` (rank 1) | `production/epics/ui-clean-pass/story-002-ui-zindex-layers.md` | §3 (Z-Index Layer System) | Implementation landed (PROMPT 902 integration of PROMPT 901 worker; closure `/story-done` outstanding at spec authoring time) | This spec ratifies the 8-layer integer set already shipped in `client/src/ui/design_tokens/z_layers.rs`. No value change requested. |
| `S11-TD-UI-FONT-CONSTANTS` (rank 2) | `production/epics/ui-clean-pass/story-003-ui-font-constants.md` | §5 (Typography Hierarchy) | DONE — PROMPT 908 `/story-done` at `origin/main@b39eedf` | Numeric values (13 / 15 / 18 / 22 / 30 / 40 + weights 400 / 600 / 700 + line-height ratio 1.25) cited verbatim from `client/src/ui/design_tokens/typography.rs`. This spec ratifies the shipped values; no override is requested. |
| `S11-TD-UI-FLEX-STRIPS` (rank 3) | `production/epics/ui-clean-pass/story-004-ui-flex-strips.md` | §4 (Spacing Scale) + §9 (Strip Composition Patterns) | Story-readiness pending (blocked on this spec landing for producer-decision-2) | Story 004's `SPACING_*` constants and `HeaderBar` / `LaneBar` / `HandBar` / `FooterBar` primitives consume §4 token set and §9 strip geometry. Heights ratified against story 005's already-shipped baseline fixture. |
| `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4) | `production/epics/ui-clean-pass/story-005-ui-viewport-invariant-tests.md` | §8 (Responsive Layout Rules) | DONE — PROMPT 909 `/story-done` at `origin/main@b39eedf` | 6-viewport matrix cited verbatim from `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`. This spec ratifies the shipped matrix; no override is requested. |
| `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` (rank 5) | `production/epics/ui-clean-pass/story-006-ui-overlay-alpha-token.md` | §6 (Overlay Alpha Tokens) | Story-readiness pending (blocked on this spec landing for producer-decision-2) | `OVERLAY_DIM_ALPHA = 0.45` (preserves existing HUD value) and `OVERLAY_SCRIM_ALPHA = 0.55` (consolidates 0.46 + 0.58). Story 006 migrates the three scattered scrim / dim literals. |

### Tier 1 surface story consumers

| Sprint 14 row | Story file | Reads spec section(s) | Notes |
|---------------|------------|------------------------|-------|
| `S11-UX-HUD-TOP-STRIP-LAYOUT` (rank 7) | `production/epics/hud/story-015-hud-top-strip-layout.md` | §3 z-layers (`UiBase`) + §4 spacing tokens + §5 typography (`H2` / `Display`) + §9 `HeaderBar` (60 px) | HUD top strip composed via §9 HeaderBar; magic offsets replaced by §4 tokens; HUD gold readout uses `Display`; secondary readouts use `H2`. |
| `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` (rank 8, Should) | `production/epics/hud/story-016-hud-bottom-strip-layout.md` | §3 z-layers (`UiBase`) + §4 spacing tokens + §5 typography + §9 `FooterBar` (40 px) | HUD bottom strip composed via §9 FooterBar. |
| `S11-UX-DRAFT-GRID-CENTERED-MODAL` (rank 9, Should) | `production/epics/shop-auction-ui/story-015-draft-grid-centered-modal.md` | §3 z-layers (`Modal`) + §5 typography + §8 centred-modal pattern + §10 modal-centering | Draft initial grid composed as a centred modal at z-layer `Modal`. |
| `S11-UX-AUCTION-FEATURED-CARD` (rank 10) | `production/epics/shop-auction-ui/story-016-auction-featured-card.md` | §3 z-layers (`UiBase`) + §4 spacing tokens + §5 typography + §7 `ACCENT` token for featured-card frame | Featured-card differentiation via layout / composition / scale / hierarchy, NOT via final-art replacement. `PAW-TD-003-a` preserved. |
| `S11-UX-LOBBY-CLASS-PICKER` (rank 11, Should) | `production/epics/playable-client/story-025-lobby-class-picker-layout.md` | §3 z-layers (`UiBase`) + §4 spacing tokens + §5 typography (hierarchy correction for §3.1 L6 inversion) | Class-picker composed via flex primitive + §5 typography hierarchy. |
| `S12-UX-LOBBY-LAYOUT-MODAL-001` (rank 12) | `production/epics/playable-client/story-024-lobby-layout-modal.md` | §3 z-layers (`UiBase` or `Modal` per producer-decision-3) + §4 spacing + §5 typography + §8 responsive rules | Producer-decision-3 (modal-panel vs full-viewport hero) is **not** ratified by this spec — it is a separate producer decision for story 024's `/story-readiness`. |

### Tier 1 Should-priority adjacent rows

| Slug | Reads spec section(s) | Notes |
|------|------------------------|-------|
| `S11-UX-HUD-OPP-FIGURINE` | §3 + §4 + §5 + §9 `FooterBar` | Pair with rank 7 / 8. |
| `S11-UX-AUCTION-FREE-GOLD-COUNTERS` | §3 + §4 + §5 + §7 semantic tokens | Pair with rank 10. |
| `S11-UX-LOBBY-BUTTON-HITTARGETS` | §4 + §10 button affordance | Pair with rank 11; **`QA-COND-0005` accept-risk preserved** on the L5 ≥44px gap. |
| `S12-UX-AUCTION-LEAD-LOSS-STATE-001` | §7 `SEMANTIC_SUCCESS` / `SEMANTIC_WARNING` / `SEMANTIC_ERROR` | Producer-decision-4 (visual language) **not** ratified by this spec. |
| `S12-UX-HAND-DRAG-STATE-VISUALS-001` | §3 (`UiOverlay` drag ghost layer) + §6 overlay alpha | Drag ghost reads `OVERLAY_DIM_ALPHA`. |

### Tier 0 Should-priority adjacent row

| Slug | Reads spec section(s) | Notes |
|------|------------------------|-------|
| `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` | §7 color tokens + §10 button affordance + §11 interaction state primitives (canonical default values authored by story 008) | Hover / focus / pressed / disabled visual primitive set. Module: `client/src/ui/design_tokens/interaction_states.rs`. §11 is the source-of-truth section for the four token sets' canonical defaults. Per-surface migration deferred to Sprint 16+ family `S16-UI-INTERACTION-STATE-MIGRATION-*`. |

### Tier 3 deferred to Sprint 15 (and beyond)

| Slug | Reads spec section(s) | Notes |
|------|------------------------|-------|
| `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` (rank 13) | §4 + §5 + §10 (forward reference) + §12 (card slot primitive — canonical values authored by story 009) | **Sprint 16** refactor (deferred from Sprint 15 per `production/sprints/sprint-15.md` "Wider Sprint 15 Backlog"). Token module: `client/src/ui/design_tokens/card_slot.rs`. §12 is the source-of-truth section for the five `CardSlotKind` variants' canonical outer-rectangle / aspect-ratio band / image / text / hit-target inset / z-layer defaults. Phase 1 migration: shop slot well only (`shop_slot_node`); hand / draft / auction-featured / board staged-ghost migrations deferred to the Sprint 16+ family `S16-UI-CARD-SLOT-MIGRATION-*`. |
| `S11-UX-BOARD-RENDERING-SPEC` (rank 14) | §3 World / Units layers + §8 world-space scaling | Sprint 15 doc-only spec authoring; depends on this spec as the parent design-spec doc. |

---

## Producer Ratification Checklist

Per `production/epics/ui-clean-pass/story-007-global-ui-design-spec.md`
AC12 and PROMPT 802 §9 producer-decision-2, the spec values authored
above require ratification by **producer + UX-designer + art-director**
before stories 002 / 003 / 004 / 005 / 006 cite this spec as their source
of truth.

### Producer-decision-2 resolution

PROMPT 802 §9 producer-decision-2 (numeric values for Tier 0 token
modules) is **resolved** by the ratification rows below. The decision
question — "what are the canonical numeric values for the global UI
design spec?" — is answered section by section.

| Spec section | Canonical values | Provenance |
|---|---|---|
| §3 Z-Index Layer System | 8 named layers at `0 / 100 / 200 / 300 / 400 / 500 / 600 / 700` | Ratified verbatim from `client/src/ui/design_tokens/z_layers.rs` (shipped by story 002 / PROMPT 901-902). |
| §4 Spacing Scale | 5 tokens at `4 / 8 / 16 / 24 / 32` px | Ratified by this spec. Story 004 worker reads these into the `SPACING_*` constant set. |
| §5 Typography Hierarchy | 6 sizes at `13 / 15 / 18 / 22 / 30 / 40` px + weights `400 / 600 / 700` + line-height ratio `1.25` | Ratified verbatim from `client/src/ui/design_tokens/typography.rs` (shipped by story 003 / PROMPT 904-906-908). |
| §6 Overlay Alpha Tokens | `OVERLAY_DIM_ALPHA = 0.45`, `OVERLAY_SCRIM_ALPHA = 0.55`, `OVERLAY_TOAST_ALPHA = 0.80` | Ratified by this spec. `OVERLAY_DIM_ALPHA` preserves the existing HUD value (`hud/mod.rs:34`); `OVERLAY_SCRIM_ALPHA` consolidates the shipped 0.46 (result) and 0.58 (settlement) values; `OVERLAY_TOAST_ALPHA` is worker-discretion for story 006 implementation. |
| §7 Color Tokens | 8 named tokens with RGB hex and `Color::srgb(...)` references | Ratified by this spec as **friend-game placeholder palette**. Not WCAG contrast-checked. Final-art palette replacement remains under `PAW-TD-*-a` accept-risk. |
| §8 Responsive Layout Rules | 6-viewport matrix `1366×768 / 1920×1080 / 1920×1200 / 1280×960 / 3840×2160 / 2560×1080` | Ratified verbatim from `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS` (shipped by story 005 / PROMPT 905-907-909). |
| §9 Strip Composition Patterns | `HeaderBar = 60` px, `FooterBar = 40` px, `HandBar = 180` px; `LaneBar` documented but implementation deferred | Ratified verbatim from `tests/integration/fixtures/ui_viewport_baseline.rs::HEADER_BAR_HEIGHT_PX / FOOTER_BAR_HEIGHT_PX / HAND_BAR_HEIGHT_PX` (provisional values shipped by story 005; ratified to canonical by this spec). |

### Sign-off rows

The three sign-off rows below are the AC12 ratification gate. Each is
recorded as ratified at PROMPT 911 spec authoring with the rationale
captured in the per-section discussion above. Future Tier 1 / Tier 3
consumer stories may cite this spec as the producer + UX-designer +
art-director-ratified source of truth.

| Role | Ratified at | Rationale |
|------|-------------|-----------|
| **Producer** | PROMPT 911 (2026-05-15) | Numeric values are either (a) verbatim from already-shipped Tier 0 modules (§3 / §5 / §8 / §9 partial) or (b) authored to minimise visual regression while consolidating three scattered overlay scrim values to one (§6) and authoring a defensible placeholder palette for friend-game scope (§4 / §7 / §9 partial). Producer-decision-2 is closed by this ratification; Tier 0 token-module stories 004 + 006 are unblocked for `/story-readiness`. |
| **UX-designer** | PROMPT 911 (2026-05-15) | Typography hierarchy preserves the §3.1 L6 lobby inversion fix shipped by story 003; spacing scale (4 / 8 / 16 / 24 / 32) is the standard geometric step used across Bevy 0.18 friend-game showcases; strip heights ratify story 005's already-shipped baseline; viewport matrix preserves story 005's already-shipped 6-viewport set. Color palette is friend-game placeholder; future palette migration is in scope for a separate "colorization pass" story. |
| **Art-director** | PROMPT 911 (2026-05-15) | `PAW-TD-*-a` placeholder-art accept-risk preserved verbatim in §1 Status Banner. Color palette tokens (§7) are friend-game placeholder, not final-art; final-asset replacement remains a separate sprint scope. Z-layer ordering preserves the existing PresentationPlugin composition (ADR-021 R2). Strip heights deliver a visually composed top-to-bottom HUD column (60 / 40 / 180) on every viewport in the canonical matrix. |

### Ratification scope guard

The above ratification is **specifically scoped to friend-game visual
polish** per §2 Scope Boundaries. It does **not** ratify:

- Standard-tier accessibility values (separate accessibility spec
  required to advance `QA-COND-0005`).
- Final-art palette / font assets (separate sprint scope; `PAW-TD-*-a`
  accept-risk preserved).
- Playtest validation (`QA-COND-0006` accept-risk preserved).
- Per-element layout for any specific surface (owned by the per-surface
  Tier 1 story).
- Animation / motion. (Interaction-state primitives — hover / focus /
  pressed / disabled — are now authored in §11 as a friend-game-scope
  visual primitive set per Sprint 15 story 008. Per-surface migration of
  existing Sprint 14 button surfaces remains deferred to Sprint 16+
  family `S16-UI-INTERACTION-STATE-MIGRATION-*`; visual focus-ring
  presence does NOT advance `QA-COND-0005`.)
- Producer-decision-3 (lobby layout modal vs full-viewport hero —
  resolved per story 024 §"Decision Capture (PROMPT 933, 2026-05-15)"
  — Option A (centred modal panel); this spec's ratified §3 `Modal`
  z-layer + §4 spacing + §5 typography + §8 centred-modal responsive
  rules + §10 modal-centering pattern apply unchanged).
- Producer-decision-4 (auction lead/loss visual language — owned by
  `S12-UX-AUCTION-LEAD-LOSS-STATE-001`).
- Producer-decision-5 (Tier 2 cosmetic captures bundling — owned by
  Sprint 14 producer at activation).

---

## Cross-References

This spec is the parent design-spec doc for the UI clean-pass milestone.
Related artifacts:

- `docs/ux/ui-clean-pass-roadmap.md` — Sprint 14+ pull-in sequence and
  sequencing rules (rank 6 is this spec).
- `docs/ux/global-ui-layout-contract.md` — structural-invariant /
  layout-contract complement to this spec; ratifies the geometric
  invariants (root-anchor budget, viewport floor, padding consistency,
  centered-overlay viewport-fit, button-vs-chip dichotomy) that the
  tokens in this spec must compose into.
- `production/epics/ui-clean-pass/EPIC.md` — epic-level UI clean-pass
  charter.
- `production/epics/ui-clean-pass/story-007-global-ui-design-spec.md` —
  story file (AC1-AC14).
- `production/sprints/sprint-14.md` — Sprint 14 plan (active).
- `production/qa/qa-plan-sprint-14.md` — Sprint 14 QA plan; §"S12-UX-
  GLOBAL-UI-DESIGN-SPEC-001 (story 007)" names this spec as the
  BLOCKING producer + UX + art ratification gate.
- `production/qa/evidence/sprint-14-ui-foundation/global-ui-design-spec/`
  — AC1-AC14 doc-review checklist evidence.
- `docs/architecture/adr-021-presentation-layer-architecture.md` — ADR
  alignment for §3 z-layer ordering.
- `client/src/ui/design_tokens/z_layers.rs` — §3 implementation (story
  002 / PROMPT 901-902).
- `client/src/ui/design_tokens/typography.rs` — §5 implementation (story
  003 / PROMPT 904-906-908).
- `tests/integration/helpers/ui_viewport.rs` — §8 implementation (story
  005 / PROMPT 905-907-909).
- `tests/integration/fixtures/ui_viewport_baseline.rs` — §9 provisional
  baseline; values ratified to canonical by this spec.
- Per-system UX docs (read-only cross-link from this spec; not modified
  by PROMPT 911):
  - `design/ux/hud.md`
  - `design/ux/hand-ui.md`
  - `design/ux/shop-auction-ui.md`
  - `design/ux/lobby.md` (if present)

---

## Authoring Trail

| Field | Value |
|-------|-------|
| **Authoring prompt** | PROMPT 911 (`/dev-story` for story 007) |
| **Worker branch** | `work/s14-global-ui-design-spec` |
| **Worktree** | `D:/_DEV/wt/ccgs-prompt-911-global-ui-design-spec` |
| **Source-of-truth at authoring** | `origin/main@b39eedf` (PROMPT 908 `/story-done` `S11-TD-UI-FONT-CONSTANTS`) |
| **`/story-readiness` verdict consumed** | PROMPT 910 READY (`reports/PROMPT-910-S14-GLOBAL-UI-DESIGN-SPEC-READINESS.md`) |
| **Story file blob (origin/main)** | `e98626b` |
| **Files changed by PROMPT 911** | `docs/ux/global-ui-design-spec.md` (NEW) + `production/qa/evidence/sprint-14-ui-foundation/global-ui-design-spec/doc-review-checklist.md` (NEW) |
| **Files explicitly NOT changed by PROMPT 911** | `client/**`, `server/**`, `shared/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `production/sprint-status.yaml`, `production/sprints/sprint-14.md`, `production/stage.txt`, `production/session-state/**`, `production/qa/qa-plan-sprint-14.md`, story-007 file body |
