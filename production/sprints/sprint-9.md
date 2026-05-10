# Sprint 9 -- 2026-05-07 to 2026-05-20

> **Status**: Closed With Conditions (2026-05-10, PROMPT 577).
> **Activated**: 2026-05-07 by Prompt 409.
> **Source of truth**: `origin/main@879fd1dc4bd426d0d3ea4a985d73975755042c7c`.
> **Status reconciliation**: Prompt 415 reconciled `S9-CONTENT-001` against
> `origin/main@6d428021558b94d7ef0185d7dbc69887bd8dd785`; neutral card
> display placeholders are integrated at `424bcfa0b60cea5dba0d1cb920ac4a3221b9ae4f`.

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
  story remained blocked at activation until the result contract completed.
  Prompt 414 post-activation readiness refresh supersedes that status.
- Sprint 9 missing story readiness package is integrated at
  `879fd1dc4bd426d0d3ea4a985d73975755042c7c`.
- No Sprint 9 smoke, QA sign-off, implementation, story-done, or gate-check is
  claimed by this activation.

## Post-Activation Updates

- S9-RS-001 result acknowledgement/result data contract is complete and
  integrated on `origin/main` (`b87e694`, with story-done corrections through
  `6d42802`).
- Prompt 414 readiness refresh marks S9-RS-002 Result Screen MVP Ready for
  implementation assignment. This readiness does not claim result-screen
  implementation, manual/browser GAME_OVER evidence, smoke, QA sign-off,
  gate-check, public release readiness, full game completion, broad
  accessibility completion, playtest validation, or full playable-client manual
  QA.

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

- S9-RS-001 is complete and integrated on `origin/main`. Any old out-of-plan
  worktree at `D:\_DEV\claude-code-game-studios-worktrees\S9-RS-001` is not the
  source of truth for Sprint 9 status.
- S9-CONTENT-001 neutral card display placeholder pack is integrated on main at
  `424bcfa0b60cea5dba0d1cb920ac4a3221b9ae4f`. It remains a supporting
  content/asset slice only; no asset approval, full card production, full
  balance completion, or Sprint 9 close-out is claimed.
- No active native operator controls implementation worker was found in the
  root source-of-truth check. S9-NATIVE-001 is ready, not in progress.
- No active Result Screen MVP implementation is claimed. S9-RS-002 is Ready
  after Prompt 414 readiness refresh and must keep rematch hidden/disabled,
  alive opponent objective identities as `Unknown`, and all carried non-claims
  intact.

## Must Have

| ID | Task | Owner | Est. Days | Status | Dependencies | Acceptance Criteria |
|---|---|---:|---:|---|---|---|
| S9-RS-001 | Result acknowledgement and result data contract | network/session gameplay programmer + UX owner | 1.00 | Complete | `design/ux/result-screen.md`; current `S2CGameOver` and `S2CGameSnapshot` contracts | Complete and integrated: `C2SAcknowledgeResult` server handling, retained `S2CGameOver` plus per-player final snapshot, all-ack cleanup, timeout cleanup, post-cleanup reconnect fallback, rematch disabled/unsupported unless separately scoped. |
| S9-NATIVE-001 | Native friend-game operator controls | client/platform programmer | 1.00 | Ready | Native lobby blank screen repair integrated; story docs ready | Native operator path is clean enough for room entry/join, class confirmation, draft/shop, auction, placement, result dependency handling, and manual evidence attempts without client-side optimistic authority or debug-only proof. |
| S9-RS-002 | Result screen MVP | UI/client programmer | 2.00 | Ready | S9-RS-001 complete; HUD `GAME_OVER` freeze; controlled Sprint 8 `GAME_OVER` evidence | Player-facing result overlay opens from `S2CGameOver` plus `S2CPhaseChanged(GameOver)`; displays victory/defeat/draw/no-result headline, cause, final round, available objective lane status, available final resources, and Return to Lobby; shows safe fallback copy when data is missing; Rematch is disabled or hidden; alive opponent objective identities remain `Unknown` unless server authority exists; HUD stays frozen underneath. |
| S9-RS-003 | Result acknowledgement implementation and cleanup handshake | session/network programmer + UI/client programmer | 1.00 | Blocked | S9-RS-002 complete and integrated; S9-RS-001 complete | Client sends `C2SAcknowledgeResult` at the decided timing; server cleanup/ended-session state handles acknowledgement idempotently; Return to Lobby clears local ended-session UI and returns to the lobby/menu flow without optimistic server-owned state changes. |
| S9-QA-001 | Manual/browser two-client `GAME_OVER` evidence closure | QA tester + orchestrator | 1.25 | In-progress (partial) | Prerequisites met; MANUAL-FG-001 (S2) blocks full two-client GUI route — requires human operator | Partial automated evidence at `e26e240` (2026-05-08): 16/16 regressions pass; server starts cleanly. Manual route not executed. S8-QA-001-W1 remains open. Closes S8-QA-001-W1 only with actual full browser/native two-client evidence through GAME_OVER, result screen, and acknowledgement. |

## Conditional Backlog And Supporting Work

