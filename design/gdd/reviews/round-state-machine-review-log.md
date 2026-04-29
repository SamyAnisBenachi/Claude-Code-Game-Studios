# Review Log — Round State Machine

## Review — 2026-04-29 — Verdict: APPROVED (post-revision)

**Scope signal:** L
**Specialists:** game-designer, systems-designer, qa-lead, network-programmer, creative-director
**Blocking items:** 13 resolved | **Recommended:** 10 noted | **Nice-to-have:** 5 noted
**Prior verdict resolved:** N/A — first review

**Summary:** The RSM's 7-state machine was structurally sound but had thirteen blocking gaps. The most critical were: RESOLUTION lacking a safety timeout and violating the "No idle spectating" pillar (resolved with `resolution_max_duration_seconds` and explicit `OnResolutionEnd` contract); the disconnect grace window at 5s being below normal browser behavior for a WASM target (raised to 30s); a Rule 5/F2 inconsistency that left DRAFT_INITIAL shop population mechanism undefined; `StartAuction` missing from the F2 entry sequence; the `S2CGameOver` protocol message and `GameOverReason` enum being absent; and RSM-31 (double-transition = double-combat) being marked ADVISORY when it is the most dangerous correctness invariant in the machine. Seven missing acceptance criteria were added (RSM-32–38) covering entry ordering, round_number invariant, simultaneous-event races, mid-RESOLUTION disconnect deferral, GAME_OVER payload correctness, mutual disconnection Draw, and the RESOLUTION safety timeout path.

**Open questions remaining:** lobby_timeout_seconds (no default), DRAFT_INITIAL gold forfeiture confirmation with Economy System, multiplayer auction card count, late-joiner snapshot strategy (must become an ADR before Economy/Board/Auction GDDs are finalised).
