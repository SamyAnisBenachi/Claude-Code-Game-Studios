use std::collections::HashMap;
use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::{prelude::*, time::Virtual};
use bevy_tweening::TweeningPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, FanSlotState, HandCardCatalog, HandCardFrame, HandRarityIcon, HandTypeIcon,
    HandUiCardAcquiredReceived, HandUiPlugin, HandUiTimingConfig, StatBadgeAr, StatBadgeAtk,
    StatBadgeHp, StatBadgeMp,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

// PROMPT 682 / Finding B v2 V3 Worker B — chrome composition regression.
// PROMPT 669's Verdict B PROVED that the 7 chrome children spawned at
// `client/src/ui/hand/mod.rs` L2566-2618 were created with `Node::default()`
// (width: Val::Auto, height: Val::Auto, position_type: Relative) and therefore
// rendered as a 0×0 row inside the fan slot — invisible even after Worker A
// (PROMPT 671) put the slot itself on-screen.
//
// This test asserts the composition contract for HU-card-slot-chrome-layout
// (story-016): every chrome child of every occupied slot must carry an
// Absolute-positioned Node with a non-zero `width`, AND must anchor against
// the slot's local box via a `Val::Percent` offset (not flow as a sibling).
//
// Coverage matches the three ACs of story-016 (HU-CHROME-01/02/03).

const ACQUIRED_CARD_COUNT: usize = 2;
const FIRST_ACQUIRED_CARD_ID: u32 = 50;
const LAYOUT_CONVERGENCE_FRAMES: usize = 4;
const VIEWPORT_WIDTH: f32 = 1280.0;
const VIEWPORT_HEIGHT: f32 = 720.0;

const EXPECTED_STAT_BADGE_PERCENT: f32 = 20.0;
const EXPECTED_ICON_PERCENT: f32 = 15.0;
const EXPECTED_ICON_CENTER_LEFT_PERCENT: f32 = (100.0 - EXPECTED_ICON_PERCENT) / 2.0;

#[test]
fn fan_slot_chrome_children_have_absolute_layout_after_placement_entry() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui_at_resolution(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);

    for offset in 0..ACQUIRED_CARD_COUNT {
        let card_id = CardId(FIRST_ACQUIRED_CARD_ID + offset as u32);
        app.world_mut()
            .write_message(HandUiCardAcquiredReceived { card_id });
        run_update(&mut app);
    }

    set_phase(&mut app, RoundPhase::Placement);
    for _ in 0..LAYOUT_CONVERGENCE_FRAMES {
        run_update(&mut app);
    }

    let occupied_slots: Vec<Entity> = (0..ACQUIRED_CARD_COUNT as u8)
        .map(|index| fan_slot(&mut app, index))
        .collect();

    for (slot_index, slot) in occupied_slots.iter().enumerate() {
        assert_eq!(
            app.world().get::<FanSlotState>(*slot).copied(),
            Some(FanSlotState::Active),
            "fan slot {slot_index} must be promoted to Active after PLACEMENT entry",
        );
    }

    let slot_children: Vec<Entity> = occupied_slots
        .iter()
        .flat_map(|slot| children_of(&mut app, *slot))
        .collect();

    // Every chrome child of every occupied slot must satisfy HU-CHROME-02
    // (width is not Val::Auto and not Val::Px(0.0)) and HU-CHROME-03
    // (position_type == Absolute). This is the canary that fires if any of the
    // 7 spawn sites is reverted to `Node::default()`.
    for child in &slot_children {
        if let Some(node) = app.world().get::<Node>(*child) {
            assert_eq!(
                node.position_type,
                PositionType::Absolute,
                "chrome child {child:?} must declare PositionType::Absolute (HU-CHROME-03); \
                 a default (Relative) value forces flow-layout siblings instead of slot-local \
                 corner anchoring",
            );
            assert!(
                !matches!(node.width, Val::Auto),
                "chrome child {child:?} Node.width must not be Val::Auto (HU-CHROME-02 \
                 regression canary — Node::default() reproduces the Verdict B 0×0 bug)",
            );
            assert!(
                !matches!(node.width, Val::Px(0.0)),
                "chrome child {child:?} Node.width must not be Val::Px(0.0) (HU-CHROME-02 \
                 regression canary)",
            );
            let width_pct = expect_percent(node.width);
            assert!(
                width_pct > 0.0,
                "chrome child {child:?} Node.width percent must be > 0; got {width_pct}",
            );
        }
    }

    // HandCardFrame: 100% × 100% overlay anchored top-left.
    let frames = collect_marker_nodes::<HandCardFrame>(&mut app, &occupied_slots);
    assert_eq!(
        frames.len(),
        ACQUIRED_CARD_COUNT,
        "expected one HandCardFrame per occupied slot",
    );
    for (slot_index, _, node) in &frames {
        assert_node_percent(*slot_index, "HandCardFrame", &node.width, 100.0);
        assert_node_percent(*slot_index, "HandCardFrame", &node.height, 100.0);
        assert_node_percent(*slot_index, "HandCardFrame", &node.left, 0.0);
        assert_node_percent(*slot_index, "HandCardFrame", &node.top, 0.0);
    }

    // Stat badges: 20% × 20% in each of the four corners.
    assert_stat_badge_corner::<StatBadgeMp>(&mut app, &occupied_slots, "StatBadgeMp", Corner::TL);
    assert_stat_badge_corner::<StatBadgeAr>(&mut app, &occupied_slots, "StatBadgeAr", Corner::TR);
    assert_stat_badge_corner::<StatBadgeAtk>(&mut app, &occupied_slots, "StatBadgeAtk", Corner::BL);
    assert_stat_badge_corner::<StatBadgeHp>(&mut app, &occupied_slots, "StatBadgeHp", Corner::BR);

    // Rarity / Type icons: 15% × 15%, centered horizontally, top vs. bottom.
    assert_icon_anchor::<HandRarityIcon>(
        &mut app,
        &occupied_slots,
        "HandRarityIcon",
        IconAnchor::Top,
    );
    assert_icon_anchor::<HandTypeIcon>(
        &mut app,
        &occupied_slots,
        "HandTypeIcon",
        IconAnchor::Bottom,
    );
}

