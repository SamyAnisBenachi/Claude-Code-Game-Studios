use std::collections::HashMap;

use bevy::prelude::*;
use server::core::economy::{AwardGold, ManaCapIncreased};
use server::core::objective_contract::ObjectiveCounters;
use server::core::pool::{PlayerPool, PlayerPools};
use server::feature::acquisition::{PlayerHands, MAX_HAND_SIZE};
use server::feature::board::{FakeObjectiveDestroyed, LaneId};
use server::feature::objective::{
    apply_consequence_path, HiddenObjectives, ObjectiveHp, ObjectiveSlot, PendingObjectiveEvents,
    FAKE_REWARD_POOL_FILTER,
};
use server::foundation::config::CardCatalog;
use server::foundation::rng::ServerRng;
use shared::card::{CardData, CardId, CardType, ClassId, Rarity, UnitType};
use shared::config::GameConfig;
use shared::session::PlayerId;

const PLAYER_A: PlayerId = PlayerId(1);
const PLAYER_B: PlayerId = PlayerId(2);
const LANE_1: LaneId = 1;
const LANE_2: LaneId = 2;
const REWARD_CARD_ID: CardId = CardId(9101);

// Precomputed through `ServerRng::from_seed(S)` followed by the D4 reward roll:
// `ServerRng::award_fake_objective_reward` seed -> `ChaCha20Rng::gen_range(0..2)`.
const SEED_PRODUCES_FREE_CARD: u64 = 0;
const SEED_PRODUCES_MANA_CAP: u64 = 1;
const SEED_PRODUCES_TWO_MANA_CAPS: u64 = 15;
const SEED_PRODUCES_MANA_THEN_FREE_CARD: u64 = 1;

fn app_with_fake_objective(seed: u64) -> App {
    let mut app = App::new();
    app.add_message::<AwardGold>();
    app.add_message::<ManaCapIncreased>();
    app.add_message::<FakeObjectiveDestroyed>();
    app.insert_resource(HiddenObjectives {
        identities: HashMap::from([((PLAYER_B, LANE_1), true), ((PLAYER_B, LANE_2), true)]),
    });
    app.insert_resource(ObjectiveCounters::default());
    app.insert_resource(PendingObjectiveEvents::default());
    app.insert_resource(ServerRng::from_seed(seed));
    app.insert_resource(PlayerHands::default());
    app.insert_resource(test_catalog());
    app.insert_resource(PlayerPools {
        pools: HashMap::from([(PLAYER_A, player_pool())]),
    });
    spawn_objective(&mut app, PLAYER_B, LANE_1);
    spawn_objective(&mut app, PLAYER_B, LANE_2);
    app
}

fn spawn_objective(app: &mut App, owner: PlayerId, lane: LaneId) {
    app.world_mut().spawn((
        ObjectiveSlot {
            lane,
            player: owner,
            destroyed: false,
        },
        ObjectiveHp { hp: 1 },
    ));
}

fn test_catalog() -> CardCatalog {
    CardCatalog {
        cards: [(REWARD_CARD_ID, test_card(REWARD_CARD_ID))]
            .into_iter()
            .collect(),
    }
}

fn test_card(id: CardId) -> CardData {
    CardData {
        id,
        name_fr: "Fake Reward".to_string(),
        name_en: "Fake Reward".to_string(),
        class: ClassId::Neutral,
        family: None,
        rarity: Rarity::Common,
        card_type: CardType::Spell,
        unit_type: UnitType::Neutral,
        cost: 1,
        atk: 0,
        hp: 0,
        mp: 0,
        ar: 0,
        keywords: vec![],
        effect_text: String::new(),
        art_id: "fake_reward".to_string(),
        pool_copies_override: Some(4),
    }
}

fn player_pool() -> PlayerPool {
    PlayerPool::initialize(&test_catalog().cards, &GameConfig::default())
}

fn hand(app: &App, player: PlayerId) -> Vec<CardId> {
    app.world()
        .resource::<PlayerHands>()
        .hands
        .get(&player)
        .cloned()
        .unwrap_or_default()
}

fn set_hand_len(app: &mut App, player: PlayerId, len: usize) {
    app.world_mut()
        .resource_mut::<PlayerHands>()
        .hands
        .insert(player, (1..=len as u32).map(CardId).collect());
}

fn read_messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let messages = app.world().resource::<Messages<T>>();
    let mut cursor = messages.get_cursor();
    cursor.read(messages).cloned().collect()
}

fn apply_fake_consequence(app: &mut App, lane: LaneId) {
    apply_consequence_path(app.world_mut(), lane, PLAYER_A, PLAYER_B);
}

fn seed_index(app: &App) -> u32 {
    app.world().resource::<ServerRng>().current_seed_index()
}

fn copies_remaining(app: &App, player: PlayerId, card_id: CardId) -> u32 {
    app.world()
        .resource::<PlayerPools>()
        .pools
        .get(&player)
        .expect("player pool exists")
        .copies_remaining(card_id)
}

#[test]
fn test_os8_fake_destruction_emits_exactly_one_d4_reward_outcome() {
    for seed in [SEED_PRODUCES_MANA_CAP, SEED_PRODUCES_FREE_CARD] {
        let mut app = app_with_fake_objective(seed);

        apply_fake_consequence(&mut app, LANE_1);

        let mana_count = read_messages::<ManaCapIncreased>(&app).len();
        let free_card_drawn = hand(&app, PLAYER_A).len() == 1;
        assert_ne!(
            mana_count == 1,
            free_card_drawn,
            "fake destruction must produce exactly one of ManaCapIncreased or FreeCardPick"
        );
        assert_eq!(
            app.world()
                .resource::<ObjectiveCounters>()
                .fake_objectives_destroyed(PLAYER_A),
            1
        );
    }
}

