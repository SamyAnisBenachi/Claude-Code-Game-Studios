//! S18-UI-HUD-OPP-CLASS-TIMER-SCOREBOARD-REPAIR (PROMPT 1139, UI-1129-03)
//! — defensive class-projection coverage.
//!
//! `AUDIT-1129` reported the OPP pill rendering the *local* player's class
//! on both clients. The audit identified `sync_class_reveal_from_*`
//! systems as the suspected mirror site. PROMPT 1139 hardens both paths
//! to prefer the canonical handshake-assigned local id
//! (`ClientSessionIdentity.player_id`, falling back to
//! `LobbyViewState.local_player_id`) over `S2CGameSnapshot.recipient_player_id`
//! so that any routing/recipient inversion cannot flip the
//! local↔opponent mapping client-side.
//!
//! This test simulates two clients sharing the same snapshot payload but
//! with opposing identities and asserts each client resolves the OPP
//! pill to the *other* player's class.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState};
use client::ui::hud::{format_opp_class_display, HudClassReveal, HudEntities, HudPlugin};
use client::ui::lobby::LobbyViewState;
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameMode, ObjectiveSnapshot, OpponentObjectiveSnapshot,
    PlacementTimerMultiplier, PlayerSnapshot, RoundPhase, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<PresentationGameSnapshotMessage>>()
        .write(PresentationGameSnapshotMessage(snapshot));
}

fn snapshot_with_recipient(
    recipient_player_id: PlayerId,
    own: PlayerSnapshot,
    opponent: PlayerSnapshot,
) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id,
        round_number: 1,
        phase: RoundPhase::Placement,
        timer_remaining_ms: Some(45_000),
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![own, opponent],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(player_id: PlayerId, class_id: ClassId) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id,
        gold: 10,
        reserved_gold: 0,
        current_mana: 5,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand: Vec::new(),
        shop_slots: Vec::new(),
        pool_snapshot: Vec::new(),
        objectives: objective_snapshots(),
        opponent_objectives: opponent_objective_snapshots(),
    }
}

fn objective_snapshots() -> Vec<ObjectiveSnapshot> {
    (1..=5)
        .map(|lane| ObjectiveSnapshot {
            lane,
            hp: 3,
            is_real: false,
            is_destroyed: false,
        })
        .collect()
}

fn opponent_objective_snapshots() -> Vec<OpponentObjectiveSnapshot> {
    (1..=5)
        .map(|lane| OpponentObjectiveSnapshot {
            lane,
            hp: 3,
            is_destroyed: false,
            was_fake: None,
        })
        .collect()
}

fn text(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have Text"))
        .0
        .clone()
}

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

#[test]
fn client_one_sees_opponent_class_on_correct_recipient_snapshot() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    app.world_mut().insert_resource(ClientSessionIdentity {
        player_id: Some(player(1)),
        session_id: None,
        session_token: None,
    });

    // Snapshot carries the correct recipient_player_id for Client 1.
    write_snapshot(
        &mut app,
        snapshot_with_recipient(
            player(1),
            player_snapshot(player(1), ClassId::Sacrier),
            player_snapshot(player(2), ClassId::Iop),
        ),
    );
    app.update();
    app.update();

    let reveal = *app.world().resource::<HudClassReveal>();
    assert_eq!(reveal.local, Some(ClassId::Sacrier));
    assert_eq!(reveal.opponent, Some(ClassId::Iop));

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.opponent_gold_prefix),
        format_opp_class_display(ClassId::Iop),
        "OPP prefix on Client 1 must render the opponent (Iop) class",
    );
}

#[test]
fn client_two_sees_opponent_class_on_correct_recipient_snapshot() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    app.world_mut().insert_resource(ClientSessionIdentity {
        player_id: Some(player(2)),
        session_id: None,
        session_token: None,
    });

    write_snapshot(
        &mut app,
        snapshot_with_recipient(
            player(2),
            player_snapshot(player(2), ClassId::Iop),
            player_snapshot(player(1), ClassId::Sacrier),
        ),
    );
    app.update();
    app.update();

    let reveal = *app.world().resource::<HudClassReveal>();
    assert_eq!(reveal.local, Some(ClassId::Iop));
    assert_eq!(reveal.opponent, Some(ClassId::Sacrier));

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.opponent_gold_prefix),
        format_opp_class_display(ClassId::Sacrier),
        "OPP prefix on Client 2 must render the opponent (Sacrier) class",
    );
}

#[test]
fn snapshot_recipient_mismatch_falls_back_to_session_identity() {
    // PROMPT 1139 defence: even if the snapshot's recipient_player_id
    // is inverted (server routing bug, mis-personalized snapshot, etc.)
    // the HUD class projection must keep working off the
    // handshake-assigned local id and resolve the OPP pill to the
    // *other* player's class.
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    // Client 1 — handshake says player_id == P1.
    app.world_mut().insert_resource(ClientSessionIdentity {
        player_id: Some(player(1)),
        session_id: None,
        session_token: None,
    });

    // Snapshot is mis-routed: recipient_player_id is P2 (the opponent),
    // not P1. Without the defence the OPP pill would mirror the local
    // class. With the defence it must still render the opponent class.
    write_snapshot(
        &mut app,
        snapshot_with_recipient(
            player(2),
            player_snapshot(player(1), ClassId::Sacrier),
            player_snapshot(player(2), ClassId::Iop),
        ),
    );
    app.update();
    app.update();

    let reveal = *app.world().resource::<HudClassReveal>();
    assert_eq!(
        reveal.local,
        Some(ClassId::Sacrier),
        "local class must resolve from ClientSessionIdentity (P1 → Sacrier), not from snapshot.recipient_player_id",
    );
    assert_eq!(
        reveal.opponent,
        Some(ClassId::Iop),
        "opponent class must resolve to the other player (Iop), not mirror local",
    );

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.opponent_gold_prefix),
        format_opp_class_display(ClassId::Iop),
        "OPP prefix must show opponent class even if snapshot recipient is inverted",
    );
}

#[test]
fn lobby_view_local_id_drives_projection_when_identity_unset() {
    // If ClientSessionIdentity hasn't been written yet (or has been
    // cleared), the lobby projection should still resolve the OPP pill
    // correctly from `LobbyViewState.local_player_id`.
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    app.world_mut().insert_resource(LobbyViewState {
        local_player_id: Some(player(1)),
        session_id: None,
        room_code: None,
        mode: GameMode::OneVOne,
        slots: Vec::new(),
        selected_class: ClassId::Sacrier,
        locked_class: Some(ClassId::Sacrier),
        revealed_classes: vec![(player(1), ClassId::Sacrier), (player(2), ClassId::Iop)],
        status: String::new(),
    });

    app.update();
    app.update();

    let reveal = *app.world().resource::<HudClassReveal>();
    assert_eq!(reveal.local, Some(ClassId::Sacrier));
    assert_eq!(reveal.opponent, Some(ClassId::Iop));

    let entities = hud_entities(&app);
    assert_eq!(
        text(&app, entities.opponent_gold_prefix),
        format_opp_class_display(ClassId::Iop),
    );
}
