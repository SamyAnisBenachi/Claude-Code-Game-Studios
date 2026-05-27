//! Bot lobby auto-confirm loop (PROMPT 1514 — BOT-ROOM-JOIN-LOOP).
//!
//! When a human owner has filled the opposing slot with a bot via the existing
//! lobby protocol (PROMPT 1430), nothing on the server currently progresses
//! the bot's class selection. `f4_session_ready` requires every occupied slot
//! to have a confirmed class in [`ClassSelections`], so a lobby that contains
//! a bot can never reach `GameActive`.
//!
//! This system fills that exact gap: for every bot slot in a `LobbyWaiting`
//! room with no class assigned yet, it picks a deterministic class, mirrors it
//! onto the slot, and inserts it into [`ClassSelections`] so the existing
//! `evaluate_room_session_ready` system can lift the lobby into the game.
//!
//! Scope discipline (per PROMPT 1514 owned-scope):
//! - No heuristics beyond a deterministic class pick.
//! - No new protocol messages, no new C2S/S2C surface.
//! - No mutation of `ClassPreviews`, `PlayerSessions`, or any post-lobby state.
//! - No reach into `BotState.rng` here — class choice is foundation-flow only
//!   and must not consume audit-logged bot RNG entropy. Future per-round
//!   decisions remain free to use `BotState::rng` per ADR-005.
//!
//! Determinism contract: class choice depends only on the session id and the
//! slot index, so two servers replaying the same lobby pick the same class.

use bevy::prelude::*;
use shared::card::ClassId;

use crate::core::session::{
    ClassSelections, LobbyState, RoomSessions, SessionId, SessionSystemSet,
};
use crate::feature::bot::state::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotPlayers, BotState,
};
use shared::protocol::RoundPhase;
use shared::session::PlayerId;

/// Closed set of legal bot class picks. `ClassId::Neutral` is excluded because
/// `confirm_class` already rejects it; mirroring that contract here keeps the
/// stub loop in sync with the human path.
const BOT_CLASS_CHOICES: [ClassId; 6] = [
    ClassId::Iop,
    ClassId::Cra,
    ClassId::Sacrier,
    ClassId::Xelor,
    ClassId::Ecaflip,
    ClassId::Sadida,
];

/// Pick a class for a bot occupying `slot_index` inside `session_id`.
///
/// Pure function. Two calls with the same arguments always return the same
/// class. The current scheme folds the session UUID into a `u64` and adds the
/// slot index so a 2v2 room with two bots picks two distinct (but stable)
/// classes for adjacent slot indices.
pub fn deterministic_class_for_bot(session_id: SessionId, slot_index: u8) -> ClassId {
    let raw = session_id.0.as_u128();
    let folded = (raw as u64) ^ ((raw >> 64) as u64);
    let bucket = folded
        .wrapping_add(u64::from(slot_index))
        .rem_euclid(BOT_CLASS_CHOICES.len() as u64) as usize;
    BOT_CLASS_CHOICES[bucket]
}

/// Derive the bot's RNG seed from its synthetic player id. Mirrors the
/// foundation contract in `feature::bot::state` (PROMPT 1423 §3.1) where the
/// seed is derived externally; here we use the synthetic id directly so the
/// seed is reproducible across server restarts of the same session.
fn bot_seed_for(bot_player_id: PlayerId) -> u64 {
    bot_player_id.0
}

/// System: for every bot slot in a `LobbyWaiting` room, auto-confirm a
/// deterministic class so the existing lobby readiness gate can fire.
///
/// Runs before `evaluate_room_session_ready` so a freshly-added bot can lift
/// the room into `GameActive` in the same Update tick as the human's confirm.
///
/// `ClassSelections` is removed during session teardown / game-over, so we
/// accept `Option<ResMut<…>>` and bail early when the resource is absent.
/// This mirrors the pattern used in `core::session::system` and prevents the
/// panic observed in the 2026-05-27 bot-vs-bot soak run.
pub fn bot_lobby_auto_confirm(
    time: Res<Time>,
    mut rooms: ResMut<RoomSessions>,
    mut selections: Option<ResMut<ClassSelections>>,
    mut bots: ResMut<BotPlayers>,
    mut decision_log: ResMut<BotDecisionLog>,
) {
    let Some(ref mut selections) = selections else {
        return;
    };
    let now_ms = (time.elapsed().as_secs_f64() * 1_000.0) as u64;

    for session_id in rooms.session_ids() {
        let Some(session) = rooms.get_mut(session_id) else {
            continue;
        };
        if session.state != LobbyState::LobbyWaiting {
            continue;
        }

        for slot in session.slots.0.iter_mut() {
            if !slot.is_bot {
                continue;
            }
            let Some(bot_player_id) = slot.player else {
                continue;
            };
            if slot.class.is_some() {
                continue;
            }
            if selections.0.contains_key(&bot_player_id) {
                continue;
            }

            let class_id = deterministic_class_for_bot(session_id, slot.index);
            slot.class = Some(class_id);
            selections.0.insert(bot_player_id, class_id);

            if !bots.contains(bot_player_id) {
                bots.insert(BotState::new(bot_player_id, bot_seed_for(bot_player_id)));
            }
            if let Some(state) = bots.get_mut(bot_player_id) {
                state.class_choice = Some(class_id);
                state.last_decision_at_ms = Some(now_ms);
            }

            let seed_snapshot = bots
                .get(bot_player_id)
                .map(|s| (s.rng_seed, s.rng_word_counter))
                .unwrap_or((bot_seed_for(bot_player_id), 0));
            decision_log.push(BotDecisionEntry {
                round_number: 0,
                phase: RoundPhase::Lobby,
                bot_player_id,
                decision: BotDecisionKind::ClassConfirmed,
                timestamp_ms: now_ms,
                legal_action_count: Some(BOT_CLASS_CHOICES.len() as u32),
                seed: seed_snapshot.0,
                seed_word_counter: seed_snapshot.1,
            });

            tracing::info!(
                session_id = ?session_id.0,
                slot_index = slot.index,
                bot_player_id = ?bot_player_id,
                class = ?class_id,
                "bot_lobby_auto_confirm: class confirmed for bot slot"
            );
        }
    }
}

/// Plugin: registers the bot lobby loop. Initialises [`BotPlayers`] and
/// [`BotDecisionLog`] (idempotent — `init_resource` is a no-op when already
/// present) so any binary that adds the plugin gets the full foundation
/// without forcing every test harness to init them by hand.
pub struct BotLobbyPlugin;

impl Plugin for BotLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BotPlayers>()
            .init_resource::<BotDecisionLog>()
            .add_systems(
                Update,
                bot_lobby_auto_confirm
                    .in_set(SessionSystemSet::LobbyEval)
                    .before(crate::core::session::evaluate_room_session_ready),
            );
    }
}
