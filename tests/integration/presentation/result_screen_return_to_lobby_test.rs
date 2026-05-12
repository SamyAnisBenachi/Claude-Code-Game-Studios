use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::result_screen::{
    ResultScreenActionRequest, ResultScreenFocusOrder, ResultScreenOutboundMessages,
    ResultScreenPlugin, ResultScreenReturnToLobbyState, ResultScreenRoot, ResultScreenViewState,
};
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameOverReason, ObjectiveSnapshot, PlacementTimerMultiplier, PlayerSnapshot,
    RoundPhase, S2CGameOver, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn duplicate_return_to_lobby_activation_sends_one_ack_and_cleans_local_result_ui() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(&mut app);

    app.world_mut()
        .write_message(ResultScreenActionRequest::ReturnToLobby);
    app.world_mut()
        .write_message(ResultScreenActionRequest::ReturnToLobby);
    app.update();
    app.update();

    assert_eq!(
        app.world()
            .resource::<ResultScreenOutboundMessages>()
            .acknowledgements
            .len(),
        1,
        "duplicate return activations must produce at most one acknowledgement"
    );

    let return_state = *app.world().resource::<ResultScreenReturnToLobbyState>();
    assert!(return_state.return_requested);
    assert!(return_state.acknowledgement_sent);
    assert!(return_state.local_cleanup_completed);

    let view_state = app.world().resource::<ResultScreenViewState>();
    assert!(view_state.cached_result.is_none());
    assert!(view_state.cached_snapshot.is_none());
    assert!(!view_state.visible);
    assert!(!view_state.snapshot_game_over_seen);
    assert_eq!(app.world().resource::<ResultScreenFocusOrder>().len(), 0);
    assert_eq!(
        app.world().resource::<State<ClientState>>().get(),
        &ClientState::Lobby
    );
    assert_eq!(result_screen_root_count(&mut app), 0);
}

#[test]
fn disconnected_transport_fallback_returns_to_lobby_without_mutating_server_phase_view() {
    test_helpers::init_test_tracing();
    let mut app = result_screen_app();
    open_result_screen(&mut app);

    app.world_mut()
        .write_message(ResultScreenActionRequest::ReturnToLobby);
    app.update();
    app.update();

    assert_eq!(
        app.world()
            .resource::<ResultScreenOutboundMessages>()
            .acknowledgements
            .len(),
        1,
        "a missing MessageSender still records one local acknowledgement attempt"
    );
    assert_eq!(
        app.world().resource::<State<ClientState>>().get(),
        &ClientState::Lobby
    );
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::GameOver,
        "return-to-lobby navigation must not overwrite the server-owned phase view"
    );
}

fn result_screen_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ResultScreenPlugin);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.world_mut()
        .resource_mut::<ClientSessionIdentity>()
        .player_id = Some(player(1));
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn open_result_screen(app: &mut App) {
    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(game_over_snapshot()));
    app.world_mut()
        .resource_mut::<ResultScreenViewState>()
        .cached_result = Some(S2CGameOver {
        loser: Some(player(2)),
        round: 9,
        reason: GameOverReason::ObjectivesDestroyed,
    });
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::GameOver;
    app.update();
    app.update();
}

fn game_over_snapshot() -> S2CGameSnapshot {
    let mut own = player_snapshot(player(1));
    own.objectives = vec![objective(1, 0, true, true), objective(2, 4, false, false)];
    let opponent = player_snapshot(player(2));

    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: own.player_id,
        round_number: 9,
        phase: RoundPhase::GameOver,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(player_id: PlayerId) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold: 8,
        reserved_gold: 0,
        current_mana: 6,
        reserve_mana: 2,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
}

fn objective(lane: u8, hp: u8, is_real: bool, is_destroyed: bool) -> ObjectiveSnapshot {
    ObjectiveSnapshot {
        lane,
        hp,
        is_real,
        is_destroyed,
    }
}

fn result_screen_root_count(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<ResultScreenRoot>>();
    query.iter(app.world()).count()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
