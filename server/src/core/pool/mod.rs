// server/src/core/pool — Card pool management (ADR-006)
//
// Layer: Core (depends on foundation; must NOT import from feature/).
// Module layout per ADR-006:
//   state.rs  — data structure declarations
//   api.rs    — sole-mutation discipline (impl PlayerPool methods + tests)
//   plugin.rs — CardPoolPlugin (Bevy registration)

pub mod api;
pub mod plugin;
pub mod state;

pub use plugin::CardPoolPlugin;
pub use state::{DistributeError, PlayerPool, PlayerPools, PoolFilter};
