# PROMPT 1701 — BOT-SOAK-NONEMPTY-PLACEMENT-VERIFY-INTEGRATION

**Date**: 2026-05-27
**Integration branch**: `integrate/1701-bot-soak-nonempty-placement-verify`
**Source branch**: `origin/worktree-1692-bot-soak-trigger-nonempty-placement`
**Merge base**: `f9324431` (origin/main HEAD — FF-ready)

---

## Objective

Verify and integrate PROMPT 1692 onto latest origin/main. Confirm the trigger
picks an affordable Minion and submits non-empty `PlacedCardSubmit` when mana
allows. Preserve the PROMPT 1678 empty-batch contract.

---

## Merge Base Confirmation

```
git merge-base origin/main origin/worktree-1692-bot-soak-trigger-nonempty-placement
→ f9324431  (= origin/main HEAD)
```

The 1692 branch diverges directly from the current main tip — **no rebase needed**,
fast-forward is safe.

---

## Path Allowlist Review

All changes are confined to owned scope:

| File | Status |
|------|--------|
| `tools/two-client-runtime/src/bot_route.rs` | Modified — PROMPT 1692 logic |
| `tools/two-client-runtime/src/bot_soak.rs` | Modified — card_info load + register record_gold_update |
| `reports/PROMPT-1692-bot-soak-trigger-nonempty-placement-realism.md` | Added — PROMPT 1692 report |

No unrelated server placement changes, no UI files, no sprint/session-state files.

---

## git diff --check

Trailing whitespace flagged only in the Markdown report file
(`reports/PROMPT-1692-bot-soak-trigger-nonempty-placement-realism.md`, lines 3-5)
using double-space Markdown line-break syntax. All `.rs` files are clean — no
whitespace errors in code.

---

## Code Review Summary

### New pure helpers (`bot_route.rs`)

**`pick_best_trigger_card(offering, card_info, mana_budget) → Option<(u32, u32)>`**

Preference order:
1. Cheapest Minion with `cost ≤ mana_budget` (can be placed immediately)
2. Cheapest Minion regardless of cost (will become placeable as mana grows)
3. Cheapest non-Minion (fallback when no Minion in pool)
4. `offering[0]` with cost 0 (fallback when card_info is empty — triggers empty-batch path)

**`build_trigger_placement(card_id, card_cost, current_mana, reserve_mana, already_placed) → Option<PlacedCardSubmit>`**

Returns `None` (→ empty batch) when:
- `already_placed == true` — idempotent, one placement per game
- `card_id == 0` — no card was purchased
- `card_cost == 0` — cards.json not loaded, cost unknown
- `total_mana < card_cost` — insufficient mana budget

When returning `Some`, mana is split `current_mana_spend` first, remainder from
`reserve_mana_spend`, matching the server `build_bot_placements` split contract.
Target is hardcoded to `PlayTarget::BoardCell { lane: 1, cell: 1 }` (Player A spawn).

### PROMPT 1678 empty-batch contract preserved

| Scenario | PROMPT 1678 outcome | PROMPT 1692 outcome |
|----------|--------------------|--------------------|
| Hand empty (no card acquired) | Empty batch submitted | Empty batch (card_id == 0 → None) |
| card_info not loaded | Empty batch | Empty batch (card_cost == 0 → None) |
| Mana insufficient for card | Empty batch | Empty batch (total_mana < cost → None) |
| Card affordable, not yet placed | N/A | Non-empty batch with correct mana split |
| Card already placed in earlier round | N/A | Empty batch (already_placed → None) |

Non-empty submissions still go through the real production server handler
(`C2SSubmitPlacement` via `ReliableChannel`) — no direct state mutation.

### Protocol types verified

All imported types confirmed present in `shared/src/protocol.rs`:
- `S2CGoldUpdate { gold: u32, current_mana: u32, reserve_mana: u32, mana_cap: u8 }` ✓
- `PlacedCardSubmit { card_id: CardId, target: PlayTarget, current_mana_spend: u32, reserve_mana_spend: u32 }` ✓
- `PlayTarget::BoardCell { lane: u8, cell: u8 }` ✓

---

## Test Results

Disk available at test time: **91 GB free** (blocker from PROMPT 1692 session resolved).

```
cargo test --package two-client-runtime --bin bot-soak-trigger

running 13 tests
test bot_route::tests::test_empty_when_already_placed ... ok
test bot_route::tests::test_mana_split_draws_current_first ... ok
test bot_route::tests::test_empty_when_cost_zero ... ok
test bot_route::tests::test_empty_when_unaffordable ... ok
test bot_route::tests::test_non_empty_placement_when_affordable ... ok
test bot_route::tests::test_pick_cheapest_affordable_minion ... ok
test bot_route::tests::test_pick_cheapest_minion_fallback_when_no_affordable ... ok
test bot_route::tests::test_pick_first_card_when_card_info_empty ... ok
test bot_route::tests::test_pick_non_minion_when_no_minion_in_offering ... ok
test bot_route::tests::test_pick_prefers_minion_over_cheaper_non_minion ... ok
test bot_route::tests::test_placement_spills_cost_into_reserve ... ok
test bot_route::tests::test_pick_returns_none_for_empty_offering ... ok
test bot_route::tests::test_empty_when_card_id_zero ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**All 13 tests PASS.**

---

## Integration Branch State

```
integrate/1701-bot-soak-nonempty-placement-verify @ 345f7b01
  commits ahead of origin/main (f9324431):
    345f7b01  docs: add PROMPT 1706 live two-client QA blocker resolution pack
    a16d0229  docs: add PROMPT 1692 bot soak trigger non-empty placement report
    a9731dce  fix(soak): PROMPT 1692
```

Note: `345f7b01` (PROMPT 1706 docs) was already present on this branch from a prior
session — it is docs-only and out-of-scope for 1701. It does not affect the 1692
code changes or their correctness.

**FF-ready for MAINLAND_ENQUEUE**: YES. The chain from `f9324431` (main) to
`a9731dce` + `a16d0229` is linear with no conflicts. Fast-forward to main is
valid after reviewing the 1706 doc commit scope.

---

## Verdict

PROMPT 1692 code is **VERIFIED**:

- Cheapest affordable Minion selection: confirmed by `test_pick_cheapest_affordable_minion`
- Non-empty `PlacedCardSubmit` when mana allows: confirmed by `test_non_empty_placement_when_affordable`
- Mana split (current first, reserve spill): confirmed by `test_placement_spills_cost_into_reserve` and `test_mana_split_draws_current_first`
- PROMPT 1678 empty-batch contract preserved for all empty-hand / unknown-cost / unaffordable cases
- All 13 pure unit tests PASS, no compilation errors

1701: BOT-SOAK-NONEMPTY-PLACEMENT-VERIFY-INTEGRATION: SHIPPED
