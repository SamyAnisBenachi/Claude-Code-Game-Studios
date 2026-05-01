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

#[derive(Message, Clone, Debug)]
pub struct ShopRefreshNeeded {
    pub player: PlayerId,
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
