//! PROMPT 1029 — hand fan stat-badge text labels.
//!
//! Before this prompt, the hand fan rendered four diamond-shaped stat-badge
//! images per card slot but never overlaid the numeric atk / hp / mp / ar
//! values. This test asserts that the new label children (one per badge) are
//! spawned and populated by `sync_fan_slot_stat_labels_system`.

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_tweening::TweeningPlugin;
use client::presentation::PlayerEconomyView;
use client::state::{ClientState, CurrentClientPhase};
use client::ui::hand::{
    FanSlotIndex, HandCardCatalog, HandUiCardAcquiredReceived, HandUiPlugin, HandUiTimingConfig,
    StatBadgeAr, StatBadgeArLabel, StatBadgeAtk, StatBadgeAtkLabel, StatBadgeHp, StatBadgeHpLabel,
    StatBadgeMp, StatBadgeMpLabel,
};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::protocol::RoundPhase;

#[path = "../../test_helpers.rs"]
mod test_helpers;

const FIRST_ACQUIRED_CARD_ID: u32 = 7_071;
const ATK_VALUE: u8 = 4;
const HP_VALUE: u8 = 5;
const MP_VALUE: u8 = 2;
const AR_VALUE: u8 = 1;
const VIEWPORT_WIDTH: f32 = 1280.0;
const VIEWPORT_HEIGHT: f32 = 720.0;

#[test]
fn fan_slot_stat_labels_render_card_values_after_acquisition() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui();

    app.world_mut().write_message(HandUiCardAcquiredReceived {
        card_id: CardId(FIRST_ACQUIRED_CARD_ID),
    });
    // PROMPT 1029: the chrome / label sync runs in StateSync once the slot
    // gets a `HandSlotCard`; two updates cover the acquisition tick + the
    // following StateSync tick.
    run_update(&mut app);
    set_phase(&mut app, RoundPhase::Placement);
    for _ in 0..4 {
        run_update(&mut app);
    }

    let atk_label_text = read_label_text::<StatBadgeAtk, StatBadgeAtkLabel>(&mut app);
    let hp_label_text = read_label_text::<StatBadgeHp, StatBadgeHpLabel>(&mut app);
    let mp_label_text = read_label_text::<StatBadgeMp, StatBadgeMpLabel>(&mut app);
    let ar_label_text = read_label_text::<StatBadgeAr, StatBadgeArLabel>(&mut app);

    assert_eq!(
        atk_label_text,
        ATK_VALUE.to_string(),
        "ATK badge label must display the card's atk value, not an empty diamond glyph",
    );
    assert_eq!(
        hp_label_text,
        HP_VALUE.to_string(),
        "HP badge label must display the card's hp value",
    );
    assert_eq!(
        mp_label_text,
        MP_VALUE.to_string(),
        "MP badge label must display the card's mp value",
    );
    assert_eq!(
        ar_label_text,
        AR_VALUE.to_string(),
        "AR badge label must display the card's ar value",
    );
}

/// PROMPT 1029 — empty slot (no acquired card) means every stat badge label
/// must be empty. Defends against a future regression where the label keeps
/// showing the previous occupant's number after the slot empties.
#[test]
fn fan_slot_stat_labels_empty_when_slot_has_no_card() {
    test_helpers::init_test_tracing();
    let mut app = app_with_hand_ui();
    run_update(&mut app);

    let atk_label_text = read_label_text::<StatBadgeAtk, StatBadgeAtkLabel>(&mut app);
    assert!(
        atk_label_text.is_empty(),
        "ATK label for an empty slot must be empty; got {atk_label_text:?}",
    );
}

// ── Harness ────────────────────────────────────────────────────────────────

fn app_with_hand_ui() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<ClientState>();
    app.insert_resource(client::asset_wiring::placeholder_assets_for_tests());
    app.add_plugins(TweeningPlugin);
    app.add_plugins(HandUiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32),
            ..default()
        },
        PrimaryWindow,
    ));
    app.insert_resource(HandCardCatalog {
        cards: test_catalog(),
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
        .resource_mut::<bevy::time::Time<bevy::time::Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::InSession);
    app.update();
    set_phase(&mut app, RoundPhase::DraftInitial);
    app.update();
    app
}

fn run_update(app: &mut App) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::ZERO);
    app.update();
}

fn set_phase(app: &mut App, phase: RoundPhase) {
    app.world_mut().resource_mut::<CurrentClientPhase>().phase = phase;
}

/// Reads the `Text` carried by the *first* slot's label component `L`. The
/// label is a grandchild of the slot (slot → badge → label) so we walk one
/// `ChildOf` hop from the label to find the badge, then a second hop to find
/// the slot, then sort the slot by `FanSlotIndex` to pick slot 0.
fn read_label_text<B: Component, L: Component>(app: &mut App) -> String {
    // Map each badge entity to the index of the slot that owns it.
    let mut slot_idx_query = app.world_mut().query::<(Entity, &FanSlotIndex)>();
    let slot_index: HashMap<Entity, u8> = slot_idx_query
        .iter(app.world())
        .map(|(e, i)| (e, i.0))
        .collect();

    let mut badge_query = app
        .world_mut()
        .query_filtered::<(Entity, &ChildOf), With<B>>();
    let badge_to_slot_idx: HashMap<Entity, u8> = badge_query
        .iter(app.world())
        .filter_map(|(badge, badge_parent)| {
            slot_index
                .get(&badge_parent.parent())
                .copied()
                .map(|idx| (badge, idx))
        })
        .collect();

    let mut label_query = app
        .world_mut()
        .query_filtered::<(&Text, &ChildOf), With<L>>();
    let mut labels: Vec<(u8, String)> = label_query
        .iter(app.world())
        .filter_map(|(text, parent)| {
            badge_to_slot_idx
                .get(&parent.parent())
                .copied()
                .map(|idx| (idx, text.0.clone()))
        })
        .collect();
    labels.sort_by_key(|(idx, _)| *idx);
    labels
        .into_iter()
        .next()
        .map(|(_, text)| text)
        .expect("at least one badge label must spawn per slot")
}

fn test_catalog() -> HashMap<CardId, CardData> {
    let mut cards = HashMap::new();
    let card = CardData {
        id: CardId(FIRST_ACQUIRED_CARD_ID),
        name_fr: "Test".to_string(),
        name_en: "Test".to_string(),
        class: ClassId::Iop,
        family: Some("Test".to_string()),
        rarity: Rarity::Common,
        card_type: CardType::Minion,
        unit_type: UnitType::Blade,
        cost: 3,
        atk: ATK_VALUE,
        hp: HP_VALUE,
        mp: MP_VALUE,
        ar: AR_VALUE,
        keywords: Vec::new(),
        effect_text: String::new(),
        art_id: format!("test_{FIRST_ACQUIRED_CARD_ID}"),
        pool_copies_override: None,
    };
    cards.insert(card.id, card);
    cards
}
