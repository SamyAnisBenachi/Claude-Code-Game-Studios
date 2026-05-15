# Story 016: S11-UX-HUD-BOTTOM-STRIP-LAYOUT -- HUD Bottom Strip Layout (Composition Only)

> **Epic**: HUD
> **Story ID**: S11-UX-HUD-BOTTOM-STRIP-LAYOUT
> **Status**: Draft -- Sprint 14 candidate (Tier 1 Must per
> `docs/ux/ui-clean-pass-roadmap.md` rank 8 / PROMPT 802 §3.2 H1 / H9);
> NOT activated; Sprint 14 NOT activated
> **Layer**: HUD / Presentation (layout / composition only)
> **Type**: UI -- layout composition + visual evidence
> **Sprint**: Sprint 14 candidate (drawn from PROMPT 802 Expert UI Layout
> audit roadmap, reconciled by `docs/ux/ui-clean-pass-roadmap.md` rank 8
> "Tier 1 Must"); NOT activated
> **Authored**: 2026-05-14 by PROMPT 879 (worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-hud-layout-story-authoring`,
> branch `story/s14-hud-layout-story-authoring`)
> **Authoring source-of-truth**: `origin/main@dd9630b` (PROMPT 877
> `integrate(s13): merge work/s13-r2-placement-crash-audit (server story 002 / PROMPT 874)`;
> session-start HEAD was `origin/main@51e6228` PROMPT 871 — worktree
> fast-forwarded to `dd9630b` during authoring to keep source-of-truth current)

---

## Status / No-Claim Banner

This story is authored as a **Sprint 14 candidate**. Sprint 14 is
**NOT** activated by PROMPT 879. Sprint 13 remains `active` and is
not changed by this authoring run. Sprint 12 remains
`closed-with-conditions` per PROMPT 817 and is not changed.

PROMPT 879 (this authoring run) does **NOT**:

- Activate Sprint 14.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion (`PAW-TD-*-a`)
- HUD final-art replacement on the bottom strip (placeholder figurine
  PNG and surrounding chrome preserved under `PAW-TD-004-a`
  accept-risk)
- closure of any other Sprint 14 candidate row from `docs/ux/ui-clean-pass-roadmap.md`

Sprint 10 / Sprint 11 / Sprint 12 / Sprint 13 dispositions unchanged.
PROMPT 761 Polish->Release gate-check FAIL evidence preserved. PROMPT
802 audit roadmap accept-risk boundaries (`PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`) preserved verbatim.

**No optimistic client-side authority is introduced or proposed.**
The HUD bottom strip is read-only over server-authoritative state
(`Res<CurrentClientPhase>`, `ManaDisplayState`, own `ClassId` via
`S2CGameSnapshot`) per ADR-021 + ADR-002. This story changes how the
bottom-strip region (own player figurine + current mana bar + reserve
mana diamond + any future bottom-anchored readouts) is **composed
visually on screen** — not what is rendered or where the data comes
from.

---

## Source Finding

- PROMPT 802 audit `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`
  §3.2 (HUD) lists defects H1, H2, H9 against the current HUD bottom
  strip in `client/src/ui/hud/mod.rs`:
  - **H1**: every HUD child uses `PositionType::Absolute` with hard-coded
    `Val::Px(margin + N)` offsets relative to the corner. No flex
    bottom-strip composition.
  - **H2**: magic offsets — figurine at `bottom: hud_margin + 60.0`,
    current mana bar / reserve mana diamond at independent
    `bottom: Val::Px(...)` per element. No shared bottom-anchored
    spacing-scale.
  - **H9**: bottom-strip layout (figurine area) is a single sprite +
    magic offset, not composed. Already-tracked PROMPT 685 row
    `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`.
- PROMPT 685 row 2 (audit row "HUD strip slice — bottom") is
  `subsumed-by` `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` per
  `docs/ux/ui-clean-pass-roadmap.md` reconciliation matrix.
- `docs/ux/ui-clean-pass-roadmap.md` rank 8 places this slug as Tier 1
  Must (0.5d) with Phase 1 dependency on ranks 1 (`S11-TD-UI-ZINDEX-LAYERS`),
  3 (`S11-TD-UI-FLEX-STRIPS`), 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`).

---

## Problem Class / Prevention Target

