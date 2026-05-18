// Scaffold API consumed by downstream stories.
#![allow(dead_code)]

use super::state::RoundPhase;
use bevy::prelude::*;
use shared::card::CardId;
use shared::protocol::{DraftPhase, GameOverReason, S2CPhaseChanged};
use shared::session::PlayerId;

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

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameOverEmitted {
    pub reason: GameOverReason,
    pub loser: Option<PlayerId>,
    pub round: u32,
}

#[derive(Message, Clone, Debug)]
pub struct BroadcastPhaseChanged {
    pub phase: RoundPhase,
    pub round: u32,
    pub timer_ms: u32,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct RsmNetworkOutbox {
    phase_changed: Vec<S2CPhaseChanged>,
}

impl RsmNetworkOutbox {
    pub fn push_phase_changed(&mut self, message: S2CPhaseChanged) {
        self.phase_changed.push(message);
    }

    pub fn phase_changed(&self) -> &[S2CPhaseChanged] {
        &self.phase_changed
    }
}

#[derive(Message, Clone, Debug)]
pub struct AuctionSettled {
    pub winner: Option<PlayerId>,
    pub final_price: u32,
    pub card_id: CardId,
}

#[derive(Message, Clone, Debug)]
pub struct ResolutionComplete;

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerDisconnected {
    pub player: PlayerId,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerReconnected {
    pub player: PlayerId,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerHeartbeat {
    pub player: PlayerId,
}

/// Internal RSM-emitted signal that an opponent has entered or is still in
/// disconnect grace. The network dispatch layer consumes this to unicast
/// `S2COpponentDisconnected` to surviving session players (PROMPT 1211).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpponentDisconnectNotice {
    pub player_id: PlayerId,
    pub grace_remaining_ms: u32,
}

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
