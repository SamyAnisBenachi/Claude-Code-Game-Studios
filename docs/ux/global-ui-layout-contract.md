# Global UI Layout Contract — Sprint 18 UI Architecture Hardening

> **Authoring prompt**: PROMPT 1188 (`S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001`)
> **Source audit**: `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md`
> §5 (Global Acceptance Contract — Going Forward) and §6 Lane K.
> **Roadmap context**: PROMPT 1180 Lane K, third wave (after PROMPT 1190
> PlayArea container + PROMPT 1191 live-spawn viewport harness).
> **Parent design spec**: `docs/ux/global-ui-design-spec.md` (Sprint 14
> friend-game visual polish source-of-truth). This contract **extends**,
> does not replace, that spec.

This document is the **canonical layout contract** for every bevy_ui
surface in the playable client. Where the parent design spec
(`docs/ux/global-ui-design-spec.md`) ratifies *tokens* (z-layers,
typography, spacing, overlay alpha, palette, strip primitives, card-slot
primitive, interaction-state primitives), this contract ratifies the
*structural invariants* that consume those tokens: which CTAs must
be visible, which panels must scroll, which sites may declare
`Overflow::visible()`, and how a button is told apart from a status chip.

---

## §1 Status / No-Claim Banner

This contract is **paperwork + a lint test**. PROMPT 1188 authors only
`docs/ux/global-ui-layout-contract.md` (this file) and
`tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs`. It does
**not** change any client / server / shared production code, any
existing test, any sprint plan, any sprint-status row, any orchestrator
state file, any QA-plan file, or any session-state file.

### What this contract does NOT claim

- Public release readiness; release-candidate readiness; full game
  completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Sprint 17 close-out; Sprint 18 activation; gate-check retry; stage
  advance from `Polish` to `Release`.
- Final-art / asset-production completion (`PAW-TD-*-a`).
- That the *current* playable client already satisfies the invariants
  below — it does not (PROMPT 1180 enumerated six structural overlaps).
  This contract is the *target state*; the lint test ships as an
  allowlisted baseline so future regressions are detectable without
  blocking the current branch.

### Accept-risk dispositions preserved verbatim

- `QA-COND-0005` — Standard-tier accessibility remains accept-risk
  (friend-game scope only). The C-2 "no off-screen primary CTA"
  invariant below is a friend-game visibility check, **not** a WCAG
  hit-target / contrast / focus-order conformance check.
- `QA-COND-0006` — playtest validation accept-risk preserved.
- `PAW-TD-*-a` — placeholder-art accept-risk preserved. The C-6
  image-fitting rule binds `NodeImageMode`, not asset replacement.

---

## §2 Scope Boundaries

This contract governs **friend-game scope** structural UI invariants
only. It is the single source of truth for:

- Supported viewport matrix (§3).
- Primary-CTA visibility invariant (§4).
- Overflow / scroll / wrap rules (§5).
- Button-vs-status-chip visual semantics (§6).
- Panel max-height / content-budget rules (§7).
- Text / image fitting rules (§8).
- Live test strategy (§9).

It is **not** the spec for:

- Token numeric values — owned by `docs/ux/global-ui-design-spec.md`
  (z-layers §3, typography §5, spacing §4, overlay alpha §6, palette §7,
  strip primitives §9, interaction-state primitives §11, card-slot
  primitive §12).
- Per-element layout (HUD top-strip child order, lobby form sequencing,
  shop slot well composition) — owned by per-surface stories.
- Board-rendering — owned by `docs/ux/board-rendering-spec.md`.
- Standard-tier accessibility — separate spec; preserves `QA-COND-0005`.
- Final-art / asset-production — separate scope; preserves `PAW-TD-*-a`.
- Animation / motion / tweens.
- Localization (string layout, RTL, text expansion).

Cross-reference rule: when a structural invariant in this contract
depends on a numeric token value, the token row in the parent design
spec is the binding source. This contract cites the section, never
restates the number.

---

## §3 Supported Viewport Matrix (C-1)