**Defect class**: HUD bottom strip composition uses absolute-positioned
children with magic per-element bottom offsets. The own-player class
figurine sits at `bottom: Val::Px(config.hud_margin_px + 60.0)`; the
current mana bar and reserve mana diamond each declare their own
`bottom: Val::Px(...)` independently, computed from `hud_margin_px`
plus an ad-hoc constant. This:

1. Cannot self-adapt when an upstream constant changes (move the
   figurine up 8 px → mana bar must be re-offset by hand to avoid
   overlap).
2. Cannot self-adapt when the figurine asset's pixel dimensions
   change (today: 64×64; if PAW-004 final-art replacement lands at
   80×80 the current offset would push mana bar into figurine pixel
   space — even though art replacement is itself out of scope, the
   refactor must be robust to it).
3. Cannot guarantee stable rendered dimensions across viewports —
   1366×768 vs 1920×1080 at the same pixel offsets renders identically
   in absolute space, but the strip can extend past the visible
   bottom edge at narrower aspect ratios.
4. Cannot enforce font sizing that is independent of viewport width.
   Anti-regression below forbids viewport-width font scaling on the
   reserve mana label.

**Prevention target**: introduce a single HUD bottom-strip flex
parent (name: `HudBottomStrip`) anchored to the bottom-left of the
viewport, hosting the own-player figurine, current mana bar, and
reserve mana diamond as flex children. Use the foundational primitives
delivered by `S11-TD-UI-FLEX-STRIPS` (flex direction, gap, padding
tokens) and `S11-TD-UI-ZINDEX-LAYERS` (HUD z layer slot). Numeric
inputs (spacing, container widths) come from
`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`.

The visual layout intent (which element goes where, anchor to
bottom-left) is **preserved unchanged from the current code** under
this story — composition refactor, not a redesign. Opponent figurine
composition is **explicitly out of scope** here (separate Sprint 14
candidate `S11-UX-HUD-OPP-FIGURINE` per PROMPT 802 §3.2 H10).

---

## Context

### Existing surface

- **`client/src/ui/hud/mod.rs`** (per ADR-021): HUD spawn function
  `spawn_hud` (lines 482-661); bottom-strip region currently composed
  via independent absolute-positioned children of `HudRoot`:
  - Class figurine at lines 567-586: `bottom: Val::Px(config.hud_margin_px + 60.0)`,
    `width: Val::Px(64.0)`, `height: Val::Px(64.0)`.
  - Current mana bar: `spawn_mana_label(...)` with `current_mana_bar_node(config.hud_margin_px)`
    — independent absolute placement.
  - Reserve mana diamond: `spawn_reserve_mana_label(...)` — independent
    absolute placement.
  - Pixel constants: `CURRENT_MANA_BAR_WIDTH_PX = 104.0`,
    `CURRENT_MANA_BAR_HEIGHT_PX = 28.0`,
    `RESERVE_MANA_DIAMOND_SIZE_PX = 74.0`,
    `RESERVE_MANA_DIAMOND_ROTATION_DEGREES = 45.0`.
- **`design/gdd/hud.md`** TR-HUD-001 / TR-HUD-002 / TR-HUD-005:
  describes the gold / mana / objective-identity render rules. Does
  NOT prescribe absolute vs flex composition.
- **`docs/ux/ui-clean-pass-roadmap.md`** sequencing rules and Phase 1
  dependency list (rank 8).
- **`reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`**
  §3.2 H1 / H2 / H9.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hud.md` TR-HUD-002 (mana display), TR-HUD-011
  (current mana bar / reserve mana diamond shape distinction — covered
  by Sprint 6 story 011, already `Ready`). All preserved by this
  story.
- **ADR-021** (Presentation Layer Architecture): `HudPlugin` 4th in
  `PresentationPlugin`; `PresentationSet` ordering preserved. This
  story does not change the system schedule, only spawn-time Node
  composition.
- **ADR-002** (Client-Server Authority): HUD remains read-only.
- **ADR-001** (Objective Identity Unicast): the bottom strip does
  NOT carry objective state; scoreboard dots are not on the bottom
  strip. Defence-in-depth check is part of acceptance.
- **TR registry**: no new TR (composition refactor of existing TRs).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits under
  `client/src/ui/hud/`. The story implementation prompt MUST activate
  this skill before editing. Bevy 0.18 `Node` composition uses the
  Required Components API; flex children must be declared with
  explicit `Display::Flex`, `flex_direction`, `align_items`,
  `justify_content`, `column_gap` / `row_gap`. The reserve-mana
  diamond uses a 45-degree rotation transform; the implementation
  prompt must preserve `RESERVE_MANA_DIAMOND_ROTATION_DEGREES` and
  confirm it composes inside a flex parent without breaking hit-region
  geometry.
- **Lightyear**: no Lightyear changes; `liv-bevy-lightyear` not
  required.

### Control Manifest Rules

- Required: HUD bottom strip composition uses a single flex parent
  (`HudBottomStrip` marker component) with explicit `Display::Flex`.
- Required: Foundational primitives from `S11-TD-UI-FLEX-STRIPS`,
  `S11-TD-UI-ZINDEX-LAYERS`, and `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  are consumed (not re-implemented inline).
