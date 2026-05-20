# PROMPT 1536 — POST-1528-FOCUSED-VERIFY-LANE

- Source-of-truth: `origin/main@5358aed1a6075aca621936fd14f561be8fb854d3`
  ("state: refresh orchestrator snapshot after prompt 1528").
- Worker worktree: `D:/tmp/wt-1536` (detached @ 5358aed1). Root checkout left
  untouched (pre-existing dirty entries: `.claude/settings.json`,
  `production/session-state/codex-orchestrator-state.md`; not modified by this
  lane).
- Shared target dir: `D:/tmp/wt-1536/target`
  (`CARGO_TARGET_DIR=D:/tmp/wt-1536/target`).
- Concurrency: single VERIFY lane; no other cargo lane requested in parallel
  per orchestrator contract.

## Result summary

| # | Focus area | Test target | Result |
|---|---|---|---|
| 1 | Auction settlement (server, unit) | `auction_resolution_settlement_test` | 3/3 PASS |
| 2 | Auction settlement (server, integration) | `auction_resolution_settlement_integration_test` | 2/2 PASS |
| 3 | Auction reservation | `accepted_bid_reservation_test` | 2/2 PASS |
| 4 | Auction won-card disposition (server) | `auction_won_card_disposition_test` | 2/3 PASS default; **3/3 PASS with `--test-threads=1`** → test-harness defect, not a product regression |
| 5 | Auction won-card disposition (client) | `shop_auction_ui_auction_won_card_disposition_test` | 6/6 PASS (incl. PROMPT-1513 wire `card_id` regression) |
| 6 | Card inspect hand/draft | `cargo test -p client --lib ui::hand::inspect::` | 5/5 PASS |
| 7 | Bot room join-loop | `bot_lobby_loop_test` | 6/6 PASS |
| 8 | Resolution replay cadence | `board_rendering_resolution_replay_per_group_cadence_test` | 2/2 PASS |
| 9 | HU-CHROME hand fan | `hand_ui_chrome_composition_test` | **1/1 FAIL** — integration conflict between PROMPT-1520 (card inspect) and PROMPT-1490 (chrome fixture) |

