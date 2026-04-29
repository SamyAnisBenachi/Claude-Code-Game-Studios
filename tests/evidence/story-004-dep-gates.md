# Story 004 — Dependency Gate Evidence

**Gate level**: BLOCKING (Integration story)
**CI Run**: 25130998038 — commit `6bdee76` / `88971ec`
**Date verified**: 2026-04-29
**Status**: ALL PASS (CI green)

## shared/ dep tree

**Command**: `cargo tree -p shared --prefix none`
**Assert**: No `bevy_ecs`, `bevy_render`, `bevy_ui`, `bevy_winit`, `tokio`

**Result**: PASS — CI green as of commit `88971ec`.

Commit history confirms the gate is meaningful:
- Commit `865a138` still had `bevy` in `shared/Cargo.toml` → bevy_ecs appeared in shared dep tree → CI correctly failed.
- Commit `88971ec` removed bevy from `shared/` → gate passes.

STATUS: [x] PASS — verified via CI run 25130998038

---

## client/ dep tree

**Command**: `cargo tree -p client --prefix none`
**Assert**: No `tokio`, `rand_chacha`

**Result**: PASS — CI green as of commit `88971ec` / CI run 25130998038.
Neither `tokio` nor `rand_chacha` appear in the client dependency tree.
Client uses Bevy (2D features) + lightyear client features only.

STATUS: [x] PASS — verified via CI run 25130998038

---

## server/ dep tree

**Command**: `cargo tree -p server --prefix none`
**Assert**: No `bevy_render`, `bevy_ui`, `bevy_winit`

**Result**: PASS — CI green as of commit `88971ec` / CI run 25130998038.
Server is configured headless: `bevy = { default-features = false, features = ["multi_threaded"] }`.
No render, UI, or winit crates appear in the server tree.

STATUS: [x] PASS — verified via CI run 25130998038

---

## Notes

Local builds blocked by Smart App Control on the dev machine.
All gate results sourced from GitHub Actions CI (authoritative).
CI URL: https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/actions/runs/25130998038
