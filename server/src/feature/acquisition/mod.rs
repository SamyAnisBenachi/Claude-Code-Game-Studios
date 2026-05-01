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
    apply_shop_refresh_trigger, build_auto_shop_slots, build_draft_initial_offering,
    build_manual_shop_slots, card_acquisition_tick_system, manual_refresh_cost,
    prepare_draft_offering_dispatch, prepare_shop_slots_dispatch,
    process_manual_refresh_shop_request, process_purchase_card, process_refresh_shop_request,
    CardAcquisitionSet, DraftOfferingDispatch, PurchaseAttemptResult, RefreshAttemptResult,
    ShopSlotsDispatch, DRAFT_INITIAL_OFFERING_COUNT, SHOP_DEDUP_RETRY_LIMIT,
};
