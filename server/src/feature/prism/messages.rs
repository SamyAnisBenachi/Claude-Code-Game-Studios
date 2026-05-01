use bevy::prelude::Message;
use shared::session::PlayerId;

/// Internal board-to-prism collection signal.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrismCollected {
    pub player_id: PlayerId,
    /// One-indexed lane number.
    pub lane: u8,
}
