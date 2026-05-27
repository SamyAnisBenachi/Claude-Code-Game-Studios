# PROMPT 1678 — BOT-PLACEMENT-LEGAL-UNIT-ACQUISITION-REPAIR

**Branch**: `worktree-1678-bot-placement-legal-unit-acquisition-repair`
**Commit**: `405be06f`
**Status**: SHIPPED

---

## Root Cause Diagnosis

### Issue 1: Empty placement batch rejected with `CardNotInHand` (Bug)

**File**: `server/src/feature/board/placement.rs` — `validate_submission_batch`

The function read `hands.hands.get(&player)` and returned `CardNotInHand` when the player was absent from `PlayerHands`, even for an empty placement batch (0 cards). Since bots never purchased cards, they had no entry in `PlayerHands`. The empty-batch submit — designed as the pre-Wave-3 fail-safe — was rejected instead of accepted.

**Consequence**: bot's `PlacementSubmitted` message was never written, so `submissions_received` was never updated for the bot. The RSM had to wait for the full placement timer (~10s) to fire instead of advancing when both players submitted.

Evidence: `server.log` line 93:
```
handle_placement_submission: submission rejected reason=CardNotInHand placements_len=0
```

### Issue 2: Bot never acquires any cards (Missing feature)

The bot emitted `DraftReadySignal` in `DraftInitial`/`DraftShop` phases but never purchased any cards. The acquisition system sent `S2CDraftOffering` to the bot (via `ShopRefreshTriggered`) but the bot had no mechanism to act on it — it just passed through the draft with an empty hand. `build_bot_placements` returned empty because `hand.is_empty()`.

Evidence: `bot-decision-log.jsonl` shows only `draft_ready` + `empty_placement_failsafe` entries, no card acquisitions. `heuristic_inputs_available=true` in the log confirms all placement heuristic resources were present.

---

## Fix 1: Empty-batch placement with absent hand entry

**File**: `server/src/feature/board/placement.rs`

Changed `validate_submission_batch` to treat absent `PlayerHands` entries as empty hands. An empty placement batch with an absent/empty hand is now accepted. Non-empty batches referencing cards not in the (absent = empty) hand still fail `CardNotInHand` via the per-card `hand.contains` check.

```rust
// Before: absent entry → CardNotInHand for ANY batch (including empty)
let Some(hand) = hands.and_then(|hands| hands.hands.get(&player)) else {
    return Some(PlacementSubmissionResult::CardNotInHand);
};

// After: absent entry treated as empty hand
let empty_hand: Vec<CardId> = Vec::new();
let hand: &Vec<CardId> = hands
    .and_then(|h| h.hands.get(&player))
    .unwrap_or(&empty_hand);
```

---

## Fix 2: `bot_draft_auto_pick` system (Wave 3.5)

**File**: `server/src/feature/bot/action_loop.rs`

Added a new `bot_draft_auto_pick` system that:
- Runs after `CardAcquisitionSet::Tick` (offering already in `ShopStates`)
- Runs before `bot_action_loop` (hand populated before the ready signal)
- For each bot in `DraftInitial`/`DraftShop`:
  - Collects candidate cards from the offering (`displayed_this_draft` for DraftInitial, `current_slots` for DraftShop)
  - Picks the cheapest affordable Minion via `pick_best_bot_card`, falling back to any affordable card
  - Calls `process_purchase_card_with_pool` — same authoritative path as human `C2SPurchaseCard`; handles hand-push + pool distribution + gold deduction atomically
  - Debounces per `(bot_id, round_number)` — at most one pick per draft phase per round

The `BotActionLoopPlugin` now registers both systems with explicit ordering:
```rust
(
    bot_draft_auto_pick.after(CardAcquisitionSet::Tick),
    bot_action_loop.after(bot_draft_auto_pick),
)
```

No new protocol messages. No special-case bypasses. No client changes.

---

## Tests Added

### `tests/unit/board-lane-system/placement_submit_authority_validation_test.rs` (+2 tests)

- `test_empty_batch_with_no_hand_entry_is_accepted_not_card_not_in_hand` — regression for Fix 1
- `test_non_empty_batch_with_no_hand_entry_is_still_card_not_in_hand` — invariant guard

### `server/src/feature/bot/action_loop.rs` (+4 unit tests)

- `pick_best_bot_card_prefers_cheapest_minion`
- `pick_best_bot_card_falls_back_to_any_when_no_minion`
- `pick_best_bot_card_respects_affordable_max`
- `pick_best_bot_card_empty_set_returns_none`

### Results

```
running 133 tests
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All 10 existing placement validation tests pass. All 5 existing wave-3 bot placement tests pass. 8 existing bot action loop tests pass.

---

## Scope Discipline

- `server/src/feature/board/placement.rs` — 7-line change (within allowed acquisition helpers)
- `server/src/feature/bot/action_loop.rs` — new system + tests (preferred scope)
- `tests/unit/board-lane-system/placement_submit_authority_validation_test.rs` — regression tests only

No client code, no production session state, no CI/config, no broad Cargo suite changes.

---

## Expected Soak Behavior After Fix

1. Bot receives draft offering during `DraftInitial`
2. `bot_draft_auto_pick` picks cheapest affordable Minion → calls `process_purchase_card_with_pool`
3. Bot hand now contains 1 card; `PlayerHands.hands[bot]` entry is created
4. Bot emits `DraftReadySignal`
5. Phase transitions to Placement
6. `build_bot_placements` finds 1 card in hand → assembles a legal `PlacedCardSubmit`
7. Bot submits non-empty placement; `handle_placement_submission` accepts it
8. RSM advances to Resolution without waiting for the 10s timer

1678: BOT-PLACEMENT-LEGAL-UNIT-ACQUISITION-REPAIR: SHIPPED
