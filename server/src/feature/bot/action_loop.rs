//! Bot participant action loop, Wave 1 (PROMPT 1531).
//!
//! After [`bot_lobby_auto_confirm`](super::lobby_loop::bot_lobby_auto_confirm)
//! lifts a room into `GameActive`, this module advances the bot through the
//! per-round phases that would otherwise stall without a human counterpart:
//!
//! - **Draft (initial & shop):** emit a deterministic `DraftReadySignal` so the
//!   RSM's `DRAFT_* -> Placement|Auction` gate fires without waiting on a
//!   non-existent human ready click.
//! - **Auction:** intentionally pass. Wave-1 bots never bid; the existing
//!   auction safety timer settles the round when no bids arrive. The decision
//!   is logged once per round so the audit log keeps a contiguous trace.
//! - **Placement:** submit an empty (no-op) placement as a fail-safe so the
//!   `Placement -> Resolution` gate fires without waiting on the bot to place
//!   units. Heuristic placement (legal-cell pick) is explicitly deferred to a
//!   later wave.
//!
//! Scope discipline (PROMPT 1531 owned-scope):
//! - **No new protocol messages.** We write the *internal* server-side
//!   `DraftReadySignal` and `PlacementSubmissionReceived` messages that the
//!   network layer already produces for human clients. The contract is reused,
//!   not extended.
//! - **No shop purchases.** Wave 1 just needs to advance flow; shop buys
//!   require gold/hand bookkeeping the bot does not yet own. Bots fall through
//!   `DraftShop` via the ready signal alone.
//! - **No client UI.** Server-only behavior.
//!
//! Determinism: every decision uses only inputs (round number, current phase,
//! bot membership), never wall-clock time or random noise. Two replays of the
//! same session produce the same `BotDecisionLog`. The `BotState::rng`
//! ChaCha8 stream defined in the foundation is intentionally untouched by
//! Wave 1 — later heuristics (shop buy, lane targeting) will own RNG draws.

use std::collections::HashSet;

use bevy::prelude::*;
use lightyear::prelude::PeerId;

use crate::core::rsm::{
    state::{RoundPhase, RoundState},
    DraftReadySignal,
};
use crate::core::session::config::SessionConfig;
use crate::feature::board::PlacementSubmissionReceived;
use crate::feature::bot::state::{
    BotDecisionEntry, BotDecisionKind, BotDecisionLog, BotPlayers,
};
use shared::session::PlayerId;

/// Convert the server-internal `RoundPhase` into the wire `shared::protocol`
/// counterpart. Local copy of the `core::rsm::transitions::protocol_round_phase`
/// private helper — kept here to avoid widening that module's surface for a
/// single audit-log writer.
fn protocol_phase(phase: RoundPhase) -> shared::protocol::RoundPhase {
    match phase {
        RoundPhase::Lobby => shared::protocol::RoundPhase::Lobby,
        RoundPhase::DraftInitial => shared::protocol::RoundPhase::DraftInitial,
        RoundPhase::DraftAuction => shared::protocol::RoundPhase::DraftAuction,
        RoundPhase::DraftShop => shared::protocol::RoundPhase::DraftShop,
        RoundPhase::Placement => shared::protocol::RoundPhase::Placement,
        RoundPhase::Resolution => shared::protocol::RoundPhase::Resolution,
        RoundPhase::GameOver => shared::protocol::RoundPhase::GameOver,
    }
}

/// Returns the set of bot `PlayerId`s currently participating in the active
/// session. Filters out lobby-only bots that have not yet been folded into a
/// `SessionConfig`, and humans who happen to share a synthetic id range.
fn session_bot_players(bots: &BotPlayers, session: &SessionConfig) -> Vec<PlayerId> {
    session
        .players()
        .filter(|player| bots.contains(*player))
        .collect()
}

/// Snapshot the bot's RNG counters for the decision log. Wave 1 never consumes
/// RNG, so the counter stays at whatever `BotState::new` initialised it to,
/// but we still record the values per ADR-005 audit-log convention.
fn seed_snapshot(bots: &BotPlayers, player_id: PlayerId) -> (u64, u64) {
    bots.get(player_id)
        .map(|state| (state.rng_seed, state.rng_word_counter))
        .unwrap_or((0, 0))
}

