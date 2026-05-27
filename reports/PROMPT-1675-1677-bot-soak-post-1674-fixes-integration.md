# PROMPT 1675-1677 -- Bot Soak Post-1674 Fixes Integration

Status: READY_FOR_MAINLAND_ENQUEUE

Source:
- Integration branch: `integrate/bot-soak-post-1674-fixes-1675-1677`
- Integration base: `origin/main@5696668e`
- Worker commits:
  - `2590d9e1` -- PROMPT 1675 missing `ClassSelections` guard
  - `370bd75c` -- PROMPT 1676 server snapshot output env wiring
  - `de9c946d` -- PROMPT 1677 placement failsafe spam debounce

Integrated changes:
- Cherry-picked PROMPT 1675 cleanly. It adds a guard so `bot_lobby_auto_confirm`
  no longer panics when `ClassSelections` is absent after/around gameover.
- Cherry-picked PROMPT 1676 with a small conflict in
  `tools/dev-launcher/Start-BotVsBotSoak.ps1` notes caused by PROMPT 1674. The
  resolution preserves both PROMPT 1674 trigger exit-code reconciliation and
  PROMPT 1676 snapshot-dir env wiring.
- Cherry-picked PROMPT 1677 cleanly. It debounces repeated placement failsafe
  decisions per bot/round.

Validation:
- Conflict markers removed.
- `git diff --check` passed.
- Path scope:
  - `server/src/feature/bot/lobby_loop.rs`
  - `server/src/feature/bot/action_loop.rs`
  - `tools/dev-launcher/Start-BotVsBotSoak.ps1`
  - reports only
- Heavy/runtime bot-vs-bot soak verification is deferred until this branch lands.

1675-1677: BOT-SOAK-POST-1674-FIXES-INTEGRATION: READY_FOR_MAINLAND_ENQUEUE
