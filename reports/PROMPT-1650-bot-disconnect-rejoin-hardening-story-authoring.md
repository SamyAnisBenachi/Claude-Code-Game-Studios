# PROMPT-1650 Report: Bot Disconnect / Re-join Hardening -- Story Authoring

**Date**: 2026-05-27
**Branch**: work/bot-disconnect-rejoin-hardening-1650
**Source-of-truth**: origin/main@178a8471

---

## Summary

Authored Story 006 (`BOT-DISCONNECT-REJOIN-006`) for the Bot & Autoplay epic to
close the open follow-up items listed in Story 001 (`BOT-ROOM-PARTICIPANT-001`).
Updated `EPIC.md` to add the new story row. No Sprint 18/19/20 activation, no
code changes, no story-done claims.

---

## Files Written

| File | Change |
|---|---|
| `production/epics/bot-and-autoplay/story-006-bot-disconnect-rejoin-hardening.md` | New -- full story spec |
| `production/epics/bot-and-autoplay/EPIC.md` | Updated -- added story 006 row, updated Non-Claims block |
| `reports/PROMPT-1650-bot-disconnect-rejoin-hardening-story-authoring.md` | New -- this report |

---

## Story 006 Coverage

Story 006 (`BOT-DISCONNECT-REJOIN-006`) is scoped to:

- **Bot join / disconnect / rejoin lifecycle**: Explicit `BotParticipantState`
  enum (`Joining` → `Active` → `Disconnected` → `Rejoin`/`Evicted`); state
  machine governing action dispatch and round-advancement.
- **Duplicate bot prevention**: Idempotency guard at bot-join time; second
  join attempt for an occupied slot is rejected server-side.
- **Round/session state safety**: Forfeit-action path unblocks round advancement
  when a bot disconnects mid-phase (AUCTION bid or PLACEMENT); configurable
  forfeit timeout (default 10 s); no deadlock on human participant's round.
- **Reconnect snapshot correctness**: On reconnect, QA snapshot path emits
  current authoritative `phase`/`round`/`client_state`; not the cached
  pre-disconnect frame. Aligns with the snapshot infrastructure from PROMPT 1597.
- **Decision-log continuity**: Stable `bot_session_id` (logical `Uuid`, not
  Lightyear connection handle) carried on all decision log entries; survives
  disconnect/rejoin boundary for post-game replay correlation.
- **Human participant safety**: All bot action paths handle `Disconnected` state
  without `unwrap()` panic; human completes the affected round normally.
- **State cleanup on eviction**: If bot does not reconnect within eviction timeout
  (default 60 s), participant record is cleaned from room map with structured
  log entry.
- **Integration test target**: `tests/integration/bot/disconnect_rejoin_test.rs`
  covering AC1–AC7; BLOCKING gate at story-done time.

---

## Epic Index Consistency

EPIC.md Stories table now lists 6 rows (001–006); all links point to existing
or newly created `.md` files. Non-Claims block updated to include Sprint 20 to
match the new story's unscheduled status.

Story 006 is:
- Sprint 20+ candidate, explicitly unactivated.
- Gated on `BOT-ROOM-PARTICIPANT-001` story-done (Sprint 19 target).
- Ledger-only; no implementation, no test evidence, no code change.

---

## Validation

- `git diff --check`: no whitespace errors detected.
- Story links in EPIC.md table are consistent with files on disk (stories
  001–006 all present).
- No forbidden files touched: `production/sprint-status.yaml` unchanged,
  `production/sprints/**` unchanged, `production/session-state/**` unchanged,
  source code unchanged, tests unchanged, QA evidence unchanged.
- No Cargo, no Trunk, no CI invocation.

---

1650: BOT-DISCONNECT-REJOIN-HARDENING-STORY-AUTHORING: SHIPPED
