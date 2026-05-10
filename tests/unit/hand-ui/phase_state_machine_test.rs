use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::{math::curve::EaseFunction, prelude::*};
use bevy_tweening::{lens::TransformScaleLens, Tween, TweenAnim};
use client::{
    card_animations::make_tween_anim,
    state::{ClientState, CurrentClientPhase},
    ui::hand::{
        FanSlotIndex, HandContents, HandFanCardClicked, HandFanRoot, HandSlotCard,
        HandSubmitButton, HandTimer, HandUiEntities, HandUiEntity, HandUiMode,
        HandUiOutboundMessages, HandUiPlugin,
    },
};
use shared::{card::CardId, protocol::RoundPhase};

#[test]
fn hu_04_resolution_hides_hand_controls_and_clears_tween_animators() {
    let mut app = app_with_hand_ui_in_session();
    set_hand(&mut app, [CardId(101), CardId(102)]);
    set_phase(&mut app, RoundPhase::Placement);
    app.update();

    let entities = hand_entities(&app);
    insert_active_tween(&mut app, entities.fan_slots[0]);
    insert_active_tween(&mut app, entities.drag_sprite);
    set_visibility(&mut app, entities.drag_sprite, Visibility::Visible);

    set_phase(&mut app, RoundPhase::Resolution);
    app.update();

    assert_eq!(
        app.world().resource::<HandUiMode>(),
        &HandUiMode::Hidden,
        "RESOLUTION should map to HIDDEN mode"
    );
    assert_visibility::<HandFanRoot>(&mut app, Visibility::Hidden);
    assert_visibility::<HandSubmitButton>(&mut app, Visibility::Hidden);
    assert_visibility::<HandTimer>(&mut app, Visibility::Hidden);
    assert_eq!(
        app.world().get::<Visibility>(entities.drag_sprite),
        Some(&Visibility::Hidden),
        "active drag sprite should be hidden on RESOLUTION entry"
    );
    assert_eq!(
        hand_ui_tween_animator_count(&mut app),
        0,
        "Hand UI phase exit should leave no TweenAnim components on Hand UI entities"
    );
}

#[test]
fn hu_05_draft_shop_restores_visible_fan_slots_from_current_hand() {
    let mut app = app_with_hand_ui_in_session();
    let expected = [CardId(201), CardId(202)];

    set_hand(&mut app, expected);
    set_phase(&mut app, RoundPhase::Resolution);
    app.update();
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    assert_eq!(app.world().resource::<HandUiMode>(), &HandUiMode::Passive);
    assert_visibility::<HandFanRoot>(&mut app, Visibility::Visible);

    let entities = hand_entities(&app);
    for (index, card_id) in expected.into_iter().enumerate() {
        let slot = entities.fan_slots[index];
        assert_eq!(
            app.world().get::<Visibility>(slot),
            Some(&Visibility::Visible)
        );
        assert_eq!(
            app.world().get::<HandSlotCard>(slot),
            Some(&HandSlotCard(card_id))
        );
    }

    for slot in entities.fan_slots.iter().copied().skip(expected.len()) {
        assert_eq!(
            app.world().get::<Visibility>(slot),
            Some(&Visibility::Hidden)
        );
        assert!(app.world().get::<HandSlotCard>(slot).is_none());
    }
}

#[test]
fn hu_06_draft_auction_clicks_are_absorbed_but_draft_shop_can_activate() {
    let mut app = app_with_hand_ui_in_session();
    let card_id = CardId(301);
    set_hand(&mut app, [card_id]);
    set_phase(&mut app, RoundPhase::DraftAuction);
    app.update();

    let slot = fan_slot(&mut app, 0);
    for _ in 0..5 {
        app.world_mut()
            .write_message(HandFanCardClicked { card: slot });
    }
    app.update();

    assert_eq!(
        app.world()
            .resource::<HandUiOutboundMessages>()
            .activate_cards
            .len(),
        0,
        "PASSIVE_LOCKED must suppress all activation sends"
    );

    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();
    app.world_mut()
        .write_message(HandFanCardClicked { card: slot });
    app.update();

    let outbound = &app
        .world()
        .resource::<HandUiOutboundMessages>()
        .activate_cards;
    assert_eq!(outbound.len(), 1);
    assert_eq!(outbound[0].card_id, card_id);
}

// Regression marker for PROMPT 562 drain-gap repair (HAND-UI-003 deviation).
// Before the repair, `handle_hand_fan_card_click_system` only pushed to the
// local `HandUiOutboundMessages.activate_cards` buffer with no
// `MessageSender<C2SActivateCard>::send::<ReliableChannel>` dispatch — mirroring
// the same drain-gap pattern PROMPT 558 (commit 27413fb) fixed for
// `C2SPurchaseCard`. After the repair, the system must mirror that canonical
// pattern: dispatch via the sender, then push to the buffer. Each click in
// `HandUiMode::PassiveActivation` (DRAFT_SHOP) must produce exactly one buffer
// entry, proving the dispatch path executes without short-circuiting.
#[test]
fn hu_06_draft_shop_each_fan_click_dispatches_one_activate_message() {
    let mut app = app_with_hand_ui_in_session();
    let card_id = CardId(401);
    set_hand(&mut app, [card_id]);
    set_phase(&mut app, RoundPhase::DraftShop);
    app.update();

    let slot = fan_slot(&mut app, 0);
    for _ in 0..3 {
        app.world_mut()
            .write_message(HandFanCardClicked { card: slot });
        app.update();
    }

    let outbound = &app
        .world()
        .resource::<HandUiOutboundMessages>()
        .activate_cards;
    assert_eq!(
        outbound.len(),
        3,
        "each DRAFT_SHOP fan click must run the sender dispatch + buffer push",
    );
    for message in outbound.iter() {
        assert_eq!(message.card_id, card_id);
    }
}

fn app_with_hand_ui_in_session() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(HandUiPlugin);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    app
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

fn set_hand<const N: usize>(app: &mut App, cards: [CardId; N]) {
    app.world_mut().resource_mut::<HandContents>().cards = cards.to_vec();
}

fn hand_entities(app: &App) -> HandUiEntities {
    *app.world().resource::<HandUiEntities>()
}

fn fan_slot(app: &mut App, index: u8) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    query
        .iter(app.world())
        .find_map(|(entity, slot_index)| (slot_index.0 == index).then_some(entity))
        .expect("fan slot should exist")
}

fn insert_active_tween(app: &mut App, entity: Entity) {
    app.world_mut()
        .entity_mut(entity)
        .insert(make_tween_anim(Tween::new(
            EaseFunction::Linear,
            Duration::from_millis(100),
            TransformScaleLens {
                start: Vec3::ONE,
                end: Vec3::splat(1.25),
            },
        )));
}

fn set_visibility(app: &mut App, entity: Entity, visibility: Visibility) {
    *app.world_mut()
        .get_mut::<Visibility>(entity)
        .expect("Hand UI entity should have Visibility") = visibility;
}

fn assert_visibility<T: Component>(app: &mut App, expected: Visibility) {
    let mut query = app.world_mut().query_filtered::<&Visibility, With<T>>();
    assert!(
        query
            .iter(app.world())
            .all(|visibility| *visibility == expected),
        "all matching entities should have {expected:?}"
    );
}

fn hand_ui_tween_animator_count(app: &mut App) -> usize {
    let mut query = app
        .world_mut()
        .query_filtered::<&TweenAnim, With<HandUiEntity>>();
    query.iter(app.world()).count()
}
