# Story 023: S18-SHOP-AUCTION-TIMER-COST-CLARITY-001 -- Surface Refresh Cost + Server-Anchored Auction Timer

> **Epic**: Shop / Auction UI
> **Story ID**: `S18-SHOP-AUCTION-TIMER-COST-CLARITY-001`
> **Status**: Draft -- Sprint 18 candidate / retro paperwork; NOT activated
> **Layer**: Presentation -- Shop / Auction UI (`client/src/ui/shop_auction/mod.rs`)
> **Type**: Logic + Integration (refresh-cost label + server-anchored auction timer)
> **Authored**: 2026-05-18 by PROMPT 1296 (landed-paperwork story-authoring wave)
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
> **Implementing PROMPT**: 1245 -- `dev-story(s18-shop-auction-timer-cost-clarity): surface refresh cost + server-anchor auction timer`
> **Implementing commit**: `418fe92`
> **Source audit**: PROMPT 1287 §2 (already-landed inventory); PROMPT 1203 B-1203-DSH-04 (shop refresh-cost not surfaced); PROMPT 1203 B-1203-DAU-02 (auction timer drift vs server)

---

## Status / No-Claim Banner

This story is a **retro paperwork stub** authored by PROMPT 1296 covering
work that **already landed on `origin/main` at `418fe92`**. PROMPT 1296
makes **no** code, test, Cargo, CI, sprint, QA, or session-state
mutations. Sprint 18 is **NOT activated** by this authoring run. All
standard non-claims preserved verbatim.

---

## Source Finding

**PROMPT 1203 B-1203-DSH-04**: the shop "Refresh" button had no
visible cost. Players had to know via tribal knowledge / the GDD how
many gold a refresh consumed.

**PROMPT 1203 B-1203-DAU-02**: the auction timer was driven entirely
client-side from a local `Time<Virtual>` driver. On any frame-rate
hitch or `Time<Virtual>` divergence the client timer drifted from the
server's authoritative phase timer, producing late or early auction
closes from the player's perspective.

PROMPT 1245 (`dev-story`) did two things in one commit:

1. Added a `Refresh (N gold)` label on the shop refresh button (or
   equivalent inline cost display) so players see the cost before
   clicking.
2. Anchored the auction timer display to the server-replicated phase
   timer rather than a client-local `Time<Virtual>` driver, so it
   matches the authoritative state.

---

## Landed Evidence (commit `418fe92`, PROMPT 1245)

Files touched by the implementing commit:

| Path | Role |
|------|------|
| `client/Cargo.toml` | Cargo wiring for the new test target. |
| `client/src/ui/shop_auction/mod.rs` | Refresh-cost label + server-anchored timer mirror. |
| `tests/integration/shop_auction_ui/refresh_label_and_timer_anchor_test.rs` (NEW) | 345 LOC integration test. |
| `tests/integration/shop_auction_ui/shop_panel_test.rs` | Existing test updated to expect cost label. |

---

## Acceptance Criteria (evidence-binding, closure-oriented)

- [ ] **AC1 -- Refresh-cost label present**:
  `client/src/ui/shop_auction/mod.rs` spawns a `Text` node (or label
  segment) on the shop refresh button reading the current refresh
  cost in gold. The value is sourced from the authoritative
  `GameConfig` / shared formula -- not a hardcoded literal.
- [ ] **AC2 -- Cost reactive to phase / round**: if the refresh-cost
  formula scales with round number, the label updates accordingly
  (verified by integration test driving round transitions).
- [ ] **AC3 -- Auction timer anchored to server source**: the auction
  timer text reads the authoritative server-replicated phase-timer
  surface (the same source story 022 DraftInitial countdown uses) --
  not a client-local `Time<Virtual>` accumulator independent of
  server state.
- [ ] **AC4 -- No drift on simulated hitch**: integration test asserts
  that injecting a virtual time spike on the client does not desync
  the auction timer display from the authoritative phase timer.
- [ ] **AC5 -- Integration test PASS**:
  `tests/integration/shop_auction_ui/refresh_label_and_timer_anchor_test.rs`
  PASSES at the Sprint 18 activation tip; existing
  `shop_panel_test.rs` continues to pass with updated assertions.
- [ ] **AC6 -- ADR-002 + ADR-009 preserved**: client mirrors only; no
  new C2S/S2C protocol surface; no second `MessageReceiver<S2CPhaseChanged>`
  drain.
- [ ] **AC7 -- /dev-story disposition**: **NOT REQUIRED** if AC1..AC6
  remain satisfied at the Sprint 18 activation tip. If regression,
  `/story-readiness` MUST return NEEDS_WORK; follow-on implementation
  required before closure.

---

## Out of Scope

- Auction-won card disposition (story 020, separate landed work via
  different PROMPT lineage; remains a Sprint 18 Must-Have per the
  Sprint 18 plan).
- DraftInitial keep modal countdown (story 022 sibling, separate
  landed work).
- HUD phase-chip disambiguation (story 019 in hud epic, separate
  landed work).
- Shop / Auction visual chrome upgrades (story 014, separate).
- Sprint 18 activation, stage advance, gate-check retry.

---

## Authoring Trail

- 2026-05-18 -- PROMPT 1296 -- Retro paperwork stub authored against
  `origin/main@1345c6b`. Files touched: this file (NEW) and
  `production/epics/shop-auction-ui/EPIC.md` (table row added).
  Implementation landed via PROMPT 1245 at `418fe92` prior to this
  authoring; this stub does not re-author or alter that work.
