# Story 002: S11-TD-UI-ZINDEX-LAYERS -- Centralised UI Z-Index Layer Constants

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-ZINDEX-LAYERS
> **Status**: Draft (Sprint 14 candidate; NOT activated)
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive
> **Sprint**: Sprint 14 candidate (Tier 0 foundational; PROMPT 802 §4 rank 0.1;
> `docs/ux/ui-clean-pass-roadmap.md` rank 1). NOT activated by this authoring
> run. Sprint 13 disposition (`active`, `Polish` stage) preserved.
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Estimated effort**: ~1.0d (PROMPT 802 §4 Tier 0.1)

---

## Status / No-Claim Banner

This story is authored as a Sprint 14 candidate. **Sprint 14 is NOT activated
by this authoring run.** The story is paperwork only -- no code change is
attempted by PROMPT 878.

PROMPT 878 (this authoring run) does NOT:

- Activate Sprint 14.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

---

## Overview

PROMPT 802 §3.9 G1 surfaced that `client/src/ui/` contains **zero** uses of
`ZIndex` or `GlobalZIndex` across 226 `Node{}` / `Style{}` usages spanning 6
files. The only surface in the audit that declares an explicit z-index is
`client/src/presentation/result_screen.rs:512` (`GlobalZIndex(100)`), which
PROMPT 802 §3.8 also called out as the lone "acceptable" UI surface.

Every overlay, modal, drag ghost, toast, dim, and settlement panel in the
current client UI relies on spawn-order for paint order. That is fragile under
reconnect / snapshot rebuild / late-message recovery / replay: any code path
that respawns UI roots out of their initial order silently breaks the visual
stack. PROMPT 802 §5 sequencing rule names this work as the refactor that must
land **first** because every Tier 1 surface story (HUD strip layout, draft
centered modal, auction featured card, lobby class-picker, lobby layout modal,
hand drag-state visuals) depends on having a named layer to spawn into rather
than relying on inline `GlobalZIndex` literals or spawn-order.

This story introduces a named-layer enum + const module so all UI roots and
overlays declare their layer by name, eliminating magic z values across the
playable client.

---

## Scope

### In Scope

- A new design-token module (likely `client/src/ui/design_tokens/z_layers.rs`,
  exact path TBD by the worker) that exports a named `UiLayer` enum or const
  module covering at minimum the following layer order (lowest to highest):
  `Background`, `World`, `Units`, `UiBase`, `UiOverlay`, `Modal`, `Toast`,
  `Debug`. Each layer maps to a stable `GlobalZIndex` integer with sufficient
  gap (e.g. multiples of 100) to allow future intermediate layers without
  re-ordering existing values.
- A doc comment on each layer explaining the canonical UI elements expected
  there (e.g. `UiOverlay` = dim overlays, settlement scrim, draft-initial
  objective overlay; `Modal` = centered panels; `Toast` = transient
  notifications).
- Migration of all existing UI roots in `client/src/ui/` (lobby, HUD top-strip,
  HUD bottom-strip, hand cards, shop panel, draft offering panel, auction
  panel, shop footer, settlement overlay, toast) to declare their layer via
  the new module rather than relying on spawn-order.
- Migration of the existing explicit `GlobalZIndex(100)` in
  `client/src/presentation/result_screen.rs:512` to the new module's `Modal`
  (or equivalent) layer constant, preserving the existing visual stack.
- A grep guard or compile-time pattern (TBD by the worker) that prevents
  reintroduction of inline `ZIndex(N)` / `GlobalZIndex(N)` literals outside
  the design-token module.

### Out of Scope

- **No Sprint 14 activation** by this story. The story exists only as a
  candidate for future Sprint 14 pull-in.
- **No public release readiness** work. Z-layer constants land as a Polish-stage
  foundational primitive; they do not advance the `Polish->Release` gate-check
  retry.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion. Layer
  constants do not address hit-target sizes, keyboard navigation, screen
  reader support, colorblind modes, text scaling, or WCAG contrast ratios.
- **No final-art / asset-production** work (`PAW-TD-*-a`). Layer constants are
  layout / composition primitives; placeholder PNG replacement is a separate
  scope.
