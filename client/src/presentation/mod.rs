use ::shared::protocol::{
    CardSource, S2CCardAcquired, S2CDraftOffering, S2CGameSnapshot, S2CObjectiveIdentities,
    S2COpponentDisconnected, S2COpponentReconnected, S2CPhaseChanged, S2CPrismRespawned,
    S2CPrismRewardDropped, S2CSessionCancelled, S2CSessionSettingsUpdated, S2CShopSlots,
};
use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;

use crate::card_animations::PendingPhaseChange;
use crate::card_animations::{CardAnimationsPlugin, CardAnimationsSet};
use crate::presentation::board_rendering::{
    BoardRenderSet, BoardRenderState, PendingResolutionScript, ResolutionRevealWait,
};
use crate::presentation::connection_lost_overlay::ConnectionLostOverlayPlugin;
use crate::presentation::debug_bot_overlay::DebugBotOverlayPlugin;
use crate::presentation::qa_snapshot::QASnapshotPlugin;
use crate::presentation::result_screen::ResultScreenPlugin;
use crate::presentation::shared::economy_view::drain_gold_update_receiver_system as drain_shared_gold_update_receiver_system;
use crate::state::{
    apply_objective_identities_message, apply_opponent_disconnected_message,
    apply_opponent_reconnected_message, apply_phase_changed_message, apply_phase_view_message,
    apply_prism_respawned_message, apply_prism_reward_dropped_message,
    apply_session_cancelled_message, apply_session_settings_updated_message,
    apply_snapshot_to_session_settings_view, should_enter_session_from_phase,
    should_enter_session_from_snapshot, ClientGameSnapshotMessage, ClientIdempotencyPlugin,
    ClientObjectiveIdentities, ClientPhaseView, ClientSessionIdentity, ClientState,
    OpponentConnectionView, PrismLifecycleView, SessionLifecycleView, SessionSettingsView,
};
use crate::ui::hand::{
    HandUiCardAcquiredReceived, HandUiDraftOfferingReceived, HandUiPlugin, HandUiSystemSet,
};
use crate::ui::hud::{HudPlugin, HudSystemSet};
use crate::ui::photosensitivity_warning::PhotosensitivityWarningPlugin;
use crate::ui::settings::SettingsAccessibilityPlugin;
use crate::ui::shop_auction::{
    ShopAuctionCardAcquiredReceived, ShopAuctionDraftOfferingReceived,
    ShopAuctionShopCardAcquiredReceived, ShopAuctionShopSlotsReceived, ShopAuctionUiPlugin,
    ShopAuctionUiSystemSet,
};

pub mod board_rendering;
pub mod connection_lost_overlay;
pub mod debug_bot_overlay;
pub mod qa_snapshot;
pub mod result_screen;
pub mod shared;

pub use crate::presentation::board_rendering::{BoardRenderingPlugin, CardAtlas};
pub use crate::presentation::shared::economy_view::{
    apply_snapshot_to_player_economy_view, project_mana_after_spend, PlayerEconomyView,
    PlayerEconomyViewUpdateSource, PresentationGameSnapshotMessage,
};
pub use crate::state::CurrentClientPhase;
pub use crate::ui::shared::{BoardLayout, LaneCell};

