# S17-UI-BID-BUTTON-PHASE-RACE-001 — Evidence

> Story: `production/epics/shop-auction-ui/story-019-bid-button-phase-race.md`
> Worker prompt: PROMPT 1116
> Worker branch: `work/s17-bid-button-phase-race`
> Base: `origin/main@30c9e0f` (PROMPT 1114 integration tip — card-display
> art-helper merged into main).
> Engine: Bevy 0.18 + `liv-bevy-018` skill. No Lightyear edits.

## Summary

Source-side remediation for `SOURCE-1077-10` (PROMPT 1077 P2 audit
finding). At the entity level the three auction bid buttons now:

1. Spawn with `Text::new(AUCTION_BID_BUTTON_LOADING_LABEL)` (=
   `"Loading…"`) instead of `Text::new("")`.
2. While `auction_state.card_id.is_none()` (the phase-entry race window
   between DraftAuction phase entry and the first `S2CAuctionCard`
   drain), the per-button `Text` is the pending label — the misleading
   numeric `BidButtonLabel` ("0g\n(+1)") is not surfaced.
3. While `AuctionBidButtonState::HiddenLeading` is active (local player
   leads the auction), the `ImageNode.image` is
   `Handle::<Image>::default()` — the baked-`?`
   `ui_bid_button_disabled.png` chrome is not loaded onto the entity.
   `Visibility::Hidden` continues to keep the row off-screen.
4. PROMPT 1042 Pass affordance preserved verbatim — `auction_bid_chrome_state`'s
   `Normal` (Enabled) and `Disabled` (every other non-`HiddenLeading`
   variant) mappings are unchanged. Only the new `HiddenLeading => None`
   branch was added.

The placeholder PNG `assets/art/ui/auction/ui_bid_button_disabled.png`
is preserved verbatim per `PAW-TD-*-a` accept-risk. No real-art
replacement, no new placeholder PNG authored, no
`BID_BUTTON_HIDDEN_LEADING_ASSET` constant introduced —
`Handle<Image>::default()` is the AC3 (b) "no-image asset" branch.

## Files changed

| Path | Change |
|------|--------|
| `client/src/ui/shop_auction/mod.rs` | New `pub const AUCTION_BID_BUTTON_LOADING_LABEL`. Spawn-state `Text::new("")` → `Text::new(AUCTION_BID_BUTTON_LOADING_LABEL)`. `sync_auction_panel_system` text branch surfaces the pending label when `card_id.is_none()`. Chrome apply site falls back to `Handle::default()` when `auction_bid_chrome_state` returns `None`. `auction_bid_chrome_state` now returns `Option<BidButtonChromeState>` with `HiddenLeading => None`. |
| `client/Cargo.toml` | New `[[test]]` entry registering `shop_auction_ui_auction_bid_buttons_phase_race_test`. Test registration only — no other Cargo edits. |
| `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` | New integration test bin covering AC1, AC2, AC3, AC4, AC5, AC7. Five `#[test]` functions, all pass against the modified client. |
| `production/qa/evidence/sprint-17-bid-button-phase-race/evidence.md` | This document. |

`assets/art/ui/auction/ui_bid_button_disabled.png` — **NOT** modified.
`client/src/asset_wiring.rs` — **NOT** modified (no new fallback
constant required; `Handle::default()` covers the chrome override).

## Cargo resource policy

Every Cargo invocation in this session used the binding policy:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

(Bash-export form used at the shell; identical effect.) `D:` free space
≈ 774 GB at session start — no `cargo clean` invoked.

## Verification

### `cargo check -p client`

```
$ cargo check -p client
    Checking client v0.1.0 (D:\_DEV\claude-code-game-studios-worktrees\s17-bid-button-phase-race\client)
    Finished `dev` profile [optimized] target(s) in 6.47s
```

PASS.

### New test bin — `shop_auction_ui_auction_bid_buttons_phase_race_test`

