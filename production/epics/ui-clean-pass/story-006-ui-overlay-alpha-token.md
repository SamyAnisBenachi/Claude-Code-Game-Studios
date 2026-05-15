# Story 006: S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 -- Single-Source Overlay Alpha Token

> **Epic**: UI Clean-Pass
> **Story ID**: S12-TD-UI-OVERLAY-ALPHA-TOKEN-001
> **Status**: Done (PROMPT 921 /story-done closure on origin/main@c4e1936)
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

- [x] **AC1 -- Overlay token authored** -- **PASS** (PROMPT 921 verified
  on origin/main@c4e1936): `client/src/ui/design_tokens/overlays.rs`
  NEW 273 lines exports `OVERLAY_DIM_ALPHA = 0.45` (:90),
  `OVERLAY_SCRIM_ALPHA = 0.55` (:116), `OVERLAY_TOAST_ALPHA = 0.80`
  (:131), each with `///` doc comment naming canonical consumers
  (HUD dim / settlement+result scrim / toast root). Inline 9-test
  unit suite asserts range / strict-ascending / pairwise-distinct /
  spec-ratified-value / dim<scrim<toast hierarchy invariants.
  Integration test `ac1_overlay_token_module_exports_required_token_set`
  asserts published surface.

- [x] **AC2 -- HUD dim migrated** -- **PASS** (PROMPT 921 verified on
  origin/main@c4e1936): `client/src/ui/hud/mod.rs:40` exports
  `pub const HUD_DIM_OVERLAY_ALPHA: f32 = overlays::OVERLAY_DIM_ALPHA`
  routing the grep-stable consumer name through the canonical token;
  call site at `:714` reads `HUD_DIM_OVERLAY_ALPHA` via
  `Color::srgba(0.0, 0.0, 0.0, HUD_DIM_OVERLAY_ALPHA)`. The previous
  inline `0.45` literal at the spawn site is gone. Integration test
  `ac2_hud_dim_overlay_alpha_routes_through_overlays_token` enforces
  the routing. Visual capture deferred to follow-on QA-tester
  deliverable under `/team-qa` (NOT in PROMPT 921 paperwork scope).

- [x] **AC3 -- Settlement scrim migrated** -- **PASS** (PROMPT 921
  verified on origin/main@c4e1936):
  `client/src/ui/shop_auction/mod.rs:~3550-3556` settlement overlay
  `BackgroundColor` reads `Color::srgba(0.02, 0.05, 0.08,
  overlays::OVERLAY_SCRIM_ALPHA)` (rustfmt wrapped the call to
  multi-line). Previous `0.58` literal at the spawn site is gone.
  Integration test `ac3_settlement_overlay_reads_canonical_scrim_alpha`
  uses whitespace-normalized matching for rustfmt-robustness. Visual
  capture deferred to follow-on QA-tester `/team-qa` deliverable.

- [x] **AC4 -- Result panel backdrop migrated** -- **PASS** (PROMPT 921
  verified on origin/main@c4e1936):
  `client/src/presentation/result_screen.rs:~518-523` result panel
  root `BackgroundColor` reads `Color::srgba(0.02, 0.025, 0.035,
  overlays::OVERLAY_SCRIM_ALPHA)` (rustfmt wrapped to multi-line).
  Previous `0.46` literal at the spawn site is gone. Integration test
  `ac4_result_screen_backdrop_reads_canonical_scrim_alpha` asserts
  the routing. Visual capture deferred to follow-on QA-tester
  `/team-qa` deliverable.

- [x] **AC5 -- Grep guard** -- **PASS** (PROMPT 921 verified on
  origin/main@c4e1936): Integration test
  `ac5_grep_guard_no_pre_migration_scrim_literals_outside_design_tokens`
  walks every `*.rs` under `client/src/` (excluding
  `client/src/ui/design_tokens/`) in whitespace-normalized form and
  asserts the three pre-migration scrim/dim literal triplets
  (`0.45` / `0.58` / `0.46` on scrim/dim spawn surfaces) are gone.
  Sanity-check
  `ac5_grep_guard_pattern_actually_detects_a_synthesized_violation`
  proves the matcher actually fires on a constructed violation.

