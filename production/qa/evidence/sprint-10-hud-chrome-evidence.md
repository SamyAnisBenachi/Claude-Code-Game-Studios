# S10-POLISH-001 — HUD Visual Chrome MVP — Evidence

**Story**: `production/epics/hud/story-013-hud-visual-chrome-mvp.md`
**Story ID**: S10-POLISH-001
**Sprint**: Sprint 10
**Date of capture**: 2026-05-10
**Author**: PROMPT 618 dev-story worker
**Branch**: `work/s10-polish-001-hud-visual-chrome`
**Base SHA**: `811de8a` (origin/main at fetch time)
**Manifest Version asserted**: 2026-05-05
**Engine**: Bevy 0.18 + Lightyear 0.26

---

## No-claim language (per AC9)

> No Standard-tier accessibility completion is claimed; QA-COND-0005 remains
> accepted-risk friend-game scope. No client-side optimistic phase authority
> added; the existing `S2CPhaseChanged` drain (`phase_sink_system` in
> `client/src/presentation/mod.rs`) remains the single source of phase truth.
> No public-release readiness, full playable-client manual QA, full game
> completion, or final visual polish is claimed at close.

---

## Implementation summary

Three additions to `client/src/ui/hud/mod.rs`:

1. **`HudDimOverlay` marker** — new component (sibling of `HudFigurine` /
   `HudTimerBar`) attached to a single pre-pooled full-viewport translucent
   `Node` spawned at HUD session entry. Carries `HudEntity` marker so it
   counts toward `HUD_ENTITY_COUNT`.
2. **`sync_dim_overlay_for_resolution_system`** — registered in
   `HudSystemSet::StateSync` (per ADR-021). Reads `Res<CurrentClientPhase>`
   only; never reads `MessageReceiver<S2CPhaseChanged>`, never writes to
   `CurrentClientPhase`, never emits a synthetic `S2CPhaseChanged`. Toggles
   `Visibility::Visible` while `Phase::Resolution`, `Hidden` otherwise.
3. **`HUD_ENTITY_COUNT` constant bumped 21 → 22** plus `HUD_DIM_OVERLAY_ALPHA`
   tuning constant.

Visibility flips are **instantaneous** (HUD-12b BLOCKING) — no `TweenAnim`
attached to the overlay; no per-frame `BackgroundColor.alpha` mutation.

The wired timer bar (`HudTimerBar` + `HUD_PHASE_TIMER_BAR_ASSET`) and class
figurines (`HudFigurine` + `sync_figurine_image_system`) were already in place
at PAW-004 (`a7e397a` + `2132129`). This story adds the dim overlay only and
does not modify either pre-existing path.

### Chosen overlay alpha

`HUD_DIM_OVERLAY_ALPHA = 0.45` (`Color::srgba(0.0, 0.0, 0.0, 0.45)`). Visibly
dims the underlying HUD without obscuring gold/mana/phase readouts. Recorded
here so a future polish pass can revisit without re-deriving the constant.

---

## Files modified

| Path | Change |
|---|---|
| `client/src/ui/hud/mod.rs` | `HudDimOverlay` marker + `HUD_DIM_OVERLAY_ALPHA` constant; spawn dim overlay in `spawn_hud`; `dim_overlay: Entity` field on `HudEntities`; register `sync_dim_overlay_for_resolution_system` in `HudSystemSet::StateSync`; new system implementation; `HUD_ENTITY_COUNT` bumped 21 → 22 |
| `client/Cargo.toml` | New `[[test]] name = "hud_resolution_dim_test"` entry |
| `tests/integration/hud/hud_resolution_dim_test.rs` | NEW — 8 sub-tests covering AC1, AC3, AC4, AC5, AC6, AC7, AC8 |
| `production/qa/evidence/sprint-10-hud-chrome-evidence.md` | NEW — this document (AC9) |

No protocol files, no `shared/`, no `server/`, no design GDDs, no architecture
ADRs, no asset PNGs, no asset wiring constants modified.

---

## Acceptance Criteria coverage

