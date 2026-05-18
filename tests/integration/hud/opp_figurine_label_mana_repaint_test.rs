//! Sprint 17 / Story 018 — S17-UI-HUD-OPP-MANA-CLEANUP-001 integration tests.
//!
//! Covers AUDIT-1076-10 + AUDIT-1076-16: the opponent figurine `ImageNode`
//! and the OPP value `Text` of the OPP pill both re-skin to the revealed
//! opponent class once `S2CClassesRevealed` lands (via the canonical
//! `LobbyViewState.revealed_classes` mirror) or when an `S2CGameSnapshot`
//! reconnect rebuild arrives. FROZEN-on-GAME_OVER + ADR-002 +
//! ADR-001 invariants are guarded so no client-side class inference is
//! introduced and no objective identity flows into the OPP carriers.
//!
//! AUDIT-1076-17 (the floating mana microbadge) is intentionally NOT
//! exercised here — its spawn site lives in `client/src/ui/hand/mod.rs`
//! (the per-card Reserve strip) which the PROMPT 1105 worker is forbidden
//! to touch. AC3 is escalated, see
//! `production/qa/evidence/sprint-17-hud-opp-mana-cleanup/evidence.md`.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::hud_figurine_asset;
use client::presentation::PresentationGameSnapshotMessage;
use client::state::{ClientSessionIdentity, ClientState, CurrentClientPhase};
use client::ui::hud::{
    format_opp_class_display, HudClassReveal, HudEntities, HudMode, HudPlugin, HudSystemSet,
    OpponentFigurineMarker,
};
use client::ui::lobby::LobbyViewState;
use shared::card::ClassId;
use shared::protocol::{
    BoardSnapshot, GameMode, ObjectiveSnapshot, OpponentObjectiveSnapshot,
    PlacementTimerMultiplier, PlayerSnapshot, RoundPhase, S2CGameSnapshot,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn client_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn read_client_source(rel: &str) -> String {
    let path = client_src_root().join(rel);
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    // Normalise CRLF → LF so the structural splits below are line-ending
    // agnostic across Windows / Unix checkouts.
    raw.replace("\r\n", "\n")
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
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
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

fn text_value(app: &App, entity: Entity) -> String {
    app.world()
        .get::<Text>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have Text"))
        .0
        .clone()
}

fn insert_lobby_reveal(app: &mut App, local: ClassId, opponent: ClassId) {
    app.world_mut().insert_resource(LobbyViewState {
        local_player_id: Some(player(1)),
        session_id: None,
        room_code: None,
        mode: GameMode::OneVOne,
        slots: Vec::new(),
        selected_class: local,
        locked_class: Some(local),
        revealed_classes: vec![(player(1), local), (player(2), opponent)],
        status: String::new(),
    });
    app.world_mut().insert_resource(ClientSessionIdentity {
        player_id: Some(player(1)),
        session_id: None,
        session_token: None,
    });
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

#[test]
fn ac1_opponent_figurine_reskins_on_classes_revealed() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    insert_lobby_reveal(&mut app, ClassId::Iop, ClassId::Ecaflip);

    app.update();
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Ecaflip),
        "opponent figurine must re-skin to the revealed opponent class via the lobby reveal path",
    );
    assert_eq!(
        app.world().resource::<HudClassReveal>().opponent,
        Some(ClassId::Ecaflip),
        "HudClassReveal.opponent must reflect the lobby reveal projection",
    );
}

#[test]
fn ac2_opp_text_label_reskins_on_classes_revealed() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    insert_lobby_reveal(&mut app, ClassId::Iop, ClassId::Ecaflip);

    app.update();
    app.update();

    let entities = hud_entities(&app);
    let prefix_text = text_value(&app, entities.opponent_gold_prefix);
    assert_eq!(
        prefix_text,
        format_opp_class_display(ClassId::Ecaflip),
        "OPP prefix label must re-skin to the revealed opponent class display string",
    );
    assert!(
        prefix_text.contains("Ecaflip"),
        "OPP prefix must carry the per-class display string after reveal (AUDIT-1076-16); got: {prefix_text:?}",
    );
    // The OPP value text remains the opponent-gold readout (still hidden /
    // placeholder until a gold broadcast arrives); the class identity rides
    // on the prefix label per the OPP-pill structural contract.
}