Extends `docs/ux/global-ui-design-spec.md` §8 by **adding a 1280×720
floor**. The parent spec ratifies a 6-viewport set starting at 1366×768;
the dev playable client launches at 1280×720 (PROMPT 1129 §0 evidence
cut), so the contract floor moves down to match the actual runtime
launch.

| Tier | Resolution | Aspect | Required | Required CTA visibility | Notes |
|------|------------|--------|----------|--------------------------|-------|
| **Floor**   | 1280 × 720  | 16:9  | YES | All primary CTAs fully visible | NEW vs parent spec §8; matches dev runtime launch. |
| **Min**     | 1366 × 768  | 16:9  | YES | All primary CTAs fully visible | Parent spec §8 minimum supported viewport. |
| **Baseline**| 1920 × 1080 | 16:9  | YES | Same | Design-source viewport; baseline captures land here. |
| **16:10**   | 1920 × 1200 | 16:10 | YES | Same | 16:10 monitor variant. |
| **4:3**     | 1280 × 960  | 4:3   | YES | Same | Legacy 4:3 monitor; aspect-stretch chrome acceptable. |
| **4K**      | 3840 × 2160 | 16:9  | YES | Same | 4K scale-up boundary. |
| **Ultrawide**| 2560 × 1080 | 21:9  | YES | Same | Ultrawide aspect-stretch boundary. |

**Sub-floor sentinel**: the card-slot primitive integration test
(`tests/integration/ui_clean_pass/card_slot_primitive_test.rs` AC4)
already exercises `1024 × 600` as a *containment-only* sentinel. This
contract does not promote the sub-floor sentinel to a supported tier;
surfaces only need to remain non-clipping there, not pixel-perfect.

**No new floor below 1280×720** is supported by this contract.
1280×720 already constrains the entire UI's vertical budget heavily
(see §7 panel max-height); dropping further would force a UX-density
re-design that is out of scope.

---

## §4 Primary CTA Visibility Invariant (C-2)

For every `RoundPhase × ClientState` cell the playable client can
reach, every primary CTA must satisfy the on-screen bounds predicate
at every viewport in §3.

### CTA list (binding)

| Surface | Primary CTAs |
|---------|--------------|
| Lobby | Create room, Join room, Refresh rooms, Requested-slot pick, Class pick, Confirm class, Return to lobby (post-game). |
| Draft Initial | Pick card slot (`HandDraftGridSlotRoot` × 9), Confirm draft (if applicable). |
| Draft Shop | Buy slot N, Refresh shop, Ready. |
| Draft Auction | Bid +K (×3 increment buttons), Pass, Confirm bid (if applicable). |
| Placement | Submit placement, Unstage (per-card affordance). |
| Resolution | (no primary CTA — observation phase) |
| Game Over | Return to lobby, Acknowledge. |
| Settings | Back / Close, Footer-close (Apply if present), Per-control toggles / steppers. |
| Photosensitivity warning | Acknowledge. |
| Connection lost | (informational; no CTA required at present.) |

### On-screen bounds predicate

For every CTA entity `e` at every viewport `(vw, vh)` in §3:

```text
e.global_x ≥ 0
e.global_y ≥ 0
e.global_x + e.computed_width  ≤ vw
e.global_y + e.computed_height ≤ vh
```

The predicate is asserted by the live-spawn harness authored by
**PROMPT 1191** (Lane B). This contract does not author the harness;
it declares the invariant the harness must enforce.

**Visibility precondition**: the predicate is asserted only when the
CTA's `Visibility` is `Visible` or `Inherited` and its containing
phase / state cell is the active cell. Hidden CTAs are skipped.

**Z-layer precondition**: the predicate is asserted on the CTA's
**own** `ComputedNode` rectangle, not the parent strip's rectangle.
A CTA that overflows its strip parent (S-01 / S-02 from the audit)
fails the predicate even when the parent strip is on-viewport.

---

## §5 Overflow / Scroll / Wrap Rules (C-3)

Every panel with `flex_direction: Column` + variable child count
**must** declare one of three overflow strategies:

