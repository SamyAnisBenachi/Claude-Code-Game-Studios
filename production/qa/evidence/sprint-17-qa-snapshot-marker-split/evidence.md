# PROMPT 1122 — S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 evidence

> **Story**: `production/epics/ui-clean-pass/story-019-qa-snapshot-marker-split.md`
> **Worker prompt**: 1122
> **Worker branch**: `work/s17-qa-snapshot-marker-split`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s17-qa-snapshot-marker-split`
> **Source-of-truth at execution**: rebased onto `origin/main@5345164`
> (strict fast-forward descendant of `89ce149`; includes PROMPT 1119 bid-button
> phase-race integration, PROMPT 1120 hand-fan-root B0004 cleanup story-done,
> PROMPT 1121 bid-button phase-race story-done).
> **Skill**: `liv-bevy-018` active for every `.rs` edit. `liv-bevy-lightyear` NOT used.

## Summary

Implements S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 ACs 1–10 in client-side code only:

- Per-sub-surface root markers introduced in HUD (`HudTopStripRoot`,
  `HudBottomStripRoot`, `HudScoreboardDotRoot`, `HudDimOverlayRoot`) and Hand
  (`HandBarRoot`, `HandDraftGridSlotRoot`, `PlacementActionPanelRoot`;
  `HandFanRoot` pre-existing). Shop/auction already exposed
  `ShopAuctionPanelRoot` per panel — that enum is the canonical
  per-sub-surface marker and is now consumed directly by `UiCountQueries`.
- Universal markers `HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity`
  marked `#[deprecated(since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001", note = …)]`
  per AC1 default option (a) in AC8 — preserved on existing entities so
  PROMPT 1022 / 1034 / 1036 historical snapshot comparisons still resolve.
- `client/src/presentation/qa_snapshot.rs`:
  - `UiCountQueries` extended with per-sub-surface marker queries that read
    `&Visibility`; visible counts use a `Visibility != Hidden` filter on the
    marker entity's *own* `Visibility` component.
  - `UiCounts` extended with new `*_visible` fields (per AC2 / AC4 / AC5);
    legacy `hud_entities` / `hand_ui_entities` / `shop_auction_entities` /
    `connection_lost_overlay_roots` / `result_screen_roots` preserved as
    `#[deprecated]` legacy fields.
  - `format_snapshot_id(counter, unix_millis, session_id)` now emits
    `{session_id}-{counter:06}-{unix_millis}` post-handshake and
    `pre-session-{counter:06}-{unix_millis}` before the handshake (AC6);
    the directory name follows (AC7).
  - `short_id` updated so the button feedback label "Saved <id>" still
    surfaces the counter token (operator-relevant chunk) under the new
    format.
  - `CCGS_QA_SNAPSHOT` env-var contract preserved verbatim (AC10).

No protocol shape change, no server change, no shared change. No new system
set, no schedule wiring change (AC11 + AC12). No closure of PROMPT 1022 findings,
no closure of AUDIT-1076-*, no Standard-tier accessibility or playtest claim,
no Sprint 17 close-out (AC13 + AC14). Worker branch pushed only; never main
(AC15). Cargo resource policy env vars set in every PowerShell session that
ran cargo (AC16).