#[test]
fn ac4_reskin_runs_in_state_sync_set() {
    let source = read_client_source("ui/hud/mod.rs");
    // The class-reveal systems are wired in the `.add_systems(Update, (...))`
    // block. Doc comments earlier in the module also reference the system
    // names so we filter for the chunk that actually contains the canonical
    // tuple-call structure `Update,\n                (\n` and the
    // `.in_set(HudSystemSet::` chain.
    let schedule_block = source
        .split(".add_systems(")
        .filter(|chunk| chunk.contains("sync_class_reveal_hud_system"))
        .filter(|chunk| chunk.contains(".in_set(HudSystemSet::StateSync)"))
        .filter(|chunk| chunk.contains("Update,"))
        .last()
        .expect("HUD plugin add_systems(Update, ...) block carrying the class-reveal systems should be present");
    let hud_reskin_idx = schedule_block
        .find("sync_class_reveal_hud_system")
        .expect("sync_class_reveal_hud_system must appear in the Update schedule");
    let after_reskin = &schedule_block[hud_reskin_idx..];
    let chain_end = after_reskin.find(',').unwrap_or(after_reskin.len());
    let reskin_chain = &after_reskin[..chain_end];
    assert!(
        reskin_chain.contains("HudSystemSet::StateSync"),
        "sync_class_reveal_hud_system must be scheduled in HudSystemSet::StateSync per ADR-021 / TR-HUD-008; chain=\n{reskin_chain}",
    );

    let lobby_drain_idx = schedule_block
        .find("sync_class_reveal_from_lobby_view_system")
        .expect("sync_class_reveal_from_lobby_view_system must appear in the Update schedule");
    let after_lobby = &schedule_block[lobby_drain_idx..];
    let chain_end = after_lobby.find(',').unwrap_or(after_lobby.len());
    let lobby_chain = &after_lobby[..chain_end];
    assert!(
        lobby_chain.contains("HudSystemSet::MessageDrain"),
        "sync_class_reveal_from_lobby_view_system must drain in HudSystemSet::MessageDrain; chain=\n{lobby_chain}",
    );

    // Sanity: no tween / animation API leaks into the new reskin path's body.
    let body_with_tail = source
        .split("pub fn sync_class_reveal_hud_system")
        .nth(1)
        .expect("sync_class_reveal_hud_system body");
    let reskin_body = body_with_tail
        .split("\n/// ")
        .next()
        .unwrap_or(body_with_tail);
    for forbidden in ["TweenAnim", "Animator::new", "Tween::new"] {
        assert!(
            !reskin_body.contains(forbidden),
            "class reveal re-skin must be instantaneous (no `{forbidden}`)",
        );
    }
    // Reference the HudSystemSet variant so a future rename surfaces here.
    let _ = HudSystemSet::StateSync;
}

#[test]
fn ac5_reconnect_snapshot_rebuilds_opp_figurine_and_label() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::Placement,
            2,
            player_snapshot(player(1), ClassId::Iop),
            player_snapshot(player(2), ClassId::Sadida),
        ),
    );
    app.update();
    app.update();

    let entities = hud_entities(&app);
    assert_eq!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Sadida),
        "snapshot rebuild must repaint the opponent figurine",
    );
    let opp_prefix = text_value(&app, entities.opponent_gold_prefix);
    assert_eq!(
        opp_prefix,
        format_opp_class_display(ClassId::Sadida),
        "snapshot rebuild must repaint the OPP prefix label",
    );
    assert_eq!(
        app.world().resource::<HudClassReveal>().opponent,
        Some(ClassId::Sadida),
        "HudClassReveal.opponent must reflect snapshot rebuild",
    );
    assert_eq!(
        app.world().resource::<HudClassReveal>().local,
        Some(ClassId::Iop),
        "HudClassReveal.local must reflect snapshot rebuild",
    );
}