pub struct PresentationPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresentationSet {
    PhaseTransition,
    MessageDrain,
    StateSync,
    AnimationTick,
}

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        tracing::info!("PresentationPlugin loaded");
        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<ClientPhaseView>()
            .init_resource::<ClientSessionIdentity>()
            .init_resource::<SessionSettingsView>()
            .init_resource::<PlayerEconomyView>()
            .init_resource::<ClientObjectiveIdentities>()
            .init_resource::<OpponentConnectionView>()
            .init_resource::<PrismLifecycleView>()
            .init_resource::<SessionLifecycleView>()
            .add_message::<ClientGameSnapshotMessage>();

        // ADR-021 registration order is a contract.
        // S13-LATE-MSG-DEDUPE-001: Install the dedupe resource and the
        // OnExit(InSession) reset before any drain plugin so consumers see a
        // ready resource at first build.
        app.add_plugins(ClientIdempotencyPlugin);
        app.add_plugins(CardAnimationsPlugin);
        app.add_plugins(BoardRenderingPlugin);
        // Sprint 18 story 020 (S18-UI-PLAY-AREA-CONTAINER-001) — `PlayArea`
        // is the canonical flex parent for the in-session middle band.
        // Registered ahead of `HandUiPlugin` / `ShopAuctionUiPlugin` so the
        // `PlayAreaRoot` resource is inserted at `OnEnter(InSession)`
        // before consumer spawn systems run; consumer plugins chain their
        // spawn systems with `.after(PlayAreaSpawnSet)`.
        app.add_plugins(crate::ui::PlayAreaPlugin);
        app.add_plugins(HandUiPlugin);
        app.add_plugins(HudPlugin);
        app.add_plugins(ShopAuctionUiPlugin);
        // PROMPT 1404 / `S19-UI-PHASE-CHANGE-BANNER-001` — transient
        // centered banner overlay on every major `RoundPhase` transition.
        // Registered after the gameplay UI roots so the banner paints
        // above HUD / hand / board (UI_OVERLAY layer) but below modals
        // (result screen / photosensitivity warning at MODAL layer).
        app.add_plugins(crate::ui::PhaseBannerPlugin);
        app.add_plugins(ResultScreenPlugin);
        // S13-CONN-LOST-UX-001 (Story 021): proactive Reconnecting / Connection
        // Lost overlay registered after ResultScreenPlugin per ADR-021.
        // Z-ordering (90 vs result screen 100) keeps the result screen on top
        // if GameOver lands while the overlay is up.
        app.add_plugins(ConnectionLostOverlayPlugin);
        app.add_plugins(SettingsAccessibilityPlugin);
        app.add_plugins(PhotosensitivityWarningPlugin);
        // PROMPT 1013 — QA snapshot overlay (disabled by default; activated
        // by `CCGS_QA_SNAPSHOT=1` env var on native, or by pre-inserting a
        // `QASnapshotConfig { enabled: true, .. }` resource in tests).
        // When disabled, the plugin spawns no UI and adds no per-frame work
        // beyond the inert system registrations.
        app.add_plugins(QASnapshotPlugin);
        // PROMPT 1614 — debug-only bot god-mode overlay (F8 toggle, env-gated
        // by `CCGS_DEBUG_UI=1`). Receives `S2CDebugBotStatePush` from the
        // server (which itself is gated by `CCGS_BOT_DEBUG_UI=1`) and renders
        // a non-interactive corner panel. When the env gate is off the plugin
        // spawns no UI and runs no per-frame systems.
        app.add_plugins(DebugBotOverlayPlugin);

        app.configure_sets(
            Update,
            (
                PresentationSet::PhaseTransition,
                PresentationSet::MessageDrain,
                PresentationSet::StateSync,
                PresentationSet::AnimationTick,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (
                HudSystemSet::PhaseTransition.in_set(PresentationSet::PhaseTransition),
                HudSystemSet::MessageDrain.in_set(PresentationSet::MessageDrain),
                HudSystemSet::StateSync.in_set(PresentationSet::StateSync),
                HandUiSystemSet::PhaseTransition.in_set(PresentationSet::PhaseTransition),
                HandUiSystemSet::MessageDrain.in_set(PresentationSet::MessageDrain),
                HandUiSystemSet::StateSync.in_set(PresentationSet::StateSync),
                ShopAuctionUiSystemSet::PhaseTransition.in_set(PresentationSet::PhaseTransition),
                ShopAuctionUiSystemSet::MessageDrain
                    .in_set(PresentationSet::MessageDrain)
                    .after(HudSystemSet::MessageDrain),
                ShopAuctionUiSystemSet::StateSync.in_set(PresentationSet::StateSync),
                CardAnimationsSet::React
                    .in_set(PresentationSet::MessageDrain)
                    .run_if(in_state(ClientState::InSession)),
            ),
        )
        .add_systems(
            Update,
            phase_sink_system
                .in_set(PresentationSet::PhaseTransition)
                .before(HudSystemSet::PhaseTransition)
                .before(HandUiSystemSet::PhaseTransition)
                .before(ShopAuctionUiSystemSet::PhaseTransition),
        )
        .add_systems(
            Update,
            (
                session_settings_sink_system,
                game_snapshot_sink_system,
                drain_shared_gold_update_receiver_system,
                draft_shop_hand_bridge_fanout_system,
                drain_objective_identities_system,
                drain_opponent_connection_messages,
                drain_prism_lifecycle_messages,
                drain_session_lifecycle_messages,
            )
                .in_set(PresentationSet::MessageDrain)
                .before(BoardRenderSet::ReadMessages)
                .before(HudSystemSet::MessageDrain)
                .before(HandUiSystemSet::MessageDrain)
                .before(ShopAuctionUiSystemSet::MessageDrain),
        );
    }
}