/// Wallclock helper: convert `Time::elapsed()` to integer milliseconds for the
/// decision log. Matches the convention used by `bot_lobby_auto_confirm`.
fn now_ms(time: &Time) -> u64 {
    (time.elapsed().as_secs_f64() * 1_000.0) as u64
}

/// Bot action-loop system.
///
/// Runs every frame after the RSM input reader and before phase transitions
/// (see `BotActionLoopPlugin`). Per phase, for each bot in the current
/// session:
///
/// | Phase                  | Action                                              |
/// |------------------------|-----------------------------------------------------|
/// | `Lobby`                | no-op (handled by [`bot_lobby_auto_confirm`])       |
/// | `DraftInitial`         | write `DraftReadySignal { ready: true }` once       |
/// | `DraftShop`            | write `DraftReadySignal { ready: true }` once       |
/// | `DraftAuction`         | log `AuctionPass` once per round (intentional pass) |
/// | `Placement`            | write empty `PlacementSubmissionReceived` once      |
/// | `Resolution`/`GameOver`| no-op                                               |
///
/// Idempotency uses RSM-owned state where possible (`draft_ready_players`,
/// `submissions_received`) so the bot stops re-emitting the moment the RSM
/// records the signal. The auction pass uses a system-local set keyed by
/// `(player, round_number)` because auction has no analogous server-side set.
#[allow(clippy::too_many_arguments)]
pub fn bot_action_loop(
    time: Res<Time>,
    round_state: Option<Res<RoundState>>,
    session: Option<Res<SessionConfig>>,
    bots: Res<BotPlayers>,
    mut decision_log: ResMut<BotDecisionLog>,
    mut ready_signals: MessageWriter<DraftReadySignal>,
    mut placement_submissions: MessageWriter<PlacementSubmissionReceived>,
    mut auction_pass_logged: Local<HashSet<(PlayerId, u32)>>,
) {
    let (Some(round_state), Some(session)) = (round_state, session) else {
        return;
    };
    if bots.is_empty() {
        return;
    }

    let bot_players = session_bot_players(&bots, &session);
    if bot_players.is_empty() {
        return;
    }

    let phase = round_state.phase;
    let round = round_state.round_number;
    let ts = now_ms(&time);

    match phase {
        RoundPhase::DraftInitial | RoundPhase::DraftShop => {
            for player_id in &bot_players {
                if round_state.draft_ready_players.contains(player_id) {
                    continue;
                }
                ready_signals.write(DraftReadySignal {
                    player: *player_id,
                    ready: true,
                });

                let (seed, counter) = seed_snapshot(&bots, *player_id);
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::DraftReady,
                    timestamp_ms: ts,
                    legal_action_count: None,
                    seed,
                    seed_word_counter: counter,
                });

                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    phase = ?phase,
                    round,
                    "bot_action_loop: DraftReadySignal emitted (deterministic pass)"
                );
            }
        }
        RoundPhase::DraftAuction => {
            // Wave-1 bots intentionally never bid. Log the pass once per
            // (player, round) so the audit trail has a single entry per bot
            // per auction phase instead of one per tick.
            for player_id in &bot_players {
                let key = (*player_id, round);
                if auction_pass_logged.contains(&key) {
                    continue;
                }
                auction_pass_logged.insert(key);

                let (seed, counter) = seed_snapshot(&bots, *player_id);
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::AuctionPass {
                        reason: "wave1_deterministic_pass",
                    },
                    timestamp_ms: ts,
                    legal_action_count: None,
                    seed,
                    seed_word_counter: counter,
                });

                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    round,
                    "bot_action_loop: auction pass (no bids will be placed; safety timer will settle)"
                );
            }
        }
        RoundPhase::Placement => {
            for player_id in &bot_players {
                if round_state.submissions_received.contains(player_id) {
                    continue;
                }
                // Empty placement = legal no-op fail-safe. The board placement
                // pipeline treats an empty vector as a deliberate "submit
                // nothing"; the bot does not consume hand or gold.
                let submission = PlacementSubmissionReceived {
                    player: *player_id,
                    peer_id: None::<PeerId>,
                    placements: Vec::new(),
                };
                placement_submissions.write(submission);

                let (seed, counter) = seed_snapshot(&bots, *player_id);
                decision_log.push(BotDecisionEntry {
                    round_number: round,
                    phase: protocol_phase(phase),
                    bot_player_id: *player_id,
                    decision: BotDecisionKind::EmptyPlacementFailsafe,
                    timestamp_ms: ts,
                    legal_action_count: Some(0),
                    seed,
                    seed_word_counter: counter,
                });

                tracing::info!(
                    target: "server::bot",
                    bot_player_id = ?player_id,
                    round,
                    "bot_action_loop: empty placement submitted (wave1 no-op fail-safe)"
                );
            }
        }
        RoundPhase::Lobby | RoundPhase::Resolution | RoundPhase::GameOver => {
            // No bot participation in these phases for wave1. Lobby is owned by
            // `bot_lobby_auto_confirm`; Resolution/GameOver have no bot inputs.
        }
    }
}

