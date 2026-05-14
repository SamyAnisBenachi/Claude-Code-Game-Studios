# Story 015: S11-UX-HUD-TOP-STRIP-LAYOUT -- HUD Top Strip Layout (Composition Only)

> **Epic**: HUD
> **Story ID**: S11-UX-HUD-TOP-STRIP-LAYOUT
> **Status**: Draft -- Sprint 14 candidate (Must Have framing per
> PROMPT 802 §3.2 H1 / H8 + `docs/ux/ui-clean-pass-roadmap.md` rank 7);
> NOT activated; Sprint 14 NOT activated
> **Layer**: HUD / Presentation (layout / composition only)
> **Type**: UI -- layout composition + visual evidence
> **Sprint**: Sprint 14 candidate (drawn from PROMPT 802 Expert UI Layout
> audit roadmap, reconciled by `docs/ux/ui-clean-pass-roadmap.md` rank 7
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
- HUD final-art replacement on the top strip (placeholder PNGs preserved
  under `PAW-TD-004-a` accept-risk)
- closure of any other Sprint 14 candidate row from `docs/ux/ui-clean-pass-roadmap.md`

Sprint 10 / Sprint 11 / Sprint 12 / Sprint 13 dispositions unchanged.
PROMPT 761 Polish->Release gate-check FAIL evidence preserved. PROMPT
802 audit roadmap accept-risk boundaries (`PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`) preserved verbatim.

**No optimistic client-side authority is introduced or proposed.**
The HUD top strip is read-only over server-authoritative state
(`Res<CurrentClientPhase>`, `GoldDisplayState`, `ManaDisplayState`)
per ADR-021 + ADR-002. This story changes how those values are
**composed visually on screen** — not what they are or where they
come from.

---

## Source Finding

- PROMPT 802 audit `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`
  §3.2 (HUD) lists defects H1, H2, H5, H8 against the current HUD top
  strip in `client/src/ui/hud/mod.rs`:
  - **H1**: every HUD child uses `PositionType::Absolute` with hard-coded
    `Val::Px(margin + N)` offsets relative to the corner. No flex top-strip
    composition.
  - **H2**: magic offsets — `top: hud_margin + 48.0` (timer bar),
    `HUD_GOLD_ROW_GAP_PX = 48.0`, `HUD_SECONDARY_ROW_GAP_PX = 28.0`. No
    shared spacing-scale constant.
  - **H5**: per-module typography constants (`HUD_GOLD_FONT_SIZE_PX = 40`,
    `HUD_RESOURCE_TEXT_MIN_SIZE_PX = 20`, `HUD_RESERVED_GOLD_FONT_SIZE_PX = 26`,
    `HUD_SECONDARY_FONT_SIZE_PX = 20`) — not shared with the rest of the UI.
  - **H8**: top-strip layout (gold / mana / phase) not composed via a flex
    parent — each line is its own absolute child.
- PROMPT 685 row 2 (audit row "HUD strip slice") is `subsumed-by`
  `S11-UX-HUD-TOP-STRIP-LAYOUT` per
  `docs/ux/ui-clean-pass-roadmap.md` reconciliation matrix.
- `docs/ux/ui-clean-pass-roadmap.md` rank 7 places this slug as Tier 1
  Must (0.75d) with Phase 1 dependency on ranks 1 (`S11-TD-UI-ZINDEX-LAYERS`),
  3 (`S11-TD-UI-FLEX-STRIPS`), 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`).
- `docs/ux/ui-clean-pass-roadmap.md` "3-4 Highest-Impact Rows For Sprint
  14 Must Have Framing" item 4 names this slug as the HUD slot of the
  Sprint 14 Must Have framing.

---

## Problem Class / Prevention Target

**Defect class**: HUD top strip composition uses absolute-positioned
children with magic per-line offsets (`top: hud_margin + N`). Every
gold / mana / phase / timer line is its own absolute child of `HudRoot`,
which:

1. Cannot self-adapt when an upstream constant changes (move one line
   down 4 px → every line below must be re-offset by hand).
2. Cannot self-adapt when string content widens (long phase label,
   `Xg (Yr)` ECONOMY_AUCTION inline gold format with two-digit reserved
   gold) — siblings can overlap because the parent does not measure them.
3. Cannot guarantee stable rendered dimensions across viewports — a
   1366×768 viewport at the same pixel offsets renders identically to
   1920×1080 in absolute space, but the strip can extend off-edge or
   collide with the bottom strip at narrower aspect ratios.
4. Cannot enforce font sizing that is independent of viewport width —
   ad-hoc `Val::Px(...)` font sizes are fixed-pixel today, which is
   the intended behaviour, but the lack of a flex parent makes it
   tempting for a follow-on author to "fix it" by switching to
   `Val::Percent(...)` font scaling. Anti-regression below forbids
   that.

**Prevention target**: introduce a single HUD top-strip flex parent
(name: `HudTopStrip`) that hosts the phase label, round counter, gold
labels (own + opponent), mana label, reserve mana label, and phase
timer bar as flex children. Use the foundational primitives delivered
by `S11-TD-UI-FLEX-STRIPS` (flex direction, gap, padding tokens) and
`S11-TD-UI-ZINDEX-LAYERS` (HUD z layer slot). Numeric inputs
(spacing, line-height, container widths) come from
`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`.

The visual layout intent (which line goes where, which is primary,
which is secondary) is **preserved unchanged from the current code**
under this story — this is a composition refactor, not a redesign of
which fields appear. The story's deliverable is the structural change
(absolute → flex), not a new information hierarchy.

---

## Context

### Existing surface

- **`client/src/ui/hud/mod.rs`** (per ADR-021): HUD spawn function
  `spawn_hud` (lines 482-661) declares 16 `Node{}` blocks; top-strip
  region currently composed via:
  - `top_left_node()` + `top_left_second_line_node()` for phase /
    round labels.
  - `spawn_gold_label` called twice with explicit `top_offset_px = 0.0`
    then `top_offset_px = HUD_GOLD_ROW_GAP_PX (48.0)`.
  - `current_mana_bar_node` for mana.
  - `spawn_reserve_mana_label` for reserve mana.
  - Timer bar at `top: Val::Px(config.hud_margin_px + 48.0)`,
    `width: Val::Px(HUD_PHASE_TIMER_BAR_MAX_WIDTH_PX)`,
    `height: Val::Px(8.0)`.
- **`design/gdd/hud.md`** TR-HUD-001 / TR-HUD-002 / TR-HUD-003:
  describes what is rendered (gold format, mana format, phase strings).
  Does **not** prescribe absolute vs flex composition.
- **`docs/ux/ui-clean-pass-roadmap.md`** sequencing rules and
  Phase 1 dependency list.
- **`reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`**
  §3.2 H1 / H2 / H5 / H8.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hud.md` TR-HUD-001 (gold), TR-HUD-002 (mana),
  TR-HUD-003 (phase + round), TR-HUD-007 (`S2CGoldUpdate` /
  `S2CGoldBroadcast` tie-break). All preserved by this story.
- **ADR-021** (Presentation Layer Architecture): `HudPlugin` 4th in
  `PresentationPlugin`; `PresentationSet` ordering preserved. This
  story does not change the system schedule, only the spawn-time Node
  composition.
- **ADR-002** (Client-Server Authority): HUD remains read-only.
- **ADR-001** (Objective Identity Unicast): scoreboard dots are not
  on the top strip; this story does not touch them.
- **TR registry**: no new TR (composition refactor of existing TRs).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits under
  `client/src/ui/hud/`. The story implementation prompt MUST activate
  this skill before editing.
- **Why mandatory**: Bevy 0.18 `Node` composition uses the Required
  Components API; flex children must be declared with explicit
  `Display::Flex`, `flex_direction`, `align_items`, `justify_content`,
  `column_gap` / `row_gap`. Pre-0.15 `NodeBundle` patterns will not
  compile.
- **Lightyear**: no Lightyear changes; `liv-bevy-lightyear` not
  required.

### Control Manifest Rules

- Required: HUD top strip composition uses a single flex parent
  (`HudTopStrip` marker component) with explicit `Display::Flex`.
- Required: Foundational primitives from `S11-TD-UI-FLEX-STRIPS`,
  `S11-TD-UI-ZINDEX-LAYERS`, and `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  are consumed (not re-implemented inline).
- Required: All existing HUD top-strip pre-pooled entity identities
  (`phase_label`, `round_counter`, `own_gold_parent`, `own_gold_span`,
  `opponent_gold_parent`, `opponent_gold_span`, `mana_label`,
  `reserve_container`, `reserve_label`, `timer_bar`) remain reachable
  via `HudEntities` — child reparenting under the new flex parent
  preserves `Entity` IDs.
- Required: ADR-021 system schedule preserved (no
  `PresentationSet` reordering, no new schedule sets added under
  this story).
- Required: ADR-002 + ADR-021 preserved (HUD remains read-only).
- Forbidden: Introducing `Val::Percent(...)` on any `font_size` /
  text-size field of a top-strip child. Font sizing remains fixed
  pixel by spec (text-fitting handled by flex container, not by
  viewport-scaled fonts).
- Forbidden: Per-line `PositionType::Absolute` on top-strip children
  after the refactor lands. The `HudRoot` itself may remain absolute
  to anchor the strip to the viewport corner; **its top-strip flex
  child sub-tree** must use flex composition.
- Forbidden: Modifying any code outside `client/src/ui/hud/` in
  service of this story. Cross-module changes are out of scope and
  require a separate story.
- Forbidden: Final-art / asset replacement on HUD top-strip elements
  (`PAW-TD-004-a` accept-risk preserved).
- Forbidden: Standard-tier accessibility hit-target ≥44px work
  (`QA-COND-0005` preserved).
- Forbidden: Modifying server protocol, ECS message routing, or any
  presentation system that consumes the HUD entities downstream.

---

## Story Classification

**Story type**: UI -- layout composition refactor + visual evidence.

This is **NOT** a:

- Logic story (no formula or state machine change; existing observer
  registration, message drain ordering, and tween cancel-and-replace
  semantics preserved).
- Integration story (no new system-set or schedule wiring).
- Final-art story (placeholder assets preserved).
- Accessibility story (hit-targets, keyboard navigation, screen reader
  support all out of scope).
- Animation story (no new tween, no new `bevy_tweening` `Animator`).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type",
UI stories deliver a **manual walkthrough doc OR interaction test**
with screenshot evidence as ADVISORY gate. Anti-regression assertions
(text fitting, stable dims, no overlap, no viewport-width font
scaling) are checkable by visual capture; a layout assertion test
under `tests/integration/ui/` may be added by the implementation
prompt **only if** `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4) has
landed and exposes the test bin scaffolding.

