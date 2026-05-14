# Story 017: S13-TWO-CLIENT-RUNTIME-HARNESS-001 -- Non-Interactive Scripted Two-Client Runtime Harness

> **Epic**: Playable Client
> **Story ID**: S13-TWO-CLIENT-RUNTIME-HARNESS-001
> **Status**: Done (PROMPT 871, 2026-05-14) -- `/story-done` paperwork closure
> verified AC1-AC13 all PASS against integrated evidence on
> `origin/main@3cf5e41` (PROMPT 870 integration merge of PROMPT 858 worker
> commit `cb4454a`). Sprint 13 Must Have row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`
> flipped `ready -> done` in `production/sprint-status.yaml` (top-level
> `sprint:` / `status:` / `stage:` unchanged). AC12 forbid-auto-closure
> preserved: `S8-QA-001-W1` remains OPEN and is NOT closed by PROMPT 871.
> **Layer**: Test Infrastructure / Friend-Game Runtime Evidence
> **Type**: Integration -- new cargo binary / workspace member (test harness)
> **Sprint**: Sprint 13 candidate (per PROMPT 803 §6 line 142; NOT activated)
> **Authored**: 2026-05-14 by PROMPT 804 (worktree
> `work/s13-runtime-hardening-story-authoring`)
> **Authoring source-of-truth**: `origin/main@b5eef0d` (PROMPT 799 Sprint 12
> QA-plan commit). Sprint 12 active per PROMPT 798 at `origin/main@796851b`.

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 804. Sprint 12 remains the active sprint
(`status: active`) and must not be changed by this authoring run.

PROMPT 804 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-12.md` or any other active sprint
  file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify `production/qa/qa-plan-sprint-12.md` or any other QA-plan file.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Close `S8-QA-001-W1`. Authoring this story does **not** by itself
  close the manual two-client GAME_OVER gap; closure (if any) is
  attempted only when the harness is built, run, and produces the
  required evidence -- and even then, the closure verdict is recorded
  under a separate `/story-done` prompt with explicit QA-lead
  sign-off.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 disposition (`closed-with-conditions`) and Sprint 11
disposition (`closed-with-conditions`) remain unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence is preserved.

**No optimistic client-side authority is introduced or proposed by this
story.** The harness is a *driver* that scripts the friend-game route
via real C2S intents against the real Lightyear server; the clients
remain read-only views over server-authoritative state. ADR-002 +
ADR-008 + ADR-009 + ADR-011 + ADR-012 binding for every step the
harness executes.

---

## Source Finding (PROMPT 803)

`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`:

- **§3 DC-14** Manual two-client route coverage gap (MED procedural):
  Automated 16/16 result-screen tests pass; manual two-client
  GAME_OVER route never captured → `S8-QA-001-W1` OPEN since
  Sprint 8. AI agent cannot operate windowed Bevy clients; needs
  human operator runbook execution.
- **§3 DC-11** Tracing target hierarchy unscoped (HIGH for
  diagnostic): Story 019 invocation `RUST_LOG=...` would capture
  only crate-level emissions; *no* `tracing::*!(target: "...")`
  calls exist. Until `S13-OBS-TRACING-TARGETS-001` lands, the
  harness's structured log capture is partial.
- **§3 DC-12** No wall-clock ISO-8601 timestamping in subscribers
  (HIGH for diagnostic). Until `S13-OBS-WALLCLOCK-TIMESTAMPS-001`
  lands, multi-process correlation requires a shell-wrapper UTC
  prefix.
- **§4 Lane E "Manual two-client route"**: automated harnesses exist
  (`tests/integration/network/os18b_two_client_objective_hp_visibility_test.rs:46`,
  `tests/integration/playable_client/full_game_over_route_test.rs:66`).
  Human operator runbook prepared at
  `production/qa/evidence/manual-friend-game-evidence-runbook.md`.
  `S8-QA-001-W1` remains OPEN.
- **§5 Must row 6 (S13-TWO-CLIENT-RUNTIME-HARNESS-001)**: "Author a
  non-interactive scripted two-client harness (cargo bin) that
  drives the full friend-game route end-to-end against the real
  server, with structured log capture, ready for Story 019
  tighter-capture invocation and for `S8-QA-001-W1` evidence".
- **§6 PROMPT-N+2 dispatch slot**: paperwork-only story-authoring
  for this row.

---

## Problem Class / Prevention Target

