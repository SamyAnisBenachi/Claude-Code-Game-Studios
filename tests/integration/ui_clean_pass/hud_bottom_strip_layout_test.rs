//! Sprint 14 / Story 016 - S14-HUD-BOTTOM-STRIP-LAYOUT integration tests.
//!
//! This bin verifies the bottom-strip structural parent for the own-player
//! figurine while keeping mana and reserve readouts owned by HudTopStrip.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::GlobalZIndex;
use client::state::ClientState;
use client::ui::design_tokens::{strips, z_layers};
use client::ui::hud::{
    CurrentManaShape, HudBottomStrip, HudEntities, HudEntity, HudFigurine, HudPlugin, HudRoot,
    HudTopStrip, ManaShapeGeometry, ManaShapeKind, ReserveManaLabel, ReserveManaShape,
    ScoreboardDot, CURRENT_MANA_BAR_HEIGHT_PX, CURRENT_MANA_BAR_WIDTH_PX, HUD_ENTITY_COUNT,
    RESERVE_MANA_DIAMOND_ROTATION_DEGREES, RESERVE_MANA_DIAMOND_SIZE_PX,
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

fn direct_children(app: &App, parent: Entity) -> Vec<Entity> {
    app.world()
        .get::<Children>(parent)
        .map(|children| children.iter().collect())
        .unwrap_or_default()
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

fn shape_geometry(app: &App, entity: Entity) -> ManaShapeGeometry {
    *app.world()
        .get::<ManaShapeGeometry>(entity)
        .expect("mana shape geometry should exist")
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
fn ac1_spawns_bottom_strip_under_root_with_figurine_child() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(count_with::<HudBottomStrip>(&mut app), 1);
    assert!(app.world().get::<HudRoot>(entities.root).is_some());
    assert!(app
        .world()
        .get::<HudBottomStrip>(entities.bottom_strip)
        .is_some());
    assert!(app
        .world()
        .get::<strips::FooterBar>(entities.bottom_strip)
        .is_some());
    assert_eq!(parent_of(&app, entities.bottom_strip), entities.root);

    let bottom_strip_node = app
        .world()
        .get::<Node>(entities.bottom_strip)
        .expect("HudBottomStrip should carry a Node");
    assert_eq!(bottom_strip_node.display, Display::Flex);
    assert_eq!(
        bottom_strip_node.height,
        Val::Px(strips::FOOTER_BAR_HEIGHT_PX)
    );

    assert_eq!(parent_of(&app, entities.figurine), entities.bottom_strip);
    assert!(
        is_descendant_of(&app, entities.figurine, entities.bottom_strip),
        "figurine should be hosted by HudBottomStrip"
    );
}

#[test]
fn ac2_bottom_strip_children_do_not_use_absolute_offsets() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    let children = direct_children(&app, entities.bottom_strip);
    assert!(
        children.contains(&entities.figurine),
        "HudBottomStrip should directly own the figurine"
    );
    for child in children {
        let node = app
            .world()
            .get::<Node>(child)
            .unwrap_or_else(|| panic!("{child:?} should carry a Node"));
        assert_ne!(
            node.position_type,
            PositionType::Absolute,
            "{child:?} should compose through HudBottomStrip flex layout"
        );
    }

    let text = read_client_source("ui/hud/mod.rs");
    let forbidden = [
        "bottom: Val::Px(strips::FOOTER_BAR_HEIGHT_PX + spacing::SPACING_XL)",
        "ChildOf(root),",
    ];
    let mut violations = Vec::new();
    let mut inside_figurine_spawn = false;
    for (line_no, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if line.contains("Name::new(\"HUD Class Figurine\")") {
            inside_figurine_spawn = true;
        }
        if inside_figurine_spawn {
            for needle in forbidden {
                if line.contains(needle) {
                    violations.push(format!("{}: {}", line_no + 1, line.trim_end()));
                }
            }
            if line.contains(".id();") {
                inside_figurine_spawn = false;
            }
        }
    }
    assert!(
        violations.is_empty(),
        "AC2 found legacy root/absolute figurine layout:\n{}",
        violations.join("\n")
    );
}

#[test]
fn ac3_hud_entities_preserve_count_and_bottom_strip_is_structural() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(count_with::<HudEntity>(&mut app), HUD_ENTITY_COUNT);
    assert!(
        app.world()
            .get::<HudEntity>(entities.bottom_strip)
            .is_none(),
        "HudBottomStrip is structural and should not change HUD_ENTITY_COUNT"
    );
    assert!(app.world().get::<HudFigurine>(entities.figurine).is_some());
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
}

