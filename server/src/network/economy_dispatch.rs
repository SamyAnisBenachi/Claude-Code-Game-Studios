use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, PeerId, Server, ServerMultiMessageSender};
use shared::protocol::{
    ReliableChannel, S2CGoldBroadcast as ProtocolGoldBroadcast, S2CGoldUpdate as ProtocolGoldUpdate,
};
use shared::session::PlayerId;

use crate::core::economy::{
    on_draft_started, S2CGoldBroadcast as EconomyGoldBroadcast, S2CGoldUpdate as EconomyGoldUpdate,
};
use crate::core::session::PlayerConnectionMap;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EconomyNetworkSet {
    Dispatch,
}

pub struct EconomyNetworkPlugin;

impl Plugin for EconomyNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EconomyNetworkOutbox>().add_systems(
            Update,
            (dispatch_gold_update, dispatch_gold_broadcast)
                .chain()
                .in_set(EconomyNetworkSet::Dispatch)
                .after(on_draft_started),
        );
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DispatchedGoldUpdate {
    pub player_id: PlayerId,
    pub peer_id: PeerId,
    pub message: ProtocolGoldUpdate,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct EconomyNetworkOutbox {
    gold_updates: Vec<DispatchedGoldUpdate>,
    gold_broadcasts: Vec<ProtocolGoldBroadcast>,
}

#[allow(dead_code)]
impl EconomyNetworkOutbox {
    pub fn push_gold_update(&mut self, dispatch: DispatchedGoldUpdate) {
        self.gold_updates.push(dispatch);
    }

    pub fn push_gold_broadcast(&mut self, message: ProtocolGoldBroadcast) {
        self.gold_broadcasts.push(message);
    }

    pub fn gold_updates(&self) -> &[DispatchedGoldUpdate] {
        &self.gold_updates
    }

    pub fn gold_broadcasts(&self) -> &[ProtocolGoldBroadcast] {
        &self.gold_broadcasts
    }
}

pub fn dispatch_gold_update(
    mut updates: MessageReader<EconomyGoldUpdate>,
    connections: Res<PlayerConnectionMap>,
    mut outbox: Option<ResMut<EconomyNetworkOutbox>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server = server.single().ok();

    for update in updates.read() {
        let Some(peer_id) = peer_for_player(&connections, update.player) else {
            warn!(
                player_id = update.player.0,
                "dispatch_gold_update skipped because no PeerId is mapped"
            );
            continue;
        };

        let message = ProtocolGoldUpdate {
            gold: update.gold,
            current_mana: update.current_mana,
            reserve_mana: update.reserve_mana,
            mana_cap: update.mana_cap.min(u32::from(u8::MAX)) as u8,
        };

        if let Some(outbox) = outbox.as_deref_mut() {
            outbox.push_gold_update(DispatchedGoldUpdate {
                player_id: update.player,
                peer_id,
                message: message.clone(),
            });
        }

        if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
            let _ = sender.send::<ProtocolGoldUpdate, ReliableChannel>(
                &message,
                server,
                &NetworkTarget::Single(peer_id),
            );
        }
    }
}

pub fn dispatch_gold_broadcast(
    mut broadcasts: MessageReader<EconomyGoldBroadcast>,
    mut outbox: Option<ResMut<EconomyNetworkOutbox>>,
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
) {
    let server = server.single().ok();

    for broadcast in broadcasts.read() {
        let message = ProtocolGoldBroadcast {
            player_id: broadcast.player_id,
            gold: broadcast.gold,
            reserved_gold: broadcast.reserved_gold,
        };

        if let Some(outbox) = outbox.as_deref_mut() {
            outbox.push_gold_broadcast(message.clone());
        }

        if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
            let _ = sender.send::<ProtocolGoldBroadcast, ReliableChannel>(
                &message,
                server,
                &NetworkTarget::All,
            );
        }
    }
}

fn peer_for_player(connections: &PlayerConnectionMap, player_id: PlayerId) -> Option<PeerId> {
    connections
        .0
        .iter()
        .find_map(|(peer_id, mapped_player)| (*mapped_player == player_id).then_some(*peer_id))
}
