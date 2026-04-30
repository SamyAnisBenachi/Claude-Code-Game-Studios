// server/src/core/economy -- Economy state and pure API.

pub mod api;
pub mod state;

pub use api::{
    add_reserve, apply_gold_award, apply_spend, can_afford_bid, can_afford_shop,
    discard_current_mana, increment_mana_cap, release_gold_reservation, reserve_gold,
    total_effective_mana, validate_spend,
};
pub use state::{InterestSnapshots, PlayerEconomies, PlayerEconomy, SpendError};