1. **`overflow: Overflow::scroll_y()` with a `max_height` clamp** —
   default strategy for modal panels and side-rail content panes
   (lobby room list, settings content pane, draft modal body).
2. **`flex_wrap: FlexWrap::Wrap` with a `max_width` clamp** —
   default strategy for top-strip / footer-strip readout rows that
   would otherwise overflow horizontally (HUD top strip pills at
   1280×720 / 1366×768).
3. **Pagination component + visible row cap** — when the child count
   is unbounded (lobby room list across many open rooms; shop slot
   well across multi-page offers). The component must declare the
   row cap as a constant; the spawn site must clamp to it.

### `Overflow::visible()` allowlist

`Overflow::visible()` is allowed only:

- On strip primitives where overflow is the design intent (the
  `HandBar` parent in `client/src/ui/design_tokens/strips.rs` —
  the hand fan extends above the strip into the play area). The
  audit identified this as the **single legitimate** use today.
- At any other site, only with an inline justification comment
  matching the regex `// AC: [^\n]+` (e.g. `// AC: TICKET-1234 — drag
  ghost must extend past HandBar`) on the same statement or the
  statement above it. The §10 lint regex enforces the comment as a
  conservative gate; absence flags the line.

The current playable client violates this rule at:

- `client/src/ui/hud/mod.rs:2806` — `hud_top_strip_node()` (no AC).
- `client/src/ui/hud/mod.rs:2816` — `hud_bottom_strip_node()` (no AC).

These two sites are tracked as the §10 lint's **baseline allowlist** —
the lint records them as known violations and flags any *new*
`Overflow::visible()` site that lacks the AC comment. PROMPT 1196
(HUD top-strip wrap + opp class repair) clears the two HUD sites; the
allowlist shrinks accordingly when the lane lands.

### Where overflow protection is missing today

The audit enumerated these silent-overflow sites. Each is owned by a
Wave-2 lane in the PROMPT 1180 roadmap:

| Site | Symptom ID | Owning lane |
|------|------------|-------------|
| `client/src/ui/lobby.rs` modal panel | L-01 | E (PROMPT 1194) |
| `client/src/ui/hud/mod.rs` top-strip | H-01 | G (PROMPT 1196) |
| `client/src/ui/hud/mod.rs` bottom-strip | H-02 | G (PROMPT 1196) |
| `client/src/ui/hand/mod.rs` placement panel | F-03 | (rolls into Lane A) |
| `client/src/ui/shop_auction/mod.rs` bottom_panel / auction_panel / footer / toast | S-01..S-07 | A + H (PROMPT 1190 + 1197) |
| `client/src/ui/shop_auction/mod.rs` draft_initial modal | S-08 | J (PROMPT 1199) |
| `client/src/ui/settings/mod.rs` panel | O-01 | F (PROMPT 1195) |
| `client/src/ui/photosensitivity_warning.rs` panel | O-02 | J (PROMPT 1199) |
| `client/src/presentation/connection_lost_overlay.rs` panel | O-03 | J (PROMPT 1199) |

This contract does not pre-empt those lanes; it declares the rule each
must satisfy.

---

## §6 Button vs Status-Chip Visual Semantics (C-4)

The audit identified RC-4 — buttons styled as text, status chips
styled as buttons — as the second-largest systemic failure after
absolute-positioned bands. This section ratifies the binding visual
contract for the three classes.

### §6.1 Button

A clickable surface that triggers a `C2S*` message, a local
`UiCommand`, or a navigation transition. Examples: lobby Create /
Join / Confirm; auction Bid +K / Pass; shop Buy slot N / Refresh /
Ready; settings Back / Apply; result-screen Acknowledge.

**Required spawn components**:

- `bevy::ui::widget::Button` marker.
- `bevy::ui::Interaction::None` initialiser (the `Interaction` change
  detector is the canonical click-feedback signal — see PROMPT 1150
  `interaction_emits_click_test.rs` for the live regression).
- `bevy::ui::BackgroundColor(_)` — opaque, taken from the §7 palette of
  the parent design spec. Transparent or zero-alpha buttons are
  forbidden.
