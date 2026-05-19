//! PROMPT 1149 — Client placement critical repair.
//!
//! Pins the new `apply_placement_board_view_from_team_map_system` and
//! `apply_placement_board_view_spawn_range_system` against the latest
//! user-test pattern (2026-05-18 run-1):
//!
//! Without this fix, `PlacementBoardView` is only ever written by
//! `apply_placement_board_view_from_snapshot_system`, which reacts to
//! `PresentationGameSnapshotMessage` — that arrives only on reconnect or
//! explicit `C2SRequestSnapshot`. In a clean session start no snapshot is
//! ever sent, so the resource stays pinned to its `Default` value
//! `(PlayerId(1), spawn_edge=LowCells, range=1)`. Player A coincidentally
//! matches; Player B always defaults to `BoardCell { lane: 1, cell: 1 }`
//! and the server rejects every submission with `SpawnRangeRejected`
//! (see `reports/PROMPT-1130-multiplayer-game-state-consistency-audit.md`
//! NEW-1130-01 and `reports/PROMPT-1126-*` AUDIT-1126-01).
//!
//! The fix listens for `PlayerTeamMapUpdated` (broadcast by lobby drains on
//! `S2CRoomCreated` / `S2CJoinAck` / `S2CSlotUpdated` AND re-broadcast on
//! `OnEnter(ClientState::InSession)`) and reads the local player from
//! `ClientSessionIdentity`. A separate consumer pipes
//! `LocalPlayerSpawnRangeChanged` (written by
//! `consume_pending_resolution_script_system` after a fake objective is
//! destroyed) into `PlacementBoardView.spawn_range_cells` so the latent
//! NEW-1130-02 expansion path is covered.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::presentation::board_rendering::PlayerTeamMap;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use client::ui::hand::{
    BoardSpawnEdge, FanSlotIndex, HandCardCatalog, HandContents, HandFanCardClicked, HandUiPlugin,
    LocalPlayerSpawnRangeChanged, PendingPlacements, PlacementBoardView,
};
use client::ui::lobby::PlayerTeamMapUpdated;
use client::ui::shared::{BoardLayout, LaneCell, BOARD_CELL_COUNT, BOARD_LANE_COUNT};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase, SessionSlot};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const MINION_CARD_ID: CardId = CardId(103);

// ── apply_placement_board_view_from_team_map_system ─────────────────────────

#[test]
fn team_map_update_for_player_b_sets_high_cells_perspective() {
    test_helpers::init_test_tracing();
    let mut app = app_for_team_map_only(PlayerId(2));

    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view.local_player_id, PlayerId(2));
    assert_eq!(view.opponent_player_id, PlayerId(1));
    assert_eq!(view.spawn_edge, BoardSpawnEdge::HighCells);
    assert_eq!(
        view.spawn_range_cells, 1,
        "team-map bootstrap must not clobber spawn_range_cells",
    );
}

#[test]
fn team_map_update_for_player_a_sets_low_cells_perspective() {
    test_helpers::init_test_tracing();
    let mut app = app_for_team_map_only(PlayerId(1));

    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view.local_player_id, PlayerId(1));
    assert_eq!(view.opponent_player_id, PlayerId(2));
    assert_eq!(view.spawn_edge, BoardSpawnEdge::LowCells);
    assert_eq!(view.spawn_range_cells, 1);
}

#[test]
fn team_map_update_without_identity_leaves_placement_board_view_default() {
    // Mirrors the early-handshake window: PlayerTeamMapUpdated may arrive
    // before `apply_handshake_message` has populated `ClientSessionIdentity`
    // (e.g. test harnesses that seed the team map before the handshake reply).
    // The system must no-op rather than crash or guess the local player.
    test_helpers::init_test_tracing();
    let mut app = app_for_team_map_only_without_identity();

    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view, PlacementBoardView::default());
}

