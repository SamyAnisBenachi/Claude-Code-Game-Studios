# Story 006: S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 -- Single-Source Overlay Alpha Token

> **Epic**: UI Clean-Pass
> **Story ID**: S12-TD-UI-OVERLAY-ALPHA-TOKEN-001
> **Status**: Draft (Sprint 14 candidate; NOT activated)
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive (design token)
> **Sprint**: Sprint 14 candidate (Tier 0 foundational; PROMPT 802 §4 rank 0.5;
> `docs/ux/ui-clean-pass-roadmap.md` rank 5). NOT activated by this authoring
> run. Sprint 13 disposition (`active`, `Polish` stage) preserved.
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Estimated effort**: ~0.25d (PROMPT 802 §4 Tier 0.5)

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

PROMPT 802 §3.2 H4 and §3.9 G4 surfaced that the playable client uses
**three different overlay alpha values** for scrim / dim / backdrop
modals, with no shared token:

- HUD dim overlay: `0.45` (`client/src/ui/hud/mod.rs:33`)
- Settlement overlay: `0.58` (`client/src/ui/shop_auction/mod.rs:3539`)
- Result screen panel backdrop: `0.46` (`client/src/presentation/result_screen.rs:510`)

The three values were authored independently and have no shared
rationale. The visual effect is that switching between game states
(combat → settlement → result) flickers between three different
darkness levels, breaking visual continuity. PROMPT 802 §3.6 A6 also
called this out under the auction surface.

This story introduces a single canonical alpha-channel design token for
modal / overlay scrim, replacing the three (or more) scattered
`Color::rgba(_, _, _, 0.x)` literals with token reads.

---

## Scope

### In Scope

- A new overlay-token entry in the design-token module (likely
  `client/src/ui/design_tokens/colors.rs` or
  `client/src/ui/design_tokens/overlays.rs`; exact path TBD by the
  worker) that exports at least the following named overlay tokens:
  - `OVERLAY_DIM_ALPHA: f32` -- gameplay dim (HUD dim, focus dim).
  - `OVERLAY_SCRIM_ALPHA: f32` -- modal scrim (settlement, result
    backdrop).
  - Optionally `OVERLAY_TOAST_ALPHA: f32` if the toast root uses a
    scrim. Worker decides based on current toast styling.
- A doc comment on each token explaining the canonical surfaces that
  consume it (e.g. `OVERLAY_DIM_ALPHA` = HUD dim overlay during
  combat focus; `OVERLAY_SCRIM_ALPHA` = settlement scrim + result
  panel backdrop).
- A canonical scrim *color* constant (e.g. `OVERLAY_SCRIM_COLOR: Color`)
  if the scrim color RGB triple is also currently scattered. PROMPT 802
  did not enumerate scrim RGB triples beyond the alpha, so this is
  worker-discretion based on what `git grep` surfaces; the story does
  not strictly require an RGB token unless the worker finds duplication.
- Migration of `client/src/ui/hud/mod.rs:33` HUD dim alpha to read
  `OVERLAY_DIM_ALPHA`.
- Migration of `client/src/ui/shop_auction/mod.rs:3539` settlement
  overlay alpha to read `OVERLAY_SCRIM_ALPHA`.
- Migration of `client/src/presentation/result_screen.rs:510` result
  panel backdrop alpha to read `OVERLAY_SCRIM_ALPHA`.