- `bevy::ui::BorderColor::all(_)` ≥ 1 px. The lint accepts any non-zero
  border declared on the same spawn block; per-side `BorderColor::new`
  variants count.
- Label `Text::new(_)` with a `TextFont` from the §5 typography
  hierarchy of the parent spec, weight `Bold` or `SemiBold`, size ≥
  `typography::BODY`.

**Recommended (not lint-enforced)**:

- Hover / pressed / disabled treatment from
  `client/src/ui/design_tokens/interaction_states.rs` (Sprint 15 story
  008). Per-surface migration owned by Sprint 18+ family
  `S18-UI-INTERACTION-STATE-MIGRATION-*` (PROMPT 1198 / Lane I).
- Cursor change on hover (deferred — Bevy 0.18 cursor API; out of
  scope here).

### §6.2 Status chip

A non-clickable surface that displays a transient *state* readout
(auction lead/loss state, hand-full banner, lobby status banner,
HUD timer countdown). Status chips are visually distinct from buttons
so the player does not mis-click on them.

**Required spawn components**:

- **NO** `Button` marker.
- **NO** `Interaction` component.
- `BackgroundColor` is `Color::NONE` OR is taken from the §7
  `SURFACE_ELEVATED` token of the parent spec at ≤ 50 % alpha.
- **NO** `BorderColor` (or `BorderColor::all(Color::NONE)`).
- Label `Text::new(_)` with `TextFont` from `typography::CAPTION` or
  `typography::BODY`, weight `Regular`.

### §6.3 HUD pill prefix label

A static prefix label inside a HUD pill container (`PHASE`, `ROUND`,
`GOLD`, `OPP`, `MANA`). Authored by PROMPT 1027 and currently
unique in the codebase (search: `HudPillPrefixLabel`).

**Required spawn components**:

- `HudPillPrefixLabel` marker.
- `Text::new(_)` with `hud_text_font(HUD_PILL_PREFIX_FONT_SIZE_PX)` and
  `TextColor(HUD_PILL_PREFIX_TEXT_COLOR)`.
- **NO** `BackgroundColor` (the pill container holds the background).
- **NO** `Button`; **NO** `Interaction`.

### §6.4 Anti-patterns flagged by the §10 lint

| Anti-pattern | Symptom in audit | Lint rule |
|--------------|------------------|-----------|
| `Button` spawn without `BackgroundColor` on the same component tuple | L-03 (lobby confirm CTA renders as `?` card) | `lint_buttons_have_background_color`. Allowlisted: documented helper-builder sites where `BackgroundColor` is supplied by a sibling `*_node()` builder function — these must be enumerated explicitly in the lint's `BUTTON_BG_ALLOWLIST` set. |
| `Button` spawn without `Interaction` on the same component tuple | Latent — click-feedback regression | `lint_buttons_have_interaction`. Allowlist same shape as above. |
| Non-button entity with `BackgroundColor` styled like a button (border + body-font label) | (Potential future regression; not yet observed broadly) | Out of scope for the initial lint — too brittle to detect statically. |

Lint output is one line per violation with file path, line number,
spawn-block excerpt, and the rule name. Baseline (allowlisted) sites
are reported as `BASELINE` not `VIOLATION`.

---

## §7 Panel Max-Height / Content-Budget Rules (C-5)

Every modal panel inside the centered-overlay pattern
(`docs/ux/global-ui-design-spec.md` §10) must declare:

1. `max_height: Val::Percent(92.0)` — already required by the parent
   spec §10. This contract restates it for completeness.
2. `overflow: Overflow::scroll_y()` — NEW invariant added by this
   contract. Lobby modal panel, settings panel, draft-initial modal
   panel, photosensitivity warning, connection-lost overlay panel all
   currently violate this rule; see §5 owning-lane table.
3. **Content budget**: at spawn time, the panel's children must sum
   their declared heights (where statically known) and the sum must
   not exceed `0.92 × floor_viewport_height = 0.92 × 720 = 662 px`.
   If the sum exceeds the budget, the spawn site must paginate, scroll,
   or wrap.

