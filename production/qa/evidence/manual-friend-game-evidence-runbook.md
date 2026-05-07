# Manual Friend-Game Evidence Runbook

Date prepared: 2026-05-07
Prepared from source of truth: `origin/main@d99d20dc1696afdb2010bf9e87ab3ce11e24826d`
Scope: future manual native/browser two-client friend-game evidence pass

This is a runbook plan only. It is not evidence, not smoke, not QA sign-off, not
a gate check, not Sprint 9 activation, not `/dev-story`, and not `/story-done`.
Do not use this document to close S8-QA-001-W1 until a future operator captures
the full manual route and writes a separate evidence artifact.

## Current Evidence Boundary

Sprint 8 has controlled internal friend-game `GAME_OVER` coverage through a
real local Lightyear server/client route. The covered automated route reaches:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(endpoint) -> RESOLUTION -> GAME_OVER`.

That controlled evidence is useful context, but it does not claim a fully
manual native or browser two-client route to `GAME_OVER`, a rendered result
screen, Return to Lobby, acknowledgement cleanup, full playable-client manual
QA, public release readiness, broad accessibility completion, playtest
validation, or full game completion.

## Known Current Blockers And Dependencies

| Dependency | Current source status | Evidence-pass impact |
|---|---|---|
| Native operator controls | `production/epics/playable-client/story-006-native-friend-game-operator-controls.md` is prepared for future implementation. Current docs say native manual progression is blocked or unreliable until controls exist. | Native route execution should wait until lobby, class, draft/shop, auction, placement, and submit controls are operator-complete. If not complete, record the exact first blocked step and stop. |
| Result acknowledgement/data contract | `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md` is prepared and Ready, but not implemented by this runbook. | Return to Lobby and `C2SAcknowledgeResult` evidence should only be claimed after the contract is implemented or an approved fallback exists. |
| Result screen MVP | `production/epics/presentation-layer/story-006-result-screen-mvp.md` is blocked on result-contract gaps. | If the route reaches `GAME_OVER` before result UI exists, capture `S2CGameOver`/`S2CPhaseChanged(GameOver)` and record result-screen/Return-to-Lobby as blocked, not failed. |
| Manual/browser `GAME_OVER` gap | S8-QA-001-W1 remains a carried warning. | Close only with a separate completed evidence artifact that captures the route below. |

## Evidence Package Target

For the future run, create a new dated evidence folder, for example:

`production/qa/evidence/captures/manual-friend-game-YYYY-MM-DD/`

Recommended artifact names:

- `metadata.md`
- `commands.md`
- `server.log`
- `client-a.log`
- `client-b.log`
- `route-checklist.md`
- `defects.md`
- `screenshots/`
- `video/`

The final evidence summary should live next to this runbook as a separate file,
for example:

`production/qa/evidence/manual-friend-game-YYYY-MM-DD.md`

## Required Metadata

Capture these before launching anything:

| Field | Required value |
|---|---|
| Commit | `git rev-parse HEAD` |
| Branch | `git status --short --branch` |
| Origin commit | `git rev-parse origin/main` |
| Commands | Exact server, client A, client B, and browser commands |
| Platform | OS, shell, GPU if relevant, browser name/version if browser route |
| Build target | Native, browser/WASM, or both |
| Server port | Default `5000` unless explicitly changed |
| Client URL | Default `ws://localhost:5000` unless explicitly changed |
| Operator names | Host/client A and joiner/client B |
| Evidence folder | Relative path under `production/qa/evidence/captures/` |

## Launch Commands

Use one real local server and two real primary clients from the same commit.
Prefer port `5000` for browser because the current browser client defaults to
`ws://localhost:5000`.

### Native Route

Server:

```powershell
$env:SERVER_PORT="5000"
cargo run -p server
```

Client A, host:

```powershell
$env:SERVER_URL="ws://localhost:5000"
cargo run -p client --bin client
```

Client B, joiner:

```powershell
$env:SERVER_URL="ws://localhost:5000"
cargo run -p client --bin client
```

### Browser Route

Server:

```powershell
$env:SERVER_PORT="5000"
cargo run -p server
```

WASM client:

```powershell
trunk serve client/index.html --release --port 8080 --address 127.0.0.1 --no-autoreload true --no-error-reporting true
```

Open two separate browser windows or isolated profiles at:

`http://127.0.0.1:8080`

If the browser client still only supports the default server URL, keep the
server on port `5000`. If a future browser server-URL override exists, record
the exact override and where it is configured.

## Manual Route Script

Use client A as the host and client B as the joiner. Capture screenshots or
video at every phase boundary and whenever a control is activated.

