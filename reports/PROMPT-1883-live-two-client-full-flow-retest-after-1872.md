# PROMPT 1883 — Live Two-Client Full-Flow Retest After PROMPT 1872

**Date:** 2026-05-28
**Worker branch:** `wt/1883-two-client-retest`
**Worktree:** `D:\tmp\wt-1883-two-client-retest`
**Base commit (HEAD):** `2ce3dc6b0a793ab16d6325636867f59e930a5aea` (origin/main)
**Launcher used:** `tools/dev-launcher/Start-TwoClients.ps1 -Port 5001`
**Evidence dir:** `D:\tmp\wt-1883-two-client-retest\production\qa\evidence\dev-runs\2026-05-28-114933\`

---

## Phase Results Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Launcher dry-run | **PASS** | exits 0, resolves worktree root, finds both binaries |
| Server startup | **PASS** | binds port 5001 in <1 s, loads 16 cards, enters Lobby |
| Client A connection | **FAIL** | connects, receives S2CHandshake, then panics — protocol mismatch |
| Client B connection | **FAIL** | connects, disconnects immediately — same protocol mismatch |
| Lobby (class select) | **BLOCKED** | clients never reach UI — crash at handshake |
| Shop / Auction | **BLOCKED** | not reached |
| Placement | **BLOCKED** | not reached |
| Resolution / Combat | **BLOCKED** | not reached |

**Overall verdict: PARTIAL** — server launch and network bind confirmed; all client-side phases blocked by binary staleness.

---

## Blocker: Client Binary Protocol Mismatch

### Root Cause

The `client.exe` and `server.exe` were compiled from different source commits and carry incompatible lightyear protocol hashes.

| Binary | Path | Built at (UTC) | Source commit |
|--------|------|---------------|---------------|
| `server.exe` | `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe` | 2026-05-28T01:13:26Z | unknown (no provenance sidecar) |
| `client.exe` | `D:\_DEV\cargo-target\ccgs-msvc\debug\client.exe` | 2026-05-21T10:42:46Z | `3a4603af` on branch `play-main` |

The client binary predates **20+ merged Rust source commits** on main, including protocol-affecting changes:

- PROMPT 1729 — S18 UI interaction state migration wave 2 (client/shared protocol types)
- PROMPT 1722 — legal_action_count added to draft/shop evidence messages
- PROMPT 1720 — winner/reason fields added to final_state
- PROMPT 1719 — placement coord logging in BotDecisionKind
- PROMPT 1678 — bot draft auto-pick + empty-batch placement fix
- PROMPT 1675 — bot lobby auto-confirm guard
- PROMPT 1672 — bot-soak-trigger headless client driver
- PROMPT 1652 — Autoplay vs Bot QA button added

### Panic Text (Client A + B, identical)

```
thread 'main' panicked at bevy_ecs-0.18.1/src/error/handler.rs:125:1:
Encountered an error in observer
  `lightyear::protocol::ProtocolCheckPlugin::receive_verify_protocol`:
  the message protocol doesn't match

Encountered a panic when applying buffers for system `MessagePlugin::recv`!
Encountered a panic in system `bevy_ecs::apply_deferred`!
Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
```

Source: `client_a.log.err`, `client_b.log.err`

### Server Timeline (from `server.log`)

```
11:49:47 INFO  server                        Lanes and Lies server starting
11:49:47 INFO  server::foundation::config    AppState::Loading — requesting game_config.ron and cards.json
11:49:47 INFO  lightyear_websocket::server   Server WebSocket starting at 0.0.0.0:5001
11:49:47 INFO  server::foundation::config    Both assets loaded — transitioning to AppState::ConfigValidation
11:49:47 INFO  server::foundation::config    Assets loaded: GameConfig + CardCatalog (16 cards)
                                             — transitioning to AppState::Lobby
11:49:59 INFO  server::network               Client connected: Raw(127.0.0.1:50120)
11:49:59 INFO  server::core::session         snapshot_sent registered for player_id=1 (fresh=true)
11:49:59 INFO  server::core::session         send_reconnect_dispatch: S2CHandshake → player_id=1
11:49:59 INFO  server::network               Client connected: Raw(127.0.0.1:50121)
11:49:59 INFO  server::network               Client disconnected: Raw(127.0.0.1:50121)  ← Client B
11:49:59 INFO  server::core::rsm::transitions RSM lightyear disconnected player_id=1 phase=Lobby
11:49:59 INFO  server::network               Client disconnected: Raw(127.0.0.1:50120)  ← Client A
```

Server is clean — no ERRORs, no panics. Both clients appear to disconnect from the server's perspective (not a server bug).

---

## Build Provenance

From `build.json` (generated at launch):

```json
{
  "git": {
    "branch": "wt/1883-two-client-retest",
    "head_sha": "2ce3dc6b0a793ab16d6325636867f59e930a5aea",
    "is_clean": true
  },
  "build": {
    "profile": "debug",
    "server": { "modified_at": "2026-05-28T01:13:26Z" },
    "client": { "modified_at": "2026-05-21T10:42:46Z" }
  },
  "last_rebuild": {
    "git": { "head_short": "3a4603af3227", "branch": "play-main" },
    "generated_at_utc": "2026-05-21T10:42:48Z"
  }
}
```

**Gap**: 7 days + 20 Rust source commits between client and server build.

---

## Rebuild Attempt

A targeted `cargo build -p server` was attempted from the worktree but was killed by the environment (exit code 137 — OOM or process timeout in the Claude Code session). No rebuild completed.

**Required operator action to unblock:**

```powershell
# From D:\_DEV\Work\Claude-Code-Game-Studios or any fresh worktree:
$env:CARGO_TARGET_DIR      = "D:\_DEV\cargo-target\ccgs-msvc"
$env:CARGO_PROFILE_DEV_DEBUG = "0"
$env:CARGO_INCREMENTAL      = "0"
$env:RUSTFLAGS              = "-C debuginfo=0 -C link-arg=/DEBUG:NONE"