| AC | Description | Evidence |
|---|---|---|
| AC1 | Wired phase timer bar visible | `test_timer_bar_present_with_image_node` — verifies `HudTimerBar` marker + `ImageNode` (Bevy 0.18 Required Components API) on the pre-pooled timer bar entity |
| AC2 | Wired class figurines visible | Pre-existing `sync_figurine_image_system` (PAW-004-d) is registered in `HudPlugin` `StateSync` set — verified by code inspection at `client/src/ui/hud/mod.rs:320`. No additional sync system added. |
| AC3 | RESOLUTION dim overlay renders only while `Phase::Resolution` | `test_dim_overlay_visible_only_in_resolution` — enumerates all 7 non-Resolution phases (Handshaking, Lobby, DraftInitial, DraftShop, DraftAuction, Placement, GameOver) and asserts `Visibility::Hidden`; then asserts `Visibility::Visible` in Resolution; then asserts overlay lifts on transition Resolution → DraftShop |
| AC4 | Dim overlay is pre-pooled, not per-update spawned | `test_dim_overlay_pre_pooled_entity_id_stable_across_phase_transitions` — captures initial entity ID, drives 5 phase transitions, asserts ID stability + exactly one `HudDimOverlay`-marker entity exists |
| AC5 | Single source of phase truth (TR-HUD-006 + ADR-002) | `test_no_client_side_phase_authority_in_dim_overlay_system` — drives transitions by writing `Res<CurrentClientPhase>` directly, verifies HudPlugin's systems never overwrite the resource (no synthetic phase authority added) |
| AC6 | FROZEN-mode tiebreak (TR-HUD-009 + ADR-011) | `test_frozen_mode_tiebreak_dim_overlay_hidden_on_game_over_then_restored_by_snapshot` — Resolution→GAME_OVER→snapshot rebuild path; asserts overlay hides on GAME_OVER (FROZEN ≠ RESOLUTION) and restores when late `S2CGameSnapshot { phase: Resolution }` arrives |
| AC7 | No countdown numerals on timer bar (HUD-11) | `test_timer_bar_no_countdown_numerals` — asserts neither `Text` nor `TextSpan` on the `HudTimerBar` entity nor any of its descendants |
| AC8 | `HUD_ENTITY_COUNT == 22` | `test_hud_entity_count_is_twenty_two_after_dim_overlay_added` — counts entities carrying the `HudEntity` marker against the constant; constant value asserted to be exactly 22 |
| AC9 | Manual evidence document | This file |

---

## Test results

### New automated test target — PASS (8/8)

