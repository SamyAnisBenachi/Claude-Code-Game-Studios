use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::{
    apply_resolution_spawn_range_changes, forward_local_spawn_range_changes, BoardCellNode,
    BoardLocalPlayer, BoardRenderState, BoardRenderingPlugin, PlayerTeamMap, SpawnHighlightState,
    StandingObjective,
};
use client::presentation::LaneCell;
use client::state::{ClientGameSnapshotMessage, ClientState, CurrentClientPhase};
use client::ui::hand::LocalPlayerSpawnRangeChanged;
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, PlayerSnapshot, ResolutionEvent, RoundPhase, S2CGameSnapshot,
    S2CResolutionEvent, TaggedEvent,
};
use shared::session::PlayerId;

#[derive(Resource)]
struct TestResolutionScript(S2CResolutionEvent);

#[test]
fn test_snapshot_seed_sets_player_side_spawn_highlights() {
    let mut app = app_in_session();
    map_teams(&mut app);

    write_snapshot(
        &mut app,
        snapshot(player_snapshot(player(1), 2), player_snapshot(player(2), 1)),
    );
    app.update();

    for lane in 1..=5 {
        assert_cell(
            &mut app,
            lane,
            1,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            2,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            8,
            SpawnHighlightState::ValidSpawn {
                player_id: player(2),
            },
        );

        for cell in 3..=7 {
            assert_cell(&mut app, lane, cell, SpawnHighlightState::Inactive);
        }
    }
}

#[test]
fn test_live_spawn_range_changed_updates_only_that_player_side_and_persists() {
    let mut app = app_in_session();
    map_teams(&mut app);

    write_snapshot(
        &mut app,
        snapshot(player_snapshot(player(1), 1), player_snapshot(player(2), 2)),
    );
    app.update();

    apply_resolution_script(&mut app, script(vec![spawn_range_changed(player(1), 2)]));

    for lane in 1..=5 {
        assert_cell(
            &mut app,
            lane,
            1,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            2,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            7,
            SpawnHighlightState::ValidSpawn {
                player_id: player(2),
            },
        );
        assert_cell(
            &mut app,
            lane,
            8,
            SpawnHighlightState::ValidSpawn {
                player_id: player(2),
            },
        );

        for cell in 3..=6 {
            assert_cell(&mut app, lane, cell, SpawnHighlightState::Inactive);
        }
    }

    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::DraftShop;
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::DraftShop;
    app.update();
    assert_cell(
        &mut app,
        3,
        2,
        SpawnHighlightState::ValidSpawn {
            player_id: player(1),
        },
    );

    *app.world_mut().resource_mut::<BoardRenderState>() = BoardRenderState::Placement;
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = RoundPhase::Placement;
    app.update();
    assert_cell(
        &mut app,
        3,
        2,
        SpawnHighlightState::ValidSpawn {
            player_id: player(1),
        },
    );
}

#[test]
fn test_objective_destroyed_without_spawn_range_changed_does_not_change_highlights() {
    let mut app = app_in_session();
    map_teams(&mut app);

    write_snapshot(
        &mut app,
        snapshot(player_snapshot(player(1), 2), player_snapshot(player(2), 1)),
    );
    app.update();

    apply_resolution_script(
        &mut app,
        script(vec![ResolutionEvent::ObjectiveDestroyed {
            target_player_id: player(2),
            lane: 3,
            was_fake: true,
        }]),
    );

    for lane in 1..=5 {
        assert_cell(
            &mut app,
            lane,
            1,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            2,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            8,
            SpawnHighlightState::ValidSpawn {
                player_id: player(2),
            },
        );

        for cell in 3..=7 {
            assert_cell(&mut app, lane, cell, SpawnHighlightState::Inactive);
        }
    }
}

#[test]
fn test_objmiss_does_not_spawn_objective_and_separate_spawn_range_change_still_applies() {
    let mut app = app_in_session();
    map_teams(&mut app);

    assert_eq!(objective_count(&mut app), 0);

    apply_resolution_script(
        &mut app,
        script(vec![
            ResolutionEvent::ObjectiveDestroyed {
                target_player_id: player(2),
                lane: 4,
                was_fake: true,
            },
            spawn_range_changed(player(1), 2),
        ]),
    );

    assert_eq!(objective_count(&mut app), 0);
    for lane in 1..=5 {
        assert_cell(
            &mut app,
            lane,
            1,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        assert_cell(
            &mut app,
            lane,
            2,
            SpawnHighlightState::ValidSpawn {
                player_id: player(1),
            },
        );
        for cell in 3..=8 {
            assert_cell(&mut app, lane, cell, SpawnHighlightState::Inactive);
        }
    }
}

fn app_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(TransformPlugin);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(BoardRenderingPlugin);

    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn map_teams(app: &mut App) {
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(player(1), 0);
    app.world_mut()
        .resource_mut::<PlayerTeamMap>()
        .insert(player(2), 1);
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<ClientGameSnapshotMessage>>()
        .write(ClientGameSnapshotMessage(snapshot));
}

fn apply_resolution_script(app: &mut App, resolution_script: S2CResolutionEvent) {
    app.insert_resource(TestResolutionScript(resolution_script));
    app.add_systems(Update, apply_test_resolution_script);
    app.update();
}

fn apply_test_resolution_script(
    script: Res<TestResolutionScript>,
    local_player: Res<BoardLocalPlayer>,
    player_team_map: Res<PlayerTeamMap>,
    mut cells: Query<(&LaneCell, &mut SpawnHighlightState, &mut Sprite), With<BoardCellNode>>,
) {
    apply_resolution_spawn_range_changes(
        &script.0,
        local_player.player_id,
        &player_team_map,
        &mut cells,
    );
}

fn snapshot(player_a: PlayerSnapshot, player_b: PlayerSnapshot) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number: 6,
        phase: RoundPhase::Resolution,
        timer_remaining_ms: Some(10_000),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
        players: vec![player_a, player_b],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(player_id: PlayerId, spawn_range_cells: u8) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold: 0,
        reserved_gold: 0,
        current_mana: 0,
        reserve_mana: 0,
        spawn_range_cells,
        mana_cap: 1,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: objectives(),
        opponent_objectives: Vec::new(),
    }
}

