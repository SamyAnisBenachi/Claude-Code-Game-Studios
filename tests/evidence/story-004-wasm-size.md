# Story 004 — WASM Bundle Size Evidence

**Command**: `cargo build -p client --target wasm32-unknown-unknown --release`
**Artifact**: `target/wasm32-unknown-unknown/release/client.wasm`
**Budget**: Raw cargo artifact ≤ 100 MB (proxy); Trunk-processed bundle ≤ 50 MB (production budget)
**Gate level**: ADVISORY (size gate is best-effort at mini-spike stage)
**CI Run**: 25130998038 — commit `6bdee76` / `88971ec`
**Date verified**: 2026-04-29

## Size measurement

**Result**: PASS — wasm-size CI job passed on run 25130998038.

Raw artifact size not measured locally (Smart App Control blocks local builds).
The CI wasm-size job passed, confirming the raw artifact is within the 100 MB proxy budget.
The workspace is at foundation skeleton stage (no game assets, no large feature sets enabled),
so the bundle is well under budget.

**Exact size**: Not measured locally — CI pass is authoritative for this story stage.

STATUS: [x] PASS — CI wasm-size job green on run 25130998038

---

## Notes

- Raw `cargo build` output is larger than the final Trunk-processed bundle
- Trunk applies wasm-opt, wasm-bindgen, LTO — final bundle is always smaller
- At foundation skeleton stage, the client crate has minimal code — size risk is low
- Bundle size should be re-measured when Bevy features and game code are added in Epic 2+
- If raw artifact grows > 100 MB at any future point: investigate with
  `cargo bloat --target wasm32-unknown-unknown --release`
- Local measurement deferred: re-verify once Smart App Control is resolved or WSL2 is used
