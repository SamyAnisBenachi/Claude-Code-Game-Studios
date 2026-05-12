# Manual Friend-Game Evidence — 2026-05-12

> **Status**: Native 2-client manual run, ad-hoc (not a planned QA pass).
> Captured because the run hit a server-side regression entering Placement round 2.
> Logs are preserved here so the orchestrator and future sessions can find them
> without having to dig into the Claude Code temp directory.

## Environment

| Field | Value |
|---|---|
| Date / time | 2026-05-12, ~12:01–12:08 UTC |
| OS / target | Windows 11 Pro, native build (not WASM) |
| Commit | `8e3d04499bb8314d31e781626ec64e4e3149c2f0` (branch `main`) |
| Working tree | Dirty — staged: `design/levels/ossuary-of-the-drowned-king.json` (added); untracked: `design/levels/dungeon-blueprint.json`. Neither file is loaded at runtime by the server or client, so they do not affect this run. |
| Rust toolchain | cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| GPU | NVIDIA GeForce RTX 5090 Laptop, Vulkan backend |
| Server port | `5000` (env `SERVER_PORT=5000`) |
| Client URL | `ws://localhost:5000` (env `SERVER_URL=ws://localhost:5000`) |

## Commands

Workspace was pre-built via `cargo build --workspace` (finished in 1m 30s on dev profile, 0 errors). Subsequent runs invoked the pre-built binaries directly to avoid cargo's artifact-directory lock contention between two parallel `cargo run` invocations.

```bash
# Server (background)
SERVER_PORT=5000 cargo run -p server

# Client A (background, pre-built binary)
SERVER_URL=ws://localhost:5000 ./target/debug/client.exe

# Client B (background, pre-built binary)
SERVER_URL=ws://localhost:5000 ./target/debug/client.exe
```

## Files

| File | Origin |
|---|---|
| `server.log` | stdout/stderr of `target\debug\server.exe` |
| `client-a.log` | stdout/stderr of Client A (`PlayerId(1)`, session_id 1) |
| `client-b.log` | stdout/stderr of Client B (`PlayerId(2)`, session_id 2) |

## Route Walked (from server log)

| Time | Event |
|---|---|
| 12:01:27 | Server up on `0.0.0.0:5000`, `AppState::Lobby` (assets: `GameConfig` + 16-card `CardCatalog`) |
| 12:06:23 / 24 | Client A then Client B connected, `S2CHandshake` dispatched |
| 12:06:43 | Client A `c2s_create_room` (OneVOne) → room `AFHS93`, session `7b8388bc-95d4-4d45-9f13-6588b5e33a2a` |
| 12:06:48 | Client A confirmed class `Iop` (Locked) |
| 12:06:59.094 | Client B `c2s_join_room` `AFHS93` slot 1 → Joined |
| 12:06:59.857 | Client B confirmed class `Iop` (Locked) |
| 12:06:59.860 | RSM `on_session_ready: entering DRAFT_INITIAL` (round 1, auction_round=false) |
| 12:06:59.862 | `acquisition_tick` drained 2 `ShopRefreshTriggered`, built 9-card offerings for both players, broadcast `S2CDraftOffering` |
| 12:06:59.863 | `send_objective_identities` to both players (`identity_count=5`) |
| 12:07:01–08 | Both players purchased cards from draft (Card 1, 5, 103) and signalled ready |
| 12:07:08.816 | RSM `advance_phase: DraftInitial → Placement` (round 1) |
| 12:07:08.816 | `placement_buffer_open: previous_submissions=0` |
| 12:07:18.817 | RSM `phase timer finished` (placement, round 1) → `Placement → Resolution` |
| 12:07:18.818–820 | `resolve_combat: round=1 kills=0`, broadcast `S2CResolutionEvent` (events_len=6) |
| 12:07:18.821 | `Resolution → next draft → DraftShop` (round 2); economy reset to gold=3 for both; shop refresh, `S2CShopSlots` (3 slots each) |
| 12:07:25–47 | Both players sent `c2s_activate_card` repeatedly (Card 1, 5, 103) |
| 12:07:48.838 | RSM `phase timer finished` (draft_shop, round 2) → `DraftShop → Placement` (round 2) |
| 12:07:48.838 | `placement_buffer_open: previous_submissions=0` (round 2 entry) |
| 12:07:48.838+ | **Server process exits with code 1** — no panic line captured in stdout |

## Regression Identified

**The server dies on the second entry into `Placement` (round 2), immediately after `placement_buffer_open` fires for round 2.** Round 1 placement + resolution + transition into DraftShop round 2 all went through cleanly. The crash is reproducible on this commit (`8e3d044`).

No stderr panic line was captured in `server.log` — likely because the panic happened on a worker thread and the default panic handler exited before the log was flushed. Next reproduction should run with `RUST_BACKTRACE=full` and merge stderr into the captured output to recover the panic message.

## Noise To Ignore

The `initialize_player_pools_on_draft_started: entered (session=true, catalog=true, config=true)` line repeats every frame from `12:06:59.860` onward (~60 hz). This is a stray top-level `tracing::info!()` call in `server/src/core/pool/system.rs:21` that fires before the system's `draft_started.read()` guard. The system itself is well-behaved; only the log line is unconditional. This is not the cause of the crash — it is unrelated logging noise.

## Suggested Next Steps

1. Re-run with `RUST_BACKTRACE=full` and stderr merged (`SERVER_PORT=5000 RUST_BACKTRACE=full cargo run -p server 2>&1 | tee server.log`) to capture the panic message at round-2 Placement entry.
2. Inspect the `OnEnter(Placement)` / `placement_buffer_open` consumer + adjacent systems for an unguarded `unwrap()` or a state assumption that holds on round 1 but not round 2 (e.g., a resource that was consumed in round 1's resolution and isn't re-initialised before round 2 placement).
3. Move the `initialize_player_pools_on_draft_started` `tracing::info!()` inside the `for message in draft_started.read()` loop to silence the per-frame noise.