- **No playtest validation** (`QA-COND-0006`). Layer constants do not produce
  playtest evidence.
- **No re-design of the layer hierarchy itself**. The named layers listed
  above are the canonical set per PROMPT 802 §3.9 G1; intermediate layers can
  be added later without re-ordering, but the canonical set is fixed.
- **No spacing / typography / overlay-alpha** token work. Those are separate
  stories (002 = z-index only; 003 = font constants; 005 = viewport-invariant
  tests; 006 = overlay alpha; 007 = global UI design spec).
- **No interaction-state primitive** work (hover / focus / pressed / disabled).
  That is `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (Tier 0 Should-priority
  adjacent row, NOT in this story).
- **No `client/src/presentation/board_rendering.rs` sprite z-order** change.
  Board sprite z-order is governed by ADR-021 `PresentationPlugin` composition
  order, not by `bevy_ui` `GlobalZIndex`.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [ ] **AC1 -- Layer module authored**: GIVEN the story commit, WHEN the new
  design-token module is inspected, THEN it exports at least 8 named layer
  constants (Background, World, Units, UiBase, UiOverlay, Modal, Toast, Debug)
  each mapped to a stable `GlobalZIndex` integer with non-overlapping ranges.
  Verification: code review + unit test asserting each constant resolves to a
  distinct value and the ordering `Background < World < Units < UiBase <
  UiOverlay < Modal < Toast < Debug` holds.

- [ ] **AC2 -- Doc comments on each layer**: GIVEN the new module, WHEN
  inspected, THEN each layer constant carries a `///` doc comment naming the
  canonical UI elements expected at that layer. Verification: `cargo doc -p
  client` succeeds and the layer constants render with their doc text.

- [ ] **AC3 -- All UI roots migrated**: GIVEN the story commit, WHEN
  `client/src/ui/` is inspected, THEN every UI root spawn (lobby root, HUD top
  strip, HUD bottom strip, hand-card root, shop panel, draft offering panel,
  auction panel, settlement overlay, toast root) declares its layer via the
  new module rather than relying on spawn-order. Verification: code review +
  grep guard from AC5.

- [ ] **AC4 -- Result-screen migrated**: GIVEN the story commit, WHEN
  `client/src/presentation/result_screen.rs:512` is inspected, THEN the
  existing inline `GlobalZIndex(100)` is replaced with the new module's
  `Modal` layer constant (or equivalent), and the result screen still paints
  at the same effective layer relative to the rest of the UI. Verification:
  visual capture comparison against pre-migration baseline at
  `production/qa/evidence/captures/result-screen-baseline-*`.

- [ ] **AC5 -- Grep guard or lint**: GIVEN the story commit, WHEN
  `client/src/` is grepped (excluding the new design-token module itself),
  THEN no inline `ZIndex(N)` or `GlobalZIndex(N)` literals remain. Verification:
  `rg "ZIndex\(|GlobalZIndex\(" client/src/ --glob '!client/src/ui/design_tokens/**'`
  returns zero hits (exact glob TBD by worker).

- [ ] **AC6 -- Reconnect / snapshot-rebuild invariant**: GIVEN a two-client
  runtime harness reconnect scenario (or an equivalent ECS-level snapshot
  rebuild test), WHEN UI roots respawn out of their initial order, THEN the
  effective paint order matches the named-layer ordering rather than the
  spawn-order. Verification: integration test asserting the painted layer
  order under a synthesized out-of-order respawn.

- [ ] **AC7 -- No magic z values remain in `client/src/ui/`**: GIVEN the
  story commit, WHEN any `Node{}` style block under `client/src/ui/` is
  inspected, THEN any z-related field uses the new module's named constants
  exclusively. Verification: code review + AC5 grep guard.

- [ ] **AC8 -- ADR-021 alignment**: GIVEN the story commit, WHEN ADR-021
  (Presentation Layer Architecture) is read alongside the new module, THEN
  the named layers are consistent with the canonical `PresentationPlugin`
  composition order described in ADR-021. Verification: doc review;
  amendment to ADR-021 if reconciliation requires it.