| ID | Task | Owner | Est. Days | Status | Pull Condition | Acceptance Criteria |
|---|---|---:|---:|---|---|---|
| S9-QA-002 | Result evidence index cleanup | orchestrator | 0.25 | Ready (pull condition met) | Blocker record and `sprint-9-result-evidence-index.md` exist at `e26e240`; awaiting human-operator S9-QA-001 manual route or explicit close-out decision | Create or update a concise Sprint 9 evidence index recording endpoint, manual/browser/native status, result screen status, acknowledgement status, evidence links, and all carried non-claims. |
| SAU-008 | Reconnect Snapshot and Late Message Recovery | UI/client programmer | 1.25 | Conditional backlog | Pull only if reconnect, snapshot, or late-message instability affects the active/result flow | Snapshot rebuild restores the correct shop/auction panel; late accepted/rejected and stale purchase/refresh confirmations do not revive inactive panels; no duplicate Lightyear receiver drains are introduced. |
| ECO-004 | Kill and Objective Awards reward-loop polish | gameplay programmer | 1.00 | Conditional backlog | Pull only if Sprint 9 evidence shows a concrete reward-loop gameplay issue | Reward changes preserve current contracts, avoid duplicate awards, land before interest snapshot, and do not expand into broad economy tuning. |
| S9-CONTENT-001 | Neutral card display placeholder pack | content designer + UI/client programmer | 0.75 | Done | Integrated on main at `424bcfa`; supporting content only | Display/zoom placeholders for current neutral cards improve route readability without claiming full card production, full balance completion, broad asset approval, or unrelated art pipeline completion. No `/story-done` was forced because no standalone story file exists. |
| S9-AUDIO-001 | Audio bootstrap + timer urgency cue | client/audio programmer | 0.75 | Optional slice — not started | Pull only if sprint capacity exists after S9-RS-002 and S9-NATIVE-001 are underway; does not touch result flow or must-have stories | `AudioPlugin` active in client build; `assets/audio/ui/hand/sfx_timer_urgency_default.ogg` placeholder exists; existing `TimerUrgencyAudio` message wired to `AudioPlayer` one-shot spawn; manual evidence doc at `production/qa/evidence/audio-timer-urgency-*.md`. No final audio, final mix, release readiness, full asset approval, Sound Bible, or audio accessibility (QA-COND-0005) claimed. |

`be8b37d` (lobby manual playability patch) is a supporting fix integrated on main; it is not a standalone Sprint 9 story. It improves room-code field UX (click-to-focus, select-existing-text, `KeyboardInput.text` normalisation, Backspace/Escape/Enter handling) and fixes duplicate same-class confirm leaving the client pending (server now re-acks with `S2CClassLocked`). Tests cover room-code select/replace, lobby state compilation, and class-confirm re-ack. No smoke, QA sign-off, S9-QA-001 evidence closure, or Sprint 8 condition resolution is claimed by this fix.

S9-RS-004 result-screen accessibility and viewport polish remains initially
owned inside S9-RS-002. It can be split later only if implementation evidence
shows a route-blocking polish issue. It does not close `QA-COND-0005`.

S9-REC-001 reconnect/result behavior hardening remains conditional. Pull it only
if S9-RS-001 or manual evidence confirms a `GAME_OVER` reconnect/result fallback
weakness that cannot be contained by the MVP contract.

## Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Manual/browser `GAME_OVER` route remains uncaptured | Medium | High | S9-QA-001 prerequisites met; MANUAL-FG-001 (S2) is now the active blocker. Automated regressions 16/16 pass at `e26e240`. Close the warning only with actual human-operator two-client route evidence. S8-QA-001-W1 remains open. |
| Result Screen MVP expands beyond the accepted MVP contract | Medium | High | Keep S9-RS-002 within Prompt 414 readiness boundaries: consume S9-RS-001 retained result data, keep Rematch hidden/disabled, keep alive opponent objectives `Unknown`, and preserve all Sprint 8 carried non-claims. |
| Old S9-RS-001 worktree is mistaken for source of truth | Low | Medium | Treat current `origin/main` and integrated story-done docs as source of truth; do not re-open old worker state unless a separate repair prompt asks for it. |
| Native operator controls are mistaken for native manual route evidence | Medium | High | S9-NATIVE-001 only enables operation; S9-QA-001 owns route evidence and closure. |
| Result screen implies full game completion or public readiness | Medium | High | Preserve non-claims in plan, stories, evidence, smoke, and sign-off. |
| Accessibility debt is accidentally represented as closed | Medium | High | Keep `QA-COND-0005` accepted-risk language explicit; result screen polish does not close broad Standard-tier accessibility. |
| Friend-game evidence is mistaken for playtest validation | Medium | High | Keep `QA-COND-0006` accepted-risk/deferred; no playtest or fun-hypothesis claim. |
| Supporting asset/content work expands into broad production | Medium | Medium | Keep S9-CONTENT-001 to integrated display placeholder support only; do not treat it as asset approval, full card production, full balance completion, or release readiness. |

## QA Plan

No Sprint 9 QA plan was found during activation. This activation does not run
`/qa-plan`, smoke, QA sign-off, `/team-qa`, or `/gate-check`.

S9-QA-001 has partial automated evidence at `e26e240` (regressions 16/16 pass; server starts cleanly) but the manual two-client GUI route remains blocked by MANUAL-FG-001 (S2). S8-QA-001-W1 is open. S9-QA-002 pull condition is met but index closure awaits the manual route outcome. A Sprint 9 QA plan is still required before any future Sprint 9 QA sign-off or production-to-polish gate claim.

## Definition Of Done

- [x] S9-RS-001 result acknowledgement/result data contract is complete and
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
