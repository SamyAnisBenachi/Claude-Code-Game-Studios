use std::collections::{HashMap, HashSet};
use std::time::Duration;

use bevy::prelude::*;
use lightyear::prelude::PeerId;
use server::core::economy::{PlayerEconomies, PlayerEconomy};
use server::core::rsm::{RoundPhase, RoundState};
use server::core::session::{
    flush_deferred_queue, process_reconnect_hello, DeferredMessage, PendingHello,
    PlayerConnectionMap, PlayerSessionData, PlayerSessions, ReconnectDispatch,
    ReconnectNetworkOutbox, ReconnectTracker, SessionConfig, SessionId,
};
use server::feature::acquisition::{
    defer_draft_offering, defer_shop_slots, DraftOfferingDispatch, PlayerHands, ShopSlotsDispatch,
};
use server::feature::auction::{
    defer_auction_outbox_for_reconnect, AuctionAcceptedDispatch, AuctionNetworkOutbox,
    AuctionRejectionDispatch,
};
use server::feature::objective::{HiddenObjectives, OBJECTIVE_LANE_COUNT};
use server::foundation::config::GameConfig;
use shared::card::{CardId, ClassId};
use shared::protocol::{
    BidRejectedReason, C2SHello, CardSource, GameMode, S2CAuctionBidAccepted,
    S2CAuctionBidRejected, S2CDraftOffering, S2CGoldUpdate, S2CShopSlots,
};
use shared::session::PlayerId;
use uuid::Uuid;

fn player(id: u64) -> PlayerId {
    PlayerId(id)
}

fn session_config(player_a: PlayerId, player_b: PlayerId) -> SessionConfig {
    SessionConfig {
        mode: GameMode::OneVOne,
        player_count: 2,
        team_map: HashMap::from([(player_a, 0), (player_b, 1)]),
        class_map: HashMap::from([(player_a, ClassId::Iop), (player_b, ClassId::Cra)]),
    }
}

fn player_sessions(player_a: PlayerId, player_b: PlayerId) -> PlayerSessions {
    let mut sessions = PlayerSessions::default();
    sessions.players.insert(
        player_a,
        PlayerSessionData {
            class: ClassId::Iop,
            class_locked: true,
        },
    );
    sessions.players.insert(
        player_b,
        PlayerSessionData {
            class: ClassId::Cra,
            class_locked: true,
        },
    );
    sessions
}

fn placement_round_state() -> RoundState {
    let mut timer = Timer::from_seconds(10.0, TimerMode::Once);
    timer.tick(Duration::from_secs(4));
    RoundState {
        phase: RoundPhase::Placement,
        round_number: 4,
        placement_timer: Some(timer),
        ..RoundState::new()
    }
}

fn hidden_objectives(player_a: PlayerId, player_b: PlayerId) -> HiddenObjectives {
    let mut hidden = HiddenObjectives::default();
    for owner in [player_a, player_b] {
        for lane in 1..=OBJECTIVE_LANE_COUNT {
            hidden
                .identities
                .insert((owner, lane), owner == player_b && lane % 2 == 0);
        }
    }
    hidden
}

