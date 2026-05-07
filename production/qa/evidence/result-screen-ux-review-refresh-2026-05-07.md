# Result Screen UX Review Refresh

> **Date**: 2026-05-07
> **Reviewer**: Codex using `ux-review` checklist
> **Document**: `design/ux/result-screen.md`
> **Source of truth**: `origin/main@d99d20dc1696afdb2010bf9e87ab3ce11e24826d`
> **Platform target**: WASM browser primary; native desktop dev/debug target
> **Accessibility tier**: Standard
> **Verdict**: NEEDS REVISION / BLOCKED BY CONTRACT AND IMPLEMENTATION DEPENDENCIES

This is a docs-only UX review refresh. It does not implement the result screen,
result acknowledgement, Return to Lobby, rematch, browser/manual evidence,
Sprint 9 activation, smoke, QA sign-off, gate-check, `/dev-story`, or
`/story-done`.

## Scope Checked

Primary documents reviewed:

- `design/ux/result-screen.md`
- `design/accessibility-requirements.md`
- `design/ux/interaction-patterns.md`
- `.claude/docs/technical-preferences.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
- `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md`
- `production/epics/hud/story-007-game-over-freeze.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/sprints/sprint-9-draft.md`

## Completeness

| UX section | Status | Notes |
|---|---|---|
| Header/status/platform/accessibility | Present | Header lists mouse + keyboard, WASM browser, Standard tier |
| Purpose and player need | Present | Covers outcome, cause, objective reveal, next action |
| Player context on arrival | Present | Covers win/loss/draw/disconnect/timeout and frozen HUD |
| Navigation position | Present | Defines terminal overlay and return to main-menu/lobby flow |
| Entry and exit points | Present with dependencies | Return to Lobby and Rematch depend on unresolved/unsupported protocol choices |
| Layout specification | Present | Result headline, objective summary, final summary, actions |
| States and variants | Present | Victory, defeat, draw, timeout, missing payload, partial reveal, rematch states |
| Interaction map | Present | Mouse, keyboard Tab/Enter, Escape focus behavior |
| Events fired | Present with open contract | `C2SAcknowledgeResult` timing is resolved by Story 009 but not implemented |
| Transitions and animations | Present | Reduced-motion alternatives are specified |
| Data requirements | Present with blockers | Full alive-opponent objective reveal and reconnect result payload are explicit gaps |
| Accessibility | Present | Keyboard, focus, contrast, color-independent labels, reduced motion |
| Localization | Present | Expansion and wrapping risks identified |
| Acceptance criteria | Present | Covers copy states, fallback, objective summary, HUD frozen, focus, reduced motion |

Completeness result: the UX spec is structurally complete, but it is not
implementation-ready until the result acknowledgement/data contract and result
screen MVP dependencies are implemented or explicitly accepted as fallback
scope.

## Required Checks From Prompt

| Check | Review result | Evidence/notes |
|---|---|---|
| Victory copy | Covered in UX spec and Story 006 AC | `VICTORY` derives from server-authored loser/reason; implementation evidence still missing |
| Defeat copy | Covered in UX spec and Story 006 AC | `DEFEAT` derives from server-authored loser/reason; implementation evidence still missing |
| Draw copy | Covered in UX spec and Story 006 AC | `DRAW` must not present a winner; Sprint 8 controlled endpoint observed draw payload |
| No-result/fallback | Covered | `NO RESULT` for `ResolutionTimeout`; missing payload path uses pending/fallback behavior |
| Return to Lobby | Covered but dependency-gated | Requires accepted `C2SAcknowledgeResult` behavior from Story 009 before QA can pass it |
| Rematch hidden/disabled | Covered | Spec/story require Rematch hidden or disabled while no protocol exists |
| Focus | Covered | Initial focus, Tab order, Enter activation, Escape-to-Return behavior specified |
| Reduced motion | Covered | Removes iris wipe, bloom flash, scale pulse, row sequencing, and repeated flashes |
| HUD frozen behind result | Covered and supported by HUD Story 007 | HUD freeze is implemented/evidenced separately; rendered result overlay still missing |
| Objective summary | Covered with data limitations | Own identities and destroyed opponent `was_fake` are authoritative; alive opponent lanes remain `Unknown` without new server reveal payload |

## Blocking Issues

1. **Result acknowledgement/data contract is not implemented**
   - Where: `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md`
   - Impact: Return to Lobby/ack evidence cannot pass until the server-owned
     acknowledgement handler, retained result data, and GAME_OVER reconnect
     behavior are implemented or explicitly replaced with an accepted fallback.
   - Required disposition: Implement Story 009 or record a producer-approved MVP
     fallback before claiming Return to Lobby/ack.

2. **Result screen is not implemented or manually evidenced**
   - Where: `production/epics/presentation-layer/story-006-result-screen-mvp.md`
   - Impact: Copy states, focus, reduced motion, objective summary, rematch
     hidden/disabled, and viewport layout are specified but have no rendered
     native/browser evidence yet.
   - Required disposition: Implement and capture the result-screen MVP before
     marking UX evidence checks pass.

3. **Full alive-opponent objective reveal has no current authoritative payload**
   - Where: `design/ux/result-screen.md` Data Requirements and Open Questions;
     Story 009 accepted decision 6
   - Impact: The MVP must display `Unknown` for alive opponent lanes unless a
     separate server-owned post-game reveal payload is added.
   - Required disposition: Preserve `Unknown` fallback in QA and evidence.
     Client-side inference is forbidden.

4. **Manual/native/browser route remains uncaptured**
   - Where: `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
   - Impact: Sprint 8 controlled real-Lightyear `GAME_OVER` evidence supports
     endpoint coverage, but not rendered/manual result-screen UX behavior.
   - Required disposition: Use the manual friend-game runbook for a future pass
     after controls and result contract work land.

