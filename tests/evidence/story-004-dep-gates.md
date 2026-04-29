# Story 004 — Dependency Gate Evidence

**Gate level**: BLOCKING (Integration story)

## shared/ dep tree
**Command**: `cargo tree -p shared --prefix none`
**Assert**: No `bevy_ecs`, `bevy_render`, `bevy_ui`, `bevy_winit`, `tokio`
<!-- Paste output here -->
STATUS: [ ] Not yet collected

## client/ dep tree
**Command**: `cargo tree -p client --prefix none`
**Assert**: No `tokio`, `rand_chacha`
<!-- Paste output here -->
STATUS: [ ] Not yet collected

## server/ dep tree
**Command**: `cargo tree -p server --prefix none`
**Assert**: No `bevy_render`, `bevy_ui`, `bevy_winit`
<!-- Paste output here -->
STATUS: [ ] Not yet collected