#[test]
fn reconnect_hello_sends_snapshot_sequence_and_restores_sang_meprise() {
    let player_a = player(1);
    let player_b = player(2);
    let old_peer = PeerId::Netcode(10);
    let new_peer = PeerId::Netcode(11);
    let opponent_peer = PeerId::Netcode(12);
    let token = [7; 16];
    let session_id = SessionId(Uuid::from_u128(99));
    let mut world = World::new();
    world.insert_resource(GameConfig(shared::config::GameConfig::default()));
    world.insert_resource(session_config(player_a, player_b));
    world.insert_resource(player_sessions(player_a, player_b));
    world.insert_resource(placement_round_state());
    world.insert_resource(PlayerEconomies(HashMap::from([(
        player_a,
        PlayerEconomy {
            gold: 10,
            current_mana: 3,
            reserve_mana: 1,
            mana_cap: 8,
            reserved_gold: 0,
        },
    )])));
    world.insert_resource(PlayerHands {
        hands: HashMap::from([(player_a, vec![CardId(101)])]),
    });
    world.insert_resource(hidden_objectives(player_a, player_b));
    world.insert_resource(PlayerConnectionMap(HashMap::from([
        (old_peer, player_a),
        (opponent_peer, player_b),
    ])));
    world.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, true), (player_b, true)]),
        deferred_queue: HashMap::new(),
        token_map: HashMap::from([(token, (session_id, player_a))]),
        sang_meprise_sent_to: HashSet::from([player_a]),
        ..Default::default()
    });

    let entity = world.spawn_empty().id();
    let result = process_reconnect_hello(
        &mut world,
        entity,
        new_peer,
        C2SHello {
            protocol_version: 1,
            session_token: Some(token),
        },
    );

    assert_eq!(result.closes.len(), 0);
    assert_eq!(result.dispatches.len(), 6);
    assert!(matches!(
        result.dispatches[0],
        ReconnectDispatch::Handshake { .. }
    ));
    assert!(matches!(
        result.dispatches[1],
        ReconnectDispatch::GameSnapshot { .. }
    ));
    assert!(matches!(
        result.dispatches[2],
        ReconnectDispatch::ObjectiveIdentities { .. }
    ));
    assert!(matches!(
        result.dispatches[3],
        ReconnectDispatch::PhaseChanged { .. }
    ));
    assert!(matches!(
        result.dispatches[4],
        ReconnectDispatch::SangMepriseReveal { .. }
    ));
    assert!(matches!(
        result.dispatches[5],
        ReconnectDispatch::OpponentReconnected { .. }
    ));

    let ReconnectDispatch::Handshake { message, .. } = &result.dispatches[0] else {
        unreachable!("checked above");
    };
    assert_eq!(message.session_token, token);

    let ReconnectDispatch::GameSnapshot { message, .. } = &result.dispatches[1] else {
        unreachable!("checked above");
    };
    assert_eq!(message.recipient_player_id, player_a);
    assert_eq!(message.timer_remaining_ms, Some(6000));
    assert!(message
        .active_sang_meprise_reveals
        .as_ref()
        .is_some_and(|reveals| !reveals.is_empty()));

    let ReconnectDispatch::ObjectiveIdentities { message, .. } = &result.dispatches[2] else {
        unreachable!("checked above");
    };
    assert_eq!(
        message.identities,
        vec![(1, false), (2, false), (3, false), (4, false), (5, false)]
    );

    let ReconnectDispatch::OpponentReconnected {
        recipients,
        message,
    } = &result.dispatches[5]
    else {
        unreachable!("checked above");
    };
    assert_eq!(message.player_id, player_a);
    assert_eq!(recipients, &vec![opponent_peer]);

    let connections = world.resource::<PlayerConnectionMap>();
    assert_eq!(connections.0.get(&new_peer), Some(&player_a));
    assert!(!connections.0.contains_key(&old_peer));
    assert_eq!(
        world
            .resource::<ReconnectTracker>()
            .snapshot_sent
            .get(&player_a),
        Some(&true)
    );
}

#[test]
fn reconnect_hello_rejects_unknown_token_and_closes_connection() {
    let peer = PeerId::Netcode(44);
    let mut world = World::new();
    world.insert_resource(GameConfig(shared::config::GameConfig::default()));
    world.insert_resource(PlayerConnectionMap::default());
    world.insert_resource(ReconnectTracker::default());

    let entity = world.spawn_empty().id();
    let result = process_reconnect_hello(
        &mut world,
        entity,
        peer,
        C2SHello {
            protocol_version: 123,
            session_token: Some([9; 16]),
        },
    );

    assert_eq!(result.dispatches.len(), 1);
    assert!(matches!(
        result.dispatches[0],
        ReconnectDispatch::HandshakeRejected { .. }
    ));
    assert_eq!(result.closes.len(), 1);
    assert_eq!(result.closes[0].peer_id, peer);
    assert!(world.resource::<PlayerConnectionMap>().0.is_empty());
}

