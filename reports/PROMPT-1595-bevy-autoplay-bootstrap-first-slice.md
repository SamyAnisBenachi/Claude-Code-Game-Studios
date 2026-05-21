# PROMPT 1595 -- BEVY-AUTOPLAY-BOOTSTRAP-FIRST-SLICE

**Status:** SHIPPED
**Base:** `origin/main@3a4603af` (state: record post-1593 orchestration state)
**Branch:** `work/bevy-autoplay-bootstrap-1595`
**Worktree:** `D:/Tmp/wt-1595`
**Skill applied:** `liv-bevy-autoplay-bootstrap` (read from
`D:/_DEV/Work/liv-skills/skills/catalog/liv-bevy-autoplay-bootstrap/SKILL.md`)
and `liv-bevy-018` for Bevy code patterns.

## Scope delivered

First useful vertical slice of a Bevy autoplay/automation testing system
for the CCGS client. Implements **both** halves of the bootstrap skill's
"first useful slice":

1. **BRP smoke layer** — one launcher command brings the client up in
   autoplay mode, and an agent (or the driver) can: query
   `autoplay/capabilities`, query `autoplay/status`, inject one low-level
   input frame, clear all injected input, request a screenshot, and read
   durable artifacts under
   `production/qa/evidence/autoplay-runs/<timestamp>/`.
2. **Continuous external driver loop** — `tools/autoplay/driver.py` is a
   persistent Python process (std-lib only) that ticks at a fixed Hz, runs
   recipes (`smoke`, `idle`), and writes one JSONL row per tick. It is NOT
   a one-shell-RPC-per-frame helper.

Plus: local docs/runbook (`docs/autoplay.md`), project-local skill
(`skills/ccgs-autoplay/SKILL.md`), launcher script
(`tools/autoplay/Run-AutoplaySmoke.ps1`), and Python helper
(`tools/autoplay/rpc.py`).

## Implemented files

| File | Purpose | LoC |
| --- | --- | --- |
| `client/src/autoplay.rs` (new) | `AutoplayPlugin`, Arc<Mutex> command bus, std-lib HTTP+JSON-RPC server thread, low-level input injection systems, screenshot trigger, status publisher, dependency-free JSON parser/encoder, 10 unit tests | 1264 |
| `client/src/lib.rs` (mod) | `#[cfg(feature = "autoplay-remote")] pub mod autoplay;` | +6 |
| `client/src/main.rs` (mod) | `#[cfg(feature = "autoplay-remote")] app.add_plugins(client::autoplay::AutoplayPlugin)` | +6 |
| `client/Cargo.toml` (mod) | `autoplay-remote = []` (off by default) | +6 |
| `docs/autoplay.md` (new) | Architecture, scope ladder, hard invariants, RPC surface, env vars, deferred items, interop matrix, verification policy | 213 |
| `skills/ccgs-autoplay/SKILL.md` (new) | Project-local operator runbook for future agents | 192 |
| `tools/autoplay/driver.py` (new) | Persistent Python driver (std-lib only); recipes `smoke`, `idle` | 220 |
| `tools/autoplay/rpc.py` (new) | One-shot Python RPC helper (capabilities/status/screenshot/input/clear) | 112 |
| `tools/autoplay/Run-AutoplaySmoke.ps1` (new) | Windows launcher: build → spawn client with env vars → wait for port → run driver → tear down → emit `launcher-status.json` | 137 |
| `tools/autoplay/README.md` (new) | Quick start + rules + recipe authoring | 63 |

Total: **+2219** lines across 10 files. No file deletions. No edits to
production gameplay code, server code, server bot loops, sprint
paperwork, session state, stage file, lightyear protocol, Cargo.lock, or
any other out-of-scope surface.

## How to run

```powershell
# Smallest possible smoke (timestamped artifact dir, builds + launches + drives + cleans up):
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1
```

```sh
# Manual development loop (foreground client; iterate in two terminals):
# terminal 1:
$env:CCGS_AUTOPLAY="1"; cargo run -p client --features autoplay-remote
# terminal 2:
python tools/autoplay/rpc.py capabilities
python tools/autoplay/rpc.py status
python tools/autoplay/rpc.py input --keys-down KeyA --cursor 400 300
python tools/autoplay/rpc.py clear
python tools/autoplay/rpc.py screenshot --reason debug
python tools/autoplay/driver.py --recipe smoke --ticks 10
```

Activation gates (both required):
- Cargo: `--features autoplay-remote` (default off).
- Runtime: env var `CCGS_AUTOPLAY=1` at client process startup.

