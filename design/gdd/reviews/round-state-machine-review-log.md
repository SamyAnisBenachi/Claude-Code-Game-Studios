# Review Log — Round State Machine

## Review — 2026-05-01 (R2 full) — Verdict: APPROVED
Scope signal: M
Specialists: game-designer, systems-designer, network-programmer, qa-lead, creative-director
Blocking items: 13 resolved inline | Recommended: 14 noted
Summary: R2 surfaced issues invisible to R1 because the Network Protocol GDD was still in revision when R1 ran. The four convergence findings (flagged by 2+ specialists): Rule 13's "(not a custom heartbeat message)" parenthetical directly contradicts NP Rule 8 and made RSM-23/25 untestable; F2 phase entry has no Bevy scheduling pins (silent runtime bug class per NP GDD's own warning); S2CPhaseChanged.timer_duration_ms uses u32 with 0 as sentinel while S2CGameSnapshot uses Option<u32> for the same semantic; RSM-34 and RSM-38 were misclassified as ADVISORY despite being fully testable Logic story invariants. Additional blockers: Rule 7 Auction System IDLE invariant not enforced (S2CPhaseChanged could fire with no auction running); auction_max_duration_seconds safe range lower bound contradicted the formula minimum stated in the same document; five missing ACs (trigger_index ordering, slot persistence, R=0 guard, GAME_OVER snapshot ordering, auction-followup timer). Creative-director verdict: fundamentally sound state machine; prior approval was premature due to NP GDD dependency gap. All corrections are bounded — no redesign required. RSM and NP should be re-verified together when either changes.
Prior verdict resolved: Yes — R1 APPROVED (2026-04-29) was a correct post-revision approval but lacked cross-verification with the not-yet-approved NP GDD.

## Review — 2026-04-29 — Verdict: APPROVED (post-revision)

**Scope signal:** L
**Specialists:** game-designer, systems-designer, qa-lead, network-programmer, creative-director
**Blocking items:** 13 resolved | **Recommended:** 10 noted | **Nice-to-have:** 5 noted
**Prior verdict resolved:** N/A — first review

**Summary:** The RSM's 7-state machine was structurally sound but had thirteen blocking gaps. The most critical were: RESOLUTION lacking a safety timeout and violating the "No idle spectating" pillar (resolved with `resolution_max_duration_seconds` and explicit `OnResolutionEnd` contract); the disconnect grace window at 5s being below normal browser behavior for a WASM target (raised to 30s); a Rule 5/F2 inconsistency that left DRAFT_INITIAL shop population mechanism undefined; `StartAuction` missing from the F2 entry sequence; the `S2CGameOver` protocol message and `GameOverReason` enum being absent; and RSM-31 (double-transition = double-combat) being marked ADVISORY when it is the most dangerous correctness invariant in the machine. Seven missing acceptance criteria were added (RSM-32–38) covering entry ordering, round_number invariant, simultaneous-event races, mid-RESOLUTION disconnect deferral, GAME_OVER payload correctness, mutual disconnection Draw, and the RESOLUTION safety timeout path.

**Open questions remaining:** lobby_timeout_seconds (no default), DRAFT_INITIAL gold forfeiture confirmation with Economy System, multiplayer auction card count, late-joiner snapshot strategy (must become an ADR before Economy/Board/Auction GDDs are finalised).