---

## Dependencies (must be Done before /dev-story on this story)

Per `docs/ux/ui-clean-pass-roadmap.md` rank 7 "Phase 1 dependency"
(ranks 1, 3, 6):

| Dependency | Slug | Why blocking |
|---|---|---|
| Z-index layers | `S11-TD-UI-ZINDEX-LAYERS` (rank 1, Tier 0 Must) | HUD top strip needs explicit z layer assignment so reconnect / snapshot rebuild can respawn its sub-tree without spawn-order fragility (PROMPT 802 §3.9 G1). The composition refactor MUST consume the layer module, not re-introduce spawn-order reliance. |
| Flex strip primitives | `S11-TD-UI-FLEX-STRIPS` (rank 3, Tier 0 Must) | Provides shared `flex_direction`, `align_items`, `justify_content`, `column_gap`, `padding` tokens. Without this, the top-strip refactor would have to re-author primitives inline and the original "magic offset" defect class would recur on the next surface. |
| Global UI design spec | `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, Tier 0 Must) | Provides numeric inputs for spacing, gap, padding, and line-height that the flex container must use. Required before this story can fix a concrete spacing scale (PROMPT 802 §9 producer-decision-2). |

**Optional but recommended** (not blocking — if absent, this story
still ships but with lower assertion coverage):

- `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4, Tier 0 Must) — if
  present, the implementation prompt MAY add a viewport-invariant
  test asserting top-strip composition holds at 1366×768 + 1920×1080.
  If absent, the visual capture in `Evidence Path` below is the sole
  evidence.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Single flex parent introduced**: GIVEN the HUD root
  spawn, WHEN `spawn_hud` runs, THEN a single new child entity carrying
  a `HudTopStrip` marker and `Display::Flex` is spawned as a child of
  `HudRoot`. All top-strip pre-pooled children (`phase_label`,
  `round_counter`, `own_gold_parent`, `opponent_gold_parent`,
  `mana_label`, `reserve_container`, `timer_bar`) are reparented under
  `HudTopStrip` (not direct children of `HudRoot`).

