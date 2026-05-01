use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::TweenAnim;
use client::{
    state::{apply_phase_changed_message, ClientState, CurrentClientPhase},
    ui::hud::{HudEntities, HudEntity, HudPlugin},
};
use shared::protocol::{RoundPhase, S2CPhaseChanged};

#[test]
fn phase_label_strings_match_all_visible_round_phases() {
    let mut app = app_with_hud_in_session();
    let cases = [
        (RoundPhase::DraftInitial, "DRAFT INITIAL"),
        (RoundPhase::DraftShop, "DRAFT"),
        (RoundPhase::DraftAuction, "AUCTION"),
        (RoundPhase::Placement, "PLACEMENT"),
        (RoundPhase::Resolution, "RESOLUTION"),
        (RoundPhase::GameOver, "GAME OVER"),
    ];

    for (phase, expected_label) in cases {
        set_phase(&mut app, phase, 3, 60_000);
        app.update();

        let entities = hud_entities(&app);
        assert_eq!(text(&app, entities.phase_label), expected_label);
        assert_eq!(text(&app, entities.round_counter), "R3");
    }
}

#[test]
fn round_counter_uses_plain_r_prefix_without_padding() {
    let mut app = app_with_hud_in_session();
    let cases = [(9, "R9"), (1, "R1"), (20, "R20"), (0, "R0")];

    for (round, expected_round) in cases {
        set_phase(&mut app, RoundPhase::Placement, round, 12_000);
        app.update();

        let entities = hud_entities(&app);
        assert_eq!(text(&app, entities.round_counter), expected_round);
    }
}

#[test]
fn phase_and_round_replace_text_in_the_same_update_without_animators() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::DraftShop, 5, 30_000);
    app.update();
    set_phase(&mut app, RoundPhase::Placement, 6, 45_000);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.phase_label), "PLACEMENT");
    assert_eq!(text(&app, entities.round_counter), "R6");
    assert!(app.world().get::<TweenAnim>(entities.phase_label).is_none());
    assert!(app
        .world()
        .get::<TweenAnim>(entities.round_counter)
        .is_none());
}

#[test]
fn multiple_phase_messages_before_update_are_last_write_wins() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::DraftShop, 5, 30_000);
    set_phase(&mut app, RoundPhase::Resolution, 6, 45_000);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.phase_label), "RESOLUTION");
    assert_eq!(text(&app, entities.round_counter), "R6");
}

#[test]
fn lobby_phase_does_not_overwrite_last_visible_label() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::DraftAuction, 4, 60_000);
    app.update();
    set_phase(&mut app, RoundPhase::Lobby, 5, 60_000);
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.phase_label), "AUCTION");
    assert_eq!(text(&app, entities.round_counter), "R4");
}

#[test]
fn phase_change_timer_duration_is_discarded_from_hud_text() {
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::Placement, 3, 60_000);
    app.update();

    assert!(!hud_text_contains(&mut app, "60000"));
    assert!(!hud_text_contains(&mut app, "60s"));
    assert!(!hud_text_contains(&mut app, "60 sec"));
    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.phase_label), "PLACEMENT");
    assert_eq!(text(&app, entities.round_counter), "R3");
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HudPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase, round_number: u32, timer_duration_ms: u32) {
    let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
    apply_phase_changed_message(
        S2CPhaseChanged {
            phase,
            round_number,
            timer_duration_ms,
        },
        &mut current,
    );
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .expect("HUD text entity should have Text")
        .0
        .clone()
}

fn hud_text_contains(app: &mut App, needle: &str) -> bool {
    let mut text_query = app
        .world_mut()
        .query_filtered::<&Text, (With<HudEntity>, With<Text>)>();
    if text_query
        .iter(app.world())
        .any(|text| text.0.contains(needle))
    {
        return true;
    }

    let mut span_query = app
        .world_mut()
        .query_filtered::<&TextSpan, (With<HudEntity>, With<TextSpan>)>();
    span_query
        .iter(app.world())
        .any(|span| span.0.contains(needle))
}
