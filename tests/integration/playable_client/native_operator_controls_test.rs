use std::collections::HashMap;
use std::time::Duration;

use bevy::ecs::message::MessageCursor;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientPhaseView, ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotState, GhostClickedEvent, HandCardCatalog, HandContents, HandSubmitButton,
    HandUiEntities, HandUiOutboundMessages, HandUiPlugin, PendingPlacements,
};
use client::ui::lobby::{
    LobbyClassButton, LobbyCommand, LobbyConfirmClassButton, LobbyCreateRoomButton,
    LobbyInputState, LobbyJoinRoomButton, LobbyRequestedSlotButton, LobbyRoomCodeField,
    LobbyUiPlugin,
};
use client::ui::shop_auction::{
    AuctionBidButtonState, DraftInitialSlotState, ShopAuctionAuctionCardReceived,
    ShopAuctionCardCatalog, ShopAuctionDraftHandView, ShopAuctionLocalGoldView,
    ShopAuctionShopSlotsReceived, ShopAuctionUiEntities, ShopAuctionUiOutboundMessages,
    ShopAuctionUiPlugin, ShopSlotState,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{PlayTarget, RoundPhase};
use shared::session::PlayerId;

const LOCAL_PLAYER: PlayerId = PlayerId(1);

#[test]
fn test_lobby_room_code_focus_separates_text_from_shortcuts() {
    let mut app = lobby_app();
    let mut command_cursor = command_cursor(&app);

    app.world_mut()
        .resource_mut::<LobbyInputState>()
        .room_code_focused = true;
    press_key(&mut app, KeyCode::Digit2);
    press_key(&mut app, KeyCode::KeyJ);

    let input = app.world().resource::<LobbyInputState>();
    assert_eq!(input.join_room_code, "2J");
    assert_eq!(input.requested_slot, 1);
    assert!(commands_since(&app, &mut command_cursor).is_empty());

    {
        let mut input = app.world_mut().resource_mut::<LobbyInputState>();
        input.room_code_focused = false;
        input.join_room_code = "AB12".to_string();
    }
    press_key(&mut app, KeyCode::Digit3);
    press_key(&mut app, KeyCode::KeyJ);

    let input = app.world().resource::<LobbyInputState>();
    assert_eq!(input.join_room_code, "AB12");
    assert_eq!(input.requested_slot, 3);
    assert_eq!(
        commands_since(&app, &mut command_cursor),
        vec![LobbyCommand::JoinRoom {
            room_code: "AB12".to_string(),
            requested_slot: 3,
        }]
    );
}

#[test]
fn test_lobby_room_code_textbox_click_selects_and_accepts_text_input() {
    let mut app = lobby_app();
    let mut command_cursor = command_cursor(&app);

    app.world_mut()
        .resource_mut::<LobbyInputState>()
        .join_room_code = "OLD123".to_string();
    let room_code = entity_with::<LobbyRoomCodeField>(&mut app);
    press_interaction(&mut app, room_code);

    let input = app.world().resource::<LobbyInputState>();
    assert!(input.room_code_focused);
    assert!(input.room_code_selected);

    type_text(&mut app, "ab-12");
    let input = app.world().resource::<LobbyInputState>();
    assert_eq!(input.join_room_code, "AB12");
    assert!(!input.room_code_selected);
    assert!(commands_since(&app, &mut command_cursor).is_empty());

    press_key(&mut app, KeyCode::Enter);
    assert_eq!(
        commands_since(&app, &mut command_cursor),
        vec![LobbyCommand::JoinRoom {
            room_code: "AB12".to_string(),
            requested_slot: 1,
        }]
    );
}

#[test]
fn test_lobby_buttons_drive_create_join_slot_class_and_confirm_commands() {
    let mut app = lobby_app();
    let mut command_cursor = command_cursor(&app);

    let room_code = entity_with::<LobbyRoomCodeField>(&mut app);
    press_interaction(&mut app, room_code);
    assert!(app.world().resource::<LobbyInputState>().room_code_focused);

    let slot = slot_button(&mut app, 2);
    press_interaction(&mut app, slot);
    assert_eq!(app.world().resource::<LobbyInputState>().requested_slot, 2);
    assert_eq!(app.world().resource::<LobbyInputState>().join_room_code, "");

    let create = entity_with::<LobbyCreateRoomButton>(&mut app);
    press_interaction(&mut app, create);
    press_interaction(&mut app, create);
    assert_eq!(
        commands_since(&app, &mut command_cursor),
        vec![LobbyCommand::CreateRoom]
    );

    app.world_mut()
        .resource_mut::<LobbyInputState>()
        .join_room_code = "xy9".to_string();
    let join = entity_with::<LobbyJoinRoomButton>(&mut app);
    press_interaction(&mut app, join);
    assert_eq!(
        commands_since(&app, &mut command_cursor),
        vec![LobbyCommand::JoinRoom {
            room_code: "XY9".to_string(),
            requested_slot: 2,
        }]
    );

    let class = class_button(&mut app, ClassId::Xelor);
    let confirm = entity_with::<LobbyConfirmClassButton>(&mut app);
    press_interaction(&mut app, class);
    press_interaction(&mut app, confirm);
    assert_eq!(
        commands_since(&app, &mut command_cursor),
        vec![
            LobbyCommand::SelectClass {
                class_id: ClassId::Xelor,
            },
            LobbyCommand::ConfirmClass {
                class_id: ClassId::Xelor,
            },
        ]
    );
}

#[test]
fn test_shop_auction_pointer_controls_emit_operator_intents() {
    let mut app = shop_app();
    set_phase(&mut app, RoundPhase::DraftInitial, 45_000);
    send_draft_offering(&mut app, (1..=9).map(CardId).collect());

    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    press_interaction(&mut app, entities.draft_initial_slots[0]);
    press_interaction(&mut app, entities.draft_initial_ready_button);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.purchase_cards.len(), 1);
    assert_eq!(outbound.purchase_cards[0].card_id, CardId(1));
    assert_eq!(outbound.ready_signals.len(), 1);
    assert!(!outbound.ready_signals[0].retract);
    assert_eq!(
        app.world()
            .get::<DraftInitialSlotState>(entities.draft_initial_slots[0]),
        Some(&DraftInitialSlotState::Pending)
    );

    set_phase(&mut app, RoundPhase::DraftShop, 30_000);
    send_shop_slots(
        &mut app,
        vec![Some(CardId(2)), Some(CardId(3)), Some(CardId(4))],
    );
    press_interaction(&mut app, entities.shop_slots[0]);
    press_interaction(&mut app, entities.shop_refresh_button);
    press_interaction(&mut app, entities.shop_ready_button);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.purchase_cards.len(), 2);
    assert_eq!(outbound.purchase_cards[1].card_id, CardId(2));
    assert_eq!(outbound.refresh_shops.len(), 1);
    assert_eq!(outbound.ready_signals.len(), 2);
    assert_eq!(
        app.world().get::<ShopSlotState>(entities.shop_slots[0]),
        Some(&ShopSlotState::Refreshing)
    );

    set_phase(&mut app, RoundPhase::DraftAuction, 20_000);
    send_auction_card(&mut app, CardId(5), 4);
    press_interaction(&mut app, entities.auction_bid_buttons[1]);

    let outbound = app.world().resource::<ShopAuctionUiOutboundMessages>();
    assert_eq!(outbound.place_bids.len(), 1);
    assert_eq!(outbound.place_bids[0].amount, 7);
    assert_eq!(
        app.world()
            .get::<AuctionBidButtonState>(entities.auction_bid_buttons[1]),
        Some(&AuctionBidButtonState::InFlight)
    );
}

