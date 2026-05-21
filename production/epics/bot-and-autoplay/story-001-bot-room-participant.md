# Story 001: BOT-ROOM-PARTICIPANT-001 -- Bot Room Participant (Join + Class Confirm + Action Loop)

> **Epic**: Bot & Autoplay
> **Story ID**: BOT-ROOM-PARTICIPANT-001
> **Status**: Draft -- ledger placeholder for landed work (story-done paperwork deferred to Sprint 19)
> **Layer**: Integration -- server-side bot participant + lobby flow
> **Type**: Integration -- bot joins a real game room through real C2S/S2C messages
> **Sprint**: Sprint 18 carry-tracking (Sprint 19 candidate for `/story-done` paperwork)
> **Authored**: 2026-05-21 by PROMPT 1608
> **Authoring source-of-truth**: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`

---

## Status / No-Claim Banner

This story is authored as a **ledger placeholder** for bot-participant
work that has already landed on `origin/main` across PROMPT 1430 (bot
protocol room foundations integration), PROMPT 1439 (bot foundation
scaffold main-land), PROMPT 1531 / 1582 (bot participant action loop
Waves 1 + 2), PROMPT 1583 (bot lobby ready auto-confirm), and PROMPT
1602 (Wave 3 placement heuristic main-land). It is **NOT closed** by
PROMPT 1608.

PROMPT 1608 (this authoring run) does NOT:

- Activate Sprint 19.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, build scripts, or CI workflows.
- Claim closure of any PROMPT listed above.
- Retry the PROMPT 761 Polish→Release gate-check.

Non-claims preserved verbatim: NO public release readiness, NO
release-candidate readiness, NO full game completion, NO Standard-tier
accessibility advancement (`QA-COND-0005`), NO playtest validation
advancement (`QA-COND-0006`), NO full playable-client manual QA, NO
`S8-QA-001-W1` closure, NO `PAW-TD-*-a` final-art completion, NO stage
advance from `Polish`.

---

## Problem Class / Prevention Target

**Defect class**: Bot participants were landing on `origin/main`
through a series of orchestrator PROMPTs without a single story file
owning the cumulative behaviour or the `/story-done` evidence chain.
This makes regression-tracking and Sprint 19 close-out paperwork
ambiguous.

**Prevention target**: One story owns the cumulative bot-participant
surface so:

- Future Wave N additions slot under a known story ID instead of
  re-spawning ad-hoc PROMPTs.
- Sprint 19 close-out paperwork can attach `/story-done` evidence to
  this row instead of leaving the work uncatalogued.
- QA can write regression tests against a named acceptance set.

---

## Acceptance Criteria (ledger; not gated by PROMPT 1608)

These criteria are written to be exercised at Sprint 19 readiness /
`/story-done` time. They are **not validated by PROMPT 1608.**

- AC1: Bot can join a real game room (host or join side) via the same
  C2S handshake a human client uses; server reports bot as a real
  participant in lobby state.
- AC2: Bot picks a class and confirms the class within the existing
  lobby ready window without manual operator input.
- AC3: Bot reaches `SessionReady` through the same Observer path as a
  human participant (ADR-012).
- AC4: Bot bids in auctions following the Wave 2 / 2.5 bid-decision
  heuristic; bids are server-validated like any other client bid.
- AC5: Bot places units following the Wave 3 placement heuristic
  (PROMPT 1602) and produces server-accepted placement ACKs.
- AC6: Bot decision telemetry is captured in the QA snapshot stream
  (PROMPT 1597) and the streamed bot-decision log for at least one
  full round.
- AC7: A real human client can complete a friend-game round against
  the bot (DRAFT_INITIAL → DRAFT_SHOP → AUCTION → PLACEMENT →
  RESOLUTION → next-loop) without server panic.
- AC8: Bot lifecycle (join, disconnect, re-join) does not corrupt
  room or session state.

---

## Implementation Lineage on `origin/main`

| PROMPT | Commit (or branch) | Slice |
|---|---|---|
| 1430 | bot-protocol-room-foundations integration | Protocol room foundations |
| 1439 | bot-player-foundation-scaffold main-land | Foundation scaffold |
| 1531 | bot-participant-action-loop Wave 1 worker | Action loop Wave 1 |
| 1582 | bot-action-loop-wave2-auction-bid worker | Auction bid decision (Wave 2) |
| 1583 | bot-lobby-ready-auto-confirm worker | Lobby ready auto-confirm |
| 1598 | bot-flow-auction-bid-funnel-wave-2-5 worker | Bid funnel Wave 2.5 |
| 1602 | `origin/main@576fbe8c` | Wave 3 placement heuristic |

Open follow-up items (not blocking this story's ledger placement):

- Bot disconnect / re-join hardening (not yet a story).
- Bot vs full reconnect-snapshot path (not yet a story).
- Bot decision telemetry schema freeze (not yet a story).

---

## Recommended Sprint 19 Follow-Up Prompts

1. `/story-readiness BOT-ROOM-PARTICIPANT-001` against the Sprint 19
   activation tip.
2. `/story-done BOT-ROOM-PARTICIPANT-001` once readiness clears, citing
   the PROMPT lineage above as the evidence chain.
3. (Optional) A scoped repair prompt if any AC fails readiness.

---

## Test Evidence (target)

- Logic: bot decision unit tests under `tests/unit/bot/` (some already
  exist via PROMPT 1531 / 1582 / 1602; this story does NOT inventory
  them).
- Integration: bot-occupied friend-game integration test or QA-snapshot
  evidence under `production/qa/evidence/` (deferred to Sprint 19).
- Manual: orchestrator-driven friend-game smoke against a bot opponent
  (deferred to Sprint 19; cross-references story 004 Autoplay-vs-Bot
  QA flow).
