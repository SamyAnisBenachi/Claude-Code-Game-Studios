use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use server::core::rsm::{
    BroadcastPhaseChanged, PhaseAdvanceRequest, RoundPhase, RoundState, RsmPlugin,
};
use server::core::session::SessionConfig;
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::{GameMode, PlacementTimerMultiplier};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config(multiplier: PlacementTimerMultiplier) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player(1), 0), (player(2), 1)]),
        class_map: HashMap::from([(player(1), ClassId::Iop), (player(2), ClassId::Cra)]),
        placement_timer_multiplier_effective: multiplier,
    }
}

fn game_config() -> GameConfig {
    let mut config = shared::config::GameConfig::default();
    config.placement_timer_seconds = 10;
    config.auction_followup_placement_timer_seconds = 12;
    GameConfig(config)
}

fn app_with_rsm(phase: RoundPhase, round_number: u32, multiplier: PlacementTimerMultiplier) -> App {
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.insert_resource(Time::<()>::default());
    app.insert_resource(game_config());
    app.insert_resource(session_config(multiplier));
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase,
        round_number,
        ..RoundState::new()
    };
    app
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn rsm_standard_placement_phase_uses_effective_multiplier_in_timer_and_broadcast() {
    let mut app = app_with_rsm(RoundPhase::DraftInitial, 1, PlacementTimerMultiplier::X3);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::DraftInitial));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert_eq!(
        rsm.placement_timer
            .as_ref()
            .expect("placement timer should be created")
            .duration(),
        Duration::from_millis(30_000)
    );
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::Placement);
    assert_eq!(broadcasts[0].timer_ms, 30_000);
}

#[test]
fn rsm_auction_followup_placement_uses_followup_base_and_effective_multiplier() {
    let mut app = app_with_rsm(RoundPhase::DraftShop, 3, PlacementTimerMultiplier::X1_5);
    app.insert_resource(PhaseAdvanceRequest::new(RoundPhase::DraftShop));

    app.update();

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::Placement);
    assert_eq!(
        rsm.placement_timer
            .as_ref()
            .expect("placement timer should be created")
            .duration(),
        Duration::from_millis(18_000)
    );
    let broadcasts = read_messages::<BroadcastPhaseChanged>(&app);
    assert_eq!(broadcasts.len(), 1);
    assert_eq!(broadcasts[0].phase, RoundPhase::Placement);
    assert_eq!(broadcasts[0].timer_ms, 18_000);
}
