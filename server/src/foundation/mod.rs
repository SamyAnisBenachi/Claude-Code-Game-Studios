// Foundation layer: GameConfig loading, ServerRng, asset pipeline

pub mod rng;
// bevy_asset_loader: no 0.18-compatible release verified as of 2026-04-29.
// Latest release supports Bevy 0.16. Upstream PR #264 (0.18 support) is still Draft.
// Fallback: manual AssetServer loading in Epic 2 (game-config-pipeline).
// Re-check crates.io before implementing Epic 2 Story 001.
