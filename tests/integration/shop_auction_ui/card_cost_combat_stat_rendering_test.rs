//! PROMPT 1029 — card cost / combat-stat rendering repair.
//!
//! Closes the QA observation that hand / draft / shop / auction surfaces
//! render diamond-shaped stat markers without numeric values. The fixes
//! land in three places inside `client::ui::shop_auction`:
//!
//!   * Draft Initial keep-9 slot label appends ATK/HP for minion-shaped cards.
//!   * Shop slot label appends ATK/HP for minion-shaped cards.
//!   * Auction featured-card `AuctionFeaturedCardStats` text node is bound to
//!     `ATK/HP · Cost Ng` (was empty since story 016 reserved the typography
//!     slot without wiring content).
//!
//! These integration tests drive the public message API and inspect the
//! resulting `Text` nodes via the stable marker components, mirroring the
//! pattern in `auction_featured_card_layout_test.rs`.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::shop_auction::{
    format_card_combat_stats, AuctionFeaturedCardStats, DraftInitialSlotIndex,
    DraftInitialSlotText, DraftInitialSlotTextLabel, ShopAuctionAuctionCardReceived,
    ShopAuctionCardCatalog, ShopAuctionDraftOfferingReceived, ShopAuctionShopSlotsReceived,
    ShopAuctionUiEntities, ShopAuctionUiPlugin, ShopSlotIndex,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const MINION_CARD_ID: u32 = 101;
const SPELL_CARD_ID: u32 = 202;

/// PROMPT 1029 — `format_card_combat_stats` returns `"ATK/HP"` for minion-like
/// cards (Minion, Structure) and an empty string otherwise. This is the
/// shared formatter consumed by every draft / shop / auction surface; locking
/// its contract here keeps the three call sites consistent without each test
/// duplicating the format-string assertion.
#[test]
fn format_card_combat_stats_minion_returns_atk_slash_hp() {
    let minion = test_card(MINION_CARD_ID, CardType::Minion, 3, 4, 5);
    assert_eq!(format_card_combat_stats(&minion), "4/5");
}

#[test]
fn format_card_combat_stats_structure_returns_atk_slash_hp() {
    let structure = test_card(MINION_CARD_ID, CardType::Structure, 3, 0, 8);
    assert_eq!(format_card_combat_stats(&structure), "0/8");
}

#[test]
fn format_card_combat_stats_spell_returns_empty() {
    let spell = test_card(SPELL_CARD_ID, CardType::Spell, 2, 0, 0);
    assert!(format_card_combat_stats(&spell).is_empty());
}

#[test]
fn format_card_combat_stats_trap_returns_empty() {
    let trap = test_card(SPELL_CARD_ID, CardType::Trap, 1, 0, 0);
    assert!(format_card_combat_stats(&trap).is_empty());
}

/// PROMPT 1029 — DraftInitial slot label for a Minion card includes
/// `Ng · ATK/HP`. Prior to this prompt the label was just `name\nNg`.
#[test]
fn draft_initial_slot_label_renders_atk_hp_for_minion() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(RoundPhase::DraftInitial);
    deliver_draft_offering(&mut app, MINION_CARD_ID);
    run_update(&mut app);

    let slot_text_entity = first_draft_initial_slot_text_entity(&mut app);
    let text = app
        .world()
        .get::<Text>(slot_text_entity)
        .expect("draft slot text node must exist")
        .0
        .clone();

    assert!(
        text.contains("Goblin"),
        "draft slot label must contain card name; got {text:?}",
    );
    assert!(
        text.contains("3g"),
        "draft slot label must include mana cost as `{{cost}}g`; got {text:?}",
    );
    assert!(
        text.contains("4/5"),
        "draft slot label must include `ATK/HP` (4/5) so players can read combat \
         stats from the keep-9 grid; got {text:?}",
    );
}

/// PROMPT 1029 — DraftInitial slot label for a Spell card does NOT add an
/// ATK/HP suffix (preserves prior `name\n{cost}g` shape for non-minion cards).
#[test]
fn draft_initial_slot_label_omits_atk_hp_for_spell() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(RoundPhase::DraftInitial);
    deliver_draft_offering(&mut app, SPELL_CARD_ID);
    run_update(&mut app);

    let slot_text_entity = first_draft_initial_slot_text_entity(&mut app);
    let text = app
        .world()
        .get::<Text>(slot_text_entity)
        .expect("draft slot text node must exist")
        .0
        .clone();

    assert!(
        !text.contains('/'),
        "spell card draft label must not include an ATK/HP `/` separator; got {text:?}",
    );
}

/// PROMPT 1029 — Shop slot label for a Minion card includes
/// `Rarity · Ng · ATK/HP`.
#[test]
fn shop_slot_label_renders_atk_hp_for_minion() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(RoundPhase::DraftShop);
    deliver_shop_slots(&mut app, [Some(CardId(MINION_CARD_ID)), None, None]);
    run_update(&mut app);

    let shop_slot = first_shop_slot_entity(&mut app);
    let text = app
        .world()
        .get::<Text>(shop_slot)
        .expect("shop slot must carry Text")
        .0
        .clone();

    assert!(
        text.contains("Goblin"),
        "shop slot label must contain card name; got {text:?}",
    );
    assert!(
        text.contains("3g"),
        "shop slot label must include mana cost; got {text:?}",
    );
    assert!(
        text.contains("4/5"),
        "shop slot label must include `ATK/HP` (4/5) for minion cards; got {text:?}",
    );
}

