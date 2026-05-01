//! Server-authoritative Card Acquisition scaffold.
//!
//! CA-001 defines state resources, the RSM trigger message, plugin
//! registration, and the minimal phase/hand-cap purchase guard used by later
//! Card Acquisition stories.
#![allow(dead_code, unused_imports)]

pub mod hands;
pub mod messages;
pub mod plugin;
pub mod state;
pub mod system;

pub use hands::{PlayerHands, MAX_HAND_SIZE};
pub use messages::{ShopRefreshTrigger, ShopRefreshTriggered};
pub use plugin::CardAcquisitionPlugin;
pub use state::{PlayerShopState, ShopPhase, ShopStates};
pub use system::{
    apply_shop_refresh_trigger, card_acquisition_tick_system, process_purchase_card,
    process_refresh_shop_request, CardAcquisitionSet, PurchaseAttemptResult, RefreshAttemptResult,
};
