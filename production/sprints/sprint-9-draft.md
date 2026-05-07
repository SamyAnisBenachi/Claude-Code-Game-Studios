# Sprint 9 Draft Plan -- Planning Only

> **Status**: Draft only. Sprint 9 is not active.
> **Source of truth**: `origin/main@c3635f02109526e50656542748396540d462ac1f`
> **Drafted**: 2026-05-07
> **Do not use this document as activation state.**

This document is a planning draft from the current Sprint 8 state. It does not
activate Sprint 9, does not close Sprint 8, and does not update
`production/sprint-status.yaml`.

## Current Conditions Carried Into The Draft

- Sprint 8 close-out is not complete.
- Latest-main CI may still be in progress for commits after the QA sign-off
  reference. This draft does not claim a new green latest-main CI result.
- Sprint 8 QA sign-off is **APPROVED WITH CONDITIONS** at
  `production/qa/qa-signoff-sprint-8-2026-05-07.md`.
- Sprint 8 smoke is **PASS WITH WARNINGS** at
  `production/qa/smoke-sprint-8-2026-05-07.md`.
- S8-QA-001-W1 remains carried: a full manually driven browser/native
  two-client GUI route to `GAME_OVER` was not captured. Manual/browser
  `GAME_OVER` is not claimed.
- Controlled internal friend-game evidence reaches `GAME_OVER` through real
  local Lightyear server/client routing. That evidence supports internal
  friend-game endpoint coverage only.
- Native client blank lobby screen repair is active separately as Prompt 367.
  Current `origin/main` does not contain a Prompt 367 closure artifact, so this
  draft treats the native launch/playability follow-up as conditional until
  activation-time status is checked.
- `QA-COND-0005` remains accepted risk for friend-game scope only. This is not
  verified Standard-tier accessibility completion.
- `QA-COND-0006` remains accepted-risk/deferred. This is not playtest evidence,
  fun-hypothesis validation, or a playtest report.

## Draft Sprint Goal

Turn the controlled internal friend-game `GAME_OVER` endpoint into a usable
player-facing result flow for the friend-game path, then close the carried
manual/browser two-client evidence gap without claiming public release
readiness, full playable-client manual QA, broad accessibility completion,
playtest validation, or full game completion.

## Activation Preconditions

Before this draft can become an active Sprint 9 plan:

- Sprint 8 close-out must finish or the producer must explicitly approve
  starting Sprint 9 with Sprint 8 close-out still pending.
- Latest `origin/main` and CI state must be rechecked at activation time.
- Prompt 367 native blank lobby repair status must be checked. If unresolved,
  the native launch/playability follow-up remains a Must Have gating item.
- Sprint 9 story files/readiness reviews must be created or confirmed before
  implementation starts.
- `production/sprint-status.yaml` must remain Sprint 8 until an explicit
  activation prompt updates it.

## Story Readiness Ownership Package

This draft now has story-file ownership for the missing Sprint 9 rows while
remaining planning-only. These docs do not activate Sprint 9 and do not update
`production/sprint-status.yaml` or `production/session-state/**`.

| Sprint Row | Story Ownership | Readiness Disposition |
|---|---|---|
| S9-RS-001 | `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md` | Standalone story exists and remains the prerequisite contract story for Result Screen MVP. |
| S9-RS-002 | `production/epics/presentation-layer/story-006-result-screen-mvp.md` | Standalone story exists and remains blocked until S9-RS-001 contract gaps are resolved or explicitly accepted as MVP fallbacks. |
| S9-RS-003 | `production/epics/game-session-system/story-010-result-acknowledgement-cleanup-handshake.md` | New standalone follow-up story owns Return to Lobby acknowledgement dispatch, local ended-session UI cleanup, idempotent server cleanup, timeout cleanup, and reconnect-after-cleanup fallback verification after S9-RS-001 and S9-RS-002. |
| S9-QA-001 | `production/epics/playable-client/story-007-manual-browser-game-over-evidence-closure.md` | New standalone evidence story owns the full manual browser/native two-client route through GAME_OVER, result screen, and acknowledgement. It closes S8-QA-001-W1 only with actual captured evidence. |
| S9-RS-004 | `production/epics/presentation-layer/story-006-result-screen-mvp.md` | No standalone story in this package. Initial focus, visible focus indicator, reduced-motion, photosensitivity, and viewport evidence ownership is explicitly inside Result Screen MVP. Split later only if implementation evidence shows a route-blocking polish issue. This does not close QA-COND-0005. |
| S9-QA-002 | `production/epics/playable-client/story-008-sprint-9-result-evidence-index-cleanup.md` | New standalone follow-up story owns the concise Sprint 9 result evidence index after S9-QA-001 evidence or blocker records exist. |

