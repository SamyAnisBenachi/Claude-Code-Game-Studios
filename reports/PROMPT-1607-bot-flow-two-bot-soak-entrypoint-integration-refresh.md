# PROMPT 1607 -- BOT-FLOW-TWO-BOT-SOAK-ENTRYPOINT-INTEGRATION-REFRESH

## Status

`READY_FOR_MAINLAND_ENQUEUE`

## Refresh summary

- Source PROMPT: 1603 (`bd67948e664ddb4a498d4ec01455ca36be724593` on
  `origin/work/bot-flow-two-bot-soak-entrypoint-1603`, originally cut from
  stale base `origin/main@237572af`).
- Prior PROMPT 1607 refresh (commit `b20c4450`) was based on
  `origin/main@0cb29c23` and pre-dated PROMPT 1602 main-land.
- Current source-of-truth base: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`
  (`feat(bot): Wave 3 placement heuristic (PROMPT 1602)`).
- Refresh strategy this turn: `git rebase origin/main` on
  `integrate/bot-flow-two-bot-soak-entrypoint-1607` in dedicated worktree
  `D:/Tmp/wt-1607`. Clean rebase, no conflicts (PROMPT 1602 touched
  `server/**` + `tests/unit/bot/**`, PROMPT 1603 touches `client/**`,
  `tools/dev-launcher/**`, `tests/playable_client/**` -- disjoint sets).
- PROMPT 1602 Wave 3 placement behavior preserved: not touched, present in
  base. PROMPT 1603 behavior preserved verbatim: debug-only `Create 2-Bot
  Soak Room` button gated by `CCGS_DEBUG_UI=1`,
  `tools/dev-launcher/Start-BotVsBotSoak.ps1` headless launcher, and root
  `start-bot-vs-bot-soak.bat` forwarder.

## Branch / commit

- Worktree: `D:/Tmp/wt-1607`
- Branch: `integrate/bot-flow-two-bot-soak-entrypoint-1607`
- Refreshed PROMPT 1603 commit (rebased): `51dbb9c08fa17b51eebf02fde9eaa2dc6e9e8612`
- Report commit (this file, post-rebase): `HEAD`
- Parent of PROMPT 1603 commit: `576fbe8ce901a8b919a4c2db58847f2d497d3d15`
  (current `origin/main` -- PROMPT 1602 Wave 3 placement)
- `git merge-base --is-ancestor origin/main HEAD` -> true (strict FF eligible
  vs `origin/main@576fbe8c`)

## Files touched (allowlist review: PASS)

```
client/Cargo.toml
client/src/ui/lobby.rs
reports/PROMPT-1603-bot-flow-two-bot-soak-entrypoint.md
reports/PROMPT-1607-bot-flow-two-bot-soak-entrypoint-integration-refresh.md
start-bot-vs-bot-soak.bat
tests/integration/playable_client/lobby_create_bot_room_test.rs
tools/dev-launcher/Start-BotVsBotSoak.ps1
```

No server bot action loop, server placement, shared protocol,
`production/**`, or unrelated Cargo/CI files were touched. Within owned
scope per PROMPT 1607 task spec.

## Validation

- `git diff --check origin/main..HEAD` -> clean.
- `git merge-base --is-ancestor origin/main HEAD` -> true.
- Path allowlist review -> PASS (see Files touched).
- Broad Cargo suite intentionally deferred to verify lane per task rules.
- Focused `cargo test -p client --test
  playable_client_lobby_create_bot_room_test` was green on prior 1607
  refresh; no source files changed during rebase (purely base change),
  so result expected unchanged. Re-running is the next verify lane.

## Next verify lane

- Re-run focused
  `cargo test -p client --test playable_client_lobby_create_bot_room_test`
  (4 cases) in the verify lane post-mainland or in a separate worker.
- Smoke: launch `start-bot-vs-bot-soak.bat` with `CCGS_DEBUG_UI=1` and
  confirm the lobby debug-only Create 2-Bot Soak Room flow still spawns
  both bots under post-1602 Wave 3 placement heuristic.

## Push outcome

To be recorded by orchestrator on mainland enqueue; this report covers
ready-state only.

## Final line

1607: BOT-FLOW-TWO-BOT-SOAK-ENTRYPOINT-INTEGRATION-REFRESH: READY_FOR_MAINLAND_ENQUEUE
