# Autoplay Harness — Architecture (First Slice)

Status: First slice landed by PROMPT 1595 (`work/bevy-autoplay-bootstrap-1595`).
Skill: see `skills/ccgs-autoplay/SKILL.md` for the project-local runbook.

This document is the architecture/policy spec for the in-repo autoplay harness.
It is read alongside the external `liv-bevy-autoplay-bootstrap` skill and the
project-local `ccgs-autoplay` skill. Future autoplay work updates this file
**before** code.

## What the autoplay harness is

A **dev-only** automation surface inside the client binary that lets an
external driver process (a Python script, a coding agent, a future dashboard,
or a QA recipe) operate the running game *as if it were a player*:

- query the client's current observable state (route, phase, window size,
  cursor, key/mouse state, snapshot counter, etc.);
- inject low-level input frames (key presses/releases, mouse button
  presses/releases, cursor screen position);
- clear all injected input;
- request a screenshot of the primary window;
- write all of the above to a durable artifact directory so a later reviewer
  can inspect what the driver did.

It is **not** a gameplay debug surface. See "Hard invariants" below.

## What it is NOT

Hard invariants — enforced by code review on every autoplay change:

- **No gameplay mutation.** The autoplay surface MUST NOT expose any of the
  following, ever:
  - `kill`, `give_xp`, `set_health`, `set_gold`, `teleport`, `select_card`,
    `advance_phase`, `force_resolution`, `set_lane_unit`, `cancel_session`,
    or any other semantic gameplay verb;
  - direct ECS writes to gameplay state (`HandContents`, `AuctionState`,
    `PlayerEconomies`, `BoardUnit`, `ObjectiveIdentity`, etc.);
  - any C2S protocol message synthesis (`C2SPlaceUnit`, `C2SDraftReady`,
    `C2SAuctionBid`, …);
  - any short-circuit of the lobby/draft/placement/resolution flow.
- **Low-level input only.** Driver actions translate to the same input
  primitives a real player produces with keyboard, mouse, and (later)
  gamepad. The autoplay code path injects into the same `ButtonInput<T>`
  resources Bevy itself fills from the OS input layer, so downstream
  gameplay code cannot tell the difference.
- **Dev-only.** The harness lives behind a Cargo feature (`autoplay-remote`)
  AND requires the env var `CCGS_AUTOPLAY=1` to actually bind the RPC port
  at runtime. Release builds do not include the harness; debug builds with
  the feature off do not start it.
- **Localhost only.** The RPC socket binds to `127.0.0.1` only; no remote
  exposure.
- **Generic ECS read helpers, if they ever land here, are debug-only and
  must not be part of autoplay pass/fail evidence.**

## Scope ladder

| Level | In scope NOW (PROMPT 1595) | Deferred |
| --- | --- | --- |
| BRP smoke (status, input, clear, screenshot) | ✅ | — |
| Continuous external driver loop (fixed tick) | ✅ | — |
| Local docs / runbook | ✅ | — |
| Project-local autoplay skill | ✅ | — |
| Launcher / RPC helper scripts | ✅ | — |
| Internal recipe library | ❌ | yes (next prompt) |
| Headless multi-client matrix | ❌ | yes |
| Capture (video / audio / timeline) | ❌ | `liv-autoplay-capture` |
| Dashboard / run browser | ❌ | `liv-autoplay-dashboard` |
| Custom scenario authoring | ❌ | future |
| Semantic gameplay mutation endpoints | ❌ | **forbidden by invariant** |
| Direct ECS state writes as pass/fail evidence | ❌ | **forbidden by invariant** |

The first useful QA target this slice enables is:

- **visible local Windows/native client launched in autoplay mode**;
- **one driven client** plus a server-side bot participant where the bot loop
  already supports the phase (Lobby ready + Draft ready + empty
  PlacementSubmission as of PROMPT 1582);
- **the driver can move the mouse, click, press keys, and capture a
  screenshot at a fixed tick rate** without invoking one shell command per
  action.