- Required: All existing bottom-strip pre-pooled entity identities
  (`figurine`, `mana_label`, `reserve_container`, `reserve_label`)
  remain reachable via `HudEntities` — child reparenting under the
  new flex parent preserves `Entity` IDs.
- Required: ADR-021 system schedule preserved (no `PresentationSet`
  reordering, no new schedule sets added under this story).
- Required: ADR-002 + ADR-021 preserved (HUD remains read-only).
- Required: TR-HUD-011 shape distinction (current mana = bar / reserve
  mana = diamond) preserved exactly. `ManaShapeGeometry` component
  unchanged. Rotation transform preserved on reserve diamond.
- Forbidden: Introducing `Val::Percent(...)` on any `font_size` /
  text-size field of a bottom-strip child. Font sizing remains fixed
  pixel by spec.
- Forbidden: Per-element `PositionType::Absolute` on bottom-strip
  children after the refactor lands. The `HudRoot` itself may remain
  absolute to anchor the viewport; the **bottom-strip flex child
  sub-tree** must use flex composition.
- Forbidden: Modifying any code outside `client/src/ui/hud/` in
  service of this story.
- Forbidden: Final-art / asset replacement on HUD bottom-strip
  elements (`PAW-TD-004-a` accept-risk preserved). The figurine
  remains the current placeholder/PAW-004 asset.
- Forbidden: Standard-tier accessibility hit-target ≥44px work on
  bottom-strip elements (`QA-COND-0005` preserved). Mana bar and
  reserve mana diamond are read-only HUD indicators and have no
  hit-target requirement; this AC restates that boundary.
- Forbidden: Opponent figurine composition (separate story
  `S11-UX-HUD-OPP-FIGURINE`).
- Forbidden: Modifying server protocol, ECS message routing, or any
  presentation system that consumes the HUD entities downstream.

---

## Story Classification

**Story type**: UI -- layout composition refactor + visual evidence.

This is **NOT** a:

- Logic story (no formula or state machine change).
- Integration story (no new system-set or schedule wiring).
- Final-art story (placeholder PAW-004 figurine preserved).
- Accessibility story (`QA-COND-0005` preserved).
- Animation story (no new tween, no new `bevy_tweening` `Animator`).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type",
UI stories deliver a **manual walkthrough doc OR interaction test**
with screenshot evidence as ADVISORY gate.

---

## Dependencies (must be Done before /dev-story on this story)

Per `docs/ux/ui-clean-pass-roadmap.md` rank 8 "Phase 1 dependency"
(ranks 1, 3, 6):

