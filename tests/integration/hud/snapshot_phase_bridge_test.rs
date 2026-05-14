//! HUD snapshot phase-bridge invariant (Sprint 12 Story 012 / Cluster B2).
//!
//! Path B (relocate) per
//! `production/epics/playable-client/story-012-fixture-hud-snapshot-phase-bridge.md`
//! "Design Decision". The bridge `S2CGameSnapshot.phase ->
//! Res<CurrentClientPhase>` is owned by `HudPlugin`'s
//! `handle_game_snapshot_system` (see `client/src/ui/hud/mod.rs:884-941`,
//! lines 940-941). The pre-existing
//! `reconnect_snapshot_rebuild_test::full_snapshot_rebuild_populates_all_hud_zones_without_respawning_entities`
//! exercises this bridge alongside nine other HUD-zone assertions; this file
//! adds a focused single-responsibility assertion so the invariant is legible
//! in trace.
//!
//! ADR-002 (no optimistic client-side authority), ADR-009 (server-authoritative
//! phase transitions), and ADR-021 (single shared phase sink) remain binding.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::{
    presentation::PresentationGameSnapshotMessage,
    state::{ClientState, CurrentClientPhase},
    ui::hud::{HudPlayerIds, HudPlugin},
};
use shared::{
    card::ClassId,
    protocol::{
        BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlayerSnapshot, RoundPhase,
        S2CGameSnapshot,
    },
    session::PlayerId,
};

#[path = "../../test_helpers.rs"]
mod test_helpers;

#[test]
fn test_hud_plugin_bridges_snapshot_phase_and_round_into_current_client_phase() {
    // Arrange
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let baseline = *app.world().resource::<CurrentClientPhase>();

    // Act -- write a snapshot whose phase + round differ from the baseline.
    let snapshot_phase = RoundPhase::Placement;
    let snapshot_round = 4;
    assert_ne!(
        baseline.phase, snapshot_phase,
        "test precondition: baseline phase must differ from snapshot phase so the bridge write is observable"
    );
    write_snapshot(&mut app, snapshot(snapshot_phase, snapshot_round));
    app.update();

    // Assert -- HudPlugin's handle_game_snapshot_system updated the resource.
    let current = app.world().resource::<CurrentClientPhase>();
    assert_eq!(
        current.phase, snapshot_phase,
        "HudPlugin must bridge S2CGameSnapshot.phase into Res<CurrentClientPhase>.phase"
    );
    assert_eq!(
        current.round, snapshot_round,
        "HudPlugin must bridge S2CGameSnapshot.round_number into Res<CurrentClientPhase>.round"
    );
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.insert_resource(HudPlayerIds {
        local_id: player(1),
        opponent_id: player(2),
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<PresentationGameSnapshotMessage>>()
        .write(PresentationGameSnapshotMessage(snapshot));
}

fn snapshot(phase: RoundPhase, round_number: u32) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number,
        phase,
        timer_remaining_ms: Some(20_000),
        placement_timer_multiplier_effective: shared::protocol::PlacementTimerMultiplier::X1,
        players: vec![
            player_snapshot(player(1), ClassId::Iop),
            player_snapshot(player(2), ClassId::Cra),
        ],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(player_id: PlayerId, class_id: ClassId) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id,
        gold: 0,
        reserved_gold: 0,
        current_mana: 0,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 1,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: empty_objective_snapshots(),
        opponent_objectives: empty_opponent_objective_snapshots(),
    }
}

fn empty_objective_snapshots() -> Vec<ObjectiveSnapshot> {
    (1..=5)
        .map(|lane| ObjectiveSnapshot {
            lane,
            hp: 3,
            is_real: false,
            is_destroyed: false,
        })
        .collect()
}

fn empty_opponent_objective_snapshots() -> Vec<OpponentObjectiveSnapshot> {
    (1..=5)
        .map(|lane| OpponentObjectiveSnapshot {
            lane,
            hp: 3,
            is_destroyed: false,
            was_fake: None,
        })
        .collect()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
