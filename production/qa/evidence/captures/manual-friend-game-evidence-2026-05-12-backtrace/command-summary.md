# Manual Friend-Game Evidence — 2026-05-12 (backtrace rerun)

> **Status**: Backtrace rerun per orchestrator instruction. **Regression did NOT reproduce.**
> 12 full rounds completed cleanly; no server panic, no backtrace to capture.
> The committed logs are noise-filtered to fit in the repo (412 KB total). Raw logs (1.3 GB)
> are preserved locally outside the repo at `C:\Users\Sam\playtest-raw-logs\2026-05-12-backtrace\`.

## Environment

| Field | Value |
|---|---|
| Date / time | 2026-05-12, ~13:24 – 14:00 UTC (server uptime ~35 min) |
| OS / target | Windows 11 Pro, native build (not WASM) |
| Commit | `8e3d04499bb8314d31e781626ec64e4e3149c2f0` (branch `main`, same as the earlier crash run) |
| Working tree | Identical to earlier run — staged `design/levels/ossuary-of-the-drowned-king.json`, untracked `design/levels/dungeon-blueprint.json`. Neither file is loaded at runtime. |
| Rust toolchain | cargo 1.95.0 / rustc 1.95.0 (59807616e 2026-04-14) |
| GPU | NVIDIA GeForce RTX 5090 Laptop, Vulkan backend |
| Server env | `SERVER_PORT=5000`, `RUST_BACKTRACE=full`, `RUST_LOG=server=info,bevy=warn` |
| Client env | `SERVER_URL=ws://localhost:5000` |

## Commands

Built clean before launching (workspace built in 4m 37s after a prior failed incremental rebuild + disk exhaustion). Launched binaries directly rather than via `cargo run` to avoid further incremental-rebuild / artifact-lock issues; the env vars (`RUST_BACKTRACE=full`, `RUST_LOG`) apply at process launch regardless of how the binary is invoked. This is a minor deviation from the literal orchestrator command (`cargo run -p server`) but produces identical runtime behavior because no source files were touched between the workspace build and the launch.

```bash
# Workspace build (single pass to produce both binaries)
cargo build --workspace

# Server (background, stderr merged, teed)
SERVER_PORT=5000 RUST_BACKTRACE=full RUST_LOG=server=info,bevy=warn \
  ./target/debug/server.exe 2>&1 | tee server.log

# Client A (background, stderr merged, teed)
SERVER_URL=ws://localhost:5000 \
  ./target/debug/client.exe 2>&1 | tee client-a.log

# Client B (background, stderr merged, teed)
SERVER_URL=ws://localhost:5000 \
  ./target/debug/client.exe 2>&1 | tee client-b.log
```

## Files

| File | Size | Origin |
|---|---|---|
| `server.log` | 232 KB | Server stdout/stderr, with `initialize_player_pools_on_draft_started` per-frame noise filtered out (`grep -v`). All phase transitions, combat resolutions, network events, and any warnings/panics are retained. |
| `client-a.log` | 82 KB | Client A (`PlayerId(1)`, session 1), with `hand_ui_phase_transition` / `hand_ui_pending_placements_cleared` / `C2SHeartbeat` / `S2CSessionConfig` / `S2CPhaseChanged` per-frame noise filtered out. |
| `client-b.log` | 93 KB | Client B (`PlayerId(2)`, session 2), same filtering as client-a. |

