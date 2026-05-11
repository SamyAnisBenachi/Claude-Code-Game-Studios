// PROMPT 696 — HU-card-drag MVP producer surface (HU-DRAG-01 .. HU-DRAG-04).
//
// Drives the producer systems added to `client/src/ui/hand/mod.rs` by writing
// the upstream `bevy_picking` events (`Pointer<Press>` / `Pointer<Move>` /
// `Pointer<Release>`) into the message bus the same way bevy_picking does in
// real gameplay. Asserts the producers emit the corresponding HandUi messages
// and that the per-frame follow system trails the drag sprite's `Node.left` /
// `Node.top` to the cursor position.
//
// Scope: stops at the drag-ended emit (PROMPT 696 forbidden zone). PROMPT 697
// owns the board-cell drop completion path that consumes drag-ended.

use std::collections::HashMap;

use bevy::camera::NormalizedRenderTarget;
use bevy::ecs::message::MessageCursor;
use bevy::picking::{
    backend::HitData,
    pointer::{Location, PointerId},
};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use client::card_animations::HandDragSprite;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    ActivePlacementDrag, FanSlotIndex, HandCardCatalog, HandContents, HandFanLayoutConfig,
    HandFanViewport, HandUiEntities, HandUiPlacementCursorMoved, HandUiPlacementDragEnded,
    HandUiPlacementDragStarted, HandUiPlugin,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const VIEWPORT_W: f32 = 1280.0;
const VIEWPORT_H: f32 = 720.0;

#[test]
fn hu_drag_01_press_on_fan_slot_emits_drag_started_and_shows_sprite() {
    test_helpers::init_test_tracing();
    let mut app = app_with_two_acquired_cards_in_placement();

    let slot = fan_slot(&mut app, 0);
    let camera = spawn_dummy_camera(&mut app);
    let drag_sprite = drag_sprite_entity(&app);
    assert_eq!(
        app.world().get::<Visibility>(drag_sprite),
        Some(&Visibility::Hidden),
        "drag sprite must start Hidden"
    );

    let mut started_cursor = drained_cursor::<HandUiPlacementDragStarted>(&app);

    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();

    let starts = messages_since(&app, &mut started_cursor);
    assert_eq!(
        starts.len(),
        1,
        "exactly one HandUiPlacementDragStarted must be emitted by Press on a fan slot"
    );
    assert_eq!(starts[0].card, slot);
    assert_eq!(starts[0].owner_id, PlayerId(1));
    assert_eq!(
        app.world().get::<Visibility>(drag_sprite),
        Some(&Visibility::Visible),
        "drag sprite must flip to Visible the same tick drag-started is consumed"
    );
}

#[test]
fn hu_drag_02_move_during_active_drag_emits_cursor_moved_and_updates_node() {
    test_helpers::init_test_tracing();
    let mut app = app_with_two_acquired_cards_in_placement();
    let slot = fan_slot(&mut app, 0);
    let camera = spawn_dummy_camera(&mut app);
    let drag_sprite = drag_sprite_entity(&app);

    // Start the drag.
    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();

    let mut moved_cursor = drained_cursor::<HandUiPlacementCursorMoved>(&app);

    // First cursor move.
    let first_pos = Vec2::new(720.0, 480.0);
    app.world_mut()
        .write_message(pointer_move(slot, camera, first_pos));
    app.update();

    let moves = messages_since(&app, &mut moved_cursor);
    assert_eq!(
        moves.len(),
        1,
        "Pointer<Move> must produce exactly one HandUiPlacementCursorMoved"
    );
    assert_eq!(moves[0].world_position, Some(first_pos));
    assert_eq!(
        node_position(&app, drag_sprite),
        (Val::Px(first_pos.x), Val::Px(first_pos.y)),
        "drag sprite Node.left/top must follow the first cursor position"
    );

    // Second cursor move — sprite must keep tracking.
    let second_pos = Vec2::new(910.0, 320.0);
    app.world_mut()
        .write_message(pointer_move(slot, camera, second_pos));
    app.update();

    let moves = messages_since(&app, &mut moved_cursor);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].world_position, Some(second_pos));
    assert_eq!(
        node_position(&app, drag_sprite),
        (Val::Px(second_pos.x), Val::Px(second_pos.y)),
        "drag sprite Node.left/top must follow subsequent cursor positions"
    );
}

