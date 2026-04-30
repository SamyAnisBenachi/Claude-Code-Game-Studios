use super::events::{
    AuctionPhaseEntered, BroadcastPhaseChanged, DraftStarted, GameOverEmitted,
    PlacementPhaseEntered, ResolutionPhaseEntered, ShopRefreshNeeded,
};
use super::state::{PhaseAdvanceRequest, RoundPhase, RoundState};
use crate::core::session::SessionConfig;
use bevy::prelude::*;
use shared::protocol::DraftPhase;

pub fn is_auction_round(round_number: u32) -> bool {
    debug_assert!(
        round_number >= 1,
        "round_number must be initialized before auction routing"
    );
    round_number % 3 == 0
}

pub fn advance_phase(
    mut rsm: ResMut<RoundState>,
    request: Option<Res<PhaseAdvanceRequest>>,
    session: Option<Res<SessionConfig>>,
    config: Option<Res<crate::foundation::config::GameConfig>>,
    mut draft_started: MessageWriter<DraftStarted>,
    mut shop_refresh: MessageWriter<ShopRefreshNeeded>,
    mut auction_entered: MessageWriter<AuctionPhaseEntered>,
    mut placement_entered: MessageWriter<PlacementPhaseEntered>,
    mut resolution_entered: MessageWriter<ResolutionPhaseEntered>,
    mut game_over_emitted: MessageWriter<GameOverEmitted>,
    mut broadcast: MessageWriter<BroadcastPhaseChanged>,
) {
    let Some(request) = request else {
        return;
    };

    if rsm.phase != request.expected_source {
        return;
    }

    if let Some(game_over) = &request.game_over {
        rsm.phase = RoundPhase::GameOver;
        game_over_emitted.write(GameOverEmitted {
            reason: game_over.reason,
            loser: game_over.loser,
        });
        broadcast.write(BroadcastPhaseChanged {
            phase: RoundPhase::GameOver,
            round: rsm.round_number,
            timer_ms: 0,
        });
        return;
    }

    match rsm.phase {
        RoundPhase::Lobby => {
            rsm.phase = RoundPhase::DraftInitial;
            emit_draft_entry(
                &mut rsm,
                &session,
                &config,
                DraftPhase::Initial,
                &mut draft_started,
                &mut shop_refresh,
                None,
                &mut auction_entered,
                &mut broadcast,
            );
        }
        RoundPhase::DraftInitial => {
            rsm.phase = RoundPhase::Placement;
            rsm.submissions_received.clear();
            placement_entered.write(PlacementPhaseEntered {
                round: rsm.round_number,
            });
            broadcast.write(BroadcastPhaseChanged {
                phase: RoundPhase::Placement,
                round: rsm.round_number,
                timer_ms: seconds_to_ms(config.as_ref().map_or(0, |c| c.placement_timer_seconds)),
            });
        }
        RoundPhase::DraftAuction => {
            rsm.phase = RoundPhase::DraftShop;
            emit_draft_entry(
                &mut rsm,
                &session,
                &config,
                DraftPhase::Shop,
                &mut draft_started,
                &mut shop_refresh,
                None,
                &mut auction_entered,
                &mut broadcast,
            );
        }
        RoundPhase::DraftShop => {
            rsm.phase = RoundPhase::Placement;
            rsm.submissions_received.clear();
            placement_entered.write(PlacementPhaseEntered {
                round: rsm.round_number,
            });
            broadcast.write(BroadcastPhaseChanged {
                phase: RoundPhase::Placement,
                round: rsm.round_number,
                timer_ms: seconds_to_ms(config.as_ref().map_or(0, |c| c.placement_timer_seconds)),
            });
        }
        RoundPhase::Placement => {
            rsm.phase = RoundPhase::Resolution;
            resolution_entered.write(ResolutionPhaseEntered {
                round: rsm.round_number,
            });
            broadcast.write(BroadcastPhaseChanged {
                phase: RoundPhase::Resolution,
                round: rsm.round_number,
                timer_ms: seconds_to_ms(
                    config
                        .as_ref()
                        .map_or(0, |c| c.resolution_max_duration_seconds),
                ),
            });
        }
        RoundPhase::Resolution => {
            rsm.round_number += 1;
            debug_assert!(
                rsm.round_number >= 1,
                "round_number was not initialized before resolution exit"
            );
            let enters_auction = is_auction_round(rsm.round_number);
            let next_round = rsm.round_number;
            let draft_phase = if enters_auction {
                rsm.phase = RoundPhase::DraftAuction;
                DraftPhase::Auction
            } else {
                rsm.phase = RoundPhase::DraftShop;
                DraftPhase::Shop
            };
            emit_draft_entry(
                &mut rsm,
                &session,
                &config,
                draft_phase,
                &mut draft_started,
                &mut shop_refresh,
                enters_auction.then_some(next_round),
                &mut auction_entered,
                &mut broadcast,
            );
        }
        RoundPhase::GameOver => {
            return;
        }
    }
}

fn emit_draft_entry(
    rsm: &mut RoundState,
    session: &Option<Res<SessionConfig>>,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
    draft_phase: DraftPhase,
    draft_started: &mut MessageWriter<DraftStarted>,
    shop_refresh: &mut MessageWriter<ShopRefreshNeeded>,
    auction_round: Option<u32>,
    auction_entered: &mut MessageWriter<AuctionPhaseEntered>,
    broadcast: &mut MessageWriter<BroadcastPhaseChanged>,
) {
    draft_started.write(DraftStarted {
        round: rsm.round_number,
        phase: draft_phase,
    });

    if let Some(session) = session {
        for player in session.players() {
            shop_refresh.write(ShopRefreshNeeded { player });
        }
    }

    if let Some(round) = auction_round {
        auction_entered.write(AuctionPhaseEntered { round });
    }

    broadcast.write(BroadcastPhaseChanged {
        phase: rsm.phase,
        round: rsm.round_number,
        timer_ms: draft_timer_ms(draft_phase, config),
    });
}

fn draft_timer_ms(
    draft_phase: DraftPhase,
    config: &Option<Res<crate::foundation::config::GameConfig>>,
) -> u32 {
    let Some(config) = config else {
        return 0;
    };
    match draft_phase {
        DraftPhase::Initial => seconds_to_ms(config.draft_initial_timer_seconds),
        DraftPhase::Auction => 0,
        DraftPhase::Shop => seconds_to_ms(config.draft_shop_timer_seconds),
    }
}

fn seconds_to_ms(seconds: u32) -> u32 {
    seconds.saturating_mul(1000)
}
