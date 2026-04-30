# Epic: Card Acquisition

> **Layer**: Feature (M2)
> **GDD**: design/gdd/card-acquisition.md
> **Architecture Module**: `server/feature/acquisition/`
> **Status**: Ready
> **Stories**: 6 stories created 2026-04-30

## Stories

| # | Story | Type | Status | TR-IDs | ADR |
|---|-------|------|--------|--------|-----|
| 001 | [State Scaffold — ShopStates, PlayerHands, Phase Machine](story-001-state-scaffold.md) | Logic | Ready | TR-CA-001, TR-CA-006 | ADR-015 |
| 002 | [Draft Initial — 9-Card Offering](story-002-draft-initial.md) | Integration | Ready | TR-CA-002 | ADR-015 |
| 003 | [Shop Draw Pipeline — Auto-Refresh, Dedup, 50/50 Split](story-003-draw-pipeline.md) | Logic | Ready | TR-CA-003, TR-CA-005, TR-CA-010 | ADR-015 |
| 004 | [Manual Refresh Cost Formula and Counter Reset](story-004-refresh-cost.md) | Logic | Ready | TR-CA-004 | ADR-015 |
| 005 | [Purchase Flow, Dead Slot, and CA18 Atomicity](story-005-purchase-flow.md) | Integration | Ready | TR-CA-008, TR-CA-009 | ADR-015 |
| 006 | [External Bypasses — PlayerHands Shared API](story-006-external-bypass.md) | Integration | Ready | TR-CA-007 | ADR-015 |

## Overview

Card Acquisition implements the server-side system that governs how players obtain
cards throughout a game session. It owns three operations: the one-time **Draft
Initial** (9-card display at game start; player selects within a 5g budget), the
**Personal Shop** (3 slots populated per DRAFT phase; purchased individually with
gold), and **Hand State** (the server-authoritative `PlayerHands` resource, capped
at 10 cards per player). A single system — `card_acquisition_tick_system` — is the
sole writer of `ShopStates` and the sole drainer of `MessageReceiver<C2SPurchaseCard>`
and `MessageReceiver<C2SRefreshShop>`. The RSM emits `ShopRefreshTriggered` on each
relevant phase entry; CA consumes it to execute auto-refresh draws. The CA18
transactional spend + distribute + refund-on-fail pair executes as sequential calls
within one system run — no cross-frame messaging between spend and refund.
`PlayerHands` is a shared resource also written by the Prism System and Objective
System during RESOLUTION (phases are mutually exclusive; no concurrent-write conflict
is possible).

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-015: Card Acquisition Shop State Machine Architecture | `ShopStates` + `PlayerHands` as server-only Resources; single-writer `card_acquisition_tick_system`; `ShopRefreshTriggered` Bevy Message for RSM→CA trigger; CA18 atomicity enforced by sequential call order within one system body | HIGH |

**Depends on (must be Accepted before implementation):**
ADR-005 (ServerRng), ADR-006 (CardPool API), ADR-008 (Lightyear channels),
ADR-009 (RSM phase resource pattern), ADR-010 (RSM event bus), ADR-013 (spend/refund
atomicity pattern).

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-CA-001 | Hand size capped at 10; server-authoritative `PlayerHands` resource; purchase rejected when `hand_len()==10` | ADR-015 ✅ |
| TR-CA-002 | DRAFT_INITIAL: 9-card display, 5g budget, 45s timer; class-filtered draws | ADR-015 ✅ |
| TR-CA-003 | Auto-refresh on DRAFT_SHOP entry via `ShopRefreshTriggered` Bevy Message (ADR-010 subscriber) | ADR-015 ✅ |
| TR-CA-004 | Manual refresh cost formula: `refresh_base_cost + min(count, refresh_cap)`; counter resets per DRAFT entry | ADR-015 ✅ |
| TR-CA-005 | Dedup against `displayed_this_draft`: 20-retry limit; short-circuit if K ≥ N (pool exhausted) | ADR-015 ✅ |
| TR-CA-006 | DRAFT_AUCTION: shop visible but locked; C2S messages silently discarded by phase gate | ADR-015 ✅ |
| TR-CA-007 | External bypasses (Prism Lane 3, Objective fake reward, Auction win) write `PlayerHands` directly; CA not in call chain | ADR-015 ✅ |
| TR-CA-008 | Transactional spend + distribute + refund-on-fail; sequential calls within one system body; no cross-frame messaging | ADR-015 ✅ |
| TR-CA-009 | Dead slot fallback: slot remains empty until manual refresh; pool returns `None` cleanly, never panics | ADR-015 ✅ |
| TR-CA-010 | 50/50 class/neutral slot roll per Rule 3 draw pipeline; class filter enforced by ADR-014 boundary | ADR-015 ✅ |

## Pre-Implementation Gates

Before any CA story can be marked Ready for implementation:

1. **Lightyear 0.26 C2S receiver type** — `MessageReceiver<C2SPurchaseCard>` exact API confirmed (same gate as ADR-013 item 1; resolved for Auction — confirm applies to CA as well).
2. **ADR-015 scheduling order verified** — `CardAcquisitionSet::Tick.after(RsmSet::Tick)` passes Bevy schedule graph dump.
3. **ADR-014 class boundary** — `draw_class_card(player_class)` API confirmed in Card Data & Pool implementation before TR-CA-002 and TR-CA-010 stories start.

## GDD Open Questions (pre-implementation gates only)

| OQ | Question | Blocks |
|----|----------|--------|
| OQ3 | Dead slot display — greyed art, empty slot, or "sold out" indicator? | Shop/Auction UI stories only; CA logic unaffected |
| OQ4 | CA18 fault-injection approach — mock Pool or explicit error-injection path? | CA18 integration test story |
| OQ5 | DRAFT_INITIAL card display order — fixed layout or sorted by rarity/cost? | Shop/Auction UI stories only |

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All 22 BLOCKING acceptance criteria (CA1–CA22) are verified against implementation
- All Logic stories have passing unit tests in `tests/unit/card_acquisition/`
- All Integration stories have passing integration tests in `tests/integration/card_acquisition/` using `World::new()` + message injection — no live Lightyear session required
- CA18 integration test confirms `refund_gold` is called before any return path following a failed `distribute()` — verified with injected `Err(DistributeError::Exhausted)`
- `ResMut<ShopStates>` confirmed to appear in exactly one system — code review gate on every CA PR
- `MessageReceiver<C2SPurchaseCard>` and `MessageReceiver<C2SRefreshShop>` each appear in exactly one system — code review gate
- `card_acquisition_tick_system` is scheduled after `rsm_tick_system` — verified by Bevy schedule graph dump

## Next Step

Run `/create-stories card-acquisition` to break this epic into implementable stories.