#[derive(Debug, Clone, Copy)]
enum Corner {
    TL,
    TR,
    BL,
    BR,
}

#[derive(Debug, Clone, Copy)]
enum IconAnchor {
    Top,
    Bottom,
}

fn assert_stat_badge_corner<M: Component>(
    app: &mut App,
    slots: &[Entity],
    label: &'static str,
    corner: Corner,
) {
    let badges = collect_marker_nodes::<M>(app, slots);
    assert_eq!(
        badges.len(),
        slots.len(),
        "expected one {label} per occupied slot; got {}",
        badges.len(),
    );
    for (slot_index, _, node) in &badges {
        assert_node_percent(*slot_index, label, &node.width, EXPECTED_STAT_BADGE_PERCENT);
        assert_node_percent(
            *slot_index,
            label,
            &node.height,
            EXPECTED_STAT_BADGE_PERCENT,
        );
        match corner {
            Corner::TL => {
                assert_node_percent(*slot_index, label, &node.left, 0.0);
                assert_node_percent(*slot_index, label, &node.top, 0.0);
                assert!(
                    matches!(node.right, Val::Auto),
                    "{label} slot {slot_index} expected Node.right == Val::Auto for top-left \
                     corner; got {:?}",
                    node.right,
                );
                assert!(
                    matches!(node.bottom, Val::Auto),
                    "{label} slot {slot_index} expected Node.bottom == Val::Auto for top-left \
                     corner; got {:?}",
                    node.bottom,
                );
            }
            Corner::TR => {
                assert_node_percent(*slot_index, label, &node.right, 0.0);
                assert_node_percent(*slot_index, label, &node.top, 0.0);
                assert!(matches!(node.left, Val::Auto));
                assert!(matches!(node.bottom, Val::Auto));
            }
            Corner::BL => {
                assert_node_percent(*slot_index, label, &node.left, 0.0);
                assert_node_percent(*slot_index, label, &node.bottom, 0.0);
                assert!(matches!(node.right, Val::Auto));
                assert!(matches!(node.top, Val::Auto));
            }
            Corner::BR => {
                assert_node_percent(*slot_index, label, &node.right, 0.0);
                assert_node_percent(*slot_index, label, &node.bottom, 0.0);
                assert!(matches!(node.left, Val::Auto));
                assert!(matches!(node.top, Val::Auto));
            }
        }
    }
}

