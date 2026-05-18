# Story 007: S18-SERVER-RSM-PLACEMENT-GRACE-001 -- Placement-Timer Grace Window for Late-Arriving Submissions

> **Epic**: Round State Machine
> **Story ID**: `S18-SERVER-RSM-PLACEMENT-GRACE-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Server -- Round State Machine timer + transition (`server/src/core/rsm/`)
> **Type**: Logic (RSM grace-window state + transition arm + unit test)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPT**: 1209 -- `fix(server/rsm): placement-timer grace window for late-arriving submissions`
> **Implementing commit**: `f48583d`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1127 §1.6 (client/server race losing the staged batch at the Placement->Resolution edge)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `f48583d`**. PROMPT 1296
makes **no** code, test, Cargo, CI, sprint, QA, or session-state
mutations. Sprint 18 is **NOT activated** by this authoring run. All
standard non-claims preserved verbatim.

---

## Source Finding

**PROMPT 1127 §1.6**: even when the client did emit a final
`C2SSubmitPlacement` (manually or via the sibling auto-submit hook --
hand-ui story 026), there was a server-side race window where the
RSM had already advanced Placement -> Resolution by the time the
submission arrived. The submission was therefore dropped on the
server side rather than the client side, with no clean signal that
the player's input was late.

PROMPT 1209 (`fix`) added a small grace window inside the RSM
placement-timer arm: after the configured placement duration expires,
the RSM holds in a transient "Placement (grace)" sub-state for a
bounded window before transitioning to Resolution. Submissions
arriving within the grace window are accepted as if they arrived on
time; submissions after the grace expires are still rejected via the
sibling rejection-feedback path (story 027 in hand-ui).

---

## Landed Evidence (commit `f48583d`, PROMPT 1209)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `server/src/core/rsm/state.rs` | Grace-window state field on `RoundState`. |
| `server/src/core/rsm/transitions.rs` | Placement-timer arm extended to hold in grace before Resolution. |
| `server/tests/rsm_timers_test.rs` | 139 LOC of new test coverage exercising the grace window. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Grace state on RoundState**:
  `server/src/core/rsm/state.rs` contains a grace-window field (or
  equivalent transient `PlacementGrace` state) representing the
  bounded post-timer window before Placement -> Resolution.
- [ ] **AC2 -- Transition arm holds in grace**:
  `server/src/core/rsm/transitions.rs` Placement-timer expiry arm
  enters the grace state instead of advancing directly to Resolution;
  Resolution entry is gated on grace expiry.
- [ ] **AC3 -- Submission accepted in grace window**: a
  `C2SSubmitPlacement` arriving during the grace window is processed
  as a normal submission and NOT dropped or rejected.
- [ ] **AC4 -- Submission rejected after grace**: a
  `C2SSubmitPlacement` arriving after grace expires follows the
  rejection path (sibling story 027 / `S2CPlacementRejected`); the
  RSM does NOT process it as accepted.
- [ ] **AC5 -- Bounded window**: the grace duration is bounded by a
  configured value (e.g. `GameConfig`-driven or const), not unbounded.
- [ ] **AC6 -- Tests PASS**: `server/tests/rsm_timers_test.rs`
  PASSES at the Sprint 18 activation tip, including the new grace-
  window coverage added in `f48583d`.
- [ ] **AC7 -- No client-authority leak**: the grace state is purely
  server-side; client RSM mirror is unaffected (ADR-002 preserved).
  Verified by grep: no client code reads / writes the grace state
  field.
- [ ] **AC8 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC7
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Client-side auto-submit hook (sibling: hand-ui story 026, separate
  landed work).
- Client-side rejection feedback surface (sibling: hand-ui story 027
  + new `S2CPlacementRejected` protocol type, separate landed work).
- RSM `submissions_received.clear()` on Placement -> Resolution
  (PROMPT 1287 Lane W9 candidate -- separate, not yet landed).
- AUDIT-1076-02 / AUDIT-1076-03 deeper server-side placement loss
  investigation. This story closes the bounded-window race, not the
  broader loss surface.
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/round-state-machine/EPIC.md` (table row added).
  Implementation landed via PROMPT 1209 at `f48583d` prior to this
  authoring; this stub does not re-author or alter that work.