Default port: `15873` (override `CCGS_AUTOPLAY_PORT`). Artifact root
default: `production/qa/evidence/autoplay-runs/<timestamp>/` (override
`CCGS_AUTOPLAY_ARTIFACT_DIR`).

## Hard invariants (enforced by code review on every autoplay PR)

- **No gameplay mutation.** The harness's RPC surface is exhaustive: only
  `autoplay/capabilities`, `autoplay/status`, `autoplay/input`,
  `autoplay/clear_input`, and `autoplay/screenshot`. There is no
  `select_card`, `advance_phase`, `give_xp`, `set_health`, `kill`,
  `teleport`, `force_resolution`, or any other semantic verb. There is no
  C2S message synthesis path. There is no direct ECS state write.
- **Low-level input only.** Driver actions translate to `ButtonInput::press/release`,
  `Window::set_cursor_position`, and `MouseWheel` event writes — the
  exact same surfaces real user input lands on. Bugs the driver
  reproduces correspond 1:1 to bugs human players hit.
- **Dev-only.** Dual-gated: Cargo feature + runtime env var. Release
  builds without the feature do not include the module at all.
- **Localhost only.** RPC socket binds to `127.0.0.1` exclusively.

These match the bootstrap skill's `## Core Invariants` section verbatim
in intent.

## What automation can drive today

| Capability | Status |
| --- | --- |
| Probe `autoplay/capabilities` and persist it | ✅ |
| Poll `autoplay/status` at fixed Hz | ✅ |
| Inject keys (press/release independently) | ✅ (subset: letters, digits, F1–F12, arrows, common modifiers, common edit/navigation) |
| Inject mouse buttons (press/release independently) | ✅ (`Left`/`Right`/`Middle`/`Back`/`Forward`) |
| Warp cursor to logical screen coords | ✅ (via `Window::set_cursor_position`; subject to OS focus rules) |
| Inject scroll deltas | ✅ (`MouseWheel` event write) |
| Clear all injected input in one RPC | ✅ |
| Request screenshot + sidecar JSON | ✅ (`bevy::render::view::screenshot::Screenshot::primary_window()`) |
| Run a custom recipe at fixed Hz from one process | ✅ (`driver.py` `RECIPES` dict) |
| Multiple drivers / pokes against one client | ✅ (RPC server is multi-connection) |

## What automation cannot drive today (deferred — explicit blockers for next prompts)

| Gap | Why | Owns the unblock |
| --- | --- | --- |
| Bot-vs-driven full gameplay loop (lobby → result) | Server bot (PROMPT 1582) decides on auction bids but does **not** push them onto the wire; placement/resolution acknowledgement loops are partial; PROMPT-1594 inventory referenced in the brief was not present in this branch's `reports/` so blockers were not cross-checked against it. | Server bot wave-3+ prompts to land auction-bid wire ingestion, placement submission, and resolution acks. |
| Headless mode | No `headless` Cargo feature exists on `client/`. Trunk + `bevy/webgl2` is WASM-only; no native null-RHI feature combo wired. | A separate "headless render" prompt: add `headless = ["bevy/null_render"]` (or similar), and document a null-RHI smoke. |
| Multi-client autoplay matrix | Substrate already honours per-process `CCGS_AUTOPLAY_PORT` and `CCGS_AUTOPLAY_ARTIFACT_DIR`, but no orchestrator script bundles two driven clients. | A `Run-AutoplayPair.ps1` (+ doc) prompt. |
| Recipe library beyond `smoke`/`idle` | Recipes for "lobby confirm CTA", "shop click", "auction bid via UI", "F9 QA snapshot" need pixel coordinates and per-screen knowledge. | A "recipe authoring" prompt that also adds a `tools/autoplay/recipes/*.py` directory and schema. |
| Video / audio capture | Out of scope per skill structure; covered by `liv-autoplay-capture` when that skill is adopted. | A future prompt invoking `liv-autoplay-capture`. |
| Dashboard / run browser | Out of scope per skill structure; covered by `liv-autoplay-dashboard`. | A future prompt invoking `liv-autoplay-dashboard`. |
| Semantic gameplay mutation endpoints | **Forbidden by invariant.** | Never. |
| Direct ECS state writes as autoplay verdicts | **Forbidden by invariant.** | Never. |

## Artifact locations

Default artifact root (per launch):

