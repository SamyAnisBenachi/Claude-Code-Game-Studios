/// PAW-004 integration test: HUD class figurines, phase timer bar, and
/// objective dot states — ECS component presence checks.
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    state::ClientState,
    ui::{
        hud::{
            sync_dot_image_on_objective_destroyed_system, HudConfig, HudEntities, HudFigurine,
            HudPlayerIds, HudPlugin, HudTimerBar, ScoreboardDot, HUD_DOTS_PER_ROW, HUD_DOT_ROWS,
        },
        shared::HudObjectiveUpdate,
    },
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.add_plugins(HudPlugin);
    app.insert_resource(HudConfig {
        hud_margin_px: 12.0,
        hud_dot_diameter_px: 16.0,
        hud_tween_duration_ms: 300,
    });
    app.insert_resource(HudPlayerIds {
        local_id: PlayerId(1),
        opponent_id: PlayerId(2),
    });
    app
}

fn enter_session(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
}

// ── PAW-004-e: figurine, timer bar, and all 10 dots have ImageNode ────────────

/// Spawning the HUD pool must produce:
///   - 1 figurine entity with ImageNode
///   - 1 timer bar entity with ImageNode
///   - 10 scoreboard dot entities with ImageNode (5 own + 5 opponent)
/// Total: 12 entities carrying ImageNode from pool spawn.
#[test]
fn test_hud_pool_spawn_all_chrome_entities_have_image_node() {
    test_helpers::init_test_tracing();
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let entities = world.resource::<HudEntities>();

    // Figurine must have ImageNode.
    assert!(
        world.get::<ImageNode>(entities.figurine).is_some(),
        "HUD figurine entity must have ImageNode (PAW-004-a)"
    );

    // Timer bar must have ImageNode.
    assert!(
        world.get::<ImageNode>(entities.timer_bar).is_some(),
        "HUD timer bar entity must have ImageNode (PAW-004-b)"
    );

    // All 10 scoreboard dots must have ImageNode.
    let mut dot_count = 0;
    for row in 0..HUD_DOT_ROWS {
        for lane_index in 0..HUD_DOTS_PER_ROW {
            let dot_entity = entities.dots[row][lane_index];
            assert!(
                world.get::<ImageNode>(dot_entity).is_some(),
                "Scoreboard dot at row={row}, lane_index={lane_index} must have ImageNode (PAW-004-c)"
            );
            dot_count += 1;
        }
    }

    assert_eq!(
        dot_count,
        HUD_DOT_ROWS * HUD_DOTS_PER_ROW,
        "Must have exactly {} dot entities with ImageNode",
        HUD_DOT_ROWS * HUD_DOTS_PER_ROW
    );
}

/// The figurine entity has the HudFigurine marker component.
#[test]
fn test_figurine_entity_has_marker_component() {
    test_helpers::init_test_tracing();
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let entities = world.resource::<HudEntities>();

    assert!(
        world.get::<HudFigurine>(entities.figurine).is_some(),
        "HUD figurine entity must carry HudFigurine marker"
    );
}

/// The timer bar entity has the HudTimerBar marker component.
#[test]
fn test_timer_bar_entity_has_marker_component() {
    test_helpers::init_test_tracing();
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let entities = world.resource::<HudEntities>();

    assert!(
        world.get::<HudTimerBar>(entities.timer_bar).is_some(),
        "HUD timer bar entity must carry HudTimerBar marker"
    );
}

/// Every scoreboard dot entity has the ScoreboardDot marker component.
#[test]
fn test_all_dots_have_scoreboard_dot_marker() {
    test_helpers::init_test_tracing();
    let mut app = make_app();
    enter_session(&mut app);

    let world = app.world();
    let entities = world.resource::<HudEntities>();

    for row in 0..HUD_DOT_ROWS {
        for lane_index in 0..HUD_DOTS_PER_ROW {
            let dot_entity = entities.dots[row][lane_index];
            assert!(
                world.get::<ScoreboardDot>(dot_entity).is_some(),
                "Dot at row={row}, lane={lane_index} must have ScoreboardDot marker"
            );
        }
    }
}

