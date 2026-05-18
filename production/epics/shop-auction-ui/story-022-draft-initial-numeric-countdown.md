# Story 022: S18-DRAFT-INITIAL-NUMERIC-COUNTDOWN-001 -- Add Modal-Local Live Seconds Readout

> **Epic**: Shop / Auction UI
> **Story ID**: `S18-DRAFT-INITIAL-NUMERIC-COUNTDOWN-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Presentation -- DraftInitial modal (`client/src/ui/shop_auction/mod.rs`)
> **Type**: Logic + Integration (live-countdown text + integration test)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPT**: 1230 -- `dev-story(s18-draft-initial-numeric-countdown): add modal-local live seconds readout`
> **Implementing commit**: `1eac32f`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1201 HUNT-1201-04; PROMPT 1203 B-1203-PLA-07

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `1eac32f`**. PROMPT 1296
makes **no** code, test, Cargo, CI, sprint, QA, or session-state
mutations. Sprint 18 is **NOT activated** by this authoring run. All
standard non-claims preserved verbatim.

---

## Source Finding

**PROMPT 1201 HUNT-1201-04 / PROMPT 1203 B-1203-PLA-07**: the
DraftInitial 9-card keep modal showed a static "X seconds to keep"
hint but no live countdown. Players had to glance at the HUD phase
chip / general timer to know how much time was left, which on small
viewports was either occluded by the modal or not legible while
keep-selection was in flight.

PROMPT 1230 (`dev-story`) added a modal-local live seconds readout
that ticks once per second off the same authoritative timer source the
HUD reads (no new server protocol; the timer is sourced from
`Res<RoundPhaseTimer>` / the existing phase-timer mirror). The readout
sits inside the DraftInitial modal panel so it remains visible while
the player interacts with the 9-card grid.

---

## Landed Evidence (commit `1eac32f`, PROMPT 1230)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `client/src/ui/shop_auction/mod.rs` | DraftInitial modal countdown text + reactive system. |
| `tests/integration/shop_auction_ui/draft_initial_countdown_test.rs` (NEW) | 255 LOC integration test covering readout updates and reset on modal close. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Countdown text inside DraftInitial modal**:
  `client/src/ui/shop_auction/mod.rs` spawns a `Text` node parented
  to the DraftInitial modal panel root, reactive to the authoritative
  per-phase timer source.
- [ ] **AC2 -- 1Hz tick cadence**: the readout updates at >= 1Hz
  granularity (one update per integer-second boundary, or finer at the
  worker's discretion). Verified by integration test asserting a
  monotonically non-increasing seconds string across ticks.
- [ ] **AC3 -- Reset on modal close**: when the DraftInitial modal
  becomes `Visibility::Hidden` (phase exit / keep-confirm), the
  countdown text is hidden / despawned with the modal -- no orphan
  countdown survives one frame past modal close.
- [ ] **AC4 -- Single timer source**: the countdown reads the same
  authoritative phase-timer surface the HUD reads (no second timer,
  no new `Res<Time<Virtual>>` driver). ADR-009 phase-state discipline
  preserved.
- [ ] **AC5 -- Integration test PASS**:
  `tests/integration/shop_auction_ui/draft_initial_countdown_test.rs`
  PASSES at the Sprint 18 activation tip, covering: initial value
  match, tick decrement, modal close cleanup, and phase re-entry.
- [ ] **AC6 -- ADR-002 + ADR-021 preserved**: client never mutates
  the authoritative timer; the countdown is a derived display
  projection. Plugin registration order unchanged.
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- HUD phase-chip disambiguation (story 019 in hud epic, separate
  landed work).
- Shop / Auction timer + refresh-cost clarity (sibling story 023,
  separate landed work).
- DraftInitial grid migration / overflow hardening (ui-clean-pass
  stories 022 + 026, separate).
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/shop-auction-ui/EPIC.md` (table row added).
  Implementation landed via PROMPT 1230 at `1eac32f` prior to this
  authoring; this stub does not re-author or alter that work.
