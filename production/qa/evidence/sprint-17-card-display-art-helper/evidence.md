# Evidence — S17-UI-CARD-DISPLAY-ART-HELPER-001 (PROMPT 1113)

> **Story**: `production/epics/ui-clean-pass/story-017-card-display-art-helper-bundle.md`
> **Worker branch**: `work/s17-card-display-art-helper`
> **Base**: `origin/main@f2ba917` (PROMPT 1112 paperwork tip; strict
> fast-forward descendant of PROMPT 1111 PARTIAL integration and PROMPT
> 1110 / 1108 / 1107 / 1106 prior story-done / integration tips).
> **Sprint**: Sprint 17 Must Have row (NOT activated by this worker;
> `production/sprint-status.yaml` / `production/sprints/sprint-17.md` /
> `production/stage.txt` / `production/session-state/*` UNTOUCHED).
> **Generated**: 2026-05-18 by PROMPT 1113 `/dev-story`.

## SOURCE-1077 findings absorbed by this bundle

| Source | Severity | Status |
|---|---|---|
| SOURCE-1077-01 | P0 — slot-well chrome lost when card art missing | **FIXED** |
| SOURCE-1077-02 | P0 — duplicate `apply_card_display_art` definitions | **FIXED** |
| SOURCE-1077-03 | P1 — `Box::leak` per render in `resolve_card_display_art` | **FIXED** |
| SOURCE-1077-04 | P1 — `resolve_card_display_art` returns path without existence check | **FIXED** |

## Chrome-preservation strategy (recorded per AC2 / AC6)

The implementing worker chose the **"do-not-touch-`ImageNode`-on-Err/Clear"**
strategy from the three options listed in the story:

> "separate child entity, or distinct `CardArtImageNode` component, or
>  `PlaceholderAssets.shop_slot_well_idle` fallback in the Err branch"

Concretely:

- `apply_card_display_art` **Ok** branch: inserts `ImageNode(card_art_path)`
  + `CardDisplayArtAsset { path }`; removes `CardDisplayArtFallback`.
