//! Sprint 14 / Story 015 - S11-UX-HUD-TOP-STRIP-LAYOUT integration tests.
//!
//! This bin is intentionally read-only over client presentation state. It
//! verifies the HUD top strip composition refactor without introducing any
//! server-authoritative state writes or schedule changes.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::GlobalZIndex;
use client::state::ClientState;
use client::ui::design_tokens::{strips, z_layers};
use client::ui::hud::{
    CurrentManaShape, GoldDisplayState, GoldLabelOwner, HudEntities, HudEntity, HudPlugin, HudRoot,
    HudTimerBar, HudTopStrip, ManaLabel, PhaseLabel, ReserveManaLabel, ReserveManaShape,
    RoundCounter, HUD_ENTITY_COUNT,
};

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

fn parent_of(app: &App, entity: Entity) -> Entity {
    app.world()
        .get::<ChildOf>(entity)
        .unwrap_or_else(|| panic!("{entity:?} should have a ChildOf parent"))
        .parent()
}

fn is_descendant_of(app: &App, entity: Entity, ancestor: Entity) -> bool {
    let mut current = entity;
    for _ in 0..8 {
        if current == ancestor {
            return true;
        }
        let Some(parent) = app.world().get::<ChildOf>(current) else {
            return false;
        };
        current = parent.parent();
    }
    false
}

fn count_with<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.iter(app.world()).count()
}

fn assert_source_order(text: &str, needles: &[&str]) {
    let mut cursor = 0;
    for needle in needles {
        let Some(offset) = text[cursor..].find(needle) else {
            panic!("expected source to contain `{needle}` after byte {cursor}");
        };
        cursor += offset + needle.len();
    }
}

#[test]
fn ac1_spawns_single_header_bar_top_strip_with_readouts_under_it() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert!(app.world().get::<HudRoot>(entities.root).is_some());
    assert!(app.world().get::<HudTopStrip>(entities.top_strip).is_some());
    assert!(app
        .world()
        .get::<strips::HeaderBar>(entities.top_strip)
        .is_some());
    assert_eq!(parent_of(&app, entities.top_strip), entities.root);

    let top_strip_node = app
        .world()
        .get::<Node>(entities.top_strip)
        .expect("HudTopStrip should carry a Node");
    assert_eq!(top_strip_node.display, Display::Flex);
    assert_eq!(top_strip_node.height, Val::Px(strips::HEADER_BAR_HEIGHT_PX));

    for child in [
        entities.phase_label,
        entities.round_counter,
        entities.own_gold_parent,
        entities.opponent_gold_parent,
        entities.mana_label,
        entities.reserve_container,
        entities.timer_bar,
    ] {
        assert_eq!(
            parent_of(&app, child),
            entities.top_strip,
            "{child:?} should be a direct HudTopStrip child"
        );
    }

    for descendant in [
        entities.own_gold_span,
        entities.opponent_gold_span,
        entities.reserve_label,
    ] {
        assert!(
            is_descendant_of(&app, descendant, entities.top_strip),
            "{descendant:?} should remain under HudTopStrip"
        );
    }
}