**Defect class (DC-14)**: The friend-game route cannot be exercised
end-to-end by an AI agent without a windowed Bevy client. Existing
automated tests use `MinimalPlugins` + protocol-level driving and do
not exercise the production client's full plugin stack. Symptoms:
`S8-QA-001-W1` (manual two-client GAME_OVER) cannot be closed
without a human operator running the windowed runbook; runtime
divergence between protocol-test green and friend-game-route red
cannot be observed.

**Prevention target**: A new cargo binary (workspace member) at
`tools/two-client-runtime/` (or canonical equivalent under
`tools/` or `examples/`) that:

- Starts the real Lightyear server (in-process or as a child
  process; implementation prompt decides).
- Spawns two production-faithful client instances (using the
  `production_client_app()` factory from `S13-FIXTURE-FACTORY-001`
  if available, or a faithful local equivalent if not).
- Connects both clients to the server over the production WebSocket
  transport.
- Scripts a deterministic friend-game route: lobby create + join,
  class select + confirm (both players), session entry, draft,
  shop, auction, placement, resolution, repeated until GAME_OVER
  (or a configurable max-round cutoff).
- Captures structured logs from server, client A, client B with
  wall-clock UTC timestamps at millisecond precision.
- Produces a final evidence bundle (logs + snapshot dumps) at a
  canonical path under `production/qa/evidence/captures/`.
- Exits 0 if the friend-game route reaches the configured endpoint
  (default: GAME_OVER); non-zero on any unexpected error.

The harness unblocks:

- Story 019 tighter-capture invocation (drag-runtime divergence
  retest with structured log capture).
- `S8-QA-001-W1` closure attempt (manual two-client GAME_OVER
  evidence).
- Future runtime-divergence diagnostic stories.

The harness is **not** an automated CI gate. It is a developer-
invokable driver that produces evidence; CI may invoke it on a
nightly cadence if/when stability allows, but the default invocation
is operator-triggered with a documented command line.

---

## Context

### Existing surface

- **Production server**: `server/src/main.rs` (Lightyear server,
  WebSocket transport, RSM plugin, session plugin, pool plugin,
  auction plugin, acquisition plugin, objective plugin, combat
  plugin).
- **Production client**: `client/src/main.rs` (Bevy 0.18 windowed app
  via Trunk for WASM, native for dev).
- **Existing automated harnesses**:
  - `tests/integration/network/os18b_two_client_objective_hp_visibility_test.rs:46`
    -- two-client protocol-level test for objective HP visibility.
  - `tests/integration/playable_client/full_game_over_route_test.rs:66`
    -- protocol-level full-game-over route test.
- **Human operator runbook**:
  `production/qa/evidence/manual-friend-game-evidence-runbook.md`
  (per PROMPT 803 §4 Lane E).
- **Reconnect / snapshot pipeline**: sound per PROMPT 803 §4 Lane C
  (`server/src/core/session/reconnect.rs:107-121,176,198-233,292-316`).

### Dependencies on other Sprint 13 candidates

- **`S13-FIXTURE-FACTORY-001` (Story 016)**: the harness benefits
  from `production_client_app()` / `production_server_app()` for
  faithful plugin composition. If Story 016 lands first, the
  harness uses it; otherwise the harness inlines a faithful local
  copy with an inline rationale comment + follow-on
  cross-reference.
- **`S13-OBS-TRACING-TARGETS-001` (Story 018)**: the harness's
  structured log capture is partial until tracing targets are
  scoped; the harness still runs and produces evidence, but the
  per-module breakdown is empty.
- **`S13-OBS-WALLCLOCK-TIMESTAMPS-001` (Story 019)**: same -- until
  wall-clock timestamps land in the production subscribers, the
  harness wraps each subprocess's stderr in a shell-style UTC
  prefix.

### GDD / ADR / TR trace

- **GDD**: friend-game route per `design/gdd/game-session-system.md`,
  `design/gdd/round-state-machine.md`, `design/gdd/hand-ui.md`,
  `design/gdd/shop-auction-ui.md`, etc. No GDD change.
- **ADR-002** (Client-Server Authority): the harness scripts C2S
  intents; the clients consume S2C broadcasts; no client-side
  optimism is introduced.
- **ADR-008** (Lightyear Channel Config): the harness uses
  production channel bindings.
- **ADR-011** (Reconnect Snapshot): the harness optionally tests
  the reconnect path by killing a client mid-game and restarting it.
- **TR registry**: no new TR. The harness references existing TRs.

### Engine

- **Engine**: Bevy 0.18 (Rust). The harness uses Bevy 0.18
  `App::update()` ticking loop for the clients (no windowed run);
  the server runs at production tick rate.
- **Lightyear**: 0.26 (Bevy 0.18 compatible). The harness connects
  via the production WebSocket transport.

