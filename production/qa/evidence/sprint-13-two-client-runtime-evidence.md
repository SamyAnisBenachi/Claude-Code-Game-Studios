# Sprint 13 — Two-Client Runtime Harness Evidence

> Story: `S13-TWO-CLIENT-RUNTIME-HARNESS-001`
> Implementation prompt: PROMPT 858 — 2026-05-14
> Worker branch: `work/s13-two-client-runtime-harness`
> Source-of-truth at start: `origin/main@9b65439`
> Evidence bundles: `production/qa/evidence/captures/sprint-13-two-client-runtime/`

## Status / No-Claim Banner (verbatim per Story 017)

> This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
> activated by PROMPT 804. Sprint 12 remains the active sprint
> (`status: active`) and must not be changed by this authoring run.

PROMPT 858 (this implementation run) does **NOT**:

- Close `S8-QA-001-W1`. AC12 explicitly forbids auto-closure. Closure (if
  any) is a separate `/story-done` prompt with QA-lead sign-off.
- Modify any Sprint 12 file (`production/sprints/sprint-12.md`,
  `production/qa/qa-plan-sprint-12.md`).
- Modify `production/stage.txt` (remains `Polish`).
- Retry the PROMPT 761 Polish→Release gate-check.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan`.
- Modify production code under `client/`, `server/`, or `shared/`.

**No optimistic client-side authority is introduced or proposed by this
story.** The harness scripts the friend-game route via real C2S intents
against the real Lightyear server; the clients are read-only views over
server-authoritative state. ADR-002 + ADR-008 + ADR-011 + ADR-012 binding
preserved.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 (`closed-with-conditions`) and Sprint 11 (`closed-with-conditions`)
dispositions are unchanged. PROMPT 761 Polish→Release gate-check FAIL
evidence is preserved.

## Cross-references

- PROMPT 803 §3 DC-14 (Manual two-client route coverage gap).
- PROMPT 803 §4 Lane E (Manual two-client route).
- PROMPT 803 §5 Must row 6 (`S13-TWO-CLIENT-RUNTIME-HARNESS-001`).
- `S8-QA-001-W1` (OPEN — not closed by this story).
- `S13-FIXTURE-FACTORY-001` / Story 016 (already landed PROMPT 853/854 —
  the harness re-uses production server / client crate exports rather
  than the factory's test-only `MinimalPlugins` substitution, because
  the harness needs the real `ServerNetworkPlugin` + WebSocket transport).
- `S13-OBS-WALLCLOCK-TIMESTAMPS-001` / Story 019 (PROMPT 837/842 —
  wall-clock UTC timestamps in production subscribers; the harness uses
  the same `UtcTime::rfc_3339()` timer per AC6).

## Canonical invocation

```text
cargo run -p two-client-runtime --bin two-client-runtime -- \
    --seed 1 --max-rounds 10
