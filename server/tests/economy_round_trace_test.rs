use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{EconomyPlugin, InterestSnapshots, PlayerEconomies, S2CGoldUpdate};
use server::core::rsm::{DraftStarted, RsmPlugin, SessionReady};
use server::core::session::SessionConfig;
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode};
use shared::session::PlayerId;

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
    }
}

fn write_draft_started(app: &mut App, round: u32, phase: DraftPhase) {
    app.world_mut().write_message(DraftStarted { round, phase });
    app.update();
}

fn gold_updates_seen(app: &App) -> usize {
    let messages = app.world().resource::<Messages<S2CGoldUpdate>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).count()
}

#[test]
fn test_economy_round_trace_rounds_one_to_three() {
    let players = [player(1), player(2)];
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(EconomyPlugin);
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(&players));

    app.world_mut().trigger(SessionReady);
    app.update();

    {
        let economies = app.world().resource::<PlayerEconomies>();
        for player in players {
            let economy = economies.0.get(&player).expect("player economy exists");
            assert_eq!(economy.current_mana, 1);
            assert_eq!(economy.gold, 5);
        }
    }

    app.world_mut()
        .resource_mut::<InterestSnapshots>()
        .0
        .extend([(player(1), 5), (player(2), 8)]);
    write_draft_started(&mut app, 2, DraftPhase::Shop);

    {
        let economies = app.world().resource::<PlayerEconomies>();
        assert_eq!(
            economies.0.get(&player(1)).expect("player 1").current_mana,
            2
        );
        assert_eq!(
            economies.0.get(&player(2)).expect("player 2").current_mana,
            2
        );
        assert_eq!(economies.0.get(&player(1)).expect("player 1").gold, 8);
        assert_eq!(economies.0.get(&player(2)).expect("player 2").gold, 8);
    }

    app.world_mut()
        .resource_mut::<InterestSnapshots>()
        .0
        .extend([(player(1), 8), (player(2), 10)]);
    write_draft_started(&mut app, 3, DraftPhase::Auction);

    let economies = app.world().resource::<PlayerEconomies>();
    assert_eq!(
        economies.0.get(&player(1)).expect("player 1").current_mana,
        3
    );
    assert_eq!(
        economies.0.get(&player(2)).expect("player 2").current_mana,
        3
    );
    assert_eq!(economies.0.get(&player(1)).expect("player 1").gold, 11);
    assert_eq!(economies.0.get(&player(2)).expect("player 2").gold, 12);
    assert_eq!(gold_updates_seen(&app), 6);
}