### Mandatory skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code in the harness.
- **`liv-bevy-lightyear`** -- mandatory for transport configuration
  and protocol-level message scripting.

### Control Manifest Rules (Test Infra / Driver scope)

- Required: The harness is deterministic. Random seeds are passed
  via CLI flag (default to a constant); the server's deterministic
  RNG (`rand_chacha`) is seeded explicitly.
- Required: The harness exits 0 on success and non-zero on any
  unexpected error or timeout.
- Required: The harness produces structured log capture at a
  canonical path; the evidence bundle includes server log, client A
  log, client B log, and a final game-state snapshot.
- Required: The harness uses the production transport (WebSocket);
  no in-process channel shortcut.
- Required: The harness does not introduce optimistic client-side
  authority for any step.
- Forbidden: Modifying production code under `client/`, `server/`,
  `shared/` (the harness lives in `tools/` and consumes the
  production crates as dependencies). The single exception is the
  test-helper export under AC8 (similar to the factory).
- Forbidden: Bypassing the production protocol (e.g., calling
  internal server APIs directly to advance phases).
- Forbidden: Hardcoding test card-data; the harness uses the same
  card pool config as production.
- Forbidden: Modifying Sprint 12 story files or evidence paths.

---

## Story Classification

**Story type**: Integration -- new cargo binary that exercises the
full production stack end-to-end.

This is **NOT** a:

- Pure refactor story.
- Documentation-only story.
- Sprint 12 expansion.
- Closure of `S8-QA-001-W1` by itself (closure is a separate
  follow-on prompt after harness evidence is captured and reviewed).

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Harness exists at the canonical path**:
  `tools/two-client-runtime/Cargo.toml` exists; the workspace
  `Cargo.toml` lists `tools/two-client-runtime` as a member;
  `cargo build --bin two-client-runtime` succeeds.
  *Closed PROMPT 858*: workspace registration verified; `cargo build`
  on `D:\_DEV\cargo-target\ccgs-msvc` succeeds.

- [x] **AC2 -- Harness starts server + two clients**: GIVEN the
  harness binary, WHEN invoked with default flags, THEN it starts
  a Lightyear server bound to a configurable port, spawns two
  production-faithful clients, and both clients connect to the
  server within a configurable timeout (default 5 s). Connection
  evidence is logged.
  *Closed PROMPT 858*: `seed-1-run-1-v2/harness.log` reports
  `both clients handshake complete tick=3 elapsed_ms=266`; second
  run reports `elapsed_ms=259`. Both <300 ms (default 5 s budget).

- [x] **AC3 -- Harness scripts the friend-game route to GAME_OVER**:
  GIVEN the default flags, WHEN the harness runs, THEN it scripts
  lobby create + join, class select + confirm (both players),
  session entry, draft, shop, auction, placement, resolution, and
  next-loop transitions until GAME_OVER is reached OR the
  configurable max-round cutoff is hit (default: 10 rounds).
  GAME_OVER detection is observed from the S2C broadcast, not
  inferred from local state. Reaching GAME_OVER is the canonical
  success endpoint.
  *Closed PROMPT 858 with canonical endpoint*: both
  `seed-1-run-1-v2` and `seed-1-run-2-v2` `final_state.json` report
  `endpoint_reached="game_over"`, `game_over_round=6`,
  `game_over_reason_draw=true`, both clients `received_game_over=true`.
  Detection source: `S2CGameOver` `MessageReceiver` in
  `tools/two-client-runtime/src/route.rs::record_game_over`.

- [x] **AC4 -- Harness uses the production transport**: GIVEN the
  harness diff, WHEN reviewed, THEN the harness uses Lightyear's
  WebSocket transport (same as `server::main`); no in-process
  channel shortcut is used. AC2/AC3 evidence demonstrates the
  WebSocket connection.
  *Closed PROMPT 858*: harness server spawns `WebSocketServerIo`
  via the production `ServerNetworkPlugin` (SERVER_PORT env var
  routes the bind to the harness-reserved port); both clients
  spawn `WebSocketClientIo::from_url(ClientConfig::default(), url)`.
  `seed-1-run-1-v2/server.log` line 3 logs
  `Server WebSocket starting at 0.0.0.0:57043`.