## Acceptance Criteria — verification table

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 universal markers split into per-sub-surface markers | PASS | `client/src/ui/hud/mod.rs` adds `HudTopStripRoot`, `HudBottomStripRoot`, `HudScoreboardDotRoot`, `HudDimOverlayRoot`; `client/src/ui/hand/mod.rs` adds `HandBarRoot`, `HandDraftGridSlotRoot`, `PlacementActionPanelRoot` (alongside pre-existing `HandFanRoot`); `client/src/ui/shop_auction/mod.rs` already provided `ShopAuctionPanelRoot` per panel and is now consumed canonically by `UiCountQueries`. Universal markers `HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity` carry `#[deprecated(since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001", …)]`. |
| AC2 `UiCountQueries` consumes per-sub-surface markers, JSON emits per-sub-surface counts | PASS | `UiCounts` gains `hud_root_visible`, `hud_top_strip_visible`, `hud_bottom_strip_visible`, `hud_scoreboard_dot_visible`, `hud_dim_overlay_visible`, `hand_bar_visible`, `hand_fan_visible`, `hand_draft_grid_slot_visible`, `placement_action_panel_visible`, `shop_draft_offering_visible`, `shop_panel_visible`, `auction_panel_visible`, `shop_footer_visible`, `auction_toast_visible`, `settlement_overlay_visible`, `connection_lost_overlay_visible`, `result_screen_visible`. Asserted by `each_per_sub_surface_marker_with_visible_visibility_contributes_to_count`. |
| AC3 Visibility filter applied | PASS | `UiCountQueries::snapshot` calls `is_visibility_visible` (true when `Visibility != Hidden`) on every per-sub-surface count. Asserted by `hidden_visibility_excludes_marker_from_per_sub_surface_counts` and `inherited_visibility_counts_as_visible`. |
| AC4 connection_lost_overlay visible flag honours Visibility | PASS | New `connection_lost_overlay_visible` field; legacy `connection_lost_overlay_roots` retained. Asserted by `connection_lost_overlay_visible_honours_own_visibility`. |
| AC5 result_screen visible flag honours Visibility | PASS | New `result_screen_visible` field; legacy `result_screen_roots` retained. Asserted by `result_screen_visible_honours_own_visibility`. |
| AC6 snapshot ID prefix includes session_id / pre-session- | PASS | `format_snapshot_id(counter, unix_millis, session_id)` signature; pre-session prefix constant `QA_SNAPSHOT_PRE_SESSION_PREFIX = "pre-session"`. Asserted by `pre_session_prefix_used_when_session_id_is_none` and `session_id_prefix_used_when_session_id_is_some`. |
| AC7 two-client capture does not alias snapshot directories | PASS | Asserted by `two_clients_with_distinct_session_ids_do_not_alias` (fixture-injected identities — the worker-allowable fallback when running two clients is not feasible during `/dev-story`). |
| AC8 legacy universal counts preserved as `#[deprecated]` | PASS (option (a)) | `UiCounts::hud_entities`, `hand_ui_entities`, `shop_auction_entities`, `connection_lost_overlay_roots`, `result_screen_roots` each carry `#[deprecated(since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001", note = "Use … instead.")]`. Asserted by `legacy_universal_counts_remain_populated_alongside_new_fields`. |
| AC9 integration test covers marker split | PASS | `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs` (NEW); 11 tests; registered as `[[test]] name = "ui_clean_pass_qa_snapshot_marker_split_test"` in `client/Cargo.toml`. All passing. |
| AC10 `CCGS_QA_SNAPSHOT=1` env-var contract preserved | PASS | `QA_SNAPSHOT_ENV_VAR` constant unchanged; `from_env_values` behaviour unchanged. Asserted by `ccgs_qa_snapshot_env_contract_preserved`. |
| AC11 no protocol / server change | PASS | `git diff --stat HEAD~..HEAD` (post-commit) shows only `client/`, `tests/`, and `production/qa/evidence/` paths. |
| AC12 ADR-021 schedule preserved | PASS | No new system set, no schedule wiring, no spawn-site relocation — new markers are inserted *alongside* the existing universal markers on the same entities. |
| AC13 no accept-risk closure claimed | PASS | This evidence file explicitly does not claim closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, any PROMPT 1022 finding, any AUDIT-1076-* finding, or any SOURCE-1077-* finding outside the three bundled here (-08 / -09 / -16). PROMPT 1112 AC3 reserve-strip carry preserved; PROMPT 1114 card-display helper behaviour preserved; PROMPT 1118 hand B0004 Transform behaviour preserved; PROMPT 1119 bid-button phase-race behaviour preserved; `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-blocked carry preserved. |
| AC14 sprint disposition preserved | PASS | No edits to `production/sprint-status.yaml`, `production/sprints/*`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`. |
| AC15 worker branch scope contained | PASS | Branch `work/s17-qa-snapshot-marker-split` pushed to origin; `main` never pushed. |
| AC16 Cargo resource policy applied | PASS | Every PowerShell session that ran cargo set `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`. D: free space at execution: ~772 GB free (well above the 40 GB threshold). |

## Test results

All gates pass under the Cargo resource policy on Windows / MSVC.

### `cargo check -p client`

```
warning: `client` (lib) generated 82 warnings
    Finished `dev` profile [optimized] target(s) in 7.49s
exit 0
```

The 82 warnings are deprecation warnings emitted at every existing spawn /
query site of `HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity` — the
deprecation surface is intentional (AC1 + AC8 default option (a)). No new
errors. No new non-deprecation warnings.

### `cargo test -p client --test ui_clean_pass_qa_snapshot_marker_split_test`

```
running 11 tests
test ccgs_qa_snapshot_env_contract_preserved ... ok
test pre_session_prefix_used_when_session_id_is_none ... ok
test session_id_prefix_used_when_session_id_is_some ... ok
test two_clients_with_distinct_session_ids_do_not_alias ... ok
test hidden_visibility_excludes_marker_from_per_sub_surface_counts ... ok
test repeated_marker_spawns_accumulate_into_visible_count ... ok
test connection_lost_overlay_visible_honours_own_visibility ... ok
test result_screen_visible_honours_own_visibility ... ok
test legacy_universal_counts_remain_populated_alongside_new_fields ... ok
test inherited_visibility_counts_as_visible ... ok
test each_per_sub_surface_marker_with_visible_visibility_contributes_to_count ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `cargo test -p client --test qa_snapshot_overlay_test` (existing bin, extended)

```
running 20 tests
... all 20 PROMPT 1013 / 1019 tests
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The existing `build_snapshot_serialises_present_resources_without_warnings`
construction site for `UiCounts` was extended with `..UiCounts::default()`
so the new per-sub-surface fields default to `0`. No semantic change to
that test's assertions.

### Adjacent surface focused bins (build-gate isolation per story §"Build gate scope")

```
hud_plugin_scaffold_test                          : 3 passed; 0 failed
hand_ui_plugin_scaffold_test                      : 8 passed; 0 failed
hud_top_strip_layout_test                         : 4 passed; 0 failed
hud_bottom_strip_layout_test                      : 8 passed; 0 failed
shop_auction_ui_shop_panel_test                   : 8 passed; 0 failed
shop_auction_ui_auction_activation_test           : 8 passed; 0 failed
shop_auction_ui_draft_initial_grid_test           : 10 passed; 0 failed
hand_fan_root_b0004_hierarchy_test                : 10 passed; 0 failed
```

### `git diff --check`

```
exit 0
```

No whitespace damage.

### Build gate isolation note (pre-existing unrelated failure)

`shop_auction_ui_plugin_scaffold_formulas_test::shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
fails with `left: 87, right: 82` on the unmodified `origin/main@5345164`
baseline before any PROMPT 1122 edit — confirmed by stashing all
PROMPT 1122 changes and re-running the bin. The assertion arithmetic
(`tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:50`) drifted
from the actual `ShopAuctionUiEntity`-tagged spawn count introduced by an
upstream worker (likely the modal-stack / surface-paint / featured-card
landings since the formula was last reconciled). Per the story §"Build
gate scope": **only owned files and directly related tests** — this row
does not block on unrelated in-flight worker drift, so the failure is
recorded here and forwarded for a separate counter-reconciliation prompt;
PROMPT 1122 makes no edit to that formula.

## Files changed (owned files only)

| Path | Change |
|------|--------|
| `client/src/presentation/qa_snapshot.rs` | Per-sub-surface queries via `UiCountQueries` (with `HandVisibilityQueries` + `ShopAuctionVisibilityQueries` sub-`SystemParam`s); new `UiCounts::*_visible` fields; legacy fields deprecated. `format_snapshot_id` takes `session_id: Option<u64>` and prefixes ids with the session id or `pre-session-`. `short_id` updated to surface the counter token under the new format. `QA_SNAPSHOT_PRE_SESSION_PREFIX` constant added. |
| `client/src/ui/hud/mod.rs` | `HudEntity` marked `#[deprecated]`. New markers `HudTopStripRoot`, `HudBottomStripRoot`, `HudScoreboardDotRoot`, `HudDimOverlayRoot`; applied at the existing spawn sites alongside the universal marker. |
| `client/src/ui/hand/mod.rs` | `HandUiEntity` marked `#[deprecated]`. New markers `HandBarRoot`, `HandDraftGridSlotRoot`, `PlacementActionPanelRoot`; applied at `hand_bar`, every `grid_slot`, and `placement_action_panel`. `HandFanRoot` left untouched. |
| `client/src/ui/shop_auction/mod.rs` | `ShopAuctionUiEntity` marked `#[deprecated]`. `ShopAuctionPanelRoot` enum doc updated to declare it the canonical per-sub-surface marker. No new entities, no spawn-site relocation. |
| `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs` | NEW. 11 tests across AC1/2/3/4/5/6/7/8/9/10. |
| `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs` | `UiCounts` literal initialiser extended with `..UiCounts::default()` so new per-sub-surface fields default to `0`; the test function carries `#[allow(deprecated)]` so reading legacy `hud_entities` does not warn. |
| `client/Cargo.toml` | One new `[[test]]` entry registering `ui_clean_pass_qa_snapshot_marker_split_test`. No other change — PROMPT 1109 Vulkan feature edits (if any) preserved verbatim. |
| `production/qa/evidence/sprint-17-qa-snapshot-marker-split/evidence.md` | NEW (this file). |

## Non-claims

- This prompt does NOT run `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, `/qa-plan` on this story.
- This prompt does NOT close any PROMPT 1022 / AUDIT-1076-* finding, nor
  any SOURCE-1077-* finding outside the three bundled here (-08 / -09 / -16).
- This prompt does NOT modify `production/sprint-status.yaml`,
  `production/sprints/*`, `production/stage.txt`,
  `production/session-state/*`, or any other production tracker / sprint
  plan / QA plan / smoke / Team-QA / gate-check artifact.
- This prompt does NOT push `main`. Only the worker branch
  `work/s17-qa-snapshot-marker-split` is pushed.
- This prompt does NOT change the snapshot output directory layout outside
  the directory-name prefix (the prefix change is in scope; deeper layout
  changes remain out).
- This prompt does NOT touch `server/`, `shared/`,
  `tests/integration/server/`, `tests/unit/server/`, `.cargo/`, `.github/`,
  `Trunk.toml`, ADRs, CLAUDE.md, or AGENTS.md.

## Conditions carried forward

Per the story §"Closure Trail / Conditions carried forward unchanged":

- Sprint 16 disposition `closed-with-conditions` UNCHANGED.
- Sprint 17 stage `Polish` UNCHANGED.
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked
  carry preserved.
- 24 PROMPT 1022 audit findings preserved as report-only.
- PROMPT 1112 AC3 reserve-strip carry preserved.
- PROMPT 1114 card-display helper behaviour preserved.
- PROMPT 1118 hand B0004 Transform behaviour preserved.
- PROMPT 1119 bid-button phase-race behaviour preserved.

`1122: S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001: WORKER-DONE`
