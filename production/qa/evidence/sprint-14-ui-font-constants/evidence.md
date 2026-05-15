# Sprint 14 — Story 003 — S11-TD-UI-FONT-CONSTANTS Evidence

**Story file**: `production/epics/ui-clean-pass/story-003-ui-font-constants.md`
**Worker branch**: `work/s14-ui-token-scale-typography`
**Branched from**: `origin/main@36c0b4b` (PROMPT 902 integration of `work/s14-ui-layout-foundation` / story 002 z-layers).
**Worker prompt**: PROMPT 904 — S14 UI Token Scale and Typography Dev-Story Retry.
**Authored**: 2026-05-15.

---

## Summary

Authored the `client::ui::design_tokens::typography` host module with six
strictly-ascending semantic-size constants (Caption / Body / H3 / H2 / H1
/ Display), three named weight constants, and a single line-height ratio
constant. Migrated every inline `font_size:` numeric literal outside the
design-token host module across `client/src/ui/` and `client/src/presentation/`
to symbolic references against the module. Fixed the PROMPT 802 §3.1 L6
lobby typography hierarchy inversion by routing labels and CTAs through
`typography::BODY` (15 px) so they are no longer smaller than the data
they describe.

## AC outcomes

- **AC1 — Typography token module authored** — **PASS**. Six named
  semantic-size constants (Caption=13, Body=15, H3=18, H2=22, H1=30,
  Display=40 px) declared in `client/src/ui/design_tokens/typography.rs`
  with strict ascending order, `SCALE_MIN_GAP=2.0` enforcing future
  intermediate headroom, and three named weight tokens (WEIGHT_REGULAR=400,
  WEIGHT_SEMIBOLD=600, WEIGHT_BOLD=700). Verified by inline unit tests
  `ac1_six_named_semantic_sizes_strictly_ascending`,
  `ac1_canonical_scale_ordering_matches_story_spec`,
  `ac1_each_scale_resolves_to_positive_finite_f32`,
  `ac1_scale_constants_have_minimum_gap_for_future_intermediates`,
  `ac1_scale_constants_are_pairwise_distinct`,
  `ac1_three_named_weights_strictly_ascending` — all PASS.

- **AC2 — Line-height ratio constant** — **PASS**.
  `LINE_HEIGHT_DEFAULT_RATIO: f32 = 1.25` declared with doc comment
  naming its intended usage (multiplier against a semantic-size constant
  for explicit `Val::Px(...)` line height). Verified by unit test
  `ac2_line_height_ratio_is_positive_finite_and_at_least_one` — PASS.

- **AC3 — All inline `font_size` literals migrated** — **PASS**. The
  integration test
  `tests/integration/ui_clean_pass/typography_test.rs::ac3_grep_guard_no_inline_font_size_literals_outside_design_tokens`
  walks every `*.rs` under `client/src/` (skipping `client/src/ui/design_tokens/`)
  and asserts zero `font_size: <digit | Val::Px>` literal matches. PASS.