#[test]
fn snapshot_overrides_team_map_when_both_arrive_same_tick() {
    // A reconnect snapshot is authoritative; the team-map bootstrap must
    // not clobber the snapshot's spawn_range_cells when both fire in the
    // same tick. System ordering pins this: the snapshot system runs
    // before the team-map system in `MessageDrain`. The snapshot writes
    // spawn_range_cells=3; the team-map system preserves it.
    test_helpers::init_test_tracing();
    let mut app = app_for_team_map_only(PlayerId(2));

    write_snapshot_with_range(
        &mut app,
        PlayerId(2),
        &[(PlayerId(1), 0), (PlayerId(2), 1)],
        3,
    );
    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view.local_player_id, PlayerId(2));
    assert_eq!(view.spawn_edge, BoardSpawnEdge::HighCells);
    assert_eq!(
        view.spawn_range_cells, 3,
        "snapshot's spawn_range_cells must survive the team-map bootstrap",
    );
}

// ── apply_placement_board_view_spawn_range_system (latent NEW-1130-02) ──────

#[test]
fn local_player_spawn_range_changed_updates_placement_board_view() {
    test_helpers::init_test_tracing();
    let mut app = app_for_team_map_only(PlayerId(2));

    // First, bootstrap the perspective so we can confirm spawn_range_cells
    // updates independently of the other fields.
    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    app.update();
    assert_eq!(
        app.world()
            .resource::<PlacementBoardView>()
            .spawn_range_cells,
        1
    );

    // Now simulate the message published by
    // `consume_pending_resolution_script_system` when a fake objective is
    // destroyed and the local player's spawn range expands.
    app.world_mut().write_message(LocalPlayerSpawnRangeChanged {
        new_spawn_range_cells: 2,
    });
    app.update();

    let view = *app.world().resource::<PlacementBoardView>();
    assert_eq!(view.spawn_range_cells, 2);
    // Perspective fields must NOT regress.
    assert_eq!(view.local_player_id, PlayerId(2));
    assert_eq!(view.spawn_edge, BoardSpawnEdge::HighCells);
}

// ── End-to-end: Player B can stage a card on cell 8 without any snapshot ────

#[test]
fn player_b_click_stage_default_lands_on_cell_eight_after_team_map_only() {
    // This is the friend-game scenario: no S2CGameSnapshot is sent during a
    // clean session start. The only PlacementBoardView writer that fires is
    // the team-map bootstrap. The click-to-stage default must still target
    // the Player B spawn cell (8), not lane:1 cell:1 (Player A's default).
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_for_player(PlayerId(2));

    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    set_hand(&mut app, &[MINION_CARD_ID]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .write_message(HandFanCardClicked { card: slot });
    app.update();

    let placements = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(placements.len(), 1, "exactly one card should be staged");
    assert_eq!(placements[0].card_id, MINION_CARD_ID);
    assert_eq!(
        placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 8 },
        "Player B's default click-stage target must land on the high-cell spawn row (cell 8) \
         once `PlayerTeamMapUpdated` has fired — even when no `S2CGameSnapshot` is sent. \
         This is the friend-game scenario captured in run-1 client-b.log (2026-05-18) where \
         every Player B submission was rejected with `SpawnRangeRejected` because the \
         click-to-stage default targeted Player A's spawn cell."
    );
}

#[test]
fn player_a_click_stage_default_lands_on_cell_one_after_team_map_only() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_for_player(PlayerId(1));

    write_team_map_update(
        &mut app,
        &[
            slot_with_team(1, 0, PlayerId(1)),
            slot_with_team(2, 1, PlayerId(2)),
        ],
    );
    set_hand(&mut app, &[MINION_CARD_ID]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let slot = fan_slot(&mut app, 0);
    app.world_mut()
        .write_message(HandFanCardClicked { card: slot });
    app.update();

    let placements = &app.world().resource::<PendingPlacements>().placements;
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].card_id, MINION_CARD_ID);
    assert_eq!(
        placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 1 },
        "Player A's default click-stage target must land on the low-cell spawn row (cell 1).",
    );
}

// ── App builders ────────────────────────────────────────────────────────────

