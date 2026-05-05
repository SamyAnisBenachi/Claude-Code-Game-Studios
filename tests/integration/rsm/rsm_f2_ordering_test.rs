use bevy::prelude::*;
use server::core::rsm::{
    advance_phase, BroadcastPhaseChanged, DraftStarted, ResolutionComplete, RoundPhase, RoundState,
    RsmPlugin, ShopRefreshTriggered,
};
use server::core::session::SessionConfig;
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;
use std::collections::HashMap;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

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
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn record_draft_started(mut messages: MessageReader<DraftStarted>, mut log: ResMut<OrderLog>) {
    for _message in messages.read() {
        log.0.push("DraftStarted");
    }
}

fn record_shop_refresh(
    mut messages: MessageReader<ShopRefreshTriggered>,
    mut log: ResMut<OrderLog>,
) {
    for message in messages.read() {
        match message.player_id.0 {
            1 => log.0.push("ShopRefreshTriggered:p1"),
            2 => log.0.push("ShopRefreshTriggered:p2"),
            _ => log.0.push("ShopRefreshTriggered:other"),
        }
    }
}

fn record_broadcast(mut messages: MessageReader<BroadcastPhaseChanged>, mut log: ResMut<OrderLog>) {
    for _message in messages.read() {
        log.0.push("BroadcastPhaseChanged");
    }
}

fn app_for_draft_entry() -> App {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_plugins(RsmPlugin);
    app.insert_resource(session_config(&players))
        .insert_resource(GameConfig(shared::config::GameConfig::default()))
        .insert_resource(Time::<()>::default())
        .insert_resource(OrderLog::default())
        .add_systems(
            Update,
            (record_draft_started, record_shop_refresh, record_broadcast)
                .chain()
                .after(advance_phase),
        );
    *app.world_mut().resource_mut::<RoundState>() = RoundState {
        phase: RoundPhase::Resolution,
        round_number: 1,
        ..RoundState::new()
    };
    app
}

#[test]
fn rsm_f2_ordering_draft_entry_subscribers_process_broadcast_last() {
    let mut app = app_for_draft_entry();

    app.world_mut().write_message(ResolutionComplete);
    app.update();

    let log = app.world().resource::<OrderLog>();
    assert_eq!(
        log.0,
        vec![
            "DraftStarted",
            "ShopRefreshTriggered:p1",
            "ShopRefreshTriggered:p2",
            "BroadcastPhaseChanged",
        ]
    );
}
