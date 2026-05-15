# Sprint 14 — S11-TD-UI-FLEX-STRIPS (story 004) Dev-Story Evidence

> **Story file**: `production/epics/ui-clean-pass/story-004-ui-flex-strips.md`
> **Authoring prompt**: PROMPT 915 (`/dev-story` worker)
> **Source-of-truth at start**: `origin/main@3d99a04` (PROMPT 912 integration of
> `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`)
> **Worktree**: `D:/_DEV/wt/ccgs-prompt-915-flex-strips`
> **Branch**: `work/s14-flex-strips`
> **Readiness verdict consumed**: PROMPT 913 READY
> (`reports/PROMPT-913-S14-FLEX-STRIPS-Story-Readiness.md`)

---

## §1 Status / No-Claim Banner

PROMPT 915 (this `/dev-story` worker) authors the Tier 0 flex-strip
composition primitive module, the spacing-scale token module, and the
HUD / hand UI migration spot-checks named by story 004 AC1-AC8. It does
**not**:

- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Modify the story 004 file body.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Modify any code under `server/` or `shared/`.

### Accept-risk dispositions preserved verbatim

- **`QA-COND-0005`** — Standard-tier accessibility remains
  **accepted-risk** (friend-game scope only). The `LOBBY_BUTTON_HEIGHT =
  30.0` ≥44px hit-target defect is **not** advanced. Keyboard
  navigation, screen-reader hints, focus-order semantics, colorblind
  modes, and text scaling remain out of scope.
- **`QA-COND-0006`** — playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**.
- **`PAW-TD-002-a` … `PAW-TD-006-a`** — placeholder-art accept-risk
  preserved across PAW-002..PAW-006. UI clean-pass repair is layout /
  composition / hierarchy / typography / z-order / spacing work and
  does **not** advance placeholder-art resolution.
- **`S8-QA-001-W1`** OPEN. Two-client GAME_OVER closure is **not**
  claimed.
- **PROMPT 761 `Polish->Release` `FAIL`** preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry
  is in scope.

### What this story does NOT claim

Public release readiness, release-candidate (RC) readiness, full game
completion, broad / Standard-tier accessibility completion, playtest /
fun-hypothesis validation, full playable-client manual QA, two-client
GAME_OVER closure, final-art / asset-production completion, Sprint 14
close-out, Polish → Release gate-check retry, stage advance from
`Polish` to `Release`, underlying drag-runtime bug fix (Sprint 12 story
019).

---

## §2 Implementation Summary

### New files

| Path | Purpose |
|---|---|
| `client/src/ui/design_tokens/spacing.rs` | `SPACING_XS / SM / MD / LG / XL` (4 / 8 / 16 / 24 / 32) per spec §4. Strict-ascending ordering + canonical-values tests. `ALL_SPACINGS_ASCENDING` + `SPACING_MIN_GAP` audit constants. |
| `client/src/ui/design_tokens/strips.rs` | `HeaderBar` / `LaneBar` / `HandBar` / `FooterBar` strip-composition primitives per spec §9. Heights 60 / 60 / 180 / 40. Each exports a marker component + a `*_node()` helper returning a `Display::Flex` `Node`. `StripContract` struct documents per-strip `flex_direction` / `justify_content` / `align_items`. Inline unit tests for AC1 / AC6. |
| `tests/integration/ui_clean_pass/strips_test.rs` | Integration test bin covering AC1 / AC2 / AC3 / AC4 / AC5 / AC6 / AC7 / AC8. Mirrors the conventions of `tests/integration/ui_clean_pass/z_layers_test.rs` and `typography_test.rs`. |
| `production/qa/evidence/sprint-14-ui-flex-strips/dev-story-evidence.md` | This file. |

### Modified files

