use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use server::core::pool::PlayerPool;
use server::feature::acquisition::DRAFT_INITIAL_OFFERING_COUNT;
use server::foundation::config::{validate_card_catalog, CardCatalog};
use shared::card::{CardData, CardId, CardType, ClassId, Rarity};

const CONTENT_001A_IDS: &[CardId] = &[
    CardId(101),
    CardId(102),
    CardId(103),
    CardId(104),
    CardId(105),
    CardId(106),
    CardId(107),
    CardId(108),
];

fn cards_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server crate should live under workspace root")
        .join("assets")
        .join("data")
        .join("cards.json")
}

fn load_cards() -> Vec<CardData> {
    let raw = std::fs::read_to_string(cards_path()).expect("assets/data/cards.json should exist");
    serde_json::from_str(&raw).expect("cards.json should deserialize as CardData")
}

fn catalog_from(cards: &[CardData]) -> CardCatalog {
    CardCatalog {
        cards: cards.iter().cloned().map(|card| (card.id, card)).collect(),
    }
}

#[test]
fn content_001a_catalog_validates_and_has_no_duplicate_ids() {
    let cards = load_cards();
    let mut seen = HashSet::new();

    for card in &cards {
        assert!(
            seen.insert(card.id),
            "duplicate card id in cards.json: {:?}",
            card.id
        );
    }

    assert_eq!(validate_card_catalog(&catalog_from(&cards)), Ok(()));
}

#[test]
fn content_001a_initial_draft_floor_covers_friend_game_classes() {
    let cards = load_cards();
    let catalog = catalog_from(&cards);
    let pool = PlayerPool::initialize(&catalog.cards, &shared::config::GameConfig::default());

    for class in [ClassId::Iop, ClassId::Cra] {
        let eligible: HashSet<CardId> = catalog
            .cards
            .iter()
            .filter(|(_, card)| card.class == class || card.class == ClassId::Neutral)
            .map(|(card_id, _)| *card_id)
            .collect();

        assert!(
            eligible.len() >= DRAFT_INITIAL_OFFERING_COUNT as usize,
            "{class:?} should have at least {DRAFT_INITIAL_OFFERING_COUNT} initial draft candidates, got {}",
            eligible.len()
        );

        let offering =
            pool.draw_initial_draft(&catalog.cards, class, DRAFT_INITIAL_OFFERING_COUNT, 327);
        let distinct: HashSet<CardId> = offering.iter().copied().collect();

        assert_eq!(offering.len(), DRAFT_INITIAL_OFFERING_COUNT as usize);
        assert_eq!(distinct.len(), DRAFT_INITIAL_OFFERING_COUNT as usize);
        assert!(offering.iter().all(|card_id| eligible.contains(card_id)));
    }
}

#[test]
fn content_001a_auction_pool_has_neutral_rare_and_legendary_candidates() {
    let cards = load_cards();
    let catalog = catalog_from(&cards);
    let pool = PlayerPool::initialize(&catalog.cards, &shared::config::GameConfig::default());

    let neutral_rare: Vec<CardId> = auction_candidates(&catalog, &pool, Rarity::Rare);
    let neutral_legendary: Vec<CardId> = auction_candidates(&catalog, &pool, Rarity::Legendary);

    assert!(
        neutral_rare.len() >= 2,
        "auction pool should include at least two Neutral Rare candidates, got {neutral_rare:?}"
    );
    assert!(
        neutral_legendary.len() >= 1,
        "auction pool should include at least one Neutral Legendary candidate"
    );

    let drawn = PlayerPool::draw_auction_card(&pool, &catalog.cards, 327)
        .expect("auction pool should draw a Neutral Rare or Legendary card");
    let drawn_card = catalog
        .cards
        .get(&drawn)
        .expect("drawn auction card should exist");
    assert_eq!(drawn_card.class, ClassId::Neutral);
    assert!(matches!(
        drawn_card.rarity,
        Rarity::Rare | Rarity::Legendary
    ));
}

#[test]
fn content_001a_new_cards_require_no_unsupported_mechanics() {
    let cards_by_id: HashMap<CardId, CardData> = load_cards()
        .into_iter()
        .map(|card| (card.id, card))
        .collect();

    for card_id in CONTENT_001A_IDS {
        let card = cards_by_id
            .get(card_id)
            .unwrap_or_else(|| panic!("CONTENT-001A card {card_id:?} should exist"));
        assert_eq!(card.class, ClassId::Neutral);
        assert!(
            matches!(
                card.card_type,
                CardType::Minion | CardType::Field | CardType::Order
            ),
            "CONTENT-001A card {card_id:?} uses unsupported type {:?}",
            card.card_type
        );
        assert!(
            card.family
                .as_ref()
                .is_some_and(|family| !family.is_empty()),
            "CONTENT-001A neutral card {card_id:?} should be indexed for neutral shop draws"
        );
        assert!(
            card.keywords.is_empty(),
            "CONTENT-001A card {card_id:?} should not add keyword requirements"
        );
        assert!(
            card.effect_text.is_empty(),
            "CONTENT-001A card {card_id:?} should not add effect engine requirements"
        );
    }
}

fn auction_candidates(catalog: &CardCatalog, pool: &PlayerPool, rarity: Rarity) -> Vec<CardId> {
    let mut candidates: Vec<CardId> = catalog
        .cards
        .iter()
        .filter(|(card_id, card)| {
            card.class == ClassId::Neutral
                && card.rarity == rarity
                && pool.copies_remaining(**card_id) > 0
        })
        .map(|(card_id, _)| *card_id)
        .collect();
    candidates.sort_by_key(|card_id| card_id.0);
    candidates
}
