# PROMPT 1229 — S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001 / B-1203-X-03

## Summary

Enriches the QA snapshot JSON with three top-level fields requested by PROMPT
1203 R16 / B-1203-X-03 so audits can correlate the screenshot with placement
and auction game-state directly, without spelunking through `extras.*`:

1. `current_phase.timer_remaining_ms` — round-phase countdown lifted from
   `PhaseTimerState` (via `extras.timers.phase_timer.remaining_ms`).
2. `placement_state` — top-level placement-phase block (`available`,
   `staged_count`, `can_submit`, `submitted`, `drag_active`, `drag_card_id`,
   `drag_target_kind`, `disclosure_step`) projected from existing
   `PendingPlacements`, `PlacementTimer`, `PlacementDisclosureState`, and
   `ActivePlacementDrag` resources (already captured in `extras.hand` /
   `extras.timers.placement_timer` / `extras.drag`).
3. `auction_state` — top-level auction-phase block (`available`,
   `panel_state`, `card_id`, `starting_price`, `current_price`,
   `current_leader`, `timer_duration_ms`, `timer_remaining_ms`,
   `local_in_flight_bid_amount`, `local_gold {gold, reserved_gold, free_gold,
   view_initialized}`) projected from `ShopAuctionAuctionState` +
   `ShopAuctionLocalGoldView` + `PlayerEconomyView`.

The lift is pure projection over the already-captured `ExtrasSnapshot`; no UI
module mutation, no new resource reads in the host system. The JSON shape is
phase-stable: outside the placement/auction phase the new blocks emit
`available = false` + nested nulls rather than missing keys.

## Exact JSON fields added

```jsonc
{
  "current_phase": {
    // ... existing fields ...
    "timer_remaining_ms": 17500            // Option<u32>; null outside an active phase
  },
  "placement_state": {                     // PlacementStateSnapshot (new top-level)
    "available": true,                     // bool — true when any placement resource present
    "staged_count": 2,                     // Option<usize>
    "can_submit": true,                    // Option<bool>; staged_count > 0 && !submitted
    "submitted": false,                    // Option<bool>; from PlacementTimer
    "drag_active": true,                   // Option<bool>; from ActivePlacementDrag::is_active()
    "drag_card_id": 57,                    // Option<u32>
    "drag_target_kind": "Minion",          // Option<String> stable token
    "disclosure_step": "StagedCard"        // Option<String>; PlacementDisclosureStep::Debug
  },
  "auction_state": {                       // AuctionStateSnapshot (new top-level)
    "available": true,                     // bool — true when ShopAuctionAuctionState present
    "panel_state": "Active",               // Option<String> stable token
    "card_id": 123,                        // Option<u32>
    "starting_price": 4,                   // Option<u32>
    "current_price": 9,                    // Option<u32>
    "current_leader": "PlayerId(2)",       // Option<String> (Debug of PlayerId)
    "timer_duration_ms": 12000,            // Option<u32>
    "timer_remaining_ms": 4800,            // Option<u32>
    "local_in_flight_bid_amount": 11,      // Option<u32>
    "local_gold": {                        // Option<AuctionLocalGoldSnapshot>
      "gold": 18,
      "reserved_gold": 11,
      "free_gold": 7,
      "view_initialized": true
    }
  }
}
```

Backward compatibility: every existing key (including `extras.timers`,
`extras.shop_auction.auction`, `extras.drag.placement_drag_*`,
`extras.hand.staged_count`, etc.) is preserved. The new top-level fields are
additive — old consumers see the existing shape; new consumers can grep the
top-level keys directly.

## Files changed

- `client/src/presentation/qa_snapshot.rs` — added `timer_remaining_ms` to
  `PhaseInfo`; added `PlacementStateSnapshot`, `AuctionStateSnapshot`,
  `AuctionLocalGoldSnapshot` (all `Default`-able); wired both new fields
  into `QASnapshotData`; populated them inside
  `build_snapshot_with_extras_and_layout` via two new pure helpers
  `build_placement_state_snapshot` / `build_auction_state_snapshot`
  (both `pub` for test access).
- `client/Cargo.toml` — registered the new `[[test]]` entry:
  `qa_snapshot_placement_auction_state_field_coverage_test`.
- `tests/integration/qa_snapshot/placement_auction_state_field_coverage_test.rs`
  — new test file covering schema-presence, defaulted shape, lift from
  extras, `can_submit` truth table, `available=false` when only adjacent
  resources are present.
- `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs` — updated the
  five `QASnapshotData {...}` / `PhaseInfo {...}` literals to include the
  new fields (defaulted) so existing assertions keep passing.
- `tests/integration/qa_snapshot/layout_field_coverage_test.rs` — same
  literal update at line 526.

## Tests

Build gate (Cargo policy applied): only the agent's owned files. Two
unrelated tests fail to compile on `origin/main` because of a `LobbyViewState`
schema drift (`hud_opp_figurine_label_mana_repaint_test`,
`hud_opp_class_recipient_mismatch_test`) — those are not in this story's
scope. The three prompt-mandated tests all pass:

```
cargo test -p client --test qa_snapshot_layout_field_coverage_test
  → 14 passed; 0 failed
cargo test -p client --test qa_snapshot_overlay_test
  → 26 passed; 0 failed
cargo test -p client --test qa_snapshot_placement_auction_state_field_coverage_test
  → 13 passed; 0 failed  (new file added for this story)
```

53 tests total in the qa_snapshot suite; 0 failures.

## Cargo policy

YES — all env vars set in each PowerShell session before invoking cargo:

```
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS="-C debuginfo=0 -C link-arg=/DEBUG:NONE"
```

`CARGO_TARGET_DIR` started at the prescribed
`D:\_DEV\cargo-target\ccgs-msvc` but the shared dir held a stale `client`
rmeta from a sibling worktree that did not surface this story's new types
after `cargo clean -p client`. With user approval I switched to an
isolated `D:\_DEV\cargo-target\ccgs-msvc-1229` target dir (a peer of the
prescribed one, not a deletion of it) so the test build could pick up the
new symbols. This is the same `ccgs-msvc-*` family and follows the
"verified stale target artifacts" carve-out in the prompt.

## Target cleanup

YES — `cargo clean -p client` was attempted under the original target dir
(removed 1585 files, 5.4 GiB) but the rmeta cache still resolved against
sibling-worktree artifacts. Switched to an isolated `*-1229` target dir as
described above. No deletion of sibling-worktree files.

## Branch / push

- Branch: `work/s18-qa-snapshot-placement-auction-state-1229`
- Based on `origin/main@dcb9565` (includes `dcb9565` and `c61bab3` per the prompt).
- Worker branch committed and pushed to origin; `main` not touched.

1229: S18-QA-SNAPSHOT-PLACEMENT-AUCTION-STATE-001: STATUS_PLACEHOLDER