#[test]
fn test_os11_two_fake_destructions_with_mana_cap_rolls_emit_twice() {
    let mut app = app_with_fake_objective(SEED_PRODUCES_TWO_MANA_CAPS);

    apply_fake_consequence(&mut app, LANE_1);
    apply_fake_consequence(&mut app, LANE_2);

    assert_eq!(
        read_messages::<ManaCapIncreased>(&app),
        vec![
            ManaCapIncreased {
                player: PLAYER_A,
                amount: 1,
            },
            ManaCapIncreased {
                player: PLAYER_A,
                amount: 1,
            },
        ]
    );
    assert!(hand(&app, PLAYER_A).is_empty());
}

#[test]
fn test_os12_mana_cap_reward_emits_once_without_ceiling_check() {
    let mut app = app_with_fake_objective(SEED_PRODUCES_MANA_CAP);

    apply_fake_consequence(&mut app, LANE_1);

    assert_eq!(
        read_messages::<ManaCapIncreased>(&app),
        vec![ManaCapIncreased {
            player: PLAYER_A,
            amount: 1,
        }]
    );
}

#[test]
fn test_os15_free_card_reward_with_full_hand_awards_one_gold_without_draw_seed() {
    let mut app = app_with_fake_objective(SEED_PRODUCES_FREE_CARD);
    set_hand_len(&mut app, PLAYER_A, MAX_HAND_SIZE);
    let seed_index_before = seed_index(&app);
    let copies_before = copies_remaining(&app, PLAYER_A, REWARD_CARD_ID);

    apply_fake_consequence(&mut app, LANE_1);

    assert_eq!(hand(&app, PLAYER_A).len(), MAX_HAND_SIZE);
    assert_eq!(seed_index(&app), seed_index_before + 1);
    assert_eq!(
        copies_remaining(&app, PLAYER_A, REWARD_CARD_ID),
        copies_before
    );
    assert_eq!(
        read_messages::<AwardGold>(&app),
        vec![
            AwardGold {
                player: PLAYER_A,
                amount: 3,
            },
            AwardGold {
                player: PLAYER_A,
                amount: 1,
            },
        ]
    );
}

#[test]
fn test_os19_seeded_d4_zero_and_one_choose_expected_paths() {
    let mut mana_app = app_with_fake_objective(SEED_PRODUCES_MANA_CAP);
    apply_fake_consequence(&mut mana_app, LANE_1);
    assert_eq!(read_messages::<ManaCapIncreased>(&mana_app).len(), 1);
    assert!(hand(&mana_app, PLAYER_A).is_empty());

    let mut free_card_app = app_with_fake_objective(SEED_PRODUCES_FREE_CARD);
    apply_fake_consequence(&mut free_card_app, LANE_1);
    assert!(read_messages::<ManaCapIncreased>(&free_card_app).is_empty());
    assert_eq!(hand(&free_card_app, PLAYER_A), vec![REWARD_CARD_ID]);
}

#[test]
fn test_os22_free_card_pool_exhausted_is_noop_after_draw_seed() {
    let mut app = app_with_fake_objective(SEED_PRODUCES_FREE_CARD);
    {
        let mut pools = app.world_mut().resource_mut::<PlayerPools>();
        let pool = pools.pools.get_mut(&PLAYER_A).expect("player pool exists");
        while pool.copies_remaining(REWARD_CARD_ID) > 0 {
            pool.distribute(REWARD_CARD_ID)
                .expect("reward card should distribute until exhausted");
        }
    }
    let seed_index_before = seed_index(&app);

    apply_fake_consequence(&mut app, LANE_1);

    assert!(hand(&app, PLAYER_A).is_empty());
    assert!(read_messages::<ManaCapIncreased>(&app).is_empty());
    assert_eq!(
        read_messages::<AwardGold>(&app),
        vec![AwardGold {
            player: PLAYER_A,
            amount: 3,
        }]
    );
    assert_eq!(seed_index(&app), seed_index_before + 2);
}

#[test]
fn test_os26_same_resolution_mana_then_free_card_consumes_one_draw_seed() {
    let mut app = app_with_fake_objective(SEED_PRODUCES_MANA_THEN_FREE_CARD);
    let seed_index_before = seed_index(&app);

    apply_fake_consequence(&mut app, LANE_1);
    apply_fake_consequence(&mut app, LANE_2);

    assert_eq!(
        read_messages::<ManaCapIncreased>(&app),
        vec![ManaCapIncreased {
            player: PLAYER_A,
            amount: 1,
        }]
    );
    assert_eq!(hand(&app, PLAYER_A), vec![REWARD_CARD_ID]);
    assert_eq!(seed_index(&app), seed_index_before + 3);
    assert_eq!(copies_remaining(&app, PLAYER_A, REWARD_CARD_ID), 3);
}

#[test]
fn test_os27_fake_reward_pool_filter_is_unfiltered() {
    assert!(FAKE_REWARD_POOL_FILTER.rarity.is_none());
    assert!(FAKE_REWARD_POOL_FILTER.class.is_none());
    assert!(FAKE_REWARD_POOL_FILTER.card_type.is_none());
    assert!(FAKE_REWARD_POOL_FILTER.card_types.is_none());
    assert!(FAKE_REWARD_POOL_FILTER.max_cost.is_none());
}
