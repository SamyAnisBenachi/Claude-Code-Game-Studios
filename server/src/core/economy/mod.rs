// server/src/core/economy -- Economy state and pure API.

pub mod api;
pub mod plugin;
pub mod state;
pub mod system;

// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use api::{
    add_reserve, apply_gold_award, apply_mana_ramp, apply_spend, can_afford_bid, can_afford_shop,
    discard_current_mana, increment_mana_cap, refund_gold, release_gold_reservation, reserve_gold,
    spend_gold, total_effective_mana, validate_auction_bid, validate_spend,
};
pub use plugin::EconomyPlugin;
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use state::{InterestSnapshots, PlayerEconomies, PlayerEconomy, SpendError};
// Scaffold API consumed by downstream stories.
#[allow(unused_imports)]
pub use system::{
    on_draft_started, on_resolution_complete, AwardGold, EconomySystemSet, ManaCapIncreased,
    S2CGoldBroadcast, S2CGoldUpdate,
};
