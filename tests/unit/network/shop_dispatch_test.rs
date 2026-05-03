use std::collections::HashMap;

use lightyear::prelude::PeerId;
use server::core::session::{DeferredMessage, PlayerConnectionMap, ReconnectTracker};
use server::feature::acquisition::{
    defer_draft_offering, defer_shop_slots, prepare_draft_offering_dispatch,
    prepare_shop_slots_dispatch, DraftOfferingDispatch, ShopSlotsDispatch,
};
use shared::card::CardId;
use shared::protocol::{
    self, ProtocolChannel, ProtocolDirection, ProtocolRegistry, S2CDraftOffering, S2CShopSlots,
};
use shared::session::PlayerId;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn card(id: u32) -> CardId {
    CardId(id)
}

fn connections() -> PlayerConnectionMap {
    PlayerConnectionMap(HashMap::from([
        (PeerId::Netcode(12), player(2)),
        (PeerId::Netcode(11), player(1)),
    ]))
}

#[test]
fn test_shop_slots_message_targets_owner_peer() {
    let message = S2CShopSlots {
        slots: vec![Some(card(101)), Some(card(102)), Some(card(103))],
    };
    let dispatch = prepare_shop_slots_dispatch(player(1), message, Some(&connections()));

    assert_eq!(dispatch.player_id, player(1));
    assert_eq!(dispatch.peer_id, Some(PeerId::Netcode(11)));
    assert_ne!(dispatch.peer_id, Some(PeerId::Netcode(12)));
    assert_eq!(
        dispatch.message.slots,
        vec![Some(card(101)), Some(card(102)), Some(card(103))]
    );

    let missing = prepare_shop_slots_dispatch(
        player(99),
        S2CShopSlots {
            slots: vec![Some(card(201))],
        },
        Some(&connections()),
    );
    assert_eq!(missing.peer_id, None);
}

#[test]
fn test_draft_offering_message_targets_owner_peer() {
    let offering = (1..=9).map(card).collect::<Vec<_>>();
    let message = S2CDraftOffering {
        card_ids: offering.clone(),
    };
    let dispatch = prepare_draft_offering_dispatch(player(1), message, Some(&connections()));

    assert_eq!(dispatch.player_id, player(1));
    assert_eq!(dispatch.peer_id, Some(PeerId::Netcode(11)));
    assert_ne!(dispatch.peer_id, Some(PeerId::Netcode(12)));
    assert_eq!(dispatch.message.card_ids, offering);

    let partial = S2CDraftOffering {
        card_ids: vec![
            card(301),
            card(302),
            card(303),
            card(304),
            card(305),
            card(306),
            card(307),
        ],
    };
    let partial_dispatch =
        prepare_draft_offering_dispatch(player(1), partial, Some(&connections()));
    assert_eq!(partial_dispatch.message.card_ids.len(), 7);
}

#[test]
fn test_partial_shop_slots_preserve_empty_slots() {
    let dispatch = prepare_shop_slots_dispatch(
        player(1),
        S2CShopSlots {
            slots: vec![Some(card(101)), Some(card(102)), None],
        },
        Some(&connections()),
    );

    assert_eq!(
        dispatch.message.slots,
        vec![Some(card(101)), Some(card(102)), None]
    );
}

#[test]
fn test_dispatch_queues_messages_while_reconnect_snapshot_pending() {
    let mut tracker = ReconnectTracker {
        snapshot_sent: HashMap::from([(player(1), false)]),
        ..Default::default()
    };
    let draft = DraftOfferingDispatch {
        player_id: player(1),
        peer_id: Some(PeerId::Netcode(11)),
        message: S2CDraftOffering {
            card_ids: vec![card(1), card(2), card(3)],
        },
    };
    let slots = ShopSlotsDispatch {
        player_id: player(1),
        peer_id: Some(PeerId::Netcode(11)),
        message: S2CShopSlots {
            slots: vec![Some(card(10)), None, Some(card(12))],
        },
    };

    assert!(defer_draft_offering(Some(&mut tracker), &draft));
    assert!(defer_shop_slots(Some(&mut tracker), &slots));

    let queued = tracker
        .deferred_queue
        .get(&player(1))
        .expect("pending reconnect player should have acquisition messages queued");
    assert_eq!(queued.len(), 2);
    match &queued[0] {
        DeferredMessage::DraftOffering(message) => {
            assert_eq!(message.card_ids, vec![card(1), card(2), card(3)]);
        }
        other => panic!("expected draft offering, got {other:?}"),
    }
    match &queued[1] {
        DeferredMessage::ShopSlots(message) => {
            assert_eq!(message.slots, vec![Some(card(10)), None, Some(card(12))]);
        }
        other => panic!("expected shop slots, got {other:?}"),
    }
}

#[test]
fn test_shop_dispatch_messages_registered_reliable_s2c() {
    #[derive(Default)]
    struct RecordingRegistry {
        messages: Vec<(String, ProtocolDirection, ProtocolChannel)>,
    }

    impl ProtocolRegistry for RecordingRegistry {
        fn add_channel<C: Send + Sync + 'static>(&mut self, _channel: ProtocolChannel) {}

        fn add_message<
            M: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        >(
            &mut self,
            direction: ProtocolDirection,
            channel: ProtocolChannel,
        ) {
            self.messages
                .push((std::any::type_name::<M>().to_string(), direction, channel));
        }
    }

    let mut registry = RecordingRegistry::default();
    protocol::register_protocol(&mut registry);

    for message_name in [
        std::any::type_name::<S2CShopSlots>(),
        std::any::type_name::<S2CDraftOffering>(),
    ] {
        let registrations = registry
            .messages
            .iter()
            .filter(|(registered, _, _)| registered == message_name)
            .collect::<Vec<_>>();
        assert_eq!(registrations.len(), 1);
        assert_eq!(
            registrations[0],
            &(
                message_name.to_string(),
                ProtocolDirection::ServerToClient,
                ProtocolChannel::Reliable,
            )
        );
    }
}
