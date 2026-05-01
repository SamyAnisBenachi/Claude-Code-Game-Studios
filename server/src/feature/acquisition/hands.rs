use std::collections::HashMap;

use bevy::prelude::Resource;
use shared::card::CardId;
use shared::session::PlayerId;

pub const MAX_HAND_SIZE: usize = 10;

/// Server-authoritative player hand state.
///
/// Card Acquisition writes this resource during DRAFT phases. Future Prism and
/// Objective stories also write it during RESOLUTION, which is phase-exclusive.
#[derive(Resource, Default)]
pub struct PlayerHands {
    pub hands: HashMap<PlayerId, Vec<CardId>>,
}

impl PlayerHands {
    pub fn hand_len(&self, player: PlayerId) -> usize {
        self.hands.get(&player).map_or(0, Vec::len)
    }

    pub fn push_card(&mut self, player: PlayerId, card_id: CardId) {
        self.hands.entry(player).or_default().push(card_id);
    }
}
