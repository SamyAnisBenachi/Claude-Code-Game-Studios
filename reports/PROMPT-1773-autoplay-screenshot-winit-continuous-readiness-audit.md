# PROMPT 1773 — AUTOPLAY-SCREENSHOT-WINIT-CONTINUOUS-READINESS-AUDIT

**Date:** 2026-05-28
**Branch:** `prompt-1773-autoplay-winit-continuous-audit`
**Worktree:** `tmpwt-1773-autoplay-winit-continuous-audit`
**Scope:** Read-only static inspection. No source edits.

---

## 1. Files Inspected

| File | What was checked |
|------|-----------------|
| `client/src/main.rs` | Bevy App setup, `DefaultPlugins`, `WindowPlugin`, presence/absence of `WinitSettings` |
| `client/src/autoplay.rs` | `AutoplayPlugin::build()` — all resource inserts, system registrations |
| `client/Cargo.toml` | `bevy` feature flags, `autoplay-remote` feature definition |
| `tmpwt-1766-screenshot-frame-advance/reports/PROMPT-1766-autoplay-screenshot-frame-advance-repair.md` | Section 5 "Known limitation / follow-up lane" |

---

## 2. Finding: WinitSettings NOT Configured for Autoplay Mode

### 2a. `client/src/main.rs` — No `WinitSettings` override

`main.rs` builds `DefaultPlugins` with only `RenderPlugin` and `WindowPlugin` overrides
(lines 71–88). No `WinitSettings` resource is inserted anywhere in the file:

```rust
// Lines 71–88 — no WinitSettings
let default_plugins = DefaultPlugins
    .build()
    .disable::<LogPlugin>()
    .set(RenderPlugin { ... })
    .set(WindowPlugin {
        primary_window: Some(Window {
            title: "Lanes and Lies".to_string(),
            present_mode: PresentMode::AutoVsync,
            ..default()
        }),
        ..default()
    });
```

Bevy's default when no `WinitSettings` is inserted: **`WinitSettings::desktop_app()`**.
That mode uses `UpdateMode::Reactive` when the window is unfocused — Bevy throttles
`Update` schedule to once every 5 seconds (the default `wait` duration), so
`status.frame` in `publish_status_system` freezes completely between wakeups.

### 2b. `client/src/autoplay.rs` — `AutoplayPlugin::build()` does not insert `WinitSettings`

`AutoplayPlugin::build()` (lines 58–113) checks `cfg.enabled`, starts the RPC thread,
and then does:

```rust
// Lines 109–111 — resource inserts in AutoplayPlugin::build()
app.insert_resource(AutoplayShared::handle(Arc::clone(&shared)))
    .insert_resource(cfg)
    .add_systems(Update, (drain_commands_system, publish_status_system));
```

**`WinitSettings::game()` is not among the inserted resources.** The plugin relies
entirely on whatever `WinitSettings` the app already has — which is
`WinitSettings::desktop_app()` (reactive, throttled).

### 2c. `client/Cargo.toml` — `autoplay-remote` feature is empty

```toml
# Line 56
autoplay-remote = []
```

There is no dependency on `bevy/winit` (or any explicit `bevy_winit` feature) that
would be needed beyond what `"2d"` already pulls in. Since `DefaultPlugins` is used
(which includes `WinitPlugin`), `bevy_winit` is already in the dependency graph and
`WinitSettings` is accessible.

### 2d. PROMPT 1766 explicitly deferred this fix

Section 5 of `PROMPT-1766-autoplay-screenshot-frame-advance-repair.md` states:

> If the Bevy window is minimised or the render pipeline is fully suspended
> (`WinitSettings` unfocused mode), `status.frame` will not advance and the
> barrier will log a warning but still capture a stale screenshot. The correct
> fix in that case is a Rust-side change: configure `WinitSettings::game()` with
> `UpdateMode::Continuous` for both focused and unfocused modes when
> `CCGS_AUTOPLAY=1`. That change requires a `cargo build` cycle and is deferred
> as a follow-up story.

**Verdict: the gap is confirmed and not yet closed.**

---

## 3. Impact Assessment

| Scenario | Current behaviour | After fix |
|----------|------------------|-----------|
| Window focused, foreground | `status.frame` advances at 60 FPS — screenshots fine | No change |
| Window unfocused (another app in front) | Frame counter throttles to ~1 tick per 5 s; screenshot captures same stale frame for up to 5 s | Frame counter runs at 60 FPS regardless of focus |
| Window minimised | Same as unfocused; OS may fully suspend rendering | Frame counter and `drain_commands_system` continue running; screenshot capture still triggers but GPU blit may be deferred by OS. Note below. |

