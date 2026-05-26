# Autoplay Harness — Architecture

Status: First slice landed by PROMPT 1595 (`work/bevy-autoplay-bootstrap-1595`).
Recipe library v1 layered on top by PROMPT 1609.
PowerShell 5.1 compatibility fix landed by PROMPT 1619, integrated in PROMPT 1620.
Runtime smoke verified in PROMPT 1621 — non-GUI phases PASS; GUI client launch is
**BLOCKED-HUMAN-GUI** (requires an interactive desktop session; not a script regression).
Skill: see `skills/ccgs-autoplay/SKILL.md` for the project-local runbook.
Operator guide: see [`docs/autoplay/evidence-operator-guide.md`](autoplay/evidence-operator-guide.md) for how to run the suite, read PASS/FAIL/BLOCKED outcomes, and use the observability tools (F8 overlay, F9 snapshot, decision logs).

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
| Internal recipe library | ✅ (PROMPT 1609) | — |
| PowerShell 5.1 compatibility (`Run-AutoplaySmoke.ps1`) | ✅ (PROMPT 1619/1620) | — |
| Non-GUI smoke phases (parse, help, env, offline checks) | ✅ PASS (PROMPT 1621) | — |
| GUI client launch (Bevy windowed, interactive desktop) | ❌ BLOCKED-HUMAN-GUI | requires interactive desktop session |
| Headless CI smoke (no display) | ❌ | Pending PROMPT 1626 feasibility conclusion — do not claim implemented |
| Headless multi-client matrix | ❌ | future |
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
PROMPT 1595 alone. PROMPT 1609 ships the recipe library that composes
phase recipes into a `full-game` path. The composite is gated on
`CCGS_AUTOPLAY_BOT_ROOM_READY=1` (PROMPT 1607's bot-vs-bot soak room)
and emits a `local.block` row + driver exit code 4 when that prerequisite
is missing — so the recipe never silently "passes" against an empty
opponent slot. The phase sub-recipes (`lobby-create`, `class-select`,
`draft-auction-probe`, `placement-drag-probe`) run independently against
a human peer for partial coverage.

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
    process.log           (client stdout/stderr if launched via Run-AutoplaySmoke.ps1)
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
| `autoplay/capabilities` | — | `{ version: u32, methods: { … }, input: {…}, invariants: string }` |
| `autoplay/status` | — | snapshot (see schema in code) |
| `autoplay/input` | `{ keys_down?: [string], keys_up?: [string], mouse_down?: [string], mouse_up?: [string], cursor?: { screen: [f32,f32] }, scroll?: [f32, f32] }` | `{ queued: u64 }` |
| `autoplay/clear_input` | — | `{ queued: u64 }` |
| `autoplay/screenshot` | `{ reason?: string }` | `{ queued: u64, relative_path: string }` |

Adding methods is allowed; renaming/removing methods is a breaking change
that requires bumping the `version` field returned by `capabilities` and
updating both the project-local skill and the driver template.

## Artifact layout

Root: `production/qa/evidence/autoplay-runs/<timestamp>/` where `<timestamp>`
is `YYYYMMDD-HHMMSS-Z` in UTC. Override with `CCGS_AUTOPLAY_ARTIFACT_DIR=<path>`.

- `status.json` — last `autoplay/status` snapshot, rewritten on every poll.
- `process.log` — client stdout+stderr, written by the launcher script.
- `driver-timeline.jsonl` — one JSON row per driver tick:
  `{ tick, recipe, elapsed_secs, status, action_results }`.
- `checkpoints.jsonl` — one JSON row per recipe `local.*` pseudo-action
  (`local.checkpoint`, `local.note`, `local.block`). Created lazily; absent
  if the recipe never emits any.
- `capabilities.json` — one-shot capability probe at driver startup.
- `driver.log` — human-readable progress log.
- `screenshots/<seq>.png` — one PNG per `autoplay/screenshot` RPC.
- `screenshots/<seq>.json` — sidecar with `{ requested_at, captured_at, reason }`.

## Recipe library (PROMPT 1609)

Recipes live under `tools/autoplay/recipes/` and are discovered by the
registry in `tools/autoplay/recipes/__init__.py`. The driver loads a
recipe by name with `--recipe <name>`; the registered set is:

| Recipe | Phase | Checkpoints |
| --- | --- | --- |
| `smoke` | substrate probe | — |
| `idle` | observability soak | — |
| `lobby-create` | lobby Create + Confirm | `lobby-loaded`, `lobby-confirmed` |
| `class-select` | class pick + Confirm | `class-select-loaded`, `class-confirmed` |
| `draft-auction-probe` | shop click → auction bid → ready | `shop-loaded`, `shop-slot-clicked`, `auction-loaded`, `auction-ready` |
| `placement-drag-probe` | hand → board drag, Submit | `placement-loaded`, `placement-dragged`, `placement-submitted` |
| `full-game` | composite, requires PROMPT 1607 soak room | all of the above + `full-game-resolution` |

### How to run

```sh
python tools/autoplay/driver.py --list-recipes
python tools/autoplay/driver.py --recipe lobby-create
python tools/autoplay/driver.py --recipe full-game --timeout 300
```

### Driver pseudo-actions

`RecipeBuilder` exposes three pseudo-methods that never hit the wire;
the driver consumes them locally and writes them to
`checkpoints.jsonl`:

| Method | Purpose | Driver effect |
| --- | --- | --- |
| `local.checkpoint` | Mark a phase boundary; optionally also screenshot. | Append `checkpoints.jsonl` row + (by default) emit `autoplay/screenshot`. |
| `local.note` | Free-form annotation (e.g. "env override failed parse"). | Append `checkpoints.jsonl` row. |
| `local.block` | Recipe cannot proceed (upstream prerequisite missing). | Append `checkpoints.jsonl` row, flip exit code to 4, stop the run. |

The driver rejects any recipe action whose `method` is outside the
union of `ALLOWED_RPC_METHODS` ∪ `LOCAL_METHODS` before sending the
first RPC. This is the second layer of the "no semantic verbs"
invariant; the Rust harness is the first.

### Coordinate overrides

Phase recipes target UI buttons at default fractional positions
(centre column / lower third). Override per-key:

```sh
$env:CCGS_AUTOPLAY_LOBBY_CREATE_BTN  = "0.50,0.60"
$env:CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN = "0.50,0.88"
# ...see tools/autoplay/README.md for the full list
```

Malformed overrides emit a `local.note` row recording the parse
failure and fall back to the default.

### Blocked steps as of PROMPT 1609

| Recipe | Blocker | Detection |
| --- | --- | --- |
| `full-game` | PROMPT 1607 bot-vs-bot soak room (`Start-BotVsBotSoak.ps1`) not yet on main. | If `CCGS_AUTOPLAY_BOT_ROOM_READY != "1"`, recipe emits `local.block` and the driver exits 4. |
| Auction bid acknowledgement | No observability surface for "bid accepted"; relies on a fixed `CCGS_AUTOPLAY_AUCTION_BID_WAIT` tick budget. | Waits then proceeds; reviewer confirms outcome from the post-bid checkpoint screenshot. |
| Placement accept/reject | No autoplay-visible signal for accepted submission; same wait-then-screenshot strategy. | Reviewer confirms outcome from `placement-submitted` screenshot. |
| Resolution / round-end | No autoplay observability for round transitions. | `full-game-resolution` checkpoint marks the wall-clock end of the recipe, not a real round boundary. |

These blockers are by design — the autoplay surface deliberately does
not expose gameplay state. Upgrading detection beyond
checkpoint-screenshot review is the job of the QA snapshot harness
(`F9` / `CCGS_QA_SNAPSHOT=1`), not autoplay.

## Env vars

| Var | Default | Purpose |
| --- | --- | --- |
| `CCGS_AUTOPLAY` | unset (off) | `1` enables the RPC thread at plugin build time. |
| `CCGS_AUTOPLAY_PORT` | `15873` | TCP port for the RPC server. `0` lets the OS pick. |
| `CCGS_AUTOPLAY_ARTIFACT_DIR` | `production/qa/evidence/autoplay-runs/<timestamp>` | Artifact root. |
| `CCGS_AUTOPLAY_DRIVER_ARTIFACT_DIR` | `production/qa/evidence/autoplay-runs/driver` | Override the driver-side artifact dir (used when the driver is launched manually, outside `Run-AutoplaySmoke.ps1`). |
| `CCGS_AUTOPLAY_BOT_ROOM_READY` | unset | Set to `1` once PROMPT 1607 bot-vs-bot soak room is live; required by the `full-game` recipe. |
| `CCGS_AUTOPLAY_<KEY>` (button frac) | per `_coords.DEFAULTS` | Per-recipe fractional coordinate overrides; see `tools/autoplay/README.md`. |
| `CCGS_AUTOPLAY_AUCTION_MOUNT_WAIT` | 12 ticks | Ticks the `draft-auction-probe` recipe waits after the shop confirm click before clicking the bid CTA. |
| `CCGS_AUTOPLAY_AUCTION_BID_WAIT` | 10 ticks | Ticks the `draft-auction-probe` recipe waits between the bid click and the ready click. |

`15873` was chosen to avoid `15702` (the default `bevy_remote` port we may
adopt later for the diagnostic surface) and to stay outside common dev port
ranges.

## What was deferred and why

| Item | Why deferred | Next prompt should… |
| --- | --- | --- |
| Internal recipe library (e.g. "lobby-confirm-then-ready") | Landed in PROMPT 1609. | Iterate on recipe coverage as new UI lands; add `--checkpoints-only` mode if reviewers want timeline-only runs. |
| Headless mode | The autoplay harness depends on WinitPlugin + RenderPlugin for screenshot capture and cursor injection; stripping them eliminates its primary value. PROMPT 1626 feasibility analysis is in progress — verdict pending. The `tools/two-client-runtime` headless path (MinimalPlugins, no display) already covers server/protocol-level CI; it is the recommended CI-grade headless smoke until a virtual display is available. | Wait for PROMPT 1626 verdict before scheduling headless autoplay work. |
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
| `tools/dev-launcher-app/` | **Compatible.** The launcher app can pass `--features autoplay-remote` + `CCGS_AUTOPLAY=1` when spawning the client. A future dev-launcher job kind ("autoplay run") would call `tools/autoplay/Run-AutoplaySmoke.ps1`. |
| Server-side bot participant (PROMPTs 1514 / 1531 / 1582) | **Complementary.** The bot drives the *server* peer; autoplay drives the *human* peer. Together they get one autoplay-driven client + one bot through phases that both sides currently support (Lobby, DraftInitial, DraftAuction decisions). |

## Verification policy

- Path allowlist + `git diff --check` on every autoplay change.
- `cargo check -p client --features autoplay-remote` whenever the harness
  code changes. Do NOT add the workspace-wide test suite to the autoplay
  gate; broad verification is a separate VERIFY lane.
- Runtime smoke when practical: `tools/autoplay/Run-AutoplaySmoke.ps1` launches the
  client, polls `autoplay/status`, sends one input frame, clears, requests a
  screenshot, and exits non-zero on any RPC failure.