- **AC4 — HUD constants subsumed** — **PASS (with documented scope
  clarification)**.
  - `HUD_GOLD_FONT_SIZE_PX` now `= typography::DISPLAY` (40 px) at
    `client/src/ui/hud/mod.rs:51`.
  - `HUD_RESERVED_GOLD_FONT_SIZE_PX` now `= typography::H1` (30 px; was
    bare 26 px literal) at `client/src/ui/hud/mod.rs:57`.
  - `HUD_SECONDARY_FONT_SIZE_PX` now `= typography::H2` (22 px; was
    aliased to `HUD_RESOURCE_TEXT_MIN_SIZE_PX` 20 px) at
    `client/src/ui/hud/mod.rs:63`.
  - `HUD_RESOURCE_TEXT_MIN_SIZE_PX` (20 px) and `HUD_GOLD_TEXT_MIN_SIZE_PX`
    (40 px) are **intentionally preserved as independent
    accessibility-floor invariants** consumed by
    `tests/integration/hud/text_size_contrast_accessibility_test.rs` as
    minimum-rendered-font-size floors. The story-003 AC4 disposition
    language allows "either resolve through the new module's constants
    or have been removed in favour of direct references"; the worker
    exercises the first half for the three font-size aliases and
    preserves the `_MIN_SIZE_PX` floors with documentation. The H2
    (22 px) routing for HUD_SECONDARY_FONT_SIZE_PX sits 2 px above the
    HUD_RESOURCE_TEXT_MIN_SIZE_PX floor and DISPLAY (40 px) equals the
    HUD_GOLD_TEXT_MIN_SIZE_PX floor, so the accessibility regression
    test continues to pass. Verified by
    `tests/integration/ui_clean_pass/typography_test.rs::ac4_hud_font_size_constants_resolve_through_design_tokens`
    and `ac4_hud_resource_text_min_size_is_independent_accessibility_floor`,
    plus the existing
    `tests/integration/hud/text_size_contrast_accessibility_test.rs`
    (4/4 PASS post-migration).