Sprint 8 conditions and non-claims remain carried: manual/browser GAME_OVER is
not claimed until S9-QA-001 captures it; QA-COND-0005 remains accepted risk for
friend-game scope only; QA-COND-0006 remains accepted-risk/deferred; no public
release readiness, broad accessibility completion, playtest validation, full
playable-client manual QA, full regression campaign, or full game completion is
claimed by this readiness package.

## Must Have

| ID | Task | Owner | Est. Days | Dependencies | Acceptance Criteria |
|---|---|---:|---:|---|---|
| S9-RS-001 | Result acknowledgement and result data contract | network/session gameplay programmer + UX owner | 1.00 | `design/ux/result-screen.md`; current `S2CGameOver` and `S2CGameSnapshot` contracts | Decide and document when `C2SAcknowledgeResult` is sent; define the minimal authoritative data source for result headline, reason, round, final resources, and objective reveal fallback; define reconnect-at-`GAME_OVER` behavior as either `S2CGameOver` resend or snapshot-carried result payload; rematch remains disabled/unsupported unless separately scoped. |
| S9-RS-002 | Result screen MVP | UI/client programmer | 2.00 | S9-RS-001; HUD GAME_OVER freeze; controlled Sprint 8 `GAME_OVER` evidence | A player-facing result overlay opens from `S2CGameOver` plus `S2CPhaseChanged(GameOver)`; displays victory/defeat/draw/no-result headline, cause, final round, available objective lane status, available final resources, and Return to Lobby; shows safe fallback copy when result/reveal data is missing; Rematch is disabled or hidden if protocol support remains undefined; HUD stays frozen underneath. |
| S9-RS-003 | Result acknowledgement implementation and cleanup handshake | session/network programmer + UI/client programmer | 1.00 | S9-RS-001; S9-RS-002 | Client sends `C2SAcknowledgeResult` at the decided timing; server cleanup/ended-session state handles acknowledgement idempotently; Return to Lobby clears local ended-session UI and returns to the lobby/menu flow without optimistic server-owned state changes. |
| S9-QA-001 | Manual/browser two-client `GAME_OVER` evidence closure | QA tester + orchestrator | 1.25 | S9-RS-002; S9-RS-003; Prompt 367 native status checked | Capture a full manually driven browser or native two-client route from lobby create/join through class confirm, draft/shop, auction, placement, resolution, `GAME_OVER`, and result screen/acknowledgement. Close S8-QA-001-W1 only if the route is actually captured. Preserve non-claims if the route remains blocked. |
| S9-NATIVE-001 | Native launch/playability repair follow-up from Prompt 367, if still open | client/platform programmer | 1.00 provisional | Prompt 367 status | If Prompt 367 remains open at activation, repair or verify the native blank lobby screen path enough for friend-game manual evidence. If Prompt 367 is already closed before Sprint 9 activation, replace this with a short verification-only checklist item. |

## Should Have

| ID | Task | Owner | Est. Days | Pull Condition | Acceptance Criteria |
|---|---|---:|---:|---|---|
| S9-REC-001 | Reconnect/result behavior hardening | network/session programmer + UI/client programmer | 1.25 | Pull if S9-RS-001 decides reconnect needs protocol work, or if manual evidence shows `GAME_OVER` reconnect/result fallback weakness | Reconnect during `GAME_OVER` rebuilds the result screen from authoritative data or shows `RESULT PENDING` with Return to Lobby usable; no client infers hidden objective identities beyond server-provided data. |
| S9-RS-004 | Result screen accessibility and viewport polish | UI/client programmer + QA tester | 0.75 | Pull after MVP is functional and before manual evidence if layout/focus issues block route capture | Keyboard Tab/Enter reaches actions in logical order; visible focus indicators exist; reduced motion removes result reveal animation; 1366x768, 1920x1080, and 150% UI scale do not overlap critical result text or controls. This does not close QA-COND-0005. |
| S9-CONTENT-001 | Friend-game support card/content/asset polish | content designer + UI/client programmer | 0.75 | Pull only if result/manual evidence shows repeated-card, missing-display, or placeholder readability issues that directly hurt the friend-game route | Small content or display fallback improvements support the friend-game route and result readability. No full card production, full balance pass, broad asset approval, or unrelated art pipeline work is claimed. |
| S9-QA-002 | Result evidence index cleanup | orchestrator | 0.25 | Pull after S9-QA-001 evidence exists | Create or update a concise Sprint 9 evidence index that records exact endpoint, manual/browser status, result-screen status, and all carried non-claims. |

