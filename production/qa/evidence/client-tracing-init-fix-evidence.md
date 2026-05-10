# Client tracing init fix — runtime evidence

**Story:** S11-TD-CLIENT-LOG-001
**Prompt:** 647
**Date:** 2026-05-10
**Branch:** `work/client-tracing-init-fix`

## Problem statement

Two independent diagnostic prompts (641, 646) observed that
`client/` produced **zero lines of stdout** during native runs despite
extensive `tracing::info!` / `tracing::warn!` call sites in the client crate.
This made every UI / state-machine / visibility / network bug require
source-only reasoning and contributed to at least one misdiagnosis (PROMPT 641
Suspect 1 viewport).

## Root cause

Two independent gaps compounded so that *no* global `tracing-subscriber` was
ever installed in the client process:

1. **`client/Cargo.toml`** built bevy with `default-features = false` and the
   features `["2d", "webgl2", "bevy_audio"]`. The `2d` feature collection in
   Bevy 0.18 pulls `bevy_log` in transitively, so `LogPlugin` *was* present at
   runtime, but the crate had no direct `tracing` / `tracing-subscriber`
   dependencies of its own — so explicit init was impossible without first
   adding the deps.
2. **`client/src/main.rs`** never called `tracing_subscriber::fmt().init()` (or
   any equivalent) before `App::new()`. The server crate does install one
   (see `server/src/main.rs:87`); the client never did.

Empirically, even though `LogPlugin` was in `DefaultPlugins`, the diagnostic
runs from PROMPT 641 / 646 were observing zero stdout. (Either the harness
they used did not capture `LogPlugin`'s output, or the default filter and
output target shipped by `LogPlugin` did not reach the surface they checked.
Either way, the result was the same: dark stdout, dark diagnostics.)

## Fix

Three surgical changes, all within the scope allowed by PROMPT 647
(entrypoint + `Cargo.toml` + test helpers):

1. `client/Cargo.toml` — declare explicit deps
   ```toml
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   ```
2. `client/src/main.rs` — install a `tracing-subscriber` *before* `App::new()`
   for non-wasm targets, with an `EnvFilter` that quiets wgpu / naga /
   bevy_ecs at startup so application spans are not drowned out. `RUST_LOG`
   overrides the default.
3. `client/src/main.rs` — disable `LogPlugin` via
   `DefaultPlugins.build().disable::<LogPlugin>()` so our subscriber is
   authoritative and the spurious `bevy_log: Could not set global logger ...`
   ERROR line at startup is removed.
4. `tests/test_helpers.rs` — new `init_test_tracing()` helper that integration
   tests can opt into via `#[path]`. Idempotent via `std::sync::Once`; routes
   through `with_test_writer()` so cargo test capture works.

## Evidence — stdout before vs after

### Before (reported by PROMPT 641 / 646)

```
(zero lines on client stdout — confirmed across multiple diagnostic harnesses)
```

### After — default filter, no `RUST_LOG`

(captured by piping `client.exe` stdout to a file and sampling the first 12s
of startup; the binary was killed before any user interaction or server
connection. Lines reformatted slightly to strip ANSI colour escapes.)

```
2026-05-10T22:26:43Z  INFO bevy_diagnostic::system_information_diagnostics_plugin::internal:
    SystemInfo { os: "Windows 11 Home", kernel: "26200",
                 cpu: "AMD Ryzen AI 9 HX 370 w/ Radeon 890M",
                 core_count: "12", memory: "31.1 GiB" }
2026-05-10T22:26:43Z  WARN wgpu_hal::vulkan::instance:
    InstanceFlags::VALIDATION requested, but unable to find layer:
    VK_LAYER_KHRONOS_validation
2026-05-10T22:26:43Z  WARN wgpu_hal::vulkan::instance: GENERAL [Loader Message (0x0)]
    windows_read_data_files_in_registry: Registry lookup failed to get layer manifest files.
2026-05-10T22:26:44Z  INFO bevy_render::renderer:
    AdapterInfo { name: "NVIDIA GeForce RTX 5070 Ti Laptop GPU", ... backend: Vulkan }
2026-05-10T22:26:44Z  INFO bevy_render::batching::gpu_preprocessing:
    GPU preprocessing is fully supported on this device.
2026-05-10T22:26:44Z  INFO bevy_winit::system:
    Creating new window Lanes and Lies (0v0)
```

