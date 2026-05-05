use ::shared::protocol::S2CPhaseChanged;
use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;

use crate::card_animations::{CardAnimationsPlugin, CardAnimationsSet};
use crate::presentation::shared::economy_view::{
    drain_game_snapshot_receiver_system as drain_shared_game_snapshot_receiver_system,
    drain_gold_update_receiver_system as drain_shared_gold_update_receiver_system,
};
use crate::state::{apply_phase_changed_message, ClientState};
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
            .init_resource::<PlayerEconomyView>();

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
                ShopAuctionUiSystemSet::MessageDrain.in_set(PresentationSet::MessageDrain),
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
                drain_shared_game_snapshot_receiver_system,
                drain_shared_gold_update_receiver_system,
            )
                .in_set(PresentationSet::MessageDrain)
                .before(HudSystemSet::MessageDrain)
                .before(HandUiSystemSet::MessageDrain)
                .before(ShopAuctionUiSystemSet::MessageDrain),
        );
    }
}

pub fn phase_sink_system(
    mut receivers: Query<&mut MessageReceiver<S2CPhaseChanged>>,
    mut current: ResMut<CurrentClientPhase>,
) {
    for mut receiver in &mut receivers {
        apply_phase_changed_messages(receiver.receive(), &mut current);
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
