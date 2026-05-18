use super::events::{
    AbortAuction, AuctionPhaseEntered, BeginResolution, BroadcastPhaseChanged, DraftReadySignal,
    DraftStarted, GameOverEmitted, LobbyComplete, OpponentDisconnectNotice, PlacementPhaseEntered,
    PlacementSubmitted, PlayerDisconnected, PlayerHeartbeat, PlayerReconnected,
    ResolutionPhaseEntered, ShopRefreshTriggered,
};
use super::state::{PendingPhaseAdvance, RoundState};
use super::transitions::{
    advance_phase, on_lightyear_connected, on_lightyear_disconnected, rsm_input_reader,
    tick_disconnect_timers, tick_rsm_timers,
};
use crate::core::objective_contract::ObjectiveCounters;
use bevy::prelude::*;

/// Round State Machine schedule labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RsmSet {
    Tick,
}

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
            .add_message::<PlayerDisconnected>()
            .add_message::<PlayerReconnected>()
            .add_message::<PlayerHeartbeat>()
            .add_message::<OpponentDisconnectNotice>()
            .add_message::<DraftReadySignal>()
            .add_message::<PlacementSubmitted>()
            // Full transition chain:
            // AuctionSystem -> CombatResolutionSystem -> rsm_input_reader ->
            // advance_phase -> [subscriber systems] -> network::dispatch_phase_changed.
            // Network dispatch is registered in ServerNetworkPlugin so core RSM
            // never imports Lightyear send APIs.
            .add_systems(
                Update,
                (
                    tick_disconnect_timers,
                    rsm_input_reader,
                    tick_rsm_timers,
                    advance_phase,
                )
                    .chain()
                    .in_set(RsmSet::Tick),
            )
            .add_observer(on_lightyear_connected)
            .add_observer(on_lightyear_disconnected);
    }
}
