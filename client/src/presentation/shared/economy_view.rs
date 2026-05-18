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
            tracing::info!(
                gold = message.gold,
                current_mana = message.current_mana,
                reserve_mana = message.reserve_mana,
                mana_cap = message.mana_cap,
                msg_type = "S2CGoldUpdate",
                "drain_gold_update: recv"
            );
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
            tracing::info!(
                player_id = ?message.recipient_player_id,
                phase = ?message.phase,
                round_number = message.round_number,
                players_len = message.players.len(),
                msg_type = "S2CGameSnapshot",
                "drain_game_snapshot: recv"
            );
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

/// Pure helper — compute the projected `(current_mana, reserve_mana)` pair the
/// player would be left with if a card of `cost` mana were paid right now.
///
/// Returns `None` when the card is unaffordable (`current + reserve < cost`),
/// matching the affordability rule used by
/// `client/src/ui/hand/drag_state_visuals.rs::slot_is_affordable` so the
/// HUD preview affordance stays consistent with the per-slot disabled
/// overlay.
///
/// Spend order mirrors the canonical default split applied at drop staging in
/// `client/src/ui/hand/mod.rs` (see `placement_drop` → `PlacedCardSubmit`
/// construction): current mana is consumed first, then reserve. Used by the
/// HUD mana label to paint a projected affordance during placement drag
/// (PROMPT 1228 / HUNT-1201-12). Display-only — never mutates server state.
pub fn project_mana_after_spend(
    current_mana: u32,
    reserve_mana: u32,
    cost: u32,
) -> Option<(u32, u32)> {
    let spend_from_current = cost.min(current_mana);
    let remaining_cost = cost.saturating_sub(spend_from_current);
    if remaining_cost > reserve_mana {
        return None;
    }
    Some((
        current_mana - spend_from_current,
        reserve_mana - remaining_cost,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_mana_zero_cost_returns_inputs_unchanged() {
        assert_eq!(project_mana_after_spend(5, 3, 0), Some((5, 3)));
    }

    #[test]
    fn project_mana_pays_from_current_first() {
        assert_eq!(project_mana_after_spend(5, 3, 2), Some((3, 3)));
    }

    #[test]
    fn project_mana_spills_into_reserve_when_current_insufficient() {
        assert_eq!(project_mana_after_spend(2, 5, 4), Some((0, 3)));
    }

    #[test]
    fn project_mana_exhausts_pools_exactly() {
        assert_eq!(project_mana_after_spend(3, 4, 7), Some((0, 0)));
    }

    #[test]
    fn project_mana_returns_none_when_unaffordable() {
        assert_eq!(project_mana_after_spend(2, 1, 5), None);
    }

    #[test]
    fn project_mana_handles_zero_pools() {
        assert_eq!(project_mana_after_spend(0, 0, 0), Some((0, 0)));
        assert_eq!(project_mana_after_spend(0, 0, 1), None);
    }
}
