# Sprint 9 -- 2026-05-07 to 2026-05-20

> **Status**: Active.
> **Activated**: 2026-05-07 by Prompt 409.
> **Source of truth**: `origin/main@879fd1dc4bd426d0d3ea4a985d73975755042c7c`.

Sprint 9 turns the controlled internal friend-game `GAME_OVER` endpoint into a
usable player-facing result flow for the friend-game path, then closes or
accurately preserves the carried manual/browser two-client `GAME_OVER` evidence
gap.

This activation is docs/status only. It does not run `/dev-story`,
`/story-done`, smoke, QA sign-off, `/team-qa`, `/gate-check`, implementation,
CI, or release readiness.

## Activation Basis

- Sprint 8 is closed with conditions.
- Native lobby blank screen repair is integrated in the tested code baseline via
  commit `1bfbf5b`.
- Native operator controls story docs exist at
  `production/epics/playable-client/story-006-native-friend-game-operator-controls.md`.
- Result acknowledgement/result data contract story docs exist at
  `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md`.
- Result Screen MVP story docs exist at
  `production/epics/presentation-layer/story-006-result-screen-mvp.md`, but the
  story remains blocked until the result contract completes.
- Sprint 9 missing story readiness package is integrated at
  `879fd1dc4bd426d0d3ea4a985d73975755042c7c`.
- No Sprint 9 smoke, QA sign-off, implementation, story-done, or gate-check is
  claimed by this activation.

## Carried Sprint 8 Conditions

- S8-QA-001-W1 remains carried: full manually driven browser/native two-client
  GUI evidence through `GAME_OVER` was not captured.
- `QA-COND-0005` remains accepted risk for friend-game scope only. It is not
  verified Standard-tier accessibility completion.
- `QA-COND-0006` remains accepted-risk/deferred. It is not playtest evidence,
  fun-hypothesis validation, or a playtest report.
- Public release readiness is not claimed.
- Release-candidate readiness is not claimed.
- Full game completion is not claimed.
- Full playable-client manual QA is not claimed.
- Broad accessibility completion is not claimed.
- Playtest or fun-hypothesis validation is not claimed.

## Sprint Goal

Deliver the minimum result acknowledgement contract, player-facing result
screen, Return to Lobby acknowledgement behavior, and manual/browser or native
two-client evidence needed to close or honestly carry the Sprint 8 `GAME_OVER`
manual evidence warning.

## Capacity

- Total workdays: 10
- Buffer: 2 days for integration and evidence friction
- Available: 8 effective planned days
- Must Have scope: 6.25 estimated days, plus conditional native operator
  controls as needed for manual evidence

## Current Coordination Notes

- S9-RS-001 has an active out-of-plan worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\S9-RS-001` on branch
  `work/s9-rs-001-result-ack-contract`. It has uncommitted local changes in
  server session/network files and is behind current `origin/main`. This is
  tracked as in progress only; no completion or integration is claimed.
- S9-CONTENT-001 has an out-of-plan pull-forward branch/worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\neutral-card-display-placeholder-pack`
  on branch `work/neutral-card-display-placeholder-pack`, with local commit
  `33d60d4` ahead of its base and not integrated to `origin/main`. This is
  supporting content/asset work in progress only; no asset approval, full card
  production, or Sprint 9 completion is claimed.
- No active native operator controls implementation worker was found in the
  root source-of-truth check. S9-NATIVE-001 is ready, not in progress.
- No active Result Screen MVP implementation is claimed. S9-RS-002 remains
  blocked until S9-RS-001 completes and is integrated.

## Must Have

| ID | Task | Owner | Est. Days | Status | Dependencies | Acceptance Criteria |
|---|---|---:|---:|---|---|---|
| S9-RS-001 | Result acknowledgement and result data contract | network/session gameplay programmer + UX owner | 1.00 | In progress, not integrated | `design/ux/result-screen.md`; current `S2CGameOver` and `S2CGameSnapshot` contracts | Decide and implement/document when `C2SAcknowledgeResult` is sent; define the minimal authoritative data source for result headline, reason, round, final resources, and objective reveal fallback; define reconnect-at-`GAME_OVER` behavior as either `S2CGameOver` resend or snapshot-carried result payload; rematch remains disabled/unsupported unless separately scoped. |
| S9-NATIVE-001 | Native friend-game operator controls | client/platform programmer | 1.00 | Ready | Native lobby blank screen repair integrated; story docs ready | Native operator path is clean enough for room entry/join, class confirmation, draft/shop, auction, placement, result dependency handling, and manual evidence attempts without client-side optimistic authority or debug-only proof. |
| S9-RS-002 | Result screen MVP | UI/client programmer | 2.00 | Blocked | S9-RS-001; HUD `GAME_OVER` freeze; controlled Sprint 8 `GAME_OVER` evidence | Player-facing result overlay opens from `S2CGameOver` plus `S2CPhaseChanged(GameOver)`; displays victory/defeat/draw/no-result headline, cause, final round, available objective lane status, available final resources, and Return to Lobby; shows safe fallback copy when data is missing; Rematch is disabled or hidden if protocol support remains undefined; HUD stays frozen underneath. |
| S9-RS-003 | Result acknowledgement implementation and cleanup handshake | session/network programmer + UI/client programmer | 1.00 | Blocked | S9-RS-001; S9-RS-002 | Client sends `C2SAcknowledgeResult` at the decided timing; server cleanup/ended-session state handles acknowledgement idempotently; Return to Lobby clears local ended-session UI and returns to the lobby/menu flow without optimistic server-owned state changes. |
| S9-QA-001 | Manual/browser two-client `GAME_OVER` evidence closure | QA tester + orchestrator | 1.25 | Blocked | S9-NATIVE-001 and/or browser route usable; S9-RS-002; S9-RS-003 | Capture a full manually driven browser or native two-client route from lobby create/join through class confirm, draft/shop, auction, placement, resolution, `GAME_OVER`, result screen, and acknowledgement. Close S8-QA-001-W1 only if the route is actually captured. Preserve non-claims if blocked. |