#[test]
fn deferred_queue_flush_preserves_original_order() {
    let player = player(1);
    let peer = PeerId::Netcode(20);
    let mut world = World::new();
    world.insert_resource(PlayerConnectionMap(HashMap::from([(peer, player)])));
    world.insert_resource(ReconnectNetworkOutbox::default());
    world.insert_resource(ReconnectTracker {
        snapshot_sent: HashMap::from([(player, true)]),
        deferred_queue: HashMap::from([(
            player,
            vec![
                DeferredMessage::GoldUpdate(S2CGoldUpdate {
                    gold: 12,
                    current_mana: 3,
                    reserve_mana: 1,
                    mana_cap: 8,
                }),
                DeferredMessage::CardAcquired {
                    card_id: CardId(99),
                    source: CardSource::PrismLane1,
                },
                DeferredMessage::PrismRewardDropped {
                    player_id: player,
                    lane: 3,
                },
            ],
        )]),
        ..Default::default()
    });

    flush_deferred_queue(&mut world);

    let outbox = world.resource::<ReconnectNetworkOutbox>();
    assert_eq!(outbox.dispatches().len(), 3);
    assert!(matches!(
        outbox.dispatches()[0],
        ReconnectDispatch::Deferred {
            message: DeferredMessage::GoldUpdate(_),
            ..
        }
    ));
    assert!(matches!(
        outbox.dispatches()[1],
        ReconnectDispatch::Deferred {
            message: DeferredMessage::CardAcquired { .. },
            ..
        }
    ));
    assert!(matches!(
        outbox.dispatches()[2],
        ReconnectDispatch::Deferred {
            message: DeferredMessage::PrismRewardDropped { .. },
            ..
        }
    ));
    assert!(world
        .resource::<ReconnectTracker>()
        .deferred_queue
        .get(&player)
        .is_some_and(Vec::is_empty));
}

#[test]
fn hello_timeout_closes_silent_connection_without_s2c() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ReconnectNetworkOutbox::default());
    let peer = PeerId::Netcode(55);
    let entity = app.world_mut().spawn_empty().id();
    app.insert_resource(ReconnectTracker {
        pending_hellos: HashMap::from([(
            peer,
            PendingHello {
                entity,
                remaining_ms: 0,
            },
        )]),
        ..Default::default()
    });

    server::core::session::hello_timeout_watchdog(app.world_mut());

    let outbox = app.world().resource::<ReconnectNetworkOutbox>();
    assert!(outbox.dispatches().is_empty());
    assert_eq!(outbox.closes().len(), 1);
    assert_eq!(outbox.closes()[0].peer_id, peer);
}

#[test]
fn acquisition_unicast_helpers_defer_while_snapshot_pending() {
    let player = player(1);
    let mut tracker = ReconnectTracker {
        snapshot_sent: HashMap::from([(player, false)]),
        ..Default::default()
    };

    let draft = DraftOfferingDispatch {
        player_id: player,
        peer_id: Some(PeerId::Netcode(1)),
        message: S2CDraftOffering {
            card_ids: vec![CardId(1)],
        },
    };
    let slots = ShopSlotsDispatch {
        player_id: player,
        peer_id: Some(PeerId::Netcode(1)),
        message: S2CShopSlots {
            slots: vec![Some(CardId(2))],
        },
    };

    assert!(defer_draft_offering(Some(&mut tracker), &draft));
    assert!(defer_shop_slots(Some(&mut tracker), &slots));

    let queued = tracker
        .deferred_queue
        .get(&player)
        .expect("pending player should have deferred acquisition messages");
    assert!(matches!(queued[0], DeferredMessage::DraftOffering(_)));
    assert!(matches!(queued[1], DeferredMessage::ShopSlots(_)));
}

#[test]
fn auction_dispatch_guard_defers_pending_reconnect_players() {
    let player_a = player(1);
    let player_b = player(2);
    let mut tracker = ReconnectTracker {
        snapshot_sent: HashMap::from([(player_a, false), (player_b, true)]),
        ..Default::default()
    };
    let connections = PlayerConnectionMap(HashMap::from([
        (PeerId::Netcode(1), player_a),
        (PeerId::Netcode(2), player_b),
    ]));
    let mut outbox = AuctionNetworkOutbox::default();
    outbox.push_rejected(AuctionRejectionDispatch {
        player_id: player_a,
        peer_id: Some(PeerId::Netcode(1)),
        message: S2CAuctionBidRejected {
            reason: BidRejectedReason::AmountTooLow,
        },
    });
    outbox.push_accepted(AuctionAcceptedDispatch {
        player_id: player_b,
        peer_id: Some(PeerId::Netcode(2)),
        message: S2CAuctionBidAccepted {
            bidder: player_b,
            amount: 4,
            new_timer_ms: 1000,
        },
    });

    let pending =
        defer_auction_outbox_for_reconnect(&outbox, Some(&connections), Some(&mut tracker));

    assert!(pending.contains(&player_a));
    let queued = tracker
        .deferred_queue
        .get(&player_a)
        .expect("pending reconnect player should have auction messages deferred");
    assert!(matches!(queued[0], DeferredMessage::AuctionBidRejected(_)));
    assert!(matches!(queued[1], DeferredMessage::AuctionBidAccepted(_)));
}
