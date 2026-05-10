// server/src/core/economy/system.rs -- Economy phase subscribers.

use bevy::prelude::*;
use shared::protocol::DraftPhase;
use shared::session::PlayerId;

use crate::core::economy::api;
use crate::core::economy::state::{InterestSnapshots, PlayerEconomies, PlayerEconomy};
use crate::core::rsm::{DraftStarted, ResolutionComplete};
use crate::core::session::SessionConfig;
use crate::foundation::config::GameConfig;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EconomySystemSet {
    /// Drains `AwardGold` and `ManaCapIncreased` messages emitted by combat /
    /// objective during RESOLUTION. Runs before `ResolutionEnd` so post-reward
    /// gold lands in the same-frame `InterestSnapshots` capture and post-reward
    /// `mana_cap` is visible to the next `DraftStarted` mana ramp.
    RewardConsumers,
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
    pub player_id: PlayerId,
    pub gold: u32,
    pub reserved_gold: u32,
}

/// Domain signal for persistent gold awarded during RESOLUTION.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AwardGold {
    pub player: PlayerId,
    pub amount: u32,
}

/// Domain signal for a fake-objective mana cap reward.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManaCapIncreased {
    pub player: PlayerId,
    pub amount: u32,
}

pub fn gold_broadcast(player_id: PlayerId, economy: &PlayerEconomy) -> S2CGoldBroadcast {
    S2CGoldBroadcast {
        player_id,
        gold: economy.gold,
        reserved_gold: economy.reserved_gold,
    }
}

/// Consumes `AwardGold` messages and applies them through `economy_api::apply_gold_award`.
///
/// `AwardGold` is currently emitted ONLY by the fake-objective FreeCardPick
/// hand-full fallback in `feature/objective/system.rs::resolve_fake_reward_free_card_pick`.
/// Combat-side objective gold uses the direct `apply_gold_award` path inside
/// `resolve_combat` to avoid double-awarding the same destruction; per the
/// control manifest, `resolve_combat` is the exclusive in-RESOLUTION writer
/// for direct kill/objective gold awards.
pub fn apply_award_gold_messages(
    mut awards: MessageReader<AwardGold>,
    mut economies: ResMut<PlayerEconomies>,
) {
    for event in awards.read() {
        if let Some(economy) = economies.0.get_mut(&event.player) {
            api::apply_gold_award(economy, event.amount);
        }
    }
}

/// Consumes `ManaCapIncreased` messages by applying `economy_api::increment_mana_cap`.
///
/// Each call to `increment_mana_cap` clamps at `GameConfig.mana_cap_max`. The
/// `amount` field is the number of unit increments (objective fake-reward
/// currently emits `amount: 1`).
pub fn apply_mana_cap_increased_messages(
    mut events: MessageReader<ManaCapIncreased>,
    mut economies: ResMut<PlayerEconomies>,
    config: Option<Res<GameConfig>>,
) {
    let Some(config) = config else { return };
    for event in events.read() {
        if let Some(economy) = economies.0.get_mut(&event.player) {
            for _ in 0..event.amount {
                api::increment_mana_cap(economy, &config.0);
            }
        }
    }
}

pub fn on_resolution_complete(
    mut resolution_complete: MessageReader<ResolutionComplete>,
    mut economies: ResMut<PlayerEconomies>,
    mut interest_snapshots: ResMut<InterestSnapshots>,
    session: Res<SessionConfig>,
) {
    for _event in resolution_complete.read() {
        for player in session.players() {
            let Some(economy) = economies.0.get_mut(&player) else {
                continue;
            };

            interest_snapshots.0.insert(player, economy.gold);
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
            let economy = economies.0.entry(player).or_insert_with(|| PlayerEconomy {
                gold: config.starting_gold,
                current_mana: 0,
                reserve_mana: 0,
                mana_cap: config.mana_cap,
                reserved_gold: 0,
            });

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
            gold_broadcasts.write(gold_broadcast(player, economy));
        }
    }
}
