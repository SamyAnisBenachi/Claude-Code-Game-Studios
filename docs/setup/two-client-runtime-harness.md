# Two-Client Runtime Harness

> Story: `S13-TWO-CLIENT-RUNTIME-HARNESS-001`
> Source: PROMPT 803 §3 DC-14, §4 Lane E, §5 Must row 6
> Path: `tools/two-client-runtime/`
> Authored: 2026-05-14 (PROMPT 858)

A non-interactive scripted two-client runtime harness that drives the
friend-game route end-to-end against the **real** Lightyear WebSocket server
in a single process. The harness exists to unblock:

- Story 019 (drag-runtime retest) tighter-capture invocation.
- A future `S8-QA-001-W1` closure attempt (manual two-client GAME_OVER
  evidence). The closure verdict is **not** rendered by this harness; AC12
  of Story 017 explicitly forbids auto-closure and reserves the verdict for
  a separate `/story-done` prompt with QA-lead sign-off.

This is a **developer-invokable** driver, not a CI gate. CI integration is
scoped to a Sprint 14 follow-on if stability allows.

## Canonical invocation

```pwsh
# Recommended Cargo policy on Windows / MSVC (story-scoped):
$env:CARGO_TARGET_DIR = 'D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL = '0'
$env:RUSTFLAGS = '-C debuginfo=0 -C link-arg=/DEBUG:NONE'

cargo build -p two-client-runtime --bin two-client-runtime
cargo run -p two-client-runtime --bin two-client-runtime -- `
    --seed 1 --max-rounds 10
```

Without overrides, evidence lands at
`production/qa/evidence/captures/sprint-13-two-client-runtime/<UTC-date>/`
(matching the `manual-friend-game-evidence-YYYY-MM-DD/` runbook precedent).

## Supported flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--seed N` | `1` | ChaCha20 seed for the deterministic server RNG. Two runs with the same seed produce byte-identical `routes_observed` blocks in `final_state.json` (AC7). |
| `--max-rounds N` | `10` | Hard cap on observed `Placement` phase entries. The harness exits with `endpoint_reached = "max_rounds"` (success) when the cap is hit. |
| `--connect-timeout-secs N` | `5` | AC2 binding -- both clients must complete the `S2CHandshake` exchange within this wall-clock budget. |
| `--overall-timeout-secs N` | `120` | Hard wall-clock cap for the entire run. Exceeding it is `endpoint_reached = "overall_timeout"` (failure). |
| `--evidence-dir PATH` | dated path under `production/qa/evidence/captures/sprint-13-two-client-runtime/` | Override the per-run evidence root. |
| `--port N` | ephemeral (`TcpListener::bind 0`) | Server bind port. Use a fixed port when running the harness in parallel with other harnesses to avoid clashes. |
| `-h` / `--help` | | Print usage and exit. |

## Evidence bundle layout (AC5)

Per successful run, the evidence directory contains:

- `harness.log` -- driver log (CLI parsing, tick orchestration, endpoint
  detection).
- `server.log` -- server `App` tracing (production tick rate, RSM phase
  transitions, S2C dispatch).
- `client_a.log` -- client A `App` tracing (probe S2C receivers,
  handshake / phase-changed observations).
- `client_b.log` -- client B `App` tracing.
- `final_state.json` -- stable serialised facts (seed, port, route counters,
  observed phases, `endpoint_reached`).

All log lines carry an ISO-8601 UTC timestamp at millisecond precision
(AC6) via the `tracing-subscriber` `UtcTime::rfc_3339()` timer. Production
subscribers already use the same timer per `S13-OBS-WALLCLOCK-TIMESTAMPS-001`
(PROMPT 837), so cross-process correlation is direct.

## Determinism (AC7)

Two runs with the same `--seed N` and `--port N` produce byte-identical
`final_state.json` (modulo wall-clock timestamps in the log files). Using
an ephemeral port (the default) means `server_port` and `websocket_bind_addr`
differ between runs; the `routes_observed` block remains identical. Use
`--port N` for full identity if downstream evidence comparison needs the
top-level fields to match too.

## Architecture notes

- Server runs in-process via the production `ServerNetworkPlugin` (real
  Lightyear `ServerPlugins`, real WebSocket transport, real protocol
  registry). The harness sets `SERVER_PORT` so the production
  `open_websocket_server` system binds to the harness-reserved port.
- Both clients run as separate Bevy `App`s (Bevy 0.18 `App::update()`
  ticking, no `DefaultPlugins` / no windowing) that connect via
  `WebSocketClientIo::from_url`. The harness ticks all three apps
  sequentially on the main thread.
- Per-role log routing uses a process-global `AtomicU8` consulted by a
  custom `MakeWriter`. The driver sets the role to `Server` / `ClientA` /
  `ClientB` / `Harness` immediately before each app's `update()` call;
  tracing emitted during that tick lands in the role's file.
- Seeded determinism is wired through `ServerRngFactory::new(fn-ptr)` --
  the `ServerRngFactory` takes a `fn` pointer (not a closure), so the seed
  is parked in a process-global cell that the factory reads at session
  start. Safe because the harness runs only one server App per process.

## Known limitations (deferred follow-ons)

| Limitation | Scope | Follow-on |
|------------|-------|-----------|
| Does **not** reach S2CGameOver by default; hits `--max-rounds` cap first. AC3 explicitly permits the max-round cutoff as a valid termination. | Reaching GAME_OVER requires placing units on the planned-objective lanes (per the protocol-level full game-over test). The harness uses empty `C2SSubmitPlacement` for simplicity. | Sprint 14 enhancement -- port the planned-objective scripting from `tests/integration/playable_client/full_game_over_route_test.rs`. |
| Does **not** test the reconnect path (no mid-game disconnect). | Out of scope per Story 017 "Out of Scope" §. | Sprint 14 follow-on. |
| Per-target tracing (`tracing::info!(target: "...")`) is currently sparse on the client side. | `S13-OBS-TRACING-TARGETS-001` (Story 018, integrated PROMPT 850) wires more targets; until those land in every receiver, the harness adds probe-side traces in `route.rs`. | Already scoped (Story 018). |
| Multi-platform CI matrix. | Out of scope. The harness is developer-invokable on Linux / macOS / Windows. | Future. |

## AC12 binding -- `S8-QA-001-W1` is NOT closed by running this harness

Running this harness produces evidence that **may** be used to close
`S8-QA-001-W1` in a subsequent `/story-done` prompt, but the closure
verdict requires:

1. Producer decision on whether the harness's automated evidence satisfies
   the "manual two-client GAME_OVER" gap, or whether a human operator
   runbook execution is still required.
2. QA-lead sign-off on the evidence bundle.
3. A separate `/story-done` prompt that cites the evidence path and
   updates `production/sprint-status.yaml`.

Story 017's implementation prompt (PROMPT 858) does **not** close
`S8-QA-001-W1`; AC12 explicitly forbids auto-closure.

## No optimistic client-side authority

ADR-002 binding. The harness scripts the route via real C2S intents only;
the clients are read-only views over server-authoritative S2C broadcasts.
No client-side mirror is mutated outside the message receivers; the harness
itself touches **no** production code in `client/`, `server/`, or `shared/`.
See `tools/two-client-runtime/src/route.rs` for the per-phase scripted
intent emitters and the per-S2C-message recorders.