| Path | Change |
|---|---|
| `client/src/ui/design_tokens/mod.rs` | Add `pub mod spacing;` + `pub mod strips;` to the existing `pub mod typography; pub mod z_layers;` block. Doc-comment refresh listing the new modules. |
| `client/src/ui/hud/mod.rs` | (1) Import `spacing, strips` from `crate::ui::design_tokens`. (2) Delete `HUD_GOLD_ROW_GAP_PX = 48.0` and `HUD_SECONDARY_ROW_GAP_PX = 28.0` magic constants. (3) Replace `HUD_GOLD_ROW_GAP_PX` call site (opponent_gold top offset) with `spacing::SPACING_XL + spacing::SPACING_MD` (48). (4) Replace `top_left_second_line_node` magic `+ HUD_SECONDARY_ROW_GAP_PX` with `+ spacing::SPACING_XL - spacing::SPACING_XS` (28). (5) Replace timer-bar magic `top: hud_margin + 48.0` with `top: strips::HEADER_BAR_HEIGHT_PX` (60). (6) Replace figurine magic `bottom: hud_margin + 60.0` with `bottom: strips::FOOTER_BAR_HEIGHT_PX + spacing::SPACING_XL` (72). (7) Spawn `strips::HeaderBar` + `strips::FooterBar` primitives as named children of the HUD root via `strips::header_bar_node()` / `strips::footer_bar_node()`. |
| `client/src/ui/hand/mod.rs` | (1) Import `strips` from `crate::ui::design_tokens`. (2) Spawn `strips::HandBar` primitive (tagged with `HandUiEntity`) via `strips::hand_bar_node()` as the new parent of `HandFanRoot`. (3) Re-parent `HandFanRoot` via `ChildOf(hand_bar)`. (4) Bump `HAND_UI_ENTITY_COUNT` by `+ 1` to account for the new `HandBar` primitive. |
| `client/Cargo.toml` | Register the new `ui_clean_pass_strips_test` integration test bin pointing at `../tests/integration/ui_clean_pass/strips_test.rs`. |

### Deleted constants

- `client/src/ui/hud/mod.rs::HUD_GOLD_ROW_GAP_PX = 48.0` — per PROMPT
  802 §3.9 G2 magic-constant inventory. Recomposed via
  `spacing::SPACING_XL + spacing::SPACING_MD = 48`.
- `client/src/ui/hud/mod.rs::HUD_SECONDARY_ROW_GAP_PX = 28.0` — same
  PROMPT 802 entry. Recomposed via
  `spacing::SPACING_XL - spacing::SPACING_XS = 28`.

The deletion is enforced by the new `ac7_no_gap_px_identifier_in_hud_module`
test in `tests/integration/ui_clean_pass/strips_test.rs`.

---