fn assert_icon_anchor<M: Component>(
    app: &mut App,
    slots: &[Entity],
    label: &'static str,
    anchor: IconAnchor,
) {
    let icons = collect_marker_nodes::<M>(app, slots);
    assert_eq!(
        icons.len(),
        slots.len(),
        "expected one {label} per occupied slot; got {}",
        icons.len(),
    );
    for (slot_index, _, node) in &icons {
        assert_node_percent(*slot_index, label, &node.width, EXPECTED_ICON_PERCENT);
        assert_node_percent(*slot_index, label, &node.height, EXPECTED_ICON_PERCENT);
        assert_node_percent(
            *slot_index,
            label,
            &node.left,
            EXPECTED_ICON_CENTER_LEFT_PERCENT,
        );
        match anchor {
            IconAnchor::Top => {
                assert_node_percent(*slot_index, label, &node.top, 0.0);
                assert!(matches!(node.bottom, Val::Auto));
            }
            IconAnchor::Bottom => {
                assert_node_percent(*slot_index, label, &node.bottom, 0.0);
                assert!(matches!(node.top, Val::Auto));
            }
        }
    }
}

fn collect_marker_nodes<M: Component>(
    app: &mut App,
    occupied_slots: &[Entity],
) -> Vec<(u8, Entity, Node)> {
    let parent_lookup: HashMap<Entity, u8> = occupied_slots
        .iter()
        .enumerate()
        .map(|(idx, entity)| (*entity, idx as u8))
        .collect();
    let mut query = app.world_mut().query::<(Entity, &ChildOf, &Node, &M)>();
    let mut out: Vec<(u8, Entity, Node)> = query
        .iter(app.world())
        .filter_map(|(entity, child_of, node, _)| {
            parent_lookup
                .get(&child_of.parent())
                .copied()
                .map(|slot_index| (slot_index, entity, node.clone()))
        })
        .collect();
    out.sort_by_key(|(slot, _, _)| *slot);
    out
}

fn assert_node_percent(slot_index: u8, label: &str, value: &Val, expected: f32) {
    match value {
        Val::Percent(actual) => {
            assert!(
                (actual - expected).abs() <= 0.01,
                "{label} slot {slot_index} expected Val::Percent({expected}); got \
                 Val::Percent({actual})",
            );
        }
        other => {
            panic!("{label} slot {slot_index} expected Val::Percent({expected}); got {other:?}",)
        }
    }
}

fn expect_percent(value: Val) -> f32 {
    match value {
        Val::Percent(v) => v,
        other => panic!("expected Val::Percent, got {other:?}"),
    }
}

fn children_of(app: &mut App, parent: Entity) -> Vec<Entity> {
    let mut query = app.world_mut().query::<(Entity, &ChildOf)>();
    query
        .iter(app.world())
        .filter_map(|(entity, child_of)| (child_of.parent() == parent).then_some(entity))
        .collect()
}

fn app_with_hand_ui_at_resolution(width: f32, height: f32) -> App {
    let mut app = base_app();
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(width as u32, height as u32),
            ..default()
        },
        PrimaryWindow,
    ));
    finalize_app(&mut app);
    app
}

fn base_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog {
        cards: test_catalog((FIRST_ACQUIRED_CARD_ID)..(FIRST_ACQUIRED_CARD_ID + 32)),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 5,
        reserve_mana: 0,
        initialized: true,
        ..default()
    });
    app.insert_resource(HandUiTimingConfig {
        card_draw_animation_ms: 280,
        purchase_timeout_ms: 3_000,
        hand_full_notification_duration_ms: 2_000,
    });
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app
}

fn finalize_app(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(app, RoundPhase::DraftInitial);
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn fan_slot(app: &mut App, slot_index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, idx)| (idx.0 == slot_index).then_some(entity))
        .expect("fan slot must exist")
}

fn test_catalog(ids: impl IntoIterator<Item = u32>) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| {
            let card = test_card(id);
            (card.id, card)
        })
        .collect()
}

fn test_card(id: u32) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: format!("Carte {id}"),
        name_en: format!("Card {id}"),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 1,
        atk: 1,
        hp: 2,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}
