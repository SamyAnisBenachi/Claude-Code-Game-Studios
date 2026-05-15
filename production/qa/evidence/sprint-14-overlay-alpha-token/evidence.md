# Sprint 14 / Story 006 — S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 — Evidence

**Story**: `production/epics/ui-clean-pass/story-006-ui-overlay-alpha-token.md`
**Spec**: `docs/ux/global-ui-design-spec.md` §6 (PROMPT 911 ratified;
PROMPT 912 integration `3d99a04`)
**Worker run**: PROMPT 916 `/dev-story`
**Source-of-truth at worker base**:
`origin/main@3d99a0482d24ce89230159ac3565f6e823b97c04`
**Worktree**: `D:/_DEV/wt/ccgs-prompt-916-overlay-alpha-token`
**Branch**: `work/s14-overlay-alpha-token`

This evidence document mirrors the structure of
`production/qa/evidence/sprint-14-ui-foundation/ui-zindex-layers/` and
`production/qa/evidence/sprint-14-ui-typography/`. AC1 / AC2 / AC3 / AC4 /
AC5 / AC7 are covered by code + automated tests; AC6 (documented
exclusions) is enumerated in this file; AC8 (accept-risk preservation) is
covered by the `git diff` checks at commit time. No optimistic
client-side authority is introduced. PAW-TD-*-a / QA-COND-0005 /
QA-COND-0006 accept-risk dispositions are unchanged. PROMPT 761
Polish→Release `FAIL` preserved. S8-QA-001-W1 OPEN.

---

## Cargo resource policy

Worker session set the binding Windows/MSVC Cargo resource policy before
every `cargo` invocation per Sprint 14 QA-plan §Worker setup:

```text
CARGO_TARGET_DIR        = D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG = 0
CARGO_PROFILE_TEST_DEBUG= 0
CARGO_INCREMENTAL       = 0
RUSTFLAGS               = -C debuginfo=0 -C link-arg=/DEBUG:NONE
```

---

## Files touched

| Path | Status | Change |
|------|--------|--------|
| `client/src/ui/design_tokens/overlays.rs` | NEW | Author the three `OVERLAY_*_ALPHA` tokens + `ALL_OVERLAY_ALPHAS_ASCENDING` array + inline `#[test] mod tests` (range / ordering / pairwise-distinct / spec-ratified-value asserts). |
| `client/src/ui/design_tokens/mod.rs` | EDIT | Append `pub mod overlays;` re-export + module-doc bullet for `overlays`. |
| `client/src/ui/hud/mod.rs` | EDIT | `use crate::ui::design_tokens::{overlays, typography, z_layers};` and `pub const HUD_DIM_OVERLAY_ALPHA: f32 = overlays::OVERLAY_DIM_ALPHA;` — preserves grep-stable consumer name. |
| `client/src/ui/shop_auction/mod.rs` | EDIT | `use crate::ui::design_tokens::{overlays, typography, z_layers};` and settlement-overlay BackgroundColor `0.58` → `overlays::OVERLAY_SCRIM_ALPHA`. |
| `client/src/presentation/result_screen.rs` | EDIT | `use crate::ui::design_tokens::{overlays, typography, z_layers};` and result-panel BackgroundColor `0.46` → `overlays::OVERLAY_SCRIM_ALPHA`. |
| `client/src/presentation/connection_lost_overlay.rs` | EDIT (comment-only) | Comment at `:205-207` rewritten to reference `overlays::OVERLAY_SCRIM_ALPHA` symbolically; the `0.32` alpha literal at `:208` is **preserved** (AC6 documented exclusion). |
| `client/Cargo.toml` | EDIT | Register `[[test]] ui_clean_pass_overlay_alpha_test` entry. |
| `tests/integration/ui_clean_pass/overlay_alpha_test.rs` | NEW | AC2 / AC3 / AC4 / AC5 / AC6 / AC7 integration assertions. |
| `production/qa/evidence/sprint-14-overlay-alpha-token/evidence.md` | NEW | This document. |

