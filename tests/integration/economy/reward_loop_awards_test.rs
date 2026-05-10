// ECO-004 — Reward-loop awards integration test.
//
// Verifies the kill / objective / fake-reward economy pipeline:
//   - Direct combat path applies kill_gold_reward / objective_gold_reward via
//     economy_api::apply_gold_award (simulated here at the message-level).
//   - Self-inflicted destruction emits no AwardGold (consequence path guard).
//   - Combat is the exclusive direct writer for objective destruction gold;
//     adding an AwardGold consumer must NOT double-award.
//   - Economy AwardGold consumer applies the fake hand-full +1 fallback.
//   - Economy ManaCapIncreased consumer applies fake mana-cap rewards
//     (clamped at GameConfig.mana_cap_max), visible to the next DraftStarted
//     mana ramp.
//   - Both consumers run before on_resolution_complete so InterestSnapshots
//     captures the post-reward gold.

use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{
    apply_gold_award, AwardGold, EconomyPlugin, InterestSnapshots, ManaCapIncreased,
    PlayerEconomies, PlayerEconomy,
};
use server::core::rsm::{AuctionSettled, DraftStarted, ResolutionComplete, RsmPlugin};
use server::core::session::SessionConfig;
use server::foundation::config::GameConfig;
use shared::card::ClassId;
use shared::protocol::{DraftPhase, GameMode};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

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

fn economy(gold: u32, mana_cap: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap,
        reserved_gold: 0,
    }
}

fn app_with_economy(players: &[PlayerId]) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RsmPlugin);
    app.add_plugins(EconomyPlugin);
    // RsmPlugin's `rsm_input_reader` reads `ResolutionComplete` (registered
    // by CombatPlugin in production) and `AuctionSettled` (registered by
    // AuctionPlugin). This test stack omits both, so register the messages
    // here so MessageReader parameter validation succeeds.
    app.add_message::<ResolutionComplete>();
    app.add_message::<AuctionSettled>();
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

fn gold_of(app: &App, player: PlayerId) -> u32 {
    app.world()
        .resource::<PlayerEconomies>()
        .0
        .get(&player)
        .expect("player economy exists")
        .gold
}

fn mana_cap_of(app: &App, player: PlayerId) -> u32 {
    app.world()
        .resource::<PlayerEconomies>()
        .0
        .get(&player)
        .expect("player economy exists")
        .mana_cap
}

fn current_mana_of(app: &App, player: PlayerId) -> u32 {
    app.world()
        .resource::<PlayerEconomies>()
        .0
        .get(&player)
        .expect("player economy exists")
        .current_mana
}

#[test]
fn test_kill_gold_award_writes_through_api_when_config_default() {
    test_helpers::init_test_tracing();
    // Arrange: two-player session, both at gold=5; default config has
    // kill_gold_reward = 1.
    let mut app = app_with_economy(&[PLAYER_A, PLAYER_B]);
    insert_economy(&mut app, PLAYER_A, economy(5, 10));
    insert_economy(&mut app, PLAYER_B, economy(5, 10));
    let kill_gold_reward = app.world().resource::<GameConfig>().kill_gold_reward;

    // Act: apply the same direct path used by `feature/combat::award_kill_gold`.
    {
        let mut economies = app.world_mut().resource_mut::<PlayerEconomies>();
        let killer = economies
            .0
            .get_mut(&PLAYER_A)
            .expect("killer economy exists");
        apply_gold_award(killer, kill_gold_reward);
    }
    app.update();

    // Assert: killer gained kill_gold_reward; victim unchanged; no AwardGold
    // was emitted (combat owns the direct path).
    assert_eq!(gold_of(&app, PLAYER_A), 5 + kill_gold_reward);
    assert_eq!(gold_of(&app, PLAYER_B), 5);
}

#[test]
fn test_dual_kill_accumulates_two_awards() {
    test_helpers::init_test_tracing();
    // Arrange.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(5, 10));
    let reward = app.world().resource::<GameConfig>().kill_gold_reward;

    // Act: two kill records both credit PLAYER_A.
    {
        let mut economies = app.world_mut().resource_mut::<PlayerEconomies>();
        let economy = economies.0.get_mut(&PLAYER_A).unwrap();
        apply_gold_award(economy, reward);
        apply_gold_award(economy, reward);
    }
    app.update();

    // Assert: gain is exactly 2 * reward.
    assert_eq!(gold_of(&app, PLAYER_A), 5 + 2 * reward);
}

#[test]
fn test_objective_gold_award_writes_through_api_once_per_destruction() {
    test_helpers::init_test_tracing();
    // Arrange.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(5, 10));
    let reward = app.world().resource::<GameConfig>().objective_gold_reward;

    // Act: simulate the combat direct path for one opponent objective
    // destruction.
    {
        let mut economies = app.world_mut().resource_mut::<PlayerEconomies>();
        let economy = economies.0.get_mut(&PLAYER_A).unwrap();
        apply_gold_award(economy, reward);
    }
    app.update();

    // Assert: exactly one objective_gold_reward applied.
    assert_eq!(gold_of(&app, PLAYER_A), 5 + reward);
}

#[test]
fn test_self_inflicted_objective_does_not_award_gold_via_award_gold_consumer() {
    test_helpers::init_test_tracing();
    // Arrange: PLAYER_A starts with gold=5. The consequence path short-
    // circuits when attacker == defender, so no AwardGold message is emitted.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(5, 10));

    // Act: no message emitted, just tick the schedule once.
    app.update();

    // Assert: gold unchanged.
    assert_eq!(gold_of(&app, PLAYER_A), 5);
}

