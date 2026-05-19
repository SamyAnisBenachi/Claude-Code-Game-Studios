//! PROMPT 1400 — S18-HUD-OPP-CLASS-MANA-MICROBADGE-POLISH-001
//! AUDIT-1131-05 / AUDIT-1392-P13 — opponent class label/pill must not
//! display a trailing `?` once the opponent class has been resolved via the
//! `S2CClassesRevealed` lobby-mirror path or the snapshot-rebuild path.
//!
//! Before the fix, `sync_class_reveal_hud_system` rewrote the
//! `opponent_gold_prefix` text from the bare `"OPP"` literal to the
//! class-extended form (e.g. `"OPP Iop"`), but the opponent gold *value*
//! sibling (`opponent_gold_parent`) still rendered the `"?"`
//! unpopulated-gold placeholder per PROMPT 1027 / REPAIR-A3. The two
//! entities are flex siblings inside `opponent_gold_pill`, so the
//! rendered HUD read `"OPP Iop ?"` — the trailing `?` was indistinguishable
//! from a class label artefact (audit screenshot PNG 2-000005).
//!
//! After the fix, once `HudClassReveal.opponent` is `Some(_)` the gold
//! placeholder for the opponent silently collapses to the empty string so
//! the pill reads as `"OPP Iop"` (no trailing glyph) until the first
//! authoritative gold value lands. Pre-reveal behaviour is unchanged: the
//! bare `"?"` still surfaces before any class identity is known so the
//! PROMPT 1027 hidden-information signal survives.
//!
//! HUD timer/phase/mana/reserve behaviour is preserved. This bin asserts
//! only against the OPP pill state — it never reads or writes the HUD
//! timer eyeball carry condition.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::state::{ClientSessionIdentity, ClientState};
use client::ui::hud::{format_opp_class_display, HudEntities, HudPlayerIds, HudPlugin};
use client::ui::lobby::LobbyViewState;
use shared::card::ClassId;
use shared::protocol::GameMode;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
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

fn hud_entities(app: &App) -> HudEntities {
    *app.world().resource::<HudEntities>()
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
        room_list: Vec::new(),
    });
    app.world_mut().insert_resource(ClientSessionIdentity {
        player_id: Some(player(1)),
        session_id: None,
        session_token: None,
    });
}

/// Pre-reveal cold start: the bare `"OPP"` prefix and the `"?"` gold
/// placeholder are intentional (PROMPT 1027 hidden-information signal).
/// The PROMPT 1400 fix MUST NOT regress this state — the `?` surfaces
/// only until class identity becomes known.
#[test]
fn pre_reveal_cold_start_keeps_question_mark_placeholder() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(text_value(&app, entities.opponent_gold_prefix), "OPP");
    assert_eq!(text_value(&app, entities.opponent_gold_parent), "?");
}

/// AC: once the opponent class is revealed via the lobby mirror path, the
/// `opponent_gold_prefix` rewrites to the class-extended form and the
/// `opponent_gold_parent` placeholder collapses to empty so the rendered
/// pill reads as `"OPP Iop"` instead of `"OPP Iop ?"`.
#[test]
fn opp_class_revealed_via_lobby_mirror_strips_trailing_question_mark() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    insert_lobby_reveal(&mut app, ClassId::Iop, ClassId::Iop);
    app.update();
    app.update();

    let prefix = text_value(&app, entities.opponent_gold_prefix);
    let value = text_value(&app, entities.opponent_gold_parent);

    // Prefix must carry the class identity per the existing
    // `format_opp_class_display` contract (AUDIT-1076-16 closure).
    assert_eq!(prefix, format_opp_class_display(ClassId::Iop));
    assert!(prefix.contains("Iop"), "prefix should contain class name");

    // PROMPT 1400 / AUDIT-1131-05 — value must NOT carry the trailing `?`
    // once class is known. Empty string is the canonical post-reveal,
    // pre-gold placeholder.
    assert!(
        !value.contains('?'),
        "opponent_gold_parent must not contain '?' after class reveal; got {value:?}"
    );
    assert_eq!(
        value, "",
        "expected empty post-reveal placeholder; got {value:?}"
    );
}