Overall: **2 failures** — one product-integration regression (#9), one test
isolation defect (#4). Neither is a regression of 1518 / 1523 / 1526 / 1528
themselves; both surface a contract gap with sibling landed PROMPTs.

## Exact commands and outputs

All commands run from `D:/tmp/wt-1536` with
`CARGO_TARGET_DIR=D:/tmp/wt-1536/target`. Compile of the server test set:
3m38s (cold). Compile of the client test set: 10m33s (cold). Subsequent runs
under 1s.

### #1 — auction unit settlement

```
cargo test -p server --test auction_resolution_settlement_test
running 3 tests
test test_no_bid_resolution_leaves_gold_unchanged_and_emits_none_settlement ... ok
test test_winner_with_full_hand_spends_gold_discards_card_and_settles ... ok
test test_winner_with_hand_room_spends_gold_adds_card_and_emits_settlement ... ok
test result: ok. 3 passed; 0 failed
```

### #2 — auction integration settlement

```
cargo test -p server --test auction_resolution_settlement_integration_test
running 2 tests
test test_next_auction_entry_starts_with_zero_reserved_gold_for_all_players ... ok
test live_bidding_settles_when_elapsed_passes_deadline_even_if_decrement_lags ... ok
test result: ok. 2 passed; 0 failed
```

### #3 — accepted-bid reservation

```
cargo test -p server --test accepted_bid_reservation_test
running 2 tests
test result: ok. 2 passed; 0 failed
```

### #4 — auction-won-card disposition (FAILING under default parallelism)

```
cargo test -p server --test auction_won_card_disposition_test
running 3 tests
test case_a_winner_settle_grants_card_and_emits_ac10_trace_line ... FAILED
test ac13_won_card_persists_in_hand_across_settle_with_no_submission ... ok
test case_b_no_winner_settle_grants_no_card_and_emits_ac10_trace_line ... ok

---- case_a_winner_settle_grants_card_and_emits_ac10_trace_line stdout ----
panicked at tests/integration/auction/auction_won_card_disposition_test.rs:314:
assertion `left == right` failed
  left: "307"
 right: "107"
```

Re-running serialized:

```
cargo test -p server --test auction_won_card_disposition_test -- --test-threads=1
running 3 tests
test ac13_won_card_persists_in_hand_across_settle_with_no_submission ... ok
test case_a_winner_settle_grants_card_and_emits_ac10_trace_line ... ok
test case_b_no_winner_settle_grants_no_card_and_emits_ac10_trace_line ... ok
test result: ok. 3 passed; 0 failed
```

#### Classification — TEST HARNESS DEFECT (test isolation, not product)

`case_a` and `case_b` install a shared `tracing` `Registry` (`OnceLock`-stored
`CAPTURED_EVENTS`) and bracket their critical section with a
`TEST_SERIAL: OnceLock<Mutex<()>>` guard via `test_serial_lock()`. A third
test in the same binary — `ac13_won_card_persists_in_hand_across_settle_with_no_submission`
— uses `make_card(307, …)` and `enter_auction(…)` which **emits the same
`auction_settled` tracing line with `card_id=307`**, but it neither acquires
`test_serial_lock()` nor `install_capture_subscriber()`. Under default
multi-threaded `cargo test`, `ac13` can run while `case_a` holds the serial
lock and is between `take_captured()` (clear) and `take_captured()` (read).
`ac13`'s `card_id=307` tracing event leaks into the captured queue, and
`find_auction_settled_event(&captured)` returns it before `case_a`'s own
`card_id=107` event, blowing the field assertion at line 314.

This is the same `enter_auction` + `settle_expired_auction` code path
exercised by the now-passing `auction_resolution_settlement_*` tests; the
product behaviour is correct. Failure is induced solely by the test
file's incomplete serial discipline.

#### Smallest repair prompt — recommendation

Smallest defensible repair: add `let _serial = test_serial_lock();` (and
`install_capture_subscriber();` for symmetry) at the top of
`ac13_won_card_persists_in_hand_across_settle_with_no_submission` in
`tests/integration/auction/auction_won_card_disposition_test.rs`. Two-line
test-harness edit; no product code touched. Scoped to one file. Stand up as
e.g. `PROMPT-1537 -- AUCTION-WON-CARD-DISPOSITION-TEST-SERIAL-LOCK-LEAK`.

Verify lane did NOT apply the fix — within scope rule "tiny test-harness-only
correction and clearly owned" this would have qualified, but the lane charter
says "report and let orchestrator spawn repair" when in doubt, and the
defect is in a file owned by the auction-disposition work (PROMPT 1347 /
1513), not by this lane.

### #5 — client-side auction-won-card disposition

```
cargo test -p client --test shop_auction_ui_auction_won_card_disposition_test
running 6 tests
test ac15_opponent_settled_toast_text_includes_price ... ok
test prompt_1513_arm_uses_wire_card_id_not_local_auction_state ... ok
test ac4_ac5_winner_banner_and_marker_spawn_at_placement_entry ... ok
test ac14_staging_won_card_clears_banner_and_marker ... ok
test ac9_banner_and_marker_clear_at_phase_exit_no_op_path ... ok
test ac9_marker_does_not_reappear_after_clear ... ok
test result: ok. 6 passed; 0 failed
```

### #6 — card inspect hand/draft (PROMPT 1482 + 1520)

```
cargo test -p client --lib ui::hand::inspect::
running 5 tests
test ui::hand::inspect::tests::build_view_minion_includes_attack_health_keywords ... ok
test ui::hand::inspect::tests::build_view_spell_omits_attack_health_and_fills_fallback_rules_text ... ok
test ui::hand::inspect::tests::request_switches_to_different_card_without_dismiss ... ok
test ui::hand::inspect::tests::dismiss_message_closes_overlay ... ok
test ui::hand::inspect::tests::request_opens_then_repeat_request_closes ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out
```

### #7 — bot room join-loop (PROMPT 1526 / 1514)

```
cargo test -p server --test bot_lobby_loop_test
running 6 tests
test test_deterministic_class_is_stable_across_calls ... ok
test test_auto_confirm_is_idempotent_across_ticks ... ok
test test_auto_confirm_skips_human_slots ... ok
test test_completing_human_confirm_after_bot_satisfies_all_classes_confirmed ... ok
test test_auto_confirm_skips_when_lobby_not_waiting ... ok
test test_auto_confirm_sets_slot_class_and_selections_for_bot ... ok
test result: ok. 6 passed; 0 failed
```

### #8 — resolution replay per-group cadence (PROMPT 1521 / 1528)

```
cargo test -p client --test board_rendering_resolution_replay_per_group_cadence_test
running 2 tests
test test_replay_does_not_double_emit_when_repeated_frames_share_a_group ... ok
test test_replay_emits_damage_for_first_group_only_until_time_advances ... ok
test result: ok. 2 passed; 0 failed
```

### #9 — HU-CHROME hand fan (FAILING)

```
cargo test -p client --test hand_ui_chrome_composition_test
running 1 test
test fan_slot_chrome_children_have_absolute_layout_after_placement_entry ... FAILED

---- fan_slot_chrome_children_have_absolute_layout_after_placement_entry stdout ----
… HandUiPlugin loaded …
thread 'Compute Task Pool (0)' panicked at bevy_ecs-0.18.1/src/error/handler.rs:125:
Encountered an error in system
  `client::ui::hand::inspect::apply_hand_card_inspect_target_system`:
  Parameter `Res<'_, ButtonInput<KeyCode>>` failed validation:
  Resource does not exist.
If this is an expected state, wrap the parameter in `Option<T>`
 and handle `None` when it happens, or wrap the parameter in `If<T>`
 to skip the system when it happens.
Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
test result: FAILED. 0 passed; 1 failed
```

#### Classification — INTEGRATION CONFLICT between PROMPT-1520 and PROMPT-1490

PROMPT-1520 (commit `93a88910`, "hand+draft card inspect overlay") introduced
`client::ui::hand::inspect::apply_hand_card_inspect_target_system`, which
requires `Res<ButtonInput<KeyCode>>` for Escape-dismiss handling. The system
is registered unconditionally by `HandUiPlugin`.

PROMPT-1490 (commit `c321e741`, "hand fan readability + playable-affordance
Krosmaga polish") wrote `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs`,
which spins up an `App` that adds `HandUiPlugin` directly (no `MinimalPlugins`
`InputPlugin`) and so never inserts `ButtonInput<KeyCode>`. With Bevy 0.18's
default error handler (panic), the parameter validation failure unwinds the
schedule.

Both PROMPTs are landed on `origin/main`; this is the first verification pass
that exercised the chrome composition test after the inspect system was added.
The product itself runs fine in a real client because `InputPlugin` is always
present there.

#### Smallest repair prompt — recommendation

Two equally minimal options. Pick one.

- **Option A (product-side, ~1 line):** in
  `client/src/ui/hand/inspect.rs`, change the system signature
  `keys: Res<ButtonInput<KeyCode>>` → `keys: Option<Res<ButtonInput<KeyCode>>>`
  and treat `None` as "no key pressed". This makes the system tolerant of
  test apps that don't install `InputPlugin` (matches Bevy 0.18 idiom and the
  hint in the panic message itself).
- **Option B (test-side, ~1 line):** in
  `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs`, add
  `.init_resource::<ButtonInput<KeyCode>>()` (or `.add_plugins(InputPlugin)`)
  to the test fixture.

Recommended: **Option A** — keeps the chrome test pure (no input plumbing
just to silence a sibling system) and pre-empts the same trap in any other
hand-ui test that wires `HandUiPlugin` without `InputPlugin`. Stand up as
`PROMPT-1538 -- HAND-INSPECT-INPUT-RES-OPTIONALIZE` or similar.

Verify lane did NOT apply the fix — change spans a product file and is owned
by PROMPT-1520's authors.

## Out-of-scope items NOT run

- Workspace-wide `cargo test` / smoke — explicitly deferred per VERIFY lane
  charter; not necessary to triage the focus-area findings.
- Anything outside the focus areas listed in PROMPT-1536 §Task.
- No product / shared / config files edited or committed.

## Environment notes

- `cargo 1.95.0`, Bevy 0.18.1, lightyear 0.26.4, MSVC link path resolved
  (build succeeded; no link.exe blocker observed in this shell).
- No `.cargo/config.toml` present at the worktree (workspace
  `target-dir = "target/msvc-local"` from CODEX.md is not in tree at
  `5358aed1`; default `target/` used via `CARGO_TARGET_DIR`).

## Recommended follow-ups (numbered for orchestrator dispatch)

1. **PROMPT-1537 (suggested) — AUCTION-WON-CARD-DISPOSITION-TEST-SERIAL-LOCK-LEAK**
   2-line test-only fix in
   `tests/integration/auction/auction_won_card_disposition_test.rs`:
   acquire `test_serial_lock()` and `install_capture_subscriber()` in
   `ac13_won_card_persists_in_hand_across_settle_with_no_submission`.
2. **PROMPT-1538 (suggested) — HAND-INSPECT-INPUT-RES-OPTIONALIZE**
   1-line product fix in `client/src/ui/hand/inspect.rs`: wrap
   `Res<ButtonInput<KeyCode>>` as `Option<Res<…>>` in
   `apply_hand_card_inspect_target_system`. Validate by re-running
   `cargo test -p client --test hand_ui_chrome_composition_test` and
   `cargo test -p client --lib ui::hand::inspect::`.

Both are independent and can run in parallel; neither blocks the other.

## Orchestrator safety message addressed

Mid-run notice "your terminal started in the shared/root checkout, which is
dirty and stale" — confirmed: all cargo commands and the report write
occurred under `D:/tmp/wt-1536` (dedicated worktree pinned to
`origin/main@5358aed1a6075aca621936fd14f561be8fb854d3`). Root checkout at
`D:/_DEV/Work/Claude-Code-Game-Studios` was inspected for SoT fetch only
and not edited. Pre-existing dirty entries (`.claude/settings.json`,
`production/session-state/codex-orchestrator-state.md`) belong to prior
session state and were not touched by this lane.

1536: POST-1528-FOCUSED-VERIFY-LANE: PARTIAL