```
$ cargo test -p client --test shop_auction_ui_auction_bid_buttons_phase_race_test
   Compiling client v0.1.0 (...)
    Finished `test` profile [optimized] target(s) in 1m 00s
     Running ..\tests\integration\shop_auction_ui\auction_bid_buttons_phase_race_test.rs

running 5 tests
test s17_phase_race_ac1_spawn_state_text_is_loading_label ... ok
test s17_phase_race_ac4_draft_auction_without_card_keeps_loading_or_hidden ... ok
test s17_phase_race_ac7_chrome_mapping_preserved_for_enabled_and_disabled_states ... ok
test s17_phase_race_ac3_ac5_hidden_leading_clears_chrome_and_hides_row ... ok
test s17_phase_race_ac2_text_updates_to_numeric_on_auction_card_arrival ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

PASS — every AC has at least one assertion exercising it.

### Adjacent bid-button regression bins

```
$ cargo test -p client --test shop_auction_ui_auction_bid_buttons_test \
    --test shop_auction_ui_auction_activation_test \
    --test shop_auction_ui_chrome_wiring_test \
    --test shop_auction_ui_auction_bid_target_focus_test \
    --test shop_auction_ui_auction_feedback_test \
    --test shop_auction_ui_auction_lead_loss_state_test \
    --test shop_auction_ui_auction_settlement_test

shop_auction_ui_auction_activation_test:        8 passed
shop_auction_ui_auction_bid_buttons_test:       9 passed
shop_auction_ui_auction_bid_target_focus_test:  4 passed
shop_auction_ui_auction_feedback_test:          6 passed
shop_auction_ui_auction_lead_loss_state_test:   4 passed
shop_auction_ui_auction_settlement_test:        7 passed
shop_auction_ui_chrome_wiring_test:             4 passed

total: 42 passed; 0 failed
```

PASS. Notably:

- `prompt_1042_bid_buttons_and_pass_render_concrete_labels` — Pass
  affordance + numeric labels intact (AC6 / PROMPT 1042 carry).
- `prompt_1042_pass_button_toggles_local_pass_state_without_outbound_bid`
  — Pass toggle preserved.
- `prompt_1042_pass_resets_when_new_auction_card_arrives` — Pass reset
  on new card preserved.
- `prompt_1042_unaffordable_bid_has_distinct_visual_state` — Affordance
  visual state preserved.
- `sau_005_local_leader_hides_bid_buttons_and_shows_badge` —
  HiddenLeading visibility contract preserved.
- `shop_auction_bid_buttons_carry_non_default_image_node_after_on_enter_in_session`
  — InSession-idle bid-button ImageNode wiring preserved (the chrome
  override is narrowed to `HiddenLeading` only; idle InSession state
  is `GenericDisabled` with `card_id == None`, so the chrome mapper
  returns `Some(Disabled)` and the existing Disabled handle continues
  to be applied).

### `git diff --check`

```
$ git diff --check
(no output)
```

PASS — no whitespace errors.

## Acceptance criteria

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 — spawn-state text non-empty meaningful | PASS | `s17_phase_race_ac1_spawn_state_text_is_loading_label` asserts every bid button carries `"Loading…"` immediately after `OnEnter(InSession)`. |
| AC2 — text updates to numeric on `S2CAuctionCard` drain | PASS | `s17_phase_race_ac2_text_updates_to_numeric_on_auction_card_arrival` asserts the pre/post-card sequence: pending → `["1g\n(+1)", "3g\n(+3)", "5g\n(+5)"]`. |
| AC3 — `HiddenLeading` chrome / visibility override | PASS | `s17_phase_race_ac3_ac5_hidden_leading_clears_chrome_and_hides_row` asserts `Visibility::Hidden` AND `ImageNode.image.id() == Handle::<Image>::default().id()` for all three buttons. Strategy (a) **and** (b) of AC3 are both held — chosen "belt-and-braces" so the entity contract is clean even if visibility is later flipped by a sibling refactor. |
| AC4 — `?` glyph not surfaced during phase-entry race | PASS | `s17_phase_race_ac4_draft_auction_without_card_keeps_loading_or_hidden` enters DraftAuction without draining `S2CAuctionCard` and asserts each button is `Visibility::Hidden` **or** carries `"Loading…"` + non-`?` chrome. Numeric `"0g\n(+1)"` is explicitly forbidden. |
| AC5 — `?` glyph not surfaced during `HiddenLeading` | PASS | Same test as AC3: `ImageNode.image` is `Handle::default()` → the baked-`?` PNG is not loaded onto the entity, and `Visibility::Hidden` keeps the row off-screen. |
| AC6 — PROMPT 1042 Pass affordance preserved | PASS | `prompt_1042_bid_buttons_and_pass_render_concrete_labels`, `prompt_1042_pass_button_toggles_local_pass_state_without_outbound_bid`, `prompt_1042_pass_resets_when_new_auction_card_arrives`, `prompt_1042_unaffordable_bid_has_distinct_visual_state` — all PASS. |
| AC7 — `auction_bid_chrome_state` Normal / Disabled mappings preserved | PASS | `s17_phase_race_ac7_chrome_mapping_preserved_for_enabled_and_disabled_states` asserts the per-entity chrome path is `BID_BUTTON_NORMAL_ASSET` for Enabled and `BID_BUTTON_DISABLED_ASSET` for Unaffordable. Function-level: only the `HiddenLeading => None` branch was added; `Enabled => Some(Normal)` and `_ => Some(Disabled)` are semantically unchanged from the prior `Enabled => Normal` / `_ => Disabled`. |
| AC8 — `ui_bid_button_disabled.png` unchanged | PASS | `git diff origin/main..HEAD -- assets/` is empty (the only `assets/` reference is the asset path constant in `client/src/asset_wiring.rs`, which is **not** modified). |
| AC9 — integration test bin authored | PASS | `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` — five `#[test]` fns covering AC1, AC2, AC3, AC4, AC5, AC7. Registered in `client/Cargo.toml` as `shop_auction_ui_auction_bid_buttons_phase_race_test`. |
| AC10 — no protocol or server change | PASS | `git diff origin/main..HEAD -- server/ shared/ tests/integration/server/` is empty. Client-side only. |
| AC11 — ADR-021 schedule preserved | PASS | `cargo check -p client` clean. No new `add_systems` calls, no new system-set, no new schedule wiring — the edit is inside the existing `sync_auction_panel_system` body. |
| AC12 — no accept-risk closure claimed | PASS | This document and the commit message explicitly do NOT claim closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `A11Y-ST-12`, any AUDIT-1076-* finding, any SOURCE-1077-* finding outside SOURCE-1077-10, or final-art replacement of the baked-`?` PNG. |
| AC13 — Sprint 17 disposition preserved | PASS | `git diff origin/main..HEAD` touches zero files under `production/sprint-status.yaml`, `production/sprints/`, `production/stage.txt`, `production/session-state/`, `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/`, or `docs/architecture/adr-*.md`. |
| AC14 — worker branch scope contained | PASS | Branch `work/s17-bid-button-phase-race` from `origin/main@30c9e0f`. Files changed: `client/src/ui/shop_auction/mod.rs`, `client/Cargo.toml` (test registration only), `tests/integration/shop_auction_ui/auction_bid_buttons_phase_race_test.rs` (new), this evidence doc. No `main` push. |
| AC15 — Cargo resource policy applied | PASS | See "Cargo resource policy" section above. |

