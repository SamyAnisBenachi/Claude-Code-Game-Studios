use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::presentation::PlayerEconomyView;
use client::state::{apply_phase_changed_message, ClientState, CurrentClientPhase};
use client::ui::hud::{HudPlayerIds, HudPlugin, HudSystemSet};
use shared::protocol::{RoundPhase, S2CGoldUpdate, S2CPhaseChanged};
use shared::session::PlayerId;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "HUD-011 Mana Shapes Harness".to_string(),
            resolution: (1366, 768).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(HudPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<HudManaShapesHarnessState>()
        .add_systems(Startup, enter_hud_harness_session_system)
        .add_systems(
            Update,
            seed_hud_mana_shapes_fixture_system
                .before(HudSystemSet::PhaseTransition)
                .run_if(in_state(ClientState::InSession)),
        );
    app.run();
}

#[derive(Resource, Default)]
struct HudManaShapesHarnessState {
    seeded: bool,
}

fn enter_hud_harness_session_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    commands.spawn((Name::new("HUD-011 Harness Camera"), Camera2d));
    commands.insert_resource(HudPlayerIds {
        local_id: PlayerId(1),
        opponent_id: PlayerId(2),
    });
    next_state.set(ClientState::InSession);
}

fn seed_hud_mana_shapes_fixture_system(
    mut state: ResMut<HudManaShapesHarnessState>,
    mut current_phase: ResMut<CurrentClientPhase>,
    mut economy_view: ResMut<PlayerEconomyView>,
) {
    if state.seeded {
        return;
    }

    apply_phase_changed_message(
        S2CPhaseChanged {
            phase: RoundPhase::DraftShop,
            round_number: 3,
            timer_duration_ms: 60_000,
        },
        &mut current_phase,
    );
    economy_view.apply_gold_update(&S2CGoldUpdate {
        gold: 8,
        current_mana: 6,
        reserve_mana: 2,
        mana_cap: 10,
    });
    state.seeded = true;
    publish_harness_ready();
}

#[cfg(target_arch = "wasm32")]
fn publish_harness_ready() {
    use wasm_bindgen::{prelude::*, JsCast};

    let Some(window) = web_sys::window() else {
        return;
    };
    let payload = JsValue::from_str(
        r#"{"ready_for_capture":true,"fixture":"hud_011_mana_shapes","current_mana":6,"mana_cap":10,"reserve_mana":2}"#,
    );
    if let Ok(callback) =
        js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("hudManaShapesReady"))
    {
        if let Some(function) = callback.dyn_ref::<js_sys::Function>() {
            let _ = function.call1(window.as_ref(), &payload);
        }
    }
    let _ = js_sys::Reflect::set(
        window.as_ref(),
        &JsValue::from_str("__hud011ManaShapesReady"),
        &JsValue::from_str("ready"),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_harness_ready() {}