No write under `server/`, `shared/`, `production/sprint-status.yaml`,
`production/sprints/sprint-14.md`, `production/qa/qa-plan-sprint-14.md`,
`production/stage.txt`, or any `production/session-state/*` — verified
via `git diff --stat` against `origin/main` at commit time.

---

## AC verdicts

### AC1 — Overlay tokens authored — PASS

`client/src/ui/design_tokens/overlays.rs` (NEW) exports three named
constants with `///` doc comments naming canonical consumers:

| Token | Value | Canonical consumer |
|-------|-------|--------------------|
| `OVERLAY_DIM_ALPHA`   | `0.45` | HUD RESOLUTION dim overlay (`client/src/ui/hud/mod.rs`, alias `HUD_DIM_OVERLAY_ALPHA`). |
| `OVERLAY_SCRIM_ALPHA` | `0.55` | Modal scrim — settlement overlay (`client/src/ui/shop_auction/mod.rs`), result panel backdrop (`client/src/presentation/result_screen.rs`). |
| `OVERLAY_TOAST_ALPHA` | `0.80` | Toast root background (`client/src/ui/shop_auction/mod.rs`); reserved for the future toast-root migration the global-ui spec §6 names. |

Inline unit tests in `overlays.rs`:

- `ac1_three_named_overlay_alphas_present`
- `ac1_every_overlay_alpha_is_strictly_between_zero_and_one`
- `ac1_overlay_alphas_strictly_ascending_dim_lt_scrim_lt_toast`
- `ac1_overlay_alphas_pairwise_distinct`
- `ac1_overlay_dim_alpha_matches_spec_ratified_value`
- `ac1_overlay_scrim_alpha_matches_spec_ratified_value`
- `ac1_overlay_toast_alpha_matches_spec_ratified_value`
- `ac7_scrim_is_heavier_than_dim_for_visual_modal_blocker`
- `ac7_toast_is_heavier_than_scrim_for_foreground_notification`

### AC2 — HUD dim migrated — PASS

`client/src/ui/hud/mod.rs:34` `HUD_DIM_OVERLAY_ALPHA` constant is now
defined as `overlays::OVERLAY_DIM_ALPHA` (not a magic literal). The
consumer name `HUD_DIM_OVERLAY_ALPHA` is preserved as a grep-stable
alias so downstream consumer code (including the existing call site at
`client/src/ui/hud/mod.rs:~664` `BackgroundColor(Color::srgba(0.0, 0.0,
0.0, HUD_DIM_OVERLAY_ALPHA))`) is unchanged.

Verified by integration test
`ac2_hud_dim_overlay_alpha_routes_through_overlays_token` and by inline
unit test `ac1_overlay_dim_alpha_matches_spec_ratified_value`.

### AC3 — Settlement scrim migrated — PASS

`client/src/ui/shop_auction/mod.rs:~3550` settlement-overlay
BackgroundColor migrated from `Color::srgba(0.02, 0.05, 0.08, 0.58)`
to `Color::srgba(0.02, 0.05, 0.08, overlays::OVERLAY_SCRIM_ALPHA)`.

Note on line-drift: the readiness report (PROMPT 914) cited
`shop_auction/mod.rs:3539` *and* `:3550`. Inspection of `origin/main` at
worker base confirmed only **one** settlement-overlay BackgroundColor
spawn site at `:3550` carrying the `0.58` alpha — the `:3539` reference
in the readiness report and the story body referenced the
`toast_node()` argument line above the settlement overlay block, not a
second BackgroundColor site. A single migration suffices for AC3.

Verified by integration test
`ac3_settlement_overlay_reads_canonical_scrim_alpha`.

### AC4 — Result panel backdrop migrated — PASS

`client/src/presentation/result_screen.rs:~518` panel-backdrop
BackgroundColor migrated from `Color::srgba(0.02, 0.025, 0.035, 0.46)`
to `Color::srgba(0.02, 0.025, 0.035, overlays::OVERLAY_SCRIM_ALPHA)`.

Verified by integration test
`ac4_result_screen_backdrop_reads_canonical_scrim_alpha`.

### AC5 — Grep guard — PASS

Two layers of grep guard:

1. **Automated integration test**
   `ac5_grep_guard_no_pre_migration_scrim_literals_outside_design_tokens`
   walks every `*.rs` file under `client/src/` outside
   `client/src/ui/design_tokens/` and asserts none of the three
   pre-migration scrim/dim literal triplets remain:
   - `Color::srgba(0.02, 0.05, 0.08, 0.58)` (settlement scrim)
   - `Color::srgba(0.02, 0.025, 0.035, 0.46)` (result backdrop)
   - `Color::srgba(0.0, 0.0, 0.0, 0.45)` / `Color::rgba(0.0, 0.0, 0.0, 0.45)` (HUD dim)
2. **Broad regex sweep** for documented-exclusion enumeration (AC6
   below). The story-required broad regex
   `Color::(s)?rgba\(.*,\s*0\.[0-9]` returns the documented-exclusion
   list in AC6 — all classified as one of (a) scrim/dim migrated /
   (b) board-ghost-preview-or-HUD-timer-urgency (separate scope) /
   (c) other-with-rationale.

A sanity-check test
`ac5_grep_guard_pattern_actually_detects_a_synthesized_violation` is
included so a buggy walker that never matches cannot silently let
violations through.

### AC6 — Documented exclusions enumerated — PASS

See §AC6 enumeration table below. Every remaining inline
`Color::(s)?rgba(_, _, _, 0.x)` literal in `client/src/` (alpha < 1.0)
is classified as one of (a) scrim/dim already migrated, (b) board ghost
preview / HUD timer urgency / connection-lost-32 / accessibility
warning (preserved-intentional, separate scope), or (c) other-with-
rationale (chrome / button-state / text colors).

### AC7 — Single visual cohesion combat→settlement→result — PASS (token-level)

Token-level verification: integration test
`ac7_overlay_token_ordering_supports_visual_cohesion` asserts
`OVERLAY_DIM_ALPHA < OVERLAY_SCRIM_ALPHA < OVERLAY_TOAST_ALPHA`.
The settlement-overlay (`shop_auction/mod.rs`) and result-panel
(`result_screen.rs`) both now read `OVERLAY_SCRIM_ALPHA = 0.55` (was
`0.58` and `0.46` respectively), eliminating the inter-state flicker
PROMPT 802 §3.2 H4 surfaced.

Visual-capture verification: not produced by this worker run. AC7's
verification requires a manual capture of the combat → settlement →
result transition at 1920×1080 with a live two-client game running
against the playable client. PROMPT 916 worker scope is `/dev-story`
implementation + targeted-test verification only; story-006 visual
captures (HUD dim, settlement scrim, result backdrop, full transition)
are produced separately by the `/team-qa` orchestration that closes the
story (NOT this PROMPT 916). The visual change introduced by this
migration is bounded:

- HUD dim: NO VALUE CHANGE (`0.45` → `0.45`).
- Settlement scrim: `0.58` → `0.55` (Δ = `−0.03` lighter).
- Result backdrop: `0.46` → `0.55` (Δ = `+0.09` darker).

Both deltas are within the "≤ 0.1 alpha-step" cohesion budget the spec
§6 ratification covers.

### AC8 — Accept-risk dispositions unchanged — PASS

Verified by `git diff origin/main...HEAD --stat -- production/sprint-status.yaml`
returning empty across the worker commit (see §Verification commands below).
`QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a` rows untouched.

---

## AC6 — Documented exclusions enumeration

Every remaining inline `Color::(s)?rgba` literal in `client/src/` (any
alpha < 1.0) collected against `origin/main@3d99a04` and classified.
This table is **the** AC6 deliverable; each row gives the file:line,
the surface, the alpha, the classification, and a one-line rationale.