fn app_for_team_map_only(local_player_id: PlayerId) -> App {
    let mut app = base_app();
    seed_identity(&mut app, Some(local_player_id));
    enter_session(&mut app);
    app
}

fn app_for_team_map_only_without_identity() -> App {
    let mut app = base_app();
    seed_identity(&mut app, None);
    enter_session(&mut app);
    app
}

fn app_with_hand_ui_for_player(local_player_id: PlayerId) -> App {
    let mut app = base_app();
    app.insert_resource(BoardLayout {
        board_origin: Vec2::ZERO,
        cell_width: 64.0,
        lane_height: 80.0,
    });
    app.insert_resource(HandCardCatalog {
        cards: HashMap::from([(MINION_CARD_ID, minion_card())]),
    });
    seed_identity(&mut app, Some(local_player_id));
    enter_session(&mut app);
    spawn_board_cells(&mut app);
    app
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app
}

fn seed_identity(app: &mut App, player_id: Option<PlayerId>) {
    let mut identity = ClientSessionIdentity::default();
    identity.player_id = player_id;
    app.insert_resource(identity);
    // HandUiPlugin also mirrors `init_resource::<PlayerTeamMap>()`, so the
    // resource exists. We do not pre-populate it: the test exercises the
    // message-driven path (PlayerTeamMapUpdated) and the bootstrap reads the
    // slots out of the message rather than the resource. The
    // `apply_slots`-driven `PlayerTeamMap` resource is owned by
    // BoardRenderingPlugin and is not loaded in these tests.
    let _team_map = app.world().resource::<PlayerTeamMap>();
}

fn enter_session(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
}

fn slot_with_team(slot: u8, team: u8, player_id: PlayerId) -> SessionSlot {
    SessionSlot {
        slot,
        team,
        player_id: Some(player_id),
        class_id: None,
        class_confirmed: false,
        is_bot: false,
    }
}

fn write_team_map_update(app: &mut App, slots: &[SessionSlot]) {
    app.world_mut().write_message(PlayerTeamMapUpdated {
        slots: slots.to_vec(),
    });
}

fn write_snapshot_with_range(
    app: &mut App,
    local_player_id: PlayerId,
    teams: &[(PlayerId, u8)],
    spawn_range_cells: u8,
) {
    use client::presentation::PresentationGameSnapshotMessage;
    use shared::protocol::{
        BoardSnapshot, PlacementTimerMultiplier, PlayerSnapshot, S2CGameSnapshot,
    };

    let players: Vec<PlayerSnapshot> = teams
        .iter()
        .map(|(player_id, _team)| PlayerSnapshot {
            player_id: *player_id,
            class_id: ClassId::Iop,
            gold: 0,
            reserved_gold: 0,
            current_mana: 10,
            reserve_mana: 0,
            spawn_range_cells,
            mana_cap: 10,
            submitted: false,
            hand: if *player_id == local_player_id {
                vec![MINION_CARD_ID]
            } else {
                Vec::new()
            },
            shop_slots: Vec::new(),
            pool_snapshot: Vec::new(),
            objectives: Vec::new(),
            opponent_objectives: Vec::new(),
        })
        .collect();

    let snapshot = S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: local_player_id,
        round_number: 1,
        phase: RoundPhase::Placement,
        timer_remaining_ms: Some(10_000),
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players,
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    };

    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(snapshot));
}

fn minion_card() -> CardData {
    CardData {
        id: MINION_CARD_ID,
        name_fr: "Coureur du Marché".to_string(),
        name_en: "Market Runner".to_string(),
        class: ClassId::Iop,
        family: Some("PROMPT-1149".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 2,
        atk: 2,
        hp: 3,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: "market_runner".to_string(),
        pool_copies_override: None,
    }
}

fn spawn_board_cells(app: &mut App) {
    for lane in 1..=BOARD_LANE_COUNT {
        for cell in 1..=BOARD_CELL_COUNT {
            app.world_mut().spawn(LaneCell { lane, cell });
        }
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand(app: &mut App, cards: &[CardId]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}