- [x] **AC5 -- Structured log capture lands at the canonical path**:
  GIVEN a successful run, WHEN
  `production/qa/evidence/captures/sprint-13-two-client-runtime/`
  is inspected, THEN the run's evidence bundle exists in a dated
  subdirectory (per the
  `manual-friend-game-evidence-YYYY-MM-DD/` precedent) containing:
  - `server.log` (server stderr capture)
  - `client_a.log` (client A stderr capture)
  - `client_b.log` (client B stderr capture)
  - `final_state.json` (final game-state snapshot dump; format
    deferred to implementation prompt)
  - `harness.log` (harness driver log)
  *Closed PROMPT 858*: all 5 files present in canonical
  `seed-1-run-1-v2/`, `seed-1-run-2-v2/`, and dated `2026-05-14/`
  subdirectories. Per-role routing via process-global `AtomicU8`
  consulted by the `MakeWriter` (sequential ticks make atomic safe).

- [x] **AC6 -- Logs have wall-clock UTC timestamps at ms
  precision**: GIVEN the log files from AC5, WHEN any line is
  inspected, THEN every line carries an ISO-8601 UTC timestamp at
  millisecond precision. If `S13-OBS-WALLCLOCK-TIMESTAMPS-001`
  has landed, the timestamps come from the production subscribers
  natively; otherwise the harness wraps each subprocess's stderr in
  a UTC-prefixing shim.
  *Closed PROMPT 858*: `tracing_subscriber::fmt::time::UtcTime::rfc_3339()`
  installed in `tools/two-client-runtime/src/logging.rs::init_role_subscriber`.
  Spot-check across all 4 files: every line carries an ISO-8601
  UTC timestamp at 7-fractional-digit precision (RFC 3339).
  S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837/842) is already in
  production; the harness uses the same timer.

- [x] **AC7 -- Determinism**: GIVEN two runs of the harness with
  the same `--seed N` flag, WHEN the resulting `final_state.json`
  files are diffed, THEN they are identical (modulo timestamps).
  The harness must be deterministic so that downstream evidence
  consumers can compare across runs.
  *Closed PROMPT 858*: `diff <(jq .routes_observed seed-1-run-1-v2/final_state.json) <(jq .routes_observed seed-1-run-2-v2/final_state.json)`
  emits no output (byte-identical). Only the top-level `server_port`
  and `websocket_bind_addr` differ between runs because the default
  invocation reserves an ephemeral port; pass `--port N` for
  top-level identity too.

- [x] **AC8 -- Production code touched minimally**: GIVEN the diff
  in `client/src/`, `server/src/`, `shared/src/`, WHEN inspected,
  THEN any new `pub` exports are scope-capped to harness-helper
  visibility (e.g., a `pub fn build_production_client_app()` if
  not already exposed by Story 016) with an inline rationale
  comment cross-referencing this story.
  *Closed PROMPT 858*: zero diffs under `client/src/`, `server/src/`,
  `shared/src/`. The harness consumes already-`pub` items
  (`server::network::ServerNetworkPlugin` + dependency plugins;
  `client::network::register_lightyear_protocol`). No new `pub`
  exports added.

- [x] **AC9 -- No optimistic client-side authority introduced**:
  GIVEN the harness diff, WHEN reviewed for any client-side
  mutation of authoritative state outside the shared phase sink,
  snapshot drainers, and S2C consumers, THEN no such mutation is
  present. ADR-002 binding. *Evidence*: text search for "no
  optimistic" in the evidence document.
  *Closed PROMPT 858*: phrase "no optimistic client-side authority"
  preserved verbatim in evidence document. `route.rs` client
  systems only emit `MessageSender::send` (real C2S intents) and
  store `MessageReceiver::receive` observations into shared
  `Arc<Atomic*>` flags. No `*View` resource or authoritative-mirror
  resource is mutated by the harness.

- [x] **AC10 -- Documented invocation in `docs/setup/`**:
  `docs/setup/two-client-runtime-harness.md` (NEW) records the
  canonical invocation, supported flags, expected evidence paths,
  and known limitations. Cross-references PROMPT 803 §3 DC-14.
  *Closed PROMPT 858*: `docs/setup/two-client-runtime-harness.md`
  authored with canonical PowerShell invocation, all CLI flags,
  evidence layout, determinism guarantees, architecture notes,
  AC12 binding restatement, and PROMPT 803 §3 DC-14 cross-link.

