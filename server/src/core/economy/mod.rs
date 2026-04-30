// server/src/core/economy -- Economy state and pure API.

pub mod api;
pub mod plugin;
pub mod state;
pub mod system;

pub use api::{
    add_reserve, apply_gold_award, apply_mana_ramp, apply_spend, can_afford_bid, can_afford_shop,
    discard_current_mana, increment_mana_cap, release_gold_reservation, reserve_gold,
    total_effective_mana, validate_spend,
};
pub use plugin::EconomyPlugin;
pub use state::{InterestSnapshots, PlayerEconomies, PlayerEconomy, SpendError};
pub use system::{initialise_player_economies, on_draft_started, S2CGoldBroadcast, S2CGoldUpdate};