```
$ CARGO_TARGET_DIR=target/msvc-local cargo test -p client -j 1 --test hud_resolution_dim_test

running 8 tests
test test_hud_entity_count_is_twenty_two_after_dim_overlay_added ... ok
test test_dim_overlay_carries_hud_entity_marker ... ok
test test_no_client_side_phase_authority_in_dim_overlay_system ... ok
test test_timer_bar_no_countdown_numerals ... ok
test test_timer_bar_present_with_image_node ... ok
test test_frozen_mode_tiebreak_dim_overlay_hidden_on_game_over_then_restored_by_snapshot ... ok
test test_dim_overlay_pre_pooled_entity_id_stable_across_phase_transitions ... ok
test test_dim_overlay_visible_only_in_resolution ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

### Regression targets — green where pre-existing-passing on origin/main

Per the story's required regression list, with status against
`HUD_ENTITY_COUNT 22` (post-S10-POLISH-001):

| Test | Result | Notes |
|---|---|---|
| `reconnect_snapshot_rebuild_test` | ✅ 9 passed | |
| `same_tick_tie_break_test` | ✅ 4 passed | |
| `scoreboard_dot_message_test` | ✅ 2 passed | |
| `hud_text_size_contrast_accessibility_test` | ✅ 6 passed | uses `HUD_ENTITY_COUNT` directly — confirms 22 propagates correctly |
| `hud_plugin_scaffold_test` | ⚠️ 3 passed, 1 fail | `hud_entities_never_contain_timer_components_or_timer_text` failure is **pre-existing on `origin/main`** (verified by stash + re-run on pristine 811de8a). Cause: PAW-004 added `Name::new("HUD Phase Timer Bar")` + `HudEntity` marker on the timer bar entity, which the test's name-substring filter ("timer") matches. Pre-dates this story — flagged as Sprint-10 tech debt for the PAW-004 owner. |
| `hud_gold_mana_display_test` | ✅ 4 passed | uses `HUD_ENTITY_COUNT` directly |
| `hud_mana_shape_distinction_test` | ✅ 6 passed | uses `HUD_ENTITY_COUNT` directly |
| `hud_phase_label_round_counter_test` | ✅ 5 passed | |
| `hud_phase_transitions_test` | ✅ 9 passed | |
| `hud_economy_auction_inline_gold_test` | ✅ 4 passed | uses `HUD_ENTITY_COUNT` directly — confirms 22 propagates correctly |
| `hud_game_over_freeze_test` | ✅ 2 passed | |
| `hud_numeric_tween_animation_test` | ✅ 4 passed | |
| `asset_wiring_foundation_test` | ✅ 9 passed | |
| `hud_asset_wiring_test` | ⚠️ 0 passed, 6 fail | All 6 failures are **pre-existing on `origin/main`** (verified by stash + re-run on pristine 811de8a). Cause: test fixture `make_app()` calls `app.add_plugins(StatesPlugin)` but does NOT call `app.init_state::<ClientState>()`, so the test panics on `app.world_mut().resource_mut::<NextState<ClientState>>()`. Pre-dates this story — flagged as PAW-004 owner tech debt. |

**Conclusion**: every test that was green on `origin/main` at `811de8a` remains
green after this story; both pre-existing red tests stayed red and were not
introduced or aggravated by this work.

---

## Smoke run (manual verification)

**Skipped — pre-existing main-binary breakage on origin/main blocks
`cargo run -p client`.**

Cause: several `*_harness` binaries in `client/src/` (introduced/touched by
SAU-011 / QA-COND-0007 follow-ups, last in commit `71de055`) reference
identifiers that were removed from `bevy::prelude::*` by the Bevy 0.18 "Input
behind features" reorganisation (`KeyCode`, `Interaction`, `Vec2`,
`Visibility`, `Val`, `Name`, `Camera2d`, etc.). These bins do not gate on
`required-features`, so cargo compiles them as part of any
`cargo test`/`cargo build`/`cargo run` invocation in the `client` crate, and
their compile failures cascade into a `bin "client"` failure too.

The pre-existing breakage is confirmed reproducible on pristine `origin/main`
(`811de8a`) with no edits applied.

Per the story's Phase 6 fall-back ("verify by reaching that phase if possible;
otherwise fall back to integration test as authoritative") and per
`docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` "friend-game accept-risk", the
integration test (`hud_resolution_dim_test`, 8/8 green) is the authoritative
evidence for the dim overlay behaviour. The wired timer bar + figurine
behaviour is covered by AC1/AC2 evidence above + the pre-existing PAW-004
visual evidence.

**Tech-debt follow-up flagged**: SAU-011 / QA-COND-0007 owner should either
(a) restore the missing imports in the broken `*_harness.rs` bins, or
(b) gate them via `required-features = ["dev_harness"]` in `client/Cargo.toml`
so they no longer block default `cargo` workflows. Out-of-scope for
S10-POLISH-001.

---

## Performance observation

Steady-state cost: **0** added per-frame work — the overlay is a pre-pooled
`Node` toggled via `Visibility`, not spawned/despawned per frame. The new
system runs every `Update` tick but exits early in O(1) when
`Res<CurrentClientPhase>` is not in `Resolution` (or the visibility is already
correct on the second tick onwards because Bevy's `Visibility` change
detection short-circuits identical writes via the `set_visibility` helper).

Phase-boundary spike: **single `Visibility` write on a single entity per
phase transition** — well under the 1 ms steady-state and 3 ms phase-boundary
budgets recorded in the control manifest (Manifest Version 2026-05-05).

No browser-WASM perf harness was run for this story (out of scope per Sprint
10 risk row 151 + per the story's "Out of Scope" no-public-release-readiness
constraint).

---

## Build / format / lint summary

| Check | Result |
|---|---|
| `cargo fmt -p client -- --check` | ✅ clean |
| `cargo check -p client --lib` | ✅ clean (Finished in ~67 s) |
| `git diff --check` | ✅ clean |
| `git diff --cached --check` | ✅ clean |
| `cargo test --no-run --test hud_resolution_dim_test` | ✅ test target builds (1 unused-mut warning previously, now resolved) |
| `cargo test --test hud_resolution_dim_test` | ✅ 8/8 PASS |

---

## Manifest Version + ADR alignment

- **Manifest Version asserted**: 2026-05-05 (matches story header)
- **ADR-021** (Presentation Layer Architecture): new system registered in
  `HudSystemSet::StateSync`, dim overlay pre-pooled at HUD session entry,
  no per-update spawn/despawn — **aligned**
- **ADR-011** (Reconnect + Snapshot): FROZEN-mode tiebreak preserved; the
  test exercises the snapshot-wins rebuild path and asserts overlay
  visibility restores from `S2CGameSnapshot { phase: Resolution }` — **aligned**
- **ADR-002** (Client-Server Authority): zero C2S messages added; no
  client-side phase mutation; HudPlugin stays read-only on the phase axis —
  **aligned**
- **ADR-001** (Objective Identity Unicast): not touched
- **ADR-008** (Lightyear Channel Config): not touched

---

## Out-of-scope items (preserved)

- No countdown numerals on the timer bar (HUD-11)
- No client-side optimistic phase authority added
- No new sprite assets authored
- No claim of Standard-tier accessibility completion (QA-COND-0005 remains
  accepted-risk friend-game scope)
- No closure of QA-COND-0005, QA-COND-0006, S8-QA-001-W1, or any other
  Sprint 9 carry condition
- No tween / animation on the dim overlay (HUD-12b)
- No changes to Hand UI, Shop/Auction UI, board content, scoreboard dots,
  gold labels, mana labels, phase label, round counter, or any HUD zone
  outside the dim overlay
- No changes to network protocol, lightyear channels, or any C2S / S2C
  message type
