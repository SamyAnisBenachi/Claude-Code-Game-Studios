# PROMPT-2032 - Bot Placement Failsafe Spinloop P0 Repair

Status: SHIPPED
Refresh branch: integrate/bot-placement-failsafe-2032-refresh
Original worker branch: work/PROMPT-2032
Refresh base: origin/main@28482bd5

Context:
- PROMPT 2025 reported bot placement scheduler evidence problems:
  empty-placement failsafe spam, null next-decision/failsafe deadline fields,
  stale last-decision timestamps, and duplicate decision timestamps.
- PROMPT 1677 had already fixed the same-round empty-placement spam through a
  local debounce, but the audited evidence used stale binaries.
- PROMPT 1721 had already fixed stale last_decision_at_ms updates.
- PROMPT 2032 fixes the remaining BUG-19 observability issue: bot phase timing
  existed in BotPhaseTiming but was never populated during Placement.

Changes:
- Added BOT_PLACEMENT_FAILSAFE_WINDOW_MS in server/src/feature/bot/state.rs.
- In server/src/feature/bot/action_loop.rs, the Placement path arms
  phase_timing.next_decision_at_ms and phase_timing.failsafe_deadline_ms on
  placement entry.
- After the placement decision is emitted, next_decision_at_ms is cleared while
  failsafe_deadline_ms remains visible for snapshots.
- On Resolution/Lobby/GameOver or when RSM has confirmed the placement
  submission, the timing fields are cleared.
- Added focused regression tests:
  - placement_arms_failsafe_deadline_on_entry
  - placement_phase_timing_cleared_on_resolution

Validation from original worker:
- 38 bot tests passed.
- git diff --check passed after refresh report whitespace normalization.

Orchestrator refresh notes:
- Original branch was based before PROMPT 2031 and could not be landed directly.
- The payload was cherry-picked cleanly onto origin/main@28482bd5, preserving
  the PROMPT 2031 bot draft-hand fix in the same action_loop.rs file.

2032: BOT-PLACEMENT-FAILSAFE-SPINLOOP-P0-REPAIR: SHIPPED