## Out Of Scope

- Activating Sprint 9 from this draft.
- Updating `production/sprint-status.yaml` to Sprint 9.
- Updating `production/session-state/active.md` for activation state.
- Sprint 8 close-out, smoke, QA sign-off, gate-check, `/dev-story`, or
  `/story-done` as part of this draft.
- Public, external, commercial, store, deployment, release-candidate, or launch
  readiness.
- Full playable-client manual QA beyond the scoped two-client friend-game
  `GAME_OVER` evidence route.
- Full game completion.
- Broad Standard-tier accessibility completion or closure of `QA-COND-0005`.
- Playtest evidence, fun-hypothesis validation, or closure of `QA-COND-0006`.
- Rematch protocol implementation unless explicitly added after S9-RS-001.
- Full alive-opponent objective identity reveal unless the result data contract
  explicitly adds authoritative server payload for it.
- Broad card catalog, balance, class, keyword, prism, audio, VFX, or asset
  production work that does not directly support the friend-game result route.

## Carried Risk Register

| Risk | Probability | Impact | Draft Mitigation |
|---|---|---|---|
| Manual/browser `GAME_OVER` route remains uncaptured | Medium | High | Make S9-QA-001 a Must Have after result MVP and native status check; only close the warning with actual evidence. |
| Result screen MVP starts before acknowledgement/data contract is clear | Medium | High | Put S9-RS-001 before result screen implementation. |
| `GAME_OVER` reconnect lacks authoritative result payload | Medium | Medium | Pull S9-REC-001 if S9-RS-001 or manual evidence confirms the gap. |
| Native blank lobby repair from Prompt 367 remains open | Unknown | High | Treat S9-NATIVE-001 as a conditional Must Have at activation. |
| Result screen implies full game completion or public readiness | Medium | High | Preserve non-claims in plan, stories, evidence, smoke, and sign-off. |
| Accessibility debt is accidentally represented as closed | Medium | High | Keep QA-COND-0005 accepted-risk language explicit; S9-RS-004 can improve the result screen without closing broad Standard-tier accessibility. |
| Friend-game evidence is mistaken for playtest validation | Medium | High | Keep QA-COND-0006 accepted-risk/deferred language explicit; no playtest/fun-hypothesis claim. |
| Polish scope expands into broad content or asset production | Medium | Medium | Pull S9-CONTENT-001 only for direct friend-game/result-route support. |

## Draft Definition Of Done

- Result data/acknowledgement contract is decided and documented.
- Result screen MVP displays the `GAME_OVER` outcome from authoritative data
  and provides Return to Lobby.
- `C2SAcknowledgeResult` behavior is implemented according to the chosen
  contract and is idempotent.
- Manual/browser or native two-client evidence captures the friend-game route
  through `GAME_OVER` and result acknowledgement, or records the exact blocker
  without closing S8-QA-001-W1.
- Prompt 367 native blank lobby status is resolved or explicitly carried with
  a blocker note.
- Reconnect/result behavior is implemented or explicitly deferred with a safe
  fallback if not needed for the friend-game route.
- `QA-COND-0005` and `QA-COND-0006` remain carried conditions unless separate,
  actual closure evidence exists later.
- No public release readiness, full playable-client manual QA, full game
  completion, broad accessibility completion, playtest validation, or full
  asset/content production is claimed.

## Verification For This Draft

This draft should be verified with:

- `git diff --check`
- `git diff --cached --check` before commit
