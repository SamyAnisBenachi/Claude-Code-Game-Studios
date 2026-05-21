---
name: ccgs-autoplay
description: Use when running, inspecting, or debugging CCGS (Claude-Code Game Studios — Lanes and Lies) autoplay sessions on the Bevy client, including manual RPC pokes, the smoke launcher, the persistent Python driver, screenshots, artifact directories, and the rules that govern what autoplay is allowed to do.
---

# ccgs-autoplay

Project-local runbook for the autoplay harness landed by PROMPT 1595.
Architecture lives in [`docs/autoplay.md`](../../docs/autoplay.md); use
this skill from the repo root when **operating** the harness.

## Current Status

- Manual remote autoplay (RPC): **✅ landed** (PROMPT 1595)
- Continuous driver loop: **✅ landed** (`tools/autoplay/driver.py`)
- Smoke launcher: **✅ landed** (`tools/autoplay/Run-AutoplaySmoke.ps1`)
- Recipe library v1: **✅ landed** (PROMPT 1609; `tools/autoplay/recipes/`)
  - `smoke`, `idle` — substrate probes
  - `lobby-create`, `class-select`, `draft-auction-probe`, `placement-drag-probe` — per-phase
  - `full-game` — composite (gated on PROMPT 1607 bot-vs-bot soak room)
- Screenshots: **✅ landed** (`autoplay/screenshot` → `<artifact-dir>/screenshots/<seq>.png`)
- Checkpoints (`checkpoints.jsonl`): **✅ landed** (PROMPT 1609; phase boundaries + blocker rows)
- Headless mode: **❌ deferred** (no `headless` Cargo feature yet)
- Multi-client matrix: **❌ deferred** (substrate supports per-instance ports already)
- Video / audio capture: **❌ deferred** (`liv-autoplay-capture` when adopted)
- Dashboard: **❌ deferred** (`liv-autoplay-dashboard` when adopted)
- PROMPT 1607 bot-vs-bot soak room: **❌ not on main**; `full-game` recipe
  emits `local.block` + driver exits with code 4 until
  `CCGS_AUTOPLAY_BOT_ROOM_READY=1` is set against a live soak room.

## Fast path

Launch the BRP smoke (default port 15873, timestamped artifact dir):

```powershell
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1
```

Run the autoplay loop against an already-running client:

```sh
python tools/autoplay/driver.py --list-recipes
python tools/autoplay/driver.py --port 15873 --recipe smoke --ticks 20 --hz 5
python tools/autoplay/driver.py --recipe lobby-create
python tools/autoplay/driver.py --recipe full-game --timeout 300
```

One-shot status / screenshot / input:

```sh
python tools/autoplay/rpc.py status
python tools/autoplay/rpc.py screenshot --reason debug
python tools/autoplay/rpc.py input --keys-down KeyA --cursor 400 300
python tools/autoplay/rpc.py clear
```

Artifacts:

```text
production/qa/evidence/autoplay-runs/<timestamp>/
  status.json
  process.log              # client stdout+stderr (launcher only)
  launcher-status.json     # launcher outcome / exit codes (launcher only)
  capabilities.json        # one-shot capability probe (driver only)
  driver-timeline.jsonl    # one row per driver tick (driver only)
  driver.log               # human-readable driver progress (driver only)
  checkpoints.jsonl        # one row per local.checkpoint / local.note / local.block (PROMPT 1609)
  screenshots/000000.png
  screenshots/000000.json  # sidecar: { seq, reason, requested_at_unix_ms }
```

### Driver exit codes

| Code | Meaning |
| --- | --- |
| 0 | recipe completed cleanly (or hit `--timeout`) |
| 1 | RPC error during the run |
| 2 | RPC server never bound during `--startup-grace` |
| 4 | recipe emitted `local.block` — upstream prerequisite missing |

Override the artifact root with `CCGS_AUTOPLAY_ARTIFACT_DIR=<path>` before
launching the client, or pass `-ArtifactDir <path>` to the smoke script.

## Rules

- **Drive only low-level input:** keys (by name, e.g. `KeyA`, `Space`,
  `Escape`, `F9`, `ArrowLeft`), mouse buttons (`Left`/`Right`/`Middle`/
  `Back`/`Forward`), cursor screen position, and scroll deltas.