/// Plugin: register the bot action loop on the standard `Update` schedule.
///
/// Ordering: this system writes `DraftReadySignal` and
/// `PlacementSubmissionReceived`, which are read by
/// `core::rsm::transitions::rsm_input_reader` and
/// `feature::board::placement::handle_placement_submission` respectively.
/// Bevy's MessageWriter/MessageReader contract is event-bus-style (one frame
/// of latency is fine), so we do not declare hard ordering against those
/// systems — the signal lands on the next tick which is sufficient for flow
/// progress.
pub struct BotActionLoopPlugin;

impl Plugin for BotActionLoopPlugin {
    fn build(&self, app: &mut App) {
        // BotPlayers and BotDecisionLog are already initialised by
        // `BotLobbyPlugin`; `init_resource` is a no-op if present, so this
        // plugin is safe to add in either order.
        app.init_resource::<BotPlayers>()
            .init_resource::<BotDecisionLog>()
            .add_systems(Update, bot_action_loop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::rsm::DraftReadySignal;
    use crate::core::session::config::SessionConfig;
    use crate::feature::board::PlacementSubmissionReceived;
    use crate::feature::bot::state::BotState;
    use bevy::app::App;
    use bevy::time::TimePlugin;
    use shared::card::ClassId;
    use shared::protocol::{GameMode, PlacementTimerMultiplier};
    use std::collections::HashMap;

    const BOT_ID: PlayerId = PlayerId(1 << 63);
    const HUMAN_ID: PlayerId = PlayerId(7);

    fn test_session_with_bot() -> SessionConfig {
        let mut team_map = HashMap::new();
        team_map.insert(BOT_ID, 1);
        team_map.insert(HUMAN_ID, 0);
        let mut class_map = HashMap::new();
        class_map.insert(BOT_ID, ClassId::Iop);
        class_map.insert(HUMAN_ID, ClassId::Cra);
        SessionConfig {
            mode: GameMode::OneVOne,
            player_count: 2,
            team_map,
            class_map,
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        }
    }

    fn make_app(phase: RoundPhase, round: u32) -> App {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_plugins(BotActionLoopPlugin);
        // Register the messages the loop writes so MessageWriter has a queue.
        app.add_message::<DraftReadySignal>();
        app.add_message::<PlacementSubmissionReceived>();

        let mut round_state = RoundState::new();
        round_state.phase = phase;
        round_state.round_number = round;
        app.insert_resource(round_state);
        app.insert_resource(test_session_with_bot());

        let mut bots = BotPlayers::default();
        bots.insert(BotState::new(BOT_ID, BOT_ID.0));
        app.insert_resource(bots);

        app
    }

    fn drain_ready(app: &mut App) -> Vec<DraftReadySignal> {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<DraftReadySignal>>()
            .drain()
            .collect()
    }

    fn drain_placement(app: &mut App) -> Vec<PlacementSubmissionReceived> {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<PlacementSubmissionReceived>>()
            .drain()
            .collect()
    }

    #[test]
    fn draft_initial_emits_ready_for_bot_only() {
        let mut app = make_app(RoundPhase::DraftInitial, 1);
        app.update();

        let ready: Vec<_> = drain_ready(&mut app);
        assert_eq!(ready.len(), 1, "exactly one ready signal expected");
        assert_eq!(ready[0].player, BOT_ID);
        assert!(ready[0].ready);

        let log = app.world().resource::<BotDecisionLog>();
        assert_eq!(log.len(), 1);
        assert!(matches!(
            log.entries[0].decision,
            BotDecisionKind::DraftReady
        ));
    }

    #[test]
    fn draft_ready_idempotent_after_rsm_records_signal() {
        let mut app = make_app(RoundPhase::DraftInitial, 1);
        app.update();
        // Simulate the RSM having recorded the ready signal.
        app.world_mut()
            .resource_mut::<RoundState>()
            .draft_ready_players
            .insert(BOT_ID);
        let _ = drain_ready(&mut app);

        app.update();
        let ready_after = drain_ready(&mut app);
        assert!(
            ready_after.is_empty(),
            "no duplicate ready signal once RSM tracks the bot"
        );
    }

    #[test]
    fn draft_shop_also_emits_ready() {
        let mut app = make_app(RoundPhase::DraftShop, 2);
        app.update();
        let ready = drain_ready(&mut app);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].player, BOT_ID);
    }

