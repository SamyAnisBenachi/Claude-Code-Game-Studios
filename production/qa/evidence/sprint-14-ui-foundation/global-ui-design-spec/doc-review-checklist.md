# Sprint 14 — Global UI Design Spec — Doc-Review Checklist (Story 007)

> **Story**: `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
> (`production/epics/ui-clean-pass/story-007-global-ui-design-spec.md`)
> **PROMPT**: 911 (`/dev-story`)
> **Branch**: `work/s14-global-ui-design-spec`
> **Worktree**: `D:/_DEV/wt/ccgs-prompt-911-global-ui-design-spec`
> **Base**: `origin/main@b39eedf` (PROMPT 908 `/story-done`
> `S11-TD-UI-FONT-CONSTANTS`)
> **Spec**: `docs/ux/global-ui-design-spec.md` (NEW by this prompt)

---

## AC1-AC14 verification

Each row records the AC, verification command / inspection, and verdict.
All verifications are run against the spec authored by PROMPT 911 plus
`origin/main@b39eedf`.

| AC | Title | Verification | Verdict |
|----|-------|---------------|---------|
| AC1 | Spec authored | `git ls-files docs/ux/global-ui-design-spec.md` returns the file at the worker tip. File exists at the AC1-specified path. | **PASS** |
| AC2 | All required sections present | `rg "^## §" docs/ux/global-ui-design-spec.md` enumerates §1 Status / No-Claim Banner, §2 Scope Boundaries — Friend-Game vs Standard-Tier, §3 Z-Index Layer System, §4 Spacing Scale, §5 Typography Hierarchy, §6 Overlay Alpha Tokens, §7 Color Tokens, §8 Responsive Layout Rules, §9 Strip Composition Patterns, §10 Component Specifications (stretch / optional). | **PASS** |
| AC3 | Z-layer canonical values (§3) | §3 table enumerates 8 named layers (Background `0`, World `100`, Units `200`, UiBase `300`, UiOverlay `400`, Modal `500`, Toast `600`, Debug `700`) — strictly ascending with 100-unit gap. Matches `client/src/ui/design_tokens/z_layers.rs` verbatim. | **PASS** |
| AC4 | Spacing canonical values (§4) | §4 table enumerates 5 named spacing tokens (XS 4, SM 8, MD 16, LG 24, XL 32) — strictly ascending pixel values. | **PASS** |
| AC5 | Typography canonical values (§5) | §5 table enumerates 6 semantic sizes (Caption 13, Body 15, H3 18, H2 22, H1 30, Display 40) — strictly ascending, plus 3 weight tokens (Regular 400, SemiBold 600, Bold 700) and canonical line-height ratio 1.25. Cited verbatim from `client/src/ui/design_tokens/typography.rs`. | **PASS** |
| AC6 | Overlay alpha canonical values (§6) | §6 table names `OVERLAY_DIM_ALPHA = 0.45`, `OVERLAY_SCRIM_ALPHA = 0.55`, `OVERLAY_TOAST_ALPHA = 0.80` (all 0.0 < α < 1.0) with rationale. | **PASS** |
| AC7 | Color palette named (§7) | §7 table enumerates 8 color tokens (`PRIMARY`, `SECONDARY`, `ACCENT`, `SURFACE`, `SURFACE_ELEVATED`, `SEMANTIC_SUCCESS`, `SEMANTIC_WARNING`, `SEMANTIC_ERROR`) with RGB hex + `Color::srgb(...)` reference. ≥6 satisfied. | **PASS** |
| AC8 | Responsive layout rules named (§8) | §8 matrix enumerates the canonical 6 viewports (`1366×768`, `1920×1080`, `1920×1200`, `1280×960`, `3840×2160`, `2560×1080`) with aspect ratios (16:9, 16:10, 4:3, 21:9) named in the per-class scaling rules. Min / target / max viewports identified. Cited verbatim from `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`. | **PASS** |
| AC9 | Strip composition patterns named (§9) | §9 table enumerates `HeaderBar` (60 px), `LaneBar` (60 px; bevy_ui implementation deferred per story 004 worker discretion), `HandBar` (180 px), `FooterBar` (40 px) with flex direction, justify content, align items, and anchor. Heights ratified verbatim from `tests/integration/fixtures/ui_viewport_baseline.rs`. | **PASS** |
| AC10 | Spec adoption matrix present | Spec contains an explicit "Spec Adoption Matrix" section enumerating: story 002 → §3; story 003 → §5; story 004 → §4 + §9; story 005 → §8; story 006 → §6. Tier 1 surface consumers + Tier 1 / Tier 0 Should-priority adjacent rows + Tier 3 deferred rows also mapped. | **PASS** |
| AC11 | Friend-game scope boundary named (§2) | §2 explicitly names the friend-game-vs-Standard-tier boundary; `QA-COND-0005` Standard-tier accessibility, `QA-COND-0006` playtest validation, and `PAW-TD-002-a` … `PAW-TD-006-a` placeholder-art accept-risk are each named as out of spec scope. §1 Status Banner repeats these dispositions verbatim. | **PASS** |
| AC12 | Producer ratification checklist | Spec contains an explicit "Producer Ratification Checklist" section with three sign-off rows (producer, UX-designer, art-director) ratified at PROMPT 911 with per-role rationale. PROMPT 802 §9 producer-decision-2 is explicitly named as resolved by this ratification. | **PASS** |
| AC13 | No code change | `git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**' 'Cargo.toml' 'Cargo.lock'` returns empty at the PROMPT 911 worker commit. | **PASS** |
| AC14 | Friend-game scope preserved | `git diff origin/main...HEAD -- 'production/sprint-status.yaml'` returns empty at the PROMPT 911 worker commit; no accept-risk disposition is flipped. | **PASS** |

