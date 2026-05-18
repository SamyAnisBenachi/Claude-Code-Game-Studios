# Story 026: S18-PLACEMENT-AUTO-SUBMIT-CLIENT-001 -- Queue Final C2SSubmitPlacement on Placement->Resolution Edge

> **Epic**: Hand UI
> **Story ID**: `S18-PLACEMENT-AUTO-SUBMIT-CLIENT-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Presentation -- Hand UI placement-phase exit hook (`client/src/ui/hand/mod.rs`)
> **Type**: Logic + Integration (phase-transition handler + drain test)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPT**: 1226 -- `dev-story(s18-placement-auto-submit-client): queue final C2SSubmitPlacement on Placement->Resolution edge`
> **Implementing commit**: `0934ea3`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1201 HUNT-1201-14 (placement auto-submit gap on timer expiry)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `0934ea3`**. PROMPT 1296
makes **no** code, test, Cargo, CI, sprint, QA, or session-state
mutations. Sprint 18 is **NOT activated** by this authoring run. All
standard non-claims preserved verbatim.

---

## Source Finding

**PROMPT 1201 HUNT-1201-14**: when the placement timer expired (or the
server advanced Placement -> Resolution before the client clicked
Submit), the client never sent the staged batch. Server placement was
therefore lost on the client-vs-server race even though the staged
cards were visible on the client side. The user-visible symptom was
"my staged cards disappeared without resolving" at the round boundary.

PROMPT 1226 (`dev-story`) added a client-side hook that queues a final
`C2SSubmitPlacement` automatically on the **observed**
Placement -> Resolution phase edge (read from `Res<CurrentClientPhase>`,
the canonical ADR-009 surface) so the staged batch survives the round
boundary. The hook fires exactly once per round, idempotent against
prior manual submits.

---

## Landed Evidence (commit `0934ea3`, PROMPT 1226)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `client/src/ui/hand/mod.rs` | Phase-transition observer + idempotent final-submit queue. |
| `tests/integration/hand-ui/hand_ui_placement_auto_submit_phase_transition_test.rs` (NEW) | 359 LOC integration test covering all relevant edges. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Auto-submit on phase edge**: `client/src/ui/hand/mod.rs`
  contains a system gated on `Res<CurrentClientPhase>` change from
  `Phase::Placement` to `Phase::Resolution` (or the equivalent guard
  pattern) that queues a final `C2SSubmitPlacement` with the current
  `PendingPlacements` payload exactly once per round.
- [ ] **AC2 -- Idempotent against manual submit**: if the user clicked
  Submit before the edge fired, the auto-submit hook MUST NOT enqueue
  a duplicate `C2SSubmitPlacement`. Verified by integration test
  assertion on `MessageWriter<C2SSubmitPlacement>` send-count == 1.
- [ ] **AC3 -- No send on phase entry**: the hook does NOT fire when
  the client first enters `Phase::Placement`; it fires only on the
  exit edge. Verified by integration test scenarios covering both
  entry and exit.
- [ ] **AC4 -- ADR-009 preserved**: the hook reads
  `Res<CurrentClientPhase>` only; it does NOT drain
  `MessageReceiver<S2CPhaseChanged>` (single-drain discipline owned by
  the shared `phase_sink_system`).
- [ ] **AC5 -- ADR-002 preserved**: client never invents a placement
  rejection; the auto-submit is a client-side convenience, not a new
  authority surface. Server retains final say on accept/reject (see
  sibling story 027 placement-submission-rejection-feedback).
- [ ] **AC6 -- Integration test PASS**:
  `tests/integration/hand-ui/hand_ui_placement_auto_submit_phase_transition_test.rs`
  PASSES at the Sprint 18 activation tip, covering: empty staged
  batch (no send), non-empty staged batch (single send), prior manual
  submit (no duplicate), pre-Placement entry (no send), and post-
  Resolution exit (no send).
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Server-side acceptance of late submissions (owned by RSM grace
  window -- story 007 in round-state-machine, separate landed work).
- Server-side rejection feedback (story 027, separate landed work).
- Drag-cursor coord-space repair (story 025, separate landed work).
- HUD timer urgency animation (existing Story 009 / 011).
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (table row added). Implementation
  landed via PROMPT 1226 at `0934ea3` prior to this authoring; this
  stub does not re-author or alter that work.