- [x] **AC6 -- Documented exclusions enumerated** -- **PASS**
  (PROMPT 921 verified on origin/main@c4e1936):
  `production/qa/evidence/sprint-14-overlay-alpha-token/evidence.md`
  §AC6 enumeration table classifies every remaining
  `Color::(s)?rgba` literal with `alpha < 1.0` in `client/src/` as
  one of: (a) scrim/dim migrated (3 sites: HUD dim, settlement,
  result backdrop), (b) preserved-intentional / separate scope
  (9 sites incl. connection-lost `0.32`, HUD timer urgency, board
  ghost preview), (c) other-with-rationale (~30 sites: button states,
  text colors, panel chrome, accessibility warning, settings shell).
  Integration test
  `ac6_connection_lost_overlay_literal_preserved_with_canonical_token_doc_reference`
  asserts the connection-lost `0.32` literal at `:214` survives and
  the surrounding comment at `:206` references `OVERLAY_SCRIM_ALPHA`
  symbolically.

- [x] **AC7 -- Single visual cohesion across game states** -- **PASS
  (token-level)** (PROMPT 921 verified on origin/main@c4e1936):
  Inline + integration tests assert
  `OVERLAY_DIM_ALPHA < OVERLAY_SCRIM_ALPHA < OVERLAY_TOAST_ALPHA`.
  Settlement and result panel now both consume
  `OVERLAY_SCRIM_ALPHA = 0.55` -- eliminating the inter-state
  flicker PROMPT 802 §3.2 H4 surfaced. Migration deltas vs
  pre-state: HUD dim no change (`0.45 -> 0.45`); settlement
  Δ=-0.03 lighter (`0.58 -> 0.55`); result Δ=+0.09 darker
  (`0.46 -> 0.55`) -- both within the spec §6 ≤ 0.1 alpha-step
  cohesion budget. Visual capture sequence at 1920×1080 (combat →
  settlement → result) is a follow-on QA-tester deliverable under
  `/team-qa` (OUT OF SCOPE for PROMPT 921 paperwork closure;
  AC7 verdict here is token-level / invariant-driven).

- [x] **AC8 -- Friend-game scope preserved** -- **PASS** (PROMPT 921
  verified on origin/main@c4e1936):
  `git diff c4e1936^1..c4e1936 --stat -- production/sprint-status.yaml`
  returns empty across the integration commit (worker + integration
  both honoured the forbidden-paths discipline).
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` accept-risk
  dispositions remain unchanged. Module-level scope-discipline
  doc comments at `client/src/ui/design_tokens/overlays.rs:56-72`
  and evidence document §Carried non-claims preserve these
  references verbatim. PROMPT 921 row-level flip is the permitted
  disposition-preserving paperwork edit and does NOT touch any
  accept-risk field.

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

---

## Closure Trail

- **PROMPT 878** -- story authoring (2026-05-14). Story file created
  as Sprint 14 candidate, NOT activated. No code change.
- **PROMPT 893** -- integration of authoring run into `origin/main`
  (merge `9f36663` per sprint-status.yaml notes).
- **PROMPT 897** -- Sprint 14 activation snapshot.
- **PROMPT 898** -- Sprint 14 QA-plan authoring at
  `production/qa/qa-plan-sprint-14.md`.
- **PROMPT 911** -- global UI design spec authoring
  (`docs/ux/global-ui-design-spec.md` §6 ratifies overlay alpha
  values `0.45` / `0.55` / `0.80`).
- **PROMPT 912** -- spec integration to `origin/main@3d99a04`.
- **PROMPT 916** -- /dev-story worker on
  `work/s14-overlay-alpha-token@837a611` from base
  `origin/main@3d99a04`. 9 files / +931 / -9. Authored
  `client/src/ui/design_tokens/overlays.rs` (273 lines) with three
  named overlay tokens + inline 9-test invariant suite; migrated
  HUD dim (`hud/mod.rs:40` + `:714`), settlement scrim
  (`shop_auction/mod.rs:~3554`), result panel backdrop
  (`result_screen.rs:~522`); registered
  `[[test]] ui_clean_pass_overlay_alpha_test` integration bin
  (8 tests); authored `production/qa/evidence/sprint-14-overlay-alpha-token/evidence.md`
  (324 lines). Worker report at
  `reports/PROMPT-916-S14-Overlay-Alpha-Token-Dev-Story.md`.
- **PROMPT 917** -- integration `--no-ff` merge of worker tip
  `837a611` into prior `origin/main@aa772a8` (PROMPT 919 tip).
  Three text-mechanical conflicts resolved (`client/Cargo.toml` +
  `design_tokens/mod.rs` + `hud/mod.rs` use-statement
  collapse), preserving both branches' additions. Merge commit
  `c4e1936bff32b5b1dd8b9b92bf69e04b1d191af3`; fast-forward push to
  `origin/main`. Integration report at
  `reports/PROMPT-917-S14-Overlay-Alpha-Token-Integration.md`.
- **PROMPT 921 (this row)** -- paperwork-only `/story-done` closure.
  Row flipped `ready -> done` with `completed: 2026-05-15` on the
  basis of AC1-AC8 verification against the integrated diff at
  `origin/main@c4e1936`. AC1-AC8 all PASS (AC7 PASS-token-level
  with visual capture deferred to follow-on `/team-qa` per worker
  scope). Sprint 14 disposition UNCHANGED `active`. Stage
  UNCHANGED `Polish`. PROMPT 761 Polish->Release `FAIL` preserved.

### Conditions carried forward unchanged

- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` accepted-risk (Standard-tier accessibility not
  claimed); overlay alpha tokens are not WCAG-compliant contrast
  conformance.
