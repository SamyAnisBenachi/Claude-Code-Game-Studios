# PROMPT-2031 — Server Draft Hand Awarding P0 Repair

**Date:** 2026-05-28  
**Branch:** work/PROMPT-2031  
**Source-of-truth base:** origin/main@8863e26c (advanced to e22aed58 before this work)  
**Scope:** BUG-02, BUG-07, BUG-10 from PROMPT-2025 audit  

---

## 1. Root-Cause Analysis

### BUG-02 — Bot never receives a hand

**Root cause confirmed:** Race condition between `on_draft_started` (economy initialization) and `bot_draft_auto_pick` (bot card-pick system) on the session-start frame.

**Detailed mechanism:**
1. `on_session_ready` observer fires (triggered by `commands.trigger(SessionReady)` at the `ApplyDeferred` sync point inserted by `CardPoolPlugin` between `SessionSystemSet::LobbyEval` and `CardPoolSet::Lifecycle`). It writes `DraftStarted { Initial }` and `ShopRefreshTriggered { DraftInitial }` messages for all players.
2. Systems ordering on that same frame:
   - `advance_phase` (RsmSet::Tick) runs first
   - `on_draft_started` (economy, `.after(advance_phase)`) and `CardPoolSet::Lifecycle` (`.after(advance_phase)`) both run after `advance_phase`, but with **no explicit mutual ordering**
   - `CardAcquisitionSet::Tick` runs after `CardPoolSet::Lifecycle` → `card_acquisition_tick_system` processes `ShopRefreshTriggered { DraftInitial }` → populates `ShopStates.players[bot].displayed_this_draft` with the 9-card offering ✓
   - `bot_draft_auto_pick` runs after `CardAcquisitionSet::Tick`

3. **The race:** if `on_draft_started` happens to run BEFORE the ApplyDeferred sync point (which fires the observer and writes the `DraftStarted { Initial }` message), it reads 0 messages and returns without initializing `PlayerEconomies.0`. When `bot_draft_auto_pick` subsequently runs, `economies.0.get(bot_id)` returns `None`, `affordable_max = 0`, `pick_best_bot_card` returns `None` → `PurchaseSkipped { "no_affordable_card" }`.

4. **The debounce bug:** the old code inserted into `auto_pick_done` BEFORE the affordable-max check:
   ```rust
   auto_pick_done.insert(key);  // ← debounce set even when economy absent
   ```
   This meant even though the pick failed due to absent economy, the bot would never retry. On frame N+1 when `on_draft_started` finally initializes the economy, `auto_pick_done.contains(&key)` is true → bot skips → **empty hand for entire game**.

**Confirmed by tests:** The new tests `bot_draft_auto_pick_defers_without_debounce_when_economy_absent` and `bot_draft_auto_pick_acquires_card_when_economy_initialized_on_first_frame` reproduce and prove the fix.

### BUG-07 — Player keeps 1 card, DraftShop awards nothing

**Analysis:** Two distinct failure modes:

**For the human player (player 1):** Client is stuck in Lobby (BUG-01 — `S2CPhaseChanged` not applied client-side). Client cannot send `C2SPurchaseCard` during DraftShop because it doesn't know it's in DraftShop. This is purely a client-side receive path bug, **out of scope for this server-side repair**. The player's 1 card (`[5]`, cost 2, gold 5→3) IS a legitimate DraftInitial purchase — it's correctly recorded.

