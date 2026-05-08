use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::asset_wiring::{insert_placeholder_assets, remove_placeholder_assets};
use client::state::ClientState;
use client::ui::hand::{
    FanSlotIndex, HandCardFrame, HandRarityIcon, HandTypeIcon, StatBadgeAr, StatBadgeAtk,
    StatBadgeHp, StatBadgeMp, HAND_FAN_SLOT_COUNT,
};

// ── App builder ───────────────────────────────────────────────────────────────

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_state::<ClientState>();
    app.add_systems(OnEnter(ClientState::InSession), insert_placeholder_assets);
    app.add_systems(OnExit(ClientState::InSession), remove_placeholder_assets);
    app.add_systems(
        OnEnter(ClientState::InSession),
        client::ui::hand::spawn_hand_ui.after(insert_placeholder_assets),
    );
    app
}

fn enter_session(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    // First update applies the state transition + OnEnter; second flushes deferred spawns.
    app.update();
    app.update();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn count_with_image_node<M: Component>(app: &mut App) -> usize {
    let mut q = app.world_mut().query_filtered::<&ImageNode, With<M>>();
    q.iter(app.world()).count()
}

fn count_child_of_with<M: Component>(app: &mut App) -> usize {
    let mut q = app.world_mut().query_filtered::<&ChildOf, (With<M>, With<ImageNode>)>();
    q.iter(app.world()).count()
}

// ── PAW-002-f: ImageNode present on all chrome child entities ─────────────────

/// Every fan slot must have exactly one HandCardFrame child with ImageNode.
#[test]
fn test_fan_slot_chrome_card_frame_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(
        count_child_of_with::<HandCardFrame>(&mut app),
        HAND_FAN_SLOT_COUNT,
        "Expected {HAND_FAN_SLOT_COUNT} HandCardFrame children with ImageNode"
    );
}

/// Every fan slot must have a StatBadgeAtk child with ImageNode.
#[test]
fn test_fan_slot_chrome_stat_badge_atk_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(count_child_of_with::<StatBadgeAtk>(&mut app), HAND_FAN_SLOT_COUNT);
}

/// Every fan slot must have a StatBadgeHp child with ImageNode.
#[test]
fn test_fan_slot_chrome_stat_badge_hp_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(count_child_of_with::<StatBadgeHp>(&mut app), HAND_FAN_SLOT_COUNT);
}

/// Every fan slot must have a StatBadgeMp child with ImageNode.
#[test]
fn test_fan_slot_chrome_stat_badge_mp_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(count_child_of_with::<StatBadgeMp>(&mut app), HAND_FAN_SLOT_COUNT);
}

/// Every fan slot must have a StatBadgeAr child with ImageNode.
#[test]
fn test_fan_slot_chrome_stat_badge_ar_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(count_child_of_with::<StatBadgeAr>(&mut app), HAND_FAN_SLOT_COUNT);
}

/// Every fan slot must have a HandRarityIcon child with ImageNode.
#[test]
fn test_fan_slot_chrome_rarity_icon_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(count_child_of_with::<HandRarityIcon>(&mut app), HAND_FAN_SLOT_COUNT);
}

/// Every fan slot must have a HandTypeIcon child with ImageNode.
#[test]
fn test_fan_slot_chrome_type_icon_image_node_present() {
    let mut app = make_app();
    enter_session(&mut app);
    assert_eq!(count_child_of_with::<HandTypeIcon>(&mut app), HAND_FAN_SLOT_COUNT);
}

/// Chrome entities must be parented to FanSlotIndex entities.
#[test]
fn test_fan_slot_chrome_children_parent_is_fan_slot() {
    let mut app = make_app();
    enter_session(&mut app);

    // Collect all FanSlotIndex entity IDs.
    let mut slot_q = app.world_mut().query_filtered::<Entity, With<FanSlotIndex>>();
    let slot_entities: std::collections::HashSet<Entity> =
        slot_q.iter(app.world()).collect();

    // Check that HandCardFrame parents are all fan slots.
    let mut frame_q = app.world_mut().query_filtered::<&ChildOf, With<HandCardFrame>>();
    let frame_parents: Vec<Entity> = frame_q.iter(app.world()).map(|co| co.parent()).collect();

    for parent in frame_parents {
        assert!(
            slot_entities.contains(&parent),
            "HandCardFrame parent {:?} is not a FanSlotIndex entity",
            parent
        );
    }
}

// ── PAW-002-e: Frame ImageNode handle is non-default ─────────────────────────

/// HandCardFrame ImageNode must not hold the default (empty) handle.
#[test]
fn test_fan_slot_chrome_card_frame_handle_non_default() {
    let mut app = make_app();
    enter_session(&mut app);

    let mut q = app.world_mut().query_filtered::<&ImageNode, With<HandCardFrame>>();
    let handles: Vec<_> = q.iter(app.world()).map(|img| img.image.clone()).collect();

    assert!(
        !handles.is_empty(),
        "No HandCardFrame entities found — spawn must have run"
    );
    for handle in handles {
        assert_ne!(
            handle,
            Handle::default(),
            "HandCardFrame ImageNode must have a non-default handle"
        );
    }
}

/// Stat badge handles must all be non-default.
#[test]
fn test_fan_slot_chrome_stat_badge_handles_non_default() {
    let mut app = make_app();
    enter_session(&mut app);

    let mut q_atk = app.world_mut().query_filtered::<&ImageNode, With<StatBadgeAtk>>();
    for img in q_atk.iter(app.world()) {
        assert_ne!(img.image, Handle::default(), "StatBadgeAtk handle must be non-default");
    }

    let mut q_hp = app.world_mut().query_filtered::<&ImageNode, With<StatBadgeHp>>();
    for img in q_hp.iter(app.world()) {
        assert_ne!(img.image, Handle::default(), "StatBadgeHp handle must be non-default");
    }

    let mut q_mp = app.world_mut().query_filtered::<&ImageNode, With<StatBadgeMp>>();
    for img in q_mp.iter(app.world()) {
        assert_ne!(img.image, Handle::default(), "StatBadgeMp handle must be non-default");
    }

    let mut q_ar = app.world_mut().query_filtered::<&ImageNode, With<StatBadgeAr>>();
    for img in q_ar.iter(app.world()) {
        assert_ne!(img.image, Handle::default(), "StatBadgeAr handle must be non-default");
    }
}

// ── PAW-002-g: No UiImage in the world ───────────────────────────────────────
// UiImage does not exist in Bevy 0.18 — enforced at compile time.