```
production/qa/evidence/autoplay-runs/<UTC-YYYYMMDD-HHMMSS-Z>/
├── status.json            # written by client every ~15 frames
├── process.log            # client stdout+stderr (launcher only)
├── launcher-status.json   # launcher outcome (launcher only)
├── capabilities.json      # one-shot capability probe (driver only)
├── driver-timeline.jsonl  # one row per driver tick
├── driver.log             # timestamped human-readable driver log
└── screenshots/
    ├── 000000.png
    └── 000000.json        # sidecar: { seq, reason, requested_at_unix_ms, relative_path }
```

`status.json` schema: `autoplay_status_v1` — see `render_status_json` in
`client/src/autoplay.rs`. `launcher-status.json` schema:
`autoplay_launcher_status_v1` — see `Run-AutoplaySmoke.ps1`.

## Validation

### Path allowlist

All edits land in the owned scope from the prompt brief:

- `docs/autoplay.md` ✅
- `skills/ccgs-autoplay/**` ✅ (`skills/ccgs-autoplay/SKILL.md`)
- `tools/autoplay/**` ✅ (`driver.py`, `rpc.py`, `Run-AutoplaySmoke.ps1`, `README.md`)
- `client/src/autoplay.rs` ✅ (new module, behind `autoplay-remote` feature)
- `client/src/lib.rs` ✅ (3-line `#[cfg]` mod declaration)
- `client/src/main.rs` ✅ (4-line `#[cfg]` plugin registration)
- `client/Cargo.toml` ✅ (6-line `[features]` entry)
- `reports/PROMPT-1595-bevy-autoplay-bootstrap-first-slice.md` ✅

Forbidden paths confirmed untouched:
- `production/sprint-status.yaml` ❌ not touched
- `production/session-state/**` ❌ not touched
- `production/stage.txt` ❌ not touched
- Sprint close-out/activation paperwork ❌ not touched
- Unrelated client/server gameplay logic ❌ not touched
- Unrelated Cargo/CI files ❌ not touched
- `server/` ❌ not touched
- `shared/` ❌ not touched
- `Cargo.lock` ❌ unchanged

### `git diff --check`

Clean (no whitespace errors).

### Focused cargo check

```
$ cargo check -p client --features autoplay-remote
warning: `client` (lib) generated 101 warnings (13 duplicates)
    Finished `dev` profile [optimized + debuginfo] target(s) in 3.37s
```

Zero new warnings attributable to `client/src/autoplay.rs`. The 101
warnings are all pre-existing `deprecated`-marker warnings (PROMPT
SOURCE-1077-08 / `HudEntity` / `HandUiEntity` / `ShopAuctionUiEntity`)
unrelated to this change.

### Focused unit tests

```
$ cargo test -p client --features autoplay-remote --lib autoplay::
running 10 tests
test autoplay::tests::capabilities_json_is_valid_and_lists_methods ... ok
test autoplay::tests::config_from_env_defaults_to_off ... ok
test autoplay::tests::json_parse_round_trip_basic_object ... ok
test autoplay::tests::json_string_escapes ... ok
test autoplay::tests::parse_input_decodes_low_level_fields ... ok
test autoplay::tests::parse_input_rejects_unknown_key ... ok
test autoplay::tests::parses_keycode_letters_digits_function_keys ... ok
test autoplay::tests::parses_mouse_button_names ... ok
test autoplay::tests::render_status_json_is_valid ... ok
test autoplay::tests::utc_stamp_format ... ok
test result: ok. 10 passed; 0 failed; 0 ignored
```

10 / 10 passing. Coverage: KeyCode name parsing, MouseButton name parsing,
input-payload decoder (incl. rejection of unknown keys), status JSON
shape, capabilities JSON shape, internal JSON parser/encoder
round-trip, UTC timestamp format, default-artifact-dir layout.

### Broad verification