#[test]
fn test_hand_pointer_controls_stage_unstage_and_submit_placement() {
    let mut app = hand_app();
    set_hand(&mut app, vec![CardId(1)]);
    set_phase(&mut app, RoundPhase::Placement, 30_000);

    let hand_entities = *app.world().resource::<HandUiEntities>();
    press_interaction(&mut app, hand_entities.fan_slots[0]);

    assert_eq!(
        app.world().get::<FanSlotState>(hand_entities.fan_slots[0]),
        Some(&FanSlotState::Ghost)
    );
    assert_eq!(
        app.world().resource::<PendingPlacements>().placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 1 }
    );

    app.world_mut()
        .write_message(GhostClickedEvent { card_id: CardId(1) });
    run_update(&mut app);
    assert!(app
        .world()
        .resource::<PendingPlacements>()
        .placements
        .is_empty());
    assert_eq!(
        app.world().get::<FanSlotState>(hand_entities.fan_slots[0]),
        Some(&FanSlotState::Active)
    );

    press_interaction(&mut app, hand_entities.fan_slots[0]);
    let submit = entity_with::<HandSubmitButton>(&mut app);
    press_interaction(&mut app, submit);

    let outbound = app.world().resource::<HandUiOutboundMessages>();
    assert_eq!(outbound.submit_placements.len(), 1);
    assert_eq!(outbound.submit_placements[0].placements.len(), 1);
    assert_eq!(
        outbound.submit_placements[0].placements[0].target,
        PlayTarget::BoardCell { lane: 1, cell: 1 }
    );
}

