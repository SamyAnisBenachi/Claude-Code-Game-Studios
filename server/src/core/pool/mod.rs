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
pub mod system;

// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use api::refresh_shop;
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use plugin::{CardPoolPlugin, CardPoolSet};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use state::{
    DistributeError, InitialDraftOffering, ManualRefreshCount, PlayerPool, PlayerPools, PoolFilter,
    ShopSlots,
};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use system::{
    clear_pool_session_resources_on_game_over, initialize_player_pools_on_draft_started,
};