### Floor-viewport content budget table

| Surface | Floor (1280×720) max content height | Current observed | Status |
|---------|--------------------------------------|------------------|--------|
| Lobby modal panel | 662 px | exceeds when ≥2 rooms listed (L-01) | VIOLATING |
| Settings panel | 662 px | 520 px hardcoded → fits, but children clip the panel at 75 % UI-scale (O-01) | DIFFERENT VIOLATION |
| Draft-initial modal | 662 px | 360 px hardcoded (S-08) | UNDER BUDGET but not scaling |
| Photosensitivity warning | 662 px | survives in practice; brittle (O-02) | LATENT |
| Connection-lost overlay | 662 px | survives in practice; brittle (O-03) | LATENT |
| Result screen | 662 px | uses `max_height: 92%` correctly (O-04) | CONFORMANT |

The **result screen** (`client/src/presentation/result_screen.rs:502-549`)
is the **reference template** — Lane J (PROMPT 1199) explicitly
references it as the migration target for the violating surfaces.

### Inner-flex children sizing rule

Children inside a `scroll_y()` panel must use **`flex_shrink: 1.0`**
(the Bevy 0.18 default) on the column. They must not declare
`flex_shrink: 0.0` on dynamically-sized rows — that combination
disables the scroll surface's ability to clamp the column's *requested*
height to the panel's *available* height.

---

## §8 Text / Image Fitting Rules (C-6)

### §8.1 Text fitting

Every text label that hosts **dynamic** content (room code, status
banner, card title, stat readout, lobby button copy) must spawn its
parent with one of:

1. **`min_width: Val::Px(label_min_px)`** + **`overflow:
   Overflow::clip_x()`** — clip overflow without wrapping. Use when
   the surrounding layout is height-fixed and wrapping would break
   the visual rhythm (HUD pill values, lobby room-code chip).
2. **`max_width: Val::Px(label_max_px)`** + ellipsis policy via Bevy
   0.18 `TextLayout` — wrap or ellipsize the surplus. Use for free-form
   body copy (status banners, card descriptions).
3. **No constraint** — only when the surrounding flex parent's
   `width` is `Val::Percent(100.0)` and there are no siblings on the
   same row. Lobby section headers fall here.

The Bevy 0.18 `TextLayout` field set is the binding API. The
`liv-bevy-018` skill is the authority for the exact field names
between releases; this contract names the *requirement* (ellipsis-or-
clip), not the API call.

### §8.2 Image fitting

Every `ImageNode` that renders card art, class portraits, or any
non-decorative image **must** declare `image_mode:
NodeImageMode::Fit` (preserve aspect ratio, fit inside parent
rectangle) OR `NodeImageMode::Auto` (intrinsic size).

The Bevy 0.18 default `NodeImageMode::Stretch` is **forbidden** on
non-decorative images. The audit identified this as the root cause
of:

- F-02 — hand-fan card art renders horizontally stretched.
- L-02 — lobby class portraits never bind (a downstream side effect:
  even when the asset binds, the default Stretch ruins the visual).
- S-04 — auction featured card paints chrome under text without an
  opaque label strip.

**Decorative images** (sprite-sheet backgrounds, full-strip
ornamental fills) may use `Stretch` when the image is authored as a
stretch-tile asset. The lint does not attempt to distinguish these;
the responsibility lives in the per-surface story.

PROMPT 1192 (Lane C — card-art image-mode policy + label-strip
primitive) is the binding implementation story for this section.

---

## §9 Live Test Strategy (C-7)

Three test tiers replace the existing fixture-baseline harness
(audit RC-5). Each tier is authored by a separate Sprint 18 lane.

### §9.1 Tier 1 — Live-spawn viewport invariants

Authored by **PROMPT 1191** (Lane B). Spawns every UI plugin behind
its state gate (`LobbyUiPlugin`, `HudPlugin`, `HandUiPlugin`,
`ShopAuctionUiPlugin`, `PresentationPlugin`), forces a layout pass,
and queries `ComputedNode` against the camera viewport.

