# Story 013: S18-SESSION-SETTINGS-ON-JOIN-001 -- Unicast S2CSessionSettingsUpdated to Joiner (F-03 Close)

> **Epic**: Game Session System
> **Story ID**: `S18-SESSION-SETTINGS-ON-JOIN-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Server -- session join handler + Lightyear unicast send-site
> **Type**: Logic + Integration (server send-site + drain test)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 Sprint 18 plan draft)
> **Implementing PROMPT**: 1212 -- `dev-story(s18-session-settings-on-join): unicast S2CSessionSettingsUpdated to joiner`
> **Implementing commit**: `6a18c78`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1202 F-03 (protocol orphan -- `S2CSessionSettingsUpdated` defined but joiner never received it on-entry)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `6a18c78`**. It exists
so that `/story-done` paperwork has a concrete target after Sprint 18
activation. PROMPT 1296 makes **no** code, test, Cargo, CI, sprint, or
QA mutations.

Sprint 18 is **NOT activated** by this authoring run. All standard
non-claims (release, RC, full-game, accessibility, playtest,
PROMPT 761 retry, stage advance) are preserved verbatim.

---

## Source Finding

**PROMPT 1202 F-03**: the second client to join a room never received
`S2CSessionSettingsUpdated` because the server only broadcast the
message when settings *changed*. A fresh joiner therefore observed
default settings on arrival and only saw the truth after the next
broadcast (which, in the friend-game flow, was usually never).

PROMPT 1212 (`dev-story`) added a unicast send on `JoinRoom` so the
joiner receives the **current** `S2CSessionSettingsUpdated` payload
exactly once on entry, in addition to the existing broadcast on
change.

---

## Landed Evidence (commit `6a18c78`, PROMPT 1212)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `server/src/core/session/mod.rs` | Session-join hook surface. |
| `server/src/core/session/system.rs` | Unicast send-site on `JoinRoom`. |
| `server/tests/placement_timer_multiplier_test.rs` | Coverage extended to assert joiner receives settings before placement timer starts. |

The send-site uses the existing single-client unicast path
(`ServerMultiMessageSender::send_to(player_id, ...)`); no new protocol
type, channel, or message structure is introduced.

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Unicast on join**: `server/src/core/session/system.rs`
  sends `S2CSessionSettingsUpdated { settings }` to the joining
  `player_id` exactly once within the join-handling system on receipt
  of `JoinRoom` (or equivalent join trigger).
- [ ] **AC2 -- F-03 protocol orphan closed**: the protocol-completeness
  invariant test classifies `S2CSessionSettingsUpdated` as
  send-on-join-path wired (no longer flagged as never-sent-on-join).
- [ ] **AC3 -- Integration coverage**:
  `server/tests/placement_timer_multiplier_test.rs` (and any sibling
  added in `6a18c78`) drives a multi-client join sequence and asserts
  the joiner's `MessageReceiver<S2CSessionSettingsUpdated>` contains
  exactly one frame matching the current server settings prior to the
  first placement-timer tick.
- [ ] **AC4 -- No duplicate broadcast regression**: existing broadcast
  on settings-change is preserved; the joiner receives one unicast on
  join AND one broadcast on each subsequent change (no dedupe leak
  introduced).
- [ ] **AC5 -- ADR-002 + ADR-008 preserved**: client remains read-only;
  reliable-channel discipline preserved.
- [ ] **AC6 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC5
  remain satisfied on `origin/main` at the Sprint 18 activation tip.
  If a regression has reverted any AC, `/story-readiness` MUST return
  NEEDS_WORK; a follow-on implementation prompt is required before
  closure.

---

## Out of Scope

- New `S2CSessionSettings*` message variants.
- Session-settings UI on the client (settings panel layout / etc).
- Multi-room or matchmaking scope.
- Sprint 18 activation, Polish -> Release retry, stage advance.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/game-session-system/EPIC.md` (table row added).
  Implementation landed via PROMPT 1212 at `6a18c78` prior to this
  authoring; this stub does not re-author or alter that work.