- `apply_card_display_art` **Err** branch: inserts
  `CardDisplayArtFallback { reason }`; removes `CardDisplayArtAsset`. The
  slot's `ImageNode` component is **not** touched — the spawn-time chrome
  (e.g. shop slot's `SHOP_SLOT_WELL_IDLE_ASSET` well) survives.
- `clear_card_display_art`: removes both `CardDisplayArtAsset` and
  `CardDisplayArtFallback`. The slot's `ImageNode` is again **not**
  touched — the spawn-time chrome survives slot vacate.

Trade-off documented: in a rare Ok → Err transition the stale Ok card-art
`ImageNode` handle persists until the next batch refresh; this is
acceptable because the audit bug (SOURCE-1077-01) is about the
spawn-time chrome being lost, not stale art between rounds. The next
batch refresh on `S2CShopSlots` re-issues `apply_card_display_art` for
every slot and overwrites the stale handle.

## AC coverage

| AC | Status | Evidence |
|---|---|---|
| AC1 single owner | ✅ | `grep -rn "fn apply_card_display_art" client/src/ shared/src/` → 1 match (`client/src/asset_wiring.rs`). Same for `clear_card_display_art`. |
| AC2 chrome survives missing art | ✅ | `shop_slot_chrome_survives_missing_card_art_apply` in `card_display_art_chrome_preservation_test.rs`. |
| AC3 no `Box::leak` | ✅ | `grep -rn "Box::leak" client/src/ shared/src/` → 0 matches; resolver returns `String`. |
| AC4 existence check probe | ✅ | `probe_card_display_art_paths` registered on `OnEnter(ClientState::InSession)`; emits `warn!` with `art_id` + `path` per missing file; counts in `MissingCardArtWarnings`. |
| AC5 happy-path apply preserves card-art + chrome | ✅ | `shop_slot_happy_path_apply_sets_card_art_binding`. Hand fan slot subtree (`HandCardFrame` child) keeps its chrome `ImageNode` independently — covered by existing `hand_ui_asset_wiring_test.rs`. |
| AC6 clear preserves chrome | ✅ | `shop_slot_chrome_survives_clear`. |
| AC7 `missing` sentinel routes through placeholder | ✅ | `resolve_missing_sentinel_routes_to_placeholder` (unit) + `missing_sentinel_resolves_to_placeholder_via_apply` (integration). |
| AC8 unit tests | ✅ | `tests/unit/asset_wiring/card_display_art_helper_test.rs` — 6/6 pass. |
| AC9 integration tests | ✅ | `tests/integration/presentation/card_display_art_chrome_preservation_test.rs` — 8/8 pass. |
| AC10 fixture art-id coverage | ✅ | `probe_does_not_warn_for_documented_missing_sentinel` + `probe_records_warning_count_resource_on_session_entry` (resource observable). |
| AC11 schedule preserved | ✅ | No new `SystemSet` added; helper consumers' `PresentationSet` placement unchanged; new probe slots into existing `OnEnter(InSession)` schedule. |
| AC12 no protocol / server change | ✅ | `git diff` touches no `server/`, `shared/`, `tests/integration/server/`, `tests/unit/server/`. |
| AC13 no accept-risk closure | ✅ | This evidence file does not claim closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other accept-risk disposition. |
| AC14 sprint disposition preserved | ✅ | `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/sprint-status.yaml`, `production/stage.txt`, `production/session-state/*`, `production/qa/*`, `production/gate-checks/*`, and `docs/architecture/adr-*.md` UNTOUCHED by this worker. |
| AC15 worker branch scope contained | ✅ | Files changed: `client/src/asset_wiring.rs`, `client/src/ui/shop_auction/mod.rs`, `client/src/ui/hand/mod.rs`, `client/Cargo.toml` (test bin registration only; no feature flag change), 2 new test bins + 4 existing test files adjusted for the `&'static str` → `String` resolver signature. Worker branch is `work/s17-card-display-art-helper`; `main` not pushed. |
| AC16 Cargo resource policy | ✅ | All `cargo check` / `cargo test` invocations ran under `$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`. Disk preflight: D: free 800.5 GB at preflight (well above the 50 GB minimum). |

## Cargo invocations + results

All ran under the policy env vars above.

```
cargo check -p client                                                 → OK
cargo check -p client --tests                                         → OK
cargo test -p client --test card_display_art_helper_test              → 6/6 pass
cargo test -p client --test card_display_art_chrome_preservation_test → 8/8 pass
cargo test -p client --test shop_auction_ui_shop_panel_test           → 10/10 pass
cargo test -p client --test shop_auction_ui_auction_activation_test   → 8/8 pass
cargo test -p client --test hand_ui_draft_initial_grid_test           → 6/6 pass
cargo test -p client --test playable_client_draft_shop_hand_bridge_test → 5/5 pass
cargo test -p client --test asset_wiring_foundation_test              → 9/9 pass
cargo test -p client --test hand_ui_asset_wiring_test                 → 10/10 pass
cargo test -p client --test shop_auction_asset_wiring_test            → 5/5 pass
cargo test -p client --test shop_auction_ui_card_cost_combat_stat_rendering_test → 8/8 pass
cargo test -p client --test ui_clean_pass_card_slot_primitive_test    → 27/27 pass

git diff --check                                                      → clean
```

## Files touched by this worker

```
client/Cargo.toml
client/src/asset_wiring.rs
client/src/ui/hand/mod.rs
client/src/ui/shop_auction/mod.rs
tests/integration/hand-ui/draft_initial_grid_test.rs
tests/integration/playable_client/draft_shop_hand_bridge_test.rs
tests/integration/presentation/card_display_art_chrome_preservation_test.rs  (NEW)
tests/integration/shop_auction_ui/auction_activation_test.rs
tests/integration/shop_auction_ui/shop_panel_test.rs
tests/unit/asset_wiring/card_display_art_helper_test.rs                       (NEW)
production/qa/evidence/sprint-17-card-display-art-helper/evidence.md          (NEW — this file)
```

Cargo.toml change is two new `[[test]]` entries that register the two new
test bins — no feature flag, no dependency, no feature surface change.

## Explicitly NOT claimed

- Sprint 17 close-out.
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`.
- Closure of any AUDIT-1076-* finding or any SOURCE-1077-* finding
  outside the four bundled here.
- Public release readiness; release-candidate readiness; full game
  completion; broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA; two-client
  GAME_OVER closure; final-art / asset-production completion; Polish ->
  Release gate-check retry; stage advance from Polish to Release.
- Per-surface card-slot primitive migration of any consumer surface
  (HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST).
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked Must Have
  carry; not closed by this row.
- 24 PROMPT 1022 audit findings; report-only, not closed by this row.
- Sprint 16 `closed-with-conditions` disposition; preserved verbatim.
- AC3 hand reserve-strip carry from PROMPT 1112; not closed by this row.

## Reporting

Final status line (relay convention):

```
1113: S17-UI-CARD-DISPLAY-ART-HELPER-001: DONE
```
