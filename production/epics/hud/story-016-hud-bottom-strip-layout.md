# Story 016: S11-UX-HUD-BOTTOM-STRIP-LAYOUT -- HUD Bottom Strip Layout (Composition Only)

> **Epic**: HUD
> **Story ID**: S11-UX-HUD-BOTTOM-STRIP-LAYOUT
> **Status**: Done -- closed by PROMPT 956 `/story-done` against
> `origin/main@45c2d03a9be6d8a23ddaabf3088397312b53481b`
> (PROMPT 955 integration of PROMPT 954 worker branch
> `work/s14-hud-bottom-strip`)
> **Layer**: HUD / Presentation (layout / composition only)
> **Type**: UI -- layout composition + visual evidence
> **Sprint**: Sprint 14 (Should Have row; rank 8 Tier 1 layout surface
> deferrable row closed by PROMPT 956)
> **Authored**: 2026-05-14 by PROMPT 879 (worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-hud-layout-story-authoring`,
> branch `story/s14-hud-layout-story-authoring`)
> **Authoring source-of-truth**: `origin/main@dd9630b` (PROMPT 877
> `integrate(s13): merge work/s13-r2-placement-crash-audit (server story 002 / PROMPT 874)`;
> session-start HEAD was `origin/main@51e6228` PROMPT 871 — worktree
> fast-forwarded to `dd9630b` during authoring to keep source-of-truth current)

---

## Status / No-Claim Banner

PROMPT 956 `/story-done` closure (2026-05-15) marks this story
Done on the basis of PROMPT 954 worker evidence, PROMPT 955
integration evidence, and the integrated files at
`origin/main@45c2d03a9be6d8a23ddaabf3088397312b53481b`.
This closure is paperwork-only. It does not modify implementation,
tests, Cargo files, Sprint 14 plan, Sprint 14 QA plan, stage, smoke,
Team-QA, gate-check, or release-check artifacts.

Runtime browser/WASM screenshots at 1920x1080 and 1366x768 remain
deferred. The automated ECS/node-intent evidence verifies structure,
fixed dimensions, parentage, z-layer consumption, top-strip ownership,
and no viewport-scaled HUD text. PROMPT 956 does not claim screenshot
capture completion.

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
(`Res<CurrentClientPhase>`, own `ClassId` via
`S2CGameSnapshot`) per ADR-021 + ADR-002. This story changes how the
bottom-strip region (own player figurine + any future bottom-anchored
readouts) is **composed
visually on screen** — not what is rendered or where the data comes
from. Current mana (`mana_label`) and reserve mana
(`reserve_container` / `reserve_label`) remain owned by `HudTopStrip` per
`S11-UX-HUD-TOP-STRIP-LAYOUT` story 015, PROMPT 942 closure, and
`tests/integration/ui_clean_pass/hud_top_strip_layout_test.rs`.

---

## Source Finding

- PROMPT 802 audit `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`
  §3.2 (HUD) lists defects H1, H2, H9 against the current HUD bottom
  strip in `client/src/ui/hud/mod.rs`:
  - **H1**: every HUD child uses `PositionType::Absolute` with hard-coded
    `Val::Px(margin + N)` offsets relative to the corner. No flex
    bottom-strip composition.
  - **H2**: magic offsets — figurine at `bottom: hud_margin + 60.0`,
    plus, historically, current mana bar / reserve mana diamond at
    independent `bottom: Val::Px(...)` offsets. Story 015 / PROMPT 942
    has since made current mana and reserve mana `HudTopStrip`
    children; this story repairs only the remaining bottom-strip-owned
    figurine/future-readout surface.
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
figurine sits at `bottom: Val::Px(config.hud_margin_px + 60.0)`.
Before story 015 / PROMPT 942, current mana and reserve mana also used
independent absolute offsets; PROMPT 942 deliberately moved those
readouts under `HudTopStrip`, and this story does **not** move them
back to the bottom strip. The remaining bottom-strip-owned defect is
the figurine area and any future bottom-anchored readout surface. This:

1. Cannot self-adapt when an upstream constant changes (for example,
   moving the figurine up 8 px means any future bottom readout or
   adjacent bottom element must be re-offset by hand to avoid overlap).
2. Cannot self-adapt when the figurine asset's pixel dimensions
   change (today: 64×64; if PAW-004 final-art replacement lands at
   80×80 the current absolute placement could collide with future
   bottom readouts or adjacent bottom elements even though art
   replacement is itself out of scope; the refactor must be robust to
   that class of change).
3. Cannot guarantee stable rendered dimensions across viewports —
   1366×768 vs 1920×1080 at the same pixel offsets renders identically
   in absolute space, but the strip can extend past the visible
   bottom edge at narrower aspect ratios.
4. Cannot enforce font sizing that is independent of viewport width.
   Anti-regression below forbids viewport-width font scaling on any
   bottom-strip child text that this story or later bottom-readout work
   introduces.

**Prevention target**: introduce a single HUD bottom-strip flex
parent (name: `HudBottomStrip`) anchored to the bottom-left of the
viewport, hosting the own-player figurine and any future
bottom-anchored readout slots as flex children. Use the foundational
primitives delivered by `S11-TD-UI-FLEX-STRIPS` (flex direction, gap,
padding tokens) and `S11-TD-UI-ZINDEX-LAYERS` (HUD z layer slot).
Numeric inputs (spacing, container widths) come from
`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`. Current mana and reserve mana are
canonical `HudTopStrip` children after story 015 / PROMPT 942 and are
not children of `HudBottomStrip`.

The visual layout intent (which element goes where, anchor to
bottom-left) is **preserved unchanged from the current code** under
this story — composition refactor, not a redesign. Opponent figurine
composition is **explicitly out of scope** here (separate Sprint 14
candidate `S11-UX-HUD-OPP-FIGURINE` per PROMPT 802 §3.2 H10).

---

## Context

### Existing surface

- **`client/src/ui/hud/mod.rs`** (per ADR-021): HUD spawn function
  `spawn_hud`; the bottom-strip-owned region remains the own-player
  class figurine area, historically placed by independent absolute
  offsets under `HudRoot`:
  - Class figurine: `bottom: Val::Px(config.hud_margin_px + 60.0)`,
    `width: Val::Px(64.0)`, `height: Val::Px(64.0)`.
  - Current mana bar and reserve mana diamond are **not**
    bottom-strip-owned after story 015 / PROMPT 942. They are direct
    `HudTopStrip` children on `origin/main`, and that ownership is
    asserted by
    `tests/integration/ui_clean_pass/hud_top_strip_layout_test.rs`.
  - Pixel constants for mana shape geometry
    (`CURRENT_MANA_BAR_WIDTH_PX`, `CURRENT_MANA_BAR_HEIGHT_PX`,
    `RESERVE_MANA_DIAMOND_SIZE_PX`,
    `RESERVE_MANA_DIAMOND_ROTATION_DEGREES`) remain preserved by this
    story, but not reparented into `HudBottomStrip`.
- **`design/gdd/hud.md`** TR-HUD-001 / TR-HUD-002 / TR-HUD-005:
  describes the gold / mana / objective-identity render rules. Does
  NOT prescribe absolute vs flex composition.
- **`docs/ux/ui-clean-pass-roadmap.md`** sequencing rules and Phase 1
  dependency list (rank 8).
- **`reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`**
  §3.2 H1 / H2 / H9.

### GDD / ADR / TR trace

- **GDD / accessibility trace**: `design/gdd/hud.md` TR-HUD-002 (mana
  display), plus HUD story 011 / `A11Y-ST-13` for the current mana bar
  / reserve mana diamond shape distinction. All preserved by this story.
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
  confirm story 016 does not move it out of `HudTopStrip`.
- **Lightyear**: no Lightyear changes; `liv-bevy-lightyear` not
  required.

### Control Manifest Rules

- Required: HUD bottom strip composition uses a single flex parent
  (`HudBottomStrip` marker component) with explicit `Display::Flex`.
- Required: Foundational primitives from `S11-TD-UI-FLEX-STRIPS`,
  `S11-TD-UI-ZINDEX-LAYERS`, and `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  are consumed (not re-implemented inline).
- Required: The bottom-strip-owned `figurine` entity remains reachable
  via `HudEntities`, and a new `bottom_strip` field points at the new
  parent. Existing top-strip readout identities (`mana_label`,
  `reserve_container`, `reserve_label`) remain reachable via
  `HudEntities` and remain under `HudTopStrip`; story 016 must not
  reparent them under `HudBottomStrip`.
- Required: ADR-021 system schedule preserved (no `PresentationSet`
  reordering, no new schedule sets added under this story).
- Required: ADR-002 + ADR-021 preserved (HUD remains read-only).
- Required: HUD story 011 / `A11Y-ST-13` shape distinction (current
  mana = bar / reserve mana = diamond) preserved exactly in
  `HudTopStrip`.
  `ManaShapeGeometry` component unchanged. Rotation transform
  preserved on reserve diamond.
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
  bottom-strip elements (`QA-COND-0005` preserved). Current mana and
  reserve mana remain top-strip read-only HUD indicators; this story
  does not use them to claim accessibility completion.
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
| Z-index layers | `S11-TD-UI-ZINDEX-LAYERS` (rank 1, Tier 0 Must) | HUD bottom strip needs explicit z layer assignment so the figurine and any future bottom readouts do not occlude the bottom-anchored RESOLUTION dim overlay incorrectly when respawned (PROMPT 802 §3.9 G1). |
| Flex strip primitives | `S11-TD-UI-FLEX-STRIPS` (rank 3, Tier 0 Must) | Provides shared flex tokens; bottom-strip layout consumes the same primitive set as the top strip. |
| Global UI design spec | `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, Tier 0 Must) | Provides numeric inputs for bottom-strip figurine spacing, future bottom-readout gaps, bottom anchor padding, and any per-element padding. |

**Sequencing relative to sibling Sprint 14 candidates**:

- `S11-UX-HUD-TOP-STRIP-LAYOUT` (story 015 in this epic): landed via
  PROMPT 942 and is now the canonical ownership contract for
  `HudTopStrip`. Current mana (`mana_label`) and reserve mana
  (`reserve_container` / `reserve_label`) must remain top-strip
  children; story 016 implementation must rebase on that contract and
  preserve
  `tests/integration/ui_clean_pass/hud_top_strip_layout_test.rs`.
- `S11-UX-HUD-OPP-FIGURINE` (story 017 in this epic): sequenced AFTER
  this story per `docs/ux/ui-clean-pass-roadmap.md` "Tier 1
  Should-Priority Adjacent Rows" table (it pairs with rank 8). The
  opponent-figurine composition will live inside (or adjacent to) the
  `HudBottomStrip` parent introduced here; authoring `S11-UX-HUD-OPP-FIGURINE`
  before this story landed would force a re-write.

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Single flex parent introduced**: GIVEN the HUD root
  spawn, WHEN `spawn_hud` runs, THEN a single new child entity carrying
  a `HudBottomStrip` marker and `Display::Flex` is spawned as a child
  of `HudRoot`. The own-player figurine is reparented under
  `HudBottomStrip` (not a direct child of `HudRoot`). Current mana and
  reserve mana are **not** reparented under `HudBottomStrip`; they
  remain direct `HudTopStrip` children per story 015 / PROMPT 942.

- [x] **AC2 -- Flex composition replaces absolute offsets on
  bottom-strip children**: GIVEN the post-refactor spawn, WHEN each
  bottom-strip child `Node` is inspected, THEN none of them carries
  `PositionType::Absolute` with `bottom: Val::Px(hud_margin + N)` or
  `left: Val::Px(hud_margin + N)` style absolute offsets. Children
  rely on `HudBottomStrip`'s `flex_direction` / gap / padding for
  their position. The `HudRoot` may keep its viewport-spanning
  absolute Node; **bottom-strip children** change.

- [x] **AC3 -- Entity identity preserved in `HudEntities`**:
  GIVEN the post-refactor `HudEntities` resource, WHEN inspected,
  THEN `figurine`, `mana_label`, `reserve_container`, `reserve_label`
  still point at the same logical entities they did pre-refactor. A
  new `bottom_strip` field is added pointing at the new parent, and
  `mana_label` / `reserve_container` / `reserve_label` remain under
  the existing `HudTopStrip` tree.

- [x] **AC4 -- HUD story 011 / `A11Y-ST-13` shape distinction preserved**:
  GIVEN the post-refactor spawn, WHEN current mana and reserve mana
  entities are inspected in `HudTopStrip`, THEN the current mana entity retains
  `ManaShapeGeometry { kind: Bar, width_px: CURRENT_MANA_BAR_WIDTH_PX,
  height_px: CURRENT_MANA_BAR_HEIGHT_PX }` and the reserve mana
  entity retains its diamond geometry with
  `RESERVE_MANA_DIAMOND_SIZE_PX` size and
  `RESERVE_MANA_DIAMOND_ROTATION_DEGREES` rotation. Shape distinction
  is not collapsed and the readouts are not moved into
  `HudBottomStrip` by the bottom-strip flex refactor.

- [x] **AC5 -- ADR-021 schedule preserved**: GIVEN a `cargo build -p
  client` (or equivalent), WHEN run against the post-refactor code,
  THEN no new system, system-set, or schedule wiring is introduced.
  `HudPlugin` registers the same sets in the same order.

- [x] **AC6 -- Visual evidence captured at two viewports**: GIVEN
  the post-refactor build runs end-to-end through the friend-game
  route, WHEN HUD is visible (any non-`Hidden` phase), THEN
  screenshots are captured at **desktop** (1920×1080) AND at a
  **smaller viewport** (1366×768 minimum). Captures land under
  `production/qa/evidence/sprint-14-hud-bottom-strip-layout/`
  (NEW) with filenames `bottom-strip-1920x1080-<phase>.png` and
  `bottom-strip-1366x768-<phase>.png` for at least one phase where
  the own-player figurine is visible. Reserve-mana lighting is
  top-strip evidence, not a bottom-strip requirement.

- [x] **AC7 -- Text fitting anti-regression**: GIVEN the captures,
  WHEN visually inspected, THEN no bottom-strip-owned text is clipped
  or truncated by its container. If story 016 introduces no
  bottom-strip text beyond the figurine, the evidence document records
  that there is no bottom-strip text fitting surface; current/reserve
  mana text remains covered by the top-strip contract.

- [x] **AC8 -- Stable dimensions anti-regression**: GIVEN the captures,
  WHEN dimensions of each bottom-strip child are measured, THEN each
  child's rendered width and height is the same at 1920×1080 as at
  1366×768. Specifically: figurine renders at 64×64 (or its current
  pixel size constant), and any future bottom-owned readout child has
  identical fixed dimensions across viewports. Current mana and
  reserve mana dimensions remain top-strip-owned assertions.

- [x] **AC9 -- No overlap anti-regression**: GIVEN the captures at
  both viewports, WHEN siblings are inspected, THEN no bottom-strip
  child visually overlaps a sibling, the timer bar (top strip), or
  any non-bottom-strip element. The evidence explicitly confirms that
  `HudBottomStrip` does not occlude the `HudTopStrip` current/reserve
  mana readouts.

- [x] **AC10 -- No viewport-width font scaling anti-regression**:
  GIVEN a grep across `client/src/ui/hud/` post-refactor, WHEN run
  with pattern `Val::Percent`/`Val::Vw`/`Val::Vh` filtered to lines
  touching `TextFont` or `font_size`, THEN zero hits on bottom-strip
  children. Reserve mana label font sizing remains fixed pixel under
  the top-strip contract and is not re-scoped to `HudBottomStrip`.

- [x] **AC11 -- Z-index layer slot consumed (not re-invented)**: GIVEN
  the post-refactor spawn, WHEN the `HudBottomStrip` z positioning is
  inspected, THEN it consumes the HUD layer slot defined by
  `S11-TD-UI-ZINDEX-LAYERS` (e.g. `HudLayers::BottomStrip` enum
  variant or equivalent named constant) — NOT a hard-coded
  `GlobalZIndex(N)` re-introduced inline.

- [x] **AC12 -- ADR-001 invariant preserved**: GIVEN the post-refactor
  build, WHEN any path that surfaces objective identity is inspected,
  THEN `was_fake` remains stripped at the Board Rendering boundary
  and is never exposed on a bottom-strip child. Scoreboard dots are
  not on the bottom strip; this AC is a defence-in-depth check.

- [x] **AC13 -- Sprint 13/14 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`
  (when authored), `production/stage.txt`, and PROMPT 761 gate-check
  artifact are diffed, THEN none of them are modified by this story.

- [x] **AC14 -- No accept-risk closure claimed**: GIVEN the evidence
  document, WHEN inspected, THEN it explicitly does NOT claim closure
  of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-004-a`,
  or any other accept-risk disposition. Final-art replacement on HUD
  bottom-strip elements is explicitly out of scope.

- [x] **AC15 -- Targeted regression passes**: GIVEN the post-refactor
  code, WHEN `cargo test -p client --lib` is run, THEN it passes.
  Existing HUD tests (story 001 scaffold, story 002 gold/mana display,
  story 011 shape distinction) continue to pass because
  `HudEntities` field identities are preserved. The story 015 top-strip
  layout regression also remains authoritative for `mana_label` and
  `reserve_container` parentage.

- [x] **AC16 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-14-hud-bottom-strip-layout/README.md`
  (NEW). Records the build commit, the two viewport captures, the
  longest-content observation (AC7), the dimension measurements
  (AC8), no-claim restatement, and cross-links to PROMPT 802 §3.2
  H1 / H2 / H9 + `docs/ux/ui-clean-pass-roadmap.md` rank 8.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hud/mod.rs` | Refactor `spawn_hud` to introduce `HudBottomStrip` flex parent + reparent the own-player figurine and any future bottom-owned readout children. Add `bottom_strip: Entity` field on `HudEntities`. Preserve `mana_label` / `reserve_container` / `reserve_label` under `HudTopStrip`. |
| `tests/integration/ui_clean_pass/hud_bottom_strip_layout_test.rs` | NEW, analogous to the accepted `hud_top_strip_layout_test.rs`, asserting `HudBottomStrip` ownership is limited to the figurine/future bottom readouts and does not reparent top-strip mana/reserve entities. |
| `production/qa/evidence/sprint-14-hud-bottom-strip-layout/README.md` | NEW evidence document. |
| `production/qa/evidence/sprint-14-hud-bottom-strip-layout/bottom-strip-1920x1080-<phase>.png` | NEW screenshot capture (desktop). |
| `production/qa/evidence/sprint-14-hud-bottom-strip-layout/bottom-strip-1366x768-<phase>.png` | NEW screenshot capture (smaller viewport). |
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
  where the own-player figurine is visible. Reserve-mana lighting is
  not required for bottom-strip evidence because reserve mana remains
  top-strip-owned.
- Longest-content observation table (per AC7).
- Per-child rendered dimension table (per AC8).
- Overlap audit (per AC9).
- Z-index layer slot citation (per AC11).
- Confirmation that `mana_label`, `reserve_container`, and
  `reserve_label` remain owned by `HudTopStrip` per story 015 /
  PROMPT 942.
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to PROMPT 802 §3.2 H1 / H2 / H9.
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` rank 8.
- Cross-link to HUD story 011 / `A11Y-ST-13` (shape distinction).

---

## Regression Commands Expected

For the implementation prompt (NOT the authoring prompt):

- `cargo build -p client` (must succeed; AC5).
- `cargo test -p client --lib` (HUD-scoped tests; AC15).
- `cargo test -p client --test hud_top_strip_layout_test` (confirms
  story 015 / PROMPT 942 mana/reserve parentage remains intact).
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`
- Grep `Val::Percent|Val::Vw|Val::Vh` filtered to `client/src/ui/hud/`
  matches against `font_size` / `TextFont` (must be zero; AC10).

The authoring prompt (PROMPT 879) runs only `git diff --check`,
`git diff --cached --check`, `git status --short --branch`.

---

## Out of Scope

- HUD top strip composition and ownership, now canonical via
  `S11-UX-HUD-TOP-STRIP-LAYOUT` story 015 / PROMPT 942.
- Moving current mana (`mana_label`) or reserve mana
  (`reserve_container` / `reserve_label`) out of `HudTopStrip`.
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
- File-collision risk against story 015 (HUD top strip): story 015
  landed via PROMPT 942 and owns the top-strip contract. Story 016
  implementation must rebase on the PROMPT 942 code, preserve
  `HudTopStrip` ownership for `mana_label`, `reserve_container`, and
  `reserve_label`, and keep the accepted top-strip layout regression
  passing.
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
5. Coordination decision: story 016 MUST run after the landed
   `S11-UX-HUD-TOP-STRIP-LAYOUT` story 015 / PROMPT 942 contract and
   MUST preserve the accepted top-strip mana/reserve parentage test.

If any precondition fails, the row holds in `ready` / `blocked` and
does NOT enter `/dev-story`.

---

## Completion Notes

- **Verdict**: Done / PASS with explicit runtime-capture deferral. AC1-AC5,
  AC10-AC12, and AC15 are backed by automated ECS/source tests and PROMPT
  955 integration checks. AC6, AC8, AC9, and AC16 preserve the deferred
  browser/WASM screenshot limitation rather than claiming PNG capture.
- **Source of truth**: `origin/main@45c2d03a9be6d8a23ddaabf3088397312b53481b`
  (PROMPT 955 integration merge of `origin/work/s14-hud-bottom-strip`
  worker evidence commit `acfc43860c22c81f5a7d3678ec54c96bc46af09f`).
- **Implementation evidence**: `client/src/ui/hud/mod.rs` now exposes
  `HudBottomStrip`, `HudEntities.bottom_strip`, `hud_bottom_strip_node()`,
  and `bottom_strip_figurine_node()`. The figurine is a flex child of
  `HudBottomStrip`; mana, reserve mana, and timer remain in `HudTopStrip`.
- **Test evidence**: PROMPT 955 reports `cargo test -p client --test
  hud_bottom_strip_layout_test` 8/8 PASS, `hud_top_strip_layout_test` 6/6
  PASS, `hud_mana_shape_distinction_test` 3/3 PASS, `ui_clean_pass_strips_test`
  20/20 PASS, and `cargo test -p client --lib` 45/45 PASS.
- **Acceptance detail**: AC1 single flex parent PASS; AC2 no absolute
  bottom-strip child offsets PASS; AC3 entity identities and structural
  `bottom_strip` PASS; AC4 current/reserve mana shape distinction PASS;
  AC5 schedule preserved PASS; AC6 visual capture deferred/no screenshot
  claim; AC7 no bottom-strip text surface PASS; AC8 fixed 64x64 figurine
  node intent PASS with runtime measurement deferred; AC9 no-overlap
  node intent PASS with runtime screenshot deferred; AC10 no viewport-scaled
  HUD font-size PASS; AC11 `z_layers::UI_BASE` consumed PASS; AC12 scoreboard
  dots outside bottom strip PASS; AC13 integration forbidden-path review PASS;
  AC14 accept-risk no-claims preserved PASS; AC15 targeted regressions PASS;
  AC16 README evidence slot present PASS with runtime capture deferred.
- **No-claims preserved**: Sprint 14 remains active; stage remains Polish;
  PROMPT 761 Polish->Release FAIL is not retried. `S8-QA-001-W1` remains
  OPEN; `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-004-a` remain
  accepted-risk. No release/RC/full-game/broad-accessibility/playtest/
  final-art/Sprint-14-closeout claim is made.

## Closure Trail

- 2026-05-14: PROMPT 879 authored the Sprint 14 candidate story.
- 2026-05-15: PROMPT 954 implemented HUD bottom strip layout on
  `work/s14-hud-bottom-strip` (`1ad7296` implementation, `acfc438`
  evidence tip).
- 2026-05-15: PROMPT 955 integrated the worker branch into `origin/main`
  at `45c2d03a9be6d8a23ddaabf3088397312b53481b`.
- 2026-05-15: PROMPT 956 performed serialized `/story-done` paperwork,
  marked this story Done, flipped the Sprint 14 row to done, appended
  `sprint_14_story_done`, and prepended session-state banners.