#[test]
fn ac6_frozen_blocks_lobby_reveal_but_snapshot_can_overwrite() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();

    // Force the HUD into Frozen (mirrors phase==GAME_OVER without exercising
    // the full phase reducer, since the reducer also paints labels we don't
    // care about here).
    *app.world_mut().resource_mut::<HudMode>() = HudMode::Frozen;
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.phase = RoundPhase::GameOver;
        current.round = 9;
    }

    // Frozen + incremental lobby reveal must NOT touch HudClassReveal.
    insert_lobby_reveal(&mut app, ClassId::Iop, ClassId::Ecaflip);
    app.update();
    app.update();

    let frozen_state = *app.world().resource::<HudClassReveal>();
    assert_eq!(
        frozen_state.opponent, None,
        "lobby reveal must be rejected while HUD is Frozen (AC6 / TR-HUD-009)",
    );
    let entities = hud_entities(&app);
    assert_ne!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Ecaflip),
        "frozen HUD must not adopt the lobby-revealed opponent class via the incremental path",
    );

    // Snapshot rebuild still overwrites.
    write_snapshot(
        &mut app,
        snapshot(
            RoundPhase::GameOver,
            9,
            player_snapshot(player(1), ClassId::Sacrier),
            player_snapshot(player(2), ClassId::Xelor),
        ),
    );
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<HudClassReveal>().opponent,
        Some(ClassId::Xelor),
        "snapshot rebuild must overwrite HudClassReveal even while Frozen",
    );
    let entities = hud_entities(&app);
    assert_eq!(
        image_handle(&app, entities.opponent_figurine),
        expected_figurine_handle(&app, ClassId::Xelor),
        "snapshot rebuild must repaint the opponent figurine even while Frozen",
    );
    let opp_prefix = text_value(&app, entities.opponent_gold_prefix);
    assert_eq!(
        opp_prefix,
        format_opp_class_display(ClassId::Xelor),
        "snapshot rebuild must repaint the OPP prefix label even while Frozen",
    );
}

#[test]
fn ac7_no_client_side_class_inference_introduced() {
    let source = read_client_source("ui/hud/mod.rs");
    let block = source
        .split("pub fn sync_class_reveal_from_lobby_view_system")
        .nth(1)
        .and_then(|tail| {
            tail.split("/// S17-UI-HUD-OPP-MANA-CLEANUP — MessageDrain: project the snapshot")
                .next()
        })
        .expect("sync_class_reveal_from_lobby_view_system body");
    for forbidden in [
        "Unit",
        "lane",
        "board",
        "Objective",
        "was_fake",
        "BoardSnapshot",
    ] {
        assert!(
            !block.contains(forbidden),
            "lobby-view class projection must not infer class from `{forbidden}`",
        );
    }
    assert!(
        !source.contains("MessageReceiver<S2CClassesRevealed>"),
        "HUD must not add a parallel Lightyear MessageReceiver — class identity flows through the lobby view + snapshot mirror",
    );
}

#[test]
fn ac8_adr_001_invariant_preserved_opp_carriers_carry_only_class() {
    let source = read_client_source("ui/hud/mod.rs");
    // Bound the sync_class_reveal_hud_system body by the next `pub fn`
    // declaration so doc comments / siblings further down the file do
    // not leak forbidden tokens into the grep scope.
    let body_with_tail = source
        .split("pub fn sync_class_reveal_hud_system")
        .nth(1)
        .expect("sync_class_reveal_hud_system body");
    let block = body_with_tail
        .split("\npub fn ")
        .next()
        .unwrap_or(body_with_tail);
    for forbidden in [
        "was_fake",
        "ObjectiveSnapshot",
        "OpponentObjectiveSnapshot",
        "Unit",
        "lane",
        "is_real",
        "ObjectiveDotState",
    ] {
        assert!(
            !block.contains(forbidden),
            "OPP figurine / label re-skin path must not surface `{forbidden}` (ADR-001 invariant)",
        );
    }
    // Defence in depth: format_opp_class_display has no leakage either.
    let display_tail = source
        .split("pub fn format_opp_class_display")
        .nth(1)
        .expect("format_opp_class_display body");
    // The function body ends at the first top-level `\n}\n` after the
    // function signature; everything past that closing brace is the
    // next sibling (its doc comment may legally mention `Objective`).
    let display_block = display_tail.split("\n}\n").next().unwrap_or(display_tail);
    for forbidden in ["was_fake", "Objective", "Unit", "lane", "is_real"] {
        assert!(
            !display_block.contains(forbidden),
            "format_opp_class_display must not embed objective identity (`{forbidden}`)",
        );
    }
}

#[test]
fn opponent_figurine_marker_still_unique() {
    // Sanity guard so the lobby projection does not accidentally bind to the
    // wrong figurine entity — the OpponentFigurineMarker remains the canonical
    // identity carrier introduced by Sprint 14 story 017.
    let mut app = app_with_hud_in_session();
    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<OpponentFigurineMarker>>();
    assert_eq!(
        q.iter(app.world()).count(),
        1,
        "opponent figurine marker should remain a singleton",
    );
}
