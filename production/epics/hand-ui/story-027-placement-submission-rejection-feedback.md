# Story 027: S18-PLACEMENT-SUBMISSION-REJECTION-FEEDBACK-001 -- Unicast S2CPlacementRejected on Every Server Rejection

> **Epic**: Hand UI
> **Story ID**: `S18-PLACEMENT-SUBMISSION-REJECTION-FEEDBACK-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Server (rejection send-site) + Presentation (Hand UI receiver-drain surface)
> **Type**: Logic + Integration (new S2C message + client surface + integration drain)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPTs**: 1244 (`dev-story`) + 1259 (integrate)
> **Implementing commits**: `7105b78` + `c703fdf`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1203 B-1203-PLA-02 (no server -> client feedback on placement rejection); PROMPT 1127 §1.6 (server-side placement loss surface)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main`** at `7105b78` (dev-story)
and `c703fdf` (integrate). PROMPT 1296 makes **no** code, test, Cargo,
CI, sprint, QA, or session-state mutations. Sprint 18 is **NOT
activated** by this authoring run. All standard non-claims preserved
verbatim.

The cross-epic placement is **hand-ui** because the client-side
surface is the Hand UI submit / placement pipeline; the server send-
site is the support that enables the hand-ui feedback. Server-side
files are listed in landed evidence but the hand-ui display surface
is the user-visible outcome.

---

## Source Finding

**PROMPT 1203 B-1203-PLA-02 / PROMPT 1127 §1.6**: when the server
rejected a `C2SSubmitPlacement` (invalid cell, mana overspend,
placement-window already closed, etc.), it logged the rejection and
silently dropped the batch. The client had **no** signal that the
submission failed -- the staged cards simply vanished at the
Placement -> Resolution edge without explanation. Players consistently
interpreted this as a bug ("my cards disappeared") rather than a
validation failure.

PROMPT 1244 (`dev-story`) + PROMPT 1259 (integrate) added:

1. A new `S2CPlacementRejected` protocol message carrying the reject
   reason (or reject-category enum).
2. A server-side send-site on every rejection arm of the placement-
   validation pipeline, unicast back to the submitting player.
3. A client-side Hand UI surface that consumes the receiver, surfaces
   the rejection to the player (toast / inline / equivalent), and
   restores the staged-cards state so the player can retry.

---

## Landed Evidence (commits `7105b78` + `c703fdf`)

Files touched by the implementing commits:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `client/src/ui/hand/mod.rs` | Client receiver drain + rejection surface. |
| `server/Cargo.toml` | Cargo wiring for the new test target. |
| `server/src/feature/board/mod.rs` | Send-site wiring. |
| `server/src/feature/board/placement.rs` | Rejection arms now emit `S2CPlacementRejected`. |
| `server/src/feature/board/plugin.rs` | Plugin registration. |
| `server/src/network/mod.rs` | Network module wiring. |
| `shared/src/protocol.rs` | New `S2CPlacementRejected` protocol type + registration. |
| `tests/integration/board-lane-system/placement_buffer_test.rs` | Coverage extended. |
| `tests/integration/board-lane-system/placement_rejection_feedback_test.rs` (NEW) | 589 LOC integration test exercising server->client rejection drain. |
| `tests/integration/board-lane-system/placement_submission_repair_test.rs` | Coverage extended. |
| `tests/integration/hand-ui/hand_ui_placement_rejection_test.rs` (NEW) | 322 LOC integration test for client-side surface. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Protocol type exists**: `shared/src/protocol.rs`
  contains a `S2CPlacementRejected` message variant (or equivalent
  enum carrying the reject reason / category) registered on the
  reliable channel per ADR-008.
- [ ] **AC2 -- Server send-site on every rejection arm**:
  `server/src/feature/board/placement.rs` emits
  `S2CPlacementRejected { ..., player_id }` on every validation-
  failure arm of the placement-submission handler. No silent drop
  path remains.
- [ ] **AC3 -- Client receiver drain present**:
  `client/src/ui/hand/mod.rs` contains a system that drains
  `MessageReceiver<S2CPlacementRejected>` and surfaces the rejection
  to the player via the Hand UI (toast / inline / equivalent
  visible cue).
- [ ] **AC4 -- Staged state restored on reject**: on rejection the
  Hand UI restores the staged-cards state so the player can correct
  and resubmit. No staged card is silently destroyed by the reject
  signal alone.
- [ ] **AC5 -- F-08 anti-pattern avoided**: the new tests exercise the
  real `MessageReceiver<S2CPlacementRejected>` drain path (not a
  direct `Presentation*Message` write).
- [ ] **AC6 -- Integration tests PASS**:
  `tests/integration/board-lane-system/placement_rejection_feedback_test.rs`
  and `tests/integration/hand-ui/hand_ui_placement_rejection_test.rs`
  PASS at the Sprint 18 activation tip; existing
  `placement_buffer_test.rs` and `placement_submission_repair_test.rs`
  PASS with updated assertions.
- [ ] **AC7 -- ADR-002 + ADR-008 preserved**: server is authoritative
  over the rejection signal; reliable-channel discipline preserved.
  Client never invents a rejection; it surfaces what the server sent.
- [ ] **AC8 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC7
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Server-side placement validation rule changes (this story carries
  no new game-logic rule, only the rejection signal for existing
  rules).
- Reject-reason copy-writing / final UX polish on the toast / inline
  surface. A future Polish candidate may revisit the wording / VFX;
  this story carries the minimum surface needed to unbreak the
  player feedback loop.
- Auto-resubmit / auto-correct of invalid placements. Out of scope --
  player must consciously fix the input.
- AUDIT-1076-02 / AUDIT-1076-03 server-side placement loss. This
  story addresses the *feedback* gap, not the underlying loss path
  (which may also be patched by the RSM grace window in story 007 of
  round-state-machine, separate landed work).
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (table row added). Implementation
  landed via PROMPT 1244 + 1259 at `7105b78` + `c703fdf` prior to
  this authoring; this stub does not re-author or alter that work.
