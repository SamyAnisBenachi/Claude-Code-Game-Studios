use super::events::{
    AuctionPhaseEntered, AuctionSettled, BroadcastPhaseChanged, DraftStarted, GameOverEmitted,
    LobbyComplete, PlacementPhaseEntered, ResolutionComplete, ResolutionPhaseEntered, SessionReady,
    ShopRefreshNeeded,
};
use super::state::RoundState;
use super::transitions::advance_phase;
use bevy::prelude::*;

pub struct RsmPlugin;

impl Plugin for RsmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoundState::new())
            .add_message::<LobbyComplete>()
            .add_message::<DraftStarted>()
            .add_message::<ShopRefreshNeeded>()
            .add_message::<AuctionPhaseEntered>()
            .add_message::<PlacementPhaseEntered>()
            .add_message::<ResolutionPhaseEntered>()
            .add_message::<GameOverEmitted>()
            .add_message::<BroadcastPhaseChanged>()
            .add_message::<AuctionSettled>()
            .add_message::<ResolutionComplete>()
            .add_systems(Update, advance_phase)
            .add_observer(on_session_ready);
    }
}

fn on_session_ready(_: On<SessionReady>) {}
