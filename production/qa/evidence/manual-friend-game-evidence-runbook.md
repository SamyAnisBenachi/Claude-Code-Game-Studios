# Manual Friend-Game Evidence Runbook

> **Status**: Prepared for future manual/native/browser execution
> **Prepared**: 2026-05-07
> **Source of truth**: `origin/main@d99d20dc1696afdb2010bf9e87ab3ce11e24826d`
> **Scope**: Internal friend-game evidence planning only

This runbook prepares the future manual/native/browser two-client evidence pass
for the friend-game route through `GAME_OVER`, result display, and Return to
Lobby acknowledgement if those controls are available at execution time.

It does not claim that the route has been manually captured. Sprint 8 controlled
real-Lightyear evidence reaches `GAME_OVER`, but Sprint 8 did not capture a
fully manual two-window native or browser route to `GAME_OVER`.

Do not close `S8-QA-001-W1`, full playable-client manual QA, public release
readiness, broad Standard-tier accessibility, playtest validation,
fun-hypothesis validation, or full game completion from this runbook alone.

## Current Dependencies And Blockers

| Dependency | Current status on source baseline | Evidence impact |
|---|---|---|
| Native operator controls | Prepared separately in `production/epics/playable-client/story-006-native-friend-game-operator-controls.md`; implementation is separate work | Native manual evidence is blocked until real operators can create/join rooms, confirm classes, buy/ready/bid/place/submit without debug-only or unreliable controls |
| Result acknowledgement/data contract | Prepared separately in `production/epics/game-session-system/story-009-result-acknowledgement-and-result-data-contract.md` | Return to Lobby and reconnect-at-`GAME_OVER` evidence should not be claimed until the accepted acknowledgement/result data contract is implemented or explicitly accepted as a fallback |
| Result screen MVP | Prepared separately in `production/epics/presentation-layer/story-006-result-screen-mvp.md` | Result copy, objective summary, reduced motion, focus, rematch hidden/disabled, and HUD-frozen checks need a rendered result screen before they can pass |
| Manual/browser `GAME_OVER` evidence | Not captured in Sprint 8; S8-QA-001-W1 remains carried | Future QA can close the warning only with actual manual/native/browser artifacts |

## Evidence Output Package

Create a dated capture folder for the run:

`production/qa/evidence/captures/manual-friend-game-evidence-YYYY-MM-DD/`

Required artifacts:

| Artifact | Required contents |
|---|---|
| `command-summary.md` | Exact commit, branch, OS/target, commands, ports, environment variables, tool versions, and start/stop times |
| `server.log` or `server-summary.md` | Server startup, room create/join, phase changes, C2S/S2C observations where available, `S2CGameOver`, acknowledgement handling if available, shutdown |
| `client-a.log` or `client-a-summary.md` | Host launch command, room creation, class confirm, draft/shop/auction/placement/result observations |
| `client-b.log` or `client-b-summary.md` | Joiner launch command, room join, class confirm, draft/shop/auction/placement/result observations |
| `screenshots/` | Lobby, class reveal, draft/shop, auction, placement, resolution, `GAME_OVER` result, Return to Lobby after acknowledgement if available |
| `video/` | Optional but preferred full route capture for both clients, or one desktop recording showing both windows/browsers |
| `defects.md` | Defect table with id, severity, owner/system, reproduction, impact, workaround, and evidence link |
| `route-summary.json` | Machine-readable phase/action/result summary if practical; keep room codes and local machine data redacted |

Record exact command output or summaries. Redact local usernames, raw room
codes, machine identifiers, and transient secrets before committing artifacts.

## Launch Commands

Use one local server and two clients from the same commit.

PowerShell native run:

```powershell
git rev-parse HEAD
git status --short --branch
$env:SERVER_PORT='5000'
cargo run -p server
```

Client A, in a separate shell:

```powershell
$env:SERVER_URL='ws://localhost:5000'
cargo run -p client --bin client
```