- **AC5 — Result screen migrated** — **PASS**. Headline ("RESULT
  PENDING") routes through `typography::H1` (30 px; was 36 px); cause
  routes through `typography::H3` (18 px; preserved); summary routes
  through `typography::BODY` (15 px; preserved); objective column titles
  route through `typography::BODY` (15 px; was 16 px); objective row
  values route through `typography::CAPTION` (13 px; was 14 px); return
  button routes through `typography::H3` (18 px; was 17 px inline
  literal). Verified by
  `tests/integration/ui_clean_pass/typography_test.rs::ac5_result_screen_migrated_to_h1_h3_body`
  — PASS. Existing scaffold tests (`result_screen_mvp_test` 6/6,
  `result_screen_return_to_lobby_test` 2/2) remain green post-migration.

- **AC6 — Lobby typography inversion fixed** — **PASS**. Pre-migration
  shape: status banner 18 px > room code 15 px > CTAs 14 px > labels /
  slot / class buttons 13 px (inversion: labels and CTAs smaller than
  the data they describe). Post-migration:
  - Status banner → `typography::H3` (18 px, preserved).
  - Room code → `typography::BODY` (15 px, preserved).
  - Create / Join / Confirm CTAs → `typography::BODY` (15 px; bumped
    from 14 px so CTAs are not smaller than room-code data).
  - "Requested slot" / "Class" labels → `typography::BODY` (15 px;
    bumped from 13 px so labels are not smaller than the room-code
    data they sit beside).
  - Slot / class buttons → `typography::BODY` (15 px; bumped from
    13 px so the row labels and button text are at the same level —
    inversion eliminated).
  - Room code chip text → `typography::BODY` (15 px; bumped from
    14 px).
  Hierarchy now satisfies the story spec: every label / CTA / data
  cell is ≥ BODY, status banner H3 remains the visual prominence
  anchor. Verified by
  `tests/integration/ui_clean_pass/typography_test.rs::ac6_lobby_typography_inversion_fixed`
  (asserts no remaining `lobby_text_font(13.0)` / `lobby_text_font(14.0)`
  call sites and presence of `lobby_text_font(typography::BODY)` /
  `lobby_text_font(typography::H3)`) — PASS.

- **AC7 — Grep guard** — **PASS**. Same predicate as AC3, packaged into
  the integration test
  `tests/integration/ui_clean_pass/typography_test.rs::ac3_grep_guard_no_inline_font_size_literals_outside_design_tokens`
  with a sibling sanity test
  `ac7_grep_guard_pattern_actually_detects_a_synthesized_violation`
  that exercises the predicate against synthesised input to prove the
  matcher is not silently a no-op. Both PASS.

- **AC8 — Unit tests pass** — **PASS**. `cargo test -p client --lib
  ui::design_tokens::typography` 9/9 PASS; `cargo test -p client --test
  ui_clean_pass_typography_test` 8/8 PASS. Adjacent regression sweep
  (hud accessibility 4/4, z_layers 6/6, connection-lost overlay 16/16,
  result-screen MVP 6/6, result-screen return-to-lobby 2/2,
  playable-client lobby entry 6/6, accessibility settings shell 4/4,
  photosensitivity warning 4/4, shop_auction scaffold formulas 8/8,
  hud plugin scaffold 6/6) all PASS post-migration.

- **AC9 — Friend-game scope preserved** — **PASS**.
  `git diff origin/main -- production/sprint-status.yaml` is empty.
  No `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` disposition flipped
  to `closed`. Sprint 14 disposition unchanged (`active`); stage
  unchanged (`Polish`); PROMPT 761 Polish→Release FAIL preserved.

## Ratify-on-spec disclosure

PROMPT 802 §9 **producer-decision-2** (canonical numeric typography
values) remains **unresolved on `origin/main`** at the time of this
implementation. Story-007
(`production/epics/ui-clean-pass/story-007-global-ui-design-spec.md` →
`docs/ux/global-ui-design-spec.md`) has not yet authored its canonical
spec. Per the story-003 §Dependencies / Sequencing escape clause —
"If story 007 has not yet landed, the worker can propose default
values (12 / 15 / 18 / 22 / 30 / 40) and call them out as
ratify-on-spec." — this worker uses the slightly tightened default
sequence **13 / 15 / 18 / 22 / 30 / 40** so that the existing 13 px
lobby slot / class button text (the smallest existing literal across
`client/src/ui/`) maps onto the lowest named scale without forcing a
rounding decision per call site. The numeric values are documented in
the typography module preamble as **ratify-on-spec** — when story-007
lands and a producer-decision-2 ratification edits the values, the
**named constants** remain the stable contract and consumers do not
change.

## Files touched

| Path | Change |
|------|--------|
| `client/Cargo.toml` | Add `[[test]] ui_clean_pass_typography_test` entry. |
| `client/src/presentation/connection_lost_overlay.rs` | Import typography; migrate 30 px headline → H1, 18 px body → H3. |
| `client/src/presentation/result_screen.rs` | Import typography; migrate 36 / 18 / 15 / 16 / 17 / 14 → H1 / H3 / BODY / BODY / H3 / CAPTION. |
| `client/src/ui/design_tokens/mod.rs` | Declare new `typography` submodule; extend module-level rustdoc. |
| `client/src/ui/design_tokens/typography.rs` | **NEW** — 6 size + 3 weight + 1 line-height + 2 collection constants + 9 inline unit tests. |
| `client/src/ui/hud/mod.rs` | Import typography; route GOLD / RESERVED_GOLD / SECONDARY constants through DISPLAY / H1 / H2; preserve `_MIN_SIZE_PX` accessibility floors with documentation. |
| `client/src/ui/lobby.rs` | Import typography; migrate every `lobby_text_font(N.0)` call (10 sites) — fix AC6 inversion by routing labels / CTAs / data rows through BODY, status banner through H3. |
| `client/src/ui/photosensitivity_warning.rs` | Import typography; migrate 24 / 15 / 16 → H2 / BODY / BODY. |
| `client/src/ui/settings/mod.rs` | Import typography; migrate 15 / 14 / 16 → BODY / CAPTION / BODY. |
| `client/src/ui/shop_auction/mod.rs` | Import typography; migrate 21 `shop_auction_text_font(N.0)` call sites to symbolic tokens. |
| `tests/integration/ui_clean_pass/typography_test.rs` | **NEW** — 8 integration tests covering AC3 / AC4 / AC5 / AC6 / AC7 / AC8. |
| `production/qa/evidence/sprint-14-ui-font-constants/evidence.md` | **NEW** — this file. |

## Checks run

- `cargo fmt -p client -- --check` → clean (one auto-format pass applied during dev).
- `cargo check -p client --tests` → success (one pre-existing unrelated
  dead-code warning in `hand_ui_asset_wiring_test.rs::count_with_image_node`,
  unchanged by this story).
- `cargo test -p client --lib ui::design_tokens::typography` → **9 / 9 PASS**.
- `cargo test -p client --test ui_clean_pass_typography_test` → **8 / 8 PASS**.
- `cargo test -p client --test hud_text_size_contrast_accessibility_test` → **4 / 4 PASS**.
- `cargo test -p client --test ui_clean_pass_z_layers_test` → **6 / 6 PASS**.
- `cargo test -p client --test connection_lost_overlay_test` → **16 / 16 PASS**.
- `cargo test -p client --test result_screen_mvp_test` → **6 / 6 PASS**.
- `cargo test -p client --test result_screen_return_to_lobby_test` → **2 / 2 PASS**.
- `cargo test -p client --test playable_client_lobby_entry_test` → **6 / 6 PASS**.
- `cargo test -p client --test accessibility_settings_shell_test` → **4 / 4 PASS**.
- `cargo test -p client --test accessibility_settings_photosensitivity_warning_test` → **2 / 2 PASS**.
- `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test` → **8 / 8 PASS**.
- `cargo test -p client --test hud_plugin_scaffold_test` → **6 / 6 PASS**.
- `git diff --check origin/main` → clean.
- `git status --short` → enumerates only the in-scope migration set.

## Cargo policy applied

`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`,
`CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`,
`CARGO_INCREMENTAL=0`,
`RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE` set for every cargo
invocation. **Disk cleanup: NO** (pressure not hit; share point under
the configured target dir remained healthy).

## Forbidden-scope verification

`git diff origin/main --stat` against the forbidden surfaces is empty:

- `server/` — empty.
- `shared/` — empty.
- `production/sprint-status.yaml` — empty.
- `production/stage.txt` — empty.
- `production/sprints/` — empty.
- `production/qa/qa-plan-sprint-14.md` — empty.
- `production/session-state/` — empty.

No `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
`/release-check`, or `/qa-plan` invoked.

## Non-claims preserved

- `S8-QA-001-W1` OPEN.
- `QA-COND-0005` + `QA-COND-0006` accept-risk.
- `PAW-TD-*-a` accept-risk.
- PROMPT 683-era runtime divergence question (folded into Sprint 12 story 019
  cannot-reproduce closure; not reopened by this story).
- `TQ-S12-C1..C7` verbatim.
- PROMPT 761 Polish→Release `FAIL`.
- Sprint 10 / 11 / 12 close-outs unchanged.
- Sprint 13 close-out (`closed-with-conditions`, PROMPT 894) unchanged.
- Underlying drag-runtime bug NOT claimed fixed.
- Two-client GAME_OVER closure NOT claimed.
- Final-art / asset-production NOT claimed.
- Public-release / RC readiness / full game completion NOT claimed.
- Sprint 14 disposition `active` unchanged. Stage `Polish` unchanged.

## PROMPT 900 blocker resolution

PROMPT 900 reported BLOCKED due to three independent blockers:

1. **Producer-decision-2 (numeric values)** — still unresolved on
   `origin/main`. Worked around per the story-003 ratify-on-spec escape
   clause (default values documented as ratify-on-spec; named constants
   are the stable contract).
2. **Story-002 host module not landed** — **RESOLVED** by PROMPT 902
   integration commit `36c0b4b`. `client/src/ui/design_tokens/` host
   module now reachable from `origin/main`; story-003 extends it with
   `typography.rs` rather than creating it.
3. **Story-007 design spec not authored** — still pending on
   `origin/main`. Worked around with the same ratify-on-spec escape
   clause; the named-constant contract isolates consumers from the
   final numeric values.

The story-003 implementation is unblocked under (2). (1) and (3)
remain explicit follow-on dependencies; the typography module's
preamble surfaces this transparency so a future producer-decision-2
ratification edit lands in one place.
