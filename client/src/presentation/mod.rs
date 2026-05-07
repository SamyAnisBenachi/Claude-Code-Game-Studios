use ::shared::protocol::{
    CardSource, S2CCardAcquired, S2CDraftOffering, S2CGameSnapshot, S2CPhaseChanged,
    S2CSessionSettingsUpdated, S2CShopSlots,
};
use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;

use crate::card_animations::PendingPhaseChange;
use crate::card_animations::{CardAnimationsPlugin, CardAnimationsSet};
use crate::presentation::board_rendering::{
    BoardRenderSet, BoardRenderState, PendingResolutionScript, ResolutionRevealWait,
};
use crate::presentation::result_screen::ResultScreenPlugin;
use crate::presentation::shared::economy_view::drain_gold_update_receiver_system as drain_shared_gold_update_receiver_system;
use crate::state::{
    apply_phase_changed_message, apply_phase_view_message, apply_session_settings_updated_message,
    apply_snapshot_to_session_settings_view, should_enter_session_from_phase,
    should_enter_session_from_snapshot, ClientGameSnapshotMessage, ClientPhaseView,
    ClientSessionIdentity, ClientState, SessionSettingsView,
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
pub mod result_screen;
pub mod shared;

pub use crate::presentation::board_rendering::{BoardRenderingPlugin, CardAtlas};
pub use crate::presentation::shared::economy_view::{
    apply_snapshot_to_player_economy_view, PlayerEconomyView, PlayerEconomyViewUpdateSource,
    PresentationGameSnapshotMessage,
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
        app.init_state::<ClientState>()
            .init_resource::<CurrentClientPhase>()
            .init_resource::<ClientPhaseView>()
            .init_resource::<ClientSessionIdentity>()
            .init_resource::<SessionSettingsView>()
            .init_resource::<PlayerEconomyView>()
            .add_message::<ClientGameSnapshotMessage>();

        // ADR-021 registration order is a contract.
        app.add_plugins(CardAnimationsPlugin);
        app.add_plugins(BoardRenderingPlugin);
        app.add_plugins(HandUiPlugin);
        app.add_plugins(HudPlugin);
        app.add_plugins(ShopAuctionUiPlugin);
        app.add_plugins(ResultScreenPlugin);
        app.add_plugins(SettingsAccessibilityPlugin);
        app.add_plugins(PhotosensitivityWarningPlugin);

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
            )
                .in_set(PresentationSet::MessageDrain)
                .before(BoardRenderSet::ReadMessages)
                .before(HudSystemSet::MessageDrain)
                .before(HandUiSystemSet::MessageDrain)
                .before(ShopAuctionUiSystemSet::MessageDrain),
        );
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
    let mut messages = Vec::new();
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
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
            let fanout = draft_offering_fanout_messages(message);
            hand_offering_writer.write(fanout.hand);
            shop_offering_writer.write(fanout.shop);
        }
    }

    for mut receiver in &mut shop_slots_receivers {
        for message in receiver.receive() {
            shop_slots_writer.write(shop_slots_message(message));
        }
    }

    for mut receiver in &mut card_acquired_receivers {
        for message in receiver.receive() {
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