- [x] **AC11 -- Sprint 12 disposition preserved**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`, and
  `production/qa/qa-plan-sprint-12.md` are diffed, THEN none of
  them are modified under this story.
  *Closed PROMPT 858*: `git diff --stat origin/main...HEAD` shows
  zero diffs under those four paths.

- [x] **AC12 -- `S8-QA-001-W1` is NOT auto-closed**: GIVEN the
  harness's evidence bundle from AC5, WHEN the implementation
  prompt completes, THEN `S8-QA-001-W1` is NOT auto-closed.
  Closure (if any) is recorded under a separate `/story-done`
  prompt with explicit QA-lead sign-off, citing the harness
  evidence + a producer decision on whether the harness's evidence
  satisfies the manual-two-client GAME_OVER gap or whether a
  human operator runbook execution is still required.
  *Closed PROMPT 858*: NO modification to any S8-QA-001-W1 tracker
  field, no modification to
  `production/qa/evidence/manual-friend-game-evidence-runbook.md`,
  no modification to `production/sprint-status.yaml`, no
  modification to any `production/qa/qa-plan-sprint-*.md`. The
  harness evidence bundle is produced and preserved; closure
  verdict is reserved for a separate `/story-done` prompt with
  QA-lead sign-off.

- [x] **AC13 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-13-two-client-runtime-evidence.md`
  (NEW; populated by the implementation prompt). Records harness
  invocation, log-bundle path, AC2-AC7 evidence, no-claim
  restatement, cross-link to PROMPT 803 §3 DC-14 + §4 Lane E +
  `S8-QA-001-W1`.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `tools/two-client-runtime/Cargo.toml` | NEW. Workspace member with `[dependencies] server = { path = "../../server" }`, `client = { path = "../../client" }`, `shared = { path = "../../shared" }`. |
| `tools/two-client-runtime/src/main.rs` | NEW. Harness driver: starts server, spawns two clients, scripts friend-game route, captures structured logs, dumps final state. |
| `tools/two-client-runtime/src/scripted_route.rs` | NEW. Per-phase scripted intents (lobby, draft, shop, auction, placement, resolution). |
| `tools/two-client-runtime/src/log_capture.rs` | NEW. UTC-timestamp shim + per-subprocess log routing. |
| `Cargo.toml` (workspace) | Updated to register `tools/two-client-runtime` as a member. |
| `client/src/lib.rs` OR `server/src/lib.rs` (OPTIONAL) | OPTIONAL: extract production-app build into a `pub fn` reusable by the harness. Scope-capped to harness-helper export. |
| `docs/setup/two-client-runtime-harness.md` | NEW. Canonical invocation + flag documentation. |
| `production/qa/evidence/captures/sprint-13-two-client-runtime/<dated-subdir>/` | NEW directory pattern. Per-run evidence bundle. |
| `production/qa/evidence/sprint-13-two-client-runtime-evidence.md` | NEW evidence document per AC13. |
| This story file | Status updates per /story-readiness or /story-done if/when Sprint 13 activates. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for all `.rs` code in the harness.
- **`liv-bevy-lightyear`** -- mandatory for the transport
  configuration, the client/server connection setup, and the
  scripted protocol-level intents.

---

## Evidence Path

`production/qa/evidence/sprint-13-two-client-runtime-evidence.md`
(NEW; populated by the implementation prompt).

Run-evidence bundles land under
`production/qa/evidence/captures/sprint-13-two-client-runtime/<dated-subdir>/`.

**Required evidence content** (deferred to implementation prompt):

- Harness invocation command line.
- AC2 connection evidence (timestamps of both clients connecting
  to the server).
- AC3 GAME_OVER evidence (S2C broadcast trace + final round number).
- AC5 evidence-bundle directory listing.
- AC6 timestamp-format spot-check (random 5 lines from each log
  file with ISO-8601 UTC ms precision verified).
- AC7 determinism evidence (two runs with same seed; diff output
  showing identity modulo timestamps).
- No-claim restatement (verbatim from "Status / No-Claim Banner"
  including "no optimistic client-side authority" and the
  `S8-QA-001-W1` is-NOT-auto-closed clause).
- Cross-link to PROMPT 803 §3 DC-14, §4 Lane E, and `S8-QA-001-W1`.

---

## Regression Commands Expected