For each `(viewport in §3, RoundPhase × ClientState in §4)` cell,
asserts:

1. **No overlap** between any two `UI_BASE` roots (overlays and modals
   excluded per parent spec §3 z-layer ordering).
2. **No clipping** of any `UI_BASE` root's bounding rectangle past
   the viewport rectangle.
3. **Stable anchor** per the parent spec §8 "Per-class scaling rules"
   table — surfaces land at the same proportional position across
   viewports.
4. **Deterministic strip height** per parent spec §9 — strip
   primitives have identical pixel heights across every viewport.
5. **§4 CTA visibility predicate** — every primary CTA satisfies the
   on-screen bounds predicate.

Replaces `tests/integration/ui_viewport_invariants_test.rs` + the
fixture file. The fixture file is **deprecated** by Lane B's landing;
this contract does not delete it because the lint test references its
shape for cross-checking.

### §9.2 Tier 2 — Cross-surface integration smoke

Authored by **PROMPT 1191** Phase 2 (same lane, second binary).
Drives a scripted session through Lobby → DraftInitial → DraftShop →
DraftAuction → Placement → Resolution → GameOver and snapshots
`ComputedNode` bounds per phase. Catches the RC-1 same-z-layer
collisions in real protocol-driven state machines.

### §9.3 Tier 3 — Snapshot-diff CI

Authored by a separate Sprint 19+ lane (deferred — out of scope
for the PROMPT 1180 roadmap). Captures `screenshot.png` at each
phase × viewport into `production/qa/evidence/ui-clean-pass/auto/`
and pixel-diffs against baseline goldens. Tolerance band 2 %;
flag-only on first 3 sprint runs.

### §9.4 Tier 0 (this prompt) — Lint

This prompt (1188) authors a single lint test
`tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs`. The
lint is **conservative** (false-positive-avoidance) and ships with
an explicit baseline allowlist for known-violating sites so the
current branch is not blocked. It is the Tier-0 catch-net under
Tiers 1-3.

---

## §10 Lint Test Authored by This Prompt

The companion test
`tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs`
implements four lint rules over the production source files in
`client/src/ui/**` and `client/src/presentation/**`:

| Rule | Check | Baseline behavior |
|------|-------|--------------------|
| **L1 — Buttons need background** | Spawn tuple containing the literal `Button,` must also contain `BackgroundColor(` on the same tuple OR call a `*_node()` builder helper listed in the `BUTTON_BG_BUILDER_ALLOWLIST` set. | Records existing violations as `BASELINE`. Fails on any *new* file containing a `Button` spawn that satisfies neither. |
| **L2 — Buttons need Interaction** | Spawn tuple containing `Button,` must also contain `Interaction::` on the same tuple. | Same baseline behavior. |
| **L3 — `Overflow::visible()` justification** | Every occurrence of `Overflow::visible()` in `client/src/ui/**` or `client/src/presentation/**` must sit on a statement or directly under a comment matching `// AC: ` (case-sensitive). The strip primitive at `client/src/ui/design_tokens/strips.rs` is excluded by file path (strip primitives are the only legitimate use per §5). | The audit identified `hud/mod.rs:2806` and `hud/mod.rs:2816` as baseline-allowlisted. Records both as `BASELINE`. Fails on any *new* site without the AC comment. |
| **L4 — Status-chip-styled-as-button (advisory)** | Reports — does not fail — when a non-`Button` spawn tuple carries `BackgroundColor` + `BorderColor` + `Text::new`. Advisory only; intentionally non-failing because the false-positive surface is too large. | Always `BASELINE`; never fails. The rule exists to surface candidates for the Sprint 18+ interaction-state migration. |

**Rule scope**: file-by-file, top-level statement scan. The lint does
not do AST parsing — it does line-windowed text scans over a single
spawn statement (between `commands.spawn((` / `.spawn((` and the
matching `));` on a balanced-paren cursor over a single logical
statement). The line-window approach matches the existing
`ui_clean_pass/strips_test.rs::ac3_*` grep-guard style.

