use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::PeerId;
use server::core::rsm::{
    AuctionSettled, BroadcastPhaseChanged, DraftReadySignal, ResolutionComplete, RoundPhase,
    RoundState, RsmPlugin,
};
use server::core::session::{GameSessionPlugin, SessionConfig};
use server::foundation::config::GameConfig;
use server::foundation::rng::ServerRng;
use server::network::resolve_signal_ready_sender;
use shared::card::ClassId;
use shared::protocol::{C2SSignalReady, GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config(players: &[PlayerId]) -> SessionConfig {
    let mut team_map = HashMap::new();
    let mut class_map = HashMap::new();

    for (index, player) in players.iter().copied().enumerate() {
        team_map.insert(player, index as u8);
        class_map.insert(player, ClassId::Iop);
    }

    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: players.len() as u8,
        team_map,
        class_map,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
    }
}

fn app_with_rsm(phase: RoundPhase, round_number: u32) -> App {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&players));
    app.insert_resource(ServerRng::new());
    app.insert_resource(Time::<()>::default());
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase,
        round_number,
        ..RoundState::new()
    };
    app
}

fn run_once(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);
    app.update();
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn test_c2s_signal_ready_maps_peer_to_draft_ready_signal() {
    test_helpers::init_test_tracing();
    let ready_peer = PeerId::Netcode(11);
    let retract_peer = PeerId::Netcode(12);
    let connections = server::core::session::PlayerConnectionMap(HashMap::from([
        (ready_peer, player(1)),
        (retract_peer, player(2)),
    ]));

    let ready =
        resolve_signal_ready_sender(&connections, ready_peer, C2SSignalReady { retract: false })
            .expect("mapped peer should produce RSM ready input");
    let retract =
        resolve_signal_ready_sender(&connections, retract_peer, C2SSignalReady { retract: true })
            .expect("mapped peer should produce RSM retract input");

    assert_eq!(ready.player, player(1));
    assert!(ready.ready);
    assert_eq!(retract.player, player(2));
    assert!(!retract.ready);
    assert!(resolve_signal_ready_sender(
        &connections,
        PeerId::Netcode(99),
        C2SSignalReady { retract: false },
    )
    .is_none());
}

#[test]
fn test_draft_initial_live_ready_advances_only_after_all_players_ready() {
    test_helpers::init_test_tracing();
    let mut app = app_with_rsm(RoundPhase::DraftInitial, 1);
    app.world_mut()
        .resource_mut::<RoundState>()
        .draft_initial_timer = Some(Timer::from_seconds(45.0, TimerMode::Once));

    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: true,
    });
    run_once(&mut app);
    assert_eq!(
        app.world().resource::<RoundState>().phase,
        RoundPhase::DraftInitial
    );

    app.world_mut().write_message(DraftReadySignal {
        player: player(2),
        ready: true,
    });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(rsm.draft_initial_timer.is_none());
    assert!(rsm.placement_timer.is_some());
    assert_eq!(
        read_messages::<BroadcastPhaseChanged>(&app)
            .last()
            .map(|message| message.phase),
        Some(RoundPhase::Placement)
    );
}

#[test]
fn test_draft_shop_live_retract_blocks_advance_until_player_ready_again() {
    test_helpers::init_test_tracing();
    let mut app = app_with_rsm(RoundPhase::DraftShop, 2);
    app.world_mut()
        .resource_mut::<RoundState>()
        .draft_shop_timer = Some(Timer::from_seconds(30.0, TimerMode::Once));

    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: true,
    });
    run_once(&mut app);
    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: false,
    });
    run_once(&mut app);
    app.world_mut().write_message(DraftReadySignal {
        player: player(2),
        ready: true,
    });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert!(!rsm.draft_ready_players.contains(&player(1)));
    assert!(rsm.draft_ready_players.contains(&player(2)));

    app.world_mut().write_message(DraftReadySignal {
        player: player(1),
        ready: true,
    });
    run_once(&mut app);

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert!(rsm.draft_shop_timer.is_none());
}