Client B, in a separate shell:

```powershell
$env:SERVER_URL='ws://localhost:5000'
cargo run -p client --bin client
```

Browser/WASM route, if the browser target is chosen and the client supports the
same controls:

```powershell
$env:SERVER_PORT='5000'
cargo run -p server
```

```powershell
cd client
trunk serve --port 8080
```

Open two independent browser contexts to the Trunk URL. Each context must use a
fresh session state unless the test explicitly covers reconnect. Confirm the
client connects to `ws://localhost:5000` or record the configured override.

If port `5000` or `8080` is in use, choose new ports and record them in
`command-summary.md`.

## Two-Client Setup

| Step | Host / Client A | Joiner / Client B | Evidence |
|---|---|---|---|
| Launch | Start client from the same commit as server | Start client from the same commit as server | Screenshot/log showing both connected or connection state |
| Room create | Activate Create Room | Wait for room code | `S2CRoomCreated` or visible room code, redacted in committed artifacts |
| Room join | Share redacted room code out-of-band | Enter room code, choose slot, activate Join | `C2SJoinRoom`, `S2CJoinAck`, visible joined slot |
| Class select | Select class and confirm | Select class and confirm | `C2SSelectClass`, `C2SConfirmClass`, `S2CClassLocked`, `S2CClassesRevealed` |
| Session start | Observe `DRAFT_INITIAL` | Observe `DRAFT_INITIAL` | Both clients show synchronized phase/session entry |

If either client cannot complete the setup because native operator controls are
missing or unreliable, stop and file a blocker against native operator controls.
Do not claim manual route completion.

## Manual Route Script

Follow the route below. If a step is blocked, record the blocker and nearest
authoritative endpoint reached.

| Route step | Required action | Pass evidence | Blocker/failure evidence |
|---|---|---|---|
| `DRAFT_INITIAL` entry | Both clients observe draft offering | Screenshot/log with offering and phase | Missing offering, phase mismatch, blank UI, or input unavailable |
| Initial purchase | Each client buys at least one offered card if possible | `C2SPurchaseCard`, `S2CCardAcquired`, `S2CGoldUpdate`, visible hand/economy update | Purchase control unavailable, no S2C convergence, or optimistic-only local change |
| Ready/retract | Exercise Ready; exercise Retract if available before final Ready | `C2SSignalReady` and authoritative phase progression | Ready control unavailable, duplicate sends, or local phase advance without S2C |
| First placement | Submit empty placement only if necessary; otherwise stage and submit a valid card | `C2SSubmitPlacement`, `S2CPlacementReveal`, `S2CResolutionEvent` | Staging/submit controls unavailable, invalid silent failure without visible recovery |
| `DRAFT_SHOP` | Buy/refresh/ready as controls allow | Shop slots, purchase/acquisition/economy S2C, ready progression | Shop controls unavailable or stale/pending state survives phase change |
| `DRAFT_AUCTION` | Place a legal bid from at least one client | `S2CAuctionCard`, `C2SPlaceBid`, accepted/rejected feedback, settlement | Bid controls unavailable, unaffordable gating broken, settlement stale after phase |
| Post-auction shop | Confirm settlement-to-shop transition and ready | Shop panel usable after settlement, no stale bid state | Auction panel remains stale or shop not usable |
| Non-empty placement | Both clients submit at least one server-owned hand card if possible | Non-empty `S2CPlacementReveal` and `UnitPlaced` in resolution replay | Placement cannot be staged/submitted manually |
| Resolution loop | Observe at least one full resolution and next phase | `S2CResolutionEvent` before following phase change | Phase/message ordering issue or unreadable replay |
| Repeat toward endpoint | Continue until `GAME_OVER` or a documented blocker | Exact route string and endpoint reached | Nearest endpoint and blocker classification |
| `GAME_OVER` | Observe `S2CGameOver` and `S2CPhaseChanged(GameOver)` | Both clients show terminal phase/result route from server data | Controlled endpoint only; no rendered/manual result claim |
| Result screen | Check victory, defeat, draw, and no-result/fallback copy if fixtures or route allow | Screenshots/logs per state, authoritative loser/reason mapping | Missing result screen or unsupported state copy |
| Result objective summary | Check five lanes per side, destroyed identity, alive `Unknown` fallback if no reveal data | Summary uses only authoritative result/snapshot data | Client infers hidden alive opponent objective identity |
| Return to Lobby / ack | Activate Return to Lobby if available | `C2SAcknowledgeResult` timing matches accepted contract; local UI returns to lobby/menu | Ack contract unavailable or Return to Lobby not implemented |
| Rematch | Confirm hidden/disabled unless protocol exists | Rematch hidden or disabled and not focusable | Enabled rematch without protocol or C2S support |