cargo build -p server
cargo build -p client --bin client

# Then re-run two-client test:
powershell -File tools\dev-launcher\Start-TwoClients.ps1
```

Or use the one-click bat: `start-two-clients.bat`
(Launcher auto-builds if binaries are MISSING; but since they EXIST and are only stale, forced rebuild via the above is required.)

---

## Server-Side State Machine Coverage (from Prior Bot-vs-Bot Soak)

The latest bot-vs-bot soak (`2026-05-27T22:26 UTC`, evidence at
`production/qa/evidence/dev-runs/2026-05-27-222625-bot-vs-bot-soak/`) completed 3 full rounds:

```
DraftShop → Placement → Resolution (×2) → GameOver(MaxRoundsReached)
```

Key verified transitions from server.log:
- `DraftShop`: both bots auto-picked cards (`card_id=101`)
- `Placement → Resolution`: both placements accepted; 2 committed records, 1 unit spawned
- `resolve_combat`: exit round=2 kills=0
- `broadcast_game_over`: reason=MaxRoundsReached, round=3

This confirms the server-side flow is intact but was tested with **bot clients** (same-commit binaries), not the stale human client.exe.

---

## Explicit Phase Coverage Table

| Phase | Coverage Method | Outcome |
|-------|----------------|---------|
| Server startup + config load | Live run (this PROMPT) | PASS |
| Server port bind | Live run (this PROMPT) | PASS |
| Client network connect | Live run (this PROMPT) | FAIL — protocol mismatch |
| Lobby UI visible | ❌ Not reached | BLOCKED |
| Class selection UI | ❌ Not reached | BLOCKED |
| Class confirmation | ❌ Not reached | BLOCKED |
| Shop/auction offers visible | ❌ Not reached | BLOCKED |
| Auction timer readable | ❌ Not reached | BLOCKED |
| Leader label perspective | ❌ Not reached | BLOCKED |
| Won-card disposition | ❌ Not reached | BLOCKED |
| Auction overlay/z-order | ❌ Not reached | BLOCKED |
| Placement drag card | ❌ Not reached | BLOCKED |
| Placement target highlight | ❌ Not reached | BLOCKED |
| Placement submit + ACK | ❌ Not reached | BLOCKED |
| Unit visible on board | ❌ Not reached | BLOCKED |
| Resolution / combat | Bot-vs-bot soak (prior) | PASS (server-side only) |
| Game over | Bot-vs-bot soak (prior) | PASS (server-side only) |

---

## Evidence Artifacts

All under `D:\tmp\wt-1883-two-client-retest\production\qa\evidence\dev-runs\2026-05-28-114933\`:

| File | Status |
|------|--------|
| `server.log` | Clean — no ERRORs, no panics |
| `server.log.err` | Empty |
| `client_a.log` | 41 lines — asset path errors then C2SHello then WARN |
| `client_a.log.err` | Protocol panic |
| `client_b.log` | Minimal — connect/disconnect |
| `client_b.log.err` | Protocol panic (identical) |
| `build.json` | Full binary provenance recorded |
| `launch-summary.json` | PIDs, ports, paths |

**Additional note**: Client logs show asset path errors from old worktree `D:\Tmp\wt-1610\client/../assets\...` — the binary was built in a `wt-1610` worktree and has a stale `MANIFEST_DIR`-derived asset root. This is a secondary issue (non-fatal in Bevy, missing assets get placeholder errors) masked by the protocol panic.

---

## Pre-existing Server on Port 5000

A stale `server.exe` (PID 61924) has been running since `2026-05-28T02:13:29` on port 5000 with no connected clients. It was left running from a prior session and was not killed during this test (port 5001 used instead). Operator may stop it:

```powershell
Stop-Process -Id 61924
```

---

## Conclusion

The two-client full-flow test is **blocked by a binary staleness mismatch**. The server starts clean and the network stack is healthy; the blocker is entirely on the client binary side. Once the operator rebuilds `client.exe` (and optionally `server.exe`) from the same commit, the launcher is ready to run a fresh two-client session.

**Recommended follow-up**: After rebuilding, run the operator checklist from PROMPT 1706 (phases L-01→R-04) with the fresh binaries. Server-side state machine evidence from the bot-vs-bot soak is current and does not need re-running.

---

1883: LIVE-TWO-CLIENT-FULL-FLOW-RETEST-AFTER-1872: PARTIAL
