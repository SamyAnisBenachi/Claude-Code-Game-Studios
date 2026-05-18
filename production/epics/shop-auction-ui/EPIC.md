# Epic: Shop / Auction UI

> **Layer**: Presentation
> **GDD**: design/gdd/shop-auction-ui.md
> **Architecture Module**: `client/src/ui/shop_auction/` - `ShopAuctionUiPlugin` (sub-plugin #5 inside `PresentationPlugin`)
> **Status**: Ready - core story set drafted for S5-21; Sprint 6 accessibility remediation stories added for A11Y-ST-12, A11Y-ST-18, and A11Y-ST-02; Sprint 14+ UI Clean-Pass layout candidates 015 / 016 / 017 / 018 authored as Draft (NOT activated)
> **Stories**: 16 story docs listed in this index - 8 Complete, 3 Ready, 1 Blocked by UX evidence/layout gate, 4 Draft (Sprint 14+ UI Clean-Pass layout candidates 015 / 016 / 017 / 018; NOT activated by their authoring; depend on the UI Clean-Pass Roadmap Tier 0 foundational stories landing first)

## Overview

Shop / Auction UI implements the client-side bevy_ui panels for every M2 economic decision: the one-time DRAFT_INITIAL offering, the DRAFT_SHOP slots and refresh flow, and the DRAFT_AUCTION bidding panel. It consumes server-authoritative card acquisition, auction, economy, phase, and snapshot messages, then renders UI state and sends only the allowed C2S intent messages for purchase, refresh, bid, and ready/retract-ready.

`ShopAuctionUiPlugin` is registered fifth in `PresentationPlugin`, after Card Animations, Board Rendering, Hand UI, and HUD. It reads `Res<CurrentClientPhase>` from the shared phase sink, never drains `MessageReceiver<S2CPhaseChanged>` directly, and uses bevy_ui `Node`/`Text`/`ImageNode` entities rather than world-space sprites.

Shared ADR-021 infrastructure is owned by [Presentation Layer Story 001](../presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md). Shop/Auction UI Story 001 must not implement `PresentationPlugin`, `PresentationSet`, or `phase_sink_system`; it depends on those surfaces before implementation.

Presentation Layer Story 001 is complete on `main` as of `d303155`. Shop/Auction UI Story 001 should still launch after Board Rendering Story 001 so the ADR-021 registration order can include `BoardRenderingPlugin` before `ShopAuctionUiPlugin`.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | bevy_ui panel ownership, `PresentationSet`, single phase sink, plugin order, animation handoff to Card Animations | HIGH |
| [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md) | Server-authoritative auction card, accepted/rejected/settled messages, timer reset, gold reservation lifecycle | HIGH |
| [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md) | Server-authoritative draft offering, shop slots, purchase/refresh flow, hand cap | HIGH |
| [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md) | `gold - reserved_gold` free-gold display and authoritative economy updates | HIGH |
| [ADR-011: Reconnect and Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md) | `S2CGameSnapshot` rebuild and late-message recovery patterns | HIGH |
| [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md) | Reliable channel, single-drain discipline, default reliable message choice | HIGH |
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | UI sends intent only; all state changes wait for S2C confirmation | LOW |

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-SAU-001 | `local_free_gold = gold - reserved_gold` computed from `S2CGoldBroadcast` | ADR-019 |
| TR-SAU-002 | Bid increment buttons render total commitment from current price plus preset offsets | ADR-013, ADR-021 |
| TR-SAU-003 | Settlement display via `S2CAuctionSettled` winner/price/card movement state | ADR-013, ADR-021 |
| TR-SAU-004 | Shop slots locked during DRAFT_AUCTION and visible as a read-only footer | ADR-015, ADR-021 |
| TR-SAU-005 | In-flight bid state and reversal on accepted/rejected server response | ADR-013, ADR-019, ADR-021 |
| TR-SAU-006 | Panel transitions, timings, and input gating across DRAFT_INITIAL, DRAFT_AUCTION, and DRAFT_SHOP | ADR-021 |

## Dependency Map

| Dependency | Existing Surface | Shop/Auction UI Use |
|------------|------------------|---------------------|
| Card Acquisition | `S2CDraftOffering`, `S2CShopSlots`, `S2CCardAcquired`, `C2SPurchaseCard`, `C2SRefreshShop` | Draft offering grid, shop slot state, purchase confirmation, refresh flow |
| Auction System | `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionBidRejected`, `S2CAuctionSettled`, `C2SPlaceBid` | Auction activation, bid buttons, leader state, settlement |
| Economy | `S2CGoldUpdate`, `S2CGoldBroadcast` with `reserved_gold` | Free-gold calculation, affordability gates, gold gate after opponent bid |
| RSM / Phase State | `S2CPhaseChanged`, `CurrentClientPhase` | Panel activation/dismissal and timer initialization |
| Game Session / Reconnect | `S2CGameSnapshot.auction_state` and shop/hand/economy snapshot fields | Rebuild after reconnect and ignore late stale messages |
| Game Config | draft, auction, shop, refresh, timeout, and transition timing knobs | Timers, refresh cost labels, preparing timeout |
| HUD | gold readout and top-layer overlay | Avoid duplicate ownership and keep HUD visible above panels |
| Hand UI | hand size, card-acquired fan movement, DRAFT hand state | Lockouts at hand cap and acquisition feedback |
| Board Rendering | visible board behind panels; shared screen layout | Avoid panel overlap with board, hand tray, and HUD |
| Card Animations | panel transitions, timer bar ease, bid feedback, settlement movement | UI requests animation; Card Animations owns tween mechanics |
| Presentation Layer | `PresentationPlugin`, `PresentationSet`, `CurrentClientPhase`, `phase_sink_system` | Provides shared scheduling and phase sink before Shop/Auction UI registers as sub-plugin #5 |

## Current Implementation Gaps

- `client/src/ui/shop_auction/` and `ShopAuctionUiPlugin` do not exist yet.
- No client-side panel node tree exists for draft offering, shop, auction, footer, toast, timer, or settlement overlays.
- No Shop/Auction UI S2C message drains are present in client code.
- `PresentationPlugin`/`phase_sink_system` infrastructure exists on `main`; Story 001 must reuse it, not redefine it locally.
- `design/ux/shop-auction-ui.md` exists. Story 009 remains the visual evidence/layout gate, while earlier stories cite the UX spec for root/lifecycle constraints as needed.
- The Shop/Auction UI GDD status has been reconciled to Approved; story-readiness should treat the GDD as approved unless later design changes reopen it.
- QA-COND-0005 remains Open. A11Y-ST-12 auction bid target/focus remediation is Complete in Story 011, and A11Y-ST-18 DRAFT_INITIAL clear objective remediation is Complete in Story 012. A11Y-ST-02 card cost, ATK, HP, and keyword readability is split into Story 013 for Shop/Auction UI-owned card surfaces before the final cross-surface Presentation evidence pass. These accessibility slices are independent from the broader blocked SAU-009 visual sign-off.

## Pre-Implementation Gates

| Gate | Blocks | Required Resolution |
|------|--------|---------------------|
| UX spec available: `design/ux/shop-auction-ui.md` | Visual evidence/layout work remains in Story 009 | Cite the UX spec in UI/layout stories; keep exact pixel evidence out of Story 001 |
| Presentation Layer scaffold complete | Story 001 no longer blocked by shared phase infrastructure | Reuse `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase`; do not redefine them locally |
| Board Rendering scaffold ordering | Story 001 launch sequencing | Complete Board Rendering Story 001 before Shop/Auction UI Story 001 so `ShopAuctionUiPlugin` can register fifth after `BoardRenderingPlugin` |
| Auction System server dispatch stability | Stories 004-007 | Auction card, accepted/rejected, settled, and gold broadcast ordering must be stable |
| Card Acquisition server dispatch stability | Stories 002-003 | Draft offering, shop slots, card acquired, purchase/refresh confirmations must be stable |
| GDD approval reconciled | No current blocker for Story 001 scaffold/formulas | Re-open only if a future design change alters Story 001 acceptance criteria |
| OQ9 leader idle window validation | Playtest/evidence follow-up | Escalate if "YOU ARE LEADING" state feels like dead time |

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`.
- All blocking acceptance criteria from `design/gdd/shop-auction-ui.md` are verified.
- UI state and message handling tests pass under `tests/unit/shop_auction_ui/` or `tests/integration/shop_auction_ui/`.
- Visual/layout/accessibility evidence is captured under `production/qa/evidence/`.
- `ShopAuctionUiPlugin` reads phase through `Res<CurrentClientPhase>` only.
- C2S messages are sent only for allowed intent: `C2SPurchaseCard`, `C2SRefreshShop`, `C2SPlaceBid`, and `C2SSignalReady`.
- No client-side optimistic purchase, refresh, gold, or ownership state is committed without S2C confirmation.

## Stories

| # | Story | Type | Status | TR-IDs | ADR |
|---|-------|------|--------|--------|-----|
| 001 | [Plugin Scaffold, Panel Tree, and Formulas](story-001-plugin-scaffold-panel-tree-and-formulas.md) | Logic | Complete | TR-SAU-001, TR-SAU-002, TR-SAU-006 | ADR-021, ADR-019 |
| 002 | [Draft Initial Grid Purchase Ready](story-002-draft-initial-grid-purchase-ready.md) | UI | Complete | TR-SAU-006 | ADR-015, ADR-021 |
| 003 | [Shop Panel Slots Refresh Purchase Ready](story-003-shop-panel-slots-refresh-purchase-ready.md) | UI | Complete | TR-SAU-004, TR-SAU-006 | ADR-015, ADR-021 |
| 004 | [Auction Panel Activation and Preparing State](story-004-auction-panel-activation-and-preparing-state.md) | Integration | Complete | TR-SAU-006 | ADR-013, ADR-021 |
| 005 | [Auction Bid Buttons, Affordability, and In-Flight](story-005-auction-bid-buttons-affordability-and-inflight.md) | UI | Complete | TR-SAU-001, TR-SAU-002, TR-SAU-005 | ADR-013, ADR-019, ADR-021 |
| 006 | [Auction Accepted/Rejected Feedback](story-006-auction-accepted-rejected-feedback.md) | Integration | Complete | TR-SAU-001, TR-SAU-005 | ADR-013, ADR-019 |
| 007 | [Auction Settlement and Shop Transition](story-007-auction-settlement-and-shop-transition.md) | Visual/Feel | Ready | TR-SAU-003, TR-SAU-006 | ADR-013, ADR-019, ADR-021 |
| 008 | [Reconnect Snapshot and Late Message Recovery](story-008-reconnect-snapshot-and-late-message-recovery.md) | Integration | Ready | TR-SAU-003, TR-SAU-006 | ADR-011, ADR-021 |
| 009 | [Visual Evidence, Layout, and Accessibility](story-009-visual-evidence-layout-and-accessibility.md) | UI | Blocked | TR-SAU-006 | ADR-021 |
| 011 | [Auction Bid Target Size and Focus Evidence](story-011-auction-bid-target-size-and-focus-evidence.md) | UI | Complete | TR-SAU-002, TR-SAU-005, TR-SAU-006 | ADR-013, ADR-019, ADR-021 |
| 012 | [Draft Initial Clear Objective Overlay](story-012-draft-initial-clear-objective-overlay.md) | UI | Complete | TR-SAU-006, A11Y-ST-18 | ADR-015, ADR-021 |
| 013 | [Card Text, Stat, and Keyword Accessibility](story-013-card-text-stat-keyword-accessibility.md) | UI | Ready | TR-CDP-010, TR-SAU-003, TR-SAU-004, TR-SAU-006, A11Y-ST-02 | ADR-002, ADR-013, ADR-015, ADR-019, ADR-021 |
| 015 | [Draft Initial Grid Centered Modal Layout](story-015-draft-grid-centered-modal.md) | UI | Draft | TR-SAU-006 | ADR-015, ADR-021 |
| 016 | [Auction Featured Card Visual Hierarchy](story-016-auction-featured-card.md) | UI | Draft | TR-SAU-002, TR-SAU-006 | ADR-013, ADR-021 |
| 017 | [Auction Free-Gold Counters Layout and Readability](story-017-auction-free-gold-counters.md) | UI | Draft | TR-SAU-001, TR-SAU-006 | ADR-013, ADR-019, ADR-021 |
| 018 | [Auction Featured Card Leading / Losing State Visual](story-018-auction-lead-loss-state.md) | UI | Draft | TR-SAU-005, TR-SAU-006 | ADR-013, ADR-021 |
| 019 | [Bid-Button Phase-Entry Race / Empty + Baked-`?` Cleanup](story-019-bid-button-phase-race.md) | UI / Integration | Draft -- Sprint 17 candidate (Should Have, `S17-UI-BID-BUTTON-PHASE-RACE-001`; PROMPT 1077 SOURCE-1077-10), NOT activated | TR-SAU-002, TR-SAU-005 | ADR-013, ADR-021 |

**Story counts**: 1 Logic, 3 Integration, 11 UI, 1 Visual/Feel.

## Sprint 14+ UI Clean-Pass Layout Candidates (NOT activated)

Stories 015 / 016 / 017 / 018 are Sprint 14+ candidates authored from the
[UI Clean-Pass Roadmap](../../../docs/ux/ui-clean-pass-roadmap.md)
and the
[PROMPT 802 Expert UI Layout Audit](../../../reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md).
They are **paperwork only**. Authoring them does **not**:

- activate Sprint 14
- pull these rows into any active sprint
- invoke `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`
- advance stage (Polish remains)
- retry the PROMPT 761 Polish->Release gate-check FAIL
- claim final-art / asset-production completion
  (`PAW-TD-002-a` / `PAW-TD-003-a` accept-risk preserved)
- claim Standard-tier accessibility completion (`QA-COND-0005`)
- claim playtest / fun-hypothesis validation (`QA-COND-0006`)
- close `S8-QA-001-W1` (two-client GAME_OVER closure)

Each of the four stories is **scoped to layout / composition /
typography / readability** for the friend-game product showcase. Each
depends on the UI Clean-Pass Roadmap Tier 0 foundational stories
(`S11-TD-UI-ZINDEX-LAYERS`, `S11-TD-UI-FONT-CONSTANTS`,
`S11-TD-UI-FLEX-STRIPS`, `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`,
`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) landing first. Each story carries
a no-claim banner preserving the accept-risk dispositions named above.

