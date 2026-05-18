//! PROMPT 1081 — Lobby Confirm CTA stateful style regression.
//!
//! AUDIT-1076-07 reported that the lobby Confirm-class button rendered as
//! a dim text band rather than a real primary-action button: the click was
//! reachable in logs (`lobby_ui_confirm_button_state: dispatching
//! ConfirmClass`) but the only chrome was a dark `srgba(0.17, 0.18, 0.14,
//! 0.95)` background with no interaction-state feedback. The repair in
//! `client/src/ui/lobby.rs` promotes the CTA to a stateful primary
//! button with seven visual states keyed off `(Interaction,
//! LobbyViewState, LobbyInputState)`.
//!
//! This bin asserts the invariants that keep the CTA visible as a real
//! button:
//!
//! - **AC1** — Every variant of `LobbyConfirmButtonStyleState` maps to
//!   a distinct `(BackgroundColor, BorderColor, TextColor)` triple so
//!   the seven states are visually disambiguated (no two states paint
//!   the same chrome).
//! - **AC2** — `lobby_confirm_button_style_state` honors the precedence
//!   `Confirmed > Waiting > InFlight > Disabled > {Pressed | Hovered |
//!   Enabled}` so a server-acknowledged reveal cannot be visually
//!   masked by an in-flight latch or hover state.
//! - **AC3** — After plugin spawn the `LobbyConfirmClassButton` entity
//!   carries the Disabled-state colors verbatim (no session_id yet)
//!   which proves the spawn path resolved colors via
//!   `lobby_confirm_button_colors` instead of using the prior hard-coded
//!   `srgba(0.17, 0.18, 0.14, 0.95)` dim band literal.
//! - **AC4** — After the per-frame refresh runs with an active session,
//!   the confirm button repaints to the Enabled-state colors — the
//!   "ready to click" treatment — so the button reads as a real primary
//!   CTA the moment a room is joined.
//!
//! Friend-game scope only. This bin does NOT advance `QA-COND-0005`
//! Standard-tier accessibility, `QA-COND-0006` playtest validation, or
//! `PAW-TD-*-a` placeholder-art accept-risk.
//!
//! ## ADR alignment
//!
//! - **ADR-021 Presentation Layer Architecture**: visual-state mapping is
//!   pure; the refresh system reads resources and writes
//!   `BackgroundColor` / `BorderColor` / `TextColor` only — no protocol
//!   shape exercised, no game-state mutation.
//! - **ADR-002 Client-Server Authority**: visual states reflect lobby
//!   resources that themselves are driven by `S2C*` messages; no
//!   optimistic client-side authority is introduced.

use bevy::asset::AssetPlugin;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::ClientState;
use client::ui::lobby::{
    lobby_confirm_button_colors, lobby_confirm_button_style_state, LobbyConfirmButtonStyleState,
    LobbyConfirmClassButton, LobbyInputState, LobbyUiPlugin, LobbyViewState,
};
use shared::card::ClassId;
use shared::protocol::{GameMode, SessionSlot};

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn spawn_lobby_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);

    app.update();
    app.update();

    app
}

const ALL_STATES: [LobbyConfirmButtonStyleState; 7] = [
    LobbyConfirmButtonStyleState::Disabled,
    LobbyConfirmButtonStyleState::Enabled,
    LobbyConfirmButtonStyleState::Hovered,
    LobbyConfirmButtonStyleState::Pressed,
    LobbyConfirmButtonStyleState::InFlight,
    LobbyConfirmButtonStyleState::Waiting,
    LobbyConfirmButtonStyleState::Confirmed,
];

fn color_to_linear(c: &BackgroundColor) -> [f32; 4] {
    let s = c.0.to_srgba();
    [s.red, s.green, s.blue, s.alpha]
}

fn border_top_color_linear(border: &BorderColor) -> [f32; 4] {
    let s = border.top.to_srgba();
    [s.red, s.green, s.blue, s.alpha]
}

fn text_color_linear(c: &TextColor) -> [f32; 4] {
    let s = c.0.to_srgba();
    [s.red, s.green, s.blue, s.alpha]
}

fn rgba_approx_eq(lhs: [f32; 4], rhs: [f32; 4]) -> bool {
    lhs.iter()
        .zip(rhs.iter())
        .all(|(a, b)| (a - b).abs() < 1.0e-4)
}