- [ ] **AC9 -- Friend-game scope preserved**: GIVEN the story commit, WHEN
  `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a` accept-risk dispositions
  are inspected, THEN none of them has been flipped to `closed` by this
  story. Verification: `git diff` of `production/sprint-status.yaml` shows
  no accept-risk disposition change.

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/ui-zindex-layers/`

Expected artifacts:

- Integration test output for AC6 reconnect / snapshot-rebuild invariant.
- Visual capture comparison for AC4 (result-screen pre/post).
- Grep-guard output for AC5.
- Layer-ordering unit test output for AC1.

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/z_layers.rs` (NEW; exact path TBD by worker) | Author named-layer enum / const module. |
| `client/src/ui/design_tokens/mod.rs` (NEW or extended) | Re-export the layer constants. |
| `client/src/ui/mod.rs` | Declare `design_tokens` submodule. |
| `client/src/ui/lobby.rs` | Replace any spawn-order-implicit z with named layer. |
| `client/src/ui/hud/mod.rs` | Replace HUD-root z reliance with named layer. |
| `client/src/ui/hand/mod.rs` | Replace card-root z reliance with named layer. |
| `client/src/ui/shop_auction/mod.rs` | Replace panel-root z reliance with named layer (shop, draft offering, auction, settlement overlay, toast). |
| `client/src/presentation/result_screen.rs` | Migrate inline `GlobalZIndex(100)` to named `Modal` constant. |
| `tests/integration/ui_zindex_layers_test.rs` (NEW) | AC6 reconnect / respawn invariant test. |
| `tests/unit/ui_zindex_layers_test.rs` (NEW) | AC1 ordering / distinctness unit test. |
| `docs/architecture/adr-021-presentation-layer-architecture.md` | Possible amendment if reconciliation requires it (AC8). |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

---

## Verification

- `cargo test -p client --lib ui_zindex` (or equivalent) -- AC1 unit test.
- `cargo test -p client --test ui_zindex_layers_test` -- AC6 integration test.
- `cargo doc -p client` -- AC2 doc coverage.
- `rg "ZIndex\(|GlobalZIndex\(" client/src/ --glob '!client/src/ui/design_tokens/**'`
  -- AC5 grep guard.
- Visual capture against `production/qa/evidence/captures/result-screen-baseline-*`
  -- AC4 result-screen invariant.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt; it
  creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Tier 0 internal sequencing** (per PROMPT 802 §8 and roadmap §3
  "Sequencing Rules"): this story is **mostly serial** with stories 003
  (font constants), 004 (flex strips), and 006 (overlay alpha token) because
  all four touch the shared design-token host module (`client/src/ui/design_tokens/`).
  Story 005 (viewport-invariant tests) is parallel-safe with this story
  (new test bin).
- **Tier 0 design-spec dependency** (per roadmap §3 rule 2): story 007
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) should be authored *first* in Phase 1
  because Tier 0 token modules need its numeric values as input. However,
  for *z-layers specifically*, the layer ordering is structural rather than
  numeric, so this story can land slightly before the spec is finalized;
  the spec authoring should then ratify the chosen integer values.
- **Tier 1 surface stories depend on this**: PROMPT 802 §5 names z-index as
  the foundational refactor that must land first. Future Tier 1 stories
  (HUD top-strip, HUD bottom-strip, draft centered modal, auction featured
  card, lobby class-picker, lobby layout modal) all expect to spawn into a
  named layer.

---

## Notes

- PROMPT 802 §3.9 G1 source: zero `ZIndex` / `GlobalZIndex` hits in
  `client/src/ui/`; the sole exception is `result_screen.rs:512`
  (`GlobalZIndex(100)`).
- PROMPT 802 §3.2 H3 specifically calls out the HUD dim overlay as fragile
  under reconnect / snapshot rebuild / late-message recovery.
- The 8-layer canonical set comes from PROMPT 878's task brief and is
  consistent with PROMPT 802 §3.9 G1's "z-index layer system" recommendation.
- ADR-021 (Presentation Layer Architecture) governs the `PresentationPlugin`
  composition order; named UI layers must remain consistent with that ADR.
- Accept-risk preservation: `PAW-TD-*-a` (placeholder art), `QA-COND-0005`
  (Standard-tier accessibility, friend-game scope only), `QA-COND-0006`
  (playtest validation) remain preserved unchanged. This story does not
  advance any of them.