Raw unfiltered logs are preserved at `C:\Users\Sam\playtest-raw-logs\2026-05-12-backtrace\`:
- `server.log.raw` — 1,160,688,141 bytes (1.16 GB)
- `client-a.log.raw` — 91,917,845 bytes (92 MB)
- `client-b.log.raw` — 82,673,504 bytes (83 MB)

The size disparity is dominated by `initialize_player_pools_on_draft_started: entered (session=true, catalog=true, config=true)` firing every frame for the full 35-minute session (~125,000+ lines). This log line is a stray `tracing::info!()` call at `server/src/core/pool/system.rs:21` that runs unconditionally before the `for message in draft_started.read()` message-reader guard — it is unrelated to the regression and exists in the current main branch as logging noise.

## Phase Timeline (from filtered server.log)

| Time | Phase / Event |
|---|---|
| 13:24:53 | Server boot → `AppState::Loading` → assets loaded (16 cards) → `AppState::Lobby` |
| 13:25:14 | Client A connected (`PlayerId(1)`, peer `127.0.0.1:61629`), `S2CHandshake` |
| 13:25:21 | Client B connected (`PlayerId(2)`, peer `127.0.0.1:61632`), `S2CHandshake` |
| 13:26:11 | Client A `c2s_create_room` (OneVOne) → room created |
| 13:26:23 | Client A `c2s_confirm_class` → Locked |
| 13:26:32 | Client B `c2s_join_room` → Joined |
| 13:26:43 | Client B `c2s_confirm_class` → Locked |
| 13:26:44 | RSM `on_session_ready: entering DRAFT_INITIAL` (player_count=2, round=0) |
| 13:27:24 | `DraftInitial → Placement` (round 1) |
| 13:27:34 | `Placement → Resolution` (round 1, kills=0) → `Resolution → DraftShop` (round 2) |
| **13:28:04** | **`DraftShop → Placement` (round 2) — passed clean, no crash** ⭐ |
| 13:28:14 | `Placement → Resolution` (round 2, kills=0) → `Resolution → DraftAuction` (round 3) |
| 13:28:42 | Auction round 3 settled (winner=None, amount=0) → `DraftAuction → DraftShop` (round 3) |
| 13:29:12 | `DraftShop → Placement` (round 3) |
| 13:29:24 | Resolution round 3 → DraftShop round 4 |
| 13:29:54 | Placement round 4 |
| 13:30:04 | Resolution round 4 → DraftShop round 5 |
| 13:30:34 | Placement round 5 |
| 13:30:44 | Resolution round 5 → DraftAuction round 6 |
| 13:31:11 | Auction round 6 settled (winner=`PlayerId(2)`, amount=6) → DraftShop round 6 |
| 13:31:41 | Placement round 6 |
| 13:31:53 | Resolution round 6 (kills=0) |
| 13:32:23+ | Continued through rounds 7–12 without crash |
| 14:00:33 | Client B window closed by user → `No windows are open, exiting` (during DraftAuction round 12) |
| 14:00:??? | Client A window closed by user |
| 14:??:??? | Server stopped manually via `TaskStop` (no panic — server was still running normally) |

## Difference vs the 12:01 Crash Run

| Behaviour | 12:01 run (crashed at R2 Placement entry) | 13:24 run (12 rounds clean, no crash) |
|---|---|---|
| Server commit | `8e3d044` | `8e3d044` (identical) |
| Combat kills | 0 every round | 0 every round |
| `c2s_purchase_card` count in DraftInitial | 4 (Cards 103, 1, 1, 5) | **0** (no draft purchases) |
| `c2s_activate_card` count in DraftShop R2 | 6 activations on cards 1, 5, 103 | **0** (idle through shop phase) |
| `placement_buffer_open` for round 2 | Fired, then process exited immediately after | Fired, then transitioned cleanly to Resolution |
| Auction interaction | Did not reach DraftAuction | Reached DraftAuction at rounds 3 (no winner) and 6 (winner=P2, amount=6) |

The two runs are on the same commit with no source changes. The only behavioural difference is **player interaction**: the crashing run purchased and activated cards in DraftInitial+DraftShop; the clean run did neither.

## Conclusion

The "server panics on entering Placement round 2 unconditionally" interpretation from the earlier capture (`production/qa/evidence/captures/manual-friend-game-evidence-2026-05-12/command-summary.md`) is **not correct**. The regression is real but **conditional on player-side card interactions in earlier phases** (purchase in DraftInitial and/or activate in DraftShop), not on phase transition timing alone.

`RUST_BACKTRACE=full` was active throughout this rerun but had no panic to capture — the process exited via external `TaskStop`, not via panic.

## Suggested Next Steps for the Orchestrator

1. **Targeted repro run**: replay the exact 12:01 interaction pattern — both players each `c2s_purchase_card` 1–2 cards from DraftInitial, ready up; in DraftShop round 2 each player `c2s_activate_card` a few times — then idle through to Placement R2 entry and watch for the panic with `RUST_BACKTRACE=full`. This is the smallest path to the panic backtrace the orchestrator originally asked for.
2. **Hypothesis to investigate first**: the crash is in a Placement-entry system that touches purchased/activated cards. Likely candidates: `placement_buffer_open` consumer, anything reading `PlayerPool` / `Hand` / `ShopSlots` on `OnEnter(Phase::Placement)` for round ≥ 2.
3. **Independent: silence the per-frame log noise**. Move the `tracing::info!()` at `server/src/core/pool/system.rs:21` inside the `for message in draft_started.read()` loop. This run produced 1.16 GB of log purely from that one stray line; future captures will be massively cheaper to handle.