## §3 AC Verdicts

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 — Strip primitive module authored | **PASS** | `client/src/ui/design_tokens/strips.rs` exports `HeaderBar` / `LaneBar` / `HandBar` / `FooterBar` marker components and `*_node()` helpers. Each helper returns a `Display::Flex` `Node` with documented `flex_direction` / `justify_content` / `align_items` (asserted in inline test `ac1_each_strip_node_declares_display_flex_and_documented_axes` and integration test `ac1_three_required_strip_primitives_exported_with_flex_display`). |
| AC2 — Spacing-scale constants | **PASS** | `client/src/ui/design_tokens/spacing.rs` exports `SPACING_XS` (4) / `SM` (8) / `MD` (16) / `LG` (24) / `XL` (32). Strict-ascending invariant asserted by inline `ac2_five_named_spacings_strictly_ascending` test and integration `ac2_spacing_scale_strictly_ascending_canonical_values`. Recomposition rule for 48 (XL + MD) and 28 (XL - XS) asserted as standalone integration tests. |
| AC3 — HUD top strip migrated | **PASS** | `client/src/ui/hud/mod.rs` (1) imports `spacing` + `strips` from `crate::ui::design_tokens`; (2) deletes `HUD_GOLD_ROW_GAP_PX` and `HUD_SECONDARY_ROW_GAP_PX`; (3) spawns `strips::HeaderBar` primitive as a child of HUD root via `strips::header_bar_node()`; (4) recomposes gold-row offset via `spacing::SPACING_XL + spacing::SPACING_MD`; (5) recomposes secondary-row offset via `spacing::SPACING_XL - spacing::SPACING_XS`; (6) anchors timer bar to `strips::HEADER_BAR_HEIGHT_PX`. Asserted by integration tests `ac3_hud_module_spawns_header_bar_primitive`, `ac3_hud_gold_row_offset_resolves_through_spacing_tokens`, `ac3_hud_secondary_row_offset_resolves_through_spacing_tokens`, `ac3_hud_timer_bar_anchors_to_header_bar_height_token`. |
| AC4 — HUD bottom strip migrated | **PASS** | `client/src/ui/hud/mod.rs` spawns `strips::FooterBar` primitive as a child of HUD root via `strips::footer_bar_node()`; figurine bottom offset recomposed via `strips::FOOTER_BAR_HEIGHT_PX + spacing::SPACING_XL` (72 = 40 + 32, same pixel value, strip-relative anchor). Asserted by integration tests `ac4_hud_module_spawns_footer_bar_primitive` and `ac4_hud_figurine_anchors_to_footer_bar_and_spacing_tokens`. |
| AC5 — Hand UI card row migrated | **PASS** | `client/src/ui/hand/mod.rs` spawns `strips::HandBar` primitive (tagged with `HandUiEntity`) via `strips::hand_bar_node()` and re-parents `HandFanRoot` via `ChildOf(hand_bar)`. The existing `f190cc7` chrome contract (7 children at 100×100% / 20×20% / 15×15%) is preserved unchanged inside `HandFanRoot`; the strip wraps it without modifying the child layout. `HAND_FAN_STRIP_HEIGHT_PX` retained as the `HandFanRoot` local height; `HandBar` is 180 px with `overflow: visible` so the fan extends 80 px above the strip footprint per PROMPT 913 Concern #2 reconciliation option (b). Asserted by integration tests `ac5_hand_module_imports_strips_and_spawns_hand_bar_primitive` and `ac5_hand_fan_root_is_a_child_of_hand_bar`. |
| AC6 — Stable dimensions across viewport ratios | **PASS** | Strip heights are declared as `pub const HEADER_BAR_HEIGHT_PX: f32 = 60.0` / `FOOTER_BAR_HEIGHT_PX = 40.0` / `HAND_BAR_HEIGHT_PX = 180.0`. The strip Node helpers return `height: Val::Px(<const>)` (pixel-fixed) and `width: Val::Percent(100.0)` (viewport-scaled). Integration test `ac6_strip_heights_are_identical_across_every_canonical_viewport` iterates the 6-viewport canonical matrix (1366×768 / 1920×1080 / 1920×1200 / 1280×960 / 3840×2160 / 2560×1080) and asserts every strip height resolves to the canonical pixel value at every viewport, plus a positive centre-play-area constraint. Inline `ac6_top_strip_does_not_overlap_bottom_strips_in_canonical_viewport` test in `strips.rs` extends the no-overlap invariant. |
| AC7 — No per-module `_GAP_PX` magic constants | **PASS** | `HUD_GOLD_ROW_GAP_PX` and `HUD_SECONDARY_ROW_GAP_PX` deleted from `client/src/ui/hud/mod.rs`. Integration test `ac7_no_gap_px_identifier_in_hud_module` walks the file and asserts no surviving `_GAP_PX` identifier outside doc comments. |
| AC8 — Strip primitive unit test | **PASS** | `cargo test -p client --test ui_clean_pass_strips_test` exercises every AC. The bin includes `ac8_each_strip_node_resolves_to_documented_flex_axis_set`, `ac8_strip_anchors_match_spec_column_composition`, and `ac8_strip_marker_components_are_distinct_zero_sized_components` as the canonical strip primitive unit-style assertions. |
| AC9 — Friend-game scope preserved | **PASS** | `production/sprint-status.yaml` is forbidden by the worker scope (not modified). The accept-risk dispositions `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` / `S8-QA-001-W1` / PROMPT 761 `FAIL` are not flipped. The story's no-claim banner is preserved verbatim in §1 of this evidence document. |

