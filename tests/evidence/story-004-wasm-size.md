# Story 004 — WASM Bundle Size Evidence

**Command**: `cargo build -p client --target wasm32-unknown-unknown --release`
**Artifact**: `target/wasm32-unknown-unknown/release/client.wasm`
**Budget**: Raw cargo artifact ≤ 100 MB (proxy); Trunk-processed bundle ≤ 50 MB (production budget)
**Gate level**: ADVISORY (size gate is best-effort at mini-spike stage)

## Size measurement
<!-- Run: stat --format="%s" target/wasm32-unknown-unknown/release/client.wasm -->
STATUS: [ ] Not yet collected

## Notes
- Raw `cargo build` output is larger than the final Trunk-processed bundle
- Trunk applies wasm-opt, wasm-bindgen, LTO — final bundle is always smaller
- If raw artifact > 100 MB: investigate with `cargo bloat --target wasm32-unknown-unknown --release`
