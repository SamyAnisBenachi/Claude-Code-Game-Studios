// server/src/core/economy/system.rs -- Economy phase subscribers.

use bevy::prelude::*;
use shared::protocol::DraftPhase;
use shared::session::PlayerId;

use crate::core::economy::api;
use crate::core::economy::state::{InterestSnapshots, PlayerEconomies, PlayerEconomy};
use crate::core::rsm::{DraftStarted, ResolutionPhaseEntered, SessionReady};
use crate::core::session::SessionConfig;
use crate::foundation::config::GameConfig;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EconomySystemSet {
    ResolutionEnd,
}

/// Internal server event consumed by the later network dispatch story.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct S2CGoldUpdate {
    pub player: PlayerId,
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u32,
}

/// Internal server event consumed by the later network dispatch story.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct S2CGoldBroadcast {
    pub player: PlayerId,
    pub gold: u32,
}

pub fn initialise_player_economies(
    _trigger: On<SessionReady>,
    session: Res<SessionConfig>,
    config: Res<GameConfig>,
    mut economies: ResMut<PlayerEconomies>,
) {
    for player in session.players() {
        economies.0.insert(
            player,
            PlayerEconomy {
                gold: config.starting_gold,
                current_mana: 0,
                reserve_mana: 0,
                mana_cap: config.mana_cap,
                reserved_gold: 0,
            },
        );
    }
}

pub fn on_resolution_phase_entered(
    mut resolution_entered: MessageReader<ResolutionPhaseEntered>,
    economies: Res<PlayerEconomies>,
    mut interest_snapshots: ResMut<InterestSnapshots>,
    session: Res<SessionConfig>,
) {
    for _event in resolution_entered.read() {
        for player in session.players() {
            let Some(economy) = economies.0.get(&player) else {
                continue;
            };

            interest_snapshots.0.insert(player, economy.gold);
        }
    }
}

pub fn discard_current_mana_at_resolution_end(
    mut resolution_entered: MessageReader<ResolutionPhaseEntered>,
    mut economies: ResMut<PlayerEconomies>,
    session: Res<SessionConfig>,
) {
    for _event in resolution_entered.read() {
        for player in session.players() {
            let Some(economy) = economies.0.get_mut(&player) else {
                continue;
            };

            api::discard_current_mana(economy);
        }
    }
}

pub fn on_draft_started(
    mut draft_started: MessageReader<DraftStarted>,
    session: Option<Res<SessionConfig>>,
    mut economies: ResMut<PlayerEconomies>,
    mut interest_snapshots: ResMut<InterestSnapshots>,
    config: Option<Res<GameConfig>>,
    mut gold_updates: MessageWriter<S2CGoldUpdate>,
    mut gold_broadcasts: MessageWriter<S2CGoldBroadcast>,
) {
    let (Some(session), Some(config)) = (session, config) else {
        return;
    };

    for event in draft_started.read() {
        for player in session.players() {
            let Some(economy) = economies.0.get_mut(&player) else {
                continue;
            };

            api::apply_mana_ramp(economy, event.round);

            if matches!(event.phase, DraftPhase::Auction | DraftPhase::Shop) {
                let snap = interest_snapshots.0.remove(&player).unwrap_or(0);
                let threshold = config.interest_threshold_gold.max(1);
                let interest = (snap / threshold).min(config.interest_max_bonus);
                api::apply_gold_award(
                    economy,
                    config.gold_baseline_per_round.saturating_add(interest),
                );
            }

            gold_updates.write(S2CGoldUpdate {
                player,
                gold: economy.gold,
                current_mana: economy.current_mana,
                reserve_mana: economy.reserve_mana,
                mana_cap: economy.mana_cap,
            });
            gold_broadcasts.write(S2CGoldBroadcast {
                player,
                gold: economy.gold,
            });
        }
    }
}