| Step | Client A action | Client B action | Required observation | Artifact |
|---|---|---|---|---|
| 1. Connect | Launch client and wait for lobby/menu | Launch client and wait for lobby/menu | Both clients connect to the same local Lightyear server and send hello/heartbeat traffic where visible in logs. | Logs and screenshot |
| 2. Create room | Create Room | Wait | Server returns room code to client A; no local-only room state is claimed before S2C confirmation. | Screenshot and server/client logs |
| 3. Join room | Share room code | Enter room code, choose slot, Join Room | Client B receives join ack or rejection. Both clients show shared room membership only from server state. | Screenshot and logs |
| 4. Select class | Select class | Select class | Each client shows local pending selection only until authoritative class state arrives. | Screenshot |
| 5. Confirm class | Confirm class | Confirm class | Both clients observe class lock/reveal and transition into `DRAFT_INITIAL`. | Logs, screenshot |
| 6. DRAFT_INITIAL | Buy offered card if available, Ready | Buy offered card if available, Ready | Purchases, hand/economy updates, ready state, and phase transition come from S2C/snapshot state. | Logs, screenshot |
| 7. DRAFT_SHOP | Buy/refresh if available, Ready or Retract/Ready | Buy/refresh if available, Ready or Retract/Ready | Shop slots, economy, readiness, and phase transition remain server-authored. | Logs, screenshot |
| 8. DRAFT_AUCTION | Bid or pass according to available controls | Bid or pass according to available controls | `DRAFT_AUCTION`, auction card, accepted/rejected bids, settlement, and acquired card are observed without stale UI after settlement. | Logs, screenshot/video |
| 9. Post-auction shop | Ready when shop returns | Ready when shop returns | Post-auction `DRAFT_SHOP` is visible and old auction feedback does not persist. | Screenshot |
| 10. PLACEMENT | Stage zero or more cards, correct invalid staging if needed, Submit | Stage zero or more cards, correct invalid staging if needed, Submit | Submit sends real `C2SSubmitPlacement`; committed board state appears only after authoritative reveal/snapshot. | Logs, screenshot/video |
| 11. RESOLUTION | Observe replay | Observe replay | `S2CPlacementReveal` and `S2CResolutionEvent` are observed before the following phase change; `UnitPlaced` is visible when applicable. | Video or phase screenshots |
| 12. Repeat loop | Continue draft/shop, auction, placement, and resolution as needed | Continue same | Route continues until `GAME_OVER` or an explicit blocker occurs. | Running checklist |
| 13. GAME_OVER | Stop inputs except result UI actions | Stop inputs except result UI actions | Both clients observe `S2CGameOver` and `S2CPhaseChanged(GameOver)` if reached. HUD/board should freeze according to implemented behavior. | Logs, screenshot/video |
| 14. Result screen | Inspect result overlay if implemented | Inspect result overlay if implemented | Result headline, cause, round/resources/objectives, and fallbacks use authoritative data only. If missing, record as dependency blocked. | Screenshot |
| 15. Return to Lobby / ack | Activate Return to Lobby if implemented | Activate Return to Lobby if implemented | `C2SAcknowledgeResult` is sent only according to the implemented contract; local UI returns to lobby/menu without optimistic server-owned state claims. | Logs, screenshot |

## Pass/Block Criteria

The future evidence pass may claim manual/browser or native `GAME_OVER` only if:

- both clients are manually operated through the same real local server,
- the room create/join and class-confirm flow are captured,
- draft/shop, auction, placement, and resolution are captured through real
  controls,
- both clients reach and capture `GAME_OVER`, and
- server/client artifacts identify the exact commit and commands.

Return to Lobby and acknowledgement may be claimed only if the result
acknowledgement/data contract and result screen behavior are implemented or an
approved fallback is explicitly documented.

If a dependency is missing, mark the run `BLOCKED` or `PARTIAL`, keep
S8-QA-001-W1 open, and record the first blocked step.

## Route Checklist Template

| Item | Status | Evidence path | Notes |
|---|---|---|---|
| Server launched from exact commit | TODO |  |  |
| Client A launched from same commit | TODO |  |  |
| Client B launched from same commit | TODO |  |  |
| Browser route launched, if used | TODO |  |  |
| Room created | TODO |  |  |
| Room joined | TODO |  |  |
| Class selected and confirmed | TODO |  |  |
| `DRAFT_INITIAL` purchase/ready | TODO |  |  |
| `DRAFT_SHOP` purchase/refresh/ready | TODO |  |  |
| `DRAFT_AUCTION` bid/settlement | TODO |  |  |
| Post-auction `DRAFT_SHOP` | TODO |  |  |
| `PLACEMENT` staged/submit path | TODO |  |  |
| `RESOLUTION` replay/events | TODO |  |  |
| Repeated loop toward endpoint | TODO |  |  |
| `GAME_OVER` observed by client A | TODO |  |  |
| `GAME_OVER` observed by client B | TODO |  |  |
| Result screen observed, if implemented | TODO |  |  |
| Return to Lobby / ack observed, if implemented | TODO |  |  |
| Defects table completed | TODO |  |  |
| Non-claims preserved | TODO |  |  |

Use `PASS`, `WARN`, `BLOCKED`, `FAIL`, or `N/A` only after evidence is captured.

## Defects Table Template

| ID | Severity | Owner/System | Step | Status | Impact | Workaround | Evidence |
|---|---|---|---|---|---|---|---|
| MFG-YYYYMMDD-001 |  |  |  |  |  |  |  |

Severity guidance:

- `S1`: route cannot launch or connect.
- `S2`: route cannot reach the next required phase.
- `S3`: route reaches endpoint with visible defect or missing artifact.
- `S4`: polish, readability, or evidence-quality issue that does not block the
  route.

## Required Non-Claims

Every future evidence summary using this runbook must explicitly preserve these
non-claims unless separate evidence exists:

- No public release readiness.
- No store, deployment, release-candidate, or launch readiness.
- No broad Standard-tier accessibility completion.
- No closure of `QA-COND-0005`.
- No playtest validation.
- No fun-hypothesis validation.
- No closure of `QA-COND-0006`.
- No full playable-client manual QA beyond this scoped route.
- No full regression campaign.
- No asset production approval.
- No full game completion.

## Source Documents

- `production/qa/evidence/sprint-8-friend-game-evidence-index.md`
- `production/qa/evidence/sprint-8-friend-game-loop-evidence.md`
- `production/qa/smoke-sprint-8-2026-05-07.md`
- `production/qa/qa-signoff-sprint-8-2026-05-07.md`
- `production/sprints/sprint-8.md`
- `production/sprints/sprint-9-draft.md`
- `production/epics/playable-client/story-006-native-friend-game-operator-controls.md`
- `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md`
- `production/epics/presentation-layer/story-006-result-screen-mvp.md`