> **Minimise caveat**: Even with `WinitSettings::game()`, `Screenshot::primary_window()`
> captures the wgpu swapchain surface. Some OS/GPU combinations suppress the swapchain
> when the window is minimised (surface is 0×0). The Python-side frame-advance barrier
> from PROMPT 1766 remains the safety net for that edge case. The Rust fix eliminates
> the unfocused-but-visible freeze; it does not guarantee captures when truly minimised.
> Autoplay smoke runs should keep the window visible (normal CI practice).

---

## 4. Recommendation: FOLLOW-UP PROMPT REQUIRED

**Status: NOT a NOOP.** A concrete Rust-side change is needed.

### Minimal fix

**File:** `client/src/autoplay.rs`
**Function:** `AutoplayPlugin::build()` — after the `tracing::info!` call
(line ~107) and before the first `app.insert_resource(...)` call (line 109).

**Change:**

```rust
// Force continuous rendering so status.frame advances even when the
// window is unfocused during automated screenshot runs (PROMPT 1773).
app.insert_resource(bevy::winit::WinitSettings::game());
```

Alternatively, import at the top of `autoplay.rs`:

```rust
use bevy::winit::WinitSettings;
// ...
// in AutoplayPlugin::build():
app.insert_resource(WinitSettings::game());
```

`WinitSettings::game()` sets both `focused_mode` and `unfocused_mode` to
`UpdateMode::Continuous`, which keeps the `Update` schedule running at full rate
regardless of window focus state.

### Import path — verify against Bevy 0.18

The project uses `bevy = { version = "0.18", default-features = false, features = ["2d", "webgl2", "bevy_audio"] }`.
`WinitSettings` is in the `bevy_winit` crate. Since `DefaultPlugins` (which includes
`WinitPlugin`) compiles successfully, `bevy_winit` is already in the graph.

Expected import in Bevy 0.18: `bevy::winit::WinitSettings` — but confirm against
`bevy 0.18` docs (the live migration guide at `bevy.org/learn/migration-guides/0-17-to-0-18/`)
before shipping; the struct may also be re-exported via `bevy::prelude`.

### Risk: LOW

- Scoped behind `CCGS_AUTOPLAY=1` (already dev-only env var).
- Single `insert_resource` call, no game logic.
- Does not affect WASM builds (winit continuous mode is expected there anyway).
- Fully reversible without touching any game systems.

### Tests / Verification

No automated test is possible for windowing behavior. Verification:

1. Build: `cargo run -p client --features autoplay-remote` with `CCGS_AUTOPLAY=1`.
2. Unfocus the Bevy window (click away, or put another window on top).
3. Poll `autoplay/status` every second via the RPC; assert `status.frame` increments
   each poll.
4. Run the full autoplay smoke recipe; confirm screenshots are non-identical
   (existing test evidence approach from PROMPT 1758/1766).

---

## 5. Concrete Follow-Up Prompt

```
PROMPT 1774 -- AUTOPLAY-WINIT-CONTINUOUS-UPDATE-RUST-FIX

Context:
- PROMPT 1773 audit confirmed: AutoplayPlugin does not insert WinitSettings::game().
  When CCGS_AUTOPLAY=1 and the Bevy window is unfocused, the default
  WinitSettings::desktop_app() throttles Update to ~1 tick/5 s. status.frame freezes
  and screenshots capture stale frames (the root cause documented in PROMPT 1766 §5).
- Source of truth: latest origin/main. Use worktree.

Task:
- In client/src/autoplay.rs, AutoplayPlugin::build(), after the tracing::info! call
  and before the first app.insert_resource call: insert WinitSettings::game().
- Verify the import path for WinitSettings in Bevy 0.18 (use bevy::winit::WinitSettings
  or bevy::prelude — confirm compiles).
- Build gate: cargo build -p client --features autoplay-remote (must compile clean).
- No other source edits. No game logic changes.

Owned scope:
- client/src/autoplay.rs only.
- Forbidden: main.rs edits, Cargo.toml changes, production/session/story edits.

Validation:
- cargo build -p client --features autoplay-remote succeeds.
- Commit + push worker branch with PROGRESS.md entry.

Final line exactly: 1774: AUTOPLAY-WINIT-CONTINUOUS-UPDATE-RUST-FIX: <STATUS>
```

---

1773: AUTOPLAY-SCREENSHOT-WINIT-CONTINUOUS-READINESS-AUDIT: SHIPPED
