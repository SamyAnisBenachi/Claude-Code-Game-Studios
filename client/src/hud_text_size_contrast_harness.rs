use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use client::presentation::PlayerEconomyView;
use client::state::{apply_phase_changed_message, ClientState, CurrentClientPhase};
use client::ui::hud::{HudGoldBroadcastMessage, HudPlayerIds, HudPlugin, HudSystemSet};
use shared::protocol::{RoundPhase, S2CGoldBroadcast, S2CGoldUpdate, S2CPhaseChanged};
use shared::session::PlayerId;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "HUD-012 Text Size Contrast Harness".to_string(),
            resolution: (1366, 768).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(HudPlugin);
    app.init_resource::<HudTextSizeContrastHarnessState>()
        .add_systems(Startup, enter_hud_harness_session_system)
        .add_systems(
            Update,
            seed_hud_text_size_contrast_fixture_system
                .before(HudSystemSet::PhaseTransition)
                .run_if(in_state(ClientState::InSession)),
        );
    app.run();
}

#[derive(Resource, Default)]
struct HudTextSizeContrastHarnessState {
    seeded: bool,
    frames_after_seed: u8,
    published: bool,
}

fn enter_hud_harness_session_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    commands.spawn((Name::new("HUD-012 Harness Camera"), Camera2d));
    commands.insert_resource(HudPlayerIds {
        local_id: PlayerId(1),
        opponent_id: PlayerId(2),
    });
    next_state.set(ClientState::InSession);
}

fn seed_hud_text_size_contrast_fixture_system(
    mut state: ResMut<HudTextSizeContrastHarnessState>,
    mut current_phase: ResMut<CurrentClientPhase>,
    mut economy_view: ResMut<PlayerEconomyView>,
    mut gold_broadcasts: MessageWriter<HudGoldBroadcastMessage>,
) {
    if !state.seeded {
        apply_phase_changed_message(
            S2CPhaseChanged {
                phase: RoundPhase::DraftAuction,
                round_number: 9,
                timer_duration_ms: 60_000,
            },
            &mut current_phase,
        );
        economy_view.apply_gold_update(&S2CGoldUpdate {
            gold: 11,
            current_mana: 6,
            reserve_mana: 2,
            mana_cap: 10,
        });
        gold_broadcasts.write(HudGoldBroadcastMessage(S2CGoldBroadcast {
            player_id: PlayerId(1),
            gold: 11,
            reserved_gold: 4,
        }));
        gold_broadcasts.write(HudGoldBroadcastMessage(S2CGoldBroadcast {
            player_id: PlayerId(2),
            gold: 8,
            reserved_gold: 3,
        }));
        state.seeded = true;
        return;
    }

    if state.published {
        return;
    }

    state.frames_after_seed = state.frames_after_seed.saturating_add(1);
    if state.frames_after_seed >= 2 {
        state.published = true;
        publish_harness_ready();
    }
}

#[cfg(target_arch = "wasm32")]
fn publish_harness_ready() {
    use wasm_bindgen::{prelude::*, JsCast};

    let Some(window) = web_sys::window() else {
        return;
    };
    let payload = JsValue::from_str(
        r#"{"ready_for_capture":true,"fixture":"hud_012_text_size_contrast","phase":"DRAFT_AUCTION","round":9,"own_gold":"11g","own_reserved_gold":" (4r)","opponent_gold":"8g","opponent_reserved_gold":" (3r)","current_mana":"6 / 10","reserve_mana":"+2 reserve","text_size_px":{"gold_primary":40,"reserved_suffix":26,"current_mana":20,"reserve_mana":20,"phase_label":20,"round_counter":20},"contrast_ratio":{"primary_text_on_hud_background":17.87,"gold_on_hud_background":12.95,"reserved_suffix_on_hud_background":6.74,"primary_text_on_current_mana_bar":13.56,"primary_text_on_reserve_diamond":14.80}}"#,
    );
    if let Ok(callback) = js_sys::Reflect::get(
        window.as_ref(),
        &JsValue::from_str("hudTextSizeContrastReady"),
    ) {
        if let Some(function) = callback.dyn_ref::<js_sys::Function>() {
            let _ = function.call1(window.as_ref(), &payload);
        }
    }
    let _ = js_sys::Reflect::set(
        window.as_ref(),
        &JsValue::from_str("__hud012TextSizeContrastReady"),
        &JsValue::from_str("ready"),
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_harness_ready() {}
