use ::shared::protocol::{PlayerSnapshot, S2CGameSnapshot, S2CGoldUpdate};
use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEconomyViewUpdateSource {
    GoldUpdate,
    Snapshot,
}

/// Client-side read model for the local player's private economy values.
///
/// This is presentation state only. It is updated from authoritative S2C
/// messages and reconnect snapshots, never from local input.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct PlayerEconomyView {
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u8,
    pub initialized: bool,
    pub last_update_source: Option<PlayerEconomyViewUpdateSource>,
}

impl PlayerEconomyView {
    pub fn apply_gold_update(&mut self, message: &S2CGoldUpdate) {
        self.gold = message.gold;
        self.current_mana = message.current_mana;
        self.reserve_mana = message.reserve_mana;
        self.mana_cap = message.mana_cap;
        self.initialized = true;
        self.last_update_source = Some(PlayerEconomyViewUpdateSource::GoldUpdate);
    }

    pub fn apply_player_snapshot(&mut self, player: &PlayerSnapshot) {
        self.gold = player.gold;
        self.current_mana = player.current_mana;
        self.reserve_mana = player.reserve_mana;
        self.mana_cap = player.mana_cap;
        self.initialized = true;
        self.last_update_source = Some(PlayerEconomyViewUpdateSource::Snapshot);
    }
}

#[derive(Message, Debug, Clone)]
pub struct PresentationGameSnapshotMessage(pub S2CGameSnapshot);

pub fn drain_gold_update_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CGoldUpdate>>,
    mut economy_view: ResMut<PlayerEconomyView>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            economy_view.apply_gold_update(&message);
        }
    }
}

pub fn drain_game_snapshot_receiver_system(
    mut receivers: Query<&mut MessageReceiver<S2CGameSnapshot>>,
    mut economy_view: ResMut<PlayerEconomyView>,
    mut snapshot_writer: MessageWriter<PresentationGameSnapshotMessage>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            if !apply_snapshot_to_player_economy_view(&message, &mut economy_view) {
                warn!(
                    "Presentation: snapshot for {:?} does not contain the local player economy",
                    message.recipient_player_id
                );
            }
            snapshot_writer.write(PresentationGameSnapshotMessage(message));
        }
    }
}

pub fn apply_snapshot_to_player_economy_view(
    snapshot: &S2CGameSnapshot,
    economy_view: &mut PlayerEconomyView,
) -> bool {
    let Some(local_player) = local_player_snapshot(snapshot) else {
        return false;
    };

    economy_view.apply_player_snapshot(local_player);
    true
}

pub fn local_player_snapshot(snapshot: &S2CGameSnapshot) -> Option<&PlayerSnapshot> {
    snapshot
        .players
        .iter()
        .find(|player| player.player_id == snapshot.recipient_player_id)
}