/// AC: the post-reveal placeholder collapse must not survive into the
/// populated state — once gold is authoritative the value reads as
/// `"{N}g"` (e.g. `"5g"`) regardless of class-reveal status.
#[test]
fn opp_gold_populated_overrides_post_reveal_placeholder() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    insert_lobby_reveal(&mut app, ClassId::Iop, ClassId::Iop);
    // Manually populate opponent gold like a test-side gold broadcast
    // would after `HudPlayerIds` was inserted — `sync_gold_text_system`
    // reads the populated state and prints "{gold}g".
    {
        let mut state = app
            .world_mut()
            .get_mut::<client::ui::hud::GoldDisplayState>(entities.opponent_gold_parent)
            .expect("opponent gold state should exist");
        state.gold = 5.0;
        state.is_populated = true;
    }
    app.update();
    app.update();

    let value = text_value(&app, entities.opponent_gold_parent);
    assert_eq!(value, "5g");
}

/// HUD-side invariant lock for AUDIT-1131-04 / AUDIT-1392-P12 —
/// the canonical HUD top strip is the sole HUD-owned mana/reserve
/// readout surface. No other HUD `Text` entity may carry the verbose
/// duplicate-mana wording (`"Reserve "`, `"Current "`, or the standalone
/// substring `"mana"` in a non-prefix label) that the floating microbadge
/// historically used. The actual duplicate microbadge spawn site lives in
/// `client/src/ui/hand/mod.rs` (the per-card Reserve strip), which the
/// PROMPT 1400 worker is forbidden to touch — this regression-lock
/// guarantees the duplicate cannot re-enter HUD code without breaking a
/// HUD-specific test, even though the visible bug cannot be closed from
/// HUD-only scope this prompt. See the PROMPT 1400 report for the
/// follow-up disposition.
///
/// Allowed exceptions:
/// - `entities.mana_label`: the canonical mana readout text ("X / Y").
/// - `entities.reserve_label`: the canonical reserve diamond text
///   ("+N reserve") — explicitly the *intended* HUD strip ownership of
///   reserve state.
/// - `entities.mana_prefix`: the static "MANA" pill prefix label.
#[test]
fn hud_owned_text_never_carries_duplicate_mana_microbadge_wording() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    let hud_root = entities.root;
    let allowed: [Entity; 3] = [
        entities.mana_label,
        entities.reserve_label,
        entities.mana_prefix,
    ];

    let texts: Vec<(Entity, String)> = {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &Text)>();
        query
            .iter(world)
            .map(|(entity, text)| (entity, text.0.clone()))
            .collect()
    };

    let mut violations: Vec<(Entity, String)> = Vec::new();
    for (entity, text) in texts {
        if !is_descendant_of(app.world(), entity, hud_root) {
            continue;
        }
        if allowed.contains(&entity) {
            continue;
        }

        // Case-insensitive `mana` substring catches any text that smells
        // like a HUD-side mana readout duplicate. Exact-case `Reserve ` /
        // `Current ` catch the verbose AUDIT-1076-17 microbadge wording.
        let lower = text.to_lowercase();
        let hits_mana = lower.contains("mana");
        let hits_reserve_verbose = text.contains("Reserve ");
        let hits_current_verbose = text.contains("Current ");
        if hits_mana || hits_reserve_verbose || hits_current_verbose {
            violations.push((entity, text));
        }
    }

    assert!(
        violations.is_empty(),
        "HUD subtree carries unexpected mana-microbadge wording on Text entities outside the canonical mana/reserve strip: {violations:?}"
    );
}

fn is_descendant_of(world: &World, entity: Entity, root: Entity) -> bool {
    let mut current = entity;
    loop {
        if current == root {
            return true;
        }
        match world.get::<ChildOf>(current) {
            Some(child_of) => current = child_of.parent(),
            None => return false,
        }
    }
}