| File:line | Surface | Alpha | Classification | Rationale |
|-----------|---------|-------|----------------|-----------|
| `ui/hud/mod.rs:~664` | HUD RESOLUTION dim BackgroundColor | `HUD_DIM_OVERLAY_ALPHA` (`overlays::OVERLAY_DIM_ALPHA` = `0.45`) | (a) scrim/dim — migrated | Sourced via the design-token alias after Sprint 14 story 006. |
| `ui/shop_auction/mod.rs:~3550` | Settlement overlay BackgroundColor | `overlays::OVERLAY_SCRIM_ALPHA` (= `0.55`) | (a) scrim/dim — migrated | Sourced via the design-token after Sprint 14 story 006. |
| `presentation/result_screen.rs:~518` | Result panel root BackgroundColor | `overlays::OVERLAY_SCRIM_ALPHA` (= `0.55`) | (a) scrim/dim — migrated | Sourced via the design-token after Sprint 14 story 006. |
| `presentation/connection_lost_overlay.rs:208` | Connection-lost overlay BackgroundColor | `0.32` | (b) preserved-intentional | Intentionally lighter than canonical scrim per the overlay's own AC7 — the connection-lost overlay must keep gameplay UI visible underneath. Comment at `:205-213` now references `OVERLAY_SCRIM_ALPHA` symbolically. NOT in scope for story-006. |
| `presentation/connection_lost_overlay.rs:231` | Connection-lost panel BackgroundColor | `0.92` | (c) other — panel chrome | Inner panel surface; not a modal scrim — heavy enough to read as a foreground panel. NOT a scrim/dim surface. |
| `presentation/connection_lost_overlay.rs:232` | Connection-lost panel BorderColor | `0.85` | (c) other — panel chrome | Border alpha for the inner panel; not a scrim/dim surface. |
| `ui/settings/mod.rs:560` | Settings shell BackgroundColor | `0.70` | (c) other — settings shell | Settings panel surface; PROMPT 802 §3.7 names settings shell as a separate Tier 1+ chrome story. NOT in scope for story-006 (story-006 scope is modal scrim / dim only). |
| `ui/photosensitivity_warning.rs:74` | Photosensitivity warning BackgroundColor | `0.94` | (c) other — accessibility warning | Critical accessibility blocker on first-launch — intentionally near-opaque (player must consent before play). NOT in scope for story-006. |
| `ui/lobby.rs:893` | Lobby panel BackgroundColor | `0.92` | (c) other — lobby chrome | Lobby panel surface; not a modal scrim. |
| `ui/lobby.rs:917` | Lobby sub-panel BackgroundColor | `0.95` | (c) other — lobby chrome | Lobby form panel surface. |
| `ui/lobby.rs:931` | Lobby class-select chip BackgroundColor | `0.95` | (c) other — lobby chrome | Lobby class-select chip; not a scrim. |
| `ui/lobby.rs:943` | Lobby class-select chip BackgroundColor | `0.95` | (c) other — lobby chrome | Lobby class-select chip; not a scrim. |
| `ui/lobby.rs:971` | Lobby class-select chip BackgroundColor | `0.95` | (c) other — lobby chrome | Lobby class-select chip; not a scrim. |
| `ui/lobby.rs:993` | Lobby class-select chip BackgroundColor | `0.95` | (c) other — lobby chrome | Lobby class-select chip; not a scrim. |
| `ui/lobby.rs:1012` | Lobby class-select chip BackgroundColor | `0.95` | (c) other — lobby chrome | Lobby class-select chip; not a scrim. |
| `ui/hud/mod.rs:76` | `HUD_RESERVED_GOLD_TEXT_COLOR` | `0.65` | (c) other — text color | Reserved-gold readout text color (faded relative to live gold). Text color, not a scrim. |
| `ui/hud/mod.rs:1912,1916,1920` | HUD phase-timer urgency colors | `0.88`/`0.95`/`0.70` | (b) preserved-intentional — separate scope | `S11-UX-HUD-TIMER-URGENCY-VISUAL-001` (Tier 2 future row) — NOT in scope for story-006. |
| `ui/hud/mod.rs:1928,1932,1936` | HUD phase-timer urgency colors | `0.92`/`1.0`/`0.92` | (b) preserved-intentional — separate scope | Same row as above. |
| `ui/shop_auction/mod.rs:2768` | Auction-bid-button row BackgroundColor | `0.95` | (c) other — button state | Bid-button row chrome; AuctionBidButtonState-driven. NOT a scrim. |
| `ui/shop_auction/mod.rs:2770` | Auction-bid-button default BackgroundColor | `0.9` | (c) other — button state | Bid-button default chrome. NOT a scrim. |
| `ui/shop_auction/mod.rs:3418` | Auction-toast text TextColor (tweened) | `toast_state.alpha()` | (c) other — tween | Toast text fade tween; alpha is dynamic, not a static scrim literal. |
| `ui/shop_auction/mod.rs:3637,3653` | Auction settlement-overlay text TextColor | `0.0` initial | (c) other — tween initial | Initial fade-from-zero state of a settlement text tween. |
| `ui/shop_auction/mod.rs:3686` | Auction bid-target chrome BackgroundColor | `0.9` | (c) other — chrome | Bid target focus chrome. NOT a scrim. |
| `ui/shop_auction/mod.rs:3795` | Draft objective overlay BackgroundColor | `0.92` | (c) other — heavy overlay | Draft initial objective overlay — intentionally near-opaque (informational copy must read clearly). Visually distinct from the modal scrim. Worker-discretion not migrated; could be a candidate for `OVERLAY_TOAST_ALPHA` consumer in a follow-on story but is currently authored at `0.92` to read as a heavier informational blocker. |
| `ui/shop_auction/mod.rs:4076` | Auction bid-button text TextColor | `0.30` | (c) other — text color | Bid button disabled text color. NOT a scrim. |
| `ui/shop_auction/mod.rs:4520,4521,4536,4537` | Auction bid-button text/state TextColor | `0.80`/`0.30`/`0.35`/`0.0` | (c) other — button-state text | AuctionBidButtonState-driven text colors. NOT a scrim. |
| `ui/shop_auction/mod.rs:4543,4544,4545,4546` | Auction bid-button background per state | `0.90`/`0.92`/`0.0`/`0.55` | (c) other — button-state chrome | AuctionBidButtonState-driven button backgrounds. Note: the `0.55` at `:4546` *coincidentally* matches `OVERLAY_SCRIM_ALPHA` but is unrelated to modal scrim semantics — it is the disabled-bid-button state color. NOT migrated (different semantic surface). |
| `card_animations/damage_numbers.rs:11,12` | Damage number text color | `1.0`/`0.0` | (c) other — text fade | Floating damage number color and fade-target. NOT a scrim. |
| `card_animations/placement.rs:18` | Placement cell highlight color | `0.85` | (c) other — gameplay highlight | Drop-target highlight color, not a scrim. |
| `presentation/board_rendering.rs:254,262,270,278,286,294,302,310,325` | Board status icon colors | `1.0` (opaque) | (c) other — fully opaque | Listed for completeness; all are alpha = `1.0` (the broad AC5 regex pattern matches but the alpha is opaque, not an overlay). |
| `presentation/board_rendering.rs:768-770` | Board condition icon colors | `1.0` (opaque) | (c) other — fully opaque | Same as above. |
| `presentation/board_rendering.rs:847` | Board lane inactive BackgroundColor | `0.55` | (c) other — gameplay highlight | Lane-inactive overlay; gameplay-layer not modal-scrim. Coincidentally matches `OVERLAY_SCRIM_ALPHA` but the semantic surface is different. |
| `presentation/board_rendering.rs:848` | Board lane valid-spawn BackgroundColor | `0.88` | (c) other — gameplay highlight | Lane-valid-spawn overlay. NOT a modal scrim. |
| `presentation/board_rendering.rs:2235` | Board chrome BackgroundColor | `0.94` | (c) other — chrome | Internal board chrome. NOT a modal scrim. |
| `presentation/board_rendering.rs:2349` | Board chrome BackgroundColor | `0.76` | (c) other — chrome | Internal board chrome. NOT a modal scrim. |
| `presentation/board_rendering.rs:2711` | Board status indicator | `0.5` | (c) other — gameplay indicator | Status indicator; not a modal scrim. |
| `presentation/board_rendering.rs:2737` | **Board ghost preview** sprite color | `0.28` | (b) preserved-intentional — separate scope | **`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`** (Tier 2 future row). NOT in scope for story-006. |
| `presentation/result_screen.rs:19` | `IDLE_BORDER` BorderColor const | `0.35` | (c) other — border | Result-screen idle-border color. NOT a scrim. |
| `presentation/result_screen.rs:541` | Result panel inner BackgroundColor | `0.94` | (c) other — panel chrome | Inner result panel surface; not a modal scrim. |
| `presentation/result_screen.rs:542` | Result panel BorderColor | `0.26` | (c) other — border | Inner panel border; not a scrim. |