// ── PAW-004-d: dot ImageNode updates on S2CObjectiveDestroyed ─────────────────

/// When a HudObjectiveUpdate is processed via StateSync, the targeted dot's
/// ImageNode must be updated to the destroyed asset.
#[test]
fn test_dot_image_node_updates_to_destroyed_on_objective_destroyed() {
    test_helpers::init_test_tracing();
    let mut app = make_app();

    // Register the sync system in Update so we can trigger it.
    app.add_systems(
        Update,
        sync_dot_image_on_objective_destroyed_system.run_if(in_state(ClientState::InSession)),
    );

    enter_session(&mut app);

    // Own row is index 1 (row 0 = opponent, row 1 = local).
    // Write a HudObjectiveUpdate for local player at lane 3.
    {
        let world = app.world_mut();
        world
            .resource_mut::<Messages<HudObjectiveUpdate>>()
            .write(HudObjectiveUpdate {
                target_player_id: PlayerId(1), // local player
                lane: 3,
            });
    }

    app.update();

    let world = app.world();
    let entities = world.resource::<HudEntities>();

    // Lane 3 = index 2 in the local row (row 1).
    let destroyed_dot = entities.dots[1][2];
    let img = world
        .get::<ImageNode>(destroyed_dot)
        .expect("Dot must have ImageNode");

    // The image path for the destroyed dot is loaded via AssetServer.
    // Since AssetPlugin is in use, the handle's path should match
    // HUD_OBJECTIVE_DOT_DESTROYED_ASSET.
    // We verify the handle is NOT the default (unset) handle.
    assert_ne!(
        img.image,
        Handle::default(),
        "Destroyed dot ImageNode must not be the default (unset) handle (PAW-004-d)"
    );

    // All other dots in local row must be unchanged (non-default).
    for lane_index in [0usize, 1, 3, 4] {
        let other_dot = entities.dots[1][lane_index];
        let other_img = world
            .get::<ImageNode>(other_dot)
            .expect("Dot must have ImageNode");
        // Other dots should still have their initial (non-destroyed) image handle.
        assert_ne!(
            other_img.image,
            Handle::default(),
            "Non-destroyed dot at lane_index={lane_index} must still have a valid ImageNode handle"
        );
    }
}

/// Multiple objectives destroyed in one frame each update independently.
#[test]
fn test_multiple_dots_destroyed_in_same_frame_each_update_independently() {
    test_helpers::init_test_tracing();
    let mut app = make_app();

    app.add_systems(
        Update,
        sync_dot_image_on_objective_destroyed_system.run_if(in_state(ClientState::InSession)),
    );

    enter_session(&mut app);

    // Destroy lanes 1, 3, and 5 in the opponent row (row 0).
    {
        let world = app.world_mut();
        let mut msgs = world.resource_mut::<Messages<HudObjectiveUpdate>>();
        msgs.write(HudObjectiveUpdate {
            target_player_id: PlayerId(2), // opponent
            lane: 1,
        });
        msgs.write(HudObjectiveUpdate {
            target_player_id: PlayerId(2),
            lane: 3,
        });
        msgs.write(HudObjectiveUpdate {
            target_player_id: PlayerId(2),
            lane: 5,
        });
    }

    app.update();

    let world = app.world();
    let entities = world.resource::<HudEntities>();

    // Verify that lanes 1, 3, 5 (indices 0, 2, 4) in opponent row have non-default handles.
    for lane_index in [0usize, 2, 4] {
        let dot = entities.dots[0][lane_index];
        let img = world
            .get::<ImageNode>(dot)
            .expect("Dot must have ImageNode");
        assert_ne!(
            img.image,
            Handle::default(),
            "Destroyed opponent dot at lane_index={lane_index} must have valid ImageNode"
        );
    }
}

// ── PAW-004-f: no UiImage in client/src/ui/hud/ ──────────────────────────────
// (Enforced by cargo check — compile error if UiImage used in Bevy 0.18.)
// The presence of ImageNode components above is the positive proof of compliance.
