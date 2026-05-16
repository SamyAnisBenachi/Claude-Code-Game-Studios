//! Sprint 14 / Story 017 - S11-UX-HUD-OPP-FIGURINE integration tests.
//!
//! Covers the opponent figurine pre-pool, bottom-strip composition, snapshot
//! asset sync, and no-new-client-authority contract.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::GlobalZIndex;
use client::asset_wiring::hud_figurine_asset;
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::design_tokens::z_layers;
use client::ui::hud::{
    HudBottomStrip, HudEntities, HudEntity, HudFigurine, HudPlugin, OpponentFigurineMarker,
    HUD_ENTITY_COUNT,
};
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, ObjectiveSnapshot, OpponentObjectiveSnapshot, PlacementTimerMultiplier,
    PlayerSnapshot, RoundPhase, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_client_source(rel: &str) -> String {
    let path = client_src_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn app_with_hud_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HudPlugin);
    app.insert_resource(HudPlayerIdsFixture::resource());
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

struct HudPlayerIdsFixture;

impl HudPlayerIdsFixture {
    fn resource() -> client::ui::hud::HudPlayerIds {
        client::ui::hud::HudPlayerIds {
            local_id: player(1),
            opponent_id: player(2),
        }
    }
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
}

fn parent_of(app: &App, entity: Entity) -> Entity {
    app.world()
        .get::<ChildOf>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have a ChildOf parent"))
        .parent()
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn image_handle(app: &App, entity: Entity) -> Handle<Image> {
    app.world()
        .get::<ImageNode>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have ImageNode"))
        .image
        .clone()
}

fn expected_figurine_handle(app: &App, class_id: ClassId) -> Handle<Image> {
    app.world()
        .resource::<AssetServer>()
        .load(hud_figurine_asset(class_id))
}

#[test]
fn ac1_ac3_opponent_figurine_is_prepooled_and_exposed() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert_eq!(
        HUD_ENTITY_COUNT, 23,
        "opponent figurine story must bump the pre-pooled HUD entity count to 23"
    );
    assert_eq!(
        count_with::<HudFigurine>(&mut app),
        2,
        "own and opponent figurines should both carry HudFigurine"
    );
    assert_eq!(count_with::<OpponentFigurineMarker>(&mut app), 1);
    assert_ne!(
        entities.figurine, entities.opponent_figurine,
        "HudEntities must expose a distinct opponent figurine entity"
    );
    assert!(app.world().get::<HudFigurine>(entities.figurine).is_some());
    assert!(app
        .world()
        .get::<HudFigurine>(entities.opponent_figurine)
        .is_some());
    assert!(app
        .world()
        .get::<OpponentFigurineMarker>(entities.opponent_figurine)
        .is_some());
    assert!(app.world().get::<ImageNode>(entities.figurine).is_some());
    assert!(app
        .world()
        .get::<ImageNode>(entities.opponent_figurine)
        .is_some());
}

#[test]
fn ac2_ac12_ac15_opponent_figurine_composes_through_bottom_strip() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert!(app
        .world()
        .get::<HudBottomStrip>(entities.bottom_strip)
        .is_some());
    assert_eq!(
        parent_of(&app, entities.opponent_figurine),
        entities.bottom_strip
    );
    assert_eq!(
        app.world().get::<GlobalZIndex>(entities.bottom_strip),
        Some(&z_layers::UI_BASE),
        "opponent figurine should inherit the bottom strip's UI_BASE z-layer"
    );
    assert!(
        app.world()
            .get::<GlobalZIndex>(entities.opponent_figurine)
            .is_none(),
        "opponent figurine must not re-invent an inline GlobalZIndex"
    );

    let node = app
        .world()
        .get::<Node>(entities.opponent_figurine)
        .expect("opponent figurine should carry a Node");
    assert_ne!(
        node.position_type,
        PositionType::Absolute,
        "opponent figurine must compose as a flex child, not an absolute root anchor"
    );
    assert_eq!(node.width, Val::Px(64.0));
    assert_eq!(node.height, Val::Px(64.0));
    assert_eq!(node.min_width, Val::Px(64.0));
    assert_eq!(node.min_height, Val::Px(64.0));
}

#[test]
fn ac4_ac5_opponent_figurine_updates_from_snapshot_class_id() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::DraftShop,
            3,
            player_snapshot(player(1), ClassId::Iop),
            player_snapshot(player(2), ClassId::Cra),
        ),
    );
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(
        image_handle(&app, entities.figurine),
        expected_figurine_handle(&app, ClassId::Iop),
        "own figurine must resolve from local snapshot class"
    );
    assert_eq!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Cra),
        "opponent figurine must resolve from opponent snapshot class"
    );

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::DraftAuction,
            4,
            player_snapshot(player(1), ClassId::Iop),
            player_snapshot(player(2), ClassId::Ecaflip),
        ),
    );
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(
        image_handle(&app, entities.figurine),
        expected_figurine_handle(&app, ClassId::Iop),
        "unchanged own class should keep the own figurine on Iop"
    );
    assert_eq!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Ecaflip),
        "later authoritative snapshot should flip the opponent figurine class"
    );
}

#[test]
fn ac5_ac6_game_over_snapshot_updates_but_incremental_paths_do_not_exist() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.phase = RoundPhase::GameOver;
        current.round = 8;
    }
    app.update();

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::GameOver,
            8,
            player_snapshot(player(1), ClassId::Sacrier),
            player_snapshot(player(2), ClassId::Xelor),
        ),
    );
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(
        app.world().resource::<CurrentClientPhase>().phase,
        RoundPhase::GameOver
    );
    assert_eq!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Xelor),
        "snapshot rebuild must still update opponent figurine at GAME_OVER"
    );

    let source = read_client_source("ui/hud/mod.rs");
    assert!(
        !source.contains("MessageReceiver<S2CClassLocked"),
        "HUD must not add a parallel S2CClassLocked drain for incremental class updates"
    );
}

#[test]
fn ac7_ac8_opponent_figurine_path_has_no_objective_or_client_inference_input() {
    let source = read_client_source("ui/hud/mod.rs");
    let sync_block = source
        .split("pub fn sync_figurine_image_system")
        .nth(1)
        .and_then(|tail| tail.split("/// PAW-004: StateSync").next())
        .expect("sync_figurine_image_system block should be present");

    assert!(sync_block.contains("snapshot_hud_players"));
    assert!(sync_block.contains("opponent.class_id"));
    for forbidden in ["was_fake", "Objective", "Unit", "lane", "board"] {
        assert!(
            !sync_block.contains(forbidden),
            "opponent figurine sync must not infer class from {forbidden}"
        );
    }
    assert!(
        !source.contains("MessageReceiver<S2CClassLocked"),
        "opponent figurine must use the existing snapshot fanout, not a new Lightyear receiver"
    );
}

fn write_snapshot(app: &mut App, snapshot: S2CGameSnapshot) {
    app.world_mut()
        .resource_mut::<Messages<PresentationGameSnapshotMessage>>()
        .write(PresentationGameSnapshotMessage(snapshot));
}

fn snapshot(
    phase: RoundPhase,
    round_number: u32,
    own: PlayerSnapshot,
    opponent: PlayerSnapshot,
) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: player(1),
        round_number,
        phase,
        timer_remaining_ms: Some(12_000),
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

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}
