// Scaffold API consumed by downstream stories.
#![allow(dead_code)]

use bevy::prelude::*;
use shared::protocol::GameOverReason;
use shared::session::PlayerId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundPhase {
    Lobby,
    DraftInitial,
    DraftAuction,
    DraftShop,
    Placement,
    Resolution,
    GameOver,
}

/// Server-authoritative RSM state.
///
/// `advance_phase` is the only system that may mutate this resource once
/// transition logic is implemented.
#[derive(Resource)]
pub struct RoundState {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub placement_timer: Option<Timer>,
    pub draft_shop_timer: Option<Timer>,
    pub draft_initial_timer: Option<Timer>,
    pub auction_safety_timer: Option<Timer>,
    pub resolution_safety_timer: Option<Timer>,
    pub draft_ready_players: HashSet<PlayerId>,
    pub submissions_received: HashSet<PlayerId>,
    pub disconnect_trackers: HashMap<PlayerId, f32>,
    pub pending_disconnect_outcome: Option<GameOverRequest>,
}

impl RoundState {
    pub fn new() -> Self {
        Self {
            phase: RoundPhase::Lobby,
            round_number: 0,
            placement_timer: None,
            draft_shop_timer: None,
            draft_initial_timer: None,
            auction_safety_timer: None,
            resolution_safety_timer: None,
            draft_ready_players: HashSet::new(),
            submissions_received: HashSet::new(),
            disconnect_trackers: HashMap::new(),
            pending_disconnect_outcome: None,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct PhaseAdvanceRequest {
    pub expected_source: RoundPhase,
    pub game_over: Option<GameOverRequest>,
}

impl PhaseAdvanceRequest {
    pub fn new(expected_source: RoundPhase) -> Self {
        Self {
            expected_source,
            game_over: None,
        }
    }

    pub fn game_over(
        expected_source: RoundPhase,
        reason: GameOverReason,
        loser: Option<PlayerId>,
    ) -> Self {
        Self {
            expected_source,
            game_over: Some(GameOverRequest { reason, loser }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameOverRequest {
    pub reason: GameOverReason,
    pub loser: Option<PlayerId>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct PendingPhaseAdvance {
    request: Option<PhaseAdvanceRequest>,
}

impl PendingPhaseAdvance {
    pub fn request(&mut self, request: PhaseAdvanceRequest) {
        if self.request.is_none() {
            self.request = Some(request);
        }
    }

    pub fn is_requested(&self) -> bool {
        self.request.is_some()
    }

    pub fn take(&mut self) -> Option<PhaseAdvanceRequest> {
        self.request.take()
    }
}

impl Default for RoundState {
    fn default() -> Self {
        Self::new()
    }
}
