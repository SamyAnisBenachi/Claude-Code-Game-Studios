use super::events::{
    AbortAuction, AuctionPhaseEntered, AuctionSettled, BroadcastPhaseChanged, DraftReadySignal,
    DraftStarted, GameOverEmitted, LobbyComplete, PlacementPhaseEntered, PlacementSubmitted,
    ResolutionComplete, ResolutionPhaseEntered, ShopRefreshNeeded,
};
use super::state::{PendingPhaseAdvance, RoundState};
use super::transitions::{advance_phase, on_session_ready, rsm_input_reader, tick_rsm_timers};
use bevy::prelude::*;

pub struct RsmPlugin;

impl Plugin for RsmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoundState::new())
            .init_resource::<PendingPhaseAdvance>()
            .add_message::<LobbyComplete>()
            .add_message::<DraftStarted>()
            .add_message::<ShopRefreshNeeded>()
            .add_message::<AuctionPhaseEntered>()
            .add_message::<AbortAuction>()
            .add_message::<PlacementPhaseEntered>()
            .add_message::<ResolutionPhaseEntered>()
            .add_message::<GameOverEmitted>()
            .add_message::<BroadcastPhaseChanged>()
            .add_message::<AuctionSettled>()
            .add_message::<ResolutionComplete>()
            .add_message::<DraftReadySignal>()
            .add_message::<PlacementSubmitted>()
            .add_systems(
                Update,
                (rsm_input_reader, tick_rsm_timers, advance_phase).chain(),
            )
            .add_observer(on_session_ready);
    }
}
