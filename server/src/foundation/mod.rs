// Foundation layer: GameConfig loading, ServerRng, asset pipeline

pub mod config;
pub mod rng;
// bevy_asset_loader: no 0.18-compatible release verified as of 2026-04-29.
// Latest release supports Bevy 0.16. Upstream PR #264 (0.18 support) is still Draft.
// ADR-004 fallback: manual AssetServer loading is used in config.rs (see ConfigPlugin).
// Re-check crates.io before Story 004 (hot-reload watcher).