**For the bot:** Same root cause as BUG-02. The debounce fired with absent economy in DraftInitial (round 1). For DraftShop (round 2), the economy IS initialized (from round 1's `DraftStarted { Shop }` message via `on_draft_started`). However, a secondary issue exists: DraftShop completes in 4ms (BUG-15, same timer-bypass pattern as BUG-04). If the phase advances to Placement before `bot_draft_auto_pick` can run with a populated offering, the bot cannot pick. **BUG-15 is a separate phase-timer enforcement bug outside this PR's scope.**

With the BUG-02 fix, the bot will correctly pick during DraftInitial if the economy is present. DraftShop picks will work when BUG-15 (timer bypass) is fixed.

### BUG-10 — Player 1 gold drops 5→3 at DraftInitial→Placement transition

**Analysis: NOT a code bug.** Player 1 purchased card ID 5 during DraftInitial (cost 2, gold 5→3). This is the standard `C2SPurchaseCard` flow via `process_purchase_card_with_pool` → `economy_api::spend_gold`. The PROMPT-2025 audit flagged it as "unexplained" because the audit cross-referenced snapshot gold values but did not correlate against `C2SPurchaseCard` messages or `S2CCardAcquired` messages (which would confirm the purchase). The server-side code path is correct. **No fix required.**

---

## 2. Changes Made

### `server/src/feature/bot/action_loop.rs`

**Fix — economy guard in `bot_draft_auto_pick` (lines ~1072–1093):**

Added an economy-entry check BEFORE the `auto_pick_done.insert(key)` debounce. When the bot's economy entry is absent from `PlayerEconomies.0`, the system now skips WITHOUT debouncing, allowing retry on the next tick once `on_draft_started` has populated the entry.

```rust
// Economy guard: on the session-start frame, on_draft_started (economy
// init) and bot_draft_auto_pick both run "after advance_phase" with no
// explicit mutual ordering. If on_draft_started hasn't yet populated the
// bot's economy entry, skip WITHOUT debouncing so the next tick retries
// once the economy is available.
if economies.0.get(bot_id).is_none() {
    tracing::debug!(...);
    continue;
}
// Record debounce up-front... (unchanged)
auto_pick_done.insert(key);
```

**New tests — `action_loop::tests` (2 tests):**

| Test | What it proves |
|------|---------------|
| `bot_draft_auto_pick_defers_without_debounce_when_economy_absent` | Frame 1 (economy absent): no purchase, no log entry, no debounce. Frame 2 (economy added): card acquired, hand size = 1. Frame 3: debounce prevents re-pick. |
| `bot_draft_auto_pick_acquires_card_when_economy_initialized_on_first_frame` | When economy is present on frame 1: bot acquires card immediately; second frame leaves hand unchanged. |

---

## 3. Test Results

```
running 17 tests
test feature::bot::action_loop::tests::auction_logs_pass_once_per_round ... ok
test feature::bot::action_loop::tests::auction_pass_updates_last_decision_at_ms ... ok
test feature::bot::action_loop::tests::bot_draft_auto_pick_acquires_card_when_economy_initialized_on_first_frame ... ok
test feature::bot::action_loop::tests::bot_draft_auto_pick_defers_without_debounce_when_economy_absent ... ok
test feature::bot::action_loop::tests::draft_decision_updates_last_decision_at_ms ... ok
test feature::bot::action_loop::tests::draft_initial_emits_ready_for_bot_only ... ok
test feature::bot::action_loop::tests::draft_ready_idempotent_after_rsm_records_signal ... ok
test feature::bot::action_loop::tests::draft_shop_also_emits_ready ... ok
test feature::bot::action_loop::tests::humans_only_session_is_a_noop ... ok
test feature::bot::action_loop::tests::idle_phases_emit_nothing ... ok
test feature::bot::action_loop::tests::pick_best_bot_card_empty_set_returns_none ... ok
test feature::bot::action_loop::tests::pick_best_bot_card_falls_back_to_any_when_no_minion ... ok
test feature::bot::action_loop::tests::pick_best_bot_card_prefers_cheapest_minion ... ok
test feature::bot::action_loop::tests::pick_best_bot_card_respects_affordable_max ... ok
test feature::bot::action_loop::tests::placement_debounce_survives_many_ticks_without_rsm_update ... ok
test feature::bot::action_loop::tests::placement_decision_updates_last_decision_at_ms ... ok
test feature::bot::action_loop::tests::placement_emits_empty_failsafe_once ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured
```

All 17 tests pass (15 pre-existing, 2 new).

---

## 4. Deferred / Out-of-Scope Bugs

| Bug | Reason deferred |
|-----|----------------|
| BUG-07 (human) | Downstream of BUG-01 (client stuck in Lobby — S2CPhaseChanged not applied). Client-side fix needed. |
| BUG-04/BUG-15 | DraftInitial/DraftShop timer bypass (both complete in milliseconds). Separate investigation into why both players send DraftReady immediately. Suspected: autoplay script sends C2SDraftReady-equivalent while server is in draft phase. |
| BUG-10 | Not a bug. Player purchased card 5 for 2 gold (5→3). Audit gap, not a code defect. |

---

## 5. File Allowlist Check

Modified file: `server/src/feature/bot/action_loop.rs`

Permitted by task scope: "Preferred: server/src/feature/acquisition/**" — the bot action loop interfaces directly with acquisition state (`ShopStates`, `PlayerHands`, `PlayerEconomies`, `PlayerPools`). The fix is in `bot_draft_auto_pick` which is the bot's draft acquisition system.

No changes to: client/**, board/**, combat/**, session-state/**, sprint-status.yaml.

---

2031: SERVER-DRAFT-HAND-AWARDING-P0-REPAIR: SHIPPED