```

Default evidence path:
`production/qa/evidence/captures/sprint-13-two-client-runtime/<UTC-date>/`.

Full docs: `docs/setup/two-client-runtime-harness.md`.

## AC2 — Connect evidence (both clients handshake within timeout)

From `seed-1-run-1-v2/harness.log`:

```text
2026-05-14T18:09:30.4824934Z  INFO harness: ephemeral websocket port reserved port=57043 url=ws://127.0.0.1:57043
2026-05-14T18:09:31.0980591Z  INFO harness: both clients handshake complete tick=3 elapsed_ms=266
```

From `seed-1-run-2-v2/harness.log`:

```text
2026-05-14T18:10:03.092787Z   INFO harness: both clients handshake complete tick=3 elapsed_ms=259
```

Both runs complete the dual-client `S2CHandshake` exchange in <300 ms
(default budget 5 s; harness invocation used 10 s for headroom). AC2 PASS.

## AC3 — GAME_OVER endpoint reached

From `seed-1-run-1-v2/final_state.json`:

```json
"placement_phase_count": 5,
"draft_shop_phase_count": 4,
"resolution_phase_count": 5,
"auction_phase_count": 2,
"host_received_game_over": true,
"joiner_received_game_over": true,
"game_over_round": 6,
"game_over_reason_draw": true,
"endpoint_reached": "game_over",
"rounds_observed": 5
```

The harness observed the canonical `S2CGameOver` broadcast on both clients
at round 6 (`GameOverReason::Draw`). The route progressed through:

- Lobby create + join.
- Class select + confirm (host=Iop, joiner=Cra).
- DRAFT_INITIAL with both players drafting one card.
- 5 × Placement phases (empty placements).
- 5 × Resolution phases.
- 4 × DraftShop phases.
- 2 × DraftAuction phases.
- Terminal GAME_OVER on round 6 (draw).

Detection source: the S2C `S2CGameOver` message receiver in
`tools/two-client-runtime/src/route.rs::record_game_over`, NOT inferred
from local client state. AC3 PASS via the canonical (GAME_OVER) endpoint.

`seed-1-run-2-v2` matches: `endpoint_reached="game_over"`, `round=6`,
both `host_received_game_over` and `joiner_received_game_over` true.

## AC4 — Production WebSocket transport

From `seed-1-run-1-v2/server.log`:

```text
2026-05-14T18:09:30.4939514Z  INFO lightyear_websocket::server: Server WebSocket starting at 0.0.0.0:57043
```

From `seed-1-run-1-v2/client_a.log`:

```text
2026-05-14T18:09:30.8257113Z  INFO harness::client_a: client A app build start url=ws://127.0.0.1:57043
```

The harness server uses the production `ServerNetworkPlugin` (which spawns
`WebSocketServerIo`) and both clients connect via `WebSocketClientIo` from
`lightyear::prelude::client::*`. No in-process channel shortcut is used.
ADR-008 binding preserved. AC4 PASS.

## AC5 — Evidence bundle layout

Canonical seed-1-run-1 evidence:

```text
production/qa/evidence/captures/sprint-13-two-client-runtime/seed-1-run-1-v2/
├── client_a.log    (3279 B,  24 lines)
├── client_b.log    (3239 B,  23 lines)
├── final_state.json (1086 B, AC2/AC3/AC7 facts)
├── harness.log     (1010 B,   6 lines)
└── server.log     (29782 B, 200 lines)
```

A second canonical run lives at `seed-1-run-2-v2/`. A default-flag run
lands at `2026-05-14/` (the dated subdir precedent
`manual-friend-game-evidence-YYYY-MM-DD/`). Three exploratory dirs
(`run-1`, `run-2`, `atomic-test`, `seed-1-run-1`, `seed-1-run-2`) preserved
on-disk per orchestrator decision (no cleanup). AC5 PASS.

## AC6 — Wall-clock UTC timestamps at ms precision

Spot-check (timestamps from `seed-1-run-1-v2/`):

| File | Sample line |
|------|-------------|
| `server.log` | `2026-05-14T18:09:30.4932433Z  INFO harness::server: server app built` |
| `client_a.log` | `2026-05-14T18:09:30.8257113Z  INFO harness::client_a: client A app build start url=ws://127.0.0.1:57043` |
| `client_b.log` | `2026-05-14T18:09:30.8290693Z  INFO harness::client_b: client B app build start url=ws://127.0.0.1:57043` |
| `harness.log` | `2026-05-14T18:09:30.4810967Z  INFO harness: harness boot ...` |

Every line carries an ISO-8601 UTC timestamp at millisecond+ precision
(the format is RFC 3339 with 7-digit fractional seconds). Source:
`tracing_subscriber::fmt::time::UtcTime::rfc_3339()` installed in
`tools/two-client-runtime/src/logging.rs::init_role_subscriber`. AC6 PASS.

## AC7 — Determinism

Two runs with `--seed 1 --max-rounds 10` were diffed:

```text
diff <(jq .routes_observed seed-1-run-1-v2/final_state.json) \
     <(jq .routes_observed seed-1-run-2-v2/final_state.json)
# (empty -- byte-identical)
```

The `routes_observed` block is byte-identical between both runs. Only the
top-level `server_port` and `websocket_bind_addr` differ because the
default invocation reserves an ephemeral port; pass `--port N` for full
identity. AC7 PASS.

## AC8 — Production code touched minimally

`git diff --stat origin/main...HEAD` is restricted to:

- `Cargo.toml` (workspace member registration only).
- `tools/two-client-runtime/Cargo.toml` (NEW).
- `tools/two-client-runtime/src/main.rs` (NEW).
- `tools/two-client-runtime/src/route.rs` (NEW).
- `tools/two-client-runtime/src/logging.rs` (NEW).
- `docs/setup/two-client-runtime-harness.md` (NEW).
- `production/epics/playable-client/story-017-two-client-runtime-harness.md`
  (AC checkbox annotations only).
- `production/qa/evidence/sprint-13-two-client-runtime-evidence.md` (NEW
  -- this file).
- `production/qa/evidence/captures/sprint-13-two-client-runtime/*` (NEW
  evidence directories).

Zero modifications under `client/src/`, `server/src/`, or `shared/src/`.
The harness consumes server and client crates as workspace dependencies
via their existing `pub` exports (`server::network::ServerNetworkPlugin`,
`client::network::register_lightyear_protocol`). AC8 PASS.

## AC9 — No optimistic client-side authority

Restated verbatim: **no optimistic client-side authority is introduced or
proposed by this story.** The harness's client systems live entirely in
`tools/two-client-runtime/src/route.rs`; every system either:

1. Reads a Lightyear `MessageReceiver<S2C*>` and stores the observed facts
   in shared `Arc<Atomic*>` flags. No client mirror or authoritative
   resource is mutated.