For the implementation prompt:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo build --bin two-client-runtime`
- `cargo run --bin two-client-runtime -- --seed 1 --max-rounds 10`
- `cargo run --bin two-client-runtime -- --seed 1 --max-rounds 10`
  (second run for AC7 determinism)
- `diff -u <run1-final-state> <run2-final-state>` (expecting
  identity modulo timestamps)
- `cargo test --workspace --tests --no-fail-fast` (existing tests
  must continue to pass)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Out of Scope

- **Closing `S8-QA-001-W1`**. AC12 forbids auto-closure. Closure
  is a separate `/story-done` prompt with QA-lead sign-off.
- **CI integration**. The harness is operator-invokable; a nightly
  CI cadence is scoped to a Sprint 14 follow-on if/when stability
  allows.
- **Automating the existing human operator runbook**
  (`production/qa/evidence/manual-friend-game-evidence-runbook.md`).
  Convertion is scoped to a Sprint 14 Nice-to-Have row
  (`S13-MANUAL-RUNBOOK-AUTOMATION-001` per PROMPT 803 §5 Nice).
- **Mid-game disconnect / reconnect scripting**. The base harness
  scripts the happy path to GAME_OVER; reconnect-path testing is a
  follow-on enhancement.
- **Multi-platform CI**. The harness is Linux/macOS/Windows
  developer-machine invokable; CI matrix expansion is out of scope.
- **Modifying production protocol shapes**. ADR-008 + ADR-003
  binding.
- **Sprint 13 activation**. No `production/sprint-status.yaml` /
  `production/stage.txt` modification under this story.
- **No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run** under this
  authoring prompt.
- **No closure of `QA-COND-0005`, `QA-COND-0006`, or any carried
  Sprint condition.
- **No claim of public release readiness, release-candidate
  readiness, full playable-client manual QA, full game completion,
  broad Standard-tier accessibility completion, playtest /
  fun-hypothesis validation, or final-art / asset-production
  completion.**

---

## Dependency Notes Against Sprint 12 Active Scope

- **Sprint 12 close-out gate**: this story's implementation MUST
  run after Sprint 12 close-out. The harness exercises the
  friend-game route end-to-end; running it before Sprint 12
  closes risks observing Sprint 12 in-progress behaviour.
- **Disjoint files from Sprint 12 Must Have rows**: the harness
  lives entirely under `tools/two-client-runtime/`; no Sprint 12
  Must Have row touches `tools/`. The optional `client::lib.rs` /
  `server::lib.rs` extraction is the only potential conflict and
  is scope-capped to a harness-helper export.
- **Coordinate with `S13-FIXTURE-FACTORY-001` (Story 016)**: if
  the factory lands first, the harness uses
  `production_client_app()` / `production_server_app()`; if not,
  the harness inlines a faithful local equivalent.
- **Coordinate with `S13-OBS-TRACING-TARGETS-001` (Story 018) and
  `S13-OBS-WALLCLOCK-TIMESTAMPS-001` (Story 019)**: timestamp and
  per-target log scope are partial until those stories land; the
  harness's UTC-prefixing shim is the fallback.
- **No shared-status writer overlap**: `production/sprint-status.yaml`
  is not touched by this story.

---

## Implementation Notes

This story is **draft** at authoring time. Activation requires (in
order):

1. Sprint 12 reaches close-out (separate prompt).
2. Sprint 13 is planned via `/sprint-plan sprint-13` (separate prompt).
3. This story passes `/story-readiness` (separate prompt).
4. Sprint 13 `/qa-plan sprint` is authored (separate prompt).
5. `/dev-story story-017-two-client-runtime-harness.md` is
   dispatched (separate prompt).

Expected implementation flow:

1. **Wave 1 -- Workspace scaffold**: New `tools/two-client-runtime/`
   cargo bin; workspace `Cargo.toml` updated; basic `cargo build`
   passes with a stub `main()`.
2. **Wave 2 -- Server + client startup**: Harness starts the
   Lightyear server, spawns two clients, both connect. AC2 met.
3. **Wave 3 -- Scripted route**: Per-phase scripted intents
   (lobby, draft, shop, auction, placement, resolution).
4. **Wave 4 -- Log capture**: Structured log capture with UTC
   timestamps. AC5 + AC6 met.
5. **Wave 5 -- Determinism pass**: AC7 verified.
6. **Wave 6 -- Docs + evidence**: `docs/setup/two-client-runtime-harness.md`
   + evidence doc.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Bevy `App::update()` ticking loop diverges from windowed run timing | Medium | Medium | Harness emulates production tick rate (60 Hz) and exit conditions; AC7 determinism catches drift early. |
| WebSocket binding flaps in CI | Medium | Low-Medium | Configurable port + retry-with-backoff; default to a high port outside privileged range. |
| Server-startup race with client connect | Medium | Medium | Connect retry-with-backoff up to AC2 timeout; harness fails loudly if both clients can't connect within the timeout. |
| Card pool / class config drift makes the scripted route non-deterministic | Low | High | AC7 determinism with `--seed N`; harness uses production config files unchanged. |
| Harness accidentally introduces optimistic client-side authority via direct ECS mutation | Low | High | AC9 + ADR-002 reviewer check; harness MUST drive via C2S intents only. |
| Sprint 12 active rows expose runtime regressions that the harness surfaces unexpectedly | Medium | Low (informational) | The harness is *meant* to surface runtime divergence; document any new divergences as follow-on stories. |
| `S8-QA-001-W1` is accidentally auto-closed by AC overreach | Low | High | AC12 explicit forbid-auto-closure; closure is a separate prompt with QA-lead sign-off. |
| Sprint 13 activation does not happen before implementation dispatch | Low | High | Activation is a separate prompt gate. |

---

## Verification (orchestrator-side, before worker dispatch)

- `production/sprint-status.yaml` `sprint:` field reads `13` after
  Sprint 13 activation; Sprint 12 close-out has landed.
- `production/stage.txt` reads `Polish` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved.
- `git diff --check` and `git diff --cached --check` pass before any
  commit.

---

## Authoring Trail

- 2026-05-14 -- PROMPT 804 -- Story file authored as a Sprint 13
  candidate for the Non-Interactive Scripted Two-Client Runtime
  Harness per PROMPT 803 §3 DC-14 / §4 Lane E / §5 Must row 6.
  Sprint 12 is `active` (PROMPT 798) and is not modified by this
  authoring run. No code changes, no smoke / gate / QA /
  `/dev-story` / `/story-done` / `/story-readiness` / `/qa-plan` run.
  Source-of-truth at authoring: `origin/main@b5eef0d`. Worker
  branch: `work/s13-runtime-hardening-story-authoring`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-runtime-hardening-story-authoring`.