- [ ] **AC2 -- Flex composition replaces absolute offsets on
  top-strip children**: GIVEN the post-refactor spawn, WHEN each
  top-strip child `Node` is inspected, THEN none of them carries
  `PositionType::Absolute` with `top: Val::Px(hud_margin + N)` or
  `left: Val::Px(hud_margin + N)` style absolute offsets. Children
  rely on `HudTopStrip`'s `flex_direction` / gap / padding for their
  position. (The `HudRoot` itself may keep its full-viewport absolute
  anchor; only top-strip children change.)

- [ ] **AC3 -- Entity identity preserved in `HudEntities`**:
  GIVEN the post-refactor `HudEntities` resource, WHEN inspected,
  THEN it still exposes every existing field (`phase_label`,
  `round_counter`, `own_gold_parent`, `own_gold_span`,
  `opponent_gold_parent`, `opponent_gold_span`, `mana_label`,
  `reserve_container`, `reserve_label`, `timer_bar`, …). A new
  `top_strip` field is added pointing at the new parent. All existing
  fields point at the same logical entities they did pre-refactor (so
  downstream systems that look up entities by `HudEntities.x` continue
  to work).

- [ ] **AC4 -- ADR-021 schedule preserved**: GIVEN a `cargo build -p
  client` (or equivalent), WHEN run against the post-refactor code,
  THEN no new system, system-set, or schedule wiring is introduced by
  this story. `HudPlugin` registers the same sets in the same order
  (`PhaseTransition` → `MessageDrain` → `StateSync` → `AnimationTick`).