pub fn drain_objective_identities_system(
    mut receivers: Query<&mut MessageReceiver<S2CObjectiveIdentities>>,
    mut identities: ResMut<ClientObjectiveIdentities>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                count = message.identities.len(),
                msg_type = "S2CObjectiveIdentities",
                "drain_objective_identities: recv"
            );
            apply_objective_identities_message(&message, &mut identities);
        }
    }
}

pub fn phase_sink_system(
    mut receivers: Query<&mut MessageReceiver<S2CPhaseChanged>>,
    mut current: ResMut<CurrentClientPhase>,
    mut phase_view: ResMut<ClientPhaseView>,
    render_state: Option<Res<BoardRenderState>>,
    pending_script: Option<Res<PendingResolutionScript>>,
    reveal_wait: Option<Res<ResolutionRevealWait>>,
    mut pending_phase: Option<ResMut<PendingPhaseChange>>,
    identity: Res<ClientSessionIdentity>,
    state: Res<State<ClientState>>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    // BUG-01/BUG-13 diagnostic: warn when connected (player_id assigned) but no
    // MessageReceiver<S2CPhaseChanged> entity exists.  In that state S2CPhaseChanged
    // can never reach CurrentClientPhase — the cause is either a missing
    // Lightyear register_message / add_direction call or a connection entity that
    // lost the component.  Before handshake the absence is normal and is silent.
    if receivers.is_empty() && identity.player_id.is_some() {
        tracing::warn!(
            target: "client::presentation",
            player_id = ?identity.player_id,
            "phase_sink: connected but no MessageReceiver<S2CPhaseChanged> entity — \
             phase changes cannot reach CurrentClientPhase (BUG-01/BUG-13 site)"
        );
    }

    let mut messages = Vec::new();
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                phase = ?message.phase,
                round_number = message.round_number,
                timer_duration_ms = message.timer_duration_ms,
                msg_type = "S2CPhaseChanged",
                "phase_sink: recv"
            );
            messages.push(message);
        }
    }

    if *state.get() == ClientState::Lobby
        && messages
            .iter()
            .any(|message| should_enter_session_from_phase(&identity, message.phase))
    {
        next_state.set(ClientState::InSession);
    }

    if messages.is_empty() {
        return;
    }

    apply_phase_changed_messages_with_resolution_gate(
        messages,
        &mut current,
        &mut phase_view,
        render_state.as_deref(),
        pending_script.as_deref(),
        reveal_wait.as_deref(),
        pending_phase.as_deref_mut(),
    );
}

pub fn apply_phase_changed_messages(
    messages: impl IntoIterator<Item = S2CPhaseChanged>,
    current: &mut CurrentClientPhase,
) {
    for message in messages {
        apply_phase_changed_message(message, current);
    }
}

pub fn apply_phase_changed_messages_with_resolution_gate(
    messages: impl IntoIterator<Item = S2CPhaseChanged>,
    current: &mut CurrentClientPhase,
    phase_view: &mut ClientPhaseView,
    render_state: Option<&BoardRenderState>,
    pending_script: Option<&PendingResolutionScript>,
    reveal_wait: Option<&ResolutionRevealWait>,
    mut pending_phase: Option<&mut PendingPhaseChange>,
) {
    for message in messages {
        if should_buffer_phase_change(render_state, pending_script, reveal_wait)
            && pending_phase.is_some()
        {
            if let Some(pending_phase) = pending_phase.as_deref_mut() {
                pending_phase.set(message);
            }
            continue;
        }

        apply_phase_changed_message(message.clone(), current);
        apply_phase_view_message(&message, phase_view);
    }
}

fn should_buffer_phase_change(
    render_state: Option<&BoardRenderState>,
    pending_script: Option<&PendingResolutionScript>,
    reveal_wait: Option<&ResolutionRevealWait>,
) -> bool {
    if render_state == Some(&BoardRenderState::ResolutionExecuting) {
        return true;
    }

    let pending_resolution_script = pending_script.is_some_and(PendingResolutionScript::is_some);
    let reveal_handoff_active = render_state == Some(&BoardRenderState::ResolutionReveal)
        || reveal_wait.is_some_and(ResolutionRevealWait::is_active);

    pending_resolution_script && reveal_handoff_active
}

