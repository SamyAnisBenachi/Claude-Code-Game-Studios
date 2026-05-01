// Scaffold API consumed by downstream stories.
#![allow(dead_code)]

use super::state::RoundPhase;
use bevy::prelude::*;
use shared::card::CardId;
use shared::protocol::{DraftPhase, GameOverReason};
use shared::session::PlayerId;

pub use crate::core::session::SessionReady;

#[derive(Message, Clone, Debug)]
pub struct LobbyComplete;

#[derive(Message, Clone, Debug)]
pub struct DraftStarted {
    pub round: u32,
    pub phase: DraftPhase,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShopRefreshTriggered {
    pub player_id: PlayerId,
    pub trigger: ShopRefreshTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopRefreshTrigger {
    /// DRAFT_INITIAL entry: draw the initial draft offering.
    DraftInitial,
    /// DRAFT_AUCTION entry: draw and lock auction-round shop slots.
    AuctionLock,
    /// DRAFT_SHOP entry on non-auction rounds: draw active shop slots.
    ShopOpen,
    /// DRAFT_AUCTION -> DRAFT_SHOP: unlock existing auction slots.
    ShopUnlock,
}

#[derive(Message, Clone, Debug)]
pub struct AuctionPhaseEntered {
    pub round: u32,
}

#[derive(Message, Clone, Debug)]
pub struct AbortAuction;

#[derive(Message, Clone, Debug)]
pub struct PlacementPhaseEntered {
    pub round: u32,
}

#[derive(Message, Clone, Debug)]
pub struct ResolutionPhaseEntered {
    pub round: u32,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeginResolution {
    pub round: u32,
}

#[derive(Message, Clone, Debug)]
pub struct GameOverEmitted {
    pub reason: GameOverReason,
    pub loser: Option<PlayerId>,
}

#[derive(Message, Clone, Debug)]
pub struct BroadcastPhaseChanged {
    pub phase: RoundPhase,
    pub round: u32,
    pub timer_ms: u32,
}

#[derive(Message, Clone, Debug)]
pub struct AuctionSettled {
    pub winner: Option<PlayerId>,
    pub final_price: u32,
    pub card_id: CardId,
}

#[derive(Message, Clone, Debug)]
pub struct ResolutionComplete;

/// Internal server signal emitted after the network layer resolves the sender
/// to a stable session player. The shared C2S payload stays Bevy-free.
#[derive(Message, Clone, Copy, Debug)]
pub struct DraftReadySignal {
    pub player: PlayerId,
    pub ready: bool,
}

/// Internal server signal emitted after a valid placement submission is accepted
/// by the input layer. Placement contents are owned by Board/Lane stories.
#[derive(Message, Clone, Copy, Debug)]
pub struct PlacementSubmitted {
    pub player: PlayerId,
}