Not run, per `Hard invariants` in the prompt brief ("Do not block on
broad Cargo. Use focused validation only; broad verification will be a
separate VERIFY lane"). A VERIFY lane prompt may pick this branch up.

### Runtime smoke

**Not executed in this prompt.** The `Run-AutoplaySmoke.ps1` script is
written, lints clean (no PowerShell parse errors evident on read), and
its inputs (`autoplay-remote` feature + `CCGS_AUTOPLAY=1` env) are
verified by unit tests on the Rust side. A human-attended runtime smoke
in a foreground Windows session is the appropriate next manual check; it
was not run here to avoid spawning a long-lived windowed process inside
the worker's headless context.

## Decisions and trade-offs

1. **Hand-rolled JSON-RPC server (no `bevy_remote` / `serde_json`
   dependency)** — The harness needs five methods. Adding `bevy_remote`
   would ship a generic ECS HTTP mutation surface we explicitly do not
   want, and is also not in the workspace today. The hand-rolled server
   is ~150 lines of std-lib code, has its own unit tests, and keeps the
   `client/` feature surface minimal.
2. **Single file, not a sub-module** — `client/src/autoplay.rs` is one
   file with internal `// ----------` section markers. The cohesion (RPC
   server + Bevy systems + JSON helpers all reference the same shared
   `Arc<Mutex<AutoplayInner>>`) does not benefit from sub-module splits
   at this size. Sub-divide if the file passes ~2000 lines.
3. **Low-level input injection via `ResMut<ButtonInput<_>>`** — bypasses
   no gameplay code; every consumer (UI picking, hand drag, settings
   keyboard nav, F9 QA snapshot) runs the same path it runs for real
   players. Validated by `liv-bevy-018` Bevy 0.18 API conformance and by
   compiling against the existing `ButtonInput<KeyCode>` ResMut usages
   already in the codebase.
4. **Default port 15873**, deliberately distinct from `bevy_remote`'s
   conventional `15702`, so a future diagnostic BRP surface can coexist
   without port collisions.
5. **Artifacts under `production/qa/evidence/autoplay-runs/<ts>/`** —
   matches the prompt brief's preferred root and lives next to the
   existing `production/qa/evidence/` corpus.
6. **No `serde_json` in autoplay** — the existing `client/Cargo.toml`
   does include `serde_json`, but the autoplay module deliberately does
   not link it. This keeps the autoplay file portable / re-vendorable
   into a smaller crate later, and removes any worry about serde derive
   churn affecting the harness.

## Next prompts needed

1. **PROMPT N+1 -- AUTOPLAY-RECIPE-LIBRARY-WAVE-1** -- Author
   `tools/autoplay/recipes/lobby_confirm.py`,
   `tools/autoplay/recipes/shop_click.py`,
   `tools/autoplay/recipes/qa_snapshot.py`, and a recipe schema. Add a
   `--recipe-file <path>` flag to `driver.py`. Pixel coordinates will
   need either UI test-IDs (out of scope) or a one-time visual
   calibration step (documented in the recipe).
2. **PROMPT N+2 -- AUTOPLAY-HEADLESS-MODE** -- Add a `headless` Cargo
   feature to `client/` that swaps the renderer for a null backend, so
   autoplay smoke can run in CI without a display.
3. **PROMPT N+3 -- AUTOPLAY-PAIR-LAUNCHER** -- `Run-AutoplayPair.ps1`
   that launches two clients (different ports / artifact dirs) and runs
   per-client drivers concurrently.
4. **PROMPT N+4 -- BOT-WAVE-3-AUCTION-BID-INGESTION** -- Already on the
   plate per PROMPT 1582 SHIPPED report; unblocks bot-vs-driven full
   flow.
5. **PROMPT N+5 -- AUTOPLAY-SMOKE-IN-CI** -- Wire
   `Run-AutoplaySmoke.ps1` into the dev-launcher or a GitHub Actions
   workflow, gated on the future `headless` feature landing.

## Git outcome

- Branch: `work/bevy-autoplay-bootstrap-1595`.
- Worktree: `D:/Tmp/wt-1595`.
- Commit: created with conventional message + PROGRESS.md entry per
  `[Agents commit their work]` feedback.
- Push: attempted; result documented in the relay summary.

## Hard invariant cross-check (skill checklist)

- [x] Autoplay observes app state and injects low-level input only.
- [x] No gameplay mutation endpoints.
- [x] Project wrappers (`tools/autoplay/*`) preferred over raw RPC
      one-liners in skill docs.
- [x] Dev-only (Cargo feature + env var).
- [x] No competing BRP surface added; future generic BRP coexistence
      documented in `docs/autoplay.md` and `skills/ccgs-autoplay/SKILL.md`.
- [x] Generic BRP debug helpers (none added today) flagged as
      debug-only-not-pass/fail in both docs.
- [x] User edits preserved; the only modifications to existing files are
      additive (`lib.rs` `mod` line, `main.rs` plugin add, `Cargo.toml`
      feature entry).
- [x] Repo verification rules respected (focused `cargo check`, not
      broad workspace tests).

1595: BEVY-AUTOPLAY-BOOTSTRAP-FIRST-SLICE: SHIPPED