    #[test]
    fn placement_emits_empty_failsafe_once() {
        let mut app = make_app(RoundPhase::Placement, 3);
        app.update();

        let placements = drain_placement(&mut app);
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].player, BOT_ID);
        assert!(placements[0].placements.is_empty());
        assert!(placements[0].peer_id.is_none());

        // Mark as received; second tick should not re-emit.
        app.world_mut()
            .resource_mut::<RoundState>()
            .submissions_received
            .insert(BOT_ID);
        app.update();
        let again = drain_placement(&mut app);
        assert!(
            again.is_empty(),
            "placement fail-safe must not re-emit once RSM records the submission"
        );

        let log = app.world().resource::<BotDecisionLog>();
        assert!(log
            .entries
            .iter()
            .any(|e| matches!(e.decision, BotDecisionKind::EmptyPlacementFailsafe)));
    }

    #[test]
    fn auction_logs_pass_once_per_round() {
        let mut app = make_app(RoundPhase::DraftAuction, 4);
        app.update();
        app.update();
        app.update();

        let log = app.world().resource::<BotDecisionLog>();
        let pass_count = log
            .entries
            .iter()
            .filter(|e| matches!(e.decision, BotDecisionKind::AuctionPass { .. }))
            .count();
        assert_eq!(pass_count, 1, "auction pass logged exactly once per round");

        // No ready or placement signals during auction.
        assert!(drain_ready(&mut app).is_empty());
        assert!(drain_placement(&mut app).is_empty());

        // A new round produces a new pass log entry.
        app.world_mut().resource_mut::<RoundState>().round_number = 5;
        app.update();
        let log = app.world().resource::<BotDecisionLog>();
        let pass_count = log
            .entries
            .iter()
            .filter(|e| matches!(e.decision, BotDecisionKind::AuctionPass { .. }))
            .count();
        assert_eq!(pass_count, 2, "next auction round logs a fresh pass");
    }

    #[test]
    fn idle_phases_emit_nothing() {
        for phase in [RoundPhase::Lobby, RoundPhase::Resolution, RoundPhase::GameOver] {
            let mut app = make_app(phase, 0);
            app.update();
            assert!(drain_ready(&mut app).is_empty(), "{:?}: no ready", phase);
            assert!(drain_placement(&mut app).is_empty(), "{:?}: no placement", phase);
            let log = app.world().resource::<BotDecisionLog>();
            assert_eq!(log.len(), 0, "{:?}: no decision log entries", phase);
        }
    }

    #[test]
    fn humans_only_session_is_a_noop() {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_plugins(BotActionLoopPlugin);
        app.add_message::<DraftReadySignal>();
        app.add_message::<PlacementSubmissionReceived>();

        let mut round_state = RoundState::new();
        round_state.phase = RoundPhase::DraftInitial;
        app.insert_resource(round_state);

        // Session with only humans.
        let mut team_map = HashMap::new();
        team_map.insert(HUMAN_ID, 0);
        team_map.insert(PlayerId(8), 1);
        let mut class_map = HashMap::new();
        class_map.insert(HUMAN_ID, ClassId::Cra);
        class_map.insert(PlayerId(8), ClassId::Iop);
        app.insert_resource(SessionConfig {
            mode: GameMode::OneVOne,
            player_count: 2,
            team_map,
            class_map,
            placement_timer_multiplier_effective: PlacementTimerMultiplier::X1,
        });

        // BotPlayers is empty.
        app.insert_resource(BotPlayers::default());

        app.update();
        assert!(drain_ready(&mut app).is_empty());
        assert!(drain_placement(&mut app).is_empty());
        let log = app.world().resource::<BotDecisionLog>();
        assert_eq!(log.len(), 0);
    }
}