Totals:

- **Classification (a) scrim/dim migrated**: 3 sites (HUD dim, settlement, result backdrop). All routed through `overlays::OVERLAY_*_ALPHA`.
- **Classification (b) preserved-intentional / separate scope**: 9 sites (connection-lost-32 + 6 HUD timer urgency + board ghost preview + 1 acknowledgment of `presentation/board_rendering.rs:2737`).
- **Classification (c) other-with-rationale**: balance (~30 sites) — button states, text colors, panel chrome, gameplay highlights, accessibility warning, settings shell.

No further migrations required for story-006 scope.

---

## Verification commands run

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'

cargo fmt -p client -- --check
cargo check -p client --tests
cargo test -p client --lib ui::design_tokens::overlays
cargo test -p client --test ui_clean_pass_overlay_alpha_test

# Regression sweep (nearby Tier 0 modules, story-prescribed targets)
cargo test -p client --lib ui::design_tokens::z_layers
cargo test -p client --lib ui::design_tokens::typography
cargo test -p client --test ui_clean_pass_z_layers_test
cargo test -p client --test ui_clean_pass_typography_test
cargo test -p client --test result_screen_mvp_test
cargo test -p client --test hud_plugin_scaffold_test
cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test
cargo test -p client --test connection_lost_overlay_test
cargo test -p client --test hud_resolution_dim_test