## Conditional Backlog And Supporting Work

| ID | Task | Owner | Est. Days | Status | Pull Condition | Acceptance Criteria |
|---|---|---:|---:|---|---|---|
| S9-QA-002 | Result evidence index cleanup | orchestrator | 0.25 | Blocked/backlog | Pull after S9-QA-001 evidence or blocker record exists | Create or update a concise Sprint 9 evidence index recording endpoint, manual/browser/native status, result screen status, acknowledgement status, evidence links, and all carried non-claims. |
| SAU-008 | Reconnect Snapshot and Late Message Recovery | UI/client programmer | 1.25 | Conditional backlog | Pull only if reconnect, snapshot, or late-message instability affects the active/result flow | Snapshot rebuild restores the correct shop/auction panel; late accepted/rejected and stale purchase/refresh confirmations do not revive inactive panels; no duplicate Lightyear receiver drains are introduced. |
| ECO-004 | Kill and Objective Awards reward-loop polish | gameplay programmer | 1.00 | Conditional backlog | Pull only if Sprint 9 evidence shows a concrete reward-loop gameplay issue | Reward changes preserve current contracts, avoid duplicate awards, land before interest snapshot, and do not expand into broad economy tuning. |
| S9-CONTENT-001 | Neutral card display placeholder pack | content designer + UI/client programmer | 0.75 | In progress, not integrated | Out-of-plan pull-forward only; integrate only if approved and non-conflicting | Display/zoom placeholders for current neutral cards improve route readability without claiming full card production, full balance completion, broad asset approval, or unrelated art pipeline completion. |

S9-RS-004 result-screen accessibility and viewport polish remains initially
owned inside S9-RS-002. It can be split later only if implementation evidence
shows a route-blocking polish issue. It does not close `QA-COND-0005`.

S9-REC-001 reconnect/result behavior hardening remains conditional. Pull it only
if S9-RS-001 or manual evidence confirms a `GAME_OVER` reconnect/result fallback
weakness that cannot be contained by the MVP contract.

## Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Manual/browser `GAME_OVER` route remains uncaptured | Medium | High | Keep S9-QA-001 blocked until result flow and operator/browser controls are usable; close the warning only with actual evidence. |
| Result Screen MVP starts before acknowledgement/data contract is integrated | Medium | High | Keep S9-RS-002 blocked until S9-RS-001 completes and is integrated. |
| Active S9-RS-001 worktree drifts from current `origin/main` | Medium | Medium | Rebase/repair in the worker worktree before integration; do not mark complete from dirty or unintegrated work. |
| Native operator controls are mistaken for native manual route evidence | Medium | High | S9-NATIVE-001 only enables operation; S9-QA-001 owns route evidence and closure. |
| Result screen implies full game completion or public readiness | Medium | High | Preserve non-claims in plan, stories, evidence, smoke, and sign-off. |
| Accessibility debt is accidentally represented as closed | Medium | High | Keep `QA-COND-0005` accepted-risk language explicit; result screen polish does not close broad Standard-tier accessibility. |
| Friend-game evidence is mistaken for playtest validation | Medium | High | Keep `QA-COND-0006` accepted-risk/deferred; no playtest or fun-hypothesis claim. |
| Supporting asset/content work expands into broad production | Medium | Medium | Keep S9-CONTENT-001 to approved display placeholder support only and require integration before claiming completion. |

## QA Plan

No Sprint 9 QA plan was found during activation. This activation does not run
`/qa-plan`, smoke, QA sign-off, `/team-qa`, or `/gate-check`.

S9-QA-001 and S9-QA-002 remain blocked until the result flow and usable
operator/browser route exist. A Sprint 9 QA plan is still required before any
future Sprint 9 QA sign-off or production-to-polish gate claim.

## Definition Of Done

- [ ] S9-RS-001 result acknowledgement/result data contract is complete and
      integrated.
- [ ] S9-NATIVE-001 is complete or explicitly waived because the browser route
      is sufficient for S9-QA-001 evidence.
- [ ] S9-RS-002 Result Screen MVP is complete and integrated.
- [ ] S9-RS-003 acknowledgement cleanup handshake is complete and integrated.
- [ ] S9-QA-001 captures the full manual/browser or native two-client route
      through result acknowledgement, or records the exact blocker without
      closing S8-QA-001-W1.
- [ ] S9-QA-002 indexes evidence or blocker disposition after S9-QA-001.
- [ ] `QA-COND-0005` and `QA-COND-0006` remain carried unless separate actual
      closure evidence exists later.
- [ ] No public release readiness, full playable-client manual QA, full game
      completion, broad accessibility completion, playtest validation, or full
      asset/content production is claimed.

## Verification For Activation

- `git diff --check`
- `git diff --cached --check` before commit