#[test]
fn ac4_mana_and_reserve_remain_top_strip_owned_and_shape_distinct() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert!(app.world().get::<HudTopStrip>(entities.top_strip).is_some());
    assert_eq!(parent_of(&app, entities.mana_label), entities.top_strip);
    assert_eq!(
        parent_of(&app, entities.reserve_container),
        entities.top_strip
    );
    assert!(
        is_descendant_of(&app, entities.reserve_label, entities.top_strip),
        "reserve label should remain under HudTopStrip"
    );
    for top_readout in [
        entities.mana_label,
        entities.reserve_container,
        entities.reserve_label,
    ] {
        assert!(
            !is_descendant_of(&app, top_readout, entities.bottom_strip),
            "{top_readout:?} must not move into HudBottomStrip"
        );
    }

    let current_geometry = shape_geometry(&app, entities.mana_label);
    assert_eq!(current_geometry.kind, ManaShapeKind::Bar);
    assert_eq!(current_geometry.width_px, CURRENT_MANA_BAR_WIDTH_PX);
    assert_eq!(current_geometry.height_px, CURRENT_MANA_BAR_HEIGHT_PX);
    assert_eq!(current_geometry.rotation_degrees, 0.0);

    let reserve_geometry = shape_geometry(&app, entities.reserve_container);
    assert_eq!(reserve_geometry.kind, ManaShapeKind::Diamond);
    assert_eq!(reserve_geometry.width_px, RESERVE_MANA_DIAMOND_SIZE_PX);
    assert_eq!(reserve_geometry.height_px, RESERVE_MANA_DIAMOND_SIZE_PX);
    assert_eq!(
        reserve_geometry.rotation_degrees,
        RESERVE_MANA_DIAMOND_ROTATION_DEGREES
    );
}

#[test]
fn ac5_hud_plugin_schedule_source_remains_unchanged() {
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
        "story 016 should not add HUD schedule sets"
    );
}

#[test]
fn ac10_hud_font_size_lines_do_not_use_viewport_scaled_values() {
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
        "AC10 forbids viewport-scaled HUD TextFont/font_size lines:\n{}",
        violations.join("\n")
    );
}

#[test]
fn ac11_root_and_bottom_strip_consume_ui_base_z_layer() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    assert_eq!(
        app.world().get::<GlobalZIndex>(entities.root),
        Some(&z_layers::UI_BASE),
        "HudRoot should consume z_layers::UI_BASE"
    );
    assert_eq!(
        app.world().get::<GlobalZIndex>(entities.bottom_strip),
        Some(&z_layers::UI_BASE),
        "HudBottomStrip should consume z_layers::UI_BASE"
    );
}

#[test]
fn ac12_bottom_strip_does_not_claim_scoreboard_objective_identity() {
    test_helpers::init_test_tracing();
    let app = app_with_hud_in_session();
    let entities = hud_entities(&app);

    for row in entities.dots {
        for dot in row {
            assert!(app.world().get::<ScoreboardDot>(dot).is_some());
            assert!(
                !is_descendant_of(&app, dot, entities.bottom_strip),
                "scoreboard dots should remain outside HudBottomStrip"
            );
        }
    }
}