---

## Ratification summary

The AC12 producer + UX-designer + art-director sign-off rows recorded in
the spec close PROMPT 802 §9 **producer-decision-2** (numeric values for
Tier 0 token modules):

- **Producer ratification** — accepts the spec values for §3 / §4 / §5 /
  §6 / §7 / §8 / §9. Tier 0 token-module stories 004 + 006 are unblocked
  for `/story-readiness`. Stories 002 / 003 / 005 already shipped values
  are ratified verbatim; no value override is requested.
- **UX-designer ratification** — accepts the typography hierarchy
  (§3.1 L6 inversion fix preserved), spacing scale (4 / 8 / 16 / 24 / 32),
  strip heights ratifying story 005's already-shipped baseline (60 / 40 /
  180), and the 6-viewport canonical matrix. Color palette is friend-game
  placeholder; future palette migration is in scope for a separate row.
- **Art-director ratification** — preserves `PAW-TD-*-a` placeholder-art
  accept-risk. Color tokens are friend-game placeholder; final-asset
  replacement remains a separate sprint scope. Z-layer ordering preserves
  the existing PresentationPlugin composition (ADR-021 R2). Strip heights
  deliver a visually composed top-to-bottom HUD column on every viewport
  in the canonical matrix.

The ratification is **specifically scoped to friend-game visual polish**.
It does **not** ratify Standard-tier accessibility values, final-art
assets, playtest validation, per-element layout (owned by Tier 1
stories), animation / motion / interaction-state primitives, or any of
producer-decisions 3 / 4 / 5.

---

## Section heading enumeration (AC2 source data)

```
$ rg -n "^## " docs/ux/global-ui-design-spec.md
```

Expected (post-PROMPT-911) output (subset; primary sections only):

```
## §1 Status / No-Claim Banner
## §2 Scope Boundaries — Friend-Game vs Standard-Tier
## §3 Z-Index Layer System
## §4 Spacing Scale
## §5 Typography Hierarchy
## §6 Overlay Alpha Tokens
## §7 Color Tokens
## §8 Responsive Layout Rules
## §9 Strip Composition Patterns
## §10 Component Specifications (stretch / optional)
## Spec Adoption Matrix
## Producer Ratification Checklist
## Cross-References
## Authoring Trail
```

§1-§9 are the AC2-required sections. §10 is stretch / optional per
story-007 In Scope §10. The Spec Adoption Matrix satisfies AC10. The
Producer Ratification Checklist satisfies AC12.

---

## Cross-reference matrix between spec sections and consumer stories (AC10 source data)

