use bevy::prelude::*;
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
    pub submissions_received: HashSet<PlayerId>,
    pub disconnect_trackers: HashMap<PlayerId, f32>,
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
            submissions_received: HashSet::new(),
            disconnect_trackers: HashMap::new(),
        }
    }
}

impl Default for RoundState {
    fn default() -> Self {
        Self::new()
    }
}