---

## §4 Reconciliation of PROMPT 913 readiness concerns

| Concern | Resolution |
|---|---|
| #1 — AC8 verification command spelling | The new `ui_clean_pass_strips_test` integration bin is the canonical path. Inline unit tests under `strips.rs` / `spacing.rs` are also exposed via `cargo test -p client --lib design_tokens::strips` / `design_tokens::spacing`. The integration bin spelling matches the QA-plan §line 205 prescription. |
| #2 — HandBar 180 vs HAND_FAN_STRIP_HEIGHT_PX 260 divergence | Worker chose option (b): `HandBar` is a 180 px-tall strip; `HandFanRoot` is its child with local height 260 px. `overflow: visible` is set on the `HandBar` node so the fan chrome extends 80 px above the strip footprint without clipping. The `f190cc7` chrome layout is preserved verbatim. This is the minimum-blast-radius reconciliation consistent with §9 of the global UI design spec. |
| #3 — Sprint-status blocker text staleness | Out of scope for this `/dev-story` worker per "forbidden writes to `production/sprint-status.yaml`". The row text refresh is deferred to the `/story-done` paperwork prompt. |
| #4 — Parallel-launch collision risk with story 006 | This worker runs serially against `origin/main@3d99a04`. The story 006 `_GAP_PX`-adjacent line ranges in `hud/mod.rs:34` are not modified by this worker. |
| #5 — Story 007 paperwork lag | Out of scope. Substantive content dependency satisfied by `docs/ux/global-ui-design-spec.md` already on `origin/main@3d99a04`. |

---

## §5 Verification commands run

PowerShell session on Windows / MSVC with the binding Sprint 14 Cargo
resource policy applied (per PROMPT 899 / 904 / 905 / 911 precedent):

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Commands (results appended after each):

```text
cargo fmt -p client -- --check       → PASS  (see PROMPT 915 final report)
cargo check -p client                → PASS
cargo test -p client --lib design_tokens::spacing
                                     → PASS  (spacing module inline tests)
cargo test -p client --lib design_tokens::strips
                                     → PASS  (strips module inline tests)
cargo test -p client --test ui_clean_pass_strips_test
                                     → PASS  (integration strip bin)
cargo test -p client --test ui_clean_pass_z_layers_test
                                     → PASS  (regression — story 002)
cargo test -p client --test ui_clean_pass_typography_test
                                     → PASS  (regression — story 003)
cargo test -p client --test ui_viewport_invariants_test
                                     → PASS  (regression — story 005)
git diff --check                     → PASS  (no whitespace errors)
git diff --cached --check            → PASS
```

The PROMPT 915 final report captures the verbatim PowerShell transcript
and confirms the **Cargo resource policy was applied** in the same
session prior to every Cargo invocation. Visual capture coverage at the
five viewport ratios listed in §AC6 is a follow-on QA-tester deliverable
under `/team-qa` and out of scope for this `/dev-story` worker per the
PROMPT 913 readiness no-claim list.

---

## §6 Carried non-claims preserved

- `S8-QA-001-W1` OPEN.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk.
- `PAW-TD-*-a` accept-risk across PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved; no third
  same-scope retest per `TQ-S12-C2`.
- `TQ-S12-C1..C7` verbatim. Sprint 12 / 11 / 10 closeouts preserved.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed.
- Sprint 13 close-out `closed-with-conditions` (PROMPT 894) preserved.
- PROMPT 761 `Polish->Release` `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; **no
  retry** in scope.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 13 carry remains
  human-operator-blocked.
- Sprint 14 disposition unchanged `active`. Stage unchanged `Polish`.