/// PROMPT 1029 — auction featured-card stats text node was empty prior to
/// this prompt. After the fix it must carry `ATK/HP {n/n} · Cost {n}g`.
#[test]
fn auction_featured_card_stats_text_renders_atk_hp_and_cost() {
    test_helpers::init_test_tracing();
    let mut app = app_in_session(RoundPhase::DraftAuction);
    deliver_auction_card(&mut app, MINION_CARD_ID);
    // Run twice so the buffered card message processes then the StateSync
    // pass writes the resulting text.
    run_update(&mut app);
    run_update(&mut app);

    let entities = app.world().resource::<ShopAuctionUiEntities>().clone();
    let stats_text = app
        .world()
        .get::<Text>(entities.auction_featured_card_stats)
        .expect("auction featured-card stats node must carry a Text")
        .0
        .clone();

    assert!(
        stats_text.contains("4/5"),
        "auction featured stats text must include ATK/HP (4/5); got {stats_text:?}",
    );
    assert!(
        stats_text.contains("3g"),
        "auction featured stats text must include the mana cost; got {stats_text:?}",
    );

    // Marker uniqueness — exactly one stats entity exists, so future refactors
    // can't accidentally split the rendering across two nodes.
    let mut stats_q = app.world_mut().query::<&AuctionFeaturedCardStats>();
    assert_eq!(
        stats_q.iter(app.world()).count(),
        1,
        "exactly one entity may carry AuctionFeaturedCardStats",
    );
}

// ── Harness helpers ─────────────────────────────────────────────────────────

fn app_in_session(phase: RoundPhase) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::image::Image>();
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.add_plugins(ShopAuctionUiPlugin);
    app.insert_resource(ShopAuctionCardCatalog {
        cards: catalog_with_min_card_set(),
    });
    app.insert_resource(PlayerEconomyView {
        gold: 10,
        initialized: true,
        ..default()
    });
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    run_update(&mut app);
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
    run_update(&mut app);
    app
}

fn catalog_with_min_card_set() -> HashMap<CardId, CardData> {
    let mut cards = HashMap::new();
    let minion = test_card(MINION_CARD_ID, CardType::Minion, 3, 4, 5);
    cards.insert(minion.id, minion);
    let spell = test_card(SPELL_CARD_ID, CardType::Spell, 2, 0, 0);
    cards.insert(spell.id, spell);
    cards
}

fn deliver_draft_offering(app: &mut App, card_id: u32) {
    app.world_mut()
        .write_message(ShopAuctionDraftOfferingReceived {
            card_ids: vec![CardId(card_id)],
        });
}

fn deliver_shop_slots(app: &mut App, slots: [Option<CardId>; 3]) {
    app.world_mut().write_message(ShopAuctionShopSlotsReceived {
        slots: slots.to_vec(),
    });
}

fn deliver_auction_card(app: &mut App, card_id: u32) {
    app.world_mut()
        .write_message(ShopAuctionAuctionCardReceived {
            card_id: CardId(card_id),
            starting_price: 1,
            timer_duration_ms: 5_000,
        });
}

fn first_draft_initial_slot_text_entity(app: &mut App) -> Entity {
    let mut query = app
        .world_mut()
        .query::<(&DraftInitialSlotIndex, &DraftInitialSlotText)>();
    let mut entries: Vec<(u8, Entity)> = query
        .iter(app.world())
        .map(|(idx, txt)| (idx.0, txt.0))
        .collect();
    entries.sort_by_key(|(idx, _)| *idx);
    let text_entity = entries
        .first()
        .map(|(_, text_entity)| *text_entity)
        .expect("at least one DraftInitial slot must spawn");

    // Confirm the text entity carries the DraftInitialSlotTextLabel marker so
    // the label semantics are not silently disconnected from the slot.
    let mut label_q = app.world_mut().query::<&DraftInitialSlotTextLabel>();
    assert!(
        label_q.get(app.world(), text_entity).is_ok(),
        "draft slot text entity must carry the DraftInitialSlotTextLabel marker",
    );
    text_entity
}

fn first_shop_slot_entity(app: &mut App) -> Entity {
    let mut query = app.world_mut().query::<(Entity, &ShopSlotIndex)>();
    let mut entries: Vec<(u8, Entity)> = query
        .iter(app.world())
        .map(|(entity, idx)| (idx.0, entity))
        .collect();
    entries.sort_by_key(|(idx, _)| *idx);
    entries
        .first()
        .map(|(_, entity)| *entity)
        .expect("at least one shop slot must spawn")
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn test_card(id: u32, card_type: CardType, cost: u32, atk: u8, hp: u8) -> CardData {
    CardData {
        id: CardId(id),
        name_fr: "Goblin".to_string(),
        name_en: "Goblin".to_string(),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type,
        unit_type: UnitType::Blade,
        cost,
        atk,
        hp,
        mp: 1,
        ar: 0,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{id}"),
        pool_copies_override: None,
    }
}