- [ ] **AC5 -- Visual evidence captured at two viewports**: GIVEN
  the post-refactor build runs end-to-end through the friend-game
  route, WHEN HUD is visible (any non-`Hidden` phase), THEN
  screenshots are captured at **desktop** (1920×1080) AND at a
  **smaller viewport** (1366×768 minimum). Captures land under
  `production/qa/evidence/sprint-14-hud-top-strip-layout/` (NEW)
  with filenames `top-strip-1920x1080-<phase>.png` and
  `top-strip-1366x768-<phase>.png` for at least one phase
  (e.g. `DraftShop`).

- [ ] **AC6 -- Text fitting anti-regression**: GIVEN the captures,
  WHEN visually inspected against the longest expected content
  (ECONOMY_AUCTION `Xg (Yr)` inline gold with two-digit reserved gold;
  phase label `DraftAuction`; round counter `Round 6 / 6`; reserve
  mana double-digit), THEN no text is clipped or truncated by its
  container. The evidence document records the longest content
  observed and confirms no clipping.

- [ ] **AC7 -- Stable dimensions anti-regression**: GIVEN the captures,
  WHEN dimensions of each top-strip child are measured (manually from
  the capture, or asserted by a layout test if `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`
  exposes it), THEN each child's rendered width and height is the same
  at 1920×1080 as at 1366×768 (i.e. fixed pixel sizing preserved; no
  viewport-width font scaling, no `Val::Percent` on font sizes).

