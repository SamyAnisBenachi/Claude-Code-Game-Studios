use std::collections::HashSet;
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::server::ServerPlugins;
use server::core::rsm::{DraftStarted, ResolutionComplete, ResolutionPhaseEntered};
use server::core::session::ReconnectTracker;
use server::feature::board::LaneId;
use server::feature::objective::{
    ObjectiveDestroyed, ObjectiveNetworkOutbox, ObjectivePlugin, ObjectiveResolutionState,
    PendingObjectiveEvents,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn destroyed(target_player_id: PlayerId, lane: LaneId, was_fake: bool) -> ObjectiveDestroyed {
    ObjectiveDestroyed {
        target_player_id,
        lane,
        was_fake,
    }
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / 60.0),
    })
    .add_message::<DraftStarted>()
    .add_message::<ResolutionPhaseEntered>()
    .add_message::<ResolutionComplete>()
    .add_plugins(ObjectivePlugin);
    app
}

fn queue_destroyed(app: &mut App, events: impl IntoIterator<Item = ObjectiveDestroyed>) {
    app.world_mut()
        .resource_mut::<PendingObjectiveEvents>()
        .queue
        .extend(events);
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

#[test]
fn test_os13a_objective_destroyed_broadcasts_only_at_resolution_end_sync() {
    test_helpers::init_test_tracing();
    let mut app = base_app();
    let event = destroyed(PLAYER_B, 3, false);
    queue_destroyed(&mut app, [event]);

    assert_eq!(
        app.world()
            .resource::<ObjectiveNetworkOutbox>()
            .destroyed_broadcasts(),
        &[]
    );

    app.world_mut().write_message(ResolutionComplete);
    app.update();

    assert!(app
        .world()
        .resource::<PendingObjectiveEvents>()
        .queue
        .is_empty());
    assert_eq!(read_messages::<ObjectiveDestroyed>(&app), vec![event]);

    let outbox = app.world().resource::<ObjectiveNetworkOutbox>();
    assert_eq!(outbox.destroyed_broadcasts(), &[event]);
}

#[test]
fn test_os13a_empty_resolution_end_sync_emits_zero_broadcasts() {
    test_helpers::init_test_tracing();
    let mut app = base_app();

    app.world_mut().write_message(ResolutionComplete);
    app.update();

    assert!(read_messages::<ObjectiveDestroyed>(&app).is_empty());
    let outbox = app.world().resource::<ObjectiveNetworkOutbox>();
    assert!(outbox.destroyed_broadcasts().is_empty());
}

#[test]
fn test_os18a_multiple_objective_destroyed_broadcasts_are_lane_ordered() {
    test_helpers::init_test_tracing();
    let mut app = base_app();
    queue_destroyed(
        &mut app,
        [
            destroyed(player(2), 3, false),
            destroyed(player(2), 1, true),
            destroyed(player(2), 5, false),
        ],
    );

    app.world_mut().write_message(ResolutionComplete);
    app.update();

    let outbox = app.world().resource::<ObjectiveNetworkOutbox>();
    let lanes = outbox
        .destroyed_broadcasts()
        .iter()
        .map(|event| event.lane)
        .collect::<Vec<_>>();
    assert_eq!(lanes, vec![1, 3, 5]);
}

#[test]
fn test_os24_sang_meprise_visibility_does_not_suppress_objective_destroyed() {
    test_helpers::init_test_tracing();
    let mut app = base_app();
    app.insert_resource(ReconnectTracker {
        sang_meprise_sent_to: HashSet::from([PLAYER_A]),
        ..Default::default()
    });
    let event = destroyed(PLAYER_B, 2, true);
    queue_destroyed(&mut app, [event]);

    app.world_mut().write_message(ResolutionComplete);
    app.update();

    let outbox = app.world().resource::<ObjectiveNetworkOutbox>();
    assert_eq!(outbox.destroyed_broadcasts(), &[event]);
    assert_eq!(read_messages::<ObjectiveDestroyed>(&app), vec![event]);
}

#[test]
fn test_resolution_phase_entered_marks_objective_resolution_ready() {
    test_helpers::init_test_tracing();
    let mut app = base_app();

    app.world_mut()
        .write_message(ResolutionPhaseEntered { round: 4 });
    app.update();

    let state = app.world().resource::<ObjectiveResolutionState>();
    assert_eq!(state.current_round(), Some(4));
    assert_eq!(state.entries_seen(), 1);
}