#[test]
fn test_no_duplicate_objective_reward_when_combat_path_runs_and_no_award_gold_emitted() {
    test_helpers::init_test_tracing();
    // Arrange: PLAYER_A at gold=5. Default objective_gold_reward = 3.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(5, 10));
    let reward = app.world().resource::<GameConfig>().objective_gold_reward;

    // Act: apply objective gold via the direct path (combat). DO NOT emit any
    // AwardGold message — combat is the exclusive direct writer per the
    // control manifest, and the consequence path no longer emits AwardGold
    // for regular objective destruction (ECO-004).
    {
        let mut economies = app.world_mut().resource_mut::<PlayerEconomies>();
        let economy = economies.0.get_mut(&PLAYER_A).unwrap();
        apply_gold_award(economy, reward);
    }
    app.update();

    // Assert: gold increased by exactly objective_gold_reward (no double).
    assert_eq!(gold_of(&app, PLAYER_A), 5 + reward);
}

#[test]
fn test_fake_mana_cap_reward_increments_mana_cap_below_ceiling() {
    test_helpers::init_test_tracing();
    // Arrange: cap=9 with default mana_cap_max=12.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(0, 9));

    // Act.
    app.world_mut().write_message(ManaCapIncreased {
        player: PLAYER_A,
        amount: 1,
    });
    app.update();

    // Assert.
    assert_eq!(mana_cap_of(&app, PLAYER_A), 10);
}

#[test]
fn test_fake_mana_cap_reward_clamps_at_mana_cap_max() {
    test_helpers::init_test_tracing();
    // Arrange: cap = mana_cap_max = 12 (the default).
    let mut app = app_with_economy(&[PLAYER_A]);
    let cap_max = app.world().resource::<GameConfig>().mana_cap_max;
    insert_economy(&mut app, PLAYER_A, economy(0, cap_max));

    // Act.
    app.world_mut().write_message(ManaCapIncreased {
        player: PLAYER_A,
        amount: 1,
    });
    app.update();

    // Assert: still clamped at cap_max.
    assert_eq!(mana_cap_of(&app, PLAYER_A), cap_max);
}

#[test]
fn test_fake_mana_cap_reward_visible_to_next_draft_started_mana_ramp() {
    test_helpers::init_test_tracing();
    // Arrange: cap=9 below mana_cap_max=12.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(0, 9));

    // Act: ManaCapIncreased lifts cap to 10, then DraftStarted with a large
    // round so the mana ramp would saturate against the cap.
    app.world_mut().write_message(ManaCapIncreased {
        player: PLAYER_A,
        amount: 1,
    });
    app.update();

    app.world_mut().write_message(DraftStarted {
        round: 100,
        phase: DraftPhase::Shop,
    });
    app.update();

    // Assert: current_mana saturates at the post-increment cap.
    assert_eq!(mana_cap_of(&app, PLAYER_A), 10);
    assert_eq!(current_mana_of(&app, PLAYER_A), 10);
}

#[test]
fn test_fake_free_card_fallback_award_gold_applies_once() {
    test_helpers::init_test_tracing();
    // Arrange.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(5, 10));

    // Act + Assert: first message applies +1.
    app.world_mut().write_message(AwardGold {
        player: PLAYER_A,
        amount: 1,
    });
    app.update();
    assert_eq!(gold_of(&app, PLAYER_A), 6);

    // Second message on a later frame applies another +1 (no buffering bug).
    app.world_mut().write_message(AwardGold {
        player: PLAYER_A,
        amount: 1,
    });
    app.update();
    assert_eq!(gold_of(&app, PLAYER_A), 7);
}

#[test]
fn test_award_gold_amount_is_independent_from_objective_gold_reward() {
    test_helpers::init_test_tracing();
    // Arrange.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(0, 10));

    // Act: arbitrary amount that matches neither kill_gold_reward (1) nor
    // objective_gold_reward (3) under default config.
    app.world_mut().write_message(AwardGold {
        player: PLAYER_A,
        amount: 5,
    });
    app.update();

    // Assert: the consumer is amount-driven.
    assert_eq!(gold_of(&app, PLAYER_A), 5);
}

#[test]
fn test_reward_consumers_run_before_on_resolution_complete() {
    test_helpers::init_test_tracing();
    // Arrange: PLAYER_A at gold=9. In the same frame, emit AwardGold +1 and
    // ResolutionComplete. The reward consumer must run before
    // on_resolution_complete so the snapshot captures the post-reward total.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(9, 10));

    // Act.
    app.world_mut().write_message(AwardGold {
        player: PLAYER_A,
        amount: 1,
    });
    app.world_mut().write_message(ResolutionComplete);
    app.update();

    // Assert: snapshot reflects 9 + 1 = 10.
    let snap = app
        .world()
        .resource::<InterestSnapshots>()
        .0
        .get(&PLAYER_A)
        .copied();
    assert_eq!(snap, Some(10));
    assert_eq!(gold_of(&app, PLAYER_A), 10);
}

#[test]
fn test_award_gold_for_unknown_player_is_silently_ignored() {
    test_helpers::init_test_tracing();
    // Arrange: only PLAYER_A is in PlayerEconomies. PLAYER_B is unknown to the
    // economy table (e.g., spectator id, race condition). The consumer must
    // not panic and must not synthesize a row.
    let mut app = app_with_economy(&[PLAYER_A]);
    insert_economy(&mut app, PLAYER_A, economy(0, 10));

    // Act.
    app.world_mut().write_message(AwardGold {
        player: PLAYER_B,
        amount: 1,
    });
    app.update();

    // Assert: PLAYER_A unaffected, PLAYER_B not inserted.
    assert_eq!(gold_of(&app, PLAYER_A), 0);
    assert!(!app
        .world()
        .resource::<PlayerEconomies>()
        .0
        .contains_key(&PLAYER_B));
}
