# PROMPT 1514 — BOT-ROOM-JOIN-LOOP

## Summary

Adds the next bot-room join-loop increment: a server-side bot occupant of a
lobby slot now picks a deterministic class on its own and is recorded in
`ClassSelections`, so a room created via the existing `C2SCreateBotRoom` /
`C2SAddBot` path can advance past `LobbyWaiting` once the human owner
confirms their class. This is foundation/flow only — no AI heuristics, no new
protocol surface, no client UI changes.

## Source-of-truth

Branched from `origin/main` at commit `5d46b9a9` ("PROMPT-1508 repair
HU-CHROME-02 hand-fan art-image width") in a dedicated worktree:

- Worktree: `D:/_DEV/claude-code-game-studios-worktrees/bot-room-join-loop-1514`
- Branch: `work/bot-room-join-loop-1514`

## Gap closed

Before this prompt, the C2S bot-add path (PROMPT 1430) inserted a synthetic
bot `PlayerId` into a slot but no system ever recorded a class for that bot
in `ClassSelections`. `f4_session_ready` requires every occupied slot to be
both filled and class-confirmed, so a lobby containing a bot could never
reach `GameActive` — the room was effectively a dead end. `add_bot_to_room`
worked, `evaluate_room_session_ready` ran, but it always returned `false`.

## What a bot can do now

- Be created via `C2SCreateBotRoom` or added via `C2SAddBot` (existing path,
  unchanged).
- Auto-confirm a deterministic class derived from
  `(session_id, slot_index)` while the room is in `LobbyWaiting`. The class
  picked is the same every time for the same room and slot, so a future
  replay tool reproduces it without extra metadata.
- Mirror that class onto the `SessionSlot.class` field so existing
  `protocol_slots` mirroring and `all_classes_confirmed` succeed.
- Register itself in `BotPlayers` the first time it confirms a class
  (creating a `BotState` with a deterministic seed = the bot's synthetic
  `PlayerId`).
- Append exactly one `BotDecisionEntry { kind: ClassConfirmed }` per bot per
  lobby into `BotDecisionLog`.
- Once the human owner confirms their class through the existing
  `handle_confirm_class` system, the lobby's `all_classes_confirmed` and
  `all_slots_filled` both return true and the existing
  `evaluate_room_session_ready` lifts the room into `GameActive` and triggers
  `SessionReady` — the bot enters the game alongside the human.

## What is explicitly still deferred

The following remain out of scope per the PROMPT 1514 owned-scope and are
called out for the next bot-flow prompt:

- **In-round bot decisions.** Draft picks, shop purchases, auction bids, and
  placement submissions are still unimplemented. The bot has a class but
  cannot meaningfully play once `DRAFT_INITIAL` starts. Expect the room to
  stall at the first phase that requires a bot action.
- **C2S confirm-class echo for bots.** This loop bypasses the public
  `confirm_class` outcome path — no `S2CClassLocked` or `S2CClassesRevealed`
  is emitted on the bot's behalf. The human client still receives its own
  lock + the reveal once both classes are present via the normal flow (the
  reveal is computed from `slot.class`, not from the outcome emitted here).
  Re-validating the reveal UX with a bot occupant is a follow-up.
- **`ClassPreviews` for bots.** The bot never publishes a preview; only the
  final confirm. If the lobby UI distinguishes preview vs locked for the bot
  slot, it will appear to "skip" the preview state. UI follow-up — not in
  scope.
- **Round-state observer / heuristic family.** No `BotPlugin` for in-game
  decisions yet. `BotLobbyPlugin` is the only registered bot system.
- **Local single-player bot mode.** Per the PROMPT directive, the room-based
  bot is preferred and the local mode stays deferred.
- **Bot disconnect / mid-game removal.** `handle_remove_bot` already exists
  for lobby; mid-game removal is out of scope.

## Files changed

Owned-scope only:

- `server/src/feature/bot/lobby_loop.rs` *(new)* — system + `BotLobbyPlugin`
  + `deterministic_class_for_bot`.
- `server/src/feature/bot/mod.rs` — re-export the new symbols.
- `server/src/main.rs` — add `BotLobbyPlugin` to the server app.
- `server/Cargo.toml` — register the new integration test.
- `tests/unit/bot/bot_lobby_loop_test.rs` *(new)* — six focused tests.

No edits to `server/src/lobby/**` (the existing class-choice handler is
sufficient — the bot loop does not need its own C2S receiver).

No edits to `shared/src/protocol/**` (no new protocol messages — the bot
participant tagging from PROMPT 1430 is reused as-is).

## Validation

- Path allowlist: all changes inside `server/src/feature/bot/**`,
  `server/src/main.rs`, `server/Cargo.toml`, `tests/unit/bot/**`,
  `reports/PROMPT-1514-bot-room-join-loop.md`. No client, shared protocol,
  or unrelated files touched.
- `cargo check -p server` — clean.
- `cargo test -p server --test bot_lobby_loop_test` — **6 passed, 0 failed**.
- `cargo test -p server --test bot_foundation_state_test` — **8 passed, 0
  failed** (no regression of the PROMPT 1428 foundation).
- `git diff --check` — clean.

Broad cargo suite intentionally skipped per PROMPT 1514 implementation rules
(VERIFY lane owns broad verification).

## Test coverage

`tests/unit/bot/bot_lobby_loop_test.rs`:

1. `test_deterministic_class_is_stable_across_calls` — pure-function
   determinism + never picks `Neutral`.
2. `test_auto_confirm_sets_slot_class_and_selections_for_bot` — slot,
   `ClassSelections`, `BotPlayers`, `BotDecisionLog` all updated on the
   first Update tick.
3. `test_auto_confirm_skips_human_slots` — human slots remain untouched.
4. `test_auto_confirm_is_idempotent_across_ticks` — exactly one decision-log
   entry per bot per lobby across repeated ticks.
5. `test_auto_confirm_skips_when_lobby_not_waiting` — `GameActive` rooms are
   left alone.
6. `test_completing_human_confirm_after_bot_satisfies_all_classes_confirmed`
   — once the human confirms via the existing path, the gate predicates
   used by `evaluate_room_session_ready` both pass.

## Implementation rules compliance

- Dedicated worktree branched from `origin/main`. ✅
- No broad cargo run — only the two focused `--test` targets. ✅
- Protocol additions: none (kept backwards-compatible). ✅
- No revert of unrelated worker edits in shared paths (server/lobby
  untouched). ✅
- No reach into client or shared protocol. ✅

## Next-step recommendations

The natural successor PROMPTs (not part of this scope):

1. **BOT-DRAFT-LOOP** — deterministic stub for `DRAFT_INITIAL`: bot picks
   one offered card per round so the draft phase advances. Same shape as
   this loop: read RSM phase, gate on `BotPlayers`, append decision log.
2. **BOT-PLACEMENT-FAILSAFE** — empty-placement submission per
   `BotDecisionKind::EmptyPlacementFailsafe` so placement does not stall on
   the bot's clock.
3. **BOT-AUCTION-PASS** — deterministic pass response so the auction phase
   resolves.

Each can follow the same isolated foundation pattern: one system, one
sub-module under `server/src/feature/bot/`, focused tests, no protocol.

---

1514: BOT-ROOM-JOIN-LOOP: SHIPPED