- Audit of `client/src/` for any other `Color::rgba(_, _, _, 0.x)` /
  `Color::srgba(_, _, _, 0.x)` literals where `alpha < 1.0`, with each
  one classified as: (a) scrim/dim (migrate to a token); (b) ghost
  preview (board drag ghost -- left untouched, separate scope under
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`); (c) other (worker
  decides; document in worker report).
- A grep guard preventing reintroduction of inline alpha literals on
  scrim / dim surfaces.

### Out of Scope

- **No Sprint 14 activation** by this story.
- **No public release readiness** work.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion. The
  token does NOT address WCAG-compliant overlay contrast ratios or
  user-controllable overlay opacity. The chosen alpha value is sized
  for friend-game scope visual cohesion.
- **No final-art / asset-production** work (`PAW-TD-*-a`).
- **No playtest validation** (`QA-COND-0006`).
- **No tween / animation** of overlay alpha. Static alpha values only.
  Future per-state tweening (e.g. fade-in scrim on settlement enter)
  is a separate scope.
- **No board ghost preview** opacity change. That is
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` (Tier 2 future candidate,
  NOT this story).
- **No HUD timer urgency** color/alpha change. That is
  `S11-UX-HUD-TIMER-URGENCY-VISUAL-001` (Tier 2 future candidate, NOT
  this story).
- **No re-pick of the canonical scrim alpha value** by this story's
  author. The chosen alpha value (likely 0.5, or 0.46 to preserve the
  result-screen baseline, or another value ratified by story 007) is
  picked by the UX-designer + art-director via story 007's design
  spec. If story 007 has not landed, the worker proposes a default
  (suggested: 0.5 for `OVERLAY_SCRIM_ALPHA`, 0.45 for `OVERLAY_DIM_ALPHA`)
  and calls it out as ratify-on-spec.
- **No z-index, typography, flex-strip, viewport-invariant test** work.
  Those are stories 002 / 003 / 004 / 005.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [ ] **AC1 -- Overlay token authored**: GIVEN the story commit, WHEN
  the new design-token module is inspected, THEN it exports at least
  two named overlay alpha constants (`OVERLAY_DIM_ALPHA` and
  `OVERLAY_SCRIM_ALPHA`) each with a doc comment naming canonical
  consumers. Verification: code review.

- [ ] **AC2 -- HUD dim migrated**: GIVEN the story commit, WHEN
  `client/src/ui/hud/mod.rs` is inspected, THEN the dim overlay alpha
  literal (previously `0.45`) is replaced with a read of
  `OVERLAY_DIM_ALPHA`. Verification: code review + visual capture
  comparing HUD dim pre/post-migration.

- [ ] **AC3 -- Settlement scrim migrated**: GIVEN the story commit,
  WHEN `client/src/ui/shop_auction/mod.rs` is inspected, THEN the
  settlement overlay alpha literal (previously `0.58`) is replaced
  with a read of `OVERLAY_SCRIM_ALPHA`. Verification: code review +
  visual capture.

- [ ] **AC4 -- Result panel backdrop migrated**: GIVEN the story
  commit, WHEN `client/src/presentation/result_screen.rs` is
  inspected, THEN the result panel backdrop alpha literal (previously
  `0.46`) is replaced with a read of `OVERLAY_SCRIM_ALPHA`.
  Verification: code review + visual capture.

- [ ] **AC5 -- Grep guard**: GIVEN the story commit, WHEN
  `client/src/` is grepped (excluding the design-token module and
  documented exclusions for board ghost preview / HUD timer urgency),
  THEN no inline `Color::rgba(_, _, _, 0.x)` or `Color::srgba(_, _,
  _, 0.x)` literal with `alpha < 1.0` remains on a scrim/dim surface
  call site. Verification: `rg "Color::(s)?rgba\(.*,\s*0\.[0-9]" client/src/
  --glob '!client/src/ui/design_tokens/**'` returns zero hits or only
  hits on documented exclusion sites.

- [ ] **AC6 -- Documented exclusions enumerated**: GIVEN the story
  commit, WHEN the worker report is inspected, THEN every remaining
  inline `alpha < 1.0` literal in `client/src/` is documented as
  either (a) scrim/dim that should have been migrated (treat as
  bug, fix in this story), (b) board ghost preview (left for
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`), or (c) other (named with
  rationale). Verification: worker report enumerates.

- [ ] **AC7 -- Single visual cohesion across game states**: GIVEN
  the migration, WHEN the playable client transitions from
  combat → settlement → result, THEN the scrim alpha is visually
  consistent across all three states. Verification: visual capture
  sequence stored under evidence path; manual eyeball.

- [ ] **AC8 -- Friend-game scope preserved**: GIVEN the story commit,
  WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a` accept-risk
  dispositions are inspected, THEN none of them has been flipped to
  `closed` by this story. Verification: `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  change.

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/ui-overlay-alpha-token/`

Expected artifacts:

- HUD dim pre/post visual capture for AC2.
- Settlement scrim pre/post visual capture for AC3.
- Result panel backdrop pre/post visual capture for AC4.
- Combat → settlement → result transition capture for AC7.
- Grep-guard output for AC5.
- Worker report enumerating any remaining inline alpha literals for AC6.

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/overlays.rs` (NEW; exact path TBD by worker) | Author overlay alpha tokens (`OVERLAY_DIM_ALPHA`, `OVERLAY_SCRIM_ALPHA`). |
| `client/src/ui/design_tokens/mod.rs` (NEW or extended) | Re-export overlay tokens. |
| `client/src/ui/mod.rs` | Declare `design_tokens` submodule (if not already from stories 002 / 003 / 004). |
| `client/src/ui/hud/mod.rs` | Migrate HUD dim alpha read (line ~33). |
| `client/src/ui/shop_auction/mod.rs` | Migrate settlement scrim alpha read (line ~3539). |
| `client/src/presentation/result_screen.rs` | Migrate result panel backdrop alpha read (line ~510). |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

---

## Verification

- `cargo test -p client --lib design_tokens` -- token module unit test
  (presence + positive value range 0.0 < alpha < 1.0).
- `rg "Color::(s)?rgba\(.*,\s*0\.[0-9]" client/src/ --glob '!client/src/ui/design_tokens/**'`
  -- AC5 grep guard.
- Visual capture sequence (combat → settlement → result) at 1920×1080
  -- AC7 cohesion verification.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt;
  it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Tier 0 internal sequencing** (per PROMPT 802 §8 and roadmap §3
  "Sequencing Rules"): this story is **mostly serial** with stories 002
  (z-index layers), 003 (font constants), and 004 (flex strips) because
  all four touch the shared design-token host module
  (`client/src/ui/design_tokens/`). Story 005 (viewport-invariant tests)
  is parallel-safe. Effort is the smallest in Tier 0 (~0.25d) so this
  story can be slotted opportunistically once one of stories 002 / 003 /
  004 has created the host module.
- **Tier 0 design-spec dependency** (per roadmap §3 rule 2): story 007
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) should be authored *first* so
  the canonical alpha value (0.45 vs 0.5 vs 0.46) is ratified by UX +
  art before this story lands. If story 007 has not landed, the worker
  proposes a default and calls it out as ratify-on-spec.
- **Tier 1 surface stories depend on this**: any Tier 1 story that
  spawns a new modal / scrim / dim overlay is expected to read from the
  token rather than re-declare an alpha literal.

---

## Notes

- PROMPT 802 §3.2 H4: HUD dim `0.45` (`hud/mod.rs:33`), settlement
  `0.58` (`shop_auction/mod.rs:3539`), result-screen backdrop `0.46`
  (`result_screen.rs:510`).
- PROMPT 802 §3.6 A6: settlement overlay backdrop alpha `0.58` ≠ HUD
  dim `0.45` ≠ result-screen backdrop `0.46` -- folds into H4.
- PROMPT 802 §3.9 G4: no shared overlay-alpha token.
- The default proposed alpha values (`OVERLAY_DIM_ALPHA = 0.45`,
  `OVERLAY_SCRIM_ALPHA = 0.5`) come from PROMPT 878's task brief and
  PROMPT 802's range of values. Story 007 (global UI design spec)
  ratifies the final values.
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved unchanged. This story does not advance any of them.
