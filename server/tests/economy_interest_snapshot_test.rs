use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{
    apply_gold_award, EconomyPlugin, InterestSnapshots, PlayerEconomies, PlayerEconomy,
};
use server::core::rsm::{DraftStarted, ResolutionComplete, RoundPhase, RoundState, RsmPlugin};
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
    app.add_plugins(EconomyPlugin);
    app.insert_resource(GameConfig(shared::config::GameConfig::default()));
    app.insert_resource(session_config(players));
    app
}

fn insert_economy(app: &mut App, player: PlayerId, economy: PlayerEconomy) {
    app.world_mut()
        .resource_mut::<PlayerEconomies>()
        .0
        .insert(player, economy);
}

fn write_resolution_complete(app: &mut App) {
    app.world_mut().write_message(ResolutionComplete);
    app.update();
}

fn write_draft_started(app: &mut App, round: u32, phase: DraftPhase) {
    app.world_mut().write_message(DraftStarted { round, phase });
    app.update();
}

#[test]
fn test_resolution_snapshot_captures_gold_at_resolution_end() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(8, 3, 0));

    write_resolution_complete(&mut app);

    let snapshots = app.world().resource::<InterestSnapshots>();
    assert_eq!(snapshots.0.get(&player(1)).copied(), Some(8));
}

#[test]
fn test_resolution_snapshot_gold_ten_yields_max_interest_next_draft() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(10, 0, 0));

    write_resolution_complete(&mut app);
    assert_eq!(
        app.world()
            .resource::<InterestSnapshots>()
            .0
            .get(&player(1))
            .copied(),
        Some(10)
    );

    write_draft_started(&mut app, 2, DraftPhase::Shop);

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.gold, 14);
    assert_eq!(economy.current_mana, 2);
}

#[test]
fn test_resolution_discards_current_mana() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(5, 4, 2));

    write_resolution_complete(&mut app);

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.current_mana, 0);
    assert_eq!(economy.reserve_mana, 2);
}

#[test]
fn test_resolution_snapshot_overwrites_stale_value() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(9, 0, 0));
    app.world_mut()
        .resource_mut::<InterestSnapshots>()
        .0
        .insert(player(1), 3);

    write_resolution_complete(&mut app);

    let snapshots = app.world().resource::<InterestSnapshots>();
    assert_eq!(snapshots.0.get(&player(1)).copied(), Some(9));
}

#[test]
fn test_zero_gold_snapshot_gives_baseline_only_next_draft() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(0, 0, 0));

    write_resolution_complete(&mut app);
    assert_eq!(
        app.world()
            .resource::<InterestSnapshots>()
            .0
            .get(&player(1))
            .copied(),
        Some(0)
    );

    write_draft_started(&mut app, 2, DraftPhase::Shop);

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.gold, 2);
    assert_eq!(economy.current_mana, 2);
}

#[test]
fn test_kill_reward_cross_threshold_uses_post_award_gold() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(9, 0, 0));

    {
        let mut economies = app.world_mut().resource_mut::<PlayerEconomies>();
        let economy = economies
            .0
            .get_mut(&player(1))
            .expect("player economy exists");
        apply_gold_award(economy, 1);
    }

    write_resolution_complete(&mut app);
    assert_eq!(
        app.world()
            .resource::<InterestSnapshots>()
            .0
            .get(&player(1))
            .copied(),
        Some(10)
    );

    write_draft_started(&mut app, 2, DraftPhase::Shop);

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.gold, 14);
}

#[test]
fn test_resolution_complete_snapshot_is_consumed_before_rsm_enters_next_draft() {
    let players = [player(1)];
    let mut app = app_with_economy(&players);
    insert_economy(&mut app, player(1), economy(10, 4, 0));

    {
        let mut rsm = app.world_mut().resource_mut::<RoundState>();
        rsm.phase = RoundPhase::Resolution;
        rsm.round_number = 1;
    }

    app.world_mut().write_message(ResolutionComplete);
    app.update();

    let rsm = app.world().resource::<RoundState>();
    assert_eq!(rsm.phase, RoundPhase::DraftShop);
    assert_eq!(rsm.round_number, 2);

    let economies = app.world().resource::<PlayerEconomies>();
    let economy = economies.0.get(&player(1)).expect("player economy exists");
    assert_eq!(economy.gold, 14);
    assert_eq!(economy.current_mana, 2);
    assert!(!app
        .world()
        .resource::<InterestSnapshots>()
        .0
        .contains_key(&player(1)));
}
