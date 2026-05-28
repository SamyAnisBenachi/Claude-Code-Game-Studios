PROMPT 2031 -- Server Draft Hand Awarding P0 Repair

Status: SHIPPED
Branch: integrate/server-draft-hand-2031-refresh
Base: origin/main@8f7d3502

Context:
- Original worker report claimed branch work/PROMPT-2031, but the remote branch still pointed at stale PROMPT-2007 report work and contained no 2031 payload.
- Orchestrator rebuilt the small server-side fix from the report on a clean branch over current origin/main.

Bug:
- Bot draft auto-pick could permanently skip card acquisition when the bot economy entry was missing for one frame at draft start.
- The system inserted the per-round debounce key before checking economy availability.
- If economy initialization arrived on the next frame, the debounce prevented retry, leaving bot hand empty.

Changes:
- Updated server/src/feature/bot/action_loop.rs so bot_draft_auto_pick checks for PlayerEconomy before inserting the debounce key.
- Missing economy now defers silently and retries on the next tick.
- Added focused tests for both delayed economy initialization and normal first-frame acquisition.

Validation:
- git diff --check -- server/src/feature/bot/action_loop.rs: PASS
- CARGO_TARGET_DIR=C:\Users\Sam\AppData\Local\Temp\ccgs-target-2031 cargo test -p server feature::bot::action_loop::tests::bot_draft_auto_pick: PASS
  - 2 target tests passed.
- cargo fmt --check was not used as a gate because current origin/main has pre-existing rustfmt diffs across unrelated files. The touched file was kept scoped to the server bot fix.

Notes:
- D: had insufficient free space for Cargo build artifacts. Validation used a temporary Cargo target on C:.
- The fix covers the BUG-02/P0 draft hand-awarding failure mode described in the worker report.

2031: SERVER-DRAFT-HAND-AWARDING-P0-REPAIR: SHIPPED