## Strategy choice — chrome override

AC3 offered two strategies for `HiddenLeading`: (a) `Visibility::Hidden`
or (b) transparent chrome ImageNode + non-rendering text. Chosen
**both** simultaneously:

- (a) Visibility was already in place (`visibility_for(footer_visible
  && next_state != HiddenLeading)`); preserved.
- (b) `ImageNode.image = Handle::<Image>::default()` is now also
  applied — added defensively. Rationale: a future refactor that flips
  the visibility logic (e.g. for a "leader badge inline with the bid
  row" layout) would otherwise reintroduce the `?` glyph onto the
  visible entity. The Handle::default() override removes that risk at
  the entity level.

The override is narrowed to `HiddenLeading` so `chrome_wiring_test`'s
"InSession-idle bid buttons must have non-default ImageNode" assertion
continues to hold — InSession idle state is `GenericDisabled` with
`card_id == None`, which the chrome mapper sends to `Some(Disabled)`.

## Non-claims

- No story-done. No sprint-status / session-state / stage.txt edits.
- No smoke / team-qa / gate-check / release-check.
- No final-art replacement claim. `ui_bid_button_disabled.png`
  preserved verbatim.
- No accessibility / hit-target / playtest-validation claim.
- No closure of any SOURCE-1077-* finding outside SOURCE-1077-10. No
  closure of any AUDIT-1076-* finding. No closure of any of the 24
  PROMPT 1022 audit findings.
- No `main` push.