**Baseline file**: the lint embeds the baseline allowlist as Rust
constants `BUTTON_NO_BG_BASELINE: &[(file, line, snippet)]` so the
allowlist is reviewable in code review. The allowlist is sorted and
deduped; the test panics if any entry duplicates another.

**Migration**: when a Wave-2 lane fixes a baseline-listed violation,
the lane removes the entry from the allowlist; the lint then enforces
the rule at that site going forward. If a lane removes the violation
but forgets to update the allowlist, the lint reports the entry as
`STALE` and fails — the allowlist may not contain references to
sites that no longer violate.

---

## §11 Adoption Matrix

| Lane (per PROMPT 1180 §6) | PROMPT ID | What it consumes from this contract |
|---|---|---|
| A — PlayArea container | 1190 | §5 overflow rules; §7 panel content budget |
| B — Live-spawn viewport harness | 1191 | §4 CTA predicate; §9.1 + §9.2 test tiers |
| C — Card-art + label-strip primitive | 1192 | §8.2 image-fitting rule |
| D — Snapshot field enrichment | 1193 | §4 CTA predicate (Q-02 bounds field); §5 overflow detection (Q-03) |
| E — Lobby panel overflow + portrait + confirm CTA | 1194 | §5 overflow rules; §6 button vs chip; §7 content budget; §8 image-fitting |
| F — Settings panel flex relayout | 1195 | §7 panel content budget; §3 viewport matrix (UI-scale invariance) |
| G — HUD top-strip wrap + opp class repair | 1196 | §5 `Overflow::visible()` allowlist clearance; §6 status-chip semantics |
| H — Shop / auction paint + bid label | 1197 | §6 button vs chip (bid buttons); §5 overflow |
| I — Interaction-state migration Wave 2 | 1198 | §6 hover / pressed / disabled (`interaction_states::*` consumer wiring) |
| J — Overlay panel overflow hardening | 1199 | §5 overflow; §7 content budget |
| **K — This contract + lint** | **1188** | Authors §3..§10 of this document and the lint test. |

---

## §12 Cross-References

- `docs/ux/global-ui-design-spec.md` — parent design spec (tokens).
- `docs/ux/ui-clean-pass-roadmap.md` — Sprint 14+ sequencing.
- `docs/ux/board-rendering-spec.md` — board-rendering authority.
- `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md` —
  source audit (§5 contract draft, §6 Lane K).
- `production/epics/ui-clean-pass/EPIC.md` — epic-level charter.
- `client/src/ui/design_tokens/*.rs` — token implementation modules
  referenced by §6 / §7 / §8.
- `tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs` —
  the lint test authored alongside this contract.
- `tests/integration/ui_clean_pass/strips_test.rs` — reference for
  the grep-guard style the lint mirrors.
- `tests/integration/ui_viewport_invariants_test.rs` — fixture-baseline
  harness deprecated by Lane B (PROMPT 1191).

---

## §13 Authoring Trail

| Field | Value |
|-------|-------|
| **Authoring prompt** | PROMPT 1188 (`S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001`) |
| **Worker branch** | `work/s18-ui-layout-contract-doc-lint-1188` |
| **Source-of-truth at authoring** | `origin/main@efb698e` (PROMPT 1173 sidecar UTF-8 BOM fix) |
| **Audit consumed** | PROMPT 1180 sections 5 + 6 Lane K |
| **Parent spec consumed** | `docs/ux/global-ui-design-spec.md` (sections 3, 4, 5, 6, 7, 8, 9, 10, 11, 12) |
| **Files authored by PROMPT 1188** | `docs/ux/global-ui-layout-contract.md` (NEW), `tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs` (NEW), `client/Cargo.toml` (NEW `[[test]]` row only) |
| **Files explicitly NOT changed by PROMPT 1188** | `client/src/**`, `server/**`, `shared/**`, `production/sprint-status.yaml`, `production/sprints/*.md`, `production/stage.txt`, `production/session-state/**`, QA evidence / smoke / gate-check / release artifacts, launcher / tooling |
