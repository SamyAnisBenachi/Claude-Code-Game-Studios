# PROMPT 1670 -- BOT-DEBUG-OVERLAY-STORY-STATUS-AC5-RULING-PREP

Date: 2026-05-27
Status: SHIPPED
Branch: `origin/prompt-1670-bot-debug-overlay-ac5-ruling-prep`
Worker commit: `62ebbfc5`
Integration refresh commit: `973ac547`

## Summary

PROMPT 1670 updated `production/epics/bot-and-autoplay/story-005-bot-debug-overlay.md`
to remove stale "not yet on origin/main" language and record the real landed
implementation lineage for the bot debug overlay.

The worker branch did not contain this report file despite the DONE relay
claiming it existed, so the orchestrator reconstructed this report during the
integration refresh from the worker commit, commit message, and story diff.

## Changes

- Added implementation-landed status for the bot debug overlay.
- Added lineage through PROMPT 1614, 1617, 1618, 1623, 1628, 1630, 1632, and
  1635.
- Added an AC status table:
  - AC1 PASS
  - AC2 PASS in code, with live visual verification advisory
  - AC3 PASS
  - AC4 PASS
  - AC5 NEEDS HUMAN RULING
  - AC6 PASS
  - AC7 PASS
- Added AC5 ruling options for the future `/story-readiness` decision:
  - Option A: accept runtime env-gating as satisfying AC5.
  - Option B: update AC5 wording to match the shipped runtime-gated design.
  - Option C: require true compile-time exclusion before story-done.

## Non-Claims

- No Sprint 19 activation.
- No `/story-readiness`.
- No `/story-done`.
- No AC5 ruling made by this prompt.
- No code changes.
- No Cargo or GUI verification claimed.

## Validation

- Worker commit changed only:
  - `production/epics/bot-and-autoplay/story-005-bot-debug-overlay.md`
- Integration refresh cherry-picked the worker commit cleanly onto current
  `origin/main`.
- `git diff --check` passed during orchestrator refresh.

1670: BOT-DEBUG-OVERLAY-STORY-STATUS-AC5-RULING-PREP: SHIPPED
