use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::feature::acquisition::{PlayerHands, PlayerShopState, ShopPhase, ShopStates};
use shared::card::CardId;
use shared::protocol::C2SPurchaseCard;
use shared::session::PlayerId;

fn economy(gold: u32) -> PlayerEconomy {
    PlayerEconomy {
        gold,
        current_mana: 0,
        reserve_mana: 0,
        mana_cap: 10,
        reserved_gold: 0,
    }
}

fn inactive_shop_state(player: PlayerId) -> ShopStates {
    ShopStates {
        players: HashMap::from([(
            player,
            PlayerShopState {
                phase: ShopPhase::Inactive,
                displayed_this_draft: Default::default(),
                current_slots: [None, None, None],
                refresh_count_this_draft: 0,
            },
        )]),
    }
}

#[test]
fn prism_bypass_pushes_card_without_purchase_or_gold_change() {
    let player = PlayerId(1);
    let card_id = CardId(42);
    let starting_gold = 7;

    let mut world = World::new();
    world.insert_resource(PlayerHands::default());
    world.insert_resource(inactive_shop_state(player));
    world.insert_resource(PlayerEconomies(HashMap::from([(
        player,
        economy(starting_gold),
    )])));

    let hand_len_before = world.resource::<PlayerHands>().hand_len(player);
    let shop_before = world
        .resource::<ShopStates>()
        .players
        .get(&player)
        .expect("inactive shop state should exist for player")
        .clone();

    {
        let mut hands = world.resource_mut::<PlayerHands>();
        hands.push_card(player, card_id);
    }

    let hands = world.resource::<PlayerHands>();
    assert_eq!(hands.hand_len(player), hand_len_before + 1);
    assert_eq!(
        hands
            .hands
            .get(&player)
            .and_then(|hand| hand.last())
            .copied(),
        Some(card_id)
    );

    let economies = world.resource::<PlayerEconomies>();
    assert_eq!(
        economies
            .0
            .get(&player)
            .expect("player economy should still exist")
            .gold,
        starting_gold
    );

    let shop_after = world
        .resource::<ShopStates>()
        .players
        .get(&player)
        .expect("inactive shop state should remain present");
    assert_eq!(shop_after, &shop_before);

    let mut purchase_receivers = world.query::<&MessageReceiver<C2SPurchaseCard>>();
    assert_eq!(
        purchase_receivers.iter(&world).count(),
        0,
        "Prism-style bypass must not route through C2SPurchaseCard receivers"
    );
}