#[test]
fn ac2_top_strip_children_do_not_use_absolute_offset_nodes() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    for child in [
        entities.phase_label,
        entities.round_counter,
        entities.own_gold_parent,
        entities.opponent_gold_parent,
        entities.mana_label,
        entities.reserve_container,
        entities.timer_bar,
    ] {
        let node = app
            .world()
            .get::<Node>(child)
            .unwrap_or_else(|| panic!("{child:?} should carry a Node"));
        assert_ne!(
            node.position_type,
            PositionType::Absolute,
            "{child:?} should compose through HudTopStrip flex layout"
        );
    }

    let text = read_client_source("ui/hud/mod.rs");
    let forbidden = [
        "top_left_node(",
        "top_left_second_line_node(",
        "top_right_node(",
        "current_mana_bar_node(config.hud_margin_px",
        "reserve_mana_diamond_node(margin_px",
        "top: Val::Px(margin_px",
        "left: Val::Px(margin_px",
        "right: Val::Px(margin_px",
        "bottom: Val::Px(margin_px",
    ];
    let mut violations = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for needle in forbidden {
            if line.contains(needle) {
                violations.push(format!("{}: {}", line_no + 1, line.trim_end()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "AC2 absolute-offset grep guard found top-strip legacy offsets:\n{}",
        violations.join("\n")
    );
}

#[test]
fn ac3_hud_entities_preserve_existing_fields_and_add_top_strip() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert!(
        app.world().get::<HudEntity>(entities.top_strip).is_none(),
        "HudTopStrip is structural and should not change HUD_ENTITY_COUNT"
    );

    assert!(app
        .world()
        .get::<PhaseLabel>(entities.phase_label)
        .is_some());
    assert!(app
        .world()
        .get::<RoundCounter>(entities.round_counter)
        .is_some());
    assert!(matches!(
        app.world().get::<GoldLabelOwner>(entities.own_gold_parent),
        Some(&GoldLabelOwner::Local)
    ));
    assert!(matches!(
        app.world()
            .get::<GoldLabelOwner>(entities.opponent_gold_parent),
        Some(&GoldLabelOwner::Opponent)
    ));
    assert!(app
        .world()
        .get::<GoldDisplayState>(entities.own_gold_parent)
        .is_some());
    assert!(app
        .world()
        .get::<GoldDisplayState>(entities.opponent_gold_parent)
        .is_some());
    assert!(app
        .world()
        .get::<TextSpan>(entities.own_gold_span)
        .is_some());
    assert!(app
        .world()
        .get::<TextSpan>(entities.opponent_gold_span)
        .is_some());
    assert!(app.world().get::<ManaLabel>(entities.mana_label).is_some());
    assert!(app
        .world()
        .get::<CurrentManaShape>(entities.mana_label)
        .is_some());
    assert!(app
        .world()
        .get::<ReserveManaShape>(entities.reserve_container)
        .is_some());
    assert!(app
        .world()
        .get::<ReserveManaLabel>(entities.reserve_label)
        .is_some());
    assert!(app.world().get::<HudTimerBar>(entities.timer_bar).is_some());
}

#[test]
fn ac4_hud_plugin_schedule_source_remains_unchanged() {
    let text = read_client_source("ui/hud/mod.rs");

    assert_eq!(
        text.matches(".configure_sets(").count(),
        1,
        "HudPlugin should still configure one HUD set chain"
    );
    assert_source_order(
        &text,
        &[
            "HudSystemSet::PhaseTransition",
            "HudSystemSet::MessageDrain",
            "HudSystemSet::StateSync",
            ".chain()",
            ".run_if(in_state(ClientState::InSession))",
        ],
    );
    assert!(text.contains(".add_systems(OnEnter(ClientState::InSession), spawn_hud)"));
    assert!(text.contains(".add_systems(OnExit(ClientState::InSession), despawn_hud)"));
    assert!(
        !text.contains("HudSystemSet::AnimationTick"),
        "story 015 should not add HUD schedule sets"
    );
}

#[test]
fn ac9_top_strip_font_size_lines_do_not_use_viewport_scaled_values() {
    let text = read_client_source("ui/hud/mod.rs");
    let mut violations = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let touches_text_font = line.contains("font_size") || line.contains("TextFont");
        let viewport_scaled =
            line.contains("Val::Percent") || line.contains("Val::Vw") || line.contains("Val::Vh");
        if touches_text_font && viewport_scaled {
            violations.push(format!("{}: {}", line_no + 1, line.trim_end()));
        }
    }
    assert!(
        violations.is_empty(),
        "AC9 forbids viewport-scaled HUD TextFont/font_size lines:\n{}",
        violations.join("\n")
    );
}

#[test]
fn ac10_root_and_top_strip_consume_ui_base_z_layer() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(
        app.world().get::<GlobalZIndex>(entities.root),
        Some(&z_layers::UI_BASE),
        "HudRoot should consume z_layers::UI_BASE"
    );
    assert_eq!(
        app.world().get::<GlobalZIndex>(entities.top_strip),
        Some(&z_layers::UI_BASE),
        "HudTopStrip should consume z_layers::UI_BASE"
    );
}
