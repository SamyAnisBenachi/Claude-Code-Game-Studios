use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PresentationPlugin;
use client::state::ClientState;
use client::ui::photosensitivity_warning::{
    PhotosensitivityWarningAcknowledge, PhotosensitivityWarningAcknowledged,
    PhotosensitivityWarningBody, PhotosensitivityWarningEntities, PhotosensitivityWarningRoot,
    PhotosensitivityWarningState, PHOTOSENSITIVITY_WARNING_COPY,
};

#[test]
fn test_warning_copy_source_is_single_observable_text() {
    assert!(PHOTOSENSITIVITY_WARNING_COPY.contains("impact flashes"));
    assert!(PHOTOSENSITIVITY_WARNING_COPY.contains("objective-destruction bursts"));
    assert!(PHOTOSENSITIVITY_WARNING_COPY.contains("phase transitions"));
}

#[test]
fn test_warning_appears_before_gameplay_exposure() {
    let mut app = warning_app();
    app.update();

    let state = app.world().resource::<PhotosensitivityWarningState>();
    assert!(!state.is_acknowledged());

    let root = app
        .world()
        .resource::<PhotosensitivityWarningEntities>()
        .root;
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Visible)
    );

    let body = warning_body_text(&mut app);
    assert_eq!(body, PHOTOSENSITIVITY_WARNING_COPY);
}

#[test]
fn test_warning_hides_when_client_enters_in_session_state() {
    // PROMPT 1026 — the photosensitivity warning must not paint over active
    // gameplay. Entering `ClientState::InSession` is the canonical signal that
    // session UI is now live (lobby ⇒ in-session handshake), so the warning's
    // visibility must flip to Hidden and the acknowledgement state must be
    // marked so it does not reappear if the player returns to Lobby.
    let mut app = warning_app();
    app.update();

    let root = app
        .world()
        .resource::<PhotosensitivityWarningEntities>()
        .root;
    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Visible),
        "warning must be visible before gameplay starts"
    );

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    assert_eq!(
        app.world().get::<Visibility>(root),
        Some(&Visibility::Hidden),
        "warning must be hidden once ClientState::InSession is entered"
    );
    let state = app.world().resource::<PhotosensitivityWarningState>();
    assert!(
        state.is_acknowledged(),
        "entering InSession must mark the warning acknowledged so it does not \
         reappear on subsequent Lobby returns this app run"
    );
}

#[test]
fn test_warning_acknowledgement_message_hides_warning() {
    let mut app = warning_app();
    app.update();

    app.world_mut()
        .write_message(PhotosensitivityWarningAcknowledged);
    app.update();

    let state = app.world().resource::<PhotosensitivityWarningState>();
    assert!(state.is_acknowledged());

    let entities = app.world().resource::<PhotosensitivityWarningEntities>();
    assert_eq!(
        app.world().get::<Visibility>(entities.root),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn test_warning_acknowledge_interaction_hides_warning() {
    let mut app = warning_app();
    app.update();

    let mut acknowledge_query = app
        .world_mut()
        .query_filtered::<Entity, With<PhotosensitivityWarningAcknowledge>>();
    let acknowledge = acknowledge_query
        .iter(app.world())
        .next()
        .expect("acknowledge entity should spawn");
    *app.world_mut()
        .get_mut::<Interaction>(acknowledge)
        .expect("acknowledge entity should have an Interaction component") = Interaction::Pressed;
    app.update();

    let state = app.world().resource::<PhotosensitivityWarningState>();
    assert!(state.is_acknowledged());

    let entities = app.world().resource::<PhotosensitivityWarningEntities>();
    assert_eq!(
        app.world().get::<Visibility>(entities.root),
        Some(&Visibility::Hidden)
    );
}

fn warning_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.add_plugins(PresentationPlugin);
    app.add_message::<client::ui::lobby::PlayerTeamMapUpdated>();
    app
}

fn warning_body_text(app: &mut App) -> String {
    app.world_mut()
        .query_filtered::<&Text, (
            With<PhotosensitivityWarningBody>,
            Without<PhotosensitivityWarningRoot>,
        )>()
        .iter(app.world())
        .next()
        .expect("warning body text should exist")
        .0
        .clone()
}
