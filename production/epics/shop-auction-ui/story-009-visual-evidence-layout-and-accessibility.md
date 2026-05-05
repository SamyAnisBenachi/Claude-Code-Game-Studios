# Story 009: Visual Evidence, Layout, and Accessibility

> **Epic**: Shop / Auction UI
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-006`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story captures the final UX/layout/accessibility evidence for DRAFT_INITIAL, DRAFT_AUCTION, and DRAFT_SHOP. The UX spec now exists; this story remains blocked until the Shop/Auction review blockers are resolved and panel states from Stories 002-007 are renderable enough to capture evidence.

## Blocker

Current blockers are review/evidence blockers, not missing-spec blockers:

- Shop/Auction UX repair must be merged before SAU-009 starts. The repair scope is: bid accessibility conflict, stale interaction pattern assumptions, tooltip storage, data requirements, toast timing, localization expansion, and vertical HUD/hand/panel layout contract.
- Stories 002-007 must render the relevant panel states before screenshots and manual accessibility evidence can be captured.

## Acceptance Criteria

- [ ] DRAFT_INITIAL header, tooltip, grid, timer, Ready state, and purchased-slot overlay match the UX spec.
- [ ] DRAFT_AUCTION panel, locked shop footer, timer, leader display, and bid button states match the UX spec.
- [ ] DRAFT_SHOP panel, refresh button, shop slots, timer, Ready state, and hand-full state match the UX spec.
- [ ] Panels do not overlap HUD, hand tray, or board-critical content at target desktop and mobile/test viewports.
- [ ] Text fits within button and panel containers for longest expected labels.
- [ ] Keyboard/focus/hover/click behavior is documented or tested for all interactive controls.
- [ ] Evidence screenshots and notes are stored under `production/qa/evidence/`.
- [ ] OQ9 "YOU ARE LEADING" idle window is explicitly observed in playtest/manual evidence.

## Implementation Notes

- This is the evidence and polish gate, not the first implementation story.
- Use browser/WASM screenshots once a dev server path exists.
- Verify panels are bevy_ui and board remains world-space behind them.
- If UX spec permits scoped desktop-only M2 evidence, document the scope in the evidence file.

## Out of Scope

- Adding new mechanics to solve OQ9. If playtest flags the leading state as dead time, create a follow-up design/change story.
- Server-side auction/shop logic.
- Board Rendering layout changes beyond coordination notes.

## QA Test Cases

- **Viewport fit**
  - Given: target desktop and narrow viewport sizes
  - When: each panel state is shown
  - Then: no text or controls overlap HUD, hand tray, or board-critical content.

- **Control state audit**
  - Given: each interactive state
  - When: focus/hover/click controls are exercised
  - Then: enabled, disabled, hidden, in-flight, and ready states are visually distinct.

- **Leader idle observation**
  - Given: local player is auction leader for at least 10 seconds with no opponent bid
  - When: playtest/manual observation is recorded
  - Then: evidence notes whether the state reads as tense or dead time.

## Test Evidence

**Required evidence**:
- UI: `production/qa/evidence/shop-auction-ui-layout-accessibility-evidence.md`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: Stories 002-007 implemented enough to render all panel states; `design/ux/shop-auction-ui.md`.
- Unlocks: Shop / Auction UI epic visual sign-off.
