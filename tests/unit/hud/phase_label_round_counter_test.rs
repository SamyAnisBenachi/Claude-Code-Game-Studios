use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_tweening::TweenAnim;
use client::{
    state::{
        apply_phase_changed_message, apply_phase_view_message, ClientPhaseView, ClientState,
        CurrentClientPhase,
    },
    ui::hud::{HudEntities, HudEntity, HudPlugin},
};
use shared::protocol::{RoundPhase, S2CPhaseChanged};

#[test]
fn phase_label_strings_match_all_visible_round_phases() {
    let mut app = app_with_hud_in_session();
    // PROMPT 1250 (S18-HUD-PHASE-CHIP-DISAMBIGUATION-001 / B-1203-DRI-02):
    // every visible phase chip now carries a unique stem so the two
    // draft sub-phases stop collapsing to a shared `DRAFT ` prefix.
    let cases = [
        (RoundPhase::DraftInitial, "KEEP-9"),
        (RoundPhase::DraftShop, "SHOP"),
        (RoundPhase::DraftAuction, "AUCTION"),
        (RoundPhase::Placement, "PLACE"),
        (RoundPhase::Resolution, "RESOLVE"),
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
fn phase_labels_are_visually_distinct_at_a_glance() {
    // PROMPT 1250 (S18-HUD-PHASE-CHIP-DISAMBIGUATION-001 / B-1203-DRI-02):
    // regression guard for the original bug — two adjacent draft phases
    // sharing a `DRAFT ` prefix made the chip read identically at a
    // glance. Assert that no two visible phase chips share their first
    // token (the stem before the first space or hyphen) and that no
    // two labels are identical. If a future label change re-introduces
    // a shared stem the test fails before the player ever sees it.
    let visible_labels = [
        client::ui::hud::phase_label_text(RoundPhase::DraftInitial)
            .expect("DraftInitial has a visible chip"),
        client::ui::hud::phase_label_text(RoundPhase::DraftShop)
            .expect("DraftShop has a visible chip"),
        client::ui::hud::phase_label_text(RoundPhase::DraftAuction)
            .expect("DraftAuction has a visible chip"),
        client::ui::hud::phase_label_text(RoundPhase::Placement)
            .expect("Placement has a visible chip"),
        client::ui::hud::phase_label_text(RoundPhase::Resolution)
            .expect("Resolution has a visible chip"),
        client::ui::hud::phase_label_text(RoundPhase::GameOver)
            .expect("GameOver has a visible chip"),
    ];

    fn first_token(label: &str) -> &str {
        label
            .split(|c: char| c == ' ' || c == '-')
            .next()
            .unwrap_or(label)
    }

    for i in 0..visible_labels.len() {
        for j in (i + 1)..visible_labels.len() {
            let (a, b) = (visible_labels[i], visible_labels[j]);
            assert_ne!(
                a, b,
                "phase chips {a:?} and {b:?} are identical — chip cannot \
                 distinguish their round phases"
            );
            assert_ne!(
                first_token(a),
                first_token(b),
                "phase chips {a:?} and {b:?} share a first token \
                 {first:?}; at-a-glance scan cannot tell them apart",
                first = first_token(a),
            );
        }
    }
}

#[test]
fn lobby_and_handshaking_have_no_visible_chip() {
    // PROMPT 1250: the lobby and handshaking phases must continue to
    // suppress the chip so the disambiguation rewrite cannot leak a
    // pre-session label into the top strip.
    assert!(client::ui::hud::phase_label_text(RoundPhase::Lobby).is_none());
    assert!(client::ui::hud::phase_label_text(RoundPhase::Handshaking).is_none());
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
    assert_eq!(text(&app, entities.phase_label), "PLACE");
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
    assert_eq!(text(&app, entities.phase_label), "RESOLVE");
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
fn phase_change_timer_duration_appears_as_countdown_seconds() {
    // S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139,
    // UI-1129-06) — successor to the deleted
    // `phase_change_timer_duration_is_discarded_from_hud_text`. The
    // numeric remaining-seconds readout is now a first-class HUD
    // surface; raw millisecond literals must NOT leak, but the
    // seconds-rounded readout (`"60s"`) MUST be present on the
    // pre-pooled `HudTimerCountdown` entity once a phase change
    // publishes a non-zero `timer_duration_ms`.
    let mut app = app_with_hud_in_session();

    set_phase(&mut app, RoundPhase::Placement, 3, 60_000);
    app.update();

    assert!(
        !hud_text_contains(&mut app, "60000"),
        "raw millisecond literal must NOT leak into HUD text"
    );
    let entities = hud_entities(&app);
    assert_eq!(text(&app, entities.phase_label), "PLACE");
    assert_eq!(text(&app, entities.round_counter), "R3");
    assert_eq!(
        text(&app, entities.timer_countdown),
        "60s",
        "countdown text must reflect the rounded remaining seconds for the phase",
    );
    assert_eq!(
        app.world().get::<Visibility>(entities.timer_countdown),
        Some(&Visibility::Visible),
        "countdown must be visible once the phase publishes a non-zero duration",
    );
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase, round_number: u32, timer_duration_ms: u32) {
    let msg = S2CPhaseChanged {
        phase,
        round_number,
        timer_duration_ms,
    };
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        // `apply_phase_changed_message` takes the message by value; clone
        // here so the view-side reducer (which borrows) can still see
        // the same payload.
        apply_phase_changed_message(
            S2CPhaseChanged {
                phase: msg.phase,
                round_number: msg.round_number,
                timer_duration_ms: msg.timer_duration_ms,
            },
            &mut current,
        );
    }
    // PROMPT 1139 (UI-1129-06): the numeric countdown reads from
    // `ClientPhaseView.timer_duration_ms`, so the fixture must update
    // both phase resources to keep the countdown coverage realistic.
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        apply_phase_view_message(&msg, &mut phase_view);
    }
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
