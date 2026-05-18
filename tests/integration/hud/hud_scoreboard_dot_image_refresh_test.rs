//! S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
//! UI-1129-11 / AUDIT-1131-03) — scoreboard dot image refresh
//! coverage.
//!
//! Pre-PROMPT-1139 the opponent row spawned with the `Unknown` /
//! fog-of-war asset and stayed painted that way for the full session
//! because `write_dot_destroyed` only mutated `BackgroundColor` and
//! `BorderColor` — the underlying `ImageNode` was never rewritten.
//! `sync_scoreboard_dot_image_for_state_system` closes that gap.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{
    HUD_OBJECTIVE_DOT_ALIVE_ASSET, HUD_OBJECTIVE_DOT_DESTROYED_ASSET,
    HUD_OBJECTIVE_DOT_UNKNOWN_ASSET,
};
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState};
use client::ui::hud::{HudEntities, HudPlugin};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlacementTimerMultiplier,
    PlayerSnapshot, RoundPhase, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.world_mut().insert_resource(ClientSessionIdentity {
        player_id: Some(PlayerId(1)),
        session_id: None,
        session_token: None,
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn image_handle(app: &App, entity: Entity) -> Handle<Image> {
    app.world()
        .get::<ImageNode>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have ImageNode"))
        .image
        .clone()
}

fn expected_handle(app: &App, asset: &'static str) -> Handle<Image> {
    app.world().resource::<AssetServer>().load(asset)
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<PresentationGameSnapshotMessage>>()
        .write(PresentationGameSnapshotMessage(snapshot));
}

fn snapshot_with_objectives(own_destroyed: [bool; 5], opp_destroyed: [bool; 5]) -> S2CGameSnapshot {
    let own = PlayerSnapshot {
        player_id: PlayerId(1),
        class_id: ClassId::Iop,
        gold: 10,
        reserved_gold: 0,
        current_mana: 5,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: own_destroyed
            .iter()
            .enumerate()
            .map(|(i, destroyed)| ObjectiveSnapshot {
                lane: (i as u8) + 1,
                hp: if *destroyed { 0 } else { 3 },
                is_real: false,
                is_destroyed: *destroyed,
            })
            .collect(),
        opponent_objectives: opp_destroyed
            .iter()
            .enumerate()
            .map(|(i, destroyed)| OpponentObjectiveSnapshot {
                lane: (i as u8) + 1,
                hp: if *destroyed { 0 } else { 3 },
                is_destroyed: *destroyed,
                was_fake: None,
            })
            .collect(),
    };
    let opponent = PlayerSnapshot {
        player_id: PlayerId(2),
        class_id: ClassId::Sacrier,
        gold: 10,
        reserved_gold: 0,
        current_mana: 5,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: opp_destroyed
            .iter()
            .enumerate()
            .map(|(i, destroyed)| ObjectiveSnapshot {
                lane: (i as u8) + 1,
                hp: if *destroyed { 0 } else { 3 },
                is_real: false,
                is_destroyed: *destroyed,
            })
            .collect(),
        opponent_objectives: Vec::new(),
    };
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: PlayerId(1),
        round_number: 1,
        phase: RoundPhase::Placement,
        timer_remaining_ms: Some(45_000),
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

#[test]
fn opponent_dots_spawn_with_unknown_image_before_first_snapshot() {
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);
    for lane in 0..5 {
        let dot = entities.dots[0][lane];
        let expected = expected_handle(&app, HUD_OBJECTIVE_DOT_UNKNOWN_ASSET);
        assert_eq!(
            image_handle(&app, dot),
            expected,
            "lane {lane}: opponent row must spawn with Unknown asset pre-snapshot",
        );
    }
}

#[test]
fn opponent_dots_repaint_to_alive_image_after_first_snapshot() {
    // Once a snapshot lands and the objective is alive (not destroyed)
    // the row must repaint to the alive asset so the user is not stuck
    // looking at the fog-of-war / skull placeholder.
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    write_snapshot(&mut app, snapshot_with_objectives([false; 5], [false; 5]));
    app.update();
    app.update();

    let entities = hud_entities(&app);
    let alive = expected_handle(&app, HUD_OBJECTIVE_DOT_ALIVE_ASSET);

    for lane in 0..5 {
        let dot = entities.dots[0][lane];
        assert_eq!(
            image_handle(&app, dot),
            alive,
            "opponent lane {lane} must repaint to Alive after snapshot (UI-1129-11)",
        );
    }
    for lane in 0..5 {
        let dot = entities.dots[1][lane];
        assert_eq!(
            image_handle(&app, dot),
            alive,
            "local lane {lane} must hold the Alive asset after snapshot",
        );
    }
}

#[test]
fn destroyed_dots_repaint_to_destroyed_image() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    let mut opp_destroyed = [false; 5];
    opp_destroyed[0] = true;
    opp_destroyed[3] = true;
    let mut own_destroyed = [false; 5];
    own_destroyed[2] = true;

    write_snapshot(
        &mut app,
        snapshot_with_objectives(own_destroyed, opp_destroyed),
    );
    app.update();
    app.update();

    let entities = hud_entities(&app);
    let alive = expected_handle(&app, HUD_OBJECTIVE_DOT_ALIVE_ASSET);
    let destroyed = expected_handle(&app, HUD_OBJECTIVE_DOT_DESTROYED_ASSET);

    assert_eq!(image_handle(&app, entities.dots[0][0]), destroyed);
    assert_eq!(image_handle(&app, entities.dots[0][1]), alive);
    assert_eq!(image_handle(&app, entities.dots[0][3]), destroyed);
    assert_eq!(image_handle(&app, entities.dots[1][2]), destroyed);
    assert_eq!(image_handle(&app, entities.dots[1][4]), alive);
}