## Result-Screen UX Checks

Run these checks only after the result screen exists. Until then, mark them
`BLOCKED - result screen not implemented`.

| Check | Expected result |
|---|---|
| Victory copy | `VICTORY`; cause maps from server-authored `loser == opponent` and `reason` |
| Defeat copy | `DEFEAT`; cause maps from server-authored `loser == local_player` and `reason` |
| Draw copy | `DRAW`; no player is presented as winner |
| No-result/fallback | `NO RESULT` for `ResolutionTimeout`, or `RESULT PENDING`/equivalent when result payload is missing |
| Objective summary | Five lanes per side; final HP and alive/destroyed state; real/fake only where server provided identity |
| Return to Lobby | Usable by click and keyboard; sends or relies on `C2SAcknowledgeResult` only according to the accepted contract |
| Rematch | Hidden or disabled while rematch protocol is unsupported; disabled control is not focusable |
| Focus | Initial focus is on heading or first available action; Tab reaches Return to Lobby; Enter activates focused action; Escape focuses Return to Lobby without auto-exiting |
| Reduced motion | No iris wipe, bloom flash, scale pulse, row sequencing, repeated flash, or card travel; all result information remains visible |
| HUD frozen behind result | HUD remains visible behind overlay, phase reads `GAME OVER`, round/final resources remain stable, no real/fake markers are added to HUD dots |
| Objective summary fallback | Alive opponent identities remain `Unknown` unless a post-game reveal payload exists |
| Viewport/layout | `1366x768`, `1920x1080`, and 150 percent UI scale show no overlap or clipped critical text/controls |

## Defects Table Template

| ID | Severity | Owner/System | Status | Reproduction | Friend-game impact | Workaround | Evidence |
|---|---|---|---|---|---|---|---|
| MANUAL-FG-001 | TBD | TBD | Open | TBD | TBD | TBD | TBD |

Severity guidance:

- `S1`: blocks launch or data safety for the run.
- `S2`: blocks the manual route or corrupts server-authoritative state.
- `S3`: impairs evidence quality or player operation but has a workaround.
- `S4`: minor evidence/readability issue.

## Completion And Claim Boundary

A future evidence pass can mark the manual route complete only when the
committed artifacts show:

- Exact commit and commands.
- One real local server and two real native/browser clients from that commit.
- Room create/join, class confirm, draft/shop, auction, placement, resolution,
  `GAME_OVER`, result display, and Return to Lobby/ack if available.
- Server, Client A, and Client B logs or summaries.
- Screenshots/video for the visible route.
- Defects table and carried conditions.

If any dependency remains unresolved, record the route as blocked or partial and
preserve the non-claims.

## Non-Claims

- No completed manual/native/browser `GAME_OVER` evidence from this runbook.
- No public release readiness.
- No store, deployment, release-candidate, or launch readiness.
- No broad Standard-tier accessibility completion.
- No closure of `QA-COND-0005`.
- No playtest validation.
- No fun-hypothesis validation.
- No closure of `QA-COND-0006`.
- No full playable-client manual QA.
- No full regression campaign.
- No full game completion.