End-to-end "drive a full game lobby → result screen" is **not** claimed by
this slice. The driver substrate exists; recipe authoring is the next
prompt's job, and missing/broken bot phases (auction bid ingestion,
placement, resolution acknowledgements) are blockers tracked in the report.

## Architecture

```
+-------------------------------+      127.0.0.1:15873 (env-overridable)
| Python driver (urllib, std)   |  ---->  HTTP POST /  (JSON-RPC 2.0)
| tools/autoplay/driver.py      |  <----  JSON response
+-------------------------------+
              |
              v
+---------------------------------------------+
| client process (cargo run -p client         |
|     --features autoplay-remote              |
|     env CCGS_AUTOPLAY=1)                    |
|                                             |
|  AutoplayPlugin                             |
|   - std::thread RPC server                  |
|   - Arc<Mutex<AutoplayShared>> command bus  |
|   - Bevy systems:                           |
|       * drain command queue                 |
|       * apply input injection               |
|       * spawn Screenshot::primary_window    |
|       * publish status snapshot             |
+---------------------------------------------+
              |
              v
  production/qa/evidence/autoplay-runs/<timestamp>/
    status.json           (last status snapshot)
    process.log           (client stdout/stderr if launched via run-autoplay.ps1)
    driver-timeline.jsonl (one row per driver tick)
    screenshots/<seq>.png (one PNG per screenshot RPC)
```

### Why a hand-rolled JSON-RPC server, not `bevy_remote`

- The repo does not currently take a `bevy_remote` / `bevy_brp_extras`
  dependency, and `client/Cargo.toml` is sensitive to feature surface (see
  the long comment block around the lightyear feature set). Adding
  `bevy_remote` would broaden the BRP attack surface and ship a
  generic-mutation HTTP server we explicitly do not want.
- The harness's RPC needs are very small (status, input, clear, screenshot,
  capabilities) and can be served by a ~150-line std-library TCP+HTTP+JSON
  thread. No tokio, no hyper, no extra serde derivations.
- If/when a future prompt decides we need the broader BRP surface for
  diagnostic reads, that decision is layered ON TOP of the autoplay
  harness, not in place of it, and is gated by an explicit ADR.

### Why low-level input as the injection surface

- Gameplay code must not have a privileged "driven by autoplay" branch.
  Injecting at `ButtonInput<KeyCode>` / `ButtonInput<MouseButton>` /
  `Window::set_physical_cursor_position` means every downstream consumer
  — UI picking, hand drag, shop click handlers, settings keyboard nav,
  HUD shortcuts — runs the **same** code paths the real user exercises.
- Bugs reproduced by autoplay match bugs reported by humans.

## RPC surface (v1, additive forever)

All requests are HTTP `POST /` with JSON-RPC 2.0 envelope. Methods:

| Method | Params | Result |
| --- | --- | --- |
| `autoplay/capabilities` | — | `{ version: u32, methods: { … }, env: {…} }` |
| `autoplay/status` | — | snapshot (see schema in code) |
| `autoplay/input` | `{ keys?: [string], mouse_buttons?: [string], cursor?: { screen: [f32,f32] }, scroll?: { x, y } }` | `{ queued: u64 }` |
| `autoplay/clear_input` | — | `{ queued: u64 }` |
| `autoplay/screenshot` | `{ reason?: string }` | `{ queued: u64, path: string }` |

Adding methods is allowed; renaming/removing methods is a breaking change
that requires bumping the `version` field returned by `capabilities` and
updating both the project-local skill and the driver template.

## Artifact layout

Root: `production/qa/evidence/autoplay-runs/<timestamp>/` where `<timestamp>`
is `YYYYMMDD-HHMMSS-Z` in UTC. Override with `CCGS_AUTOPLAY_ARTIFACT_DIR=<path>`.

- `status.json` — last `autoplay/status` snapshot, rewritten on every poll.
- `process.log` — client stdout+stderr, written by the launcher script.
- `driver-timeline.jsonl` — one JSON row per driver tick:
  `{ elapsed_secs, recipe, status }`.