pub fn session_settings_sink_system(
    mut receivers: Query<&mut MessageReceiver<S2CSessionSettingsUpdated>>,
    mut settings_view: ResMut<SessionSettingsView>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                placement_timer_multiplier_effective = ?message.placement_timer_multiplier_effective,
                msg_type = "S2CSessionSettingsUpdated",
                "session_settings_sink: recv"
            );
            apply_session_settings_updated_message(&message, &mut settings_view);
        }
    }
}

pub fn game_snapshot_sink_system(
    mut receivers: Query<&mut MessageReceiver<S2CGameSnapshot>>,
    mut economy_view: ResMut<PlayerEconomyView>,
    mut settings_view: ResMut<SessionSettingsView>,
    mut board_writer: MessageWriter<ClientGameSnapshotMessage>,
    mut presentation_writer: MessageWriter<PresentationGameSnapshotMessage>,
    identity: Res<ClientSessionIdentity>,
    state: Res<State<ClientState>>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                player_id = ?message.recipient_player_id,
                phase = ?message.phase,
                round_number = message.round_number,
                timer_remaining_ms = ?message.timer_remaining_ms,
                players_len = message.players.len(),
                msg_type = "S2CGameSnapshot",
                "game_snapshot_sink: recv"
            );
            if *state.get() == ClientState::Lobby
                && should_enter_session_from_snapshot(&identity, &message)
            {
                next_state.set(ClientState::InSession);
            }
            if !apply_snapshot_to_player_economy_view(&message, &mut economy_view) {
                warn!(
                    "Presentation: snapshot for {:?} does not contain the local player economy",
                    message.recipient_player_id
                );
            }
            apply_snapshot_to_session_settings_view(&message, &mut settings_view);
            board_writer.write(ClientGameSnapshotMessage(message.clone()));
            presentation_writer.write(PresentationGameSnapshotMessage(message));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftOfferingFanout {
    pub hand: HandUiDraftOfferingReceived,
    pub shop: ShopAuctionDraftOfferingReceived,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardAcquiredFanout {
    pub hand: HandUiCardAcquiredReceived,
    pub draft_initial: Option<ShopAuctionCardAcquiredReceived>,
    pub shop_purchase: Option<ShopAuctionShopCardAcquiredReceived>,
}

pub fn draft_offering_fanout_messages(message: S2CDraftOffering) -> DraftOfferingFanout {
    DraftOfferingFanout {
        hand: HandUiDraftOfferingReceived {
            card_ids: message.card_ids.clone(),
        },
        shop: ShopAuctionDraftOfferingReceived {
            card_ids: message.card_ids,
        },
    }
}

pub fn shop_slots_message(message: S2CShopSlots) -> ShopAuctionShopSlotsReceived {
    ShopAuctionShopSlotsReceived {
        slots: message.slots,
    }
}

pub fn card_acquired_fanout_messages(message: S2CCardAcquired) -> CardAcquiredFanout {
    let hand = HandUiCardAcquiredReceived {
        card_id: message.card_id,
    };
    let (draft_initial, shop_purchase) = match message.source {
        CardSource::DraftInitial => (
            Some(ShopAuctionCardAcquiredReceived {
                card_id: message.card_id,
            }),
            None,
        ),
        CardSource::ShopPurchase => (
            None,
            Some(ShopAuctionShopCardAcquiredReceived {
                card_id: message.card_id,
            }),
        ),
        _ => (None, None),
    };

    CardAcquiredFanout {
        hand,
        draft_initial,
        shop_purchase,
    }
}

pub fn drain_opponent_connection_messages(
    mut disconnected_receivers: Query<&mut MessageReceiver<S2COpponentDisconnected>>,
    mut reconnected_receivers: Query<&mut MessageReceiver<S2COpponentReconnected>>,
    mut view: ResMut<OpponentConnectionView>,
) {
    for mut receiver in &mut disconnected_receivers {
        for message in receiver.receive() {
            tracing::info!(
                target: "client::presentation",
                player_id = ?message.player_id,
                grace_remaining_ms = message.grace_remaining_ms,
                msg_type = "S2COpponentDisconnected",
                "drain_opponent_connection: recv"
            );
            apply_opponent_disconnected_message(&message, &mut view);
        }
    }

    for mut receiver in &mut reconnected_receivers {
        for message in receiver.receive() {
            tracing::info!(
                target: "client::presentation",
                player_id = ?message.player_id,
                msg_type = "S2COpponentReconnected",
                "drain_opponent_connection: recv"
            );
            apply_opponent_reconnected_message(&message, &mut view);
        }
    }
}

pub fn drain_prism_lifecycle_messages(
    mut respawned_receivers: Query<&mut MessageReceiver<S2CPrismRespawned>>,
    mut reward_dropped_receivers: Query<&mut MessageReceiver<S2CPrismRewardDropped>>,
    mut view: ResMut<PrismLifecycleView>,
) {
    for mut receiver in &mut respawned_receivers {
        for message in receiver.receive() {
            tracing::info!(
                target: "client::presentation",
                player_id = ?message.player_id,
                msg_type = "S2CPrismRespawned",
                "drain_prism_lifecycle: recv"
            );
            apply_prism_respawned_message(&message, &mut view);
        }
    }

    for mut receiver in &mut reward_dropped_receivers {
        for message in receiver.receive() {
            tracing::info!(
                target: "client::presentation",
                player_id = ?message.player_id,
                lane = message.lane,
                msg_type = "S2CPrismRewardDropped",
                "drain_prism_lifecycle: recv"
            );
            apply_prism_reward_dropped_message(&message, &mut view);
        }
    }
}

pub fn drain_session_lifecycle_messages(
    mut receivers: Query<&mut MessageReceiver<S2CSessionCancelled>>,
    mut view: ResMut<SessionLifecycleView>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            tracing::info!(
                target: "client::presentation",
                reason = ?message.reason,
                msg_type = "S2CSessionCancelled",
                "drain_session_lifecycle: recv"
            );
            apply_session_cancelled_message(&message, &mut view);
        }
    }
}