- **Never** drive a recipe with a semantic gameplay verb (`kill`,
  `select_card`, `advance_phase`, `give_xp`, …). The Rust harness has no
  such endpoint, and the next person to land one MUST be turned away —
  see `docs/autoplay.md` "Hard invariants".
- **Never** use generic BRP mutation/debug helpers as autoplay pass/fail
  evidence. If/when a diagnostic BRP surface lands later, it is for
  inspection only, not for verdicts.
- **One driver process per session.** Do not loop `rpc.py` in a shell as a
  poor person's autoplay engine. Use `driver.py`.
- **Always launch through `Run-AutoplaySmoke.ps1` for recorded evidence.**
  Manual `cargo run -p client --features autoplay-remote` + `CCGS_AUTOPLAY=1`
  works for development, but produces no `process.log` or
  `launcher-status.json`.

## Activation

The harness is dual-gated — both must hold:

1. **Cargo feature**: build/run with `--features autoplay-remote`. The
   feature is OFF by default, so stock `cargo run -p client`,
   `trunk build`, and CI workspace builds never include the harness.
2. **Env var**: `CCGS_AUTOPLAY=1` at process startup. The plugin checks
   this in its `build` method; any other value (including unset) is a
   no-op even when the feature is compiled in.

Optional env vars:

| Var | Default | Purpose |
| --- | --- | --- |
| `CCGS_AUTOPLAY_PORT` | `15873` | TCP port for the RPC server (`127.0.0.1` only). |
| `CCGS_AUTOPLAY_ARTIFACT_DIR` | timestamped under `production/qa/evidence/autoplay-runs/` | Artifact root. |

## Commands

### RPC surface (HTTP JSON-RPC 2.0 on `127.0.0.1:<port>`)

| Method | Params | Result |
| --- | --- | --- |
| `autoplay/capabilities` | — | `{ version, methods, input, invariants }` |
| `autoplay/status` | — | snapshot (frame, uptime, window size, cursor, pressed keys/buttons, command count, screenshot count, last_error) |
| `autoplay/input` | `keys_down?`, `keys_up?`, `mouse_down?`, `mouse_up?`, `cursor.screen?`, `scroll?` | `{ queued }` |
| `autoplay/clear_input` | — | `{ queued }` |
| `autoplay/screenshot` | `reason?` | `{ queued, relative_path }` |

### Adding a recipe

Recipes are one Python module per recipe under
`tools/autoplay/recipes/`. Each module exposes `NAME`, `DESCRIPTION`,
and `build(ctx) -> list[dict]`; register it in
`tools/autoplay/recipes/__init__.py`. Use the `RecipeBuilder`
primitives from `recipes._builder` (click, drag, press, checkpoint,
note, block); the driver rejects any action whose `method` is outside
the autoplay allowlist or the `local.*` pseudo-method set.

### Recipe library

| Recipe | Phase | Checkpoints |
| --- | --- | --- |
| `smoke` | substrate probe | — |
| `idle` | observability soak | — |
| `lobby-create` | lobby Create + Confirm | `lobby-loaded`, `lobby-confirmed` |
| `class-select` | class pick + Confirm | `class-select-loaded`, `class-confirmed` |
| `draft-auction-probe` | shop click → auction bid → ready | `shop-loaded`, `shop-slot-clicked`, `auction-loaded`, `auction-ready` |
| `placement-drag-probe` | hand → board drag, Submit | `placement-loaded`, `placement-dragged`, `placement-submitted` |
| `full-game` | composite, gated by PROMPT 1607 | all of the above + `full-game-resolution` |

The `full-game` recipe requires the PROMPT 1607 bot-vs-bot soak room.
Until that lane lands, set `CCGS_AUTOPLAY_BOT_ROOM_READY=1` only when
running against a live soak instance; otherwise the recipe writes a
`local.block` row to `checkpoints.jsonl` and the driver exits 4.

## Remote surface

### Project-local autoplay endpoints (safe — low-level input)

See the table under "Commands".

### Coexisting / future diagnostic endpoints

- **`QASnapshotPlugin`** (`CCGS_QA_SNAPSHOT=1`, `F9` shortcut) — human-
  operated; captures PNG **and** structured ECS snapshot to
  `qa-snapshots/`. Independent of autoplay; both may run in the same
  process. Recipes can press `F9` via `autoplay/input` to trigger a QA
  snapshot for a labelled artifact.