| Dependency | Slug | Why blocking |
|---|---|---|
| Z-index layers | `S11-TD-UI-ZINDEX-LAYERS` (rank 1, Tier 0 Must) | HUD bottom strip needs explicit z layer assignment so the figurine + mana bar do not occlude the bottom-anchored RESOLUTION dim overlay incorrectly when respawned (PROMPT 802 §3.9 G1). |
| Flex strip primitives | `S11-TD-UI-FLEX-STRIPS` (rank 3, Tier 0 Must) | Provides shared flex tokens; bottom-strip layout consumes the same primitive set as the top strip. |
| Global UI design spec | `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, Tier 0 Must) | Provides numeric inputs for figurine-to-mana-bar gap, bottom anchor padding, and any per-element padding. |

**Sequencing relative to sibling Sprint 14 candidates**:

- `S11-UX-HUD-TOP-STRIP-LAYOUT` (story 015 in this epic): parallel-safe
  (different region of `client/src/ui/hud/mod.rs`); both refactors edit
  the same spawn function, so workers MUST coordinate / serialize
  edits on `client/src/ui/hud/mod.rs` per Sprint 14 activation
  guidance.
- `S11-UX-HUD-OPP-FIGURINE` (story 017 in this epic): sequenced AFTER
  this story per `docs/ux/ui-clean-pass-roadmap.md` "Tier 1
  Should-Priority Adjacent Rows" table (it pairs with rank 8). The
  opponent-figurine composition will live inside (or adjacent to) the
  `HudBottomStrip` parent introduced here; authoring `S11-UX-HUD-OPP-FIGURINE`
  before this story landed would force a re-write.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Single flex parent introduced**: GIVEN the HUD root
  spawn, WHEN `spawn_hud` runs, THEN a single new child entity carrying
  a `HudBottomStrip` marker and `Display::Flex` is spawned as a child
  of `HudRoot`. The own-player figurine, current mana bar, and
  reserve mana container are reparented under `HudBottomStrip` (not
  direct children of `HudRoot`).

- [ ] **AC2 -- Flex composition replaces absolute offsets on
  bottom-strip children**: GIVEN the post-refactor spawn, WHEN each
  bottom-strip child `Node` is inspected, THEN none of them carries
  `PositionType::Absolute` with `bottom: Val::Px(hud_margin + N)` or
  `left: Val::Px(hud_margin + N)` style absolute offsets. Children
  rely on `HudBottomStrip`'s `flex_direction` / gap / padding for
  their position. The `HudRoot` may keep its viewport-spanning
  absolute Node; **bottom-strip children** change.

- [ ] **AC3 -- Entity identity preserved in `HudEntities`**:
  GIVEN the post-refactor `HudEntities` resource, WHEN inspected,
  THEN `figurine`, `mana_label`, `reserve_container`, `reserve_label`
  still point at the same logical entities they did pre-refactor. A
  new `bottom_strip` field is added pointing at the new parent.

- [ ] **AC4 -- TR-HUD-011 shape distinction preserved**: GIVEN the
  post-refactor spawn, WHEN current mana and reserve mana entities
  are inspected, THEN the current mana entity retains
  `ManaShapeGeometry { kind: Bar, width_px: CURRENT_MANA_BAR_WIDTH_PX,
  height_px: CURRENT_MANA_BAR_HEIGHT_PX }` and the reserve mana
  entity retains its diamond geometry with
  `RESERVE_MANA_DIAMOND_SIZE_PX` size and
  `RESERVE_MANA_DIAMOND_ROTATION_DEGREES` rotation. Shape distinction
  is not collapsed by the flex refactor.

- [ ] **AC5 -- ADR-021 schedule preserved**: GIVEN a `cargo build -p
  client` (or equivalent), WHEN run against the post-refactor code,
  THEN no new system, system-set, or schedule wiring is introduced.
  `HudPlugin` registers the same sets in the same order.

- [ ] **AC6 -- Visual evidence captured at two viewports**: GIVEN
  the post-refactor build runs end-to-end through the friend-game
  route, WHEN HUD is visible (any non-`Hidden` phase), THEN
  screenshots are captured at **desktop** (1920×1080) AND at a
  **smaller viewport** (1366×768 minimum). Captures land under
  `production/qa/evidence/sprint-14-hud-bottom-strip-layout/`
  (NEW) with filenames `bottom-strip-1920x1080-<phase>.png` and
  `bottom-strip-1366x768-<phase>.png` for at least one phase that
  lights the reserve-mana diamond (ECONOMY_AUCTION phase with
  `reserve_mana > 0`).

- [ ] **AC7 -- Text fitting anti-regression**: GIVEN the captures,
  WHEN visually inspected against the longest expected mana content
  (current `99 / 99`, reserve `99`), THEN no text is clipped or
  truncated by its container. The evidence document records the
  longest content observed and confirms no clipping.

- [ ] **AC8 -- Stable dimensions anti-regression**: GIVEN the captures,
  WHEN dimensions of each bottom-strip child are measured, THEN each
  child's rendered width and height is the same at 1920×1080 as at
  1366×768. Specifically: figurine renders at 64×64 (or its current
  pixel size constant), current mana bar at
  `CURRENT_MANA_BAR_WIDTH_PX × CURRENT_MANA_BAR_HEIGHT_PX`, reserve
  mana diamond at `RESERVE_MANA_DIAMOND_SIZE_PX × RESERVE_MANA_DIAMOND_SIZE_PX`,
  identical across viewports.

- [ ] **AC9 -- No overlap anti-regression**: GIVEN the captures at
  both viewports, WHEN siblings are inspected, THEN no bottom-strip
  child visually overlaps a sibling, the timer bar (top strip), or
  any non-bottom-strip element. Captures from any phase that lights
  the reserve-mana label (ECONOMY_AUCTION with `reserve_mana > 0`)
  are included.

- [ ] **AC10 -- No viewport-width font scaling anti-regression**:
  GIVEN a grep across `client/src/ui/hud/` post-refactor, WHEN run
  with pattern `Val::Percent`/`Val::Vw`/`Val::Vh` filtered to lines
  touching `TextFont` or `font_size`, THEN zero hits on bottom-strip
  children. Reserve mana label font sizing remains fixed pixel.

- [ ] **AC11 -- Z-index layer slot consumed (not re-invented)**: GIVEN
  the post-refactor spawn, WHEN the `HudBottomStrip` z positioning is
  inspected, THEN it consumes the HUD layer slot defined by
  `S11-TD-UI-ZINDEX-LAYERS` (e.g. `HudLayers::BottomStrip` enum
  variant or equivalent named constant) — NOT a hard-coded
  `GlobalZIndex(N)` re-introduced inline.

- [ ] **AC12 -- ADR-001 invariant preserved**: GIVEN the post-refactor
  build, WHEN any path that surfaces objective identity is inspected,
  THEN `was_fake` remains stripped at the Board Rendering boundary
  and is never exposed on a bottom-strip child. Scoreboard dots are
  not on the bottom strip; this AC is a defence-in-depth check.

- [ ] **AC13 -- Sprint 13/14 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`
  (when authored), `production/stage.txt`, and PROMPT 761 gate-check
  artifact are diffed, THEN none of them are modified by this story.