Story 018 additionally carries PROMPT 802 §9 producer-decision-4
resolution from PROMPT 967: static token-colored border-frame on the
Story 016 `AuctionFeaturedCardFrame` (`SEMANTIC_SUCCESS` leading,
`SEMANTIC_ERROR` losing, existing `ACCENT` neutral / pre-bid). No
new art, pulse, chevron, or badge is selected; `/story-readiness`
must re-run before `/dev-story`.

## Sprint 6 Candidate Order

Recommended Sprint 6 sequence:
1. Presentation Layer Story 001 - shared plugin, set, and phase sink foundation.
2. Story 001 - plugin scaffold and pure formulas.
3. Story 002 - DRAFT_INITIAL purchase and ready path.
4. Story 004 - auction activation/preparing state.
5. Story 005 - auction bid buttons and affordability.
6. Story 006 - accepted/rejected feedback and two-message gold gate.
7. Story 003 - DRAFT_SHOP slots, refresh, purchase, ready.
8. Story 007 - settlement and transition after auction server path is stable.
9. Story 008 - reconnect and late-message recovery.
10. Story 011 - A11Y-ST-12 bid target size, focus visibility, and browser/WASM evidence after SAU-005 and SAU-006.
11. Story 012 - A11Y-ST-18 DRAFT_INITIAL clear objective overlay, dismissal, retrieval, and browser/WASM evidence after SAU-002.
12. Story 013 - A11Y-ST-02 card cost, ATK, HP, and keyword readability for Shop/Auction UI-owned card surfaces after SAU-002, SAU-003, SAU-004, SAU-007, SAU-011, and SAU-012.
13. Story 009 - full visual evidence after all scoped panel states and accessibility remediation slices exist.

## Next Step

After Board Rendering Story 001 is complete, run `/story-readiness production/epics/shop-auction-ui/story-001-plugin-scaffold-panel-tree-and-formulas.md` before implementation. Use `liv-bevy-018` for every Bevy `.rs` file and `liv-bevy-lightyear` for every Lightyear/networking `.rs` file.
