use std::collections::{HashMap, HashSet};

use bevy::prelude::Resource;
use shared::card::CardId;
use shared::session::PlayerId;

pub const SHOP_SLOT_COUNT: usize = 3;

/// Server-authoritative per-player shop state.
///
/// `card_acquisition_tick_system` is the only system that may hold
/// `ResMut<ShopStates>`.
#[derive(Resource, Default)]
pub struct ShopStates {
    pub players: HashMap<PlayerId, PlayerShopState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerShopState {
    pub phase: ShopPhase,
    pub displayed_this_draft: HashSet<CardId>,
    pub current_slots: [Option<CardId>; SHOP_SLOT_COUNT],
    pub refresh_count_this_draft: u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopPhase {
    #[default]
    Inactive,
    DraftInitial,
    AuctionLock,
    ShopActive,
}

impl Default for PlayerShopState {
    fn default() -> Self {
        Self {
            phase: ShopPhase::Inactive,
            displayed_this_draft: HashSet::new(),
            current_slots: [None; SHOP_SLOT_COUNT],
            refresh_count_this_draft: 0,
        }
    }
}

impl ShopStates {
    pub fn player_state_mut(&mut self, player: PlayerId) -> &mut PlayerShopState {
        self.players.entry(player).or_default()
    }

    pub fn phase_for(&self, player: PlayerId) -> ShopPhase {
        self.players
            .get(&player)
            .map(|state| state.phase)
            .unwrap_or(ShopPhase::Inactive)
    }
}

impl PlayerShopState {
    pub fn displays_card(&self, card_id: CardId) -> bool {
        self.current_slots
            .iter()
            .any(|slot| slot.is_some_and(|displayed| displayed == card_id))
    }
}