#[test]
fn hu_drag_03_release_emits_drag_ended_and_clears_active_drag() {
    test_helpers::init_test_tracing();
    let mut app = app_with_two_acquired_cards_in_placement();
    let slot = fan_slot(&mut app, 0);
    let camera = spawn_dummy_camera(&mut app);

    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();
    assert!(
        active_drag_active(&app),
        "ActivePlacementDrag must be active after Press is consumed"
    );
    let mut ended_cursor = drained_cursor::<HandUiPlacementDragEnded>(&app);

    let release_pos = Vec2::new(820.0, 380.0);
    app.world_mut()
        .write_message(pointer_release(slot, camera, release_pos));
    app.update();

    let ends = messages_since(&app, &mut ended_cursor);
    assert_eq!(
        ends.len(),
        1,
        "Pointer<Release> must produce exactly one HandUiPlacementDragEnded"
    );
    // The existing `handle_placement_drag_ended_system` consumes the message
    // and calls `active_drag.clear()`. Sprite Visibility flip back to Hidden is
    // gated on `HandUiPlacementDropResolved`, which the BoardCell drop branch
    // (PROMPT 697) will produce on release-over-cell — out of scope here.
    assert!(
        !active_drag_active(&app),
        "ActivePlacementDrag must be cleared after drag-ended is consumed"
    );
}

#[test]
fn hu_drag_04_full_press_move_release_sequence_tracks_sprite_and_ends_clean() {
    test_helpers::init_test_tracing();
    let mut app = app_with_two_acquired_cards_in_placement();
    let slot = fan_slot(&mut app, 0);
    let camera = spawn_dummy_camera(&mut app);
    let drag_sprite = drag_sprite_entity(&app);

    let mut started_cursor = drained_cursor::<HandUiPlacementDragStarted>(&app);
    let mut moved_cursor = drained_cursor::<HandUiPlacementCursorMoved>(&app);
    let mut ended_cursor = drained_cursor::<HandUiPlacementDragEnded>(&app);

    // Press at fan slot.
    app.world_mut()
        .write_message(pointer_press(slot, camera, Vec2::new(640.0, 600.0)));
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(drag_sprite),
        Some(&Visibility::Visible),
    );

    // Drift towards a board cell location.
    for pos in [Vec2::new(680.0, 540.0), Vec2::new(740.0, 460.0)] {
        app.world_mut()
            .write_message(pointer_move(slot, camera, pos));
        app.update();
        assert_eq!(
            node_position(&app, drag_sprite),
            (Val::Px(pos.x), Val::Px(pos.y))
        );
    }

    // Release over a board-cell-area position.
    let drop_pos = Vec2::new(820.0, 380.0);
    app.world_mut()
        .write_message(pointer_release(slot, camera, drop_pos));
    app.update();

    assert_eq!(messages_since(&app, &mut started_cursor).len(), 1);
    let moves = messages_since(&app, &mut moved_cursor);
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].world_position, Some(Vec2::new(680.0, 540.0)));
    assert_eq!(moves[1].world_position, Some(Vec2::new(740.0, 460.0)));
    assert_eq!(messages_since(&app, &mut ended_cursor).len(), 1);
    // Drag state is fully reset; sprite hide gates on drop-resolved (PROMPT 697).
    assert!(!active_drag_active(&app));
    // The drag sprite was visible during the drag and stays visible until a
    // drop-resolved message hides it — verify the visibility went Visible
    // sometime in the sequence (i.e. the drag-started path ran at least once).
    let _ = drag_sprite;
}