- [ ] **AC8 -- No overlap anti-regression**: GIVEN the captures at
  both viewports, WHEN siblings are inspected, THEN no top-strip
  child visually overlaps a sibling, the timer bar, or any
  non-top-strip element (HUD figurine, scoreboard dots, dim overlay,
  bottom-strip elements). Captures from any phase that lights the
  reserve-mana label (ECONOMY_AUCTION with `reserve_mana > 0`) are
  included.

- [ ] **AC9 -- No viewport-width font scaling anti-regression**:
  GIVEN a grep across `client/src/ui/hud/` post-refactor, WHEN run
  with pattern `Val::Percent`/`Val::Vw`/`Val::Vh` filtered to lines
  touching `TextFont` or `font_size`, THEN zero hits. Font sizes on
  top-strip children remain fixed pixel (`Val::Px`) per
  `HUD_GOLD_FONT_SIZE_PX` / `HUD_SECONDARY_FONT_SIZE_PX` /
  `HUD_RESERVED_GOLD_FONT_SIZE_PX` (or their `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  successors).

- [ ] **AC10 -- Z-index layer slot consumed (not re-invented)**: GIVEN
  the post-refactor spawn, WHEN the `HudRoot` and `HudTopStrip` z
  positioning is inspected, THEN they consume the HUD layer slot
  defined by `S11-TD-UI-ZINDEX-LAYERS` (e.g. `HudLayers::TopStrip`
  enum variant or equivalent named constant) — NOT a hard-coded
  `GlobalZIndex(N)` re-introduced inline.

- [ ] **AC11 -- ADR-001 invariant preserved**: GIVEN the post-refactor
  build, WHEN any path that surfaces objective identity is inspected,
  THEN `was_fake` remains stripped at the Board Rendering boundary
  and is never exposed on a top-strip child. (Scoreboard dots are not
  on the top strip; this AC is a defence-in-depth check that the
  refactor did not accidentally bring objective state into the
  strip.)

- [ ] **AC12 -- Sprint 13/14 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`
  (when authored), `production/stage.txt`, and PROMPT 761 gate-check
  artifact are diffed, THEN none of them are modified by this story.

- [ ] **AC13 -- No accept-risk closure claimed**: GIVEN the evidence
  document, WHEN inspected, THEN it explicitly does NOT claim closure
  of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-004-a`,
  or any other accept-risk disposition. Final-art replacement on HUD
  top-strip elements is explicitly out of scope.

- [ ] **AC14 -- Targeted regression passes**: GIVEN the post-refactor
  code, WHEN `cargo test -p client --lib` is run (HUD-scoped unit
  tests in `client/src/ui/hud/`), THEN it passes. Existing HUD
  observer + tween + tie-break tests (story 004, story 009, story
  010) continue to pass because the entity identities in
  `HudEntities` are preserved (AC3).

- [ ] **AC15 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-14-hud-top-strip-layout/README.md`
  (NEW). Records the build commit, the two viewport captures, the
  longest-content observation (AC6), the dimension measurements
  (AC7), no-claim restatement, and cross-links to PROMPT 802 §3.2
  H1 / H2 / H5 / H8 + `docs/ux/ui-clean-pass-roadmap.md` rank 7.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hud/mod.rs` | Refactor `spawn_hud` to introduce `HudTopStrip` flex parent + reparent top-strip children. Replace `top_left_node()` / `top_left_second_line_node()` / `current_mana_bar_node()` / inline timer bar `Node{}` with flex-child `Node{}`s. Add `top_strip: Entity` field on `HudEntities`. |
| `client/src/ui/hud/<new-or-existing-submod>.rs` | If the flex strip primitive lives in a hud-local helper module (decision is implementation-prompt scope), it lands here. Most likely the primitive comes from `S11-TD-UI-FLEX-STRIPS`' shared module under `client/src/ui/design_tokens/` (or similar). |
| `tests/integration/ui/hud_top_strip_test.rs` | NEW *iff* `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4) exposes the test bin scaffolding. Optional. |
| `production/qa/evidence/sprint-14-hud-top-strip-layout/README.md` | NEW evidence document. |
| `production/qa/evidence/sprint-14-hud-top-strip-layout/top-strip-1920x1080-draft-shop.png` | NEW screenshot capture (desktop). |
| `production/qa/evidence/sprint-14-hud-top-strip-layout/top-strip-1366x768-draft-shop.png` | NEW screenshot capture (smaller viewport). |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set. Per the
forbidden-modules list in PROMPT 879 framing, `client/src/`,
`server/src/`, `shared/src/`, `tests/`, and `Cargo.toml` are NOT
touched by the authoring prompt — only by a future implementation
prompt run after Sprint 14 activates.