git diff origin/main...HEAD --stat -- \
  shared/src/protocol.rs \
  server/ \
  production/qa/qa-plan-sprint-14.md \
  production/sprints/sprint-14.md \
  production/sprint-status.yaml \
  production/stage.txt
# All six MUST be empty for a worker /dev-story run.
```

Per the Sprint 14 QA-plan no-full-workspace-tests-by-default policy,
full-workspace `cargo test` is deferred to the end-of-sprint integration
smoke. Targeted tests above cover all story-006 ACs.

See `worker-final-report.md` (Step 1 chat message, mirrored at
`reports/PROMPT-916-S14-Overlay-Alpha-Token-Dev-Story.md`) for verbatim
command results.

---

## Carry-forward statements (unchanged on `origin/main`)

- `S8-QA-001-W1` OPEN (two-client GAME_OVER closure not claimed).
- `QA-COND-0005` accepted-risk (Standard-tier accessibility not claimed).
- `QA-COND-0006` accepted-risk (playtest validation not claimed).
- `PAW-TD-*-a` accept-risk preserved (final-art / asset-production not claimed).
- PROMPT 761 Polish→Release `FAIL` preserved (no retry attempted).
- PROMPT 683-era runtime divergence question preserved (folded into Sprint 12 story 019 cannot-reproduce closure — no third same-scope retest).
- `TQ-S12-C1..C7` verbatim.
- Sprint 13 / 12 / 11 / 10 closeouts unchanged.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed.
- Stage UNCHANGED `Polish`.
- Sprint 14 disposition UNCHANGED (worker does not flip `production/sprint-status.yaml`).

---

## ADR / engine compliance

- Pure `pub const f32` design-token module — no `bevy_ui` APIs at module-author time; consumers continue to call `Color::srgba(...)` with the token plugged in as the alpha channel parameter.
- `Color::srgba(f32, f32, f32, f32)` is the stable Bevy 0.18 constructor — verified unchanged.
- `liv-bevy-018` skill discipline applied to every `.rs` edited (HUD module, shop_auction module, result_screen, connection_lost_overlay, design_tokens/mod.rs, design_tokens/overlays.rs, integration test).
- `liv-bevy-lightyear` not needed — no networking surface in scope.

---

## Final status line

`916: S12-TD-UI-OVERLAY-ALPHA-TOKEN-001: PASS`