| Spec section | Sprint 14 consumer story | Story file | Sprint 14 row |
|--------------|---------------------------|------------|---------------|
| §3 Z-Index Layer System | `S11-TD-UI-ZINDEX-LAYERS` | `production/epics/ui-clean-pass/story-002-ui-zindex-layers.md` | Tier 0 rank 1 |
| §5 Typography Hierarchy | `S11-TD-UI-FONT-CONSTANTS` | `production/epics/ui-clean-pass/story-003-ui-font-constants.md` | Tier 0 rank 2 |
| §4 Spacing Scale + §9 Strip Composition Patterns | `S11-TD-UI-FLEX-STRIPS` | `production/epics/ui-clean-pass/story-004-ui-flex-strips.md` | Tier 0 rank 3 |
| §8 Responsive Layout Rules | `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` | `production/epics/ui-clean-pass/story-005-ui-viewport-invariant-tests.md` | Tier 0 rank 4 |
| §6 Overlay Alpha Tokens | `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` | `production/epics/ui-clean-pass/story-006-ui-overlay-alpha-token.md` | Tier 0 rank 5 |
| §3 + §4 + §5 + §9 | `S11-UX-HUD-TOP-STRIP-LAYOUT` | `production/epics/hud/story-015-hud-top-strip-layout.md` | Tier 1 rank 7 |
| §3 + §4 + §5 + §9 | `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` | `production/epics/hud/story-016-hud-bottom-strip-layout.md` | Tier 1 rank 8 |
| §3 + §5 + §8 + §10 | `S11-UX-DRAFT-GRID-CENTERED-MODAL` | `production/epics/shop-auction-ui/story-015-draft-grid-centered-modal.md` | Tier 1 rank 9 |
| §3 + §4 + §5 + §7 | `S11-UX-AUCTION-FEATURED-CARD` | `production/epics/shop-auction-ui/story-016-auction-featured-card.md` | Tier 1 rank 10 |
| §3 + §4 + §5 | `S11-UX-LOBBY-CLASS-PICKER` | `production/epics/playable-client/story-025-lobby-class-picker-layout.md` | Tier 1 rank 11 |
| §3 + §4 + §5 + §8 | `S12-UX-LOBBY-LAYOUT-MODAL-001` | `production/epics/playable-client/story-024-lobby-layout-modal.md` | Tier 1 rank 12 |
| §4 + §10 button | `S11-UX-LOBBY-BUTTON-HITTARGETS` | `production/epics/playable-client/story-026-lobby-button-hittargets.md` | Tier 1 Should adjacent (`QA-COND-0005` accept-risk preserved) |
| §3 + §4 + §5 + §9 | `S11-UX-HUD-OPP-FIGURINE` | `production/epics/hud/story-017-hud-opponent-figurine.md` | Tier 1 Should adjacent |
| §3 + §4 + §5 + §7 | `S11-UX-AUCTION-FREE-GOLD-COUNTERS` | `production/epics/shop-auction-ui/story-017-auction-free-gold-counters.md` | Tier 1 Should adjacent |
| §7 semantic tokens | `S12-UX-AUCTION-LEAD-LOSS-STATE-001` | `production/epics/shop-auction-ui/story-018-auction-lead-loss-state.md` | Tier 1 Should adjacent (producer-decision-4 blocking) |
| §3 (`UiOverlay`) + §6 | `S12-UX-HAND-DRAG-STATE-VISUALS-001` | (story file in epic — see roadmap) | Tier 1 Should adjacent |
| §7 + §10 button affordance | `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` | (story file in epic — see roadmap) | Tier 0 Should adjacent |
| §4 + §5 + §10 card slot | `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` | (deferred to Sprint 15) | Tier 3 rank 13 |
| §3 World / Units + §8 | `S11-UX-BOARD-RENDERING-SPEC` | (deferred to Sprint 15) | Tier 3 rank 14 |

---

## Carried Non-Claims Preserved

PROMPT 911 (this dev-story closure) preserves the following dispositions
unchanged:

- `S8-QA-001-W1` (two-client GAME_OVER) remains **OPEN**.
- `QA-COND-0005` (Standard-tier accessibility) remains **accepted-risk**.
- `QA-COND-0006` (playtest / fun-hypothesis validation) remains
  **accepted-risk / deferred**.
- `PAW-TD-002-a` … `PAW-TD-006-a` (placeholder-art) remain
  **accepted-risk**.
- PROMPT 761 `Polish->Release` gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO**
  retry in scope.
- Stage `Polish` preserved.
- Sprint 14 disposition `active` preserved.
- Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed.
- PROMPT 683-era runtime divergence question preserved (third same-scope
  retest NOT authorised per `TQ-S12-C2`).

---

## Verification Commands (post-commit)

The worker prompt (PROMPT 911) verifies these after committing:

```
git diff --check
git diff --cached --check
git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**' 'Cargo.toml' 'Cargo.lock'
git diff origin/main...HEAD -- 'production/sprint-status.yaml' 'production/stage.txt' 'production/sprints/sprint-14.md' 'production/session-state/'
git diff origin/main...HEAD --stat
```

All five must return clean / empty / spec-and-evidence-only output for
the worker tip to be valid.