---

## Required Skills

- `liv-bevy-018` (MANDATORY for the implementation prompt).
- `liv-bevy-lightyear`: NOT required (no Lightyear changes).

The authoring prompt (PROMPT 879) does NOT invoke either skill
because no code is touched at authoring time.

---

## Evidence Path

`production/qa/evidence/sprint-14-hud-top-strip-layout/README.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- Build commit hash and branch.
- Two screenshots minimum: 1920×1080 + 1366×768 at the same phase
  (recommend `DraftShop` or `DraftAuction` because both light the
  longest-content cases).
- Longest-content observation table (per AC6).
- Per-child rendered dimension table (per AC7).
- Overlap audit (per AC8).
- Z-index layer slot citation (per AC10).
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to PROMPT 802 §3.2 H1 / H2 / H5 / H8.
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` rank 7.

---

## Regression Commands Expected

For the implementation prompt (NOT the authoring prompt):

- `cargo build -p client` (must succeed; AC4).
- `cargo test -p client --lib` (HUD-scoped tests; AC14).
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`
- Grep `Val::Percent|Val::Vw|Val::Vh` filtered to `client/src/ui/hud/`
  matches against `font_size` / `TextFont` (must be zero; AC9).

The authoring prompt (PROMPT 879) runs only `git diff --check`,
`git diff --cached --check`, `git status --short --branch` as
required by PROMPT 879 verification block.

---

## Out of Scope

- Any change to top-strip information hierarchy (which fields appear,
  which is primary, which is secondary). Composition refactor only.
- HUD bottom strip (figurine area) composition — separate Sprint 14
  candidate `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`.
- HUD opponent figurine composition — separate Sprint 14 candidate
  `S11-UX-HUD-OPP-FIGURINE`.
- HUD timer urgency visual treatment — separate Sprint 14+ candidate
  `S11-UX-HUD-TIMER-URGENCY-VISUAL-001`.
- HUD timer eyeball visual check (a manual capture story already
  authored at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`,
  Sprint 13 Should Have).
- Standard-tier accessibility on HUD top strip (`QA-COND-0005`
  preserved).
- Final-art replacement on HUD top-strip elements (`PAW-TD-004-a`
  preserved).
- Cross-surface design-token authoring (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`
  is its own story).
- Z-index layer module authoring (`S11-TD-UI-ZINDEX-LAYERS` is its
  own story).
- Flex strip primitive authoring (`S11-TD-UI-FLEX-STRIPS` is its own
  story).
- Viewport-invariant test bin authoring (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`
  is its own story; this story optionally **consumes** that bin but
  does not author it).
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
  none known. Sprint 13 story 018 (tracing targets) touches the
  `target:` argument inside `tracing` macros, which is orthogonal to
  spawn-time Node composition. If a Sprint 13 row touches
  `client/src/ui/hud/mod.rs` after this story enters `/dev-story`,
  the worker must rebase / re-check on Sprint 14 activation HEAD
  before starting.
- This story landed under the existing HUD epic (no new epic
  created); the HUD epic remains `Ready`.

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

If any precondition fails, the row holds in `ready` / `blocked` and
does NOT enter `/dev-story`.