- [ ] **AC14 -- No accept-risk closure claimed**: GIVEN the evidence
  document, WHEN inspected, THEN it explicitly does NOT claim closure
  of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-004-a`,
  or any other accept-risk disposition. Final-art replacement on HUD
  bottom-strip elements is explicitly out of scope.

- [ ] **AC15 -- Targeted regression passes**: GIVEN the post-refactor
  code, WHEN `cargo test -p client --lib` is run, THEN it passes.
  Existing HUD tests (story 001 scaffold, story 002 gold/mana display,
  story 011 shape distinction) continue to pass because
  `HudEntities` field identities are preserved.

- [ ] **AC16 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-14-hud-bottom-strip-layout/README.md`
  (NEW). Records the build commit, the two viewport captures, the
  longest-content observation (AC7), the dimension measurements
  (AC8), no-claim restatement, and cross-links to PROMPT 802 §3.2
  H1 / H2 / H9 + `docs/ux/ui-clean-pass-roadmap.md` rank 8.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hud/mod.rs` | Refactor `spawn_hud` to introduce `HudBottomStrip` flex parent + reparent bottom-strip children. Add `bottom_strip: Entity` field on `HudEntities`. |
| `client/src/ui/hud/<existing-mana-submod-if-any>.rs` | If the mana-bar / reserve-diamond spawn lives in a submodule today, refactor here too. |
| `tests/integration/ui/hud_bottom_strip_test.rs` | NEW *iff* `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4) exposes the test bin scaffolding. Optional. |
| `production/qa/evidence/sprint-14-hud-bottom-strip-layout/README.md` | NEW evidence document. |
| `production/qa/evidence/sprint-14-hud-bottom-strip-layout/bottom-strip-1920x1080-draft-auction.png` | NEW screenshot capture (desktop). |
| `production/qa/evidence/sprint-14-hud-bottom-strip-layout/bottom-strip-1366x768-draft-auction.png` | NEW screenshot capture (smaller viewport). |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. Per PROMPT 879 framing,
`client/src/`, `server/src/`, `shared/src/`, `tests/`, and
`Cargo.toml` are NOT touched by the authoring prompt — only by a
future implementation prompt run after Sprint 14 activates.

---

## Required Skills