fn lobby_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_plugins(LobbyUiPlugin);
    run_update(&mut app);
    app
}

fn shop_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: test_catalog(1..=12),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 20,
        initialized: true,
        ..default()
    });
    app.insert_resource(ShopAuctionLocalGoldView {
        player_id: Some(LOCAL_PLAYER),
        gold: 20,
        reserved_gold: 0,
        initialized: true,
    });
    app.insert_resource(ShopAuctionDraftHandView { hand_size: 0 });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn hand_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(HandUiPlugin);
    app.insert_resource(HandCardCatalog {
        cards: test_catalog(1..=4),
    });
    app.insert_resource(PlayerEconomyView {
        current_mana: 5,
        reserve_mana: 0,
        mana_cap: 10,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app
}

fn set_phase(app: &mut App, phase: RoundPhase, timer_duration_ms: u32) {
    let round = app.world().resource::<CurrentClientPhase>().round + 1;
    {
        let mut current = app.world_mut().resource_mut::<CurrentClientPhase>();
        current.phase = phase;
        current.round = round;
    }
    {
        let mut phase_view = app.world_mut().resource_mut::<ClientPhaseView>();
        phase_view.phase = phase;
        phase_view.round_number = round;
        phase_view.timer_duration_ms = timer_duration_ms;
    }
    run_update(app);
}

fn set_hand(app: &mut App, cards: Vec<CardId>) {
    app.world_mut().resource_mut::<HandContents>().cards = cards;
}

fn send_draft_offering(app: &mut App, card_ids: Vec<CardId>) {
    app.world_mut()
        .write_message(client::ui::shop_auction::ShopAuctionDraftOfferingReceived { card_ids });
    run_update(app);
}

fn send_shop_slots(app: &mut App, slots: Vec<Option<CardId>>) {
    app.world_mut()
        .write_message(ShopAuctionShopSlotsReceived { slots });
    run_update(app);
}

fn send_auction_card(app: &mut App, card_id: CardId, starting_price: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id,
            starting_price,
        });
    run_update(app);
}

fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    run_update(app);
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(key);
        keys.clear();
    }
}

fn type_text(app: &mut App, text: &str) {
    let window = app.world_mut().spawn_empty().id();
    for character in text.chars() {
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyA,
            logical_key: Key::Character(character.to_string().into()),
            state: ButtonState::Pressed,
            text: Some(character.to_string().into()),
            repeat: false,
            window,
        });
    }
    run_update(app);
}

fn press_interaction(app: &mut App, entity: Entity) {
    *app.world_mut()
        .get_mut::<Interaction>(entity)
        .expect("operator control should expose Interaction") = Interaction::Pressed;
    run_update(app);
    *app.world_mut()
        .get_mut::<Interaction>(entity)
        .expect("operator control should expose Interaction") = Interaction::None;
    run_update(app);
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn command_cursor(app: &App) -> MessageCursor<LobbyCommand> {
    app.world()
        .resource::<Messages<LobbyCommand>>()
        .get_cursor()
}

fn commands_since(app: &App, cursor: &mut MessageCursor<LobbyCommand>) -> Vec<LobbyCommand> {
    let messages = app.world().resource::<Messages<LobbyCommand>>();
    cursor.read(messages).cloned().collect()
}

fn entity_with<T: Component>(app: &mut App) -> Entity {
    let mut query = app.world_mut().query_filtered::<Entity, With<T>>();
    query.single(app.world()).expect("entity should exist")
}

fn slot_button(app: &mut App, slot: u8) -> Entity {
    let mut query = app
        .world_mut()
        .query::<(Entity, &LobbyRequestedSlotButton)>();
    query
        .iter(app.world())
        .find_map(|(entity, button)| (button.slot == slot).then_some(entity))
        .expect("slot button should exist")
}

fn class_button(app: &mut App, class_id: ClassId) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &LobbyClassButton)>();
    query
        .iter(app.world())
        .find_map(|(entity, button)| (button.class_id == class_id).then_some(entity))
        .expect("class button should exist")
}

fn test_catalog(ids: impl IntoIterator<Item = u32>) -> HashMap<CardId, CardData> {
    ids.into_iter()
        .map(|id| {
            let card = CardData {
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
            };
            (card.id, card)
        })
        .collect()
}
