# Story 012: S18-OPPONENT-DISCONNECT-BROADCAST-001 -- Wire Server Send-Site for S2COpponentDisconnected (F-01 Close)

> **Epic**: Game Session System
> **Story ID**: `S18-OPPONENT-DISCONNECT-BROADCAST-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Server -- RSM disconnect handling + Lightyear send-site
> **Type**: Logic + Integration (server send-site + protocol drain test)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 Sprint 18 plan draft)
> **Implementing PROMPT**: 1211 -- `fix(s18-opponent-disconnect-broadcast): wire server send-site for S2COpponentDisconnected`
> **Implementing commit**: `dbacb85`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1202 F-01 (protocol orphan -- `S2COpponentDisconnected` defined but never sent)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `dbacb85`**. It exists
so that `/story-done` paperwork has a concrete target after Sprint 18
activation. PROMPT 1296 makes **no** code changes, no test changes, no
Cargo/CI changes, and no sprint-status / stage / session-state /
sprint-plan / QA-plan mutations.

Sprint 18 is **NOT activated** by this authoring run. Top-level
`sprint: 17`, `status: closed-with-conditions`, `stage: Polish` are
preserved verbatim.

PROMPT 1296 makes **NONE** of the standard non-claims: no public-release
readiness, no RC, no full-game completion, no `QA-COND-0005` /
`QA-COND-0006` advancement, no `PAW-TD-*-a` final-art completion, no
`Polish -> Release` retry, no stage advance, no `S8-QA-001-W1` closure,
no `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure, no Sprint 17 close-out
reopen.

---

## Source Finding

**PROMPT 1202 F-01**: `S2COpponentDisconnected` was defined and
registered in `shared/src/protocol.rs` but had **zero server-side
producer**. Clients drained the buffer (`drain_opponent_disconnected_receiver_system`)
but the buffer was always empty. The disconnect path therefore never
surfaced opponent-side feedback in PLACEMENT / DRAFT_SHOP / DRAFT_AUCTION
rounds.

PROMPT 1211 (`dev-story`) wired the missing send-site: server RSM
detects the disconnect transition and emits
`S2COpponentDisconnected { player_id }` via the existing
`ServerMultiMessageSender` path so the remaining client receives it
before the lobby tear-down.

---

## Landed Evidence (commit `dbacb85`, PROMPT 1211)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `server/src/core/rsm/events.rs` | New disconnect-edge event variant. |
| `server/src/core/rsm/plugin.rs` | Plugin registration of the new transition. |
| `server/src/core/rsm/state.rs` | State field carrying the disconnect target. |
| `server/src/core/rsm/transitions.rs` | Send-site for `S2COpponentDisconnected`. |
| `server/src/network/mod.rs` | Module wiring for dispatch. |
| `server/src/network/rsm_dispatch.rs` | Lightyear send wrapper. |
| `tests/invariants/protocol_completeness_test.rs` | F-01 row flipped from orphan to wired. |
| `tests/integration/network/opponent_disconnect_dispatch_test.rs` | Integration evidence (new file). |
| `tests/unit/rsm/rsm_disconnect_test.rs` | Unit coverage extended. |

The integration test exercises the real `ServerMultiMessageSender ->
MessageReceiver<S2COpponentDisconnected>` path; the unit tests cover
the RSM edge.

---

## Acceptance Criteria (evidence-binding, closure-oriented)

These ACs are **retro** -- they describe behaviour already on
`origin/main`. They are written so `/story-done` paperwork can confirm
the landed evidence still satisfies them at the Sprint 18 activation
tip. If activation HEAD diverges and an AC is no longer satisfied,
readiness MUST return NEEDS_WORK rather than auto-closing.

- [ ] **AC1 -- Send-site present**: `server/src/core/rsm/transitions.rs`
  contains a write of `S2COpponentDisconnected { player_id }` via the
  shared `ServerMultiMessageSender` (or equivalent) when the RSM
  detects an opponent disconnect during an active round.
- [ ] **AC2 -- F-01 protocol orphan closed**: `tests/invariants/protocol_completeness_test.rs`
  classifies `S2COpponentDisconnected` as a wired message (no longer
  on the orphan allowlist).
- [ ] **AC3 -- Integration test PASS**:
  `tests/integration/network/opponent_disconnect_dispatch_test.rs`
  drives a two-app server + client fixture, triggers a disconnect, and
  asserts the surviving client receives exactly one
  `S2COpponentDisconnected { player_id }` matching the disconnected peer.
- [ ] **AC4 -- Unit RSM coverage**:
  `tests/unit/rsm/rsm_disconnect_test.rs` asserts the disconnect
  transition produces the send-side event variant in the expected RSM
  arms (active round vs. lobby).
- [ ] **AC5 -- No regression in adjacent server tests**: `server/tests/`
  remains green at the Sprint 18 activation tip (verification at
  `/story-done` time; no new tests required by this story).
- [ ] **AC6 -- ADR-002 + ADR-008 preserved**: client remains read-only
  over the disconnect-broadcast surface; reliable channel discipline
  preserved. Verified by grep over `client/` for any
  `ResMut<S2COpponentDisconnected>` or new send-site (zero hits).
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied on `origin/main` at the Sprint 18 activation tip.
  If a regression has reverted any AC, `/story-readiness` MUST return
  NEEDS_WORK and a follow-on implementation prompt is required before
  closure.

---

## Out of Scope

- Client-side UX for the disconnect signal (toast, modal, lobby
  return). Owned by `S18-OPPONENT-DISCONNECT-LOBBY-RETURN-001` /
  related playable-client stories; not folded here.
- Reconnect / rejoin flow. Owned by `playable-client` story-021
  (`conn-lost-ux.md`) and game-session-system story-007 (reconnect
  snapshot).
- Sprint 17 close-out reopen.
- Sprint 18 activation.
- Any Polish -> Release gate retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. File touched by this authoring run: this
  file (NEW) and `production/epics/game-session-system/EPIC.md` (table
  row added; status counts not refreshed). No code change. No `cargo`,
  no `trunk`, no `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan` invoked.
  Implementation already landed via PROMPT 1211 at commit `dbacb85`
  prior to this authoring; this stub does not re-author or alter that
  work.
