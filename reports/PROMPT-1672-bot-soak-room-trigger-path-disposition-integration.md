# PROMPT 1672 - BOT-SOAK-ROOM-TRIGGER-PATH-DISPOSITION-INTEGRATION

Status: READY_FOR_MAINLAND_ENQUEUE

## Summary

Integrated PROMPT 1672 from `origin/prompt-1672-bot-soak-trigger` onto current
`origin/main` as `integrate/bot-soak-trigger-1672`.

PROMPT 1672 selects and implements the normal-protocol headless client driver
lane for bot-vs-bot soak runs:

- adds `bot-soak-trigger`, a headless Bevy/Lightyear client binary;
- sends normal production C2S messages rather than mutating server state;
- creates a bot room through `C2SCreateBotRoom`;
- drives the human-proxy slot through class confirm, draft initial, placement,
  draft shop, and auction phases;
- exits successfully on `S2CGameOver` or configured max-round cutoff;
- wires `tools/dev-launcher/Start-BotVsBotSoak.ps1` to build and run the
  trigger instead of sleeping while no client is connected.

## Source

- Worker branch: `origin/prompt-1672-bot-soak-trigger`
- Worker commit: `1fae2641`
- Integration branch: `integrate/bot-soak-trigger-1672`
- Integration base: current `origin/main` after PROMPT 1673 state update

## Scope

Changed files:

- `tools/dev-launcher/Start-BotVsBotSoak.ps1`
- `tools/two-client-runtime/Cargo.toml`
- `tools/two-client-runtime/src/bot_route.rs`
- `tools/two-client-runtime/src/bot_soak.rs`
- `reports/PROMPT-1672-bot-soak-room-trigger-path-disposition-integration.md`

## Validation

- Cherry-pick onto current `origin/main`: clean.
- `git diff --check origin/main..HEAD`: PASS.
- Bevy/Lightyear review: route uses production Lightyear message senders and
  receivers, no direct authoritative server mutation.
- Cargo-heavy verification was not rerun in this integration lane. Worker report
  recorded `cargo build -p two-client-runtime --bin bot-soak-trigger` PASS.

## Remaining Gate

Run the live bot-vs-bot bounded soak after this lands:

```powershell
.\tools\dev-launcher\Start-BotVsBotSoak.ps1 -MaxRounds 3 -DurationSeconds 60 -PlayRepoRoot "D:\_DEV\Work\Claude-Code-Game-Studios"
```

Required evidence:

- `soak-summary.json` trigger exit code `0`;
- `bot-soak-trigger/final_state.json` with `received_game_over=true` or
  max-round cutoff reached as designed;
- bot decision log populated when snapshot/log env is enabled;
- server snapshots populated for round/phase evidence.

1672: BOT-SOAK-ROOM-TRIGGER-PATH-DISPOSITION-INTEGRATION: READY_FOR_MAINLAND_ENQUEUE