2. Writes a Lightyear `MessageSender<C2S*>` to emit a real client → server
   intent. The server is the sole authority for state transitions.

No `*View` resource is mutated by the harness. No `bevy::ecs::Commands`
spawn is performed outside the client connection entity. AC9 PASS.

## AC10 — Documented invocation

`docs/setup/two-client-runtime-harness.md` (NEW) records:

- Canonical PowerShell invocation with the Cargo policy env vars.
- All supported CLI flags + defaults.
- Evidence bundle layout (AC5 binding).
- Determinism guarantees (AC7 binding).
- Architecture notes (in-process tick loop, per-role log routing, seeded
  RNG factory).
- Known limitations + Sprint 14 follow-on scope.
- AC12 binding restatement.
- ADR-002 no-optimistic-client-authority restatement.

Cross-link to PROMPT 803 §3 DC-14 is preserved in the doc. AC10 PASS.

## AC11 — Sprint 12 disposition preserved

`git diff --stat origin/main...HEAD` shows zero changes under:

- `production/sprint-status.yaml`
- `production/sprints/sprint-12.md`
- `production/stage.txt`
- `production/qa/qa-plan-sprint-12.md`

AC11 PASS.

## AC12 — `S8-QA-001-W1` is NOT auto-closed

The harness produces the automated leg of the manual two-client GAME_OVER
evidence (this evidence document + `seed-1-run-1-v2/` + `seed-1-run-2-v2/`
log bundles), but does **NOT**:

- Update any `S8-QA-001-W1` status field.
- Modify `production/qa/qa-plan-sprint-*.md` carry-conditions.
- Modify the manual runbook at
  `production/qa/evidence/manual-friend-game-evidence-runbook.md`.
- Update `production/sprint-status.yaml`.

Closure (if any) is a separate `/story-done` prompt that must:

1. Cite the evidence path above.
2. Record a producer decision on whether the harness's automated
   evidence satisfies the manual two-client GAME_OVER gap or whether a
   human operator runbook execution is still required.
3. Carry explicit QA-lead sign-off.

AC12 PASS — `S8-QA-001-W1` remains OPEN.

## AC13 — Evidence document slot

This file IS the AC13 evidence document slot. AC13 PASS.

## Regression commands run

- `cargo fmt --all -- --check` -- PASS.
- `cargo check -p two-client-runtime --bin two-client-runtime` -- PASS.
- `cargo build -p two-client-runtime --bin two-client-runtime` -- PASS.
- Two harness runs with `--seed 1 --max-rounds 10` -- both exit 0,
  reach GAME_OVER, byte-identical `routes_observed`.
- `git diff --check origin/main...HEAD` -- PASS.

Full-workspace `cargo test --workspace --tests --no-fail-fast` was NOT
run per Sprint 13 QA-plan no-full-workspace-tests-by-default policy and
the story's Cargo policy directive (story-prescribed targeted checks
only). The harness binary itself is the AC's targeted check.

## AC1-AC13 dispatch summary

| AC | Verdict | Evidence link |
|----|---------|---------------|
| AC1 -- harness exists at canonical path | PASS | `tools/two-client-runtime/Cargo.toml` + `cargo build` log |
| AC2 -- server + 2 clients, both connect | PASS | `seed-1-run-1-v2/harness.log` line 4 (`elapsed_ms=266`) |
| AC3 -- friend-game route to GAME_OVER | PASS | `seed-1-run-1-v2/final_state.json` `endpoint_reached="game_over"` round 6 |
| AC4 -- production WebSocket transport | PASS | `seed-1-run-1-v2/server.log` Lightyear websocket start line |
| AC5 -- structured log capture canonical path | PASS | 5 files per dated subdir under `production/qa/evidence/captures/sprint-13-two-client-runtime/` |
| AC6 -- ISO-8601 UTC ms timestamps | PASS | Every log line shows `YYYY-MM-DDTHH:MM:SS.fffZ` |
| AC7 -- determinism | PASS | `diff` of `routes_observed` between two `--seed 1` runs is empty |
| AC8 -- production code touched minimally | PASS | `git diff --stat` shows zero changes under `client/`, `server/`, `shared/` |
| AC9 -- no optimistic client-side authority | PASS | Phrase preserved verbatim in this document; route.rs binds to ADR-002 |
| AC10 -- documented invocation | PASS | `docs/setup/two-client-runtime-harness.md` (NEW) |
| AC11 -- Sprint 12 disposition preserved | PASS | No diff under `production/sprint-status.yaml`, `production/sprints/sprint-12.md`, `production/stage.txt`, `production/qa/qa-plan-sprint-12.md` |
| AC12 -- `S8-QA-001-W1` is NOT auto-closed | PASS | No tracker file modified; closure deferred to separate `/story-done` |
| AC13 -- evidence document slot | PASS | This file |