- 2026-05-14 -- PROMPT 858 -- `/dev-story` implementation. New cargo
  bin at `tools/two-client-runtime/` (workspace member) with
  `src/main.rs` (CLI + orchestration), `src/route.rs` (scripted
  friend-game route), and `src/logging.rs` (per-role log routing
  via process-global `AtomicU8` consulted by a custom `MakeWriter`).
  Harness builds + runs cleanly. Two canonical seed-1 runs both
  reach the canonical `S2CGameOver` endpoint at round 6 with
  byte-identical `routes_observed` `final_state.json` blocks
  (AC7 PASS). Per-role log files (`server.log`, `client_a.log`,
  `client_b.log`, `harness.log`, `final_state.json`) land at
  `production/qa/evidence/captures/sprint-13-two-client-runtime/`
  in `seed-1-run-1-v2/`, `seed-1-run-2-v2/`, dated `2026-05-14/`,
  and three exploratory dirs preserved per orchestrator decision.
  AC1-AC13 all PASS; AC12 forbid-auto-closure preserved
  (`S8-QA-001-W1` NOT modified). Zero diffs under `client/src/`,
  `server/src/`, `shared/src/`, `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/stage.txt`,
  `production/qa/qa-plan-sprint-12.md`. Source-of-truth at start:
  `origin/main@9b65439`. Worker branch:
  `work/s13-two-client-runtime-harness`. Worktree:
  `D:\_DEV\claude-code-game-studios-worktrees\s13-two-client-runtime-harness`.
  Evidence doc:
  `production/qa/evidence/sprint-13-two-client-runtime-evidence.md`.
  Setup doc: `docs/setup/two-client-runtime-harness.md`.

- 2026-05-14 -- PROMPT 870 -- `/integrate` merge of
  `work/s13-two-client-runtime-harness` worker tip `cb4454a` onto
  `origin/main`. Final integration commit
  `3cf5e419a8a0c055de0acb84991515f0f0ef280e` is a real merge commit
  with PROMPT 866 closure `6cdab34` as first-parent and PROMPT 858
  worker `cb4454a` as second-parent. Single merge with `--no-ff` over
  the prior `origin/main@6cdab34`; no rebase, no conflict. Files
  changed match worker scope exactly: 49 files / +4211 / -21
  (`Cargo.lock` +17, workspace `Cargo.toml` +1/-1, new
  `tools/two-client-runtime/{Cargo.toml,src/main.rs,src/route.rs,src/logging.rs}`,
  `docs/setup/two-client-runtime-harness.md` NEW, story file inline
  AC annotations, `production/qa/evidence/sprint-13-two-client-runtime-evidence.md`
  NEW 312 lines, eight evidence-bundle subdirectories under
  `production/qa/evidence/captures/sprint-13-two-client-runtime/`).
  Zero files under `client/src/`, `server/src/`, `shared/src/`,
  `tests/`, `production/sprint-status.yaml`, `production/sprints/sprint-12.md`,
  `production/sprints/sprint-13.md`, `production/stage.txt`,
  `production/qa/qa-plan-sprint-12.md`, `production/qa/qa-plan-sprint-13.md`,
  `production/gate-checks/*`. Fast-forward push `6cdab34..3cf5e41` to
  `origin/main`, no force. PROMPT 870 did NOT run `/story-done`,
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or
  `/qa-plan`. AC12 forbid-auto-closure preserved: `S8-QA-001-W1` is
  NOT modified by PROMPT 870.