- `QA-COND-0006` accepted-risk (playtest validation deferred).
- `PAW-TD-*-a` accept-risk preserved (final-art / asset-production
  not claimed).
- `PROMPT 683`-era runtime divergence question preserved (folded
  into Sprint 12 story 019 cannot-reproduce closure; third
  same-scope retest NOT authorised per `TQ-S12-C2`).
- `TQ-S12-C1..C7` preserved verbatim; `TQ-S12-C7` NOT closed.
- Sprint 13 / 12 / 11 / 10 closeouts preserved unchanged.
- All prior Sprint 13 `/story-done` closures preserved unchanged.
- All prior Sprint 14 `/story-done` closures preserved verbatim
  (PROMPT 909 viewport-invariant-tests first + PROMPT 908
  font-constants second + PROMPT 903 z-index-layers third
  reconcile + PROMPT 919 flex-strips fourth); PROMPT 921 appended
  as fifth `sprint_14_story_done` block.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 14 Should Have carry
  preserved (status `ready`, human-operator-blocked, no LLM
  `/story-done` authorised).
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row
  preserved as-is.
- Sprint 14 activation snapshot under `sprint_14_activation:`
  block (PROMPT 897) preserved unchanged (per-row
  `status_at_activation` values are activation-time snapshots,
  NOT live status -- live row status moves under `stories:`).

### Explicitly NOT claimed by PROMPT 921

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from `Polish` to `Release`
- Sprint 14 close-out (Sprint 14 remains `active`; 5 of 17 rows
  closed after PROMPT 921)
- closure of any other Sprint 14 row
- Tier 0 rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001` story 007)
  `/story-done` closure -- separate downstream prompt
- visual capture sequence at 1920×1080 across combat / settlement /
  result (follow-on QA-tester deliverable under `/team-qa`)
- full-workspace `cargo test --workspace --tests --no-fail-fast`
  result claim (PROMPT 921 paperwork-only; PROMPT 916 + 917 ran
  narrow targeted scope per qa-plan-sprint-14
  no-full-workspace-tests-by-default policy)

### Downstream unblock

Tier 1 surface stories that consume a modal scrim (HUD top strip
015 / auction featured card 016 / lobby layout modal 024) now have
a single canonical `OVERLAY_SCRIM_ALPHA` to read rather than picking
their own alpha literal. The overlay alpha token module is the
fifth Tier 0 foundation primitive landed in Sprint 14 (after
z-index layers, font constants, viewport-invariant tests, flex
strips). Tier 0 rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001` story
007) is on `origin/main` via PROMPT 912 integration `3d99a04`
ancestor of `c4e1936` -- `/story-done` for that row is a separate
downstream prompt.
