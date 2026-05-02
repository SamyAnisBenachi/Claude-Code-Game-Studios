use super::events::{
    AbortAuction, AuctionPhaseEntered, AuctionSettled, BeginResolution, BroadcastPhaseChanged,
    DraftReadySignal, DraftStarted, GameOverEmitted, LobbyComplete, PlacementPhaseEntered,
    PlacementSubmitted, PlayerDisconnected, PlayerHeartbeat, PlayerReconnected, ResolutionComplete,
    ResolutionPhaseEntered, ShopRefreshTriggered,
};
use super::state::{PendingPhaseAdvance, RoundState};
use super::transitions::{
    advance_phase, on_lightyear_connected, on_lightyear_disconnected, rsm_input_reader,
    tick_disconnect_timers, tick_rsm_timers,
};
use crate::core::objective_contract::ObjectiveCounters;
use bevy::prelude::*;

pub struct RsmPlugin;

impl Plugin for RsmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoundState::new())
            .init_resource::<PendingPhaseAdvance>()
            .init_resource::<ObjectiveCounters>()
            .init_resource::<super::events::RsmNetworkOutbox>()
            .add_message::<LobbyComplete>()
            .add_message::<DraftStarted>()
            .add_message::<ShopRefreshTriggered>()
            .add_message::<AuctionPhaseEntered>()
            .add_message::<AbortAuction>()
            .add_message::<PlacementPhaseEntered>()
            .add_message::<ResolutionPhaseEntered>()
            .add_message::<BeginResolution>()
            .add_message::<GameOverEmitted>()
            .add_message::<BroadcastPhaseChanged>()
            .add_message::<AuctionSettled>()
            .add_message::<ResolutionComplete>()
            .add_message::<PlayerDisconnected>()
            .add_message::<PlayerReconnected>()
            .add_message::<PlayerHeartbeat>()
            .add_message::<DraftReadySignal>()
            .add_message::<PlacementSubmitted>()
            .add_systems(
                Update,
                (
                    tick_disconnect_timers,
                    rsm_input_reader,
                    tick_rsm_timers,
                    advance_phase,
                )
                    .chain(),
            )
            .add_observer(on_lightyear_connected)
            .add_observer(on_lightyear_disconnected);
    }
}