- `screenshots/<seq>.png` — one PNG per `autoplay/screenshot` RPC.
- `screenshots/<seq>.json` — sidecar with `{ requested_at, captured_at, reason }`.

## Env vars

| Var | Default | Purpose |
| --- | --- | --- |
| `CCGS_AUTOPLAY` | unset (off) | `1` enables the RPC thread at plugin build time. |
| `CCGS_AUTOPLAY_PORT` | `15873` | TCP port for the RPC server. `0` lets the OS pick. |
| `CCGS_AUTOPLAY_ARTIFACT_DIR` | `production/qa/evidence/autoplay-runs/<timestamp>` | Artifact root. |

`15873` was chosen to avoid `15702` (the default `bevy_remote` port we may
adopt later for the diagnostic surface) and to stay outside common dev port
ranges.

## What was deferred and why

| Item | Why deferred | Next prompt should… |
| --- | --- | --- |
| Internal recipe library (e.g. "lobby-confirm-then-ready") | Recipes need to be small and observable; first prove the substrate works. | Author `tools/autoplay/recipes/*.py` and a recipe schema. |
| Headless mode | Requires `cargo run -p client --no-default-features --features autoplay-remote,headless` and a headless render target; no `headless` feature exists yet. | Add a `headless` Cargo feature and document a wgpu null-backend run. |
| Multi-client matrix | Requires per-instance ports + per-instance artifact dirs; substrate already accepts `CCGS_AUTOPLAY_PORT`/`CCGS_AUTOPLAY_ARTIFACT_DIR` overrides. | Drive 2 clients from one orchestrator script using `--port` flags. |
| Video / audio capture | Out of scope for this skill. | Use `liv-autoplay-capture` when adopted. |
| Dashboard | Out of scope for this skill. | Use `liv-autoplay-dashboard` when adopted. |
| Custom scenario authoring | Premature before recipes exist. | After recipe library lands. |
| Semantic gameplay mutation endpoints | **Forbidden by invariant.** | Never. |
| Direct ECS state writes as pass/fail evidence | **Forbidden by invariant.** | Never. |

## Interop with existing repo infrastructure

| Existing surface | Relationship to autoplay |
| --- | --- |
| `QASnapshotPlugin` (F9 in-game capture, `CCGS_QA_SNAPSHOT=1`) | **Independent.** The QA snapshot is human-operated, captures both a PNG and a structured `snapshot.json` of ECS state, and is gated by `CCGS_QA_SNAPSHOT`. Autoplay's screenshot is just a PNG of the primary window. Both may coexist in the same process. The driver can press `F9` via `autoplay/input` to trigger a QA snapshot when a recipe wants a labelled artifact. |
| `tools/two-client-runtime/` | **Different layer.** Two-client-runtime ticks server + 2 clients in-process via `App::update()` for protocol-level harness work. Autoplay drives the **windowed** client like a human. They do not compete. |
| `tools/dev-launcher-app/` | **Compatible.** The launcher app can pass `--features autoplay-remote` + `CCGS_AUTOPLAY=1` when spawning the client. A future dev-launcher job kind ("autoplay run") would call `tools/autoplay/run-autoplay.ps1`. |
| Server-side bot participant (PROMPTs 1514 / 1531 / 1582) | **Complementary.** The bot drives the *server* peer; autoplay drives the *human* peer. Together they get one autoplay-driven client + one bot through phases that both sides currently support (Lobby, DraftInitial, DraftAuction decisions). |

## Verification policy

- Path allowlist + `git diff --check` on every autoplay change.
- `cargo check -p client --features autoplay-remote` whenever the harness
  code changes. Do NOT add the workspace-wide test suite to the autoplay
  gate; broad verification is a separate VERIFY lane.
- Runtime smoke when practical: `tools/autoplay/run-smoke.ps1` launches the
  client, polls `autoplay/status`, sends one input frame, clears, requests a
  screenshot, and exits non-zero on any RPC failure.
