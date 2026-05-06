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

pub use hands::{hand_push, HandFullError, PlayerHands, MAX_HAND_SIZE};
pub use messages::{ShopRefreshTrigger, ShopRefreshTriggered};
pub use plugin::CardAcquisitionPlugin;
pub use state::{PlayerShopState, ShopPhase, ShopStates};
pub use system::{
    apply_shop_refresh_trigger, build_auto_shop_slots, build_draft_initial_offering,
    build_manual_shop_slots, card_acquisition_tick_system, defer_card_acquired,
    defer_draft_offering, defer_shop_slots, economy_gold_update_for_player, manual_refresh_cost,
    prepare_card_acquired_dispatch, prepare_draft_offering_dispatch, prepare_shop_slots_dispatch,
    process_manual_refresh_shop_request, process_purchase_card, process_purchase_card_with_pool,
    process_refresh_shop_request, purchase_card_source, purchase_network_events_for_result,
    CardAcquiredDispatch, CardAcquisitionSet, DraftOfferingDispatch, PurchaseAttemptResult,
    PurchaseNetworkEvents, PurchasePool, RefreshAttemptResult, ShopSlotsDispatch,
    DRAFT_INITIAL_OFFERING_COUNT, SHOP_DEDUP_RETRY_LIMIT,
};