// ── Test app setup ───────────────────────────────────────────────────────────

fn app_with_two_acquired_cards_in_placement() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandFanViewport {
        width_px: VIEWPORT_W,
        height_px: VIEWPORT_H,
    });
    app.insert_resource(HandFanLayoutConfig::default());
    // Minimal catalog so `resolve_placement_target_kind` returns Some
    // for the two acquired CardIds. Without it,
    // `handle_placement_drag_started_system` bails before flipping the
    // drag sprite to Visible.
    app.insert_resource(HandCardCatalog {
        cards: test_catalog([CardId(50), CardId(51)]),
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();

    // Simulate two acquisitions by populating the hand directly, then enter PLACEMENT.
    app.world_mut().resource_mut::<HandContents>().cards = vec![CardId(50), CardId(51)];
    set_phase(&mut app, RoundPhase::Placement);
    app.update();
    app
}

fn test_catalog<const N: usize>(ids: [CardId; N]) -> HashMap<CardId, CardData> {
    ids.into_iter().map(|id| (id, test_card(id))).collect()
}

fn test_card(id: CardId) -> CardData {
    CardData {
        id,
        name_fr: format!("Carte {}", id.0),
        name_en: format!("Card {}", id.0),
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
        art_id: format!("test_{}", id.0),
        pool_copies_override: None,
    }
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn drag_sprite_entity(app: &App) -> Entity {
    app.world().resource::<HandUiEntities>().drag_sprite
}

fn active_drag_active(app: &App) -> bool {
    let drag = app.world().resource::<ActivePlacementDrag>();
    drag.card.is_some() && drag.target_kind.is_some()
}

fn spawn_dummy_camera(app: &mut App) -> Entity {
    app.world_mut().spawn(()).id()
}

fn node_position(app: &App, entity: Entity) -> (Val, Val) {
    let node = app
        .world()
        .get::<Node>(entity)
        .expect("entity must carry Node");
    (node.left, node.top)
}

// Confirm at compile time that HandDragSprite is in scope for use sites.
#[allow(dead_code)]
fn assert_hand_drag_sprite_imported() -> std::marker::PhantomData<HandDragSprite> {
    std::marker::PhantomData
}

// ── bevy_picking event factories ─────────────────────────────────────────────

fn pointer_location(position: Vec2) -> Location {
    Location {
        target: NormalizedRenderTarget::None {
            width: VIEWPORT_W as u32,
            height: VIEWPORT_H as u32,
        },
        position,
    }
}

fn hit_data(camera: Entity) -> HitData {
    HitData::new(camera, 0.0, None, None)
}

fn pointer_press(target: Entity, camera: Entity, position: Vec2) -> Pointer<Press> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(position),
        Press {
            button: PointerButton::Primary,
            hit: hit_data(camera),
        },
        target,
    )
}

fn pointer_move(target: Entity, camera: Entity, position: Vec2) -> Pointer<Move> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(position),
        Move {
            hit: hit_data(camera),
            delta: Vec2::ZERO,
        },
        target,
    )
}

fn pointer_release(target: Entity, camera: Entity, position: Vec2) -> Pointer<Release> {
    Pointer::new(
        PointerId::Mouse,
        pointer_location(position),
        Release {
            button: PointerButton::Primary,
            hit: hit_data(camera),
        },
        target,
    )
}

// ── Message-cursor helpers ────────────────────────────────────────────────────

fn drained_cursor<M: Message + Clone>(app: &App) -> MessageCursor<M> {
    let messages = app.world().resource::<Messages<M>>();
    let mut cursor = messages.get_cursor();
    let _ = cursor.read(messages).count();
    cursor
}

fn messages_since<M: Message + Clone>(app: &App, cursor: &mut MessageCursor<M>) -> Vec<M> {
    let messages = app.world().resource::<Messages<M>>();
    cursor.read(messages).cloned().collect()
}