- **`bevy_remote` / generic BRP** — **not** currently in the workspace;
  if/when added, scope it to **diagnostic reads** only and document
  reuse here. Generic BRP `insert`/`mutate` helpers MUST NOT participate
  in autoplay verdicts.

## Artifacts

| File | Owner | Schema |
| --- | --- | --- |
| `status.json` | client (`autoplay-remote`) | `autoplay_status_v1` (overwritten every ~15 frames) |
| `screenshots/<seq>.png` | client | binary PNG |
| `screenshots/<seq>.json` | client | `{ seq, reason, requested_at_unix_ms, relative_path }` |
| `capabilities.json` | driver | RPC response of `autoplay/capabilities` |
| `driver-timeline.jsonl` | driver | one row per tick: `{ tick, recipe, elapsed_secs, status, action_result }` |
| `driver.log` | driver | timestamped human-readable progress |
| `process.log` | launcher | client stdout+stderr |
| `launcher-status.json` | launcher | `autoplay_launcher_status_v1` |

## Known gaps

| Gap | What is missing | Impact on QA today |
| --- | --- | --- |
| Bot-driven full gameplay | Server bot decides on auction bids (PROMPT 1582) but does not push them onto the wire; placement / resolution acknowledgement loops are partial. | Autoplay can drive lobby → DraftInitial ready → DraftAuction view, but not a full lobby-to-result loop without a human on the other side. |
| Headless render | No `headless` Cargo feature; the client always opens a window. | Autoplay smoke requires an interactive desktop session on Windows. |
| Multi-client orchestration | Substrate is per-process (port + artifact dir overrides); no script bundles two clients yet. | Two-client autoplay is a `Run-AutoplayPair.ps1` away, but it is not in this slice. |
| Recipe library | Only `smoke` and `idle` ship. | Recipes for "lobby confirm CTA", "shop click", "auction bid via UI" are the next prompt. |
| Capture / dashboard | Not in scope for this prompt. | Use the launcher's PNG + JSONL artifacts; visual review by eyeball. |
| Cursor warp on unfocused window | `Window::set_cursor_position` may be ignored by the OS when the window is not focused. | Always run autoplay against the foreground window during smoke. |

## Operating workflow

1. Pick the smallest run that answers the question.
   - "Does autoplay work at all?" → `Run-AutoplaySmoke.ps1` (≈30 s).
   - "Why did the driver fail on tick N?" → tail
     `<artifact-dir>/driver-timeline.jsonl` and look at `last_error` in
     `status.json` from tick N.
   - "Is the client responsive?" → `python tools/autoplay/rpc.py status`
     repeated by hand.
2. Treat any of these as evidence to inspect, **not** as final verdicts:
   - launcher exit code (RPC port never bound vs. driver returned
     non-zero are different bugs);
   - driver exit code (RPC error mid-run vs. clean tick limit are
     different bugs);
   - `last_error` field of `status.json`;
   - process.log entries containing `client::autoplay`.
3. Before claiming a recipe succeeded, eyeball the latest screenshot in
   `screenshots/` — the harness has no visual oracle.

## Failure triage

| Symptom | Likely cause | Look here |
| --- | --- | --- |
| Launcher reports `rpc_port_never_bound` | Port collision, feature not compiled in, env var missing, or client crash before `AutoplayPlugin::build`. | `process.log`, `tools/autoplay/Run-AutoplaySmoke.ps1` build step output. |
| Driver `startup failed: ...` | Same as above, or client still loading assets past `--startup-grace`. | Increase `--startup-grace`, or watch `process.log` for the `AutoplayPlugin enabled` line. |
| Status returns but `frame` never increments | Bevy Update schedule is stalled (often a panic on render thread). | `process.log` for the panic backtrace. |
| Input RPC returns OK but the game does not react | Cursor warp blocked by OS focus rules; UI picking backend not loaded (`ui_picking` feature off); recipe targeted wrong screen coords. | `status.json.cursor_logical` after the RPC; `status.json.keys_pressed` to confirm injection landed. |
| Screenshot RPC returns OK but file is empty/missing | Render pipeline didn't finish capture before client shutdown; observer dropped. | Sidecar `<seq>.json` confirms request was queued; PNG missing means capture failed. Add a small post-screenshot sleep before killing the client. |
| Driver timeline has gaps in tick numbers | Driver process was suspended (OS scheduler, debugger). | Re-run with `--hz` halved. |
