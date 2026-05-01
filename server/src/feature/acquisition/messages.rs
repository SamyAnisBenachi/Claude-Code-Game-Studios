use bevy::prelude::Message;
use shared::session::PlayerId;

/// Internal RSM -> Card Acquisition trigger.
///
/// This is a Bevy buffered message, not a Lightyear network message.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShopRefreshTriggered {
    pub player_id: PlayerId,
    pub trigger: ShopRefreshTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopRefreshTrigger {
    /// DRAFT_INITIAL entry: future story draws the 9-card offering.
    DraftInitial,
    /// DRAFT_AUCTION entry: future story draws 3 slots and locks them.
    AuctionLock,
    /// DRAFT_SHOP entry on non-auction rounds: future story draws 3 slots.
    ShopOpen,
    /// DRAFT_AUCTION -> DRAFT_SHOP: unlock existing slots without clearing dedup.
    ShopUnlock,
}
