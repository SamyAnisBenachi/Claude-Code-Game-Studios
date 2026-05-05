use ::shared::protocol::{S2CGameSnapshot, S2CPhaseChanged};
use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;

use crate::card_animations::{CardAnimationsPlugin, CardAnimationsSet};
use crate::presentation::board_rendering::BoardRenderSet;
use crate::presentation::shared::economy_view::drain_gold_update_receiver_system as drain_shared_gold_update_receiver_system;
use crate::state::{
    apply_phase_changed_message, apply_phase_view_message, ClientGameSnapshotMessage,
    ClientPhaseView, ClientState,
};
use crate::ui::hand::{HandUiPlugin, HandUiSystemSet};
use crate::ui::hud::{HudPlugin, HudSystemSet};
use crate::ui::shop_auction::{ShopAuctionUiPlugin, ShopAuctionUiSystemSet};

pub mod board_rendering;
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
            .init_resource::<PlayerEconomyView>()
            .add_message::<ClientGameSnapshotMessage>();

        // ADR-021 registration order is a contract.
        app.add_plugins(CardAnimationsPlugin);
        app.add_plugins(BoardRenderingPlugin);
        app.add_plugins(HandUiPlugin);
        app.add_plugins(HudPlugin);
        app.add_plugins(ShopAuctionUiPlugin);

        app.configure_sets(
            Update,
            (
                PresentationSet::PhaseTransition,
                PresentationSet::MessageDrain,
                PresentationSet::StateSync,
                PresentationSet::AnimationTick,
            )
                .chain()
                .run_if(in_state(ClientState::InSession)),
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
                CardAnimationsSet::React.in_set(PresentationSet::MessageDrain),
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
                game_snapshot_sink_system,
                drain_shared_gold_update_receiver_system,
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
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            apply_phase_changed_message(message.clone(), &mut current);
            apply_phase_view_message(&message, &mut phase_view);
        }
    }
}

pub fn apply_phase_changed_messages(
    messages: impl IntoIterator<Item = S2CPhaseChanged>,
    current: &mut CurrentClientPhase,
) {
    for message in messages {
        apply_phase_changed_message(message, current);
    }
}

pub fn game_snapshot_sink_system(
    mut receivers: Query<&mut MessageReceiver<S2CGameSnapshot>>,
    mut economy_view: ResMut<PlayerEconomyView>,
    mut board_writer: MessageWriter<ClientGameSnapshotMessage>,
    mut presentation_writer: MessageWriter<PresentationGameSnapshotMessage>,
) {
    for mut receiver in &mut receivers {
        for message in receiver.receive() {
            if !apply_snapshot_to_player_economy_view(&message, &mut economy_view) {
                warn!(
                    "Presentation: snapshot for {:?} does not contain the local player economy",
                    message.recipient_player_id
                );
            }
            board_writer.write(ClientGameSnapshotMessage(message.clone()));
            presentation_writer.write(PresentationGameSnapshotMessage(message));
        }
    }
}