## Advisory Issues

| Issue | Risk | Recommended follow-up |
|---|---|---|
| Interaction pattern library still lists Score / Result Panel as a gap | Medium | Add Result Overlay and Objective Reveal Summary patterns when the result-screen spec is accepted for implementation |
| Result copy needs concrete localized strings beyond labels | Medium | During implementation evidence, capture exact cause lines for objective loss, disconnect, draw, timeout, and missing payload |
| Focus behavior depends on Bevy/browser accessibility capability | Medium | Evidence should show keyboard focus visually even if semantic screen-reader focus is limited |
| Reduced-motion setting plumbing may depend on settings implementation state | Medium | Evidence must record whether the real setting drives result-screen motion or whether reduced-motion remains blocked |

## Acceptance Criteria Refresh

Future result-screen evidence should include:

- Victory, defeat, draw, no-result/timeout, missing result fallback, disconnect,
  and partial objective-data fallback copy.
- Return to Lobby click and keyboard activation.
- `C2SAcknowledgeResult` timing and server handling, if the accepted contract is
  implemented.
- Rematch hidden or disabled with no focus target and no rematch C2S.
- Initial focus, Tab order, Enter activation, and Escape focus behavior.
- Reduced-motion capture.
- HUD frozen behind the overlay: phase `GAME OVER`, final round/resources stable,
  no real/fake HUD dot mutation.
- Objective summary using authoritative data only, including `Unknown` for alive
  opponent identity without post-game reveal payload.
- Viewports `1366x768`, `1920x1080`, and 150 percent UI scale with no critical
  overlap.

## Non-Claims

- No result screen implementation.
- No result acknowledgement implementation.
- No Return to Lobby/ack evidence.
- No rematch protocol.
- No manual/native/browser `GAME_OVER` route evidence.
- No Sprint 9 activation.
- No smoke, QA sign-off, gate-check, `/dev-story`, or `/story-done`.
- No public release readiness.
- No broad Standard-tier accessibility completion or `QA-COND-0005` closure.
- No playtest validation, fun-hypothesis validation, or `QA-COND-0006` closure.
- No full playable-client manual QA.
- No full game completion.