pub fn draft_shop_hand_bridge_fanout_system(
    mut draft_offering_receivers: Query<&mut MessageReceiver<S2CDraftOffering>>,
    mut shop_slots_receivers: Query<&mut MessageReceiver<S2CShopSlots>>,
    mut card_acquired_receivers: Query<&mut MessageReceiver<S2CCardAcquired>>,
    mut hand_offering_writer: MessageWriter<HandUiDraftOfferingReceived>,
    mut shop_offering_writer: MessageWriter<ShopAuctionDraftOfferingReceived>,
    mut shop_slots_writer: MessageWriter<ShopAuctionShopSlotsReceived>,
    mut hand_acquired_writer: MessageWriter<HandUiCardAcquiredReceived>,
    mut draft_acquired_writer: MessageWriter<ShopAuctionCardAcquiredReceived>,
    mut shop_acquired_writer: MessageWriter<ShopAuctionShopCardAcquiredReceived>,
) {
    for mut receiver in &mut draft_offering_receivers {
        for message in receiver.receive() {
            tracing::info!(
                count = message.card_ids.len(),
                msg_type = "S2CDraftOffering",
                "draft_shop_hand_bridge_fanout: recv"
            );
            let fanout = draft_offering_fanout_messages(message);
            hand_offering_writer.write(fanout.hand);
            shop_offering_writer.write(fanout.shop);
        }
    }

    for mut receiver in &mut shop_slots_receivers {
        for message in receiver.receive() {
            tracing::info!(
                slots_len = message.slots.len(),
                msg_type = "S2CShopSlots",
                "draft_shop_hand_bridge_fanout: recv"
            );
            shop_slots_writer.write(shop_slots_message(message));
        }
    }

    for mut receiver in &mut card_acquired_receivers {
        for message in receiver.receive() {
            tracing::info!(
                card_id = ?message.card_id,
                source = ?message.source,
                msg_type = "S2CCardAcquired",
                "draft_shop_hand_bridge_fanout: recv"
            );
            let fanout = card_acquired_fanout_messages(message);
            hand_acquired_writer.write(fanout.hand);
            if let Some(message) = fanout.draft_initial {
                draft_acquired_writer.write(message);
            }
            if let Some(message) = fanout.shop_purchase {
                shop_acquired_writer.write(message);
            }
        }
    }
}