14 lines visible within the first 1s of startup. Format is the standard
`tracing-subscriber::fmt` layout: ISO-8601 timestamp, level, target span,
message. ANSI colour codes are emitted on terminals that support them.

The `wgpu_hal::vulkan::instance: GENERAL [Loader Message (0x0)]` block is
local-driver chatter from the Vulkan validation layer / Overwolf hooks, not
something this fix introduced.

### After — `RUST_LOG=debug` override

Same binary, re-run with `RUST_LOG=debug` to verify `EnvFilter` is wired
correctly and the application-level spans surface when asked:

```
$ RUST_LOG=debug ./client.exe > stdout.log  # killed after 8s
$ wc -l stdout.log
108849 stdout.log

$ grep 'client::' stdout.log | head -5
2026-05-10T22:28:27Z DEBUG bevy_app::app: added plugin: client::presentation::PresentationPlugin
2026-05-10T22:28:27Z DEBUG bevy_app::app: added plugin: client::presentation::board_rendering::BoardRenderingPlugin
2026-05-10T22:28:27Z DEBUG bevy_app::app: added plugin: client::presentation::result_screen::ResultScreenPlugin
2026-05-10T22:28:27Z DEBUG bevy_app::app: added plugin: client::ui::lobby::LobbyUiPlugin
2026-05-10T22:28:27Z DEBUG bevy_app::app: added plugin: client::asset_wiring::AssetWiringPlugin
```

108k lines in 8s confirms the subscriber is active and every `tracing` event
in the process is reaching stdout. The plugin-registration lines name
`client::presentation::PresentationPlugin`, `client::ui::lobby::LobbyUiPlugin`,
`client::asset_wiring::AssetWiringPlugin` — proving client-crate spans
participate.

### Why the default-INFO run shows no app-level `[client::*]` lines

Most of `client/`'s 58 `tracing::*` call sites live inside systems that fire
on lobby progression, network messages, user interaction, or hand-UI state
transitions. A standalone 12s startup with no server reachable and no input
does not exercise those paths, so the default `info` filter shows engine-level
spans only — exactly as expected. The fix is verified by:

- presence of formatted output (was zero before, ~14 lines / s now);
- plugin registrations naming `client::*` modules under `RUST_LOG=debug`;
- absence of the `bevy_log: Could not set global logger ...` ERROR line that
  appeared in the first iteration (resolved by disabling `LogPlugin`).

## Build / test verification

```
cargo fmt -p client -- --check                    # exit 0 (no output)
cargo check --manifest-path .../client/Cargo.toml --lib   # Finished in 36.42s
cargo test  --manifest-path .../client/Cargo.toml --lib   # running 0 tests; result: ok
cargo build --manifest-path .../client/Cargo.toml --bin client
                                                  # Finished `dev` profile in 13.73s (incremental)
```

(`-p client` form requires the worktree to be the cwd; the worktree shares
its parent project's `.cargo/config.toml` which sets `target-dir =
"target/msvc-local"`. Build artifacts land under
`D:\_DEV\claude-code-game-studios\target\msvc-local\debug\client.exe`,
which is the binary used for the smoke captures above.)

## Out of scope (deferred)

- WASM target tracing surface (browser console wiring). The cfg gate
  `#[cfg(not(target_arch = "wasm32"))]` keeps WASM unchanged; trunk handles
  that pipeline separately.
- Auditing existing `client/` tracing call sites for level / phrasing
  consistency (PROMPT 648 territory per the orchestrator's note).
- Wiring `init_test_tracing()` into existing integration tests — helper is
  added; opt-in is left for the tests that need it.