- 2026-05-14 -- PROMPT 871 -- `/story-done` paperwork closure. Verified
  HEAD fast-forwarded to `origin/main == 3cf5e41` (PROMPT 870
  integration merge; PROMPT 858 worker `cb4454a` reachable as the
  merge's second-parent). AC1-AC13 all verified PASS against integrated
  evidence on `origin/main@3cf5e41` (see
  `production/qa/evidence/sprint-13-two-client-runtime-evidence.md`
  AC2-AC13 sections + worker commit `cb4454a` `git show --stat`). AC12
  forbid-auto-closure independently re-checked: no modification to any
  `S8-QA-001-W1` tracker field, no modification to
  `production/qa/evidence/manual-friend-game-evidence-runbook.md`, no
  modification to any `production/qa/qa-plan-sprint-*.md`. PROMPT 871
  paperwork-only run at root checkout (serialized shared-status writer
  per 2026-05-13 override; matches PROMPT 868 / 865 / 856 / 854 / 851
  / 844 / 843 / 840 / 835 paperwork pattern). Files changed by PROMPT
  871: this story file (Status header flipped `Implemented` -> `Done`
  with PROMPT 871 closure context; this Closure Trail entry appended);
  `production/sprint-status.yaml` (Sprint 13 Must Have row
  `S13-TWO-CLIENT-RUNTIME-HARNESS-001` flipped `status: ready -> done`
  with `completed: 2026-05-14`, `worker_prompt: 858`, `worker_commit:
  cb4454a625421229f5adfb64d5036fbcff3fa940`, `integration_prompt: 870`,
  `integration_commit: 3cf5e419a8a0c055de0acb84991515f0f0ef280e`,
  `story_done_prompt: 871`, `test_evidence` + `acceptance_evidence`
  cross-links, plus 3 new notes lines; top-level `updated:` refreshed;
  `sprint_13_story_done:` block extended with PROMPT 871 entry as a
  sibling to the prior PROMPT 833 / 840 / 843 / 844 / 850 / 851 / 854
  / 856 / 865 / 868 entries -- eleventh `/story-done` block of
  Sprint 13); `production/session-state/active.md` (PROMPT 871 banner
  prepended above PROMPT 868 banner); `production/session-state/codex-orchestrator-state.md`
  (PROMPT 871 section prepended above PROMPT 868 section). Zero file
  under `client/`, `server/`, `shared/`, `tests/`,
  `tools/two-client-runtime/`, `production/stage.txt`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-12.md`,
  `production/qa/qa-plan-sprint-13.md`, `production/qa/qa-plan-sprint-12.md`,
  `production/qa/evidence/*` (evidence doc already on origin/main via
  PROMPT 870), `production/gate-checks/*` modified. No cargo invoked
  by PROMPT 871. No `/dev-story`, `/story-readiness`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan` invoked.
  AC12 forbid-auto-closure preserved: `S8-QA-001-W1` remains OPEN.
  Sprint 13 disposition UNCHANGED (`active`; NOT closed-out by PROMPT
  871). Stage UNCHANGED `Polish`. PROMPT 761 Polish->Release FAIL
  preserved. Sprint 13 progress after PROMPT 871: **6 of 6 Must Have
  done** (Sprint 13 Must Have track COMPLETE); 3 of 6 Should Have done;
  3 of 7 Nice to Have done; total **12 of 19** rows closed.

### Conditions carried forward unchanged

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap remains
  OPEN. AC12 forbid-auto-closure explicitly preserved by PROMPT 871.
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk
  (friend-game scope only).
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  `PAW-002..PAW-006`.
- PROMPT 683-era runtime divergence question preserved unchanged.
- PROMPT 761 Polish->Release gate-check FAIL preserved.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed.
- `TQ-S12-C1..C7` (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 10 (`closed-with-conditions`), Sprint 11
  (`closed-with-conditions`), Sprint 12 (`closed-with-conditions` per
  PROMPT 817) dispositions preserved unchanged.

### Explicitly NOT claimed by PROMPT 871

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from Polish to Release
- underlying drag-runtime bug fix
- full UI clean-pass repair
- Sprint 13 close-out (Sprint 13 remains `active`; Must Have track
  complete but 3 of 6 Should Have + 4 of 7 Nice to Have rows remain
  open)
- harness CI integration (operator-invokable only; nightly cadence
  scoped to Sprint 14 follow-on)
- harness mid-game disconnect/reconnect scripting (happy-path GAME_OVER
  only)