#[test]
fn ac1_every_state_paints_a_distinct_triple() {
    test_helpers::init_test_tracing();

    let triples: Vec<(LobbyConfirmButtonStyleState, [f32; 4], [f32; 4], [f32; 4])> = ALL_STATES
        .iter()
        .map(|state| {
            let (bg, border, text) = lobby_confirm_button_colors(*state);
            (
                *state,
                color_to_linear(&bg),
                border_top_color_linear(&border),
                text_color_linear(&text),
            )
        })
        .collect();

    for i in 0..triples.len() {
        for j in (i + 1)..triples.len() {
            let (lhs_state, lhs_bg, lhs_border, lhs_text) = triples[i];
            let (rhs_state, rhs_bg, rhs_border, rhs_text) = triples[j];
            let identical = rgba_approx_eq(lhs_bg, rhs_bg)
                && rgba_approx_eq(lhs_border, rhs_border)
                && rgba_approx_eq(lhs_text, rhs_text);
            assert!(
                !identical,
                "AC1: confirm-button states {:?} and {:?} paint the same \
                 (BackgroundColor, BorderColor, TextColor) triple. Each \
                 state must be visually distinct so the player can read \
                 the CTA state at a glance.",
                lhs_state, rhs_state,
            );
        }
    }
}

#[test]
fn ac1_enabled_is_clearly_brighter_than_disabled() {
    test_helpers::init_test_tracing();

    // Disabled vs Enabled is the load-bearing distinction for
    // AUDIT-1076-07: when the user has just joined a room, the CTA must
    // visually transition from a dim "not yet" surface to a bright
    // "click me" surface. Sum-of-channels is a coarse but stable
    // brightness proxy that survives palette tweaks within the
    // primary gold treatment.
    let (enabled_bg, _, _) = lobby_confirm_button_colors(LobbyConfirmButtonStyleState::Enabled);
    let (disabled_bg, _, _) = lobby_confirm_button_colors(LobbyConfirmButtonStyleState::Disabled);

    let enabled_brightness = brightness(&enabled_bg);
    let disabled_brightness = brightness(&disabled_bg);

    assert!(
        enabled_brightness > disabled_brightness + 0.5,
        "AC1: Enabled background brightness ({enabled_brightness:.3}) must \
         be clearly greater than Disabled brightness ({disabled_brightness:.3}) \
         by at least 0.5 on the sum-of-RGB scale so the CTA reads as a \
         primary action when the session is active."
    );
}

fn brightness(bg: &BackgroundColor) -> f32 {
    let s = bg.0.to_srgba();
    (s.red + s.green + s.blue) * s.alpha
}

#[test]
fn ac2_state_precedence_confirmed_beats_everything_else() {
    test_helpers::init_test_tracing();

    let mut lobby = LobbyViewState {
        session_id: Some("session-1".to_string()),
        locked_class: Some(ClassId::Iop),
        revealed_classes: vec![(shared::session::PlayerId(1), ClassId::Iop)],
        ..LobbyViewState::default()
    };
    let mut input = LobbyInputState {
        class_confirm_in_flight: true,
        ..LobbyInputState::default()
    };

    // Hover + every other latch — Confirmed must still win.
    for interaction in [
        Interaction::None,
        Interaction::Hovered,
        Interaction::Pressed,
    ] {
        let state = lobby_confirm_button_style_state(&lobby, &input, interaction);
        assert_eq!(
            state,
            LobbyConfirmButtonStyleState::Confirmed,
            "AC2: with revealed_classes non-empty the state must be \
             Confirmed regardless of interaction (got {state:?} for \
             {interaction:?})"
        );
    }

    // Drop the reveal and Waiting becomes the floor.
    lobby.revealed_classes.clear();
    let state = lobby_confirm_button_style_state(&lobby, &input, Interaction::Pressed);
    assert_eq!(
        state,
        LobbyConfirmButtonStyleState::Waiting,
        "AC2: with locked_class Some and reveal empty, Waiting must win \
         over InFlight + Pressed (got {state:?})"
    );

    // Drop the lock and InFlight wins over Pressed.
    lobby.locked_class = None;
    let state = lobby_confirm_button_style_state(&lobby, &input, Interaction::Pressed);
    assert_eq!(
        state,
        LobbyConfirmButtonStyleState::InFlight,
        "AC2: with class_confirm_in_flight true and no locked_class, \
         InFlight must win over Pressed (got {state:?})"
    );

    // Drop the in-flight latch with a session and confirm Pressed surfaces.
    input.class_confirm_in_flight = false;
    let state = lobby_confirm_button_style_state(&lobby, &input, Interaction::Pressed);
    assert_eq!(
        state,
        LobbyConfirmButtonStyleState::Pressed,
        "AC2: with session_id Some and no latches, Pressed must surface \
         (got {state:?})"
    );

    // Drop the session and Disabled wins over every interaction.
    lobby.session_id = None;
    for interaction in [
        Interaction::None,
        Interaction::Hovered,
        Interaction::Pressed,
    ] {
        let state = lobby_confirm_button_style_state(&lobby, &input, interaction);
        assert_eq!(
            state,
            LobbyConfirmButtonStyleState::Disabled,
            "AC2: with session_id None the state must be Disabled \
             regardless of interaction (got {state:?} for {interaction:?})"
        );
    }
}