- `liv-bevy-018` (MANDATORY for the implementation prompt).
- `liv-bevy-lightyear`: NOT required (no Lightyear changes).

The authoring prompt (PROMPT 879) does NOT invoke either skill
because no code is touched at authoring time.

---

## Evidence Path

`production/qa/evidence/sprint-14-hud-bottom-strip-layout/README.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- Build commit hash and branch.
- Two screenshots minimum: 1920×1080 + 1366×768 at the same phase
  (recommend `DraftAuction` because it is the canonical ECONOMY_AUCTION
  phase where the reserve mana diamond lights).
- Longest-content observation table (per AC7).
- Per-child rendered dimension table (per AC8).
- Overlap audit (per AC9).
- Z-index layer slot citation (per AC11).
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to PROMPT 802 §3.2 H1 / H2 / H9.
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` rank 8.
- Cross-link to TR-HUD-011 (shape distinction).

---

## Regression Commands Expected

For the implementation prompt (NOT the authoring prompt):

- `cargo build -p client` (must succeed; AC5).
- `cargo test -p client --lib` (HUD-scoped tests; AC15).
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`
- Grep `Val::Percent|Val::Vw|Val::Vh` filtered to `client/src/ui/hud/`
  matches against `font_size` / `TextFont` (must be zero; AC10).

The authoring prompt (PROMPT 879) runs only `git diff --check`,
`git diff --cached --check`, `git status --short --branch`.

---

## Out of Scope

- HUD top strip composition — separate Sprint 14 candidate
  `S11-UX-HUD-TOP-STRIP-LAYOUT` (story 015).
- HUD opponent figurine composition — separate Sprint 14 candidate
  `S11-UX-HUD-OPP-FIGURINE` (story 017).
- Final-art replacement on HUD bottom-strip elements (`PAW-TD-004-a`
  preserved).
- Standard-tier accessibility on HUD bottom strip (`QA-COND-0005`
  preserved).
- Mana bar fill animation tween (already covered by existing HUD
  stories).
- Scoreboard dot composition (dots are not on the bottom strip).
- Cross-surface design-token authoring (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  is its own story).
- Z-index layer module authoring (`S11-TD-UI-ZINDEX-LAYERS` is its
  own story).
- Flex strip primitive authoring (`S11-TD-UI-FLEX-STRIPS` is its own
  story).
- Sprint 14 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under PROMPT 879
  (this authoring prompt).

---

## Dependency Notes Against Sprint 13 Active Scope

- Sprint 13 active scope (runtime hardening + Sprint 12 cleanup +
  UI-audit-roadmap-prep) does NOT include this story. Sprint 13
  remains unchanged.
- File-collision risk against Sprint 13 rows on `client/src/ui/hud/`:
  none known.
- File-collision risk against sibling Sprint 14 candidate story 015
  (HUD top strip): both stories edit `spawn_hud` in
  `client/src/ui/hud/mod.rs`. The implementation worker order is the
  responsibility of the Sprint 14 activation orchestrator — both can
  in principle author in parallel, but at `/dev-story` time the
  second worker must rebase on the first's commit and adjust the
  spawn-time tree accordingly.
- This story lands under the existing HUD epic; the HUD epic
  remains `Ready`.

---

## Sprint 14 Activation Preconditions (for the orchestrator that
activates Sprint 14)

Before this story enters `/dev-story` in Sprint 14:

1. Sprint 14 activation prompt MUST re-state the accept-risk
   preservations from `docs/ux/ui-clean-pass-roadmap.md` "Accept-Risk
   Dispositions Preserved" — `PAW-TD-004-a`, `QA-COND-0005`,
   `QA-COND-0006`.
2. Sprint 14 QA plan MUST exist and pass `/qa-plan sprint`.
3. The three Tier 0 dependencies (ranks 1, 3, 6) MUST be **Done**
   (not just Ready) before this story enters `/dev-story`.
4. `/story-readiness` MUST pass on this story file against the
   Sprint 14 activation HEAD.
5. Coordination decision: if `S11-UX-HUD-TOP-STRIP-LAYOUT` (story 015)
   is sequenced same-wave, the activation prompt MUST nominate a
   worker order (top first or bottom first) and the second worker
   MUST rebase on the first's commit.

If any precondition fails, the row holds in `ready` / `blocked` and
does NOT enter `/dev-story`.