fn objectives() -> Vec<ObjectiveSnapshot> {
    (1..=5)
        .map(|lane| ObjectiveSnapshot {
            lane,
            hp: 5,
            is_real: true,
            is_destroyed: false,
        })
        .collect()
}

fn script(events: Vec<ResolutionEvent>) -> S2CResolutionEvent {
    S2CResolutionEvent {
        round: 6,
        events: events
            .into_iter()
            .enumerate()
            .map(|(index, event)| TaggedEvent {
                sub_step: 6,
                trigger_index: index as u32,
                event,
            })
            .collect(),
    }
}

fn spawn_range_changed(player_id: PlayerId, new_spawn_range_cells: u8) -> ResolutionEvent {
    ResolutionEvent::SpawnRangeChanged {
        player_id,
        new_spawn_range_cells,
    }
}

fn assert_cell(app: &mut App, lane: u8, cell: u8, expected: SpawnHighlightState) {
    let (actual, tint) = cell_state(app, lane, cell);
    assert_eq!(actual, expected, "lane={lane} cell={cell}");
    assert_eq!(tint, expected.tint(), "lane={lane} cell={cell}");
}

fn cell_state(app: &mut App, lane: u8, cell: u8) -> (SpawnHighlightState, Color) {
    let mut query = app
        .world_mut()
        .query_filtered::<(&LaneCell, &SpawnHighlightState, &Sprite), With<BoardCellNode>>();
    query
        .iter(app.world())
        .find_map(|(lane_cell, state, sprite)| {
            (lane_cell.lane == lane && lane_cell.cell == cell).then_some((*state, sprite.color))
        })
        .expect("board cell should exist")
}

fn objective_count(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<StandingObjective>>();
    query.iter(app.world()).count()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

// ── PROMPT 1149 — `forward_local_spawn_range_changes` (producer side) ──────

#[derive(Resource)]
struct CapturedLocalSpawnRangeChanges(Vec<LocalPlayerSpawnRangeChanged>);

#[test]
fn forward_local_spawn_range_changes_emits_message_for_local_player_only() {
    let mut app = App::new();
    app.add_message::<LocalPlayerSpawnRangeChanged>();
    app.insert_resource(CapturedLocalSpawnRangeChanges(Vec::new()));
    app.add_systems(
        Update,
        (
            forward_for_player_one_system,
            capture_local_spawn_range_changes,
        )
            .chain(),
    );

    app.world_mut().spawn(ForwardingScript(script(vec![
        spawn_range_changed(player(1), 2),
        spawn_range_changed(player(2), 4),
    ])));
    app.update();

    let captured = &app.world().resource::<CapturedLocalSpawnRangeChanges>().0;
    assert_eq!(
        captured.len(),
        1,
        "exactly one local-player SpawnRangeChanged must be forwarded; opponent events are dropped"
    );
    assert_eq!(captured[0].new_spawn_range_cells, 2);
}

#[test]
fn forward_local_spawn_range_changes_noop_when_local_player_is_none() {
    let mut app = App::new();
    app.add_message::<LocalPlayerSpawnRangeChanged>();
    app.insert_resource(CapturedLocalSpawnRangeChanges(Vec::new()));
    app.add_systems(
        Update,
        (
            forward_with_no_local_player_system,
            capture_local_spawn_range_changes,
        )
            .chain(),
    );

    app.world_mut().spawn(ForwardingScript(script(vec![spawn_range_changed(
        player(1),
        2,
    )])));
    app.update();

    let captured = &app.world().resource::<CapturedLocalSpawnRangeChanges>().0;
    assert!(
        captured.is_empty(),
        "forward_local_spawn_range_changes must no-op when local_player_id is None"
    );
}

#[derive(Component)]
struct ForwardingScript(S2CResolutionEvent);

fn forward_for_player_one_system(
    scripts: Query<&ForwardingScript>,
    mut writer: MessageWriter<LocalPlayerSpawnRangeChanged>,
) {
    for script in &scripts {
        forward_local_spawn_range_changes(&script.0, Some(player(1)), &mut writer);
    }
}

fn forward_with_no_local_player_system(
    scripts: Query<&ForwardingScript>,
    mut writer: MessageWriter<LocalPlayerSpawnRangeChanged>,
) {
    for script in &scripts {
        forward_local_spawn_range_changes(&script.0, None, &mut writer);
    }
}

fn capture_local_spawn_range_changes(
    mut updates: MessageReader<LocalPlayerSpawnRangeChanged>,
    mut captured: ResMut<CapturedLocalSpawnRangeChanges>,
) {
    for update in updates.read() {
        captured.0.push(*update);
    }
}