#[test]
fn ac3_spawn_paints_disabled_colors_when_no_session_id() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    // Default lobby has no session_id — spawn-time state is Disabled.
    let world = app.world_mut();
    let (bg, border, text) = {
        let mut confirms =
            world.query_filtered::<(&BackgroundColor, &BorderColor, &TextColor), With<LobbyConfirmClassButton>>();
        confirms
            .single(world)
            .map(|(b, bdr, t)| (*b, bdr.clone(), *t))
            .expect("AC3: single LobbyConfirmClassButton entity must exist")
    };

    let (expected_bg, expected_border, expected_text) =
        lobby_confirm_button_colors(LobbyConfirmButtonStyleState::Disabled);

    assert_eq!(
        color_to_linear(&bg),
        color_to_linear(&expected_bg),
        "AC3: spawned BackgroundColor must match Disabled-state literal \
         from `lobby_confirm_button_colors` — proves spawn path resolves \
         colors via the helper instead of the pre-PROMPT-1081 hard-coded \
         dim olive `srgba(0.17, 0.18, 0.14, 0.95)`"
    );
    assert_eq!(
        border_top_color_linear(&border),
        border_top_color_linear(&expected_border),
        "AC3: spawned BorderColor must match Disabled-state literal"
    );
    assert_eq!(
        text_color_linear(&text),
        text_color_linear(&expected_text),
        "AC3: spawned TextColor must match Disabled-state literal"
    );
}

#[test]
fn ac4_refresh_repaints_enabled_when_session_arrives() {
    test_helpers::init_test_tracing();
    let mut app = spawn_lobby_test_app();

    // Simulate `S2CRoomCreated` arriving: lobby acquires a session_id +
    // two slots. The refresh system on the next Update tick must repaint
    // the CTA from Disabled (dim) to Enabled (bright primary).
    {
        let mut lobby = app.world_mut().resource_mut::<LobbyViewState>();
        lobby.session_id = Some("session-1".to_string());
        lobby.room_code = Some("AAAA".to_string());
        lobby.mode = GameMode::OneVOne;
        lobby.slots = vec![
            SessionSlot {
                slot: 0,
                team: 0,
                player_id: Some(shared::session::PlayerId(1)),
                class_id: None,
                class_confirmed: false,
            },
            SessionSlot {
                slot: 1,
                team: 1,
                player_id: Some(shared::session::PlayerId(2)),
                class_id: None,
                class_confirmed: false,
            },
        ];
    }

    // Two ticks: first to propagate the resource change, second so the
    // refresh system's write is visible to the assertion query.
    app.update();
    app.update();

    let world = app.world_mut();
    let bg = {
        let mut confirms =
            world.query_filtered::<&BackgroundColor, With<LobbyConfirmClassButton>>();
        *confirms
            .single(world)
            .expect("AC4: single LobbyConfirmClassButton entity must exist")
    };

    let (expected_bg, _, _) = lobby_confirm_button_colors(LobbyConfirmButtonStyleState::Enabled);
    assert_eq!(
        color_to_linear(&bg),
        color_to_linear(&expected_bg),
        "AC4: after the session arrives the refresh system must repaint \
         the CTA with the Enabled-state BackgroundColor — the moment a \
         room is joined the button must read as the primary action. \
         Without the per-frame refresh the button would freeze on its \
         spawn-time Disabled chrome and reproduce AUDIT-1076-07."
    );
}

#[test]
fn ac5_friend_game_scope_no_claim_documented_inline() {
    // Source-embedded scope guard mirroring sibling tests.
    let source = include_str!("lobby_confirm_button_stateful_style_test.rs");
    assert!(
        source.contains("QA-COND-0005"),
        "AC5: friend-game-scope no-claim restatement must reference QA-COND-0005"
    );
    assert!(
        source.contains("QA-COND-0006"),
        "AC5: friend-game-scope no-claim restatement must reference QA-COND-0006"
    );
    assert!(
        source.contains("PAW-TD"),
        "AC5: friend-game-scope no-claim restatement must reference PAW-TD-*-a"
    );
}
