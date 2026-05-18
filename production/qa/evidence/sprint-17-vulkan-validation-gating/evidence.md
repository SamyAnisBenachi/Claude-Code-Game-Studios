# S17-OPS-VULKAN-VALIDATION-GATING-001 -- /dev-story Worker Evidence

> **Story**: `production/epics/devops/story-007-vulkan-validation-gating.md`
> **Source audit**: AUDIT-1076-18 (P3) -- 3 Vulkan validation-layer
> warnings on every client launch when `VK_LAYER_KHRONOS_validation`
> is absent on the host.
> **Sprint**: Sprint 17, Nice to Have row.
> **Activation HEAD**: `origin/main@ff47075` (PROMPT 1100 Sprint 17
> QA plan tip).
> **Worker branch**: `work/s17-vulkan-validation-gating` from `ff47075`.
> **Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s17-vulkan-validation-gating`.
> **/dev-story prompt**: PROMPT 1103.
> **Skill activation**: `liv-bevy-018` (mandatory). `liv-bevy-lightyear`
> not activated (no networking edits).

---

## Strategy Chosen

`--features wgpu-validation` Cargo feature gate, OFF by default.

### Justification

The story allowed either `cfg!(debug_assertions)` or a
`--features wgpu-validation` feature gate. AC1 requires **zero**
`VK_LAYER_KHRONOS_validation` warnings on `cargo build -p client`
(default dev/debug profile) and `trunk build` (default dev WASM).
The `cfg!(debug_assertions)` strategy keeps the validation flag ON
in dev builds, which is the same condition that produced the audit
finding -- it would not satisfy AC1 against the audit's exact
reproduction profile. The feature gate is the only strategy that
unambiguously satisfies AC1 *and* preserves the opt-in path (AC2)
via `cargo build -p client --features wgpu-validation`.

This also matches the Sprint 17 plan row wording exactly: "gated
on a cargo feature so prod / CI logs stay clean".

---

## Owned-File Change Set

| File | Change |
|---|---|
| `client/Cargo.toml` | Added `wgpu-validation = []` Cargo feature (empty deps list -- only a `cfg(feature = "wgpu-validation")` switch). |
| `client/src/main.rs` | Imported `bevy::render::{RenderPlugin, settings::{InstanceFlags, RenderCreation, WgpuSettings}}`. Added a `RenderPlugin` `.set(...)` override that constructs `WgpuSettings { instance_flags, ..default() }` where `instance_flags = InstanceFlags::from_build_config()` if `cfg!(feature = "wgpu-validation")` and `InstanceFlags::empty()` otherwise. |

No edits to `server/`, `shared/`, `tests/`, workspace `Cargo.toml`,
`.github/`, `.cargo/`, `Trunk.toml`, `production/sprints/`,
`production/sprint-status.yaml`, `production/stage.txt`,
`production/session-state/`, `production/qa/qa-plan-*`,
`production/qa/smoke-*`, `production/qa/team-qa-*`,
`production/gate-checks/`, any other `production/epics/*` story file,
or `docs/architecture/adr-*.md`.

---

## Build Gate (Cargo Resource Policy Applied)

Per AC10, every `cargo` invocation set:

```
$env:CARGO_TARGET_DIR     = 'D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG  = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL    = '0'
$env:RUSTFLAGS            = '-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

D: free space was >= 760 GB at the start of the run (pre-flight check
satisfied; no stale-target cleanup required).

### cargo check -p client (default features)

```
=== POLICY APPLIED ===
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
=== cargo check -p client ===
    Checking client v0.1.0 (D:\_DEV\claude-code-game-studios-worktrees\s17-vulkan-validation-gating\client)
    Finished `dev` profile [optimized] target(s) in 1.15s
=== EXIT 0 ===
```

PASS. Zero new warnings on the touched files.

### cargo build -p client (default features)

```
   Compiling client v0.1.0 (...\s17-vulkan-validation-gating\client)
    Finished `dev` profile [optimized] target(s) in 1m 05s
```

PASS.

### cargo build -p client --features wgpu-validation

```
   Compiling client v0.1.0 (...\s17-vulkan-validation-gating\client)
    Finished `dev` profile [optimized] target(s) in 2m 06s
```

PASS.

---

## Launch-Log Evidence (AC1 + AC2 + AC4)

GUI client launched non-interactively via PowerShell `Start-Process`
+ 8-second `Stop-Process` timeout (the client has no clean shutdown
signal; we only need wgpu instance-init and DefaultPlugins startup
logs which all fire in the first ~3 seconds). stderr was empty in
both runs (tracing-subscriber routes everything to stdout per
`client/src/main.rs` lines 35-48); stdout was captured.

The host does NOT have `VK_LAYER_KHRONOS_validation` installed --
that is the exact condition under which AUDIT-1076-18 reproduced
the warnings, and it is the dev/test/end-user-typical Windows
configuration the audit recommendation targeted.

### AC1: default debug launch -- log file `debug-default-launch.log`

```
INFO bevy_diagnostic::system_information_diagnostics_plugin::internal: SystemInfo { ... NVIDIA GeForce RTX 5090 Laptop GPU ... }
INFO bevy_render::renderer: AdapterInfo { name: "NVIDIA GeForce RTX 5090 Laptop GPU", ..., backend: Vulkan }
INFO client::audio: AudioSystemPlugin loaded
INFO client::network: ClientNetworkPlugin loaded
...
INFO bevy_winit::system: Creating new window Lanes and Lies (0v0)
```

Grep:

```
Select-String -Pattern 'VK_LAYER_KHRONOS_validation|InstanceFlags::VALIDATION' debug-default-launch.log
match count: 0
```

**ZERO** Vulkan validation-layer warnings. AC1 satisfied.

AC4 satisfied too: `bevy_render::renderer: AdapterInfo` confirms
the Vulkan backend still initialised normally, the adapter (RTX
5090) was selected, the primary window was created, and every
client plugin (HUD / hand / shop / lobby / audio / network /
presentation / asset wiring / qa snapshot / settings) reported
"loaded" without panic. The renderer continues to function; only
the unused validation-layer request was removed.

### AC2: opt-in launch -- log file `debug-feature-wgpu-validation-launch.log`

Built with `cargo build -p client --features wgpu-validation`.

```
INFO bevy_diagnostic::system_information_diagnostics_plugin::internal: SystemInfo { ... }
WARN wgpu_hal::vulkan::instance: InstanceFlags::VALIDATION requested, but unable to find layer: VK_LAYER_KHRONOS_validation
WARN wgpu_hal::vulkan::instance: GENERAL [Loader Message (0x0)]
    windows_read_data_files_in_registry: Registry lookup failed to get layer manifest files.
WARN wgpu_hal::vulkan::instance:     objects: (type: INSTANCE, hndl: ..., name: ?)
INFO bevy_render::renderer: AdapterInfo { ... backend: Vulkan }
```

Grep:

```
Select-String -Pattern 'VK_LAYER_KHRONOS_validation|InstanceFlags::VALIDATION' debug-feature-wgpu-validation-launch.log
match count: 1
```

The `InstanceFlags::VALIDATION requested` line is present, exactly
as expected for the opt-in path on a machine where the validation
layer is not installed. On a machine with the layer installed, this
would produce real validation diagnostics instead. AC2 satisfied.

---

## Acceptance-Criteria Mapping

| AC | Evidence |
|---|---|
| AC1 -- default build emits zero VK validation warnings | `debug-default-launch.log` grep match count = 0 |
| AC2 -- opt-in restores validation request | `debug-feature-wgpu-validation-launch.log` grep match count = 1 |
| AC3 -- Sprint 17 smoke confirms zero warnings | Deferred to Sprint 17 smoke prompt (BLOCKING gate; NOT this worker's scope) |
| AC4 -- WGPU plugin still functions | Both launch logs show `AdapterInfo` Vulkan + `Creating new window Lanes and Lies` + every client plugin loaded |
| AC5 -- no workspace Cargo dependency change | `git diff` only touches `client/Cargo.toml` (new client-local feature) -- workspace root `Cargo.toml` untouched |
| AC6 -- no protocol or server change | `git diff` shows zero changes under `server/`, `shared/`, `tests/integration/server/` |
| AC7 -- no accept-risk closure claimed | Commit message + this evidence file claim only AUDIT-1076-18; explicitly NOT closing S8-QA-001-W1, QA-COND-0005, QA-COND-0006, PAW-TD-*-a, any other AUDIT-1076-* finding, or any SOURCE-1077-* finding |
| AC8 -- Sprint 17 disposition preserved | Worker touched zero files under `production/sprint-status.yaml`, `production/sprints/`, `production/stage.txt`, `production/session-state/`, `production/qa/qa-plan-*`, `production/gate-checks/`, `docs/architecture/adr-*.md` |
| AC9 -- worker branch scope contained | Branch `work/s17-vulkan-validation-gating` from `ff47075`; pushes worker branch only |
| AC10 -- Cargo resource policy applied | Every cargo invocation ran under the five env vars above |

---

## Out-of-Scope Reminder

This worker does NOT close any AUDIT-1076-* finding outside
AUDIT-1076-18, does NOT close any SOURCE-1077-* finding, does NOT
close any of the 24 PROMPT 1022 audit findings, does NOT close
`S11-HUD-TIMER-EYEBALL-VISUAL-001`, does NOT remove the WGPU
validation capability (the opt-in path remains), does NOT advance
`QA-COND-0005`, `QA-COND-0006`, or `PAW-TD-*-a`, does NOT claim
release readiness, accessibility completion, playtest validation,
two-client GAME_OVER closure, final-art completion, Polish->Release
gate-check retry, or stage advance. Sprint 16 disposition
`closed-with-conditions` preserved. PROMPT 761 Polish->Release
gate-check `FAIL` preserved.
