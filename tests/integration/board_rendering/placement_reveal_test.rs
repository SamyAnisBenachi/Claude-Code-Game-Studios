use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::card_animations::{CardAnimationsPlugin, PlacementRevealAnimReady};
use client::presentation::board_rendering::{
    revealed_placement_unit_state, BoardRenderState, BoardRenderingPlugin, BoardRevealTimingConfig,
    BoardUnit, BoardUnitCard, BoardUnitOwner, BoardUnitRenderSource, PendingResolutionScript,
    PlacementRevealCollectState, ResolutionRevealWait, SnapshotRecoveryReason,
    SnapshotRecoveryRequested,
};
use client::presentation::LaneCell;
use client::state::ClientState;
use shared::card::CardId;
use shared::protocol::{PlacedCardReveal, PlayTarget, S2CPlacementReveal, S2CResolutionEvent};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn test_placement_reveal_collects_one_frame_and_emits_sorted_opponent_batch() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let opponent_late = spawn_board_unit(&mut app, 101, player(2), CardId(10), 3, 4);
    let opponent_early = spawn_board_unit(&mut app, 102, player(2), CardId(11), 1, 8);
    let _local_unit = spawn_board_unit(&mut app, 201, player(1), CardId(12), 2, 3);
    let mut cursor = drained_cursor::<PlacementRevealAnimReady>(&app);

    let reveal = S2CPlacementReveal {
        placements: vec![
            placed(player(1), CardId(12), 2, 3),
            placed(player(2), CardId(10), 3, 4),
            placed(player(2), CardId(11), 1, 8),
        ],
    };

    let collected_count = app
        .world_mut()
        .resource_mut::<PlacementRevealCollectState>()
        .start_from_reveal(&reveal, Some(player(1)));
    assert_eq!(collected_count, 2);

    app.update();
    assert!(
        messages_since(&app, &mut cursor).is_empty(),
        "first update is the collect window and should not emit tween requests"
    );

    app.update();
    let messages = messages_since(&app, &mut cursor);
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].entries.len(), 2);
    assert_eq!(messages[0].entries[0].unit, opponent_early);
    assert_eq!(messages[0].entries[0].lane, 1);
    assert_eq!(messages[0].entries[0].cell, 8);
    assert_eq!(messages[0].entries[1].unit, opponent_late);
    assert_eq!(messages[0].entries[1].lane, 3);
    assert_eq!(messages[0].entries[1].cell, 4);
}

#[test]
fn test_resolution_reveal_stuck_requests_one_snapshot() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let mut cursor = drained_cursor::<SnapshotRecoveryRequested>(&app);

    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::ResolutionReveal;
    app.world_mut()
        .resource_mut::<ResolutionRevealWait>()
        .start();
    let timeout_ms = app
        .world()
        .resource::<BoardRevealTimingConfig>()
        .resolution_reveal_timeout_ms;

    run_for(&mut app, Duration::from_millis(timeout_ms));
    let messages = messages_since(&app, &mut cursor);
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].reason,
        SnapshotRecoveryReason::ResolutionRevealStuck
    );

    run_for(&mut app, Duration::from_secs(3));
    assert!(
        messages_since(&app, &mut cursor).is_empty(),
        "stuck reveal recovery should request only once"
    );
}

#[test]
fn test_pending_resolution_script_stuck_requests_snapshot_and_keeps_script() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session();
    let script = S2CResolutionEvent {
        round: 7,
        events: Vec::new(),
    };
    let mut cursor = drained_cursor::<SnapshotRecoveryRequested>(&app);

    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::Placement;
    app.world_mut()
        .resource_mut::<PendingResolutionScript>()
        .set(script.clone());
    let timeout_ms = app
        .world()
        .resource::<BoardRevealTimingConfig>()
        .resolution_reveal_timeout_ms;

    run_for(&mut app, Duration::from_millis(timeout_ms));
    let messages = messages_since(&app, &mut cursor);
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0].reason,
        SnapshotRecoveryReason::PendingResolutionScriptStuck
    );
    assert_eq!(
        app.world().resource::<PendingResolutionScript>().script(),
        Some(&script)
    );

    run_for(&mut app, Duration::from_secs(3));
    assert!(
        messages_since(&app, &mut cursor).is_empty(),
        "pending script recovery should request only once"
    );
    assert_eq!(
        app.world().resource::<PendingResolutionScript>().script(),
        Some(&script)
    );
}

#[test]
fn test_placement_reveal_unit_state_is_visible_without_authoritative_snapshot() {
    let unit = revealed_placement_unit_state(player(2), CardId(101), 3, 7);

    assert_eq!(unit.owner_id, player(2));
    assert_eq!(unit.card_id, Some(CardId(101)));
    assert_eq!(
        unit.location,
        shared::protocol::UnitBoardLocation::BoardCell { lane: 3, cell: 7 }
    );
    assert_eq!(
        unit.unit_id & 0xF000_0000_0000_0000,
        0xF000_0000_0000_0000,
        "placement-reveal fallback units use a deterministic synthetic id namespace"
    );
    assert!(
        unit.stats.is_none(),
        "S2CPlacementReveal carries accepted placement records, not authoritative stats"
    );
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(CardAnimationsPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    app.update();
    app
}

fn run_for(app: &mut App, duration: Duration) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(duration);
    app.update();
}

fn spawn_board_unit(
    app: &mut App,
    unit_id: u64,
    owner_id: PlayerId,
    card_id: CardId,
    lane: u8,
    cell: u8,
) -> Entity {
    app.world_mut()
        .spawn((
            BoardUnit { unit_id },
            BoardUnitRenderSource::AuthoritativeSnapshot,
            BoardUnitOwner(owner_id),
            BoardUnitCard {
                card_id: Some(card_id),
                frame_index: 0,
                used_missing_art_fallback: false,
            },
            LaneCell { lane, cell },
        ))
        .id()
}

fn placed(owner_id: PlayerId, card_id: CardId, lane: u8, cell: u8) -> PlacedCardReveal {
    PlacedCardReveal {
        owner_id,
        card_id,
        target: PlayTarget::BoardCell { lane, cell },
    }
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn drained_cursor<M: Message + Clone>(app: &App) -> MessageCursor<M> {
    let messages = app.world().resource::<Messages<M>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn messages_since<M: Message + Clone>(app: &App, cursor: &mut MessageCursor<M>) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    cursor.read(messages).cloned().collect()
}
