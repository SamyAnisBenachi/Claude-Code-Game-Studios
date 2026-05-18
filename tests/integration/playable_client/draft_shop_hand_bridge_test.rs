use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::{prelude::*, time::Virtual};
use bevy_tweening::TweeningPlugin;
use client::asset_wiring::{
    default_client_card_catalog, enter_in_session_via_fixture, resolve_card_display_art,
};
use client::presentation::{
    card_acquired_fanout_messages, draft_offering_fanout_messages, shop_slots_message,
    PlayerEconomyView, PresentationGameSnapshotMessage,
};
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{GridSlotCard, HandCardCatalog, HandContents, HandUiEntities, HandUiPlugin};
use client::ui::shop_auction::{
    DraftInitialSlotCard, DraftInitialSlotState, ShopAuctionCardCatalog, ShopAuctionDraftHandView,
    ShopAuctionShopSlotClicked, ShopAuctionUiEntities, ShopAuctionUiPlugin, ShopSlotCard,
    ShopSlotState,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::{
    BoardSnapshot, CardSource, PlacementTimerMultiplier, PlayerSnapshot, RoundPhase,
    S2CCardAcquired, S2CDraftOffering, S2CGameSnapshot, S2CGoldUpdate, S2CShopSlots,
};
use shared::session::PlayerId;

#[path = "../../test_helpers.rs"]
mod test_helpers;

fn app_in_phase(phase: RoundPhase, gold: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(HandCardCatalog {
        cards: test_catalog(1..=24),
    });
    app.insert_resource(ShopAuctionCardCatalog {
        cards: test_catalog(1..=24),
    });
    app.insert_resource(PlayerEconomyView {
        gold,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    enter_in_session_via_fixture(&mut app);
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    run_update(&mut app);
    app
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

#[test]
fn test_one_draft_offering_fanout_updates_hand_grid_and_shop_grid() {
    test_helpers::init_test_tracing();
    let mut app = app_in_phase(RoundPhase::DraftInitial, 5);
    let card_ids = (1..=9).map(CardId).collect::<Vec<_>>();
    let fanout = draft_offering_fanout_messages(S2CDraftOffering {
        card_ids: card_ids.clone(),
    });

    app.world_mut().write_message(fanout.hand);
    app.world_mut().write_message(fanout.shop);
    run_update(&mut app);

    assert_eq!(hand_grid_cards(&mut app), card_ids);
    assert_eq!(shop_draft_initial_cards(&mut app), card_ids);
}

#[test]
fn test_one_card_acquired_fanout_updates_hand_and_draft_pending_purchase() {
    test_helpers::init_test_tracing();
    let mut app = app_in_phase(RoundPhase::DraftInitial, 5);
    let card_ids = (1..=9).map(CardId).collect::<Vec<_>>();
    let offering = draft_offering_fanout_messages(S2CDraftOffering {
        card_ids: card_ids.clone(),
    });
    app.world_mut().write_message(offering.hand);
    app.world_mut().write_message(offering.shop);
    run_update(&mut app);

    let fanout = card_acquired_fanout_messages(S2CCardAcquired {
        card_id: CardId(4),
        source: CardSource::DraftInitial,
    });
    app.world_mut().write_message(fanout.hand);
    app.world_mut().write_message(
        fanout
            .draft_initial
            .expect("draft acquisition fans to shop UI"),
    );
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<HandContents>().cards,
        vec![CardId(4)]
    );
    assert_eq!(
        app.world().resource::<ShopAuctionDraftHandView>().hand_size,
        1
    );
    assert_eq!(
        shop_draft_slot_state(&mut app, CardId(4)),
        Some(DraftInitialSlotState::Purchased)
    );
}

#[test]
fn test_shop_purchase_reconciles_hand_size_slots_and_shared_economy() {
    test_helpers::init_test_tracing();
    let mut app = app_in_phase(RoundPhase::DraftShop, 5);
    app.world_mut()
        .write_message(shop_slots_message(S2CShopSlots {
            slots: vec![Some(CardId(1)), Some(CardId(2)), Some(CardId(3))],
        }));
    run_update(&mut app);

    let shop_slot = shop_slot_entity(&mut app, CardId(1));
    app.world_mut()
        .write_message(ShopAuctionShopSlotClicked { slot: shop_slot });
    run_update(&mut app);
    assert_eq!(
        app.world().get::<ShopSlotState>(shop_slot),
        Some(&ShopSlotState::PendingPurchase)
    );

    let fanout = card_acquired_fanout_messages(S2CCardAcquired {
        card_id: CardId(1),
        source: CardSource::ShopPurchase,
    });
    app.world_mut().write_message(fanout.hand);
    app.world_mut().write_message(
        fanout
            .shop_purchase
            .expect("shop acquisition fans to shop UI"),
    );
    app.world_mut()
        .write_message(shop_slots_message(S2CShopSlots {
            slots: vec![None, Some(CardId(2)), Some(CardId(3))],
        }));
    app.world_mut()
        .resource_mut::<PlayerEconomyView>()
        .apply_gold_update(&S2CGoldUpdate {
            gold: 3,
            current_mana: 0,
            reserve_mana: 0,
            mana_cap: 10,
        });
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<HandContents>().cards,
        vec![CardId(1)]
    );
    assert_eq!(
        app.world().resource::<ShopAuctionDraftHandView>().hand_size,
        1
    );
    assert_eq!(app.world().resource::<PlayerEconomyView>().gold, 3);
    assert_eq!(
        app.world().get::<ShopSlotState>(shop_slot),
        Some(&ShopSlotState::Empty)
    );
    assert!(app.world().get::<ShopSlotCard>(shop_slot).is_none());
}

#[test]
fn test_draft_shop_snapshot_seeds_hand_and_shop_before_live_messages() {
    test_helpers::init_test_tracing();
    let mut app = app_in_phase(RoundPhase::DraftShop, 5);

    app.world_mut()
        .write_message(PresentationGameSnapshotMessage(snapshot(
            vec![CardId(7), CardId(8)],
            vec![Some(CardId(2)), None, Some(CardId(3))],
        )));
    run_update(&mut app);

    assert_eq!(
        app.world().resource::<HandContents>().cards,
        vec![CardId(7), CardId(8)]
    );
    assert_eq!(
        app.world().resource::<ShopAuctionDraftHandView>().hand_size,
        2
    );
    assert_eq!(
        shop_slot_cards(&mut app),
        vec![Some(CardId(2)), None, Some(CardId(3))]
    );
}

#[test]
fn test_cards_json_art_ids_resolve_to_display_assets() {
    test_helpers::init_test_tracing();
    let catalog = default_client_card_catalog();
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("client crate should live under the repository root");
    let mut unresolved = Vec::new();

    for card in catalog.values() {
        let expected_path = format!("art/cards/display/card_{}_art_display.png", card.art_id);

        match resolve_card_display_art(Some(card)) {
            Ok(path) if path == expected_path => {
                let asset_path = repo_root.join("assets").join(&path);
                if !asset_path.is_file() {
                    unresolved.push(format!("{} -> {path} missing file", card.art_id));
                }
            }
            Ok(path) => unresolved.push(format!("{} resolved to unexpected {path}", card.art_id)),
            Err(reason) => unresolved.push(format!("{} unresolved: {reason:?}", card.art_id)),
        }
    }

    unresolved.sort();
    assert!(
        unresolved.is_empty(),
        "all assets/data/cards.json art_ids should resolve through display asset wiring:\n{}",
        unresolved.join("\n")
    );
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
        cost: 2,
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

fn snapshot(hand: Vec<CardId>, shop_slots: Vec<Option<CardId>>) -> S2CGameSnapshot {
    S2CGameSnapshot {
        protocol_version: 1,
        recipient_player_id: PlayerId(1),
        round_number: 2,
        phase: RoundPhase::DraftShop,
        timer_remaining_ms: None,
        placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        players: vec![
            player_snapshot(PlayerId(1), hand, shop_slots),
            player_snapshot(PlayerId(2), Vec::new(), Vec::new()),
        ],
        board: BoardSnapshot::default(),
        auction_state: None,
        active_sang_meprise_reveals: None,
    }
}

fn player_snapshot(
    player_id: PlayerId,
    hand: Vec<CardId>,
    shop_slots: Vec<Option<CardId>>,
) -> PlayerSnapshot {
    PlayerSnapshot {
        player_id,
        class_id: ClassId::Iop,
        gold: 5,
        reserved_gold: 0,
        current_mana: 1,
        reserve_mana: 0,
        spawn_range_cells: 1,
        mana_cap: 10,
        submitted: false,
        hand,
        shop_slots,
        pool_snapshot: Vec::new(),
        objectives: Vec::new(),
        opponent_objectives: Vec::new(),
    }
}

fn hand_grid_cards(app: &mut App) -> Vec<CardId> {
    let entities = *app.world().resource::<HandUiEntities>();
    entities
        .grid_slots
        .iter()
        .filter_map(|entity| app.world().get::<GridSlotCard>(*entity).map(|card| card.0))
        .collect()
}

fn shop_draft_initial_cards(app: &mut App) -> Vec<CardId> {
    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    entities
        .draft_initial_slots
        .iter()
        .filter_map(|entity| {
            app.world()
                .get::<DraftInitialSlotCard>(*entity)
                .map(|card| card.0)
        })
        .collect()
}

fn shop_draft_slot_state(app: &mut App, card_id: CardId) -> Option<DraftInitialSlotState> {
    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    entities.draft_initial_slots.iter().find_map(|entity| {
        let card = app.world().get::<DraftInitialSlotCard>(*entity)?;
        (card.0 == card_id).then(|| {
            *app.world()
                .get::<DraftInitialSlotState>(*entity)
                .expect("draft slot should have state")
        })
    })
}

fn shop_slot_cards(app: &mut App) -> Vec<Option<CardId>> {
    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    entities
        .shop_slots
        .iter()
        .map(|entity| app.world().get::<ShopSlotCard>(*entity).map(|slot| slot.0))
        .collect()
}

fn shop_slot_entity(app: &mut App, card_id: CardId) -> Entity {
    let entities = *app.world().resource::<ShopAuctionUiEntities>();
    entities
        .shop_slots
        .iter()
        .copied()
        .find(|entity| {
            app.world()
                .get::<ShopSlotCard>(*entity)
                .is_some_and(|slot| slot.0 == card_id)
        })
        .expect("shop card should be rendered in a slot")
}
