# PROMPT 1674 -- Bot Soak Launcher Trigger Exit Code Integration

Status: READY_FOR_MAINLAND_ENQUEUE

Source:
- Worker branch: `origin/wt/1674-bot-soak-launcher-trigger-exit-code-repair`
- Worker commit: `959bcf38`
- Integration branch: `integrate/bot-soak-launcher-exit-code-1674`
- Integration base: `origin/main@276e78f1`

Integrated changes:
- Cherry-picked PROMPT 1674 launcher-only repair onto current `origin/main`.
- Scope remained limited to `tools/dev-launcher/Start-BotVsBotSoak.ps1` plus this report.
- The fix reconciles a missing PowerShell process `ExitCode` from `bot-soak-trigger/final_state.json` and records `trigger_exit_code_source` in `soak-summary.json`.

Validation:
- Cherry-pick: clean.
- Path scope: launcher script plus report only.
- Heavy/runtime bot soak verification deferred until PROMPT 1675-1677 land and a combined live soak rerun can validate all four repair lanes together.

1674: BOT-SOAK-LAUNCHER-TRIGGER-EXIT-CODE-INTEGRATION: READY_FOR_MAINLAND_ENQUEUE
