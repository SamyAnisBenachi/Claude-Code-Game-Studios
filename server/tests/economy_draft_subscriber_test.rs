use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{
    EconomyPlugin, InterestSnapshots, PlayerEconomies, PlayerEconomy, S2CGoldBroadcast,
    S2CGoldUpdate,
};
use server::core::rsm::{AuctionSettled, DraftStarted, ResolutionComplete, RsmPlugin};
use server::core::session::{GameSessionPlugin, SessionConfig, SessionReady};
use server::foundation::config::GameConfig;
use server::foundation::rng::ServerRng;
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
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
    }
}

fn economy(gold: u32, current_mana: u32, reserve_mana: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana,
        reserve_mana,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn app_with_economy(players: &[PlayerId]) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_plugins(EconomyPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(players));
    app.insert_resource(ServerRng::new());
    app
}

fn read_gold_updates(app: &App) -> Vec<S2CGoldUpdate> {
    let messages = app.world().resource::<Messages<S2CGoldUpdate>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().collect()
}

fn read_gold_broadcasts(app: &App) -> Vec<S2CGoldBroadcast> {
    let messages = app.world().resource::<Messages<S2CGoldBroadcast>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).copied().collect()
}

#[test]
fn test_economy_draft_plugin_registers_cleanly_in_headless_app() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(EconomyPlugin);
    app.add_message::<AuctionSettled>();
    app.add_message::<ResolutionComplete>();

    app.update();

    assert!(app.world().get_resource::<PlayerEconomies>().is_some());
    assert!(app.world().get_resource::<InterestSnapshots>().is_some());
}

#[test]
fn test_economy_draft_initialises_players_on_session_ready() {
    let players = [player(1), player(2)];
    let mut app = app_with_economy(&players);

    app.world_mut().trigger(SessionReady);
    app.update();

    let economies = app.world().resource::<PlayerEconomies>();
    for player in players {
        let economy = economies.0.get(&player).expect("player economy exists");
        assert_eq!(economy.gold, 5);
        assert_eq!(economy.current_mana, 1);
        assert_eq!(economy.reserve_mana, 0);
        assert_eq!(economy.mana_cap, 10);
        assert_eq!(economy.reserved_gold, 0);
    }
}

#[test]
fn test_economy_draft_round_one_initial_adds_no_gold() {
    let players = [player(1), player(2)];
    let mut app = app_with_economy(&players);
    app.world_mut().trigger(SessionReady);

    app.update();

    let economies = app.world().resource::<PlayerEconomies>();
    for player in players {
        let economy = economies.0.get(&player).expect("player economy exists");
        assert_eq!(economy.gold, 5);
        assert_eq!(economy.current_mana, 1);
    }
}

#[test]
fn test_economy_draft_preserves_reserve_and_applies_round_mana_ramp() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .insert(player(1), economy(5, 0, 7));
    app.world_mut().write_message(DraftStarted {
        round: 2,
        phase: DraftPhase::Shop,
    });

    app.update();

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.reserve_mana, 7);
    assert_eq!(economy.current_mana, 2);
}

#[test]
fn test_economy_draft_applies_baseline_plus_interest_and_clears_snapshot() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .insert(player(1), economy(8, 0, 0));
    app.world_mut()
        .resource_mut::<InterestSnapshots>()
        .0
        .insert(player(1), 8);
    app.world_mut().write_message(DraftStarted {
        round: 2,
        phase: DraftPhase::Shop,
    });

    app.update();

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.gold, 11);
    assert_eq!(economy.current_mana, 2);
    assert!(!app
        .world()
        .resource::<InterestSnapshots>()
        .0
        .contains_key(&player(1)));
}

#[test]
fn test_economy_draft_missing_snapshot_adds_baseline_only() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .insert(player(1), economy(8, 0, 0));
    app.world_mut().write_message(DraftStarted {
        round: 2,
        phase: DraftPhase::Auction,
    });

    app.update();

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.gold, 10);
}

#[test]
fn test_economy_draft_writes_gold_update_and_broadcast_per_player() {
    let players = [player(1), player(2)];
    let mut app = app_with_economy(&players);
    app.world_mut().trigger(SessionReady);

    app.update();

    let updates = read_gold_updates(&app);
    let broadcasts = read_gold_broadcasts(&app);
    assert_eq!(updates.len(), 2);
    assert_eq!(broadcasts.len(), 2);
    assert!(updates.iter().all(|update| update.gold == 5));
    assert!(updates.iter().all(|update| update.current_mana == 1));
}
